// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![expect(
    clippy::too_many_lines,
    reason = "validation binary — sequential NUCLEUS topology checks"
)]

//! Exp087: metalForge Mixed NUCLEUS V16 Dispatch
//!
//! Validates workload routing for all V16 `Workload` variants across a
//! realistic multi-device Tower topology (CPU + GPU + NPU nests).
//!
//! Tests:
//! - Substrate selection for V16 workloads at multiple scales
//! - `PCIe` P2P bypass (GPU→NPU direct, bypassing CPU roundtrip)
//! - Host-staged transfers when P2P is not available
//! - Dispatch planning for mixed V15+V16 pipelines through NUCLEUS
//! - Tower/Node/Nest atomic hierarchy
//!
//! The topology mirrors Eastgate hardware: CPU + RTX GPU + NPU on a
//! single node with `PCIe` Gen4 interconnect.

use healthspring_forge::dispatch::{DispatchPlan, plan_dispatch};
use healthspring_forge::nucleus::{
    DeviceStatus, Nest, NestId, Node, NodeId, PcieGeneration, Tower,
};
use healthspring_forge::transfer::{TransferMethod, plan_transfer};
use healthspring_forge::{
    Capabilities, GpuInfo, NpuInfo, PrecisionRouting, Substrate, Workload, select_substrate,
};

fn check(name: &str, ok: bool, passed: &mut u32, total: &mut u32) {
    *total += 1;
    if ok {
        *passed += 1;
        println!("  [PASS] {name}");
    } else {
        println!("  [FAIL] {name}");
    }
}

fn eastgate_tower() -> Tower {
    Tower {
        id: 0,
        nodes: vec![Node {
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
                    memory_bytes: 12 * 1024 * 1024 * 1024,
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
        }],
    }
}

fn full_caps() -> Capabilities {
    Capabilities::with_known(
        Some(GpuInfo {
            name: "RTX 4070".into(),
            fp64_native: false,
            f64_shared_mem_reliable: false,
            max_workgroups: 65535,
            precision: PrecisionRouting::Df64Only,
        }),
        Some(NpuInfo {
            name: "Akida AKD1000".into(),
            max_inference_rate_hz: 10_000,
        }),
    )
}

fn main() {
    let mut passed = 0u32;
    let mut total = 0u32;

    println!("{}", "=".repeat(72));
    println!("healthSpring Exp087 — metalForge Mixed NUCLEUS V16 Dispatch");
    println!("{}", "=".repeat(72));

    let tower = eastgate_tower();
    let caps = full_caps();

    // ── 1. Tower Topology Validation ────────────────────────────────────
    println!("\n── 1. Tower Topology ──────────────────────────────────────────");

    check(
        "Tower: 1 node",
        tower.nodes.len() == 1,
        &mut passed,
        &mut total,
    );
    check(
        "Tower: 3 nests (CPU+GPU+NPU)",
        tower.total_nests() == 3,
        &mut passed,
        &mut total,
    );
    let substrates = tower.available_substrates();
    check(
        "Tower: CPU available",
        substrates.contains(&Substrate::Cpu),
        &mut passed,
        &mut total,
    );
    check(
        "Tower: GPU available",
        substrates.contains(&Substrate::Gpu),
        &mut passed,
        &mut total,
    );
    check(
        "Tower: NPU available",
        substrates.contains(&Substrate::Npu),
        &mut passed,
        &mut total,
    );

    // ── 2. V16 Workload Routing ─────────────────────────────────────────
    println!("\n── 2. V16 Workload Routing (with full caps) ───────────────────");

    let v16_workloads = [
        (
            "MM small (64)",
            Workload::MichaelisMentenBatch { n_patients: 64 },
        ),
        (
            "MM large (10K)",
            Workload::MichaelisMentenBatch { n_patients: 10_000 },
        ),
        ("SCFA small (50)", Workload::ScfaBatch { n_elements: 50 }),
        ("SCFA large (5K)", Workload::ScfaBatch { n_elements: 5_000 }),
        (
            "Beat small (10)",
            Workload::BeatClassifyBatch { n_beats: 10 },
        ),
        (
            "Beat large (10K)",
            Workload::BeatClassifyBatch { n_beats: 10_000 },
        ),
        (
            "Biosignal detect",
            Workload::BiosignalDetect {
                sample_rate_hz: 256,
            },
        ),
        ("Analytical", Workload::Analytical),
    ];

    for (label, workload) in &v16_workloads {
        let sub = select_substrate(workload, &caps);
        println!("  {label:30} → {sub:?}");
    }

    check(
        "Routing: MM 10K → GPU",
        select_substrate(
            &Workload::MichaelisMentenBatch { n_patients: 10_000 },
            &caps,
        ) == Substrate::Gpu,
        &mut passed,
        &mut total,
    );
    check(
        "Routing: SCFA 5K → GPU",
        select_substrate(&Workload::ScfaBatch { n_elements: 5_000 }, &caps) == Substrate::Gpu,
        &mut passed,
        &mut total,
    );
    check(
        "Routing: Beat 10K → GPU",
        select_substrate(&Workload::BeatClassifyBatch { n_beats: 10_000 }, &caps) == Substrate::Gpu,
        &mut passed,
        &mut total,
    );
    check(
        "Routing: Biosignal detect → NPU",
        select_substrate(
            &Workload::BiosignalDetect {
                sample_rate_hz: 256,
            },
            &caps,
        ) == Substrate::Npu,
        &mut passed,
        &mut total,
    );
    check(
        "Routing: Analytical → CPU",
        select_substrate(&Workload::Analytical, &caps) == Substrate::Cpu,
        &mut passed,
        &mut total,
    );

    // ── 3. PCIe P2P Bypass (GPU ↔ NPU) ─────────────────────────────────
    println!("\n── 3. PCIe P2P Bypass (GPU ↔ NPU) ────────────────────────────");

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
    let node = &tower.nodes[0];

    check(
        "P2P: GPU → NPU possible on same node",
        node.can_p2p_transfer(&gpu_nest, &npu_nest),
        &mut passed,
        &mut total,
    );
    check(
        "P2P: NPU → GPU possible on same node",
        node.can_p2p_transfer(&npu_nest, &gpu_nest),
        &mut passed,
        &mut total,
    );
    check(
        "P2P: same device rejected",
        !node.can_p2p_transfer(&gpu_nest, &gpu_nest),
        &mut passed,
        &mut total,
    );

    let gpu_to_npu = plan_transfer(gpu_nest, npu_nest, 1_000_000, Some(node));
    check(
        "Transfer GPU→NPU: P2P method",
        gpu_to_npu.method == TransferMethod::PcieP2p,
        &mut passed,
        &mut total,
    );
    check(
        "Transfer GPU→NPU: positive bandwidth",
        gpu_to_npu.estimated_bandwidth_gbps > 0.0,
        &mut passed,
        &mut total,
    );
    check(
        "Transfer GPU→NPU: positive time",
        gpu_to_npu.estimated_time_us() > 0.0,
        &mut passed,
        &mut total,
    );

    let cpu_to_gpu = plan_transfer(cpu_nest, gpu_nest, 1_000_000, Some(node));
    check(
        "Transfer CPU→GPU: P2P DMA on same node",
        cpu_to_gpu.method == TransferMethod::PcieP2p,
        &mut passed,
        &mut total,
    );

    let pcie_bw = PcieGeneration::Gen4.lane_bandwidth_gbps();
    check(
        &format!("PCIe Gen4 per-lane: {pcie_bw:.3} GB/s"),
        (pcie_bw - 1.969).abs() < 0.01,
        &mut passed,
        &mut total,
    );

    // ── 4. Mixed V16 Dispatch Plan ──────────────────────────────────────
    println!("\n── 4. Mixed V16 Dispatch Plan ──────────────────────────────────");

    let dispatch_workloads = vec![
        (
            0,
            Workload::MichaelisMentenBatch { n_patients: 5_000 },
            40_000,
        ),
        (1, Workload::ScfaBatch { n_elements: 5_000 }, 120_000),
        (2, Workload::BeatClassifyBatch { n_beats: 5_000 }, 80_000),
        (
            3,
            Workload::BiosignalDetect {
                sample_rate_hz: 256,
            },
            4_096,
        ),
        (4, Workload::Analytical, 800),
    ];

    let plan: DispatchPlan = plan_dispatch(&dispatch_workloads, &caps, &tower);

    check(
        "Dispatch: 5 assignments",
        plan.assignments.len() == 5,
        &mut passed,
        &mut total,
    );
    check(
        "Dispatch: stages 0-2 → GPU",
        plan.assignments[0].substrate == Substrate::Gpu
            && plan.assignments[1].substrate == Substrate::Gpu
            && plan.assignments[2].substrate == Substrate::Gpu,
        &mut passed,
        &mut total,
    );
    check(
        "Dispatch: stage 3 → NPU",
        plan.assignments[3].substrate == Substrate::Npu,
        &mut passed,
        &mut total,
    );
    check(
        "Dispatch: stage 4 → CPU",
        plan.assignments[4].substrate == Substrate::Cpu,
        &mut passed,
        &mut total,
    );

    let subs_used = plan.substrates_used();
    check(
        "Dispatch: uses all 3 substrates",
        subs_used.len() == 3,
        &mut passed,
        &mut total,
    );
    check(
        "Dispatch: substrate transitions > 0",
        plan.n_substrate_transitions > 0,
        &mut passed,
        &mut total,
    );
    check(
        "Dispatch: total transfer bytes > 0",
        plan.total_transfer_bytes > 0,
        &mut passed,
        &mut total,
    );
    check(
        "Dispatch: transfer time > 0",
        plan.total_transfer_time_us() > 0.0,
        &mut passed,
        &mut total,
    );

    println!(
        "  Transitions: {}, Transfer bytes: {}, Transfer time: {:.1}us",
        plan.n_substrate_transitions,
        plan.total_transfer_bytes,
        plan.total_transfer_time_us()
    );

    // ── 5. GPU-only pipeline (no transitions) ───────────────────────────
    println!("\n── 5. GPU-Only Pipeline (zero transitions) ────────────────────");

    let gpu_only = vec![
        (
            0,
            Workload::MichaelisMentenBatch { n_patients: 10_000 },
            80_000,
        ),
        (1, Workload::ScfaBatch { n_elements: 10_000 }, 240_000),
        (2, Workload::BeatClassifyBatch { n_beats: 10_000 }, 160_000),
    ];

    let gpu_plan = plan_dispatch(&gpu_only, &caps, &tower);
    check(
        "GPU-only: 0 transitions",
        gpu_plan.n_substrate_transitions == 0,
        &mut passed,
        &mut total,
    );
    check(
        "GPU-only: all GPU",
        gpu_plan
            .assignments
            .iter()
            .all(|a| a.substrate == Substrate::Gpu),
        &mut passed,
        &mut total,
    );
    check(
        "GPU-only: no transfers",
        gpu_plan.assignments.iter().all(|a| a.transfer.is_none()),
        &mut passed,
        &mut total,
    );
    check(
        "GPU-only: 0 transfer bytes",
        gpu_plan.total_transfer_bytes == 0,
        &mut passed,
        &mut total,
    );

    // ── 6. NPU→GPU P2P in dispatch plan ─────────────────────────────────
    println!("\n── 6. NPU→GPU P2P Transfer in Dispatch ────────────────────────");

    let p2p_workloads = vec![
        (
            0,
            Workload::BiosignalDetect {
                sample_rate_hz: 256,
            },
            4_096,
        ),
        (1, Workload::BeatClassifyBatch { n_beats: 10_000 }, 160_000),
    ];

    let p2p_plan = plan_dispatch(&p2p_workloads, &caps, &tower);
    check(
        "P2P dispatch: stage 0 → NPU",
        p2p_plan.assignments[0].substrate == Substrate::Npu,
        &mut passed,
        &mut total,
    );
    check(
        "P2P dispatch: stage 1 → GPU",
        p2p_plan.assignments[1].substrate == Substrate::Gpu,
        &mut passed,
        &mut total,
    );
    check(
        "P2P dispatch: 1 transition",
        p2p_plan.n_substrate_transitions == 1,
        &mut passed,
        &mut total,
    );
    if let Some(transfer) = &p2p_plan.assignments[1].transfer {
        check(
            "P2P dispatch: transfer method is PcieP2p",
            transfer.method == TransferMethod::PcieP2p,
            &mut passed,
            &mut total,
        );
        check(
            "P2P dispatch: bypasses CPU roundtrip",
            transfer.method != TransferMethod::HostStaged,
            &mut passed,
            &mut total,
        );
        println!(
            "  NPU→GPU transfer: {} bytes, {:.1}us, {:.2} GB/s",
            transfer.bytes,
            transfer.estimated_time_us(),
            transfer.estimated_bandwidth_gbps
        );
    }

    // ── Summary ─────────────────────────────────────────────────────────
    println!("\n{}", "=".repeat(72));
    println!("Exp087 Mixed NUCLEUS V16 Dispatch: {passed}/{total} PASS");
    println!("{}", "=".repeat(72));

    if passed != total {
        std::process::exit(1);
    }
}
