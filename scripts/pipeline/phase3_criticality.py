import json
import glob
import os
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt

def analyze_gamma_sweep(base_dir):
    gamma_dirs = glob.glob(os.path.join(base_dir, "gamma_*"))
    
    if not gamma_dirs:
        print("No gamma sweep data found in", base_dir)
        return
        
    results = []
    
    for g_dir in sorted(gamma_dirs):
        gamma_val = float(os.path.basename(g_dir).split("_")[1])
        
        # Load interactions to compute R^2
        files = glob.glob(os.path.join(g_dir, "interaction_events*.json"))
        events = []
        for f in files:
            with open(f, 'r') as fp:
                events.extend(json.load(fp))
                
        # Calculate Signal (R^2 between persistence and drift)
        records = []
        for e in events:
            if e.get("post_a") is None: continue
            pre_pa = e["pre_a"][2]
            post_pa = e["post_a"][2]
            dp_a = abs(post_pa - pre_pa) / (abs(pre_pa) + 1e-6)
            age_a = e["pre_a"][10]
            records.append((age_a, dp_a))
            
        df = pd.DataFrame(records, columns=['persistence', 'drift'])
        
        signal = 0
        if len(df) > 10:
            corr = df['persistence'].corr(df['drift'])
            # Since correlation is negative (drift goes down as persistence goes up), signal is R^2
            signal = corr**2 if not np.isnan(corr) else 0
            
        results.append({'gamma': gamma_val, 'signal': signal, 'events_count': len(events)})

    res_df = pd.DataFrame(results).sort_values('gamma')
    
    # Compute sharpness (first derivative of signal)
    res_df['sharpness'] = np.gradient(res_df['signal'], res_df['gamma'])
    # Compute jump (second derivative)
    res_df['jump'] = np.gradient(res_df['sharpness'], res_df['gamma'])
    
    # Plotting
    fig, ax1 = plt.subplots(figsize=(10, 6))
    
    ax1.plot(res_df['gamma'], res_df['signal'], 'b-o', linewidth=2, label='Signal (R²)')
    ax1.set_xlabel('Nonlinear Coupling (γ)')
    ax1.set_ylabel('Signal Strength', color='b')
    ax1.tick_params(axis='y', labelcolor='b')
    
    ax2 = ax1.twinx()
    ax2.plot(res_df['gamma'], res_df['sharpness'], 'r--s', linewidth=2, label='Sharpness (d/dγ)')
    ax2.set_ylabel('Sharpness', color='r')
    ax2.tick_params(axis='y', labelcolor='r')
    
    plt.title('Phase 3: Gamma Criticality Sweep')
    fig.tight_layout()
    plt.savefig('exports/phase3_criticality.png')
    print("Saved plot to exports/phase3_criticality.png")
    
    print("\n--- Criticality Detection ---")
    print(res_df.to_string(index=False))
    
    jump_idx = np.argmax(np.abs(res_df['jump']))
    gamma_star = res_df['gamma'].iloc[jump_idx]
    
    sharpness_peak_idx = np.argmax(res_df['sharpness'])
    gamma_sharp = res_df['gamma'].iloc[sharpness_peak_idx]
    
    print(f"\nSignal jump at γ ≈ {gamma_star:.2f}")
    print(f"Sharpness peak at γ ≈ {gamma_sharp:.2f}")
    
    if abs(jump_idx - sharpness_peak_idx) <= 1:
        print("✅ CONSISTENT CRITICALITY: jump and sharpness align")
    else:
        print("❌ INCONSISTENT: jump and sharpness don't align")

if __name__ == "__main__":
    analyze_gamma_sweep("exports")
