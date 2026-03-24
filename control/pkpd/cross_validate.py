# SPDX-License-Identifier: AGPL-3.0-or-later

#!/usr/bin/env python3
"""
Cross-validation: verify Python baseline JSON self-consistency.

Reads the Python baseline JSON files and checks that key numerical values
satisfy analytical identities, range constraints, and ordering invariants
(e.g. IC50 = E_max/2, AUC positive, monotonicity). Tolerances are drawn
from specs/TOLERANCE_REGISTRY.md. Covers experiments across all 7 tracks:
pkpd, microbiome, biosignal, endocrine, validation, comparative, discovery.

NOTE: This script does NOT compare Python vs Rust outputs. The Rust
experiment binaries (exp010, exp011, exp012, etc.) load baseline JSON and
compare their own outputs against it. This script validates the baselines
themselves against known analytical expectations.
"""

import json
import os
import sys

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
CONTROL_DIR = os.path.dirname(SCRIPT_DIR)

# Import centralized tolerances — single source of truth from control/tolerances.py
# which mirrors ecoPrimal/src/tolerances.rs exactly.
sys.path.insert(0, CONTROL_DIR)
import tolerances as tol_registry  # noqa: E402

TOL_MACHINE = tol_registry.MACHINE_EPSILON
TOL_MACHINE_LOOSE = tol_registry.DIVERSITY_CROSS_VALIDATE
TOL_NUMERICAL = tol_registry.HALF_LIFE_POINT
TOL_AUC = tol_registry.LEVEL_SPACING_RATIO
TOL_AUC_IV = 1.0  # 1% of ~100 for IV AUC numerical vs analytical
TOL_TMAX = tol_registry.TMAX_NUMERICAL
TOL_POPULATION = tol_registry.LOGNORMAL_RECOVERY
TOL_SHANNON = tol_registry.HALF_LIFE_POINT


def load_baseline(name, subdir=None):
    """Load baseline JSON. If subdir given, load from control/<subdir>/."""
    if subdir:
        path = os.path.join(CONTROL_DIR, subdir, name)
    else:
        path = os.path.join(SCRIPT_DIR, name)
    with open(path) as f:
        return json.load(f)


def check(label, python_val, rust_val, tol=TOL_MACHINE):
    diff = abs(float(python_val) - float(rust_val))
    ok = diff < tol
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] {label}: py={python_val}, rs={rust_val}, diff={diff:.2e}")
    return ok


def check_range(label, val, lo, hi):
    v = float(val)
    ok = lo <= v <= hi
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] {label}: {v} in [{lo}, {hi}]")
    return ok


def check_ordering(label, vals, expected_order="<"):
    """Check vals[0] < vals[1] < ... (or > for descending)."""
    floats = [float(v) for v in vals]
    if expected_order == "<":
        ok = all(floats[i] < floats[i + 1] for i in range(len(floats) - 1))
    else:
        ok = all(floats[i] > floats[i + 1] for i in range(len(floats) - 1))
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] {label}: {' < '.join(str(v) for v in floats)}")
    return ok


def main():
    passed = 0
    failed = 0

    print("=" * 72)
    print("Python ↔ Rust Cross-Validation (7 tracks)")
    print("=" * 72)

    # --- Exp001: Hill Dose-Response (pkpd) ---
    print("\n--- Exp001: Hill Dose-Response ---")
    b1 = load_baseline("exp001_baseline.json")

    for drug in ["baricitinib", "upadacitinib", "abrocitinib", "oclacitinib"]:
        ok = check(f"{drug} at IC50", b1[f"{drug}_at_ic50"], 0.5, tol=TOL_MACHINE)
        passed += ok
        failed += not ok

    ok = check("cooperativity n=1 below IC50", b1["cooperativity"]["n1"], 1.0 / 3.0, tol=TOL_MACHINE)
    passed += ok
    failed += not ok
    ok = check("cooperativity n=2 below IC50", b1["cooperativity"]["n2"], 0.2, tol=TOL_MACHINE)
    passed += ok
    failed += not ok

    for drug in ["baricitinib", "upadacitinib", "abrocitinib", "oclacitinib"]:
        ec = b1[f"{drug}_ec_values"]
        ok = ec["ec10"] < ec["ec50"] < ec["ec90"]
        status = "MATCH" if ok else "MISMATCH"
        print(f"  [{status}] {drug} EC ordering: {ec['ec10']:.2f} < {ec['ec50']:.2f} < {ec['ec90']:.2f}")
        passed += ok
        failed += not ok

    # --- Exp002: One-Compartment PK (pkpd) ---
    print("\n--- Exp002: One-Compartment PK ---")
    b2 = load_baseline("exp002_baseline.json")

    ok = check("IV C(0)", b2["iv_c0"], 10.0, tol=TOL_MACHINE)
    passed += ok
    failed += not ok

    ok = check("IV at half-life", b2["iv_at_half_life"], 5.0, tol=TOL_NUMERICAL)
    passed += ok
    failed += not ok

    ok = check("Oral C(0)", b2["oral_c0"], 0.0, tol=TOL_MACHINE)
    passed += ok
    failed += not ok

    ok = check("Oral Cmax", b2["oral_cmax"], 4.3105, tol=TOL_AUC)
    passed += ok
    failed += not ok

    ok = check("Oral Tmax", b2["oral_tmax"], 1.634, tol=TOL_TMAX)
    passed += ok
    failed += not ok

    ok = check("IV AUC (py vs analytical)", b2["iv_auc_numerical"], b2["iv_auc_analytical"], tol=TOL_AUC_IV)
    passed += ok
    failed += not ok

    ok = check("Oral AUC (py vs analytical)", b2["oral_auc"], b2["oral_auc_analytical"], tol=TOL_AUC)
    passed += ok
    failed += not ok

    # --- Exp003: Two-Compartment PK (pkpd) ---
    print("\n--- Exp003: Two-Compartment PK ---")
    b3 = load_baseline("exp003_baseline.json")

    ok = check("C(0)", b3["c0"], 16.0, tol=TOL_MACHINE_LOOSE)
    passed += ok
    failed += not ok

    ok = check("AUC numerical vs analytical", b3["auc_numerical"], b3["auc_analytical"], tol=TOL_AUC)
    passed += ok
    failed += not ok

    ok = check("A coeff", b3["A_coeff"], 14.4, tol=TOL_MACHINE_LOOSE)
    passed += ok
    failed += not ok

    ok = check("B coeff", b3["B_coeff"], 1.6, tol=TOL_AUC)
    passed += ok
    failed += not ok

    ok = check_range("alpha + beta", b3["sum_macro"], 1.0, 1.2)
    passed += ok
    failed += not ok

    ok = b3["all_nonneg"] and b3["monotonic_dec"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] structural: all_nonneg={b3['all_nonneg']}, monotonic_dec={b3['monotonic_dec']}")
    passed += ok
    failed += not ok

    # --- Exp004: mAb PK Transfer (pkpd) ---
    print("\n--- Exp004: mAb PK Cross-Species Transfer ---")
    b4 = load_baseline("exp004_baseline.json")

    ok = check_range("hl_scaled_days", b4["hl_scaled_days"], 15.0, 30.0)
    passed += ok
    failed += not ok

    ok = check_range("vd_scaled_L", b4["vd_scaled_L"], 3.0, 8.0)
    passed += ok
    failed += not ok

    ok = b4["hl_in_range"] and b4["vd_in_range"] and b4["all_nonneg"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] structural: hl_in_range, vd_in_range, all_nonneg")
    passed += ok
    failed += not ok

    ok = b4["loki_cmax"] > 0 and b4["nemo_cmax"] > 0 and b4["dupi_cmax"] > 0
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] Cmax positive: loki={b4['loki_cmax']:.2f}, nemo={b4['nemo_cmax']:.2f}, dupi={b4['dupi_cmax']:.2f}")
    passed += ok
    failed += not ok

    # --- Exp005: Population PK (pkpd) ---
    print("\n--- Exp005: Population PK Monte Carlo ---")
    b5 = load_baseline("exp005_baseline.json")

    ok = check_range("auc_median", b5["auc_median"], 0.2, 0.5)
    passed += ok
    failed += not ok

    ok = b5["auc_p5"] < b5["auc_p50"] < b5["auc_p95"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] AUC percentiles: p5 < p50 < p95")
    passed += ok
    failed += not ok

    ok = check("auc_median vs theoretical", b5["auc_median"], b5["auc_theoretical"], tol=TOL_POPULATION)
    passed += ok
    failed += not ok

    ok = check_range("cl_mean", b5["cl_mean"], 8.0, 12.0)
    passed += ok
    failed += not ok

    # --- Exp010: Diversity Indices (microbiome) ---
    print("\n--- Exp010: Microbiome Diversity Indices ---")
    b10 = load_baseline("exp010_baseline.json", "microbiome")

    sh = b10["shannon_all"]
    ok = sh["healthy_gut"] > sh["cdiff_colonized"] > sh["dysbiotic_gut"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] Shannon: healthy > cdiff > dysbiotic")
    passed += ok
    failed += not ok

    ok = sh["perfectly_even"] >= sh["healthy_gut"] > sh["monoculture"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] Shannon: even >= healthy > monoculture")
    passed += ok
    failed += not ok

    ok = b10["pielou_healthy"] > b10["pielou_cdiff"] > b10["pielou_dysbiotic"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] Pielou: healthy > cdiff > dysbiotic")
    passed += ok
    failed += not ok

    ok = b10["all_ranges_valid"] and b10["all_normalized"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] structural: all_ranges_valid, all_normalized")
    passed += ok
    failed += not ok

    # --- Exp011: Anderson Gut Lattice (microbiome) ---
    print("\n--- Exp011: Anderson Localization Gut Lattice ---")
    b11 = load_baseline("exp011_baseline.json", "microbiome")

    ipr = b11["ipr_means"]
    ok = ipr["healthy"] > ipr["moderate"] > ipr["recovering"] > ipr["dysbiotic"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] IPR: healthy > moderate > recovering > dysbiotic")
    passed += ok
    failed += not ok

    xi = b11["xi_means"]
    ok = xi["dysbiotic"] > xi["recovering"] > xi["moderate"] > xi["healthy"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] xi: dysbiotic > recovering > moderate > healthy")
    passed += ok
    failed += not ok

    ok = b11["cr_healthy"] > b11["cr_dysbiotic"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] CR: healthy > dysbiotic")
    passed += ok
    failed += not ok

    # --- Exp012: C. diff Resistance (microbiome) ---
    print("\n--- Exp012: C. diff Colonization Resistance ---")
    b12 = load_baseline("exp012_baseline.json", "microbiome")

    scores = b12["scores"]
    cr_vals = [scores[k]["cr"] for k in ["dysbiotic", "cdiff", "healthy", "post_fmt", "even"]]
    ok = cr_vals[0] < cr_vals[1] < cr_vals[2] < cr_vals[3] < cr_vals[4]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] CR ordering: dysbiotic < cdiff < healthy < post_fmt < even")
    passed += ok
    failed += not ok

    ok = scores["healthy"]["composite"] > scores["dysbiotic"]["composite"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] composite: healthy > dysbiotic")
    passed += ok
    failed += not ok

    # --- Exp020: Pan-Tompkins QRS (biosignal) ---
    print("\n--- Exp020: Pan-Tompkins QRS Detection ---")
    b20 = load_baseline("exp020_baseline.json", "biosignal")

    ok = b20["detection_metrics"]["sensitivity"] == 1.0 and b20["detection_metrics"]["ppv"] == 1.0
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] sensitivity=1.0, ppv=1.0")
    passed += ok
    failed += not ok

    ok = b20["n_detected"] == b20["n_true_peaks"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] n_detected == n_true_peaks ({b20['n_detected']})")
    passed += ok
    failed += not ok

    ok = check_range("hr_detected_bpm", b20["hr_detected"], 60.0, 90.0)
    passed += ok
    failed += not ok

    # --- Exp030: Testosterone IM PK (endocrine) ---
    print("\n--- Exp030: Testosterone IM Injection PK ---")
    b30 = load_baseline("exp030_baseline.json", "endocrine")

    ok = check("single C(0)", b30["single_c0"], 0.0, tol=TOL_MACHINE)
    passed += ok
    failed += not ok

    ok = b30["single_nonneg"] and b30["weekly_nonneg"] and b30["biweekly_nonneg"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] structural: all nonneg")
    passed += ok
    failed += not ok

    ok = check_range("single_cmax", b30["single_cmax"], 0.5, 2.0)
    passed += ok
    failed += not ok

    ok = b30["biweekly_cmax"] > b30["weekly_ss_cmax"] > b30["single_cmax"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] Cmax: biweekly > weekly > single")
    passed += ok
    failed += not ok

    # --- Exp031: Testosterone Pellet (endocrine) ---
    print("\n--- Exp031: Testosterone Pellet Depot PK ---")
    b31 = load_baseline("exp031_baseline.json", "endocrine")

    ok = check("C(0)", b31["c0"], 0.0, tol=TOL_MACHINE)
    passed += ok
    failed += not ok

    ok = check("AUC numerical vs analytical", b31["auc_numerical"], b31["auc_analytical"], tol=5.0)  # 10% of ~50
    passed += ok
    failed += not ok

    ok = b31["all_nonneg"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] structural: all_nonneg")
    passed += ok
    failed += not ok

    ok = check_range("c_mean_plateau", b31["c_mean_plateau"], 2.0, 2.5)
    passed += ok
    failed += not ok

    # --- Exp032: Age Testosterone Decline (endocrine) ---
    print("\n--- Exp032: Age-Related Testosterone Decline ---")
    b32 = load_baseline("exp032_baseline.json", "endocrine")

    ok = check("t_at_30", b32["t_at_30"], 600.0, tol=TOL_MACHINE)
    passed += ok
    failed += not ok

    ok = b32["monotonic_dec"] and b32["all_positive"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] structural: monotonic_dec, all_positive")
    passed += ok
    failed += not ok

    ok = b32["frac_hypogonadal_age50"] < b32["frac_hypogonadal_age60"] < b32["frac_hypogonadal_age70"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] hypogonadal fraction increases with age")
    passed += ok
    failed += not ok

    # --- Exp033: TRT Weight Trajectory (endocrine) ---
    print("\n--- Exp033: TRT Weight/Waist Trajectory ---")
    b33 = load_baseline("exp033_baseline.json", "endocrine")

    ok = check("dw_at_0", b33["dw_at_0"], 0.0, tol=TOL_MACHINE)
    passed += ok
    failed += not ok

    ok = check("dw_at_60mo", b33["dw_at_60mo"], -16.0, tol=TOL_MACHINE_LOOSE)
    passed += ok
    failed += not ok

    ok = b33["weight_mono_dec"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] structural: weight_mono_dec")
    passed += ok
    failed += not ok

    ok = b33["dw_at_60mo"] <= 0 and b33["dwc_at_60mo"] <= 0
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] loss at 60mo: dw={b33['dw_at_60mo']}, dwc={b33['dwc_at_60mo']}")
    passed += ok
    failed += not ok

    # --- Exp034: TRT Cardiovascular (endocrine) ---
    print("\n--- Exp034: TRT Cardiovascular Response ---")
    b34 = load_baseline("exp034_baseline.json", "endocrine")

    ok = check_range("ldl_60mo", b34["ldl_60mo"], 120.0, 140.0)
    passed += ok
    failed += not ok

    ok = check_range("hdl_60mo", b34["hdl_60mo"], 50.0, 60.0)
    passed += ok
    failed += not ok

    ok = b34["hdl_60mo"] > b34["params"]["HDL"]["baseline"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] HDL improves: {b34['params']['HDL']['baseline']} -> {b34['hdl_60mo']:.1f}")
    passed += ok
    failed += not ok

    ok = b34["all_smooth"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] structural: all_smooth")
    passed += ok
    failed += not ok

    # --- Exp035: TRT Diabetes (endocrine) ---
    print("\n--- Exp035: TRT and Type 2 Diabetes ---")
    b35 = load_baseline("exp035_baseline.json", "endocrine")

    ok = b35["hba1c_12mo"] < b35["hba1c_0"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] HbA1c improves: {b35['hba1c_0']} -> {b35['hba1c_12mo']:.2f}")
    passed += ok
    failed += not ok

    ok = b35["hba1c_monotonic"] and b35["all_improve"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] structural: hba1c_monotonic, all_improve")
    passed += ok
    failed += not ok

    ok = check_range("homa_12mo", b35["homa_12mo"], 2.5, 4.0)
    passed += ok
    failed += not ok

    # --- Exp036: Population TRT Monte Carlo (endocrine) ---
    print("\n--- Exp036: Population TRT Monte Carlo ---")
    b36 = load_baseline("exp036_baseline.json", "endocrine")

    ok = b36["auc_p5"] < b36["auc_p50"] < b36["auc_p95"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] AUC percentiles: p5 < p50 < p95")
    passed += ok
    failed += not ok

    ok = check_range("cmax_median", b36["cmax_median"], 1.5, 4.0)
    passed += ok
    failed += not ok

    ok = check_range("frac_hypogonadal", b36["frac_hypogonadal"], 0.1, 0.5)
    passed += ok
    failed += not ok

    ok = b36["weight_loss_mean"] < 0
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] weight_loss_mean < 0 ({b36['weight_loss_mean']:.1f})")
    passed += ok
    failed += not ok

    # --- Exp037: Testosterone-Gut Axis (endocrine) ---
    print("\n--- Exp037: Testosterone-Gut Axis ---")
    b37 = load_baseline("exp037_baseline.json", "endocrine")

    ok = b37["pielou_mean_healthy"] > b37["pielou_mean_dysbiotic"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] Pielou: healthy > dysbiotic")
    passed += ok
    failed += not ok

    ok = check_range("pielou_mean_healthy", b37["pielou_mean_healthy"], 0.9, 1.0)
    passed += ok
    failed += not ok

    ok = b37["response_mean_healthy"] < 0 and b37["response_mean_dysbiotic"] < 0
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] weight response negative (loss) for both")
    passed += ok
    failed += not ok

    ok = -1 <= b37["pielou_response_corr"] <= 1
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] correlation in [-1,1]: {b37['pielou_response_corr']:.3f}")
    passed += ok
    failed += not ok

    # --- Exp013: FMT rCDI (microbiome) ---
    print("\n--- Exp013: FMT Microbiota Transplant for rCDI ---")
    b13 = load_baseline("exp013_baseline.json", "microbiome")

    pre = b13["pre_fmt"]
    eng_07 = b13["engraftment_levels"]["0.7"]
    ok = eng_07["shannon"] > pre["shannon"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] FMT 0.7 improves Shannon: {pre['shannon']:.4f} -> {eng_07['shannon']:.4f}")
    passed += ok
    failed += not ok

    levels = ["0.3", "0.5", "0.7", "0.9"]
    shannons = [b13["engraftment_levels"][l]["shannon"] for l in levels]
    ok = all(shannons[i] < shannons[i + 1] for i in range(len(shannons) - 1))
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] Shannon monotonic with engraftment")
    passed += ok
    failed += not ok

    bc_donor = [b13["engraftment_levels"][l]["bray_curtis_to_donor"] for l in levels]
    ok = all(bc_donor[i] > bc_donor[i + 1] for i in range(len(bc_donor) - 1))
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] BC(post,donor) decreases with engraftment")
    passed += ok
    failed += not ok

    ok = eng_07["pielou"] > pre["pielou"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] Pielou improves post-FMT: {pre['pielou']:.4f} -> {eng_07['pielou']:.4f}")
    passed += ok
    failed += not ok

    # --- Exp021: HRV Metrics (biosignal) ---
    print("\n--- Exp021: HRV Metrics ---")
    b21 = load_baseline("exp021_baseline.json", "biosignal")

    ok = check_range("sdnn_ms", b21["sdnn_ms"], 0.0, 200.0)
    passed += ok
    failed += not ok

    ok = check_range("rmssd_ms", b21["rmssd_ms"], 0.0, 200.0)
    passed += ok
    failed += not ok

    ok = check_range("pnn50", b21["pnn50"], 0.0, 100.0)
    passed += ok
    failed += not ok

    ok = check_range("hr_detected", b21["hr_detected"], 60.0, 90.0)
    passed += ok
    failed += not ok

    ok = b21["n_detected"] >= 10 and b21["n_detected"] <= 14
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] n_detected={b21['n_detected']} in [10,14]")
    passed += ok
    failed += not ok

    # --- Exp022: PPG SpO2 (biosignal) ---
    print("\n--- Exp022: PPG SpO2 Calibration ---")
    b22 = load_baseline("exp022_baseline.json", "biosignal")

    rv = b22["spo2_r_values"]
    ok = check("SpO2 at R=0.4", rv["r_0.4"], 100.0)
    passed += ok
    failed += not ok

    ok = check("SpO2 at R=0.6", rv["r_0.6"], 95.0)
    passed += ok
    failed += not ok

    ok = check("SpO2 at R=0.8", rv["r_0.8"], 90.0)
    passed += ok
    failed += not ok

    ok = check("SpO2 at R=1.0", rv["r_1.0"], 85.0)
    passed += ok
    failed += not ok

    ppg = b22["synthetic_ppg"]
    ok = check_range("spo2_recovered_97", ppg["spo2_recovered_97"], 92.0, 102.0)
    passed += ok
    failed += not ok

    # --- Exp040: barraCuda CPU Parity (validation) ---
    print("\n--- Exp040: barraCuda CPU Parity ---")
    b40 = load_baseline("exp040_baseline.json", "validation")

    checks_40 = b40["checks"]
    ok = check("hill_e_at_ec50", checks_40["hill_e_at_ec50"], 0.5, tol=TOL_MACHINE)
    passed += ok
    failed += not ok

    ok = check("hill_e_at_zero", checks_40["hill_e_at_zero"], 0.0, tol=TOL_MACHINE)
    passed += ok
    failed += not ok

    ok = check("shannon_uniform", checks_40["shannon_uniform"], 2.302585, tol=TOL_SHANNON)
    passed += ok
    failed += not ok

    ok = check("pielou_uniform", checks_40["pielou_uniform"], 1.0, tol=TOL_MACHINE)
    passed += ok
    failed += not ok

    ok = check("bray_curtis_identical", checks_40["bray_curtis_identical"], 0.0, tol=TOL_MACHINE)
    passed += ok
    failed += not ok

    # --- Exp006: PBPK Compartments (pkpd) ---
    print("\n--- Exp006: PBPK Compartments ---")
    b6 = load_baseline("exp006_baseline.json")

    ok = check("PBPK C(0)", b6["c0_mg_per_L"], 20.0, tol=TOL_MACHINE_LOOSE)
    passed += ok
    failed += not ok

    ok = b6["auc_mg_hr_per_L"] > 0
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] AUC positive: {b6['auc_mg_hr_per_L']:.4f}")
    passed += ok
    failed += not ok

    ok = b6["c_24hr_mg_per_L"] < b6["c0_mg_per_L"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] C(24hr) < C(0): {b6['c_24hr_mg_per_L']:.4f} < {b6['c0_mg_per_L']:.4f}")
    passed += ok
    failed += not ok

    ok = b6["c_48hr_mg_per_L"] < b6["c_24hr_mg_per_L"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] C(48hr) < C(24hr): {b6['c_48hr_mg_per_L']:.4f} < {b6['c_24hr_mg_per_L']:.4f}")
    passed += ok
    failed += not ok

    # --- Exp023: Biosignal Fusion (biosignal) ---
    print("\n--- Exp023: Multi-Channel Biosignal Fusion ---")
    b23 = load_baseline("exp023_baseline.json", "biosignal")

    fus = b23["fusion"]
    ok = check_range("fusion HR", fus["heart_rate_bpm"], 60.0, 90.0)
    passed += ok
    failed += not ok

    ok = check_range("fusion SpO2", fus["spo2_percent"], 90.0, 100.0)
    passed += ok
    failed += not ok

    ok = check_range("stress_index", fus["stress_index"], 0.0, 1.0)
    passed += ok
    failed += not ok

    ok = check_range("overall_score", fus["overall_score"], 0.0, 100.0)
    passed += ok
    failed += not ok

    # --- Exp038: HRV × TRT Cardiovascular (endocrine) ---
    print("\n--- Exp038: HRV × TRT Cardiovascular ---")
    b38 = load_baseline("exp038_baseline.json", "endocrine")

    ok = check("SDNN at t=0", b38["sdnn_at_0"], b38["params"]["sdnn_base_ms"], tol=TOL_MACHINE)
    passed += ok
    failed += not ok

    ok = b38["sdnn_monotonic"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] SDNN monotonic improvement")
    passed += ok
    failed += not ok

    ok = b38["risk_post_trt"] < b38["risk_pre_trt"]
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] risk decreases: {b38['risk_pre_trt']:.3f} -> {b38['risk_post_trt']:.3f}")
    passed += ok
    failed += not ok

    ok = b38["risk_reduction_ratio"] > 0.5
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] risk reduction > 50%: {b38['risk_reduction_ratio']:.1%}")
    passed += ok
    failed += not ok

    # --- Exp090: MATRIX Drug Repurposing (discovery) ---
    print("\n--- Exp090: MATRIX Scoring ---")
    b90 = load_baseline("exp090_baseline.json", "discovery")

    compounds = b90["compounds"]
    ok = all(c["combined_score"] >= 0 for c in compounds)
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] all combined_scores >= 0")
    passed += ok
    failed += not ok

    ok = all(0.0 <= c["pathway_score"] <= 1.0 for c in compounds)
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] pathway_scores in [0,1]")
    passed += ok
    failed += not ok

    ok = all(0.0 <= c["tissue_geometry"] <= 1.0 for c in compounds)
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] tissue_geometry in [0,1]")
    passed += ok
    failed += not ok

    # --- Exp091: HTS Analysis (discovery) ---
    print("\n--- Exp091: ADDRC High-Throughput Screening ---")
    b91 = load_baseline("exp091_baseline.json", "discovery")

    ok = check_range("z_prime", b91["z_prime"], 0.5, 1.0)
    passed += ok
    failed += not ok

    signals = b91["signals"]
    ok = signals[0]["hit_classification"] == "Strong" and signals[-1]["hit_classification"] == "Inactive"
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] hit classification: strong hit at low signal, inactive at high")
    passed += ok
    failed += not ok

    ok = all(0.0 <= s["percent_inhibition"] <= 100.0 for s in signals)
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] percent_inhibition in [0,100]")
    passed += ok
    failed += not ok

    # --- Exp100: Canine IL-31 (comparative) ---
    print("\n--- Exp100: Canine IL-31 Serum Kinetics ---")
    b100 = load_baseline("exp100_baseline.json", "comparative")

    ok = check("baseline_pg_ml", b100["baseline_pg_ml"], 44.5, tol=TOL_MACHINE)
    passed += ok
    failed += not ok

    tc = b100["time_courses"]
    untreated_0 = tc["Untreated"]["il31_pg_ml"][0]
    ok = check("untreated t=0", untreated_0, 44.5, tol=TOL_MACHINE)
    passed += ok
    failed += not ok

    ocla_end = tc["Oclacitinib"]["il31_pg_ml"][-1]
    loki_end = tc["Lokivetmab"]["il31_pg_ml"][-1]
    ok = loki_end < ocla_end < untreated_0
    status = "MATCH" if ok else "MISMATCH"
    print(f"  [{status}] IL-31 at t_end: loki < ocla < untreated")
    passed += ok
    failed += not ok

    # --- Exp104: Cross-Species PK (comparative) ---
    print("\n--- Exp104: Cross-Species PK Scaling ---")
    b104 = load_baseline("exp104_baseline.json", "comparative")

    scalings = b104.get("scalings", [])
    if scalings:
        ok = all(s.get("cl_scaled", 0) > 0 for s in scalings)
        status = "MATCH" if ok else "MISMATCH"
        print(f"  [{status}] all scaled CL > 0")
        passed += ok
        failed += not ok
    else:
        print("  [SKIP] no scalings data")

    # --- Summary ---
    total = passed + failed
    print(f"\n{'=' * 72}")
    print(f"CROSS-VALIDATION: {passed}/{total} MATCH, {failed}/{total} MISMATCH")
    print(f"{'=' * 72}")

    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
