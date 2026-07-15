//! `check-generated`: the non-writing stale-artifact checker.
//!
//! Computes the expected bytes of every generated artifact in memory
//! and compares them with the committed files. Never writes. The
//! writing path is `generate-all`.
//!
//! ADR-010: stdout result data (one JSON [`artifacts::CheckReport`]
//! object) is emitted only on success (exit 0: everything current);
//! on failure stdout stays empty, per-artifact verdicts are emitted
//! as JSON diagnostics on stderr, and the exit code is the branch
//! signal. TTY refusal applies.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use cli_common::{
    BaseArgs, CommandExit, emit_control_plane_record, init_json_tracing_with_debug,
    install_json_panic_hook, parse_args_or_exit, set_panic_payload_reporting_enabled,
    stdout_tty_refusal_record,
};

const COMMAND_NAME: &str = "check-generated";

#[derive(Parser)]
#[command(
    name = "check-generated",
    version,
    about = "Check committed generated artifacts against their typed sources without writing"
)]
struct Args {
    #[command(flatten)]
    base: BaseArgs,

    /// Generated directory to check. Defaults to the committed
    /// generated directory (packages/model/generated).
    #[arg(long, value_name = "DIR")]
    generated_dir: Option<PathBuf>,
}

fn main() -> ExitCode {
    install_json_panic_hook(COMMAND_NAME);
    let args = parse_args_or_exit::<Args>();

    set_panic_payload_reporting_enabled(args.base.debug);
    init_json_tracing_with_debug(args.base.debug, tracing::Level::INFO);

    if let Some(record) = stdout_tty_refusal_record(COMMAND_NAME) {
        let _ignored = emit_control_plane_record(&record);
        return CommandExit::Usage.exit_code();
    }

    let dir = args.generated_dir.unwrap_or_else(artifacts::generated_dir);

    let report = match artifacts::check(&dir) {
        Ok(report) => report,
        Err(error) => {
            tracing::error!(error = %error, "check failed");
            return CommandExit::Failure.exit_code();
        }
    };

    if !report.current {
        for artifact in &report.artifacts {
            if artifact.status != artifacts::ArtifactFreshness::Current {
                tracing::error!(
                    artifact = %artifact.name,
                    status = ?artifact.status,
                    "generated artifact is not current; run \
                     `cargo run -p tripod-artifacts --bin generate-all` \
                     and commit the diff",
                );
            }
        }
        for stray in &report.unexpected {
            tracing::error!(
                file = %stray,
                "unexpected file in generated directory",
            );
        }
        return CommandExit::Failure.exit_code();
    }

    match cli_common::emit(&report) {
        Ok(()) => CommandExit::Success.exit_code(),
        Err(error) => {
            tracing::error!(error = %error, "failed to write JSON report to stdout");
            CommandExit::Failure.exit_code()
        }
    }
}
