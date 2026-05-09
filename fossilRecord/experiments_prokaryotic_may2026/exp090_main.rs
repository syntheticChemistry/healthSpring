// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
//! Exp090: Anderson-augmented MATRIX drug repurposing scoring (Fajgenbaum 2018) Anderson-augmented MATRIX drug repurposing scoring (Fajgenbaum 2018)

use healthspring_barracuda::discovery::{
    TissueContext, disorder_impact_factor, matrix_combined_score, pathway_selectivity_score,
    score_compound, tissue_geometry_factor,
};
use healthspring_barracuda::provenance::{AnalyticalProvenance, log_analytical};
use healthspring_barracuda::tolerances::{
    DETERMINISM, DISORDER_IMPACT, MATRIX_COMBINED, MATRIX_PATHWAY, TISSUE_GEOMETRY,
    TISSUE_GEOMETRY_SATURATION, TISSUE_GEOMETRY_ZERO,
};
use healthspring_barracuda::validation::ValidationHarness;

const MATRIX_PROV: AnalyticalProvenance = AnalyticalProvenance {
    formula: "MATRIX pathway × geometry × disorder (Fajgenbaum 2018)",
    reference: "Fajgenbaum DC et al. NEJM 379:1941",
    doi: None,
};

struct JakPanel {
    oclacitinib: (f64, [f64; 3]),
    tofacitinib: (f64, [f64; 3]),
    ruxolitinib: (f64, [f64; 3]),
    baricitinib: (f64, [f64; 3]),
}

fn validate_pathway_selectivity(h: &mut ValidationHarness, panel: &JakPanel) -> f64 {
    let score_ocla = pathway_selectivity_score(panel.oclacitinib.0, &panel.oclacitinib.1);
    let score_tofa = pathway_selectivity_score(panel.tofacitinib.0, &panel.tofacitinib.1);
    let score_ruxo = pathway_selectivity_score(panel.ruxolitinib.0, &panel.ruxolitinib.1);
    let score_bari = pathway_selectivity_score(panel.baricitinib.0, &panel.baricitinib.1);
    h.check_bool(
        "pathway: oclacitinib highest JAK1 selectivity",
        score_ocla > score_tofa && score_ocla > score_ruxo && score_ocla > score_bari,
    );
    h.check_abs(
        "pathway: no off-targets → 0",
        pathway_selectivity_score(10.0, &[]),
        0.0,
        MATRIX_PATHWAY,
    );
    h.check_abs(
        "pathway: equal IC50s → 0.5",
        pathway_selectivity_score(10.0, &[10.0, 10.0]),
        0.5,
        MATRIX_PATHWAY,
    );
    score_ocla
}

fn validate_tissue_geometry(h: &mut ValidationHarness) {
    h.check_lower(
        "tissue: large xi → near 1",
        tissue_geometry_factor(100.0, 1.0),
        TISSUE_GEOMETRY_SATURATION,
    );
    h.check_upper(
        "tissue: small xi → near 0",
        tissue_geometry_factor(0.01, 1.0),
        TISSUE_GEOMETRY_ZERO,
    );
    h.check_abs(
        "tissue: zero thickness → 0",
        tissue_geometry_factor(10.0, 0.0),
        0.0,
        TISSUE_GEOMETRY,
    );
}

fn validate_disorder_impact(h: &mut ValidationHarness) {
    h.check_abs(
        "disorder: neutral → 1.0",
        disorder_impact_factor(5.0, 5.0),
        1.0,
        DISORDER_IMPACT,
    );
    h.check_lower(
        "disorder: beneficial → >1",
        disorder_impact_factor(5.0, 7.5),
        1.0,
    );
    h.check_upper(
        "disorder: harmful → <1",
        disorder_impact_factor(5.0, 2.0),
        1.0,
    );
    h.check_abs(
        "disorder: capped at 2.0",
        disorder_impact_factor(1.0, 100.0),
        2.0,
        DISORDER_IMPACT,
    );
}

fn validate_combined_scoring(
    h: &mut ValidationHarness,
    panel: &JakPanel,
    ctx: &TissueContext,
    score_ocla: f64,
) {
    let combined = matrix_combined_score(0.8, 0.6, 1.2);
    h.check_abs(
        "combined = pathway × geometry × disorder",
        combined,
        0.8 * 0.6 * 1.2,
        MATRIX_COMBINED,
    );

    let entry = score_compound("test", "AD", panel.oclacitinib.0, &panel.oclacitinib.1, ctx);
    let manual = score_ocla
        * tissue_geometry_factor(ctx.localization_length, ctx.tissue_thickness)
        * disorder_impact_factor(ctx.w_baseline, ctx.w_treated);
    h.check_abs(
        "score_compound: correct combined",
        entry.combined_score,
        manual,
        MATRIX_COMBINED,
    );

    let entries = [
        (
            "Oclacitinib",
            panel.oclacitinib.0,
            panel.oclacitinib.1.as_slice(),
        ),
        (
            "Tofacitinib",
            panel.tofacitinib.0,
            panel.tofacitinib.1.as_slice(),
        ),
        (
            "Ruxolitinib",
            panel.ruxolitinib.0,
            panel.ruxolitinib.1.as_slice(),
        ),
        (
            "Baricitinib",
            panel.baricitinib.0,
            panel.baricitinib.1.as_slice(),
        ),
    ];
    let mut scored: Vec<_> = entries
        .iter()
        .map(|(n, on, off)| score_compound(n, "AD", *on, off, ctx))
        .collect();
    scored.sort_by(|a, b| {
        b.combined_score
            .partial_cmp(&a.combined_score)
            .unwrap_or(core::cmp::Ordering::Equal)
    });
    h.check_bool(
        "panel: oclacitinib highest for AD",
        scored[0].compound == "Oclacitinib",
    );

    let good_ctx = TissueContext {
        localization_length: 50.0,
        ..*ctx
    };
    let poor_ctx = TissueContext {
        localization_length: 0.1,
        ..*ctx
    };
    let good = score_compound("a", "AD", 5.0, &[500.0], &good_ctx);
    let poor = score_compound("b", "AD", 5.0, &[500.0], &poor_ctx);
    h.check_bool(
        "geometry reranks: same pathway, different combined",
        (good.pathway_score - poor.pathway_score).abs() < MATRIX_PATHWAY
            && good.combined_score > poor.combined_score,
    );

    let run1 = score_compound(
        "Oclacitinib",
        "AD",
        panel.oclacitinib.0,
        &panel.oclacitinib.1,
        ctx,
    );
    let run2 = score_compound(
        "Oclacitinib",
        "AD",
        panel.oclacitinib.0,
        &panel.oclacitinib.1,
        ctx,
    );
    h.check_abs(
        "determinism: re-score identical",
        (run1.combined_score - run2.combined_score).abs(),
        0.0,
        DETERMINISM,
    );
}

fn main() {
    let mut h = ValidationHarness::new("exp090_matrix_scoring");
    log_analytical(&MATRIX_PROV);

    let panel = JakPanel {
        oclacitinib: (10.0, [1000.0, 10000.0, 10000.0]),
        tofacitinib: (3.2, [4.1, 1.6, 34.0]),
        ruxolitinib: (3.3, [2.8, 428.0, 19.0]),
        baricitinib: (5.9, [5.7, 560.0, 53.0]),
    };
    let ctx = TissueContext {
        localization_length: 10.0,
        tissue_thickness: 1.0,
        w_baseline: 5.0,
        w_treated: 5.0,
    };

    let score_ocla = validate_pathway_selectivity(&mut h, &panel);
    validate_tissue_geometry(&mut h);
    validate_disorder_impact(&mut h);
    validate_combined_scoring(&mut h, &panel, &ctx, score_ocla);

    h.exit();
}
