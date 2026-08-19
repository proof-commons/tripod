//! `check-target-elements-prototypes`: the compound-prototype gate.
//!
//! Runs one compound-prototype case matrix through an explicitly selected
//! external executor and publishes the typed prototype report. Under
//! ADR-010 this is a one-JSON-result command: direct-mode stdout (refused
//! on a terminal), or a build-mode report asset plus success stamp
//! published in that order (ADR-014).
//!
//! # Why this is a separate command
//!
//! `check-target-elements-native` answers what the reviewed primitives
//! did. This answers whether a multi-step construction held together
//! across a whole target output. One command emitting both would produce
//! a report whose rows a reader could not tell apart, and Guide 10
//! prefers the separate checker for exactly that reason (§6.3). The two
//! share the executor driver, the protocol, and the supervision; they
//! share no case identity, no claim census, and no completeness
//! statement.
//!
//! # One relation per run
//!
//! `--relation` selects the matrix. A run answers one relation, and the
//! report records both the relation and the role that answers for it, so
//! a run cannot file one matrix's coverage under the other's role.
//!
//! # No credential argument
//!
//! There is no `--rpc-user`, `--rpc-password`, `--cookie`, `--token`,
//! `--wallet`, or `--private-key`, and none may be added. The command's
//! whole input surface is the reviewed contract, a relation, two public
//! development identifiers, an executable path, an explicit declaration
//! of what that executable is, a timeout, and the output destinations.
//!
//! If the selected executor talks to a node, it owns that conversation.
//! A regtest node's cookie inside a disposable data directory is public
//! test-fixture material under
//! `(´[ADR015-rule:security:test-material]´)`, and it is the *executor's*
//! material either way: this interface neither accepts a cookie path nor
//! reads one.
//!
//! # A declaration is not a proof
//!
//! `--executor-class` records what the caller says the program is. The
//! harness cannot tell a mock from an interpreter and does not pretend
//! to: declaring a mock refuses the gate outright, and declaring a
//! reviewed runner is provenance a reader may check against the
//! executor's recorded handshake.
//!
//! # What a run establishes
//!
//! Every claim of the selected relation has bearing cases among its own
//! matrix's rows, so a complete run reports the relation's own
//! completeness rather than a partial one. A run against a mock still
//! cannot be evidence: the gate refuses a declared mock outright, and a
//! mock declared otherwise answers with the matrix's own expectations,
//! which the report records in its executor provenance for a reader to
//! catch.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::{Parser, ValueEnum};
use cli_common::{BaseArgs, CheckOutputArgs, install_json_panic_hook, run_check_command};
use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    TargetContractVersion, reviewed_elements_tapscript, validate_reviewed_development_binding,
};
use target_elements_conformance::executor::{
    DEFAULT_EXECUTOR_TIMEOUT, ExecutorConfiguration, ExecutorTrust, execute_canonical_prototypes,
};
use target_elements_conformance::prototype::{
    CanonicalPrototypeMatrix, ConstructorPrototypeMatrix, WideFloorPrototypeMatrix,
    constructor_case_matrix, wide_floor_case_matrix,
};
use target_elements_conformance::prototype_report::PrototypeConformanceReport;
use target_elements_conformance::prototype_validate::{
    PrototypeReportValidationInputs, evaluate_prototypes, prototype_gate, validate_prototype_report,
};

const COMMAND_NAME: &str = "check-target-elements-prototypes";

/// What the caller declares the selected executor to be.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum ExecutorClass {
    /// A mock. Such a run can never satisfy the gate.
    Mock,
    /// The reviewed nonmock runner the gate requires.
    ReviewedNonMock,
}

/// Which compound-prototype matrix to run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum Relation {
    /// The metadata-constructor continuity matrix.
    ConstructorContinuity,
    /// The wide-arithmetic floor matrix.
    WideFloor,
}

#[derive(Parser)]
#[command(
    name = "check-target-elements-prototypes",
    version,
    about = "Execute one compound-prototype matrix through an explicit external executor"
)]
struct Args {
    #[command(flatten)]
    base: BaseArgs,

    /// Which compound-prototype matrix to run. One relation per run.
    #[arg(long, value_name = "RELATION")]
    relation: Relation,

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

/// Runs one matrix and gates the result.
fn run(args: &Args) -> Result<PrototypeConformanceReport, String> {
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
                // an intent the matrix does not exercise.
                [],
            ),
            None,
        ),
    )
    .map_err(|_| "the development binding did not validate".to_owned())?;
    debug_assert_eq!(definition.version(), TargetContractVersion::V2);

    // Selected on this command's own argument rather than on the typed
    // relation. The typed relation is deliberately non-exhaustive, so a
    // match on it here would need a catch-all arm — and the only thing a
    // catch-all could do is run one relation's matrix for the other's
    // name, which is exactly what the role and relation being recorded
    // separately exists to make impossible.
    //
    // Each relation's canonical matrix is now its own type, and the
    // borrowed view handed to the evaluator carries the relation with it,
    // so the relation is no longer a separate argument that could name
    // one matrix while another was supplied.
    let owned = match args.relation {
        Relation::ConstructorContinuity => OwnedCanonicalMatrix::Constructor(
            constructor_case_matrix(&target).map_err(|error| error.to_string())?,
        ),
        Relation::WideFloor => OwnedCanonicalMatrix::WideFloor(
            wide_floor_case_matrix(&target).map_err(|error| error.to_string())?,
        ),
    };
    let matrix = owned.borrowed();

    let configuration = ExecutorConfiguration::new(
        &args.executor,
        match args.executor_class {
            ExecutorClass::Mock => ExecutorTrust::Mock,
            ExecutorClass::ReviewedNonMock => ExecutorTrust::ReviewedNonMock,
        },
        args.executor_timeout_seconds
            .map_or(DEFAULT_EXECUTOR_TIMEOUT, Duration::from_secs),
    );

    let transcript = execute_canonical_prototypes(&target, &binding, &configuration, matrix)
        .map_err(|error| error.to_string())?;
    let report = evaluate_prototypes(&target, &binding, matrix, &transcript)
        .map_err(|error| error.to_string())?;

    // The report this command just built goes back through the validator
    // before the gate sees it. That is not a formality: the validator is
    // the only thing that establishes the report's censuses are exact,
    // and a command that skipped it for its own output would be trusting
    // the one report nobody else has checked.
    let validated = validate_prototype_report(
        report,
        PrototypeReportValidationInputs {
            target: &target,
            binding: &binding,
            matrix,
            transcript: &transcript,
        },
    )
    .map_err(|error| error.to_string())?;

    // The gate decides whether this run is evidence. A refused run
    // publishes nothing: no stdout result, no report asset, and no fresh
    // stamp date.
    prototype_gate(&validated).map_err(|error| error.to_string())?;
    Ok(validated.into_report())
}

/// One relation's canonical matrix, owned for the length of the run.
///
/// The evaluator and the validator take a borrowed
/// [`CanonicalPrototypeMatrix`], which carries its relation with it. The
/// owner has to outlive both, and the two matrices are different types,
/// so this is where the selected one lives.
enum OwnedCanonicalMatrix {
    /// The constructor-continuity matrix.
    Constructor(ConstructorPrototypeMatrix),
    /// The wide-floor matrix.
    WideFloor(WideFloorPrototypeMatrix),
}

impl OwnedCanonicalMatrix {
    /// The borrowed view the evidence path accepts.
    const fn borrowed(&self) -> CanonicalPrototypeMatrix<'_> {
        match self {
            Self::Constructor(matrix) => CanonicalPrototypeMatrix::Constructor(matrix),
            Self::WideFloor(matrix) => CanonicalPrototypeMatrix::WideFloor(matrix),
        }
    }
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
