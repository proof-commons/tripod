//! `emit-normalization-matrix`: the canonical §10.4 matrix, as JSON.
//!
//! The matrix and the claim are owned by
//! [`target_elements_conformance::normalization`], and this command is
//! the only way anything outside the crate obtains them — the same
//! reasoning `emit-conservation-matrix` rests on: a runner that restated
//! the claim in another language would drift from it, and a drifted
//! claim is undetectable exactly where it matters, because a wrong
//! amount still looks like an amount.
//!
//! # It emits the rows, and separately the requests
//!
//! `rows` carries the whole matrix, expectations and reasoning included,
//! for a reader and for the report. `requests` carries what may be sent
//! to an executor: the row identity and its subject, and no expected
//! layer `(´[PLAN-rule:guide11-exec:request-subject]´)`.
//!
//! The separation matters more here than for a conservation row. Three
//! of these rows are answered by the report layer rather than by the
//! target, and an executor that knew which ones could report a
//! disagreement it never observed.

//! # The document is the library's, and the contract is this file's
//!
//! What this command publishes is built by
//! [`target_elements_conformance::emit::normalization_matrix_document`],
//! where it is testable without spawning anything. What remains here is
//! the part that cannot be: argument parsing, the shared panic hook, the
//! terminal refusal, and the write to stdout — the ADR-010 subprocess
//! contract `(´[ADR010-rule:output:streams]´)`.
//!
//! The claim's conservation was an assertion inside this command, which
//! is to say a crash. It is now a refusal the library returns, which is
//! both a better diagnostic and the reason it can be checked at all.

use std::process::ExitCode;

use clap::Parser;
use cli_common::{BaseArgs, install_json_panic_hook, run_stdout_json_command};
use target_elements_conformance::emit::normalization_matrix_document;

const COMMAND_NAME: &str = "emit-normalization-matrix";

#[derive(Parser)]
#[command(
    name = "emit-normalization-matrix",
    version,
    about = "Emit the canonical owner-authorized normalization matrix as JSON"
)]
struct Args {
    #[command(flatten)]
    base: BaseArgs,
}

fn main() -> ExitCode {
    // Installed before parsing so an early panic still fails closed with
    // a JSON-only record (ADR-010 early-startup rule).
    install_json_panic_hook(COMMAND_NAME);
    let args = cli_common::parse_args_or_exit::<Args>();
    run_stdout_json_command(
        COMMAND_NAME,
        args.base.debug,
        tracing::Level::INFO,
        normalization_matrix_document,
    )
}
