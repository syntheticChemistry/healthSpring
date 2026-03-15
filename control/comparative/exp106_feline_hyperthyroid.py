#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""
Feline hyperthyroidism methimazole PK baseline.

Reference: Trepanier, J Vet Intern Med (2006) 20:151-163.
Methimazole pharmacokinetics in cats vs humans.
"""

import json
import sys
from datetime import datetime

import numpy as np


def euler_sim(Vmax: float, Km: float, Vd: float, dose: float, t_end: float, dt: float):
    """Euler: dC/dt = -Vmax*C/(Vd*(Km+C)), C0 = dose/Vd."""
    n = int(t_end / dt) + 1
    t = np.arange(n) * dt
    C = np.zeros(n)
    C[0] = dose / Vd
    for i in range(1, n):
        dC = -Vmax * C[i - 1] / (Vd * (Km + C[i - 1]))
        C[i] = C[i - 1] + dC * dt
        C[i] = max(0.0, C[i])
    return t, C


def apparent_t_half(C: float, Vmax: float, Km: float, Vd: float) -> float:
    """Apparent t1/2 = 0.693 * (Km + C) * Vd / Vmax."""
    return 0.693 * (Km + C) * Vd / Vmax


def Css(R: float, Vmax: float, Km: float) -> float:
    """Steady-state: Css = Km * R / (Vmax - R) for R < Vmax."""
    if R >= Vmax:
        return np.inf
    return Km * R / (Vmax - R)


def T4_response(t: np.ndarray, C: np.ndarray, baseline: float) -> np.ndarray:
    """T4(t) = 2.5 + (baseline - 2.5) * exp(-0.1 * C/(1+C) * t)."""
    factor = 0.1 * C / (1.0 + C)
    return 2.5 + (baseline - 2.5) * np.exp(-factor * t)


def main() -> None:
    dose = 2.5
    t_end = 24.0
    dt = 0.5
    baseline_T4 = 5.0

    params = {
        "feline": {"Vmax": 3.6, "Km": 1.5, "Vd": 1.2},
        "human": {"Vmax": 30.0, "Km": 2.0, "Vd": 40.0},
    }

    results = []
    for species, p in params.items():
        t, C = euler_sim(p["Vmax"], p["Km"], p["Vd"], dose, t_end, dt)
        t_half_0 = apparent_t_half(C[0], p["Vmax"], p["Km"], p["Vd"])
        t_half_end = apparent_t_half(C[-1], p["Vmax"], p["Km"], p["Vd"])
        R = dose / (t_end / 24.0)
        css = Css(R, p["Vmax"], p["Km"])
        T4 = T4_response(t, C, baseline_T4)

        results.append({
            "species": species,
            "Vmax": p["Vmax"],
            "Km": p["Km"],
            "Vd": p["Vd"],
            "dose_mg": dose,
            "time_hr": t.tolist(),
            "C_mg_L": C.tolist(),
            "apparent_t_half_0_hr": float(t_half_0),
            "apparent_t_half_end_hr": float(t_half_end),
            "Css_mg_L": float(css) if np.isfinite(css) else None,
            "T4_response": T4.tolist(),
        })

    out = {
        "results": results,
        "parameters": {"dose_mg": dose, "t_end_hr": t_end, "dt_hr": dt, "baseline_T4": baseline_T4},
        "_provenance": {
            "date": datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            "script": "control/comparative/exp106_feline_hyperthyroid.py",
            "command": " ".join(sys.argv),
            "git_commit": "baseline",
            "python": sys.version.split()[0],
            "numpy": np.__version__,
        },
    }
    print(json.dumps(out, indent=2))


if __name__ == "__main__":
    main()
