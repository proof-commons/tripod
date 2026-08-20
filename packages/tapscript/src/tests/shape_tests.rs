//! The concrete shape policy, checked against §9's own conditions.
//!
//! Every rejection §9.1's validity condition can produce is reached
//! here by a shape that violates exactly one conjunct, so a constructor
//! that stopped checking one of them fails a test rather than silently
//! admitting a specialization nobody built a program for.
//!
//! The §9.3 audit is checked in both directions: the demonstration
//! candidate meets every condition, and a deliberately narrow set fails
//! exactly the conditions it is narrow in. An audit checked only on a
//! passing set would agree with itself.

use std::collections::BTreeSet;
use std::num::NonZeroU8;

use crate::shape::{
    CandidateShapeSet, CompactAshShape, CompactAshShapeBounds, MINIMUM_ASH_INPUTS, ShapeRejection,
    SponsorChangePresence, UsefulCandidateCondition, demonstration_shape_set,
};

/// A nonzero count for the fixtures.
fn count(value: u8) -> NonZeroU8 {
    NonZeroU8::new(value).expect("the fixture counts are nonzero")
}

/// Bounds admitting four ASH inputs and one sponsor input.
fn bounds() -> CompactAshShapeBounds {
    CompactAshShapeBounds::new(count(4), 1).expect("four is above the minimum")
}

/// One valid shape, built through the checked constructor.
fn shape(ash: u8, sponsors: u8, change: SponsorChangePresence) -> CompactAshShape {
    CompactAshShape::new(bounds(), count(ash), sponsors, change).expect("a valid shape")
}

#[test]
fn a_batch_below_the_minimum_is_refused() {
    // One source is not an aggregation, and the semantic contract has
    // no unary form. A backend that emitted for it would be emitting
    // for an operation the architecture does not declare.
    assert_eq!(
        CompactAshShape::new(bounds(), count(1), 0, SponsorChangePresence::Absent),
        Err(ShapeRejection::AshInputsBelowMinimum {
            offered: 1,
            minimum: MINIMUM_ASH_INPUTS,
        }),
    );
}

#[test]
fn bounds_below_the_minimum_admit_no_shape_and_are_refused_at_construction() {
    // The refusal is at the bounds rather than at every shape built
    // from them: bounds that admit nothing are a mistake about the
    // candidate, not about one shape.
    assert_eq!(
        CompactAshShapeBounds::new(count(1), 0),
        Err(ShapeRejection::AshInputsBelowMinimum {
            offered: 1,
            minimum: MINIMUM_ASH_INPUTS,
        }),
    );
}

#[test]
fn a_count_above_its_bound_is_refused_on_both_axes() {
    assert_eq!(
        CompactAshShape::new(bounds(), count(5), 0, SponsorChangePresence::Absent),
        Err(ShapeRejection::AshInputsAboveBound {
            offered: 5,
            bound: 4,
        }),
    );
    assert_eq!(
        CompactAshShape::new(bounds(), count(2), 2, SponsorChangePresence::Absent),
        Err(ShapeRejection::SponsorInputsAboveBound {
            offered: 2,
            bound: 1,
        }),
    );
}

#[test]
fn sponsor_change_without_a_sponsor_region_is_refused() {
    // §9.1: `sponsor_change ⇒ sponsor_inputs > 0`. A change role with
    // no region is a role with nothing to be change *of*, and §10.5
    // recognizes it by position within a region that would not exist.
    assert_eq!(
        CompactAshShape::new(bounds(), count(2), 0, SponsorChangePresence::Present),
        Err(ShapeRejection::SponsorChangeWithoutSponsorInput),
    );
}

#[test]
fn a_shape_fixes_the_exact_input_and_output_counts_the_coordinator_authenticates() {
    // Hand-walked against §10.1 and §10.2. The sponsorless shape has no
    // fee output at all, because the reviewed target represents a zero
    // fee by the absence of the output and refuses a zero-valued one;
    // the sponsored shape has one, and one more output again when it
    // declares change.
    let sponsorless = shape(3, 0, SponsorChangePresence::Absent);
    assert_eq!(sponsorless.inputs(), 3);
    assert_eq!(sponsorless.outputs(), 1);
    assert_eq!(sponsorless.ash_range(), (0, 3));
    assert_eq!(sponsorless.sponsor_range(), (3, 3));
    assert!(!sponsorless.sponsored());

    let sponsored = shape(3, 1, SponsorChangePresence::Absent);
    assert_eq!(sponsored.inputs(), 4);
    assert_eq!(sponsored.outputs(), 2);
    assert_eq!(sponsored.ash_range(), (0, 3));
    assert_eq!(sponsored.sponsor_range(), (3, 4));
    assert!(sponsored.sponsored());

    let with_change = shape(3, 1, SponsorChangePresence::Present);
    assert_eq!(with_change.inputs(), 4);
    assert_eq!(with_change.outputs(), 3);
}

#[test]
fn the_sponsor_suffix_starts_where_the_ash_range_ends_for_every_admitted_shape() {
    // §10.4's "exact suffix start" is a property of the shape rather
    // than of a particular transaction, so it is checked over the whole
    // candidate set rather than on one example. The two ranges tile the
    // input count with no gap and no overlap, which is what makes "no
    // ASH member lies in the sponsor suffix" a structural fact.
    for shape in demonstration_shape_set().shapes() {
        let (ash_start, ash_end) = shape.ash_range();
        let (sponsor_start, sponsor_end) = shape.sponsor_range();

        assert_eq!(ash_start, 0);
        assert_eq!(ash_end, sponsor_start);
        assert_eq!(u16::from(sponsor_end), shape.inputs());
    }
}

#[test]
fn the_demonstration_candidate_meets_every_useful_candidate_condition() {
    // §9.3: a candidate supporting only two ASH inputs is useful as a
    // first wave and insufficient for Phase-4 exit. This one supports
    // two through four, sponsorless and sponsored, with change present
    // and absent.
    let candidate = demonstration_shape_set();

    assert_eq!(candidate.unmet_conditions(), BTreeSet::new());
    assert!(!candidate.sparse_counts_declared());
    assert_eq!(candidate.shapes().count(), 9);
    assert_eq!(candidate.bounds().ash_inputs(), 4);
    assert_eq!(candidate.bounds().sponsor_inputs(), 1);
}

#[test]
fn a_narrow_candidate_fails_exactly_the_conditions_it_is_narrow_in() {
    // The negative control. A set holding only the minimum sponsorless
    // batch fails four of the five conditions and meets the fifth,
    // which is what distinguishes a real audit from one that reports
    // whatever it was handed.
    let only_minimum = CandidateShapeSet::new(
        bounds(),
        BTreeSet::from([shape(2, 0, SponsorChangePresence::Absent)]),
        false,
    );

    assert_eq!(
        only_minimum.unmet_conditions(),
        BTreeSet::from([
            UsefulCandidateCondition::BatchAboveMinimum,
            UsefulCandidateCondition::SponsoredShape,
            UsefulCandidateCondition::SponsorChangePresent,
            UsefulCandidateCondition::DenseAshCounts,
        ]),
    );
    assert!(
        !only_minimum
            .unmet_conditions()
            .contains(&UsefulCandidateCondition::SponsorChangeAbsent),
    );
}

#[test]
fn a_gap_in_the_counts_is_a_finding_unless_the_candidate_declared_it_sparse() {
    // §9.3 admits a sparse supported-count set only where the candidate
    // uses one explicitly and reports the limitation. The same shapes
    // therefore audit differently depending on whether the declaration
    // was made — which is the whole difference between a reported
    // limitation and an omission.
    let shapes = BTreeSet::from([
        shape(2, 1, SponsorChangePresence::Present),
        shape(4, 1, SponsorChangePresence::Absent),
    ]);

    let undeclared = CandidateShapeSet::new(bounds(), shapes.clone(), false);
    assert!(
        undeclared
            .unmet_conditions()
            .contains(&UsefulCandidateCondition::DenseAshCounts),
    );

    let declared = CandidateShapeSet::new(bounds(), shapes, true);
    assert!(
        !declared
            .unmet_conditions()
            .contains(&UsefulCandidateCondition::DenseAshCounts),
    );
    assert!(declared.sparse_counts_declared());
}

#[test]
fn membership_is_the_set_and_not_an_inequality() {
    // §9.2: a generic "inspect the first n inputs" description is not
    // an emitted program. A shape inside the bounds but outside the set
    // is unbuilt, and `admits` says so — a membership test derived from
    // the bounds would report it supported.
    let sparse = CandidateShapeSet::new(
        bounds(),
        BTreeSet::from([shape(2, 0, SponsorChangePresence::Absent)]),
        true,
    );

    assert!(sparse.admits(shape(2, 0, SponsorChangePresence::Absent)));
    assert!(!sparse.admits(shape(3, 0, SponsorChangePresence::Absent)));
}
