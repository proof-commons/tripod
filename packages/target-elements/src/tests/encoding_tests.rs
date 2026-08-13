//! Independent cross-checks of the encoding registry.
//!
//! The expected byte layouts here are written out literally. The crate
//! contains no encoder, so there is no risk of the expectation being
//! produced by the thing under test; what these tests check is that
//! the declared prefixes and widths reconstruct the layouts the review
//! recorded.

use std::collections::{BTreeMap, BTreeSet};

use crate::definition::reviewed_elements_tapscript;
use crate::encoding::{
    ByteOrder, CanonicalEncodingRule, EncodingClass, EncodingDomain, EncodingSpec, PayloadWidth,
    UnknownPrefixRule,
};

/// Fetches one encoding specification.
fn spec(class: EncodingClass) -> EncodingSpec {
    reviewed_elements_tapscript()
        .expect("the reviewed contract validates")
        .definition()
        .encodings()
        .get(&class)
        .expect("every encoding key has a specification")
        .clone()
}

/// The total serialized width of a prefixed field: one prefix byte
/// plus the payload.
fn serialized_width(spec: &EncodingSpec) -> Option<usize> {
    let payload = match spec.payload() {
        PayloadWidth::Exact(width) => width.get(),
        PayloadWidth::Absent => 0,
        PayloadWidth::Bounded { .. } => return None,
    };
    Some(usize::from(!spec.prefixes().is_empty()) + payload)
}

#[test]
fn the_encoding_census_is_complete_and_duplicate_free() {
    let declared: BTreeSet<EncodingClass> = EncodingClass::ALL.iter().copied().collect();
    assert_eq!(declared.len(), EncodingClass::ALL.len());

    let registered: BTreeSet<EncodingClass> = reviewed_elements_tapscript()
        .expect("the reviewed contract validates")
        .definition()
        .encodings()
        .keys()
        .copied()
        .collect();
    assert_eq!(registered, declared);
}

#[test]
fn the_confidential_field_prefixes_are_the_reviewed_bytes() {
    // Stated literally. These are the bytes a decoder must switch on,
    // and getting one wrong would make a valid field unreadable while
    // the contract still validated.
    let expected: &[(EncodingClass, &[u8])] = &[
        (EncodingClass::ExplicitAsset, &[0x01]),
        (EncodingClass::ConfidentialAsset, &[0x0a, 0x0b]),
        (EncodingClass::ExplicitValue, &[0x01]),
        (EncodingClass::ConfidentialValue, &[0x08, 0x09]),
        (EncodingClass::ExplicitNonce, &[0x01]),
        (EncodingClass::ConfidentialNonce, &[0x02, 0x03]),
        (EncodingClass::CompressedPublicKey, &[0x02, 0x03]),
    ];

    for (class, bytes) in expected {
        let declared: Vec<u8> = spec(*class).prefixes().iter().copied().collect();
        assert_eq!(&declared[..], *bytes, "{class:?}");
    }
}

#[test]
fn the_explicit_byte_marks_three_different_fields() {
    // The same prefix means "explicit" in the asset, value, and nonce
    // groups. A global uniqueness rule would have rejected this
    // correct contract, which is exactly why prefixes are scoped to a
    // field group.
    for class in [
        EncodingClass::ExplicitAsset,
        EncodingClass::ExplicitValue,
        EncodingClass::ExplicitNonce,
    ] {
        assert!(spec(class).prefixes().contains(&0x01), "{class:?}");
    }

    let domains: BTreeSet<EncodingDomain> = [
        EncodingClass::ExplicitAsset,
        EncodingClass::ExplicitValue,
        EncodingClass::ExplicitNonce,
    ]
    .into_iter()
    .map(|class| spec(class).domain())
    .collect();
    assert_eq!(domains.len(), 3, "each sits in its own field group");
}

#[test]
fn the_serialized_field_widths_match_the_reviewed_layout() {
    // An asset field is one prefix byte and a thirty-two byte payload
    // in both forms. A value field is one prefix byte and eight bytes
    // when explicit, and one prefix byte and thirty-two when blinded:
    // the widths differ between the forms of one field, which is why
    // the payload width belongs to the form and not to the field.
    let expected: &[(EncodingClass, usize)] = &[
        (EncodingClass::ExplicitAsset, 33),
        (EncodingClass::ConfidentialAsset, 33),
        (EncodingClass::ExplicitValue, 9),
        (EncodingClass::ConfidentialValue, 33),
        (EncodingClass::ExplicitNonce, 33),
        (EncodingClass::ConfidentialNonce, 33),
        (EncodingClass::CompressedPublicKey, 33),
        (EncodingClass::SchnorrSignature, 64),
        (EncodingClass::XOnlyPublicKey, 32),
        (EncodingClass::OutPointTxid, 32),
        (EncodingClass::OutPointIndex, 4),
        (EncodingClass::OutPointFlags, 1),
        (EncodingClass::Sequence, 4),
        (EncodingClass::Sha256Digest, 32),
        (EncodingClass::EcScalar, 32),
        (EncodingClass::TaprootTweak, 32),
    ];

    for (class, width) in expected {
        assert_eq!(
            serialized_width(&spec(*class)),
            Some(*width),
            "{class:?} serialized width"
        );
    }
}

#[test]
fn the_absent_forms_carry_no_bytes_at_all() {
    // The empty item is a form in its own right, not a zero-length
    // payload of some other form.
    for class in [EncodingClass::NullValue, EncodingClass::NullNonce] {
        let spec = spec(class);
        assert_eq!(spec.payload(), PayloadWidth::Absent, "{class:?}");
        assert!(spec.prefixes().is_empty(), "{class:?}");
        assert_eq!(serialized_width(&spec), Some(0), "{class:?}");
    }
}

#[test]
fn only_numeric_encodings_carry_a_byte_order() {
    let numeric: BTreeMap<EncodingClass, ByteOrder> = reviewed_elements_tapscript()
        .expect("the reviewed contract validates")
        .definition()
        .encodings()
        .iter()
        .filter_map(|(class, spec)| spec.byte_order().map(|order| (*class, order)))
        .collect();

    let expected: BTreeMap<EncodingClass, ByteOrder> = [
        (EncodingClass::ExplicitValue, ByteOrder::LittleEndian),
        (EncodingClass::OutPointIndex, ByteOrder::LittleEndian),
        (EncodingClass::Sequence, ByteOrder::LittleEndian),
        (EncodingClass::ScriptNumber, ByteOrder::LittleEndian),
        (EncodingClass::SignedLittleEndian64, ByteOrder::LittleEndian),
        (
            EncodingClass::UnsignedLittleEndian32,
            ByteOrder::LittleEndian,
        ),
        (
            EncodingClass::UnsignedLittleEndian64,
            ByteOrder::LittleEndian,
        ),
    ]
    .into_iter()
    .collect();

    assert_eq!(numeric, expected);

    // An asset identifier is opaque bytes. Giving it an order would
    // claim a numeric reading the field does not have.
    assert_eq!(spec(EncodingClass::ExplicitAsset).byte_order(), None);
    assert_eq!(spec(EncodingClass::Sha256Digest).byte_order(), None);
}

#[test]
fn the_hash_state_width_is_genuinely_variable() {
    // Forty bytes of state plus whatever partial block it holds.
    let spec = spec(EncodingClass::Sha256Context);
    match spec.payload() {
        PayloadWidth::Bounded { minimum, maximum } => {
            assert_eq!(minimum, 40);
            assert_eq!(maximum.get(), 103);
        }
        other => panic!("a hash state is not one fixed width: {other:?}"),
    }
}

#[test]
fn every_encoding_rejects_an_unrecognized_prefix() {
    // There is exactly one rule, and no encoding may opt out of it. An
    // unknown prefix preserved as an opaque valid value would let an
    // unreviewed encoding flow through a program as though it had been
    // reviewed.
    let definition = reviewed_elements_tapscript().expect("the reviewed contract validates");
    for (class, spec) in definition.definition().encodings() {
        assert_eq!(
            spec.unknown_prefix(),
            UnknownPrefixRule::Reject,
            "{class:?}"
        );
        assert!(!spec.evidence().is_empty(), "{class:?} names evidence");
    }
}

#[test]
fn the_script_number_is_minimal_and_the_fixed_widths_are_not() {
    assert_eq!(
        spec(EncodingClass::ScriptNumber).canonicality(),
        CanonicalEncodingRule::Minimal
    );
    for class in [
        EncodingClass::SignedLittleEndian64,
        EncodingClass::UnsignedLittleEndian32,
        EncodingClass::UnsignedLittleEndian64,
    ] {
        assert_eq!(
            spec(class).canonicality(),
            CanonicalEncodingRule::FixedWidth,
            "{class:?}"
        );
    }
}

#[test]
fn asset_and_value_confidentiality_are_independent() {
    // Nothing in the registry lets one be inferred from the other:
    // they sit in different field groups with different prefixes and,
    // in the explicit case, different widths.
    let asset = spec(EncodingClass::ConfidentialAsset);
    let value = spec(EncodingClass::ConfidentialValue);
    assert_ne!(asset.domain(), value.domain());
    assert!(asset.prefixes().is_disjoint(value.prefixes()));

    assert_ne!(
        spec(EncodingClass::ExplicitAsset).payload(),
        spec(EncodingClass::ExplicitValue).payload()
    );
}
