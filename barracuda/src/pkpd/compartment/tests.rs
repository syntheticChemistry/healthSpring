// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::pkpd::{auc_trapezoidal, find_cmax_tmax, pk_multiple_dose};
use core::f64::consts::LN_2;

const TOL: f64 = 1e-10;

const DOSE_IV: f64 = 500.0;
const VD_IV: f64 = 50.0;
const HL_IV: f64 = 6.0;

const DOSE_ORAL: f64 = 250.0;
const F_ORAL: f64 = 0.8;
const VD_ORAL: f64 = 35.0;
const HL_ORAL: f64 = 4.0;
const KA_ORAL: f64 = 1.5;

const V1_2C: f64 = 15.0;
const K10_2C: f64 = 0.35;
const K12_2C: f64 = 0.6;
const K21_2C: f64 = 0.15;
const DOSE_2C: f64 = 240.0;

#[test]
fn iv_c0_equals_dose_over_vd() {
    let c0 = pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, 0.0);
    assert!((c0 - DOSE_IV / VD_IV).abs() < TOL);
}

#[test]
fn iv_at_half_life_is_half() {
    let c = pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, HL_IV);
    let expected = (DOSE_IV / VD_IV) / 2.0;
    assert!((c - expected).abs() < 1e-6);
}

#[test]
fn iv_monotonically_decreasing() {
    let times: Vec<f64> = (0..1000).map(|i| 48.0 * f64::from(i) / 999.0).collect();
    let concs: Vec<f64> = times
        .iter()
        .map(|&t| pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, t))
        .collect();
    for w in concs.windows(2) {
        assert!(w[0] >= w[1] - TOL, "IV not monotonically decreasing");
    }
}

#[test]
fn iv_auc_matches_analytical() {
    let times: Vec<f64> = (0..1000).map(|i| 48.0 * f64::from(i) / 999.0).collect();
    let concs: Vec<f64> = times
        .iter()
        .map(|&t| pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, t))
        .collect();
    let auc_num = auc_trapezoidal(&times, &concs);
    let k_e = LN_2 / HL_IV;
    let auc_ana = DOSE_IV / (VD_IV * k_e);
    let rel_err = (auc_num - auc_ana).abs() / auc_ana;
    assert!(rel_err < 0.01, "AUC rel err {rel_err}");
}

#[test]
fn oral_c0_is_zero() {
    let k_e = LN_2 / HL_ORAL;
    let c = pk_oral_one_compartment(DOSE_ORAL, F_ORAL, VD_ORAL, KA_ORAL, k_e, 0.0);
    assert!(c.abs() < TOL);
}

#[test]
fn oral_cmax_at_positive_tmax() {
    let k_e = LN_2 / HL_ORAL;
    let times: Vec<f64> = (0..1000).map(|i| 48.0 * f64::from(i) / 999.0).collect();
    let concs: Vec<f64> = times
        .iter()
        .map(|&t| pk_oral_one_compartment(DOSE_ORAL, F_ORAL, VD_ORAL, KA_ORAL, k_e, t))
        .collect();
    let (cmax, tmax) = find_cmax_tmax(&times, &concs);
    assert!(cmax > 0.0, "Cmax > 0");
    assert!(tmax > 0.0, "Tmax > 0");
}

#[test]
fn oral_tmax_matches_analytical() {
    let k_e = LN_2 / HL_ORAL;
    let tmax_ana = oral_tmax(KA_ORAL, k_e);

    let times: Vec<f64> = (0..1000).map(|i| 48.0 * f64::from(i) / 999.0).collect();
    let concs: Vec<f64> = times
        .iter()
        .map(|&t| pk_oral_one_compartment(DOSE_ORAL, F_ORAL, VD_ORAL, KA_ORAL, k_e, t))
        .collect();
    let (_, tmax_num) = find_cmax_tmax(&times, &concs);
    assert!(
        (tmax_num - tmax_ana).abs() < 0.1,
        "Tmax numerical={tmax_num}, analytical={tmax_ana}"
    );
}

#[test]
fn oral_decays_by_48hr() {
    let k_e = LN_2 / HL_ORAL;
    let c = pk_oral_one_compartment(DOSE_ORAL, F_ORAL, VD_ORAL, KA_ORAL, k_e, 48.0);
    assert!(c < 0.01, "C(48hr) = {c}");
}

#[test]
fn oral_auc_matches_analytical() {
    let k_e = LN_2 / HL_ORAL;
    let times: Vec<f64> = (0..1000).map(|i| 48.0 * f64::from(i) / 999.0).collect();
    let concs: Vec<f64> = times
        .iter()
        .map(|&t| pk_oral_one_compartment(DOSE_ORAL, F_ORAL, VD_ORAL, KA_ORAL, k_e, t))
        .collect();
    let auc_num = auc_trapezoidal(&times, &concs);
    let auc_ana = (F_ORAL * DOSE_ORAL) / (VD_ORAL * k_e);
    let rel_err = (auc_num - auc_ana).abs() / auc_ana;
    assert!(rel_err < 0.01, "Oral AUC rel err {rel_err}");
}

#[test]
fn multiple_iv_doses_accumulate() {
    let c0 = DOSE_IV / VD_IV;
    let times: Vec<f64> = (0..1000).map(|i| 48.0 * f64::from(i) / 999.0).collect();
    let c_multi = pk_multiple_dose(|t| pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, t), 8.0, 6, &times);
    let peak_after_first = times
        .iter()
        .zip(c_multi.iter())
        .filter_map(|(t, c)| if *t >= 8.0 { Some(*c) } else { None })
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        peak_after_first > c0,
        "multi-dose peak {peak_after_first} > single C0 {c0}"
    );
}

#[test]
fn all_concentrations_nonneg() {
    let k_e = LN_2 / HL_ORAL;
    let times: Vec<f64> = (0..1000).map(|i| 48.0 * f64::from(i) / 999.0).collect();

    let c_iv: Vec<f64> = times
        .iter()
        .map(|&t| pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, t))
        .collect();
    let c_oral: Vec<f64> = times
        .iter()
        .map(|&t| pk_oral_one_compartment(DOSE_ORAL, F_ORAL, VD_ORAL, KA_ORAL, k_e, t))
        .collect();
    let c_multi = pk_multiple_dose(|t| pk_iv_bolus(DOSE_IV, VD_IV, HL_IV, t), 8.0, 6, &times);

    assert!(c_iv.iter().all(|&c| c >= 0.0), "IV non-negative");
    assert!(c_oral.iter().all(|&c| c >= 0.0), "Oral non-negative");
    assert!(c_multi.iter().all(|&c| c >= -1e-12), "Multi non-negative");
}

#[test]
fn two_comp_alpha_gt_beta() {
    let (alpha, beta) = micro_to_macro(K10_2C, K12_2C, K21_2C);
    assert!(alpha > beta, "α={alpha} should > β={beta}");
}

#[test]
fn two_comp_sum_identity() {
    let (alpha, beta) = micro_to_macro(K10_2C, K12_2C, K21_2C);
    let sum_macro = alpha + beta;
    let sum_micro = K10_2C + K12_2C + K21_2C;
    assert!((sum_macro - sum_micro).abs() < TOL, "α+β = k10+k12+k21");
}

#[test]
fn two_comp_product_identity() {
    let (alpha, beta) = micro_to_macro(K10_2C, K12_2C, K21_2C);
    let prod_macro = alpha * beta;
    let prod_micro = K10_2C * K21_2C;
    assert!((prod_macro - prod_micro).abs() < TOL, "α·β = k10·k21");
}

#[test]
fn two_comp_c0() {
    let c = pk_two_compartment_iv(DOSE_2C, V1_2C, K10_2C, K12_2C, K21_2C, 0.0);
    assert!((c - DOSE_2C / V1_2C).abs() < TOL, "C(0) = Dose/V1");
}

#[test]
fn two_comp_nonneg() {
    let times: Vec<f64> = (0..2000).map(|i| 168.0 * f64::from(i) / 1999.0).collect();
    let concs: Vec<f64> = times
        .iter()
        .map(|&t| pk_two_compartment_iv(DOSE_2C, V1_2C, K10_2C, K12_2C, K21_2C, t))
        .collect();
    assert!(concs.iter().all(|&c| c >= -1e-12), "all non-negative");
}

#[test]
fn two_comp_monotonic_dec() {
    let times: Vec<f64> = (0..2000).map(|i| 168.0 * f64::from(i) / 1999.0).collect();
    let concs: Vec<f64> = times
        .iter()
        .map(|&t| pk_two_compartment_iv(DOSE_2C, V1_2C, K10_2C, K12_2C, K21_2C, t))
        .collect();
    for w in concs.windows(2) {
        assert!(w[0] >= w[1] - 1e-12, "central monotonic decreasing");
    }
}

#[test]
fn two_comp_half_lives_ordered() {
    let (alpha, beta) = micro_to_macro(K10_2C, K12_2C, K21_2C);
    let t_half_alpha = LN_2 / alpha;
    let t_half_beta = LN_2 / beta;
    assert!(t_half_alpha < t_half_beta, "t½α < t½β");
}

#[test]
fn two_comp_auc_analytical() {
    let times: Vec<f64> = (0..2000).map(|i| 168.0 * f64::from(i) / 1999.0).collect();
    let concs: Vec<f64> = times
        .iter()
        .map(|&t| pk_two_compartment_iv(DOSE_2C, V1_2C, K10_2C, K12_2C, K21_2C, t))
        .collect();
    let auc_num = auc_trapezoidal(&times, &concs);
    let auc_ana = DOSE_2C / (V1_2C * K10_2C);
    let rel_err = (auc_num - auc_ana).abs() / auc_ana;
    assert!(rel_err < 0.01, "AUC rel err {rel_err}");
}

#[test]
fn two_comp_a_plus_b_eq_c0() {
    let c0 = DOSE_2C / V1_2C;
    let (alpha, beta) = micro_to_macro(K10_2C, K12_2C, K21_2C);
    let (a, b) = two_compartment_ab(c0, alpha, beta, K21_2C);
    assert!((a + b - c0).abs() < TOL, "A+B = C0");
}

#[test]
fn two_comp_reduces_to_one() {
    let times: Vec<f64> = (0..2000).map(|i| 168.0 * f64::from(i) / 1999.0).collect();
    let c_two: Vec<f64> = times
        .iter()
        .map(|&t| pk_two_compartment_iv(DOSE_2C, V1_2C, K10_2C, 0.0, K21_2C, t))
        .collect();
    let c_one: Vec<f64> = times
        .iter()
        .map(|&t| (DOSE_2C / V1_2C) * (-K10_2C * t).exp())
        .collect();
    let max_diff: f64 = c_two
        .iter()
        .zip(c_one.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f64::max);
    assert!(max_diff < 1e-10, "k12=0 → one-compartment, diff={max_diff}");
}
