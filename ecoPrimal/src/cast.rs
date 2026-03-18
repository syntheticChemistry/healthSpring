// SPDX-License-Identifier: AGPL-3.0-or-later
//! Safe numeric cast helpers — absorbed from groundSpring V112.
//!
//! Centralizes `as` casts that would otherwise scatter `#[expect]` attributes
//! across the codebase. Each function documents its precision guarantee.

/// `usize` → `f64`, exact for lengths up to 2^53.
#[inline]
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "exact for lengths up to 2^53")]
pub const fn usize_f64(n: usize) -> f64 {
    n as f64
}

/// `u64` → `f64`, exact for values up to 2^53.
#[inline]
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "exact for values up to 2^53")]
pub const fn u64_f64(n: u64) -> f64 {
    n as f64
}

/// `f64` → `usize` via truncation. Caller must ensure `x >= 0.0`.
#[inline]
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "callers ensure x is non-negative and within usize range"
)]
pub const fn f64_usize(x: f64) -> usize {
    x as usize
}

/// `usize` → `u32`, saturating at `u32::MAX`.
#[inline]
#[must_use]
pub fn usize_u32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tolerances;

    #[test]
    fn usize_f64_exact() {
        assert!((usize_f64(1_000_000) - 1_000_000.0).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn u64_f64_exact() {
        assert!((u64_f64(42) - 42.0).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn f64_usize_truncates() {
        assert_eq!(f64_usize(3.7), 3);
    }

    #[test]
    fn f64_usize_zero() {
        assert_eq!(f64_usize(0.0), 0);
    }

    #[test]
    fn usize_u32_saturates() {
        assert_eq!(usize_u32(100), 100);
        assert_eq!(usize_u32(usize::MAX), u32::MAX);
    }
}
