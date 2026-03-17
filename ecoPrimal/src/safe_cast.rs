// SPDX-License-Identifier: AGPL-3.0-or-later
//! Safe numeric casts for GPU dispatch parameters.
//!
//! Replaces raw `as` casts with checked conversions that return `Result`.
//! Prevents silent truncation when converting between usize, u32, u64, f64.

/// Convert `usize` to `u32`, returning `Err` on overflow.
///
/// # Errors
///
/// Returns [`CastError`] if `n` exceeds `u32::MAX`.
#[inline]
pub fn usize_u32(n: usize) -> Result<u32, CastError> {
    u32::try_from(n).map_err(|_| CastError {
        from: "usize",
        to: "u32",
        value: n.to_string(),
    })
}

/// Convert `usize` to `u64`.
#[inline]
#[must_use]
pub const fn usize_u64(n: usize) -> u64 {
    n as u64
}

/// Maximum `usize` value exactly representable in f64 (2^53).
const USIZE_F64_MAX_EXACT: usize = 1 << 53;

/// Convert `usize` to `f64`, returning `Err` if precision would be lost (n > 2^53).
///
/// # Errors
///
/// Returns [`CastError`] if `n` exceeds 2^53 (f64 safe integer range).
#[inline]
pub fn usize_f64(n: usize) -> Result<f64, CastError> {
    if n <= USIZE_F64_MAX_EXACT {
        #[expect(clippy::cast_precision_loss, reason = "n <= 2^53 is exact")]
        Ok(n as f64)
    } else {
        Err(CastError {
            from: "usize",
            to: "f64",
            value: n.to_string(),
        })
    }
}

/// Convert `f64` to `f32`, returning `Err` if value overflows f32 range.
///
/// # Errors
///
/// Returns [`CastError`] if `v` is finite and outside the range of `f32`.
#[inline]
pub fn f64_f32(v: f64) -> Result<f32, CastError> {
    let min_f64 = f64::from(f32::MIN);
    let max_f64 = f64::from(f32::MAX);
    if v.is_finite() && (v < min_f64 || v > max_f64) {
        Err(CastError {
            from: "f64",
            to: "f32",
            value: v.to_string(),
        })
    } else {
        #[expect(clippy::cast_possible_truncation, reason = "range checked above")]
        Ok(v as f32)
    }
}

/// Error returned when a numeric cast would overflow or lose precision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CastError {
    /// Source type name.
    pub from: &'static str,
    /// Target type name.
    pub to: &'static str,
    /// String representation of the value that failed to convert.
    pub value: String,
}

impl std::fmt::Display for CastError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "cast error: {} ({}) cannot be safely converted to {}",
            self.from, self.value, self.to
        )
    }
}

impl std::error::Error for CastError {}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test code uses unwrap_err for error assertions"
)]
mod tests {
    use super::*;

    #[test]
    fn usize_u32_ok() {
        assert_eq!(usize_u32(0), Ok(0));
        assert_eq!(usize_u32(100), Ok(100));
        assert_eq!(usize_u32(u32::MAX as usize), Ok(u32::MAX));
    }

    #[test]
    fn usize_u32_overflow() {
        let err = usize_u32(u32::MAX as usize + 1).unwrap_err();
        assert_eq!(err.from, "usize");
        assert_eq!(err.to, "u32");
    }

    #[test]
    fn usize_u64_ok() {
        assert_eq!(usize_u64(0), 0);
        assert_eq!(usize_u64(usize::MAX), usize::MAX as u64);
    }

    #[test]
    fn usize_f64_exact() {
        assert_eq!(usize_f64(0), Ok(0.0));
        assert_eq!(usize_f64(1_000_000), Ok(1_000_000.0));
        #[expect(clippy::cast_precision_loss, reason = "exact power of 2 fits f64")]
        let expected = (1u64 << 53) as f64;
        assert_eq!(usize_f64(1 << 53), Ok(expected));
    }

    #[test]
    fn usize_f64_precision_lost() {
        let err = usize_f64((1 << 53) + 1).unwrap_err();
        assert_eq!(err.from, "usize");
        assert_eq!(err.to, "f64");
    }

    #[test]
    fn f64_f32_ok() {
        assert_eq!(f64_f32(0.0), Ok(0.0));
        assert_eq!(f64_f32(1.5), Ok(1.5));
        assert_eq!(f64_f32(f64::from(f32::MAX)), Ok(f32::MAX));
        assert_eq!(f64_f32(f64::from(f32::MIN)), Ok(f32::MIN));
    }

    #[test]
    fn f64_f32_overflow_positive() {
        let err = f64_f32(f64::from(f32::MAX) * 2.0).unwrap_err();
        assert_eq!(err.from, "f64");
        assert_eq!(err.to, "f32");
    }

    #[test]
    fn f64_f32_overflow_negative() {
        let err = f64_f32(f64::from(f32::MIN) * 2.0).unwrap_err();
        assert_eq!(err.from, "f64");
        assert_eq!(err.to, "f32");
    }

    #[test]
    fn f64_f32_inf_nan_allowed() {
        assert!(f64_f32(f64::INFINITY).is_ok());
        assert!(f64_f32(f64::NEG_INFINITY).is_ok());
        assert!(f64_f32(f64::NAN).is_ok());
    }

    #[test]
    fn cast_error_display() {
        let err = CastError {
            from: "usize",
            to: "u32",
            value: "5000000000".to_string(),
        };
        let s = err.to_string();
        assert!(s.contains("usize"));
        assert!(s.contains("u32"));
        assert!(s.contains("5000000000"));
    }
}
