// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared validation harness for experiment binaries.
//!
//! Follows the hotSpring pattern: each experiment binary creates a
//! [`ValidationHarness`], registers checks via `check_abs`, `check_rel`,
//! etc., and exits 0 (all pass) or 1 (any fail).
//!
//! This replaces ad-hoc `passed`/`failed` counters across 61 experiments
//! with a single, auditable pattern linked to `TOLERANCE_REGISTRY.md`.

use core::fmt;

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
    pub label: String,
    pub passed: bool,
    pub observed: f64,
    pub expected: f64,
    pub tolerance: f64,
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
}

impl ValidationHarness {
    #[must_use]
    pub fn new(name: &str) -> Self {
        println!("=== {name} ===");
        Self {
            name: name.into(),
            checks: Vec::new(),
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

    /// Relative tolerance check: `|observed - expected| / |expected| ≤ tol`.
    /// Falls back to absolute if `|expected| < 1e-15`.
    pub fn check_rel(&mut self, label: &str, observed: f64, expected: f64, tol: f64) {
        let passed = if expected.abs() < 1e-15 {
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
        println!("{check}");
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

    /// Print summary and exit process with 0 (all pass) or 1 (any fail).
    pub fn exit(&self) -> ! {
        let total = self.checks.len();
        let passed = self.passed();
        let failed = self.failed();
        println!(
            "\n--- {} summary: {passed}/{total} passed, {failed} failed ---",
            self.name,
        );
        std::process::exit(i32::from(failed > 0));
    }

    /// Return whether all checks passed (for non-binary use).
    #[must_use]
    pub fn all_passed(&self) -> bool {
        self.checks.iter().all(|c| c.passed)
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

/// Root Mean Square Error between observed and predicted.
///
/// # Panics
///
/// Panics if `observed` and `predicted` have different lengths.
#[expect(clippy::cast_precision_loss, reason = "n ≪ 2^52")]
#[must_use]
pub fn rmse(observed: &[f64], predicted: &[f64]) -> f64 {
    assert_eq!(observed.len(), predicted.len());
    let n = observed.len();
    if n == 0 {
        return 0.0;
    }
    let sum_sq: f64 = observed
        .iter()
        .zip(predicted)
        .map(|(o, p)| (o - p).powi(2))
        .sum();
    (sum_sq / n as f64).sqrt()
}

/// Mean Absolute Error.
///
/// # Panics
///
/// Panics if `observed` and `predicted` have different lengths.
#[expect(clippy::cast_precision_loss, reason = "n ≪ 2^52")]
#[must_use]
pub fn mae(observed: &[f64], predicted: &[f64]) -> f64 {
    assert_eq!(observed.len(), predicted.len());
    let n = observed.len();
    if n == 0 {
        return 0.0;
    }
    let sum_abs: f64 = observed
        .iter()
        .zip(predicted)
        .map(|(o, p)| (o - p).abs())
        .sum();
    sum_abs / n as f64
}

/// Nash-Sutcliffe Efficiency (1.0 = perfect, 0.0 = mean model, negative = worse).
///
/// # Panics
///
/// Panics if `observed` and `predicted` have different lengths.
#[expect(clippy::cast_precision_loss, reason = "n ≪ 2^52")]
#[must_use]
pub fn nse(observed: &[f64], predicted: &[f64]) -> f64 {
    assert_eq!(observed.len(), predicted.len());
    let n = observed.len();
    if n == 0 {
        return 0.0;
    }
    let mean_obs: f64 = observed.iter().sum::<f64>() / n as f64;
    let ss_res: f64 = observed
        .iter()
        .zip(predicted)
        .map(|(o, p)| (o - p).powi(2))
        .sum();
    let ss_tot: f64 = observed.iter().map(|o| (o - mean_obs).powi(2)).sum();
    if ss_tot < 1e-15 {
        return 1.0;
    }
    1.0 - ss_res / ss_tot
}

/// Coefficient of determination (R-squared).
#[must_use]
pub fn r_squared(observed: &[f64], predicted: &[f64]) -> f64 {
    nse(observed, predicted)
}

/// Willmott Index of Agreement (0.0 to 1.0).
///
/// # Panics
///
/// Panics if `observed` and `predicted` have different lengths.
#[expect(clippy::cast_precision_loss, reason = "n ≪ 2^52")]
#[must_use]
pub fn index_of_agreement(observed: &[f64], predicted: &[f64]) -> f64 {
    assert_eq!(observed.len(), predicted.len());
    let n = observed.len();
    if n == 0 {
        return 0.0;
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
    if ss_pot < 1e-15 {
        return 1.0;
    }
    1.0 - ss_res / ss_pot
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(rmse(&data, &data) < 1e-15);
    }

    #[test]
    fn mae_perfect() {
        let data = [1.0, 2.0, 3.0];
        assert!(mae(&data, &data) < 1e-15);
    }

    #[test]
    fn nse_perfect() {
        let data = [1.0, 2.0, 3.0];
        assert!((nse(&data, &data) - 1.0).abs() < 1e-15);
    }

    #[test]
    fn index_of_agreement_perfect() {
        let data = [1.0, 2.0, 3.0];
        assert!((index_of_agreement(&data, &data) - 1.0).abs() < 1e-15);
    }
}
