use std::time::Instant;
use serde_json::json;
use std::fs::File;
use std::io::Write;

use hcsn_rust::hypergraph::Hypergraph;
use hcsn_rust::rewrite_engine::RewriteEngine;
use hcsn_rust::observables::{worldline_interaction_graph, compute_omega};

const STABILIZE_STEPS_BEFORE_PROBE: usize = 150;
const INTERACTION_STEPS: usize = 1500;
const OMEGA_TARGET: f64 = 1.10;
const OMEGA_TOL: f64 = 0.05;
const SEED: u64 = 1;

fn main() {
    let mut h = Hypergraph::new();
    let v1 = h.add_vertex();
    let v2 = h.add_vertex();
    h.add_causal_relation(v1.id, v2.id);
    h.add_hyperedge(vec![v1.id, v2.id]);

    let mut engine = RewriteEngine::new(h, 0.6, Some(SEED));
    
    // -------------------------------
    // Reach target Ω
    // -------------------------------
    loop {
        engine.step();
        let inter = worldline_interaction_graph(&engine.h, 0.0);
        let omega = compute_omega(&inter);
        if (omega - OMEGA_TARGET).abs() < OMEGA_TOL {
            break;
        }
    }

    // -------------------------------
    // Inject first proto-particle
    // -------------------------------
    let ok = engine.force_defect(0.3, 30);
    if !ok {
        println!("[warn] First proto-particle injection failed — continuing anyway");
    }
    
    let first_injection_time = engine.time;

    for _ in 0..STABILIZE_STEPS_BEFORE_PROBE {
        engine.step();
    }

    // -------------------------------
    // Safety reseed
    // -------------------------------
    if engine.xi.is_empty() {
        if let Some(&vid) = engine.h.vertices.keys().next() {
            engine.xi.insert(vid, 0.2);
            println!("[inject] re-seeded ξ at v={}", vid);
        }
    }

    // -------------------------------
    // Inject second proto-particle
    // -------------------------------
    let ok = engine.force_second_proto_object(0.3, 1.0, 6, 50);
    if !ok {
        println!("[warn] Second proto-particle injection failed — continuing experiment");
    }

    let second_injection_time = engine.time;

    // -------------------------------
    // Interaction observation
    // -------------------------------
    let mut interaction_log = Vec::new();

    for _ in 0..INTERACTION_STEPS {
        let t0 = Instant::now();
        engine.step();
        
        if engine.time % 200 == 0 {
            println!("[geom-live] topo={} xi={}", engine.topo_distance_memory.len(), engine.xi_distance_memory.len());
        }
        
        let t1 = Instant::now();
        let inter = worldline_interaction_graph(&engine.h, 0.0);
        
        let xi_count = engine.xi.values().filter(|&&x| x > engine.xi_threshold && x.is_finite()).count();
        
        // Count clusters (simplification logic for stats only)
        let topo_pairs = engine.topo_distance_memory.len();
        let xi_pairs = engine.xi_distance_memory.len();
        
        let t2 = Instant::now();
        
        let omega = compute_omega(&inter);

        interaction_log.push(json!({
            "t": engine.time,
            "Ω": (omega * 1000000.0).round() / 1000000.0,
            "xi": {
                "count": xi_count,
            },
            "geometry": {
                "topo_pairs": topo_pairs,
                "xi_pairs": xi_pairs,
            },
            "graph": {
                "vertices": engine.h.vertices.len(),
                "hyperedges": engine.h.hyperedges.len(),
                "interaction_nodes": inter.len(),
            }
        }));

        let t3 = Instant::now();

        if engine.time % 100 == 0 {
            println!(
                "[perf] t={} engine={:.2}ms observer={:.2}ms total={:.2}ms",
                engine.time,
                (t1 - t0).as_secs_f64() * 1000.0,
                (t3 - t2).as_secs_f64() * 1000.0,
                (t3 - t0).as_secs_f64() * 1000.0
            );
        }
    }

    let out = json!({
        "metadata": {
            "seed": SEED,
            "omega_target": OMEGA_TARGET,
            "interaction_steps": INTERACTION_STEPS,
            "first_injection_time": first_injection_time,
            "second_injection_time": second_injection_time,
        },
        "interaction_log": interaction_log,
    });

    // Write to file
    std::fs::create_dir_all("exports").unwrap_or_default();
    let mut file = File::create("exports/interaction_experiment.json").expect("Unable to create file");
    file.write_all(out.to_string().as_bytes()).expect("Unable to write data");

    println!("Saved → exports/interaction_experiment.json");
}
