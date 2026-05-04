import csv, math, os, sys
import numpy as np
from pathlib import Path

def analyze_split(filepath, target_tid):
    chi_vals = []
    dp_vals = []
    stab_vals = []
    theta_vals = []
    
    with open(filepath, 'r') as f:
        reader = csv.DictReader(f)
        for row in reader:
            if int(row['thread_id']) != target_tid:
                continue
            try:
                chi = float(row['chi'])
                pre = float(row['pre_p_mag'])
                post = float(row['post_p_mag'])
                dp = abs(post - pre)
                stab = float(row['pre_s_mean'])
                theta = float(row['theta'])
                
                chi_vals.append(chi)
                dp_vals.append(dp)
                stab_vals.append(stab)
                theta_vals.append(theta)
            except: continue
            
    if not dp_vals: return None
    
    # Calculate R-squared for Stability vs dp
    y = np.array(dp_vals)
    x = np.array(stab_vals)
    if len(x) < 5: return None
    
    corr_matrix = np.corrcoef(x, y)
    r_sq = corr_matrix[0,1]**2
    
    return {
        "n": len(dp_vals),
        "r_squared_stability": r_sq,
        "mean_theta": np.mean(theta_vals),
        "mean_dp": np.mean(dp_vals)
    }

def main(campaign_csv):
    print(f"=== HCSN REPRODUCIBILITY ANALYZER: {campaign_csv} ===")
    
    tids = set()
    with open(campaign_csv, 'r') as f:
        reader = csv.DictReader(f)
        if 'thread_id' not in reader.fieldnames:
            print(f"ERROR: File {campaign_csv} is missing 'thread_id' column. Skipping.")
            return
        for row in reader: tids.add(int(row['thread_id']))
        
    results = {}
    for tid in sorted(list(tids)):
        res = analyze_split(campaign_csv, tid)
        if res:
            results[tid] = res
            print(f"Seed {tid}: N={res['n']:4} | R2(S)={res['r_squared_stability']:.4f} | <Theta>={res['mean_theta']:.2f}")

    if not results:
        print("No valid data found in splits.")
        return

    # Consistency Verdict
    r2_vals = [r['r_squared_stability'] for r in results.values()]
    theta_vals = [r['mean_theta'] for r in results.values()]
    
    strong_r2 = sum(1 for r in r2_vals if r > 0.70)
    theta_stability = np.std(theta_vals) / np.mean(theta_vals) if np.mean(theta_vals) != 0 else 0
    
    print("\n--- CONSISTENCY VERDICT ---")
    print(f"Reliability: {strong_r2}/{len(results)} seeds passed R2 > 0.70")
    print(f"Scattering Variance: {theta_stability*100:.2f}%")
    
    if strong_r2 >= 4:
        print("STATUS: STRONG REPRODUCIBILITY")
    elif strong_r2 >= 3:
        print("STATUS: MODERATE REPRODUCIBILITY")
    else:
        print("STATUS: DIVERGENT SIGNAL")

if __name__ == "__main__":
    if len(sys.argv) > 1:
        main(sys.argv[1])
    else:
        print("Usage: python3 analyze_reproducibility.py <csv_file>")
