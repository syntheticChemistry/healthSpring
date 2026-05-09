// SPDX-License-Identifier: AGPL-3.0-or-later

//! `rhizoCrypt` DAG provenance IPC client.

use serde_json::Value;

use crate::composition::HealthCompositionContext;

/// Create a DAG session for an experiment.
///
/// # Errors
///
/// Returns IPC error if rhizoCrypt is unavailable.
pub fn dag_session_create(
    ctx: &mut HealthCompositionContext,
    experiment: &str,
) -> Result<Value, primalspring::ipc::IpcError> {
    ctx.inner().call(
        "dag",
        "dag.session.create",
        serde_json::json!({"experiment": experiment}),
    )
}

/// Append an event to an existing DAG session.
///
/// # Errors
///
/// Returns IPC error if rhizoCrypt is unavailable.
pub fn dag_event_append(
    ctx: &mut HealthCompositionContext,
    session_id: &str,
    event: &str,
    data: &Value,
) -> Result<Value, primalspring::ipc::IpcError> {
    ctx.inner().call(
        "dag",
        "dag.event.append",
        serde_json::json!({"session_id": session_id, "event": event, "data": data}),
    )
}
