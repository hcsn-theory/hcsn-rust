import os
import subprocess
import json

def run_test(name, cmd_args, expected_file):
    print(f"\n[+] Running {name}...")
    cmd = f"cargo run --release --bin run_simulation -- --steps 3000 {cmd_args}"
    subprocess.run(cmd, shell=True)
    
    if os.path.exists(expected_file):
        with open(expected_file, 'r') as f:
            data = json.load(f)
        particles = len(data)
        print(f"    --> Particles Emerged: {particles}")
        
        if particles > 0:
            ages = [p["age"] for p in data]
            sizes = [p["max_size"] for p in data]
            print(f"    --> Max Age: {max(ages)}, Avg Vol: {sum(sizes)/len(sizes):.1f}")
        return particles
    else:
        print("    --> ERROR: File not found.")
        return -1

def main():
    print("==================================================")
    print(" PHASE 2: CONDITIONAL EMERGENCE EXPERIMENTS       ")
    print("==================================================")
    
    results = {}
    
    # Resume Experiment A
    for p in [0.60, 0.65, 0.70]:
        c = run_test(f"Exp A (p_create={p})", f"--p_create {p}", f"exports/particle_lifetimes_p{p:.2f}.json")
        results[f"A_p{p}"] = c
        
    # Experiment B
    for n in [0.05, 0.15, 0.35]:
        c = run_test(f"Exp B (noise_bias={n})", f"--p_create 0.55 --noise_bias {n}", f"exports/particle_lifetimes_p0.55.json")
        results[f"B_n{n}"] = c
        
    # Experiment C
    for d in [0.0001, 0.001]:
        c = run_test(f"Exp C (defect_inj={d})", f"--p_create 0.55 --defect_injection {d}", f"exports/particle_lifetimes_p0.55.json")
        results[f"C_d{d}"] = c

    # Experiment D
    for g in [0.95, 0.99]:
        c = run_test(f"Exp D (freeze={g})", f"--p_create 0.55 --geometry_freeze {g}", f"exports/particle_lifetimes_p0.55.json")
        results[f"D_g{g}"] = c

    print("\n==================================================")
    print(" PHASE 2 SUMMARY RESULTS")
    print("==================================================")
    for k, v in results.items():
        print(f" {k:<15} : {v:>4} particles")

if __name__ == '__main__':
    main()
