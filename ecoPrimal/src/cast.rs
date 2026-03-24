// SPDX-License-Identifier: AGPL-3.0-or-later
//! Safe numeric cast helpers — absorbed from groundSpring V112/V122.
//!
//! Centralizes `as` casts that would otherwise scatter `#[expect]` attributes
//! across the codebase. Each function documents its precision guarantee.
//!
//! Two flavours exist:
//! - This module: **infallible** `const fn` casts for hot paths where the
//!   caller's domain guarantees safety (array lengths, GPU workgroup sizes).
//! - [`crate::safe_cast`]: **fallible** `Result`-returning casts for untrusted
//!   or boundary values.

// ── usize → numeric ─────────────────────────────────────────────────

/// `usize` → `f64`, exact for lengths up to 2^53.
#[inline]
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "exact for lengths up to 2^53")]
pub const fn usize_f64(n: usize) -> f64 {
    n as f64
}

/// `usize` → `u32`, saturating at `u32::MAX`.
#[inline]
#[must_use]
pub fn usize_u32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// `usize` → `u64` (lossless on 64-bit, widening on 32-bit).
#[inline]
#[must_use]
pub const fn usize_u64(n: usize) -> u64 {
    n as u64
}

// ── u64 → numeric ───────────────────────────────────────────────────

/// `u64` → `f64`, exact for values up to 2^53.
#[inline]
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "exact for values up to 2^53")]
pub const fn u64_f64(n: u64) -> f64 {
    n as f64
}

/// `u64` → `u32` via truncation (keeps low 32 bits).
/// Primary use: PRNG seed truncation for GPU dispatch (`u64` seed → `u32` workgroup param).
#[inline]
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "intentional seed truncation"
)]
pub const fn u64_u32_truncate(n: u64) -> u32 {
    n as u32
}

// ── u32 → numeric ───────────────────────────────────────────────────

/// `u32` → `f64` (lossless — all u32 values fit in f64).
#[inline]
#[must_use]
pub const fn u32_f64(n: u32) -> f64 {
    n as f64
}

/// `u32` → `usize` (lossless on 32- and 64-bit targets).
#[inline]
#[must_use]
pub const fn u32_usize(n: u32) -> usize {
    n as usize
}

// ── i32 / i16 → numeric ────────────────────────────────────────────

/// `i32` → `f64` (lossless — all i32 values fit in f64).
#[inline]
#[must_use]
pub const fn i32_f64(n: i32) -> f64 {
    n as f64
}

/// `i16` → `f64` (lossless — all i16 values fit in f64).
#[inline]
#[must_use]
pub const fn i16_f64(n: i16) -> f64 {
    n as f64
}

// ── f64 → integer ───────────────────────────────────────────────────

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

/// `f64` → `u32` via truncation. Caller must ensure `0.0 <= x <= u32::MAX`.
#[inline]
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "callers ensure x is non-negative and within u32 range"
)]
pub const fn f64_u32(x: f64) -> u32 {
    x as u32
}

/// `f64` → `u64` via truncation. Caller must ensure `0.0 <= x <= 2^53`.
#[inline]
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "callers ensure x is non-negative and within safe range"
)]
pub const fn f64_u64(x: f64) -> u64 {
    x as u64
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
    fn u64_u32_truncate_low_bits() {
        assert_eq!(u64_u32_truncate(42), 42);
        assert_eq!(u64_u32_truncate(0xFFFF_FFFF), u32::MAX);
        assert_eq!(u64_u32_truncate(0x1_0000_0000), 0);
    }

    #[test]
    fn u32_f64_exact() {
        assert!((u32_f64(u32::MAX) - f64::from(u32::MAX)).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn u32_usize_identity() {
        assert_eq!(u32_usize(0), 0);
        assert_eq!(u32_usize(u32::MAX), u32::MAX as usize);
    }

    #[test]
    fn i32_f64_exact() {
        assert!((i32_f64(-100) - (-100.0)).abs() < tolerances::MACHINE_EPSILON);
    }

    #[test]
    fn i16_f64_exact() {
        assert!((i16_f64(i16::MIN) - f64::from(i16::MIN)).abs() < tolerances::MACHINE_EPSILON);
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
    fn f64_u32_truncates() {
        assert_eq!(f64_u32(3.9), 3);
        assert_eq!(f64_u32(0.0), 0);
    }

    #[test]
    fn f64_u64_truncates() {
        assert_eq!(f64_u64(100.9), 100);
    }

    #[test]
    fn usize_u32_saturates() {
        assert_eq!(usize_u32(100), 100);
        assert_eq!(usize_u32(usize::MAX), u32::MAX);
    }

    #[test]
    fn usize_u64_widening() {
        assert_eq!(usize_u64(42), 42_u64);
    }
}
