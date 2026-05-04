# Graph Report - hcsn-rust  (2026-05-04)

## Corpus Check
- 44 files · ~73,348 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 227 nodes · 312 edges · 36 communities detected
- Extraction: 89% EXTRACTED · 11% INFERRED · 0% AMBIGUOUS · INFERRED: 34 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `d6f29e11`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 8|Community 8]]
- [[_COMMUNITY_Community 9|Community 9]]
- [[_COMMUNITY_Community 10|Community 10]]
- [[_COMMUNITY_Community 11|Community 11]]
- [[_COMMUNITY_Community 12|Community 12]]
- [[_COMMUNITY_Community 13|Community 13]]
- [[_COMMUNITY_Community 14|Community 14]]
- [[_COMMUNITY_Community 15|Community 15]]
- [[_COMMUNITY_Community 16|Community 16]]
- [[_COMMUNITY_Community 17|Community 17]]
- [[_COMMUNITY_Community 18|Community 18]]
- [[_COMMUNITY_Community 19|Community 19]]
- [[_COMMUNITY_Community 20|Community 20]]
- [[_COMMUNITY_Community 21|Community 21]]
- [[_COMMUNITY_Community 22|Community 22]]
- [[_COMMUNITY_Community 23|Community 23]]
- [[_COMMUNITY_Community 24|Community 24]]
- [[_COMMUNITY_Community 25|Community 25]]
- [[_COMMUNITY_Community 26|Community 26]]
- [[_COMMUNITY_Community 27|Community 27]]
- [[_COMMUNITY_Community 28|Community 28]]
- [[_COMMUNITY_Community 29|Community 29]]
- [[_COMMUNITY_Community 30|Community 30]]
- [[_COMMUNITY_Community 31|Community 31]]
- [[_COMMUNITY_Community 32|Community 32]]
- [[_COMMUNITY_Community 33|Community 33]]
- [[_COMMUNITY_Community 34|Community 34]]
- [[_COMMUNITY_Community 35|Community 35]]

## God Nodes (most connected - your core abstractions)
1. `RewriteEngine` - 23 edges
2. `Hypergraph` - 19 edges
3. `worldline_interaction_graph()` - 8 edges
4. `main()` - 7 edges
5. `compute_omega()` - 6 edges
6. `compute_coherence_raw()` - 6 edges
7. `main()` - 6 edges
8. `main()` - 6 edges
9. `analyze_all_seeds()` - 6 edges
10. `main()` - 5 edges

## Surprising Connections (you probably didn't know these)
- `main()` --calls--> `causal_interval_size()`  [INFERRED]
  scripts/legacy/exploratory/test_physics.rs → src/observables.rs
- `main()` --calls--> `worldline_interaction_graph()`  [INFERRED]
  src/bin/robustness_pipeline.rs → src/observables.rs
- `main()` --calls--> `compute_coherence_raw()`  [INFERRED]
  src/bin/run_simulation.rs → src/observables.rs
- `main()` --calls--> `detect_candidate_knot_neighborhoods()`  [INFERRED]
  src/bin/run_simulation.rs → src/observables.rs
- `run_universe()` --calls--> `myrheim_meyer_dimension()`  [INFERRED]
  src/bin/exp_critical_scan.rs → src/observables.rs

## Communities (37 total, 24 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.19
Nodes (4): RewriteEngine, edge_creation_rule(), UndoRecord, vertex_fusion_rule()

### Community 1 - "Community 1"
Cohesion: 0.12
Nodes (3): Hyperedge, Hypergraph, Vertex

### Community 2 - "Community 2"
Cohesion: 0.11
Nodes (14): main(), run_universe(), main(), average_large_interval(), causal_interval_size(), component_radius(), compute_coherence_raw(), defect_density() (+6 more)

### Community 3 - "Community 3"
Cohesion: 0.14
Nodes (16): ChiBin, dot(), ForceLawStats, mag(), main(), ScatteringDist, SymmetryProfile, main() (+8 more)

### Community 4 - "Community 4"
Cohesion: 0.31
Nodes (4): get_mem_usage_percent(), main(), main(), Persistence

### Community 5 - "Community 5"
Cohesion: 0.39
Nodes (5): compute_correlation(), compute_mle_alpha(), main(), TrackerConfig, TrackerState

### Community 6 - "Community 6"
Cohesion: 0.67
Nodes (5): analyze_all_seeds(), extract_kinematics(), piecewise_linear(), power_law(), sigmoid()

### Community 7 - "Community 7"
Cohesion: 0.48
Nodes (5): analyze_phase_space(), causal_hierarchy_report(), main(), Tests: Does Age -> Stability? Or Independent?, Main Research Logic: Maps R2 and Signal Gain across (Age, Stability) space.

### Community 8 - "Community 8"
Cohesion: 0.4
Nodes (4): ConservationMode, DefectLogEntry, EmergenceMode, XiCurrentLogEntry

### Community 9 - "Community 9"
Cohesion: 0.5
Nodes (4): Event, fit_branching_ratio(), FitResult, main()

### Community 10 - "Community 10"
Cohesion: 0.67
Nodes (3): calculate_alpha(), main(), PureStats

### Community 11 - "Community 11"
Cohesion: 0.83
Nodes (3): compute_2d_conditional_variance(), compute_conditional_variance(), main()

## Knowledge Gaps
- **20 isolated node(s):** `UndoRecord`, `TopologicalKnot`, `InteractionEvent`, `DefectLogEntry`, `XiCurrentLogEntry` (+15 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **24 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `worldline_interaction_graph()` connect `Community 3` to `Community 0`, `Community 2`, `Community 5`?**
  _High betweenness centrality (0.044) - this node is a cross-community bridge._
- **Why does `RewriteEngine` connect `Community 0` to `Community 8`, `Community 2`?**
  _High betweenness centrality (0.030) - this node is a cross-community bridge._
- **Why does `compute_omega()` connect `Community 3` to `Community 0`, `Community 2`?**
  _High betweenness centrality (0.022) - this node is a cross-community bridge._
- **Are the 7 inferred relationships involving `worldline_interaction_graph()` (e.g. with `.step()` and `.force_second_proto_object()`) actually correct?**
  _`worldline_interaction_graph()` has 7 INFERRED edges - model-reasoned connections that need verification._
- **Are the 2 inferred relationships involving `main()` (e.g. with `worldline_interaction_graph()` and `.process_knot_update_static()`) actually correct?**
  _`main()` has 2 INFERRED edges - model-reasoned connections that need verification._
- **Are the 5 inferred relationships involving `compute_omega()` (e.g. with `.step()` and `main()`) actually correct?**
  _`compute_omega()` has 5 INFERRED edges - model-reasoned connections that need verification._
- **What connects `UndoRecord`, `TopologicalKnot`, `InteractionEvent` to the rest of the system?**
  _20 weakly-connected nodes found - possible documentation gaps or missing edges._