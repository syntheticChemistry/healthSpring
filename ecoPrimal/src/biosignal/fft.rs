// SPDX-License-Identifier: AGPL-3.0-only
//! Real FFT — radix-2 Cooley-Tukey with zero-pad to power-of-two.
//!
//! Pure Rust, no external dependency. Replaces the previous O(n²) DFT with
//! O(n log n) for biosignal processing (ECG bandpass, HRV power spectrum).
//!
//! Non-power-of-two inputs are zero-padded; output is trimmed back to the
//! original length in `irfft`.

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

/// Smallest power of two >= n.
const fn next_power_of_two(n: usize) -> usize {
    n.next_power_of_two()
}

/// In-place radix-2 Cooley-Tukey complex FFT (decimation-in-time).
///
/// `re` and `im` must have length `n` where `n` is a power of two.
fn fft_complex_inplace(re: &mut [f64], im: &mut [f64], inverse: bool) {
    let n = re.len();
    debug_assert!(n.is_power_of_two());
    debug_assert_eq!(re.len(), im.len());

    // Bit-reversal permutation.
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }

    // Butterfly stages.
    let sign = if inverse { 1.0 } else { -1.0 };
    let mut len = 2;
    while len <= n {
        let half = len / 2;
        let angle = sign * 2.0 * PI / idx_to_f64(len);
        let (w_re, w_im) = (angle.cos(), angle.sin());

        let mut i = 0;
        while i < n {
            let (mut cur_re, mut cur_im): (f64, f64) = (1.0, 0.0);
            for k in 0..half {
                let u_re = re[i + k];
                let u_im = im[i + k];
                let v_re = cur_re.mul_add(re[i + k + half], -(cur_im * im[i + k + half]));
                let v_im = cur_re.mul_add(im[i + k + half], cur_im * re[i + k + half]);
                re[i + k] = u_re + v_re;
                im[i + k] = u_im + v_im;
                re[i + k + half] = u_re - v_re;
                im[i + k + half] = u_im - v_im;
                let next_re = cur_re.mul_add(w_re, -(cur_im * w_im));
                let next_im = cur_re.mul_add(w_im, cur_im * w_re);
                cur_re = next_re;
                cur_im = next_im;
            }
            i += len;
        }
        len <<= 1;
    }

    if inverse {
        let nf = idx_to_f64(n);
        for (r, i) in re.iter_mut().zip(im.iter_mut()) {
            *r /= nf;
            *i /= nf;
        }
    }
}

/// Forward real FFT: returns `(re, im)` arrays of length `n_padded/2 + 1`.
///
/// Input is zero-padded to the next power of two for the radix-2 kernel.
/// Callers that compute bin frequencies should use `(re.len() - 1) * 2`
/// as the effective sample count for `freq = k * fs / n_effective`.
#[must_use]
pub fn rfft(signal: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let n = signal.len();
    if n == 0 {
        return (vec![0.0], vec![0.0]);
    }
    let n_padded = next_power_of_two(n);
    let mut re = vec![0.0; n_padded];
    let mut im = vec![0.0; n_padded];
    re[..n].copy_from_slice(signal);

    fft_complex_inplace(&mut re, &mut im, false);

    let n_freq = n_padded / 2 + 1;
    re.truncate(n_freq);
    im.truncate(n_freq);
    (re, im)
}

/// Inverse real FFT from `n_freq`-length spectra back to `n` time-domain samples.
///
/// Reconstructs via the padded FFT size `(re.len() - 1) * 2`, then truncates
/// to `n` samples.
#[must_use]
pub fn irfft(re: &[f64], im: &[f64], n: usize) -> Vec<f64> {
    if n == 0 {
        return vec![];
    }
    let n_padded = (re.len() - 1) * 2;
    let n_padded = n_padded.max(next_power_of_two(n));
    let n_freq = re.len();

    let mut full_re = vec![0.0; n_padded];
    let mut full_im = vec![0.0; n_padded];

    let copy_len = n_freq.min(n_padded);
    full_re[..copy_len].copy_from_slice(&re[..copy_len]);
    full_im[..copy_len].copy_from_slice(&im[..copy_len]);

    for k in 1..n_padded / 2 {
        if k < n_freq {
            full_re[n_padded - k] = re[k];
            full_im[n_padded - k] = -im[k];
        }
    }

    fft_complex_inplace(&mut full_re, &mut full_im, true);

    full_re.truncate(n);
    full_re
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn rfft_dc_component() {
        let signal = vec![1.0; 8];
        let (re, im) = rfft(&signal);
        assert!((re[0] - 8.0).abs() < 1e-10, "DC = sum of all samples");
        assert!(im[0].abs() < 1e-10, "DC imaginary = 0");
        for (k, component) in re.iter().enumerate().skip(1) {
            assert!(
                component.abs() < 1e-10,
                "no AC for constant signal at k={k}"
            );
        }
    }

    #[test]
    fn rfft_irfft_roundtrip() {
        let signal = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let (re, im) = rfft(&signal);
        let recovered = irfft(&re, &im, signal.len());
        for (i, (&orig, &rec)) in signal.iter().zip(recovered.iter()).enumerate() {
            assert!(
                (orig - rec).abs() < 1e-10,
                "mismatch at {i}: {orig} vs {rec}"
            );
        }
    }

    #[test]
    fn rfft_irfft_roundtrip_non_power_of_two() {
        let signal: Vec<f64> = (0..13).map(|i| idx_to_f64(i) * 0.3).collect();
        let (re, im) = rfft(&signal);
        let recovered = irfft(&re, &im, signal.len());
        for (i, (&orig, &rec)) in signal.iter().zip(recovered.iter()).enumerate() {
            assert!(
                (orig - rec).abs() < 1e-8,
                "mismatch at {i}: {orig} vs {rec}"
            );
        }
    }

    #[test]
    fn rfft_single_tone() {
        let n = 64;
        let signal: Vec<f64> = (0..n)
            .map(|i| (2.0 * PI * 4.0 * idx_to_f64(i) / idx_to_f64(n)).sin())
            .collect();
        let (re, im) = rfft(&signal);
        let magnitudes: Vec<f64> = re.iter().zip(im.iter()).map(|(r, i)| r.hypot(*i)).collect();
        let peak_bin = magnitudes
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap()
            .0;
        assert_eq!(peak_bin, 4, "peak at bin 4 for 4-cycle sine");
    }

    #[test]
    fn empty_signals() {
        let (re, im) = rfft(&[]);
        assert_eq!(re.len(), 1);
        assert_eq!(im.len(), 1);
        let out = irfft(&re, &im, 0);
        assert!(out.is_empty());
    }
}
