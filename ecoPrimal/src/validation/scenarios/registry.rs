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
        super::s_hill_dose_response::SCENARIO(),
        super::s_one_compartment_pk::SCENARIO(),
        super::s_population_pk::SCENARIO(),
        super::s_michaelis_menten::SCENARIO(),
        super::s_diversity_indices::SCENARIO(),
        super::s_anderson_gut::SCENARIO(),
        super::s_pan_tompkins_qrs::SCENARIO(),
        super::s_hrv_metrics::SCENARIO(),
        super::s_testosterone_pk::SCENARIO(),
        super::s_canine_il31::SCENARIO(),
        super::s_matrix_scoring::SCENARIO(),
        super::s_composition_parity::SCENARIO(),
        super::s_live_provenance::SCENARIO(),
        super::s_live_health::SCENARIO(),
        super::s_barracuda_parity::SCENARIO(),
        super::s_nucleus_parity::SCENARIO(),
    ]
}
