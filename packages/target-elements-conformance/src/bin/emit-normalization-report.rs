//! `emit-normalization-report`: the §14 safety report, from a run record.
//!
//! # Why the judgement happens here and not in the runner
//!
//! Three §10.4 rows are refused by the report layer: the target accepts
//! the transaction and the claim does not. Deciding that means comparing
//! the claim with the transaction, and the claim lives in this crate. A
//! runner that made the comparison would be a second implementation of
//! the claim, free to drift from the first — and a drifted claim agrees
//! with whatever it is compared against.
//!
//! So the runner is transport: it records the adapter's responses
//! verbatim. This command reads that record, rebuilds the matrix and its
//! expectations from [`target_elements_conformance::normalization`], and
//! produces the typed report. The expectations are never read from the
//! run record, so a run cannot supply the answer it is checked against.

//! # The document is the library's, and the contract is this file's
//!
//! The report itself is built by
//! [`target_elements_conformance::emit::normalization_report_document`],
//! where it is testable without spawning anything. What remains here is
//! the part that cannot be: argument parsing, the shared panic hook, the
//! terminal refusal, reading the named file, and the write to stdout —
//! the ADR-010 subprocess contract
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
use target_elements_conformance::emit::normalization_report_document;

const COMMAND_NAME: &str = "emit-normalization-report";

#[derive(Parser)]
#[command(
    name = "emit-normalization-report",
    version,
    about = "Build the normalization safety report from a run record"
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
        normalization_report_document(&record)
    })
}
