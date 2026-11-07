// Standard-input-only public observation target.

use cli_common::{CommandExit, install_command_subscriber, run_stdout_json_command};
use std::io::{self, IsTerminal, Read};
use std::process::ExitCode;
use vectors::maturity_observer::ObserverManifest;
use vectors::maturity_observer::ObserverNodeReader;
use vectors::maturity_observer::observe_maturity;
use vectors::maturity_observer::render_maturity_observation;

fn main() -> ExitCode {
    install_command_subscriber("maturity-observer-cli");
    if io::stdin().is_terminal() {
        return CommandExit::Success.exit_code();
    }
    let mut input = String::new();
    let read = io::stdin().read_to_string(&mut input);
    // An ordinary test run has no acquisition and makes no observation claim.
    if read.is_ok() && input.is_empty() {
        return CommandExit::Success.exit_code();
    }
    run_stdout_json_command(
        "maturity-observer-cli",
        false,
        tracing::Level::INFO,
        move || {
            read.map_err(|error| error.to_string())?;
            let manifest = ObserverManifest::parse(&input)?;
            let reader = ObserverNodeReader::parse(&input)?;
            let result = observe_maturity(&manifest, &reader)?;
            serde_json::from_str::<serde_json::Value>(&render_maturity_observation(&result))
                .map_err(|error| error.to_string())
        },
    )
}
