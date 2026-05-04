use fixedbitset::FixedBitSet;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::Rng;
use rand::SeedableRng;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::hypergraph::Hypergraph;
use crate::observables::{
    component_radius, compute_coherence_raw, compute_omega, detect_candidate_knot_neighborhoods,
    InteractionEvent, TopologicalKnot,
};
use crate::physics_params::PhysicsParams;
use crate::rules::{edge_creation_rule, UndoRecord};

#[derive(serde::Serialize, Clone)]
pub struct DefectLogEntry {
    pub time: usize,
}

#[derive(serde::Serialize, Clone)]
pub struct XiCurrentLogEntry {
    pub time: usize,
    pub delta_xi: HashMap<u64, f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum ConservationMode {
    Baseline,
    Pairwise,
    StabilityScaled,
    FluxCompensated,
    MassCoupled,
    TimeSymmetry,
    Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmergenceMode {
    Control,  // Regime A: No stability bonus, hard threshold
    Assisted, // Regime B: Honest inheritance + Sigmoid threshold
    Forced,   // Regime C: Constant bonus + hard threshold
}

pub struct RewriteEngine {
    pub h: Hypergraph,
    pub p_create: f64,
    pub mode: EmergenceMode,
    pub pure_mode: bool,

    // Physics params
    pub gamma_time: f64,
    pub gamma_ext: f64,
    pub epsilon_label_violation: f64,
    pub params: PhysicsParams,

    // ξ field
    pub xi: HashMap<u64, f64>,
    pub prev_xi: HashMap<u64, f64>,
    pub xi_threshold: f64,
    pub xi_decay: f64,
    pub xi_coupling: f64,

    // cluster + geometry memory
    pub topo_distance_memory: HashMap<(String, usize, usize), f64>,
    pub xi_distance_memory: HashMap<(String, usize, usize), f64>,
    pub distance_memory_decay: f64,
    pub geometry_stride: usize,

    // knot tracking (Hypothesis 3)
    pub active_knots: HashMap<u64, TopologicalKnot>,
    pub next_knot_id: u64,
    pub dead_knots: Vec<TopologicalKnot>,
    pub interaction_events: Vec<InteractionEvent>,

    // logs
    pub rewrite_history: Vec<UndoRecord>, // Simplified for now
    pub xi_current_log: Vec<XiCurrentLogEntry>,
    pub defect_log: Vec<DefectLogEntry>,

    // rewrite bookkeeping
    pub last_rewrite: Option<UndoRecord>,
    pub cached_inter: Option<HashMap<u64, FixedBitSet>>,
    pub interaction_counts: HashMap<(u64, u64), u16>,
    pub forced_time: Option<usize>,
    pub time: usize,
    pub verbose: bool,
    pub print_interval: usize,

    // probes
    pending_bridge: Option<(u64, u64)>,
    pending_bridge_time: Option<usize>,

    last_step_time: f64,
    pub attempted_rewrites: usize,
    pub suppressed_rewrites: usize,

    // Per-vertex stability memory: accumulates for vertices in coherent structures
    pub stability: HashMap<u64, f64>,

    pub coupled_vertices: HashSet<u64>,
    pub active_interactions: HashMap<(u64, u64), InteractionEvent>,

    // Conservation Mode
    pub conservation_mode: ConservationMode,
    pub p_rev: f64,                            // For Hypothesis E
    pub momentum_reservoir: HashMap<u64, f64>, // For Hypothesis C

    // Observability
    pub thread_id: Option<usize>,
    pub max_steps: usize,
    pub rng: SmallRng,
}

impl RewriteEngine {
    pub fn new(h: Hypergraph, p_create: f64, seed: Option<u64>) -> Self {
        // Initialize deterministic SmallRng from seed or random entropy
        let rng = match seed {
            Some(s) => SmallRng::seed_from_u64(s),
            None => SmallRng::from_entropy(),
        };

        Self {
            h,
            p_create,
            gamma_time: 0.1,
            gamma_ext: 0.05,
            epsilon_label_violation: 0.08,
            params: PhysicsParams::new(),
            mode: EmergenceMode::Assisted, // Default to v5.1 Honest Assisted

            xi: HashMap::new(),
            prev_xi: HashMap::new(),
            xi_threshold: 1e-6,
            xi_decay: 0.70,
            xi_coupling: 0.6,

            topo_distance_memory: HashMap::new(),
            xi_distance_memory: HashMap::new(),
            distance_memory_decay: 0.9,
            geometry_stride: 5,

            active_knots: HashMap::new(),
            next_knot_id: 1,
            dead_knots: Vec::new(),
            interaction_events: Vec::new(),

            rewrite_history: Vec::new(),
            xi_current_log: Vec::new(),
            defect_log: Vec::new(),

            last_rewrite: None,
            interaction_counts: HashMap::new(),
            forced_time: None,
            time: 0,
            verbose: true,
            print_interval: 50,

            cached_inter: None,
            pending_bridge: None,
            pending_bridge_time: None,

            last_step_time: 0.0,
            attempted_rewrites: 0,
            suppressed_rewrites: 0,
            stability: HashMap::new(),
            coupled_vertices: HashSet::new(),
            active_interactions: HashMap::new(),
            pure_mode: false,

            conservation_mode: ConservationMode::Baseline,
            p_rev: 0.05,
            momentum_reservoir: HashMap::new(),
            thread_id: None,
            max_steps: 0,
            rng,
        }
    }

    // --------------------------------------------------
    // Main step
    // --------------------------------------------------
    pub fn step(&mut self) -> bool {
        self.time += 1;

        // Perform periodic memory scrub (safeguard against ghost bit expansion)
        if self.time % 100 == 0 {
            self.h.scrub_ghost_bits();

            // Status heartbeat print reduced to 1000 to keep console clean
            if self.time % 1000 == 0 {
                let tid = self.thread_id.unwrap_or(0);
                if self.max_steps > 0 {
                    println!("Thread {}: Step {}/{}", tid, self.time, self.max_steps);
                } else {
                    println!("Thread {}: Step {}...", tid, self.time);
                }
            }
        }

        let t0 = Instant::now();
        self.prev_xi = self.xi.clone();

        // --- HYPER-FLOW BOOTSTRAP (Step 1 only) ---
        if self.cached_inter.is_none() {
            let inter = crate::observables::worldline_interaction_graph(&self.h, 0.1);
            self.cached_inter = Some(inter);

            // Populate reference counts for the initial state
            for edge in self.h.hyperedges.values() {
                let wl_ids: Vec<u64> = edge
                    .vertices
                    .iter()
                    .filter(|&&id| {
                        self.h.vertices.get(&id).map_or(false, |v| {
                            v.depth >= (0.1 * self.h.max_chain_length() as f64) as usize
                        })
                    })
                    .copied()
                    .collect();
                for &i in &wl_ids {
                    for &j in &wl_ids {
                        if i < j {
                            *self.interaction_counts.entry((i, j)).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
        let mut inter = self.cached_inter.take().unwrap();

        // --- Hypothesis E: Interaction Time Symmetry (Stochastic Undo) ---
        if self.params.enable_conservation_patches
            && self.conservation_mode == ConservationMode::TimeSymmetry
        {
            let rng = &mut self.rng;

            // Check for high local momentum error threshold (0.5)
            let mut high_error = false;
            for knot in self.active_knots.values() {
                let p_before = knot.prev_momentum;
                let p_after = knot.momentum;
                let eps_p = (p_after - p_before).abs() / (p_before.abs() + 1e-6);
                if eps_p > 0.5 {
                    high_error = true;
                    break;
                }
            }

            // p_undo = 0.2 if high error exists
            let p_undo = if high_error { 0.2 } else { 0.05 };

            if rng.gen::<f64>() < p_undo {
                if let Some(record) = self.rewrite_history.pop() {
                    // Incremental Undo
                    self.cached_inter = Some(inter);
                    self.undo_changes(record.clone());
                    self.update_interaction_graph_delta(&record, false);
                    inter = self.cached_inter.take().unwrap();

                    // Even on backstep, we update internal states to maintain physical continuity
                    self.update_topological_knots(&inter);
                    self.update_stability(&inter);
                    self.perform_kinematics_and_interactions(&inter);
                    self.cached_inter = Some(inter);
                    return true;
                }
            }
        }

        // Experiment C: Spontaneous Vacuum Nucleation (Bypassed in Pure Mode)
        if !self.pure_mode && self.params.defect_injection > 0.0 {
            let rng = &mut self.rng;
            if rng.gen::<f64>() < self.params.defect_injection {
                let v1 = self.h.add_vertex();
                let v2 = self.h.add_vertex();
                let v3 = self.h.add_vertex();
                let v4 = self.h.add_vertex();

                let nodes = vec![v1.id, v2.id, v3.id, v4.id];
                for i in 0..4 {
                    for j in i + 1..4 {
                        self.h.add_causal_relation(nodes[i], nodes[j]);
                        self.h.add_hyperedge(vec![nodes[i], nodes[j]]);
                    }
                }
            }
        }

        // ---------------------------------
        // Propose rewrite
        // ---------------------------------
        let undo_opt = self.propose_rewrite(&inter);
        if undo_opt.is_none() && self.time % 200 == 0 {
            if self.verbose {
                println!("[debug] rewrite skipped at t = {}", self.time);
            }
        }

        let undo = match undo_opt {
            Some(u) => u,
            None => {
                self.cached_inter = Some(inter);
                return false;
            }
        };

        // Cache last rewrite internally
        let undo_clone = undo.clone();
        self.rewrite_history.push(undo_clone.clone());
        if self.rewrite_history.len() > 200 {
            self.rewrite_history
                .drain(0..self.rewrite_history.len() - 200);
        }

        self.last_rewrite = Some(undo_clone);

        // ---------------------------------
        // Acceptance rule
        // ---------------------------------
        let accept_prob = 1.0;

        let rng = &mut self.rng;
        let accepted = rng.gen::<f64>() <= accept_prob;

        if !accepted {
            self.undo_changes(undo.clone());
            // Incremental Undo
            self.cached_inter = Some(inter);
            self.update_interaction_graph_delta(&undo, false);
            inter = self.cached_inter.take().unwrap();
        } else {
            // Incremental Apply
            self.cached_inter = Some(inter);
            self.update_interaction_graph_delta(&undo, true);
            inter = self.cached_inter.take().unwrap();

            // -----------------------------
            // ξ inheritance & propagation (Bypassed in Pure Mode)
            // -----------------------------
            if !self.pure_mode {
                let touched = self.touched_vertices();
                let mut parents = Vec::new();
                for p in &touched {
                    if let Some(&val) = self.xi.get(p) {
                        if val > self.xi_threshold {
                            parents.push(*p);
                        }
                    }
                }

                if let Some(lr) = &self.last_rewrite {
                    for &vid in &lr.added_vertices {
                        if !parents.is_empty() {
                            let mut sum_xi = 0.0;
                            for &p in &parents {
                                sum_xi += self.xi.get(&p).unwrap_or(&0.0);
                            }
                            let inherited = sum_xi / parents.len() as f64;
                            *self.xi.entry(vid).or_insert(0.0) += 0.5 * inherited;
                        }
                    }
                }

                let xi_clusters = self.xi_clusters(&inter);
                self.propagate_xi(&inter, &xi_clusters);
            }

            // --------------------------------------------------
            // Deferred causal bridge
            // --------------------------------------------------
            if let (Some((u, v)), Some(pt)) = (self.pending_bridge, self.pending_bridge_time) {
                if self.time.saturating_sub(pt) >= 20 {
                    if self.h.vertices.contains_key(&u) && self.h.vertices.contains_key(&v) {
                        self.h.add_causal_relation(u, v);
                    }
                    self.pending_bridge = None;
                    self.pending_bridge_time = None;
                }
            }

            // -----------------------------
            // Geometry updates
            // -----------------------------
            if self.time % self.geometry_stride == 0 {
                let xi_support: HashSet<u64> = self
                    .xi
                    .iter()
                    .filter(|(&_v, &x)| x > self.xi_threshold && x.is_finite())
                    .map(|(v, _)| *v)
                    .collect();

                if xi_support.len() >= 2 {
                    // Update topo (v5.6.0 uses inter)
                    self.update_topo_distance_memory(&inter, &xi_support);
                    // Update xi
                    self.update_xi_distance_memory(&inter);
                }
            }

            self.record_xi_current(&inter);

            self.update_topological_knots(&inter);
            self.update_stability(&inter);
            self.perform_kinematics_and_interactions(&inter);
        }

        // Timing + diagnostics
        self.last_step_time = t0.elapsed().as_secs_f64();

        if self.time % 200 == 0 {
            if self.verbose {
                println!("[debug] max causal depth = {}", self.h.max_chain_length());
            }
        }

        if self.verbose && self.time % self.print_interval == 0 {
            // A particle must persist for at least 50 steps and remain dimensionally bounded (radius < 5.0)
            let valid_knots = self
                .active_knots
                .values()
                .filter(|k| k.age >= 50 && k.radius < 5.0)
                .count();
            let geom_pairs = self.topo_distance_memory.len() + self.xi_distance_memory.len();

            let supp_ratio = if self.attempted_rewrites > 0 {
                self.suppressed_rewrites as f64 / self.attempted_rewrites as f64
            } else {
                0.0
            };

            // IMPORTANT:
            // Ω (hierarchical closure) is a derived observable computed from the
            // instantaneous interaction graph. It does NOT influence rewrite dynamics
            // and has no causal propagation. It is used only for diagnostics.
            let omega = compute_omega(&inter);

            println!(
                "[engine] t={} step={:.2}ms Ω={:.6} knots={} geom_pairs={} supp_ratio={:.3}",
                self.time,
                self.last_step_time * 1000.0,
                omega,
                valid_knots,
                geom_pairs,
                supp_ratio
            );

            // reset stats for window
            self.attempted_rewrites = 0;
            self.suppressed_rewrites = 0;
        }

        self.cached_inter = Some(inter);
        accepted
    }
    pub fn update_interaction_graph_delta(&mut self, record: &UndoRecord, applying: bool) {
        let fraction = 0.1;
        let h = &self.h;
        let max_depth = h.max_chain_length() as f64;
        let cutoff = (fraction * max_depth) as usize;

        let inter_map = self.cached_inter.get_or_insert_with(HashMap::new);
        let max_id = h.vertices.keys().max().unwrap_or(&0);
        let cap = (*max_id as usize + 1).max(1024);

        let mut process_edge = |v_ids: &[u64], weight: i32| {
            let wl_ids: Vec<u64> = v_ids
                .iter()
                .filter(|&&id| h.vertices.get(&id).map_or(false, |v| v.depth >= cutoff))
                .copied()
                .collect();

            for &i in &wl_ids {
                for &j in &wl_ids {
                    if i == j {
                        continue;
                    }
                    let pair = if i < j { (i, j) } else { (j, i) };
                    let count = self.interaction_counts.entry(pair).or_insert(0);

                    if weight > 0 {
                        if *count == 0 {
                            let needed = (i as usize).max(j as usize) + 1;

                            let bs_i = inter_map
                                .entry(i)
                                .or_insert_with(|| FixedBitSet::with_capacity(cap));
                            if needed > bs_i.len() {
                                bs_i.grow(needed);
                            }
                            bs_i.insert(i as usize);
                            bs_i.insert(j as usize);

                            let bs_j = inter_map
                                .entry(j)
                                .or_insert_with(|| FixedBitSet::with_capacity(cap));
                            if needed > bs_j.len() {
                                bs_j.grow(needed);
                            }
                            bs_j.insert(j as usize);
                            bs_j.insert(i as usize);
                        }
                        *count += 1;
                    } else {
                        if *count > 0 {
                            *count -= 1;
                            if *count == 0 {
                                if let Some(bs) = inter_map.get_mut(&i) {
                                    if (j as usize) < bs.len() {
                                        bs.set(j as usize, false);
                                    }
                                }
                                if let Some(bs) = inter_map.get_mut(&j) {
                                    if (i as usize) < bs.len() {
                                        bs.set(i as usize, false);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };

        if applying {
            // Add added_edges
            for &eid in &record.added_edges {
                if let Some(edge) = h.hyperedges.get(&eid) {
                    process_edge(&edge.vertices, 1);
                }
            }
            // Remove removed_edges
            for edge in record.removed_edges.values() {
                process_edge(&edge.vertices, -1);
            }
        } else {
            // Undo: Remove added
            for &eid in &record.added_edges {
                // Accessing h here might be tricky if it's already removed,
                // but UndoRecord usually happens right after.
                if let Some(edge) = h.hyperedges.get(&eid) {
                    process_edge(&edge.vertices, -1);
                }
            }
            // Undo: Add removed
            for edge in record.removed_edges.values() {
                process_edge(&edge.vertices, 1);
            }
        }
    }

    pub fn update_topological_knots(&mut self, inter: &HashMap<u64, FixedBitSet>) {
        Self::process_knot_update_static(
            &self.h,
            inter,
            &mut self.active_knots,
            &mut self.dead_knots,
            &mut self.next_knot_id,
            &self.stability,
            self.time,
            1.2, // Default coherence threshold
            0.3, // Default tracking overlap threshold
        );

        // Post-tracking kinematics (still depends on engine state for interactions)
        self.perform_kinematics_and_interactions(inter);
    }

    pub fn process_knot_update_static(
        h: &Hypergraph,
        inter: &HashMap<u64, FixedBitSet>,
        active_knots: &mut HashMap<u64, TopologicalKnot>,
        dead_knots: &mut Vec<TopologicalKnot>,
        next_knot_id: &mut u64,
        _stability: &HashMap<u64, f64>,
        current_time: usize,
        min_coherence: f64,
        overlap_threshold: f64,
    ) {
        let candidates = detect_candidate_knot_neighborhoods(h, inter, min_coherence);
        let mut next_active_knots = HashMap::new();

        // Step 1: Update existing knots
        let mut matched_candidates = HashSet::new();
        for (_, knot) in active_knots.iter() {
            let mut best_idx = None;
            let mut best_overlap = 0.0;

            for (i, cand) in candidates.iter().enumerate() {
                let intersection = knot.vertices.intersection(cand).count() as f64;
                let min_s = knot.vertices.len().min(cand.len()) as f64;
                let overlap = if min_s > 0.0 {
                    intersection / min_s
                } else {
                    0.0
                };

                if overlap > overlap_threshold && overlap > best_overlap {
                    best_overlap = overlap;
                    best_idx = Some(i);
                }
            }

            if let Some(idx) = best_idx {
                matched_candidates.insert(idx);
                let cand = &candidates[idx];
                let (ie, be) = compute_coherence_raw(cand, inter);
                let coherence = if be > 0 {
                    ie as f64 / be as f64
                } else if ie > 0 {
                    10.0
                } else {
                    0.0
                };

                let mut updated_knot = knot.clone();
                updated_knot.vertices = cand.clone();
                updated_knot.age += 10;
                updated_knot.radius = component_radius(cand, inter);
                updated_knot.coherence = coherence;
                if cand.len() > updated_knot.max_size {
                    updated_knot.max_size = cand.len();
                }
                if cand.len() < updated_knot.min_size {
                    updated_knot.min_size = cand.len();
                }

                // Normalize position by max vertex ID so the proxy stays in (0, 1].
                // Without this, monotonically-growing IDs cause unbounded velocity → Inf → NaN
                // which silently poisons all conservation-mode corrections (Hyp A/C/D/E).
                let max_id = h.vertices.keys().max().cloned().unwrap_or(1) as f64;
                let mean_pos =
                    cand.iter().map(|&v| v as f64).sum::<f64>() / (cand.len() as f64 * max_id);
                let prev_pos = knot
                    .position_history
                    .last()
                    .map(|(_, p, _)| *p)
                    .unwrap_or(mean_pos);
                updated_knot.velocity = (mean_pos - prev_pos).abs() / 10.0;

                updated_knot
                    .position_history
                    .push((current_time, mean_pos, coherence));
                if updated_knot.position_history.len() > 100 {
                    updated_knot
                        .position_history
                        .drain(..updated_knot.position_history.len() - 100);
                }

                next_active_knots.insert(updated_knot.id, updated_knot);
            } else {
                if knot.age >= 50 {
                    dead_knots.push(knot.clone());
                    if dead_knots.len() > 200 {
                        dead_knots.drain(0..dead_knots.len() - 200);
                    }
                }
            }
        }

        // Step 2: Spawn new knots from unmatched candidates
        for (idx, cand) in candidates.iter().enumerate() {
            if matched_candidates.contains(&idx) {
                continue;
            }

            let (ie, be) = compute_coherence_raw(cand, inter);
            let coherence = if be > 0 {
                ie as f64 / be as f64
            } else if ie > 0 {
                10.0
            } else {
                0.0
            };

            // Only spawn if it meets structural criteria
            if coherence > min_coherence {
                let max_id = h.vertices.keys().max().cloned().unwrap_or(1) as f64;
                let mean_pos =
                    cand.iter().map(|&v| v as f64).sum::<f64>() / (cand.len() as f64 * max_id);
                let new_knot = TopologicalKnot {
                    id: *next_knot_id,
                    vertices: cand.clone(),
                    age: 10,
                    radius: component_radius(cand, inter),
                    max_size: cand.len(),
                    min_size: cand.len(),
                    coherence,
                    velocity: 0.0,
                    velocity_avg: (0.0, 0.0),
                    mass: cand.len() as f64 * coherence.powi(2),
                    momentum: 0.0,
                    energy: 0.0,
                    prev_mass: cand.len() as f64 * coherence.powi(2),
                    prev_momentum: 0.0,
                    position_history: vec![(current_time, mean_pos, coherence)],
                };
                next_active_knots.insert(new_knot.id, new_knot);
                *next_knot_id += 1;
            }
        }

        *active_knots = next_active_knots;
    }

    fn perform_kinematics_and_interactions(&mut self, _inter: &HashMap<u64, FixedBitSet>) {
        // Step 3: Formal Kinematics
        self.coupled_vertices.clear();
        let active_ids: Vec<_> = self.active_knots.keys().copied().collect();
        for &id in &active_ids {
            let knot = self.active_knots.get_mut(&id).unwrap();

            // Store previous state for conservation modes
            knot.prev_mass = knot.mass;
            knot.prev_momentum = knot.momentum;

            knot.mass = knot.vertices.len() as f64 * knot.coherence.powi(2);
            let hist = &knot.position_history;
            if hist.len() > 1 {
                // Use CONSECUTIVE frames (last two) for instantaneous velocity.
                // Using first→last spans different normalization baselines (max_id grows
                // monotonically), so the positional delta explodes even after normalizing.
                // Consecutive frames are always within ~10 simulation steps of each other,
                // so max_id barely changes and the delta is small and physically meaningful.
                let (t1, p1, _) = hist[hist.len() - 2];
                let (t2, p2, _) = hist[hist.len() - 1];
                let dt = (t2 - t1) as f64;
                if dt > 0.0 {
                    let mut dv = (p2 - p1) / dt;

                    // Hypothesis B: Exponential Inertia (Inertial Cooling v5.0)
                    if self.params.enable_conservation_patches
                        && (self.conservation_mode == ConservationMode::StabilityScaled
                            || self.conservation_mode == ConservationMode::Hybrid)
                    {
                        let stab = knot
                            .vertices
                            .iter()
                            .map(|&v| self.stability.get(&v).unwrap_or(&0.0))
                            .sum::<f64>()
                            / knot.vertices.len() as f64;
                        // v_new = v_old * exp(-S / 30.0) + v_min
                        dv = dv * (-stab / 30.0).exp() + 0.05;
                    }

                    // Hard clamp: velocity must stay physically bounded.
                    // Normalized position ∈ (0,1], so |Δpos/Δt| should never exceed ~1.
                    // Clamp to ±10.0 as a generous safety ceiling.
                    dv = dv.clamp(-10.0, 10.0);

                    knot.velocity_avg = (dv, 0.0);
                }
            }
            knot.momentum = knot.mass * knot.velocity_avg.0;

            // Energy Definition: E = 0.5 * M * v^2 + lambda * stability
            let stab_sum = knot
                .vertices
                .iter()
                .map(|&v| self.stability.get(&v).unwrap_or(&0.0))
                .sum::<f64>();
            let mean_stability = if !knot.vertices.is_empty() {
                stab_sum / knot.vertices.len() as f64
            } else {
                0.0
            };
            knot.energy = 0.5 * knot.mass * knot.velocity_avg.0.powi(2) + 0.02 * mean_stability;
        }

        // --- Hypothesis D: Mass Coupling (Soft Newtonian) ---
        if self.params.enable_conservation_patches
            && self.conservation_mode == ConservationMode::MassCoupled
        {
            for knot in self.active_knots.values_mut() {
                let p_before = knot.prev_momentum;
                let p_after = knot.momentum;
                let correction = (p_before - p_after) * 0.3; // 30% soft constraint
                knot.velocity_avg.0 += correction / (knot.mass + 1e-6);
                knot.momentum = knot.mass * knot.velocity_avg.0;
            }
        }

        // --- Hypothesis C: Local Flux Compensation (Residual Field) ---
        if self.params.enable_conservation_patches
            && (self.conservation_mode == ConservationMode::FluxCompensated
                || self.conservation_mode == ConservationMode::Hybrid)
        {
            for knot in self.active_knots.values_mut() {
                let p_before = knot.prev_momentum;
                let p_after = knot.momentum;
                let delta_p = p_before - p_after;

                // Leak excess to neighborhood reservoir
                if delta_p.abs() > 1e-6 {
                    let per_v = delta_p / (knot.vertices.len() as f64).max(1.0);
                    for &v in &knot.vertices {
                        *self.momentum_reservoir.entry(v).or_insert(0.0) += per_v;
                    }
                }

                // Diffusive re-absorption (alpha = 0.15)
                for &v in &knot.vertices {
                    if let Some(res) = self.momentum_reservoir.get_mut(&v) {
                        let siphon = *res * 0.15;
                        knot.velocity_avg.0 += siphon / (knot.mass + 1e-6);
                        *res -= siphon;
                    }
                }
            }

            // Reservoir Decay (0.999) - Near-lossless emergence
            for res in self.momentum_reservoir.values_mut() {
                *res *= 0.999;
            }

            // --- Causal Redistribution (v5.0 Phase Transition Support) ---
            // Distribute momentum from vertices that are NO LONGER in the graph
            // into their 1-hop causal neighborhood.
            let mut diffusions = Vec::new();
            let mut orphans = Vec::new();
            for (&v, &mom) in &self.momentum_reservoir {
                if !self.h.vertices.contains_key(&v) {
                    orphans.push(v);
                    if let Some(neighbors) = _inter.get(&v) {
                        let n_deg = neighbors.len() as f64;
                        if n_deg > 0.0 {
                            let share = mom / n_deg;
                            for nbr_idx in neighbors.ones() {
                                let nbr = nbr_idx as u64;
                                diffusions.push((nbr, share));
                            }
                        }
                    }
                }
            }

            for v in orphans {
                self.momentum_reservoir.remove(&v);
            }
            for (v, share) in diffusions {
                *self.momentum_reservoir.entry(v).or_insert(0.0) += share;
            }
        }

        // Detect overlaps and manage persistent InteractionEvents
        let mut current_overlaps = HashSet::new();
        for i in 0..active_ids.len() {
            for j in i + 1..active_ids.len() {
                let id_a = active_ids[i];
                let id_b = active_ids[j];
                let knot_a = &self.active_knots[&id_a];
                let knot_b = &self.active_knots[&id_b];

                let intersection = knot_a.vertices.intersection(&knot_b.vertices).count();
                if intersection > 0 {
                    let pair = if id_a < id_b {
                        (id_a, id_b)
                    } else {
                        (id_b, id_a)
                    };
                    let chi = intersection as f64
                        / (knot_a.vertices.len().min(knot_b.vertices.len()) as f64);

                    // Noise filter for active engagement
                    if chi > 0.015 {
                        current_overlaps.insert(pair);

                        if !self.active_interactions.contains_key(&pair) {
                            let m_a = knot_a.mass;
                            let m_b = knot_b.mass;
                            let res = (2.0 * knot_a.coherence * knot_b.coherence)
                                / (knot_a.coherence.powi(2) + knot_b.coherence.powi(2)).max(1e-6);

                            let stab_a = knot_a
                                .vertices
                                .iter()
                                .map(|&v| self.stability.get(&v).unwrap_or(&0.0))
                                .sum::<f64>()
                                / knot_a.vertices.len() as f64;
                            let stab_b = knot_b
                                .vertices
                                .iter()
                                .map(|&v| self.stability.get(&v).unwrap_or(&0.0))
                                .sum::<f64>()
                                / knot_b.vertices.len() as f64;

                            let (int_a, bnd_a) = compute_coherence_raw(&knot_a.vertices, _inter);
                            let ratio_a = int_a as f64 / (int_a + bnd_a).max(1) as f64;
                            let (int_b, bnd_b) = compute_coherence_raw(&knot_b.vertices, _inter);
                            let ratio_b = int_b as f64 / (int_b + bnd_b).max(1) as f64;

                            self.active_interactions.insert(
                                pair,
                                InteractionEvent {
                                    start_time: self.time,
                                    end_time: None,
                                    duration: 0,
                                    knot_a: id_a,
                                    knot_b: id_b,
                                    overlap_size: intersection,
                                    overlap_depth: chi,
                                    resonance: res,
                                    pre_a: (
                                        m_a,
                                        knot_a.velocity,
                                        m_a * knot_a.velocity_avg.0,
                                        knot_a.velocity_avg,
                                        knot_a.coherence,
                                        stab_a,
                                        knot_a.radius,
                                        knot_a.vertices.len(),
                                        ratio_a,
                                        knot_a.energy,
                                        knot_a.age,
                                    ),
                                    pre_b: (
                                        m_b,
                                        knot_b.velocity,
                                        m_b * knot_b.velocity_avg.0,
                                        knot_b.velocity_avg,
                                        knot_b.coherence,
                                        stab_b,
                                        knot_b.radius,
                                        knot_b.vertices.len(),
                                        ratio_b,
                                        knot_b.energy,
                                        knot_b.age,
                                    ),
                                    post_a: None,
                                    post_b: None,
                                    steps_below_threshold: 0,
                                },
                            );
                        } else {
                            // Update max chi seen during interaction
                            let event = self.active_interactions.get_mut(&pair).unwrap();
                            if chi > event.overlap_depth {
                                event.overlap_depth = chi;
                            }
                            event.steps_below_threshold = 0;
                        }
                    }

                    if chi > 0.4 {
                        for &v in &knot_a.vertices {
                            self.coupled_vertices.insert(v);
                        }
                        for &v in &knot_b.vertices {
                            self.coupled_vertices.insert(v);
                        }
                    }
                }
            }
        }

        // --- Hypothesis A: Pairwise Symmetry (Symmetric Correction) ---
        if self.params.enable_conservation_patches
            && (self.conservation_mode == ConservationMode::Pairwise
                || self.conservation_mode == ConservationMode::Hybrid)
        {
            let mut corrections = Vec::new();
            for (pair, event) in self.active_interactions.iter() {
                if current_overlaps.contains(pair) {
                    if let (Some(ka), Some(kb)) = (
                        self.active_knots.get(&event.knot_a),
                        self.active_knots.get(&event.knot_b),
                    ) {
                        let p_a_before = ka.prev_momentum;
                        let p_b_before = kb.prev_momentum;
                        let p_a_after = ka.momentum;
                        let p_b_after = kb.momentum;

                        let delta_total = (p_a_after + p_b_after) - (p_a_before + p_b_before);

                        // Hybrid A: Asymptotic Symmetry Ramp (v6.0 Phase Transition)
                        let s_avg = (event.pre_a.5 + event.pre_b.5) / 2.0;

                        // k ramps non-linearly with gamma. Physics sharpens as S -> 20.0
                        let k = (s_avg / 20.0).powf(self.params.nonlinear_coupling).min(1.0);

                        let correction = -k * 0.5 * delta_total;
                        // Guard: only apply correction if finite (safety net for edge cases)
                        if correction.is_finite() {
                            corrections.push((event.knot_a, event.knot_b, correction));
                        }
                    }
                }
            }

            for (id_a, id_b, corr) in corrections {
                if let Some(ka) = self.active_knots.get_mut(&id_a) {
                    ka.velocity_avg.0 += corr / (ka.mass + 1e-6);
                    ka.momentum = ka.mass * ka.velocity_avg.0;
                }
                if let Some(kb) = self.active_knots.get_mut(&id_b) {
                    kb.velocity_avg.0 += corr / (kb.mass + 1e-6);
                    kb.momentum = kb.mass * kb.velocity_avg.0;
                }
            }
        }

        // Finalize interactions that have ended (stability window)
        let mut finished = Vec::new();
        for (pair, event) in self.active_interactions.iter_mut() {
            if !current_overlaps.contains(pair) {
                event.steps_below_threshold += 1;
                // Since this check runs every step, threshold 10 keeps transient overlaps filtered.
                if event.steps_below_threshold >= 10 {
                    finished.push(*pair);
                }
            }
        }

        for pair in finished {
            let mut event = self.active_interactions.remove(&pair).unwrap();
            event.end_time = Some(self.time);
            event.duration = self.time.saturating_sub(event.start_time);

            // Capture post state
            if let Some(ka) = self.active_knots.get(&event.knot_a) {
                let stab = ka
                    .vertices
                    .iter()
                    .map(|&v| self.stability.get(&v).unwrap_or(&0.0))
                    .sum::<f64>()
                    / ka.vertices.len() as f64;
                let (int, bnd) = compute_coherence_raw(&ka.vertices, _inter);
                let ratio = int as f64 / (int + bnd).max(1) as f64;
                event.post_a = Some((
                    ka.mass,
                    ka.velocity,
                    ka.mass * ka.velocity_avg.0,
                    ka.velocity_avg,
                    ka.coherence,
                    stab,
                    ka.radius,
                    ka.vertices.len(),
                    ratio,
                    ka.energy,
                    ka.age,
                ));
            }
            if let Some(kb) = self.active_knots.get(&event.knot_b) {
                let stab = kb
                    .vertices
                    .iter()
                    .map(|&v| self.stability.get(&v).unwrap_or(&0.0))
                    .sum::<f64>()
                    / kb.vertices.len() as f64;
                let (int, bnd) = compute_coherence_raw(&kb.vertices, _inter);
                let ratio = int as f64 / (int + bnd).max(1) as f64;
                event.post_b = Some((
                    kb.mass,
                    kb.velocity,
                    kb.mass * kb.velocity_avg.0,
                    kb.velocity_avg,
                    kb.coherence,
                    stab,
                    kb.radius,
                    kb.vertices.len(),
                    ratio,
                    kb.energy,
                    kb.age,
                ));
            }

            self.interaction_events.push(event);
        }
    }

    /// Update per-vertex stability memory.
    /// Vertices inside active knots accumulate stability.
    /// All stability values decay slowly each cycle.
    fn update_stability(&mut self, _inter: &HashMap<u64, FixedBitSet>) {
        // (1) Stability Decay (nu)
        let stability_decay = self.params.stability_decay;

        // Decay all existing stability
        for val in self.stability.values_mut() {
            *val *= stability_decay;
        }

        // Accumulate stability for vertices in active knots
        for knot in self.active_knots.values() {
            // Natural selection baseline: G = 1.0
            let gain = 1.0;
            for &v in &knot.vertices {
                let entry = self.stability.entry(v).or_insert(0.0);
                *entry += gain;
            }
        }

        // Clean up: remove entries for vertices no longer in the graph
        self.stability
            .retain(|v, _| self.h.vertices.contains_key(v));

        // Cap stability to prevent runaway (v5.0 Phase transition expansion)
        for val in self.stability.values_mut() {
            if *val > 50.0 {
                *val = 50.0;
            }
        }
    }

    // --------------------------------------------------
    // Rewrite proposal
    // --------------------------------------------------
    fn propose_rewrite(&mut self, inter: &HashMap<u64, FixedBitSet>) -> Option<UndoRecord> {
        let rng = &mut self.rng;

        self.attempted_rewrites += 1;

        let vertices: Vec<u64> = self.h.vertices.keys().copied().collect();
        if vertices.is_empty() {
            return None;
        }
        let anchor_v = *vertices.choose(rng).unwrap();

        // --- Pure Mode Bypass ---
        if self.pure_mode {
            // Uniformly pick between edge creation (p_create) and vertex fusion
            if rng.gen::<f64>() < 0.9 {
                return crate::rules::edge_creation_rule(
                    &mut self.h,
                    Some(anchor_v),
                    self.p_create,
                );
            } else {
                return crate::rules::vertex_fusion_rule(&mut self.h, Some(anchor_v));
            }
        }

        // --- Density metric ---
        let clustering = crate::observables::local_clustering(inter, anchor_v);
        let degree = inter.get(&anchor_v).map(|n| n.len()).unwrap_or(0) as f64;

        let total_degree: usize = inter.values().map(|n| n.len()).sum();
        let avg_degree = if inter.is_empty() {
            1.0
        } else {
            (total_degree as f64) / (inter.len() as f64)
        }
        .max(1.0);

        let local_density = clustering * (degree / avg_degree);

        // --- (1) Unified suppression: density + coherence-dependent survival ---
        // Base suppression from density, enhanced by structural coherence for formed structures.
        // alpha_eff = alpha_base + lambda * max(0, coherence - threshold)
        // This makes high-coherence neighborhoods progressively harder to destroy
        // without a separate blocking gate that compounds to freeze everything.

        // Compute coherence first (needed for suppression + growth)
        let mut internal_edges: u32 = 0;
        let mut boundary_edges: u32 = 0;

        if let Some(neighbors) = inter.get(&anchor_v) {
            let mut neighborhood = HashSet::new();
            for nbr_idx in neighbors.ones() {
                neighborhood.insert(nbr_idx as u64);
            }
            neighborhood.insert(anchor_v);

            let (ie, be) = crate::observables::compute_coherence_raw(&neighborhood, inter);
            internal_edges = ie;
            boundary_edges = be;
        }

        let coherence = if boundary_edges > 0 {
            internal_edges as f64 / boundary_edges as f64
        } else if internal_edges > 0 {
            10.0
        } else {
            0.0
        };

        // Unified suppression with structure-dependent survival
        let alpha_base = 2.0;
        let lambda = 0.5;
        let survival_threshold = 1.0;
        let neighborhood_size = inter.get(&anchor_v).map(|n| n.len()).unwrap_or(0) + 1;

        let coherence_boost = if neighborhood_size >= 4 && coherence > survival_threshold {
            lambda * (coherence - survival_threshold)
        } else {
            0.0
        };

        // Nonlinear memory: stability^γ creates threshold effect
        // Low stability ≈ no protection, high stability ≈ strong protection
        let mu = self.params.memory_coupling;
        let gamma = self.params.nonlinear_coupling;
        let stability_cap = 30.0_f64;
        let vertex_stability = self.stability.get(&anchor_v).copied().unwrap_or(0.0);
        let normalized_stability = (vertex_stability / stability_cap).min(1.0);
        let memory_contribution = mu * stability_cap * normalized_stability.powf(gamma);

        let alpha_eff = alpha_base + coherence_boost + memory_contribution;

        // Phase 8: Coupling Pulse - reduce protection during deep overlap
        let coupling_modifier = if self.coupled_vertices.contains(&anchor_v) {
            0.2
        } else {
            1.0
        };
        let rewrite_prob = (-(alpha_eff * coupling_modifier) * local_density).exp();

        if rng.gen::<f64>() > rewrite_prob {
            self.suppressed_rewrites += 1;
            return None; // Suppressed (density + structural protection)
        }

        let theta = 1.3; // Nucleation threshold
        let beta = 1.5;
        let growth = if coherence > theta { beta } else { 0.0 };

        // --- (3) Boundary tension: inhibits growth at high boundary ratio ---
        let boundary_ratio = if coherence > 0.0 {
            1.0 / coherence
        } else {
            10.0
        };
        let gamma = 20.0;
        let boundary_term = 1.0 / (1.0 + gamma * boundary_ratio);

        // --- Full emergence bias ---
        let growth_bias = 1.0 + growth * boundary_term;

        let p_creation = (0.90 * growth_bias).min(0.99);
        let p_fusion = (0.05 / growth_bias).min(0.99);

        if rng.gen::<f64>() < p_creation {
            return crate::rules::edge_creation_rule(&mut self.h, Some(anchor_v), self.p_create);
        }

        if self.h.vertices.len() > 200 && rng.gen::<f64>() < p_fusion {
            return crate::rules::vertex_fusion_rule(&mut self.h, Some(anchor_v));
        }

        None
    }

    // --------------------------------------------------
    // ξ propagation
    // --------------------------------------------------
    fn propagate_xi(&mut self, inter: &HashMap<u64, FixedBitSet>, clusters: &HashMap<u64, usize>) {
        let mut new_xi = self.xi.clone();
        let xi_max = 1e6;

        let protect_clusters = if let Some(ft) = self.forced_time {
            self.time.saturating_sub(ft) < 999999
        } else {
            false
        };

        for (&v, &xi_v) in &self.xi {
            if xi_v < self.xi_threshold {
                continue;
            }

            let cid_v = clusters.get(&v);
            let xi_v_decayed = xi_v * self.xi_decay;

            if let Some(neighbors) = inter.get(&v) {
                let deg = (neighbors.ones().count() as f64).max(1.0);
                for u_idx in neighbors.ones() {
                    let u = u_idx as u64;
                    let cid_u = clusters.get(&u);

                    if protect_clusters {
                        if let (Some(cu), Some(cv)) = (cid_u, cid_v) {
                            if cu != cv {
                                continue;
                            }
                        }
                    }

                    *new_xi.entry(u).or_insert(0.0) += 0.15 * xi_v_decayed / deg;
                }
            }

            *new_xi.entry(v).or_insert(0.0) += 0.7 * xi_v_decayed;
        }

        for val in new_xi.values_mut() {
            if *val > xi_max {
                *val = xi_max;
            }
        }

        self.xi = new_xi;
    }

    // --------------------------------------------------
    // ξ clusters
    // --------------------------------------------------
    fn xi_clusters(&self, inter: &HashMap<u64, FixedBitSet>) -> HashMap<u64, usize> {
        let mut clusters = HashMap::new();
        let mut visited = HashSet::new();
        let mut cid = 0;

        let xi_vertices: HashSet<u64> = self
            .xi
            .iter()
            .filter(|(&v, &x)| x > self.xi_threshold && self.h.vertices.contains_key(&v))
            .map(|(v, _)| *v)
            .collect();

        for &v in &xi_vertices {
            if visited.contains(&v) {
                continue;
            }
            let mut stack = vec![v];
            visited.insert(v);
            clusters.insert(v, cid);

            while let Some(u) = stack.pop() {
                if let Some(nbrs) = inter.get(&u) {
                    for w_idx in nbrs.ones() {
                        let w = w_idx as u64;
                        if xi_vertices.contains(&w) && !visited.contains(&w) {
                            visited.insert(w);
                            clusters.insert(w, cid);
                            stack.push(w);
                        }
                    }
                }
            }
            cid += 1;
        }

        clusters
    }

    // --------------------------------------------------
    // Topological clusters (PURE TOPOLOGY)
    // --------------------------------------------------
    fn topo_clusters(&self, inter: &HashMap<u64, FixedBitSet>) -> HashMap<u64, usize> {
        let mut clusters = HashMap::new();
        let mut visited = HashSet::new();
        let mut cid = 0;

        for &v in inter.keys() {
            if visited.contains(&v) {
                continue;
            }
            let mut stack = vec![v];
            visited.insert(v);
            clusters.insert(v, cid);

            while let Some(u) = stack.pop() {
                if let Some(nbrs) = inter.get(&u) {
                    for w_idx in nbrs.ones() {
                        let w = w_idx as u64;
                        if !visited.contains(&w) {
                            visited.insert(w);
                            clusters.insert(w, cid);
                            stack.push(w);
                        }
                    }
                }
            }
            cid += 1;
        }

        clusters
    }

    // --------------------------------------------------
    // Geometry memory (G1)
    // --------------------------------------------------
    fn update_topo_distance_memory(
        &mut self,
        inter: &HashMap<u64, FixedBitSet>,
        restrict_to: &HashSet<u64>,
    ) {
        let topo = self.topo_clusters(inter);

        let mut topo_groups: HashMap<usize, Vec<u64>> = HashMap::new();
        for &v in restrict_to {
            if let Some(&cid) = topo.get(&v) {
                topo_groups.entry(cid).or_default().push(v);
            }
        }

        let topo_ids: Vec<usize> = topo_groups.keys().cloned().collect();
        if topo_ids.is_empty() {
            return;
        }

        let max_depth = self.geometry_depth();

        // 🔧 Fallback: store intra-component scale if only one topo component
        if topo_ids.len() == 1 {
            let verts = &topo_groups[&topo_ids[0]];
            if verts.len() >= 2 {
                let mid = verts.len() / 2;
                let mut a_verts = verts[..mid].to_vec();
                if a_verts.len() > 25 {
                    a_verts.truncate(25);
                }
                let b_verts: HashSet<u64> = verts[mid..].iter().cloned().collect();

                let mut min_d = usize::MAX;
                for &v in &a_verts {
                    let d = self.graph_distance(inter, v, &b_verts, max_depth);
                    if d < min_d {
                        min_d = d;
                    }
                }

                if min_d < usize::MAX {
                    let key = ("topo".to_string(), topo_ids[0], topo_ids[0]);
                    let d_f = min_d as f64;
                    let prev = *self.topo_distance_memory.get(&key).unwrap_or(&d_f);
                    let new_val = self.distance_memory_decay * prev
                        + (1.0 - self.distance_memory_decay) * d_f;
                    self.topo_distance_memory.insert(key, new_val);
                }
            }
        }

        for i in 0..topo_ids.len() {
            for j in (i + 1)..topo_ids.len() {
                let mut a_verts = topo_groups[&topo_ids[i]].clone();
                if a_verts.len() > 25 {
                    a_verts.truncate(25);
                }
                let b_verts: HashSet<u64> = topo_groups[&topo_ids[j]].iter().cloned().collect();

                let mut min_d = usize::MAX;
                for &v in &a_verts {
                    let d = self.graph_distance(inter, v, &b_verts, max_depth);
                    if d < min_d {
                        min_d = d;
                    }
                }

                if min_d < usize::MAX {
                    let key = ("topo".to_string(), topo_ids[i], topo_ids[j]);
                    let d_f = min_d as f64;
                    let prev = *self.topo_distance_memory.get(&key).unwrap_or(&d_f);
                    let new_val = self.distance_memory_decay * prev
                        + (1.0 - self.distance_memory_decay) * d_f;
                    self.topo_distance_memory.insert(key, new_val);
                }
            }
        }
    }

    // --------------------------------------------------
    // Cluster distance memory (G2)
    // --------------------------------------------------
    fn update_xi_distance_memory(&mut self, inter: &HashMap<u64, FixedBitSet>) {
        let xi_clusters = self.xi_clusters(inter);

        let mut cluster_to_vertices: HashMap<usize, Vec<u64>> = HashMap::new();
        for (&v, &cid) in &xi_clusters {
            cluster_to_vertices.entry(cid).or_default().push(v);
        }

        let cluster_ids: Vec<usize> = cluster_to_vertices.keys().cloned().collect();
        if cluster_ids.is_empty() {
            return;
        }

        let max_depth = self.geometry_depth();

        // 🔧 Fallback: intra-component scale if only one xi cluster
        if cluster_ids.len() == 1 {
            let verts = &cluster_to_vertices[&cluster_ids[0]];
            if verts.len() >= 2 {
                let mid = verts.len() / 2;
                let mut a_verts = verts[..mid].to_vec();
                if a_verts.len() > 25 {
                    a_verts.truncate(25);
                }
                let b_verts: HashSet<u64> = verts[mid..].iter().cloned().collect();

                let mut min_d = usize::MAX;
                for &v in &a_verts {
                    let d = self.graph_distance(inter, v, &b_verts, max_depth);
                    if d < min_d {
                        min_d = d;
                    }
                }

                if min_d < usize::MAX {
                    let key = ("xi".to_string(), cluster_ids[0], cluster_ids[0]);
                    let d_f = min_d as f64;
                    let prev = *self.xi_distance_memory.get(&key).unwrap_or(&d_f);
                    let new_val = self.distance_memory_decay * prev
                        + (1.0 - self.distance_memory_decay) * d_f;
                    self.xi_distance_memory.insert(key, new_val);
                    if self.verbose {
                        println!(
                            "[geom-add] xi_pair (intra-cluster) ({}, {}) d={}",
                            cluster_ids[0], cluster_ids[0], min_d
                        );
                    }
                }
            }
        }

        for i in 0..cluster_ids.len() {
            for j in (i + 1)..cluster_ids.len() {
                let mut a_verts = cluster_to_vertices[&cluster_ids[i]].clone();
                let b_verts: HashSet<u64> = cluster_to_vertices[&cluster_ids[j]]
                    .iter()
                    .cloned()
                    .collect();

                // limit sampling to avoid O(N²) blowup
                if a_verts.len() > 25 {
                    a_verts.truncate(25);
                }

                let mut min_d = usize::MAX;

                for &v in &a_verts {
                    let d = self.graph_distance(inter, v, &b_verts, max_depth);
                    if d < min_d {
                        min_d = d;
                    }
                }

                if min_d < usize::MAX {
                    let key = ("xi".to_string(), cluster_ids[i], cluster_ids[j]);
                    let d_f = min_d as f64;
                    let prev = *self.xi_distance_memory.get(&key).unwrap_or(&d_f);
                    let new_val = self.distance_memory_decay * prev
                        + (1.0 - self.distance_memory_decay) * d_f;
                    self.xi_distance_memory.insert(key, new_val);

                    if self.verbose {
                        println!(
                            "[geom-add] xi_pair ({}, {}) d={}",
                            cluster_ids[i], cluster_ids[j], min_d
                        );
                    }
                }
            }
        }
    }

    fn record_xi_current(&mut self, _inter: &HashMap<u64, FixedBitSet>) {
        let touched = self.touched_vertices();
        let mut delta_xi = HashMap::new();
        for v in touched {
            if self.prev_xi.contains_key(&v) {
                let current = self.xi.get(&v).unwrap_or(&0.0);
                let prev = self.prev_xi.get(&v).unwrap_or(&0.0);
                let delta = current - prev;
                if delta.is_finite() {
                    delta_xi.insert(v, delta);
                }
            }
        }

        if !delta_xi.is_empty() {
            self.xi_current_log.push(XiCurrentLogEntry {
                time: self.time,
                delta_xi,
            });
        }
    }

    fn touched_vertices(&self) -> HashSet<u64> {
        let mut touched = HashSet::new();
        if let Some(lr) = &self.last_rewrite {
            for &v in &lr.added_vertices {
                touched.insert(v);
            }
            if let Some(v) = &lr.removed_vertex {
                touched.insert(v.id);
            }
        }
        touched
    }

    fn geometry_depth(&self) -> usize {
        let n = self.h.vertices.len() as f64;
        16.max((n + 1.0).log2() as usize * 4)
    }

    fn graph_distance(
        &self,
        inter: &HashMap<u64, FixedBitSet>,
        start: u64,
        targets: &HashSet<u64>,
        max_depth: usize,
    ) -> usize {
        if targets.contains(&start) {
            return 0;
        }

        let mut visited = HashSet::new();
        visited.insert(start);

        let mut frontier = HashSet::new();
        frontier.insert(start);

        let mut depth = 0;

        while !frontier.is_empty() && depth < max_depth {
            depth += 1;
            let mut next_frontier = HashSet::new();

            for &v in &frontier {
                if let Some(nbrs) = inter.get(&v) {
                    for u_idx in nbrs.ones() {
                        let u = u_idx as u64;
                        if visited.contains(&u) {
                            continue;
                        }
                        if targets.contains(&u) {
                            return depth;
                        }
                        visited.insert(u);
                        next_frontier.insert(u);
                    }
                }
            }
            frontier = next_frontier;
        }
        usize::MAX
    }

    fn undo_changes(&mut self, record: UndoRecord) {
        // --- Stability Inheritance (v5.0) ---
        // Transfer 90% of stability from the anchor/target vertices to the new vertices
        let mut target_stability = 0.0;
        let targets: Vec<u64> = record
            .target
            .iter()
            .map(|&id| self.h.vertices.get(&id).map(|v| v.id).unwrap_or(0))
            .collect();

        if !targets.is_empty() {
            target_stability = targets
                .iter()
                .map(|&v| self.stability.get(&v).copied().unwrap_or(0.0))
                .sum::<f64>()
                / targets.len() as f64;
        }

        // Inheritance flows to newly added vertices (S_new = 0.9 * S_old)
        let inheritance = target_stability * 0.9;
        for &new_v_id in &record.added_vertices {
            let s = self.stability.entry(new_v_id).or_insert(0.0);
            *s = (*s + inheritance).min(50.0);
        }

        // Apply hypergraph changes
        self.h.execute_undo_record(record);
    }

    // --------------------------------------------------
    // Forced probes (matter injection)
    // --------------------------------------------------
    pub fn force_defect(&mut self, magnitude: f64, max_tries: usize) -> bool {
        if self.h.vertices.is_empty() {
            return false;
        }

        let rng = &mut self.rng;
        let vertex_ids: Vec<u64> = self.h.vertices.keys().cloned().collect();

        for _ in 0..max_tries {
            let vid = *vertex_ids.choose(rng).unwrap();
            let undo = edge_creation_rule(&mut self.h, Some(vid), self.p_create);

            if let Some(u) = undo {
                *self.xi.entry(vid).or_insert(0.0) += magnitude;
                self.forced_time = Some(self.time);

                if self.verbose {
                    println!("[inject] defect at t={} v={}", self.time, vid);
                }

                // Track rewrite internally
                self.last_rewrite = Some(UndoRecord {
                    target: Vec::new(),
                    added_vertices: u.added_vertices.clone(),
                    removed_vertex: u.removed_vertex.clone(),
                    kept_vertex: u.kept_vertex.clone(),
                    added_edges: Vec::new(),
                    added_causal: Vec::new(),
                    removed_edges: HashMap::new(),
                    old_causal_future: HashMap::new(),
                    old_causal_past: HashMap::new(),
                    old_parents: HashMap::new(),
                    old_children: HashMap::new(),
                });
                return true;
            }
        }
        false
    }

    pub fn force_second_proto_object(
        &mut self,
        _omega_kick: f64, // Not currently used directly in python injection logic
        xi_seed: f64,
        min_distance: usize,
        max_tries: usize,
    ) -> bool {
        let mut xi_support = HashSet::new();
        for (&vid, &x) in &self.xi {
            if x > self.xi_threshold && self.h.vertices.contains_key(&vid) {
                xi_support.insert(vid);
            }
        }

        if xi_support.is_empty() {
            return false;
        }

        let inter = crate::observables::worldline_interaction_graph(&self.h, 0.0);
        let n = self.h.vertices.len().max(1) as f64;
        let max_depth = 20.max((n.log2() * 4.0) as usize);

        // restrict candidates to the same connected component to avoid `inf` distances
        let mut reachable = xi_support.clone();
        let mut frontier = xi_support.clone();
        let mut depth = 0;

        while !frontier.is_empty() && depth < max_depth {
            depth += 1;
            let mut nxt = HashSet::new();
            for &v in &frontier {
                if let Some(nbrs) = inter.get(&v) {
                    for u_idx in nbrs.ones() {
                        let u = u_idx as u64;
                        if !reachable.contains(&u) {
                            reachable.insert(u);
                            nxt.insert(u);
                        }
                    }
                }
            }
            frontier = nxt;
        }

        let mut candidates: Vec<u64> = reachable.difference(&xi_support).cloned().collect();
        if candidates.is_empty() {
            // island is completely isolated, fallback to global graph
            candidates = self.h.vertices.keys().cloned().collect();
        }

        let mut best_vid = None;
        let mut best_d = 0;

        for _ in 0..max_tries {
            let vid = *candidates.choose(&mut self.rng).unwrap();
            if xi_support.contains(&vid) {
                continue;
            }

            let d = self.graph_distance(&inter, vid, &xi_support, max_depth);

            if d > best_d || best_vid.is_none() {
                best_d = d;
                best_vid = Some(vid);
            }

            if d >= min_distance && d != usize::MAX {
                self.xi.insert(vid, xi_seed);
                self.forced_time = Some(self.time);
                if self.verbose {
                    println!("### SECOND PROBE at t={} | v={} | d={}", self.time, vid, d);
                }
                return true;
            }
        }

        if let Some(vid) = best_vid {
            self.xi.insert(vid, xi_seed);

            if let Some(&u) = xi_support.iter().next() {
                self.pending_bridge = Some((u, vid));
                self.pending_bridge_time = Some(self.time);
            }

            self.forced_time = Some(self.time);
            if self.verbose {
                println!(
                    "### SECOND PROBE (fallback) at t={} | v={} | d={}",
                    self.time, vid, best_d
                );
            }
            return true;
        }

        false
    }

    pub fn compute_local_suppression_for_knot(
        &self,
        knot: &crate::observables::TopologicalKnot,
    ) -> f64 {
        let alpha_base = 2.0;
        let lambda = 0.5;
        let survival_threshold = 1.0;

        let avg_neighborhood = if knot.vertices.is_empty() {
            0.0
        } else {
            knot.vertices
                .iter()
                .map(|v| self.h.coordination_number(*v))
                .sum::<usize>() as f64
                / knot.vertices.len() as f64
        };

        let coherence_boost = if avg_neighborhood >= 4.0 && knot.coherence > survival_threshold {
            lambda * (knot.coherence - survival_threshold)
        } else {
            0.0
        };

        let mu = self.params.memory_coupling;
        let gamma = self.params.nonlinear_coupling;
        let stability_cap = 30.0_f64;
        let mean_stability = if knot.vertices.is_empty() {
            0.0
        } else {
            knot.vertices
                .iter()
                .map(|v| self.stability.get(v).copied().unwrap_or(0.0))
                .sum::<f64>()
                / knot.vertices.len() as f64
        };

        let normalized_stability = (mean_stability / stability_cap).min(1.0);
        let memory_contribution = mu * stability_cap * normalized_stability.powf(gamma);

        let alpha_eff = alpha_base + coherence_boost + memory_contribution;
        let coupling_modifier = 1.0;
        let local_density = 1.0;

        (-(alpha_eff * coupling_modifier) * local_density).exp()
    }

    pub fn export_mechanism_correlation_data(&self) -> serde_json::Value {
        let mut data = Vec::new();

        for knot in self.active_knots.values() {
            if knot.age < 50 {
                continue;
            }
            let mean_stab = if knot.vertices.is_empty() {
                0.0
            } else {
                knot.vertices
                    .iter()
                    .map(|v| self.stability.get(v).copied().unwrap_or(0.0))
                    .sum::<f64>()
                    / knot.vertices.len() as f64
            };

            let suppression = self.compute_local_suppression_for_knot(knot);
            let memory = knot.age as f64;
            let damping = (-mean_stab / 30.0).exp();

            data.push(serde_json::json!({
                "knot_id": knot.id,
                "age": knot.age,
                "stability": mean_stab,
                "coherence": knot.coherence,
                "suppression": suppression,
                "memory": memory,
                "damping": damping,
                "survived": true,
            }));
        }

        for knot in &self.dead_knots {
            if knot.age < 50 {
                continue;
            }
            let mean_stab = if knot.vertices.is_empty() {
                0.0
            } else {
                knot.vertices
                    .iter()
                    .map(|v| self.stability.get(v).copied().unwrap_or(0.0))
                    .sum::<f64>()
                    / knot.vertices.len() as f64
            };

            let suppression = self.compute_local_suppression_for_knot(knot);
            let memory = knot.age as f64;
            let damping = (-mean_stab / 30.0).exp();

            data.push(serde_json::json!({
                "knot_id": knot.id,
                "age": knot.age,
                "stability": mean_stab,
                "coherence": knot.coherence,
                "suppression": suppression,
                "memory": memory,
                "damping": damping,
                "survived": false,
            }));
        }

        serde_json::json!(data)
    }
}
