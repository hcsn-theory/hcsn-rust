import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import os

def run_audit_v3(csv_path, output_tag):
    if not os.path.exists(csv_path):
        print(f"Error: {csv_path} not found.")
        return None

    df = pd.read_csv(csv_path)
    
    # 1. MOMENTUM CALCULATIONS
    df['dPx'] = df['post_px'] - df['pre_px']
    df['dPy'] = df['post_py'] - df['pre_py']
    df['dP_mag'] = np.sqrt(df['dPx']**2 + df['dPy']**2)
    df['P_pre_mag'] = np.sqrt(df['pre_px']**2 + df['pre_py']**2).replace(0, 1e-6)
    
    df['eps_P'] = df['dP_mag'] / df['P_pre_mag']
    
    # 2. ENERGY CALCULATIONS
    df['dE'] = df['post_E_total'] - df['pre_E_total']
    df['E_pre_total'] = df['pre_E_total'].replace(0, 1e-6)
    
    df['eps_E'] = df['dE'].abs() / df['E_pre_total'].abs()

    # Filter pathological outliers (knot death)
    df = df[(df['eps_P'] < 5.0) & (df['eps_E'] < 5.0)]

    # 3. METRICS
    mean_eps_p = df['eps_P'].mean()
    mean_eps_e = df['eps_E'].mean()
    eps_sys = np.sqrt(mean_eps_p**2 + mean_eps_e**2)
    
    global_drift_px = df['dPx'].sum()
    total_pre_px = df['pre_px'].abs().sum()
    global_drift_ratio = (abs(global_drift_px) / (total_pre_px + 1e-6)) * 100 # percentage

    # 4. PLOTTING
    plt.figure(figsize=(15, 5))
    
    # Subplot 1: P/E vs Stability
    s_scaling_P = df.groupby('stability_bin')['eps_P'].mean().sort_index()
    s_scaling_E = df.groupby('stability_bin')['eps_E'].mean().sort_index()
    
    plt.subplot(1, 2, 1)
    plt.plot(s_scaling_P.index, s_scaling_P.values, 'o-', color='blue', label='eps_P')
    plt.plot(s_scaling_E.index, s_scaling_E.values, 'o--', color='red', label='eps_E')
    plt.xlabel('Knots Stability (S)')
    plt.ylabel('Fidelity Error')
    plt.title(f'Hybrid S-Scaling Audit ({output_tag})')
    plt.grid(True, alpha=0.3)
    plt.legend()

    # Subplot 2: Sys Error Distribution
    df['eps_sys_local'] = np.sqrt(df['eps_P']**2 + df['eps_E']**2)
    plt.subplot(1, 2, 2)
    plt.hist(df['eps_sys_local'], bins=30, color='purple', edgecolor='black', alpha=0.7)
    plt.axvline(0.5, color='green', linestyle='--', label='Target (0.5)')
    plt.xlabel('Composite Sys Error (eps_sys)')
    plt.ylabel('Count')
    plt.title('Hybrid System Fidelity')
    plt.yscale('log')
    plt.legend()

    plt.tight_layout()
    plt.savefig(f'exports/audit_v3_{output_tag}.png')
    plt.close()

    print(f"\n========================================")
    print(f"HYBRID CONSERVATION REPORT: {output_tag}")
    print(f"========================================")
    print(f"Samples:         {len(df)}")
    print(f"Mean eps_P:      {mean_eps_p:.6f} {'[OK]' if mean_eps_p < 0.2 else '[FAIL]'}")
    print(f"Mean eps_E:      {mean_eps_e:.6f} {'[OK]' if mean_eps_e < 0.5 else '[FAIL]'}")
    print(f"Composite eps_sys:{eps_sys:.6f}")
    print(f"Global Drift Px: {global_drift_px:.4f} (Ratio: {global_drift_ratio:.4f}%)")
    print(f"========================================\n")

    return {
        "eps_P": mean_eps_p,
        "eps_E": mean_eps_e,
        "eps_sys": eps_sys,
        "drift": global_drift_px
    }

if __name__ == "__main__":
    mode = os.environ.get("HCSN_CONSERVATION_MODE", "Hybrid")
    run_audit_v3("exports/conservation_raw.csv", mode)
