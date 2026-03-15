# SPDX-License-Identifier: AGPL-3.0-only

#!/usr/bin/env python3
"""
healthSpring Exp013 — FMT (Fecal Microbiota Transplant) for rCDI

Models how a donor microbiome restores diversity and colonization
resistance in a recipient with recurrent C. diff infection (rCDI).

Model:
  1. Donor = healthy gut, Recipient = dysbiotic (rCDI)
  2. Post-FMT blend: (1 - engraftment)*recipient + engraftment*donor, normalized
  3. Diversity indices: Shannon, Simpson, Pielou
  4. Bray-Curtis dissimilarity: post vs donor, post vs recipient
  5. Colonization resistance: Pielou → Anderson disorder → ξ → CR

Engraftment levels: 0.3, 0.5, 0.7, 0.9

Reference:
  van Nood et al. 2013 (NEJM) — FMT for rCDI
  Buffie et al. 2015 (Nature) — colonization resistance

Provenance:
  Baseline date:   2026-03-08
  Command:         python3 control/microbiome/exp013_fmt_rcdi.py
  Environment:     Python 3.10+, NumPy, seed=42
"""

import json
import os
import sys

import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
SEED = 42
L = 100
N_SAMPLES = 30
W_SCALE = 10.0
ENGRAFTMENT_LEVELS = [0.3, 0.5, 0.7, 0.9]

# Same community profiles as barracuda
HEALTHY_GUT = np.array([0.25, 0.20, 0.15, 0.12, 0.10, 0.08, 0.05, 0.03, 0.01, 0.01])
DYSBIOTIC_GUT = np.array([0.85, 0.05, 0.03, 0.02, 0.02, 0.01, 0.01, 0.005, 0.003, 0.002])


def shannon(p):
    p = p[p > 0]
    return float(-np.sum(p * np.log(p))) if len(p) > 0 else 0.0


def simpson(p):
    return float(1.0 - np.sum(p ** 2))


def pielou(p):
    s = len(p)
    if s <= 1:
        return 0.0
    h = shannon(p)
    return float(h / np.log(s))


def fmt_blend(donor, recipient, engraftment):
    """Blended_i = (1-e)*recipient_i + e*donor_i, then normalized."""
    n = max(len(donor), len(recipient))
    blended = np.zeros(n)
    for i in range(n):
        d = donor[i] if i < len(donor) else 0.0
        r = recipient[i] if i < len(recipient) else 0.0
        blended[i] = (1.0 - engraftment) * r + engraftment * d
    total = np.sum(blended)
    if total > 0:
        blended /= total
    return blended


def bray_curtis(a, b):
    """BC = 1 - 2*Σ min(a_i, b_i) / (Σ a_i + Σ b_i). BC=0 identical, BC=1 different."""
    n = max(len(a), len(b))
    sum_min = sum_a = sum_b = 0.0
    for i in range(n):
        ai = a[i] if i < len(a) else 0.0
        bi = b[i] if i < len(b) else 0.0
        sum_min += min(ai, bi)
        sum_a += ai
        sum_b += bi
    denom = sum_a + sum_b
    return float(1.0 - 2.0 * sum_min / denom) if denom > 0 else 0.0


def build_anderson(L_size, W, rng, t=1.0):
    epsilon = rng.uniform(-W / 2, W / 2, L_size)
    H = np.diag(epsilon)
    for i in range(L_size - 1):
        H[i, i + 1] = t
        H[i + 1, i] = t
    return H


def ipr(psi):
    return float(np.sum(np.abs(psi) ** 4))


def xi_from_ipr(ipr_val):
    return 1.0 / ipr_val if ipr_val > 0 else float("inf")


def colonization_resistance(community, rng):
    """CR from Pielou → Anderson disorder → ξ → 1/ξ."""
    j = pielou(community)
    w = j * W_SCALE
    xis = []
    for _ in range(N_SAMPLES):
        H = build_anderson(L, w, rng)
        _, vecs = np.linalg.eigh(H)
        mid_state = vecs[:, L // 2]
        xi = xi_from_ipr(ipr(mid_state))
        xis.append(xi)
    mean_xi = np.mean(xis)
    return float(1.0 / mean_xi) if mean_xi > 0 else 0.0


def main():
    rng = np.random.default_rng(SEED)

    print("=" * 72)
    print("healthSpring Exp013: FMT for rCDI — Diversity & Colonization Resistance")
    print(f"  Donor=healthy, Recipient=dysbiotic, engraftment={ENGRAFTMENT_LEVELS}")
    print("=" * 72)

    # Pre-FMT (recipient) metrics
    pre_shannon = shannon(DYSBIOTIC_GUT)
    pre_simpson = simpson(DYSBIOTIC_GUT)
    pre_pielou = pielou(DYSBIOTIC_GUT)
    pre_cr = colonization_resistance(DYSBIOTIC_GUT, rng)
    pre_bc_donor = bray_curtis(DYSBIOTIC_GUT, HEALTHY_GUT)

    print(f"\n  Pre-FMT (recipient): H'={pre_shannon:.4f}  D={pre_simpson:.4f}  "
          f"J={pre_pielou:.4f}  CR={pre_cr:.4f}  BC(donor)={pre_bc_donor:.4f}")

    results = {
        "pre_fmt": {
            "shannon": pre_shannon,
            "simpson": pre_simpson,
            "pielou": pre_pielou,
            "cr": pre_cr,
            "bray_curtis_to_donor": pre_bc_donor,
        },
        "donor": {
            "shannon": shannon(HEALTHY_GUT),
            "simpson": simpson(HEALTHY_GUT),
            "pielou": pielou(HEALTHY_GUT),
            "cr": colonization_resistance(HEALTHY_GUT, rng),
        },
        "engraftment_levels": {},
    }

    for eng in ENGRAFTMENT_LEVELS:
        blended = fmt_blend(HEALTHY_GUT, DYSBIOTIC_GUT, eng)
        h = shannon(blended)
        d = simpson(blended)
        j = pielou(blended)
        cr = colonization_resistance(blended, rng)
        bc_donor = bray_curtis(blended, HEALTHY_GUT)
        bc_recipient = bray_curtis(blended, DYSBIOTIC_GUT)

        results["engraftment_levels"][str(eng)] = {
            "shannon": h,
            "simpson": d,
            "pielou": j,
            "cr": cr,
            "bray_curtis_to_donor": bc_donor,
            "bray_curtis_to_recipient": bc_recipient,
        }

        print(f"\n  Engraftment {eng:.1f}: H'={h:.4f}  D={d:.4f}  J={j:.4f}  "
              f"CR={cr:.4f}  BC(donor)={bc_donor:.4f}")

    # Write baseline
    baseline_path = os.path.join(SCRIPT_DIR, "exp013_baseline.json")
    baseline_out = {
        "_source": "healthSpring Exp013: FMT for rCDI",
        "_method": "fmt_blend, Bray-Curtis, Shannon/Simpson/Pielou, CR",
        "donor": "healthy_gut",
        "recipient": "dysbiotic_gut",
        "engraftment_levels": ENGRAFTMENT_LEVELS,
        **results,
        "_provenance": {
            "date": "2026-03-08",
            "python": sys.version,
            "numpy": np.__version__,
        },
    }
    with open(baseline_path, "w") as f:
        json.dump(baseline_out, f, indent=2, default=str)
    print(f"\nBaseline written to {baseline_path}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
