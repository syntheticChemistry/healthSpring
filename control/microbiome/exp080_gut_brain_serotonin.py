# SPDX-License-Identifier: AGPL-3.0-or-later

#!/usr/bin/env python3
"""
healthSpring Exp080 — Gut-Brain Serotonin Pathway (Python Baseline)

Microbiome diversity → tryptophan → serotonin production.
Reference: Yano et al. 2015 (Cell), Clarke et al. 2013.

Provenance:
  Baseline date:   2026-03-15
  Command:         python3 control/microbiome/exp080_gut_brain_serotonin.py
  Environment:     Python 3.10+, NumPy
"""
import json, os, sys
import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

COMMUNITIES = {
    "healthy": [0.25, 0.20, 0.15, 0.12, 0.10, 0.08, 0.05, 0.03, 0.01, 0.01],
    "dysbiotic": [0.85, 0.05, 0.03, 0.02, 0.02, 0.01, 0.01, 0.005, 0.003, 0.002],
    "cdiff": [0.60, 0.15, 0.10, 0.05, 0.04, 0.03, 0.02, 0.005, 0.003, 0.002],
}

def shannon(p):
    p = np.array(p); p = p[p > 0]
    return float(-np.sum(p * np.log(p)))

def trp_availability(dietary, h):
    frac = 0.4 + 0.4 / (1 + np.exp(-3.0 * (h - 1.5)))
    return dietary * frac

def serotonin(trp, h, k=0.8, scale=0.1):
    div_factor = 1.0 / (1.0 + np.exp(-(h - 1.5) / scale))
    return k * trp * div_factor

def main():
    passed = failed = 0
    print("=" * 72)
    print("healthSpring Exp080: Gut-Brain Serotonin (Python)")
    print("=" * 72)

    h_vals = {k: shannon(v) for k, v in COMMUNITIES.items()}
    trp_vals = {k: trp_availability(200, h) for k, h in h_vals.items()}
    ser_vals = {k: serotonin(trp_vals[k], h_vals[k]) for k in COMMUNITIES}

    checks = [
        ("Trp healthy > dysbiotic", trp_vals["healthy"] > trp_vals["dysbiotic"]),
        ("Trp in range", 80 < trp_vals["healthy"] < 180),
        ("5-HT healthy > dysbiotic", ser_vals["healthy"] > ser_vals["dysbiotic"]),
        ("All 5-HT positive", all(v > 0 for v in ser_vals.values())),
        ("5-HT ordering", ser_vals["healthy"] > ser_vals["cdiff"] > ser_vals["dysbiotic"]),
        ("Sigmoid shape", serotonin(100, 0.5, 1, 0.1) < serotonin(100, 1.5, 1, 0.1) < serotonin(100, 2.5, 1, 0.1)),
        ("Midpoint ~50", 40 < serotonin(100, 1.5, 1, 0.1) < 60),
        ("Trp monotone", all(trp_availability(200, i*0.15) <= trp_availability(200, (i+1)*0.15)+1e-10 for i in range(19))),
        ("FMT recovery", serotonin(trp_availability(200, 2.1), 2.1) > ser_vals["cdiff"]),
        ("Zero trp", abs(serotonin(0, 2.2)) < 1e-10),
    ]
    for name, ok in checks:
        print(f"  [{('PASS' if ok else 'FAIL')}] {name}")
        passed += ok; failed += (not ok)

    path = os.path.join(SCRIPT_DIR, "exp080_baseline.json")
    with open(path, "w") as f:
        json.dump({"_source": "Exp080 Serotonin", "shannon": h_vals,
                    "serotonin": {k: float(v) for k, v in ser_vals.items()},
                    "_provenance": {"date": "2026-03-10"}}, f, indent=2)
    total = passed + failed
    print(f"\n{'='*72}\nTOTAL: {passed}/{total} PASS, {failed}/{total} FAIL\n{'='*72}")
    return 0 if failed == 0 else 1

if __name__ == "__main__":
    sys.exit(main())
