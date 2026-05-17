// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::comparative::species_params;
use crate::microbiome;
use crate::pkpd;
use crate::tolerances;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "equine-laminitis",
            track: Track::Comparative,
            tier: Tier::Rust,
            source_experiment: "exp110",
            description: "Equine laminitis: Anderson lattice hoof model, phenylbutazone MM PK, allometric scaling.",
        },
        run,
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "lattice size L < 2^52 — lossless"
)]
fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Hoof lamellae as Anderson lattice");

    let l = 50;
    let t_hop = 1.0;
    let disorder: Vec<f64> = (0..l)
        .map(|i| {
            let x = i as f64 / l as f64;
            2.0 * (x - 0.5) * (x - 0.5)
        })
        .collect();

    let h_mat = microbiome::anderson_hamiltonian_1d(&disorder, t_hop);
    v.check_bool(
        "hamiltonian_flat_size",
        h_mat.len() == l * l,
        &format!("len={}, expected={}", h_mat.len(), l * l),
    );

    let symmetric = (0..l).all(|i| {
        (0..l).all(|j| (h_mat[i * l + j] - h_mat[j * l + i]).abs() < tolerances::MACHINE_EPSILON)
    });
    v.check_bool("hamiltonian_symmetric", symmetric, "");

    let ipr_delta = {
        let mut delta = vec![0.0; l];
        delta[l / 2] = 1.0;
        microbiome::inverse_participation_ratio(&delta)
    };
    v.check_abs_or_rel(
        "ipr_delta_state_is_one",
        ipr_delta,
        1.0,
        tolerances::MACHINE_EPSILON,
        tolerances::TEST_ASSERTION_LOOSE,
    );

    v.section("Phase 2: Structural — Phenylbutazone Michaelis-Menten PK");

    let params = pkpd::MichaelisMentenParams {
        vmax: 800.0,
        km: 25.0,
        vd: 45.0,
    };
    let dose = 500.0;

    let (_, concs) = pkpd::mm_pk_simulate(&params, dose, 24.0, 0.01);
    v.check_bool("trajectory_non_empty", !concs.is_empty(), "");
    v.check_abs_or_rel(
        "initial_concentration",
        concs[0],
        dose / params.vd,
        tolerances::MACHINE_EPSILON,
        tolerances::TEST_ASSERTION_LOOSE,
    );

    let monotone = concs
        .windows(2)
        .all(|w| w[1] <= w[0] + tolerances::MACHINE_EPSILON);
    v.check_bool("mm_pk_monotone_decrease", monotone, "");

    v.section("Phase 3: Structural — Allometric scaling horse vs canine");

    let cl_ref = 1.0;
    let canine_bw = 15.0;
    let equine_bw = 500.0;

    let cl_equine = species_params::allometric_clearance(cl_ref, canine_bw, equine_bw);
    let cl_canine = species_params::allometric_clearance(cl_ref, canine_bw, canine_bw);

    v.check_bool(
        "equine_cl_per_kg_less_than_canine",
        cl_equine / equine_bw < cl_canine / canine_bw,
        &format!(
            "equine_cl/kg={:.6}, canine_cl/kg={:.6}",
            cl_equine / equine_bw,
            cl_canine / canine_bw
        ),
    );

    let t_half = species_params::allometric_half_life(10.0, 2.0);
    let expected = core::f64::consts::LN_2 * 10.0 / 2.0;
    v.check_abs_or_rel(
        "half_life_identity",
        t_half,
        expected,
        tolerances::MACHINE_EPSILON,
        tolerances::TEST_ASSERTION_LOOSE,
    );
}
