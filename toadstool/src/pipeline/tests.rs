// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pipeline unit tests.

use super::*;
use crate::stage::{ReduceKind, Stage, StageOp, TransformKind};
use healthspring_forge::Substrate;

fn make_generate_stage() -> Stage {
    Stage {
        name: "gen".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Generate {
            n_elements: 10,
            seed: 42,
        },
    }
}

fn make_transform_stage() -> Stage {
    Stage {
        name: "hill".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::ElementwiseTransform {
            kind: TransformKind::Hill {
                emax: 1.0,
                ec50: 0.5,
                n: 2.0,
            },
        },
    }
}

fn make_reduce_stage() -> Stage {
    Stage {
        name: "mean".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Reduce {
            kind: ReduceKind::Mean,
        },
    }
}

fn make_diversity_stage() -> Stage {
    Stage {
        name: "diversity".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::DiversityReduce {
            communities: vec![vec![0.25, 0.25, 0.25, 0.25], vec![0.9, 0.05, 0.03, 0.02]],
        },
    }
}

fn make_population_pk_stage() -> Stage {
    Stage {
        name: "pop_pk".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::PopulationPk {
            n_patients: 20,
            dose_mg: 4.0,
            f_bioavail: 0.79,
            seed: 123,
        },
    }
}

fn make_filter_stage(threshold: f64) -> Stage {
    Stage {
        name: "filter".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Filter { threshold },
    }
}

#[test]
fn pipeline_new_is_empty() {
    let p = Pipeline::new("test");
    assert!(p.is_empty());
    assert_eq!(p.len(), 0);
    assert_eq!(p.name, "test");
}

#[test]
fn pipeline_add_stage_increases_len() {
    let mut p = Pipeline::new("test");
    p.add_stage(make_generate_stage());
    assert_eq!(p.len(), 1);
    p.add_stage(make_transform_stage());
    assert_eq!(p.len(), 2);
}

#[test]
fn pipeline_stage_names() {
    let mut p = Pipeline::new("test");
    p.add_stage(make_generate_stage());
    p.add_stage(make_transform_stage());
    p.add_stage(make_reduce_stage());
    assert_eq!(p.stage_names(), ["gen", "hill", "mean"]);
}

#[test]
fn pipeline_execute_cpu_generate_transform_reduce() {
    let mut p = Pipeline::new("dose_response");
    p.add_stage(make_generate_stage());
    p.add_stage(make_transform_stage());
    p.add_stage(make_reduce_stage());

    let result = p.execute_cpu();
    assert!(result.success);
    assert_eq!(result.stage_results.len(), 3);
    assert!(result.total_time_us >= 0.0);

    // First stage: 10 elements generated
    assert_eq!(result.stage_results[0].output_data.len(), 10);

    // Second stage: 10 elements transformed
    assert_eq!(result.stage_results[1].output_data.len(), 10);

    // Third stage: single scalar (mean)
    assert_eq!(result.stage_results[2].output_data.len(), 1);
}

#[test]
fn pipeline_execute_empty_returns_empty_result() {
    let p = Pipeline::new("empty");
    let result = p.execute_cpu();
    assert!(result.success);
    assert!(result.stage_results.is_empty());
    assert!(result.total_time_us.abs() < f64::EPSILON);
}

#[test]
fn pipeline_execute_cpu_with_diversity_reduce() {
    let mut p = Pipeline::new("diversity");
    p.add_stage(make_diversity_stage());

    let result = p.execute_cpu();
    assert!(result.success);
    assert_eq!(result.stage_results.len(), 1);
    // DiversityReduce outputs [shannon1, simpson1, shannon2, simpson2] per community
    assert_eq!(result.stage_results[0].output_data.len(), 4);
    assert!(result.total_time_us > 0.0);
}

#[test]
fn pipeline_execute_cpu_with_population_pk() {
    let mut p = Pipeline::new("pop_pk");
    p.add_stage(make_population_pk_stage());

    let result = p.execute_cpu();
    assert!(result.success);
    assert_eq!(result.stage_results.len(), 1);
    assert_eq!(result.stage_results[0].output_data.len(), 20);
    assert!(result.stage_results[0].output_data.iter().all(|&v| v > 0.0));
}

#[test]
fn pipeline_execute_cpu_with_filter() {
    let mut p = Pipeline::new("filtered");
    p.add_stage(make_generate_stage());
    p.add_stage(make_filter_stage(0.5));

    let result = p.execute_cpu();
    assert!(result.success);
    assert_eq!(result.stage_results.len(), 2);
    let filtered = &result.stage_results[1].output_data;
    assert!(filtered.iter().all(|&x| x > 0.5));
    assert!(filtered.len() <= 10);
}

#[test]
fn pipeline_success_tracking_generate_transform_filter_reduce() {
    let mut p = Pipeline::new("full");
    p.add_stage(make_generate_stage());
    p.add_stage(make_transform_stage());
    p.add_stage(make_filter_stage(0.1));
    p.add_stage(Stage {
        name: "sum".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Reduce {
            kind: ReduceKind::Sum,
        },
    });

    let result = p.execute_cpu();
    assert!(result.success);
    assert_eq!(result.stage_results.len(), 4);
    assert_eq!(result.stage_results[3].output_data.len(), 1);
}

#[test]
fn pipeline_data_flow_correctness() {
    let mut p = Pipeline::new("flow");
    p.add_stage(Stage {
        name: "gen".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Generate {
            n_elements: 5,
            seed: 999,
        },
    });
    p.add_stage(Stage {
        name: "square".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::ElementwiseTransform {
            kind: TransformKind::Square,
        },
    });
    p.add_stage(Stage {
        name: "sum".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Reduce {
            kind: ReduceKind::Sum,
        },
    });

    let result = p.execute_cpu();
    assert!(result.success);
    let gen_out = &result.stage_results[0].output_data;
    let square_out = &result.stage_results[1].output_data;
    let sum_out = &result.stage_results[2].output_data;

    assert_eq!(gen_out.len(), 5);
    assert_eq!(square_out.len(), 5);
    for i in 0..5 {
        let expected = gen_out[i] * gen_out[i];
        assert!(
            (square_out[i] - expected).abs() < 1e-10,
            "square[{i}] = {} expected {}",
            square_out[i],
            expected
        );
    }
    let expected_sum: f64 = square_out.iter().sum();
    assert!(
        (sum_out[0] - expected_sum).abs() < 1e-10,
        "sum = {} expected {}",
        sum_out[0],
        expected_sum
    );
}

#[test]
fn pipeline_result_total_time_positive() {
    let mut p = Pipeline::new("timed");
    p.add_stage(make_generate_stage());
    p.add_stage(make_reduce_stage());

    let result = p.execute_cpu();
    assert!(result.success);
    assert!(result.total_time_us >= 0.0);
}

#[test]
fn pipeline_reduce_sum() {
    let mut p = Pipeline::new("sum");
    p.add_stage(Stage {
        name: "gen".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Generate {
            n_elements: 4,
            seed: 1,
        },
    });
    p.add_stage(Stage {
        name: "sum".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Reduce {
            kind: ReduceKind::Sum,
        },
    });
    let result = p.execute_cpu();
    assert!(result.success);
    assert_eq!(result.stage_results[1].output_data.len(), 1);
}

#[test]
fn pipeline_reduce_max() {
    let mut p = Pipeline::new("max");
    p.add_stage(make_generate_stage());
    p.add_stage(Stage {
        name: "max".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Reduce {
            kind: ReduceKind::Max,
        },
    });
    let result = p.execute_cpu();
    assert!(result.success);
    assert_eq!(result.stage_results[1].output_data.len(), 1);
    assert!(result.stage_results[1].output_data[0] <= 1.0);
}

#[test]
fn pipeline_reduce_min() {
    let mut p = Pipeline::new("min");
    p.add_stage(make_generate_stage());
    p.add_stage(Stage {
        name: "min".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Reduce {
            kind: ReduceKind::Min,
        },
    });
    let result = p.execute_cpu();
    assert!(result.success);
    assert_eq!(result.stage_results[1].output_data.len(), 1);
    assert!(result.stage_results[1].output_data[0] >= 0.0);
}

#[test]
fn pipeline_transform_square() {
    let mut p = Pipeline::new("square");
    p.add_stage(Stage {
        name: "gen".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Generate {
            n_elements: 3,
            seed: 0,
        },
    });
    p.add_stage(Stage {
        name: "sq".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::ElementwiseTransform {
            kind: TransformKind::Square,
        },
    });
    let result = p.execute_cpu();
    assert!(result.success);
    let out = &result.stage_results[1].output_data;
    assert_eq!(out.len(), 3);
    for (i, &out_val) in out.iter().enumerate() {
        let inp = result.stage_results[0].output_data[i];
        assert!(inp.mul_add(-inp, out_val).abs() < 1e-10);
    }
}

#[test]
fn pipeline_transform_exp_decay() {
    let mut p = Pipeline::new("exp_decay");
    p.add_stage(Stage {
        name: "gen".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::Generate {
            n_elements: 5,
            seed: 42,
        },
    });
    p.add_stage(Stage {
        name: "decay".into(),
        substrate: Substrate::Cpu,
        operation: StageOp::ElementwiseTransform {
            kind: TransformKind::ExpDecay { k: 0.1, t: 1.0 },
        },
    });
    let result = p.execute_cpu();
    assert!(result.success);
    assert_eq!(result.stage_results[1].output_data.len(), 5);
}

#[test]
fn pipeline_execute_streaming_invokes_callback() {
    let mut p = Pipeline::new("streaming_test");
    p.add_stage(make_generate_stage());
    p.add_stage(make_transform_stage());
    p.add_stage(make_reduce_stage());

    let mut callback_calls = Vec::new();
    let result = p.execute_streaming(|idx, total, stage_result| {
        callback_calls.push((idx, total, stage_result.stage_name.clone()));
    });

    assert!(result.success);
    assert_eq!(result.stage_results.len(), 3);
    assert_eq!(callback_calls.len(), 3);
    assert_eq!(callback_calls[0], (0, 3, "gen".to_string()));
    assert_eq!(callback_calls[1], (1, 3, "hill".to_string()));
    assert_eq!(callback_calls[2], (2, 3, "mean".to_string()));
}

#[test]
fn pipeline_execute_streaming_matches_cpu() {
    let mut p = Pipeline::new("streaming_cpu_match");
    p.add_stage(make_generate_stage());
    p.add_stage(make_transform_stage());
    p.add_stage(make_reduce_stage());

    let cpu_result = p.execute_cpu();
    let streaming_result = p.execute_streaming(|_, _, _| {});

    assert_eq!(
        cpu_result.stage_results.len(),
        streaming_result.stage_results.len()
    );
    for (cpu, stream) in cpu_result
        .stage_results
        .iter()
        .zip(streaming_result.stage_results.iter())
    {
        assert_eq!(cpu.output_data, stream.output_data);
        assert_eq!(cpu.stage_name, stream.stage_name);
    }
}
