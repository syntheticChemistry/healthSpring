// SPDX-License-Identifier: AGPL-3.0-or-later
//! Baseline provenance tracking for validation targets.
//!
//! Every hardcoded expected value in a validation binary should trace
//! back to a documented Python run with [`BaselineProvenance`] or to
//! a published formula with [`AnalyticalProvenance`].
//!
//! Follows the hotSpring provenance pattern. healthSpring Python baselines
//! store provenance in `_provenance` JSON objects (date, script, command,
//! `git_commit`, python, numpy). This module provides Rust-side types to
//! consume and validate that provenance chain.
//!
//! The [`PROVENANCE_REGISTRY`] enumerates all Python control scripts in
//! `control/` for completeness verification.

use serde::{Deserialize, Serialize};

/// Record for a Python baseline control script in the provenance registry.
#[derive(Debug, Clone)]
pub struct ProvenanceRecord {
    /// Track: pkpd, microbiome, biosignal, endocrine, validation, comparative, discovery, scripts.
    pub track: &'static str,
    /// Experiment identifier (e.g. `exp001`, `exp010`, `cross_validate`).
    pub experiment: &'static str,
    /// Path to the Python script relative to repo root.
    pub python_script: &'static str,
    /// Minimum Python version (e.g. 3.11+).
    pub python_version: &'static str,
    /// One-line description of the script.
    pub description: &'static str,
    /// Number of validation checks in the script (0 if not applicable).
    pub checks: u32,
}

/// Registry of all Python control scripts in `control/`.
///
/// Used to verify that every `.py` file in `control/` has a corresponding
/// entry. Run `cargo test registry_complete` to validate.
pub const PROVENANCE_REGISTRY: &[ProvenanceRecord] = &[
    // Track 1: PK/PD
    ProvenanceRecord {
        track: "pkpd",
        experiment: "exp001",
        python_script: "control/pkpd/exp001_hill_dose_response.py",
        python_version: "3.10+",
        description: "Hill dose-response 4-parameter IC50/EC50 for JAK inhibitors",
        checks: 8,
    },
    ProvenanceRecord {
        track: "pkpd",
        experiment: "exp002",
        python_script: "control/pkpd/exp002_one_compartment_pk.py",
        python_version: "3.10+",
        description: "One-compartment PK: IV bolus, oral absorption, AUC, Cmax, Tmax",
        checks: 0,
    },
    ProvenanceRecord {
        track: "pkpd",
        experiment: "exp003",
        python_script: "control/pkpd/exp003_two_compartment_pk.py",
        python_version: "3.10+",
        description: "Two-compartment PK model (IV bolus)",
        checks: 0,
    },
    ProvenanceRecord {
        track: "pkpd",
        experiment: "exp004",
        python_script: "control/pkpd/exp004_mab_pk_transfer.py",
        python_version: "3.10+",
        description: "mAb PK cross-species transfer (lokivetmab → nemolizumab/dupilumab)",
        checks: 0,
    },
    ProvenanceRecord {
        track: "pkpd",
        experiment: "exp005",
        python_script: "control/pkpd/exp005_population_pk.py",
        python_version: "3.10+",
        description: "Population PK Monte Carlo with IIV",
        checks: 0,
    },
    ProvenanceRecord {
        track: "pkpd",
        experiment: "exp006",
        python_script: "control/pkpd/exp006_pbpk_compartments.py",
        python_version: "3.10+",
        description: "PBPK 5-tissue compartments (liver, kidney, muscle, fat, rest)",
        checks: 0,
    },
    ProvenanceRecord {
        track: "pkpd",
        experiment: "exp077",
        python_script: "control/pkpd/exp077_michaelis_menten_pk.py",
        python_version: "3.10+",
        description: "Michaelis-Menten nonlinear PK (phenytoin-like)",
        checks: 0,
    },
    ProvenanceRecord {
        track: "pkpd",
        experiment: "cross_validate",
        python_script: "control/pkpd/cross_validate.py",
        python_version: "3.10+",
        description: "Cross-validation: Python baseline JSON self-consistency",
        checks: 0,
    },
    // Track 2: Microbiome
    ProvenanceRecord {
        track: "microbiome",
        experiment: "exp010",
        python_script: "control/microbiome/exp010_diversity_indices.py",
        python_version: "3.10+",
        description: "Shannon, Simpson, Pielou, Chao1 diversity indices",
        checks: 0,
    },
    ProvenanceRecord {
        track: "microbiome",
        experiment: "exp011",
        python_script: "control/microbiome/exp011_anderson_gut_lattice.py",
        python_version: "3.10+",
        description: "Anderson localization in 1D gut microbiome lattice",
        checks: 0,
    },
    ProvenanceRecord {
        track: "microbiome",
        experiment: "exp012",
        python_script: "control/microbiome/exp012_cdiff_resistance.py",
        python_version: "3.10+",
        description: "C. difficile colonization resistance score",
        checks: 0,
    },
    ProvenanceRecord {
        track: "microbiome",
        experiment: "exp013",
        python_script: "control/microbiome/exp013_fmt_rcdi.py",
        python_version: "3.10+",
        description: "FMT for recurrent C. diff infection (rCDI)",
        checks: 0,
    },
    ProvenanceRecord {
        track: "microbiome",
        experiment: "exp078",
        python_script: "control/microbiome/exp078_antibiotic_perturbation.py",
        python_version: "3.10+",
        description: "Antibiotic perturbation recovery (Shannon diversity)",
        checks: 0,
    },
    ProvenanceRecord {
        track: "microbiome",
        experiment: "exp079",
        python_script: "control/microbiome/exp079_scfa_production.py",
        python_version: "3.10+",
        description: "SCFA production (Michaelis-Menten fermentation)",
        checks: 0,
    },
    ProvenanceRecord {
        track: "microbiome",
        experiment: "exp080",
        python_script: "control/microbiome/exp080_gut_brain_serotonin.py",
        python_version: "3.10+",
        description: "Gut-brain serotonin pathway (diversity → tryptophan)",
        checks: 0,
    },
    // Track 3: Biosignal
    ProvenanceRecord {
        track: "biosignal",
        experiment: "exp020",
        python_script: "control/biosignal/exp020_pan_tompkins_qrs.py",
        python_version: "3.10+",
        description: "Pan-Tompkins QRS detection in ECG",
        checks: 0,
    },
    ProvenanceRecord {
        track: "biosignal",
        experiment: "exp021",
        python_script: "control/biosignal/exp021_hrv_metrics.py",
        python_version: "3.10+",
        description: "HRV metrics (RMSSD, pNN50, SDNN)",
        checks: 0,
    },
    ProvenanceRecord {
        track: "biosignal",
        experiment: "exp022",
        python_script: "control/biosignal/exp022_ppg_spo2.py",
        python_version: "3.10+",
        description: "PPG SpO2 R-value calibration",
        checks: 0,
    },
    ProvenanceRecord {
        track: "biosignal",
        experiment: "exp023",
        python_script: "control/biosignal/exp023_fusion.py",
        python_version: "3.10+",
        description: "Multi-channel biosignal fusion (ECG + PPG + EDA)",
        checks: 0,
    },
    ProvenanceRecord {
        track: "biosignal",
        experiment: "exp081",
        python_script: "control/biosignal/exp081_eda_stress.py",
        python_version: "3.10+",
        description: "EDA autonomic stress detection",
        checks: 0,
    },
    ProvenanceRecord {
        track: "biosignal",
        experiment: "exp082",
        python_script: "control/biosignal/exp082_arrhythmia_classification.py",
        python_version: "3.10+",
        description: "Arrhythmia beat classification (Normal, PVC, PAC)",
        checks: 0,
    },
    // Track 4: Endocrine
    ProvenanceRecord {
        track: "endocrine",
        experiment: "exp030",
        python_script: "control/endocrine/exp030_testosterone_im_pk.py",
        python_version: "3.10+",
        description: "Testosterone IM injection PK (cypionate depot)",
        checks: 0,
    },
    ProvenanceRecord {
        track: "endocrine",
        experiment: "exp031",
        python_script: "control/endocrine/exp031_testosterone_pellet_pk.py",
        python_version: "3.10+",
        description: "Testosterone pellet depot PK (zero-order release)",
        checks: 0,
    },
    ProvenanceRecord {
        track: "endocrine",
        experiment: "exp032",
        python_script: "control/endocrine/exp032_age_testosterone_decline.py",
        python_version: "3.10+",
        description: "Age-related testosterone decline (Harman 2001)",
        checks: 0,
    },
    ProvenanceRecord {
        track: "endocrine",
        experiment: "exp033",
        python_script: "control/endocrine/exp033_trt_weight_trajectory.py",
        python_version: "3.10+",
        description: "TRT metabolic response: weight/waist trajectory",
        checks: 0,
    },
    ProvenanceRecord {
        track: "endocrine",
        experiment: "exp034",
        python_script: "control/endocrine/exp034_trt_cardiovascular.py",
        python_version: "3.10+",
        description: "TRT cardiovascular response (LDL, HDL, CRP, BP)",
        checks: 0,
    },
    ProvenanceRecord {
        track: "endocrine",
        experiment: "exp035",
        python_script: "control/endocrine/exp035_trt_diabetes.py",
        python_version: "3.10+",
        description: "TRT and Type 2 diabetes (HbA1c, HOMA-IR)",
        checks: 0,
    },
    ProvenanceRecord {
        track: "endocrine",
        experiment: "exp036",
        python_script: "control/endocrine/exp036_population_trt_montecarlo.py",
        python_version: "3.10+",
        description: "Population TRT Monte Carlo (10K virtual patients)",
        checks: 0,
    },
    ProvenanceRecord {
        track: "endocrine",
        experiment: "exp037",
        python_script: "control/endocrine/exp037_testosterone_gut_axis.py",
        python_version: "3.10+",
        description: "Testosterone-gut axis: microbiome stratification",
        checks: 0,
    },
    ProvenanceRecord {
        track: "endocrine",
        experiment: "exp038",
        python_script: "control/endocrine/exp038_hrv_trt_cardiovascular.py",
        python_version: "3.10+",
        description: "HRV × TRT cardiovascular cross-track",
        checks: 0,
    },
    // Track 5: Comparative
    ProvenanceRecord {
        track: "comparative",
        experiment: "exp100",
        python_script: "control/comparative/exp100_canine_il31.py",
        python_version: "3.10+",
        description: "Canine IL-31 serum kinetics in atopic dermatitis",
        checks: 0,
    },
    ProvenanceRecord {
        track: "comparative",
        experiment: "exp101",
        python_script: "control/comparative/exp101_canine_jak1.py",
        python_version: "3.10+",
        description: "Canine oclacitinib JAK1 selectivity validation",
        checks: 0,
    },
    ProvenanceRecord {
        track: "comparative",
        experiment: "exp102",
        python_script: "control/comparative/exp102_il31_pruritus_timecourse.py",
        python_version: "3.10+",
        description: "IL-31 pruritus time-course baseline",
        checks: 0,
    },
    ProvenanceRecord {
        track: "comparative",
        experiment: "exp103",
        python_script: "control/comparative/exp103_lokivetmab_duration.py",
        python_version: "3.10+",
        description: "Lokivetmab dose-duration baseline",
        checks: 0,
    },
    ProvenanceRecord {
        track: "comparative",
        experiment: "exp104",
        python_script: "control/comparative/exp104_cross_species_pk.py",
        python_version: "3.10+",
        description: "Cross-species allometric PK scaling",
        checks: 0,
    },
    ProvenanceRecord {
        track: "comparative",
        experiment: "exp105",
        python_script: "control/comparative/exp105_canine_gut_anderson.py",
        python_version: "3.10+",
        description: "Canine gut Anderson diversity baseline",
        checks: 0,
    },
    ProvenanceRecord {
        track: "comparative",
        experiment: "exp106",
        python_script: "control/comparative/exp106_feline_hyperthyroid.py",
        python_version: "3.10+",
        description: "Feline hyperthyroidism methimazole PK baseline",
        checks: 0,
    },
    // Track 6: Discovery
    ProvenanceRecord {
        track: "discovery",
        experiment: "exp090",
        python_script: "control/discovery/exp090_matrix_scoring.py",
        python_version: "3.10+",
        description: "Anderson-augmented MATRIX drug repurposing scoring",
        checks: 0,
    },
    ProvenanceRecord {
        track: "discovery",
        experiment: "exp091",
        python_script: "control/discovery/exp091_addrc_hts.py",
        python_version: "3.10+",
        description: "ADDRC high-throughput screening analysis (Z', SSMD)",
        checks: 0,
    },
    ProvenanceRecord {
        track: "discovery",
        experiment: "exp092",
        python_script: "control/discovery/exp092_compound_library.py",
        python_version: "3.10+",
        description: "ADDRC compound library batch IC50 profiling",
        checks: 0,
    },
    ProvenanceRecord {
        track: "discovery",
        experiment: "exp093",
        python_script: "control/discovery/exp093_chembl_jak_panel.py",
        python_version: "3.10+",
        description: "ChEMBL JAK inhibitor selectivity panel",
        checks: 0,
    },
    ProvenanceRecord {
        track: "discovery",
        experiment: "exp094",
        python_script: "control/discovery/exp094_rho_mrtf_fibrosis.py",
        python_version: "3.10+",
        description: "Rho/MRTF/SRF fibrosis scoring baseline",
        checks: 0,
    },
    // Track 7: Validation
    ProvenanceRecord {
        track: "validation",
        experiment: "exp040",
        python_script: "control/validation/exp040_barracuda_cpu.py",
        python_version: "3.10+",
        description: "barraCuda CPU parity analytical baselines",
        checks: 0,
    },
    // Track 8: Scripts
    ProvenanceRecord {
        track: "scripts",
        experiment: "bench_barracuda",
        python_script: "control/scripts/bench_barracuda_cpu_vs_python.py",
        python_version: "3.10+",
        description: "barraCuda CPU vs Python benchmark timing",
        checks: 0,
    },
    ProvenanceRecord {
        track: "scripts",
        experiment: "bench_v16",
        python_script: "control/scripts/bench_v16_cpu_vs_python.py",
        python_version: "3.10+",
        description: "V16 CPU parity benchmarks (Python baseline)",
        checks: 0,
    },
    ProvenanceRecord {
        track: "scripts",
        experiment: "compare_v16",
        python_script: "control/scripts/compare_v16_benchmarks.py",
        python_version: "3.10+",
        description: "Rust CPU vs Python benchmark comparison",
        checks: 0,
    },
    ProvenanceRecord {
        track: "scripts",
        experiment: "exp085",
        python_script: "control/scripts/control_exp085_gpu_scaling.py",
        python_version: "3.10+",
        description: "GPU scaling validation (Python control)",
        checks: 0,
    },
    ProvenanceRecord {
        track: "scripts",
        experiment: "tolerances",
        python_script: "control/tolerances.py",
        python_version: "3.10+",
        description: "Centralized tolerance constants (Python mirror of tolerances.rs)",
        checks: 0,
    },
    ProvenanceRecord {
        track: "scripts",
        experiment: "update_provenance",
        python_script: "control/update_provenance.py",
        python_version: "3.10+",
        description: "Update baseline JSON files with provenance metadata",
        checks: 0,
    },
];

/// Provenance for a Python-derived baseline value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineProvenance {
    pub date: String,
    pub script: String,
    pub command: String,
    pub git_commit: String,
    pub python: String,
    pub numpy: String,
}

/// Provenance for an analytically-derived expected value.
#[derive(Debug, Clone)]
pub struct AnalyticalProvenance {
    pub formula: &'static str,
    pub reference: &'static str,
    pub doi: Option<&'static str>,
}

/// Load provenance from a baseline JSON string (the `_provenance` field).
///
/// Returns `None` if the JSON does not contain `_provenance` or it fails
/// to deserialize.
#[must_use]
pub fn load_provenance(json_str: &str) -> Option<BaselineProvenance> {
    let val: serde_json::Value = serde_json::from_str(json_str).ok()?;
    let prov = val.get("_provenance")?;
    serde_json::from_value(prov.clone()).ok()
}

/// Log provenance to stdout (for validation binary output).
pub fn log_provenance(prov: &BaselineProvenance) {
    println!(
        "  provenance: script={}, commit={}, date={}, python={}, numpy={}",
        prov.script, prov.git_commit, prov.date, prov.python, prov.numpy,
    );
}

/// Log analytical provenance to stdout.
pub fn log_analytical(prov: &AnalyticalProvenance) {
    print!("  analytical: {} — {}", prov.formula, prov.reference);
    if let Some(doi) = prov.doi {
        print!(" (doi:{doi})");
    }
    println!();
}

/// Well-known analytical provenances for healthSpring experiments.
pub mod known {
    use super::AnalyticalProvenance;

    pub const HILL_AT_IC50: AnalyticalProvenance = AnalyticalProvenance {
        formula: "R = C^n / (IC50^n + C^n) at C=IC50 → 0.5",
        reference: "Hill 1910, J Physiol",
        doi: Some("10.1113/jphysiol.1910.sp001397"),
    };

    pub const IV_BOLUS_C0: AnalyticalProvenance = AnalyticalProvenance {
        formula: "C(0) = Dose / Vd",
        reference: "Rowland & Tozer, Clinical Pharmacokinetics",
        doi: None,
    };

    pub const ORAL_C0_ZERO: AnalyticalProvenance = AnalyticalProvenance {
        formula: "C(0) = 0 (oral absorption delay)",
        reference: "Bateman equation initial condition",
        doi: None,
    };

    pub const SHANNON_UNIFORM: AnalyticalProvenance = AnalyticalProvenance {
        formula: "H' = ln(S) for uniform distribution",
        reference: "Shannon 1948, Bell System Technical Journal",
        doi: Some("10.1002/j.1538-7305.1948.tb01338.x"),
    };

    pub const IPR_DELTA: AnalyticalProvenance = AnalyticalProvenance {
        formula: "IPR(δ) = 1 (single-site state)",
        reference: "Anderson 1958, Phys Rev",
        doi: Some("10.1103/PhysRev.109.1492"),
    };

    pub const TESTOSTERONE_DECLINE: AnalyticalProvenance = AnalyticalProvenance {
        formula: "T(age) = T0 · exp(-k · (age - onset))",
        reference: "Harman et al. 2001, JCEM",
        doi: Some("10.1210/jcem.86.2.7219"),
    };
}

#[cfg(test)]
fn count_py_files(dir: &std::path::Path) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    let mut n = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            n += count_py_files(&path);
        } else if path.extension().is_some_and(|e| e == "py") {
            n += 1;
        }
    }
    n
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn parse_provenance_from_json() {
        let json = r#"{
            "shannon": 2.1,
            "_provenance": {
                "date": "2026-03-08",
                "script": "control/microbiome/exp010_diversity_indices.py",
                "command": "python3 control/microbiome/exp010_diversity_indices.py",
                "git_commit": "e93a81b6",
                "python": "3.10.12",
                "numpy": "2.1.3"
            }
        }"#;
        let prov = load_provenance(json).expect("should parse");
        assert_eq!(prov.date, "2026-03-08");
        assert!(prov.script.contains("exp010"));
    }

    #[test]
    fn missing_provenance_returns_none() {
        let json = r#"{"shannon": 2.1}"#;
        assert!(load_provenance(json).is_none());
    }

    #[test]
    fn analytical_provenance_has_doi() {
        assert!(known::HILL_AT_IC50.doi.is_some());
        assert!(known::SHANNON_UNIFORM.doi.is_some());
    }

    #[test]
    fn registry_complete() {
        let control_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("control");
        let count = count_py_files(&control_dir);
        assert_eq!(
            PROVENANCE_REGISTRY.len(),
            count,
            "PROVENANCE_REGISTRY has {} entries but control/ has {} .py files — add missing entries",
            PROVENANCE_REGISTRY.len(),
            count,
        );
    }
}
