// SPDX-License-Identifier: AGPL-3.0-or-later
//! Inter-device data transfer planning for mixed hardware systems.
//!
//! When a pipeline stage on one Nest produces output consumed by a stage
//! on a different Nest, metalForge plans the optimal transfer path:
//!
//! - **`PCIe` P2P DMA**: Fastest. GPU↔NPU on the same Node, bypassing CPU.
//!   Requires both devices on the same `PCIe` root complex.
//!
//! - **Staged via host**: CPU mediates. Device A → host RAM → Device B.
//!   Used when P2P is unavailable (different IOMMU groups, etc.).
//!
//! - **Network IPC**: Cross-Node transfers via Unix sockets or shared memory.
//!   Managed by biomeOS graph edges, not metalForge directly.

use crate::nucleus::{NestId, Node, PcieGeneration};

/// Transfer method between two Nests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferMethod {
    /// Direct `PCIe` peer-to-peer DMA (no CPU roundtrip).
    PcieP2p,
    /// Staged through host CPU memory.
    HostStaged,
    /// Cross-node network transfer (IPC via biomeOS).
    NetworkIpc,
    /// No transfer needed — source and destination are the same device.
    None,
}

/// Planned data transfer between two Nests.
#[derive(Debug, Clone)]
pub struct TransferPlan {
    /// Producer nest for this handoff.
    pub src: NestId,
    /// Consumer nest for the next pipeline stage.
    pub dst: NestId,
    /// `PCIe` P2P, host round-trip, IPC, or no-op when `src == dst`.
    pub method: TransferMethod,
    /// Effective throughput used for `estimated_time_us` (conservative where unknown).
    pub estimated_bandwidth_gbps: f64,
    /// Payload size attributed to this edge (stage output bytes).
    pub bytes: u64,
}

impl TransferPlan {
    /// Estimated transfer time in microseconds.
    #[must_use]
    pub fn estimated_time_us(&self) -> f64 {
        if self.estimated_bandwidth_gbps <= 0.0 || self.bytes == 0 {
            return 0.0;
        }
        #[expect(clippy::cast_precision_loss, reason = "byte counts ≪ 2^52")]
        let bytes_f64 = self.bytes as f64;
        let gbps = self.estimated_bandwidth_gbps;
        (bytes_f64 / (gbps * 1e9)) * 1e6
    }
}

/// Plan the optimal transfer method between two Nests.
///
/// Evaluates whether P2P DMA is possible; falls back to host-staged
/// or network IPC based on topology.
#[must_use]
pub fn plan_transfer(
    src: NestId,
    dst: NestId,
    bytes: u64,
    local_node: Option<&Node>,
) -> TransferPlan {
    if src == dst {
        return TransferPlan {
            src,
            dst,
            method: TransferMethod::None,
            estimated_bandwidth_gbps: f64::INFINITY,
            bytes,
        };
    }

    let same_node = src.tower == dst.tower && src.node == dst.node;

    if same_node {
        if let Some(node) = local_node {
            if node.can_p2p_transfer(&src, &dst) {
                let lanes = 16_u32;
                let bw = node.pcie_gen.lane_bandwidth_gbps() * f64::from(lanes);
                return TransferPlan {
                    src,
                    dst,
                    method: TransferMethod::PcieP2p,
                    estimated_bandwidth_gbps: bw,
                    bytes,
                };
            }
        }

        let host_bw = PcieGeneration::Gen4.lane_bandwidth_gbps() * 16.0 * 0.5;
        TransferPlan {
            src,
            dst,
            method: TransferMethod::HostStaged,
            estimated_bandwidth_gbps: host_bw,
            bytes,
        }
    } else {
        // Conservative cross-node estimate. biomeOS or the caller should
        // override via actual link probing when the network topology is known.
        const DEFAULT_NETWORK_BW_GBPS: f64 = 1.0;
        TransferPlan {
            src,
            dst,
            method: TransferMethod::NetworkIpc,
            estimated_bandwidth_gbps: DEFAULT_NETWORK_BW_GBPS,
            bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Substrate;
    use crate::nucleus::{DeviceStatus, Nest, Node, NodeId};

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
    fn same_device_no_transfer() {
        let id = NestId {
            tower: 0,
            node: 0,
            device: 1,
        };
        let plan = plan_transfer(id, id, 1024, None);
        assert_eq!(plan.method, TransferMethod::None);
    }

    #[test]
    fn gpu_to_npu_p2p_on_same_node() {
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
        let plan = plan_transfer(gpu, npu, 1_000_000, Some(&node));
        assert_eq!(plan.method, TransferMethod::PcieP2p);
        assert!(plan.estimated_bandwidth_gbps > 20.0, "16 lanes Gen4");
    }

    #[test]
    fn same_node_fallback_to_host_staged() {
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
        let plan = plan_transfer(gpu, npu, 1024, None);
        assert_eq!(plan.method, TransferMethod::HostStaged);
    }

    #[test]
    fn cross_node_uses_network_ipc() {
        let local = NestId {
            tower: 0,
            node: 0,
            device: 1,
        };
        let remote = NestId {
            tower: 0,
            node: 1,
            device: 0,
        };
        let plan = plan_transfer(local, remote, 4096, None);
        assert_eq!(plan.method, TransferMethod::NetworkIpc);
    }

    #[test]
    fn transfer_time_calculation() {
        let plan = TransferPlan {
            src: NestId {
                tower: 0,
                node: 0,
                device: 0,
            },
            dst: NestId {
                tower: 0,
                node: 0,
                device: 1,
            },
            method: TransferMethod::PcieP2p,
            estimated_bandwidth_gbps: 31.504,
            bytes: 31_504_000,
        };
        let time = plan.estimated_time_us();
        assert!(
            (time - 1000.0).abs() < 1.0,
            "~1 ms for 31.5 MB at 31.5 GB/s, got {time}"
        );
    }

    #[test]
    fn zero_bytes_zero_time() {
        let plan = TransferPlan {
            src: NestId {
                tower: 0,
                node: 0,
                device: 0,
            },
            dst: NestId {
                tower: 0,
                node: 0,
                device: 1,
            },
            method: TransferMethod::PcieP2p,
            estimated_bandwidth_gbps: 31.504,
            bytes: 0,
        };
        assert!(plan.estimated_time_us().abs() < 1e-15);
    }
}
