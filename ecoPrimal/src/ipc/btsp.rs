// SPDX-License-Identifier: AGPL-3.0-or-later
//! BTSP (`BearDog` Transport Security Protocol) client handshake.
//!
//! Implements the 4-step handshake for authenticated primal connections
//! per primalSpring's `btsp_handshake.rs` pattern:
//!
//! 1. **`ClientHello`** — primal name + family ID
//! 2. **`ServerHello`** — server nonce
//! 3. **`ChallengeResponse`** — HMAC over nonce using family seed (HKDF-derived)
//! 4. **`HandshakeComplete`** — success / failure
//!
//! The family seed is read from `FAMILY_SEED` (base64-encoded). When the
//! seed is not set, BTSP is skipped (standalone mode) — the handshake is
//! never required for the primal to start.

use serde::{Deserialize, Serialize};

use crate::ipc::socket;

/// BTSP handshake message types.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BtspMessage {
    /// Step 1: client announces identity.
    ClientHello {
        /// Primal name (e.g. `healthspring`).
        primal: String,
        /// Family identifier for trust boundary.
        family_id: String,
    },
    /// Step 2: server responds with challenge nonce.
    ServerHello {
        /// Server-generated nonce for HMAC challenge.
        nonce: String,
    },
    /// Step 3: client proves family membership.
    ChallengeResponse {
        /// HMAC over server nonce using HKDF-derived key.
        hmac: String,
    },
    /// Step 4: server confirms or rejects.
    HandshakeComplete {
        /// Whether the handshake was accepted.
        accepted: bool,
        /// Rejection reason (if any).
        reason: Option<String>,
    },
}

/// Read the base64-encoded family seed from environment.
///
/// Returns `None` in standalone mode (no `FAMILY_SEED` set), which is
/// the normal case for development and single-family deployments.
#[must_use]
pub fn family_seed_from_env() -> Option<Vec<u8>> {
    use base64_decode;

    std::env::var("FAMILY_SEED")
        .ok()
        .and_then(|s| base64_decode(&s))
}

/// Build a `ClientHello` message for this primal.
#[must_use]
pub fn client_hello() -> BtspMessage {
    BtspMessage::ClientHello {
        primal: crate::PRIMAL_NAME.to_owned(),
        family_id: socket::get_family_id(),
    }
}

/// Whether BTSP is available (family seed is configured).
#[must_use]
pub fn btsp_available() -> bool {
    std::env::var("FAMILY_SEED").is_ok()
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "value is masked to 8 bits before cast"
)]
fn base64_decode(input: &str) -> Option<Vec<u8>> {
    let input = input.trim();
    let mut buf = Vec::with_capacity(input.len());
    let mut accum: u32 = 0;
    let mut bits: u32 = 0;

    for byte in input.bytes() {
        let val = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' | b' ' | b'\n' | b'\r' | b'\t' => continue,
            _ => return None,
        };
        accum = (accum << 6) | u32::from(val);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            buf.push((accum >> bits) as u8);
            accum &= (1 << bits) - 1;
        }
    }
    Some(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_hello_contains_primal_name() {
        let msg = client_hello();
        match msg {
            BtspMessage::ClientHello { primal, .. } => {
                assert_eq!(primal, "healthspring");
            }
            _ => panic!("expected ClientHello"),
        }
    }

    #[test]
    fn btsp_available_reflects_env() {
        if std::env::var("FAMILY_SEED").is_err() {
            assert!(!btsp_available());
        }
    }

    #[test]
    fn base64_decode_works() {
        let decoded = base64_decode("aGVsbG8=");
        assert_eq!(decoded, Some(b"hello".to_vec()));
    }

    #[test]
    fn base64_decode_empty() {
        let decoded = base64_decode("");
        assert_eq!(decoded, Some(vec![]));
    }

    #[test]
    fn base64_decode_invalid_char() {
        let decoded = base64_decode("abc!");
        assert_eq!(decoded, None);
    }

    #[test]
    fn handshake_message_serializes() {
        let hello = client_hello();
        let json = serde_json::to_string(&hello);
        assert!(json.is_ok());
        let json_str = json.expect("serialization should succeed in test");
        assert!(json_str.contains("ClientHello"));
    }
}
