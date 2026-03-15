// SPDX-License-Identifier: AGPL-3.0-only
//! Beat morphology classification for arrhythmia detection.
//!
//! Template-matching approach: compare each detected QRS complex
//! against reference templates for Normal, PVC, and PAC beats.
//!
//! Reference: MIT-BIH Arrhythmia Database (Moody & Mark 2001),
//! AAMI EC57 performance standards.

/// Beat classification labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeatClass {
    /// Normal sinus beat
    Normal,
    /// Premature Ventricular Contraction (wide QRS, no preceding P-wave)
    Pvc,
    /// Premature Atrial Contraction (early, narrow QRS)
    Pac,
    /// Unclassifiable
    Unknown,
}

impl std::fmt::Display for BeatClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "N"),
            Self::Pvc => write!(f, "V"),
            Self::Pac => write!(f, "A"),
            Self::Unknown => write!(f, "?"),
        }
    }
}

/// Template for beat classification.
#[derive(Debug, Clone)]
pub struct BeatTemplate {
    pub class: BeatClass,
    pub waveform: Vec<f64>,
}

/// Classification result for a single beat.
#[derive(Debug, Clone)]
pub struct BeatResult {
    pub peak_index: usize,
    pub class: BeatClass,
    pub correlation: f64,
}

/// Classification confusion matrix.
#[derive(Debug, Clone, Default)]
pub struct ConfusionMatrix {
    pub true_positive: u32,
    pub false_positive: u32,
    pub true_negative: u32,
    pub false_negative: u32,
}

impl ConfusionMatrix {
    #[must_use]
    pub fn sensitivity(&self) -> f64 {
        let denom = self.true_positive + self.false_negative;
        if denom == 0 {
            return 0.0;
        }
        f64::from(self.true_positive) / f64::from(denom)
    }

    #[must_use]
    pub fn specificity(&self) -> f64 {
        let denom = self.true_negative + self.false_positive;
        if denom == 0 {
            return 0.0;
        }
        f64::from(self.true_negative) / f64::from(denom)
    }

    #[must_use]
    pub fn ppv(&self) -> f64 {
        let denom = self.true_positive + self.false_positive;
        if denom == 0 {
            return 0.0;
        }
        f64::from(self.true_positive) / f64::from(denom)
    }
}

/// Extract a QRS window from the signal around a peak.
#[must_use]
pub fn extract_beat_window(signal: &[f64], peak: usize, half_width: usize) -> Vec<f64> {
    let start = peak.saturating_sub(half_width);
    let end = (peak + half_width + 1).min(signal.len());
    signal[start..end].to_vec()
}

/// Normalized cross-correlation between two waveforms.
#[must_use]
pub fn normalized_correlation(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }

    #[expect(clippy::cast_precision_loss, reason = "window length fits f64")]
    let n_f64 = n as f64;
    let mean_a: f64 = a.iter().take(n).sum::<f64>() / n_f64;
    let mean_b: f64 = b.iter().take(n).sum::<f64>() / n_f64;

    let mut cov = 0.0;
    let mut var_a = 0.0;
    let mut var_b = 0.0;
    for i in 0..n {
        let da = a[i] - mean_a;
        let db = b[i] - mean_b;
        cov += da * db;
        var_a += da * da;
        var_b += db * db;
    }

    let denom = (var_a * var_b).sqrt();
    if denom < 1e-14 {
        return 0.0;
    }
    cov / denom
}

/// Classify a beat by template matching.
///
/// Returns the `BeatClass` of the best-matching template if the correlation
/// exceeds `min_correlation`, otherwise `Unknown`.
#[must_use]
pub fn classify_beat(
    beat_window: &[f64],
    templates: &[BeatTemplate],
    min_correlation: f64,
) -> (BeatClass, f64) {
    let mut best_class = BeatClass::Unknown;
    let mut best_corr = f64::NEG_INFINITY;

    for tmpl in templates {
        let corr = normalized_correlation(beat_window, &tmpl.waveform);
        if corr > best_corr {
            best_corr = corr;
            best_class = tmpl.class;
        }
    }

    if best_corr >= min_correlation {
        (best_class, best_corr)
    } else {
        (BeatClass::Unknown, best_corr)
    }
}

/// Classify all detected beats in a signal.
#[must_use]
pub fn classify_all_beats(
    signal: &[f64],
    peaks: &[usize],
    templates: &[BeatTemplate],
    half_width: usize,
    min_correlation: f64,
) -> Vec<BeatResult> {
    peaks
        .iter()
        .map(|&pk| {
            let window = extract_beat_window(signal, pk, half_width);
            let (class, correlation) = classify_beat(&window, templates, min_correlation);
            BeatResult {
                peak_index: pk,
                class,
                correlation,
            }
        })
        .collect()
}

/// Generate synthetic normal QRS template (Gaussian P-QRS-T model).
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "sample count fits f64")]
pub fn generate_normal_template(n_samples: usize) -> Vec<f64> {
    let center = n_samples as f64 / 2.0;
    let qrs_width = n_samples as f64 / 6.0;
    (0..n_samples)
        .map(|i| {
            let t = i as f64 - center;
            (-(t * t) / (2.0 * qrs_width * qrs_width)).exp()
        })
        .collect()
}

/// Generate synthetic PVC template (wider, deeper QRS).
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "sample count fits f64")]
pub fn generate_pvc_template(n_samples: usize) -> Vec<f64> {
    let center = n_samples as f64 / 2.0;
    let qrs_width = n_samples as f64 / 4.0;
    (0..n_samples)
        .map(|i| {
            let t = i as f64 - center;
            -(-(t * t) / (2.0 * qrs_width * qrs_width)).exp()
        })
        .collect()
}

/// Generate synthetic PAC template (narrower, early).
#[must_use]
#[expect(clippy::cast_precision_loss, reason = "sample count fits f64")]
pub fn generate_pac_template(n_samples: usize) -> Vec<f64> {
    let center = (n_samples as f64).mul_add(-0.1, n_samples as f64 / 2.0);
    let qrs_width = n_samples as f64 / 7.0;
    (0..n_samples)
        .map(|i| {
            let t = i as f64 - center;
            0.8 * (-(t * t) / (2.0 * qrs_width * qrs_width)).exp()
        })
        .collect()
}

/// Compute confusion matrix for a specific class.
#[must_use]
pub fn confusion_for_class(
    predictions: &[BeatClass],
    truths: &[BeatClass],
    target: BeatClass,
) -> ConfusionMatrix {
    let mut cm = ConfusionMatrix::default();
    for (p, t) in predictions.iter().zip(truths.iter()) {
        match (*p == target, *t == target) {
            (true, true) => cm.true_positive += 1,
            (true, false) => cm.false_positive += 1,
            (false, true) => cm.false_negative += 1,
            (false, false) => cm.true_negative += 1,
        }
    }
    cm
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn normal_template_peak_at_center() {
        let n = 41;
        let t = generate_normal_template(n);
        let max_idx = t
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap()
            .0;
        let center = n / 2;
        assert!(
            max_idx.abs_diff(center) <= 1,
            "peak should be near center: got {max_idx}, expected ~{center}"
        );
    }

    #[test]
    fn pvc_template_negative_peak() {
        let t = generate_pvc_template(41);
        let min_val = t.iter().copied().fold(f64::INFINITY, f64::min);
        assert!(
            min_val < 0.0,
            "PVC template should have negative deflection"
        );
    }

    #[test]
    fn self_correlation_is_one() {
        let t = generate_normal_template(41);
        let corr = normalized_correlation(&t, &t);
        assert!(
            (corr - 1.0).abs() < 1e-10,
            "self-correlation should be 1.0: {corr}"
        );
    }

    #[test]
    fn normal_vs_pvc_low_correlation() {
        let n = generate_normal_template(41);
        let p = generate_pvc_template(41);
        let corr = normalized_correlation(&n, &p);
        assert!(
            corr < 0.5,
            "normal vs PVC correlation should be low: {corr}"
        );
    }

    #[test]
    fn classify_identifies_matching_template() {
        let templates = vec![
            BeatTemplate {
                class: BeatClass::Normal,
                waveform: generate_normal_template(41),
            },
            BeatTemplate {
                class: BeatClass::Pvc,
                waveform: generate_pvc_template(41),
            },
        ];
        let beat = generate_normal_template(41);
        let (class, corr) = classify_beat(&beat, &templates, 0.7);
        assert_eq!(class, BeatClass::Normal);
        assert!(corr > 0.99);
    }

    #[test]
    fn confusion_matrix_sensitivity() {
        let cm = ConfusionMatrix {
            true_positive: 90,
            false_negative: 10,
            true_negative: 85,
            false_positive: 15,
        };
        assert!((cm.sensitivity() - 0.9).abs() < 1e-10);
        assert!((cm.specificity() - 0.85).abs() < 1e-10);
    }
}
