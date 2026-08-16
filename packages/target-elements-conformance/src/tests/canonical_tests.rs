//! The canonical branch-order grind.

use target_elements::reviewed_elements_tapscript;

use crate::constructor::canonical::{CanonicalSide, construct_canonically_ordered};
use crate::constructor::internal_key::UNSPENDABLE_INTERNAL_KEY;
use crate::constructor::metadata::PrototypeMetadata;
use crate::constructor::metadata_leaf::metadata_leaf_script;
use crate::constructor::tree::FixtureTapTree;
use crate::prototype_program::PrototypeProgram;

/// The reviewed contract.
fn target() -> target_elements::ReviewedElementsTapscriptDefinition {
    reviewed_elements_tapscript().expect("the reviewed contract validates")
}

/// The static operation subtree: the prototype program, as its leaf.
fn static_subtree() -> FixtureTapTree {
    let reviewed = target();
    let program =
        PrototypeProgram::continuity(&reviewed).expect("the continuity prototype is admitted");
    FixtureTapTree::leaf(program.encode(&reviewed))
}

/// One metadata object, at the schema's first counter.
fn metadata() -> PrototypeMetadata {
    PrototypeMetadata {
        schema: 1,
        object_kind: 1,
        counter: 0,
        flags: 0,
        nonce: 0,
    }
}

#[test]
fn the_grind_puts_the_metadata_leaf_on_the_side_the_program_hashes_it_on() {
    // The property the whole construction rests on. The program hashes
    // the metadata leaf first and the static root second, and cannot
    // order them; the creator makes that order the canonical one by
    // advancing the representation nonce
    // (Guide-10 `rule:guide10:tapbranch-order`).
    let reviewed = target();
    let subtree = static_subtree();
    let built = construct_canonically_ordered(
        &reviewed,
        &UNSPENDABLE_INTERNAL_KEY,
        &metadata(),
        &subtree,
        &subtree,
        1_000,
    )
    .expect("a canonical nonce is found within the bound");

    let leaf_script = metadata_leaf_script(&reviewed, &built.metadata().encode())
        .expect("the ground metadata is expressible");
    let leaf = FixtureTapTree::leaf(leaf_script);
    assert!(
        CanonicalSide::MetadataFirst.holds(&leaf.node_hash(), &subtree.node_hash()),
        "the ground nonce puts the metadata leaf first"
    );

    // The tree the instance commits to is the branch over exactly those
    // two children, and it determines the output the fixture states.
    assert_eq!(
        built.tree().node_hash(),
        FixtureTapTree::branch(leaf, subtree).node_hash()
    );
    assert_eq!(built.output().merkle_root(), &built.tree().node_hash());
}

#[test]
fn the_grind_is_deterministic_and_public() {
    // Anybody holding the object derives the same nonce by the same
    // search, which is what makes the nonce not a secret and not a
    // thing to be communicated (Guide-10 `rule:guide10:public-data`).
    let reviewed = target();
    let subtree = static_subtree();
    let run = || {
        construct_canonically_ordered(
            &reviewed,
            &UNSPENDABLE_INTERNAL_KEY,
            &metadata(),
            &subtree,
            &subtree,
            1_000,
        )
        .expect("a canonical nonce is found within the bound")
    };
    assert_eq!(run(), run());
}

#[test]
fn about_half_of_the_nonces_are_refused_and_that_is_the_expected_cost() {
    // The grind's cost, measured rather than asserted. The ordering
    // condition holds for about half of the nonces, so a small number of
    // attempts is what a reader should expect and a large one would mean
    // something else was wrong.
    let reviewed = target();
    let subtree = static_subtree();
    let built = construct_canonically_ordered(
        &reviewed,
        &UNSPENDABLE_INTERNAL_KEY,
        &metadata(),
        &subtree,
        &subtree,
        1_000,
    )
    .expect("a canonical nonce is found within the bound");

    assert!(built.attempts() >= 1);
    assert!(
        built.attempts() < 100,
        "the grind converged in {} attempts",
        built.attempts()
    );
}

#[test]
fn a_search_with_no_room_reports_its_own_bound() {
    // The honest floor. A bound of one admits only the nonce zero, and
    // where that nonce is on the wrong side the search says so rather
    // than pretending the construction is total.
    let reviewed = target();
    let subtree = static_subtree();
    let outcome = construct_canonically_ordered(
        &reviewed,
        &UNSPENDABLE_INTERNAL_KEY,
        &metadata(),
        &subtree,
        &subtree,
        1,
    );

    // Either nonce zero happened to be canonical, or the search reports
    // exhaustion. Both are correct; what is not admissible is a
    // successful result whose nonce is outside the bound.
    match outcome {
        Ok(built) => assert_eq!(built.attempts(), 1),
        Err(defect) => assert_eq!(
            defect,
            crate::constructor::canonical::CanonicalOrderDefect::SearchExhausted { attempts: 1 }
        ),
    }
}
