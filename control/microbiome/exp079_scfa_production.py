# SPDX-License-Identifier: AGPL-3.0-or-later

#!/usr/bin/env python3
"""
healthSpring Exp079 — SCFA Production Model (Python Baseline)

Michaelis-Menten fermentation: fiber → acetate, propionate, butyrate.
Reference: den Besten et al. 2013, Cummings 1987.

Provenance:
  Baseline date:   2026-03-15
  Command:         python3 control/microbiome/exp079_scfa_production.py
  Environment:     Python 3.10+, NumPy
"""
import json, os, sys
import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

HEALTHY = {"vmax_a": 60, "km_a": 8, "vmax_p": 20, "km_p": 10, "vmax_b": 15, "km_b": 12}
DYSBIOTIC = {"vmax_a": 55, "km_a": 8, "vmax_p": 18, "km_p": 10, "vmax_b": 5, "km_b": 15}

def scfa(fiber, p):
    mm = lambda v, k: v * fiber / (k + fiber)
    return mm(p["vmax_a"], p["km_a"]), mm(p["vmax_p"], p["km_p"]), mm(p["vmax_b"], p["km_b"])

def main():
    passed = failed = 0
    print("=" * 72)
    print("healthSpring Exp079: SCFA Production (Python)")
    print("=" * 72)

    a, p, b = scfa(20, HEALTHY)
    total = a + p + b
    a_f, p_f, b_f = a/total, p/total, b/total
    a_d, p_d, b_d = scfa(20, DYSBIOTIC)

    checks = [
        ("Ratio 60:20:15", 0.50 < a_f < 0.70 and 0.15 < p_f < 0.30),
        ("Acetate dominant", a > p > b),
        ("Saturation", scfa(100, HEALTHY)[0] / scfa(5, HEALTHY)[0] < 20),
        ("Zero fiber", all(v < 1e-10 for v in scfa(0, HEALTHY))),
        ("Dysbiotic less butyrate", b > b_d),
        ("Dysbiotic butyrate frac lower", b_f > b_d/(a_d+p_d+b_d)),
        ("More fiber more SCFA", sum(scfa(30, HEALTHY)) > sum(scfa(5, HEALTHY))),
        ("All positive", a > 0 and p > 0 and b > 0),
        ("Propionate range", 0.15 <= p_f <= 0.30),
        ("Butyrate range", 0.10 <= b_f <= 0.25),
    ]
    for name, ok in checks:
        print(f"  [{('PASS' if ok else 'FAIL')}] {name}")
        passed += ok; failed += (not ok)

    path = os.path.join(SCRIPT_DIR, "exp079_baseline.json")
    with open(path, "w") as f:
        json.dump({"_source": "Exp079 SCFA", "acetate": a, "propionate": p,
                    "butyrate": b, "_provenance": {"date": "2026-03-10"}}, f, indent=2)
    total_c = passed + failed
    print(f"\n{'='*72}\nTOTAL: {passed}/{total_c} PASS, {failed}/{total_c} FAIL\n{'='*72}")
    return 0 if failed == 0 else 1

if __name__ == "__main__":
    sys.exit(main())
