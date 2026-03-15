# SPDX-License-Identifier: AGPL-3.0-or-later

#!/usr/bin/env python3
"""
healthSpring Exp078 — Antibiotic Perturbation Recovery (Python Baseline)

Models Shannon diversity decline under ciprofloxacin and recovery.
Reference: Dethlefsen & Relman 2011, PNAS 108: 4554-4561.

Provenance:
  Baseline date:   2026-03-15
  Command:         python3 control/microbiome/exp078_antibiotic_perturbation.py
  Environment:     Python 3.10+, NumPy
"""
import json, os, sys
import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

def perturbation_model(h0, depth, k_dec, k_rec, treat_days, total_days, dt):
    n = int(total_days / dt)
    h_nadir = h0 * (1 - depth)
    result = []
    for i in range(n + 1):
        t = i * dt
        if t <= treat_days:
            h = h0 - (h0 - h_nadir) * (1 - np.exp(-k_dec * t))
        else:
            t_rec = t - treat_days
            h_end = h0 - (h0 - h_nadir) * (1 - np.exp(-k_dec * treat_days))
            target = h0 * (1 - depth * 0.15)
            h = h_end + (target - h_end) * (1 - np.exp(-k_rec * t_rec))
        result.append((t, h))
    return result

def main():
    passed = failed = 0
    print("=" * 72)
    print("healthSpring Exp078: Antibiotic Perturbation (Python)")
    print("=" * 72)

    traj = perturbation_model(2.2, 0.5, 0.3, 0.1, 7.0, 42.0, 0.1)
    times, shannons = zip(*traj)
    shannons = np.array(shannons)

    # Check 1-10 mirror Rust binary
    checks = [
        ("H0 start", abs(shannons[0] - 2.2) < 0.01),
        ("Decline", shannons[70] < 2.2),  # t=7
        ("Nadir < H0", np.min(shannons) < 2.2),
        ("Nadir timing", times[np.argmin(shannons)] <= 9.0),
        ("Recovery", shannons[-1] > np.min(shannons)),
        ("Incomplete", shannons[-1] < 2.2),
        ("Depth 30-60%", 0.3 < (2.2 - np.min(shannons))/2.2 < 0.6),
        ("All positive", np.all(shannons > 0)),
        ("Mono decline 5d", all(shannons[i+1] <= shannons[i]+1e-10 for i in range(50))),
        ("Mono recovery 10d+", all(shannons[i+1] >= shannons[i]-1e-10 for i in range(100, len(shannons)-1))),
    ]
    for name, ok in checks:
        print(f"  [{('PASS' if ok else 'FAIL')}] {name}")
        passed += ok; failed += (not ok)

    path = os.path.join(SCRIPT_DIR, "exp078_baseline.json")
    with open(path, "w") as f:
        json.dump({"_source": "Exp078 Antibiotic", "nadir": float(np.min(shannons)),
                    "h_final": float(shannons[-1]),
                    "_provenance": {"date": "2026-03-10"}}, f, indent=2)
    total = passed + failed
    print(f"\n{'='*72}\nTOTAL: {passed}/{total} PASS, {failed}/{total} FAIL\n{'='*72}")
    return 0 if failed == 0 else 1

if __name__ == "__main__":
    sys.exit(main())
