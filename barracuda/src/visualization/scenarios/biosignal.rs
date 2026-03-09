// SPDX-License-Identifier: AGPL-3.0-or-later

use super::super::types::{ClinicalRange, HealthScenario, ScenarioEdge};
use super::{edge, gauge, node, scaffold, spectrum, timeseries};
use crate::biosignal;

/// Compute HRV power spectral density via DFT of RR interval series.
///
/// Returns `(frequencies_hz, power_ms2_per_hz)` covering the clinically
/// relevant 0–0.5 Hz band (VLF + LF + HF).
#[expect(
    clippy::cast_precision_loss,
    reason = "DFT indices and sample counts well within f64 mantissa"
)]
fn hrv_power_spectrum(rr_intervals_ms: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let n = rr_intervals_ms.len();
    if n < 4 {
        return (vec![], vec![]);
    }

    let mean_rr: f64 = rr_intervals_ms.iter().sum::<f64>() / n as f64;
    let mean_rr_s = mean_rr / 1000.0;
    let fs = 1.0 / mean_rr_s;

    let centered: Vec<f64> = rr_intervals_ms.iter().map(|&r| r - mean_rr).collect();

    let n_freq = n / 2;
    let mut freqs = Vec::with_capacity(n_freq);
    let mut power = Vec::with_capacity(n_freq);

    for k in 1..=n_freq {
        let freq = k as f64 * fs / n as f64;
        if freq > 0.5 {
            break;
        }

        let mut re = 0.0;
        let mut im = 0.0;
        for (i, &val) in centered.iter().enumerate() {
            let angle = 2.0 * std::f64::consts::PI * k as f64 * i as f64 / n as f64;
            re += val * angle.cos();
            im -= val * angle.sin();
        }
        let psd = (re * re + im * im) / (n as f64 * fs);
        freqs.push(freq);
        power.push(psd);
    }

    (freqs, power)
}

/// Build a complete biosignal study scenario with real computed data.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "signal sample counts well within safe range"
)]
#[expect(
    clippy::too_many_lines,
    reason = "4 sub-studies including signal generation"
)]
pub fn biosignal_study() -> (HealthScenario, Vec<ScenarioEdge>) {
    let mut s = scaffold(
        "healthSpring Biosignal Study",
        "Pan-Tompkins QRS, HRV metrics, PPG SpO2, multi-channel fusion — 4 experiments",
    );

    let fs = 360.0;
    let (ecg, true_peaks) = biosignal::generate_synthetic_ecg(fs, 10.0, 72.0, 0.05, 42);
    let result = biosignal::pan_tompkins(&ecg, fs);
    let metrics = biosignal::evaluate_detection(&result.peaks, &true_peaks, (0.1 * fs) as usize);
    let hr = biosignal::heart_rate_from_peaks(&result.peaks, fs);
    let ecg_t: Vec<f64> = (0..ecg.len()).map(|i| i as f64 / fs).collect();
    let ecg_ds_step = ecg.len() / 500;
    let ecg_ds_step = ecg_ds_step.max(1);
    let ecg_t_ds: Vec<f64> = ecg_t.iter().step_by(ecg_ds_step).copied().collect();
    let ecg_ds: Vec<f64> = ecg.iter().step_by(ecg_ds_step).copied().collect();
    let bp_ds: Vec<f64> = result
        .bandpass
        .iter()
        .step_by(ecg_ds_step)
        .copied()
        .collect();

    let deriv_ds: Vec<f64> = result
        .derivative
        .iter()
        .step_by(ecg_ds_step)
        .copied()
        .collect();
    let squared_ds: Vec<f64> = result
        .squared
        .iter()
        .step_by(ecg_ds_step)
        .copied()
        .collect();
    let mwi_ds: Vec<f64> = result.mwi.iter().step_by(ecg_ds_step).copied().collect();
    let ecg_t_ds2 = ecg_t_ds.clone();
    let ecg_t_ds3 = ecg_t_ds.clone();
    let ecg_t_ds4 = ecg_t_ds.clone();

    s.ecosystem.primals.push(node(
        "qrs",
        "Pan-Tompkins QRS Detection",
        "compute",
        &["science.biosignal.pan_tompkins"],
        vec![
            timeseries(
                "ecg_raw",
                "ECG (Raw)",
                "Time (s)",
                "Amplitude",
                "mV",
                ecg_t_ds.clone(),
                ecg_ds,
            ),
            timeseries(
                "ecg_bandpass",
                "ECG (Bandpass)",
                "Time (s)",
                "Amplitude",
                "mV",
                ecg_t_ds,
                bp_ds,
            ),
            timeseries(
                "ecg_derivative",
                "ECG (Derivative)",
                "Time (s)",
                "Amplitude",
                "mV/s",
                ecg_t_ds2,
                deriv_ds,
            ),
            timeseries(
                "ecg_squared",
                "ECG (Squared)",
                "Time (s)",
                "Amplitude",
                "mV²",
                ecg_t_ds3,
                squared_ds,
            ),
            timeseries(
                "ecg_mwi",
                "ECG (Moving Window Integration)",
                "Time (s)",
                "Amplitude",
                "a.u.",
                ecg_t_ds4,
                mwi_ds,
            ),
            gauge(
                "hr",
                "Heart Rate",
                hr,
                40.0,
                140.0,
                "bpm",
                [60.0, 100.0],
                [40.0, 60.0],
            ),
            gauge(
                "sensitivity",
                "Detection Sensitivity",
                metrics.sensitivity * 100.0,
                0.0,
                100.0,
                "%",
                [90.0, 100.0],
                [80.0, 90.0],
            ),
            gauge(
                "ppv",
                "Detection PPV",
                metrics.ppv * 100.0,
                0.0,
                100.0,
                "%",
                [90.0, 100.0],
                [80.0, 90.0],
            ),
        ],
        vec![ClinicalRange {
            label: "Normal HR".into(),
            min: 60.0,
            max: 100.0,
            status: "normal".into(),
        }],
    ));

    // HRV (exp021)
    let sdnn = biosignal::sdnn_ms(&result.peaks, fs);
    let rmssd = biosignal::rmssd_ms(&result.peaks, fs);
    let pnn50 = biosignal::pnn50(&result.peaks, fs);
    let rr_intervals: Vec<f64> = result
        .peaks
        .windows(2)
        .map(|w| (w[1] - w[0]) as f64 / fs * 1000.0)
        .collect();
    let rr_x: Vec<f64> = (0..rr_intervals.len()).map(|i| i as f64).collect();

    // HRV power spectrum via DFT of RR intervals
    // Standard frequency bands: VLF 0.003–0.04 Hz, LF 0.04–0.15 Hz, HF 0.15–0.4 Hz
    let (hrv_freqs, hrv_power) = hrv_power_spectrum(&rr_intervals);

    s.ecosystem.primals.push(node(
        "hrv",
        "HRV Metrics",
        "compute",
        &["science.biosignal.hrv"],
        vec![
            timeseries(
                "rr_tachogram",
                "RR Tachogram",
                "Beat #",
                "RR Interval",
                "ms",
                rr_x,
                rr_intervals,
            ),
            spectrum(
                "hrv_psd",
                "HRV Power Spectrum",
                hrv_freqs,
                hrv_power,
                "ms²/Hz",
            ),
            gauge(
                "sdnn",
                "SDNN",
                sdnn,
                0.0,
                200.0,
                "ms",
                [50.0, 150.0],
                [20.0, 50.0],
            ),
            gauge(
                "rmssd",
                "RMSSD",
                rmssd,
                0.0,
                100.0,
                "ms",
                [20.0, 60.0],
                [10.0, 20.0],
            ),
            gauge(
                "pnn50",
                "pNN50",
                pnn50,
                0.0,
                100.0,
                "%",
                [10.0, 50.0],
                [3.0, 10.0],
            ),
        ],
        vec![],
    ));

    // PPG SpO2 (exp022)
    let ppg = biosignal::generate_synthetic_ppg(fs, 5.0, 72.0, 97.0, 42);
    let (ac_red, dc_red) = biosignal::ppg_extract_ac_dc(&ppg.red);
    let (ac_ir, dc_ir) = biosignal::ppg_extract_ac_dc(&ppg.ir);
    let r_val = biosignal::ppg_r_value(ac_red, dc_red, ac_ir, dc_ir);
    let spo2 = biosignal::spo2_from_r(r_val);
    let r_sweep: Vec<f64> = (0..20).map(|i| 0.3 + 0.05 * f64::from(i)).collect();
    let spo2_sweep: Vec<f64> = r_sweep.iter().map(|&r| biosignal::spo2_from_r(r)).collect();
    s.ecosystem.primals.push(node(
        "spo2",
        "PPG SpO2",
        "compute",
        &["science.biosignal.ppg_spo2"],
        vec![
            timeseries(
                "r_calibration",
                "R-value vs SpO2 Calibration",
                "R-value",
                "SpO2",
                "%",
                r_sweep,
                spo2_sweep,
            ),
            gauge(
                "spo2",
                "SpO2",
                spo2,
                70.0,
                100.0,
                "%",
                [95.0, 100.0],
                [90.0, 95.0],
            ),
        ],
        vec![
            ClinicalRange {
                label: "Normal SpO2".into(),
                min: 95.0,
                max: 100.0,
                status: "normal".into(),
            },
            ClinicalRange {
                label: "Hypoxemia".into(),
                min: 70.0,
                max: 90.0,
                status: "critical".into(),
            },
        ],
    ));

    // Fusion (exp023)
    let eda = biosignal::generate_synthetic_eda(4.0, 30.0, 5.0, &[5.0, 12.0, 20.0, 25.0], 0.8, 42);
    let scl = biosignal::eda_scl(&eda, 16);
    let phasic = biosignal::eda_phasic(&eda, 16);
    let scr_peaks = biosignal::eda_detect_scr(&phasic, 0.1, 8);
    let fused = biosignal::fuse_channels(&result.peaks, fs, spo2, scr_peaks.len(), 30.0);
    let eda_t: Vec<f64> = (0..scl.len()).map(|i| i as f64 / 4.0).collect();
    s.ecosystem.primals.push(node(
        "fusion",
        "Multi-Channel Fusion",
        "compute",
        &["science.biosignal.fusion"],
        vec![
            timeseries(
                "eda_scl",
                "EDA Skin Conductance Level",
                "Time (s)",
                "SCL",
                "µS",
                eda_t.clone(),
                scl,
            ),
            timeseries(
                "eda_phasic",
                "EDA Phasic Component",
                "Time (s)",
                "Phasic",
                "µS",
                eda_t,
                phasic,
            ),
            gauge(
                "stress",
                "Stress Index",
                fused.stress_index,
                0.0,
                1.0,
                "index",
                [0.0, 0.4],
                [0.4, 0.7],
            ),
            gauge(
                "overall",
                "Overall Health Score",
                fused.overall_score,
                0.0,
                100.0,
                "score",
                [70.0, 100.0],
                [50.0, 70.0],
            ),
        ],
        vec![],
    ));

    let edges = vec![
        edge("qrs", "hrv", "R-peaks → HRV"),
        edge("spo2", "fusion", "SpO2 → fusion"),
        edge("hrv", "fusion", "HRV → fusion"),
    ];
    (s, edges)
}
