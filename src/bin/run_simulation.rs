use std::time::Instant;
use serde::Serialize;
use hcsn_rust::hypergraph::Hypergraph;
use hcsn_rust::rewrite_engine::RewriteEngine;
use hcsn_rust::observables::{
    worldline_interaction_graph,
    compute_omega,
    compute_coherence_raw,
    detect_candidate_knot_neighborhoods,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Serialize)]
struct Config {
    seed: u64,
    max_steps: usize,
    sample_interval: usize,
    p_create: f64,
    noise_bias: f64,
    defect_injection: f64,
    geometry_freeze: f64,
    log_file: Option<String>,
}

fn save_data(engine: &RewriteEngine, config: &Config) {
    println!("\nExporting final datasets...");
    
    // Export lifetime distribution with worldline data
    let tau_c = 1000; 
    let mut lifetimes = Vec::new();
    
    for k in &engine.dead_knots {
        if k.age < 50 { continue; }
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
    
    let out_file = format!("exports/particle_lifetimes_p{:.2}_s{}.json", config.p_create, config.seed);
    std::fs::create_dir_all("exports").unwrap();
    std::fs::write(&out_file, serde_json::to_string_pretty(&lifetimes).unwrap()).unwrap();
    
    // Export interaction events
    let out_ev = format!("exports/interaction_events_p{:.2}_s{}.json", config.p_create, config.seed);
    std::fs::write(&out_ev, serde_json::to_string_pretty(&engine.interaction_events).unwrap()).unwrap();
    
    let particle_count = lifetimes.iter().filter(|p| p["particle_candidate"] == true).count();
    println!("Exported {} interaction events to {}", engine.interaction_events.len(), out_ev);
    println!("Exported {} worldlines ({} candidates >= {}) to {}", 
        lifetimes.len(), particle_count, tau_c, out_file);
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
        log_file: None,
    };

    let args: Vec<String> = std::env::args().collect();
    let mut arg_idx = 1;
    while arg_idx < args.len() {
        if args[arg_idx] == "--steps" && arg_idx + 1 < args.len() {
            if let Ok(m) = args[arg_idx+1].parse::<usize>() { config.max_steps = m; }
            arg_idx += 2;
        } else if args[arg_idx] == "--p_create" && arg_idx + 1 < args.len() {
            if let Ok(p) = args[arg_idx+1].parse::<f64>() { config.p_create = p; }
            arg_idx += 2;
        } else if args[arg_idx] == "--seed" && arg_idx + 1 < args.len() {
            if let Ok(s) = args[arg_idx+1].parse::<u64>() { config.seed = s; }
            arg_idx += 2;
        } else if args[arg_idx] == "--log-to" && arg_idx + 1 < args.len() {
            config.log_file = Some(args[arg_idx+1].clone());
            arg_idx += 2;
        } else {
            arg_idx += 1;
        }
    }

    // Auto-discovery of Gantry logs directory
    if config.log_file.is_none() {
        let gantry_logs = std::path::PathBuf::from("/home/saif/antigravity/gantry/logs");
        if gantry_logs.exists() {
            let auto_path = gantry_logs.join(format!("auto_sim_{}_{}.jsonl", std::process::id(), config.seed));
            config.log_file = Some(auto_path.to_str().unwrap().to_string());
            println!("[AUTO-LOG] Gantry directory detected. Tracking enabled: {}", config.log_file.as_ref().unwrap());
        }
    }

    println!("\n======================================================================================");
    println!("RUN STARTED (Rust Engine) | Seed: {} | Steps: {}", config.seed, config.max_steps);
    println!("======================================================================================");
    
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        println!("\n[SIGNAL] Interrupt detected! Finishing current step and saving data...");
    }).expect("Error setting Ctrl-C handler");

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
    let last_k = engine.h.average_coordination();
    let last_l = engine.h.max_chain_length();
    let mut last_omega = 0.0;
    // let prev_total_stability = 0.0;
    let start_time = Instant::now();
    let mut last_print_time = Instant::now();

    println!(" time  |   V   |  <k>  | Δ<k> |  L  | ΔL | acc%   | omega | knots | all_k | max_coh | step_ms");

    for _ in 1..=config.max_steps {
        if !running.load(Ordering::SeqCst) {
            break;
        }

        let success = engine.step();
        if success { accepted += 1; } else { rejected += 1; }

        if engine.time % config.sample_interval == 0 {
            let inter = worldline_interaction_graph(&engine.h, 0.0);
            let k = engine.h.average_coordination();
            let l = engine.h.max_chain_length();
            let dk = k - last_k;
            let d_l = l as isize - last_l as isize;

            let omega = compute_omega(&inter);
            let _domega = omega - last_omega;
            
            let total_attempts = accepted + rejected;
            let acc_ratio = if total_attempts > 0 { accepted as f64 / total_attempts as f64 } else { 0.0 };

            let valid_knots = engine.active_knots.values().filter(|k| k.age >= 50 && k.radius < 5.0).count();
            let total_knots = engine.active_knots.len();

            let candidates = detect_candidate_knot_neighborhoods(&engine.h, &inter, 0.0);
            let mut max_coh: f64 = 0.0;
            for cand in candidates {
                let (ie, be) = compute_coherence_raw(&cand, &inter);
                let coh = if be > 0 { ie as f64 / be as f64 } else if ie > 0 { 10.0 } else { 0.0 };
                if coh > max_coh { max_coh = coh; }
            }

            let step_ms = last_print_time.elapsed().as_millis();
            last_print_time = Instant::now();

            println!(
                "{:6} | {:5} | {:5.2} | {:+5.2} | {:3} | {:+3} | {:5.1}% | {:5.3} | {:5} | {:5} | {:7.3} | {:7}",
                engine.time, engine.h.vertices.len(), k, dk, l, d_l, 
                acc_ratio * 100.0, omega, valid_knots, total_knots, max_coh, step_ms
            );

            last_omega = omega;

            if let Some(ref path) = config.log_file {
                if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                    let log_entry = serde_json::json!({
                        "time": engine.time,
                        "vertices": engine.h.vertices.len(),
                        "coordination": k,
                        "chain_length": l,
                        "acc_pct": acc_ratio * 100.0,
                        "omega": omega,
                        "knots": valid_knots,
                        "total_knots": total_knots,
                        "max_coh": max_coh,
                        "step_ms": step_ms,
                        "pid": std::process::id(),
                    });
                    let _ = writeln!(file, "{}", log_entry.to_string());
                }
            }
        }
    }

    let end_time = start_time.elapsed();
    println!("\nSimulation loop ended. Wall time: {:.2} s", end_time.as_secs_f64());
    
    save_data(&engine, &config);
    
    println!("\n======================================================================================");
    println!("FINISHED");
}
