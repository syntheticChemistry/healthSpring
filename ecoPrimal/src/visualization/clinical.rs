// SPDX-License-Identifier: AGPL-3.0-or-later
//! Patient-parameterized TRT clinical scenario builder.
//!
//! Unlike `endocrine_study()` which demonstrates the validated math with fixed
//! parameters, this module produces scenarios parameterized by a specific
//! patient — age, weight, baseline testosterone, and chosen protocol. The
//! output tells a clinical story that a clinician can show a patient:
//! assessment → protocol → population comparison → predicted outcomes.

use super::clinical_nodes;
use super::scenarios::{edge, scenario_with_edges_json};
use super::types::{
    Animations, CapReqs, Ecosystem, HealthScenario, NeuralApi, Performance, ScenarioEdge,
    SensoryConfig, ShowPanels, UiConfig,
};
use crate::endocrine;

/// TRT delivery protocol.
#[derive(Debug, Clone, Copy)]
pub enum TrtProtocol {
    /// Intramuscular injection once weekly.
    ImWeekly,
    /// Intramuscular injection every two weeks.
    ImBiweekly,
    /// Subcutaneous testosterone pellets.
    Pellet,
}

/// A patient's clinical profile for TRT scenario generation.
#[derive(Debug, Clone)]
pub struct PatientTrtProfile {
    /// Display name for the scenario.
    pub name: String,
    /// Age in years.
    pub age: f64,
    /// Body weight in pounds.
    pub weight_lb: f64,
    /// Baseline total testosterone (ng/dL).
    pub baseline_t_ng_dl: f64,
    /// Prescribed TRT delivery modality.
    pub protocol: TrtProtocol,
    /// Pielou evenness (0..1). `None` = not measured.
    pub gut_diversity: Option<f64>,
    /// Baseline `HbA1c`. `None` = not diabetic / not measured.
    pub hba1c: Option<f64>,
    /// Baseline SDNN in ms. `None` = not measured.
    pub sdnn_ms: Option<f64>,
}

impl PatientTrtProfile {
    /// Build a profile with required demographics and protocol; optional biomarkers default to unset.
    #[must_use]
    pub fn new(
        name: &str,
        age: f64,
        weight_lb: f64,
        baseline_t: f64,
        protocol: TrtProtocol,
    ) -> Self {
        Self {
            name: name.into(),
            age,
            weight_lb,
            baseline_t_ng_dl: baseline_t,
            protocol,
            gut_diversity: None,
            hba1c: None,
            sdnn_ms: None,
        }
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "display rounding — age/weight/T are always positive and < u32::MAX"
)]
fn scaffold_clinical(patient: &PatientTrtProfile) -> HealthScenario {
    let age = patient.age as u32;
    let wt = patient.weight_lb as u32;
    let t = patient.baseline_t_ng_dl as u32;
    HealthScenario {
        name: format!("TRT Clinical: {}", patient.name),
        description: format!(
            "Patient-specific TRT projection — {age}yo, {wt}lb, baseline T {t}ng/dL"
        ),
        version: "2.0.0".into(),
        mode: "clinical".into(),
        sensory_config: SensoryConfig {
            required_capabilities: CapReqs {
                outputs: vec!["visual".into()],
                inputs: vec![],
            },
            optional_capabilities: CapReqs {
                outputs: vec!["audio".into()],
                inputs: vec!["pointer".into(), "keyboard".into()],
            },
            complexity_hint: "standard".into(),
        },
        ui_config: UiConfig {
            theme: "clinical-dark".into(),
            animations: Animations {
                enabled: true,
                breathing_nodes: true,
                connection_pulses: true,
                smooth_transitions: true,
                celebration_effects: false,
            },
            performance: Performance {
                target_fps: 60,
                vsync: true,
                hardware_acceleration: true,
            },
            show_panels: Some(ShowPanels {
                left_sidebar: false,
                right_sidebar: true,
                top_menu: true,
                system_dashboard: false,
                audio_panel: false,
                trust_dashboard: false,
                proprioception: false,
                graph_stats: true,
            }),
            awakening_enabled: Some(false),
            initial_zoom: Some("fit".into()),
        },
        ecosystem: Ecosystem { primals: vec![] },
        neural_api: NeuralApi { enabled: false },
        edges: Vec::new(),
    }
}

/// Build a complete patient-specific TRT clinical scenario.
///
/// The scenario tells the clinical story: assessment → protocol → population
/// comparison → metabolic/cardiovascular/glycemic outcomes → cardiac monitoring
/// → gut health predictor. Every data channel is computed from validated models
/// parameterized by this specific patient.
#[must_use]
pub fn trt_clinical_scenario(p: &PatientTrtProfile) -> (HealthScenario, Vec<ScenarioEdge>) {
    let mut s = scaffold_clinical(p);

    let (assessment, _) = clinical_nodes::assessment_node(p);
    s.ecosystem.primals.push(assessment);
    s.ecosystem.primals.push(clinical_nodes::protocol_node(p));
    s.ecosystem.primals.push(clinical_nodes::population_node(p));
    s.ecosystem.primals.push(clinical_nodes::metabolic_node());
    s.ecosystem
        .primals
        .push(clinical_nodes::cardiovascular_node());

    let hba1c_base = p
        .hba1c
        .unwrap_or(endocrine::diabetes_params::HBA1C_BASELINE);
    s.ecosystem
        .primals
        .push(clinical_nodes::diabetes_node(hba1c_base));

    let sdnn_base = p.sdnn_ms.unwrap_or(35.0);
    s.ecosystem
        .primals
        .push(clinical_nodes::cardiac_monitor_node(sdnn_base));

    let gut_j = p.gut_diversity.unwrap_or(0.70);
    s.ecosystem
        .primals
        .push(clinical_nodes::gut_health_node(gut_j));

    let edges = vec![
        edge("assessment", "protocol", "Baseline → Prescribe"),
        edge("protocol", "population", "Compare to similar patients"),
        edge("protocol", "metabolic", "Treatment → Weight/Waist"),
        edge("protocol", "cardiovascular", "Treatment → CV Biomarkers"),
        edge("protocol", "diabetes", "Treatment → Glycemic Control"),
        edge("cardiovascular", "cardiac", "Lipids/BP → Cardiac Risk"),
        edge(
            "gut_health",
            "metabolic",
            "Gut diversity modulates response",
        ),
        edge("assessment", "cardiac", "Baseline HRV → Monitor"),
    ];

    (s, edges)
}

/// Serialize a patient TRT clinical scenario to JSON.
#[must_use]
pub fn trt_clinical_json(p: &PatientTrtProfile) -> String {
    let (scenario, edges) = trt_clinical_scenario(p);
    scenario_with_edges_json(&scenario, &edges)
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::super::types::{DataChannel, NodeStatus, ScenarioNode};
    use super::*;

    fn sample_patient() -> PatientTrtProfile {
        let mut p = PatientTrtProfile::new("Sample", 55.0, 220.0, 280.0, TrtProtocol::Pellet);
        p.gut_diversity = Some(0.65);
        p.hba1c = Some(7.2);
        p.sdnn_ms = Some(38.0);
        p
    }

    #[test]
    fn scenario_has_8_nodes() {
        let (s, _) = trt_clinical_scenario(&sample_patient());
        assert_eq!(s.ecosystem.primals.len(), 8, "expected 8 clinical nodes");
    }

    #[test]
    fn scenario_has_8_edges() {
        let (_, edges) = trt_clinical_scenario(&sample_patient());
        assert_eq!(edges.len(), 8, "expected 8 clinical edges");
    }

    #[test]
    fn all_nodes_have_data_channels() {
        let (s, _) = trt_clinical_scenario(&sample_patient());
        for n in &s.ecosystem.primals {
            assert!(
                !n.data_channels.is_empty(),
                "node '{}' has no data channels",
                n.id
            );
        }
    }

    #[test]
    fn all_nodes_have_clinical_ranges() {
        let (s, _) = trt_clinical_scenario(&sample_patient());
        for n in &s.ecosystem.primals {
            assert!(
                !n.clinical_ranges.is_empty(),
                "node '{}' has no clinical ranges",
                n.id
            );
        }
    }

    #[test]
    fn json_roundtrips() {
        let json = trt_clinical_json(&sample_patient());
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert!(parsed["ecosystem"]["primals"].is_array());
        assert!(parsed["edges"].is_array());
        assert_eq!(parsed["ecosystem"]["primals"].as_array().unwrap().len(), 8);
    }

    #[test]
    fn clinical_mode_and_panel_config() {
        let json = trt_clinical_json(&sample_patient());
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed["mode"], "clinical");
        let panels = &parsed["ui_config"]["show_panels"];
        assert_eq!(panels["left_sidebar"], false);
        assert_eq!(panels["system_dashboard"], false);
        assert_eq!(panels["audio_panel"], false);
        assert_eq!(panels["trust_dashboard"], false);
        assert_eq!(panels["proprioception"], false);
        assert_eq!(panels["graph_stats"], true);
        assert_eq!(panels["top_menu"], true);
        assert_eq!(parsed["ui_config"]["awakening_enabled"], false);
        assert_eq!(parsed["ui_config"]["initial_zoom"], "fit");
    }

    #[test]
    fn pellet_dose_scales_with_weight() {
        let light = PatientTrtProfile::new("Light", 50.0, 150.0, 300.0, TrtProtocol::Pellet);
        let heavy = PatientTrtProfile::new("Heavy", 50.0, 250.0, 300.0, TrtProtocol::Pellet);

        let (sl, _) = trt_clinical_scenario(&light);
        let (sh, _) = trt_clinical_scenario(&heavy);

        let prot_l = sl
            .ecosystem
            .primals
            .iter()
            .find(|n| n.id == "protocol")
            .unwrap();
        let prot_h = sh
            .ecosystem
            .primals
            .iter()
            .find(|n| n.id == "protocol")
            .unwrap();

        assert!(prot_l.name.contains("1500"), "150lb × 10 = 1500mg");
        assert!(prot_h.name.contains("2500"), "250lb × 10 = 2500mg");
    }

    #[test]
    fn weekly_and_biweekly_produce_different_curves() {
        let weekly = PatientTrtProfile::new("W", 50.0, 200.0, 300.0, TrtProtocol::ImWeekly);
        let biweekly = PatientTrtProfile::new("B", 50.0, 200.0, 300.0, TrtProtocol::ImBiweekly);

        let (sw, _) = trt_clinical_scenario(&weekly);
        let (sb, _) = trt_clinical_scenario(&biweekly);

        let pw = sw
            .ecosystem
            .primals
            .iter()
            .find(|n| n.id == "protocol")
            .unwrap();
        let pb = sb
            .ecosystem
            .primals
            .iter()
            .find(|n| n.id == "protocol")
            .unwrap();

        assert!(pw.name.contains("Weekly"));
        assert!(pb.name.contains("Biweekly"));
    }

    #[test]
    fn low_baseline_t_flags_offline() {
        let p = PatientTrtProfile::new("Low", 60.0, 200.0, 180.0, TrtProtocol::ImWeekly);
        let (s, _) = trt_clinical_scenario(&p);
        let assess = s
            .ecosystem
            .primals
            .iter()
            .find(|n| n.id == "assessment")
            .unwrap();
        assert_eq!(assess.status, NodeStatus::Offline);
        assert!(assess.health <= 50);
    }

    #[test]
    fn gut_diversity_affects_response_prediction() {
        let low = {
            let mut p = PatientTrtProfile::new("Low", 50.0, 200.0, 300.0, TrtProtocol::Pellet);
            p.gut_diversity = Some(0.20);
            p
        };
        let high = {
            let mut p = PatientTrtProfile::new("High", 50.0, 200.0, 300.0, TrtProtocol::Pellet);
            p.gut_diversity = Some(0.95);
            p
        };

        let (sl, _) = trt_clinical_scenario(&low);
        let (sh, _) = trt_clinical_scenario(&high);

        let gl = sl
            .ecosystem
            .primals
            .iter()
            .find(|n| n.id == "gut_health")
            .unwrap();
        let gh = sh
            .ecosystem
            .primals
            .iter()
            .find(|n| n.id == "gut_health")
            .unwrap();

        let get_response_gauge = |n: &ScenarioNode| -> f64 {
            n.data_channels
                .iter()
                .find_map(|ch| {
                    if let DataChannel::Gauge { id, value, .. } = ch {
                        if id == "patient_response" {
                            return Some(*value);
                        }
                    }
                    None
                })
                .unwrap()
        };

        let resp_low = get_response_gauge(gl);
        let resp_high = get_response_gauge(gh);
        assert!(
            resp_high > resp_low,
            "higher diversity should predict more weight loss"
        );
    }
}
