// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "ltee-b5-symbiont-pkpd",
            track: Track::PkPd,
            tier: Tier::Rust,
            source_experiment: "ltee_b5",
            description: "LTEE B5 symbiont PK/PD — 8 cross-tier checks (colonization, molecule, efficacy, PK) vs Python baseline.",
        },
        run,
    }
}

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

const BENCH_CFU_DAY7: f64 = 307_839_277.318_023_74;
const BENCH_CFU_FINAL: f64 = 999_494_652.935_254_5;
const BENCH_DOUBLING_HOURS: f64 = 13.862_943_611_198_906;
const BENCH_T_HALF_MAX_DAYS: f64 = 7.675_200_305_813_208;
const BENCH_SS_MOLECULE_NG: f64 = 424.508_778_380_487_6;
const BENCH_KNOCKDOWN_SS: f64 = 0.852_528_249_503_891_3;
const BENCH_PK_HALF_LIFE_H: f64 = 8.317_766_166_719_343;

fn logistic_growth(t: f64, n0: f64, k: f64, r: f64) -> f64 {
    let ratio = (k - n0) / n0;
    k / ratio.mul_add((-r * t).exp(), 1.0)
}

fn hill_knockdown(conc: f64, ec50: f64, hill_n: f64, e_max: f64) -> f64 {
    let c_n = conc.powf(hill_n);
    let ec_n = ec50.powf(hill_n);
    e_max * c_n / (c_n + ec_n)
}

struct SimResult {
    time: Vec<f64>,
    cfu: Vec<f64>,
    molecule_ng: Vec<f64>,
    knockdown: Vec<f64>,
}

fn simulate() -> SimResult {
    #[expect(clippy::cast_possible_truncation, reason = "sim step count fits usize")]
    #[expect(clippy::cast_sign_loss, reason = "total_days/dt is positive")]
    let n_steps = (TOTAL_DAYS / DT) as usize;

    let mut time = Vec::with_capacity(n_steps + 1);
    let mut cfu = Vec::with_capacity(n_steps + 1);
    let mut molecule_pg = vec![0.0_f64; n_steps + 1];

    for i in 0..=n_steps {
        #[expect(clippy::cast_precision_loss, reason = "small sim grids only")]
        let t = DT * i as f64;
        time.push(t);
        cfu.push(logistic_growth(t, N0, K, R));
    }

    for i in 0..n_steps {
        let input_rate = cfu[i] * PROD_RATE;
        molecule_pg[i + 1] =
            DT.mul_add(F_BIO.mul_add(input_rate, -(KE * molecule_pg[i])), molecule_pg[i]);
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

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    let sim = simulate();

    v.section("Phase 1: Colonization Dynamics");

    let idx7 = idx_nearest(&sim.time, 7.0);
    v.check_abs_or_rel(
        "b5_colonization_day7",
        sim.cfu[idx7],
        BENCH_CFU_DAY7,
        1.0,
        1e-4,
    );

    let cfu_final = sim.cfu[sim.cfu.len() - 1];
    v.check_abs_or_rel(
        "b5_colonization_final",
        cfu_final,
        BENCH_CFU_FINAL,
        1.0,
        1e-4,
    );

    let doubling = (2.0_f64.ln() / R) * 24.0;
    v.check_abs_or_rel(
        "b5_doubling_time_hours",
        doubling,
        BENCH_DOUBLING_HOURS,
        1e-6,
        1e-6,
    );

    let t_half = ((K - N0) / N0).ln() / R;
    v.check_abs_or_rel(
        "b5_time_to_half_capacity",
        t_half,
        BENCH_T_HALF_MAX_DAYS,
        1e-6,
        1e-6,
    );

    v.section("Phase 2: Molecule Production & Efficacy");

    let mol_final = sim.molecule_ng[sim.molecule_ng.len() - 1];
    v.check_abs_or_rel(
        "b5_steady_state_molecule_ng",
        mol_final,
        BENCH_SS_MOLECULE_NG,
        1.0,
        1e-3,
    );

    let idx_d1 = idx_nearest(&sim.time, 1.0);
    let monotonic = sim.molecule_ng[idx_d1..]
        .windows(2)
        .all(|w| w[1] >= w[0] - 1e-10);
    v.check_bool(
        "b5_molecule_monotonic_post_day1",
        monotonic,
        "molecule_ng monotonically increasing after day 1",
    );

    let kd_final = sim.knockdown[sim.knockdown.len() - 1];
    v.check_abs_or_rel(
        "b5_knockdown_steady_state",
        kd_final,
        BENCH_KNOCKDOWN_SS,
        1e-3,
        1e-3,
    );

    let pk_half = (2.0_f64.ln() / KE) * 24.0;
    v.check_abs_or_rel(
        "b5_pk_half_life_hours",
        pk_half,
        BENCH_PK_HALF_LIFE_H,
        1e-6,
        1e-6,
    );
}
