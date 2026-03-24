// SPDX-License-Identifier: AGPL-3.0-or-later
//! [`OrExit`]: unwrap-or-exit for binaries without panics.

use tracing::error;

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

#[cfg(test)]
mod tests {
    use super::OrExit;

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
