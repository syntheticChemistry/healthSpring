// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]

//! Exp112 — Composition validation: PK/PD dispatch parity.
//!
//! Validates that calling PK/PD science methods through the IPC dispatch
//! layer (as biomeOS would via JSON-RPC) produces identical results to
//! direct Rust function calls. This is the Tier 4 composition validation:
//!
//!   Python baseline → Rust validation → **IPC dispatch parity**
//!
//! Every check compares `dispatch_science(method, params)` output against
//! the direct Rust function call that was already validated against Python.

use healthspring_barracuda::ipc::dispatch::dispatch_science;
use healthspring_barracuda::pkpd::{self, ALL_INHIBITORS};
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::ValidationHarness;

fn validate_hill_parity(h: &mut ValidationHarness) {
    for drug in ALL_INHIBITORS {
        let direct =
            pkpd::hill_dose_response(drug.ic50_jak1_nm, drug.ic50_jak1_nm, drug.hill_n, 1.0);

        let params = serde_json::json!({
            "concentration": drug.ic50_jak1_nm,
            "ic50": drug.ic50_jak1_nm,
            "hill_n": drug.hill_n,
            "e_max": 1.0,
        });
        let dispatched = dispatch_science("science.pkpd.hill_dose_response", &params)
            .and_then(|v| v.get("response").and_then(serde_json::Value::as_f64));

        if let Some(ipc_val) = dispatched {
            h.check_abs(
                &format!("{} Hill IPC parity", drug.name),
                ipc_val,
                direct,
                tolerances::DETERMINISM,
            );
        } else {
            h.check_bool(
                &format!("{} Hill dispatch returned result", drug.name),
                false,
            );
        }
    }
}

fn validate_compartment_and_auc(h: &mut ValidationHarness) {
    let direct_iv = pkpd::pk_iv_bolus(100.0, 10.0, 6.93, 0.0);
    let params_iv = serde_json::json!({
        "dose_mg": 100.0, "vd": 10.0, "half_life_hr": 6.93, "t": 0.0,
    });
    let dispatched_iv = dispatch_science("science.pkpd.one_compartment_pk", &params_iv)
        .and_then(|v| v.get("concentration").and_then(serde_json::Value::as_f64));
    if let Some(ipc_val) = dispatched_iv {
        h.check_abs(
            "1-comp IV C(0) IPC parity",
            ipc_val,
            direct_iv,
            tolerances::DETERMINISM,
        );
    } else {
        h.check_bool("1-comp IV dispatch returned result", false);
    }

    let times = vec![0.0, 1.0, 2.0, 3.0];
    let concs = vec![10.0, 7.0, 3.0, 1.0];
    let direct_auc = pkpd::auc_trapezoidal(&times, &concs);
    let params_auc = serde_json::json!({
        "times": times, "concentrations": concs,
    });
    let dispatched_auc = dispatch_science("science.pkpd.auc_trapezoidal", &params_auc)
        .and_then(|v| v.get("auc").and_then(serde_json::Value::as_f64));
    if let Some(ipc_val) = dispatched_auc {
        h.check_abs(
            "AUC trapezoidal IPC parity",
            ipc_val,
            direct_auc,
            tolerances::DETERMINISM,
        );
    } else {
        h.check_bool("AUC dispatch returned result", false);
    }

    let direct_allo = pkpd::allometric_scale(10.0, 70.0, 70.0, 0.75);
    let params_allo = serde_json::json!({
        "param_animal": 10.0, "bw_animal": 70.0, "bw_human": 70.0,
    });
    let dispatched_allo = dispatch_science("science.pkpd.allometric_scale", &params_allo)
        .and_then(|v| v.get("scaled_param").and_then(serde_json::Value::as_f64));
    if let Some(ipc_val) = dispatched_allo {
        h.check_abs(
            "Allometric IPC parity",
            ipc_val,
            direct_allo,
            tolerances::DETERMINISM,
        );
    } else {
        h.check_bool("Allometric dispatch returned result", false);
    }
}

fn validate_mm_and_pop(h: &mut ValidationHarness) {
    let mm_params = pkpd::PHENYTOIN_PARAMS;
    let (_, direct_concs) = pkpd::mm_pk_simulate(&mm_params, 25.0, 72.0, 0.1);
    let direct_auc_mm = pkpd::mm_auc(&direct_concs, 0.1);
    let params_mm = serde_json::json!({
        "c0": 25.0, "duration_hr": 72.0, "dt": 0.1,
    });
    let dispatched_mm = dispatch_science("science.pkpd.michaelis_menten_nonlinear", &params_mm);
    if let Some(v) = &dispatched_mm {
        if let Some(ipc_auc) = v.get("auc").and_then(serde_json::Value::as_f64) {
            h.check_abs(
                "MM AUC IPC parity",
                ipc_auc,
                direct_auc_mm,
                tolerances::DETERMINISM,
            );
        } else {
            h.check_bool("MM dispatch returned auc", false);
        }
        if let Some(c_final) = v.get("c_final").and_then(serde_json::Value::as_f64) {
            let direct_final = direct_concs.last().copied().unwrap_or(0.0);
            h.check_abs(
                "MM c_final IPC parity",
                c_final,
                direct_final,
                tolerances::DETERMINISM,
            );
        } else {
            h.check_bool("MM dispatch returned c_final", false);
        }
    } else {
        h.check_bool("MM dispatch returned result", false);
        h.check_bool("MM dispatch returned c_final", false);
    }

    let params_pop = serde_json::json!({"n": 10, "seed": 42});
    let dispatched_pop = dispatch_science("science.pkpd.population_pk", &params_pop);
    if let Some(v) = &dispatched_pop {
        let has_auc_mean = v
            .get("auc_mean")
            .and_then(serde_json::Value::as_f64)
            .is_some();
        let has_n = v.get("n").and_then(serde_json::Value::as_u64) == Some(10);
        h.check_bool("PopPK IPC returns auc_mean", has_auc_mean);
        h.check_bool("PopPK IPC returns correct n", has_n);
    } else {
        h.check_bool("PopPK dispatch returned result", false);
        h.check_bool("PopPK IPC returns correct n", false);
    }
}

fn main() {
    let mut h = ValidationHarness::new("Exp112 Composition PK/PD Dispatch Parity");

    validate_hill_parity(&mut h);
    validate_compartment_and_auc(&mut h);
    validate_mm_and_pop(&mut h);

    let params_det = serde_json::json!({
        "concentration": 15.0, "ic50": 10.0, "hill_n": 2.0, "e_max": 1.0,
    });
    let run1 = dispatch_science("science.pkpd.hill_dose_response", &params_det)
        .and_then(|v| v.get("response").and_then(serde_json::Value::as_f64));
    let run2 = dispatch_science("science.pkpd.hill_dose_response", &params_det)
        .and_then(|v| v.get("response").and_then(serde_json::Value::as_f64));
    if let (Some(r1), Some(r2)) = (run1, run2) {
        h.check_abs("Hill dispatch determinism", r1, r2, tolerances::DETERMINISM);
    } else {
        h.check_bool("Hill dispatch determinism (both returned)", false);
    }

    h.exit();
}
