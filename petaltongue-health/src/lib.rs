// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

//! healthSpring diagnostic UI prototype for petalTongue evolution.
//!
//! This crate demonstrates what petalTongue needs to absorb:
//! - Time-series chart rendering (PK curves, tachograms)
//! - Distribution histograms (population Monte Carlo)
//! - Bar charts (genus abundances)
//! - Gauge widgets (clinical metrics with reference ranges)
//! - Interactive node detail panels
//! - Domain-specific clinical formatting

pub mod render;
pub mod theme;
