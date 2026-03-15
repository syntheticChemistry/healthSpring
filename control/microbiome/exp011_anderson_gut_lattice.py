# SPDX-License-Identifier: AGPL-3.0-only

#!/usr/bin/env python3
"""
healthSpring Exp011 — Anderson Localization in Gut Lattice

Validates the Anderson localization framework applied to a 1D gut
microbiome lattice. The core thesis: colonization resistance is an
Anderson localization phenomenon — a diverse microbiome confines
pathogenic signals, while dysbiosis breaks localization.

Model:
  - 1D tight-binding Anderson lattice: H = diag(ε_i) + t*(off-diagonal)
  - On-site energies ε_i sampled from uniform(-W/2, W/2)
  - W (disorder) mapped from Pielou evenness of gut community
  - Hopping t = 1.0 (normalized)
  - Localization length ξ from inverse participation ratio (IPR)

In 1D, all states are localized for any W > 0 (proven theorem).
Higher W → shorter ξ → better confinement.

Connection:
  - wetSpring Exp107: Anderson localization in soil (same physics)
  - neuralSpring nS-604: three-compartment tissue lattice (AD skin)
  - healthSpring: gut microbiome → colonization resistance score

Reference:
  Anderson PW (1958), Thouless DJ (1972)
  Lozupone & Knight (2005) for microbiome diversity

Provenance:
  Baseline date:   2026-03-08
  Command:         python3 control/microbiome/exp011_anderson_gut_lattice.py
  Environment:     Python 3.10+, NumPy, seed=42
"""

import json
import os
import sys

import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
SEED = 42
L = 200          # lattice size
N_SAMPLES = 50   # disorder realizations
T_HOP = 1.0      # hopping parameter


def build_anderson_1d(L, W, rng, t=1.0):
    """Build 1D Anderson tight-binding Hamiltonian."""
    epsilon = rng.uniform(-W / 2, W / 2, L)
    H = np.diag(epsilon)
    for i in range(L - 1):
        H[i, i + 1] = t
        H[i + 1, i] = t
    return H


def inverse_participation_ratio(psi):
    """IPR = Σ |ψ_i|^4. Localized → IPR ~ 1/ξ, extended → IPR ~ 1/L."""
    return float(np.sum(np.abs(psi) ** 4))


def localization_length_from_ipr(ipr):
    """ξ ≈ 1 / (IPR * L) ... but simpler: ξ ≈ 1/IPR for normalized states."""
    return 1.0 / ipr if ipr > 0 else float("inf")


def level_spacing_ratio(eigenvalues):
    """Mean level spacing ratio <r> from ordered eigenvalues.

    r_n = min(s_n, s_{n+1}) / max(s_n, s_{n+1})
    Poisson (localized): <r> ≈ 0.386
    GOE (extended): <r> ≈ 0.5307
    """
    spacings = np.diff(np.sort(eigenvalues))
    if len(spacings) < 2:
        return 0.0
    r_vals = []
    for i in range(len(spacings) - 1):
        s_n = spacings[i]
        s_n1 = spacings[i + 1]
        if max(s_n, s_n1) > 0:
            r_vals.append(min(s_n, s_n1) / max(s_n, s_n1))
    return float(np.mean(r_vals)) if r_vals else 0.0


def pielou_to_disorder(evenness, w_scale=10.0):
    """Map Pielou evenness J → Anderson disorder W."""
    return evenness * w_scale


def main():
    total_passed = 0
    total_failed = 0
    baseline = {}
    rng = np.random.default_rng(SEED)

    print("=" * 72)
    print("healthSpring Exp011: Anderson Localization in Gut Lattice")
    print(f"  L={L}, N_samples={N_SAMPLES}, seed={SEED}")
    print("=" * 72)

    # Disorder values: low (dysbiotic), medium (recovering), high (healthy)
    W_VALUES = [1.0, 3.0, 6.0, 10.0]
    W_LABELS = ["dysbiotic", "recovering", "moderate", "healthy"]

    ipr_means = {}
    xi_means = {}
    r_means = {}

    for W, label in zip(W_VALUES, W_LABELS):
        iprs = []
        xis = []
        rs = []
        for _ in range(N_SAMPLES):
            H = build_anderson_1d(L, W, rng, T_HOP)
            eigvals, eigvecs = np.linalg.eigh(H)
            mid = L // 2
            psi_mid = eigvecs[:, mid]
            ipr = inverse_participation_ratio(psi_mid)
            xi = localization_length_from_ipr(ipr)
            r = level_spacing_ratio(eigvals)
            iprs.append(ipr)
            xis.append(xi)
            rs.append(r)
        ipr_means[label] = float(np.mean(iprs))
        xi_means[label] = float(np.mean(xis))
        r_means[label] = float(np.mean(rs))

    baseline["ipr_means"] = ipr_means
    baseline["xi_means"] = xi_means
    baseline["r_means"] = r_means

    # ------------------------------------------------------------------
    # Check 1: All IPR > 0
    # ------------------------------------------------------------------
    print("\n--- Check 1: All IPR > 0 ---")
    if all(v > 0 for v in ipr_means.values()):
        print(f"  [PASS] " + ", ".join(f"{k}={v:.6f}" for k, v in ipr_means.items()))
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 2: Higher disorder → higher IPR (more localized)
    # ------------------------------------------------------------------
    print("\n--- Check 2: Higher W → higher IPR ---")
    ipr_list = [ipr_means[l] for l in W_LABELS]
    monotonic = all(ipr_list[i] <= ipr_list[i + 1] * 1.5 for i in range(len(ipr_list) - 1))
    if ipr_list[-1] > ipr_list[0]:
        print(f"  [PASS] IPR(healthy)={ipr_list[-1]:.6f} > IPR(dysbiotic)={ipr_list[0]:.6f}")
        total_passed += 1
    else:
        print(f"  [FAIL] {ipr_list}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 3: Higher disorder → shorter localization length
    # ------------------------------------------------------------------
    print("\n--- Check 3: Higher W → shorter ξ ---")
    xi_list = [xi_means[l] for l in W_LABELS]
    if xi_list[0] > xi_list[-1]:
        print(f"  [PASS] ξ(dysbiotic)={xi_list[0]:.1f} > ξ(healthy)={xi_list[-1]:.1f}")
        total_passed += 1
    else:
        print(f"  [FAIL] {xi_list}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 4: Level spacing ratio → Poisson for strong disorder
    # ------------------------------------------------------------------
    print("\n--- Check 4: Strong disorder → Poisson statistics ---")
    R_POISSON = 0.386
    r_healthy = r_means["healthy"]
    if abs(r_healthy - R_POISSON) < 0.08:
        print(f"  [PASS] <r>(W=10) = {r_healthy:.4f} ≈ Poisson({R_POISSON})")
        total_passed += 1
    else:
        print(f"  [FAIL] <r> = {r_healthy:.4f}, expected ~{R_POISSON}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 5: Weak disorder → closer to GOE
    # ------------------------------------------------------------------
    print("\n--- Check 5: Weak disorder → closer to GOE ---")
    R_GOE = 0.5307
    r_dysbiotic = r_means["dysbiotic"]
    if r_dysbiotic > r_healthy:
        print(f"  [PASS] <r>(W=1)={r_dysbiotic:.4f} > <r>(W=10)={r_healthy:.4f} (toward GOE)")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 6: ξ(W=1) >> ξ(W=10) — weak disorder much more extended
    # ------------------------------------------------------------------
    print("\n--- Check 6: ξ ratio weak/strong >> 1 ---")
    xi_ratio = xi_list[0] / xi_list[-1] if xi_list[-1] > 0 else 0
    baseline["xi_ratio_weak_strong"] = float(xi_ratio)
    if xi_ratio > 2.0:
        print(f"  [PASS] ξ(W=1)/ξ(W=10) = {xi_ratio:.2f}")
        total_passed += 1
    else:
        print(f"  [FAIL] ratio = {xi_ratio:.2f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 7: Pielou → disorder mapping produces valid W range
    # ------------------------------------------------------------------
    print("\n--- Check 7: Pielou → W mapping ---")
    j_healthy = 0.863  # from Exp010
    j_dysbiotic = 0.303
    w_h = pielou_to_disorder(j_healthy)
    w_d = pielou_to_disorder(j_dysbiotic)
    baseline["pielou_w_healthy"] = w_h
    baseline["pielou_w_dysbiotic"] = w_d
    if w_h > w_d and w_h > 0 and w_d > 0:
        print(f"  [PASS] W(healthy)={w_h:.2f} > W(dysbiotic)={w_d:.2f}")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 8: Eigenvalue count = L
    # ------------------------------------------------------------------
    print("\n--- Check 8: Eigenvalue count = L ---")
    H_test = build_anderson_1d(L, 5.0, rng, T_HOP)
    eigvals_test = np.linalg.eigvalsh(H_test)
    if len(eigvals_test) == L:
        print(f"  [PASS] {len(eigvals_test)} eigenvalues for L={L}")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 9: Hamiltonian is symmetric
    # ------------------------------------------------------------------
    print("\n--- Check 9: Hamiltonian symmetric ---")
    if np.allclose(H_test, H_test.T):
        print(f"  [PASS] H = H^T")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 10: Eigenvectors normalized
    # ------------------------------------------------------------------
    print("\n--- Check 10: Eigenvectors normalized ---")
    _, vecs = np.linalg.eigh(H_test)
    norms = np.sum(vecs ** 2, axis=0)
    if np.allclose(norms, 1.0, atol=1e-10):
        print(f"  [PASS] all |ψ|² = 1.0")
        total_passed += 1
    else:
        print(f"  [FAIL] max deviation = {np.max(np.abs(norms - 1.0)):.2e}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 11: Colonization resistance interpretation
    # ------------------------------------------------------------------
    print("\n--- Check 11: Colonization resistance score ---")
    cr_healthy = 1.0 / xi_means["healthy"]  # higher ξ⁻¹ = more localized = more resistant
    cr_dysbiotic = 1.0 / xi_means["dysbiotic"]
    baseline["cr_healthy"] = float(cr_healthy)
    baseline["cr_dysbiotic"] = float(cr_dysbiotic)
    if cr_healthy > cr_dysbiotic:
        print(f"  [PASS] CR(healthy)={cr_healthy:.4f} > CR(dysbiotic)={cr_dysbiotic:.4f}")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 12: W=0 → all states extended (delocalized)
    # ------------------------------------------------------------------
    print("\n--- Check 12: W=0 → extended states ---")
    H_clean = build_anderson_1d(L, 0.0, rng, T_HOP)
    _, vecs_clean = np.linalg.eigh(H_clean)
    ipr_clean = inverse_participation_ratio(vecs_clean[:, L // 2])
    xi_clean = localization_length_from_ipr(ipr_clean)
    baseline["xi_W0"] = float(xi_clean)
    if xi_clean > xi_means["healthy"] * 2:
        print(f"  [PASS] ξ(W=0)={xi_clean:.1f} >> ξ(W=10)={xi_means['healthy']:.1f}")
        total_passed += 1
    else:
        print(f"  [FAIL] ξ(W=0)={xi_clean:.1f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Write baseline
    # ------------------------------------------------------------------
    baseline_path = os.path.join(SCRIPT_DIR, "exp011_baseline.json")
    baseline_out = {
        "_source": "healthSpring Exp011: Anderson Localization in Gut Lattice",
        "_method": "1D tight-binding, uniform disorder, eigh, IPR, level spacing",
        "lattice_size": L,
        "n_samples": N_SAMPLES,
        "w_values": W_VALUES,
        "seed": SEED,
        **baseline,
        "_provenance": {
            "date": "2026-03-08",
            "python": sys.version,
            "numpy": np.__version__,
        },
    }
    with open(baseline_path, "w") as f:
        json.dump(baseline_out, f, indent=2, default=str)
    print(f"\nBaseline written to {baseline_path}")

    total = total_passed + total_failed
    print(f"\n{'=' * 72}")
    print(f"TOTAL: {total_passed}/{total} PASS, {total_failed}/{total} FAIL")
    print(f"{'=' * 72}")

    return 0 if total_failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
