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
    CommandExit, emit_control_plane_record, init_json_tracing_with_debug, install_json_panic_hook,
    parse_args_from, set_panic_payload_reporting_enabled,
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
    // The panic hook precedes argument parsing (ADR-010): a panic anywhere in
    // the process emits one JSON diagnostic, never Rust's text hook.
    install_json_panic_hook(COMMAND_NAME);

    let args: Args = match parse_args_from::<Args, _, _>(env::args_os()) {
        Ok(parsed) => parsed,
        Err(exit) => {
            let _ignored = emit_control_plane_record(&exit.record);
            return exit.exit_code();
        }
    };

    set_panic_payload_reporting_enabled(args.debug);
    init_json_tracing_with_debug(args.debug, tracing::Level::INFO);

    // Failure injection belongs to the mock tool, not the production wrapper:
    // no output is written, mirroring a real render failure.
    if args.fail {
        tracing::error!("mock TeX child failure injected");
        return CommandExit::Failure.exit_code();
    }

    match run_mock_child(args.child, &args.outdir) {
        Ok(()) => CommandExit::Success.exit_code(),
        Err(error) => {
            tracing::error!(error = %error, "mock TeX child failed to write outputs");
            CommandExit::Failure.exit_code()
        }
    }
}
