// SPDX-License-Identifier: AGPL-3.0-or-later
//! NUCLEUS atomics — hierarchical compute topology for sovereign hardware.
//!
//! The NUCLEUS model organizes heterogeneous compute resources into three
//! concentric layers, each coordinated by biomeOS graphs:
//!
//! - **Nest**: Single device (one GPU, one NPU, or CPU cores).
//!   Smallest schedulable unit. All memory is device-local.
//!
//! - **Node**: Single machine with multiple Nests connected via PCIe/NVLink.
//!   Nests within a Node can transfer data without network overhead.
//!
//! - **Tower**: Cluster of Nodes connected via network fabric.
//!   Inter-Node transfers use IPC (Unix sockets, shared memory).
//!
//! ## Example Topology
//!
//! ```text
//! Tower (cluster)
//! ├── Node 0 (workstation)
//! │   ├── Nest 0: CPU (16 cores, 64 GB)
//! │   ├── Nest 1: GPU (RTX 4090, 24 GB VRAM)
//! │   └── Nest 2: NPU (Akida AKD1000, 1.2 TOPS)
//! └── Node 1 (headless server)
//!     ├── Nest 0: CPU (32 cores, 128 GB)
//!     └── Nest 1: GPU (A100, 80 GB VRAM)
//! ```

use crate::Substrate;

/// Nest — a single compute device within a Node.
///
/// The atomic unit of hardware. Each Nest owns exclusive device memory
/// and can execute one workload kernel at a time.
#[derive(Debug, Clone)]
pub struct Nest {
    pub id: NestId,
    pub substrate: Substrate,
    pub memory_bytes: u64,
    pub status: DeviceStatus,
}

/// Unique identifier for a Nest within the Tower topology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NestId {
    pub tower: u16,
    pub node: u16,
    pub device: u16,
}

impl std::fmt::Display for NestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "T{}.N{}.D{}", self.tower, self.node, self.device)
    }
}

/// Node — a single machine hosting one or more Nests.
///
/// Nests within a Node share the same `PCIe` bus and can perform
/// peer-to-peer DMA transfers without CPU involvement.
#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub nests: Vec<Nest>,
    pub pcie_gen: PcieGeneration,
}

/// Unique identifier for a Node within the Tower.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId {
    pub tower: u16,
    pub node: u16,
}

/// Tower — a cluster of Nodes connected via network fabric.
#[derive(Debug, Clone)]
pub struct Tower {
    pub id: u16,
    pub nodes: Vec<Node>,
}

/// Runtime status of a compute device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceStatus {
    /// Ready to accept workloads.
    Available,
    /// Currently executing a workload.
    Busy,
    /// Device detected but not initialized.
    Uninitialized,
    /// Device failed health check.
    Faulted,
}

/// `PCIe` generation determines peer-to-peer transfer bandwidth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PcieGeneration {
    Gen3,
    Gen4,
    Gen5,
}

impl PcieGeneration {
    /// Theoretical per-lane bandwidth in GB/s.
    #[must_use]
    pub const fn lane_bandwidth_gbps(self) -> f64 {
        match self {
            Self::Gen3 => 0.985,
            Self::Gen4 => 1.969,
            Self::Gen5 => 3.938,
        }
    }
}

impl Node {
    /// List Nests that match a given substrate type.
    #[must_use]
    pub fn nests_by_substrate(&self, substrate: Substrate) -> Vec<&Nest> {
        self.nests
            .iter()
            .filter(|n| n.substrate == substrate)
            .collect()
    }

    /// Check whether two Nests on this Node can do peer-to-peer DMA
    /// (bypassing CPU roundtrip).
    #[must_use]
    pub fn can_p2p_transfer(&self, src: &NestId, dst: &NestId) -> bool {
        let src_exists = self.nests.iter().any(|n| n.id == *src);
        let dst_exists = self.nests.iter().any(|n| n.id == *dst);
        let same_node = src.tower == dst.tower && src.node == dst.node;
        let different_device = src.device != dst.device;
        src_exists && dst_exists && same_node && different_device
    }

    /// Count available (non-busy, non-faulted) Nests.
    #[must_use]
    pub fn available_nests(&self) -> usize {
        self.nests
            .iter()
            .filter(|n| n.status == DeviceStatus::Available)
            .count()
    }
}

impl Tower {
    /// Total number of Nests across all Nodes.
    #[must_use]
    pub fn total_nests(&self) -> usize {
        self.nodes.iter().map(|n| n.nests.len()).sum()
    }

    /// Discover all available substrates across the Tower.
    #[must_use]
    pub fn available_substrates(&self) -> Vec<Substrate> {
        let mut subs = Vec::new();
        for node in &self.nodes {
            for nest in &node.nests {
                if nest.status == DeviceStatus::Available && !subs.contains(&nest.substrate) {
                    subs.push(nest.substrate);
                }
            }
        }
        subs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_node() -> Node {
        Node {
            id: NodeId { tower: 0, node: 0 },
            nests: vec![
                Nest {
                    id: NestId {
                        tower: 0,
                        node: 0,
                        device: 0,
                    },
                    substrate: Substrate::Cpu,
                    memory_bytes: 64 * 1024 * 1024 * 1024,
                    status: DeviceStatus::Available,
                },
                Nest {
                    id: NestId {
                        tower: 0,
                        node: 0,
                        device: 1,
                    },
                    substrate: Substrate::Gpu,
                    memory_bytes: 24 * 1024 * 1024 * 1024,
                    status: DeviceStatus::Available,
                },
                Nest {
                    id: NestId {
                        tower: 0,
                        node: 0,
                        device: 2,
                    },
                    substrate: Substrate::Npu,
                    memory_bytes: 256 * 1024 * 1024,
                    status: DeviceStatus::Available,
                },
            ],
            pcie_gen: PcieGeneration::Gen4,
        }
    }

    #[test]
    fn nest_id_display() {
        let id = NestId {
            tower: 0,
            node: 1,
            device: 2,
        };
        assert_eq!(format!("{id}"), "T0.N1.D2");
    }

    #[test]
    fn nests_by_substrate_filters() {
        let node = test_node();
        assert_eq!(node.nests_by_substrate(Substrate::Gpu).len(), 1);
        assert_eq!(node.nests_by_substrate(Substrate::Cpu).len(), 1);
        assert_eq!(node.nests_by_substrate(Substrate::Npu).len(), 1);
    }

    #[test]
    fn p2p_transfer_same_node_different_device() {
        let node = test_node();
        let gpu = NestId {
            tower: 0,
            node: 0,
            device: 1,
        };
        let npu = NestId {
            tower: 0,
            node: 0,
            device: 2,
        };
        assert!(node.can_p2p_transfer(&gpu, &npu));
    }

    #[test]
    fn p2p_transfer_same_device_rejected() {
        let node = test_node();
        let gpu = NestId {
            tower: 0,
            node: 0,
            device: 1,
        };
        assert!(!node.can_p2p_transfer(&gpu, &gpu));
    }

    #[test]
    fn p2p_transfer_different_node_rejected() {
        let node = test_node();
        let local_gpu = NestId {
            tower: 0,
            node: 0,
            device: 1,
        };
        let remote_gpu = NestId {
            tower: 0,
            node: 1,
            device: 1,
        };
        assert!(!node.can_p2p_transfer(&local_gpu, &remote_gpu));
    }

    #[test]
    fn available_nests_count() {
        let node = test_node();
        assert_eq!(node.available_nests(), 3);
    }

    #[test]
    fn tower_total_nests() {
        let tower = Tower {
            id: 0,
            nodes: vec![test_node()],
        };
        assert_eq!(tower.total_nests(), 3);
    }

    #[test]
    fn tower_available_substrates() {
        let tower = Tower {
            id: 0,
            nodes: vec![test_node()],
        };
        let subs = tower.available_substrates();
        assert!(subs.contains(&Substrate::Cpu));
        assert!(subs.contains(&Substrate::Gpu));
        assert!(subs.contains(&Substrate::Npu));
    }

    #[test]
    fn pcie_gen4_bandwidth() {
        let bw = PcieGeneration::Gen4.lane_bandwidth_gbps();
        assert!((bw - 1.969).abs() < 1e-6);
    }
}
