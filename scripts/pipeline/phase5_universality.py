import json
import glob
import os
import pandas as pd
import matplotlib.pyplot as plt

def extract_phases(directory):
    files = glob.glob(os.path.join(directory, "particle_lifetimes_*.json"))
    records = []
    for f in files:
        with open(f, 'r') as fp:
            data = json.load(fp)
            for k in data:
                age = k["age"]
                coh = k.get("coherence", 0.0)
                # Matter phase is defined strictly as structure with coherence > 1.0
                is_matter = coh > 1.0
                records.append({'age': age, 'is_matter': is_matter})
    return pd.DataFrame(records)

def analyze_universality(base_dir):
    variants = ['standard', 'high_noise', 'low_freeze', 'aggressive_control']
    data = {}
    
    for v in variants:
        df = extract_phases(os.path.join(base_dir, v))
        if len(df) > 0:
            total_spacetime_volume = df['age'].sum()
            matter_spacetime_volume = df[df['is_matter']]['age'].sum()
            matter_frac = matter_spacetime_volume / total_spacetime_volume if total_spacetime_volume > 0 else 0
            
            data[v] = {'total_volume': total_spacetime_volume, 'matter_frac': matter_frac}
            
    print("--- Universality Test Results ---")
    for k, v in data.items():
        print(f"Variant: {k:20s} | Total Spacetime Vol: {v['total_volume']:8.0f} | Matter Phase Fraction: {v['matter_frac']:.4f}")
        
    # Plot
    if data:
        fig, ax = plt.subplots(figsize=(8, 5))
        labels = list(data.keys())
        fracs = [data[k]['matter_frac'] for k in labels]
        
        ax.bar(labels, fracs, color=['blue', 'orange', 'green'])
        ax.set_ylabel('Matter Phase Fraction')
        ax.set_title('Phase 5: Universality of Matter Phase Across Rulesets')
        
        plt.tight_layout()
        plt.savefig('exports/phase5_universality.png')
        print("Saved plot to exports/phase5_universality.png")

if __name__ == "__main__":
    analyze_universality("exports/universality")
