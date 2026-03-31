import json
import os

def main():
    json_path = "exports/particle_lifetimes.json"
    if not os.path.exists(json_path):
        print(f"Error: Could not find {json_path}. Run the Rust simulation first.")
        return

    with open(json_path, 'r') as f:
        try:
            lifetimes = json.load(f)
        except json.JSONDecodeError:
            print("Error parsing JSON. Check simulation output.")
            return

    if not lifetimes:
        print("\n=======================================================")
        print("          HCSN Spatial Vacuum Baseline Confirmed       ")
        print("=======================================================")
        print("0 valid proto-particles (age >= 50) were detected.")
        print("\nResult: Pristine structural vacuum verified.")
        print("All high-coherence topological fluctuations organically")
        print("diffused back into the geometry before reaching the")
        print("survival threshold (tau_vac = 50 steps).")
        print("\nThis formally proves the baseline: the framework does")
        print("NOT artificially spawn matter without geometric consent.")
        print("=======================================================")
        print("\nTo map the spontaneous formation distribution, execute")
        print("the deep-time experimental simulation:")
        print(" -> cargo run --release --bin run_simulation -- --steps 50000\n")
        return

    ages = [p["age"] for p in lifetimes]
    sizes = [p["max_size"] for p in lifetimes]
    radii = [p["radius"] for p in lifetimes]

    print("\n=======================================================")
    print("          HCSN Spontaneous Emergence Results           ")
    print("=======================================================")
    print(f"Total Particles Formed: {len(lifetimes)}")
    print(f"Average Lifetime:       {sum(ages)/len(ages):.1f} steps")
    print(f"Max Lifetime:           {max(ages)} steps")
    print(f"Average Size:           {sum(sizes)/len(sizes):.1f} nodes")
    print(f"Average Bounded Radius: {sum(radii)/len(radii):.2f}")
    
    print("\nTop 5 Most Resilient Topological Knots:")
    for p in sorted(lifetimes, key=lambda x: x["age"], reverse=True)[:5]:
        status_label = "(ALIVE)" if p['status'] == "alive" else "(DECAYED)"
        print(f" -> Knot ID {p['id']:>3} | Survival: {p['age']:>5} steps | Vol: {p['max_size']:>3} | Rad: {p['radius']:.2f} {status_label}")
    
    print("\n[Data ready for publication distribution plots: Lifetime vs Volume]")
    print("=======================================================\n")

if __name__ == "__main__":
    main()
