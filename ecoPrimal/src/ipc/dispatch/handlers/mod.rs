// SPDX-License-Identifier: AGPL-3.0-or-later
//! IPC capability handlers — domain-specific JSON-RPC param extraction and dispatch.
//!
//! Param helpers (f, fa, ua, missing) are shared across all handler modules.

use serde_json::Value;

pub(super) mod biosignal;
pub(super) mod clinical;
pub(super) mod microbiome;
pub(super) mod pkpd;
pub(super) mod simulation;
pub(super) mod toxicology;

// ═══════════════════════════════════════════════════════════════════════════
// Param extraction helpers — used by all domain modules
// ═══════════════════════════════════════════════════════════════════════════

pub(super) fn f(params: &Value, key: &str) -> Option<f64> {
    params.get(key).and_then(Value::as_f64)
}

pub(super) fn fa(params: &Value, key: &str) -> Option<Vec<f64>> {
    params.get(key).and_then(|v| {
        v.as_array()
            .map(|arr| arr.iter().filter_map(Value::as_f64).collect())
    })
}

pub(super) fn ua(params: &Value, key: &str) -> Option<Vec<u64>> {
    params.get(key).and_then(|v| {
        v.as_array()
            .map(|arr| arr.iter().filter_map(Value::as_u64).collect())
    })
}

/// Extract a JSON integer as `usize`, saturating on overflow.
pub(super) fn sz(params: &Value, key: &str) -> Option<usize> {
    params
        .get(key)
        .and_then(Value::as_u64)
        .map(|v| usize::try_from(v).unwrap_or(usize::MAX))
}

/// Extract a JSON integer as `usize` with a default value.
pub(super) fn sz_or(params: &Value, key: &str, default: usize) -> usize {
    sz(params, key).unwrap_or(default)
}

/// Convert a JSON `u64` array into `Vec<usize>`, saturating on overflow.
pub(super) fn sza(params: &Value, key: &str) -> Option<Vec<usize>> {
    params.get(key).and_then(|v| {
        v.as_array().map(|arr| {
            arr.iter()
                .filter_map(Value::as_u64)
                .map(|v| usize::try_from(v).unwrap_or(usize::MAX))
                .collect()
        })
    })
}

pub(super) fn missing(name: &str) -> Value {
    serde_json::json!({"error": "missing_params", "param": name})
}
