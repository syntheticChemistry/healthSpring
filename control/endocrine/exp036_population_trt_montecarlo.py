# SPDX-License-Identifier: AGPL-3.0-or-later

#!/usr/bin/env python3
"""
healthSpring Exp036 — Population TRT Monte Carlo (10K Virtual Patients)

Extends Exp005 (Population PK Monte Carlo) to testosterone replacement therapy.
Simulates 10,000 virtual patients with inter-individual variability (IIV) on
PK parameters and TRT outcome biomarkers, then computes population-level metrics.

Model:
  - IM depot PK (Bateman equation) with lognormal IIV on Vd, k_a, k_e
  - Steady-state weekly 100mg cypionate dosing (8 doses)
  - Age-related T0 baseline with population variability
  - Metabolic response heterogeneity (weight loss, HbA1c)

Population parameters (from Exp030-035 + published registries):
  Vd:  typical 70 L, CV 25%
  k_a: typical 0.46 /day (t½_abs ≈ 1.5d), CV 30%
  k_e: typical 0.087 /day (t½ ≈ 8d), CV 20%
  T0:  typical 600 ng/dL, CV 25%
  Age: uniform [40, 75]

Reference: Saad 2013 (n=411), Traish 2014 (n=261+260), Composite registries
           Mok Ch.4/5/6

Provenance:
  Baseline date:   2026-03-08
  Command:         python3 control/endocrine/exp036_population_trt_montecarlo.py
  Environment:     Python 3.10+, NumPy, seed=42
"""

import json
import os
import sys

import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
SEED = 42
N_PATIENTS = 10000
N_DOSES = 8
DOSE_MG = 100.0
INTERVAL_DAYS = 7.0
F_IM = 1.0
T_MAX_DAYS = 56.0
N_TIMEPOINTS = 500

# Population PK parameters (testosterone cypionate IM)
POP_VD = {"typical": 70.0, "cv": 0.25}
POP_KA = {"typical": 0.462, "cv": 0.30}  # ln2/1.5
POP_KE = {"typical": 0.0866, "cv": 0.20}  # ln2/8

# Baseline testosterone
POP_T0 = {"typical": 600.0, "cv": 0.25}  # ng/dL at age 30
DECLINE_RATE = 0.017  # 1.7%/yr

# Metabolic response IIV
WEIGHT_LOSS_MAX = -16.0   # kg at 5yr for responders
WEIGHT_LOSS_CV = 0.50     # high heterogeneity in response


def sample_lognormal(rng, typical, cv, n):
    omega_sq = np.log(1 + cv ** 2)
    mu = np.log(typical) - omega_sq / 2
    return rng.lognormal(mu, np.sqrt(omega_sq), n)


def pk_im_depot(dose, f, vd, ka, ke, t):
    denom = ka - ke
    if abs(denom) < 1e-12:
        return 0.0
    coeff = (f * dose * ka) / (vd * denom)
    return coeff * (np.exp(-ke * t) - np.exp(-ka * t))


def pk_multiple_dose_single(dose, f, vd, ka, ke, interval, n_doses, t_arr):
    total = np.zeros_like(t_arr)
    for d in range(n_doses):
        t_shifted = t_arr - d * interval
        mask = t_shifted >= 0
        contrib = np.zeros_like(t_arr)
        safe_t = np.where(mask, t_shifted, 0.0)
        denom = ka - ke
        if abs(denom) > 1e-12:
            coeff = (f * dose * ka) / (vd * denom)
            contrib = np.where(mask, coeff * (np.exp(-ke * safe_t) - np.exp(-ka * safe_t)), 0.0)
        total += contrib
    return total


def main():
    total_passed = 0
    total_failed = 0
    baseline = {}
    rng = np.random.default_rng(SEED)

    print("=" * 72)
    print(f"healthSpring Exp036: Population TRT Monte Carlo ({N_PATIENTS} patients)")
    print("=" * 72)

    t = np.linspace(0, T_MAX_DAYS, N_TIMEPOINTS)

    # Sample population parameters
    vd = sample_lognormal(rng, POP_VD["typical"], POP_VD["cv"], N_PATIENTS)
    ka = sample_lognormal(rng, POP_KA["typical"], POP_KA["cv"], N_PATIENTS)
    ke = sample_lognormal(rng, POP_KE["typical"], POP_KE["cv"], N_PATIENTS)
    t0 = sample_lognormal(rng, POP_T0["typical"], POP_T0["cv"], N_PATIENTS)
    ages = rng.uniform(40, 75, N_PATIENTS)

    # Age-adjusted baseline T
    t0_adj = t0 * np.exp(-DECLINE_RATE * (ages - 30))

    baseline["vd_mean"] = float(np.mean(vd))
    baseline["ka_mean"] = float(np.mean(ka))
    baseline["ke_mean"] = float(np.mean(ke))
    baseline["t0_adj_mean"] = float(np.mean(t0_adj))
    baseline["age_mean"] = float(np.mean(ages))

    # Compute PK curves (steady state from 8 weekly doses)
    cmax_arr = np.zeros(N_PATIENTS)
    tmax_arr = np.zeros(N_PATIENTS)
    auc_arr = np.zeros(N_PATIENTS)
    trough_arr = np.zeros(N_PATIENTS)

    for i in range(N_PATIENTS):
        c_i = pk_multiple_dose_single(DOSE_MG, F_IM, vd[i], ka[i], ke[i],
                                       INTERVAL_DAYS, N_DOSES, t)
        cmax_arr[i] = np.max(c_i)
        tmax_arr[i] = t[np.argmax(c_i)]
        auc_arr[i] = float(np.trapezoid(c_i, t))
        # Trough: min in last dosing interval
        last_start = (N_DOSES - 1) * INTERVAL_DAYS
        mask_last = t >= last_start
        if np.any(mask_last):
            trough_arr[i] = np.min(c_i[mask_last])

    baseline["cmax_median"] = float(np.median(cmax_arr))
    baseline["cmax_cv"] = float(np.std(cmax_arr) / np.mean(cmax_arr))
    baseline["auc_median"] = float(np.median(auc_arr))
    baseline["auc_cv"] = float(np.std(auc_arr) / np.mean(auc_arr))
    baseline["trough_median"] = float(np.median(trough_arr))

    # Metabolic response (weight loss at 12 months, heterogeneous)
    weight_response = sample_lognormal(rng, abs(WEIGHT_LOSS_MAX), WEIGHT_LOSS_CV, N_PATIENTS)
    weight_loss = -weight_response  # negative = loss
    # Responders have higher trough levels (correlation)
    trough_norm = (trough_arr - np.mean(trough_arr)) / (np.std(trough_arr) + 1e-12)
    weight_loss_adj = weight_loss * (1.0 + 0.2 * trough_norm)  # modest PK-response correlation

    # Hypogonadism classification
    frac_hypo = float(np.mean(t0_adj < 300))

    # --- Check 1: All PK parameters positive ---
    print("\n--- Check 1: All PK parameters positive ---")
    if np.all(vd > 0) and np.all(ka > 0) and np.all(ke > 0):
        print(f"  [PASS] Vd, k_a, k_e all > 0 for {N_PATIENTS} patients")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # --- Check 2: Vd mean near typical ---
    print("\n--- Check 2: Vd mean near typical ---")
    vd_rel = abs(np.mean(vd) - POP_VD["typical"]) / POP_VD["typical"]
    if vd_rel < 0.05:
        print(f"  [PASS] Vd mean={np.mean(vd):.2f}, typical={POP_VD['typical']}, err={vd_rel:.3%}")
        total_passed += 1
    else:
        print(f"  [FAIL] err={vd_rel:.3%}")
        total_failed += 1

    # --- Check 3: All Cmax > 0 ---
    print("\n--- Check 3: All Cmax > 0 ---")
    if np.all(cmax_arr > 0):
        print(f"  [PASS] min(Cmax)={np.min(cmax_arr):.6f}")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # --- Check 4: All AUC > 0 ---
    print("\n--- Check 4: All AUC > 0 ---")
    if np.all(auc_arr > 0):
        print(f"  [PASS] min(AUC)={np.min(auc_arr):.4f}")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # --- Check 5: AUC has meaningful variability ---
    print("\n--- Check 5: AUC variability (CV > 0.15) ---")
    if baseline["auc_cv"] > 0.15:
        print(f"  [PASS] AUC CV = {baseline['auc_cv']:.3f}")
        total_passed += 1
    else:
        print(f"  [FAIL] CV = {baseline['auc_cv']:.3f}")
        total_failed += 1

    # --- Check 6: Cmax variability ---
    print("\n--- Check 6: Cmax variability (CV > 0.15) ---")
    if baseline["cmax_cv"] > 0.15:
        print(f"  [PASS] Cmax CV = {baseline['cmax_cv']:.3f}")
        total_passed += 1
    else:
        print(f"  [FAIL] CV = {baseline['cmax_cv']:.3f}")
        total_failed += 1

    # --- Check 7: AUC percentiles ordered ---
    print("\n--- Check 7: AUC percentiles ordered ---")
    p5, p50, p95 = np.percentile(auc_arr, [5, 50, 95])
    baseline["auc_p5"] = float(p5)
    baseline["auc_p50"] = float(p50)
    baseline["auc_p95"] = float(p95)
    if p5 < p50 < p95:
        print(f"  [PASS] P5={p5:.2f} < P50={p50:.2f} < P95={p95:.2f}")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # --- Check 8: Higher ke → lower AUC (negative correlation) ---
    print("\n--- Check 8: ke-AUC negative correlation ---")
    corr = np.corrcoef(ke, auc_arr)[0, 1]
    baseline["ke_auc_corr"] = float(corr)
    if corr < -0.3:
        print(f"  [PASS] r(ke, AUC) = {corr:.4f}")
        total_passed += 1
    else:
        print(f"  [FAIL] r = {corr:.4f}")
        total_failed += 1

    # --- Check 9: Fraction hypogonadal plausible ---
    print("\n--- Check 9: Fraction hypogonadal (T0 < 300) ---")
    baseline["frac_hypogonadal"] = frac_hypo
    if 0.10 < frac_hypo < 0.70:
        print(f"  [PASS] {frac_hypo:.1%} hypogonadal (plausible for age 40-75)")
        total_passed += 1
    else:
        print(f"  [FAIL] {frac_hypo:.1%}")
        total_failed += 1

    # --- Check 10: Older patients have lower baseline T ---
    print("\n--- Check 10: Age-T0 negative correlation ---")
    corr_age_t = np.corrcoef(ages, t0_adj)[0, 1]
    baseline["age_t0_corr"] = float(corr_age_t)
    if corr_age_t < -0.3:
        print(f"  [PASS] r(age, T0) = {corr_age_t:.4f}")
        total_passed += 1
    else:
        print(f"  [FAIL] r = {corr_age_t:.4f}")
        total_failed += 1

    # --- Check 11: Cohort size correct ---
    print("\n--- Check 11: Cohort size = 10K ---")
    if len(cmax_arr) == N_PATIENTS:
        print(f"  [PASS] {N_PATIENTS} patients simulated")
        total_passed += 1
    else:
        print(f"  [FAIL] {len(cmax_arr)} patients")
        total_failed += 1

    # --- Check 12: Weight loss distribution plausible ---
    print("\n--- Check 12: Weight loss distribution ---")
    mean_wl = float(np.mean(weight_loss_adj))
    baseline["weight_loss_mean"] = mean_wl
    if -25.0 < mean_wl < -5.0:
        print(f"  [PASS] Mean weight change = {mean_wl:.2f} kg (plausible)")
        total_passed += 1
    else:
        print(f"  [FAIL] Mean = {mean_wl:.2f} kg")
        total_failed += 1

    # --- Write baseline ---
    baseline_path = os.path.join(SCRIPT_DIR, "exp036_baseline.json")
    baseline_out = {
        "_source": "healthSpring Exp036: Population TRT Monte Carlo",
        "_method": "10K virtual patients, IM depot PK + metabolic response",
        "n_patients": N_PATIENTS,
        "seed": SEED,
        "pop_params": {
            "vd": POP_VD, "ka": POP_KA, "ke": POP_KE,
            "t0": POP_T0, "decline_rate": DECLINE_RATE,
        },
        **baseline,
        "_provenance": {
            "date": "2026-03-08",
            "python": sys.version,
            "numpy": np.__version__,
        },
    }
    with open(baseline_path, "w") as f:
        json.dump(baseline_out, f, indent=2, default=str)
    print(f"\nBaseline written to {baseline_path}")

    total = total_passed + total_failed
    print(f"\n{'=' * 72}")
    print(f"TOTAL: {total_passed}/{total} PASS, {total_failed}/{total} FAIL")
    print(f"{'=' * 72}")

    return 0 if total_failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
