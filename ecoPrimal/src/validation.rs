// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared validation harness for experiment binaries.
//!
//! Follows the hotSpring pattern: each experiment binary creates a
//! [`ValidationHarness`], registers checks via `check_abs`, `check_rel`,
//! etc., and exits 0 (all pass) or 1 (any fail).
//!
//! This replaces ad-hoc `passed`/`failed` counters across 83 experiments
//! with a single, auditable pattern linked to `TOLERANCE_REGISTRY.md`.

use core::fmt;
use std::sync::Once;

use tracing::{error, info};

use crate::tolerances::MACHINE_EPSILON_STRICT;

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

/// How a tolerance bound is interpreted.
#[derive(Debug, Clone, Copy)]
pub enum ToleranceMode {
    /// `|observed - expected| ≤ tol`
    Absolute,
    /// `|observed - expected| / |expected| ≤ tol` (guards against expected ≈ 0)
    Relative,
    /// `observed ≤ bound`
    UpperBound,
    /// `observed ≥ bound`
    LowerBound,
    /// Exact integer/structural equality.
    Exact,
}

impl fmt::Display for ToleranceMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Absolute => write!(f, "abs"),
            Self::Relative => write!(f, "rel"),
            Self::UpperBound => write!(f, "≤"),
            Self::LowerBound => write!(f, "≥"),
            Self::Exact => write!(f, "=="),
        }
    }
}

/// A single validation check with result.
#[derive(Debug)]
pub struct Check {
    /// Human-readable check name.
    pub label: String,
    /// Whether the check succeeded.
    pub passed: bool,
    /// Observed numeric value (or structural encoding for bool/exact).
    pub observed: f64,
    /// Expected reference value (or bound for inequality modes).
    pub expected: f64,
    /// Tolerance or bound parameter for the chosen mode.
    pub tolerance: f64,
    /// How `tolerance` and `expected` are interpreted.
    pub mode: ToleranceMode,
}

impl fmt::Display for Check {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tag = if self.passed { "PASS" } else { "FAIL" };
        write!(
            f,
            "[{tag}] {}: observed={:.10}, expected={:.10}, tol={} ({})",
            self.label, self.observed, self.expected, self.tolerance, self.mode,
        )
    }
}

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
    sink: Box<dyn ValidationSink>,
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
            sink: Box::new(TracingSink),
        }
    }

    /// Create a harness with a silent sink (no output).
    #[must_use]
    pub fn silent(name: &str) -> Self {
        Self {
            name: name.into(),
            checks: Vec::new(),
            sink: Box::new(SilentSink),
        }
    }

    /// Create a harness with a custom sink.
    #[must_use]
    pub fn with_sink(name: &str, sink: Box<dyn ValidationSink>) -> Self {
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
        if expected.abs() < MACHINE_EPSILON_STRICT {
            self.check_abs(label, observed, expected, tol);
        } else {
            self.check_rel(label, observed, expected, tol);
        }
    }

    /// Relative tolerance check: `|observed - expected| / |expected| ≤ tol`.
    /// Falls back to absolute if `|expected| < 1e-15`.
    pub fn check_rel(&mut self, label: &str, observed: f64, expected: f64, tol: f64) {
        let passed = if expected.abs() < MACHINE_EPSILON_STRICT {
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

// ── OrExit trait (absorbed from wetSpring V123) ─────────────────────────

/// Panic-free error handling for validation/utility binaries.
///
/// Replaces `.expect()` / `.unwrap()` with structured `eprintln!` + `exit(1)`.
/// For `Result<T, E>` and `Option<T>` — every binary call site becomes:
///
/// ```rust,no_run
/// # use healthspring_barracuda::validation::OrExit;
/// # let path = std::path::Path::new("/tmp/test");
/// let data = std::fs::read_to_string(path).or_exit("read config");
/// ```
pub trait OrExit<T> {
    /// Unwrap or print context to stderr and `exit(1)`.
    fn or_exit(self, context: &str) -> T;
}

impl<T, E: std::fmt::Display> OrExit<T> for Result<T, E> {
    fn or_exit(self, context: &str) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                error!(context = %context, error = %e, "FATAL: {context}: {e}");
                std::process::exit(1)
            }
        }
    }
}

impl<T> OrExit<T> for Option<T> {
    fn or_exit(self, context: &str) -> T {
        self.unwrap_or_else(|| {
            error!(context = %context, "FATAL: {context}");
            std::process::exit(1)
        })
    }
}

/// Convert a `usize` length to `f64` with an explicit precision-loss acknowledgement.
///
/// Useful at IPC boundaries where collection lengths appear in arithmetic.
#[expect(clippy::cast_precision_loss, reason = "collection size ≪ 2^52")]
#[must_use]
#[inline]
pub const fn len_f64(n: usize) -> f64 {
    n as f64
}

/// Length mismatch error for metric functions.
#[derive(Debug, Clone, Copy)]
pub struct LengthMismatch {
    /// Length of the observed slice.
    pub observed: usize,
    /// Length of the predicted slice.
    pub predicted: usize,
}

impl fmt::Display for LengthMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "length mismatch: observed={}, predicted={}",
            self.observed, self.predicted
        )
    }
}

impl std::error::Error for LengthMismatch {}

/// Verify equal lengths, returning `Err` on mismatch.
const fn check_lengths(observed: &[f64], predicted: &[f64]) -> Result<usize, LengthMismatch> {
    if observed.len() != predicted.len() {
        return Err(LengthMismatch {
            observed: observed.len(),
            predicted: predicted.len(),
        });
    }
    Ok(observed.len())
}

/// Root Mean Square Error between observed and predicted.
///
/// # Errors
///
/// Returns [`LengthMismatch`] if `observed` and `predicted` differ in length.
#[expect(clippy::cast_precision_loss, reason = "n ≪ 2^52")]
pub fn rmse(observed: &[f64], predicted: &[f64]) -> Result<f64, LengthMismatch> {
    let n = check_lengths(observed, predicted)?;
    if n == 0 {
        return Ok(0.0);
    }
    let sum_sq: f64 = observed
        .iter()
        .zip(predicted)
        .map(|(o, p)| (o - p).powi(2))
        .sum();
    Ok((sum_sq / n as f64).sqrt())
}

/// Mean Absolute Error.
///
/// # Errors
///
/// Returns [`LengthMismatch`] if `observed` and `predicted` differ in length.
#[expect(clippy::cast_precision_loss, reason = "n ≪ 2^52")]
pub fn mae(observed: &[f64], predicted: &[f64]) -> Result<f64, LengthMismatch> {
    let n = check_lengths(observed, predicted)?;
    if n == 0 {
        return Ok(0.0);
    }
    let sum_abs: f64 = observed
        .iter()
        .zip(predicted)
        .map(|(o, p)| (o - p).abs())
        .sum();
    Ok(sum_abs / n as f64)
}

/// Nash-Sutcliffe Efficiency (1.0 = perfect, 0.0 = mean model, negative = worse).
///
/// # Errors
///
/// Returns [`LengthMismatch`] if `observed` and `predicted` differ in length.
#[expect(clippy::cast_precision_loss, reason = "n ≪ 2^52")]
pub fn nse(observed: &[f64], predicted: &[f64]) -> Result<f64, LengthMismatch> {
    let n = check_lengths(observed, predicted)?;
    if n == 0 {
        return Ok(0.0);
    }
    let mean_obs: f64 = observed.iter().sum::<f64>() / n as f64;
    let ss_res: f64 = observed
        .iter()
        .zip(predicted)
        .map(|(o, p)| (o - p).powi(2))
        .sum();
    let ss_tot: f64 = observed.iter().map(|o| (o - mean_obs).powi(2)).sum();
    if ss_tot < MACHINE_EPSILON_STRICT {
        return Ok(1.0);
    }
    Ok(1.0 - ss_res / ss_tot)
}

/// Coefficient of determination (R-squared).
///
/// # Errors
///
/// Returns [`LengthMismatch`] if `observed` and `predicted` differ in length.
pub fn r_squared(observed: &[f64], predicted: &[f64]) -> Result<f64, LengthMismatch> {
    nse(observed, predicted)
}

/// Willmott Index of Agreement (0.0 to 1.0).
///
/// # Errors
///
/// Returns [`LengthMismatch`] if `observed` and `predicted` differ in length.
#[expect(clippy::cast_precision_loss, reason = "n ≪ 2^52")]
pub fn index_of_agreement(observed: &[f64], predicted: &[f64]) -> Result<f64, LengthMismatch> {
    let n = check_lengths(observed, predicted)?;
    if n == 0 {
        return Ok(0.0);
    }
    let mean_obs: f64 = observed.iter().sum::<f64>() / n as f64;
    let ss_res: f64 = observed
        .iter()
        .zip(predicted)
        .map(|(o, p)| (o - p).powi(2))
        .sum();
    let ss_pot: f64 = observed
        .iter()
        .zip(predicted)
        .map(|(o, p)| ((p - mean_obs).abs() + (o - mean_obs).abs()).powi(2))
        .sum();
    if ss_pot < MACHINE_EPSILON_STRICT {
        return Ok(1.0);
    }
    Ok(1.0 - ss_res / ss_pot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tolerances;

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
    fn rmse_perfect() {
        let data = [1.0, 2.0, 3.0];
        assert!(rmse(&data, &data).expect("same length") < tolerances::DIVISION_GUARD);
    }

    #[test]
    fn mae_perfect() {
        let data = [1.0, 2.0, 3.0];
        assert!(mae(&data, &data).expect("same length") < tolerances::DIVISION_GUARD);
    }

    #[test]
    fn nse_perfect() {
        let data = [1.0, 2.0, 3.0];
        assert!((nse(&data, &data).expect("same length") - 1.0).abs() < tolerances::DIVISION_GUARD);
    }

    #[test]
    fn index_of_agreement_perfect() {
        let data = [1.0, 2.0, 3.0];
        assert!(
            (index_of_agreement(&data, &data).expect("same length") - 1.0).abs()
                < tolerances::DIVISION_GUARD
        );
    }

    #[test]
    fn rmse_length_mismatch() {
        assert!(rmse(&[1.0, 2.0], &[1.0]).is_err());
    }

    #[test]
    fn mae_length_mismatch() {
        assert!(mae(&[1.0, 2.0], &[1.0]).is_err());
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
        let sink = Box::new(CollectingSink::default());
        let mut h = ValidationHarness::with_sink("test", sink);
        h.check_abs("check1", 1.0, 1.0, 1e-10);
        assert!(h.all_passed());
    }

    #[test]
    fn or_exit_result_ok() {
        let r: Result<i32, &str> = Ok(42);
        assert_eq!(r.or_exit("should not fail"), 42);
    }

    #[test]
    fn or_exit_option_some() {
        let o: Option<i32> = Some(7);
        assert_eq!(o.or_exit("should not fail"), 7);
    }
}
