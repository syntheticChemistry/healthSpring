// SPDX-License-Identifier: AGPL-3.0-only
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

//! Exp062: `PCIe` P2P transfer validation with realistic healthSpring workloads.
//!
//! Exercises `transfer.rs` with actual data sizes from healthSpring pipelines,
//! validates `TransferMethod` selection and bandwidth calculations across
//! `PCIe` Gen3/4/5 topologies.

use healthspring_forge::Substrate;
use healthspring_forge::nucleus::{DeviceStatus, Nest, NestId, Node, NodeId, PcieGeneration};
use healthspring_forge::transfer::{TransferMethod, plan_transfer};

fn check(name: &str, ok: bool, passed: &mut u32, total: &mut u32) {
    *total += 1;
    if ok {
        *passed += 1;
        println!("  [PASS] {name}");
    } else {
        println!("  [FAIL] {name}");
    }
}

fn make_node(pcie: PcieGeneration) -> Node {
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
        pcie_gen: pcie,
    }
}

const GPU: NestId = NestId {
    tower: 0,
    node: 0,
    device: 1,
};
const NPU: NestId = NestId {
    tower: 0,
    node: 0,
    device: 2,
};
const CPU: NestId = NestId {
    tower: 0,
    node: 0,
    device: 0,
};
const REMOTE_GPU: NestId = NestId {
    tower: 0,
    node: 1,
    device: 1,
};

#[expect(
    clippy::too_many_lines,
    clippy::similar_names,
    reason = "validation binary — PCIe transfer plan checks; gpu_npu/npu_cpu denote distinct transfer directions"
)]
fn main() {
    struct WorkloadTransfer {
        name: &'static str,
        bytes: u64,
        description: &'static str,
    }

    println!("Exp062 PCIe P2P Transfer Validation");
    println!("====================================\n");

    let mut passed = 0u32;
    let mut total = 0u32;

    // -----------------------------------------------------------------------
    // Section 1: TransferMethod selection
    // -----------------------------------------------------------------------
    println!("--- Section 1: Transfer method selection ---");

    let node_gen4 = make_node(PcieGeneration::Gen4);

    let same_dev = plan_transfer(GPU, GPU, 1024, Some(&node_gen4));
    check(
        "same device -> None",
        same_dev.method == TransferMethod::None,
        &mut passed,
        &mut total,
    );

    let gpu_npu = plan_transfer(GPU, NPU, 1024, Some(&node_gen4));
    check(
        "GPU->NPU same node -> PcieP2p",
        gpu_npu.method == TransferMethod::PcieP2p,
        &mut passed,
        &mut total,
    );

    let npu_cpu = plan_transfer(NPU, CPU, 1024, Some(&node_gen4));
    check(
        "NPU->CPU same node -> PcieP2p",
        npu_cpu.method == TransferMethod::PcieP2p,
        &mut passed,
        &mut total,
    );

    let gpu_cpu = plan_transfer(GPU, CPU, 1024, Some(&node_gen4));
    check(
        "GPU->CPU same node -> PcieP2p",
        gpu_cpu.method == TransferMethod::PcieP2p,
        &mut passed,
        &mut total,
    );

    let no_node = plan_transfer(GPU, NPU, 1024, None);
    check(
        "no node info -> HostStaged",
        no_node.method == TransferMethod::HostStaged,
        &mut passed,
        &mut total,
    );

    let cross_node = plan_transfer(GPU, REMOTE_GPU, 1024, Some(&node_gen4));
    check(
        "cross-node -> NetworkIpc",
        cross_node.method == TransferMethod::NetworkIpc,
        &mut passed,
        &mut total,
    );

    // -----------------------------------------------------------------------
    // Section 2: Realistic healthSpring workload sizes
    // -----------------------------------------------------------------------
    println!("\n--- Section 2: Realistic workload transfer sizes ---");

    let workloads = [
        WorkloadTransfer {
            name: "PopPK 10K patients",
            bytes: 10_000 * 8,
            description: "AUC f64 per patient",
        },
        WorkloadTransfer {
            name: "PopPK 1M patients",
            bytes: 1_000_000 * 8,
            description: "AUC f64 per patient (scaling)",
        },
        WorkloadTransfer {
            name: "Diversity 1K communities",
            bytes: 1_000 * 16,
            description: "(shannon, simpson) f64 pairs",
        },
        WorkloadTransfer {
            name: "Biosignal 256 Hz * 60s",
            bytes: 256 * 60 * 8,
            description: "1 minute ECG stream",
        },
        WorkloadTransfer {
            name: "Hill 100K concentrations",
            bytes: 100_000 * 8,
            description: "dose-response output",
        },
    ];

    for w in &workloads {
        let plan = plan_transfer(GPU, NPU, w.bytes, Some(&node_gen4));
        let time_us = plan.estimated_time_us();
        println!(
            "  {}: {} bytes ({}) -> {:.3} us via {:?}",
            w.name, w.bytes, w.description, time_us, plan.method,
        );
        check(
            &format!("{}: transfer time > 0", w.name),
            time_us > 0.0,
            &mut passed,
            &mut total,
        );
        check(
            &format!("{}: bandwidth > 0", w.name),
            plan.estimated_bandwidth_gbps > 0.0,
            &mut passed,
            &mut total,
        );
    }

    // -----------------------------------------------------------------------
    // Section 3: PCIe generation bandwidth comparison
    // -----------------------------------------------------------------------
    println!("\n--- Section 3: PCIe Gen3/4/5 bandwidth comparison ---");

    let bytes_1mb: u64 = 1_000_000;
    let mut prev_time = f64::MAX;

    for (pcie_gen, node) in [
        (PcieGeneration::Gen3, make_node(PcieGeneration::Gen3)),
        (PcieGeneration::Gen4, make_node(PcieGeneration::Gen4)),
        (PcieGeneration::Gen5, make_node(PcieGeneration::Gen5)),
    ] {
        let plan = plan_transfer(GPU, NPU, bytes_1mb, Some(&node));
        let time_us = plan.estimated_time_us();
        let bw_gbps = plan.estimated_bandwidth_gbps;
        println!("  {pcie_gen:?}: {bw_gbps:.1} GB/s x16 lanes, 1 MB in {time_us:.2} us",);

        check(
            &format!("{pcie_gen:?}: faster than previous gen"),
            time_us < prev_time,
            &mut passed,
            &mut total,
        );
        prev_time = time_us;
    }

    let gen3_lane = PcieGeneration::Gen3.lane_bandwidth_gbps();
    let gen4_lane = PcieGeneration::Gen4.lane_bandwidth_gbps();
    let gen5_lane = PcieGeneration::Gen5.lane_bandwidth_gbps();

    check(
        "Gen4 ~2x Gen3",
        (gen4_lane / gen3_lane - 2.0).abs() < 0.1,
        &mut passed,
        &mut total,
    );
    check(
        "Gen5 ~2x Gen4",
        (gen5_lane / gen4_lane - 2.0).abs() < 0.1,
        &mut passed,
        &mut total,
    );

    // -----------------------------------------------------------------------
    // Section 4: Zero-byte and same-device edge cases
    // -----------------------------------------------------------------------
    println!("\n--- Section 4: Edge cases ---");

    let zero_plan = plan_transfer(GPU, NPU, 0, Some(&node_gen4));
    check(
        "0 bytes -> 0 time",
        zero_plan.estimated_time_us().abs() < 1e-15,
        &mut passed,
        &mut total,
    );

    let same_plan = plan_transfer(CPU, CPU, 1_000_000, Some(&node_gen4));
    check(
        "same device -> infinite bw",
        same_plan.estimated_bandwidth_gbps.is_infinite(),
        &mut passed,
        &mut total,
    );
    check(
        "same device -> 0 time",
        same_plan.estimated_time_us().abs() < 1e-15,
        &mut passed,
        &mut total,
    );

    // -----------------------------------------------------------------------
    // Section 5: Transfer overhead vs compute time ratio
    // -----------------------------------------------------------------------
    println!("\n--- Section 5: Transfer overhead analysis ---");

    let pop_pk_compute_us = 300.0;
    let pop_pk_transfer = plan_transfer(GPU, CPU, 10_000 * 8, Some(&node_gen4));
    let overhead_pct = (pop_pk_transfer.estimated_time_us() / pop_pk_compute_us) * 100.0;
    println!(
        "  PopPK 10K: compute ~{pop_pk_compute_us:.0}us, transfer {:.3}us ({overhead_pct:.4}%)",
        pop_pk_transfer.estimated_time_us(),
    );
    check(
        "PopPK transfer < 1% of compute",
        overhead_pct < 1.0,
        &mut passed,
        &mut total,
    );

    let hill_compute_us = 500.0;
    let hill_transfer = plan_transfer(GPU, CPU, 100_000 * 8, Some(&node_gen4));
    let hill_pct = (hill_transfer.estimated_time_us() / hill_compute_us) * 100.0;
    println!(
        "  Hill 100K: compute ~{hill_compute_us:.0}us, transfer {:.3}us ({hill_pct:.4}%)",
        hill_transfer.estimated_time_us(),
    );
    check(
        "Hill 100K transfer < 6% of compute",
        hill_pct < 6.0,
        &mut passed,
        &mut total,
    );

    // -----------------------------------------------------------------------
    // Summary
    // -----------------------------------------------------------------------
    println!("\n====================================");
    println!("Exp062 PCIe Transfer: {passed}/{total} checks passed");
    std::process::exit(i32::from(passed != total));
}
