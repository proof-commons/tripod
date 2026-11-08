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
    CommandExit, command_child_status, command_completed, command_failed_kind, command_span,
    configure_command_diagnostics, emit_control_plane_record, install_command_subscriber,
    parse_args_from, set_panic_payload_reporting_enabled,
};
use execwrap::{ExecError, RoutingConfig, preflight_routing, run};

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
    install_command_subscriber(COMMAND_NAME);

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
    configure_command_diagnostics(args.debug, default_level(&args));

    let cfg = RoutingConfig {
        redirect: args.redirect,
        redirect_output: args.redirect_output,
        redirect_error: args.redirect_error,
        redirect_prefixed: args.redirect_prefixed,
        notify_on_write: args.notify_on_write,
        debug: args.debug,
        quiet: args.quiet,
    };

    // Routing conflicts and missing child-stream routes are usage
    // failures before any file is opened or child is spawned. Caller
    // paths stay out of the fixed control record.
    match preflight_routing(&cfg) {
        Err(ExecError::AmbiguousRedirection { .. }) => {
            let record =
                cli_common::ControlPlaneRecord::usage_error(COMMAND_NAME, "ambiguous_redirection");
            let _ignored = emit_control_plane_record(&record);
            return CommandExit::Usage.exit_code();
        }
        Err(ExecError::UnroutedStream { .. }) => {
            let record =
                cli_common::ControlPlaneRecord::usage_error(COMMAND_NAME, "unrouted_stream");
            let _ignored = emit_control_plane_record(&record);
            return CommandExit::Usage.exit_code();
        }
        Ok(()) => {}
        Err(_) => {
            let span = command_span(COMMAND_NAME, "asset");
            let _entered = span.enter();
            command_failed_kind("routing", "io");
            return CommandExit::Failure.exit_code();
        }
    }

    let span = command_span(COMMAND_NAME, "asset");
    let _entered = span.enter();

    if let Some(directory) = &args.change_directory
        && env::set_current_dir(directory).is_err()
    {
        command_failed_kind("setup", "change_directory");
        return CommandExit::Failure.exit_code();
    }

    let outcome = match run(&args.command, &cfg, |_| {}) {
        Ok(outcome) => outcome,
        Err(ExecError::Spawn { .. }) => {
            command_failed_kind("child", "spawn");
            return CommandExit::Failure.exit_code();
        }
        Err(_) => {
            command_failed_kind("child", "io");
            return CommandExit::Failure.exit_code();
        }
    };

    // Losing captured output is a wrapper failure even when the child
    // succeeded: a build must not silently continue on truncated logs.
    if outcome.data_loss && outcome.exit_code == 0 {
        command_failed_kind("child", "data_loss");
        return CommandExit::Failure.exit_code();
    }

    let exit = wrapper_exit_from_child_status(outcome.exit_code);
    if outcome.exit_code == 0 {
        command_completed("asset", false);
    } else if outcome.data_loss {
        command_failed_kind("child", "data_loss");
    } else if (3..=255).contains(&outcome.exit_code) {
        command_child_status("asset", outcome.exit_code);
    } else {
        command_failed_kind("child", "child_exit");
    }
    exit
}

/// Map a relayed child status onto the wrapper's exit code, keeping the
/// reserved workspace classes intact: 0 stays success, a child 1 or 2 becomes
/// wrapper runtime failure 1 (so code 2 stays reserved for wrapper usage), and
/// child statuses 3..=255 (including `128 + signal`) are relayed unchanged.
fn wrapper_exit_from_child_status(status: i32) -> ExitCode {
    let status = status.clamp(0, i32::from(u8::MAX));

    match status {
        0 => CommandExit::Success.exit_code(),
        1 | 2 => CommandExit::Failure.exit_code(),
        other => ExitCode::from(u8::try_from(other).unwrap_or(1)),
    }
}

const fn default_level(args: &Args) -> tracing::Level {
    if args.quiet {
        tracing::Level::WARN
    } else {
        tracing::Level::INFO
    }
}
