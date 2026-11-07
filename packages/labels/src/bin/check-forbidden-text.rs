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
use cli_common::{BaseArgs, CheckOutputArgs, install_command_subscriber, run_check_command};
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
    install_command_subscriber(COMMAND_NAME);
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
    let enabled = tracing::enabled!(target: "labels::forbidden", tracing::Level::INFO);
    let span = if enabled {
        tracing::info_span!(target: "labels::forbidden", "scan_forbidden_text")
    } else {
        tracing::Span::none()
    };
    let _entered = span.enter();
    let mut child_exit_code = None;
    let mut child_failed = false;
    let result = scan_core(args, &mut child_exit_code, &mut child_failed, enabled);
    if enabled {
        match &result {
            Ok(report) => {
                tracing::info!(target: "labels::forbidden", outcome = "completed", valid = report.valid, refusal_count = u64::try_from(report.tracked_matches.len()).unwrap_or(u64::MAX), "tracked audit completed");
            }
            Err(_) if child_failed => {
                if let Some(code) = child_exit_code {
                    tracing::info!(target: "labels::forbidden", outcome = "failed", error_kind = "git", child_exit_code = code, "tracked audit completed");
                } else {
                    tracing::info!(target: "labels::forbidden", outcome = "failed", error_kind = "git", "tracked audit completed");
                }
            }
            Err(_) => {
                tracing::info!(target: "labels::forbidden", outcome = "failed", error_kind = "environment", "tracked audit completed");
            }
        }
    }
    match result {
        Ok(report) if !report.valid => anyhow::bail!("forbidden token found in tracked files"),
        other => other,
    }
}

fn scan_core(
    args: &Args,
    child_exit_code: &mut Option<i32>,
    child_failed: &mut bool,
    enabled: bool,
) -> anyhow::Result<ForbiddenTextReport> {
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
            let matches = parse_grep_matches(&text);
            for entry in &matches {
                tracing::error!(target: "labels::forbidden", diagnostic_code = "forbidden_token", path = entry.path.as_str(), line = u64::from(entry.line), policy = entry.policy.as_str(), detail = "forbidden token found in tracked file; use Vec<_>, collect::<Vec<_>>(), or an escaped prose spelling", "forbidden text found");
            }
            Ok(ForbiddenTextReport {
                schema: FORBIDDEN_TEXT_SCHEMA,
                tracked_matches: matches,
                valid: false,
            })
        }
        other => {
            // The git program is argument-supplied, so its stderr is
            // arbitrary child output, not a typed field (ADR-010/ADR-015).
            // Report only the process status; omit the raw child stderr.
            *child_failed = true;
            if enabled {
                *child_exit_code = other;
            }
            anyhow::bail!("git grep failed while checking forbidden text");
        }
    }
}
