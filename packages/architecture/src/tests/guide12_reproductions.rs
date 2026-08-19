//! Guide-12 preflight reproductions owned by this crate.
//!
//! The test here demonstrates its register row by *passing* while the
//! defect is present. Wave 0 reproduces and does not repair, so the wave
//! that fixes the row flips the assertion.
//!
//! - `G12-R02` — the anchor-set hash frames an unvalidated string set by
//!   newline join, so one name holding a newline and two names have one
//!   preimage.

use crate::canonical::anchor_set_hash;

/// `G12-R02`: two different anchor sets share one preimage.
///
/// The recipe sorts, deduplicates, joins with a newline, and hashes. The
/// separator is admitted inside a member, so a set holding the single
/// name `a\nb` and a set holding the two names `a` and `b` produce the
/// same joined text and therefore the same digest. Neither the function
/// nor its parameter type refuses a name that is not an anchor name at
/// all, which is the second half of the row: an arbitrary string can
/// acquire an attestation anchor identity.
///
/// The assertion is the defect. A wave that hashes a validated
/// anchor-set type — which no such name can inhabit — flips it, and does
/// so without disturbing the recipe: the framing that the pin was
/// computed under stays exactly as it is.
#[test]
fn a_newline_bearing_anchor_name_collides_with_two_names() {
    let one_name = anchor_set_hash(["a\nb"]);
    let two_names = anchor_set_hash(["a", "b"]);

    assert_eq!(
        one_name, two_names,
        "G12-R02: the framing is expected to be ambiguous while the row is open",
    );
}
