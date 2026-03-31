import json
import numpy as np
import matplotlib.pyplot as plt
from scipy.optimize import curve_fit
import os

def sigmoid(x, A, k, x0):
    return A / (1 + np.exp(-k * (x - x0)))

def piecewise_linear(x, k, x0):
    return np.maximum(0, k * (x - x0))

def power_law(x, A, beta, x0):
    return np.where(x > x0, A * (x - x0)**beta, 0)

def extract_kinematics(event):
    # pre_a: (m, v, p, (vx, vy))
    pre_a = event["pre_a"]
    pre_b = event["pre_b"]
    post_a = event["post_a"]
    post_b = event["post_b"]
    
    if post_a is None or post_b is None:
        return None
    
    # Calculate delta_p for both
    # p_vec = m * (vx, vy)
    p_pre_a = np.array(pre_a[3]) * pre_a[0]
    p_post_a = np.array(post_a[3]) * post_a[0]
    dp_a = np.linalg.norm(p_post_a - p_pre_a)
    
    p_pre_b = np.array(pre_b[3]) * pre_b[0]
    p_post_b = np.array(post_b[3]) * post_b[0]
    dp_b = np.linalg.norm(p_post_b - p_pre_b)
    
    # Scattering angle (degrees)
    cos_theta = np.dot(p_pre_a, p_post_a) / (np.linalg.norm(p_pre_a) * np.linalg.norm(p_post_a) + 1e-9)
    theta = np.degrees(np.arccos(np.clip(cos_theta, -1, 1)))
    
    return {
        "chi": event["overlap_depth"],
        "dp": (dp_a + dp_b) / 2.0,
        "theta": theta,
        "m_sum": pre_a[0] + pre_b[0],
        "m_diff": abs(pre_a[0] - pre_b[0]),
        "v_rel": np.linalg.norm(np.array(pre_a[3]) - np.array(pre_b[3]))
    }

def analyze_all_seeds(p=0.64, seeds=[1, 2, 3, 4, 5]):
    all_events = []
    for s in seeds:
        path = f"exports/interaction_events_p{p:.2f}_s{s}.json"
        if os.path.exists(path):
            with open(path, 'r') as f:
                events = json.load(f)
                for ev in events:
                    kin = extract_kinematics(ev)
                    if kin: all_events.append(kin)
    
    if not all_events:
        print("No interaction events found.")
        return

    # Filter by mass/velocity bands to reduce noise (as per reviewer advice)
    # Target: similar masses and relative velocities
    m_values = np.array([x["m_sum"] for x in all_events])
    v_values = np.array([x["v_rel"] for x in all_events])
    
    m_median = np.median(m_values)
    v_median = np.median(v_values)
    
    # Heuristic: 30% band around median mass and velocity
    filtered = [x for x in all_events if 
                abs(x["m_sum"] - m_median) < 0.3 * m_median and
                abs(x["v_rel"] - v_median) < 0.3 * v_median]
    
    if len(filtered) < 10:
        print(f"Warning: Stricter filter resulted in too few events ({len(filtered)}). Relaxing to 50% band.")
        filtered = [x for x in all_events if 
                    abs(x["m_sum"] - m_median) < 0.5 * m_median and
                    abs(x["v_rel"] - v_median) < 0.5 * v_median]

    if len(filtered) < 5:
        print(f"Warning: Very low event count ({len(filtered)}). Using all events.")
        filtered = all_events

    chi = np.array([x["chi"] for x in filtered])
    dp = np.array([x["dp"] for x in filtered])
    theta = np.array([x["theta"] for x in filtered])

    # Plot 1: Delta P vs Chi
    plt.figure(figsize=(12, 8))
    plt.scatter(chi, dp, alpha=0.4, color='blue', label="Filtered Interaction Data")
    
    x_fit = np.linspace(0, max(chi) if len(chi) > 0 else 1.0, 200)
    
    # 1. Sigmoid Fit
    try:
        popt_sig, _ = curve_fit(sigmoid, chi, dp, p0=[max(dp), 15, 0.2], maxfev=2000)
        plt.plot(x_fit, sigmoid(x_fit, *popt_sig), 'r--', linewidth=2, label=f"Sigmoid (chi_c={popt_sig[2]:.2f}, R2={np.corrcoef(dp, sigmoid(chi, *popt_sig))[0,1]**2:.2f})")
        print(f"Sigmoid Fit: A={popt_sig[0]:.2f}, k={popt_sig[1]:.2f}, x0={popt_sig[2]:.2f}")
    except Exception as e:
        print(f"Sigmoid fit failed: {e}")

    # 2. Piecewise Linear Fit
    try:
        popt_pw, _ = curve_fit(piecewise_linear, chi, dp, p0=[50, 0.2], maxfev=2000)
        plt.plot(x_fit, piecewise_linear(x_fit, *popt_pw), 'g-', linewidth=2, label=f"Piecewise (chi_c={popt_pw[1]:.2f})")
        print(f"Piecewise Fit: k={popt_pw[0]:.2f}, x0={popt_pw[1]:.2f}")
    except Exception as e:
        print(f"Piecewise fit failed: {e}")

    # 3. Power Law Fit
    try:
        popt_pow, _ = curve_fit(power_law, chi, dp, p0=[100, 1.5, 0.15], maxfev=2000)
        plt.plot(x_fit, power_law(x_fit, *popt_pow), 'm:', linewidth=2, label=f"Power Law (beta={popt_pow[1]:.2f})")
        print(f"Power Law Fit: A={popt_pow[0]:.2f}, beta={popt_pow[1]:.2f}, x0={popt_pow[2]:.2f}")
    except Exception as e:
        print(f"Power law fit failed: {e}")

    plt.xlabel("Overlap Depth (chi) [Normalised Structural Intersection]")
    plt.ylabel("Impulse Change (delta_p) [Topological Momentum Shift]")
    plt.title(f"HCSN Interaction Phenomenology: Impulse vs Overlap (p={p:.2f}, seed-aggregated)")
    plt.grid(True, alpha=0.3)
    plt.legend()
    plt.savefig("interaction_law_fitting.png")
    
    # Plot 2: Scattering Angle Distribution
    plt.figure(figsize=(10, 6))
    plt.hist(theta, bins=30, alpha=0.7, color='teal', edgecolor='black')
    plt.axvline(np.mean(theta), color='red', linestyle='--', label=f'Mean: {np.mean(theta):.1f}°')
    plt.xlabel("Scattering Angle (degrees)")
    plt.ylabel("Event Frequency")
    plt.title(f"Scattering Angular Isymmetry (p={p:.2f}, N={len(filtered)})")
    plt.legend()
    plt.savefig("scattering_angles.png")

    print(f"\nFinal Analysis of {len(filtered)} filtered interactions across {len(seeds)} worlds.")
    print(f"Typical Mass: {m_median:.2f} | Typical Rel Velocity: {v_median:.4f}")
    print(f"Mean Scattering Angle: {np.mean(theta):.2f}°")
    print(f"Back-scattering probability (>90°): {np.mean(theta > 90)*100:.1f}%")

if __name__ == "__main__":
    analyze_all_seeds()
