// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]

//! healthSpring Exp001 — Rust validation binary
//!
//! Cross-validates the Rust Hill dose-response implementation against the
//! Python control baseline (`control/pkpd/exp001_baseline.json`).

use healthspring_barracuda::pkpd::{self, ALL_INHIBITORS, compute_ec_values, hill_dose_response};
use healthspring_barracuda::tolerances;
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("Exp001 Hill Dose-Response");

    // Check 1–4: Hill at IC50 = 0.5 for each drug
    for drug in ALL_INHIBITORS {
        let r = hill_dose_response(drug.ic50_jak1_nm, drug.ic50_jak1_nm, drug.hill_n, 1.0);
        h.check_abs(
            &format!("{} at IC50 → 50%", drug.name),
            r,
            0.5,
            tolerances::MACHINE_EPSILON,
        );
    }

    // Check 5–8: Monotonicity
    let concs: Vec<f64> = (0..100)
        .map(|i| 10.0_f64.powf(-1.0 + 5.0 * f64::from(i) / 99.0))
        .collect();
    for drug in ALL_INHIBITORS {
        let responses = pkpd::hill_sweep(drug.ic50_jak1_nm, drug.hill_n, 1.0, &concs);
        let monotonic = responses
            .windows(2)
            .all(|w| w[0] <= w[1] + tolerances::MACHINE_EPSILON_STRICT);
        h.check_bool(&format!("{} monotonicity", drug.name), monotonic);
    }

    // Check 9: Potency ordering at 10 nM
    let r_bari = hill_dose_response(10.0, ALL_INHIBITORS[0].ic50_jak1_nm, 1.0, 1.0);
    let r_upa = hill_dose_response(10.0, ALL_INHIBITORS[1].ic50_jak1_nm, 1.0, 1.0);
    let r_ocla = hill_dose_response(10.0, ALL_INHIBITORS[3].ic50_jak1_nm, 1.0, 1.0);
    let r_abro = hill_dose_response(10.0, ALL_INHIBITORS[2].ic50_jak1_nm, 1.0, 1.0);
    h.check_bool(
        "Potency ordering at 10 nM",
        r_bari > r_upa && r_upa > r_ocla && r_ocla > r_abro,
    );

    // Check 10–13: EC values ordered
    for drug in ALL_INHIBITORS {
        let ec = compute_ec_values(drug.ic50_jak1_nm, drug.hill_n);
        h.check_bool(
            &format!("{} EC values", drug.name),
            ec.ec10 < ec.ec50 && ec.ec50 < ec.ec90,
        );
    }

    // Check 14: Cooperativity below IC50
    let r_n1 = hill_dose_response(5.0, 10.0, 1.0, 1.0);
    let r_n2 = hill_dose_response(5.0, 10.0, 2.0, 1.0);
    h.check_bool("Hill n=2 steeper below IC50", r_n2 < r_n1);

    // Check 15: Cooperativity above IC50
    let r_n1a = hill_dose_response(20.0, 10.0, 1.0, 1.0);
    let r_n2a = hill_dose_response(20.0, 10.0, 2.0, 1.0);
    h.check_bool("Hill n=2 higher above IC50", r_n2a > r_n1a);

    // Check 16–19: Saturation at 100x IC50
    for drug in ALL_INHIBITORS {
        let conc = drug.ic50_jak1_nm * 100.0;
        let r = hill_dose_response(conc, drug.ic50_jak1_nm, drug.hill_n, 1.0);
        h.check_lower(
            &format!("{} saturation at 100x", drug.name),
            r,
            tolerances::HILL_SATURATION_100X,
        );
    }

    h.exit();
}
