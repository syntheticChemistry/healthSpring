#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Exp097: Affinity landscape modeling for low-affinity binding regime.

Implements fractional occupancy, composite binding score, Gini coefficient,
and low-affinity selectivity. Reference: affinity_landscape.rs
"""

import json
import os
import subprocess
import sys
from datetime import datetime, timezone

import numpy as np


def fractional_occupancy(concentration: float, ic50: float, hill_n: float) -> float:
    """Hill fractional occupancy: C^n / (IC50^n + C^n)."""
    if ic50 <= 0 or concentration < 0:
        return 0.0
    c_n = concentration**hill_n
    ic50_n = ic50**hill_n
    return c_n / (ic50_n + c_n)


def composite_binding_score(occupancies: list[float]) -> float:
    """Composite binding: 1 - product(1 - f_i)."""
    if not occupancies:
        return 0.0
    product_complement = 1.0
    for f in occupancies:
        product_complement *= 1.0 - max(0.0, min(1.0, f))
    return 1.0 - product_complement


def compute_gini(values: list[float]) -> float:
    """Gini coefficient (0 = uniform, 1 = maximally unequal)."""
    n = len(values)
    if n < 2:
        return 0.0
    sorted_vals = sorted(values)
    total = sum(sorted_vals)
    if total < 1e-15:
        return 0.0
    numerator = sum((2 * (i + 1) - n - 1) * v for i, v in enumerate(sorted_vals))
    return max(0.0, min(1.0, numerator / (n * total)))


def low_affinity_selectivity(
    target_occupancies: list[float], nontarget_occupancies: list[float]
) -> float:
    """Selectivity = composite_target / composite_nontarget."""
    s_target = composite_binding_score(target_occupancies)
    s_nontarget = composite_binding_score(nontarget_occupancies)
    if s_nontarget < 1e-15:
        return float("inf") if s_target >= 1e-15 else 1.0
    return s_target / s_nontarget


def main() -> None:
    script_dir = os.path.dirname(os.path.abspath(__file__))

    # Cancer composite: 15 markers at IC50=20, conc=1.0, n=1.0
    cancer_ic50s = [20.0] * 15
    cancer_occupancies = [
        fractional_occupancy(1.0, ic50, 1.0) for ic50 in cancer_ic50s
    ]
    cancer_composite = composite_binding_score(cancer_occupancies)

    # Normal composite: 4 markers at IC50=200, conc=1.0, n=1.0
    normal_ic50s = [200.0] * 4
    normal_occupancies = [
        fractional_occupancy(1.0, ic50, 1.0) for ic50 in normal_ic50s
    ]
    normal_composite = composite_binding_score(normal_occupancies)

    # Low-affinity selectivity
    sel = low_affinity_selectivity(cancer_occupancies, normal_occupancies)

    # Individual occupancy at conc=1, IC50=20, n=1
    individual_occupancy = fractional_occupancy(1.0, 20.0, 1.0)

    # Gini: uniform [0.0196]*20 → 0.0
    uniform_gini = compute_gini([0.0196] * 20)

    # Gini: skewed [0.909, 0.002, 0.002, ...] → > 0.5
    skewed_vals = [0.909] + [0.002] * 19
    skewed_gini = compute_gini(skewed_vals)

    try:
        git_commit = subprocess.check_output(
            ["git", "rev-parse", "HEAD"], text=True
        ).strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        git_commit = "unknown"

    out = {
        "experiment": "exp097_affinity_landscape",
        "fractional_occupancy": {
            "formula": "C^n / (IC50^n + C^n)",
            "individual_occupancy": individual_occupancy,
            "params": {"conc": 1.0, "ic50": 20.0, "n": 1.0},
        },
        "composite_binding_score": {
            "formula": "1 - product(1 - f_i)",
            "cancer_composite": cancer_composite,
            "cancer_params": {"n_markers": 15, "ic50": 20.0, "conc": 1.0, "n": 1.0},
            "cancer_occupancies": cancer_occupancies,
            "normal_composite": normal_composite,
            "normal_params": {"n_markers": 4, "ic50": 200.0, "conc": 1.0, "n": 1.0},
            "normal_occupancies": normal_occupancies,
        },
        "low_affinity_selectivity": sel,
        "gini": {
            "uniform": uniform_gini,
            "uniform_values": [0.0196] * 20,
            "skewed": skewed_gini,
            "skewed_values": skewed_vals,
        },
        "_provenance": {
            "date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
            "script": "control/discovery/exp097_affinity_landscape.py",
            "command": " ".join(sys.argv) or f"python3 {__file__}",
            "git_commit": git_commit,
            "python_version": sys.version.split()[0],
            "numpy_version": np.__version__,
        },
    }

    print(json.dumps(out, indent=2))

    baseline_path = os.path.join(script_dir, "exp097_baseline.json")
    with open(baseline_path, "w") as f:
        json.dump(out, f, indent=2)
    print(f"\nBaseline written to {baseline_path}", file=sys.stderr)


if __name__ == "__main__":
    main()
