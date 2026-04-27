// SPDX-License-Identifier: AGPL-3.0-or-later

use super::super::types::{
    ClinicalRange, ClinicalStatus, HealthScenario, NodeType, ScenarioEdge,
};
use super::{bar, edge, gauge, heatmap, node, scaffold, spectrum, timeseries};
use crate::microbiome;

/// Build a complete microbiome study scenario with real computed data.
#[must_use]
#[expect(clippy::too_many_lines, reason = "4 sub-studies, each compact")]
#[expect(
    clippy::cast_precision_loss,
    reason = "lattice size ≤ 50, well within f64 mantissa"
)]
pub fn microbiome_study() -> (HealthScenario, Vec<ScenarioEdge>) {
    let mut scenario = scaffold(
        "healthSpring Microbiome Study",
        "Diversity indices, Anderson lattice, C. diff resistance, FMT — 4 experiments",
    );

    let communities: [(&str, Vec<f64>); 4] = [
        ("Healthy", microbiome::communities::HEALTHY_GUT.to_vec()),
        ("Dysbiotic", microbiome::communities::DYSBIOTIC_GUT.to_vec()),
        ("C. diff", microbiome::communities::CDIFF_COLONIZED.to_vec()),
        ("Even", microbiome::communities::PERFECTLY_EVEN.to_vec()),
    ];

    // Diversity indices (exp010)
    let mut shannon_vals = Vec::new();
    let mut simpson_vals = Vec::new();
    let mut pielou_vals = Vec::new();
    let mut cats = Vec::new();
    for (name, ab) in &communities {
        cats.push((*name).to_string());
        shannon_vals.push(microbiome::shannon_index(ab));
        simpson_vals.push(microbiome::simpson_index(ab));
        pielou_vals.push(microbiome::pielou_evenness(ab));
    }
    // Bray-Curtis dissimilarity matrix between all community pairs
    let n_comm = communities.len();
    let mut bc_matrix = Vec::with_capacity(n_comm * n_comm);
    for (_, ab_row) in &communities {
        for (_, ab_col) in &communities {
            bc_matrix.push(microbiome::bray_curtis(ab_row, ab_col));
        }
    }

    scenario.ecosystem.primals.push(node(
        "diversity",
        "Diversity Indices",
        NodeType::Compute,
        &["science.microbiome.diversity"],
        vec![
            bar("shannon", "Shannon H′", &cats, shannon_vals, "nats"),
            bar(
                "simpson",
                "Simpson D",
                &cats,
                simpson_vals,
                "probability",
            ),
            bar(
                "pielou",
                "Pielou J",
                &cats,
                pielou_vals.clone(),
                "evenness",
            ),
            heatmap(
                "bray_curtis",
                "Bray-Curtis Dissimilarity",
                cats.clone(),
                cats.clone(),
                bc_matrix,
                "BC",
            ),
        ],
        vec![
            ClinicalRange {
                label: "Healthy Shannon".into(),
                min: 2.5,
                max: 4.0,
                status: ClinicalStatus::Normal,
            },
            ClinicalRange {
                label: "Dysbiotic Shannon".into(),
                min: 0.0,
                max: 1.5,
                status: ClinicalStatus::Critical,
            },
        ],
    ));

    // Anderson lattice (exp011)
    let healthy_j = microbiome::pielou_evenness(&microbiome::communities::HEALTHY_GUT);
    let dysbiotic_j = microbiome::pielou_evenness(&microbiome::communities::DYSBIOTIC_GUT);
    let w_healthy = microbiome::evenness_to_disorder(healthy_j, 5.0);
    let w_dysbiotic = microbiome::evenness_to_disorder(dysbiotic_j, 5.0);
    let lattice_size: usize = 50;
    let half = lattice_size / 2;
    let disorder_h: Vec<f64> = (0..lattice_size)
        .map(|i| {
            let offset = if i >= half {
                (i - half) as f64
            } else {
                -((half - i) as f64)
            };
            w_healthy * 0.1f64.mul_add(offset, 1.0)
        })
        .collect();
    let (eigvals, eigvecs) = microbiome::anderson_diagonalize(&disorder_h, 1.0);
    let mid_psi =
        &eigvecs[(lattice_size / 2) * lattice_size..(lattice_size / 2 + 1) * lattice_size];
    let ipr = microbiome::inverse_participation_ratio(mid_psi);
    let xi = microbiome::localization_length_from_ipr(ipr);
    let cr = microbiome::colonization_resistance(xi);
    #[expect(clippy::cast_precision_loss, reason = "lattice index fits f64")]
    let site_indices: Vec<f64> = (0..lattice_size).map(|i| i as f64).collect();

    let ipr_spectrum: Vec<f64> = (0..lattice_size)
        .map(|k| {
            let psi = &eigvecs[k * lattice_size..(k + 1) * lattice_size];
            microbiome::inverse_participation_ratio(psi)
        })
        .collect();

    scenario.ecosystem.primals.push(node(
        "anderson",
        "Anderson Gut Lattice",
        NodeType::Compute,
        &["science.microbiome.anderson_lattice"],
        vec![
            spectrum(
                "anderson_eigvals",
                "Anderson Eigenvalue Spectrum",
                site_indices.clone(),
                eigvals,
                "energy (a.u.)",
            ),
            spectrum(
                "anderson_ipr_spectrum",
                "Anderson IPR Spectrum",
                site_indices,
                ipr_spectrum,
                "IPR",
            ),
            gauge(
                "ipr",
                "Inverse Participation Ratio",
                ipr,
                0.0,
                1.0,
                "dimensionless",
                [0.0, 0.1],
                [0.1, 0.5],
            ),
            gauge(
                "xi",
                "Localization Length ξ",
                xi,
                0.0,
                100.0,
                "sites",
                [5.0, 50.0],
                [1.0, 5.0],
            ),
            gauge(
                "cr",
                "Colonization Resistance",
                cr,
                0.0,
                1.0,
                "1/ξ",
                [0.02, 0.5],
                [0.5, 0.9],
            ),
            bar(
                "disorder",
                "Anderson Disorder W",
                vec!["Healthy".into(), "Dysbiotic".into()],
                vec![w_healthy, w_dysbiotic],
                "a.u.",
            ),
        ],
        vec![],
    ));

    // C. diff resistance (exp012)
    let mut cr_cats = Vec::new();
    let mut cr_vals = Vec::new();
    for (name, ab) in &communities {
        let j = microbiome::pielou_evenness(ab);
        let w = microbiome::evenness_to_disorder(j, 5.0);
        let disorder_v: Vec<f64> = (0..lattice_size)
            .map(|i| {
                let offset = if i >= half {
                    (i - half) as f64
                } else {
                    -((half - i) as f64)
                };
                w * 0.1f64.mul_add(offset, 1.0)
            })
            .collect();
        let (_ev, vecs) = microbiome::anderson_diagonalize(&disorder_v, 1.0);
        let mid = lattice_size / 2;
        let mid_psi = &vecs[mid * lattice_size..(mid + 1) * lattice_size];
        let local_ipr = microbiome::inverse_participation_ratio(mid_psi);
        let local_xi = microbiome::localization_length_from_ipr(local_ipr);
        cr_cats.push((*name).to_string());
        cr_vals.push(microbiome::colonization_resistance(local_xi));
    }
    scenario.ecosystem.primals.push(node(
        "cdiff",
        "C. diff Colonization Resistance",
        NodeType::Compute,
        &["science.microbiome.cdiff_resistance"],
        vec![bar(
            "cr_compare",
            "Colonization Resistance by Community",
            &cr_cats,
            cr_vals,
            "1/ξ",
        )],
        vec![ClinicalRange {
            label: "Protective CR".into(),
            min: 0.05,
            max: 1.0,
            status: ClinicalStatus::Normal,
        }],
    ));

    // FMT engraftment (exp013)
    let donor = &microbiome::communities::HEALTHY_GUT[..];
    let recipient = &microbiome::communities::CDIFF_COLONIZED[..];
    let engraftments = [0.2, 0.4, 0.6, 0.8, 1.0];
    let mut eng_x = Vec::new();
    let mut shannon_y = Vec::new();
    let mut bc_y = Vec::new();
    for &e in &engraftments {
        let post = microbiome::fmt_blend(donor, recipient, e);
        eng_x.push(e);
        shannon_y.push(microbiome::shannon_index(&post));
        bc_y.push(microbiome::bray_curtis(&post, donor));
    }
    scenario.ecosystem.primals.push(node(
        "fmt",
        "FMT Engraftment",
        NodeType::Compute,
        &["science.microbiome.fmt"],
        vec![
            timeseries(
                "fmt_shannon",
                "Shannon vs Engraftment",
                "Engraftment",
                "Shannon H′",
                "nats",
                &eng_x,
                shannon_y,
            ),
            timeseries(
                "fmt_bc",
                "Bray-Curtis vs Engraftment",
                "Engraftment",
                "BC Dissimilarity",
                "BC",
                &eng_x,
                bc_y,
            ),
        ],
        vec![],
    ));

    let edges = vec![
        edge("diversity", "anderson", "evenness → disorder"),
        edge("anderson", "cdiff", "ξ → resistance"),
        edge("cdiff", "fmt", "FMT intervention"),
    ];
    (scenario, edges)
}
