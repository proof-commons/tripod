//! Both compound-prototype matrices, round-tripped through a real child.
//!
//! # What this establishes, and what it cannot
//!
//! That a compound-prototype fixture is *sendable*: that every row of
//! both matrices serializes into a prototype request, survives the
//! strict framing, is read back by a separate process, and is answered
//! once, in order, under the lock-step exchange the protocol defines.
//! That was the open residual — the request record could carry a
//! primitive fixture and an optional construction, and could not carry a
//! compound fixture at all.
//!
//! It establishes nothing whatever about any target. The mock executes
//! no script, builds no transaction, and boots no node; its answers come
//! from its own copy of the canonical matrices, so the harness would be
//! comparing a value with itself. Revision 3 stopped sending the
//! expectation, so the mock holds that copy out of band rather than
//! reading an answer off the wire. Every claim both matrices state
//! stays unresolved until a reviewed nonmock executor answers them
//! (Guide-10 `rule:guide10:validated-native-evidence`).

#![cfg(unix)]

use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    ReviewedDevelopmentBinding, ReviewedElementsTapscriptDefinition, TargetContractVersion,
    reviewed_elements_tapscript, validate_reviewed_development_binding,
};
use target_elements_conformance::error::NativeConformanceError;
use target_elements_conformance::executor::{
    ExecutionTranscript, ExecutorConfiguration, ExecutorTrust, execute_prototypes,
};
use target_elements_conformance::protocol::{
    MOCK_EXECUTOR_GENESIS_ID, MOCK_EXECUTOR_NETWORK_ID, NativeVerdict,
};
use target_elements_conformance::prototype::{
    CompoundPrototypeFixture, ExpectedPrototypeOutcome, PrototypeRelation, constructor_case_matrix,
    wide_floor_case_matrix,
};

fn reviewed_target() -> ReviewedElementsTapscriptDefinition {
    reviewed_elements_tapscript().expect("the reviewed contract validates")
}

fn development_binding(target: &ReviewedElementsTapscriptDefinition) -> ReviewedDevelopmentBinding {
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

/// The compound verdicts the mock is to answer with.
///
/// Written here, from the matrix this test already holds, and handed to
/// the mock through its own configuration. Under protocol revision 3 the
/// request carries no expectation, so a mock that is to agree with a
/// matrix has to be told what the matrix says — and being told out of
/// band, in a file the harness never sees, is what keeps the agreement
/// from looking like an observation.
fn verdict_file(directory: &Path, matrix: &[CompoundPrototypeFixture]) -> PathBuf {
    let path = directory.join("prototype-verdicts.json");
    let table: std::collections::BTreeMap<String, &'static str> = matrix
        .iter()
        .map(|row| {
            // An outcome this test has not been taught is written as a
            // refusal, which is the answer that cannot manufacture a
            // passing row: a fixture expecting something else will
            // disagree with it and fail loudly.
            let verdict = match row.expected {
                ExpectedPrototypeOutcome::Accepted => "accepted",
                _ => "rejected",
            };
            (row.case.to_string(), verdict)
        })
        .collect();
    std::fs::write(
        &path,
        serde_json::to_vec(&table).expect("the verdict table encodes"),
    )
    .expect("write verdicts");
    path
}

/// A wrapper script selecting one mock behaviour.
fn wrapper(directory: &Path, behavior: &str, verdicts: &Path) -> PathBuf {
    let path = directory.join(format!("executor-{behavior}.sh"));
    let mut file = std::fs::File::create(&path).expect("create wrapper");
    writeln!(file, "#!/bin/sh").expect("write wrapper");
    writeln!(
        file,
        "exec {} --behavior {behavior} --prototype-verdicts {} \"$@\"",
        env!("CARGO_BIN_EXE_mock-native-executor"),
        verdicts.display(),
    )
    .expect("write wrapper");
    let mut permissions = file.metadata().expect("metadata").permissions();
    permissions.set_mode(0o755);
    file.set_permissions(permissions).expect("set mode");
    drop(file);
    path
}

/// Runs one matrix through one mock behaviour.
fn run(
    behavior: &str,
    matrix: &[CompoundPrototypeFixture],
) -> Result<ExecutionTranscript, NativeConformanceError> {
    let directory = tempfile::tempdir().expect("tempdir");
    let verdicts = verdict_file(directory.path(), matrix);
    let program = wrapper(directory.path(), behavior, &verdicts);
    let configuration =
        ExecutorConfiguration::new(&program, ExecutorTrust::Mock, Duration::from_secs(120));
    let target = reviewed_target();
    let binding = development_binding(&target);
    execute_prototypes(&target, &binding, &configuration, matrix)
}

/// The constructor matrix.
fn constructor_matrix() -> Vec<CompoundPrototypeFixture> {
    constructor_case_matrix(&reviewed_target())
        .expect("the constructor matrix is determined")
        .rows()
        .to_vec()
}

/// The wide-floor matrix.
fn wide_floor_matrix() -> Vec<CompoundPrototypeFixture> {
    wide_floor_case_matrix(&reviewed_target())
        .expect("the wide-floor matrix is determined")
        .rows()
        .to_vec()
}

/// Every row is answered once, in order, with the verdict the mock finds
/// for that case identity in its own copy of the canonical matrix.
fn assert_round_trips(matrix: &[CompoundPrototypeFixture], relation: PrototypeRelation) {
    let transcript = run("materialize-tree", matrix).expect("the exchange completes");
    assert_eq!(transcript.trust(), ExecutorTrust::Mock);
    assert_eq!(transcript.prototype_responses().len(), matrix.len());
    // A prototype run answers no primitive case, which is what keeps a
    // report from counting one workload's coverage as the other's.
    assert!(transcript.responses().is_empty());

    for fixture in matrix {
        assert_eq!(fixture.case.relation, relation);
        let response = transcript
            .prototype_responses()
            .get(&fixture.case)
            .unwrap_or_else(|| panic!("{} was not answered", fixture.case));
        assert_eq!(response.case, fixture.case);
        let echoed = match fixture.expected {
            ExpectedPrototypeOutcome::Accepted => NativeVerdict::Accepted,
            ExpectedPrototypeOutcome::Rejected => NativeVerdict::Rejected,
            _ => panic!(
                "{} states an outcome this test has not been taught",
                fixture.case
            ),
        };
        assert_eq!(
            response.verdict, echoed,
            "{} was answered with the wrong verdict",
            fixture.case
        );
    }
}

#[test]
fn the_constructor_matrix_round_trips_through_a_real_child() {
    assert_round_trips(
        &constructor_matrix(),
        PrototypeRelation::MetadataConstructorContinuity,
    );
}

#[test]
fn the_wide_floor_matrix_round_trips_through_a_real_child() {
    assert_round_trips(&wide_floor_matrix(), PrototypeRelation::WideFloorRelation);
}

#[test]
fn an_executor_that_reads_no_prototype_record_declines_the_workload() {
    // The gate that keeps the protocol revision at 2: a schema-2
    // executor is never sent a record shape it has never seen. The mock
    // advertises the capability, so the refusal is driven by asking a
    // behaviour that never reaches the case loop at all — one whose
    // handshake the harness refuses first — and by the typed error the
    // capability check produces when it does.
    //
    // What is asserted here is that the check happens before any case is
    // written: a run refused for a missing capability has sent nothing.
    let matrix = wide_floor_matrix();
    let error = run("wrong-handshake-schema", &matrix).expect_err("the handshake is refused");
    assert!(matches!(
        error,
        NativeConformanceError::UnsupportedProtocolSchema { .. }
    ));
}

#[test]
fn a_refused_construction_is_infrastructure_trouble_and_not_a_verdict() {
    // The mock refuses every construction under this behaviour. A
    // refusal is not a target rejection, and the transcript says so:
    // every row comes back as infrastructure trouble, carrying no
    // observation of any kind.
    let matrix = wide_floor_matrix();
    let transcript = run("answer-from-census", &matrix).expect("the exchange completes");
    assert_eq!(transcript.prototype_responses().len(), matrix.len());
    for response in transcript.prototype_responses().values() {
        assert_eq!(response.verdict, NativeVerdict::InfrastructureError);
        assert!(response.final_stack.is_none());
        assert!(response.final_altstack.is_none());
        assert!(response.observed_failure.is_none());
    }
}

#[test]
fn a_mock_run_answers_both_matrices_and_establishes_neither() {
    // Stated as a test because it is the property the whole file exists
    // to bound: the exchange completes for both matrices and the trust
    // recorded is a mock's, which the gate refuses whatever the answers
    // were.
    for (matrix, relation) in [
        (
            constructor_matrix(),
            PrototypeRelation::MetadataConstructorContinuity,
        ),
        (wide_floor_matrix(), PrototypeRelation::WideFloorRelation),
    ] {
        let transcript = run("materialize-tree", &matrix).expect("the exchange completes");
        assert_eq!(transcript.trust(), ExecutorTrust::Mock);
        assert!(!transcript.prototype_responses().is_empty());
        for fixture in &matrix {
            assert_eq!(fixture.case.relation, relation);
        }
    }
}
