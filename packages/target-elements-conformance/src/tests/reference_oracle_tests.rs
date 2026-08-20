//! Cross-checks against the adopted reference implementation.
//!
//! # Claim class, once, for every test in this file
//!
//! **Reference-implementation conformance, not independent evidence.**
//! The `elements` and `secp256k1-zkp` crates bind the same C library
//! the node vendors, so agreement here says the first-party computation
//! matches the target's own implementation. It is not a second
//! independent opinion and it is not a specification check. Each test
//! restates the class in one line so that a reader arriving at a single
//! function cannot mistake what it proves.
//!
//! # What is compared, and against what
//!
//! Two genuinely separate computations of the same object, every time:
//! the first-party side computes from the reviewed target contract with
//! this workspace's own encoder, hashing, and curve arithmetic, and the
//! reference side computes through the vendored C library's bindings.
//! Neither is derived from the other. A disagreement is a defect
//! finding about one of the two, never a reason to adjust either.

use std::collections::{BTreeMap, BTreeSet};

use elements::encode::{deserialize, serialize};
use elements::hashes::sha256d;
use elements::schnorr::TapTweak as _;
use elements::taproot::{
    ControlBlock, LeafVersion as ReferenceLeafVersion, TapLeafHash, TapNodeHash,
};
use elements::{Script, Transaction};
use secp256k1_zkp::{Secp256k1, XOnlyPublicKey};
use tapscript::bundle::LeafRole;
use transaction::{PinnedAshInstance, TargetTransaction, commit_tree};
use vectors::bundle::{INTERNAL_KEY, PINNED_PROGRAM, fixture_bundle};
use vectors::fixture::positive_semantic_census;
use vectors::materialize::{
    AshFunding, MaterializedTargetVector, is_materializable, materialize, vector_id,
};

use crate::reference::{
    FIXTURE_MERKLE_ROOT, FIXTURE_REFERENCE_OUTPUT_KEY, FIXTURE_REFERENCE_OUTPUT_KEY_PARITY_BIT,
    REFERENCE_KEY_BYTES,
};

/// How many sponsorless positive fixtures the census materializes.
///
/// Restated rather than derived so that a fixture silently leaving the
/// set fails a count here instead of quietly shrinking the cross-check.
const SPONSORLESS_FIXTURES: usize = 9;

/// The leaf version the reviewed target commits tapscript leaves under.
///
/// Provenance: `TAPROOT_LEAF_TAPSCRIPT` in the reviewed interpreter,
/// which the reference crate names by the same constant.
const REVIEWED_LEAF_VERSION: u8 = 0xc4;

/// How many leaves the fixture bundle's committed tree carries.
///
/// Restated rather than derived, for the same reason the fixture count
/// is: a leaf silently leaving the tree would shrink every per-leaf
/// cross-check below without failing one of them.
const COMMITTED_LEAVES: usize = 12;

/// Every sponsorless fixture, materialized once.
fn materialized() -> Vec<MaterializedTargetVector> {
    let fixture = fixture_bundle().expect("the fixture bundle builds");
    let census = positive_semantic_census().expect("the positive census builds");
    let vectors: Vec<_> = census
        .iter()
        .filter(|case| is_materializable(case))
        .map(|case| {
            // The canonical fixtures are cross-checked as artifacts, not
            // executed: their inputs are the named placeholder coins,
            // which is what makes these exact bytes reproducible here.
            let funding = AshFunding::unexecutable_placeholder(vector_id(case));
            materialize(&fixture, case, &funding)
                .unwrap_or_else(|error| panic!("{:?} did not materialize: {error:?}", case.id()))
        })
        .collect();
    assert_eq!(
        vectors.len(),
        SPONSORLESS_FIXTURES,
        "the sponsorless fixture set changed size, so the cross-check no longer covers what it says"
    );
    vectors
}

/// The fixture bundle's committed tree, hashed first-party.
fn committed_tree() -> transaction::CommittedTree {
    let fixture = fixture_bundle().expect("the fixture bundle builds");
    commit_tree(fixture.target(), fixture.linked()).expect("the fixture tree hashes")
}

/// The fixture bundle's pinned instance.
fn fixture_pin() -> PinnedAshInstance {
    fixture_bundle()
        .expect("the fixture bundle builds")
        .pin()
        .clone()
}

// --- (a) Transaction encoding ----------------------------------------

#[test]
fn every_fixture_transaction_round_trips_through_the_reference_decoder() {
    // Reference-implementation conformance, not independent evidence.
    //
    // The first-party encoder produced these bytes from the reviewed
    // serialization rules. The reference decoder reads them back and
    // re-encodes them; a byte difference means one of the two encoders
    // is wrong about the target's wire format.
    let vectors = materialized();
    for vector in &vectors {
        let decoded: Transaction = deserialize(vector.bytes()).unwrap_or_else(|error| {
            panic!(
                "{:?}: the reference decoder refused first-party bytes: {error}",
                vector.id()
            )
        });
        assert_eq!(
            serialize(&decoded),
            vector.bytes(),
            "{:?}: the reference re-encoding is not the first-party encoding",
            vector.id()
        );
    }
}

#[test]
fn the_reference_and_first_party_decoders_agree_on_structure() {
    // Reference-implementation conformance, not independent evidence.
    //
    // A round trip alone would pass for a decoder that treated the
    // bytes as opaque. Comparing the input and output counts and each
    // output's script confirms both decoders read the same structure
    // out of the same bytes.
    let vectors = materialized();
    for vector in &vectors {
        let reference: Transaction = deserialize(vector.bytes()).expect("the reference decodes");
        let first_party =
            TargetTransaction::decode(vector.bytes()).expect("the first party decodes");

        assert_eq!(
            reference.input.len(),
            first_party.inputs().len(),
            "{:?}: input counts differ",
            vector.id()
        );
        assert_eq!(
            reference.output.len(),
            first_party.outputs().len(),
            "{:?}: output counts differ",
            vector.id()
        );
        for (index, (reference_out, first_party_out)) in reference
            .output
            .iter()
            .zip(first_party.outputs().iter())
            .enumerate()
        {
            assert_eq!(
                reference_out.script_pubkey.as_bytes(),
                first_party_out.program(),
                "{:?}: output {index} programs differ",
                vector.id()
            );
        }
        assert_eq!(
            reference.version,
            first_party.version(),
            "{:?}: versions differ",
            vector.id()
        );
    }
}

#[test]
fn the_reference_txid_is_the_first_party_witness_stripped_digest() {
    // Reference-implementation conformance, not independent evidence.
    //
    // The reference crate computes a txid by hashing the witness-
    // stripped serialization twice. The first party publishes that
    // serialization and computes no digest of it, so hashing it here is
    // the comparison: agreement says `encode_without_witness` really is
    // the txid preimage the target uses.
    let vectors = materialized();
    for vector in &vectors {
        let reference: Transaction = deserialize(vector.bytes()).expect("the reference decodes");
        let first_party =
            TargetTransaction::decode(vector.bytes()).expect("the first party decodes");

        let stripped = sha256d::Hash::hash(&first_party.encode_without_witness());
        assert_eq!(
            stripped.to_byte_array(),
            reference.txid().to_byte_array(),
            "{:?}: the txid preimage is not the witness-stripped encoding",
            vector.id()
        );

        let full = sha256d::Hash::hash(&first_party.encode());
        assert_eq!(
            full.to_byte_array(),
            reference.wtxid().to_byte_array(),
            "{:?}: the wtxid preimage is not the full encoding",
            vector.id()
        );
    }
}

#[test]
fn the_two_weight_rules_agree_on_every_fixture() {
    // Reference-implementation conformance, not independent evidence.
    //
    // The first party computes weight as the witness-stripped length
    // times three plus the total length. The reference crate computes
    // it structurally, field by field, scaling non-witness data by
    // four. These are different formulas over the same transaction and
    // they must land on the same number.
    let vectors = materialized();
    for vector in &vectors {
        let reference: Transaction = deserialize(vector.bytes()).expect("the reference decodes");
        let first_party =
            TargetTransaction::decode(vector.bytes()).expect("the first party decodes");

        assert_eq!(
            u64::try_from(reference.weight()).expect("a fixture weight fits"),
            first_party.weight(),
            "{:?}: the weight rules disagree",
            vector.id()
        );
        assert_eq!(
            u64::try_from(reference.vsize()).expect("a fixture vsize fits"),
            first_party.virtual_size(),
            "{:?}: the virtual-size rules disagree",
            vector.id()
        );
        // The materialized record must carry the same numbers it was
        // built with; a record that drifted from its own bytes would
        // make every downstream limit check meaningless.
        assert_eq!(first_party.weight(), vector.weight(), "{:?}", vector.id());
        assert_eq!(
            first_party.virtual_size(),
            vector.virtual_size(),
            "{:?}",
            vector.id()
        );
    }
}

#[test]
fn the_fixture_transactions_stay_distinct_under_the_reference_digest() {
    // Reference-implementation conformance, not independent evidence.
    //
    // Nine fixtures must be nine transactions. A collision under the
    // reference txid would mean one vector could stand in for another
    // when a node reports on it.
    let vectors = materialized();
    let digests: BTreeSet<[u8; 32]> = vectors
        .iter()
        .map(|vector| {
            let reference: Transaction =
                deserialize(vector.bytes()).expect("the reference decodes");
            reference.txid().to_byte_array()
        })
        .collect();
    assert_eq!(
        digests.len(),
        SPONSORLESS_FIXTURES,
        "two fixtures share a reference txid"
    );
}

// --- (b) Taproot commitment ------------------------------------------

#[test]
fn every_leaf_hash_matches_the_reference_tapleaf_hash() {
    // Reference-implementation conformance, not independent evidence.
    //
    // The first party hashes a leaf from the reviewed tag string and
    // the compact-size framing it read from the target's source. The
    // reference crate hashes it under its own `TapLeaf/elements` tag.
    // Byte equality per leaf is what says the framing is right.
    let tree = committed_tree();
    let version = ReferenceLeafVersion::from_u8(REVIEWED_LEAF_VERSION)
        .expect("the reviewed leaf version is a leaf version");
    assert_eq!(
        tree.leaf_version().get(),
        REVIEWED_LEAF_VERSION,
        "the fixture tree no longer commits under the reviewed leaf version"
    );

    let mut compared = 0_usize;
    for (role, script) in tree.leaf_programs() {
        let first_party = tree
            .leaf_hashes()
            .get(role)
            .expect("every committed leaf has a hash");
        let reference = TapLeafHash::from_script(&Script::from(script.clone()), version);
        assert_eq!(
            reference.to_byte_array(),
            *first_party,
            "{role:?}: the leaf hashes differ"
        );
        compared += 1;
    }
    assert_eq!(
        compared, COMMITTED_LEAVES,
        "the committed leaf set changed size, so the cross-check no longer covers what it says"
    );
}

#[test]
fn every_leaf_path_folds_to_the_first_party_root_under_reference_branch_hashing() {
    // Reference-implementation conformance, not independent evidence.
    //
    // The first party built each path and the root with its own
    // lexicographic branch hashing. Folding the same path through the
    // reference crate's own control-block verification arithmetic must
    // reach the same root, or one of the two orderings is wrong.
    let tree = committed_tree();
    let version = ReferenceLeafVersion::from_u8(REVIEWED_LEAF_VERSION)
        .expect("the reviewed leaf version is a leaf version");
    let root = tree.merkle_root();

    for (role, script) in tree.leaf_programs() {
        let path = tree.paths().get(role).expect("every leaf has a path");
        let leaf = TapLeafHash::from_script(&Script::from(script.clone()), version);
        let mut current = TapNodeHash::from_byte_array(leaf.to_byte_array());
        for sibling in path {
            current = reference_branch(current, TapNodeHash::from_byte_array(*sibling));
        }
        assert_eq!(
            current.to_byte_array(),
            root,
            "{role:?}: the path does not fold to the first-party root"
        );
    }
}

#[test]
fn the_pinned_merkle_root_is_the_tree_the_fixture_bundle_links() {
    // Reference-implementation conformance, not independent evidence.
    //
    // The pin exists so that a tree change fails here rather than
    // downstream, where it would surface as an output key nobody can
    // spend.
    assert_eq!(
        committed_tree().merkle_root(),
        FIXTURE_MERKLE_ROOT,
        "the fixture tree changed; the pinned reference vectors are stale"
    );
}

#[test]
fn every_control_block_parses_and_carries_the_reviewed_leaf_version() {
    // Reference-implementation conformance, not independent evidence.
    //
    // A control block the reference parser rejects is one no node will
    // read, whatever the first party believes about its layout.
    let tree = committed_tree();
    let pin = fixture_pin();
    for role in tree.leaf_programs().keys() {
        let bytes = tree
            .control_block(*role, &pin)
            .expect("every committed leaf has a control block");
        let parsed = ControlBlock::from_slice(&bytes)
            .unwrap_or_else(|error| panic!("{role:?}: the reference parser refused: {error}"));
        assert_eq!(
            parsed.leaf_version.as_u8(),
            REVIEWED_LEAF_VERSION,
            "{role:?}: the control block does not carry the reviewed leaf version"
        );
        assert_eq!(
            parsed.internal_key.serialize(),
            INTERNAL_KEY,
            "{role:?}: the control block names a different internal key"
        );
        assert_eq!(
            parsed.serialize(),
            bytes,
            "{role:?}: the reference re-serialization is not the first-party bytes"
        );
    }
}

// --- (c) Output key ---------------------------------------------------

#[test]
fn the_reference_output_key_is_the_pinned_reference_vector() {
    // Reference-implementation conformance, not independent evidence.
    //
    // This does NOT discharge `PinnedOutputKeyUnverifiedAgainstTree`.
    // Discharging it needs the funding ceremony against a real node —
    // an output actually created at this program and a spend the node
    // accepts. What this does is give the pin a reference value, so
    // that tree-construction drift is caught here instead of in a
    // later wave.
    let (key, parity) = reference_output_key();
    assert_eq!(
        key, FIXTURE_REFERENCE_OUTPUT_KEY,
        "the reference output key moved; recorded vector is stale"
    );
    assert_eq!(
        parity, FIXTURE_REFERENCE_OUTPUT_KEY_PARITY_BIT,
        "the reference output key parity moved"
    );
}

#[test]
fn every_leaf_verifies_against_the_reference_output_key() {
    // Reference-implementation conformance, not independent evidence.
    //
    // The strongest check the reference crate offers: its own
    // `verify_taproot_commitment` recomputes the leaf hash, folds the
    // path, applies the tweak, and checks the tweaked key — all inside
    // reference code, against a control block the first party built.
    //
    // The control blocks are the fixture's own, unmodified. An earlier
    // revision had to rebuild them with the reference-derived parity
    // because the fixture declared a parity it had not derived; the pin
    // now carries the derived bit, so rebuilding would only weaken what
    // this test covers.
    let tree = committed_tree();
    let pin = fixture_pin();
    let secp = Secp256k1::verification_only();
    let (key, _) = reference_output_key();
    let internal = XOnlyPublicKey::from_slice(&INTERNAL_KEY).expect("the internal key lifts");
    let output_key = XOnlyPublicKey::from_slice(&key).expect("the output key lifts");
    let output_key = elements::schnorr::TweakedPublicKey::new(output_key);

    let mut verified = 0_usize;
    for (role, script) in tree.leaf_programs() {
        let bytes = tree
            .control_block(*role, &pin)
            .expect("every committed leaf has a control block");
        let block = ControlBlock::from_slice(&bytes).expect("the control block parses");
        assert_eq!(
            block.internal_key, internal,
            "{role:?}: the control block names a different internal key"
        );
        assert!(
            block.verify_taproot_commitment(&secp, &output_key, &Script::from(script.clone())),
            "{role:?}: the reference verifier rejects the first-party commitment"
        );
        verified += 1;
    }
    assert_eq!(
        verified, COMMITTED_LEAVES,
        "fewer leaves were verified than the tree carries"
    );
}

#[test]
fn the_fixture_pin_is_the_output_key_this_tree_derives() {
    // Reference-implementation conformance, not independent evidence.
    //
    // This is the guard that makes the fixture pin *derived* rather
    // than declared, and it is the only thing standing between the two
    // sides.
    //
    // `vectors::bundle::PINNED_PROGRAM` is the witness program every
    // fixture ASH input pays to and the successor pays back to. It
    // states the taproot output key of the fixture's own committed tree
    // as a literal, because the vectors package must not depend on this
    // one — §16.2 fixes that direction and this package dev-depends on
    // vectors, so the reverse edge would close a cycle. A literal on
    // the far side of a boundary is only as good as the check that it
    // still equals what it claims to be, and this is that check: the
    // key is recomputed here from the fixture's internal key and its
    // merkle root, through the reference bindings, and compared.
    //
    // The two halves an earlier revision got wrong are both covered.
    // The program must be a curve point at all — a fixed byte pattern
    // is not, and an output at one is unspendable by construction,
    // since no control block can satisfy a commitment check against a
    // program that is not a key. And the parity must be the derived
    // bit, not a chosen one: a control block states that bit, an
    // x-only program cannot, and a wrong guess is rejected for every
    // leaf even when every hash in the path is right.
    let (key, parity) = reference_output_key();

    let pinned = XOnlyPublicKey::from_slice(&PINNED_PROGRAM)
        .expect("the pinned program is a curve point; an output at it must be spendable at all");
    assert_eq!(
        pinned.serialize(),
        PINNED_PROGRAM,
        "the pinned program does not round-trip through the reference key parser"
    );
    assert_eq!(
        PINNED_PROGRAM, key,
        "the pinned program is not the output key this tree derives; the fixture pin has drifted from the tree"
    );
    assert_eq!(
        PINNED_PROGRAM, FIXTURE_REFERENCE_OUTPUT_KEY,
        "the pinned program and the recorded reference vector disagree"
    );

    assert_eq!(
        fixture_pin().parity().bit(),
        parity,
        "the fixture's pinned parity is not the derived one; its control blocks state the wrong y coordinate"
    );

    // None of this discharges `PinnedOutputKeyUnverifiedAgainstTree`.
    // A derived pin makes the funding ceremony *possible*; discharging
    // still needs a real node to have created an output at this program
    // and accepted a spend of it.
}

// --- (d) Confidential primitives --------------------------------------

#[test]
fn the_first_party_asset_generators_match_the_reference_library() {
    // Reference-implementation conformance, not independent evidence.
    //
    // The first-party oracle derives an asset generator from the
    // published curve map over this workspace's bignum; the reference
    // library derives it in C. Byte equality of the 33-byte
    // serialization covers the map, the two generations, the sum, and
    // the prefix convention at once.
    let secp = Secp256k1::signing_only();
    let mut compared = 0_usize;
    for asset in fixture_assets() {
        let first_party = crate::commitment_oracle::generator::serialized_asset_generator(&asset)
            .expect("the first-party oracle derives the generator");
        let reference =
            secp256k1_zkp::Generator::new_unblinded(&secp, secp256k1_zkp::Tag::from(asset))
                .serialize();
        assert_eq!(
            first_party, reference,
            "the asset generators differ for {asset:02x?}"
        );
        compared += 1;
    }
    assert_eq!(compared, fixture_assets().len());
}

#[test]
fn the_first_party_commitments_match_the_reference_library() {
    // Reference-implementation conformance, not independent evidence.
    //
    // A Pedersen commitment over an explicit asset's generator,
    // computed first-party as `rG + vH` and by the reference library in
    // C. The blinding factors are public fixtures
    // `(´[ADR015-rule:security:test-material]´)`.
    let secp = Secp256k1::signing_only();
    let mut compared = 0_usize;
    for asset in fixture_assets() {
        for (amount, blinder) in fixture_openings() {
            let first_party =
                crate::commitment_oracle::commitment::commitment(&asset, amount, &blinder)
                    .expect("the first-party oracle commits");
            let generator =
                secp256k1_zkp::Generator::new_unblinded(&secp, secp256k1_zkp::Tag::from(asset));
            let reference = secp256k1_zkp::PedersenCommitment::new(
                &secp,
                amount,
                secp256k1_zkp::Tweak::from_inner(blinder).expect("the blinder is a scalar"),
                generator,
            )
            .serialize();
            assert_eq!(
                first_party, reference,
                "the commitments differ for {asset:02x?} at {amount}"
            );
            compared += 1;
        }
    }
    assert_eq!(
        compared,
        fixture_assets().len() * fixture_openings().len(),
        "the commitment cross-check covered fewer pairs than it states"
    );
}

// --- Shared helpers ---------------------------------------------------

/// The reference crate's own branch hashing over two nodes.
///
/// Folding a path through `ControlBlock::verify_taproot_commitment`
/// would be the purer route, but it needs an output key, which is the
/// thing a root check must not presuppose. So this uses the reference
/// crate's own `TapBranch/elements` engine, with the same ordering rule
/// that function applies: the tag and the framing come from the
/// reference side, and only the two-line ordering is restated here.
fn reference_branch(left: TapNodeHash, right: TapNodeHash) -> TapNodeHash {
    use elements::hashes::{HashEngine as _, sha256t};
    let mut engine = sha256t::Hash::<elements::taproot::TapBranchTag>::engine();
    if left.to_byte_array() < right.to_byte_array() {
        engine.input(left.as_ref());
        engine.input(right.as_ref());
    } else {
        engine.input(right.as_ref());
        engine.input(left.as_ref());
    }
    TapNodeHash::from_byte_array(
        sha256t::Hash::<elements::taproot::TapBranchTag>::from_engine(engine).to_byte_array(),
    )
}

/// The fixture output key and its parity bit, from the reference crate.
fn reference_output_key() -> ([u8; REFERENCE_KEY_BYTES], u8) {
    let secp = Secp256k1::verification_only();
    let internal = XOnlyPublicKey::from_slice(&INTERNAL_KEY).expect("the internal key lifts");
    let root = TapNodeHash::from_byte_array(committed_tree().merkle_root());
    let (tweaked, parity) = internal.tap_tweak(&secp, Some(root));
    let key: XOnlyPublicKey = tweaked.into();
    (
        key.serialize(),
        u8::from(parity == secp256k1_zkp::Parity::Odd),
    )
}

/// The asset identifiers the fixture bundle uses.
fn fixture_assets() -> Vec<[u8; 32]> {
    vec![
        vectors::bundle::CLOSED_ASSET,
        vectors::bundle::RESERVE_ASSET,
    ]
}

/// Public disposable openings for the commitment cross-check
/// `(´[ADR015-rule:security:test-material]´)`.
fn fixture_openings() -> Vec<(u64, [u8; 32])> {
    vec![(0, [0x11; 32]), (1, [0x22; 32]), (21_000_000, [0x33; 32])]
}

/// Guard: the cross-check must not lose a leaf silently.
#[test]
fn the_cross_checked_leaf_set_is_the_whole_committed_tree() {
    let tree = committed_tree();
    let programs: BTreeSet<&LeafRole> = tree.leaf_programs().keys().collect();
    let hashes: BTreeSet<&LeafRole> = tree.leaf_hashes().keys().collect();
    let paths: BTreeSet<&LeafRole> = tree.paths().keys().collect();
    assert_eq!(programs, hashes, "a leaf has a program but no hash");
    assert_eq!(programs, paths, "a leaf has a program but no path");

    // Every distinct leaf must produce a distinct hash, or the tree
    // would commit to one program twice and a control block would
    // authenticate the wrong leaf.
    let distinct: BTreeSet<&[u8; 32]> = tree.leaf_hashes().values().collect();
    let by_role: BTreeMap<&LeafRole, &[u8; 32]> = tree.leaf_hashes().iter().collect();
    assert_eq!(
        distinct.len(),
        by_role.len(),
        "two committed leaves share a hash"
    );
    assert_eq!(by_role.len(), COMMITTED_LEAVES, "the leaf count moved");
}
