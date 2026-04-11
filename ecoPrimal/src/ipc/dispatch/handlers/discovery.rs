// SPDX-License-Identifier: AGPL-3.0-or-later
//! Drug discovery capability handlers.

use serde_json::Value;

use super::{f, fa, missing};
use crate::discovery::{compound, fibrosis, hts, matrix_score};

/// MATRIX drug repurposing score with Anderson geometry.
pub fn dispatch_matrix_score(params: &Value) -> Value {
    let (Some(on_target_ic50), Some(off_target_ic50s)) =
        (f(params, "on_target_ic50"), fa(params, "off_target_ic50s"))
    else {
        return missing("on_target_ic50, off_target_ic50s");
    };
    let localization_length = f(params, "localization_length").unwrap_or(10.0);
    let tissue_thickness = f(params, "tissue_thickness").unwrap_or(5.0);
    let w_baseline = f(params, "w_baseline").unwrap_or(3.0);
    let w_treated = f(params, "w_treated").unwrap_or(2.0);

    let pathway = matrix_score::pathway_selectivity_score(on_target_ic50, &off_target_ic50s);
    let geometry = matrix_score::tissue_geometry_factor(localization_length, tissue_thickness);
    let disorder = matrix_score::disorder_impact_factor(w_baseline, w_treated);
    let combined = matrix_score::matrix_combined_score(pathway, geometry, disorder);

    serde_json::json!({
        "pathway_score": pathway,
        "tissue_geometry": geometry,
        "disorder_factor": disorder,
        "combined_score": combined,
    })
}

/// High-throughput screening analysis.
pub fn dispatch_hts_analysis(params: &Value) -> Value {
    let (Some(pos_mean), Some(pos_std), Some(neg_mean), Some(neg_std)) = (
        f(params, "pos_mean"),
        f(params, "pos_std"),
        f(params, "neg_mean"),
        f(params, "neg_std"),
    ) else {
        return missing("pos_mean, pos_std, neg_mean, neg_std");
    };

    let z_prime = hts::z_prime_factor(pos_mean, pos_std, neg_mean, neg_std);

    let signals = fa(params, "signals").unwrap_or_default();
    let sample_stds = fa(params, "sample_stds").unwrap_or_default();

    let hits = if signals.len() == sample_stds.len() && !signals.is_empty() {
        hts::classify_hits(&signals, &sample_stds, neg_mean, neg_std, pos_mean)
    } else {
        Vec::new()
    };

    let hit_count = hits
        .iter()
        .filter(|h| h.classification != hts::HitClass::Inactive)
        .count();

    serde_json::json!({
        "z_prime": z_prime,
        "total_compounds": signals.len(),
        "hits": hit_count,
        "hit_details": hits.iter().map(|h| serde_json::json!({
            "index": h.index,
            "percent_inhibition": h.percent_inhibition,
            "ssmd": h.ssmd_value,
            "classification": format!("{:?}", h.classification),
        })).collect::<Vec<_>>(),
    })
}

/// Compound library IC50 profiling.
pub fn dispatch_compound_library(params: &Value) -> Value {
    let (Some(concentrations), Some(responses)) =
        (fa(params, "concentrations"), fa(params, "responses"))
    else {
        return missing("concentrations, responses");
    };
    let estimate = compound::estimate_ic50(&concentrations, &responses);
    serde_json::json!({
        "ic50": estimate.ic50,
        "hill_n": estimate.hill_n,
        "emax": estimate.emax,
        "r_squared": estimate.r_squared,
    })
}

/// Fibrosis pathway scoring for anti-fibrotic compounds.
pub fn dispatch_fibrosis_pathway(params: &Value) -> Value {
    let (Some(concentration_um), Some(rho_ic50), Some(mrtf_ic50), Some(srf_ic50)) = (
        f(params, "concentration_um"),
        f(params, "rho_ic50_um"),
        f(params, "mrtf_ic50_um"),
        f(params, "srf_ic50_um"),
    ) else {
        return missing("concentration_um, rho_ic50_um, mrtf_ic50_um, srf_ic50_um");
    };
    let compound_name = params
        .get("compound")
        .and_then(Value::as_str)
        .unwrap_or("query");

    let compound_profile = fibrosis::AntiFibroticCompound {
        name: compound_name.to_owned(),
        rho_ic50_um: rho_ic50,
        mrtf_ic50_um: mrtf_ic50,
        srf_ic50_um: srf_ic50,
    };
    let score = fibrosis::score_anti_fibrotic(&compound_profile, concentration_um);

    let localization_length = f(params, "localization_length").unwrap_or(10.0);
    let tissue_thickness = f(params, "tissue_thickness").unwrap_or(5.0);
    let geometry = fibrosis::fibrotic_geometry_factor(localization_length, tissue_thickness);

    serde_json::json!({
        "compound": score.compound,
        "rho_inhibition": score.rho_inhibition,
        "mrtf_block": score.mrtf_block,
        "srf_reduction": score.srf_reduction,
        "anti_fibrotic_score": score.anti_fibrotic_score,
        "fibrotic_geometry": geometry,
    })
}
