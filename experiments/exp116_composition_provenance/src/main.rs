// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(clippy::nursery)]

//! Exp116 — Composition validation: Provenance session lifecycle.
//!
//! Validates the provenance trio integration: begin/record/complete session
//! lifecycle, registry completeness, and data session round-trip. These are
//! the composition-layer provenance contracts that biomeOS relies on.

use healthspring_barracuda::data;
use healthspring_barracuda::provenance::{self, all_records, registry_len};
use healthspring_barracuda::validation::ValidationHarness;

fn main() {
    let mut h = ValidationHarness::new("Exp116 Composition Provenance Lifecycle");

    // ── Provenance registry completeness ────────────────────────────
    let tracks = provenance::distinct_tracks();
    h.check_bool("Provenance registry has multiple tracks", tracks.len() >= 5);

    h.check_bool(
        "Provenance registry has records for experiments",
        registry_len() >= 10,
    );

    let mut well_formed = true;
    for record in all_records() {
        if record.python_script.is_empty()
            || record.experiment.is_empty()
            || record.git_commit.is_empty()
        {
            well_formed = false;
        }
    }
    h.check_bool("All provenance records are well-formed", well_formed);

    // ── Trio availability probe ─────────────────────────────────────
    let _ = data::trio_available();
    h.check_bool("Trio availability probe returns without panic", true);

    // ── Data session lifecycle: begin → record → complete ───────────
    let session = data::begin_data_session("exp116_provenance_test");
    h.check_bool("Session begin returns an id", !session.id.is_empty());

    let step = serde_json::json!({
        "source": "test",
        "operation": "validate_provenance_lifecycle",
    });
    let _ = data::record_fetch_step(&session.id, &step);
    h.check_bool("Session record returns without panic", true);

    let chain = data::complete_data_session(&session.id, "AGPL-3.0-or-later");
    h.check_bool("Session complete returns status", !chain.status.is_empty());

    // ── Record lookup by experiment ─────────────────────────────────
    let hill_record = provenance::record_for_experiment("exp001");
    h.check_bool("Provenance lookup finds exp001", hill_record.is_some());
    if let Some(rec) = hill_record {
        h.check_bool(
            "exp001 provenance has python_script",
            !rec.python_script.is_empty(),
        );
        h.check_bool(
            "exp001 provenance has git_commit",
            !rec.git_commit.is_empty(),
        );
        h.check_bool(
            "exp001 provenance has exact_command",
            !rec.exact_command.is_empty(),
        );
    }

    // ── Record lookup by track ──────────────────────────────────────
    let pkpd_count = provenance::records_for_track("pkpd").count();
    h.check_bool("PKPD track has provenance records", pkpd_count > 0);

    let microbiome_count = provenance::records_for_track("microbiome").count();
    h.check_bool(
        "Microbiome track has provenance records",
        microbiome_count > 0,
    );

    // ── Session determinism ─────────────────────────────────────────
    let s1 = data::begin_data_session("determinism_a");
    let s2 = data::begin_data_session("determinism_b");
    h.check_bool("Distinct sessions get distinct IDs", s1.id != s2.id);

    h.exit();
}
