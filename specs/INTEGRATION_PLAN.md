<!-- SPDX-License-Identifier: CC-BY-SA-4.0 (scyBorg: AGPL-3.0 code + ORC mechanics + CC-BY-SA-4.0 creative) -->
# healthSpring Integration Plan: NestGate + biomeOS + Atomic Stack

**Last Updated**: March 16, 2026

---

## 1. Current State

healthSpring is a **biomeOS niche** — discoverable at runtime via capability probes. 73 experiments (Exp001-106), 603 tests:
- Python + NumPy (Tier 0 — 42 baselines with provenance, 113/113 cross-validation checks across all 7 tracks)
- Rust + healthspring-barracuda crate (Tier 1 — 603 tests, 55+ JSON-RPC capabilities)
- GPU via wgpu/WGSL (Tier 2 — 6 shaders, GpuContext, fused pipeline)
- metalForge substrate routing (Tier 3 — 33 tests)
- petalTongue IPC integration (Exp064 — Unix socket JSON-RPC push)
- Capability-based primal discovery (zero hardcoded names)
- Songbird announcement on startup

## 2. Integration Targets

### 2.1 NestGate (Data Layer)

**Already implemented** in phase1/nestgate:
- `NCBILiveProvider`: ESearch, ESummary, EFetch for nucleotide, protein, pubmed, sra, taxonomy
- JSON-RPC: `data.ncbi_search`, `data.ncbi_fetch`
- Storage: `storage.store`, `storage.retrieve` with ZFS snapshots

**healthSpring needs**:

| Capability | NestGate Route | Use Case |
|------------|---------------|----------|
| PubMed search | `data.ncbi_search { database: "pubmed", query: "testosterone replacement therapy" }` | Literature scan for claim verification (Mok Track 4) |
| PubMed abstract fetch | `data.ncbi_fetch { database: "pubmed", accession: "PMID_..." }` | Extract methods, endpoints, sample sizes from cited papers |
| GEO expression data | `data.ncbi_fetch { database: "nucleotide", accession: "GSE..." }` | Androgen receptor expression for endocrine modeling |
| SRA 16S gut data | `data.ncbi_fetch { database: "sra", accession: "SRP..." }` | Gut microbiome profiling for Track 2 × Track 4 cross-study |
| Provenance snapshots | `storage.store { key: "healthspring:exp036:baseline", ... }` | Reproducible data versioning (ZFS) |

**Integration pattern**:
```
healthSpring experiment binary
    → capability_call("data.ncbi_search", { database: "pubmed", query: "Saad testosterone weight loss registry", max_results: 20 })
    → capability_call("storage.store", { key: "healthspring:mok:saad_2013_refs", data: results })
```

**Action items**:
1. Add `nestgate-client` dependency to healthspring-barracuda (optional feature)
2. Create `healthspring::data::ncbi` module with typed wrappers for PubMed/SRA queries
3. Create `control/data/fetch_mok_references.py` for initial PubMed metadata harvest

### 2.2 biomeOS NUCLEUS (Orchestration)

**Already implemented** in phase2/biomeOS:
- Atomic modes: Tower, Node, Nest, Full
- Deployment graphs: `nest_deploy.toml`, `node_atomic_compute.toml`, `wetspring_deploy.toml`
- Capability registry: 124 semantic translations
- LifecycleManager with health checks and graceful shutdown

**healthSpring integration path**:

#### Atomic Tower (Crypto + Network)
- healthSpring registers via BearDog + Songbird
- Capability registration: `science.pkpd.*`, `science.microbiome.*`, `science.biosignal.*`
- Required for LAN HPC multi-gate coordination

#### Atomic Node (Tower + Compute)
- Add ToadStool for GPU dispatch
- Population PK Monte Carlo: `compute.submit { workload: "pop_pk_100k", params: {...} }`
- Anderson eigensolve: `compute.submit { workload: "anderson_gut_L1000", params: {...} }`

#### Atomic Nest (Tower + Storage)
- NestGate for NCBI data and experiment results
- ZFS snapshots for reproducibility
- PubMed metadata cache

#### Full Stack
- All of the above: data fetch → compute → store → validate
- Distributed across LAN HPC gates

**Deployment graph** (proposed `healthspring_deploy.toml`):
```toml
[graph]
name = "healthspring"
description = "Human health science pipeline"

[[stages]]
name = "beardog"
primal = "beardog"
capabilities = ["crypto.identity", "crypto.verify"]

[[stages]]
name = "songbird"
primal = "songbird"
depends_on = ["beardog"]
capabilities = ["net.http", "net.discovery"]

[[stages]]
name = "nestgate"
primal = "nestgate"
depends_on = ["songbird"]
capabilities = ["data.ncbi_search", "data.ncbi_fetch", "storage.store", "storage.retrieve"]

[[stages]]
name = "toadstool"
primal = "toadstool"
depends_on = ["songbird"]
capabilities = ["compute.execute", "compute.submit"]

[[stages]]
name = "healthspring"
primal = "healthspring"
depends_on = ["nestgate", "toadstool"]
capabilities = [
    "science.pkpd.hill",
    "science.pkpd.population_pk",
    "science.microbiome.diversity",
    "science.microbiome.anderson_gut",
    "science.microbiome.colonization_resistance",
    "science.biosignal.pan_tompkins",
    "science.endocrine.testosterone_pk",
]
```

### 2.3 Atomic Tower/Node/Nest Local Deployment

**Before LAN HPC** — run locally on Eastgate:

```bash
# Tower only (development, capability registration)
biomeos nucleus start --mode tower

# Node (Tower + ToadStool GPU on local RTX 4070)
biomeos nucleus start --mode node

# Nest (Tower + NestGate for NCBI + local storage)
biomeos nucleus start --mode nest

# Full (all primals, single machine)
biomeos nucleus start --mode full
```

**After LAN HPC** — distributed:

| Gate | Atomic | Role |
|------|--------|------|
| Eastgate | Tower | Development coordination |
| Northgate | Node | GPU compute (RTX 5090, population Monte Carlo) |
| Strandgate | Nest | NCBI data + bulk processing (256GB, dual EPYC) |
| biomeGate | Node | f64 validation (Titan V native f64) |
| Westgate | Nest | Cold storage (76TB ZFS archive) |

Cross-gate routing via 10GbE:
```
Eastgate(Tower) → Strandgate(Nest): data.ncbi_fetch → SRA reads
Eastgate(Tower) → Northgate(Node): compute.submit → pop_pk_100K GPU
Northgate(Node) → Westgate(Nest): storage.store → experiment results
```

## 3. Write → Absorb → Lean Cycle

### Currently Writing (healthSpring-local)
- Testosterone PK models (IM/pellet/topical)
- Cross-track Monte Carlo (microbiome × endocrine)
- Clinical claim verification pipeline
- Biosignal FFT (pure Rust DFT, will absorb upstream FFT)

### Ready to Absorb (from barraCuda/toadStool)
- Population PK GPU dispatch (embarrassingly parallel)
- Anderson eigensolve (Lanczos GPU from hotSpring lineage)
- FFT for biosignal (from hotSpring)
- Mixed-effects MCMC (from neuralSpring optimization)

### Ready to Lean (consume upstream)
- Diversity indices (wetSpring `science.diversity`)
- Shannon/Simpson GPU (already in barraCuda)
- ODE solver (when absorbed from wetSpring)

### Potential Upstream Contributions
- Testosterone PK depot model (pellet zero-order release) → generalizes to any depot drug
- Clinical claim verification pipeline → novel methodology for all springs
- Cross-compartment allometric scaling → extends neuralSpring's mAb work
- Colonization resistance score → novel Anderson application for barraCuda

## 4. Phased Rollout

| Phase | Timeline | Integration | Experiments |
|-------|----------|-------------|-------------|
| **1: Local CPU** | Current | None (standalone) | Exp001-020, Exp030-035 |
| **2: NestGate data** | After 10GbE cables | Nest atomic on Strandgate | NCBI PubMed, GEO, SRA for Track 4 |
| **3: GPU compute** | After barraCuda absorbs pop PK | Node atomic on Northgate | Exp005+ (100K), Exp036 (10K TRT) |
| **4: Full pipeline** | After LAN HPC operational | Full atomic across gates | D4 cross-track, MIT-BIH ECG, large-lattice Anderson |
| **5: metalForge** | After toadStool matures | GPU → NPU → CPU dispatch | Streaming ECG on Akida NPU, GPU Monte Carlo, CPU validation |
