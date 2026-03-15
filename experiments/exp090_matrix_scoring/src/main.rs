// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
//! Exp090: Anderson-augmented MATRIX drug repurposing scoring (Fajgenbaum 2018) Anderson-augmented MATRIX drug repurposing scoring (Fajgenbaum 2018)

use healthspring_barracuda::discovery::{
    disorder_impact_factor, matrix_combined_score, pathway_selectivity_score, score_compound,
    tissue_geometry_factor, TissueContext,
};
use healthspring_barracuda::provenance::{log_analytical, AnalyticalProvenance};
use healthspring_barracuda::tolerances::{
    DETERMINISM, DISORDER_IMPACT, MATRIX_COMBINED, MATRIX_PATHWAY, TISSUE_GEOMETRY,
};
use healthspring_barracuda::validation::ValidationHarness;

const MATRIX_PROV: AnalyticalProvenance = AnalyticalProvenance {
    formula: "MATRIX pathway × geometry × disorder (Fajgenbaum 2018)",
    reference: "Fajgenbaum DC et al. NEJM 379:1941",
    doi: None,
};

fn main() {
    let mut h = ValidationHarness::new("exp090_matrix_scoring");
    log_analytical(&MATRIX_PROV);

    let oclacitinib = (10.0, [1000.0, 10000.0, 10000.0]);
    let tofacitinib = (3.2, [4.1, 1.6, 34.0]);
    let ruxolitinib = (3.3, [2.8, 428.0, 19.0]);
    let baricitinib = (5.9, [5.7, 560.0, 53.0]);

    let ctx = TissueContext {
        localization_length: 10.0,
        tissue_thickness: 1.0,
        w_baseline: 5.0,
        w_treated: 5.0,
    };

    // 1. Oclacitinib highest JAK1 selectivity
    let score_ocla = pathway_selectivity_score(oclacitinib.0, &oclacitinib.1);
    let score_tofa = pathway_selectivity_score(tofacitinib.0, &tofacitinib.1);
    let score_ruxo = pathway_selectivity_score(ruxolitinib.0, &ruxolitinib.1);
    let score_bari = pathway_selectivity_score(baricitinib.0, &baricitinib.1);
    h.check_bool(
        "pathway: oclacitinib highest JAK1 selectivity",
        score_ocla > score_tofa && score_ocla > score_ruxo && score_ocla > score_bari,
    );

    // 2. No off-targets → 0.0
    h.check_abs("pathway: no off-targets → 0", pathway_selectivity_score(10.0, &[]), 0.0, MATRIX_PATHWAY);

    // 3. Equal IC50s → 0.5
    h.check_abs("pathway: equal IC50s → 0.5", pathway_selectivity_score(10.0, &[10.0, 10.0]), 0.5, MATRIX_PATHWAY);

    // 4. Large ξ → factor near 1.0
    h.check_lower("tissue: large xi → near 1", tissue_geometry_factor(100.0, 1.0), 0.99);

    // 5. Small ξ → factor near 0.0
    h.check_upper("tissue: small xi → near 0", tissue_geometry_factor(0.01, 1.0), 0.02);

    // 6. Zero thickness → 0.0
    h.check_abs("tissue: zero thickness → 0", tissue_geometry_factor(10.0, 0.0), 0.0, TISSUE_GEOMETRY);

    // 7. Neutral disorder → 1.0
    h.check_abs("disorder: neutral → 1.0", disorder_impact_factor(5.0, 5.0), 1.0, DISORDER_IMPACT);

    // 8. Beneficial → >1.0
    h.check_lower("disorder: beneficial → >1", disorder_impact_factor(5.0, 7.5), 1.0);

    // 9. Harmful → <1.0
    h.check_upper("disorder: harmful → <1", disorder_impact_factor(5.0, 2.0), 1.0);

    // 10. Capped at 2.0
    h.check_abs("disorder: capped at 2.0", disorder_impact_factor(1.0, 100.0), 2.0, DISORDER_IMPACT);

    // 11. Product identity
    let combined = matrix_combined_score(0.8, 0.6, 1.2);
    h.check_abs("combined = pathway × geometry × disorder", combined, 0.8 * 0.6 * 1.2, MATRIX_COMBINED);

    // 12. score_compound correct combined
    let entry = score_compound("test", "AD", oclacitinib.0, &oclacitinib.1, &ctx);
    let manual = score_ocla
        * tissue_geometry_factor(ctx.localization_length, ctx.tissue_thickness)
        * disorder_impact_factor(ctx.w_baseline, ctx.w_treated);
    h.check_abs("score_compound: correct combined", entry.combined_score, manual, MATRIX_COMBINED);

    // 13. Panel ranking: oclacitinib highest
    let panel = [
        ("Oclacitinib", oclacitinib.0, oclacitinib.1.as_slice()),
        ("Tofacitinib", tofacitinib.0, tofacitinib.1.as_slice()),
        ("Ruxolitinib", ruxolitinib.0, ruxolitinib.1.as_slice()),
        ("Baricitinib", baricitinib.0, baricitinib.1.as_slice()),
    ];
    let mut scored: Vec<_> = panel
        .iter()
        .map(|(n, on, off)| score_compound(n, "AD", *on, off, &ctx))
        .collect();
    scored.sort_by(|a, b| {
        b.combined_score
            .partial_cmp(&a.combined_score)
            .unwrap_or(core::cmp::Ordering::Equal)
    });
    h.check_bool("panel: oclacitinib highest for AD", scored[0].compound == "Oclacitinib");

    // 14. Geometry reranks
    let good_ctx = TissueContext { localization_length: 50.0, ..ctx };
    let poor_ctx = TissueContext { localization_length: 0.1, ..ctx };
    let good = score_compound("a", "AD", 5.0, &[500.0], &good_ctx);
    let poor = score_compound("b", "AD", 5.0, &[500.0], &poor_ctx);
    h.check_bool(
        "geometry reranks: same pathway, different combined",
        (good.pathway_score - poor.pathway_score).abs() < MATRIX_PATHWAY
            && good.combined_score > poor.combined_score,
    );

    // 15. Determinism
    let run1 = score_compound("Oclacitinib", "AD", oclacitinib.0, &oclacitinib.1, &ctx);
    let run2 = score_compound("Oclacitinib", "AD", oclacitinib.0, &oclacitinib.1, &ctx);
    h.check_abs("determinism: re-score identical", (run1.combined_score - run2.combined_score).abs(), 0.0, DETERMINISM);

    h.exit();
}
