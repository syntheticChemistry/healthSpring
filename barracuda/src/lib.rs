// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]

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
//! | [`biosignal`] | Pan-Tompkins QRS, HRV, PPG SpO2, fusion | — (NPU target) |
//! | [`endocrine`] | Testosterone PK, TRT outcomes, decline models | — |
//! | [`diagnostic`] | Integrated 4-track patient pipeline | — |
//! | [`gpu`] | GPU dispatch: `GpuOp`, `GpuContext`, fused pipeline | All shaders |
//! | [`visualization`] | petalTongue schema: `DataChannel`, `ClinicalRange` | — |
//! | [`wfdb`] | PhysioNet WFDB format parser (`.hea`, `.dat`, `.atr`) | — |

pub mod biosignal;
pub mod diagnostic;
pub mod endocrine;
pub mod gpu;
pub mod microbiome;
pub mod pkpd;
pub mod rng;
pub mod visualization;
pub mod wfdb;
