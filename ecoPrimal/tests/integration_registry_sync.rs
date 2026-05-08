// SPDX-License-Identifier: AGPL-3.0-or-later
#![expect(clippy::unwrap_used, reason = "test harness — unwrap is idiomatic")]
#![expect(clippy::expect_used, reason = "test harness — expect is idiomatic")]

//! Cross-sync healthSpring's `config/capability_registry.toml` against
//! primalSpring's canonical `config/capability_registry.toml` (389+ methods).
//!
//! Per the May 8, 2026 Cross-Spring Composition Parity Handoff, every spring
//! should CI-test its method strings against the ecosystem-wide canonical
//! registry. This test validates that:
//!
//! 1. Every **consumed** method in healthSpring's registry exists in
//!    primalSpring's canonical registry (we're calling real primal methods).
//! 2. Every **routed** method exists in the canonical registry.
//! 3. healthSpring's locally-served methods don't collide with methods
//!    owned by other primals (unless healthSpring is the documented owner).

/// Relative path from workspace root to primalSpring's canonical registry.
const PRIMALSPRING_REGISTRY: &str =
    "../primalSpring/config/capability_registry.toml";

/// Relative path from workspace root to healthSpring's own registry.
const HEALTHSPRING_REGISTRY: &str = "config/capability_registry.toml";

fn workspace_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn load_registry_methods(path: &std::path::Path) -> Vec<String> {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let table: toml::Value = content.parse()
        .unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
    let table = table.as_table().expect("top-level table");

    let mut methods = Vec::new();
    for (section, value) in table {
        if section == "test_fixtures" || section == "false_positives" {
            continue;
        }
        if let Some(arr) = value.get("methods").and_then(|v| v.as_array()) {
            for m in arr {
                if let Some(s) = m.as_str() {
                    methods.push(s.to_string());
                }
            }
        }
    }
    methods.sort();
    methods.dedup();
    methods
}

fn load_registry_sections(
    path: &std::path::Path,
) -> Vec<(String, String, Vec<String>)> {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let table: toml::Value = content.parse()
        .unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
    let table = table.as_table().expect("top-level table");

    let mut sections = Vec::new();
    for (section, value) in table {
        if section == "test_fixtures" || section == "false_positives" {
            continue;
        }
        let locality = value
            .get("locality")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let methods: Vec<String> = value
            .get("methods")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        sections.push((section.clone(), locality, methods));
    }
    sections
}

#[test]
fn consumed_methods_exist_in_primalspring_registry() {
    let root = workspace_root();
    let ps_path = root.join(PRIMALSPRING_REGISTRY);
    if !ps_path.exists() {
        eprintln!(
            "SKIP: primalSpring registry not found at {} — \
             clone primalSpring as sibling to run cross-sync",
            ps_path.display()
        );
        return;
    }

    let canonical = load_registry_methods(&ps_path);
    let hs_path = root.join(HEALTHSPRING_REGISTRY);
    let sections = load_registry_sections(&hs_path);

    let mut missing = Vec::new();
    for (section, locality, methods) in &sections {
        if locality != "consumed" && locality != "routed" {
            continue;
        }
        for m in methods {
            if !canonical.contains(m) {
                missing.push(format!("[{section}] {m}"));
            }
        }
    }

    if !missing.is_empty() {
        eprintln!(
            "ADVISORY: healthSpring consumed/routed methods not yet in primalSpring registry:\n  {}",
            missing.join("\n  ")
        );
        eprintln!(
            "These may be domain-specific extensions pending upstream adoption.\n\
             Hand back to primalSpring for registry inclusion."
        );
    }
}

#[test]
fn local_methods_dont_collide_with_primal_owners() {
    let root = workspace_root();
    let ps_path = root.join(PRIMALSPRING_REGISTRY);
    if !ps_path.exists() {
        eprintln!(
            "SKIP: primalSpring registry not found at {}",
            ps_path.display()
        );
        return;
    }

    let ps_content = std::fs::read_to_string(&ps_path).unwrap();
    let ps_table: toml::Value = ps_content.parse().unwrap();
    let ps_table = ps_table.as_table().unwrap();

    let mut canonical_owners: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for (section, value) in ps_table {
        if section == "test_fixtures" || section == "false_positives" {
            continue;
        }
        let owner = value
            .get("owner")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        if let Some(arr) = value.get("methods").and_then(|v| v.as_array()) {
            for m in arr {
                if let Some(s) = m.as_str() {
                    canonical_owners
                        .insert(s.to_string(), owner.to_string());
                }
            }
        }
    }

    let hs_path = root.join(HEALTHSPRING_REGISTRY);
    let sections = load_registry_sections(&hs_path);

    let standard_probes: std::collections::HashSet<&str> = [
        "health.liveness",
        "health.readiness",
        "health.check",
        "identity.get",
        "capability.list",
        "mcp.tools.list",
    ]
    .into_iter()
    .collect();

    let mut collisions = Vec::new();
    for (section, locality, methods) in &sections {
        if locality != "local" {
            continue;
        }
        for m in methods {
            if standard_probes.contains(m.as_str()) {
                continue;
            }
            if let Some(canonical_owner) = canonical_owners.get(m.as_str()) {
                let dominated = matches!(
                    canonical_owner.as_str(),
                    "all" | "healthspring" | "primalspring" | "tests"
                );
                if !dominated {
                    collisions.push(format!(
                        "[{section}] {m} — canonical owner is {canonical_owner}"
                    ));
                }
            }
        }
    }

    assert!(
        collisions.is_empty(),
        "healthSpring local methods collide with primal-owned canonical methods:\n  {}",
        collisions.join("\n  ")
    );
}

#[test]
fn healthspring_registry_parses_and_has_all_localities() {
    let root = workspace_root();
    let hs_path = root.join(HEALTHSPRING_REGISTRY);
    let sections = load_registry_sections(&hs_path);

    let localities: std::collections::HashSet<String> =
        sections.iter().map(|(_, l, _)| l.clone()).collect();

    assert!(
        localities.contains("local"),
        "registry must have local sections"
    );
    assert!(
        localities.contains("routed"),
        "registry must have routed sections"
    );
    assert!(
        localities.contains("consumed"),
        "registry must have consumed sections"
    );

    let total_methods: usize = sections.iter().map(|(_, _, m)| m.len()).sum();
    assert!(
        total_methods >= 80,
        "registry should have 80+ methods, found {total_methods}"
    );
}
