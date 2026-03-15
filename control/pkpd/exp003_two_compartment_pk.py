# SPDX-License-Identifier: AGPL-3.0-only

#!/usr/bin/env python3
"""
healthSpring Exp003 — Two-Compartment PK Model (IV Bolus)

Validates the two-compartment open model for IV bolus administration:
  C(t) = A * exp(-α * t) + B * exp(-β * t)

where:
  α = fast distribution phase rate constant
  β = slow elimination phase rate constant
  A, B = macro constants derived from micro-constants (k12, k21, k10)

Micro-constant model:
  k10 = elimination from central
  k12 = distribution central → peripheral
  k21 = redistribution peripheral → central

Reference: Rowland & Tozer, Clinical Pharmacokinetics (Chapter 19)
Extension: Exp002 one-compartment → two-compartment

Provenance:
  Baseline date:   2026-03-08
  Command:         python3 control/pkpd/exp003_two_compartment_pk.py
  Environment:     Python 3.10+, NumPy
"""

import json
import os
import sys

import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

# Model drug: gentamicin-like aminoglycoside (IV bolus, two-compartment)
DRUG = {
    "name": "Gentamicin-like",
    "dose_mg": 240.0,
    "v1_L": 15.0,      # central volume
    "k10_hr": 0.35,     # elimination rate from central
    "k12_hr": 0.6,      # distribution central → peripheral
    "k21_hr": 0.15,     # redistribution peripheral → central
}

TIME_HR = np.linspace(0, 168, 2000)  # 7 days — sufficient for β-phase washout


def micro_to_macro(k10, k12, k21):
    """Convert micro-constants to macro-constants (α, β, A_coeff, B_coeff).

    α + β = k10 + k12 + k21
    α * β = k10 * k21
    """
    s = k10 + k12 + k21
    p = k10 * k21
    discriminant = s ** 2 - 4 * p
    if discriminant < 0:
        raise ValueError("Negative discriminant — invalid micro-constants")
    sqrt_d = np.sqrt(discriminant)
    alpha = (s + sqrt_d) / 2.0
    beta = (s - sqrt_d) / 2.0
    return alpha, beta


def two_compartment_iv(dose, v1, k10, k12, k21, t):
    """Two-compartment IV bolus: C(t) = A*exp(-α*t) + B*exp(-β*t)."""
    alpha, beta = micro_to_macro(k10, k12, k21)
    c0 = dose / v1
    A = c0 * (alpha - k21) / (alpha - beta)
    B = c0 * (k21 - beta) / (alpha - beta)
    return A * np.exp(-alpha * t) + B * np.exp(-beta * t)


def peripheral_concentration(dose, v1, k10, k12, k21, t, v2=None):
    """Peripheral compartment amount over time (numerical integration)."""
    dt = t[1] - t[0]
    c_central = two_compartment_iv(dose, v1, k10, k12, k21, t)
    a_periph = np.zeros_like(t)
    for i in range(1, len(t)):
        da = k12 * c_central[i - 1] * v1 - k21 * a_periph[i - 1]
        a_periph[i] = a_periph[i - 1] + da * dt
    if v2 is None:
        v2 = v1 * k12 / k21
    return a_periph / v2


def auc_trapezoidal(t, c):
    """AUC by trapezoidal rule."""
    return float(np.trapezoid(c, t))


def main():
    total_passed = 0
    total_failed = 0
    baseline = {}

    d = DRUG
    alpha, beta = micro_to_macro(d["k10_hr"], d["k12_hr"], d["k21_hr"])

    print("=" * 72)
    print("healthSpring Exp003: Two-Compartment PK (IV Bolus)")
    print("=" * 72)
    print(f"\n  Drug: {d['name']}")
    print(f"  α = {alpha:.6f} hr⁻¹  (distribution)")
    print(f"  β = {beta:.6f} hr⁻¹  (elimination)")

    baseline["alpha"] = float(alpha)
    baseline["beta"] = float(beta)

    # ------------------------------------------------------------------
    # Check 1: α > β (distribution faster than elimination)
    # ------------------------------------------------------------------
    print("\n--- Check 1: α > β ---")
    if alpha > beta:
        print(f"  [PASS] α={alpha:.6f} > β={beta:.6f}")
        total_passed += 1
    else:
        print(f"  [FAIL] α={alpha:.6f} <= β={beta:.6f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 2: α + β = k10 + k12 + k21
    # ------------------------------------------------------------------
    print("\n--- Check 2: α + β = k10 + k12 + k21 ---")
    sum_macro = alpha + beta
    sum_micro = d["k10_hr"] + d["k12_hr"] + d["k21_hr"]
    baseline["sum_macro"] = float(sum_macro)
    baseline["sum_micro"] = float(sum_micro)
    if abs(sum_macro - sum_micro) < 1e-10:
        print(f"  [PASS] {sum_macro:.10f} == {sum_micro:.10f}")
        total_passed += 1
    else:
        print(f"  [FAIL] {sum_macro:.10f} != {sum_micro:.10f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 3: α * β = k10 * k21
    # ------------------------------------------------------------------
    print("\n--- Check 3: α * β = k10 * k21 ---")
    prod_macro = alpha * beta
    prod_micro = d["k10_hr"] * d["k21_hr"]
    baseline["prod_macro"] = float(prod_macro)
    baseline["prod_micro"] = float(prod_micro)
    if abs(prod_macro - prod_micro) < 1e-10:
        print(f"  [PASS] {prod_macro:.10f} == {prod_micro:.10f}")
        total_passed += 1
    else:
        print(f"  [FAIL] {prod_macro:.10f} != {prod_micro:.10f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 4: C(0) = Dose / V1
    # ------------------------------------------------------------------
    print("\n--- Check 4: C(0) = Dose / V1 ---")
    c_at_0 = two_compartment_iv(d["dose_mg"], d["v1_L"], d["k10_hr"],
                                 d["k12_hr"], d["k21_hr"], 0.0)
    c0_expected = d["dose_mg"] / d["v1_L"]
    baseline["c0"] = float(c_at_0)
    if abs(c_at_0 - c0_expected) < 1e-10:
        print(f"  [PASS] C(0) = {c_at_0:.6f} mg/L")
        total_passed += 1
    else:
        print(f"  [FAIL] C(0) = {c_at_0:.6f}, expected {c0_expected:.6f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 5: Biexponential curve is always non-negative
    # ------------------------------------------------------------------
    print("\n--- Check 5: All concentrations ≥ 0 ---")
    c_curve = two_compartment_iv(d["dose_mg"], d["v1_L"], d["k10_hr"],
                                  d["k12_hr"], d["k21_hr"], TIME_HR)
    all_nonneg = np.all(c_curve >= -1e-12)
    baseline["all_nonneg"] = bool(all_nonneg)
    if all_nonneg:
        print(f"  [PASS] min(C) = {np.min(c_curve):.6e}")
        total_passed += 1
    else:
        print(f"  [FAIL] min(C) = {np.min(c_curve):.6e}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 6: Curve monotonically decreasing (central compartment, IV bolus)
    # ------------------------------------------------------------------
    print("\n--- Check 6: Central concentration monotonically decreasing ---")
    mono_dec = all(c_curve[i] >= c_curve[i + 1] - 1e-12 for i in range(len(c_curve) - 1))
    baseline["monotonic_dec"] = mono_dec
    if mono_dec:
        print(f"  [PASS]")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 7: Distribution half-life < elimination half-life
    # ------------------------------------------------------------------
    print("\n--- Check 7: t½α < t½β ---")
    t_half_alpha = np.log(2) / alpha
    t_half_beta = np.log(2) / beta
    baseline["t_half_alpha"] = float(t_half_alpha)
    baseline["t_half_beta"] = float(t_half_beta)
    if t_half_alpha < t_half_beta:
        print(f"  [PASS] t½α = {t_half_alpha:.3f} hr < t½β = {t_half_beta:.3f} hr")
        total_passed += 1
    else:
        print(f"  [FAIL] t½α = {t_half_alpha:.3f} >= t½β = {t_half_beta:.3f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 8: AUC = Dose / (V1 * k10) [analytical]
    # ------------------------------------------------------------------
    print("\n--- Check 8: AUC analytical ---")
    auc_numerical = auc_trapezoidal(TIME_HR, c_curve)
    auc_analytical = d["dose_mg"] / (d["v1_L"] * d["k10_hr"])
    baseline["auc_numerical"] = auc_numerical
    baseline["auc_analytical"] = auc_analytical
    rel_err = abs(auc_numerical - auc_analytical) / auc_analytical
    if rel_err < 0.01:
        print(f"  [PASS] num={auc_numerical:.2f}, ana={auc_analytical:.2f} (err={rel_err:.4%})")
        total_passed += 1
    else:
        print(f"  [FAIL] err={rel_err:.4%}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 9: A + B = C0 (macro-constant identity)
    # ------------------------------------------------------------------
    print("\n--- Check 9: A + B = C0 ---")
    c0 = d["dose_mg"] / d["v1_L"]
    A_coeff = c0 * (alpha - d["k21_hr"]) / (alpha - beta)
    B_coeff = c0 * (d["k21_hr"] - beta) / (alpha - beta)
    baseline["A_coeff"] = float(A_coeff)
    baseline["B_coeff"] = float(B_coeff)
    if abs(A_coeff + B_coeff - c0) < 1e-10:
        print(f"  [PASS] A={A_coeff:.6f} + B={B_coeff:.6f} = {A_coeff + B_coeff:.6f} = C0")
        total_passed += 1
    else:
        print(f"  [FAIL] A+B = {A_coeff + B_coeff:.6f} != C0={c0:.6f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 10: Peripheral compartment peaks then declines
    # ------------------------------------------------------------------
    print("\n--- Check 10: Peripheral compartment peaks then declines ---")
    c_periph = peripheral_concentration(d["dose_mg"], d["v1_L"], d["k10_hr"],
                                         d["k12_hr"], d["k21_hr"], TIME_HR)
    idx_peak = np.argmax(c_periph)
    baseline["periph_peak_time"] = float(TIME_HR[idx_peak])
    baseline["periph_peak_conc"] = float(c_periph[idx_peak])
    if 0 < idx_peak < len(TIME_HR) - 1:
        print(f"  [PASS] Peripheral peaks at t={TIME_HR[idx_peak]:.2f} hr, C={c_periph[idx_peak]:.4f}")
        total_passed += 1
    else:
        print(f"  [FAIL] Peak at boundary (idx={idx_peak})")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 11: At late times, β-phase dominates (log-linear terminal)
    # ------------------------------------------------------------------
    print("\n--- Check 11: Terminal phase log-linearity ---")
    late_mask = TIME_HR > 8.0
    c_late = c_curve[late_mask]
    t_late = TIME_HR[late_mask]
    log_c_late = np.log(c_late[c_late > 0])
    t_valid = t_late[:len(log_c_late)]
    if len(log_c_late) > 10:
        slope, intercept = np.polyfit(t_valid, log_c_late, 1)
        slope_err = abs(slope - (-beta)) / beta
        baseline["terminal_slope"] = float(slope)
        baseline["terminal_slope_expected"] = float(-beta)
        baseline["terminal_slope_err"] = float(slope_err)
        if slope_err < 0.01:
            print(f"  [PASS] Terminal slope={slope:.6f}, expected={-beta:.6f} (err={slope_err:.4%})")
            total_passed += 1
        else:
            print(f"  [FAIL] slope err={slope_err:.4%}")
            total_failed += 1
    else:
        print(f"  [FAIL] Insufficient late data")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 12: Two-compartment reduces to one-compartment when k12 = 0
    # ------------------------------------------------------------------
    print("\n--- Check 12: k12=0 reduces to one-compartment ---")
    c_one = two_compartment_iv(d["dose_mg"], d["v1_L"], d["k10_hr"], 0.0, d["k21_hr"], TIME_HR)
    c_ref = (d["dose_mg"] / d["v1_L"]) * np.exp(-d["k10_hr"] * TIME_HR)
    max_diff = float(np.max(np.abs(c_one - c_ref)))
    baseline["reduction_max_diff"] = max_diff
    if max_diff < 1e-10:
        print(f"  [PASS] max|diff| = {max_diff:.2e}")
        total_passed += 1
    else:
        print(f"  [FAIL] max|diff| = {max_diff:.2e}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 13: V2 = V1 * k12 / k21
    # ------------------------------------------------------------------
    print("\n--- Check 13: Peripheral volume V2 = V1 * k12 / k21 ---")
    v2 = d["v1_L"] * d["k12_hr"] / d["k21_hr"]
    baseline["v2_L"] = float(v2)
    if v2 > 0:
        print(f"  [PASS] V2 = {v2:.2f} L")
        total_passed += 1
    else:
        print(f"  [FAIL] V2 = {v2:.2f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 14: Vss = V1 + V2 (steady-state volume)
    # ------------------------------------------------------------------
    print("\n--- Check 14: Vss = V1 + V2 ---")
    vss = d["v1_L"] + v2
    baseline["vss_L"] = float(vss)
    if vss > d["v1_L"]:
        print(f"  [PASS] Vss = {vss:.2f} L (> V1={d['v1_L']:.2f})")
        total_passed += 1
    else:
        print(f"  [FAIL] Vss = {vss:.2f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 15: Clearance = V1 * k10
    # ------------------------------------------------------------------
    print("\n--- Check 15: CL = V1 * k10 ---")
    cl = d["v1_L"] * d["k10_hr"]
    cl_from_auc = d["dose_mg"] / auc_analytical
    baseline["clearance"] = float(cl)
    if abs(cl - cl_from_auc) < 1e-10:
        print(f"  [PASS] CL = {cl:.4f} L/hr (= Dose/AUC)")
        total_passed += 1
    else:
        print(f"  [FAIL] CL = {cl:.4f}, Dose/AUC = {cl_from_auc:.4f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Write baseline JSON
    # ------------------------------------------------------------------
    baseline_path = os.path.join(SCRIPT_DIR, "exp003_baseline.json")
    baseline_out = {
        "_source": "healthSpring Exp003: Two-Compartment PK (IV Bolus)",
        "_method": "Biexponential: C(t) = A*exp(-αt) + B*exp(-βt)",
        "drug": DRUG,
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
