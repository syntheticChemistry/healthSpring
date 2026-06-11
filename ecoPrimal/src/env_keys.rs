// SPDX-License-Identifier: AGPL-3.0-or-later

//! Centralized environment variable keys for healthSpring.
//!
//! Mirrors the primalSpring `env_keys` pattern. All env var names used
//! across the codebase are defined here as constants so typos are caught
//! at compile time and `grep` returns a single canonical source.
#![allow(missing_docs, reason = "env var constants are self-documenting")]

// ── Gate / mesh identity ────────────────────────────────────────────

pub const GATE_NAME: &str = "GATE_NAME";
pub const GATE_ID: &str = "GATE_ID";
pub const FAMILY_SEED: &str = "FAMILY_SEED";
pub const FAMILY_ID: &str = "FAMILY_ID";
pub const BIOMEOS_FAMILY_ID: &str = "BIOMEOS_FAMILY_ID";
pub const HEALTHSPRING_FAMILY_ID: &str = "HEALTHSPRING_FAMILY_ID";

// ── Socket directories ─────────────────────────────────────────────

pub const BIOMEOS_SOCKET_DIR: &str = "BIOMEOS_SOCKET_DIR";
pub const XDG_RUNTIME_DIR: &str = "XDG_RUNTIME_DIR";

// ── Primal socket overrides ─────────────────────────────────────────

pub const HEALTHSPRING_SOCKET: &str = "HEALTHSPRING_SOCKET";
pub const HEALTHSPRING_PORT: &str = "HEALTHSPRING_PORT";
pub const BIOMEOS_ORCHESTRATOR_SOCKET: &str = "BIOMEOS_ORCHESTRATOR_SOCKET";
pub const BIOMEOS_SOCKET: &str = "BIOMEOS_SOCKET";
pub const PRIMALSPRING_SOCKET: &str = "PRIMALSPRING_SOCKET";
pub const PETALTONGUE_SOCKET: &str = "PETALTONGUE_SOCKET";
pub const BIOMEOS_FALLBACK_PRIMAL: &str = "BIOMEOS_FALLBACK_PRIMAL";

// ── Capability-based primal discovery ───────────────────────────────

pub const HEALTHSPRING_COMPUTE_PRIMAL: &str = "HEALTHSPRING_COMPUTE_PRIMAL";
pub const HEALTHSPRING_DATA_PRIMAL: &str = "HEALTHSPRING_DATA_PRIMAL";
pub const HEALTHSPRING_SHADER_PRIMAL: &str = "HEALTHSPRING_SHADER_PRIMAL";
pub const HEALTHSPRING_INFERENCE_PRIMAL: &str = "HEALTHSPRING_INFERENCE_PRIMAL";
pub const HEALTHSPRING_EPHEMERAL_PRIMAL: &str = "HEALTHSPRING_EPHEMERAL_PRIMAL";
pub const HEALTHSPRING_PERMANENCE_PRIMAL: &str = "HEALTHSPRING_PERMANENCE_PRIMAL";
pub const HEALTHSPRING_ATTRIBUTION_PRIMAL: &str = "HEALTHSPRING_ATTRIBUTION_PRIMAL";
pub const HEALTHSPRING_DATA_SOCKET: &str = "HEALTHSPRING_DATA_SOCKET";
pub const HEALTHSPRING_DATA_PROVIDER: &str = "HEALTHSPRING_DATA_PROVIDER";

// ── Data / storage ──────────────────────────────────────────────────

pub const HEALTHSPRING_COLD_STORAGE: &str = "HEALTHSPRING_COLD_STORAGE";
pub const HEALTHSPRING_DATA_ROOT: &str = "HEALTHSPRING_DATA_ROOT";
pub const NCBI_API_KEY: &str = "NCBI_API_KEY";
pub const HEALTHSPRING_NCBI_EUTILS_BASE: &str = "HEALTHSPRING_NCBI_EUTILS_BASE";
pub const HEALTHSPRING_NCBI_SRA_BASE: &str = "HEALTHSPRING_NCBI_SRA_BASE";

// ── Data provider socket prefixes ───────────────────────────────────

pub const DATA_PROVIDER_SOCK_PREFIX: &str = "DATA_PROVIDER_SOCK_PREFIX";
pub const DATA_PROVIDER_SOCKET: &str = "DATA_PROVIDER_SOCKET";

// ── Helpers ─────────────────────────────────────────────────────────

/// Read the local gate identity from `GATE_NAME` or `GATE_ID`.
#[must_use]
pub fn gate_identity() -> Option<String> {
    std::env::var(GATE_NAME)
        .or_else(|_| std::env::var(GATE_ID))
        .ok()
        .filter(|s| !s.is_empty())
}

/// Whether `FAMILY_SEED` is set (indicates BTSP/auth-capable deployment).
#[must_use]
pub fn family_seed_active() -> bool {
    std::env::var(FAMILY_SEED).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_identity_returns_none_when_unset() {
        if std::env::var(GATE_NAME).is_err() && std::env::var(GATE_ID).is_err() {
            assert!(gate_identity().is_none());
        }
    }

    #[test]
    fn constants_are_nonempty() {
        let all = [
            GATE_NAME, GATE_ID, FAMILY_SEED, FAMILY_ID,
            BIOMEOS_SOCKET_DIR, XDG_RUNTIME_DIR,
            HEALTHSPRING_SOCKET, HEALTHSPRING_PORT,
            HEALTHSPRING_COMPUTE_PRIMAL, HEALTHSPRING_DATA_PRIMAL,
        ];
        for key in &all {
            assert!(!key.is_empty());
        }
    }
}
