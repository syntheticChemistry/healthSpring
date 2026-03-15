# SPDX-License-Identifier: AGPL-3.0-or-later

#!/usr/bin/env python3
"""
healthSpring Exp002 — One-Compartment PK Model

Validates single-compartment pharmacokinetic models:
  - IV bolus: C(t) = (Dose/Vd) * exp(-k_e * t)
  - Oral (first-order absorption): C(t) = (F*Dose*k_a)/(Vd*(k_a-k_e)) * (exp(-k_e*t) - exp(-k_a*t))
  - AUC computation (trapezoidal rule)
  - Cmax and Tmax for oral dosing
  - Multiple dosing (superposition)

Reference: Rowland & Tozer, Clinical Pharmacokinetics (Chapter 3-4)
Extension: neuralSpring nS-603 (lokivetmab exponential decay)

Provenance:
  Baseline date:   2026-03-08
  Command:         python3 control/pkpd/exp002_one_compartment_pk.py
  Environment:     Python 3.10+, NumPy
"""

import json
import os
import sys

import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

# Model drug parameters (generic, textbook-style)
DRUG_IV = {
    "name": "Drug-A (IV bolus)",
    "dose_mg": 500.0,
    "vd_L": 50.0,
    "half_life_hr": 6.0,
}

DRUG_ORAL = {
    "name": "Drug-B (oral)",
    "dose_mg": 250.0,
    "f_bioavail": 0.8,
    "vd_L": 35.0,
    "half_life_hr": 4.0,
    "k_a_hr": 1.5,  # absorption rate constant
}

TIME_HR = np.linspace(0, 48, 1000)


def pk_iv_bolus(dose_mg, vd_L, half_life_hr, t):
    """One-compartment IV bolus: C(t) = C0 * exp(-k_e * t)."""
    k_e = np.log(2) / half_life_hr
    c0 = dose_mg / vd_L
    return c0 * np.exp(-k_e * t)


def pk_oral_one_compartment(dose_mg, f, vd_L, k_a, k_e, t):
    """One-compartment oral absorption: Bateman equation."""
    if abs(k_a - k_e) < 1e-12:
        return np.zeros_like(t)
    coeff = (f * dose_mg * k_a) / (vd_L * (k_a - k_e))
    return coeff * (np.exp(-k_e * t) - np.exp(-k_a * t))


def auc_trapezoidal(t, c):
    """AUC by trapezoidal rule."""
    return float(np.trapezoid(c, t))


def find_cmax_tmax(t, c):
    """Find Cmax and Tmax from concentration-time curve."""
    idx = np.argmax(c)
    return float(c[idx]), float(t[idx])


def pk_multiple_dose(single_dose_func, dose_interval_hr, n_doses, t):
    """Superposition principle for multiple dosing."""
    total = np.zeros_like(t)
    for i in range(n_doses):
        t_shifted = t - i * dose_interval_hr
        contribution = np.where(t_shifted >= 0, single_dose_func(t_shifted), 0.0)
        total += contribution
    return total


def main():
    total_passed = 0
    total_failed = 0
    baseline = {}

    print("=" * 72)
    print("healthSpring Exp002: One-Compartment PK Models")
    print("=" * 72)

    # === IV BOLUS ===

    k_e_iv = np.log(2) / DRUG_IV["half_life_hr"]
    c0_iv = DRUG_IV["dose_mg"] / DRUG_IV["vd_L"]

    # ------------------------------------------------------------------
    # Check 1: C(0) = Dose/Vd
    # ------------------------------------------------------------------
    print("\n--- Check 1: IV bolus C(0) = Dose/Vd ---")
    c_at_0 = pk_iv_bolus(DRUG_IV["dose_mg"], DRUG_IV["vd_L"], DRUG_IV["half_life_hr"], 0.0)
    expected_c0 = DRUG_IV["dose_mg"] / DRUG_IV["vd_L"]
    baseline["iv_c0"] = float(c_at_0)
    if abs(c_at_0 - expected_c0) < 1e-10:
        print(f"  [PASS] C(0) = {c_at_0:.4f} mg/L (expected {expected_c0:.4f})")
        total_passed += 1
    else:
        print(f"  [FAIL] C(0) = {c_at_0:.4f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 2: At t = half-life, C = C0/2
    # ------------------------------------------------------------------
    print("\n--- Check 2: IV bolus at t=half-life → C0/2 ---")
    c_at_hl = pk_iv_bolus(DRUG_IV["dose_mg"], DRUG_IV["vd_L"],
                           DRUG_IV["half_life_hr"], DRUG_IV["half_life_hr"])
    expected_hl = c0_iv / 2.0
    baseline["iv_at_half_life"] = float(c_at_hl)
    if abs(c_at_hl - expected_hl) < 1e-6:
        print(f"  [PASS] C(t½) = {c_at_hl:.6f} (expected {expected_hl:.6f})")
        total_passed += 1
    else:
        print(f"  [FAIL] C(t½) = {c_at_hl:.6f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 3: IV curve monotonically decreasing
    # ------------------------------------------------------------------
    print("\n--- Check 3: IV bolus monotonically decreasing ---")
    c_iv = pk_iv_bolus(DRUG_IV["dose_mg"], DRUG_IV["vd_L"],
                        DRUG_IV["half_life_hr"], TIME_HR)
    mono_dec = all(c_iv[i] >= c_iv[i + 1] for i in range(len(c_iv) - 1))
    baseline["iv_monotonic_dec"] = mono_dec
    if mono_dec:
        print(f"  [PASS] IV curve monotonically decreasing over 48hr")
        total_passed += 1
    else:
        print(f"  [FAIL] Not monotonically decreasing")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 4: IV AUC = Dose / (Vd * k_e)
    # ------------------------------------------------------------------
    print("\n--- Check 4: IV AUC analytical check ---")
    auc_iv_numerical = auc_trapezoidal(TIME_HR, c_iv)
    auc_iv_analytical = DRUG_IV["dose_mg"] / (DRUG_IV["vd_L"] * k_e_iv)
    baseline["iv_auc_numerical"] = auc_iv_numerical
    baseline["iv_auc_analytical"] = auc_iv_analytical
    rel_err = abs(auc_iv_numerical - auc_iv_analytical) / auc_iv_analytical
    if rel_err < 0.01:
        print(f"  [PASS] AUC numerical={auc_iv_numerical:.2f}, analytical={auc_iv_analytical:.2f} (err={rel_err:.4%})")
        total_passed += 1
    else:
        print(f"  [FAIL] AUC numerical={auc_iv_numerical:.2f}, analytical={auc_iv_analytical:.2f} (err={rel_err:.4%})")
        total_failed += 1

    # === ORAL DOSING ===

    k_e_oral = np.log(2) / DRUG_ORAL["half_life_hr"]
    k_a_oral = DRUG_ORAL["k_a_hr"]

    # ------------------------------------------------------------------
    # Check 5: Oral C(0) = 0 (no drug absorbed yet)
    # ------------------------------------------------------------------
    print("\n--- Check 5: Oral C(0) = 0 ---")
    c_oral_0 = pk_oral_one_compartment(
        DRUG_ORAL["dose_mg"], DRUG_ORAL["f_bioavail"],
        DRUG_ORAL["vd_L"], k_a_oral, k_e_oral, 0.0)
    baseline["oral_c0"] = float(c_oral_0)
    if abs(c_oral_0) < 1e-10:
        print(f"  [PASS] C(0) = {c_oral_0:.10f}")
        total_passed += 1
    else:
        print(f"  [FAIL] C(0) = {c_oral_0:.10f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 6: Oral Cmax occurs at Tmax > 0
    # ------------------------------------------------------------------
    print("\n--- Check 6: Oral Cmax at Tmax > 0 ---")
    c_oral = pk_oral_one_compartment(
        DRUG_ORAL["dose_mg"], DRUG_ORAL["f_bioavail"],
        DRUG_ORAL["vd_L"], k_a_oral, k_e_oral, TIME_HR)
    cmax, tmax = find_cmax_tmax(TIME_HR, c_oral)
    baseline["oral_cmax"] = cmax
    baseline["oral_tmax"] = tmax
    if tmax > 0 and cmax > 0:
        print(f"  [PASS] Cmax = {cmax:.4f} mg/L at Tmax = {tmax:.2f} hr")
        total_passed += 1
    else:
        print(f"  [FAIL] Cmax={cmax}, Tmax={tmax}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 7: Analytical Tmax = ln(k_a/k_e) / (k_a - k_e)
    # ------------------------------------------------------------------
    print("\n--- Check 7: Tmax analytical formula ---")
    tmax_analytical = np.log(k_a_oral / k_e_oral) / (k_a_oral - k_e_oral)
    baseline["oral_tmax_analytical"] = float(tmax_analytical)
    if abs(tmax - tmax_analytical) < 0.1:  # within 0.1 hr (discretization)
        print(f"  [PASS] Tmax numerical={tmax:.3f}, analytical={tmax_analytical:.3f} hr")
        total_passed += 1
    else:
        print(f"  [FAIL] Tmax numerical={tmax:.3f}, analytical={tmax_analytical:.3f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 8: Oral curve returns to near-zero by 48hr
    # ------------------------------------------------------------------
    print("\n--- Check 8: Oral concentration → 0 by 48hr ---")
    c_at_48 = float(c_oral[-1])
    baseline["oral_c_48hr"] = c_at_48
    if c_at_48 < 0.01:
        print(f"  [PASS] C(48hr) = {c_at_48:.6f} mg/L (< 0.01)")
        total_passed += 1
    else:
        print(f"  [FAIL] C(48hr) = {c_at_48:.6f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 9: Oral AUC > 0
    # ------------------------------------------------------------------
    print("\n--- Check 9: Oral AUC > 0 ---")
    auc_oral = auc_trapezoidal(TIME_HR, c_oral)
    baseline["oral_auc"] = auc_oral
    if auc_oral > 0:
        print(f"  [PASS] Oral AUC = {auc_oral:.2f} mg·hr/L")
        total_passed += 1
    else:
        print(f"  [FAIL] Oral AUC = {auc_oral:.2f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 10: Oral AUC = F * Dose / (Vd * k_e)
    # ------------------------------------------------------------------
    print("\n--- Check 10: Oral AUC analytical ---")
    auc_oral_analytical = (DRUG_ORAL["f_bioavail"] * DRUG_ORAL["dose_mg"]) / (DRUG_ORAL["vd_L"] * k_e_oral)
    baseline["oral_auc_analytical"] = auc_oral_analytical
    rel_err_oral = abs(auc_oral - auc_oral_analytical) / auc_oral_analytical
    if rel_err_oral < 0.01:
        print(f"  [PASS] AUC numerical={auc_oral:.2f}, analytical={auc_oral_analytical:.2f} (err={rel_err_oral:.4%})")
        total_passed += 1
    else:
        print(f"  [FAIL] err={rel_err_oral:.4%}")
        total_failed += 1

    # === MULTIPLE DOSING ===

    # ------------------------------------------------------------------
    # Check 11: Multiple IV doses accumulate
    # ------------------------------------------------------------------
    print("\n--- Check 11: Multiple IV doses accumulate ---")
    def single_iv(t):
        return pk_iv_bolus(DRUG_IV["dose_mg"], DRUG_IV["vd_L"],
                           DRUG_IV["half_life_hr"], t)
    dose_interval = 8.0  # every 8 hours
    c_multi = pk_multiple_dose(single_iv, dose_interval, 6, TIME_HR)
    cmax_multi, _ = find_cmax_tmax(TIME_HR[TIME_HR >= dose_interval], c_multi[TIME_HR >= dose_interval])
    baseline["multi_dose_cmax_after_first"] = float(cmax_multi)
    if cmax_multi > c0_iv:
        print(f"  [PASS] Multi-dose Cmax={cmax_multi:.4f} > single C0={c0_iv:.4f}")
        total_passed += 1
    else:
        print(f"  [FAIL] No accumulation: {cmax_multi:.4f} <= {c0_iv:.4f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 12: All concentrations non-negative
    # ------------------------------------------------------------------
    print("\n--- Check 12: All concentrations ≥ 0 ---")
    all_nonneg_iv = np.all(c_iv >= 0)
    all_nonneg_oral = np.all(c_oral >= 0)
    all_nonneg_multi = np.all(c_multi >= -1e-12)
    baseline["nonneg_iv"] = bool(all_nonneg_iv)
    baseline["nonneg_oral"] = bool(all_nonneg_oral)
    baseline["nonneg_multi"] = bool(all_nonneg_multi)
    if all_nonneg_iv and all_nonneg_oral and all_nonneg_multi:
        print(f"  [PASS] All curves non-negative")
        total_passed += 1
    else:
        print(f"  [FAIL] Negative concentrations detected")
        total_failed += 1

    # ------------------------------------------------------------------
    # Write baseline JSON
    # ------------------------------------------------------------------
    baseline_path = os.path.join(SCRIPT_DIR, "exp002_baseline.json")
    baseline_out = {
        "_source": "healthSpring Exp002: One-Compartment PK Models",
        "_method": "IV bolus + oral Bateman + multiple dosing",
        "drug_iv": DRUG_IV,
        "drug_oral": DRUG_ORAL,
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

    # ------------------------------------------------------------------
    # Summary
    # ------------------------------------------------------------------
    total = total_passed + total_failed
    print(f"\n{'=' * 72}")
    print(f"TOTAL: {total_passed}/{total} PASS, {total_failed}/{total} FAIL")
    print(f"{'=' * 72}")

    return 0 if total_failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
