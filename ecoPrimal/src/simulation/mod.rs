// SPDX-License-Identifier: AGPL-3.0-or-later
//! Multi-scale causal simulation: the internals of the terrarium.
//!
//! The biphasic dose-response curve is the glass. This module builds the
//! causal chain *inside* — from molecular binding through cellular stress
//! pathways, tissue integration, organism fitness, population dynamics,
//! to ecosystem structure.
//!
//! ## The Causal Chain
//!
//! ```text
//! dose → binding → stress sensing → pathway activation → cellular fitness
//!   → tissue integration → organism fitness → population → ecosystem
//! ```
//!
//! Each level is mechanistic (it explains *why*), not phenomenological
//! (which only describes *what*). The biphasic shape EMERGES from the
//! competition between repair pathways (saturating benefit) and damage
//! accumulation (unbounded harm).
//!
//! ## Spring Ownership
//!
//! | Scale | Spring | Module |
//! |-------|--------|--------|
//! | Molecular (binding) | healthSpring | `discovery::affinity_landscape` |
//! | Cellular (stress) | healthSpring | `simulation` (this module) |
//! | Tissue (integration) | healthSpring | `toxicology` |
//! | Organism (PK/PD) | healthSpring | `pkpd`, `toxicology` |
//! | Population | wetSpring, groundSpring | `simulation` (basic), wetSpring (full) |
//! | Ecosystem | all springs | `simulation` (framework), wateringHole (coordination) |

mod causal_chain;
mod ecosystem;
mod population;
mod stress;

pub use causal_chain::*;
pub use ecosystem::*;
pub use population::*;
pub use stress::*;
