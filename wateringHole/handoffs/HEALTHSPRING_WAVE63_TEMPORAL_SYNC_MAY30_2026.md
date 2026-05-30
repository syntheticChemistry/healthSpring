# healthSpring Wave 63 — Temporal Sync + BTSP Probe + pseudoSpore Profile

**Date**: May 30, 2026
**Gate**: ironGate (operational)
**From**: healthSpring
**Wave**: 63 — River Delta syntheticChemistry Temporal Sync

---

## Summary

healthSpring absorbs Wave 63 delta-wide directives. All three assigned tasks complete:

1. **Temporal sync** — `cascade-pull.sh --source temporal` operational. 21/22 repos synced. healthSpring at PARITY on both remotes.
2. **BTSP `btsp.capabilities` probe** — `probe_btsp_capabilities()` + `should_upgrade_btsp()` implemented in `ipc/btsp.rs`. Resolves Gap #20.
3. **`domain_profile.toml`** — Clinical PK-PD drug interaction profile authored. Ready for `litho emit-pseudospore`.

---

## Delta-Wide Compliance

| Directive | healthSpring Status |
|-----------|-------------------|
| Temporal sync tooling | DONE — `--source temporal` verified, 21/22 synced |
| CONTEXT.md drift | CLEAN — not listed as dirty |
| composition_nucleus.sh debt | N/A — already fossilized |
| pseudoSpore domain profile | DONE — `domain_profile.toml` at spring root |

---

## BTSP Probe Pattern

New public API in `ecoPrimal/src/ipc/btsp.rs`:

```rust
pub fn probe_btsp_capabilities(socket_path: &Path) -> Option<BtspCapabilities>
pub fn should_upgrade_btsp(socket_path: &Path) -> bool
```

Pattern: connect cleartext → call `btsp.capabilities` JSON-RPC → if `server: true`, proceed with handshake; otherwise stay cleartext. Prevents Gap #20 (BTSP-unaware primals misparse `ClientHello`).

4 new tests added (btsp module: 10 total).

---

## domain_profile.toml

6 entity groups:
- `pbpk_compartments` — organ-level ADME via compartmental ODE
- `pd_response` — Hill equation dose-response
- `drug_interaction` — CYP450 inhibition/induction, AUC fold-change
- `microbiome_metabolism` — Anderson localization + Michaelis-Menten
- `population_pk` — GPU Monte Carlo via barraCuda
- `symbiont_pkpd` — LTEE B5 engineered symbiont model

5 derivation pipelines, 8 audit checks, 5 figure specifications.

Next: `litho emit-pseudospore --spring healthSpring --domain-profile ./domain_profile.toml`

---

## Metrics

| Metric | Before | After |
|--------|--------|-------|
| Tests | 1,052 | 1,056 |
| BTSP probe | not implemented | `probe_btsp_capabilities` + `should_upgrade_btsp` |
| domain_profile.toml | absent | authored (6 modules, 8 checks) |
| Temporal sync | untested | operational (21/22 CONVERGE/PARITY) |
| Gap #20 | open | RESOLVED |
| Clippy warnings | 0 | 0 |

---

## Upstream Notes

- **toadStool DIVERGE**: forgejo and origin have diverged commits. Needs quorumSignal review (not healthSpring concern).
- **pseudoSpore emission**: Profile authored but `litho emit-pseudospore` requires lithoSpore CLI and NestGate `content.put`. Will emit once `litho` binary is available in plasmidBin.
- **Forgejo mirror conversion**: healthSpring is priority 5-8 (lower priority, pull-only gate). Awaiting upstream membrane conversion.
