use std::collections::{HashMap, HashSet};
use fixedbitset::FixedBitSet;
use rand::seq::SliceRandom;
use crate::hypergraph::Hypergraph;

pub fn average_coordination(h: &Hypergraph) -> f64 {
    // Compute average coordination number <k>.
    h.average_coordination()
}

pub fn causal_interval_size(h: &Hypergraph, u_id: u64, v_id: u64) -> usize {
    // |I(u, v)| = |J+(u) ∩ J-(v)|
    let future_u = h.causal_future(u_id);
    let past_v = h.causal_past(v_id);
    future_u.intersection(&past_v).count()
}

pub fn myrheim_meyer_dimension(h: &Hypergraph, samples: usize, min_interval: usize) -> Option<f64> {
    /*
    Myrheim–Meyer dimension estimator with interval filtering.
    Only considers sufficiently large causal intervals.
    */
    let vertices: Vec<u64> = h.vertices.keys().cloned().collect();
    if vertices.len() < 2 {
        return None;
    }

    let mut sizes = Vec::new();
    let mut rng = rand::thread_rng();

    for _ in 0..samples {
        let sample: Vec<&u64> = vertices.choose_multiple(&mut rng, 2).collect();
        let u_id = *sample[0];
        let v_id = *sample[1];

        if h.is_causally_related(u_id, v_id) {
            let i = causal_interval_size(h, u_id, v_id);
            if i >= min_interval {
                sizes.push(i);
            }
        }
    }

    if sizes.is_empty() {
        return None;
    }

    let avg_i = sizes.iter().sum::<usize>() as f64 / sizes.len() as f64;
    let n = h.vertices.len() as f64;

    if avg_i <= 1.0 {
        return None;
    }

    let dim = 2.0 * n.ln() / avg_i.ln();
    if dim.is_finite() { Some(dim) } else { None }
}

pub fn average_large_interval(h: &Hypergraph, samples: usize, min_interval: usize) -> f64 {
    /*
    Measure average size of large causal intervals.
    Returns 0 if none exist.
    */
    let vertices: Vec<u64> = h.vertices.keys().cloned().collect();
    if vertices.len() < 2 {
        return 0.0;
    }

    let mut sizes = Vec::new();
    let mut rng = rand::thread_rng();

    for _ in 0..samples {
        let sample: Vec<&u64> = vertices.choose_multiple(&mut rng, 2).collect();
        let u_id = *sample[0];
        let v_id = *sample[1];

        if h.is_causally_related(u_id, v_id) {
            let i = causal_interval_size(h, u_id, v_id);
            if i >= min_interval {
                sizes.push(i);
            }
        }
    }

    if sizes.is_empty() {
        return 0.0;
    }

    sizes.iter().sum::<usize>() as f64 / sizes.len() as f64
}

pub fn adjacency_overlap(h_before: &Hypergraph, h_after: &Hypergraph) -> f64 {
    // Fraction of hyperedges that persist after a rewrite.
    if h_before.hyperedges.is_empty() {
        return 1.0;
    }

    let mut before = HashSet::new();
    for e in h_before.hyperedges.values() {
        let mut sorted_verts = e.vertices.clone();
        sorted_verts.sort_unstable();
        before.insert(sorted_verts);
    }

    let mut after = HashSet::new();
    for e in h_after.hyperedges.values() {
        let mut sorted_verts = e.vertices.clone();
        sorted_verts.sort_unstable();
        after.insert(sorted_verts);
    }

    let intersection_count = before.intersection(&after).count();
    let max_len = before.len().max(1);

    intersection_count as f64 / max_len as f64
}

pub fn interaction_concentration(interactions: &HashMap<u64, FixedBitSet>) -> f64 {
    /*
    Φ = max degree / total degree
    interactions: dict {node_id: set(neighbors)}
    */
    let degrees: Vec<usize> = interactions.values().map(|v| v.ones().count()).collect();
    if degrees.is_empty() {
        return 0.0;
    }
    let sum_degrees: usize = degrees.iter().sum();
    if sum_degrees == 0 {
        return 0.0;
    }
    
    let max_degree = *degrees.iter().max().unwrap();
    max_degree as f64 / sum_degrees as f64
}

pub fn worldline_interaction_graph(h: &Hypergraph, fraction: f64) -> HashMap<u64, FixedBitSet> {
    // Build interaction graph among deep worldlines.
    let max_depth = h.max_chain_length() as f64;
    let cutoff = (fraction * max_depth) as usize;

    let wl_ids: HashSet<u64> = h.vertices.values()
        .filter(|v| v.depth >= cutoff)
        .map(|v| v.id)
        .collect();

    let mut interactions: HashMap<u64, FixedBitSet> = HashMap::new();
    let max_id = h.vertices.keys().max().unwrap_or(&0);
    let cap = (*max_id as usize + 1).max(1024);

    for edge in h.hyperedges.values() {
        let ids: Vec<u64> = edge.vertices.iter()
            .filter(|id| wl_ids.contains(id))
            .cloned()
            .collect();
            
        for &i in &ids {
            for &j in &ids {
                if i != j {
                    interactions.entry(i)
                        .or_insert_with(|| FixedBitSet::with_capacity(cap))
                        .insert(j as usize);
                }
            }
        }
    }

    interactions
}

pub fn count_triangles(interactions: &HashMap<u64, FixedBitSet>) -> usize {
    /*
    Count triangles in an undirected interaction graph.
    interactions: dict {node_id: set(neighbors)}
    */
    let mut triangles = 0;
    
    for (&u, nbrs_u) in interactions {
        for v_idx in nbrs_u.ones() {
            let v = v_idx as u64;
            if v <= u {
                continue;
            }
            if let Some(nbrs_v) = interactions.get(&v) {
                // common neighbors w form triangles u-v-w
                // Use bitset-native intersection count if possible, but manual check is fine for triangles logic
                for w_idx in nbrs_u.ones() {
                    let w = w_idx as u64;
                    if w > v && nbrs_v.contains(w_idx) {
                        triangles += 1;
                    }
                }
            }
        }
    }
    triangles
}

/*
Ω is a consistent structural observable derived from local clustering statistics. 
While not unique, it provides a stable, scale-local diagnostic of structural organization.
*/
pub fn compute_omega(inter: &HashMap<u64, FixedBitSet>) -> f64 {
    let mut total = 0.0;
    let mut count = 0;

    for neighbors in inter.values() {
        let k = neighbors.ones().count();

        if k < 2 {
            continue;
        }

        let mut edges_between_neighbors = 0;
        let mut possible = 0;

        let neigh_vec: Vec<usize> = neighbors.ones().collect();

        for i in 0..neigh_vec.len() {
            for j in (i + 1)..neigh_vec.len() {
                possible += 1;
                if let Some(nbrs) = inter.get(&(neigh_vec[i] as u64)) {
                    if nbrs.contains(neigh_vec[j]) {
                        edges_between_neighbors += 1;
                    }
                }
            }
        }

        let local_c = (edges_between_neighbors as f64) / (possible as f64);
        total += local_c;
        count += 1;
    }

    if count == 0 { 0.0 } else { total / count as f64 }
}



pub fn label_frustration(h: &Hypergraph) -> usize {
    let mut mismatches = 0;
    for edge in h.hyperedges.values() {
        let mut labels = HashSet::new();
        for &v_id in &edge.vertices {
            if let Some(v) = h.vertices.get(&v_id) {
                labels.insert(v.label);
            }
        }
        if labels.len() > 1 {
            mismatches += 1;
        }
    }
    mismatches
}

pub fn defect_density(h: &Hypergraph) -> f64 {
    if h.hyperedges.is_empty() {
        return 0.0;
    }
    label_frustration(h) as f64 / h.hyperedges.len() as f64
}

pub fn local_omega(
    _h: &Hypergraph,
    inter: &HashMap<u64, FixedBitSet>,
    v: u64,
) -> f64 {
    /*
    Local contribution to Ω. Defined as the fraction of interactions 
    involving vertex v that participate in closed local motifs.
    */
    let neighbors = match inter.get(&v) {
        Some(set) => set,
        None => return 0.0,
    };
    
    if neighbors.is_empty() {
        return 0.0;
    }

    let mut closed = 0;
    for u_idx in neighbors.ones() {
        if let Some(n_u) = inter.get(&(u_idx as u64)) {
            if n_u.contains(v as usize) {
                closed += 1;
            }
        }
    }

    closed as f64 / (neighbors.ones().count() as f64).max(1.0)
}

pub fn compute_coherence_raw(neighborhood: &HashSet<u64>, inter: &HashMap<u64, FixedBitSet>) -> (u32, u32) {
    let mut internal: u32 = 0;
    let mut boundary: u32 = 0;
    for &n in neighborhood {
        if let Some(n_nbrs) = inter.get(&n) {
            for nn_idx in n_nbrs.ones() {
                if neighborhood.contains(&(nn_idx as u64)) {
                    internal += 1;
                } else {
                    boundary += 1;
                }
            }
        }
    }
    (internal / 2, boundary)
}

#[derive(Debug, Clone)]
pub struct TopologicalKnot {
    pub id: u64,
    pub vertices: HashSet<u64>,
    pub age: usize,
    pub max_size: usize,
    pub min_size: usize,
    pub radius: f64,
    // Worldline tracking
    pub coherence: f64,
    pub velocity: f64,           // scalar magnitude
    pub velocity_avg: (f64, f64), // (dx, dy) approximated over short history
    pub mass: f64,               // size * coherence^eta
    pub momentum: f64,           // m * v
    pub energy: f64,             // 0.5 * m * v^2
    pub prev_mass: f64,
    pub prev_momentum: f64,
    pub position_history: Vec<(usize, f64, f64)>, // (time, centroid_x_approx, coherence)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct InteractionEvent {
    pub start_time: usize,
    pub end_time: Option<usize>,
    pub duration: usize,
    pub knot_a: u64,
    pub knot_b: u64,
    pub overlap_size: usize,
    pub overlap_depth: f64,   // chi = max overlap reached
    pub resonance: f64,       // A = (2*coh_a*coh_b)/(coh_a^2 + coh_b^2)
    
    // Kinematic & Structural states 
    // [m, v_scalar, p_scalar, (vx, vy), coherence, mean_stability, radius, size, boundary_ratio, energy]
    pub pre_a: (f64, f64, f64, (f64, f64), f64, f64, f64, usize, f64, f64),
    pub pre_b: (f64, f64, f64, (f64, f64), f64, f64, f64, usize, f64, f64),
    pub post_a: Option<(f64, f64, f64, (f64, f64), f64, f64, f64, usize, f64, f64)>,
    pub post_b: Option<(f64, f64, f64, (f64, f64), f64, f64, f64, usize, f64, f64)>,

    // For internal lifecycle tracking
    #[serde(skip)]
    pub steps_below_threshold: usize,
}

pub fn component_radius(comp: &HashSet<u64>, inter: &HashMap<u64, FixedBitSet>) -> f64 {
    if comp.len() < 2 { return 0.0; }
    let mut total_dist = 0;
    let mut pairs = 0;
    
    for &start in comp {
        let mut visited = HashSet::new();
        visited.insert(start);
        let mut queue = vec![(start, 0)];
        let mut head = 0;
        
        while head < queue.len() {
            let (node, dist) = queue[head];
            head += 1;
            total_dist += dist;
            if dist > 0 { pairs += 1; }
            
            if let Some(nbrs) = inter.get(&node) {
                for nbr_idx in nbrs.ones() {
                    let nbr = nbr_idx as u64;
                    if comp.contains(&nbr) && !visited.contains(&nbr) {
                        visited.insert(nbr);
                        queue.push((nbr, dist + 1));
                    }
                }
            }
        }
    }
    if pairs == 0 { 0.0 } else { (total_dist as f64) / (pairs as f64) }
}

pub fn local_clustering(inter: &HashMap<u64, FixedBitSet>, v: u64) -> f64 {
    let nbrs = match inter.get(&v) {
        Some(set) => set,
        None => return 0.0,
    };
    if nbrs.len() < 2 { return 0.0; }
    let mut links = 0;
    for u_idx in nbrs.ones() {
        if let Some(u_nbrs) = inter.get(&(u_idx as u64)) {
            for w_idx in nbrs.ones() {
                if u_idx != w_idx && u_nbrs.contains(w_idx) {
                    links += 1;
                }
            }
        }
    }
    let possible = (nbrs.ones().count() * (nbrs.ones().count().saturating_sub(1))) as f64;
    (links as f64) / possible
}

pub fn detect_candidate_knot_neighborhoods(
    h: &Hypergraph,
    inter: &HashMap<u64, FixedBitSet>,
    min_coherence: f64,
) -> Vec<HashSet<u64>> {
    // Step 1: For each vertex, evaluate its 1-hop neighborhood as a candidate region
    let theta_comp = 0.6;  // compactness threshold
    
    let mut seed_regions: Vec<HashSet<u64>> = Vec::new();
    
    for &v in h.vertices.keys() {
        if let Some(neighbors) = inter.get(&v) {
            let mut neighborhood: HashSet<u64> = neighbors.ones().map(|idx| idx as u64).collect();
            neighborhood.insert(v);
            
            if neighborhood.len() < 3 { continue; }
            
            // Count internal vs boundary edges
            let (internal, boundary) = compute_coherence_raw(&neighborhood, inter);
            
            let coherence = if boundary > 0 {
                internal as f64 / boundary as f64
            } else if internal > 0 {
                10.0 // pure clique
            } else {
                0.0
            };
            
            let total = internal + boundary;
            let compactness = if total > 0 {
                internal as f64 / total as f64
            } else {
                0.0
            };
            
            // Hard threshold — NO statistics
            if coherence > min_coherence && compactness > theta_comp {
                seed_regions.push(neighborhood);
            }
        }
    }
    
    // Step 2: Merge overlapping seed regions via Union-Find and Lookup Map
    if seed_regions.is_empty() { return Vec::new(); }
    
    let n = seed_regions.len();
    let mut parent: Vec<usize> = (0..n).collect();
    
    fn find(i: usize, p: &mut Vec<usize>) -> usize {
        if p[i] == i { i }
        else {
            let root = find(p[i], p);
            p[i] = root;
            root
        }
    }
    
    fn union(i: usize, j: usize, p: &mut Vec<usize>) {
        let root_i = find(i, p);
        let root_j = find(j, p);
        if root_i != root_j {
            p[root_i] = root_j;
        }
    }

    // Map vertex -> list of seed region indices it belongs to
    let mut v_to_s: HashMap<u64, Vec<usize>> = HashMap::new();
    for (idx, seed) in seed_regions.iter().enumerate() {
        for &v in seed {
            v_to_s.entry(v).or_default().push(idx);
        }
    }

    // Evaluate merges only for seeds sharing at least one vertex
    for seeds in v_to_s.values() {
        for i in 0..seeds.len() {
            for j in i+1..seeds.len() {
                let s1 = seeds[i];
                let s2 = seeds[j];
                if find(s1, &mut parent) == find(s2, &mut parent) { continue; }
                
                let overlap = seed_regions[s1].intersection(&seed_regions[s2]).count();
                let min_s = seed_regions[s1].len().min(seed_regions[s2].len());
                if min_s > 0 && (overlap as f64 / min_s as f64) > 0.3 {
                    union(s1, s2, &mut parent);
                }
            }
        }
    }

    // Grouping seeds by Union-Find root
    let mut groups: HashMap<usize, HashSet<u64>> = HashMap::new();
    for i in 0..n {
        let root = find(i, &mut parent);
        let group = groups.entry(root).or_default();
        for &v in &seed_regions[i] {
            group.insert(v);
        }
    }

    groups.into_values().collect()
}
