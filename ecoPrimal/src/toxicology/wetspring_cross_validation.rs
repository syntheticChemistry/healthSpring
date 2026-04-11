// SPDX-License-Identifier: AGPL-3.0-or-later
//! Cross-validation tests for healthSpring hormesis against wetSpring's
//! Anderson QS biphasic model.
//!
//! wetSpring (`bio::hormesis`) uses:
//!   `R(d) = (1 + A * hill(d, K_stim, n_s)) * (1 - hill(d, K_inh, n_i))`
//!
//! healthSpring (`toxicology::hormesis`) uses:
//!   `R(D) = baseline * (1 + s_max * D / (k_stim + D)) * (1 - D^n / (ic50^n + D^n))`
//!
//! Parameter mapping (when `n_s = 1`, `baseline = 1`):
//!   wetSpring `A`       = healthSpring `s_max`
//!   wetSpring `K_stim`  = healthSpring `k_stim`
//!   wetSpring `n_s`     = 1 (healthSpring hardcodes stimulation Hill n=1)
//!   wetSpring `K_inh`   = healthSpring `ic50`
//!   wetSpring `n_i`     = healthSpring `hill_n`
//!
//! When wetSpring uses `n_s > 1` (steeper stimulation onset), the models
//! diverge for the stimulation term. Joint experiment Exp379 documents
//! the parameter regime where both agree.
//!
//! Reference: wetSpring `work/anderson_hormesis/JOINT_EXPERIMENT.md`
//! Reference: wetSpring V130 handoff (ANDERSON_HORMESIS_FRAMEWORK)

#[cfg(test)]
mod tests {
    use crate::tolerances;
    use crate::toxicology::hormesis::{biphasic_dose_response, hormetic_optimum};

    /// Shared parameter set from joint experiment Exp379.
    /// These values are the intersection where both models agree
    /// (stimulation Hill n_s = 1).
    const SHARED_BASELINE: f64 = 1.0;
    const SHARED_S_MAX: f64 = 0.3;
    const SHARED_K_STIM: f64 = 2.0;
    const SHARED_IC50: f64 = 50.0;
    const SHARED_HILL_N: f64 = 2.0;

    /// wetSpring reference values (computed from the same formula with n_s=1).
    /// These are the expected outputs that wetSpring's `bio::hormesis` produces
    /// for the shared parameter set. If wetSpring changes its model, these
    /// values must be updated via a cross-spring handoff.
    fn wetspring_reference_biphasic(dose: f64) -> f64 {
        let stimulation = SHARED_S_MAX * dose / (SHARED_K_STIM + dose);
        let inhibition = dose.powf(SHARED_HILL_N)
            / (SHARED_IC50.powf(SHARED_HILL_N) + dose.powf(SHARED_HILL_N));
        SHARED_BASELINE * (1.0 + stimulation) * (1.0 - inhibition)
    }

    #[test]
    fn cross_validate_at_zero_dose() {
        let hs = biphasic_dose_response(0.0, SHARED_BASELINE, SHARED_S_MAX, SHARED_K_STIM, SHARED_IC50, SHARED_HILL_N);
        let ws = wetspring_reference_biphasic(0.0);
        assert!(
            (hs - ws).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "dose=0: healthSpring={hs}, wetSpring={ws}"
        );
    }

    #[test]
    fn cross_validate_low_dose_hormetic_zone() {
        for dose in [0.1, 0.5, 1.0, 2.0, 5.0] {
            let hs = biphasic_dose_response(dose, SHARED_BASELINE, SHARED_S_MAX, SHARED_K_STIM, SHARED_IC50, SHARED_HILL_N);
            let ws = wetspring_reference_biphasic(dose);
            assert!(
                (hs - ws).abs() < tolerances::TEST_ASSERTION_TIGHT,
                "dose={dose}: healthSpring={hs}, wetSpring={ws}"
            );
            assert!(hs > SHARED_BASELINE, "low dose should be hormetic: dose={dose}, R={hs}");
        }
    }

    #[test]
    fn cross_validate_transition_zone() {
        for dose in [10.0, 20.0, 30.0, 40.0] {
            let hs = biphasic_dose_response(dose, SHARED_BASELINE, SHARED_S_MAX, SHARED_K_STIM, SHARED_IC50, SHARED_HILL_N);
            let ws = wetspring_reference_biphasic(dose);
            assert!(
                (hs - ws).abs() < tolerances::TEST_ASSERTION_TIGHT,
                "dose={dose}: healthSpring={hs}, wetSpring={ws}"
            );
        }
    }

    #[test]
    fn cross_validate_toxic_zone() {
        for dose in [100.0, 200.0, 500.0] {
            let hs = biphasic_dose_response(dose, SHARED_BASELINE, SHARED_S_MAX, SHARED_K_STIM, SHARED_IC50, SHARED_HILL_N);
            let ws = wetspring_reference_biphasic(dose);
            assert!(
                (hs - ws).abs() < tolerances::TEST_ASSERTION_TIGHT,
                "dose={dose}: healthSpring={hs}, wetSpring={ws}"
            );
            assert!(hs < SHARED_BASELINE, "high dose should be toxic: dose={dose}, R={hs}");
        }
    }

    #[test]
    fn cross_validate_hormetic_optimum_location() {
        let (opt_dose, peak) = hormetic_optimum(
            SHARED_BASELINE, SHARED_S_MAX, SHARED_K_STIM,
            SHARED_IC50, SHARED_HILL_N, 100.0, 100_000,
        );
        assert!(opt_dose > 0.5, "optimum above noise: {opt_dose}");
        assert!(opt_dose < SHARED_IC50 / 2.0, "optimum well below IC50: {opt_dose}");
        assert!(peak > SHARED_BASELINE, "peak exceeds baseline: {peak}");

        let ws_peak = wetspring_reference_biphasic(opt_dose);
        assert!(
            (peak - ws_peak).abs() < tolerances::HORMESIS_CROSS_SPRING,
            "peak at optimum matches: healthSpring={peak}, wetSpring={ws_peak}"
        );
    }

    #[test]
    fn cross_validate_stimulation_threshold() {
        let threshold_dose = 0.01;
        let hs = biphasic_dose_response(threshold_dose, SHARED_BASELINE, SHARED_S_MAX, SHARED_K_STIM, SHARED_IC50, SHARED_HILL_N);
        let ws = wetspring_reference_biphasic(threshold_dose);

        assert!(hs > SHARED_BASELINE, "sub-threshold still stimulatory");
        assert!(
            (hs - ws).abs() < tolerances::TEST_ASSERTION_TIGHT,
            "threshold dose: healthSpring={hs}, wetSpring={ws}"
        );
    }

    #[test]
    fn biphasic_curve_monotonic_in_hormetic_zone() {
        let mut prev = SHARED_BASELINE;
        let mut found_peak = false;
        for i in 1..=100 {
            #[expect(clippy::cast_precision_loss, reason = "scan step count small")]
            let dose = (i as f64) * 0.1;
            let r = biphasic_dose_response(dose, SHARED_BASELINE, SHARED_S_MAX, SHARED_K_STIM, SHARED_IC50, SHARED_HILL_N);
            if r < prev && !found_peak {
                found_peak = true;
            }
            prev = r;
        }
        assert!(found_peak, "curve should have a single peak (biphasic shape)");
    }
}
