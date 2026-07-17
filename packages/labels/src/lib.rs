//! Repository-wide documentation-label tooling.
//!
//! This crate owns label harvesting, imported-citation validation, and
//! derivative registry rendering. It is documentation tooling, never a
//! protocol or compiler semantic input.

#![forbid(unsafe_code)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::too_many_lines
)]

pub mod census;
pub mod check;
pub mod diagnostic;
pub mod label;
pub mod latex;
pub mod markdown;
pub mod owner;
pub mod registry;
pub mod render;
pub mod repository;
pub mod rust_source;
pub mod source;

pub use census::{CensusGroup, RepositoryCensus, group_crate_sources};
pub use check::{CheckReport, check_repository};
pub use diagnostic::{LabelDiagnostic, LabelErrorCode, Severity};
pub use label::{Label, LabelShape};
pub use repository::{generate_registers, model_labels_json};

#[cfg(test)]
mod tests;
