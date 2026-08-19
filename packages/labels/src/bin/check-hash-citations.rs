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
use cli_common::{BaseArgs, CheckOutputArgs, install_json_panic_hook, run_check_command};
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

fn scan(args: &Args) -> anyhow::Result<HashCitationReport> {
    let table = std::fs::read_to_string(&args.families).context("reading the families table")?;
    let rules = load_rules(&table).context("reading the families table")?;

    let tracked = tracked_files(args)?;
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
    if report.valid {
        report_idle_rules(&report);
        return Ok(report);
    }
    for refusal in &report.refusals {
        tracing::error!(
            path = %refusal.path,
            line = refusal.line,
            column = refusal.column,
            value = %refusal.value,
            shape = %refusal.shape,
            "no rule in the families table describes this value; add a rule \
             naming what it measures and what writes it",
        );
    }
    anyhow::bail!("tracked values that no rule describes");
}

// A rule that claims nothing is reported and never enforced: rules are
// standing claims about kinds of value, and one whose kind is absent
// today is not thereby wrong. Removing it would also break the growth
// rule the table rests on.
fn report_idle_rules(report: &HashCitationReport) {
    let idle: Vec<_> = report
        .claims
        .iter()
        .filter(|claim| claim.occurrences == 0)
        .map(|claim| claim.id.as_str())
        .collect();
    if idle.is_empty() {
        return;
    }
    tracing::info!(
        rules = report.claims.len(),
        idle = idle.len(),
        ids = %idle.join(" "),
        "families that claimed nothing in this tree",
    );
}

fn tracked_files(args: &Args) -> anyhow::Result<Vec<String>> {
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
        tracing::error!(
            status = ?listing.status.code(),
            "git ls-files failed while listing the tracked set",
        );
        anyhow::bail!("git ls-files failed while listing the tracked set");
    }

    let text = String::from_utf8(listing.stdout).context("decoding git ls-files output")?;
    Ok(text
        .split('\0')
        .filter(|entry| !entry.is_empty())
        .map(std::borrow::ToOwned::to_owned)
        .collect())
}
