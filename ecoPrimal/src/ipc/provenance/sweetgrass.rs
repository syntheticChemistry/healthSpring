// SPDX-License-Identifier: AGPL-3.0-or-later

//! `sweetGrass` braid/analytics provenance IPC client.

use serde_json::Value;

use crate::composition::HealthCompositionContext;

/// Create a new provenance braid.
///
/// # Errors
///
/// Returns IPC error if sweetGrass is unavailable.
pub fn braid_create(
    ctx: &mut HealthCompositionContext,
    experiment: &str,
) -> Result<Value, primalspring::ipc::IpcError> {
    ctx.inner().call(
        "braid",
        "braid.create",
        serde_json::json!({"experiment": experiment}),
    )
}

/// Commit to an existing braid.
///
/// # Errors
///
/// Returns IPC error if sweetGrass is unavailable.
pub fn braid_commit(
    ctx: &mut HealthCompositionContext,
    braid_id: &str,
    data: &Value,
) -> Result<Value, primalspring::ipc::IpcError> {
    ctx.inner().call(
        "braid",
        "braid.commit",
        serde_json::json!({"braid_id": braid_id, "data": data}),
    )
}
