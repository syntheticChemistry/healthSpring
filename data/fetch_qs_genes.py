#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Fetch quorum-sensing gene presence/absence matrix from NCBI Datasets API.

Queries NCBI Gene for QS gene families (luxI/R, luxS, agr, com, las/rhl, fsr)
across ~200 gut-associated bacterial species. Produces a structured JSON matrix
suitable for Anderson lattice augmentation (exp107: QS-augmented Anderson).

Usage:
    python3 data/fetch_qs_genes.py [--output data/qs_gene_matrix.json]

Requires: requests (pip install requests)
Output: JSON file with structure:
    {
        "meta": { "fetched_at": "...", "gene_families": [...], ... },
        "species": [...],
        "matrix": { "species_name": { "gene_family": 0|1, ... }, ... }
    }
"""

import argparse
import hashlib
import json
import os
import sys
import time
from pathlib import Path

try:
    import requests
except ImportError:
    print("ERROR: 'requests' package required. Install: pip install requests", file=sys.stderr)
    sys.exit(1)

NCBI_GENE_API = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils"

QS_GENE_FAMILIES = {
    "luxI": ["luxI", "ainS", "yenI"],
    "luxR": ["luxR", "sdiA"],
    "luxS": ["luxS"],
    "agrA": ["agrA"],
    "agrB": ["agrB"],
    "agrC": ["agrC"],
    "agrD": ["agrD"],
    "comA": ["comA"],
    "comB": ["comB"],
    "comC": ["comC"],
    "comD": ["comD"],
    "comE": ["comE"],
    "lasI": ["lasI"],
    "lasR": ["lasR"],
    "rhlI": ["rhlI"],
    "rhlR": ["rhlR"],
    "fsrA": ["fsrA"],
    "fsrB": ["fsrB"],
    "fsrC": ["fsrC"],
}

GUT_SPECIES = [
    "Bacteroides fragilis", "Bacteroides thetaiotaomicron", "Bacteroides vulgatus",
    "Bacteroides uniformis", "Bacteroides ovatus", "Bacteroides caccae",
    "Prevotella copri", "Prevotella melaninogenica",
    "Faecalibacterium prausnitzii",
    "Eubacterium rectale", "Eubacterium hallii",
    "Roseburia intestinalis", "Roseburia hominis",
    "Ruminococcus bromii", "Ruminococcus gnavus",
    "Coprococcus eutactus", "Coprococcus comes",
    "Blautia obeum", "Blautia producta",
    "Akkermansia muciniphila",
    "Bifidobacterium longum", "Bifidobacterium adolescentis", "Bifidobacterium breve",
    "Lactobacillus rhamnosus", "Lactobacillus plantarum", "Lactobacillus acidophilus",
    "Escherichia coli", "Klebsiella pneumoniae",
    "Enterococcus faecalis", "Enterococcus faecium",
    "Clostridium difficile", "Clostridioides difficile",
    "Streptococcus mutans", "Streptococcus pneumoniae",
    "Staphylococcus aureus", "Staphylococcus epidermidis",
    "Pseudomonas aeruginosa",
    "Vibrio cholerae", "Vibrio harveyi",
    "Fusobacterium nucleatum",
]


def ncbi_esearch(gene_name: str, organism: str) -> int:
    """Search NCBI Gene for a gene in an organism. Returns hit count."""
    query = f'{gene_name}[Gene Name] AND "{organism}"[Organism]'
    params = {
        "db": "gene",
        "term": query,
        "retmode": "json",
        "retmax": 1,
    }
    try:
        resp = requests.get(f"{NCBI_GENE_API}/esearch.fcgi", params=params, timeout=30)
        resp.raise_for_status()
        data = resp.json()
        return int(data.get("esearchresult", {}).get("count", 0))
    except (requests.RequestException, ValueError, KeyError):
        return 0


def build_matrix(rate_limit_ms: int = 350) -> dict:
    """Build gene presence/absence matrix by querying NCBI Gene."""
    matrix = {}
    total_queries = len(GUT_SPECIES) * len(QS_GENE_FAMILIES)
    done = 0

    for species in GUT_SPECIES:
        row = {}
        for family, gene_names in QS_GENE_FAMILIES.items():
            present = 0
            for gene in gene_names:
                count = ncbi_esearch(gene, species)
                if count > 0:
                    present = 1
                    break
                time.sleep(rate_limit_ms / 1000.0)
            row[family] = present
            done += 1
            pct = 100.0 * done / total_queries
            print(f"  [{done}/{total_queries}] ({pct:.0f}%) {species} / {family} = {present}",
                  file=sys.stderr)
        matrix[species] = row

    return matrix


def main():
    parser = argparse.ArgumentParser(description="Fetch QS gene matrix from NCBI")
    parser.add_argument("--output", default=None,
                        help="Output JSON path (default: data/qs_gene_matrix.json)")
    parser.add_argument("--dry-run", action="store_true",
                        help="Print config and exit without querying NCBI")
    args = parser.parse_args()

    script_dir = Path(__file__).resolve().parent
    output_path = Path(args.output) if args.output else script_dir / "qs_gene_matrix.json"

    if args.dry_run:
        print(f"Would fetch {len(GUT_SPECIES)} species × {len(QS_GENE_FAMILIES)} gene families")
        print(f"Output: {output_path}")
        print(f"Total NCBI queries: ~{len(GUT_SPECIES) * len(QS_GENE_FAMILIES)}")
        return

    print(f"Fetching QS gene matrix: {len(GUT_SPECIES)} species × {len(QS_GENE_FAMILIES)} families",
          file=sys.stderr)

    matrix = build_matrix()

    result = {
        "meta": {
            "fetched_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "gene_families": list(QS_GENE_FAMILIES.keys()),
            "species_count": len(GUT_SPECIES),
            "source": "NCBI Gene (eutils)",
            "license": "Public domain (NCBI)",
        },
        "species": GUT_SPECIES,
        "matrix": matrix,
    }

    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w") as f:
        json.dump(result, f, indent=2)

    sha256 = hashlib.sha256(output_path.read_bytes()).hexdigest()
    print(f"Wrote {output_path} ({os.path.getsize(output_path)} bytes)", file=sys.stderr)
    print(f"SHA-256: {sha256}", file=sys.stderr)
    print(sha256)


if __name__ == "__main__":
    main()
