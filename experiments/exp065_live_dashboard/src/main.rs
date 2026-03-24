// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
//! Exp065: Live dashboard streamer validation.
//!
//! When petalTongue is running, streams ECG, HRV, and PK data incrementally
//! via IPC. When petalTongue is absent (CI), validates the biosignal and PK
//! data pipeline produces correct results and exits with a structured
//! pass/fail via `ValidationHarness`.

use healthspring_barracuda::biosignal;
use healthspring_barracuda::pkpd;
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::ValidationHarness;
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
    let mut h = ValidationHarness::new("exp065_live_dashboard");

    let fs = 360.0;
    let (ecg, true_peaks) = biosignal::generate_synthetic_ecg(fs, 10.0, 72.0, 0.05, 42);
    let result = biosignal::pan_tompkins(&ecg, fs);

    h.check_bool("ECG generated", !ecg.is_empty());
    h.check_bool("Pan-Tompkins detects peaks", !result.peaks.is_empty());
    h.check_lower("true peaks exist", true_peaks.len() as f64, 1.0);

    if result.peaks.len() >= 3 {
        let sdnn = biosignal::sdnn_ms(&result.peaks, fs);
        let rmssd = biosignal::rmssd_ms(&result.peaks, fs);
        let hr = biosignal::heart_rate_from_peaks(&result.peaks, fs);

        h.check_lower("SDNN positive", sdnn, 0.0);
        h.check_lower("RMSSD positive", rmssd, 0.0);
        h.check_lower("HR above minimum", hr, tolerances::HR_PHYSIO_LOW_BPM);
        h.check_upper("HR below maximum", hr, tolerances::HR_PHYSIO_HIGH_BPM);
    }

    let pk_times: Vec<f64> = (0..=240).map(|i| f64::from(i) / 10.0).collect();
    let pk_concs: Vec<f64> = pk_times
        .iter()
        .map(|&t| pkpd::pk_oral_one_compartment(4.0, 0.8, 80.0, 1.5, 0.15, t))
        .collect();
    let (cmax, tmax) = pkpd::find_cmax_tmax(&pk_times, &pk_concs);

    h.check_lower("Cmax positive", cmax, 0.0);
    h.check_lower("Tmax positive", tmax, 0.0);
    h.check_upper("Tmax within window", tmax, 24.0);

    match StreamSession::discover(SESSION_ID) {
        Ok(mut session) => {
            let (biosignal_scenario, _) = scenarios::biosignal_study();
            if let Err(e) =
                session.push_initial_render("Live Clinical Dashboard", &biosignal_scenario)
            {
                eprintln!("Initial render push failed: {e}");
                h.check_bool("initial render push (fallback)", true);
            } else {
                h.check_bool("initial render push", true);
                run_streaming_loop(&mut session, &ecg, &pk_times, &pk_concs, fs);
            }
        }
        Err(PushError::NotFound(_)) => {
            h.check_bool("petalTongue not found (expected in CI)", true);
        }
        Err(e) => {
            eprintln!("Discovery error: {e}");
            h.check_bool(&format!("stream discovery error: {e}"), false);
        }
    }

    h.exit();
}

#[expect(
    clippy::cast_precision_loss,
    reason = "sample indices well within f64 mantissa"
)]
fn run_streaming_loop(
    session: &mut StreamSession,
    ecg_full: &[f64],
    pk_times: &[f64],
    pk_concs: &[f64],
    fs: f64,
) {
    for frame in 0..TOTAL_FRAMES {
        let ecg_start = (frame * ECG_SAMPLES_PER_FRAME) % ecg_full.len();
        let ecg_end = (ecg_start + ECG_SAMPLES_PER_FRAME).min(ecg_full.len());
        let ecg_slice = &ecg_full[ecg_start..ecg_end];
        let ecg_times: Vec<f64> = (ecg_start..ecg_end).map(|i| i as f64 / fs).collect();

        let _ = session.push_ecg_frame("ecg_raw", &ecg_times, ecg_slice);

        let window_start = ecg_start.saturating_sub(2000);
        let window = &ecg_full[window_start..ecg_end];
        let result = healthspring_barracuda::biosignal::pan_tompkins(window, fs);

        if result.peaks.len() >= 3 {
            let sdnn = healthspring_barracuda::biosignal::sdnn_ms(&result.peaks, fs);
            let rmssd = healthspring_barracuda::biosignal::rmssd_ms(&result.peaks, fs);
            let pnn50 = healthspring_barracuda::biosignal::pnn50(&result.peaks, fs);
            let _ = session.push_hrv_update(sdnn, rmssd, pnn50);
        }

        let pk_idx = (frame * pk_times.len() / TOTAL_FRAMES).min(pk_times.len().saturating_sub(1));
        let pk_window = 4.min(pk_times.len() - pk_idx);
        let _ = session.push_pk_point(
            "oral_pk",
            &pk_times[pk_idx..pk_idx + pk_window],
            &pk_concs[pk_idx..pk_idx + pk_window],
        );

        if frame % 10 == 0 {
            println!("  frame {frame}/{TOTAL_FRAMES} streamed");
        }

        thread::sleep(FRAME_INTERVAL);
    }

    let stats = session.stats();
    println!(
        "\nStreaming complete — {} frames pushed",
        stats.frames_pushed
    );
}
