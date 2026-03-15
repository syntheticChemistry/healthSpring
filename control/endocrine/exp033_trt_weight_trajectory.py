# SPDX-License-Identifier: AGPL-3.0-only

#!/usr/bin/env python3
"""
healthSpring Exp033 — TRT Metabolic Response: Weight/Waist Trajectory

Models long-term weight and waist circumference changes under TRT:
  - Logarithmic trajectory: ΔBW(t) = β * ln(1 + t/τ)
  - Saad 2013 registry (n=411): mean -16kg over 5 years
  - Saad 2016 (n=411): waist -12cm over 5 years
  - Interrupted TRT: rebound effect (weight regain)

Reference: Saad 2013 (Obesity), Saad 2016 (Int J Obesity, n=411)
           Traish 2014 (IJCP, n=261+260)
           Mok Ch.4: "TRT results in sustained long-term weight loss"

Provenance:
  Baseline date:   2026-03-08
  Command:         python3 control/endocrine/exp033_trt_weight_trajectory.py
  Environment:     Python 3.10+, NumPy
"""

import json
import os
import sys

import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

# Saad 2013 registry data (digitized from published curves)
# Continuous TRT group (n=264), 5 years
BASELINE_WEIGHT_KG = 113.0
BASELINE_WAIST_CM = 112.0
BASELINE_BMI = 37.0

# Observed endpoints at 5 years (Saad 2013):
WEIGHT_LOSS_5YR_KG = -16.0      # mean weight loss
WAIST_LOSS_5YR_CM = -12.0       # mean waist reduction
BMI_LOSS_5YR = -5.6

# Time constant for logarithmic trajectory
TAU_MONTHS = 6.0   # characteristic time
MONTHS = np.arange(0, 61, dtype=float)  # 0 to 60 months (5 years)


def weight_trajectory(months, delta_final, tau, duration=60.0):
    """Logarithmic weight change: ΔW(t) = delta_final * ln(1 + t/τ) / ln(1 + duration/τ)."""
    norm = np.log(1 + duration / tau)
    return delta_final * np.log(1 + months / tau) / norm


def interrupted_trajectory(months, delta_final, tau, stop_month, resume_month, duration=60.0):
    """Weight trajectory with TRT interruption: rebound during gap, resume after."""
    result = np.zeros_like(months, dtype=float)
    norm = np.log(1 + duration / tau)

    for i, m in enumerate(months):
        if m <= stop_month:
            result[i] = delta_final * np.log(1 + m / tau) / norm
        elif m <= resume_month:
            # Rebound: exponential return toward baseline
            w_at_stop = delta_final * np.log(1 + stop_month / tau) / norm
            rebound_tau = 12.0  # months to rebound
            gap = m - stop_month
            result[i] = w_at_stop * np.exp(-gap / rebound_tau)
        else:
            w_at_resume = delta_final * np.log(1 + stop_month / tau) / norm * np.exp(-(resume_month - stop_month) / 12.0)
            extra = m - resume_month
            additional_loss = (delta_final - w_at_resume) * np.log(1 + extra / tau) / norm
            result[i] = w_at_resume + additional_loss

    return result


def main():
    total_passed = 0
    total_failed = 0
    baseline = {}

    print("=" * 72)
    print("healthSpring Exp033: TRT Weight/Waist Trajectory")
    print("=" * 72)

    dw = weight_trajectory(MONTHS, WEIGHT_LOSS_5YR_KG, TAU_MONTHS)
    dwc = weight_trajectory(MONTHS, WAIST_LOSS_5YR_CM, TAU_MONTHS)

    # --- Check 1: ΔW(0) = 0 ---
    print("\n--- Check 1: ΔW(0) = 0 ---")
    baseline["dw_at_0"] = float(dw[0])
    if abs(dw[0]) < 1e-10:
        print(f"  [PASS] ΔW(0) = {dw[0]:.10f}")
        total_passed += 1
    else:
        print(f"  [FAIL] ΔW(0) = {dw[0]:.10f}")
        total_failed += 1

    # --- Check 2: ΔW(60) matches Saad 2013 target ---
    print("\n--- Check 2: ΔW(60 months) ≈ -16 kg ---")
    dw_60 = float(dw[-1])
    baseline["dw_at_60mo"] = dw_60
    if abs(dw_60 - WEIGHT_LOSS_5YR_KG) / abs(WEIGHT_LOSS_5YR_KG) < 0.01:
        print(f"  [PASS] ΔW(5yr) = {dw_60:.2f} kg (target {WEIGHT_LOSS_5YR_KG})")
        total_passed += 1
    else:
        print(f"  [FAIL] ΔW(5yr) = {dw_60:.2f} kg")
        total_failed += 1

    # --- Check 3: Weight loss monotonically increases ---
    print("\n--- Check 3: Weight loss monotonically increasing ---")
    mono = all(dw[i] >= dw[i + 1] for i in range(len(dw) - 1))
    baseline["weight_mono_dec"] = mono
    if mono:
        print(f"  [PASS] Weight decreases monotonically")
        total_passed += 1
    else:
        print(f"  [FAIL] Not monotonically decreasing")
        total_failed += 1

    # --- Check 4: Most weight loss in first 2 years (logarithmic shape) ---
    print("\n--- Check 4: Front-loaded weight loss (logarithmic) ---")
    dw_24 = float(dw[24])
    frac_early = dw_24 / dw_60 if dw_60 != 0 else 0
    baseline["frac_loss_by_24mo"] = frac_early
    if frac_early > 0.60:
        print(f"  [PASS] {frac_early:.1%} of total loss by 24 months (logarithmic)")
        total_passed += 1
    else:
        print(f"  [FAIL] Only {frac_early:.1%} by 24 months")
        total_failed += 1

    # --- Check 5: Waist circumference parallels weight ---
    print("\n--- Check 5: Waist trajectory parallels weight ---")
    dwc_60 = float(dwc[-1])
    baseline["dwc_at_60mo"] = dwc_60
    if abs(dwc_60 - WAIST_LOSS_5YR_CM) / abs(WAIST_LOSS_5YR_CM) < 0.01:
        print(f"  [PASS] ΔWaist(5yr) = {dwc_60:.2f} cm (target {WAIST_LOSS_5YR_CM})")
        total_passed += 1
    else:
        print(f"  [FAIL] ΔWaist(5yr) = {dwc_60:.2f} cm")
        total_failed += 1

    # --- Check 6: BMI trajectory ---
    print("\n--- Check 6: BMI reduction ---")
    dw_bmi = weight_trajectory(MONTHS, BMI_LOSS_5YR, TAU_MONTHS)
    bmi_final = float(dw_bmi[-1])
    baseline["dbmi_at_60mo"] = bmi_final
    if abs(bmi_final - BMI_LOSS_5YR) / abs(BMI_LOSS_5YR) < 0.01:
        print(f"  [PASS] ΔBMI(5yr) = {bmi_final:.2f} (target {BMI_LOSS_5YR})")
        total_passed += 1
    else:
        print(f"  [FAIL] ΔBMI(5yr) = {bmi_final:.2f}")
        total_failed += 1

    # --- Check 7: Interrupted TRT shows rebound ---
    print("\n--- Check 7: Interrupted TRT → weight rebound ---")
    dw_int = interrupted_trajectory(MONTHS, WEIGHT_LOSS_5YR_KG, TAU_MONTHS,
                                     stop_month=24.0, resume_month=36.0)
    dw_cont = dw.copy()
    # At month 30 (mid-interruption), interrupted should be less negative than continuous
    w_30_int = float(dw_int[30])
    w_30_cont = float(dw_cont[30])
    baseline["w_30_interrupted"] = w_30_int
    baseline["w_30_continuous"] = w_30_cont
    if w_30_int > w_30_cont:
        print(f"  [PASS] Interrupted ΔW(30mo) = {w_30_int:.2f} > continuous = {w_30_cont:.2f} (rebound)")
        total_passed += 1
    else:
        print(f"  [FAIL] No rebound: interrupted = {w_30_int:.2f}, continuous = {w_30_cont:.2f}")
        total_failed += 1

    # --- Check 8: Resumed TRT resumes weight loss ---
    print("\n--- Check 8: Resumed TRT resumes weight loss ---")
    w_48_int = float(dw_int[48])
    w_36_int = float(dw_int[36])
    baseline["w_48_interrupted"] = w_48_int
    baseline["w_36_interrupted"] = w_36_int
    if w_48_int < w_36_int:
        print(f"  [PASS] After resume: ΔW(48)={w_48_int:.2f} < ΔW(36)={w_36_int:.2f}")
        total_passed += 1
    else:
        print(f"  [FAIL] No resumed loss: {w_48_int:.2f} >= {w_36_int:.2f}")
        total_failed += 1

    # --- Check 9: Interrupted group has less total loss ---
    print("\n--- Check 9: Interrupted < continuous at endpoint ---")
    w_60_int = float(dw_int[-1])
    baseline["w_60_interrupted"] = w_60_int
    if w_60_int > dw_60:
        print(f"  [PASS] Interrupted ΔW(60)={w_60_int:.2f} > continuous ΔW(60)={dw_60:.2f}")
        total_passed += 1
    else:
        print(f"  [FAIL] Interrupted {w_60_int:.2f} <= continuous {dw_60:.2f}")
        total_failed += 1

    # --- Check 10: Annual weight loss rate decreases (decelerating) ---
    print("\n--- Check 10: Decelerating weight loss rate ---")
    rate_yr1 = float(dw[12] - dw[0])
    rate_yr5 = float(dw[60] - dw[48])
    baseline["rate_yr1_kg"] = rate_yr1
    baseline["rate_yr5_kg"] = rate_yr5
    if abs(rate_yr1) > abs(rate_yr5):
        print(f"  [PASS] Year 1 rate = {rate_yr1:.2f} kg > Year 5 rate = {rate_yr5:.2f} kg")
        total_passed += 1
    else:
        print(f"  [FAIL] Not decelerating: yr1={rate_yr1:.2f}, yr5={rate_yr5:.2f}")
        total_failed += 1

    # --- Write baseline JSON ---
    baseline_path = os.path.join(SCRIPT_DIR, "exp033_baseline.json")
    baseline_out = {
        "_source": "healthSpring Exp033: TRT Weight/Waist Trajectory",
        "_method": "Logarithmic trajectory + interrupted rebound",
        "params": {
            "baseline_weight_kg": BASELINE_WEIGHT_KG,
            "baseline_waist_cm": BASELINE_WAIST_CM,
            "weight_loss_5yr_kg": WEIGHT_LOSS_5YR_KG,
            "waist_loss_5yr_cm": WAIST_LOSS_5YR_CM,
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
