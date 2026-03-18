// SPDX-License-Identifier: AGPL-3.0-or-later
//! IPC server module — JSON-RPC 2.0 over Unix domain socket.
//!
//! Exposes healthSpring's science capabilities to `biomeOS` via the
//! `SPRING_AS_PROVIDER_PATTERN`. Socket discovery follows XDG conventions.

pub mod compute_dispatch;
pub mod data_dispatch;
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
