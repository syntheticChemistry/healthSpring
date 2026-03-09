<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# Sub-Thesis: Sovereign Edge Biosignal Processing

**Faculty**: TBD
**Track**: 3 — Biosignal
**Experiments**: Exp020 (Pan-Tompkins QRS), Exp021 (HRV metrics), Exp022 (PPG SpO2), Exp023 (multi-channel fusion)
**Status**: Complete — 4 experiments, 44 binary checks, 44 Python cross-validation checks
**Last Updated**: March 9, 2026

---

## Thesis

Real-time biosignal processing — ECG QRS detection, heart rate variability, pulse oximetry,
and multi-channel fusion — can run on sovereign hardware via Pure Rust at clinically useful
latency, enabling edge deployment on wearables and field devices without cloud dependencies.

---

## Experiments

| Exp | Title | Key Validation |
|-----|-------|----------------|
| 020 | Pan-Tompkins QRS Detection | Bandpass → derivative → squaring → MWI → peak detect. Sensitivity > 99%, PPV > 95% at 360 Hz |
| 021 | HRV Metrics | SDNN, RMSSD, pNN50 from RR intervals. Cross-validated against Python |
| 022 | PPG SpO2 | R-value calibration curve, synthetic PPG generation, SpO2 recovery within ±2% |
| 023 | Biosignal Fusion | ECG + PPG + EDA → FusedHealthAssessment (HR, SDNN, RMSSD, SpO2, stress, overall score) |

---

## Key Results

- Pan-Tompkins at 360 Hz: ~0.5ms per heartbeat detection (real-time capable)
- HRV metrics match Python reference within 1e-10 tolerance
- SpO2 calibration: linear R-value → SpO2 mapping validated against Beer-Lambert model
- Fusion composite score combines 5 channels into 0–100 health assessment

---

## Modules

- `barracuda/src/biosignal.rs`: `pan_tompkins`, `heart_rate_from_peaks`, `sdnn_ms`, `rmssd_ms`, `pnn50`, `ppg_r_value`, `spo2_from_r`, `eda_scl`, `eda_phasic`, `eda_detect_scr`, `fuse_channels`, synthetic generators
- `control/biosignal/`: Python baselines for all 4 experiments

---

## petalTongue Visualization (V7)

`scenarios::biosignal_study()` produces a 4-node scenario with:
- TimeSeries: ECG raw + bandpass, RR tachogram, R-value calibration, EDA SCL + phasic
- Gauges: HR, sensitivity, PPV, SDNN, RMSSD, pNN50, SpO2, stress index, overall score
- Clinical ranges: Normal HR (60–100 bpm), normal SpO2 (95–100%), hypoxemia (< 90%)

---

## Edge Deployment Path

- **CPU**: All algorithms run at real-time latency on ARM Cortex-A (Raspberry Pi 4+)
- **NPU**: Pan-Tompkins streaming detection targeted for Akida AKD1000 (hotSpring lineage)
- **GPU**: Not primary target — biosignal data volumes typically small enough for CPU
