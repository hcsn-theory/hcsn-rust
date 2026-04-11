# HCSN-Rust: Complete Project Knowledge Map
> **Last Updated:** 2026-04-11 | **Version:** Phase 12 (Interaction Theory)
> **Author:** Saif Mukhtar — HCSN Theory Group
> **DOI:** [10.55277/researchhub.fvahxvpt.1](https://doi.org/10.55277/researchhub.fvahxvpt.1)

---

## Table of Contents
1. [Theory Background](#1-theory-background)
2. [Project Goal and Current Phase](#2-project-goal-and-current-phase)
3. [Directory Structure](#3-directory-structure)
4. [Core Library Modules (src/)](#4-core-library-modules-src)
5. [Binary Executables (src/bin/)](#5-binary-executables-srcbin)
6. [Python Orchestration Scripts](#6-python-orchestration-scripts)
7. [Analysis Scripts (scripts/)](#7-analysis-scripts-scripts)
8. [Environment Variables / Configuration](#8-environment-variables--configuration)
9. [Data Flow and Output Files](#9-data-flow-and-output-files)
10. [Key Physics Concepts and Observables](#10-key-physics-concepts-and-observables)
11. [Key Experimental Results](#11-key-experimental-results)
12. [Known Issues / Numerical Instabilities](#12-known-issues--numerical-instabilities)
13. [How to Build and Run](#13-how-to-build-and-run)
14. [Dependency Map (Cargo.toml)](#14-dependency-map-cargotoml)

---

## 1. Theory Background

**HCSN = Hierarchical Causal Structure Network**

The theory proposes that the universe is fundamentally **computational and discrete** — built from directed causal events and relations on a hypergraph. The core hypothesis is:

> "Matter (particles) are not put in — they spontaneously *emerge* as persistent, localized structural motifs (Topological Knots) from stochastic graph-rewrite dynamics applied to an initially empty or minimal causal network."

### Conceptual Pillars

| Concept | Description |
|:---|:---|
| **Hypergraph** | The "spacetime fabric" — vertices are events, hyperedges group related events |
| **Causal Relations** | Directed links encoding which events causally precede others |
| **Rewrite Rules** | Stochastic rules that transform the hypergraph (edge creation, vertex fusion) |
| **Topological Knots** | Persistent, localized subgraphs with high structural coherence; analogues of particles |
| **ξ-field** | An internal field that seeds and propagates structural defects (proto-matter) |
| **Ω (Omega)** | Hierarchical closure: mean local clustering coefficient — purely diagnostic, no feedback |
| **Structural Coherence** | Ratio of internal edges to boundary edges in a neighborhood |
| **Worldline** | The trajectory of a knot through simulation time |

---

## 2. Project Goal and Current Phase

### Overall Goal
Migrate from an ad-hoc Python particle-injection system to a **pure structural, observation-based framework** where matter condenses from a vacuum baseline through rewrite dynamics alone.

### Phase History Summary
- **Phases 1–6**: Initial Python prototype, basic emergence signals
- **Phase 7–9**: Migration to Rust; robustness validation, Pure Emergence confirmed
- **Phase 10–11**: Interaction Theory — force law discovery, scattering data
- **Phase 12 (CURRENT)**: Full interaction theory. Discovered:
  - **Threshold-activated force law** (only fires when structural overlap χ > 0.14)
  - **Critical stability** at p=0.64, γ=2.2 → power-law lifetime distribution α≈1.7–2.0
  - **Non-isotropic scattering** with mean deflection angle ≈71.5°

### Key Discoveries (Phase 12)
| Feature | Value | Physical Meaning |
|:---|:---|:---|
| Interaction Threshold χc | 0.14 | Topological "gap" protection — force only fires above this overlap |
| Coupling constant k | 182.1 | Strength of impulse transfer |
| Mean deflection angle θ | 71.5° | High-energy scattering bias; non-isotropic |
| Lifetime power-law α | 1.7–2.0 | Self-similar particle stability |
| Hazard rate decrease | ~58% over lifetime | Non-Markovian = intrinsic maturation |

---

## 3. Directory Structure

```
hcsn-rust/
│
├── Cargo.toml                      # Rust project manifest + all binary targets
├── Cargo.lock                      # Locked dependency versions
├── LICENSE                         # MIT License
├── README.md                       # High-level project README
├── PROJECT_KNOWLEDGE_MAP.md        # ← THIS FILE — comprehensive reference
├── hcsn-rust-src-overview.md       # Older partial src overview (superseded by this file)
├── validation_report_final.md      # Final robustness validation results
├── test_physics.rs                 # One-off physics test file
├── test_physics                    # Compiled test binary (4MB)
│
├── src/                            # Main Rust library + entry point
│   ├── lib.rs                      # Module declarations
│   ├── main.rs                     # Dual-core entry point (cargo run)
│   ├── hypergraph.rs               # Core graph data structures
│   ├── observables.rs              # Physics measurements + knot detection
│   ├── physics_params.rs           # Env-var-driven physics parameters
│   ├── persistence.rs              # CSV/JSON export utilities
│   ├── rewrite_engine.rs           # Main simulation engine (1550 lines)
│   ├── rules.rs                    # Stochastic rewrite rules + undo system
│   └── bin/                        # Standalone experiment binaries
│       ├── run_simulation.rs       # ★ MAIN production binary
│       ├── force_law_aggregator.rs # Parallel multi-thread interaction data gen
│       ├── force_law_analyzer.rs   # Analyzes interaction data for force law
│       ├── force_law_experiment.rs # Force law experiment runner
│       ├── force_law_fit.rs        # Curve fitting (Model A: peaked, Model B: sigmoid)
│       ├── exp_critical_scan.rs    # Sweeps p_create to find critical point
│       ├── interaction_experiment.rs # Two-probe interaction test
│       ├── phase_detection.rs      # Phase transition detector
│       ├── pure_emergence_test.rs  # Tests emergence without any feedback fields
│       ├── robustness_attack.rs    # Stress-tests particle identity
│       └── robustness_pipeline.rs  # Full robustness validation sweep
│
├── scripts/                        # Analysis shell + Python scripts
│   ├── build_phase_diagram.py
│   ├── conservation_analyzer.py
│   ├── conservation_audit_v2.py
│   ├── emergence_audit_v4.py → v5_1.py   # Emergence audit pipeline versions
│   ├── emergence_scaling_v5_2.py
│   ├── energy_audit_v3.py
│   ├── run_emergence_audit_v4.sh → v5_1.sh
│   ├── run_emergence_scaling_v5_2.sh
│   ├── run_hybrid_audit_v3.sh
│   └── run_sweep_v2.sh
│
├── hypotheses/
│   └── emergence_log.md            # Detailed running log of all hypotheses tested
│
├── research_docs/                  # Task tracking + implementation plans
│   ├── implementation_plan.md
│   ├── task.md
│   └── walkthrough.md
│
├── exports/                        # All simulation output goes here
│   ├── hcsn_main_YYYY-MM-DD_HH-MM-SS.csv    # Main CSV output (interaction events)
│   ├── hcsn_aggregator_*.csv                  # Force law aggregator output
│   ├── particle_lifetimes_p*.json             # Knot lifetime JSON per seed
│   ├── interaction_events_p*.json             # Full interaction event log
│   ├── pure_emergence_summary.json            # Pure mode results
│   └── interaction_experiment.json            # Two-probe experiment data
│
├── run_production.py               # Orchestrates Seeds 4 & 5 (p=0.64, 60k steps)
├── run_experiment_A.py             # Experiment A runner
├── run_phase_2.py                  # Phase 2 experiment runner
├── analyze_force_law.py            # Force law functional fit (Python)
├── analyze_lifetimes.py            # Ensemble hazard rate & P(τ) analysis
├── sweep_regimes.py                # Parameter regime sweep
├── interaction_law_fitting.png     # Force law fit visualization
└── scattering_angles.png           # Scattering angle distribution
```

---

## 4. Core Library Modules (src/)

### `src/lib.rs` (7 lines)
**Role:** Module gateway. Declares all 6 public modules that form the library.
```rust
pub mod hypergraph;
pub mod observables;
pub mod physics_params;
pub mod rewrite_engine;
pub mod rules;
pub mod persistence;
```

---

### `src/hypergraph.rs` (282 lines)
**Role:** The foundational data structure — represents the evolving causal hypergraph (the "spacetime").

#### Key Structs

| Struct | Fields | Purpose |
|:---|:---|:---|
| `Vertex` | `id: u64`, `depth: usize`, `label: i32`, `parents`, `children` | Single event/node. Label ∈ {+1, -1} (charge-like). Depth = causal chain length from root. |
| `Hyperedge` | `id: u64`, `vertices: Vec<u64>` | Group of causally related events |
| `Hypergraph` | `vertices`, `hyperedges`, `causal_future`, `causal_past` | Entire causal network. Causal sets stored as `FixedBitSet`s for O(1) membership queries |

#### Key Methods

| Method | What it does |
|:---|:---|
| `add_vertex()` | Creates a vertex, initializes its causal bitsets |
| `add_causal_relation(u, v)` | Adds directed causal link u→v. Updates transitive closure via bitset OR propagation |
| `add_hyperedge(verts)` | Creates a hyperedge grouping those vertices |
| `merge_causal_identity(keep, remove)` | Merges causal histories during vertex fusion |
| `is_causally_related(u, v)` | O(1) reachability check using bitsets |
| `scrub_ghost_bits()` | Periodic cleanup — removes causal references to deleted vertices |
| `causal_future(v)` / `causal_past(v)` | Returns J+(v) or J-(v) as a HashSet |
| `max_chain_length()` | Causal depth = longest worldline in the graph |
| `average_coordination()` | Mean hyperedge degree per vertex |
| `execute_undo_record(record)` | Reverts a rewrite step using an `UndoRecord` |

#### Design Notes
- IDs are global atomics (`AtomicU64`), so they are unique even across threads
- Bitsets start at capacity 524288, growing dynamically
- Depth is updated lazily when causal relations are added

---

### `src/observables.rs` (505 lines)
**Role:** All physics measurements and the knot (particle) detection pipeline.

#### Measurement Functions

| Function | What it computes |
|:---|:---|
| `average_coordination(h)` | Mean number of hyperedges per vertex |
| `causal_interval_size(h, u, v)` | `\|J⁺(u) ∩ J⁻(v)\|` — spacetime volume between events |
| `myrheim_meyer_dimension(h, samples, min_interval)` | Estimates emergent spacetime dimension using causal interval statistics |
| `adjacency_overlap(before, after)` | Fraction of hyperedges that survived a rewrite |
| `interaction_concentration(interactions)` | Φ = max_degree / total_degree — concentration metric |
| `worldline_interaction_graph(h, fraction)` | Builds interaction graph among deep worldlines (depth ≥ fraction × max_depth) |
| `count_triangles(interactions)` | Counts 3-cliques in interaction graph |
| `compute_omega(inter)` | Ω = mean local clustering coefficient (Watts–Strogatz style) |
| `label_frustration(h)` | Count of hyperedges with mixed vertex labels |
| `defect_density(h)` | Fraction of frustrated edges |
| `local_omega(h, inter, v)` | Local Ω contribution at vertex v |
| `compute_coherence_raw(neighborhood, inter)` | Returns (internal_edges, boundary_edges) |
| `component_radius(comp, inter)` | Mean BFS distance within a subgraph |
| `local_clustering(inter, v)` | Local clustering coefficient at v |
| `detect_candidate_knot_neighborhoods(h, inter, min_coherence)` | ★ Core detector: finds all coherent subgraphs (particle candidates) using coherence + compactness thresholds, then merges overlapping regions via Union-Find |

#### Key Structs

```rust
pub struct TopologicalKnot {
    pub id: u64,
    pub vertices: HashSet<u64>,
    pub age: usize,
    pub max_size / min_size: usize,
    pub radius: f64,
    pub coherence: f64,
    pub velocity: f64,
    pub velocity_avg: (f64, f64),   // (dx, dy)
    pub mass: f64,                  // size * coherence²
    pub momentum: f64,              // m * v
    pub energy: f64,                // 0.5*m*v² + 0.02*stability
    pub prev_mass / prev_momentum: f64,
    pub position_history: Vec<(time, centroid_x, coherence)>,
}

pub struct InteractionEvent {
    pub start_time / end_time / duration: usize,
    pub knot_a / knot_b: u64,
    pub overlap_size: usize,
    pub overlap_depth: f64,     // χ = max overlap fraction
    pub resonance: f64,         // A = 2*Ca*Cb / (Ca²+Cb²)
    pub pre_a / pre_b: (m, v, p, (vx,vy), coh, stab, radius, size, boundary_ratio, energy),
    pub post_a / post_b: Option<same>,
    pub steps_below_threshold: usize,
}
```

#### Knot Detection Algorithm
1. For each vertex v, compute its 1-hop neighborhood N(v)
2. Compute coherence = internal_edges / boundary_edges
3. Compute compactness = internal / total (threshold: 0.6)
4. Keep neighborhood as "seed" if both thresholds met
5. Merge overlapping seeds via Union-Find (overlap > 30% of smaller group → merge)
6. Return merged groups as candidate knots

---

### `src/physics_params.rs` (70 lines)
**Role:** Centralizes all tunable physics parameters, loaded from environment variables with defaults.

| Parameter | Env Var | Default | Meaning |
|:---|:---|:---|:---|
| `gamma_defect` | `HCSN_GAMMA_DEFECT` | 0.15 | Defect injection rate |
| `inertia_scale` | `HCSN_INERTIA_SCALE` | 1.0 | Inertia scaling factor |
| `interaction_boost` | `HCSN_INTERACTION_BOOST` | 1.02 | Interaction amplitude boost |
| `stability_decay` (ν) | `HCSN_NU` | 0.975 | Per-step stability decay rate |
| `nonlinear_coupling` (γ) | `HCSN_GAMMA` | 2.2 | Memory nonlinearity exponent |
| `memory_coupling` (μ) | `HCSN_MU` | 0.3 | How strongly stability suppresses rewrites |
| `defect_injection` | `HCSN_DEFECT_INJECTION` | 0.0 | Spontaneous vacuum nucleation rate |
| `noise_bias` | (hardcoded) | 0.0 | Not used in env; set inline |
| `geometry_freeze` | (hardcoded) | 0.9 | Distance memory decay rate |

---

### `src/persistence.rs` (64 lines)
**Role:** All file I/O for streaming simulation results to disk.

| Method | What it does |
|:---|:---|
| `generate_filename(prefix)` | Creates `exports/hcsn_<prefix>_YYYY-MM-DD_HH-MM-SS.csv` |
| `write_header(writer)` | Writes the 17-column CSV header |
| `format_event(event)` | Converts an `InteractionEvent` to a CSV row (skips events with duration < 3) |
| `open_writer(filename)` | Opens a `BufWriter<File>` in append mode |

#### CSV Columns (17 total)
```
pre_px, pre_py, post_px, post_py,
pre_p_mag, post_p_mag,
pre_mass, post_mass,
pre_s_sum, post_s_sum, pre_s_mean, post_s_mean,
chi,
duration,
stability_bin,
pre_E_total, post_E_total
```
Where:
- `p_x` = total momentum x-component (mass × velocity_x, summed over both knots)
- `mass` = size × coherence²
- `s` = mean stability of vertices in the knot
- `chi` = peak structural overlap fraction during the interaction
- `stability_bin` = rounded bucket of pre_s_mean (in steps of 5)
- `E_total` = kinetic + 0.02×stability

---

### `src/rules.rs` (203 lines)
**Role:** Defines the two fundamental stochastic rewrite operations and the undo/rollback system.

#### `UndoRecord` Struct
Stores all changes made during a single rewrite so it can be reversed:
```rust
pub struct UndoRecord {
    pub target: Vec<u64>,           // Vertices that were the anchor
    pub added_vertices: Vec<u64>,
    pub added_edges: Vec<u64>,
    pub added_causal: Vec<(u64, u64)>,
    pub removed_vertex: Option<Vertex>,
    pub kept_vertex: Option<u64>,
    pub removed_edges: HashMap<u64, Hyperedge>,
    pub old_causal_future/past: HashMap<u64, FixedBitSet>,
    pub old_parents/children: HashMap<u64, Vec<u64>>,
}
```

#### Rule 1: `edge_creation_rule(h, anchor, p_rule)`
**"Birth"** — Grows the graph:
1. Select a random hyperedge (optionally anchored to a vertex)
2. With probability `p_rule` (loop closure): connect 2 random existing vertices if not already causally related
3. Create a new vertex
4. Connect it causally to all vertices in the selected edge
5. With probability 0.3 per vertex, also connect ancestors via "causal thickening"
6. Create a new hyperedge containing all the edge's vertices + the new vertex

#### Rule 2: `vertex_fusion_rule(h, anchor)`
**"Contraction"** — Shrinks the graph:
1. Requires ≥3 vertices and ≥1 edge with 3+ vertices
2. Selects two vertices from an edge (keep one, remove one)
3. Merges the causal identity of the removed vertex into the kept one
4. Redirects all causal connections (parents/children)
5. Removes all hyperedges that contained the removed vertex
6. Deletes the vertex

---

### `src/rewrite_engine.rs` (1550 lines)
**Role:** The heart of the simulation. Orchestrates the complete per-step update loop.

#### Key Enums

```rust
pub enum EmergenceMode {
    Control,   // No stability bonus, hard threshold — baseline
    Assisted,  // Honest inheritance + sigmoid threshold — DEFAULT
    Forced,    // Constant bonus + hard threshold — experimental
}

pub enum ConservationMode {
    Baseline,           // No conservation
    Pairwise,           // Symmetric momentum correction
    StabilityScaled,    // Inertial cooling (v5.0)
    FluxCompensated,    // Momentum reservoir leak/absorb (Hyp C)
    MassCoupled,        // Soft Newtonian correction (Hyp D)
    TimeSymmetry,       // Stochastic undo for conservation (Hyp E)
    Hybrid,             // Pairwise + FluxCompensated — PRODUCTION DEFAULT
}
```

#### `RewriteEngine` State Fields
```
h: Hypergraph                       — the live causal network
p_create: f64                       — base edge creation probability
mode: EmergenceMode                 — which emergence regime
pure_mode: bool                     — bypass all memory/stability fields
params: PhysicsParams               — physics parameters
xi: HashMap<u64, f64>              — the ξ-field (defect/matter seed field)
xi_threshold: 1e-6                  — minimum xi to propagate
xi_decay: 0.70                      — xi decays 70% each propagation step
xi_coupling: 0.6                    — xi propagation coupling constant
active_knots: HashMap<u64, TK>      — currently alive topological knots
dead_knots: Vec<TK>                 — knots that have dissolved (last 200)
interaction_events: Vec<IE>         — completed interaction records
stability: HashMap<u64, f64>        — per-vertex stability memory
conservation_mode: ConservationMode — how momentum is conserved
momentum_reservoir: HashMap<u64, f64> — flux compensation reservoir
topo_distance_memory                — G1: inter-component topology distances
xi_distance_memory                  — G2: inter-cluster xi distances
distance_memory_decay: 0.9          — exponential memory decay rate
geometry_stride: 5                  — geometry update frequency (steps)
```

#### `step()` — The Main Loop (called once per simulation tick)
```
1. Increment time counter
2. Every 100 steps: scrub ghost bits
3. If first step: bootstrap the interaction graph cache
4. [TimeSymmetry] Stochastic undo of previous step if momentum error > 0.5
5. [if defect_injection > 0] Spontaneous vacuum nucleation (4-vertex clique)
6. Propose a rewrite (propose_rewrite):
   a. Compute local density = clustering × (degree / avg_degree)
   b. Compute local coherence (internal/boundary edges)
   c. Apply unified suppression: alpha_eff = 2.0 + coherence_boost + memory_contribution
   d. Rewrite probability = exp(-alpha_eff × density)
   e. If not suppressed: apply growth bias based on coherence vs nucleation threshold
   f. Call edge_creation_rule or vertex_fusion_rule
7. Accept/reject (always accepted at probability 1.0 currently)
8. If accepted:
   a. Update interaction graph incrementally (delta update)
   b. Propagate ξ-field through network
   c. Execute deferred causal bridge (if pending)
   d. Update geometry memory (every 5 steps)
   e. Every 10 steps: update_topological_knots(), update_stability(), kinematics
9. Print diagnostics every print_interval steps
```

#### `propose_rewrite()` — Suppression Logic
The engine suppresses rewrites to protect coherent structures:
```
alpha_eff = alpha_base (2.0)
          + coherence_boost (if size ≥ 4 and coherence > 1.0: λ×(coh-1.0), λ=0.5)
          + memory_contribution (μ × cap × (stability/cap)^γ)

rewrite_prob = exp(-alpha_eff × coupling_modifier × local_density)

coupling_modifier = 0.2 if vertex is in an active deep-overlap interaction, else 1.0
                    → Reduces protection during interactions, allowing scattering
```

#### Kinematics (perform_kinematics_and_interactions)
```
For each active knot:
  mass = size × coherence²
  velocity = Δcentroid / Δtime  (optionally cooled by stability in Hybrid mode)
  momentum = mass × velocity
  energy = 0.5 × mass × velocity² + 0.02 × mean_stability

Overlap Detection:
  chi = |intersection| / min(|A|, |B|)
  If chi > 0.015: start or update InteractionEvent
  If chi > 0.4: mark vertices as "coupled" (reduced rewrite protection)

Interaction Finalization:
  When overlap drops to 0 for 1 check (= 10 steps):
    Capture post-state, compute final duration, push to interaction_events
```

#### Conservation Modes (Hypotheses A–E)
- **Pairwise (A)**: Both knots in an interaction receive symmetric momentum correction: `correction = -k × 0.5 × Δp_total`, where k ramps with stability (physics emerges as stability → 20)
- **FluxCompensated (C)**: Excess momentum leaked to a per-vertex reservoir; diffusively re-absorbed at rate 0.15; dead vertex momentum diffused to neighbors
- **MassCoupled (D)**: Soft Newtonian correction: `30% × (p_before - p_after)`
- **TimeSymmetry (E)**: Stochastic undo (p=0.2 if high error, p=0.05 otherwise)
- **Hybrid (DEFAULT)**: Combines Pairwise + FluxCompensated

#### Stability System
```
Each step (every 10):
  All stability values *= stability_decay (0.975)
  For each vertex in an active knot: stability += 1.0
  Stability is capped at 50.0
  Dead-vertex entries are pruned
```

#### ξ-Field Propagation
```
new_xi[u] += 0.15 × xi[v] × xi_decay / degree(v)   (to each neighbor u)
new_xi[v] += 0.70 × xi[v] × xi_decay               (self-retention)
Cap: xi_max = 1e6
```
Cluster-protected: xi does not propagate across different xi-clusters when a forced injection is active.

---

### `src/main.rs` (78 lines)
**Role:** Default entry point when `cargo run` is called. Dual-core parallel simulation.

- Reads `HCSN_STEPS` (default 250000) and `HCSN_P_CREATE` (default 0.58) from env
- Hard-locks to 2 threads
- Each thread initializes its own `Hypergraph` and `RewriteEngine`
- Mode: `EmergenceMode::Assisted`, `ConservationMode::Hybrid`
- Streams interaction events to shared CSV file (every 2000 steps)

---

## 5. Binary Executables (src/bin/)

### ★ `run_simulation` (run_simulation.rs — 243 lines)
**THE MAIN PRODUCTION BINARY** — use this for experiments.

**Usage:**
```bash
cargo run --release --bin run_simulation -- \
  --steps 60000 \
  --p_create 0.64 \
  --seed 1 \
  [--log-to /path/to/logfile.jsonl]
```

**What it does:**
1. Initializes a 2-vertex seed hypergraph
2. Runs the simulation loop with Ctrl-C safe-exit (SIGINT handler)
3. Prints progress table every `sample_interval` steps:
   ```
   time | V | <k> | Δ<k> | L | ΔL | acc% | omega | knots | all_k | max_coh | step_ms
   ```
4. On completion or interrupt, saves:
   - `exports/particle_lifetimes_p{P}_s{SEED}.json` — worldline data for all knots
   - `exports/interaction_events_p{P}_s{SEED}.json` — all interaction events
5. Auto-detects Gantry log directory for live monitoring

---

### `force_law_aggregator` (force_law_aggregator.rs — 94 lines)
**Purpose:** Generates large interaction datasets in parallel for force-law fitting.

- Runs 2 threads × `HCSN_STEPS` (default 125000) steps each
- Seeds 16 pre-formed 4-vertex cliques per thread (instead of minimal 2-vertex seed)
- Uses `ConservationMode::Hybrid`, `EmergenceMode::Assisted`
- Streams results to `exports/hcsn_aggregator_*.csv`

**Env Vars:** `HCSN_P_CREATE`, `HCSN_STEPS`, `HCSN_EMERGENCE_MODE`

---

### `force_law_fit` (force_law_fit.rs — 128 lines)
**Purpose:** Fits two functional models to the χ → Δp interaction data.

- Reads `exports/interaction_points_raw.csv`
- Bins data into 50 bins for noise reduction
- **Model A**: `Δp = A × χ × exp(-χ/x₀)` (Peaked Coupling)
- **Model B**: `Δp = L / (1 + exp(-k × (χ - xc)))` (Sigmoidal Saturation)
- Reports best-fit parameters and R²
- Prints the winner model

---

### `exp_critical_scan` (exp_critical_scan.rs — 48 lines)
**Purpose:** Sweeps `p_create` to find the critical point where the graph transitions from sparse to complex.

- Scans: p ∈ [0.47, 0.48, 0.49, 0.50, 0.51, 0.52, 0.53]
- For each p: runs 10000 steps, records vertices, ⟨k⟩, Myrheim–Meyer dimension
- Identified critical region near p ≈ 0.51–0.52

---

### `interaction_experiment` (interaction_experiment.rs — 145 lines)
**Purpose:** Controlled two-probe interaction experiment.

1. Runs until Ω ≈ 1.10 (target structural organization)
2. Injects first proto-particle via `force_defect(0.3, 30 tries)`
3. Waits 150 steps for stabilization
4. Injects second proto-particle via `force_second_proto_object()`
5. Runs 1500 steps observing the interaction
6. Saves to `exports/interaction_experiment.json`

---

### `pure_emergence_test` (pure_emergence_test.rs — 168 lines)
**Purpose:** Tests if particles emerge from rules alone, with ALL feedback fields disabled.

- Sets `engine.pure_mode = true` (bypasses ξ, stability, coherence-gates)
- Starts from a 4-vertex clique seed
- Runs 25000 steps
- Computes MLE power-law exponent α for lifetime distribution
- **Key Result:** TRUE EMERGENCE confirmed (α=1.283, max lifetime 18,690 steps)
- Prints hazard rate table

---

### `robustness_pipeline` (robustness_pipeline.rs — 174 lines)
**Purpose:** Validates that particles are robustly detected across a range of coherence/overlap thresholds.

- Runs 20000 steps with a single engine
- Every 10 steps, re-runs `process_knot_update_static()` with ALL 20 parameter combinations:
  - Coherence θ ∈ {1.2, 1.4, 1.6, 1.8, 2.0}
  - Overlap OV ∈ {0.5, 0.6, 0.7, 0.8}
- Reports particle count, α, PSI, and lifetime correlation vs baseline
- **Result:** α varies only ±5.4%, confirming structural invariance

---

### `robustness_attack` (robustness_attack.rs — 6141 bytes)
**Purpose:** Stress-tests whether identified particles maintain identity under extreme perturbations.

---

### `phase_detection` (phase_detection.rs — 5723 bytes)
**Purpose:** Detects phase transitions in the evolving hypergraph.

---

### `force_law_analyzer` / `force_law_experiment` (8817 / 8749 bytes)
**Purpose:** Detailed analysis and runner for the topological force law experiments.

---

## 6. Python Orchestration Scripts

### `run_production.py`
Orchestrates production runs for seeds 4 and 5:
```python
seeds = [4, 5]
p_create = 0.64
steps = 60000
# Runs: cargo run --release --bin run_simulation -- --steps 60000 --p_create 0.640 --seed {s}
```

### `run_phase_2.py`
Phase 2 experiment orchestration.

### `run_experiment_A.py`
Experiment A specific runner.

### `analyze_force_law.py`
Python-side functional fitting of the interaction force law from CSV exports.

### `analyze_lifetimes.py`
Ensemble hazard rate analysis and P(τ) power-law fitting from JSON exports.

### `sweep_regimes.py`
Sweeps multiple parameter regimes to map the phase space.

---

## 7. Analysis Scripts (scripts/)

| Script | Purpose |
|:---|:---|
| `build_phase_diagram.py` | Builds a 2D phase diagram from swept parameters |
| `conservation_analyzer.py` | Checks momentum/energy conservation in exported data |
| `conservation_audit_v2.py` | Updated conservation audit |
| `emergence_audit_v4/5/v5_1.py` | Full emergence pipeline: detects, classifies, and counts knots |
| `emergence_scaling_v5_2.py` | Scaling analysis of emergent structures |
| `energy_audit_v3.py` | Energy balance audit across interaction events |
| Shell scripts (`.sh`) | Runner scripts for the Python analysis tools, configuring env vars |

---

## 8. Environment Variables / Configuration

All physics parameters can be overridden at runtime:

```bash
# Core simulation
HCSN_STEPS=250000          # Number of simulation steps per thread
HCSN_P_CREATE=0.64         # Base edge creation probability

# Physics parameters (mapped to PhysicsParams struct)
HCSN_GAMMA_DEFECT=0.15     # Defect sensitivity
HCSN_INERTIA_SCALE=1.0     # Inertia scaling
HCSN_INTERACTION_BOOST=1.02
HCSN_NU=0.975              # Stability decay per step (ν)
HCSN_GAMMA=2.2             # Nonlinear coupling exponent (γ)
HCSN_MU=0.3                # Memory coupling strength (μ)
HCSN_DEFECT_INJECTION=0.0  # Rate of spontaneous 4-vertex nucleation

# For force_law_aggregator only
HCSN_EMERGENCE_MODE=Assisted  # Control | Assisted | Forced
```

---

## 9. Data Flow and Output Files

```
Simulation Engine
     │
     ├── Every 2000 steps ──► exports/hcsn_*.csv
     │                         (streaming via BufWriter + Mutex)
     │
     └── On completion ──────► exports/particle_lifetimes_p{P}_s{SEED}.json
                           └── exports/interaction_events_p{P}_s{SEED}.json
```

### CSV Schema (17 columns)
See [persistence.rs](#srcpersistencers-64-lines) section for full column definitions.

**⚠️ KNOWN ISSUE:** The CSV exports can contain `NaN` and `inf` values due to numerical overflow. Quantities like `post_px` and `post_p_mag` can grow to astronomically large numbers (100s of digits) in early simulation steps, then overflow to `inf`/`NaN`. This is the primary known bug requiring investigation. See Section 12.

### JSON Schema (particle_lifetimes)
```json
{
  "id": 1,
  "status": "dead|alive",
  "age": 1500,
  "max_size": 12,
  "radius": 2.3,
  "coherence": 1.8,
  "velocity": 0.5,
  "velocity_avg": [0.5, 0.0],
  "mass": 38.88,
  "momentum": 19.44,
  "worldline_length": 150,
  "particle_candidate": true,
  "mean_stability": 12.5
}
```

---

## 10. Key Physics Concepts and Observables

### Coherence (κ)
```
κ = internal_edges / boundary_edges
```
Measures how much a neighborhood "faces inward" vs outward. High coherence (κ > 1.2) → knot candidate.

### Structural Overlap (χ)
```
χ = |A ∩ B| / min(|A|, |B|)
```
Fraction of the smaller knot that overlaps the larger. Used as the force law input.

### Omega (Ω) — Hierarchical Closure
```
Ω = mean Watts-Strogatz local clustering coefficient
```
Purely diagnostic. Does NOT feed back into the dynamics. Typical value at steady state: ≈0.67–0.693.

### Myrheim–Meyer Dimension
Estimates emergent spacetime dimension from causal interval statistics:
```
d = 2 × ln(N) / ln(⟨|I(u,v)|⟩)
```
where |I(u,v)| = |J⁺(u) ∩ J⁻(v)|.

### Mass
```
m = size × coherence²
```

### Momentum / Energy
```
p = m × v_x
E = 0.5 × m × v² + 0.02 × mean_stability
```

### Power-Law Exponent (α)
MLE estimate of lifetime distribution P(τ) ∝ τ^(-α):
```
α = 1 + N / Σ ln(τᵢ / τ_min)
```
Target regime: 1.7 ≤ α ≤ 2.0.

---

## 11. Key Experimental Results

### Phase Diagram
- **p_create < 0.50**: Sparse regime, no persistent structures
- **p_create ≈ 0.51–0.52**: Critical point — phase transition
- **p_create = 0.64**: Targeted production parameter — rich structural regime

### Topological Force Law
```
Δp = 0 (if χ < 0.14)
Δp ≈ 182.1 × χ × exp(-χ/0.3)  (if χ ≥ 0.14)
```
- Threshold-activated: no interaction below χ_c = 0.14 (topological protection)
- Peaked decay form (Model A wins over sigmoidal Model B)

### Robustness Validation
| Coherence θ | Count | α | PSI | Correlation r |
|:---|:---|:---|:---|:---|
| 1.2 | 264 | 1.83 | 0.20 | 1.00 (baseline) |
| 2.0 | 64 | 1.79 | 0.55 | 0.57 |

**Conclusion:** α invariant at ±5.4% across detection thresholds.

### Pure Mode (No Feedback)
- Max lifetime: 18,690 steps
- α = 1.283 (true emergence zone)
- Hazard rate decreases 58% over particle lifetime

---

## 12. Known Issues / Numerical Instabilities

### Critical Bug: Exponential Momentum Overflow

**Symptom:** In the exported CSV (e.g., `hcsn_main_2026-04-10_03-25-25.csv`):
- `post_px` and `post_p_mag` grow to values with 200–300+ digits within the first ~50 rows
- By row ~94, values become `NaN` and `inf`
- Subsequent rows have `NaN, 0.000000` for all momentum and position fields

**Root Cause (Suspected):**
The momentum calculation in `persistence.rs` computes:
```rust
let m = size as f64 * coherence.powi(2);
let pre_px = (m_a * velocity_x_a) + (m_b * velocity_x_b);
```
But velocity is computed as `Δcentroid / Δtime` where centroid is the mean vertex ID — an unbounded, monotonically increasing quantity as new vertices are added. Combined with mass also growing as the graph grows and coherence potentially high, momentum can compound exponentially.

Additionally, the ξ-field propagation uses `xi_max = 1e6` as a cap but the momentum computation is uncapped.

**Impact:** All momentum/energy columns in the CSV are unreliable. Mass and stability columns (which don't use vertex IDs) appear more reasonable.

**Work Needed:**
- Switch velocity to track centroid via a rolling relative position (delta from previous centroid)
- Apply momentum bounds checking before writing to CSV
- Consider normalizing vertex IDs or using structural metrics for position proxy

---

## 13. How to Build and Run

### Prerequisites
- Rust (stable), `cargo`
- Python 3.x (for orchestration scripts)

### Build
```bash
cd /home/saif/hcsn-nexus/hcsn-rust
cargo build --release
```

### Quick Simulation (Single Seed)
```bash
cargo run --release --bin run_simulation -- --steps 10000 --p_create 0.64 --seed 1
```

### Production Run (Seeds 4–5)
```bash
python3 run_production.py
```

### Force Law Data Generation
```bash
HCSN_STEPS=100000 HCSN_P_CREATE=0.64 cargo run --release --bin force_law_aggregator
```

### Analyze Force Law (Python)
```bash
python3 analyze_force_law.py
```

### Pure Emergence Test
```bash
cargo run --release --bin pure_emergence_test
```

### Critical Point Scan
```bash
cargo run --release --bin exp_critical_scan
```

### Robustness Validation
```bash
cargo run --release --bin robustness_pipeline -- --steps 20000
```

---

## 14. Dependency Map (Cargo.toml)

| Crate | Version | Purpose |
|:---|:---|:---|
| `rand` 0.8 (small_rng) | RNG | Random number generation (thread_rng) |
| `serde` 1.0 (derive) | Serialization | JSON/CSV struct serialization |
| `serde_json` 1.0 | JSON output | Export to `.json` files |
| `ctrlc` 3.2 | Signal handling | SIGINT safe-exit in run_simulation |
| `rayon` 1.7 | Parallelism | Parallel thread iteration in aggregator/main |
| `fixedbitset` 0.4 | Bitsets | O(1) causal set membership, O(N) union |
| `chrono` 0.4 | Timestamps | Filename generation with current datetime |

---

## 15. Hypotheses and Experiments Log

See [`hypotheses/emergence_log.md`](hypotheses/emergence_log.md) for the full running log of all 12+ phases of experiments, each with hypothesis, test, and result.

See [`research_docs/walkthrough.md`](research_docs/walkthrough.md) for the most recent phase walkthrough.

---

*This file was auto-generated by comprehensive code reading on 2026-04-11.*
*For any changes to the codebase, update the relevant section of this file.*
