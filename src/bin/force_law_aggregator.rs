use hcsn_rust::hypergraph::Hypergraph;
use hcsn_rust::rewrite_engine::RewriteEngine;
use rayon::prelude::*;
use std::fs::File;
use std::io::Write;
use std::env;
use std::sync::{Arc, Mutex};

fn mag(a: (f64, f64)) -> f64 { (a.0 * a.0 + a.1 * a.1).sqrt().max(1e-6) }

fn main() {
    env::set_var("RAYON_NUM_THREADS", "2");
    println!("=== HCSN PARALLEL INTERACTION AGGREGATOR (MULTI-DIM) ===");
    println!("Goal: Collect N >= 100 multi-dim samples across 40,000 aggregate steps");

    let p_create = 0.69;
    let nu = 0.975;
    let gamma = 2.0;
    let mu = 0.3;
    let steps_per_thread = 20000;
    let num_threads = 2;

    env::set_var("HCSN_NU", nu.to_string());
    env::set_var("HCSN_GAMMA", gamma.to_string());
    env::set_var("HCSN_MU", mu.to_string());

    let all_points = Arc::new(Mutex::new(Vec::<(f64, f64, f64, f64, f64, f64, f64)>::new()));

    (0..num_threads).into_par_iter().for_each(|i| {
        let mut h = Hypergraph::new();
        let v1 = h.add_vertex().id;
        let v2 = h.add_vertex().id;
        let v3 = h.add_vertex().id;
        let v4 = h.add_vertex().id;
        let nodes = vec![v1, v2, v3, v4];
        for n1 in 0..4 {
            for n2 in n1+1..4 {
                h.add_causal_relation(nodes[n1], nodes[n2]);
                h.add_hyperedge(vec![nodes[n1], nodes[n2]]);
            }
        }

        let mut engine = RewriteEngine::new(h, p_create, None);
        engine.pure_mode = true;
        engine.verbose = false;

        println!("  Thread {}: Starting 20,000 steps...", i);
        for _ in 0..steps_per_thread {
            engine.step();
        }

        let mut points = Vec::new();
        for event in engine.interaction_events.iter().filter(|e| e.duration >= 3) {
            let chi = event.overlap_depth;
            
            // Particle A
            let p_pre_a = event.pre_a.2;
            let coh_a = event.pre_a.4;
            let stab_a = event.pre_a.5;
            let rad_a = event.pre_a.6;
            let size_a = event.pre_a.7;
            let ratio_a = event.pre_a.8;

            if let Some(post_a) = event.post_a {
                let p_post_a = post_a.2;
                if p_pre_a.abs() > 0.01 {
                    let dp_norm = (p_post_a - p_pre_a).abs() / (p_pre_a.abs() + p_post_a.abs() + 1e-6);
                    points.push((chi, dp_norm, coh_a, stab_a, rad_a, size_a as f64, ratio_a));
                }
            }

            // Particle B
            let p_pre_b = event.pre_b.2;
            let coh_b = event.pre_b.4;
            let stab_b = event.pre_b.5;
            let rad_b = event.pre_b.6;
            let size_b = event.pre_b.7;
            let ratio_b = event.pre_b.8;

            if let Some(post_b) = event.post_b {
                let p_post_b = post_b.2;
                if p_pre_b.abs() > 0.01 {
                    let dp_norm = (p_post_b - p_pre_b).abs() / (p_pre_b.abs() + p_post_b.abs() + 1e-6);
                    points.push((chi, dp_norm, coh_b, stab_b, rad_b, size_b as f64, ratio_b));
                }
            }
        }
        all_points.lock().unwrap().extend(points);
        println!("  Thread {}: Done.", i);
    });

    let final_points = all_points.lock().unwrap();
    println!("\n=== Aggregation Complete ===");
    println!("Total Aggregate Steps: {}", steps_per_thread * num_threads);
    println!("Total Samples Collected (N): {}", final_points.len());

    let mut file = File::create("exports/interaction_points_raw.csv").unwrap();
    writeln!(file, "chi,dp_norm,coh,stab,rad,size,ratio").unwrap();
    for (chi, dp, coh, stab, rad, size, ratio) in final_points.iter() {
        writeln!(file, "{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}", 
            chi, dp, coh, stab, rad, size, ratio).unwrap();
    }
    println!("Data exported to exports/interaction_points_raw.csv");
}
