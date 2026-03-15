#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
"""
Fetch QS gene presence/absence matrix from NCBI Gene + UniProt + KEGG.

Three-source strategy for maximum coverage:
  1. NCBI Gene — gene name search per genus (fast, misses non-standard names)
  2. UniProt — keyword "quorum sensing" reviewed entries (catches curated QS proteins)
  3. KEGG Orthology — KO entries for QS gene families (catches pathway-annotated genes)

Sources are merged with OR logic: if ANY source finds a QS family gene in a
genus, it's marked as present.

Output: qs_gene_matrix.json — directly consumable by ecoPrimal/src/qs.rs.

Reproducibility:
  - All queries are deterministic (same database state → same results)
  - Output includes _provenance block with date, script, commit, command
  - Idempotent: skips if output exists and --force is not passed
  - NCBI API key optional but recommended (10 req/sec vs 3 req/sec)

Environment:
  NCBI_API_KEY    — NCBI E-utilities API key (auto-discovered from testing-secrets)
  NCBI_EMAIL      — Email for NCBI API access (required by NCBI policy)
  HEALTHSPRING_DATA_ROOT — Output directory (default: script directory)

Usage:
  python fetch_qs_genes.py [--force] [--ncbi-only]
"""
from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

try:
    from Bio import Entrez
except ImportError:
    sys.exit(
        "biopython is required. Install with: pip install biopython==1.84\n"
        "Or: pip install -r requirements.txt"
    )

try:
    import requests
except ImportError:
    requests = None  # type: ignore[assignment]

# ── QS gene families ─────────────────────────────────────────────────────────
# Each family has representative gene names queried against NCBI Gene.
# Reference: specs/QS_GENE_PROFILING.md

QS_FAMILIES: dict[str, list[str]] = {
    "LuxIR": ["luxI", "luxR"],
    "LuxS": ["luxS"],
    "Agr": ["agrA", "agrB", "agrC", "agrD"],
    "Com": ["comA", "comB", "comC", "comD", "comE"],
    "LasRhl": ["lasI", "lasR", "rhlI", "rhlR"],
    "Fsr": ["fsrA", "fsrB", "fsrC"],
}

# KEGG Orthology entries for each QS family.
# These capture pathway-annotated genes that may have non-standard names.
KEGG_KO_FAMILIES: dict[str, list[str]] = {
    "LuxIR": ["K13060", "K13061"],  # luxI (AHL synthase), luxR (transcriptional activator)
    "LuxS": ["K07173"],              # luxS (S-ribosylhomocysteine lyase / AI-2 synthase)
    "Agr": ["K12251", "K12252", "K12253", "K12254"],  # agrA, agrB, agrC, agrD
    "Com": ["K12289", "K12290"],     # comA (response regulator), comD (histidine kinase)
    "LasRhl": ["K13060", "K13061"],  # same KO family as LuxIR (AHL-based)
    "Fsr": ["K12251"],               # fsrA shares KO with agrA (response regulator family)
}

# UniProt keyword patterns for each QS family
UNIPROT_FAMILY_KEYWORDS: dict[str, list[str]] = {
    "LuxIR": ["luxI", "luxR", "acyl-homoserine lactone synthase"],
    "LuxS": ["luxS", "autoinducer-2", "S-ribosylhomocysteine lyase"],
    "Agr": ["agrA", "agrB", "agrC", "agrD", "autoinducing peptide"],
    "Com": ["comA", "comD", "competence stimulating peptide"],
    "LasRhl": ["lasI", "lasR", "rhlI", "rhlR"],
    "Fsr": ["fsrA", "fsrB", "fsrC", "gelatinase biosynthesis"],
}

# ── Common human gut microbe genera ──────────────────────────────────────────

GUT_GENERA: list[str] = [
    # Firmicutes
    "Clostridium", "Clostridioides", "Faecalibacterium", "Roseburia",
    "Eubacterium", "Ruminococcus", "Coprococcus", "Dorea", "Blautia",
    "Lachnospira", "Anaerostipes", "Butyrivibrio", "Lactobacillus",
    "Limosilactobacillus", "Lacticaseibacillus", "Lactiplantibacillus",
    "Enterococcus", "Streptococcus", "Staphylococcus", "Peptoclostridium",
    "Peptostreptococcus", "Veillonella", "Megasphaera", "Dialister",
    "Acidaminococcus", "Christensenella",
    # Bacteroidetes
    "Bacteroides", "Prevotella", "Parabacteroides", "Alistipes",
    "Porphyromonas", "Barnesiella", "Odoribacter", "Tannerella",
    # Actinobacteria
    "Bifidobacterium", "Collinsella", "Eggerthella", "Actinomyces",
    "Atopobium", "Slackia",
    # Proteobacteria
    "Escherichia", "Klebsiella", "Enterobacter", "Citrobacter",
    "Proteus", "Salmonella", "Shigella", "Campylobacter",
    "Helicobacter", "Desulfovibrio", "Bilophila", "Sutterella",
    "Parasutterella", "Pseudomonas", "Acinetobacter",
    # Verrucomicrobia
    "Akkermansia",
    # Fusobacteria
    "Fusobacterium",
    # Synergistetes
    "Synergistes",
    # Euryarchaeota (methanogenic archaea in gut)
    "Methanobrevibacter",
]

_REQUEST_INTERVAL = 0.35


def _get_git_commit() -> str:
    """Get current git commit hash, or 'unknown' if not in a git repo."""
    try:
        result = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            capture_output=True, text=True, check=True,
            cwd=Path(__file__).parent.parent,
        )
        return result.stdout.strip()[:12]
    except (subprocess.CalledProcessError, FileNotFoundError):
        return "unknown"


def _discover_ncbi_api_key() -> str:
    """Discover NCBI API key from env or ecoPrimals testing-secrets."""
    key = os.environ.get("NCBI_API_KEY", "")
    if key:
        return key

    secrets_paths = [
        Path(__file__).resolve().parent.parent.parent / "testing-secrets" / "api-keys.toml",
        Path.home() / ".ncbi" / "api_key",
    ]
    for path in secrets_paths:
        if path.exists():
            text = path.read_text()
            if path.name == "api-keys.toml":
                for line in text.splitlines():
                    if line.strip().startswith("ncbi_api_key"):
                        parts = line.split("=", 1)
                        if len(parts) == 2:
                            return parts[1].strip().strip('"').strip("'")
            else:
                return text.strip()
    return ""


def _setup_entrez() -> None:
    """Configure Biopython Entrez with API key and email."""
    Entrez.email = os.environ.get("NCBI_EMAIL", "healthspring@ecoprimal.dev")
    api_key = _discover_ncbi_api_key()
    if api_key:
        Entrez.api_key = api_key
        print("  NCBI API key: set (10 req/sec)")
    else:
        print("  NCBI API key: not set (3 req/sec — set NCBI_API_KEY for faster fetching)")


# ── Source 1: NCBI Gene ──────────────────────────────────────────────────────

def _esearch_gene(gene_name: str, genus: str) -> list[str]:
    """Search NCBI Gene for a gene name within a genus. Returns list of GeneIDs."""
    query = f'{gene_name}[Gene Name] AND "{genus}"[Organism]'
    try:
        handle = Entrez.esearch(db="gene", term=query, retmax=500)
        record = Entrez.read(handle)
        handle.close()
        time.sleep(_REQUEST_INTERVAL)
        return record.get("IdList", [])
    except Exception as exc:
        print(f"    Warning: esearch failed for {gene_name}/{genus}: {exc}")
        time.sleep(_REQUEST_INTERVAL * 2)
        return []


def _ncbi_gene_scan(verbose: bool = True) -> dict[str, dict[str, bool]]:
    """Source 1: NCBI Gene name search per genus × family."""
    results: dict[str, dict[str, bool]] = {}
    total = len(GUT_GENERA) * len(QS_FAMILIES)
    done = 0

    print("\n[Source 1/3] NCBI Gene search")
    for genus in GUT_GENERA:
        profile: dict[str, bool] = {}
        for family_name, gene_list in QS_FAMILIES.items():
            found = False
            for gene in gene_list:
                if _esearch_gene(gene, genus):
                    found = True
                    break
            profile[family_name] = found
            done += 1
            if verbose and done % 20 == 0:
                print(f"  NCBI: {done}/{total} ({100*done//total}%)")
        results[genus] = profile
        if verbose:
            hits = sum(1 for v in profile.values() if v)
            if hits > 0:
                print(f"  {genus}: {hits}/6 (NCBI Gene)")

    return results


# ── Source 2: UniProt reviewed entries ────────────────────────────────────────

def _uniprot_qs_scan(verbose: bool = True) -> dict[str, dict[str, bool]]:
    """Source 2: UniProt keyword search for reviewed QS proteins per genus."""
    results: dict[str, dict[str, bool]] = {g: {f: False for f in QS_FAMILIES} for g in GUT_GENERA}

    if requests is None:
        print("\n[Source 2/3] UniProt — SKIPPED (requests not installed)")
        return results

    print("\n[Source 2/3] UniProt reviewed QS proteins")
    base = "https://rest.uniprot.org/uniprotkb/search"

    for family_name, keywords in UNIPROT_FAMILY_KEYWORDS.items():
        for genus in GUT_GENERA:
            keyword_clause = " OR ".join(f'"{kw}"' for kw in keywords)
            query = f"(organism_name:{genus}) AND ({keyword_clause}) AND (reviewed:true)"
            try:
                resp = requests.get(
                    base,
                    params={"query": query, "format": "json", "size": "1"},
                    timeout=15,
                )
                if resp.ok:
                    data = resp.json()
                    if data.get("results"):
                        results[genus][family_name] = True
                time.sleep(0.2)
            except Exception as exc:
                if verbose:
                    print(f"    Warning: UniProt query failed for {family_name}/{genus}: {exc}")
                time.sleep(0.5)

    if verbose:
        found_genera = [g for g in GUT_GENERA if any(results[g].values())]
        print(f"  UniProt found QS genes in {len(found_genera)} genera")
        for g in found_genera:
            hits = sum(1 for v in results[g].values() if v)
            families_hit = [f for f in QS_FAMILIES if results[g][f]]
            print(f"    {g}: {hits}/6 ({', '.join(families_hit)})")

    return results


# ── Source 3: KEGG Orthology ─────────────────────────────────────────────────

def _kegg_qs_scan(verbose: bool = True) -> dict[str, dict[str, bool]]:
    """Source 3: KEGG Orthology lookup for QS KO entries per genus."""
    results: dict[str, dict[str, bool]] = {g: {f: False for f in QS_FAMILIES} for g in GUT_GENERA}

    if requests is None:
        print("\n[Source 3/3] KEGG Orthology — SKIPPED (requests not installed)")
        return results

    print("\n[Source 3/3] KEGG Orthology QS gene families")
    base = "https://rest.kegg.jp"

    genus_lower = {g.lower(): g for g in GUT_GENERA}
    seen_kos: set[str] = set()

    for family_name, ko_list in KEGG_KO_FAMILIES.items():
        for ko_id in ko_list:
            if ko_id in seen_kos:
                continue
            seen_kos.add(ko_id)

            try:
                resp = requests.get(f"{base}/link/genes/{ko_id.lower()}", timeout=15)
                if not resp.ok:
                    time.sleep(0.5)
                    continue

                for line in resp.text.strip().splitlines():
                    parts = line.split("\t")
                    if len(parts) < 2:
                        continue
                    gene_ref = parts[1]
                    # gene_ref format: "org:gene_id" e.g. "eco:b2687"
                    # We need to check if the organism's genus matches
                    org_code = gene_ref.split(":")[0] if ":" in gene_ref else ""
                    if not org_code:
                        continue

                time.sleep(0.3)
            except Exception as exc:
                if verbose:
                    print(f"    Warning: KEGG query failed for {ko_id}: {exc}")
                time.sleep(0.5)

    # KEGG link/genes returns organism codes, not genus names.
    # Map KEGG organism codes to genera via KEGG /list/organism
    # This is expensive, so we use a targeted approach: query KEGG
    # for each genus directly via /find/genes
    print("  Querying KEGG /find/genes per genus...")
    for family_name, gene_list in QS_FAMILIES.items():
        for genus in GUT_GENERA:
            for gene in gene_list:
                try:
                    resp = requests.get(
                        f"{base}/find/genes/{gene}+{genus}",
                        timeout=15,
                    )
                    if resp.ok and resp.text.strip():
                        lines = resp.text.strip().splitlines()
                        if lines:
                            results[genus][family_name] = True
                            break
                    time.sleep(0.15)
                except Exception:
                    time.sleep(0.3)
            # Short-circuit: if already found from any gene, skip rest
            if results[genus][family_name]:
                continue

    if verbose:
        found_genera = [g for g in GUT_GENERA if any(results[g].values())]
        print(f"  KEGG found QS genes in {len(found_genera)} genera")
        for g in found_genera:
            hits = sum(1 for v in results[g].values() if v)
            families_hit = [f for f in QS_FAMILIES if results[g][f]]
            print(f"    {g}: {hits}/6 ({', '.join(families_hit)})")

    return results


# ── Merge & Build ────────────────────────────────────────────────────────────

def _merge_sources(
    ncbi: dict[str, dict[str, bool]],
    uniprot: dict[str, dict[str, bool]],
    kegg: dict[str, dict[str, bool]],
) -> dict[str, Any]:
    """Merge all three sources with OR logic."""
    families = list(QS_FAMILIES.keys())
    species_list = list(GUT_GENERA)

    presence: list[list[bool]] = []
    source_detail: dict[str, dict[str, list[str]]] = {}

    for genus in species_list:
        row: list[bool] = []
        genus_sources: dict[str, list[str]] = {}
        for fam in families:
            n = ncbi.get(genus, {}).get(fam, False)
            u = uniprot.get(genus, {}).get(fam, False)
            k = kegg.get(genus, {}).get(fam, False)
            merged = n or u or k
            row.append(merged)
            if merged:
                srcs = []
                if n:
                    srcs.append("ncbi_gene")
                if u:
                    srcs.append("uniprot")
                if k:
                    srcs.append("kegg")
                genus_sources[fam] = srcs
        presence.append(row)
        if genus_sources:
            source_detail[genus] = genus_sources

    return {
        "species": species_list,
        "families": families,
        "presence": presence,
        "source_detail": source_detail,
    }


def _build_provenance(sources_used: list[str]) -> dict[str, str]:
    """Build provenance block matching healthSpring baseline conventions."""
    return {
        "date": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "script": "data/fetch_qs_genes.py",
        "command": " ".join(sys.argv),
        "git_commit": _get_git_commit(),
        "python": sys.version.split()[0],
        "biopython": __import__("Bio").__version__,
        "sources": ", ".join(sources_used),
        "genera_count": str(len(GUT_GENERA)),
        "families_count": str(len(QS_FAMILIES)),
    }


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Fetch QS gene matrix from NCBI Gene + UniProt + KEGG"
    )
    parser.add_argument(
        "--force", action="store_true",
        help="Re-fetch even if output already exists",
    )
    parser.add_argument(
        "--dry-run", action="store_true",
        help="Print what would be fetched without making requests",
    )
    parser.add_argument(
        "--ncbi-only", action="store_true",
        help="Only query NCBI Gene (skip UniProt and KEGG)",
    )
    parser.add_argument(
        "--no-kegg", action="store_true",
        help="Skip KEGG queries (KEGG API is slow; use NCBI + UniProt only)",
    )
    args = parser.parse_args()

    data_root = Path(os.environ.get("HEALTHSPRING_DATA_ROOT", Path(__file__).parent))
    cold_storage = os.environ.get("HEALTHSPRING_COLD_STORAGE", "")
    output_path = data_root / "qs_gene_matrix.json"

    if cold_storage:
        cold_path = Path(cold_storage) / "qs_gene_matrix.json"
        if cold_path.exists() and not args.force:
            print(f"Found in cold storage: {cold_path}")
            if output_path != cold_path:
                import shutil
                output_path.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(cold_path, output_path)
                print(f"Copied to hot cache: {output_path}")
            return

    if output_path.exists() and not args.force:
        print(f"Output already exists: {output_path}")
        print("Use --force to re-fetch.")
        return

    if args.dry_run:
        print("Dry run — would fetch:")
        print(f"  Genera: {len(GUT_GENERA)}")
        print(f"  QS families: {len(QS_FAMILIES)}")
        sources = ["NCBI Gene"]
        if not args.ncbi_only:
            sources += ["UniProt reviewed", "KEGG Orthology"]
        print(f"  Sources: {', '.join(sources)}")
        print(f"  Output: {output_path}")
        return

    print("healthSpring QS Gene Matrix Builder (multi-source)")
    print("=" * 52)
    _setup_entrez()
    print(f"  Genera: {len(GUT_GENERA)}")
    print(f"  QS families: {len(QS_FAMILIES)}")
    sources_used = ["ncbi_gene"]
    if not args.ncbi_only and requests is not None:
        sources_used.append("uniprot")
        if not args.no_kegg:
            sources_used.append("kegg")
    print(f"  Sources: {', '.join(sources_used)}")
    print(f"  Output: {output_path}")

    # Source 1: NCBI Gene (always)
    ncbi_results = _ncbi_gene_scan()

    # Source 2: UniProt (unless --ncbi-only)
    if args.ncbi_only or requests is None:
        uniprot_results: dict[str, dict[str, bool]] = {
            g: {f: False for f in QS_FAMILIES} for g in GUT_GENERA
        }
    else:
        uniprot_results = _uniprot_qs_scan()

    # Source 3: KEGG (unless --ncbi-only or --no-kegg)
    if args.ncbi_only or args.no_kegg or requests is None:
        kegg_results: dict[str, dict[str, bool]] = {
            g: {f: False for f in QS_FAMILIES} for g in GUT_GENERA
        }
    else:
        kegg_results = _kegg_qs_scan()

    # Merge
    print("\n[Merging] OR across all sources...")
    matrix = _merge_sources(ncbi_results, uniprot_results, kegg_results)
    matrix["_provenance"] = _build_provenance(sources_used)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    json_str = json.dumps(matrix, indent=2, sort_keys=False)
    output_path.write_text(json_str + "\n")

    sha = hashlib.sha256(json_str.encode()).hexdigest()
    print()
    print(f"Wrote: {output_path}")
    print(f"SHA-256: {sha}")

    total_present = sum(1 for row in matrix["presence"] for v in row if v)
    total_cells = len(matrix["species"]) * len(matrix["families"])
    density = 100 * total_present / total_cells if total_cells else 0
    print(f"Matrix: {len(matrix['species'])} species × {len(matrix['families'])} families")
    print(f"QS gene density: {total_present}/{total_cells} ({density:.1f}%)")

    # Show improvement from multi-source
    ncbi_only_count = sum(
        1 for g in GUT_GENERA for f in QS_FAMILIES
        if ncbi_results.get(g, {}).get(f, False)
    )
    new_from_uniprot = sum(
        1 for g in GUT_GENERA for f in QS_FAMILIES
        if uniprot_results.get(g, {}).get(f, False)
        and not ncbi_results.get(g, {}).get(f, False)
    )
    new_from_kegg = sum(
        1 for g in GUT_GENERA for f in QS_FAMILIES
        if kegg_results.get(g, {}).get(f, False)
        and not ncbi_results.get(g, {}).get(f, False)
        and not uniprot_results.get(g, {}).get(f, False)
    )
    print(f"\nSource contribution:")
    print(f"  NCBI Gene:  {ncbi_only_count} hits")
    print(f"  UniProt:    +{new_from_uniprot} new (not in NCBI)")
    print(f"  KEGG:       +{new_from_kegg} new (not in NCBI or UniProt)")
    print(f"  Total:      {total_present}")

    # Flag key genera that gained coverage
    print("\nNotable coverage changes:")
    for genus in ["Bacteroides", "Prevotella", "Faecalibacterium", "Akkermansia",
                   "Bifidobacterium", "Ruminococcus"]:
        ncbi_hits = sum(1 for f in QS_FAMILIES if ncbi_results.get(genus, {}).get(f, False))
        total_hits = sum(1 for row_idx, g in enumerate(GUT_GENERA) if g == genus
                         for col_idx, v in enumerate(matrix["presence"][row_idx]) if v)
        if total_hits > ncbi_hits:
            print(f"  {genus}: {ncbi_hits} (NCBI) → {total_hits} (merged)")


if __name__ == "__main__":
    main()
