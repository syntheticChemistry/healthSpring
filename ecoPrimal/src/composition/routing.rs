// SPDX-License-Identifier: AGPL-3.0-or-later

//! Health-domain capability-to-primal routing.

use crate::primal_names;

/// All capabilities that healthSpring's NUCLEUS composition may use.
pub const ALL_CAPS: &[&str] = &[
    "tensor",
    "shader",
    "compute",
    "security",
    "discovery",
    "storage",
    "dag",
    "commit",
    "braid",
    "inference",
    "visualization",
    "orchestration",
    "coordination",
];

/// Map a capability domain to its canonical provider primal.
#[must_use]
pub fn capability_to_primal(capability: &str) -> &'static str {
    match capability {
        "tensor" | "stats" => primal_names::BARRACUDA,
        "shader" => primal_names::CORALREEF,
        "compute" => primal_names::TOADSTOOL,
        "security" | "crypto" => primal_names::BEARDOG,
        "discovery" | "net.discovery" => primal_names::SONGBIRD,
        "storage" => primal_names::NESTGATE,
        "dag" => primal_names::RHIZOCRYPT,
        "commit" | "ledger" | "spine" | "merkle" => primal_names::LOAMSPINE,
        "braid" | "attribution" => primal_names::SWEETGRASS,
        "inference" | "model" => primal_names::SQUIRREL,
        "visualization" => primal_names::PETALTONGUE,
        "orchestration" | "lifecycle" => primal_names::BIOMEOS,
        "coordination" => "primalspring",
        _ => "unknown",
    }
}
