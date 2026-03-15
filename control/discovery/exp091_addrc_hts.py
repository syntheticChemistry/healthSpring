#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Exp091: ADDRC high-throughput screening analysis (Lisabeth 2024)

Implements Z'-factor, SSMD, percent inhibition, and hit classification
for a synthetic 96-well plate with known controls.

Reference: Zhang JH et al. J Biomol Screen 4:67 (1999)
"""

import json
import math
import sys
from datetime import datetime, timezone

import numpy as np


def z_prime(pos_mean: float, pos_std: float, neg_mean: float, neg_std: float) -> float:
    """Z' = 1 - 3(pos_std + neg_std) / |pos_mean - neg_mean|"""
    separation = abs(pos_mean - neg_mean)
    if separation < 1e-15:
        return float("-inf")
    return 1.0 - 3.0 * (pos_std + neg_std) / separation


def ssmd(
    sample_mean: float, sample_std: float, neg_mean: float, neg_std: float
) -> float:
    """SSMD = (sample_mean - neg_mean) / sqrt(sample_std² + neg_std²)"""
    denom = math.sqrt(sample_std**2 + neg_std**2)
    if denom < 1e-15:
        return 0.0
    return (sample_mean - neg_mean) / denom


def percent_inhibition(signal: float, pos_mean: float, neg_mean: float) -> float:
    """100 * (neg_mean - signal) / (neg_mean - pos_mean)"""
    range_val = neg_mean - pos_mean
    if abs(range_val) < 1e-15:
        return 0.0
    return 100.0 * (neg_mean - signal) / range_val


def classify_hit(ssmd_abs: float) -> str:
    """Strong |SSMD|>3, Moderate 2-3, Weak 1-2, Inactive <1"""
    if ssmd_abs > 3.0:
        return "Strong"
    if ssmd_abs > 2.0:
        return "Moderate"
    if ssmd_abs > 1.0:
        return "Weak"
    return "Inactive"


def main() -> None:
    pos_mean = 10.0
    pos_std = 2.0
    neg_mean = 90.0
    neg_std = 3.0

    signals = [15.0, 45.0, 70.0, 88.0]
    compound_stds = [1.0, 5.0, 3.0, 2.0]

    zp = z_prime(pos_mean, pos_std, neg_mean, neg_std)

    results = []
    for i, (signal, cstd) in enumerate(zip(signals, compound_stds)):
        pct = percent_inhibition(signal, pos_mean, neg_mean)
        ssmd_val = ssmd(signal, cstd, neg_mean, neg_std)
        classification = classify_hit(abs(ssmd_val))
        results.append({
            "index": i,
            "signal": signal,
            "sample_std": cstd,
            "percent_inhibition": pct,
            "ssmd": ssmd_val,
            "hit_classification": classification,
        })

    out = {
        "experiment": "exp091_addrc_hts",
        "plate_params": {
            "pos_mean": pos_mean,
            "pos_std": pos_std,
            "neg_mean": neg_mean,
            "neg_std": neg_std,
        },
        "z_prime": zp,
        "signals": results,
        "_provenance": {
            "date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
            "script": "control/discovery/exp091_addrc_hts.py",
            "command": " ".join(sys.argv) or f"python3 {__file__}",
            "git_commit": "baseline",
            "python_version": sys.version.split()[0],
            "numpy_version": np.__version__,
        },
    }

    print(json.dumps(out, indent=2))


if __name__ == "__main__":
    main()
