//! `check-labels`: the non-writing repository label gate.
//!
//! Every subject file arrives by role-tagged argument (ADR-014): the
//! build system states census membership, this binary validates shape
//! and re-verifies the census against the on-disk discovery. On success
//! the JSON report is published under the active output mode (ADR-014):
//! direct-mode stdout, or a build-mode report asset plus success stamp.

use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use cli_common::{BaseArgs, CheckOutputArgs, install_json_panic_hook, run_check_command};

const COMMAND_NAME: &str = "check-labels";

#[derive(Parser)]
#[command(
    name = "check-labels",
    version,
    about = "Check repository documentation labels without writing"
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
    /// Other first-party crate Rust sources (packages/<name>/src).
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
    #[command(flatten)]
    output: CheckOutputArgs,
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
            model_labels_json: resolve(&self.model_labels_json),
        })
    }
}

fn main() -> ExitCode {
    // Installed before parsing so an early panic still fails closed
    // with a JSON-only record (ADR-010 early-startup rule).
    install_json_panic_hook(COMMAND_NAME);
    let args = cli_common::parse_args_or_exit::<Args>();
    run_check_command(
        COMMAND_NAME,
        args.base.debug,
        tracing::Level::INFO,
        &args.output,
        || {
            let paths = args.census()?;
            let (report, diagnostics) = labels::check_repository(&paths);
            if report.valid {
                return Ok(report);
            }
            for diagnostic in diagnostics {
                tracing::error!(code = ?diagnostic.code, path = %diagnostic.path, line = diagnostic.line, message = %diagnostic.message, "label check failed");
            }
            Err("repository label validation failed".to_owned())
        },
    )
}
