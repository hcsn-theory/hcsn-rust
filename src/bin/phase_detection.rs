use hcsn_rust::hypergraph::Hypergraph;
use hcsn_rust::observables::compute_omega;
use hcsn_rust::rewrite_engine::RewriteEngine;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Serialize)]
struct PhaseStats {
    alpha: f64,
    max_lifetime: usize,
    mean_lifetime: f64,
    regime: String,
    hazard_trend: String,
}

fn calculate_alpha(lifetimes: &[usize], x_min: f64) -> f64 {
    let filtered: Vec<f64> = lifetimes
        .iter()
        .filter(|&&l| l as f64 >= x_min)
        .map(|&l| l as f64)
        .collect();

    if filtered.is_empty() {
        return 0.0;
    }

    let n = filtered.len() as f64;
    let sum_log = filtered.iter().map(|&x| (x / x_min).ln()).sum::<f64>();
    1.0 + n / sum_log
}

fn main() {
    // Performance safety: Limit threads
    env::set_var("RAYON_NUM_THREADS", "4");

    println!("=== HCSN PHASE-DETECTION EXPERIMENT ===");
    println!("Goal: Find the Critical Particle Regime (1.8 <= α <= 2.2)");

    // Set parameters for Phase 10c: Search for Criticality
    let p_create = 0.67;
    let nu = 0.975;
    let gamma = 2.0;
    let mu = 0.3;

    env::set_var("HCSN_NU", nu.to_string());
    env::set_var("HCSN_GAMMA", gamma.to_string());
    env::set_var("HCSN_MU", mu.to_string());

    println!("Configuration:");
    println!("  p_create: {}", p_create);
    println!("  nu (stability_decay): {}", nu);
    println!("  gamma (nonlinear_exp): {}", gamma);
    println!("  mu (memory_coupling): {}", mu);
    println!("  threads: 4");

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

    let mut engine = RewriteEngine::new(h, p_create, None);
    engine.pure_mode = false; // Enable stabilization/memory
    engine.verbose = false;
    engine.print_interval = 1000;

    let total_steps = 15000;
    println!("\nRunning {} steps...", total_steps);

    for i in 1..=total_steps {
        engine.step();
        if i % 1000 == 0 {
            let active = engine.active_knots.len();
            let omega = hcsn_rust::observables::compute_omega_graph(&hcsn_rust::observables::worldline_interaction_graph(
                &engine.h, 0.0,
            ));
            println!("  t={} | active_knots={} | Ω={:.4}", i, active, omega);
        }
    }

    println!("\n=== Final Analysis ===");

    let mut lifetimes = Vec::new();
    for k in &engine.dead_knots {
        lifetimes.push(k.age);
    }
    for k in engine.active_knots.values() {
        lifetimes.push(k.age);
    }

    if lifetimes.is_empty() {
        println!("ERROR: No structures detected. Try increasing p_create.");
        return;
    }

    let x_min = 100.0;
    let max_l = *lifetimes.iter().max().unwrap_or(&0);
    let mean_l = lifetimes.iter().sum::<usize>() as f64 / lifetimes.len() as f64;
    let alpha = calculate_alpha(&lifetimes, x_min);

    let regime = if alpha < 1.5 {
        "CONDENSED (Over-stable, solid-like)"
    } else if alpha >= 1.8 && alpha <= 2.2 {
        "CRITICAL PARTICLE REGIME ✅"
    } else if alpha > 2.5 {
        "METASTABLE / EVAPORATING"
    } else {
        "TRANSITIONAL"
    };

    // Hazard Rate
    let mut death_buckets: HashMap<usize, usize> = HashMap::new();
    let mut life_buckets: HashMap<usize, usize> = HashMap::new();
    let bucket_size = 500;
    let mut hazards = Vec::new();

    for &l in &lifetimes {
        let b = l / bucket_size;
        for i in 0..=b {
            *life_buckets.entry(i).or_insert(0) += 1;
        }
        *death_buckets.entry(b).or_insert(0) += 1;
    }

    println!("Hazard Rate Analysis:");
    println!("| Tau Range | Deaths | At Risk | h(τ) |");
    for i in 0..5 {
        let start = i * bucket_size;
        let end = (i + 1) * bucket_size;
        let deaths = *death_buckets.get(&i).unwrap_or(&0);
        let at_risk = *life_buckets.get(&i).unwrap_or(&0);
        if at_risk > 0 {
            let h = deaths as f64 / at_risk as f64;
            hazards.push(h);
            println!(
                "| {}-{} | {} | {} | {:.4} |",
                start, end, deaths, at_risk, h
            );
        }
    }

    let hazard_behavior = if hazards.len() >= 2 {
        if hazards[0] > *hazards.last().unwrap() {
            "DECREASING"
        } else if hazards[0] < *hazards.last().unwrap() {
            "INCREASING"
        } else {
            "FLAT"
        }
    } else {
        "INSUFFICIENT DATA"
    };

    println!("\n----------------Summary------------------");
    println!("α = {:.3}", alpha);
    println!("max lifetime = {}", max_l);
    println!("hazard behavior = {}", hazard_behavior);
    println!("regime = {}", regime);
    println!("-----------------------------------------");

    if alpha >= 1.8 && alpha <= 2.2 {
        println!("\nREADY FOR PROMPT 2: Force Law Extraction");
    } else {
        println!("\nADJUSTMENT NEEDED:");
        if alpha < 1.8 {
            println!("  Action: Increase p_create to push toward criticality.");
        } else {
            println!("  Action: Decrease p_create to stabilize evaporation.");
        }
    }

    // Export results
    let stats = PhaseStats {
        alpha,
        max_lifetime: max_l,
        mean_lifetime: mean_l,
        regime: regime.to_string(),
        hazard_trend: hazard_behavior.to_string(),
    };
    fs::create_dir_all("exports").ok();
    let _ = fs::write(
        "exports/phase_detection_summary.json",
        serde_json::to_string_pretty(&stats).unwrap(),
    );
    let _ = fs::write(
        "exports/phase_detection_lifetimes.json",
        serde_json::to_string_pretty(&lifetimes).unwrap(),
    );
}
