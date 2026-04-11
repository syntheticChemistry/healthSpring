// SPDX-License-Identifier: AGPL-3.0-or-later
//! Comparative medicine capability handlers.

use serde_json::Value;

use super::{f, missing};
use crate::comparative::{canine, species_params};

/// Cross-species PK scaling via allometric exponents.
pub fn dispatch_cross_species_pk(params: &Value) -> Value {
    let (Some(cl_ref), Some(bw_ref), Some(bw_target)) = (
        f(params, "cl_ref"),
        f(params, "bw_ref"),
        f(params, "bw_target"),
    ) else {
        return missing("cl_ref, bw_ref, bw_target");
    };
    let vd_ref = f(params, "vd_ref").unwrap_or(1.0);
    let cl = species_params::allometric_clearance(cl_ref, bw_ref, bw_target);
    let vd = species_params::allometric_volume(vd_ref, bw_ref, bw_target);
    let half_life = species_params::allometric_half_life(vd, cl);
    serde_json::json!({
        "scaled_clearance": cl,
        "scaled_volume": vd,
        "predicted_half_life": half_life,
    })
}

/// Canine IL-31 serum kinetics under treatment.
pub fn dispatch_canine_il31(params: &Value) -> Value {
    let (Some(baseline_pg_ml), Some(t_hr)) = (
        f(params, "baseline_pg_ml"),
        f(params, "t_hr"),
    ) else {
        return missing("baseline_pg_ml, t_hr");
    };
    let treatment = match params
        .get("treatment")
        .and_then(Value::as_str)
        .unwrap_or("untreated")
    {
        "oclacitinib" => canine::CanineIl31Treatment::Oclacitinib,
        "lokivetmab" => canine::CanineIl31Treatment::Lokivetmab,
        _ => canine::CanineIl31Treatment::Untreated,
    };
    let il31 = canine::il31_serum_kinetics(baseline_pg_ml, t_hr, treatment);
    let pruritus = canine::pruritus_vas_response(il31);
    serde_json::json!({
        "il31_pg_ml": il31,
        "pruritus_vas": pruritus,
        "treatment": format!("{treatment:?}"),
    })
}

/// Canine JAK1 selectivity panel.
pub fn dispatch_canine_jak1(params: &Value) -> Value {
    let jak1 = f(params, "jak1_nm").unwrap_or(10.0);
    let jak2 = f(params, "jak2_nm").unwrap_or(1000.0);
    let jak3 = f(params, "jak3_nm").unwrap_or(1000.0);
    let tyk2 = f(params, "tyk2_nm").unwrap_or(1000.0);
    let compound = params
        .get("compound")
        .and_then(Value::as_str)
        .unwrap_or("query");

    let panel = canine::JakIc50Panel {
        compound: compound.to_owned(),
        species: "query".to_owned(),
        jak1_nm: jak1,
        jak2_nm: jak2,
        jak3_nm: jak3,
        tyk2_nm: tyk2,
    };
    let selectivity = panel.jak1_selectivity();

    let reference = canine::canine_jak_ic50_panel();
    let ref_selectivity = reference.jak1_selectivity();

    serde_json::json!({
        "jak1_selectivity": selectivity,
        "reference_selectivity": ref_selectivity,
        "reference_compound": reference.compound,
        "ratio_vs_reference": selectivity / ref_selectivity,
    })
}
