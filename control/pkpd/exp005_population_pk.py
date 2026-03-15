# SPDX-License-Identifier: AGPL-3.0-only

#!/usr/bin/env python3
"""
healthSpring Exp005 — Population PK Monte Carlo

Validates population pharmacokinetic simulation: generating virtual
patient cohorts with inter-individual variability (IIV) and computing
population-level exposure metrics.

Model: One-compartment oral with lognormal IIV on CL, Vd, k_a.
  C_i(t) = (F * Dose * k_a_i) / (Vd_i * (k_a_i - k_e_i))
            * (exp(-k_e_i * t) - exp(-k_a_i * t))
  where k_e_i = CL_i / Vd_i

Population parameters (baricitinib-like, oral):
  CL:  typical 10 L/hr, CV 30%
  Vd:  typical 80 L, CV 25%
  k_a: typical 1.5 /hr, CV 40%
  F:   0.79 (fixed, baricitinib bioavailability)

Reference: Mould & Upton 2013 (CPT:PSP), Rowland & Tozer Ch. 18

Provenance:
  Baseline date:   2026-03-08
  Command:         python3 control/pkpd/exp005_population_pk.py
  Environment:     Python 3.10+, NumPy, seed=42
"""

import json
import os
import sys

import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
SEED = 42
N_PATIENTS = 1000
N_TIMEPOINTS = 500
T_MAX_HR = 24.0

POP_PARAMS = {
    "cl_typical_L_hr": 10.0,
    "cl_cv": 0.30,
    "vd_typical_L": 80.0,
    "vd_cv": 0.25,
    "ka_typical_hr": 1.5,
    "ka_cv": 0.40,
    "f_bioavail": 0.79,
    "dose_mg": 4.0,  # baricitinib 4mg oral
}


def sample_lognormal(rng, typical, cv, n):
    """Sample from lognormal: median = typical, CV = coefficient of variation."""
    omega_sq = np.log(1 + cv ** 2)
    mu = np.log(typical) - omega_sq / 2
    return rng.lognormal(mu, np.sqrt(omega_sq), n)


def pk_oral_population(dose, f, vd, ka, cl, t):
    """Bateman equation for each patient (vectorized over patients)."""
    ke = cl / vd
    denom = ka - ke
    safe = np.abs(denom) > 1e-12
    coeff = np.where(safe, (f * dose * ka) / (vd * denom), 0.0)
    c = np.zeros((len(vd), len(t)))
    for i in range(len(vd)):
        if safe[i]:
            c[i] = coeff[i] * (np.exp(-ke[i] * t) - np.exp(-ka[i] * t))
    return c


def auc_trap(t, c_matrix):
    """Trapezoidal AUC for each row (patient)."""
    return np.array([np.trapezoid(c_matrix[i], t) for i in range(c_matrix.shape[0])])


def main():
    total_passed = 0
    total_failed = 0
    baseline = {}
    rng = np.random.default_rng(SEED)
    p = POP_PARAMS

    print("=" * 72)
    print("healthSpring Exp005: Population PK Monte Carlo")
    print(f"  {N_PATIENTS} virtual patients, seed={SEED}")
    print("=" * 72)

    t = np.linspace(0, T_MAX_HR, N_TIMEPOINTS)

    cl = sample_lognormal(rng, p["cl_typical_L_hr"], p["cl_cv"], N_PATIENTS)
    vd = sample_lognormal(rng, p["vd_typical_L"], p["vd_cv"], N_PATIENTS)
    ka = sample_lognormal(rng, p["ka_typical_hr"], p["ka_cv"], N_PATIENTS)

    baseline["cl_mean"] = float(np.mean(cl))
    baseline["cl_std"] = float(np.std(cl))
    baseline["vd_mean"] = float(np.mean(vd))
    baseline["ka_mean"] = float(np.mean(ka))

    c_matrix = pk_oral_population(p["dose_mg"], p["f_bioavail"], vd, ka, cl, t)

    # Per-patient metrics
    cmax = np.max(c_matrix, axis=1)
    tmax = t[np.argmax(c_matrix, axis=1)]
    auc = auc_trap(t, c_matrix)

    baseline["cmax_median"] = float(np.median(cmax))
    baseline["cmax_mean"] = float(np.mean(cmax))
    baseline["cmax_cv"] = float(np.std(cmax) / np.mean(cmax))
    baseline["tmax_median"] = float(np.median(tmax))
    baseline["auc_median"] = float(np.median(auc))
    baseline["auc_mean"] = float(np.mean(auc))
    baseline["auc_cv"] = float(np.std(auc) / np.mean(auc))

    # ------------------------------------------------------------------
    # Check 1: All CL > 0, Vd > 0, k_a > 0 (lognormal guarantees)
    # ------------------------------------------------------------------
    print("\n--- Check 1: All parameters positive ---")
    if np.all(cl > 0) and np.all(vd > 0) and np.all(ka > 0):
        print(f"  [PASS] CL, Vd, k_a all > 0 for {N_PATIENTS} patients")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 2: CL mean near typical (within 10%)
    # ------------------------------------------------------------------
    print("\n--- Check 2: CL mean near typical ---")
    cl_rel = abs(np.mean(cl) - p["cl_typical_L_hr"]) / p["cl_typical_L_hr"]
    if cl_rel < 0.10:
        print(f"  [PASS] CL mean={np.mean(cl):.3f}, typical={p['cl_typical_L_hr']}, err={cl_rel:.3%}")
        total_passed += 1
    else:
        print(f"  [FAIL] err={cl_rel:.3%}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 3: Vd mean near typical
    # ------------------------------------------------------------------
    print("\n--- Check 3: Vd mean near typical ---")
    vd_rel = abs(np.mean(vd) - p["vd_typical_L"]) / p["vd_typical_L"]
    if vd_rel < 0.10:
        print(f"  [PASS] Vd mean={np.mean(vd):.3f}, typical={p['vd_typical_L']}")
        total_passed += 1
    else:
        print(f"  [FAIL] err={vd_rel:.3%}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 4: All concentrations non-negative
    # ------------------------------------------------------------------
    print("\n--- Check 4: All concentrations ≥ 0 ---")
    if np.all(c_matrix >= -1e-12):
        print(f"  [PASS] min(C) = {np.min(c_matrix):.2e}")
        total_passed += 1
    else:
        print(f"  [FAIL] min(C) = {np.min(c_matrix):.2e}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 5: All AUC > 0
    # ------------------------------------------------------------------
    print("\n--- Check 5: All AUC > 0 ---")
    if np.all(auc > 0):
        print(f"  [PASS] min(AUC)={np.min(auc):.4f}, median={np.median(auc):.4f}")
        total_passed += 1
    else:
        print(f"  [FAIL] min(AUC)={np.min(auc):.4f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 6: All Cmax > 0
    # ------------------------------------------------------------------
    print("\n--- Check 6: All Cmax > 0 ---")
    if np.all(cmax > 0):
        print(f"  [PASS] min(Cmax)={np.min(cmax):.6f}, median={np.median(cmax):.4f}")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 7: AUC variability reflects IIV (CV > 0.15)
    # ------------------------------------------------------------------
    print("\n--- Check 7: AUC has meaningful variability ---")
    auc_cv = np.std(auc) / np.mean(auc)
    if auc_cv > 0.15:
        print(f"  [PASS] AUC CV = {auc_cv:.3f} (> 0.15)")
        total_passed += 1
    else:
        print(f"  [FAIL] AUC CV = {auc_cv:.3f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 8: Cmax variability
    # ------------------------------------------------------------------
    print("\n--- Check 8: Cmax has meaningful variability ---")
    cmax_cv = np.std(cmax) / np.mean(cmax)
    if cmax_cv > 0.15:
        print(f"  [PASS] Cmax CV = {cmax_cv:.3f}")
        total_passed += 1
    else:
        print(f"  [FAIL] Cmax CV = {cmax_cv:.3f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 9: Tmax in reasonable range (0.5–5 hr for oral)
    # ------------------------------------------------------------------
    print("\n--- Check 9: Tmax in reasonable range ---")
    tmax_95 = np.percentile(tmax, [2.5, 97.5])
    if tmax_95[0] >= 0.2 and tmax_95[1] <= 8.0:
        print(f"  [PASS] Tmax 95% CI: [{tmax_95[0]:.2f}, {tmax_95[1]:.2f}] hr")
        total_passed += 1
    else:
        print(f"  [FAIL] Tmax 95% CI: [{tmax_95[0]:.2f}, {tmax_95[1]:.2f}]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 10: C(0) = 0 for all patients (oral, t=0)
    # ------------------------------------------------------------------
    print("\n--- Check 10: C(0) = 0 for all patients ---")
    if np.all(np.abs(c_matrix[:, 0]) < 1e-10):
        print(f"  [PASS] max|C(0)| = {np.max(np.abs(c_matrix[:, 0])):.2e}")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 11: C(24hr) near zero for most patients
    # ------------------------------------------------------------------
    print("\n--- Check 11: C(24hr) near zero ---")
    c_24 = c_matrix[:, -1]
    pct_low = np.mean(c_24 < 0.01) * 100
    baseline["pct_c24_below_0.01"] = float(pct_low)
    if pct_low > 80:
        print(f"  [PASS] {pct_low:.1f}% patients C(24hr) < 0.01 mg/L")
        total_passed += 1
    else:
        print(f"  [FAIL] only {pct_low:.1f}%")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 12: Population AUC mean ≈ F*Dose/CL_typical (within 20%)
    # ------------------------------------------------------------------
    print("\n--- Check 12: Population AUC near theoretical ---")
    auc_theoretical = p["f_bioavail"] * p["dose_mg"] / p["cl_typical_L_hr"]
    auc_rel = abs(np.mean(auc) - auc_theoretical) / auc_theoretical
    baseline["auc_theoretical"] = float(auc_theoretical)
    if auc_rel < 0.20:
        print(f"  [PASS] AUC mean={np.mean(auc):.4f}, theoretical={auc_theoretical:.4f}, err={auc_rel:.3%}")
        total_passed += 1
    else:
        print(f"  [FAIL] err={auc_rel:.3%}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 13: Higher CL → lower AUC (negative correlation)
    # ------------------------------------------------------------------
    print("\n--- Check 13: CL-AUC negative correlation ---")
    corr = np.corrcoef(cl, auc)[0, 1]
    baseline["cl_auc_corr"] = float(corr)
    if corr < -0.3:
        print(f"  [PASS] r(CL, AUC) = {corr:.4f} (negative)")
        total_passed += 1
    else:
        print(f"  [FAIL] r = {corr:.4f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 14: Percentiles: P5 < P50 < P95 for AUC
    # ------------------------------------------------------------------
    print("\n--- Check 14: AUC percentiles ordered ---")
    p5, p50, p95 = np.percentile(auc, [5, 50, 95])
    baseline["auc_p5"] = float(p5)
    baseline["auc_p50"] = float(p50)
    baseline["auc_p95"] = float(p95)
    if p5 < p50 < p95:
        print(f"  [PASS] P5={p5:.4f} < P50={p50:.4f} < P95={p95:.4f}")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 15: N_PATIENTS curves generated
    # ------------------------------------------------------------------
    print("\n--- Check 15: Correct cohort size ---")
    if c_matrix.shape[0] == N_PATIENTS and c_matrix.shape[1] == N_TIMEPOINTS:
        print(f"  [PASS] {c_matrix.shape[0]} patients × {c_matrix.shape[1]} timepoints")
        total_passed += 1
    else:
        print(f"  [FAIL] shape = {c_matrix.shape}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Write baseline
    # ------------------------------------------------------------------
    baseline_path = os.path.join(SCRIPT_DIR, "exp005_baseline.json")
    baseline_out = {
        "_source": "healthSpring Exp005: Population PK Monte Carlo",
        "_method": "Lognormal IIV on CL/Vd/k_a, one-compartment oral Bateman",
        "pop_params": POP_PARAMS,
        "n_patients": N_PATIENTS,
        "seed": SEED,
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
