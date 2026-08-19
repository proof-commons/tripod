//! Guide-12 preflight reproductions owned by this crate.
//!
//! Each test here belongs to a row of the Guide-12 preflight register.
//! While a row is open its test passes by asserting the wrong thing
//! happens, which is what Wave 0 recorded: it reproduces and does not
//! repair. The wave that fixes a row flips that row's assertions, and
//! they then stand as the guarantee that the repair holds. A row marked
//! CLOSED below is one whose test has been flipped.
//!
//! - `G12-R01` — CLOSED: an abbreviated expectation can no longer be
//!   stated, because the width is a type and the expectation's members
//!   are private.
//! - `G12-R09` — CLOSED: both sides declare protocol revision 4, the
//!   conservation response declares the openings the adapter writes,
//!   and the lifecycle step has typed request and response records that
//!   round-trip the adapter's own wire shape.
//! - `G12-R14` — CLOSED: the infrastructure arm refuses every
//!   interpreter figure whatever the executor advertises, and the
//!   normalization response counts its witness sizes among the
//!   observations a run that did not happen may not carry.
//!
//! - `G12-R05` — CLOSED: the normalization response census is exact in
//!   both directions, and the ingestion that decides it now has a
//!   library seam to be tested through.
//!
//! `G12-R06`'s reproduction lives beside the report it is about, in
//! `lifecycle_report.rs`, where that module's own fixtures build the
//! record. `G12-R03`'s lives in `emit_tests.rs`: the row's disposition
//! was a source read because the commands had no library seam, and
//! `emit.rs` is now that seam, so the documents are checked without
//! spawning anything.
//!
//! The Python lanes (`G12-R04`, `G12-R07`, `G12-R15`) are repaired and
//! recorded in the register rather than here, because this crate cannot
//! host a test for the other side of the protocol. `G12-R07` was
//! discharged by recomputation over the whole script-error class table
//! and `G12-R15` by a behavioural probe of the supervision module;
//! `G12-R04`'s runtime half stays blocked on a live node, which is
//! stated as a blocker rather than stood in for.

use std::collections::BTreeSet;

use crate::fixture::{NativeCaseGroup, NativeCaseId};
use crate::lifecycle::LifecycleOutcome;
use crate::normalization::canonical_mutation_matrix;
use crate::normalization_report::{
    NormalizationIngestionDefect, ingest_normalization_responses, mutation_wire_spelling,
};
use crate::protocol::{
    ExecutorCapability, LifecycleCheck, LifecycleStepRole, LifecycleSubject,
    NATIVE_PROTOCOL_SCHEMA, NativeConservationResponse, NativeExecutionResponse,
    NativeLifecycleRequest, NativeLifecycleResponse, NativeNormalizationResponse,
    NativePrototypeResponse, NativeResourceObservation, NativeVerdict, NormalizationCaseId,
    ResponseShapeDefect, validate_response_shape,
};
use crate::prototype::{PrototypeCaseId, PrototypeRelation};
use crate::provenance::{
    ExpectedExecutorProvenance, FULL_REVISION_WIDTH, FullRevisionId, MINIMUM_REVISION_PREFIX_WIDTH,
    ProvenanceSyntaxDefect, validate_executor_provenance,
};
use crate::report::{ExecutorDeclaration, ExecutorProvenance};

use super::support::{
    TEST_BINARY_REVISION, TEST_INTENDED_TIP, TEST_LOCAL_TOPIC, TEST_UPSTREAM_BASE,
    nonmock_handshake,
};

/// The reported provenance of a well-behaved nonmock run.
fn reported_provenance() -> ExecutorProvenance {
    let handshake = nonmock_handshake();
    ExecutorProvenance {
        protocol_schema: handshake.protocol_schema,
        adapter_name: handshake.adapter_name,
        adapter_version: handshake.adapter_version,
        framework_revision: handshake.framework_revision,
        node_name: handshake.node_name,
        node_version: handshake.node_version,
        binary_reported_revision: handshake.binary_reported_revision,
        intended_executed_tip: handshake.intended_executed_tip,
        upstream_base: handshake.upstream_base,
        included_local_topics: handshake.included_local_topics,
        supported_domains: handshake.supported_domains,
        supported_leaf_versions: handshake.supported_leaf_versions,
        capabilities: handshake.capabilities,
        declaration: ExecutorDeclaration::ReviewedNonMock,
    }
}

/// `G12-R01`: an abbreviated expectation cannot reach the gate.
///
/// The row was that [`ExpectedExecutorProvenance`] refused an
/// abbreviation in its constructor and then handed out three public
/// fields, so a caller assembled the same type by literal and every
/// comparison in the rule ran between seven digits — a run naming a
/// different forty-digit object with the same first seven passed.
///
/// The repair is that the width is a type. [`FullRevisionId`] has one
/// validating constructor, the expectation's members are private and
/// hold that type, and the gate re-asserts the width before comparing
/// anything. This test states what remains reachable through the public
/// API, which is only the refusal.
///
/// The struct literal the row relied on is now a compile error rather
/// than a runtime one, so it cannot be written here at all. The public
/// surface that makes it impossible is checked separately, by the
/// accessor-only expectation test in `provenance_tests`.
#[test]
fn an_abbreviated_expectation_is_refused_before_it_can_be_expected() {
    let abbreviation = &TEST_INTENDED_TIP[..MINIMUM_REVISION_PREFIX_WIDTH];
    assert_eq!(abbreviation, TEST_BINARY_REVISION);

    // Every public route to an expectation refuses the abbreviation,
    // naming the width rather than some generic syntax defect.
    assert_eq!(
        ExpectedExecutorProvenance::new(abbreviation, TEST_UPSTREAM_BASE, [TEST_LOCAL_TOPIC]),
        Err(ProvenanceSyntaxDefect::RevisionNotFullWidth),
    );
    assert_eq!(
        ExpectedExecutorProvenance::new(TEST_INTENDED_TIP, abbreviation, [TEST_LOCAL_TOPIC]),
        Err(ProvenanceSyntaxDefect::RevisionNotFullWidth),
    );
    assert_eq!(
        FullRevisionId::new(abbreviation),
        Err(ProvenanceSyntaxDefect::RevisionNotFullWidth),
    );

    // The full expectation is the only one that exists, and the gate
    // accepts the honest run against it: the binary's abbreviation is
    // compared as a prefix of the full tip, which is the rule.
    let expected =
        ExpectedExecutorProvenance::new(TEST_INTENDED_TIP, TEST_UPSTREAM_BASE, [TEST_LOCAL_TOPIC])
            .expect("the full identifiers state an expectation");
    assert_eq!(expected.intended_tip().as_str().len(), FULL_REVISION_WIDTH);

    let validated = validate_executor_provenance(&reported_provenance(), &expected)
        .expect("the honest run matches its expectation");
    // What the gate carries forward is the full identifier, never the
    // abbreviation the binary reported.
    assert_eq!(validated.intended_tip().as_str(), TEST_INTENDED_TIP);
    assert_eq!(validated.binary_reported_revision().as_str(), abbreviation);

    // And a run declaring the abbreviation as its intended tip is now a
    // mismatch, because the expectation it is compared against is full.
    let mut reported = reported_provenance();
    reported.intended_executed_tip = Some(abbreviation.to_owned());
    assert!(
        validate_executor_provenance(&reported, &expected).is_err(),
        "G12-R01: a prefix declared as the intended tip is not the expected tip",
    );
}

/// `G12-R05`: the response census is exact in both directions.
///
/// The row's disposition was a source read, because the ingestion sat
/// in a binary with no library seam: responses were indexed into a map
/// keyed by row name, so a second answer for one row replaced the first
/// silently, and the report loop walked the matrix rather than the
/// census, so a response naming a row the matrix does not carry was
/// dropped without a word.
///
/// The ingestion now lives in the module that owns the report, which is
/// what makes this test possible, and it refuses rather than repairs:
/// a report is a statement about the canonical matrix, so anything but
/// an exact census is a run the report cannot describe.
#[test]
fn a_normalization_run_answers_the_matrix_exactly_once_each() {
    let matrix = canonical_mutation_matrix();
    let complete = || {
        matrix
            .iter()
            .map(|row| normalization_response(&mutation_wire_spelling(row.mutation)))
            .collect::<Vec<_>>()
    };

    // The honest census is accepted, and is the whole matrix.
    let answered = ingest_normalization_responses(complete()).expect("a complete census");
    assert_eq!(answered.len(), matrix.len());

    // A duplicated row is refused rather than resolved by arrival
    // order, which is the half the map's own insert used to swallow.
    let first = mutation_wire_spelling(matrix[0].mutation);
    let mut duplicated = complete();
    duplicated.push(normalization_response(&first));
    assert_eq!(
        ingest_normalization_responses(duplicated),
        Err(NormalizationIngestionDefect::DuplicateResponse { row: first.clone() }),
    );

    // A row the matrix does not carry is named rather than dropped,
    // which is the half the report loop used to walk straight past.
    let mut unexpected = complete();
    unexpected.push(normalization_response("not-a-canonical-row"));
    assert_eq!(
        ingest_normalization_responses(unexpected),
        Err(NormalizationIngestionDefect::UnexpectedRow {
            row: "not-a-canonical-row".to_owned(),
        }),
    );

    // And a missing row is still a missing row.
    let short = complete().into_iter().skip(1).collect::<Vec<_>>();
    assert_eq!(
        ingest_normalization_responses(short),
        Err(NormalizationIngestionDefect::UnansweredRow { row: first }),
    );
}

/// One well-formed response answering the named row.
fn normalization_response(row: &str) -> NativeNormalizationResponse {
    NativeNormalizationResponse {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: NormalizationCaseId {
            normalization: row.to_owned(),
        },
        observed_layer: crate::protocol::ObservedOutcomeLayer::Accepted,
        observed_detail: None,
        claimed_outputs: Vec::new(),
        observed_outputs: Vec::new(),
        authorization_profile: None,
        observed_witness_sizes: Vec::new(),
        transaction_bytes: None,
    }
}

/// `G12-R09`: the conservation response its own protocol type can read.
///
/// Both sides declare [`NATIVE_PROTOCOL_SCHEMA`]. Under revision 3 that
/// declaration was false: the executor wrote an `observed_openings`
/// array on every conservation response — it is the leg the three-way
/// comparison meets at — and [`NativeConservationResponse`] carried
/// `deny_unknown_fields` without that member, so the typed reader
/// refused the very record the adapter at the same declared revision
/// produced.
///
/// Revision 4 states the union both sides were implementing, and the
/// assertion is now the guarantee. The record the adapter writes parses,
/// the openings survive the round trip rather than being tolerated and
/// dropped, and a member neither side declares is still refused — a
/// revision that read anything offered to it would have closed this row
/// by removing the property that makes a revision mean something.
#[test]
fn a_conservation_response_round_trips_through_its_own_type() {
    let response = NativeConservationResponse {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: crate::conservation::ConservationRowId {
            ordinal: 1,
            name: "explicit-in-explicit-out".to_owned(),
        },
        observed_layer: crate::protocol::ObservedOutcomeLayer::Accepted,
        observed_detail: None,
        transaction_bytes: None,
        observed_value_commitments: Vec::new(),
        observed_asset_commitments: Vec::new(),
        observed_openings: vec![crate::protocol::ConservationOpening {
            vout: 1,
            amount_satoshis: 100_000,
            asset: "aa".repeat(32),
            amount_blinder: "bb".repeat(32),
            asset_blinder: "cc".repeat(32),
        }],
    };

    // The adapter's own shape, openings included, is what the type
    // reads — and it reads back as the same value, so the leg of the
    // comparison is carried rather than silently discarded.
    let wire = serde_json::to_value(&response).expect("the response serializes");
    assert!(
        wire.get("observed_openings").is_some(),
        "the member the adapter always writes is a declared member",
    );
    let parsed = serde_json::from_value::<NativeConservationResponse>(wire.clone())
        .expect("G12-R09: the typed reader reads the adapter's own shape");
    assert_eq!(parsed, response, "the openings survive the round trip");

    // Strictness is intact: unknown members are still refused.
    let mut unknown = wire;
    unknown["observed_something_else"] = serde_json::json!([]);
    assert!(
        serde_json::from_value::<NativeConservationResponse>(unknown).is_err(),
        "a member no revision declares is refused",
    );
}

/// `G12-R09`: openings cannot ride on a run that never happened.
///
/// The adjudication this row required. An opening is read back out of a
/// transaction the node created and confirmed, by looking up the coins
/// that transaction made. A response whose layer says the execution
/// never occurred describes no such transaction, so a blinding factor
/// beside it is a value with no possible provenance — and §7.4's
/// three-way comparison would be resting on it.
///
/// So the openings join the infrastructure refusal that already holds
/// the bytes and the commitments, rather than being admitted as an
/// unusually detailed failure report.
#[test]
fn an_infrastructure_conservation_response_carries_no_openings() {
    let response = NativeConservationResponse {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: crate::conservation::ConservationRowId {
            ordinal: 1,
            name: "explicit-in-explicit-out".to_owned(),
        },
        observed_layer: crate::protocol::ObservedOutcomeLayer::ExecutorInfrastructureFailure,
        observed_detail: Some("the adapter reached no node".to_owned()),
        transaction_bytes: None,
        observed_value_commitments: Vec::new(),
        observed_asset_commitments: Vec::new(),
        observed_openings: Vec::new(),
    };
    assert!(
        response.validate_shape().is_ok(),
        "a run that did not happen, reporting nothing, is well formed",
    );

    let contradictory = NativeConservationResponse {
        observed_openings: vec![crate::protocol::ConservationOpening {
            vout: 0,
            amount_satoshis: 1,
            asset: "aa".repeat(32),
            amount_blinder: "bb".repeat(32),
            asset_blinder: "cc".repeat(32),
        }],
        ..response
    };
    assert_eq!(
        contradictory.validate_shape(),
        Err(crate::protocol::ResponseShapeDefect::InfrastructureResponseCarriesObservation),
        "G12-R09: an opening is an observation, and a run that did not happen made none",
    );
}

/// `G12-R14`: an infrastructure response carries no interpreter figures.
///
/// The infrastructure arm of the shape rules states that a run which did
/// not happen observed nothing, and it used to check the failure class
/// and the two stacks and return. The resource observation was the one
/// field it never read, so a response saying the execution never
/// occurred could still carry a peak stack depth, a spent validation
/// budget, and a transaction weight — figures only a run that happened
/// could produce.
///
/// The capability set makes no difference, and that is the point of
/// checking both here. The arm returns before the
/// advertised-observation rule, so leaving the refusal to that rule
/// would have let an executor that advertises resource observation
/// report figures for a run it never made. The arm now refuses them
/// itself.
///
/// Where the line falls is `observes_interpreter`'s to say rather than
/// this arm's: the script's size and the initial stack's depth are the
/// fixture's own, restated by every executor, and are not observations
/// of anything. They stay legal here, which the last assertion pins.
#[test]
fn an_infrastructure_response_may_still_carry_interpreter_figures() {
    let observed = NativeResourceObservation {
        script_bytes: 33,
        initial_stack_items: 1,
        peak_stack_items: Some(4),
        peak_altstack_items: Some(0),
        maximum_element_bytes: Some(32),
        validation_budget_used: Some(50),
        transaction_weight: Some(400),
    };
    assert!(observed.observes_interpreter());

    let response = NativeExecutionResponse {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: NativeCaseId::new(NativeCaseGroup::Arithmetic, None, 1),
        verdict: NativeVerdict::InfrastructureError,
        final_stack: None,
        final_altstack: None,
        observed_failure: None,
        resources: observed,
    };

    for capabilities in [
        BTreeSet::new(),
        BTreeSet::from([ExecutorCapability::ResourceObservation]),
    ] {
        assert_eq!(
            validate_response_shape(&response, &capabilities),
            Err(ResponseShapeDefect::InfrastructureResponseCarriesObservation),
            "G12-R14: a run that did not happen may report no interpreter figure, \
             whatever the executor advertises",
        );

        // The same rule, reached through the prototype path, which
        // shares the shape rules rather than restating them.
        let prototype = NativePrototypeResponse {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case: PrototypeCaseId {
                relation: PrototypeRelation::WideFloorRelation,
                name: "shape".to_owned(),
            },
            verdict: NativeVerdict::InfrastructureError,
            final_stack: None,
            final_altstack: None,
            observed_failure: None,
            resources: observed,
        };
        assert_eq!(
            prototype.validate_shape(&capabilities),
            Err(ResponseShapeDefect::InfrastructureResponseCarriesObservation),
        );
    }

    // The fixture's own figures are not observations and stay legal: a
    // response that restates the script it was handed still says the run
    // never happened.
    let restated = NativeExecutionResponse {
        resources: NativeResourceObservation {
            script_bytes: 33,
            initial_stack_items: 1,
            ..NativeResourceObservation::default()
        },
        ..response
    };
    assert!(!restated.resources.observes_interpreter());
    assert_eq!(validate_response_shape(&restated, &BTreeSet::new()), Ok(()));
}

/// `G12-R14`, the normalization half: witness sizes are an observation.
///
/// The normalization response already refused a transaction, observed
/// outputs, and an authorization profile on a run that did not happen.
/// The witness sizes were left out, although they are read from the very
/// transaction the response may not claim to have built, and although
/// they are the evidence the authorization profile rests on — so the
/// profile could be refused while its own support was admitted beside
/// it.
///
/// The rule is enforced rather than merely stated: every response
/// reaching `ingest_normalization_responses` is shape-checked before it
/// is filed, which the second half asserts by ingesting one.
#[test]
fn a_normalization_run_that_did_not_happen_reports_no_witness_sizes() {
    let row = mutation_wire_spelling(
        canonical_mutation_matrix()
            .first()
            .expect("the matrix carries at least one row")
            .mutation,
    );

    let mut response = normalization_response(&row);
    response.observed_layer = crate::protocol::ObservedOutcomeLayer::ExecutorInfrastructureFailure;
    assert_eq!(response.validate_shape(), Ok(()));

    response.observed_witness_sizes = vec![vec![64]];
    assert_eq!(
        response.validate_shape(),
        Err(ResponseShapeDefect::InfrastructureResponseCarriesObservation),
    );

    // The same refusal, reached the way a run reaches it.
    let responses: Vec<_> = canonical_mutation_matrix()
        .into_iter()
        .map(|entry| {
            let spelling = mutation_wire_spelling(entry.mutation);
            if spelling == row {
                response.clone()
            } else {
                normalization_response(&spelling)
            }
        })
        .collect();
    assert_eq!(
        ingest_normalization_responses(responses),
        Err(NormalizationIngestionDefect::SelfContradictoryResponse {
            row,
            defect: ResponseShapeDefect::InfrastructureResponseCarriesObservation,
        }),
    );
}

/// `G12-R09`: the adapter's lifecycle answer, read as a typed record.
///
/// The lifecycle step was the one workload with no protocol type on this
/// side at all: the exchange was read out of an untyped value tree, so
/// the schema was whatever the adapter happened to write that day and
/// nothing could disagree with it. This is the wire shape
/// `answer_lifecycle_step` produces for a verifying process, written out
/// literally rather than built from the type, so that the assertion is
/// about the adapter's record and not about this crate's own
/// serialization agreeing with itself.
#[test]
fn a_verify_lifecycle_answer_is_read_by_its_protocol_type() {
    let wire = serde_json::json!({
        "schema": NATIVE_PROTOCOL_SCHEMA,
        "case": {"lifecycle": "verify"},
        "outcome": "verified",
        "handoff": null,
        "authorization_profile": null,
        "observed_witness_sizes": [],
        "observed_outputs": [],
        "checks": [
            {
                "check": "chain_context_genesis",
                "expected": "aa",
                "observed": "aa",
                "agrees": true,
            },
            {
                "check": "owner_object_spendable_by_this_process",
                "expected": "False",
                "observed": "False",
                "agrees": true,
            },
        ],
        "spend": {
            "txid": "dd",
            "amount": 100_000_u64,
            "destination": "an-address",
            "destination_script": "51",
            "consumed_outpoint": {"txid": "ee", "vout": 0},
            "consumed_owner_object": false,
        },
        "superseded_by": null,
        "supersede_failure": null,
        "detail": null,
    });

    let response: NativeLifecycleResponse = serde_json::from_value(wire.clone())
        .expect("G12-R09: the adapter's lifecycle answer is a record this side declares");
    assert_eq!(response.outcome, LifecycleOutcome::Verified);
    assert_eq!(response.checks.len(), 2);
    assert!(response.validate_shape().is_ok());

    // Both directions, as the revision requires: what this side writes
    // is what the adapter reads.
    let round_tripped = serde_json::to_value(&response).expect("the response serializes");
    assert_eq!(round_tripped, wire, "the record survives both directions");
}

/// `G12-R09`: a lifecycle step that did not run observed nothing.
///
/// The lifecycle spelling of the rule the other response shapes keep. A
/// handoff names a transaction that reached a block, a check reports
/// what the chain said, and a spend is a transaction that was confirmed;
/// a step whose outcome says no process reached the chain produced none
/// of them. The detail is exempt, because a failure is entitled to a
/// reason.
#[test]
fn an_infrastructure_lifecycle_answer_carries_no_observation() {
    let refused = serde_json::json!({
        "schema": NATIVE_PROTOCOL_SCHEMA,
        "case": {"lifecycle": "construct"},
        "outcome": "executor_infrastructure_failure",
        "handoff": null,
        "authorization_profile": null,
        "observed_witness_sizes": [],
        "observed_outputs": [],
        "checks": [],
        "spend": null,
        "superseded_by": null,
        "supersede_failure": null,
        "detail": "the adapter reached no node",
    });
    let response: NativeLifecycleResponse =
        serde_json::from_value(refused).expect("the refusal is a record this side declares");
    assert!(
        response.validate_shape().is_ok(),
        "a step that did not run, reporting nothing but its reason, is well formed",
    );

    let contradictory = NativeLifecycleResponse {
        checks: vec![LifecycleCheck {
            check: "chain_context_genesis".to_owned(),
            expected: "aa".to_owned(),
            observed: "aa".to_owned(),
            agrees: true,
        }],
        ..response
    };
    assert_eq!(
        contradictory.validate_shape(),
        Err(ResponseShapeDefect::InfrastructureResponseCarriesObservation),
        "G12-R09: a check is an observation, and a step that did not run made none",
    );
}

/// `G12-R09`: the lifecycle request the runner sends is a typed record.
///
/// The role lives in the case identity and the subject shape follows
/// from it, so the two cannot disagree. The subject is written out as
/// the runner sends it.
#[test]
fn a_lifecycle_request_is_read_by_its_protocol_type() {
    let verify = serde_json::json!({
        "schema": NATIVE_PROTOCOL_SCHEMA,
        "case": {"lifecycle": "verify"},
        "subject": {"handoff": canonical_handoff_wire()},
    });
    let request: NativeLifecycleRequest =
        serde_json::from_value(verify).expect("G12-R09: the verify request is a declared record");
    assert_eq!(request.case.lifecycle, LifecycleStepRole::Verify);
    assert!(
        matches!(request.subject, LifecycleSubject::Verify(_)),
        "the subject is read as the role's own shape",
    );

    // A subject member neither side declares is refused rather than
    // ignored, which is the property that makes the revision mean
    // something.
    let unknown = serde_json::json!({
        "schema": NATIVE_PROTOCOL_SCHEMA,
        "case": {"lifecycle": "verify"},
        "subject": {"handoff": canonical_handoff_wire(), "owner_private_key": "00"},
    });
    assert!(
        serde_json::from_value::<NativeLifecycleRequest>(unknown).is_err(),
        "an undeclared subject member is refused",
    );
}

/// One published record, in the adapter's own wire spelling.
fn canonical_handoff_wire() -> serde_json::Value {
    serde_json::json!({
        "schema": crate::lifecycle::HANDOFF_SCHEMA,
        "chain_name": "elementsregtest",
        "network_id": "aa",
        "genesis_id": "bb",
        "txid": "cc",
        "output_index": 0,
        "block_hash": "dd",
        "block_height": 101,
        "raw_transaction": "00",
        "claimed_explicit_amount": 100_000_u64,
        "claimed_explicit_asset": "ee",
        "claimed_owner_address": "an-address",
    })
}
