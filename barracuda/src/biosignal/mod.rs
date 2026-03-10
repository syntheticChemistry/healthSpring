// SPDX-License-Identifier: AGPL-3.0-or-later
//! Physiological biosignal processing pipelines.
//!
//! Leverages `BarraCUDA` attention mechanisms + NPU inference for
//! real-time health monitoring on sovereign hardware.
//! - ECG anomaly detection (R-peak, arrhythmia)
//! - PPG-based `SpO2` estimation
//! - Continuous glucose monitoring analytics
//! - Heart rate variability (HRV) analysis
//! - Wearable sensor fusion (IMU + PPG + temperature)
//!
//! ## Constants
//!
//! ECG bandpass range and MWI window follow Pan & Tompkins (1985).
//! `SpO2` calibration coefficients are from Beer-Lambert linearization.
//!
//! ## Tier 1 (CPU) — Exp020
//!
//! Pan-Tompkins QRS detection algorithm:
//! 1. Bandpass filter (5–15 Hz via frequency domain)
//! 2. Five-point derivative
//! 3. Squaring (nonlinear amplification)
//! 4. Moving-window integration
//! 5. Adaptive peak detection with refractory period

pub mod ecg;
pub mod eda;
pub mod fft;
pub mod fusion;
pub mod hrv;
pub mod ppg;

// Re-export all public items at the module level for backwards compatibility.
pub use ecg::{
    DetectionMetrics, PanTompkinsResult, bandpass_filter, derivative_filter, detect_peaks,
    evaluate_detection, generate_synthetic_ecg, moving_window_integration, pan_tompkins, squaring,
};
pub use eda::{eda_detect_scr, eda_phasic, eda_scl, generate_synthetic_eda};
pub use fusion::{FusedHealthAssessment, fuse_channels};
pub use hrv::{heart_rate_from_peaks, pnn50, rmssd_ms, sdnn_ms};
pub use ppg::{SyntheticPpg, generate_synthetic_ppg, ppg_extract_ac_dc, ppg_r_value, spo2_from_r};
