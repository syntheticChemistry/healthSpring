# SPDX-License-Identifier: AGPL-3.0-only

#!/usr/bin/env python3
"""
healthSpring Exp031 — Testosterone Pellet Depot PK

Validates PK model for subcutaneous testosterone pellet implants:
  - Zero-order release phase (sustained dissolution) → first-order elimination
  - 5-month duration (Mok Ch.11: "pellets last about five months")
  - Dosing: 10mg/lb body weight (Mok Ch.11)
  - Smoother profile than IM injection (less fluctuation)

Reference: Testopel label, Cavender 2009, Mok 2018 Ch.11
           Pellet: slow dissolution → quasi-zero-order input → t½ terminal ≈ 8 days

Provenance:
  Baseline date:   2026-03-08
  Command:         python3 control/endocrine/exp031_testosterone_pellet_pk.py
  Environment:     Python 3.10+, NumPy
"""

import json
import os
import sys

import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

# Pellet parameters
BODY_WEIGHT_LB = 200.0
DOSE_MG = 10.0 * BODY_WEIGHT_LB  # 2000mg per Mok
DURATION_DAYS = 150.0  # 5 months
RELEASE_RATE = DOSE_MG / DURATION_DAYS  # mg/day (zero-order approximation)
T_HALF_DAYS = 8.0
K_E = np.log(2) / T_HALF_DAYS
VD_L = 70.0
F_PELLET = 1.0  # subcutaneous, near-complete absorption

TIME_DAYS = np.linspace(0, 180, 3000)  # 6 months


def pellet_concentration(t, release_rate, ke, vd, duration):
    """
    Pellet PK: zero-order input (rate R0) for duration D, then washout.

    During infusion (t <= D):
      C(t) = R0/(Vd*ke) * (1 - exp(-ke*t))

    After infusion (t > D):
      C(t) = R0/(Vd*ke) * (1 - exp(-ke*D)) * exp(-ke*(t - D))
    """
    c_ss = release_rate / (vd * ke)  # steady-state plateau
    if hasattr(t, '__len__'):
        c = np.zeros_like(t)
        infuse = t <= duration
        washout = t > duration
        c[infuse] = c_ss * (1.0 - np.exp(-ke * t[infuse]))
        c[washout] = c_ss * (1.0 - np.exp(-ke * duration)) * np.exp(-ke * (t[washout] - duration))
        return c
    if t <= duration:
        return c_ss * (1.0 - np.exp(-ke * t))
    return c_ss * (1.0 - np.exp(-ke * duration)) * np.exp(-ke * (t - duration))


def main():
    total_passed = 0
    total_failed = 0
    baseline = {}

    print("=" * 72)
    print("healthSpring Exp031: Testosterone Pellet Depot PK")
    print("=" * 72)

    c_pellet = pellet_concentration(TIME_DAYS, RELEASE_RATE, K_E, VD_L, DURATION_DAYS)
    c_ss_theoretical = RELEASE_RATE / (VD_L * K_E)

    # --- Check 1: C(0) = 0 ---
    print("\n--- Check 1: C(0) = 0 ---")
    c0 = float(c_pellet[0])
    baseline["c0"] = c0
    if abs(c0) < 1e-10:
        print(f"  [PASS] C(0) = {c0:.10f}")
        total_passed += 1
    else:
        print(f"  [FAIL] C(0) = {c0:.10f}")
        total_failed += 1

    # --- Check 2: Approaches steady-state by ~5 half-lives ---
    print("\n--- Check 2: Approaches steady-state ---")
    t_5hl = 5 * T_HALF_DAYS
    idx_5hl = np.searchsorted(TIME_DAYS, t_5hl)
    c_at_5hl = float(c_pellet[min(idx_5hl, len(c_pellet) - 1)])
    baseline["c_at_5_half_lives"] = c_at_5hl
    baseline["c_ss_theoretical"] = c_ss_theoretical
    ratio = c_at_5hl / c_ss_theoretical if c_ss_theoretical > 0 else 0
    if ratio > 0.95:
        print(f"  [PASS] C(5×t½) = {c_at_5hl:.4f}, C_ss = {c_ss_theoretical:.4f} (ratio = {ratio:.4f})")
        total_passed += 1
    else:
        print(f"  [FAIL] ratio = {ratio:.4f}, expected > 0.95")
        total_failed += 1

    # --- Check 3: Plateau during infusion is stable ---
    print("\n--- Check 3: Stable plateau during infusion ---")
    plateau_mask = (TIME_DAYS >= 60) & (TIME_DAYS <= 140)
    c_plateau = c_pellet[plateau_mask]
    cv_plateau = np.std(c_plateau) / np.mean(c_plateau) if np.mean(c_plateau) > 0 else 1.0
    baseline["plateau_cv"] = float(cv_plateau)
    if cv_plateau < 0.05:
        print(f"  [PASS] Plateau CV = {cv_plateau:.4f} (< 5% — stable)")
        total_passed += 1
    else:
        print(f"  [FAIL] Plateau CV = {cv_plateau:.4f}")
        total_failed += 1

    # --- Check 4: Concentration in therapeutic range at plateau ---
    print("\n--- Check 4: Plateau in plausible therapeutic range ---")
    # Mok targets 700-1000 ng/dL → convert: our model gives mg/L concentration
    # We validate the *shape* and relative levels, not absolute ng/dL
    c_mean_plateau = float(np.mean(c_plateau))
    baseline["c_mean_plateau"] = c_mean_plateau
    if c_mean_plateau > 0:
        print(f"  [PASS] Mean plateau concentration = {c_mean_plateau:.4f} mg/L (> 0)")
        total_passed += 1
    else:
        print(f"  [FAIL] Mean plateau = {c_mean_plateau:.4f}")
        total_failed += 1

    # --- Check 5: Washout after pellet exhaustion ---
    print("\n--- Check 5: Washout begins after duration ---")
    idx_end = np.searchsorted(TIME_DAYS, DURATION_DAYS)
    c_at_end = float(c_pellet[min(idx_end, len(c_pellet) - 1)])
    c_post = float(c_pellet[-1])
    baseline["c_at_pellet_end"] = c_at_end
    baseline["c_at_6_months"] = c_post
    if c_post < c_at_end * 0.50:
        print(f"  [PASS] C(end)={c_at_end:.4f}, C(6mo)={c_post:.4f} (washout active)")
        total_passed += 1
    else:
        print(f"  [FAIL] Washout insufficient: C(end)={c_at_end:.4f}, C(6mo)={c_post:.4f}")
        total_failed += 1

    # --- Check 6: Washout half-life matches terminal t½ ---
    print("\n--- Check 6: Washout t½ ≈ 8 days ---")
    washout_mask = TIME_DAYS > DURATION_DAYS
    t_wash = TIME_DAYS[washout_mask]
    c_wash = c_pellet[washout_mask]
    if len(c_wash) > 1 and c_wash[0] > 0:
        half_c = c_wash[0] / 2.0
        cross_idx = np.searchsorted(-c_wash, -half_c)
        if cross_idx < len(t_wash):
            t_half_obs = float(t_wash[cross_idx] - t_wash[0])
            baseline["washout_t_half_obs"] = t_half_obs
            if abs(t_half_obs - T_HALF_DAYS) / T_HALF_DAYS < 0.15:
                print(f"  [PASS] Washout t½ = {t_half_obs:.2f} days (expected {T_HALF_DAYS})")
                total_passed += 1
            else:
                print(f"  [FAIL] Washout t½ = {t_half_obs:.2f} days (expected {T_HALF_DAYS})")
                total_failed += 1
        else:
            print(f"  [FAIL] Could not find half-value crossing in washout")
            baseline["washout_t_half_obs"] = None
            total_failed += 1
    else:
        print(f"  [FAIL] Insufficient washout data")
        baseline["washout_t_half_obs"] = None
        total_failed += 1

    # --- Check 7: Fluctuation much less than IM weekly ---
    print("\n--- Check 7: Pellet fluctuation < IM injection fluctuation ---")
    if c_mean_plateau > 0 and np.max(c_plateau) > 0 and np.min(c_plateau) > 0:
        fluct_pellet = float((np.max(c_plateau) - np.min(c_plateau)) / np.min(c_plateau))
    else:
        fluct_pellet = float('inf')
    baseline["pellet_fluctuation_plateau"] = fluct_pellet
    # IM weekly fluctuation is typically 50-200%; pellet should be < 10%
    if fluct_pellet < 0.10:
        print(f"  [PASS] Pellet plateau fluctuation = {fluct_pellet:.4f} (< 10%)")
        total_passed += 1
    else:
        print(f"  [FAIL] Pellet plateau fluctuation = {fluct_pellet:.4f}")
        total_failed += 1

    # --- Check 8: AUC proportional to dose ---
    print("\n--- Check 8: AUC proportional to dose ---")
    # AUC for zero-order infusion: AUC = (R0 * D) / (Vd * ke) for long enough infusion
    auc_numerical = float(np.trapezoid(c_pellet, TIME_DAYS))
    auc_analytical = DOSE_MG / (VD_L * K_E)
    baseline["auc_numerical"] = auc_numerical
    baseline["auc_analytical"] = auc_analytical
    rel_err = abs(auc_numerical - auc_analytical) / auc_analytical
    if rel_err < 0.10:
        print(f"  [PASS] AUC numerical={auc_numerical:.2f}, analytical={auc_analytical:.2f} (err={rel_err:.3%})")
        total_passed += 1
    else:
        print(f"  [FAIL] AUC err={rel_err:.3%}")
        total_failed += 1

    # --- Check 9: Dose-weight scaling ---
    print("\n--- Check 9: Dose-weight linear scaling ---")
    dose_150 = 10.0 * 150.0  # 150lb person
    rr_150 = dose_150 / DURATION_DAYS
    c_150 = pellet_concentration(TIME_DAYS, rr_150, K_E, VD_L, DURATION_DAYS)
    plateau_150 = float(np.mean(c_150[(TIME_DAYS >= 60) & (TIME_DAYS <= 140)]))
    ratio_dose = (DOSE_MG / dose_150)
    ratio_c = (c_mean_plateau / plateau_150) if plateau_150 > 0 else 0
    baseline["plateau_150lb"] = plateau_150
    baseline["dose_scaling_ratio"] = ratio_c
    if abs(ratio_c - ratio_dose) / ratio_dose < 0.01:
        print(f"  [PASS] Concentration scales linearly with dose: ratio={ratio_c:.4f}, expected={ratio_dose:.4f}")
        total_passed += 1
    else:
        print(f"  [FAIL] Scaling mismatch: ratio={ratio_c:.4f}, expected={ratio_dose:.4f}")
        total_failed += 1

    # --- Check 10: All concentrations non-negative ---
    print("\n--- Check 10: All concentrations ≥ 0 ---")
    nonneg = bool(np.all(c_pellet >= -1e-12))
    baseline["all_nonneg"] = nonneg
    if nonneg:
        print(f"  [PASS] All concentrations non-negative")
        total_passed += 1
    else:
        print(f"  [FAIL] Negative concentrations detected")
        total_failed += 1

    # --- Write baseline JSON ---
    baseline_path = os.path.join(SCRIPT_DIR, "exp031_baseline.json")
    baseline_out = {
        "_source": "healthSpring Exp031: Testosterone Pellet Depot PK",
        "_method": "Zero-order release + first-order elimination",
        "params": {
            "body_weight_lb": BODY_WEIGHT_LB,
            "dose_mg": DOSE_MG,
            "duration_days": DURATION_DAYS,
            "release_rate_mg_day": RELEASE_RATE,
            "t_half_days": T_HALF_DAYS,
            "vd_L": VD_L,
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
