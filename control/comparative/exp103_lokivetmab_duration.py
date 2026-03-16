#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Lokivetmab dose-duration baseline.

Reference: Michels et al., Vet Dermatol (2016) 27:478-e129.
Lokivetmab (Cytopoint) dose-response and duration in canine atopic dermatitis.
"""

import json
import sys
from datetime import datetime

import numpy as np


def main() -> None:
    doses = [0.25, 0.5, 1.0, 2.0, 4.0]
    bw = 15.0
    threshold = 1.0

    results = []
    for dose in doses:
        t_half = 7.0 * dose / 0.5 + 7.0
        k_el = np.log(2) / t_half
        C0 = (dose * bw / (85.0 * bw)) * 1000.0
        effective_duration = np.log(C0 / threshold) / k_el if C0 > threshold else 0.0
        onset_hr = max(1.0, min(12.0, 3.0 / np.sqrt(dose)))

        results.append({
            "dose_mg_kg": dose,
            "bw_kg": bw,
            "t_half_days": float(t_half),
            "k_el_per_day": float(k_el),
            "C0_ug_mL": float(C0),
            "effective_duration_days": float(effective_duration),
            "onset_hr": float(onset_hr),
        })

    out = {
        "results": results,
        "parameters": {"bw_kg": bw, "threshold_ug_mL": threshold},
        "_provenance": {
            "date": datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            "script": "control/comparative/exp103_lokivetmab_duration.py",
            "command": " ".join(sys.argv),
            "git_commit": "baseline",
            "python": sys.version.split()[0],
            "numpy": np.__version__,
        },
    }
    print(json.dumps(out, indent=2))


if __name__ == "__main__":
    main()
