// SPDX-License-Identifier: AGPL-3.0-only
//! Microbiome clinical node builders (gut health, diversity).

use crate::endocrine;
use crate::visualization::scenarios::{bar, gauge, node};
use crate::visualization::types::{ClinicalRange, ScenarioNode};

pub fn gut_health_node(diversity: f64) -> ScenarioNode {
    let communities = [
        ("High Diversity", 0.95),
        ("Moderate", 0.70),
        ("Low Diversity", 0.30),
        ("Patient", diversity),
    ];

    let mut cats = Vec::new();
    let mut responses = Vec::new();
    let mut xi_max: f64 = 0.0;

    let mut xis = Vec::new();
    for &(_, j) in &communities {
        let w = endocrine::evenness_to_disorder(j, endocrine::gut_axis_params::DISORDER_SCALE);
        let xi =
            endocrine::anderson_localization_length(w, endocrine::gut_axis_params::LATTICE_SIZE);
        xis.push(xi);
        if xi > xi_max {
            xi_max = xi;
        }
    }

    for (i, &(name, _)) in communities.iter().enumerate() {
        let resp = endocrine::gut_metabolic_response(
            xis[i],
            xi_max,
            endocrine::gut_axis_params::BASE_RESPONSE_KG,
        );
        cats.push(name.to_string());
        responses.push(resp);
    }

    let patient_resp = responses.last().copied().unwrap_or(0.0);

    node(
        "gut_health",
        "Gut Health Factor (Cross-Track)",
        "compute",
        &[
            "clinical.predictor.gut_diversity",
            "clinical.predictor.metabolic_response",
        ],
        vec![
            bar(
                "gut_response",
                "Expected Weight Loss by Gut Diversity",
                cats,
                responses,
                "kg",
            ),
            gauge(
                "patient_gut",
                "Patient Gut Diversity (Pielou J)",
                diversity,
                0.0,
                1.0,
                "J",
                [0.6, 1.0],
                [0.3, 0.6],
            ),
            gauge(
                "patient_response",
                "Patient Predicted Weight Loss",
                patient_resp.abs(),
                0.0,
                20.0,
                "kg",
                [8.0, 20.0],
                [4.0, 8.0],
            ),
        ],
        vec![
            ClinicalRange {
                label: "Diverse gut".into(),
                min: 0.6,
                max: 1.0,
                status: "normal".into(),
            },
            ClinicalRange {
                label: "Low diversity".into(),
                min: 0.0,
                max: 0.4,
                status: "critical".into(),
            },
        ],
    )
}
