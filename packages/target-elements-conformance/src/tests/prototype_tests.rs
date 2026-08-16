//! What the compound-prototype fixture language refuses.
//!
//! Every test here is a way a fixture could be satisfied by a
//! transaction other than the one it means
//! (Guide-10 `rule:guide10:fixture-validation`).

use std::collections::BTreeSet;

use target_elements::{LeafVersion, ReviewedElementsTapscriptDefinition};

use crate::claim::ClaimRequirement;
use crate::constructor::internal_key::UNSPENDABLE_INTERNAL_KEY;
use crate::constructor::tree::{
    ConstructionDefect, FixtureTapTree, TreeDefect, construct, control_block_of_version_byte,
    leaf_hash_of_version_byte,
};
use crate::fixture::{ExpectedResourceObservation, PrimitiveFixture, ResourceExpectation};
use crate::protocol::{ExecutorCapability, NATIVE_PROTOCOL_SCHEMA, NativeExecutionRequest};
use crate::prototype::{
    CompoundPrototypeFixture, ExpectedPrototypeOutcome, OutputRole, PrototypeCaseId,
    PrototypeClaim, PrototypeConstruction, PrototypeFixtureDefect, PrototypeOutput,
    PrototypeRelation,
};

/// The reviewed contract these fixtures are stated against.
fn target() -> ReviewedElementsTapscriptDefinition {
    crate::tests::support::reviewed_target()
}

/// The operation script a constructor fixture executes.
const OPERATION: &[u8] = b"operation";

/// The metadata leaf's script.
const METADATA: &[u8] = b"metadata";

/// A resource expectation that compares nothing.
///
/// A constructor fixture states no exact figure yet: what one costs is
/// a measurement a later wave takes, and an invented figure would fail
/// an honest executor over a number no contract states.
const fn recorded_only() -> ExpectedResourceObservation {
    ExpectedResourceObservation {
        script_bytes: ResourceExpectation::RecordedOnly,
        initial_stack_items: ResourceExpectation::RecordedOnly,
        peak_stack_items: ResourceExpectation::RecordedOnly,
        peak_altstack_items: ResourceExpectation::RecordedOnly,
        maximum_element_bytes: ResourceExpectation::RecordedOnly,
        validation_budget_used: ResourceExpectation::RecordedOnly,
        transaction_weight: ResourceExpectation::RecordedOnly,
    }
}

/// One fixture from the primitive census, for the protocol tests.
fn any_primitive_fixture() -> PrimitiveFixture {
    let target = target();
    let binding = crate::tests::support::development_binding(&target);
    crate::census::canonical_census(&target, &binding)
        .expect("the canonical census is coherent")
        .iter()
        .next()
        .expect("the census is not empty")
        .clone()
}

/// One coherent constructor fixture.
fn coherent(target: &ReviewedElementsTapscriptDefinition) -> CompoundPrototypeFixture {
    let executing = FixtureTapTree::leaf(OPERATION.to_vec());
    let tree = FixtureTapTree::branch(executing.clone(), FixtureTapTree::leaf(METADATA.to_vec()));
    let output = construct(&UNSPENDABLE_INTERNAL_KEY, &tree, &executing)
        .expect("the published key and this tree construct");

    CompoundPrototypeFixture {
        case: PrototypeCaseId {
            relation: PrototypeRelation::MetadataConstructorContinuity,
            name: "predecessor_program".to_owned(),
        },
        claims: BTreeSet::from([PrototypeClaim::PredecessorProgramObserved]),
        target_contract_version: target.definition().version().get(),
        script: OPERATION.to_vec(),
        initial_stack: Vec::new(),
        construction: PrototypeConstruction {
            internal_key: UNSPENDABLE_INTERNAL_KEY,
            tree,
            executing_leaf: executing,
            control: Some(output.control_block().to_vec()),
            predecessor_program: output.output_program().to_vec(),
            outputs: vec![PrototypeOutput {
                role: OutputRole::Successor,
                program: output.output_program().to_vec(),
            }],
        },
        expected: ExpectedPrototypeOutcome::Accepted,
        expected_resources: recorded_only(),
    }
}

#[test]
fn a_coherent_constructor_fixture_validates() {
    let target = target();
    assert_eq!(coherent(&target).defect(&target), None);
    assert!(coherent(&target).is_coherent(&target));
    assert_eq!(
        coherent(&target).reviewed_leaf_version(),
        Some(LeafVersion::TAPSCRIPT)
    );
}

#[test]
fn a_fixture_whose_tree_omits_its_executing_leaf_is_refused() {
    // The sharpest of the rules: with no determined control path an
    // executor could authenticate some other leaf and the fixture would
    // have no way to notice.
    let target = target();
    let mut fixture = coherent(&target);
    fixture.script = b"elsewhere".to_vec();
    fixture.construction.executing_leaf = FixtureTapTree::leaf(b"elsewhere".to_vec());
    fixture.construction.control = None;

    assert_eq!(
        fixture.defect(&target),
        Some(PrototypeFixtureDefect::Tree(
            TreeDefect::ExecutingLeafAbsent
        ))
    );
}

#[test]
fn a_fixture_whose_tree_repeats_its_executing_leaf_is_refused() {
    let target = target();
    let mut fixture = coherent(&target);
    let executing = FixtureTapTree::leaf(OPERATION.to_vec());
    fixture.construction.tree = FixtureTapTree::branch(executing.clone(), executing);
    fixture.construction.control = None;

    assert_eq!(
        fixture.defect(&target),
        Some(PrototypeFixtureDefect::Tree(
            TreeDefect::ExecutingLeafRepeated
        ))
    );
}

#[test]
fn a_fixture_whose_script_is_not_its_executing_leaf_is_refused() {
    // The script the fixture hands the executor and the script the tree
    // commits to must be one script. Two would mean the executor runs
    // one and the tree authenticates the other.
    let target = target();
    let mut fixture = coherent(&target);
    fixture.script = b"something else".to_vec();

    assert_eq!(
        fixture.defect(&target),
        Some(PrototypeFixtureDefect::ExecutingLeafScriptMismatch)
    );
}

#[test]
fn a_fixture_at_an_unreviewed_leaf_version_is_refused() {
    let target = target();
    let mut fixture = coherent(&target);
    let executing = FixtureTapTree::leaf_of_version(0xc0, OPERATION.to_vec());
    fixture.construction.tree =
        FixtureTapTree::branch(executing.clone(), FixtureTapTree::leaf(METADATA.to_vec()));
    fixture.construction.executing_leaf = executing;
    fixture.construction.control = None;

    assert_eq!(
        fixture.defect(&target),
        Some(PrototypeFixtureDefect::ExecutingLeafVersionUnreviewed { stated: 0xc0 })
    );
}

#[test]
fn a_stated_control_block_must_be_the_one_the_tree_determines() {
    let target = target();
    let mut fixture = coherent(&target);

    // A path node from the wrong tree.
    let mut wrong = fixture
        .construction
        .control
        .clone()
        .expect("the coherent fixture states one");
    let last = wrong.len() - 1;
    wrong[last] ^= 0xff;
    fixture.construction.control = Some(wrong);
    assert_eq!(
        fixture.defect(&target),
        Some(PrototypeFixtureDefect::ControlBlockMismatch)
    );

    // A different internal key.
    let mut foreign = coherent(&target);
    let mut block = foreign
        .construction
        .control
        .clone()
        .expect("the coherent fixture states one");
    block[1] ^= 0xff;
    foreign.construction.control = Some(block);
    assert_eq!(
        foreign.defect(&target),
        Some(PrototypeFixtureDefect::ControlBlockMismatch)
    );

    // A short block.
    let mut truncated = coherent(&target);
    let mut block = truncated
        .construction
        .control
        .clone()
        .expect("the coherent fixture states one");
    block.truncate(33);
    truncated.construction.control = Some(block);
    assert_eq!(
        truncated.defect(&target),
        Some(PrototypeFixtureDefect::ControlBlockMismatch)
    );
}

#[test]
fn only_the_parity_the_construction_determines_satisfies_a_control_block() {
    // The parity bit is not free.
    //
    // An earlier version of this validator masked it out, reasoning that
    // a fixture states a tree rather than an output key. That admitted
    // fixtures that were coherent here and refused by every executor
    // that derives its own control block, which is every honest one:
    // the parity is determined by the internal key and the tree exactly
    // as the rest of the block is.
    let target = target();
    let fixture = coherent(&target);
    let executing = leaf_hash_of_version_byte(LeafVersion::TAPSCRIPT.get(), OPERATION);
    let path = fixture
        .construction
        .tree
        .path_to(&executing)
        .expect("the leaf is in the tree");
    let determined = fixture
        .construction
        .control
        .as_ref()
        .expect("the coherent fixture states one");
    let parity = determined[0] & 0x01;

    let mut wrong_parity = coherent(&target);
    wrong_parity.construction.control = Some(control_block_of_version_byte(
        LeafVersion::TAPSCRIPT.get(),
        parity ^ 1,
        &UNSPENDABLE_INTERNAL_KEY,
        &path,
    ));
    assert_eq!(
        wrong_parity.defect(&target),
        Some(PrototypeFixtureDefect::ControlBlockMismatch),
        "the other parity is not the one this construction determines"
    );

    // And the leaf version's own bits are not free either.
    let mut wrong_version = coherent(&target);
    wrong_version.construction.control = Some(control_block_of_version_byte(
        0xc0,
        parity,
        &UNSPENDABLE_INTERNAL_KEY,
        &path,
    ));
    assert_eq!(
        wrong_version.defect(&target),
        Some(PrototypeFixtureDefect::ControlBlockMismatch)
    );
}

#[test]
fn a_predecessor_program_the_tree_does_not_determine_is_refused() {
    // The consumed input's program is not a free field: it is what this
    // internal key and this tree commit to, and no other tree commits
    // to it. A fixture stating some other program describes a spend of
    // an output its own tree does not commit to, and every honest
    // executor would refuse it after the harness had called it
    // coherent.
    let target = target();
    let mut fixture = coherent(&target);
    fixture.construction.predecessor_program[5] ^= 0xff;

    assert_eq!(
        fixture.defect(&target),
        Some(PrototypeFixtureDefect::PredecessorProgramMismatch)
    );
}

#[test]
fn a_fixture_whose_construction_has_no_output_key_is_refused() {
    // Coherence must imply constructibility. An internal key that is
    // not a curve point determines no output key at all, and a fixture
    // stating one used to pass every check here and fail only when an
    // executor tried to build it.
    let target = target();
    let mut fixture = coherent(&target);
    fixture.construction.internal_key = [0_u8; 32];

    assert!(
        matches!(
            fixture.defect(&target),
            Some(PrototypeFixtureDefect::NotConstructible(_))
        ),
        "an internal key off the curve determines nothing"
    );
}

#[test]
fn a_branch_offered_as_the_executing_leaf_is_named_accurately() {
    // A branch is not an absent leaf. It is a node a spend cannot
    // execute at all, and reporting the two the same way would send a
    // reader looking for the wrong defect.
    let target = target();
    let mut fixture = coherent(&target);
    fixture.construction.executing_leaf = fixture.construction.tree.clone();

    assert_eq!(
        fixture.defect(&target),
        Some(PrototypeFixtureDefect::ExecutingLeafIsNotALeaf)
    );

    // And the oracle names it the same way rather than calling it an
    // absent leaf.
    assert_eq!(
        construct(
            &UNSPENDABLE_INTERNAL_KEY,
            &fixture.construction.tree,
            &fixture.construction.tree,
        ),
        Err(ConstructionDefect::Tree(
            TreeDefect::ExecutingLeafIsNotALeaf
        ))
    );
}

#[test]
fn a_successor_output_role_must_appear_exactly_once() {
    let target = target();

    let mut none = coherent(&target);
    none.construction.outputs.clear();
    assert_eq!(
        none.defect(&target),
        Some(PrototypeFixtureDefect::OutputRoleNotUnique {
            role: OutputRole::Successor,
            found: 0,
        })
    );

    let mut twice = coherent(&target);
    let duplicate = twice.construction.outputs[0].clone();
    twice.construction.outputs.push(duplicate);
    assert_eq!(
        twice.defect(&target),
        Some(PrototypeFixtureDefect::OutputRoleNotUnique {
            role: OutputRole::Successor,
            found: 2,
        })
    );
}

#[test]
fn a_fixture_that_claims_nothing_is_refused() {
    // A pass would establish nothing, so the case is not worth running
    // and the language says so rather than counting it.
    let target = target();
    let mut fixture = coherent(&target);
    fixture.claims.clear();
    assert_eq!(
        fixture.defect(&target),
        Some(PrototypeFixtureDefect::NoClaims)
    );
}

#[test]
fn a_fixture_stated_against_another_contract_revision_is_refused() {
    let target = target();
    let mut fixture = coherent(&target);
    fixture.target_contract_version = fixture.target_contract_version.wrapping_add(1);
    assert!(matches!(
        fixture.defect(&target),
        Some(PrototypeFixtureDefect::ContractRevisionMismatch { .. })
    ));
}

// -- The claim vocabulary -----------------------------------------

#[test]
fn every_claim_belongs_to_exactly_one_relation() {
    // The registry now carries both relations' claims, so the property
    // is a partition rather than a constant: every claim names one
    // relation, both relations are named, and no claim of one is
    // reachable as coverage of the other.
    let mut constructor = 0_usize;
    let mut wide_floor = 0_usize;
    for claim in PrototypeClaim::ALL {
        match claim.relation() {
            PrototypeRelation::MetadataConstructorContinuity => constructor += 1,
            PrototypeRelation::WideFloorRelation => wide_floor += 1,
        }
    }
    assert_eq!(constructor + wide_floor, PrototypeClaim::ALL.len());
    assert!(constructor > 0);
    assert!(wide_floor > 0);
}

#[test]
fn every_constructor_claim_is_unresolved_with_a_stated_reason() {
    // The honest state of this wave: the language and the reference
    // oracle exist, and no executed case bears on any constructor claim
    // yet. Unresolved is not success and is not failure -- it is the
    // project saying which corner it has not established, in a form a
    // reader can enumerate (Guide-10 `rule:guide10:claim-registry`).
    for claim in PrototypeClaim::ALL {
        match claim.requirement() {
            ClaimRequirement::Unresolved(reason) => {
                assert!(!reason.is_empty(), "{claim:?}");
            }
            ClaimRequirement::Required => {
                panic!("{claim:?} is recorded as required, but no case can bear on it yet")
            }
        }
    }
}

#[test]
fn the_claim_census_has_no_repetitions() {
    let distinct: BTreeSet<_> = PrototypeClaim::ALL.iter().collect();
    assert_eq!(distinct.len(), PrototypeClaim::ALL.len());

    let relations: BTreeSet<_> = PrototypeRelation::ALL.iter().collect();
    assert_eq!(relations.len(), PrototypeRelation::ALL.len());
}

// -- The protocol boundary ----------------------------------------

#[test]
fn a_request_without_a_construction_is_byte_identical_to_a_schema_two_request() {
    // The whole justification for keeping the protocol revision at 2:
    // the new field is omitted from the wire entirely rather than
    // written as null, so a schema-2 executor's strict framing sees
    // exactly the message it always saw
    // (Guide-10 `rule:guide10:schema-migration`).
    let fixture = any_primitive_fixture();
    let request = NativeExecutionRequest {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: fixture.case(),
        fixture,
        construction: None,
    };

    let encoded = serde_json::to_string(&request).expect("the request encodes");
    assert!(
        !encoded.contains("construction"),
        "an absent construction must not appear on the wire: {encoded}"
    );
}

#[test]
fn a_tree_bearing_request_round_trips() {
    let target = target();
    let fixture = any_primitive_fixture();
    let request = NativeExecutionRequest {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: fixture.case(),
        fixture,
        construction: Some(coherent(&target).construction),
    };

    let encoded = serde_json::to_string(&request).expect("the request encodes");
    assert!(encoded.contains("construction"));
    let decoded: NativeExecutionRequest =
        serde_json::from_str(&encoded).expect("the request decodes");
    assert_eq!(decoded, request);
}

#[test]
fn an_executor_that_did_not_advertise_a_tree_is_not_sent_one() {
    // The gate that keeps the protocol revision at 2, exercised rather
    // than described. Without it the additive field would reach an
    // executor whose strict framing rejects the whole message.
    let mut handshake = crate::tests::support::nonmock_handshake();
    handshake
        .capabilities
        .remove(&ExecutorCapability::TreeMaterialization);
    assert!(!handshake.materializes_trees());

    handshake
        .capabilities
        .insert(ExecutorCapability::TreeMaterialization);
    assert!(handshake.materializes_trees());
}

#[test]
fn tree_materialization_is_an_advertised_capability() {
    // A tree-bearing request goes only to an executor that said it can
    // build one. That is what makes the additive field safe for a
    // schema-2 executor rather than merely convenient.
    let encoded = serde_json::to_string(&ExecutorCapability::TreeMaterialization)
        .expect("the capability encodes");
    assert_eq!(encoded, "\"tree_materialization\"");
}

#[test]
fn a_tree_survives_a_wire_round_trip() {
    let tree = FixtureTapTree::branch(
        FixtureTapTree::leaf(OPERATION.to_vec()),
        FixtureTapTree::branch(
            FixtureTapTree::leaf(METADATA.to_vec()),
            FixtureTapTree::leaf_of_version(0xc0, b"other".to_vec()),
        ),
    );
    let encoded = serde_json::to_string(&tree).expect("the tree encodes");
    let decoded: FixtureTapTree = serde_json::from_str(&encoded).expect("the tree decodes");
    assert_eq!(decoded, tree);
    assert_eq!(decoded.node_hash(), tree.node_hash());
}

#[test]
fn a_constructor_fixture_states_no_exact_resource_figure_yet() {
    // What a constructor instance costs is a measurement a later wave
    // takes. An invented figure would fail an honest executor over a
    // number no contract states (Guide-10 `rule:guide10:no-calibration`).
    let target = target();
    let fixture = coherent(&target);
    for expectation in [
        fixture.expected_resources.script_bytes,
        fixture.expected_resources.peak_stack_items,
        fixture.expected_resources.transaction_weight,
    ] {
        assert_eq!(expectation, ResourceExpectation::RecordedOnly);
    }
}
