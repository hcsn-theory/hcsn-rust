# hcsn-rust Source Code Overview

This document provides a comprehensive breakdown of the main Rust source files in the `src/` folder of the `hcsn-rust` project, summarizing their purpose and key logic.

---

## Core Library Files

### `lib.rs`
Declares and exposes all main modules:
- `hypergraph`
- `observables`
- `physics_params`
- `rewrite_engine`
- `rules`
- `persistence`

---

### `hypergraph.rs`
Implements the core data structures:
- `Vertex`, `Hyperedge`, and `Hypergraph` (with causal relations, bitset-based reachability, and worldline tracking).
- Methods for adding vertices/edges, updating causal structure, merging identities, and undoing changes.

---

### `observables.rs`
Provides physical/statistical measurements:
- Coordination number, causal interval size, Myrheim-Meyer dimension estimator, interaction graphs, clustering, knot detection, and more.
- Defines `TopologicalKnot` and `InteractionEvent` structs for tracking emergent structures and events.

---

### `physics_params.rs`
Manages simulation parameters, loading them from environment variables.
- Parameters include coupling constants, noise, decay rates, etc., encapsulated in the `PhysicsParams` struct.

---

### `persistence.rs`
Handles export of simulation data (CSV/JSON), especially for interaction events and worldline statistics.
- Provides utilities for generating filenames, writing headers, formatting events, and opening writers.

---

### `rewrite_engine.rs`
Implements the main simulation loop, state, and step logic.
- Handles stochastic application of rewrite rules, knot (particle) tracking, and interaction event logging.

---

### `rules.rs`
Encodes the stochastic graph rewrite rules (edge creation, vertex fusion, etc.) and undo/rollback logic for simulation steps.
- Defines `UndoRecord` for reversible operations.

---

### `main.rs`
Example entry point for a dual-core simulation.
- Sets up parameters, initializes the hypergraph, runs parallel simulations, and streams results to disk.

---

## Binaries (`src/bin/`)
Each file in `src/bin/` is a separate executable for specific experiments or analyses (e.g., `run_simulation.rs`, `exp_critical_scan.rs`, `force_law_fit.rs`, etc.).
- They typically instantiate a `Hypergraph`, wrap it in a `RewriteEngine`, and run a simulation loop, exporting results for analysis.

---

If you need a deep dive into a specific file or further details on the binaries, let me know!
