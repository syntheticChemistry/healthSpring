// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
//! Exp092: ADDRC compound library batch IC50 profiling
//!
//! Validates selectivity index, IC50 estimation from Hill curves, batch sweep,
//! and ranking for a synthetic 5-compound panel across 2 targets.

use healthspring_barracuda::discovery::{
    CompoundProfile, batch_ic50_sweep, estimate_ic50, rank_by_selectivity, selectivity_index,
};
use healthspring_barracuda::provenance::{AnalyticalProvenance, log_analytical};
use healthspring_barracuda::tolerances::{DETERMINISM, SELECTIVITY_INDEX};
use healthspring_barracuda::validation::ValidationHarness;

const HILL_PROV: AnalyticalProvenance = AnalyticalProvenance {
    formula: "R = Emax × C^n / (IC50^n + C^n)",
    reference: "Hill 1910, J Physiol",
    doi: Some("10.1113/jphysiol.1910.sp001397"),
};

fn hill_response(conc: f64, ic50: f64, n: f64, emax: f64) -> f64 {
    emax * conc.powf(n) / (ic50.powf(n) + conc.powf(n))
}

fn build_panel() -> Vec<CompoundProfile> {
    let targets = vec!["JAK1".to_string(), "JAK2".to_string()];
    vec![
        CompoundProfile {
            name: "Compound A".into(),
            ic50_nm: vec![5.0, 500.0],
            target_names: targets.clone(),
        },
        CompoundProfile {
            name: "Compound B".into(),
            ic50_nm: vec![50.0, 60.0],
            target_names: targets.clone(),
        },
        CompoundProfile {
            name: "Compound C".into(),
            ic50_nm: vec![2.0, 2000.0],
            target_names: targets.clone(),
        },
        CompoundProfile {
            name: "Compound D".into(),
            ic50_nm: vec![100.0, 200.0],
            target_names: targets.clone(),
        },
        CompoundProfile {
            name: "Compound E".into(),
            ic50_nm: vec![10.0, 1000.0],
            target_names: targets,
        },
    ]
}

fn validate_batch_ranking(h: &mut ValidationHarness, compounds: &[CompoundProfile]) {
    let mut scorecards = batch_ic50_sweep(compounds, 0);
    h.check_exact(
        "batch_ic50_sweep: correct count",
        scorecards.len() as u64,
        5,
    );

    rank_by_selectivity(&mut scorecards);
    let ordered = scorecards
        .windows(2)
        .all(|w| w[0].primary_selectivity >= w[1].primary_selectivity);
    h.check_bool("rank_by_selectivity: most selective first", ordered);
    h.check_bool(
        "ranking: Compound C first",
        scorecards[0].compound == "Compound C",
    );
    h.check_bool(
        "ranking: Compound B last",
        scorecards[4].compound == "Compound B",
    );

    let all_positive = scorecards.iter().all(|s| s.primary_selectivity > 0.0);
    h.check_bool("all scorecards: positive selectivity", all_positive);

    let mut scorecards2 = batch_ic50_sweep(compounds, 0);
    rank_by_selectivity(&mut scorecards2);
    let same_ranking = scorecards.iter().zip(scorecards2.iter()).all(|(a, b)| {
        a.compound == b.compound
            && (a.primary_selectivity - b.primary_selectivity).abs() < DETERMINISM
    });
    h.check_bool("determinism: same panel → same ranking", same_ranking);
}

fn main() {
    let mut h = ValidationHarness::new("exp092_compound_library");
    log_analytical(&HILL_PROV);

    h.check_abs(
        "selectivity_index: SI(5,500)=100",
        selectivity_index(5.0, 500.0),
        100.0,
        SELECTIVITY_INDEX,
    );
    h.check_abs(
        "selectivity_index: SI(50,60)=1.2",
        selectivity_index(50.0, 60.0),
        1.2,
        SELECTIVITY_INDEX,
    );
    h.check_abs(
        "selectivity_index: zero on-target → 0",
        selectivity_index(0.0, 100.0),
        0.0,
        SELECTIVITY_INDEX,
    );

    let ic50_true = 10.0;
    let concs: Vec<f64> = (0..8)
        .map(|i| 0.1 * 10.0_f64.powf(0.5 * f64::from(i)))
        .collect();
    let responses: Vec<f64> = concs
        .iter()
        .map(|&c| hill_response(c, ic50_true, 1.0, 1.0))
        .collect();
    let est = estimate_ic50(&concs, &responses);
    h.check_bool(
        "estimate_ic50: recovers IC50 within 10%",
        est.ic50.is_finite() && (est.ic50 - ic50_true).abs() / ic50_true < 0.10,
    );
    h.check_bool(
        "estimate_ic50: insufficient data → NaN",
        estimate_ic50(&[1.0], &[0.5]).ic50.is_nan(),
    );
    h.check_lower("estimate_ic50: R² > 0.9", est.r_squared, 0.9);

    let compounds = build_panel();
    validate_batch_ranking(&mut h, &compounds);

    h.exit();
}
