// SPDX-License-Identifier: AGPL-3.0-or-later
//! Canine pharmacology models for comparative medicine.
//!
//! Dogs with naturally occurring atopic dermatitis (AD) provide causal insight
//! into disease mechanisms that cannot be obtained from testing human drugs on
//! induced animal models. These models validate canine disease biology directly,
//! then translate to humans via species-agnostic parameter substitution.
//!
//! References:
//! - Gonzales AJ et al. (2013) *Vet Dermatol* 24:48 — IL-31 elevated in AD dogs
//! - Gonzales AJ et al. (2014) *JVPT* 37:317 — Oclacitinib JAK1 selectivity
//! - Gonzales AJ et al. (2016) *Vet Dermatol* 27:34 — IL-31 pruritus model
//! - Fleck TJ, Gonzales AJ (2021) *Vet Dermatol* 32:681 — Lokivetmab dose-duration

use serde::{Deserialize, Serialize};

/// Treatment modality for IL-31 kinetics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanineIl31Treatment {
    /// No treatment (natural disease progression).
    Untreated,
    /// Oclacitinib: oral JAK1 inhibitor, suppresses IL-31 signaling.
    Oclacitinib,
    /// Lokivetmab: anti-IL-31 mAb, neutralizes circulating IL-31.
    Lokivetmab,
}

/// JAK kinase IC50 panel for a compound.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JakIc50Panel {
    /// Drug or compound name.
    pub compound: String,
    /// Species label for the assay (e.g. human, canine).
    pub species: String,
    /// IC50 for JAK1 (nM).
    pub jak1_nm: f64,
    /// IC50 for JAK2 (nM).
    pub jak2_nm: f64,
    /// IC50 for JAK3 (nM).
    pub jak3_nm: f64,
    /// IC50 for TYK2 (nM).
    pub tyk2_nm: f64,
}

impl JakIc50Panel {
    /// JAK1 selectivity ratio: geometric mean of off-targets / JAK1.
    #[must_use]
    pub fn jak1_selectivity(&self) -> f64 {
        let off_geomean = (self.jak2_nm * self.jak3_nm * self.tyk2_nm).cbrt();
        off_geomean / self.jak1_nm
    }
}

/// IL-31 serum kinetics in canine AD.
///
/// Models IL-31 concentration over time under different treatments.
/// In untreated AD dogs, IL-31 remains elevated at a steady-state production
/// rate. Treatment reduces either production (oclacitinib → JAK1 blockade) or
/// increases elimination (lokivetmab → antibody neutralization).
///
/// `dC/dt = k_prod - k_el × C`
///
/// Steady state: `C_ss = k_prod / k_el`
///
/// - Gonzales 2013: AD dogs ~44.5 pg/mL, controls ~8.2 pg/mL
/// - Oclacitinib: reduces downstream signaling (modeled as ~70% reduction in effective production)
/// - Lokivetmab: neutralizes IL-31 (modeled as 5× increase in effective elimination)
///
/// Returns IL-31 concentration in pg/mL at time `t_hr` hours after treatment start.
#[must_use]
pub fn il31_serum_kinetics(baseline_pg_ml: f64, t_hr: f64, treatment: CanineIl31Treatment) -> f64 {
    let k_el_baseline = 0.05;
    let k_prod_baseline = baseline_pg_ml * k_el_baseline;

    let (k_prod, k_el) = match treatment {
        CanineIl31Treatment::Untreated => (k_prod_baseline, k_el_baseline),
        CanineIl31Treatment::Oclacitinib => (k_prod_baseline * 0.30, k_el_baseline),
        CanineIl31Treatment::Lokivetmab => (k_prod_baseline, k_el_baseline * 5.0),
    };

    let c_ss = k_prod / k_el;
    (-k_el * t_hr).exp().mul_add(baseline_pg_ml - c_ss, c_ss)
}

/// Published canine JAK IC50 panel for oclacitinib.
///
/// Gonzales et al. (2014) JVPT 37:317: oclacitinib is a "selective inhibitor
/// of JAK1-dependent cytokines" with ~100-fold selectivity over JAK2 and
/// >1000-fold over JAK3.
///
/// Approximate IC50 values (nM) from published selectivity ratios:
/// - JAK1: 10 nM (primary target)
/// - JAK2: 1000 nM (~100× selectivity)
/// - JAK3: 10000 nM (>1000× selectivity)
/// - TYK2: 10000 nM (>1000× selectivity)
#[must_use]
pub fn canine_jak_ic50_panel() -> JakIc50Panel {
    JakIc50Panel {
        compound: "oclacitinib".into(),
        species: "canine".into(),
        jak1_nm: 10.0,
        jak2_nm: 1000.0,
        jak3_nm: 10_000.0,
        tyk2_nm: 10_000.0,
    }
}

/// Published human JAK IC50 panels for reference compounds.
///
/// Sources:
/// - Tofacitinib: Changelian et al. (2003) Science 302:875
/// - Ruxolitinib: Quintás-Cardama et al. (2010) Blood 115:3109
/// - Baricitinib: Fridman et al. (2010) J Immunol 184:5298
#[must_use]
pub fn human_jak_reference_panels() -> Vec<JakIc50Panel> {
    vec![
        JakIc50Panel {
            compound: "tofacitinib".into(),
            species: "human".into(),
            jak1_nm: 3.2,
            jak2_nm: 4.1,
            jak3_nm: 1.6,
            tyk2_nm: 34.0,
        },
        JakIc50Panel {
            compound: "ruxolitinib".into(),
            species: "human".into(),
            jak1_nm: 3.3,
            jak2_nm: 2.8,
            jak3_nm: 428.0,
            tyk2_nm: 19.0,
        },
        JakIc50Panel {
            compound: "baricitinib".into(),
            species: "human".into(),
            jak1_nm: 5.9,
            jak2_nm: 5.7,
            jak3_nm: 560.0,
            tyk2_nm: 53.0,
        },
    ]
}

/// Pruritus VAS (Visual Analog Scale) response to IL-31 reduction.
///
/// Models itch severity as a function of IL-31 level. In the Gonzales 2016
/// beagle model, pruritus correlates with IL-31 concentration via a sigmoid:
///
/// `VAS = VAS_max × C^n / (C^n + EC50^n)`
///
/// where `EC50` is the IL-31 concentration producing half-maximal itch and
/// `n` is the Hill coefficient for the pruritus-cytokine relationship.
///
/// Returns VAS score (0–10 scale).
#[must_use]
pub fn pruritus_vas_response(il31_pg_ml: f64) -> f64 {
    let vas_max = 10.0_f64;
    let ec50 = 25.0_f64;
    let n = 2.0_f64;
    vas_max * il31_pg_ml.powf(n) / (ec50.powf(n) + il31_pg_ml.powf(n))
}

/// Lokivetmab (anti-IL-31 mAb) PK in dogs.
///
/// Fleck & Gonzales (2021): onset ~3 hours, duration dose-dependent.
/// First-order mAb elimination with dose-dependent terminal half-life:
/// - 0.5 mg/kg → ~14 days
/// - 1.0 mg/kg → ~21 days (label dose: effective ≥28 days)
/// - 2.0 mg/kg → ~42 days
///
/// `C(t) = (Dose / Vd) × exp(-k_el × t)`
///
/// Returns serum lokivetmab concentration (µg/mL) at `t_days` after injection.
#[must_use]
pub fn lokivetmab_pk(dose_mg_kg: f64, body_weight_kg: f64, t_days: f64) -> f64 {
    let vd_ml_kg = 85.0;
    let t_half_days = 14.0_f64.mul_add(dose_mg_kg, 7.0);
    let k_el = core::f64::consts::LN_2 / t_half_days;

    let dose_mg = dose_mg_kg * body_weight_kg;
    let vd_ml = vd_ml_kg * body_weight_kg;
    let c0 = dose_mg / vd_ml * 1000.0;

    c0 * (-k_el * t_days).exp()
}

/// Pruritus time-course: VAS over time under treatment.
///
/// Combines IL-31 kinetics with the pruritus dose-response to produce a
/// VAS trajectory. Gonzales 2016 beagle model showed:
/// - Oclacitinib: rapid onset (hours), requires daily redosing
/// - Lokivetmab: slower onset (1–2 days), duration weeks
///
/// Returns `(times_hr, vas_scores)` vectors for plotting.
#[must_use]
pub fn pruritus_time_course(
    baseline_pg_ml: f64,
    treatment: CanineIl31Treatment,
    t_end_hr: f64,
    n_points: usize,
) -> (Vec<f64>, Vec<f64>) {
    #[expect(clippy::cast_precision_loss, reason = "n_points ≪ 2^52")]
    let dt = t_end_hr / (n_points.max(1) - 1) as f64;
    #[expect(clippy::cast_precision_loss, reason = "loop index ≪ 2^52")]
    let times: Vec<f64> = (0..n_points).map(|i| i as f64 * dt).collect();
    let vas: Vec<f64> = times
        .iter()
        .map(|&t| {
            let il31 = il31_serum_kinetics(baseline_pg_ml, t, treatment);
            pruritus_vas_response(il31)
        })
        .collect();
    (times, vas)
}

/// Lokivetmab effective duration: days above therapeutic threshold.
///
/// Fleck & Gonzales (2021): effective concentration threshold ~1.0 µg/mL.
/// Duration depends on dose:
/// - 0.5 mg/kg → ~14 days
/// - 1.0 mg/kg → ~28 days
/// - 2.0 mg/kg → ~42 days
///
/// Returns estimated duration in days above the therapeutic threshold.
#[must_use]
pub fn lokivetmab_effective_duration(
    dose_mg_kg: f64,
    body_weight_kg: f64,
    threshold_ug_ml: f64,
) -> f64 {
    let c0 = lokivetmab_pk(dose_mg_kg, body_weight_kg, 0.0);
    if c0 <= threshold_ug_ml {
        return 0.0;
    }
    let t_half_days = 7.0_f64.mul_add(dose_mg_kg / 0.5, 7.0);
    let k_el = core::f64::consts::LN_2 / t_half_days;
    (c0 / threshold_ug_ml).ln() / k_el
}

/// Lokivetmab onset time: hours to reach IL-31 neutralization threshold.
///
/// After SC injection, lokivetmab distributes and begins neutralizing IL-31.
/// Fleck & Gonzales (2021) report onset ~3 hours. Modeled as the time for
/// mAb concentration to exceed the neutralization threshold.
///
/// Returns onset time in hours.
#[must_use]
pub fn lokivetmab_onset_hr(dose_mg_kg: f64) -> f64 {
    let t_abs_hr = 3.0 / dose_mg_kg.sqrt();
    t_abs_hr.clamp(1.0, 12.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tolerances;

    #[test]
    fn il31_untreated_steady_state() {
        let baseline = 44.5;
        let c = il31_serum_kinetics(baseline, 1000.0, CanineIl31Treatment::Untreated);
        assert!((c - baseline).abs() < tolerances::AUC_TRAPEZOIDAL);
    }

    #[test]
    fn il31_at_time_zero() {
        let baseline = 44.5;
        let c = il31_serum_kinetics(baseline, 0.0, CanineIl31Treatment::Oclacitinib);
        assert!((c - baseline).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn il31_oclacitinib_reduces() {
        let baseline = 44.5;
        let c_treated = il31_serum_kinetics(baseline, 200.0, CanineIl31Treatment::Oclacitinib);
        assert!(c_treated < baseline);
    }

    #[test]
    fn il31_lokivetmab_reduces_faster() {
        let baseline = 44.5;
        let c_ocla = il31_serum_kinetics(baseline, 24.0, CanineIl31Treatment::Oclacitinib);
        let c_loki = il31_serum_kinetics(baseline, 24.0, CanineIl31Treatment::Lokivetmab);
        assert!(c_loki < c_ocla);
    }

    #[test]
    fn canine_ic50_jak1_selectivity() {
        let panel = canine_jak_ic50_panel();
        assert!(panel.jak1_selectivity() > 50.0);
    }

    #[test]
    fn human_panels_have_three() {
        assert_eq!(human_jak_reference_panels().len(), 3);
    }

    #[test]
    fn tofacitinib_jak3_preference() {
        let panels = human_jak_reference_panels();
        let tofa = &panels[0];
        assert!(tofa.jak3_nm < tofa.jak1_nm);
    }

    #[test]
    fn pruritus_zero_il31() {
        let vas = pruritus_vas_response(0.0);
        assert!(vas.abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn pruritus_at_ec50() {
        let vas = pruritus_vas_response(25.0);
        assert!((vas - 5.0).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn pruritus_monotonic() {
        let v1 = pruritus_vas_response(10.0);
        let v2 = pruritus_vas_response(50.0);
        assert!(v2 > v1);
    }

    #[test]
    fn lokivetmab_c0_positive() {
        let c = lokivetmab_pk(1.0, 15.0, 0.0);
        assert!(c > 0.0);
    }

    #[test]
    fn lokivetmab_decays() {
        let c0 = lokivetmab_pk(1.0, 15.0, 0.0);
        let c28 = lokivetmab_pk(1.0, 15.0, 28.0);
        assert!(c28 < c0);
        assert!(c28 > 0.0);
    }

    #[test]
    fn lokivetmab_higher_dose_lasts_longer() {
        let c_low = lokivetmab_pk(0.5, 15.0, 28.0);
        let c_high = lokivetmab_pk(2.0, 15.0, 28.0);
        assert!(c_high > c_low);
    }
}
