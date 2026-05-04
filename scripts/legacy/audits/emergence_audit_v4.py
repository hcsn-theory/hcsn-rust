import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import os

def run_emergence_audit(csv_path, output_tag):
    if not os.path.exists(csv_path):
        print(f"Error: {csv_path} not found.")
        return None

    df = pd.read_csv(csv_path)
    
    # 1. PHYSICAL DIMENSIONS
    df['dPx'] = df['post_px'] - df['pre_px']
    df['dP_mag'] = df['dPx'].abs()
    df['P_pre_mag'] = df['pre_px'].abs().replace(0, 1e-6)
    
    df['eps_P'] = df['dP_mag'] / df['P_pre_mag']
    
    # 2. ENERGY DIMENSIONS
    df['dE'] = df['post_E_total'] - df['pre_E_total']
    df['E_pre_total'] = df['pre_E_total'].abs().replace(0, 1e-6)
    df['eps_E'] = df['dE'].abs() / df['E_pre_total']

    # Filter pathological outliers (interactions during rewrite failures)
    df = df[(df['eps_P'] < 5.0) & (df['eps_E'] < 5.0)]

    # 3. BINNING BY STABILITY (S)
    # Target: Show that epsilon decreases as S increases
    bins = np.linspace(0, 30, 11) # 0 to 30 in 11 bins (step 3)
    df['stab_bin'] = pd.cut(df['pre_s_mean'], bins=bins, labels=bins[:-1])
    
    s_stats = df.groupby('stab_bin').agg({
        'eps_P': ['mean', 'std', 'count'],
        'eps_E': ['mean', 'std']
    }).dropna()

    # 4. PLOTTING THE EMERGENCE CURVE
    plt.figure(figsize=(15, 6))
    
    # Subplot 1: Emergence Curve (eps_P vs S)
    plt.subplot(1, 2, 1)
    plt.errorbar(s_stats.index.astype(float), s_stats[('eps_P', 'mean')], 
                 yerr=s_stats[('eps_P', 'std')] / np.sqrt(s_stats[('eps_P', 'count')]), 
                 fmt='o-', color='blue', label='eps_P (Momentum)')
    plt.errorbar(s_stats.index.astype(float), s_stats[('eps_E', 'mean')], 
                 yerr=s_stats[('eps_E', 'std')] / np.sqrt(s_stats[('eps_P', 'count')]), 
                 fmt='s--', color='green', label='eps_E (Energy)')
    
    plt.axhline(0.2, color='red', linestyle=':', label='High-Fidelity Threshold (0.2)')
    plt.xlabel('Topological Stability (S)')
    plt.ylabel('Conservation Error (eps)')
    plt.title(f'HCSN v4.0: Emergence of Conservation Laws\n(eps -> 0 as S -> high)')
    plt.grid(True, alpha=0.3)
    plt.legend()

    # Subplot 2: Sample Hub
    plt.subplot(1, 2, 2)
    plt.bar(s_stats.index.astype(float), s_stats[('eps_P', 'count')], color='gray', alpha=0.5)
    plt.xlabel('Topological Stability (S)')
    plt.ylabel('Interaction Count (N)')
    plt.title('Interaction Density vs. Stability')
    plt.grid(axis='y', alpha=0.3)

    plt.tight_layout()
    plt.savefig(f'exports/emergence_audit_v4_{output_tag}.png')
    plt.close()

    # 5. REPORTING
    mean_eps_p_high_s = s_stats[s_stats.index.astype(float) > 15][('eps_P', 'mean')].mean()
    mean_eps_p_low_s  = s_stats[s_stats.index.astype(float) <= 15][('eps_P', 'mean')].mean()

    print(f"\n========================================")
    print(f"HCSN v4.0 EMERGENCE REPORT: {output_tag}")
    print(f"========================================")
    print(f"Total Samples:       {len(df)}")
    print(f"Low-Stability Error:  {mean_eps_p_low_s:.6f}")
    print(f"High-Stability Error: {mean_eps_p_high_s:.6f}")
    
    if mean_eps_p_high_s < mean_eps_p_low_s:
        improvement = (1 - mean_eps_p_high_s / mean_eps_p_low_s) * 100
        print(f"Emergence Verdict:    POSITIVE (+{improvement:.1f}% Fidelity Improvement)")
    else:
        print(f"Emergence Verdict:    NEGATIVE (No scaling observed)")
    
    print(f"========================================\n")

if __name__ == "__main__":
    mode = os.environ.get("HCSN_CONSERVATION_MODE", "Hybrid")
    run_emergence_audit("exports/conservation_raw.csv", mode)
