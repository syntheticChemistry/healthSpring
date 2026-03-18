// SPDX-License-Identifier: AGPL-3.0-or-later
//! Streaming WFDB format parser for `PhysioNet` MIT-BIH database loading.
//!
//! Implements the WFDB (Waveform Database) format used by `PhysioNet` for
//! distributing ECG and other biosignal recordings. Supports:
//! - `.hea` header file parsing (record metadata, signal specifications)
//! - `.dat` binary signal data reading (Format 212, 16, 80, 310, 311)
//! - `.atr` annotation file parsing (beat labels, rhythm annotations)
//!
//! Zero-copy where possible: binary data is read as byte slices and decoded
//! on-the-fly without buffering entire files.
//!
//! ## `PhysioNet` MIT-BIH Arrhythmia Database
//!
//! 48 half-hour two-channel ECG recordings from 47 subjects (records 100-234).
//! Digitized at 360 Hz, 11-bit resolution, ±5 mV range.
//!
//! ## References
//!
//! - Goldberger et al., "`PhysioBank`, `PhysioToolkit`, and `PhysioNet`" (2000)
//! - WFDB specification: <https://physionet.org/content/wfdb/>

pub mod annotations;
pub mod parser;

pub use annotations::*;
pub use parser::*;

/// Errors that can occur during WFDB parsing.
#[derive(Debug)]
pub enum WfdbError {
    /// Header format is invalid or unrecognised.
    InvalidHeader(String),
    /// Signal data format not supported.
    UnsupportedFormat(u16),
    /// Data file is truncated or corrupt.
    DataTruncated,
    /// Annotation file is malformed.
    InvalidAnnotation(String),
}

impl core::fmt::Display for WfdbError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidHeader(msg) => write!(f, "invalid WFDB header: {msg}"),
            Self::UnsupportedFormat(fmt) => write!(f, "unsupported signal format: {fmt}"),
            Self::DataTruncated => write!(f, "signal data truncated"),
            Self::InvalidAnnotation(msg) => write!(f, "invalid annotation: {msg}"),
        }
    }
}

impl std::error::Error for WfdbError {}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;
    use crate::tolerances;

    #[test]
    fn parse_header_mitbih_100() {
        let hea = "\
100 2 360 650000
100.dat 212 200 1024 11 1024 -10 0 MLII
100.dat 212 200 1024 11 1024 0 0 V5
# This is a comment
";
        let header = parse_header(hea).expect("parse header");
        assert_eq!(header.record_name, "100");
        assert_eq!(header.n_signals, 2);
        assert!((header.sampling_frequency - 360.0).abs() < tolerances::TEST_ASSERTION_TIGHT);
        assert_eq!(header.n_samples, Some(650_000));
        assert_eq!(header.signals.len(), 2);
        assert_eq!(header.signals[0].format, 212);
        assert!((header.signals[0].gain - 200.0).abs() < tolerances::TEST_ASSERTION_TIGHT);
        assert_eq!(header.signals[0].baseline, 1024);
        assert_eq!(header.signals[0].adc_resolution, 11);
        assert_eq!(header.signals[0].description, "MLII");
        assert_eq!(header.signals[1].description, "V5");
    }

    #[test]
    fn parse_header_minimal() {
        let hea = "rec01 1 250\nrec01.dat 16 1000\n";
        let header = parse_header(hea).expect("parse minimal header");
        assert_eq!(header.record_name, "rec01");
        assert_eq!(header.n_signals, 1);
        assert!((header.sampling_frequency - 250.0).abs() < tolerances::TEST_ASSERTION_TIGHT);
        assert_eq!(header.signals[0].format, 16);
        assert!((header.signals[0].gain - 1000.0).abs() < tolerances::TEST_ASSERTION_TIGHT);
    }

    #[test]
    fn decode_format_212_roundtrip() {
        let s1_values: Vec<i16> = vec![100, -200, 500, -1000, 2047];
        let s2_values: Vec<i16> = vec![-50, 300, -800, 1500, -2048];

        let mut data = Vec::new();
        for (&s1, &s2) in s1_values.iter().zip(s2_values.iter()) {
            let u1 = u16::from_ne_bytes((if s1 < 0 { s1 + 4096 } else { s1 }).to_ne_bytes());
            let u2 = u16::from_ne_bytes((if s2 < 0 { s2 + 4096 } else { s2 }).to_ne_bytes());
            data.push(u8::try_from(u1 & 0xFF).unwrap_or(0));
            data.push(u8::try_from((u1 >> 8) | ((u2 >> 8) << 4)).unwrap_or(0));
            data.push(u8::try_from(u2 & 0xFF).unwrap_or(0));
        }

        let channels = decode_format_212(&data, 5, 2).expect("decode");
        assert_eq!(channels[0], s1_values);
        assert_eq!(channels[1], s2_values);
    }

    #[test]
    fn decode_format_16_basic() {
        let samples: Vec<i16> = vec![100, 200, -300, 400, -500, 600];
        let mut data = Vec::new();
        for &s in &samples {
            data.extend_from_slice(&s.to_le_bytes());
        }

        let channels = decode_format_16(&data, 3, 2).expect("decode");
        assert_eq!(channels[0], vec![100, -300, -500]);
        assert_eq!(channels[1], vec![200, 400, 600]);
    }

    #[test]
    fn adc_to_physical_conversion() {
        let samples = vec![1024_i16, 1224, 824];
        let physical = adc_to_physical(&samples, 200.0, 1024);
        assert!(
            (physical[0]).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "baseline → 0 mV"
        );
        assert!(
            (physical[1] - 1.0).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "+200 ADC units → +1.0 mV at gain 200"
        );
        assert!(
            (physical[2] + 1.0).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "-200 ADC units → -1.0 mV"
        );
    }

    #[test]
    fn format_212_streaming_iterator() {
        let data: Vec<u8> = vec![0x00, 0x04, 0x00, 0xFF, 0x07, 0xFF];
        assert_eq!(Format212Iter::new(&data).count(), 2);
    }

    #[test]
    fn parse_annotations_basic() {
        let mut data = Vec::new();
        let delta: u16 = 100;
        let ann_type: u16 = 1;
        let lo = u8::try_from(delta & 0xFF).unwrap_or(0);
        let hi = u8::try_from((ann_type << 2) | (delta >> 8)).unwrap_or(0);
        data.push(lo);
        data.push(hi);
        data.push(lo);
        data.push(hi);

        // Terminator
        data.extend_from_slice(&[0, 0]);

        let anns = parse_annotations(&data).expect("parse annotations");
        assert_eq!(anns.len(), 2);
        assert_eq!(anns[0].sample, 100);
        assert_eq!(anns[0].beat_type, BeatType::Normal);
        assert_eq!(anns[1].sample, 200);
    }

    #[test]
    fn beat_type_codes() {
        assert_eq!(BeatType::from_code(1), BeatType::Normal);
        assert_eq!(BeatType::from_code(5), BeatType::AtrialPremature);
        assert_eq!(BeatType::from_code(6), BeatType::VentricularPremature);
        assert_eq!(BeatType::from_code(99), BeatType::Unknown(99));
    }

    #[test]
    fn adc_to_physical_zero_gain() {
        let samples = vec![100_i16, 200, 300];
        let physical = adc_to_physical(&samples, 0.0, 0);
        assert!(
            physical
                .iter()
                .all(|&v| v.abs() < tolerances::DIVISION_GUARD)
        );
    }

    #[test]
    fn header_parse_error_empty() {
        let result = parse_header("");
        assert!(result.is_err());
    }

    #[test]
    fn format_16_iter_basic() {
        let samples: Vec<i16> = vec![100, 200, -300, 400, -500, 600];
        let mut data = Vec::new();
        for &s in &samples {
            data.extend_from_slice(&s.to_le_bytes());
        }
        let collected: Vec<Vec<i16>> = Format16Iter::new(&data, 2).collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], vec![100, 200]);
        assert_eq!(collected[1], vec![-300, 400]);
        assert_eq!(collected[2], vec![-500, 600]);
    }

    #[test]
    fn format_16_iter_truncated() {
        let data = [0u8; 5];
        assert_eq!(
            Format16Iter::new(&data, 2).count(),
            1,
            "only one complete pair in 5 bytes"
        );
    }

    #[test]
    fn adc_to_physical_iter_roundtrip() {
        let adc_samples = vec![1024_i16, 1224, 824];
        let physical: Vec<f64> =
            AdcToPhysicalIter::new(adc_samples.into_iter(), 200.0, 1024).collect();
        assert!((physical[0]).abs() < tolerances::TEST_ASSERTION_TIGHT);
        assert!((physical[1] - 1.0).abs() < tolerances::TEST_ASSERTION_TIGHT);
        assert!((physical[2] + 1.0).abs() < tolerances::TEST_ASSERTION_TIGHT);
    }

    #[test]
    fn format_212_iter_matches_decode() {
        let s1_values: Vec<i16> = vec![100, -200, 500];
        let s2_values: Vec<i16> = vec![-50, 300, -800];

        let mut data = Vec::new();
        for (&s1, &s2) in s1_values.iter().zip(s2_values.iter()) {
            let u1 = u16::from_ne_bytes((if s1 < 0 { s1 + 4096 } else { s1 }).to_ne_bytes());
            let u2 = u16::from_ne_bytes((if s2 < 0 { s2 + 4096 } else { s2 }).to_ne_bytes());
            data.push(u8::try_from(u1 & 0xFF).unwrap_or(0));
            data.push(u8::try_from((u1 >> 8) | ((u2 >> 8) << 4)).unwrap_or(0));
            data.push(u8::try_from(u2 & 0xFF).unwrap_or(0));
        }

        let iter_result: Vec<(i16, i16)> = Format212Iter::new(&data).collect();
        let decode_result = decode_format_212(&data, 3, 2).unwrap();

        for (i, (s1, s2)) in iter_result.iter().enumerate() {
            assert_eq!(*s1, decode_result[0][i], "ch0 sample {i}");
            assert_eq!(*s2, decode_result[1][i], "ch1 sample {i}");
        }
    }
}
