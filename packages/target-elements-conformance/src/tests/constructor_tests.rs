//! What the constructor oracle is checked against.
//!
//! # The published vectors are the point
//!
//! An oracle checked only against itself establishes nothing. The
//! strongest available second opinion is the target's own published
//! taproot vector file, which the upstream tree carries regenerated for
//! this target's tag strings: it was produced by the target's
//! implementation, not by anything here, and it fixes the tweak, the
//! tweaked key, the output program, and the exact control blocks
//! (Guide-10 `rule:guide10:constructor-oracle`).
//!
//! Provenance for every vector below:
//! `src/test/data/bip341_wallet_vectors.json`, the `scriptPubKey`
//! array, transcribed by index. They are public test data
//! (Guide-10 `rule:guide10:test-material`).
//!
//! Two things about those vectors need saying. Their leaf versions are
//! `0xc0` and `0xfa`, not this target's reviewed `0xc4`: the version
//! byte is an input to the leaf hash, so the vectors check the hashing
//! and the curve arithmetic exactly, and say nothing about which
//! version the target admits. And their first entry has no tree at all,
//! which is the case where the tweak preimage omits the merkle root
//! rather than zeroing it.

use target_elements::LeafVersion;

use crate::constructor::curve::{FIELD_ELEMENT_BYTES, PointDecodingDefect, lift_x};
use crate::constructor::internal_key::UNSPENDABLE_INTERNAL_KEY;
use crate::constructor::metadata::{
    METADATA_BYTES, MetadataDefect, PrototypeMetadata, TransitionDefect,
};
use crate::constructor::tagged::{Digest32, sha256, tagged_hash};
use crate::constructor::totality::{TotalityDefect, TweakTotalityPolicy, construct_under_policy};
use crate::constructor::tree::{
    ConstructionDefect, FixtureTapTree, TreeDefect, TweakDefect, branch_hash, construct,
    control_block, leaf_hash, leaf_hash_of_version_byte, output_program, tweak, tweak_without_tree,
    tweaked_key,
};

/// Reads one transcribed vector field.
fn bytes(hexadecimal: &str) -> Vec<u8> {
    assert!(
        hexadecimal.len().is_multiple_of(2),
        "a whole number of bytes"
    );
    (0..hexadecimal.len() / 2)
        .map(|index| {
            u8::from_str_radix(&hexadecimal[index * 2..index * 2 + 2], 16)
                .expect("a transcribed vector is hexadecimal")
        })
        .collect()
}

/// Reads one transcribed 32-byte field.
fn digest(hexadecimal: &str) -> Digest32 {
    let mut value = [0_u8; 32];
    value.copy_from_slice(&bytes(hexadecimal));
    value
}

/// The x-only key of one transcribed vector.
fn key(hexadecimal: &str) -> [u8; FIELD_ELEMENT_BYTES] {
    digest(hexadecimal)
}

/// The hexadecimal form of a byte string, for a readable failure.
fn shown(value: &[u8]) -> String {
    use std::fmt::Write as _;

    value.iter().fold(String::new(), |mut text, byte| {
        let _ = write!(text, "{byte:02x}");
        text
    })
}

// -- The published vectors ----------------------------------------

#[test]
fn the_treeless_vector_hashes_only_the_internal_key() {
    // Vector 0. No script tree, so the tweak preimage is the internal
    // key alone. A construction that wrote 32 zero bytes for the absent
    // root would produce a different key here and nowhere else.
    let internal = key("d6889cb081036e0faefa3a35157ad71086b123b2b144b649798b494c300a961d");
    let expected_tweak = digest("a42a817e8bb316e03e9d36f5f2a69452da4699e1222a35e21e71b945664625e7");

    let computed = tweak_without_tree(&internal);
    assert_eq!(shown(&computed), shown(&expected_tweak));

    let (output_key, _) = tweaked_key(&internal, &computed).expect("the vector has an output key");
    assert_eq!(
        shown(&output_key),
        "3ce74e4e5c292c1fbf94eaaf2ec64fb4700e28250070bfd5704867ee31e30420"
    );
    assert_eq!(
        shown(&output_program(&output_key)),
        "51203ce74e4e5c292c1fbf94eaaf2ec64fb4700e28250070bfd5704867ee31e30420"
    );
}

#[test]
fn the_single_leaf_vector_reproduces_exactly() {
    // Vector 1. One leaf, so the merkle root is the leaf hash itself
    // and the control block carries no path. Its first byte is `c1`:
    // the leaf version with an odd output key's parity bit set.
    //
    // # A stale field in the vector file
    //
    // The file's `intermediary.leafHashes` are *not* this target's leaf
    // hashes. They are the upstream Bitcoin-tagged values, left behind
    // when the file was regenerated: every `merkleRoot`, `tweak`,
    // `tweakedPubkey`, `scriptPubKey`, and control block in it is
    // Elements-tagged and reproduces here, while every `leafHashes`
    // entry reproduces only under the Bitcoin tags. Checked across all
    // six of the file's tree-bearing vectors.
    //
    // So `leafHashes` is not asserted anywhere in this file. Asserting
    // it would have been asserting the wrong target's answer, and the
    // single-leaf vector is where that shows plainest: here the leaf
    // hash *is* the merkle root, and the file states two different
    // values for them.
    let internal = key("187791b6f712a8ea41c8ecdd0ee77fab3e85263b37e1ec18a3651926b3a6cf27");
    let script = bytes("20d85a959b0290bf19bb89ed43c916be835475d013da4b362117393e25a48229b8ac");

    let root = leaf_hash_of_version_byte(0xc0, &script);
    assert_eq!(
        shown(&root),
        "a24918646b6b0ba80fee8ced329493103182c886ba737a1d3da69fd8e8cb3cf3"
    );

    let computed = tweak(&internal, &root);
    assert_eq!(
        shown(&computed),
        "ce08db99c02d29ccebcf4b7b4b7d88111a5f1eaa23a1216340876338f26976d2"
    );

    let (output_key, parity) = tweaked_key(&internal, &computed).expect("the vector has a key");
    assert_eq!(
        shown(&output_program(&output_key)),
        "5120f8c9791817c6781858c5e7c8314cd36d2b0c22b0c138e756bfe3a6fd3e39bc00"
    );

    // A path-free control block, and an odd output key.
    assert_eq!(parity, 1);
    let mut block = vec![0xc0 | parity];
    block.extend_from_slice(&internal);
    assert_eq!(
        shown(&block),
        "c1187791b6f712a8ea41c8ecdd0ee77fab3e85263b37e1ec18a3651926b3a6cf27"
    );
}

#[test]
fn the_two_leaf_vector_reproduces_exactly() {
    // Vector 3. Two leaves under different leaf versions, which is
    // also the smallest tree where the branch ordering can be wrong.
    let internal = key("ee4fe085983462a184015d1f782d6a5f8b9c2b60130aff050ce221ecf3786592");
    let first = leaf_hash_of_version_byte(
        0xc0,
        &bytes("20387671353e273264c495656e27e39ba899ea8fee3bb69fb2a680e22093447d48ac"),
    );
    let second = leaf_hash_of_version_byte(0xfa, &bytes("06424950333431"));

    assert_eq!(
        shown(&first),
        "d0d1c6f231d663dd8d0aab290dac759b230da655aff020da1fa88d61f1eae0ea"
    );
    assert_eq!(
        shown(&second),
        "01c55a7ecd2af27d5bff8614e20f0b316942e398af7496c6a23d54e47c91e140"
    );

    let root = branch_hash(&first, &second);
    assert_eq!(
        shown(&root),
        "fc45a034cc4618d0f1688fd29604d5c711a4f22754ec0cc4c5512434abcefd21"
    );

    // The order the children are offered in must not matter: the
    // target sorts them, and so does this.
    assert_eq!(shown(&branch_hash(&second, &first)), shown(&root));

    let computed = tweak(&internal, &root);
    assert_eq!(
        shown(&computed),
        "f72f92121c50790646791db0f848ce7571d81cf83fef6dba91b21aff25f902fb"
    );

    let (output_key, parity) = tweaked_key(&internal, &computed).expect("the vector has a key");
    assert_eq!(
        shown(&output_key),
        "3318184ee1f2b40f87119febc1e5d343c9829ce8d1b39a3f93886de2ea94bce1"
    );
    assert_eq!(
        shown(&output_program(&output_key)),
        "51203318184ee1f2b40f87119febc1e5d343c9829ce8d1b39a3f93886de2ea94bce1"
    );

    // Both of the vector's control blocks, byte for byte. The first
    // byte of each is its own leaf version with the same parity bit.
    let mut expected_first = vec![0xc0 | parity];
    expected_first.extend_from_slice(&internal);
    expected_first.extend_from_slice(&second);
    assert_eq!(
        shown(&expected_first),
        "c1ee4fe085983462a184015d1f782d6a5f8b9c2b60130aff050ce221ecf378659201c55a7ecd2af27d5bff8614e20f0b316942e398af7496c6a23d54e47c91e140"
    );

    let mut expected_second = vec![0xfa | parity];
    expected_second.extend_from_slice(&internal);
    expected_second.extend_from_slice(&first);
    assert_eq!(
        shown(&expected_second),
        "fbee4fe085983462a184015d1f782d6a5f8b9c2b60130aff050ce221ecf3786592d0d1c6f231d663dd8d0aab290dac759b230da655aff020da1fa88d61f1eae0ea"
    );
}

#[test]
fn the_three_leaf_vector_reproduces_exactly_including_both_path_depths() {
    // Vector 5. An unbalanced tree: one leaf at depth one and two at
    // depth two, so the control paths have different lengths and the
    // deeper ones must be ordered deepest sibling first.
    let internal = key("e0dfe2300b0dd746a3f8674dfd4525623639042569d829c7f0eed9602d263e6f");
    let leaves = [
        "2072ea6adcf1d371dea8fba1035a09f3d24ed5a059799bae114084130ee5898e69ac",
        "202352d137f2f3ab38d1eaa976758873377fa5ebb817372c71e2c542313d4abda8ac",
        "207337c0dd4253cb86f2c43a2351aadd82cccb12a172cd120452b9bb8324f2186aac",
    ]
    .map(|script| leaf_hash_of_version_byte(0xc0, &bytes(script)));

    assert_eq!(
        shown(&leaves[0]),
        "8d1a84ecd32fab8bb15a78f8d757d8489d3777d70d02b894a76dc4d9080ee232"
    );

    let deep = branch_hash(&leaves[1], &leaves[2]);
    assert_eq!(
        shown(&deep),
        "fe7e4669dfd85b744abb7be99d240df87459078026a10588dc9849845e1a9abb"
    );
    let root = branch_hash(&leaves[0], &deep);
    assert_eq!(
        shown(&root),
        "d0071e27366be098f194de1b3b11903eab22db9d0bae6455b4c9eaae3de917b4"
    );

    let computed = tweak(&internal, &root);
    let (output_key, parity) = tweaked_key(&internal, &computed).expect("the vector has a key");
    assert_eq!(
        shown(&output_key),
        "a3822f8a999c9f396b975379883d57daa16e5e6489f2e85c0f5ad980eb42c8ad"
    );
    // This vector's output key has an even y, so its control blocks
    // begin with the bare leaf version. Vector 3's did not: between
    // them both parities are covered.
    assert_eq!(parity, 0);

    // The shallow leaf's path is one node; the deep ones are two, with
    // the immediate sibling first and the uncle second.
    let mut shallow = vec![0xc0 | parity];
    shallow.extend_from_slice(&internal);
    shallow.extend_from_slice(&deep);
    assert_eq!(
        shown(&shallow),
        "c0e0dfe2300b0dd746a3f8674dfd4525623639042569d829c7f0eed9602d263e6ffe7e4669dfd85b744abb7be99d240df87459078026a10588dc9849845e1a9abb"
    );

    let mut deeper = vec![0xc0 | parity];
    deeper.extend_from_slice(&internal);
    deeper.extend_from_slice(&leaves[2]);
    deeper.extend_from_slice(&leaves[0]);
    assert_eq!(
        shown(&deeper),
        "c0e0dfe2300b0dd746a3f8674dfd4525623639042569d829c7f0eed9602d263e6f452798d4907a9b7d094dfc63161f0ab0ac28e74f955b779a49596bab286a3bab8d1a84ecd32fab8bb15a78f8d757d8489d3777d70d02b894a76dc4d9080ee232"
    );
}

// -- The tagged hash and the curve --------------------------------

#[test]
fn a_tagged_hash_is_the_tag_digest_twice_and_then_the_message() {
    // The construction itself, against a hand-composed preimage.
    // Provenance: `src/hash.cpp:85-93`.
    let prefix = sha256(b"TapLeaf/elements");
    let mut preimage = Vec::new();
    preimage.extend_from_slice(&prefix);
    preimage.extend_from_slice(&prefix);
    preimage.extend_from_slice(b"message");
    assert_eq!(
        tagged_hash("TapLeaf/elements", b"message"),
        sha256(&preimage)
    );
}

#[test]
fn a_tag_separates_domains() {
    assert_ne!(
        tagged_hash("TapLeaf/elements", b""),
        tagged_hash("TapBranch/elements", b"")
    );
    // And the target's tags are not the upstream Bitcoin ones, which is
    // the mistake that would produce a well-formed wrong answer.
    assert_ne!(
        tagged_hash("TapLeaf/elements", b""),
        tagged_hash("TapLeaf", b"")
    );
}

#[test]
fn the_internal_key_is_the_published_derivation() {
    // The constant is recomputed rather than trusted: it is the digest
    // of the generator's uncompressed encoding
    // (Guide-10 `rule:guide10:internal-key`).
    let generator = crate::constructor::curve::generator();
    assert_eq!(
        sha256(&generator.uncompressed_bytes()),
        UNSPENDABLE_INTERNAL_KEY
    );

    // And not of the compressed one, which is a different point
    // entirely and the easy mistake to make here.
    let mut compressed = vec![0x02];
    compressed.extend_from_slice(&generator.x_only_bytes());
    assert!(generator.has_even_y(), "the generator's y is even");
    assert_ne!(sha256(&compressed), UNSPENDABLE_INTERNAL_KEY);

    // And it is a point, which a nothing-up-my-sleeve x coordinate is
    // not guaranteed to be: roughly half of all field elements are not.
    assert!(lift_x(&UNSPENDABLE_INTERNAL_KEY).is_ok());
}

#[test]
fn a_field_element_that_is_no_point_is_refused() {
    // One that is not on the curve, and one that is not a field
    // element at all.
    assert_eq!(lift_x(&[0_u8; 32]), Err(PointDecodingDefect::NotOnCurve));
    assert_eq!(
        lift_x(&[0xff_u8; 32]),
        Err(PointDecodingDefect::NotAFieldElement)
    );
}

#[test]
fn lifting_an_x_coordinate_always_takes_the_even_point() {
    let point = lift_x(&UNSPENDABLE_INTERNAL_KEY).expect("the published point is a point");
    assert!(point.has_even_y());
    assert_eq!(point.parity_bit(), 0);
    assert_eq!(point.x_only_bytes(), UNSPENDABLE_INTERNAL_KEY);
}

// -- The typed tree -----------------------------------------------

/// A leaf under the reviewed leaf version.
fn reviewed_leaf(script: &[u8]) -> FixtureTapTree {
    FixtureTapTree::leaf(script.to_vec())
}

#[test]
fn a_tree_determines_one_control_path_for_each_leaf() {
    let first = reviewed_leaf(b"first");
    let second = reviewed_leaf(b"second");
    let third = reviewed_leaf(b"third");
    let tree = FixtureTapTree::branch(first.clone(), FixtureTapTree::branch(second, third.clone()));

    assert_eq!(tree.leaf_count(), 3);
    assert_eq!(tree.distinct_leaf_hashes().len(), 3);

    let shallow = tree
        .path_to(&leaf_hash(LeafVersion::TAPSCRIPT, b"first"))
        .expect("the leaf is in the tree");
    assert_eq!(shallow.len(), 1);

    let deep = tree
        .path_to(&leaf_hash(LeafVersion::TAPSCRIPT, b"second"))
        .expect("the leaf is in the tree");
    assert_eq!(deep.len(), 2);
    // Deepest sibling first: the immediate one, then the uncle.
    assert_eq!(deep[0], third.node_hash());
    assert_eq!(deep[1], first.node_hash());
}

#[test]
fn a_leaf_outside_the_tree_has_no_path() {
    let tree = FixtureTapTree::branch(reviewed_leaf(b"first"), reviewed_leaf(b"second"));
    assert_eq!(
        tree.path_to(&leaf_hash(LeafVersion::TAPSCRIPT, b"absent")),
        Err(TreeDefect::ExecutingLeafAbsent)
    );
}

#[test]
fn a_repeated_leaf_determines_no_path() {
    // Two identical leaves have the same hash, so a control path to
    // "the" leaf is not determined. The fixture rule requires the
    // executing leaf to occur exactly once
    // (Guide-10 `rule:guide10:fixture-validation`).
    let tree = FixtureTapTree::branch(reviewed_leaf(b"same"), reviewed_leaf(b"same"));
    assert_eq!(
        tree.path_to(&leaf_hash(LeafVersion::TAPSCRIPT, b"same")),
        Err(TreeDefect::ExecutingLeafRepeated)
    );
}

#[test]
fn a_branch_hashes_the_same_whichever_way_its_children_are_stated() {
    // The fixture's stated order is not the hashed order.
    let first = reviewed_leaf(b"first");
    let second = reviewed_leaf(b"second");
    assert_eq!(
        FixtureTapTree::branch(first.clone(), second.clone()).node_hash(),
        FixtureTapTree::branch(second, first).node_hash()
    );
}

#[test]
fn a_construction_produces_every_value_a_fixture_states() {
    let leaf = reviewed_leaf(b"operation");
    let metadata = reviewed_leaf(b"metadata");
    let tree = FixtureTapTree::branch(leaf.clone(), metadata);

    let output = construct(&UNSPENDABLE_INTERNAL_KEY, &tree, &leaf)
        .expect("the published key and a two-leaf tree construct");

    assert_eq!(output.merkle_root(), &tree.node_hash());
    assert_eq!(
        output.executing_leaf_hash(),
        &leaf_hash(LeafVersion::TAPSCRIPT, b"operation")
    );
    assert_eq!(output.output_program().len(), 34);
    assert_eq!(output.output_program()[0], 0x51);
    assert_eq!(output.output_program()[1], 0x20);
    assert_eq!(&output.output_program()[2..], output.output_key());
    // One base and one path node.
    assert_eq!(output.control_block().len(), 33 + 32);
    assert_eq!(
        output.control_block()[0],
        LeafVersion::TAPSCRIPT.get() | output.parity()
    );
    assert_eq!(&output.control_block()[1..33], &UNSPENDABLE_INTERNAL_KEY);
    assert_eq!(
        output.control_block(),
        control_block(
            LeafVersion::TAPSCRIPT,
            output.parity(),
            &UNSPENDABLE_INTERNAL_KEY,
            &tree
                .path_to(output.executing_leaf_hash())
                .expect("the leaf is in the tree"),
        )
    );
}

#[test]
fn constructing_against_a_leaf_outside_the_tree_is_refused() {
    let tree = FixtureTapTree::branch(reviewed_leaf(b"first"), reviewed_leaf(b"second"));
    assert_eq!(
        construct(&UNSPENDABLE_INTERNAL_KEY, &tree, &reviewed_leaf(b"absent")),
        Err(ConstructionDefect::Tree(TreeDefect::ExecutingLeafAbsent))
    );
}

#[test]
fn a_tweak_that_is_not_a_scalar_has_no_output_key() {
    // The tweak-totality failure mode, exhibited rather than argued.
    // The group order itself is the smallest value that is not a valid
    // multiplier (Guide-10 `rule:guide10:tweak-totality`).
    let order = digest("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141");
    assert_eq!(
        tweaked_key(&UNSPENDABLE_INTERNAL_KEY, &order),
        Err(TweakDefect::TweakNotAScalar)
    );
    assert_eq!(
        tweaked_key(&UNSPENDABLE_INTERNAL_KEY, &[0_u8; 32]),
        Err(TweakDefect::TweakNotAScalar)
    );
}

#[test]
fn an_internal_key_that_is_no_point_has_no_output_key() {
    assert_eq!(
        tweaked_key(
            &[0_u8; 32],
            &digest("0000000000000000000000000000000000000000000000000000000000000001")
        ),
        Err(TweakDefect::InternalKeyNotOnCurve(
            PointDecodingDefect::NotOnCurve
        ))
    );
}

// -- The metadata schema ------------------------------------------

/// A representative object.
const fn representative() -> PrototypeMetadata {
    PrototypeMetadata {
        schema: 1,
        object_kind: 7,
        counter: 42,
        flags: 0x0000_0003,
        nonce: 0,
    }
}

#[test]
fn a_metadata_object_encodes_to_one_exact_width() {
    assert_eq!(representative().encode().len(), METADATA_BYTES);
}

#[test]
fn a_metadata_encoding_round_trips_at_the_extremes() {
    for object in [
        PrototypeMetadata {
            schema: 0,
            object_kind: 0,
            counter: 0,
            flags: 0,
            nonce: 0,
        },
        representative(),
        PrototypeMetadata {
            schema: u32::MAX,
            object_kind: u32::MAX,
            counter: u64::MAX,
            flags: u32::MAX,
            nonce: u32::MAX,
        },
    ] {
        let encoded = object.encode();
        assert_eq!(PrototypeMetadata::decode(&encoded), Ok(object));
    }
}

#[test]
fn a_noncanonical_encoding_decodes_to_nothing() {
    let mut encoded = representative().encode();
    // A reserved byte that is not zero.
    encoded[47] = 1;
    assert_eq!(
        PrototypeMetadata::decode(&encoded),
        Err(MetadataDefect::ReservedFieldSet)
    );

    // A foreign domain.
    let mut foreign = representative().encode();
    foreign[0] = b'X';
    assert_eq!(
        PrototypeMetadata::decode(&foreign),
        Err(MetadataDefect::ForeignDomain)
    );

    // A short encoding.
    assert_eq!(
        PrototypeMetadata::decode(&representative().encode()[..47]),
        Err(MetadataDefect::WrongWidth { offered: 47 })
    );
}

#[test]
fn a_transition_moves_the_counter_and_nothing_else() {
    let before = representative();
    let after = before.successor().expect("the counter has room");
    assert_eq!(after.counter, before.counter + 1);
    assert_eq!(after.schema, before.schema);
    assert_eq!(after.object_kind, before.object_kind);
    assert_eq!(after.flags, before.flags);
}

// -- Tweak totality -----------------------------------------------

#[test]
fn a_nonce_is_a_representation_and_not_a_state() {
    // Both halves of what makes the retry policy admissible: two
    // encodings of one state, and a nonce that cannot survive a
    // transition (Guide-10 `rule:guide10:tweak-totality`).
    let object = representative();
    let rewritten = object.with_nonce(17);
    assert!(object.same_state(&rewritten));
    assert_ne!(object.encode(), rewritten.encode());

    // The successor's counter does not depend on the nonce, and the
    // nonce does not carry.
    let from_zero = object.successor().expect("the counter has room");
    let from_seventeen = rewritten.successor().expect("the counter has room");
    assert!(from_zero.same_state(&from_seventeen));
    assert_eq!(from_zero.encode(), from_seventeen.encode());
    assert_eq!(from_zero.nonce, 0);
}

#[test]
fn every_policy_constructs_an_ordinary_instance_on_the_first_attempt() {
    let static_leaf = reviewed_leaf(b"operation");
    for policy in [
        TweakTotalityPolicy::RejectInstance,
        TweakTotalityPolicy::CanonicalNonceRetry {
            maximum_attempts: 8,
        },
        TweakTotalityPolicy::NamedNegligibleResidual,
    ] {
        let outcome = construct_under_policy(
            &UNSPENDABLE_INTERNAL_KEY,
            representative(),
            &static_leaf,
            policy,
            |object| FixtureTapTree::branch(static_leaf.clone(), reviewed_leaf(&object.encode())),
        )
        .expect("an ordinary instance constructs");

        assert_eq!(outcome.attempts(), 1, "{policy:?}");
        assert_eq!(outcome.metadata().nonce, 0, "{policy:?}");
    }
}

#[test]
fn a_defect_no_nonce_can_repair_is_not_retried() {
    // A tree that does not contain the executing leaf fails the same
    // way for every nonce, so the retry policy declines rather than
    // spinning through its bound.
    let static_leaf = reviewed_leaf(b"operation");
    let outcome = construct_under_policy(
        &UNSPENDABLE_INTERNAL_KEY,
        representative(),
        &reviewed_leaf(b"not in the tree"),
        TweakTotalityPolicy::CanonicalNonceRetry {
            maximum_attempts: 1_000,
        },
        |object| FixtureTapTree::branch(static_leaf.clone(), reviewed_leaf(&object.encode())),
    );

    assert_eq!(
        outcome,
        Err(TotalityDefect::NotRepairableByRetry(
            ConstructionDefect::Tree(TreeDefect::ExecutingLeafAbsent)
        ))
    );
}

#[test]
fn the_corpus_measures_no_retry_and_claims_nothing_about_the_tail() {
    // The measurement Guide-10 §9.12 asks for, and the honest reading
    // of it.
    //
    // What is measured: over this corpus, every instance had an output
    // key at nonce zero, so the retry policy and the reject policy are
    // indistinguishable on everything anybody has constructed.
    //
    // What is NOT established: that a retry is never needed. An
    // instance needs one when a hash of public data lands at or above
    // the group order, which happens for roughly one input in 2^128. A
    // corpus this size — or any size a test can run — cannot observe
    // that even once, so a run of zero is exactly what a correct
    // implementation and a broken one would both produce. The residual
    // stays named rather than measured away
    // (Guide-10 `rule:guide10:tweak-totality`).
    let static_leaf = reviewed_leaf(b"operation");
    let mut instances = 0_u32;
    let mut retried = 0_u32;

    for counter in 0..64_u64 {
        for flags in [0_u32, 1, u32::MAX] {
            for object_kind in [0_u32, 7] {
                let object = PrototypeMetadata {
                    schema: 1,
                    object_kind,
                    counter,
                    flags,
                    nonce: 0,
                };
                let outcome = construct_under_policy(
                    &UNSPENDABLE_INTERNAL_KEY,
                    object,
                    &static_leaf,
                    TweakTotalityPolicy::CanonicalNonceRetry {
                        maximum_attempts: 64,
                    },
                    |written| {
                        FixtureTapTree::branch(
                            static_leaf.clone(),
                            reviewed_leaf(&written.encode()),
                        )
                    },
                )
                .expect("every corpus instance constructs");

                instances += 1;
                if outcome.attempts() > 1 {
                    retried += 1;
                }
            }
        }
    }

    assert_eq!(instances, 384);
    assert_eq!(retried, 0, "no corpus instance needed a retry");
}

#[test]
fn an_exhausted_counter_has_no_successor() {
    // Refused rather than wrapped: a wrapped counter would make two
    // distinct states of one object indistinguishable.
    let exhausted = PrototypeMetadata {
        counter: u64::MAX,
        ..representative()
    };
    assert_eq!(
        exhausted.successor(),
        Err(TransitionDefect::CounterExhausted)
    );
}

#[test]
fn a_metadata_transition_changes_the_leaf_and_therefore_the_output() {
    // The property the whole constructor rests on: the successor is a
    // different output program, derived from the same static subtree
    // and the same internal key.
    let before = representative();
    let after = before.successor().expect("the counter has room");
    let static_leaf = reviewed_leaf(b"operation");

    let of = |object: &PrototypeMetadata| {
        let tree = FixtureTapTree::branch(static_leaf.clone(), reviewed_leaf(&object.encode()));
        construct(&UNSPENDABLE_INTERNAL_KEY, &tree, &static_leaf)
            .expect("the construction has an output key")
    };

    let predecessor = of(&before);
    let successor = of(&after);
    assert_ne!(predecessor.output_program(), successor.output_program());
    assert_ne!(predecessor.merkle_root(), successor.merkle_root());
    // The executing leaf is the same one, and its hash does not move
    // with the metadata.
    assert_eq!(
        predecessor.executing_leaf_hash(),
        successor.executing_leaf_hash()
    );
}
