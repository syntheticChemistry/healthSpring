// SPDX-License-Identifier: AGPL-3.0-or-later
//! IPC capability handlers — domain-specific JSON-RPC param extraction and dispatch.
//!
//! Param helpers (f, fa, ua, missing) are shared across all handler modules.

use serde_json::Value;

pub(super) mod biosignal;
pub(super) mod clinical;
pub(super) mod microbiome;
pub(super) mod pkpd;
pub(super) mod simulation;
pub(super) mod toxicology;

// ═══════════════════════════════════════════════════════════════════════════
// Param extraction helpers — used by all domain modules
// ═══════════════════════════════════════════════════════════════════════════

pub(super) fn f(params: &Value, key: &str) -> Option<f64> {
    params.get(key).and_then(Value::as_f64)
}

pub(super) fn fa(params: &Value, key: &str) -> Option<Vec<f64>> {
    params.get(key).and_then(|v| {
        v.as_array()
            .map(|arr| arr.iter().filter_map(Value::as_f64).collect())
    })
}

pub(super) fn ua(params: &Value, key: &str) -> Option<Vec<u64>> {
    params.get(key).and_then(|v| {
        v.as_array()
            .map(|arr| arr.iter().filter_map(Value::as_u64).collect())
    })
}

/// Extract a JSON integer as `usize`, saturating on overflow.
pub(super) fn sz(params: &Value, key: &str) -> Option<usize> {
    params
        .get(key)
        .and_then(Value::as_u64)
        .map(|v| usize::try_from(v).unwrap_or(usize::MAX))
}

/// Extract a JSON integer as `usize` with a default value.
pub(super) fn sz_or(params: &Value, key: &str, default: usize) -> usize {
    sz(params, key).unwrap_or(default)
}

/// Convert a JSON `u64` array into `Vec<usize>`, saturating on overflow.
pub(super) fn sza(params: &Value, key: &str) -> Option<Vec<usize>> {
    params.get(key).and_then(|v| {
        v.as_array().map(|arr| {
            arr.iter()
                .filter_map(Value::as_u64)
                .map(|v| usize::try_from(v).unwrap_or(usize::MAX))
                .collect()
        })
    })
}

pub(super) fn missing(name: &str) -> Value {
    serde_json::json!({"error": "missing_params", "param": name})
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn param_f_extracts_float() {
        let p = json!({"dose": 10.5});
        assert!((f(&p, "dose").unwrap() - 10.5).abs() < 1e-15);
        assert!(f(&p, "missing").is_none());
    }

    #[test]
    fn param_fa_extracts_float_array() {
        let p = json!({"times": [0.0, 1.0, 2.0]});
        let v = fa(&p, "times").unwrap();
        assert_eq!(v.len(), 3);
        assert!((v[1] - 1.0).abs() < 1e-15);
    }

    #[test]
    fn param_ua_extracts_u64_array() {
        let p = json!({"ids": [1, 2, 3]});
        let v = ua(&p, "ids").unwrap();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn param_sz_extracts_usize() {
        let p = json!({"n": 100});
        assert_eq!(sz(&p, "n"), Some(100));
        assert_eq!(sz_or(&p, "n", 50), 100);
        assert_eq!(sz_or(&p, "missing", 50), 50);
    }

    #[test]
    fn param_sza_extracts_usize_array() {
        let p = json!({"counts": [10, 20, 30]});
        let v = sza(&p, "counts").unwrap();
        assert_eq!(v, vec![10, 20, 30]);
    }

    #[test]
    fn missing_returns_error_json() {
        let v = missing("dose, ic50");
        assert_eq!(v["error"], "missing_params");
        assert_eq!(v["param"], "dose, ic50");
    }

    // ── PK/PD handler tests ──────────────────────────────────────────

    #[test]
    fn dispatch_hill_returns_response() {
        let p = json!({"concentration": 10.0, "ic50": 10.0, "hill_n": 2.0, "e_max": 1.0});
        let r = pkpd::dispatch_hill(&p);
        let response = r["response"].as_f64().unwrap();
        assert!((response - 0.5).abs() < 0.01, "at IC50, response ≈ 0.5: {response}");
    }

    #[test]
    fn dispatch_hill_missing_params() {
        let p = json!({"concentration": 10.0});
        let r = pkpd::dispatch_hill(&p);
        assert_eq!(r["error"], "missing_params");
    }

    #[test]
    fn dispatch_one_compartment_iv() {
        let p = json!({"dose_mg": 100.0, "vd": 10.0, "half_life_hr": 6.93, "t": 0.0});
        let r = pkpd::dispatch_one_compartment(&p);
        let c = r["concentration"].as_f64().unwrap();
        assert!((c - 10.0).abs() < 0.01, "C(0) = dose/vd = 10: {c}");
    }

    #[test]
    fn dispatch_one_compartment_oral() {
        let p = json!({"route": "oral", "dose": 100.0, "f": 0.8, "vd": 10.0, "ka": 1.0, "ke": 0.1, "t": 0.0});
        let r = pkpd::dispatch_one_compartment(&p);
        let c = r["concentration"].as_f64().unwrap();
        assert!((c - 0.0).abs() < 0.01, "oral C(0) = 0: {c}");
    }

    #[test]
    fn dispatch_auc_basic() {
        let p = json!({"times": [0.0, 1.0, 2.0], "concentrations": [10.0, 5.0, 0.0]});
        let r = pkpd::dispatch_auc(&p);
        let auc = r["auc"].as_f64().unwrap();
        assert!(auc > 0.0, "AUC should be positive: {auc}");
    }

    #[test]
    fn dispatch_allometric_roundtrip() {
        let p = json!({"param_animal": 10.0, "bw_animal": 70.0, "bw_human": 70.0});
        let r = pkpd::dispatch_allometric(&p);
        let scaled = r["scaled_param"].as_f64().unwrap();
        assert!((scaled - 10.0).abs() < 0.01, "same weight → identity: {scaled}");
    }

    #[test]
    fn dispatch_population_pk_returns_means() {
        let p = json!({"n": 10, "seed": 42});
        let r = pkpd::dispatch_population_pk(&p);
        assert!(r["n"].as_u64().unwrap() == 10);
        assert!(r["auc_mean"].as_f64().unwrap() > 0.0);
    }

    #[test]
    fn dispatch_pbpk_returns_auc() {
        let p = json!({"dose_mg": 100.0, "duration_hr": 1.0});
        let r = pkpd::dispatch_pbpk(&p);
        assert!(r["auc"].as_f64().unwrap() > 0.0);
        assert!(r["n_steps"].as_u64().unwrap() > 0);
    }

    // ── Toxicology handler tests ─────────────────────────────────────

    #[test]
    fn dispatch_biphasic_returns_fitness() {
        let p = json!({
            "dose": 5.0, "baseline": 100.0, "s_max": 0.3,
            "k_stim": 2.0, "ic50": 50.0, "hill_n": 2.0
        });
        let r = toxicology::dispatch_biphasic_dose_response(&p);
        let fitness = r["fitness"].as_f64().unwrap();
        assert!(fitness > 0.0, "fitness should be positive: {fitness}");
    }

    #[test]
    fn dispatch_biphasic_missing_params() {
        let p = json!({"dose": 5.0});
        let r = toxicology::dispatch_biphasic_dose_response(&p);
        assert_eq!(r["error"], "missing_params");
    }

    #[test]
    fn dispatch_toxicity_landscape_returns_metrics() {
        let p = json!({
            "concentration": 10.0,
            "tissue_ic50s": [50.0, 100.0],
            "tissue_sensitivities": [1.0, 0.5],
            "tissue_repairs": [0.05, 0.1]
        });
        let r = toxicology::dispatch_toxicity_landscape(&p);
        assert!(r["systemic_burden"].as_f64().is_some());
        assert!(r["tox_ipr"].as_f64().is_some());
        assert!(r["clearance_linear"].as_bool().is_some());
    }

    #[test]
    fn dispatch_hormetic_optimum_returns_dose() {
        let p = json!({
            "baseline": 100.0, "s_max": 0.3, "k_stim": 2.0,
            "ic50": 50.0, "hill_n": 2.0
        });
        let r = toxicology::dispatch_hormetic_optimum(&p);
        let dose = r["optimal_dose"].as_f64().unwrap();
        assert!(dose > 0.0, "optimal dose > 0: {dose}");
        let peak = r["peak_fitness"].as_f64().unwrap();
        assert!(peak >= 100.0, "peak ≥ baseline: {peak}");
    }

    // ── Simulation handler tests ─────────────────────────────────────

    #[test]
    fn dispatch_mechanistic_fitness_returns_hormesis_flag() {
        let p = json!({
            "dose": 2.0, "baseline": 100.0,
            "damage_ic50": 50.0, "damage_hill_n": 2.0
        });
        let r = simulation::dispatch_mechanistic_fitness(&p);
        assert!(r["fitness"].as_f64().is_some());
        assert!(r["is_hormetic"].as_bool().is_some());
    }

    #[test]
    fn dispatch_ecosystem_simulate_returns_populations() {
        let p = json!({
            "dose": 5.0, "baseline": 100.0,
            "damage_hill_n": 2.0, "t_end": 50.0
        });
        let r = simulation::dispatch_ecosystem_simulate(&p);
        let pops = r["final_populations"].as_array().unwrap();
        assert_eq!(pops.len(), 2);
    }

    // ── Microbiome handler tests (basic) ─────────────────────────────

    #[test]
    fn dispatch_shannon_returns_index() {
        let p = json!({"abundances": [0.2, 0.3, 0.5]});
        let r = microbiome::dispatch_shannon(&p);
        let h = r["shannon"].as_f64().unwrap();
        assert!(h > 0.0, "Shannon of mixed community > 0: {h}");
    }

    // ── Clinical handler tests (basic) ───────────────────────────────

    #[test]
    fn dispatch_cardiac_risk_returns_result() {
        let p = json!({
            "sdnn_ms": 80.0, "testosterone_ng_dl": 400.0, "baseline_risk": 1.0
        });
        let r = clinical::dispatch_cardiac_risk(&p);
        assert!(r.get("composite_risk").is_some());
    }
}
