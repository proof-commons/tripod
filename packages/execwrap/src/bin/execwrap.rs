//! `execwrap` binary: spawn a child process and route its stdout/stderr to log
//! files, with optional notifications.
//!
//! This is the Rust port of the legacy `bin/execwrap.pl` script. Diagnostics
//! follow the ADR-010 JSON-on-stderr contract through
//! `cli-common`.

use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use cli_common::{
    CommandExit, emit_control_plane_record, init_json_tracing_with_debug, install_json_panic_hook,
    parse_args_from, set_panic_payload_reporting_enabled,
};
use execwrap::{ExecError, Notification, RoutingConfig, preflight_routing, run};

const COMMAND_NAME: &str = "execwrap";

#[derive(Debug, Parser)]
#[command(
    name = "execwrap",
    version,
    about = "Spawn a command and route its stdout/stderr to log files",
    trailing_var_arg = false
)]
struct Args {
    /// Redirect both STDOUT and STDERR to this file (raw, merged).
    #[arg(long = "redirect", value_name = "FILE")]
    redirect: Option<PathBuf>,

    /// Redirect STDOUT to this file.
    #[arg(long = "redirect-output", value_name = "FILE")]
    redirect_output: Option<PathBuf>,

    /// Redirect STDERR to this file.
    #[arg(long = "redirect-error", value_name = "FILE")]
    redirect_error: Option<PathBuf>,

    /// Redirect both streams to this file with per-stream prefixes.
    #[arg(long = "redirect-prefixed", value_name = "FILE")]
    redirect_prefixed: Option<PathBuf>,

    /// Emit notifications when log files are written.
    #[arg(long = "notify-on-write")]
    notify_on_write: bool,

    /// Change to this directory before executing the command.
    #[arg(long = "change-directory", short = 'C', value_name = "DIR")]
    change_directory: Option<PathBuf>,

    /// Enable debug-level diagnostic logging.
    #[arg(long, short = 'd')]
    debug: bool,

    /// Suppress info-level diagnostic logging.
    #[arg(long, short = 'q')]
    quiet: bool,

    /// Command and its arguments (after `--`).
    #[arg(last = true, required = true, num_args = 1.., value_name = "COMMAND")]
    command: Vec<OsString>,
}

fn main() -> ExitCode {
    // The panic hook precedes argument parsing (ADR-010): a panic
    // anywhere in the process emits one JSON diagnostic, never Rust's
    // text hook.
    install_json_panic_hook(COMMAND_NAME);

    // clap owns help/version/usage handling; the `last = true`
    // positional makes the `--` separator a clap-enforced requirement,
    // so `--help`/`--version` exit 0 with JSON records and every argv
    // failure is usage class 2.
    let args: Args = match parse_args_from::<Args, _, _>(env::args_os()) {
        Ok(parsed) => parsed,
        Err(exit) => {
            let _ignored = emit_control_plane_record(&exit.record);
            return exit.exit_code();
        }
    };

    set_panic_payload_reporting_enabled(args.debug);
    init_json_tracing_with_debug(args.debug, default_level(&args));

    let cfg = RoutingConfig {
        redirect: args.redirect,
        redirect_output: args.redirect_output,
        redirect_error: args.redirect_error,
        redirect_prefixed: args.redirect_prefixed,
        notify_on_write: args.notify_on_write,
        debug: args.debug,
        quiet: args.quiet,
    };

    // Conflicting routing for one path is an invalid argument
    // combination: usage class 2, before any side effect.
    if let Err(error @ ExecError::AmbiguousRedirection { .. }) = preflight_routing(&cfg) {
        let record = cli_common::ControlPlaneRecord::usage_error(
            COMMAND_NAME,
            &error.to_string(),
            "ambiguous_redirection",
        );
        let _ignored = emit_control_plane_record(&record);
        return CommandExit::Usage.exit_code();
    }

    if let Some(directory) = &args.change_directory
        && let Err(error) = env::set_current_dir(directory)
    {
        tracing::error!(
            error = %error,
            directory = %directory.display(),
            "failed to change working directory",
        );
        return CommandExit::Failure.exit_code();
    }

    let outcome = match run(&args.command, &cfg, emit_notification) {
        Ok(outcome) => outcome,
        Err(ExecError::Spawn { program, source }) => {
            tracing::error!(error = %source, program = %program.to_string_lossy(), "failed to spawn child");
            return CommandExit::Failure.exit_code();
        }
        Err(error) => {
            tracing::error!(error = %error, "execwrap setup failed");
            return CommandExit::Failure.exit_code();
        }
    };

    // Losing captured output is a wrapper failure even when the child
    // succeeded: a build must not silently continue on truncated logs.
    if outcome.data_loss && outcome.exit_code == 0 {
        tracing::error!("captured output may be incomplete; treating the run as failed");
        return CommandExit::Failure.exit_code();
    }

    let code = u8::try_from(outcome.exit_code.clamp(0, i32::from(u8::MAX))).unwrap_or(1);
    ExitCode::from(code)
}

const fn default_level(args: &Args) -> tracing::Level {
    if args.quiet {
        tracing::Level::WARN
    } else {
        tracing::Level::INFO
    }
}

fn emit_notification(notification: &Notification) {
    match notification {
        Notification::WritingToLog { path, stream } => {
            tracing::info!(
                path = %path.display(),
                stream = stream.label(),
                "Writing to log ({}): {}",
                stream.label(),
                path.display(),
            );
        }
        Notification::WroteToLog { path } => {
            tracing::info!(
                path = %path.display(),
                "Wrote to log file: {}",
                path.display(),
            );
        }
    }
}
