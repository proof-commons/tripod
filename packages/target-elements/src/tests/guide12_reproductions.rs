//! Guide-12 preflight reproductions owned by this crate.
//!
//! The test here demonstrates its register row by *passing* while the
//! defect is present. Wave 0 reproduces and does not repair, so the wave
//! that fixes the row flips the assertion.
//!
//! - `G12-R10` — the confidential nonce is typed as recording the
//!   squareness of its y coordinate, although its prefixes are the
//!   compressed-point pair whose convention is oddness.

use crate::confidential::{PointParityConvention, reviewed_confidential_review_facts};
use crate::definition::reviewed_elements_tapscript;
use crate::encoding::EncodingClass;

/// `G12-R10`: the nonce claims a convention its prefixes do not use.
///
/// The nonce's committed form is a transported point rather than a
/// commitment this contract reasons about, and the fact table says so in
/// a comment beside the field. The type has no way to say it, though:
/// every field must name one convention, so the nonce names the one its
/// neighbours use. What its prefixes actually are is settled by the
/// encoding registry, where the nonce's committed pair and the
/// compressed-public-key class's pair are the same two bytes — and the
/// compressed class is the one the curve-checking primitives accept,
/// whose prefix records oddness.
///
/// The consequence is not cosmetic. The reviewed opening blockers rest
/// on the two conventions disagreeing, so a field that claims the wrong
/// one is a fact a later pattern could reason from by analogy — the
/// exact move the distinction exists to forbid.
///
/// The assertions are the defect. A wave that corrects the typed fact,
/// or that adds an explicit no-parity-claim state for a transported
/// point, flips them.
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
        PointParityConvention::QuadraticResidue,
        "G12-R10: the nonce is expected to claim squareness while the row is open",
    );
    assert_ne!(nonce.parity(), PointParityConvention::CompressedOddness);
}
