#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""
Rho/MRTF/SRF fibrosis scoring baseline.

Reference: Evelyn et al., J Biol Chem (2011) 286:28097-28110.
CCG-1423 and CCG-203971 inhibit Rho/MRTF/SRF signaling in fibrosis.
"""

import json
import sys
from datetime import datetime

import numpy as np


def fractional_inhibition(conc: float, ic50: float) -> float:
    """Fractional inhibition = conc / (ic50 + conc)."""
    return conc / (ic50 + conc) if (ic50 + conc) > 0 else 0.0


def anti_fibrotic_score(rho: float, mrtf: float, srf: float) -> float:
    """Weighted anti-fibrotic score: 0.2*rho + 0.5*mrtf + 0.3*srf."""
    return 0.2 * rho + 0.5 * mrtf + 0.3 * srf


def fibrotic_geometry_factor(xi: float, L: float) -> float:
    """Geometry factor: exp(-xi/L)."""
    return np.exp(-xi / L)


def main() -> None:
    concentrations = [0.0, 1.0, 5.0, 10.0, 50.0]
    compounds = {
        "CCG-1423": {"rho_ic50": 10.0, "mrtf_ic50": 3.2, "srf_ic50": 5.0},
        "CCG-203971": {"rho_ic50": 15.0, "mrtf_ic50": 1.5, "srf_ic50": 2.8},
    }
    xi, L, disorder = 1.0, 2.0, 1.0

    results = []
    for name, ics in compounds.items():
        for conc in concentrations:
            rho = fractional_inhibition(conc, ics["rho_ic50"])
            mrtf = fractional_inhibition(conc, ics["mrtf_ic50"])
            srf = fractional_inhibition(conc, ics["srf_ic50"])
            anti = anti_fibrotic_score(rho, mrtf, srf)
            geometry = fibrotic_geometry_factor(xi, L)
            fibrosis_matrix_score = anti * geometry * disorder
            results.append({
                "compound": name,
                "conc_uM": conc,
                "rho_inhibition": float(rho),
                "mrtf_inhibition": float(mrtf),
                "srf_inhibition": float(srf),
                "anti_fibrotic_score": float(anti),
                "geometry_factor": float(geometry),
                "fibrosis_matrix_score": float(fibrosis_matrix_score),
            })

    out = {
        "results": results,
        "parameters": {"xi": xi, "L": L, "disorder": disorder},
        "_provenance": {
            "date": datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            "script": "control/discovery/exp094_rho_mrtf_fibrosis.py",
            "command": " ".join(sys.argv),
            "git_commit": "baseline",
            "python": sys.version.split()[0],
            "numpy": np.__version__,
        },
    }
    print(json.dumps(out, indent=2))


if __name__ == "__main__":
    main()
