// SPDX-License-Identifier: AGPL-3.0-or-later
//! Endocrine cardiac monitoring node (HRV, SDNN, composite risk).

use crate::endocrine;
use crate::visualization::scenarios::{bar, gauge, node, timeseries};
use crate::visualization::types::{ClinicalRange, ClinicalStatus, NodeType, ScenarioNode};

pub fn cardiac_monitor_node(sdnn_base: f64) -> ScenarioNode {
    let delta = 20.0;
    let tau = 6.0;
    let months: Vec<f64> = (0..=240).map(|i| f64::from(i) / 10.0).collect();

    let sdnn_curve: Vec<f64> = months
        .iter()
        .map(|&m| endocrine::hrv_trt_response(sdnn_base, delta, tau, m))
        .collect();

    let risk_pre = endocrine::cardiac_risk_composite(sdnn_base, 280.0, 1.0);
    let risk_post = endocrine::cardiac_risk_composite(sdnn_base + delta, 500.0, 1.0);
    let reduction_pct = (1.0 - risk_post / risk_pre) * 100.0;

    node(
        "cardiac",
        "Cardiac Monitoring (HRV + Composite Risk)",
        NodeType::Compute,
        &["clinical.monitor.hrv", "clinical.monitor.cardiac_risk"],
        vec![
            timeseries(
                "sdnn",
                "SDNN on TRT",
                "Month",
                "SDNN (ms)",
                "ms",
                &months,
                sdnn_curve,
            ),
            bar(
                "risk_compare",
                "Cardiac Risk: Pre vs Post TRT",
                vec!["Pre-TRT".into(), "12-Month TRT".into()],
                vec![risk_pre, risk_post],
                "composite score",
            ),
            gauge(
                "risk_reduction",
                "Projected Risk Reduction",
                reduction_pct,
                0.0,
                100.0,
                "%",
                [15.0, 60.0],
                [5.0, 15.0],
            ),
        ],
        vec![
            ClinicalRange {
                label: "SDNN healthy".into(),
                min: 50.0,
                max: 200.0,
                status: ClinicalStatus::Normal,
            },
            ClinicalRange {
                label: "SDNN reduced".into(),
                min: 20.0,
                max: 50.0,
                status: ClinicalStatus::Warning,
            },
        ],
    )
}
