// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

//! Standalone interactive diagnostic UI for healthSpring.
//!
//! Generates a synthetic patient, runs the full diagnostic pipeline +
//! population Monte Carlo, and renders an interactive egui dashboard
//! with topology graph, time-series charts, distributions, and gauges.
//!
//! This is the prototype UI that petalTongue absorbs.

use eframe::egui::{self, RichText, ScrollArea};
use healthspring_barracuda::diagnostic::{
    PatientProfile, Sex, assess_patient, population_montecarlo,
};
use healthspring_barracuda::visualization::{
    ScenarioNode, annotate_population, assessment_to_scenario,
};
use petaltongue_health::{render, theme};

struct HealthApp {
    nodes: Vec<ScenarioNode>,
    selected_node: Option<String>,
}

impl HealthApp {
    fn new() -> Self {
        let mut patient = PatientProfile::minimal(55.0, 85.0, Sex::Male);
        patient.testosterone_ng_dl = Some(450.0);
        patient.on_trt = true;
        patient.trt_months = 12.0;
        patient.gut_abundances = Some(vec![0.30, 0.25, 0.18, 0.12, 0.08, 0.04, 0.03]);
        patient.ecg_peaks = Some(vec![0, 320, 630, 950, 1260, 1580, 1900, 2210, 2530, 2850]);
        patient.ecg_fs = 360.0;
        patient.ppg_spo2 = Some(97.5);

        let assessment = assess_patient(&patient);
        let pop = population_montecarlo(&patient, 1000, 42);
        let scenario = assessment_to_scenario(&assessment, "Male 55y — TRT 12mo");
        let annotated = annotate_population(scenario, &pop);

        Self {
            nodes: annotated.ecosystem.primals,
            selected_node: Some("patient".into()),
        }
    }
}

impl eframe::App for HealthApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("healthSpring Diagnostic")
                        .heading()
                        .color(theme::HEALTHY),
                );
                ui.separator();
                ui.label(
                    RichText::new("petalTongue Evolution Prototype")
                        .color(theme::TEXT_DIM)
                        .italics(),
                );
            });
        });

        egui::TopBottomPanel::bottom("stats_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for node in &self.nodes {
                    let color = theme::health_color(node.health);
                    let short: String = node.name.chars().take(12).collect();
                    if ui
                        .selectable_label(
                            self.selected_node.as_deref() == Some(&node.id),
                            RichText::new(format!("{short}: {}%", node.health)).color(color),
                        )
                        .clicked()
                    {
                        self.selected_node = Some(node.id.clone());
                    }
                }
            });
        });

        egui::SidePanel::right("detail_panel")
            .min_width(350.0)
            .max_width(500.0)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    if let Some(ref sel_id) = self.selected_node {
                        if let Some(node) = self.nodes.iter().find(|n| &n.id == sel_id) {
                            render::draw_node_detail(ui, node);
                        }
                    } else {
                        ui.label(
                            RichText::new("Click a node to inspect")
                                .color(theme::TEXT_DIM)
                                .italics(),
                        );
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(
                RichText::new("Diagnostic Topology")
                    .heading()
                    .color(theme::TEXT_PRIMARY),
            );
            render::draw_topology(ui, &self.nodes, &mut self.selected_node);
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_title("healthSpring Diagnostic — petalTongue Prototype"),
        ..Default::default()
    };
    eframe::run_native(
        "healthspring-ui",
        options,
        Box::new(|_cc| Ok(Box::new(HealthApp::new()))),
    )
}
