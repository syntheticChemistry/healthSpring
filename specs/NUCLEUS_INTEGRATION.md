<!-- SPDX-License-Identifier: CC-BY-SA-4.0 (scyBorg: AGPL-3.0 code + ORC mechanics + CC-BY-SA-4.0 creative) -->
# biomeOS NUCLEUS Local Integration Plan

**Last Updated**: April 19, 2026
**Status**: V56 — guideStone Level 4 (NUCLEUS validated, 49/49 live). Live IPC parity via barraCuda RTX 3070. BLAKE3 CHECKSUMS (17 files). primalSpring v0.9.16. 948+ tests, 94 experiments, 84+ JSON-RPC capabilities (62 science + 22 infra). Three-tier harness: Tier 1 local, Tier 2 IPC-wired (barraCuda live), Tier 3 primal proof. exp112–122 + guideStone. ecoBin 0.9.0.
**Depends on**: biomeOS (phase2/biomeOS/), NestGate (phase1/nestgate/), toadStool, metalForge

---

## Overview

healthSpring already has a complete deployment graph (`wateringHole/healthspring_deploy.toml`)
defining 10 stages across 5 primals (beardog, songbird, nestgate, toadstool, healthspring,
petaltongue). This document specifies the concrete steps to activate that graph, starting
locally on Eastgate and extending to LAN HPC as cables are connected.

---

## Step 1: Nest Atomic Locally (Eastgate)

**Goal**: NestGate running locally for NCBI data caching.

### What Already Exists

- `phase1/nestgate/` — NestGate crate with `NCBILiveProvider`
- `ncbi_live_provider.rs` — ESearch, ESummary, EFetch for nucleotide, protein, pubmed, sra, taxonomy
- `storage.rs` — content-addressed store with ZFS snapshot integration
- `discovery.rs` — service discovery via capability matching
- `rpc.rs` — JSON-RPC server

### Actions

1. **Start NestGate locally**:
   ```bash
   biomeos nucleus start --mode nest
   ```
   This starts beardog (identity), songbird (discovery), and nestgate (data + storage).

2. **Start healthSpring primal**:
   ```bash
   cargo run --bin healthspring_primal
   # or, with the niche deploy graph:
   biomeos deploy --graph graphs/healthspring_niche_deploy.toml
   ```
   healthSpring registers 50+ capabilities with the orchestrator via `capability.register`.

3. **Register healthSpring capabilities**:
   NestGate advertises `data.ncbi_search`, `data.ncbi_fetch`, `storage.store`, `storage.retrieve`.
   healthSpring discovers these via songbird capability matching (no hardcoded endpoints).

3. **Wire healthSpring `data` module** (see NestGate Data Provider design below):
   ```rust
   // Three-tier fetch: biomeOS → NestGate cache → direct NCBI HTTP
   let provider = NcbiProvider::discover().await?;
   let results = provider.search("pubmed", "testosterone replacement therapy").await?;
   ```

4. **Content-addressed caching**:
   All NCBI fetches cached under `ncbi:{db}:{accession}` keys.
   Subsequent requests hit local cache, not NCBI servers.

### Validation

- Fetch a PubMed abstract via NestGate → verify content matches direct NCBI fetch
- Store experiment result → retrieve and verify integrity
- Disconnect network → verify cached data still accessible

### Data Targets (Phase 1)

| Target | NestGate Route | Size | Use |
|--------|---------------|------|-----|
| PubMed TRT literature | `data.ncbi_search { database: "pubmed" }` | ~50MB | Mok claim verification |
| NCBI Gene QS families | `data.ncbi_search { database: "gene" }` | ~200MB | QS gene matrix construction |
| UniProt QS proteins | `data.ncbi_fetch { database: "protein" }` | ~500MB | QS gene validation |
| KEGG pathways (SCFA) | External REST API | ~100MB | Metabolic pathway mapping |

---

## Step 2: Node Atomic Locally (Eastgate)

**Goal**: toadStool GPU dispatch through NUCLEUS, using local RTX 4070.

### What Already Exists

- `toadstool/` — streaming pipeline with `StageOp` → `GpuOp` conversion
- `ecoPrimal/src/gpu/` — `GpuContext`, 6 WGSL shaders, `execute_fused()`
- `metalForge/forge/` — `Workload` routing with 9 variants, `select_substrate()`
- `wateringHole/healthspring_deploy.toml` — toadstool stage with `compute.execute`, `compute.submit`, `compute.gpu.dispatch`

### Actions

1. **Start Node Atomic locally**:
   ```bash
   biomeos nucleus start --mode node
   ```
   This adds toadStool to the Nest stack.

2. **GPU workload dispatch via NUCLEUS**:
   Instead of direct `GpuContext::execute()`, route through:
   ```rust
   capability_call("compute.gpu.dispatch", GpuOp::MichaelisMentenBatch { ... })
   ```
   toadStool handles device acquisition, shader compilation, and result readback.

3. **metalForge substrate selection via NUCLEUS**:
   ```rust
   let substrate = capability_call("compute.select_substrate", workload);
   match substrate {
       Substrate::Gpu => capability_call("compute.gpu.dispatch", op),
       Substrate::Cpu => execute_cpu(op),
       Substrate::Npu => capability_call("compute.npu.dispatch", op),
   }
   ```

### Validation

- Run Exp083 GPU parity checks through NUCLEUS dispatch → same 25/25 results
- metalForge routing matches direct `select_substrate()` calls
- Latency overhead < 1ms for local Unix socket RPC

---

## Step 3: LAN Mesh (10GbE cables pending)

**Goal**: Multi-gate NUCLEUS with specialized roles per machine.

### Hardware Layout

| Gate | CPU | GPU | RAM | Role | Atomic Mode |
|------|-----|-----|-----|------|-------------|
| **Eastgate** | i9-12900 | RTX 4070 (12GB) | 32GB | Development, Tower coordination | Tower |
| **Northgate** | i9-14900K | RTX 5090 (32GB) | 192GB | Heavy GPU compute | Node |
| **Strandgate** | Dual EPYC | — | 256GB | NCBI data, batch CPU | Nest |
| **biomeGate** | TR 3970X | Titan V (12GB) | 256GB | f64-native GPU validation | Node |
| **Westgate** | — | — | — | ZFS archive (76TB) | Nest |

### Network

- 10GbE switches and NICs already installed
- Cat6a/DAC cables are the only pending hardware
- Expected: ~1ms RPC latency, ~1GB/s sustained throughput

### Actions

1. **Start services on each gate**:
   ```bash
   # Eastgate (Tower — coordinator)
   biomeos nucleus start --mode tower

   # Northgate (Node — GPU)
   biomeos nucleus start --mode node

   # Strandgate (Nest — data)
   biomeos nucleus start --mode nest

   # Westgate (Nest — archive)
   biomeos nucleus start --mode nest --storage-only
   ```

2. **Cross-gate capability discovery**:
   Songbird multicast on 10GbE → all gates discover each other.
   healthSpring on Eastgate sees:
   - `compute.gpu.dispatch` on Northgate (RTX 5090)
   - `data.ncbi_fetch` on Strandgate (256GB RAM)
   - `storage.store` on Westgate (76TB ZFS)

3. **Workload routing**:
   ```
   Eastgate: healthspring experiment binary
       → metalForge: Workload::MichaelisMentenBatch { n_patients: 100_000 }
       → select_substrate → GPU (element_count > threshold)
       → capability_call("compute.gpu.dispatch") → routes to Northgate
       → Northgate: GpuContext::execute(MichaelisMentenBatch { ... })
       → result streams back over 10GbE (~100MB in ~0.1s)
   ```

### Validation

- Population PK 100K patients dispatched to Northgate GPU — verify same results as Eastgate RTX 4070
- NCBI fetch routed to Strandgate — verify content-addressed caching on Westgate ZFS
- Cross-gate latency < 2ms per RPC call
- Throughput: 100K patient MM batch in < 1s end-to-end

### Data Flow Example: Real 16S Pipeline

```
Eastgate (Tower) ─── "process HMP 16S data for Anderson analysis"
    │
    ├──→ Strandgate (Nest): data.ncbi_fetch { database: "sra", accession: "SRP..." }
    │       └── Downloads ~50GB from NCBI SRA
    │       └── Caches to Westgate ZFS: ncbi:sra:SRP...
    │
    ├──→ Strandgate (Nest): DADA2 pipeline (64 EPYC cores, 256GB RAM)
    │       └── OTU tables → species abundances
    │
    ├──→ Eastgate (local): QS gene matrix lookup → W_effective
    │
    ├──→ Northgate (Node): compute.gpu.dispatch Anderson eigensolve (L=1000)
    │       └── RTX 5090: Lanczos on 1M×1M Hamiltonian
    │
    └──→ Eastgate: Assemble results → petalTongue visualization
```

---

## Step 4: Full NUCLEUS

**Goal**: biomeOS orchestrates the complete healthSpring DAG defined in
`healthspring_deploy.toml`.

### Full Pipeline Execution

biomeOS reads `healthspring_deploy.toml` and resolves the 10-stage DAG:

```
beardog → songbird → nestgate ─┬──→ microbiome ─┐
                      toadstool ┤               ├──→ diagnostic → clinical → petaltongue → results
                                ├──→ pkpd ───────┤
                                ├──→ biosignal ──┤
                                └──→ endocrine ──┘
```

Each stage is placed on the optimal gate via metalForge:
- **nestgate** → Strandgate (data) + Westgate (archive)
- **toadstool** → Northgate (GPU) + biomeGate (f64 validation)
- **pkpd/microbiome/biosignal/endocrine/diagnostic** → Eastgate (coordination) with GPU dispatch to Northgate
- **petaltongue** → Eastgate (display)
- **results** → Westgate (ZFS archive)

### New Capabilities for V17+ Stages

The deployment graph should be extended with V16/V17 capabilities:

```toml
# In pkpd stage:
"science.pkpd.michaelis_menten_pk",
"science.pkpd.michaelis_menten_batch_gpu",

# In microbiome stage:
"science.microbiome.antibiotic_perturbation",
"science.microbiome.scfa_production",
"science.microbiome.scfa_batch_gpu",
"science.microbiome.gut_brain_serotonin",
"science.microbiome.qs_gene_density",
"science.microbiome.effective_disorder",

# In biosignal stage:
"science.biosignal.eda_stress",
"science.biosignal.arrhythmia_classify",
"science.biosignal.beat_classify_batch_gpu",
```

### Plasmodium Collective

Full NUCLEUS uses `.family.seed` trust for cross-gate authentication:
- All gates share the same family seed (sovereign, no external CA)
- beardog handles key generation and verification
- Songbird discovery limited to trusted family members

### Monitoring and Health

biomeOS NUCLEUS provides:
- Per-gate health checks (CPU, GPU, RAM, storage)
- Per-stage latency tracking
- Workload queue depth per Node
- NestGate cache hit/miss ratio
- ZFS snapshot provenance audit

---

## Dependencies

| Component | Location | Status |
|-----------|----------|--------|
| biomeOS NUCLEUS | phase2/biomeOS/ | Existing (v0.x) |
| NestGate | phase1/nestgate/ | Existing (live NCBI provider) |
| toadStool | phase1/toadstool/ | Existing (S142) |
| metalForge | healthSpring/metalForge/ | Existing (33 tests, 9 Workloads) |
| healthspring_deploy.toml | wateringHole/ | Existing (10 stages, 5 primals) |
| 10GbE cables | Physical | **Pending** — switches + NICs installed |
| NestGate data provider | ecoPrimal/src/data/ | **Planned** (see NestGate design) |

---

## Timeline

| Step | When | Effort | Blocker |
|------|------|--------|---------|
| 1: Nest Atomic locally | Now | 2 days | None |
| 2: Node Atomic locally | Now | 1 day | None |
| 3: LAN mesh | After cables | 3 days | 10GbE cables |
| 4: Full NUCLEUS | After Step 3 | 1 week | Step 3 complete |
