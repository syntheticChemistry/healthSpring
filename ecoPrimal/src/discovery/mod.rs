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

pub mod compound;
pub mod fibrosis;
pub mod hts;
pub mod matrix_score;

pub use compound::{
    batch_ic50_sweep, estimate_ic50, rank_by_selectivity, selectivity_index, CompoundProfile,
    CompoundScorecard, Ic50Estimate, TargetProfile,
};
pub use fibrosis::{
    ccg_1423, ccg_203971, fibrosis_matrix_score, fibrotic_geometry_factor, fractional_inhibition,
    score_anti_fibrotic, AntiFibroticCompound, FibrosisPathwayScore,
};
pub use hts::{classify_hits, percent_inhibition, ssmd, z_prime_factor, HitClass, HitResult};
pub use matrix_score::{
    disorder_impact_factor, matrix_combined_score, pathway_selectivity_score, score_compound,
    score_panel, tissue_geometry_factor, MatrixEntry, TissueContext,
};
