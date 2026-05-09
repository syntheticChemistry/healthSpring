// SPDX-License-Identifier: AGPL-3.0-or-later

//! Certification organelle — self-validating NUCLEUS node (library module).
//!
//! Absorbs the `healthspring_guidestone` binary logic into a callable library.
//! Three-tier primal proof per `GUIDESTONE_COMPOSITION_STANDARD` v1.2.0:
//!
//! - **Tier 1 (LOCAL):** Bare properties 1–5 + domain science. Always green in CI.
//! - **Tier 2 (IPC-WIRED):** IPC parity via live primals. `check_skip` when absent.
//! - **Tier 3 (FULL NUCLEUS):** Full science parity through NUCLEUS.

mod bare;
mod composition;
mod domain;

pub use bare::{
    validate_deterministic_output, validate_environment_agnostic, validate_reference_traceable,
    validate_self_verifying, validate_tolerance_documented,
};
pub use composition::{
    validate_barracuda_math_ipc, validate_manifest_capabilities, validate_primal_proof,
};
pub use domain::validate_domain_science;

use primalspring::composition::{CompositionContext, validate_liveness};
use primalspring::validation::ValidationResult;

/// Maximum certification layer. healthSpring uses 3 tiers.
pub const MAX_TIER: u8 = 3;

/// Run the full certification suite, returning a `ValidationResult`.
///
/// Maps to primalSpring's `certify()` pattern:
/// - `max_tier == 1`: bare properties + domain science only
/// - `max_tier >= 2`: adds IPC parity checks
/// - `max_tier >= 3`: adds full primal proof
///
/// Exit semantics: 0 = all pass, 1 = failure, 2 = bare only (no primals).
#[must_use]
pub fn certify(max_tier: u8) -> ValidationResult {
    let mut v = ValidationResult::new("healthspring certification");

    // ── Tier 1: LOCAL (always green, no IPC) ────────────────────────────
    v.section("Tier 1: Bare Properties (1–5)");
    validate_deterministic_output(&mut v);
    validate_reference_traceable(&mut v);
    validate_self_verifying(&mut v);
    validate_environment_agnostic(&mut v);
    validate_tolerance_documented(&mut v);

    v.section("Tier 1: Domain Science (local)");
    validate_domain_science(&mut v);

    if max_tier < 2 {
        v.finish();
        return v;
    }

    // ── Tier 2: IPC-WIRED (skip when primals absent) ───────────────────
    v.section("Tier 2: NUCLEUS Discovery");
    let mut ctx = CompositionContext::from_live_discovery_with_fallback();
    let alive = validate_liveness(
        &mut ctx,
        &mut v,
        &["tensor", "security", "storage", "dag", "commit"],
    );

    if alive == 0 {
        v.section("Summary (Tier 1 bare)");
        v.finish();
        return v;
    }

    v.section("Tier 2: barraCuda Math IPC");
    validate_barracuda_math_ipc(&mut ctx, &mut v);

    v.section("Tier 2: Manifest Capabilities");
    validate_manifest_capabilities(&mut ctx, &mut v);

    if max_tier < 3 {
        v.finish();
        return v;
    }

    // ── Tier 3: FULL NUCLEUS (primal proof) ────────────────────────────
    v.section("Tier 3: Primal Proof — Science via IPC");
    validate_primal_proof(&mut ctx, &mut v);

    v.finish();
    v
}
