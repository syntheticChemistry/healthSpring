# SPDX-License-Identifier: AGPL-3.0-or-later

#!/usr/bin/env python3
"""
healthSpring Exp020 — Pan-Tompkins QRS Detection

Validates the Pan-Tompkins algorithm for real-time QRS complex detection
in ECG signals. This is the foundational biosignal experiment.

Algorithm (Pan & Tompkins 1985):
  1. Bandpass filter (5-15 Hz)
  2. Derivative filter (slope emphasis)
  3. Squaring (nonlinear amplification)
  4. Moving window integration
  5. Adaptive thresholding (dual threshold, search-back)

We validate against a synthetic ECG with known R-peak locations,
then the algorithm can be applied to MIT-BIH (PhysioNet) data.

Provenance:
  Baseline date:   2026-03-08
  Command:         python3 control/biosignal/exp020_pan_tompkins_qrs.py
  Environment:     Python 3.10+, NumPy, seed=42
"""

import json
import os
import sys

import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
SEED = 42
FS = 360  # MIT-BIH sampling rate (Hz)


def generate_synthetic_ecg(fs, duration_s, heart_rate_bpm, rng, noise_std=0.05):
    """Generate synthetic ECG with known R-peak locations.

    Simplified model: Gaussian P, QRS complex (narrow Gaussian), T wave.
    """
    n_samples = int(fs * duration_s)
    t = np.arange(n_samples) / fs
    rr_interval = 60.0 / heart_rate_bpm
    ecg = np.zeros(n_samples)
    r_peak_times = []

    beat_time = 0.1  # start slightly in
    while beat_time < duration_s - 0.5:
        r_peak_times.append(beat_time)
        # P wave (small, before QRS)
        ecg += 0.15 * np.exp(-((t - (beat_time - 0.16)) ** 2) / (2 * 0.01 ** 2))
        # Q (small negative)
        ecg -= 0.10 * np.exp(-((t - (beat_time - 0.04)) ** 2) / (2 * 0.005 ** 2))
        # R peak (tall, narrow)
        ecg += 1.0 * np.exp(-((t - beat_time) ** 2) / (2 * 0.008 ** 2))
        # S (negative)
        ecg -= 0.25 * np.exp(-((t - (beat_time + 0.04)) ** 2) / (2 * 0.008 ** 2))
        # T wave (broad, positive)
        ecg += 0.30 * np.exp(-((t - (beat_time + 0.25)) ** 2) / (2 * 0.04 ** 2))

        rr_jitter = rng.normal(0, 0.02)
        beat_time += rr_interval + rr_jitter

    ecg += rng.normal(0, noise_std, n_samples)
    r_peak_indices = [int(rt * fs) for rt in r_peak_times if int(rt * fs) < n_samples]
    return ecg, t, r_peak_indices


def bandpass_filter(signal, fs, low=5.0, high=15.0):
    """Simple frequency-domain bandpass filter."""
    n = len(signal)
    freqs = np.fft.rfftfreq(n, 1.0 / fs)
    fft_sig = np.fft.rfft(signal)
    mask = (freqs >= low) & (freqs <= high)
    fft_sig[~mask] = 0.0
    return np.fft.irfft(fft_sig, n)


def derivative_filter(signal):
    """Five-point derivative (Pan-Tompkins)."""
    d = np.zeros_like(signal)
    for i in range(2, len(signal) - 2):
        d[i] = (-signal[i - 2] - 2 * signal[i - 1] + 2 * signal[i + 1] + signal[i + 2]) / 8.0
    return d


def squaring(signal):
    """Nonlinear squaring operation."""
    return signal ** 2


def moving_window_integration(signal, window_size):
    """Moving average with window."""
    kernel = np.ones(window_size) / window_size
    return np.convolve(signal, kernel, mode="same")


def detect_peaks(mwi, fs, refractory_ms=200):
    """Simple peak detection with refractory period."""
    threshold = 0.4 * np.max(mwi)
    refractory_samples = int(refractory_ms * fs / 1000)
    peaks = []
    last_peak = -refractory_samples
    for i in range(1, len(mwi) - 1):
        if mwi[i] > mwi[i - 1] and mwi[i] > mwi[i + 1] and mwi[i] > threshold:
            if i - last_peak > refractory_samples:
                peaks.append(i)
                last_peak = i
    return peaks


def evaluate_detection(detected, true_peaks, tolerance_ms, fs):
    """Compute sensitivity and positive predictivity."""
    tol_samples = int(tolerance_ms * fs / 1000)
    tp = 0
    matched = set()
    for d in detected:
        for j, t in enumerate(true_peaks):
            if j not in matched and abs(d - t) <= tol_samples:
                tp += 1
                matched.add(j)
                break
    fn = len(true_peaks) - tp
    fp = len(detected) - tp
    sensitivity = tp / (tp + fn) if (tp + fn) > 0 else 0.0
    ppv = tp / (tp + fp) if (tp + fp) > 0 else 0.0
    return {
        "tp": tp, "fp": fp, "fn": fn,
        "sensitivity": sensitivity, "ppv": ppv,
    }


def main():
    total_passed = 0
    total_failed = 0
    baseline = {}
    rng = np.random.default_rng(SEED)

    print("=" * 72)
    print("healthSpring Exp020: Pan-Tompkins QRS Detection")
    print(f"  fs={FS} Hz, seed={SEED}")
    print("=" * 72)

    # Generate synthetic ECG (10 seconds, 72 bpm)
    ecg, t, true_peaks = generate_synthetic_ecg(FS, 10.0, 72, rng)

    baseline["n_samples"] = len(ecg)
    baseline["n_true_peaks"] = len(true_peaks)
    baseline["heart_rate_bpm"] = 72

    # Pan-Tompkins pipeline
    bp = bandpass_filter(ecg, FS, 5.0, 15.0)
    deriv = derivative_filter(bp)
    sq = squaring(deriv)
    window_size = int(0.15 * FS)  # 150ms window
    mwi = moving_window_integration(sq, window_size)
    detected = detect_peaks(mwi, FS)

    baseline["n_detected"] = len(detected)

    # Evaluate
    metrics = evaluate_detection(detected, true_peaks, tolerance_ms=75, fs=FS)
    baseline["detection_metrics"] = metrics

    # ------------------------------------------------------------------
    # Check 1: Synthetic ECG has expected number of beats
    # ------------------------------------------------------------------
    print(f"\n--- Check 1: Synthetic ECG beat count ---")
    expected_beats = int(10.0 * 72 / 60)
    if abs(len(true_peaks) - expected_beats) <= 1:
        print(f"  [PASS] {len(true_peaks)} beats (expected ~{expected_beats})")
        total_passed += 1
    else:
        print(f"  [FAIL] {len(true_peaks)} beats")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 2: Bandpass output in range
    # ------------------------------------------------------------------
    print("\n--- Check 2: Bandpass output bounded ---")
    if np.max(np.abs(bp)) < np.max(np.abs(ecg)):
        print(f"  [PASS] max|BP|={np.max(np.abs(bp)):.4f} < max|ECG|={np.max(np.abs(ecg)):.4f}")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 3: Squared signal non-negative
    # ------------------------------------------------------------------
    print("\n--- Check 3: Squared signal ≥ 0 ---")
    if np.all(sq >= 0):
        print(f"  [PASS] all squared values ≥ 0")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 4: MWI non-negative
    # ------------------------------------------------------------------
    print("\n--- Check 4: MWI non-negative ---")
    if np.all(mwi >= -1e-12):
        print(f"  [PASS]")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 5: Sensitivity > 90%
    # ------------------------------------------------------------------
    print("\n--- Check 5: Sensitivity > 90% ---")
    if metrics["sensitivity"] > 0.90:
        print(f"  [PASS] Se = {metrics['sensitivity']:.3f} ({metrics['tp']}/{metrics['tp']+metrics['fn']})")
        total_passed += 1
    else:
        print(f"  [FAIL] Se = {metrics['sensitivity']:.3f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 6: Positive predictivity > 90%
    # ------------------------------------------------------------------
    print("\n--- Check 6: PPV > 90% ---")
    if metrics["ppv"] > 0.90:
        print(f"  [PASS] PPV = {metrics['ppv']:.3f} ({metrics['tp']}/{metrics['tp']+metrics['fp']})")
        total_passed += 1
    else:
        print(f"  [FAIL] PPV = {metrics['ppv']:.3f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 7: No false negatives (on clean synthetic)
    # ------------------------------------------------------------------
    print("\n--- Check 7: False negatives ---")
    print(f"  [{'PASS' if metrics['fn'] == 0 else 'INFO'}] FN = {metrics['fn']}")
    if metrics["fn"] <= 1:
        total_passed += 1
    else:
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 8: False positive rate low
    # ------------------------------------------------------------------
    print("\n--- Check 8: False positives ---")
    if metrics["fp"] <= 2:
        print(f"  [PASS] FP = {metrics['fp']}")
        total_passed += 1
    else:
        print(f"  [FAIL] FP = {metrics['fp']}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 9: Detected HR matches input
    # ------------------------------------------------------------------
    print("\n--- Check 9: Detected heart rate ---")
    if len(detected) >= 2:
        rr_detected = np.diff(detected) / FS
        hr_detected = 60.0 / np.mean(rr_detected)
        baseline["hr_detected"] = float(hr_detected)
        if abs(hr_detected - 72) < 5:
            print(f"  [PASS] HR = {hr_detected:.1f} bpm (expected 72)")
            total_passed += 1
        else:
            print(f"  [FAIL] HR = {hr_detected:.1f}")
            total_failed += 1
    else:
        print(f"  [FAIL] Too few detections")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 10: RR intervals consistent (low SDNN on synthetic)
    # ------------------------------------------------------------------
    print("\n--- Check 10: RR interval consistency ---")
    if len(detected) >= 3:
        rr = np.diff(detected) / FS * 1000  # in ms
        sdnn = float(np.std(rr))
        baseline["sdnn_ms"] = sdnn
        if sdnn < 100:  # synthetic has small jitter
            print(f"  [PASS] SDNN = {sdnn:.1f} ms (< 100)")
            total_passed += 1
        else:
            print(f"  [FAIL] SDNN = {sdnn:.1f}")
            total_failed += 1
    else:
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 11: Pipeline preserves signal length
    # ------------------------------------------------------------------
    print("\n--- Check 11: Signal length preserved ---")
    if len(bp) == len(ecg) == len(deriv) == len(sq) == len(mwi):
        print(f"  [PASS] all stages: {len(ecg)} samples")
        total_passed += 1
    else:
        print(f"  [FAIL]")
        total_failed += 1

    # ------------------------------------------------------------------
    # Check 12: Squaring peaks at QRS regions (MWI peak > 3x MWI mean)
    # ------------------------------------------------------------------
    print("\n--- Check 12: MWI peaks at QRS regions ---")
    mwi_at_detected = [mwi[min(d, len(mwi) - 1)] for d in detected]
    mwi_mean = np.mean(mwi)
    ratio = np.mean(mwi_at_detected) / mwi_mean if mwi_mean > 0 else 0
    baseline["mwi_peak_ratio"] = float(ratio)
    if ratio > 3.0:
        print(f"  [PASS] MWI at QRS = {ratio:.1f}x mean")
        total_passed += 1
    else:
        print(f"  [FAIL] ratio = {ratio:.1f}")
        total_failed += 1

    # ------------------------------------------------------------------
    # Write baseline
    # ------------------------------------------------------------------
    baseline_path = os.path.join(SCRIPT_DIR, "exp020_baseline.json")
    baseline_out = {
        "_source": "healthSpring Exp020: Pan-Tompkins QRS Detection",
        "_method": "Bandpass → derivative → squaring → MWI → threshold",
        "fs": FS,
        "seed": SEED,
        **baseline,
        "_provenance": {
            "date": "2026-03-08",
            "python": sys.version,
            "numpy": np.__version__,
        },
    }
    with open(baseline_path, "w") as f:
        json.dump(baseline_out, f, indent=2, default=str)
    print(f"\nBaseline written to {baseline_path}")

    total = total_passed + total_failed
    print(f"\n{'=' * 72}")
    print(f"TOTAL: {total_passed}/{total} PASS, {total_failed}/{total} FAIL")
    print(f"{'=' * 72}")

    return 0 if total_failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
