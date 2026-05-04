import os
import subprocess
import re
import pandas as pd
import matplotlib.pyplot as plt

def run_cmd(cmd, env_update=None):
    new_env = os.environ.copy()
    if env_update:
        new_env.update(env_update)
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True, env=new_env)
    return result.stdout, result.stderr

def main():
    p_values = [round(0.60 + i*0.01, 2) for i in range(11)]
    results = []

    print("=== HCSN Interaction Phase Diagram Builder ===")
    
    # Ensure export directory exists
    os.makedirs("exports", exist_ok=True)

    for p in p_values:
        csv_file = f"exports/interaction_{p:.2f}.csv"
        
        # 1. Check if simulation data exists
        if not os.path.exists(csv_file):
            print(f">>> PASS {p:.2f}: Simulating missing data (40,000 steps)...")
            env = {
                "HCSN_P_CREATE": str(p),
                "HCSN_OUT_FILE": csv_file
            }
            stdout, stderr = run_cmd("cargo run --release --bin force_law_aggregator", env)
            if "Aggregation Complete" not in stdout:
                print(f"!!! Error in simulation for p={p}: {stderr}")
                continue
        else:
            print(f">>> PASS {p:.2f}: Using existing data.")

        # 2. Run Robustness Attack (Fit)
        print(f"--- Fitting p={p:.2f} ---")
        fit_env = {"HCSN_IN_FILE": csv_file}
        stdout, stderr = run_cmd("cargo run --release --bin robustness_attack", fit_env)
        
        # 3. Parse Output
        slope_match = re.search(r"Slope:\s+([-]?\d+\.\d+)", stdout)
        r2_match = re.search(r"R²:\s+(\d+\.\d+)", stdout)
        n_match = re.search(r"Total Samples:\s+(\d+)", stdout)

        if slope_match and r2_match:
            slope = float(slope_match.group(1))
            r2 = float(r2_match.group(1))
            n = int(n_match.group(1)) if n_match else 0
            
            results.append({
                "p_create": p,
                "slope": slope,
                "r2": r2,
                "n_samples": n
            })
            print(f"    Result: b={slope:.6f}, R2={r2:.4f} (N={n})")
        else:
            print(f"!!! Could not parse results for p={p}: {stdout[:200]}")

    # 4. Save Summary
    df = pd.DataFrame(results)
    df.to_csv("exports/phase_diagram_results.csv", index=False)
    print("\nSummary saved to exports/phase_diagram_results.csv")

    # 5. Plotting
    if not df.empty:
        fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(10, 8), sharex=True)
        
        ax1.plot(df["p_create"], df["slope"], 'o-', color='crimson', linewidth=2)
        ax1.set_ylabel("Interaction Slope (b)")
        ax1.set_title("HCSN Interaction Phase Diagram")
        ax1.grid(True, alpha=0.3)
        
        ax2.plot(df["p_create"], df["r2"], 's--', color='navy', linewidth=2)
        ax2.set_ylabel("Consistency (R²)")
        ax2.set_xlabel("Vacuum Creation Rate (p_create)")
        ax2.grid(True, alpha=0.3)
        
        plt.tight_layout()
        plt.savefig("exports/phase_diagram.png")
        print("Plot saved to exports/phase_diagram.png")

if __name__ == "__main__":
    main()
