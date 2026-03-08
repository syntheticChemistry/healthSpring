// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pipeline stage: a single compute operation within a pipeline.

use healthspring_forge::Substrate;

/// A single compute stage within a pipeline.
#[derive(Debug)]
pub struct Stage {
    pub name: String,
    pub substrate: Substrate,
    pub operation: StageOp,
}

/// The operation a stage performs.
#[derive(Debug, Clone)]
pub enum StageOp {
    /// Generate input data (source stage).
    Generate { n_elements: usize, seed: u64 },
    /// Element-wise transform: apply f(x) to each element.
    ElementwiseTransform { kind: TransformKind },
    /// Reduce: aggregate elements to a scalar or smaller array.
    Reduce { kind: ReduceKind },
    /// Filter: keep elements matching a predicate.
    Filter { threshold: f64 },
}

/// Kind of element-wise transform.
#[derive(Debug, Clone, Copy)]
pub enum TransformKind {
    /// Hill dose-response: E = Emax * c^n / (EC50^n + c^n)
    Hill { emax: f64, ec50: f64, n: f64 },
    /// Squaring: y = x^2
    Square,
    /// Exponential decay: y = x * exp(-k * t)
    ExpDecay { k: f64, t: f64 },
}

/// Kind of reduction.
#[derive(Debug, Clone, Copy)]
pub enum ReduceKind {
    Sum,
    Mean,
    Max,
    Min,
    Variance,
}

/// Result of executing a single stage.
#[derive(Debug, Clone)]
pub struct StageResult {
    pub stage_name: String,
    pub substrate: Substrate,
    pub output_data: Vec<f64>,
    pub elapsed_us: f64,
    pub success: bool,
}

impl Stage {
    /// Execute this stage on CPU.
    #[must_use]
    #[expect(clippy::cast_precision_loss)]
    pub fn execute(&self, input: Option<&[f64]>) -> StageResult {
        let start = std::time::Instant::now();
        let output = match &self.operation {
            StageOp::Generate { n_elements, seed } => generate_data(*n_elements, *seed),
            StageOp::ElementwiseTransform { kind } => {
                let data = input.unwrap_or(&[]);
                apply_transform(data, *kind)
            }
            StageOp::Reduce { kind } => {
                let data = input.unwrap_or(&[]);
                vec![apply_reduce(data, *kind)]
            }
            StageOp::Filter { threshold } => {
                let data = input.unwrap_or(&[]);
                data.iter().copied().filter(|&x| x > *threshold).collect()
            }
        };
        let elapsed = start.elapsed().as_micros() as f64;

        StageResult {
            stage_name: self.name.clone(),
            substrate: self.substrate,
            output_data: output,
            elapsed_us: elapsed,
            success: true,
        }
    }
}

#[expect(clippy::cast_precision_loss)]
fn generate_data(n: usize, seed: u64) -> Vec<f64> {
    let mut data = Vec::with_capacity(n);
    let mut state = seed;
    for _ in 0..n {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        let val = (state >> 33) as f64 / f64::from(u32::MAX);
        data.push(val);
    }
    data
}

fn apply_transform(data: &[f64], kind: TransformKind) -> Vec<f64> {
    match kind {
        TransformKind::Hill { emax, ec50, n } => data
            .iter()
            .map(|&c| {
                if c <= 0.0 {
                    return 0.0;
                }
                let cn = c.powf(n);
                emax * cn / (ec50.powf(n) + cn)
            })
            .collect(),
        TransformKind::Square => data.iter().map(|&x| x * x).collect(),
        TransformKind::ExpDecay { k, t } => data.iter().map(|&x| x * (-k * t).exp()).collect(),
    }
}

#[expect(clippy::cast_precision_loss)]
fn apply_reduce(data: &[f64], kind: ReduceKind) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    match kind {
        ReduceKind::Sum => data.iter().sum(),
        ReduceKind::Mean => data.iter().sum::<f64>() / data.len() as f64,
        ReduceKind::Max => data.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        ReduceKind::Min => data.iter().copied().fold(f64::INFINITY, f64::min),
        ReduceKind::Variance => {
            let mean = data.iter().sum::<f64>() / data.len() as f64;
            data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64
        }
    }
}

#[cfg(test)]
mod tests {
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
            assert!(v >= 0.0 && v <= 1.0, "value {v} outside [0,1]");
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
        assert_eq!(r.output_data[0], 0.0);
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
}
