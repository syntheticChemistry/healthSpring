<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Primal-Proof IPC Mapping — healthSpring Domain → Precision Routes

**Last Updated**: May 13, 2026 (V64e)
**Wire Contract**: `primalSpring/docs/LIVE_SCIENCE_API.md`

This document maps healthSpring's domain science operations to their
`barracuda.precision.route` queries and `toadstool.validate` pre-flight
checks. When the `primal-proof` feature is active, these IPC calls replace
linked library paths.

---

## Precision Route Mapping

Each domain operation class maps to a `barracuda.precision.route` query
with a physics domain and optional hardware hint. The recommended tier
determines whether GPU dispatch needs coralReef shader compilation.

| healthSpring Domain | `precision.route` Domain | Expected Tier | FMA Safe | Compiler | Notes |
|---------------------|-------------------------|:-------------:|:--------:|:--------:|-------|
| **PK/PD** | | | | | |
| `science.pkpd.hill_dose_response` | `statistics` | F64 | yes | no | Sigmoid fits — F64 sufficient |
| `science.pkpd.one_compartment_pk` | `population_pk` | F64 | yes | no | ODE solve — standard precision |
| `science.pkpd.population_pk` | `population_pk` | F64 | yes | no | NLME parameter estimation |
| `science.pkpd.nlme_foce` | `population_pk` | F64 | yes | no | FOCE objective function |
| `science.pkpd.nlme_saem` | `population_pk` | F64 | yes | no | SAEM stochastic approximation |
| `science.pkpd.michaelis_menten_nonlinear` | `statistics` | F64 | yes | no | Nonlinear enzyme kinetics |
| **Microbiome** | | | | | |
| `science.microbiome.shannon_index` | `statistics` | F64 | yes | no | Log-sum — F64 for stability |
| `science.microbiome.anderson_gut` | `eigensolve` | F64 | yes | no | Disorder matrix eigendecomposition |
| `science.microbiome.chao1` | `statistics` | F32 | yes | no | Integer counting — F32 sufficient |
| `science.microbiome.bray_curtis` | `statistics` | F64 | yes | no | Dissimilarity metric |
| **Biosignal** | | | | | |
| `science.biosignal.pan_tompkins` | `statistics` | F32 | yes | no | QRS detection — time-domain filter |
| `science.biosignal.hrv_metrics` | `statistics` | F64 | yes | no | Frequency-domain HRV |
| `science.biosignal.arrhythmia_classification` | `inference` | F32 | yes | no | Template correlation |
| **Toxicology** | | | | | |
| `science.toxicology.toxicity_landscape` | `statistics` | F64 | yes | no | Multi-tissue accumulation model |
| `science.toxicology.biphasic_dose_response` | `statistics` | F64 | yes | no | Hormesis U-curve fitting |
| **Simulation** | | | | | |
| `science.simulation.mechanistic_fitness` | `molecular_dynamics` | F64 | no | yes | Fitness landscape simulation |
| `science.simulation.ecosystem_simulate` | `molecular_dynamics` | F64 | no | yes | Multi-species ODE system |

---

## Workload Pre-Flight Mapping

Each healthSpring workload TOML in `projectNUCLEUS/workloads/healthspring/`
is validated via `toadstool.validate` before dispatch.

| Workload TOML | Required Capabilities | GPU Required |
|---------------|----------------------|:------------:|
| `healthspring-pk-validation.toml` | `compute` | no |
| `healthspring-microbiome-validation.toml` | `compute` | no |
| `healthspring-biosignal-validation.toml` | `compute` | no |
| `healthspring-certification.toml` | `compute` | no |

---

## IPC Module Locations

| Upstream Method | healthSpring Module | Function |
|-----------------|-------------------|----------|
| `toadstool.validate` | `ipc::compute_dispatch` | `validate_workload()` |
| `toadstool.list_workloads` | `ipc::compute_dispatch` | `list_workloads()` |
| `compute.dispatch.submit` | `ipc::compute_dispatch` | `submit()` |
| `compute.dispatch.result` | `ipc::compute_dispatch` | `result()` |
| `compute.dispatch.capabilities` | `ipc::compute_dispatch` | `capabilities()` |
| `barracuda.precision.route` | `ipc::barracuda_client` | `BarraCudaClient::precision_route()` |
| `stats.mean` | `ipc::barracuda_client` | `BarraCudaClient::stats_mean()` |
| `stats.std_dev` | `ipc::barracuda_client` | `BarraCudaClient::stats_std_dev()` |
| `rng.normal` | `ipc::barracuda_client` | `BarraCudaClient::rng_normal()` |
| `crypto.contract.propose` | `ipc::tower_atomic` | `TowerAtomic::ionic_propose()` |
| `crypto.contract.countersign` | `ipc::tower_atomic` | `TowerAtomic::ionic_countersign()` |
| `crypto.contract.verify` | `ipc::tower_atomic` | `TowerAtomic::ionic_verify()` |

---

## Feature Gate

All IPC paths are available when `default = []` (IPC-first). The
`barracuda-lib` feature re-enables direct library linking for local
development and benchmarking. The `primal-proof` mode (IPC-only, no
library fallback) is the deployment target.
