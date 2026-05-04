import json
import glob
import os
import numpy as np
import pandas as pd
from scipy.stats import spearmanr

def analyze_conservation_replications(directory):
    files = glob.glob(os.path.join(directory, "interaction_events_*.json"))
    if not files:
        print("No replication files found.")
        return

    results = []
    
    for f in files:
        seed = f.split("_s")[-1].split(".")[0]
        with open(f, 'r') as fp:
            events = json.load(fp)
            
        records = []
        for e in events:
            if e.get("post_a") is None: continue
            pre_pa = e["pre_a"][2]
            post_pa = e["post_a"][2]
            dp_a = abs(post_pa - pre_pa) / (abs(pre_pa) + 1e-6)
            age_a = e["pre_a"][10]
            records.append({'drift': dp_a, 'persistence': age_a})
            
        df = pd.DataFrame(records)
        if len(df) > 10:
            rho, pval = spearmanr(df['persistence'], df['drift'])
            results.append({'seed': seed, 'rho': rho, 'p': pval, 'N': len(df)})
            
    res_df = pd.DataFrame(results)
    print("--- Phase 4 Replication Results ---")
    print(res_df.to_string(index=False))
    
    mean_rho = res_df['rho'].mean()
    std_rho = res_df['rho'].std()
    worst_seed = res_df.loc[res_df['rho'].idxmax()] # Maximum correlation (closest to 0 or positive)
    
    print(f"\nMean Spearman ρ: {mean_rho:.4f} ± {std_rho:.4f}")
    print(f"Worst-case seed: {worst_seed['seed']} (ρ = {worst_seed['rho']:.4f})")
    
    if mean_rho < -0.5 and worst_seed['rho'] < -0.3:
        print("\n✅ ROBUST EMERGENT CONSERVATION: Strong negative correlation persists across all graph sizes and seeds.")
    else:
        print("\n❌ FRAGILE CONSERVATION: The correlation breaks down under replication.")

if __name__ == "__main__":
    analyze_conservation_replications("exports/conservation/replication")
