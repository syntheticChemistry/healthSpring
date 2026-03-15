#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""
IL-31 pruritus time-course baseline.

Reference: Gonzales et al., Vet Dermatol (2013) 24:23-e7.
IL-31 in canine atopic dermatitis; oclacitinib and lokivetmab effects.
"""

import json
import sys
from datetime import datetime

import numpy as np


def il31_t(t: np.ndarray, C_ss: float, baseline: float, k_el: float) -> np.ndarray:
    """IL-31 concentration: C_ss + (baseline - C_ss) * exp(-k_el * t)."""
    return C_ss + (baseline - C_ss) * np.exp(-k_el * t)


def vas(C: np.ndarray) -> np.ndarray:
    """VAS = 10 * C^2 / (25^2 + C^2)."""
    return 10.0 * C**2 / (25.0**2 + C**2)


def main() -> None:
    baseline = 44.5
    k_el_base = 0.05
    k_prod_base = baseline * 0.05

    treatments = {
        "untreated": {"k_prod": k_prod_base, "k_el": k_el_base},
        "oclacitinib": {"k_prod": k_prod_base * 0.30, "k_el": k_el_base},
        "lokivetmab": {"k_prod": k_prod_base, "k_el": k_el_base * 5.0},
    }

    t = np.linspace(0, 500, 100)
    results = []

    for name, params in treatments.items():
        C_ss = params["k_prod"] / params["k_el"]
        C = il31_t(t, C_ss, baseline, params["k_el"])
        V = vas(C)
        results.append({
            "treatment": name,
            "k_prod": params["k_prod"],
            "k_el": params["k_el"],
            "C_ss": float(C_ss),
            "time_hr": t.tolist(),
            "il31_conc": C.tolist(),
            "VAS": V.tolist(),
        })

    out = {
        "results": results,
        "parameters": {"baseline": baseline, "k_el_base": k_el_base},
        "_provenance": {
            "date": datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            "script": "control/comparative/exp102_il31_pruritus_timecourse.py",
            "command": " ".join(sys.argv),
            "git_commit": "baseline",
            "python": sys.version.split()[0],
            "numpy": np.__version__,
        },
    }
    print(json.dumps(out, indent=2))


if __name__ == "__main__":
    main()
