//! Executor-protocol message tests.

use std::collections::BTreeSet;

use target_elements::ExecutionDomain;

use crate::protocol::{
    ExecutorCapability, ExecutorEnvironmentObservation, ExecutorHandshake, HandshakeRequest,
    NATIVE_PROTOCOL_SCHEMA, NativeExecutionResponse, NativeResourceObservation, NativeVerdict,
    ObservedFailureClass, ProtocolLimits, ProtocolPhase, ResponseShapeDefect, WireExecutionDomain,
    validate_response_shape,
};

fn handshake() -> ExecutorHandshake {
    ExecutorHandshake {
        protocol_schema: NATIVE_PROTOCOL_SCHEMA,
        adapter_name: "example-adapter".to_owned(),
        adapter_version: "0.0.0".to_owned(),
        framework_revision: None,
        node_name: "example-executor".to_owned(),
        node_version: "0.0.0".to_owned(),
        binary_reported_revision: None,
        intended_executed_tip: None,
        upstream_base: None,
        included_local_topics: BTreeSet::new(),
        supported_domains: BTreeSet::from([WireExecutionDomain::Tapscript]),
        supported_leaf_versions: BTreeSet::from([0xc4]),
        capabilities: BTreeSet::from([ExecutorCapability::FinalStackReporting]),
    }
}

/// One response, in whatever shape a test needs.
fn response(
    verdict: NativeVerdict,
    observed_failure: Option<ObservedFailureClass>,
    final_stack: Option<Vec<Vec<u8>>>,
) -> NativeExecutionResponse {
    NativeExecutionResponse {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: crate::fixture::NativeCaseId::new(
            crate::fixture::NativeCaseGroup::Arithmetic,
            Some(target_elements::OpcodeId::Add64),
            0,
        ),
        verdict,
        final_stack,
        final_altstack: None,
        observed_failure,
        resources: NativeResourceObservation::default(),
    }
}

#[test]
fn an_accepted_response_naming_a_failure_class_is_malformed() {
    // Not a passing case with a stray field: it is a message the
    // protocol does not define, and reading a verdict out of it would
    // mean believing whichever half happens to match.
    let capabilities = BTreeSet::from([ExecutorCapability::FailureClassReporting]);
    assert_eq!(
        validate_response_shape(
            &response(
                NativeVerdict::Accepted,
                Some(ObservedFailureClass::EvaluatedFalse),
                None,
            ),
            &capabilities,
        ),
        Err(ResponseShapeDefect::AcceptedResponseNamesFailure),
    );
    assert!(
        validate_response_shape(
            &response(NativeVerdict::Accepted, None, None),
            &capabilities
        )
        .is_ok()
    );
}

#[test]
fn a_response_may_not_contradict_the_interface_its_executor_advertised() {
    let none = BTreeSet::new();
    let classes = BTreeSet::from([ExecutorCapability::FailureClassReporting]);
    let stacks = BTreeSet::from([
        ExecutorCapability::FailureClassReporting,
        ExecutorCapability::FinalStackReporting,
    ]);

    // A class from an executor that says it distinguishes none.
    assert_eq!(
        validate_response_shape(
            &response(
                NativeVerdict::Rejected,
                Some(ObservedFailureClass::EvaluatedFalse),
                None,
            ),
            &none,
        ),
        Err(ResponseShapeDefect::FailureClassWithoutAdvertisedReporting),
    );
    // No class from an executor that says it distinguishes them.
    assert_eq!(
        validate_response_shape(&response(NativeVerdict::Rejected, None, None), &classes),
        Err(ResponseShapeDefect::RejectedResponseOmitsAdvertisedFailure),
    );
    // A stack from an executor that observes none.
    assert_eq!(
        validate_response_shape(
            &response(
                NativeVerdict::Rejected,
                Some(ObservedFailureClass::EvaluatedFalse),
                Some(Vec::new()),
            ),
            &classes,
        ),
        Err(ResponseShapeDefect::StackWithoutAdvertisedReporting),
    );
    // No stack from an executor that says it observes one.
    assert_eq!(
        validate_response_shape(
            &response(
                NativeVerdict::Rejected,
                Some(ObservedFailureClass::EvaluatedFalse),
                None,
            ),
            &stacks,
        ),
        Err(ResponseShapeDefect::AdvertisedStackOmitted),
    );
}

#[test]
fn an_infrastructure_response_carries_no_target_observation() {
    let capabilities = BTreeSet::from([
        ExecutorCapability::FailureClassReporting,
        ExecutorCapability::FinalStackReporting,
    ]);
    // A run that did not happen observed nothing, and an executor that
    // advertises stack reporting is not thereby required to invent one
    // for a case it could not run.
    assert!(
        validate_response_shape(
            &response(NativeVerdict::InfrastructureError, None, None),
            &capabilities,
        )
        .is_ok()
    );
    assert_eq!(
        validate_response_shape(
            &response(NativeVerdict::InfrastructureError, None, Some(Vec::new())),
            &capabilities,
        ),
        Err(ResponseShapeDefect::InfrastructureResponseCarriesObservation),
    );
}

#[test]
fn an_interpreter_figure_needs_an_executor_that_observes_one() {
    let capabilities = BTreeSet::from([ExecutorCapability::FailureClassReporting]);
    let mut answer = response(
        NativeVerdict::Rejected,
        Some(ObservedFailureClass::EvaluatedFalse),
        None,
    );
    assert!(validate_response_shape(&answer, &capabilities).is_ok());
    answer.resources.peak_stack_items = Some(3);
    assert_eq!(
        validate_response_shape(&answer, &capabilities),
        Err(ResponseShapeDefect::ResourceWithoutAdvertisedObservation),
    );
}

#[test]
fn a_record_bound_is_named_for_every_phase() {
    let limits = ProtocolLimits::DEFAULT;
    assert_eq!(
        limits.for_phase(ProtocolPhase::Handshake),
        limits.maximum_handshake_bytes,
    );
    assert_eq!(
        limits.for_phase(ProtocolPhase::Environment),
        limits.maximum_environment_bytes,
    );
    assert_eq!(
        limits.for_phase(ProtocolPhase::Response),
        limits.maximum_response_bytes,
    );
    assert_eq!(
        limits.for_phase(ProtocolPhase::Shutdown),
        limits.maximum_trailing_record_bytes,
    );
    assert_eq!(limits, ProtocolLimits::default());
}

#[test]
fn an_environment_observation_round_trips_and_refuses_unknown_fields() {
    let observation = crate::tests::support::observed_environment();
    let json = serde_json::to_string(&observation).expect("an observation serializes");
    let parsed: ExecutorEnvironmentObservation =
        serde_json::from_str(&json).expect("an observation parses");
    assert_eq!(parsed, observation);

    let mut value = serde_json::to_value(&observation).expect("an observation serializes");
    value
        .as_object_mut()
        .expect("an observation is an object")
        .insert("trust_me".to_owned(), serde_json::Value::Bool(true));
    serde_json::from_value::<ExecutorEnvironmentObservation>(value)
        .expect_err("an unknown observation field is refused");
}

#[test]
fn every_reviewed_execution_domain_has_a_wire_form() {
    for domain in ExecutionDomain::ALL {
        assert!(
            WireExecutionDomain::of(*domain).is_some(),
            "every reviewed domain must have a wire form",
        );
    }
}

#[test]
fn the_handshake_request_states_this_harnesss_schema() {
    assert_eq!(HandshakeRequest::default().schema, NATIVE_PROTOCOL_SCHEMA);
}

#[test]
fn this_harness_speaks_schema_two_and_not_schema_one() {
    // Stated as a value rather than left implicit. Schema 2 adds the
    // environment observation, the separated provenance roles, and the
    // bounded-record contract; a schema-1 executor establishes none of
    // them, so the two are refused for each other rather than reconciled
    // by reading whichever fields happen to overlap.
    assert_eq!(NATIVE_PROTOCOL_SCHEMA, 2);
    assert_ne!(NATIVE_PROTOCOL_SCHEMA, 1);
}

#[test]
fn a_handshake_round_trips() {
    let json = serde_json::to_string(&handshake()).expect("a handshake serializes");
    let parsed: ExecutorHandshake = serde_json::from_str(&json).expect("a handshake parses");
    assert_eq!(parsed, handshake());
}

#[test]
fn an_unknown_handshake_field_fails_closed() {
    let mut value = serde_json::to_value(handshake()).expect("a handshake serializes");
    value
        .as_object_mut()
        .expect("a handshake is an object")
        .insert("trust_me".to_owned(), serde_json::Value::Bool(true));
    serde_json::from_value::<ExecutorHandshake>(value)
        .expect_err("an unknown handshake field is refused");
}

#[test]
fn a_response_round_trips_and_refuses_unknown_fields() {
    let response = NativeExecutionResponse {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: crate::fixture::NativeCaseId::new(
            crate::fixture::NativeCaseGroup::Arithmetic,
            Some(target_elements::OpcodeId::Add64),
            0,
        ),
        verdict: NativeVerdict::Rejected,
        final_stack: None,
        final_altstack: None,
        observed_failure: Some(ObservedFailureClass::ArithmeticOverflow),
        resources: NativeResourceObservation::default(),
    };

    let json = serde_json::to_string(&response).expect("a response serializes");
    let parsed: NativeExecutionResponse = serde_json::from_str(&json).expect("a response parses");
    assert_eq!(parsed, response);

    let mut value = serde_json::to_value(&response).expect("a response serializes");
    value
        .as_object_mut()
        .expect("a response is an object")
        .insert("note".to_owned(), serde_json::Value::from("hello"));
    serde_json::from_value::<NativeExecutionResponse>(value)
        .expect_err("an unknown response field is refused");
}

#[test]
fn the_three_verdicts_stay_distinct() {
    let spellings: BTreeSet<String> = [
        NativeVerdict::Accepted,
        NativeVerdict::Rejected,
        NativeVerdict::InfrastructureError,
    ]
    .into_iter()
    .map(|verdict| serde_json::to_string(&verdict).expect("a verdict serializes"))
    .collect();
    assert_eq!(spellings.len(), 3, "no two verdicts share a spelling");
    assert!(spellings.contains("\"infrastructure_error\""));
}
