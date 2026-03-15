# SPDX-License-Identifier: AGPL-3.0-only
#!/usr/bin/env python3
"""
healthSpring Exp077 — Michaelis-Menten Nonlinear PK (Python Baseline)

Capacity-limited elimination: dC/dt = -Vmax*C/(Km + C).
Phenytoin-like parameters: Vmax=500 mg/day, Km=5 mg/L, Vd=50 L.

Reference: Rowland & Tozer Ch. 20, Ludden et al. 1977.
"""
import json, os, sys
import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

VMAX = 500.0   # mg/day
KM = 5.0       # mg/L
VD = 50.0      # L

def mm_simulate(dose, t_end, dt):
    n = int(t_end / dt)
    t = np.zeros(n + 1)
    c = np.zeros(n + 1)
    c[0] = dose / VD
    for i in range(n):
        elim = VMAX * c[i] / (KM + c[i])
        c[i+1] = max(0, c[i] - (elim / VD) * dt)
        t[i+1] = t[i] + dt
    return t, c

def mm_css(rate):
    if rate >= VMAX:
        return None
    return rate * KM / (VMAX - rate)

def mm_half_life(conc):
    return 0.693 * (KM + conc) * VD / VMAX

def mm_auc_analytical(dose):
    c0 = dose / VD
    return KM * c0 * VD / VMAX + c0**2 * VD / (2 * VMAX)

def main():
    passed = failed = 0
    baseline = {}
    print("=" * 72)
    print("healthSpring Exp077: Michaelis-Menten PK (Python)")
    print("=" * 72)

    # Check 1: C0 = dose/Vd
    t, c = mm_simulate(300, 1, 0.001)
    c0 = 300 / VD
    ok = abs(c[0] - c0) < 1e-10
    print(f"\n  Check 1: C0={c[0]:.2f} ≈ {c0:.2f}: {'PASS' if ok else 'FAIL'}")
    passed += ok; failed += (not ok)

    # Check 2: Monotone
    t10, c10 = mm_simulate(300, 10, 0.001)
    ok = all(c10[i+1] <= c10[i] + 1e-14 for i in range(len(c10)-1))
    print(f"  Check 2: Monotone decline: {'PASS' if ok else 'FAIL'}")
    passed += ok; failed += (not ok)

    # Check 3: Dose-dependent t½
    t_low = mm_half_life(1.0)
    t_mid = mm_half_life(5.0)
    t_high = mm_half_life(20.0)
    ok = t_low < t_mid < t_high
    print(f"  Check 3: t½: {t_low:.2f} < {t_mid:.2f} < {t_high:.2f}: {'PASS' if ok else 'FAIL'}")
    passed += ok; failed += (not ok)

    # Check 4: Low-dose linearity
    auc10 = mm_auc_analytical(10)
    auc20 = mm_auc_analytical(20)
    ratio = (auc20/auc10) / 2.0
    ok = abs(ratio - 1.0) < 0.15
    print(f"  Check 4: Low-dose ratio={ratio:.3f} ≈ 1.0: {'PASS' if ok else 'FAIL'}")
    passed += ok; failed += (not ok)

    # Check 5: High-dose supralinear
    auc200 = mm_auc_analytical(200)
    auc400 = mm_auc_analytical(400)
    ratio_h = (auc400/auc200) / 2.0
    ok = ratio_h > 1.0
    print(f"  Check 5: High-dose ratio={ratio_h:.3f} > 1.0: {'PASS' if ok else 'FAIL'}")
    passed += ok; failed += (not ok)

    # Check 6: Css exists
    css = mm_css(250)
    ok = css is not None and css > 0
    print(f"  Check 6: Css(250)={css:.2f}: {'PASS' if ok else 'FAIL'}")
    passed += ok; failed += (not ok)

    # Check 7: No Css at Vmax
    ok = mm_css(500) is None
    print(f"  Check 7: No Css at Vmax: {'PASS' if ok else 'FAIL'}")
    passed += ok; failed += (not ok)

    # Check 8: Css steep near Vmax
    css100 = mm_css(100)
    css400 = mm_css(400)
    ok = css400 / css100 > 4.0
    print(f"  Check 8: Css steep: {css400/css100:.1f}×: {'PASS' if ok else 'FAIL'}")
    passed += ok; failed += (not ok)

    # Check 9: Numerical AUC ≈ analytical
    _, c_long = mm_simulate(300, 20, 0.0001)
    num_auc = np.trapz(c_long, dx=0.0001)
    anal_auc = mm_auc_analytical(300)
    rel_err = abs(num_auc - anal_auc) / anal_auc
    ok = rel_err < 0.02
    print(f"  Check 9: AUC num={num_auc:.2f} anal={anal_auc:.2f} err={rel_err:.4f}: {'PASS' if ok else 'FAIL'}")
    passed += ok; failed += (not ok)

    # Check 10: AUC supralinear
    ok = auc400 / auc200 > 2.0
    print(f"  Check 10: AUC(400)/AUC(200) = {auc400/auc200:.3f} > 2: {'PASS' if ok else 'FAIL'}")
    passed += ok; failed += (not ok)

    baseline.update({
        "c0": float(c[0]), "auc_analytical_300": float(anal_auc),
        "css_250": float(css), "t_half_low": float(t_low),
        "t_half_high": float(t_high), "nonlinearity_ratio_high": float(ratio_h),
    })

    path = os.path.join(SCRIPT_DIR, "exp077_baseline.json")
    with open(path, "w") as f:
        json.dump({"_source": "Exp077 MM PK", **baseline,
                    "_provenance": {"date": "2026-03-10", "python": sys.version,
                                    "numpy": np.__version__}}, f, indent=2)
    print(f"\nBaseline: {path}")
    total = passed + failed
    print(f"\n{'='*72}\nTOTAL: {passed}/{total} PASS, {failed}/{total} FAIL\n{'='*72}")
    return 0 if failed == 0 else 1

if __name__ == "__main__":
    sys.exit(main())
