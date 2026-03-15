# SPDX-License-Identifier: AGPL-3.0-only

#!/usr/bin/env python3
"""
healthSpring Exp038 — HRV × TRT Cardiovascular Cross-Track (Mok D3)

Cross-track experiment linking biosignal HRV metrics to TRT cardiovascular outcomes.
Hypothesis: improved HRV (higher SDNN) correlates with TRT cardiovascular benefit
(reduced hazard ratio).

Models:
  - hrv_trt_response: SDNN(months) = sdnn_base + delta_sdnn * (1 - exp(-months/tau))
  - cardiac_risk_composite: Risk = baseline_risk * hrv_factor * testosterone_factor

Reference: Kleiger 1987 (NEJM) — SDNN < 50ms ~5× cardiac mortality risk
           Sharma 2015 — TRT cardiovascular benefit

Provenance:
  Command:         python3 control/endocrine/exp038_hrv_trt_cardiovascular.py
  Environment:     Python 3.10+, NumPy
"""

import json
import os
import sys

import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

# HRV × TRT parameters (baseline SDNN=35ms, delta=20ms, tau=6mo)
SDNN_BASELINE_MS = 35.0
DELTA_SDNN_MS = 20.0
TAU_MONTHS = 6.0
TOTAL_MONTHS = 24.0

# Pre-TRT vs post-TRT states
PRE_TRT_SDNN_MS = 35.0
PRE_TRT_T_NGDL = 250.0
POST_TRT_SDNN_MS = 55.0  # 35 + 20 after full response
POST_TRT_T_NGDL = 500.0
BASELINE_RISK = 1.0


def hrv_trt_response(sdnn_base_ms, delta_sdnn_ms, tau_months, months):
    """SDNN trajectory under TRT."""
    return sdnn_base_ms + delta_sdnn_ms * (1.0 - np.exp(-months / tau_months))


def cardiac_risk_composite(sdnn_ms, testosterone_ng_dl, baseline_risk):
    """Composite cardiac risk from HRV + testosterone."""
    if sdnn_ms < 50.0:
        hrv_factor = 2.0 - sdnn_ms / 50.0
    elif sdnn_ms > 100.0:
        hrv_factor = 0.5
    else:
        hrv_factor = 1.0 - 0.5 * (sdnn_ms - 50.0) / 50.0

    if testosterone_ng_dl < 300.0:
        t_factor = 2.0 - testosterone_ng_dl / 300.0
    elif testosterone_ng_dl > 500.0:
        t_factor = 0.5
    else:
        t_factor = 1.0 - 0.5 * (testosterone_ng_dl - 300.0) / 200.0

    return baseline_risk * hrv_factor * t_factor


def main():
    total_passed = 0
    total_failed = 0
    baseline = {}

    print("=" * 72)
    print("healthSpring Exp038: HRV × TRT Cardiovascular (Mok D3)")
    print("=" * 72)

    # --- HRV trajectory for 24 months ---
    months_arr = np.arange(0, 25, dtype=float)
    sdnn_trajectory = hrv_trt_response(
        SDNN_BASELINE_MS, DELTA_SDNN_MS, TAU_MONTHS, months_arr
    )

    # --- Check 1: HRV at t=0 equals baseline SDNN ---
    print("\n--- Check 1: HRV at t=0 equals baseline SDNN ---")
    sdnn_0 = float(sdnn_trajectory[0])
    baseline["sdnn_at_0"] = sdnn_0
    if abs(sdnn_0 - SDNN_BASELINE_MS) < 1e-10:
        print(f"  [PASS] SDNN(0) = {sdnn_0:.1f} ms")
        total_passed += 1
    else:
        print(f"  [FAIL] SDNN(0) = {sdnn_0:.1f}, expected {SDNN_BASELINE_MS}")
        total_failed += 1

    # --- Check 2: HRV improves monotonically ---
    print("\n--- Check 2: HRV improves monotonically ---")
    monotonic = all(
        sdnn_trajectory[i] <= sdnn_trajectory[i + 1] + 1e-12
        for i in range(len(sdnn_trajectory) - 1)
    )
    baseline["sdnn_monotonic"] = monotonic
    if monotonic:
        print("  [PASS] SDNN increases monotonically")
        total_passed += 1
    else:
        print("  [FAIL] Non-monotonic SDNN trajectory")
        total_failed += 1

    # --- Check 3: HRV approaches base + delta asymptotically ---
    print("\n--- Check 3: HRV approaches base + delta asymptotically ---")
    sdnn_24 = float(sdnn_trajectory[-1])
    sdnn_asymptote = SDNN_BASELINE_MS + DELTA_SDNN_MS
    baseline["sdnn_at_24mo"] = sdnn_24
    baseline["sdnn_asymptote"] = sdnn_asymptote
    if abs(sdnn_24 - sdnn_asymptote) < 1.0:
        print(f"  [PASS] SDNN(24) = {sdnn_24:.2f} ≈ {sdnn_asymptote:.0f} ms")
        total_passed += 1
    else:
        print(f"  [FAIL] SDNN(24) = {sdnn_24:.2f}, asymptote = {sdnn_asymptote}")
        total_failed += 1

    # --- Cardiac risk: pre-TRT vs post-TRT ---
    risk_pre = cardiac_risk_composite(PRE_TRT_SDNN_MS, PRE_TRT_T_NGDL, BASELINE_RISK)
    risk_post = cardiac_risk_composite(POST_TRT_SDNN_MS, POST_TRT_T_NGDL, BASELINE_RISK)

    # --- Check 4: Cardiac risk decreases with TRT ---
    print("\n--- Check 4: Cardiac risk decreases with TRT ---")
    baseline["risk_pre_trt"] = float(risk_pre)
    baseline["risk_post_trt"] = float(risk_post)
    if risk_pre > risk_post:
        print(f"  [PASS] Risk: {risk_pre:.3f} → {risk_post:.3f}")
        total_passed += 1
    else:
        print(f"  [FAIL] Risk: pre={risk_pre:.3f}, post={risk_post:.3f}")
        total_failed += 1

    # --- Check 5: Risk reduction ratio ---
    print("\n--- Check 5: Risk reduction ratio ---")
    risk_reduction = (risk_pre - risk_post) / risk_pre
    baseline["risk_reduction_ratio"] = float(risk_reduction)
    if risk_reduction > 0.5:
        print(f"  [PASS] Risk reduction = {risk_reduction:.1%} (> 50%)")
        total_passed += 1
    else:
        print(f"  [FAIL] Risk reduction = {risk_reduction:.1%}")
        total_failed += 1

    # --- Check 6: Low SDNN (<50ms) → risk factor > 1.0 ---
    print("\n--- Check 6: Low SDNN (<50ms) → risk factor > 1.0 ---")
    risk_low_sdnn = cardiac_risk_composite(30.0, 400.0, 1.0)
    risk_high_sdnn = cardiac_risk_composite(120.0, 400.0, 1.0)
    baseline["risk_low_sdnn"] = float(risk_low_sdnn)
    baseline["risk_high_sdnn"] = float(risk_high_sdnn)
    if risk_low_sdnn > risk_high_sdnn:
        print(f"  [PASS] Low SDNN risk={risk_low_sdnn:.2f} > high SDNN risk={risk_high_sdnn:.2f}")
        total_passed += 1
    else:
        print("  [FAIL] Low SDNN should yield higher risk")
        total_failed += 1

    # --- Write baseline JSON ---
    baseline_path = os.path.join(SCRIPT_DIR, "exp038_baseline.json")
    baseline_out = {
        "_source": "healthSpring Exp038: HRV × TRT Cardiovascular (Mok D3)",
        "_method": "hrv_trt_response + cardiac_risk_composite",
        "params": {
            "sdnn_base_ms": SDNN_BASELINE_MS,
            "delta_sdnn_ms": DELTA_SDNN_MS,
            "tau_months": TAU_MONTHS,
            "total_months": TOTAL_MONTHS,
            "pre_trt": {"sdnn_ms": PRE_TRT_SDNN_MS, "t_ngdl": PRE_TRT_T_NGDL},
            "post_trt": {"sdnn_ms": POST_TRT_SDNN_MS, "t_ngdl": POST_TRT_T_NGDL},
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
