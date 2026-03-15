// SPDX-License-Identifier: AGPL-3.0-only
//! Capability dispatch — maps JSON-RPC method names to science modules.
//!
//! Each handler extracts parameters from JSON, calls the domain function,
//! and returns the result as JSON. Missing or invalid params yield an
//! `"error"` key in the result (not a JSON-RPC error envelope — the
//! binary layer handles that).

use serde_json::Value;

use crate::{biosignal, diagnostic, endocrine, microbiome, pkpd};

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

        // ── Biosignal ────────────────────────────────────────────────
        "science.biosignal.pan_tompkins" => dispatch_pan_tompkins(params),
        "science.biosignal.hrv_metrics" => dispatch_hrv(params),
        "science.biosignal.ppg_spo2" => dispatch_ppg_spo2(params),
        "science.biosignal.eda_analysis" => dispatch_eda(params),
        "science.biosignal.eda_stress_detection" => dispatch_eda_stress(params),
        "science.biosignal.arrhythmia_classification" => dispatch_arrhythmia(params),
        "science.biosignal.fuse_channels" => dispatch_fuse(params),

        // ── Endocrine ────────────────────────────────────────────────
        "science.endocrine.testosterone_pk" => dispatch_testosterone_pk(params),
        "science.endocrine.trt_outcomes" => dispatch_trt_outcomes(params),
        "science.endocrine.hrv_trt_response" => dispatch_hrv_trt(params),
        "science.endocrine.cardiac_risk" => dispatch_cardiac_risk(params),

        // ── Diagnostic ───────────────────────────────────────────────
        "science.diagnostic.assess_patient" => dispatch_assess_patient(params),
        "science.diagnostic.composite_risk" => dispatch_composite_risk(params),

        _ => return None,
    };
    Some(result)
}

// ═══════════════════════════════════════════════════════════════════════════
// Param extraction helpers
// ═══════════════════════════════════════════════════════════════════════════

fn f(params: &Value, key: &str) -> Option<f64> {
    params.get(key).and_then(Value::as_f64)
}

fn fa(params: &Value, key: &str) -> Option<Vec<f64>> {
    params.get(key).and_then(|v| {
        v.as_array().map(|arr| {
            arr.iter()
                .filter_map(Value::as_f64)
                .collect()
        })
    })
}

fn ua(params: &Value, key: &str) -> Option<Vec<u64>> {
    params.get(key).and_then(|v| {
        v.as_array().map(|arr| {
            arr.iter()
                .filter_map(Value::as_u64)
                .collect()
        })
    })
}

fn missing(name: &str) -> Value {
    serde_json::json!({"error": "missing_params", "param": name})
}

// ═══════════════════════════════════════════════════════════════════════════
// PK/PD handlers
// ═══════════════════════════════════════════════════════════════════════════

fn dispatch_hill(params: &Value) -> Value {
    let (Some(concentration), Some(ic50), Some(hill_n), Some(e_max)) =
        (f(params, "concentration"), f(params, "ic50"), f(params, "hill_n"), f(params, "e_max"))
    else {
        return missing("concentration, ic50, hill_n, e_max");
    };
    let response = pkpd::hill_dose_response(concentration, ic50, hill_n, e_max);
    let ec = pkpd::compute_ec_values(ic50, hill_n);
    serde_json::json!({
        "response": response,
        "ec10": ec.ec10, "ec50": ec.ec50, "ec90": ec.ec90,
    })
}

fn dispatch_one_compartment(params: &Value) -> Value {
    let route = params.get("route").and_then(Value::as_str).unwrap_or("iv");
    if route == "oral" {
        let (Some(dose), Some(f_bio), Some(vd), Some(ka), Some(ke), Some(t)) =
            (f(params, "dose"), f(params, "f"), f(params, "vd"),
             f(params, "ka"), f(params, "ke"), f(params, "t"))
        else {
            return missing("dose, f, vd, ka, ke, t");
        };
        let c = pkpd::pk_oral_one_compartment(dose, f_bio, vd, ka, ke, t);
        serde_json::json!({"concentration": c, "route": "oral"})
    } else {
        let (Some(dose), Some(vd), Some(half_life), Some(t)) =
            (f(params, "dose_mg"), f(params, "vd"), f(params, "half_life_hr"), f(params, "t"))
        else {
            return missing("dose_mg, vd, half_life_hr, t");
        };
        let c = pkpd::pk_iv_bolus(dose, vd, half_life, t);
        serde_json::json!({"concentration": c, "route": "iv"})
    }
}

fn dispatch_two_compartment(params: &Value) -> Value {
    let (Some(c0), Some(alpha), Some(beta), Some(k21), Some(t)) =
        (f(params, "c0"), f(params, "alpha"), f(params, "beta"),
         f(params, "k21"), f(params, "t"))
    else {
        return missing("c0, alpha, beta, k21, t");
    };
    let (a, b) = pkpd::two_compartment_ab(c0, alpha, beta, k21);
    let c = a * (-alpha * t).exp() + b * (-beta * t).exp();
    serde_json::json!({"concentration": c, "A": a, "B": b})
}

fn dispatch_pbpk(params: &Value) -> Value {
    let (Some(dose), Some(duration)) =
        (f(params, "dose_mg"), f(params, "duration_hr"))
    else {
        return missing("dose_mg, duration_hr");
    };
    let dt = f(params, "dt").unwrap_or(0.01);
    let blood_volume = f(params, "blood_volume_l").unwrap_or(5.0);
    let tissues = pkpd::standard_human_tissues();
    let (times, venous, _state) = pkpd::pbpk_iv_simulate(&tissues, dose, blood_volume, duration, dt);
    let auc = pkpd::pbpk_auc(&times, &venous);
    serde_json::json!({
        "n_steps": times.len(),
        "auc": auc,
        "peak_plasma": venous.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    })
}

fn dispatch_population_pk(params: &Value) -> Value {
    let n = params.get("n").and_then(Value::as_u64).unwrap_or(100) as usize;
    let seed = params.get("seed").and_then(Value::as_u64).unwrap_or(42);
    let times: Vec<f64> = (0..=480).map(|i| f64::from(i) * 0.1).collect();
    let results = pkpd::population_pk_monte_carlo(
        n,
        seed,
        pkpd::pop_baricitinib::CL,
        pkpd::pop_baricitinib::VD,
        pkpd::pop_baricitinib::KA,
        pkpd::pop_baricitinib::DOSE_MG,
        pkpd::pop_baricitinib::F_BIOAVAIL,
        &times,
    );
    let n_res = results.len().max(1) as f64;
    serde_json::json!({
        "n": results.len(),
        "auc_mean": results.iter().map(|r| r.auc).sum::<f64>() / n_res,
        "cmax_mean": results.iter().map(|r| r.cmax).sum::<f64>() / n_res,
    })
}

fn dispatch_allometric(params: &Value) -> Value {
    let (Some(param_animal), Some(bw_animal), Some(bw_human)) =
        (f(params, "param_animal"), f(params, "bw_animal"), f(params, "bw_human"))
    else {
        return missing("param_animal, bw_animal, bw_human");
    };
    let exponent = f(params, "exponent").unwrap_or(0.75);
    let scaled = pkpd::allometric_scale(param_animal, bw_animal, bw_human, exponent);
    serde_json::json!({"scaled_param": scaled, "exponent": exponent})
}

fn dispatch_auc(params: &Value) -> Value {
    let (Some(times), Some(concs)) = (fa(params, "times"), fa(params, "concentrations")) else {
        return missing("times, concentrations");
    };
    let auc = pkpd::auc_trapezoidal(&times, &concs);
    serde_json::json!({"auc": auc})
}

fn dispatch_nca(params: &Value) -> Value {
    let (Some(times), Some(concs)) = (fa(params, "times"), fa(params, "concentrations")) else {
        return missing("times, concentrations");
    };
    let dose = f(params, "dose").unwrap_or(100.0);
    let min_pts = params.get("min_terminal_points").and_then(Value::as_u64).unwrap_or(3) as usize;
    let r = pkpd::nca_iv(&times, &concs, dose, min_pts);
    serde_json::json!({
        "cmax": r.cmax,
        "tmax": r.tmax,
        "lambda_z": r.lambda_z,
        "half_life": r.half_life,
        "auc_last": r.auc_last,
        "auc_inf": r.auc_inf,
        "auc_extrap_pct": r.auc_extrap_pct,
        "mrt": r.mrt,
        "cl_obs": r.cl_obs,
        "vss_obs": r.vss_obs,
        "r_squared": r.r_squared,
    })
}

fn dispatch_mm(params: &Value) -> Value {
    let vmax = f(params, "vmax").unwrap_or(pkpd::PHENYTOIN_PARAMS.vmax);
    let km = f(params, "km").unwrap_or(pkpd::PHENYTOIN_PARAMS.km);
    let vd = f(params, "vd").unwrap_or(pkpd::PHENYTOIN_PARAMS.vd);
    let c0 = f(params, "c0").unwrap_or(25.0);
    let duration = f(params, "duration_hr").unwrap_or(72.0);
    let dt = f(params, "dt").unwrap_or(0.1);
    let p = pkpd::MichaelisMentenParams { vmax, km, vd };
    let (times, concs) = pkpd::mm_pk_simulate(&p, c0, duration, dt);
    let auc = pkpd::mm_auc(&concs, dt);
    serde_json::json!({
        "n_steps": times.len(),
        "auc": auc,
        "c_final": concs.last().copied().unwrap_or(0.0),
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// Microbiome handlers
// ═══════════════════════════════════════════════════════════════════════════

fn dispatch_shannon(params: &Value) -> Value {
    let Some(abundances) = fa(params, "abundances") else {
        return missing("abundances");
    };
    serde_json::json!({"shannon": microbiome::shannon_index(&abundances)})
}

fn dispatch_simpson(params: &Value) -> Value {
    let Some(abundances) = fa(params, "abundances") else {
        return missing("abundances");
    };
    let d = microbiome::simpson_index(&abundances);
    let inv = microbiome::inverse_simpson(&abundances);
    serde_json::json!({"simpson": d, "inverse_simpson": inv})
}

fn dispatch_pielou(params: &Value) -> Value {
    let Some(abundances) = fa(params, "abundances") else {
        return missing("abundances");
    };
    serde_json::json!({"pielou": microbiome::pielou_evenness(&abundances)})
}

fn dispatch_chao1(params: &Value) -> Value {
    let Some(counts) = ua(params, "counts") else {
        return missing("counts");
    };
    serde_json::json!({"chao1": microbiome::chao1(&counts)})
}

fn dispatch_anderson_gut(params: &Value) -> Value {
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

fn dispatch_colonization(params: &Value) -> Value {
    let Some(xi) = f(params, "xi") else {
        return missing("xi");
    };
    serde_json::json!({"resistance": microbiome::colonization_resistance(xi)})
}

fn dispatch_fmt_blend(params: &Value) -> Value {
    let (Some(donor), Some(recipient)) = (fa(params, "donor"), fa(params, "recipient")) else {
        return missing("donor, recipient");
    };
    let engraftment = f(params, "engraftment").unwrap_or(0.5);
    let blended = microbiome::fmt_blend(&donor, &recipient, engraftment);
    let h = microbiome::shannon_index(&blended);
    serde_json::json!({"blended": blended, "shannon": h})
}

fn dispatch_bray_curtis(params: &Value) -> Value {
    let (Some(a), Some(b)) = (fa(params, "a"), fa(params, "b")) else {
        return missing("a, b");
    };
    serde_json::json!({"bray_curtis": microbiome::bray_curtis(&a, &b)})
}

fn dispatch_antibiotic(params: &Value) -> Value {
    let h0 = f(params, "h0").unwrap_or(3.0);
    let depth = f(params, "depth").unwrap_or(0.6);
    let k_decline = f(params, "k_decline").unwrap_or(0.5);
    let k_recovery = f(params, "k_recovery").unwrap_or(0.1);
    let treatment_days = f(params, "treatment_days").unwrap_or(7.0);
    let total_days = f(params, "total_days").unwrap_or(90.0);
    let dt = f(params, "dt").unwrap_or(0.5);
    let trajectory = microbiome::antibiotic_perturbation(
        h0, depth, k_decline, k_recovery, treatment_days, total_days, dt,
    );
    let h_final = trajectory.last().map_or(h0, |&(_, h)| h);
    serde_json::json!({
        "n_steps": trajectory.len(),
        "h0": h0,
        "h_final": h_final,
        "h_nadir": trajectory.iter().map(|&(_, h)| h).fold(f64::INFINITY, f64::min),
    })
}

fn dispatch_scfa(params: &Value) -> Value {
    let fiber = f(params, "fiber_g_per_l").unwrap_or(10.0);
    let scfa_params = microbiome::SCFA_HEALTHY_PARAMS;
    let (acetate, propionate, butyrate) = microbiome::scfa_production(fiber, &scfa_params);
    serde_json::json!({
        "acetate_mM": acetate,
        "propionate_mM": propionate,
        "butyrate_mM": butyrate,
    })
}

fn dispatch_gut_brain(params: &Value) -> Value {
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

// ═══════════════════════════════════════════════════════════════════════════
// Biosignal handlers
// ═══════════════════════════════════════════════════════════════════════════

fn dispatch_pan_tompkins(params: &Value) -> Value {
    let Some(signal) = fa(params, "signal") else {
        return missing("signal");
    };
    let fs = f(params, "fs").unwrap_or(360.0);
    let result = biosignal::pan_tompkins(&signal, fs);
    let hr = biosignal::heart_rate_from_peaks(&result.peaks, fs);
    serde_json::json!({
        "peaks": result.peaks,
        "n_beats": result.peaks.len(),
        "heart_rate_bpm": hr,
    })
}

fn dispatch_hrv(params: &Value) -> Value {
    let peaks: Vec<usize> = params
        .get("peaks")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_u64).map(|v| v as usize).collect())
        .unwrap_or_default();
    if peaks.len() < 3 {
        return missing("peaks (array of at least 3 R-peak indices)");
    }
    let fs = f(params, "fs").unwrap_or(360.0);
    serde_json::json!({
        "heart_rate_bpm": biosignal::heart_rate_from_peaks(&peaks, fs),
        "sdnn_ms": biosignal::sdnn_ms(&peaks, fs),
        "rmssd_ms": biosignal::rmssd_ms(&peaks, fs),
        "pnn50": biosignal::pnn50(&peaks, fs),
    })
}

fn dispatch_ppg_spo2(params: &Value) -> Value {
    let (Some(ac_red), Some(dc_red), Some(ac_ir), Some(dc_ir)) =
        (f(params, "ac_red"), f(params, "dc_red"), f(params, "ac_ir"), f(params, "dc_ir"))
    else {
        return missing("ac_red, dc_red, ac_ir, dc_ir");
    };
    let r = biosignal::ppg_r_value(ac_red, dc_red, ac_ir, dc_ir);
    let spo2 = biosignal::spo2_from_r(r);
    serde_json::json!({"r_value": r, "spo2_percent": spo2})
}

fn dispatch_eda(params: &Value) -> Value {
    let Some(signal) = fa(params, "signal") else {
        return missing("signal");
    };
    let window = params.get("window_samples").and_then(Value::as_u64).unwrap_or(50) as usize;
    let scl = biosignal::eda_scl(&signal, window);
    let phasic = biosignal::eda_phasic(&signal, window);
    serde_json::json!({
        "scl_samples": scl.len(),
        "phasic_samples": phasic.len(),
        "scl_mean": scl.iter().sum::<f64>() / scl.len().max(1) as f64,
    })
}

fn dispatch_eda_stress(params: &Value) -> Value {
    let Some(signal) = fa(params, "signal") else {
        return missing("signal");
    };
    let min_interval = params.get("min_interval_samples").and_then(Value::as_u64).unwrap_or(50) as usize;
    let threshold = f(params, "threshold_us").unwrap_or(0.05);
    let fs = f(params, "fs").unwrap_or(4.0);
    let duration_s = signal.len() as f64 / fs;
    let scrs = biosignal::eda_detect_scr(&signal, threshold, min_interval);
    let rate = biosignal::scr_rate(scrs.len(), duration_s);
    let mean_scl = if signal.is_empty() { 0.0 } else { signal.iter().sum::<f64>() / signal.len() as f64 };
    let recovery_s = f(params, "recovery_s").unwrap_or(3.0);
    let stress_idx = biosignal::compute_stress_index(rate, mean_scl, recovery_s);
    serde_json::json!({
        "scr_count": scrs.len(),
        "scr_rate_per_min": rate,
        "stress_index": stress_idx,
    })
}

fn dispatch_arrhythmia(params: &Value) -> Value {
    let Some(signal) = fa(params, "signal") else {
        return missing("signal");
    };
    let fs = f(params, "fs").unwrap_or(360.0);
    let half_width = params.get("half_width").and_then(Value::as_u64).unwrap_or(25) as usize;
    let min_corr = f(params, "min_correlation").unwrap_or(0.7);
    let pt = biosignal::pan_tompkins(&signal, fs);

    let normal_waveform = biosignal::generate_normal_template(half_width * 2 + 1);
    let normal_template = biosignal::BeatTemplate {
        class: biosignal::BeatClass::Normal,
        waveform: normal_waveform,
    };
    let results = biosignal::classify_all_beats(&signal, &pt.peaks, &[normal_template], half_width, min_corr);
    let n_normal = results.iter().filter(|r| r.class == biosignal::BeatClass::Normal).count();
    serde_json::json!({
        "total_beats": results.len(),
        "normal": n_normal,
        "abnormal": results.len() - n_normal,
    })
}

fn dispatch_fuse(params: &Value) -> Value {
    let peaks: Vec<usize> = params
        .get("peaks")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_u64).map(|v| v as usize).collect())
        .unwrap_or_default();
    let fs = f(params, "fs").unwrap_or(360.0);
    let spo2 = f(params, "spo2").unwrap_or(98.0);
    let scr_count = params.get("scr_count").and_then(Value::as_u64).unwrap_or(0) as usize;
    let eda_duration_s = f(params, "eda_duration_s").unwrap_or(0.0);

    let assessment = biosignal::fuse_channels(&peaks, fs, spo2, scr_count, eda_duration_s);
    serde_json::json!({
        "heart_rate_bpm": assessment.heart_rate_bpm,
        "sdnn_ms": assessment.hrv_sdnn_ms,
        "rmssd_ms": assessment.hrv_rmssd_ms,
        "spo2_percent": assessment.spo2_percent,
        "stress_index": assessment.stress_index,
        "overall_score": assessment.overall_score,
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// Endocrine handlers
// ═══════════════════════════════════════════════════════════════════════════

fn dispatch_testosterone_pk(params: &Value) -> Value {
    let dose = f(params, "dose_mg").unwrap_or(endocrine::testosterone_cypionate::DOSE_WEEKLY_MG);
    let f_im = f(params, "f_im").unwrap_or(endocrine::testosterone_cypionate::F_IM);
    let vd = f(params, "vd").unwrap_or(endocrine::testosterone_cypionate::VD_L);
    let ka = f(params, "ka").unwrap_or(endocrine::testosterone_cypionate::K_A_IM);
    let ke = f(params, "ke").unwrap_or(endocrine::testosterone_cypionate::K_E);
    let t = f(params, "t").unwrap_or(0.0);
    let c = endocrine::pk_im_depot(dose, f_im, vd, ka, ke, t);
    serde_json::json!({"concentration": c, "t": t, "route": "im_depot"})
}

fn dispatch_trt_outcomes(params: &Value) -> Value {
    let month = f(params, "month").unwrap_or(12.0);
    let testosterone = f(params, "testosterone_ng_dl").unwrap_or(600.0);

    let dw = endocrine::weight_trajectory(month, -16.0, 6.0, 60.0);
    let hr = endocrine::hazard_ratio_model(testosterone, 300.0, 0.44);
    let hba1c = endocrine::hba1c_trajectory(month, 7.60, -0.37, 3.0);

    serde_json::json!({
        "month": month,
        "weight_change_kg": dw,
        "hazard_ratio": hr,
        "hba1c": hba1c,
    })
}

fn dispatch_hrv_trt(params: &Value) -> Value {
    let base_sdnn = f(params, "base_sdnn").unwrap_or(40.0);
    let delta_sdnn = f(params, "delta_sdnn").unwrap_or(20.0);
    let tau = f(params, "tau_months").unwrap_or(6.0);
    let month = f(params, "month").unwrap_or(12.0);
    let sdnn = endocrine::hrv_trt_response(base_sdnn, delta_sdnn, tau, month);
    serde_json::json!({"sdnn_ms": sdnn, "month": month})
}

fn dispatch_cardiac_risk(params: &Value) -> Value {
    let sdnn = f(params, "sdnn_ms").unwrap_or(80.0);
    let testosterone = f(params, "testosterone_ng_dl").unwrap_or(400.0);
    let baseline_risk = f(params, "baseline_risk").unwrap_or(1.0);
    let risk = endocrine::cardiac_risk_composite(sdnn, testosterone, baseline_risk);
    serde_json::json!({"composite_risk": risk})
}

// ═══════════════════════════════════════════════════════════════════════════
// Diagnostic handlers
// ═══════════════════════════════════════════════════════════════════════════

fn dispatch_assess_patient(params: &Value) -> Value {
    let age = f(params, "age_years").unwrap_or(45.0);
    let weight = f(params, "weight_kg").unwrap_or(85.0);
    let sex = match params.get("sex").and_then(Value::as_str).unwrap_or("male") {
        "female" | "Female" | "F" => diagnostic::Sex::Female,
        _ => diagnostic::Sex::Male,
    };

    let mut profile = diagnostic::PatientProfile::minimal(age, weight, sex);
    profile.testosterone_ng_dl = f(params, "testosterone_ng_dl");
    profile.on_trt = params.get("on_trt").and_then(Value::as_bool).unwrap_or(false);
    profile.trt_months = f(params, "trt_months").unwrap_or(0.0);
    profile.gut_abundances = fa(params, "gut_abundances");
    profile.ppg_spo2 = f(params, "ppg_spo2");
    profile.ecg_fs = f(params, "ecg_fs").unwrap_or(360.0);
    profile.ecg_peaks = params.get("ecg_peaks").and_then(Value::as_array).map(|arr| {
        arr.iter().filter_map(Value::as_u64).map(|v| v as usize).collect()
    });
    profile.scr_count = params.get("scr_count").and_then(Value::as_u64).map(|v| v as usize);
    profile.eda_duration_s = f(params, "eda_duration_s").unwrap_or(0.0);

    let assessment = diagnostic::assess_patient(&profile);
    serde_json::json!({
        "pk": {
            "hill_response_at_ec50": assessment.pk.hill_response_at_ec50,
            "oral_cmax": assessment.pk.oral_cmax,
            "oral_tmax_hr": assessment.pk.oral_tmax_hr,
            "oral_auc": assessment.pk.oral_auc,
        },
        "microbiome": {
            "shannon": assessment.microbiome.shannon,
            "pielou": assessment.microbiome.pielou_evenness,
            "colonization_resistance": assessment.microbiome.colonization_resistance,
        },
        "biosignal": {
            "heart_rate_bpm": assessment.biosignal.heart_rate_bpm,
            "sdnn_ms": assessment.biosignal.sdnn_ms,
            "spo2_percent": assessment.biosignal.spo2_percent,
        },
        "endocrine": {
            "predicted_testosterone": assessment.endocrine.predicted_testosterone,
            "cardiac_risk": assessment.endocrine.cardiac_risk,
        },
    })
}

fn dispatch_composite_risk(params: &Value) -> Value {
    let age = f(params, "age_years").unwrap_or(45.0);
    let weight = f(params, "weight_kg").unwrap_or(85.0);
    let sex = match params.get("sex").and_then(Value::as_str).unwrap_or("male") {
        "female" | "Female" | "F" => diagnostic::Sex::Female,
        _ => diagnostic::Sex::Male,
    };
    let mut profile = diagnostic::PatientProfile::minimal(age, weight, sex);
    profile.testosterone_ng_dl = f(params, "testosterone_ng_dl");
    profile.gut_abundances = fa(params, "gut_abundances");

    let assessment = diagnostic::assess_patient(&profile);
    serde_json::json!({
        "composite_risk": assessment.composite_risk,
        "microbiome_risk": format!("{:?}", assessment.microbiome.risk_level),
        "cardiac_risk": assessment.endocrine.cardiac_risk,
    })
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
        let response = v.get("response").and_then(Value::as_f64).expect("has response");
        assert!((response - 0.5).abs() < 1e-10);
    }

    #[test]
    fn shannon_dispatch_works() {
        let params = serde_json::json!({"abundances": [0.25, 0.25, 0.25, 0.25]});
        let result = dispatch_science("science.microbiome.shannon_index", &params);
        assert!(result.is_some());
        let v = result.expect("dispatch returned Some");
        let h = v.get("shannon").and_then(Value::as_f64).expect("has shannon");
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
