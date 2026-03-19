#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Exp111: Causal terrarium — multi-scale mechanistic simulation.

Implements pathway activation, damage accumulation, mechanistic cell fitness,
and population steady state. Reference: simulation.rs
"""

import json
import os
import subprocess
import sys
from datetime import datetime, timezone

import numpy as np


def pathway_activation(dose: float, max_benefit: float, k_half: float, hill_n: float) -> float:
    """benefit(D) = max_benefit * D^n / (k_half^n + D^n)."""
    if dose <= 0:
        return 0.0
    d_n = dose**hill_n
    k_n = k_half**hill_n
    return max_benefit * d_n / (k_n + d_n)


def damage_accumulation(dose: float, ic50: float, hill_n: float) -> float:
    """damage(D) = D^n / (ic50^n + D^n)."""
    if dose <= 0 or ic50 <= 0:
        return 0.0
    d_n = dose**hill_n
    ic50_n = ic50**hill_n
    return d_n / (ic50_n + d_n)


def mechanistic_cell_fitness(
    dose: float,
    baseline: float,
    pathways: list[dict],
    damage_ic50: float,
    damage_hill_n: float,
) -> float:
    """fitness = baseline * product(1 + activation_i) * (1 - damage)."""
    benefit = 1.0
    for p in pathways:
        act = pathway_activation(dose, p["max_benefit"], p["k_half"], p["hill_n"])
        benefit *= 1.0 + act
    damage = damage_accumulation(dose, damage_ic50, damage_hill_n)
    return baseline * benefit * (1.0 - damage)


def mechanistic_cell_fitness_detailed(
    dose: float,
    baseline: float,
    pathways: list[dict],
    damage_ic50: float,
    damage_hill_n: float,
) -> tuple[float, list[float], float]:
    """Returns (fitness, pathway_activations, damage_fraction)."""
    activations = [
        pathway_activation(dose, p["max_benefit"], p["k_half"], p["hill_n"])
        for p in pathways
    ]
    damage = damage_accumulation(dose, damage_ic50, damage_hill_n)
    benefit = 1.0
    for a in activations:
        benefit *= 1.0 + a
    fitness = baseline * benefit * (1.0 - damage)
    return fitness, activations, damage


def population_steady_state(k_base: float, fitness: float, baseline_fitness: float) -> float:
    """N_ss = K_base * (fitness / baseline_fitness)."""
    if baseline_fitness < 1e-15:
        return k_base
    return k_base * (fitness / baseline_fitness)


def standard_eukaryotic_pathways() -> list[dict]:
    """HSP, SOD, p53, mTOR."""
    return [
        {"name": "HSP (heat shock proteins)", "max_benefit": 0.12, "k_half": 0.5, "hill_n": 1.5},
        {"name": "antioxidant (SOD/catalase)", "max_benefit": 0.10, "k_half": 1.0, "hill_n": 2.0},
        {"name": "DNA repair (p53/BRCA)", "max_benefit": 0.08, "k_half": 2.0, "hill_n": 2.0},
        {"name": "autophagy (mTOR/AMPK)", "max_benefit": 0.15, "k_half": 3.0, "hill_n": 1.0},
    ]


def main() -> None:
    script_dir = os.path.dirname(os.path.abspath(__file__))

    pathways = standard_eukaryotic_pathways()
    baseline = 100.0
    damage_ic50 = 50.0
    damage_hill_n = 2.0
    k_base = 10_000.0

    # At dose=0 → 100.0
    fitness_d0 = mechanistic_cell_fitness(0.0, baseline, pathways, damage_ic50, damage_hill_n)

    # At dose=3 → expect ~137
    fitness_d3, activations_d3, damage_d3 = mechanistic_cell_fitness_detailed(
        3.0, baseline, pathways, damage_ic50, damage_hill_n
    )

    # At dose=80 → expect ~43
    fitness_d80, activations_d80, damage_d80 = mechanistic_cell_fitness_detailed(
        80.0, baseline, pathways, damage_ic50, damage_hill_n
    )

    # Population steady state at various doses
    pop_ss_d0 = population_steady_state(k_base, fitness_d0, baseline)
    pop_ss_d3 = population_steady_state(k_base, fitness_d3, baseline)
    pop_ss_d80 = population_steady_state(k_base, fitness_d80, baseline)

    try:
        git_commit = subprocess.check_output(
            ["git", "rev-parse", "HEAD"], text=True
        ).strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        git_commit = "unknown"

    out = {
        "experiment": "exp111_causal_terrarium",
        "pathways": pathways,
        "formulas": {
            "pathway_activation": "max * D^n / (k^n + D^n)",
            "damage": "D^n / (IC50^n + D^n)",
            "mechanistic_cell_fitness": "baseline * product(1 + activation_i) * (1 - damage)",
            "population_steady_state": "K_base * (fitness / baseline)",
        },
        "dose_zero": {
            "fitness": fitness_d0,
            "population_ss": pop_ss_d0,
        },
        "dose_three": {
            "fitness": fitness_d3,
            "pathway_activations": activations_d3,
            "damage_fraction": damage_d3,
            "population_ss": pop_ss_d3,
        },
        "dose_eighty": {
            "fitness": fitness_d80,
            "pathway_activations": activations_d80,
            "damage_fraction": damage_d80,
            "population_ss": pop_ss_d80,
        },
        "params": {
            "baseline": baseline,
            "damage_ic50": damage_ic50,
            "damage_hill_n": damage_hill_n,
            "k_base": k_base,
        },
        "_provenance": {
            "date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
            "script": "control/simulation/exp111_causal_terrarium.py",
            "command": " ".join(sys.argv) or f"python3 {__file__}",
            "git_commit": git_commit,
            "python_version": sys.version.split()[0],
            "numpy_version": np.__version__,
        },
    }

    print(json.dumps(out, indent=2))

    baseline_path = os.path.join(script_dir, "exp111_baseline.json")
    with open(baseline_path, "w") as f:
        json.dump(out, f, indent=2)
    print(f"\nBaseline written to {baseline_path}", file=sys.stderr)


if __name__ == "__main__":
    main()
