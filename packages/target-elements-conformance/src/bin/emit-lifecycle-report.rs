//! `emit-lifecycle-report`: the §14.1 `FreshProcessLifecycle` report.
//!
//! # The judgement happens here, and the runner is transport
//!
//! The same division `emit-normalization-report` rests on, and for a
//! sharper reason. This lane's runner is the process that CORRUPTS the
//! public record: it builds the wrong txid, the copied bytes, and the
//! foreign genesis. A runner that also decided whether each corruption
//! was caught would be marking its own work, and the one row it would
//! never fail is the one it got wrong.
//!
//! So the runner records what each reading process observed, verbatim,
//! and this command rebuilds every expectation from
//! [`target_elements_conformance::lifecycle::canonical_lifecycle_matrix`]
//! before comparing. A row the run did not answer is a missing row, not
//! a passing one.

//! # The document is the library's, and the contract is this file's
//!
//! The report and every gate it must meet are built by
//! [`target_elements_conformance::emit::lifecycle_report_document`],
//! where they are testable without spawning anything. What remains here
//! is the part that cannot be: argument parsing, the shared panic hook,
//! the terminal refusal, reading the named file, and the write to
//! stdout — the ADR-010 subprocess contract
//! `(´[ADR010-rule:output:streams]´)`.
//!
//! The run record's path is a caller's argument. A failure to read it
//! names what failed and does not interpolate the path: an operator's
//! directory layout is not part of this command's diagnostic vocabulary
//! (G12-R04).

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use cli_common::{BaseArgs, install_json_panic_hook, run_stdout_json_command};
use target_elements_conformance::emit::lifecycle_report_document;

const COMMAND_NAME: &str = "emit-lifecycle-report";

#[derive(Parser)]
#[command(
    name = "emit-lifecycle-report",
    version,
    about = "Build the fresh-process lifecycle report from a run record"
)]
struct Args {
    #[command(flatten)]
    base: BaseArgs,

    /// The run record to read.
    #[arg(long, value_name = "PATH")]
    run_record: PathBuf,
}

fn main() -> ExitCode {
    // Installed before parsing so an early panic still fails closed with
    // a JSON-only record (ADR-010 early-startup rule).
    install_json_panic_hook(COMMAND_NAME);
    let args = cli_common::parse_args_or_exit::<Args>();
    run_stdout_json_command(COMMAND_NAME, args.base.debug, tracing::Level::INFO, || {
        let text = std::fs::read_to_string(&args.run_record)
            .map_err(|error| format!("the run record could not be read: {error}"))?;
        let record: serde_json::Value = serde_json::from_str(&text)
            .map_err(|error| format!("the run record is not JSON: {error}"))?;
        lifecycle_report_document(&record)
    })
}
