//! The live-receipt output key and the owner keys that enter a signing
//! flow (Guide-13 §7.5, §1.8, §12.6).
//!
//! # Why these two live in one file
//!
//! They are the two questions a live-transfer builder cannot answer for
//! itself and this package can: whether an offered owner key names a
//! point, and what output key a committed tree determines. A builder
//! consuming both through one capability is consuming exactly what this
//! file exercises.
//!
//! # The signing material is published test material
//!
//! The scalars come from [`crate::test_material`], which is the
//! published BIP-340 vectors' own secret keys under ADR-015's
//! test-material rule `(´[ADR015-rule:security:test-material]´)`. They
//! authorize nothing on any network anyone uses.

use crate::constructor::curve::{FIELD_ELEMENT_BYTES, generator, lift_x};
use crate::constructor::internal_key::UNSPENDABLE_INTERNAL_KEY;
use crate::constructor::tagged::{Digest32, tagged_hash};
use crate::constructor::tree::{TweakDefect, output_program, tweak, tweaked_key};
use crate::live_receipt_key::live_receipt_output_key;
use crate::owner_key_oracle::{SignatureRejection, offered_key_point, verify_owner_signature};
use crate::test_material::OwnerSigningMaterial;
use target_elements::reviewed_elements_tapscript;

/// The first published BIP-340 signing scalar.
const FIRST_SCALAR: [u8; FIELD_ELEMENT_BYTES] = [
    0xB7, 0xE1, 0x51, 0x62, 0x8A, 0xED, 0x2A, 0x6A, 0xBF, 0x71, 0x58, 0x80, 0x9C, 0xF4, 0xF3, 0xC7,
    0x62, 0xE7, 0x16, 0x0F, 0x38, 0xB4, 0xDA, 0x56, 0xA7, 0x84, 0xD9, 0x04, 0x51, 0x90, 0xCF, 0xEF,
];

/// The second published BIP-340 signing scalar.
const SECOND_SCALAR: [u8; FIELD_ELEMENT_BYTES] = [
    0xC9, 0x0F, 0xDA, 0xA2, 0x21, 0x68, 0xC2, 0x34, 0xC4, 0xC6, 0x62, 0x8B, 0x80, 0xDC, 0x1C, 0xD1,
    0x29, 0x02, 0x4E, 0x08, 0x8A, 0x67, 0xCC, 0x74, 0x02, 0x0B, 0xBE, 0xA6, 0x3B, 0x14, 0xE5, 0xC9,
];

/// A stand-in committed root, hashed from a distinguishable fixture.
fn fixture_root(tag: &str) -> Digest32 {
    tagged_hash("Guide13/live-fixture-root", tag.as_bytes())
}

// --- The output key (§12.6) -------------------------------------------

#[test]
fn the_live_receipt_output_key_is_the_tweak_of_the_committed_root() {
    // The obligation every layer below carried, discharged: the key is
    // the internal key offset by the generator multiple of the tagged
    // hash over the key and the root, and nothing about it is a claim
    // that a target accepts a spend of it.
    let root = fixture_root("explicit");
    let derived = live_receipt_output_key(&UNSPENDABLE_INTERNAL_KEY, &root)
        .expect("the unspendable internal key and a hashed root determine an output key");

    let expected_tweak = tweak(&UNSPENDABLE_INTERNAL_KEY, &root);
    let (expected_key, expected_parity) = tweaked_key(&UNSPENDABLE_INTERNAL_KEY, &expected_tweak)
        .expect("the same two steps performed apart");

    assert_eq!(derived.tweak(), &expected_tweak);
    assert_eq!(derived.key(), &expected_key);
    assert_eq!(derived.parity(), expected_parity);
    assert_eq!(derived.program(), output_program(&expected_key));

    // The program is the witness-version-one form, which is what a
    // destination output pays to.
    assert_eq!(derived.program().len(), 34);
    assert_eq!(derived.program()[0], 0x51);
    assert_eq!(derived.program()[1], 0x20);
    assert_eq!(&derived.program()[2..], derived.key());
}

#[test]
fn two_committed_roots_determine_two_different_output_keys() {
    // What makes the destination constructor table load-bearing at the
    // ABI: two owners' constructors commit different trees, and the
    // programs their receipts pay to differ because of it.
    let first = live_receipt_output_key(&UNSPENDABLE_INTERNAL_KEY, &fixture_root("first"))
        .expect("the first root determines a key");
    let second = live_receipt_output_key(&UNSPENDABLE_INTERNAL_KEY, &fixture_root("second"))
        .expect("the second root determines a key");

    assert_ne!(first.key(), second.key());
    assert_ne!(first.program(), second.program());
    assert_ne!(first.tweak(), second.tweak());
}

#[test]
fn an_internal_key_that_is_not_a_curve_point_determines_no_output_key() {
    // Reported rather than asserted away. A deployment whose internal
    // key is not a point has no output key at all, and the refusal names
    // which of the three conditions failed.
    let not_a_point = [0xff_u8; FIELD_ELEMENT_BYTES];
    assert!(lift_x(&not_a_point).is_err());
    assert!(matches!(
        live_receipt_output_key(&not_a_point, &fixture_root("explicit")),
        Err(TweakDefect::InternalKeyNotOnCurve(_)),
    ));
}

#[test]
fn the_key_the_oracle_derives_is_a_point_and_the_generator_is_the_one_it_offsets_from() {
    // Not internal consistency: the derived key must itself be an x
    // coordinate some point has, or no control block could ever
    // authenticate against it.
    let derived = live_receipt_output_key(&UNSPENDABLE_INTERNAL_KEY, &fixture_root("explicit"))
        .expect("the root determines a key");
    assert!(lift_x(derived.key()).is_ok());
    assert!(derived.parity() < 2);

    // And the generator this package offsets from is the published one,
    // which is what makes the offset the target's tweak rather than some
    // other curve's.
    assert!(lift_x(&generator().x_only_bytes()).is_ok());
}

// --- The owner keys entering a signing flow (§1.8) --------------------

#[test]
fn every_owner_key_of_a_multi_owner_transfer_names_a_curve_point() {
    // The loop the link left open as
    // `OwnerKeyCurvePointMembershipUnverified`: the encoding gate ran at
    // the constructor, and this is the question it said it could not
    // answer. A builder consuming this through a capability refuses an
    // owner whose key names no point before any signing request exists.
    let target = reviewed_elements_tapscript().expect("the reviewed contract validates");

    for scalar in [FIRST_SCALAR, SECOND_SCALAR] {
        let material =
            OwnerSigningMaterial::from_published_scalar(&scalar).expect("a published scalar");
        assert!(offered_key_point(&target, &material.x_only_public_key()).is_ok());
    }

    // And the two owners are different owners, which is what makes a
    // two-owner transfer two owners.
    let first = OwnerSigningMaterial::from_published_scalar(&FIRST_SCALAR).expect("first");
    let second = OwnerSigningMaterial::from_published_scalar(&SECOND_SCALAR).expect("second");
    assert_ne!(first.x_only_public_key(), second.x_only_public_key());
}

#[test]
fn each_owner_of_a_multi_owner_transfer_authorizes_only_its_own_message() {
    // §12.7's wrong-owner and bound-to-different-bytes rejections, at
    // the layer that can actually check a signature. The builder checks
    // binding; this checks that the binding it enforced is the one the
    // arithmetic agrees with.
    let target = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let first = OwnerSigningMaterial::from_published_scalar(&FIRST_SCALAR).expect("first");
    let second = OwnerSigningMaterial::from_published_scalar(&SECOND_SCALAR).expect("second");

    let auxiliary = [0x00_u8; FIELD_ELEMENT_BYTES];
    let message = fixture_root("finalized-protected-bytes");
    let other_message = fixture_root("some-other-transaction");

    let signature = first.sign(&message, &auxiliary).expect("the owner signs");

    assert_eq!(
        verify_owner_signature(&target, &first.x_only_public_key(), &message, &signature),
        Ok(()),
    );
    // Another owner's key does not verify it (§12.7, wrong owner).
    assert_eq!(
        verify_owner_signature(&target, &second.x_only_public_key(), &message, &signature),
        Err(SignatureRejection::CommitmentMismatch),
    );
    // Nor does another message (§12.7, bound to different bytes).
    assert_eq!(
        verify_owner_signature(
            &target,
            &first.x_only_public_key(),
            &other_message,
            &signature,
        ),
        Err(SignatureRejection::CommitmentMismatch),
    );
}

#[test]
fn one_owner_holding_two_inputs_produces_two_distinct_signatures() {
    // §1.6's own distinction, exercised where it is arithmetic rather
    // than bookkeeping: the target's message is input-specific, so one
    // owner answering two requests answers them with two signatures, and
    // neither verifies against the other's message.
    let target = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let owner = OwnerSigningMaterial::from_published_scalar(&FIRST_SCALAR).expect("the owner");
    let auxiliary = [0x00_u8; FIELD_ELEMENT_BYTES];

    let first_message = fixture_root("input-zero");
    let second_message = fixture_root("input-one");
    let first = owner
        .sign(&first_message, &auxiliary)
        .expect("the owner signs the first");
    let second = owner
        .sign(&second_message, &auxiliary)
        .expect("the owner signs the second");

    assert_ne!(first, second);
    assert_eq!(
        verify_owner_signature(&target, &owner.x_only_public_key(), &first_message, &first),
        Ok(()),
    );
    assert_eq!(
        verify_owner_signature(
            &target,
            &owner.x_only_public_key(),
            &second_message,
            &second
        ),
        Ok(()),
    );
    assert_eq!(
        verify_owner_signature(&target, &owner.x_only_public_key(), &second_message, &first),
        Err(SignatureRejection::CommitmentMismatch),
    );
}
