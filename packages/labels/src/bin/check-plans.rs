//! `check-plans`: the non-writing plan-tree structure and hygiene gate.
//!
//! Subject files arrive by argument (ADR-014): the build system states
//! census membership, this binary re-verifies it against the on-disk
//! discovery. On success the JSON report is published under the active
//! output mode (ADR-014): direct-mode stdout, or a build-mode report
//! asset plus success stamp.

use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use cli_common::{BaseArgs, CheckOutputArgs, install_command_subscriber, run_check_command};

const COMMAND_NAME: &str = "check-plans";

#[derive(Parser)]
#[command(
    name = "check-plans",
    version,
    about = "Check plan-tree documentation structure without writing"
)]
struct Args {
    #[command(flatten)]
    base: BaseArgs,
    /// Repository root; relative subject paths resolve against it.
    #[arg(long, value_name = "DIR")]
    repository_root: PathBuf,
    /// Census subject Markdown files under adr/ and plans/. At least
    /// one is required: the build system states census membership and
    /// this binary re-verifies it, so an invocation with no declared
    /// census is a usage error rather than a discovery-driven run
    /// (ADR-014).
    #[arg(long = "subject", value_name = "FILE", required = true)]
    subjects: Vec<PathBuf>,
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
        || {
            let outcome = labels::plans::check_plans(&args.repository_root, &args.subjects)?;
            for warning in &outcome.warnings {
                tracing::warn!(target: "labels::plans", diagnostic_code = "plan_warning", detail = warning.as_str(), "plan check warning");
            }
            if outcome.report.soft_target_exceeded {
                tracing::warn!(target: "labels::plans", diagnostic_code = "soft_target_exceeded", detail = "combined Markdown exceeds soft target", "plan check warning");
            }
            if outcome.report.valid {
                return Ok(outcome.report);
            }
            for failure in &outcome.failures {
                tracing::error!(target: "labels::plans", diagnostic_code = "plan_failure", detail = failure.as_str(), "plan check failed");
            }
            Err(anyhow::anyhow!("plan-tree validation failed"))
        },
    )
}
