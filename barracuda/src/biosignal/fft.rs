// SPDX-License-Identifier: AGPL-3.0-or-later
//! Minimal real FFT (DFT-based, no external dependency).
//!
//! O(n²) reference implementation — adequate for short biosignal segments.
//! Marked for replacement with radix-2 FFT (`fft_radix2_f64`) when
//! barraCuda absorbs it.

use std::f64::consts::PI;

/// Convert index to f64 (avoids repeated `clippy::cast_precision_loss`).
#[expect(clippy::cast_precision_loss, reason = "indices ≪ 2^52")]
pub(crate) const fn idx_to_f64(v: usize) -> f64 {
    v as f64
}

#[expect(clippy::cast_precision_loss, reason = "indices ≪ 2^52")]
pub(crate) const fn u64_to_f64(v: u64) -> f64 {
    v as f64
}

/// Forward real FFT: returns `(re, im)` arrays of length `n/2 + 1`.
#[must_use]
pub fn rfft(signal: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let n = signal.len();
    let n_freq = n / 2 + 1;
    let mut re = vec![0.0; n_freq];
    let mut im = vec![0.0; n_freq];
    let nf = idx_to_f64(n);
    for k in 0..n_freq {
        let kf = idx_to_f64(k);
        for (j, &s) in signal.iter().enumerate() {
            let angle = 2.0 * PI * kf * idx_to_f64(j) / nf;
            re[k] += s * angle.cos();
            im[k] -= s * angle.sin();
        }
    }
    (re, im)
}

/// Inverse real FFT from `n_freq`-length spectra back to `n` time-domain samples.
#[must_use]
pub fn irfft(re: &[f64], im: &[f64], n: usize) -> Vec<f64> {
    let n_freq = re.len();
    let mut out = vec![0.0; n];
    let nf = idx_to_f64(n);
    for (j, slot) in out.iter_mut().enumerate() {
        let jf = idx_to_f64(j);
        for k in 0..n_freq {
            let angle = 2.0 * PI * idx_to_f64(k) * jf / nf;
            let mut contribution = re[k].mul_add(angle.cos(), -(im[k] * angle.sin()));
            if k > 0 && k < n_freq - 1 {
                contribution *= 2.0;
            }
            *slot += contribution;
        }
        *slot /= nf;
    }
    out
}
