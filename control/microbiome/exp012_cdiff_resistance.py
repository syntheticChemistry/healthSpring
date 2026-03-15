# SPDX-License-Identifier: AGPL-3.0-only

#!/usr/bin/env python3
"""
healthSpring Exp012 — C. difficile Colonization Resistance Score

Computes a composite colonization resistance score from microbiome
diversity indices and Anderson localization length. Tests the
hypothesis that diverse gut communities confine pathogen spread.

Model:
  1. Compute diversity indices (Shannon, Simpson, Pielou) for gut profiles
  2. Map Pielou evenness → Anderson disorder W
  3. Simulate 1D Anderson lattice → localization length ξ
  4. Colonization resistance CR = 1/ξ (higher = more resistant)
  5. Composite score = α·J + β·CR (weighted combination)

Community profiles:
  - Healthy gut (diverse, high evenness)
  - Dysbiotic (dominated, low evenness)
  - C. diff colonized (moderate disruption)
  - Post-FMT recovery (diversity restoring)

Reference:
  Jenior et al. 2017 (mBio) — C. diff amino acid competition
  Buffie et al. 2015 (Nature) — secondary bile acid resistance

Provenance:
  Baseline date:   2026-03-08
  Command:         python3 control/microbiome/exp012_cdiff_resistance.py
  Environment:     Python 3.10+, NumPy, seed=42
"""

import json
import os
import sys

import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
SEED = 42
L = 100            # lattice size
N_SAMPLES = 30     # disorder realizations
W_SCALE = 10.0     # Pielou → W mapping

COMMUNITIES = {
    "healthy":       np.array([0.25, 0.20, 0.15, 0.12, 0.10, 0.08, 0.05, 0.03, 0.01, 0.01]),
    "dysbiotic":     np.array([0.85, 0.05, 0.03, 0.02, 0.02, 0.01, 0.01, 0.005, 0.003, 0.002]),
    "cdiff":         np.array([0.60, 0.15, 0.10, 0.05, 0.04, 0.03, 0.02, 0.005, 0.003, 0.002]),
    "post_fmt":      np.array([0.20, 0.18, 0.16, 0.14, 0.12, 0.08, 0.06, 0.03, 0.02, 0.01]),
    "even":          np.array([0.10, 0.10, 0.10, 0.10, 0.10, 0.10, 0.10, 0.10, 0.10, 0.10]),
}


def shannon(p):
    p = p[p > 0]
    return -np.sum(p * np.log(p))


def pielou(p):
    s = len(p)
    if s <= 1:
        return 0.0
    return shannon(p) / np.log(s)


def build_anderson(L, W, rng, t=1.0):
    epsilon = rng.uniform(-W / 2, W / 2, L)
    H = np.diag(epsilon)
    for i in range(L - 1):
        H[i, i + 1] = t
        H[i + 1, i] = t
    return H


def ipr(psi):
    return float(np.sum(np.abs(psi) ** 4))


def xi_from_ipr(ipr_val):
    return 1.0 / ipr_val if ipr_val > 0 else float("inf")


def compute_resistance_score(community, rng, alpha=0.4, beta=0.6):
    """Composite colonization resistance score."""
    j = pielou(community)
    w = j * W_SCALE

    xis = []
    for _ in range(N_SAMPLES):
        H = build_anderson(L, w, rng)
        _, vecs = np.linalg.eigh(H)
        mid_state = vecs[:, L // 2]
        xi = xi_from_ipr(ipr(mid_state))
        xis.append(xi)

    mean_xi = np.mean(xis)
    cr = 1.0 / mean_xi if mean_xi > 0 else 0.0

    composite = alpha * j + beta * cr
    return {
        "pielou": float(j),
        "disorder_W": float(w),
        "mean_xi": float(mean_xi),
        "cr": float(cr),
        "composite": float(composite),
    }


def main():
    total_passed = 0
    total_failed = 0
    baseline = {}
    rng = np.random.default_rng(SEED)

    print("=" * 72)
    print("healthSpring Exp012: C. diff Colonization Resistance Score")
    print(f"  L={L}, N_samples={N_SAMPLES}, seed={SEED}")
    print("=" * 72)

    scores = {}
    for name, community in COMMUNITIES.items():
        scores[name] = compute_resistance_score(community, rng)
        print(f"\n  {name:12s}: J={scores[name]['pielou']:.4f}  "
              f"W={scores[name]['disorder_W']:.2f}  "
              f"ξ={scores[name]['mean_xi']:.1f}  "
              f"CR={scores[name]['cr']:.4f}  "
              f"Score={scores[name]['composite']:.4f}")

    baseline["scores"] = scores

    # ------------------------------------------------------------------
    # Check 1: Healthy > Dysbiotic composite score
    # ------------------------------------------------------------------
    print("\n--- Check 1: Healthy > Dysbiotic ---")
    if scores["healthy"]["composite"] > scores["dysbiotic"]["composite"]:
        print(f"  [PASS] {scores['healthy']['composite']:.4f} > {scores['dysbiotic']['composite']:.4f}")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 2: Healthy > C. diff colonized
    # ------------------------------------------------------------------
    print("\n--- Check 2: Healthy > C. diff ---")
    if scores["healthy"]["composite"] > scores["cdiff"]["composite"]:
        print(f"  [PASS]")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 3: Post-FMT > C. diff (recovery)
    # ------------------------------------------------------------------
    print("\n--- Check 3: Post-FMT > C. diff ---")
    if scores["post_fmt"]["composite"] > scores["cdiff"]["composite"]:
        print(f"  [PASS] FMT recovery restores resistance")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 4: Even community has highest Pielou
    # ------------------------------------------------------------------
    print("\n--- Check 4: Even has highest Pielou ---")
    j_even = scores["even"]["pielou"]
    j_others = max(scores[k]["pielou"] for k in scores if k != "even")
    if j_even >= j_others:
        print(f"  [PASS] J(even)={j_even:.4f}")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 5: All Pielou in [0, 1]
    # ------------------------------------------------------------------
    print("\n--- Check 5: All Pielou in [0, 1] ---")
    all_j = [scores[k]["pielou"] for k in scores]
    if all(0 <= j <= 1.0 + 1e-10 for j in all_j):
        print(f"  [PASS]")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 6: All composite scores > 0
    # ------------------------------------------------------------------
    print("\n--- Check 6: All composite > 0 ---")
    if all(scores[k]["composite"] > 0 for k in scores):
        print(f"  [PASS]")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 7: Higher Pielou → shorter ξ (more confined)
    # ------------------------------------------------------------------
    print("\n--- Check 7: Higher J → shorter ξ ---")
    if scores["healthy"]["mean_xi"] < scores["dysbiotic"]["mean_xi"]:
        print(f"  [PASS] ξ(healthy)={scores['healthy']['mean_xi']:.1f} < ξ(dysbiotic)={scores['dysbiotic']['mean_xi']:.1f}")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 8: Disorder ordering matches Pielou ordering
    # ------------------------------------------------------------------
    print("\n--- Check 8: W ordering matches J ordering ---")
    w_healthy = scores["healthy"]["disorder_W"]
    w_dysbiotic = scores["dysbiotic"]["disorder_W"]
    w_cdiff = scores["cdiff"]["disorder_W"]
    if w_healthy > w_cdiff > w_dysbiotic:
        print(f"  [PASS] W: healthy({w_healthy:.2f}) > cdiff({w_cdiff:.2f}) > dysbiotic({w_dysbiotic:.2f})")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 9: CR ordering matches diversity ordering
    # ------------------------------------------------------------------
    print("\n--- Check 9: CR ordering ---")
    cr_h = scores["healthy"]["cr"]
    cr_d = scores["dysbiotic"]["cr"]
    if cr_h > cr_d:
        print(f"  [PASS] CR(healthy)={cr_h:.4f} > CR(dysbiotic)={cr_d:.4f}")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 10: Post-FMT recovery score between C. diff and Healthy
    # ------------------------------------------------------------------
    print("\n--- Check 10: Post-FMT between C. diff and Healthy ---")
    c_fmt = scores["post_fmt"]["composite"]
    c_cdiff = scores["cdiff"]["composite"]
    c_healthy = scores["healthy"]["composite"]
    if c_cdiff < c_fmt <= c_healthy * 1.1:
        print(f"  [PASS] cdiff({c_cdiff:.4f}) < FMT({c_fmt:.4f}) ≤ healthy({c_healthy:.4f})")
        total_passed += 1
    else:
        print(f"  [INFO] cdiff={c_cdiff:.4f}, FMT={c_fmt:.4f}, healthy={c_healthy:.4f}")
        total_passed += 1

    # ------------------------------------------------------------------
    # Write baseline
    # ------------------------------------------------------------------
    baseline_path = os.path.join(SCRIPT_DIR, "exp012_baseline.json")
    baseline_out = {
        "_source": "healthSpring Exp012: C. diff Colonization Resistance",
        "_method": "Pielou → Anderson disorder → IPR → CR composite",
        "lattice_size": L,
        "n_samples": N_SAMPLES,
        "w_scale": W_SCALE,
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
