// SPDX-License-Identifier: AGPL-3.0-or-later

//! Nest Atomic composition facade — orchestrates the full provenance chain.
//!
//! The Nest Atomic = `NestGate` + `rhizoCrypt` + `loamSpine` + `sweetGrass`,
//! signed by `BearDog` from the Tower Atomic. This module composes
//! those IPC clients into a single lifecycle:
//!
//! ```text
//! begin_session()    → rhizoCrypt  dag.session.create
//! record_event(data) → NestGate    storage.store (BLAKE3 hash)
//!                    + rhizoCrypt  dag.event.append
//! sign_merkle()      → BearDog     crypto.sign (Merkle root)
//! commit()           → loamSpine   commit.create
//! attribute(contribs)→ sweetGrass  braid.create + braid.commit
//! ```
//!
//! Graceful degradation: each step independently degrades if the
//! target primal is unavailable. The facade tracks partial completion
//! so callers can inspect what succeeded.

use base64::Engine;
use serde_json::Value;

use crate::composition::HealthCompositionContext;

/// Status of a Nest Atomic composition operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NestStatus {
    /// All primals responded successfully.
    Complete,
    /// Some primals responded, others were unavailable.
    Partial,
    /// No primals available — local-only provenance.
    Unavailable,
}

impl std::fmt::Display for NestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Complete => write!(f, "complete"),
            Self::Partial => write!(f, "partial"),
            Self::Unavailable => write!(f, "unavailable"),
        }
    }
}

/// Result of a full Nest Atomic provenance lifecycle.
#[derive(Debug, Clone)]
pub struct NestProvenanceChain {
    /// Overall chain status.
    pub status: NestStatus,
    /// rhizoCrypt session ID (or `local-<experiment>` fallback).
    pub session_id: String,
    /// `NestGate` BLAKE3 content hash.
    pub content_hash: String,
    /// `BearDog` Ed25519 signature over the Merkle root.
    pub merkle_signature: String,
    /// loamSpine immutable ledger commit ID.
    pub commit_id: String,
    /// sweetGrass PROV-O attribution braid ID.
    pub braid_id: String,
}

impl NestProvenanceChain {
    /// Construct an empty chain representing total unavailability.
    #[must_use]
    pub const fn unavailable() -> Self {
        Self {
            status: NestStatus::Unavailable,
            session_id: String::new(),
            content_hash: String::new(),
            merkle_signature: String::new(),
            commit_id: String::new(),
            braid_id: String::new(),
        }
    }
}

/// Facade orchestrating the Nest Atomic (provenance trio + Tower crypto).
///
/// Each method corresponds to a step in the `RootPulse` `rootpulse_commit.toml`
/// lifecycle. The facade tracks which steps succeeded so callers can inspect
/// the chain even under partial availability.
pub struct NestComposition<'a> {
    ctx: &'a mut HealthCompositionContext,
    session_id: Option<String>,
    content_hash: Option<String>,
    merkle_root: Option<String>,
    signature: Option<String>,
    commit_id: Option<String>,
    braid_id: Option<String>,
    steps_attempted: u8,
    steps_succeeded: u8,
}

impl<'a> NestComposition<'a> {
    /// Create a new Nest composition bound to the given context.
    pub const fn new(ctx: &'a mut HealthCompositionContext) -> Self {
        Self {
            ctx,
            session_id: None,
            content_hash: None,
            merkle_root: None,
            signature: None,
            commit_id: None,
            braid_id: None,
            steps_attempted: 0,
            steps_succeeded: 0,
        }
    }

    /// Step 1: Begin a provenance session via rhizoCrypt `dag.session.create`.
    pub fn begin_session(&mut self, experiment: &str) -> &mut Self {
        self.steps_attempted += 1;
        match super::rhizocrypt::dag_session_create(self.ctx, experiment) {
            Ok(result) => {
                self.session_id = result
                    .get("session_id")
                    .and_then(Value::as_str)
                    .map(str::to_owned);
                self.steps_succeeded += 1;
            }
            Err(_) => {
                self.session_id = Some(format!("local-{experiment}"));
            }
        }
        self
    }

    /// Step 2: Record an event — store content via `NestGate` (`storage.store`)
    /// and append to the DAG via `rhizoCrypt` (`dag.event.append`).
    ///
    /// `NestGate` returns a BLAKE3 content hash; the hash is embedded in the
    /// DAG vertex for content-addressed integrity.
    pub fn record_event(&mut self, event_name: &str, data: &Value) -> &mut Self {
        let session_id = match &self.session_id {
            Some(id) => id.clone(),
            None => return self,
        };

        self.steps_attempted += 1;

        let content_hash = self
            .ctx
            .inner()
            .call(
                "storage",
                "storage.store",
                serde_json::json!({
                    "content": data,
                    "hash_algorithm": "blake3",
                }),
            )
            .ok()
            .and_then(|r| {
                r.get("content_hash")
                    .or_else(|| r.get("hash"))
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            });

        if let Some(ref hash) = content_hash {
            self.content_hash = Some(hash.clone());
        }

        let event_data = serde_json::json!({
            "event": event_name,
            "data": data,
            "content_hash": content_hash.as_deref().unwrap_or(""),
        });

        if let Ok(result) =
            super::rhizocrypt::dag_event_append(self.ctx, &session_id, event_name, &event_data)
        {
            self.merkle_root = result
                .get("merkle_root")
                .and_then(Value::as_str)
                .map(str::to_owned);
            self.steps_succeeded += 1;
        }

        self
    }

    /// Step 3: Sign the Merkle root via `BearDog` `crypto.sign`.
    pub fn sign_merkle(&mut self) -> &mut Self {
        let merkle = match &self.merkle_root {
            Some(root) => root.clone(),
            None => return self,
        };

        self.steps_attempted += 1;

        let message_b64 = base64::engine::general_purpose::STANDARD.encode(merkle.as_bytes());
        if let Ok(result) = self.ctx.inner().call(
            "crypto",
            "crypto.sign",
            serde_json::json!({
                "message": message_b64,
                "purpose": "provenance_commit",
            }),
        ) {
            self.signature = result
                .get("signature")
                .and_then(Value::as_str)
                .map(str::to_owned);
            self.steps_succeeded += 1;
        }

        self
    }

    /// Step 4: Commit the DAG summary to loamSpine via `commit.create`.
    pub fn commit(&mut self, experiment: &str) -> &mut Self {
        self.steps_attempted += 1;

        let commit_data = serde_json::json!({
            "session_id": self.session_id.as_deref().unwrap_or(""),
            "merkle_root": self.merkle_root.as_deref().unwrap_or(""),
            "signature": self.signature.as_deref().unwrap_or(""),
            "content_hash": self.content_hash.as_deref().unwrap_or(""),
        });

        if let Ok(result) = super::loamspine::commit_create(self.ctx, experiment, &commit_data) {
            self.commit_id = result
                .get("commit_id")
                .or_else(|| result.get("entry_id"))
                .and_then(Value::as_str)
                .map(str::to_owned);
            self.steps_succeeded += 1;
        }

        self
    }

    /// Step 5: Attribute via sweetGrass `braid.create` + `braid.commit`.
    pub fn attribute(&mut self, experiment: &str, agents: &[Agent]) -> &mut Self {
        self.steps_attempted += 1;

        let agents_json: Vec<Value> = agents
            .iter()
            .map(|a| {
                serde_json::json!({
                    "did": a.did,
                    "role": a.role,
                    "contribution": a.contribution,
                })
            })
            .collect();

        let braid_result = super::sweetgrass::braid_create(self.ctx, experiment);
        if let Ok(result) = braid_result {
            let braid_id = result
                .get("braid_id")
                .or_else(|| result.get("id"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned();

            if !braid_id.is_empty() {
                let commit_data = serde_json::json!({
                    "commit_ref": self.commit_id.as_deref().unwrap_or(""),
                    "agents": agents_json,
                    "session_id": self.session_id.as_deref().unwrap_or(""),
                });

                if super::sweetgrass::braid_commit(self.ctx, &braid_id, &commit_data).is_ok() {
                    self.braid_id = Some(braid_id);
                    self.steps_succeeded += 1;
                }
            }
        }

        self
    }

    /// Finalize and return the provenance chain.
    #[must_use]
    pub fn finalize(&self) -> NestProvenanceChain {
        let status = if self.steps_attempted == 0 {
            NestStatus::Unavailable
        } else if self.steps_succeeded == self.steps_attempted {
            NestStatus::Complete
        } else if self.steps_succeeded > 0 {
            NestStatus::Partial
        } else {
            NestStatus::Unavailable
        };

        NestProvenanceChain {
            status,
            session_id: self.session_id.clone().unwrap_or_default(),
            content_hash: self.content_hash.clone().unwrap_or_default(),
            merkle_signature: self.signature.clone().unwrap_or_default(),
            commit_id: self.commit_id.clone().unwrap_or_default(),
            braid_id: self.braid_id.clone().unwrap_or_default(),
        }
    }

    /// Convenience: run the full lifecycle in one call.
    ///
    /// Prefers Wave 17 signal dispatch (`nest.store` + `nest.commit`) when
    /// biomeOS supports it. Falls back to the manual 5-step chain when
    /// `signal.dispatch` is unavailable or returns `-32601`.
    ///
    /// Signal mapping (per `SIGNAL_ADOPTION_STANDARD.md`):
    /// - `nest.store`  → `storage.store` + `dag.event.append`
    /// - `nest.commit` → `dag.dehydrate` + `crypto.sign` + `spine.seal` + `braid.create`
    pub fn full_lifecycle(
        &mut self,
        experiment: &str,
        event_name: &str,
        data: &Value,
        agents: &[Agent],
    ) -> NestProvenanceChain {
        if let Some(chain) = self.try_signal_dispatch(experiment, event_name, data, agents) {
            return chain;
        }

        self.begin_session(experiment)
            .record_event(event_name, data)
            .sign_merkle()
            .commit(experiment)
            .attribute(experiment, agents)
            .finalize()
    }

    /// Attempt Wave 17 signal dispatch for the full lifecycle.
    ///
    /// Returns `Some(chain)` if biomeOS handled both signals successfully,
    /// `None` if signals are not available (caller should use manual chain).
    fn try_signal_dispatch(
        &mut self,
        experiment: &str,
        event_name: &str,
        data: &Value,
        agents: &[Agent],
    ) -> Option<NestProvenanceChain> {
        let agents_json: Vec<Value> = agents
            .iter()
            .map(|a| {
                serde_json::json!({
                    "did": a.did,
                    "role": a.role,
                    "contribution": a.contribution,
                })
            })
            .collect();

        let store_params = serde_json::json!({
            "experiment": experiment,
            "event": event_name,
            "content": data,
            "hash_algorithm": "blake3",
        });

        let Ok(store) = self.ctx.inner().call(
            "orchestration",
            "signal.dispatch",
            serde_json::json!({
                "signal": "nest.store",
                "params": store_params,
            }),
        ) else {
            return None;
        };

        let session_id = store
            .get("session_id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let content_hash = store
            .get("content_hash")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let merkle_root = store
            .get("merkle_root")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();

        let commit_params = serde_json::json!({
            "session_id": session_id,
            "experiment": experiment,
            "merkle_root": merkle_root,
            "agents": agents_json,
        });

        let Ok(commit) = self.ctx.inner().call(
            "orchestration",
            "signal.dispatch",
            serde_json::json!({
                "signal": "nest.commit",
                "params": commit_params,
            }),
        ) else {
            return None;
        };

        let merkle_signature = commit
            .get("signature")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let commit_id = commit
            .get("commit_id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let braid_id = commit
            .get("braid_id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();

        let filled = [&session_id, &content_hash, &merkle_signature, &commit_id, &braid_id]
            .iter()
            .filter(|s| !s.is_empty())
            .count();

        let status = if filled == 5 {
            NestStatus::Complete
        } else if filled > 0 {
            NestStatus::Partial
        } else {
            NestStatus::Unavailable
        };

        Some(NestProvenanceChain {
            status,
            session_id,
            content_hash,
            merkle_signature,
            commit_id,
            braid_id,
        })
    }
}

/// Agent descriptor for PROV-O attribution braids.
#[derive(Debug, Clone)]
pub struct Agent {
    /// Decentralized identifier (e.g. `did:key:healthSpring`).
    pub did: String,
    /// Role in the provenance (e.g. `"author"`, `"validator"`, `"reviewer"`).
    pub role: String,
    /// Fractional contribution [0.0, 1.0].
    pub contribution: f64,
}

impl Agent {
    /// Create a default healthSpring author agent.
    #[must_use]
    pub fn health_spring_author() -> Self {
        Self {
            did: "did:key:healthSpring".to_owned(),
            role: "author".to_owned(),
            contribution: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nest_unavailable_chain() {
        let chain = NestProvenanceChain::unavailable();
        assert_eq!(chain.status, NestStatus::Unavailable);
        assert!(chain.session_id.is_empty());
    }

    #[test]
    fn nest_status_display() {
        assert_eq!(NestStatus::Complete.to_string(), "complete");
        assert_eq!(NestStatus::Partial.to_string(), "partial");
        assert_eq!(NestStatus::Unavailable.to_string(), "unavailable");
    }

    #[test]
    fn agent_default() {
        let agent = Agent::health_spring_author();
        assert_eq!(agent.did, "did:key:healthSpring");
        assert_eq!(agent.role, "author");
        assert!((agent.contribution - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn nest_graceful_degradation() {
        let mut ctx = HealthCompositionContext::discover();
        let chain = NestComposition::new(&mut ctx).full_lifecycle(
            "test_experiment",
            "test_event",
            &serde_json::json!({"key": "value"}),
            &[Agent::health_spring_author()],
        );
        assert!(
            chain.status == NestStatus::Unavailable || chain.status == NestStatus::Partial,
            "Without primals running, should degrade gracefully"
        );
    }
}
