# Fossil Record: healthspring_guidestone (Prokaryotic)

**Archived:** May 9, 2026  
**Reason:** Absorbed into `certification/` organelle as part of eukaryotic evolution  
**Standard:** GUIDESTONE_COMPOSITION_STANDARD v1.2.0

## Migration Table

| Old (Binary) | New (Library Module) |
|---|---|
| `healthspring_guidestone` binary | `healthspring_barracuda::certification::certify()` |
| `bare.rs` → `validate_*` functions | `certification/bare.rs` |
| `domain.rs` → `validate_domain_science` | `certification/domain.rs` |
| `domain.rs` → IPC validation | `certification/composition.rs` |
| `main.rs` → orchestrator | `certification/mod.rs::certify()` |

## Legacy Usage

The binary at `ecoPrimal/src/bin/healthspring_guidestone/main.rs` now delegates
to the library module. It is retained for backward compatibility but deprecated
in favor of `healthspring_unibin certify`.

## Three-Tier Validation

- **Tier 1 (LOCAL):** Bare properties 1–5 + domain science
- **Tier 2 (IPC-WIRED):** barraCuda math IPC parity, manifest capabilities
- **Tier 3 (FULL NUCLEUS):** Primal proof — full science parity through IPC
