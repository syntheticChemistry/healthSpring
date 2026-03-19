// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
//! Exp097: Affinity landscape — computational preprocessing of the low-affinity
//! binding regime for cancer targeting, infection response, and probiotic adhesion.
//!
//! Uses computation as experiment preprocessor: models binding landscapes to
//! identify low-affinity regimes of interest *before* plate screening.
//!
//! Three studies:
//! 1. **Composite cancer targeting**: multiple weak binders create selective signal
//! 2. **Colonization resistance**: probiotic adhesion through cumulative weak binding
//! 3. **Affinity distribution analysis**: Gini-based characterization of binding breadth

use healthspring_barracuda::discovery;
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("exp097_affinity_landscape");

    study_1_composite_cancer_targeting(&mut h);
    study_2_colonization_resistance(&mut h);
    study_3_affinity_distribution(&mut h);
    study_4_cross_reactivity(&mut h);

    h.exit();
}

/// Study 1: Composite cancer targeting via coincidence detection.
///
/// Cancer cells express disordered surface markers — multiple weak binding
/// events coincide to create a composite signal. Normal cells have fewer
/// matching markers, so the composite signal is below threshold.
fn study_1_composite_cancer_targeting(h: &mut ValidationHarness) {
    println!("\n─── Study 1: Composite Cancer Targeting ───");

    let compound_conc = 1.0;
    let hill_n = 1.0;

    let cancer_ic50s: Vec<f64> = vec![20.0; 15];
    let normal_ic50s: Vec<f64> = vec![200.0; 4];

    let cancer_profile = discovery::binding_profile(compound_conc, &cancer_ic50s, hill_n);
    let normal_profile = discovery::binding_profile(compound_conc, &normal_ic50s, hill_n);

    let cancer_composite = discovery::composite_binding_score(&cancer_profile);
    let normal_composite = discovery::composite_binding_score(&normal_profile);

    println!("  Cancer: 15 markers at IC50=20µM → composite = {cancer_composite:.4}");
    println!("  Normal:  4 markers at IC50=200µM → composite = {normal_composite:.4}");

    h.check_lower(
        "cancer composite > 50% (many weak binders accumulate)",
        cancer_composite,
        0.5,
    );
    h.check_upper(
        "normal composite < 5% (few weak binders don't accumulate)",
        normal_composite,
        0.05,
    );

    let selectivity = discovery::low_affinity_selectivity(&cancer_profile, &normal_profile);
    println!("  Low-affinity selectivity index: {selectivity:.1}x");

    h.check_lower("selectivity > 10x (cancer vs normal)", selectivity, 10.0);

    let individual_occupancy = discovery::fractional_occupancy(compound_conc, 20.0, hill_n);
    println!("  Individual binding per marker: {individual_occupancy:.4} (deliberately weak)");

    h.check_upper(
        "individual occupancy < 10% (each interaction is weak)",
        individual_occupancy,
        0.10,
    );
    h.check_lower(
        "individual occupancy > 1% (not negligible)",
        individual_occupancy,
        0.01,
    );

    println!("  → Selectivity emerges from coincidence, not from individual affinity");
}

/// Study 2: Colonization resistance through cumulative weak binding.
///
/// Multiple probiotic species, each weakly adhering to epithelial sites,
/// create cumulative occupancy that prevents pathogen colonization.
fn study_2_colonization_resistance(h: &mut ValidationHarness) {
    println!("\n─── Study 2: Colonization Resistance ───");

    let n_sites = 50;
    let resistance_threshold = 0.4;

    let diverse_community: Vec<Vec<f64>> = (0..8)
        .map(|seed| discovery::disorder_adhesion_profile(n_sites, 8.0, 0.5, 100 + seed))
        .collect();

    let cr_diverse = discovery::colonization_resistance(&diverse_community, resistance_threshold);
    println!("  Diverse community (8 species, weak adhesion): CR = {cr_diverse:.3}");

    h.check_lower(
        "diverse community: CR > 70% (cumulative weak binding protects)",
        cr_diverse,
        0.70,
    );

    let monoculture: Vec<Vec<f64>> =
        vec![discovery::disorder_adhesion_profile(n_sites, 3.0, 0.5, 42)];

    let cr_mono = discovery::colonization_resistance(&monoculture, resistance_threshold);
    println!("  Monoculture (1 species, weak adhesion): CR = {cr_mono:.3}");

    h.check_upper(
        "monoculture: CR < diverse (single weak binder insufficient)",
        cr_mono,
        cr_diverse,
    );

    let strong_mono: Vec<Vec<f64>> =
        vec![discovery::disorder_adhesion_profile(n_sites, 0.5, 0.5, 42)];

    let cr_strong = discovery::colonization_resistance(&strong_mono, resistance_threshold);
    println!("  Single strong binder (1 species, strong adhesion): CR = {cr_strong:.3}");

    println!("  → Diversity of weak binders > monoculture of any strength for coverage");
}

/// Study 3: Affinity distribution analysis using Gini coefficient.
///
/// Characterizes compounds by their binding breadth (Gini) rather than
/// just their best hit (max IC50). Low Gini = broad weak binding;
/// high Gini = narrow strong binding.
fn study_3_affinity_distribution(h: &mut ValidationHarness) {
    println!("\n─── Study 3: Affinity Distribution Analysis ───");

    let conc = 1.0;
    let hill_n = 1.0;
    let threshold = 0.01;

    let broad_ic50s = vec![50.0; 20];
    let broad = discovery::analyze_affinity_distribution(conc, &broad_ic50s, hill_n, threshold);
    println!(
        "  Broad weak binder: gini={:.3}, breadth={:.1}%, mean_occ={:.4}, composite={:.4}",
        broad.gini,
        broad.breadth * 100.0,
        broad.mean_occupancy,
        broad.composite_score,
    );

    let mut narrow_ic50s = vec![500.0; 20];
    narrow_ic50s[0] = 0.1;
    let narrow = discovery::analyze_affinity_distribution(conc, &narrow_ic50s, hill_n, threshold);
    println!(
        "  Narrow strong binder: gini={:.3}, breadth={:.1}%, mean_occ={:.4}, composite={:.4}",
        narrow.gini,
        narrow.breadth * 100.0,
        narrow.mean_occupancy,
        narrow.composite_score,
    );

    h.check_upper("broad binder: low Gini (< 0.15)", broad.gini, 0.15);
    h.check_lower("narrow binder: high Gini (> 0.5)", narrow.gini, 0.5);

    h.check_lower("broad binder: high breadth (> 90%)", broad.breadth, 0.90);
    h.check_upper("narrow binder: low breadth (< 20%)", narrow.breadth, 0.20);

    h.check_lower(
        "narrow binder: higher composite (strong single hit dominates)",
        narrow.composite_score,
        broad.composite_score,
    );

    let broad_coverage_value = broad.breadth * broad.composite_score;
    let narrow_coverage_value = narrow.breadth * narrow.composite_score;
    println!(
        "  Coverage value (breadth × composite): broad={broad_coverage_value:.4} vs narrow={narrow_coverage_value:.4}"
    );
    h.check_lower(
        "broad binder: higher coverage value (breadth × composite)",
        broad_coverage_value,
        narrow_coverage_value,
    );

    println!("  → Gini discriminates binding strategy: broad-weak vs narrow-strong");
    println!("  → Traditional HTS selects for high Gini; low-affinity regime selects for low Gini");
    println!("  → Coverage value (breadth × composite) captures the advantage of broad binding");
}

/// Study 4: Cross-reactivity matrix for compound panel.
///
/// Builds the full compound × target binding landscape, demonstrating
/// how the same data used for traditional hit-calling can be reanalyzed
/// for low-affinity applications.
fn study_4_cross_reactivity(h: &mut ValidationHarness) {
    println!("\n─── Study 4: Cross-Reactivity Matrix ───");

    let compound_ic50s = vec![
        vec![0.5, 100.0, 100.0, 100.0, 100.0],
        vec![100.0, 0.3, 100.0, 100.0, 100.0],
        vec![30.0, 40.0, 35.0, 25.0, 45.0],
        vec![20.0, 20.0, 20.0, 20.0, 20.0],
    ];

    let matrix = discovery::cross_reactivity_matrix(1.0, &compound_ic50s, 1.0);

    let target_names = ["JAK1", "JAK2", "JAK3", "TYK2", "EGFR"];
    let compound_names = [
        "Selective-JAK1",
        "Selective-JAK2",
        "Broad-moderate",
        "Broad-weak",
    ];

    println!(
        "  {:>18} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "", target_names[0], target_names[1], target_names[2], target_names[3], target_names[4]
    );
    for (i, row) in matrix.iter().enumerate() {
        print!("  {:>18}", compound_names[i]);
        for &val in row {
            print!(" {val:>8.4}");
        }
        println!();
    }

    h.check_exact("matrix rows = 4 compounds", matrix.len() as u64, 4);
    h.check_exact("matrix cols = 5 targets", matrix[0].len() as u64, 5);

    let selective_composite = discovery::composite_binding_score(&matrix[0]);
    let broad_composite = discovery::composite_binding_score(&matrix[3]);
    println!("\n  Selective (JAK1): composite = {selective_composite:.4}");
    println!("  Broad-weak (all): composite = {broad_composite:.4}");

    h.check_lower(
        "selective has higher composite (one strong hit)",
        selective_composite,
        broad_composite,
    );

    let broad_dist = discovery::analyze_affinity_distribution(1.0, &compound_ic50s[3], 1.0, 0.01);
    let selective_dist =
        discovery::analyze_affinity_distribution(1.0, &compound_ic50s[0], 1.0, 0.01);

    h.check_lower(
        "broad-weak: higher breadth than selective",
        broad_dist.breadth,
        selective_dist.breadth,
    );
    h.check_upper(
        "broad-weak: lower Gini (more uniform binding)",
        broad_dist.gini,
        selective_dist.gini,
    );

    println!("  → Same data, different question: 'what binds broadly?' vs 'what binds strongly?'");
    println!("  → Computation as preprocessor: identify broad-weak candidates BEFORE screening");
    println!("  → Low Gini + high breadth = candidate for combinatorial targeting");
}
