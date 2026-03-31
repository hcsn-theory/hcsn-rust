use std::collections::{HashMap, HashSet};
use std::time::Instant;
use rand::Rng;
use serde::Serialize;
use rand::seq::SliceRandom;

use crate::hypergraph::Hypergraph;
use crate::physics_params::PhysicsParams;
use crate::rules::{edge_creation_rule, UndoRecord};
use crate::observables::{
    worldline_interaction_graph, hierarchical_closure, TopologicalKnot, InteractionEvent, 
    detect_candidate_knots, component_radius, compute_coherence_raw,
};

#[derive(Serialize, Clone)]
pub struct DefectLogEntry {
    pub time: usize,
}

#[derive(Serialize, Clone)]
pub struct XiCurrentLogEntry {
    pub time: usize,
    pub delta_xi: HashMap<u64, f64>,
}

pub struct RewriteEngine {
    pub h: Hypergraph,
    pub p_create: f64,
    
    // Physics params
    pub gamma_time: f64,
    pub gamma_ext: f64,
    pub gamma_closure: f64,
    pub gamma_hier: f64,
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
    pub forced_time: Option<usize>,
    pub time: usize,
    pub verbose: bool,
    pub print_interval: usize,

    // caching
    cached_inter: Option<HashMap<u64, HashSet<u64>>>,
    cached_omega: Option<f64>,
    
    // probes
    pending_bridge: Option<(u64, u64)>,
    pending_bridge_time: Option<usize>,
    
    last_step_time: f64,
    pub attempted_rewrites: usize,
    pub suppressed_rewrites: usize,
    
    // Per-vertex stability memory: accumulates for vertices in coherent structures
    pub stability: HashMap<u64, f64>,
    
    // Phase 8: Coupling Pulse tracking
    pub coupled_vertices: HashSet<u64>,
    pub active_interactions: HashMap<(u64, u64), InteractionEvent>,
}



impl RewriteEngine {
    pub fn new(h: Hypergraph, p_create: f64, _seed: Option<u64>) -> Self {
        // Rust's rand::thread_rng() replaces the fixed seed initialization for now,
        // though a SmallRng could be seeded specifically.

        Self {
            h,
            p_create,
            gamma_time: 0.1,
            gamma_ext: 0.05,
            gamma_closure: 0.05,
            gamma_hier: 0.06,
            epsilon_label_violation: 0.08,
            params: PhysicsParams::new(),
            
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
            forced_time: None,
            time: 0,
            verbose: true,
            print_interval: 50,
            
            cached_inter: None,
            cached_omega: None,
            pending_bridge: None,
            pending_bridge_time: None,
            
            last_step_time: 0.0,
            attempted_rewrites: 0,
            suppressed_rewrites: 0,
            stability: HashMap::new(),
            coupled_vertices: HashSet::new(),
            active_interactions: HashMap::new(),
        }
    }

    // --------------------------------------------------
    // Main step
    // --------------------------------------------------
    pub fn step(&mut self) -> bool {
        self.time += 1;
        let t0 = Instant::now();
        self.prev_xi = self.xi.clone();

        let inter_before = if let Some(inter) = &self.cached_inter {
            inter.clone()
        } else {
            worldline_interaction_graph(&self.h, 0.0)
        };
        
        let omega_before = self.cached_omega.unwrap_or(0.0);

        // Experiment C: Spontaneous Vacuum Nucleation
        if self.params.defect_injection > 0.0 {
            let mut rng = rand::thread_rng();
            if rng.gen::<f64>() < self.params.defect_injection {
                let v1 = self.h.add_vertex();
                let v2 = self.h.add_vertex();
                let v3 = self.h.add_vertex();
                let v4 = self.h.add_vertex();
                
                let nodes = vec![v1.id, v2.id, v3.id, v4.id];
                for i in 0..4 {
                    for j in i+1..4 {
                        self.h.add_causal_relation(nodes[i], nodes[j]);
                        self.h.add_hyperedge(vec![nodes[i], nodes[j]]);
                    }
                }
            }
        }

        // ---------------------------------
        // Propose rewrite
        // ---------------------------------
        let undo_opt = self.propose_rewrite(&inter_before);
        if undo_opt.is_none() && self.time % 200 == 0 {
            if self.verbose {
                println!("[debug] rewrite skipped at t = {}", self.time);
            }
        }

        let undo = match undo_opt {
            Some(u) => u,
            None => return false,
        };

        // Cache last rewrite internally
        // (Just cloning IDs to find touched vertices later)
        self.last_rewrite = Some(UndoRecord {
            added_vertices: undo.added_vertices.clone(),
            removed_vertex: undo.removed_vertex.clone(),
            kept_vertex: undo.kept_vertex,
            added_edges: Vec::new(),
            added_causal: Vec::new(),
            removed_edges: HashMap::new(),
            old_causal: HashMap::new(),
        });

        // ---------------------------------
        // Tentative interaction graph
        // ---------------------------------
        let inter_after = worldline_interaction_graph(&self.h, 0.0);
        
        if self.time % 200 == 0 {
            if self.verbose {
                println!("[debug] interaction nodes = {}", inter_after.len());
            }
        }

        let omega_after = if self.time % 50 == 0 {
            hierarchical_closure(&self.h, &inter_after)
        } else {
            omega_before
        };
        
        let delta_omega = omega_after - omega_before;

        // ---------------------------------
        // Acceptance rule
        // ---------------------------------
        let mut accept_prob = 1.0;
        if delta_omega.abs() > self.epsilon_label_violation {
            let v_len = self.h.vertices.len() as f64;
            let gamma = self.params.gamma_defect * (-v_len / 800.0).exp();
            accept_prob *= (-gamma * delta_omega.abs()).exp();
        }

        // Experiment B: Targeted Metropolis-Hastings Noise Bias
        if self.params.noise_bias > 0.0 && delta_omega > 0.0 {
            accept_prob *= (self.params.noise_bias * delta_omega).exp();
            if accept_prob > 1.0 { accept_prob = 1.0; }
        }

        let mut rng = rand::thread_rng();
        let accepted = rng.gen::<f64>() <= accept_prob;
        let omega_print;

        if !accepted {
            self.undo_changes(undo);
            self.cached_inter = Some(inter_before);
            self.cached_omega = Some(omega_before);
            omega_print = omega_before;
        } else {
            // Cache accepted state
            self.cached_inter = Some(inter_after.clone());
            self.cached_omega = Some(omega_after);
            omega_print = omega_after;

            // -----------------------------
            // ξ inheritance
            // -----------------------------
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

            // -----------------------------
            // ξ propagation
            // -----------------------------
            let xi_clusters = self.xi_clusters(&inter_after);
            self.propagate_xi(&inter_after, &xi_clusters);
            
            let geom_inter = inter_after.clone();
            
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
                let xi_support: HashSet<u64> = self.xi.iter()
                    .filter(|(&_v, &x)| x > self.xi_threshold && x.is_finite())
                    .map(|(v, _)| *v)
                    .collect();

                if xi_support.len() >= 2 {
                    // Update topo
                    self.update_topo_distance_memory(&geom_inter, &xi_support);
                    // Update xi (fallback logic included inside)
                    self.update_xi_distance_memory(&geom_inter);
                }
            }

            self.record_xi_current(&geom_inter);
            
            if self.time % 10 == 0 {
                self.update_topological_knots(&geom_inter);
                self.update_stability(&geom_inter);
            }
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
            let valid_knots = self.active_knots.values().filter(|k| k.age >= 50 && k.radius < 5.0).count();
            let geom_pairs = self.topo_distance_memory.len() + self.xi_distance_memory.len();
            
            let supp_ratio = if self.attempted_rewrites > 0 {
                self.suppressed_rewrites as f64 / self.attempted_rewrites as f64
            } else { 0.0 };

            println!(
                "[engine] t={} step={:.2}ms Ω={:.6} knots={} geom_pairs={} supp_ratio={:.3}",
                self.time,
                self.last_step_time * 1000.0,
                omega_print,
                valid_knots,
                geom_pairs,
                supp_ratio
            );
            
            // reset stats for window
            self.attempted_rewrites = 0;
            self.suppressed_rewrites = 0;
        }

        accepted
    }

    pub fn update_topological_knots(&mut self, inter: &HashMap<u64, HashSet<u64>>) {
        let candidates = detect_candidate_knots(&self.h, inter, 1.2);

        let mut next_active_knots = HashMap::new();
        
        // Step 1: Track matches for velocity and pre-stats
        let mut knot_pre_stats: HashMap<u64, (f64, usize, f64)> = HashMap::new();
        for (id, knot) in &self.active_knots {
            let stab = knot.vertices.iter()
                .map(|v| self.stability.get(v).copied().unwrap_or(0.0))
                .sum::<f64>() / (knot.vertices.len() as f64).max(1.0);
            knot_pre_stats.insert(*id, (knot.coherence, knot.vertices.len(), stab));
        }

        // Step 2: Update existing knots
        let mut matched_candidates = HashSet::new();
        for (_, knot) in &self.active_knots {
            let mut best_idx = None;
            let mut best_overlap = 0.0;
            
            for (i, cand) in candidates.iter().enumerate() {
                let intersection = knot.vertices.intersection(cand).count() as f64;
                let min_s = knot.vertices.len().min(cand.len()) as f64;
                let overlap = if min_s > 0.0 { intersection / min_s } else { 0.0 };
                
                if overlap > 0.3 && overlap > best_overlap {
                    best_overlap = overlap;
                    best_idx = Some(i);
                }
            }
            
            if let Some(idx) = best_idx {
                matched_candidates.insert(idx);
                let cand = &candidates[idx];
                let (ie, be) = compute_coherence_raw(cand, inter);
                let coherence = if be > 0 { ie as f64 / be as f64 } 
                                else if ie > 0 { 10.0 } else { 0.0 };
                
                let mut updated_knot = knot.clone();
                updated_knot.vertices = cand.clone();
                updated_knot.age += 10;
                updated_knot.radius = component_radius(cand, inter);
                updated_knot.coherence = coherence;
                if cand.len() > updated_knot.max_size { updated_knot.max_size = cand.len(); }
                if cand.len() < updated_knot.min_size { updated_knot.min_size = cand.len(); }
                
                // Velocity: dist between centroids / time step
                let mean_pos = cand.iter().map(|&v| v as f64).sum::<f64>() / cand.len() as f64;
                let prev_pos = knot.position_history.last().map(|(_, p, _)| *p).unwrap_or(mean_pos);
                updated_knot.velocity = (mean_pos - prev_pos).abs() / 10.0;
                
                updated_knot.position_history.push((self.time, mean_pos, coherence));
                if updated_knot.position_history.len() > 100 {
                    updated_knot.position_history.drain(..updated_knot.position_history.len()-100);
                }
                
                next_active_knots.insert(updated_knot.id, updated_knot);
            } else {
                if knot.age >= 50 {
                    self.dead_knots.push(knot.clone());
                }
            }
        }

        // Step 3: Formal Kinematics
        self.coupled_vertices.clear();
        let active_ids: Vec<_> = next_active_knots.keys().copied().collect();
        for &id in &active_ids {
            let knot = next_active_knots.get_mut(&id).unwrap();
            knot.mass = knot.vertices.len() as f64 * knot.coherence.powi(2);
            let hist = &knot.position_history;
            if hist.len() > 1 {
                let (t1, p1, _) = hist[0];
                let (t2, p2, _) = hist[hist.len()-1];
                let dt = (t2 - t1) as f64;
                if dt > 0.0 {
                    knot.velocity_avg = ((p2 - p1) / dt, 0.0);
                }
            }
            knot.momentum = knot.mass * knot.velocity_avg.0;
        }
            
        // Detect overlaps and manage persistent InteractionEvents
        let mut current_overlaps = HashSet::new();
        for i in 0..active_ids.len() {
            for j in i+1..active_ids.len() {
                let id_a = active_ids[i];
                let id_b = active_ids[j];
                let knot_a = &next_active_knots[&id_a];
                let knot_b = &next_active_knots[&id_b];
                
                let intersection = knot_a.vertices.intersection(&knot_b.vertices).count();
                if intersection > 0 {
                    let pair = if id_a < id_b { (id_a, id_b) } else { (id_b, id_a) };
                    current_overlaps.insert(pair);
                    
                    let chi = intersection as f64 / (knot_a.vertices.len().min(knot_b.vertices.len()) as f64);
                    let res = (2.0 * knot_a.coherence * knot_b.coherence) / 
                              (knot_a.coherence.powi(2) + knot_b.coherence.powi(2)).max(1e-6);

                    if !self.active_interactions.contains_key(&pair) {
                        let pre_a_tuple = knot_pre_stats.get(&id_a).cloned().unwrap_or((0.0, 0, 0.0));
                        let pre_b_tuple = knot_pre_stats.get(&id_b).cloned().unwrap_or((0.0, 0, 0.0));
                        let m_a = pre_a_tuple.1 as f64 * pre_a_tuple.0.powi(2);
                        let m_b = pre_b_tuple.1 as f64 * pre_b_tuple.0.powi(2);

                        self.active_interactions.insert(pair, InteractionEvent {
                            time: self.time, knot_a: id_a, knot_b: id_b, overlap_size: intersection,
                            overlap_depth: chi, resonance: res,
                            pre_a: (m_a, knot_a.velocity, m_a * knot_a.velocity, knot_a.velocity_avg),
                            pre_b: (m_b, knot_b.velocity, m_b * knot_b.velocity, knot_b.velocity_avg),
                            post_a: None, post_b: None,
                        });
                    }
                    
                    if chi > 0.4 {
                        for &v in &knot_a.vertices { self.coupled_vertices.insert(v); }
                        for &v in &knot_b.vertices { self.coupled_vertices.insert(v); }
                    }
                }
            }
        }

        // Finalize interactions that have ended
        let mut finished = Vec::new();
        for (&pair, ev) in &mut self.active_interactions {
            if !current_overlaps.contains(&pair) {
                let (id_a, id_b) = pair;
                if let Some(k_a) = next_active_knots.get(&id_a) {
                    ev.post_a = Some((k_a.mass, k_a.velocity, k_a.momentum, k_a.velocity_avg));
                }
                if let Some(k_b) = next_active_knots.get(&id_b) {
                    ev.post_b = Some((k_b.mass, k_b.velocity, k_b.momentum, k_b.velocity_avg));
                }
                finished.push(pair);
            }
        }
        for pair in finished {
            if let Some(ev) = self.active_interactions.remove(&pair) {
                self.interaction_events.push(ev);
            }
        }
        
        // Step 4: Handle new candidates
        for (idx, cand) in candidates.iter().enumerate() {
            if !matched_candidates.contains(&idx) {
                let (ie, be) = compute_coherence_raw(cand, inter);
                let coherence = if be > 0 { ie as f64 / be as f64 } 
                                else if ie > 0 { 10.0 } else { 0.0 };
                
                if coherence > 1.1 || cand.len() > 10 {
                    let mut new_knot = TopologicalKnot {
                        id: self.time as u64 * 1000 + idx as u64,
                        vertices: cand.clone(),
                        age: 10,
                        max_size: cand.len(),
                        min_size: cand.len(),
                        radius: component_radius(cand, inter),
                        coherence,
                        velocity: 0.0,
                        velocity_avg: (0.0, 0.0),
                        mass: cand.len() as f64 * coherence.powi(2),
                        momentum: 0.0,
                        position_history: Vec::new(),
                    };
                    let mean_pos = cand.iter().map(|&v| v as f64).sum::<f64>() / cand.len() as f64;
                    new_knot.position_history.push((self.time, mean_pos, coherence));
                    next_active_knots.insert(new_knot.id, new_knot);
                }
            }
        }

        
        self.active_knots = next_active_knots;
    }

    /// Update per-vertex stability memory.
    /// Vertices inside active knots accumulate stability.
    /// All stability values decay slowly each cycle.
    fn update_stability(&mut self, _inter: &HashMap<u64, HashSet<u64>>) {
        let stability_decay = 0.975; // Lowered to push back to critical regime (was 0.985)
        let stability_gain = 1.0;
        
        // Decay all existing stability
        for val in self.stability.values_mut() {
            *val *= stability_decay;
        }
        
        // Accumulate stability for vertices in active knots
        for knot in self.active_knots.values() {
            for &v in &knot.vertices {
                let entry = self.stability.entry(v).or_insert(0.0);
                *entry += stability_gain;
            }
        }
        
        // Clean up: remove entries for vertices no longer in the graph
        self.stability.retain(|v, _| self.h.vertices.contains_key(v));
        
        // Cap stability to prevent runaway
        for val in self.stability.values_mut() {
            if *val > 30.0 { *val = 30.0; }
        }
    }

    // --------------------------------------------------
    // Rewrite proposal
    // --------------------------------------------------
    fn propose_rewrite(&mut self, inter: &HashMap<u64, HashSet<u64>>) -> Option<UndoRecord> {
        let mut rng = rand::thread_rng();

        self.attempted_rewrites += 1;

        let vertices: Vec<u64> = self.h.vertices.keys().copied().collect();
        if vertices.is_empty() { return None; }
        let anchor_v = *vertices.choose(&mut rng).unwrap();

        // --- Density metric ---
        let clustering = crate::observables::local_clustering(inter, anchor_v);
        let degree = inter.get(&anchor_v).map(|n| n.len()).unwrap_or(0) as f64;
        
        let total_degree: usize = inter.values().map(|n| n.len()).sum();
        let avg_degree = if inter.is_empty() { 1.0 } else { (total_degree as f64) / (inter.len() as f64) }.max(1.0);
        
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
            let neighborhood: HashSet<u64> = {
                let mut s = neighbors.clone();
                s.insert(anchor_v);
                s
            };
            
            for &n in &neighborhood {
                if let Some(n_nbrs) = inter.get(&n) {
                    for &nn in n_nbrs {
                        if neighborhood.contains(&nn) {
                            internal_edges += 1;
                        } else {
                            boundary_edges += 1;
                        }
                    }
                }
            }
            internal_edges /= 2;
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
        let mu = 0.3; // memory coupling
        let gamma = 2.2; // Phase 10b: Production target (alpha ≈ 2.26)
        let stability_cap = 30.0_f64;
        let vertex_stability = self.stability.get(&anchor_v).copied().unwrap_or(0.0);
        let normalized_stability = (vertex_stability / stability_cap).min(1.0);
        let memory_contribution = mu * stability_cap * normalized_stability.powf(gamma);
        
        let alpha_eff = alpha_base + coherence_boost + memory_contribution;
        
        // Phase 8: Coupling Pulse - reduce protection during deep overlap
        let coupling_modifier = if self.coupled_vertices.contains(&anchor_v) { 0.2 } else { 1.0 };
        let rewrite_prob = (-(alpha_eff * coupling_modifier) * local_density).exp();
        
        if rng.gen::<f64>() > rewrite_prob {
            self.suppressed_rewrites += 1;
            return None; // Suppressed (density + structural protection)
        }

        let theta = 1.3; // Nucleation threshold
        let beta = 1.5;
        let growth = if coherence > theta { beta } else { 0.0 };

        // --- (3) Boundary tension: inhibits growth at high boundary ratio ---
        let boundary_ratio = if coherence > 0.0 { 1.0 / coherence } else { 10.0 };
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
    fn propagate_xi(&mut self, inter: &HashMap<u64, HashSet<u64>>, clusters: &HashMap<u64, usize>) {
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

            let empty_set = HashSet::new();
            let neighbors = inter.get(&v).unwrap_or(&empty_set);
            let deg = (neighbors.len() as f64).max(1.0);

            for &u in neighbors {
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
    fn xi_clusters(&self, inter: &HashMap<u64, HashSet<u64>>) -> HashMap<u64, usize> {
        let mut clusters = HashMap::new();
        let mut visited = HashSet::new();
        let mut cid = 0;

        let xi_vertices: HashSet<u64> = self.xi.iter()
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
                    for &w in nbrs {
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
    fn topo_clusters(&self, inter: &HashMap<u64, HashSet<u64>>) -> HashMap<u64, usize> {
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
                    for &w in nbrs {
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
    fn update_topo_distance_memory(&mut self, inter: &HashMap<u64, HashSet<u64>>, restrict_to: &HashSet<u64>) {
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
                    let new_val = self.distance_memory_decay * prev + (1.0 - self.distance_memory_decay) * d_f;
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
                    let new_val = self.distance_memory_decay * prev + (1.0 - self.distance_memory_decay) * d_f;
                    self.topo_distance_memory.insert(key, new_val);
                }
            }
        }
    }

    // --------------------------------------------------
    // Cluster distance memory (G2)
    // --------------------------------------------------
    fn update_xi_distance_memory(&mut self, inter: &HashMap<u64, HashSet<u64>>) {
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
                    let new_val = self.distance_memory_decay * prev + (1.0 - self.distance_memory_decay) * d_f;
                    self.xi_distance_memory.insert(key, new_val);
                    if self.verbose {
                        println!("[geom-add] xi_pair (intra-cluster) ({}, {}) d={}", cluster_ids[0], cluster_ids[0], min_d);
                    }
                }
            }
        }

        for i in 0..cluster_ids.len() {
            for j in (i + 1)..cluster_ids.len() {
                let mut a_verts = cluster_to_vertices[&cluster_ids[i]].clone();
                let b_verts: HashSet<u64> = cluster_to_vertices[&cluster_ids[j]].iter().cloned().collect();

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
                    let new_val = self.distance_memory_decay * prev + (1.0 - self.distance_memory_decay) * d_f;
                    self.xi_distance_memory.insert(key, new_val);
                    
                    if self.verbose {
                        println!("[geom-add] xi_pair ({}, {}) d={}", cluster_ids[i], cluster_ids[j], min_d);
                    }
                }
            }
        }
    }

    fn record_xi_current(&mut self, _inter: &HashMap<u64, HashSet<u64>>) {
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
        inter: &HashMap<u64, HashSet<u64>>,
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
                    for &u in nbrs {
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

    fn undo_changes(&mut self, undo: UndoRecord) {
        if let Some(v) = undo.removed_vertex {
            self.h.vertices.insert(v.id, v.clone());
            self.h.causal_order.insert(v.id, HashSet::new());
        }

        for (eid, e) in undo.removed_edges {
            self.h.hyperedges.insert(eid, e);
        }

        for eid in undo.added_edges {
            self.h.hyperedges.remove(&eid);
        }

        for vid in undo.added_vertices {
            self.h.vertices.remove(&vid);
            self.h.causal_order.remove(&vid);
        }
    }

    // --------------------------------------------------
    // Forced probes (matter injection)
    // --------------------------------------------------
    pub fn force_defect(&mut self, magnitude: f64, max_tries: usize) -> bool {
        if self.h.vertices.is_empty() {
            return false;
        }

        let mut rng = rand::thread_rng();
        let vertex_ids: Vec<u64> = self.h.vertices.keys().cloned().collect();

        for _ in 0..max_tries {
            let vid = *vertex_ids.choose(&mut rng).unwrap();
            let undo = edge_creation_rule(&mut self.h, Some(vid), self.p_create);

            if let Some(u) = undo {
                *self.xi.entry(vid).or_insert(0.0) += magnitude;
                self.forced_time = Some(self.time);
                
                if self.verbose {
                    println!("[inject] defect at t={} v={}", self.time, vid);
                }
                
                // Track rewrite internally
                self.last_rewrite = Some(UndoRecord {
                    added_vertices: u.added_vertices.clone(),
                    removed_vertex: u.removed_vertex.clone(),
                    kept_vertex: u.kept_vertex.clone(),
                    added_edges: Vec::new(),
                    added_causal: Vec::new(),
                    removed_edges: HashMap::new(),
                    old_causal: HashMap::new(),
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

        let mut rng = rand::thread_rng();

        // restrict candidates to the same connected component to avoid `inf` distances
        let mut reachable = xi_support.clone();
        let mut frontier = xi_support.clone();
        let mut depth = 0;
        
        while !frontier.is_empty() && depth < max_depth {
            depth += 1;
            let mut nxt = HashSet::new();
            for &v in &frontier {
                if let Some(nbrs) = inter.get(&v) {
                    for &u in nbrs {
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
            let vid = *candidates.choose(&mut rng).unwrap();
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
                println!("### SECOND PROBE (fallback) at t={} | v={} | d={}", self.time, vid, best_d);
            }
            return true;
        }

        false
    }
}
