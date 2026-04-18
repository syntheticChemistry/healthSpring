// SPDX-License-Identifier: AGPL-3.0-or-later
//! [`ValidationHarness`] for registering checks and exiting with pass/fail codes.

use std::sync::Once;

use tracing::info;

use crate::tolerances;

use super::check::{Check, ToleranceMode};
use super::sink::ValidationSink;

static TRACING_INIT: Once = Once::new();

/// Initialize a minimal tracing subscriber for validation binary output.
///
/// Called automatically by [`ValidationHarness::new`]. Safe to call multiple
/// times — only the first invocation installs a subscriber.
fn init_validation_tracing() {
    TRACING_INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .without_time()
            .with_target(false)
            .with_level(false)
            .init();
    });
}

/// Accumulates validation checks and produces a pass/fail exit code.
///
/// ```rust,no_run
/// use healthspring_barracuda::validation::ValidationHarness;
///
/// let mut h = ValidationHarness::new("exp001_hill_dose_response");
/// h.check_abs("Hill at IC50", 0.500_000_000_1, 0.5, 1e-10);
/// h.check_bool("Monotonic", true);
/// h.exit(); // exits process with 0 or 1
/// ```
pub struct ValidationHarness {
    name: String,
    checks: Vec<Check>,
    sink: ValidationSink,
}

impl ValidationHarness {
    /// Create a harness with the default tracing sink.
    #[must_use]
    pub fn new(name: &str) -> Self {
        init_validation_tracing();
        info!(experiment = %name, "=== {name} ===");
        Self {
            name: name.into(),
            checks: Vec::new(),
            sink: ValidationSink::Tracing,
        }
    }

    /// Create a harness with a silent sink (no output).
    #[must_use]
    pub fn silent(name: &str) -> Self {
        Self {
            name: name.into(),
            checks: Vec::new(),
            sink: ValidationSink::Silent,
        }
    }

    /// Create a harness with a custom sink.
    #[must_use]
    pub fn with_sink(name: &str, sink: ValidationSink) -> Self {
        Self {
            name: name.into(),
            checks: Vec::new(),
            sink,
        }
    }

    /// Absolute tolerance check: `|observed - expected| ≤ tol`.
    pub fn check_abs(&mut self, label: &str, observed: f64, expected: f64, tol: f64) {
        let passed = (observed - expected).abs() <= tol;
        self.push(
            label,
            passed,
            observed,
            expected,
            tol,
            ToleranceMode::Absolute,
        );
    }

    /// Smart tolerance check: uses relative if `|expected| ≥ 1e-15`, absolute otherwise.
    ///
    /// Absorbed from groundSpring V120 / ludoSpring V29 — auto-selects the
    /// appropriate mode so callers don't need to reason about near-zero values.
    pub fn check_abs_or_rel(&mut self, label: &str, observed: f64, expected: f64, tol: f64) {
        if expected.abs() < tolerances::MACHINE_EPSILON_STRICT {
            self.check_abs(label, observed, expected, tol);
        } else {
            self.check_rel(label, observed, expected, tol);
        }
    }

    /// Relative tolerance check: `|observed - expected| / |expected| ≤ tol`.
    /// Falls back to absolute if `|expected| < 1e-15`.
    pub fn check_rel(&mut self, label: &str, observed: f64, expected: f64, tol: f64) {
        let passed = if expected.abs() < tolerances::MACHINE_EPSILON_STRICT {
            (observed - expected).abs() <= tol
        } else {
            ((observed - expected) / expected).abs() <= tol
        };
        self.push(
            label,
            passed,
            observed,
            expected,
            tol,
            ToleranceMode::Relative,
        );
    }

    /// Upper bound check: `observed ≤ bound`.
    pub fn check_upper(&mut self, label: &str, observed: f64, bound: f64) {
        let passed = observed <= bound;
        self.push(
            label,
            passed,
            observed,
            bound,
            0.0,
            ToleranceMode::UpperBound,
        );
    }

    /// Lower bound check: `observed ≥ bound`.
    pub fn check_lower(&mut self, label: &str, observed: f64, bound: f64) {
        let passed = observed >= bound;
        self.push(
            label,
            passed,
            observed,
            bound,
            0.0,
            ToleranceMode::LowerBound,
        );
    }

    /// Boolean check (pass/fail with no numeric comparison).
    pub fn check_bool(&mut self, label: &str, condition: bool) {
        self.push(
            label,
            condition,
            f64::from(u8::from(condition)),
            1.0,
            0.0,
            ToleranceMode::Exact,
        );
    }

    /// Exact equality for structural checks (node counts, etc.).
    #[expect(clippy::cast_precision_loss, reason = "structural counts fit f64")]
    pub fn check_exact(&mut self, label: &str, observed: u64, expected: u64) {
        let passed = observed == expected;
        self.push(
            label,
            passed,
            observed as f64,
            expected as f64,
            0.0,
            ToleranceMode::Exact,
        );
    }

    fn push(
        &mut self,
        label: &str,
        passed: bool,
        observed: f64,
        expected: f64,
        tolerance: f64,
        mode: ToleranceMode,
    ) {
        let check = Check {
            label: label.into(),
            passed,
            observed,
            expected,
            tolerance,
            mode,
        };
        self.sink.emit(&check);
        self.checks.push(check);
    }

    /// Number of passing checks.
    #[must_use]
    pub fn passed(&self) -> usize {
        self.checks.iter().filter(|c| c.passed).count()
    }

    /// Number of failing checks.
    #[must_use]
    pub fn failed(&self) -> usize {
        self.checks.iter().filter(|c| !c.passed).count()
    }

    /// Log summary and exit process with 0 (all pass) or 1 (any fail).
    pub fn exit(&mut self) -> ! {
        let total = self.checks.len();
        let passed = self.passed();
        let failed = self.failed();
        self.sink.summary(&self.name, passed, failed, total);
        std::process::exit(i32::from(failed > 0));
    }

    /// Exit with code 2 to indicate a skipped experiment.
    ///
    /// Absorbed from ludoSpring V29 — distinguishes "skip" (exit 2) from
    /// "fail" (exit 1) in CI, useful for GPU-absent or conditional runs.
    pub fn exit_skipped(reason: &str) -> ! {
        init_validation_tracing();
        info!(reason, "SKIP: {reason}");
        std::process::exit(2)
    }

    /// Return whether all checks passed (for non-binary use).
    #[must_use]
    pub fn all_passed(&self) -> bool {
        self.checks.iter().all(|c| c.passed)
    }
}

#[cfg(test)]
mod tests {
    use super::super::sink::{CollectingSink, ValidationSink};
    use super::ValidationHarness;

    #[test]
    fn abs_check_pass() {
        let mut h = ValidationHarness::new("test");
        h.check_abs("exact", 0.5, 0.5, 1e-10);
        assert!(h.all_passed());
    }

    #[test]
    fn abs_check_fail() {
        let mut h = ValidationHarness::new("test");
        h.check_abs("miss", 0.6, 0.5, 1e-10);
        assert_eq!(h.failed(), 1);
    }

    #[test]
    fn rel_check_pass() {
        let mut h = ValidationHarness::new("test");
        h.check_rel("close", 1.005, 1.0, 0.01);
        assert!(h.all_passed());
    }

    #[test]
    fn upper_bound() {
        let mut h = ValidationHarness::new("test");
        h.check_upper("within", 50.0, 100.0);
        assert!(h.all_passed());
        h.check_upper("over", 150.0, 100.0);
        assert_eq!(h.failed(), 1);
    }

    #[test]
    fn lower_bound() {
        let mut h = ValidationHarness::new("test");
        h.check_lower("above", 0.9, 0.8);
        assert!(h.all_passed());
    }

    #[test]
    fn bool_check() {
        let mut h = ValidationHarness::new("test");
        h.check_bool("true_cond", true);
        h.check_bool("false_cond", false);
        assert_eq!(h.passed(), 1);
        assert_eq!(h.failed(), 1);
    }

    #[test]
    fn exact_check() {
        let mut h = ValidationHarness::new("test");
        h.check_exact("match", 28, 28);
        h.check_exact("mismatch", 27, 28);
        assert_eq!(h.passed(), 1);
        assert_eq!(h.failed(), 1);
    }

    #[test]
    fn abs_or_rel_near_zero_uses_abs() {
        let mut h = ValidationHarness::silent("test");
        h.check_abs_or_rel("near zero", 1e-16, 0.0, 1e-10);
        assert!(h.all_passed());
    }

    #[test]
    fn abs_or_rel_nonzero_uses_rel() {
        let mut h = ValidationHarness::silent("test");
        h.check_abs_or_rel("relative", 1.005, 1.0, 0.01);
        assert!(h.all_passed());
    }

    #[test]
    fn silent_harness_no_output() {
        let mut h = ValidationHarness::silent("test");
        h.check_abs("pass", 1.0, 1.0, 1e-10);
        assert!(h.all_passed());
    }

    #[test]
    fn collecting_sink() {
        let sink = ValidationSink::Collecting(CollectingSink::default());
        let mut h = ValidationHarness::with_sink("test", sink);
        h.check_abs("check1", 1.0, 1.0, 1e-10);
        assert!(h.all_passed());
    }
}
