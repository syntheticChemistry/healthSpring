#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Exp093: ChEMBL JAK inhibitor selectivity panel (DD-004)

Implements published IC50 panels for 4 compounds, Hill dose-response at C=IC50,
pathway_selectivity_score for JAK1 target, and MATRIX scoring with shared
tissue params.

Reference: Hill AV (1910) J Physiol 40:iv; Fajgenbaum DC et al. NEJM 379:1941 (2018)
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


def selectivity_index(ic50_on: float, ic50_off: float) -> float:
    """SI = ic50_off / ic50_on."""
    if ic50_on <= 0:
        return 0.0
    return ic50_off / ic50_on


def hill_dose_response(conc: float, ic50: float, n: float, e_max: float) -> float:
    """R = E_max * C^n / (IC50^n + C^n). At C=IC50 → 0.5."""
    if ic50 <= 0 or conc < 0:
        return 0.0
    c_n = conc**n
    ic50_n = ic50**n
    return e_max * c_n / (ic50_n + c_n)


def main() -> None:
    # Published IC50 panels (nM): JAK1, JAK2, JAK3, TYK2
    canine_ocla = {"name": "oclacitinib", "jak1": 10.0, "jak2": 1000.0, "jak3": 10000.0, "tyk2": 10000.0}
    human_panels = [
        {"name": "tofacitinib", "jak1": 3.2, "jak2": 4.1, "jak3": 1.6, "tyk2": 34.0},
        {"name": "ruxolitinib", "jak1": 3.3, "jak2": 2.8, "jak3": 428.0, "tyk2": 19.0},
        {"name": "baricitinib", "jak1": 5.9, "jak2": 5.7, "jak3": 560.0, "tyk2": 53.0},
    ]

    localization_length = 10.0
    tissue_thickness = 1.0
    w_baseline = 5.0
    w_treated = 5.0

    all_panels = [canine_ocla] + human_panels
    compounds = []
    for p in all_panels:
        ic50_on = p["jak1"]
        ic50_off = [p["jak2"], p["jak3"], p["tyk2"]]
        pathway = pathway_selectivity_score(ic50_on, ic50_off)
        geometry = tissue_geometry_factor(localization_length, tissue_thickness)
        disorder = disorder_impact_factor(w_baseline, w_treated)
        combined = pathway * geometry * disorder

        hill_at_ic50 = hill_dose_response(ic50_on, ic50_on, 1.0, 1.0)
        si_jak1_jak2 = selectivity_index(p["jak1"], p["jak2"])

        compounds.append({
            "compound": p["name"],
            "ic50_jak1_nm": p["jak1"],
            "ic50_jak2_nm": p["jak2"],
            "ic50_jak3_nm": p["jak3"],
            "ic50_tyk2_nm": p["tyk2"],
            "selectivity_index_jak1_jak2": si_jak1_jak2,
            "hill_at_c_equals_ic50": hill_at_ic50,
            "pathway_score": pathway,
            "tissue_geometry": geometry,
            "disorder_factor": disorder,
            "matrix_combined_score": combined,
        })

    compounds.sort(key=lambda x: x["matrix_combined_score"], reverse=True)

    out = {
        "experiment": "exp093_chembl_jak_panel",
        "disease": "AD",
        "target": "JAK1",
        "localization_length": localization_length,
        "tissue_thickness": tissue_thickness,
        "compounds": compounds,
        "_provenance": {
            "date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
            "script": "control/discovery/exp093_chembl_jak_panel.py",
            "command": " ".join(sys.argv) or f"python3 {__file__}",
            "git_commit": "baseline",
            "python_version": sys.version.split()[0],
            "numpy_version": np.__version__,
        },
    }

    print(json.dumps(out, indent=2))


if __name__ == "__main__":
    main()
