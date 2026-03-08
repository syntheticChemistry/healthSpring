// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integrated diagnostic pipeline composing all four healthSpring tracks.
//!
//! Takes a patient profile and runs PK/PD modeling, microbiome risk assessment,
//! biosignal analysis, and endocrine outcome prediction. Cross-track models
//! (gut-TRT axis, HRV-TRT cardiac) integrate the results. Population Monte Carlo
//! places the patient within a virtual cohort for percentile context.

use crate::{biosignal, endocrine, microbiome, pkpd};

/// Biological sex for allometric scaling and reference ranges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sex {
    Male,
    Female,
}

/// Input patient profile — all fields optional except demographics.
#[derive(Debug, Clone)]
pub struct PatientProfile {
    pub age_years: f64,
    pub weight_kg: f64,
    pub sex: Sex,
    pub testosterone_ng_dl: Option<f64>,
    pub on_trt: bool,
    pub trt_months: f64,
    pub gut_abundances: Option<Vec<f64>>,
    pub ecg_peaks: Option<Vec<usize>>,
    pub ecg_fs: f64,
    pub ppg_spo2: Option<f64>,
    pub scr_count: Option<usize>,
    pub eda_duration_s: f64,
}

impl PatientProfile {
    #[must_use]
    pub fn minimal(age_years: f64, weight_kg: f64, sex: Sex) -> Self {
        Self {
            age_years,
            weight_kg,
            sex,
            testosterone_ng_dl: None,
            on_trt: false,
            trt_months: 0.0,
            gut_abundances: None,
            ecg_peaks: None,
            ecg_fs: 360.0,
            ppg_spo2: None,
            scr_count: None,
            eda_duration_s: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PkAssessment {
    pub hill_response_at_ec50: f64,
    pub oral_cmax: f64,
    pub oral_tmax_hr: f64,
    pub oral_auc: f64,
    pub allometric_cl: f64,
    pub allometric_vd: f64,
}

#[derive(Debug, Clone)]
pub struct MicrobiomeAssessment {
    pub shannon: f64,
    pub simpson: f64,
    pub pielou_evenness: f64,
    pub colonization_resistance: f64,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone)]
pub struct BiosignalAssessment {
    pub heart_rate_bpm: f64,
    pub sdnn_ms: f64,
    pub rmssd_ms: f64,
    pub spo2_percent: f64,
    pub stress_index: f64,
    pub overall_score: f64,
}

#[derive(Debug, Clone)]
pub struct EndocrineAssessment {
    pub predicted_testosterone: f64,
    pub age_decline_rate: f64,
    pub hrv_trt_sdnn: f64,
    pub cardiac_risk: f64,
    pub metabolic_response: f64,
}

#[derive(Debug, Clone)]
pub struct CrossTrackAssessment {
    pub gut_trt_response: f64,
    pub hrv_cardiac_composite: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Moderate,
    High,
    Critical,
}

/// Full diagnostic result composing all tracks.
#[derive(Debug, Clone)]
pub struct DiagnosticAssessment {
    pub pk: PkAssessment,
    pub microbiome: MicrobiomeAssessment,
    pub biosignal: BiosignalAssessment,
    pub endocrine: EndocrineAssessment,
    pub cross_track: CrossTrackAssessment,
    pub composite_risk: f64,
}

/// Population Monte Carlo result.
#[derive(Debug, Clone)]
pub struct PopulationResult {
    pub n_patients: usize,
    pub composite_risks: Vec<f64>,
    pub mean_risk: f64,
    pub std_risk: f64,
    pub patient_percentile: f64,
}

const REF_BW_KG: f64 = 70.0;
const REF_CL_L_HR: f64 = 0.15;
const REF_VD_L: f64 = 15.0;
const DEFAULT_F: f64 = 0.79;
const DEFAULT_KA: f64 = 1.5;
const DEFAULT_DOSE_MG: f64 = 4.0;
const T_DECLINE_RATE: f64 = 0.016;
const T_DECLINE_ONSET: f64 = 30.0;
const DEFAULT_T0_MALE: f64 = 600.0;
const DEFAULT_T0_FEMALE: f64 = 40.0;
const SDNN_DELTA_TRT: f64 = 15.0;
const SDNN_TAU_MONTHS: f64 = 6.0;
const CARDIAC_BASELINE_RISK: f64 = 0.1;
const TRT_GUT_XI_MAX: f64 = 100.0;
const TRT_GUT_BASE_RESPONSE: f64 = 0.8;
const DEFAULT_EMAX: f64 = 100.0;
const DEFAULT_EC50: f64 = 10.0;
const DEFAULT_HILL_N: f64 = 1.5;
const GUT_W_SCALE: f64 = 8.0;

fn assess_pk(profile: &PatientProfile) -> PkAssessment {
    let cl = pkpd::allometric_scale(REF_CL_L_HR, REF_BW_KG, profile.weight_kg, 0.75);
    let vd = pkpd::allometric_scale(REF_VD_L, REF_BW_KG, profile.weight_kg, 1.0);
    let ke = cl / vd;

    let hill_at_ec50 =
        pkpd::hill_dose_response(DEFAULT_EC50, DEFAULT_EC50, DEFAULT_HILL_N, DEFAULT_EMAX);

    let n_points = 100;
    let t_max = 24.0;
    let dt = t_max / f64::from(n_points);
    let times: Vec<f64> = (0..=n_points).map(|i| f64::from(i) * dt).collect();
    let concs: Vec<f64> = times
        .iter()
        .map(|&t| pkpd::pk_oral_one_compartment(DEFAULT_DOSE_MG, DEFAULT_F, vd, DEFAULT_KA, ke, t))
        .collect();

    let (cmax, tmax_hr) = pkpd::find_cmax_tmax(&times, &concs);
    let auc = pkpd::auc_trapezoidal(&times, &concs);

    PkAssessment {
        hill_response_at_ec50: hill_at_ec50,
        oral_cmax: cmax,
        oral_tmax_hr: tmax_hr,
        oral_auc: auc,
        allometric_cl: cl,
        allometric_vd: vd,
    }
}

fn assess_microbiome(profile: &PatientProfile) -> MicrobiomeAssessment {
    let abundances = profile
        .gut_abundances
        .as_deref()
        .unwrap_or(&[0.4, 0.3, 0.15, 0.1, 0.05]);

    let shannon = microbiome::shannon_index(abundances);
    let simpson = microbiome::simpson_index(abundances);
    let evenness = microbiome::pielou_evenness(abundances);

    let disorder = microbiome::evenness_to_disorder(evenness, GUT_W_SCALE);
    let n_sites = 50;
    let disorder_field: Vec<f64> = (0..n_sites)
        .map(|i| disorder * (1.0 + 0.1 * ((f64::from(i) * 0.7).sin())))
        .collect();
    let eigenvalues = microbiome::anderson_hamiltonian_1d(&disorder_field, 1.0);

    let mid = eigenvalues.len() / 2;
    let mid_psi = &eigenvalues[mid.saturating_sub(1)..=(mid).min(eigenvalues.len() - 1)];
    let ipr = if mid_psi.is_empty() {
        1.0
    } else {
        microbiome::inverse_participation_ratio(mid_psi)
    };
    let xi = microbiome::localization_length_from_ipr(ipr);
    let resistance = microbiome::colonization_resistance(xi);

    let risk_level = if resistance > 0.7 {
        RiskLevel::Low
    } else if resistance > 0.4 {
        RiskLevel::Moderate
    } else if resistance > 0.2 {
        RiskLevel::High
    } else {
        RiskLevel::Critical
    };

    MicrobiomeAssessment {
        shannon,
        simpson,
        pielou_evenness: evenness,
        colonization_resistance: resistance,
        risk_level,
    }
}

fn assess_biosignal(profile: &PatientProfile) -> BiosignalAssessment {
    if let Some(ref peaks) = profile.ecg_peaks {
        if peaks.len() >= 3 {
            let spo2 = profile.ppg_spo2.unwrap_or(97.0);
            let scr = profile.scr_count.unwrap_or(0);
            let fused =
                biosignal::fuse_channels(peaks, profile.ecg_fs, spo2, scr, profile.eda_duration_s);
            return BiosignalAssessment {
                heart_rate_bpm: fused.heart_rate_bpm,
                sdnn_ms: fused.hrv_sdnn_ms,
                rmssd_ms: fused.hrv_rmssd_ms,
                spo2_percent: fused.spo2_percent,
                stress_index: fused.stress_index,
                overall_score: fused.overall_score,
            };
        }
    }

    BiosignalAssessment {
        heart_rate_bpm: 72.0,
        sdnn_ms: 50.0,
        rmssd_ms: 35.0,
        spo2_percent: profile.ppg_spo2.unwrap_or(97.0),
        stress_index: 0.3,
        overall_score: 70.0,
    }
}

fn assess_endocrine(profile: &PatientProfile) -> EndocrineAssessment {
    let baseline_t = match profile.sex {
        Sex::Male => DEFAULT_T0_MALE,
        Sex::Female => DEFAULT_T0_FEMALE,
    };

    let predicted_t = profile.testosterone_ng_dl.unwrap_or_else(|| {
        endocrine::testosterone_decline(
            baseline_t,
            T_DECLINE_RATE,
            profile.age_years,
            T_DECLINE_ONSET,
        )
    });

    let sdnn_base = 50.0;
    let hrv_sdnn = if profile.on_trt {
        endocrine::hrv_trt_response(
            sdnn_base,
            SDNN_DELTA_TRT,
            SDNN_TAU_MONTHS,
            profile.trt_months,
        )
    } else {
        sdnn_base
    };

    let cardiac = endocrine::cardiac_risk_composite(hrv_sdnn, predicted_t, CARDIAC_BASELINE_RISK);

    let metabolic = if profile.on_trt {
        endocrine::weight_trajectory(profile.trt_months, -12.0, 12.0, 60.0)
    } else {
        0.0
    };

    EndocrineAssessment {
        predicted_testosterone: predicted_t,
        age_decline_rate: T_DECLINE_RATE,
        hrv_trt_sdnn: hrv_sdnn,
        cardiac_risk: cardiac,
        metabolic_response: metabolic,
    }
}

fn assess_cross_track(
    microbiome: &MicrobiomeAssessment,
    endocrine: &EndocrineAssessment,
    biosignal: &BiosignalAssessment,
) -> CrossTrackAssessment {
    let disorder = endocrine::evenness_to_disorder(microbiome.pielou_evenness, GUT_W_SCALE);
    let xi = endocrine::anderson_localization_length(disorder, 50.0);
    let gut_response = endocrine::gut_metabolic_response(xi, TRT_GUT_XI_MAX, TRT_GUT_BASE_RESPONSE);

    let hrv_cardiac = endocrine::cardiac_risk_composite(
        biosignal.sdnn_ms,
        endocrine.predicted_testosterone,
        CARDIAC_BASELINE_RISK,
    );

    CrossTrackAssessment {
        gut_trt_response: gut_response,
        hrv_cardiac_composite: hrv_cardiac,
    }
}

fn composite_risk(
    microbiome: &MicrobiomeAssessment,
    biosignal: &BiosignalAssessment,
    endocrine: &EndocrineAssessment,
    cross_track: &CrossTrackAssessment,
) -> f64 {
    let micro_risk = 1.0 - microbiome.colonization_resistance;
    let bio_risk = biosignal.stress_index;
    let endo_risk = endocrine.cardiac_risk;
    let cross_risk = cross_track.hrv_cardiac_composite;

    (0.25 * micro_risk + 0.25 * bio_risk + 0.25 * endo_risk + 0.25 * cross_risk).clamp(0.0, 1.0)
}

/// Run the full diagnostic pipeline for a single patient.
#[must_use]
pub fn assess_patient(profile: &PatientProfile) -> DiagnosticAssessment {
    let pk = assess_pk(profile);
    let microbiome = assess_microbiome(profile);
    let biosignal = assess_biosignal(profile);
    let endocrine = assess_endocrine(profile);
    let cross_track = assess_cross_track(&microbiome, &endocrine, &biosignal);
    let risk = composite_risk(&microbiome, &biosignal, &endocrine, &cross_track);

    DiagnosticAssessment {
        pk,
        microbiome,
        biosignal,
        endocrine,
        cross_track,
        composite_risk: risk,
    }
}

/// Run population Monte Carlo: generate `n_patients` virtual patients with
/// lognormal inter-individual variability around the base profile, assess each,
/// and compute where the base patient falls in the distribution.
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "patient indices fit f64")]
pub fn population_montecarlo(
    base_profile: &PatientProfile,
    n_patients: usize,
    seed: u64,
) -> PopulationResult {
    let base_assessment = assess_patient(base_profile);
    let base_risk = base_assessment.composite_risk;

    let mut rng = seed;
    let mut risks = Vec::with_capacity(n_patients);

    for _ in 0..n_patients {
        rng = rng.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let u1 = (rng >> 33) as f64 / (1u64 << 31) as f64;
        rng = rng.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let u2 = (rng >> 33) as f64 / (1u64 << 31) as f64;

        let u1_safe = u1.max(1e-10);
        let z = (-2.0 * u1_safe.ln()).sqrt() * (std::f64::consts::TAU * u2).cos();

        let age_var = (profile_cv_lognormal(base_profile.age_years, 0.15, z)).max(18.0);
        let weight_var = profile_cv_lognormal(base_profile.weight_kg, 0.20, z).max(30.0);

        rng = rng.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let z2_u1 = ((rng >> 33) as f64 / (1u64 << 31) as f64).max(1e-10);
        rng = rng.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let z2_u2 = (rng >> 33) as f64 / (1u64 << 31) as f64;
        let z2 = (-2.0 * z2_u1.ln()).sqrt() * (std::f64::consts::TAU * z2_u2).cos();

        let t_var = base_profile
            .testosterone_ng_dl
            .map(|t| profile_cv_lognormal(t, 0.30, z2).max(10.0));

        let virtual_profile = PatientProfile {
            age_years: age_var,
            weight_kg: weight_var,
            testosterone_ng_dl: t_var,
            ..base_profile.clone()
        };

        let assessment = assess_patient(&virtual_profile);
        risks.push(assessment.composite_risk);
    }

    let mean = risks.iter().sum::<f64>() / n_patients as f64;
    let variance = risks.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n_patients as f64;
    let std = variance.sqrt();

    let below_count = risks.iter().filter(|&&r| r <= base_risk).count();
    let percentile = below_count as f64 / n_patients as f64 * 100.0;

    PopulationResult {
        n_patients,
        composite_risks: risks,
        mean_risk: mean,
        std_risk: std,
        patient_percentile: percentile,
    }
}

fn profile_cv_lognormal(typical: f64, cv: f64, z: f64) -> f64 {
    let (mu, sigma) = endocrine::lognormal_params(typical, cv);
    (mu + sigma * z).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_profile() -> PatientProfile {
        let mut p = PatientProfile::minimal(55.0, 85.0, Sex::Male);
        p.testosterone_ng_dl = Some(450.0);
        p.on_trt = true;
        p.trt_months = 12.0;
        p.gut_abundances = Some(vec![0.35, 0.25, 0.20, 0.10, 0.05, 0.03, 0.02]);
        p
    }

    #[test]
    fn assess_patient_produces_valid_output() {
        let result = assess_patient(&test_profile());

        assert!(result.pk.oral_cmax > 0.0);
        assert!(result.pk.oral_auc > 0.0);
        assert!(result.pk.hill_response_at_ec50 > 0.0);

        assert!(result.microbiome.shannon > 0.0);
        assert!(result.microbiome.colonization_resistance >= 0.0);

        assert!(result.biosignal.heart_rate_bpm > 0.0);
        assert!(result.biosignal.spo2_percent > 0.0);

        assert!(result.endocrine.predicted_testosterone > 0.0);
        assert!(result.endocrine.hrv_trt_sdnn > 50.0);

        assert!((0.0..=1.0).contains(&result.composite_risk));
    }

    #[test]
    fn pk_assessment_allometric_scaling() {
        let light = PatientProfile::minimal(40.0, 50.0, Sex::Female);
        let heavy = PatientProfile::minimal(40.0, 100.0, Sex::Male);

        let pk_light = assess_pk(&light);
        let pk_heavy = assess_pk(&heavy);

        assert!(pk_heavy.allometric_cl > pk_light.allometric_cl);
        assert!(pk_heavy.allometric_vd > pk_light.allometric_vd);
    }

    #[test]
    fn microbiome_risk_levels() {
        let healthy = vec![0.20, 0.18, 0.15, 0.12, 0.10, 0.08, 0.07, 0.05, 0.03, 0.02];
        let dysbiotic = vec![0.90, 0.05, 0.03, 0.02];

        let mut p = PatientProfile::minimal(40.0, 70.0, Sex::Male);
        p.gut_abundances = Some(healthy);
        let a1 = assess_microbiome(&p);

        p.gut_abundances = Some(dysbiotic);
        let a2 = assess_microbiome(&p);

        assert!(a1.shannon > a2.shannon);
        assert!(a1.pielou_evenness > a2.pielou_evenness);
    }

    #[test]
    fn trt_improves_endocrine_metrics() {
        let mut off_trt = PatientProfile::minimal(55.0, 85.0, Sex::Male);
        off_trt.testosterone_ng_dl = Some(300.0);

        let mut on_trt = off_trt.clone();
        on_trt.on_trt = true;
        on_trt.trt_months = 24.0;

        let e_off = assess_endocrine(&off_trt);
        let e_on = assess_endocrine(&on_trt);

        assert!(e_on.hrv_trt_sdnn > e_off.hrv_trt_sdnn);
        assert!(e_on.metabolic_response < 0.0);
    }

    #[test]
    fn cross_track_gut_response_positive() {
        let result = assess_patient(&test_profile());
        assert!(result.cross_track.gut_trt_response > 0.0);
        assert!(result.cross_track.hrv_cardiac_composite >= 0.0);
    }

    #[test]
    fn population_montecarlo_distribution() {
        let result = population_montecarlo(&test_profile(), 500, 42);

        assert_eq!(result.n_patients, 500);
        assert_eq!(result.composite_risks.len(), 500);
        assert!(result.mean_risk > 0.0);
        assert!(result.std_risk > 0.0);
        assert!((0.0..=100.0).contains(&result.patient_percentile));
    }

    #[test]
    fn population_deterministic() {
        let p = test_profile();
        let r1 = population_montecarlo(&p, 200, 99);
        let r2 = population_montecarlo(&p, 200, 99);
        assert_eq!(r1.mean_risk, r2.mean_risk);
        assert_eq!(r1.patient_percentile, r2.patient_percentile);
    }

    #[test]
    fn minimal_profile_no_panic() {
        let p = PatientProfile::minimal(30.0, 70.0, Sex::Female);
        let result = assess_patient(&p);
        assert!((0.0..=1.0).contains(&result.composite_risk));
    }
}
