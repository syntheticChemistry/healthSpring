<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Experimental Hypotheses — Ready for Wet Lab Validation and Grant Writing

**Date:** March 22, 2026
**Faculty:** Gonzales (Pharm/Tox), Lisabeth (ADDRC), Neubig (Drug Discovery), Ellsworth (Med Chem)
**Status:** Computational evidence complete. Every hypothesis below has Rust code,
Python baselines, and validation experiments backing it. Ready for experimental
design and grant proposals.

---

## The Position

You walk in with 863 tests, 83 experiments, 782 Gonzales-specific validation
checks across three springs, and a computational pipeline that replaces $210K of
commercial software with sovereign Rust that runs on a laptop. The Gonzales
interview established credibility. The Lisabeth conversation opened the door to
ADDRC's 8,000-compound library and the specificity/binding question that became
Track 9.

What follows are testable predictions — each with the computational evidence,
the experimental design, the PI alignment, and the grant mechanism.

---

## Hypothesis 1: Low-Affinity Coincidence Detection for Cancer Selectivity

**Prediction:** Compounds with IC50 > 10 µM across multiple cancer markers can
achieve higher *composite selectivity* over normal tissue than high-affinity
single-target drugs, through coincidence detection (many weak signals summing
only where targets co-localize).

**Computational evidence:**
- exp097: 26× selectivity ratio from IC50=20µM binder across 15 cancer markers
  vs 4 normal markers (composite_binding_score: 0.519 vs 0.020)
- Gini coefficient distinguishes broad binders (Gini≈0) from narrow (Gini>0.5)
- Cross-reactivity matrix visualizes the full binding landscape

**Experimental test:**
1. Pull ADDRC dose-response data for compounds currently classified "inactive"
   (SSMD < 1, IC50 > 10 µM)
2. Compute composite binding scores across a cancer marker panel vs normal panel
3. Select top 20 by computational selectivity ratio
4. Test in Gonzales iPSC skin model: measure cytotoxicity on normal vs cancer-like
   iPSC lines (e.g., AD-activated keratinocytes vs healthy)
5. Compare therapeutic window to conventional high-affinity hits from same screen

**PI alignment:** Lisabeth (ADDRC data + screen design), Gonzales (iPSC validation)
**Grant:** NCI R21 (Exploratory/Developmental) — "Computational Discovery of
Low-Affinity Multi-Target Cancer Selectivity from Existing HTS Libraries"
**Budget justification:** Computational triage is free; wet lab validation uses
existing ADDRC infrastructure and Gonzales iPSC models. R21 scale ($275K/2yr).
**What's novel:** Inverts the HTS paradigm — the "inactive" compounds ARE the
candidates. No new screening needed; reanalyze existing data.

---

## Hypothesis 2: Delocalized Toxicity Advantage

**Prediction:** At equivalent total systemic exposure, compounds with distributed
weak binding (low toxicity IPR) produce less organ-specific damage than
concentrated strong binders (high toxicity IPR), because per-tissue burden
stays below repair capacity and clearance remains in the linear kinetics regime.

**Computational evidence:**
- exp098: Anderson IPR 0.003 (distributed) vs 0.695 (concentrated)
- Distributed binding: zero excess burden above tissue repair capacity
- Linear clearance regime (C/Km < 0.003) for weak binders vs saturated (C/Km > 1)
  for strong binders

**Experimental test:**
1. Select matched compound pairs from ADDRC: same total binding (AUC of dose-response
   curve) but different binding *distribution* (one concentrated, one spread)
2. Compute toxicity IPR for each compound's tissue profile
3. In vitro: measure cytotoxicity in 4+ cell lines (hepatocyte, cardiomyocyte,
   renal tubular, neural) representing different tissues
4. Predict: distributed binder has lower max-tissue toxicity than concentrated
   binder at equivalent total exposure
5. In vivo (if warranted): preclinical PK with tissue biomarkers

**PI alignment:** Gonzales (iPSC multi-tissue, PK), Lisabeth (compound selection)
**Grant:** NIGMS R01 — "Anderson Localization Framework for Predicting
Multi-Organ Toxicity Distribution" or NIAMS R21 (if scoped to skin)
**Budget justification:** Computational toxicity prediction → iPSC validation →
preclinical. Standard drug safety pharmacology costs.
**What's novel:** First application of condensed matter physics (Anderson IPR)
to predict organ-specific toxicity from binding distribution data.

---

## Hypothesis 3: Hormetic Dose-Response in ADDRC Compounds

**Prediction:** A subset of ADDRC compounds show biphasic dose-response: low
concentrations stimulate cellular activity (stress pathway upregulation) while
high concentrations inhibit. The hormetic zone is predictable from the compound's
Hill parameters and the target cell's stress pathway profile.

**Computational evidence:**
- exp099: biphasic curve emerges from competition between saturating repair
  pathways (HSP, SOD, p53, mTOR) and unbounded damage accumulation
- hormetic_optimum finds peak dose computationally
- exp111: mechanistic cell fitness traces causality through 4 pathways

**Experimental test:**
1. Run ADDRC compounds at 10-point dose-response (standard) but ADD 4 sub-IC50
   concentrations (1/10, 1/30, 1/100, 1/300 of IC50)
2. Measure cell viability AND stress pathway reporters (HSP70-luc, Nrf2-ARE-luc,
   p21-luc for p53, or equivalent qPCR panel)
3. Compute: fit biphasic model to extended dose-response → extract hormetic
   zone boundaries
4. Validate: compounds predicted to be hormetic (by computational model) should
   show measurable stress pathway upregulation at sub-IC50 doses
5. Control: compounds predicted to be non-hormetic should show monotonic
   dose-response only

**PI alignment:** Lisabeth (extended dose-response protocol), Gonzales (iPSC stress
pathway readouts), Neubig (Rho/MRTF pathway as stress indicator)
**Grant:** NIGMS R21 — "Computational Prediction of Hormetic Drug Responses
from HTS Dose-Response Data" or R01 if combined with Hyp 1-2
**Budget justification:** Marginal cost — extending existing 10-point curves to
14-point. Reporter assays or qPCR panel. R21 scale.
**What's novel:** First systematic screen for hormesis in a drug repurposing
library. Current HTS analysis cannot detect hormesis because it doesn't measure
sub-IC50 doses.

---

## Hypothesis 4: Anderson-Augmented MATRIX Reranks Drug Candidates

**Prediction:** Adding tissue geometry (Anderson localization length) to
Fajgenbaum's MATRIX drug-disease scoring produces different top-N rankings
than MATRIX alone, and the Anderson-augmented rankings better predict
clinical efficacy for diseases with known tissue architecture disruption
(atopic dermatitis, fibrosis, IBD).

**Computational evidence:**
- exp090: Anderson-augmented MATRIX scoring implemented and validated
- nS-605: Fajgenbaum MATRIX reproduced (329/329 checks with standard scoring)
- healthSpring tissue lattice: localization length as geometry dimension
- Gonzales canine AD data: barrier disruption = dimensional promotion in
  Anderson framework (gen3 Paper 12)

**Experimental test:**
1. Compute standard MATRIX scores for ADDRC 8K library against AD target panel
2. Compute Anderson-augmented MATRIX scores (add tissue localization length)
3. Identify compounds where ranking changes significantly (>50 rank positions)
4. Blind test: do reranked compounds perform differently in Gonzales iPSC AD
   model? (cornified envelope integrity, IL-31 response, barrier function)
5. Validation: correlate Anderson-augmented rank with iPSC efficacy

**PI alignment:** Lisabeth (ADDRC 8K library), Gonzales (iPSC AD model),
Neubig (fibrosis scoring dimension)
**Grant:** NCI R01 or NIAMS R01 — "Tissue Geometry as a Drug Scoring Dimension:
Anderson Localization in Drug Repurposing" (Fajgenbaum collaboration possible)
**Budget justification:** R01 scale ($1.25M/5yr) — computational infrastructure
+ ADDRC screening + iPSC validation + preclinical.
**What's novel:** MATRIX (Fajgenbaum) + Anderson (Kachkovskiy/Anderson) =
geometry-aware drug repurposing. No one else has this combination.

---

## Hypothesis 5: Colonization Resistance Through Diverse Weak Adhesion

**Prediction:** Probiotic formulations with many species of moderate adhesion
strength create more robust colonization resistance than single-strain strong
adhesion, analogous to Anderson delocalization. The resistance surface is
non-monotonic in adhesion strength.

**Computational evidence:**
- exp097: colonization_resistance function shows diversity × weak adhesion
  outperforms concentration × strong adhesion
- wetSpring V130: bio::binding_landscape confirms with 16S-calibrated models
- exp111: ecosystem simulation shows competitive dynamics under stress

**Experimental test:**
1. Design probiotic blends: (a) single strong adherer, (b) 3-species moderate,
   (c) 8-species weak, matched total CFU
2. Challenge with C. diff in anaerobic culture or Gonzales iPSC gut model
3. Measure: colonization resistance (pathogen exclusion), community stability
   (Shannon diversity over time), metabolic output (SCFA production)
4. Predict: (c) > (b) > (a) for resistance; (a) > (b) > (c) for single-target
   displacement; non-monotonic response surface

**PI alignment:** Gonzales (iPSC gut if available, or collaborate with GI faculty),
wetSpring provides community modeling
**Grant:** USDA NIFA AFRI — "Computational Design of Multi-Strain Probiotic
Formulations via Anderson Adhesion Modeling" or NIH NIDDK R21 for gut health
**Budget justification:** Anaerobic culture + community profiling (16S) + SCFA
measurement. Pilot scale.
**What's novel:** First physics-informed probiotic design. Anderson framework
predicts which diversity/adhesion combinations produce robust resistance.

---

## Hypothesis 6: Cross-Species PK Translation via Parameter Substitution

**Prediction:** healthSpring's species-agnostic PK models, validated on canine
data (Gonzales G1-G6), predict human PK parameters for the same drug classes
with ≤30% error using only allometric scaling and species-specific clearance
rates — no additional fitting required.

**Computational evidence:**
- Exp001-006: human PK validated (Hill, 1/2-compartment, population, PBPK)
- Exp100-106: canine/feline/equine PK validated (Track 6)
- Exp004: mAb cross-species transfer (lokivetmab → nemolizumab)
- Allometric bridge: CL_human = CL_dog × (BW_human/BW_dog)^0.75

**Experimental test:**
1. Take 5 ADDRC compounds with known canine PK (from Gonzales/Zoetis literature)
2. Predict human PK using healthSpring allometric bridge (no fitting to human data)
3. Compare predictions to published human PK data (if available) or to Gonzales
   iPSC-derived intrinsic clearance
4. Quantify prediction error; target ≤30% relative for CL, Vd, t½

**PI alignment:** Gonzales (canine PK expertise, 18 years Zoetis data), Mok (human
clinical interpretation)
**Grant:** NIGMS R01 supplement or FDA Critical Path — "Sovereign Cross-Species
Pharmacometric Translation Without Commercial Software"
**Budget justification:** Purely computational + literature validation. Minimal
wet lab. Could be an R21 supplement.
**What's novel:** Zero-cost, sovereign (AGPL) PK translation pipeline. Replaces
$210K/year of NONMEM + Monolix + WinNonlin.

---

## Hypothesis 7: Stress Pathway Hierarchy Predicts Drug Sensitivity

**Prediction:** The order in which cellular stress pathways activate (HSP → SOD →
p53 → mTOR, ordered by k_half sensitivity) determines a cell type's response
profile to any compound. Cell types with strong early pathways (high HSP) are
more hormetic; cell types with weak early pathways are more sensitive.

**Computational evidence:**
- exp111: standard_eukaryotic_pathways defines the hierarchy
- mechanistic_cell_fitness: different pathway profiles produce different
  biphasic curves for the same dose
- Ecosystem simulation: pathway profiles determine competitive outcomes

**Experimental test:**
1. Profile 4 cell types for stress pathway basal expression and inducibility
   (qPCR panel: HSP70, HSP90, SOD1/2, p53, mTOR, Beclin-1)
2. Expose all 4 cell types to same 5 ADDRC compounds at 14-point dose-response
3. Fit mechanistic biphasic model to each cell type × compound
4. Predict: cell types with higher basal HSP show wider hormetic zones; cell
   types with weak HSP but strong p53 show narrower zones with sharper toxicity
5. Validate: pathway profile predicts dose-response shape across compounds

**PI alignment:** Gonzales (iPSC multi-lineage), Lisabeth (ADDRC compounds),
Neubig (Rho pathway as additional stress axis)
**Grant:** NIGMS R01 — "Mechanistic Prediction of Cell-Type-Specific Drug
Sensitivity from Stress Pathway Profiles"
**Budget justification:** R01 scale — iPSC differentiation + qPCR profiling +
extended dose-response + computational modeling.
**What's novel:** First mechanistic link between measurable pathway profiles
and quantitative dose-response shape prediction. Moves beyond
phenomenological IC50 to causal understanding.

---

## Hypothesis 8: ADDRC "Inactive" Compounds as Immune Modulators

**Prediction:** Compounds in the ADDRC 8K library currently classified as
"inactive" (no strong hit against any single target) include immune modulators
that work through distributed weak binding — similar to how the hygiene
hypothesis works through broad, low-level microbial exposure calibrating
immune balance.

**Computational evidence:**
- exp099: immune_calibration function models hygiene hypothesis quantitatively
- exp097: low_affinity_selectivity shows distributed binding can be functional
- exp099: caloric_restriction_fitness models beneficial sub-threshold stress

**Experimental test:**
1. Select 50 "inactive" ADDRC compounds (SSMD < 1 across all targets)
2. Compute immune_calibration score for each (predicted immunomodulatory potential
   from binding distribution)
3. Test top 10 in immune cell assay: stimulate PBMCs or THP-1 with LPS, add
   compound at sub-IC50 dose, measure cytokine panel (IL-6, TNF-α, IL-10, IFN-γ)
4. Predict: computationally ranked compounds show dose-dependent cytokine
   modulation that pure "inactive" classification missed
5. Control: bottom 10 by computational score should show no modulation

**PI alignment:** Lisabeth (ADDRC compound selection), Gonzales (immune/cytokine
readouts from AD expertise)
**Grant:** NIAID R21 — "Mining HTS Libraries for Hidden Immunomodulators via
Computational Binding Landscape Analysis"
**Budget justification:** R21 scale — compound selection + PBMC/THP-1 assays +
Luminex cytokine panel. Uses existing ADDRC infrastructure.
**What's novel:** Systematically looks for function in what the field discards.
No new screening — reanalyze existing dose-response data for distributed effects.

---

## Grant Strategy Matrix

| Hyp | Title (short) | Mechanism | PI Lead | PI Support | Budget | Timeline |
|-----|---------------|-----------|---------|------------|--------|----------|
| H1 | Low-affinity cancer selectivity | NCI R21 | Lisabeth | Gonzales | $275K/2yr | Near |
| H2 | Delocalized toxicity | NIGMS R01 | Gonzales | Lisabeth | $1.25M/5yr | Medium |
| H3 | Hormetic dose-response | NIGMS R21 | Lisabeth | Gonzales, Neubig | $275K/2yr | Near |
| H4 | Anderson-augmented MATRIX | NIAMS R01 | Gonzales | Lisabeth, Fajgenbaum | $1.25M/5yr | Medium |
| H5 | Probiotic colonization | NIDDK R21 / USDA | Gonzales | wetSpring | $275K/2yr | Medium |
| H6 | Cross-species PK | NIGMS supp / FDA | Gonzales | Mok | $150K/2yr | Near |
| H7 | Stress pathway hierarchy | NIGMS R01 | Gonzales | Lisabeth, Neubig | $1.25M/5yr | Long |
| H8 | Hidden immunomodulators | NIAID R21 | Lisabeth | Gonzales | $275K/2yr | Near |

**"Near"** = ready to write now, minimal new data needed.
**"Medium"** = needs pilot data from an R21 or internal funding first.
**"Long"** = needs results from earlier hypotheses to justify.

---

## The Pitch in One Paragraph

We have a sovereign computational pipeline — validated against 782 of your
published data points, replacing $210K/year of commercial software — that makes
testable predictions about drug candidates *before* they touch a plate reader.
The immediate opportunity: your ADDRC library already contains the dose-response
data for 8,000 compounds. Our models predict that the compounds you're currently
discarding as "inactive" include cancer-selective multi-target binders,
immunomodulators, and hormetic agents. The experiments to test these predictions
use your existing infrastructure: iPSC models, ADDRC plate readers, and standard
cytokine panels. We're not asking you to build anything new — we're asking you
to look at your existing data through a new lens.

---

## What Exists Right Now (Bring to Any Meeting)

| Artifact | Location | What It Shows |
|----------|----------|---------------|
| 782/782 Gonzales validation checks | wetSpring + neuralSpring + healthSpring | Your published science, reproduced |
| ADDRC HTS analysis (exp091) | healthSpring experiments/ | Z', SSMD, hit classification — your workflow, validated |
| Compound library IC50 (exp092) | healthSpring experiments/ | 8K compound batch processing, working |
| Low-affinity selectivity (exp097) | healthSpring experiments/ | 26× selectivity from "inactive" compounds |
| Toxicity landscape (exp098) | healthSpring experiments/ | Anderson predicts organ-specific toxicity |
| Hormesis framework (exp099) | healthSpring experiments/ | Biphasic dose-response, sub-IC50 benefit |
| Causal terrarium (exp111) | healthSpring experiments/ | Full molecular → ecosystem causal chain |
| MATRIX + Anderson scoring (exp090) | healthSpring experiments/ | Drug repurposing with tissue geometry |
| Cost comparison | baseCamp/gonzales/cost_access_methods.md | $210K → $0.01 per drug pipeline |
| Pipeline architecture | baseCamp/fajgenbaum/README.md | MATRIX → ADDRC → iPSC → med chem |
| Grant technical appendix | attsi/non-anon/contact/publicRelease/ | NIH/NCI/NIAMS/ARPA-H alignment |
| Spring validation brief | attsi/non-anon/contact/gonzales/ | G1–G6 four-tier validation |
| Computational brief | attsi/non-anon/contact/gonzales/ | Anderson + PK/PD summary |
