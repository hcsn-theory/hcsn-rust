import csv, json, os, sys, time
import numpy as np
import pandas as pd
from pathlib import Path
from scipy.ndimage import gaussian_filter

def analyze_phase_space(df, epsilon=1e-6, min_n=50):
    """
    Main Research Logic: Maps R2 and Signal Gain across (Age, Stability) space.
    """
    # 1. Normalized Momentum Change (Equation of State)
    df['dp_norm'] = (df['post_p_mag'] - df['pre_p_mag']).abs() / (df['pre_p_mag'] + epsilon)
    df['age_min'] = df[['pre_age_a', 'pre_age_b']].min(axis=1)
    
    # 2. Adaptive Percentile Binning (v4.1 Guaranteed Density)
    # We use 5 bins (quintiles) to ensure N >= 50 even in sparse regimes
    df['age_bin_idx'] = pd.qcut(df['age_min'], 5, labels=False, duplicates='drop')
    df['stab_bin_idx'] = pd.qcut(df['pre_s_mean'], 5, labels=False, duplicates='drop')
    
    # Grid initialization (5x5 Master Map)
    grid_r2 = np.full((5, 5), np.nan)
    grid_n = np.zeros((5, 5))
    
    for i in range(5):
        for j in range(5):
            subset = df[(df['age_bin_idx'] == i) & (df['stab_bin_idx'] == j)]
            grid_n[i, j] = len(subset)
            
            if len(subset) >= min_n:
                r = subset['pre_s_mean'].corr(subset['dp_norm'])
                grid_r2[i, j] = r**2 if not np.isnan(r) else 0.0
                
    # 3. NaN-Safe Smoothing
    # We only smooth non-NaN regions to avoid bias leakage
    mask = ~np.isnan(grid_r2)
    smoothed_r2 = np.copy(grid_r2)
    smoothed_r2[~mask] = 0
    smoothed_r2 = gaussian_filter(smoothed_r2, sigma=1.0)
    smoothed_r2[~mask] = np.nan
    
    return {
        "grid_r2": grid_r2,
        "smoothed_r2": smoothed_r2,
        "grid_n": grid_n,
    }

def causal_hierarchy_report(df, epsilon=1e-6):
    """
    Tests: Does Age -> Stability? Or Independent?
    """
    from sklearn.linear_model import LinearRegression
    
    clean_df = df.dropna(subset=['dp_norm', 'pre_s_mean', 'age_min'])
    if len(clean_df) < 100: return "Insufficient data for Causal Test"
    
    X_s = clean_df[['pre_s_mean']]
    X_t = clean_df[['age_min']]
    X_both = clean_df[['pre_s_mean', 'age_min']]
    y = clean_df['dp_norm']
    
    r2_s = LinearRegression().fit(X_s, y).score(X_s, y)
    r2_t = LinearRegression().fit(X_t, y).score(X_t, y)
    r2_both = LinearRegression().fit(X_both, y).score(X_both, y)
    
    return {
        "r2_stability": r2_s,
        "r2_age": r2_t,
        "r2_combined": r2_both,
        "gain": r2_both - r2_s
    }

import math
def main():
    export_dir = Path("exports")
    
    if len(sys.argv) > 1:
        file_path = export_dir / sys.argv[1]
    else:
        # Find newest file
        csv_files = sorted(export_dir.glob("hcsn_aggregator_*.csv"), key=os.path.getmtime)
        if not csv_files:
            print("No datasets found.")
            return
        file_path = csv_files[-1]
        
    print(f"Analyzing Discovery Dataset: {file_path.name}")
    
    df = pd.read_csv(file_path)
    if 'pre_age_a' not in df.columns:
        print("Dataset missing Age dimension. Re-instrumentation required.")
        return

    # Phase Analysis
    results = analyze_phase_space(df)
    
    print("\n" + "="*65)
    print("      HCSN PHASE DISCOVERY REPORT (v4.0)")
    print("="*65)
    
    # Identify Plateau
    valid_r2 = results['grid_r2'][~np.isnan(results['grid_r2'])]
    if len(valid_r2) > 0:
        plateau_mean = np.mean(valid_r2[valid_r2 > 0.05])
        vacuum_mean = np.mean(valid_r2[valid_r2 <= 0.05])
        gain = plateau_mean / (vacuum_mean + 1e-6)
        
        print(f"Phase Signal Gain: {gain:.2f}x")
        print(f"Plateau R2 Mean:   {plateau_mean:.4f}")
        print(f"Vacuum R2 Mean:    {vacuum_mean:.4f}")
    else:
        print("No statistically significant bins detected (N < 50 everywhere).")

    # Causal Test
    causal = causal_hierarchy_report(df)
    if isinstance(causal, dict):
        print("\nCausal Hierarchy Test:")
        print(f" - R2 (S only):   {causal['r2_stability']:.4f}")
        print(f" - R2 (t only):   {causal['r2_age']:.4f}")
        print(f" - R2 (Combined): {causal['r2_combined']:.4f}")
        print(f" - Info Gain (t): {causal['gain']:.4e}")
    
    # Boundary Extraction
    print("\nBoundary Extraction (R2 > 0.15):")
    # Simple threshold scan
    for r_thresh in [0.10, 0.15, 0.20]:
        matching_bins = np.sum(results['smoothed_r2'] > r_thresh)
        print(f" - Contour {r_thresh:.2f}: {matching_bins} signal pixels")

    print("\n" + "="*65)

if __name__ == "__main__":
    main()
