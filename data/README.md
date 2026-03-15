<!-- SPDX-License-Identifier: CC-BY-SA-4.0 (scyBorg: AGPL-3.0 code + ORC mechanics + CC-BY-SA-4.0 creative) -->
# healthSpring Data

Open-source, reproducible data fetching for healthSpring experiments. Every
dataset is downloaded from public repositories using pinned scripts with
documented provenance.

**No data is committed to this repository.** Only the scripts, manifest, and
documentation are tracked in git. Downloaded data lives on local storage
(NVMe hot cache or Westgate ZFS cold archive).

---

## Quick Start

```bash
# Set NCBI API key (optional but recommended — 10 req/sec vs 3 req/sec)
export NCBI_API_KEY="your-key-here"

# Optional: override data root (default: ./data/)
export HEALTHSPRING_DATA_ROOT="/path/to/local/storage"

# Optional: cold archive for LAN HPC (Westgate ZFS)
export HEALTHSPRING_COLD_STORAGE="/mnt/westgate/healthspring/data"

# Install Python dependencies
pip install -r requirements.txt

# Fetch QS gene matrix (NCBI Gene/Protein, ~5 min)
python fetch_qs_genes.py

# Fetch MIT-BIH records (PhysioNet, ~2 min) — planned
# bash fetch_mitbih.sh

# Fetch ChEMBL Hill panel (ChEMBL REST API, ~10 min) — planned
# python fetch_chembl.py
```

---

## Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `NCBI_API_KEY` | Auto-discovered | NCBI E-utilities API key (3 req/sec without, 10 with). Auto-discovers from `ecoPrimals/testing-secrets/api-keys.toml` or `~/.ncbi/api_key` |
| `NCBI_EMAIL` | None | Required by NCBI for API access |
| `HEALTHSPRING_DATA_ROOT` | `./` (this directory) | Where downloaded data is stored |
| `HEALTHSPRING_COLD_STORAGE` | None | LAN cold archive path; scripts check here before downloading |

---

## Reproducibility Guarantees

1. **Pinned dependencies**: `requirements.txt` pins exact versions
2. **Pinned accessions**: `manifest.toml` lists every dataset with accession IDs
3. **Checksums**: After first download, SHA-256 checksums are recorded in manifest
4. **Idempotent**: Scripts skip downloads if output exists with matching checksum
5. **Provenance**: Every output JSON includes a `_provenance` block (date, script, commit, command)
6. **Seeds**: Any random operations use fixed seeds
7. **Cold-storage aware**: If `HEALTHSPRING_COLD_STORAGE` is set, scripts check there before fetching from network

---

## Data Sources

All data comes from open, public repositories. No proprietary data dependencies.

| Dataset | Source | License | Script | Tracks |
|---------|--------|---------|--------|--------|
| QS gene matrix (6 families × ~200 species) | NCBI Gene/Protein | Public domain | `fetch_qs_genes.py` | 2 (Microbiome) |
| MIT-BIH Arrhythmia (48 records) | PhysioNet | ODC-By | `fetch_mitbih.sh` (planned) | 3 (Biosignal) |
| ChEMBL Hill panel (50+ compounds) | ChEMBL REST | CC-BY-SA 3.0 | `fetch_chembl.py` (planned) | 1 (PK/PD), 7 (Drug Discovery) |
| HMP 16S profiles | NCBI SRA | Public domain | `fetch_hmp_16s.py` (planned) | 2 (Microbiome) |
| GEO androgen receptor expression | NCBI GEO | Public domain | `fetch_geo_ar.py` (planned) | 4 (Endocrinology) |

See `manifest.toml` for the complete pinned dataset registry.

---

## LAN HPC Data Flow

```
Network (NCBI/PhysioNet/ChEMBL)
    │
    ▼
Hot Cache ($HEALTHSPRING_DATA_ROOT)
    │  NVMe on Eastgate/Strandgate
    │
    ▼
Cold Archive ($HEALTHSPRING_COLD_STORAGE)
    │  Westgate 76TB ZFS
    │
    ▼
Compute Nodes (Northgate GPU / Strandgate CPU / Eastgate utility)
    pull from cold archive on demand via LAN 10GbE
```

Scripts always check cold storage before network. Once data is on ZFS, no
further internet access is needed for reproducibility.
