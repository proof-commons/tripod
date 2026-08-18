//! `generate-attestation-register`: the companion register writer.
//!
//! The acceptee owns this generator (ADR-020). Unlike the two upstream
//! registers, the derivation is corpus-wide: the evidence rows carry a
//! mint census, and a census answers to every owner. The census
//! therefore arrives whole, by role-tagged argument (ADR-014), and the
//! register output is an argument-supplied asset; the build system
//! wraps this command with its own stamp.

use std::{collections::BTreeMap, path::PathBuf, process::ExitCode};

use clap::Parser;
use cli_common::{BaseArgs, install_json_panic_hook, run_no_stdout_command};

const COMMAND_NAME: &str = "generate-attestation-register";

#[derive(Parser)]
#[command(
    name = "generate-attestation-register",
    version,
    about = "Generate the companion attestation register"
)]
struct Args {
    #[command(flatten)]
    base: BaseArgs,
    /// Repository root; relative subject paths resolve against it.
    #[arg(long, value_name = "DIR")]
    repository_root: PathBuf,
    /// The Attestation specification root TeX file.
    #[arg(long, value_name = "FILE")]
    attestation_main: PathBuf,
    /// Attestation specification section TeX files.
    #[arg(long = "attestation-section", value_name = "FILE")]
    attestation_sections: Vec<PathBuf>,
    /// The realization Markdown document.
    #[arg(long, value_name = "FILE")]
    realization: PathBuf,
    /// Numbered ADR Markdown records.
    #[arg(long = "adr", value_name = "FILE")]
    adrs: Vec<PathBuf>,
    /// Planning Markdown files (generated registers excluded).
    #[arg(long = "plan", value_name = "FILE")]
    plans: Vec<PathBuf>,
    /// Repository documentation Markdown (the DOC owner).
    #[arg(long = "doc", value_name = "FILE")]
    docs: Vec<PathBuf>,
    /// Model crate Rust sources.
    #[arg(long = "model-source", value_name = "FILE")]
    model_sources: Vec<PathBuf>,
    /// Other first-party crate Rust sources.
    #[arg(long = "crate-source", value_name = "FILE")]
    crate_sources: Vec<PathBuf>,
    /// The generated Layer-0 register publication.
    #[arg(long, value_name = "FILE")]
    specification_register: PathBuf,
    /// The generated realization register publication.
    #[arg(long, value_name = "FILE")]
    realization_register: PathBuf,
    /// The generated model-label publication.
    #[arg(long, value_name = "FILE")]
    model_labels_json: PathBuf,
    /// The companion attestation register: this generator's one output,
    /// and its own census entry. The two are the same path by
    /// construction, so the generator takes the label census unchanged
    /// and no second spelling of the output can drift from it.
    #[arg(long, value_name = "FILE")]
    attestation_register: PathBuf,
}

impl Args {
    fn census(&self) -> Result<labels::RepositoryCensus, String> {
        let root = &self.repository_root;
        let resolve = |path: &PathBuf| labels::RepositoryCensus::resolve(root, path.clone());
        let resolve_all = |paths: &[PathBuf]| paths.iter().map(resolve).collect::<Vec<_>>();
        Ok(labels::RepositoryCensus {
            root: root.clone(),
            attestation_main: resolve(&self.attestation_main),
            attestation_sections: resolve_all(&self.attestation_sections),
            realization: resolve(&self.realization),
            adrs: resolve_all(&self.adrs),
            plans: resolve_all(&self.plans),
            docs: resolve_all(&self.docs),
            model_sources: resolve_all(&self.model_sources),
            crate_sources: labels::group_crate_sources(root, self.crate_sources.clone())?,
            specification_register: resolve(&self.specification_register),
            realization_register: resolve(&self.realization_register),
            attestation_register: resolve(&self.attestation_register),
            model_labels_json: resolve(&self.model_labels_json),
            traversal: BTreeMap::new(),
        })
    }
}

fn main() -> ExitCode {
    // Installed before parsing so an early panic still fails closed
    // with a JSON-only record (ADR-010 early-startup rule).
    install_json_panic_hook(COMMAND_NAME);
    let args = cli_common::parse_args_or_exit::<Args>();
    run_no_stdout_command(COMMAND_NAME, args.base.debug, tracing::Level::INFO, || {
        let paths = args.census().map_err(|message| {
            labels::repository::GenerateError::Validation(vec![labels::LabelDiagnostic::error(
                labels::LabelErrorCode::UnregisteredOwner,
                &labels::source::SourceLocation::new(std::path::Path::new("Cargo.toml"), 1, 1),
                message,
            )])
        })?;
        let output = paths.attestation_register.clone();
        let register = labels::generate_attestation_register(&paths, &output).inspect_err(
            |error| {
                for diagnostic in error.diagnostics().iter().filter(|d| d.is_error()) {
                    tracing::error!(code = ?diagnostic.code, path = %diagnostic.path, line = diagnostic.line, message = %diagnostic.message, "attestation source validation failed");
                }
            },
        )?;
        tracing::info!(path = %register.path.display(), bytes = register.bytes, "attestation register written");
        Ok::<(), labels::repository::GenerateError>(())
    })
}
