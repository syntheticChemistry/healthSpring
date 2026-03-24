// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared validation harness for experiment binaries.
//!
//! Follows the hotSpring pattern: each experiment binary creates a
//! [`ValidationHarness`], registers checks via `check_abs`, `check_rel`,
//! etc., and exits 0 (all pass) or 1 (any fail).
//!
//! This replaces ad-hoc `passed`/`failed` counters across 83 experiments
//! with a single, auditable pattern linked to `TOLERANCE_REGISTRY.md`.

mod check;
mod harness;
mod metrics;
mod or_exit;
mod sink;

pub use check::{Check, ToleranceMode};
pub use harness::ValidationHarness;
pub use metrics::{LengthMismatch, index_of_agreement, len_f64, mae, nse, r_squared, rmse};
pub use or_exit::OrExit;
pub use sink::{CollectingSink, SilentSink, TracingSink, ValidationSink};
