// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

//! healthSpring guideStone — self-validating NUCLEUS node.
//!
//! A guideStone carries 5 certified properties:
//!
//! 1. **Deterministic Output** — same binary, same results, any architecture
//! 2. **Reference-Traceable** — every number traces to a paper or proof
//! 3. **Self-Verifying** — tampered inputs detected, non-zero exit
//! 4. **Environment-Agnostic** — pure Rust, ecoBin, no network, no sudo
//! 5. **Tolerance-Documented** — every tolerance has a derivation
//!
//! ## Exit codes
//!
//! - `0` — all checks passed (NUCLEUS certified)
//! - `1` — at least one check failed
//! - `2` — no NUCLEUS deployed (bare guideStone only)
//!
//! ## Layered certification
//!
//! `primalspring_guidestone` validates composition correctness (6 layers).
//! This binary validates healthSpring's domain science ON TOP of that base.
//!
//! ## Usage
//!
//! ```bash
//! # Bare guideStone (no primals needed)
//! cargo run --features guidestone --bin healthspring_guidestone
//!
//! # With NUCLEUS deployed (IPC parity)
//! biomeos deploy --graph healthspring_enclave_proto_nucleate.toml
//! cargo run --features guidestone --bin healthspring_guidestone
//! ```

mod bare;
mod domain;

use primalspring::composition::{CompositionContext, validate_liveness};
use primalspring::validation::ValidationResult;

fn main() {
    eprintln!("╔══════════════════════════════════════════════════════════════════╗");
    eprintln!("║  healthSpring guideStone — self-validating NUCLEUS node        ║");
    eprintln!("║  Domain: clinical health (PK/PD, microbiome, biosignal)        ║");
    eprintln!("╚══════════════════════════════════════════════════════════════════╝\n");

    eprintln!(
        "substrate: {} {}",
        std::env::consts::ARCH,
        std::env::consts::OS
    );
    eprintln!("engine:    cpu-native (pure Rust, NUCLEUS auto-detected)\n");

    let mut v = ValidationResult::new("healthspring guideStone");

    // ── Phase 1: Bare guideStone (Properties 1–5, no primals needed) ─────

    v.section("Bare Properties (1–5)");
    bare::validate_deterministic_output(&mut v);
    bare::validate_reference_traceable(&mut v);
    bare::validate_environment_agnostic(&mut v);
    bare::validate_tolerance_documented(&mut v);

    // ── Phase 2: NUCLEUS additive layer (IPC parity) ─────────────────────

    v.section("NUCLEUS Discovery");
    let mut ctx = CompositionContext::from_live_discovery_with_fallback();
    let alive = validate_liveness(
        &mut ctx,
        &mut v,
        &["tensor", "security", "storage", "dag", "commit"],
    );

    if alive == 0 {
        eprintln!("\nNo NUCLEUS primals discovered. Bare guideStone only.");
        v.section("Summary (bare)");
        let code = bare_exit_code(&v);
        v.finish();
        std::process::exit(code);
    }

    // ── Phase 3: Domain science via IPC ──────────────────────────────────

    v.section("barraCuda Math IPC");
    domain::validate_barracuda_math_ipc(&mut ctx, &mut v);

    v.section("Validation Capabilities (proto-nucleate manifest)");
    domain::validate_manifest_capabilities(&mut ctx, &mut v);

    v.section("Domain Science Parity");
    domain::validate_domain_science(&mut v);

    // ── Finish ───────────────────────────────────────────────────────────

    v.finish();
    std::process::exit(v.exit_code());
}

/// Bare guideStone exit code: 0 if bare passed, 2 if any bare check failed
/// (no primals to do full validation, but bare is fine).
const fn bare_exit_code(v: &ValidationResult) -> i32 {
    if v.failed == 0 && v.passed > 0 { 2 } else { 1 }
}
