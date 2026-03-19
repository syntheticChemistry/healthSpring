// SPDX-License-Identifier: AGPL-3.0-or-later
//! Drug discovery pipeline: MATRIX scoring, HTS analysis, compound IC50 profiling.
//!
//! Track 7 in the healthSpring paper queue. The pipeline:
//! ```text
//! compound_library → batch_ic50_sweep → matrix_score → HTS validation → iPSC readout
//! ```
//!
//! ## Anderson-Augmented MATRIX
//!
//! Fajgenbaum's MATRIX framework (2018 NEJM, 2019 JCI) scores drug-disease pairs
//! via pathway overlap. healthSpring augments with Anderson tissue geometry: a drug's
//! tissue penetration depends on whether it creates extended or localized states in
//! the tissue lattice. Combined scoring reranks candidates in physically meaningful
//! ways that pathway analysis alone cannot capture.

pub mod affinity_landscape;
pub mod compound;
pub mod fibrosis;
pub mod hts;
pub mod matrix_score;

pub use affinity_landscape::{
    AffinityDistribution, analyze_affinity_distribution, binding_profile, colonization_resistance,
    composite_binding_score, cross_reactivity_matrix, disorder_adhesion_profile,
    fractional_occupancy, low_affinity_selectivity,
};
pub use compound::{
    CompoundProfile, CompoundScorecard, Ic50Estimate, TargetProfile, batch_ic50_sweep,
    estimate_ic50, rank_by_selectivity, selectivity_index,
};
pub use fibrosis::{
    AntiFibroticCompound, FibrosisPathwayScore, ccg_1423, ccg_203971, fibrosis_matrix_score,
    fibrotic_geometry_factor, fractional_inhibition, score_anti_fibrotic,
};
pub use hts::{HitClass, HitResult, classify_hits, percent_inhibition, ssmd, z_prime_factor};
pub use matrix_score::{
    MatrixEntry, TissueContext, disorder_impact_factor, matrix_combined_score,
    pathway_selectivity_score, score_compound, score_panel, tissue_geometry_factor,
};
