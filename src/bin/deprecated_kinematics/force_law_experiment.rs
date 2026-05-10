use hcsn_rust::hypergraph::Hypergraph;
use hcsn_rust::observables::compute_omega;
use hcsn_rust::rewrite_engine::RewriteEngine;
use serde::Serialize;
use std::env;
use std::fs;

#[derive(Serialize)]
struct ForceLawStats {
    total_samples: usize,      // Independent A/B points
    total_interactions: usize, // Event count
    chi_bins: Vec<ChiBin>,
    scat_dist: ScatteringDist,
    symmetry_profile: SymmetryProfile,
    best_fit_model: String,
    interpretation: String,
}

#[derive(Serialize)]
struct ChiBin {
    range: String,
    mean_dp_norm: f64,
    var_dp_norm: f64,
    mean_duration: f64,
    count: usize,
}

#[derive(Serialize)]
struct ScatteringDist {
    pass_through: f64, // < 30
    deflection: f64,   // 30-150
    back_scatter: f64, // > 150
}

#[derive(Serialize)]
struct SymmetryProfile {
    symmetric: f64,  // eta < 0.2
    asymmetric: f64, // eta >= 0.2
}

fn dot(a: (f64, f64), b: (f64, f64)) -> f64 {
    a.0 * b.0 + a.1 * b.1
}
fn mag(a: (f64, f64)) -> f64 {
    (a.0 * a.0 + a.1 * a.1).sqrt().max(1e-6)
}

fn main() {
    env::set_var("RAYON_NUM_THREADS", "4");
    println!("=== HCSN CORRECTED FORCE LAW EXTRACTION ===");
    println!("Goal: Extract independent Δp_norm = f(χ) with corrected physics");

    let p_create = 0.69;
    let nu = 0.975;
    let gamma = 2.0;
    let mu = 0.3;

    env::set_var("HCSN_NU", nu.to_string());
    env::set_var("HCSN_GAMMA", gamma.to_string());
    env::set_var("HCSN_MU", mu.to_string());

    let mut h = Hypergraph::new();
    let v1 = h.add_vertex().id;
    let v2 = h.add_vertex().id;
    let v3 = h.add_vertex().id;
    let v4 = h.add_vertex().id;
    let nodes = vec![v1, v2, v3, v4];
    for i in 0..4 {
        for j in i + 1..4 {
            h.add_causal_relation(nodes[i], nodes[j]);
            h.add_hyperedge(vec![nodes[i], nodes[j]]);
        }
    }

    let mut engine = RewriteEngine::new(h, p_create, None);
    engine.pure_mode = false;
    engine.verbose = false;
    engine.print_interval = 2000;

    let total_steps = 60000;
    println!("\nRunning {} steps production simulation...", total_steps);

    for i in 1..=total_steps {
        engine.step();
        if i % 2000 == 0 {
            let active = engine.active_knots.len();
            let omega = compute_omega(&hcsn_rust::observables::worldline_interaction_graph(
                &engine.h, 0.0,
            ));
            println!(
                "  t={} | active_knots={} | Ω={:.4} | interactions_logged={}",
                i,
                active,
                omega,
                engine.interaction_events.len()
            );
        }
    }

    println!("\n=== Extracting Corrected Dataset ===");
    let raw_events = &engine.interaction_events;

    let mut total_samples = 0;
    let mut bin_data: Vec<(f64, f64, String, Vec<f64>, Vec<f64>)> = vec![
        (0.00, 0.05, "0.00-0.05".to_string(), Vec::new(), Vec::new()),
        (0.05, 0.10, "0.05-0.10".to_string(), Vec::new(), Vec::new()),
        (0.10, 0.20, "0.10-0.20".to_string(), Vec::new(), Vec::new()),
        (0.20, 0.30, "0.20-0.30".to_string(), Vec::new(), Vec::new()),
        (0.30, 1.00, "0.30+".to_string(), Vec::new(), Vec::new()),
    ];

    let mut pass = 0;
    let mut defl = 0;
    let mut back = 0;
    let mut sym = 0;
    let mut asym = 0;

    const EPS: f64 = 1e-6;

    for ev in raw_events.iter().filter(|e| e.duration >= 3) {
        let chi = ev.overlap_depth;
        let mut dp_norm_a = -1.0;
        let mut dp_norm_b = -1.0;

        // Process Particle A
        if let Some(post_a) = ev.post_a {
            let p_pre = ev.pre_a.2.abs();
            let p_post = post_a.2.abs();

            if p_pre >= 0.01 {
                let dp_norm = (p_post - p_pre).abs() / (p_pre + p_post + EPS);
                dp_norm_a = dp_norm;

                // Track sample for binning
                for bin in bin_data.iter_mut() {
                    if chi >= bin.0 && chi < bin.1 {
                        bin.3.push(dp_norm);
                        bin.4.push(ev.duration as f64);
                        total_samples += 1;
                        break;
                    }
                }

                // Scattering angle (Counted for A)
                let cos_theta = dot(ev.pre_a.3, post_a.3) / (mag(ev.pre_a.3) * mag(post_a.3));
                let theta = cos_theta.clamp(-1.0, 1.0).acos().to_degrees();
                if theta < 30.0 {
                    pass += 1;
                } else if theta < 150.0 {
                    defl += 1;
                } else {
                    back += 1;
                }
            }
        }

        // Process Particle B
        if let Some(post_b) = ev.post_b {
            let p_pre = ev.pre_b.2.abs();
            let p_post = post_b.2.abs();

            if p_pre >= 0.01 {
                let dp_norm = (p_post - p_pre).abs() / (p_pre + p_post + EPS);
                dp_norm_b = dp_norm;

                // Track sample for binning
                for bin in bin_data.iter_mut() {
                    if chi >= bin.0 && chi < bin.1 {
                        bin.3.push(dp_norm);
                        bin.4.push(ev.duration as f64);
                        total_samples += 1;
                        break;
                    }
                }

                // Scattering angle (Counted for B)
                let cos_theta = dot(ev.pre_b.3, post_b.3) / (mag(ev.pre_b.3) * mag(post_b.3));
                let theta = cos_theta.clamp(-1.0, 1.0).acos().to_degrees();
                if theta < 30.0 {
                    pass += 1;
                } else if theta < 150.0 {
                    defl += 1;
                } else {
                    back += 1;
                }
            }
        }

        // Symmetry η = |Δp_norm_a - Δp_norm_b| / (Δp_norm_a + Δp_norm_b + ε)
        if dp_norm_a >= 0.0 && dp_norm_b >= 0.0 {
            let eta = (dp_norm_a - dp_norm_b).abs() / (dp_norm_a + dp_norm_b + EPS);
            if eta < 0.2 {
                sym += 1;
            } else {
                asym += 1;
            }
        }
    }

    if total_samples == 0 {
        println!("ERROR: No samples survived filters. Check momentum magnitudes.");
        return;
    }

    let mut chi_bins = Vec::new();
    for bin in bin_data {
        let count = bin.3.len();
        if count == 0 {
            chi_bins.push(ChiBin {
                range: bin.2,
                mean_dp_norm: 0.0,
                var_dp_norm: 0.0,
                mean_duration: 0.0,
                count: 0,
            });
            continue;
        }
        let mean_dp = bin.3.iter().sum::<f64>() / count as f64;
        let mean_dur = bin.4.iter().sum::<f64>() / count as f64;
        let var_dp = bin.3.iter().map(|&x| (x - mean_dp).powi(2)).sum::<f64>() / count as f64;
        chi_bins.push(ChiBin {
            range: bin.2,
            mean_dp_norm: mean_dp,
            var_dp_norm: var_dp,
            mean_duration: mean_dur,
            count,
        });
    }

    let total_scat = (pass + defl + back) as f64;
    let scat_dist = ScatteringDist {
        pass_through: pass as f64 / total_scat * 100.0,
        deflection: defl as f64 / total_scat * 100.0,
        back_scatter: back as f64 / total_scat * 100.0,
    };

    let total_sym = (sym + asym) as f64;
    let symmetry_profile = SymmetryProfile {
        symmetric: if total_sym > 0.0 {
            sym as f64 / total_sym * 100.0
        } else {
            0.0
        },
        asymmetric: if total_sym > 0.0 {
            asym as f64 / total_sym * 100.0
        } else {
            0.0
        },
    };

    let best_fit = if chi_bins[0].mean_dp_norm < 0.1 && chi_bins[4].mean_dp_norm > 0.3 {
        "THRESHOLD MODEL (Sigmoid)".to_string()
    } else {
        "SOFT INTERACTION (Gradual)".to_string()
    };

    println!("\n=== Corrected Interaction Result ===");
    println!("| χ Bin | Samples | Mean Δp_norm | Mean Duration (τ) |");
    println!("|-------|---------|--------------|-------------------|");
    for bin in &chi_bins {
        println!(
            "| {} | {} | {:.4} | {:.1} |",
            bin.range, bin.count, bin.mean_dp_norm, bin.mean_duration
        );
    }

    println!("\nScattering Distribution:");
    println!("  Pass-through (<30°): {:.1}%", scat_dist.pass_through);
    println!("  Deflection (30-150°): {:.1}%", scat_dist.deflection);
    println!("  Back-scatter (>150°): {:.1}%", scat_dist.back_scatter);

    println!("\nSymmetry Profile:");
    println!("  Symmetric (η < 0.2):  {:.1}%", symmetry_profile.symmetric);
    println!(
        "  Asymmetric (η >= 0.2): {:.1}%",
        symmetry_profile.asymmetric
    );

    println!("\nFit: {}", best_fit);

    fs::create_dir_all("exports").ok();
    let stats = ForceLawStats {
        total_samples,
        total_interactions: raw_events.len(),
        chi_bins,
        scat_dist,
        symmetry_profile,
        best_fit_model: best_fit,
        interpretation: "Corrected physics extraction complete.".to_string(),
    };
    fs::write(
        "exports/force_law_summary_corrected.json",
        serde_json::to_string_pretty(&stats).unwrap(),
    )
    .unwrap();
    println!("\nSTAGED: Ready for Final Phase Analysis.");
}
