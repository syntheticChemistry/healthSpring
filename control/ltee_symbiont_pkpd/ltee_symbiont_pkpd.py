#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 ecoPrimals
"""
LTEE B5 — Symbiont PK/PD: Engineered Gut Bacteria as Drug Delivery

Reproduces the core pharmacokinetic modeling from:
  Leonard SP, Perutka J, Powell JE, Geng P, Richhart DD, Byrom M, Kar S,
  Davies BW, Moran NA (2024) "One-step genome engineering of E. coli
  colonizing the honeybee gut." mBio 15(3):e03342-23.

The paper demonstrates engineered Snodgrassella alvi colonizing the
honeybee (Apis mellifera) gut and producing therapeutic molecules (dsRNA
for gene knockdown). We model four coupled processes:

  1. Logistic colonization dynamics (S. alvi in bee gut)
  2. Biomass-proportional molecule production
  3. One-compartment gut-lumen PK (first-order elimination)
  4. Hill dose-response efficacy (target knockdown)

This is the Python Tier 0 baseline for the LTEE Targeted GuideStone
artifact. The expected_values.json defines tolerance envelopes; Rust
validation (Tier 1) will reproduce these dynamics in healthSpring.

This is the critical path: B5 reproduction → lithoSpore ltee-symbiont-pk.

Provenance:
  Paper:      Leonard et al. 2024, mBio 15(3):e03342-23
  DOI:        10.1128/mbio.03342-23
  LTEE ID:    B5
  Created:    2026-05-11
  Command:    python3 control/ltee_symbiont_pkpd/ltee_symbiont_pkpd.py
  Env:        Python 3.10+, NumPy
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

try:
    import numpy as np
except ImportError:
    np = None

import math

SCRIPT_DIR = Path(__file__).resolve().parent
EXPECTED = SCRIPT_DIR / "expected_values.json"
BENCHMARK = SCRIPT_DIR / "benchmark_ltee_symbiont.json"


def load_expected():
    with open(EXPECTED) as f:
        return json.load(f)


# ── Array helpers (pure-Python fallback when numpy unavailable) ───────

def _linspace(start, stop, n):
    if np is not None:
        return np.linspace(start, stop, n)
    step = (stop - start) / (n - 1)
    return [start + i * step for i in range(n)]


def _zeros(n):
    if np is not None:
        return np.zeros(n)
    return [0.0] * n


def _exp(x):
    if np is not None:
        return np.exp(x)
    if isinstance(x, (list, tuple)):
        return [math.exp(v) for v in x]
    return math.exp(x)


def _power(base, exp_val):
    if np is not None:
        return np.power(base, exp_val)
    if isinstance(base, (list, tuple)):
        return [math.pow(b, exp_val) for b in base]
    return math.pow(base, exp_val)


def _diff(arr):
    if np is not None:
        return np.diff(arr)
    return [arr[i + 1] - arr[i] for i in range(len(arr) - 1)]


def _argmin_abs_diff(arr, val):
    if np is not None:
        return int(np.argmin(np.abs(np.array(arr) - val)))
    return min(range(len(arr)), key=lambda i: abs(arr[i] - val))


# ── Colonization dynamics ─────────────────────────────────────────────

def logistic_growth(t_days, n0, k, r):
    """Logistic growth: N(t) = K / (1 + ((K - N0) / N0) * exp(-r * t))."""
    ratio = (k - n0) / n0
    neg_r_t = [-r * t for t in t_days] if isinstance(t_days, list) else -r * t_days
    exp_vals = _exp(neg_r_t)
    if isinstance(exp_vals, list):
        return [k / (1.0 + ratio * e) for e in exp_vals]
    return k / (1.0 + ratio * exp_vals)


def doubling_time_from_rate(r):
    """T_double = ln(2) / r."""
    return math.log(2.0) / r


def time_to_half_capacity(n0, k, r):
    """Time when N(t) = K/2 from logistic model."""
    return math.log((k - n0) / n0) / r


# ── Molecule production ───────────────────────────────────────────────

def production_rate(cfu, rate_per_cfu):
    """Total production = CFU * per-cell rate."""
    return cfu * rate_per_cfu


# ── Gut lumen PK (one-compartment) ───────────────────────────────────

def gut_lumen_concentration(production_total, ke, v_dist, f_bioavail):
    """Steady-state: C_ss = (F * Production) / (ke * V)."""
    return f_bioavail * production_total / (ke * v_dist)


def half_life_from_ke(ke):
    """t_half = ln(2) / ke."""
    return math.log(2.0) / ke


# ── Efficacy (Hill dose-response) ────────────────────────────────────

def hill_knockdown(conc, ec50, hill_n, e_max):
    """Fractional knockdown: E = Emax * C^n / (C^n + EC50^n)."""
    c_n = _power(conc, hill_n)
    ec_n = math.pow(ec50, hill_n)
    if isinstance(c_n, list):
        return [e_max * c / (c + ec_n) for c in c_n]
    return e_max * c_n / (c_n + ec_n)


# ── Simulation ────────────────────────────────────────────────────────

def simulate_symbiont_pkpd(params, dt=0.1, total_days=14):
    """Run coupled colonization → production → PK → efficacy simulation."""
    col = params["colonization"]
    prod = params["production_kinetics"]
    pk = params["symbiont_pk"]
    eff = params["efficacy"]

    n0 = col["initial_cfu_per_gut"]
    k = col["carrying_capacity_cfu"]
    r = col["growth_rate_per_day"]

    ke = pk["ke_per_day"]
    f_bio = pk["bioavailability_gut"]
    prod_rate_val = prod["production_rate_pg_per_cfu_per_day"]

    n_steps = int(total_days / dt)
    time = _linspace(0.0, total_days, n_steps + 1)
    cfu = logistic_growth(time, n0, k, r)

    molecule_pg = _zeros(n_steps + 1)
    for i in range(n_steps):
        cfu_i = cfu[i] if isinstance(cfu, list) else float(cfu[i])
        input_rate = cfu_i * prod_rate_val
        molecule_pg[i + 1] = molecule_pg[i] + dt * (f_bio * input_rate - ke * molecule_pg[i])

    molecule_ng = [m / 1000.0 for m in molecule_pg] if isinstance(molecule_pg, list) else molecule_pg / 1000.0

    knockdown = hill_knockdown(
        molecule_ng,
        eff["ec50_ng_per_gut"],
        eff["hill_n"],
        eff["max_knockdown_fraction"],
    )

    return {
        "time": time,
        "cfu": cfu,
        "molecule_ng": molecule_ng,
        "knockdown": knockdown,
    }


# ── Checks ────────────────────────────────────────────────────────────

def main():
    expected = load_expected()
    params = expected["model_parameters"]
    outcomes = expected["expected_outcomes"]

    total_passed = 0
    total_failed = 0
    benchmark = {"paper_id": "B5", "paper": "Leonard2024"}

    print("=" * 72)
    print("LTEE B5: Symbiont PK/PD — Leonard et al. 2024")
    print("=" * 72)

    sim = simulate_symbiont_pkpd(params)

    # Check 1: Colonization at day 7
    print("\n--- Check 1: Colonization at day 7 ---")
    idx_day7 = _argmin_abs_diff(sim["time"], 7.0)
    cfu_day7 = sim["cfu"][idx_day7]
    expected_cfu = outcomes["colonization_at_day_7_cfu"]
    tol = outcomes["colonization_at_day_7_tolerance"]
    ratio = cfu_day7 / expected_cfu
    benchmark["colonization_day7_cfu"] = float(cfu_day7)
    if abs(ratio - 1.0) < tol:
        print(f"  [PASS] CFU at day 7: {cfu_day7:.2e} (expected {expected_cfu:.2e}, ratio {ratio:.4f})")
        total_passed += 1
    else:
        print(f"  [FAIL] CFU at day 7: {cfu_day7:.2e} (expected {expected_cfu:.2e}, ratio {ratio:.4f})")
        total_failed += 1

    # Check 2: Carrying capacity approached
    print("\n--- Check 2: Colonization approaches carrying capacity ---")
    k = params["colonization"]["carrying_capacity_cfu"]
    cfu_final = sim["cfu"][-1]
    ratio_k = cfu_final / k
    benchmark["colonization_final_cfu"] = float(cfu_final)
    if ratio_k > 0.99:
        print(f"  [PASS] Final CFU: {cfu_final:.2e} ({ratio_k:.4f} of K={k:.2e})")
        total_passed += 1
    else:
        print(f"  [FAIL] Final CFU: {cfu_final:.2e} ({ratio_k:.4f} of K)")
        total_failed += 1

    # Check 3: Doubling time
    print("\n--- Check 3: Doubling time ---")
    r = params["colonization"]["growth_rate_per_day"]
    t_double_hours = doubling_time_from_rate(r) * 24.0
    expected_double = params["colonization"]["doubling_time_hours"]
    benchmark["doubling_time_hours"] = float(t_double_hours)
    if abs(t_double_hours - expected_double) < 1.0:
        print(f"  [PASS] Doubling time: {t_double_hours:.1f} h (expected {expected_double} h)")
        total_passed += 1
    else:
        print(f"  [FAIL] Doubling time: {t_double_hours:.1f} h (expected {expected_double} h)")
        total_failed += 1

    # Check 4: Time to half-max colonization
    print("\n--- Check 4: Time to half-max colonization ---")
    t_half = time_to_half_capacity(
        params["colonization"]["initial_cfu_per_gut"],
        k, r,
    )
    expected_t_half = outcomes["time_to_half_max_colonization_days"]
    benchmark["time_to_half_max_days"] = float(t_half)
    if abs(t_half - expected_t_half) < 1.0:
        print(f"  [PASS] t_half = {t_half:.2f} days (expected ~{expected_t_half})")
        total_passed += 1
    else:
        print(f"  [FAIL] t_half = {t_half:.2f} days (expected ~{expected_t_half})")
        total_failed += 1

    # Check 5: Steady-state molecule concentration
    print("\n--- Check 5: Steady-state molecule concentration ---")
    mol_final = sim["molecule_ng"][-1]
    expected_mol = outcomes["steady_state_molecule_ng"]
    mol_tol = outcomes["steady_state_molecule_tolerance"]
    mol_ratio = mol_final / expected_mol
    benchmark["steady_state_molecule_ng"] = float(mol_final)
    if abs(mol_ratio - 1.0) < mol_tol:
        print(f"  [PASS] Steady-state: {mol_final:.1f} ng (expected {expected_mol}, ratio {mol_ratio:.4f})")
        total_passed += 1
    else:
        print(f"  [FAIL] Steady-state: {mol_final:.1f} ng (expected {expected_mol}, ratio {mol_ratio:.4f})")
        total_failed += 1

    # Check 6: Molecule is monotonically increasing after colonization
    print("\n--- Check 6: Molecule monotonically increasing (day 1-14) ---")
    idx_d1 = _argmin_abs_diff(sim["time"], 1.0)
    mol_slice = sim["molecule_ng"][idx_d1:]
    diffs = _diff(mol_slice)
    monotonic = all(d >= -1e-6 for d in diffs)
    benchmark["molecule_monotonic_post_d1"] = bool(monotonic)
    if monotonic:
        print(f"  [PASS] Molecule concentration monotonically increasing after day 1")
        total_passed += 1
    else:
        n_decreases = sum(1 for d in diffs if d < -1e-6)
        print(f"  [FAIL] {n_decreases} decreases in molecule concentration after day 1")
        total_failed += 1

    # Check 7: Knockdown at steady state
    print("\n--- Check 7: Knockdown efficacy at steady state ---")
    kd_final = sim["knockdown"][-1]
    expected_kd = outcomes["knockdown_at_steady_state"]
    kd_tol = outcomes["knockdown_tolerance"]
    benchmark["knockdown_steady_state"] = float(kd_final)
    if abs(kd_final - expected_kd) < kd_tol:
        print(f"  [PASS] Knockdown: {kd_final:.4f} (expected {expected_kd}, tol {kd_tol})")
        total_passed += 1
    else:
        print(f"  [FAIL] Knockdown: {kd_final:.4f} (expected {expected_kd})")
        total_failed += 1

    # Check 8: Half-life from ke
    print("\n--- Check 8: PK half-life ---")
    ke = params["symbiont_pk"]["ke_per_day"]
    t_half_pk = half_life_from_ke(ke) * 24.0
    expected_half = params["symbiont_pk"]["half_life_hours"]
    benchmark["pk_half_life_hours"] = float(t_half_pk)
    if abs(t_half_pk - expected_half) < 1.0:
        print(f"  [PASS] PK half-life: {t_half_pk:.1f} h (expected {expected_half} h)")
        total_passed += 1
    else:
        print(f"  [FAIL] PK half-life: {t_half_pk:.1f} h (expected {expected_half} h)")
        total_failed += 1

    # Write benchmark
    benchmark["_provenance"] = {
        "date": "2026-05-11",
        "python": sys.version,
        "numpy": np.__version__ if np is not None else "pure-python-fallback",
    }
    with open(BENCHMARK, "w") as f:
        json.dump(benchmark, f, indent=2, default=str)
    print(f"\nBenchmark written to {BENCHMARK}")

    total = total_passed + total_failed
    print(f"\n{'=' * 72}")
    print(f"TOTAL: {total_passed}/{total} PASS, {total_failed}/{total} FAIL")
    print(f"{'=' * 72}")

    return 0 if total_failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
