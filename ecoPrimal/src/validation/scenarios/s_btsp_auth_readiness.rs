// SPDX-License-Identifier: AGPL-3.0-or-later

//! BTSP auth readiness scenario — S4 validation for ironGate.
//!
//! Validates that the composition context correctly probes primals for
//! BTSP server support and reports authentication state. This is the
//! healthSpring-side readiness check for the S4 formal auth gate.

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "btsp-auth-readiness",
            track: Track::Composition,
            tier: Tier::Both,
            source_experiment: "s4_auth",
            description: "BTSP auth readiness — probe pattern, escalation state, S4 gate pre-check.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, ctx: &mut CompositionContext) {
    v.section("Phase 1: BTSP Probe Pattern (structural)");

    let probe_struct_ok = std::mem::size_of::<crate::ipc::btsp::BtspCapabilities>() > 0;
    v.check_bool(
        "btsp.capabilities_struct_defined",
        probe_struct_ok,
        "BtspCapabilities struct exists",
    );

    v.check_bool(
        "btsp.probe_fn_callable",
        true,
        "probe_btsp_capabilities() is public and callable",
    );

    v.check_bool(
        "btsp.upgrade_fn_callable",
        true,
        "should_upgrade_btsp() is public and callable",
    );

    let nonexistent = std::path::Path::new("/nonexistent/btsp_probe_scenario.sock");
    let probe_result = crate::ipc::btsp::probe_btsp_capabilities(nonexistent);
    v.check_bool(
        "btsp.probe_nonexistent_returns_none",
        probe_result.is_none(),
        "probe of nonexistent socket correctly returns None",
    );

    let no_seed_no_upgrade = !crate::ipc::btsp::should_upgrade_btsp(nonexistent)
        || std::env::var("FAMILY_SEED").is_ok();
    v.check_bool(
        "btsp.upgrade_guard",
        no_seed_no_upgrade,
        "should_upgrade_btsp returns false without FAMILY_SEED",
    );

    v.section("Phase 2: Composition BTSP State");

    let security_caps = ["security", "storage", "dag", "commit", "tensor"];
    let family_seed_set = std::env::var("FAMILY_SEED").is_ok();

    let mut probed = 0u32;
    let mut authenticated = 0u32;

    for cap in &security_caps {
        match ctx.btsp_authenticated(cap) {
            Some(true) => {
                v.check_bool(
                    &format!("btsp.ctx_auth:{cap}"),
                    true,
                    "BTSP-authenticated via CompositionContext",
                );
                authenticated += 1;
                probed += 1;
            }
            Some(false) => {
                if family_seed_set {
                    v.check_bool(
                        &format!("btsp.ctx_auth:{cap}"),
                        false,
                        "cleartext despite FAMILY_SEED — primal lacks BTSP server",
                    );
                } else {
                    v.check_skip(
                        &format!("btsp.ctx_auth:{cap}"),
                        "cleartext (standalone, no FAMILY_SEED)",
                    );
                }
                probed += 1;
            }
            None => {
                v.check_skip(
                    &format!("btsp.ctx_auth:{cap}"),
                    "capability not discovered",
                );
            }
        }
    }

    v.section("Phase 3: S4 Auth Gate Summary");

    v.check_bool(
        "btsp.s4_probe_coverage",
        probed > 0 || ctx.available_capabilities().is_empty(),
        &format!("{probed}/{} security capabilities probed", security_caps.len()),
    );

    if family_seed_set {
        v.check_bool(
            "btsp.s4_auth_active",
            authenticated > 0,
            &format!("{authenticated}/{probed} caps BTSP-authenticated (S4 active)"),
        );
    } else {
        v.check_skip(
            "btsp.s4_auth_active",
            "S4 auth gate requires FAMILY_SEED — not set in this environment",
        );
    }
}
