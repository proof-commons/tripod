//! Guide-12 preflight reproductions owned by this crate.
//!
//! Every test here demonstrates a row of the Guide-12 preflight register
//! by *passing* while the defect is present: it asserts the wrong thing
//! happens and names the row it belongs to. Wave 0 reproduces and does
//! not repair, so the wave that fixes a row flips that row's assertions,
//! which then stand as the guarantee that the repair holds.
//!
//! - `G12-R01` — an abbreviated expectation reaches the provenance gate,
//!   because the expectation type's fields are public.
//! - `G12-R09` — the executor and this crate both declare protocol
//!   revision 3 while carrying conservation responses the Rust type
//!   cannot read.
//! - `G12-R14` — the shape rules leave an infrastructure response's
//!   resource observation unread.
//!
//! `G12-R06`'s reproduction lives beside the report it is about, in
//! `lifecycle_report.rs`, where that module's own fixtures build the
//! record. The rows whose subjects are the shipped binaries
//! (`G12-R03`, `G12-R05`) and the Python lanes (`G12-R04`, `G12-R07`,
//! `G12-R15`) carry source-read dispositions in the register instead:
//! the first two sit in `src/bin` with no library seam, and the rest
//! need a live node.

use std::collections::BTreeSet;

use crate::fixture::{NativeCaseGroup, NativeCaseId};
use crate::protocol::{
    ExecutorCapability, NATIVE_PROTOCOL_SCHEMA, NativeConservationResponse,
    NativeExecutionResponse, NativeResourceObservation, NativeVerdict, validate_response_shape,
};
use crate::provenance::{
    ExpectedExecutorProvenance, MINIMUM_REVISION_PREFIX_WIDTH, RevisionId,
    validate_executor_provenance,
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

/// `G12-R01`: an abbreviated expectation reaches the gate.
///
/// [`ExpectedExecutorProvenance::new`] refuses an abbreviation, and says
/// why: an expectation stated as a prefix compares a prefix against a
/// prefix, which is a weaker statement than the type exists to make. The
/// three fields are public, though, so a caller assembles the same type
/// by literal without passing through that constructor, and
/// [`RevisionId::new`] hands out the seven-digit value it needs.
///
/// What the gate then does is the row. Both equality checks compare the
/// reported abbreviation with the expected one, and the binary's own
/// revision is checked with `matches_full` against the *expectation* —
/// so with a seven-digit expectation, every comparison in the rule is
/// between seven digits, and a run naming a different forty-digit object
/// with the same first seven passes.
///
/// The assertion is the defect. A wave that makes the fields private, or
/// that revalidates the expected width at the gate, flips it.
#[test]
fn abbreviated_expectation_bypasses_the_full_width_constructor() {
    let abbreviation = &TEST_INTENDED_TIP[..MINIMUM_REVISION_PREFIX_WIDTH];
    assert_eq!(abbreviation, TEST_BINARY_REVISION);

    // The constructor refuses it.
    assert!(
        ExpectedExecutorProvenance::new(abbreviation, TEST_UPSTREAM_BASE, [TEST_LOCAL_TOPIC])
            .is_err()
    );

    // The literal does not.
    let expected = ExpectedExecutorProvenance {
        intended_tip: RevisionId::new(abbreviation).expect("an abbreviation is admitted syntax"),
        upstream_base: RevisionId::full(TEST_UPSTREAM_BASE).expect("a full identifier"),
        included_local_topics: BTreeSet::from([crate::provenance::TopicName::new(
            TEST_LOCAL_TOPIC,
        )
        .expect("an admitted topic")]),
    };

    let mut reported = reported_provenance();
    reported.intended_executed_tip = Some(abbreviation.to_owned());

    let validated = validate_executor_provenance(&reported, &expected).expect(
        "G12-R01: the gate is expected to accept a prefix expectation while the row is open",
    );
    assert_eq!(validated.intended_tip().as_str(), abbreviation);
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
