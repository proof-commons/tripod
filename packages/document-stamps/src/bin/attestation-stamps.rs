//! `attestation-stamps`: derive deterministic Attestation paper
//! metadata from committed Git state and emit it as one JSON object.
//!
//! The build system supplies the Git program, the repository root, the
//! revision, the paper subtree, and the exact publication-input set. On
//! success the four prepared values go to stdout as one JSON line
//! (ADR-010); the command performs no writes.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use cli_common::{BaseArgs, install_json_panic_hook, run_stdout_json_command};
use document_stamps::StampRequest;

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
    /// Git revision committed objects and history are read from.
    #[arg(long, value_name = "REV", default_value = "HEAD")]
    tree_ref: String,
    /// Repository-relative paper subtree.
    #[arg(long, value_name = "DIR")]
    tree: PathBuf,
    /// Exact publication-input members (repository-relative).
    #[arg(long = "input", value_name = "FILE", required = true)]
    inputs: Vec<PathBuf>,
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
    run_stdout_json_command(COMMAND_NAME, debug, tracing::Level::INFO, move || {
        document_stamps::run(&request)
    })
}
