//! Standard-input-only public observation target.

use std::io::{self, IsTerminal, Read, Write};
use vectors::maturity_observer::ObserverManifest;
use vectors::maturity_observer::ObserverNodeReader;
use vectors::maturity_observer::observe_maturity;
use vectors::maturity_observer::render_maturity_observation;

fn main() -> Result<(), String> {
    if io::stdin().is_terminal() {
        return Ok(());
    }
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| e.to_string())?;
    // An ordinary test run has no acquisition and makes no observation claim.
    if input.is_empty() {
        return Ok(());
    }
    let manifest = ObserverManifest::parse(&input)?;
    let reader = ObserverNodeReader::parse(&input)?;
    let result = observe_maturity(&manifest, &reader)?;
    writeln!(io::stdout(), "{}", render_maturity_observation(&result)).map_err(|e| e.to_string())
}
