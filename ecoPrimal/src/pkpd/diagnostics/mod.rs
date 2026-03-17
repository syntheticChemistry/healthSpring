// SPDX-License-Identifier: AGPL-3.0-or-later

//! Population PK diagnostics — CWRES, VPC, and GOF (Goodness-of-Fit).
//!
//! Standard population PK model evaluation plots as described in FDA
//! guidance and Hooker et al. (2007). All diagnostic data is returned
//! as structured types ready for `petalTongue` `DataChannel` rendering.
//!
//! ## Diagnostics
//!
//! - **CWRES** ([`compute_cwres`]): Conditional Weighted Residuals — residuals
//!   standardised by the FOCE conditional variance approximation.
//! - **VPC** ([`compute_vpc`]): Visual Predictive Check — simulation-based model
//!   evaluation comparing observed data quantiles against model-predicted
//!   quantiles.
//! - **GOF** ([`compute_gof`]): Goodness-of-Fit — observed vs predicted, residual
//!   vs time, and QQ-normal plots.

mod cwres;
mod gof;
mod vpc;

// Re-export all public types and functions for backward compatibility.
pub use cwres::{CwresSummary, SubjectCwres, compute_cwres, cwres_summary};
pub use gof::{GofResult, compute_gof};
pub use vpc::{VpcConfig, VpcResult, compute_vpc};
