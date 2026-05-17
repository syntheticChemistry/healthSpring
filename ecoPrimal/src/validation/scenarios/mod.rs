// SPDX-License-Identifier: AGPL-3.0-or-later

//! Validation scenarios — structured test harness for healthSpring.
//!
//! Each scenario is a pure function `run(v: &mut ValidationResult, ctx: &mut CompositionContext)`
//! with `ScenarioMeta` metadata (id, track, tier, provenance, description).

mod registry;

// Scenario modules (one per absorbed experiment)
mod s_anderson_gut;
mod s_antibiotic_perturbation;
mod s_barracuda_parity;
mod s_beat_classification;
mod s_biosignal_fusion;
mod s_canine_gut;
mod s_canine_il31;
mod s_canine_jak1;
mod s_cardiac_risk;
mod s_causal_simulation;
mod s_colonization_resistance;
mod s_composition_parity;
mod s_compound_library;
mod s_cross_species_pk;
mod s_diabetes_trt;
mod s_diversity_indices;
mod s_eda_stress;
mod s_feline_methimazole;
mod s_fibrosis_pathway;
mod s_fmt_blend;
mod s_gut_axis;
mod s_hill_dose_response;
mod s_hormesis;
mod s_hrv_metrics;
mod s_hrv_trt;
mod s_hts_analysis;
mod s_jak_panel;
mod s_live_health;
mod s_live_provenance;
mod s_lokivetmab;
mod s_mab_pk;
mod s_matrix_scoring;
mod s_michaelis_menten;
mod s_nest_atomic;
mod s_nucleus_parity;
mod s_one_compartment_pk;
mod s_pan_tompkins_qrs;
mod s_pbpk;
mod s_pellet_pk;
mod s_pop_trt;
mod s_population_pk;
mod s_ppg_spo2;
mod s_pruritus;
mod s_scfa_serotonin;
mod s_testosterone_decline;
mod s_testosterone_pk;
mod s_tox_landscape;
mod s_toxicology;
mod s_trt_outcomes;
mod s_two_compartment_pk;

pub use registry::{Scenario, Tier, Track, build_registry};
