// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]

//! Exp103: Lokivetmab dose-duration relationship (Fleck/Gonzales 2021, CM-004)
//!
//! Validates lokivetmab PK, effective duration, and onset time.

use healthspring_barracuda::comparative::canine::{
    lokivetmab_effective_duration, lokivetmab_onset_hr, lokivetmab_pk,
};
use healthspring_barracuda::provenance::{log_analytical, AnalyticalProvenance};
use healthspring_barracuda::tolerances::{DETERMINISM, LOKIVETMAB_DECAY};
use healthspring_barracuda::validation::ValidationHarness;

const BODY_WEIGHT_KG: f64 = 15.0;
/// Therapeutic threshold for duration checks (Fleck/Gonzales: ~3 µg/mL for ~14d at 0.5 mg/kg).
const THRESHOLD_UG_ML: f64 = 3.0;

fn main() {
    let mut h = ValidationHarness::new("exp103_lokivetmab_duration");

    log_analytical(&AnalyticalProvenance {
        formula: "C(t) = (D/Vd) × exp(-k_el × t), t_half = 7×(dose/0.5) + 7",
        reference: "Fleck/Gonzales 2021, Vet Dermatol 32:681",
        doi: None,
    });

    // 1. lokivetmab_pk at t=0: C0 > 0
    let c0_at_1mg = lokivetmab_pk(1.0, BODY_WEIGHT_KG, 0.0);
    h.check_bool("lokivetmab_pk at t=0: C0 > 0", c0_at_1mg > 0.0);

    // 2. lokivetmab_pk monotonically decays
    let c0 = lokivetmab_pk(1.0, BODY_WEIGHT_KG, 0.0);
    let c7 = lokivetmab_pk(1.0, BODY_WEIGHT_KG, 7.0);
    let c28 = lokivetmab_pk(1.0, BODY_WEIGHT_KG, 28.0);
    h.check_bool(
        "lokivetmab_pk monotonically decays",
        c0 > c7 && c7 > c28,
    );

    // 3. Higher dose → higher C at any time
    let c_low = lokivetmab_pk(0.5, BODY_WEIGHT_KG, 7.0);
    let c_mid = lokivetmab_pk(1.0, BODY_WEIGHT_KG, 7.0);
    let c_high = lokivetmab_pk(2.0, BODY_WEIGHT_KG, 7.0);
    h.check_bool(
        "Higher dose → higher C at any time",
        c_low < c_mid && c_mid < c_high,
    );

    // 4. lokivetmab_effective_duration: 0.5 mg/kg → ~14 days (within 3 days)
    let dur_05 = lokivetmab_effective_duration(0.5, BODY_WEIGHT_KG, THRESHOLD_UG_ML);
    h.check_abs(
        "lokivetmab_effective_duration: 0.5 mg/kg → ~14 days",
        dur_05,
        14.0,
        3.0,
    );

    // 5. lokivetmab_effective_duration: 1.0 mg/kg → ~30-45 days (model-dependent)
    let dur_10 = lokivetmab_effective_duration(1.0, BODY_WEIGHT_KG, THRESHOLD_UG_ML);
    h.check_bool(
        "lokivetmab_effective_duration: 1.0 mg/kg → ~30-45 days",
        (30.0..=50.0).contains(&dur_10),
    );

    // 6. lokivetmab_effective_duration: 2.0 mg/kg → ~60-90 days (model-dependent)
    let dur_20 = lokivetmab_effective_duration(2.0, BODY_WEIGHT_KG, THRESHOLD_UG_ML);
    h.check_bool(
        "lokivetmab_effective_duration: 2.0 mg/kg → ~60-120 days",
        (60.0..=130.0).contains(&dur_20),
    );

    // 7. Duration increases with dose (monotonic)
    h.check_bool(
        "Duration increases with dose (monotonic)",
        dur_05 < dur_10 && dur_10 < dur_20,
    );

    // 8. lokivetmab_onset_hr: 1.0 mg/kg → ~3 hours (within 2h)
    let onset_1mg = lokivetmab_onset_hr(1.0);
    h.check_abs(
        "lokivetmab_onset_hr: 1.0 mg/kg → ~3 hours",
        onset_1mg,
        3.0,
        2.0,
    );

    // 9. lokivetmab_onset_hr: higher dose → shorter or equal onset
    let onset_05 = lokivetmab_onset_hr(0.5);
    let onset_2 = lokivetmab_onset_hr(2.0);
    h.check_bool(
        "lokivetmab_onset_hr: higher dose → shorter or equal onset",
        onset_2 <= onset_05,
    );

    // 10. lokivetmab_onset_hr: result in [1, 12] range
    let onset_any = lokivetmab_onset_hr(0.5);
    h.check_bool(
        "lokivetmab_onset_hr: result in [1, 12] range",
        (1.0..=12.0).contains(&onset_any),
    );

    // 11. C0 scales linearly with dose
    let c0_05 = lokivetmab_pk(0.5, BODY_WEIGHT_KG, 0.0);
    let c0_10 = lokivetmab_pk(1.0, BODY_WEIGHT_KG, 0.0);
    let c0_20 = lokivetmab_pk(2.0, BODY_WEIGHT_KG, 0.0);
    let ratio_10_05 = c0_10 / c0_05;
    let ratio_20_10 = c0_20 / c0_10;
    h.check_abs(
        "C0 scales linearly with dose (2× dose → 2× C0)",
        ratio_10_05,
        2.0,
        LOKIVETMAB_DECAY * 100.0,
    );
    h.check_abs(
        "C0 scales linearly with dose (2× dose → 2× C0) at high",
        ratio_20_10,
        2.0,
        LOKIVETMAB_DECAY * 100.0,
    );

    // 12. Duration 0 when dose too low (C0 below threshold)
    let threshold_high = 1000.0;
    let dur_tiny = lokivetmab_effective_duration(0.01, BODY_WEIGHT_KG, threshold_high);
    h.check_bool(
        "Duration 0 when dose too low (C0 below threshold)",
        dur_tiny <= 0.0,
    );

    // 13. All concentrations non-negative
    let concs = [
        lokivetmab_pk(0.5, BODY_WEIGHT_KG, 0.0),
        lokivetmab_pk(1.0, BODY_WEIGHT_KG, 28.0),
        lokivetmab_pk(2.0, BODY_WEIGHT_KG, 50.0),
    ];
    h.check_bool(
        "All concentrations non-negative",
        concs.iter().all(|&c| c >= 0.0),
    );

    // 14. Determinism
    let r1 = lokivetmab_pk(1.0, BODY_WEIGHT_KG, 14.0);
    let r2 = lokivetmab_pk(1.0, BODY_WEIGHT_KG, 14.0);
    h.check_abs("Determinism", r1, r2, DETERMINISM);

    h.exit();
}
