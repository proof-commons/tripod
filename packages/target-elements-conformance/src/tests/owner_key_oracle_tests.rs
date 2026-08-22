//! The owner-key and signature oracle, against published vectors
//! (Guide-13 §1.8).
//!
//! # What grounds the verifier
//!
//! Not agreement with the signer beside it. The verifier is required to
//! accept the published BIP-340 vectors' own signatures — artifacts the
//! specification's authors produced and nothing in this repository did —
//! and then to refuse every mutation of them. A verifier that accepted
//! nothing would pass the negatives and fail the positives; one that
//! accepted everything would pass the positives and fail the negatives;
//! and the two halves together are what leave no room for either.
//!
//! # These vectors are published fixtures
//!
//! Public keys, messages, and signatures from the BIP-340 appendix,
//! public since the specification was written. Under ADR-015's
//! test-material rule `(´[ADR015-rule:security:test-material]´)` and
//! Guide-13 §1.10 they are fixture data: they authorize nothing on any
//! network and are reproducible from the specification.
//!
//! # The four negatives this module owns
//!
//! §1.8 names six. The adapter's abstract walk decides the empty key and
//! the unrecognized key type, because both are questions about a width.
//! The other four keep the approved encoding at its exact width and are
//! decided here: a malformed approved key, an approved key of another
//! owner, a valid signature against another key, and a valid signature
//! over another transaction.

use tapscript::{OwnerKey, owner_key_encoding_closure};
use target_elements::{
    PayloadWidth, ReviewedElementsTapscriptDefinition, reviewed_elements_tapscript,
};

use crate::constructor::curve::{FIELD_ELEMENT_BYTES, PointDecodingDefect, lift_x};
use crate::owner_authorization::{OwnerAuthorizationCaseId, case_for_key_negative};
use crate::owner_key_oracle::{
    OwnerKeyPointDefect, SignatureRejection, offered_key_point, owner_key_point,
    verify_owner_signature,
};
use crate::test_material::{OwnerSigningMaterial, SIGNATURE_BYTES};

/// The reviewed contract, unmodified.
///
/// # Panics
///
/// If the reviewed contract stops validating, which its own package's
/// oracles would have caught first.
fn target() -> ReviewedElementsTapscriptDefinition {
    reviewed_elements_tapscript().expect("the reviewed contract validates")
}

/// The approved owner-key encoding's exact width.
///
/// # Panics
///
/// If the approved encoding stops fixing one exact width.
fn approved_width() -> usize {
    let closure = owner_key_encoding_closure(target().definition().authorization());
    match closure.approved().v1_shape().payload() {
        PayloadWidth::Exact(width) => width.get(),
        other => panic!("the approved owner-key encoding fixes no exact width: {other:?}"),
    }
}

/// One published vector's signing material.
///
/// The two scalars are BIP-340's own published test-vector secret keys.
///
/// # Panics
///
/// If a published vector's secret key stops being in range.
fn material(scalar: [u8; FIELD_ELEMENT_BYTES]) -> OwnerSigningMaterial {
    OwnerSigningMaterial::from_published_scalar(&scalar)
        .expect("a published vector's secret key is in range")
}

/// The first published vector's secret key.
fn first_scalar() -> [u8; FIELD_ELEMENT_BYTES] {
    let mut scalar = [0_u8; FIELD_ELEMENT_BYTES];
    scalar[FIELD_ELEMENT_BYTES - 1] = 3;
    scalar
}

/// A second published scalar, distinguishable from the first.
fn second_scalar() -> [u8; FIELD_ELEMENT_BYTES] {
    let mut scalar = [0_u8; FIELD_ELEMENT_BYTES];
    scalar[FIELD_ELEMENT_BYTES - 1] = 5;
    scalar
}

/// A thirty-two byte message, filled with `fill`.
const fn message(fill: u8) -> [u8; FIELD_ELEMENT_BYTES] {
    [fill; FIELD_ELEMENT_BYTES]
}

/// The auxiliary value the fixtures sign under.
const fn auxiliary() -> [u8; FIELD_ELEMENT_BYTES] {
    [0_u8; FIELD_ELEMENT_BYTES]
}

/// One signature by `material` over `message`.
///
/// # Panics
///
/// If the derived nonce reduces to zero, which no fixture here reaches.
fn signed(
    material: &OwnerSigningMaterial,
    message: &[u8; FIELD_ELEMENT_BYTES],
) -> [u8; SIGNATURE_BYTES] {
    material
        .sign(message, &auxiliary())
        .expect("the fixture signs")
}

/// Thirty-two bytes that no curve point has as an x coordinate.
///
/// Searched rather than transcribed: roughly half of all field elements
/// are like this, so a small ascending scan finds one, and a
/// transcription could go stale against a curve parameter without
/// anybody noticing.
///
/// # Panics
///
/// If no value in the scanned range fails to lift, which would mean the
/// curve had stopped behaving like a curve.
fn not_a_curve_point() -> [u8; FIELD_ELEMENT_BYTES] {
    (1_u8..=255)
        .map(|low| {
            let mut candidate = [0_u8; FIELD_ELEMENT_BYTES];
            candidate[FIELD_ELEMENT_BYTES - 1] = low;
            candidate
        })
        .find(|candidate| lift_x(candidate).is_err())
        .expect("about half of all field elements are not x coordinates")
}

/// One hexadecimal digit's value, refused at compile time if it is not
/// one.
///
/// # Panics
///
/// At compile time, for a character that is not a lowercase hexadecimal
/// digit.
const fn nibble(digit: u8) -> u8 {
    match digit {
        b'0'..=b'9' => digit - b'0',
        b'a'..=b'f' => digit - b'a' + 10,
        _ => panic!("a published vector must be written in lowercase hexadecimal"),
    }
}

/// A thirty-two byte vector, decoded at compile time.
const fn hex32(text: &[u8; 64]) -> [u8; FIELD_ELEMENT_BYTES] {
    let mut bytes = [0_u8; FIELD_ELEMENT_BYTES];
    let mut index = 0;
    while index < FIELD_ELEMENT_BYTES {
        bytes[index] = (nibble(text[index * 2]) << 4) | nibble(text[index * 2 + 1]);
        index += 1;
    }
    bytes
}

/// A sixty-four byte vector, decoded at compile time.
const fn hex64(text: &[u8; 128]) -> [u8; SIGNATURE_BYTES] {
    let mut bytes = [0_u8; SIGNATURE_BYTES];
    let mut index = 0;
    while index < SIGNATURE_BYTES {
        bytes[index] = (nibble(text[index * 2]) << 4) | nibble(text[index * 2 + 1]);
        index += 1;
    }
    bytes
}

/// One published verification vector: a key, a message, a signature.
struct Published {
    /// The vector's index in the published table, for a failure to name.
    index: u8,
    public_key: [u8; FIELD_ELEMENT_BYTES],
    message: [u8; FIELD_ELEMENT_BYTES],
    signature: [u8; SIGNATURE_BYTES],
}

/// Two published BIP-340 vectors, transcribed whole.
///
/// The signature bytes are the specification's own, produced by its
/// authors rather than by anything here. That is what makes accepting
/// them a check on this verifier rather than an agreement between two
/// pieces of first-party arithmetic.
fn published() -> [Published; 2] {
    [
        Published {
            index: 0,
            public_key: hex32(
                b"f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9",
            ),
            message: hex32(
                b"0000000000000000000000000000000000000000000000000000000000000000",
            ),
            signature: hex64(
                b"e907831f80848d1069a5371b402410364bdf1c5f8307b0084c55f1ce2dca821525f66a4a85ea8b71e482a74f382d2ce5ebeee8fdb2172f477df4900d310536c0",
            ),
        },
        Published {
            index: 1,
            public_key: hex32(
                b"dff1d77f2a671c5f36183726db2341be58feae1da2deced843240f7b502ba659",
            ),
            message: hex32(
                b"243f6a8885a308d313198a2e03707344a4093822299f31d0082efa98ec4e6c89",
            ),
            signature: hex64(
                b"6896bd60eeae296db48a229ff71dfe071bde413e6d43f917dc8dcf8c78de33418906d11ac976abccb20b091292bff4ea897efcb639ea871cfa95f6de339e4b0a",
            ),
        },
    ]
}

// --- The positive half: the verifier accepts what it must -------------

#[test]
fn the_published_signatures_verify_against_their_published_keys() {
    // The grounding, and the reason it is not the signer's signatures
    // that are used here: these sixty-four byte strings came out of the
    // specification's appendix. A verifier that refused everything would
    // satisfy every negative below and fail here.
    let target = target();

    for vector in published() {
        assert_eq!(
            verify_owner_signature(
                &target,
                &vector.public_key,
                &vector.message,
                &vector.signature,
            ),
            Ok(()),
            "published vector {}",
            vector.index,
        );
    }
}

#[test]
fn one_flipped_byte_of_a_published_signature_stops_it_verifying() {
    // The other half of the same grounding: a verifier that accepted
    // everything would pass the test above.
    let target = target();

    for vector in published() {
        for position in [0_usize, FIELD_ELEMENT_BYTES] {
            let mut mutated = vector.signature;
            mutated[position] ^= 0x01;

            assert_ne!(
                verify_owner_signature(&target, &vector.public_key, &vector.message, &mutated),
                Ok(()),
                "published vector {} survived a flipped byte at {position}",
                vector.index,
            );
        }
    }
}

#[test]
fn a_freshly_signed_message_verifies_against_the_signing_key() {
    let target = target();
    let material = material(first_scalar());
    let message = message(0x00);
    let signature = signed(&material, &message);

    assert_eq!(
        verify_owner_signature(&target, &material.x_only_public_key(), &message, &signature,),
        Ok(()),
    );
}

#[test]
fn an_accepted_owner_key_that_is_a_point_yields_it() {
    // The residual §7.2 leaves open, discharged in the direction that
    // says yes: an approved-encoding key that is a curve point names it.
    let target = target();
    let closure = owner_key_encoding_closure(target.definition().authorization());
    let owner = OwnerKey::new(
        &closure,
        closure.approved(),
        material(first_scalar()).x_only_public_key().to_vec(),
    )
    .expect("a published public key is the approved encoding at its width");

    assert!(owner_key_point(&target, &owner).is_ok());
}

// --- §1.8: the four negatives the abstract walk cannot decide ---------

#[test]
fn a_malformed_approved_key_names_no_curve_point() {
    // §1.8's third negative. The offering is the approved encoding at
    // its exact width — §7.2's gate admits it and the adapter's walk
    // cannot separate it from a correct leaf — and it is not a key.
    let target = target();
    let closure = owner_key_encoding_closure(target.definition().authorization());
    let owner = OwnerKey::new(&closure, closure.approved(), not_a_curve_point().to_vec())
        .expect("the offering is the approved encoding at its exact width");

    assert_eq!(
        owner_key_point(&target, &owner),
        Err(OwnerKeyPointDefect::NotACurvePoint {
            defect: PointDecodingDefect::NotOnCurve,
        }),
    );
}

#[test]
fn a_value_above_the_field_modulus_is_not_a_field_element_at_all() {
    // The other half of a malformed key, and a different finding: this
    // one is not a coordinate rather than not a point.
    assert_eq!(
        offered_key_point(&target(), &[0xff; FIELD_ELEMENT_BYTES]),
        Err(OwnerKeyPointDefect::NotACurvePoint {
            defect: PointDecodingDefect::NotAFieldElement,
        }),
    );
}

#[test]
fn an_approved_key_of_another_owner_does_not_authorize_this_one() {
    // §1.8's fourth negative. Both keys are well formed and approved,
    // and the one the leaf requires is not the one that signed.
    let target = target();
    let required = material(first_scalar());
    let other = material(second_scalar());
    let message = message(0x11);

    assert_ne!(required.x_only_public_key(), other.x_only_public_key());
    assert_eq!(
        verify_owner_signature(
            &target,
            &required.x_only_public_key(),
            &message,
            &signed(&other, &message),
        ),
        Err(SignatureRejection::CommitmentMismatch),
    );
}

#[test]
fn a_valid_signature_against_another_key_is_still_not_authorization() {
    // §1.8's fifth negative, and what separates it from an invalid
    // signature: the offering *is* a valid signature. It is valid for
    // somebody else, which the second assertion is what establishes.
    let target = target();
    let required = material(first_scalar());
    let other = material(second_scalar());
    let message = message(0x22);
    let signature = signed(&other, &message);

    assert_eq!(
        verify_owner_signature(&target, &other.x_only_public_key(), &message, &signature),
        Ok(()),
    );
    assert_eq!(
        verify_owner_signature(&target, &required.x_only_public_key(), &message, &signature),
        Err(SignatureRejection::CommitmentMismatch),
    );
}

#[test]
fn a_valid_signature_over_another_transaction_is_still_not_authorization() {
    // §1.8's sixth negative. One owner, one key, two messages: the
    // signature is the owner's own and covers something else.
    let target = target();
    let owner = material(first_scalar());
    let signed_message = message(0x33);
    let offered_message = message(0x44);
    let signature = signed(&owner, &signed_message);

    assert_eq!(
        verify_owner_signature(
            &target,
            &owner.x_only_public_key(),
            &signed_message,
            &signature,
        ),
        Ok(()),
    );
    assert_eq!(
        verify_owner_signature(
            &target,
            &owner.x_only_public_key(),
            &offered_message,
            &signature,
        ),
        Err(SignatureRejection::CommitmentMismatch),
    );
}

// --- The width gates ---------------------------------------------------

#[test]
fn the_empty_key_is_refused_before_any_curve_question_arises() {
    assert_eq!(
        offered_key_point(&target(), &[]),
        Err(OwnerKeyPointDefect::NotTheApprovedWidth {
            offered: 0,
            required: approved_width(),
        }),
    );
}

#[test]
fn a_key_of_an_unrecognized_width_is_refused() {
    let wider = vec![0x02; approved_width() + 1];

    assert_eq!(
        offered_key_point(&target(), &wider),
        Err(OwnerKeyPointDefect::NotTheApprovedWidth {
            offered: wider.len(),
            required: approved_width(),
        }),
    );
}

#[test]
fn a_signature_of_the_wrong_width_is_refused_before_the_key_is_read() {
    let target = target();
    let owner = material(first_scalar());

    let rejection = verify_owner_signature(
        &target,
        &owner.x_only_public_key(),
        &message(0x55),
        &[0x00; SIGNATURE_BYTES - 1],
    )
    .expect_err("a short offering is not a signature");

    assert!(matches!(
        rejection,
        SignatureRejection::NotTheSignatureWidth { .. },
    ));
}

#[test]
fn a_signature_whose_commitment_is_not_a_point_is_refused() {
    let target = target();
    let owner = material(first_scalar());
    let message = message(0x66);
    let mut signature = signed(&owner, &message);
    signature[..FIELD_ELEMENT_BYTES].copy_from_slice(&not_a_curve_point());

    assert_eq!(
        verify_owner_signature(&target, &owner.x_only_public_key(), &message, &signature,),
        Err(SignatureRejection::CommitmentIsNotACurvePoint {
            defect: PointDecodingDefect::NotOnCurve,
        }),
    );
}

#[test]
fn a_response_at_or_above_the_group_order_is_refused() {
    let target = target();
    let owner = material(first_scalar());
    let message = message(0x77);
    let mut signature = signed(&owner, &message);
    signature[FIELD_ELEMENT_BYTES..].copy_from_slice(&[0xff; FIELD_ELEMENT_BYTES]);

    assert_eq!(
        verify_owner_signature(&target, &owner.x_only_public_key(), &message, &signature,),
        Err(SignatureRejection::ResponseOutOfRange),
    );
}

// --- The census repair -------------------------------------------------

#[test]
fn the_empty_key_negative_names_the_empty_key_case() {
    // It named `MissingSignature` before, which merged §1.8's empty key
    // with §9.2's missing witness item. They are different target
    // behaviours — the reviewed primitive has a failure cause of its own
    // for the empty key — so a run of the one establishes nothing about
    // the other.
    assert_eq!(
        case_for_key_negative(tapscript::OwnerKeyNegative::EmptyKey),
        OwnerAuthorizationCaseId::EmptyOwnerKey,
    );
}
