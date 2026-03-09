// SPDX-License-Identifier: AGPL-3.0-or-later
//! Physiologically-Based Pharmacokinetic (PBPK) modeling.
//!
//! Organ-specific compartments connected by blood flow.
//! Each tissue has volume, blood flow, and partition coefficient.
//! Gabrielsson & Weiner, *Pharmacokinetic and Pharmacodynamic Data Analysis*.

/// Tissue compartment parameters for PBPK.
#[derive(Clone, Debug)]
pub struct TissueCompartment {
    pub name: &'static str,
    pub volume_l: f64,
    pub blood_flow_l_per_hr: f64,
    pub kp: f64,                 // tissue-plasma partition coefficient
    pub clearance_l_per_hr: f64, // hepatic/renal clearance (0 for non-eliminating)
}

/// PBPK model state: concentration in each tissue.
#[derive(Clone, Debug)]
pub struct PbpkState {
    pub concentrations: Vec<f64>,
    pub venous_conc: f64,
    pub time_hr: f64,
}

/// Standard human tissue parameters (70 kg adult).
#[must_use]
pub fn standard_human_tissues() -> Vec<TissueCompartment> {
    vec![
        TissueCompartment {
            name: "liver",
            volume_l: 1.5,
            blood_flow_l_per_hr: 90.0,
            kp: 3.0,
            clearance_l_per_hr: 15.0, // hepatic clearance
        },
        TissueCompartment {
            name: "kidney",
            volume_l: 0.31,
            blood_flow_l_per_hr: 72.0,
            kp: 2.5,
            clearance_l_per_hr: 0.0,
        },
        TissueCompartment {
            name: "muscle",
            volume_l: 28.0,
            blood_flow_l_per_hr: 54.0,
            kp: 0.8,
            clearance_l_per_hr: 0.0,
        },
        TissueCompartment {
            name: "fat",
            volume_l: 14.0,
            blood_flow_l_per_hr: 18.0,
            kp: 5.0,
            clearance_l_per_hr: 0.0,
        },
        TissueCompartment {
            name: "rest",
            volume_l: 10.0,
            blood_flow_l_per_hr: 96.0,
            kp: 1.0,
            clearance_l_per_hr: 0.0,
        },
    ]
}

/// Total cardiac output (sum of tissue blood flows).
#[must_use]
pub fn cardiac_output(tissues: &[TissueCompartment]) -> f64 {
    tissues.iter().map(|t| t.blood_flow_l_per_hr).sum()
}

/// Simulate PBPK IV bolus with Euler integration.
///
/// Returns time-concentration profile as `(times, venous_concentrations)`.
/// Initial dose goes into venous blood, distributed immediately to arterial.
#[must_use]
pub fn pbpk_iv_simulate(
    tissues: &[TissueCompartment],
    dose_mg: f64,
    blood_volume_l: f64,
    duration_hr: f64,
    dt: f64,
) -> (Vec<f64>, Vec<f64>, PbpkState) {
    let n_tissues = tissues.len();
    let q_total = cardiac_output(tissues);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "n_steps bounded by duration/dt"
    )]
    let n_steps = (duration_hr / dt) as usize;

    let mut conc = vec![0.0; n_tissues];
    let mut c_venous = dose_mg / blood_volume_l;

    let mut times = Vec::with_capacity(n_steps + 1);
    let mut venous_profile = Vec::with_capacity(n_steps + 1);

    times.push(0.0);
    venous_profile.push(c_venous);

    for step in 1..=n_steps {
        let c_arterial = c_venous;

        // Update each tissue
        for (i, tissue) in tissues.iter().enumerate() {
            let c_free = conc[i] / tissue.kp;
            let uptake = tissue.blood_flow_l_per_hr * (c_arterial - c_free) / tissue.volume_l;
            let elimination = tissue.clearance_l_per_hr * c_free / tissue.volume_l;
            conc[i] += (uptake - elimination) * dt;
            if conc[i] < 0.0 {
                conc[i] = 0.0;
            }
        }

        // Compute venous concentration from tissue efflux
        let mut venous_num = 0.0;
        for (i, tissue) in tissues.iter().enumerate() {
            venous_num += tissue.blood_flow_l_per_hr * conc[i] / tissue.kp;
        }
        c_venous = venous_num / q_total;

        #[expect(clippy::cast_precision_loss, reason = "step bounded by n_steps")]
        let t = (step as f64) * dt;
        times.push(t);
        venous_profile.push(c_venous);
    }

    let state = PbpkState {
        concentrations: conc,
        venous_conc: c_venous,
        time_hr: duration_hr,
    };

    (times, venous_profile, state)
}

/// Per-tissue concentration profiles from a PBPK simulation.
#[derive(Clone, Debug)]
pub struct PbpkTissueProfiles {
    pub times: Vec<f64>,
    pub tissue_names: Vec<String>,
    /// `profiles[tissue_idx][time_idx]`
    pub profiles: Vec<Vec<f64>>,
}

/// Run PBPK IV bolus and collect per-tissue concentration profiles.
///
/// Returns `PbpkTissueProfiles` with down-sampled time series for each
/// tissue (at most `max_points` samples) for visualization.
#[must_use]
pub fn pbpk_iv_tissue_profiles(
    tissues: &[TissueCompartment],
    dose_mg: f64,
    blood_volume_l: f64,
    duration_hr: f64,
    dt: f64,
    max_points: usize,
) -> PbpkTissueProfiles {
    let n_tissues = tissues.len();
    let q_total = cardiac_output(tissues);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "n_steps bounded by duration/dt"
    )]
    let n_steps = (duration_hr / dt) as usize;
    let step_size = (n_steps / max_points).max(1);

    let mut conc = vec![0.0; n_tissues];
    let mut c_venous = dose_mg / blood_volume_l;

    let mut times = Vec::new();
    let mut profiles: Vec<Vec<f64>> = (0..n_tissues).map(|_| Vec::new()).collect();

    for step in 0..=n_steps {
        if step > 0 {
            let c_arterial = c_venous;
            for (i, tissue) in tissues.iter().enumerate() {
                let c_free = conc[i] / tissue.kp;
                let uptake = tissue.blood_flow_l_per_hr * (c_arterial - c_free) / tissue.volume_l;
                let elimination = tissue.clearance_l_per_hr * c_free / tissue.volume_l;
                conc[i] += (uptake - elimination) * dt;
                if conc[i] < 0.0 {
                    conc[i] = 0.0;
                }
            }
            let mut venous_num = 0.0;
            for (i, tissue) in tissues.iter().enumerate() {
                venous_num += tissue.blood_flow_l_per_hr * conc[i] / tissue.kp;
            }
            c_venous = venous_num / q_total;
        }

        if step % step_size == 0 || step == n_steps {
            #[expect(clippy::cast_precision_loss, reason = "step bounded by n_steps")]
            let t = (step as f64) * dt;
            times.push(t);
            for (i, profile) in profiles.iter_mut().enumerate() {
                profile.push(conc[i]);
            }
        }
    }

    PbpkTissueProfiles {
        times,
        tissue_names: tissues.iter().map(|t| t.name.to_string()).collect(),
        profiles,
    }
}

/// Compute AUC from PBPK profile using trapezoidal rule.
#[must_use]
pub fn pbpk_auc(times: &[f64], concentrations: &[f64]) -> f64 {
    super::auc_trapezoidal(times, concentrations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_tissues_have_five_compartments() {
        let tissues = standard_human_tissues();
        assert_eq!(tissues.len(), 5);
    }

    #[test]
    fn cardiac_output_reasonable() {
        let tissues = standard_human_tissues();
        let co = cardiac_output(&tissues);
        assert!(co > 200.0 && co < 400.0, "CO={co} L/hr should be ~330");
    }

    #[test]
    fn pbpk_iv_concentration_decays() {
        let tissues = standard_human_tissues();
        let (_, profile, _) = pbpk_iv_simulate(&tissues, 100.0, 5.0, 24.0, 0.01);
        assert!(
            profile.last().unwrap() < &profile[0],
            "concentration decays"
        );
    }

    #[test]
    fn pbpk_iv_all_nonneg() {
        let tissues = standard_human_tissues();
        let (_, profile, state) = pbpk_iv_simulate(&tissues, 100.0, 5.0, 48.0, 0.01);
        assert!(profile.iter().all(|&c| c >= 0.0), "all venous >= 0");
        assert!(
            state.concentrations.iter().all(|&c| c >= 0.0),
            "all tissue >= 0"
        );
    }

    #[test]
    fn pbpk_auc_positive() {
        let tissues = standard_human_tissues();
        let (times, profile, _) = pbpk_iv_simulate(&tissues, 100.0, 5.0, 48.0, 0.01);
        let auc = pbpk_auc(&times, &profile);
        assert!(auc > 0.0, "AUC must be positive");
    }

    #[test]
    fn pbpk_mass_balance_initial() {
        let tissues = standard_human_tissues();
        let (_, profile, _) = pbpk_iv_simulate(&tissues, 100.0, 5.0, 0.01, 0.001);
        let c0 = 100.0 / 5.0;
        assert!((profile[0] - c0).abs() < 1e-6, "initial C = dose/Vblood");
    }

    #[test]
    fn pbpk_tissue_profiles_correct_dimensions() {
        let tissues = standard_human_tissues();
        let tp = pbpk_iv_tissue_profiles(&tissues, 100.0, 5.0, 24.0, 0.01, 200);
        assert_eq!(tp.tissue_names.len(), 5);
        assert_eq!(tp.profiles.len(), 5);
        for profile in &tp.profiles {
            assert_eq!(profile.len(), tp.times.len());
        }
        assert!(tp.times.len() <= 210, "down-sampled to ~200 points");
    }

    #[test]
    fn pbpk_tissue_profiles_nonneg() {
        let tissues = standard_human_tissues();
        let tp = pbpk_iv_tissue_profiles(&tissues, 100.0, 5.0, 24.0, 0.01, 100);
        for profile in &tp.profiles {
            assert!(profile.iter().all(|&c| c >= 0.0));
        }
    }

    #[test]
    fn pbpk_deterministic() {
        let tissues = standard_human_tissues();
        let (t1, p1, _) = pbpk_iv_simulate(&tissues, 100.0, 5.0, 24.0, 0.01);
        let (t2, p2, _) = pbpk_iv_simulate(&tissues, 100.0, 5.0, 24.0, 0.01);
        for (a, b) in p1.iter().zip(p2.iter()) {
            assert_eq!(a.to_bits(), b.to_bits(), "PBPK must be bit-identical");
        }
    }
}
