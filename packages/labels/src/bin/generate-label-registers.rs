//! `generate-label-registers`: the register-publication writer.
//!
//! Scoped derivation (ADR-019): only the attestation and realization
//! sources feed the registers, and they arrive by role-tagged argument
//! (ADR-014). The two register outputs are argument-supplied assets;
//! the build system wraps this command with its own stamp.

use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use cli_common::{BaseArgs, install_json_panic_hook, run_no_stdout_command};

const COMMAND_NAME: &str = "generate-label-registers";

#[derive(Parser)]
#[command(
    name = "generate-label-registers",
    version,
    about = "Generate planning upstream-label registers"
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
    /// Output path of the specification register.
    #[arg(long, value_name = "FILE")]
    specification_register_output: PathBuf,
    /// Output path of the realization register.
    #[arg(long, value_name = "FILE")]
    realization_register_output: PathBuf,
}

fn main() -> ExitCode {
    // Installed before parsing so an early panic still fails closed
    // with a JSON-only record (ADR-010 early-startup rule).
    install_json_panic_hook(COMMAND_NAME);
    let args = cli_common::parse_args_or_exit::<Args>();
    run_no_stdout_command(COMMAND_NAME, args.base.debug, tracing::Level::INFO, || {
        let root = &args.repository_root;
        let resolve = |path: &PathBuf| labels::RepositoryCensus::resolve(root, path.clone());
        let paths = labels::RepositoryCensus {
            root: root.clone(),
            attestation_main: resolve(&args.attestation_main),
            attestation_sections: args.attestation_sections.iter().map(resolve).collect(),
            realization: resolve(&args.realization),
            ..labels::RepositoryCensus::default()
        };
        let registers = labels::generate_registers(
            &paths,
            &resolve(&args.specification_register_output),
            &resolve(&args.realization_register_output),
        )
        .inspect_err(|error| {
            for diagnostic in error.diagnostics() {
                tracing::error!(code = ?diagnostic.code, path = %diagnostic.path, line = diagnostic.line, message = %diagnostic.message, "label source validation failed");
            }
        })?;
        for register in registers {
            tracing::info!(path = %register.path.display(), bytes = register.bytes, "label register written");
        }
        Ok::<(), labels::repository::GenerateError>(())
    })
}
