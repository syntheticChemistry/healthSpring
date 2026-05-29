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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_caps_entries_route_to_known_primal() {
        for cap in ALL_CAPS {
            let primal = capability_to_primal(cap);
            assert_ne!(
                primal, "unknown",
                "ALL_CAPS entry '{cap}' routes to 'unknown' — missing match arm"
            );
        }
    }

    #[test]
    fn all_caps_has_no_duplicates() {
        let mut seen = std::collections::HashSet::new();
        for cap in ALL_CAPS {
            assert!(
                seen.insert(cap),
                "ALL_CAPS contains duplicate: '{cap}'"
            );
        }
    }

    #[test]
    fn canonical_routing_table() {
        assert_eq!(capability_to_primal("tensor"), primal_names::BARRACUDA);
        assert_eq!(capability_to_primal("stats"), primal_names::BARRACUDA);
        assert_eq!(capability_to_primal("shader"), primal_names::CORALREEF);
        assert_eq!(capability_to_primal("compute"), primal_names::TOADSTOOL);
        assert_eq!(capability_to_primal("security"), primal_names::BEARDOG);
        assert_eq!(capability_to_primal("crypto"), primal_names::BEARDOG);
        assert_eq!(capability_to_primal("fido2"), primal_names::BEARDOG);
        assert_eq!(capability_to_primal("discovery"), primal_names::SONGBIRD);
        assert_eq!(capability_to_primal("net.discovery"), primal_names::SONGBIRD);
        assert_eq!(capability_to_primal("storage"), primal_names::NESTGATE);
        assert_eq!(capability_to_primal("content"), primal_names::NESTGATE);
        assert_eq!(capability_to_primal("dag"), primal_names::RHIZOCRYPT);
        assert_eq!(capability_to_primal("commit"), primal_names::LOAMSPINE);
        assert_eq!(capability_to_primal("ledger"), primal_names::LOAMSPINE);
        assert_eq!(capability_to_primal("spine"), primal_names::LOAMSPINE);
        assert_eq!(capability_to_primal("merkle"), primal_names::LOAMSPINE);
        assert_eq!(capability_to_primal("braid"), primal_names::SWEETGRASS);
        assert_eq!(capability_to_primal("attribution"), primal_names::SWEETGRASS);
        assert_eq!(capability_to_primal("inference"), primal_names::SQUIRREL);
        assert_eq!(capability_to_primal("model"), primal_names::SQUIRREL);
        assert_eq!(capability_to_primal("visualization"), primal_names::PETALTONGUE);
        assert_eq!(capability_to_primal("orchestration"), primal_names::BIOMEOS);
        assert_eq!(capability_to_primal("lifecycle"), primal_names::BIOMEOS);
        assert_eq!(capability_to_primal("signal"), primal_names::BIOMEOS);
        assert_eq!(capability_to_primal("coordination"), "primalspring");
        assert_eq!(capability_to_primal("bonding"), "primalspring");
        assert_eq!(capability_to_primal("primal"), "primalspring");
        assert_eq!(capability_to_primal("audit"), primal_names::SKUNKBAT);
        assert_eq!(capability_to_primal("audit.log"), primal_names::SKUNKBAT);
        assert_eq!(capability_to_primal("defense"), primal_names::SKUNKBAT);
        assert_eq!(capability_to_primal("security.audit"), primal_names::SKUNKBAT);
        assert_eq!(capability_to_primal("certificate"), "ecosystem");
        assert_eq!(capability_to_primal("genetic"), "ecosystem");
    }

    #[test]
    fn unknown_capability_returns_unknown() {
        assert_eq!(capability_to_primal("nonexistent"), "unknown");
        assert_eq!(capability_to_primal(""), "unknown");
        assert_eq!(capability_to_primal("health.liveness"), "unknown");
    }
}
