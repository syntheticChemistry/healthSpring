// SPDX-License-Identifier: AGPL-3.0-or-later
//! Microbiome and QS capability handlers.

use serde_json::Value;

use crate::{data::NcbiProvider, endocrine, microbiome, qs};

use super::{f, fa, missing, ua};

pub fn dispatch_shannon(params: &Value) -> Value {
    let Some(abundances) = fa(params, "abundances") else {
        return missing("abundances");
    };
    serde_json::json!({"shannon": microbiome::shannon_index(&abundances)})
}

pub fn dispatch_simpson(params: &Value) -> Value {
    let Some(abundances) = fa(params, "abundances") else {
        return missing("abundances");
    };
    let d = microbiome::simpson_index(&abundances);
    let inv = microbiome::inverse_simpson(&abundances);
    serde_json::json!({"simpson": d, "inverse_simpson": inv})
}

pub fn dispatch_pielou(params: &Value) -> Value {
    let Some(abundances) = fa(params, "abundances") else {
        return missing("abundances");
    };
    serde_json::json!({"pielou": microbiome::pielou_evenness(&abundances)})
}

pub fn dispatch_chao1(params: &Value) -> Value {
    let Some(counts) = ua(params, "counts") else {
        return missing("counts");
    };
    serde_json::json!({"chao1": microbiome::chao1(&counts)})
}

pub fn dispatch_anderson_gut(params: &Value) -> Value {
    let Some(disorder) = fa(params, "disorder") else {
        return missing("disorder");
    };
    let t_hop = f(params, "t_hop").unwrap_or(1.0);
    let (eigenvalues, ipr_values) = microbiome::anderson_diagonalize(&disorder, t_hop);
    serde_json::json!({
        "n_sites": disorder.len(),
        "eigenvalues": eigenvalues,
        "ipr": ipr_values,
        "mean_ipr": ipr_values.iter().sum::<f64>() / ipr_values.len() as f64,
    })
}

pub fn dispatch_colonization(params: &Value) -> Value {
    let Some(xi) = f(params, "xi") else {
        return missing("xi");
    };
    serde_json::json!({"resistance": microbiome::colonization_resistance(xi)})
}

pub fn dispatch_fmt_blend(params: &Value) -> Value {
    let (Some(donor), Some(recipient)) = (fa(params, "donor"), fa(params, "recipient")) else {
        return missing("donor, recipient");
    };
    let engraftment = f(params, "engraftment").unwrap_or(0.5);
    let blended = microbiome::fmt_blend(&donor, &recipient, engraftment);
    let h = microbiome::shannon_index(&blended);
    serde_json::json!({"blended": blended, "shannon": h})
}

pub fn dispatch_bray_curtis(params: &Value) -> Value {
    let (Some(a), Some(b)) = (fa(params, "a"), fa(params, "b")) else {
        return missing("a, b");
    };
    serde_json::json!({"bray_curtis": microbiome::bray_curtis(&a, &b)})
}

pub fn dispatch_antibiotic(params: &Value) -> Value {
    let h0 = f(params, "h0").unwrap_or(3.0);
    let depth = f(params, "depth").unwrap_or(0.6);
    let k_decline = f(params, "k_decline").unwrap_or(0.5);
    let k_recovery = f(params, "k_recovery").unwrap_or(0.1);
    let treatment_days = f(params, "treatment_days").unwrap_or(7.0);
    let total_days = f(params, "total_days").unwrap_or(90.0);
    let dt = f(params, "dt").unwrap_or(0.5);
    let trajectory = microbiome::antibiotic_perturbation(
        h0,
        depth,
        k_decline,
        k_recovery,
        treatment_days,
        total_days,
        dt,
    );
    let h_final = trajectory.last().map_or(h0, |&(_, h)| h);
    serde_json::json!({
        "n_steps": trajectory.len(),
        "h0": h0,
        "h_final": h_final,
        "h_nadir": trajectory.iter().map(|&(_, h)| h).fold(f64::INFINITY, f64::min),
    })
}

pub fn dispatch_scfa(params: &Value) -> Value {
    let fiber = f(params, "fiber_g_per_l").unwrap_or(10.0);
    let scfa_params = microbiome::SCFA_HEALTHY_PARAMS;
    let (acetate, propionate, butyrate) = microbiome::scfa_production(fiber, &scfa_params);
    serde_json::json!({
        "acetate_mM": acetate,
        "propionate_mM": propionate,
        "butyrate_mM": butyrate,
    })
}

pub fn dispatch_gut_brain(params: &Value) -> Value {
    let Some(abundances) = fa(params, "abundances") else {
        return missing("abundances");
    };
    let h = microbiome::shannon_index(&abundances);
    let j = microbiome::pielou_evenness(&abundances);
    let w = microbiome::evenness_to_disorder(j, 5.0);
    let xi = endocrine::anderson_localization_length(w, abundances.len() as f64);
    let serotonin_proxy = 1.0 - (-xi / 20.0).exp();
    serde_json::json!({
        "shannon": h,
        "pielou": j,
        "disorder_w": w,
        "localization_xi": xi,
        "serotonin_proxy": serotonin_proxy,
    })
}

pub fn dispatch_qs_profile(params: &Value) -> Value {
    let Some(abundances) = fa(params, "abundances") else {
        return missing("abundances");
    };
    let provider = NcbiProvider::discover();
    let Ok(matrix) = provider.load_qs_matrix() else {
        return serde_json::json!({
            "error": "qs_matrix_unavailable",
            "message": "QS gene matrix not found. Run data/fetch_qs_genes.py or set HEALTHSPRING_DATA_ROOT.",
        });
    };
    let profile = qs::qs_profile(&abundances, &matrix);
    serde_json::json!({
        "family_densities": profile.family_densities,
        "total_qs_density": profile.total_qs_density,
        "signaling_diversity": profile.signaling_diversity,
    })
}

pub fn dispatch_qs_effective_disorder(params: &Value) -> Value {
    let Some(pielou_j) = f(params, "pielou_j") else {
        return missing("pielou_j");
    };
    let Some(abundances) = fa(params, "abundances") else {
        return missing("abundances");
    };
    let provider = NcbiProvider::discover();
    let Ok(matrix) = provider.load_qs_matrix() else {
        return serde_json::json!({
            "error": "qs_matrix_unavailable",
            "message": "QS gene matrix not found. Run data/fetch_qs_genes.py or set HEALTHSPRING_DATA_ROOT.",
        });
    };
    let profile = qs::qs_profile(&abundances, &matrix);
    let alpha = f(params, "alpha").unwrap_or(0.7);
    let w_scale = f(params, "w_scale").unwrap_or(5.0);
    let w_effective = qs::effective_disorder(pielou_j, &profile, alpha, w_scale);
    serde_json::json!({
        "effective_disorder": w_effective,
        "total_qs_density": profile.total_qs_density,
    })
}
