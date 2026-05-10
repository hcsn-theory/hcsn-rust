use hcsn_rust::hypergraph::Hypergraph;
use hcsn_rust::observables::{worldline_interaction_graph, TopologicalKnot};
use hcsn_rust::rewrite_engine::RewriteEngine;
use std::collections::HashMap;
use std::time::Instant;

const STEPS_DEFAULT: usize = 20000;
const TAU_C: usize = 600;
const COH_VALS: [f64; 5] = [1.2, 1.4, 1.6, 1.8, 2.0];
const OV_VALS: [f64; 4] = [0.5, 0.6, 0.7, 0.8];
const SEED: u64 = 42;

struct TrackerConfig {
    coh: f64,
    ov: f64,
}

struct TrackerState {
    config: TrackerConfig,
    active_knots: HashMap<u64, TopologicalKnot>,
    dead_knots: Vec<TopologicalKnot>,
    next_knot_id: u64,
    // (min_vertex_id) -> lifetime
    identity_map: HashMap<u64, usize>,
}

impl TrackerState {
    fn new(coh: f64, ov: f64) -> Self {
        Self {
            config: TrackerConfig { coh, ov },
            active_knots: HashMap::new(),
            dead_knots: Vec::new(),
            next_knot_id: 0,
            identity_map: HashMap::new(),
        }
    }

    fn record_final_identities(&mut self) {
        // Active ones
        for knot in self.active_knots.values() {
            if knot.age >= TAU_C {
                let id = *knot.vertices.iter().min().unwrap_or(&0);
                self.identity_map.insert(id, knot.age);
            }
        }
        // Dead ones
        for knot in &self.dead_knots {
            if knot.age >= TAU_C {
                let id = *knot.vertices.iter().min().unwrap_or(&0);
                self.identity_map.insert(id, knot.age);
            }
        }
    }
}

fn compute_mle_alpha(lifetimes: &[usize], x_min: f64) -> f64 {
    let n = lifetimes.len() as f64;
    if n < 5.0 {
        return 0.0;
    }
    let mut sum_ln = 0.0;
    for &l in lifetimes {
        sum_ln += (l as f64 / x_min).ln();
    }
    1.0 + (n / sum_ln)
}

fn compute_correlation(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len();
    if n < 2 {
        return 1.0;
    }
    let (mut sum_x, mut sum_y, mut sum_xx, mut sum_yy, mut sum_xy) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for i in 0..n {
        sum_x += x[i];
        sum_y += y[i];
        sum_xx += x[i] * x[i];
        sum_yy += y[i] * y[i];
        sum_xy += x[i] * y[i];
    }
    let num = n as f64 * sum_xy - sum_x * sum_y;
    let den = ((n as f64 * sum_xx - sum_x * sum_x) * (n as f64 * sum_yy - sum_y * sum_y)).sqrt();
    if den == 0.0 {
        0.0
    } else {
        num / den
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let steps = if args.len() > 2 && args[1] == "--steps" {
        args[2].parse().unwrap_or(STEPS_DEFAULT)
    } else {
        STEPS_DEFAULT
    };

    println!("Initializing HCSN Robustness Validation Pipeline...");
    println!("Parameters: {} steps, seed={}", steps, SEED);

    let mut h = Hypergraph::new();
    let v1 = h.add_vertex();
    let v2 = h.add_vertex();
    h.add_hyperedge(vec![v1.id, v2.id]);
    h.add_causal_relation(v1.id, v2.id);

    let mut engine = RewriteEngine::new(h, 0.65, Some(SEED));
    let mut trackers = Vec::new();
    for &coh in &COH_VALS {
        for &ov in &OV_VALS {
            trackers.push(TrackerState::new(coh, ov));
        }
    }

    let start = Instant::now();
    for step in 1..=steps {
        engine.step();

        if step % 10 == 0 {
            let inter = worldline_interaction_graph(&engine.h, 0.0);
            for tracker in trackers.iter_mut() {
                RewriteEngine::process_knot_update_static(
                    &engine.h,
                    &mut tracker.active_knots,
                    &mut tracker.dead_knots,
                    &mut tracker.next_knot_id,
                    &engine.stability,
                    engine.time,
                    tracker.config.coh,
                    tracker.config.ov,
                );
            }
        }

        if step % 5000 == 0 {
            println!(
                "Progress: {}/{} steps ({:.1}s)",
                step,
                steps,
                start.elapsed().as_secs_f64()
            );
        }
    }

    println!("\nSimulation Complete. Analyzing Results...");
    for t in trackers.iter_mut() {
        t.record_final_identities();
    }

    println!("| Coherence | Overlap | Count | Alpha | PSI | Lifetime Correlation (vs Baseline) |");
    println!("|-----------|---------|-------|-------|-----|-----------------------------------|");

    let baseline_idx = 0;
    for tracker in trackers.iter() {
        let lifetimes_all: Vec<usize> = tracker
            .dead_knots
            .iter()
            .map(|k| k.age)
            .chain(tracker.active_knots.values().map(|k| k.age))
            .collect();
        let total_candidates = lifetimes_all.len();
        let particle_lifetimes: Vec<usize> = lifetimes_all
            .iter()
            .cloned()
            .filter(|&l| l >= TAU_C)
            .collect();
        let particle_count = particle_lifetimes.len();
        let alpha = compute_mle_alpha(&particle_lifetimes, TAU_C as f64);
        let psi = if total_candidates > 0 {
            particle_count as f64 / total_candidates as f64
        } else {
            0.0
        };

        let mut x = Vec::new();
        let mut y = Vec::new();
        for (id, &life_baseline) in &trackers[baseline_idx].identity_map {
            if let Some(&life_target) = tracker.identity_map.get(id) {
                x.push(life_baseline as f64);
                y.push(life_target as f64);
            }
        }
        let correlation = compute_correlation(&x, &y);

        println!(
            "| {:<9.1} | {:<7.1} | {:<5} | {:<5.2} | {:<3.2} | {:<33.2} |",
            tracker.config.coh, tracker.config.ov, particle_count, alpha, psi, correlation
        );
    }

    let baseline = &trackers[baseline_idx];
    let lifetimes: Vec<usize> = baseline
        .dead_knots
        .iter()
        .map(|k| k.age)
        .chain(baseline.active_knots.values().map(|k| k.age))
        .collect();
    println!("\nHazard Rate Analysis (Baseline θ=1.2, OV=0.5):");
    println!("| Tau Range | Deaths | At Risk | Hazard Rate h(τ) |");
    println!("|-----------|--------|---------|------------------|");
    for i in 0..10 {
        let start_tau = i * 1000;
        let end_tau = (i + 1) * 1000;
        let at_risk = lifetimes.iter().filter(|&&l| l >= start_tau).count();
        let deaths = baseline
            .dead_knots
            .iter()
            .filter(|k| k.age >= start_tau && k.age < end_tau)
            .count();
        let h = if at_risk > 0 {
            deaths as f64 / at_risk as f64
        } else {
            0.0
        };
        println!(
            "| {}-{} | {:<6} | {:<7} | {:<16.4} |",
            start_tau, end_tau, deaths, at_risk, h
        );
    }
}
