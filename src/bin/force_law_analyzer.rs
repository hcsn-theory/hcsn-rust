use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

fn main() {
    println!("=== HCSN MULTI-DIMENSIONAL FORCE LAW ANALYZER ===");
    
    let path = "exports/interaction_points_raw.csv";
    let file = File::open(path).expect("Could not open interaction_points_raw.csv");
    let reader = BufReader::new(file);
    
    let mut data = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        if i == 0 { continue; } // skip header
        let l = line.unwrap();
        let parts: Vec<f64> = l.split(',').map(|s| s.parse().unwrap_or(0.0)).collect();
        if parts.len() >= 7 {
            data.push(parts);
        }
    }
    
    println!("Total Samples: {}", data.len());
    if data.is_empty() { return; }

    // Bin settings
    let bins_chi = 20;
    let bins_x = 20;

    // 1. Compute Base Var1 = Var(Δp | χ bins)
    let var1 = compute_conditional_variance(&data, 0, 1, bins_chi);
    println!("Baseline Variance | Var(Δp | χ): {:.6}", var1);
    
    println!("\nCandidate Variable | Variance Reduction R = (Var1 - Var2)/Var1");
    println!("----------------------------------------------------------");

    let candidates = vec![
        ("Coherence (coh)", 2),
        ("Stability (stab)", 3),
        ("Radius (rad)", 4),
        ("Size (size)", 5),
        ("Boundary Ratio (ratio)", 6),
    ];

    let mut results = Vec::new();

    for (name, idx) in candidates {
        let var2 = compute_2d_conditional_variance(&data, 0, idx, 1, bins_chi, bins_x);
        let r = (var1 - var2) / var1;
        println!("{:<25} | R = {:.4} (Var2: {:.6})", name, r, var2);
        results.push((name, r));
    }

    // 2. Combined Variables
    println!("\nCombined Variables:");
    println!("----------------------------------------------------------");
    
    // Composite: χ * coh
    let data_chi_coh: Vec<Vec<f64>> = data.iter().map(|v| vec![v[0], v[1], v[0] * v[2]]).collect();
    let var_chi_coh = compute_2d_conditional_variance(&data_chi_coh, 0, 2, 1, bins_chi, bins_x);
    let r_chi_coh = (var1 - var_chi_coh) / var1;
    println!("{:<25} | R = {:.4}", "χ * Coherence", r_chi_coh);

    // Composite: χ * stab
    let data_chi_stab: Vec<Vec<f64>> = data.iter().map(|v| vec![v[0], v[1], v[0] * v[3]]).collect();
    let var_chi_stab = compute_2d_conditional_variance(&data_chi_stab, 0, 2, 1, bins_chi, bins_x);
    let r_chi_stab = (var1 - var_chi_stab) / var1;
    println!("{:<25} | R = {:.4}", "χ * Stability", r_chi_stab);

    // Composite: χ * ratio
    let data_chi_ratio: Vec<Vec<f64>> = data.iter().map(|v| vec![v[0], v[1], v[0] * v[6]]).collect();
    let var_chi_ratio = compute_2d_conditional_variance(&data_chi_ratio, 0, 2, 1, bins_chi, bins_x);
    let r_chi_ratio = (var1 - var_chi_ratio) / var1;
    println!("{:<25} | R = {:.4}", "χ * Ratio", r_chi_ratio);

    // 3. High-Resolution Distribution Analysis (100 bins)
    println!("\nHigh-Resolution Distribution Analysis (Δp 100-bin Histogram):");
    println!("----------------------------------------------------------");
    let n_bins = 100;
    let mut hist = vec![0; n_bins];
    for row in &data {
        let bin = (row[1] * (n_bins as f64)).min(n_bins as f64 - 1.0) as usize;
        hist[bin] += 1;
    }

    // Peak Detection
    let (mut p1_idx, mut p1_val) = (0, 0);
    for i in 0..n_bins / 2 {
        if hist[i] > p1_val { p1_val = hist[i]; p1_idx = i; }
    }
    
    let (mut p2_idx, mut p2_val) = (0, 0);
    for i in n_bins / 2..n_bins {
        if hist[i] > p2_val { p2_val = hist[i]; p2_idx = i; }
    }

    // Valley Detection (between p1 and p2)
    let (mut v_idx, mut v_val) = (0, usize::MAX);
    for i in p1_idx..p2_idx {
        if hist[i] < v_val { v_val = hist[i]; v_idx = i; }
    }

    // Peak Separation Analysis
    let p1_x = p1_idx as f64 / n_bins as f64;
    let p2_x = p2_idx as f64 / n_bins as f64;
    let sep = p2_x - p1_x;
    
    // Simple Width Estimation (FWHM approx)
    let mut w1 = 0;
    for i in p1_idx..n_bins/2 { if hist[i] < p1_val / 2 { w1 = i - p1_idx; break; } }
    let mut w2 = 0;
    for i in (n_bins/2..p2_idx).rev() { if hist[i] < p2_val / 2 { w2 = p2_idx - i; break; } }
    let total_width = (w1 as f64 + w2 as f64) / n_bins as f64;

    for i in 0..n_bins {
        if hist[i] > 0 {
            let bar = "*".repeat((hist[i] * 50 / p2_val.max(p1_val)).min(50));
            let marker = if i == p1_idx { " [PEAK 1]" } else if i == p2_idx { " [PEAK 2]" } else if i == v_idx { " [VALLEY]" } else { "" };
            if hist[i] > 10 || marker != "" {
                println!("{:.3} | {:<50} (n={}){}", i as f64 / n_bins as f64, bar, hist[i], marker);
            }
        }
    }

    println!("\nSpectral Statistics:");
    println!("----------------------------------------------------------");
    println!("Peak 1 (Pass-Through): {:.3}", p1_x);
    println!("Peak 2 (Reflection):   {:.3}", p2_x);
    println!("Valley (Coupling Gap): {:.3} (n={})", v_idx as f64 / n_bins as f64, v_val);
    println!("Peak Separation:       {:.4}", sep);
    if total_width > 0.0 {
        println!("Resolution Factor Q:   {:.4}", sep / total_width);
    }

    // 4. Branching Ratio Analysis (P_drag vs Stability)
    println!("\nBranching Ratio Analysis | P(Drag) where Δp > 0.1");
    println!("----------------------------------------------------------");
    
    // Bin by Stability
    let mut stab_bins: HashMap<usize, (usize, usize)> = HashMap::new(); // bin -> (drag_count, total_count)
    let min_s = data.iter().map(|v| v[3]).fold(f64::INFINITY, f64::min);
    let max_s = data.iter().map(|v| v[3]).fold(f64::NEG_INFINITY, f64::max);
    let range_s = (max_s - min_s).max(1e-6);

    for row in &data {
        let b = (((row[3] - min_s) / range_s) * 9.0) as usize; // 10 bins
        let entry = stab_bins.entry(b).or_insert((0, 0));
        entry.1 += 1;
        if row[1] > 0.1 {
            entry.0 += 1;
        }
    }

    let mut sorted_keys: Vec<_> = stab_bins.keys().cloned().collect();
    sorted_keys.sort();
    
    println!("{:<20} | {:<10} | {:<10}", "Stability Bin", "P(Drag)", "Total N");
    for k in sorted_keys {
        let (drag, total) = stab_bins[&k];
        let p = drag as f64 / total as f64;
        let s_val = min_s + (k as f64 / 9.0) * range_s;
        println!("{:<20.4} | {:<10.4} | {:<10}", s_val, p, total);
    }
}

fn compute_conditional_variance(data: &Vec<Vec<f64>>, idx_cond: usize, idx_val: usize, n_bins: usize) -> f64 {
    let mut bins: HashMap<usize, Vec<f64>> = HashMap::new();
    
    let min_c = data.iter().map(|v| v[idx_cond]).fold(f64::INFINITY, f64::min);
    let max_c = data.iter().map(|v| v[idx_cond]).fold(f64::NEG_INFINITY, f64::max);
    let range = (max_c - min_c).max(1e-6);

    for row in data {
        let b = (((row[idx_cond] - min_c) / range) * (n_bins as f64 - 1.0)) as usize;
        bins.entry(b).or_default().push(row[idx_val]);
    }

    let mut total_weighted_var = 0.0;
    let mut total_samples = 0;

    for vals in bins.values() {
        if vals.len() > 1 {
            let mean = vals.iter().sum::<f64>() / vals.len() as f64;
            let variance = vals.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / vals.len() as f64;
            total_weighted_var += variance * (vals.len() as f64);
            total_samples += vals.len();
        }
    }

    if total_samples == 0 { 0.0 } else { total_weighted_var / total_samples as f64 }
}

fn compute_2d_conditional_variance(data: &Vec<Vec<f64>>, idx_c1: usize, idx_c2: usize, idx_val: usize, n1: usize, n2: usize) -> f64 {
    let mut bins: HashMap<(usize, usize), Vec<f64>> = HashMap::new();
    
    let min1 = data.iter().map(|v| v[idx_c1]).fold(f64::INFINITY, f64::min);
    let max1 = data.iter().map(|v| v[idx_c1]).fold(f64::NEG_INFINITY, f64::max);
    let range1 = (max1 - min1).max(1e-6);

    let min2 = data.iter().map(|v| v[idx_c2]).fold(f64::INFINITY, f64::min);
    let max2 = data.iter().map(|v| v[idx_c2]).fold(f64::NEG_INFINITY, f64::max);
    let range2 = (max2 - min2).max(1e-6);

    for row in data {
        let b1 = (((row[idx_c1] - min1) / range1) * (n1 as f64 - 1.0)) as usize;
        let b2 = (((row[idx_c2] - min2) / range2) * (n2 as f64 - 1.0)) as usize;
        bins.entry((b1, b2)).or_default().push(row[idx_val]);
    }

    let mut total_weighted_var = 0.0;
    let mut total_samples = 0;

    for vals in bins.values() {
        if vals.len() > 1 {
            let mean = vals.iter().sum::<f64>() / vals.len() as f64;
            let variance = vals.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / vals.len() as f64;
            total_weighted_var += variance * (vals.len() as f64);
            total_samples += vals.len();
        }
    }

    if total_samples == 0 { 0.0 } else { total_weighted_var / total_samples as f64 }
}
