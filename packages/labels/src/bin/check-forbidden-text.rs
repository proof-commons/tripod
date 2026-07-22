//! `check-forbidden-text`: repository audit rejecting a forbidden
//! angle-bracket placeholder token in tracked files.
//!
//! This is an always-fresh repository audit (ADR-014): it invokes
//! `git grep` over the complete tracked set on every build rather than
//! tracking a fixed input list, so a newly committed occurrence fails
//! immediately. On a clean tree it publishes a JSON report through the
//! shared two-mode checker contract; a match emits JSON diagnostics on
//! stderr and fails without publishing a report or touching the stamp.

use std::{path::PathBuf, process::ExitCode};

use anyhow::Context;
use clap::Parser;
use cli_common::{BaseArgs, CheckOutputArgs, install_json_panic_hook, run_check_command};
use labels::forbidden::{
    FORBIDDEN_TEXT_SCHEMA, ForbiddenTextReport, forbidden_needle, parse_grep_matches,
};

const COMMAND_NAME: &str = "check-forbidden-text";

#[derive(Parser)]
#[command(
    name = "check-forbidden-text",
    version,
    about = "Reject a forbidden angle-bracket placeholder token in tracked files"
)]
struct Args {
    #[command(flatten)]
    base: BaseArgs,
    /// Repository root the tracked-file scan runs in.
    #[arg(long, value_name = "DIR")]
    repository_root: PathBuf,
    /// The git program to invoke for the tracked-file scan.
    #[arg(long, value_name = "PROGRAM")]
    git: PathBuf,
    #[command(flatten)]
    output: CheckOutputArgs,
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
        || scan(&args),
    )
}

fn scan(args: &Args) -> anyhow::Result<ForbiddenTextReport> {
    let needle = forbidden_needle();

    let listing = std::process::Command::new(&args.git)
        .arg("-C")
        .arg(&args.repository_root)
        .args(["grep", "-n", "-F", needle.as_str(), "--", "."])
        .output()
        .context("invoking git grep")?;

    // git grep: exit 0 = matches found, 1 = no matches, >1 = error.
    match listing.status.code() {
        Some(1) => Ok(ForbiddenTextReport {
            schema: FORBIDDEN_TEXT_SCHEMA,
            tracked_matches: Vec::new(),
            valid: true,
        }),
        Some(0) => {
            let text = String::from_utf8(listing.stdout).context("decoding git grep output")?;
            for entry in parse_grep_matches(&text) {
                tracing::error!(
                    path = %entry.path,
                    line = entry.line,
                    policy = %entry.policy,
                    "forbidden token found in tracked file; use Vec<_>, \
                     collect::<Vec<_>>(), or an escaped prose spelling",
                );
            }
            anyhow::bail!("forbidden token found in tracked files");
        }
        other => {
            tracing::error!(
                status = ?other,
                stderr = %String::from_utf8_lossy(&listing.stderr),
                "git grep failed while checking forbidden text",
            );
            anyhow::bail!("git grep failed while checking forbidden text");
        }
    }
}
