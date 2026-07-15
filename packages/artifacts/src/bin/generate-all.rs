//! `generate-all`: the workspace's single generated-artifact writer.
//!
//! Renders every artifact in [`artifacts::ARTIFACT_NAMES`] from its
//! typed source and writes it atomically into `--output` (default:
//! the committed `packages/model/generated` directory). The check
//! path is `check-generated`, which never writes.
//!
//! ADR-010: side-effect command; no stdout result data. Written files
//! are assets routed by `--output`; diagnostics are JSON on stderr.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use cli_common::{BaseArgs, install_json_panic_hook, parse_args_or_exit, run_no_stdout_command};

const COMMAND_NAME: &str = "generate-all";

#[derive(Parser)]
#[command(
    name = "generate-all",
    version,
    about = "Regenerate every derivative artifact from its typed source"
)]
struct Args {
    #[command(flatten)]
    base: BaseArgs,

    /// Output directory for the generated artifacts. Defaults to the
    /// committed generated directory (packages/model/generated).
    #[arg(long, value_name = "DIR")]
    output: Option<PathBuf>,
}

fn main() -> ExitCode {
    install_json_panic_hook(COMMAND_NAME);
    let args = parse_args_or_exit::<Args>();

    run_no_stdout_command(COMMAND_NAME, args.base.debug, tracing::Level::INFO, || {
        let output = args.output.clone().unwrap_or_else(artifacts::generated_dir);

        std::fs::create_dir_all(&output)?;

        for artifact in artifacts::expected_artifacts()? {
            let path = output.join(artifact.name);
            artifacts::atomic_write(&path, &artifact.bytes)?;
            tracing::info!(
                path = %path.display(),
                bytes = artifact.bytes.len(),
                "artifact written",
            );
        }

        // The generated directory is owned by this generator: report
        // strays so the census stays explicit (the checker fails them).
        for entry in std::fs::read_dir(&output)? {
            let name = entry?.file_name().to_string_lossy().into_owned();
            if !artifacts::ARTIFACT_NAMES.contains(&name.as_str()) {
                tracing::warn!(
                    file = %name,
                    "unexpected file in generated directory (not owned by generate-all)",
                );
            }
        }

        anyhow::Ok(())
    })
}
