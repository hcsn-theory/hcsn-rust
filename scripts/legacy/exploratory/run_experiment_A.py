import os
import json
import subprocess

def run_experiment_A():
    print("==================================================")
    print(" EXPERIMENT A: Near-Critical Omega (Metastability)")
    print("==================================================")
    
    p_values = [0.45, 0.50, 0.60, 0.65, 0.70]
    steps = 20000
    
    results = {}
    
    for p in p_values:
        print(f"\n[+] Running vacuum geometry at p_create = {p} for {steps} steps...")
        cmd = f"cargo run --release --bin run_simulation -- --steps {steps} --p_create {p}"
        
        # Stream stdout clearly
        subprocess.run(cmd, shell=True)
        
        json_file = f"exports/particle_lifetimes_p{p:.2f}.json"
        if os.path.exists(json_file):
            with open(json_file, 'r') as f:
                data = json.load(f)
                
            particles = len(data)
            max_age = max([d["age"] for d in data]) if data else 0
            avg_size = (sum([d["max_size"] for d in data]) / particles) if particles else 0
            
            print(f"    --> Particles Emerged: {particles}")
            if particles > 0:
                print(f"    --> Max Lifetime:      {max_age} steps")
                print(f"    --> Avg Bound Vol:     {avg_size:.1f} nodes")
            
            results[p] = particles
        else:
            print(f"    --> Error: {json_file} was not generated.")
            
    print("\n==================================================")
    print(" EXPERIMENT A SUMMARY")
    print("==================================================")
    for p, count in results.items():
        print(f" p_create = {p:.2f} : {count} anomalies formed")
        
if __name__ == "__main__":
    run_experiment_A()
