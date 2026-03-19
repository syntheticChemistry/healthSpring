// SPDX-License-Identifier: AGPL-3.0-or-later
//! Toxicology and hormesis capability handlers.

use serde_json::Value;

use super::{f, fa, missing};
use crate::toxicology;

pub fn dispatch_biphasic_dose_response(params: &Value) -> Value {
    let (Some(dose), Some(baseline), Some(s_max), Some(k_stim), Some(ic50), Some(hill_n)) = (
        f(params, "dose"),
        f(params, "baseline"),
        f(params, "s_max"),
        f(params, "k_stim"),
        f(params, "ic50"),
        f(params, "hill_n"),
    ) else {
        return missing("dose, baseline, s_max, k_stim, ic50, hill_n");
    };
    let response = toxicology::biphasic_dose_response(dose, baseline, s_max, k_stim, ic50, hill_n);
    serde_json::json!({"fitness": response})
}

pub fn dispatch_toxicity_landscape(params: &Value) -> Value {
    let (Some(concentration), Some(ic50s), Some(sensitivities), Some(repairs)) = (
        f(params, "concentration"),
        fa(params, "tissue_ic50s"),
        fa(params, "tissue_sensitivities"),
        fa(params, "tissue_repairs"),
    ) else {
        return missing("concentration, tissue_ic50s, tissue_sensitivities, tissue_repairs");
    };
    let hill_n = f(params, "hill_n").unwrap_or(1.0);
    let km = f(params, "km").unwrap_or(10.0);
    let threshold = f(params, "clearance_threshold").unwrap_or(0.20);
    let landscape = toxicology::compute_toxicity_landscape(
        concentration,
        &ic50s,
        &sensitivities,
        &repairs,
        hill_n,
        km,
        threshold,
    );
    serde_json::json!({
        "systemic_burden": landscape.systemic_burden,
        "excess_burden": landscape.excess_burden,
        "tox_ipr": landscape.tox_ipr,
        "localization_length": landscape.localization_length,
        "max_clearance_utilization": landscape.max_clearance_utilization,
        "clearance_linear": landscape.clearance_linear,
        "delocalization_advantage": landscape.delocalization_advantage,
    })
}

pub fn dispatch_hormetic_optimum(params: &Value) -> Value {
    let (Some(baseline), Some(s_max), Some(k_stim), Some(ic50), Some(hill_n)) = (
        f(params, "baseline"),
        f(params, "s_max"),
        f(params, "k_stim"),
        f(params, "ic50"),
        f(params, "hill_n"),
    ) else {
        return missing("baseline, s_max, k_stim, ic50, hill_n");
    };
    let dose_max = f(params, "dose_max").unwrap_or(100.0);
    let (opt_dose, peak) =
        toxicology::hormetic_optimum(baseline, s_max, k_stim, ic50, hill_n, dose_max, 10000);
    serde_json::json!({"optimal_dose": opt_dose, "peak_fitness": peak})
}
