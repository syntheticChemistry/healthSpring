# SPDX-License-Identifier: AGPL-3.0-only
#!/usr/bin/env python3
"""
healthSpring Exp084 — V16 CPU Parity Benchmarks (Python Baseline)

Benchmarks the 6 V16 primitives in Python for direct comparison with Rust.
Outputs bench_results_v16_python.json.

Primitives: Michaelis-Menten PK, antibiotic perturbation, SCFA production,
gut-brain serotonin, EDA stress detection, arrhythmia beat classification.
"""
import json, os, sys, time
import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
N_ITER = 100

def bench(name, func, n_iter=N_ITER):
    times = []
    for _ in range(n_iter):
        t0 = time.perf_counter_ns()
        func()
        times.append((time.perf_counter_ns() - t0) / 1000.0)
    times.sort()
    return {
        "name": name, "n_iterations": n_iter,
        "mean_us": sum(times) / len(times),
        "min_us": times[0], "max_us": times[-1],
        "p50_us": times[len(times)//2],
        "p95_us": times[int(len(times)*0.95)],
    }

# ── 1. Michaelis-Menten PK ──────────────────────────────────────────

VMAX, KM, VD = 500.0, 5.0, 50.0

def mm_simulate(dose, t_end, dt):
    n = int(t_end / dt)
    c = np.zeros(n + 1)
    c[0] = dose / VD
    for i in range(n):
        elim = VMAX * c[i] / (KM + c[i])
        c[i+1] = max(0, c[i] - (elim / VD) * dt)
    return c

def mm_auc_analytical(dose):
    c0 = dose / VD
    return KM * c0 * VD / VMAX + c0**2 * VD / (2 * VMAX)

def mm_half_life(conc):
    return 0.693 * (KM + conc) * VD / VMAX

# ── 2. Antibiotic Perturbation ───────────────────────────────────────

def antibiotic_perturbation(h0, depth, k_dec, k_rec, treat_days, total_days, dt):
    n = int(total_days / dt)
    h_nadir = h0 * (1 - depth)
    result = []
    for i in range(n + 1):
        t = i * dt
        if t <= treat_days:
            h = h0 - (h0 - h_nadir) * (1 - np.exp(-k_dec * t))
        else:
            t_rec = t - treat_days
            h_at_end = h0 - (h0 - h_nadir) * (1 - np.exp(-k_dec * treat_days))
            recovery_target = h0 * (1 - depth * 0.15)
            h = h_at_end + (recovery_target - h_at_end) * (1 - np.exp(-k_rec * t_rec))
        result.append((t, h))
    return result

# ── 3. SCFA Production ───────────────────────────────────────────────

def scfa(fiber, vmax_a, km_a, vmax_p, km_p, vmax_b, km_b):
    mm = lambda v, k: v * fiber / (k + fiber)
    return mm(vmax_a, km_a), mm(vmax_p, km_p), mm(vmax_b, km_b)

# ── 4. Gut-Brain Serotonin ───────────────────────────────────────────

def serotonin(trp, shannon_h, k_synth=0.01, scale=0.5):
    h_ref = 1.5
    diversity_factor = 1.0 / (1.0 + np.exp(-(shannon_h - h_ref) / scale))
    return k_synth * trp * diversity_factor

def trp_availability(trp, shannon_h):
    frac = 0.4 + 0.4 / (1 + np.exp(-3 * (shannon_h - 1.5)))
    return trp * frac

# ── 5. EDA Stress Detection ─────────────────────────────────────────

def eda_scl(signal, window):
    return np.convolve(signal, np.ones(window)/window, mode='same')

def eda_phasic(signal, window):
    return signal - eda_scl(signal, window)

def eda_detect_scr(phasic, threshold, min_dist):
    peaks = []
    for i in range(1, len(phasic)-1):
        if phasic[i] > threshold and phasic[i] > phasic[i-1] and phasic[i] > phasic[i+1]:
            if not peaks or (i - peaks[-1]) >= min_dist:
                peaks.append(i)
    return peaks

# ── 6. Arrhythmia Classification ────────────────────────────────────

def normal_template(n):
    x = np.linspace(-3, 3, n)
    return np.exp(-x**2 / 0.5)

def pvc_template(n):
    x = np.linspace(-3, 3, n)
    return -np.exp(-x**2 / 0.8)

def pac_template(n):
    x = np.linspace(-3, 3, n)
    return np.exp(-x**2 / 0.3) * 0.7

def norm_corr(a, b):
    a = np.array(a); b = np.array(b)
    ma = a - a.mean(); mb = b - b.mean()
    na = np.sqrt(np.sum(ma**2)); nb = np.sqrt(np.sum(mb**2))
    if na < 1e-15 or nb < 1e-15:
        return 0.0
    return float(np.sum(ma * mb) / (na * nb))

def classify_beat(beat, templates, min_corr=0.5):
    best_name = "Unknown"
    best_corr = -999.0
    for name, tmpl in templates:
        c = norm_corr(beat, tmpl)
        if c > best_corr:
            best_corr = c
            best_name = name
    return (best_name, best_corr) if best_corr >= min_corr else ("Unknown", best_corr)


def main():
    passed = failed = 0
    benchmarks = []

    print("=" * 72)
    print("healthSpring Exp084: V16 CPU Parity Benchmarks (Python)")
    print("=" * 72)

    # ── 1. Michaelis-Menten PK
    print("\n── 1. Michaelis-Menten PK ──────────────────────────────────────")
    r = bench("mm_pk_simulate_300mg_10d", lambda: mm_simulate(300, 10, 0.001))
    print(f"  {r['name']:<42} mean={r['mean_us']:.1f}us  p95={r['p95_us']:.1f}us")
    benchmarks.append(r)

    r = bench("mm_auc_analytical_x100", lambda: [mm_auc_analytical(d*5+10) for d in range(100)])
    print(f"  {r['name']:<42} mean={r['mean_us']:.1f}us  p95={r['p95_us']:.1f}us")
    benchmarks.append(r)

    r = bench("mm_half_life_sweep_100", lambda: [mm_half_life(i*0.3+0.1) for i in range(100)])
    print(f"  {r['name']:<42} mean={r['mean_us']:.1f}us  p95={r['p95_us']:.1f}us")
    benchmarks.append(r)

    c0 = 300 / VD
    ok = abs(c0 - 6.0) < 0.01
    print(f"  [{'PASS' if ok else 'FAIL'}] mm_c0_correct")
    passed += ok; failed += (not ok)

    auc = mm_auc_analytical(300)
    ok = auc > 0
    print(f"  [{'PASS' if ok else 'FAIL'}] mm_auc_positive")
    passed += ok; failed += (not ok)

    ok = mm_half_life(1) < mm_half_life(20)
    print(f"  [{'PASS' if ok else 'FAIL'}] mm_half_life_dose_dependent")
    passed += ok; failed += (not ok)

    # ── 2. Antibiotic Perturbation
    print("\n── 2. Antibiotic Perturbation ──────────────────────────────────")
    r = bench("antibiotic_perturb_30d", lambda: antibiotic_perturbation(2.2, 0.7, 5.0, 0.1, 5.0, 30.0, 0.01))
    print(f"  {r['name']:<42} mean={r['mean_us']:.1f}us  p95={r['p95_us']:.1f}us")
    benchmarks.append(r)

    r = bench("antibiotic_perturb_365d", lambda: antibiotic_perturbation(2.2, 0.7, 5.0, 0.1, 5.0, 365.0, 0.1))
    print(f"  {r['name']:<42} mean={r['mean_us']:.1f}us  p95={r['p95_us']:.1f}us")
    benchmarks.append(r)

    traj = antibiotic_perturbation(2.2, 0.7, 5.0, 0.1, 5.0, 30.0, 0.01)
    h_vals = [h for _, h in traj]
    ok = min(h_vals) < h_vals[0]
    print(f"  [{'PASS' if ok else 'FAIL'}] antibiotic_drops")
    passed += ok; failed += (not ok)

    ok = h_vals[-1] > min(h_vals)
    print(f"  [{'PASS' if ok else 'FAIL'}] antibiotic_recovers")
    passed += ok; failed += (not ok)

    ok = h_vals[-1] < h_vals[0]
    print(f"  [{'PASS' if ok else 'FAIL'}] antibiotic_not_full_recovery")
    passed += ok; failed += (not ok)

    # ── 3. SCFA Production
    print("\n── 3. SCFA Production ─────────────────────────────────────────")
    r = bench("scfa_healthy_x1000", lambda: [scfa(i*0.05+0.1, 60, 8, 20, 10, 15, 12) for i in range(1000)])
    print(f"  {r['name']:<42} mean={r['mean_us']:.1f}us  p95={r['p95_us']:.1f}us")
    benchmarks.append(r)

    r = bench("scfa_dysbiotic_x1000", lambda: [scfa(i*0.05+0.1, 55, 8, 18, 10, 5, 15) for i in range(1000)])
    print(f"  {r['name']:<42} mean={r['mean_us']:.1f}us  p95={r['p95_us']:.1f}us")
    benchmarks.append(r)

    a, p, b = scfa(20, 60, 8, 20, 10, 15, 12)
    total = a + p + b
    ok = a > p > b
    print(f"  [{'PASS' if ok else 'FAIL'}] scfa_acetate_dominant")
    passed += ok; failed += (not ok)
    ok = 0.50 < a/total < 0.70
    print(f"  [{'PASS' if ok else 'FAIL'}] scfa_ratio_normal")
    passed += ok; failed += (not ok)
    _, _, b_d = scfa(20, 55, 8, 18, 10, 5, 15)
    ok = b > b_d
    print(f"  [{'PASS' if ok else 'FAIL'}] scfa_dysbiotic_less_butyrate")
    passed += ok; failed += (not ok)

    # ── 4. Gut-Brain Serotonin
    print("\n── 4. Gut-Brain Serotonin ─────────────────────────────────────")
    r = bench("serotonin_sweep_1000", lambda: [serotonin(i*0.1+10, 0.5+i*0.002) for i in range(1000)])
    print(f"  {r['name']:<42} mean={r['mean_us']:.1f}us  p95={r['p95_us']:.1f}us")
    benchmarks.append(r)

    r = bench("tryptophan_availability_sweep_1000", lambda: [trp_availability(i*0.1+10, 0.5+i*0.002) for i in range(1000)])
    print(f"  {r['name']:<42} mean={r['mean_us']:.1f}us  p95={r['p95_us']:.1f}us")
    benchmarks.append(r)

    h5 = serotonin(50, 2.2); d5 = serotonin(50, 0.8)
    ok = h5 > 0 and d5 > 0
    print(f"  [{'PASS' if ok else 'FAIL'}] serotonin_positive")
    passed += ok; failed += (not ok)
    ok = h5 > d5
    print(f"  [{'PASS' if ok else 'FAIL'}] serotonin_diversity_dependent")
    passed += ok; failed += (not ok)
    ok = trp_availability(100, 2.2) > trp_availability(100, 0.8)
    print(f"  [{'PASS' if ok else 'FAIL'}] tryptophan_higher_healthy")
    passed += ok; failed += (not ok)

    # ── 5. EDA Stress Detection
    print("\n── 5. EDA Stress Detection ────────────────────────────────────")
    eda_sig = np.array([2.0 + 0.5*np.sin(i/100*0.3) + (1.5 if i%300<30 else 0) for i in range(2000)])

    r = bench("eda_scl_2000_samples", lambda: eda_scl(eda_sig, 200))
    print(f"  {r['name']:<42} mean={r['mean_us']:.1f}us  p95={r['p95_us']:.1f}us")
    benchmarks.append(r)

    r = bench("eda_phasic_2000_samples", lambda: eda_phasic(eda_sig, 200))
    print(f"  {r['name']:<42} mean={r['mean_us']:.1f}us  p95={r['p95_us']:.1f}us")
    benchmarks.append(r)

    ph = eda_phasic(eda_sig, 200)
    r = bench("eda_detect_scr_2000", lambda: eda_detect_scr(ph, 0.05, 30))
    print(f"  {r['name']:<42} mean={r['mean_us']:.1f}us  p95={r['p95_us']:.1f}us")
    benchmarks.append(r)

    peaks = eda_detect_scr(ph, 0.05, 30)
    ok = len(peaks) > 0
    print(f"  [{'PASS' if ok else 'FAIL'}] eda_scr_found")
    passed += ok; failed += (not ok)

    # ── 6. Arrhythmia Classification
    print("\n── 6. Arrhythmia Beat Classification ──────────────────────────")
    n_tmpl = normal_template(60)
    p_tmpl = pvc_template(60)
    a_tmpl = pac_template(60)
    tmpls = [("Normal", n_tmpl), ("PVC", p_tmpl), ("PAC", a_tmpl)]

    r = bench("classify_beat_x1000", lambda: [classify_beat(n_tmpl, tmpls) for _ in range(1000)])
    print(f"  {r['name']:<42} mean={r['mean_us']:.1f}us  p95={r['p95_us']:.1f}us")
    benchmarks.append(r)

    r = bench("normalized_correlation_x1000", lambda: [norm_corr(n_tmpl, p_tmpl) for _ in range(1000)])
    print(f"  {r['name']:<42} mean={r['mean_us']:.1f}us  p95={r['p95_us']:.1f}us")
    benchmarks.append(r)

    cls, corr = classify_beat(n_tmpl, tmpls)
    ok = cls == "Normal" and corr > 0.99
    print(f"  [{'PASS' if ok else 'FAIL'}] classify_normal_as_normal (corr={corr:.4f})")
    passed += ok; failed += (not ok)

    cls_p, _ = classify_beat(p_tmpl, tmpls)
    ok = cls_p == "PVC"
    print(f"  [{'PASS' if ok else 'FAIL'}] classify_pvc_as_pvc")
    passed += ok; failed += (not ok)

    sc = norm_corr(n_tmpl, n_tmpl)
    ok = abs(sc - 1.0) < 1e-10
    print(f"  [{'PASS' if ok else 'FAIL'}] self_correlation_is_1")
    passed += ok; failed += (not ok)

    cc = norm_corr(n_tmpl, p_tmpl)
    ok = cc < 0.8
    print(f"  [{'PASS' if ok else 'FAIL'}] cross_corr_less_than_1")
    passed += ok; failed += (not ok)

    # ── Summary
    suite = {"tier": "python_v16", "experiment": "exp084", "benchmarks": benchmarks,
             "_provenance": {"date": "2026-03-10", "python": sys.version, "numpy": np.__version__}}
    path = os.path.join(SCRIPT_DIR, "bench_results_v16_python.json")
    with open(path, "w") as f:
        json.dump(suite, f, indent=2)
    print(f"\nResults written to {path}")

    total = passed + failed
    print(f"\n{'='*72}")
    print(f"Exp084 V16 CPU Parity (Python): {passed}/{total} PASS, {failed}/{total} FAIL")
    print(f"{'='*72}")
    return 0 if failed == 0 else 1

if __name__ == "__main__":
    sys.exit(main())
