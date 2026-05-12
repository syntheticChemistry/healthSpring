// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

//! LTEE B5 validation binary — Symbiont PK/PD (Leonard et al. 2024).
//!
//! Reproduces four coupled models from the Python Tier 0 baseline:
//!   1. Logistic colonization dynamics (S. alvi in bee gut)
//!   2. Biomass-proportional molecule production
//!   3. One-compartment gut-lumen pharmacokinetics
//!   4. Hill dose-response efficacy (target knockdown)
//!
//! Reference: `control/ltee_symbiont_pkpd/expected_values.json`
//! Benchmark: `control/ltee_symbiont_pkpd/benchmark_ltee_symbiont.json`
//!
//! This is the Tier 1 (Rust CPU) parity binary for lithoSpore ingestion.

use healthspring_barracuda::validation::ValidationHarness;

// ── Model parameters (from expected_values.json) ─────────────────────

const N0: f64 = 1e5;
const K: f64 = 1e9;
const R: f64 = 1.2;

const PROD_RATE: f64 = 0.001;
const F_BIO: f64 = 0.85;
const KE: f64 = 2.0;

const EC50: f64 = 100.0;
const HILL_N: f64 = 1.5;
const E_MAX: f64 = 0.95;

const DT: f64 = 0.1;
const TOTAL_DAYS: f64 = 14.0;

// ── Benchmark values (from Python baseline) ──────────────────────────

const BENCH_CFU_DAY7: f64 = 307_839_277.318_023_74;
const BENCH_CFU_FINAL: f64 = 999_494_652.935_254_5;
const BENCH_DOUBLING_HOURS: f64 = 13.862_943_611_198_906;
const BENCH_T_HALF_MAX_DAYS: f64 = 7.675_200_305_813_208;
const BENCH_SS_MOLECULE_NG: f64 = 424.508_778_380_487_6;
const BENCH_KNOCKDOWN_SS: f64 = 0.852_528_249_503_891_3;
const BENCH_PK_HALF_LIFE_H: f64 = 8.317_766_166_719_343;

// ── Core models ──────────────────────────────────────────────────────

fn logistic_growth(t: f64, n0: f64, k: f64, r: f64) -> f64 {
    let ratio = (k - n0) / n0;
    k / ratio.mul_add((-r * t).exp(), 1.0)
}

fn doubling_time_hours(r: f64) -> f64 {
    (2.0_f64.ln() / r) * 24.0
}

fn time_to_half_capacity(n0: f64, k: f64, r: f64) -> f64 {
    ((k - n0) / n0).ln() / r
}

fn hill_knockdown(conc: f64, ec50: f64, hill_n: f64, e_max: f64) -> f64 {
    let c_n = conc.powf(hill_n);
    let ec_n = ec50.powf(hill_n);
    e_max * c_n / (c_n + ec_n)
}

fn pk_half_life_hours(ke: f64) -> f64 {
    (2.0_f64.ln() / ke) * 24.0
}

// ── Simulation ───────────────────────────────────────────────────────

struct SimResult {
    time: Vec<f64>,
    cfu: Vec<f64>,
    molecule_ng: Vec<f64>,
    knockdown: Vec<f64>,
}

fn simulate(dt: f64, total_days: f64) -> SimResult {
    #[expect(clippy::cast_possible_truncation, reason = "sim step count fits usize")]
    #[expect(clippy::cast_sign_loss, reason = "total_days/dt is positive")]
    let n_steps = (total_days / dt) as usize;

    let mut time = Vec::with_capacity(n_steps + 1);
    let mut cfu = Vec::with_capacity(n_steps + 1);
    let mut molecule_pg = vec![0.0_f64; n_steps + 1];

    for i in 0..=n_steps {
        #[expect(clippy::cast_precision_loss, reason = "small sim grids only")]
        let t = dt * i as f64;
        time.push(t);
        cfu.push(logistic_growth(t, N0, K, R));
    }

    for i in 0..n_steps {
        let input_rate = cfu[i] * PROD_RATE;
        molecule_pg[i + 1] =
            dt.mul_add(F_BIO.mul_add(input_rate, -(KE * molecule_pg[i])), molecule_pg[i]);
    }

    let conc_ng: Vec<f64> = molecule_pg.iter().map(|&m| m / 1000.0).collect();

    let knockdown: Vec<f64> = conc_ng
        .iter()
        .map(|&c| hill_knockdown(c, EC50, HILL_N, E_MAX))
        .collect();

    SimResult {
        time,
        cfu,
        molecule_ng: conc_ng,
        knockdown,
    }
}

// ── Checks ───────────────────────────────────────────────────────────

fn idx_nearest(time: &[f64], target: f64) -> usize {
    time.iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            ((**a) - target)
                .abs()
                .partial_cmp(&((**b) - target).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map_or(0, |(i, _)| i)
}

fn validate_colonization(h: &mut ValidationHarness, sim: &SimResult) {
    let idx7 = idx_nearest(&sim.time, 7.0);
    h.check_rel(
        "B5 colonization at day 7 (vs Python)",
        sim.cfu[idx7],
        BENCH_CFU_DAY7,
        1e-4,
    );

    let cfu_final = sim.cfu[sim.cfu.len() - 1];
    h.check_rel(
        "B5 colonization approaches carrying capacity",
        cfu_final,
        BENCH_CFU_FINAL,
        1e-4,
    );

    h.check_rel(
        "B5 doubling time (hours)",
        doubling_time_hours(R),
        BENCH_DOUBLING_HOURS,
        1e-6,
    );

    h.check_rel(
        "B5 time to half-max colonization (days)",
        time_to_half_capacity(N0, K, R),
        BENCH_T_HALF_MAX_DAYS,
        1e-6,
    );
}

fn validate_molecule(h: &mut ValidationHarness, sim: &SimResult) {
    let mol_final = sim.molecule_ng[sim.molecule_ng.len() - 1];
    h.check_rel(
        "B5 steady-state molecule (ng)",
        mol_final,
        BENCH_SS_MOLECULE_NG,
        1e-3,
    );

    let idx_d1 = idx_nearest(&sim.time, 1.0);
    let monotonic = sim.molecule_ng[idx_d1..]
        .windows(2)
        .all(|w| w[1] >= w[0] - 1e-10);
    h.check_bool("B5 molecule monotonically increasing post-day-1", monotonic);
}

fn validate_efficacy(h: &mut ValidationHarness, sim: &SimResult) {
    let kd_final = sim.knockdown[sim.knockdown.len() - 1];
    h.check_rel(
        "B5 knockdown at steady state",
        kd_final,
        BENCH_KNOCKDOWN_SS,
        1e-3,
    );
}

fn validate_pk(h: &mut ValidationHarness) {
    h.check_rel(
        "B5 PK half-life (hours)",
        pk_half_life_hours(KE),
        BENCH_PK_HALF_LIFE_H,
        1e-6,
    );
}

// ── JSON output ──────────────────────────────────────────────────────

fn print_json(outcome: &healthspring_barracuda::validation::ValidationOutcome) {
    println!(
        "{{\"name\":\"{}\",\"passed\":{},\"failed\":{},\"total\":{}}}",
        outcome.name, outcome.passed, outcome.failed, outcome.total,
    );
}

// ── Main ─────────────────────────────────────────────────────────────

fn main() {
    let json_mode = std::env::args().any(|a| a == "--format" || a == "json")
        && std::env::args().any(|a| a == "json" || a == "--format");

    let mut h = if json_mode {
        ValidationHarness::silent("validate_ltee_b5")
    } else {
        ValidationHarness::new("validate_ltee_b5")
    };

    let sim = simulate(DT, TOTAL_DAYS);

    validate_colonization(&mut h, &sim);
    validate_molecule(&mut h, &sim);
    validate_efficacy(&mut h, &sim);
    validate_pk(&mut h);

    let outcome = h.finish();
    if json_mode {
        print_json(&outcome);
    }
    std::process::exit(outcome.exit_code());
}
