// SPDX-License-Identifier: AGPL-3.0-only
//! IPC server module — JSON-RPC 2.0 over Unix domain socket.
//!
//! Exposes healthSpring's science capabilities to `biomeOS` via the
//! `SPRING_AS_PROVIDER_PATTERN`. Socket discovery follows XDG conventions.

// JSON→Rust boundary: numeric casts from serde_json u64/f64 are bounded.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::too_many_lines,
)]
pub mod dispatch;
pub mod rpc;
pub mod socket;
