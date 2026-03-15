# SPDX-License-Identifier: AGPL-3.0-only

#!/usr/bin/env python3
"""
healthSpring Exp001 — Hill Dose-Response for Human JAK Inhibitors

Validates generalized Hill equation dose-response curves for human
atopic dermatitis JAK inhibitors, extending neuralSpring nS-601
(canine oclacitinib) to human therapeutics.

Drug data from published Phase III trials:
  - Baricitinib (Olumiant): JAK1/JAK2, IC50_JAK1 ≈ 5.9 nM
  - Upadacitinib (Rinvoq):  JAK1-selective, IC50_JAK1 ≈ 8 nM
  - Abrocitinib (Cibinqo):  JAK1-selective, IC50_JAK1 ≈ 29 nM
  - Oclacitinib (Apoquel):   JAK1, IC50_JAK1 = 10 nM (Gonzales 2014, canine reference)

Provenance:
  Baseline date:   2026-03-08
  Command:         python3 control/pkpd/exp001_hill_dose_response.py
  Environment:     Python 3.10+, NumPy
"""

import json
import os
import sys

import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
SEED = 42

# Published IC50 values (nM) for JAK1 pathway
HUMAN_JAK_INHIBITORS = {
    "baricitinib":  {"ic50_jak1_nm": 5.9,  "hill_n": 1.0, "selectivity": "JAK1/JAK2"},
    "upadacitinib": {"ic50_jak1_nm": 8.0,  "hill_n": 1.0, "selectivity": "JAK1"},
    "abrocitinib":  {"ic50_jak1_nm": 29.0, "hill_n": 1.0, "selectivity": "JAK1"},
    "oclacitinib":  {"ic50_jak1_nm": 10.0, "hill_n": 1.0, "selectivity": "JAK1 (canine)"},
}

# Concentration range for dose-response curves (nM)
CONCENTRATIONS = np.logspace(-1, 4, 100)  # 0.1 to 10,000 nM


def hill_equation(conc, ic50, hill_n, e_max=1.0):
    """Generalized Hill equation: E = E_max * C^n / (C^n + IC50^n)."""
    c_n = np.power(conc, hill_n)
    ic50_n = np.power(ic50, hill_n)
    return e_max * c_n / (c_n + ic50_n)


def compute_ec_values(ic50, hill_n):
    """Compute EC10, EC50, EC90 from Hill parameters."""
    ec50 = ic50
    ec10 = ic50 * (0.1 / 0.9) ** (1.0 / hill_n)
    ec90 = ic50 * (0.9 / 0.1) ** (1.0 / hill_n)
    return {"ec10": float(ec10), "ec50": float(ec50), "ec90": float(ec90)}


def main():
    total_passed = 0
    total_failed = 0
    baseline = {}

    print("=" * 72)
    print("healthSpring Exp001: Hill Dose-Response — Human JAK Inhibitors")
    print("=" * 72)

    # ------------------------------------------------------------------
    # Check 1: Hill equation at IC50 = 50% response (for each drug)
    # ------------------------------------------------------------------
    for drug_name, params in HUMAN_JAK_INHIBITORS.items():
        print(f"\n--- Check: {drug_name} at IC50 → 50% ---")
        ic50 = params["ic50_jak1_nm"]
        response = hill_equation(ic50, ic50, params["hill_n"])
        baseline[f"{drug_name}_at_ic50"] = float(response)
        if abs(response - 0.5) < 1e-10:
            print(f"  [PASS] {drug_name}: response={response:.10f} at IC50={ic50}nM")
            total_passed += 1
        else:
            print(f"  [FAIL] {drug_name}: response={response:.10f}")
            total_failed += 1

    # ------------------------------------------------------------------
    # Check 5: Dose-response curves are monotonically increasing
    # ------------------------------------------------------------------
    for drug_name, params in HUMAN_JAK_INHIBITORS.items():
        print(f"\n--- Check: {drug_name} monotonicity ---")
        ic50 = params["ic50_jak1_nm"]
        responses = hill_equation(CONCENTRATIONS, ic50, params["hill_n"])
        monotonic = all(responses[i] <= responses[i + 1] for i in range(len(responses) - 1))
        baseline[f"{drug_name}_monotonic"] = monotonic
        if monotonic:
            print(f"  [PASS] {drug_name}: monotonically increasing across 0.1-10000 nM")
            total_passed += 1
        else:
            print(f"  [FAIL] {drug_name}: NOT monotonic")
            total_failed += 1

    # ------------------------------------------------------------------
    # Check 9: Potency ordering (lower IC50 = more potent at equal conc)
    # ------------------------------------------------------------------
    print("\n--- Check: Potency ordering at 10 nM ---")
    responses_at_10 = {}
    for drug_name, params in HUMAN_JAK_INHIBITORS.items():
        r = hill_equation(10.0, params["ic50_jak1_nm"], params["hill_n"])
        responses_at_10[drug_name] = float(r)

    baseline["responses_at_10nm"] = responses_at_10
    sorted_by_response = sorted(responses_at_10.items(), key=lambda x: x[1], reverse=True)
    sorted_by_ic50 = sorted(HUMAN_JAK_INHIBITORS.items(), key=lambda x: x[1]["ic50_jak1_nm"])

    potency_order_correct = [x[0] for x in sorted_by_response] == [x[0] for x in sorted_by_ic50]
    if potency_order_correct:
        order_str = " > ".join(f"{n}({r:.3f})" for n, r in sorted_by_response)
        print(f"  [PASS] Potency order: {order_str}")
        total_passed += 1
    else:
        print(f"  [FAIL] Potency order mismatch")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 10: EC10/EC50/EC90 values are ordered correctly
    # ------------------------------------------------------------------
    for drug_name, params in HUMAN_JAK_INHIBITORS.items():
        print(f"\n--- Check: {drug_name} EC values ordered ---")
        ec = compute_ec_values(params["ic50_jak1_nm"], params["hill_n"])
        baseline[f"{drug_name}_ec_values"] = ec
        if ec["ec10"] < ec["ec50"] < ec["ec90"]:
            print(f"  [PASS] EC10={ec['ec10']:.2f} < EC50={ec['ec50']:.2f} < EC90={ec['ec90']:.2f} nM")
            total_passed += 1
        else:
            print(f"  [FAIL] EC values not ordered: {ec}")
            total_failed += 1

    # ------------------------------------------------------------------
    # Check 14: Hill n=2 is steeper than n=1 below IC50
    # ------------------------------------------------------------------
    print("\n--- Check: Hill cooperativity (n=2 steeper below IC50) ---")
    test_ic50 = 10.0
    test_conc = 5.0  # below IC50
    r_n1 = hill_equation(test_conc, test_ic50, 1.0)
    r_n2 = hill_equation(test_conc, test_ic50, 2.0)
    baseline["cooperativity"] = {"n1": float(r_n1), "n2": float(r_n2)}
    if r_n2 < r_n1:
        print(f"  [PASS] n=2 ({r_n2:.4f}) < n=1 ({r_n1:.4f}) below IC50")
        total_passed += 1
    else:
        print(f"  [FAIL] n=2 ({r_n2:.4f}) >= n=1 ({r_n1:.4f})")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 15: Hill n=2 above IC50 — n=2 higher than n=1
    # ------------------------------------------------------------------
    print("\n--- Check: Hill cooperativity (n=2 higher above IC50) ---")
    test_conc_above = 20.0  # above IC50
    r_n1_above = hill_equation(test_conc_above, test_ic50, 1.0)
    r_n2_above = hill_equation(test_conc_above, test_ic50, 2.0)
    baseline["cooperativity_above"] = {"n1": float(r_n1_above), "n2": float(r_n2_above)}
    if r_n2_above > r_n1_above:
        print(f"  [PASS] n=2 ({r_n2_above:.4f}) > n=1 ({r_n1_above:.4f}) above IC50")
        total_passed += 1
    else:
        print(f"  [FAIL] n=2 ({r_n2_above:.4f}) <= n=1 ({r_n1_above:.4f})")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 16: At very high concentration, all drugs → ~1.0
    # ------------------------------------------------------------------
    print("\n--- Check: Saturation at 100x IC50 ---")
    for drug_name, params in HUMAN_JAK_INHIBITORS.items():
        conc_100x = params["ic50_jak1_nm"] * 100
        r = hill_equation(conc_100x, params["ic50_jak1_nm"], params["hill_n"])
        if r > 0.99:
            print(f"  [PASS] {drug_name}: {r:.6f} > 0.99 at {conc_100x:.0f} nM")
            total_passed += 1
        else:
            print(f"  [FAIL] {drug_name}: {r:.6f}")
            total_failed += 1

    # ------------------------------------------------------------------
    # Write baseline JSON
    # ------------------------------------------------------------------
    baseline_path = os.path.join(SCRIPT_DIR, "exp001_baseline.json")
    baseline_out = {
        "_source": "healthSpring Exp001: Hill Dose-Response (Human JAK Inhibitors)",
        "_method": "Generalized Hill equation",
        "seed": SEED,
        "drugs": HUMAN_JAK_INHIBITORS,
        **baseline,
        "_provenance": {
            "date": "2026-03-08",
            "python": sys.version,
            "numpy": np.__version__,
        },
    }
    with open(baseline_path, "w") as f:
        json.dump(baseline_out, f, indent=2, default=str)
    print(f"\nBaseline written to {baseline_path}")

    # ------------------------------------------------------------------
    # Summary
    # ------------------------------------------------------------------
    total = total_passed + total_failed
    print(f"\n{'=' * 72}")
    print(f"TOTAL: {total_passed}/{total} PASS, {total_failed}/{total} FAIL")
    print(f"{'=' * 72}")

    return 0 if total_failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
