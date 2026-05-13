// SPDX-License-Identifier: AGPL-3.0-or-later

//! `loamSpine` ledger/merkle provenance IPC client.
//!
//! Canonical wire names per loamSpine v0.9.16 GAP-36 reconciliation:
//!   - Ledger lifecycle: `spine.create`, `spine.get`, `spine.seal`
//!   - Entry operations: `entry.append`, `entry.get`, `entry.get_tip`
//!   - Certificate: `certificate.mint`

use serde_json::Value;

use crate::composition::HealthCompositionContext;

/// Create a new ledger spine via `spine.create`.
///
/// # Errors
///
/// Returns IPC error if loamSpine is unavailable.
pub fn spine_create(
    ctx: &mut HealthCompositionContext,
    experiment: &str,
    data: &Value,
) -> Result<Value, primalspring::ipc::IpcError> {
    ctx.inner().call(
        "commit",
        "spine.create",
        serde_json::json!({"experiment": experiment, "data": data}),
    )
}

/// Append an entry to the immutable ledger via `entry.append`.
///
/// # Errors
///
/// Returns IPC error if loamSpine is unavailable.
pub fn entry_append(
    ctx: &mut HealthCompositionContext,
    spine_id: &str,
    entry: &Value,
) -> Result<Value, primalspring::ipc::IpcError> {
    ctx.inner().call(
        "commit",
        "entry.append",
        serde_json::json!({"spine_id": spine_id, "entry": entry}),
    )
}

// Backward-compatible aliases for callers using the old names.

/// Create a new commit — delegates to [`spine_create`].
///
/// # Errors
///
/// Returns IPC error if loamSpine is unavailable.
pub fn commit_create(
    ctx: &mut HealthCompositionContext,
    experiment: &str,
    data: &Value,
) -> Result<Value, primalspring::ipc::IpcError> {
    spine_create(ctx, experiment, data)
}

/// Append to an existing ledger entry — delegates to [`entry_append`].
///
/// # Errors
///
/// Returns IPC error if loamSpine is unavailable.
pub fn ledger_append(
    ctx: &mut HealthCompositionContext,
    commit_id: &str,
    entry: &Value,
) -> Result<Value, primalspring::ipc::IpcError> {
    entry_append(ctx, commit_id, entry)
}
