#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::too_many_lines
)]

pub mod adoption;
pub mod attestation;
pub mod census;
pub mod check;
pub mod diagnostic;
pub mod forbidden;
pub mod heads;
pub mod label;
pub mod latex;
pub mod markdown;
pub mod nearmiss;
pub mod owner;
pub mod participation;
pub mod plans;
pub mod registry;
pub mod render;
pub mod repository;
pub mod rust_source;
pub mod shape;
pub mod source;

pub use census::{CensusGroup, RepositoryCensus, group_crate_sources};
pub use check::{CheckReport, check_repository};
pub use diagnostic::{LabelDiagnostic, LabelErrorCode, Severity};
pub use label::{Label, LabelShape};
pub use repository::{
    attestation_base, generate_attestation_register, generate_registers, model_labels_json,
};

#[cfg(test)]
mod tests;
