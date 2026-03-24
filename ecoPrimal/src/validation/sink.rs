// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pluggable sinks for emitting validation check output.

use tracing::{error, info};

use super::check::Check;

/// Output sink for validation results.
///
/// Absorbed from wetSpring V132 — allows composable validation output
/// for CI (stdout), testing (silent/collecting), and streaming.
pub trait ValidationSink {
    /// Emit a single check result.
    fn emit(&mut self, check: &Check);
    /// Emit a summary line.
    fn summary(&mut self, name: &str, passed: usize, failed: usize, total: usize);
}

/// Default sink: emits check lines via the `tracing` subscriber (typically stdout).
pub struct TracingSink;

impl ValidationSink for TracingSink {
    fn emit(&mut self, check: &Check) {
        if check.passed {
            info!("{check}");
        } else {
            error!("{check}");
        }
    }

    fn summary(&mut self, name: &str, passed: usize, failed: usize, total: usize) {
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
}

/// Silent sink: discards all harness output (library/unit tests).
pub struct SilentSink;

impl ValidationSink for SilentSink {
    fn emit(&mut self, _check: &Check) {}
    fn summary(&mut self, _name: &str, _passed: usize, _failed: usize, _total: usize) {}
}

/// Collecting sink: records formatted checks and summaries for assertions.
#[derive(Default)]
pub struct CollectingSink {
    /// All emitted checks.
    pub collected: Vec<String>,
}

impl ValidationSink for CollectingSink {
    fn emit(&mut self, check: &Check) {
        self.collected.push(format!("{check}"));
    }

    fn summary(&mut self, name: &str, passed: usize, failed: usize, total: usize) {
        self.collected
            .push(format!("{name}: {passed}/{total} passed, {failed} failed"));
    }
}
