//! `attestation-stamps`: derive deterministic Attestation paper
//! metadata from committed Git state.
//!
//! The build system supplies the Git program, the repository root, the
//! revision, the paper subtree, and the exact publication-input set. In
//! the default mode the four prepared values go to stdout as one JSON
//! line (ADR-010) and nothing is written. In render mode (the
//! `--template`/`--stamps-output`/`--epoch-output` arguments) the command
//! instead writes the generated `stamps.tex` and `source-date-epoch`
//! file, emitting no stdout.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use cli_common::{
    BaseArgs, install_json_panic_hook, run_no_stdout_command, run_stdout_json_command,
};
use document_stamps::{RenderRequest, StampRequest};

const COMMAND_NAME: &str = "attestation-stamps";

#[derive(Parser)]
#[command(
    name = "attestation-stamps",
    version,
    about = "Derive deterministic Attestation paper metadata from Git"
)]
struct Args {
    #[command(flatten)]
    base: BaseArgs,
    /// Git executable supplied by the build system.
    #[arg(long, value_name = "PROGRAM")]
    git: PathBuf,
    /// Repository root against which all paths resolve.
    #[arg(long, value_name = "DIR")]
    repository_root: PathBuf,
    /// Git revision to verify against checked-out HEAD. Only revisions
    /// resolving to the same commit as HEAD are accepted.
    #[arg(long, value_name = "REV", default_value = "HEAD")]
    tree_ref: String,
    /// Repository-relative paper subtree.
    #[arg(long, value_name = "DIR")]
    tree: PathBuf,
    /// Exact publication-input members (repository-relative).
    #[arg(long = "input", value_name = "FILE", required = true)]
    inputs: Vec<PathBuf>,
    /// Render mode: the `stamps.tex.in` template to fill. Requires the
    /// two output arguments; enables writing instead of JSON stdout.
    #[arg(
        long,
        value_name = "FILE",
        requires_all = ["stamps_output", "epoch_output"]
    )]
    template: Option<PathBuf>,
    /// Render mode: where the generated `stamps.tex` is written.
    #[arg(long, value_name = "FILE", requires = "template")]
    stamps_output: Option<PathBuf>,
    /// Render mode: where the `source-date-epoch` file is written.
    #[arg(long, value_name = "FILE", requires = "template")]
    epoch_output: Option<PathBuf>,
}

fn main() -> ExitCode {
    // Installed before parsing so an early panic still fails closed with
    // a JSON-only record (ADR-010 early-startup rule).
    install_json_panic_hook(COMMAND_NAME);
    let args = cli_common::parse_args_or_exit::<Args>();
    let debug = args.base.debug;
    let request = StampRequest {
        git: args.git,
        repository_root: args.repository_root,
        tree_ref: args.tree_ref,
        tree: args.tree,
        inputs: args.inputs,
    };

    // `requires_all` makes the render arguments all-or-nothing, so this
    // tuple is either fully populated (render mode) or fully empty (JSON).
    if let (Some(template), Some(stamps_output), Some(epoch_output)) =
        (args.template, args.stamps_output, args.epoch_output)
    {
        run_no_stdout_command(COMMAND_NAME, debug, tracing::Level::INFO, move || {
            document_stamps::render(&RenderRequest {
                stamps: &request,
                template: &template,
                stamps_output: &stamps_output,
                epoch_output: &epoch_output,
            })
        })
    } else {
        run_stdout_json_command(COMMAND_NAME, debug, tracing::Level::INFO, move || {
            document_stamps::run(&request)
        })
    }
}
