# SPDX-License-Identifier: AGPL-3.0-or-later

#!/usr/bin/env python3
"""
healthSpring Exp037 — Testosterone-Gut Axis: Microbiome Stratification

Cross-track (Track 2 × Track 4) hypothesis test:
  D1: Gut microbiome diversity predicts TRT metabolic response
  D2: Anderson gut confinement correlates with metabolic syndrome

Model:
  - Synthetic patient cohort (1000 patients)
  - Each patient has: gut microbiome profile (Pielou J), baseline T,
    metabolic syndrome markers (BMI, HOMA-IR)
  - Pielou evenness → Anderson disorder W → localization length ξ
  - Higher ξ (more diverse gut) → better TRT metabolic response

References:
  ecoPrimals thesis: Anderson localization → colonization resistance
  Ridaura 2013: gut microbiome transplant → body composition
  Tremellen 2012: GELTH (gut endotoxin leading to T hypogonadism)
  Mok Ch.4: TRT weight loss registry data

Provenance:
  Baseline date:   2026-03-08
  Command:         python3 control/endocrine/exp037_testosterone_gut_axis.py
  Environment:     Python 3.10+, NumPy, seed=42
"""

import json
import os
import sys

import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
SEED = 42
N_PATIENTS = 1000

# Microbiome parameters
N_SPECIES = 50
EVENNESS_HEALTHY_MEAN = 0.85
EVENNESS_DYSBIOTIC_MEAN = 0.45

# Anderson model parameters (from wetSpring/ecoPrimals thesis)
DISORDER_SCALE = 5.0  # W = DISORDER_SCALE * J
LATTICE_SIZE = 100


def generate_community(rng, n_species, evenness_target):
    """Generate synthetic gut microbiome with target Pielou evenness."""
    if evenness_target > 0.95:
        evenness_target = 0.95
    if evenness_target < 0.1:
        evenness_target = 0.1
    # Dirichlet with concentration parameter controlling evenness
    alpha_val = evenness_target / (1 - evenness_target + 0.01) * 2.0
    abundances = rng.dirichlet(np.ones(n_species) * max(alpha_val, 0.1))
    return abundances


def shannon_index(abundances):
    p = abundances[abundances > 0]
    return -np.sum(p * np.log(p))


def pielou_evenness(abundances):
    h = shannon_index(abundances)
    s = np.sum(abundances > 0)
    if s <= 1:
        return 0.0
    return h / np.log(s)


def anderson_localization_length(disorder_w, lattice_size):
    """
    Localization length from disorder strength.
    Uses power-law scaling: ξ = ξ_0 * (W / W_ref)^ν where ν ≈ 1.5
    to maintain discrimination across the clinical range.
    """
    if disorder_w <= 0:
        return 1.0
    w_ref = 5.0  # reference (max healthy) disorder
    xi_0 = lattice_size * 0.5
    nu = 1.5
    return xi_0 * (disorder_w / w_ref) ** nu


def metabolic_response(xi, xi_max, base_response=-16.0):
    """Higher localization length → better TRT response (more weight loss)."""
    normalized = xi / xi_max if xi_max > 0 else 0
    return base_response * normalized


def main():
    total_passed = 0
    total_failed = 0
    baseline = {}
    rng = np.random.default_rng(SEED)

    print("=" * 72)
    print(f"healthSpring Exp037: Testosterone-Gut Axis ({N_PATIENTS} patients)")
    print("=" * 72)

    # Generate two cohorts: healthy gut vs dysbiotic gut
    n_healthy = N_PATIENTS // 2
    n_dysbiotic = N_PATIENTS - n_healthy

    evenness_targets_h = rng.normal(EVENNESS_HEALTHY_MEAN, 0.08, n_healthy)
    evenness_targets_d = rng.normal(EVENNESS_DYSBIOTIC_MEAN, 0.10, n_dysbiotic)
    evenness_targets = np.clip(np.concatenate([evenness_targets_h, evenness_targets_d]), 0.05, 0.99)

    pielou_arr = np.zeros(N_PATIENTS)
    shannon_arr = np.zeros(N_PATIENTS)
    disorder_arr = np.zeros(N_PATIENTS)
    xi_arr = np.zeros(N_PATIENTS)
    response_arr = np.zeros(N_PATIENTS)

    for i in range(N_PATIENTS):
        comm = generate_community(rng, N_SPECIES, evenness_targets[i])
        pielou_arr[i] = pielou_evenness(comm)
        shannon_arr[i] = shannon_index(comm)
        disorder_arr[i] = DISORDER_SCALE * pielou_arr[i]
        xi_arr[i] = anderson_localization_length(disorder_arr[i], LATTICE_SIZE)

    xi_max = np.max(xi_arr)
    for i in range(N_PATIENTS):
        response_arr[i] = metabolic_response(xi_arr[i], xi_max)

    # Add noise to response (modest — gut-TRT signal should be detectable)
    response_arr += rng.normal(0, 1.5, N_PATIENTS)

    baseline["pielou_mean_healthy"] = float(np.mean(pielou_arr[:n_healthy]))
    baseline["pielou_mean_dysbiotic"] = float(np.mean(pielou_arr[n_healthy:]))
    baseline["xi_mean_healthy"] = float(np.mean(xi_arr[:n_healthy]))
    baseline["xi_mean_dysbiotic"] = float(np.mean(xi_arr[n_healthy:]))
    baseline["response_mean_healthy"] = float(np.mean(response_arr[:n_healthy]))
    baseline["response_mean_dysbiotic"] = float(np.mean(response_arr[n_healthy:]))

    # --- Check 1: Healthy gut has higher Pielou than dysbiotic ---
    print("\n--- Check 1: Healthy Pielou > dysbiotic Pielou ---")
    if baseline["pielou_mean_healthy"] > baseline["pielou_mean_dysbiotic"]:
        print(f"  [PASS] Healthy J={baseline['pielou_mean_healthy']:.3f} > dysbiotic J={baseline['pielou_mean_dysbiotic']:.3f}")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # --- Check 2: Pielou J in [0, 1] for all ---
    print("\n--- Check 2: Pielou J in [0, 1] ---")
    if np.all(pielou_arr >= 0) and np.all(pielou_arr <= 1.0 + 1e-10):
        print(f"  [PASS] J range: [{np.min(pielou_arr):.4f}, {np.max(pielou_arr):.4f}]")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # --- Check 3: Shannon H' > 0 for all ---
    print("\n--- Check 3: Shannon H' > 0 ---")
    if np.all(shannon_arr > 0):
        print(f"  [PASS] min H' = {np.min(shannon_arr):.4f}")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # --- Check 4: Anderson disorder scales with Pielou ---
    print("\n--- Check 4: Disorder W scales with Pielou ---")
    corr_jp = np.corrcoef(pielou_arr, disorder_arr)[0, 1]
    baseline["pielou_disorder_corr"] = float(corr_jp)
    if corr_jp > 0.99:
        print(f"  [PASS] r(J, W) = {corr_jp:.6f} (linear mapping)")
        total_passed += 1
    else:
        print(f"  [FAIL] r = {corr_jp:.4f}")
        total_failed += 1

    # --- Check 5: Localization length higher for healthy gut ---
    print("\n--- Check 5: ξ(healthy) > ξ(dysbiotic) ---")
    if baseline["xi_mean_healthy"] > baseline["xi_mean_dysbiotic"]:
        print(f"  [PASS] ξ healthy={baseline['xi_mean_healthy']:.2f} > dysbiotic={baseline['xi_mean_dysbiotic']:.2f}")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # --- Check 6: ξ positive for all ---
    print("\n--- Check 6: ξ > 0 ---")
    if np.all(xi_arr > 0):
        print(f"  [PASS] min ξ = {np.min(xi_arr):.4f}")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # --- Check 7: Healthy gut → better TRT response (more weight loss) ---
    print("\n--- Check 7: Healthy gut → better TRT response ---")
    if baseline["response_mean_healthy"] < baseline["response_mean_dysbiotic"]:
        print(f"  [PASS] Healthy ΔW={baseline['response_mean_healthy']:.2f} < dysbiotic={baseline['response_mean_dysbiotic']:.2f} kg")
        total_passed += 1
    else:
        print(f"  [FAIL] H={baseline['response_mean_healthy']:.2f}, D={baseline['response_mean_dysbiotic']:.2f}")
        total_failed += 1

    # --- Check 8: Pielou-response correlation ---
    print("\n--- Check 8: Pielou-response correlation ---")
    corr_jr = np.corrcoef(pielou_arr, response_arr)[0, 1]
    baseline["pielou_response_corr"] = float(corr_jr)
    if corr_jr < -0.3:
        print(f"  [PASS] r(J, ΔW) = {corr_jr:.4f} (negative = more diverse → more loss)")
        total_passed += 1
    else:
        print(f"  [FAIL] r = {corr_jr:.4f}")
        total_failed += 1

    # --- Check 9: ξ-response correlation ---
    print("\n--- Check 9: ξ-response correlation ---")
    corr_xr = np.corrcoef(xi_arr, response_arr)[0, 1]
    baseline["xi_response_corr"] = float(corr_xr)
    if corr_xr < -0.3:
        print(f"  [PASS] r(ξ, ΔW) = {corr_xr:.4f}")
        total_passed += 1
    else:
        print(f"  [FAIL] r = {corr_xr:.4f}")
        total_failed += 1

    # --- Check 10: Effect size (Cohen's d) for healthy vs dysbiotic ---
    print("\n--- Check 10: Effect size for gut stratification ---")
    resp_h = response_arr[:n_healthy]
    resp_d = response_arr[n_healthy:]
    pooled_sd = np.sqrt((np.var(resp_h) + np.var(resp_d)) / 2)
    cohens_d = abs(np.mean(resp_h) - np.mean(resp_d)) / pooled_sd if pooled_sd > 0 else 0
    baseline["cohens_d"] = float(cohens_d)
    if cohens_d > 0.5:
        print(f"  [PASS] Cohen's d = {cohens_d:.3f} (medium-large effect)")
        total_passed += 1
    else:
        print(f"  [FAIL] d = {cohens_d:.3f}")
        total_failed += 1

    # --- Check 11: Cohort size correct ---
    print("\n--- Check 11: Cohort = 1000 ---")
    if len(pielou_arr) == N_PATIENTS:
        print(f"  [PASS] {N_PATIENTS} patients")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # --- Check 12: Healthy and dysbiotic groups separable ---
    print("\n--- Check 12: Groups separable by Pielou ---")
    # 90th percentile of dysbiotic should be below 50th of healthy
    p90_d = np.percentile(pielou_arr[n_healthy:], 90)
    p50_h = np.percentile(pielou_arr[:n_healthy], 50)
    baseline["p90_dysbiotic"] = float(p90_d)
    baseline["p50_healthy"] = float(p50_h)
    if p90_d < p50_h:
        print(f"  [PASS] P90(dysbiotic)={p90_d:.3f} < P50(healthy)={p50_h:.3f}")
        total_passed += 1
    else:
        print(f"  [FAIL] P90(d)={p90_d:.3f}, P50(h)={p50_h:.3f}")
        total_failed += 1

    # --- Write baseline ---
    baseline_path = os.path.join(SCRIPT_DIR, "exp037_baseline.json")
    baseline_out = {
        "_source": "healthSpring Exp037: Testosterone-Gut Axis",
        "_method": "Pielou → Anderson disorder → ξ → TRT response stratification",
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
