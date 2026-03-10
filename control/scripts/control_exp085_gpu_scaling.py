# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
"""
healthSpring Exp085 — GPU Scaling Validation (Python control)

Cross-validates the scaling results from Exp085:
- MM batch AUC correctness at reference scale
- SCFA production correctness at reference scale
- Beat classification correctness for known templates
- Linear scaling expectation for embarrassingly parallel ops
"""
import json, os, sys
import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

VMAX, KM, VD = 500.0, 5.0, 50.0

def mm_simulate_one(dose, dt, n_steps, vmax_eff):
    c = np.zeros(n_steps + 1)
    c[0] = dose / VD
    for i in range(n_steps):
        elim = vmax_eff * c[i] / (KM + c[i])
        c[i + 1] = max(0, c[i] - (elim / VD) * dt)
    return np.trapezoid(c, dx=dt)

def scfa(fiber, vmax_a=60, km_a=8, vmax_p=20, km_p=10, vmax_b=15, km_b=12):
    mm = lambda v, k: v * fiber / (k + fiber)
    return mm(vmax_a, km_a), mm(vmax_p, km_p), mm(vmax_b, km_b)

def norm_corr(a, b):
    a, b = np.array(a), np.array(b)
    ma, mb = a - a.mean(), b - b.mean()
    na, nb = np.sqrt((ma**2).sum()), np.sqrt((mb**2).sum())
    if na < 1e-15 or nb < 1e-15:
        return 0.0
    return float((ma * mb).sum() / (na * nb))

def normal_template(n):
    x = np.linspace(-3, 3, n)
    return np.exp(-x**2 / 0.5)

def pvc_template(n):
    x = np.linspace(-3, 3, n)
    return -np.exp(-x**2 / 0.8)

def pac_template(n):
    x = np.linspace(-3, 3, n)
    return np.exp(-x**2 / 0.3) * 0.7

def main():
    passed = failed = 0
    print("=" * 72)
    print("healthSpring Exp085: GPU Scaling Validation (Python)")
    print("=" * 72)

    # MM: reference AUC at known parameters
    auc = mm_simulate_one(300, 0.01, 2000, VMAX)
    ok = auc > 0
    print(f"  [{'PASS' if ok else 'FAIL'}] MM reference AUC > 0 ({auc:.2f})")
    passed += ok; failed += (not ok)

    ok = 1.0 < auc < 200.0
    print(f"  [{'PASS' if ok else 'FAIL'}] MM AUC physiological range")
    passed += ok; failed += (not ok)

    # SCFA: healthy ratios
    a, p, b = scfa(20)
    total = a + p + b
    ok = a > p > b
    print(f"  [{'PASS' if ok else 'FAIL'}] SCFA acetate dominant")
    passed += ok; failed += (not ok)

    ok = 0.50 < a / total < 0.70
    print(f"  [{'PASS' if ok else 'FAIL'}] SCFA ratio normal")
    passed += ok; failed += (not ok)

    ok = all(v > 0 for v in scfa(0.1))
    print(f"  [{'PASS' if ok else 'FAIL'}] SCFA all positive at low fiber")
    passed += ok; failed += (not ok)

    # SCFA: monotone with fiber
    s5 = sum(scfa(5))
    s50 = sum(scfa(50))
    ok = s50 > s5
    print(f"  [{'PASS' if ok else 'FAIL'}] SCFA monotone with fiber ({s5:.1f} < {s50:.1f})")
    passed += ok; failed += (not ok)

    # Beat classify: self-correlation
    n_tmpl = normal_template(41)
    p_tmpl = pvc_template(41)
    ok = abs(norm_corr(n_tmpl, n_tmpl) - 1.0) < 1e-10
    print(f"  [{'PASS' if ok else 'FAIL'}] Self-correlation = 1.0")
    passed += ok; failed += (not ok)

    cc = norm_corr(n_tmpl, p_tmpl)
    ok = cc < 0.8
    print(f"  [{'PASS' if ok else 'FAIL'}] Normal-PVC cross-corr < 0.8 ({cc:.3f})")
    passed += ok; failed += (not ok)

    # Scaling linearity check from Rust results
    rust_path = os.path.join(SCRIPT_DIR, "bench_results_v16_gpu_scaling.json")
    if os.path.exists(rust_path):
        with open(rust_path) as f:
            rust = json.load(f)
        mm_benches = [r for r in rust["results"] if r["op"] == "mm_batch"]
        if len(mm_benches) >= 2:
            scale_ratio = mm_benches[-1]["scale"] / mm_benches[0]["scale"]
            time_ratio = mm_benches[-1]["mean_us"] / mm_benches[0]["mean_us"]
            ok = time_ratio < scale_ratio * 2.0
            print(f"  [{'PASS' if ok else 'FAIL'}] MM scaling subquadratic ({time_ratio:.1f}× time for {scale_ratio:.0f}× scale)")
            passed += ok; failed += (not ok)

        scfa_benches = [r for r in rust["results"] if r["op"] == "scfa_batch"]
        if len(scfa_benches) >= 2:
            scale_ratio = scfa_benches[-1]["scale"] / scfa_benches[0]["scale"]
            time_ratio = scfa_benches[-1]["mean_us"] / scfa_benches[0]["mean_us"]
            ok = time_ratio < scale_ratio * 2.0
            print(f"  [{'PASS' if ok else 'FAIL'}] SCFA scaling subquadratic ({time_ratio:.1f}× time for {scale_ratio:.0f}× scale)")
            passed += ok; failed += (not ok)
    else:
        print("  [SKIP] No Rust scaling results found")

    total_c = passed + failed
    print(f"\n{'='*72}")
    print(f"Exp085 GPU Scaling (Python): {passed}/{total_c} PASS, {failed}/{total_c} FAIL")
    print(f"{'='*72}")
    return 0 if failed == 0 else 1

if __name__ == "__main__":
    sys.exit(main())
