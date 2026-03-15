#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Exp092: ADDRC compound library batch IC50 profiling

Implements selectivity index, IC50 estimation from Hill curves via linear
interpolation at 50% response, and ranking for a synthetic 5-compound panel.

Reference: Hill AV (1910) J Physiol 40:iv
"""

import json
import math
import sys
from datetime import datetime, timezone

import numpy as np


def selectivity_index(ic50_on: float, ic50_off: float) -> float:
    """SI = ic50_off / ic50_on. Returns 0 if ic50_on <= 0."""
    if ic50_on <= 0:
        return 0.0
    return ic50_off / ic50_on


def hill_response(conc: float, ic50: float, n: float, emax: float = 1.0) -> float:
    """R = Emax * C^n / (IC50^n + C^n)"""
    c_n = conc**n
    ic50_n = ic50**n
    return emax * c_n / (ic50_n + c_n)


def estimate_ic50(concentrations: list[float], responses: list[float]) -> float:
    """Estimate IC50 by linear interpolation at 50% response."""
    if len(concentrations) < 2 or len(concentrations) != len(responses):
        return float("nan")


    emax = max(responses)
    half_max = emax * 0.5

    for i in range(len(concentrations) - 1):
        c0, c1 = concentrations[i], concentrations[i + 1]
        r0, r1 = responses[i], responses[i + 1]
        if (r0 <= half_max <= r1) or (r0 >= half_max >= r1):
            if abs(r1 - r0) < 1e-15:
                frac = 0.5
            else:
                frac = (half_max - r0) / (r1 - r0)
            return c0 + frac * (c1 - c0)
    return float("nan")


def main() -> None:
    # 5 synthetic compounds: (name, JAK1_ic50, JAK2_ic50)
    compounds = [
        ("Compound A", 5.0, 500.0),
        ("Compound B", 50.0, 60.0),
        ("Compound C", 2.0, 2000.0),
        ("Compound D", 100.0, 200.0),
        ("Compound E", 10.0, 1000.0),
    ]

    n = 1.0
    emax = 1.0
    # 8-point dose-response: 0.1 * 10^(0.5*i) for i=0..7
    concentrations = [0.1 * (10.0 ** (0.5 * i)) for i in range(8)]

    scorecards = []
    for name, jak1_ic50, jak2_ic50 in compounds:
        responses = [
            hill_response(c, jak1_ic50, n, emax) for c in concentrations
        ]
        est_ic50 = estimate_ic50(concentrations, responses)
        si = selectivity_index(jak1_ic50, jak2_ic50)
        scorecards.append({
            "compound": name,
            "ic50_jak1_nm": jak1_ic50,
            "ic50_jak2_nm": jak2_ic50,
            "ic50_estimate_from_curve": est_ic50,
            "selectivity_index": si,
        })

    # Rank by selectivity (descending)
    scorecards.sort(key=lambda x: x["selectivity_index"], reverse=True)
    for rank, sc in enumerate(scorecards, 1):
        sc["rank"] = rank

    out = {
        "experiment": "exp092_compound_library",
        "targets": ["JAK1", "JAK2"],
        "compounds": scorecards,
        "_provenance": {
            "date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
            "script": "control/discovery/exp092_compound_library.py",
            "command": " ".join(sys.argv) or f"python3 {__file__}",
            "git_commit": "baseline",
            "python_version": sys.version.split()[0],
            "numpy_version": np.__version__,
        },
    }

    print(json.dumps(out, indent=2))


if __name__ == "__main__":
    main()
