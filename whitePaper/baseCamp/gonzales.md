# healthSpring baseCamp: Gonzales PK/PD → Human Therapeutics

**Faculty**: Andrea J. Gonzales (MSU Pharmacology & Toxicology), Erika Lisabeth (ADDRC), Richard Neubig (Drug Discovery)
**Status**: Complete — 6 experiments validated (Exp001–006), 84 Python + 84 Rust binary + 46 lib unit tests
**Parent**: gen3/baseCamp Paper 12 (Immunological Anderson), gen3/baseCamp Paper 13 (healthSpring)

---

## Overview

This sub-thesis extends the Gonzales veterinary pharmacology work — validated across wetSpring (359/359 checks) and neuralSpring (329/329 checks) — to human therapeutic applications. The mathematics of dose-response, pharmacokinetics, and cytokine signaling is species-agnostic. The models validated against canine atopic dermatitis data apply directly to human atopic dermatitis, differing only in species-specific parameters (clearance rates, receptor densities, tissue volumes).

---

## Validated Foundation (from wetSpring + neuralSpring)

| Experiment | Paper | What was validated | Checks |
|-----------|-------|-------------------|--------|
| nS-601 | G2 (Gonzales) | Hill dose-response, oclacitinib IC50 as Anderson barrier | 48 Python + 240 Rust |
| nS-602 | G3 (Gonzales) | Pruritus time-series, exponential recovery | included |
| nS-603 | G4 (Gonzales) | Lokivetmab PK decay, dose-duration regression | included |
| nS-604 | G6 (Gonzales) | Three-compartment tissue lattice (immune/skin/neural) | included |
| nS-605 | Fajgenbaum | MATRIX drug repurposing with Anderson geometry | included |
| Exp273–279 | Anderson | Immunological Anderson framework (cytokine localization) | 157/157 |
| Exp280–286 | G1–G6 | Full Gonzales paper reproductions | 202/202 |

**Total validated**: 688/688 checks across two springs.

---

## Human Extensions — Completed

### Extension 1: Human JAK Inhibitor PK/PD (from nS-601) — COMPLETE

Oclacitinib (canine) → baricitinib, upadacitinib, abrocitinib (human JAK inhibitors for AD). Same Hill equation, different IC50 values and selectivity profiles.

**Completed (Exp001)**:
- Hill dose-response curves for 4 human JAK inhibitors — **19/19 Python, 18/18 Rust**
- IC50 identity, monotonicity, potency ordering, EC10/50/90, cooperativity, saturation
- Published IC50 data: baricitinib 5.9 nM, upadacitinib 8 nM, abrocitinib 29 nM, oclacitinib 10 nM
- Rust module: `pkpd::hill_dose_response`, `pkpd::hill_sweep`, `pkpd::compute_ec_values`

**Planned (Tier 2):** GPU-vectorized Hill sweep for selectivity modeling

### Extension 2: One-Compartment PK (Rowland & Tozer) — COMPLETE

Textbook PK validation: IV bolus decay, oral Bateman equation, AUC, Cmax/Tmax, multiple dosing.

**Completed (Exp002)**:
- IV + oral + multi-dose models — **12/12 Python, 18/18 Rust**
- AUC trapezoidal vs analytical (< 0.4% error), Tmax analytical formula match
- Rust module: `pkpd::pk_iv_bolus`, `pkpd::pk_oral_one_compartment`, `pkpd::auc_trapezoidal`

### Extension 3: Two-Compartment PK (Rowland & Tozer Ch. 19) — COMPLETE

Biexponential model with distribution (α) and elimination (β) phases.

**Completed (Exp003)**:
- Micro-to-macro conversion, α/β identity checks, peripheral compartment — **15/15 Python, 11/11 Rust**
- Terminal phase log-linearity (slope error < 0.002%), reduction to one-compartment when k12=0
- Rust module: `pkpd::micro_to_macro`, `pkpd::pk_two_compartment_iv`, `pkpd::two_compartment_ab`

### Extension 4: mAb PK Cross-Species Transfer (from nS-603) — COMPLETE

Lokivetmab → nemolizumab/dupilumab via allometric scaling.

**Completed (Exp004)**:
- Allometric scaling (CL b=0.75, Vd b=1.0, t½ b=0.25) — **12/12 Python, 7/7 Rust**
- Scaled half-life 20.6 days (within published 14–28 day range for nemolizumab)
- Duration prediction transfer from canine dose-duration regression
- Rust module: `pkpd::allometric_scale`, `pkpd::mab_pk_sc`

## Human Extensions — Completed (continued)

### Extension 5: Population PK Monte Carlo — COMPLETE

1,000 virtual patients with lognormal inter-individual variability on CL, Vd, k_a. Baricitinib-like oral dosing (4mg, F=0.79).

**Completed (Exp005)**:
- 15/15 Python checks, 12/12 Rust checks
- CL-AUC negative correlation (r = -0.92)
- Population AUC mean within 0.07% of theoretical F*D/CL
- Tmax 95% CI: [1.06, 3.41] hr (reasonable for oral)
- Rust module: `pkpd::population_pk_cpu`, `pkpd::LognormalParam`, `pkpd::pop_baricitinib`
- **GPU target**: Scale to 100K–1M patients via BarraCUDA (embarrassingly parallel, each patient = independent ODE)

### Extension 6: PBPK Compartments (Gabrielsson & Weiner) — COMPLETE

5-tissue physiologically-based PK model (liver, kidney, muscle, fat, rest) connected by blood flow with tissue-plasma partition coefficients.

**Completed (Exp006)**:
- 13/13 Rust checks
- Standard 70 kg adult model, cardiac output ~330 L/hr
- Mass conservation validated, hepatic clearance dominates elimination
- Fat compartment accumulates most (Kp=5.0), liver clears fastest
- Euler integration with dt=0.01 hr, verified deterministic
- Rust module: `pkpd::pbpk` (TissueCompartment, PbpkState, pbpk_iv_simulate, cardiac_output)
- **GPU target**: Parallel across patients (each patient = independent PBPK ODE system)

## Human Extensions — Planned

### Extension 7: Population PK GPU Scale-Up

- Scale Exp005 to 100K → 1M virtual patients
- GPU dispatch via BarraCUDA (each patient = independent ODE, embarrassingly parallel)
- Dosing optimization: trough above EC90
- Hardware target: Northgate RTX 5090 (32GB VRAM, ~100MB for 100K patients)

### Extension 8: Human Monoclonal Antibody PK (from nS-603)

- Refit nS-603 PK model with human nemolizumab parameters
- Validate against published nemolizumab Phase III data (Kabashima 2020, Silverberg 2021)

### Extension 9: Human Tissue Lattice (from nS-604)

The three-compartment tissue lattice (immune/skin/neural) validated for canine AD extends to human AD. The compartment structure is identical; tissue parameters differ.

**healthSpring work**:
- Human skin thickness, dermal/epidermal partitioning
- Human IL-4/IL-13/IL-31 receptor densities
- Barrier disruption dynamics (TEWL correlation)
- Dimensional promotion: 2D surface → 3D tissue penetration

### Extension 10: Drug Repurposing via ADDRC (from nS-605)

The Fajgenbaum MATRIX with Anderson geometry scoring identifies candidate molecules. Lisabeth's ADDRC provides the HTS infrastructure to screen them. Gonzales provides iPSC-derived skin models for validation.

**healthSpring work**:
- Anderson-augmented MATRIX scoring for human AD targets
- HTS assay design for ADDRC screening
- iPSC validation protocol for top candidates
- Ellsworth medicinal chemistry optimization pathway

---

## Open Questions

1. Does the 3-point lokivetmab dose-duration model (R² = 0.971) generalize to human mAb dosing, or does the limited data produce overfitting artifacts?
2. Can population PK on GPU provide dosing recommendations for narrow-therapeutic-index drugs (e.g., vancomycin, aminoglycosides) in real clinical time?
3. Does Anderson localization in the three-compartment tissue lattice predict which AD patients will respond to which therapy class (anti-IL-31 vs anti-IL-4Rα vs JAK inhibitor)?
4. Can the MATRIX → ADDRC → iPSC pipeline identify novel AD therapeutics not currently in development?

---

## Relationship to MSU Drug Discovery Program

```
healthSpring computational models
    ↓ Anderson-augmented MATRIX scoring
Lisabeth ADDRC (high-throughput screening)
    ↓ Hit validation
Gonzales iPSC skin models (functional validation)
    ↓ Lead optimization
Ellsworth medicinal chemistry
    ↓ Candidate
Preclinical / clinical development
```

healthSpring provides the computational front end of this pipeline. The springs validated the math. healthSpring makes it operational.
