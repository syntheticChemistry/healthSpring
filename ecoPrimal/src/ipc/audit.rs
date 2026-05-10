// SPDX-License-Identifier: AGPL-3.0-or-later

//! `skunkBat` audit logging IPC client.
//!
//! Forwards audit events to the ecosystem audit primal for DAG recording
//! and braid analytics via `rhizoCrypt` + `sweetGrass`.

use serde_json::Value;

use crate::composition::HealthCompositionContext;

/// Log an audit event via `skunkBat`.
///
/// # Errors
///
/// Returns IPC error if `skunkBat` is unavailable.
pub fn audit_log(
    ctx: &mut HealthCompositionContext,
    event_type: &str,
    data: &Value,
) -> Result<Value, primalspring::ipc::IpcError> {
    ctx.inner().call(
        "audit",
        "audit.log",
        serde_json::json!({
            "source": crate::PRIMAL_NAME,
            "event_type": event_type,
            "data": data,
        }),
    )
}

/// Log a certification result via `skunkBat`.
///
/// # Errors
///
/// Returns IPC error if `skunkBat` is unavailable.
pub fn audit_certification(
    ctx: &mut HealthCompositionContext,
    tier: u8,
    passed: u32,
    failed: u32,
    skipped: u32,
) -> Result<Value, primalspring::ipc::IpcError> {
    ctx.inner().call(
        "audit",
        "audit.log",
        serde_json::json!({
            "source": crate::PRIMAL_NAME,
            "event_type": "certification",
            "data": {
                "tier": tier,
                "passed": passed,
                "failed": failed,
                "skipped": skipped,
            },
        }),
    )
}
