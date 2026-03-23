// SPDX-License-Identifier: AGPL-3.0-or-later
//! PK/PD and NLME capability handlers.

use serde_json::Value;

use crate::pkpd;
use crate::tolerances;

use super::{f, fa, missing, sz_or};

const NLME_DEFAULT_THETA: [f64; 3] = [2.3, 4.4, 0.4];
const NLME_DEFAULT_OMEGA: [f64; 3] = [0.04, 0.04, 0.09];
const NLME_DEFAULT_SEED: u64 = 42;

fn parse_nlme_subjects(params: &Value) -> Option<Vec<pkpd::Subject>> {
    let arr = params.get("subjects")?.as_array()?;
    let mut subjects = Vec::with_capacity(arr.len());
    for subj_v in arr {
        let dose = subj_v.get("dose").and_then(Value::as_f64)?;
        let times = fa(subj_v, "times")?;
        let observations = fa(subj_v, "observations")?;
        if times.len() != observations.len() || times.is_empty() {
            return None;
        }
        subjects.push(pkpd::Subject {
            times,
            observations,
            dose,
        });
    }
    if subjects.is_empty() {
        return None;
    }
    Some(subjects)
}

pub fn dispatch_hill(params: &Value) -> Value {
    let (Some(concentration), Some(ic50), Some(hill_n), Some(e_max)) = (
        f(params, "concentration"),
        f(params, "ic50"),
        f(params, "hill_n"),
        f(params, "e_max"),
    ) else {
        return missing("concentration, ic50, hill_n, e_max");
    };
    let response = pkpd::hill_dose_response(concentration, ic50, hill_n, e_max);
    let ec = pkpd::compute_ec_values(ic50, hill_n);
    serde_json::json!({
        "response": response,
        "ec10": ec.ec10, "ec50": ec.ec50, "ec90": ec.ec90,
    })
}

pub fn dispatch_one_compartment(params: &Value) -> Value {
    let route = params.get("route").and_then(Value::as_str).unwrap_or("iv");
    if route == "oral" {
        let (Some(dose), Some(f_bio), Some(vd), Some(ka), Some(ke), Some(t)) = (
            f(params, "dose"),
            f(params, "f"),
            f(params, "vd"),
            f(params, "ka"),
            f(params, "ke"),
            f(params, "t"),
        ) else {
            return missing("dose, f, vd, ka, ke, t");
        };
        let c = pkpd::pk_oral_one_compartment(dose, f_bio, vd, ka, ke, t);
        serde_json::json!({"concentration": c, "route": "oral"})
    } else {
        let (Some(dose), Some(vd), Some(half_life), Some(t)) = (
            f(params, "dose_mg"),
            f(params, "vd"),
            f(params, "half_life_hr"),
            f(params, "t"),
        ) else {
            return missing("dose_mg, vd, half_life_hr, t");
        };
        let c = pkpd::pk_iv_bolus(dose, vd, half_life, t);
        serde_json::json!({"concentration": c, "route": "iv"})
    }
}

pub fn dispatch_two_compartment(params: &Value) -> Value {
    let (Some(c0), Some(alpha), Some(beta), Some(k21), Some(t)) = (
        f(params, "c0"),
        f(params, "alpha"),
        f(params, "beta"),
        f(params, "k21"),
        f(params, "t"),
    ) else {
        return missing("c0, alpha, beta, k21, t");
    };
    let (a, b) = pkpd::two_compartment_ab(c0, alpha, beta, k21);
    let c = a.mul_add((-alpha * t).exp(), b * (-beta * t).exp());
    serde_json::json!({"concentration": c, "A": a, "B": b})
}

pub fn dispatch_pbpk(params: &Value) -> Value {
    let (Some(dose), Some(duration)) = (f(params, "dose_mg"), f(params, "duration_hr")) else {
        return missing("dose_mg, duration_hr");
    };
    let dt = f(params, "dt").unwrap_or(0.01);
    let blood_volume = f(params, "blood_volume_l").unwrap_or(5.0);
    let tissues = pkpd::standard_human_tissues();
    let (times, venous, _state) =
        pkpd::pbpk_iv_simulate(&tissues, dose, blood_volume, duration, dt);
    let auc = pkpd::pbpk_auc(&times, &venous);
    serde_json::json!({
        "n_steps": times.len(),
        "auc": auc,
        "peak_plasma": venous.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    })
}

pub fn dispatch_population_pk(params: &Value) -> Value {
    let n = sz_or(params, "n", 100);
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
    #[expect(clippy::cast_precision_loss, reason = "population size fits f64")]
    let n_res = results.len().max(1) as f64;
    serde_json::json!({
        "n": results.len(),
        "auc_mean": results.iter().map(|r| r.auc).sum::<f64>() / n_res,
        "cmax_mean": results.iter().map(|r| r.cmax).sum::<f64>() / n_res,
    })
}

pub fn dispatch_allometric(params: &Value) -> Value {
    let (Some(param_animal), Some(bw_animal), Some(bw_human)) = (
        f(params, "param_animal"),
        f(params, "bw_animal"),
        f(params, "bw_human"),
    ) else {
        return missing("param_animal, bw_animal, bw_human");
    };
    let exponent = f(params, "exponent").unwrap_or(0.75);
    let scaled = pkpd::allometric_scale(param_animal, bw_animal, bw_human, exponent);
    serde_json::json!({"scaled_param": scaled, "exponent": exponent})
}

pub fn dispatch_auc(params: &Value) -> Value {
    let (Some(times), Some(concs)) = (fa(params, "times"), fa(params, "concentrations")) else {
        return missing("times, concentrations");
    };
    let auc = pkpd::auc_trapezoidal(&times, &concs);
    serde_json::json!({"auc": auc})
}

pub fn dispatch_nca(params: &Value) -> Value {
    let (Some(times), Some(concs)) = (fa(params, "times"), fa(params, "concentrations")) else {
        return missing("times, concentrations");
    };
    let dose = f(params, "dose").unwrap_or(100.0);
    let min_pts = sz_or(params, "min_terminal_points", 3);
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

pub fn dispatch_mm(params: &Value) -> Value {
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

pub fn dispatch_nlme_foce(params: &Value) -> Value {
    let Some(subjects) = parse_nlme_subjects(params) else {
        return missing("subjects (array of {dose, times, observations})");
    };
    let theta = fa(params, "theta").unwrap_or_else(|| NLME_DEFAULT_THETA.to_vec());
    let omega = fa(params, "omega").unwrap_or_else(|| NLME_DEFAULT_OMEGA.to_vec());
    let sigma = f(params, "sigma").unwrap_or(tolerances::VPC_DEFAULT_SIGMA);
    let config = pkpd::NlmeConfig {
        n_theta: theta.len(),
        n_eta: omega.len(),
        max_iter: sz_or(params, "max_iter", 200),
        tol: f(params, "tol").unwrap_or(tolerances::NLME_DEFAULT_TOL),
        seed: params
            .get("seed")
            .and_then(Value::as_u64)
            .unwrap_or(NLME_DEFAULT_SEED),
    };
    let result = pkpd::foce(
        pkpd::oral_one_compartment_model,
        &subjects,
        &theta,
        &omega,
        sigma,
        &config,
    );
    serde_json::json!({
        "theta": result.theta,
        "omega_diag": result.omega_diag,
        "sigma": result.sigma,
        "objective": result.objective,
        "converged": result.converged,
        "iterations": result.iterations,
    })
}

pub fn dispatch_nlme_saem(params: &Value) -> Value {
    let Some(subjects) = parse_nlme_subjects(params) else {
        return missing("subjects (array of {dose, times, observations})");
    };
    let theta = fa(params, "theta").unwrap_or_else(|| NLME_DEFAULT_THETA.to_vec());
    let omega = fa(params, "omega").unwrap_or_else(|| NLME_DEFAULT_OMEGA.to_vec());
    let sigma = f(params, "sigma").unwrap_or(tolerances::VPC_DEFAULT_SIGMA);
    let config = pkpd::NlmeConfig {
        n_theta: theta.len(),
        n_eta: omega.len(),
        max_iter: sz_or(params, "max_iter", 300),
        tol: f(params, "tol").unwrap_or(tolerances::NLME_DEFAULT_TOL),
        seed: params
            .get("seed")
            .and_then(Value::as_u64)
            .unwrap_or(NLME_DEFAULT_SEED),
    };
    let result = pkpd::saem(
        pkpd::oral_one_compartment_model,
        &subjects,
        &theta,
        &omega,
        sigma,
        &config,
    );
    serde_json::json!({
        "theta": result.theta,
        "omega_diag": result.omega_diag,
        "sigma": result.sigma,
        "objective": result.objective,
        "converged": result.converged,
        "iterations": result.iterations,
    })
}

pub fn dispatch_cwres(params: &Value) -> Value {
    let Some(subjects) = parse_nlme_subjects(params) else {
        return missing("subjects (array of {dose, times, observations})");
    };
    let theta = fa(params, "theta").unwrap_or_else(|| NLME_DEFAULT_THETA.to_vec());
    let omega = fa(params, "omega").unwrap_or_else(|| NLME_DEFAULT_OMEGA.to_vec());
    let sigma = f(params, "sigma").unwrap_or(tolerances::VPC_DEFAULT_SIGMA);
    let config = pkpd::NlmeConfig {
        n_theta: theta.len(),
        n_eta: omega.len(),
        max_iter: sz_or(params, "max_iter", 200),
        tol: f(params, "tol").unwrap_or(tolerances::NLME_DEFAULT_TOL),
        seed: params
            .get("seed")
            .and_then(Value::as_u64)
            .unwrap_or(NLME_DEFAULT_SEED),
    };
    let result = pkpd::foce(
        pkpd::oral_one_compartment_model,
        &subjects,
        &theta,
        &omega,
        sigma,
        &config,
    );
    let subject_cwres = pkpd::compute_cwres(pkpd::oral_one_compartment_model, &subjects, &result);
    let summary = pkpd::cwres_summary(&subject_cwres);
    serde_json::json!({
        "mean": summary.mean,
        "std_dev": summary.std_dev,
        "n_residuals": summary.all_residuals.len(),
    })
}

pub fn dispatch_vpc(params: &Value) -> Value {
    let Some(subjects) = parse_nlme_subjects(params) else {
        return missing("subjects (array of {dose, times, observations})");
    };
    let theta = fa(params, "theta").unwrap_or_else(|| NLME_DEFAULT_THETA.to_vec());
    let omega = fa(params, "omega").unwrap_or_else(|| NLME_DEFAULT_OMEGA.to_vec());
    let sigma = f(params, "sigma").unwrap_or(tolerances::VPC_DEFAULT_SIGMA);
    let config = pkpd::NlmeConfig {
        n_theta: theta.len(),
        n_eta: omega.len(),
        max_iter: sz_or(params, "max_iter", 200),
        tol: f(params, "tol").unwrap_or(tolerances::NLME_DEFAULT_TOL),
        seed: params
            .get("seed")
            .and_then(Value::as_u64)
            .unwrap_or(NLME_DEFAULT_SEED),
    };
    let result = pkpd::foce(
        pkpd::oral_one_compartment_model,
        &subjects,
        &theta,
        &omega,
        sigma,
        &config,
    );
    let vpc_config = pkpd::VpcConfig {
        n_simulations: sz_or(params, "n_simulations", 200),
        n_bins: sz_or(params, "n_bins", 10),
        seed: params.get("seed").and_then(Value::as_u64).unwrap_or(42),
    };
    let vpc = pkpd::compute_vpc(
        pkpd::oral_one_compartment_model,
        &subjects,
        &result,
        &vpc_config,
    );
    serde_json::json!({
        "time_bins": vpc.time_bins,
        "obs_p5": vpc.obs_p5,
        "obs_p50": vpc.obs_p50,
        "obs_p95": vpc.obs_p95,
        "sim_p5": vpc.sim_p5,
        "sim_p50": vpc.sim_p50,
        "sim_p95": vpc.sim_p95,
    })
}

pub fn dispatch_gof(params: &Value) -> Value {
    let Some(subjects) = parse_nlme_subjects(params) else {
        return missing("subjects (array of {dose, times, observations})");
    };
    let theta = fa(params, "theta").unwrap_or_else(|| NLME_DEFAULT_THETA.to_vec());
    let omega = fa(params, "omega").unwrap_or_else(|| NLME_DEFAULT_OMEGA.to_vec());
    let sigma = f(params, "sigma").unwrap_or(tolerances::VPC_DEFAULT_SIGMA);
    let config = pkpd::NlmeConfig {
        n_theta: theta.len(),
        n_eta: omega.len(),
        max_iter: sz_or(params, "max_iter", 200),
        tol: f(params, "tol").unwrap_or(tolerances::NLME_DEFAULT_TOL),
        seed: params
            .get("seed")
            .and_then(Value::as_u64)
            .unwrap_or(NLME_DEFAULT_SEED),
    };
    let result = pkpd::foce(
        pkpd::oral_one_compartment_model,
        &subjects,
        &theta,
        &omega,
        sigma,
        &config,
    );
    let gof = pkpd::compute_gof(pkpd::oral_one_compartment_model, &subjects, &result);
    serde_json::json!({
        "r_squared_individual": gof.r_squared_individual,
        "r_squared_population": gof.r_squared_population,
        "n_observations": gof.observed.len(),
    })
}
