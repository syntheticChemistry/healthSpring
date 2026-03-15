// SPDX-License-Identifier: AGPL-3.0-or-later
//! Node builders for patient-parameterized TRT clinical scenarios.
//!
//! Each function produces a single [`ScenarioNode`] populated with
//! validated endocrine models (Mok, Saad, Sharma, Kapoor).
//!
//! Domain-focused submodules:
//! - [`endocrine_assessment`] — patient assessment (T decline, age projection)
//! - [`endocrine_outcomes`] — metabolic, cardiovascular, glycemic (Saad, Sharma, Kapoor)
//! - [`endocrine_cardiac`] — cardiac monitoring (HRV, SDNN)
//! - [`pkpd`] — protocol PK, population comparison
//! - [`microbiome`] — gut health / diversity

mod endocrine_assessment;
mod endocrine_cardiac;
mod endocrine_outcomes;
mod microbiome;
mod pkpd;

pub(super) use endocrine_assessment::assessment_node;
pub(super) use endocrine_cardiac::cardiac_monitor_node;
pub(super) use endocrine_outcomes::{cardiovascular_node, diabetes_node, metabolic_node};
pub(super) use microbiome::gut_health_node;
pub(super) use pkpd::{population_node, protocol_node};
