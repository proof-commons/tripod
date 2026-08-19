//! `emit-conservation-matrix`: the canonical §8.4 matrix, as JSON.
//!
//! The matrix is owned by
//! [`target_elements_conformance::conservation`], and this command is the
//! only way anything outside the crate obtains it. That is the point: a
//! runner that re-derived the fixture language in another language would
//! drift from it, and the drift would be invisible precisely where it
//! mattered — a blinding factor computed by a slightly different recipe
//! still looks like a blinding factor.
//!
//! # It emits the rows, and separately the requests
//!
//! `rows` carries the whole matrix, expectations included, for a reader
//! and for the report. `requests` carries what may be sent to an
//! executor: the row identity and its subject, and no expected layer
//! `(´[PLAN-rule:guide11-exec:request-subject]´)`. Two shapes rather than
//! one, so that the boundary is a property of the data rather than a
//! discipline a runner is trusted to observe.
//!
//! Deferred rows appear in `rows` and never in `requests`: a row this
//! wave established it cannot materialize is not asked of a target.

//! # The document is the library's, and the contract is this file's
//!
//! What this command publishes is built by
//! [`target_elements_conformance::emit::conservation_matrix_document`],
//! where it is testable without spawning anything. What remains here is
//! the part that cannot be: argument parsing, the shared panic hook, the
//! terminal refusal, and the write to stdout — the ADR-010 subprocess
//! contract `(´[ADR010-rule:output:streams]´)`.

use std::process::ExitCode;

use clap::Parser;
use cli_common::{BaseArgs, install_json_panic_hook, run_stdout_json_command};
use target_elements_conformance::emit::conservation_matrix_document;

const COMMAND_NAME: &str = "emit-conservation-matrix";

#[derive(Parser)]
#[command(
    name = "emit-conservation-matrix",
    version,
    about = "Emit the canonical confidential-conservation matrix as JSON"
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
    run_stdout_json_command(COMMAND_NAME, args.base.debug, tracing::Level::INFO, || {
        Ok::<_, std::convert::Infallible>(conservation_matrix_document())
    })
}
