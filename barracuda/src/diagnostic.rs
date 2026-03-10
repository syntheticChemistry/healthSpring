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
    /// Full 0..24h concentration-time curve (101 points).
    pub curve_times_hr: Vec<f64>,
    pub curve_concs_mg_l: Vec<f64>,
    /// Hill dose-response sweep (50 points, log-spaced).
    pub hill_concs: Vec<f64>,
    pub hill_responses: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct MicrobiomeAssessment {
    pub shannon: f64,
    pub simpson: f64,
    pub pielou_evenness: f64,
    pub colonization_resistance: f64,
    pub risk_level: RiskLevel,
    /// Raw genus-level relative abundances (passthrough from profile).
    pub abundances: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct BiosignalAssessment {
    pub heart_rate_bpm: f64,
    pub sdnn_ms: f64,
    pub rmssd_ms: f64,
    pub spo2_percent: f64,
    pub stress_index: f64,
    pub overall_score: f64,
    /// RR intervals in ms (when ECG peaks available).
    pub rr_intervals_ms: Vec<f64>,
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

/// Configurable pipeline parameters. All values have documented defaults
/// sourced from published literature. Callers can override any parameter
/// to adapt the pipeline to different drugs, populations, or clinical
/// protocols without modifying library code.
#[derive(Debug, Clone)]
pub struct DiagnosticConfig {
    // PK/PD
    pub ref_bw_kg: f64,
    pub ref_cl_l_hr: f64,
    pub ref_vd_l: f64,
    pub bioavailability: f64,
    pub absorption_rate: f64,
    pub dose_mg: f64,
    pub emax: f64,
    pub ec50: f64,
    pub hill_n: f64,
    pub pk_curve_points: u32,
    pub pk_curve_hours: f64,
    pub hill_sweep_points: u32,

    // Endocrine
    pub t_decline_rate: f64,
    pub t_decline_onset_years: f64,
    pub baseline_t_male: f64,
    pub baseline_t_female: f64,
    pub sdnn_delta_trt: f64,
    pub sdnn_tau_months: f64,
    pub cardiac_baseline_risk: f64,

    // Cross-track
    pub gut_xi_max: f64,
    pub gut_base_response: f64,
    pub gut_w_scale: f64,
    pub anderson_lattice_sites: usize,

    // Biosignal fallback (used when ECG data insufficient)
    pub fallback_hr_bpm: f64,
    pub fallback_sdnn_ms: f64,
    pub fallback_rmssd_ms: f64,
    pub fallback_spo2: f64,
    pub fallback_stress: f64,
    pub fallback_overall_score: f64,

    // Microbiome risk thresholds
    pub resistance_low_threshold: f64,
    pub resistance_moderate_threshold: f64,
    pub resistance_high_threshold: f64,

    // Composite risk weights
    pub weight_microbiome: f64,
    pub weight_biosignal: f64,
    pub weight_endocrine: f64,
    pub weight_cross_track: f64,

    // Population Monte Carlo IIV
    pub mc_age_cv: f64,
    pub mc_weight_cv: f64,
    pub mc_testosterone_cv: f64,
}

impl Default for DiagnosticConfig {
    /// Published-literature defaults. Sources documented in whitePaper/baseCamp/.
    fn default() -> Self {
        Self {
            // Rowland & Tozer Ch. 3 — reference adult oral PK
            ref_bw_kg: 70.0,
            ref_cl_l_hr: 0.15,
            ref_vd_l: 15.0,
            bioavailability: 0.79,
            absorption_rate: 1.5,
            dose_mg: 4.0,
            emax: 100.0,
            ec50: 10.0,
            hill_n: 1.5,
            pk_curve_points: 100,
            pk_curve_hours: 24.0,
            hill_sweep_points: 50,

            // Harman 2001 (BLSA, n=890)
            t_decline_rate: 0.016,
            t_decline_onset_years: 30.0,
            baseline_t_male: 600.0,
            baseline_t_female: 40.0,

            // Task Force 1996, Mok Ch. 6
            sdnn_delta_trt: 15.0,
            sdnn_tau_months: 6.0,
            cardiac_baseline_risk: 0.1,

            // wetSpring Anderson extension
            gut_xi_max: 100.0,
            gut_base_response: 0.8,
            gut_w_scale: 8.0,
            anderson_lattice_sites: 50,

            // NICE clinical defaults when ECG is absent
            fallback_hr_bpm: 72.0,
            fallback_sdnn_ms: 50.0,
            fallback_rmssd_ms: 35.0,
            fallback_spo2: 97.0,
            fallback_stress: 0.3,
            fallback_overall_score: 70.0,

            // Jenior 2021 colonization resistance thresholds
            resistance_low_threshold: 0.7,
            resistance_moderate_threshold: 0.4,
            resistance_high_threshold: 0.2,

            // Equal weighting (default)
            weight_microbiome: 0.25,
            weight_biosignal: 0.25,
            weight_endocrine: 0.25,
            weight_cross_track: 0.25,

            // Mould & Upton 2013 typical IIV
            mc_age_cv: 0.15,
            mc_weight_cv: 0.20,
            mc_testosterone_cv: 0.30,
        }
    }
}

impl DiagnosticConfig {
    /// Default configuration sourced from published literature.
    #[must_use]
    pub fn published() -> Self {
        Self::default()
    }
}

fn assess_pk(profile: &PatientProfile, cfg: &DiagnosticConfig) -> PkAssessment {
    let cl = pkpd::allometric_scale(cfg.ref_cl_l_hr, cfg.ref_bw_kg, profile.weight_kg, 0.75);
    let vd = pkpd::allometric_scale(cfg.ref_vd_l, cfg.ref_bw_kg, profile.weight_kg, 1.0);
    let ke = cl / vd;

    let hill_at_ec50 = pkpd::hill_dose_response(cfg.ec50, cfg.ec50, cfg.hill_n, cfg.emax);

    let n_points = cfg.pk_curve_points;
    let t_max = cfg.pk_curve_hours;
    let dt = t_max / f64::from(n_points);
    let times: Vec<f64> = (0..=n_points).map(|i| f64::from(i) * dt).collect();
    let concs: Vec<f64> = times
        .iter()
        .map(|&t| {
            pkpd::pk_oral_one_compartment(
                cfg.dose_mg,
                cfg.bioavailability,
                vd,
                cfg.absorption_rate,
                ke,
                t,
            )
        })
        .collect();

    let (cmax, tmax_hr) = pkpd::find_cmax_tmax(&times, &concs);
    let auc = pkpd::auc_trapezoidal(&times, &concs);

    let hill_n_pts = cfg.hill_sweep_points;
    let hill_concs: Vec<f64> = (0..hill_n_pts)
        .map(|i| {
            let frac = f64::from(i) / f64::from(hill_n_pts - 1);
            0.1 * (cfg.ec50 * 100.0_f64).powf(frac)
        })
        .collect();
    let hill_responses = pkpd::hill_sweep(cfg.ec50, cfg.hill_n, cfg.emax, &hill_concs);

    PkAssessment {
        hill_response_at_ec50: hill_at_ec50,
        oral_cmax: cmax,
        oral_tmax_hr: tmax_hr,
        oral_auc: auc,
        allometric_cl: cl,
        allometric_vd: vd,
        curve_times_hr: times,
        curve_concs_mg_l: concs,
        hill_concs,
        hill_responses,
    }
}

fn assess_microbiome(profile: &PatientProfile, cfg: &DiagnosticConfig) -> MicrobiomeAssessment {
    let abundances = profile
        .gut_abundances
        .as_deref()
        .unwrap_or(&[0.4, 0.3, 0.15, 0.1, 0.05]);

    let shannon = microbiome::shannon_index(abundances);
    let simpson = microbiome::simpson_index(abundances);
    let evenness = microbiome::pielou_evenness(abundances);

    let disorder = microbiome::evenness_to_disorder(evenness, cfg.gut_w_scale);
    let n_sites = cfg.anderson_lattice_sites;
    let disorder_field: Vec<f64> = (0..n_sites)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "lattice index fits f64")]
            let fi = i as f64;
            disorder * (1.0 + 0.1 * ((fi * 0.7).sin()))
        })
        .collect();
    let (_eigenvalues, eigenvectors) = microbiome::anderson_diagonalize(&disorder_field, 1.0);

    let mid = n_sites / 2;
    let mid_psi = &eigenvectors[mid * n_sites..(mid + 1) * n_sites];
    let ipr = microbiome::inverse_participation_ratio(mid_psi);
    let xi = microbiome::localization_length_from_ipr(ipr);
    let resistance = microbiome::colonization_resistance(xi);

    let risk_level = if resistance > cfg.resistance_low_threshold {
        RiskLevel::Low
    } else if resistance > cfg.resistance_moderate_threshold {
        RiskLevel::Moderate
    } else if resistance > cfg.resistance_high_threshold {
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
        abundances: abundances.to_vec(),
    }
}

fn assess_biosignal(profile: &PatientProfile, cfg: &DiagnosticConfig) -> BiosignalAssessment {
    if let Some(ref peaks) = profile.ecg_peaks {
        if peaks.len() >= 3 {
            let spo2 = profile.ppg_spo2.unwrap_or(97.0);
            let scr = profile.scr_count.unwrap_or(0);
            let fused =
                biosignal::fuse_channels(peaks, profile.ecg_fs, spo2, scr, profile.eda_duration_s);
            let rr: Vec<f64> = peaks
                .windows(2)
                .map(|w| {
                    #[expect(
                        clippy::cast_precision_loss,
                        reason = "peak indices fit in f64 mantissa"
                    )]
                    let diff = w[1] as f64 - w[0] as f64;
                    diff / profile.ecg_fs * 1000.0
                })
                .collect();
            return BiosignalAssessment {
                heart_rate_bpm: fused.heart_rate_bpm,
                sdnn_ms: fused.hrv_sdnn_ms,
                rmssd_ms: fused.hrv_rmssd_ms,
                spo2_percent: fused.spo2_percent,
                stress_index: fused.stress_index,
                overall_score: fused.overall_score,
                rr_intervals_ms: rr,
            };
        }
    }

    BiosignalAssessment {
        heart_rate_bpm: cfg.fallback_hr_bpm,
        sdnn_ms: cfg.fallback_sdnn_ms,
        rmssd_ms: cfg.fallback_rmssd_ms,
        spo2_percent: profile.ppg_spo2.unwrap_or(cfg.fallback_spo2),
        stress_index: cfg.fallback_stress,
        overall_score: cfg.fallback_overall_score,
        rr_intervals_ms: Vec::new(),
    }
}

fn assess_endocrine(profile: &PatientProfile, cfg: &DiagnosticConfig) -> EndocrineAssessment {
    let baseline_t = match profile.sex {
        Sex::Male => cfg.baseline_t_male,
        Sex::Female => cfg.baseline_t_female,
    };

    let predicted_t = profile.testosterone_ng_dl.unwrap_or_else(|| {
        endocrine::testosterone_decline(
            baseline_t,
            cfg.t_decline_rate,
            profile.age_years,
            cfg.t_decline_onset_years,
        )
    });

    let sdnn_base = cfg.fallback_sdnn_ms;
    let hrv_sdnn = if profile.on_trt {
        endocrine::hrv_trt_response(
            sdnn_base,
            cfg.sdnn_delta_trt,
            cfg.sdnn_tau_months,
            profile.trt_months,
        )
    } else {
        sdnn_base
    };

    let cardiac =
        endocrine::cardiac_risk_composite(hrv_sdnn, predicted_t, cfg.cardiac_baseline_risk);

    let metabolic = if profile.on_trt {
        endocrine::weight_trajectory(profile.trt_months, -12.0, 12.0, 60.0)
    } else {
        0.0
    };

    EndocrineAssessment {
        predicted_testosterone: predicted_t,
        age_decline_rate: cfg.t_decline_rate,
        hrv_trt_sdnn: hrv_sdnn,
        cardiac_risk: cardiac,
        metabolic_response: metabolic,
    }
}

fn assess_cross_track(
    microbiome: &MicrobiomeAssessment,
    endocrine: &EndocrineAssessment,
    biosignal: &BiosignalAssessment,
    cfg: &DiagnosticConfig,
) -> CrossTrackAssessment {
    let disorder = endocrine::evenness_to_disorder(microbiome.pielou_evenness, cfg.gut_w_scale);
    #[expect(clippy::cast_precision_loss, reason = "lattice site count fits f64")]
    let lattice_f = cfg.anderson_lattice_sites as f64;
    let xi = endocrine::anderson_localization_length(disorder, lattice_f);
    let gut_response = endocrine::gut_metabolic_response(xi, cfg.gut_xi_max, cfg.gut_base_response);

    let hrv_cardiac = endocrine::cardiac_risk_composite(
        biosignal.sdnn_ms,
        endocrine.predicted_testosterone,
        cfg.cardiac_baseline_risk,
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
    cfg: &DiagnosticConfig,
) -> f64 {
    let micro_risk = 1.0 - microbiome.colonization_resistance;
    let bio_risk = biosignal.stress_index;
    let endo_risk = endocrine.cardiac_risk;
    let cross_risk = cross_track.hrv_cardiac_composite;

    (cfg.weight_microbiome * micro_risk
        + cfg.weight_biosignal * bio_risk
        + cfg.weight_endocrine * endo_risk
        + cfg.weight_cross_track * cross_risk)
        .clamp(0.0, 1.0)
}

/// Run the full diagnostic pipeline with published-literature defaults.
#[must_use]
pub fn assess_patient(profile: &PatientProfile) -> DiagnosticAssessment {
    assess_patient_with_config(profile, &DiagnosticConfig::default())
}

/// Run the full diagnostic pipeline with custom configuration.
#[must_use]
pub fn assess_patient_with_config(
    profile: &PatientProfile,
    cfg: &DiagnosticConfig,
) -> DiagnosticAssessment {
    let pk = assess_pk(profile, cfg);
    let microbiome = assess_microbiome(profile, cfg);
    let biosignal = assess_biosignal(profile, cfg);
    let endocrine = assess_endocrine(profile, cfg);
    let cross_track = assess_cross_track(&microbiome, &endocrine, &biosignal, cfg);
    let risk = composite_risk(&microbiome, &biosignal, &endocrine, &cross_track, cfg);

    DiagnosticAssessment {
        pk,
        microbiome,
        biosignal,
        endocrine,
        cross_track,
        composite_risk: risk,
    }
}

/// Run population Monte Carlo with published-literature defaults.
#[must_use]
pub fn population_montecarlo(
    base_profile: &PatientProfile,
    n_patients: usize,
    seed: u64,
) -> PopulationResult {
    population_montecarlo_with_config(base_profile, n_patients, seed, &DiagnosticConfig::default())
}

/// Run population Monte Carlo with custom configuration.
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "patient indices fit f64")]
pub fn population_montecarlo_with_config(
    base_profile: &PatientProfile,
    n_patients: usize,
    seed: u64,
    cfg: &DiagnosticConfig,
) -> PopulationResult {
    let base_assessment = assess_patient_with_config(base_profile, cfg);
    let base_risk = base_assessment.composite_risk;

    let mut rng = seed;
    let mut risks = Vec::with_capacity(n_patients);

    for _ in 0..n_patients {
        let (z, next_rng) = crate::rng::normal_sample(rng);
        rng = next_rng;

        let age_var = (profile_cv_lognormal(base_profile.age_years, cfg.mc_age_cv, z)).max(18.0);
        let weight_var =
            profile_cv_lognormal(base_profile.weight_kg, cfg.mc_weight_cv, z).max(30.0);

        let (z2, next_rng2) = crate::rng::normal_sample(rng);
        rng = next_rng2;

        let t_var = base_profile
            .testosterone_ng_dl
            .map(|t| profile_cv_lognormal(t, cfg.mc_testosterone_cv, z2).max(10.0));

        let virtual_profile = PatientProfile {
            age_years: age_var,
            weight_kg: weight_var,
            testosterone_ng_dl: t_var,
            ..base_profile.clone()
        };

        let assessment = assess_patient_with_config(&virtual_profile, cfg);
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
        let cfg = DiagnosticConfig::default();
        let light = PatientProfile::minimal(40.0, 50.0, Sex::Female);
        let heavy = PatientProfile::minimal(40.0, 100.0, Sex::Male);

        let pk_light = assess_pk(&light, &cfg);
        let pk_heavy = assess_pk(&heavy, &cfg);

        assert!(pk_heavy.allometric_cl > pk_light.allometric_cl);
        assert!(pk_heavy.allometric_vd > pk_light.allometric_vd);
    }

    #[test]
    fn microbiome_risk_levels() {
        let cfg = DiagnosticConfig::default();
        let healthy = vec![0.20, 0.18, 0.15, 0.12, 0.10, 0.08, 0.07, 0.05, 0.03, 0.02];
        let dysbiotic = vec![0.90, 0.05, 0.03, 0.02];

        let mut p = PatientProfile::minimal(40.0, 70.0, Sex::Male);
        p.gut_abundances = Some(healthy);
        let a1 = assess_microbiome(&p, &cfg);

        p.gut_abundances = Some(dysbiotic);
        let a2 = assess_microbiome(&p, &cfg);

        assert!(a1.shannon > a2.shannon);
        assert!(a1.pielou_evenness > a2.pielou_evenness);
    }

    #[test]
    fn trt_improves_endocrine_metrics() {
        let cfg = DiagnosticConfig::default();
        let mut off_trt = PatientProfile::minimal(55.0, 85.0, Sex::Male);
        off_trt.testosterone_ng_dl = Some(300.0);

        let mut on_trt = off_trt.clone();
        on_trt.on_trt = true;
        on_trt.trt_months = 24.0;

        let e_off = assess_endocrine(&off_trt, &cfg);
        let e_on = assess_endocrine(&on_trt, &cfg);

        assert!(e_on.hrv_trt_sdnn > e_off.hrv_trt_sdnn);
        assert!(e_on.metabolic_response < 0.0);
    }

    #[test]
    fn custom_config_changes_output() {
        let p = test_profile();
        let default_result = assess_patient(&p);

        let custom_cfg = DiagnosticConfig {
            dose_mg: 8.0,
            weight_microbiome: 0.5,
            weight_biosignal: 0.2,
            weight_endocrine: 0.2,
            weight_cross_track: 0.1,
            ..DiagnosticConfig::default()
        };

        let custom_result = assess_patient_with_config(&p, &custom_cfg);

        assert!(
            custom_result.pk.oral_cmax > default_result.pk.oral_cmax,
            "doubled dose should increase Cmax"
        );
        assert!(
            (custom_result.composite_risk - default_result.composite_risk).abs() > 1e-6,
            "different weights should change composite risk"
        );
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
        assert_eq!(r1.mean_risk.to_bits(), r2.mean_risk.to_bits());
        assert_eq!(r1.patient_percentile.to_bits(), r2.patient_percentile.to_bits());
    }

    #[test]
    fn minimal_profile_no_panic() {
        let p = PatientProfile::minimal(30.0, 70.0, Sex::Female);
        let result = assess_patient(&p);
        assert!((0.0..=1.0).contains(&result.composite_risk));
    }
}
