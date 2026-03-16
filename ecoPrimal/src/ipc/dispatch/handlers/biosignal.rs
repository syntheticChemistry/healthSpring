// SPDX-License-Identifier: AGPL-3.0-or-later
//! Biosignal and WFDB capability handlers.

use serde_json::Value;

use crate::{biosignal, wfdb};

use super::{f, fa, missing, sz_or, sza};

pub fn dispatch_pan_tompkins(params: &Value) -> Value {
    let Some(signal) = fa(params, "signal") else {
        return missing("signal");
    };
    let fs = f(params, "fs").unwrap_or(360.0);
    let result = biosignal::pan_tompkins(&signal, fs);
    let hr = biosignal::heart_rate_from_peaks(&result.peaks, fs);
    serde_json::json!({
        "peaks": result.peaks,
        "n_beats": result.peaks.len(),
        "heart_rate_bpm": hr,
    })
}

pub fn dispatch_hrv(params: &Value) -> Value {
    let peaks: Vec<usize> = sza(params, "peaks").unwrap_or_default();
    if peaks.len() < 3 {
        return missing("peaks (array of at least 3 R-peak indices)");
    }
    let fs = f(params, "fs").unwrap_or(360.0);
    serde_json::json!({
        "heart_rate_bpm": biosignal::heart_rate_from_peaks(&peaks, fs),
        "sdnn_ms": biosignal::sdnn_ms(&peaks, fs),
        "rmssd_ms": biosignal::rmssd_ms(&peaks, fs),
        "pnn50": biosignal::pnn50(&peaks, fs),
    })
}

pub fn dispatch_ppg_spo2(params: &Value) -> Value {
    let (Some(ac_red), Some(dc_red), Some(ac_ir), Some(dc_ir)) = (
        f(params, "ac_red"),
        f(params, "dc_red"),
        f(params, "ac_ir"),
        f(params, "dc_ir"),
    ) else {
        return missing("ac_red, dc_red, ac_ir, dc_ir");
    };
    let r = biosignal::ppg_r_value(ac_red, dc_red, ac_ir, dc_ir);
    let spo2 = biosignal::spo2_from_r(r);
    serde_json::json!({"r_value": r, "spo2_percent": spo2})
}

pub fn dispatch_eda(params: &Value) -> Value {
    let Some(signal) = fa(params, "signal") else {
        return missing("signal");
    };
    let window = sz_or(params, "window_samples", 50);
    let scl = biosignal::eda_scl(&signal, window);
    let phasic = biosignal::eda_phasic(&signal, window);
    serde_json::json!({
        "scl_samples": scl.len(),
        "phasic_samples": phasic.len(),
        "scl_mean": scl.iter().sum::<f64>() / crate::validation::len_f64(scl.len().max(1)),
    })
}

pub fn dispatch_eda_stress(params: &Value) -> Value {
    let Some(signal) = fa(params, "signal") else {
        return missing("signal");
    };
    let min_interval = sz_or(params, "min_interval_samples", 50);
    let threshold = f(params, "threshold_us").unwrap_or(0.05);
    let fs = f(params, "fs").unwrap_or(4.0);
    let duration_s = crate::validation::len_f64(signal.len()) / fs;
    let scrs = biosignal::eda_detect_scr(&signal, threshold, min_interval);
    let rate = biosignal::scr_rate(scrs.len(), duration_s);
    let mean_scl = if signal.is_empty() {
        0.0
    } else {
        signal.iter().sum::<f64>() / crate::validation::len_f64(signal.len())
    };
    let recovery_s = f(params, "recovery_s").unwrap_or(3.0);
    let stress_idx = biosignal::compute_stress_index(rate, mean_scl, recovery_s);
    serde_json::json!({
        "scr_count": scrs.len(),
        "scr_rate_per_min": rate,
        "stress_index": stress_idx,
    })
}

pub fn dispatch_arrhythmia(params: &Value) -> Value {
    let Some(signal) = fa(params, "signal") else {
        return missing("signal");
    };
    let fs = f(params, "fs").unwrap_or(360.0);
    let half_width = sz_or(params, "half_width", 25);
    let min_corr = f(params, "min_correlation").unwrap_or(0.7);
    let pt = biosignal::pan_tompkins(&signal, fs);

    let normal_waveform = biosignal::generate_normal_template(half_width * 2 + 1);
    let normal_template = biosignal::BeatTemplate {
        class: biosignal::BeatClass::Normal,
        waveform: normal_waveform,
    };
    let results =
        biosignal::classify_all_beats(&signal, &pt.peaks, &[normal_template], half_width, min_corr);
    let n_normal = results
        .iter()
        .filter(|r| r.class == biosignal::BeatClass::Normal)
        .count();
    serde_json::json!({
        "total_beats": results.len(),
        "normal": n_normal,
        "abnormal": results.len() - n_normal,
    })
}

pub fn dispatch_fuse(params: &Value) -> Value {
    let peaks: Vec<usize> = sza(params, "peaks").unwrap_or_default();
    let fs = f(params, "fs").unwrap_or(360.0);
    let spo2 = f(params, "spo2").unwrap_or(98.0);
    let scr_count = sz_or(params, "scr_count", 0);
    let eda_duration_s = f(params, "eda_duration_s").unwrap_or(0.0);

    let assessment = biosignal::fuse_channels(&peaks, fs, spo2, scr_count, eda_duration_s);
    serde_json::json!({
        "heart_rate_bpm": assessment.heart_rate_bpm,
        "sdnn_ms": assessment.hrv_sdnn_ms,
        "rmssd_ms": assessment.hrv_rmssd_ms,
        "spo2_percent": assessment.spo2_percent,
        "stress_index": assessment.stress_index,
        "overall_score": assessment.overall_score,
    })
}

pub fn dispatch_wfdb_decode(params: &Value) -> Value {
    let Some(header_text) = params.get("header").and_then(Value::as_str) else {
        return missing("header");
    };
    match wfdb::parse_header(header_text) {
        Ok(hdr) => serde_json::json!({
            "record_name": hdr.record_name,
            "n_signals": hdr.n_signals,
            "sampling_frequency": hdr.sampling_frequency,
            "n_samples": hdr.n_samples,
            "signals": hdr.signals.iter().map(|s| serde_json::json!({
                "filename": s.filename,
                "format": s.format,
                "gain": s.gain,
                "baseline": s.baseline,
                "adc_resolution": s.adc_resolution,
                "description": s.description,
            })).collect::<Vec<_>>(),
        }),
        Err(e) => serde_json::json!({
            "error": "wfdb_parse_error",
            "message": e.to_string(),
        }),
    }
}
