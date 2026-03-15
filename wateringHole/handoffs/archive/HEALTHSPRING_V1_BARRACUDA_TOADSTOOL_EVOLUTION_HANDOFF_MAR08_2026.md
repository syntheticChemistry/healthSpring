# healthSpring V1 → barraCuda + toadStool Evolution Handoff

**Date**: March 8, 2026
**From**: healthSpring (human health applications)
**To**: barraCuda (GPU math), toadStool (heterogeneous dispatch)
**License**: AGPL-3.0-or-later
**healthSpring Version**: V1 (Tier 0+1 complete)
**barraCuda Version**: v0.3.3
**toadStool Version**: S130+

---

## Executive Summary

- healthSpring is the sixth ecoPrimals spring, focused on **human health applications** (PK/PD, microbiome, biosignal)
- **5 experiments complete** at Tier 0 (Python) + Tier 1 (Rust CPU): 72 Python checks, 46 Rust tests, 17 cross-validation matches
- **Zero barraCuda GPU primitives consumed yet** — the entire Tier 1 is pure CPU Rust
- This handoff documents **what healthSpring needs from barraCuda for Tier 2** and what it can **contribute back for absorption**
- Key opportunity: healthSpring's PK/PD and microbiome models are **low-complexity, high-parallelism** — ideal GPU candidates

---

## Part 1: What healthSpring Built (Tier 1 CPU)

### 1.1 PK/PD Module (`barracuda/src/pkpd.rs`)

14 public functions, 34 unit tests:

| Function | Purpose | GPU Candidate? |
|----------|---------|:--------------:|
| `hill_dose_response` | Generalized Hill equation (E_max · C^n / (C^n + IC50^n)) | Yes — vectorize over concentrations |
| `hill_sweep` | Sweep Hill across concentration array | Yes — trivially parallel |
| `compute_ec_values` | EC10/EC50/EC90 from Hill parameters | Low priority |
| `pk_iv_bolus` | One-compartment IV: C0 · exp(-k_e · t) | Yes — vectorize over time |
| `pk_oral_one_compartment` | Bateman equation for oral absorption | Yes — vectorize over time |
| `oral_tmax` | Analytical Tmax = ln(k_a/k_e) / (k_a - k_e) | No (scalar) |
| `auc_trapezoidal` | Trapezoidal AUC from discrete data | Yes — parallel prefix sum |
| `find_cmax_tmax` | Peak from PK curve | Yes — parallel reduction |
| `pk_multiple_dose` | Superposition for repeated dosing | Yes — map-reduce |
| `micro_to_macro` | Two-compartment micro → macro conversion | No (scalar) |
| `pk_two_compartment_iv` | Biexponential: A·exp(-αt) + B·exp(-βt) | Yes — vectorize over time |
| `two_compartment_ab` | Macro-coefficients A, B | No (scalar) |
| `allometric_scale` | Cross-species parameter scaling | No (scalar) |
| `mab_pk_sc` | mAb subcutaneous PK (Bateman) | Yes — vectorize over time |

### 1.2 Microbiome Module (`barracuda/src/microbiome.rs`)

6 public functions, 12 unit tests:

| Function | Purpose | GPU Candidate? |
|----------|---------|:--------------:|
| `shannon_index` | Shannon H' = -Σ p·ln(p) | Yes — parallel log-multiply-reduce |
| `simpson_index` | Simpson D = 1 - Σ p² | Yes — parallel square-reduce |
| `inverse_simpson` | 1 / Σ p² | Yes |
| `pielou_evenness` | J = H' / ln(S) | Depends on `shannon_index` |
| `chao1` | Richness estimator from count data | Low priority (small N) |
| `evenness_to_disorder` | Pielou → Anderson W mapping | No (scalar) |

### 1.3 Current barraCuda Usage

**None.** The `barracuda-core` dependency is declared but no imports are used. All Tier 1 implementations are self-contained Pure Rust with no external compute calls. This is intentional — Tier 1 establishes ground truth before GPU introduces floating-point variance.

---

## Part 2: What healthSpring Needs (Tier 2 GPU)

### 2.1 Priority 1: Population PK Monte Carlo (Exp005)

The first GPU experiment. Simulate 1,000 virtual patients with inter-individual variability:

```
For each patient i in 1..1000:
    sample PK parameters from lognormal(μ, σ²)  — CL, Vd, k_a
    compute concentration-time curve over 0..48hr (1000 timepoints)
    compute AUC, Cmax, Tmax
```

**barraCuda primitives needed:**
- `exp(x)` — exponential decay kernel (exists in barraCuda core)
- `log(x)` — lognormal sampling (exists)
- Random number generation — does barraCuda have a GPU RNG? If not, we need `rng_normal_gpu`
- `sum` / `mean` / `variance` — population statistics (exists)

**New shader needed:** `population_pk_sample.wgsl`
- Input: parameter distributions (μ_CL, σ_CL, μ_Vd, σ_Vd, μ_ka, σ_ka), dose, time array
- Output: per-patient concentration-time curves, AUC, Cmax
- Workgroup: 1 patient per workgroup, 1000 workgroups

**toadStool action:** This is a map-reduce pattern. Can toadStool's existing dispatch handle 1000-wide workgroup launch + per-workgroup reduction?

### 2.2 Priority 2: Compartmental ODE Solvers

wetSpring already has ODE shaders (Euler, RK4) in its Write phase. **Check absorption status:**
- If absorbed into barraCuda → healthSpring consumes directly
- If still local to wetSpring → healthSpring should not duplicate; wait for absorption or co-evolve

**barraCuda action:** Confirm ODE solver absorption status. If absorbed, document the API so healthSpring can wire in.

### 2.3 Priority 3: Diversity Metrics on GPU

For large-scale microbiome studies (thousands of samples), parallel diversity computation:
- `shannon_diversity_gpu`: parallel log-multiply-sum over abundance vectors
- `simpson_diversity_gpu`: parallel square-sum

These reduce to existing barraCuda primitives (`log`, element-wise multiply, `sum`). May not need custom shaders — could be a **fused op chain** using existing kernels.

**barraCuda action:** Is there a fused-op mechanism for `log → multiply → sum`? If so, healthSpring can skip the Write phase entirely.

### 2.4 Priority 4: Biosignal Shaders

For Track 3 (not yet started at Tier 0):
- `bandpass_iir_gpu` — IIR filter for ECG signal conditioning
- `qrs_detect_gpu` — parallel R-peak detection across multi-channel ECG

These are similar to existing signal processing in airSpring. Check for cross-spring reuse.

---

## Part 3: Absorption Candidates (healthSpring → barraCuda)

Once healthSpring writes and validates local GPU shaders, the following generalize:

| Local Shader | Generalizes To | Absorption Target |
|-------------|---------------|-------------------|
| `population_pk_sample.wgsl` | Any Monte Carlo parameter sweep | `barraCuda::monte_carlo` module |
| `hill_equation_gpu.wgsl` | Sigmoidal response curves | `barraCuda::pharmacology` or `bio` module |
| `shannon_diversity_gpu.wgsl` | Information entropy | `barraCuda::stats::entropy` |
| `bandpass_iir_gpu.wgsl` | General IIR filtering | `barraCuda::signal::filter` |

These follow the standard Write → Validate → Hand off → Absorb → Lean cycle.

---

## Part 4: Observations for barraCuda/toadStool Evolution

### 4.1 What We Learned

1. **Allometric scaling is pure math** — no GPU needed. The `(BW_human/BW_animal)^b` pattern is universal across pharmacology. Could be a utility in barraCuda core.

2. **PK models are embarrassingly parallel across patients** — each patient's concentration-time curve is independent. This is the ideal GPU workload: 1000 independent Bateman equations with different parameters.

3. **Microbiome diversity is a reduction pattern** — Shannon, Simpson, and Chao1 all reduce to element-wise-op → sum. If barraCuda has a generic `map_reduce` shader, we can avoid writing domain-specific ones.

4. **Two-compartment PK has a clean analytical solution** — no ODE solver needed for IV bolus (biexponential). ODE solvers are only needed for complex PBPK or population models with inter-compartment feedback.

5. **Cross-validation between Python (NumPy) and Rust (f64) showed exact parity** at 17/17 checkpoints. The f64-canonical approach works. GPU tier will introduce fp64 variance from shader arithmetic — need to document tolerances.

### 4.2 Opportunities for toadStool

1. **NPU for biosignal** — Akida AKD1000 is ideal for real-time ECG/PPG inference. healthSpring Track 3 will need toadStool's NPU dispatch path for Exp024 (NPU real-time inference).

2. **metalForge for clinical deployment** — a clinical PK calculator that runs on CPU (desktop), GPU (server), or NPU (wearable) depending on available hardware. This is the metalForge value proposition: same validated model, cross-substrate dispatch.

3. **NUCLEUS integration** — healthSpring experiments could run as biomeOS jobs in a NUCLEUS deployment, enabling distributed population PK across a primal cluster.

---

## Part 5: Action Items

### For barraCuda team:

| # | Action | Priority |
|---|--------|----------|
| 1 | Confirm ODE solver (Euler, RK4) absorption status from wetSpring | High |
| 2 | Confirm GPU RNG availability (normal distribution sampling) | High |
| 3 | Assess fused-op chain capability for `log → mul → sum` pattern | Medium |
| 4 | Consider `allometric_scale` utility in core math | Low |

### For toadStool team:

| # | Action | Priority |
|---|--------|----------|
| 1 | Validate 1000-workgroup dispatch for population PK Monte Carlo | High |
| 2 | NPU dispatch path readiness for biosignal inference (Exp024) | Medium |
| 3 | metalForge cross-substrate routing for clinical PK models | Medium |

### For healthSpring (next):

| # | Action | Priority |
|---|--------|----------|
| 1 | Write `population_pk_sample.wgsl` (Exp005 — first GPU experiment) | High |
| 2 | Write Anderson localization gut lattice (Exp011 — CPU first, then GPU) | High |
| 3 | Start Track 3 biosignal controls (Exp020 Pan-Tompkins) | Medium |
| 4 | Update Cargo.toml to consume barraCuda ODE solvers when absorbed | When available |

---

## Appendix: healthSpring Rust API Summary

```rust
// pkpd module — 14 public functions
pub fn hill_dose_response(concentration, ic50, hill_n, e_max) -> f64;
pub fn hill_sweep(ic50, hill_n, e_max, concentrations) -> Vec<f64>;
pub fn compute_ec_values(ic50, hill_n) -> EcValues;
pub fn pk_iv_bolus(dose_mg, vd_l, half_life_hr, t_hr) -> f64;
pub fn pk_oral_one_compartment(dose_mg, f, vd_l, k_a, k_e, t_hr) -> f64;
pub fn oral_tmax(k_a, k_e) -> f64;
pub fn auc_trapezoidal(times, concentrations) -> f64;
pub fn find_cmax_tmax(times, concentrations) -> (f64, f64);
pub fn pk_multiple_dose(single_dose, interval, n_doses, times) -> Vec<f64>;
pub fn micro_to_macro(k10, k12, k21) -> (f64, f64);
pub fn pk_two_compartment_iv(dose, v1, k10, k12, k21, t) -> f64;
pub fn two_compartment_ab(c0, alpha, beta, k21) -> (f64, f64);
pub fn allometric_scale(param, bw_animal, bw_human, exponent) -> f64;
pub fn mab_pk_sc(dose, vd, half_life, t) -> f64;

// microbiome module — 6 public functions
pub fn shannon_index(abundances) -> f64;
pub fn simpson_index(abundances) -> f64;
pub fn inverse_simpson(abundances) -> f64;
pub fn pielou_evenness(abundances) -> f64;
pub fn chao1(counts) -> f64;
pub fn evenness_to_disorder(evenness, w_scale) -> f64;
```
