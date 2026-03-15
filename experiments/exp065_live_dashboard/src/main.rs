// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
//! Live dashboard streamer: pushes ECG, HRV, and PK data incrementally
//! to petalTongue via IPC, simulating a real-time clinical monitoring session.

use healthspring_barracuda::biosignal;
use healthspring_barracuda::pkpd;
use healthspring_barracuda::visualization::ipc_push::PushError;
use healthspring_barracuda::visualization::scenarios;
use healthspring_barracuda::visualization::stream::StreamSession;

use std::thread;
use std::time::Duration;

const SESSION_ID: &str = "healthspring-live";
const FRAME_INTERVAL: Duration = Duration::from_millis(250);
const ECG_SAMPLES_PER_FRAME: usize = 90;
const TOTAL_FRAMES: usize = 120;

#[expect(
    clippy::cast_precision_loss,
    reason = "sample indices well within f64 mantissa"
)]
fn main() {
    println!("=== healthSpring Live Dashboard Streamer ===\n");

    let mut session = match StreamSession::discover(SESSION_ID) {
        Ok(s) => {
            println!("petalTongue found — streaming via IPC\n");
            s
        }
        Err(PushError::NotFound(_)) => {
            println!("petalTongue not running — falling back to stdout demo\n");
            run_stdout_demo();
            return;
        }
        Err(e) => {
            eprintln!("Discovery error: {e}");
            std::process::exit(1);
        }
    };

    let (biosignal_scenario, _) = scenarios::biosignal_study();
    if let Err(e) = session.push_initial_render("Live Clinical Dashboard", &biosignal_scenario) {
        eprintln!("Initial render push failed: {e}");
        eprintln!("Falling back to stdout demo\n");
        run_stdout_demo();
        return;
    }
    println!("Pushed initial biosignal scenario\n");

    let fs = 360.0;
    let (ecg_full, _) = biosignal::generate_synthetic_ecg(fs, 30.0, 72.0, 0.05, 42);

    let pk_times: Vec<f64> = (0..=480).map(|i| f64::from(i) / 10.0).collect();
    let pk_concs: Vec<f64> = pk_times
        .iter()
        .map(|&t| pkpd::pk_oral_one_compartment(4.0, 0.8, 80.0, 1.5, 0.15, t))
        .collect();

    println!(
        "Streaming {TOTAL_FRAMES} frames at {}ms intervals …",
        FRAME_INTERVAL.as_millis()
    );
    println!("  ECG: {ECG_SAMPLES_PER_FRAME} samples/frame");
    println!("  HRV: rolling SDNN/RMSSD/pNN50 gauges");
    println!("  PK:  oral concentration curve\n");

    for frame in 0..TOTAL_FRAMES {
        let ecg_start = (frame * ECG_SAMPLES_PER_FRAME) % ecg_full.len();
        let ecg_end = (ecg_start + ECG_SAMPLES_PER_FRAME).min(ecg_full.len());
        let ecg_slice = &ecg_full[ecg_start..ecg_end];
        let ecg_times: Vec<f64> = (ecg_start..ecg_end).map(|i| i as f64 / fs).collect();

        if let Err(e) = session.push_ecg_frame("ecg_raw", &ecg_times, ecg_slice) {
            eprintln!("  frame {frame}: ECG append failed: {e}");
        }

        let window_start = ecg_start.saturating_sub(2000);
        let window_end = ecg_end;
        let window = &ecg_full[window_start..window_end];
        let result = biosignal::pan_tompkins(window, fs);

        if result.peaks.len() >= 3 {
            let sdnn = biosignal::sdnn_ms(&result.peaks, fs);
            let rmssd = biosignal::rmssd_ms(&result.peaks, fs);
            let pnn50 = biosignal::pnn50(&result.peaks, fs);

            let _ = session.push_hrv_update(sdnn, rmssd, pnn50);
        }

        let pk_idx = (frame * pk_times.len() / TOTAL_FRAMES).min(pk_times.len().saturating_sub(1));
        let pk_window = 4.min(pk_times.len() - pk_idx);
        let pk_t = &pk_times[pk_idx..pk_idx + pk_window];
        let pk_c = &pk_concs[pk_idx..pk_idx + pk_window];
        let _ = session.push_pk_point("oral_pk", pk_t, pk_c);

        if frame % 10 == 0 {
            println!("  frame {frame}/{TOTAL_FRAMES} streamed");
        }

        thread::sleep(FRAME_INTERVAL);
    }

    let stats = session.stats();
    println!(
        "\nStreaming complete — {} frames pushed to petalTongue",
        stats.frames_pushed
    );
    if let Some(avg) = stats.avg_push_latency() {
        println!("  Avg push latency: {avg:?}");
    }
    if stats.errors > 0 {
        println!("  Push errors: {}", stats.errors);
    }
    if stats.cooldowns > 0 {
        println!("  Backpressure cooldowns: {}", stats.cooldowns);
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "0.1 * fs is a small positive value; truncation is safe"
)]
fn run_stdout_demo() {
    println!("Running stdout demo (no petalTongue connection)\n");

    let fs = 360.0;
    let (ecg, true_peaks) = biosignal::generate_synthetic_ecg(fs, 10.0, 72.0, 0.05, 42);
    let result = biosignal::pan_tompkins(&ecg, fs);
    let metrics = biosignal::evaluate_detection(&result.peaks, &true_peaks, (0.1 * fs) as usize);

    println!("ECG Analysis (10s @ 360 Hz):");
    println!("  Samples:     {}", ecg.len());
    println!("  Peaks found: {}", result.peaks.len());
    println!("  Sensitivity: {:.1}%", metrics.sensitivity * 100.0);
    println!("  PPV:         {:.1}%", metrics.ppv * 100.0);

    let hr = biosignal::heart_rate_from_peaks(&result.peaks, fs);
    let sdnn = biosignal::sdnn_ms(&result.peaks, fs);
    let rmssd = biosignal::rmssd_ms(&result.peaks, fs);
    let pnn50 = biosignal::pnn50(&result.peaks, fs);

    println!("\nHRV Metrics:");
    println!("  Heart Rate:  {hr:.1} bpm");
    println!("  SDNN:        {sdnn:.1} ms");
    println!("  RMSSD:       {rmssd:.1} ms");
    println!("  pNN50:       {pnn50:.1}%");

    let pk_times: Vec<f64> = (0..=240).map(|i| f64::from(i) / 10.0).collect();
    let pk_concs: Vec<f64> = pk_times
        .iter()
        .map(|&t| pkpd::pk_oral_one_compartment(4.0, 0.8, 80.0, 1.5, 0.15, t))
        .collect();
    let (cmax, tmax) = pkpd::find_cmax_tmax(&pk_times, &pk_concs);

    println!("\nPK Monitoring (oral, 24h):");
    println!("  Cmax:  {cmax:.4} mg/L");
    println!("  Tmax:  {tmax:.1} hr");

    println!("\nStdout demo complete — start petalTongue to see live visualization");
}
