// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]

//! healthSpring barraCuda — health science compute library.
//!
//! Pure Rust implementations of PK/PD, microbiome, biosignal, endocrine, and
//! diagnostic models. GPU-accelerated via WGSL shaders behind the `gpu` feature.
//!
//! ## Modules
//!
//! | Module | Domain | GPU Shader |
//! |--------|--------|-----------|
//! | [`pkpd`] | Hill dose-response, compartmental PK, population MC | `hill_dose_response_f64.wgsl`, `population_pk_f64.wgsl` |
//! | [`microbiome`] | Shannon, Simpson, Pielou, Chao1, Anderson lattice | `diversity_f64.wgsl` |
//! | [`microbiome_transfer`] | Cross-spring gut Anderson params for neuralSpring | — |
//! | [`biosignal`] | Pan-Tompkins QRS, HRV, PPG SpO2, fusion | — (NPU target) |
//! | [`endocrine`] | Testosterone PK, TRT outcomes, decline models | — |
//! | [`diagnostic`] | Integrated 4-track patient pipeline | — |
//! | [`discovery`] | MATRIX scoring, HTS analysis, compound IC50, fibrosis | — (GPU: Hill sweep) |
//! | [`comparative`] | Species PK, canine IL-31/JAK1, feline MM PK, allometric | — |
//! | [`gpu`] | GPU dispatch: `GpuOp`, `GpuContext`, fused pipeline | All shaders |
//! | [`validation`] | Shared `ValidationHarness` (hotSpring pattern) | — |
//! | [`simulation`] | Multi-scale causal chain: stress pathways → cells → tissue → population → ecosystem | — |
//! | [`toxicology`] | Systemic burden, Anderson toxicity landscape, clearance regime | — |
//! | [`tolerances`] | Centralized tolerance constants from `TOLERANCE_REGISTRY.md` | — |
//! | [`provenance`] | Baseline provenance tracking (script, commit, date) | — |
//! | [`qs`] | QS gene profiling: functional Anderson disorder | — |
//! | [`data`] | Three-tier fetch: biomeOS → NestGate → NCBI HTTP | — |
//! | [`visualization`] | petalTongue schema: `DataChannel`, `ClinicalRange` | — |
//! | [`uncertainty`] | Bootstrap, jackknife, bias–variance decomposition | — |
//! | [`wfdb`] | PhysioNet WFDB format parser (`.hea`, `.dat`, `.atr`) | — |
//! | [`cast`] | Safe numeric cast helpers (`usize_f64`, `u64_f64`, `f64_usize`) | — |
//! | [`safe_cast`] | Checked casts returning `Result` (`usize_u32`, `usize_f64`, `f64_f32`) | — |

/// Canonical primal identity — single source of truth for all modules.
pub const PRIMAL_NAME: &str = "healthspring";
/// The biomeOS domain this primal serves.
pub const PRIMAL_DOMAIN: &str = "health";
/// QS gene matrix data file name (shared across storage, fetch, and QS modules).
pub const QS_GENE_MATRIX_FILE: &str = "qs_gene_matrix.json";

pub mod biosignal;
pub mod cast;
pub mod comparative;
pub mod data;
pub mod diagnostic;
pub mod discovery;
pub mod endocrine;
pub mod gpu;
pub mod ipc;
pub mod microbiome;
pub mod microbiome_transfer;
pub mod niche;
pub mod pkpd;
pub mod primal_names;
pub mod provenance;
pub mod qs;
pub mod rng;
pub mod safe_cast;
pub mod simulation;
pub mod tolerances;
pub mod toxicology;
pub mod uncertainty;
pub mod validation;
pub mod visualization;
pub mod wfdb;
