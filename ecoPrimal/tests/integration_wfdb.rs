// SPDX-License-Identifier: AGPL-3.0-or-later
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests use unwrap/expect for concise assertions"
)]
//! WFDB format round-trip integration tests.
//!
//! Verifies that encode → decode → re-encode produces identical binary
//! output for all supported WFDB formats. Exercises the full pipeline:
//! header parse, signal decode, ADC-to-physical, annotation parse.

use healthspring_barracuda::tolerances;
use healthspring_barracuda::wfdb;

// ── Format 212 round-trip ────────────────────────────────────────────

/// Encode a pair of 12-bit samples into WFDB Format 212 (3 bytes).
///
/// This matches the PhysioNet specification: each sample pair occupies 3 bytes
/// with 12 bits per sample, sign-extended via two's complement offset by 4096.
fn encode_format_212_pair(s1: i16, s2: i16) -> [u8; 3] {
    let u1 = u16::from_ne_bytes((if s1 < 0 { s1 + 4096 } else { s1 }).to_ne_bytes());
    let u2 = u16::from_ne_bytes((if s2 < 0 { s2 + 4096 } else { s2 }).to_ne_bytes());
    [
        (u1 & 0xFF) as u8,
        ((u1 >> 8) | ((u2 >> 8) << 4)) as u8,
        (u2 & 0xFF) as u8,
    ]
}

#[test]
fn format_212_encode_decode_roundtrip_identity() {
    let n_samples = 100;
    let n_channels = 2;

    let ch1: Vec<i16> = (0..n_samples)
        .map(|i| {
            let t = i as f64 / 360.0;
            (200.0 * (2.0 * std::f64::consts::PI * 1.2 * t).sin()) as i16
        })
        .collect();
    let ch2: Vec<i16> = ch1.iter().map(|&s| s / 2).collect();

    let mut encoded = Vec::with_capacity(n_samples * 3);
    for i in 0..n_samples {
        encoded.extend_from_slice(&encode_format_212_pair(ch1[i], ch2[i]));
    }

    let decoded = wfdb::decode_format_212(&encoded, n_samples, n_channels).expect("decode");
    assert_eq!(decoded.len(), n_channels);
    assert_eq!(decoded[0].len(), n_samples);
    assert_eq!(decoded[1].len(), n_samples);
    assert_eq!(decoded[0], ch1, "channel 0 round-trip identity");
    assert_eq!(decoded[1], ch2, "channel 1 round-trip identity");

    let mut re_encoded = Vec::with_capacity(n_samples * 3);
    for i in 0..n_samples {
        re_encoded.extend_from_slice(&encode_format_212_pair(decoded[0][i], decoded[1][i]));
    }
    assert_eq!(encoded, re_encoded, "binary round-trip identity");
}

// ── Format 16 round-trip ─────────────────────────────────────────────

#[test]
fn format_16_encode_decode_roundtrip_identity() {
    let n_samples = 300;
    let n_channels = 2;
    let ch1: Vec<i16> = (0..n_samples).map(|i| (i as i16).wrapping_mul(7)).collect();
    let ch2: Vec<i16> = (0..n_samples)
        .map(|i| (i as i16).wrapping_mul(13).wrapping_add(100))
        .collect();

    let mut encoded = Vec::with_capacity(n_samples * n_channels * 2);
    for i in 0..n_samples {
        encoded.extend_from_slice(&ch1[i].to_le_bytes());
        encoded.extend_from_slice(&ch2[i].to_le_bytes());
    }

    let decoded = wfdb::decode_format_16(&encoded, n_samples, n_channels).expect("decode");
    assert_eq!(decoded[0], ch1, "format 16 channel 0 identity");
    assert_eq!(decoded[1], ch2, "format 16 channel 1 identity");

    let mut re_encoded = Vec::with_capacity(n_samples * n_channels * 2);
    for i in 0..n_samples {
        re_encoded.extend_from_slice(&decoded[0][i].to_le_bytes());
        re_encoded.extend_from_slice(&decoded[1][i].to_le_bytes());
    }
    assert_eq!(encoded, re_encoded, "format 16 binary round-trip identity");
}

// ── ADC → physical → back ────────────────────────────────────────────

#[test]
fn adc_physical_roundtrip() {
    let gain = 200.0;
    let baseline = 1024;
    let adc_samples: Vec<i16> = vec![824, 924, 1024, 1124, 1224];
    let physical = wfdb::adc_to_physical(&adc_samples, gain, baseline);

    let recovered: Vec<i16> = physical
        .iter()
        .map(|&v| {
            let adc_f = v.mul_add(gain, f64::from(baseline));
            adc_f.round() as i16
        })
        .collect();
    assert_eq!(recovered, adc_samples, "ADC → physical → ADC identity");
}

// ── Header parse round-trip ──────────────────────────────────────────

#[test]
fn header_parse_fields_roundtrip() {
    let hea = "\
100 2 360 650000
100.dat 212 200 1024 11 1024 -10 0 MLII
100.dat 212 200 1024 11 1024 0 0 V5
";
    let h = wfdb::parse_header(hea).expect("parse");
    assert_eq!(h.record_name, "100");
    assert_eq!(h.n_signals, 2);
    assert!((h.sampling_frequency - 360.0).abs() < tolerances::TEST_ASSERTION_TIGHT);
    assert_eq!(h.n_samples, Some(650_000));
    assert_eq!(h.signals[0].format, 212);
    assert_eq!(h.signals[1].description, "V5");

    let h2 = wfdb::parse_header(hea).expect("re-parse");
    assert_eq!(h.record_name, h2.record_name);
    assert_eq!(h.n_signals, h2.n_signals);
    assert_eq!(
        h.sampling_frequency.to_bits(),
        h2.sampling_frequency.to_bits(),
        "header parse is deterministic"
    );
}

// ── Annotation parse ─────────────────────────────────────────────────

#[test]
fn annotation_parse_and_types() {
    let mut data = Vec::new();
    for &(delta, ann_type) in &[(100_u16, 1_u16), (50, 5), (75, 6)] {
        let lo = (delta & 0xFF) as u8;
        let hi = ((ann_type << 2) | (delta >> 8)) as u8;
        data.push(lo);
        data.push(hi);
    }
    data.extend_from_slice(&[0, 0]);

    let anns = wfdb::parse_annotations(&data).expect("parse");
    assert_eq!(anns.len(), 3);
    assert_eq!(anns[0].sample, 100);
    assert_eq!(anns[0].beat_type, wfdb::BeatType::Normal);
    assert_eq!(anns[1].sample, 150);
    assert_eq!(anns[1].beat_type, wfdb::BeatType::AtrialPremature);
    assert_eq!(anns[2].sample, 225);
    assert_eq!(anns[2].beat_type, wfdb::BeatType::VentricularPremature);

    let anns2 = wfdb::parse_annotations(&data).expect("re-parse");
    assert_eq!(anns.len(), anns2.len());
    for (a, b) in anns.iter().zip(anns2.iter()) {
        assert_eq!(a.sample, b.sample, "annotation parse is deterministic");
        assert_eq!(a.beat_type, b.beat_type);
    }
}
