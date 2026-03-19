#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Exp099: Hormesis and biphasic dose-response modeling.

Implements biphasic_dose_response, hormetic_optimum, mithridatism,
caloric_restriction, and ecological_hormesis.
Reference: toxicology.rs
"""

import json
import os
import subprocess
import sys
from datetime import datetime, timezone

import numpy as np


def biphasic_dose_response(
    dose: float,
    baseline: float,
    s_max: float,
    k_stim: float,
    ic50: float,
    hill_n: float,
) -> float:
    """R(D) = baseline * (1 + s_max*D/(k_stim+D)) * (1 - D^n/(ic50^n + D^n))."""
    if dose <= 0:
        return baseline
    stimulation = s_max * dose / (k_stim + dose)
    inhibition = (dose**hill_n) / (ic50**hill_n + dose**hill_n)
    return baseline * (1.0 + stimulation) * (1.0 - inhibition)


def hormetic_optimum(
    baseline: float,
    s_max: float,
    k_stim: float,
    ic50: float,
    hill_n: float,
    dose_max: float,
    n_steps: int,
) -> tuple[float, float]:
    """Grid search for dose that maximizes fitness."""
    best_dose = 0.0
    best_fitness = baseline
    for i in range(n_steps + 1):
        dose = dose_max * i / n_steps
        fitness = biphasic_dose_response(dose, baseline, s_max, k_stim, ic50, hill_n)
        if fitness > best_fitness:
            best_fitness = fitness
            best_dose = dose
    return best_dose, best_fitness


def mithridatism_ic50_adapted(
    ic50_naive: float, max_adapt: float, k_adapt: float, n_exposures: float
) -> float:
    """IC50_adapted = IC50_naive * (1 + max_adapt * n / (k_adapt + n))."""
    adaptation = max_adapt * n_exposures / (k_adapt + n_exposures)
    return ic50_naive * (1.0 + adaptation)


def caloric_restriction_fitness(
    restriction_fraction: float,
    baseline_lifespan: float,
    longevity_gain: float,
    k_autophagy: float,
    starvation_ic50: float,
    hill_n: float,
) -> float:
    """Same biphasic with restriction_fraction as dose."""
    return biphasic_dose_response(
        restriction_fraction,
        baseline_lifespan,
        longevity_gain,
        k_autophagy,
        starvation_ic50,
        hill_n,
    )


def ecological_hormesis(
    pesticide_concentration: float,
    baseline_population: float,
    stress_response_gain: float,
    k_stress: float,
    lethal_ic50: float,
    hill_n: float,
) -> float:
    """Same biphasic for grasshopper population vs pesticide."""
    return biphasic_dose_response(
        pesticide_concentration,
        baseline_population,
        stress_response_gain,
        k_stress,
        lethal_ic50,
        hill_n,
    )


def main() -> None:
    script_dir = os.path.dirname(os.path.abspath(__file__))

    # At D=0 → 100.0
    at_zero = biphasic_dose_response(0.0, 100.0, 0.5, 1.0, 50.0, 2.0)

    # At D=1, base=100, s_max=0.5, k_stim=1.0, ic50=50, n=2 → ~125
    at_d1 = biphasic_dose_response(1.0, 100.0, 0.5, 1.0, 50.0, 2.0)

    # Hormetic optimum: grid search 0-100 with 10000 steps
    opt_dose, opt_value = hormetic_optimum(
        baseline=100.0, s_max=0.5, k_stim=1.0, ic50=50.0, hill_n=2.0,
        dose_max=100.0, n_steps=10000
    )

    # Mithridatism: IC50_adapted = IC50_naive * (1 + max_adapt * n / (k_adapt + n))
    ic50_naive = 10.0
    max_adapt = 5.0
    k_adapt = 10.0
    n_exposures = 20.0
    ic50_adapted = mithridatism_ic50_adapted(ic50_naive, max_adapt, k_adapt, n_exposures)

    # Caloric restriction: same biphasic with restriction_fraction as dose
    cr_ad_lib = caloric_restriction_fitness(0.0, 80.0, 0.3, 0.15, 0.7, 3.0)
    cr_mild = caloric_restriction_fitness(0.2, 80.0, 0.3, 0.15, 0.7, 3.0)
    cr_severe = caloric_restriction_fitness(0.9, 80.0, 0.3, 0.15, 0.7, 3.0)

    # Ecological hormesis: grasshopper population
    eco_no_pesticide = ecological_hormesis(0.0, 1000.0, 0.4, 0.5, 20.0, 2.0)
    eco_weak = ecological_hormesis(1.0, 1000.0, 0.4, 0.5, 20.0, 2.0)
    eco_strong = ecological_hormesis(50.0, 1000.0, 0.4, 0.5, 20.0, 2.0)

    try:
        git_commit = subprocess.check_output(
            ["git", "rev-parse", "HEAD"], text=True
        ).strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        git_commit = "unknown"

    out = {
        "experiment": "exp099_hormesis",
        "biphasic_dose_response": {
            "formula": "baseline * (1 + s_max*D/(k_stim+D)) * (1 - D^n/(ic50^n + D^n))",
            "at_D_zero": at_zero,
            "params_zero": {"baseline": 100.0},
            "at_D_one": at_d1,
            "params_D1": {
                "baseline": 100.0, "s_max": 0.5, "k_stim": 1.0,
                "ic50": 50.0, "n": 2.0,
            },
        },
        "hormetic_optimum": {
            "peak_dose": opt_dose,
            "peak_value": opt_value,
            "params": {
                "baseline": 100.0, "s_max": 0.5, "k_stim": 1.0,
                "ic50": 50.0, "hill_n": 2.0, "dose_max": 100.0, "n_steps": 10000,
            },
        },
        "mithridatism": {
            "formula": "IC50_adapted = IC50_naive * (1 + max_adapt * n / (k_adapt + n))",
            "ic50_naive": ic50_naive,
            "ic50_adapted": ic50_adapted,
            "params": {"max_adapt": max_adapt, "k_adapt": k_adapt, "n_exposures": n_exposures},
        },
        "caloric_restriction": {
            "ad_libitum": cr_ad_lib,
            "mild_restriction_0_2": cr_mild,
            "severe_restriction_0_9": cr_severe,
            "params": {"baseline": 80.0, "longevity_gain": 0.3, "k_autophagy": 0.15, "starvation_ic50": 0.7, "hill_n": 3.0},
        },
        "ecological_hormesis": {
            "no_pesticide": eco_no_pesticide,
            "weak_pesticide_1": eco_weak,
            "strong_pesticide_50": eco_strong,
            "params": {"baseline": 1000.0, "stress_gain": 0.4, "k_stress": 0.5, "lethal_ic50": 20.0, "hill_n": 2.0},
        },
        "_provenance": {
            "date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
            "script": "control/toxicology/exp099_hormesis.py",
            "command": " ".join(sys.argv) or f"python3 {__file__}",
            "git_commit": git_commit,
            "python_version": sys.version.split()[0],
            "numpy_version": np.__version__,
        },
    }

    print(json.dumps(out, indent=2))

    baseline_path = os.path.join(script_dir, "exp099_baseline.json")
    with open(baseline_path, "w") as f:
        json.dump(out, f, indent=2)
    print(f"\nBaseline written to {baseline_path}", file=sys.stderr)


if __name__ == "__main__":
    main()
