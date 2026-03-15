# SPDX-License-Identifier: AGPL-3.0-or-later

#!/usr/bin/env python3
"""
healthSpring Exp006 — PBPK Compartments

Physiologically-Based Pharmacokinetic (PBPK) modeling with organ-specific
compartments connected by blood flow.

Model: 5-tissue PBPK (liver, kidney, muscle, fat, rest)
  - Each tissue: volume, blood flow, Kp (tissue-plasma partition)
  - Liver has hepatic clearance
  - IV bolus: dose into venous blood, well-stirred mixing

Reference: Gabrielsson & Weiner, Pharmacokinetic and Pharmacodynamic Data Analysis

Provenance:
  Baseline date:   2026-03-08
  Command:         python3 control/pkpd/exp006_pbpk_compartments.py
  Environment:     Python 3.10+, NumPy
"""

import json
import os
import subprocess
import sys

import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIR))

# Standard human tissue parameters (70 kg adult) — match barracuda pbpk.rs
STANDARD_TISSUES = [
    {"name": "liver", "volume_l": 1.5, "blood_flow_l_per_hr": 90.0, "kp": 3.0, "clearance_l_per_hr": 15.0},
    {"name": "kidney", "volume_l": 0.31, "blood_flow_l_per_hr": 72.0, "kp": 2.5, "clearance_l_per_hr": 0.0},
    {"name": "muscle", "volume_l": 28.0, "blood_flow_l_per_hr": 54.0, "kp": 0.8, "clearance_l_per_hr": 0.0},
    {"name": "fat", "volume_l": 14.0, "blood_flow_l_per_hr": 18.0, "kp": 5.0, "clearance_l_per_hr": 0.0},
    {"name": "rest", "volume_l": 10.0, "blood_flow_l_per_hr": 96.0, "kp": 1.0, "clearance_l_per_hr": 0.0},
]

DOSE_MG = 100.0
BLOOD_VOLUME_L = 5.0
DURATION_HR = 48.0
DT = 0.01


def cardiac_output(tissues):
    return sum(t["blood_flow_l_per_hr"] for t in tissues)


def pbpk_iv_simulate(tissues, dose_mg, blood_volume_l, duration_hr, dt):
    """Euler integration PBPK IV bolus."""
    n_tissues = len(tissues)
    q_total = cardiac_output(tissues)

    conc = [0.0] * n_tissues
    c_venous = dose_mg / blood_volume_l

    times = [0.0]
    venous_profile = [c_venous]

    n_steps = int(duration_hr / dt)
    for step in range(1, n_steps + 1):
        c_arterial = c_venous

        for i, tissue in enumerate(tissues):
            c_free = conc[i] / tissue["kp"]
            uptake = tissue["blood_flow_l_per_hr"] * (c_arterial - c_free) / tissue["volume_l"]
            elimination = tissue["clearance_l_per_hr"] * c_free / tissue["volume_l"]
            conc[i] += (uptake - elimination) * dt
            conc[i] = max(0.0, conc[i])

        venous_num = sum(t["blood_flow_l_per_hr"] * conc[i] / t["kp"] for i, t in enumerate(tissues))
        c_venous = venous_num / q_total

        t = step * dt
        times.append(t)
        venous_profile.append(c_venous)

    return times, venous_profile, conc


def auc_trapezoidal(times, concentrations):
    if len(times) < 2:
        return 0.0
    return float(np.trapezoid(concentrations, times))


def main():
    print("=" * 72)
    print("healthSpring Exp006: PBPK Compartments")
    print(f"  100 mg IV bolus, 5 L blood, 48 hr, dt={DT}")
    print("=" * 72)

    times, venous_profile, tissue_conc_final = pbpk_iv_simulate(
        STANDARD_TISSUES, DOSE_MG, BLOOD_VOLUME_L, DURATION_HR, DT
    )

    times_arr = np.array(times)
    profile_arr = np.array(venous_profile)

    # C(0)
    c0 = venous_profile[0]
    assert abs(c0 - DOSE_MG / BLOOD_VOLUME_L) < 1e-6

    # AUC
    auc = auc_trapezoidal(times, venous_profile)

    # C(24hr), C(48hr)
    idx_24 = np.searchsorted(times_arr, 24.0)
    idx_48 = np.searchsorted(times_arr, 48.0)
    c_24hr = venous_profile[min(idx_24, len(venous_profile) - 1)]
    c_48hr = venous_profile[min(idx_48, len(venous_profile) - 1)]

    # Tissue concentrations at 24hr — re-run to capture state at 24hr
    _, _, tissue_at_24 = pbpk_iv_simulate(
        STANDARD_TISSUES, DOSE_MG, BLOOD_VOLUME_L, 24.0, DT
    )
    tissue_names = [t["name"] for t in STANDARD_TISSUES]
    tissue_conc_24hr = {name: float(c) for name, c in zip(tissue_names, tissue_at_24)}

    baseline = {
        "_source": "healthSpring Exp006: PBPK Compartments",
        "_method": "5-tissue PBPK, Euler integration, IV bolus",
        "dose_mg": DOSE_MG,
        "blood_volume_l": BLOOD_VOLUME_L,
        "duration_hr": DURATION_HR,
        "dt": DT,
        "c0_mg_per_L": float(c0),
        "auc_mg_hr_per_L": float(auc),
        "c_24hr_mg_per_L": float(c_24hr),
        "c_48hr_mg_per_L": float(c_48hr),
        "tissue_concentrations_24hr": tissue_conc_24hr,
        "cardiac_output_L_per_hr": float(cardiac_output(STANDARD_TISSUES)),
        "n_tissues": len(STANDARD_TISSUES),
        "_provenance": {
            "date": "2026-03-08",
            "python": sys.version,
            "numpy": np.__version__,
            "git_commit": (
                subprocess.run(
                    ["git", "rev-parse", "HEAD"],
                    capture_output=True,
                    text=True,
                    cwd=REPO_ROOT,
                ).stdout.strip()
                or "unknown"
            ),
            "command": "python3 control/pkpd/exp006_pbpk_compartments.py",
            "script": "control/pkpd/exp006_pbpk_compartments.py",
        },
    }

    baseline_path = os.path.join(SCRIPT_DIR, "exp006_baseline.json")
    with open(baseline_path, "w") as f:
        json.dump(baseline, f, indent=2)
    print(f"\nBaseline written to {baseline_path}")
    print(f"  C(0) = {c0:.4f} mg/L")
    print(f"  AUC = {auc:.4f} mg·hr/L")
    print(f"  C(24hr) = {c_24hr:.6f} mg/L")
    print(f"  C(48hr) = {c_48hr:.6f} mg/L")
    print(f"  Tissue @ 24hr: {tissue_conc_24hr}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
