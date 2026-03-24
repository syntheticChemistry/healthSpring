// SPDX-License-Identifier: AGPL-3.0-or-later
//! WFDB beat annotation parsing (`.atr` files).

use super::WfdbError;

/// Annotation beat type (subset of WFDB annotation codes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeatType {
    /// Normal sinus beat (code 1).
    Normal,
    /// Left bundle branch block beat (code 7).
    LeftBundleBranch,
    /// Right bundle branch block beat (code 8).
    RightBundleBranch,
    /// Atrial premature beat (code 5).
    AtrialPremature,
    /// Ventricular premature beat (code 6).
    VentricularPremature,
    /// Paced beat (code 11).
    PacedBeat,
    /// Fusion of ventricular and normal beat (code 10).
    FusionVentricular,
    /// Unrecognized or non-standard WFDB code.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beat_type_known_codes() {
        assert_eq!(BeatType::from_code(1), BeatType::Normal);
        assert_eq!(BeatType::from_code(5), BeatType::AtrialPremature);
        assert_eq!(BeatType::from_code(6), BeatType::VentricularPremature);
        assert_eq!(BeatType::from_code(7), BeatType::LeftBundleBranch);
        assert_eq!(BeatType::from_code(8), BeatType::RightBundleBranch);
        assert_eq!(BeatType::from_code(10), BeatType::FusionVentricular);
        assert_eq!(BeatType::from_code(11), BeatType::PacedBeat);
    }

    #[test]
    fn beat_type_unknown_code() {
        assert_eq!(BeatType::from_code(0), BeatType::Unknown(0));
        assert_eq!(BeatType::from_code(99), BeatType::Unknown(99));
    }

    #[test]
    fn parse_empty_data() {
        let data: &[u8] = &[];
        let anns = parse_annotations(data).expect("empty data is valid");
        assert!(anns.is_empty());
    }

    #[test]
    fn parse_single_byte_insufficient() {
        let data: &[u8] = &[0x01];
        let anns = parse_annotations(data).expect("single byte terminates cleanly");
        assert!(anns.is_empty());
    }

    #[test]
    fn parse_terminator() {
        let data: &[u8] = &[0x00, 0x00];
        let anns = parse_annotations(data).expect("terminator is valid");
        assert!(anns.is_empty());
    }

    #[test]
    fn parse_single_normal_beat() {
        // delta=100 (low byte), type=1 (Normal), delta_high=0
        // b1 = (1 << 2) | 0 = 4
        let data: &[u8] = &[100, 0x04, 0x00, 0x00];
        let anns = parse_annotations(data).expect("single beat");
        assert_eq!(anns.len(), 1);
        assert_eq!(anns[0].sample, 100);
        assert_eq!(anns[0].beat_type, BeatType::Normal);
    }

    #[test]
    fn parse_multiple_beats_accumulates_delta() {
        // Beat 1: delta=50, type=1 (Normal)  → sample=50
        // Beat 2: delta=30, type=6 (VPB)     → sample=80
        let data: &[u8] = &[
            50, 0x04, // Normal at delta=50
            30, 0x18, // VPB (6<<2=24=0x18) at delta=30
            0x00, 0x00, // terminator
        ];
        let anns = parse_annotations(data).expect("two beats");
        assert_eq!(anns.len(), 2);
        assert_eq!(anns[0].sample, 50);
        assert_eq!(anns[0].beat_type, BeatType::Normal);
        assert_eq!(anns[1].sample, 80);
        assert_eq!(anns[1].beat_type, BeatType::VentricularPremature);
    }

    #[test]
    fn parse_aux_annotation_skipped() {
        // AUX type = 63, delta = 4 (aux_len = 4 bytes)
        // b1 = (63 << 2) | 0 = 252
        let data: &[u8] = &[
            4, 252, // AUX with 4 bytes of aux data
            b'(', b'N', b')', 0, // 4 bytes of aux data (padded)
            50, 0x04, // Normal beat at delta=50
            0x00, 0x00, // terminator
        ];
        let anns = parse_annotations(data).expect("AUX skipped");
        assert_eq!(anns.len(), 1);
        assert_eq!(anns[0].sample, 50);
    }

    #[test]
    fn parse_truncated_aux_returns_error() {
        // AUX type = 63, delta = 10 (aux_len = 10 bytes, but data is too short)
        let data: &[u8] = &[10, 252, 0x01, 0x02];
        let result = parse_annotations(data);
        assert!(result.is_err());
    }

    #[test]
    fn parse_skip_annotation() {
        // SKIP type = 59 (59 << 2 = 236)
        // delta in next 4 bytes: skip_lo=1000, skip_hi=0 → jump 1000 samples
        let data: &[u8] = &[
            0, 236, // SKIP, delta=0
            0xE8, 0x03, 0x00, 0x00, // skip_lo=1000, skip_hi=0
            50, 0x04, // Normal beat at delta=50
            0x00, 0x00, // terminator
        ];
        let anns = parse_annotations(data).expect("SKIP processed");
        assert_eq!(anns.len(), 1);
        assert_eq!(anns[0].sample, 1050, "1000 (SKIP) + 50 (delta)");
    }

    #[test]
    fn parse_truncated_skip_returns_error() {
        // SKIP type = 59 but not enough bytes for the 4-byte skip value
        let data: &[u8] = &[0, 236, 0x01, 0x02];
        let result = parse_annotations(data);
        assert!(result.is_err());
    }
}
