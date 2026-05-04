use std::env;

// ============================================================
// Physics parameters (externally controlled, engine-safe)
// ============================================================

#[derive(Debug, Clone)]
pub struct PhysicsParams {
    pub gamma_defect: f64,
    pub inertia_scale: f64,
    pub interaction_boost: f64,
    pub stability_decay: f64,
    pub nonlinear_coupling: f64,
    pub memory_coupling: f64,
    pub noise_bias: f64,
    pub defect_injection: f64,
    pub geometry_freeze: f64,
    pub enable_conservation_patches: bool,
    pub export_mechanisms: bool,
}

impl PhysicsParams {
    pub fn new() -> Self {
        let gamma_defect = env::var("HCSN_GAMMA_DEFECT")
            .unwrap_or_else(|_| "0.15".to_string())
            .parse()
            .unwrap_or(0.15);

        let inertia_scale = env::var("HCSN_INERTIA_SCALE")
            .unwrap_or_else(|_| "1.0".to_string())
            .parse()
            .unwrap_or(1.0);

        let interaction_boost = env::var("HCSN_INTERACTION_BOOST")
            .unwrap_or_else(|_| "1.02".to_string())
            .parse()
            .unwrap_or(1.02);

        let stability_decay = env::var("HCSN_NU")
            .unwrap_or_else(|_| "0.975".to_string())
            .parse()
            .unwrap_or(0.975);

        let nonlinear_coupling = env::var("HCSN_GAMMA")
            .unwrap_or_else(|_| "2.2".to_string())
            .parse()
            .unwrap_or(2.2);

        let memory_coupling = env::var("HCSN_MU")
            .unwrap_or_else(|_| "0.3".to_string())
            .parse()
            .unwrap_or(0.3);

        let defect_injection = env::var("HCSN_DEFECT_INJECTION")
            .unwrap_or_else(|_| "0.0".to_string())
            .parse()
            .unwrap_or(0.0);

        let enable_conservation_patches =
            env::var("HCSN_PATCHES").unwrap_or_else(|_| "1".to_string()) != "0";

        let export_mechanisms =
            env::var("HCSN_EXPORT_MECHANISMS").unwrap_or_else(|_| "0".to_string()) == "1";

        Self {
            gamma_defect,
            inertia_scale,
            interaction_boost,
            stability_decay,
            nonlinear_coupling,
            memory_coupling,
            noise_bias: 0.0,
            defect_injection,
            geometry_freeze: 0.9,
            enable_conservation_patches,
            export_mechanisms,
        }
    }
}
