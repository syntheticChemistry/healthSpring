// SPDX-License-Identifier: AGPL-3.0-or-later
//! Hill equation, EC values, and JAK inhibitor reference data.

// ═══════════════════════════════════════════════════════════════════════
// Hill dose-response (Exp001)
// ═══════════════════════════════════════════════════════════════════════

/// Generalized Hill equation: `E = E_max * C^n / (C^n + IC50^n)`.
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
    let c_n = concentration.powf(hill_n);
    let ic50_n = ic50.powf(hill_n);
    e_max * c_n / (c_n + ic50_n)
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
}
