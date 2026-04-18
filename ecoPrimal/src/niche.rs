// SPDX-License-Identifier: AGPL-3.0-or-later
//! Niche self-knowledge for healthSpring.
//!
//! Follows the cross-spring pattern established by hotSpring, wetSpring,
//! airSpring, neuralSpring, ludoSpring, and groundSpring. Every constant
//! here is machine-readable self-knowledge — used by biomeOS for discovery,
//! routing, and composition validation.

/// Canonical niche identity.
pub const PRIMAL_ID: &str = "healthspring";

/// biomeOS domain this spring serves.
pub const NICHE_DOMAIN: &str = "health";

/// Proto-nucleate graph defining this spring's NUCLEUS composition target.
///
/// healthSpring's graph is kept standalone (not in `downstream_manifest.toml`)
/// because of the dual-tower ionic bridge pattern.
pub const PROTO_NUCLEATE: &str =
    "primalSpring/graphs/downstream/healthspring_enclave_proto_nucleate.toml";

/// NUCLEUS fragments this spring composes.
pub const FRAGMENTS: &[&str] = &["tower_atomic", "nest_atomic"];

/// Particle profile for this spring's composition.
pub const PARTICLE_PROFILE: &str = "neutron_heavy";

/// Bond type for cross-atomic compositions.
pub const BOND_TYPE: &str = "ionic";

/// Trust model within the composition.
pub const TRUST_MODEL: &str = "dual_tower_enclave";

/// Primals this niche depends on (germination order matters).
pub const DEPENDENCIES: &[NicheDependency] = &[
    NicheDependency {
        name: "beardog",
        role: "security",
        required: true,
        capability: "crypto",
    },
    NicheDependency {
        name: "songbird",
        role: "discovery",
        required: true,
        capability: "discovery",
    },
    NicheDependency {
        name: "nestgate",
        role: "storage",
        required: false,
        capability: "storage",
    },
    NicheDependency {
        name: "toadstool",
        role: "compute",
        required: false,
        capability: "compute",
    },
    NicheDependency {
        name: "barracuda",
        role: "math",
        required: false,
        capability: "math",
    },
    NicheDependency {
        name: "squirrel",
        role: "inference",
        required: false,
        capability: "inference",
    },
];

/// Capabilities this niche advertises to the ecosystem.
pub const CAPABILITIES: &[&str] = &[
    "health.liveness",
    "health.readiness",
    "health.check",
    "identity.get",
    "capability.list",
    "mcp.tools.list",
    "composition.health_health",
    "health.pharmacology",
    "health.genomics",
    "health.clinical",
    "health.de_identify",
    "health.aggregate",
];

/// Capabilities this niche consumes from other primals.
pub const CONSUMED_CAPABILITIES: &[&str] = &[
    "crypto.hash",
    "crypto.sign",
    "crypto.ionic_bond",
    "discovery.find_by_capability",
    "storage.store",
    "storage.retrieve",
    "storage.egress_fence",
    "compute.offload",
    "shader.compile",
    "inference.complete",
    "inference.embed",
    "inference.models",
    "dag.create_session",
    "dag.append_event",
    "dag.dehydrate",
    "commit.session",
    "provenance.create_braid",
];

/// A dependency on another primal in the ecosystem.
pub struct NicheDependency {
    /// Conventional socket-name prefix for this primal.
    pub name: &'static str,
    /// Role this primal fills in the composition.
    pub role: &'static str,
    /// Whether the spring can function without this primal.
    pub required: bool,
    /// Capability domain used for `by_capability` discovery.
    pub capability: &'static str,
}

/// Composition validation experiments.
///
/// Each entry maps an experiment binary to its validation tier:
/// - Tier 3: in-process dispatch parity (science via dispatch layer)
/// - Tier 4: live IPC parity (science via Unix socket JSON-RPC)
/// - Tier 5: NUCLEUS composition (health probes, provenance trio, deploy graph)
pub const COMPOSITION_EXPERIMENTS: &[(&str, &str)] = &[
    ("exp112_composition_pkpd", "tier3_dispatch_parity"),
    ("exp113_composition_microbiome", "tier3_dispatch_parity"),
    (
        "exp114_composition_health_triad",
        "tier3_capability_surface",
    ),
    ("exp115_composition_proto_nucleate", "tier3_structural"),
    ("exp116_composition_provenance", "tier3_provenance_session"),
    (
        "exp117_composition_ipc_roundtrip",
        "tier3_wire_serialization",
    ),
    (
        "exp118_composition_deploy_graph_validation",
        "tier3_deploy_graph",
    ),
    ("exp119_composition_live_parity", "tier4_live_ipc_parity"),
    (
        "exp120_composition_live_provenance",
        "tier4_live_provenance_trio",
    ),
    ("exp121_composition_live_health", "tier4_live_health_probes"),
];

/// Relative cost estimates (CPU milliseconds for typical inputs).
pub const COST_ESTIMATES: &[(&str, f64)] = &[
    ("science.pkpd.hill_dose_response", 0.01),
    ("science.pkpd.one_compartment_pk", 0.1),
    ("science.pkpd.population_pk", 50.0),
    ("science.pkpd.nlme_foce", 500.0),
    ("science.pkpd.nlme_saem", 800.0),
    ("science.microbiome.shannon_index", 0.01),
    ("science.microbiome.anderson_gut", 5.0),
    ("science.biosignal.pan_tompkins", 1.0),
    ("science.biosignal.arrhythmia_classification", 2.0),
    ("science.diagnostic.assess_patient", 10.0),
    ("science.diagnostic.population_montecarlo", 100.0),
];

/// Operation dependencies for biomeOS Pathway Learner.
pub const OPERATION_DEPENDENCIES: &[(&str, &[&str])] = &[
    (
        "science.diagnostic.assess_patient",
        &[
            "science.pkpd.one_compartment_pk",
            "science.microbiome.shannon_index",
            "science.microbiome.anderson_gut",
            "science.biosignal.hrv_metrics",
            "science.endocrine.testosterone_pk",
        ],
    ),
    (
        "science.diagnostic.population_montecarlo",
        &["science.diagnostic.assess_patient"],
    ),
    (
        "science.clinical.trt_scenario",
        &[
            "science.endocrine.testosterone_pk",
            "science.endocrine.trt_outcomes",
            "science.biosignal.hrv_metrics",
        ],
    ),
    (
        "science.biosignal.fuse_channels",
        &[
            "science.biosignal.pan_tompkins",
            "science.biosignal.ppg_spo2",
            "science.biosignal.eda_analysis",
        ],
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primal_identity_consistent() {
        assert_eq!(PRIMAL_ID, crate::PRIMAL_NAME);
        assert_eq!(NICHE_DOMAIN, crate::PRIMAL_DOMAIN);
    }

    #[test]
    fn fragments_match_proto_nucleate() {
        assert!(FRAGMENTS.contains(&"tower_atomic"));
        assert!(FRAGMENTS.contains(&"nest_atomic"));
        assert_eq!(FRAGMENTS.len(), 2);
    }

    #[test]
    fn dependencies_include_tower_atomic() {
        let names: Vec<&str> = DEPENDENCIES.iter().map(|d| d.name).collect();
        assert!(names.contains(&"beardog"));
        assert!(names.contains(&"songbird"));
    }

    #[test]
    fn required_deps_are_tower_only() {
        let required: Vec<&str> = DEPENDENCIES
            .iter()
            .filter(|d| d.required)
            .map(|d| d.name)
            .collect();
        assert_eq!(required, vec!["beardog", "songbird"]);
    }

    #[test]
    fn capabilities_include_probes() {
        assert!(CAPABILITIES.contains(&"health.liveness"));
        assert!(CAPABILITIES.contains(&"health.readiness"));
        assert!(CAPABILITIES.contains(&"capability.list"));
    }

    #[test]
    fn cost_estimates_all_positive() {
        for (method, cost) in COST_ESTIMATES {
            assert!(*cost > 0.0, "{method} has non-positive cost");
        }
    }

    #[test]
    fn composition_experiments_cover_all_tiers() {
        let tiers: Vec<&str> = COMPOSITION_EXPERIMENTS.iter().map(|(_, t)| *t).collect();
        assert!(
            tiers.iter().any(|t| t.starts_with("tier3")),
            "must have tier3 experiments"
        );
        assert!(
            tiers.iter().any(|t| t.starts_with("tier4")),
            "must have tier4 experiments"
        );
    }
}
