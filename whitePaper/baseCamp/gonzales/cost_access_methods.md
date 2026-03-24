<!-- SPDX-License-Identifier: CC-BY-SA-4.0 (scyBorg: AGPL-3.0 code + ORC mechanics + CC-BY-SA-4.0 creative) -->
# Cost, Access, and Methods — Gonzales Lab Comparative Pharmacology

**Last Updated**: March 22, 2026
**Audience**: Gonzales lab members, ADDRC collaborators, veterinary pharmacologists
entering the healthSpring ecosystem.

---

## 1. Cost Comparison

### Traditional Veterinary → Human PK Pipeline

| Line Item | Typical Cost | Notes |
|-----------|------------:|-------|
| NONMEM license | $2,000/yr | Population PK estimation |
| Monolix license | $1,500/yr | SAEM estimation |
| WinNonlin license | $3,000/yr | NCA analysis |
| SAS/R/Python infra | $500–5,000/yr | Statistical computing |
| CRO population PK analysis | $50,000–200,000 | Per drug program |
| Allometric scaling (manual) | Weeks of analyst time | Per species bridge |
| **Typical per-drug pipeline** | **$75,000–$210,000** | Conservative estimate |

### healthSpring Equivalent

| Line Item | Cost | Notes |
|-----------|-----:|-------|
| FOCE estimation | $0.00 | Sovereign replacement (Exp075, validated) |
| SAEM estimation | $0.00 | Sovereign replacement (Exp075, validated) |
| NCA analysis | $0.00 | Sovereign replacement (Exp075, validated) |
| Population PK Monte Carlo (GPU) | ~$0.001 | Electricity for 100K patients |
| Cross-species allometric bridge | $0.00 | Parameterized, not manual |
| Hill dose-response panel | ~$0.001 | GPU Hill sweep |
| **Per-drug pipeline** | **~$0.01** | Including GPU compute |

### What the Lab Gets

For every compound Gonzales lab tests in iPSC skin models, healthSpring provides:
- Pre-screening MATRIX + Anderson score (computational triage)
- Population PK prediction (100K virtual patients, per-species)
- Tissue penetration prediction (Anderson geometry)
- Dosing recommendation (AUC, Cmax, trough optimization)
- Cross-species bridge (canine → human parameter translation)
- Microbiome impact prediction (gut dysbiosis risk)

All computed before any wet-lab resources are consumed.

---

## 2. Data Source Comparison

### What Gonzales Lab Has

| Source | Type | Access |
|--------|------|--------|
| G1–G6 published papers | Canine IC50, PK, dose-response, pruritus | Open (peer-reviewed journals) |
| iPSC skin model readouts | Viability, cytokine levels | Lab-internal (collaboration) |
| Zoetis compound data (18 years) | Proprietary veterinary PK | Restricted (prior employer) |
| ADDRC compound library | 8,000 compounds, HTS-ready | Collaboration (MSU) |

### What healthSpring Adds

| Source | Type | Access | Gonzales Lab Benefit |
|--------|------|--------|---------------------|
| ChEMBL (2M+ bioactivities) | IC50/Ki/EC50 across targets | Open (EBI) | Benchmark iPSC IC50 against literature |
| FDA CVM Green Book | All approved veterinary drugs | Open (FDA) | Cross-reference canine PK parameters |
| NCBI Gene (QS families) | Microbial signaling genes | Open (NCBI) | Connect drug effects to microbiome |
| Published PK parameter sets | CL, Vd, t½ for 100s of drugs | Open (literature) | Species-agnostic PK modeling |
| wetSpring 16S pipeline | Gut microbiome analytics | Open (ecoPrimals) | Anderson colonization resistance |
| neuralSpring nS-601–605 | Gonzales papers, validated | Open (ecoPrimals) | Foundation for all extensions |

### Key Insight

Gonzales lab produces high-quality experimental data (iPSC, canine models).
healthSpring wraps that data in a computational pipeline that multiplies its value:
every iPSC readout feeds population PK, cross-species modeling, and MATRIX scoring.
The experimental data becomes computable across the entire drug × disease × species space.

---

## 3. Methods Comparison

### Traditional Veterinary Pharmacology Pipeline

```
1. Canine study → IC50, PK parameters (months, $100K+)
2. Manual allometric scaling → human parameter estimates (weeks)
3. NONMEM/Monolix → population PK (weeks, $50K+ CRO)
4. WinNonlin → NCA metrics (days)
5. Manual report → regulatory filing
```

### healthSpring Pipeline

```
1. Published IC50/PK ingested → Python baseline → Rust validation (hours, $0)
2. Species-agnostic PK model parameterized (minutes, $0)
3. GPU population PK Monte Carlo → 100K patients (seconds, ~$0.001)
4. Anderson-augmented MATRIX scoring (seconds, ~$0.001)
5. petalTongue visualization → clinical scenario (seconds, $0)
6. Cross-species validation → allometric bridge (automatic, $0)
```

### What Changes for the Lab

| Task | Before | After (healthSpring) |
|------|--------|---------------------|
| Population PK | CRO, weeks, $50K+ | GPU, seconds, $0 |
| NCA analysis | WinNonlin, days, $3K/yr | Rust NCA, instant, $0 |
| Cross-species scaling | Manual spreadsheet, weeks | Automated allometric bridge, instant |
| Drug candidate ranking | Pathway intuition | MATRIX + Anderson quantitative score |
| Compound triage for iPSC | Literature review, weeks | GPU Hill sweep (8K compounds), seconds |
| Clinical scenario generation | Manual case report | petalTongue patient explorer, automatic |

---

## 4. Access

All healthSpring tools are available immediately:

```bash
git clone <healthSpring repo>
cargo test --workspace        # 863 tests, zero unsafe code
cargo run --bin exp001_hill   # Hill dose-response (Gonzales IC50 data)
cargo run --bin exp004_mab    # mAb cross-species PK (lokivetmab → nemolizumab)
cargo run --bin exp075_nlme   # FOCE/SAEM population PK (sovereign NONMEM)
```

No licenses. No cloud accounts. No CRO contracts. No Python dependencies.

### What Gonzales Lab Should Do First

1. **Run the existing experiments** — see your published data reproduced (Exp001–006)
2. **Test with new iPSC IC50 data** — feed IC50 values into `hill_dose_response`
3. **Run population PK on a candidate** — `population_pk_cpu` with compound parameters
4. **Review MATRIX scores** — see how Anderson geometry reranks your candidate set
5. **Discuss ADDRC integration** — how computational triage feeds Lisabeth's HTS
