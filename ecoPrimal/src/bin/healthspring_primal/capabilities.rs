// SPDX-License-Identifier: AGPL-3.0-or-later

//! Capability definitions and semantic mappings for the healthSpring primal.

pub use healthspring_barracuda::PRIMAL_DOMAIN;
pub use healthspring_barracuda::PRIMAL_NAME;

/// Every capability this primal advertises to `biomeOS`.
pub const ALL_CAPABILITIES: &[&str] = &[
    // ── PK/PD ────────────────────────────────────────────────────────
    "science.pkpd.hill_dose_response",
    "science.pkpd.one_compartment_pk",
    "science.pkpd.two_compartment_pk",
    "science.pkpd.pbpk_simulate",
    "science.pkpd.population_pk",
    "science.pkpd.allometric_scale",
    "science.pkpd.auc_trapezoidal",
    "science.pkpd.nlme_foce",
    "science.pkpd.nlme_saem",
    "science.pkpd.nca_analysis",
    "science.pkpd.cwres_diagnostics",
    "science.pkpd.vpc_simulate",
    "science.pkpd.gof_compute",
    "science.pkpd.michaelis_menten_nonlinear",
    // ── Microbiome ───────────────────────────────────────────────────
    "science.microbiome.shannon_index",
    "science.microbiome.simpson_index",
    "science.microbiome.pielou_evenness",
    "science.microbiome.chao1",
    "science.microbiome.anderson_gut",
    "science.microbiome.colonization_resistance",
    "science.microbiome.fmt_blend",
    "science.microbiome.bray_curtis",
    "science.microbiome.antibiotic_perturbation",
    "science.microbiome.scfa_production",
    "science.microbiome.gut_brain_serotonin",
    "science.microbiome.qs_gene_profile",
    "science.microbiome.qs_effective_disorder",
    // ── Biosignal ────────────────────────────────────────────────────
    "science.biosignal.pan_tompkins",
    "science.biosignal.hrv_metrics",
    "science.biosignal.ppg_spo2",
    "science.biosignal.eda_analysis",
    "science.biosignal.eda_stress_detection",
    "science.biosignal.arrhythmia_classification",
    "science.biosignal.fuse_channels",
    "science.biosignal.wfdb_decode",
    // ── Endocrine ────────────────────────────────────────────────────
    "science.endocrine.testosterone_pk",
    "science.endocrine.trt_outcomes",
    "science.endocrine.population_trt",
    "science.endocrine.hrv_trt_response",
    "science.endocrine.cardiac_risk",
    // ── Diagnostic ───────────────────────────────────────────────────
    "science.diagnostic.assess_patient",
    "science.diagnostic.population_montecarlo",
    "science.diagnostic.composite_risk",
    // ── Clinical ─────────────────────────────────────────────────────
    "science.clinical.trt_scenario",
    "science.clinical.patient_parameterize",
    "science.clinical.risk_annotate",
    // ── Comparative ──────────────────────────────────────────────────
    "science.comparative.cross_species_pk",
    "science.comparative.canine_il31",
    "science.comparative.canine_jak1",
    // ── Discovery ────────────────────────────────────────────────────
    "science.discovery.matrix_score",
    "science.discovery.hts_analysis",
    "science.discovery.compound_library",
    "science.discovery.fibrosis_pathway",
    // ── Toxicology ───────────────────────────────────────────────────
    "science.toxicology.biphasic_dose_response",
    "science.toxicology.toxicity_landscape",
    "science.toxicology.hormetic_optimum",
    // ── Simulation ───────────────────────────────────────────────────
    "science.simulation.mechanistic_fitness",
    "science.simulation.ecosystem_simulate",
    // ── Proto-nucleate aliases (NUCLEUS_SPRING_ALIGNMENT.md) ──────────
    // Maps proto-nucleate `health.*` capabilities to the science surface.
    "health.pharmacology",
    "health.genomics",
    "health.clinical",
    "health.de_identify",
    "health.aggregate",
    // ── Composition health (COMPOSITION_HEALTH_STANDARD.md) ──────────
    "composition.health_health",
    // ── Provenance trio (`biomeOS` composition) ──────────────────────
    "provenance.begin",
    "provenance.record",
    "provenance.complete",
    "provenance.status",
    // ── Cross-primal ─────────────────────────────────────────────────
    "primal.forward",
    "primal.discover",
    // ── Health probes (DEPLOYMENT_VALIDATION_STANDARD alignment) ─────
    "health.liveness",
    "health.readiness",
    "health.check",
    // ── Identity (Capability Wire Standard April 2026) ───────────────
    "identity.get",
    // ── Niche deployment (`biomeOS` graph composition) ───────────────
    "capability.list",
    "mcp.tools.list",
    // ── Compute offload (Node Atomic) ────────────────────────────────
    "compute.offload",
    "compute.shader_compile", // coralReef coordination
    // ── Data (`NestGate` routing) ────────────────────────────────────
    "data.fetch",
    // ── Inference (Squirrel / neuralSpring coordination) ─────────────
    "model.inference_route",
    "inference.complete",
    "inference.embed",
    "inference.models",
    "inference.route",
];

/// Build semantic mappings for capability registration with biomeOS.
///
/// Maps short method names to fully-qualified `domain.verb` methods.
/// Built from `ALL_CAPABILITIES` to stay in sync automatically.
pub fn build_semantic_mappings() -> serde_json::Value {
    let mut map = serde_json::Map::with_capacity(ALL_CAPABILITIES.len());
    for cap in ALL_CAPABILITIES {
        if let Some(short) = cap.rsplit('.').next() {
            map.insert(
                short.to_string(),
                serde_json::Value::String((*cap).to_string()),
            );
        }
    }
    serde_json::Value::Object(map)
}

/// Capabilities served locally by this primal (dispatched in-process).
pub const LOCAL_CAPABILITIES: &[&str] = &[
    "science.pkpd.hill_dose_response",
    "science.pkpd.one_compartment_pk",
    "science.pkpd.two_compartment_pk",
    "science.pkpd.pbpk_simulate",
    "science.pkpd.population_pk",
    "science.pkpd.allometric_scale",
    "science.pkpd.auc_trapezoidal",
    "science.pkpd.nlme_foce",
    "science.pkpd.nlme_saem",
    "science.pkpd.nca_analysis",
    "science.pkpd.cwres_diagnostics",
    "science.pkpd.vpc_simulate",
    "science.pkpd.gof_compute",
    "science.pkpd.michaelis_menten_nonlinear",
    "science.microbiome.shannon_index",
    "science.microbiome.simpson_index",
    "science.microbiome.pielou_evenness",
    "science.microbiome.chao1",
    "science.microbiome.anderson_gut",
    "science.microbiome.colonization_resistance",
    "science.microbiome.fmt_blend",
    "science.microbiome.bray_curtis",
    "science.microbiome.antibiotic_perturbation",
    "science.microbiome.scfa_production",
    "science.microbiome.gut_brain_serotonin",
    "science.microbiome.qs_gene_profile",
    "science.microbiome.qs_effective_disorder",
    "science.biosignal.pan_tompkins",
    "science.biosignal.hrv_metrics",
    "science.biosignal.ppg_spo2",
    "science.biosignal.eda_analysis",
    "science.biosignal.eda_stress_detection",
    "science.biosignal.arrhythmia_classification",
    "science.biosignal.fuse_channels",
    "science.biosignal.wfdb_decode",
    "science.endocrine.testosterone_pk",
    "science.endocrine.trt_outcomes",
    "science.endocrine.population_trt",
    "science.endocrine.hrv_trt_response",
    "science.endocrine.cardiac_risk",
    "science.diagnostic.assess_patient",
    "science.diagnostic.population_montecarlo",
    "science.diagnostic.composite_risk",
    "science.clinical.trt_scenario",
    "science.clinical.patient_parameterize",
    "science.clinical.risk_annotate",
    "science.comparative.cross_species_pk",
    "science.comparative.canine_il31",
    "science.comparative.canine_jak1",
    "science.discovery.matrix_score",
    "science.discovery.hts_analysis",
    "science.discovery.compound_library",
    "science.discovery.fibrosis_pathway",
    "science.toxicology.biphasic_dose_response",
    "science.toxicology.toxicity_landscape",
    "science.toxicology.hormetic_optimum",
    "science.simulation.mechanistic_fitness",
    "science.simulation.ecosystem_simulate",
    "health.pharmacology",
    "health.genomics",
    "health.clinical",
    "health.de_identify",
    "health.aggregate",
    "composition.health_health",
    "provenance.begin",
    "provenance.record",
    "provenance.complete",
    "provenance.status",
    "primal.forward",
    "primal.discover",
    "health.liveness",
    "health.readiness",
    "health.check",
    "identity.get",
    "capability.list",
    "mcp.tools.list",
];

/// Capabilities routed to external providers via IPC (not served locally).
///
/// Each entry maps a capability method to the **capability domain** used for
/// `by_capability` discovery at runtime — never a hardcoded primal identity.
/// The discovery layer resolves the domain to whichever primal currently
/// advertises it (e.g., `"compute"` resolves to toadStool today, but could
/// resolve to any primal that registers compute capabilities in the future).
pub const ROUTED_CAPABILITIES: &[(&str, &str)] = &[
    ("compute.offload", "compute"),
    ("compute.shader_compile", "shader"),
    ("data.fetch", "storage"),
    ("model.inference_route", "inference"),
    ("inference.complete", "inference"),
    ("inference.embed", "inference"),
    ("inference.models", "inference"),
    ("inference.route", "inference"),
];

/// Capability listing per Capability Wire Standard (April 2026).
///
/// Response includes the canonical `methods` flat array for biomeOS O(1)
/// discovery, plus enriched `science` / `infrastructure` groupings and
/// Pathway Learner metadata (`operation_dependencies`, `cost_estimates`).
pub fn handle_capability_list() -> serde_json::Value {
    let methods: Vec<&str> = ALL_CAPABILITIES.to_vec();

    let science: Vec<&str> = ALL_CAPABILITIES
        .iter()
        .filter(|c| c.starts_with("science."))
        .copied()
        .collect();
    let infra: Vec<&str> = ALL_CAPABILITIES
        .iter()
        .filter(|c| {
            c.starts_with("primal.")
                || c.starts_with("compute.")
                || c.starts_with("data.")
                || c.starts_with("model.")
                || c.starts_with("inference.")
                || c.starts_with("capability.")
                || c.starts_with("provenance.")
                || c.starts_with("composition.")
                || c.starts_with("health.")
                || c.starts_with("identity.")
                || c.starts_with("mcp.")
        })
        .copied()
        .collect();

    serde_json::json!({
        "primal": PRIMAL_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "domain": PRIMAL_DOMAIN,
        "methods": methods,
        "total": ALL_CAPABILITIES.len(),
        "science": science,
        "infrastructure": infra,
        "provided_capabilities": provided_capabilities(),
        "operation_dependencies": operation_dependencies(),
        "cost_estimates": cost_estimates(),
    })
}

/// Identity probe per Capability Discovery Standard.
pub fn handle_identity_get() -> serde_json::Value {
    serde_json::json!({
        "primal": PRIMAL_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "domain": PRIMAL_DOMAIN,
        "license": "AGPL-3.0-or-later",
        "architecture": "ecoBin",
        "composition_model": "nucleated",
        "particle_profile": "neutron_heavy",
        "proto_nucleate": "healthspring_enclave_proto_nucleate",
    })
}

/// Structured capability groupings (local vs routed) for biomeOS.
fn provided_capabilities() -> serde_json::Value {
    let local: Vec<serde_json::Value> = LOCAL_CAPABILITIES
        .iter()
        .map(|c| {
            serde_json::json!({
                "method": c,
                "served_locally": true,
            })
        })
        .collect();

    let routed: Vec<serde_json::Value> = ROUTED_CAPABILITIES
        .iter()
        .map(|(method, capability_domain)| {
            serde_json::json!({
                "method": method,
                "served_locally": false,
                "by_capability": capability_domain,
            })
        })
        .collect();

    let mut all = local;
    all.extend(routed);
    serde_json::Value::Array(all)
}

/// Dependency graph between science operations for Pathway Learner.
fn operation_dependencies() -> serde_json::Value {
    serde_json::json!({
        "science.diagnostic.assess_patient": [
            "science.pkpd.one_compartment_pk",
            "science.microbiome.shannon_index",
            "science.microbiome.anderson_gut",
            "science.biosignal.hrv_metrics",
            "science.endocrine.testosterone_pk",
        ],
        "science.diagnostic.population_montecarlo": [
            "science.diagnostic.assess_patient",
        ],
        "science.diagnostic.composite_risk": [
            "science.diagnostic.assess_patient",
        ],
        "science.clinical.trt_scenario": [
            "science.endocrine.testosterone_pk",
            "science.endocrine.trt_outcomes",
            "science.biosignal.hrv_metrics",
        ],
        "science.pkpd.nlme_foce": [
            "science.pkpd.population_pk",
        ],
        "science.pkpd.nlme_saem": [
            "science.pkpd.population_pk",
        ],
        "science.pkpd.vpc_simulate": [
            "science.pkpd.nlme_foce",
        ],
        "science.microbiome.gut_brain_serotonin": [
            "science.microbiome.shannon_index",
        ],
        "science.biosignal.fuse_channels": [
            "science.biosignal.pan_tompkins",
            "science.biosignal.ppg_spo2",
            "science.biosignal.eda_analysis",
        ],
    })
}

/// Relative cost estimates (CPU milliseconds for typical inputs).
fn cost_estimates() -> serde_json::Value {
    serde_json::json!({
        "science.pkpd.hill_dose_response":            {"cpu_ms": 0.01, "gpu_eligible": true},
        "science.pkpd.one_compartment_pk":             {"cpu_ms": 0.1,  "gpu_eligible": false},
        "science.pkpd.population_pk":                  {"cpu_ms": 50.0, "gpu_eligible": true},
        "science.pkpd.nlme_foce":                      {"cpu_ms": 500.0, "gpu_eligible": false},
        "science.pkpd.nlme_saem":                      {"cpu_ms": 800.0, "gpu_eligible": false},
        "science.microbiome.shannon_index":            {"cpu_ms": 0.01, "gpu_eligible": true},
        "science.microbiome.anderson_gut":             {"cpu_ms": 5.0,  "gpu_eligible": false},
        "science.biosignal.pan_tompkins":              {"cpu_ms": 1.0,  "gpu_eligible": false},
        "science.biosignal.arrhythmia_classification": {"cpu_ms": 2.0,  "gpu_eligible": true},
        "science.diagnostic.assess_patient":           {"cpu_ms": 10.0, "gpu_eligible": false},
        "science.diagnostic.population_montecarlo":    {"cpu_ms": 100.0, "gpu_eligible": true},
    })
}

/// Subcommand: version
pub fn cmd_version() {
    println!("{PRIMAL_NAME} {}", env!("CARGO_PKG_VERSION"));
    println!("  Domain:    {PRIMAL_DOMAIN}");
    println!("  License:   AGPL-3.0-or-later");
    println!("  Arch:      UniBin / ecoBin v3.0");
    println!("  Runtime:   {}", std::env::consts::ARCH);
}

/// Subcommand: capabilities
pub fn cmd_capabilities() {
    let science_count = ALL_CAPABILITIES
        .iter()
        .filter(|c| c.starts_with("science."))
        .count();
    let infra_count = ALL_CAPABILITIES.len() - science_count;

    println!("{PRIMAL_NAME} — {science_count} science + {infra_count} infrastructure capabilities");
    println!();
    for cap in ALL_CAPABILITIES {
        println!("  {cap}");
    }
}
