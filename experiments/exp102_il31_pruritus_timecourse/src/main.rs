// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]

//! Exp102: Canine IL-31 pruritus time-course recovery (Gonzales 2016, CM-003)
//!
//! Validates pruritus VAS time-course under oclacitinib and lokivetmab treatment.

use healthspring_barracuda::comparative::canine::{
    CanineIl31Treatment, pruritus_time_course, pruritus_vas_response,
};
use healthspring_barracuda::provenance::{AnalyticalProvenance, log_analytical};
use healthspring_barracuda::tolerances::{DETERMINISM, PRURITUS_AT_EC50, PRURITUS_TIME_COURSE};
use healthspring_barracuda::validation::ValidationHarness;

const BASELINE_PG_ML: f64 = 44.5;
const T_END_HR: f64 = 200.0;
const N_POINTS: usize = 101;

fn main() {
    let mut h = ValidationHarness::new("exp102_il31_pruritus_timecourse");

    log_analytical(&AnalyticalProvenance {
        formula: "VAS(t) = VAS_max × C(t)^n / (EC50^n + C(t)^n)",
        reference: "Gonzales 2016, Vet Dermatol 27:34",
        doi: None,
    });

    // 1. Untreated time-course: VAS remains high (near initial) at t=200hr
    let (_times_untreated, vas_untreated) = pruritus_time_course(
        BASELINE_PG_ML,
        CanineIl31Treatment::Untreated,
        T_END_HR,
        N_POINTS,
    );
    let vas_untreated_end = vas_untreated.last().copied().unwrap_or(0.0);
    let vas_untreated_start = vas_untreated.first().copied().unwrap_or(0.0);
    h.check_bool(
        "Untreated time-course: VAS remains high at t=200hr",
        vas_untreated_end > 7.0 && (vas_untreated_end - vas_untreated_start).abs() < 1.0,
    );

    // 2. Oclacitinib time-course: VAS decreases over 200hr
    let (_, vas_ocla) = pruritus_time_course(
        BASELINE_PG_ML,
        CanineIl31Treatment::Oclacitinib,
        T_END_HR,
        N_POINTS,
    );
    let vas_ocla_start = vas_ocla.first().copied().unwrap_or(0.0);
    let vas_ocla_end = vas_ocla.last().copied().unwrap_or(0.0);
    h.check_bool(
        "Oclacitinib time-course: VAS decreases over 200hr",
        vas_ocla_end < vas_ocla_start,
    );

    // 3. Lokivetmab time-course: VAS decreases over 200hr
    let (_, vas_loki) = pruritus_time_course(
        BASELINE_PG_ML,
        CanineIl31Treatment::Lokivetmab,
        T_END_HR,
        N_POINTS,
    );
    let vas_loki_start = vas_loki.first().copied().unwrap_or(0.0);
    let vas_loki_end = vas_loki.last().copied().unwrap_or(0.0);
    h.check_bool(
        "Lokivetmab time-course: VAS decreases over 200hr",
        vas_loki_end < vas_loki_start,
    );

    // 4. Lokivetmab reduces VAS faster than oclacitinib at 24hr
    let (times_24hr, vas_ocla_24) = pruritus_time_course(
        BASELINE_PG_ML,
        CanineIl31Treatment::Oclacitinib,
        24.0,
        N_POINTS,
    );
    let (_, vas_loki_24) = pruritus_time_course(
        BASELINE_PG_ML,
        CanineIl31Treatment::Lokivetmab,
        24.0,
        N_POINTS,
    );
    let idx_24 = times_24hr
        .iter()
        .position(|&t| t >= 24.0)
        .unwrap_or(N_POINTS - 1);
    let vas_ocla_at_24 = vas_ocla_24.get(idx_24).copied().unwrap_or(0.0);
    let vas_loki_at_24 = vas_loki_24.get(idx_24).copied().unwrap_or(0.0);
    h.check_bool(
        "Lokivetmab reduces VAS faster than oclacitinib at 24hr",
        vas_loki_at_24 < vas_ocla_at_24,
    );

    // 5. All VAS values in [0, 10]
    let all_vas: Vec<f64> = vas_untreated
        .iter()
        .chain(vas_ocla.iter())
        .chain(vas_loki.iter())
        .copied()
        .collect();
    h.check_bool(
        "All VAS values in [0, 10]",
        all_vas.iter().all(|&v| (0.0..=10.0).contains(&v)),
    );

    // 6. Time-course returns correct n_points
    h.check_exact(
        "Time-course returns correct n_points",
        vas_untreated.len() as u64,
        N_POINTS as u64,
    );

    // 7. VAS at t=0 matches baseline pruritus_vas_response(44.5)
    let vas_baseline = pruritus_vas_response(BASELINE_PG_ML);
    h.check_abs(
        "VAS at t=0 matches baseline pruritus_vas_response(44.5)",
        vas_untreated_start,
        vas_baseline,
        PRURITUS_AT_EC50,
    );

    // 8. Oclacitinib final VAS < untreated final VAS
    h.check_bool(
        "Oclacitinib final VAS < untreated final VAS",
        vas_ocla_end < vas_untreated_end,
    );

    // 9. Lokivetmab final VAS < oclacitinib final VAS (at same endpoint)
    h.check_bool(
        "Lokivetmab final VAS < oclacitinib final VAS",
        vas_loki_end < vas_ocla_end,
    );

    // 10. Untreated VAS is monotonically stable (approximately constant)
    let untreated_stable = vas_untreated
        .windows(2)
        .all(|w| (w[1] - w[0]).abs() < PRURITUS_TIME_COURSE * 10.0);
    h.check_bool("Untreated VAS is monotonically stable", untreated_stable);

    // 11. Treated VAS is monotonically decreasing (early phase)
    let (_, vas_ocla_early) =
        pruritus_time_course(BASELINE_PG_ML, CanineIl31Treatment::Oclacitinib, 50.0, 26);
    let early_decreasing = vas_ocla_early
        .windows(2)
        .all(|w| w[1] <= w[0] + PRURITUS_TIME_COURSE);
    h.check_bool(
        "Treated VAS is monotonically decreasing (early phase)",
        early_decreasing,
    );

    // 12. VAS at endpoint with lokivetmab < 3.0 (effective treatment)
    h.check_bool(
        "VAS at endpoint with lokivetmab < 3.0 (effective treatment)",
        vas_loki_end < 3.0,
    );

    // 13. Determinism: same inputs → identical time-course
    let (_, vas_run1) = pruritus_time_course(
        BASELINE_PG_ML,
        CanineIl31Treatment::Oclacitinib,
        T_END_HR,
        N_POINTS,
    );
    let (_, vas_run2) = pruritus_time_course(
        BASELINE_PG_ML,
        CanineIl31Treatment::Oclacitinib,
        T_END_HR,
        N_POINTS,
    );
    let vas_diff: f64 = vas_run1
        .iter()
        .zip(vas_run2.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f64::max);
    h.check_abs(
        "Determinism: same inputs → identical time-course",
        vas_diff,
        0.0,
        DETERMINISM,
    );

    h.exit();
}
