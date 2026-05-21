<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# healthSpring V64n — Upstream Audit Absorption Handoff

**Date**: May 14, 2026
**From**: healthSpring (Nest Atomic Specialist)
**To**: primalSpring, infra/plasmidBin
**Trigger**: primalSpring ecosystem status update — May 14, 2026

---

## What Changed

healthSpring absorbed the May 14 upstream directive: Tower atomic mandated as
bearDog + songBird + skunkBat; barraCuda v0.4.0 released; sourDough internalization planned.

### Local Fixes (all in this commit)

1. **Tower atomic in deploy graphs** — All 4 deploy-style TOML graphs updated to include
   skunkBat in Tower atomic comments, `depends_on`, and section placement.
2. **`healthspring_nest_atomic.toml`** — Stale `defense.audit`/`defense.recon`/`defense.threat`
   updated to `security.audit_log`/`baseline.observe`/`baseline.anomaly`.
3. **`healthspring_niche_deploy.toml`** — rhizoCrypt, loamSpine, sweetGrass wire names
   canonicalized. skunkBat `by_capability` → `"audit"`.
4. **`routing.rs`** — Added `"content"` domain → NestGate (CAS surface); `"stats"` to `ALL_CAPS`.
5. **`niche.rs` CONSUMED_CAPABILITIES** — Canonical wire names throughout. Added
   `crypto.contract.*`, `content.*`. Removed legacy `crypto.ionic_bond`, `audit.log`.
6. **`capability_registry.toml`** — Full sync with canonical wire names and current
   skunkBat/sweetGrass/rhizoCrypt surfaces.
7. **Cargo.toml** — barraCuda version comment updated to v0.4.0.

### Upstream Items (hand back to primalSpring / infra)

| # | Item | Owner | Action Needed |
|---|------|-------|---------------|
| GAP-43 | `infra/plasmidBin/manifest.toml` healthSpring entry stale: `tests = 1014`, note `V64e` | infra/plasmidBin | Update to `tests = 1018`, `V64n`, `evolution = "composing"` |
| GAP-44 | `ports.env` `NICHE_HEALTHSPRING` only lists 8 primals, missing toadstool/barracuda/coralreef/petaltongue | infra/plasmidBin | Sync with `[niches.healthspring]` in manifest.toml (12 primals) |
| GAP-45 | sourDough internalization: 15 shell scripts in healthSpring are candidates | primals/sourDough | Map `tools/composition_*.sh` → `sourdough validate composition`; `data/fetch_*.sh` → `sourdough harvest`; `tools/check_method_strings.sh` → `sourdough validate` |

### Composing → Composed Blockers

All remaining blockers are upstream/coordination — healthSpring has no local debt:

- **Ionic bridge** (GAP-2) — not implemented upstream
- **BTSP transport** (GAP-20) — `FAMILY_SEED` breaks mixed deploys
- **Foundation Thread 10** (GAP-42) — awaiting sporeGarden structure
- **Nest live deploy** — needs running primals for live `s_nest_atomic`

### barraCuda v0.4.0

healthSpring uses path deps that resolve to the upstream v0.4.0 workspace. No API
breakage detected — all 1,018 tests pass, 0 clippy warnings. The precision/E2E and
VFIO sovereign dispatch features are available but not yet exercised by healthSpring
(compute trio integration is a future sprint).

---

## Metrics

| Metric | Value |
|--------|-------|
| Tests (workspace) | 1,018 |
| Clippy warnings | 0 |
| TODO/FIXME | 0 |
| Open local gaps | 0 |
| Upstream gaps documented | 3 (GAP-43, GAP-44, GAP-45) |
