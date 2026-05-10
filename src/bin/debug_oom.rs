use hcsn_rust::hypergraph::Hypergraph;
use hcsn_rust::rewrite_engine::RewriteEngine;

fn main() {
    println!("DEBUG: Starting OOM investigation...");
    let mut h = Hypergraph::new();
    let v1 = h.add_vertex();
    let v2 = h.add_vertex();
    h.add_causal_relation(v1.id, v2.id);
    h.add_hyperedge(vec![v1.id, v2.id]);

    let mut engine = RewriteEngine::new(h, 0.1, Some(1));
    
    for i in 1..=1000 {
        engine.step();
        if i % 10 == 0 {
            println!("Step {}: V={}, E={}, Knots={}, History={}, Interactions={}", 
                i, 
                engine.h.vertices.len(), 
                engine.h.hyperedges.len(),
                engine.active_knots.len(),
                engine.rewrite_history.len(),
                engine.active_interactions.len()
            );
        }
    }
    println!("DEBUG: Finished 1000 steps without OOM.");
}
