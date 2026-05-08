// SPDX-License-Identifier: AGPL-3.0-or-later
//! Endocrine / testosterone scenario tests.

#![allow(unused_imports)]

use super::*;
use crate::tolerances;
use crate::visualization::{DataChannel, EdgeType, HealthScenario, NodeStatus, NodeType, ScenarioEdge};

#[test]
fn endocrine_study_structure() {
    let (scenario, edges) = super::endocrine_study();
    super::assert_study_invariants(
        &scenario,
        &edges,
        &[
            "t_im",
            "t_pellet",
            "age_decline",
            "trt_weight",
            "trt_cardio",
            "trt_diabetes",
            "gut_axis",
            "hrv_cardiac",
        ],
        7,
    );
}

#[test]
fn endocrine_study_capabilities() {
    let (scenario, _) = super::endocrine_study();
    let caps: std::collections::HashSet<String> = scenario
        .ecosystem
        .primals
        .iter()
        .flat_map(|n| n.capabilities.clone())
        .collect();
    assert!(caps.contains("science.endocrine.testosterone_im"));
    assert!(caps.contains("science.endocrine.testosterone_pellet"));
    assert!(caps.contains("science.endocrine.testosterone_decline"));
    assert!(caps.contains("science.endocrine.trt_weight"));
    assert!(caps.contains("science.endocrine.trt_cardiovascular"));
    assert!(caps.contains("science.endocrine.trt_diabetes"));
    assert!(caps.contains("science.endocrine.gut_trt_axis"));
    assert!(caps.contains("science.endocrine.hrv_trt"));
}

#[test]
fn endocrine_study_json_roundtrips() {
    let (scenario, edges) = super::endocrine_study();
    super::assert_json_roundtrips(&scenario, &edges);
}
