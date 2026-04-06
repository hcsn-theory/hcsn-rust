import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import os

def run_audit(csv_path, output_tag):
    if not os.path.exists(csv_path):
        print(f"Error: {csv_path} not found.")
        return None

    df = pd.read_csv(csv_path)
    
    # Differential analysis
    df['dPx'] = df['post_px'] - df['pre_px']
    df['dPy'] = df['post_py'] - df['pre_py']
    df['dP_mag'] = np.sqrt(df['dPx']**2 + df['dPy']**2)
    df['P_total_pre'] = np.sqrt(df['pre_px']**2 + df['pre_py']**2)
    
    # local epsilon: |Delta P| / |P_total_pre|
    # Avoid div by zero
    df['eps_P_local'] = df['dP_mag'] / df['P_total_pre'].replace(0, 1e-6)
    
    # Filter pathological cases (outliers > 1000% error often represent knot death)
    df = df[df['eps_P_local'] < 10.0]

    # 1. LOCAL INTEGRITY
    mean_eps_local = df['eps_P_local'].mean()
    
    # 2. GLOBAL INTEGRITY (Drift)
    # Total sum of deltas / sum of pre-momenta
    global_drift = df['dPx'].sum()
    total_pre = df['pre_px'].abs().sum()
    eps_P_global = abs(global_drift) / (total_pre + 1e-6)

    # 3. MASS INTEGRITY
    df['dM'] = df['post_mass'] - df['pre_mass']
    eps_M = (df['dM'].abs() / df['pre_mass'].replace(0, 1e-6)).mean()

    # 4. STABILITY SCALING
    # Group by stability_bin and find mean eps_P_local
    s_scaling = df.groupby('stability_bin')['eps_P_local'].mean().sort_index()

    # 5. PLOTTING
    plt.figure(figsize=(12, 5))
    
    # Subplot 1: S-Scaling Curve
    plt.subplot(1, 2, 1)
    plt.plot(s_scaling.index, s_scaling.values, 'o-', color='blue', label='Eps_P vs S')
    plt.xlabel('Knots Stability (S)')
    plt.ylabel('Mean Momentum Error (eps_P)')
    plt.title(f'Stability Cooling Effect ({output_tag})')
    plt.grid(True, alpha=0.3)
    plt.legend()

    # Subplot 2: Local Error Distribution
    plt.subplot(1, 2, 2)
    plt.hist(df['eps_P_local'], bins=30, color='skyblue', edgecolor='black', alpha=0.7)
    plt.axvline(1.0, color='red', linestyle='--', label='Milestone 1 (eps=1.0)')
    plt.xlabel('Local Momentum Error (eps_P)')
    plt.ylabel('Count')
    plt.title('Interaction Fidelity Distribution')
    plt.yscale('log')
    plt.legend()

    plt.tight_layout()
    plt.savefig(f'exports/audit_v2_{output_tag}.png')
    plt.close()

    return {
        "mode": output_tag,
        "n_samples": len(df),
        "eps_P_local": mean_eps_local,
        "eps_P_global": eps_P_global,
        "eps_M": eps_M,
        "global_drift_px": global_drift
    }

if __name__ == "__main__":
    mode = os.environ.get("HCSN_CONSERVATION_MODE", "Baseline")
    results = run_audit("exports/conservation_raw.csv", mode)
    if results:
        print(f"\n=== AUDIT RESULTS: {mode} ===")
        print(f"Samples:      {results['n_samples']}")
        print(f"Eps_P_local:  {results['eps_P_local']:.6f}")
        print(f"Eps_P_global: {results['eps_P_global']:.6f}")
        print(f"Eps_M:        {results['eps_M']:.6f}")
        print(f"Global Drift: {results['global_drift_px']:.4f}")
