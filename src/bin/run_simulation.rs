use std::time::Instant;
use serde::Serialize;
use hcsn_rust::hypergraph::Hypergraph;
use hcsn_rust::rewrite_engine::RewriteEngine;
use hcsn_rust::observables::{
    worldline_interaction_graph,
    interaction_concentration,
    closure_density,
    hierarchical_closure,
};

#[derive(Serialize)]
struct Config {
    seed: u64,
    max_steps: usize,
    sample_interval: usize,
    p_create: f64,
    noise_bias: f64,
    defect_injection: f64,
    geometry_freeze: f64,
}

fn main() {
    let mut config = Config {
        seed: 1,
        max_steps: 5000,
        sample_interval: 1000,
        p_create: 0.60,
        noise_bias: 0.0,
        defect_injection: 0.0,
        geometry_freeze: 0.9,
    };

    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--steps" && i + 1 < args.len() {
            if let Ok(m) = args[i+1].parse::<usize>() {
                config.max_steps = m;
            }
            i += 2;
        } else if args[i] == "--p_create" && i + 1 < args.len() {
            if let Ok(p) = args[i+1].parse::<f64>() {
                config.p_create = p;
            }
            i += 2;
        } else if args[i] == "--noise_bias" && i + 1 < args.len() {
            if let Ok(p) = args[i+1].parse::<f64>() { config.noise_bias = p; }
            i += 2;
        } else if args[i] == "--defect_injection" && i + 1 < args.len() {
            if let Ok(p) = args[i+1].parse::<f64>() { config.defect_injection = p; }
            i += 2;
        } else if args[i] == "--geometry_freeze" && i + 1 < args.len() {
            if let Ok(p) = args[i+1].parse::<f64>() { config.geometry_freeze = p; }
            i += 2;
        } else {
            i += 1;
        }
    }

    println!("\n======================================================================================");
    println!("RUN STARTED (Rust Engine)");
    println!("======================================================================================");
    
    println!(
        " time  |   V   |  <k>  | Δ<k> |  L  | ΔL |    Φ    |    Ψ    | acc%   |   omega   |  domega |knots|all_k| max_coh | supp% | step_ms"
    );

    let mut h = Hypergraph::new();
    let v1 = h.add_vertex();
    let v2 = h.add_vertex();
    
    h.add_causal_relation(v1.id, v2.id);
    h.add_hyperedge(vec![v1.id, v2.id]);

    let mut engine = RewriteEngine::new(h, config.p_create, Some(config.seed));
    engine.params.noise_bias = config.noise_bias;
    engine.params.defect_injection = config.defect_injection;
    engine.distance_memory_decay = config.geometry_freeze;
    engine.verbose = false;
    
    let mut accepted = 0;
    let mut rejected = 0;

    let mut last_k = engine.h.average_coordination();
    let mut last_l = engine.h.max_chain_length();
    let inter_start = worldline_interaction_graph(&engine.h, 0.0);
    let mut last_omega = hierarchical_closure(&engine.h, &inter_start);
    let mut prev_total_stability = 0.0;
    let start_time = Instant::now();
    let mut last_print_time = Instant::now();

    for _ in 1..=config.max_steps {
        let success = engine.step();
        
        if success {
            accepted += 1;
        } else {
            rejected += 1;
        }

        if engine.time % config.sample_interval != 0 {
            continue;
        }

        let inter = worldline_interaction_graph(&engine.h, 0.0);
        let k = engine.h.average_coordination();
        let l = engine.h.max_chain_length();

        let dk = k - last_k;
        let d_l = l as isize - last_l as isize;

        let omega = hierarchical_closure(&engine.h, &inter);
        let domega = omega - last_omega;
        last_omega = omega;

        let total_attempts = accepted + rejected;
        let acc_ratio = if total_attempts > 0 {
            accepted as f64 / total_attempts as f64
        } else {
            0.0
        };

        // Output stable valid structures that resist diffusion and persist over time
        let valid_knots = engine.active_knots.values().filter(|k| k.age >= 50 && k.radius < 5.0).count();
        let total_knots = engine.active_knots.len();

        // Compute density metrics
        let mut densities = Vec::new();
        let mut sum_k = 0;
        for nbrs in inter.values() { sum_k += nbrs.len(); }
        let avg_k = if inter.is_empty() { 1.0 } else { (sum_k as f64) / (inter.len() as f64) }.max(1.0);
        
        for &v in engine.h.vertices.keys() {
            let c = hcsn_rust::observables::local_clustering(&inter, v);
            let d = inter.get(&v).map(|n| n.len()).unwrap_or(0) as f64;
            densities.push(c * (d / avg_k));
        }
        densities.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mean_d = if densities.is_empty() { 0.0 } else { densities.iter().sum::<f64>() / densities.len() as f64 };
        
        let total_stability: f64 = engine.stability.values().sum();
        let flux = total_stability - prev_total_stability;
        prev_total_stability = total_stability;
        
        // Compute max local coherence (internal_edges / boundary_edges for each vertex's 1-hop neighborhood)
        let mut max_coh: f64 = 0.0;
        for &v in engine.h.vertices.keys() {
            if let Some(neighbors) = inter.get(&v) {
                let mut neighborhood = neighbors.clone();
                neighborhood.insert(v);
                let mut ie: u32 = 0;
                let mut be: u32 = 0;
                for &n in &neighborhood {
                    if let Some(n_nbrs) = inter.get(&n) {
                        for &nn in n_nbrs {
                            if neighborhood.contains(&nn) { ie += 1; } else { be += 1; }
                        }
                    }
                }
                ie /= 2;
                let coh = if be > 0 { ie as f64 / be as f64 } else if ie > 0 { 10.0 } else { 0.0 };
                if coh > max_coh { max_coh = coh; }
            }
        }
        
        let supp_ratio = if engine.attempted_rewrites > 0 {
            engine.suppressed_rewrites as f64 / engine.attempted_rewrites as f64
        } else { 0.0 };
        engine.attempted_rewrites = 0;
        engine.suppressed_rewrites = 0;

        let step_ms = last_print_time.elapsed().as_millis();
        last_print_time = Instant::now();

            println!(
                "{:6} | {:5} | {:5.2} | {:+5.2} | {:3} | {:+3} | {:7.4} | {:7.4} | {:5.2}%   | {:7.4} | {:+7.4} | {:5} | {:4} | {:7.3} | {:5.1}% | {:7.0} | {:+8.1}",
                engine.time,
                engine.h.vertices.len(),
                k,
                dk,
                l,
                d_l,
                interaction_concentration(&inter),
                closure_density(&inter),
                acc_ratio * 100.0,
                omega,
                domega,
                valid_knots,
                total_knots,
                max_coh,
                supp_ratio * 100.0,
                step_ms,
                flux
            );

        last_k = k;
        last_l = l;
        last_omega = omega;
    }

    let end_time = start_time.elapsed();

    println!("\nRun complete.");
    println!("Total steps: {}", engine.time);
    println!("Accepted: {}, Rejected: {}", accepted, rejected);
    let total = (accepted + rejected).max(1);
    println!("Acceptance ratio: {:.3}", accepted as f64 / total as f64);
    println!("Wall time: {:.2} s", end_time.as_secs_f64());
    
    // Export lifetime distribution with worldline data
    let tau_c = 1000; // Updated for Phase 7
    let mut lifetimes = Vec::new();
    for k in &engine.dead_knots {
        let is_particle = k.age >= tau_c && k.coherence > 1.0;
        let mean_stab: f64 = if k.vertices.is_empty() { 0.0 } else {
            k.vertices.iter()
                .map(|v| engine.stability.get(v).copied().unwrap_or(0.0))
                .sum::<f64>() / k.vertices.len() as f64
        };
        lifetimes.push(serde_json::json!({
            "id": k.id, "status": "dead", "age": k.age, 
            "max_size": k.max_size, "radius": k.radius,
            "coherence": k.coherence,
            "velocity": k.velocity,
            "velocity_avg": k.velocity_avg,
            "mass": k.mass,
            "momentum": k.momentum,
            "worldline_length": k.position_history.len(),
            "particle_candidate": is_particle,
            "mean_stability": mean_stab
        }));
    }
    for k in engine.active_knots.values() {
        if k.age >= 50 {
            let is_particle = k.age >= tau_c && k.coherence > 1.0;
            let mean_stab: f64 = if k.vertices.is_empty() { 0.0 } else {
                k.vertices.iter()
                    .map(|v| engine.stability.get(v).copied().unwrap_or(0.0))
                    .sum::<f64>() / k.vertices.len() as f64
            };
            lifetimes.push(serde_json::json!({
                "id": k.id, "status": "alive", "age": k.age,
                "max_size": k.max_size, "radius": k.radius,
                "coherence": k.coherence,
                "velocity": k.velocity,
                "velocity_avg": k.velocity_avg,
                "mass": k.mass,
                "momentum": k.momentum,
                "worldline_length": k.position_history.len(),
                "particle_candidate": is_particle,
                "mean_stability": mean_stab
            }));
        }
    }
    
    let out_file = format!("exports/particle_lifetimes_p{:.2}.json", config.p_create);
    std::fs::create_dir_all("exports").unwrap();
    std::fs::write(&out_file, serde_json::to_string_pretty(&lifetimes).unwrap()).unwrap();
    
    // Export interaction events
    let out_ev = format!("exports/interaction_events_p{:.2}.json", config.p_create);
    std::fs::write(&out_ev, serde_json::to_string_pretty(&engine.interaction_events).unwrap()).unwrap();
    println!("Exported {} raw interaction events to {}", engine.interaction_events.len(), out_ev);
    
    let particle_count = lifetimes.iter().filter(|p| p["particle_candidate"] == true).count();
    println!("Exported {} proto-particles ({} particle candidates with τ≥{} and coh>1.0) to {}", 
        lifetimes.len(), particle_count, tau_c, out_file);
    
    println!("\n======================================================================================");
    println!("RUN COMPLETE");
}
