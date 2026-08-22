//! Guide-13 preflight reproductions owned by this crate.
//!
//! - `G13-R17` — CONFIRMED, and REPAIRED at the source. The row was the
//!   one entry of this wave with no failing-test form: "the census
//!   cannot *prove* completeness" is the absence of a check, not a
//!   wrong answer, so nothing here ever failed. The tests below exhibit
//!   the exact gap the row named, and record what closing it changed.
//!
//! # What the helper does and does not establish
//!
//! `canonical_census` makes two checks: the census is strictly
//! increasing, and every member *present in the analysis* occurs in it.
//! Both are real, and the standing tests in `target_tests` cover them.
//! Neither ranges over the enum, so a variant omitted from the census
//! is invisible whenever the analysis does not happen to emit it — and
//! `assess_complete_census` builds its "complete" input out of the very
//! constant whose completeness is in question. That is still true of
//! the helper, which this repair did not touch, and the stand-in tests
//! below keep it on record.
//!
//! # What the repair changed
//!
//! The doc comment on `RequiredCapability::ALL` used to state the
//! stronger property directly — "a member added to the enum and
//! forgotten here fails at the boundary" — and that sentence was what
//! the row falsified. It is gone, and so is the second list that made
//! it false. `RequiredCapability` and its census are now generated from
//! one declaration by `census_enum!`, per Guide-13 §8.3: a member
//! omitted from the census is no longer a defect the boundary has to
//! catch, because it cannot be written. The repair is structural, so
//! the property it establishes is a compile-time one; what remains
//! testable is that the generated census is canonical where the
//! boundary reads it, which
//! [`the_generated_capability_census_is_canonical_at_the_boundary`]
//! recomputes.

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

/// `G13-R17`: the generated census satisfies the boundary it feeds.
///
/// `ALL` and the enum are no longer two authored lists, so the drift
/// this row was filed about has no form to take: there is nothing for a
/// test to compare, and a length assertion against a hand-counted
/// number would now be a tautology dressed as a tripwire.
///
/// What is worth recomputing is the promise the macro makes on the
/// census it emits. It emits members in declaration order and fixes the
/// derives so that `Ord` is declaration order too, which is what makes
/// the result strictly increasing without anyone maintaining the
/// ordering. Below that claim is checked the way the boundary checks
/// it, by running the real helper over the real census with every
/// member present: a generated census that failed here would be a
/// generator defect rather than a forgotten line.
#[test]
fn the_generated_capability_census_is_canonical_at_the_boundary() {
    let every_member = RequiredCapability::ALL.iter().copied().collect();

    let ordered = canonical_census(&every_member, RequiredCapability::ALL, |capability| {
        CompileError::NoncanonicalCapabilityCensus { capability }
    })
    .expect("the generated census is strictly increasing and covers every present member");

    assert_eq!(ordered, RequiredCapability::ALL);
    assert!(
        !ordered.is_empty(),
        "an empty census would satisfy the assertion above vacuously",
    );
}
