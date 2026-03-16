// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
//! Exp101: Canine oclacitinib JAK1 selectivity validation (Gonzales 2014, CM-002)
//!
//! Validates canine oclacitinib IC50 panel and compares against human JAK inhibitors.

use healthspring_barracuda::comparative::canine::{
    JakIc50Panel, canine_jak_ic50_panel, human_jak_reference_panels,
};
use healthspring_barracuda::discovery::{pathway_selectivity_score, selectivity_index};
use healthspring_barracuda::pkpd::hill_dose_response;
use healthspring_barracuda::provenance::{AnalyticalProvenance, log_analytical};
use healthspring_barracuda::tolerances::{
    HILL_SATURATION_100X, JAK1_SELECTIVITY, MACHINE_EPSILON, PRURITUS_AT_EC50,
};
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("exp101_canine_jak1");

    // Provenance: Hill identity
    log_analytical(&AnalyticalProvenance {
        formula: "R = C^n / (IC50^n + C^n) at C=IC50 → 0.5",
        reference: "Hill 1910, J Physiol",
        doi: Some("10.1113/jphysiol.1910.sp001397"),
    });

    // Provenance: Oclacitinib selectivity
    log_analytical(&AnalyticalProvenance {
        formula: "JAK1 selectivity = geomean(JAK2,JAK3,TYK2) / JAK1",
        reference: "Gonzales 2014, JVPT 37:317",
        doi: None,
    });

    let oclacitinib = canine_jak_ic50_panel();
    let human_panels = human_jak_reference_panels();

    // Check 1: Oclacitinib JAK1 IC50 = 10.0 nM
    h.check_abs(
        "Oclacitinib JAK1 IC50 = 10.0 nM",
        oclacitinib.jak1_nm,
        10.0,
        MACHINE_EPSILON,
    );

    // Check 2: Oclacitinib JAK2 IC50 = 1000.0 nM
    h.check_abs(
        "Oclacitinib JAK2 IC50 = 1000.0 nM",
        oclacitinib.jak2_nm,
        1000.0,
        MACHINE_EPSILON,
    );

    // Check 3: Oclacitinib JAK3 IC50 = 10000.0 nM
    h.check_abs(
        "Oclacitinib JAK3 IC50 = 10000.0 nM",
        oclacitinib.jak3_nm,
        10_000.0,
        MACHINE_EPSILON,
    );

    // Check 4: Oclacitinib TYK2 IC50 = 10000.0 nM
    h.check_abs(
        "Oclacitinib TYK2 IC50 = 10000.0 nM",
        oclacitinib.tyk2_nm,
        10_000.0,
        MACHINE_EPSILON,
    );

    // Check 5: JAK1 selectivity ratio > 50
    let sel_ratio = oclacitinib.jak1_selectivity();
    h.check_bool("JAK1 selectivity ratio > 50", sel_ratio > 50.0);

    // Check 6: Oclacitinib most JAK1-selective in 4-compound panel
    let mut all_panels: Vec<&JakIc50Panel> = human_panels.iter().collect();
    all_panels.push(&oclacitinib);
    let most_selective = all_panels
        .iter()
        .max_by(|a, b| {
            a.jak1_selectivity()
                .partial_cmp(&b.jak1_selectivity())
                .unwrap_or(core::cmp::Ordering::Equal)
        })
        .map_or("", |p| p.compound.as_str());
    h.check_bool(
        "Oclacitinib most JAK1-selective in 4-compound panel",
        most_selective == "oclacitinib",
    );

    // Check 7: Tofacitinib prefers JAK3 (JAK3 < JAK1)
    let tofacitinib = &human_panels[0];
    h.check_bool(
        "Tofacitinib prefers JAK3 (JAK3 < JAK1)",
        tofacitinib.jak3_nm < tofacitinib.jak1_nm,
    );

    // Check 8: Ruxolitinib JAK1 ≈ JAK2 (within 20% — dual inhibitor)
    let ruxolitinib = &human_panels[1];
    let jak1_jak2_ratio = ruxolitinib.jak1_nm / ruxolitinib.jak2_nm;
    h.check_rel(
        "Ruxolitinib JAK1 ≈ JAK2 (within 20%)",
        jak1_jak2_ratio,
        1.0,
        0.20,
    );

    // Check 9: Hill dose-response at C=IC50: R = 0.5 (analytical identity for all)
    let hill_at_ic50_all: Vec<f64> = [
        &oclacitinib,
        &human_panels[0],
        &human_panels[1],
        &human_panels[2],
    ]
    .iter()
    .map(|p| hill_dose_response(p.jak1_nm, p.jak1_nm, 1.0, 1.0))
    .collect();
    let all_half_max = hill_at_ic50_all
        .iter()
        .all(|&r| (r - 0.5).abs() < PRURITUS_AT_EC50);
    h.check_bool("Hill at C=IC50: R = 0.5 for all panels", all_half_max);

    // Check 10: Hill dose-response at C=100*IC50: R > 0.99 (saturation)
    let r_sat = hill_dose_response(1000.0, 10.0, 1.0, 1.0);
    h.check_bool(
        "Hill dose-response at C=100*IC50: R > 0.99",
        r_sat > HILL_SATURATION_100X,
    );

    // Check 11: Hill dose-response at C=0: R = 0.0
    let r_zero = hill_dose_response(0.0, 10.0, 1.0, 1.0);
    h.check_abs(
        "Hill dose-response at C=0: R = 0.0",
        r_zero,
        0.0,
        MACHINE_EPSILON,
    );

    // Check 12: pathway_selectivity_score: oclacitinib > baricitinib for JAK1 target
    let ocla_off = [
        oclacitinib.jak2_nm,
        oclacitinib.jak3_nm,
        oclacitinib.tyk2_nm,
    ];
    let baricitinib = &human_panels[2];
    let bari_off = [
        baricitinib.jak2_nm,
        baricitinib.jak3_nm,
        baricitinib.tyk2_nm,
    ];
    let ocla_score = pathway_selectivity_score(oclacitinib.jak1_nm, &ocla_off);
    let bari_score = pathway_selectivity_score(baricitinib.jak1_nm, &bari_off);
    h.check_bool(
        "pathway_selectivity_score: oclacitinib > baricitinib for JAK1 target",
        ocla_score > bari_score,
    );

    // Check 13: selectivity_index(10, 1000) = 100
    let si = selectivity_index(10.0, 1000.0);
    h.check_abs(
        "selectivity_index(10, 1000) = 100",
        si,
        100.0,
        JAK1_SELECTIVITY,
    );

    // Check 14: All panels have 4 targets
    let all_have_four = human_panels.iter().all(|p| {
        [p.jak1_nm, p.jak2_nm, p.jak3_nm, p.tyk2_nm]
            .iter()
            .all(|x| x.is_finite())
    }) && [
        oclacitinib.jak1_nm,
        oclacitinib.jak2_nm,
        oclacitinib.jak3_nm,
        oclacitinib.tyk2_nm,
    ]
    .iter()
    .all(|x| x.is_finite());
    h.check_bool("All panels have 4 targets", all_have_four);

    // Check 15: Determinism: same panel → same selectivity
    let sel1 = canine_jak_ic50_panel().jak1_selectivity();
    let sel2 = canine_jak_ic50_panel().jak1_selectivity();
    h.check_abs(
        "Determinism: same panel → same selectivity",
        sel1,
        sel2,
        MACHINE_EPSILON,
    );

    h.exit();
}
