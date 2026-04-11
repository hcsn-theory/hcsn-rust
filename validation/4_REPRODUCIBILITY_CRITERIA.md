# Reproducibility Success Criteria (Phase 13)

This document formalizes the requirements for the **1M-Step Reproducibility Campaign** to ensure scientific rigor and minimize observer bias.

## Interpretation Framework
Instead of claiming "Fundamental Quantization," we adopt a defensible, empirical framework:
> *"Testing whether structural persistence systematically correlates with effective interaction strength under controlled stochastic variation."*

## 1. Strong Reproducibility Thresholds
We define "High-Confidence" findings if the following are met in the 5 experimental seeds (42–46):

- **Primary Variable Correlation**: Stability ($S$) must explain **>70% of variance** ($R^2 > 0.70$) in at least **4 out of 5** seeds.
- **Scattering Stability**: The mean scattering angle must stay within **$\pm 10\%$** of the v12.1 baseline (71.5°).
- **Branch Capture**: Qualitative two-branch structure (Pass-Through vs. Reflection) must be identifiable in at least **3 out of 5** seeds.

## 2. Significance Benchmarking (Null Control)
The campaign includes an isolated Null-Control run (**Seed 47**) with `MU=0` and `NU=0`.
- **Success Condition**: The Stability $R^2$ must show **Regression Collapse** ($R^2 < 0.20$), indicating that the observed correlation is unique to the Structural Persistence mechanism and not artifacts of graph geometry.

## 3. Resource & Growth Constraints
- **RAM Threshold**: Memory usage must be monitored. Usage > 92% triggers immediate archival.
- **Vertex Density**: Growth must be roughly linear ($dV/dt \approx \text{const}$) across all 5 seeds. Significant outliers must be flagged as "Nucleation Anomaly."
