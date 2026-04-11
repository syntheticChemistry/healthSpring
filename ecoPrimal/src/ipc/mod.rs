// SPDX-License-Identifier: AGPL-3.0-or-later
//! IPC module — JSON-RPC 2.0 over platform-agnostic transports.
//!
//! Exposes healthSpring's science capabilities to `biomeOS` via the
//! `SPRING_AS_PROVIDER_PATTERN`. Transport selection is runtime-based
//! following ecoBin v3.0 (Unix sockets, TCP fallback).

pub mod btsp;
pub mod client;
pub mod compute_dispatch;
pub mod data_dispatch;
pub mod discover;
pub mod dispatch;
pub mod error;
pub mod inference_dispatch;
pub mod lifecycle_dispatch;
pub mod mcp;
#[cfg(test)]
pub mod proptest_ipc;
pub mod protocol;
pub mod resilience;
pub mod rpc;
pub mod shader_dispatch;
pub mod socket;
pub mod tower_atomic;
pub mod transport;
