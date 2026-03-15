#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Exp104: Cross-species allometric PK scaling (CM-005)

Implements allometric scaling: CL_target = CL_ref*(BW_target/BW_ref)^0.75,
Vd_target = Vd_ref*(BW_target/BW_ref)^1.0, t_half = ln(2)*Vd/CL.
Scales canine → human and verifies roundtrip identity.

Reference: Mahmood I (2006) J Pharm Sci 95:1810
"""

import json
import math
import sys
from datetime import datetime, timezone

import numpy as np

LN2 = math.log(2.0)


def allometric_clearance(cl_ref: float, bw_ref: float, bw_target: float) -> float:
    """CL_target = CL_ref * (BW_target/BW_ref)^0.75"""
    return cl_ref * (bw_target / bw_ref) ** 0.75


def allometric_volume(vd_ref: float, bw_ref: float, bw_target: float) -> float:
    """Vd_target = Vd_ref * (BW_target/BW_ref)^1.0"""
    return vd_ref * (bw_target / bw_ref)


def allometric_half_life(vd: float, cl: float) -> float:
    """t_half = ln(2) * Vd / CL. Returns inf if CL <= 0."""
    if cl <= 0:
        return float("inf")
    return LN2 * vd / cl


def scale_to_species(
    cl_per_kg: float,
    vd_per_kg: float,
    bw_ref: float,
    bw_target: float,
) -> tuple[float, float]:
    """
    Scale PK from reference to target species.
    CL and Vd are per-kg; we scale total CL and Vd then divide by target BW.
    """
    cl_total_ref = cl_per_kg * bw_ref
    vd_total_ref = vd_per_kg * bw_ref

    cl_total_target = allometric_clearance(cl_total_ref, bw_ref, bw_target)
    vd_total_target = allometric_volume(vd_total_ref, bw_ref, bw_target)

    cl_per_kg_target = cl_total_target / bw_target
    vd_per_kg_target = vd_total_target / bw_target

    return cl_per_kg_target, vd_per_kg_target


def main() -> None:
    # Oclacitinib PK: Canine 15kg, Human 70kg
    canine = {"bw_kg": 15.0, "cl_l_hr_kg": 0.82, "vd_l_kg": 2.9, "F": 0.89}
    human = {"bw_kg": 70.0, "cl_l_hr_kg": 0.38, "vd_l_kg": 1.24, "F": 0.74}

    # Scale canine → human
    cl_human_scaled, vd_human_scaled = scale_to_species(
        canine["cl_l_hr_kg"], canine["vd_l_kg"],
        canine["bw_kg"], human["bw_kg"],
    )

    # Roundtrip: canine → canine (same BW)
    cl_roundtrip, vd_roundtrip = scale_to_species(
        canine["cl_l_hr_kg"], canine["vd_l_kg"],
        canine["bw_kg"], canine["bw_kg"],
    )

    # Half-lives (total Vd and CL)
    t_half_canine = allometric_half_life(
        canine["vd_l_kg"] * canine["bw_kg"],
        canine["cl_l_hr_kg"] * canine["bw_kg"],
    )
    t_half_human_ref = allometric_half_life(
        human["vd_l_kg"] * human["bw_kg"],
        human["cl_l_hr_kg"] * human["bw_kg"],
    )
    t_half_human_scaled = allometric_half_life(
        vd_human_scaled * human["bw_kg"],
        cl_human_scaled * human["bw_kg"],
    )

    out = {
        "experiment": "exp104_cross_species_pk",
        "canine": {
            "body_weight_kg": canine["bw_kg"],
            "clearance_l_hr_kg": canine["cl_l_hr_kg"],
            "volume_distribution_l_kg": canine["vd_l_kg"],
            "bioavailability": canine["F"],
            "half_life_hr": t_half_canine,
        },
        "human_reference": {
            "body_weight_kg": human["bw_kg"],
            "clearance_l_hr_kg": human["cl_l_hr_kg"],
            "volume_distribution_l_kg": human["vd_l_kg"],
            "bioavailability": human["F"],
            "half_life_hr": t_half_human_ref,
        },
        "canine_scaled_to_human": {
            "clearance_l_hr_kg": cl_human_scaled,
            "volume_distribution_l_kg": vd_human_scaled,
            "half_life_hr": t_half_human_scaled,
        },
        "roundtrip_canine_to_canine": {
            "clearance_l_hr_kg": cl_roundtrip,
            "volume_distribution_l_kg": vd_roundtrip,
            "identity_check": abs(cl_roundtrip - canine["cl_l_hr_kg"]) < 1e-6
            and abs(vd_roundtrip - canine["vd_l_kg"]) < 1e-6,
        },
        "_provenance": {
            "date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
            "script": "control/comparative/exp104_cross_species_pk.py",
            "command": " ".join(sys.argv) or f"python3 {__file__}",
            "git_commit": "baseline",
            "python_version": sys.version.split()[0],
            "numpy_version": np.__version__,
        },
    }

    print(json.dumps(out, indent=2))


if __name__ == "__main__":
    main()
