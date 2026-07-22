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
use cli_common::{BaseArgs, CheckOutputArgs, install_json_panic_hook, run_check_command};
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
    install_json_panic_hook(COMMAND_NAME);
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
    let pattern =
        regex::Regex::new(&args.exclude_pattern).context("compiling the exclusion pattern")?;

    let listing = std::process::Command::new(&args.git)
        .arg("-C")
        .arg(&args.repository_root)
        .args(["ls-files", "-z"])
        .output()
        .context("invoking git ls-files")?;
    if !listing.status.success() {
        // The git program is argument-supplied, so its stderr is
        // arbitrary child output, not a typed field (ADR-010). Report
        // only the process status and a fixed message; the raw child
        // stderr is omitted rather than relayed or heuristically
        // redacted.
        tracing::error!(
            status = %listing.status,
            "git ls-files failed",
        );
        anyhow::bail!("git ls-files failed");
    }
    let tracked = String::from_utf8(listing.stdout).context("decoding git ls-files output")?;

    let report = audit_census(
        tracked.split('\0').filter(|path| !path.is_empty()),
        args.declared.iter().map(String::as_str),
        args.excluded.iter().map(String::as_str),
        &pattern,
    );
    if report.valid {
        return Ok(report);
    }
    for path in &report.missing_from_census {
        tracing::error!(
            path = %path,
            "tracked lint subject is missing from its directory's meson.build census list",
        );
    }
    for path in &report.not_tracked {
        tracing::error!(
            path = %path,
            "declared census entry is not a tracked lint subject; git add it or drop the list entry",
        );
    }
    anyhow::bail!("the hand-managed build census disagrees with git ls-files");
}
