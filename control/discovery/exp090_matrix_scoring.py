#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Exp090: Anderson-augmented MATRIX drug repurposing scoring (Fajgenbaum 2018)

Implements pathway selectivity, tissue geometry, disorder impact, and combined
MATRIX scoring for a panel of JAK inhibitors in atopic dermatitis.

Reference: Fajgenbaum DC et al. NEJM 379:1941 (2018)
"""

import json
import math
import sys
from datetime import datetime, timezone

import numpy as np


def pathway_selectivity_score(ic50_on: float, ic50_off_list: list[float]) -> float:
    """geomean(off) / (geomean(off) + on). Returns 0 if no off-targets."""
    if not ic50_off_list:
        return 0.0
    log_sum = sum(math.log(x) for x in ic50_off_list)
    geomean = math.exp(log_sum / len(ic50_off_list))
    return geomean / (geomean + ic50_on)


def tissue_geometry_factor(xi: float, L: float) -> float:
    """1 - exp(-xi/L). Returns 0 if L<=0 or xi<=0."""
    if L <= 0 or xi <= 0:
        return 0.0
    return 1.0 - math.exp(-xi / L)


def disorder_impact_factor(w_base: float, w_treat: float) -> float:
    """clamp(1 + (w_treat - w_base)/w_base, 0, 2)."""
    if w_base <= 0:
        return 1.0
    raw = 1.0 + (w_treat - w_base) / w_base
    return max(0.0, min(2.0, raw))


def matrix_combined_score(pathway: float, geometry: float, disorder: float) -> float:
    """combined = pathway × geometry × disorder"""
    return pathway * geometry * disorder


def score_compound(
    name: str,
    ic50_on: float,
    ic50_off: list[float],
    localization_length: float,
    tissue_thickness: float,
    w_baseline: float,
    w_treated: float,
) -> dict:
    """Score a compound against AD (JAK1 target)."""
    pathway = pathway_selectivity_score(ic50_on, ic50_off)
    geometry = tissue_geometry_factor(localization_length, tissue_thickness)
    disorder = disorder_impact_factor(w_baseline, w_treated)
    combined = matrix_combined_score(pathway, geometry, disorder)
    return {
        "compound": name,
        "pathway_score": pathway,
        "tissue_geometry": geometry,
        "disorder_factor": disorder,
        "combined_score": combined,
    }


def main() -> None:
    # JAK inhibitor panel (IC50 nM): JAK1, JAK2, JAK3, TYK2 — AD targets JAK1
    panel = [
        ("Oclacitinib", 10.0, [1000.0, 10000.0, 10000.0]),
        ("Tofacitinib", 3.2, [4.1, 1.6, 34.0]),
        ("Ruxolitinib", 3.3, [2.8, 428.0, 19.0]),
        ("Baricitinib", 5.9, [5.7, 560.0, 53.0]),
    ]

    tissue_thickness = 1.0
    localization_length = 10.0
    w_baseline = 5.0
    w_treated = 5.0

    scores = []
    for name, ic50_on, ic50_off in panel:
        entry = score_compound(
            name, ic50_on, ic50_off,
            localization_length, tissue_thickness,
            w_baseline, w_treated,
        )
        scores.append(entry)

    # Sort by combined score descending (highest first)
    scores.sort(key=lambda x: x["combined_score"], reverse=True)

    out = {
        "experiment": "exp090_matrix_scoring",
        "tissue_thickness": tissue_thickness,
        "localization_length": localization_length,
        "w_baseline": w_baseline,
        "w_treated": w_treated,
        "compounds": scores,
        "_provenance": {
            "date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
            "script": "control/discovery/exp090_matrix_scoring.py",
            "command": " ".join(sys.argv) or f"python3 {__file__}",
            "git_commit": "baseline",
            "python_version": sys.version.split()[0],
            "numpy_version": np.__version__,
        },
    }

    print(json.dumps(out, indent=2))


if __name__ == "__main__":
    main()
