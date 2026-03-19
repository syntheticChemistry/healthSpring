# Cross-Spring Hormesis: One Curve, Every Domain

## The Unifying Shape

Every domain in ecoPrimals encounters the same biphasic dose-response:

```
fitness
  |        ___
  |      /     \
  |     /       \
  |----/         \
  |  /             \
  | /               \___
  |__________________________ dose
  0     optimal    IC50
      (hormetic)
```

Low stress improves fitness. High stress destroys it. The transition is an
Anderson localization event. This isn't a metaphor — it's the same math.

## Domain Map

| Spring | Stressor | Low-dose effect | High-dose effect |
|--------|----------|----------------|------------------|
| **healthSpring** | Drug/toxin | Adaptive stress response, longevity | Organ toxicity |
| **groundSpring** | Pesticide/herbicide | Plant growth stimulation, pest fitness | Crop/organism death |
| **airSpring** | Air pollution, UV | Antioxidant upregulation | Respiratory/skin damage |
| **wetSpring** | Antimicrobial | Community restructuring, resilience | Dysbiosis, resistance |

All four share the same `biphasic_dose_response(dose, baseline, s_max, k_stim, ic50, hill_n)`.

## The Examples

### 1. Weak Pesticide → More Grasshoppers (groundSpring × airSpring)

**Computational evidence** (exp099, Study 2):
- At dose=2.8 (the hormetic optimum): grasshopper population increases 31.4%
- A weak pesticide makes the pest problem *worse*

Why this happens:
1. **Direct hormesis**: mild stress upregulates reproduction, heat shock
   proteins, antioxidant defenses. The grasshoppers that survive are fitter.
2. **Indirect ecology**: the pesticide suppresses competitors, predators,
   or parasites more than the target pest. The pest is released.
3. **Both mechanisms are computable** before spraying — groundSpring models
   the direct response, airSpring models the environmental dispersal and
   non-target exposure, wetSpring models the soil microbiome disruption.

### 2. Caloric Restriction → Longevity (healthSpring)

**Computational evidence** (exp099, Study 5):
- Optimal CR: 19% restriction → 91.6 years (14.5% gain over 80-year baseline)
- 50% restriction: 72.2 years (9.8% loss)
- 90% restriction: 32.2 years (59.8% loss)

The mechanism: autophagy (cellular self-eating under nutrient stress)
clears damaged organelles. Sirtuins and AMPK activate repair pathways.
Mitochondrial efficiency improves. All of these are *stress responses*
that enhance fitness — as long as the caloric deficit stays below the
threshold where muscle wasting, immune suppression, and organ failure begin.

### 3. Mithridatism — Can You Make Yourself Immune to Poison? (healthSpring)

**Computational evidence** (exp099, Study 3):
- Yes: 50 low-dose exposures shifts IC50 from 10.0 to 48.5 (5x tolerance)
- At a lethal dose (D=20): naive organism fitness = 25.7, adapted = 101.1
- **But it costs you**: resting fitness drops from 100.0 to 92.0 (8% loss)

The adaptation saturates (Hill-shaped): early exposures build tolerance
quickly, later ones have diminishing returns. The metabolic cost also
saturates: maintaining detoxification enzymes, efflux pumps, and
metallothioneins requires energy.

Mithridates VI of Pontus survived assassination attempts — but he was
never as healthy at rest as an unexposed person. When he finally wanted
to die (by poison), he couldn't. He had to ask a soldier to stab him.

### 4. Hygiene Hypothesis — Peanut Allergies (healthSpring × wetSpring)

**Computational evidence** (exp099, Study 4):
- Sterile environment: immune competence = 0.30 (uncalibrated)
- Moderate microbial exposure: competence = 0.86 (well-calibrated)
- Overwhelming exposure: competence = 0.03 (overwhelmed)

The LEAP study (Du Toit et al. 2015 NEJM): early peanut exposure reduced
peanut allergy by 81% in high-risk infants. The immune system needs
*training data* — microbial exposure provides it.

This connects directly to wetSpring's microbiome diversity models:
- High diversity = high microbial exposure = calibrated immune system
- Low diversity = sterile or dysbiotic = uncalibrated → allergies
- The Anderson disorder parameter of the gut microbiome literally
  predicts immune calibration

### 5. Herbicide Hormesis in Plants (groundSpring)

Low-dose glyphosate has been shown to stimulate growth in some plant
species (Cedergreen 2008 *Pest Management Science*). The mechanism:
sublethal inhibition of the shikimate pathway triggers compensatory
upregulation of aromatic amino acid synthesis. The plant overcompensates,
producing more biomass than untreated controls.

This is the same biphasic math. groundSpring can model it before
application — predicting which species will benefit and which will suffer.

### 6. Radiation Hormesis (airSpring × healthSpring)

Low-dose radiation (<100 mSv) may stimulate DNA repair mechanisms,
leading to a net reduction in cancer risk compared to zero exposure.
This is controversial but fits the biphasic model: at low doses,
the repair response exceeds the damage rate.

## The Anderson Connection

**Why does mild stress help?** Because the stress response is
**delocalized** across many repair pathways.

At low doses:
- Heat shock proteins activate broadly
- Antioxidant enzymes upregulate across tissues
- Autophagy clears damaged organelles everywhere
- DNA repair enzymes scan the whole genome
- The response IPR is low → delocalized → net improvement

At high doses:
- Damage concentrates at vulnerable sites (Anderson localization)
- Repair machinery is overwhelmed at those sites
- Other pathways sit idle — the response is wasted
- The damage IPR is high → localized → net harm

**The transition from hormetic to toxic is a delocalization → localization
transition.** This is not a metaphor. It's the same math, the same
IPR calculation, the same Anderson Hamiltonian.

## The Non-Reducibility Argument

You can't understand pesticide hormesis without ecology (groundSpring),
atmospheric dispersal (airSpring), soil microbiome response (wetSpring),
and human health impact (healthSpring). You can't understand peanut
allergies without immunology, microbiome diversity, and early childhood
nutrition. You can't understand caloric restriction without autophagy,
mitochondrial biology, and evolutionary stress response.

**All science is interconnected and non-reducible.**

The ecoPrimals architecture reflects this: springs don't import each other,
they coordinate through wateringHole handoffs and shared barraCuda
primitives. The `biphasic_dose_response` function in healthSpring is the
same math that groundSpring will use for herbicide hormesis, airSpring for
radiation response, and wetSpring for antimicrobial community effects.

The sovereign compute stack isn't just replicating existing analysis —
it's connecting domains that have been studied in isolation. The hormesis
curve is one curve. Every spring lives on it.

## Implementation Status

| Component | Location | Tests |
|-----------|----------|-------|
| `biphasic_dose_response` | `toxicology.rs` | 4 tests |
| `hormetic_optimum` | `toxicology.rs` | 1 test |
| `mithridatism_adaptation` / `_fitness` | `toxicology.rs` | 4 tests |
| `immune_calibration` | `toxicology.rs` | 1 test |
| `caloric_restriction_fitness` | `toxicology.rs` | 1 test |
| `ecological_hormesis` | `toxicology.rs` | 1 test |
| `hormesis_localization` | `toxicology.rs` | 2 tests |
| exp099 validation binary | `experiments/exp099_hormesis/` | 27/27 pass |

## Future Work

| Spring | Experiment | Description |
|--------|-----------|-------------|
| groundSpring | exp_gs_001 | Herbicide hormesis in crop species |
| airSpring | exp_as_001 | Particulate matter hormesis in respiratory tissue |
| wetSpring | exp_ws_joint | Antimicrobial hormesis in gut community |
| Joint | exp_eco_001 | Full ecosystem hormesis: pesticide → soil → water → air → human |

The joint experiment would sweep: pesticide concentration × application
method × soil type × weather → predict crop yield, pest population,
soil microbiome diversity, water contamination, air dispersal, and
human exposure. All on one biphasic curve per organism per pathway.
