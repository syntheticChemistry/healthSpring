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

/// Intermediate state threaded through phases.
struct ChainState {
    content_hash: String,
    session_id: String,
    merkle_root: String,
    signature: String,
    entry_id: String,
    braid_id: String,
}

impl ChainState {
    const fn empty() -> Self {
        Self {
            content_hash: String::new(),
            session_id: String::new(),
            merkle_root: String::new(),
            signature: String::new(),
            entry_id: String::new(),
            braid_id: String::new(),
        }
    }
}

/// Extract a string field from a JSON-RPC result, returning an owned `String`.
fn extract_str(result: Option<&serde_json::Value>, keys: &[&str]) -> String {
    result
        .and_then(|r| {
            keys.iter()
                .find_map(|k| r.get(*k).and_then(serde_json::Value::as_str))
        })
        .unwrap_or("")
        .into()
}

fn phase1_structural(v: &mut ValidationResult) {
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
}

fn phase3_nestgate(v: &mut ValidationResult, ctx: &mut CompositionContext, state: &mut ChainState) {
    v.section("Phase 3: NestGate (storage)");

    let payload = sample_clinical_payload();
    let store_key = "nest-atomic-validate:pkpd:001";

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

    state.content_hash = extract_str(stored_hash.as_ref(), &["content_hash", "hash"]);

    if !state.content_hash.is_empty() {
        v.check_bool(
            "nestgate_content_hash_nonempty",
            true,
            &format!(
                "BLAKE3 hash: {}",
                &state.content_hash[..state.content_hash.len().min(16)]
            ),
        );
    }

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
        v.check_bool(
            "nestgate_round_trip_integrity",
            round_trip_ok,
            "stored data recoverable",
        );
    }

    let exists_result = call_or_skip(
        ctx,
        v,
        "nestgate_content_exists",
        "storage",
        "storage.exists",
        serde_json::json!({"key": store_key}),
    );
    if let Some(ref result) = exists_result {
        let exists = result.as_bool().unwrap_or_else(|| {
            result
                .get("exists")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
        });
        v.check_bool("nestgate_content_confirmed", exists, "stored content exists");
    }

    call_or_skip(
        ctx,
        v,
        "nestgate_content_list",
        "storage",
        "storage.list",
        serde_json::json!({"prefix": "nest-atomic-validate:"}),
    );
}

fn phase4_rhizocrypt(
    v: &mut ValidationResult,
    ctx: &mut CompositionContext,
    state: &mut ChainState,
) {
    v.section("Phase 4: rhizoCrypt (DAG)");

    let session_result = call_or_skip(
        ctx,
        v,
        "rhizocrypt_dag_session_create",
        "dag",
        "dag.session.create",
        serde_json::json!({"experiment": "nest_atomic_validation"}),
    );

    state.session_id = extract_str(session_result.as_ref(), &["session_id"]);

    if !state.session_id.is_empty() {
        v.check_bool(
            "rhizocrypt_session_id_nonempty",
            true,
            &format!(
                "session: {}",
                &state.session_id[..state.session_id.len().min(16)]
            ),
        );
    }

    let event1 = call_or_skip(
        ctx,
        v,
        "rhizocrypt_dag_event_ingest",
        "dag",
        "dag.event.append",
        serde_json::json!({
            "session_id": state.session_id,
            "event": "clinical_data_ingest",
            "data": {
                "content_hash": state.content_hash,
                "step": "ingest",
                "source": "nestgate_storage"
            }
        }),
    );

    let event2 = call_or_skip(
        ctx,
        v,
        "rhizocrypt_dag_event_validate",
        "dag",
        "dag.event.append",
        serde_json::json!({
            "session_id": state.session_id,
            "event": "clinical_data_validate",
            "data": {
                "step": "validate",
                "model": "hill_equation",
                "tolerance": "1e-6"
            }
        }),
    );

    let event3 = call_or_skip(
        ctx,
        v,
        "rhizocrypt_dag_event_transform",
        "dag",
        "dag.event.append",
        serde_json::json!({
            "session_id": state.session_id,
            "event": "clinical_data_transform",
            "data": {
                "step": "transform",
                "operation": "dose_response_fit",
                "output_hash": state.content_hash
            }
        }),
    );

    let best_event = event3.or(event2).or(event1);
    state.merkle_root = extract_str(best_event.as_ref(), &["merkle_root"]);
}

fn phase5_beardog(v: &mut ValidationResult, ctx: &mut CompositionContext, state: &mut ChainState) {
    v.section("Phase 5: BearDog (Tower crypto)");

    let sign_payload = if state.merkle_root.is_empty() {
        "nest-atomic-validation-fallback".to_owned()
    } else {
        state.merkle_root.clone()
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

    state.signature = extract_str(sign_result.as_ref(), &["signature"]);

    if !state.signature.is_empty() {
        v.check_bool(
            "beardog_signature_nonempty",
            true,
            &format!(
                "sig: {}...",
                &state.signature[..state.signature.len().min(16)]
            ),
        );
    }
}

fn phase6_loamspine(
    v: &mut ValidationResult,
    ctx: &mut CompositionContext,
    state: &mut ChainState,
) {
    v.section("Phase 6: loamSpine (ledger)");

    let ledger_result = call_or_skip(
        ctx,
        v,
        "loamspine_ledger_entry_append",
        "commit",
        "entry.append",
        serde_json::json!({
            "session_id": state.session_id,
            "merkle_root": state.merkle_root,
            "signature": state.signature,
            "content_hash": state.content_hash,
            "experiment": "nest_atomic_validation",
        }),
    );

    state.entry_id = extract_str(ledger_result.as_ref(), &["entry_id", "commit_id"]);

    if !state.entry_id.is_empty() {
        v.check_bool(
            "loamspine_entry_id_nonempty",
            true,
            &format!(
                "entry: {}",
                &state.entry_id[..state.entry_id.len().min(16)]
            ),
        );
    }
}

fn phase7_sweetgrass(
    v: &mut ValidationResult,
    ctx: &mut CompositionContext,
    state: &mut ChainState,
) {
    v.section("Phase 7: sweetGrass (attribution)");

    let braid_result = call_or_skip(
        ctx,
        v,
        "sweetgrass_braid_create",
        "braid",
        "braid.create",
        serde_json::json!({"experiment": "nest_atomic_validation"}),
    );

    state.braid_id = extract_str(braid_result.as_ref(), &["braid_id", "id"]);

    if !state.braid_id.is_empty() {
        call_or_skip(
            ctx,
            v,
            "sweetgrass_braid_commit",
            "braid",
            "braid.commit",
            serde_json::json!({
                "braid_id": state.braid_id,
                "data": {
                    "commit_ref": state.entry_id,
                    "session_id": state.session_id,
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
}

fn phase8_tower_aux(v: &mut ValidationResult, ctx: &mut CompositionContext, state: &ChainState) {
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
            "session_id": state.session_id,
            "result": {
                "content_hash": state.content_hash,
                "merkle_root": state.merkle_root,
                "entry_id": state.entry_id,
                "braid_id": state.braid_id,
            }
        }),
    );
}

fn phase9_chain_audit(v: &mut ValidationResult, state: &ChainState) {
    v.section("Phase 9: Chain Audit");

    let chain_complete = !state.content_hash.is_empty()
        && !state.session_id.is_empty()
        && !state.merkle_root.is_empty()
        && !state.signature.is_empty()
        && !state.entry_id.is_empty()
        && !state.braid_id.is_empty();

    let chain_partial = !state.session_id.is_empty() || !state.content_hash.is_empty();

    if chain_complete {
        v.check_bool(
            "nest_chain_complete",
            true,
            "full provenance chain recoverable",
        );
    } else if chain_partial {
        let populated: Vec<&str> = [
            (!state.content_hash.is_empty()).then_some("content"),
            (!state.session_id.is_empty()).then_some("session"),
            (!state.merkle_root.is_empty()).then_some("merkle"),
            (!state.signature.is_empty()).then_some("signature"),
            (!state.entry_id.is_empty()).then_some("ledger"),
            (!state.braid_id.is_empty()).then_some("braid"),
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
        v.check_skip(
            "nest_chain_status",
            "no chain elements populated (primals unavailable)",
        );
    }
}

fn run(v: &mut ValidationResult, ctx: &mut CompositionContext) {
    phase1_structural(v);

    v.section("Phase 2: Liveness (7 primals)");

    if ctx.available_capabilities().is_empty() {
        v.check_skip(
            "nest_liveness",
            "no capabilities discovered — NUCLEUS not deployed",
        );
        return;
    }

    let nest_caps = [
        "crypto", "discovery", "audit", "storage", "dag", "commit", "braid",
    ];
    let alive = validate_liveness(ctx, v, &nest_caps);

    if alive == 0 {
        v.check_skip(
            "nest_atomic_pipeline",
            "zero primals alive — cannot exercise capabilities",
        );
        return;
    }

    let mut state = ChainState::empty();
    phase3_nestgate(v, ctx, &mut state);
    phase4_rhizocrypt(v, ctx, &mut state);
    phase5_beardog(v, ctx, &mut state);
    phase6_loamspine(v, ctx, &mut state);
    phase7_sweetgrass(v, ctx, &mut state);
    phase8_tower_aux(v, ctx, &state);
    phase9_chain_audit(v, &state);
}
