// SPDX-License-Identifier: AGPL-3.0-or-later
//! Provenance registry: enumerates all Python control scripts in `control/`.
//!
//! This is the data layer — one entry per Python baseline script.
//! See [`super::ProvenanceRecord`] for the record type.
//!
//! Data is split across two submodules for the 1000-LOC standard:
//! - [`records_science`]: Tracks 1–5 (PK/PD, Microbiome, Biosignal, Endocrine, Comparative)
//! - [`records_infra`]: Tracks 6–10 (Discovery, Validation, Scripts, Toxicology, Simulation)
//!   plus GPU parity, composition validation, and demo experiments.

use super::ProvenanceRecord;
use super::records_infra;
use super::records_science;

/// Well-known track identifiers used in the registry.
pub mod tracks {
    /// PK/PD (pharmacokinetics/pharmacodynamics).
    pub const PKPD: &str = "pkpd";
    /// Microbiome (diversity, SCFA, beta-diversity).
    pub const MICROBIOME: &str = "microbiome";
    /// Biosignal (ECG, HRV, beat classification).
    pub const BIOSIGNAL: &str = "biosignal";
    /// Endocrine (hormones, diurnal rhythms, HPA axis).
    pub const ENDOCRINE: &str = "endocrine";
    /// Comparative (cross-species PK, allometry).
    pub const COMPARATIVE: &str = "comparative";
    /// Discovery (localization, spectral, material).
    pub const DISCOVERY: &str = "discovery";
    /// Validation (cross-validate harness).
    pub const VALIDATION: &str = "validation";
    /// Scripts (utility scripts).
    pub const SCRIPTS: &str = "scripts";
    /// Toxicology (dose-response, hormesis).
    pub const TOXICOLOGY: &str = "toxicology";
    /// Simulation (mechanistic models).
    pub const SIMULATION: &str = "simulation";
}

/// Science baselines: Tracks 1–5 (Python-backed provenance records).
pub const PROVENANCE_SCIENCE: &[ProvenanceRecord] = records_science::RECORDS;

/// Infrastructure, validation, and composition records: Tracks 6–10+.
pub const PROVENANCE_INFRA: &[ProvenanceRecord] = records_infra::RECORDS;

/// Total number of provenance records across both partitions.
#[must_use]
pub const fn registry_len() -> usize {
    PROVENANCE_SCIENCE.len() + PROVENANCE_INFRA.len()
}

/// Iterate over all provenance records (science + infrastructure).
pub fn all_records() -> impl Iterator<Item = &'static ProvenanceRecord> {
    PROVENANCE_SCIENCE.iter().chain(PROVENANCE_INFRA.iter())
}

/// Iterate over records belonging to a specific track.
pub fn records_for_track(track: &str) -> impl Iterator<Item = &'static ProvenanceRecord> {
    all_records().filter(move |r| r.track == track)
}

/// Look up a single record by experiment id (e.g. `"exp001"`).
#[must_use]
pub fn record_for_experiment(experiment: &str) -> Option<&'static ProvenanceRecord> {
    all_records().find(|r| r.experiment == experiment)
}

/// Distinct track names present in the registry.
#[must_use]
pub fn distinct_tracks() -> Vec<&'static str> {
    let mut seen = Vec::new();
    for r in all_records() {
        if !seen.contains(&r.track) {
            seen.push(r.track);
        }
    }
    seen
}
