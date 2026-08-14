//! Executor-protocol coverage against a real misbehaving child.
//!
//! Every case here runs `mock-native-executor` as an actual subprocess,
//! selected through a small wrapper script exactly as a caller selects
//! an executor. The harness passes no arguments to an executor, so the
//! wrapper is how a behaviour is chosen — which is also the shape a real
//! executor adapter takes.
//!
//! A mock proves the *protocol*. It proves nothing about any target: its
//! answers are the fixture's own expectations read back, and the gate
//! refuses a mock run for exactly that reason.

#![cfg(unix)]

use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tapscript::{StackItem, TapscriptInstruction, TapscriptProgram};
use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    OpcodeId, ReviewedElementsTapscriptDefinition, TargetContractVersion,
    ValidatedDevelopmentBinding, reviewed_elements_tapscript, validate_development_binding,
};
use target_elements_conformance::error::NativeConformanceError;
use target_elements_conformance::executor::{
    ExecutionTranscript, ExecutorConfiguration, ExecutorTrust, execute,
};
use target_elements_conformance::fixture::{
    ExpectedPrimitiveOutcome, NativeCaseGroup, NativeCaseId, PrimitiveFixture, PrimitiveFixtureSet,
};
use target_elements_conformance::protocol::{NativeVerdict, ProtocolPhase};

/// The string the noisy mock writes on its stderr.
const NOISE: &str = "MOCK_EXECUTOR_STDERR_THAT_MUST_NOT_BE_RELAYED";

fn reviewed_target() -> ReviewedElementsTapscriptDefinition {
    reviewed_elements_tapscript().expect("the reviewed contract validates")
}

fn development_binding(
    target: &ReviewedElementsTapscriptDefinition,
) -> ValidatedDevelopmentBinding {
    let binding = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V1,
        DeploymentEnvironment::Development,
        [0x11; 32],
        [0x22; 32],
        ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
        None,
    );
    validate_development_binding(target.validated(), binding).expect("the binding validates")
}

/// Two fixtures, differing only in ordinal.
fn fixtures() -> PrimitiveFixtureSet {
    let target = reviewed_target();
    let binding = development_binding(&target);
    let program = TapscriptProgram::new(vec![TapscriptInstruction::Opcode(OpcodeId::Add64)])
        .expect("the program is within the work limit");
    let stack = [
        StackItem::signed_le64(&target, 2),
        StackItem::signed_le64(&target, 3),
    ];
    let build = |ordinal: u32| {
        PrimitiveFixture::new(
            &target,
            &binding,
            NativeCaseId::new(NativeCaseGroup::Arithmetic, Some(OpcodeId::Add64), ordinal),
            &program,
            &stack,
            None,
            ExpectedPrimitiveOutcome::accept(Some(vec![
                StackItem::signed_le64(&target, 5).bytes().to_vec(),
            ])),
        )
        .expect("the reviewed domain has a wire spelling")
    };
    PrimitiveFixtureSet::new([build(0), build(1)]).expect("distinct cases")
}

/// A wrapper script selecting one mock behaviour.
fn wrapper(directory: &Path, behavior: &str) -> PathBuf {
    let path = directory.join(format!("executor-{behavior}.sh"));
    let mut file = std::fs::File::create(&path).expect("create wrapper");
    writeln!(file, "#!/bin/sh").expect("write wrapper");
    writeln!(
        file,
        "exec {} --behavior {behavior} \"$@\"",
        env!("CARGO_BIN_EXE_mock-native-executor"),
    )
    .expect("write wrapper");
    let mut permissions = file.metadata().expect("metadata").permissions();
    permissions.set_mode(0o755);
    file.set_permissions(permissions).expect("set mode");
    drop(file);
    path
}

/// Runs the two fixtures through one mock behaviour.
fn run(behavior: &str) -> Result<ExecutionTranscript, NativeConformanceError> {
    run_with_timeout(behavior, Duration::from_secs(30))
}

fn run_with_timeout(
    behavior: &str,
    timeout: Duration,
) -> Result<ExecutionTranscript, NativeConformanceError> {
    let directory = tempfile::tempdir().expect("tempdir");
    let program = wrapper(directory.path(), behavior);
    let configuration = ExecutorConfiguration::new(&program, ExecutorTrust::Mock, timeout);
    execute(&reviewed_target(), &configuration, &fixtures())
}

#[test]
fn a_well_behaved_executor_answers_every_case_once() {
    let transcript = run("echo-expected").expect("the exchange completes");
    assert_eq!(transcript.responses().len(), 2);
    assert_eq!(transcript.trust(), ExecutorTrust::Mock);
    assert_eq!(
        transcript.handshake().implementation_name,
        "mock-native-executor",
    );
    for response in transcript.responses().values() {
        assert_eq!(response.verdict, NativeVerdict::Accepted);
    }
}

#[test]
fn a_handshake_schema_this_harness_does_not_speak_fails_closed() {
    let error = run("wrong-handshake-schema").expect_err("the schema is refused");
    assert!(matches!(
        error,
        NativeConformanceError::UnsupportedProtocolSchema { .. },
    ));
}

#[test]
fn a_malformed_line_fails_closed() {
    let error = run("malformed-json").expect_err("the line is refused");
    assert!(matches!(
        error,
        NativeConformanceError::MalformedResponse {
            phase: ProtocolPhase::Handshake,
        },
    ));
}

#[test]
fn an_unknown_response_field_fails_closed() {
    let error = run("unknown-field").expect_err("the field is refused");
    assert!(matches!(
        error,
        NativeConformanceError::MalformedResponse {
            phase: ProtocolPhase::Response,
        },
    ));
}

#[test]
fn a_response_schema_this_harness_does_not_speak_fails_closed() {
    let error = run("wrong-response-schema").expect_err("the schema is refused");
    assert!(matches!(
        error,
        NativeConformanceError::UnsupportedProtocolSchema { .. },
    ));
}

#[test]
fn a_duplicated_result_fails_closed() {
    let error = run("duplicate-result").expect_err("the duplicate is refused");
    assert!(
        matches!(error, NativeConformanceError::DuplicateCaseResponse(_)),
        "expected a duplicate-response failure, got {error}",
    );
}

#[test]
fn a_result_for_a_case_never_asked_about_fails_closed() {
    let error = run("unexpected-result").expect_err("the stray result is refused");
    assert!(
        matches!(error, NativeConformanceError::UnexpectedCaseResponse(_)),
        "expected an unexpected-response failure, got {error}",
    );
}

#[test]
fn a_reordered_result_fails_closed() {
    let error = run("reordered-result").expect_err("the reordering is refused");
    assert!(
        matches!(error, NativeConformanceError::ResponseOrderViolation { .. }),
        "expected an ordering failure, got {error}",
    );
}

#[test]
fn a_missing_result_fails_closed() {
    let error = run("missing-result").expect_err("the silence is refused");
    assert!(
        matches!(error, NativeConformanceError::MissingCaseResponse(_)),
        "expected a missing-response failure, got {error}",
    );
}

#[test]
fn protocol_data_after_the_last_response_fails_closed() {
    let error = run("trailing-data").expect_err("the trailing data is refused");
    assert!(matches!(
        error,
        NativeConformanceError::TrailingProtocolData,
    ));
}

#[test]
fn a_child_that_never_starts_the_protocol_fails_closed() {
    let error = run("die-early").expect_err("the early exit is refused");
    assert!(matches!(
        error,
        NativeConformanceError::ExecutorHandshakeFailed,
    ));
}

#[test]
fn a_nonzero_exit_fails_closed() {
    let error = run("exit-nonzero").expect_err("the exit status is refused");
    assert!(
        matches!(error, NativeConformanceError::ExecutorExited { .. }),
        "expected an exit-status failure, got {error}",
    );
}

#[test]
fn an_executor_that_cannot_be_started_fails_closed() {
    let directory = tempfile::tempdir().expect("tempdir");
    let missing = directory.path().join("no-such-executor");
    let configuration =
        ExecutorConfiguration::new(&missing, ExecutorTrust::Mock, Duration::from_secs(5));
    let error =
        execute(&reviewed_target(), &configuration, &fixtures()).expect_err("nothing to start");
    assert!(matches!(
        error,
        NativeConformanceError::ExecutorStartupFailed,
    ));
}

#[test]
fn a_hanging_executor_is_a_timeout_and_never_a_rejection() {
    let error =
        run_with_timeout("hang", Duration::from_millis(300)).expect_err("the hang is refused");
    assert!(
        matches!(error, NativeConformanceError::ExecutorTimeout),
        "a timeout must be its own failure, got {error}",
    );
}

#[test]
fn infrastructure_trouble_is_reported_as_itself() {
    let transcript = run("infrastructure-error").expect("the exchange completes");
    for response in transcript.responses().values() {
        assert_eq!(response.verdict, NativeVerdict::InfrastructureError);
    }
}

#[test]
fn raw_child_stderr_never_reaches_the_harness() {
    let transcript = run("noisy-stderr").expect("the exchange completes");
    let rendered = format!("{transcript:?}");
    assert!(
        !rendered.contains(NOISE),
        "the child's stderr must be omitted, not relayed: {rendered}",
    );
}
