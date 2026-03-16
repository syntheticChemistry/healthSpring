// SPDX-License-Identifier: AGPL-3.0-or-later
//! Assessment pipeline: PK, microbiome, biosignal, endocrine, cross-track.

use super::{
    BiosignalAssessment, CrossTrackAssessment, DiagnosticAssessment, DiagnosticConfig,
    EndocrineAssessment, MicrobiomeAssessment, PatientProfile, PkAssessment, RiskLevel, Sex,
};
use crate::{biosignal, endocrine, microbiome, pkpd};

pub(super) fn assess_pk(profile: &PatientProfile, cfg: &DiagnosticConfig) -> PkAssessment {
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

pub(super) fn assess_microbiome(
    profile: &PatientProfile,
    cfg: &DiagnosticConfig,
) -> MicrobiomeAssessment {
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
            disorder * 0.1f64.mul_add((fi * 0.7).sin(), 1.0)
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

pub(super) fn assess_biosignal(
    profile: &PatientProfile,
    cfg: &DiagnosticConfig,
) -> BiosignalAssessment {
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

pub(super) fn assess_endocrine(
    profile: &PatientProfile,
    cfg: &DiagnosticConfig,
) -> EndocrineAssessment {
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

pub(super) fn assess_cross_track(
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

pub(super) fn composite_risk(
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

    cfg.weight_cross_track
        .mul_add(
            cross_risk,
            cfg.weight_endocrine.mul_add(
                endo_risk,
                cfg.weight_microbiome
                    .mul_add(micro_risk, cfg.weight_biosignal * bio_risk),
            ),
        )
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
