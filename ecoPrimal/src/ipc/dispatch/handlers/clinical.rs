// SPDX-License-Identifier: AGPL-3.0-or-later
//! Endocrine, diagnostic, and clinical capability handlers.

use serde_json::Value;

use crate::{diagnostic, endocrine, pkpd};

use super::{f, fa, sza, sz_or};

fn percentile_sorted(v: &mut [f64], p: f64) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        reason = "percentile index is always non-negative and fits usize"
    )]
    let idx = (p / 100.0 * (v.len() - 1) as f64).round() as usize;
    v.get(idx).copied().unwrap_or(0.0)
}

fn build_profile_from_params(params: &Value) -> diagnostic::PatientProfile {
    let age = f(params, "age_years").unwrap_or(45.0);
    let weight = f(params, "weight_kg").unwrap_or(85.0);
    let sex = match params.get("sex").and_then(Value::as_str).unwrap_or("male") {
        "female" | "Female" | "F" => diagnostic::Sex::Female,
        _ => diagnostic::Sex::Male,
    };
    let mut profile = diagnostic::PatientProfile::minimal(age, weight, sex);
    profile.testosterone_ng_dl = f(params, "testosterone_ng_dl");
    profile.on_trt = params
        .get("on_trt")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    profile.trt_months = f(params, "trt_months").unwrap_or(0.0);
    profile.gut_abundances = fa(params, "gut_abundances");
    profile.ppg_spo2 = f(params, "ppg_spo2");
    profile.ecg_fs = f(params, "ecg_fs").unwrap_or(360.0);
    profile.ecg_peaks = sza(params, "ecg_peaks");
    profile.scr_count = super::sz(params, "scr_count");
    profile.eda_duration_s = f(params, "eda_duration_s").unwrap_or(0.0);
    profile
}

// ── Endocrine ─────────────────────────────────────────────────────────────

pub fn dispatch_testosterone_pk(params: &Value) -> Value {
    let dose = f(params, "dose_mg").unwrap_or(endocrine::testosterone_cypionate::DOSE_WEEKLY_MG);
    let f_im = f(params, "f_im").unwrap_or(endocrine::testosterone_cypionate::F_IM);
    let vd = f(params, "vd").unwrap_or(endocrine::testosterone_cypionate::VD_L);
    let ka = f(params, "ka").unwrap_or(endocrine::testosterone_cypionate::K_A_IM);
    let ke = f(params, "ke").unwrap_or(endocrine::testosterone_cypionate::K_E);
    let t = f(params, "t").unwrap_or(0.0);
    let c = endocrine::pk_im_depot(dose, f_im, vd, ka, ke, t);
    serde_json::json!({"concentration": c, "t": t, "route": "im_depot"})
}

pub fn dispatch_trt_outcomes(params: &Value) -> Value {
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

pub fn dispatch_hrv_trt(params: &Value) -> Value {
    let base_sdnn = f(params, "base_sdnn").unwrap_or(40.0);
    let delta_sdnn = f(params, "delta_sdnn").unwrap_or(20.0);
    let tau = f(params, "tau_months").unwrap_or(6.0);
    let month = f(params, "month").unwrap_or(12.0);
    let sdnn = endocrine::hrv_trt_response(base_sdnn, delta_sdnn, tau, month);
    serde_json::json!({"sdnn_ms": sdnn, "month": month})
}

pub fn dispatch_cardiac_risk(params: &Value) -> Value {
    let sdnn = f(params, "sdnn_ms").unwrap_or(80.0);
    let testosterone = f(params, "testosterone_ng_dl").unwrap_or(400.0);
    let baseline_risk = f(params, "baseline_risk").unwrap_or(1.0);
    let risk = endocrine::cardiac_risk_composite(sdnn, testosterone, baseline_risk);
    serde_json::json!({"composite_risk": risk})
}

pub fn dispatch_population_trt(params: &Value) -> Value {
    use endocrine::pop_trt;
    use endocrine::testosterone_cypionate as tc;

    let n_patients = sz_or(params, "n_patients", 100);
    let seed = params.get("seed").and_then(Value::as_u64).unwrap_or(42);
    let age_min = f(params, "age_min").unwrap_or(40.0);
    let age_max = f(params, "age_max").unwrap_or(75.0);

    let times: Vec<f64> = (0..=500).map(|i| 56.0 * f64::from(i) / 500.0).collect();
    let (mu_vd, sig_vd) = endocrine::lognormal_params(pop_trt::VD_TYPICAL, pop_trt::VD_CV);
    let (mu_ka, sig_ka) = endocrine::lognormal_params(pop_trt::KA_TYPICAL, pop_trt::KA_CV);
    let (mu_elim, sig_elim) = endocrine::lognormal_params(pop_trt::KE_TYPICAL, pop_trt::KE_CV);

    let mut rng = seed;
    let mut cmax_arr = Vec::with_capacity(n_patients);
    let mut auc_arr = Vec::with_capacity(n_patients);

    for i in 0..n_patients {
        let (z1, r1) = crate::rng::normal_sample(rng);
        rng = r1;
        let (z2, r2) = crate::rng::normal_sample(rng);
        rng = r2;
        let (z3, r3) = crate::rng::normal_sample(rng);
        rng = r3;
        let _age = age_min + (age_max - age_min) * crate::validation::len_f64(i) / crate::validation::len_f64(n_patients.max(1));
        let vd = sig_vd.mul_add(z1, mu_vd).exp();
        let ka = sig_ka.mul_add(z2, mu_ka).exp();
        let ke = sig_elim.mul_add(z3, mu_elim).exp();

        let concs = pkpd::pk_multiple_dose(
            |t| endocrine::pk_im_depot(tc::DOSE_WEEKLY_MG, tc::F_IM, vd, ka, ke, t),
            tc::INTERVAL_WEEKLY,
            8,
            &times,
        );
        let (cmax, _) = pkpd::find_cmax_tmax(&times, &concs);
        cmax_arr.push(cmax);
        auc_arr.push(pkpd::auc_trapezoidal(&times, &concs));
    }

    let n_f = crate::validation::len_f64(n_patients);
    let cmax_mean = cmax_arr.iter().sum::<f64>() / n_f;
    let auc_mean = auc_arr.iter().sum::<f64>() / n_f;
    serde_json::json!({
        "n_patients": n_patients,
        "cmax_mean": cmax_mean,
        "auc_mean": auc_mean,
        "cmax_p5": percentile_sorted(&mut cmax_arr.clone(), 5.0),
        "cmax_p95": percentile_sorted(&mut cmax_arr.clone(), 95.0),
    })
}

// ── Diagnostic ────────────────────────────────────────────────────────────

pub fn dispatch_assess_patient(params: &Value) -> Value {
    let age = f(params, "age_years").unwrap_or(45.0);
    let weight = f(params, "weight_kg").unwrap_or(85.0);
    let sex = match params.get("sex").and_then(Value::as_str).unwrap_or("male") {
        "female" | "Female" | "F" => diagnostic::Sex::Female,
        _ => diagnostic::Sex::Male,
    };

    let mut profile = diagnostic::PatientProfile::minimal(age, weight, sex);
    profile.testosterone_ng_dl = f(params, "testosterone_ng_dl");
    profile.on_trt = params
        .get("on_trt")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    profile.trt_months = f(params, "trt_months").unwrap_or(0.0);
    profile.gut_abundances = fa(params, "gut_abundances");
    profile.ppg_spo2 = f(params, "ppg_spo2");
    profile.ecg_fs = f(params, "ecg_fs").unwrap_or(360.0);
    profile.ecg_peaks = sza(params, "ecg_peaks");
    profile.scr_count = super::sz(params, "scr_count");
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

pub fn dispatch_composite_risk(params: &Value) -> Value {
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

pub fn dispatch_population_montecarlo(params: &Value) -> Value {
    let profile = build_profile_from_params(params);
    let n = sz_or(params, "n_patients", 100);
    let seed = params.get("seed").and_then(Value::as_u64).unwrap_or(42);
    let result = diagnostic::population_montecarlo(&profile, n, seed);
    serde_json::json!({
        "n_patients": result.n_patients,
        "mean_risk": result.mean_risk,
        "std_risk": result.std_risk,
        "patient_percentile": result.patient_percentile,
        "composite_risks": result.composite_risks,
    })
}

// ── Clinical ─────────────────────────────────────────────────────────────

pub fn dispatch_trt_scenario(params: &Value) -> Value {
    let profile = build_profile_from_params(params);
    let month = f(params, "month").unwrap_or(12.0);
    let testosterone = f(params, "testosterone_ng_dl")
        .unwrap_or_else(|| profile.testosterone_ng_dl.unwrap_or(600.0));

    let dw = endocrine::weight_trajectory(month, -16.0, 6.0, 60.0);
    let hr = endocrine::hazard_ratio_model(testosterone, 300.0, 0.44);
    let hba1c = endocrine::hba1c_trajectory(month, 7.60, -0.37, 3.0);
    let base_sdnn = f(params, "base_sdnn").unwrap_or(40.0);
    let sdnn = endocrine::hrv_trt_response(base_sdnn, 20.0, 6.0, month);
    let cardiac = endocrine::cardiac_risk_composite(sdnn, testosterone, 0.1);

    serde_json::json!({
        "month": month,
        "weight_change_kg": dw,
        "hazard_ratio": hr,
        "hba1c": hba1c,
        "sdnn_ms": sdnn,
        "cardiac_risk": cardiac,
    })
}

pub fn dispatch_patient_parameterize(params: &Value) -> Value {
    let profile = build_profile_from_params(params);
    serde_json::json!({
        "age_years": profile.age_years,
        "weight_kg": profile.weight_kg,
        "sex": match profile.sex {
            diagnostic::Sex::Male => "male",
            diagnostic::Sex::Female => "female",
        },
        "testosterone_ng_dl": profile.testosterone_ng_dl,
        "on_trt": profile.on_trt,
        "trt_months": profile.trt_months,
        "ecg_fs": profile.ecg_fs,
        "eda_duration_s": profile.eda_duration_s,
    })
}

pub fn dispatch_risk_annotate(params: &Value) -> Value {
    let profile = build_profile_from_params(params);
    let assessment = diagnostic::assess_patient(&profile);
    let risk_level = match assessment.composite_risk {
        r if r < 0.25 => "low",
        r if r < 0.5 => "moderate",
        r if r < 0.75 => "high",
        _ => "critical",
    };
    serde_json::json!({
        "composite_risk": assessment.composite_risk,
        "risk_level": risk_level,
        "annotations": {
            "microbiome": format!("{:?}", assessment.microbiome.risk_level),
            "cardiac": assessment.endocrine.cardiac_risk,
            "biosignal_stress": assessment.biosignal.stress_index,
        },
    })
}
