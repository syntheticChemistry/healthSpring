// SPDX-License-Identifier: AGPL-3.0-or-later

//! Provenance trio IPC clients — `rhizoCrypt`, `loamSpine`, `sweetGrass`.
//!
//! Each module wraps JSON-RPC calls to the corresponding provenance primal
//! for DAG operations, ledger/merkle commits, and braid analytics.

pub mod loamspine;
pub mod nest;
pub mod rhizocrypt;
pub mod sweetgrass;
