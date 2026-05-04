import json
import os
import numpy as np

def analyze_ensemble(p=0.64, seeds=[1, 2, 3, 4, 5]):
    all_particles = []
    for s in seeds:
        path = f"exports/particle_lifetimes_p{p:.2f}_s{s}.json"
        if os.path.exists(path):
            with open(path, 'r') as f:
                try:
                    data = json.load(f)
                    all_particles.extend(data)
                except:
                    print(f"Error reading {path}")
    
    if not all_particles:
        print(f"No lifetime data found for p={p:.2f} ensemble.")
        return

    ages = np.array([p["age"] for p in all_particles])
    max_sizes = np.array([p["max_size"] for p in all_particles])
    
    # Calculate Power Law Exponent (Alpha) via Hazard Rate
    # Approximation: Alpha = 1 + N / sum(log(tau / tau_min))
    tau_min = 1000
    candidates = ages[ages >= tau_min]
    if len(candidates) > 10:
        alpha = 1 + len(candidates) / np.sum(np.log(candidates / tau_min))
    else:
        alpha = 0.0

    print("\n" + "="*60)
    print(f"       HCSN ENSEMBLE LIFETIME ANALYSIS (p={p:.2f})")
    print("="*60)
    print(f"Total Particles (Seeds 1-5): {len(all_particles)}")
    print(f"Particles > 10k steps:      {np.sum(ages >= 10000)}")
    print(f"Max Lifetime Observed:       {np.max(ages)} steps")
    print(f"Ensemble Growth Exponent(α): {alpha:.3f} (Target: 1.7-2.2)")
    print(f"Typical Particle Mass:       {np.mean(max_sizes):.1f} nodes")
    print("-"*60)
    
    print("Top 5 Most Resilient Topological Knots (Ensemble-Wide):")
    top_5 = sorted(all_particles, key=lambda x: x["age"], reverse=True)[:5]
    for i, p_info in enumerate(top_5):
        print(f" {i+1}. Knot ID {p_info['id']:>4} | Age: {p_info['age']:>6} | Max Vol: {p_info['max_size']:>4}")
    
    print("="*60 + "\n")

if __name__ == "__main__":
    analyze_ensemble()
