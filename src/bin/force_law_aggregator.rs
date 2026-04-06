use hcsn_rust::hypergraph::Hypergraph;
use hcsn_rust::rewrite_engine::{RewriteEngine, EmergenceMode, ConservationMode};
use hcsn_rust::persistence::Persistence;
use rayon::prelude::*;
use std::io::{Write};
use std::sync::{Arc, Mutex};
use std::env;

fn main() {
    let num_threads: usize = 2; // Upgraded to Dual-Core
    println!("=== HCSN PARALLEL INTERACTION AGGREGATOR (v5.9.1) ===");
    
    let p_create = env::var("HCSN_P_CREATE")
        .unwrap_or_else(|_| "0.58".to_string())
        .parse()
        .unwrap_or(0.58);
        
    let steps_per_thread = env::var("HCSN_STEPS")
        .unwrap_or_else(|_| "125000".to_string())
        .parse()
        .unwrap_or(125000);

    let emergence_mode_str = env::var("HCSN_EMERGENCE_MODE")
        .unwrap_or_else(|_| "Assisted".to_string());
    
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

    (0..num_threads).into_par_iter().for_each(|tid| {
        let mut h = Hypergraph::new();
        // Seed vacuum defects for aggregator (Pre-interaction state)
        for _ in 0..16 {
            let mut knot_nodes = Vec::new();
            for _ in 0..4 { knot_nodes.push(h.add_vertex().id); }
            for n1 in 0..4 {
                for n2 in n1+1..4 {
                    h.add_causal_relation(knot_nodes[n1], knot_nodes[n2]);
                    h.add_hyperedge(vec![knot_nodes[n1], knot_nodes[n2]]);
                }
            }
        }

        let mut engine = RewriteEngine::new(h, p_create, None);
        engine.pure_mode = false;
        engine.conservation_mode = ConservationMode::Hybrid;
        engine.mode = emergence_mode;
        engine.thread_id = Some(tid);
        engine.max_steps = steps_per_thread;
        engine.verbose = false;

        println!("  Thread {}: Initiating {} step parallel simulation...", tid, steps_per_thread);
        for s in 1..=steps_per_thread {
            engine.step();
            
            // Periodic Stream Flush (SSD Persistence @ 2,000 steps)
            if s % 2000 == 0 || s == steps_per_thread {
                let mut writer_lock = shared_writer.lock().unwrap();
                let events = std::mem::take(&mut engine.interaction_events);
                for event in events {
                    if let Some(csv_row) = Persistence::format_event(&event) {
                        writeln!(writer_lock, "{}", csv_row).unwrap();
                    }
                }
                if s % 10000 == 0 {
                    writer_lock.flush().unwrap();
                }
            }
        }
    });

    println!("\n=== Conservation Aggregation Complete ===");
    println!("High-rigor dataset streamed to {}", out_file);
}
