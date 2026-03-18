// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]

//! toadStool — sovereign compute dispatch for healthSpring.
//!
//! Owns hardware interaction and provides a unidirectional streaming
//! pipeline model: data flows CPU → GPU → output, no round-trips
//! during computation.
//!
//! ## Pipeline Model
//!
//! ```text
//! Source (CPU) ─── Upload ──→ Device (GPU/NPU) ─── Compute ──→ Download ──→ Sink (CPU)
//!     │                           │                                │
//!     └── Prepare buffers         └── Execute `WGSL` shader        └── Collect results
//! ```
//!
//! ## Relationship to Primals
//!
//! - **barraCuda**: Provides `WGSL` shaders and Tensor operations
//! - **metalForge**: Routes workloads to substrates (CPU/GPU/NPU)
//! - **toadStool**: Manages the pipeline lifecycle (this crate)
//! - **coralReef**: Compiles `WGSL` → native GPU binary (external)

pub mod pipeline;
pub mod stage;
