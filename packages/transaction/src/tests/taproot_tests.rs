//! Committed-tree hashing, against digests computed outside this
//! crate.
//!
//! # Why the expectations are constants
//!
//! A tagged hash checked against another call to the same tagged hash
//! establishes nothing. The digests below were computed separately,
//! from the published construction and the target's own tag strings,
//! and are written out here so that a wrong tag, a missing length
//! prefix, or a branch ordering mistake shows up as a mismatch rather
//! than as an internally consistent tree that commits to nothing the
//! target recognizes.
//!
//! # The tree is checked the way the target checks it
//!
//! The end-to-end test walks a control block's path upward from the
//! leaf hash the way `ComputeTaprootMerkleRoot` does, which is the
//! opposite direction from the way this crate builds it. Two directions
//! agreeing is worth something; one direction agreeing with itself is
//! not.

use linker::backend::LeafRole;
use target_elements::LeafVersion;

use crate::taproot::{
    Digest32, TAPROOT_LEAF_MASK, branch_hash, commit_tree, leaf_hash, tagged_hash,
};
use crate::tests::{INTERNAL_KEY, candidate_abi, linked_bundle, pin, reviewed_target};

/// The tapleaf hash of the two-byte script `51 93`, under the target's
/// own leaf tag and leaf version.
const SHORT_LEAF: Digest32 = [
    0x6a, 0x6c, 0xc9, 0x79, 0xa6, 0x2f, 0x96, 0x44, 0xe1, 0x0f, 0xb9, 0x9a, 0x1c, 0x26, 0xa1, 0x89,
    0x51, 0x9a, 0xad, 0xe5, 0x4f, 0x6a, 0xef, 0x9d, 0x85, 0x1f, 0x5e, 0xb6, 0x35, 0xdf, 0xa7, 0xdc,
];

/// The tapleaf hash of the three-byte script `52 93 87`.
const LONGER_LEAF: Digest32 = [
    0x4f, 0x95, 0x93, 0xa5, 0xb5, 0xbd, 0x24, 0xde, 0x80, 0x30, 0xb2, 0xc5, 0xdd, 0xcb, 0xc6, 0x44,
    0x1c, 0x6b, 0xb9, 0x05, 0xe2, 0xad, 0xf2, 0xb8, 0xd2, 0xb2, 0x10, 0xdd, 0x63, 0xd3, 0x19, 0x81,
];

/// The branch hash over those two leaves.
const BRANCH: Digest32 = [
    0x10, 0x9e, 0xca, 0xa6, 0x37, 0x49, 0x52, 0xbb, 0xe5, 0xca, 0x98, 0x74, 0x1c, 0xea, 0xf1, 0xc7,
    0x31, 0xd7, 0xd6, 0x66, 0x8a, 0x7d, 0xb6, 0x7d, 0x0a, 0x1e, 0xb3, 0x34, 0xe6, 0x2f, 0xd7, 0x2f,
];

/// The tagged hash of the empty message under the leaf tag.
const EMPTY_UNDER_LEAF_TAG: Digest32 = [
    0x33, 0xc7, 0xfb, 0x7e, 0xba, 0x56, 0x34, 0x19, 0x99, 0x6d, 0x14, 0x29, 0x4a, 0x9a, 0xca, 0x9e,
    0x8a, 0xea, 0xb5, 0xc2, 0xad, 0xb8, 0x7c, 0x81, 0x84, 0xfc, 0x93, 0x54, 0x06, 0x52, 0x6c, 0xd9,
];

#[test]
fn the_tagged_hash_matches_an_independently_computed_digest() {
    assert_eq!(tagged_hash("TapLeaf/elements", &[]), EMPTY_UNDER_LEAF_TAG);
}

#[test]
fn a_leaf_hash_covers_the_version_the_length_and_the_script() {
    assert_eq!(leaf_hash(LeafVersion::TAPSCRIPT, &[0x51, 0x93]), SHORT_LEAF);
    assert_eq!(
        leaf_hash(LeafVersion::TAPSCRIPT, &[0x52, 0x93, 0x87]),
        LONGER_LEAF
    );

    // The length prefix is what keeps a script and a longer one
    // beginning with it apart. Without it the two preimages below would
    // differ only in bytes the tag does not separate.
    assert_ne!(
        leaf_hash(LeafVersion::TAPSCRIPT, &[0x51]),
        leaf_hash(LeafVersion::TAPSCRIPT, &[0x51, 0x00])
    );
}

#[test]
fn a_branch_hash_orders_its_children() {
    assert_eq!(branch_hash(SHORT_LEAF, LONGER_LEAF), BRANCH);
    // The ordering is what lets a control block carry a bare path with
    // no side bits, so the two argument orders must agree.
    assert_eq!(branch_hash(LONGER_LEAF, SHORT_LEAF), BRANCH);
}

#[test]
fn every_committed_leaf_reaches_the_root_along_its_own_path() {
    // The target's own verification: start at the leaf hash, fold each
    // path element in with the branch hash, and arrive at the merkle
    // root. This crate builds the paths downward from the tree, so an
    // agreement here is between two directions rather than one.
    let target = reviewed_target();
    let bundle = linked_bundle();
    let tree = commit_tree(&target, &bundle).expect("the demonstration tree hashes");

    assert!(!tree.paths().is_empty());
    for (leaf, path) in tree.paths() {
        let mut node = *tree
            .leaf_hashes()
            .get(leaf)
            .expect("every leaf with a path has a hash");
        for sibling in path {
            node = branch_hash(node, *sibling);
        }
        assert_eq!(node, tree.merkle_root(), "leaf {leaf:?} reaches the root");
    }
}

#[test]
fn the_leaf_hashes_are_taken_over_the_programs_the_tree_keeps() {
    let target = reviewed_target();
    let bundle = linked_bundle();
    let tree = commit_tree(&target, &bundle).expect("the demonstration tree hashes");

    for (leaf, script) in tree.leaf_programs() {
        assert_eq!(
            tree.leaf_hashes().get(leaf),
            Some(&leaf_hash(tree.leaf_version(), script))
        );
        // And those programs are the linked ones, not some re-encoding
        // of a pre-link program.
        assert_eq!(
            script,
            &bundle
                .program(*leaf)
                .expect("the tree commits only to linked leaves")
                .program()
                .encode(&target)
        );
    }
}

#[test]
fn a_control_block_carries_the_version_the_parity_the_key_and_the_path() {
    let abi = candidate_abi();
    let leaf = *abi
        .tree()
        .paths()
        .keys()
        .next()
        .expect("the demonstration tree commits to leaves");
    let block = abi
        .tree()
        .control_block(leaf, abi.pin())
        .expect("a committed leaf has a control block");
    let path = abi.tree().paths().get(&leaf).expect("its path");

    assert_eq!(block.len(), 33 + 32 * path.len());
    assert_eq!(block[0] & TAPROOT_LEAF_MASK, LeafVersion::TAPSCRIPT.get());
    assert_eq!(block[0] & 1, pin().parity().bit());
    assert_eq!(&block[1..33], &INTERNAL_KEY);
    for (index, sibling) in path.iter().enumerate() {
        assert_eq!(&block[33 + 32 * index..33 + 32 * (index + 1)], sibling);
    }
}

#[test]
fn a_leaf_the_tree_does_not_commit_to_has_no_control_block() {
    let abi = candidate_abi();
    let absent = LeafRole::Member { ash_inputs: 200 };
    assert!(abi.tree().control_block(absent, abi.pin()).is_err());
}
