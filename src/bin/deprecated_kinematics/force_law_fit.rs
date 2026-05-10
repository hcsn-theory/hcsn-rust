use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
struct Point {
    chi: f64,
    dp: f64,
}

fn main() {
    println!("=== HCSN FUNCTIONAL INTERACTION FITTER ===");

    let path = "exports/interaction_points_raw.csv";
    let file = File::open(path).expect("Could not open interaction data");
    let reader = BufReader::new(file);

    let mut points = Vec::new();
    for line in reader.lines().skip(1) {
        let l = line.unwrap();
        let parts: Vec<&str> = l.split(',').collect();
        if parts.len() >= 2 {
            let chi: f64 = parts[0].parse().unwrap();
            let dp: f64 = parts[1].parse().unwrap();
            points.push(Point { chi, dp });
        }
    }

    let n = points.len();
    println!("Loaded N = {} data points.", n);

    if n == 0 {
        return;
    }

    // --- PHASE 1: BINNING (Noise Reduction) ---
    const NUM_BINS: usize = 50;
    let mut bins: Vec<Vec<f64>> = vec![Vec::new(); NUM_BINS];
    let min_chi = points.iter().map(|p| p.chi).fold(f64::INFINITY, f64::min);
    let max_chi = points
        .iter()
        .map(|p| p.chi)
        .fold(f64::NEG_INFINITY, f64::max);
    let range_chi = (max_chi - min_chi).max(1e-6);

    for p in &points {
        let mut bin_idx = ((p.chi - min_chi) / range_chi * NUM_BINS as f64) as usize;
        if bin_idx >= NUM_BINS {
            bin_idx = NUM_BINS - 1;
        }
        bins[bin_idx].push(p.dp);
    }

    let mut binned_points = Vec::new();
    for i in 0..NUM_BINS {
        if !bins[i].is_empty() {
            let b_chi = min_chi + (i as f64 + 0.5) * (range_chi / NUM_BINS as f64);
            let b_dp: f64 = bins[i].iter().sum::<f64>() / bins[i].len() as f64;
            binned_points.push(Point {
                chi: b_chi,
                dp: b_dp,
            });
        }
    }
    println!(
        "Reduced noise to {} binned mean points.",
        binned_points.len()
    );
    for p in &binned_points {
        println!("  chi={:.4}, dp_mean={:.4}", p.chi, p.dp);
    }

    // --- PHASE 2: FITTING BINNED DATA ---
    let sum_dp: f64 = binned_points.iter().map(|p| p.dp).sum();
    let mean_dp = sum_dp / binned_points.len() as f64;
    let ss_tot: f64 = binned_points.iter().map(|p| (p.dp - mean_dp).powi(2)).sum();

    println!("\n--- Model A: Peaked Decay (A * chi * exp(-chi/x0)) ---");
    let mut best_a = 0.0;
    let mut best_x0 = 0.0;
    let mut min_sse_a = f64::INFINITY;

    for a_int in 5..200 {
        // A from 0.5 to 20.0
        let a = a_int as f64 * 0.1;
        for x0_int in 1..80 {
            // x0 from 0.01 to 0.8
            let x0 = x0_int as f64 * 0.01;
            let mut sse = 0.0;
            for p in &binned_points {
                let pred = a * p.chi * (-p.chi / x0).exp();
                sse += (p.dp - pred).powi(2);
            }
            if sse < min_sse_a {
                min_sse_a = sse;
                best_a = a;
                best_x0 = x0;
            }
        }
    }
    let r2_a = 1.0 - (min_sse_a / ss_tot);
    println!(
        "Best Fit: Δp_norm = {:.4} * χ * exp(-χ / {:.4})",
        best_a, best_x0
    );
    println!("R-squared: {:.6}", r2_a);

    println!("\n--- Model B: Sigmoidal Saturation (L / (1 + exp(-k*(chi - xc)))) ---");
    let mut best_l = 0.0;
    let mut best_k = 0.0;
    let mut best_xc = 0.0;
    let mut min_sse_b = f64::INFINITY;

    for l_int in 1..20 {
        // L from 0.1 to 2.0
        let l = l_int as f64 * 0.1;
        for k_int in 1..50 {
            // k from 1 to 50
            let k = k_int as f64;
            for xc_int in 1..40 {
                // xc from 0.01 to 0.4
                let xc = xc_int as f64 * 0.01;
                let mut sse = 0.0;
                for p in &binned_points {
                    let pred = l / (1.0 + (-k * (p.chi - xc)).exp());
                    sse += (p.dp - pred).powi(2);
                }
                if sse < min_sse_b {
                    min_sse_b = sse;
                    best_l = l;
                    best_k = k;
                    best_xc = xc;
                }
            }
        }
    }
    let r2_b = 1.0 - (min_sse_b / ss_tot);
    println!(
        "Best Fit: Δp_norm = {:.4} / (1 + exp(-{:.4} * (χ - {:.4})))",
        best_l, best_k, best_xc
    );
    println!("R-squared: {:.6}", r2_b);

    println!("\n=== VERDICT ===");
    if r2_a > r2_b {
        println!("Winner: Model A (Peaked Coupling)");
        println!("Equation: Δp_norm = {:.4}χ * e^(-χ/{:.4})", best_a, best_x0);
    } else {
        println!("Winner: Model B (Sigmoidal Saturation)");
        println!(
            "Equation: Δp_norm = {:.4} / (1 + e^(-{:.4}(χ - {:.4})))",
            best_l, best_k, best_xc
        );
    }
}
