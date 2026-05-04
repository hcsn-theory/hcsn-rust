import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import os

def run_emergence_audit_v5(csv_path, output_tag):
    if not os.path.exists(csv_path):
        print(f"Error: {csv_path} not found.")
        return None

    df = pd.read_csv(csv_path)
    
    # 1. PHYSICAL DIMENSIONS
    df['dPx'] = df['post_px'] - df['pre_px']
    df['dP_mag'] = df['dPx'].abs()
    df['P_pre_mag'] = df['pre_px'].abs().replace(0, 1e-6)
    df['eps_P'] = df['dP_mag'] / df['P_pre_mag']
    
    df['dE'] = df['post_E_total'] - df['pre_E_total']
    df['E_pre_total'] = df['pre_E_total'].abs().replace(0, 1e-6)
    df['eps_E'] = df['dE'].abs() / df['E_pre_total']

    # Filter extreme pathological outliers (interactions during rewrite failures)
    df = df[(df['eps_P'] < 10.0) & (df['eps_E'] < 10.0)]

    # 2. DEFINING REGIMES
    # Baseline Noise: S < 5
    # Transition:    5 <= S < 15
    # Emergent:      S >= 15
    conditions = [
        (df['pre_s_mean'] < 5.0),
        (df['pre_s_mean'] >= 5.0) & (df['pre_s_mean'] < 15.0),
        (df['pre_s_mean'] >= 15.0)
    ]
    regimes = ['Baseline Noise', 'Transition', 'Emergent Physics']
    df['regime'] = np.select(conditions, regimes, default='Other')
    
    # 3. BINNING BY STABILITY (S)
    bins = np.linspace(0, 50, 11) # 0 to 50 in 11 bins (step 5)
    df['stab_bin'] = pd.cut(df['pre_s_mean'], bins=bins, labels=bins[:-1])
    
    s_stats = df.groupby('stab_bin').agg({
        'eps_P': ['mean', 'std', 'count'],
        'eps_E': ['mean', 'std']
    }).dropna()

    # 4. PLOTTING THE PHASE DIAGRAM
    plt.figure(figsize=(18, 7))
    
    # Subplot 1: Emergence Curve (eps_P vs S)
    plt.subplot(1, 2, 1)
    
    # Shade Regimes
    plt.axvspan(0, 5, color='red', alpha=0.1, label='Regime 1: Noise')
    plt.axvspan(5, 15, color='orange', alpha=0.1, label='Regime 2: Transition')
    plt.axvspan(15, 50, color='green', alpha=0.1, label='Regime 3: Emergent Physics')
    
    plt.errorbar(s_stats.index.astype(float), s_stats[('eps_P', 'mean')], 
                 yerr=s_stats[('eps_P', 'std')] / np.sqrt(s_stats[('eps_P', 'count')].replace(0, 1)), 
                 fmt='o-', color='blue', label='eps_P (Momentum)')
    
    plt.axhline(0.2, color='black', linestyle=':', alpha=0.5, label='Fidelity Target (0.2)')
    plt.xlabel('Topological Stability (S)')
    plt.ylabel('Conservation Error (eps)')
    plt.title(f'HCSN v5.0: Conservation Phase Diagram\n(Transition Nucleation Threshold S = 15)')
    plt.yscale('log')
    plt.grid(True, which="both", alpha=0.3)
    plt.legend()

    # Subplot 2: Statistical Verification
    plt.subplot(1, 2, 2)
    regime_stats = df.groupby('regime')['eps_P'].mean().reindex(regimes)
    colors = ['red', 'orange', 'green']
    plt.bar(regimes, regime_stats, color=colors, alpha=0.6)
    plt.ylabel('Mean eps_P (Log Scale)')
    plt.yscale('log')
    plt.title('Emergence Proof: Fidelity Improvement by Regime')
    
    # Add labels to bars
    for i, v in enumerate(regime_stats):
        plt.text(i, v, f"{v:.4f}", ha='center', va='bottom', fontweight='bold')

    plt.tight_layout()
    plt.savefig(f'exports/emergence_phase_diagram_v5_{output_tag}.png')
    plt.close()

    # 5. FINAL REPORTING
    print(f"\n========================================")
    print(f"HCSN v5.0 PHASE AUDIT: {output_tag}")
    print(f"========================================")
    print(f"Total Interactions:  {len(df)}")
    
    for r in regimes:
        reg_df = df[df['regime'] == r]
        if not reg_df.empty:
            print(f"{r: <16}: eps_P = {reg_df['eps_P'].mean():.6f} (N={len(reg_df)})")
        else:
            print(f"{r: <16}: No samples found.")
            
    # Emergence Verdict
    physics_df = df[df['regime'] == 'Emergent Physics']
    noise_df = df[df['regime'] == 'Baseline Noise']
    
    if not physics_df.empty and not noise_df.empty:
        q = physics_df['eps_P'].mean() / noise_df['eps_P'].mean()
        reduction = (1 - q) * 100
        print(f"----------------------------------------")
        print(f"Fidelity Gain:       +{reduction:.1f}%")
        if reduction > 50:
            print(f"Verdict:             🎯 PHASE TRANSITION CONFIRMED")
        else:
            print(f"Verdict:             Weak Emergence")
    else:
        print(f"Verdict:             INCONCLUSIVE (Missing Regimes)")
    print(f"========================================\n")

if __name__ == "__main__":
    mode = os.environ.get("HCSN_CONSERVATION_MODE", "Hybrid")
    run_emergence_audit_v5("exports/conservation_raw.csv", mode)
