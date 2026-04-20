// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

//! healthSpring guideStone — self-validating NUCLEUS node.
//!
//! Three-tier primal proof harness per `GUIDESTONE_COMPOSITION_STANDARD` v1.2.0:
//!
//! - **Tier 1 (LOCAL):** Bare properties 1–5 + domain science. Always green in CI.
//! - **Tier 2 (IPC-WIRED):** IPC parity via live primals. `check_skip` when absent.
//! - **Tier 3 (FULL NUCLEUS):** Deploy from plasmidBin, validate externally.
//!
//! ## Five certified properties
//!
//! 1. **Deterministic Output** — same binary, same results, any architecture
//! 2. **Reference-Traceable** — every number traces to a paper or proof
//! 3. **Self-Verifying** — BLAKE3 checksums detect tampering (v1.1.0)
//! 4. **Environment-Agnostic** — pure Rust, ecoBin, no network, no sudo
//! 5. **Tolerance-Documented** — every tolerance has a derivation
//!
//! ## Exit codes
//!
//! - `0` — all checks passed (NUCLEUS certified)
//! - `1` — at least one check failed
//! - `2` — no NUCLEUS deployed (bare guideStone only — Tier 1 passed)
//!
//! ## Usage
//!
//! ```bash
//! # Tier 1 only (no primals needed)
//! cargo run --features guidestone --bin healthspring_guidestone
//!
//! # Tier 2+3: deploy NUCLEUS from plasmidBin, then validate
//! export FAMILY_ID="healthspring-validation"
//! export BEARDOG_FAMILY_SEED="$(head -c 32 /dev/urandom | xxd -p)"
//! ./nucleus_launcher.sh --composition full start
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
    eprintln!("║  Standard: GUIDESTONE_COMPOSITION_STANDARD v1.2.0             ║");
    eprintln!("╚══════════════════════════════════════════════════════════════════╝\n");

    eprintln!(
        "substrate: {} {}",
        std::env::consts::ARCH,
        std::env::consts::OS
    );
    let family = std::env::var("FAMILY_ID").unwrap_or_else(|_| "default".to_owned());
    eprintln!("family:    {family}");
    eprintln!("engine:    cpu-native (pure Rust, NUCLEUS auto-detected)\n");

    let mut v = ValidationResult::new("healthspring guideStone");

    // ── Tier 1: LOCAL_CAPABILITIES (always green, no IPC) ────────────────

    v.section("Tier 1: Bare Properties (1–5)");
    bare::validate_deterministic_output(&mut v);
    bare::validate_reference_traceable(&mut v);
    bare::validate_self_verifying(&mut v);
    bare::validate_environment_agnostic(&mut v);
    bare::validate_tolerance_documented(&mut v);

    v.section("Tier 1: Domain Science (local)");
    domain::validate_domain_science(&mut v);

    // ── Tier 2: IPC-WIRED (skip when primals absent) ─────────────────────

    v.section("Tier 2: NUCLEUS Discovery");
    let mut ctx = CompositionContext::from_live_discovery_with_fallback();
    let alive = validate_liveness(
        &mut ctx,
        &mut v,
        &["tensor", "security", "storage", "dag", "commit"],
    );

    if alive == 0 {
        eprintln!("\nNo NUCLEUS primals discovered. Tier 1 (bare) only.");
        v.section("Summary (Tier 1 bare)");
        let code = bare_exit_code(&v);
        v.finish();
        std::process::exit(code);
    }

    v.section("Tier 2: barraCuda Math IPC");
    domain::validate_barracuda_math_ipc(&mut ctx, &mut v);

    v.section("Tier 2: Manifest Capabilities");
    domain::validate_manifest_capabilities(&mut ctx, &mut v);

    // ── Tier 3: FULL NUCLEUS (primal proof) ──────────────────────────────

    v.section("Tier 3: Primal Proof — Science via IPC");
    domain::validate_primal_proof(&mut ctx, &mut v);

    // ── Finish ───────────────────────────────────────────────────────────

    v.finish();
    std::process::exit(v.exit_code());
}

/// Bare guideStone exit code: 0 if bare passed, 2 if no primals
/// (Tier 1 validated, but Tier 2/3 not exercised).
const fn bare_exit_code(v: &ValidationResult) -> i32 {
    if v.failed == 0 && v.passed > 0 { 2 } else { 1 }
}
