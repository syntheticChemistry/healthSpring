// SPDX-License-Identifier: AGPL-3.0-only
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

use serde::{Deserialize, Serialize};

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
}
