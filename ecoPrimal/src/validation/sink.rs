// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pluggable sinks for emitting validation check output.
//!
//! Uses enum dispatch instead of `dyn` trait objects to satisfy the
//! stadial zero-dyn invariant.

use tracing::{error, info};

use super::check::Check;

/// Enum-dispatched validation output sink.
///
/// Absorbed from wetSpring V132 — allows composable validation output
/// for CI (stdout), testing (silent/collecting), and streaming.
/// Converted from `dyn ValidationSink` to enum dispatch for stadial
/// zero-dyn compliance.
pub enum ValidationSink {
    /// Emits check lines via the `tracing` subscriber (typically stdout).
    Tracing,
    /// Discards all harness output (library/unit tests).
    Silent,
    /// Records formatted checks and summaries for assertions.
    Collecting(CollectingSink),
}

impl ValidationSink {
    /// Emit a single check result.
    pub fn emit(&mut self, check: &Check) {
        match self {
            Self::Tracing => {
                if check.passed {
                    info!("{check}");
                } else {
                    error!("{check}");
                }
            }
            Self::Silent => {}
            Self::Collecting(sink) => sink.collected.push(format!("{check}")),
        }
    }

    /// Emit a summary line.
    pub fn summary(&mut self, name: &str, passed: usize, failed: usize, total: usize) {
        match self {
            Self::Tracing => {
                if failed > 0 {
                    error!(
                        experiment = %name, passed, failed, total,
                        "--- {name} summary: {passed}/{total} passed, {failed} failed ---",
                    );
                } else {
                    info!(
                        experiment = %name, passed, total,
                        "--- {name} summary: {passed}/{total} passed, 0 failed ---",
                    );
                }
            }
            Self::Silent => {}
            Self::Collecting(sink) => {
                sink.collected
                    .push(format!("{name}: {passed}/{total} passed, {failed} failed"));
            }
        }
    }
}

/// Collecting sink state: records formatted checks and summaries for assertions.
#[derive(Default)]
pub struct CollectingSink {
    /// All emitted checks.
    pub collected: Vec<String>,
}
