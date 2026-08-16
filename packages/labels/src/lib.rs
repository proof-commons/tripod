//! Repository-wide documentation-label tooling.
//!
//! This crate owns label harvesting, imported-citation validation, and
//! derivative registry rendering. It is documentation tooling, never a
//! protocol or compiler semantic input. No semantic package consumes
//! its registries or its planning labels.
//!
//! # Boundary
//!
//! Every subject file arrives by role-tagged command-line argument
//! (ADR-014). The build system states census membership from its
//! hand-managed per-directory lists, and the binaries re-verify that
//! census against the on-disk tree before trusting it. Nothing here
//! resolves a repository path from its own compiled location.
//!
//! Plan-local labels remain non-normative and non-identity-bearing.
//! Their mints, citations, and owner crossings are mechanically checked
//! under ADR-013, and that is all a label is: a documentation
//! cross-reference, never an identity a protocol depends on.
//!
//! # Public modules
//!
//! The vocabulary, smallest first:
//!
//! - [`label`] — [`Label`], the validated owner-local token, and
//!   [`LabelShape`], the per-owner arity grammar it is parsed against.
//! - [`owner`] — `LabelOwner`, the seven label owners and their
//!   prefixes, and `ImportedLabel`, a parsed owner-qualified citation.
//! - [`source`] — `SourceLocation`, a one-based line and column against
//!   a repository-relative path.
//! - [`diagnostic`] — [`LabelDiagnostic`], [`LabelErrorCode`] (21
//!   codes), and [`Severity`]. Diagnostics are data, not failures.
//! - [`registry`] — `LabelMint` and `LabelRegistry`, the
//!   insertion-checked label store, plus `RegistrySet` for all owners
//!   at once.
//! - [`render`] — the three pure renderers: `specification_register`,
//!   `realization_register`, and `model_labels_json`.
//!
//! The harvesters, each reading the census off disk:
//!
//! - [`markdown`] — `scan_markdown`, the accepted inline-code and
//!   fence grammar.
//! - [`latex`] — `harvest_attestation`, over the Layer-0 LaTeX sources.
//! - [`rust_source`] — `harvest_model` and `harvest_crates`, over Rust
//!   comment tokens.
//! - [`census`] — [`RepositoryCensus`], the role-grouped file census,
//!   [`CensusGroup`], and the tracked-file audit.
//!
//! The lanes, each backing one binary:
//!
//! - [`check`] — [`check_repository`], the non-writing label gate.
//! - [`plans`] — `check_plans`, the plan-tree structure and hygiene
//!   lane.
//! - [`repository`] — the whole-repository harvest, the citation graph,
//!   [`generate_registers`] (the only writing entry point), and
//!   [`model_labels_json`].
//! - [`forbidden`] — the forbidden placeholder-token audit.
//!
//! # Quickstart: parse a label and an imported citation
//!
//! Both parsers are pure and touch no filesystem.
//!
//! ```
//! use labels::owner::{ImportedLabel, LabelOwner};
//! use labels::{Label, LabelShape};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // An owner-local label is parsed against its owner's arity grammar.
//! let local = Label::parse("def:model:classes", LabelShape::Attestation)?;
//! assert_eq!(local.as_str(), "def:model:classes");
//!
//! // A citation naming another owner carries that owner's prefix.
//! let imported = ImportedLabel::parse("ADR012-rule:labels:decision")?;
//! assert_eq!(imported.owner, LabelOwner::Adr(12));
//! assert_eq!(imported.label.as_str(), "rule:labels:decision");
//! assert_eq!(imported.owner.prefix(), "ADR012-");
//! # Ok(())
//! # }
//! ```
//!
//! # Quickstart: mint into a registry and render a register
//!
//! The mint-to-render pipeline is pure as well, so a consumer can drive
//! it without a repository on disk.
//!
//! ```
//! use labels::owner::LabelOwner;
//! use labels::registry::{LabelMint, LabelRegistry};
//! use labels::render::specification_register;
//! use labels::source::SourceLocation;
//! use labels::{Label, LabelShape};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mint = |name: &str| -> Result<LabelMint, Box<dyn std::error::Error>> {
//!     Ok(LabelMint {
//!         owner: LabelOwner::Attestation,
//!         label: Label::parse(name, LabelShape::Attestation)?,
//!         location: SourceLocation::new("papers/attestation/model.tex", 12, 1),
//!         home: None,
//!     })
//! };
//!
//! let mut registry = LabelRegistry::default();
//! registry
//!     .insert(mint("def:model:classes")?)
//!     .expect("the first mint of a label is accepted");
//!
//! // A duplicate is refused and handed back, never silently overwritten.
//! assert!(registry.insert(mint("def:model:classes")?).is_err());
//! assert_eq!(registry.len(), 1);
//!
//! // The register renders deterministically from the registry alone.
//! let register = specification_register(&registry);
//! assert!(register.contains("``[A-def:model:classes]``"));
//! # Ok(())
//! # }
//! ```
//!
//! # Quickstart: scan Markdown for label tokens
//!
//! ```
//! use labels::markdown::{InlineCodeContext, scan_markdown};
//! use std::path::Path;
//!
//! // The path is a diagnostic label only; the text is the input.
//! let scan = scan_markdown(Path::new("fixture.md"), "# Fixture · `sec:fixture`\n");
//!
//! assert!(scan.diagnostics.is_empty());
//! assert_eq!(scan.code_spans.len(), 1);
//!
//! // Only a bare span mints; a parenthesized one is a citation.
//! assert_eq!(scan.code_spans[0].context, InlineCodeContext::Bare);
//! ```
//!
//! # What needs a repository on disk
//!
//! [`check_repository`], [`generate_registers`], [`model_labels_json`],
//! `plans::check_plans`, `RepositoryCensus::discover`, and the three
//! harvesters all read the tree. There is no in-memory constructor for
//! a [`CheckReport`], and none is offered: the gate's subject is a real
//! checkout.
//!
//! # Errors and diagnostics
//!
//! The crate separates two kinds of wrong. A *diagnostic* is a located
//! defect in the documentation being checked, returned as data in a
//! [`LabelDiagnostic`] vector, and it never aborts a run — a lane
//! collects every defect and reports them together. An *error* is an
//! environmental fault, and only the writing and parsing entry points
//! return one: `LabelParseError`, `owner::OwnerParseError`, and
//! `repository::GenerateError`.
//!
//! The full public-API tour, the binary command lines, and the
//! per-owner label grammars are in the package README.

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
pub mod forbidden;
pub mod label;
pub mod latex;
pub mod markdown;
pub mod owner;
pub mod plans;
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
