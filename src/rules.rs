use std::collections::{HashMap, HashSet};
use rand::Rng;
use rand::seq::SliceRandom;
use crate::hypergraph::{Hypergraph, Hyperedge, Vertex};
use fixedbitset::FixedBitSet;

/// Explict struct to replace Python's dynamic undo dictionary
#[derive(Clone, Debug, Default)]
pub struct UndoRecord {
    pub target: Vec<u64>,
    pub added_vertices: Vec<u64>,
    pub added_edges: Vec<u64>,
    pub added_causal: Vec<(u64, u64)>,
    pub removed_vertex: Option<Vertex>,
    pub kept_vertex: Option<u64>,
    pub removed_edges: HashMap<u64, Hyperedge>,
    pub old_causal_future: HashMap<u64, FixedBitSet>,
    pub old_causal_past: HashMap<u64, FixedBitSet>,
    pub old_parents: HashMap<u64, Vec<u64>>,
    pub old_children: HashMap<u64, Vec<u64>>,
}

pub fn edge_creation_rule(
    h: &mut Hypergraph,
    anchor_vertex_id: Option<u64>,
    p_rule: f64,
) -> Option<UndoRecord> {
    if h.hyperedges.is_empty() {
        return None;
    }

    let mut undo = UndoRecord::default();
    let mut rng = rand::thread_rng();

    // Select target edge
    let edge = if let Some(anchor_id) = anchor_vertex_id {
        let candidates: Vec<&Hyperedge> = h.hyperedges
            .values()
            .filter(|e| e.vertices.contains(&anchor_id))
            .collect();
            
        if candidates.is_empty() {
            return None;
        }
        (*candidates.choose(&mut rng).unwrap()).clone()
    } else {
        let edges: Vec<&Hyperedge> = h.hyperedges.values().collect();
        (*edges.choose(&mut rng).unwrap()).clone()
    };

    // --------------------------------------------------
    // LOOP CLOSURE (CRITICAL FOR GEOMETRY)
    // --------------------------------------------------
    // With small probability, connect existing vertices
    // instead of creating a new one (breaks tree structure)
    if rng.gen::<f64>() < p_rule && h.vertices.len() >= 3 {
        let vertex_ids: Vec<u64> = h.vertices.keys().copied().collect();
        let sample: Vec<&u64> = vertex_ids.choose_multiple(&mut rng, 2).collect();
        let u_id = *sample[0];
        let v_id = *sample[1];

        if !h.is_causally_related(u_id, v_id) {
            h.add_causal_relation(u_id, v_id);
            
            let e = h.add_hyperedge(vec![u_id, v_id]);
            
            undo.added_edges.push(e.id);
            undo.added_causal.push((u_id, v_id));
        }
    }

    // create new vertex
    let new_vertex_id = h.add_vertex().id;
    undo.added_vertices.push(new_vertex_id);

    // connect causally to vertices in the chosen edge
    for &v_id in &edge.vertices {
        h.add_causal_relation(v_id, new_vertex_id);
        undo.added_causal.push((v_id, new_vertex_id));
    }

    // causal thickening
    for &v_id in &edge.vertices {
        let past = h.causal_past(v_id);
        for u_id in past {
            if rng.gen::<f64>() < 0.3 {
                h.add_causal_relation(u_id, new_vertex_id);
                undo.added_causal.push((u_id, new_vertex_id));
            }
        }
    }

    // create new hyperedge
    let mut new_edge_vertices = edge.vertices.clone();
    new_edge_vertices.push(new_vertex_id);
    let e = h.add_hyperedge(new_edge_vertices);
    undo.added_edges.push(e.id);
    undo.target = edge.vertices.clone();

    Some(undo)
}

pub fn vertex_fusion_rule(h: &mut Hypergraph, anchor_vertex_id: Option<u64>) -> Option<UndoRecord> {
    if h.vertices.len() < 3 || h.hyperedges.is_empty() {
        return None;
    }

    let mut rng = rand::thread_rng();
    
    let edge = if let Some(anchor_id) = anchor_vertex_id {
        let candidates: Vec<&Hyperedge> = h.hyperedges
            .values()
            .filter(|e| e.vertices.contains(&anchor_id))
            .collect();
            
        if candidates.is_empty() {
            return None;
        }
        (*candidates.choose(&mut rng).unwrap()).clone()
    } else {
        let edges: Vec<&Hyperedge> = h.hyperedges.values().collect();
        (*edges.choose(&mut rng).unwrap()).clone()
    };

    if edge.vertices.len() < 3 {
        return None;
    }

    let v_keep_id = edge.vertices[0];
    let v_remove_id = edge.vertices[1];

    let has_remaining_edges = h.hyperedges.values()
        .any(|e| !e.vertices.contains(&v_remove_id));
        
    if !has_remaining_edges {
        return None;
    }

    let mut undo = UndoRecord::default();
    undo.target = vec![v_remove_id];
    undo.kept_vertex = Some(v_keep_id);
    let v_remove = h.vertices.get(&v_remove_id).unwrap().clone();
    undo.removed_vertex = Some(v_remove);

    // log causal relations and adjacency
    for u_id in h.vertices.keys().cloned().collect::<Vec<_>>() {
        if h.is_causally_related(u_id, v_remove_id) {
            if let Some(fb) = h.causal_future_bitset(u_id) {
                undo.old_causal_future.insert(u_id, fb);
            }
            if let Some(pb) = h.causal_past_bitset(u_id) {
                undo.old_causal_past.insert(u_id, pb);
            }
            if let Some(v) = h.vertices.get(&u_id) {
                undo.old_parents.insert(u_id, v.parents.clone());
                undo.old_children.insert(u_id, v.children.clone());
            }
        }
    }

    // Since we are removing v_remove_id, we need to manually update 1-hop adjacency of its neighbors
    let parents = h.vertices.get(&v_remove_id).unwrap().parents.clone();
    let children = h.vertices.get(&v_remove_id).unwrap().children.clone();
    
    h.merge_causal_identity(v_keep_id, v_remove_id);

    for p_id in parents {
        if let Some(p) = h.vertices.get_mut(&p_id) {
            p.children.retain(|&id| id != v_remove_id);
            p.children.push(v_keep_id);
        }
        h.add_causal_relation(p_id, v_keep_id);
    }
    for c_id in children {
        if let Some(c) = h.vertices.get_mut(&c_id) {
            c.parents.retain(|&id| id != v_remove_id);
            c.parents.push(v_keep_id);
        }
        h.add_causal_relation(v_keep_id, c_id);
    }

    // remove edges containing v_remove_id
    let mut edges_to_remove = Vec::new();
    for (eid, e) in &h.hyperedges {
        if e.vertices.contains(&v_remove_id) {
            edges_to_remove.push(*eid);
        }
    }
    
    for eid in edges_to_remove {
        if let Some(e) = h.hyperedges.remove(&eid) {
            undo.removed_edges.insert(eid, e);
        }
    }

    // remove vertex
    h.vertices.remove(&v_remove_id);
    h.causal_future.remove(&v_remove_id);
    h.causal_past.remove(&v_remove_id);

    Some(undo)
}
