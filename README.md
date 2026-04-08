# 🦀 HCSN Rust — Hierarchical Causal Structure Network

[![DOI](https://img.shields.io/badge/DOI-10.55277%2Fresearchhub.fvahxvpt.1-blue)](https://doi.org/10.55277/researchhub.fvahxvpt.1)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)

---

> **HCSN** explores the hypothesis that the universe is fundamentally computational — 
> built from discrete events and directed causal relations.

## ✨ Latest State: Phase 12 (Interaction Theory)

This repository has successfully concluded the **Matter Emergence & Interaction** campaign. We have transitioned from stochastic graph-rewriting to a predictive physical theory of emergent topological matter.

### 🔬 Key Discoveries
*   **Topological Force Law**: Interaction is **threshold-activated**. Impulse ($\Delta p$) only transmits when structural overlap $\chi > 0.14$.
*   **Critical Stability**: Targeted at $p=0.64$, $\gamma=2.2$ to achieve a stable power-law lifetime distribution ($\alpha \approx 1.7-2.0$).
*   **Scattering Geometry**: Interaction is non-isotropic, with a mean deflection angle of **$71.5^\circ$**.

---

## 📋 Table of Contents
- [Repository Structure](#repository-structure)
- [Quick Start](#quick-start)
- [Analysis Suite](#analysis-suite)
- [Production Pipeline](#production-pipeline)

---

## Repository Structure
```text
hcsn-rust/
├── src/
│   ├── rewrite_engine.rs       # Core engine with SIGINT safe-exit
│   ├── hypergraph.rs           # Causal memory & graph structures
│   └── bin/run_simulation.rs   # Main production binary
├── research_docs/              # Phase 1-12 Plans & Task Trackers
├── exports/                    # Seed-specific worldline & interaction JSONs
├── run_production.py           # Multi-seed orchestration (Seeds 1-5)
├── analyze_force_law.py        # Functional fit: Δp = f(χ)
└── analyze_lifetimes.py        # Ensemble hazard rate & P(τ) analysis
```

---

## Quick Start

### 1. Build
```bash
cargo build --release
```

### 2. Run Production Ensemble (Seeds 1-5)
```bash
python3 run_production.py
```

### 3. Analyze Interactions (Force Law)
```bash
python3 analyze_force_law.py
```

### 4. Analyze Lifetimes (Stability)
```bash
python3 analyze_lifetimes.py
```

---

## 📈 Interaction Phenomenology
The system confirms that "Matter" is a persistent topological knot that interacts via a discrete, depth-dependent force. 

| Feature | Value | Physical Outcome |
|:---|:---|:---|
| Threshold ($\chi_c$) | $0.14$ | Topological "Gap" protection |
| Coupling ($k$) | $182.1$ | Interaction strength |
| Deflection ($\theta$) | $71.5^\circ$ | High-energy scattering bias |

---

## 📜 License
Published under the **MIT License**. See `LICENSE` for details.
© 2026 HCSN Theory Group (Saif Mukhtar)
