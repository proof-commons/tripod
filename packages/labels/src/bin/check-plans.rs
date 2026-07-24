//! `check-plans`: the non-writing plan-tree structure and hygiene gate.
//!
//! Subject files arrive by argument (ADR-014): the build system states
//! census membership, this binary re-verifies it against the on-disk
//! discovery. On success the JSON report is published under the active
//! output mode (ADR-014): direct-mode stdout, or a build-mode report
//! asset plus success stamp.

use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use cli_common::{BaseArgs, CheckOutputArgs, install_json_panic_hook, run_check_command};

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
    /// Census subject Markdown files under adr/ and plans/.
    #[arg(long = "subject", value_name = "FILE")]
    subjects: Vec<PathBuf>,
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
        || {
            let outcome = labels::plans::check_plans(&args.repository_root, &args.subjects)?;
            tracing::info!(
                adr_bytes = outcome.report.adr_bytes,
                plans_bytes = outcome.report.plans_bytes,
                combined_bytes = outcome.report.combined_bytes,
                hard_cap_bytes = outcome.report.hard_cap_bytes,
                soft_target_bytes = outcome.report.soft_target_bytes,
                "markdown weight"
            );
            for warning in &outcome.warnings {
                tracing::warn!(message = %warning, "plan-tree warning");
            }
            if outcome.report.soft_target_exceeded {
                tracing::warn!("combined Markdown exceeds soft target");
            }
            if outcome.report.valid {
                return Ok(outcome.report);
            }
            for failure in &outcome.failures {
                tracing::error!(message = %failure, "plan-tree check failed");
            }
            Err(anyhow::anyhow!("plan-tree validation failed"))
        },
    )
}
