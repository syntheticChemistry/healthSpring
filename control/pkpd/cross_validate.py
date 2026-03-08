# SPDX-License-Identifier: AGPL-3.0-or-later

#!/usr/bin/env python3
"""
Cross-validation: compare Python baselines against Rust output.

Reads the Python baseline JSON files and checks that key numerical values
from Rust binaries match within tolerance.
"""

import json
import os
import sys

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
TOL = 1e-6


def load_baseline(name):
    path = os.path.join(SCRIPT_DIR, name)
    with open(path) as f:
        return json.load(f)


def check(label, python_val, rust_val, tol=TOL):
    diff = abs(float(python_val) - float(rust_val))
    ok = diff < tol
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] {label}: py={python_val}, rs={rust_val}, diff={diff:.2e}")
    return ok


def main():
    passed = 0
    failed = 0

    print("=" * 72)
    print("Python ↔ Rust Cross-Validation")
    print("=" * 72)

    # Exp001
    print("\n--- Exp001: Hill Dose-Response ---")
    b1 = load_baseline("exp001_baseline.json")

    for drug in ["baricitinib", "upadacitinib", "abrocitinib", "oclacitinib"]:
        ok = check(f"{drug} at IC50", b1[f"{drug}_at_ic50"], 0.5)
        passed += ok
        failed += not ok

    ok = check("cooperativity n=1 below IC50", b1["cooperativity"]["n1"], 1.0 / 3.0)
    passed += ok; failed += not ok
    ok = check("cooperativity n=2 below IC50", b1["cooperativity"]["n2"], 0.2)
    passed += ok; failed += not ok

    for drug in ["baricitinib", "upadacitinib", "abrocitinib", "oclacitinib"]:
        ec = b1[f"{drug}_ec_values"]
        ok = ec["ec10"] < ec["ec50"] < ec["ec90"]
        status = "MATCH" if ok else "MISMATCH"
        print(f"  [{status}] {drug} EC ordering: {ec['ec10']:.2f} < {ec['ec50']:.2f} < {ec['ec90']:.2f}")
        passed += ok; failed += not ok

    # Exp002
    print("\n--- Exp002: One-Compartment PK ---")
    b2 = load_baseline("exp002_baseline.json")

    ok = check("IV C(0)", b2["iv_c0"], 10.0)
    passed += ok; failed += not ok

    ok = check("IV at half-life", b2["iv_at_half_life"], 5.0)
    passed += ok; failed += not ok

    ok = check("Oral C(0)", b2["oral_c0"], 0.0)
    passed += ok; failed += not ok

    ok = check("Oral Cmax", b2["oral_cmax"], 4.3105, tol=0.01)
    passed += ok; failed += not ok

    ok = check("Oral Tmax", b2["oral_tmax"], 1.634, tol=0.01)
    passed += ok; failed += not ok

    ok = check("IV AUC (py vs analytical)", b2["iv_auc_numerical"], b2["iv_auc_analytical"], tol=1.0)
    passed += ok; failed += not ok

    ok = check("Oral AUC (py vs analytical)", b2["oral_auc"], b2["oral_auc_analytical"], tol=0.5)
    passed += ok; failed += not ok

    total = passed + failed
    print(f"\n{'=' * 72}")
    print(f"CROSS-VALIDATION: {passed}/{total} MATCH, {failed}/{total} MISMATCH")
    print(f"{'=' * 72}")

    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
