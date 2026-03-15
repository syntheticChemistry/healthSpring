#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
"""
healthSpring Python CPU Benchmark — barraCuda CPU Parity Timing

Measures Python reference computation times for the core operations that
healthSpring implements in Rust. groundSpring pattern: 3 tiers measured
independently, results written to JSON for cross-comparison.

  Tier 0: Python baseline (this script)
  Tier 1: Rust CPU (cargo run --release --bin exp040_barracuda_cpu_parity)
  Tier 2: barraCuda GPU (LIVE — 3 WGSL shaders via wgpu, Exp053-055)

Usage:
    python3 control/scripts/bench_barracuda_cpu_vs_python.py [--n_iterations 100]

Output:
    control/scripts/bench_results_python.json
"""

import json
import math
import os
import sys
import time

try:
    import numpy as np
except ImportError:
    print("NumPy required: pip install numpy", file=sys.stderr)
    sys.exit(1)


SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
DEFAULT_ITERATIONS = 100


def hill_equation(conc, ic50, hill_n, e_max=1.0):
    c_n = conc**hill_n
    ic50_n = ic50**hill_n
    return e_max * c_n / (c_n + ic50_n)


def hill_sweep_python(concs, ic50, hill_n, e_max):
    return [hill_equation(c, ic50, hill_n, e_max) for c in concs]


def pk_oral_one_compartment(dose, f_bio, vd, ka, ke, t):
    if abs(ka - ke) < 1e-12:
        return dose * f_bio / vd * ka * t * math.exp(-ke * t)
    return dose * f_bio * ka / (vd * (ka - ke)) * (math.exp(-ke * t) - math.exp(-ka * t))


def pk_curve_python(dose, f_bio, vd, ka, ke, n_points=100, t_max=24.0):
    dt = t_max / n_points
    times = [i * dt for i in range(n_points + 1)]
    concs = [pk_oral_one_compartment(dose, f_bio, vd, ka, ke, t) for t in times]
    return times, concs


def shannon_index(abundances):
    return -sum(p * math.log(p) for p in abundances if p > 0)


def simpson_index(abundances):
    return 1.0 - sum(p * p for p in abundances)


def pielou_evenness(abundances):
    s = len([p for p in abundances if p > 0])
    if s <= 1:
        return 0.0
    h = shannon_index(abundances)
    return h / math.log(s)


def auc_trapezoidal(times, concs):
    total = 0.0
    for i in range(1, len(times)):
        dt = times[i] - times[i - 1]
        total += 0.5 * (concs[i] + concs[i - 1]) * dt
    return total


def population_montecarlo_python(n_patients, dose, f_bio, vd_base, ka, bw_ref, seed=42):
    """Run n_patients virtual patients with lognormal IIV on weight."""
    rng = np.random.default_rng(seed)
    risks = []
    for _ in range(n_patients):
        bw = rng.lognormal(math.log(70.0), 0.20)
        cl = 0.15 * (bw / bw_ref) ** 0.75
        vd = vd_base * (bw / bw_ref) ** 1.0
        ke = cl / vd
        _, concs = pk_curve_python(dose, f_bio, vd, ka, ke)
        cmax = max(concs)
        auc = auc_trapezoidal(
            [i * 24.0 / 100 for i in range(101)], concs
        )
        risk = min(1.0, max(0.0, cmax * 0.5 + (1.0 - auc / 10.0) * 0.5))
        risks.append(risk)
    return risks


def bench(name, func, n_iterations):
    """Run a benchmark and return timing stats."""
    times_s = []
    result = None
    for _ in range(n_iterations):
        start = time.perf_counter()
        result = func()
        elapsed = time.perf_counter() - start
        times_s.append(elapsed)

    arr = np.array(times_s)
    return {
        "name": name,
        "n_iterations": n_iterations,
        "mean_ms": float(arr.mean() * 1000),
        "std_ms": float(arr.std() * 1000),
        "min_ms": float(arr.min() * 1000),
        "max_ms": float(arr.max() * 1000),
        "p50_ms": float(np.median(arr) * 1000),
        "p95_ms": float(np.percentile(arr, 95) * 1000),
    }, result


def main():
    n_iter = DEFAULT_ITERATIONS
    if "--n_iterations" in sys.argv:
        idx = sys.argv.index("--n_iterations")
        n_iter = int(sys.argv[idx + 1])

    print("=" * 72)
    print("healthSpring Python CPU Benchmark — barraCuda CPU Parity Timing")
    print(f"Iterations per benchmark: {n_iter}")
    print("=" * 72)

    benchmarks = []

    # 1. Hill sweep (50 concentrations)
    concs_50 = [0.1 * (1000.0 ** (i / 49)) for i in range(50)]
    stats, _ = bench(
        "hill_sweep_50",
        lambda: hill_sweep_python(concs_50, 10.0, 1.5, 100.0),
        n_iter,
    )
    benchmarks.append(stats)
    print(f"  {stats['name']:40s} mean={stats['mean_ms']:.4f}ms  p95={stats['p95_ms']:.4f}ms")

    # 2. PK curve (101 points)
    cl = 0.15 * (85.0 / 70.0) ** 0.75
    vd = 15.0 * (85.0 / 70.0) ** 1.0
    ke = cl / vd
    stats, _ = bench(
        "pk_curve_101_points",
        lambda: pk_curve_python(4.0, 0.79, vd, 1.5, ke),
        n_iter,
    )
    benchmarks.append(stats)
    print(f"  {stats['name']:40s} mean={stats['mean_ms']:.4f}ms  p95={stats['p95_ms']:.4f}ms")

    # 3. Shannon/Simpson/Pielou (7 genera)
    abund = [0.35, 0.25, 0.20, 0.10, 0.05, 0.03, 0.02]
    stats, _ = bench(
        "diversity_indices_7_genera",
        lambda: (shannon_index(abund), simpson_index(abund), pielou_evenness(abund)),
        n_iter,
    )
    benchmarks.append(stats)
    print(f"  {stats['name']:40s} mean={stats['mean_ms']:.4f}ms  p95={stats['p95_ms']:.4f}ms")

    # 4. AUC trapezoidal (101 points)
    times_pk, concs_pk = pk_curve_python(4.0, 0.79, vd, 1.5, ke)
    stats, _ = bench(
        "auc_trapezoidal_101_points",
        lambda: auc_trapezoidal(times_pk, concs_pk),
        n_iter,
    )
    benchmarks.append(stats)
    print(f"  {stats['name']:40s} mean={stats['mean_ms']:.4f}ms  p95={stats['p95_ms']:.4f}ms")

    # 5. Population Monte Carlo (500 patients)
    stats, _ = bench(
        "population_montecarlo_500",
        lambda: population_montecarlo_python(500, 4.0, 0.79, 15.0, 1.5, 70.0),
        max(1, n_iter // 10),
    )
    benchmarks.append(stats)
    print(f"  {stats['name']:40s} mean={stats['mean_ms']:.4f}ms  p95={stats['p95_ms']:.4f}ms")

    # 6. Population Monte Carlo (5000 patients)
    stats, _ = bench(
        "population_montecarlo_5000",
        lambda: population_montecarlo_python(5000, 4.0, 0.79, 15.0, 1.5, 70.0),
        max(1, n_iter // 20),
    )
    benchmarks.append(stats)
    print(f"  {stats['name']:40s} mean={stats['mean_ms']:.4f}ms  p95={stats['p95_ms']:.4f}ms")

    results = {
        "tier": "python_cpu",
        "python_version": sys.version.split()[0],
        "numpy_version": np.__version__,
        "n_iterations": n_iter,
        "benchmarks": benchmarks,
    }

    out_path = os.path.join(SCRIPT_DIR, "bench_results_python.json")
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(results, f, indent=2)

    print(f"\nResults written to {out_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
