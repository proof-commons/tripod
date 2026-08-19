//! Guide-12 preflight reproductions owned by this crate.
//!
//! The test here belongs to its register row. While the row is open the
//! test passes by asserting the defect; the wave that fixes the row
//! flips the assertion, which then stands as the guarantee.
//!
//! - `G12-R02` — CLOSED: the anchor-set hash takes a validated set, so
//!   no name can hold the separator the framing joins with.

use crate::canonical::{AnchorName, AnchorNameDefect, ValidatedAnchorSet, anchor_set_hash};

/// `G12-R02`: no two anchor sets share one preimage.
///
/// The recipe sorts, deduplicates, joins with a newline, and hashes. The
/// separator used to be admitted inside a member, so a set holding the
/// single name `a\nb` and a set holding the two names `a` and `b`
/// produced the same joined text and therefore the same digest. Neither
/// the function nor its parameter type refused a name that was not an
/// anchor name at all, so an arbitrary string could acquire an attestation
/// anchor identity.
///
/// The repair is a validated set that no such name can inhabit, and it
/// leaves the recipe alone: the framing the pin was computed under is
/// exactly what it was, which the published-recipe test still checks
/// against its own independently computed digest.
#[test]
fn no_anchor_name_can_hold_the_separator_the_framing_joins_with() {
    // The collision the row was about cannot be built: the newline is
    // refused by name, before any general character rule.
    assert_eq!(
        AnchorName::new("a\nb"),
        Err(AnchorNameDefect::HoldsTheSetSeparator),
    );
    assert_eq!(
        ValidatedAnchorSet::new(["a\nb"]),
        Err(AnchorNameDefect::HoldsTheSetSeparator),
    );

    // The two names of the former collision are not anchor names
    // either, which is the row's second half: an arbitrary string can no
    // longer acquire an anchor identity.
    for arbitrary in ["a", "b", "not a label", "def:model", "DEF:MODEL:CLASSES"] {
        assert!(
            AnchorName::new(arbitrary).is_err(),
            "{arbitrary:?} is not an anchor name",
        );
    }

    // What the hash does take is a set of real anchor names, and
    // distinct sets of them keep distinct preimages.
    let one = ValidatedAnchorSet::new(["def:model:classes"]).expect("an anchor name");
    let two = ValidatedAnchorSet::new(["def:model:classes", "rem:model:calibration"])
        .expect("two anchor names");
    assert_ne!(anchor_set_hash(&one), anchor_set_hash(&two));

    // And the set is a set: order and repetition are not part of it.
    let repeated = ValidatedAnchorSet::new([
        "rem:model:calibration",
        "def:model:classes",
        "def:model:classes",
    ])
    .expect("two anchor names");
    assert_eq!(anchor_set_hash(&two), anchor_set_hash(&repeated));
    assert_eq!(repeated.len(), 2);
}
