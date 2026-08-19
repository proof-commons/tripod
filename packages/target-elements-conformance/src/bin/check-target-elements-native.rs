//! `check-target-elements-native`: the target-native primitive gate.
//!
//! Runs the canonical fixture census through an explicitly selected
//! external executor and publishes the typed conformance report. Under
//! ADR-010 this is a one-JSON-result command: direct-mode stdout
//! (refused on a terminal), or a build-mode report asset plus success
//! stamp published in that order (ADR-014).
//!
//! # No credential argument
//!
//! There is no `--rpc-user`, `--rpc-password`, `--cookie`, `--token`,
//! `--wallet`, or `--private-key`, and none may be added. The command's
//! whole input surface is the reviewed contract, two public development
//! identifiers, an executable path, an explicit declaration of what that
//! executable is, a timeout, and the output destinations.
//!
//! If the selected executor talks to a node, it owns that conversation.
//! A regtest node's cookie inside a disposable data directory is public
//! test-fixture material under
//! `(´[ADR015-rule:security:test-material]´)` — generated for a throwaway
//! chain, authorizing nothing of value, discarded with the environment —
//! but it is the *executor's* material either way: this interface
//! neither accepts a cookie path nor reads one, so the classification
//! never has to be relied upon here.
//!
//! # A declaration is not a proof
//!
//! `--executor-class` records what the caller says the program is. The
//! harness cannot tell a mock from an interpreter, and does not pretend
//! to: declaring a mock refuses the gate outright, and declaring a
//! reviewed runner is provenance a reader may check against the
//! executor's recorded handshake and the backlog gate record.
//!
//! # What a run establishes
//!
//! The canonical census covers every reviewed primitive, and every
//! required evidence row has cases bearing on it. Three rows remain
//! outside the required plan by design — sighash semantics, commitment
//! equality, and confidential-value conservation — so a complete run
//! reports partial completeness rather than a clean sweep, which is the
//! honest state of the evidence.
//!
//! A run against a mock still cannot be evidence: the gate refuses a
//! declared mock outright, and a mock declared otherwise answers with the
//! census's own expectations, which the report records in its executor
//! provenance for a reader to catch.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::{Parser, ValueEnum};
use cli_common::{BaseArgs, CheckOutputArgs, install_json_panic_hook, run_check_command};
use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    TargetContractVersion, reviewed_elements_tapscript, validate_reviewed_development_binding,
};
use target_elements_conformance::claim::claim_registry;
use target_elements_conformance::executor::{
    DEFAULT_EXECUTOR_TIMEOUT, ExecutorConfiguration, ExecutorTrust, execute_canonical,
};
use target_elements_conformance::fixture::canonical_fixture_set;
use target_elements_conformance::report::NativeConformanceReport;
use target_elements_conformance::validate::{
    NativeReportValidationInputs, evaluate, gate, guide_nine_evidence_plan, validate_native_report,
};

const COMMAND_NAME: &str = "check-target-elements-native";

/// What the caller declares the selected executor to be.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum ExecutorClass {
    /// A mock. Such a run can never satisfy the gate.
    Mock,
    /// The reviewed nonmock runner the gate requires.
    ReviewedNonMock,
}

#[derive(Parser)]
#[command(
    name = "check-target-elements-native",
    version,
    about = "Execute the reviewed target primitives through an explicit external executor"
)]
struct Args {
    #[command(flatten)]
    base: BaseArgs,

    /// The external executor to run. Selecting it grants execution
    /// authority (ADR-015); it receives no arguments and no secrets.
    #[arg(long, value_name = "PROGRAM")]
    executor: PathBuf,

    /// What the caller declares that executor to be.
    #[arg(long, value_name = "CLASS")]
    executor_class: ExecutorClass,

    /// How long the whole run may take before the executor is stopped.
    #[arg(long, value_name = "SECONDS")]
    executor_timeout_seconds: Option<u64>,

    /// The public development network identifier, as 64 hex digits.
    #[arg(long, value_name = "HEX")]
    network_id: String,

    /// The public development genesis identifier, as 64 hex digits.
    #[arg(long, value_name = "HEX")]
    genesis_id: String,

    #[command(flatten)]
    output: CheckOutputArgs,
}

fn main() -> ExitCode {
    // Installed before parsing so an early panic still fails closed with
    // a JSON-only record (ADR-010 early-startup rule).
    install_json_panic_hook(COMMAND_NAME);
    let args = cli_common::parse_args_or_exit::<Args>();
    run_check_command(
        COMMAND_NAME,
        args.base.debug,
        tracing::Level::INFO,
        &args.output,
        || run(&args),
    )
}

/// Runs the census and gates the result.
fn run(args: &Args) -> Result<NativeConformanceReport, String> {
    let network_id = identifier(&args.network_id, "network")?;
    let genesis_id = identifier(&args.genesis_id, "genesis")?;

    let target = reviewed_elements_tapscript()
        .map_err(|_| "the reviewed target contract did not validate".to_owned())?;
    let definition = target.definition();
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            definition.version(),
            DeploymentEnvironment::Development,
            network_id,
            genesis_id,
            ActivationDeclaration::new(
                true,
                LeafVersion::TAPSCRIPT,
                // The run relies on no capability beyond the reviewed
                // domain itself: a declaration naming more would assert
                // an intent the fixture census does not exercise.
                [],
            ),
            None,
        ),
    )
    .map_err(|_| "the development binding did not validate".to_owned())?;
    debug_assert_eq!(definition.version(), TargetContractVersion::V2);

    let fixtures = canonical_fixture_set(&target, &binding).map_err(|error| error.to_string())?;
    let plan = guide_nine_evidence_plan().map_err(|error| error.to_string())?;
    let registry = claim_registry().map_err(|error| error.to_string())?;

    let configuration = ExecutorConfiguration::new(
        &args.executor,
        match args.executor_class {
            ExecutorClass::Mock => ExecutorTrust::Mock,
            ExecutorClass::ReviewedNonMock => ExecutorTrust::ReviewedNonMock,
        },
        args.executor_timeout_seconds
            .map_or(DEFAULT_EXECUTOR_TIMEOUT, Duration::from_secs),
    );

    let transcript = execute_canonical(&target, &binding, &configuration, &fixtures)
        .map_err(|error| error.to_string())?;
    let report = evaluate(&target, &binding, &fixtures, &transcript, &plan, &registry)
        .map_err(|error| error.to_string())?;

    // The report this command just built goes back through the validator
    // before the gate sees it. That is not a formality: the validator is
    // the only thing that establishes the report's censuses are exact,
    // and a command that skipped it for its own output would be trusting
    // the one report nobody else has checked.
    let validated = validate_native_report(
        report,
        NativeReportValidationInputs {
            target: &target,
            binding: &binding,
            fixtures: &fixtures,
            plan: &plan,
            registry: &registry,
            transcript: &transcript,
        },
    )
    .map_err(|error| error.to_string())?;

    // The gate decides whether this run is evidence. A refused run
    // publishes nothing: no stdout result, no report asset, and no fresh
    // stamp date.
    gate(&validated).map_err(|error| error.to_string())?;
    Ok(validated.into_report())
}

/// One 32-byte public identifier, from 64 hex digits.
fn identifier(text: &str, role: &str) -> Result<[u8; 32], String> {
    let digits: Vec<u8> = text.as_bytes().to_vec();
    if digits.len() != 64 {
        return Err(format!("the {role} identifier must be 64 hex digits"));
    }
    let mut bytes = [0_u8; 32];
    for (index, pair) in digits.as_chunks::<2>().0.iter().enumerate() {
        let high = digit(pair[0]).ok_or_else(|| format!("the {role} identifier is not hex"))?;
        let low = digit(pair[1]).ok_or_else(|| format!("the {role} identifier is not hex"))?;
        bytes[index] = (high << 4) | low;
    }
    Ok(bytes)
}

/// One hexadecimal digit's value.
const fn digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
