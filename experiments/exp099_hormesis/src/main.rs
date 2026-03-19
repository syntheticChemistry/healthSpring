// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]
//! Exp099: Hormesis — biphasic dose-response across domains.
//!
//! Computational evidence that mild stress improves fitness across
//! every biological domain: pesticides/ecology (groundSpring/airSpring),
//! immune calibration (hygiene hypothesis), caloric restriction,
//! and poison tolerance (mithridatism). All share the same biphasic
//! shape and the same Anderson localization transition.
//!
//! Six studies:
//! 1. Biphasic dose-response: the hormetic curve
//! 2. Ecological hormesis: weak pesticide → more grasshoppers
//! 3. Mithridatism: self-dosing builds tolerance at a cost
//! 4. Hygiene hypothesis: microbial exposure calibrates immunity
//! 5. Caloric restriction: mild hunger → longevity
//! 6. Anderson transition: hormetic→toxic is delocalized→localized

use healthspring_barracuda::tolerances;
use healthspring_barracuda::toxicology;
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("exp099_hormesis");

    study_1_biphasic_curve(&mut h);
    study_2_ecological_hormesis(&mut h);
    study_3_mithridatism(&mut h);
    study_4_hygiene_hypothesis(&mut h);
    study_5_caloric_restriction(&mut h);
    study_6_anderson_transition(&mut h);

    h.exit();
}

/// Study 1: The biphasic dose-response curve.
///
/// Verify the fundamental shape: baseline → peak → decline → zero.
fn study_1_biphasic_curve(h: &mut ValidationHarness) {
    println!("\n─── Study 1: The Biphasic Dose-Response Curve ───");

    let baseline = 100.0;
    let s_max = 0.5;
    let k_stim = 1.0;
    let ic50 = 50.0;
    let hill_n = 2.0;

    let r_zero = toxicology::biphasic_dose_response(0.0, baseline, s_max, k_stim, ic50, hill_n);
    let r_low = toxicology::biphasic_dose_response(1.0, baseline, s_max, k_stim, ic50, hill_n);
    let r_ic50 = toxicology::biphasic_dose_response(50.0, baseline, s_max, k_stim, ic50, hill_n);
    let r_high = toxicology::biphasic_dose_response(200.0, baseline, s_max, k_stim, ic50, hill_n);

    println!("  Dose =   0: fitness = {r_zero:.1} (baseline)");
    println!("  Dose =   1: fitness = {r_low:.1} (hormetic zone)");
    println!("  Dose =  50: fitness = {r_ic50:.1} (transition)");
    println!("  Dose = 200: fitness = {r_high:.2} (toxic)");

    h.check_abs(
        "dose=0 → baseline",
        r_zero,
        baseline,
        tolerances::MACHINE_EPSILON,
    );
    h.check_lower("low dose exceeds baseline (hormesis)", r_low, baseline);
    h.check_upper("high dose below baseline (toxicity)", r_high, baseline);

    let (opt_dose, peak) =
        toxicology::hormetic_optimum(baseline, s_max, k_stim, ic50, hill_n, 100.0, 10000);
    let gain_pct = (peak / baseline - 1.0) * 100.0;
    println!("  Hormetic optimum: dose={opt_dose:.2}, peak={peak:.1} ({gain_pct:.1}% gain)");

    h.check_lower("hormetic peak > baseline", peak, baseline);
    h.check_lower("optimal dose > 0", opt_dose, 0.0);
    h.check_upper("optimal dose < IC50", opt_dose, ic50);

    println!("  → Classic inverted-U shape: stress helps, then hurts");
}

/// Study 2: Ecological hormesis — weak pesticide makes more grasshoppers.
///
/// This is the non-intuitive result: a low pesticide concentration can
/// increase pest populations through direct stress-response stimulation.
fn study_2_ecological_hormesis(h: &mut ValidationHarness) {
    println!("\n─── Study 2: Ecological Hormesis (Grasshoppers) ───");

    let baseline_pop = 10_000.0;
    let stress_gain = 0.4;
    let k_stress = 0.5;
    let lethal_ic50 = 20.0;
    let hill_n = 2.0;

    let doses = [0.0, 0.1, 0.5, 1.0, 5.0, 15.0, 30.0, 50.0];
    println!("  Pesticide dose → grasshopper population:");
    for &d in &doses {
        let pop = toxicology::ecological_hormesis(
            d,
            baseline_pop,
            stress_gain,
            k_stress,
            lethal_ic50,
            hill_n,
        );
        let change = (pop / baseline_pop - 1.0) * 100.0;
        let marker = if pop > baseline_pop {
            "↑"
        } else if pop < baseline_pop {
            "↓"
        } else {
            "="
        };
        println!("    dose={d:5.1} → pop={pop:8.0} ({change:+.1}%) {marker}");
    }

    let pop_weak = toxicology::ecological_hormesis(
        0.5,
        baseline_pop,
        stress_gain,
        k_stress,
        lethal_ic50,
        hill_n,
    );
    let pop_strong = toxicology::ecological_hormesis(
        30.0,
        baseline_pop,
        stress_gain,
        k_stress,
        lethal_ic50,
        hill_n,
    );

    h.check_lower(
        "weak pesticide: population INCREASES",
        pop_weak,
        baseline_pop,
    );
    h.check_upper(
        "strong pesticide: population decreases",
        pop_strong,
        baseline_pop,
    );

    let (opt_dose, peak_pop) = toxicology::hormetic_optimum(
        baseline_pop,
        stress_gain,
        k_stress,
        lethal_ic50,
        hill_n,
        50.0,
        10000,
    );
    let extra_pct = (peak_pop / baseline_pop - 1.0) * 100.0;
    println!(
        "  Peak population at dose={opt_dose:.2}: {peak_pop:.0} ({extra_pct:.1}% above baseline)"
    );

    h.check_lower(
        "peak population exceeds baseline (hormesis creates more pests)",
        peak_pop,
        baseline_pop,
    );

    println!("  → A weak pesticide can make the pest problem WORSE");
    println!("  → groundSpring/airSpring: predict this before spraying");
}

/// Study 3: Mithridatism — building poison tolerance at a cost.
///
/// Can you actually make yourself immune to poison by self-dosing?
/// Yes — but it costs you baseline fitness.
fn study_3_mithridatism(h: &mut ValidationHarness) {
    println!("\n─── Study 3: Mithridatism ───");

    let params = toxicology::MithridatismParams {
        ic50_naive: 10.0,
        max_adaptation: 5.0,
        k_adapt: 15.0,
        max_cost: 0.12,
        k_cost: 25.0,
    };

    let baseline = 100.0;
    let s_max = 0.3;
    let k_stim = 1.0;
    let hill_n = 2.0;

    println!("  Exposures → adapted IC50, cost, fitness at lethal dose (D=20):");
    let exposures = [0.0, 5.0, 15.0, 30.0, 50.0, 100.0];
    for &n in &exposures {
        let (ic50_a, cost) = toxicology::mithridatism_adaptation(&params, n);
        let fitness =
            toxicology::mithridatism_fitness(20.0, baseline, s_max, k_stim, hill_n, &params, n);
        let naive_fitness = toxicology::biphasic_dose_response(
            20.0,
            baseline,
            s_max,
            k_stim,
            params.ic50_naive,
            hill_n,
        );
        let survival = if fitness > naive_fitness {
            "SURVIVES"
        } else {
            "dies"
        };
        println!(
            "    n={n:5.0}: IC50={ic50_a:5.1}, cost={:.1}%, fitness={fitness:6.1} ({survival}) vs naive={naive_fitness:.1}",
            cost * 100.0
        );
    }

    let (ic50_naive, _) = toxicology::mithridatism_adaptation(&params, 0.0);
    let (ic50_adapted, cost_adapted) = toxicology::mithridatism_adaptation(&params, 50.0);

    h.check_abs(
        "naive IC50 = 10.0",
        ic50_naive,
        10.0,
        tolerances::MACHINE_EPSILON,
    );
    h.check_lower("50 exposures raises IC50", ic50_adapted, ic50_naive);
    h.check_lower("adaptation has metabolic cost > 5%", cost_adapted, 0.05);
    h.check_upper("cost bounded by max (12%)", cost_adapted, 0.12);

    let lethal_dose = 20.0;
    let naive = toxicology::biphasic_dose_response(
        lethal_dose,
        baseline,
        s_max,
        k_stim,
        params.ic50_naive,
        hill_n,
    );
    let adapted = toxicology::mithridatism_fitness(
        lethal_dose,
        baseline,
        s_max,
        k_stim,
        hill_n,
        &params,
        50.0,
    );

    h.check_lower(
        "adapted organism survives dose that harms naive",
        adapted,
        naive,
    );

    let resting_naive =
        toxicology::biphasic_dose_response(0.0, baseline, s_max, k_stim, params.ic50_naive, hill_n);
    let resting_adapted =
        toxicology::mithridatism_fitness(0.0, baseline, s_max, k_stim, hill_n, &params, 50.0);
    println!("\n  At rest (dose=0): naive={resting_naive:.1}, adapted={resting_adapted:.1}");
    h.check_upper(
        "adaptation costs resting fitness (the price of immunity)",
        resting_adapted,
        resting_naive,
    );

    println!("  → Yes, you can build tolerance. But it costs you.");
    println!(
        "  → Mithridates survived, but he was never as healthy as an unexposed person at rest."
    );
}

/// Study 4: Hygiene hypothesis — immune calibration through exposure.
///
/// Too little microbial exposure → uncalibrated immune system → allergies.
/// Moderate exposure → properly calibrated → robust.
/// Excessive exposure → overwhelmed → infection.
fn study_4_hygiene_hypothesis(h: &mut ValidationHarness) {
    println!("\n─── Study 4: Hygiene Hypothesis ───");

    let baseline_competence = 0.3;
    let calibration_gain = 2.0;
    let k_cal = 0.5;
    let overwhelm_ic50 = 100.0;
    let hill_n = 2.0;

    println!("  Microbial exposure → immune competence:");
    let exposures = [0.0, 0.01, 0.1, 0.5, 2.0, 10.0, 50.0, 200.0, 500.0];
    for &e in &exposures {
        let ic = toxicology::immune_calibration(
            e,
            baseline_competence,
            calibration_gain,
            k_cal,
            overwhelm_ic50,
            hill_n,
        );
        let status = if ic < 0.5 {
            "uncalibrated"
        } else if ic > 0.8 {
            "well-calibrated"
        } else {
            "moderate"
        };
        println!("    exposure={e:6.2} → competence={ic:.3} ({status})");
    }

    let sterile = toxicology::immune_calibration(
        0.0,
        baseline_competence,
        calibration_gain,
        k_cal,
        overwhelm_ic50,
        hill_n,
    );
    let farm_kid = toxicology::immune_calibration(
        5.0,
        baseline_competence,
        calibration_gain,
        k_cal,
        overwhelm_ic50,
        hill_n,
    );
    let overwhelmed = toxicology::immune_calibration(
        500.0,
        baseline_competence,
        calibration_gain,
        k_cal,
        overwhelm_ic50,
        hill_n,
    );

    h.check_lower(
        "moderate exposure > sterile (farm kids have fewer allergies)",
        farm_kid,
        sterile,
    );
    h.check_lower(
        "moderate exposure > overwhelming (too much causes infection)",
        farm_kid,
        overwhelmed,
    );
    h.check_upper(
        "sterile environment: poor immune competence (< 0.5)",
        sterile,
        0.5,
    );

    println!("\n  → Peanut allergies in toddlers: early exposure = calibration");
    println!("  → LEAP study (Du Toit 2015): early peanut → 81% reduction in allergy");
    println!("  → The immune system needs training data, just like a model");
}

/// Study 5: Caloric restriction — mild hunger extends lifespan.
fn study_5_caloric_restriction(h: &mut ValidationHarness) {
    println!("\n─── Study 5: Caloric Restriction ───");

    let baseline_lifespan = 80.0;
    let longevity_gain = 0.3;
    let k_autophagy = 0.15;
    let starvation_ic50 = 0.70;
    let hill_n = 3.0;

    println!("  Caloric restriction → predicted lifespan:");
    let restrictions = [0.0, 0.05, 0.10, 0.15, 0.20, 0.30, 0.50, 0.70, 0.90];
    for &r in &restrictions {
        let lifespan = toxicology::caloric_restriction_fitness(
            r,
            baseline_lifespan,
            longevity_gain,
            k_autophagy,
            starvation_ic50,
            hill_n,
        );
        let pct_change = (lifespan / baseline_lifespan - 1.0) * 100.0;
        let label = match r {
            x if x < 0.01 => "ad libitum",
            x if x < 0.25 => "mild CR",
            x if x < 0.50 => "moderate CR",
            _ => "severe deficit",
        };
        println!(
            "    restriction={:.0}% → lifespan={lifespan:.1} yr ({pct_change:+.1}%) [{label}]",
            r * 100.0
        );
    }

    let ad_lib = toxicology::caloric_restriction_fitness(
        0.0,
        baseline_lifespan,
        longevity_gain,
        k_autophagy,
        starvation_ic50,
        hill_n,
    );
    let mild_cr = toxicology::caloric_restriction_fitness(
        0.15,
        baseline_lifespan,
        longevity_gain,
        k_autophagy,
        starvation_ic50,
        hill_n,
    );
    let severe = toxicology::caloric_restriction_fitness(
        0.85,
        baseline_lifespan,
        longevity_gain,
        k_autophagy,
        starvation_ic50,
        hill_n,
    );

    let (opt_cr, peak_lifespan) = toxicology::hormetic_optimum(
        baseline_lifespan,
        longevity_gain,
        k_autophagy,
        starvation_ic50,
        hill_n,
        0.9,
        10000,
    );

    println!(
        "\n  Optimal CR: {:.0}% restriction → {peak_lifespan:.1} years ({:.1}% gain)",
        opt_cr * 100.0,
        (peak_lifespan / baseline_lifespan - 1.0) * 100.0
    );

    h.check_abs(
        "ad libitum → baseline",
        ad_lib,
        baseline_lifespan,
        tolerances::MACHINE_EPSILON,
    );
    h.check_lower("mild CR extends lifespan", mild_cr, baseline_lifespan);
    h.check_upper("severe deficit shortens life", severe, baseline_lifespan);
    h.check_lower("optimal CR > 0%", opt_cr, 0.0);
    h.check_upper("optimal CR < 50%", opt_cr, 0.50);

    println!("  → Mattison 2017: rhesus monkeys on 30% CR lived ~3 years longer");
    println!("  → Fontana 2015: autophagy, sirtuins, mitochondrial efficiency");
    println!("  → The same biphasic curve that governs pesticide hormesis governs aging");
}

/// Study 6: Anderson localization transition — hormetic→toxic.
///
/// The transition from beneficial stress to harmful toxicity IS an
/// Anderson localization transition. At low doses, the stress response
/// is delocalized (spread across repair pathways). At high doses,
/// damage localizes at vulnerable sites.
fn study_6_anderson_transition(h: &mut ValidationHarness) {
    println!("\n─── Study 6: Anderson Localization Transition ───");

    let loc_params = toxicology::HormesisLocalizationParams {
        baseline: 100.0,
        s_max: 0.5,
        k_stim: 1.0,
        ic50: 50.0,
        hill_n: 2.0,
        n_pathways: 12,
        disorder_w: 0.6,
        seed: 42,
    };

    println!("  Dose → IPR → interpretation:");
    let doses = [0.1, 0.5, 1.0, 5.0, 15.0, 30.0, 60.0, 100.0];
    for &d in &doses {
        let (ipr, interp) = toxicology::hormesis_localization(d, &loc_params);
        let fitness = toxicology::biphasic_dose_response(
            d,
            loc_params.baseline,
            loc_params.s_max,
            loc_params.k_stim,
            loc_params.ic50,
            loc_params.hill_n,
        );
        println!("    dose={d:5.1} → IPR={ipr:.4}, fitness={fitness:6.1} → {interp}");
    }

    let (ipr_low, interp_low) = toxicology::hormesis_localization(0.5, &loc_params);
    let (_, interp_high) = toxicology::hormesis_localization(80.0, &loc_params);

    h.check_upper(
        "low dose: delocalized stress (IPR < 0.15)",
        ipr_low,
        tolerances::TOX_IPR_DELOCALIZED,
    );
    h.check_bool(
        "low dose interpreted as hormetic",
        interp_low.contains("hormetic"),
    );
    h.check_bool(
        "high dose interpreted as toxic or declining",
        interp_high.contains("toxic") || interp_high.contains("declining"),
    );

    let fitness_low = toxicology::biphasic_dose_response(
        0.5,
        loc_params.baseline,
        loc_params.s_max,
        loc_params.k_stim,
        loc_params.ic50,
        loc_params.hill_n,
    );
    let fitness_high = toxicology::biphasic_dose_response(
        80.0,
        loc_params.baseline,
        loc_params.s_max,
        loc_params.k_stim,
        loc_params.ic50,
        loc_params.hill_n,
    );
    h.check_lower(
        "hormetic dose has higher fitness than toxic dose",
        fitness_low,
        fitness_high,
    );

    println!("\n  → The hormetic-to-toxic transition is an Anderson localization transition");
    println!("  → Delocalized stress: many repair pathways share the load (beneficial)");
    println!("  → Localized damage: vulnerable sites overwhelmed (harmful)");
    println!("  → Same physics: pesticides, poisons, caloric restriction, microbes");
    println!("  → groundSpring (soil/plants) + airSpring (environmental dispersal) + ");
    println!("    healthSpring (human health) + wetSpring (microbiome) — all one curve");
}
