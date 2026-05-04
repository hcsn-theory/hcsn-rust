import json
import os
import numpy as np
import scipy.stats as stats
import matplotlib.pyplot as plt

def analyze_dataset(p, data, steps=20000):
    ages = sorted([x["age"] for x in data if x["age"] >= 50])
    if len(ages) < 10:
        return {"p": p, "count": len(ages), "alpha": 0.0, "hazard_slope": 0.0, "condensed": False}

    # 1. Power Law Fit (Tail > 500)
    tail = [a for a in ages if a > 500]
    if len(tail) > 5:
        log_ages = np.log(tail)
        # We want P(tau) ~ tau^-alpha => log(P) ~ -alpha log(tau)
        # Using Zipf-like rank-frequency fit: r ~ f^-alpha => log(r) ~ -alpha log(f)
        # Simpler: using MLE for continuous power law
        x_min = 500
        alpha = 1 + len(tail) / sum(np.log(np.array(tail) / x_min))
    else:
        alpha = 0.0

    # 2. Hazard Decay check
    # Binning to find h(tau)
    bins = np.linspace(50, steps, 20)
    counts, edges = np.histogram(ages, bins=bins)
    # Survived to edge[i] = sum(counts[i:])
    survived = np.cumsum(counts[::-1])[::-1]
    
    hazard = []
    for i in range(len(counts) - 1):
        if survived[i] > 0:
            hazard.append(counts[i] / survived[i])
        else:
            hazard.append(0)
    
    # Check slope of hazard for first half
    h_y = np.array(hazard[:10])
    h_x = (edges[1:11] + edges[:10]) / 2
    if len(h_y) > 2:
        slope, _, _, _, _ = stats.linregress(h_x, h_y)
    else:
        slope = 0.0

    # 3. Condensation
    max_age = max(ages)
    condensed = max_age > 0.8 * steps

    return {
        "p": p,
        "count": len(ages),
        "alpha": alpha,
        "hazard_slope": slope,
        "condensed": condensed,
        "max_age": max_age
    }

def main():
    p_values = [0.63, 0.64, 0.65]
    results = []
    
    print("\n--- Advanced Regime Analysis ---")
    print("p_create | count | alpha | haz_slope | condensation | max_age")
    print("-------------------------------------------------------------")

    for p in p_values:
        p = round(p, 2)
        path = f"exports/particle_lifetimes_p{p:.2f}.json"
        if os.path.exists(path):
            with open(path, 'r') as f:
                data = json.load(f)
            
            res = analyze_dataset(p, data)
            results.append(res)
            
            cond_str = "YES" if res["condensed"] else "NO"
            print(f"{res['p']:7.2f} | {res['count']:5} | {res['alpha']:5.2f} | {res['hazard_slope']:9.4e} | {cond_str:12} | {res['max_age']:7}")

    # Best critical candidate?
    critical_candidates = [r for r in results if 1.5 <= r["alpha"] <= 2.5 and not r["condensed"]]
    if critical_candidates:
        print("\n\n🔥 Critical Phase Candidates:")
        for c in critical_candidates:
            print(f" -> p = {c['p']:.2f} (alpha = {c['alpha']:.2f})")
    else:
        # Relax constraints
        best = sorted(results, key=lambda x: abs(x["alpha"] - 2.0))[0]
        print(f"\nNo exact critical candidate in [1.5, 2.5]. Closest: p = {best['p']:.2f} (alpha = {best['alpha']:.2f})")

if __name__ == "__main__":
    main()
