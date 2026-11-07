//! `check-hash-citations`: repository audit accounting for every
//! hexadecimal value in the tracked tree.
//!
//! This is an always-fresh repository audit (ADR-014): it asks git for
//! the complete tracked set on every build rather than tracking a fixed
//! input list, so a value committed into a new file is adjudicated
//! immediately. Every occurrence must be claimed by a rule in the
//! committed families table, which names what the value measures and
//! what writes it; an occurrence nothing claims fails the check.
//!
//! The audit places rather than recomputes. Whether a digest is correct
//! belongs to the test that owns it, and re-deriving it here would
//! duplicate that suite and fail for reasons that are not lint failures.
//! On a clean tree it publishes a JSON report through the shared
//! two-mode checker contract; a refusal emits JSON diagnostics on stderr
//! and fails without publishing a report or touching the stamp.

use std::{path::PathBuf, process::ExitCode};

use anyhow::Context;
use clap::Parser;
use cli_common::{BaseArgs, CheckOutputArgs, install_command_subscriber, run_check_command};
use labels::hashcite::{HashCitationReport, Occurrence, adjudicate, load_rules, scan_file};

const COMMAND_NAME: &str = "check-hash-citations";

#[derive(Parser)]
#[command(
    name = "check-hash-citations",
    version,
    about = "Account for every hexadecimal value in the tracked tree"
)]
struct Args {
    #[command(flatten)]
    base: BaseArgs,
    /// Repository root the tracked-file scan runs in.
    #[arg(long, value_name = "DIR")]
    repository_root: PathBuf,
    /// The git program to invoke for the tracked-file listing.
    #[arg(long, value_name = "PROGRAM")]
    git: PathBuf,
    /// The committed families table naming the permitted values.
    #[arg(long, value_name = "FILE")]
    families: PathBuf,
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

fn scan(args: &Args) -> anyhow::Result<HashCitationReport> {
    let enabled = tracing::enabled!(target: "labels::hashcite", tracing::Level::INFO);
    let span = if enabled {
        tracing::info_span!(target: "labels::hashcite", "scan_hash_citations")
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
                tracing::info!(target: "labels::hashcite", outcome = "completed", valid = report.valid, scanned_file_count = u64::try_from(report.files_scanned).unwrap_or(u64::MAX), refusal_count = u64::try_from(report.refusals.len()).unwrap_or(u64::MAX), rule_count = u64::try_from(report.claims.len()).unwrap_or(u64::MAX), idle_rule_count = u64::try_from(report.claims.iter().filter(|claim| claim.occurrences == 0).count()).unwrap_or(u64::MAX), "tracked audit completed");
            }
            Err(_) if child_failed => {
                if let Some(code) = child_exit_code {
                    tracing::info!(target: "labels::hashcite", outcome = "failed", error_kind = "git", child_exit_code = code, "tracked audit completed");
                } else {
                    tracing::info!(target: "labels::hashcite", outcome = "failed", error_kind = "git", "tracked audit completed");
                }
            }
            Err(_) => {
                tracing::info!(target: "labels::hashcite", outcome = "failed", error_kind = "environment", "tracked audit completed");
            }
        }
    }
    match result {
        Ok(report) if !report.valid => anyhow::bail!("tracked values that no rule describes"),
        other => other,
    }
}

fn scan_core(
    args: &Args,
    child_exit_code: &mut Option<i32>,
    child_failed: &mut bool,
    enabled: bool,
) -> anyhow::Result<HashCitationReport> {
    let table = std::fs::read_to_string(&args.families).context("reading the families table")?;
    let rules = load_rules(&table).context("reading the families table")?;

    let tracked = tracked_files(args, child_exit_code, child_failed, enabled)?;
    let mut occurrences: Vec<Occurrence> = Vec::new();
    let mut files_scanned = 0usize;
    for relative in &tracked {
        // A tracked file that is not text carries no value a reader
        // could cite, so it is out of the population rather than an
        // error: refusing to run over a repository that stores an image
        // would make the audit unusable for the sake of nothing.
        let Ok(text) = std::fs::read_to_string(args.repository_root.join(relative)) else {
            continue;
        };
        files_scanned += 1;
        occurrences.extend(scan_file(relative, &text));
    }

    let report = adjudicate(&rules, &occurrences, files_scanned);
    for refusal in &report.refusals {
        tracing::error!(target: "labels::hashcite", diagnostic_code = "unclaimed_hash_citation", path = refusal.path.as_str(), line = u64::from(refusal.line), column = u64::from(refusal.column), shape = refusal.shape.as_str(), detail = "no rule in the families table describes this value; add a rule naming what it measures and what writes it", "hash citation unclaimed");
    }
    Ok(report)
}

fn tracked_files(
    args: &Args,
    child_exit_code: &mut Option<i32>,
    child_failed: &mut bool,
    enabled: bool,
) -> anyhow::Result<Vec<String>> {
    let listing = std::process::Command::new(&args.git)
        .arg("-C")
        .arg(&args.repository_root)
        .args(["ls-files", "-z"])
        .output()
        .context("invoking git ls-files")?;

    if !listing.status.success() {
        // The git program is argument-supplied, so its stderr is
        // arbitrary child output, not a typed field (ADR-010/ADR-015).
        // Report only the process status; omit the raw child stderr.
        *child_failed = true;
        if enabled {
            *child_exit_code = listing.status.code();
        }
        anyhow::bail!("git ls-files failed while listing the tracked set");
    }

    let text = String::from_utf8(listing.stdout).context("decoding git ls-files output")?;
    Ok(text
        .split('\0')
        .filter(|entry| !entry.is_empty())
        .map(std::borrow::ToOwned::to_owned)
        .collect())
}
