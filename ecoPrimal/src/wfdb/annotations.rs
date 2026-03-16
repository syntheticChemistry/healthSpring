// SPDX-License-Identifier: AGPL-3.0-or-later
//! WFDB beat annotation parsing (`.atr` files).

use super::WfdbError;

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
