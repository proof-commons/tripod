//! Guide-12 preflight reproductions owned by this crate.
//!
//! The test here belongs to its register row. While the row is open the
//! test passes by asserting the defect; the wave that fixes the row
//! flips the assertion, which then stands as the guarantee.
//!
//! - `G12-R10` — CLOSED: a field's parity convention is read off its
//!   committed prefixes rather than stated beside them, so the nonce
//!   records the oddness its compressed pair uses and no field can
//!   name a convention its bytes do not use.

use crate::confidential::{PointParityConvention, reviewed_confidential_review_facts};
use crate::definition::reviewed_elements_tapscript;
use crate::encoding::EncodingClass;

/// `G12-R10`: the nonce names the convention its prefixes do use.
///
/// The nonce's committed form is a point this contract transports
/// rather than one it commits to, and it is written with the standard
/// compressed pair. Which pair that is comes from the encoding
/// registry, where the nonce's committed prefixes and the
/// compressed-public-key class's prefixes are the same two bytes — and
/// the compressed class is the one the curve-checking primitives
/// accept, whose prefix records oddness.
///
/// The row is closed by construction rather than by correction. The
/// convention is no longer stated beside the prefixes; it is read off
/// them, so a field claiming a convention its bytes do not use is not a
/// value anyone can write. The reviewed opening blockers rest on the
/// two conventions disagreeing, and the table now exhibits the
/// disagreement instead of flattening it.
#[test]
fn the_nonce_field_claims_a_parity_convention_its_prefixes_do_not_use() {
    let contract = reviewed_elements_tapscript().expect("the reviewed contract validates");
    let encodings = contract.definition().encodings();
    let prefixes = |class: EncodingClass| {
        encodings
            .get(&class)
            .expect("every encoding key has a specification")
            .prefixes()
            .clone()
    };

    // The same two bytes, from the registry rather than restated.
    assert_eq!(
        prefixes(EncodingClass::ConfidentialNonce),
        prefixes(EncodingClass::CompressedPublicKey),
    );

    let nonce = reviewed_confidential_review_facts().nonce();
    assert_eq!(
        nonce.parity(),
        PointParityConvention::CompressedOddness,
        "G12-R10: the nonce records the convention of the pair it is written with",
    );
    assert_ne!(nonce.parity(), PointParityConvention::QuadraticResidue);

    // The disagreement the opening blockers rest on, exhibited by two
    // fields of one table rather than argued for in prose.
    let facts = reviewed_confidential_review_facts();
    assert_ne!(facts.value().parity(), facts.nonce().parity());
    assert_ne!(facts.asset().parity(), facts.nonce().parity());
}
