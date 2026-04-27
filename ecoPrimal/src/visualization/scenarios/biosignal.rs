// SPDX-License-Identifier: AGPL-3.0-or-later

use super::super::types::{
    ClinicalRange, ClinicalStatus, HealthScenario, NodeType, ScenarioEdge,
};
use super::{bar, edge, gauge, node, scaffold, spectrum, timeseries};
use crate::biosignal;
use crate::wfdb;

/// Compute HRV power spectral density via DFT of RR interval series.
///
/// Returns `(frequencies_hz, power_ms2_per_hz)` covering the clinically
/// relevant 0–0.5 Hz band (VLF + LF + HF).
///
/// Delegates to [`crate::biosignal::fft::rfft`] for the DFT computation,
/// then converts to one-sided PSD with appropriate frequency binning.
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
    let (re_all, im_all) = crate::biosignal::fft::rfft(&centered);

    let n_effective = ((re_all.len() - 1) * 2) as f64;
    let mut freqs = Vec::with_capacity(re_all.len());
    let mut power = Vec::with_capacity(re_all.len());

    for k in 1..re_all.len() {
        let freq = k as f64 * fs / n_effective;
        if freq > 0.5 {
            break;
        }
        let psd = re_all[k].mul_add(re_all[k], im_all[k] * im_all[k]) / (n_effective * fs);
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
    s.ecosystem.primals.push(node(
        "qrs",
        "Pan-Tompkins QRS Detection",
        NodeType::Compute,
        &["science.biosignal.pan_tompkins"],
        vec![
            timeseries(
                "ecg_raw",
                "ECG (Raw)",
                "Time (s)",
                "Amplitude",
                "mV",
                &ecg_t_ds,
                ecg_ds,
            ),
            timeseries(
                "ecg_bandpass",
                "ECG (Bandpass)",
                "Time (s)",
                "Amplitude",
                "mV",
                &ecg_t_ds,
                bp_ds,
            ),
            timeseries(
                "ecg_derivative",
                "ECG (Derivative)",
                "Time (s)",
                "Amplitude",
                "mV/s",
                &ecg_t_ds,
                deriv_ds,
            ),
            timeseries(
                "ecg_squared",
                "ECG (Squared)",
                "Time (s)",
                "Amplitude",
                "mV²",
                &ecg_t_ds,
                squared_ds,
            ),
            timeseries(
                "ecg_mwi",
                "ECG (Moving Window Integration)",
                "Time (s)",
                "Amplitude",
                "a.u.",
                &ecg_t_ds,
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
            status: ClinicalStatus::Normal,
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
        NodeType::Compute,
        &["science.biosignal.hrv"],
        vec![
            timeseries(
                "rr_tachogram",
                "RR Tachogram",
                "Beat #",
                "RR Interval",
                "ms",
                &rr_x,
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
    let r_sweep: Vec<f64> = (0..20)
        .map(|i| 0.05f64.mul_add(f64::from(i), 0.3))
        .collect();
    let spo2_sweep: Vec<f64> = r_sweep.iter().map(|&r| biosignal::spo2_from_r(r)).collect();
    s.ecosystem.primals.push(node(
        "spo2",
        "PPG SpO2",
        NodeType::Compute,
        &["science.biosignal.ppg_spo2"],
        vec![
            timeseries(
                "r_calibration",
                "R-value vs SpO2 Calibration",
                "R-value",
                "SpO2",
                "%",
                &r_sweep,
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
                status: ClinicalStatus::Normal,
            },
            ClinicalRange {
                label: "Hypoxemia".into(),
                min: 70.0,
                max: 90.0,
                status: ClinicalStatus::Critical,
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
        NodeType::Compute,
        &["science.biosignal.fusion"],
        vec![
            timeseries(
                "eda_scl",
                "EDA Skin Conductance Level",
                "Time (s)",
                "SCL",
                "µS",
                &eda_t,
                scl,
            ),
            timeseries(
                "eda_phasic",
                "EDA Phasic Component",
                "Time (s)",
                "Phasic",
                "µS",
                &eda_t,
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

    // WFDB ECG (synthetic Format 212 round-trip)
    build_wfdb_ecg_node(&mut s);

    let edges = vec![
        edge("qrs", "hrv", "R-peaks → HRV"),
        edge("spo2", "fusion", "SpO2 → fusion"),
        edge("hrv", "fusion", "HRV → fusion"),
        edge("qrs", "wfdb_ecg", "ECG → WFDB format"),
    ];
    (s, edges)
}

/// Build a WFDB ECG node exercising Format 212 encode/decode round-trip.
#[expect(
    clippy::too_many_lines,
    reason = "Format 212 encode + decode + annotation parse + 5 channels"
)]
#[expect(
    clippy::cast_precision_loss,
    reason = "sample counts well within f64 mantissa"
)]
fn build_wfdb_ecg_node(s: &mut HealthScenario) {
    let sample_freq = 360.0;
    let duration_s = 2.0;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "sample count from bounded product"
    )]
    let n_samples = (sample_freq * duration_s) as usize;

    let raw_ch1: Vec<i16> = (0..n_samples)
        .map(|i| {
            let t = i as f64 / sample_freq;
            let ecg_like = (2.0 * std::f64::consts::PI * 1.2 * t)
                .sin()
                .mul_add(200.0, (2.0 * std::f64::consts::PI * 0.2 * t).sin() * 50.0);
            #[expect(
                clippy::cast_possible_truncation,
                reason = "synthetic ECG amplitude bounded to ±300"
            )]
            let sample = ecg_like as i16;
            sample
        })
        .collect();
    let raw_ch2: Vec<i16> = raw_ch1.iter().map(|&s| s / 2).collect();

    let mut encoded = Vec::with_capacity(n_samples * 3 / 2 + 3);
    for i in 0..n_samples {
        let s1 = raw_ch1[i];
        let s2 = raw_ch2[i];
        #[expect(clippy::cast_sign_loss, reason = "masking to 8 bits")]
        let b0 = (s1 & 0xFF) as u8;
        #[expect(
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation,
            reason = "masking to 4+4 = 8 bits"
        )]
        let b1 = (((s1 >> 8) & 0x0F) | ((s2 & 0x0F) << 4)) as u8;
        #[expect(clippy::cast_sign_loss, reason = "shift result fits u8")]
        let b2 = ((s2 >> 4) & 0xFF) as u8;
        encoded.push(b0);
        encoded.push(b1);
        encoded.push(b2);
    }

    let decoded = match wfdb::decode_format_212(&encoded, n_samples, 2) {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!(error = %e, "WFDB Format 212 round-trip decode failed — skipping node");
            return;
        }
    };

    let gain = 200.0;
    let baseline = 0;
    let physical = wfdb::adc_to_physical(&decoded[0], gain, baseline);
    let time_axis: Vec<f64> = (0..physical.len())
        .map(|i| i as f64 / sample_freq)
        .collect();

    let ann_bytes: Vec<u8> = vec![0x14, 0x04, 0x50, 0x04, 0xA0, 0x04, 0xE0, 0x14, 0x00, 0x00];
    let annotations = wfdb::parse_annotations(&ann_bytes).unwrap_or_default();

    let mut beat_counts = std::collections::HashMap::new();
    for ann in &annotations {
        let label = format!("{:?}", ann.beat_type);
        *beat_counts.entry(label).or_insert(0u32) += 1;
    }
    let (beat_labels, beat_vals): (Vec<String>, Vec<f64>) = if beat_counts.is_empty() {
        (vec!["Normal".into()], vec![1.0])
    } else {
        beat_counts
            .into_iter()
            .map(|(k, v)| (k, f64::from(v)))
            .unzip()
    };

    s.ecosystem.primals.push(node(
        "wfdb_ecg",
        "WFDB ECG Format Parser",
        NodeType::Compute,
        &["science.biosignal.wfdb_format212"],
        vec![
            timeseries(
                "wfdb_signal",
                "Decoded ECG (Format 212)",
                "Time (s)",
                "Voltage",
                "mV",
                &time_axis,
                physical,
            ),
            bar(
                "beat_types",
                "Beat Type Distribution",
                &beat_labels,
                beat_vals,
                "count",
            ),
            gauge(
                "wfdb_fs",
                "Sampling Frequency",
                sample_freq,
                0.0,
                1000.0,
                "Hz",
                [250.0, 500.0],
                [100.0, 250.0],
            ),
            gauge(
                "wfdb_annotations",
                "Annotations",
                annotations.len() as f64,
                0.0,
                100.0,
                "count",
                [1.0, 50.0],
                [0.0, 1.0],
            ),
            gauge(
                "wfdb_duration",
                "Record Duration",
                duration_s,
                0.0,
                60.0,
                "seconds",
                [1.0, 30.0],
                [0.0, 1.0],
            ),
        ],
        vec![],
    ));
}
