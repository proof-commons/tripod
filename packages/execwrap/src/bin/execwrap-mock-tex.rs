//! `execwrap-mock-tex`: a build/test-internal command that fabricates the
//! deterministic outputs of a mocked TeX child (`xelatex`, `biber`, or
//! `latexmk`) so the Meson graph can be exercised without a real TeX
//! toolchain.
//!
//! This is deliberately NOT the production wrapper. `execwrap` always executes
//! the child command it is given; test simulation is unreachable through it.
//! Under ADR-010 this is a side-effect command: it writes its outputs under
//! `--outdir`, emits no stdout, and reports every diagnostic as JSON on stderr.

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use cli_common::{
    CommandExit, command_completed, command_failed, command_span, configure_command_diagnostics,
    emit_control_plane_record, install_command_subscriber, parse_args_from,
    set_panic_payload_reporting_enabled,
};
use execwrap::{MockChild, run_mock_child};

const COMMAND_NAME: &str = "execwrap-mock-tex";

#[derive(Debug, Parser)]
#[command(
    name = "execwrap-mock-tex",
    version,
    about = "Fabricate deterministic mock TeX outputs for the build graph (test-only)"
)]
struct Args {
    /// Which TeX child to simulate.
    #[arg(long = "child", value_name = "KIND")]
    child: MockChild,

    /// Directory the simulated child writes its outputs to.
    #[arg(long = "outdir", value_name = "DIR")]
    outdir: PathBuf,

    /// Fail without writing any output (injected render failure).
    #[arg(long = "fail")]
    fail: bool,

    /// Enable debug-level diagnostic logging.
    #[arg(long, short = 'd')]
    debug: bool,
}

fn main() -> ExitCode {
    install_command_subscriber(COMMAND_NAME);

    let args: Args = match parse_args_from::<Args, _, _>(env::args_os()) {
        Ok(parsed) => parsed,
        Err(exit) => {
            let _ignored = emit_control_plane_record(&exit.record);
            return exit.exit_code();
        }
    };

    set_panic_payload_reporting_enabled(args.debug);
    configure_command_diagnostics(args.debug, tracing::Level::INFO);
    let span = command_span(COMMAND_NAME, "asset");
    let _entered = span.enter();

    // Failure injection belongs to the mock tool, not the production wrapper:
    // no output is written, mirroring a real render failure.
    if args.fail {
        command_failed("mock", &"injected_failure");
        return CommandExit::Failure.exit_code();
    }

    if run_mock_child(args.child, &args.outdir).is_ok() {
        command_completed("asset", false);
        CommandExit::Success.exit_code()
    } else {
        command_failed("mock", &"io");
        CommandExit::Failure.exit_code()
    }
}
