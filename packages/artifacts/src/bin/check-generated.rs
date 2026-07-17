//! `check-generated`: the non-writing stale-artifact checker.
//!
//! Computes the expected bytes of every generated artifact in memory
//! and compares them with the committed files. Never writes. The
//! writing path is `generate-all`. The generated directory and the
//! scoped label census arrive by argument (ADR-014); on success the
//! optional `--stamp` file is touched for the build graph.
//!
//! ADR-010: stdout result data (one JSON [`artifacts::CheckReport`]
//! object) is emitted only on success (exit 0: everything current);
//! on failure stdout stays empty, per-artifact verdicts are emitted
//! as JSON diagnostics on stderr, and the exit code is the branch
//! signal. TTY refusal applies.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use cli_common::{
    BaseArgs, CommandExit, emit_control_plane_record, init_json_tracing_with_debug,
    install_json_panic_hook, parse_args_or_exit, set_panic_payload_reporting_enabled,
    stdout_tty_refusal_record, touch_stamp,
};

const COMMAND_NAME: &str = "check-generated";

#[derive(Parser)]
#[command(
    name = "check-generated",
    version,
    about = "Check committed generated artifacts against their typed sources without writing"
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

    /// Generated directory to check.
    #[arg(long, value_name = "DIR")]
    generated_dir: PathBuf,

    /// Stamp file touched on success (ADR-014 output-or-stamp).
    #[arg(long, value_name = "FILE")]
    stamp: Option<PathBuf>,
}

fn main() -> ExitCode {
    install_json_panic_hook(COMMAND_NAME);
    let args = parse_args_or_exit::<Args>();

    set_panic_payload_reporting_enabled(args.base.debug);
    init_json_tracing_with_debug(args.base.debug, tracing::Level::INFO);

    if let Some(record) = stdout_tty_refusal_record(COMMAND_NAME) {
        let _ignored = emit_control_plane_record(&record);
        return CommandExit::Usage.exit_code();
    }

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
    let dir = resolve(&args.generated_dir);

    let report = match artifacts::check(&dir, &census) {
        Ok(report) => report,
        Err(error) => {
            tracing::error!(error = %error, "check failed");
            return CommandExit::Failure.exit_code();
        }
    };

    if !report.current {
        for artifact in &report.artifacts {
            if artifact.status != artifacts::ArtifactFreshness::Current {
                tracing::error!(
                    artifact = %artifact.name,
                    status = ?artifact.status,
                    "generated artifact is not current; run \
                     `meson compile -C <builddir> generate-artifacts` \
                     and commit the diff",
                );
            }
        }
        for stray in &report.unexpected {
            tracing::error!(
                file = %stray,
                "unexpected file in generated directory",
            );
        }
        return CommandExit::Failure.exit_code();
    }

    if let Some(stamp) = &args.stamp
        && let Err(error) = touch_stamp(stamp)
    {
        tracing::error!(error = %error, "failed to touch the stamp file");
        return CommandExit::Failure.exit_code();
    }

    match cli_common::emit(&report) {
        Ok(()) => CommandExit::Success.exit_code(),
        Err(error) => {
            tracing::error!(error = %error, "failed to write JSON report to stdout");
            CommandExit::Failure.exit_code()
        }
    }
}
