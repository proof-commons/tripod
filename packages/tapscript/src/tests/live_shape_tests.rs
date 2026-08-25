//! Oracles for the live-transfer shape (Guide-13 §5.1, §12.1, §12.2,
//! §18.1).
//!
//! Every invariant the shape claims is checked against the counts it was
//! built from rather than against another accessor of the same value: a
//! test that derived the input total from the ranges and compared it
//! with the total the ranges were derived from would prove only that one
//! addition was performed twice.

use std::collections::BTreeSet;
use std::num::NonZeroU8;

use crate::live_shape::{
    FeePresence, LiveShapeRejection, LiveTransferShape, LiveTransferShapeBounds,
    LiveTransferShapeSet, MINIMUM_TRANSFER_RECEIPT_INPUTS, MINIMUM_TRANSFER_RECEIPT_OUTPUTS,
    demonstration_live_shape_set, dense_live_shape_set, fee_bearing_live_shape_set,
};
use crate::shape::SponsorChangePresence;

/// A nonzero count for the fixtures below.
fn count(value: u8) -> NonZeroU8 {
    NonZeroU8::new(value).expect("the fixture counts are nonzero")
}

/// Bounds wide enough that no fixture is refused for the wrong reason.
fn bounds() -> LiveTransferShapeBounds {
    LiveTransferShapeBounds::new(count(4), count(4), 2)
}

/// One shape under [`bounds`].
fn shape(
    inputs: u8,
    outputs: u8,
    sponsors: u8,
    change: SponsorChangePresence,
) -> Result<LiveTransferShape, LiveShapeRejection> {
    LiveTransferShape::new(bounds(), count(inputs), count(outputs), sponsors, change)
}

// --- §5.1: the cardinality relation the live transfer actually has ---

#[test]
fn the_receipt_minima_are_one_and_the_one_to_one_shape_is_admissible() {
    assert_eq!(MINIMUM_TRANSFER_RECEIPT_INPUTS, 1);
    assert_eq!(MINIMUM_TRANSFER_RECEIPT_OUTPUTS, 1);

    let unary = shape(1, 1, 0, SponsorChangePresence::Absent).expect("one-to-one is admissible");

    assert_eq!(unary.receipt_inputs(), 1);
    assert_eq!(unary.receipt_outputs(), 1);
}

/// The reason the live transfer has its own shape type: the two
/// operations disagree about the smallest input count that means
/// anything, so a shared minimum would have to be conditional on the
/// operation.
///
/// A compile-time assertion rather than a test body, because both sides
/// are constants: this way a change to either minimum stops the crate
/// building rather than failing one test after everything else has been
/// compiled against the new value.
const _: () = assert!(MINIMUM_TRANSFER_RECEIPT_INPUTS < crate::shape::MINIMUM_ASH_INPUTS);

#[test]
fn every_admitted_composition_of_the_semantic_relation_has_a_shape() {
    // §5.3's four compositions, each as the counts that distinguish it.
    for (inputs, outputs) in [(1, 1), (1, 3), (3, 1), (3, 3)] {
        shape(inputs, outputs, 0, SponsorChangePresence::Absent)
            .expect("every admitted composition has a shape");
    }
}

// --- The validity condition, one refusal per conjunct ---

#[test]
fn receipt_inputs_above_the_bound_are_unbuilt_rather_than_invalid() {
    assert_eq!(
        shape(5, 1, 0, SponsorChangePresence::Absent),
        Err(LiveShapeRejection::ReceiptInputsAboveBound {
            offered: 5,
            bound: 4,
        })
    );
}

#[test]
fn receipt_outputs_above_the_bound_are_refused_on_their_own_axis() {
    assert_eq!(
        shape(1, 5, 0, SponsorChangePresence::Absent),
        Err(LiveShapeRejection::ReceiptOutputsAboveBound {
            offered: 5,
            bound: 4,
        })
    );
}

#[test]
fn sponsor_inputs_above_the_bound_are_refused() {
    assert_eq!(
        shape(1, 1, 3, SponsorChangePresence::Absent),
        Err(LiveShapeRejection::SponsorInputsAboveBound {
            offered: 3,
            bound: 2,
        })
    );
}

#[test]
fn a_change_role_needs_a_sponsor_region_to_sit_in() {
    assert_eq!(
        shape(1, 1, 0, SponsorChangePresence::Present),
        Err(LiveShapeRejection::SponsorChangeWithoutSponsorInput)
    );
}

// --- §12.1 and §12.2: the layout the counts fix ---

#[test]
fn the_input_ranges_partition_every_input_position_exactly_once() {
    for sponsors in 0..=2 {
        for inputs in 1..=4 {
            let subject = shape(inputs, 1, sponsors, SponsorChangePresence::Absent)
                .expect("the fixture is inside the bounds");

            let (receipts_first, receipts_end) = subject.receipt_input_range();
            let (sponsor_first, sponsor_end) = subject.sponsor_range();

            // The receipt family is a prefix, the sponsor suffix abuts
            // it, and the two together are the whole input side.
            assert_eq!(receipts_first, 0);
            assert_eq!(receipts_end, sponsor_first);
            assert_eq!(sponsor_end, subject.inputs());
            assert_eq!(u16::from(inputs), receipts_end - receipts_first);
            assert_eq!(u16::from(sponsors), sponsor_end - sponsor_first);
        }
    }
}

#[test]
fn the_destination_range_is_the_output_prefix() {
    let subject = shape(2, 3, 1, SponsorChangePresence::Present).expect("inside the bounds");

    assert_eq!(subject.destination_range(), (0, 3));
    // Three destinations, one change role, one fee role.
    assert_eq!(subject.outputs(), 5);
}

#[test]
fn a_sponsorless_shape_carries_no_fee_output() {
    let subject = shape(2, 2, 0, SponsorChangePresence::Absent).expect("inside the bounds");

    assert!(!subject.sponsored());
    // The destinations alone: a zero fee is the absence of the output.
    assert_eq!(subject.outputs(), 2);
}

#[test]
fn the_position_domain_cannot_overflow_the_widest_research_candidate() {
    // §18.1's widest listed research candidates, at the top of every
    // axis at once. The claim is about the domain, so it is checked
    // where the domain is worst.
    let widest = LiveTransferShapeBounds::new(count(64), count(64), 16);
    let subject = LiveTransferShape::new(
        widest,
        count(64),
        count(64),
        16,
        SponsorChangePresence::Present,
    )
    .expect("the widest research candidate is a shape");

    assert_eq!(subject.inputs(), 80);
    assert_eq!(subject.outputs(), 66);
    assert_eq!(subject.sponsor_range(), (64, 80));
}

// --- The candidate set ---

#[test]
fn a_candidate_with_nothing_to_emit_is_refused() {
    assert_eq!(
        LiveTransferShapeSet::new(bounds(), BTreeSet::new(), false),
        Err(LiveShapeRejection::EmptyShapeSet)
    );
}

#[test]
fn a_member_outside_the_sets_own_bounds_is_refused() {
    // Built under a wide window, offered to a narrow one: the set that
    // advertises the narrow bound must not carry it.
    let wide = LiveTransferShapeBounds::new(count(4), count(4), 0);
    let member = LiveTransferShape::new(wide, count(4), count(1), 0, SponsorChangePresence::Absent)
        .expect("inside the wide window");
    let narrow = LiveTransferShapeBounds::new(count(2), count(4), 0);

    assert_eq!(
        LiveTransferShapeSet::new(narrow, BTreeSet::from([member]), false),
        Err(LiveShapeRejection::ReceiptInputsAboveBound {
            offered: 4,
            bound: 2,
        })
    );
}

#[test]
fn a_sparsity_declaration_over_a_dense_set_is_refused() {
    let dense = dense_live_shape_set(bounds());

    assert_eq!(
        LiveTransferShapeSet::new(bounds(), dense.shapes().collect(), true),
        Err(LiveShapeRejection::DenseSetDeclaredSparse)
    );
}

#[test]
fn the_dense_unrolling_carries_every_count_in_range_on_both_axes() {
    let set = dense_live_shape_set(bounds());

    for inputs in MINIMUM_TRANSFER_RECEIPT_INPUTS..=bounds().receipt_inputs() {
        assert!(
            set.shapes().any(|shape| shape.receipt_inputs() == inputs),
            "no shape carries {inputs} receipt inputs"
        );
    }
    for outputs in MINIMUM_TRANSFER_RECEIPT_OUTPUTS..=bounds().receipt_outputs() {
        assert!(
            set.shapes().any(|shape| shape.receipt_outputs() == outputs),
            "no shape carries {outputs} destinations"
        );
    }
    assert!(!set.sparse_counts_declared());
    assert!(!set.is_empty());
}

#[test]
fn the_dense_unrolling_has_the_cardinality_the_axes_predict() {
    // Counted independently of the loop that built it: every
    // (inputs, outputs) pair carries the sponsorless form plus both
    // change presences for each admitted sponsor count.
    let subject = bounds();
    let per_pair = 1 + 2 * usize::from(subject.sponsor_inputs());
    let pairs = usize::from(subject.receipt_inputs()) * usize::from(subject.receipt_outputs());

    assert_eq!(dense_live_shape_set(subject).len(), pairs * per_pair);
}

#[test]
fn the_member_leaf_census_is_the_receipt_input_counts() {
    let set = dense_live_shape_set(bounds());

    assert_eq!(set.receipt_input_counts(), BTreeSet::from([1, 2, 3, 4]));
}

#[test]
fn the_demonstration_set_admits_every_semantic_composition_and_both_sponsor_forms() {
    let set = demonstration_live_shape_set();

    for (inputs, outputs) in [(1, 1), (1, 3), (3, 1), (3, 3)] {
        assert!(
            set.shapes()
                .any(|shape| shape.receipt_inputs() == inputs
                    && shape.receipt_outputs() == outputs),
            "the demonstration set has no {inputs}-to-{outputs} shape"
        );
    }
    assert!(set.shapes().any(|shape| !shape.sponsored()));
    assert!(
        set.shapes()
            .any(|shape| shape.sponsor_change() == SponsorChangePresence::Present)
    );
    assert!(
        set.shapes()
            .any(|shape| shape.sponsored()
                && shape.sponsor_change() == SponsorChangePresence::Absent)
    );
}

#[test]
fn a_shape_the_candidate_did_not_build_is_not_admitted() {
    let set = demonstration_live_shape_set();
    let outside = LiveTransferShape::new(
        LiveTransferShapeBounds::new(count(4), count(4), 1),
        count(4),
        count(1),
        0,
        SponsorChangePresence::Absent,
    )
    .expect("admissible under the wider window");

    assert!(!set.admits(outside));
}

#[test]
fn the_demonstration_set_carries_no_fee_bearing_member_and_therefore_moves_no_tree() {
    // The running form of the one hazard the fee axis had. A candidate's
    // shape set becomes one coordinator leaf per shape, the leaves tweak
    // the taproot output key, and that key is the destination program
    // every recorded fixture digest was taken over. So a fee-bearing
    // member reaching the demonstration set would silently move digests
    // belonging to runs a pinned node already accepted, and the failure
    // would surface far away from the edit that caused it. Asked here, it
    // surfaces at the edit.
    let set = demonstration_live_shape_set();

    assert_eq!(
        set.bounds().sponsorless_fee(),
        FeePresence::Absent,
        "the demonstration bounds must leave the fee axis off"
    );
    for shape in set.shapes() {
        assert_eq!(
            shape.fee() == FeePresence::Present,
            shape.sponsored(),
            "a demonstration shape's fee still follows its form exactly"
        );
    }
}

#[test]
fn the_fee_bearing_set_is_the_demonstration_set_plus_the_forms_that_pay_their_own_fee() {
    // Stated as a containment and a difference rather than a count, so
    // the claim survives a bound change: everything the demonstration
    // candidate emits a program for, the fee-bearing candidate emits the
    // same program for, and what it adds is sponsorless and fee-bearing
    // and nothing else.
    let demonstration = demonstration_live_shape_set();
    let widened = fee_bearing_live_shape_set();

    for shape in demonstration.shapes() {
        assert!(
            widened.admits(shape),
            "the fee-bearing candidate drops a demonstration shape: {shape:?}"
        );
    }

    let added: Vec<LiveTransferShape> = widened
        .shapes()
        .filter(|shape| !demonstration.admits(*shape))
        .collect();
    assert!(!added.is_empty(), "the fee axis added nothing at all");
    for shape in &added {
        assert!(!shape.sponsored(), "the added forms carry no sponsor");
        assert_eq!(shape.fee(), FeePresence::Present);
        assert_eq!(
            shape.outputs(),
            u16::from(shape.receipt_outputs()) + 1,
            "one fee position beyond the destinations, and nothing else"
        );
    }
}

#[test]
fn a_sponsored_form_may_not_decline_the_fee_role() {
    // The reviewed reading the fee axis had to keep: a sponsor region
    // exists in order to pay the target, so the sponsored form's fee is
    // not optional. Checked through the widened bounds so the refusal
    // cannot be the axis simply being off.
    let bounds = LiveTransferShapeBounds::new(count(3), count(3), 1).admitting_sponsorless_fee();

    for shape in dense_live_shape_set(bounds).shapes() {
        if shape.sponsored() {
            assert_eq!(
                shape.fee(),
                FeePresence::Present,
                "a sponsored shape reached the set without a fee role: {shape:?}"
            );
        }
    }
}

#[test]
fn a_candidate_that_emits_no_fee_bearing_program_refuses_the_form_as_unbuilt() {
    // Unbuilt HERE rather than invalid anywhere, which is the same
    // distinction the count bounds draw and is why it takes a rejection
    // of its own rather than reusing one about validity.
    let narrow = LiveTransferShapeBounds::new(count(3), count(3), 1);

    assert_eq!(
        LiveTransferShape::paying_its_own_fee(narrow, count(1), count(1)),
        Err(LiveShapeRejection::SponsorlessFeeBeyondBound)
    );
    assert!(
        LiveTransferShape::paying_its_own_fee(
            narrow.admitting_sponsorless_fee(),
            count(1),
            count(1)
        )
        .is_ok(),
        "the same shape is built where the candidate admits it"
    );
}
