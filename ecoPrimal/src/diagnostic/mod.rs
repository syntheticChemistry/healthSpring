// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integrated diagnostic pipeline composing all four healthSpring tracks.
//!
//! Takes a patient profile and runs PK/PD modeling, microbiome risk assessment,
//! biosignal analysis, and endocrine outcome prediction. Cross-track models
//! (gut-TRT axis, HRV-TRT cardiac) integrate the results. Population Monte Carlo
//! places the patient within a virtual cohort for percentile context.

pub mod assessment;
pub mod population;

/// Biological sex for allometric scaling and reference ranges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sex {
    /// Male sex for clearance and male reference nomograms.
    Male,
    /// Female sex for clearance and female reference nomograms.
    Female,
}

/// Input patient profile — all fields optional except demographics.
#[derive(Debug, Clone)]
pub struct PatientProfile {
    /// Chronological age in years for scaling and decline models.
    pub age_years: f64,
    /// Body mass in kilograms for allometric PK scaling.
    pub weight_kg: f64,
    /// Biological sex for clearance volumes and reference ranges.
    pub sex: Sex,
    /// Measured or assumed serum testosterone (ng/dL); omit if unknown.
    pub testosterone_ng_dl: Option<f64>,
    /// Whether the patient receives testosterone replacement therapy.
    pub on_trt: bool,
    /// Months on TRT for adaptation and HRV–TRT coupling models.
    pub trt_months: f64,
    /// Optional genus-level relative abundances for microbiome metrics.
    pub gut_abundances: Option<Vec<f64>>,
    /// Optional R-peak sample indices for RR-based HRV from ECG.
    pub ecg_peaks: Option<Vec<usize>>,
    /// ECG sampling rate (Hz) used to convert peaks to RR intervals.
    pub ecg_fs: f64,
    /// Optional `SpO₂` from pulse oximetry when available (% or fraction per ingest).
    pub ppg_spo2: Option<f64>,
    /// Optional skin conductance response count over the EDA window.
    pub scr_count: Option<usize>,
    /// EDA recording length in seconds (normalizes SCR density).
    pub eda_duration_s: f64,
}

impl PatientProfile {
    /// Minimal profile: demographics only; other inputs default to empty or clinical fallbacks.
    #[must_use]
    pub const fn minimal(age_years: f64, weight_kg: f64, sex: Sex) -> Self {
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

/// PK/PD outputs: oral PK, allometric scaling, and Hill dose–response.
#[derive(Debug, Clone)]
pub struct PkAssessment {
    /// Hill model response at EC50 (fraction of Emax).
    pub hill_response_at_ec50: f64,
    /// Peak plasma concentration after the modeled oral dose (mg/L).
    pub oral_cmax: f64,
    /// Time to Cmax after oral dosing (h).
    pub oral_tmax_hr: f64,
    /// Area under the plasma concentration–time curve (mg·h/L).
    pub oral_auc: f64,
    /// Allometrically scaled clearance (L/h) for the patient’s weight.
    pub allometric_cl: f64,
    /// Allometrically scaled volume of distribution (L).
    pub allometric_vd: f64,
    /// Full 0..24h concentration-time curve (101 points).
    pub curve_times_hr: Vec<f64>,
    /// Plasma concentrations (mg/L) aligned with `curve_times_hr`.
    pub curve_concs_mg_l: Vec<f64>,
    /// Hill dose-response sweep (50 points, log-spaced).
    pub hill_concs: Vec<f64>,
    /// Hill response values aligned with `hill_concs` (same units as Emax).
    pub hill_responses: Vec<f64>,
}

/// Gut microbiome diversity, colonization resistance, and discrete risk band.
#[derive(Debug, Clone)]
pub struct MicrobiomeAssessment {
    /// Shannon diversity of the abundance vector.
    pub shannon: f64,
    /// Simpson index (dominance-sensitive diversity).
    pub simpson: f64,
    /// Pielou evenness (J) given richness.
    pub pielou_evenness: f64,
    /// Surrogate for pathogen exclusion capacity (higher is more protective).
    pub colonization_resistance: f64,
    /// Discrete risk band from configured resistance thresholds.
    pub risk_level: RiskLevel,
    /// Raw genus-level relative abundances (passthrough from profile).
    pub abundances: Vec<f64>,
}

/// Cardiovascular and autonomic summary (HRV, `SpO₂`, stress) from biosignals or fallbacks.
#[derive(Debug, Clone)]
pub struct BiosignalAssessment {
    /// Mean heart rate (bpm) from RR intervals or fallback.
    pub heart_rate_bpm: f64,
    /// RR standard deviation—global HRV (ms).
    pub sdnn_ms: f64,
    /// Root mean square of successive RR differences—parasympathetic proxy (ms).
    pub rmssd_ms: f64,
    /// Peripheral oxygen saturation (%).
    pub spo2_percent: f64,
    /// Scalar stress load (0–1) from EDA/physiology when available.
    pub stress_index: f64,
    /// Aggregated wellness score on the pipeline’s 0–100 scale.
    pub overall_score: f64,
    /// RR intervals in ms (when ECG peaks available).
    pub rr_intervals_ms: Vec<f64>,
}

/// Endocrine track: testosterone prediction, TRT–HRV coupling, and related risks.
#[derive(Debug, Clone)]
pub struct EndocrineAssessment {
    /// Model-predicted serum testosterone (ng/dL).
    pub predicted_testosterone: f64,
    /// Annual fractional decline rate applied after onset age (1/y).
    pub age_decline_rate: f64,
    /// SDNN (ms) after modeled TRT effect on autonomic tone.
    pub hrv_trt_sdnn: f64,
    /// Composite cardiac risk scalar from pipeline cross-effects (0–1 scale).
    pub cardiac_risk: f64,
    /// Signed metabolic load or direction per model (e.g. fuel utilization shift).
    pub metabolic_response: f64,
}

/// Integrated gut–TRT and HRV–cardiac cross-track signals.
#[derive(Debug, Clone)]
pub struct CrossTrackAssessment {
    /// Modulated gut-mediated response linked to TRT context.
    pub gut_trt_response: f64,
    /// Combined HRV and cardiac risk signal for the cross-track axis.
    pub hrv_cardiac_composite: f64,
}

/// Discrete microbiome risk band from colonization resistance thresholds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    /// Resistance above the “low risk” cutoff.
    Low,
    /// Intermediate resistance band.
    Moderate,
    /// Elevated dysbiosis or low resistance band.
    High,
    /// Severe dysbiosis or critically low resistance.
    Critical,
}

/// Full diagnostic result composing all tracks.
#[derive(Debug, Clone)]
pub struct DiagnosticAssessment {
    /// PK/PD, Hill curve, and concentration–time outputs.
    pub pk: PkAssessment,
    /// Diversity, colonization resistance, and microbiome risk band.
    pub microbiome: MicrobiomeAssessment,
    /// HRV, `SpO₂`, stress, and composite biosignal score.
    pub biosignal: BiosignalAssessment,
    /// Testosterone trajectory, TRT–HRV, cardiac and metabolic terms.
    pub endocrine: EndocrineAssessment,
    /// Gut–TRT and HRV–cardiac integrated metrics.
    pub cross_track: CrossTrackAssessment,
    /// Weighted aggregate risk on [0, 1] across enabled tracks.
    pub composite_risk: f64,
}

/// Population Monte Carlo result.
#[derive(Debug, Clone)]
pub struct PopulationResult {
    /// Virtual cohort size (number of Monte Carlo draws).
    pub n_patients: usize,
    /// One composite risk sample per draw.
    pub composite_risks: Vec<f64>,
    /// Mean composite risk across the cohort.
    pub mean_risk: f64,
    /// Standard deviation of composite risk across draws.
    pub std_risk: f64,
    /// Subject’s percentile vs the simulated cohort (0–100).
    pub patient_percentile: f64,
}

/// Configurable pipeline parameters.
///
/// All values have documented defaults sourced from published literature.
/// Callers can override any parameter to adapt the pipeline to different
/// drugs, populations, or clinical protocols without modifying library code.
#[derive(Debug, Clone)]
pub struct DiagnosticConfig {
    // PK/PD
    /// Reference body weight for published CL and Vd (kg).
    pub ref_bw_kg: f64,
    /// Reference adult oral clearance (L/h).
    pub ref_cl_l_hr: f64,
    /// Reference volume of distribution (L).
    pub ref_vd_l: f64,
    /// Oral bioavailability F (fraction absorbed into systemic circulation).
    pub bioavailability: f64,
    /// First-order absorption rate constant ka (1/h).
    pub absorption_rate: f64,
    /// Oral dose simulated in the PK model (mg).
    pub dose_mg: f64,
    /// Hill Emax—maximum pharmacodynamic response (model units, often %).
    pub emax: f64,
    /// Half-maximal plasma concentration EC50 (mg/L).
    pub ec50: f64,
    /// Hill slope (sigmoidicity) for the PD link.
    pub hill_n: f64,
    /// Number of time samples for the oral concentration–time curve.
    pub pk_curve_points: u32,
    /// Duration of the simulated PK curve (h).
    pub pk_curve_hours: f64,
    /// Number of concentration points in the Hill dose–response sweep.
    pub hill_sweep_points: u32,

    // Endocrine
    /// Annual fractional testosterone decline after onset age (1/y).
    pub t_decline_rate: f64,
    /// Age (years) at which modeled age-related T decline begins.
    pub t_decline_onset_years: f64,
    /// Reference serum testosterone for males (ng/dL).
    pub baseline_t_male: f64,
    /// Reference serum testosterone for females (ng/dL).
    pub baseline_t_female: f64,
    /// Expected SDNN increase at TRT steady state vs baseline (ms).
    pub sdnn_delta_trt: f64,
    /// Time constant for TRT effect on SDNN to approach steady state (months).
    pub sdnn_tau_months: f64,
    /// Baseline cardiac risk scalar before cross-track modifiers (0–1).
    pub cardiac_baseline_risk: f64,

    // Cross-track
    /// Saturation scale for gut–TRT coupling (limits response magnitude).
    pub gut_xi_max: f64,
    /// Baseline gut-mediated response before abundance weighting.
    pub gut_base_response: f64,
    /// Scales Pielou evenness to Anderson disorder W (`W = J × scale`) for gut coupling.
    pub gut_w_scale: f64,
    /// Lattice size for the Anderson-style gut interaction grid.
    pub anderson_lattice_sites: usize,

    // Biosignal fallback (used when ECG data insufficient)
    /// Default heart rate when RR intervals cannot be derived (bpm).
    pub fallback_hr_bpm: f64,
    /// Default SDNN when HRV is unavailable (ms).
    pub fallback_sdnn_ms: f64,
    /// Default RMSSD when HRV is unavailable (ms).
    pub fallback_rmssd_ms: f64,
    /// Default `SpO₂` when oximetry is missing (%).
    pub fallback_spo2: f64,
    /// Default stress index when EDA is missing (0–1).
    pub fallback_stress: f64,
    /// Default biosignal composite score when signals are insufficient (0–100).
    pub fallback_overall_score: f64,

    // Microbiome risk thresholds
    /// Minimum colonization resistance scored as low microbiome risk.
    pub resistance_low_threshold: f64,
    /// Upper bound of the moderate risk band (exclusive of “low”).
    pub resistance_moderate_threshold: f64,
    /// Upper bound of the high risk band (below this is not “critical”).
    pub resistance_high_threshold: f64,

    // Composite risk weights
    /// Relative weight of the microbiome track in `composite_risk`.
    pub weight_microbiome: f64,
    /// Relative weight of the biosignal track in `composite_risk`.
    pub weight_biosignal: f64,
    /// Relative weight of the endocrine track in `composite_risk`.
    pub weight_endocrine: f64,
    /// Relative weight of cross-track terms in `composite_risk`.
    pub weight_cross_track: f64,

    // Population Monte Carlo IIV
    /// Coefficient of variation for age in virtual cohort sampling.
    pub mc_age_cv: f64,
    /// Coefficient of variation for weight in virtual cohort sampling.
    pub mc_weight_cv: f64,
    /// Coefficient of variation for testosterone in virtual cohort sampling.
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

/// Run the full diagnostic pipeline using [`DiagnosticConfig::default`].
pub use assessment::assess_patient;
/// Run the full diagnostic pipeline with an explicit [`DiagnosticConfig`].
pub use assessment::assess_patient_with_config;
/// Monte Carlo virtual cohort: composite risk distribution and subject percentile (default config).
pub use population::population_montecarlo;
/// Same as [`population_montecarlo`] with explicit [`DiagnosticConfig`] and inter-individual variability.
pub use population::population_montecarlo_with_config;

#[cfg(test)]
mod tests {
    use super::assessment::{assess_endocrine, assess_microbiome, assess_pk};
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
        assert_eq!(
            r1.patient_percentile.to_bits(),
            r2.patient_percentile.to_bits()
        );
    }

    #[test]
    fn minimal_profile_no_panic() {
        let p = PatientProfile::minimal(30.0, 70.0, Sex::Female);
        let result = assess_patient(&p);
        assert!((0.0..=1.0).contains(&result.composite_risk));
    }
}
