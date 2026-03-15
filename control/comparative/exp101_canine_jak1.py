#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Exp101: Canine oclacitinib JAK1 selectivity validation (Gonzales 2014, CM-002)

Implements JAK1 selectivity = (JAK2*JAK3*TYK2)^(1/3) / JAK1, Hill dose-response
at C=IC50 → 0.5, and pathway_selectivity_score for each compound.

Reference: Gonzales AJ et al. JVPT 37:317 (2014)
"""

import json
import math
import sys
from datetime import datetime, timezone

import numpy as np


def jak1_selectivity(jak1: float, jak2: float, jak3: float, tyk2: float) -> float:
    """(JAK2*JAK3*TYK2)^(1/3) / JAK1"""
    off_geomean = (jak2 * jak3 * tyk2) ** (1.0 / 3.0)
    return off_geomean / jak1


def selectivity_index(ic50_on: float, ic50_off: float) -> float:
    """SI = ic50_off / ic50_on."""
    if ic50_on <= 0:
        return 0.0
    return ic50_off / ic50_on


def pathway_selectivity_score(ic50_on: float, ic50_off_list: list[float]) -> float:
    """geomean(off) / (geomean(off) + on). Returns 0 if no off-targets."""
    if not ic50_off_list:
        return 0.0
    log_sum = sum(math.log(x) for x in ic50_off_list)
    geomean = math.exp(log_sum / len(ic50_off_list))
    return geomean / (geomean + ic50_on)


def hill_dose_response(conc: float, ic50: float, n: float, e_max: float) -> float:
    """R = E_max * C^n / (IC50^n + C^n). At C=IC50 → 0.5."""
    if ic50 <= 0 or conc < 0:
        return 0.0
    c_n = conc**n
    ic50_n = ic50**n
    return e_max * c_n / (ic50_n + c_n)


def main() -> None:
    oclacitinib = {"name": "oclacitinib", "jak1": 10.0, "jak2": 1000.0, "jak3": 10000.0, "tyk2": 10000.0}
    human_panels = [
        {"name": "tofacitinib", "jak1": 3.2, "jak2": 4.1, "jak3": 1.6, "tyk2": 34.0},
        {"name": "ruxolitinib", "jak1": 3.3, "jak2": 2.8, "jak3": 428.0, "tyk2": 19.0},
        {"name": "baricitinib", "jak1": 5.9, "jak2": 5.7, "jak3": 560.0, "tyk2": 53.0},
    ]

    all_panels = [oclacitinib] + human_panels
    compounds = []
    for p in all_panels:
        sel = jak1_selectivity(p["jak1"], p["jak2"], p["jak3"], p["tyk2"])
        hill_at_ic50 = hill_dose_response(p["jak1"], p["jak1"], 1.0, 1.0)
        pathway = pathway_selectivity_score(p["jak1"], [p["jak2"], p["jak3"], p["tyk2"]])
        compounds.append({
            "compound": p["name"],
            "ic50_jak1_nm": p["jak1"],
            "ic50_jak2_nm": p["jak2"],
            "ic50_jak3_nm": p["jak3"],
            "ic50_tyk2_nm": p["tyk2"],
            "jak1_selectivity": sel,
            "hill_at_c_equals_ic50": hill_at_ic50,
            "pathway_selectivity_score": pathway,
        })

    out = {
        "experiment": "exp101_canine_jak1",
        "compounds": compounds,
        "selectivity_index_10_1000": selectivity_index(10.0, 1000.0),
        "_provenance": {
            "date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
            "script": "control/comparative/exp101_canine_jak1.py",
            "command": " ".join(sys.argv) or f"python3 {__file__}",
            "git_commit": "baseline",
            "python_version": sys.version.split()[0],
            "numpy_version": np.__version__,
        },
    }

    print(json.dumps(out, indent=2))


if __name__ == "__main__":
    main()
