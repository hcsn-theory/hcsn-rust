# 🦀 HCSN Rust — Hierarchical Causal Structure Network

[![DOI](https://img.shields.io/badge/DOI-10.55277%2Fresearchhub.fvahxvpt.1-blue)](https://doi.org/10.55277/researchhub.fvahxvpt.1)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![ORCID](https://img.shields.io/badge/ORCID-0009--0004--1698--5729-green.svg)](https://orcid.org/0009-0004-1698-5729)

---

> **HCSN** explores the hypothesis that the universe is fundamentally computational —
> built from discrete events and directed causal relations, with no assumed background space, time, or quantum framework.

📺 **[Watch the overview on YouTube →](https://youtu.be/A0oh6Rlx03Y)**
📄 **[Read the paper on ResearchHub →](https://doi.org/10.55277/researchhub.fvahxvpt.1)**

---

## ✨ Overview: The Rust Migration

This repository contains the **high-performance Rust implementation** of the HCSN simulation engine.

The original engine was written in Python (`hcsn-sim`). While highly flexible for prototyping, the deep causal graph traversal required for detecting emergent structural geometry (like `hierarchical_closure`) scales poorly in Python.

* **Exact Parity:** The Rust version guarantees a 1:1 algorithmic mapping of the Python origin.
* **Massive Performance Gain:** Processes 10,000 algorithmic steps (including complex geometrical observables) in under 90 seconds on a single thread. The original Python engine takes considerably longer (~15x to 40x slower).
* **Environment-Driven:** Physics parameters are controlled via environment variables at runtime, sidestepping the need for recompilation.

---

## 📋 Table of Contents

- [Repository Structure](#repository-structure)
- [Quick Start](#quick-start)
- [Physics Parameters](#physics-parameters)
- [Diagnostics Explained](#diagnostics-explained)
- [Companion Repositories](#companion-repositories)
- [Citation](#citation)
- [License & Contact](#license--contact)

---

## Repository Structure

The `hcsn-rust/` workspace is fully self-contained as a standard Cargo crate:

```text
hcsn-rust/
├── Cargo.toml                  # Workspace & crate definition
├── src/
│   ├── lib.rs                  # Module declarations
│   ├── hypergraph.rs           # Vertex, Hyperedge, causal memory DAG
│   ├── rules.rs                # Local rewrite rules (creation, fusion)
│   ├── observables.rs          # Physics diagnostics (<k>, Ω, Φ, Ψ, Dimensions)
│   ├── physics_params.rs       # Environment-variable controlled parameters
│   └── rewrite_engine.rs       # Main event loop, ξ field propagation
└── src/bin/
    ├── run_simulation.rs       # Main simulation universe block
    ├── exp_critical_scan.rs    # Scan p_create values and estimate dimensions
    └── interaction_experiment.rs # Two defect proto-particle interaction test
```

---

## Quick Start

### 1. Requirements

Ensure you have the Rust toolchain installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Build and Run the Main Simulation

This runs the main simulation loop and prints periodic diagnostic tables mimicking the Python engine's terminal output.

```bash
cd hcsn-rust
cargo run --bin run_simulation --release
```

### 3. Run the Dimension Criticality Scan

Evaluate the emergent Myrheim-Meyer dimension across varying creation probabilities (`p_create`).

```bash
cargo run --bin exp_critical_scan --release
```

### 4. Run the Interaction Experiment

Simulates the injection of two defect proto-particles into a stable geometry and records their interaction metrics to JSON.

```bash
cargo run --bin interaction_experiment --release
```

---

## Physics Parameters

Constants are removed from the source code. The physics of the simulation can be dynamically tuned by passing environment variables:

| Variable | Default Value | Description |
|---|---|---|
| `HCSN_GAMMA_DEFECT` | `0.15` | Defect penalization constant scaling acceptance probability. |
| `HCSN_INERTIA_SCALE` | `1.0` | Mass/inertia resistance modifier. |
| `HCSN_INTERACTION_BOOST` | `1.02` | Interaction coefficient for the shared rewrite pool. |

Example overriding default parameter execution:
```bash
HCSN_GAMMA_DEFECT=0.2 cargo run --bin run_simulation --release
```

---

## Diagnostics Explained

The generated numerical measurements match the Python framework identically:

| Symbol | Name | Meaning | Target Range |
|:------:|:-----|:--------|:------------|
| ⟨k⟩ | Avg coordination | Controls effective dimensionality | ≈ 7.5–8.5 for spacetime-like geometry |
| L | Causal depth | Maximum causal chain length — emergent time | Grows with rewrites |
| Φ | Interaction concentration | Hub dominance (lower = more uniform) | Small Φ preferred |
| Ψ | Closure density | Redundancy in causal closure | Non-zero = error correction |
| Ω | Hierarchical closure | RG-like stability across scales | > 1.0 for persistent structure |

**Phase Interpretation:**

| Ω Regime | Behavior |
|----------|----------|
| Ω < 1.0 (subcritical) | Transient defects, no stable transport |
| Ω ≈ 1.08–1.18 (critical) | Phase transition, marginal stability |
| Ω > 1.2 (supercritical) | Persistent worldlines, stable emergent structure |

---

## Companion Repositories

* **[HCSN Theory](https://github.com/hcsn-theory/HCSN-core-Theory):** Canonical documentation, methodologies, and derivations for emergent mass, metrics, and particles.
* **[HCSN Python Sim](https://github.com/hcsn-theory/hcsn-sim):** Origin Python implementation, visualization suites, CSV exporters, Blender cinematic renderers, and analysis macros.

---

## Citation

If you use HCSN in your research, please cite the framework documentation:

> Saif Mukhtar. *HCSN: A Hierarchical Causal Structure Network Framework for Emergent Physics.* ResearchHub, 2026. DOI: [10.55277/researchhub.fvahxvpt.1](https://doi.org/10.55277/researchhub.fvahxvpt.1)

**BibTeX:**
```bibtex
@article{mukhtar2026hcsn,
  author  = {Saif Mukhtar},
  title   = {HCSN: A Hierarchical Causal Structure Network Framework for Emergent Physics},
  year    = {2026},
  doi     = {10.55277/researchhub.fvahxvpt.1},
  url     = {https://doi.org/10.55277/researchhub.fvahxvpt.1}
}
```

---

## License & Contact

Published under the **Apache 2.0** licence.

For collaboration or questions, open an issue or contact via GitHub: [hcsn-theory](https://github.com/hcsn-theory)

---

> *"The universe may not be described by computation — it may be computation."*
