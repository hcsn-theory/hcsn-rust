# HCSN Scripts Directory

This directory contains all the automation, analysis, and exploratory scripts for the HCSN project. Everything is cleanly categorized so you know exactly what is safe to run and what is an archive.

---

## 1. `pipeline/` (Active Scientific Pipeline)
This is the rigorous, peer-review-ready validation suite (v3.1). **Use these scripts for your active research.**

### Orchestrators (Bash)
- **`run_gamma_sweep.sh`**: Sweeps `γ` to find the phase transition critical point.
- **`run_conservation_test.sh`**: Runs paired unpatched vs patched simulations to prove emergent conservation.
- **`run_conservation_replication.sh`**: Massively replicates the conservation test across varying graph sizes and 10 seeds.
- **`run_universality_test.sh`**: Tests extreme parameters (noise bias, geometry freeze) to prove the Matter Phase is universal.
- **`run_aggressive_universality.sh`**: Brutally forces the engine into `Control` mode to disable structural inheritance and test pure topological emergence.

### Analyzers (Python)
*Run these after the corresponding orchestrator finishes to generate your plots and CSVs.*
- **`phase1_correlation.py`**: Performs Random Forest PCA to prove Correlation Collapse.
- **`phase2_map.py`**: Generates the 2D Phase Diagram.
- **`phase3_criticality.py`**: Computes signal jumps and sharpness to locate the exact criticality boundary.
- **`phase4_conservation.py`**: Calculates the Spearman rank correlation of momentum drift reduction.
- **`phase5_universality.py`**: Computes the true spacetime volume fraction of the Matter Phase across different universal conditions.

---

## 2. `legacy/` (Archive)
This folder contains older scripts that are preserved for historical context. **Do not use these for current research.**

### `legacy/audits/`
- Older versions of the emergence audit pipelines (e.g., v4 and v5) that were used before the physics engine was fully hardened.

### `legacy/exploratory/`
- Various Python plotting scripts, data scrapers, and the raw `test_physics.rs` files that were previously scattered in the root directory. They are kept here so no code is deleted.
