// SPDX-License-Identifier: AGPL-3.0-or-later
//! Capability dispatch — maps JSON-RPC method names to science modules.
//!
//! Each handler extracts parameters from JSON, calls the domain function,
//! and returns the result as JSON. Missing or invalid params yield an
//! `"error"` key in the result (not a JSON-RPC error envelope — the
//! binary layer handles that).

mod handlers;

use serde_json::Value;

use handlers::{
    biosignal::{
        dispatch_arrhythmia, dispatch_eda, dispatch_eda_stress, dispatch_fuse, dispatch_hrv,
        dispatch_pan_tompkins, dispatch_ppg_spo2, dispatch_wfdb_decode,
    },
    clinical::{
        dispatch_assess_patient, dispatch_cardiac_risk, dispatch_composite_risk, dispatch_hrv_trt,
        dispatch_patient_parameterize, dispatch_population_montecarlo, dispatch_population_trt,
        dispatch_risk_annotate, dispatch_testosterone_pk, dispatch_trt_outcomes,
        dispatch_trt_scenario,
    },
    microbiome::{
        dispatch_anderson_gut, dispatch_antibiotic, dispatch_bray_curtis, dispatch_chao1,
        dispatch_colonization, dispatch_fmt_blend, dispatch_gut_brain, dispatch_pielou,
        dispatch_qs_effective_disorder, dispatch_qs_profile, dispatch_scfa, dispatch_shannon,
        dispatch_simpson,
    },
    pkpd::{
        dispatch_allometric, dispatch_auc, dispatch_cwres, dispatch_gof, dispatch_hill,
        dispatch_mm, dispatch_nca, dispatch_nlme_foce, dispatch_nlme_saem,
        dispatch_one_compartment, dispatch_pbpk, dispatch_population_pk, dispatch_two_compartment,
        dispatch_vpc,
    },
};

/// Dispatch a science method. Returns `None` if the method is unknown.
#[must_use]
pub fn dispatch_science(method: &str, params: &Value) -> Option<Value> {
    let result = match method {
        // ── PK/PD ────────────────────────────────────────────────────
        "science.pkpd.hill_dose_response" => dispatch_hill(params),
        "science.pkpd.one_compartment_pk" => dispatch_one_compartment(params),
        "science.pkpd.two_compartment_pk" => dispatch_two_compartment(params),
        "science.pkpd.pbpk_simulate" => dispatch_pbpk(params),
        "science.pkpd.population_pk" => dispatch_population_pk(params),
        "science.pkpd.allometric_scale" => dispatch_allometric(params),
        "science.pkpd.auc_trapezoidal" => dispatch_auc(params),
        "science.pkpd.nca_analysis" => dispatch_nca(params),
        "science.pkpd.michaelis_menten_nonlinear" => dispatch_mm(params),
        "science.pkpd.nlme_foce" => dispatch_nlme_foce(params),
        "science.pkpd.nlme_saem" => dispatch_nlme_saem(params),
        "science.pkpd.cwres_diagnostics" => dispatch_cwres(params),
        "science.pkpd.vpc_simulate" => dispatch_vpc(params),
        "science.pkpd.gof_compute" => dispatch_gof(params),

        // ── Microbiome ───────────────────────────────────────────────
        "science.microbiome.shannon_index" => dispatch_shannon(params),
        "science.microbiome.simpson_index" => dispatch_simpson(params),
        "science.microbiome.pielou_evenness" => dispatch_pielou(params),
        "science.microbiome.chao1" => dispatch_chao1(params),
        "science.microbiome.anderson_gut" => dispatch_anderson_gut(params),
        "science.microbiome.colonization_resistance" => dispatch_colonization(params),
        "science.microbiome.fmt_blend" => dispatch_fmt_blend(params),
        "science.microbiome.bray_curtis" => dispatch_bray_curtis(params),
        "science.microbiome.antibiotic_perturbation" => dispatch_antibiotic(params),
        "science.microbiome.scfa_production" => dispatch_scfa(params),
        "science.microbiome.gut_brain_serotonin" => dispatch_gut_brain(params),
        "science.microbiome.qs_gene_profile" => dispatch_qs_profile(params),
        "science.microbiome.qs_effective_disorder" => dispatch_qs_effective_disorder(params),

        // ── Biosignal ────────────────────────────────────────────────
        "science.biosignal.pan_tompkins" => dispatch_pan_tompkins(params),
        "science.biosignal.hrv_metrics" => dispatch_hrv(params),
        "science.biosignal.ppg_spo2" => dispatch_ppg_spo2(params),
        "science.biosignal.eda_analysis" => dispatch_eda(params),
        "science.biosignal.eda_stress_detection" => dispatch_eda_stress(params),
        "science.biosignal.arrhythmia_classification" => dispatch_arrhythmia(params),
        "science.biosignal.fuse_channels" => dispatch_fuse(params),
        "science.biosignal.wfdb_decode" => dispatch_wfdb_decode(params),

        // ── Endocrine ────────────────────────────────────────────────
        "science.endocrine.testosterone_pk" => dispatch_testosterone_pk(params),
        "science.endocrine.trt_outcomes" => dispatch_trt_outcomes(params),
        "science.endocrine.hrv_trt_response" => dispatch_hrv_trt(params),
        "science.endocrine.cardiac_risk" => dispatch_cardiac_risk(params),
        "science.endocrine.population_trt" => dispatch_population_trt(params),

        // ── Diagnostic ───────────────────────────────────────────────
        "science.diagnostic.assess_patient" => dispatch_assess_patient(params),
        "science.diagnostic.composite_risk" => dispatch_composite_risk(params),
        "science.diagnostic.population_montecarlo" => dispatch_population_montecarlo(params),

        // ── Clinical ────────────────────────────────────────────────
        "science.clinical.trt_scenario" => dispatch_trt_scenario(params),
        "science.clinical.patient_parameterize" => dispatch_patient_parameterize(params),
        "science.clinical.risk_annotate" => dispatch_risk_annotate(params),

        _ => return None,
    };
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hill_dispatch_works() {
        let params = serde_json::json!({
            "concentration": 10.0, "ic50": 10.0, "hill_n": 1.0, "e_max": 1.0,
        });
        let result = dispatch_science("science.pkpd.hill_dose_response", &params);
        assert!(result.is_some());
        let v = result.expect("dispatch returned Some");
        let response = v
            .get("response")
            .and_then(Value::as_f64)
            .expect("has response");
        assert!((response - 0.5).abs() < 1e-10);
    }

    #[test]
    fn shannon_dispatch_works() {
        let params = serde_json::json!({"abundances": [0.25, 0.25, 0.25, 0.25]});
        let result = dispatch_science("science.microbiome.shannon_index", &params);
        assert!(result.is_some());
        let v = result.expect("dispatch returned Some");
        let h = v
            .get("shannon")
            .and_then(Value::as_f64)
            .expect("has shannon");
        assert!((h - 4.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn unknown_method_returns_none() {
        let result = dispatch_science("science.unknown.method", &serde_json::json!({}));
        assert!(result.is_none());
    }

    #[test]
    fn missing_params_returns_error() {
        let result = dispatch_science("science.pkpd.hill_dose_response", &serde_json::json!({}));
        assert!(result.is_some());
        let v = result.expect("dispatch returned Some");
        assert!(v.get("error").is_some());
    }
}
