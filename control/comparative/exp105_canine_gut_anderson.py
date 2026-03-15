#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""
Canine gut Anderson diversity baseline.

Reference: Anderson et al., Appl Environ Microbiol (2006) 72:5445-5456.
Distance-based diversity metrics for microbial communities.
"""

import json
import sys
from datetime import datetime

import numpy as np


def shannon(p: np.ndarray) -> float:
    """Shannon entropy: -sum(p_i * ln(p_i))."""
    p = np.array(p)
    p = p[p > 0]
    return float(-np.sum(p * np.log(p)))


def pielou(p: np.ndarray) -> float:
    """Pielou's evenness: shannon(p) / ln(len(p))."""
    p = np.array(p)
    n = len(p[p > 0])
    if n <= 1:
        return 0.0
    return shannon(p) / np.log(n)


def wellness_score(p: np.ndarray) -> float:
    """W = 10.0 * pielou, capped at 10.0."""
    return min(10.0, 10.0 * pielou(p))


def main() -> None:
    communities = {
        "healthy": [0.20, 0.18, 0.17, 0.16, 0.15, 0.14],
        "ad_dog": [0.60, 0.15, 0.10, 0.08, 0.05, 0.02],
        "treated": [0.30, 0.20, 0.18, 0.15, 0.10, 0.07],
    }

    results = []
    for name, p in communities.items():
        p_arr = np.array(p)
        s = shannon(p_arr)
        pie = pielou(p_arr)
        w = wellness_score(p_arr)
        results.append({
            "community": name,
            "abundances": p,
            "shannon": s,
            "pielou": pie,
            "W": w,
        })

    # Pairwise comparisons
    comparisons = []
    names = list(communities.keys())
    for i in range(len(names)):
        for j in range(i + 1, len(names)):
            a, b = names[i], names[j]
            s_a = shannon(np.array(communities[a]))
            s_b = shannon(np.array(communities[b]))
            comparisons.append({
                "community_a": a,
                "community_b": b,
                "shannon_diff": s_a - s_b,
                "pielou_diff": pielou(np.array(communities[a])) - pielou(np.array(communities[b])),
            })

    out = {
        "communities": results,
        "comparisons": comparisons,
        "_provenance": {
            "date": datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            "script": "control/comparative/exp105_canine_gut_anderson.py",
            "command": " ".join(sys.argv),
            "git_commit": "baseline",
            "python": sys.version.split()[0],
            "numpy": np.__version__,
        },
    }
    print(json.dumps(out, indent=2))


if __name__ == "__main__":
    main()
