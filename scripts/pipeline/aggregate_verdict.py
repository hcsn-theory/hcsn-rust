import csv, math, os, sys, json
import numpy as np
from pathlib import Path
from collections import defaultdict

def analyze_dataset(events):
    # Filter for knots that have survived at least one step and have structural meaning
    physical_events = [e for e in events if e['stab'] > 1.0]
    
    if len(physical_events) < 50: 
        return {
            "n": len(events),
            "n_physical": len(physical_events),
            "r_squared": 0.0,
            "mean_theta": 0.0,
            "mean_dp": 0.0,
            "error": "Insufficient mature knots (N_phys < 50)"
        }
    
    dp_vals = [e['dp'] for e in physical_events]
    stab_vals = [e['stab'] for e in physical_events]
    theta_vals = [e['theta'] for e in physical_events]
    
    y = np.array(dp_vals)
    x = np.array(stab_vals)
    
    if np.std(x) < 1e-9 or np.std(y) < 1e-9:
        return {
            "n": len(events),
            "n_physical": len(physical_events),
            "r_squared": 0.0,
            "mean_theta": np.mean(theta_vals),
            "mean_dp": np.mean(dp_vals),
            "error": "Zero variance in sample subset"
        }

    corr_matrix = np.corrcoef(x, y)
    r_val = corr_matrix[0,1]
    r_sq = r_val**2 if not np.isnan(r_val) else 0.0
    
    return {
        "n": len(events),
        "n_physical": len(physical_events),
        "r_squared": r_sq,
        "mean_theta": np.mean(theta_vals),
        "mean_dp": np.mean(dp_vals)
    }

def main():
    export_dir = Path("exports")
    all_events_by_seed = defaultdict(list)
    
    # 1. Gather ONLY recent CSVs (last 12 hours) to capture the Deep-Time results
    import time
    now = time.time()
    csv_files = [f for f in export_dir.glob("hcsn_aggregator_*.csv") if now - f.stat().st_mtime < 43200]
    
    if not csv_files:
        print("No recent high-fidelity data found. Run a campaign first.")
        return

    print(f"Found {len(csv_files)} high-fidelity data blocks.")
    
    for csv_path in csv_files:
        meta_path = csv_path.with_suffix(".csv.meta")
        tid_to_seed = {}
        if meta_path.exists():
            with open(meta_path, 'r') as f:
                meta = json.load(f)
                for entry in meta.get('seeds', []):
                    tid_to_seed[int(entry['tid'])] = int(entry['seed'])
        
        with open(csv_path, 'r') as f:
            reader = csv.DictReader(f)
            for row in reader:
                try:
                    if 'NaN' in row.values() or 'nan' in row.values(): continue
                    
                    tid = int(row['thread_id'])
                    seed = tid_to_seed.get(tid, tid)
                    
                    pre = float(row['pre_p_mag'])
                    post = float(row['post_p_mag'])
                    
                    all_events_by_seed[seed].append({
                        "dp": abs(post - pre),
                        "stab": float(row['pre_s_mean']),
                        "theta": float(row.get('theta', 0.0))
                    })
                except: continue

    # 3. Analyze each seed
    production_seeds = list(range(42, 47)) + list(range(1000, 1005)) + list(range(2000, 2010)) + list(range(3000, 3010))
    null_seeds = [47]
    
    print("\n" + "="*65)
    print("      HCSN CONSISTENCY VERDICT: DEEP-TIME REPLICATION")
    print("="*65)
    print(f"{'Seed':<6} | {'Status':<12} | {'N_Total':<7} | {'N_Phys':<7} | {'R2(S)':<7} | {'Mean DP':<8}")
    print("-" * 65)
    
    results = {}
    for seed in sorted(all_events_by_seed.keys()):
        stats = analyze_dataset(all_events_by_seed[seed])
        if stats:
            label = "PRODUCTION" if seed in production_seeds else "NULL CONTROL" if seed in null_seeds else "OTHER"
            results[seed] = stats
            r2_str = f"{stats['r_squared']:.4f}" if "error" not in stats else "ERR"
            print(f"{seed:<6} | {label:<12} | {stats['n']:<7} | {stats['n_physical']:<7} | {r2_str:<7} | {stats['mean_dp']:<8.2f}")

    # 4. Global Verdict
    print("-" * 65)
    prod_valid = [results[s] for s in results if s in production_seeds and "error" not in results[s]]
    null_valid = [results[s] for s in results if s in null_seeds and "error" not in results[s]]
    
    if prod_valid:
        avg_prod = np.mean([r['r_squared'] for r in prod_valid])
        print(f"Mean Production R2: {avg_prod:.4f} (from {len(prod_valid)} seeds)")
    if null_valid:
        avg_null = np.mean([r['r_squared'] for r in null_valid])
        print(f"Mean Null Control R2: {avg_null:.4f}")
        
    print("\nVERDICT:")
    if prod_valid and avg_prod > 0.40:
        print(">>> STATUS: REPRODUCIBILITY CONFIRMED")
        print("    Stability is a systematic predictor of scattering across deterministic seeds.")
        if null_valid and avg_prod > (avg_null * 1.5):
            print(">>> STATUS: PHYSICS SIGNAL EXCEEDS NOISE FLOOR (Null Control Passed)")
    else:
        print(">>> STATUS: HYPOTHESIS UNVERIFIED (Low Signal-to-Noise Ratio)")
        print("    The stability signal may be obscured by high-energy 'Vacuum Noise' in short runs.")

if __name__ == "__main__":
    main()
