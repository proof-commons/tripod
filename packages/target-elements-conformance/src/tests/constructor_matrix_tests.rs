//! The constructor matrix, checked as a matrix.
//!
//! Every test here asks a question about the whole set rather than about
//! one row: that every row states a coherent case, that every row is
//! distinct, that the accepting rows and the refusing rows are both
//! present, that the witness each row carries is the one the emitted
//! program's own contract consumes, and that every claim the vocabulary
//! declares has at least one row bearing on it.
//!
//! None of it is target evidence. A coherent fixture is a fixture an
//! executor can be asked to run, and what a target does with it is the
//! native run's answer `(´[PLAN-rule:guide10:validated-native-evidence]´)`.

use std::collections::BTreeSet;

use target_elements::{ReviewedElementsTapscriptDefinition, StackValueType};

use crate::constructor::metadata::METADATA_BYTES;
use crate::prototype::{
    CompoundPrototypeFixture, ExpectedPrototypeOutcome, PrototypeClaim, PrototypeRelation,
    bearing_cases, constructor_case_matrix,
};
use crate::prototype_program::PrototypeProgram;

/// The reviewed contract these rows are stated against.
fn target() -> ReviewedElementsTapscriptDefinition {
    crate::tests::support::reviewed_target()
}

/// The authored matrix.
fn matrix() -> Vec<CompoundPrototypeFixture> {
    constructor_case_matrix(&target())
        .expect("the constructor matrix is determined")
        .rows()
        .to_vec()
}

/// The claims this relation owns.
///
/// The registry carries both prototypes' claims, and a matrix answers
/// only for its own: demanding a wide-floor bearing case of a
/// constructor row would be demanding coverage of a relation these rows
/// do not state `(´[PLAN-rule:guide10:claim-coverage]´)`.
fn constructor_claims() -> Vec<PrototypeClaim> {
    PrototypeClaim::ALL
        .iter()
        .copied()
        .filter(|claim| claim.relation() == PrototypeRelation::MetadataConstructorContinuity)
        .collect()
}

#[test]
fn every_row_states_a_coherent_case() {
    // The fixture language's own rules, applied to every row: the tree
    // contains the executing leaf exactly once, the consumed input's
    // program is the one that tree determines, the stated control block
    // is the one it determines, the successor role appears exactly once,
    // and every claim belongs to the relation
    // (´[PLAN-rule:guide10:fixture-validation]´).
    let target = target();
    for fixture in matrix() {
        assert_eq!(
            fixture.defect(&target),
            None,
            "{} states a coherent case",
            fixture.case
        );
    }
}

#[test]
fn the_matrix_is_the_stated_size() {
    // The census count, stated so that a row lost to a refactor is a
    // failure here rather than a silently smaller matrix.
    let matrix = matrix();
    assert_eq!(matrix.len(), 36);
}

#[test]
fn every_case_name_is_distinct() {
    // Duplicate identities would make two rows one row in every report
    // that indexed them (´[PLAN-rule:guide10:fixture-determinism]´).
    let matrix = matrix();
    let names: BTreeSet<String> = matrix
        .iter()
        .map(|fixture| fixture.case.name.clone())
        .collect();
    assert_eq!(names.len(), matrix.len());
}

#[test]
fn every_row_belongs_to_the_constructor_relation() {
    for fixture in matrix() {
        assert_eq!(
            fixture.case.relation,
            PrototypeRelation::MetadataConstructorContinuity
        );
    }
}

#[test]
fn the_matrix_carries_both_verdicts_and_far_more_refusals() {
    // A matrix of accepting rows would establish that the program can be
    // satisfied and nothing about what it refuses, which is the whole
    // point of a threat matrix
    // (´[PLAN-tab:guide10:constructor-threats]´).
    let matrix = matrix();
    let accepted = matrix
        .iter()
        .filter(|fixture| fixture.expected == ExpectedPrototypeOutcome::Accepted)
        .count();
    let rejected = matrix
        .iter()
        .filter(|fixture| fixture.expected == ExpectedPrototypeOutcome::Rejected)
        .count();

    assert!(accepted >= 4, "both parities and both counter ends");
    assert!(rejected > accepted * 4);
    assert_eq!(accepted + rejected, matrix.len());
}

#[test]
fn every_row_carries_the_witness_the_program_consumes() {
    // The rows and the emitted program agree about the witness by
    // checking rather than by convention: every item of every
    // constructor row is the width the program's own initial-stack
    // contract fixes for it, in the same order.
    //
    // The metadata-leaf rows are exempt and are the reason the check is
    // written this way: they spend a different leaf running a different
    // script, so the witness the composed program consumes is not the
    // witness they carry, and a rule that applied to them would be a
    // rule about the wrong script.
    let target = target();
    let program = PrototypeProgram::continuity(&target).expect("the prototype is admitted");
    let expected: Vec<usize> = program
        .initial_stack()
        .main()
        .iter()
        .map(|value| match value {
            StackValueType::Bytes { minimum, maximum } if minimum == maximum => *minimum,
            // The compressed key, whose width the encoding fixes.
            _ => 33,
        })
        .collect();

    for fixture in matrix() {
        if fixture.case.name.starts_with("metadata_leaf_spend") {
            continue;
        }
        let widths: Vec<usize> = fixture.initial_stack.iter().map(Vec::len).collect();
        // Exactly one row deliberately carries an object of the wrong
        // width, and one carries a wide one: those are the mutations,
        // and they are recognizable by name rather than by exemption.
        if fixture.case.name.contains("metadata_bytes") || fixture.case.name.contains("truncated") {
            assert_ne!(widths, expected, "{} mutates a width", fixture.case);
            continue;
        }
        assert_eq!(
            widths, expected,
            "{} carries the stated witness",
            fixture.case
        );
    }
}

#[test]
fn the_predecessor_object_is_the_one_the_tree_commits_to() {
    // The accepting rows' witnessed object is the object its own tree's
    // metadata leaf covers. A row whose witness disagreed with its tree
    // would be a rejection row wearing an accepting expectation.
    for fixture in matrix() {
        if fixture.expected != ExpectedPrototypeOutcome::Accepted {
            continue;
        }
        let object = fixture
            .initial_stack
            .get(3)
            .expect("an accepting row carries five witness items");
        assert_eq!(object.len(), METADATA_BYTES);

        let committed = crate::constructor::metadata_leaf::metadata_leaf_script(&target(), object)
            .expect("the object is expressible as a leaf");
        let leaf = crate::constructor::tree::FixtureTapTree::leaf(committed);
        assert!(
            fixture
                .construction
                .tree
                .distinct_leaf_hashes()
                .contains(&leaf.node_hash()),
            "{} witnesses the object its tree commits to",
            fixture.case
        );
    }
}

#[test]
fn every_declared_claim_has_a_bearing_case() {
    // The coverage question, answered from the rows themselves. A claim
    // with no bearing case is a corner of the constructor nothing would
    // establish even under a passing run
    // (´[PLAN-rule:guide10:claim-coverage]´).
    let matrix = matrix();
    let bearing = bearing_cases(&matrix);

    for claim in constructor_claims() {
        let cases = bearing
            .get(&claim)
            .unwrap_or_else(|| panic!("{claim:?} has a bearing case"));
        assert!(!cases.is_empty());
    }
}

#[test]
fn every_claim_is_borne_by_a_refusal_as_well_as_an_acceptance() {
    // A claim borne only by accepting rows would be established by a run
    // in which the program checked nothing: what shows a check is
    // performed is a case that fails when the checked thing is wrong.
    //
    // The unspendability claim is the one exception and is stated as
    // such: the metadata leaf is fail-closed, so every case bearing on it
    // is a refusal and an accepting one would be the defect.
    let matrix = matrix();
    for claim in constructor_claims() {
        let bearing: Vec<&CompoundPrototypeFixture> = matrix
            .iter()
            .filter(|fixture| fixture.claims.contains(&claim))
            .collect();
        let accepts = bearing
            .iter()
            .filter(|fixture| fixture.expected == ExpectedPrototypeOutcome::Accepted)
            .count();
        let rejects = bearing
            .iter()
            .filter(|fixture| fixture.expected == ExpectedPrototypeOutcome::Rejected)
            .count();

        assert!(rejects > 0, "{claim:?} is borne by a refusal");
        if claim == PrototypeClaim::MetadataLeafUnspendableObserved {
            assert_eq!(accepts, 0, "an unspendable leaf has no accepting case");
        } else {
            assert!(accepts > 0, "{claim:?} is borne by an acceptance");
        }
    }
}

#[test]
fn the_matrix_is_the_same_matrix_every_time() {
    // Fixture determinism, checked rather than assumed: the rows come
    // from a nonce grind and a parity search, and either could have made
    // the set depend on something other than its inputs
    // (´[PLAN-rule:guide10:fixture-determinism]´).
    assert_eq!(matrix(), matrix());
}

#[test]
fn every_threat_matrix_row_is_named_by_a_case() {
    // The §9.14 rows, as an enumeration checked against the authored
    // names. A row this matrix does not cover is visible here rather
    // than absent.
    let names: BTreeSet<String> = matrix()
        .into_iter()
        .map(|fixture| fixture.case.name)
        .collect();

    for required in [
        "predecessor_counter_changed",
        "predecessor_domain_changed",
        "predecessor_schema_changed",
        "predecessor_object_kind_changed",
        "predecessor_flags_changed",
        "predecessor_reserved_field_nonzero",
        "successor_counter_unchanged",
        "successor_counter_incremented_by_two",
        "successor_flags_changed",
        "predecessor_truncated_metadata",
        "predecessor_alternate_field_order",
        "predecessor_trailing_metadata_bytes",
        "wrong_metadata_leaf_framing",
        "wrong_static_root",
        "split_predecessor_and_successor_roots",
        "wrong_internal_key",
        "wrong_predecessor_output_key_parity",
        "noncanonical_branch_order",
        "successor_output_carries_another_program",
        "metadata_leaf_spend_one_true_item",
        "constructor_from_another_schema",
        "stale_constructor_successor_subtree",
    ] {
        assert!(names.contains(required), "the matrix states {required}");
    }
}
