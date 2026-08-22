//! The test-only owner-signing capability, against published vectors
//! (Guide-13 §14.3, §1.10).
//!
//! # Why the vectors are the whole test
//!
//! A signer checked against a verifier written beside it is one opinion
//! agreeing with itself, and a signer checked for well-formedness is
//! not checked at all: every one of the sixty-four bytes could be wrong
//! and the shape would still be right. So the comparison is against
//! artifacts nothing in this repository produced — the published BIP-340
//! test vectors, whose secret keys, auxiliary values, messages, and
//! signatures are all in the specification's own appendix, transcribed
//! here and reproduced byte for byte.
//!
//! The census already records what one wrong nibble in a transcribed
//! vector cost, so the transcriptions below are checked against each
//! other in the only way available: each vector's public key is derived
//! from its secret key rather than transcribed alongside it and then
//! compared with the published one, so a mistyped scalar and a mistyped
//! key cannot agree by accident.
//!
//! # These scalars are published fixtures
//!
//! Every scalar in this file is a published BIP-340 test vector's own
//! secret key, public since the specification was written. Under
//! ADR-015's test-material rule
//! `(´[ADR015-rule:security:test-material]´)` and Guide-13 §1.10 they
//! are fixture data and not secret material: they authorize nothing on
//! any network, they are reproducible from the specification rather
//! than generated here, and they must not be reused outside these
//! tests.

use crate::constructor::curve::FIELD_ELEMENT_BYTES;
use crate::test_material::{OwnerSigningMaterial, SIGNATURE_BYTES, TestSigningDefect};

/// One hexadecimal digit's value, refused at compile time if it is not
/// one.
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

/// One published signing vector, transcribed whole.
struct Vector {
    /// The vector's index in the published table, for a failure to name.
    index: u8,
    scalar: [u8; FIELD_ELEMENT_BYTES],
    public_key: [u8; FIELD_ELEMENT_BYTES],
    auxiliary: [u8; FIELD_ELEMENT_BYTES],
    message: [u8; FIELD_ELEMENT_BYTES],
    signature: [u8; SIGNATURE_BYTES],
}

/// The published vectors whose secret key the specification states.
///
/// The first three of the table. Those are the ones that exercise a
/// signer: the later entries publish only a public key and check
/// verification, which is not what this module does.
///
/// Vector two is the one that matters most here. Its secret key's
/// public point has an odd y coordinate, so a signer that skipped the
/// negation step would still produce a well-formed signature and would
/// still reproduce vectors zero and one exactly.
fn vectors() -> [Vector; 3] {
    [
        Vector {
            index: 0,
            scalar: hex32(
                b"0000000000000000000000000000000000000000000000000000000000000003",
            ),
            public_key: hex32(
                b"f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9",
            ),
            auxiliary: hex32(
                b"0000000000000000000000000000000000000000000000000000000000000000",
            ),
            message: hex32(
                b"0000000000000000000000000000000000000000000000000000000000000000",
            ),
            signature: hex64(
                b"e907831f80848d1069a5371b402410364bdf1c5f8307b0084c55f1ce2dca821525f66a4a85ea8b71e482a74f382d2ce5ebeee8fdb2172f477df4900d310536c0",
            ),
        },
        Vector {
            index: 1,
            scalar: hex32(
                b"b7e151628aed2a6abf7158809cf4f3c762e7160f38b4da56a784d9045190cfef",
            ),
            public_key: hex32(
                b"dff1d77f2a671c5f36183726db2341be58feae1da2deced843240f7b502ba659",
            ),
            auxiliary: hex32(
                b"0000000000000000000000000000000000000000000000000000000000000001",
            ),
            message: hex32(
                b"243f6a8885a308d313198a2e03707344a4093822299f31d0082efa98ec4e6c89",
            ),
            signature: hex64(
                b"6896bd60eeae296db48a229ff71dfe071bde413e6d43f917dc8dcf8c78de33418906d11ac976abccb20b091292bff4ea897efcb639ea871cfa95f6de339e4b0a",
            ),
        },
        Vector {
            index: 2,
            scalar: hex32(
                b"c90fdaa22168c234c4c6628b80dc1cd129024e088a67cc74020bbea63b14e5c9",
            ),
            public_key: hex32(
                b"dd308afec5777e13121fa72b9cc1b7cc0139715309b086c960e18fd969774eb8",
            ),
            auxiliary: hex32(
                b"c87aa53824b4d7ae2eb035a2b5bbbccc080e76cdc6d1692c4b0b62d798e6d906",
            ),
            message: hex32(
                b"7e2d58d8b3bcdf1abadec7829054f90dda9805aab56c77333024b9d0a508b75c",
            ),
            signature: hex64(
                b"5831aaeed7b44bb74e5eab94ba9d4294c49bcf2a60728d8b4c200f50dd313c1bab745879a5ad954a72c45a91c3a51d3c7adea98d82f8481e0e1e03674a6f3fb7",
            ),
        },
    ]
}

#[test]
fn the_derived_public_key_is_the_published_one() {
    for vector in vectors() {
        let material = OwnerSigningMaterial::from_published_scalar(&vector.scalar)
            .expect("a published vector's secret key is in range");

        // Derived rather than transcribed alongside the key, so a
        // mistyped scalar and a mistyped key cannot agree by accident.
        assert_eq!(
            material.x_only_public_key(),
            vector.public_key,
            "vector {}",
            vector.index,
        );
    }
}

#[test]
fn each_published_vector_is_reproduced_byte_for_byte() {
    for vector in vectors() {
        let material = OwnerSigningMaterial::from_published_scalar(&vector.scalar)
            .expect("a published vector's secret key is in range");
        let produced = material
            .sign(&vector.message, &vector.auxiliary)
            .expect("a published vector signs");

        assert_eq!(produced, vector.signature, "vector {}", vector.index);
    }
}

#[test]
fn signing_is_deterministic_in_its_stated_inputs() {
    // §1.10 asks test material to be reproducible from explicit test
    // inputs where the materializer permits it. Nothing here consults a
    // clock or a random source, so two calls with the same message and
    // auxiliary value must agree — and the vector comparison above
    // would not catch a signer that had become deterministic in the
    // wrong thing.
    let vector = &vectors()[1];
    let material = OwnerSigningMaterial::from_published_scalar(&vector.scalar).expect("in range");
    let once = material
        .sign(&vector.message, &vector.auxiliary)
        .expect("signs");
    let again = material
        .sign(&vector.message, &vector.auxiliary)
        .expect("signs");

    assert_eq!(once, again);
}

#[test]
fn a_different_message_or_auxiliary_value_signs_differently() {
    // The property every mutation case rests on: the signature is a
    // function of the message, so a transaction whose protected data
    // changed after signing does not carry the signature that was
    // taken. A signer that ignored its message would still reproduce
    // one vector and would make every mutation case vacuous.
    let vector = &vectors()[1];
    let material = OwnerSigningMaterial::from_published_scalar(&vector.scalar).expect("in range");
    let signed = material
        .sign(&vector.message, &vector.auxiliary)
        .expect("signs");

    let mut other_message = vector.message;
    other_message[0] ^= 0x01;
    assert_ne!(
        material
            .sign(&other_message, &vector.auxiliary)
            .expect("signs"),
        signed,
    );

    let mut other_auxiliary = vector.auxiliary;
    other_auxiliary[0] ^= 0x01;
    assert_ne!(
        material
            .sign(&vector.message, &other_auxiliary)
            .expect("signs"),
        signed,
    );
}

#[test]
fn two_owners_sign_one_message_differently() {
    // The wrong-owner case's material: two published scalars, one
    // message, two signatures and two keys, none of which a target
    // could confuse.
    let [first, second, _] = vectors();
    let left = OwnerSigningMaterial::from_published_scalar(&first.scalar).expect("in range");
    let right = OwnerSigningMaterial::from_published_scalar(&second.scalar).expect("in range");

    assert_ne!(left.x_only_public_key(), right.x_only_public_key());
    assert_ne!(
        left.sign(&first.message, &first.auxiliary).expect("signs"),
        right.sign(&first.message, &first.auxiliary).expect("signs"),
    );
}

#[test]
fn a_scalar_outside_the_group_is_not_a_key() {
    // Both ends of the stated range, as typed refusals rather than
    // panics: a materializer that accepted zero would publish a key
    // with no signature, and one that accepted the order or above would
    // publish a key that is some other key.
    //
    // The refusals are read through `err` rather than compared as
    // results, because the material type deliberately has no `Debug`: a
    // type that could print itself would print the scalar, and not
    // being printable is worth more than an assertion's convenience.
    let refusal = |scalar: [u8; FIELD_ELEMENT_BYTES]| {
        OwnerSigningMaterial::from_published_scalar(&scalar).err()
    };

    assert_eq!(
        refusal([0_u8; FIELD_ELEMENT_BYTES]),
        Some(TestSigningDefect::ScalarOutOfRange),
    );

    // The group order itself, and the largest 32-byte value.
    assert_eq!(
        refusal(hex32(
            b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141",
        )),
        Some(TestSigningDefect::ScalarOutOfRange),
    );
    assert_eq!(
        refusal([0xff_u8; FIELD_ELEMENT_BYTES]),
        Some(TestSigningDefect::ScalarOutOfRange),
    );

    // One below the order is a key, so the bound is exclusive rather
    // than the check being unconditional.
    assert!(
        OwnerSigningMaterial::from_published_scalar(&hex32(
            b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140",
        ))
        .is_ok(),
    );
}
