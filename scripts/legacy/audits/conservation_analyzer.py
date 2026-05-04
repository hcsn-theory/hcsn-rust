import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import os

def analyze_conservation(path="exports/conservation_raw.csv"):
    if not os.path.exists(path):
        print(f"Error: {path} not found.")
        return

    df = pd.read_csv(path)
    
    # --- FILTERING RULES ---
    # duration >= 3, |pre_p| >= 0.01
    initial_n = len(df)
    df = df[df['duration'] >= 3]
    df = df[df['pre_p_mag'] >= 0.01]
    filtered_n = len(df)
    
    print(f"=== HCSN CONSERVATION DIAGNOSTICS ===")
    print(f"Samples: {filtered_n} (Filtered from {initial_n})")
    print("-" * 40)

    results = {}

    # --- 1. VECTOR MOMENTUM ---
    df['dx'] = df['post_px'] - df['pre_px']
    df['dy'] = df['post_py'] - df['pre_py']
    df['dp_vec'] = np.sqrt(df['dx']**2 + df['dy']**2)
    pre_p_vec_mag = np.sqrt(df['pre_px']**2 + df['pre_py']**2)
    df['eps_vec'] = df['dp_vec'] / (pre_p_vec_mag + 1e-6)

    results['Vector Momentum'] = {
        'mean_abs_delta': df['dp_vec'].mean(),
        'std_abs_delta': df['dp_vec'].std(),
        'mean_dx': df['dx'].mean(),
        'mean_dy': df['dy'].mean(),
        'eps_mean': df['eps_vec'].mean()
    }

    # --- 2. SCALAR MOMENTUM ---
    df['dp_scalar'] = (df['post_p_mag'] - df['pre_p_mag'])
    df['eps_scalar'] = np.abs(df['dp_scalar']) / (np.abs(df['pre_p_mag']) + 1e-6)
    
    results['Scalar Momentum'] = {
        'mean_abs_delta': np.abs(df['dp_scalar']).mean(),
        'std_abs_delta': df['dp_scalar'].std(),
        'eps_mean': df['eps_scalar'].mean()
    }

    # --- 3. MASS PROXY ---
    df['dm'] = df['post_mass'] - df['pre_mass']
    df['eps_mass'] = np.abs(df['dm']) / (np.abs(df['pre_mass']) + 1e-6)
    
    results['Mass Proxy'] = {
        'mean_abs_delta': np.abs(df['dm']).mean(),
        'std_abs_delta': df['dm'].std(),
        'eps_mean': df['eps_mass'].mean()
    }

    # --- 4. STABILITY SUM ---
    df['ds_sum'] = df['post_s_sum'] - df['pre_s_sum']
    df['eps_s_sum'] = np.abs(df['ds_sum']) / (np.abs(df['pre_s_sum']) + 1e-6)
    
    results['Stability Sum'] = {
        'mean_abs_delta': np.abs(df['ds_sum']).mean(),
        'std_abs_delta': df['ds_sum'].std(),
        'eps_mean': df['eps_s_sum'].mean()
    }

    # --- OUTPUT & CLASSIFICATION ---
    for qty, stats in results.items():
        print(f"\n=== {qty.upper()} ===")
        print(f"mean(|Δ|):   {stats['mean_abs_delta']:.6f}")
        print(f"std(|Δ|):    {stats['std_abs_delta']:.6f}")
        if 'mean_dx' in stats:
            print(f"mean(ΔPx):   {stats['mean_dx']:.6f}")
            print(f"mean(ΔPy):   {stats['mean_dy']:.6f}")
        print(f"ε_mean:      {stats['eps_mean']:.6f}")
        
        # Classification
        eps = stats['eps_mean']
        if eps < 0.1:
            verdict = "STRONG CONSERVATION"
        elif eps < 0.5:
            verdict = "WEAK CONSERVATION"
        else:
            verdict = "NO CONSERVATION"
        print(f"Verdict:     {verdict}")

    # --- SPECIAL DETECTIONS ---
    print("\n=== SPECIAL DETECTIONS ===")
    
    # 1. Statistical Conservation
    if abs(results['Vector Momentum']['mean_dx']) < 0.005 and abs(results['Vector Momentum']['mean_dy']) < 0.005:
        print(">>> STATISTICAL CONSERVATION DETECTED")
    
    # 2. Emergent Conservation (Stability Bins)
    stab_groups = df.groupby('stability_bin')['eps_vec'].mean()
    if len(stab_groups) > 1:
        # Check if high stability has lower eps than low stability
        low_stab_eps = stab_groups.iloc[0]
        high_stab_eps = stab_groups.iloc[-1]
        if high_stab_eps < low_stab_eps * 0.7:
            print(f">>> EMERGENT CONSERVATION (HIGH-STABILITY) [Low S: {low_stab_eps:.3f} -> High S: {high_stab_eps:.3f}]")

    # 3. Global Drift
    # Cumulative sum of momentum changes
    cum_px_drift = df['dx'].sum()
    if abs(cum_px_drift) > 1.0:
        print(f">>> GLOBAL DRIFT DETECTED (Total Px shift: {cum_px_drift:.4f})")

    # --- VISUALIZATION ---
    fig, axes = plt.subplots(1, 3, figsize=(18, 5))
    
    # Histogram ΔP
    axes[0].hist(df['dp_vec'], bins=50, color='skyblue', edgecolor='black')
    axes[0].set_title("Distribution of |ΔP| (Vector)")
    axes[0].set_xlabel("|ΔP|")
    
    # Histogram ΔM
    axes[1].hist(df['dm'], bins=50, color='salmon', edgecolor='black')
    axes[1].set_title("Distribution of ΔM (Mass Proxy)")
    axes[1].set_xlabel("ΔM")
    
    # Stability vs Error
    axes[2].plot(stab_groups.index, stab_groups.values, marker='o', linestyle='-', color='green')
    axes[2].set_title("Conservation Error vs Stability")
    axes[2].set_xlabel("Stability Bin")
    axes[2].set_ylabel("ε_mean")

    plt.tight_layout()
    plt.savefig("exports/conservation_diagnostics.png")
    print(f"\nVisual diagnostics saved to exports/conservation_diagnostics.png")

if __name__ == "__main__":
    analyze_conservation()
