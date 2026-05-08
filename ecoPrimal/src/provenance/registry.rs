// SPDX-License-Identifier: AGPL-3.0-or-later
//! Provenance registry: enumerates all Python control scripts in `control/`.
//!
//! This is the data layer — one entry per Python baseline script.
//! See [`super::ProvenanceRecord`] for the record type.
//!
//! Data is split across five submodules for the 1000-LOC standard:
//! - [`records_science`]: Tracks 1–5 (PK/PD, Microbiome, Biosignal, Endocrine, Comparative)
//! - [`records_discovery`]: Track 6 (Discovery, affinity, fibrosis, delivery)
//! - [`records_gpu`]: GPU parity and compute benchmark experiments
//! - [`records_composition`]: Composition validation Tier 4–5 (exp112–122)
//! - [`records_infra`]: Remaining: validation, scripts, toxicology, simulation,
//!   diagnostic, NLME, QS/real-data, and demo experiments.

use super::ProvenanceRecord;
use super::records_composition;
use super::records_discovery;
use super::records_gpu;
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
    /// Composition (NUCLEUS composition validation).
    pub const COMPOSITION: &str = "composition";
}

/// Science baselines: Tracks 1–5 (Python-backed provenance records).
pub const PROVENANCE_SCIENCE: &[ProvenanceRecord] = records_science::RECORDS;

/// Infrastructure, validation, and misc records.
pub const PROVENANCE_INFRA: &[ProvenanceRecord] = records_infra::RECORDS;

/// Discovery experiments (Track 6).
pub const PROVENANCE_DISCOVERY: &[ProvenanceRecord] = records_discovery::RECORDS;

/// GPU parity and compute benchmark experiments.
pub const PROVENANCE_GPU: &[ProvenanceRecord] = records_gpu::RECORDS;

/// Composition validation experiments (Tier 4–5).
pub const PROVENANCE_COMPOSITION: &[ProvenanceRecord] = records_composition::RECORDS;

/// Total number of provenance records across all partitions.
#[must_use]
pub const fn registry_len() -> usize {
    PROVENANCE_SCIENCE.len()
        + PROVENANCE_INFRA.len()
        + PROVENANCE_DISCOVERY.len()
        + PROVENANCE_GPU.len()
        + PROVENANCE_COMPOSITION.len()
}

/// Iterate over all provenance records (all partitions).
pub fn all_records() -> impl Iterator<Item = &'static ProvenanceRecord> {
    PROVENANCE_SCIENCE
        .iter()
        .chain(PROVENANCE_INFRA.iter())
        .chain(PROVENANCE_DISCOVERY.iter())
        .chain(PROVENANCE_GPU.iter())
        .chain(PROVENANCE_COMPOSITION.iter())
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
