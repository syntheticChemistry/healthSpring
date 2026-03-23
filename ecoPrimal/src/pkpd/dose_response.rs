// SPDX-License-Identifier: AGPL-3.0-or-later
//! Hill equation, EC values, and JAK inhibitor reference data.

// ═══════════════════════════════════════════════════════════════════════
// Hill dose-response (Exp001)
// ═══════════════════════════════════════════════════════════════════════

/// Generalized Hill equation: `E = E_max * C^n / (C^n + IC50^n)`.
///
/// Delegates core computation to `barracuda::stats::hill`; multiplies by
/// `E_max` for dose-response scaling.
///
/// When `n = 1` this reduces to Michaelis-Menten. The Hill coefficient
/// captures cooperativity: `n > 1` = positive (steeper), `n < 1` = negative.
///
/// ```
/// use healthspring_barracuda::pkpd::hill_dose_response;
///
/// // At C = IC50, response is exactly E_max/2
/// let r = hill_dose_response(10.0, 10.0, 1.0, 1.0);
/// assert!((r - 0.5).abs() < 1e-10);
/// ```
#[must_use]
pub fn hill_dose_response(concentration: f64, ic50: f64, hill_n: f64, e_max: f64) -> f64 {
    if ic50 <= 0.0 || concentration < 0.0 {
        return 0.0;
    }
    e_max * barracuda::stats::hill(concentration, ic50, hill_n)
}

/// Sweep Hill dose-response across a concentration array.
#[must_use]
pub fn hill_sweep(ic50: f64, hill_n: f64, e_max: f64, concentrations: &[f64]) -> Vec<f64> {
    concentrations
        .iter()
        .map(|&c| hill_dose_response(c, ic50, hill_n, e_max))
        .collect()
}

/// Compute EC10, EC50, EC90 from Hill parameters.
///
/// `EC_x = IC50 * (x / (1 - x))^(1/n)`
#[must_use]
pub fn compute_ec_values(ic50: f64, hill_n: f64) -> EcValues {
    let ec50 = ic50;
    let ec10 = ic50 * (0.1_f64 / 0.9).powf(1.0 / hill_n);
    let ec90 = ic50 * (0.9_f64 / 0.1).powf(1.0 / hill_n);
    EcValues { ec10, ec50, ec90 }
}

/// EC10 / EC50 / EC90 triplet.
#[derive(Debug, Clone, Copy)]
pub struct EcValues {
    pub ec10: f64,
    pub ec50: f64,
    pub ec90: f64,
}

// ═══════════════════════════════════════════════════════════════════════
// Human JAK inhibitor reference data
// ═══════════════════════════════════════════════════════════════════════

/// Human JAK inhibitor drug profile.
#[derive(Debug, Clone)]
pub struct JakInhibitor {
    pub name: &'static str,
    pub ic50_jak1_nm: f64,
    pub hill_n: f64,
    pub selectivity: &'static str,
}

pub const BARICITINIB: JakInhibitor = JakInhibitor {
    name: "baricitinib",
    ic50_jak1_nm: 5.9,
    hill_n: 1.0,
    selectivity: "JAK1/JAK2",
};

pub const UPADACITINIB: JakInhibitor = JakInhibitor {
    name: "upadacitinib",
    ic50_jak1_nm: 8.0,
    hill_n: 1.0,
    selectivity: "JAK1",
};

pub const ABROCITINIB: JakInhibitor = JakInhibitor {
    name: "abrocitinib",
    ic50_jak1_nm: 29.0,
    hill_n: 1.0,
    selectivity: "JAK1",
};

pub const OCLACITINIB: JakInhibitor = JakInhibitor {
    name: "oclacitinib",
    ic50_jak1_nm: 10.0,
    hill_n: 1.0,
    selectivity: "JAK1 (canine)",
};

pub const ALL_INHIBITORS: [&JakInhibitor; 4] =
    [&BARICITINIB, &UPADACITINIB, &ABROCITINIB, &OCLACITINIB];

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-10;

    #[test]
    fn hill_at_ic50_is_half() {
        for drug in ALL_INHIBITORS {
            let r = hill_dose_response(drug.ic50_jak1_nm, drug.ic50_jak1_nm, drug.hill_n, 1.0);
            assert!(
                (r - 0.5).abs() < TOL,
                "{}: at IC50 got {r}, expected 0.5",
                drug.name
            );
        }
    }

    #[test]
    fn hill_monotonic_per_drug() {
        let concs: Vec<f64> = (0..100)
            .map(|i| 10.0_f64.powf(-1.0 + 5.0 * f64::from(i) / 99.0))
            .collect();
        for drug in ALL_INHIBITORS {
            let responses = hill_sweep(drug.ic50_jak1_nm, drug.hill_n, 1.0, &concs);
            for w in responses.windows(2) {
                assert!(w[0] <= w[1] + TOL, "{}: not monotonic", drug.name);
            }
        }
    }

    #[test]
    fn potency_ordering_at_10nm() {
        let r_bari = hill_dose_response(10.0, BARICITINIB.ic50_jak1_nm, 1.0, 1.0);
        let r_upa = hill_dose_response(10.0, UPADACITINIB.ic50_jak1_nm, 1.0, 1.0);
        let r_ocla = hill_dose_response(10.0, OCLACITINIB.ic50_jak1_nm, 1.0, 1.0);
        let r_abro = hill_dose_response(10.0, ABROCITINIB.ic50_jak1_nm, 1.0, 1.0);
        assert!(r_bari > r_upa, "baricitinib > upadacitinib");
        assert!(r_upa > r_ocla, "upadacitinib > oclacitinib");
        assert!(r_ocla > r_abro, "oclacitinib > abrocitinib");
    }

    #[test]
    fn ec_values_ordered() {
        for drug in ALL_INHIBITORS {
            let ec = compute_ec_values(drug.ic50_jak1_nm, drug.hill_n);
            assert!(ec.ec10 < ec.ec50, "{}: EC10 < EC50", drug.name);
            assert!(ec.ec50 < ec.ec90, "{}: EC50 < EC90", drug.name);
        }
    }

    #[test]
    fn hill_cooperativity_below_ic50() {
        let r_n1 = hill_dose_response(5.0, 10.0, 1.0, 1.0);
        let r_n2 = hill_dose_response(5.0, 10.0, 2.0, 1.0);
        assert!(r_n2 < r_n1, "n=2 steeper below IC50");
    }

    #[test]
    fn hill_cooperativity_above_ic50() {
        let r_n1 = hill_dose_response(20.0, 10.0, 1.0, 1.0);
        let r_n2 = hill_dose_response(20.0, 10.0, 2.0, 1.0);
        assert!(r_n2 > r_n1, "n=2 higher above IC50");
    }

    #[test]
    fn saturation_at_100x() {
        for drug in ALL_INHIBITORS {
            let conc = drug.ic50_jak1_nm * 100.0;
            let r = hill_dose_response(conc, drug.ic50_jak1_nm, drug.hill_n, 1.0);
            assert!(r > 0.99, "{}: saturation {r} at 100x IC50", drug.name);
        }
    }

    /// Cross-validate: `barracuda::stats::hill(x, k, n)` produces the
    /// same result as `hill_dose_response(x, k, n, 1.0)` for normalized
    /// Hill (emax=1). The upstream function was absorbed from neuralSpring
    /// and uses the same formula.
    #[test]
    fn cross_validate_hill_vs_upstream() {
        let concs = [0.1, 1.0, 5.0, 10.0, 50.0, 100.0, 1000.0];
        let k = 10.0;
        let n = 2.0;
        for &c in &concs {
            let local = hill_dose_response(c, k, n, 1.0);
            let upstream = barracuda::stats::hill(c, k, n);
            assert!(
                (local - upstream).abs() < 1e-12,
                "Hill mismatch at c={c}: local={local}, upstream={upstream}"
            );
        }
    }
}

#[cfg(test)]
mod proptest_numerical {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Hill response is always in [0, e_max] for valid inputs.
        #[test]
        fn hill_response_bounded(
            c in 1e-6_f64..1e6,
            ic50 in 1e-3_f64..1e5,
            n in 0.1_f64..10.0,
            e_max in 0.1_f64..1000.0,
        ) {
            let r = hill_dose_response(c, ic50, n, e_max);
            prop_assert!(r >= 0.0, "response negative: {r}");
            prop_assert!(r <= e_max + 1e-10, "response {r} > e_max {e_max}");
        }

        /// Hill at C = IC50 is always E_max/2 (analytical identity).
        #[test]
        fn hill_at_ic50_identity(
            ic50 in 1e-3_f64..1e5,
            n in 0.1_f64..10.0,
            e_max in 0.1_f64..1000.0,
        ) {
            let r = hill_dose_response(ic50, ic50, n, e_max);
            let expected = e_max / 2.0;
            prop_assert!(
                (r - expected).abs() < 1e-8 * e_max.max(1.0),
                "Hill at IC50: got {r}, expected {expected}"
            );
        }

        /// Hill is monotonically non-decreasing for increasing concentration.
        #[test]
        fn hill_monotonic(
            ic50 in 1e-3_f64..1e5,
            n in 0.5_f64..5.0,
            c1 in 1e-6_f64..1e5,
            delta in 0.0_f64..1e4,
        ) {
            let c2 = c1 + delta;
            let r1 = hill_dose_response(c1, ic50, n, 1.0);
            let r2 = hill_dose_response(c2, ic50, n, 1.0);
            prop_assert!(r2 >= r1 - 1e-12, "not monotonic: r({c1})={r1} > r({c2})={r2}");
        }

        /// EC values are always ordered: EC10 < EC50 < EC90.
        #[test]
        fn ec_values_always_ordered(
            ic50 in 1e-3_f64..1e5,
            n in 0.1_f64..10.0,
        ) {
            let ec = compute_ec_values(ic50, n);
            prop_assert!(ec.ec10 < ec.ec50, "EC10 {0} >= EC50 {1}", ec.ec10, ec.ec50);
            prop_assert!(ec.ec50 < ec.ec90, "EC50 {0} >= EC90 {1}", ec.ec50, ec.ec90);
        }

        /// Hill with zero concentration returns zero.
        #[test]
        fn hill_zero_at_zero(
            ic50 in 1e-3_f64..1e5,
            n in 0.1_f64..10.0,
            e_max in 0.1_f64..1000.0,
        ) {
            let r = hill_dose_response(0.0, ic50, n, e_max);
            prop_assert!((r).abs() < 1e-15, "Hill(0) should be 0, got {r}");
        }
    }
}
