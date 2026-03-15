# SPDX-License-Identifier: AGPL-3.0-or-later

#!/usr/bin/env python3
"""
healthSpring Exp010 — Microbiome Diversity Indices

Validates α-diversity metrics for gut microbiome analysis:
  - Shannon diversity index: H' = -Σ p_i * ln(p_i)
  - Simpson diversity index: D = 1 - Σ p_i²
  - Inverse Simpson: 1/D_simple = 1 / Σ p_i²
  - Pielou evenness: J = H' / ln(S)
  - Species richness (observed OTUs)
  - Chao1 estimator for total richness

Connects to:
  - wetSpring Track 1 (16S rRNA pipelines)
  - Anderson localization: Pielou evenness → disorder W mapping
  - C. diff risk: low diversity = high risk

Reference:
  Shannon CE (1948), Simpson EH (1949), Chao A (1984)
  Lozupone & Knight (2005) for gut microbiome application

Provenance:
  Baseline date:   2026-03-08
  Command:         python3 control/microbiome/exp010_diversity_indices.py
  Environment:     Python 3.10+, NumPy
"""

import json
import os
import sys

import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
SEED = 42

# Synthetic gut microbiome communities (relative abundance)
HEALTHY_GUT = {
    "name": "healthy_gut",
    "abundances": [0.25, 0.20, 0.15, 0.12, 0.10, 0.08, 0.05, 0.03, 0.01, 0.01],
    "description": "Diverse healthy adult gut",
}

DYSBIOTIC_GUT = {
    "name": "dysbiotic_gut",
    "abundances": [0.85, 0.05, 0.03, 0.02, 0.02, 0.01, 0.01, 0.005, 0.003, 0.002],
    "description": "Post-antibiotic dysbiosis (single-species dominance)",
}

CDIFF_GUT = {
    "name": "cdiff_colonized",
    "abundances": [0.60, 0.15, 0.10, 0.05, 0.04, 0.03, 0.02, 0.005, 0.003, 0.002],
    "description": "C. difficile colonization (reduced diversity)",
}

PERFECTLY_EVEN = {
    "name": "perfectly_even",
    "abundances": [0.1] * 10,
    "description": "Theoretical maximum evenness (10 species)",
}

SINGLETON = {
    "name": "monoculture",
    "abundances": [1.0],
    "description": "Single species (minimum diversity)",
}

ALL_COMMUNITIES = [HEALTHY_GUT, DYSBIOTIC_GUT, CDIFF_GUT, PERFECTLY_EVEN, SINGLETON]

# Synthetic count data for Chao1 (integers)
COUNTS_HEALTHY = [250, 200, 150, 120, 100, 80, 50, 30, 10, 5, 3, 2, 1, 1, 1]
COUNTS_DEPLETED = [850, 50, 30, 20, 15, 10, 5, 5, 3, 2, 1, 1]


def shannon_index(abundances):
    """Shannon diversity: H' = -Σ p_i * ln(p_i)."""
    return float(-sum(p * np.log(p) for p in abundances if p > 0))


def simpson_index(abundances):
    """Simpson diversity: D = 1 - Σ p_i²."""
    return float(1.0 - sum(p ** 2 for p in abundances))


def inverse_simpson(abundances):
    """Inverse Simpson: 1 / Σ p_i²."""
    d = sum(p ** 2 for p in abundances)
    return float(1.0 / d) if d > 0 else 0.0


def pielou_evenness(abundances):
    """Pielou J = H' / ln(S)."""
    s = len(abundances)
    if s <= 1:
        return 0.0
    h = shannon_index(abundances)
    return float(h / np.log(s))


def chao1(counts):
    """Chao1 richness estimator: S_obs + f1²/(2*f2).

    f1 = singletons (count=1), f2 = doubletons (count=2).
    """
    s_obs = len(counts)
    f1 = sum(1 for c in counts if c == 1)
    f2 = sum(1 for c in counts if c == 2)
    if f2 == 0:
        return float(s_obs + f1 * (f1 - 1) / 2) if f1 > 1 else float(s_obs)
    return float(s_obs + f1 ** 2 / (2 * f2))


def evenness_to_disorder(evenness, w_scale=10.0):
    """Map Pielou evenness to Anderson disorder W."""
    return evenness * w_scale


def main():
    total_passed = 0
    total_failed = 0
    baseline = {}

    print("=" * 72)
    print("healthSpring Exp010: Microbiome Diversity Indices")
    print("=" * 72)

    # ------------------------------------------------------------------
    # Check 1: Shannon — perfectly even = ln(S)
    # ------------------------------------------------------------------
    print("\n--- Check 1: Shannon perfectly even = ln(S) ---")
    h_even = shannon_index(PERFECTLY_EVEN["abundances"])
    expected_h = np.log(len(PERFECTLY_EVEN["abundances"]))
    baseline["shannon_even"] = h_even
    if abs(h_even - expected_h) < 1e-10:
        print(f"  [PASS] H' = {h_even:.10f} = ln({len(PERFECTLY_EVEN['abundances'])}) = {expected_h:.10f}")
        total_passed += 1
    else:
        print(f"  [FAIL] H' = {h_even:.10f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 2: Shannon — monoculture = 0
    # ------------------------------------------------------------------
    print("\n--- Check 2: Shannon monoculture = 0 ---")
    h_mono = shannon_index(SINGLETON["abundances"])
    baseline["shannon_mono"] = h_mono
    if abs(h_mono) < 1e-10:
        print(f"  [PASS] H' = {h_mono:.10f}")
        total_passed += 1
    else:
        print(f"  [FAIL] H' = {h_mono:.10f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 3: Shannon — healthy > dysbiotic > c.diff > mono
    # ------------------------------------------------------------------
    print("\n--- Check 3: Shannon ordering ---")
    h_vals = {c["name"]: shannon_index(c["abundances"]) for c in ALL_COMMUNITIES}
    baseline["shannon_all"] = h_vals
    ordered = (h_vals["perfectly_even"] > h_vals["healthy_gut"] >
               h_vals["cdiff_colonized"] > h_vals["dysbiotic_gut"] >
               h_vals["monoculture"])
    if ordered:
        order_str = " > ".join(f"{n}({v:.3f})" for n, v in
                                sorted(h_vals.items(), key=lambda x: x[1], reverse=True))
        print(f"  [PASS] {order_str}")
        total_passed += 1
    else:
        print(f"  [FAIL] {h_vals}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 4: Simpson — perfectly even
    # ------------------------------------------------------------------
    print("\n--- Check 4: Simpson perfectly even ---")
    d_even = simpson_index(PERFECTLY_EVEN["abundances"])
    expected_d = 1.0 - len(PERFECTLY_EVEN["abundances"]) * (0.1 ** 2)
    baseline["simpson_even"] = d_even
    if abs(d_even - expected_d) < 1e-10:
        print(f"  [PASS] D = {d_even:.10f}")
        total_passed += 1
    else:
        print(f"  [FAIL] D = {d_even:.10f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 5: Simpson — monoculture = 0
    # ------------------------------------------------------------------
    print("\n--- Check 5: Simpson monoculture = 0 ---")
    d_mono = simpson_index(SINGLETON["abundances"])
    baseline["simpson_mono"] = d_mono
    if abs(d_mono) < 1e-10:
        print(f"  [PASS] D = {d_mono:.10f}")
        total_passed += 1
    else:
        print(f"  [FAIL] D = {d_mono:.10f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 6: Simpson — healthy > dysbiotic
    # ------------------------------------------------------------------
    print("\n--- Check 6: Simpson ordering ---")
    d_healthy = simpson_index(HEALTHY_GUT["abundances"])
    d_dysbiotic = simpson_index(DYSBIOTIC_GUT["abundances"])
    baseline["simpson_healthy"] = d_healthy
    baseline["simpson_dysbiotic"] = d_dysbiotic
    if d_healthy > d_dysbiotic:
        print(f"  [PASS] healthy {d_healthy:.4f} > dysbiotic {d_dysbiotic:.4f}")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 7: Inverse Simpson — even = S
    # ------------------------------------------------------------------
    print("\n--- Check 7: Inverse Simpson even = S ---")
    inv_even = inverse_simpson(PERFECTLY_EVEN["abundances"])
    baseline["inv_simpson_even"] = inv_even
    if abs(inv_even - 10.0) < 1e-10:
        print(f"  [PASS] 1/D = {inv_even:.10f} = S=10")
        total_passed += 1
    else:
        print(f"  [FAIL] 1/D = {inv_even:.10f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 8: Pielou — even = 1.0
    # ------------------------------------------------------------------
    print("\n--- Check 8: Pielou evenness even = 1.0 ---")
    j_even = pielou_evenness(PERFECTLY_EVEN["abundances"])
    baseline["pielou_even"] = j_even
    if abs(j_even - 1.0) < 1e-10:
        print(f"  [PASS] J = {j_even:.10f}")
        total_passed += 1
    else:
        print(f"  [FAIL] J = {j_even:.10f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 9: Pielou — ordering matches Shannon
    # ------------------------------------------------------------------
    print("\n--- Check 9: Pielou ordering ---")
    j_healthy = pielou_evenness(HEALTHY_GUT["abundances"])
    j_dysbiotic = pielou_evenness(DYSBIOTIC_GUT["abundances"])
    j_cdiff = pielou_evenness(CDIFF_GUT["abundances"])
    baseline["pielou_healthy"] = j_healthy
    baseline["pielou_dysbiotic"] = j_dysbiotic
    baseline["pielou_cdiff"] = j_cdiff
    if j_healthy > j_cdiff > j_dysbiotic:
        print(f"  [PASS] healthy({j_healthy:.3f}) > cdiff({j_cdiff:.3f}) > dysbiotic({j_dysbiotic:.3f})")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 10: Chao1 ≥ S_obs
    # ------------------------------------------------------------------
    print("\n--- Check 10: Chao1 ≥ observed richness ---")
    chao1_h = chao1(COUNTS_HEALTHY)
    chao1_d = chao1(COUNTS_DEPLETED)
    baseline["chao1_healthy"] = chao1_h
    baseline["chao1_depleted"] = chao1_d
    if chao1_h >= len(COUNTS_HEALTHY) and chao1_d >= len(COUNTS_DEPLETED):
        print(f"  [PASS] Healthy: Chao1={chao1_h:.1f} ≥ S_obs={len(COUNTS_HEALTHY)}; "
              f"Depleted: Chao1={chao1_d:.1f} ≥ S_obs={len(COUNTS_DEPLETED)}")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 11: Chao1 healthy > Chao1 depleted
    # ------------------------------------------------------------------
    print("\n--- Check 11: Chao1 healthy > depleted ---")
    if chao1_h > chao1_d:
        print(f"  [PASS] {chao1_h:.1f} > {chao1_d:.1f}")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 12: Anderson disorder mapping — more diverse = more disorder
    # ------------------------------------------------------------------
    print("\n--- Check 12: Pielou → Anderson disorder W ---")
    w_healthy = evenness_to_disorder(j_healthy)
    w_dysbiotic = evenness_to_disorder(j_dysbiotic)
    baseline["anderson_w_healthy"] = w_healthy
    baseline["anderson_w_dysbiotic"] = w_dysbiotic
    if w_healthy > w_dysbiotic:
        print(f"  [PASS] W(healthy)={w_healthy:.3f} > W(dysbiotic)={w_dysbiotic:.3f}")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 13: All indices in valid range
    # ------------------------------------------------------------------
    print("\n--- Check 13: All indices in valid ranges ---")
    all_valid = True
    for comm in ALL_COMMUNITIES:
        ab = comm["abundances"]
        h = shannon_index(ab)
        d = simpson_index(ab)
        j = pielou_evenness(ab)
        if h < -1e-10:
            all_valid = False
        if not (-1e-10 <= d <= 1.0 + 1e-10):
            all_valid = False
        if len(ab) > 1 and not (-1e-10 <= j <= 1.0 + 1e-10):
            all_valid = False
    baseline["all_ranges_valid"] = all_valid
    if all_valid:
        print(f"  [PASS] H' ≥ 0, 0 ≤ D ≤ 1, 0 ≤ J ≤ 1 for all communities")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 14: Abundances sum to 1.0 (input validation)
    # ------------------------------------------------------------------
    print("\n--- Check 14: Abundance normalization ---")
    all_normalized = all(abs(sum(c["abundances"]) - 1.0) < 1e-6 for c in ALL_COMMUNITIES)
    baseline["all_normalized"] = all_normalized
    if all_normalized:
        print(f"  [PASS] All communities sum to 1.0")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Write baseline
    # ------------------------------------------------------------------
    baseline_path = os.path.join(SCRIPT_DIR, "exp010_baseline.json")
    baseline_out = {
        "_source": "healthSpring Exp010: Microbiome Diversity Indices",
        "_method": "Shannon, Simpson, Pielou, Chao1",
        "communities": {c["name"]: c for c in ALL_COMMUNITIES},
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
