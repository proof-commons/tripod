use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use cli_common::{BaseArgs, install_json_panic_hook, run_no_stdout_command};

const COMMAND_NAME: &str = "generate-label-registers";

#[derive(Parser)]
#[command(
    name = "generate-label-registers",
    version,
    about = "Generate planning upstream-label registers"
)]
struct Args {
    #[command(flatten)]
    base: BaseArgs,
    #[arg(long, value_name = "DIR")]
    repository_root: Option<PathBuf>,
    #[arg(long, value_name = "DIR")]
    output_root: PathBuf,
}

fn main() -> ExitCode {
    // Installed before parsing so an early panic still fails closed
    // with a JSON-only record (ADR-010 early-startup rule).
    install_json_panic_hook(COMMAND_NAME);
    let args = cli_common::parse_args_or_exit::<Args>();
    run_no_stdout_command(COMMAND_NAME, args.base.debug, tracing::Level::INFO, || {
        let paths = args.repository_root.map_or_else(
            labels::RepositoryPaths::workspace_default,
            labels::RepositoryPaths::from_root,
        );
        let registers = labels::generate_registers(&paths, &args.output_root)
                .inspect_err(|error| {
                    for diagnostic in error.diagnostics() {
                        tracing::error!(code = ?diagnostic.code, path = %diagnostic.path, line = diagnostic.line, message = %diagnostic.message, "label source validation failed");
                    }
                })?;
        for register in registers {
            tracing::info!(path = %register.path.display(), bytes = register.bytes, "label register written");
        }
        Ok::<(), labels::repository::GenerateError>(())
    })
}
