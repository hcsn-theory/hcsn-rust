use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use rand::Rng;

static VERTEX_ID_COUNTER: AtomicU64 = AtomicU64::new(0);
static EDGE_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub struct Vertex {
    pub id: u64,
    pub depth: usize, // Worldline (causal) depth
    pub label: i32,   // topological/charge-like label
}

impl Vertex {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let label = if rng.gen_bool(0.5) { 1 } else { -1 };
        Self {
            id: VERTEX_ID_COUNTER.fetch_add(1, Ordering::SeqCst),
            depth: 1,
            label,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Hyperedge {
    pub id: u64,
    pub vertices: Vec<u64>,
}

impl Hyperedge {
    pub fn new(vertices: Vec<u64>) -> Self {
        Self {
            id: EDGE_ID_COUNTER.fetch_add(1, Ordering::SeqCst),
            vertices,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Hypergraph {
    pub vertices: HashMap<u64, Vertex>,
    pub hyperedges: HashMap<u64, Hyperedge>,
    pub causal_order: HashMap<u64, HashSet<u64>>, // u.id -> set of v.id
}

impl Hypergraph {
    pub fn new() -> Self {
        Self {
            vertices: HashMap::new(),
            hyperedges: HashMap::new(),
            causal_order: HashMap::new(),
        }
    }

    // ---------- Vertex operations ----------

    pub fn add_vertex(&mut self) -> Vertex {
        let v = Vertex::new();
        self.causal_order.entry(v.id).or_default().insert(v.id); // reflexivity
        self.vertices.insert(v.id, v.clone());
        v
    }

    // ---------- Hyperedge operations ----------

    pub fn add_hyperedge(&mut self, vertices: Vec<u64>) -> Hyperedge {
        let edge = Hyperedge::new(vertices);
        self.hyperedges.insert(edge.id, edge.clone());
        edge
    }

    // ---------- Causal structure ----------

    pub fn add_causal_relation(&mut self, u_id: u64, v_id: u64) {
        // Add causal relation u → v and update worldline depth.
        self.causal_order.entry(u_id).or_default().insert(v_id);

        let u_depth = self.vertices.get(&u_id).map(|u| u.depth).unwrap_or(1);
        
        if let Some(v) = self.vertices.get_mut(&v_id) {
            v.depth = v.depth.max(u_depth + 1);
        }
    }

    pub fn is_causally_related(&self, u_id: u64, v_id: u64) -> bool {
        self.causal_order.get(&u_id).map_or(false, |set| set.contains(&v_id))
    }

    pub fn causal_future(&self, v_id: u64) -> HashSet<u64> {
        self.causal_order.get(&v_id).cloned().unwrap_or_default()
    }

    pub fn causal_past(&self, v_id: u64) -> HashSet<u64> {
        self.causal_order
            .iter()
            .filter_map(|(u_id, future)| {
                if future.contains(&v_id) {
                    Some(*u_id)
                } else {
                    None
                }
            })
            .collect()
    }

    // ---------- Observables ----------

    pub fn coordination_number(&self, v_id: u64) -> usize {
        // Degree: number of hyperedges containing v.
        self.hyperedges
            .values()
            .filter(|e| e.vertices.contains(&v_id))
            .count()
    }

    pub fn average_coordination(&self) -> f64 {
        if self.vertices.is_empty() {
            return 0.0;
        }
        let total_coordination: usize = self
            .vertices
            .keys()
            .map(|&v_id| self.coordination_number(v_id))
            .sum();
        total_coordination as f64 / self.vertices.len() as f64
    }

    // ---------- Worldline inertia ----------

    pub fn max_chain_length(&self) -> usize {
        // Maximum causal chain length in the hypergraph.
        self.vertices.values().map(|v| v.depth).max().unwrap_or(0)
    }
}
