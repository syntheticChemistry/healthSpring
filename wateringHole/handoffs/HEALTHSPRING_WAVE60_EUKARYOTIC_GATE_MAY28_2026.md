# healthSpring — Wave 60 Eukaryotic Gate Onboarding

**Date:** May 28, 2026
**From:** healthSpring V65a (ironGate)
**Wave:** 60 — Eukaryotic Gate Onboarding
**License:** AGPL-3.0-or-later

---

## Context

primalSpring Wave 60 declares the eukaryotic unicellular state: each
gate is an independent cell with NUCLEUS running, springs validating,
and syncing through the VPS periplasm (Forgejo at `git.primals.eco`).
ironGate was already onboarded — this wave validates the cascade-pull
pattern and fixes local debt found during the review.

## Actions Taken

### 1. Forgejo Sync Verified

- **Forgejo remote**: Already configured on healthSpring
  (`ssh://git@git.primals.eco:2222/syntheticChemistry/healthSpring.git`)
- **cascade-pull dry-run**: `GATE_NAME=ironGate cascade-pull.sh --gate auto --dry-run`
  → 22 repos in ironGate profile, 20 found on disk, 2 MISSING (see gaps)
- **cascade-pull live**: `--source forgejo` pulled successfully —
  18 CURRENT, 2 UPDATED (cellMembrane, projectNUCLEUS), 0 FAILED
- **ensure-remotes**: All 20 available repos have `forgejo` remote configured

### 2. Bug Fix — `scripts/visualize.sh` ECO_ROOT Ordering

`ECO_ROOT` was used on line 59 (petalTongue sandbox sync) before being
defined on line 72. This caused the sync block to silently skip on
first invocation. Fixed by moving `ECO_ROOT` definition to immediately
after `PROJECT_ROOT` (line 19), removing the duplicate later definition.

### 3. Version String Alignment — ecoBin

`capabilities.rs` printed "ecoBin v3.0" while all docs reference
"ecoBin 0.9.0". Clarified to "ecoBin 0.9.0 (spec v3.0)" — the 0.9.0
is the package version, v3.0 is the ecoBin specification version.

### 4. CONTEXT.md Gate Deployment — Launcher Path Fix

The `Launch` row referenced `./tools/nucleus_launcher.sh` and
`./tools/cell_launcher.sh` which don't exist under healthSpring.
Corrected to `../primalSpring/tools/nucleus_launcher.sh` and
`../primalSpring/tools/cell_launcher.sh`.

### 5. Doc Sweep — Wave 60 / May 28

- **21 active docs** updated from May 25 → May 28, 2026
- **12+ status banners** updated from "Wave 50 Covalent HPC" to
  "Wave 60 Eukaryotic Gate"
- README, CONTEXT, wateringHole, specs, whitePaper, experiments all current
- Cargo.toml pin comment updated to Wave 60

## Current Status

| Metric | Value |
|--------|-------|
| **Gate** | ironGate |
| **NUCLEUS** | 13/13 (all primals via plasmidBin) |
| **Tests** | 1,021 (all passing) |
| **Scenarios** | 57 |
| **Capabilities** | 88 |
| **Forgejo sync** | Live — cascade-pull 22-repo profile |
| **Deep debt** | Zero (all 7 categories) |
| **Clippy** | Zero warnings (pedantic+nursery) |

## Upstream Gaps for Primal Teams

### GAP-W60-1: `ecosystem_manifest.toml` songbird path case mismatch

The manifest defines `repos.songbird` with `local_path = "primals/songbird"`
(lowercase b), but the actual directory on ironGate is `primals/songBird`
(capital B). This causes cascade-pull to report MISSING for songbird on
every gate.

**Fix**: Update manifest `local_path` to `"primals/songBird"` or rename
the directory to match.

### GAP-W60-2: `ecosystem_manifest.toml` nestGate path is workspace root

`repos.nestGate` has `local_path = "."` which resolves to the ecoPrimals
workspace root. That directory is not a git repository (no `.git/`), so
cascade-pull always reports MISSING.

**Fix**: If nestGate lives inside the workspace root's git repo, update
`local_path` to the actual subdirectory. If the workspace root IS the
nestGate repo, the root needs `git init`.

---

*healthSpring Wave 60: ironGate eukaryotic. 13/13 NUCLEUS, Forgejo sync live, cascade-pull 22-repo. Yeast runs the biosphere.*
