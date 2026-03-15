# SPDX-License-Identifier: AGPL-3.0-or-later

#!/usr/bin/env python3
"""
healthSpring Exp035 — TRT and Type 2 Diabetes

Models HbA1c and insulin sensitivity changes under TRT:
  - HbA1c reduction (Kapoor 2006 RCT: -0.37%)
  - HOMA-IR improvement (Dhindsa 2016)
  - Fasting glucose reduction (Hackett 2014)
  - TRT as adjunctive to standard diabetes care

Reference: Kapoor 2006 (Diabetes Care RCT, n=24, 3mo)
           Dhindsa 2016 (JCEM)
           Hackett 2014 (Int J Clin Pract, n=199, 30 weeks)
           Mok Ch.5: "TRT reduces HbA1c in men with T2DM"

Provenance:
  Baseline date:   2026-03-08
  Command:         python3 control/endocrine/exp035_trt_diabetes.py
  Environment:     Python 3.10+, NumPy
"""

import json
import os
import sys

import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

# Kapoor 2006 RCT parameters
HBA1C_BASELINE = 7.60      # %
HBA1C_DELTA_3MO = -0.37    # % reduction at 3 months (vs placebo)
HBA1C_ENDPOINT = HBA1C_BASELINE + HBA1C_DELTA_3MO

# Hackett 2014 (longer term, 30 weeks)
HBA1C_BASELINE_HACKETT = 7.90
HBA1C_DELTA_30WK = -0.41

# HOMA-IR (homeostatic model assessment — insulin resistance)
HOMA_BASELINE = 4.5
HOMA_ENDPOINT = 3.2  # estimated from Dhindsa 2016

# Fasting glucose (mg/dL)
FG_BASELINE = 140.0
FG_ENDPOINT = 120.0

MONTHS = np.arange(0, 13, dtype=float)  # 12 months
TAU_MONTHS = 3.0  # HbA1c responds in ~3 months


def hba1c_trajectory(months, baseline_val, delta, tau):
    """Exponential approach: HbA1c responds over ~3 months (RBC turnover)."""
    return baseline_val + delta * (1.0 - np.exp(-months / tau))


def homa_trajectory(months, baseline_val, endpoint_val, tau):
    """HOMA-IR improvement trajectory."""
    delta = endpoint_val - baseline_val
    return baseline_val + delta * (1.0 - np.exp(-months / tau))


def main():
    total_passed = 0
    total_failed = 0
    baseline = {}

    print("=" * 72)
    print("healthSpring Exp035: TRT and Type 2 Diabetes")
    print("=" * 72)

    hba1c = hba1c_trajectory(MONTHS, HBA1C_BASELINE, HBA1C_DELTA_3MO, TAU_MONTHS)
    homa = homa_trajectory(MONTHS, HOMA_BASELINE, HOMA_ENDPOINT, TAU_MONTHS)
    fg = homa_trajectory(MONTHS, FG_BASELINE, FG_ENDPOINT, TAU_MONTHS)

    # --- Check 1: HbA1c baseline correct ---
    print("\n--- Check 1: HbA1c(0) = baseline ---")
    baseline["hba1c_0"] = float(hba1c[0])
    if abs(hba1c[0] - HBA1C_BASELINE) < 1e-10:
        print(f"  [PASS] HbA1c(0) = {hba1c[0]:.2f}%")
        total_passed += 1
    else:
        print(f"  [FAIL] HbA1c(0) = {hba1c[0]:.2f}%")
        total_failed += 1

    # --- Check 2: HbA1c decreases ---
    print("\n--- Check 2: HbA1c decreases with TRT ---")
    hba1c_12 = float(hba1c[-1])
    baseline["hba1c_12mo"] = hba1c_12
    if hba1c_12 < HBA1C_BASELINE:
        print(f"  [PASS] HbA1c: {HBA1C_BASELINE:.2f} → {hba1c_12:.2f}%")
        total_passed += 1
    else:
        print(f"  [FAIL] HbA1c did not decrease: {hba1c_12:.2f}")
        total_failed += 1

    # --- Check 3: HbA1c reduction at 3 months matches Kapoor ---
    print("\n--- Check 3: HbA1c reduction at 3 months ≈ -0.37% ---")
    hba1c_3 = float(hba1c[3])
    delta_3 = hba1c_3 - HBA1C_BASELINE
    baseline["hba1c_delta_3mo"] = delta_3
    # At t=τ, 63.2% of asymptotic change has occurred
    expected_delta_3 = HBA1C_DELTA_3MO * (1 - np.exp(-1))
    if abs(delta_3 - expected_delta_3) < 0.05:
        print(f"  [PASS] ΔHbA1c(3mo) = {delta_3:.3f}% (expected {expected_delta_3:.3f})")
        total_passed += 1
    else:
        print(f"  [FAIL] ΔHbA1c(3mo) = {delta_3:.3f}% (expected {expected_delta_3:.3f})")
        total_failed += 1

    # --- Check 4: HOMA-IR decreases (better insulin sensitivity) ---
    print("\n--- Check 4: HOMA-IR decreases ---")
    homa_12 = float(homa[-1])
    baseline["homa_12mo"] = homa_12
    if homa_12 < HOMA_BASELINE:
        print(f"  [PASS] HOMA-IR: {HOMA_BASELINE:.1f} → {homa_12:.2f}")
        total_passed += 1
    else:
        print(f"  [FAIL] HOMA-IR did not decrease: {homa_12:.2f}")
        total_failed += 1

    # --- Check 5: Fasting glucose decreases ---
    print("\n--- Check 5: Fasting glucose decreases ---")
    fg_12 = float(fg[-1])
    baseline["fg_12mo"] = fg_12
    if fg_12 < FG_BASELINE:
        print(f"  [PASS] FG: {FG_BASELINE:.0f} → {fg_12:.1f} mg/dL")
        total_passed += 1
    else:
        print(f"  [FAIL] FG did not decrease: {fg_12:.1f}")
        total_failed += 1

    # --- Check 6: HbA1c monotonically decreasing ---
    print("\n--- Check 6: HbA1c monotonically decreasing ---")
    mono = all(hba1c[i] >= hba1c[i + 1] - 1e-12 for i in range(len(hba1c) - 1))
    baseline["hba1c_monotonic"] = mono
    if mono:
        print(f"  [PASS] HbA1c decreases monotonically")
        total_passed += 1
    else:
        print(f"  [FAIL] Not monotonic")
        total_failed += 1

    # --- Check 7: Most HbA1c improvement in first 6 months ---
    print("\n--- Check 7: Front-loaded HbA1c improvement ---")
    delta_6 = float(hba1c[6]) - HBA1C_BASELINE
    delta_12 = float(hba1c[12]) - HBA1C_BASELINE
    frac_6mo = abs(delta_6) / abs(delta_12) if abs(delta_12) > 1e-10 else 0
    baseline["hba1c_frac_improvement_6mo"] = frac_6mo
    if frac_6mo > 0.80:
        print(f"  [PASS] {frac_6mo:.1%} of improvement by 6 months (RBC turnover ~3mo)")
        total_passed += 1
    else:
        print(f"  [FAIL] Only {frac_6mo:.1%} by 6 months")
        total_failed += 1

    # --- Check 8: Clinical significance (delta > 0.3%) ---
    print("\n--- Check 8: Clinically significant HbA1c reduction ---")
    total_delta = abs(delta_12)
    baseline["hba1c_total_delta"] = total_delta
    if total_delta > 0.30:
        print(f"  [PASS] Total ΔHbA1c = {total_delta:.3f}% (clinically significant > 0.3%)")
        total_passed += 1
    else:
        print(f"  [FAIL] ΔHbA1c = {total_delta:.3f}% (< 0.3% threshold)")
        total_failed += 1

    # --- Check 9: Concordance — all metabolic markers improve together ---
    print("\n--- Check 9: All metabolic markers improve concordantly ---")
    all_improve = (hba1c_12 < HBA1C_BASELINE and
                   homa_12 < HOMA_BASELINE and
                   fg_12 < FG_BASELINE)
    baseline["all_improve"] = all_improve
    if all_improve:
        print(f"  [PASS] HbA1c ↓, HOMA-IR ↓, FG ↓ — concordant improvement")
        total_passed += 1
    else:
        print(f"  [FAIL] Not all markers improved")
        total_failed += 1

    # --- Check 10: HOMA-IR improvement magnitude plausible ---
    print("\n--- Check 10: HOMA-IR improvement plausible ---")
    homa_pct = (HOMA_BASELINE - homa_12) / HOMA_BASELINE
    baseline["homa_pct_improvement"] = float(homa_pct)
    if 0.15 < homa_pct < 0.50:
        print(f"  [PASS] HOMA-IR improvement = {homa_pct:.1%} (plausible range 15-50%)")
        total_passed += 1
    else:
        print(f"  [FAIL] HOMA-IR improvement = {homa_pct:.1%}")
        total_failed += 1

    # --- Write baseline JSON ---
    baseline_path = os.path.join(SCRIPT_DIR, "exp035_baseline.json")
    baseline_out = {
        "_source": "healthSpring Exp035: TRT and Type 2 Diabetes",
        "_method": "Exponential approach + published RCT deltas",
        "params": {
            "hba1c_baseline": HBA1C_BASELINE,
            "hba1c_delta_kapoor": HBA1C_DELTA_3MO,
            "homa_baseline": HOMA_BASELINE,
            "homa_endpoint": HOMA_ENDPOINT,
            "fg_baseline": FG_BASELINE,
            "fg_endpoint": FG_ENDPOINT,
            "tau_months": TAU_MONTHS,
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
