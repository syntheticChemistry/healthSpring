# Low-Affinity Binding Landscape: Computation as Experiment Preprocessor

## Thesis

Standard drug discovery optimizes for **strong, specific binding** — high-affinity
hits from HTS plate screens. This is correct when you want a scalpel. But nature
uses the full affinity spectrum. The immune system, gut colonization, and tissue
homeostasis all rely on **weak, distributed, combinatorial binding** that the
current HTS → hit → lead pipeline systematically discards.

**What if the data we throw away is the data we need?**

This study explores the low-affinity regime (IC50 > 10 µM, Kd > 1 µM) that
ADDRC HTS screens classify as "inactive" and asks: what applications exist for
binders that are deliberately weak, deliberately broad, and deliberately tuned
to interact with populations of targets rather than single ones?

## Computation as Preprocessor

The key methodological shift: **computation comes before the experiment, not after.**

Traditional pipeline:
```
wet lab screen → computational analysis → hit list → validation
```

Proposed pipeline:
```
computational binding landscape → predict affinity distributions →
identify low-affinity regimes of interest → design targeted assays →
wet lab validation of computational predictions
```

This inverts the relationship: the computation isn't analyzing hits, it's
**designing the question**. The plate screener becomes a validator of
computational predictions rather than a discovery engine.

### Why This Matters for ADDRC

Lisabeth's ADDRC 8,000-compound library generates a full dose-response curve per
compound. Current analysis discards everything below the hit threshold (SSMD > 3,
Z' > 0.5). But that dose-response data contains the entire affinity landscape —
including the low-affinity tail that we systematically ignore.

Computational preprocessing can:
1. Model the full affinity distribution (not just the high end)
2. Identify compounds with interesting low-affinity profiles
3. Predict combinatorial effects of multiple weak binders
4. Design validation assays tuned for the low-affinity regime

## Three Application Domains

### 1. Combinatorial Cancer Targeting (healthSpring)

**Concept**: A treatment that has *mild affinity* for your genetic signature
and *full targeting* only of specific aberrant systems.

Like a combination lock: each individual binding event is weak and reversible,
but the simultaneous coincidence of 3-5 weak interactions on a cancer cell
creates a composite signal strong enough for targeted delivery or immune
flagging.

**Biological precedent**: Natural killer (NK) cell activation requires
integration of multiple activating receptor signals, each individually
insufficient. The "missing self" model (Kärre 1986) shows that absence
of inhibitory signals + presence of multiple weak activating signals
= kill decision.

**Computational model**:
- Hill dose-response at each binding site with IC50 >> EC50 (deliberately weak)
- Composite response = product of individual fractional occupancies
- Selectivity emerges from coincidence, not from individual affinity
- Anderson disorder model: healthy cells have "ordered" surface marker
  landscape (localized binding, no composite signal); cancer cells have
  "disordered" landscape (delocalized binding, composite signal activates)

**Experiment design** (Exp097):
- Generate synthetic compound panels with tuned IC50 distributions
- Model composite binding on normal vs cancer surface marker profiles
- Compute selectivity index at the *population* level (not single target)
- Validate against Gonzales iPSC model for therapeutic window

### 2. Broad-Spectrum Infection Response

**Concept**: Low-affinity antibodies that provide partial neutralization across
pathogen variants. Not a scalpel — a net.

**Biological precedent**: Natural IgM antibodies are polyreactive, low-affinity,
and provide innate defense before the adaptive immune system engages.
Broadly neutralizing antibodies (bnAbs) for HIV target conserved epitopes
with moderate affinity.

**Computational model**:
- Cross-reactivity score: fraction of pathogen variants where fractional
  occupancy exceeds a minimal threshold (10-20%)
- Coverage breadth vs depth tradeoff curve
- Relates to Lisabeth's Brucella screen: instead of finding the strongest
  hit, find compounds with consistent weak activity across related targets

### 3. Probiotic Adhesion & Gut Colonization (healthSpring × wetSpring)

**Concept**: Probiotic bacteria need *mild* adhesion to intestinal epithelium —
strong enough to colonize, weak enough to not invade. The binding landscape
of bacterial adhesins is inherently low-affinity, multivalent, and
population-level.

**Biological precedent**: Lactobacillus surface layer proteins bind mucin
with Kd ~ 1-10 µM. Competitive exclusion of C. diff depends on many bacteria
each weakly binding, creating a colonization-resistant "biofilm" through
cumulative occupancy.

**Anderson localization connection** (wetSpring):
- Gut epithelial surface = 1D lattice with disorder (mucin glycosylation variation)
- Bacterial adhesin binding = hopping amplitude in the Anderson tight-binding model
- **Strong binding** → localization (single species dominates a niche)
- **Weak binding** → delocalization (multiple species coexist, diversity maintained)
- Dysbiosis = loss of delocalized state → pathogen localization in cleared niche

This maps directly to the existing Anderson lattice code in healthSpring
(`microbiome::anderson_gut_lattice`) and wetSpring's diversity modules.

**Joint experiment** (Exp098):
- healthSpring: model adhesin binding landscape (Hill multi-target, low IC50)
- wetSpring: model colonization dynamics using Anderson disorder parameter
- Joint: compute the "colonization resistance surface" — the manifold where
  diversity, adhesion strength, and epithelial disorder interact
- Validate against C. diff colonization resistance data (exp012 baselines)

## Faculty Alignment

| Faculty | Role | Domain |
|---------|------|--------|
| **Gonzales** | iPSC validation of low-affinity panels; canine AD (lokivetmab = mAb, known Kd) | Cancer, skin |
| **Lisabeth** | ADDRC HTS data — full dose-response curves including low-affinity tail | Screening |
| **Neubig** | Rho/MRTF pathway — multi-target scoring already exists | Fibrosis |
| **Mok** | TRT outcomes — hormone receptor binding is inherently moderate-affinity | Endocrine |

## Computational Infrastructure

### Existing (ready to use)
- `discovery::hts` — Z', SSMD, hit classification (extend threshold ranges)
- `discovery::compound` — IC50 estimation, selectivity index, Hill fit
- `discovery::matrix_score` — MATRIX combined scoring (extend for composite)
- `pkpd::dose_response` — Hill equation at arbitrary IC50
- `microbiome::anderson_gut_lattice` — disorder-localization model
- `microbiome::cdiff_colonization` — colonization resistance
- GPU shaders: `hill_dose_response_f64.wgsl`, `diversity_f64.wgsl`

### New (to build)
- `discovery::affinity_landscape` — full binding landscape modeling
  - Composite binding score (product of weak fractional occupancies)
  - Affinity distribution analysis (not just hits)
  - Cross-reactivity matrix
  - Low-affinity selectivity index
- `discovery::combinatorial_targeting` — multi-weak-binder coincidence model
  - NK cell integration analogy
  - Surface marker profile comparison (normal vs cancer)
- Joint wetSpring experiment: colonization resistance surface computation

## Experiment Numbering

| ID | Title | Domain |
|----|-------|--------|
| exp097 | Affinity landscape & composite targeting | healthSpring discovery |
| exp098 | Colonization resistance surface (joint) | healthSpring × wetSpring |

## Key Insight

The ADDRC plate screener becomes **most powerful** not as a primary discovery
tool, but as a **computational prediction validator**. We use the full
dose-response dataset (including the "noise floor") as ground truth for our
binding landscape models. The computation doesn't analyze the hits — it
predicts where the interesting non-hits are, and the plate reader confirms.

This is **computation as experimental design** — the sovereign compute stack
isn't just validating baselines, it's generating hypotheses that couldn't
emerge from traditional analysis.
