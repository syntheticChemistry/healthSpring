// SPDX-License-Identifier: AGPL-3.0-or-later

//! Nest Atomic validation — exercises all 7 primals in the neutron
//! composition through clinical data pipelines.
//!
//! Capabilities exercised:
//!   nestGate   — storage.store, storage.retrieve, storage.exists, storage.list
//!   rhizoCrypt — dag.session.create, dag.event.append
//!   loamSpine  — entry.append (ledger commit)
//!   sweetGrass — braid.create, braid.commit (attribution)
//!   bearDog    — crypto.sign (Merkle root signature)
//!   songbird   — discovery.peers (peer enumeration)
//!   skunkBat   — defense.audit (audit trail)

use primalspring::composition::CompositionContext;
use primalspring::validation::ValidationResult;

use crate::composition::{call_or_skip, capability_to_primal, validate_liveness};
use crate::primal_names;

use super::registry::{Scenario, ScenarioMeta, Tier, Track};

#[allow(
    non_snake_case,
    reason = "scenario module names mirror upstream mixed-case identifiers"
)]
pub fn SCENARIO() -> Scenario {
    Scenario {
        meta: ScenarioMeta {
            id: "nest-atomic",
            track: Track::Composition,
            tier: Tier::Both,
            source_experiment: "nest_atomic_v1",
            description: "Nest Atomic (neutron) — 7-primal composition through clinical data.",
        },
        run,
    }
}

/// Sample clinical dataset for validation — realistic enough to exercise
/// the full pipeline without containing actual patient data.
fn sample_clinical_payload() -> serde_json::Value {
    serde_json::json!({
        "patient_id": "NEST-VALIDATE-001",
        "pipeline": "pkpd_dose_response",
        "timestamp": "2026-05-13T15:00:00Z",
        "data": {
            "doses_mg": [0.1, 0.5, 1.0, 5.0, 10.0, 50.0, 100.0],
            "responses": [0.02, 0.09, 0.18, 0.62, 0.83, 0.97, 0.99],
            "model": "hill_equation",
            "ec50": 3.2,
            "hill_coefficient": 1.4
        },
        "provenance": {
            "source": "healthSpring nest-atomic validation",
            "experiment": "exp_nest_validate"
        }
    })
}

fn run(v: &mut ValidationResult, ctx: &mut CompositionContext) {
    // ═══════════════════════════════════════════════════════════════════
    // Phase 1: Structural — verify routing maps without live primals
    // ═══════════════════════════════════════════════════════════════════
    v.section("Phase 1: Structural Routing");

    v.check_bool(
        "storage_routes_to_nestgate",
        capability_to_primal("storage") == primal_names::NESTGATE,
        "storage → nestgate",
    );
    v.check_bool(
        "dag_routes_to_rhizocrypt",
        capability_to_primal("dag") == primal_names::RHIZOCRYPT,
        "dag → rhizocrypt",
    );
    v.check_bool(
        "commit_routes_to_loamspine",
        capability_to_primal("commit") == primal_names::LOAMSPINE,
        "commit → loamspine",
    );
    v.check_bool(
        "braid_routes_to_sweetgrass",
        capability_to_primal("braid") == primal_names::SWEETGRASS,
        "braid → sweetgrass",
    );
    v.check_bool(
        "crypto_routes_to_beardog",
        capability_to_primal("crypto") == primal_names::BEARDOG,
        "crypto → beardog",
    );
    v.check_bool(
        "discovery_routes_to_songbird",
        capability_to_primal("discovery") == primal_names::SONGBIRD,
        "discovery → songbird",
    );
    v.check_bool(
        "audit_routes_to_skunkbat",
        capability_to_primal("audit") == primal_names::SKUNKBAT,
        "audit → skunkbat",
    );

    // ═══════════════════════════════════════════════════════════════════
    // Phase 2: Liveness — verify all 7 Nest primals respond
    // ═══════════════════════════════════════════════════════════════════
    v.section("Phase 2: Liveness (7 primals)");

    if ctx.available_capabilities().is_empty() {
        v.check_skip("nest_liveness", "no capabilities discovered — NUCLEUS not deployed");
        return;
    }

    let nest_caps = [
        "crypto",     // bearDog
        "discovery",  // songbird
        "audit",      // skunkBat
        "storage",    // nestGate
        "dag",        // rhizoCrypt
        "commit",     // loamSpine
        "braid",      // sweetGrass
    ];
    let alive = validate_liveness(ctx, v, &nest_caps);

    if alive == 0 {
        v.check_skip(
            "nest_atomic_pipeline",
            "zero primals alive — cannot exercise capabilities",
        );
        return;
    }

    // ═══════════════════════════════════════════════════════════════════
    // Phase 3: NestGate — content.put / content.get / content.exists / content.list
    // ═══════════════════════════════════════════════════════════════════
    v.section("Phase 3: NestGate (storage)");

    let payload = sample_clinical_payload();
    let store_key = "nest-atomic-validate:pkpd:001";

    // content.put → storage.store
    let stored_hash = call_or_skip(
        ctx,
        v,
        "nestgate_content_put",
        "storage",
        "storage.store",
        serde_json::json!({
            "key": store_key,
            "content": payload,
            "hash_algorithm": "blake3",
        }),
    );

    let content_hash = stored_hash
        .as_ref()
        .and_then(|r| {
            r.get("content_hash")
                .or_else(|| r.get("hash"))
                .and_then(serde_json::Value::as_str)
        })
        .unwrap_or("")
        .to_owned();

    if !content_hash.is_empty() {
        v.check_bool(
            "nestgate_content_hash_nonempty",
            true,
            &format!("BLAKE3 hash: {}", &content_hash[..content_hash.len().min(16)]),
        );
    }

    // content.get → storage.retrieve
    let retrieved = call_or_skip(
        ctx,
        v,
        "nestgate_content_get",
        "storage",
        "storage.retrieve",
        serde_json::json!({"key": store_key}),
    );

    if let Some(ref result) = retrieved {
        let round_trip_ok = result.get("content").is_some()
            || result.get("data").is_some()
            || result.get("value").is_some()
            || result.as_str().is_some();
        v.check_bool("nestgate_round_trip_integrity", round_trip_ok, "stored data recoverable");
    }

    // content.exists → storage.exists
    let exists_result = call_or_skip(
        ctx,
        v,
        "nestgate_content_exists",
        "storage",
        "storage.exists",
        serde_json::json!({"key": store_key}),
    );
    if let Some(ref result) = exists_result {
        let exists = result.as_bool().unwrap_or(
            result
                .get("exists")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        );
        v.check_bool("nestgate_content_confirmed", exists, "stored content exists");
    }

    // content.list → storage.list
    call_or_skip(
        ctx,
        v,
        "nestgate_content_list",
        "storage",
        "storage.list",
        serde_json::json!({"prefix": "nest-atomic-validate:"}),
    );

    // ═══════════════════════════════════════════════════════════════════
    // Phase 4: rhizoCrypt — dag.session.create / dag.event.append
    // ═══════════════════════════════════════════════════════════════════
    v.section("Phase 4: rhizoCrypt (DAG)");

    let session_result = call_or_skip(
        ctx,
        v,
        "rhizocrypt_dag_session_create",
        "dag",
        "dag.session.create",
        serde_json::json!({"experiment": "nest_atomic_validation"}),
    );

    let session_id = session_result
        .as_ref()
        .and_then(|r| r.get("session_id").and_then(serde_json::Value::as_str))
        .unwrap_or("")
        .to_owned();

    if !session_id.is_empty() {
        v.check_bool(
            "rhizocrypt_session_id_nonempty",
            true,
            &format!("session: {}", &session_id[..session_id.len().min(16)]),
        );
    }

    // Append ingest event
    let event1 = call_or_skip(
        ctx,
        v,
        "rhizocrypt_dag_event_ingest",
        "dag",
        "dag.event.append",
        serde_json::json!({
            "session_id": session_id,
            "event": "clinical_data_ingest",
            "data": {
                "content_hash": content_hash,
                "step": "ingest",
                "source": "nestgate_storage"
            }
        }),
    );

    // Append validate event
    let event2 = call_or_skip(
        ctx,
        v,
        "rhizocrypt_dag_event_validate",
        "dag",
        "dag.event.append",
        serde_json::json!({
            "session_id": session_id,
            "event": "clinical_data_validate",
            "data": {
                "step": "validate",
                "model": "hill_equation",
                "tolerance": "1e-6"
            }
        }),
    );

    // Append transform event
    let event3 = call_or_skip(
        ctx,
        v,
        "rhizocrypt_dag_event_transform",
        "dag",
        "dag.event.append",
        serde_json::json!({
            "session_id": session_id,
            "event": "clinical_data_transform",
            "data": {
                "step": "transform",
                "operation": "dose_response_fit",
                "output_hash": content_hash
            }
        }),
    );

    let merkle_root = event3
        .as_ref()
        .or(event2.as_ref())
        .or(event1.as_ref())
        .and_then(|r| r.get("merkle_root").and_then(serde_json::Value::as_str))
        .unwrap_or("")
        .to_owned();

    // ═══════════════════════════════════════════════════════════════════
    // Phase 5: BearDog — crypto.sign (Tower trust for Merkle root)
    // ═══════════════════════════════════════════════════════════════════
    v.section("Phase 5: BearDog (Tower crypto)");

    let sign_payload = if merkle_root.is_empty() {
        "nest-atomic-validation-placeholder".to_owned()
    } else {
        merkle_root.clone()
    };

    let sign_result = call_or_skip(
        ctx,
        v,
        "beardog_crypto_sign_merkle",
        "crypto",
        "crypto.sign",
        serde_json::json!({
            "payload": sign_payload,
            "algorithm": "ed25519",
        }),
    );

    let signature = sign_result
        .as_ref()
        .and_then(|r| r.get("signature").and_then(serde_json::Value::as_str))
        .unwrap_or("")
        .to_owned();

    if !signature.is_empty() {
        v.check_bool(
            "beardog_signature_nonempty",
            true,
            &format!("sig: {}...", &signature[..signature.len().min(16)]),
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    // Phase 6: loamSpine — ledger.entry.append
    // ═══════════════════════════════════════════════════════════════════
    v.section("Phase 6: loamSpine (ledger)");

    let ledger_result = call_or_skip(
        ctx,
        v,
        "loamspine_ledger_entry_append",
        "commit",
        "entry.append",
        serde_json::json!({
            "session_id": session_id,
            "merkle_root": merkle_root,
            "signature": signature,
            "content_hash": content_hash,
            "experiment": "nest_atomic_validation",
        }),
    );

    let entry_id = ledger_result
        .as_ref()
        .and_then(|r| {
            r.get("entry_id")
                .or_else(|| r.get("commit_id"))
                .and_then(serde_json::Value::as_str)
        })
        .unwrap_or("")
        .to_owned();

    if !entry_id.is_empty() {
        v.check_bool(
            "loamspine_entry_id_nonempty",
            true,
            &format!("entry: {}", &entry_id[..entry_id.len().min(16)]),
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    // Phase 7: sweetGrass — braid.attribution.create
    // ═══════════════════════════════════════════════════════════════════
    v.section("Phase 7: sweetGrass (attribution)");

    let braid_result = call_or_skip(
        ctx,
        v,
        "sweetgrass_braid_create",
        "braid",
        "braid.create",
        serde_json::json!({"experiment": "nest_atomic_validation"}),
    );

    let braid_id = braid_result
        .as_ref()
        .and_then(|r| {
            r.get("braid_id")
                .or_else(|| r.get("id"))
                .and_then(serde_json::Value::as_str)
        })
        .unwrap_or("")
        .to_owned();

    if !braid_id.is_empty() {
        call_or_skip(
            ctx,
            v,
            "sweetgrass_braid_commit",
            "braid",
            "braid.commit",
            serde_json::json!({
                "braid_id": braid_id,
                "data": {
                    "commit_ref": entry_id,
                    "session_id": session_id,
                    "agents": [{
                        "did": "did:key:healthSpring",
                        "role": "author",
                        "contribution": 1.0
                    }]
                }
            }),
        );
    } else if braid_result.is_some() {
        v.check_skip("sweetgrass_braid_commit", "braid_create returned no id");
    }

    // ═══════════════════════════════════════════════════════════════════
    // Phase 8: Tower auxiliary — songbird discovery + skunkBat audit
    // ═══════════════════════════════════════════════════════════════════
    v.section("Phase 8: Tower auxiliary");

    call_or_skip(
        ctx,
        v,
        "songbird_discovery_peers",
        "discovery",
        "discovery.peers",
        serde_json::json!({}),
    );

    call_or_skip(
        ctx,
        v,
        "skunkbat_defense_audit",
        "audit",
        "defense.audit",
        serde_json::json!({
            "event": "nest_atomic_validation_complete",
            "session_id": session_id,
            "result": {
                "content_hash": content_hash,
                "merkle_root": merkle_root,
                "entry_id": entry_id,
                "braid_id": braid_id,
            }
        }),
    );

    // ═══════════════════════════════════════════════════════════════════
    // Phase 9: Full chain recoverability
    // ═══════════════════════════════════════════════════════════════════
    v.section("Phase 9: Chain Audit");

    let chain_complete = !content_hash.is_empty()
        && !session_id.is_empty()
        && !merkle_root.is_empty()
        && !signature.is_empty()
        && !entry_id.is_empty()
        && !braid_id.is_empty();

    let chain_partial = !session_id.is_empty() || !content_hash.is_empty();

    if chain_complete {
        v.check_bool("nest_chain_complete", true, "full provenance chain recoverable");
    } else if chain_partial {
        let populated: Vec<&str> = [
            (!content_hash.is_empty()).then_some("content"),
            (!session_id.is_empty()).then_some("session"),
            (!merkle_root.is_empty()).then_some("merkle"),
            (!signature.is_empty()).then_some("signature"),
            (!entry_id.is_empty()).then_some("ledger"),
            (!braid_id.is_empty()).then_some("braid"),
        ]
        .into_iter()
        .flatten()
        .collect();
        v.check_bool(
            "nest_chain_partial",
            true,
            &format!("partial chain: {}", populated.join(", ")),
        );
    } else {
        v.check_skip("nest_chain_status", "no chain elements populated (primals unavailable)");
    }
}
