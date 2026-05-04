use fixedbitset::FixedBitSet;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};

static VERTEX_ID_COUNTER: AtomicU64 = AtomicU64::new(0);
static EDGE_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub struct Vertex {
    pub id: u64,
    pub depth: usize, // Worldline (causal) depth
    pub label: i32,   // topological/charge-like label
    pub parents: Vec<u64>,
    pub children: Vec<u64>,
}

impl Vertex {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let label = if rng.gen_bool(0.5) { 1 } else { -1 };
        Self {
            id: VERTEX_ID_COUNTER.fetch_add(1, Ordering::SeqCst),
            depth: 1,
            label,
            parents: Vec::new(),
            children: Vec::new(),
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
    pub causal_future: HashMap<u64, FixedBitSet>, // u.id -> J+(u)
    pub causal_past: HashMap<u64, FixedBitSet>,   // u.id -> J-(u)
}

impl Hypergraph {
    pub fn new() -> Self {
        Self {
            vertices: HashMap::new(),
            hyperedges: HashMap::new(),
            causal_future: HashMap::new(),
            causal_past: HashMap::new(),
        }
    }

    fn init_bitset() -> FixedBitSet {
        FixedBitSet::with_capacity(524288)
    }

    fn ensure_capacity(bs: &mut FixedBitSet, min_capacity: u64) {
        if (bs.len() as u64) < min_capacity {
            let new_cap = (min_capacity as usize).max(524288).next_power_of_two();
            bs.grow(new_cap);
        }
    }

    // ---------- Vertex operations ----------

    pub fn add_vertex(&mut self) -> Vertex {
        let v = Vertex::new();

        let mut future = Self::init_bitset();
        Self::ensure_capacity(&mut future, v.id);
        future.insert(v.id as usize);
        self.causal_future.insert(v.id, future);

        let mut past = Self::init_bitset();
        Self::ensure_capacity(&mut past, v.id);
        past.insert(v.id as usize);
        self.causal_past.insert(v.id, past);

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
        if self.is_causally_related(u_id, v_id) {
            return;
        }

        // 1-hop adjacency (Guarded against duplicate entries)
        if let Some(u) = self.vertices.get_mut(&u_id) {
            if !u.children.contains(&v_id) {
                u.children.push(v_id);
            }
        }
        if let Some(v) = self.vertices.get_mut(&v_id) {
            if !v.parents.contains(&u_id) {
                v.parents.push(u_id);
            }
        }

        // Transitive Update: Every p in J-(u) reaches every f in J+(v)
        // Optimization: Use bitwise OR logic
        let past_u = self
            .causal_past
            .get(&u_id)
            .cloned()
            .unwrap_or_else(|| Self::init_bitset());
        let future_v = self
            .causal_future
            .get(&v_id)
            .cloned()
            .unwrap_or_else(|| Self::init_bitset());

        for p_idx in past_u.ones() {
            let p_id = p_idx as u64;
            if let Some(p_future) = self.causal_future.get_mut(&p_id) {
                Self::ensure_capacity(p_future, future_v.len() as u64);
                p_future.union_with(&future_v);
            }
        }

        for f_idx in future_v.ones() {
            let f_id = f_idx as u64;
            if let Some(f_past) = self.causal_past.get_mut(&f_id) {
                Self::ensure_capacity(f_past, past_u.len() as u64);
                f_past.union_with(&past_u);
            }
        }

        let u_depth = self.vertices.get(&u_id).map(|u| u.depth).unwrap_or(1);
        if let Some(v) = self.vertices.get_mut(&v_id) {
            v.depth = v.depth.max(u_depth + 1);
        }
    }

    pub fn merge_causal_identity(&mut self, id_keep: u64, id_remove: u64) {
        if let (Some(f_remove), Some(p_remove)) = (
            self.causal_future.get(&id_remove).cloned(),
            self.causal_past.get(&id_remove).cloned(),
        ) {
            if let Some(f_keep) = self.causal_future.get_mut(&id_keep) {
                Self::ensure_capacity(f_keep, f_remove.len() as u64);
                f_keep.union_with(&f_remove);
            }
            if let Some(p_keep) = self.causal_past.get_mut(&id_keep) {
                Self::ensure_capacity(p_keep, p_remove.len() as u64);
                p_keep.union_with(&p_remove);
            }
        }
    }

    pub fn is_causally_related(&self, u_id: u64, v_id: u64) -> bool {
        self.causal_future.get(&u_id).map_or(false, |bs| {
            if (v_id as usize) < bs.len() {
                bs.contains(v_id as usize)
            } else {
                false
            }
        })
    }

    fn _touched_vertices(&self, v_id: u64) -> HashSet<u64> {
        self.causal_future
            .get(&v_id)
            .map(|bs| bs.ones().map(|i| i as u64).collect())
            .unwrap_or_default()
    }

    pub fn scrub_ghost_bits(&mut self) {
        if self.vertices.is_empty() {
            return;
        }
        let max_id: u64 = *self.vertices.keys().max().unwrap_or(&0);
        let mut mask = FixedBitSet::with_capacity(max_id as usize + 1);
        for &id in self.vertices.keys() {
            mask.insert(id as usize);
        }

        // Physically delete bitsets for dead vertices
        self.causal_future
            .retain(|k, _| self.vertices.contains_key(k));
        self.causal_past
            .retain(|k, _| self.vertices.contains_key(k));

        for bs in self.causal_future.values_mut() {
            bs.intersect_with(&mask);
        }
        for bs in self.causal_past.values_mut() {
            bs.intersect_with(&mask);
        }
    }

    pub fn causal_future(&self, v_id: u64) -> HashSet<u64> {
        self.causal_future
            .get(&v_id)
            .map(|bs| bs.ones().map(|i| i as u64).collect())
            .unwrap_or_default()
    }

    pub fn causal_past(&self, v_id: u64) -> HashSet<u64> {
        self.causal_past
            .get(&v_id)
            .map(|bs| bs.ones().map(|i| i as u64).collect())
            .unwrap_or_default()
    }

    pub fn causal_future_bitset(&self, v_id: u64) -> Option<FixedBitSet> {
        self.causal_future.get(&v_id).cloned()
    }

    pub fn causal_past_bitset(&self, v_id: u64) -> Option<FixedBitSet> {
        self.causal_past.get(&v_id).cloned()
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

    pub fn execute_undo_record(&mut self, record: crate::rules::UndoRecord) {
        // 1. Remove added vertices and their causal entries
        for v_id in &record.added_vertices {
            self.vertices.remove(v_id);
            self.causal_future.remove(v_id);
            self.causal_past.remove(v_id);
        }

        // 2. Remove added edges
        for eid in &record.added_edges {
            self.hyperedges.remove(eid);
        }

        // 3. Restore removed vertex
        if let Some(v) = record.removed_vertex {
            self.vertices.insert(v.id, v);
            // Causal entries for removed vertices are handled by step 5
        }

        // 4. Restore removed edges
        for (eid, e) in record.removed_edges {
            self.hyperedges.insert(eid, e);
        }

        // 5. Restore causal state and adjacency for modified vertices
        for (u_id, future_bs) in record.old_causal_future {
            self.causal_future.insert(u_id, future_bs);
        }
        for (u_id, past_bs) in record.old_causal_past {
            self.causal_past.insert(u_id, past_bs);
        }
        for (u_id, parents) in record.old_parents {
            if let Some(v) = self.vertices.get_mut(&u_id) {
                v.parents = parents;
            }
        }
        for (u_id, children) in record.old_children {
            if let Some(v) = self.vertices.get_mut(&u_id) {
                v.children = children;
            }
        }
    }
}
