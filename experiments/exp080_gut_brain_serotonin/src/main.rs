// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]

//! Exp080: Gut-Brain Serotonin Pathway
//!
//! ~90% of body serotonin is gut-derived. Microbiome diversity modulates
//! tryptophan availability for enterochromaffin cell synthesis.
//! Tests the diversity → tryptophan → serotonin causal chain.
//!
//! Reference: Yano et al. 2015 (Cell), Clarke et al. 2013,
//!            Cryan & Dinan 2012 (Nat Rev Neurosci).

use healthspring_barracuda::microbiome;
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("exp080_gut_brain_serotonin");

    let dietary_trp = 200.0;
    let k_synth = 0.8;
    let scale = 0.1;

    let h_healthy = microbiome::shannon_index(&microbiome::communities::HEALTHY_GUT);
    let h_dysbiotic = microbiome::shannon_index(&microbiome::communities::DYSBIOTIC_GUT);
    let h_cdiff = microbiome::shannon_index(&microbiome::communities::CDIFF_COLONIZED);

    let trp_healthy = microbiome::tryptophan_availability(dietary_trp, h_healthy);
    let trp_dysbiotic = microbiome::tryptophan_availability(dietary_trp, h_dysbiotic);

    h.check_bool(
        "diversity→trp: healthy > dysbiotic",
        trp_healthy > trp_dysbiotic,
    );
    h.check_lower(
        "trp_healthy ≥ physiological low",
        trp_healthy,
        tolerances::TRP_RANGE_LOW,
    );
    h.check_upper(
        "trp_healthy ≤ physiological high",
        trp_healthy,
        tolerances::TRP_RANGE_HIGH,
    );

    let ser_healthy = microbiome::gut_serotonin_production(trp_healthy, h_healthy, k_synth, scale);
    let ser_dysbiotic =
        microbiome::gut_serotonin_production(trp_dysbiotic, h_dysbiotic, k_synth, scale);
    h.check_bool(
        "diversity→5HT: healthy > dysbiotic",
        ser_healthy > ser_dysbiotic,
    );

    let ser_cdiff = microbiome::gut_serotonin_production(
        microbiome::tryptophan_availability(dietary_trp, h_cdiff),
        h_cdiff,
        k_synth,
        scale,
    );
    h.check_bool(
        "all serotonin positive",
        ser_healthy > 0.0 && ser_dysbiotic > 0.0 && ser_cdiff > 0.0,
    );
    h.check_bool(
        "5HT ordering: healthy > cdiff > dysbiotic",
        ser_healthy > ser_cdiff && ser_cdiff > ser_dysbiotic,
    );

    let low_div = microbiome::gut_serotonin_production(100.0, 0.5, 1.0, 0.1);
    let mid_div = microbiome::gut_serotonin_production(100.0, 1.5, 1.0, 0.1);
    let high_div = microbiome::gut_serotonin_production(100.0, 2.5, 1.0, 0.1);
    h.check_bool(
        "sigmoid: low < mid < high",
        low_div < mid_div && mid_div < high_div,
    );

    let at_midpoint = microbiome::gut_serotonin_production(100.0, 1.5, 1.0, 0.1);
    h.check_lower(
        "sigmoid midpoint ≥ low",
        at_midpoint,
        tolerances::SEROTONIN_MIDPOINT_LOW,
    );
    h.check_upper(
        "sigmoid midpoint ≤ high",
        at_midpoint,
        tolerances::SEROTONIN_MIDPOINT_HIGH,
    );

    let steps: Vec<f64> = (0..20).map(|i| f64::from(i) * 0.15).collect();
    let trps: Vec<f64> = steps
        .iter()
        .map(|&div| microbiome::tryptophan_availability(200.0, div))
        .collect();
    let monotone = trps
        .windows(2)
        .all(|w| w[1] >= w[0] - tolerances::MACHINE_EPSILON);
    h.check_bool("tryptophan monotone with diversity", monotone);

    let h_post_fmt = 2.1;
    let ser_post_fmt = microbiome::gut_serotonin_production(
        microbiome::tryptophan_availability(dietary_trp, h_post_fmt),
        h_post_fmt,
        k_synth,
        scale,
    );
    h.check_bool(
        "FMT recovery: post-FMT 5HT > cdiff",
        ser_post_fmt > ser_cdiff,
    );

    let ser_zero = microbiome::gut_serotonin_production(0.0, h_healthy, k_synth, scale);
    h.check_abs(
        "zero trp → zero 5HT",
        ser_zero,
        0.0,
        tolerances::MACHINE_EPSILON,
    );

    h.exit();
}
