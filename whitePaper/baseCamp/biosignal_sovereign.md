<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
# Sub-Thesis: Sovereign Edge Biosignal Processing

**Faculty**: TBD
**Track**: 3 — Biosignal
**Experiments**: Exp020 (Pan-Tompkins QRS), Exp021 (HRV metrics), Exp022 (PPG SpO2), Exp023 (multi-channel fusion)
**Status**: Complete — 4 experiments, 44 binary checks, 44 Python cross-validation checks. Pan-Tompkins 5-stage intermediates (raw, bandpass, derivative, squared, MWI) now visualized as individual TimeSeries channels in petalTongue (8 channels on QRS node). V13: LCG PRNG centralized to `rng.rs`. V14.1: biosignal.rs refactored to 6 domain-coherent submodules (ecg, hrv, ppg, eda, fusion, fft).
**Last Updated**: March 10, 2026

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

- `barracuda/src/biosignal/`: Domain-coherent submodules — `ecg.rs` (Pan-Tompkins), `hrv.rs` (SDNN, RMSSD, pNN50), `ppg.rs` (SpO2), `eda.rs` (SCL, phasic, SCR), `fusion.rs` (FusedHealthAssessment), `fft.rs` (DFT utilities). `mod.rs` re-exports all public items for API compatibility.
- `control/biosignal/`: Python baselines for all 4 experiments

---

## petalTongue Visualization (V12)

`scenarios::biosignal_study()` produces a 4-node scenario with:
- TimeSeries: ECG raw + bandpass, RR tachogram, R-value calibration, EDA SCL + phasic
- **V12**: Pan-Tompkins intermediates: derivative, squared, and MWI signals exposed as 3 additional TimeSeries channels on the QRS node (8 total channels) — making every processing stage visible
- Gauges: HR, sensitivity, PPV, SDNN, RMSSD, pNN50, SpO2, stress index, overall score
- Clinical ranges: Normal HR (60–100 bpm), normal SpO2 (95–100%), hypoxemia (< 90%)

---

## Edge Deployment Path

- **CPU**: All algorithms run at real-time latency on ARM Cortex-A (Raspberry Pi 4+)
- **NPU**: Pan-Tompkins streaming detection targeted for Akida AKD1000 (hotSpring lineage)
- **GPU**: Not primary target — biosignal data volumes typically small enough for CPU
