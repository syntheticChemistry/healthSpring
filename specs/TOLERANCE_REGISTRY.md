<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# healthSpring Tolerance Registry

## Philosophy

- **Machine epsilon class** (1e-10 to 1e-15): Used for analytical identities where f64 arithmetic should be exact to machine precision. Justified by IEEE 754 double precision (52-bit mantissa, ε ≈ 2.2e-16).
- **Numerical method class** (1e-6 to 0.01): Used for trapezoidal AUC, numerical Tmax, and iterative methods where discretization error is inherent. Tolerance scales with step size.
- **Population/statistical class** (0.05 to 0.15): Used for Monte Carlo and population models where finite sample size introduces sampling error. Tolerance reflects √(1/N) convergence.
- **Clinical plausibility class** (0.25 to 0.60): Used for qualitative checks (front-loaded > 55%, CV > 15%) based on published literature ranges.

## Registry

### Machine Epsilon Class (1e-10 to 1e-15)

| Experiment | Check | Tolerance | Class | Justification |
|-----------|-------|-----------|-------|---------------|
| exp001 | Hill at IC50 → 0.5 | 1e-10 | Machine epsilon | Analytical identity: R = C^n/(IC50^n + C^n) at C=IC50 gives exactly 0.5. f64 arithmetic. |
| exp001 | Monotonicity | 1e-15 | Machine epsilon | Hill equation is strictly monotonic; allow rounding in w[0] ≤ w[1] comparison. |
| exp002 | IV C(0) = Dose/Vd | 1e-10 | Machine epsilon | Exact analytical formula C(0) = Dose/Vd. |
| exp002 | Oral C(0) = 0 | 1e-10 | Machine epsilon | Oral absorption has zero concentration at t=0 by definition. |
| exp002 | Monotonic decreasing | 1e-15 | Machine epsilon | IV bolus C(t) = C0·exp(-k·t) is strictly decreasing; allow rounding. |
| exp002 | Non-negative (multi-dose) | 1e-12 | Machine epsilon | Superposition of non-negative curves; small negative from cancellation. |
| exp003 | α+β = k10+k12+k21, α·β = k10·k21 | 1e-8 | Machine epsilon | Macro/micro rate identity from quadratic roots. Slightly looser than 1e-10 for accumulated ops. |
| exp003 | C(0) = Dose/V1, A+B = C0, V2, Vss, CL | 1e-8 | Machine epsilon | Analytical compartment identities. |
| exp003 | k12=0 → one-compartment | 1e-10 | Machine epsilon | Both curves use same analytical formula when k12=0; max diff from f64. |
| exp003 | Non-negative, monotonic | 1e-12 | Machine epsilon | Allow rounding in c ≥ 0 and w[0] ≥ w[1] checks. |
| exp003 | Linear regression denom | 1e-15 | Machine epsilon | Guard against singular matrix in slope computation. |
| exp004 | b=0 → identity, C(0)=0 | 1e-10 | Machine epsilon | Allometric scale(100,15,70,0) = 100 exactly; mAb SC C(0)=0. |
| exp004 | Non-negative curves | 1e-10 | Machine epsilon | Same as TOL; allow rounding. |
| exp004 | CL ratio = BW^0.75 | 1e-6 | Numerical | Allometric exponent is exact; ratio involves powf and division. |
| exp005 | C(0) = 0 (oral) | 1e-12 | Machine epsilon | Oral model C(0)=0 by definition. |
| exp010 | Shannon, Simpson, Pielou, Chao1, baseline | 1e-8 | Machine epsilon | Diversity indices are closed-form; cross-validation to Python. |
| exp010 | Abundance normalization | 1e-6 | Numerical | Communities sum to 1.0; float accumulation over ~15 species. |
| exp011 | H symmetric, diagonal, hopping | 1e-14 | Machine epsilon | Anderson Hamiltonian H[i,j]=H[j,i], H[i,i]=disorder[i]; exact construction. |
| exp011 | IPR(delta)=1, ξ(0.25)=4 | 1e-14 | Machine epsilon | IPR of |ψ|²=δ gives 1; ξ = 1/IPR gives 4 for IPR=0.25. |
| exp011 | IPR(uniform) = 1/L | 1e-10 | Machine epsilon | Uniform state has IPR = Σp² = L·(1/L)² = 1/L. |
| exp011 | Level spacing r ≈ 1 | 0.02 | Numerical | Uniform eigenvalues; r has finite-sample variance. |
| exp011 | Pielou → W mapping | 0.01 | Numerical | W = scale·J; cross-validate to Python baseline 8.63. |
| exp012 | Pielou in [0,1] | 1e-10 | Machine epsilon | Boundary check for floating point. |
| exp012 | Hamiltonian symmetric | 1e-14 | Machine epsilon | H = H^T by construction. |
| exp012 | W match Python | 0.01 | Numerical | Cross-validation to exp012 Python baseline. |
| exp020 | MWI ≥ 0 | 1e-12 | Machine epsilon | MWI is sum of squared; allow rounding. |
| exp030 | C(0) = 0 | 1e-10 | Machine epsilon | IM depot C(0)=0 by definition. |
| exp030 | Non-negative | 1e-12 | Machine epsilon | Allow rounding in c ≥ 0. |
| exp031 | C(0) = 0 | 1e-10 | Machine epsilon | Pellet C(0)=0 by definition. |
| exp031 | Non-negative | 1e-12 | Machine epsilon | Allow rounding. |
| exp032 | T(30) = T0 | 1e-10 | Machine epsilon | testosterone_decline(age=30, onset=30) = T0 exactly. |
| exp032 | T0 below threshold → onset | 1e-10 | Machine epsilon | age_at_threshold returns onset when T0 ≤ threshold. |
| exp033 | ΔW(0) = 0 | 1e-10 | Machine epsilon | weight_trajectory(0) = 0 by definition. |
| exp033 | ΔW(60), waist, BMI | 1e-8 | Machine epsilon | Target values; 1e-8 for multi-step trajectory. |
| exp033 | Monotonic | 1e-12 | Machine epsilon | Allow rounding in trajectory comparison. |
| exp034 | Baselines at t=0 | 1e-10 | Machine epsilon | biomarker_trajectory(0) = baseline exactly. |
| exp034 | HR(normalized) = 0.44 | 1e-10 | Machine epsilon | hazard_ratio_model(T≥threshold) = hr_normalized. |
| exp034 | Monotonic trajectories | 1e-12 | Machine epsilon | Allow rounding. |
| exp035 | HbA1c(0) = baseline | 1e-10 | Machine epsilon | hba1c_trajectory(0) = baseline exactly. |
| exp035 | HbA1c monotonic | 1e-12 | Machine epsilon | Allow rounding. |
| exp037 | W = scale·J | 1e-10 | Machine epsilon | evenness_to_disorder is W = scale·J for linear mapping. |
| exp037 | ξ(W=0) = 1 | 1e-10 | Machine epsilon | Zero disorder → minimal localization length. |
| exp037 | Pielou in [0,1] | 0.001 | Numerical | Boundary slack for J ≤ 1.001. |
| barracuda/pkpd/compartment | IV C(0), oral C(0), identities | 1e-10 | Machine epsilon | Same as experiments; unit tests. |
| barracuda/pkpd/compartment | IV at half-life | 1e-6 | Numerical | C(t½) = C0/2; exp() has small error. |
| barracuda/pkpd/compartment | Non-negative, monotonic | 1e-12 | Machine epsilon | Same as experiments. |
| barracuda/pkpd/compartment | k12=0 reduction | 1e-10 | Machine epsilon | Analytical equivalence. |
| barracuda/pkpd/dose_response | Hill at IC50 | 1e-10 | Machine epsilon | Same as exp001. |
| barracuda/pkpd/allometry | b=0 identity | 1e-10 | Machine epsilon | Same as exp004. |
| barracuda/pkpd/allometry | CL ratio | 1e-6 | Numerical | Same as exp004. |
| barracuda/endocrine | testosterone_decline, biomarker, HR | 1e-10, 1e-8 | Machine epsilon | Same as experiments. |
| barracuda/endocrine | biomarker at 10τ | 0.5 | Numerical | Trajectory approaches endpoint; 0.5 unit absolute at t=10τ. |
| barracuda/endocrine | lognormal mean | 0.01 | Numerical | Mean of lognormal from μ,σ; 1% relative. |
| barracuda/biosignal | Perfect detection sensitivity | 1e-10 | Machine epsilon | No noise → Se=1 exactly. |

### Numerical Method Class (1e-6 to 0.01)

| Experiment | Check | Tolerance | Class | Justification |
|-----------|-------|-----------|-------|---------------|
| exp002 | IV at half-life C0/2 | 1e-6 | Numerical | C(t½)=C0·exp(-ln2) = C0/2; exp() and division accumulate error. |
| exp002 | AUC analytical | 0.01 | Numerical | Trapezoidal rule with N=1000 steps over 48 hr; O(1/N²) error ≈ 1e-6 for smooth curve, 1% conservative. |
| exp002 | Tmax numerical vs analytical | 0.1 | Numerical | find_cmax_tmax uses discrete grid; 0.1 hr = 6 min at 1000 pts over 48 hr. |
| exp002 | Oral decay by 48 hr | 0.01 | Numerical | C(48hr) should be negligible; 1% of peak is clinical "gone". |
| exp003 | AUC analytical | 0.01 | Numerical | Trapezoidal with N=2000 over 168 hr; same rationale as exp002. |
| exp003 | Terminal slope ≈ -β | 0.01 | Numerical | Linear regression on log(C) vs t; slope has O(1/√n) error. |
| exp004 | Tmax human ≥ canine | 0.5 days | Numerical | find_cmax_tmax grid resolution; 0.5 day acceptable for cross-species. |
| exp005 | AUC ≈ F*D/CL | 0.10 | Numerical | AUC truncated at 24 hr vs infinite; ~10% truncation error. |
| exp010 | Abundance sum = 1 | 1e-6 | Numerical | Float accumulation over species. |
| exp011 | Uniform spacing r ≈ 1 | 0.02 | Numerical | Level spacing ratio for uniform spectrum; finite-sample variance. |
| exp011, exp012 | W match Python | 0.01 | Numerical | Cross-validation to Python baseline; 1% relative. |
| exp031 | Dose-weight scaling | 0.01 | Numerical | Ratio C(200lb)/C(150lb) vs dose ratio; trapezoidal AUC involved. |
| exp031 | AUC ≈ D/(Vd·ke) | 0.10 | Numerical | Trapezoidal over 180 days, N=3000; pellet has ramp-up/washout. |
| exp032 | 1%/yr residual at 90 | 0.01 | Numerical | exp(-0.01·60) ≈ 0.549; 1% relative on exponential. |
| exp032 | 3%/yr residual at 90 | 0.01 | Numerical | exp(-0.03·60) ≈ 0.165; same. |
| barracuda/pkpd/compartment | AUC rel err | 0.01 | Numerical | Same as exp002/exp003. |
| barracuda/pkpd/compartment | Tmax | 0.1 | Numerical | Same as exp002. |
| barracuda/pkpd/compartment | C(48hr) < 0.01 | 0.01 | Numerical | Same as exp002. |

### Population/Statistical Class (0.05 to 0.15)

| Experiment | Check | Tolerance | Class | Justification |
|-----------|-------|-----------|-------|---------------|
| exp005 | Lognormal μ recovers typical | 0.05 | Population | exp(μ) = median; N=1 no sampling, but lognormal param conversion. |
| exp005 | Vd median | 0.05 | Population | Deterministic cohort; 5% for parameter recovery. |
| exp005 | Ka median | 0.10 | Population | Ka has higher CV (0.4); 10% for typical recovery. |
| exp036 | Vd median near typical | 0.05 | Population | N=100 deterministic percentiles; median ≈ typical. |
| exp036 | AUC CV > 0.15 | 0.15 | Population | Population AUC must show variability; CV 15% is lower bound. |
| exp031 | Plateau CV < 5% | 0.05 | Population | Pellet plateau should be stable; 5% CV is tight. |
| exp031 | Washout t½ | 0.15 | Numerical | Numerical half-life from discrete search; 15% rel err. |
| exp031 | Pellet fluctuation < 10% | 0.10 | Population | Plateau (max-min)/min; 10% for sustained-release. |
| exp030 | Accumulation factor R | 0.25 | Population | R_obs from numerical cmax_ss vs analytical; 25% for multi-dose. |
| exp030 | AUC analytical equality | 0.01 | Numerical | Weekly vs biweekly AUC per 14 days; exact by design. |

### Clinical Plausibility Class (0.25 to 0.60)

| Experiment | Check | Tolerance | Class | Justification |
|-----------|-------|-----------|-------|---------------|
| exp033 | Front-loaded weight loss | > 0.60 | Clinical | >60% of 5-yr loss by 24 mo; TRT weight loss literature. |
| exp034 | Front-loaded LDL | > 0.55 | Clinical | >55% of LDL improvement by 12 mo; lipid trajectory studies. |
| exp035 | Front-loaded HbA1c | > 0.80 | Clinical | >80% of HbA1c drop by 6 mo; diabetes TRT trials. |
| exp035 | HbA1c clinically significant | > 0.30 | Clinical | ΔHbA1c > 0.3% is clinically meaningful. |
| exp035 | HOMA improvement | 0.15–0.50 | Clinical | 15–50% HOMA-IR reduction; literature range. |
| exp020 | Sensitivity > 0.8 | 0.80 | Clinical | Pan-Tompkins QRS detection; 80% Se is acceptable. |
| exp020 | PPV > 0.8 | 0.80 | Clinical | 80% PPV for peak detection. |
| exp020 | HR within 10 bpm | 10.0 | Clinical | Synthetic ECG at 72 bpm; ±10 bpm for detection. |
| exp020 | SDNN < 200 ms | 200.0 | Clinical | Synthetic RR variability; 200 ms upper bound. |
| exp020 | Peak match tolerance | 75 ms | Clinical | 75 ms = 27 samples at 360 Hz; QRS width ~80–120 ms. |
| exp031 | Steady-state ratio | > 0.95 | Clinical | C(5t½)/C_ss > 95%; 5 half-lives to steady state. |
| exp031 | Washout > 50% | 0.50 | Clinical | C(6 mo) < 50% of C(end); pellet washout. |

### GPU Parity Class (1e-4 to 0.25)

| Experiment | Check | Tolerance | Class | Justification |
|-----------|-------|-----------|-------|---------------|
| exp053 | Hill GPU vs CPU | 1e-4 rel | GPU f32 transcendental | WGSL uses f32 exp/log for fractional Hill n; ~7 decimal digits. f64 GPU is driver-dependent. |
| exp053 | PopPK GPU mean AUC vs CPU | 0.25 | GPU statistical | GPU uses xorshift32+Wang hash PRNG vs CPU LCG; different streams → statistical parity only. |
| exp053 | PopPK GPU std AUC vs CPU | 0.25 | GPU statistical | Same PRNG difference; ±25% on std with different random streams. |
| exp053 | PopPK AUC range [0.1, 10] | range | GPU validity | AUC must be positive and bounded; independent of PRNG. |
| exp054 | Fused Hill vs individual | 1e-4 | GPU f32 transcendental | Same shader, single vs fused encoder; f32 precision limit. |
| exp054 | Fused PopPK vs individual | 0.25 | GPU statistical | Statistical parity for fused vs individual dispatch. |
| exp055 | GPU scaling linearity | 0.01 | GPU numerical | Throughput must scale with problem size; 1% for timing noise. |
| exp060 | CPU vs GPU pipeline parity | 1e-4 | GPU f32 transcendental | Same as exp053 Hill parity. |
| exp062 | Zero transfer bytes | 1e-15 | Machine epsilon | Same-substrate transfers should have zero overhead. |
| exp006 | Mass conservation | 0.25 | Numerical | PBPK mass balance: sum of tissue masses vs dose; Euler discretization. |

### CPU Parity Class (1e-10)

| Experiment | Check | Tolerance | Class | Justification |
|-----------|-------|-----------|-------|---------------|
| exp067 | Hill GPU-less CPU parity | 1e-10 | Machine epsilon | CPU-only validation of all three GPU op types (Hill, PopPK, Diversity) without GPU hardware; pure f64 arithmetic. |
| exp067 | PopPK CPU deterministic | 1e-10 | Machine epsilon | LCG-based population PK with deterministic seeds; f64 exact. |
| exp067 | Diversity CPU parity | 1e-10 | Machine epsilon | Shannon/Simpson CPU fallback vs library; identical codepath. |
| exp069 | ExpDecay pipeline stage | 1e-10 | Machine epsilon | `y = exp(-0.01 * x)` pipeline transform; f64 transcendental at small argument. |
| exp069 | Dispatch plan substrate consistency | 1e-10 | Machine epsilon | metalForge substrate selection must be deterministic for given workload. |

### NLME / Pipeline Class (V14)

| Experiment | Check | Tolerance | Class | Justification |
|-----------|-------|-----------|-------|---------------|
| exp075 | FOCE theta recovery (CL, Vd, Ka) | 0.30 rel | Population | FOCE on 30 subjects with IIV; 30% relative for parameter recovery is standard in NLME literature. |
| exp075 | SAEM theta recovery | 0.50 rel | Population | SAEM with stochastic E-step; wider tolerance than FOCE due to Monte Carlo noise in 200 iterations. |
| exp075 | NCA lambda_z | 0.05 rel | Numerical | Terminal slope from log-linear regression; 5% for discrete sampling. |
| exp075 | NCA AUC_inf | 0.05 rel | Numerical | AUC∞ = AUC_last + C_last/λz; 5% propagated from λz tolerance. |
| exp075 | CWRES mean | 2.0 abs | Population | CWRES should be ~N(0,1); mean <2.0 is a standard NLME diagnostic criterion. |
| exp075 | GOF R-squared | ≥0 | Population | Observed vs predicted R²≥0; any positive correlation indicates model captures signal. |
| exp075 | FOCE objective deterministic | 1e-10 | Machine epsilon | Same seed → same objective function value. |
| exp075 | SAEM objective deterministic | 1e-10 | Machine epsilon | Same seed → same objective function value. |
| exp076 | Node/edge counts per track | exact | Structural | Scenario builders must produce exact topology. |
| exp076 | Channel type presence | exact | Structural | All 7 DataChannel types must appear in full study. |
| exp076 | Full study totals | 28 nodes, 29 edges | Structural | Regression guard against missing/extra nodes. |

### Summary by Experiment

| Experiment | Key Tolerances | Primary Class |
|-----------|----------------|---------------|
| exp001 | 1e-10 (IC50), 1e-15 (mono), 0.99 (saturation) | Machine epsilon |
| exp002 | 1e-10, 1e-6, 1e-15, 0.01, 0.1, 1e-12 | Mixed |
| exp003 | 1e-8, 1e-12, 1e-10, 0.01 | Mixed |
| exp004 | 1e-10, 1e-6, 0.5 (Tmax) | Machine epsilon + numerical |
| exp005 | 0.05, 0.10, 0.10, 1e-12 | Population + numerical |
| exp010 | 1e-8, 1e-6 | Machine epsilon |
| exp011 | 1e-14, 1e-10, 0.02, 0.01 | Machine epsilon + numerical |
| exp012 | 1e-10, 1e-14, 0.01 | Mixed |
| exp020 | 1e-12, 0.8, 10, 200, 75 ms | Clinical + machine epsilon |
| exp030 | 1e-10, 1e-12, 0.15 (C decay), 0.01, 0.25 | Mixed |
| exp031 | 1e-10, 0.95, 0.05, 0.10, 0.15, 0.01 | Mixed |
| exp032 | 1e-10, 0.01 | Machine epsilon + numerical |
| exp033 | 1e-10, 1e-8, 1e-12, 0.60 | Machine epsilon + clinical |
| exp034 | 1e-10, 1e-12, 0.55 | Machine epsilon + clinical |
| exp035 | 1e-10, 1e-12, 0.05, 0.80, 0.30, 0.15–0.50 | Mixed |
| exp036 | 0.05, 0.15 | Population |
| exp037 | 1e-10, 0.001 | Machine epsilon |
| exp053 | 1e-4, 0.25 | GPU parity |
| exp054 | 1e-4, 0.25 | GPU parity |
| exp060 | 1e-4, 0.25 | GPU parity |
| exp062 | 1e-15 | Machine epsilon |
| exp067 | 1e-10 | Machine epsilon (CPU parity) |
| exp069 | 1e-10 | Machine epsilon (dispatch) |
| exp075 | 0.30, 0.50, 0.05, 2.0, ≥0, 1e-10 | Population + numerical + machine epsilon |
| exp076 | exact (structural) | Structural (node/edge/channel counts) |
