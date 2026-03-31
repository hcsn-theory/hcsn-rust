import subprocess
import signal
import sys

def run_simulation(seed, steps=60000, p=0.64):
    cmd = [
        "cargo", "run", "--release", "--bin", "run_simulation",
        "--", 
        "--steps", str(steps), 
        "--p_create", f"{p:.3f}",
        "--seed", str(seed)
    ]
    print(f"\n[PRODUCTION] Seed {seed} | Steps {steps} | P {p:.3f}")
    try:
        subprocess.run(cmd, check=True)
    except subprocess.CalledProcessError:
        print(f"\n[ERROR] Simulation for seed {seed} failed or was interrupted.")
        sys.exit(1)

def main():
    # Seeds 1, 2, and 3 are finalized.
    seeds = [4, 5]
    p_create = 0.64
    steps = 60000
    
    print(f"--- Starting Phase 11 Production: Steps={steps} ---")
    for s in seeds:
        run_simulation(s, steps, p_create)

if __name__ == "__main__":
    main()
