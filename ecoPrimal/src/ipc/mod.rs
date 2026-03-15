// SPDX-License-Identifier: AGPL-3.0-or-later
//! IPC server module — JSON-RPC 2.0 over Unix domain socket.
//!
//! Exposes healthSpring's science capabilities to `biomeOS` via the
//! `SPRING_AS_PROVIDER_PATTERN`. Socket discovery follows XDG conventions.

// JSON→Rust boundary: numeric casts from serde_json u64/f64 are bounded
// by the domain (e.g. patient count, dose mg) and validated before use.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "JSON-RPC params arrive as serde_json Number; casts are domain-bounded"
)]
pub mod dispatch;
pub mod rpc;
pub mod socket;
