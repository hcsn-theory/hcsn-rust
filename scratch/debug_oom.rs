use hcsn_rust::hypergraph::Hypergraph;
use hcsn_rust::rewrite_engine::RewriteEngine;
use std::mem::size_of_val;

fn main() {
    let mut h = Hypergraph::new();
    let v1 = h.add_vertex().id;
    let v2 = h.add_vertex().id;
    h.add_hyperedge(vec![v1, v2]);
    h.add_causal_relation(v1, v2);

    let mut engine = RewriteEngine::new(h, 0.65, Some(1));
    engine.pure_mode = true;

    for step in 1..=1000 {
        engine.step();
        
        if step % 100 == 0 {
            println!("--- Step {} ---", step);
            println!("Vertices: {}", engine.h.vertices.len());
            println!("Edges: {}", engine.h.hyperedges.len());
            println!("Active Edges: {}", engine.h.active_edge_ids.len());
            println!("Causal Future Map: {}", engine.h.causal_future.len());
            println!("Causal Past Map: {}", engine.h.causal_past.len());
            println!("Interaction Events Vec: {}", engine.interaction_events.len());
            println!("Active Interactions Map: {}", engine.active_interactions.len());
            println!("Stability Map: {}", engine.stability.len());
            println!("Xi Map: {}", engine.xi.len());
            
            // Inspect a random bitset size
            if let Some(bs) = engine.h.causal_future.values().next() {
                println!("Sample Bitset Capacity (bits): {}", bs.len());
            }

            // Estimate total memory of a bitset map
            let bitset_mem: usize = engine.h.causal_future.values().map(|bs| bs.as_slice().len() * 8).sum();
            println!("Total Causal Future Bitset Raw Mem: {} bytes", bitset_mem);
        }
    }
}
