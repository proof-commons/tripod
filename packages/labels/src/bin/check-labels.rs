use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use cli_common::{BaseArgs, install_json_panic_hook, run_stdout_json_command};

const COMMAND_NAME: &str = "check-labels";

#[derive(Parser)]
#[command(
    name = "check-labels",
    version,
    about = "Check repository documentation labels without writing"
)]
struct Args {
    #[command(flatten)]
    base: BaseArgs,
    #[arg(long, value_name = "DIR")]
    repository_root: Option<PathBuf>,
}

fn main() -> ExitCode {
    // Installed before parsing so an early panic still fails closed
    // with a JSON-only record (ADR-010 early-startup rule).
    install_json_panic_hook(COMMAND_NAME);
    let args = cli_common::parse_args_or_exit::<Args>();
    run_stdout_json_command(COMMAND_NAME, args.base.debug, tracing::Level::INFO, || {
        let paths = args.repository_root.map_or_else(
            labels::RepositoryPaths::workspace_default,
            labels::RepositoryPaths::from_root,
        );
        let (report, diagnostics) = labels::check_repository(&paths);
        if report.valid {
            return Ok(report);
        }
        for diagnostic in diagnostics {
            tracing::error!(code = ?diagnostic.code, path = %diagnostic.path, line = diagnostic.line, message = %diagnostic.message, "label check failed");
        }
        Err("repository label validation failed")
    })
}
