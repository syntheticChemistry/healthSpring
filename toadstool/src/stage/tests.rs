// SPDX-License-Identifier: AGPL-3.0-or-later
//! Stage unit tests.

use super::*;
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
    assert!((r.output_data[0] - 6.0).abs() < 1e-10);
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
    assert!((r.output_data[0] - 4.0).abs() < 1e-10);
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
