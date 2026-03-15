#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Exp100: Canine IL-31 serum kinetics in atopic dermatitis (Gonzales 2013, CM-001)

Implements IL-31 kinetics: dC/dt = k_prod - k_el*C, solved analytically as
C(t) = C_ss + (C0 - C_ss) * exp(-k_el * t). Also pruritus VAS response.

Reference: Gonzales AJ et al. Vet Dermatol 24:48 (2013)
"""

import json
import math
import sys
from datetime import datetime, timezone

import numpy as np


def il31_serum_kinetics(
    baseline_pg_ml: float,
    t_hr: float,
    treatment: str,
) -> float:
    """
    C(t) = C_ss + (C0 - C_ss) * exp(-k_el * t)
    Untreated: k_prod = baseline*0.05, k_el = 0.05 → C_ss = baseline
    Oclacitinib: k_prod reduced to 30% → C_ss = 0.3*baseline
    Lokivetmab: k_el increased 5× → C_ss = baseline/5
    """
    k_el_baseline = 0.05
    k_prod_baseline = baseline_pg_ml * k_el_baseline

    if treatment == "Untreated":
        k_prod, k_el = k_prod_baseline, k_el_baseline
    elif treatment == "Oclacitinib":
        k_prod, k_el = k_prod_baseline * 0.30, k_el_baseline
    elif treatment == "Lokivetmab":
        k_prod, k_el = k_prod_baseline, k_el_baseline * 5.0
    else:
        raise ValueError(f"Unknown treatment: {treatment}")

    c_ss = k_prod / k_el
    return c_ss + (baseline_pg_ml - c_ss) * math.exp(-k_el * t_hr)


def pruritus_vas(il31_pg_ml: float) -> float:
    """VAS = 10 * C^2 / (25^2 + C^2) — Hill with EC50=25, n=2, VAS_max=10."""
    vas_max = 10.0
    ec50 = 25.0
    n = 2.0
    return vas_max * (il31_pg_ml**n) / (ec50**n + il31_pg_ml**n)


def main() -> None:
    baseline = 44.5  # pg/mL
    time_points = [0.0, 24.0, 48.0, 100.0, 200.0, 500.0, 1000.0]
    treatments = ["Untreated", "Oclacitinib", "Lokivetmab"]

    time_courses = {}
    for treatment in treatments:
        concs = [il31_serum_kinetics(baseline, t, treatment) for t in time_points]
        vas_vals = [pruritus_vas(c) for c in concs]
        time_courses[treatment] = {
            "time_hr": time_points,
            "il31_pg_ml": concs,
            "pruritus_vas": vas_vals,
        }

    out = {
        "experiment": "exp100_canine_il31",
        "baseline_pg_ml": baseline,
        "k_el_baseline": 0.05,
        "time_courses": time_courses,
        "_provenance": {
            "date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
            "script": "control/comparative/exp100_canine_il31.py",
            "command": " ".join(sys.argv) or f"python3 {__file__}",
            "git_commit": "baseline",
            "python_version": sys.version.split()[0],
            "numpy_version": np.__version__,
        },
    }

    print(json.dumps(out, indent=2))


if __name__ == "__main__":
    main()
