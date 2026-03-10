<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# healthSpring NestGate Data Provider Design

**Last Updated**: March 10, 2026
**Status**: Design — follows wetSpring pattern (`wetSpring/barracuda/src/ncbi/nestgate/`)
**Depends on**: phase1/nestgate/, phase2/biomeOS/, NCBI E-utilities API

---

## Overview

healthSpring needs transparent access to NCBI databases (PubMed, Gene, SRA, GEO) and other
external data sources (ChEMBL, PhysioNet, KEGG) with local caching and provenance tracking.
This module provides a three-tier fetch pattern identical to wetSpring's NestGate integration:

```
Tier 1: biomeOS capability.call → Tier 2: NestGate cache → Tier 3: Direct HTTP
```

---

## Module Structure

```
barracuda/src/data/
├── mod.rs          — Public API re-exports, NcbiProvider type
├── discovery.rs    — Socket discovery (biomeOS, NestGate)
├── fetch.rs        — Three-tier fetch logic
├── rpc.rs          — JSON-RPC 2.0 over Unix socket
└── storage.rs      — Content-addressed store/retrieve/exists
```

---

## Public API

### Discovery

```rust
/// Discover the biomeOS orchestrator socket.
/// Checks: $BIOMEOS_SOCKET → $XDG_RUNTIME_DIR/biomeos/biomeos-default.sock → /tmp/
pub fn discover_biomeos_socket() -> Option<PathBuf>

/// Discover the NestGate data provider socket.
/// Checks: $NESTGATE_SOCKET → $XDG_RUNTIME_DIR/biomeos/nestgate-default.sock → /tmp/
pub fn discover_socket() -> Option<PathBuf>

/// Returns true if NestGate is enabled via HEALTHSPRING_DATA_PROVIDER=nestgate
pub fn is_enabled() -> bool
```

### Fetch

```rust
/// Three-tier fetch: biomeOS → NestGate cache → direct NCBI HTTP.
///
/// Returns the fetched content as a String (XML for PubMed, FASTA for sequences, etc.)
pub fn fetch_tiered(db: &str, id: &str, api_key: &str) -> Result<String, DataError>

/// Fetch via biomeOS capability.call("science.ncbi_fetch", { db, id, api_key }).
pub fn fetch_via_biomeos(
    biomeos_socket: &Path,
    db: &str,
    id: &str,
    api_key: &str,
) -> Result<String, DataError>

/// Fetch via NestGate: check cache → if miss, fetch from NCBI → store in cache.
pub fn fetch_or_fallback(
    socket: &Path,
    db: &str,
    id: &str,
    api_key: &str,
) -> Result<String, DataError>

/// Search NCBI database. Returns list of IDs matching the query.
pub fn search_tiered(
    db: &str,
    query: &str,
    max_results: u32,
    api_key: &str,
) -> Result<Vec<String>, DataError>
```

### Storage

```rust
/// Check if a key exists in NestGate content-addressed store.
pub fn exists(socket: &Path, key: &str) -> Result<bool, DataError>

/// Retrieve a value from NestGate by key.
pub fn retrieve(socket: &Path, key: &str) -> Result<String, DataError>

/// Store a value in NestGate under the given key.
pub fn store(socket: &Path, key: &str, value: &str) -> Result<(), DataError>
```

### High-Level Typed API

```rust
/// healthSpring-specific NCBI provider with typed accessors.
pub struct NcbiProvider {
    api_key: String,
}

impl NcbiProvider {
    /// Create a new provider. API key from $NCBI_API_KEY or empty (rate-limited).
    pub fn new() -> Self;

    /// Search PubMed for abstracts matching a query.
    pub fn search_pubmed(&self, query: &str, max_results: u32) -> Result<Vec<String>, DataError>;

    /// Fetch a PubMed abstract by PMID.
    pub fn fetch_pubmed(&self, pmid: &str) -> Result<PubMedAbstract, DataError>;

    /// Search NCBI Gene for gene records.
    pub fn search_gene(&self, query: &str, max_results: u32) -> Result<Vec<String>, DataError>;

    /// Fetch a gene record by GeneID.
    pub fn fetch_gene(&self, gene_id: &str) -> Result<GeneRecord, DataError>;

    /// Search NCBI SRA for sequencing runs.
    pub fn search_sra(&self, query: &str, max_results: u32) -> Result<Vec<String>, DataError>;

    /// Fetch SRA run metadata by accession.
    pub fn fetch_sra_metadata(&self, accession: &str) -> Result<SraMetadata, DataError>;

    /// Search NCBI GEO for expression datasets.
    pub fn search_geo(&self, query: &str, max_results: u32) -> Result<Vec<String>, DataError>;
}
```

### Typed Result Structures

```rust
pub struct PubMedAbstract {
    pub pmid: String,
    pub title: String,
    pub authors: Vec<String>,
    pub journal: String,
    pub year: u16,
    pub abstract_text: String,
}

pub struct GeneRecord {
    pub gene_id: String,
    pub symbol: String,
    pub organism: String,
    pub description: String,
    pub aliases: Vec<String>,
}

pub struct SraMetadata {
    pub accession: String,
    pub study_title: String,
    pub organism: String,
    pub instrument: String,
    pub library_strategy: String,
    pub total_bases: u64,
}
```

---

## Storage Key Format

Follows wetSpring convention — semantic keys, not content hashes:

```
ncbi:{database}:{identifier}
```

Examples:
- `ncbi:pubmed:33456789` — PubMed abstract
- `ncbi:gene:945803` — NCBI Gene record for luxS in E. coli
- `ncbi:sra:SRP048479` — SRA study metadata
- `ncbi:geo:GSE12345` — GEO expression dataset metadata

For non-NCBI sources:
- `chembl:activity:{chembl_id}` — ChEMBL bioactivity data
- `physionet:record:{db}:{record_id}` — PhysioNet waveform metadata
- `kegg:pathway:{pathway_id}` — KEGG metabolic pathway

---

## Three-Tier Fetch Flow

```
fetch_tiered("pubmed", "33456789", api_key)
    │
    ├── Tier 1: discover_biomeos_socket() → Some(path)
    │   └── capability.call("science.ncbi_fetch", { db: "pubmed", id: "33456789" })
    │       └── Success → return
    │       └── Fail → fall through
    │
    ├── Tier 2: is_enabled() && discover_socket() → Some(path)
    │   ├── exists(socket, "ncbi:pubmed:33456789") → true
    │   │   └── retrieve(socket, "ncbi:pubmed:33456789") → return
    │   └── exists → false
    │       └── efetch("pubmed", "33456789") → content
    │       └── store(socket, "ncbi:pubmed:33456789", content)
    │       └── return content
    │
    └── Tier 3: efetch("pubmed", "33456789") → direct NCBI HTTP
        └── return content (no caching)
```

---

## RPC Transport

JSON-RPC 2.0 over Unix domain socket, newline-delimited (matching wetSpring exactly):

```json
{"jsonrpc": "2.0", "method": "storage.exists", "params": {"key": "ncbi:pubmed:33456789", "family_id": "default"}, "id": 1}\n
```

Response:
```json
{"jsonrpc": "2.0", "result": true, "id": 1}\n
```

---

## Error Type

```rust
#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("NestGate socket not found")]
    SocketNotFound,

    #[error("NestGate RPC error: {0}")]
    Rpc(String),

    #[error("NCBI HTTP error: {status} for {url}")]
    NcbiHttp { status: u16, url: String },

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

---

## Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `HEALTHSPRING_DATA_PROVIDER` | `""` | Set to `nestgate` to enable Tier 2 caching |
| `NCBI_API_KEY` | `""` | NCBI E-utilities API key (10 req/s with key, 3/s without) |
| `NESTGATE_SOCKET` | Auto-discover | Override NestGate socket path |
| `BIOMEOS_SOCKET` | Auto-discover | Override biomeOS socket path |

---

## Feature Gate

The data module is behind an optional Cargo feature to keep the core barracuda crate lean:

```toml
[features]
default = []
nestgate = ["dep:thiserror"]
```

When `nestgate` is not enabled, `NcbiProvider` methods that require network access return
`DataError::SocketNotFound`. The QS gene matrix and other cached data can still be loaded
from local JSON files.

---

## Target Data Sources by Priority

| Priority | Source | Module | Key Pattern | Use |
|:--------:|--------|--------|-------------|-----|
| 1 | NCBI Gene | `fetch_gene` | `ncbi:gene:{id}` | QS gene matrix (luxI, luxS, agrB, etc.) |
| 2 | PubMed | `fetch_pubmed` | `ncbi:pubmed:{pmid}` | Mok claim verification, paper references |
| 3 | NCBI SRA | `fetch_sra_metadata` | `ncbi:sra:{accession}` | 16S gut microbiome raw reads (HMP, Dethlefsen) |
| 4 | NCBI GEO | `search_geo` | `ncbi:geo:{accession}` | Androgen receptor expression |
| 5 | ChEMBL | Future: `fetch_chembl` | `chembl:activity:{id}` | Drug panel IC50/EC50 expansion |
| 6 | PhysioNet | Future: `fetch_physionet` | `physionet:record:{db}:{id}` | MIT-BIH, MIMIC-III metadata |
| 7 | KEGG | Future: `fetch_kegg` | `kegg:pathway:{id}` | SCFA metabolic pathways |

---

## Testing Strategy

1. **Unit tests** (no network): Mock RPC responses, test parsing, key generation, error handling
2. **Integration tests** (NestGate running): Start NestGate locally, test full three-tier flow
3. **Experiment validation**: Exp084 (QS gene matrix) uses `NcbiProvider` to fetch gene records,
   validates against known gene families

---

## Dependencies on Other Modules

| Module | Dependency | Direction |
|--------|-----------|-----------|
| `microbiome.rs` | `data::NcbiProvider` | Consumes gene records for QS matrix |
| `toadstool/src/stage.rs` | None | Data fetch is pre-pipeline (input preparation) |
| `metalForge` | None | Data fetch is not a compute workload |
| `specs/QS_GENE_PROFILING.md` | `data::fetch_gene` | Uses to build presence/absence matrix |
