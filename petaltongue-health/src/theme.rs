// SPDX-License-Identifier: AGPL-3.0-or-later
//! Clinical color theme for healthSpring diagnostic UI.

use eframe::egui::Color32;

pub const HEALTHY: Color32 = Color32::from_rgb(46, 204, 113);
pub const WARNING: Color32 = Color32::from_rgb(241, 196, 15);
pub const CRITICAL: Color32 = Color32::from_rgb(231, 76, 60);
pub const INFO: Color32 = Color32::from_rgb(52, 152, 219);
pub const POPULATION: Color32 = Color32::from_rgb(155, 89, 182);
pub const BG_PANEL: Color32 = Color32::from_rgb(30, 30, 40);
pub const BG_CARD: Color32 = Color32::from_rgb(40, 42, 54);
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(248, 248, 242);
pub const TEXT_DIM: Color32 = Color32::from_rgb(128, 128, 160);

/// Map a health score (0..100) to a clinical color.
#[must_use]
pub fn health_color(health: u8) -> Color32 {
    if health >= 90 {
        HEALTHY
    } else if health >= 50 {
        WARNING
    } else {
        CRITICAL
    }
}
