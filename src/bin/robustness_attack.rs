use std::fs::File;
use std::io::{BufRead, BufReader};
use std::env;
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct Event {
    chi: f64,
    dp_norm: f64,
    coh: f64,
    stab: f64,
    rad: f64,
    size: f64,
    ratio: f64,
}

struct FitResult {
    slope: f64,
    intercept: f64,
    r2: f64,
}

fn main() {
    println!("=== HCSN BRANCHING LAW ROBUSTNESS ATTACK ===");
    
    let path = env::var("HCSN_IN_FILE")
        .unwrap_or_else(|_| "exports/interaction_points_raw.csv".to_string());
        
    println!("Target Dataset: {}", path);
    let file = File::open(&path).expect("Could not open interaction data file");
    let reader = BufReader::new(file);
    
    let mut original_data = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        if i == 0 { continue; } // skip header
        let l = line.unwrap();
        let parts: Vec<f64> = l.split(',').map(|s| s.parse().unwrap_or(0.0)).collect();
        if parts.len() >= 7 {
            original_data.push(Event {
                chi: parts[0], dp_norm: parts[1], coh: parts[2],
                stab: parts[3], rad: parts[4], size: parts[5], ratio: parts[6],
            });
        }
    }
    
    println!("Total Samples: {}", original_data.len());

    // 1. ORIGINAL FIT
    let original_fit = fit_branching_ratio(&original_data);
    println!("\n[1] ORIGINAL FIT:");
    println!("    Slope:     {:.6}", original_fit.slope);
    println!("    Intercept: {:.6}", original_fit.intercept);
    println!("    R²:        {:.6}", original_fit.r2);

    // 2. SHUFFLE TEST
    let mut shuffled_data = original_data.clone();
    let mut stabilities: Vec<f64> = shuffled_data.iter().map(|e| e.stab).collect();
    let mut rng = rand::thread_rng();
    stabilities.shuffle(&mut rng);
    for (i, e) in shuffled_data.iter_mut().enumerate() {
        e.stab = stabilities[i];
    }
    let shuffled_fit = fit_branching_ratio(&shuffled_data);
    println!("\n[2] SHUFFLE TEST (S randomly permuted):");
    println!("    Slope:     {:.6}", shuffled_fit.slope);
    println!("    R²:        {:.6}", shuffled_fit.r2);

    // 3. NOISE TEST
    let mut noise_data = original_data.clone();
    let min_s = original_data.iter().map(|e| e.stab).fold(f64::INFINITY, f64::min);
    let max_s = original_data.iter().map(|e| e.stab).fold(f64::NEG_INFINITY, f64::max);
    for e in noise_data.iter_mut() {
        e.stab = rng.gen_range(min_s..max_s);
    }
    let noise_fit = fit_branching_ratio(&noise_data);
    println!("\n[3] NOISE TEST (S replaced by random Uniform):");
    println!("    Slope:     {:.6}", noise_fit.slope);
    println!("    R²:        {:.6}", noise_fit.r2);

    // 4. BOOTSTRAP TEST (70% sample x 5)
    println!("\n[4] BOOTSTRAP TEST (5x 70% sub-samples):");
    let mut slopes = Vec::new();
    for i in 0..5 {
        let mut subset = Vec::new();
        let n_subset = (original_data.len() as f64 * 0.7) as usize;
        let mut indices: Vec<usize> = (0..original_data.len()).collect();
        indices.shuffle(&mut rng);
        for &idx in indices.iter().take(n_subset) {
            subset.push(original_data[idx].clone());
        }
        let f = fit_branching_ratio(&subset);
        slopes.push(f.slope);
        println!("    Iter {}: Slope = {:.6} (R²={:.4})", i, f.slope, f.r2);
    }
    
    let mean_slope: f64 = slopes.iter().sum::<f64>() / slopes.len() as f64;
    let var_slope: f64 = slopes.iter().map(|s| (s - mean_slope).powi(2)).sum::<f64>() / slopes.len() as f64;
    println!("    Mean Slope: {:.6} ± {:.6}", mean_slope, var_slope.sqrt());

    // VERDICT
    println!("\n=== FINAL VERDICT ===");
    let slope_collapsed = shuffled_fit.slope.abs() < original_fit.slope.abs() * 0.1;
    let r2_collapsed = shuffled_fit.r2 < original_fit.r2 * 0.2;
    let bootstrap_stable = (var_slope.sqrt() / mean_slope.abs()) < 0.15;

    if slope_collapsed && r2_collapsed && bootstrap_stable {
        println!(">>> RESULT: REAL SIGNAL (Robustness Verified) <<<");
        println!("The anti-correlation between Stability and Reflection is non-random.");
    } else {
        println!(">>> RESULT: POTENTIAL ARTIFACT <<<");
        if !slope_collapsed { println!("- Shuffle test did not collapse slope."); }
        if !r2_collapsed { println!("- Shuffle test did not collapse R²."); }
        if !bootstrap_stable { println!("- Bootstrap variance is high: fit is unstable."); }
    }
}

fn fit_branching_ratio(data: &Vec<Event>) -> FitResult {
    if data.len() < 20 { return FitResult { slope: 0.0, intercept: 0.0, r2: 0.0 }; }
    
    // 1. Equal-N Binning (Adaptive)
    let mut sorted_data = data.clone();
    sorted_data.sort_by(|a, b| a.stab.partial_cmp(&b.stab).unwrap_or(std::cmp::Ordering::Equal));
    
    let n_bins = 10;
    let samples_per_bin = sorted_data.len() / n_bins;
    
    let mut x_points = Vec::new();
    let mut y_points = Vec::new();
    
    for i in 0..n_bins {
        let start = i * samples_per_bin;
        let end = if i == n_bins - 1 { sorted_data.len() } else { (i + 1) * samples_per_bin };
        let chunk = &sorted_data[start..end];
        
        let mean_s = chunk.iter().map(|e| e.stab).sum::<f64>() / chunk.len() as f64;
        let n_drag = chunk.iter().filter(|e| e.dp_norm > 0.1).count();
        let p_drag = n_drag as f64 / chunk.len() as f64;
        
        x_points.push(mean_s);
        y_points.push(p_drag);
    }

    if x_points.len() < 2 {
        return FitResult { slope: 0.0, intercept: 0.0, r2: 0.0 };
    }

    // Linear Regression (OLS)
    let n = x_points.len() as f64;
    let mean_x = x_points.iter().sum::<f64>() / n;
    let mean_y = y_points.iter().sum::<f64>() / n;

    let mut ss_xx = 0.0;
    let mut ss_xy = 0.0;
    let mut ss_yy = 0.0;

    for i in 0..x_points.len() {
        let dx = x_points[i] - mean_x;
        let dy = y_points[i] - mean_y;
        ss_xx += dx * dx;
        ss_xy += dx * dy;
        ss_yy += dy * dy;
    }

    let b = ss_xy / ss_xx;
    let a = mean_y - b * mean_x;
    let r2 = (ss_xy * ss_xy) / (ss_xx * ss_yy);

    FitResult { slope: b, intercept: a, r2 }
}
