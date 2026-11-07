//! `census-audit`: welds the hand-managed build census to git.
//!
//! The build system passes the declared census (composed from the
//! per-directory meson.build lists), a categorical exclusion pattern,
//! the explicit per-directory exclusion files, and the git program to
//! use. This binary invokes `git ls-files` itself so the audit is
//! fresh on every build — a tracked file added without a list entry
//! fails immediately, not at the next reconfigure. On success the JSON
//! report is published under the active output mode (ADR-014):
//! direct-mode stdout, or a build-mode report asset plus success stamp.

use std::{path::PathBuf, process::ExitCode};

use anyhow::Context;
use clap::Parser;
use cli_common::{BaseArgs, CheckOutputArgs, install_command_subscriber, run_check_command};
use labels::census::audit_census;

const COMMAND_NAME: &str = "census-audit";

#[derive(Parser)]
#[command(
    name = "census-audit",
    version,
    about = "Audit the hand-managed build census against git ls-files"
)]
struct Args {
    #[command(flatten)]
    base: BaseArgs,
    /// Repository root the tracked-file listing is taken from.
    #[arg(long, value_name = "DIR")]
    repository_root: PathBuf,
    /// The git program to invoke for the tracked-file listing.
    #[arg(long, value_name = "PROGRAM")]
    git: PathBuf,
    /// Regex matching tracked paths that are categorically not lint
    /// subjects (build definitions, licences, archives, ...).
    #[arg(long, value_name = "REGEX")]
    exclude_pattern: String,
    /// Same-typed files deliberately outside the census, declared in
    /// their directory's meson.build exclusion list.
    #[arg(long = "excluded", value_name = "FILE")]
    excluded: Vec<String>,
    #[command(flatten)]
    output: CheckOutputArgs,
    /// The declared census: every lint subject, repository-relative.
    #[arg(value_name = "FILE")]
    declared: Vec<String>,
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
        || audit(&args),
    )
}

fn audit(args: &Args) -> anyhow::Result<labels::census::CensusAuditReport> {
    let enabled = tracing::enabled!(target: "labels::census", tracing::Level::INFO);
    let span = if enabled {
        tracing::info_span!(target: "labels::census", "audit_census", declared_file_count = u64::try_from(args.declared.len()).unwrap_or(u64::MAX))
    } else {
        tracing::Span::none()
    };
    let _entered = span.enter();
    let mut child_exit_code = None;
    let mut child_failed = false;
    let result = audit_core(args, &mut child_exit_code, &mut child_failed, enabled);
    if enabled {
        match &result {
            Ok(report) => {
                tracing::info!(target: "labels::census", outcome = "completed", valid = report.valid, scanned_file_count = u64::try_from(report.tracked).unwrap_or(u64::MAX), refusal_count = u64::try_from(report.missing_from_census.len().saturating_add(report.not_tracked.len()).saturating_add(report.disallowed_modes.len())).unwrap_or(u64::MAX), "tracked audit completed");
            }
            Err(_) if child_failed => {
                if let Some(code) = child_exit_code {
                    tracing::info!(target: "labels::census", outcome = "failed", error_kind = "git", child_exit_code = code, "tracked audit completed");
                } else {
                    tracing::info!(target: "labels::census", outcome = "failed", error_kind = "git", "tracked audit completed");
                }
            }
            Err(_) => {
                tracing::info!(target: "labels::census", outcome = "failed", error_kind = "environment", "tracked audit completed");
            }
        }
    }
    match result {
        Ok(report) if !report.valid && report.disallowed_modes.is_empty() => {
            anyhow::bail!("the hand-managed build census disagrees with git ls-files")
        }
        Ok(report) if !report.valid => {
            anyhow::bail!("the tracked repository contains an entry that is not an ordinary blob")
        }
        other => other,
    }
}

fn audit_core(
    args: &Args,
    child_exit_code: &mut Option<i32>,
    child_failed: &mut bool,
    enabled: bool,
) -> anyhow::Result<labels::census::CensusAuditReport> {
    let pattern =
        regex::Regex::new(&args.exclude_pattern).context("compiling the exclusion pattern")?;

    // The listing carries Git modes, so this one call answers both
    // census membership and repository shape (ADR-014 tracked-entry
    // modes). No other first-party tool re-derives either fact.
    let listing = std::process::Command::new(&args.git)
        .arg("-C")
        .arg(&args.repository_root)
        .args(["ls-files", "--stage", "-z"])
        .output()
        .context("invoking git ls-files")?;
    if !listing.status.success() {
        // The git program is argument-supplied, so its stderr is
        // arbitrary child output, not a typed field (ADR-010). Report
        // only the process status and a fixed message; the raw child
        // stderr is omitted rather than relayed or heuristically
        // redacted.
        *child_failed = true;
        if enabled {
            *child_exit_code = listing.status.code();
        }
        anyhow::bail!("git ls-files failed");
    }
    let tracked = String::from_utf8(listing.stdout).context("decoding git ls-files output")?;
    let entries = labels::census::parse_tracked_listing(&tracked)
        .map_err(|error| anyhow::anyhow!("{error}"))
        .context("parsing the mode-bearing tracked-file listing")?;

    let report = audit_census(
        entries,
        args.declared.iter().map(String::as_str),
        args.excluded.iter().map(String::as_str),
        &pattern,
    );
    for path in &report.missing_from_census {
        tracing::error!(target: "labels::census", diagnostic_code = "missing_from_census", path = path.as_str(), detail = "tracked lint subject is missing from its directory's meson.build census list", "census mismatch");
    }
    for path in &report.not_tracked {
        tracing::error!(target: "labels::census", diagnostic_code = "not_tracked", path = path.as_str(), detail = "declared census entry is not a tracked lint subject; git add it or drop the list entry", "census mismatch");
    }
    for defect in &report.disallowed_modes {
        if tracing::enabled!(target: "labels::census", tracing::Level::ERROR) {
            tracing::error!(target: "labels::census", diagnostic_code = "disallowed_mode", path = defect.path.as_str(), mode = defect.mode.as_str(), detail = "tracked entry is not an ordinary blob; the repository carries no symlinks, gitlinks, or submodules", "census mismatch");
        }
    }
    Ok(report)
}
