// Quick scenario JSON dump — run with: cargo run --bin exp050_scenario_dump
use healthspring_barracuda::diagnostic::{
    PatientProfile, Sex, assess_patient, population_montecarlo,
};
use healthspring_barracuda::visualization::full_scenario_json;

fn main() {
    let mut patient = PatientProfile::minimal(55.0, 85.0, Sex::Male);
    patient.testosterone_ng_dl = Some(450.0);
    patient.on_trt = true;
    patient.trt_months = 12.0;
    patient.gut_abundances = Some(vec![0.30, 0.25, 0.18, 0.12, 0.08, 0.04, 0.03]);

    let assessment = assess_patient(&patient);
    let pop = population_montecarlo(&patient, 1000, 42);
    print!(
        "{}",
        full_scenario_json(&assessment, &pop, "Male 55y, TRT 12mo")
    );
}
