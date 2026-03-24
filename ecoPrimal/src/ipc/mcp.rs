// SPDX-License-Identifier: AGPL-3.0-or-later
//! MCP tool definitions for healthSpring capabilities.
//!
//! Provides typed schemas describing healthSpring's JSON-RPC methods
//! so that Squirrel and other AI coordinators can discover and invoke
//! them programmatically. Follows the wetSpring V127 MCP pattern.

use serde_json::{Value, json};

/// A single MCP tool definition.
pub struct McpToolDef {
    /// JSON-RPC method name (e.g. `science.pkpd.hill_dose_response`).
    pub name: &'static str,
    /// Human-readable description for AI tool selection.
    pub description: &'static str,
    /// JSON Schema for input parameters.
    pub input_schema: fn() -> Value,
}

fn pkpd_tools_a() -> Vec<McpToolDef> {
    vec![
        McpToolDef {
            name: "science.pkpd.hill_dose_response",
            description: "Compute 4-parameter Hill dose-response curve (IC50/EC50)",
            input_schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "concentration": {"type": "number", "description": "Drug concentration"},
                        "ic50": {"type": "number", "description": "Half-maximal inhibitory concentration"},
                        "hill_n": {"type": "number", "description": "Hill slope"},
                        "e_max": {"type": "number", "description": "Maximal effect"}
                    },
                    "required": ["concentration", "ic50", "hill_n", "e_max"]
                })
            },
        },
        McpToolDef {
            name: "science.pkpd.one_compartment_pk",
            description: "One-compartment PK: IV bolus or oral absorption",
            input_schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "route": {"type": "string", "description": "iv or oral", "default": "iv"},
                        "dose_mg": {"type": "number", "description": "Dose (mg), IV route"},
                        "vd": {"type": "number", "description": "Volume of distribution (L)"},
                        "half_life_hr": {"type": "number", "description": "Half-life (h), IV route"},
                        "t": {"type": "number", "description": "Time point (h)"},
                        "dose": {"type": "number", "description": "Dose, oral route"},
                        "f": {"type": "number", "description": "Bioavailability, oral route"},
                        "ka": {"type": "number", "description": "Absorption rate, oral route"},
                        "ke": {"type": "number", "description": "Elimination rate, oral route"}
                    },
                    "required": []
                })
            },
        },
        McpToolDef {
            name: "science.pkpd.two_compartment_pk",
            description: "Two-compartment PK model concentration at time t",
            input_schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "c0": {"type": "number", "description": "Initial concentration"},
                        "alpha": {"type": "number", "description": "Fast disposition rate"},
                        "beta": {"type": "number", "description": "Slow disposition rate"},
                        "k21": {"type": "number", "description": "Peripheral-to-central rate"},
                        "t": {"type": "number", "description": "Time (h)"}
                    },
                    "required": ["c0", "alpha", "beta", "k21", "t"]
                })
            },
        },
    ]
}

fn pkpd_tools_b() -> Vec<McpToolDef> {
    vec![
        McpToolDef {
            name: "science.pkpd.pbpk_simulate",
            description: "Physiologically-based PK simulation (IV infusion)",
            input_schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "dose_mg": {"type": "number", "description": "Dose (mg)"},
                        "duration_hr": {"type": "number", "description": "Infusion duration (h)"},
                        "dt": {"type": "number", "description": "Time step", "default": 0.01},
                        "blood_volume_l": {"type": "number", "description": "Blood volume (L)", "default": 5.0}
                    },
                    "required": ["dose_mg", "duration_hr"]
                })
            },
        },
        McpToolDef {
            name: "science.pkpd.population_pk",
            description: "Population PK Monte Carlo (baricitinib-like)",
            input_schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "n": {"type": "integer", "description": "Number of subjects", "default": 100},
                        "seed": {"type": "integer", "description": "RNG seed", "default": 42}
                    }
                })
            },
        },
        McpToolDef {
            name: "science.pkpd.michaelis_menten_nonlinear",
            description: "Michaelis-Menten nonlinear PK (phenytoin-like)",
            input_schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "vmax": {"type": "number", "description": "Max elimination rate"},
                        "km": {"type": "number", "description": "Michaelis constant"},
                        "vd": {"type": "number", "description": "Volume of distribution"},
                        "c0": {"type": "number", "description": "Initial concentration", "default": 25.0},
                        "duration_hr": {"type": "number", "description": "Simulation duration (h)", "default": 72.0},
                        "dt": {"type": "number", "description": "Time step", "default": 0.1}
                    }
                })
            },
        },
    ]
}

fn pkpd_tools() -> Vec<McpToolDef> {
    let mut t = pkpd_tools_a();
    t.extend(pkpd_tools_b());
    t
}

fn microbiome_tools() -> Vec<McpToolDef> {
    vec![
        McpToolDef {
            name: "science.microbiome.shannon_index",
            description: "Shannon diversity index from abundance vector",
            input_schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "abundances": {"type": "array", "items": {"type": "number"}, "description": "Species abundances (proportions)"}
                    },
                    "required": ["abundances"]
                })
            },
        },
        McpToolDef {
            name: "science.microbiome.anderson_gut",
            description: "Anderson localization on gut disorder matrix (eigenvalues, IPR)",
            input_schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "disorder": {"type": "array", "items": {"type": "number"}, "description": "Disorder values per site"},
                        "t_hop": {"type": "number", "description": "Hopping parameter", "default": 1.0}
                    },
                    "required": ["disorder"]
                })
            },
        },
        McpToolDef {
            name: "science.microbiome.colonization_resistance",
            description: "Colonization resistance metric (C. difficile susceptibility)",
            input_schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "xi": {"type": "number", "description": "Localization length (Anderson)"}
                    },
                    "required": ["xi"]
                })
            },
        },
        McpToolDef {
            name: "science.microbiome.scfa_production",
            description: "Short-chain fatty acid production from fiber input",
            input_schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "fiber_g_per_l": {"type": "number", "description": "Fiber (g/L)", "default": 10.0}
                    }
                })
            },
        },
        McpToolDef {
            name: "science.microbiome.qs_effective_disorder",
            description: "Quorum-sensing effective disorder from Pielou + QS gene profile",
            input_schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "pielou_j": {"type": "number", "description": "Pielou evenness"},
                        "abundances": {"type": "array", "items": {"type": "number"}, "description": "Species abundances"},
                        "alpha": {"type": "number", "default": 0.7},
                        "w_scale": {"type": "number", "default": 5.0}
                    },
                    "required": ["pielou_j", "abundances"]
                })
            },
        },
    ]
}

fn biosignal_tools() -> Vec<McpToolDef> {
    vec![
        McpToolDef {
            name: "science.biosignal.pan_tompkins",
            description: "Pan-Tompkins QRS detection and heart rate from ECG",
            input_schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "signal": {"type": "array", "items": {"type": "number"}, "description": "ECG samples"},
                        "fs": {"type": "number", "description": "Sampling frequency (Hz)", "default": 360.0}
                    },
                    "required": ["signal"]
                })
            },
        },
        McpToolDef {
            name: "science.biosignal.hrv_metrics",
            description: "HRV metrics (SDNN, RMSSD, pNN50) from R-peak indices",
            input_schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "peaks": {"type": "array", "items": {"type": "integer"}, "description": "R-peak sample indices"},
                        "fs": {"type": "number", "description": "Sampling frequency (Hz)", "default": 360.0}
                    },
                    "required": ["peaks"]
                })
            },
        },
        McpToolDef {
            name: "science.biosignal.ppg_spo2",
            description: "SpO2 calibration from PPG AC/DC red and IR channels",
            input_schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "ac_red": {"type": "number", "description": "AC component, red"},
                        "dc_red": {"type": "number", "description": "DC component, red"},
                        "ac_ir": {"type": "number", "description": "AC component, infrared"},
                        "dc_ir": {"type": "number", "description": "DC component, infrared"}
                    },
                    "required": ["ac_red", "dc_red", "ac_ir", "dc_ir"]
                })
            },
        },
        McpToolDef {
            name: "science.biosignal.arrhythmia_classification",
            description: "Beat classification (normal vs abnormal) from ECG",
            input_schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "signal": {"type": "array", "items": {"type": "number"}, "description": "ECG samples"},
                        "fs": {"type": "number", "default": 360.0},
                        "half_width": {"type": "integer", "default": 25},
                        "min_correlation": {"type": "number", "default": 0.7}
                    },
                    "required": ["signal"]
                })
            },
        },
    ]
}

fn endocrine_tools() -> Vec<McpToolDef> {
    vec![
        McpToolDef {
            name: "science.endocrine.testosterone_pk",
            description: "Testosterone IM depot PK (cypionate-like)",
            input_schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "dose_mg": {"type": "number", "description": "Weekly dose (mg)"},
                        "f_im": {"type": "number", "description": "IM bioavailability"},
                        "vd": {"type": "number", "description": "Volume of distribution (L)"},
                        "ka": {"type": "number", "description": "Absorption rate"},
                        "ke": {"type": "number", "description": "Elimination rate"},
                        "t": {"type": "number", "description": "Time (h)"}
                    }
                })
            },
        },
        McpToolDef {
            name: "science.endocrine.trt_outcomes",
            description: "TRT outcomes: weight change, hazard ratio, HbA1c trajectory",
            input_schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "month": {"type": "number", "description": "Months on TRT", "default": 12.0},
                        "testosterone_ng_dl": {"type": "number", "description": "Serum testosterone (ng/dL)", "default": 600.0}
                    }
                })
            },
        },
        McpToolDef {
            name: "science.microbiome.gut_brain_serotonin",
            description: "Gut-brain axis: Shannon, Pielou, disorder, serotonin proxy from abundances",
            input_schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "abundances": {"type": "array", "items": {"type": "number"}, "description": "Gut species abundances"}
                    },
                    "required": ["abundances"]
                })
            },
        },
    ]
}

fn compute_and_health_tools() -> Vec<McpToolDef> {
    vec![
        McpToolDef {
            name: "compute.offload",
            description: "Offload compute to Node Atomic GPU via compute.dispatch",
            input_schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "operation": {"type": "string", "description": "Compute operation name"},
                        "params": {"type": "object", "description": "Operation-specific params"}
                    }
                })
            },
        },
        McpToolDef {
            name: "compute.shader_compile",
            description: "Sovereign WGSL shader compilation via shader.compile capability",
            input_schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "source": {"type": "string", "description": "WGSL source"},
                        "entry_point": {"type": "string", "description": "Entry point name"}
                    }
                })
            },
        },
        // ── Model (1 tool) ──────────────────────────────────────────────
        McpToolDef {
            name: "model.inference_route",
            description: "Route model inference via model.inference capability",
            input_schema: || {
                json!({
                    "type": "object",
                    "properties": {
                        "operation": {"type": "string", "description": "infer or other", "default": "infer"},
                        "model": {"type": "string"},
                        "input": {"type": "object"}
                    }
                })
            },
        },
        // ── Health (2 tools) ────────────────────────────────────────────
        McpToolDef {
            name: "health.liveness",
            description: "Liveness probe: is the primal alive?",
            input_schema: || json!({"type": "object", "properties": {}}),
        },
        McpToolDef {
            name: "health.readiness",
            description: "Readiness probe: subsystems available",
            input_schema: || json!({"type": "object", "properties": {}}),
        },
    ]
}

/// Returns the full set of MCP tool definitions for healthSpring.
#[must_use]
pub fn tool_definitions() -> Vec<McpToolDef> {
    let mut tools = Vec::new();
    tools.extend(pkpd_tools());
    tools.extend(microbiome_tools());
    tools.extend(biosignal_tools());
    tools.extend(endocrine_tools());
    tools.extend(compute_and_health_tools());
    tools
}

/// Returns tool definitions as JSON for `mcp.tools.list` RPC response.
///
/// Format matches MCP tool schema: `{ name, description, inputSchema }`.
#[must_use]
pub fn tool_definitions_json() -> Value {
    let tools: Vec<Value> = tool_definitions()
        .into_iter()
        .map(|t| {
            json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": (t.input_schema)(),
            })
        })
        .collect();
    json!({ "tools": tools })
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use super::*;

    #[test]
    fn tool_definitions_not_empty() {
        let tools = tool_definitions();
        assert!(
            tools.len() > 10,
            "expected >10 MCP tools, got {}",
            tools.len()
        );
    }

    #[test]
    fn all_tools_have_semantic_names() {
        for tool in tool_definitions() {
            assert!(
                tool.name.contains('.'),
                "tool name should be semantic (domain.operation): {}",
                tool.name
            );
        }
    }

    #[test]
    fn all_schemas_are_valid_json_objects() {
        for tool in tool_definitions() {
            let schema = (tool.input_schema)();
            assert_eq!(
                schema["type"], "object",
                "tool {} schema should be an object",
                tool.name
            );
            assert!(
                schema.get("properties").is_some(),
                "tool {} schema should have properties",
                tool.name
            );
        }
    }

    #[test]
    fn tool_definitions_json_has_tools_array() {
        let j = tool_definitions_json();
        let arr = j["tools"].as_array().unwrap();
        assert!(!arr.is_empty());
        for tool in arr {
            assert!(tool["name"].is_string());
            assert!(tool["description"].is_string());
            assert!(tool["inputSchema"].is_object());
        }
    }

    #[test]
    fn no_duplicate_tool_names() {
        let tools = tool_definitions();
        let mut names: Vec<&str> = tools.iter().map(|t| t.name).collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();
        assert_eq!(before, names.len(), "duplicate MCP tool names detected");
    }
}
