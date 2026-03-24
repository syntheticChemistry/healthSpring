// SPDX-License-Identifier: AGPL-3.0-or-later
//! Tolerance modes and the [`Check`] record for a single validation result.

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
