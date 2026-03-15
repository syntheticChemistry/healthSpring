// SPDX-License-Identifier: AGPL-3.0-or-later
//! Comparative medicine / One Health — species-agnostic mathematics.
//!
//! Track 6 in the healthSpring paper queue. The causal inversion principle:
//! study disease where it naturally occurs (dogs get atopic dermatitis, cats
//! get hyperthyroidism), then translate to humans via parameter substitution.
//! The math is species-invariant; only parameters change.
//!
//! ## Modules
//!
//! | Module | Domain |
//! |--------|--------|
//! | [`species_params`] | Species PK parameter registry + allometric scaling |
//! | [`canine`] | Canine-specific pharmacology: IL-31, JAK1, lokivetmab |

pub mod canine;
pub mod feline;
pub mod species_params;

pub use canine::{
    canine_jak_ic50_panel, il31_serum_kinetics, lokivetmab_effective_duration, lokivetmab_onset_hr,
    lokivetmab_pk, pruritus_time_course, pruritus_vas_response, CanineIl31Treatment, JakIc50Panel,
};
pub use feline::{
    methimazole_apparent_half_life, methimazole_css, methimazole_simulate, t4_response,
    FelineMethimazoleParams, FELINE_METHIMAZOLE, HUMAN_METHIMAZOLE,
};
pub use species_params::{
    allometric_clearance, allometric_half_life, allometric_volume, scale_across_species, Species,
    SpeciesPkParams,
};
