# healthSpring White Paper

**Date:** March 10, 2026
**Status:** V20 — 5 tracks + diagnostics + GPU + visualization + clinical TRT + streaming + interaction + NLME + V16 primitives + GPU scaling + toadStool dispatch + mixed NUCLEUS, 61 experiments validated (Tier 0+1+2+3). Full-stack portability: barraCuda CPU → GPU (6 WGSL shaders) → toadStool dispatch → metalForge NUCLEUS routing (PCIe P2P bypass). Rust 84× faster than Python (V18). 395 tests, 194 Python cross-validation checks. V20: petalTongue V16 visualization (34-node full study, 16 scenarios, patient explorer).
**License:** AGPL-3.0-or-later

---

## Document Index

| Document | Purpose | Audience |
|----------|---------|----------|
| [STUDY.md](STUDY.md) | Main narrative — abstract, domain rationale, validation plan | Reviewers, collaborators, clinicians |
| [METHODOLOGY.md](METHODOLOGY.md) | Validation protocol — four-tier design, acceptance criteria | Technical validation |

---

## Study Questions

1. Can validated PK/PD algorithms (Hill, compartmental, population Monte Carlo) be implemented
   in Pure Rust with GPU acceleration, replacing Python/NONMEM/R dependency
   chains while maintaining documented numerical tolerances?

2. Can the Anderson localization framework validated in soil microbiome
   (wetSpring) be transferred to gut microbiome colonization resistance,
   predicting *C. difficile* risk from 16S profiles?

3. Can real-time biosignal processing (ECG QRS detection) run on sovereign
   hardware via barraCuda at clinically useful latency?

4. Can quantifiable clinical claims about Testosterone Replacement Therapy
   be systematically validated against published registry data, creating
   a reusable "claim verification" pipeline?

5. Can GPU-accelerated population PK simulations (Monte Carlo, virtual
   patients) provide clinically actionable dosing recommendations faster
   than existing tools?

---

## baseCamp Sub-Theses

| Sub-thesis | Domain | Faculty | Status |
|-----------|--------|---------|--------|
| [gonzales.md](baseCamp/gonzales.md) | PK/PD + immunology extensions to human therapeutics | Gonzales, Lisabeth, Neubig | **Complete** — Exp001-006 (Track 1) |
| [mok_testosterone.md](baseCamp/mok_testosterone.md) | TRT claim verification + endocrine modeling | Mok (Allure Medical) | **Complete** — Exp030-038 (Track 4) |
| [cdiff_colonization.md](baseCamp/cdiff_colonization.md) | Anderson localization → gut colonization resistance | (TBD) | **Complete** — Exp010-013 (Track 2) |
| [biosignal_sovereign.md](baseCamp/biosignal_sovereign.md) | Edge biosignal processing | (TBD) | **Complete** — Exp020-023 (Track 3) |

---

## baseCamp

The `baseCamp/` directory contains independent sub-thesis documents. Each represents a
faculty-linked line of inquiry that produced validated experiments. See the
[baseCamp README](baseCamp/README.md) for the experiment inventory and validation summary.
