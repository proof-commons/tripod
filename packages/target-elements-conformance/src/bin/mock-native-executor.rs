//! `mock-native-executor`: a test-only child that speaks the executor
//! protocol badly on purpose.
//!
//! This is NOT an Elements executor. It runs no interpreter, executes no
//! script, and observes nothing about any target.
//!
//! # Where its answers come from, now that the request has none
//!
//! They used to be echoes of the expectation carried in the request.
//! Protocol revision 3 removed it: a request carries the execution
//! subject and nothing else, so there is no answer in it to read
//! `(´[PLAN-rule:guide11-exec:request-subject]´)`.
//!
//! So this mock keeps its own table instead, and the two halves of it
//! reach the mock by the two routes the guide admits.
//!
//! Primitive verdicts come from the mock's own copy of the canonical
//! census, constructed from the reviewed contract over the development
//! binding it states it observed, and indexed by case identity. It is
//! authored the first time a primitive request arrives rather than at
//! startup, so a prototype-only run never pays for it.
//!
//! Compound verdicts come through this command's own configuration: the
//! `--prototype-verdicts` file, written by whoever spawns the mock. That
//! is deliberate rather than symmetric. Authoring the compound matrices
//! grinds nonces and costs seconds apiece, and several mock processes
//! doing it at once turned a fast suite slow enough that a loaded machine
//! could push a run past the harness's timeout — reporting the mock's own
//! startup cost as a protocol failure.
//!
//! Either way a case identity the table does not hold is answered by the
//! `--unknown-case` policy for a primitive, and refused outright for a
//! compound one: inventing a spend verdict for a construction nobody
//! stated an outcome for is exactly the guess the refusal branch exists
//! to avoid.
//!
//! None of this is evidence, and for the same reason as before: the
//! answers are the very expectations the harness will compare them
//! against, so a green run against this mock says only that the harness
//! can compare a value with itself. What changed is that the
//! self-comparison now travels out of band, through channels no wire
//! request touches and an honest adapter has no equivalent of.
//!
//! It exists so the protocol's failure paths — a malformed line, a wrong
//! schema, a duplicated, missing, reordered, or unexpected result, a
//! child that dies, a child that hangs, a child that writes noise on
//! stderr — can be driven to failure by a real subprocess. A run using
//! it must be declared a mock run, and the gate refuses such a run
//! whatever the report says (Guide-9 §1.6, §11.8).
//!
//! It also answers tree-bearing requests, so that a validation test can
//! drive both of a constructor case's outcomes without an Elements node.
//! Materializing here means recomputing the stated construction with
//! this package's own oracle and comparing the stated predecessor
//! program and control block against what that oracle determines — no
//! transaction is built and no target is consulted, so the answer is
//! still the table's and still not evidence. A
//! construction that does not check out, and every construction under
//! the refusing behavior, is refused rather than answered.
//!
//! Under ADR-010 this command's stdout is protocol data, in the NDJSON
//! form the harness's protocol documents.
//!
//! # The two diagnostic destinations
//!
//! The harness hands every executor an `--output` file for its own typed
//! facts and an `--elements-output` file for raw text out of whatever it
//! ran. This mock takes both and writes to both, because a mock that
//! ignored them would let the harness pass an unusable path forever with
//! nothing noticing.
//!
//! Its `noisy-stderr` behavior still writes on stderr, and that is the
//! point of it: an arbitrary caller-selected executor may write anywhere,
//! and what the harness promises is that nothing it wrote there is
//! relayed. The reviewed adapter's promise — that it writes nothing there
//! at all — is a different promise, checked where that adapter lives.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use cli_common::{
    CommandExit, emit_control_plane_record, install_json_panic_hook, parse_args_from,
};
use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    TargetContractVersion, reviewed_elements_tapscript, validate_reviewed_development_binding,
};
use target_elements_conformance::constructor::tree::construct;
use target_elements_conformance::fixture::{
    ExpectedPrimitiveOutcome, NativeCaseId, canonical_fixture_set,
};
use target_elements_conformance::protocol::{
    ExecutorCapability, ExecutorEnvironmentObservation, ExecutorHandshake,
    MOCK_EXECUTOR_GENESIS_ID, MOCK_EXECUTOR_NETWORK_ID, NATIVE_PROTOCOL_SCHEMA,
    NativeExecutionRequest, NativeExecutionResponse, NativePrototypeRequest,
    NativePrototypeResponse, NativeResourceObservation, NativeVerdict, ObservedFailureClass,
    WireEnvironment, WireExecutionDomain,
};
use target_elements_conformance::prototype::{
    ExpectedPrototypeOutcome, PrototypeCaseId, PrototypeConstruction, PrototypeRelation,
};

const COMMAND_NAME: &str = "mock-native-executor";

/// How the mock misbehaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum Behavior {
    /// Answer every case from this mock's own census-derived table.
    AnswerFromCensus,
    /// Answer the handshake with a schema the harness does not speak.
    WrongHandshakeSchema,
    /// Answer the handshake as an executor of the previous revision.
    ///
    /// Not a variation on the wrong-schema behavior above, which offers a
    /// revision that never existed. This one is the migration case: a
    /// working revision-2 adapter, of exactly the kind this workspace ran
    /// before, meeting a revision-3 harness. It must be refused loudly
    /// rather than have its records read as revision-3 ones
    /// `(´[PLAN-rule:guide11-exec:request-subject]´)`.
    PreviousRevisionHandshake,
    /// Write a line that is not JSON at all.
    MalformedJson,
    /// Write a response carrying a field the protocol does not define.
    UnknownField,
    /// Answer a case with the wrong protocol schema.
    WrongResponseSchema,
    /// Answer the first case twice.
    DuplicateResult,
    /// Answer a case that was never asked about.
    UnexpectedResult,
    /// Answer the first request with the second case's identity.
    ReorderedResult,
    /// Stop answering after the handshake.
    MissingResult,
    /// Write one further line after the last response.
    TrailingData,
    /// Report infrastructure trouble instead of a verdict.
    InfrastructureError,
    /// Exit before the handshake.
    DieEarly,
    /// Answer everything, then exit nonzero.
    ExitNonzero,
    /// Never answer at all.
    Hang,
    /// Answer correctly, but write a secret-shaped line on stderr first.
    NoisyStderr,
    /// Write a blank record before the handshake.
    BlankRecord,
    /// Write a handshake record larger than the harness's bound.
    OversizedHandshake,
    /// Write an oversized handshake record and never terminate it.
    UnterminatedHandshake,
    /// State an environment observation naming another chain's genesis.
    WrongGenesis,
    /// State an environment observation naming another network.
    WrongNetwork,
    /// State an environment observation in which the reviewed leaf is
    /// not active.
    InactiveLeafVersion,
    /// Answer every case as accepted while also naming a failure class.
    AcceptedWithFailureClass,
    /// Write a blank record after the last response.
    TrailingBlankRecord,
    /// State no environment observation at all.
    MissingEnvironment,
    /// Recompute a stated construction and answer the case where it
    /// checks out.
    MaterializeTree,
    /// Refuse every stated construction as one it cannot build exactly.
    RefuseTreeMaterialization,
}

/// How much filler an oversized record carries.
///
/// Past the harness's handshake bound, so that the refusal is the bound
/// rather than the size of any real message.
const OVERSIZED_RECORD_BYTES: usize = 128 * 1024;

/// This mock's own handshake.
fn handshake(schema: u32) -> ExecutorHandshake {
    ExecutorHandshake {
        protocol_schema: schema,
        adapter_name: "mock-native-executor".to_owned(),
        adapter_version: "0.0.0".to_owned(),
        framework_revision: None,
        node_name: "mock-native-executor".to_owned(),
        node_version: "0.0.0".to_owned(),
        // A mock builds nothing and executes nothing, so it states no
        // binary revision, no intended tip, and no upstream base. The
        // report then records that this run establishes no workspace
        // provenance, which is exactly true of it.
        binary_reported_revision: None,
        intended_executed_tip: None,
        upstream_base: None,
        included_local_topics: BTreeSet::new(),
        supported_domains: BTreeSet::from([WireExecutionDomain::Tapscript]),
        supported_leaf_versions: BTreeSet::from([TAPSCRIPT_LEAF_VERSION]),
        capabilities: BTreeSet::from([
            ExecutorCapability::FinalStackReporting,
            ExecutorCapability::FinalAltstackReporting,
            ExecutorCapability::FailureClassReporting,
            ExecutorCapability::TransactionContext,
            // Advertised because this mock can recompute a stated
            // construction with the package's own oracle and say whether
            // the stated program and control block are the ones it
            // determines. That is a check, not an execution: no
            // transaction is built and no target is consulted, which is
            // why a run against this mock is still not evidence.
            ExecutorCapability::TreeMaterialization,
            // Advertised for the same reason and with the same limit: it
            // reads the record and checks the construction, and it still
            // executes nothing.
            ExecutorCapability::CompoundPrototypeFixtures,
        ]),
        // Not advertised, and the absence is the honest answer: this
        // mock materializes nothing, mines nothing, and reads nothing
        // back, so it holds neither the capability nor an advertisement
        // to go with it. An executor claiming one without the other is
        // refused at the handshake, and a mock is not exempt from that.
        confidential_funding: None,
    }
}

/// The environment this mock says it ran on.
fn environment(behavior: Behavior) -> ExecutorEnvironmentObservation {
    let mut observation = ExecutorEnvironmentObservation {
        schema: NATIVE_PROTOCOL_SCHEMA,
        environment: WireEnvironment::Development,
        chain_name: "mock-development-chain".to_owned(),
        network_id: MOCK_EXECUTOR_NETWORK_ID,
        genesis_id: MOCK_EXECUTOR_GENESIS_ID,
        active_domains: BTreeSet::from([WireExecutionDomain::Tapscript]),
        active_leaf_versions: BTreeSet::from([TAPSCRIPT_LEAF_VERSION]),
    };
    match behavior {
        Behavior::WrongGenesis => observation.genesis_id = [0xee; 32],
        Behavior::WrongNetwork => observation.network_id = [0xdd; 32],
        Behavior::InactiveLeafVersion => observation.active_leaf_versions = BTreeSet::new(),
        _ => {}
    }
    observation
}

/// The leaf version byte the reviewed contract fixes.
const TAPSCRIPT_LEAF_VERSION: u8 = 0xc4;

/// What this mock answers a case its own table does not hold.
///
/// The protocol tests drive the exchange with small ad hoc censuses whose
/// case identities belong to no canonical census, and those tests are
/// about framing rather than about verdicts. Naming the answer as an
/// argument keeps it out of the wire request: it is this command's own
/// configuration, chosen by the wrapper script that selects the mock,
/// which is exactly the out-of-band channel a real adapter would use for
/// its own settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum UnknownCasePolicy {
    /// Answer that the target accepted the spend.
    Accept,
    /// Answer that the target rejected the spend.
    Reject,
    /// Answer that the case could not be run at all.
    Refuse,
}

/// This mock's own answers, keyed by case identity.
///
/// Never derived from a request. A primitive case the table does not hold
/// is answered by the [`UnknownCasePolicy`], and a census that cannot be
/// authored at all leaves every case to that policy — the mock states what
/// it can answer rather than failing a protocol test over a census it did
/// not need. A compound case the table does not hold is refused outright.
///
/// # Why the primitive half is authored on first use
///
/// Every canonical primitive run needs the whole census, so the mock
/// keeps its own copy — but a prototype-only run needs none of it, and a
/// protocol test driving two ad hoc cases needs none of it either.
/// Authoring it before the handshake made both wait for a census they
/// never asked about, and a run whose executor is still authoring
/// fixtures when the harness's timeout expires is reported as a timeout:
/// the eager version turned the mock's own startup cost into a protocol
/// failure. It is therefore authored the first time a primitive request
/// arrives.
struct AnswerTable {
    primitives: std::sync::OnceLock<BTreeMap<NativeCaseId, ExpectedPrimitiveOutcome>>,
    prototypes: BTreeMap<String, ExpectedPrototypeOutcome>,
    unknown: UnknownCasePolicy,
}

impl AnswerTable {
    /// A table under one unknown-case policy and one compound verdict
    /// file.
    ///
    /// # Why the two halves come from different places
    ///
    /// The primitive census is cheap to author and every canonical
    /// primitive run needs all of it, so the mock authors its own copy.
    /// The compound matrices are not: authoring the constructor matrix
    /// grinds nonces, and four mock processes doing it at once turned a
    /// fast test suite into a slow one that a loaded machine could push
    /// past the harness's timeout.
    ///
    /// So compound verdicts arrive through this command's own
    /// configuration instead — a file the test that spawns the mock
    /// writes from the matrix it already holds. That is the other route
    /// the guide admits, and it is the more honest of the two about what
    /// a mock is: the answers plainly come from outside, through a
    /// channel no wire request touches
    /// `(´[PLAN-rule:guide11-exec:request-subject]´)`.
    fn new(unknown: UnknownCasePolicy, verdicts: Option<&std::path::Path>) -> Self {
        let prototypes = verdicts
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|text| {
                serde_json::from_str::<BTreeMap<String, ExpectedPrototypeOutcome>>(&text).ok()
            })
            .unwrap_or_default();
        Self {
            primitives: std::sync::OnceLock::new(),
            prototypes,
            unknown,
        }
    }

    /// What the canonical primitive census expects of one case.
    fn primitive(&self, case: NativeCaseId) -> Option<&ExpectedPrimitiveOutcome> {
        self.primitives
            .get_or_init(|| {
                let mut table = BTreeMap::new();
                if let Some(target) = reviewed()
                    && let Some(binding) = mock_binding(&target)
                    && let Ok(census) = canonical_fixture_set(&target, &binding)
                {
                    for fixture in census.iter() {
                        table.insert(fixture.case(), fixture.expected().clone());
                    }
                }
                table
            })
            .get(&case)
    }

    /// What this mock was told to answer for one compound case.
    fn prototype(&self, case: &PrototypeCaseId) -> Option<ExpectedPrototypeOutcome> {
        self.prototypes.get(&case.to_string()).copied()
    }
}

/// The reviewed contract, where it validates.
fn reviewed() -> Option<target_elements::ReviewedElementsTapscriptDefinition> {
    reviewed_elements_tapscript().ok()
}

/// The development binding this mock states it observed.
fn mock_binding(
    target: &target_elements::ReviewedElementsTapscriptDefinition,
) -> Option<target_elements::ReviewedDevelopmentBinding> {
    let declared = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V2,
        DeploymentEnvironment::Development,
        MOCK_EXECUTOR_NETWORK_ID,
        MOCK_EXECUTOR_GENESIS_ID,
        ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
        None,
    );
    validate_reviewed_development_binding(target, declared).ok()
}

/// The fixed text the noisy behavior writes on its stderr.
///
/// The harness must never relay it. It is a test-only string and names
/// nothing.
const NOISE: &str = "MOCK_EXECUTOR_STDERR_THAT_MUST_NOT_BE_RELAYED";

#[derive(Parser)]
#[command(
    name = "mock-native-executor",
    version,
    about = "Speak the native executor protocol badly on purpose (test-only)"
)]
struct Args {
    /// How the mock misbehaves.
    #[arg(long, value_name = "BEHAVIOR")]
    behavior: Behavior,
    /// What to answer a case this mock's own table does not hold.
    #[arg(long, value_name = "POLICY", default_value = "accept")]
    unknown_case: UnknownCasePolicy,
    /// The compound verdicts this mock answers with, as a JSON object
    /// keyed by the case's `relation::name` spelling.
    ///
    /// This command's own configuration, established outside the
    /// harness's interface exactly as a real adapter's would be. Nothing
    /// here reaches the wire, and a compound case the file does not name
    /// is refused rather than guessed.
    #[arg(long, value_name = "PATH")]
    prototype_verdicts: Option<std::path::PathBuf>,

    /// Where this executor writes its own typed diagnostics.
    ///
    /// The harness passes this to every executor it starts, so the mock
    /// takes it on the same terms a real adapter does: required, and
    /// written to. A mock that ignored it would let the harness pass an
    /// unwritable path forever without any test noticing.
    #[arg(long, value_name = "FILE")]
    output: std::path::PathBuf,

    /// Where this executor quarantines raw text out of its own children.
    ///
    /// The mock has no children, so it files one entry saying which
    /// behavior it ran. The point of writing here at all is that the
    /// destination is exercised.
    #[arg(long, value_name = "FILE")]
    elements_output: std::path::PathBuf,
}

/// Records that this mock started, in both destinations the harness named.
///
/// Failing to write is not fatal here. The mock's subject is the wire
/// protocol, and a diagnostic file it could not open is a fact about the
/// host rather than about the exchange under test; the harness's own
/// startup path is what refuses an unusable destination.
fn note_startup(args: &Args) {
    let typed = format!("mock-native-executor: behavior {:?}\n", args.behavior);
    let _ignored = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&args.output)
        .and_then(|mut file| file.write_all(typed.as_bytes()));
    let _ignored = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&args.elements_output)
        .and_then(|mut file| file.write_all(b"----- mock-native-executor: no child text -----\n"));
}

fn main() -> ExitCode {
    install_json_panic_hook(COMMAND_NAME);
    let args: Args = match parse_args_from::<Args, _, _>(std::env::args_os()) {
        Ok(parsed) => parsed,
        Err(exit) => {
            let _ignored = emit_control_plane_record(&exit.record);
            return exit.exit_code();
        }
    };

    note_startup(&args);
    let table = AnswerTable::new(args.unknown_case, args.prototype_verdicts.as_deref());
    run(args.behavior, &table)
        .map_or_else(|_| CommandExit::Failure.exit_code(), CommandExit::exit_code)
}

/// Writes one record the strict framing defines nothing for.
///
/// Answers whether the exchange is over: each of these ends the mock's
/// part, because the harness must refuse the record rather than read on.
fn malformed_record(stdout: &mut impl Write, behavior: Behavior) -> std::io::Result<bool> {
    match behavior {
        Behavior::MalformedJson => writeln!(stdout, "{{this is not json")?,
        // Not skipped, and not filler: the framing has no empty record.
        Behavior::BlankRecord => writeln!(stdout)?,
        // Two shapes of oversized record: one that terminates past the
        // bound and one that never terminates at all. The harness must
        // refuse both without reading past the bound.
        Behavior::OversizedHandshake | Behavior::UnterminatedHandshake => {
            let filler = "x".repeat(OVERSIZED_RECORD_BYTES);
            write!(stdout, "{{\"padding\":\"{filler}\"")?;
            if behavior == Behavior::OversizedHandshake {
                writeln!(stdout, "}}")?;
            }
        }
        _ => return Ok(false),
    }
    stdout.flush()?;
    if behavior == Behavior::UnterminatedHandshake {
        // Stay alive holding the unterminated record open, so the
        // refusal is the harness's bound rather than an end of stream
        // that would have ended the read anyway.
        std::thread::sleep(std::time::Duration::from_secs(30));
    }
    Ok(true)
}

/// Speaks the protocol in the selected way.
fn run(behavior: Behavior, table: &AnswerTable) -> std::io::Result<CommandExit> {
    if behavior == Behavior::DieEarly {
        return Ok(CommandExit::Failure);
    }
    if behavior == Behavior::NoisyStderr {
        let mut stderr = std::io::stderr();
        writeln!(stderr, "{NOISE}")?;
        stderr.flush()?;
    }

    let input = std::io::stdin();
    let mut stdout = std::io::stdout();

    // The handshake request is read and discarded: what it says does not
    // change how this mock behaves.
    let _request = read_line(&input)?;

    if behavior == Behavior::Hang {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    if malformed_record(&mut stdout, behavior)? {
        return Ok(CommandExit::Success);
    }

    let schema = match behavior {
        Behavior::WrongHandshakeSchema => NATIVE_PROTOCOL_SCHEMA + 1,
        Behavior::PreviousRevisionHandshake => NATIVE_PROTOCOL_SCHEMA - 1,
        _ => NATIVE_PROTOCOL_SCHEMA,
    };
    write_json(&mut stdout, &handshake(schema))?;

    if behavior == Behavior::MissingEnvironment {
        return Ok(CommandExit::Success);
    }
    write_json(&mut stdout, &environment(behavior))?;

    if behavior == Behavior::MissingResult {
        return Ok(CommandExit::Success);
    }

    let mut first: Option<NativeExecutionResponse> = None;
    while let Some(line) = read_line(&input)? {
        if line.trim().is_empty() {
            continue;
        }
        // A prototype request is its own record shape, so it is tried
        // first: a primitive request never parses as one, and the
        // misbehaviors below are all statements about a primitive case
        // identity and so do not apply to it.
        if let Ok(request) = serde_json::from_str::<NativePrototypeRequest>(&line) {
            write_json(&mut stdout, &answer_prototype(&request, behavior, table))?;
            continue;
        }
        let Ok(request) = serde_json::from_str::<NativeExecutionRequest>(&line) else {
            return Ok(CommandExit::Failure);
        };
        let response = answer(&request, behavior, table);

        match behavior {
            Behavior::UnknownField => {
                let mut value = serde_json::to_value(&response)
                    .map_err(|error| std::io::Error::other(error.to_string()))?;
                if let Some(object) = value.as_object_mut() {
                    object.insert("commentary".to_owned(), serde_json::Value::from("extra"));
                }
                write_json(&mut stdout, &value)?;
            }
            Behavior::ReorderedResult => {
                let mut answer = response.clone();
                answer.case = shifted_case(&request);
                write_json(&mut stdout, &answer)?;
            }
            Behavior::UnexpectedResult => {
                let mut answer = response.clone();
                answer.case = unknown_case();
                write_json(&mut stdout, &answer)?;
            }
            Behavior::DuplicateResult => {
                let answer = first.clone().unwrap_or_else(|| response.clone());
                write_json(&mut stdout, &answer)?;
            }
            _ => write_json(&mut stdout, &response)?,
        }

        if first.is_none() {
            first = Some(response);
        }
    }

    if behavior == Behavior::TrailingData {
        write_json(&mut stdout, &serde_json::json!({"unsolicited": true}))?;
    }
    if behavior == Behavior::TrailingBlankRecord {
        writeln!(stdout)?;
        stdout.flush()?;
    }

    Ok(if behavior == Behavior::ExitNonzero {
        CommandExit::Failure
    } else {
        CommandExit::Success
    })
}

/// This mock's own answer for one case.
///
/// The case identity is looked up in the table this mock built for
/// itself. Nothing in the request is consulted for the *answer*: the
/// subject supplies the script and the stack, which are restatements of
/// what was handed over and not observations, and the construction is
/// checked rather than believed.
fn answer(
    request: &NativeExecutionRequest,
    behavior: Behavior,
    table: &AnswerTable,
) -> NativeExecutionResponse {
    let schema = if behavior == Behavior::WrongResponseSchema {
        NATIVE_PROTOCOL_SCHEMA + 7
    } else {
        NATIVE_PROTOCOL_SCHEMA
    };

    if behavior == Behavior::InfrastructureError {
        return refusal(schema, request);
    }

    // A construction is a requirement to materialize one exact tree, so a
    // case bearing one is answered only where this mock has checked it.
    // Echoing the expectation for a construction it did not check would be
    // the silent substitution the stated tree exists to prevent, and a
    // refusal is not a target verdict.
    if let Some(construction) = &request.construction
        && !materializes(
            construction,
            PrototypeRelation::MetadataConstructorContinuity,
            behavior,
        )
    {
        return refusal(schema, request);
    }

    // The subject's own figures. These are the two exact rows, and both
    // are restatements of what the executor was handed rather than
    // measurements of an execution — which is why a mock that measures
    // nothing may still state them.
    let resources = NativeResourceObservation {
        script_bytes: request.subject.script.len() as u64,
        initial_stack_items: request.subject.initial_stack.len() as u64,
        ..NativeResourceObservation::default()
    };

    // This mock advertises stack reporting, so every answer states both
    // stacks. Where the table fixes none, the mock states the empty one:
    // it measured nothing either way, and a response that advertised
    // stack reporting and then omitted the stack would be malformed
    // protocol rather than a weak observation.
    let stack = |stated: &Option<Vec<Vec<u8>>>| Some(stated.clone().unwrap_or_default());

    if behavior == Behavior::AcceptedWithFailureClass {
        return NativeExecutionResponse {
            schema,
            case: request.case,
            verdict: NativeVerdict::Accepted,
            final_stack: Some(Vec::new()),
            final_altstack: Some(Vec::new()),
            observed_failure: Some(ObservedFailureClass::EvaluatedFalse),
            resources,
        };
    }

    let Some(stated) = table.primitive(request.case) else {
        return unknown_primitive(schema, request, resources, table.unknown);
    };

    match stated {
        ExpectedPrimitiveOutcome::Accept {
            static_final_stack,
            static_final_altstack,
        } => NativeExecutionResponse {
            schema,
            case: request.case,
            verdict: NativeVerdict::Accepted,
            final_stack: stack(static_final_stack),
            final_altstack: stack(static_final_altstack),
            observed_failure: None,
            resources,
        },
        ExpectedPrimitiveOutcome::Reject {
            classes,
            static_final_stack,
            static_final_altstack,
        } => NativeExecutionResponse {
            schema,
            case: request.case,
            verdict: NativeVerdict::Rejected,
            final_stack: stack(static_final_stack),
            final_altstack: stack(static_final_altstack),
            // The first class the table admits, which is the only one a
            // mock could pick without measuring anything.
            observed_failure: classes.iter().next().copied(),
            resources,
        },
    }
}

/// One case this mock's table does not hold, answered by its policy.
fn unknown_primitive(
    schema: u32,
    request: &NativeExecutionRequest,
    resources: NativeResourceObservation,
    policy: UnknownCasePolicy,
) -> NativeExecutionResponse {
    match policy {
        UnknownCasePolicy::Accept => NativeExecutionResponse {
            schema,
            case: request.case,
            verdict: NativeVerdict::Accepted,
            final_stack: Some(Vec::new()),
            final_altstack: Some(Vec::new()),
            observed_failure: None,
            resources,
        },
        UnknownCasePolicy::Reject => NativeExecutionResponse {
            schema,
            case: request.case,
            verdict: NativeVerdict::Rejected,
            final_stack: Some(Vec::new()),
            final_altstack: Some(Vec::new()),
            // The one class a refused spend always exhibits from
            // outside, since this mock distinguishes none it observed.
            observed_failure: Some(ObservedFailureClass::NonSingletonFinalStack),
            resources,
        },
        UnknownCasePolicy::Refuse => refusal(schema, request),
    }
}

/// One case refused, carrying no observation of any kind.
///
/// A refusal says the executor did not run the case. Every field
/// describing what a target did is absent, because a run that did not
/// happen observed nothing.
fn refusal(schema: u32, request: &NativeExecutionRequest) -> NativeExecutionResponse {
    NativeExecutionResponse {
        schema,
        case: request.case,
        verdict: NativeVerdict::InfrastructureError,
        final_stack: None,
        final_altstack: None,
        observed_failure: None,
        resources: NativeResourceObservation::default(),
    }
}

/// Whether this mock can account for the stated construction exactly.
///
/// # What the check is, and what it is not
///
/// The construction is recomputed with this package's own oracle, and
/// the stated predecessor program and control block are compared against
/// what that oracle determines. Nothing is executed: the mock builds no
/// transaction, boots no node, and observes no target. So a construction
/// that checks out lets the case be answered from the fixture's own
/// expectation as every other case here is, and a run against this mock
/// remains a mock run.
///
/// [`Behavior::RefuseTreeMaterialization`] answers false whatever the
/// construction says, so that the refusal branch can be driven by a
/// coherent fixture rather than only by a broken one.
fn materializes(
    construction: &PrototypeConstruction,
    relation: PrototypeRelation,
    behavior: Behavior,
) -> bool {
    if behavior != Behavior::MaterializeTree {
        return false;
    }
    let Ok(built) = construct(
        &construction.internal_key,
        &construction.tree,
        &construction.executing_leaf,
    ) else {
        return false;
    };
    if built.output_program() != construction.predecessor_program {
        return false;
    }
    if let Some(stated) = &construction.control
        && stated != built.control_block()
    {
        return false;
    }
    // An output the transaction could not carry uniquely is one this
    // mock has no single answer for either. Which roles those are is the
    // relation's own question: the wide-floor pattern reads no
    // transaction field and requires no output at all.
    let required = relation.required_output_roles();
    if construction.outputs.len() != required.len() {
        return false;
    }
    required.iter().all(|role| {
        construction
            .outputs
            .iter()
            .filter(|output| output.role == *role)
            .count()
            == 1
    })
}

/// One compound-prototype case, answered from this mock's own table.
///
/// The same lookup the primitive path performs, over the same
/// construction check. Nothing is executed here either: what the mock
/// establishes is that the record round-trips and that the stated
/// construction is the one the oracle determines.
fn answer_prototype(
    request: &NativePrototypeRequest,
    behavior: Behavior,
    table: &AnswerTable,
) -> NativePrototypeResponse {
    let schema = if behavior == Behavior::WrongResponseSchema {
        NATIVE_PROTOCOL_SCHEMA + 7
    } else {
        NATIVE_PROTOCOL_SCHEMA
    };

    let resources = NativeResourceObservation {
        script_bytes: request.subject.script.len() as u64,
        initial_stack_items: request.subject.initial_stack.len() as u64,
        ..NativeResourceObservation::default()
    };

    let stated = table.prototype(&request.subject.case);
    if behavior == Behavior::InfrastructureError
        || stated.is_none()
        || !materializes(
            &request.subject.construction,
            request.subject.case.relation,
            behavior,
        )
    {
        // A compound case this mock's table does not hold is refused
        // rather than answered under the unknown-case policy: the
        // primitive policy exists for the protocol tests' ad hoc
        // censuses, and every prototype exchange in this workspace runs a
        // canonical matrix. Inventing a spend verdict for a construction
        // nobody stated an outcome for would be the invention the refusal
        // branch exists to avoid.
        return NativePrototypeResponse {
            schema,
            case: request.case.clone(),
            verdict: NativeVerdict::InfrastructureError,
            final_stack: None,
            final_altstack: None,
            observed_failure: None,
            resources: NativeResourceObservation::default(),
        };
    }

    match stated.unwrap_or(ExpectedPrototypeOutcome::Rejected) {
        ExpectedPrototypeOutcome::Accepted => NativePrototypeResponse {
            schema,
            case: request.case.clone(),
            verdict: NativeVerdict::Accepted,
            // A spend verdict exposes no interpreter stack, and this
            // mock has none to expose. It advertises stack reporting, so
            // it states the empty stack rather than omitting the field.
            final_stack: Some(Vec::new()),
            final_altstack: Some(Vec::new()),
            observed_failure: None,
            resources,
        },
        ExpectedPrototypeOutcome::Rejected => NativePrototypeResponse {
            schema,
            case: request.case.clone(),
            verdict: NativeVerdict::Rejected,
            final_stack: Some(Vec::new()),
            final_altstack: Some(Vec::new()),
            // The mock distinguishes no class it observed, and it
            // advertises class reporting, so it names the one class a
            // refused spend always exhibits from outside: evaluation did
            // not end in one true item.
            observed_failure: Some(ObservedFailureClass::NonSingletonFinalStack),
            resources,
        },
        // An outcome this mock has not been taught is not one it can
        // echo. Answering it as either verdict would be inventing a
        // target result, so it reports that it could not run the case.
        _ => NativePrototypeResponse {
            schema,
            case: request.case.clone(),
            verdict: NativeVerdict::InfrastructureError,
            final_stack: None,
            final_altstack: None,
            observed_failure: None,
            resources: NativeResourceObservation::default(),
        },
    }
}

/// The requested case, with its ordinal moved on by one.
const fn shifted_case(request: &NativeExecutionRequest) -> NativeCaseId {
    NativeCaseId::new(
        request.case.group(),
        request.case.opcode(),
        request.case.ordinal().wrapping_add(1),
    )
}

/// A case no census declares.
const fn unknown_case() -> NativeCaseId {
    use target_elements_conformance::fixture::NativeCaseGroup;
    NativeCaseId::new(NativeCaseGroup::Resource, None, u32::MAX)
}

/// Reads one line, or `None` at end of stream.
fn read_line(input: &std::io::Stdin) -> std::io::Result<Option<String>> {
    let mut line = String::new();
    if input.read_line(&mut line)? == 0 {
        return Ok(None);
    }
    Ok(Some(line))
}

/// Writes one NDJSON line.
fn write_json<T: serde::Serialize>(writer: &mut impl Write, value: &T) -> std::io::Result<()> {
    let mut bytes =
        serde_json::to_vec(value).map_err(|error| std::io::Error::other(error.to_string()))?;
    bytes.push(b'\n');
    writer.write_all(&bytes)?;
    writer.flush()
}
