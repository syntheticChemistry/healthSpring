#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Exp098: Toxicology and cytotoxicity landscape modeling.

Implements TissueToxProfile, systemic burden, excess burden, toxicity IPR,
localization length (xi), and clearance utilization.
Reference: toxicology.rs
"""

import json
import os
import subprocess
import sys
from datetime import datetime, timezone

import numpy as np


def systemic_burden_score(tissues: list[dict]) -> float:
    """Weighted sum: sum(occupancy_i * sensitivity_i)."""
    return sum(t["occupancy"] * t["sensitivity"] for t in tissues)


def tissue_excess_burden(tissues: list[dict]) -> tuple[list[float], float]:
    """Excess = max(0, occ*sens - repair_capacity) per tissue."""
    excesses = []
    for t in tissues:
        burden = t["occupancy"] * t["sensitivity"]
        excess = max(0.0, burden - t["repair_capacity"])
        excesses.append(excess)
    return excesses, sum(excesses)


def toxicity_ipr(tissues: list[dict]) -> float:
    """IPR = sum(p_i^4) where p_i = (occ_i * sens_i) / total."""
    weights = [t["occupancy"] * t["sensitivity"] for t in tissues]
    total = sum(weights)
    if total < 1e-15:
        return 0.0
    return sum((w / total) ** 4 for w in weights)


def toxicity_localization_length(tissues: list[dict]) -> float:
    """xi = 1 / IPR (effective number of tissues sharing burden)."""
    ipr = toxicity_ipr(tissues)
    if ipr < 1e-15:
        return float("inf")
    return 1.0 / ipr


def clearance_utilization(concentration: float, km: float) -> float:
    """C / (Km + C) — Michaelis-Menten saturation fraction."""
    if concentration < 0:
        return 0.0
    return concentration / (km + concentration)


def main() -> None:
    script_dir = os.path.dirname(os.path.abspath(__file__))

    # Strong binder: liver at occ=0.85 sens=1.0, rest at 0.01-0.02
    strong_tissues = [
        {"name": "liver", "occupancy": 0.85, "sensitivity": 1.0, "repair_capacity": 0.05},
        {"name": "heart", "occupancy": 0.02, "sensitivity": 1.0, "repair_capacity": 0.05},
        {"name": "kidney", "occupancy": 0.01, "sensitivity": 1.0, "repair_capacity": 0.05},
        {"name": "lung", "occupancy": 0.01, "sensitivity": 1.0, "repair_capacity": 0.05},
        {"name": "gut", "occupancy": 0.02, "sensitivity": 1.0, "repair_capacity": 0.05},
        {"name": "brain", "occupancy": 0.01, "sensitivity": 1.0, "repair_capacity": 0.05},
        {"name": "muscle", "occupancy": 0.01, "sensitivity": 1.0, "repair_capacity": 0.05},
        {"name": "skin", "occupancy": 0.02, "sensitivity": 1.0, "repair_capacity": 0.05},
    ]
    strong_systemic = systemic_burden_score(strong_tissues)
    strong_excesses, strong_excess_total = tissue_excess_burden(strong_tissues)
    strong_ipr = toxicity_ipr(strong_tissues)
    strong_xi = toxicity_localization_length(strong_tissues)

    # Weak binder: all tissues at occ=0.02-0.05
    weak_tissues = [
        {"name": f"tissue_{i}", "occupancy": 0.02 + 0.03 * (i % 3) / 2, "sensitivity": 1.0, "repair_capacity": 0.05}
        for i in range(10)
    ]
    weak_systemic = systemic_burden_score(weak_tissues)
    weak_excesses, weak_excess_total = tissue_excess_burden(weak_tissues)
    weak_ipr = toxicity_ipr(weak_tissues)
    weak_xi = toxicity_localization_length(weak_tissues)

    # Clearance utilization at C=0.03, Km=10
    clearance_util = clearance_utilization(0.03, 10.0)

    try:
        git_commit = subprocess.check_output(
            ["git", "rev-parse", "HEAD"], text=True
        ).strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        git_commit = "unknown"

    out = {
        "experiment": "exp098_toxicity_landscape",
        "strong_binder": {
            "description": "Liver at occ=0.85 sens=1.0, rest at 0.01-0.02",
            "tissues": strong_tissues,
            "systemic_burden": strong_systemic,
            "excess_burden": strong_excess_total,
            "tissue_excesses": strong_excesses,
            "toxicity_ipr": strong_ipr,
            "localization_length_xi": strong_xi,
        },
        "weak_binder": {
            "description": "All tissues at occ=0.02-0.05",
            "tissues": weak_tissues,
            "systemic_burden": weak_systemic,
            "excess_burden": weak_excess_total,
            "tissue_excesses": weak_excesses,
            "toxicity_ipr": weak_ipr,
            "localization_length_xi": weak_xi,
        },
        "clearance_utilization": {
            "value": clearance_util,
            "params": {"C": 0.03, "Km": 10.0},
            "formula": "C / (Km + C)",
        },
        "_provenance": {
            "date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
            "script": "control/toxicology/exp098_toxicity_landscape.py",
            "command": " ".join(sys.argv) or f"python3 {__file__}",
            "git_commit": git_commit,
            "python_version": sys.version.split()[0],
            "numpy_version": np.__version__,
        },
    }

    print(json.dumps(out, indent=2))

    baseline_path = os.path.join(script_dir, "exp098_baseline.json")
    with open(baseline_path, "w") as f:
        json.dump(out, f, indent=2)
    print(f"\nBaseline written to {baseline_path}", file=sys.stderr)


if __name__ == "__main__":
    main()
