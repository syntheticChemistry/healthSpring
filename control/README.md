<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# healthSpring Python Control Baselines

Tier 0 reference implementations for all healthSpring experiments.
Every Rust validation binary cross-validates against the JSON baselines
produced by these scripts.

## Requirements

```
Python >= 3.10
numpy  == 2.1.3   (pinned in requirements.txt)
```

## Directory Layout

| Directory | Tracks | Experiments |
|-----------|--------|-------------|
| `pkpd/` | PK/PD | exp001–exp006 |
| `microbiome/` | Microbiome | exp010–exp013 |
| `biosignal/` | Biosignal | exp020–exp023 |
| `endocrine/` | Endocrinology | exp030–exp038 |
| `validation/` | barraCuda CPU parity | exp040 |
| `scripts/` | Benchmarks | CPU timing comparison |

## Regenerating Baselines

From the repository root:

```bash
# Single experiment
python3 control/pkpd/exp001_hill_dose_response.py

# All baselines (runs each script, overwrites JSON)
for py in control/pkpd/*.py control/microbiome/*.py \
          control/biosignal/*.py control/endocrine/*.py \
          control/validation/*.py; do
    [ "$(basename "$py")" = "cross_validate.py" ] && continue
    python3 "$py"
done

# Update provenance metadata (git commit, command, script path)
python3 control/update_provenance.py
```

## Provenance

Each `*_baseline.json` contains a `_provenance` block:

```json
{
  "_provenance": {
    "date": "2026-03-08",
    "script": "control/pkpd/exp001_hill_dose_response.py",
    "command": "python3 control/pkpd/exp001_hill_dose_response.py",
    "git_commit": "e93a81b6bdd85d537eb5a3904847c1cfc603ab28",
    "python": "3.10.12",
    "numpy": "2.1.3"
  }
}
```

`update_provenance.py` refreshes `git_commit`, `command`, and `script`
from the current HEAD without re-running computations.

## Cross-Validation

```bash
python3 control/pkpd/cross_validate.py
```

Loads each baseline JSON and compares values against the Rust binary
outputs. Reports 104 checks across all 24 Tier 0+1 experiments.

## Benchmarks

```bash
python3 control/scripts/bench_barracuda_cpu_vs_python.py [--n_iterations 100]
```

Writes `bench_results_python.json` for Tier 0 vs Tier 1 timing comparison.
The Rust mirror is `cargo bench --bench cpu_parity` (Criterion).

## Tier 2/3 Experiments (No Python Baseline)

Experiments exp053–exp076 validate Rust-internal parity (CPU vs GPU,
dispatch routing, pipeline topology, petalTongue scenarios). These are
Tier 2/3 checks that do not require Python baselines — correctness is
established by Tier 1 parity, then GPU/dispatch checks confirm compute
substrate independence.
