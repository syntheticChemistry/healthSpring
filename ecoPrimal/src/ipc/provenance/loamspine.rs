// SPDX-License-Identifier: AGPL-3.0-or-later

//! `loamSpine` ledger/merkle provenance IPC client.

use serde_json::Value;

use crate::composition::HealthCompositionContext;

/// Create a new commit in the provenance ledger.
///
/// # Errors
///
/// Returns IPC error if loamSpine is unavailable.
pub fn commit_create(
    ctx: &mut HealthCompositionContext,
    experiment: &str,
    data: &Value,
) -> Result<Value, primalspring::ipc::IpcError> {
    ctx.inner().call(
        "commit",
        "commit.create",
        serde_json::json!({"experiment": experiment, "data": data}),
    )
}

/// Append to an existing ledger entry.
///
/// # Errors
///
/// Returns IPC error if loamSpine is unavailable.
pub fn ledger_append(
    ctx: &mut HealthCompositionContext,
    commit_id: &str,
    entry: &Value,
) -> Result<Value, primalspring::ipc::IpcError> {
    ctx.inner().call(
        "commit",
        "ledger.append",
        serde_json::json!({"commit_id": commit_id, "entry": entry}),
    )
}
