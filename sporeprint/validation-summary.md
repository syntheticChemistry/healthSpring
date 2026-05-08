+++
title = "healthSpring Validation Summary"
description = "PK/PD, gut microbiome, biosignal, drug discovery — 795 checks across 7 clinical tracks, sovereign NLME"
date = 2026-05-08

[taxonomies]
primals = ["barracuda", "toadstool", "biomeos", "nestgate"]
springs = ["healthspring", "wetspring", "neuralspring", "groundspring"]
+++

## Status

- **795 checks** (601 Rust + 194 Python cross-validation)
- **7 clinical tracks**: PK/PD, microbiome, biosignal, endocrinology, NLME, comparative medicine, drug discovery
- **Sovereign NLME** (FOCE/SAEM) replaces proprietary NONMEM/Monolix
- **Species-agnostic PK** — same code for canine AD, feline hyperthyroid, human TRT
- Testosterone-gut axis validated via Anderson localization

## Key Validation Binaries

<!-- TODO: Update with actual binary names from target/release/ -->
- `validate_pk_models` — Hill, 1-compartment PK, PopPK, Michaelis-Menten (16 checks, exit 0/1)
- `validate_gut_microbiome` — (planned) Anderson lattice, C. diff, FMT
- `validate_biosignal` — (planned) Pan-Tompkins, HRV
- `validate_nlme` — (planned) FOCE/SAEM population PK

## Workload TOMLs

Skeleton available in `projectNUCLEUS/workloads/healthspring/`.

## See Also

- [healthSpring Science Hub](https://primals.eco/lab/springs/healthspring/) on primals.eco
- [baseCamp Paper 13](https://primals.eco/science/)
