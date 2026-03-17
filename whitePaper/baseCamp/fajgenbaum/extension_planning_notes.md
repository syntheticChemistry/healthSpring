<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Fajgenbaum Extension Planning Notes

**Status**: Addressed in V21–V23. See EXTENSION_PLAN.md for the structured plan. Superseded by V35.

The original informal planning questions (dataset expansion, new systems, Anderson QS
gene profiling, compute/data hunger, LAN tower deployment, NestGate NCBI integration,
biomeOS orchestration) were addressed across V21 (domain evolution, data hunger profile),
V22 (biomeOS niche deployment, NestGate routing), and V23 (three-tier data fetch,
capability-based discovery).

Key deliverables:
- `specs/COMPUTE_DATA_PROFILE.md` — compute and data hunger assessment
- `specs/NESTGATE_DATA_PROVIDER.md` — NestGate integration design
- `specs/QS_GENE_PROFILING.md` — QS gene profiling design
- `data/fetch_qs_genes.py` — NCBI/UniProt/KEGG gene matrix fetcher
- `ecoPrimal/src/data/fetch.rs` — three-tier fetch (biomeOS → NestGate → local)
- `graphs/healthspring_niche_deploy.toml` — biomeOS niche deployment graph
