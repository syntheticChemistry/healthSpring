// SPDX-License-Identifier: AGPL-3.0-or-later
//! Biosignal / ECG / HRV / PPG scenario tests.

#![allow(unused_imports, reason = "shared test imports across scenario modules")]

use super::*;
use crate::tolerances;
use crate::visualization::{
    DataChannel, EdgeType, HealthScenario, NodeStatus, NodeType, ScenarioEdge,
};

#[test]
fn biosignal_study_structure() {
    let (scenario, edges) = super::biosignal_study();
    super::assert_study_invariants(
        &scenario,
        &edges,
        &["qrs", "hrv", "spo2", "fusion", "wfdb_ecg"],
        4,
    );
}

#[test]
fn biosignal_study_capabilities() {
    let (scenario, _) = super::biosignal_study();
    let caps: std::collections::HashSet<String> = scenario
        .ecosystem
        .primals
        .iter()
        .flat_map(|n| n.capabilities.clone())
        .collect();
    assert!(caps.contains("science.biosignal.pan_tompkins"));
    assert!(caps.contains("science.biosignal.hrv"));
    assert!(caps.contains("science.biosignal.ppg_spo2"));
    assert!(caps.contains("science.biosignal.fusion"));
    assert!(caps.contains("science.biosignal.wfdb_format212"));
}

#[test]
fn biosignal_study_json_roundtrips() {
    let (scenario, edges) = super::biosignal_study();
    super::assert_json_roundtrips(&scenario, &edges);
}

#[test]
fn biosignal_study_has_spectrum_channel() {
    let (scenario, _) = super::biosignal_study();
    let hrv_node = scenario
        .ecosystem
        .primals
        .iter()
        .find(|n| n.id == "hrv")
        .expect("hrv node");
    let has_spectrum = hrv_node
        .data_channels
        .iter()
        .any(|ch| matches!(ch, DataChannel::Spectrum { id, .. } if id == "hrv_psd"));
    assert!(has_spectrum, "HRV node should have a Spectrum channel");
}
