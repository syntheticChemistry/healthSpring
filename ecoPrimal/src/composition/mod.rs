// SPDX-License-Identifier: AGPL-3.0-or-later

//! Composition context for healthSpring NUCLEUS deployments.
//!
//! Re-exports primalSpring's [`CompositionContext`] and adds health-domain
//! helpers for barraCuda statistics, provenance trio, and visualization routing.

mod context;
mod routing;

pub use context::HealthCompositionContext;
pub use primalspring::composition::CompositionContext;
pub use primalspring::composition::{
    call_or_skip, is_skip_error, validate_liveness, validate_parity, validate_parity_flex,
    DiscoveryPath, method_to_capability_domain,
};
pub use primalspring::composition::mesh::{GateNode, MeshRoute, MeshTopology};
pub use routing::{ALL_CAPS, capability_to_primal};
