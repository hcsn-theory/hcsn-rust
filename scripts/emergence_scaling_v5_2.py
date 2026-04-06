import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import os

def run_scaling_analysis(csv_path='exports/interaction_points_raw.csv'):
    if not os.path.exists(csv_path):
        print(f"Error: {csv_path} not found. Run the aggregator first.")
        return

    df = pd.read_csv(csv_path)
    if len(df) < 10:
        print("Error: Insufficient samples for scaling study.")
        return

    # 1. FIDELITY CALCULATIONS
    df['dPx'] = df['post_px'] - df['pre_px']
    df['dP_mag'] = df['dPx'].abs()
    df['P_pre_mag'] = df['pre_px'].abs().replace(0, 1e-6)
    df['eps_P'] = df['dP_mag'] / df['P_pre_mag']
    
    # 2. SELECTION METRICS
    # Conservation Event: eps_P < 0.2 (High Fidelity)
    df['is_conserved'] = (df['eps_P'] < 0.2).astype(int)

    # 3. BINNING BY STABILITY (S)
    # Use bins of width 2 for smooth scaling
    bins = np.arange(0, 52, 2)
    df['stab_bin'] = pd.cut(df['pre_s_mean'], bins=bins, labels=bins[:-1])
    
    # Calculate conditional stats per bin
    scaling_stats = df.groupby('stab_bin')['eps_P'].agg(['mean', 'std', 'count'])
    prob_stats = df.groupby('stab_bin')['is_conserved'].mean()
    
    # 4. PLOTTING THE PROOF
    plt.figure(figsize=(18, 7))
    
    # Left: Asymptotic Fidelity Convergence
    plt.subplot(1, 2, 1)
    plt.errorbar(scaling_stats.index.astype(float), scaling_stats['mean'], 
                 yerr=scaling_stats['std'] / np.sqrt(scaling_stats['count'].replace(0, 1)),
                 fmt='o-', color='blue', alpha=0.7, label='Mean Error (eps_P)')
    
    plt.axhline(0.2, color='red', linestyle='--', alpha=0.5, label='Conservation Threshold (0.2)')
    plt.title('HCSN v5.2: Asymptotic Fidelity Scaling\n(Natural Selection Phase)')
    plt.xlabel('Structural Stability (S)')
    plt.ylabel('Conservation Error (eps_P)')
    plt.yscale('log')
    plt.grid(True, which="both", alpha=0.3)
    plt.legend()
    
    # Right: Probability of Conservation
    plt.subplot(1, 2, 2)
    plt.plot(prob_stats.index.astype(float), prob_stats.values, 's-', color='green', label='P(eps_P < 0.2 | S)')
    
    plt.title('Emergent Law: Probability of Conservation\n(Crystallization of Symmetries)')
    plt.xlabel('Structural Stability (S)')
    plt.ylabel('P(Conservation)')
    plt.ylim(-0.05, 1.05)
    plt.grid(True, alpha=0.3)
    plt.legend()
    
    plt.tight_layout()
    plt.savefig('exports/emergence_scaling_v5_2.png')
    plt.close()

    # 5. FINAL VERDICT
    high_s_df = df[df['pre_s_mean'] > 30]
    law_fidelity = (high_s_df['eps_P'] < 0.2).mean() * 100 if not high_s_df.empty else 0
    
    print("\n========================================")
    print("HCSN v5.2: SCALING STUDY VERDICT")
    print("========================================")
    print(f"Total Samples:       {len(df)}")
    print(f"High-Stability (S>30): {len(high_s_df)} samples")
    print(f"Law Fidelity (S>30):   {law_fidelity:.2f}%")
    
    if law_fidelity > 85.0:
        print("Verdict:             🎯 EMERGENT LAW CONFIRMED")
    elif law_fidelity > 50.0:
        print("Verdict:             STOCHASTIC TRANSITION DETECTED")
    else:
        print("Verdict:             INSUFFICIENT SELECTION PRESSURE")
    print("========================================\n")

if __name__ == "__main__":
    run_scaling_analysis()
