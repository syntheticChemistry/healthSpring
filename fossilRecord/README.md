<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# fossilRecord — healthSpring Archived Sources

Prokaryotic-era source code preserved for traceability. When healthSpring
evolves (absorbs experiment mains into validation scenarios, consolidates
binaries into UniBin, etc.), the original sources are archived here with
dated subdirectories rather than deleted.

These files are **not compiled** — they exist as a fossil record. The
authoritative, compiled code lives in `ecoPrimal/src/`.

## Contents

Archived sources were consolidated upstream to `ecoPrimals/fossilRecord/` at V61.
Locally archived tools are retained here as provenance artifacts.

| Archive | What Was Archived | When |
|---------|-------------------|------|
| `experiments_prokaryotic_may2026/` | 16 experiment `main.rs` files absorbed into `validation/scenarios/` | V61 (May 2026) |
| `guidestone_prokaryotic_may2026/` | Standalone `healthspring_guidestone` binary sources absorbed into `certification/` organelle | V61 (May 2026) |
| `py_to_notebook.py` | One-shot Python-to-notebook converter (53 notebooks generated; tool complete) | V65a (May 2026) |
| `composition_template.sh` | primalSpring composition starter (superseded by upstream tooling) | V65a (May 2026) |
| `composition_nucleus.sh` | Single-family NUCLEUS launcher (superseded by `plasmidBin/nucleus_launcher.sh` + `cell_launcher.sh`) | V65a (May 2026) |

The V61 directories lived here briefly during V61 migration, then moved to the
ecosystem-level fossil record. This README remains as the provenance pointer.

## Policy

Archives are append-only. New subdirectories are added when code is
absorbed or retired; existing archives are never modified.
