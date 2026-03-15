// SPDX-License-Identifier: AGPL-3.0-only

use super::super::types::{HealthScenario, ScenarioEdge};
use super::topology::{
    DispatchStageInfo, TopologyNest, TopologyNode, TopologyTransfer, dispatch_scenario,
    topology_scenario,
};
use super::{bar, edge, gauge, node, scaffold, timeseries};

/// Build a V16 GPU scaling scenario showing timing vs batch size for each op.
///
/// Uses representative benchmarks (same ops validated in Exp085).
#[must_use]
pub fn gpu_scaling_study() -> (HealthScenario, Vec<ScenarioEdge>) {
    let mut s = scaffold(
        "healthSpring GPU Scaling",
        "V16 GPU op timing vs batch size — MM PK, SCFA, Beat Classify",
    );

    let scales: Vec<f64> = vec![64.0, 256.0, 1024.0, 4096.0];

    let mm_times = vec![0.8, 1.2, 3.5, 12.0];
    let scfa_times = vec![0.5, 0.9, 2.8, 9.5];
    let beat_times = vec![0.3, 0.7, 2.2, 8.0];

    s.ecosystem.primals.push(node(
        "gpu_scaling",
        "V16 GPU Scaling",
        "compute",
        &["compute.gpu.v16_scaling"],
        vec![
            timeseries(
                "mm_scaling",
                "MM Batch Timing",
                "Batch Size",
                "Time (ms)",
                "ms",
                scales.clone(),
                mm_times,
            ),
            timeseries(
                "scfa_scaling",
                "SCFA Batch Timing",
                "Batch Size",
                "Time (ms)",
                "ms",
                scales.clone(),
                scfa_times,
            ),
            timeseries(
                "beat_scaling",
                "Beat Classify Timing",
                "Batch Size",
                "Time (ms)",
                "ms",
                scales,
                beat_times,
            ),
            bar(
                "rust_vs_python",
                "Rust/Python Speedup",
                vec!["MM PK".into(), "SCFA".into(), "Beat Classify".into()],
                vec![45.0, 38.0, 52.0],
                "×",
            ),
            gauge(
                "overall_speedup",
                "Overall Rust/Python Speedup",
                45.0,
                1.0,
                100.0,
                "×",
                [10.0, 80.0],
                [1.0, 10.0],
            ),
        ],
        vec![],
    ));

    (s, vec![])
}

/// Build a NUCLEUS topology scenario for the V16 dispatch workstation.
///
/// Represents an `eastgate_tower` with CPU + GPU + NPU, showing V16
/// workload routing and `PCIe` P2P transfers.
#[must_use]
pub fn v16_topology_study() -> (HealthScenario, Vec<ScenarioEdge>) {
    let nodes = vec![TopologyNode {
        node_id: 0,
        pcie_gen: "Gen4".to_string(),
        nests: vec![
            TopologyNest {
                device_id: 0,
                substrate: "cpu".to_string(),
                label: "CPU (16 cores, AVX-512)".to_string(),
                memory_total_bytes: 64 * 1024 * 1024 * 1024,
                memory_used_bytes: 8 * 1024 * 1024 * 1024,
                utilization_pct: 12.0,
                available: true,
            },
            TopologyNest {
                device_id: 1,
                substrate: "gpu".to_string(),
                label: "RTX 4090 (24 GB VRAM)".to_string(),
                memory_total_bytes: 24 * 1024 * 1024 * 1024,
                memory_used_bytes: 512 * 1024 * 1024,
                utilization_pct: 35.0,
                available: true,
            },
            TopologyNest {
                device_id: 2,
                substrate: "npu".to_string(),
                label: "Akida AKD1000".to_string(),
                memory_total_bytes: 256 * 1024 * 1024,
                memory_used_bytes: 32 * 1024 * 1024,
                utilization_pct: 0.0,
                available: true,
            },
        ],
    }];

    let transfers = vec![
        TopologyTransfer {
            src_id: "T0.N0.D1".to_string(),
            dst_id: "T0.N0.D2".to_string(),
            label: "PCIe P2P GPU→NPU 31.5 GB/s".to_string(),
        },
        TopologyTransfer {
            src_id: "T0.N0.D2".to_string(),
            dst_id: "T0.N0.D0".to_string(),
            label: "PCIe P2P NPU→CPU 31.5 GB/s".to_string(),
        },
    ];

    topology_scenario(0, &nodes, &transfers)
}

/// Build a V16 dispatch plan scenario showing stage assignments.
///
/// 5-stage mixed pipeline: MM Batch (GPU) → SCFA Batch (GPU) →
/// Beat Classify (GPU) → NPU post-process → CPU diagnostic report.
#[must_use]
pub fn v16_dispatch_study() -> (HealthScenario, Vec<ScenarioEdge>) {
    let stages = vec![
        DispatchStageInfo {
            name: "MM Batch (1024 patients)".into(),
            substrate: "gpu".into(),
            elapsed_us: 3500.0,
            output_elements: 1024,
        },
        DispatchStageInfo {
            name: "SCFA Batch (1024 patients)".into(),
            substrate: "gpu".into(),
            elapsed_us: 2800.0,
            output_elements: 3072,
        },
        DispatchStageInfo {
            name: "Beat Classify (1024 windows)".into(),
            substrate: "gpu".into(),
            elapsed_us: 2200.0,
            output_elements: 1024,
        },
        DispatchStageInfo {
            name: "NPU Post-Process".into(),
            substrate: "npu".into(),
            elapsed_us: 150.0,
            output_elements: 512,
        },
        DispatchStageInfo {
            name: "Diagnostic Report".into(),
            substrate: "cpu".into(),
            elapsed_us: 25.0,
            output_elements: 1,
        },
    ];

    dispatch_scenario("V16 Mixed NUCLEUS Dispatch", &stages)
}

/// Combined compute pipeline scenario (scaling + topology + dispatch).
#[must_use]
pub fn compute_pipeline_study() -> (HealthScenario, Vec<ScenarioEdge>) {
    let (scaling, scaling_edges) = gpu_scaling_study();
    let (topo, topo_edges) = v16_topology_study();
    let (dispatch, dispatch_edges) = v16_dispatch_study();

    let mut s = scaffold(
        "healthSpring Compute Pipeline",
        "GPU scaling, NUCLEUS topology, and mixed dispatch — full compute visualization",
    );

    for track in [scaling, topo, dispatch] {
        for n in track.ecosystem.primals {
            s.ecosystem.primals.push(n);
        }
    }

    let mut all_edges = Vec::new();
    all_edges.extend(scaling_edges);
    all_edges.extend(topo_edges);
    all_edges.extend(dispatch_edges);

    all_edges.push(edge(
        "gpu_scaling",
        "dispatch_summary",
        "scaling → dispatch plan",
    ));

    (s, all_edges)
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn gpu_scaling_study_structure() {
        let (s, edges) = gpu_scaling_study();
        assert_eq!(s.ecosystem.primals.len(), 1);
        assert_eq!(s.ecosystem.primals[0].id, "gpu_scaling");
        assert!(edges.is_empty());
    }

    #[test]
    fn v16_topology_structure() {
        let (s, edges) = v16_topology_study();
        assert_eq!(s.ecosystem.primals.len(), 3);
        assert!(!edges.is_empty());
    }

    #[test]
    fn v16_dispatch_structure() {
        let (s, edges) = v16_dispatch_study();
        assert_eq!(s.ecosystem.primals.len(), 6);
        assert!(!edges.is_empty());
    }

    #[test]
    fn compute_pipeline_combines_all() {
        let (s, edges) = compute_pipeline_study();
        assert_eq!(s.ecosystem.primals.len(), 10);
        assert!(edges.iter().any(|e| e.from == "gpu_scaling"));
    }

    #[test]
    fn compute_pipeline_json_valid() {
        let (scenario, edges) = compute_pipeline_study();
        let json = crate::visualization::scenarios::scenario_with_edges_json(&scenario, &edges);
        let val: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert!(val.is_object());
        assert!(val["edges"].is_array());
    }
}
