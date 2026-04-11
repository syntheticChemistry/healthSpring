// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]

//! Exp113 — Composition validation: Microbiome dispatch parity.
//!
//! Validates IPC dispatch layer produces identical results to direct Rust
//! calls for all microbiome science methods. Tier 4 composition validation.

use healthspring_barracuda::ipc::dispatch::dispatch_science;
use healthspring_barracuda::microbiome;
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::ValidationHarness;

fn validate_shannon_simpson(h: &mut ValidationHarness) {
    let abundances = vec![0.4, 0.3, 0.2, 0.1];
    let uniform = vec![0.25, 0.25, 0.25, 0.25];

    let direct_shannon = microbiome::shannon_index(&abundances);
    let dispatched = dispatch_science(
        "science.microbiome.shannon_index",
        &serde_json::json!({"abundances": abundances}),
    )
    .and_then(|v| v.get("shannon").and_then(serde_json::Value::as_f64));
    if let Some(ipc_val) = dispatched {
        h.check_abs(
            "Shannon IPC parity",
            ipc_val,
            direct_shannon,
            tolerances::DETERMINISM,
        );
    } else {
        h.check_bool("Shannon dispatch returned result", false);
    }

    let direct_uniform = microbiome::shannon_index(&uniform);
    let dispatched_uni = dispatch_science(
        "science.microbiome.shannon_index",
        &serde_json::json!({"abundances": uniform}),
    )
    .and_then(|v| v.get("shannon").and_then(serde_json::Value::as_f64));
    if let Some(ipc_val) = dispatched_uni {
        h.check_abs(
            "Shannon uniform IPC parity",
            ipc_val,
            direct_uniform,
            tolerances::DETERMINISM,
        );
        h.check_abs(
            "Shannon uniform = ln(4)",
            ipc_val,
            4.0_f64.ln(),
            tolerances::MACHINE_EPSILON,
        );
    } else {
        h.check_bool("Shannon uniform dispatch returned result", false);
        h.check_bool("Shannon uniform = ln(4)", false);
    }

    let direct_simpson = microbiome::simpson_index(&abundances);
    let dispatched_sim = dispatch_science(
        "science.microbiome.simpson_index",
        &serde_json::json!({"abundances": abundances}),
    )
    .and_then(|v| v.get("simpson").and_then(serde_json::Value::as_f64));
    if let Some(ipc_val) = dispatched_sim {
        h.check_abs(
            "Simpson IPC parity",
            ipc_val,
            direct_simpson,
            tolerances::DETERMINISM,
        );
    } else {
        h.check_bool("Simpson dispatch returned result", false);
    }
}

fn validate_richness_and_ecology(h: &mut ValidationHarness) {
    let abundances = vec![0.4, 0.3, 0.2, 0.1];
    let counts: Vec<u64> = vec![10, 5, 3, 1, 1, 1, 1, 1];

    let direct_pielou = microbiome::pielou_evenness(&abundances);
    let dispatched_pie = dispatch_science(
        "science.microbiome.pielou_evenness",
        &serde_json::json!({"abundances": abundances}),
    )
    .and_then(|v| v.get("pielou").and_then(serde_json::Value::as_f64));
    if let Some(ipc_val) = dispatched_pie {
        h.check_abs(
            "Pielou IPC parity",
            ipc_val,
            direct_pielou,
            tolerances::DETERMINISM,
        );
    } else {
        h.check_bool("Pielou dispatch returned result", false);
    }

    let direct_chao1 = microbiome::chao1(&counts);
    let dispatched_chao = dispatch_science(
        "science.microbiome.chao1",
        &serde_json::json!({"counts": counts}),
    )
    .and_then(|v| v.get("chao1").and_then(serde_json::Value::as_f64));
    if let Some(ipc_val) = dispatched_chao {
        h.check_abs(
            "Chao1 IPC parity",
            ipc_val,
            direct_chao1,
            tolerances::DETERMINISM,
        );
    } else {
        h.check_bool("Chao1 dispatch returned result", false);
    }

    let direct_cr = microbiome::colonization_resistance(5.0);
    let dispatched_cr = dispatch_science(
        "science.microbiome.colonization_resistance",
        &serde_json::json!({"xi": 5.0}),
    )
    .and_then(|v| v.get("resistance").and_then(serde_json::Value::as_f64));
    if let Some(ipc_val) = dispatched_cr {
        h.check_abs(
            "Colonization IPC parity",
            ipc_val,
            direct_cr,
            tolerances::DETERMINISM,
        );
    } else {
        h.check_bool("Colonization dispatch returned result", false);
    }
}

fn validate_anderson(h: &mut ValidationHarness) {
    let disorder = vec![1.0, 0.5, 2.0, 1.5, 0.8];
    let (direct_eig, direct_ipr) = microbiome::anderson_diagonalize(&disorder, 1.0);
    let dispatched_anderson = dispatch_science(
        "science.microbiome.anderson_gut",
        &serde_json::json!({"disorder": disorder, "t_hop": 1.0}),
    );
    if let Some(v) = &dispatched_anderson {
        if let Some(eig_arr) = v.get("eigenvalues").and_then(serde_json::Value::as_array) {
            let n = direct_eig.len().min(eig_arr.len());
            let mut eig_match = true;
            for i in 0..n {
                if let Some(e) = eig_arr[i].as_f64() {
                    if (e - direct_eig[i]).abs() > tolerances::DETERMINISM {
                        eig_match = false;
                    }
                }
            }
            h.check_bool("Anderson eigenvalues IPC parity", eig_match);
        } else {
            h.check_bool("Anderson eigenvalues returned", false);
        }
        if let Some(ipr_arr) = v.get("ipr").and_then(serde_json::Value::as_array) {
            let n = direct_ipr.len().min(ipr_arr.len());
            let mut ipr_match = true;
            for i in 0..n {
                if let Some(ip) = ipr_arr[i].as_f64() {
                    if (ip - direct_ipr[i]).abs() > tolerances::DETERMINISM {
                        ipr_match = false;
                    }
                }
            }
            h.check_bool("Anderson IPR IPC parity", ipr_match);
        } else {
            h.check_bool("Anderson IPR returned", false);
        }
    } else {
        h.check_bool("Anderson gut dispatch returned result", false);
        h.check_bool("Anderson IPR returned", false);
    }
}

fn main() {
    let mut h = ValidationHarness::new("Exp113 Composition Microbiome Dispatch Parity");

    validate_shannon_simpson(&mut h);
    validate_richness_and_ecology(&mut h);
    validate_anderson(&mut h);

    let p = serde_json::json!({"abundances": [0.5, 0.3, 0.2]});
    let r1 = dispatch_science("science.microbiome.shannon_index", &p)
        .and_then(|v| v.get("shannon").and_then(serde_json::Value::as_f64));
    let r2 = dispatch_science("science.microbiome.shannon_index", &p)
        .and_then(|v| v.get("shannon").and_then(serde_json::Value::as_f64));
    if let (Some(a), Some(b)) = (r1, r2) {
        h.check_abs(
            "Shannon dispatch determinism",
            a,
            b,
            tolerances::DETERMINISM,
        );
    } else {
        h.check_bool("Shannon dispatch determinism (both returned)", false);
    }

    h.exit();
}
