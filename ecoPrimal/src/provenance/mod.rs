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

mod registry;

/// All registered Python control scripts under `control/`, for completeness checks.
pub use registry::PROVENANCE_REGISTRY;
pub use registry::{distinct_tracks, record_for_experiment, records_for_track, tracks};

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
    /// Git commit hash of the Python baseline run (empty string if not applicable).
    pub git_commit: &'static str,
    /// Date the baseline was generated (empty string if not applicable).
    pub run_date: &'static str,
    /// Exact command used to generate the baseline (empty string if not applicable).
    pub exact_command: &'static str,
    /// Literature-derived baseline source: citation (DOI, database accession), figure/table/dataset (empty if Python-derived).
    pub baseline_source: &'static str,
}

/// Provenance for a Python-derived baseline value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineProvenance {
    /// Calendar date string for the baseline generation run.
    pub date: String,
    /// Path to the control script that produced the baseline.
    pub script: String,
    /// Shell command used to reproduce the baseline output.
    pub command: String,
    /// Repository revision recorded at baseline time.
    pub git_commit: String,
    /// Python interpreter version line (e.g. `3.11.x`).
    pub python: String,
    /// `NumPy` version string pinned for numerical parity.
    pub numpy: String,
}

/// Provenance for an analytically-derived expected value.
#[derive(Debug, Clone)]
pub struct AnalyticalProvenance {
    /// Closed-form or symbolic expression being asserted in tests.
    pub formula: &'static str,
    /// Textbook or paper citation backing the formula.
    pub reference: &'static str,
    /// Optional DOI when the reference is formally citable.
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

/// Emit provenance as a structured tracing event.
///
/// Validation binaries should initialize a `tracing_subscriber` that writes
/// to stdout so these events remain visible in CI logs.
pub fn log_provenance(prov: &BaselineProvenance) {
    tracing::info!(
        script = %prov.script,
        commit = %prov.git_commit,
        date = %prov.date,
        python = %prov.python,
        numpy = %prov.numpy,
        "baseline provenance",
    );
}

/// Emit analytical provenance as a structured tracing event.
pub fn log_analytical(prov: &AnalyticalProvenance) {
    if let Some(doi) = prov.doi {
        tracing::info!(
            formula = %prov.formula,
            reference = %prov.reference,
            doi = %doi,
            "analytical provenance",
        );
    } else {
        tracing::info!(
            formula = %prov.formula,
            reference = %prov.reference,
            "analytical provenance",
        );
    }
}

/// Well-known analytical provenances for healthSpring experiments.
pub mod known {
    use super::AnalyticalProvenance;

    /// Hill equation at half-saturation (reference PK/PD anchor).
    pub const HILL_AT_IC50: AnalyticalProvenance = AnalyticalProvenance {
        formula: "R = C^n / (IC50^n + C^n) at C=IC50 → 0.5",
        reference: "Hill 1910, J Physiol",
        doi: Some("10.1113/jphysiol.1910.sp001397"),
    };

    /// Intravenous bolus initial concentration from dose and Vd.
    pub const IV_BOLUS_C0: AnalyticalProvenance = AnalyticalProvenance {
        formula: "C(0) = Dose / Vd",
        reference: "Rowland & Tozer, Clinical Pharmacokinetics",
        doi: None,
    };

    /// Zero central concentration immediately after oral dosing (absorption lag).
    pub const ORAL_C0_ZERO: AnalyticalProvenance = AnalyticalProvenance {
        formula: "C(0) = 0 (oral absorption delay)",
        reference: "Bateman equation initial condition",
        doi: None,
    };

    /// Shannon entropy for a discrete uniform abundance distribution.
    pub const SHANNON_UNIFORM: AnalyticalProvenance = AnalyticalProvenance {
        formula: "H' = ln(S) for uniform distribution",
        reference: "Shannon 1948, Bell System Technical Journal",
        doi: Some("10.1002/j.1538-7305.1948.tb01338.x"),
    };

    /// Inverse participation ratio for a single occupied site (localization toy).
    pub const IPR_DELTA: AnalyticalProvenance = AnalyticalProvenance {
        formula: "IPR(δ) = 1 (single-site state)",
        reference: "Anderson 1958, Phys Rev",
        doi: Some("10.1103/PhysRev.109.1492"),
    };

    /// Exponential age decline of total testosterone (longitudinal aging form).
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
