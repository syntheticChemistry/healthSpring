// SPDX-License-Identifier: AGPL-3.0-or-later
//! Stage unit tests.

use super::*;
use healthspring_barracuda::tolerances::MACHINE_EPSILON;
use healthspring_forge::Substrate;

#[test]
fn generate_produces_deterministic_data() {
    let stage = Stage {
        name: "gen".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Generate {
            n_elements: 5,
            seed: 123,
        },
    };
    let r1 = stage.execute(None);
    let r2 = stage.execute(None);
    assert_eq!(r1.output_data, r2.output_data);
    assert_eq!(r1.output_data.len(), 5);
}

#[test]
fn generate_values_in_unit_interval() {
    let stage = Stage {
        name: "gen".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Generate {
            n_elements: 100,
            seed: 0,
        },
    };
    let r = stage.execute(None);
    for &v in &r.output_data {
        assert!((0.0..=1.0).contains(&v), "value {v} outside [0,1]");
    }
}

#[test]
fn transform_square() {
    let stage = Stage {
        name: "sq".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::ElementwiseTransform {
            kind: TransformKind::Square,
        },
    };
    let input = [1.0, 2.0, 3.0];
    let r = stage.execute(Some(&input));
    assert_eq!(r.output_data, [1.0, 4.0, 9.0]);
}

#[test]
fn transform_hill_at_zero_is_zero() {
    let stage = Stage {
        name: "hill".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::ElementwiseTransform {
            kind: TransformKind::Hill {
                emax: 1.0,
                ec50: 0.5,
                n: 2.0,
            },
        },
    };
    let input = [0.0];
    let r = stage.execute(Some(&input));
    assert!(r.output_data[0].abs() < f64::EPSILON);
}

#[test]
fn reduce_sum() {
    let stage = Stage {
        name: "sum".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Reduce {
            kind: ReduceKind::Sum,
        },
    };
    let input = [1.0, 2.0, 3.0];
    let r = stage.execute(Some(&input));
    assert_eq!(r.output_data.len(), 1);
    assert!((r.output_data[0] - 6.0).abs() < MACHINE_EPSILON);
}

#[test]
fn reduce_mean() {
    let stage = Stage {
        name: "mean".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Reduce {
            kind: ReduceKind::Mean,
        },
    };
    let input = [2.0, 4.0, 6.0];
    let r = stage.execute(Some(&input));
    assert_eq!(r.output_data.len(), 1);
    assert!((r.output_data[0] - 4.0).abs() < MACHINE_EPSILON);
}

#[test]
fn reduce_empty_returns_zero() {
    let stage = Stage {
        name: "sum".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Reduce {
            kind: ReduceKind::Sum,
        },
    };
    let r = stage.execute(Some(&[]));
    assert_eq!(r.output_data, [0.0]);
}

#[test]
fn filter_keeps_above_threshold() {
    let stage = Stage {
        name: "filter".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Filter { threshold: 0.5 },
    };
    let input = [0.3, 0.6, 0.4, 0.8];
    let r = stage.execute(Some(&input));
    assert_eq!(r.output_data, [0.6, 0.8]);
}

#[test]
fn population_pk_produces_positive_aucs() {
    let stage = Stage {
        name: "pop_pk".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::PopulationPk {
            n_patients: 50,
            dose_mg: 4.0,
            f_bioavail: 0.79,
            seed: 42,
        },
    };
    let r = stage.execute(None);
    assert_eq!(r.output_data.len(), 50);
    assert!(r.output_data.iter().all(|&a| a > 0.0));
}

#[test]
fn population_pk_maps_to_gpu_op() {
    let stage = Stage {
        name: "pop_pk".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::PopulationPk {
            n_patients: 100,
            dose_mg: 4.0,
            f_bioavail: 0.79,
            seed: 7,
        },
    };
    let op = stage.to_gpu_op(None);
    let Some(op) = op else {
        panic!("to_gpu_op returns Some for PopulationPk");
    };
    assert!(matches!(op, GpuOp::PopulationPkBatch { .. }));
}

#[test]
fn diversity_reduce_computes_shannon_simpson() {
    let stage = Stage {
        name: "diversity".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::DiversityReduce {
            communities: vec![vec![0.25, 0.25, 0.25, 0.25], vec![0.9, 0.05, 0.03, 0.02]],
        },
    };
    let r = stage.execute(None);
    assert_eq!(r.output_data.len(), 4);
    let (even_shannon, even_simpson) = (r.output_data[0], r.output_data[1]);
    let (dom_shannon, _dom_simpson) = (r.output_data[2], r.output_data[3]);
    assert!(even_shannon > dom_shannon, "even > dominated");
    assert!(even_simpson > 0.0);
}

#[test]
fn diversity_reduce_maps_to_gpu_op() {
    let stage = Stage {
        name: "diversity".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::DiversityReduce {
            communities: vec![vec![0.5, 0.5]],
        },
    };
    let op = stage.to_gpu_op(None);
    let Some(op) = op else {
        panic!("to_gpu_op returns Some for DiversityReduce");
    };
    assert!(matches!(op, GpuOp::DiversityBatch { .. }));
}

#[test]
fn michaelis_menten_batch_executes_on_cpu() {
    let stage = Stage {
        name: "mm".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::MichaelisMentenBatch {
            vmax: 500.0,
            km: 5.0,
            vd: 50.0,
            dt: 0.01,
            n_steps: 200,
            n_patients: 8,
            seed: 42,
        },
    };
    let r = stage.execute(None);
    assert!(r.success);
    assert_eq!(r.output_data.len(), 8);
    assert!(r.output_data.iter().all(|&a| a > 0.0), "all AUC positive");
}

#[test]
fn michaelis_menten_batch_maps_to_gpu_op() {
    let stage = Stage {
        name: "mm".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::MichaelisMentenBatch {
            vmax: 500.0,
            km: 5.0,
            vd: 50.0,
            dt: 0.01,
            n_steps: 200,
            n_patients: 8,
            seed: 42,
        },
    };
    let op = stage.to_gpu_op(None);
    assert!(op.is_some());
    assert!(matches!(op, Some(GpuOp::MichaelisMentenBatch { .. })));
}

#[test]
fn scfa_batch_executes_on_cpu() {
    let stage = Stage {
        name: "scfa".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::ScfaBatch {
            params: healthspring_barracuda::microbiome::SCFA_HEALTHY_PARAMS,
            fiber_inputs: vec![5.0, 10.0, 20.0],
        },
    };
    let r = stage.execute(None);
    assert!(r.success);
    assert_eq!(r.output_data.len(), 9, "3 fibers × 3 SCFAs");
    assert!(r.output_data.iter().all(|&v| v > 0.0), "all SCFA positive");
}

#[test]
fn scfa_batch_maps_to_gpu_op() {
    let stage = Stage {
        name: "scfa".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::ScfaBatch {
            params: healthspring_barracuda::microbiome::SCFA_HEALTHY_PARAMS,
            fiber_inputs: vec![5.0],
        },
    };
    assert!(stage.to_gpu_op(None).is_some());
}

#[test]
fn beat_classify_batch_executes_on_cpu() {
    use healthspring_barracuda::biosignal::classification;
    let templates = vec![
        classification::generate_normal_template(41),
        classification::generate_pvc_template(41),
        classification::generate_pac_template(41),
    ];
    let beats = vec![
        classification::generate_normal_template(41),
        classification::generate_pvc_template(41),
    ];
    let stage = Stage {
        name: "classify".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::BeatClassifyBatch { beats, templates },
    };
    let r = stage.execute(None);
    assert!(r.success);
    assert_eq!(r.output_data.len(), 4, "2 beats × 2 values (idx, corr)");
}

#[test]
fn beat_classify_batch_maps_to_gpu_op() {
    let stage = Stage {
        name: "classify".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::BeatClassifyBatch {
            beats: vec![vec![0.0; 41]],
            templates: vec![vec![0.0; 41]],
        },
    };
    assert!(stage.to_gpu_op(None).is_some());
}

#[test]
fn biosignal_fusion_produces_fused_signal() {
    let stage = Stage {
        name: "fusion".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::BiosignalFusion { n_channels: 2 },
    };
    let input = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let r = stage.execute(Some(&input));
    assert!(r.success);
    assert!(!r.output_data.is_empty());
}

#[test]
fn biosignal_fusion_empty_input() {
    let stage = Stage {
        name: "fusion".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::BiosignalFusion { n_channels: 2 },
    };
    let r = stage.execute(Some(&[]));
    assert!(r.success);
    assert!(r.output_data.is_empty());
}

#[test]
fn biosignal_fusion_zero_channels() {
    let stage = Stage {
        name: "fusion".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::BiosignalFusion { n_channels: 0 },
    };
    let r = stage.execute(Some(&[1.0, 2.0]));
    assert!(r.success);
    assert!(r.output_data.is_empty());
}

#[test]
fn auc_trapezoidal_computes_area() {
    let stage = Stage {
        name: "auc".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::AucTrapezoidal { t_max: 10.0 },
    };
    let concs = [1.0, 1.0, 1.0, 1.0, 1.0];
    let r = stage.execute(Some(&concs));
    assert!(r.success);
    assert_eq!(r.output_data.len(), 1);
    assert!(
        (r.output_data[0] - 10.0).abs() < 0.01,
        "constant 1 over [0,10] = 10"
    );
}

#[test]
fn auc_trapezoidal_single_point_is_zero() {
    let stage = Stage {
        name: "auc".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::AucTrapezoidal { t_max: 10.0 },
    };
    let r = stage.execute(Some(&[5.0]));
    assert!(r.success);
    assert_eq!(r.output_data, [0.0]);
}

#[test]
fn bray_curtis_pairwise_dissimilarity() {
    let stage = Stage {
        name: "bc".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::BrayCurtis {
            communities: vec![
                vec![0.5, 0.3, 0.2],
                vec![0.5, 0.3, 0.2],
                vec![0.9, 0.05, 0.05],
            ],
        },
    };
    let r = stage.execute(None);
    assert!(r.success);
    assert_eq!(r.output_data.len(), 3, "3 pairs from 3 communities");
    assert!(
        r.output_data[0].abs() < MACHINE_EPSILON,
        "identical communities → BC = 0"
    );
    assert!(r.output_data[1] > 0.0, "different communities → BC > 0");
}

#[test]
fn reduce_variance_known_values() {
    let stage = Stage {
        name: "var".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Reduce {
            kind: ReduceKind::Variance,
        },
    };
    let input = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let r = stage.execute(Some(&input));
    assert!(r.success);
    assert_eq!(r.output_data.len(), 1);
    assert!(
        (r.output_data[0] - 4.0).abs() < 0.01,
        "variance of [2,4,4,4,5,5,7,9] = 4.0"
    );
}

#[test]
fn exp_decay_transform_reduces_values() {
    let stage = Stage {
        name: "decay".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::ElementwiseTransform {
            kind: TransformKind::ExpDecay { k: 1.0, t: 1.0 },
        },
    };
    let input = [10.0, 20.0];
    let r = stage.execute(Some(&input));
    assert!(r.success);
    let factor = (-1.0_f64).exp();
    assert!((r.output_data[0] - 10.0 * factor).abs() < MACHINE_EPSILON);
    assert!((r.output_data[1] - 20.0 * factor).abs() < MACHINE_EPSILON);
}

#[test]
fn filter_non_gpu_mappable() {
    let stage = Stage {
        name: "filter".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Filter { threshold: 0.5 },
    };
    assert!(
        stage.to_gpu_op(Some(&[1.0])).is_none(),
        "filter has no GPU kernel"
    );
}

#[test]
fn auc_trapezoidal_non_gpu_mappable() {
    let stage = Stage {
        name: "auc".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::AucTrapezoidal { t_max: 10.0 },
    };
    assert!(stage.to_gpu_op(Some(&[1.0])).is_none());
}

#[test]
fn bray_curtis_non_gpu_mappable() {
    let stage = Stage {
        name: "bc".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::BrayCurtis {
            communities: vec![vec![0.5, 0.5]],
        },
    };
    assert!(stage.to_gpu_op(None).is_none());
}

#[test]
fn biosignal_fusion_non_gpu_mappable() {
    let stage = Stage {
        name: "fusion".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::BiosignalFusion { n_channels: 2 },
    };
    assert!(stage.to_gpu_op(Some(&[1.0, 2.0])).is_none());
}

#[test]
fn hill_transform_maps_to_gpu_op_with_input() {
    let stage = Stage {
        name: "hill".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::ElementwiseTransform {
            kind: TransformKind::Hill {
                emax: 1.0,
                ec50: 0.5,
                n: 2.0,
            },
        },
    };
    let input = [0.1, 0.5, 1.0];
    let op = stage.to_gpu_op(Some(&input));
    assert!(op.is_some());
    if let Some(GpuOp::HillSweep { concentrations, .. }) = op {
        assert_eq!(concentrations, &[0.1, 0.5, 1.0]);
    }
}

#[test]
fn stage_result_reports_cpu_substrate() {
    let stage = Stage {
        name: "gen".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Generate {
            n_elements: 3,
            seed: 1,
        },
    };
    let r = stage.execute(None);
    assert_eq!(r.substrate, Substrate::Cpu);
    assert!(r.elapsed_us >= 0.0);
}

#[test]
fn square_transform_non_gpu_mappable() {
    let stage = Stage {
        name: "sq".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::ElementwiseTransform {
            kind: TransformKind::Square,
        },
    };
    assert!(stage.to_gpu_op(Some(&[1.0])).is_none());
}
