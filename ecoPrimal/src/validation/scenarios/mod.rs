// SPDX-License-Identifier: AGPL-3.0-or-later

//! Validation scenarios — structured test harness for healthSpring.
//!
//! Each scenario is a pure function `run(v: &mut ValidationResult, ctx: &mut CompositionContext)`
//! with `ScenarioMeta` metadata (id, track, tier, provenance, description).

mod registry;

// Scenario modules (one per absorbed experiment)
mod s_anderson_gut;
mod s_barracuda_parity;
mod s_canine_il31;
mod s_composition_parity;
mod s_diversity_indices;
mod s_hill_dose_response;
mod s_hrv_metrics;
mod s_live_health;
mod s_live_provenance;
mod s_matrix_scoring;
mod s_michaelis_menten;
mod s_nucleus_parity;
mod s_one_compartment_pk;
mod s_pan_tompkins_qrs;
mod s_population_pk;
mod s_testosterone_pk;
mod s_toxicology;

pub use registry::{Scenario, Tier, Track, build_registry};
