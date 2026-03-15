// SPDX-License-Identifier: AGPL-3.0-only
#![deny(clippy::all)]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — linear check sequence"
)]

//! Exp070: `PCIe` P2P bypass validation — verifies NPU-to-GPU direct transfer
//! path selection, bandwidth estimation, and zero-copy detection.

use healthspring_forge::Substrate;
use healthspring_forge::nucleus::{
    DeviceStatus, Nest, NestId, Node, NodeId, PcieGeneration, Tower,
};
use healthspring_forge::transfer::{TransferMethod, plan_transfer};

fn main() {
    let mut passed = 0u32;
    let mut failed = 0u32;

    macro_rules! check {
        ($name:expr, $cond:expr) => {
            if $cond {
                passed += 1;
                println!("  [PASS] {}", $name);
            } else {
                eprintln!("  [FAIL] {}", $name);
                failed += 1;
            }
        };
    }

    println!("Exp070: PCIe P2P Bypass Validation");
    println!("====================================");

    // Build a Node with GPU and NPU on the same PCIe bus
    let gpu_nest = NestId {
        tower: 0,
        node: 0,
        device: 1,
    };
    let npu_nest = NestId {
        tower: 0,
        node: 0,
        device: 2,
    };
    let cpu_nest = NestId {
        tower: 0,
        node: 0,
        device: 0,
    };

    let node_gen4 = Node {
        id: NodeId { tower: 0, node: 0 },
        nests: vec![
            Nest {
                id: cpu_nest,
                substrate: Substrate::Cpu,
                memory_bytes: 64 * 1024 * 1024 * 1024,
                status: DeviceStatus::Available,
            },
            Nest {
                id: gpu_nest,
                substrate: Substrate::Gpu,
                memory_bytes: 24 * 1024 * 1024 * 1024,
                status: DeviceStatus::Available,
            },
            Nest {
                id: npu_nest,
                substrate: Substrate::Npu,
                memory_bytes: 256 * 1024 * 1024,
                status: DeviceStatus::Available,
            },
        ],
        pcie_gen: PcieGeneration::Gen4,
    };

    // --- P2P selected over host-staged ---
    println!("\n=== P2P Transfer Selection ===");
    let plan = plan_transfer(npu_nest, gpu_nest, 1_000_000, Some(&node_gen4));
    check!(
        "npu_to_gpu_p2p_selected",
        plan.method == TransferMethod::PcieP2p
    );
    check!(
        "p2p_bypasses_cpu",
        plan.method != TransferMethod::HostStaged
    );

    let plan_rev = plan_transfer(gpu_nest, npu_nest, 1_000_000, Some(&node_gen4));
    check!(
        "gpu_to_npu_p2p_selected",
        plan_rev.method == TransferMethod::PcieP2p
    );

    let plan_cpu_gpu = plan_transfer(cpu_nest, gpu_nest, 1_000_000, Some(&node_gen4));
    check!(
        "cpu_to_gpu_p2p_selected",
        plan_cpu_gpu.method == TransferMethod::PcieP2p
    );

    // Without node topology info, falls back to host-staged
    let plan_no_node = plan_transfer(npu_nest, gpu_nest, 1_000_000, None);
    check!(
        "no_topology_falls_to_host_staged",
        plan_no_node.method == TransferMethod::HostStaged
    );

    // Same device → no transfer
    let plan_same = plan_transfer(gpu_nest, gpu_nest, 1024, Some(&node_gen4));
    check!(
        "same_device_no_transfer",
        plan_same.method == TransferMethod::None
    );

    // --- Bandwidth estimation for Gen4 ---
    println!("\n=== Gen4 Bandwidth ===");
    let gen4_bw = PcieGeneration::Gen4.lane_bandwidth_gbps();
    let gen4_16x = gen4_bw * 16.0;
    check!(
        &format!("gen4_16x_bandwidth_{gen4_16x:.1}_gbps"),
        (gen4_16x - 31.504).abs() < 0.1
    );
    check!(
        "p2p_bandwidth_matches_gen4_16x",
        (plan.estimated_bandwidth_gbps - gen4_16x).abs() < 0.1
    );

    // --- Gen3 and Gen5 bandwidth ---
    println!("\n=== Gen3/Gen5 Bandwidth ===");
    let gen3_bw = PcieGeneration::Gen3.lane_bandwidth_gbps() * 16.0;
    let gen5_bw = PcieGeneration::Gen5.lane_bandwidth_gbps() * 16.0;
    check!(
        &format!("gen3_16x_{gen3_bw:.1}_gbps"),
        (gen3_bw - 15.76).abs() < 0.1
    );
    check!(
        &format!("gen5_16x_{gen5_bw:.1}_gbps"),
        (gen5_bw - 63.008).abs() < 0.1
    );
    check!(
        "gen5_gt_gen4_gt_gen3",
        gen5_bw > gen4_16x && gen4_16x > gen3_bw
    );

    // Gen5 node transfer
    let node_gen5 = Node {
        id: NodeId { tower: 0, node: 0 },
        nests: node_gen4.nests.clone(),
        pcie_gen: PcieGeneration::Gen5,
    };
    let plan_gen5 = plan_transfer(npu_nest, gpu_nest, 1_000_000, Some(&node_gen5));
    check!(
        "gen5_p2p_faster",
        plan_gen5.estimated_bandwidth_gbps > plan.estimated_bandwidth_gbps
    );

    // --- Transfer time estimation ---
    println!("\n=== Transfer Time ===");
    let data_size: u64 = 100 * 1024 * 1024; // 100 MB
    let plan_100mb = plan_transfer(npu_nest, gpu_nest, data_size, Some(&node_gen4));
    let time_us = plan_100mb.estimated_time_us();
    check!(
        &format!("100mb_transfer_time_{time_us:.0}us"),
        time_us > 1000.0 && time_us < 100_000.0
    );

    // Zero bytes → zero time
    let plan_zero = plan_transfer(npu_nest, gpu_nest, 0, Some(&node_gen4));
    check!(
        "zero_bytes_zero_time",
        plan_zero.estimated_time_us().abs() < 1e-15
    );

    // --- Zero-copy path (same Node, shared bus) ---
    println!("\n=== Zero-Copy Path ===");
    check!(
        "p2p_is_zero_copy_path",
        plan.method == TransferMethod::PcieP2p
    );
    check!(
        "nests_share_node",
        node_gen4.can_p2p_transfer(&npu_nest, &gpu_nest)
    );

    // --- Cross-node (different Node) → network IPC ---
    println!("\n=== Cross-Node Transfer ===");
    let remote_gpu = NestId {
        tower: 0,
        node: 1,
        device: 1,
    };
    let plan_cross = plan_transfer(npu_nest, remote_gpu, 1_000_000, Some(&node_gen4));
    check!(
        "cross_node_network_ipc",
        plan_cross.method == TransferMethod::NetworkIpc
    );

    // --- Tower topology validation ---
    println!("\n=== Tower Topology ===");
    let tower = Tower {
        id: 0,
        nodes: vec![node_gen4, {
            Node {
                id: NodeId { tower: 0, node: 1 },
                nests: vec![
                    Nest {
                        id: NestId {
                            tower: 0,
                            node: 1,
                            device: 0,
                        },
                        substrate: Substrate::Cpu,
                        memory_bytes: 128 * 1024 * 1024 * 1024,
                        status: DeviceStatus::Available,
                    },
                    Nest {
                        id: NestId {
                            tower: 0,
                            node: 1,
                            device: 1,
                        },
                        substrate: Substrate::Gpu,
                        memory_bytes: 80 * 1024 * 1024 * 1024,
                        status: DeviceStatus::Available,
                    },
                ],
                pcie_gen: PcieGeneration::Gen5,
            }
        }],
    };
    check!("tower_5_nests", tower.total_nests() == 5);
    let subs = tower.available_substrates();
    check!("tower_has_cpu", subs.contains(&Substrate::Cpu));
    check!("tower_has_gpu", subs.contains(&Substrate::Gpu));
    check!("tower_has_npu", subs.contains(&Substrate::Npu));

    let total = passed + failed;
    println!("\n====================================");
    println!("Exp070 PCIe P2P Bypass: {passed}/{total} checks passed");
    std::process::exit(i32::from(passed != total));
}
