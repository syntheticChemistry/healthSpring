// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Lint policy: workspace-level [lints] in root Cargo.toml.
// forbid(unsafe_code), deny(clippy::{all,pedantic,nursery,unwrap_used,expect_used}).

//! metalForge — heterogeneous compute dispatch for healthSpring.
//!
//! Routes workloads to CPU, GPU, or NPU based on runtime capability discovery.
//! Organizes hardware via NUCLEUS atomics (Tower → Node → Nest) and plans
//! inter-device transfers (`PCIe` P2P DMA, host-staged, network IPC).
//!
//! ## ABSORPTION STATUS (barraCuda / toadStool / biomeOS)
//!
//! - `Substrate` + `Workload` enum -> barraCuda workload classification
//! - `select_substrate()` threshold routing -> toadStool dispatcher
//! - `Capabilities::discover()` -> barraCuda hardware probe
//! - `DispatchPlan` + NUCLEUS topology -> biomeOS graph planner
//!
//! ## Architecture
//!
//! ```text
//! biomeOS graph (DAG of pipeline stages)
//!     │
//!     ▼
//! metalForge dispatch ── selects substrate per stage
//!     │
//!     ▼
//! NUCLEUS topology ── Tower → Node → Nest
//!     │
//!     ├── `PCIe` P2P DMA (GPU↔NPU, bypass CPU)
//!     ├── Host-staged (CPU mediates)
//!     └── Network IPC (cross-node via biomeOS)
//! ```

mod discovery;
mod routing;
mod types;

/// Stage→Nest assignment and transfer planning for pipeline execution.
pub mod dispatch;
/// NUCLEUS topology (`Tower` → `Node` → `Nest`) for heterogeneous scheduling.
pub mod nucleus;
/// Inter-`Nest` transfer path selection (`PCIe` P2P, host-staged, or IPC).
pub mod transfer;

pub use discovery::Capabilities;
pub use routing::{select_substrate, select_substrate_with_thresholds};
pub use types::{DispatchThresholds, GpuInfo, NpuInfo, PrecisionRouting, Substrate, Workload};
