# SPDX-License-Identifier: AGPL-3.0-only

#!/usr/bin/env python3
"""
healthSpring Exp032 — Age-Related Testosterone Decline

Models the well-established age-related decline in serum testosterone:
  - Exponential decay: T(age) = T0 * exp(-k * (age - 30))
  - Rate: 1-3% per year after age 30 (Mok Intro, Harman 2001, Feldman 2002)
  - Hypogonadism thresholds: 300 ng/dL (clinical), 280 ng/dL (BCBS <50), 200 ng/dL (BCBS >50)
  - Population variability via lognormal T0

Reference: Harman 2001 (BLSA, n=890), Feldman 2002 (MMAS, n=1709)
           Mok: "beginning at age 30, testosterone in men declines at 1%-3% per year"

Provenance:
  Baseline date:   2026-03-08
  Command:         python3 control/endocrine/exp032_age_testosterone_decline.py
  Environment:     Python 3.10+, NumPy
"""

import json
import os
import sys

import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

# Published parameters
T0_MEAN_NGDL = 600.0   # typical total T at age 30 (ng/dL)
T0_CV = 0.25            # inter-individual CV in baseline T
DECLINE_RATE_LOW = 0.01   # 1% per year
DECLINE_RATE_MID = 0.017  # ~1.7% per year (Harman 2001 estimate)
DECLINE_RATE_HIGH = 0.03  # 3% per year

# Hypogonadism thresholds
THRESHOLD_CLINICAL = 300.0    # ng/dL — clinical low-T
THRESHOLD_BCBS_UNDER50 = 280.0
THRESHOLD_BCBS_OVER50 = 200.0

AGES = np.arange(30, 91, dtype=float)


def testosterone_decline(t0, rate, ages, onset=30.0):
    """Exponential decline: T(age) = T0 * exp(-rate * (age - onset))."""
    return t0 * np.exp(-rate * (ages - onset))


def age_at_threshold(t0, rate, threshold, onset=30.0):
    """Age when T crosses below threshold."""
    if t0 <= threshold:
        return onset
    return onset + np.log(t0 / threshold) / rate


def main():
    total_passed = 0
    total_failed = 0
    baseline = {}

    print("=" * 72)
    print("healthSpring Exp032: Age-Related Testosterone Decline")
    print("=" * 72)

    # --- Check 1: T(30) = T0 ---
    print("\n--- Check 1: T(30) = T0 ---")
    t_at_30 = testosterone_decline(T0_MEAN_NGDL, DECLINE_RATE_MID, np.array([30.0]))[0]
    baseline["t_at_30"] = float(t_at_30)
    if abs(t_at_30 - T0_MEAN_NGDL) < 1e-10:
        print(f"  [PASS] T(30) = {t_at_30:.2f} ng/dL (= T0)")
        total_passed += 1
    else:
        print(f"  [FAIL] T(30) = {t_at_30:.2f} ng/dL (expected {T0_MEAN_NGDL})")
        total_failed += 1

    # --- Check 2: Monotonically decreasing ---
    print("\n--- Check 2: Monotonically decreasing with age ---")
    t_curve = testosterone_decline(T0_MEAN_NGDL, DECLINE_RATE_MID, AGES)
    mono_dec = all(t_curve[i] >= t_curve[i + 1] for i in range(len(t_curve) - 1))
    baseline["monotonic_dec"] = mono_dec
    if mono_dec:
        print(f"  [PASS] T declines monotonically from {t_curve[0]:.1f} to {t_curve[-1]:.1f} ng/dL")
        total_passed += 1
    else:
        print(f"  [FAIL] Not monotonically decreasing")
        total_failed += 1

    # --- Check 3: 1%/yr decline produces ~45% reduction by age 90 ---
    print("\n--- Check 3: 1%/yr → ~45% decline by age 90 ---")
    t_90_low = testosterone_decline(T0_MEAN_NGDL, DECLINE_RATE_LOW, np.array([90.0]))[0]
    pct_remaining_low = t_90_low / T0_MEAN_NGDL
    baseline["pct_remaining_1pct_age90"] = float(pct_remaining_low)
    expected_low = np.exp(-0.01 * 60)  # exp(-0.6) ≈ 0.549
    if abs(pct_remaining_low - expected_low) < 0.01:
        print(f"  [PASS] At 1%/yr: T(90) = {pct_remaining_low:.1%} of T0 (expected {expected_low:.1%})")
        total_passed += 1
    else:
        print(f"  [FAIL] {pct_remaining_low:.1%} vs expected {expected_low:.1%}")
        total_failed += 1

    # --- Check 4: 3%/yr decline produces ~83% reduction by age 90 ---
    print("\n--- Check 4: 3%/yr → ~83% decline by age 90 ---")
    t_90_high = testosterone_decline(T0_MEAN_NGDL, DECLINE_RATE_HIGH, np.array([90.0]))[0]
    pct_remaining_high = t_90_high / T0_MEAN_NGDL
    baseline["pct_remaining_3pct_age90"] = float(pct_remaining_high)
    expected_high = np.exp(-0.03 * 60)
    if abs(pct_remaining_high - expected_high) < 0.01:
        print(f"  [PASS] At 3%/yr: T(90) = {pct_remaining_high:.1%} of T0 (expected {expected_high:.1%})")
        total_passed += 1
    else:
        print(f"  [FAIL] {pct_remaining_high:.1%} vs expected {expected_high:.1%}")
        total_failed += 1

    # --- Check 5: Age at clinical threshold (300 ng/dL) ---
    print("\n--- Check 5: Age at clinical threshold (300 ng/dL) ---")
    age_300_mid = age_at_threshold(T0_MEAN_NGDL, DECLINE_RATE_MID, THRESHOLD_CLINICAL)
    baseline["age_at_300_mid_rate"] = float(age_300_mid)
    if 50 < age_300_mid < 80:
        print(f"  [PASS] At 1.7%/yr: T crosses 300 ng/dL at age {age_300_mid:.1f}")
        total_passed += 1
    else:
        print(f"  [FAIL] Age at 300 = {age_300_mid:.1f} (expected 50-80)")
        total_failed += 1

    # --- Check 6: Faster decline → earlier threshold crossing ---
    print("\n--- Check 6: Faster decline → earlier threshold ---")
    age_300_low = age_at_threshold(T0_MEAN_NGDL, DECLINE_RATE_LOW, THRESHOLD_CLINICAL)
    age_300_high = age_at_threshold(T0_MEAN_NGDL, DECLINE_RATE_HIGH, THRESHOLD_CLINICAL)
    baseline["age_at_300_1pct"] = float(age_300_low)
    baseline["age_at_300_3pct"] = float(age_300_high)
    if age_300_high < age_300_mid < age_300_low:
        print(f"  [PASS] 3%: age {age_300_high:.1f} < 1.7%: age {age_300_mid:.1f} < 1%: age {age_300_low:.1f}")
        total_passed += 1
    else:
        print(f"  [FAIL] Ordering violated")
        total_failed += 1

    # --- Check 7: Population simulation (lognormal T0) ---
    print("\n--- Check 7: Population T0 variability ---")
    rng = np.random.default_rng(42)
    omega_sq = np.log(1 + T0_CV**2)
    mu_ln = np.log(T0_MEAN_NGDL) - omega_sq / 2
    sigma_ln = np.sqrt(omega_sq)
    t0_pop = rng.lognormal(mu_ln, sigma_ln, 10000)
    baseline["pop_t0_mean"] = float(np.mean(t0_pop))
    baseline["pop_t0_median"] = float(np.median(t0_pop))
    baseline["pop_t0_cv"] = float(np.std(t0_pop) / np.mean(t0_pop))
    # Median of lognormal should be close to typical
    if abs(np.median(t0_pop) - T0_MEAN_NGDL) / T0_MEAN_NGDL < 0.05:
        print(f"  [PASS] Pop median T0 = {np.median(t0_pop):.1f} ng/dL (target {T0_MEAN_NGDL})")
        total_passed += 1
    else:
        print(f"  [FAIL] Pop median T0 = {np.median(t0_pop):.1f}")
        total_failed += 1

    # --- Check 8: Fraction hypogonadal by age 60 (population) ---
    print("\n--- Check 8: Fraction hypogonadal by age 60 ---")
    t_at_60 = t0_pop * np.exp(-DECLINE_RATE_MID * 30)
    frac_hypo_60 = float(np.mean(t_at_60 < THRESHOLD_CLINICAL))
    baseline["frac_hypogonadal_age60"] = frac_hypo_60
    # Published estimates: ~20-40% of men over 45 have low T
    if 0.05 < frac_hypo_60 < 0.60:
        print(f"  [PASS] {frac_hypo_60:.1%} hypogonadal at age 60 (plausible range)")
        total_passed += 1
    else:
        print(f"  [FAIL] {frac_hypo_60:.1%} — outside plausible range [5%, 60%]")
        total_failed += 1

    # --- Check 9: Fraction hypogonadal increases with age ---
    print("\n--- Check 9: Fraction hypogonadal increases with age ---")
    t_at_50 = t0_pop * np.exp(-DECLINE_RATE_MID * 20)
    t_at_70 = t0_pop * np.exp(-DECLINE_RATE_MID * 40)
    frac_50 = float(np.mean(t_at_50 < THRESHOLD_CLINICAL))
    frac_70 = float(np.mean(t_at_70 < THRESHOLD_CLINICAL))
    baseline["frac_hypogonadal_age50"] = frac_50
    baseline["frac_hypogonadal_age70"] = frac_70
    if frac_50 < frac_hypo_60 < frac_70:
        print(f"  [PASS] Fraction hypogonadal: age50={frac_50:.1%} < age60={frac_hypo_60:.1%} < age70={frac_70:.1%}")
        total_passed += 1
    else:
        print(f"  [FAIL] Ordering violated: {frac_50:.1%}, {frac_hypo_60:.1%}, {frac_70:.1%}")
        total_failed += 1

    # --- Check 10: All concentrations positive ---
    print("\n--- Check 10: All model concentrations > 0 ---")
    all_pos = bool(np.all(t_curve > 0))
    baseline["all_positive"] = all_pos
    if all_pos:
        print(f"  [PASS] All T values positive (exponential decay)")
        total_passed += 1
    else:
        print(f"  [FAIL] Non-positive values found")
        total_failed += 1

    # --- Write baseline JSON ---
    baseline_path = os.path.join(SCRIPT_DIR, "exp032_baseline.json")
    baseline_out = {
        "_source": "healthSpring Exp032: Age-Related Testosterone Decline",
        "_method": "Exponential decay + lognormal population variability",
        "params": {
            "T0_mean_ngdl": T0_MEAN_NGDL,
            "T0_cv": T0_CV,
            "decline_rates": [DECLINE_RATE_LOW, DECLINE_RATE_MID, DECLINE_RATE_HIGH],
            "threshold_clinical": THRESHOLD_CLINICAL,
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
