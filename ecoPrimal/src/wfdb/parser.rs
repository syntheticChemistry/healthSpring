// SPDX-License-Identifier: AGPL-3.0-or-later
//! WFDB format parsing: header (`.hea`) and binary signal data (`.dat`).

use super::WfdbError;

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
