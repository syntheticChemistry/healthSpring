// SPDX-License-Identifier: AGPL-3.0-or-later
//! NUCLEUS topology scenario builder for petalTongue visualization.
//!
//! Renders a Tower/Node/Nest hardware topology as a petalTongue scenario graph,
//! with dispatch plan assignments shown as node properties and PCIe/network
//! transfers as edges.

use super::{ScenarioEdge, bar, edge, gauge, node, scaffold};
use crate::visualization::types::HealthScenario;

/// Build a petalTongue scenario from a NUCLEUS topology description.
///
/// Each Nest becomes a graph node with gauge channels for memory capacity
/// and utilization. Edges represent `PCIe` or network transfer paths.
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "memory bytes → f64 for gauge display"
)]
pub fn topology_scenario(
    tower_id: u16,
    nodes: &[TopologyNode],
    transfers: &[TopologyTransfer],
) -> (HealthScenario, Vec<ScenarioEdge>) {
    let mut scenario = scaffold(
        &format!("NUCLEUS Tower {tower_id}"),
        "Hardware topology — Tower/Node/Nest hierarchy with PCIe P2P and dispatch assignments",
    );

    let mut edges = Vec::new();

    for topo_node in nodes {
        for nest in &topo_node.nests {
            let nest_id = format!("T{}.N{}.D{}", tower_id, topo_node.node_id, nest.device_id);
            let channels = vec![
                gauge(
                    &format!("{nest_id}_mem"),
                    "Memory",
                    nest.memory_used_bytes as f64,
                    0.0,
                    nest.memory_total_bytes as f64,
                    "bytes",
                    [0.0, nest.memory_total_bytes as f64 * 0.8],
                    [
                        nest.memory_total_bytes as f64 * 0.8,
                        nest.memory_total_bytes as f64 * 0.95,
                    ],
                ),
                gauge(
                    &format!("{nest_id}_util"),
                    "Utilization",
                    nest.utilization_pct,
                    0.0,
                    100.0,
                    "%",
                    [0.0, 80.0],
                    [80.0, 95.0],
                ),
            ];

            let status = if nest.available {
                "healthy"
            } else {
                "degraded"
            };
            let scenario_node = super::ScenarioNode {
                id: nest_id.clone(),
                name: nest.label.clone(),
                node_type: nest.substrate.clone(),
                family: "nucleus".into(),
                status: status.into(),
                health: if nest.available { 100 } else { 50 },
                confidence: 90,
                position: None,
                capabilities: vec![nest.substrate.clone()],
                data_channels: channels,
                clinical_ranges: vec![],
            };
            scenario.ecosystem.primals.push(scenario_node);
        }

        // Intra-node edges
        if topo_node.nests.len() > 1 {
            for i in 0..topo_node.nests.len() {
                for j in (i + 1)..topo_node.nests.len() {
                    let src = format!(
                        "T{}.N{}.D{}",
                        tower_id, topo_node.node_id, topo_node.nests[i].device_id
                    );
                    let dst = format!(
                        "T{}.N{}.D{}",
                        tower_id, topo_node.node_id, topo_node.nests[j].device_id
                    );
                    edges.push(edge(&src, &dst, &format!("PCIe {}", topo_node.pcie_gen)));
                }
            }
        }
    }

    // Transfer plan edges
    for transfer in transfers {
        edges.push(edge(&transfer.src_id, &transfer.dst_id, &transfer.label));
    }

    (scenario, edges)
}

/// Build a petalTongue scenario showing dispatch plan assignments.
///
/// Each stage becomes a node, with timing as a bar chart and substrate
/// assignment as a gauge.
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "element counts → f64 for gauge display"
)]
pub fn dispatch_scenario(
    name: &str,
    stages: &[DispatchStageInfo],
) -> (HealthScenario, Vec<ScenarioEdge>) {
    let mut scenario = scaffold(
        name,
        "Dispatch plan — stage assignments and timing across heterogeneous hardware",
    );
    let mut edges = Vec::new();

    let stage_names: Vec<String> = stages.iter().map(|s| s.name.clone()).collect();
    let stage_times: Vec<f64> = stages.iter().map(|s| s.elapsed_us).collect();

    // Summary node with timing bar chart
    let summary = node(
        "dispatch_summary",
        "Dispatch Plan",
        "orchestrator",
        &["compute.dispatch"],
        vec![bar(
            "stage_timing",
            "Stage Timing",
            stage_names,
            stage_times,
            "μs",
        )],
        vec![],
    );
    scenario.ecosystem.primals.push(summary);

    for (i, stage) in stages.iter().enumerate() {
        let stage_node = node(
            &format!("stage_{i}"),
            &stage.name,
            &stage.substrate,
            &["compute.execute"],
            vec![
                gauge(
                    &format!("stage_{i}_time"),
                    "Time",
                    stage.elapsed_us,
                    0.0,
                    stages.iter().map(|s| s.elapsed_us).fold(0.0_f64, f64::max) * 1.5,
                    "μs",
                    [0.0, stage.elapsed_us * 2.0],
                    [stage.elapsed_us * 2.0, stage.elapsed_us * 5.0],
                ),
                gauge(
                    &format!("stage_{i}_elements"),
                    "Output Elements",
                    stage.output_elements as f64,
                    0.0,
                    (stage.output_elements as f64 * 2.0).max(1.0),
                    "elements",
                    [0.0, stage.output_elements as f64 * 1.5],
                    [
                        stage.output_elements as f64 * 1.5,
                        stage.output_elements as f64 * 3.0,
                    ],
                ),
            ],
            vec![],
        );
        scenario.ecosystem.primals.push(stage_node);

        edges.push(edge("dispatch_summary", &format!("stage_{i}"), "dispatch"));
        if i > 0 {
            edges.push(edge(
                &format!("stage_{}", i - 1),
                &format!("stage_{i}"),
                "data-flow",
            ));
        }
    }

    (scenario, edges)
}

/// Description of a Nest for topology visualization.
pub struct TopologyNest {
    pub device_id: u16,
    pub substrate: String,
    pub label: String,
    pub memory_total_bytes: u64,
    pub memory_used_bytes: u64,
    pub utilization_pct: f64,
    pub available: bool,
}

/// Description of a Node for topology visualization.
pub struct TopologyNode {
    pub node_id: u16,
    pub pcie_gen: String,
    pub nests: Vec<TopologyNest>,
}

/// Description of a transfer for topology visualization.
pub struct TopologyTransfer {
    pub src_id: String,
    pub dst_id: String,
    pub label: String,
}

/// Description of a stage in a dispatch plan for visualization.
pub struct DispatchStageInfo {
    pub name: String,
    pub substrate: String,
    pub elapsed_us: f64,
    pub output_elements: usize,
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;

    fn test_topology() -> (Vec<TopologyNode>, Vec<TopologyTransfer>) {
        let nodes = vec![TopologyNode {
            node_id: 0,
            pcie_gen: "Gen4".to_string(),
            nests: vec![
                TopologyNest {
                    device_id: 0,
                    substrate: "cpu".to_string(),
                    label: "CPU (16 cores)".to_string(),
                    memory_total_bytes: 64 * 1024 * 1024 * 1024,
                    memory_used_bytes: 8 * 1024 * 1024 * 1024,
                    utilization_pct: 15.0,
                    available: true,
                },
                TopologyNest {
                    device_id: 1,
                    substrate: "gpu".to_string(),
                    label: "RTX 4090".to_string(),
                    memory_total_bytes: 24 * 1024 * 1024 * 1024,
                    memory_used_bytes: 2 * 1024 * 1024 * 1024,
                    utilization_pct: 42.0,
                    available: true,
                },
                TopologyNest {
                    device_id: 2,
                    substrate: "npu".to_string(),
                    label: "Akida AKD1000".to_string(),
                    memory_total_bytes: 256 * 1024 * 1024,
                    memory_used_bytes: 64 * 1024 * 1024,
                    utilization_pct: 0.0,
                    available: true,
                },
            ],
        }];
        let transfers = vec![TopologyTransfer {
            src_id: "T0.N0.D2".to_string(),
            dst_id: "T0.N0.D1".to_string(),
            label: "PCIe P2P 31.5 GB/s".to_string(),
        }];
        (nodes, transfers)
    }

    #[test]
    fn topology_scenario_creates_nodes() {
        let (nodes, transfers) = test_topology();
        let (scenario, edges) = topology_scenario(0, &nodes, &transfers);
        assert_eq!(scenario.ecosystem.primals.len(), 3);
        assert!(!edges.is_empty());
    }

    #[test]
    fn topology_scenario_node_ids() {
        let (nodes, transfers) = test_topology();
        let (scenario, _) = topology_scenario(0, &nodes, &transfers);
        let ids: Vec<&str> = scenario
            .ecosystem
            .primals
            .iter()
            .map(|n| n.id.as_str())
            .collect();
        assert!(ids.contains(&"T0.N0.D0"));
        assert!(ids.contains(&"T0.N0.D1"));
        assert!(ids.contains(&"T0.N0.D2"));
    }

    #[test]
    fn topology_scenario_has_gauges() {
        let (nodes, transfers) = test_topology();
        let (scenario, _) = topology_scenario(0, &nodes, &transfers);
        for primal in &scenario.ecosystem.primals {
            assert_eq!(primal.data_channels.len(), 2);
        }
    }

    #[test]
    fn dispatch_scenario_creates_stages() {
        let stages = vec![
            DispatchStageInfo {
                name: "biosignal".into(),
                substrate: "npu".into(),
                elapsed_us: 50.0,
                output_elements: 360,
            },
            DispatchStageInfo {
                name: "pop_pk".into(),
                substrate: "gpu".into(),
                elapsed_us: 1200.0,
                output_elements: 5000,
            },
            DispatchStageInfo {
                name: "diagnostic".into(),
                substrate: "cpu".into(),
                elapsed_us: 10.0,
                output_elements: 1,
            },
        ];
        let (scenario, edges) = dispatch_scenario("mixed_clinical", &stages);
        assert_eq!(scenario.ecosystem.primals.len(), 4);
        assert!(!edges.is_empty());
        assert_eq!(scenario.ecosystem.primals[0].id, "dispatch_summary");
    }

    #[test]
    fn dispatch_scenario_json_valid() {
        let stages = vec![DispatchStageInfo {
            name: "test".into(),
            substrate: "cpu".into(),
            elapsed_us: 100.0,
            output_elements: 50,
        }];
        let (scenario, edges) = dispatch_scenario("test", &stages);
        let json = crate::visualization::scenarios::scenario_with_edges_json(&scenario, &edges);
        let val: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert!(val.is_object());
    }
}
