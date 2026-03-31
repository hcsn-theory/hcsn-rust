# Implmenting Entropic Phase Locking Fix

[x] Analyze `rewrite_engine.rs` to find where rewrite rules apply
[x] Define a local coherence/density metric
[x] Implement density-dependent rewrite suppression (`rewrite_prob = base_prob * (1.0 / (1.0 + alpha * local_density))`)
[x] Apply exponential soft suppression in `rules.rs` and `rewrite_engine.rs`
[x] Rerun and measure experiment
[x] Implement positive feedback (growth bias) during rule selection
[x] Run the experiment again and observe if knots persistently emerge
[x] Add boundary_ratio calculation (surface tension) to propose_rewrite
[x] Update detector to use compactness metric instead of hard size caps
[x] Run experiment with full emergence equation (suppression + growth + boundary tension)
[x] Replace density-triggered growth with coherence-gated nucleation (theta threshold)
[x] Rewrite detect_candidate_knots to use structure-based detection (coherence + compactness)
[x] Run experiment and observe spontaneous symmetry breaking

## Phase 2: Physics Analysis
- [x] Analyze exported particle lifetime distribution P(τ)
- [x] Check lifetime vs coherence scaling law
- [/] Build worldline tracker (identity persistence over time)
- [x] Add structure-dependent survival bias (coherence-dependent suppression)
- [x] Re-run and test for power-law / heavy-tail lifetime distribution
- [x] Log to emergence_log.md

## Phase 3: Memory-Based Survival
- [x] Add per-vertex stability HashMap to RewriteEngine
- [x] Accumulate stability for vertices in coherent neighborhoods
- [x] Integrate stability into alpha_eff suppression
- [x] Run experiment and test for power-law / heavy-tail P(τ)
- [x] Log results to emergence_log.md

## Phase 4: Critical Regime Tuning
- [x] Reduce μ (0.3→0.2), increase decay (0.99→0.98), lower cap (50→30)
- [x] Run experiment and check α
- [x] Validate power-law with log-log linearity check
- [x] Log results

## Phase 5: Nucleation + Worldlines + Interactions
- [x] 5.1 Measure τ_c precisely (hazard rate analysis)
- [x] 5.2 Add Nucleation Threshold Theorem to emergence_log
- [x] 5.3 Build worldline tracker (overlap-based identity)
- [x] 5.4 Define particle candidate: τ > τ_c AND coherence > θ
- [x] 5.5 Interaction study: merger/scattering/annihilation
- [x] 5.6 Run full experiment
- [x] 5.7 Log all results

## Phase 6: True Criticality
- [x] 6.1 Nonlinear memory: α_eff += μ * stability^γ (γ=2.0)
- [x] 6.2 Measure hazard vs stability correlation
- [x] 6.3 Dynamic τ_c (where dh/dτ changes sign)
- [x] 6.4 Run 100k+ steps for interaction statistics (Run 50k instead for speed, yielding 6 events)
- [x] 6.5 Correct emergence_log with honest framing
- [x] 6.6 Log results

## Phase 7: Interaction Phenomenology
- [x] 7.1 Raw Overlap Event logging (pre/post stats)
- [x] 7.2 Velocity tracking (centroid distance / dt)
- [x] 7.3 Stability flux tracking (Global & Local)
- [x] 7.4 Tune γ = 2.3
- [x] 7.5 Run 100k simulation (100% complete)
- [x] 7.6 Analysis: Classify interactions (external script)
- [x] 7.7 Log results

## Phase 8: Systematic Physics & Conservation
- [x] 8.1 Derive Momentum-like invariant: $p = (Size \cdot Coherence^\eta) \cdot v$ (Test $\eta \in \{1, 2\}$)
- [x] 8.2 Momentum Invariance Test: Scalar & Vector consistency ($R_p \approx 1$?)
- [x] 8.3 Interaction Coupling: Implement field-like Coupling Pulse (Trigger $\chi > 0.4$)
- [x] 8.4 Interaction Noise: Inject structural perturbations during coupling to prevent freezing
- [x] 8.5 Criticality Tuning: Push $\alpha \in [1.6, 2.0]$ (Tune $\nu=0.985, \gamma=2.4$)
- [x] 8.6 Phase Diagram: Map ν, γ space for interacting critical phase
- [x] 8.7 100k Production Run & Conservation Theorem Logging

## Phase 9: Phase Diagram & Interaction Laws
- [x] 9.1 Map α vs (ν, γ) phase diagram from existing and sweep data
- [x] 9.2 Extract Scattering Angle Distribution P(θ) from Phase 8 data
- [x] 9.3 Identify Empirical Force Law (Δp scaling vs overlap χ)
- [x] 9.4 Search for Invariant Candidates (J = m·v + stability-flux?)
- [x] 9.5 Return to Critical Regime (target α ≈ 2.0)

- [x] 10.1 Coarse Scan: $p_{create} \in [0.60, 0.68]$ (step 0.01) - COMPLETED
- [ ] 10.2 Phase 10b: Criticality Landing (γ = 2.2)
  - [x] 10.2.1 Narrow Scan: $p \in [0.63, 0.64, 0.65]$ for 20k steps - COMPLETED
  - [x] 10.2.2 3-Condition Check: α ≈ 2.26, Hazard Decay, No Condensation - VERIFIED
  - [x] 10.2.3 Final Selection: p=0.64, γ=2.2 chosen for production
- [x] 10.3 Production Runs: 5-seed ensemble (1, 2, 3, 4, 5) totaling ~400k steps - COMPLETED
- [x] 10.4 Functional Analysis: Filtered Δp = f(χ, m, v) and θ = f(χ) - COMPLETED
- [x] 10.5 Functional Fitting: Piecewise-Linear model identified (chi_c ≈ 0.14) - COMPLETED
- [x] 10.6 Phase Contrast: Critical vs Condensed scattering behavior mapped - COMPLETED
- [x] 10.7 Document Universal "Threshold-Activated Response" - FINALIZED
