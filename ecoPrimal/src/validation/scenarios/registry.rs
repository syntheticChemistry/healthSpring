// SPDX-License-Identifier: AGPL-3.0-or-later

//! Scenario registry — track taxonomy, tier classification, and scenario collection.
#![allow(
    missing_docs,
    reason = "ScenarioMeta taxonomy uses terse domain labels"
)]

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

/// Health-domain track taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Track {
    PkPd,
    Microbiome,
    Biosignal,
    Endocrine,
    Comparative,
    Discovery,
    Composition,
    Toxicology,
}

/// Validation tier (when the scenario can run).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// Pure Rust structural — no IPC needed.
    Rust,
    /// Requires live deployed primals.
    Live,
    /// Has both Rust-only and Live phases.
    Both,
}

/// Metadata for a single validation scenario.
pub struct ScenarioMeta {
    pub id: &'static str,
    pub track: Track,
    pub tier: Tier,
    pub source_experiment: &'static str,
    pub description: &'static str,
}

/// A validation scenario: metadata + run function.
pub struct Scenario {
    pub meta: ScenarioMeta,
    pub run: fn(&mut ValidationResult, &mut CompositionContext),
}

/// Build the full scenario registry.
#[must_use]
pub fn build_registry() -> Vec<Scenario> {
    vec![
        // ── PK/PD (Track 1) ─────────────────────────────────────────
        super::s_hill_dose_response::SCENARIO(),
        super::s_one_compartment_pk::SCENARIO(),
        super::s_two_compartment_pk::SCENARIO(),
        super::s_mab_pk::SCENARIO(),
        super::s_population_pk::SCENARIO(),
        super::s_pbpk::SCENARIO(),
        super::s_michaelis_menten::SCENARIO(),
        // ── Microbiome (Track 2) ────────────────────────────────────
        super::s_diversity_indices::SCENARIO(),
        super::s_anderson_gut::SCENARIO(),
        super::s_fmt_blend::SCENARIO(),
        super::s_colonization_resistance::SCENARIO(),
        super::s_antibiotic_perturbation::SCENARIO(),
        super::s_scfa_serotonin::SCENARIO(),
        super::s_gut_brain_serotonin::SCENARIO(),
        super::s_qs_anderson::SCENARIO(),
        super::s_real_16s::SCENARIO(),
        // ── Biosignal (Track 3) ─────────────────────────────────────
        super::s_pan_tompkins_qrs::SCENARIO(),
        super::s_hrv_metrics::SCENARIO(),
        super::s_ppg_spo2::SCENARIO(),
        super::s_biosignal_fusion::SCENARIO(),
        super::s_eda_stress::SCENARIO(),
        super::s_beat_classification::SCENARIO(),
        super::s_mitbih_arrhythmia::SCENARIO(),
        // ── Endocrine (Track 4) ─────────────────────────────────────
        super::s_testosterone_pk::SCENARIO(),
        super::s_pellet_pk::SCENARIO(),
        super::s_testosterone_decline::SCENARIO(),
        super::s_trt_outcomes::SCENARIO(),
        super::s_cardiac_risk::SCENARIO(),
        super::s_diabetes_trt::SCENARIO(),
        super::s_pop_trt::SCENARIO(),
        super::s_gut_axis::SCENARIO(),
        super::s_hrv_trt::SCENARIO(),
        // ── Discovery (Track 7) ─────────────────────────────────────
        super::s_matrix_scoring::SCENARIO(),
        super::s_hts_analysis::SCENARIO(),
        super::s_compound_library::SCENARIO(),
        super::s_jak_panel::SCENARIO(),
        super::s_fibrosis_pathway::SCENARIO(),
        super::s_ipsc_skin::SCENARIO(),
        super::s_niclosamide::SCENARIO(),
        super::s_causal_simulation::SCENARIO(),
        // ── Toxicology ──────────────────────────────────────────────
        super::s_toxicology::SCENARIO(),
        super::s_tox_landscape::SCENARIO(),
        super::s_hormesis::SCENARIO(),
        // ── Comparative (Track 6) ───────────────────────────────────
        super::s_canine_il31::SCENARIO(),
        super::s_canine_jak1::SCENARIO(),
        super::s_pruritus::SCENARIO(),
        super::s_lokivetmab::SCENARIO(),
        super::s_cross_species_pk::SCENARIO(),
        super::s_canine_gut::SCENARIO(),
        super::s_feline_methimazole::SCENARIO(),
        super::s_equine_laminitis::SCENARIO(),
        // ── Composition / Infrastructure ────────────────────────────
        super::s_composition_parity::SCENARIO(),
        super::s_live_provenance::SCENARIO(),
        super::s_live_health::SCENARIO(),
        super::s_barracuda_parity::SCENARIO(),
        super::s_nucleus_parity::SCENARIO(),
        super::s_nest_atomic::SCENARIO(),
    ]
}
