// SPDX-License-Identifier: AGPL-3.0-or-later
//! ODE system implementations for barraCuda's generic batched RK4 solver.
//!
//! Each struct implements `barracuda::numerical::OdeSystem`, providing:
//! - A WGSL derivative for GPU batch integration
//! - A CPU derivative for validation and small batches
//! - Metadata (`system_name`, `N_VARS`, `N_PARAMS`)
//!
//! These replace the hand-rolled WGSL shaders in `dispatch/` with the
//! generic `BatchedOdeRK4::generate_shader()` path, aligning healthSpring
//! with the ecosystem ODE codegen pattern from wetSpring.

use barracuda::numerical::OdeSystem;

/// Michaelis-Menten (capacity-limited) elimination PK.
///
/// State: `[C]` (drug concentration, mg/L)
/// Params: `[Vmax, Km, Vd]`
///
/// ```text
/// dC/dt = -Vmax * C / (Vd * (Km + C))
/// ```
///
/// Used for phenytoin, ethanol, and other saturable-metabolism drugs.
pub struct MichaelisMentenOde;

impl OdeSystem for MichaelisMentenOde {
    const N_VARS: usize = 1;
    const N_PARAMS: usize = 3;

    fn system_name() -> &'static str {
        "michaelis_menten_pk"
    }

    fn wgsl_derivative() -> &'static str {
        "
fn fmax_d(a: f64, b: f64) -> f64 {
    if (a >= b) { return a; }
    return b;
}
fn deriv(state: array<f64, 1>, params: array<f64, 3>, t: f64) -> array<f64, 1> {
    let vmax = params[0];
    let km   = params[1];
    let vd   = params[2];
    let c    = fmax_d(state[0], 0.0);
    var dy: array<f64, 1>;
    dy[0] = -(vmax * c) / (vd * (km + c + 1e-30));
    return dy;
}
"
    }

    fn cpu_derivative(_t: f64, state: &[f64], params: &[f64]) -> Vec<f64> {
        let c = state[0].max(0.0);
        let vmax = params[0];
        let km = params[1];
        let vd = params[2];
        vec![-(vmax * c) / (vd * (km + c + 1e-30))]
    }
}

/// Oral one-compartment first-order PK.
///
/// State: `[A_gut, C_plasma]` (gut amount mg, plasma concentration mg/L)
/// Params: `[dose, F, Vd, Ka, Ke]`
///
/// ```text
/// dA_gut/dt   = -Ka * A_gut
/// dC_plasma/dt = (Ka * A_gut * F / Vd) - Ke * C_plasma
/// ```
///
/// Standard model for oral dosing with first-order absorption and elimination.
pub struct OralOneCompartmentOde;

impl OdeSystem for OralOneCompartmentOde {
    const N_VARS: usize = 2;
    const N_PARAMS: usize = 5;

    fn system_name() -> &'static str {
        "oral_one_compartment_pk"
    }

    fn wgsl_derivative() -> &'static str {
        "
fn fmax_d(a: f64, b: f64) -> f64 {
    if (a >= b) { return a; }
    return b;
}
fn deriv(state: array<f64, 2>, params: array<f64, 5>, t: f64) -> array<f64, 2> {
    let f_bio = params[1];
    let vd    = params[2];
    let ka    = params[3];
    let ke    = params[4];
    let a_gut = fmax_d(state[0], 0.0);
    let c_p   = fmax_d(state[1], 0.0);
    var dy: array<f64, 2>;
    dy[0] = -ka * a_gut;
    dy[1] = (ka * a_gut * f_bio / (vd + 1e-30)) - ke * c_p;
    return dy;
}
"
    }

    fn cpu_derivative(_t: f64, state: &[f64], params: &[f64]) -> Vec<f64> {
        let f_bio = params[1];
        let vd = params[2];
        let ka = params[3];
        let ke = params[4];
        let a_gut = state[0].max(0.0);
        let c_p = state[1].max(0.0);
        vec![
            -ka * a_gut,
            ka.mul_add(a_gut * f_bio / (vd + 1e-30), -ke * c_p),
        ]
    }
}

/// IV two-compartment PK.
///
/// State: `[C1, C2]` (central concentration, peripheral concentration)
/// Params: `[k10, k12, k21, V1]`
///
/// ```text
/// dC1/dt = -k10*C1 - k12*C1 + k21*C2
/// dC2/dt =  k12*C1 - k21*C2
/// ```
pub struct TwoCompartmentOde;

impl OdeSystem for TwoCompartmentOde {
    const N_VARS: usize = 2;
    const N_PARAMS: usize = 4;

    fn system_name() -> &'static str {
        "two_compartment_pk"
    }

    fn wgsl_derivative() -> &'static str {
        "
fn deriv(state: array<f64, 2>, params: array<f64, 4>, t: f64) -> array<f64, 2> {
    let k10 = params[0];
    let k12 = params[1];
    let k21 = params[2];
    let c1  = state[0];
    let c2  = state[1];
    var dy: array<f64, 2>;
    dy[0] = -(k10 + k12) * c1 + k21 * c2;
    dy[1] = k12 * c1 - k21 * c2;
    return dy;
}
"
    }

    fn cpu_derivative(_t: f64, state: &[f64], params: &[f64]) -> Vec<f64> {
        let k10 = params[0];
        let k12 = params[1];
        let k21 = params[2];
        let c1 = state[0];
        let c2 = state[1];
        vec![
            k21.mul_add(c2, -(k10 + k12) * c1),
            k12.mul_add(c1, -k21 * c2),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use barracuda::numerical::BatchedOdeRK4;

    #[test]
    fn mm_cpu_integration_decays() {
        let initial = vec![25.0];
        let params = vec![10.0, 4.0, 50.0];
        let result =
            BatchedOdeRK4::<MichaelisMentenOde>::integrate_cpu(&initial, &params, 0.1, 1000, 1)
                .unwrap();
        assert!(result[0] < 25.0, "concentration should decrease");
        assert!(result[0] >= 0.0, "concentration must stay non-negative");
    }

    #[test]
    fn mm_shader_generation() {
        let wgsl = BatchedOdeRK4::<MichaelisMentenOde>::generate_shader();
        assert!(
            wgsl.contains("fn deriv"),
            "shader should include derivative"
        );
    }

    #[test]
    fn oral_1comp_cpu_absorbs_then_eliminates() {
        let initial = vec![100.0, 0.0];
        let params = vec![100.0, 0.8, 50.0, 1.0, 0.1];
        let result =
            BatchedOdeRK4::<OralOneCompartmentOde>::integrate_cpu(&initial, &params, 0.01, 5000, 1)
                .unwrap();
        assert!(result[0] < 1.0, "gut should be nearly empty: {}", result[0]);
        assert!(result[1] >= 0.0, "plasma stays non-negative");
    }

    #[test]
    fn oral_1comp_shader_generation() {
        let wgsl = BatchedOdeRK4::<OralOneCompartmentOde>::generate_shader();
        assert!(wgsl.contains("fn deriv"));
    }

    #[test]
    fn two_comp_mass_balance() {
        let c0 = 10.0;
        let initial = vec![c0, 0.0];
        let params = vec![0.1, 0.3, 0.2, 5.0];
        let result =
            BatchedOdeRK4::<TwoCompartmentOde>::integrate_cpu(&initial, &params, 0.01, 10_000, 1)
                .unwrap();
        assert!(result[0] < c0, "central should decrease");
        assert!(result[0] >= 0.0 && result[1] >= 0.0);
    }

    #[test]
    fn two_comp_shader_generation() {
        let wgsl = BatchedOdeRK4::<TwoCompartmentOde>::generate_shader();
        assert!(wgsl.contains("fn deriv"));
    }

    #[test]
    fn mm_batch_integration() {
        let initial = vec![25.0, 10.0];
        let params = vec![10.0, 4.0, 50.0, 10.0, 4.0, 50.0];
        let result =
            BatchedOdeRK4::<MichaelisMentenOde>::integrate_cpu(&initial, &params, 0.1, 500, 2)
                .unwrap();
        assert!(result[0] < 25.0);
        assert!(result[1] < 10.0);
        assert!(result[0] > result[1], "higher initial → higher final");
    }
}
