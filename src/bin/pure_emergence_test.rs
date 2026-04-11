use std::collections::HashMap;
use hcsn_rust::hypergraph::Hypergraph;
use hcsn_rust::rewrite_engine::RewriteEngine;
use hcsn_rust::observables::compute_omega;
use serde::Serialize;
use std::fs;

#[derive(Serialize)]
struct PureStats {
    total_knots_detected: usize,
    max_lifetime: usize,
    mean_lifetime: f64,
    alpha: f64,
    emergence_level: String,
}

fn calculate_alpha(lifetimes: &[usize], x_min: f64) -> f64 {
    let filtered: Vec<f64> = lifetimes.iter()
        .filter(|&&l| l as f64 >= x_min)
        .map(|&l| l as f64)
        .collect();
    
    if filtered.is_empty() { return 0.0; }
    
    let n = filtered.len() as f64;
    let sum_log = filtered.iter().map(|&x| (x / x_min).ln()).sum::<f64>();
    1.0 + n / sum_log
}

fn main() {
    println!("=== HCSN PURE EMERGENCE TEST ===");
    println!("Goal: Test if particles emerge from rules alone (no memory/stability/coherence-gates).");
    
    let mut h = Hypergraph::new();
    // Seed with a small 4-vertex clique
    let v1 = h.add_vertex().id;
    let v2 = h.add_vertex().id;
    let v3 = h.add_vertex().id;
    let v4 = h.add_vertex().id;
    let nodes = vec![v1, v2, v3, v4];
    for i in 0..nodes.len() {
        for j in i + 1..nodes.len() {
            h.add_causal_relation(nodes[i], nodes[j]);
            h.add_hyperedge(vec![nodes[i], nodes[j]]);
        }
    }

    let mut engine = RewriteEngine::new(h, 0.60, None);
    engine.pure_mode = true;
    engine.verbose = false;
    engine.print_interval = 1000;

    let total_steps = 25000;
    println!("Running {} steps in pure_mode...", total_steps);

    let mut knot_counts = Vec::new();

    for i in 1..=total_steps {
        engine.step();
        if i % 1000 == 0 {
            let active = engine.active_knots.len();
            knot_counts.push(active);
            println!("  t={} | active_knots={} | Ω={:.4}", i, active, compute_omega(&hcsn_rust::observables::worldline_interaction_graph(&engine.h, 0.0)));
            
            // Eager Save
            let mut current_lifetimes = Vec::new();
            for k in &engine.dead_knots { current_lifetimes.push(k.age); }
            for k in engine.active_knots.values() { current_lifetimes.push(k.age); }
            
            let max_l = *current_lifetimes.iter().max().unwrap_or(&0);
            let mean_l = current_lifetimes.iter().sum::<usize>() as f64 / current_lifetimes.len() as f64;
            let alpha = calculate_alpha(&current_lifetimes, 50.0);
            
            let stats = PureStats {
                total_knots_detected: current_lifetimes.len(),
                max_lifetime: max_l,
                mean_lifetime: mean_l,
                alpha,
                emergence_level: "IN-PROGRESS".to_string(),
            };
            
            fs::create_dir_all("exports").ok();
            let _ = fs::write("exports/particle_lifetimes_pure.json", serde_json::to_string_pretty(&current_lifetimes).unwrap());
            let _ = fs::write("exports/pure_emergence_summary.json", serde_json::to_string_pretty(&stats).unwrap());
            println!("  [Eager Save] Progress persisted at t={}", i);
        }
    }

    println!("\nAnalysis of Resulting Structures:");
    
    let mut lifetimes = Vec::new();
    for k in &engine.dead_knots {
        lifetimes.push(k.age);
    }
    for k in engine.active_knots.values() {
        lifetimes.push(k.age);
    }

    if lifetimes.is_empty() {
        println!("CONCLUSION: No structures detected.");
        return;
    }

    let max_l = *lifetimes.iter().max().unwrap_or(&0);
    let mean_l = lifetimes.iter().sum::<usize>() as f64 / lifetimes.len() as f64;
    let alpha = calculate_alpha(&lifetimes, 50.0);

    let emergence = if max_l < 200 {
        "NO EMERGENCE (Structures are short-lived artifacts)"
    } else if max_l >= 1000 {
        if alpha > 1.0 && alpha < 3.0 {
            "TRUE EMERGENCE (Stable power-law lifetimes detected)"
        } else {
            "WEAK EMERGENCE (Long-lived structures exist but lack scaling)"
        }
    } else {
        "WEAK EMERGENCE (Intermediate lifetimes detected)"
    };

    println!("--------------------------------------------------");
    println!("Max Lifetime:    {}", max_l);
    println!("Mean Lifetime:   {:.2}", mean_l);
    println!("Alpha (MLE):     {:.3}", alpha);
    println!("Knot Count (t_end): {}", engine.active_knots.len());
    println!("CONCLUSION:      {}", emergence);
    println!("--------------------------------------------------");

    // Export
    let stats = PureStats {
        total_knots_detected: lifetimes.len(),
        max_lifetime: max_l,
        mean_lifetime: mean_l,
        alpha,
        emergence_level: emergence.to_string(),
    };

    fs::create_dir_all("exports").unwrap();
    fs::write("exports/particle_lifetimes_pure.json", serde_json::to_string_pretty(&lifetimes).unwrap()).unwrap();
    fs::write("exports/pure_emergence_summary.json", serde_json::to_string_pretty(&stats).unwrap()).unwrap();

    // Hazard Rate Table
    println!("\nHazard Rate Analysis (Pure Mode):");
    let mut death_buckets: HashMap<usize, usize> = HashMap::new();
    let mut life_buckets: HashMap<usize, usize> = HashMap::new();
    let bucket_size = 500;

    for &l in &lifetimes {
        let b = l / bucket_size;
        for i in 0..=b {
            *life_buckets.entry(i).or_insert(0) += 1;
        }
        *death_buckets.entry(b).or_insert(0) += 1;
    }

    println!("| Tau Range | Deaths | At Risk | Hazard Rate h(τ) |");
    println!("|-----------|--------|---------|------------------|");
    for i in 0..10 {
        let start = i * bucket_size;
        let end = (i + 1) * bucket_size;
        let deaths = death_buckets.get(&i).unwrap_or(&0);
        let at_risk = life_buckets.get(&i).unwrap_or(&0);
        if *at_risk > 0 {
            let h_tau = *deaths as f64 / *at_risk as f64;
            println!("| {}-{} | {} | {} | {:.4} |", start, end, deaths, at_risk, h_tau);
        }
    }
}
