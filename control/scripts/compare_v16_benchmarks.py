# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
"""
healthSpring Exp084 — Benchmark Comparison: Rust CPU vs Python

Reads bench_results_v16_rust_cpu.json and bench_results_v16_python.json,
prints a side-by-side speedup table.
"""
import json, os, sys

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

def load(name):
    path = os.path.join(SCRIPT_DIR, name)
    with open(path) as f:
        return json.load(f)

def main():
    rust = load("bench_results_v16_rust_cpu.json")
    python = load("bench_results_v16_python.json")

    rust_by_name = {b["name"]: b for b in rust["benchmarks"]}
    python_by_name = {b["name"]: b for b in python["benchmarks"]}

    common = sorted(set(rust_by_name) & set(python_by_name))

    print("=" * 90)
    print("Exp084: Rust CPU vs Python — V16 Primitive Speedup Table")
    print("=" * 90)
    print(f"{'Benchmark':<44} {'Python (us)':>12} {'Rust (us)':>12} {'Speedup':>10}")
    print("-" * 90)

    total_py = total_rs = 0.0
    for name in common:
        py_mean = python_by_name[name]["mean_us"]
        rs_mean = rust_by_name[name]["mean_us"]
        speedup = py_mean / rs_mean if rs_mean > 0 else float("inf")
        total_py += py_mean
        total_rs += rs_mean
        print(f"  {name:<42} {py_mean:>12.1f} {rs_mean:>12.1f} {speedup:>9.1f}x")

    print("-" * 90)
    overall = total_py / total_rs if total_rs > 0 else float("inf")
    print(f"  {'TOTAL':<42} {total_py:>12.1f} {total_rs:>12.1f} {overall:>9.1f}x")
    print("=" * 90)
    print(f"\nRust CPU is {overall:.0f}x faster than Python across all V16 primitives (mean).")
    return 0

if __name__ == "__main__":
    sys.exit(main())
