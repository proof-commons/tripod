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
        confidential_funding: None,
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

    let revision_five = r#"{
        "schema": 5,
        "case": {"operation": "fund", "step": "issue"},
        "observed_layer": "accepted",
        "observed_detail": null,
        "issued_asset": "aa",
        "funded_outputs": [],
        "confidential_funded_outputs": [],
        "mined_readback": null,
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
    let parsed: NativeOperationResponse = serde_json::from_str(revision_five)
        .expect("a revision-5 record without the sponsor members reads");
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
            OperationStepKind::Submit => response.accepted_txid = Some("99".repeat(32)),
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
