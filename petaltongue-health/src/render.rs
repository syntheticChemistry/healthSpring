// SPDX-License-Identifier: AGPL-3.0-or-later
//! Rendering functions for diagnostic data channels using egui.
//!
//! Each function takes an `egui::Ui` and a `DataChannel` and draws the
//! appropriate visualization. These are the prototypes petalTongue absorbs
//! into `petal-tongue-graph`.
//!
//! Rendering code necessarily converts between f64 domain values and f32
//! pixel coordinates; these truncations are intentional and harmless.
#![expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "rendering: domain f64 → pixel f32 truncation is intentional"
)]

use crate::theme;
use eframe::egui::{self, Color32, RichText, Ui};
use egui_plot::{Bar, BarChart, Line, Plot, PlotPoints, VLine};
use healthspring_barracuda::visualization::{DataChannel, ScenarioNode};

/// Draw a single data channel.
pub fn draw_channel(ui: &mut Ui, channel: &DataChannel) {
    match channel {
        DataChannel::TimeSeries {
            label,
            x_label,
            y_label,
            x_values,
            y_values,
            ..
        } => draw_timeseries(ui, label, x_label, y_label, x_values, y_values),
        DataChannel::Distribution {
            label,
            values,
            mean,
            std,
            patient_value,
            ..
        } => draw_distribution(ui, label, values, *mean, *std, *patient_value),
        DataChannel::Bar {
            label,
            categories,
            values,
            ..
        } => draw_bar_chart(ui, label, categories, values),
        DataChannel::Gauge {
            label,
            value,
            min,
            max,
            unit,
            normal_range,
            warning_range,
            ..
        } => draw_gauge(
            ui,
            label,
            *value,
            *min,
            *max,
            unit,
            normal_range,
            warning_range,
        ),
    }
}

fn draw_timeseries(
    ui: &mut Ui,
    label: &str,
    x_label: &str,
    y_label: &str,
    x_values: &[f64],
    y_values: &[f64],
) {
    ui.label(RichText::new(label).strong().color(theme::TEXT_PRIMARY));
    let points: PlotPoints = x_values
        .iter()
        .zip(y_values.iter())
        .map(|(&x, &y)| [x, y])
        .collect();
    let line = Line::new(points).color(theme::INFO).name(label);

    Plot::new(label)
        .height(160.0)
        .x_axis_label(x_label)
        .y_axis_label(y_label)
        .show_axes(true)
        .show(ui, |plot_ui| {
            plot_ui.line(line);
        });
}

fn draw_distribution(
    ui: &mut Ui,
    label: &str,
    values: &[f64],
    mean: f64,
    std: f64,
    patient_value: f64,
) {
    ui.label(RichText::new(label).strong().color(theme::TEXT_PRIMARY));

    let n_bins = 30;
    let (lo, hi) = values
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(v), hi.max(v))
        });
    let bin_width = (hi - lo) / n_bins as f64;
    if bin_width <= 0.0 {
        ui.label("No spread in distribution");
        return;
    }

    let mut counts = vec![0u32; n_bins];
    for &v in values {
        let idx = ((v - lo) / bin_width).floor() as usize;
        let idx = idx.min(n_bins - 1);
        counts[idx] += 1;
    }

    let bars: Vec<Bar> = counts
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            let center = lo + (i as f64 + 0.5) * bin_width;
            Bar::new(center, f64::from(c))
                .width(bin_width * 0.9)
                .fill(theme::POPULATION.gamma_multiply(0.7))
        })
        .collect();

    let chart = BarChart::new(bars).name("Distribution");

    Plot::new(label)
        .height(160.0)
        .show_axes(true)
        .show(ui, |plot_ui| {
            plot_ui.bar_chart(chart);
            plot_ui.vline(
                VLine::new(mean)
                    .color(theme::INFO)
                    .name(format!("Mean: {mean:.4}")),
            );
            plot_ui.vline(VLine::new(mean + std).color(theme::TEXT_DIM).name("+1 SD"));
            plot_ui.vline(VLine::new(mean - std).color(theme::TEXT_DIM).name("-1 SD"));
            if patient_value > 0.0 {
                plot_ui.vline(
                    VLine::new(lo + patient_value * (hi - lo))
                        .color(theme::WARNING)
                        .name("Patient"),
                );
            }
        });
}

fn draw_bar_chart(ui: &mut Ui, label: &str, categories: &[String], values: &[f64]) {
    ui.label(RichText::new(label).strong().color(theme::TEXT_PRIMARY));

    let bars: Vec<Bar> = values
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let name = categories.get(i).map_or("?", String::as_str);
            Bar::new(i as f64, v)
                .width(0.7)
                .name(name)
                .fill(theme::INFO.gamma_multiply(0.8))
        })
        .collect();

    let chart = BarChart::new(bars).name(label);

    Plot::new(label)
        .height(120.0)
        .show_axes(true)
        .show(ui, |plot_ui| {
            plot_ui.bar_chart(chart);
        });
}

#[expect(
    clippy::too_many_arguments,
    reason = "matches DataChannel::Gauge fields"
)]
fn draw_gauge(
    ui: &mut Ui,
    label: &str,
    value: f64,
    min: f64,
    max: f64,
    unit: &str,
    normal_range: &[f64; 2],
    warning_range: &[f64; 2],
) {
    let color = if value >= normal_range[0] && value <= normal_range[1] {
        theme::HEALTHY
    } else if value >= warning_range[0] && value <= warning_range[1] {
        theme::WARNING
    } else {
        theme::CRITICAL
    };

    ui.horizontal(|ui| {
        ui.label(RichText::new(label).color(theme::TEXT_DIM));
        ui.label(
            RichText::new(format!("{value:.1} {unit}"))
                .strong()
                .color(color),
        );
    });

    let frac = ((value - min) / (max - min)).clamp(0.0, 1.0);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().min(300.0), 14.0),
        egui::Sense::hover(),
    );

    let painter = ui.painter();
    painter.rect_filled(rect, 4.0, theme::BG_CARD);

    let normal_left = ((normal_range[0] - min) / (max - min)).clamp(0.0, 1.0);
    let normal_right = ((normal_range[1] - min) / (max - min)).clamp(0.0, 1.0);
    let nr = egui::Rect::from_min_max(
        egui::pos2(rect.left() + rect.width() * normal_left as f32, rect.top()),
        egui::pos2(
            rect.left() + rect.width() * normal_right as f32,
            rect.bottom(),
        ),
    );
    painter.rect_filled(nr, 2.0, theme::HEALTHY.gamma_multiply(0.2));

    let bar_width = rect.width() * frac as f32;
    let bar_rect = egui::Rect::from_min_size(rect.min, egui::vec2(bar_width, rect.height()));
    painter.rect_filled(bar_rect, 4.0, color.gamma_multiply(0.8));
}

/// Draw a node detail panel with all its data channels.
pub fn draw_node_detail(ui: &mut Ui, node: &ScenarioNode) {
    let color = theme::health_color(node.health);

    ui.horizontal(|ui| {
        ui.label(RichText::new(&node.name).heading().color(color));
        ui.label(
            RichText::new(format!("{}%", node.health))
                .strong()
                .color(color),
        );
        ui.label(RichText::new(&node.status).color(theme::TEXT_DIM).italics());
    });

    if !node.capabilities.is_empty() {
        ui.horizontal_wrapped(|ui| {
            for cap in &node.capabilities {
                let short = cap.rsplit('.').next().unwrap_or(cap);
                ui.label(
                    RichText::new(short)
                        .small()
                        .background_color(theme::BG_CARD)
                        .color(theme::TEXT_DIM),
                );
            }
        });
    }

    ui.separator();

    for channel in &node.data_channels {
        draw_channel(ui, channel);
        ui.add_space(8.0);
    }
}

/// Draw the topology as a simple node graph.
pub fn draw_topology(ui: &mut Ui, nodes: &[ScenarioNode], selected: &mut Option<String>) {
    let (response, painter) = ui.allocate_painter(
        egui::vec2(ui.available_width(), 400.0),
        egui::Sense::click(),
    );
    let rect = response.rect;

    let scale_x = rect.width() / 960.0;
    let scale_y = rect.height() / 700.0;

    painter.rect_filled(rect, 8.0, theme::BG_PANEL);

    for node in nodes {
        let cx = rect.left() + node.position.x as f32 * scale_x;
        let cy = rect.top() + node.position.y as f32 * scale_y;
        let center = egui::pos2(cx, cy);
        let radius = 28.0;
        let color = theme::health_color(node.health);
        let is_selected = selected.as_deref() == Some(&node.id);

        if is_selected {
            painter.circle_filled(center, radius + 4.0, Color32::WHITE);
        }
        painter.circle_filled(center, radius, color.gamma_multiply(0.85));
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            node.health.to_string(),
            egui::FontId::proportional(14.0),
            Color32::WHITE,
        );

        let name_short: String = node.name.chars().take(14).collect();
        painter.text(
            egui::pos2(cx, cy + radius + 12.0),
            egui::Align2::CENTER_CENTER,
            name_short,
            egui::FontId::proportional(11.0),
            theme::TEXT_DIM,
        );

        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                if center.distance(pos) < radius + 5.0 {
                    *selected = Some(node.id.clone());
                }
            }
        }
    }
}
