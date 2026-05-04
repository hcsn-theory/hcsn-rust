use hcsn_rust::hypergraph::Hypergraph;
use hcsn_rust::observables::{average_coordination, myrheim_meyer_dimension};
use hcsn_rust::rewrite_engine::RewriteEngine;

fn run_universe(p_create: f64, steps: usize, seed: u64) -> (usize, f64, Option<f64>) {
    let mut h = Hypergraph::new();

    let v1 = h.add_vertex();
    let v2 = h.add_vertex();
    h.add_causal_relation(v1.id, v2.id);
    h.add_hyperedge(vec![v1.id, v2.id]);

    let mut engine = RewriteEngine::new(h, p_create, Some(seed));

    // Silence verbose logs for critical scan
    engine.verbose = false;

    for _ in 0..steps {
        engine.step();
    }

    let k_avg = average_coordination(&engine.h);
    let dim = myrheim_meyer_dimension(&engine.h, 800, 20);

    (engine.h.vertices.len(), k_avg, dim)
}

fn main() {
    println!("p_create | vertices | <k>    | dimension");
    println!("------------------------------------------");

    let p_values = vec![0.47, 0.48, 0.49, 0.50, 0.51, 0.52, 0.53];

    for p in p_values {
        let (vertices, k_avg, dim) = run_universe(p, 10000, 1);

        let dim_str = match dim {
            Some(d) => format!("{:.2}", d),
            None => "None".to_string(),
        };

        println!("{:7.2} | {:8} | {:6.2} | {}", p, vertices, k_avg, dim_str);
    }
}
