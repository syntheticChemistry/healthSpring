# SPDX-License-Identifier: AGPL-3.0-or-later

#!/usr/bin/env python3
"""
healthSpring Exp040 — barraCuda CPU Parity Analytical Baselines

Documents the analytical reference values that both healthSpring pure-Rust
and barraCuda GPU implementations must agree on. These are mathematical
contracts, not statistical estimates.

Provenance:
  Baseline date:   2026-03-08
  Command:         python3 control/validation/exp040_barracuda_cpu.py
  Environment:     Python 3.10+, NumPy
"""

import json
import math
import os
import sys

import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))


def hill_equation(conc, ic50, hill_n, e_max=1.0):
    """Generalized Hill: E = E_max * C^n / (C^n + IC50^n)."""
    if ic50 <= 0 or conc < 0:
        return 0.0
    c_n = conc**hill_n
    ic50_n = ic50**hill_n
    return e_max * c_n / (c_n + ic50_n)


def pk_iv_bolus(dose, vd, half_life_hr, t_hr):
    """One-compartment IV: C(t) = (Dose/Vd) * exp(-k_e * t)."""
    if half_life_hr <= 0 or vd <= 0:
        return 0.0
    k_e = math.log(2) / half_life_hr
    c0 = dose / vd
    return c0 * math.exp(-k_e * t_hr)


def two_compartment_auc_analytical(dose, v1, k10):
    """Two-compartment IV AUC = dose / (V1 * k10) = dose/CL."""
    return dose / (v1 * k10)


def shannon_index(abundances):
    """Shannon H' = -Σ p_i * ln(p_i). Uniform: H' = ln(S)."""
    return -sum(p * math.log(p) for p in abundances if p > 0)


def simpson_index(abundances):
    """Simpson D = 1 - Σ p_i². Uniform: D = 1 - 1/S."""
    return 1.0 - sum(p * p for p in abundances)


def pielou_evenness(abundances):
    """Pielou J = H' / ln(S). Uniform: J = 1.0."""
    s = len([p for p in abundances if p > 0])
    if s <= 1:
        return 0.0
    h = shannon_index(abundances)
    return h / math.log(s)


def chao1(counts):
    """Chao1 = S_obs + f1²/(2*f2). No singletons: Chao1 = S_obs."""
    s_obs = len(counts)
    f1 = sum(1 for c in counts if c == 1)
    f2 = sum(1 for c in counts if c == 2)
    if f2 == 0:
        return s_obs + f1 * (f1 - 1) / 2.0 if f1 > 1 else s_obs
    return s_obs + f1 * f1 / (2.0 * f2)


def bray_curtis(a, b):
    """BC = 1 - 2*Σ min(a_i,b_i) / (Σ a_i + Σ b_i). Identical: BC = 0."""
    n = max(len(a), len(b))
    sum_min = sum(min(a[i] if i < len(a) else 0, b[i] if i < len(b) else 0) for i in range(n))
    sum_a = sum(a)
    sum_b = sum(b)
    denom = sum_a + sum_b
    return 1.0 - 2.0 * sum_min / denom if denom > 0 else 0.0


def ppg_r_value(ac_red, dc_red, ac_ir, dc_ir):
    """R = (AC_red/DC_red) / (AC_ir/DC_ir)."""
    if abs(dc_red) < 1e-15 or abs(dc_ir) < 1e-15 or abs(ac_ir) < 1e-15:
        return float("nan")
    return (ac_red / dc_red) / (ac_ir / dc_ir)


def main():
    baseline = {
        "_source": "healthSpring Exp040: barraCuda CPU Parity — Analytical Contract",
        "_method": "Analytical known-values (textbook identities)",
        "checks": {},
        "_provenance": {
            "date": "2026-03-08",
            "python": sys.version.split()[0],
            "numpy": np.__version__,
            "command": "python3 control/validation/exp040_barracuda_cpu.py",
            "script": "control/validation/exp040_barracuda_cpu.py",
        },
    }

    # PK/PD
    ec50, emax = 10.0, 1.0
    baseline["checks"]["hill_e_at_ec50"] = float(hill_equation(ec50, ec50, 1.0, emax))
    baseline["checks"]["hill_e_at_zero"] = float(hill_equation(0.0, 10.0, 1.0, 1.0))
    baseline["checks"]["hill_e_at_inf"] = float(hill_equation(1e12, 10.0, 1.0, 1.0))

    dose, vd, half_life = 100.0, 25.0, 6.0
    baseline["checks"]["one_comp_c0"] = float(pk_iv_bolus(dose, vd, half_life, 0.0))
    baseline["checks"]["one_comp_c_at_half_life"] = float(
        pk_iv_bolus(dose, vd, half_life, half_life)
    )

    dose2, v1, k10 = 240.0, 15.0, 0.35
    baseline["checks"]["two_comp_auc_analytical"] = float(
        two_compartment_auc_analytical(dose2, v1, k10)
    )

    # Microbiome
    s = 10
    uniform = [1.0 / s] * s
    baseline["checks"]["shannon_uniform"] = float(shannon_index(uniform))
    baseline["checks"]["simpson_uniform"] = float(simpson_index(uniform))
    baseline["checks"]["pielou_uniform"] = float(pielou_evenness(uniform))

    counts_no_singletons = [2, 2, 2, 5, 10]
    baseline["checks"]["chao1_no_singletons"] = float(chao1(counts_no_singletons))

    community = [0.25, 0.20, 0.15, 0.12, 0.10, 0.08, 0.05, 0.03, 0.01, 0.01]
    baseline["checks"]["bray_curtis_identical"] = float(bray_curtis(community, community))

    # Biosignal
    baseline["checks"]["ppg_r_value"] = float(ppg_r_value(0.02, 1.0, 0.04, 1.0))

    # Write baseline
    out_path = os.path.join(SCRIPT_DIR, "exp040_baseline.json")
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(baseline, f, indent=2)

    print("=" * 72)
    print("healthSpring Exp040: barraCuda CPU Parity — Analytical Baselines")
    print("=" * 72)
    for k, v in baseline["checks"].items():
        print(f"  {k}: {v}")
    print(f"\nWrote {out_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
