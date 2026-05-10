import json
import os
import sys
import numpy as np

def analyze_folder(path):
    # Find files
    event_files = [f for f in os.listdir(path) if f.startswith("interaction_events")]
    lifetime_files = [f for f in os.listdir(path) if f.startswith("particle_lifetimes")]
    
    if not event_files:
        return None
    
    with open(os.path.join(path, event_files[0]), 'r') as f:
        events = json.load(f)
        
    with open(os.path.join(path, lifetime_files[0]), 'r') as f:
        lifetimes = json.load(f)
        
    # 1. Relational Velocity Stats
    v_rels = [e['v_rel_smoothed'] for e in events]
    v_mean = np.mean(v_rels) if v_rels else 0
    v_std = np.std(v_rels) if v_rels else 0
    
    # 2. Momentum Conservation (Diagnostic Proxy)
    # pre_px is pre_a[3] + pre_b[3] (Wait, index 3 is diagnostic_v_abs)
    # Momentum is index 2 in our tuple: [m, v_rel, p_rel, diag_v_abs, ...]
    # Wait, in the code:
    # pre_a: (m_a, 0.0, 0.0, knot_a.diagnostic_v_abs, ...)
    # So we use diagnostic_v_abs * mass for the conservation check
    
    pre_ps = []
    post_ps = []
    
    for e in events:
        if e['post_a'] and e['post_b']:
            # Total Momentum = m_a * v_a + m_b * v_b
            # Using diagnostic_v_abs (index 3)
            p_pre = e['pre_a'][0] * e['pre_a'][3] + e['pre_b'][0] * e['pre_b'][3]
            p_post = e['post_a'][0] * e['post_a'][3] + e['post_b'][0] * e['post_b'][3]
            pre_ps.append(p_pre)
            post_ps.append(p_post)
            
    correlation = np.corrcoef(pre_ps, post_ps)[0, 1] if len(pre_ps) > 1 else 0
    
    # 3. Stability
    avg_lifetime = np.mean([l['age'] for l in lifetimes]) if lifetimes else 0
    particle_candidates = len([l for l in lifetimes if l.get('particle_candidate', False)])
    
    return {
        "v_mean": v_mean,
        "v_std": v_std,
        "p_corr": correlation,
        "avg_lifetime": avg_lifetime,
        "particle_candidates": particle_candidates,
        "event_count": len(events)
    }

def main():
    if len(sys.argv) < 2:
        print("Usage: python analyze_results.py <experiment_dir>")
        return

    exp_dir = sys.argv[1]
    baseline = analyze_folder(os.path.join(exp_dir, "baseline"))
    hybrid = analyze_folder(os.path.join(exp_dir, "hybrid"))
    
    report = f"""# HCSN Large-Scale Experiment Report
Generated from: {exp_dir}

## 1. Executive Summary
This experiment compared two $10^5$ step universes to evaluate the impact of Relational Relief and Conservation Patches on topological matter emergence.

| Metric | Baseline (Pure) | Hybrid (Patched) | Improvement |
|--------|----------------|------------------|-------------|
| Total Interactions | {baseline['event_count']} | {hybrid['event_count']} | {((hybrid['event_count']/baseline['event_count'])-1)*100:.1f}% |
| Avg Knot Lifetime | {baseline['avg_lifetime']:.1f} | {hybrid['avg_lifetime']:.1f} | {((hybrid['avg_lifetime']/baseline['avg_lifetime'])-1)*100:.1f}% |
| Particle Candidates | {baseline['particle_candidates']} | {hybrid['particle_candidates']} | {((hybrid['particle_candidates']/(baseline['particle_candidates'] or 1))-1)*100:.1f}% |
| Momentum Correlation (ρ) | {baseline['p_corr']:.4f} | {hybrid['p_corr']:.4f} | {hybrid['p_corr'] - baseline['p_corr']:.4f} |

## 2. Kinematic Analysis
### Relational Velocity ($v_{{rel}}$)
*   **Baseline**: $\mu = {baseline['v_mean']:.2e}$, $\sigma = {baseline['v_std']:.2e}$
*   **Hybrid**: $\mu = {hybrid['v_mean']:.2e}$, $\sigma = {hybrid['v_std']:.2e}$

### Diagnostic Momentum Conservation
The correlation $\rho$ measures how well the structural momentum is preserved across interaction events. A higher $\rho$ in the Hybrid universe indicates that the **Asymptotic Symmetry Ramp** and **Local Flux Compensation** are successfully stabilizing the topological dynamics.

## 3. Conclusion
The Hybrid universe shows **{"more" if hybrid['particle_candidates'] > baseline['particle_candidates'] else "less"}** robust particle emergence and **{"better" if hybrid['p_corr'] > baseline['p_corr'] else "worse"}** conservation of structural diagnostics. 

---
*End of Report*
"""
    
    with open(os.path.join(exp_dir, "EXPERIMENT_REPORT.md"), "w") as f:
        f.write(report)
    
    print(f"Report generated successfully in {exp_dir}")

if __name__ == "__main__":
    main()
