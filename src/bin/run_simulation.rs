use hcsn_rust::hypergraph::Hypergraph;
use hcsn_rust::observables::{
    compute_omega,
};
use hcsn_rust::rewrite_engine::RewriteEngine;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use hcsn_rust::rewrite_engine::EngineConfig;

#[derive(Serialize)]
struct Config {
    seed: u64,
    max_steps: usize,
    sample_interval: usize,
    aggressive_mode: bool,
    pure_mode: bool,
    baseline_mode: bool,
    log_file: Option<String>,
    #[serde(skip)]
    engine: EngineConfig,
}

fn get_rss_mb() -> f64 {
    use std::fs;
    let statm = fs::read_to_string("/proc/self/statm").unwrap_or_default();
    let rss_pages = statm.split_whitespace().nth(1).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
    let page_size = 4096; // Standard on most Linux
    (rss_pages * page_size) as f64 / 1024.0 / 1024.0
}

fn save_data(engine: &RewriteEngine, config: &Config) {
    println!("\nExporting final datasets...");

    // Export lifetime distribution with worldline data
    let tau_c = 1000;
    let mut lifetimes = Vec::new();

    for k in &engine.dead_knots {
        if k.age < 50 {
            continue;
        }
        let is_particle = k.age >= tau_c && k.coherence > 1.0;
        let mean_stab: f64 = if k.vertices.is_empty() {
            0.0
        } else {
            k.vertices
                .iter()
                .map(|v| engine.stability.get(v).copied().unwrap_or(0.0))
                .sum::<f64>()
                / k.vertices.len() as f64
        };
        lifetimes.push(serde_json::json!({
            "id": k.id, "status": "dead", "age": k.age,
            "max_size": k.max_size, "radius": k.radius,
            "coherence": k.coherence,
            "diagnostic_v_abs": k.diagnostic_v_abs,
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
            let mean_stab: f64 = if k.vertices.is_empty() {
                0.0
            } else {
                k.vertices
                    .iter()
                    .map(|v| engine.stability.get(v).copied().unwrap_or(0.0))
                    .sum::<f64>()
                    / k.vertices.len() as f64
            };
            lifetimes.push(serde_json::json!({
                "id": k.id, "status": "alive", "age": k.age,
                "max_size": k.max_size, "radius": k.radius,
                "coherence": k.coherence,
                "diagnostic_v_abs": k.diagnostic_v_abs,
                "mass": k.mass,
                "momentum": k.momentum,
                "worldline_length": k.position_history.len(),
                "particle_candidate": is_particle,
                "mean_stability": mean_stab
            }));
        }
    }

    let p_val = if engine.params.enable_conservation_patches { 1 } else { 0 };
    let a_val = if config.aggressive_mode { 1 } else { 0 };
    let suffix = format!("g{}_mu{}_P{}_A{}_s{}", 
        engine.params.nonlinear_coupling, 
        engine.params.memory_coupling, 
        p_val, 
        a_val, 
        config.seed
    );

    let out_file = format!("exports/particle_lifetimes_{}.json", suffix);
    std::fs::create_dir_all("exports").unwrap();
    std::fs::write(&out_file, serde_json::to_string_pretty(&lifetimes).unwrap()).unwrap();

    let particle_count = lifetimes
        .iter()
        .filter(|p| p["particle_candidate"] == true)
        .count();
    println!(
        "Exported {} worldlines ({} candidates >= {}) to {}",
        lifetimes.len(),
        particle_count,
        tau_c,
        out_file
    );
    if engine.params.export_mechanisms {
        let mechanisms_data = engine.export_mechanism_correlation_data();
        let out_mech = format!("exports/hcsn_mechanisms_{}.json", suffix);
        std::fs::write(
            &out_mech,
            serde_json::to_string_pretty(&mechanisms_data).unwrap(),
        )
        .unwrap();
        println!("Exported mechanism correlation data to {}", out_mech);
    }
}

fn main() {
    let mut config = Config {
        seed: 1,
        max_steps: 5000,
        sample_interval: 1000,
        aggressive_mode: false,
        pure_mode: false,
        baseline_mode: false,
        log_file: None,
        engine: EngineConfig::default(),
    };

    let args: Vec<String> = std::env::args().collect();
    let mut arg_idx = 1;
    while arg_idx < args.len() {
        if args[arg_idx] == "--steps" && arg_idx + 1 < args.len() {
            if let Ok(m) = args[arg_idx + 1].parse::<usize>() {
                config.max_steps = m;
            }
            arg_idx += 2;
        } else if args[arg_idx] == "--p_create" && arg_idx + 1 < args.len() {
            if let Ok(p) = args[arg_idx + 1].parse::<f64>() { config.engine.p_create = p; }
            arg_idx += 2;
        } else if args[arg_idx] == "--p_fusion" && arg_idx + 1 < args.len() {
            if let Ok(p) = args[arg_idx + 1].parse::<f64>() { config.engine.p_fusion = Some(p); }
            arg_idx += 2;
        } else if args[arg_idx] == "--gamma_defect" && arg_idx + 1 < args.len() {
            if let Ok(p) = args[arg_idx + 1].parse::<f64>() { config.engine.gamma_defect = Some(p); }
            arg_idx += 2;
        } else if args[arg_idx] == "--inertia_scale" && arg_idx + 1 < args.len() {
            if let Ok(p) = args[arg_idx + 1].parse::<f64>() { config.engine.inertia_scale = Some(p); }
            arg_idx += 2;
        } else if args[arg_idx] == "--interaction_boost" && arg_idx + 1 < args.len() {
            if let Ok(p) = args[arg_idx + 1].parse::<f64>() { config.engine.interaction_boost = Some(p); }
            arg_idx += 2;
        } else if args[arg_idx] == "--nu" && arg_idx + 1 < args.len() {
            if let Ok(p) = args[arg_idx + 1].parse::<f64>() { config.engine.stability_decay = Some(p); }
            arg_idx += 2;
        } else if args[arg_idx] == "--gamma" && arg_idx + 1 < args.len() {
            if let Ok(p) = args[arg_idx + 1].parse::<f64>() { config.engine.nonlinear_coupling = Some(p); }
            arg_idx += 2;
        } else if args[arg_idx] == "--mu" && arg_idx + 1 < args.len() {
            if let Ok(p) = args[arg_idx + 1].parse::<f64>() { config.engine.memory_coupling = Some(p); }
            arg_idx += 2;
        } else if args[arg_idx] == "--defect_injection" && arg_idx + 1 < args.len() {
            if let Ok(p) = args[arg_idx + 1].parse::<f64>() { config.engine.defect_injection = Some(p); }
            arg_idx += 2;
        } else if args[arg_idx] == "--disable_patches" {
            config.engine.disable_patches = true;
            arg_idx += 1;
        } else if args[arg_idx] == "--seed" && arg_idx + 1 < args.len() {
            if let Ok(s) = args[arg_idx + 1].parse::<u64>() {
                config.seed = s;
                config.engine.seed = Some(s);
            }
            arg_idx += 2;
        } else if args[arg_idx] == "--log-to" && arg_idx + 1 < args.len() {
            config.log_file = Some(args[arg_idx + 1].clone());
            arg_idx += 2;
        } else if args[arg_idx] == "--aggressive_mode" {
            config.aggressive_mode = true;
            arg_idx += 1;
        } else if args[arg_idx] == "--track_interval" && arg_idx + 1 < args.len() {
            if let Ok(p) = args[arg_idx + 1].parse::<usize>() { config.engine.track_interval = Some(p); }
            arg_idx += 2;
        } else if args[arg_idx] == "--pure" {
            config.pure_mode = true;
            arg_idx += 1;
        } else if args[arg_idx] == "--baseline" {
            config.baseline_mode = true;
            arg_idx += 1;
        } else {
            arg_idx += 1;
        }
    }

    // Auto-discovery of Gantry logs directory
    if config.log_file.is_none() {
        let gantry_logs = std::path::PathBuf::from("/home/saif/antigravity/gantry/logs");
        if gantry_logs.exists() {
            let auto_path = gantry_logs.join(format!(
                "auto_sim_{}_{}.jsonl",
                std::process::id(),
                config.seed
            ));
            config.log_file = Some(auto_path.to_str().unwrap().to_string());
            println!(
                "[AUTO-LOG] Gantry directory detected. Tracking enabled: {}",
                config.log_file.as_ref().unwrap()
            );
        }
    }

    println!(
        "\n======================================================================================"
    );
    println!(
        "RUN STARTED (Rust Engine) | Seed: {} | Steps: {}",
        config.seed, config.max_steps
    );
    println!(
        "======================================================================================"
    );

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        println!("\n[SIGNAL] Interrupt detected! Finishing current step and saving data...");
    })
    .expect("Error setting Ctrl-C handler");

    let mut h = Hypergraph::new();
    let v1 = h.add_vertex().id;
    let v2 = h.add_vertex().id;
    let v3 = h.add_vertex().id;
    let v4 = h.add_vertex().id;
    h.add_hyperedge(vec![v1, v2, v3, v4]);
    h.add_causal_relation(v1, v2);
    h.add_causal_relation(v1, v3);
    h.add_causal_relation(v2, v4);

    println!("DEBUG: Initializing engine...");
    let mut engine = RewriteEngine::with_config(h, config.engine.clone());
    println!("DEBUG: Engine initialized. Starting loop...");
    
    if config.aggressive_mode {
        engine.mode = hcsn_rust::rewrite_engine::EmergenceMode::Control;
    }
    if config.pure_mode {
        engine.pure_mode = true;
    }
    engine.verbose = false;

    let mut accepted = 0;
    let mut rejected = 0;
    let mut last_k = engine.h.average_coordination();
    let mut last_l = engine.h.max_chain_length();
    let mut last_omega = 0.0;
    // let prev_total_stability = 0.0;
    let start_time = Instant::now();
    let mut last_print_time = Instant::now();

    println!(" time  |   V   |  <k>  | Δ<k> |  L  | ΔL | acc%   | omega | knots | all_k | max_coh | step_ms | RSS_MB");

    // Setup incremental persistence
    let interaction_filename = hcsn_rust::persistence::Persistence::generate_filename("sim_events");
    let mut event_writer = hcsn_rust::persistence::Persistence::open_writer(&interaction_filename);
    let _ = hcsn_rust::persistence::Persistence::write_header(&mut event_writer);

    for _ in 1..=config.max_steps {
        if !running.load(Ordering::SeqCst) {
            break;
        }

        let success = engine.step();
        if success {
            accepted += 1;
        } else {
            rejected += 1;
        }

        // Consolidated output and maintenance
        if engine.time % 100 == 0 {
            hcsn_rust::persistence::Persistence::flush_events(&mut event_writer, &mut engine.interaction_events, 0);
            engine.interaction_events.shrink_to_fit();
            
            let is_sample_step = engine.time % config.sample_interval == 0;
            let mut log_entry: Option<serde_json::Value> = None;

            if is_sample_step {
                let k = engine.h.average_coordination();
                let l = engine.h.max_chain_length();
                let dk = k - last_k;
                let d_l = l as isize - last_l as isize;
                
                let omega = if config.baseline_mode {
                    0.0
                } else {
                    compute_omega(&engine.h)
                };
                
                let acc_rate = (accepted as f64 / (accepted + rejected) as f64) * 100.0;
                
                let max_coh = if engine.active_knots.is_empty() {
                    0.0
                } else {
                    engine.active_knots.values()
                        .map(|k| k.coherence)
                        .fold(0.0, f64::max)
                };
                
                let step_ms = last_print_time.elapsed().as_millis() as f64 / config.sample_interval as f64;
                last_print_time = Instant::now();

                println!(
                    "{:5} | {:5}/{:5} | {:5.2} | {:+5.2} | {:3} | {:+3} | {:5.1}% | {:5.3} | {:5} | {:5} | {:7.3} | {:7.2} | {:7.1} (max_id: {})",
                    engine.time, engine.h.vertices.len(), engine.h.hyperedges.len(), k, dk, l, d_l, acc_rate, omega, engine.active_knots.len(), engine.rewrite_history.len(), max_coh, step_ms, get_rss_mb(), engine.h.max_vertex_id()
                );

                if config.log_file.is_some() {
                    log_entry = Some(serde_json::json!({
                        "time": engine.time,
                        "V": engine.h.vertices.len(),
                        "k": k,
                        "L": l,
                        "acc%": acc_rate,
                        "omega": omega,
                        "knots": engine.active_knots.len(),
                        "max_coh": max_coh,
                        "step_ms": step_ms,
                        "rss_mb": get_rss_mb()
                    }));
                }

                last_k = k;
                last_l = l;
                last_omega = omega;
            } else {
                // Heartbeat (100 steps)
                let acc_rate = (accepted as f64 / (accepted + rejected) as f64) * 100.0;
                println!(
                    "{:5} | {:5}/{:5} | {:5.2} | {:5.2} | {:3} | {:+3} | {:5.1}% | {:5} | {:5} | {:5} | {:7.3} | {:7.2} | {:7.1} (max_id: {})",
                    engine.time, engine.h.vertices.len(), engine.h.hyperedges.len(), engine.h.average_coordination(), 0.0, 0, 0, acc_rate, "...", engine.active_knots.len(), 0, 0.0, 0.0, get_rss_mb(), engine.h.max_vertex_id()
                );
            }

            if let (Some(ref path), Some(entry)) = (&config.log_file, log_entry) {
                if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                    let _ = writeln!(file, "{}", entry.to_string());
                }
            }
        }
    }

    // Final flush of remaining events
    hcsn_rust::persistence::Persistence::flush_events(&mut event_writer, &mut engine.interaction_events, 0);

    let end_time = start_time.elapsed();
    println!(
        "\nSimulation loop ended. Wall time: {:.2} s",
        end_time.as_secs_f64()
    );

    save_data(&engine, &config);
    if engine.params.export_mechanisms {
        let mechanisms_data = engine.export_mechanism_correlation_data();
        let p_val = if engine.params.enable_conservation_patches { 1 } else { 0 };
        let a_val = if config.aggressive_mode { 1 } else { 0 };
        let suffix = format!("g{}_mu{}_P{}_A{}_s{}", 
            engine.params.nonlinear_coupling, 
            engine.params.memory_coupling, 
            p_val, 
            a_val, 
            config.seed
        );
        let out_mech = format!("exports/hcsn_mechanisms_{}.json", suffix);
        std::fs::write(
            &out_mech,
            serde_json::to_string_pretty(&mechanisms_data).unwrap(),
        )
        .unwrap();
        println!("Exported mechanism correlation data to {}", out_mech);
    }

    println!(
        "\n======================================================================================"
    );
    println!("FINISHED");
}
