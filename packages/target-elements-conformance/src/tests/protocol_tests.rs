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
fn this_harness_speaks_schema_four_and_no_earlier_one() {
    // Stated as a value rather than left implicit. Schema 4 declares the
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
    assert_eq!(NATIVE_PROTOCOL_SCHEMA, 4);
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

/// The four operation subjects, and the one thing that could go wrong.
///
/// `OperationSubject` is untagged, so a subject is recognized by its
/// members alone. Four shapes now share that discrimination where two
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
    ];
    let kinds = [
        OperationStepKind::Fund,
        OperationStepKind::Submit,
        OperationStepKind::FundSponsor,
        OperationStepKind::SignSponsor,
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

/// An executor that never heard of the sponsor steps still parses.
///
/// This is the whole justification for leaving the revision where it is:
/// the two added response members are defaulted, so a record written by
/// an executor that predates them is read rather than refused.
#[test]
fn a_response_without_the_sponsor_members_still_parses() {
    use crate::protocol::NativeOperationResponse;

    let written = r#"{
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
    let parsed: NativeOperationResponse =
        serde_json::from_str(written).expect("a pre-sponsor response still reads");
    assert!(
        parsed.sponsor_witness.is_empty(),
        "a revision-4 response defaults to no sponsor witness"
    );
    assert_eq!(parsed.signature_bound_to, None);
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
        accepted_txid: None,
        sponsor_witness: vec![vec![0x30], vec![0x02]],
        signature_bound_to: Some(vec![0x02]),
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
        accepted_txid: None,
        sponsor_witness: stack,
        signature_bound_to: bound,
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
