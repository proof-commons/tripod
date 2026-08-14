//! `generate-all`: the workspace's single generated-artifact writer.
//!
//! Renders every artifact in [`artifacts::ARTIFACT_NAMES`] from its
//! typed source and writes it atomically into `--output`. The scoped
//! label census feeding the model-label publication arrives by
//! argument (ADR-014). The check path is `check-generated`, which
//! never writes.
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

    /// Repository root; relative subject paths resolve against it.
    #[arg(long, value_name = "DIR")]
    repository_root: PathBuf,

    /// The Attestation specification root TeX file.
    #[arg(long, value_name = "FILE")]
    attestation_main: PathBuf,

    /// Attestation specification section TeX files.
    #[arg(long = "attestation-section", value_name = "FILE")]
    attestation_sections: Vec<PathBuf>,

    /// The realization Markdown document.
    #[arg(long, value_name = "FILE")]
    realization: PathBuf,

    /// Model crate Rust sources.
    #[arg(long = "model-source", value_name = "FILE")]
    model_sources: Vec<PathBuf>,

    /// Output directory for the generated artifacts.
    #[arg(long, value_name = "DIR")]
    output: PathBuf,
}

fn main() -> ExitCode {
    install_json_panic_hook(COMMAND_NAME);
    let args = parse_args_or_exit::<Args>();

    run_no_stdout_command(COMMAND_NAME, args.base.debug, tracing::Level::INFO, || {
        let root = &args.repository_root;
        let resolve = |path: &PathBuf| labels::RepositoryCensus::resolve(root, path.clone());
        let census = labels::RepositoryCensus {
            root: root.clone(),
            attestation_main: resolve(&args.attestation_main),
            attestation_sections: args.attestation_sections.iter().map(resolve).collect(),
            realization: resolve(&args.realization),
            model_sources: args.model_sources.iter().map(resolve).collect(),
            ..labels::RepositoryCensus::default()
        };
        let output = resolve(&args.output);

        std::fs::create_dir_all(&output)?;

        // Batch publication (T3): derive and stage every artifact
        // before the first final path changes, so a failed generation
        // cannot leave the generated directory in mixed generations.
        let artifacts = artifacts::expected_artifacts(&census)?;
        let paths = artifacts
            .iter()
            .map(|artifact| output.join(artifact.name))
            .collect::<Vec<_>>();
        let assets = artifacts
            .iter()
            .zip(&paths)
            .map(|(artifact, path)| cli_common::PublicationAsset {
                role: artifact.name,
                path,
                bytes: &artifact.bytes,
            })
            .collect::<Vec<_>>();

        for result in cli_common::publish_batch(&assets)? {
            tracing::info!(
                path = %result.path.display(),
                bytes_changed = result.change.bytes_changed,
                mode_changed = result.change.mode_changed,
                "artifact published",
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
