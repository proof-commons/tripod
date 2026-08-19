//! Executor-protocol coverage against a real misbehaving child.
//!
//! Every case here runs `mock-native-executor` as an actual subprocess,
//! selected through a small wrapper script exactly as a caller selects
//! an executor. The harness passes no arguments to an executor, so the
//! wrapper is how a behaviour is chosen — which is also the shape a real
//! executor adapter takes.
//!
//! A mock proves the *protocol*. It proves nothing about any target: its
//! answers come from a table it built for itself out of the very censuses
//! the harness compares them against, and the gate refuses a mock run for
//! exactly that reason.

#![cfg(unix)]

use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tapscript::{StackItem, TapscriptInstruction, TapscriptProgram};
use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    OpcodeId, ReviewedDevelopmentBinding, ReviewedElementsTapscriptDefinition,
    TargetContractVersion, reviewed_elements_tapscript, validate_reviewed_development_binding,
};
use target_elements_conformance::error::NativeConformanceError;
use target_elements_conformance::executor::{
    ExecutionTranscript, ExecutorConfiguration, ExecutorTrust, execute,
};
use target_elements_conformance::fixture::{
    ExpectedPrimitiveOutcome, NativeCaseGroup, NativeCaseId, PrimitiveFixture, PrimitiveFixtureSet,
};
use target_elements_conformance::protocol::{
    MOCK_EXECUTOR_GENESIS_ID, MOCK_EXECUTOR_NETWORK_ID, NATIVE_PROTOCOL_SCHEMA, NativeVerdict,
    ProtocolLimits, ProtocolPhase, ResponseShapeDefect,
};

/// The string the noisy mock writes on its stderr.
const NOISE: &str = "MOCK_EXECUTOR_STDERR_THAT_MUST_NOT_BE_RELAYED";

fn reviewed_target() -> ReviewedElementsTapscriptDefinition {
    reviewed_elements_tapscript().expect("the reviewed contract validates")
}

fn development_binding(target: &ReviewedElementsTapscriptDefinition) -> ReviewedDevelopmentBinding {
    // The identifiers are the ones the mock states it observed. A run
    // whose executor observed another chain is refused before any case
    // executes, which is what several cases below drive.
    let binding = DevelopmentDeploymentBinding::new(
        TargetContractVersion::V2,
        DeploymentEnvironment::Development,
        MOCK_EXECUTOR_NETWORK_ID,
        MOCK_EXECUTOR_GENESIS_ID,
        ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
        None,
    );
    validate_reviewed_development_binding(target, binding).expect("the binding validates")
}

/// The ordinal the first ad hoc case takes.
///
/// Deliberately far outside the canonical census. These fixtures are the
/// caller's own — two additions of the same two operands — and their case
/// identities must not collide with a canonical case's, because the mock
/// answers by looking a case identity up in its own census-derived table.
/// A collision would have the mock answer these fixtures with whatever the
/// canonical case of that identity expects, which is a different question
/// from the one these tests ask.
///
/// That is not a wrinkle to work around: it is protocol revision 3 doing
/// its job. Under revision 2 the mock read the expectation out of the
/// request and could not help but agree with any fixture it was handed.
const AD_HOC_ORDINAL: u32 = 900_000;

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
            NativeCaseId::new(
                NativeCaseGroup::Arithmetic,
                Some(OpcodeId::Add64),
                AD_HOC_ORDINAL + ordinal,
            ),
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
    run_with(behavior, timeout, ProtocolLimits::DEFAULT)
}

fn run_with(
    behavior: &str,
    timeout: Duration,
    limits: ProtocolLimits,
) -> Result<ExecutionTranscript, NativeConformanceError> {
    let directory = tempfile::tempdir().expect("tempdir");
    let program = wrapper(directory.path(), behavior);
    let configuration =
        ExecutorConfiguration::new(&program, ExecutorTrust::Mock, timeout).with_limits(limits);
    let target = reviewed_target();
    let binding = development_binding(&target);
    execute(&target, &binding, &configuration, &fixtures())
}

#[test]
fn a_well_behaved_executor_answers_every_case_once() {
    let transcript = run("answer-from-census").expect("the exchange completes");
    assert_eq!(transcript.responses().len(), 2);
    assert_eq!(transcript.trust(), ExecutorTrust::Mock);
    assert_eq!(transcript.handshake().node_name, "mock-native-executor");
    // A mock builds nothing, so it establishes no workspace provenance
    // and says so rather than substituting a plausible revision.
    assert_eq!(transcript.handshake().binary_reported_revision, None);
    assert_eq!(transcript.handshake().intended_executed_tip, None);
    assert_eq!(
        transcript.environment().genesis_id,
        MOCK_EXECUTOR_GENESIS_ID,
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
fn an_executor_of_the_previous_revision_fails_loudly() {
    // The migration case, driven through a real subprocess rather than
    // argued about. A revision-2 adapter answers a question revision 3 no
    // longer asks — its requests carried the expectation — so it is
    // refused at the handshake by name, before any case is sent and long
    // before any record of its could be read as a revision-3 one.
    let error = run("previous-revision-handshake").expect_err("the old revision is refused");
    assert!(
        matches!(
            error,
            NativeConformanceError::UnsupportedProtocolSchema { offered }
                if offered == NATIVE_PROTOCOL_SCHEMA - 1,
        ),
        "expected the previous revision to be named, got {error}",
    );
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
    assert!(
        matches!(error, NativeConformanceError::ExecutorHandshakeFailed),
        "expected a handshake failure, got {error}",
    );
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
    let target = reviewed_target();
    let binding = development_binding(&target);
    let error =
        execute(&target, &binding, &configuration, &fixtures()).expect_err("nothing to start");
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
fn a_blank_record_is_refused_rather_than_skipped() {
    // The framing defines one nonempty JSON object per record. A framing
    // that skipped empty ones could not tell an executor that said
    // nothing from an executor that finished.
    let error = run("blank-record").expect_err("the blank record is refused");
    assert!(
        matches!(
            error,
            NativeConformanceError::BlankProtocolRecord {
                phase: ProtocolPhase::Handshake,
            },
        ),
        "expected a blank-record failure, got {error}",
    );
}

#[test]
fn a_blank_record_after_the_last_response_is_trailing_data() {
    let error = run("trailing-blank-record").expect_err("the trailing record is refused");
    assert!(
        matches!(error, NativeConformanceError::TrailingProtocolData),
        "expected trailing protocol data, got {error}",
    );
}

#[test]
fn a_record_past_the_bound_is_refused() {
    let error = run("oversized-handshake").expect_err("the oversized record is refused");
    assert!(
        matches!(
            error,
            NativeConformanceError::ProtocolRecordTooLarge {
                phase: ProtocolPhase::Handshake,
                ..
            },
        ),
        "expected an oversized-record failure, got {error}",
    );
}

#[test]
fn an_unterminated_oversized_record_is_refused_without_unbounded_allocation() {
    // The child holds the record open and never emits a newline. The
    // harness must refuse at its own bound rather than allocate until
    // the host intervenes — which is why the bound here is small and the
    // timeout is long enough that a timeout would not be the answer.
    let limits = ProtocolLimits {
        maximum_handshake_bytes: 1024,
        ..ProtocolLimits::DEFAULT
    };
    let error = run_with("unterminated-handshake", Duration::from_secs(20), limits)
        .expect_err("the unterminated record is refused");
    assert!(
        matches!(
            error,
            NativeConformanceError::ProtocolRecordTooLarge {
                phase: ProtocolPhase::Handshake,
                maximum: 1024,
            },
        ),
        "expected an oversized-record failure at the stated bound, got {error}",
    );
}

#[test]
fn a_record_at_exactly_the_bound_is_accepted() {
    // The bound is on the record, and a record of exactly the maximum is
    // a record within it. An off-by-one here would refuse honest
    // executors at the boundary the protocol documents.
    let handshake_bytes = {
        let transcript = run("answer-from-census").expect("the exchange completes");
        serde_json::to_vec(transcript.handshake())
            .expect("a handshake serializes")
            .len()
    };
    let limits = ProtocolLimits {
        maximum_handshake_bytes: handshake_bytes,
        ..ProtocolLimits::DEFAULT
    };
    run_with("answer-from-census", Duration::from_secs(30), limits)
        .expect("a record of exactly the maximum is within the bound");

    let limits = ProtocolLimits {
        maximum_handshake_bytes: handshake_bytes - 1,
        ..ProtocolLimits::DEFAULT
    };
    let error = run_with("answer-from-census", Duration::from_secs(30), limits)
        .expect_err("one byte past the bound is refused");
    assert!(
        matches!(
            error,
            NativeConformanceError::ProtocolRecordTooLarge {
                phase: ProtocolPhase::Handshake,
                ..
            },
        ),
        "expected an oversized-record failure, got {error}",
    );
}

#[test]
fn an_executor_that_states_no_environment_fails_closed() {
    let error = run("missing-environment").expect_err("the silence is refused");
    assert!(
        matches!(error, NativeConformanceError::MissingEnvironmentObservation),
        "expected a missing-environment failure, got {error}",
    );
}

#[test]
fn an_executor_that_ran_another_chain_fails_before_any_case() {
    let error = run("wrong-genesis").expect_err("the other chain is refused");
    assert!(
        matches!(error, NativeConformanceError::GenesisObservationMismatch),
        "expected a genesis mismatch, got {error}",
    );

    let error = run("wrong-network").expect_err("the other network is refused");
    assert!(
        matches!(error, NativeConformanceError::EnvironmentBindingMismatch),
        "expected an environment mismatch, got {error}",
    );

    let error = run("inactive-leaf-version").expect_err("the inactive leaf is refused");
    assert!(
        matches!(error, NativeConformanceError::ActivationObservationMismatch),
        "expected an activation mismatch, got {error}",
    );
}

#[test]
fn an_accepted_response_naming_a_failure_class_is_not_a_passing_case() {
    let error = run("accepted-with-failure-class").expect_err("the contradiction is refused");
    assert!(
        matches!(
            error,
            NativeConformanceError::MalformedResponseShape {
                defect: ResponseShapeDefect::AcceptedResponseNamesFailure,
                ..
            },
        ),
        "expected a response-shape failure, got {error}",
    );
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
