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
//! - `G12-R09` — the executor and this crate both declare protocol
//!   revision 3 while carrying conservation responses the Rust type
//!   cannot read.
//! - `G12-R14` — the shape rules leave an infrastructure response's
//!   resource observation unread.
//!
//! - `G12-R05` — CLOSED: the normalization response census is exact in
//!   both directions, and the ingestion that decides it now has a
//!   library seam to be tested through.
//!
//! `G12-R06`'s reproduction lives beside the report it is about, in
//! `lifecycle_report.rs`, where that module's own fixtures build the
//! record. `G12-R03` and the Python lanes (`G12-R04`, `G12-R07`,
//! `G12-R15`) carry source-read dispositions in the register instead:
//! the first sits in `src/bin` with no library seam, and the rest need
//! a live node.

use std::collections::BTreeSet;

use crate::fixture::{NativeCaseGroup, NativeCaseId};
use crate::normalization::canonical_mutation_matrix;
use crate::normalization_report::{
    NormalizationIngestionDefect, ingest_normalization_responses, mutation_wire_spelling,
};
use crate::protocol::{
    ExecutorCapability, NATIVE_PROTOCOL_SCHEMA, NativeConservationResponse,
    NativeExecutionResponse, NativeNormalizationResponse, NativeResourceObservation, NativeVerdict,
    NormalizationCaseId, validate_response_shape,
};
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

/// `G12-R09`: a revision-3 response its own protocol type cannot read.
///
/// Both sides declare [`NATIVE_PROTOCOL_SCHEMA`]. The executor writes an
/// `observed_openings` array on every conservation response — it is the
/// leg the three-way comparison meets at — and
/// [`NativeConservationResponse`] carries `deny_unknown_fields` without
/// that member, so the typed reader refuses the very record the adapter
/// at the same declared revision produces.
///
/// The assertion is the defect: the field's presence alone decides it,
/// as the round trip through the type's own serialization shows. A wave
/// that unifies the schemas under a new revision, or gives the two
/// executors distinct experimental schemas, flips it.
#[test]
fn a_revision_three_conservation_response_is_unreadable_by_its_own_type() {
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
    };

    let mut wire = serde_json::to_value(&response).expect("the response serializes");
    assert!(serde_json::from_value::<NativeConservationResponse>(wire.clone()).is_ok());

    // The one member the adapter always writes and the type never
    // declares.
    wire["observed_openings"] = serde_json::json!([]);
    assert!(
        serde_json::from_value::<NativeConservationResponse>(wire).is_err(),
        "G12-R09: the typed reader is expected to refuse the adapter's own shape \
         while the row is open",
    );
}

/// `G12-R14`: an infrastructure response keeps its interpreter figures.
///
/// The infrastructure arm of the shape rules states that a run which did
/// not happen observed nothing, and then checks the failure class and
/// the two stacks and returns. The resource observation is the one field
/// it never reads, so a response saying the execution never occurred may
/// still carry a peak stack depth, a spent validation budget, and a
/// transaction weight — figures only a run that happened could produce.
///
/// The capability set makes no difference: the arm returns before the
/// advertised-observation check that would otherwise refuse them, so an
/// executor that says it observes no interpreter figure passes too. Both
/// are asserted. The conservation response's own `validate_shape` does
/// make the equivalent check, which is why the row is about the
/// execution and prototype path.
///
/// The assertions are the defect. A wave that rejects every target
/// observation on a non-target outcome flips them.
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
            Ok(()),
            "G12-R14: the shape rules are expected to admit the figures while the row is open",
        );
    }
}
