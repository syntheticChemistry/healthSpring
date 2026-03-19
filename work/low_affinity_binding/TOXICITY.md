# The Body's Burden: Toxicity of Delocalized Binding

## The Question

We've shown computationally that weak, distributed binding can achieve
selective targeting through coincidence detection (exp097). But selectivity
isn't safety. If a compound weakly binds *everywhere*, what's the systemic
cost? Can the body actually handle it?

## The Answer: Three Lines of Evidence

### 1. Repair Capacity Buffer

Every tissue has a baseline repair capacity — the amount of binding insult
it can absorb without adverse effect. For weak binders, the per-tissue
occupancy stays below this threshold.

**Computational evidence** (exp098, Study 2):
- Weak distributed binder (IC50=30µM at 8 tissues): excess burden = 0.015
- Strong localized binder (IC50=0.3µM at liver): excess burden = 0.72
- Only 1 of 8 tissues stressed for the weak binder vs the liver being
  overwhelmed for the strong binder

The key insight: **strong binding exceeds repair capacity at the target tissue
even though total systemic binding may be lower.** The weak binder touches
more tissues but stays within repair capacity almost everywhere.

### 2. Linear Clearance Regime

Michaelis-Menten kinetics create a critical regime boundary:
- At low C (C << Km): clearance is first-order — fast, predictable, scalable
- At high C (C >> Km): clearance saturates — slow, unpredictable, accumulation

Weak binders at low per-tissue concentrations live in the linear regime.
Strong binders at high local concentrations can saturate hepatic clearance.

**Computational evidence** (exp098, Study 3):
- Weak binder: max clearance utilization = 0.3% (deep in linear regime)
- Strong binder: max clearance utilization = 71.4% (saturated, nonlinear PK)
- All 8 tissues for the weak binder: C/Km < 0.01

This means the body clears weak binders predictably and efficiently. There's
no risk of nonlinear accumulation. The pharmacokinetics are boring — and
boring PK is safe PK.

### 3. Anderson Delocalization

The toxicity profile of a compound across tissues maps directly to
Anderson localization:

| Property | Strong binder | Weak binder |
|----------|--------------|-------------|
| Toxicity IPR | 0.695 (localized) | 0.003 (delocalized) |
| Localization length ξ | 1.4 tissues | 351 tissues |
| Regime | Organ-specific toxicity | Distributed load |
| Analog | Localized wavefunction | Extended wavefunction |

**Delocalized toxicity** means:
- No single tissue bears a disproportionate burden
- Repair machinery across the whole body shares the load
- No "hot spots" that trigger organ-specific failure
- Even with disordered tissue sensitivities (W=0.8), the weak binder
  stays delocalized (ξ > 300)

### 4. Hormesis Bonus

At very low binding levels (IC50 > 200µM), the systemic burden falls
into the **hormetic zone** — the range where low-level stress triggers
adaptive protection mechanisms. This is the immunological equivalent
of exercise: a small challenge that makes the system stronger.

## The Delocalization Hypothesis

**Strong binder toxicity is a localization problem.**

When a drug binds strongly to one target, the toxic burden localizes —
concentrating damage at one tissue beyond its repair capacity while the
rest of the body's repair machinery sits idle. The result is
organ-specific toxicity (hepatotoxicity, cardiotoxicity, nephrotoxicity).

**Weak binder toxicity is a delocalization phenomenon.**

When many weak binders spread their load across tissues, no single site
is overwhelmed. The body's distributed repair machinery handles it.
Clearance stays in the predictable linear regime. And at very low levels,
hormesis may provide a net benefit.

**Anderson's insight applies**: in a disordered system, strong coupling
leads to localization (concentrated, trapped). Weak coupling leads to
delocalization (spread, flowing). For toxicity, delocalization is the
safer state.

## Implications for Drug Design

### Traditional Pipeline
- Screen for strong binders → worry about toxicity later
- Therapeutic index = LD50/ED50 (single target)
- Toxicology is reactive: find the damage, then mitigate

### Proposed Pipeline
- Compute binding landscape → identify compounds with broad-weak profiles
- Compute toxicity landscape → verify delocalized burden stays below
  tissue repair thresholds → verify clearance stays linear
- **Toxicology is predictive**: the computation tells you the compound
  is safe *before* you test it

### New Safety Metrics

| Metric | What it captures | Goal |
|--------|-----------------|------|
| Toxicity IPR | Burden concentration | < 0.15 (delocalized) |
| Localization length ξ | Tissues sharing burden | > N_tissues/2 |
| Max clearance utilization | Clearance regime | < 20% (linear) |
| Excess burden | Repair capacity buffer | 0.0 (fully buffered) |
| Delocalization advantage | Distributed vs localized safety ratio | > 1 (∞ is ideal) |

## Connection to Existing Infrastructure

### Uses
- `discovery::fractional_occupancy` — Hill binding at each tissue
- `pkpd::nonlinear` — Michaelis-Menten clearance saturation
- `microbiome::anderson` — IPR, localization length concepts
- `tolerances` — centralized thresholds for all safety checks

### Validates Against
- exp095 iPSC viability (Hill cytotoxicity — our IC50 model matches)
- exp096 niclosamide PBPK (tissue geometry factor from Anderson)
- Gonzales iPSC skin model readouts (viability vs drug concentration)

### Extends To
- Joint wetSpring experiment: probiotic adhesion toxicity — is broad gut
  binding safe for the epithelium? The colonization resistance model +
  toxicity landscape answers this
- FDA FAERS integration: validate delocalization predictions against
  real adverse event frequencies

## Experiment Status

| ID | Title | Checks | Status |
|----|-------|--------|--------|
| exp097 | Affinity landscape | 18/18 | pass |
| exp098 | Toxicity landscape | 22/22 | pass |
