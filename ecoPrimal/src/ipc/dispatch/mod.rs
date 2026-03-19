// SPDX-License-Identifier: AGPL-3.0-or-later
//! Capability dispatch — maps JSON-RPC method names to science modules.
//!
//! Each handler extracts parameters from JSON, calls the domain function,
//! and returns the result as JSON. Missing or invalid params yield an
//! `"error"` key in the result (not a JSON-RPC error envelope — the
//! binary layer handles that).

mod handlers;

use serde_json::Value;

struct CapabilityEntry {
    method: &'static str,
    handler: fn(&Value) -> Value,
    domain: &'static str,
}

static REGISTRY: &[CapabilityEntry] = &[
    CapabilityEntry {
        method: "science.pkpd.hill_dose_response",
        handler: handlers::pkpd::dispatch_hill,
        domain: "pkpd",
    },
    CapabilityEntry {
        method: "science.pkpd.one_compartment_pk",
        handler: handlers::pkpd::dispatch_one_compartment,
        domain: "pkpd",
    },
    CapabilityEntry {
        method: "science.pkpd.two_compartment_pk",
        handler: handlers::pkpd::dispatch_two_compartment,
        domain: "pkpd",
    },
    CapabilityEntry {
        method: "science.pkpd.pbpk_simulate",
        handler: handlers::pkpd::dispatch_pbpk,
        domain: "pkpd",
    },
    CapabilityEntry {
        method: "science.pkpd.population_pk",
        handler: handlers::pkpd::dispatch_population_pk,
        domain: "pkpd",
    },
    CapabilityEntry {
        method: "science.pkpd.allometric_scale",
        handler: handlers::pkpd::dispatch_allometric,
        domain: "pkpd",
    },
    CapabilityEntry {
        method: "science.pkpd.auc_trapezoidal",
        handler: handlers::pkpd::dispatch_auc,
        domain: "pkpd",
    },
    CapabilityEntry {
        method: "science.pkpd.nca_analysis",
        handler: handlers::pkpd::dispatch_nca,
        domain: "pkpd",
    },
    CapabilityEntry {
        method: "science.pkpd.michaelis_menten_nonlinear",
        handler: handlers::pkpd::dispatch_mm,
        domain: "pkpd",
    },
    CapabilityEntry {
        method: "science.pkpd.nlme_foce",
        handler: handlers::pkpd::dispatch_nlme_foce,
        domain: "pkpd",
    },
    CapabilityEntry {
        method: "science.pkpd.nlme_saem",
        handler: handlers::pkpd::dispatch_nlme_saem,
        domain: "pkpd",
    },
    CapabilityEntry {
        method: "science.pkpd.cwres_diagnostics",
        handler: handlers::pkpd::dispatch_cwres,
        domain: "pkpd",
    },
    CapabilityEntry {
        method: "science.pkpd.vpc_simulate",
        handler: handlers::pkpd::dispatch_vpc,
        domain: "pkpd",
    },
    CapabilityEntry {
        method: "science.pkpd.gof_compute",
        handler: handlers::pkpd::dispatch_gof,
        domain: "pkpd",
    },
    CapabilityEntry {
        method: "science.microbiome.shannon_index",
        handler: handlers::microbiome::dispatch_shannon,
        domain: "microbiome",
    },
    CapabilityEntry {
        method: "science.microbiome.simpson_index",
        handler: handlers::microbiome::dispatch_simpson,
        domain: "microbiome",
    },
    CapabilityEntry {
        method: "science.microbiome.pielou_evenness",
        handler: handlers::microbiome::dispatch_pielou,
        domain: "microbiome",
    },
    CapabilityEntry {
        method: "science.microbiome.chao1",
        handler: handlers::microbiome::dispatch_chao1,
        domain: "microbiome",
    },
    CapabilityEntry {
        method: "science.microbiome.anderson_gut",
        handler: handlers::microbiome::dispatch_anderson_gut,
        domain: "microbiome",
    },
    CapabilityEntry {
        method: "science.microbiome.colonization_resistance",
        handler: handlers::microbiome::dispatch_colonization,
        domain: "microbiome",
    },
    CapabilityEntry {
        method: "science.microbiome.fmt_blend",
        handler: handlers::microbiome::dispatch_fmt_blend,
        domain: "microbiome",
    },
    CapabilityEntry {
        method: "science.microbiome.bray_curtis",
        handler: handlers::microbiome::dispatch_bray_curtis,
        domain: "microbiome",
    },
    CapabilityEntry {
        method: "science.microbiome.antibiotic_perturbation",
        handler: handlers::microbiome::dispatch_antibiotic,
        domain: "microbiome",
    },
    CapabilityEntry {
        method: "science.microbiome.scfa_production",
        handler: handlers::microbiome::dispatch_scfa,
        domain: "microbiome",
    },
    CapabilityEntry {
        method: "science.microbiome.gut_brain_serotonin",
        handler: handlers::microbiome::dispatch_gut_brain,
        domain: "microbiome",
    },
    CapabilityEntry {
        method: "science.microbiome.qs_gene_profile",
        handler: handlers::microbiome::dispatch_qs_profile,
        domain: "microbiome",
    },
    CapabilityEntry {
        method: "science.microbiome.qs_effective_disorder",
        handler: handlers::microbiome::dispatch_qs_effective_disorder,
        domain: "microbiome",
    },
    CapabilityEntry {
        method: "science.biosignal.pan_tompkins",
        handler: handlers::biosignal::dispatch_pan_tompkins,
        domain: "biosignal",
    },
    CapabilityEntry {
        method: "science.biosignal.hrv_metrics",
        handler: handlers::biosignal::dispatch_hrv,
        domain: "biosignal",
    },
    CapabilityEntry {
        method: "science.biosignal.ppg_spo2",
        handler: handlers::biosignal::dispatch_ppg_spo2,
        domain: "biosignal",
    },
    CapabilityEntry {
        method: "science.biosignal.eda_analysis",
        handler: handlers::biosignal::dispatch_eda,
        domain: "biosignal",
    },
    CapabilityEntry {
        method: "science.biosignal.eda_stress_detection",
        handler: handlers::biosignal::dispatch_eda_stress,
        domain: "biosignal",
    },
    CapabilityEntry {
        method: "science.biosignal.arrhythmia_classification",
        handler: handlers::biosignal::dispatch_arrhythmia,
        domain: "biosignal",
    },
    CapabilityEntry {
        method: "science.biosignal.fuse_channels",
        handler: handlers::biosignal::dispatch_fuse,
        domain: "biosignal",
    },
    CapabilityEntry {
        method: "science.biosignal.wfdb_decode",
        handler: handlers::biosignal::dispatch_wfdb_decode,
        domain: "biosignal",
    },
    CapabilityEntry {
        method: "science.endocrine.testosterone_pk",
        handler: handlers::clinical::dispatch_testosterone_pk,
        domain: "endocrine",
    },
    CapabilityEntry {
        method: "science.endocrine.trt_outcomes",
        handler: handlers::clinical::dispatch_trt_outcomes,
        domain: "endocrine",
    },
    CapabilityEntry {
        method: "science.endocrine.hrv_trt_response",
        handler: handlers::clinical::dispatch_hrv_trt,
        domain: "endocrine",
    },
    CapabilityEntry {
        method: "science.endocrine.cardiac_risk",
        handler: handlers::clinical::dispatch_cardiac_risk,
        domain: "endocrine",
    },
    CapabilityEntry {
        method: "science.endocrine.population_trt",
        handler: handlers::clinical::dispatch_population_trt,
        domain: "endocrine",
    },
    CapabilityEntry {
        method: "science.diagnostic.assess_patient",
        handler: handlers::clinical::dispatch_assess_patient,
        domain: "diagnostic",
    },
    CapabilityEntry {
        method: "science.diagnostic.composite_risk",
        handler: handlers::clinical::dispatch_composite_risk,
        domain: "diagnostic",
    },
    CapabilityEntry {
        method: "science.diagnostic.population_montecarlo",
        handler: handlers::clinical::dispatch_population_montecarlo,
        domain: "diagnostic",
    },
    CapabilityEntry {
        method: "science.clinical.trt_scenario",
        handler: handlers::clinical::dispatch_trt_scenario,
        domain: "clinical",
    },
    CapabilityEntry {
        method: "science.clinical.patient_parameterize",
        handler: handlers::clinical::dispatch_patient_parameterize,
        domain: "clinical",
    },
    CapabilityEntry {
        method: "science.clinical.risk_annotate",
        handler: handlers::clinical::dispatch_risk_annotate,
        domain: "clinical",
    },
    CapabilityEntry {
        method: "science.toxicology.biphasic_dose_response",
        handler: handlers::toxicology::dispatch_biphasic_dose_response,
        domain: "toxicology",
    },
    CapabilityEntry {
        method: "science.toxicology.toxicity_landscape",
        handler: handlers::toxicology::dispatch_toxicity_landscape,
        domain: "toxicology",
    },
    CapabilityEntry {
        method: "science.toxicology.hormetic_optimum",
        handler: handlers::toxicology::dispatch_hormetic_optimum,
        domain: "toxicology",
    },
    CapabilityEntry {
        method: "science.simulation.mechanistic_fitness",
        handler: handlers::simulation::dispatch_mechanistic_fitness,
        domain: "simulation",
    },
    CapabilityEntry {
        method: "science.simulation.ecosystem_simulate",
        handler: handlers::simulation::dispatch_ecosystem_simulate,
        domain: "simulation",
    },
];

/// Dispatch a science method. Returns `None` if the method is unknown.
#[must_use]
pub fn dispatch_science(method: &str, params: &Value) -> Option<Value> {
    REGISTRY
        .iter()
        .find(|e| e.method == method)
        .map(|e| (e.handler)(params))
}

/// List all registered science capabilities and their domains.
#[must_use]
pub fn registered_capabilities() -> Vec<(&'static str, &'static str)> {
    REGISTRY.iter().map(|e| (e.method, e.domain)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tolerances;

    #[test]
    fn hill_dispatch_works() {
        let params = serde_json::json!({
            "concentration": 10.0, "ic50": 10.0, "hill_n": 1.0, "e_max": 1.0,
        });
        let result = dispatch_science("science.pkpd.hill_dose_response", &params);
        let Some(v) = result else {
            panic!("dispatch returned None for hill_dose_response");
        };
        let Some(response) = v.get("response").and_then(Value::as_f64) else {
            panic!("response field missing");
        };
        assert!((response - 0.5).abs() < tolerances::TEST_ASSERTION_TIGHT);
    }

    #[test]
    fn shannon_dispatch_works() {
        let params = serde_json::json!({"abundances": [0.25, 0.25, 0.25, 0.25]});
        let result = dispatch_science("science.microbiome.shannon_index", &params);
        let Some(v) = result else {
            panic!("dispatch returned None for shannon_index");
        };
        let Some(h) = v.get("shannon").and_then(Value::as_f64) else {
            panic!("shannon field missing");
        };
        assert!((h - 4.0_f64.ln()).abs() < tolerances::TEST_ASSERTION_TIGHT);
    }

    #[test]
    fn unknown_method_returns_none() {
        let result = dispatch_science("science.unknown.method", &serde_json::json!({}));
        assert!(result.is_none());
    }

    #[test]
    fn missing_params_returns_error() {
        let result = dispatch_science("science.pkpd.hill_dose_response", &serde_json::json!({}));
        let Some(v) = result else {
            panic!("dispatch returned None for missing params");
        };
        assert!(v.get("error").is_some());
    }

    #[test]
    fn registry_lists_all_capabilities() {
        let caps = registered_capabilities();
        assert!(caps.len() >= 51, "registry should have 51+ capabilities");
        assert!(
            caps.iter()
                .any(|(m, _)| *m == "science.pkpd.hill_dose_response")
        );
        assert!(
            caps.iter()
                .any(|(m, _)| *m == "science.microbiome.shannon_index")
        );
    }
}
