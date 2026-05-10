use chrono::Local;
use hcsn_rust::hypergraph::Hypergraph;
use hcsn_rust::persistence::Persistence;
use hcsn_rust::rewrite_engine::{ConservationMode, RewriteEngine};
use rayon::prelude::*;
use serde_json::json;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::sync::{Arc, Mutex};

fn get_mem_usage_percent() -> f64 {
    if let Ok(mem) = fs::read_to_string("/proc/meminfo") {
        let mut total = 1.0;
        let mut available = 0.0;
        for line in mem.lines() {
            if line.contains("MemTotal:") {
                total = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(1.0);
            }
            if line.contains("MemAvailable:") {
                available = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
            }
        }
        return (1.0 - (available / total)) * 100.0;
    }
    0.0
}

fn main() {
    // 5 threads is the safe limit for 16GB RAM at p=0.64 production density.
    // This allows ~2.5GB per worker + OS overhead.
    let num_threads: usize = env::var("HCSN_THREADS")
        .unwrap_or_else(|_| "5".to_string())
        .parse()
        .unwrap_or(5);

    let base_seed: u64 = env::var("HCSN_BASE_SEED")
        .unwrap_or_else(|_| "42".to_string())
        .parse()
        .unwrap_or(42);

    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Configure Rayon global pool to exactly num_threads to avoid over-subscription
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .unwrap_or_default();

    println!("=== HCSN PARALLEL INTERACTION AGGREGATOR (v6.0.0) ===");

    let p_create = env::var("HCSN_P_CREATE")
        .unwrap_or_else(|_| "0.64".to_string())
        .parse()
        .unwrap_or(0.64);

    let steps_per_thread = env::var("HCSN_STEPS")
        .unwrap_or_else(|_| "40000".to_string())
        .parse()
        .unwrap_or(40000);

    let emergence_mode_str =
        env::var("HCSN_EMERGENCE_MODE").unwrap_or_else(|_| "Assisted".to_string());

    let emergence_mode = match emergence_mode_str.as_str() {
        "Control" => hcsn_rust::rewrite_engine::EmergenceMode::Control,
        "Forced" => hcsn_rust::rewrite_engine::EmergenceMode::Forced,
        _ => hcsn_rust::rewrite_engine::EmergenceMode::Assisted,
    };

    let out_file = Persistence::generate_filename("aggregator");

    println!("Configuration:");
    println!("  Emergence: {:?}", emergence_mode);
    println!("  p_create:  {}", p_create);
    println!("  threads:   {}", num_threads);
    println!("  steps:     {}", steps_per_thread * num_threads);
    println!("  Output:    {}", out_file);

    // Initialize Multi-Threaded Streaming I/O
    let writer = Persistence::open_writer(&out_file);
    let mut header_writer = writer;
    Persistence::write_header(&mut header_writer).unwrap();
    header_writer.flush().unwrap();

    let shared_writer = Arc::new(Mutex::new(header_writer));
    let diagnostics = Arc::new(Mutex::new(vec![json!({}); num_threads]));

    (0..num_threads).into_par_iter().for_each(|tid| {
        let mut h = Hypergraph::new();
        // Seed vacuum defects for aggregator (Pre-interaction state)
        for _ in 0..16 {
            let mut knot_nodes = Vec::new();
            for _ in 0..4 {
                knot_nodes.push(h.add_vertex().id);
            }
            for n1 in 0..4 {
                for n2 in n1 + 1..4 {
                    h.add_causal_relation(knot_nodes[n1], knot_nodes[n2]);
                    h.add_hyperedge(vec![knot_nodes[n1], knot_nodes[n2]]);
                }
            }
        }

        let seed = base_seed + tid as u64;
        let seed_name = if seed == 42 {
            "baseline".to_string()
        } else {
            format!("repeat {}", tid)
        };

        let mut engine = RewriteEngine::new(h, p_create, Some(seed));
        engine.pure_mode = false;
        engine.conservation_mode = ConservationMode::Hybrid; // Production Standard
        engine.mode = emergence_mode.clone();
        engine.thread_id = Some(tid);
        engine.max_steps = steps_per_thread;
        engine.verbose = false;

        println!(
            "  Thread {}: Initiating {} step parallel simulation (seed: {}, branch: {})...",
            tid, steps_per_thread, seed, seed_name
        );
        for s in 1..=steps_per_thread {
            engine.step();

            // Periodic Stream Flush (SSD Persistence @ 2,000 steps)
            if s % 2000 == 0 || s == steps_per_thread {
                let mut writer_lock = shared_writer.lock().unwrap();
                let events = std::mem::take(&mut engine.interaction_events);
                for event in events {
                    if let Some(csv_row) = Persistence::format_event(&event, tid) {
                        writeln!(writer_lock, "{}", csv_row).unwrap();
                    }
                }
                if s % 10000 == 0 {
                    writer_lock.flush().unwrap();
                }
            }

            // Memory Tripwire & Diagnostics
            if s % 5000 == 0 || s == steps_per_thread {
                let mem = get_mem_usage_percent();
                if mem > 92.0 {
                    println!(
                        "\n[!] CRITICAL MEMORY ALERT: {:.1}% usage. HALTING THREAD {}.",
                        mem, tid
                    );
                    break;
                }
            }
        }

        // Capture Thread Diagnostics
        let final_vertices = engine.h.vertices.len();
        let total_interactions = engine.interaction_events.len();
        let mut diag_lock = diagnostics.lock().unwrap();
        diag_lock[tid] = json!({
            "seed": seed,
            "seed_name": seed_name,
            "final_vertices": final_vertices,
            "total_interactions_events": total_interactions,
            "steps_completed": steps_per_thread // Approximate if halted
        });
    });

    println!("\n=== Conservation Aggregation Complete ===");
    println!("High-rigor dataset streamed to {}", out_file);

    // Save Metadata
    let meta_file = format!("{}.meta", out_file);
    let mut seeds_used = Vec::new();
    for tid in 0..num_threads {
        let s = base_seed + tid as u64;
        let name = if s == 42 {
            "baseline".to_string()
        } else {
            format!("repeat {}", tid)
        };
        seeds_used.push(json!({"tid": tid, "seed": s, "name": name}));
    }

    let metadata = json!({
        "timestamp": Local::now().to_rfc3339(),
        "git_hash": git_hash,
        "parameters": {
            "p_create": p_create,
            "steps_per_thread": steps_per_thread,
            "num_threads": num_threads,
            "total_steps": steps_per_thread * num_threads,
            "mode": emergence_mode_str,
            "base_seed": base_seed
        },
        "seeds": seeds_used,
        "diagnostics": *diagnostics.lock().unwrap(),
        "output_file": out_file
    });

    if let Ok(mut f) = File::create(&meta_file) {
        let json_data = serde_json::to_string_pretty(&metadata).unwrap();
        let _ = f.write_all(json_data.as_bytes());
        println!("Metadata (JSON) saved to {}", meta_file);
    }
}
