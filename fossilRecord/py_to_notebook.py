#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Convert Python control scripts to .ipynb notebooks with paper linkage.

Creates paired notebooks alongside the .py sources. Each notebook gets:
- A markdown cell with paper linkage (DOI, experiment reference, track)
- The script body split into logical cells at comment-delimited sections
- Jupytext-compatible metadata pairing the .py and .ipynb

Usage: python3 tools/py_to_notebook.py [--dry-run]
"""

import json
import os
import re
import sys
from pathlib import Path

CONTROL_DIR = Path(__file__).resolve().parent.parent / "control"

TRACK_PAPERS = {
    "pkpd": "Track 1 — Pharmacokinetics / Pharmacodynamics",
    "microbiome": "Track 2 — Microbiome Analytics",
    "biosignal": "Track 3 — Biosignal Processing",
    "endocrine": "Track 4 — Endocrine & TRT",
    "comparative": "Track 5 — Comparative Medicine",
    "discovery": "Track 6 — Drug Discovery & Repurposing",
    "validation": "Track 7 — Cross-Validation",
    "scripts": "Track 8 — Benchmark & Utility Scripts",
    "toxicology": "Track 9 — Toxicology",
    "simulation": "Track 9 — Simulation",
}


def extract_docstring(source: str) -> str:
    """Extract the module-level docstring if present."""
    match = re.match(r'^(?:#.*\n)*\s*(?:\'\'\'|""")(.+?)(?:\'\'\'|""")', source, re.DOTALL)
    if match:
        return match.group(1).strip()
    for line in source.split("\n"):
        stripped = line.strip()
        if stripped.startswith("# ") and not stripped.startswith("# SPDX"):
            return stripped[2:]
        if stripped and not stripped.startswith("#"):
            break
    return ""


def extract_experiment_id(filename: str) -> str:
    """Extract exp### from filename."""
    match = re.search(r"(exp\d+)", filename)
    return match.group(1) if match else ""


def split_into_cells(source: str) -> list:
    """Split source into logical cells at section markers."""
    lines = source.split("\n")
    cells = []
    current = []
    in_header = True

    for line in lines:
        stripped = line.strip()
        if stripped.startswith("# SPDX") or stripped.startswith("#!/"):
            continue
        if in_header and (stripped.startswith('"""') or stripped.startswith("'''")):
            continue
        in_header = False

        is_section = (
            stripped.startswith("# ──")
            or stripped.startswith("# ==")
            or stripped.startswith("# --")
            or (stripped.startswith("# ") and stripped.endswith(" ──"))
        )

        if is_section and current:
            cells.append("\n".join(current))
            current = []

        current.append(line)

    if current:
        content = "\n".join(current).strip()
        if content:
            cells.append(content)

    return cells if cells else [source]


def make_notebook(py_path: Path, track: str, dry_run: bool = False) -> Path:
    """Convert a .py file to a paired .ipynb notebook."""
    source = py_path.read_text()
    exp_id = extract_experiment_id(py_path.name)
    description = extract_docstring(source)
    track_label = TRACK_PAPERS.get(track, f"Track — {track}")

    header_lines = [
        f"# {py_path.stem}",
        "",
        f"**{track_label}**",
        "",
    ]
    if exp_id:
        header_lines.append(f"**Experiment:** `{exp_id}`")
    if description:
        header_lines.append(f"**Description:** {description}")
    header_lines.extend([
        "",
        f"**Source:** `{py_path.relative_to(CONTROL_DIR.parent)}`",
        "",
        "## Paper Linkage",
        "",
        "See `specs/PAPER_REVIEW_QUEUE.md` for the full paper → experiment mapping.",
        f"This notebook is the `.ipynb` pair of the `.py` control script for `{exp_id or py_path.stem}`.",
    ])

    cells = [
        {
            "cell_type": "markdown",
            "metadata": {},
            "source": [line + "\n" for line in header_lines],
        }
    ]

    code_cells = split_into_cells(source)
    for cell_source in code_cells:
        stripped = cell_source.strip()
        if not stripped:
            continue
        cells.append({
            "cell_type": "code",
            "execution_count": None,
            "metadata": {},
            "outputs": [],
            "source": [line + "\n" for line in stripped.split("\n")],
        })

    notebook = {
        "cells": cells,
        "metadata": {
            "kernelspec": {
                "display_name": "Python 3",
                "language": "python",
                "name": "python3",
            },
            "language_info": {
                "name": "python",
                "version": "3.10.0",
            },
            "jupytext": {
                "formats": "ipynb,py:percent",
                "text_representation": {
                    "extension": ".py",
                    "format_name": "percent",
                },
            },
        },
        "nbformat": 4,
        "nbformat_minor": 5,
    }

    nb_path = py_path.with_suffix(".ipynb")
    if not dry_run:
        nb_path.write_text(json.dumps(notebook, indent=1, ensure_ascii=False) + "\n")

    return nb_path


def main():
    dry_run = "--dry-run" in sys.argv
    converted = 0

    for py_path in sorted(CONTROL_DIR.rglob("*.py")):
        rel = py_path.relative_to(CONTROL_DIR)
        parts = rel.parts
        track = parts[0] if len(parts) > 1 else "scripts"

        if py_path.name == "__init__.py":
            continue

        nb_path = make_notebook(py_path, track, dry_run=dry_run)
        status = "would create" if dry_run else "created"
        print(f"  {status}: {nb_path.relative_to(CONTROL_DIR.parent)}")
        converted += 1

    print(f"\n{'Would convert' if dry_run else 'Converted'} {converted} scripts to notebooks.")


if __name__ == "__main__":
    main()
