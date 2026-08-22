//! Guide-13 preflight reproductions owned by this crate.
//!
//! - `G13-R17` — CONFIRMED, and the one row of this wave with no
//!   failing-test form. "The census cannot *prove* completeness" is the
//!   absence of a check, not a wrong answer, so nothing here fails
//!   today; what the tests below do is exhibit the exact gap, so the
//!   claim is on record as measured rather than argued.
//!
//! # What the helper does and does not establish
//!
//! `canonical_census` makes two checks: the census is strictly
//! increasing, and every member *present in the analysis* occurs in it.
//! Both are real, and the standing tests in `target_tests` cover them.
//! Neither ranges over the enum, so a variant omitted from the census
//! is invisible whenever the analysis does not happen to emit it — and
//! `assess_complete_census` builds its "complete" input out of the very
//! constant whose completeness is in question.
//!
//! The doc comment on `RequiredCapability::ALL` states the stronger
//! property directly: "a member added to the enum and forgotten here
//! fails at the boundary." That sentence is what the row falsifies.

use std::collections::BTreeSet;

use crate::capability::RequiredCapability;
use crate::error::CompileError;
use crate::target::canonical_census;

/// A stand-in domain whose complete membership is known right here.
///
/// The row is about a property of *any* census over a closed domain, so
/// it is exhibited on a domain this module owns outright rather than on
/// `RequiredCapability`, whose completeness is precisely the thing that
/// cannot be settled. Three members, and the middle one is the one a
/// census will omit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum StandIn {
    First,
    Second,
    Third,
}

/// A census that silently omits [`StandIn::Second`].
const INCOMPLETE: &[StandIn] = &[StandIn::First, StandIn::Third];

/// Run the helper over the stand-in domain.
fn census(present: &BTreeSet<StandIn>, census: &[StandIn]) -> Result<Vec<StandIn>, CompileError> {
    canonical_census(present, census, |_| {
        CompileError::NoncanonicalCapabilityCensus {
            capability: RequiredCapability::OwnerAuthorization,
        }
    })
}

/// `G13-R17`: an omitted member is invisible while nothing emits it.
///
/// The census below is missing a third of its domain. It is strictly
/// increasing, and every member the analysis actually presents does
/// occur in it, so both checks pass and the helper returns the ordered
/// members as though the census were whole.
///
/// This is the whole of the row: the property the constant advertises
/// is completeness, and the check available is inclusion.
#[test]
fn a_census_omitting_a_member_nothing_emits_is_accepted_as_canonical() {
    let present = BTreeSet::from([StandIn::First, StandIn::Third]);

    assert_eq!(
        census(&present, INCOMPLETE).expect("the helper accepts an incomplete census"),
        vec![StandIn::First, StandIn::Third],
    );
}

/// `G13-R17`: the omission is caught only by accident.
///
/// The same incomplete census, and the only thing that changed is that
/// the analysis happened to emit the omitted member. That is what makes
/// the existing coverage weaker than it reads: detection depends on the
/// analysis, not on the census, so a capability that no current
/// analysis emits can be missing from `ALL` indefinitely.
#[test]
fn the_same_omission_is_caught_only_when_something_happens_to_emit_it() {
    let present = BTreeSet::from([StandIn::First, StandIn::Second, StandIn::Third]);

    assert!(
        census(&present, INCOMPLETE).is_err(),
        "the omitted member is reported only because the analysis presented it",
    );
}

/// `G13-R17`: the two checks that are real, on the stand-in domain.
///
/// Recorded here so the row is not read as "the helper checks nothing".
/// It checks order and inclusion exactly, and a repair must keep both
/// while adding the completeness obligation it lacks.
#[test]
fn the_helper_still_refuses_a_misordered_or_repeated_census() {
    let present = BTreeSet::new();

    assert!(
        census(&present, &[StandIn::Third, StandIn::First]).is_err(),
        "a census out of the domain's own order is refused",
    );
    assert!(
        census(&present, &[StandIn::First, StandIn::First]).is_err(),
        "a census naming one member twice is refused",
    );
}

/// `G13-R17`: `ALL` and the enum are bound by two authored lists.
///
/// The standing test `the_capability_census_is_complete_and_duplicate_free`
/// compares `RequiredCapability::ALL` with a literal array written
/// beside it — its own comment says a "census compared only with itself
/// agrees with itself". Both lists are hand-maintained, so a variant
/// added to the enum and to neither list changes nothing that any test
/// observes.
///
/// What can be recorded without a generator is the count the two lists
/// currently agree on. It is a tripwire and not a proof, and it is
/// written here rather than in the census tests so that nothing reads
/// it as the completeness check the row says is missing.
#[test]
fn the_capability_census_records_the_membership_it_was_measured_at() {
    assert_eq!(
        RequiredCapability::ALL.len(),
        13,
        "the census changed size; the enum and `ALL` must be checked against each other by hand \
         until they are generated from one source",
    );
}
