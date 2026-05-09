// SPDX-License-Identifier: AGPL-3.0-or-later
//! Microbiome / diversity scenario tests.

#![allow(unused_imports, reason = "shared test imports across scenario modules")]

use super::*;
use crate::tolerances;
use crate::visualization::{
    DataChannel, EdgeType, HealthScenario, NodeStatus, NodeType, ScenarioEdge,
};

#[test]
fn microbiome_study_structure() {
    let (scenario, edges) = super::microbiome_study();
    super::assert_study_invariants(
        &scenario,
        &edges,
        &["diversity", "anderson", "cdiff", "fmt"],
        3,
    );
}

#[test]
fn microbiome_study_capabilities() {
    let (scenario, _) = super::microbiome_study();
    let caps: std::collections::HashSet<String> = scenario
        .ecosystem
        .primals
        .iter()
        .flat_map(|n| n.capabilities.clone())
        .collect();
    assert!(caps.contains("science.microbiome.diversity"));
    assert!(caps.contains("science.microbiome.anderson_lattice"));
    assert!(caps.contains("science.microbiome.cdiff_resistance"));
    assert!(caps.contains("science.microbiome.fmt"));
}

#[test]
fn microbiome_study_json_roundtrips() {
    let (scenario, edges) = super::microbiome_study();
    super::assert_json_roundtrips(&scenario, &edges);
}

#[test]
fn microbiome_study_has_heatmap_channel() {
    let (scenario, _) = super::microbiome_study();
    let div_node = scenario
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "diversity")
        .expect("diversity node");
    let has_heatmap = div_node
        .data_channels
        .iter()
        .any(|ch| matches!(ch, DataChannel::Heatmap { id, .. } if id == "bray_curtis"));
    assert!(has_heatmap, "Diversity node should have a Heatmap channel");
}
