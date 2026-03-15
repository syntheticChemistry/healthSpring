// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
//! Exp093: `ChEMBL` JAK inhibitor selectivity panel (DD-004)

use healthspring_barracuda::comparative::canine::{
    canine_jak_ic50_panel, human_jak_reference_panels,
};
use healthspring_barracuda::discovery::{
    pathway_selectivity_score, score_panel, selectivity_index, TissueContext,
};
use healthspring_barracuda::pkpd::hill_dose_response;
use healthspring_barracuda::provenance::{log_analytical, AnalyticalProvenance};
use healthspring_barracuda::tolerances::{DETERMINISM, HILL_AT_EC50, JAK1_SELECTIVITY};
use healthspring_barracuda::validation::ValidationHarness;

const HILL_PROV: AnalyticalProvenance = AnalyticalProvenance {
    formula: "R = Emax × C^n / (IC50^n + C^n) at C=IC50 → 0.5",
    reference: "Hill 1910, J Physiol",
    doi: Some("10.1113/jphysiol.1910.sp001397"),
};

fn main() {
    let mut h = ValidationHarness::new("exp093_chembl_jak_panel");
    log_analytical(&HILL_PROV);

    let canine = canine_jak_ic50_panel();
    let human_panels = human_jak_reference_panels();

    // 1. Canine oclacitinib: JAK1 = 10 nM
    h.check_abs("canine oclacitinib: JAK1 = 10 nM", canine.jak1_nm, 10.0, JAK1_SELECTIVITY);

    // 2. JAK1 selectivity > 50
    h.check_lower("canine oclacitinib: JAK1 selectivity > 50", canine.jak1_selectivity(), 50.0);

    // 3. Tofacitinib: JAK3 < JAK1
    let tofa = &human_panels[0];
    h.check_bool("tofacitinib: JAK3 < JAK1", tofa.jak3_nm < tofa.jak1_nm);

    // 4. Ruxolitinib: JAK1 ≈ JAK2
    let ruxo = &human_panels[1];
    h.check_upper("ruxolitinib: JAK1 ≈ JAK2", (ruxo.jak1_nm / ruxo.jak2_nm - 1.0).abs(), 0.25);

    // Build compound panel
    let panel: Vec<(&str, f64, Vec<f64>)> = {
        let mut v = vec![("oclacitinib", canine.jak1_nm, vec![canine.jak2_nm, canine.jak3_nm, canine.tyk2_nm])];
        for p in &human_panels {
            v.push((p.compound.as_str(), p.jak1_nm, vec![p.jak2_nm, p.jak3_nm, p.tyk2_nm]));
        }
        v
    };
    let compounds_ref: Vec<(&str, f64, &[f64])> = panel.iter().map(|(n, on, off)| (*n, *on, off.as_slice())).collect();

    // 5. Oclacitinib highest pathway selectivity
    let score_ocla = pathway_selectivity_score(canine.jak1_nm, &[canine.jak2_nm, canine.jak3_nm, canine.tyk2_nm]);
    let score_tofa = pathway_selectivity_score(tofa.jak1_nm, &[tofa.jak2_nm, tofa.jak3_nm, tofa.tyk2_nm]);
    h.check_bool("pathway: oclacitinib highest for JAK1", score_ocla > score_tofa);

    // 6. Tofacitinib lowest pathway selectivity for JAK1
    let bari = &human_panels[2];
    let score_ruxo = pathway_selectivity_score(ruxo.jak1_nm, &[ruxo.jak2_nm, ruxo.jak3_nm, ruxo.tyk2_nm]);
    let score_bari = pathway_selectivity_score(bari.jak1_nm, &[bari.jak2_nm, bari.jak3_nm, bari.tyk2_nm]);
    h.check_bool(
        "pathway: tofacitinib lowest for JAK1",
        score_tofa <= score_ocla && score_tofa <= score_ruxo && score_tofa <= score_bari,
    );

    // 7. Hill at IC50 = 0.5 for all
    for (i, &ic50) in [canine.jak1_nm, tofa.jak1_nm, ruxo.jak1_nm, bari.jak1_nm].iter().enumerate() {
        h.check_abs(&format!("Hill at IC50: compound {i}"), hill_dose_response(ic50, ic50, 1.0, 1.0), 0.5, HILL_AT_EC50);
    }

    // 8. Hill monotonicity
    let concs: Vec<f64> = (0..50).map(|i| 10.0_f64.powf(-1.0 + 4.0 * f64::from(i) / 49.0)).collect();
    for (i, &ic50) in [canine.jak1_nm, tofa.jak1_nm, ruxo.jak1_nm, bari.jak1_nm].iter().enumerate() {
        let mut prev = 0.0;
        let monotonic = concs.iter().all(|&c| {
            let r = hill_dose_response(c, ic50, 1.0, 1.0);
            let ok = r >= prev - 1e-12;
            prev = r;
            ok
        });
        h.check_bool(&format!("Hill monotonicity: compound {i}"), monotonic);
    }

    // 9. MATRIX panel: 4 entries
    let ctx = TissueContext { localization_length: 10.0, tissue_thickness: 1.0, w_baseline: 5.0, w_treated: 5.0 };
    let matrix_entries = score_panel(&compounds_ref, "AD", &ctx);
    let count = matrix_entries.len() as u64;
    h.check_exact("MATRIX panel: 4 entries", count, 4);

    // 10. Oclacitinib ranks highest
    let mut sorted = matrix_entries;
    sorted.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap_or(core::cmp::Ordering::Equal));
    h.check_bool("MATRIX: oclacitinib ranks highest", sorted[0].compound == "oclacitinib");

    // 11. Cross-species JAK1 IC50 in reasonable range
    h.check_bool("cross-species: canine JAK1 in [1, 50] nM", canine.jak1_nm > 1.0 && canine.jak1_nm < 50.0);

    // 12. SI(JAK1, JAK2) = 100
    h.check_abs("SI: oclacitinib JAK1/JAK2 = 100", selectivity_index(canine.jak1_nm, canine.jak2_nm), 100.0, JAK1_SELECTIVITY);

    // 13. SI(JAK1, JAK3) tofacitinib = 0.5
    h.check_abs("SI: tofacitinib JAK1/JAK3 = 0.5", selectivity_index(tofa.jak1_nm, tofa.jak3_nm), 0.5, JAK1_SELECTIVITY);

    // 14. All Hill non-negative
    let all_nonneg = concs.iter().all(|&c| {
        [canine.jak1_nm, tofa.jak1_nm, ruxo.jak1_nm, bari.jak1_nm]
            .iter()
            .all(|&ic50| hill_dose_response(c, ic50, 1.0, 1.0) >= -1e-12)
    });
    h.check_bool("Hill: all responses non-negative", all_nonneg);

    // 15. Determinism
    let run1 = score_panel(&compounds_ref, "AD", &ctx);
    let run2 = score_panel(&compounds_ref, "AD", &ctx);
    let same = run1.iter().zip(run2.iter()).all(|(a, b)| (a.combined_score - b.combined_score).abs() < DETERMINISM);
    h.check_bool("determinism: same panel → same scores", same);

    h.exit();
}
