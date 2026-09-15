//! Executor-protocol message tests.

use std::collections::BTreeSet;

use target_elements::ExecutionDomain;

use crate::protocol::{
    ConservationOpening, ExecutorCapability, ExecutorEnvironmentObservation, ExecutorHandshake,
    HandshakeRequest, NATIVE_PROTOCOL_SCHEMA, NativeConservationResponse, NativeExecutionResponse,
    NativeOperationResponse, NativePrototypeResponse, NativeResourceObservation, NativeVerdict,
    ObservedFailureClass, ObservedOutcomeLayer, OperationCaseId, OperationStepKind, ProtocolLimits,
    ProtocolPhase, ResponseShapeDefect, WireExecutionDomain, validate_response_shape,
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
        confidential_funding: None,
    }
}

#[test]
fn the_native_protocol_is_revision_eight() {
    assert_eq!(NATIVE_PROTOCOL_SCHEMA, 8);
}

/// One response, in whatever shape a test needs.
fn response(
    verdict: NativeVerdict,
    observed_failure: Option<ObservedFailureClass>,
    final_stack: Option<Vec<Vec<u8>>>,
) -> NativeExecutionResponse {
    let resources = if matches!(verdict, NativeVerdict::InfrastructureError) {
        NativeResourceObservation::default()
    } else {
        NativeResourceObservation {
            script_bytes: Some(33),
            initial_stack_items: Some(1),
            ..NativeResourceObservation::default()
        }
    };
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
        resources,
    }
}

/// One conservation response, with only observations its layer permits.
fn conservation_response(layer: ObservedOutcomeLayer) -> NativeConservationResponse {
    NativeConservationResponse {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: crate::conservation::ConservationRowId {
            ordinal: 1,
            name: "revision-seven-shape".to_owned(),
        },
        observed_layer: layer,
        observed_detail: None,
        transaction_bytes: layer.is_target_verdict().then(|| vec![0x02]),
        observed_value_commitments: Vec::new(),
        observed_asset_commitments: Vec::new(),
        observed_openings: Vec::new(),
    }
}

/// One operation response, carrying no operation-specific observation.
fn operation_response(layer: ObservedOutcomeLayer) -> NativeOperationResponse {
    NativeOperationResponse {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: OperationCaseId {
            operation: OperationStepKind::Submit,
            step: "revision-seven-shape".to_owned(),
        },
        observed_layer: layer,
        observed_detail: None,
        issued_asset: None,
        funded_outputs: Vec::new(),
        confidential_funded_outputs: Vec::new(),
        mined_readback: None,
        accepted_txid: None,
        sponsor_witness: Vec::new(),
        signature_bound_to: None,
        script_path_witness: Vec::new(),
        signer_public_key: None,
        signed_profile: None,
        signing_genesis: None,
        resources: NativeResourceObservation::default(),
    }
}

#[test]
fn revision_seven_conservation_shapes_are_enforced_per_layer() {
    assert_eq!(
        conservation_response(ObservedOutcomeLayer::Accepted).validate_shape(),
        Ok(()),
        "accepted explicit transactions need no commitments or openings",
    );
    let mut accepted = conservation_response(ObservedOutcomeLayer::Accepted);
    accepted.transaction_bytes = None;
    assert_eq!(
        accepted.validate_shape(),
        Err(ResponseShapeDefect::AcceptedConservationOmitsTransaction),
    );

    for layer in [
        ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
        ObservedOutcomeLayer::ScriptPathRejection,
        ObservedOutcomeLayer::KeyPathRejection,
        ObservedOutcomeLayer::RelayPolicyRejection,
    ] {
        let mut rejected = conservation_response(layer);
        rejected.observed_openings = vec![ConservationOpening {
            vout: 0,
            amount_satoshis: 1,
            asset: "aa".repeat(32),
            amount_blinder: "bb".repeat(32),
            asset_blinder: "cc".repeat(32),
        }];
        assert_eq!(
            rejected.validate_shape(),
            Err(ResponseShapeDefect::RefusedConservationCarriesOpenings),
            "{layer:?} carried openings",
        );
    }

    let mut rejected = conservation_response(ObservedOutcomeLayer::ConsensusRejectionBeforeScript);
    rejected.observed_value_commitments = vec![vec![0x08; 33]];
    rejected.observed_asset_commitments = vec![vec![0x0a; 33]];
    assert_eq!(rejected.validate_shape(), Ok(()));
}

#[test]
fn a_non_verdict_conservation_response_carries_no_observation() {
    for layer in [
        ObservedOutcomeLayer::FixtureConstructionFailure,
        ObservedOutcomeLayer::ExecutorInfrastructureFailure,
    ] {
        let empty = conservation_response(layer);
        assert_eq!(empty.validate_shape(), Ok(()));

        let mut bytes = empty.clone();
        bytes.transaction_bytes = Some(vec![0x02]);
        assert_eq!(
            bytes.validate_shape(),
            Err(ResponseShapeDefect::InfrastructureResponseCarriesObservation),
        );

        let mut commitments = empty;
        commitments.observed_value_commitments = vec![vec![0x08; 33]];
        assert_eq!(
            commitments.validate_shape(),
            Err(ResponseShapeDefect::InfrastructureResponseCarriesObservation),
        );
    }
}

#[test]
fn resource_fixture_members_are_required_but_nullable() {
    let wire = serde_json::to_value(NativeResourceObservation::default())
        .expect("a resource observation serializes");
    assert_eq!(wire["script_bytes"], serde_json::Value::Null);
    assert_eq!(wire["initial_stack_items"], serde_json::Value::Null);
    let decoded: NativeResourceObservation =
        serde_json::from_value(wire.clone()).expect("explicit null resource members decode");
    assert_eq!(decoded.script_bytes, None);
    assert_eq!(decoded.initial_stack_items, None);

    for member in ["script_bytes", "initial_stack_items"] {
        let mut omitted = wire.clone();
        omitted
            .as_object_mut()
            .expect("the resource observation is an object")
            .remove(member);
        serde_json::from_value::<NativeResourceObservation>(omitted)
            .expect_err("a required-nullable resource member was omitted");
    }
}

#[test]
fn primitive_and_prototype_verdicts_require_fixture_resources() {
    let mut primitive = response(NativeVerdict::Accepted, None, None);
    primitive.resources.script_bytes = None;
    assert_eq!(
        validate_response_shape(&primitive, &BTreeSet::new()),
        Err(ResponseShapeDefect::ExecutionResponseOmitsFixtureResources),
    );

    let prototype = NativePrototypeResponse {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: crate::prototype::PrototypeCaseId {
            relation: crate::prototype::PrototypeRelation::WideFloorRelation,
            name: "revision-seven-shape".to_owned(),
        },
        verdict: NativeVerdict::Rejected,
        final_stack: None,
        final_altstack: None,
        observed_failure: None,
        resources: NativeResourceObservation {
            script_bytes: Some(33),
            initial_stack_items: None,
            ..NativeResourceObservation::default()
        },
    };
    assert_eq!(
        prototype.validate_shape(&BTreeSet::new()),
        Err(ResponseShapeDefect::ExecutionResponseOmitsFixtureResources),
    );
}

#[test]
fn non_verdict_resource_records_are_uniformly_empty() {
    let mut primitive = response(NativeVerdict::InfrastructureError, None, None);
    primitive.resources.script_bytes = Some(33);
    assert_eq!(
        validate_response_shape(&primitive, &BTreeSet::new()),
        Err(ResponseShapeDefect::InfrastructureResponseCarriesObservation),
    );

    let mut operation = operation_response(ObservedOutcomeLayer::ExecutorInfrastructureFailure);
    operation.resources.initial_stack_items = Some(1);
    assert_eq!(
        operation.validate_shape(),
        Err(ResponseShapeDefect::InfrastructureResponseCarriesObservation),
    );
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

/// A key-path refusal has its own wire name, and it is a verdict.
#[test]
fn the_key_path_layer_is_spelled_and_counted_as_its_own() {
    use crate::protocol::ObservedOutcomeLayer;

    // The wire spelling, checked through serde rather than read off the
    // attribute: the adapter emits this exact string, and a rename here
    // that nobody carried across would leave the two sides describing
    // the same event under two names.
    let wire = serde_json::to_string(&ObservedOutcomeLayer::KeyPathRejection)
        .expect("the layer serializes");
    assert_eq!(wire, "\"key_path_rejection\"");
    assert_eq!(
        serde_json::from_str::<ObservedOutcomeLayer>("\"key_path_rejection\"")
            .expect("the layer parses"),
        ObservedOutcomeLayer::KeyPathRejection,
    );

    // It is NOT the script-path name, which is the whole reason it
    // exists, and it is a verdict the target reached rather than a
    // failure around the run.
    assert_ne!(
        ObservedOutcomeLayer::KeyPathRejection,
        ObservedOutcomeLayer::ScriptPathRejection,
    );
    assert!(ObservedOutcomeLayer::KeyPathRejection.is_target_verdict());
    assert_eq!(
        ObservedOutcomeLayer::KeyPathRejection.to_string(),
        "key-path rejection",
    );
}

#[test]
fn the_handshake_request_states_this_harnesss_schema() {
    assert_eq!(HandshakeRequest::default().schema, NATIVE_PROTOCOL_SCHEMA);
}

#[test]
fn this_harness_speaks_schema_eight_and_no_earlier_one() {
    // Schema 8 adds required script-path signing response members.
    // Stated as a value rather than left implicit. Schema 7 tightens the
    // conservation response shapes and makes fixture resource figures
    // required but nullable. Schema 6 widens the observed-layer
    // vocabulary with `key_path_rejection`. No record shape moves for it,
    // and the break is real all the same: an earlier harness refuses a
    // name it has never heard, so an adapter that has
    // learned to tell a key-path refusal from a script-path one would
    // have its answer read as a transport failure rather than as the
    // verdict the target reached. Schema 5 declares the
    // confidential funding arm: a fifth operation subject and two
    // response members that are not defaulted, so a revision-4 executor
    // can neither parse a revision-5 request nor produce a revision-5
    // answer. Schema 4 declares the
    // conservation openings and the typed lifecycle records, so that one
    // revision names one schema rather than two disagreeing ones; schema
    // 3 removed the expectation from the request; schema 2 added the
    // environment observation, the separated provenance roles, and the
    // bounded-record contract. An executor of any earlier revision
    // answers a question this one no longer asks, or writes a record it
    // no longer means, so they are refused for each other rather than
    // reconciled by reading whichever fields happen to overlap
    // (´[PLAN-rule:guide12-exec:protocol-revision]´).
    //
    // This assertion is also the tripwire that keeps the two
    // implementations moving together: the adapter's constant of the
    // same name is what it is compared against in the field, and a bump
    // that reached only one side is the fault G12-R09 recorded.
    assert_eq!(NATIVE_PROTOCOL_SCHEMA, 8);
    assert_ne!(NATIVE_PROTOCOL_SCHEMA, 7);
    assert_ne!(NATIVE_PROTOCOL_SCHEMA, 6);
    assert_ne!(NATIVE_PROTOCOL_SCHEMA, 5);
    assert_ne!(NATIVE_PROTOCOL_SCHEMA, 4);
    assert_ne!(NATIVE_PROTOCOL_SCHEMA, 3);
    assert_ne!(NATIVE_PROTOCOL_SCHEMA, 2);
    assert_ne!(NATIVE_PROTOCOL_SCHEMA, 1);
}

#[test]
fn a_request_carries_no_expectation_of_any_kind() {
    // The revision-3 boundary, checked over the encoded record rather
    // than over the type: a field added to the subject by a later wave
    // would have to pass this to reach the wire
    // (´[PLAN-rule:guide11-exec:request-subject]´).
    let target = crate::tests::support::reviewed_target();
    let binding = crate::tests::support::development_binding(&target);
    let census = crate::fixture::canonical_fixture_set(&target, &binding)
        .expect("the canonical census states");
    let fixture = census.iter().next().expect("the census is not empty");

    let request = crate::protocol::NativeExecutionRequest {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: fixture.case(),
        subject: fixture.subject(),
        construction: None,
    };
    let encoded = serde_json::to_string(&request).expect("the request encodes");

    for forbidden in [
        "expected",
        "expected_resources",
        "claims",
        "static_final_stack",
        "static_final_altstack",
        "classes",
        "evidence",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "a revision-3 request must not carry {forbidden}: {encoded}",
        );
    }
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

/// The reviewed adapter's failure-class table, as it is written.
///
/// Read from the adapter's own source rather than restated here. A copy
/// of the table in this crate would be a second table, and the failure
/// this test exists to catch is exactly the two disagreeing: the adapter
/// names a class the harness does not admit, the harness refuses the
/// response, and every refusing row of a native run becomes a malformed
/// response instead of the target verdict it is.
fn adapter_failure_class_table() -> Vec<(String, String)> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("the crate sits two levels below the repository root")
        .to_path_buf();
    let source = std::fs::read_to_string(root.join("scripts/elements-native-executor.py"))
        .expect("the reviewed adapter is in the repository");

    let mut rows = Vec::new();
    let mut inside = false;
    for line in source.lines() {
        if line.starts_with("FAILURE_CLASS_BY_SCRIPT_ERROR = {") {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        if line.starts_with('}') {
            break;
        }
        let trimmed = line.trim();
        // Comment lines carry the reasoning and no entry.
        if !trimmed.starts_with('"') {
            continue;
        }
        let entry = trimmed.trim_end_matches(',');
        let (key, value) = entry
            .rsplit_once("\": \"")
            .expect("every table entry is a quoted key and a quoted value");
        rows.push((
            key.trim_start_matches('"').to_owned(),
            value.trim_end_matches('"').to_owned(),
        ));
    }
    assert!(!rows.is_empty(), "the adapter's table was located and read");
    rows
}

/// Every class the reviewed adapter names is one this harness admits.
///
/// The direction that matters. A harness class no adapter reports is an
/// observation nothing has made yet, which is an ordinary state; an
/// adapter class the harness cannot parse is a run that cannot report
/// what it saw.
#[test]
fn every_adapter_failure_class_is_one_the_harness_admits() {
    for (script_error, class) in adapter_failure_class_table() {
        let quoted = format!("\"{class}\"");
        serde_json::from_str::<ObservedFailureClass>(&quoted).unwrap_or_else(|_| {
            panic!("the adapter maps {script_error:?} to {class:?}, which this harness cannot read")
        });
    }
}

/// The five target refusals this wave gave a class of their own.
///
/// Pinned by their exact adapter strings, because the point of the work
/// was that reporting no class for an observed refusal reports less than
/// was observed — and, from an adapter advertising failure-class
/// reporting, is an unanswerable response rather than a cautious one.
#[test]
fn the_shape_and_authentication_refusals_are_classified() {
    let table = adapter_failure_class_table();
    let expected = [
        ("Stack size limit exceeded", "stack_size_limit_exceeded"),
        ("Script is too big", "script_size_limit_exceeded"),
        (
            "Operation limit exceeded",
            "script_operation_limit_exceeded",
        ),
        (
            "Invalid Schnorr signature hash type",
            "invalid_signature_hash_type",
        ),
        (
            "Invalid Taproot control block size",
            "malformed_control_block",
        ),
    ];
    for (script_error, class) in expected {
        let found = table
            .iter()
            .find(|(key, _)| key == script_error)
            .unwrap_or_else(|| panic!("the adapter classifies {script_error:?}"));
        assert_eq!(found.1, class, "{script_error:?} keeps its reviewed class");
    }
}

/// No two failure classes share a wire spelling.
///
/// A shared spelling would make two observations one value, and a report
/// would record whichever the parser reached first.
#[test]
fn every_failure_class_spelling_is_distinct() {
    let classes = [
        ObservedFailureClass::StackUnderflow,
        ObservedFailureClass::InvalidOperandWidth,
        ObservedFailureClass::MalformedScriptNumber,
        ObservedFailureClass::ScriptNumberRangeExceeded,
        ObservedFailureClass::FixedWidthConversionRefused,
        ObservedFailureClass::UnsupportedExecutionDomain,
        ObservedFailureClass::IntrospectionContextUnavailable,
        ObservedFailureClass::IntrospectionIndexOutOfRange,
        ObservedFailureClass::HashContextLoad,
        ObservedFailureClass::HashContextWrite,
        ObservedFailureClass::ArithmeticOverflow,
        ObservedFailureClass::DivisionByZero,
        ObservedFailureClass::EmptySignature,
        ObservedFailureClass::InvalidSignature,
        ObservedFailureClass::InvalidSignatureHashType,
        ObservedFailureClass::InvalidPublicKeyEncoding,
        ObservedFailureClass::InvalidCurveRelation,
        ObservedFailureClass::UnsatisfiedTimelock,
        ObservedFailureClass::NegativeTimelock,
        ObservedFailureClass::ValidationBudgetExhausted,
        ObservedFailureClass::ScriptSizeLimitExceeded,
        ObservedFailureClass::ScriptOperationLimitExceeded,
        ObservedFailureClass::StackSizeLimitExceeded,
        ObservedFailureClass::UnknownOpcode,
        ObservedFailureClass::MalformedPush,
        ObservedFailureClass::LeafVersionRejected,
        ObservedFailureClass::MalformedControlBlock,
        ObservedFailureClass::EvaluatedFalse,
        ObservedFailureClass::ResultSizeExceeded,
        ObservedFailureClass::SliceOutOfRange,
        ObservedFailureClass::UnequalOperands,
        ObservedFailureClass::FalseVerification,
        ObservedFailureClass::MismatchedOperandWidths,
        ObservedFailureClass::NonSingletonFinalStack,
    ];
    let spellings: BTreeSet<String> = classes
        .iter()
        .map(|class| serde_json::to_string(class).expect("a failure class serializes"))
        .collect();
    assert_eq!(
        spellings.len(),
        classes.len(),
        "no two failure classes share a spelling",
    );
}

/// The five operation subjects, and the one thing that could go wrong.
///
/// `OperationSubject` is untagged, so a subject is recognized by its
/// members alone. Five shapes now share that discrimination where two
/// used to, and a shape parsing as the wrong variant would be answered
/// by the wrong half of an adapter. Each is therefore round-tripped and
/// then checked to land on its own variant, which is the property the
/// untagged representation actually needs.
#[test]
fn every_operation_subject_round_trips_to_its_own_variant() {
    use crate::protocol::{
        OperationStepKind, OperationSubject, TargetFundingSubject, TargetSponsorFundingSubject,
        TargetSponsorSigningSubject, TargetSubmissionSubject, WireOutpoint, WireSighashProfile,
    };

    use super::support::confidential_subject;

    let subjects = [
        OperationSubject::Funding(Box::new(TargetFundingSubject {
            issue_asset: false,
            asset: Some("aa".to_owned()),
            output_program: vec![0x51, 0x20],
            outputs: 1,
            amount_per_output: 7,
        })),
        OperationSubject::Submission(Box::new(TargetSubmissionSubject {
            transaction_bytes: vec![0x02, 0x00],
        })),
        OperationSubject::SponsorFunding(Box::new(TargetSponsorFundingSubject {
            sponsor_outputs: 2,
            amount_per_sponsor_output: 900,
        })),
        OperationSubject::SponsorSigning(Box::new(TargetSponsorSigningSubject {
            finalized_transaction: vec![0x02, 0x00, 0x01],
            sponsor_input_index: 3,
            sponsor_outpoint: WireOutpoint {
                txid: "ab".repeat(32),
                vout: 1,
            },
            sighash_profile: WireSighashProfile::AllInputsAllOutputs,
        })),
        OperationSubject::ConfidentialFunding(Box::new(confidential_subject())),
    ];
    let kinds = [
        OperationStepKind::Fund,
        OperationStepKind::Submit,
        OperationStepKind::FundSponsor,
        OperationStepKind::SignSponsor,
        OperationStepKind::FundConfidential,
    ];

    for (subject, kind) in subjects.iter().zip(kinds) {
        let text = serde_json::to_string(subject).expect("the subject serializes");
        let parsed: OperationSubject = serde_json::from_str(&text).expect("the subject parses");
        assert_eq!(&parsed, subject, "a subject changed on the way round");
        assert_eq!(parsed.kind(), kind, "a subject parsed as another kind");
    }
}

/// A step kind is spelled one way, on the wire and in a diagnostic.
///
/// The defect this closes was found by a live node and not by a test: a
/// `Display` spelling the two sponsor kinds with hyphens while serde
/// spelled them with underscores meant the adapter, which reads the
/// kind out of the record, refused every sponsor step as being of an
/// unknown kind. Nothing compared the two spellings until an adapter
/// did.
#[test]
fn a_step_kind_renders_the_way_it_serializes() {
    use crate::protocol::OperationStepKind;

    for kind in [
        OperationStepKind::Fund,
        OperationStepKind::Submit,
        OperationStepKind::FundSponsor,
        OperationStepKind::SignSponsor,
        OperationStepKind::FundConfidential,
    ] {
        let wire = serde_json::to_string(&kind).expect("the kind serializes");
        // The serialized form is a JSON string, so the quotes come off
        // before the comparison; what is compared is the word itself.
        let wire = wire.trim_matches('"');
        assert_eq!(
            kind.to_string(),
            wire,
            "the kind renders and serializes differently",
        );
    }
}

/// The sponsor steps are gated, and gated together.
#[test]
fn a_sponsor_step_is_refused_by_an_executor_that_did_not_advertise_one() {
    use crate::protocol::{
        OperationSubject, TargetSponsorFundingSubject, TargetSponsorSigningSubject, WireOutpoint,
        WireSighashProfile,
    };

    let funding = OperationSubject::SponsorFunding(Box::new(TargetSponsorFundingSubject {
        sponsor_outputs: 1,
        amount_per_sponsor_output: 1,
    }));
    let signing = OperationSubject::SponsorSigning(Box::new(TargetSponsorSigningSubject {
        finalized_transaction: vec![0x02],
        sponsor_input_index: 0,
        sponsor_outpoint: WireOutpoint {
            txid: "cd".repeat(32),
            vout: 0,
        },
        sighash_profile: WireSighashProfile::AllInputsAllOutputs,
    }));

    // An executor holding every other operation capability is still
    // refused both sponsor steps: being able to fund a ceremony and to
    // submit a transaction says nothing about holding a reserve.
    let mut executor = handshake();
    executor.capabilities = BTreeSet::from([
        ExecutorCapability::TestFundingCeremony,
        ExecutorCapability::TargetTransactionSubmission,
    ]);
    assert!(!executor.runs_operation_step(&funding));
    assert!(!executor.runs_operation_step(&signing));

    // And the one capability admits both halves, which is what makes
    // them one capability rather than two.
    executor
        .capabilities
        .insert(ExecutorCapability::TestSponsorAuthorization);
    assert!(executor.runs_operation_step(&funding));
    assert!(executor.runs_operation_step(&signing));
}

/// The two ways a member can be added, held apart in one record.
///
/// The sponsor members were defaulted so that adding them was NOT a
/// revision: a record written by an executor that predates them is read
/// rather than refused, and that half is unchanged here.
///
/// The confidential members are not defaulted so that adding them IS
/// one. A revision-4 record therefore no longer parses at all, which is
/// the property that keeps a revision-4 executor from answering a
/// confidential request with silence in the members the answer lives in.
/// The refusal is the schema's, before any capability is consulted.
#[test]
fn a_revision_four_response_no_longer_parses_and_the_sponsor_members_still_default() {
    use crate::protocol::NativeOperationResponse;

    let revision_four = r#"{
        "schema": 4,
        "case": {"operation": "fund", "step": "issue"},
        "observed_layer": "accepted",
        "observed_detail": null,
        "issued_asset": "aa",
        "funded_outputs": [],
        "accepted_txid": null,
        "resources": {
            "script_bytes": 0,
            "initial_stack_items": 0,
            "peak_stack_items": null,
            "peak_altstack_items": null,
            "maximum_element_bytes": null,
            "validation_budget_used": null,
            "transaction_weight": null
        }
    }"#;
    assert!(
        serde_json::from_str::<NativeOperationResponse>(revision_four).is_err(),
        "a record omitting the undefaulted confidential members must be refused",
    );

    let revision_eight = r#"{
        "schema": 8,
        "case": {"operation": "fund", "step": "issue"},
        "observed_layer": "accepted",
        "observed_detail": null,
        "issued_asset": "aa",
        "funded_outputs": [],
        "confidential_funded_outputs": [],
        "mined_readback": null,
        "accepted_txid": null,
        "script_path_witness": [],
        "signer_public_key": null,
        "signed_profile": null,
        "signing_genesis": null,
        "resources": {
            "script_bytes": 0,
            "initial_stack_items": 0,
            "peak_stack_items": null,
            "peak_altstack_items": null,
            "maximum_element_bytes": null,
            "validation_budget_used": null,
            "transaction_weight": null
        }
    }"#;
    let parsed: NativeOperationResponse = serde_json::from_str(revision_eight)
        .expect("a current record without the sponsor members reads");
    assert!(
        parsed.sponsor_witness.is_empty(),
        "the sponsor witness is still defaulted",
    );
    assert_eq!(parsed.signature_bound_to, None);
    assert_eq!(parsed.confidential_funded_outputs, Vec::new());
    assert_eq!(parsed.mined_readback, None);
}

/// An authorization belongs to the step that asked for one.
#[test]
fn only_a_signing_step_may_report_an_authorization() {
    use crate::protocol::{
        NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId, OperationStepKind,
    };

    let signed = |kind: OperationStepKind| NativeOperationResponse {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: OperationCaseId {
            operation: kind,
            step: "step".to_owned(),
        },
        observed_layer: ObservedOutcomeLayer::Accepted,
        observed_detail: None,
        issued_asset: None,
        funded_outputs: Vec::new(),
        confidential_funded_outputs: Vec::new(),
        mined_readback: None,
        accepted_txid: None,
        sponsor_witness: vec![vec![0x30], vec![0x02]],
        signature_bound_to: Some(vec![0x02]),
        script_path_witness: Vec::new(),
        signer_public_key: None,
        signed_profile: None,
        signing_genesis: None,
        resources: NativeResourceObservation::default(),
    };

    signed(OperationStepKind::SignSponsor)
        .validate_shape()
        .expect("the step that was asked for an authorization may report one");
    for kind in [
        OperationStepKind::Fund,
        OperationStepKind::Submit,
        OperationStepKind::FundSponsor,
    ] {
        assert_eq!(
            signed(kind).validate_shape(),
            Err(ResponseShapeDefect::OperationResponseMismatchesStep),
            "{kind} reported an authorization nobody asked it for",
        );
    }
}

/// An accepted signing step owes both halves of an authorization.
#[test]
fn an_accepted_authorization_states_a_stack_and_what_it_was_bound_to() {
    use crate::protocol::{
        NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId, OperationStepKind,
    };

    let partial = |stack: Vec<Vec<u8>>, bound: Option<Vec<u8>>| NativeOperationResponse {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: OperationCaseId {
            operation: OperationStepKind::SignSponsor,
            step: "sign".to_owned(),
        },
        observed_layer: ObservedOutcomeLayer::Accepted,
        observed_detail: None,
        issued_asset: None,
        funded_outputs: Vec::new(),
        confidential_funded_outputs: Vec::new(),
        mined_readback: None,
        accepted_txid: None,
        sponsor_witness: stack,
        signature_bound_to: bound,
        script_path_witness: Vec::new(),
        signer_public_key: None,
        signed_profile: None,
        signing_genesis: None,
        resources: NativeResourceObservation::default(),
    };

    // A stack with nothing to bind it to, and a binding with no stack.
    // Either alone is unusable, and an acceptance reporting one is
    // indistinguishable from an adapter that returned the layer without
    // doing the work.
    assert_eq!(
        partial(vec![vec![0x30]], None).validate_shape(),
        Err(ResponseShapeDefect::AcceptedOperationOmitsObservation)
    );
    assert_eq!(
        partial(Vec::new(), Some(vec![0x02])).validate_shape(),
        Err(ResponseShapeDefect::AcceptedOperationOmitsObservation)
    );
    partial(vec![vec![0x30]], Some(vec![0x02]))
        .validate_shape()
        .expect("both halves is an authorization");
}

// -- G13-R14: the response shapes, over role, kind, and verdict -------

/// The lifecycle response shape, held to the whole product of role and
/// outcome.
///
/// `G13-R14` found `validate_shape` reading one member of the record —
/// whether the step ran — and nothing else, so a construct response
/// could report the verifier's verdict, a verify response could publish
/// a handoff, and neither was a shape the protocol defines. The rules
/// are now a match over the pair, and these tests walk the pair.
mod lifecycle_response_shapes {
    use super::*;

    use crate::lifecycle::{HANDOFF_SCHEMA, LifecycleOutcome, PublicHandoff};
    use crate::normalization::{AuthorizationProfile, ObservedOutput};
    use crate::protocol::{
        LifecycleCaseId, LifecycleCheck, LifecycleSpend, LifecycleStepRole,
        NativeLifecycleResponse, WireOutpoint,
    };

    /// Every outcome this record can carry, in one list.
    ///
    /// Written out rather than derived, because the point of the tests
    /// below is to walk the product of role and outcome, and a list
    /// derived from the type would agree with the type by construction.
    const EVERY_OUTCOME: [LifecycleOutcome; 10] = [
        LifecycleOutcome::Constructed,
        LifecycleOutcome::Verified,
        LifecycleOutcome::RefusedEvidenceAbsent,
        LifecycleOutcome::RefusedCopiedEvidence,
        LifecycleOutcome::RefusedOutputAbsent,
        LifecycleOutcome::RefusedOutputNotExplicit,
        LifecycleOutcome::RefusedOutputSpent,
        LifecycleOutcome::RefusedWrongChainContext,
        LifecycleOutcome::ExecutorInfrastructureFailure,
        LifecycleOutcome::FixtureConstructionFailure,
    ];

    /// The six refusals a reading process reaches.
    const EVERY_REFUSAL: [LifecycleOutcome; 6] = [
        LifecycleOutcome::RefusedEvidenceAbsent,
        LifecycleOutcome::RefusedCopiedEvidence,
        LifecycleOutcome::RefusedOutputAbsent,
        LifecycleOutcome::RefusedOutputNotExplicit,
        LifecycleOutcome::RefusedOutputSpent,
        LifecycleOutcome::RefusedWrongChainContext,
    ];

    /// One response with nothing in it but its identity.
    fn bare(role: LifecycleStepRole, outcome: LifecycleOutcome) -> NativeLifecycleResponse {
        NativeLifecycleResponse {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case: LifecycleCaseId { lifecycle: role },
            outcome,
            handoff: None,
            authorization_profile: None,
            observed_witness_sizes: Vec::new(),
            observed_outputs: Vec::new(),
            checks: Vec::new(),
            spend: None,
            superseded_by: None,
            supersede_failure: None,
            detail: None,
        }
    }

    /// What Process A publishes, in the shape it publishes it.
    fn handoff() -> PublicHandoff {
        PublicHandoff {
            schema: HANDOFF_SCHEMA.to_owned(),
            chain_name: "a-development-chain".to_owned(),
            network_id: "11".repeat(32),
            genesis_id: "22".repeat(32),
            txid: "33".repeat(32),
            output_index: 0,
            block_hash: "44".repeat(32),
            block_height: 101,
            raw_transaction: "0200000000".to_owned(),
            claimed_explicit_amount: 100_000,
            claimed_explicit_asset: "55".repeat(32),
            claimed_owner_address: "an-address".to_owned(),
        }
    }

    /// One thing Process B looked for.
    fn check() -> LifecycleCheck {
        LifecycleCheck {
            check: "chain_context_genesis".to_owned(),
            expected: "22".repeat(32),
            observed: "22".repeat(32),
            agrees: true,
        }
    }

    /// The spend Process B builds out of its own funds.
    fn spend() -> LifecycleSpend {
        LifecycleSpend {
            txid: "66".repeat(32),
            amount: 90_000,
            destination: "another-address".to_owned(),
            destination_script: "0014".to_owned(),
            consumed_outpoint: WireOutpoint {
                txid: "77".repeat(32),
                vout: 1,
            },
            consumed_owner_object: false,
        }
    }

    /// A well-formed record for the publishing process's own success.
    fn published() -> NativeLifecycleResponse {
        NativeLifecycleResponse {
            handoff: Some(handoff()),
            ..bare(LifecycleStepRole::Construct, LifecycleOutcome::Constructed)
        }
    }

    /// A well-formed record for the reading process's own success.
    fn verified() -> NativeLifecycleResponse {
        NativeLifecycleResponse {
            checks: vec![check()],
            spend: Some(spend()),
            ..bare(LifecycleStepRole::Verify, LifecycleOutcome::Verified)
        }
    }

    /// The verdict, taken over a record that made the round trip.
    ///
    /// Every contradiction below is a record the wire carries and the
    /// parser accepts — `deny_unknown_fields` has nothing to object to,
    /// since every member is declared. That is the whole point of the
    /// row: what refuses these is the shape rule, and nothing else
    /// would.
    fn verdict(response: &NativeLifecycleResponse) -> Result<(), ResponseShapeDefect> {
        let text = serde_json::to_string(response).expect("the response serializes");
        let read: NativeLifecycleResponse =
            serde_json::from_str(&text).expect("the record is one this side declares");
        assert_eq!(&read, response, "the record survives both directions");
        read.validate_shape()
    }

    /// Every pair of role and outcome is classified, and only the pairs
    /// the role reaches are admitted.
    #[test]
    fn a_verdict_belongs_to_the_role_that_reaches_it() {
        for outcome in EVERY_OUTCOME {
            for role in [LifecycleStepRole::Construct, LifecycleStepRole::Verify] {
                let reachable = match role {
                    LifecycleStepRole::Construct => matches!(
                        outcome,
                        LifecycleOutcome::Constructed
                            | LifecycleOutcome::ExecutorInfrastructureFailure
                            | LifecycleOutcome::FixtureConstructionFailure
                    ),
                    LifecycleStepRole::Verify => !matches!(outcome, LifecycleOutcome::Constructed),
                };
                if reachable {
                    continue;
                }
                assert_eq!(
                    verdict(&bare(role, outcome)),
                    Err(ResponseShapeDefect::LifecycleOutcomeMismatchesRole),
                    "{role:?} cannot report {outcome:?}",
                );
            }
        }
    }

    /// A construct step may not report what only the reading process
    /// produces.
    #[test]
    fn a_construct_response_carries_no_reader_member() {
        verdict(&published()).expect("a published record with a handoff is well formed");
        for contradictory in [
            NativeLifecycleResponse {
                checks: vec![check()],
                ..published()
            },
            NativeLifecycleResponse {
                spend: Some(spend()),
                ..published()
            },
        ] {
            assert_eq!(
                verdict(&contradictory),
                Err(ResponseShapeDefect::LifecycleResponseMismatchesStep),
            );
        }
    }

    /// A verify step may not report what only the publishing process
    /// produces — under its acceptance or under any of its refusals.
    #[test]
    fn a_verify_response_carries_no_publisher_member() {
        for outcome in EVERY_REFUSAL {
            verdict(&NativeLifecycleResponse {
                checks: vec![check()],
                ..bare(LifecycleStepRole::Verify, outcome)
            })
            .expect("a refusal that looked something up is well formed");
        }
        verdict(&verified()).expect("a verification with checks and a spend is well formed");

        // Each publishing member in turn, against a record that is
        // otherwise exactly the well-formed verification.
        let publisher: [NativeLifecycleResponse; 5] = [
            NativeLifecycleResponse {
                handoff: Some(handoff()),
                ..verified()
            },
            NativeLifecycleResponse {
                authorization_profile: Some(AuthorizationProfile::SighashDefault),
                ..verified()
            },
            NativeLifecycleResponse {
                observed_witness_sizes: vec![vec![64]],
                ..verified()
            },
            NativeLifecycleResponse {
                observed_outputs: vec![ObservedOutput {
                    script_pubkey: vec![0x51],
                    explicit_amount: Some(100_000),
                    explicit_asset: None,
                    is_fee: false,
                }],
                ..verified()
            },
            NativeLifecycleResponse {
                superseded_by: Some("88".repeat(32)),
                ..verified()
            },
        ];
        for contradictory in publisher {
            assert_eq!(
                verdict(&contradictory),
                Err(ResponseShapeDefect::LifecycleResponseMismatchesStep),
            );
        }

        // The stale row's failure reason is a reason rather than an
        // observation, which is why it is exempt from the did-not-run
        // rule — and it is still Process A's, so a reading process has
        // no business reporting one under any outcome at all.
        assert_eq!(
            verdict(&NativeLifecycleResponse {
                supersede_failure: Some("the coin was already spent".to_owned()),
                ..verified()
            }),
            Err(ResponseShapeDefect::LifecycleResponseMismatchesStep),
        );
        assert_eq!(
            verdict(&NativeLifecycleResponse {
                supersede_failure: Some("the coin was already spent".to_owned()),
                ..bare(
                    LifecycleStepRole::Verify,
                    LifecycleOutcome::ExecutorInfrastructureFailure
                )
            }),
            Err(ResponseShapeDefect::LifecycleResponseMismatchesStep),
        );
    }

    /// A refusal stops short of the spend, which is built only after
    /// everything the record claimed has been confirmed.
    #[test]
    fn a_refusing_verify_response_reports_no_spend() {
        for outcome in EVERY_REFUSAL {
            assert_eq!(
                verdict(&NativeLifecycleResponse {
                    checks: vec![check()],
                    spend: Some(spend()),
                    ..bare(LifecycleStepRole::Verify, outcome)
                }),
                Err(ResponseShapeDefect::LifecycleResponseMismatchesStep),
                "{outcome:?} reported a spend its own verdict says was never reached",
            );
        }
    }

    /// A step that ran owes what its outcome rests on.
    #[test]
    fn a_step_that_ran_reports_what_its_outcome_rests_on() {
        // Publishing, with nothing published.
        assert_eq!(
            verdict(&bare(
                LifecycleStepRole::Construct,
                LifecycleOutcome::Constructed
            )),
            Err(ResponseShapeDefect::LifecycleStepOmitsObservation),
        );

        // Verification, missing either half of what it claims.
        for contradictory in [
            NativeLifecycleResponse {
                checks: Vec::new(),
                ..verified()
            },
            NativeLifecycleResponse {
                spend: None,
                ..verified()
            },
        ] {
            assert_eq!(
                verdict(&contradictory),
                Err(ResponseShapeDefect::LifecycleStepOmitsObservation),
            );
        }

        // A refusal with no check behind it is a verdict with no
        // evidence: every one of them is reached by looking something
        // up and finding it wrong.
        for outcome in EVERY_REFUSAL {
            assert_eq!(
                verdict(&bare(LifecycleStepRole::Verify, outcome)),
                Err(ResponseShapeDefect::LifecycleStepOmitsObservation),
                "{outcome:?} was reported with nothing looked up",
            );
        }
    }

    /// A step that did not run is still entitled to say why.
    #[test]
    fn a_step_that_did_not_run_reports_only_its_reason() {
        for outcome in [
            LifecycleOutcome::ExecutorInfrastructureFailure,
            LifecycleOutcome::FixtureConstructionFailure,
        ] {
            for role in [LifecycleStepRole::Construct, LifecycleStepRole::Verify] {
                verdict(&NativeLifecycleResponse {
                    detail: Some("the adapter reached no node".to_owned()),
                    ..bare(role, outcome)
                })
                .expect("a failure is entitled to a reason");
            }
            assert_eq!(
                verdict(&NativeLifecycleResponse {
                    handoff: Some(handoff()),
                    ..bare(LifecycleStepRole::Construct, outcome)
                }),
                Err(ResponseShapeDefect::InfrastructureResponseCarriesObservation),
            );
        }
    }
}

/// The operation response shape, held to the product of kind and
/// verdict.
///
/// The half `G13-R14` names is the converse of the rule that was there:
/// an acceptance had to show what its kind produces, and a refusal was
/// free to show it too.
mod operation_response_shapes {
    use super::*;

    use crate::protocol::{
        ConfidentialFundedOutput, FundedOutput, MinedFundingReadback, NativeOperationResponse,
        ObservedOutcomeLayer, OperationCaseId, OperationStepKind, WireOutpoint,
    };

    /// Every kind of step this record answers.
    const EVERY_KIND: [OperationStepKind; 5] = [
        OperationStepKind::Fund,
        OperationStepKind::Submit,
        OperationStepKind::FundSponsor,
        OperationStepKind::SignSponsor,
        OperationStepKind::FundConfidential,
    ];

    /// The three layers at which the target refused what it judged.
    ///
    /// All three are verdicts: the target answered. The two layers that
    /// are not verdicts are governed by the older rule, and are checked
    /// where that rule is.
    const EVERY_REJECTION: [ObservedOutcomeLayer; 3] = [
        ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
        ObservedOutcomeLayer::ScriptPathRejection,
        ObservedOutcomeLayer::RelayPolicyRejection,
    ];

    /// One response with nothing in it but its identity.
    fn bare(kind: OperationStepKind, layer: ObservedOutcomeLayer) -> NativeOperationResponse {
        NativeOperationResponse {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case: OperationCaseId {
                operation: kind,
                step: "step".to_owned(),
            },
            observed_layer: layer,
            observed_detail: None,
            issued_asset: None,
            funded_outputs: Vec::new(),
            confidential_funded_outputs: Vec::new(),
            mined_readback: None,
            accepted_txid: None,
            sponsor_witness: Vec::new(),
            signature_bound_to: None,
            script_path_witness: Vec::new(),
            signer_public_key: None,
            signed_profile: None,
            signing_genesis: None,
            resources: NativeResourceObservation::default(),
        }
    }

    /// One coin a funding step created.
    fn coin() -> FundedOutput {
        FundedOutput {
            outpoint: WireOutpoint {
                txid: "33".repeat(32),
                vout: 0,
            },
            asset: "an-asset".to_owned(),
            amount_satoshis: 100_000,
            script: "5120".to_owned(),
        }
    }

    /// One output a confidential funding step created.
    ///
    /// Arbitrary public development bytes. Nothing here is a commitment
    /// anything opened, a proof anything verified, or material of any
    /// kind: the shape census is about which members a kind may carry,
    /// and it reaches no target and checks no arithmetic.
    fn confidential_coin() -> ConfidentialFundedOutput {
        ConfidentialFundedOutput {
            outpoint: WireOutpoint {
                txid: "44".repeat(32),
                vout: 0,
            },
            explicit_asset: "an-asset".to_owned(),
            value_commitment: vec![0x08; 33],
            nonce: vec![0x02; 33],
            script: "5120".to_owned(),
            output_witness_index: 0,
            surjection_proof: Vec::new(),
            rangeproof: vec![0x11; 64],
        }
    }

    /// One mined readback.
    fn readback() -> MinedFundingReadback {
        MinedFundingReadback {
            transaction_id: "44".repeat(32),
            witness_transaction_id: "55".repeat(32),
            block_hash: "66".repeat(32),
            block_height: 101,
            raw_transaction: vec![0x02, 0x00, 0x00, 0x00],
        }
    }

    /// The verdict, taken over a record that made the round trip.
    fn verdict(response: &NativeOperationResponse) -> Result<(), ResponseShapeDefect> {
        let text = serde_json::to_string(response).expect("the response serializes");
        let read: NativeOperationResponse =
            serde_json::from_str(&text).expect("the record is one this side declares");
        assert_eq!(&read, response, "the record survives both directions");
        read.validate_shape()
    }

    /// What each kind produces when the target accepts it.
    fn accepted(kind: OperationStepKind) -> NativeOperationResponse {
        let mut response = bare(kind, ObservedOutcomeLayer::Accepted);
        match kind {
            OperationStepKind::Fund => {
                response.issued_asset = Some("an-asset".to_owned());
                response.funded_outputs = vec![coin()];
            }
            OperationStepKind::FundSponsor => response.funded_outputs = vec![coin()],
            // Both halves, on the reasoning the confidential arm gives:
            // an identity with no readback is a name for bytes nobody
            // can look at again.
            OperationStepKind::Submit => {
                response.accepted_txid = Some("99".repeat(32));
                response.mined_readback = Some(readback());
            }
            OperationStepKind::SignScriptPath => return super::script_path_response(),
            OperationStepKind::SignSponsor => {
                response.sponsor_witness = vec![vec![0x30; 71], vec![0x02; 33]];
                response.signature_bound_to = Some(vec![0x02, 0x00]);
            }
            // Both halves, because an acceptance owes both: outputs with
            // no mined readback are proofs nobody can check against a
            // chain, and a readback with no outputs is a block identity
            // with nothing in it.
            OperationStepKind::FundConfidential => {
                response.issued_asset = Some("an-asset".to_owned());
                response.confidential_funded_outputs = vec![confidential_coin()];
                response.mined_readback = Some(readback());
            }
            // The same two halves and no issued asset, because the
            // reserve arm funds coins of an asset the chain already has
            // and no run brings one into existence.
            OperationStepKind::FundConfidentialSponsor => {
                response.confidential_funded_outputs = vec![confidential_coin()];
                response.mined_readback = Some(readback());
            }
        }
        response
    }

    /// The acceptances themselves are well formed, which is what makes
    /// the refusals below a statement about the verdict.
    #[test]
    fn an_acceptance_reporting_what_its_kind_produces_is_well_formed() {
        for kind in EVERY_KIND {
            verdict(&accepted(kind)).expect("the acceptance shows what its kind produces");
        }
    }

    /// A refused step reports none of what an acceptance produces.
    ///
    /// The whole product: every kind, at every layer the target refused
    /// at, carrying exactly what that kind's acceptance would have.
    #[test]
    fn a_refused_step_reports_none_of_what_an_acceptance_produces() {
        for kind in EVERY_KIND {
            for layer in EVERY_REJECTION {
                // A refusal with nothing to show for it is the shape
                // the protocol defines, and a reason is always allowed.
                verdict(&NativeOperationResponse {
                    observed_detail: Some("the target refused it".to_owned()),
                    ..bare(kind, layer)
                })
                .expect("a refusal with nothing to show for it is well formed");

                assert_eq!(
                    verdict(&NativeOperationResponse {
                        observed_layer: layer,
                        ..accepted(kind)
                    }),
                    Err(ResponseShapeDefect::RefusedOperationCarriesObservation),
                    "a {kind} refused at {layer} reported what only an acceptance produces",
                );
            }
        }
    }

    /// Each owned member is refused on a rejection on its own.
    ///
    /// Separately, because a rule that only fired when every one of a
    /// kind's members was present would admit the record carrying one.
    #[test]
    fn each_owned_member_alone_is_refused_on_a_rejection() {
        for layer in EVERY_REJECTION {
            for contradictory in [
                NativeOperationResponse {
                    issued_asset: Some("an-asset".to_owned()),
                    ..bare(OperationStepKind::Fund, layer)
                },
                NativeOperationResponse {
                    funded_outputs: vec![coin()],
                    ..bare(OperationStepKind::Fund, layer)
                },
                NativeOperationResponse {
                    funded_outputs: vec![coin()],
                    ..bare(OperationStepKind::FundSponsor, layer)
                },
                NativeOperationResponse {
                    accepted_txid: Some("99".repeat(32)),
                    ..bare(OperationStepKind::Submit, layer)
                },
                NativeOperationResponse {
                    sponsor_witness: vec![vec![0x30; 71]],
                    ..bare(OperationStepKind::SignSponsor, layer)
                },
                NativeOperationResponse {
                    signature_bound_to: Some(vec![0x02, 0x00]),
                    ..bare(OperationStepKind::SignSponsor, layer)
                },
                NativeOperationResponse {
                    issued_asset: Some("an-asset".to_owned()),
                    ..bare(OperationStepKind::FundConfidential, layer)
                },
                NativeOperationResponse {
                    confidential_funded_outputs: vec![confidential_coin()],
                    ..bare(OperationStepKind::FundConfidential, layer)
                },
                NativeOperationResponse {
                    mined_readback: Some(readback()),
                    ..bare(OperationStepKind::FundConfidential, layer)
                },
            ] {
                assert_eq!(
                    verdict(&contradictory),
                    Err(ResponseShapeDefect::RefusedOperationCarriesObservation),
                );
            }
        }
    }

    /// A step reports nothing another kind produces, whatever it
    /// concluded.
    ///
    /// The cross-kind half, over the whole product: each kind's own
    /// well-formed acceptance, plus one member some other kind owns.
    #[test]
    fn a_step_reports_nothing_another_kind_produces() {
        for kind in EVERY_KIND {
            let mut foreign = Vec::new();
            // The two arms that may issue the disposable asset are the
            // two funding arms: both create coins carrying an asset the
            // target may have chosen in the same step.
            if !matches!(
                kind,
                OperationStepKind::Fund | OperationStepKind::FundConfidential
            ) {
                foreign.push(NativeOperationResponse {
                    issued_asset: Some("an-asset".to_owned()),
                    ..accepted(kind)
                });
            }
            if !matches!(kind, OperationStepKind::FundConfidential) {
                foreign.push(NativeOperationResponse {
                    confidential_funded_outputs: vec![confidential_coin()],
                    mined_readback: Some(readback()),
                    ..accepted(kind)
                });
            }
            if !matches!(
                kind,
                OperationStepKind::Fund | OperationStepKind::FundSponsor
            ) {
                foreign.push(NativeOperationResponse {
                    funded_outputs: vec![coin()],
                    ..accepted(kind)
                });
            }
            if !matches!(kind, OperationStepKind::Submit) {
                foreign.push(NativeOperationResponse {
                    accepted_txid: Some("99".repeat(32)),
                    ..accepted(kind)
                });
            }
            if !matches!(kind, OperationStepKind::SignSponsor) {
                foreign.push(NativeOperationResponse {
                    sponsor_witness: vec![vec![0x30; 71]],
                    signature_bound_to: Some(vec![0x02, 0x00]),
                    ..accepted(kind)
                });
            }

            assert!(
                foreign.len() >= 2,
                "every kind leaves at least two of the four member groups to other kinds",
            );
            for contradictory in foreign {
                assert_eq!(
                    verdict(&contradictory),
                    Err(ResponseShapeDefect::OperationResponseMismatchesStep),
                    "a {kind} reported a member of some other kind",
                );
            }
        }
    }
}

pub(super) fn script_path_subject() -> crate::protocol::TargetScriptPathSigningSubject {
    use crate::protocol::{
        TargetScriptPathSigningSubject, WireSighashProfile, WireSpentOutput, WireTapleaf,
    };
    TargetScriptPathSigningSubject {
        finalized_transaction: vec![2, 0, 1],
        input_index: 1,
        spent_outputs: vec![
            WireSpentOutput {
                asset_field: vec![1; 33],
                value_field: vec![1; 9],
                program: vec![0x51],
            },
            WireSpentOutput {
                asset_field: vec![10; 33],
                value_field: vec![8; 33],
                program: vec![0x52],
            },
        ],
        executing_leaf: WireTapleaf {
            leaf_version: 0xc4,
            script: vec![0xac],
            control_block: vec![0xc4; 33],
        },
        sighash_profile: WireSighashProfile::AllInputsAllOutputs,
        signer: crate::test_material::PublicTestSignerHandle::First,
    }
}

pub(super) fn script_path_response() -> NativeOperationResponse {
    let subject = script_path_subject();
    NativeOperationResponse {
        case: OperationCaseId {
            operation: OperationStepKind::SignScriptPath,
            step: "sign-leaf".to_owned(),
        },
        script_path_witness: vec![vec![0x55; crate::test_material::SIGNATURE_BYTES]],
        signer_public_key: Some(
            subject
                .signer
                .x_only_public_key()
                .expect("published key resolves"),
        ),
        signed_profile: Some(subject.sighash_profile),
        signing_genesis: Some(crate::protocol::MOCK_EXECUTOR_GENESIS_ID),
        signature_bound_to: Some(subject.finalized_transaction),
        ..operation_response(ObservedOutcomeLayer::Accepted)
    }
}

#[test]
fn public_signer_handles_are_a_closed_position_census() {
    use crate::test_material::PublicTestSignerHandle;
    for (handle, spelling) in [
        (PublicTestSignerHandle::First, "first"),
        (PublicTestSignerHandle::Third, "third"),
    ] {
        assert_eq!(
            serde_json::to_value(handle).expect("handle serializes"),
            spelling
        );
        assert_eq!(
            serde_json::from_value::<PublicTestSignerHandle>(serde_json::json!(spelling))
                .expect("handle reads"),
            handle
        );
    }
    for value in [
        serde_json::json!("second"),
        serde_json::json!("owner"),
        serde_json::json!({"first": 1}),
        serde_json::json!([]),
    ] {
        assert!(serde_json::from_value::<PublicTestSignerHandle>(value).is_err());
    }
}

#[test]
fn public_handles_resolve_to_the_published_appendix_keys() {
    use crate::test_material::PublicTestSignerHandle;
    for (handle, published) in [
        (
            PublicTestSignerHandle::First,
            "dff1d77f2a671c5f36183726db2341be58feae1da2deced843240f7b502ba659",
        ),
        (
            PublicTestSignerHandle::Third,
            "25d1dff95105f5253c4022f628a996ad3a0d95fbf21d468a1b33f8c160d8f517",
        ),
    ] {
        let expected: Vec<_> = published
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| {
                u8::from_str_radix(std::str::from_utf8(pair).expect("ASCII"), 16)
                    .expect("published hex")
            })
            .collect();
        let material = handle.material().expect("published material resolves");
        assert_eq!(material.x_only_public_key().as_slice(), expected);
        assert_eq!(
            handle.x_only_public_key().expect("key resolves"),
            material.x_only_public_key()
        );
    }
}

#[test]
fn script_path_subject_has_exactly_the_public_context_members() {
    let subject = script_path_subject();
    let value = serde_json::to_value(&subject).expect("subject serializes");
    assert_eq!(
        value,
        serde_json::json!({
            "finalized_transaction": [2, 0, 1], "input_index": 1,
            "spent_outputs": [
                {"asset_field": vec![1; 33], "value_field": vec![1; 9], "program": [81]},
                {"asset_field": vec![10; 33], "value_field": vec![8; 33], "program": [82]}
            ],
            "executing_leaf": {"leaf_version": 196, "script": [172], "control_block": vec![196; 33]},
            "sighash_profile": "all_inputs_all_outputs", "signer": "first"
        })
    );
    assert_eq!(
        serde_json::from_value::<crate::protocol::TargetScriptPathSigningSubject>(value)
            .expect("subject reads"),
        subject
    );
}

#[test]
fn script_path_subject_refuses_missing_and_forbidden_members() {
    use crate::protocol::TargetScriptPathSigningSubject;
    let original = serde_json::to_value(script_path_subject()).expect("subject serializes");
    for name in original.as_object().expect("object").keys() {
        let mut value = original.clone();
        value.as_object_mut().expect("object").remove(name);
        assert!(
            serde_json::from_value::<TargetScriptPathSigningSubject>(value).is_err(),
            "{name}"
        );
    }
    for name in [
        "scalar",
        "private_key",
        "digest",
        "genesis",
        "expected_verdict",
        "maturity",
        "state",
        "operator_key",
    ] {
        let mut value = original.clone();
        value[name] = serde_json::json!([1, 2, 3]);
        assert!(
            serde_json::from_value::<TargetScriptPathSigningSubject>(value).is_err(),
            "{name}"
        );
    }
    for (member, extra) in [
        ("executing_leaf", "annex"),
        ("executing_leaf", "codeseparator_pos"),
    ] {
        let mut value = original.clone();
        value[member][extra] = serde_json::json!(0);
        assert!(serde_json::from_value::<TargetScriptPathSigningSubject>(value).is_err());
    }
    let mut value = original;
    value["spent_outputs"][0]["amount"] = serde_json::json!(1);
    assert!(serde_json::from_value::<TargetScriptPathSigningSubject>(value).is_err());
}

#[test]
fn script_path_untagged_arm_is_disjoint_from_every_existing_subject() {
    use crate::protocol::{
        OperationSubject, TargetConfidentialFundingSubject,
        TargetConfidentialSponsorFundingSubject, TargetFundingSubject,
        TargetScriptPathSigningSubject, TargetSponsorFundingSubject, TargetSponsorSigningSubject,
        TargetSubmissionSubject,
    };
    let value = serde_json::to_value(script_path_subject()).expect("subject serializes");
    assert!(serde_json::from_value::<TargetFundingSubject>(value.clone()).is_err());
    assert!(serde_json::from_value::<TargetSubmissionSubject>(value.clone()).is_err());
    assert!(serde_json::from_value::<TargetSponsorFundingSubject>(value.clone()).is_err());
    assert!(serde_json::from_value::<TargetSponsorSigningSubject>(value.clone()).is_err());
    assert!(serde_json::from_value::<TargetConfidentialFundingSubject>(value.clone()).is_err());
    assert!(
        serde_json::from_value::<TargetConfidentialSponsorFundingSubject>(value.clone()).is_err()
    );
    assert!(matches!(
        serde_json::from_value::<OperationSubject>(value),
        Ok(OperationSubject::ScriptPathSigning(_))
    ));
    for other in [
        serde_json::json!({"transaction_bytes": [1]}),
        serde_json::json!({"sponsor_outputs": 1, "amount_per_sponsor_output": 1}),
        serde_json::to_value(super::support::confidential_subject())
            .expect("confidential subject serializes"),
    ] {
        assert!(serde_json::from_value::<OperationSubject>(other.clone()).is_ok());
        assert!(serde_json::from_value::<TargetScriptPathSigningSubject>(other).is_err());
    }
}

#[test]
fn script_path_response_requires_every_new_wire_member_even_on_refusal() {
    for layer in [
        ObservedOutcomeLayer::Accepted,
        ObservedOutcomeLayer::ExecutorInfrastructureFailure,
    ] {
        let mut response = script_path_response();
        response.observed_layer = layer;
        let original = serde_json::to_value(response).expect("response serializes");
        for name in [
            "script_path_witness",
            "signer_public_key",
            "signed_profile",
            "signing_genesis",
        ] {
            let mut value = original.clone();
            value.as_object_mut().expect("object").remove(name);
            assert!(
                serde_json::from_value::<NativeOperationResponse>(value).is_err(),
                "{name}"
            );
        }
    }
}

#[test]
fn script_path_acceptance_owes_one_signature_and_all_bindings() {
    let response = script_path_response();
    let encoded = serde_json::to_string(&response).expect("response serializes");
    let decoded: NativeOperationResponse = serde_json::from_str(&encoded).expect("response reads");
    assert_eq!(decoded, response);
    assert_eq!(decoded.validate_shape(), Ok(()));
    for name in [
        "signer_public_key",
        "signed_profile",
        "signing_genesis",
        "signature_bound_to",
    ] {
        let mut value = serde_json::to_value(&response).expect("response serializes");
        value[name] = serde_json::Value::Null;
        assert_eq!(
            serde_json::from_value::<NativeOperationResponse>(value)
                .expect("nullable member")
                .validate_shape(),
            Err(ResponseShapeDefect::AcceptedOperationOmitsObservation)
        );
    }
    for witness in [
        vec![],
        vec![vec![]],
        vec![vec![1; 63]],
        vec![vec![1; 65]],
        vec![vec![1; 64]; 2],
    ] {
        assert_eq!(
            NativeOperationResponse {
                script_path_witness: witness,
                ..response.clone()
            }
            .validate_shape(),
            Err(ResponseShapeDefect::ScriptPathWitnessMalformed)
        );
    }
}

#[test]
fn script_path_refusals_carry_no_success_observations() {
    let signed = serde_json::to_value(script_path_response()).expect("response serializes");
    for layer in [
        ObservedOutcomeLayer::ExecutorInfrastructureFailure,
        ObservedOutcomeLayer::FixtureConstructionFailure,
        ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
        ObservedOutcomeLayer::ScriptPathRejection,
        ObservedOutcomeLayer::KeyPathRejection,
        ObservedOutcomeLayer::RelayPolicyRejection,
    ] {
        let mut empty = operation_response(layer);
        empty.case.operation = OperationStepKind::SignScriptPath;
        assert_eq!(empty.validate_shape(), Ok(()));
        for name in [
            "script_path_witness",
            "signer_public_key",
            "signed_profile",
            "signing_genesis",
            "signature_bound_to",
        ] {
            let mut value = serde_json::to_value(&empty).expect("response serializes");
            value[name] = signed[name].clone();
            let expected = if layer.is_target_verdict() {
                ResponseShapeDefect::RefusedOperationCarriesObservation
            } else {
                ResponseShapeDefect::InfrastructureResponseCarriesObservation
            };
            assert_eq!(
                serde_json::from_value::<NativeOperationResponse>(value)
                    .expect("member reads")
                    .validate_shape(),
                Err(expected),
                "{layer:?}: {name}"
            );
        }
    }
}

#[test]
fn script_path_acceptance_creates_and_submits_nothing() {
    let response = script_path_response();
    for foreign in [
        NativeOperationResponse {
            issued_asset: Some("asset".to_owned()),
            ..response.clone()
        },
        NativeOperationResponse {
            accepted_txid: Some("txid".to_owned()),
            ..response.clone()
        },
        NativeOperationResponse {
            sponsor_witness: vec![vec![1]],
            ..response
        },
    ] {
        assert_eq!(
            foreign.validate_shape(),
            Err(ResponseShapeDefect::OperationResponseMismatchesStep)
        );
    }
    for kind in [
        OperationStepKind::Fund,
        OperationStepKind::Submit,
        OperationStepKind::FundSponsor,
        OperationStepKind::SignSponsor,
        OperationStepKind::FundConfidential,
        OperationStepKind::FundConfidentialSponsor,
    ] {
        let mut foreign = script_path_response();
        foreign.case.operation = kind;
        assert_eq!(
            foreign.validate_shape(),
            Err(ResponseShapeDefect::OperationResponseMismatchesStep)
        );
    }
}

#[test]
fn unknown_signing_profiles_fail_deserialization_in_both_carriers() {
    let mut subject = serde_json::to_value(script_path_subject()).expect("subject serializes");
    subject["sighash_profile"] = serde_json::json!("single");
    assert!(
        serde_json::from_value::<crate::protocol::TargetScriptPathSigningSubject>(subject).is_err()
    );
    let mut response = serde_json::to_value(script_path_response()).expect("response serializes");
    response["signed_profile"] = serde_json::json!("single");
    assert!(serde_json::from_value::<NativeOperationResponse>(response).is_err());
}

#[test]
fn adapter_schema_and_committed_signer_scalars_match_rust() {
    use crate::test_material::{FIRST_SCALAR, THIRD_SCALAR};
    let adapter = include_str!("../../../../scripts/elements-native-executor.py");
    let schema = adapter
        .lines()
        .find_map(|line| line.strip_prefix("NATIVE_PROTOCOL_SCHEMA = "))
        .expect("adapter schema")
        .parse::<u32>()
        .expect("integer schema");
    assert_eq!(schema, NATIVE_PROTOCOL_SCHEMA);
    for (name, expected) in [
        ("PUBLIC_TEST_FIRST_SCALAR", FIRST_SCALAR),
        ("PUBLIC_TEST_THIRD_SCALAR", THIRD_SCALAR),
    ] {
        let prefix = format!("{name} = bytes([");
        let source = adapter
            .lines()
            .find_map(|line| line.strip_prefix(&prefix))
            .and_then(|line| line.strip_suffix("])"))
            .expect("literal public octets");
        let actual: Vec<u8> = source
            .split(',')
            .map(|octet| octet.trim().parse().expect("octet"))
            .collect();
        assert_eq!(actual, expected);
    }
}

#[test]
fn the_historical_revision_seven_corpus_still_uses_its_own_parser() {
    vectors::run_of_record().expect("the immutable revision-seven corpus still validates");
}

pub(super) fn script_path_binding_mutations()
-> Vec<(NativeOperationResponse, Option<ResponseShapeDefect>)> {
    let positive = script_path_response();
    let mut rows = vec![(positive.clone(), None)];
    for member in ["key", "genesis"] {
        for mutation in ["zero", "first", "middle", "last", "other"] {
            let mut response = positive.clone();
            let bytes = if member == "key" {
                response.signer_public_key.as_mut().expect("key")
            } else {
                response.signing_genesis.as_mut().expect("genesis")
            };
            match mutation {
                "zero" => *bytes = [0; 32],
                "first" => bytes[0] ^= 1,
                "middle" => bytes[16] ^= 1,
                "last" => bytes[31] ^= 1,
                _ => {
                    *bytes = crate::test_material::PublicTestSignerHandle::Third
                        .x_only_public_key()
                        .expect("other published key");
                }
            }
            rows.push((
                response,
                Some(if member == "key" {
                    ResponseShapeDefect::ScriptPathSignerMismatch
                } else {
                    ResponseShapeDefect::ScriptPathGenesisMismatch
                }),
            ));
        }
    }
    for mutation in ["empty", "long", "short", "first", "middle", "last"] {
        let mut response = positive.clone();
        let echo = response.signature_bound_to.as_mut().expect("echo");
        match mutation {
            "empty" => echo.clear(),
            "long" => echo.push(0),
            "short" => {
                echo.pop();
            }
            "first" => echo[0] ^= 1,
            "middle" => echo[1] ^= 1,
            _ => echo[2] ^= 1,
        }
        rows.push((
            response,
            Some(ResponseShapeDefect::ScriptPathTransactionMismatch),
        ));
    }
    rows
}

#[test]
fn signing_binding_bytes_are_shape_valid_but_not_interchangeable() {
    let rows = script_path_binding_mutations();
    assert_eq!(rows.len(), 17);
    for (response, _) in rows {
        assert_eq!(response.validate_shape(), Ok(()));
    }
}

#[test]
fn signing_witness_sweep_extends_the_existing_width_census() {
    let positive = script_path_response();
    for (witness, verdict) in [
        (positive.script_path_witness.clone(), Ok(())),
        (
            vec![vec![1]],
            Err(ResponseShapeDefect::ScriptPathWitnessMalformed),
        ),
        (
            vec![vec![1; 64], vec![]],
            Err(ResponseShapeDefect::ScriptPathWitnessMalformed),
        ),
    ] {
        assert_eq!(
            NativeOperationResponse {
                script_path_witness: witness,
                ..positive.clone()
            }
            .validate_shape(),
            verdict
        );
    }
    for position in [0, 32, 63] {
        let mut response = positive.clone();
        response.script_path_witness[0][position] ^= 1;
        assert_eq!(response.validate_shape(), Ok(()));
    }
}

#[test]
fn signing_fixed_width_members_fail_at_deserialization() {
    for member in ["signer_public_key", "signing_genesis"] {
        for width in [0, 31, 33] {
            let mut value = serde_json::to_value(script_path_response()).expect("response");
            value[member] = serde_json::json!(vec![1; width]);
            assert!(
                serde_json::from_value::<NativeOperationResponse>(value).is_err(),
                "{member}/{width}"
            );
        }
    }
}

fn foreign_signing_observations() -> Vec<(&'static str, serde_json::Value)> {
    vec![
        ("issued_asset", serde_json::json!("asset")),
        (
            "funded_outputs",
            serde_json::json!([{
                "outpoint": {"txid": "coin", "vout": 0},
                "asset": "asset", "amount_satoshis": 1, "script": "51"
            }]),
        ),
        ("accepted_txid", serde_json::json!("txid")),
        (
            "mined_readback",
            serde_json::json!({
                "transaction_id": "txid", "witness_transaction_id": "wtxid",
                "block_hash": "block", "block_height": 1, "raw_transaction": [2]
            }),
        ),
        ("sponsor_witness", serde_json::json!([[1]])),
        (
            "confidential_funded_outputs",
            serde_json::json!([{
                "outpoint": {"txid": "coin", "vout": 0}, "explicit_asset": "asset",
                "value_commitment": [8], "nonce": [2], "script": "51",
                "output_witness_index": 0, "surjection_proof": [], "rangeproof": [1]
            }]),
        ),
    ]
}

#[test]
fn every_script_path_refusal_layer_rejects_foreign_success_data() {
    let mut rows = 0;
    for layer in [
        ObservedOutcomeLayer::ExecutorInfrastructureFailure,
        ObservedOutcomeLayer::FixtureConstructionFailure,
        ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
        ObservedOutcomeLayer::ScriptPathRejection,
        ObservedOutcomeLayer::KeyPathRejection,
        ObservedOutcomeLayer::RelayPolicyRejection,
    ] {
        let mut empty = operation_response(layer);
        empty.case.operation = OperationStepKind::SignScriptPath;
        for (member, data) in foreign_signing_observations() {
            let mut value = serde_json::to_value(&empty).expect("response");
            value[member] = data;
            let response: NativeOperationResponse = serde_json::from_value(value).expect("member");
            let expected = if layer.is_target_verdict() {
                ResponseShapeDefect::OperationResponseMismatchesStep
            } else {
                ResponseShapeDefect::InfrastructureResponseCarriesObservation
            };
            assert_eq!(
                response.validate_shape(),
                Err(expected),
                "{layer:?}/{member}"
            );
            rows += 1;
        }
    }
    assert_eq!(rows, 36);
}

#[test]
fn signing_acceptance_rejects_funding_and_readback_data() {
    for (member, data) in foreign_signing_observations() {
        if ![
            "funded_outputs",
            "mined_readback",
            "confidential_funded_outputs",
        ]
        .contains(&member)
        {
            continue;
        }
        let mut value = serde_json::to_value(script_path_response()).expect("response");
        value[member] = data;
        let response: NativeOperationResponse = serde_json::from_value(value).expect("member");
        assert_eq!(
            response.validate_shape(),
            Err(ResponseShapeDefect::OperationResponseMismatchesStep),
            "{member}"
        );
    }
}

#[test]
fn signing_subject_wrong_kinds_and_integer_edges_fail_deserialization() {
    let original = serde_json::to_value(script_path_subject()).expect("subject");
    for pointer in [
        "/finalized_transaction",
        "/input_index",
        "/spent_outputs",
        "/executing_leaf",
        "/sighash_profile",
        "/signer",
        "/spent_outputs/0/asset_field",
        "/spent_outputs/0/value_field",
        "/spent_outputs/0/program",
        "/executing_leaf/leaf_version",
        "/executing_leaf/script",
        "/executing_leaf/control_block",
    ] {
        let mut value = original.clone();
        *value.pointer_mut(pointer).expect("member") = serde_json::json!(false);
        assert!(
            serde_json::from_value::<crate::protocol::TargetScriptPathSigningSubject>(value)
                .is_err(),
            "{pointer}"
        );
    }
    for index in [
        serde_json::json!(-1),
        serde_json::json!(u64::from(u32::MAX) + 1),
    ] {
        let mut value = original.clone();
        value["input_index"] = index;
        assert!(
            serde_json::from_value::<crate::protocol::TargetScriptPathSigningSubject>(value)
                .is_err()
        );
    }
}

#[test]
fn signing_response_cannot_claim_its_own_evidence_standing() {
    let positive = script_path_response();
    assert_eq!(positive.validate_shape(), Ok(()));
    let mut value = serde_json::to_value(positive).expect("response");
    value["evidence_standing"] = serde_json::json!("native_verified");
    let error = serde_json::from_value::<NativeOperationResponse>(value)
        .expect_err("standing is derived by the consumer");
    assert!(
        error
            .to_string()
            .contains("unknown field `evidence_standing`")
    );
}

#[test]
fn canonical_report_refuses_caller_authored_wall_time() {
    use super::support::{
        development_binding, nonmock_handshake, observed_environment, reviewed_target, subjects_of,
    };
    use crate::executor::{ExecutionTranscript, ExecutorTrust, TranscriptParts};
    let target = reviewed_target();
    let binding = development_binding(&target);
    let fixtures = crate::fixture::canonical_fixture_set(&target, &binding).expect("fixtures");
    let responses = (&fixtures)
        .into_iter()
        .map(|fixture| {
            let answer = NativeExecutionResponse {
                case: fixture.case(),
                ..response(NativeVerdict::InfrastructureError, None, None)
            };
            (answer.case, answer)
        })
        .collect();
    let transcript = ExecutionTranscript::for_tests(TranscriptParts {
        target: &target,
        binding: &binding,
        handshake: nonmock_handshake(),
        environment: observed_environment(),
        trust: ExecutorTrust::Mock,
        requests: subjects_of(&fixtures),
        responses,
    });
    let report = crate::validate::evaluate(
        &target,
        &binding,
        &fixtures,
        &transcript,
        &crate::validate::guide_nine_evidence_plan().expect("plan"),
        &crate::claim::claim_registry().expect("claims"),
    )
    .expect("infrastructure-only report");
    let original = serde_json::to_value(report).expect("report");
    assert!(
        serde_json::from_value::<crate::report::NativeConformanceReport>(original.clone()).is_ok()
    );
    let mut value = original;
    value["wall_time"] = serde_json::json!(1);
    let error = serde_json::from_value::<crate::report::NativeConformanceReport>(value)
        .expect_err("wall time is not canonical evidence");
    assert!(error.to_string().contains("unknown field `wall_time`"));
}

#[test]
fn conservation_ingestion_compares_observed_and_expected_layers() {
    use crate::conservation::{ExpectedOutcomeLayer, canonical_conservation_matrix};
    use crate::conservation_report::{RowVerdict, ingest_conservation_response};
    let row = canonical_conservation_matrix()
        .into_iter()
        .find(|row| row.expected_layer == ExpectedOutcomeLayer::Accepted)
        .expect("an accepted control row");
    for (layer, verdict) in [
        (ObservedOutcomeLayer::Accepted, RowVerdict::Agrees),
        (
            ObservedOutcomeLayer::ScriptPathRejection,
            RowVerdict::Disagrees,
        ),
        (
            ObservedOutcomeLayer::KeyPathRejection,
            RowVerdict::Disagrees,
        ),
        (
            ObservedOutcomeLayer::RelayPolicyRejection,
            RowVerdict::Disagrees,
        ),
        (
            ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
            RowVerdict::Disagrees,
        ),
        (
            ObservedOutcomeLayer::FixtureConstructionFailure,
            RowVerdict::NotTargetEvidence,
        ),
        (
            ObservedOutcomeLayer::ExecutorInfrastructureFailure,
            RowVerdict::NotTargetEvidence,
        ),
    ] {
        let mut response = conservation_response(layer);
        response.case = row.id.clone();
        let outcome = ingest_conservation_response(&row, response).expect("valid response shape");
        assert_eq!(outcome.verdict, verdict, "{layer:?}");
        assert_eq!(outcome.observed_layer, Some(layer));
    }
}
