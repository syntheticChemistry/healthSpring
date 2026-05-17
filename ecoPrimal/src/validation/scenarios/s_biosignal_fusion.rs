// SPDX-License-Identifier: AGPL-3.0-or-later

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::biosignal::fusion;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "biosignal-fusion",
            track: Track::Biosignal,
            tier: Tier::Rust,
            source_experiment: "exp023",
            description: "Multi-channel biosignal fusion: ECG+PPG+EDA → fused health assessment.",
        },
        run,
    }
}

fn run(v: &mut ValidationResult, _ctx: &mut CompositionContext) {
    v.section("Phase 1: Structural — Channel Fusion");

    let peaks: Vec<usize> = (0..20).map(|i| i * 250).collect();
    let fs = 250.0;
    let spo2 = 97.0;
    let scr_count = 5;
    let eda_duration_s = 60.0;

    let result = fusion::fuse_channels(&peaks, fs, spo2, scr_count, eda_duration_s);

    v.check_bool(
        "heart_rate_positive",
        result.heart_rate_bpm > 0.0,
        &format!("hr={}", result.heart_rate_bpm),
    );

    v.check_bool(
        "sdnn_nonneg",
        result.hrv_sdnn_ms >= 0.0,
        &format!("sdnn={}", result.hrv_sdnn_ms),
    );

    v.check_bool(
        "stress_index_bounded",
        result.stress_index >= 0.0 && result.stress_index <= 1.0,
        &format!("stress={}", result.stress_index),
    );

    v.check_bool(
        "overall_score_bounded",
        result.overall_score >= 0.0 && result.overall_score <= 100.0,
        &format!("score={}", result.overall_score),
    );

    v.check_bool(
        "spo2_passed_through",
        (result.spo2_percent - spo2).abs() < 1e-10,
        &format!("spo2={}", result.spo2_percent),
    );
}
