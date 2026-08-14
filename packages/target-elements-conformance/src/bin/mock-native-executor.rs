//! `mock-native-executor`: a test-only child that speaks the executor
//! protocol badly on purpose.
//!
//! This is NOT an Elements executor. It runs no interpreter, executes no
//! script, and observes nothing about any target. Its answers are echoes
//! of the fixture's own stated expectation, which is precisely why a run
//! against it can never be target-native evidence: the harness would be
//! comparing a fixture with itself.
//!
//! It exists so the protocol's failure paths — a malformed line, a wrong
//! schema, a duplicated, missing, reordered, or unexpected result, a
//! child that dies, a child that hangs, a child that writes noise on
//! stderr — can be driven to failure by a real subprocess. A run using
//! it must be declared a mock run, and the gate refuses such a run
//! whatever the report says (Guide-9 §1.6, §11.8).
//!
//! Under ADR-010 this command's stdout is protocol data, in the NDJSON
//! form the harness's protocol documents.

use std::collections::BTreeSet;
use std::io::Write;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use cli_common::{
    CommandExit, emit_control_plane_record, install_json_panic_hook, parse_args_from,
};
use target_elements_conformance::protocol::{
    ExecutorCapability, ExecutorHandshake, NATIVE_PROTOCOL_SCHEMA, NativeExecutionRequest,
    NativeExecutionResponse, NativeResourceObservation, NativeVerdict, WireExecutionDomain,
};

const COMMAND_NAME: &str = "mock-native-executor";

/// How the mock misbehaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum Behavior {
    /// Answer every case with the fixture's own stated expectation.
    EchoExpected,
    /// Answer the handshake with a schema the harness does not speak.
    WrongHandshakeSchema,
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

    run(args.behavior).map_or_else(|_| CommandExit::Failure.exit_code(), CommandExit::exit_code)
}

/// Speaks the protocol in the selected way.
fn run(behavior: Behavior) -> std::io::Result<CommandExit> {
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

    if behavior == Behavior::MalformedJson {
        writeln!(stdout, "{{this is not json")?;
        stdout.flush()?;
        return Ok(CommandExit::Success);
    }

    let schema = if behavior == Behavior::WrongHandshakeSchema {
        NATIVE_PROTOCOL_SCHEMA + 1
    } else {
        NATIVE_PROTOCOL_SCHEMA
    };
    write_json(
        &mut stdout,
        &ExecutorHandshake {
            protocol_schema: schema,
            implementation_name: "mock-native-executor".to_owned(),
            implementation_version: "0.0.0".to_owned(),
            upstream_revision: None,
            supported_domains: BTreeSet::from([WireExecutionDomain::Tapscript]),
            supported_leaf_versions: BTreeSet::from([0xc4]),
            capabilities: BTreeSet::from([
                ExecutorCapability::FinalStackReporting,
                ExecutorCapability::FinalAltstackReporting,
                ExecutorCapability::FailureClassReporting,
            ]),
        },
    )?;

    if behavior == Behavior::MissingResult {
        return Ok(CommandExit::Success);
    }

    let mut first: Option<NativeExecutionResponse> = None;
    while let Some(line) = read_line(&input)? {
        if line.trim().is_empty() {
            continue;
        }
        let Ok(request) = serde_json::from_str::<NativeExecutionRequest>(&line) else {
            return Ok(CommandExit::Failure);
        };
        let response = echo(&request, behavior);

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

    Ok(if behavior == Behavior::ExitNonzero {
        CommandExit::Failure
    } else {
        CommandExit::Success
    })
}

/// The fixture's own stated expectation, echoed back.
fn echo(request: &NativeExecutionRequest, behavior: Behavior) -> NativeExecutionResponse {
    use target_elements_conformance::fixture::ExpectedPrimitiveOutcome;

    let schema = if behavior == Behavior::WrongResponseSchema {
        NATIVE_PROTOCOL_SCHEMA + 7
    } else {
        NATIVE_PROTOCOL_SCHEMA
    };

    if behavior == Behavior::InfrastructureError {
        return NativeExecutionResponse {
            schema,
            case: request.case,
            verdict: NativeVerdict::InfrastructureError,
            final_stack: None,
            final_altstack: None,
            observed_failure: None,
            resources: NativeResourceObservation::default(),
        };
    }

    // The fixture's own figures, echoed like everything else: the mock
    // measures nothing, and a run against it is not evidence.
    let resources = NativeResourceObservation {
        script_bytes: request.fixture.script().len() as u64,
        initial_stack_items: request.fixture.initial_stack().len() as u64,
        ..NativeResourceObservation::default()
    };

    match request.fixture.expected() {
        ExpectedPrimitiveOutcome::Accept {
            static_final_stack,
            static_final_altstack,
        } => NativeExecutionResponse {
            schema,
            case: request.case,
            verdict: NativeVerdict::Accepted,
            final_stack: static_final_stack.clone(),
            final_altstack: static_final_altstack.clone(),
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
            final_stack: static_final_stack.clone(),
            final_altstack: static_final_altstack.clone(),
            // The first class the fixture admits, which is the only one
            // a mock could pick without measuring anything.
            observed_failure: classes.iter().next().copied(),
            resources,
        },
    }
}

/// The requested case, with its ordinal moved on by one.
const fn shifted_case(
    request: &NativeExecutionRequest,
) -> target_elements_conformance::fixture::NativeCaseId {
    use target_elements_conformance::fixture::NativeCaseId;
    NativeCaseId::new(
        request.case.group(),
        request.case.opcode(),
        request.case.ordinal().wrapping_add(1),
    )
}

/// A case no census declares.
const fn unknown_case() -> target_elements_conformance::fixture::NativeCaseId {
    use target_elements_conformance::fixture::{NativeCaseGroup, NativeCaseId};
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
