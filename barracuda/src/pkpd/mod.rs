// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pharmacokinetic / pharmacodynamic modeling pipelines.
//!
//! Extends neuralSpring nS-601–605 (veterinary PK/PD) to human therapeutics.
//!
//! ## Tier 1 (CPU)
//!
//! - [`hill_dose_response`]: Generalized Hill equation for dose-response
//! - [`pk_iv_bolus`]: One-compartment IV bolus decay
//! - [`pk_oral_one_compartment`]: Bateman equation for oral absorption
//! - [`auc_trapezoidal`]: AUC by trapezoidal rule
//! - [`find_cmax_tmax`]: Peak concentration and time from PK curve
//! - [`pk_multiple_dose`]: Superposition for repeated dosing
//! - [`compute_ec_values`]: EC10/EC50/EC90 from Hill parameters
//! - [`nca_iv`]: Non-Compartmental Analysis (`lambda_z`, AUC extrapolation, MRT, Vss, `CLss`)
//! - [`foce`]: FOCE population PK estimation (NONMEM replacement)
//! - [`saem`]: SAEM population PK estimation (Monolix replacement)
//!
//! ## Human JAK Inhibitor Reference Data
//!
//! Published IC50 values from Phase III trial literature:
//! - Baricitinib (Olumiant): JAK1/JAK2, IC50 ≈ 5.9 nM
//! - Upadacitinib (Rinvoq): JAK1, IC50 ≈ 8 nM
//! - Abrocitinib (Cibinqo): JAK1, IC50 ≈ 29 nM
//! - Oclacitinib (Apoquel): JAK1, IC50 = 10 nM (canine reference, Gonzales 2014)

mod allometry;
mod compartment;
pub mod diagnostics;
mod dose_response;
pub mod nca;
pub mod nlme;
pub mod nonlinear;
mod pbpk;
mod population;
mod util;

// Re-export submodules for public API
pub use allometry::{allometric_exp, lokivetmab_canine};
pub use allometry::{allometric_scale, mab_pk_sc};
pub use compartment::{
    micro_to_macro, oral_tmax, pk_iv_bolus, pk_oral_one_compartment, pk_two_compartment_iv,
    two_compartment_ab,
};
pub use diagnostics::{
    CwresSummary, GofResult, SubjectCwres, VpcConfig, VpcResult, compute_cwres, compute_gof,
    compute_vpc, cwres_summary,
};
pub use dose_response::{
    ABROCITINIB, ALL_INHIBITORS, BARICITINIB, EcValues, JakInhibitor, OCLACITINIB, UPADACITINIB,
    compute_ec_values, hill_dose_response, hill_sweep,
};
pub use nca::{NcaResult, aumc_trapezoidal, nca_iv};
pub use nlme::{
    NlmeConfig, NlmeResult, Subject, SyntheticPopConfig, foce, generate_synthetic_population,
    iv_one_compartment_model, oral_one_compartment_model, saem,
};
pub use nonlinear::{
    MichaelisMentenParams, PHENYTOIN_PARAMS, mm_apparent_half_life, mm_auc, mm_auc_analytical,
    mm_css_infusion, mm_nonlinearity_ratio, mm_pk_simulate,
};
pub use pbpk::{
    PbpkState, PbpkTissueProfiles, TissueCompartment, cardiac_output, pbpk_auc, pbpk_iv_simulate,
    pbpk_iv_tissue_profiles, standard_human_tissues,
};
pub use population::{LognormalParam, PatientExposure, pop_baricitinib, population_pk_cpu};
pub use util::{auc_trapezoidal, find_cmax_tmax, pk_multiple_dose};
