# Fossil Record: Absorbed Experiments (Prokaryotic)

**Archived:** May 9, 2026  
**Reason:** Representative experiments absorbed into `validation/scenarios/` module

## Provenance Table

| Experiment | Scenario ID | Track | Tier |
|---|---|---|---|
| exp001 | hill-dose-response | PkPd | Rust |
| exp002 | one-compartment-pk | PkPd | Rust |
| exp005 | population-pk | PkPd | Rust |
| exp010 | diversity-indices | Microbiome | Rust |
| exp011 | anderson-gut | Microbiome | Rust |
| exp020 | pan-tompkins-qrs | Biosignal | Rust |
| exp021 | hrv-metrics | Biosignal | Rust |
| exp030 | testosterone-pk | Endocrine | Rust |
| exp077 | michaelis-menten | PkPd | Rust |
| exp090 | matrix-scoring | Discovery | Rust |
| exp100 | canine-il31 | Comparative | Rust |
| exp119 | composition-parity | Composition | Live |
| exp120 | live-provenance | Composition | Live |
| exp121 | live-health | Composition | Live |
| exp122 | barracuda-parity | Composition | Live |
| exp123 | nucleus-parity | Composition | Both |

## Notes

- The original experiment binaries are NOT deleted — they remain as standalone
  binaries in `experiments/`. Only the main.rs files are archived here as a
  record of the code at the time of absorption.
- The absorbed logic now lives in `ecoPrimal/src/validation/scenarios/s_*.rs`.
- 79 remaining experiments are not absorbed — they remain standalone.
