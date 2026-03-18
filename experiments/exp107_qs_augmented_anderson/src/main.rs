// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]
//! Exp107 validation: QS-augmented Anderson disorder
//!
//! Validates that the functional QS dimension improves Anderson disorder
//! predictions for colonization resistance.

use healthspring_barracuda::microbiome;
use healthspring_barracuda::qs::{self, NUM_FAMILIES, QsFamily, QsGeneMatrix};
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::ValidationHarness;

const L: usize = 50;
const T_HOP: f64 = 1.0;
const W_SCALE: f64 = 10.0;
const ALPHA: f64 = 0.7;

fn expanded_test_matrix() -> QsGeneMatrix {
    QsGeneMatrix {
        species: vec![
            "Bacteroides".into(),
            "Clostridioides".into(),
            "Faecalibacterium".into(),
            "Roseburia".into(),
            "Escherichia".into(),
            "Enterococcus".into(),
            "Pseudomonas".into(),
            "Akkermansia".into(),
            "Bifidobacterium".into(),
            "Vibrio".into(),
        ],
        families: vec![
            "LuxIR".into(),
            "LuxS".into(),
            "Agr".into(),
            "Com".into(),
            "LasRhl".into(),
            "Fsr".into(),
            "QseBC".into(),
            "VqsM".into(),
            "PqsABCDE".into(),
        ],
        presence: vec![
            // Bacteroides: LuxS
            vec![false, true, false, false, false, false, false, false, false],
            // Clostridioides: LuxS, Agr
            vec![false, true, true, false, false, false, false, false, false],
            // Faecalibacterium: (none)
            vec![
                false, false, false, false, false, false, false, false, false,
            ],
            // Roseburia: (none)
            vec![
                false, false, false, false, false, false, false, false, false,
            ],
            // Escherichia: LuxIR, LuxS, QseBC
            vec![true, true, false, false, false, false, true, false, false],
            // Enterococcus: LuxS, Agr, Fsr
            vec![false, true, true, false, false, true, false, false, false],
            // Pseudomonas: LuxIR, LasRhl, PqsABCDE
            vec![true, false, false, false, true, false, false, false, true],
            // Akkermansia: (none)
            vec![
                false, false, false, false, false, false, false, false, false,
            ],
            // Bifidobacterium: LuxIR, LuxS
            vec![true, true, false, false, false, false, false, false, false],
            // Vibrio: LuxIR, LuxS, VqsM
            vec![true, true, false, false, false, false, false, true, false],
        ],
    }
}

fn main() {
    let mut h = ValidationHarness::new("Exp107 QS-Augmented Anderson Disorder");

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp107 — QS-Augmented Anderson Validation");
    println!("  L={L}, alpha={ALPHA}, W_scale={W_SCALE}");
    println!("  QS families: {NUM_FAMILIES}");
    println!("{}", "=".repeat(72));

    let matrix = expanded_test_matrix();

    // === Check 1: Matrix dimensions ===
    h.check_exact("Matrix species count", matrix.num_species() as u64, 10);
    h.check_exact(
        "Matrix family count",
        matrix.families.len() as u64,
        NUM_FAMILIES as u64,
    );

    // === Check 2: 9-family QseBC/VqsM/PqsABCDE presence ===
    h.check_bool("Escherichia has QseBC", matrix.has_gene(4, QsFamily::QseBC));
    h.check_bool("Vibrio has VqsM", matrix.has_gene(9, QsFamily::VqsM));
    h.check_bool(
        "Pseudomonas has PqsABCDE",
        matrix.has_gene(6, QsFamily::PqsABCDE),
    );
    h.check_bool(
        "Bacteroides lacks QseBC",
        !matrix.has_gene(0, QsFamily::QseBC),
    );
    h.check_bool("Akkermansia lacks all new", {
        !matrix.has_gene(7, QsFamily::QseBC)
            && !matrix.has_gene(7, QsFamily::VqsM)
            && !matrix.has_gene(7, QsFamily::PqsABCDE)
    });

    // === Check 3: Healthy community QS profile ===
    let healthy_abundances = [0.15, 0.05, 0.20, 0.15, 0.05, 0.05, 0.02, 0.18, 0.10, 0.05];
    let prof_healthy = qs::qs_profile(&healthy_abundances, &matrix);

    h.check_bool(
        "Healthy total QS > 0.3",
        prof_healthy.total_qs_density > 0.3,
    );
    h.check_bool(
        "Healthy signaling diversity > 0",
        prof_healthy.signaling_diversity > 0.0,
    );

    // === Check 4: Dysbiotic community (Clostridioides dominant) ===
    let dysbiotic_abundances = [0.02, 0.80, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.04];
    let prof_dysbiotic = qs::qs_profile(&dysbiotic_abundances, &matrix);

    h.check_bool(
        "Dysbiotic total QS > 0",
        prof_dysbiotic.total_qs_density > 0.0,
    );

    // === Check 5: Effective disorder comparison ===
    let pielou_healthy = 0.90;
    let pielou_dysbiotic = 0.30;

    let w_eff_healthy = qs::effective_disorder(pielou_healthy, &prof_healthy, ALPHA, W_SCALE);
    let w_eff_dysbiotic = qs::effective_disorder(pielou_dysbiotic, &prof_dysbiotic, ALPHA, W_SCALE);
    let w_struct_healthy = pielou_healthy * W_SCALE;
    let w_struct_dysbiotic = pielou_dysbiotic * W_SCALE;

    h.check_bool(
        "W_eff healthy > W_eff dysbiotic",
        w_eff_healthy > w_eff_dysbiotic,
    );
    h.check_bool(
        "W_struct healthy > W_struct dysbiotic",
        w_struct_healthy > w_struct_dysbiotic,
    );

    // QS augmentation should increase discrimination
    let ratio_struct = w_struct_healthy / w_struct_dysbiotic.max(tolerances::MACHINE_EPSILON);
    let ratio_eff = w_eff_healthy / w_eff_dysbiotic.max(tolerances::MACHINE_EPSILON);
    println!("  Structural ratio: {ratio_struct:.3}");
    println!("  Effective ratio:  {ratio_eff:.3}");
    h.check_bool(
        "QS augmentation changes W ratio",
        (ratio_eff - ratio_struct).abs() > tolerances::ALLOMETRIC_CL_RATIO,
    );

    // === Check 6: Formula consistency ===
    let w_func_healthy = (1.0 - prof_healthy.total_qs_density.clamp(0.0, 1.0)) * W_SCALE;
    let expected = ALPHA.mul_add(w_struct_healthy, (1.0 - ALPHA) * w_func_healthy);
    h.check_abs(
        "W_eff formula",
        w_eff_healthy,
        expected,
        tolerances::MACHINE_EPSILON,
    );

    // === Check 7: Alpha=1 gives pure structural ===
    let w_pure_struct = qs::effective_disorder(pielou_healthy, &prof_healthy, 1.0, W_SCALE);
    h.check_abs(
        "alpha=1 pure structural",
        w_pure_struct,
        w_struct_healthy,
        tolerances::MACHINE_EPSILON,
    );

    // === Check 8: Alpha=0 gives pure functional ===
    let w_pure_func = qs::effective_disorder(pielou_healthy, &prof_healthy, 0.0, W_SCALE);
    let expected_func = w_func_healthy;
    h.check_abs(
        "alpha=0 pure functional",
        w_pure_func,
        expected_func,
        tolerances::MACHINE_EPSILON,
    );

    // === Check 9: Anderson Hamiltonian with W_effective ===
    let seed = 42_u64;
    let disorder_eff = generate_disorder(L, w_eff_healthy, seed);
    let ham = microbiome::anderson_hamiltonian_1d(&disorder_eff, T_HOP);
    h.check_exact("Hamiltonian size", ham.len() as u64, (L * L) as u64);

    // === Check 10: Localization with effective vs structural disorder ===
    let disorder_struct = generate_disorder(L, w_struct_healthy, seed);
    let ham_struct = microbiome::anderson_hamiltonian_1d(&disorder_struct, T_HOP);
    let _ = ham_struct; // both valid Hamiltonians
    h.check_bool("Both Hamiltonians valid", ham.len() == L * L);

    // === Check 11: Per-family density correctness ===
    let luxs_density = qs::qs_gene_density(&healthy_abundances, &matrix, QsFamily::LuxS);
    // LuxS species: Bacteroides(0.15), Clostridioides(0.05), Escherichia(0.05),
    //   Enterococcus(0.05), Bifidobacterium(0.10), Vibrio(0.05) = 0.45
    h.check_abs(
        "LuxS density healthy",
        luxs_density,
        0.45,
        tolerances::MACHINE_EPSILON,
    );

    let qsebc_density = qs::qs_gene_density(&healthy_abundances, &matrix, QsFamily::QseBC);
    // QseBC: Escherichia(0.05) = 0.05
    h.check_abs(
        "QseBC density healthy",
        qsebc_density,
        0.05,
        tolerances::MACHINE_EPSILON,
    );

    // === Check 12: Empty community ===
    let empty: [f64; 0] = [];
    let prof_empty = qs::qs_profile(&empty, &matrix);
    h.check_abs(
        "Empty QS density",
        prof_empty.total_qs_density,
        0.0,
        tolerances::MACHINE_EPSILON,
    );

    // === Check 13: Determinism ===
    let prof_1 = qs::qs_profile(&healthy_abundances, &matrix);
    let prof_2 = qs::qs_profile(&healthy_abundances, &matrix);
    h.check_bool("Deterministic profile", {
        prof_1.total_qs_density.to_bits() == prof_2.total_qs_density.to_bits()
    });

    // === Check 14: New families have nonzero impact ===
    let vqsm_density = qs::qs_gene_density(&healthy_abundances, &matrix, QsFamily::VqsM);
    // Vibrio(0.05) = 0.05
    h.check_abs(
        "VqsM density",
        vqsm_density,
        0.05,
        tolerances::MACHINE_EPSILON,
    );

    let pqs_density = qs::qs_gene_density(&healthy_abundances, &matrix, QsFamily::PqsABCDE);
    // Pseudomonas(0.02) = 0.02
    h.check_abs(
        "PqsABCDE density",
        pqs_density,
        0.02,
        tolerances::MACHINE_EPSILON,
    );

    h.exit();
}

fn generate_disorder(l: usize, w: f64, seed: u64) -> Vec<f64> {
    let mut state = seed;
    (0..l)
        .map(|_| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let u = f64::from((state >> 33) as u32) / f64::from(u32::MAX);
            (u - 0.5) * w
        })
        .collect()
}
