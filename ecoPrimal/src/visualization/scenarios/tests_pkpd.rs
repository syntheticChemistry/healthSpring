// SPDX-License-Identifier: AGPL-3.0-or-later
//! PK/PD and dose-response scenario tests (including NLME).

#![allow(unused_imports, reason = "shared test imports across scenario modules")]

use super::*;
use crate::tolerances;
use crate::visualization::{
    DataChannel, EdgeType, HealthScenario, NodeStatus, NodeType, ScenarioEdge,
};

#[test]
fn pkpd_study_structure() {
    let (scenario, edges) = super::pkpd_study();
    super::assert_study_invariants(
        &scenario,
        &edges,
        &["hill", "one_comp", "two_comp", "mab", "pop_pk", "pbpk"],
        5,
    );
}

#[test]
fn pkpd_study_capabilities() {
    let (scenario, _) = super::pkpd_study();
    let caps: std::collections::HashSet<String> = scenario
        .ecosystem
        .primals
        .iter()
        .flat_map(|n| n.capabilities.clone())
        .collect();
    assert!(caps.contains("science.pkpd.hill_dose_response"));
    assert!(caps.contains("science.pkpd.one_compartment_pk"));
    assert!(caps.contains("science.pkpd.two_compartment_pk"));
    assert!(caps.contains("science.pkpd.allometric_scaling"));
    assert!(caps.contains("science.pkpd.population_pk"));
    assert!(caps.contains("science.pkpd.pbpk"));
}

#[test]
fn pkpd_study_json_roundtrips() {
    let (scenario, edges) = super::pkpd_study();
    super::assert_json_roundtrips(&scenario, &edges);
}

#[test]
fn nlme_study_structure() {
    let (scenario, edges) = super::nlme_study();
    super::assert_study_invariants(
        &scenario,
        &edges,
        &[
            "nlme_population",
            "nca_metrics",
            "cwres_diagnostics",
            "vpc_check",
            "gof_fit",
        ],
        5,
    );
}

#[test]
fn nlme_study_capabilities() {
    let (scenario, _) = super::nlme_study();
    let caps: std::collections::HashSet<String> = scenario
        .ecosystem
        .primals
        .iter()
        .flat_map(|n| n.capabilities.clone())
        .collect();
    assert!(caps.contains("science.pkpd.nlme_foce"));
    assert!(caps.contains("science.pkpd.nca"));
    assert!(caps.contains("science.pkpd.nlme_diagnostics"));
}

#[test]
fn nlme_study_json_roundtrips() {
    let (scenario, edges) = super::nlme_study();
    super::assert_json_roundtrips(&scenario, &edges);
}

#[test]
fn pkpd_study_has_scatter3d_channel() {
    let (scenario, _) = super::pkpd_study();
    let pop_node = scenario
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "pop_pk")
        .expect("pop_pk node");
    let has_scatter = pop_node
        .data_channels
        .iter()
        .any(|ch| matches!(ch, DataChannel::Scatter3D { id, .. } if id == "pop_pk_3d"));
    assert!(has_scatter, "Pop PK node should have a Scatter3D channel");
}
