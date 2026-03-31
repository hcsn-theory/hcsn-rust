import subprocess
import os
import json
import numpy as np

def run_sim(p, steps=20000):
    cmd = [
        "cargo", "run", "--release", "--bin", "run_simulation",
        "--", "--steps", str(steps), "--p_create", f"{p:.2f}"
    ]
    print(f"Running: {' '.join(cmd)}")
    subprocess.run(cmd, check=True, capture_output=True)

def main():
    p_values = [0.63, 0.64, 0.65]
    results = []
    
    for p in p_values:
        p = round(p, 2)
        print(f"\n--- Scanning p_create = {p:.2f} ---")
        run_sim(p)
        
        # We assume the simulation exports to exports/particle_lifetimes_p{p:.2f}.json
        path = f"exports/particle_lifetimes_p{p:.2f}.json"
        if os.path.exists(path):
            with open(path, 'r') as f:
                data = json.load(f)
            
            ages = [x["age"] for x in data]
            if ages:
                max_age = max(ages)
                avg_age = sum(ages) / len(ages)
                results.append({
                    "p": p,
                    "count": len(ages),
                    "max_age": max_age,
                    "avg_age": avg_age
                })
            else:
                results.append({"p": p, "count": 0, "max_age": 0, "avg_age": 0})
        else:
            print(f"Warning: {path} not found")

    print("\n\n=== Sweep Results ===")
    print("p_create | count | max_age | avg_age")
    print("-----------------------------------")
    for r in results:
        print(f"{r['p']:7.2f} | {r['count']:5} | {r['max_age']:7} | {r['avg_age']:7.1f}")

if __name__ == "__main__":
    main()
