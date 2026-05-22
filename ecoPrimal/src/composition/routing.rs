// SPDX-License-Identifier: AGPL-3.0-or-later

//! Health-domain capability-to-primal routing.

use crate::primal_names;

/// All capabilities that healthSpring's NUCLEUS composition may use.
pub const ALL_CAPS: &[&str] = &[
    "tensor",
    "stats",
    "shader",
    "compute",
    "security",
    "discovery",
    "storage",
    "content",
    "dag",
    "commit",
    "braid",
    "inference",
    "visualization",
    "orchestration",
    "coordination",
    "bonding",
    "audit",
    "signal",
    "certificate",
    "genetic",
    "fido2",
    "primal",
];

/// Map a capability domain to its canonical provider primal.
#[must_use]
pub fn capability_to_primal(capability: &str) -> &'static str {
    match capability {
        "tensor" | "stats" => primal_names::BARRACUDA,
        "shader" => primal_names::CORALREEF,
        "compute" => primal_names::TOADSTOOL,
        "security" | "crypto" | "fido2" => primal_names::BEARDOG,
        "discovery" | "net.discovery" => primal_names::SONGBIRD,
        "storage" | "content" => primal_names::NESTGATE,
        "dag" => primal_names::RHIZOCRYPT,
        "commit" | "ledger" | "spine" | "merkle" => primal_names::LOAMSPINE,
        "braid" | "attribution" => primal_names::SWEETGRASS,
        "inference" | "model" => primal_names::SQUIRREL,
        "visualization" => primal_names::PETALTONGUE,
        "orchestration" | "lifecycle" | "signal" => primal_names::BIOMEOS,
        "coordination" | "bonding" | "primal" => "primalspring",
        "audit" | "audit.log" | "defense" | "security.audit" => primal_names::SKUNKBAT,
        "certificate" | "genetic" => "ecosystem",
        _ => "unknown",
    }
}
