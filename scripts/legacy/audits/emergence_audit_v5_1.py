import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import os

def analyze_regime(csv_path, label):
    if not os.path.exists(csv_path):
        print(f"Error: {csv_path} not found.")
        return None, None

    df = pd.read_csv(csv_path)
    if len(df) < 5:
        return None, None

    # 1. PHYSICAL DIMENSIONS
    df['dPx'] = df['post_px'] - df['pre_px']
    df['dP_mag'] = df['dPx'].abs()
    df['P_pre_mag'] = df['pre_px'].abs().replace(0, 1e-6)
    df['eps_P'] = df['dP_mag'] / df['P_pre_mag']
    
    # Filter extreme pathological outliers 
    df = df[df['eps_P'] < 10.0]

    # 2. DEFINING REGIMES
    # Baseline Noise: S < 5
    # Transition:    5 <= S < 12 (v5.1 centers at 12)
    # Emergent:      S >= 12
    conditions = [
        (df['pre_s_mean'] < 5.0),
        (df['pre_s_mean'] >= 5.0) & (df['pre_s_mean'] < 12.0),
        (df['pre_s_mean'] >= 12.0)
    ]
    regimes = ['Noise', 'Transition', 'Emergent Physics']
    df['regime'] = np.select(conditions, regimes, default='Other')
    
    # 3. BINNING BY STABILITY (S)
    bins = np.linspace(0, 50, 11) 
    df['stab_bin'] = pd.cut(df['pre_s_mean'], bins=bins, labels=bins[:-1])
    
    s_stats = df.groupby('stab_bin')['eps_P'].agg(['mean', 'std', 'count']).dropna()
    mean_eps_phys = df[df['regime'] == 'Emergent Physics']['eps_P'].mean()
    
    return s_stats, (label, mean_eps_phys, len(df))

def run_comparative_audit():
    results = []
    
    # Regimes to compare
    regimes = [
        ('exports/v5_1_control.csv', 'Control (Pure v5.0)'),
        ('exports/v5_1_assisted.csv', 'Assisted (Honest v5.1)'),
        ('exports/v5_1_forced.csv', 'Forced (Artificial v5.1)')
    ]
    
    plt.figure(figsize=(18, 8))
    plt.subplot(1, 2, 1) # Main Emergence Curves
    
    colors = ['gray', 'blue', 'red']
    
    for i, (path, label) in enumerate(regimes):
        stats, meta = analyze_regime(path, label)
        if stats is not None:
            plt.errorbar(stats.index.astype(float), stats[('mean')], 
                         yerr=stats[('std')] / np.sqrt(stats[('count')].replace(0, 1)), 
                         fmt='o-', color=colors[i], label=label, alpha=0.8)
            results.append(meta)

    plt.axvspan(0, 5, color='gray', alpha=0.05, label='Noise Regime')
    plt.axvspan(12, 50, color='green', alpha=0.05, label='Conserved Regime')
    
    plt.axhline(0.2, color='black', linestyle=':', alpha=0.5, label='Fidelity Target (0.2)')
    plt.xlabel('Topological Stability (S)')
    plt.ylabel('Conservation Error (eps_P)')
    plt.title('HCSN v5.1: Comparative Phase Diagram\nNatural vs Assisted vs Forced Emergence')
    plt.yscale('log')
    plt.grid(True, which="both", alpha=0.3)
    plt.legend()

    # Verdict Plot
    plt.subplot(1, 2, 2)
    labels = [r[0] for r in results]
    eps_vals = [r[1] for r in results]
    counts = [r[2] for r in results]
    
    bar_colors = colors[:len(results)]
    plt.bar(labels, eps_vals, color=bar_colors, alpha=0.6)
    plt.ylabel('Mean eps_P in Physics Regime (S > 12)')
    plt.yscale('log')
    plt.title('Emergence Verification: Regime Fidelity Comparison')
    
    for i, v in enumerate(eps_vals):
        if pd.notnull(v):
            plt.text(i, v, f"{v:.4f}\n(N={counts[i]})", ha='center', va='bottom', fontweight='bold')
        else:
             plt.text(i, 0.1, f"Missing Regime\n(N={counts[i]})", ha='center', va='bottom', color='red')

    plt.tight_layout()
    plt.savefig('exports/comparative_phase_diagram_v5_1.png')
    plt.close()

    # 5. FINAL REPORTING
    print(f"\n========================================")
    print(f"HCSN v5.1: COMPARATIVE PHASE AUDIT")
    print(f"========================================")
    for r in results:
        v_str = f"{r[1]:.6f}" if pd.notnull(r[1]) else "INCONCLUSIVE"
        print(f"{r[0]: <24}: eps_P={v_str} (N_total={r[2]})")
    
    if len(results) >= 2:
        control_eps = results[0][1]
        assisted_eps = results[1][1]
        
        if pd.notnull(control_eps) and pd.notnull(assisted_eps):
            improvement = (1 - (assisted_eps / control_eps)) * 100
            print(f"----------------------------------------")
            print(f"Emergence Ratio (B/A): {assisted_eps/control_eps:.4f}")
            if assisted_eps < control_eps:
                print(f"Verdict:             🎯 NATURAL PHASE TRANSITION CONFIRMED")
            else:
                print(f"Verdict:             Assistance Required")
        else:
             print(f"Verdict:             INCONCLUSIVE (Missing Regimes)")
    print(f"========================================\n")

if __name__ == "__main__":
    run_comparative_audit()
