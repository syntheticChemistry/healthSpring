# healthSpring Stability Tiers

**Version**: 1.0
**Date**: May 28, 2026 (V65a)
**Reference**: `primalSpring/config/capability_registry.toml` — stability annotations

---

## Principle

Method names consumed by downstream systems (lithoSpore, gardens, other
springs) are stability-annotated. Renaming a `stable` method requires
versioning and downstream notification per the primalSpring contract.

---

## IPC Capability Alignment

healthSpring's routing table (`composition/routing.rs`) maps capability
domains to primal providers. All capability strings used in IPC align
with the **canonical stable names** in the upstream registry:

| healthSpring Domain | Registry Domain | Stability | Canonical |
|--------------------|-----------------|-----------|-----------| 
| `dag` | `[dag]` | **stable** | Yes |
| `commit` / `ledger` / `spine` / `merkle` | `[spine]` / `[entry]` | **stable** | Yes |
| `braid` / `attribution` | `[braid]` / `[attribution]` | **stable** | Yes |
| `storage` / `content` | `[storage]` / `[content]` | **stable** | Yes |
| `security` / `crypto` / `fido2` | `[crypto]` / `[fido2]` | **stable** | Yes |
| `discovery` / `net.discovery` | `[discovery]` | **stable** | Yes |
| `stats` / `tensor` | `[stats]` / `[tensor]` | varies | Yes |
| `compute` | `[compute]` | varies | Yes |
| `shader` | `[shader]` | varies | Yes |
| `visualization` | `[visualization]` | **stable** | Yes |
| `orchestration` / `lifecycle` / `signal` | `[lifecycle]` / `[signals.*]` | **stable** | Yes |
| `inference` / `model` | `[inference]` | varies | Yes |
| `bonding` | `[bonding]` | **stable** | Yes |
| `audit` / `audit.log` / `defense` | `[defense]` | varies | Yes |

**Status**: All IPC capability strings are canonical. No local aliases
requiring the sweetGrass GAP-36 wire-name alias pattern. `bonding.*`
domain added in V64z (Wave 38 IonicContractRegistry absorption).

---

## Niche Science Method Tiers

healthSpring exposes 58 science methods via `ipc/dispatch`. These are
*spring-niche* methods — not in the upstream primal registry (they are
served by healthSpring itself, not by primals). They follow the same
stability scheme:

### stable — deployed consumers may depend on these

These methods are consumed by lithoSpore (Module 8 candidate) and
cross-spring validation. Do not rename without versioning.

| Method | Domain | Consumer |
|--------|--------|----------|
| `science.pkpd.hill_dose_response` | pkpd | lithoSpore Module 8, exp080+ |
| `science.pkpd.one_compartment_pk` | pkpd | lithoSpore Module 8, exp020+ |
| `science.pkpd.two_compartment_pk` | pkpd | exp035+ |
| `science.pkpd.auc_trapezoidal` | pkpd | exp048+ |
| `science.pkpd.nca_analysis` | pkpd | exp058+ |
| `science.microbiome.shannon_index` | microbiome | exp001+, cross-spring |
| `science.microbiome.simpson_index` | microbiome | exp002+ |
| `science.microbiome.pielou_evenness` | microbiome | exp003+ |
| `science.microbiome.chao1` | microbiome | exp004+ |
| `science.microbiome.anderson_gut` | microbiome | exp010+, groundSpring |
| `science.microbiome.colonization_resistance` | microbiome | exp028+ |
| `science.microbiome.bray_curtis` | microbiome | exp033+ |
| `science.biosignal.pan_tompkins` | biosignal | exp060+, neuralSpring |
| `science.biosignal.hrv_metrics` | biosignal | exp061+ |
| `science.biosignal.ppg_spo2` | biosignal | exp062+ |

### evolving — may change between major waves

| Method | Domain | Notes |
|--------|--------|-------|
| `science.pkpd.pbpk_simulate` | pkpd | Compartment model may expand |
| `science.pkpd.population_pk` | pkpd | NLME integration may refactor |
| `science.pkpd.nlme_foce` | pkpd | Algorithm refinement ongoing |
| `science.pkpd.nlme_saem` | pkpd | Algorithm refinement ongoing |
| `science.pkpd.cwres_diagnostics` | pkpd | Diagnostic output format evolving |
| `science.pkpd.vpc_simulate` | pkpd | Simulation engine evolving |
| `science.pkpd.gof_compute` | pkpd | Goodness-of-fit metrics evolving |
| `science.microbiome.fmt_blend` | microbiome | Blend algorithm evolving |
| `science.microbiome.antibiotic_perturbation` | microbiome | Perturbation model evolving |
| `science.microbiome.scfa_production` | microbiome | Production model evolving |
| `science.microbiome.gut_brain_serotonin` | microbiome | Pathway model evolving |
| `science.microbiome.qs_gene_profile` | microbiome | QS integration evolving |
| `science.microbiome.qs_effective_disorder` | microbiome | QS-Anderson evolving |
| `science.biosignal.eda_analysis` | biosignal | Feature extraction evolving |
| `science.biosignal.eda_stress_detection` | biosignal | Detection thresholds evolving |
| `science.biosignal.arrhythmia_classification` | biosignal | Template library evolving |
| `science.biosignal.fuse_channels` | biosignal | Fusion algorithm evolving |
| `science.biosignal.wfdb_decode` | biosignal | Format support expanding |
| `science.endocrine.*` (6 methods) | endocrine | TRT models under active research |
| `science.diagnostic.*` (3 methods) | diagnostic | Risk models under active research |
| `science.clinical.*` (3 methods) | clinical | Scenario models evolving |
| `science.comparative.*` (3 methods) | comparative | Cross-species models evolving |
| `science.discovery.*` (4 methods) | discovery | Drug discovery pipeline evolving |
| `science.toxicology.*` (3 methods) | toxicology | Dose-response models evolving |
| `science.simulation.*` (2 methods) | simulation | Ecosystem models evolving |

### internal — no stability guarantee

| Method | Domain | Notes |
|--------|--------|-------|
| `science.pkpd.allometric_scale` | pkpd | Internal scaling helper |
| `science.pkpd.michaelis_menten_nonlinear` | pkpd | Internal MM helper |

---

## lithoSpore Module 8 Coordination

B5 Leonard PK/PD is the next lithoSpore module candidate. The following
methods are in the critical path for Module 8 ingestion:

- `science.pkpd.hill_dose_response` (stable)
- `science.pkpd.one_compartment_pk` (stable)

When lithoSpore requests the `expected_values.json` format, these method
names MUST match the wire format. The current `control/ltee_symbiont_pkpd/expected_values.json`
already uses the correct names.

---

## Wire-Name Alias Policy

healthSpring does **not** use local aliases for upstream capability strings.
All IPC routing in `composition/routing.rs` uses canonical registry names.
No GAP-36 wire-name alias documentation is needed.
