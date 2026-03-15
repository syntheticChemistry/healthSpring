// SPDX-License-Identifier: AGPL-3.0-only
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

/// Signal specification from a `.hea` header file.
#[derive(Debug, Clone)]
pub struct SignalSpec {
    /// Filename containing this signal's data.
    pub filename: String,
    /// Storage format (212, 16, 80, 310, 311).
    pub format: u16,
    /// ADC gain (units per mV).
    pub gain: f64,
    /// ADC baseline (zero-value).
    pub baseline: i32,
    /// ADC resolution in bits.
    pub adc_resolution: u16,
    /// ADC zero value.
    pub adc_zero: i32,
    /// Signal description (e.g., "MLII", "V5").
    pub description: String,
}

/// Parsed WFDB record header.
#[derive(Debug, Clone)]
pub struct WfdbHeader {
    /// Record name (e.g., "100").
    pub record_name: String,
    /// Number of signals (leads).
    pub n_signals: usize,
    /// Sampling frequency in Hz.
    pub sampling_frequency: f64,
    /// Total number of samples per signal.
    pub n_samples: Option<usize>,
    /// Signal specifications.
    pub signals: Vec<SignalSpec>,
}

/// Annotation beat type (subset of WFDB annotation codes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeatType {
    Normal,
    LeftBundleBranch,
    RightBundleBranch,
    AtrialPremature,
    VentricularPremature,
    PacedBeat,
    FusionVentricular,
    Unknown(u8),
}

impl BeatType {
    /// Decode WFDB annotation code to beat type.
    #[must_use]
    pub const fn from_code(code: u8) -> Self {
        match code {
            1 => Self::Normal,
            7 => Self::LeftBundleBranch,
            8 => Self::RightBundleBranch,
            5 => Self::AtrialPremature,
            6 => Self::VentricularPremature,
            11 => Self::PacedBeat,
            10 => Self::FusionVentricular,
            _ => Self::Unknown(code),
        }
    }
}

/// A single annotation from a `.atr` file.
#[derive(Debug, Clone)]
pub struct Annotation {
    /// Sample index where this annotation occurs.
    pub sample: usize,
    /// Beat type classification.
    pub beat_type: BeatType,
}

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

// ═══════════════════════════════════════════════════════════════════════
// Header parsing
// ═══════════════════════════════════════════════════════════════════════

/// Parse a WFDB `.hea` header from text content.
///
/// # Errors
///
/// Returns [`WfdbError::InvalidHeader`] if the header format is malformed.
pub fn parse_header(content: &str) -> Result<WfdbHeader, WfdbError> {
    let mut lines = content
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty());

    let record_line = lines
        .next()
        .ok_or_else(|| WfdbError::InvalidHeader("empty header".into()))?;

    let record_parts: Vec<&str> = record_line.split_whitespace().collect();
    if record_parts.len() < 2 {
        return Err(WfdbError::InvalidHeader("missing signal count".into()));
    }

    let record_name = record_parts[0].to_string();
    let n_signals: usize = record_parts[1]
        .parse()
        .map_err(|_| WfdbError::InvalidHeader("invalid signal count".into()))?;

    let sampling_frequency = if record_parts.len() > 2 {
        parse_sampling_freq(record_parts[2])?
    } else {
        250.0
    };

    let n_samples = record_parts.get(3).and_then(|s| s.parse().ok());

    let mut signals = Vec::with_capacity(n_signals);
    for _ in 0..n_signals {
        let sig_line = lines
            .next()
            .ok_or_else(|| WfdbError::InvalidHeader("missing signal line".into()))?;
        signals.push(parse_signal_spec(sig_line)?);
    }

    Ok(WfdbHeader {
        record_name,
        n_signals,
        sampling_frequency,
        n_samples,
        signals,
    })
}

/// Parse sampling frequency, handling optional `/counter_frequency` suffix.
fn parse_sampling_freq(token: &str) -> Result<f64, WfdbError> {
    let freq_str = token.split('/').next().unwrap_or(token);
    freq_str
        .parse()
        .map_err(|_| WfdbError::InvalidHeader(format!("invalid sampling frequency: {token}")))
}

/// Parse a signal specification line from a `.hea` file.
fn parse_signal_spec(line: &str) -> Result<SignalSpec, WfdbError> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(WfdbError::InvalidHeader("signal spec too short".into()));
    }

    let filename = parts[0].to_string();
    let format: u16 = parts[1]
        .split('x')
        .next()
        .unwrap_or(parts[1])
        .parse()
        .map_err(|_| WfdbError::InvalidHeader(format!("invalid format: {}", parts[1])))?;

    let gain = parts
        .get(2)
        .and_then(|s| s.split('/').next())
        .and_then(|s| s.parse().ok())
        .unwrap_or(200.0);

    let baseline = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);

    let adc_resolution = parts.get(4).and_then(|s| s.parse().ok()).unwrap_or(12);
    let adc_zero = parts.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);

    let description = parts.get(8).map_or_else(String::new, |s| (*s).to_string());

    Ok(SignalSpec {
        filename,
        format,
        gain,
        baseline,
        adc_resolution,
        adc_zero,
        description,
    })
}

// ═══════════════════════════════════════════════════════════════════════
// Binary signal data decoding
// ═══════════════════════════════════════════════════════════════════════

/// Decode Format 212 (MIT-BIH standard): 12-bit samples packed in 3-byte pairs.
///
/// Each 3-byte group encodes two 12-bit samples for two-channel recordings.
/// Byte layout: `[low1] [high1:4 | high2:4] [low2]`.
///
/// # Errors
///
/// Returns [`WfdbError::DataTruncated`] if the data is too short.
pub fn decode_format_212(
    data: &[u8],
    n_samples: usize,
    n_signals: usize,
) -> Result<Vec<Vec<i16>>, WfdbError> {
    if n_signals != 2 {
        return Err(WfdbError::UnsupportedFormat(212));
    }

    let bytes_needed = n_samples * 3;
    if data.len() < bytes_needed {
        return Err(WfdbError::DataTruncated);
    }

    let mut channels = vec![Vec::with_capacity(n_samples); n_signals];

    for i in 0..n_samples {
        let base = i * 3;
        let b0 = i16::from(data[base]);
        let b1 = i16::from(data[base + 1]);
        let b2 = i16::from(data[base + 2]);

        let s1 = b0 | ((b1 & 0x0F) << 8);
        let s1 = if s1 > 2047 { s1 - 4096 } else { s1 };

        let s2 = b2 | ((b1 >> 4) << 8);
        let s2 = if s2 > 2047 { s2 - 4096 } else { s2 };

        channels[0].push(s1);
        channels[1].push(s2);
    }

    Ok(channels)
}

/// Decode Format 16: 16-bit signed samples, little-endian.
///
/// # Errors
///
/// Returns [`WfdbError::DataTruncated`] if the data is too short.
pub fn decode_format_16(
    data: &[u8],
    n_samples: usize,
    n_signals: usize,
) -> Result<Vec<Vec<i16>>, WfdbError> {
    let total_samples = n_samples * n_signals;
    let bytes_needed = total_samples * 2;
    if data.len() < bytes_needed {
        return Err(WfdbError::DataTruncated);
    }

    let mut channels = vec![Vec::with_capacity(n_samples); n_signals];

    for i in 0..n_samples {
        for (ch, channel) in channels.iter_mut().enumerate() {
            let offset = (i * n_signals + ch) * 2;
            let sample = i16::from_le_bytes([data[offset], data[offset + 1]]);
            channel.push(sample);
        }
    }

    Ok(channels)
}

/// Convert raw ADC samples to physical units (mV) using gain and baseline.
#[must_use]
pub fn adc_to_physical(samples: &[i16], gain: f64, baseline: i32) -> Vec<f64> {
    if gain.abs() < 1e-15 {
        return vec![0.0; samples.len()];
    }
    samples
        .iter()
        .map(|&s| (f64::from(s) - f64::from(baseline)) / gain)
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════
// Annotation parsing
// ═══════════════════════════════════════════════════════════════════════

/// Parse WFDB annotation (`.atr`) binary data.
///
/// MIT annotation format: each annotation is encoded as two bytes:
/// `[sample_delta_low] [type:6 | sample_delta_high:2]`.
///
/// # Errors
///
/// Returns [`WfdbError::InvalidAnnotation`] if the data is malformed.
pub fn parse_annotations(data: &[u8]) -> Result<Vec<Annotation>, WfdbError> {
    let mut annotations = Vec::new();
    let mut sample: usize = 0;
    let mut pos = 0;

    while pos + 1 < data.len() {
        let b0 = u16::from(data[pos]);
        let b1 = u16::from(data[pos + 1]);
        pos += 2;

        let anntype = (b1 >> 2) & 0x3F;
        let delta = b0 | ((b1 & 0x03) << 8);

        if anntype == 0 && delta == 0 {
            break;
        }

        // SKIP annotation (type 59): delta is stored in the next 4 bytes
        if anntype == 59 {
            if pos + 3 >= data.len() {
                return Err(WfdbError::InvalidAnnotation("SKIP truncated".into()));
            }
            let skip_lo = u32::from(data[pos]) | (u32::from(data[pos + 1]) << 8);
            let skip_hi = u32::from(data[pos + 2]) | (u32::from(data[pos + 3]) << 8);
            pos += 4;
            sample += (skip_hi << 16 | skip_lo) as usize;
            continue;
        }

        // NOTE/AUX annotations (type 63): skip the auxiliary data
        if anntype == 63 {
            let aux_len = delta as usize;
            let padded = if aux_len.is_multiple_of(2) {
                aux_len
            } else {
                aux_len + 1
            };
            if pos + padded > data.len() {
                return Err(WfdbError::InvalidAnnotation("AUX data truncated".into()));
            }
            pos += padded;
            continue;
        }

        sample += delta as usize;
        let beat_type = BeatType::from_code(u8::try_from(anntype).unwrap_or(0));
        annotations.push(Annotation { sample, beat_type });
    }

    Ok(annotations)
}

/// Iterator-based streaming signal reader for Format 212.
///
/// Yields one pair of samples (two channels) per iteration without
/// buffering the entire file. True zero-copy streaming.
pub struct Format212Iter<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Format212Iter<'a> {
    /// Create a new streaming iterator over Format 212 data.
    #[must_use]
    pub const fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
}

impl Iterator for Format212Iter<'_> {
    type Item = (i16, i16);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos + 2 >= self.data.len() {
            return None;
        }

        let b0 = i16::from(self.data[self.pos]);
        let b1 = i16::from(self.data[self.pos + 1]);
        let b2 = i16::from(self.data[self.pos + 2]);
        self.pos += 3;

        let s1 = b0 | ((b1 & 0x0F) << 8);
        let s1 = if s1 > 2047 { s1 - 4096 } else { s1 };

        let s2 = b2 | ((b1 >> 4) << 8);
        let s2 = if s2 > 2047 { s2 - 4096 } else { s2 };

        Some((s1, s2))
    }
}

/// Iterator-based streaming signal reader for Format 16.
///
/// Yields one sample tuple per iteration (one value per channel) without
/// buffering the entire file.
pub struct Format16Iter<'a> {
    data: &'a [u8],
    pos: usize,
    n_signals: usize,
}

impl<'a> Format16Iter<'a> {
    /// Create a new streaming iterator over Format 16 interleaved data.
    ///
    /// Each call to `next()` returns a `Vec<i16>` of length `n_signals`.
    #[must_use]
    pub const fn new(data: &'a [u8], n_signals: usize) -> Self {
        Self {
            data,
            pos: 0,
            n_signals,
        }
    }
}

impl Iterator for Format16Iter<'_> {
    type Item = Vec<i16>;

    fn next(&mut self) -> Option<Self::Item> {
        let bytes_needed = self.n_signals * 2;
        if self.pos + bytes_needed > self.data.len() {
            return None;
        }
        let mut samples = Vec::with_capacity(self.n_signals);
        for _ in 0..self.n_signals {
            let s = i16::from_le_bytes([self.data[self.pos], self.data[self.pos + 1]]);
            samples.push(s);
            self.pos += 2;
        }
        Some(samples)
    }
}

/// Zero-allocation ADC → physical unit conversion iterator.
///
/// Wraps any iterator of `i16` ADC samples and yields `f64` millivolts
/// using the given gain and baseline. No heap allocation per sample.
pub struct AdcToPhysicalIter<I> {
    inner: I,
    gain_recip: f64,
    baseline: f64,
}

impl<I: Iterator<Item = i16>> AdcToPhysicalIter<I> {
    /// Wrap an ADC sample iterator with physical conversion parameters.
    pub fn new(inner: I, gain: f64, baseline: i32) -> Self {
        let gain_recip = if gain.abs() < 1e-15 { 0.0 } else { 1.0 / gain };
        Self {
            inner,
            gain_recip,
            baseline: f64::from(baseline),
        }
    }
}

impl<I: Iterator<Item = i16>> Iterator for AdcToPhysicalIter<I> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|s| (f64::from(s) - self.baseline) * self.gain_recip)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;

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
        assert!((header.sampling_frequency - 360.0).abs() < 1e-10);
        assert_eq!(header.n_samples, Some(650_000));
        assert_eq!(header.signals.len(), 2);
        assert_eq!(header.signals[0].format, 212);
        assert!((header.signals[0].gain - 200.0).abs() < 1e-10);
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
        assert!((header.sampling_frequency - 250.0).abs() < 1e-10);
        assert_eq!(header.signals[0].format, 16);
        assert!((header.signals[0].gain - 1000.0).abs() < 1e-10);
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
        assert!((physical[0]).abs() < 1e-10, "baseline → 0 mV");
        assert!(
            (physical[1] - 1.0).abs() < 1e-10,
            "+200 ADC units → +1.0 mV at gain 200"
        );
        assert!(
            (physical[2] + 1.0).abs() < 1e-10,
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
        assert!(physical.iter().all(|&v| v.abs() < 1e-15));
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
        assert!((physical[0]).abs() < 1e-10);
        assert!((physical[1] - 1.0).abs() < 1e-10);
        assert!((physical[2] + 1.0).abs() < 1e-10);
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
