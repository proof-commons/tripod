//! Oracles for the explicit live-transfer plan (Guide-13 §10.4 – §10.8).
//!
//! # What is checked against what
//!
//! Nothing here asks a fragment to confirm its own reasoning. The
//! conservation's arithmetic discipline is read off the emitted
//! instruction list rather than off the documentation, and the claim that
//! consuming each flag immediately is load-bearing is executed: the
//! fragment is walked twice, once through its first addition and once
//! through the verification after it, and the overflow result that
//! survives the first walk is gone from the second. The sponsor
//! fragment's amount-freedom is likewise a census over emitted
//! primitives, because a fragment that never introspects a value field
//! cannot have read an amount whatever it says.
//!
//! The family ranges are checked over every shape the demonstration
//! candidate admits rather than over one, because a gap is a property of
//! a shape and a census that held for the shape somebody happened to pick
//! would say nothing about the rest.
//!
//! # The symbols below are public test material
//!
//! Distinguishable byte strings standing in for values a later wave
//! resolves, in the sense `(´[ADR015-rule:security:test-material]´)`
//! fixes. None of them is a key, and none of them is an amount: the two
//! assets differ because the isolation relation is exactly that they
//! differ, and no test here fixes a receipt value at all — §10.5 is
//! value-parametric and the oracles below are written so that no amount
//! could make one of them pass.

use std::collections::BTreeSet;
use std::num::NonZeroU8;

use compiler::live_transfer_plan::LiveTransferRepresentationPlan;
use target_elements::{EncodingClass, OpcodeId};

use super::{live_transfer_plan, live_transfer_symbols, reviewed_target};
use crate::authorization::owner_key_encoding_closure;
use crate::bundle::FieldSide;
use crate::instruction::{StackItem, TapscriptInstruction};
use crate::live_constructor::{
    OwnerKey, StaticLiveReceiptConstructor, derive_live_receipt_constructor,
    static_transfer_leaf_set,
};
use crate::live_pattern::{
    LiveFragmentId, LiveTransferPatternId, LiveTransferSymbols, RecognitionResidual,
    final_stack_defects, live_coordinator_program, live_program_precondition,
    live_transfer_patterns, local_recognition_fragment,
};
use crate::live_plan::{
    CompleteFamilyRanges, FamilyRangeDefect, LiveFamily, LiveFamilyRange, LiveInputFamily,
    LiveOutputFamily, destination_closure_fragment, explicit_conservation_fragment,
    family_range_defects, has_sponsor_region, issuance_absence_fragment, live_family_ranges,
    live_sponsor_isolation_fragment, opens_an_amount, reads_a_value_field,
};
use crate::live_shape::{LiveTransferShape, demonstration_live_shape_set};
use crate::program::TapscriptProgram;
use crate::shape::SponsorChangePresence;
use crate::stack::{AbstractLimits, AbstractStackState, validate_program};

// --- Fixtures ---------------------------------------------------------

/// The link-time symbol set.
fn symbols() -> LiveTransferSymbols {
    live_transfer_symbols(&reviewed_target())
}

/// A nonzero count.
///
/// # Panics
///
/// Never: every caller passes a literal above zero.
fn count(value: u8) -> NonZeroU8 {
    NonZeroU8::new(value).expect("the fixture counts are nonzero")
}

/// One admitted shape, by its four axes.
///
/// # Panics
///
/// If the demonstration bounds stop admitting it, which would make the
/// fixture rather than the fragment the thing under test.
fn shape(
    receipt_inputs: u8,
    receipt_outputs: u8,
    sponsor_inputs: u8,
    change: SponsorChangePresence,
) -> LiveTransferShape {
    LiveTransferShape::new(
        demonstration_live_shape_set().bounds(),
        count(receipt_inputs),
        count(receipt_outputs),
        sponsor_inputs,
        change,
    )
    .expect("the demonstration bounds admit the fixture shape")
}

/// A sponsorless shape of these two counts.
fn plain(receipt_inputs: u8, receipt_outputs: u8) -> LiveTransferShape {
    shape(
        receipt_inputs,
        receipt_outputs,
        0,
        SponsorChangePresence::Absent,
    )
}

/// The explicit-plan reference constructor.
///
/// # Panics
///
/// If the reference parts stop deriving a constructor, which Wave 4's own
/// oracles would have caught first.
fn explicit() -> StaticLiveReceiptConstructor {
    let representation = LiveTransferRepresentationPlan::Explicit;
    let shapes = demonstration_live_shape_set();
    let leaves = static_transfer_leaf_set(representation, &shapes);
    let closure = owner_key_encoding_closure(reviewed_target().definition().authorization());
    let owner = OwnerKey::new(&closure, closure.approved(), vec![0x11; 32])
        .expect("the fixture is the approved encoding at its exact width");

    derive_live_receipt_constructor(
        &reviewed_target(),
        &live_transfer_plan(),
        representation,
        owner,
        shapes,
        leaves,
    )
    .expect("the reference subject derives")
}

/// How many times one program schedules a reviewed primitive.
fn schedules(program: &TapscriptProgram, id: OpcodeId) -> usize {
    program
        .instructions()
        .iter()
        .filter(|instruction| **instruction == TapscriptInstruction::Opcode(id))
        .count()
}

/// Every literal one program pushes, in program order.
fn pushed(program: &TapscriptProgram) -> Vec<StackItem> {
    program
        .instructions()
        .iter()
        .filter_map(|instruction| match instruction {
            TapscriptInstruction::Push(item) => Some(item.clone()),
            TapscriptInstruction::Opcode(_) => None,
        })
        .collect()
}

/// The reviewed prefix byte of one encoding class.
///
/// # Panics
///
/// If the class states no prefix, which the asset classes do.
fn class_prefix(class: EncodingClass) -> u8 {
    reviewed_target()
        .definition()
        .encodings()
        .get(&class)
        .and_then(|spec| spec.prefixes().iter().next().copied())
        .expect("the asset encodings are prefix discriminated")
}

/// Walks one program from the empty stack.
///
/// # Panics
///
/// If the program does not schedule, which is a defect in the fragment
/// rather than a finding about the target.
fn walk(program: &TapscriptProgram) -> crate::stack::AbstractExecutionResult {
    let target = reviewed_target();
    validate_program(
        &target,
        program,
        &AbstractStackState::from_main(Vec::new()),
        AbstractLimits::for_target(&target),
    )
    .expect("the fragment schedules")
}

// --- Complete family ranges (§10.3, §12.1, §12.2) ---------------------

#[test]
fn every_position_of_every_admitted_shape_is_held_by_exactly_one_family() {
    // Over the whole candidate rather than over one shape: a gap is a
    // property of a shape, and the absence relations of §10.8 rest on
    // there being none in any shape this candidate emits a program for.
    for shape in demonstration_live_shape_set().shapes() {
        let ranges = live_family_ranges(shape);
        assert_eq!(
            family_range_defects(&ranges),
            Vec::new(),
            "the family ranges of {shape:?} do not cover its transaction",
        );

        // Every run lies on the side it was filed under, so a family
        // cannot be classified on one side and counted on the other.
        for range in ranges.inputs() {
            assert_eq!(range.family().side(), FieldSide::Input);
        }
        for range in ranges.outputs() {
            assert_eq!(range.family().side(), FieldSide::Output);
        }

        // Stated twice on purpose, the second time from the ranges
        // themselves: the covered positions are exactly the transaction's.
        let covered = |ranges: &[crate::live_plan::LiveFamilyRange]| {
            ranges
                .iter()
                .flat_map(|range| range.first()..range.end())
                .collect::<BTreeSet<_>>()
        };
        assert_eq!(
            covered(ranges.inputs()),
            (0..shape.inputs()).collect::<BTreeSet<_>>(),
        );
        assert_eq!(
            covered(ranges.outputs()),
            (0..shape.outputs()).collect::<BTreeSet<_>>(),
        );
    }
}

#[test]
fn the_coordinator_holds_input_zero_and_the_members_hold_the_rest() {
    let ranges = live_family_ranges(plain(3, 2));
    let anchor = ranges
        .range(LiveFamily::Input(LiveInputFamily::Coordinator))
        .expect("every shape has a coordinator");
    let members = ranges
        .range(LiveFamily::Input(LiveInputFamily::Member))
        .expect("a three-receipt shape has members");

    assert_eq!((anchor.first(), anchor.end()), (0, 1));
    assert_eq!((members.first(), members.end()), (1, 3));
    // And a one-to-one transfer has no member run at all rather than an
    // empty one, which the defect census would refuse.
    assert!(
        live_family_ranges(plain(1, 1))
            .range(LiveFamily::Input(LiveInputFamily::Member))
            .is_none(),
    );
}

#[test]
fn no_range_is_authenticated_by_nothing() {
    // The property that makes the census evidence rather than a diagram:
    // every run names emitted fragments, so a family nobody tests is a
    // defect rather than a row.
    for shape in demonstration_live_shape_set().shapes() {
        for range in live_family_ranges(shape).ranges() {
            assert!(
                range.authenticated_by().next().is_some(),
                "{:?} of {shape:?} is authenticated by nothing",
                range.family(),
            );
        }
    }
}

#[test]
fn a_gap_an_overlap_an_empty_run_and_an_unauthenticated_run_are_all_refused() {
    // The four defects, each executed against a census stated through the
    // same public constructor a consumer would use. The census is the
    // claim and the validator is the check, so a claim that does not hold
    // is something a test can build rather than something to describe.
    let subject = plain(2, 3);
    let full = |first, end| {
        LiveFamilyRange::new(
            LiveFamily::Output(LiveOutputFamily::Destination),
            first,
            end,
            BTreeSet::from([LiveFragmentId::DestinationClosure]),
        )
    };
    let census = |outputs: Vec<LiveFamilyRange>| {
        CompleteFamilyRanges::new(
            subject,
            live_family_ranges(subject).inputs().to_vec(),
            outputs,
        )
    };

    // A run one position short of the output count.
    let gap = census(vec![full(0, subject.outputs() - 1)]);
    assert!(
        family_range_defects(&gap).contains(&FamilyRangeDefect::PositionUnaccounted {
            side: FieldSide::Output,
            position: subject.outputs() - 1,
        }),
    );

    // Two runs over the same positions.
    let overlap = census(vec![full(0, subject.outputs()), full(0, subject.outputs())]);
    assert!(
        family_range_defects(&overlap).contains(&FamilyRangeDefect::PositionClaimedTwice {
            side: FieldSide::Output,
            position: 0,
        }),
    );

    // A run covering nothing, which is an absent family rather than a
    // narrow one.
    let empty = census(vec![full(0, subject.outputs()), full(1, 1)]);
    assert!(
        family_range_defects(&empty).contains(&FamilyRangeDefect::RangeIsEmpty {
            family: LiveFamily::Output(LiveOutputFamily::Destination),
        })
    );

    // And a run no emitted fragment stands behind, which is a family the
    // coordinator has described rather than authenticated.
    let unbacked = census(vec![LiveFamilyRange::new(
        LiveFamily::Output(LiveOutputFamily::Destination),
        0,
        subject.outputs(),
        BTreeSet::new(),
    )]);
    assert!(family_range_defects(&unbacked).contains(
        &FamilyRangeDefect::RangeAuthenticatedByNothing {
            family: LiveFamily::Output(LiveOutputFamily::Destination),
        }
    ),);
}

// --- §10.4: destination constructor closure ---------------------------

#[test]
fn every_destination_is_compared_with_the_exact_linked_protocol_asset() {
    let target = reviewed_target();
    let symbols = symbols();
    let subject = plain(2, 3);
    let fragment = destination_closure_fragment(&target, &symbols, subject)
        .expect("the closure fragment assembles");

    // Five literals per destination and no others: the position, the
    // explicit-asset prefix, the exact linked asset, the position again,
    // and the version a live-receipt constructor's program is read at. A
    // fragment comparing against anything else would have a sixth.
    let literals = pushed(&fragment);
    assert_eq!(literals.len(), 5 * usize::from(subject.receipt_outputs()));
    for chunk in literals.chunks(5) {
        assert_eq!(
            chunk[1].bytes(),
            &[class_prefix(EncodingClass::ExplicitAsset)]
        );
        assert_eq!(chunk[2], *symbols.protocol_asset());
    }
    // One asset comparison per destination, against the linked symbol
    // every time.
    assert_eq!(
        literals
            .iter()
            .filter(|item| *item == symbols.protocol_asset())
            .count(),
        usize::from(subject.receipt_outputs()),
    );
    assert_eq!(
        schedules(&fragment, OpcodeId::InspectOutputAsset),
        usize::from(subject.receipt_outputs()),
    );
    assert_eq!(
        schedules(&fragment, OpcodeId::InspectOutputScriptPubKey),
        usize::from(subject.receipt_outputs()),
    );
}

#[test]
fn no_output_outside_the_destination_range_may_carry_the_protocol_asset() {
    // §10.4's second sentence. Its bytes are the sponsor fragment's: every
    // output position the destination range does not hold is required to
    // carry the reserve asset, which the compiler's own plan keeps distinct
    // from the protocol asset, and the exact output count leaves no third
    // kind of position.
    let target = reviewed_target();
    let symbols = symbols();
    let subject = shape(1, 1, 1, SponsorChangePresence::Present);
    let isolation = live_sponsor_isolation_fragment(&target, &symbols, subject)
        .expect("the isolation fragment assembles");

    assert_ne!(symbols.protocol_asset(), symbols.reserve_asset());
    assert!(
        pushed(&isolation)
            .iter()
            .all(|item| item != symbols.protocol_asset()),
    );
    // Both non-destination outputs are tested, which with the count is
    // every position outside the destination range.
    let ranges = live_family_ranges(subject);
    let destinations = ranges
        .range(LiveFamily::Output(LiveOutputFamily::Destination))
        .expect("every shape has destinations");
    assert_eq!(
        u32::from(subject.outputs() - destinations.count()),
        2,
        "the fixture is the shape with both sponsor output roles",
    );
    assert_eq!(
        schedules(&isolation, OpcodeId::InspectOutputAsset),
        2,
        "an output role outside the destination range went untested",
    );
}

#[test]
fn the_destination_closure_carries_the_constructor_identity_residual() {
    // The one thing §10.4 states that no in-script comparison reaches, and
    // the pattern says so rather than reading as complete.
    let patterns = live_transfer_patterns(
        &reviewed_target(),
        &symbols(),
        &explicit(),
        shape(2, 2, 1, SponsorChangePresence::Present),
    )
    .expect("the census builds");

    for id in [
        LiveTransferPatternId::LiveDestinationClosureV1,
        LiveTransferPatternId::LiveCoordinatorProgramV1,
    ] {
        assert!(
            patterns[&id]
                .residuals()
                .contains(&RecognitionResidual::LinkedDestinationConstructorIdentity),
            "{id:?} claims a destination's constructor is established",
        );
    }
}

// --- §10.5: exact explicit aggregate conservation ---------------------

#[test]
fn every_addition_has_its_success_flag_consumed_by_the_next_instruction() {
    // §6.2's "every arithmetic success flag is consumed immediately",
    // read off the emitted instruction list. Not "somewhere later": the
    // very next instruction, so no unchecked result is ever live across a
    // third one.
    for subject in demonstration_live_shape_set().shapes() {
        let fragment = explicit_conservation_fragment(&reviewed_target(), subject)
            .expect("the conservation fragment assembles");
        let instructions = fragment.instructions();

        let mut additions = 0;
        for (position, instruction) in instructions.iter().enumerate() {
            if *instruction != TapscriptInstruction::Opcode(OpcodeId::Add64) {
                continue;
            }
            additions += 1;
            assert_eq!(
                instructions.get(position + 1),
                Some(&TapscriptInstruction::Opcode(OpcodeId::Verify)),
                "an addition of {subject:?} leaves its flag for a later instruction",
            );
        }

        // Exactly the additions the two folds need, and no more: one
        // fewer than each side's count.
        assert_eq!(
            additions,
            usize::from(subject.receipt_inputs() - 1) + usize::from(subject.receipt_outputs() - 1),
        );
    }
}

#[test]
fn the_two_sums_are_computed_independently_and_then_compared() {
    let subject = plain(3, 2);
    let fragment = explicit_conservation_fragment(&reviewed_target(), subject)
        .expect("the conservation fragment assembles");

    // One read per position on each side, and neither total is derived
    // from the other: the output side is read and folded before the input
    // side is read at all.
    assert_eq!(
        schedules(&fragment, OpcodeId::InspectOutputValue),
        usize::from(subject.receipt_outputs()),
    );
    assert_eq!(
        schedules(&fragment, OpcodeId::InspectInputValue),
        usize::from(subject.receipt_inputs()),
    );
    let instructions = fragment.instructions();
    let last_output = instructions
        .iter()
        .rposition(|instruction| {
            *instruction == TapscriptInstruction::Opcode(OpcodeId::InspectOutputValue)
        })
        .expect("the output side is read");
    let first_input = instructions
        .iter()
        .position(|instruction| {
            *instruction == TapscriptInstruction::Opcode(OpcodeId::InspectInputValue)
        })
        .expect("the input side is read");
    assert!(last_output < first_input);

    // And the closing comparison is the verifying form, so no equality
    // result survives for a caller to read as truth.
    assert_eq!(
        instructions.last(),
        Some(&TapscriptInstruction::Opcode(OpcodeId::EqualVerify)),
    );
}

#[test]
fn the_conservation_is_a_function_of_the_two_counts_and_of_nothing_else() {
    // §1.3's value-parametric rule, as a property of the emitted bytes:
    // two shapes agreeing on the receipt counts and differing in every
    // other axis emit the same conservation, so no amount and no sponsor
    // fact can have reached it.
    let target = reviewed_target();
    let sponsorless = explicit_conservation_fragment(&target, plain(2, 3))
        .expect("the conservation fragment assembles");
    let sponsored =
        explicit_conservation_fragment(&target, shape(2, 3, 1, SponsorChangePresence::Present))
            .expect("the conservation fragment assembles");

    assert_eq!(sponsorless, sponsored);

    // The only fixed-width literals it pushes are the semantic domain's
    // two bounds. A third would be an amount, and there is none — which
    // is why split, merge, and redistribution are the same instructions
    // over different counts rather than different programs.
    let wide: BTreeSet<Vec<u8>> = pushed(&sponsorless)
        .iter()
        .filter(|item| item.len() == 8)
        .map(|item| item.bytes().to_vec())
        .collect();
    assert_eq!(
        wide.len(),
        2,
        "the fragment pushes an eight-byte literal that is not a domain bound"
    );
}

#[test]
fn split_merge_and_redistribution_are_the_same_relation_over_different_counts() {
    // §5.3's four compositions, each emitted, walked, and held to §10.9.
    // No one-to-one evidence is being read as general support: every one
    // of them is built and validated in its own right.
    let target = reviewed_target();
    let symbols = symbols();
    let constructor = explicit();

    for (inputs, outputs) in [(1, 1), (1, 3), (3, 1), (3, 3)] {
        let subject = plain(inputs, outputs);
        let program = live_coordinator_program(&target, &symbols, &constructor, subject)
            .expect("the coordinator program emits");

        assert_eq!(
            final_stack_defects(&target, &program, &live_program_precondition(&target))
                .expect("the program walks"),
            Vec::new(),
            "the {inputs}-to-{outputs} coordinator does not satisfy §10.9",
        );
    }
}

#[test]
fn an_addition_whose_flag_is_not_consumed_leaves_the_overflow_result_behind() {
    // The executed form of the discipline above, over the prefix of the
    // fragment that ends at its first addition. With the verification the
    // overflow ends the spend and no failure state survives; without it
    // the target's false is still on the stack, which is precisely the
    // unchecked arithmetic result §6.2 forbids.
    let target = reviewed_target();
    let fragment = explicit_conservation_fragment(&target, plain(2, 2))
        .expect("the conservation fragment assembles");
    let instructions = fragment.instructions();
    let addition = instructions
        .iter()
        .position(|instruction| *instruction == TapscriptInstruction::Opcode(OpcodeId::Add64))
        .expect("the fragment adds");
    assert_eq!(
        instructions.get(addition + 1),
        Some(&TapscriptInstruction::Opcode(OpcodeId::Verify)),
    );

    let prefix = |through: usize| {
        walk(
            &TapscriptProgram::new(instructions[..=through].to_vec())
                .expect("the prefix is within the bound"),
        )
    };

    let unchecked = prefix(addition);
    assert!(
        !unchecked.nonaborting_failure().is_empty(),
        "the addition has no overflow form for the verification to consume",
    );

    let checked = prefix(addition + 1);
    assert!(
        checked.nonaborting_failure().is_empty(),
        "the verification left the overflow result behind: {:?}",
        checked.nonaborting_failure(),
    );
    // And the successful form survives, so the verification refuses the
    // overflow rather than refusing the addition.
    assert!(!checked.success().is_empty());
}

// --- §10.7: sponsor isolation -----------------------------------------

#[test]
fn the_sponsor_fragment_holds_no_primitive_that_could_read_an_amount() {
    // §1.9, as a property of the emitted bytes. Not a value introspection
    // whose result is discarded, and not a comparison with zero: no
    // primitive that could read an amount is scheduled at all.
    let target = reviewed_target();
    let symbols = symbols();

    for subject in demonstration_live_shape_set().shapes() {
        let fragment = live_sponsor_isolation_fragment(&target, &symbols, subject)
            .expect("the isolation fragment assembles");
        assert!(
            !reads_a_value_field(&fragment),
            "the sponsor isolation of {subject:?} introspects a value field",
        );
        assert!(!opens_an_amount(&fragment));
    }

    // And neither property is vacuous: the conservation fragment, which
    // is the one fragment allowed to read amounts, does both.
    let conservation = explicit_conservation_fragment(&target, plain(2, 2))
        .expect("the conservation fragment assembles");
    assert!(reads_a_value_field(&conservation));
    assert!(opens_an_amount(&conservation));
}

#[test]
fn recognition_stays_amount_free_now_that_the_coordinator_reads_amounts() {
    // Wave 5's property, held. The coordinator reads amounts in exactly
    // one fragment, and the per-input recognition every receipt runs —
    // the private plan's included — is not it.
    let target = reviewed_target();
    for representation in [
        LiveTransferRepresentationPlan::Explicit,
        LiveTransferRepresentationPlan::PrivateCommitted,
    ] {
        let fragment = local_recognition_fragment(&target, &symbols(), representation)
            .expect("the recognition fragment assembles");
        // It reads the value *field*, because which form the field is in
        // is exactly what §10.1 has it establish — and it opens nothing,
        // because the payload is dropped where it stands.
        assert!(reads_a_value_field(&fragment));
        assert!(
            !opens_an_amount(&fragment),
            "the {representation:?} recognition opens an amount",
        );
    }
}

#[test]
fn every_sponsor_input_and_output_role_is_required_to_carry_the_reserve_asset() {
    let target = reviewed_target();
    let symbols = symbols();
    let subject = shape(1, 1, 1, SponsorChangePresence::Present);
    let fragment = live_sponsor_isolation_fragment(&target, &symbols, subject)
        .expect("the isolation fragment assembles");

    // One sponsor input, the change role, and the fee role: three asset
    // comparisons, all against the reserve asset.
    assert_eq!(schedules(&fragment, OpcodeId::InspectInputAsset), 1);
    assert_eq!(schedules(&fragment, OpcodeId::InspectOutputAsset), 2);
    assert_eq!(
        pushed(&fragment)
            .iter()
            .filter(|item| *item == symbols.reserve_asset())
            .count(),
        3,
    );
    // The change role's program and the fee role's digest are both
    // compared, which is what makes the roles exact rather than merely
    // reserve-asset-carrying.
    assert!(
        pushed(&fragment)
            .iter()
            .any(|item| item == symbols.sponsor_change_program()),
    );
    assert!(
        pushed(&fragment)
            .iter()
            .any(|item| item == symbols.fee_program_digest()),
    );
}

#[test]
fn a_present_sponsor_change_cannot_be_materialized_as_an_absent_one() {
    // The tapscript half of the preflight's `G13-R10`: the change role's
    // presence is a declared axis of the shape, it moves the emitted
    // bytes, and it moves the exact output count the coordinator pins. A
    // transaction built without the change role therefore fails a shape
    // whose class claims one, rather than passing as the sponsorless form.
    let target = reviewed_target();
    let symbols = symbols();
    let present = shape(1, 1, 1, SponsorChangePresence::Present);
    let absent = shape(1, 1, 1, SponsorChangePresence::Absent);

    assert_ne!(present.outputs(), absent.outputs());
    assert_ne!(
        live_sponsor_isolation_fragment(&target, &symbols, present)
            .expect("the isolation fragment assembles"),
        live_sponsor_isolation_fragment(&target, &symbols, absent)
            .expect("the isolation fragment assembles"),
    );
    assert_ne!(
        crate::live_pattern::live_cardinality_fragment(&target, present)
            .expect("the cardinality fragment assembles"),
        crate::live_pattern::live_cardinality_fragment(&target, absent)
            .expect("the cardinality fragment assembles"),
    );
}

#[test]
fn a_sponsorless_shape_has_its_region_settled_by_the_counts_alone() {
    let subject = plain(2, 2);
    assert!(!has_sponsor_region(subject));
    assert_eq!(
        live_sponsor_isolation_fragment(&reviewed_target(), &symbols(), subject)
            .expect("the isolation fragment assembles")
            .instructions(),
        [],
    );
    // Which is not a hole: the shape's own input and output counts leave
    // no position for a sponsor role to occupy.
    assert_eq!(
        u32::from(subject.inputs()),
        u32::from(subject.receipt_inputs())
    );
    assert_eq!(
        u32::from(subject.outputs()),
        u32::from(subject.receipt_outputs()),
    );
}

// --- §10.8: absence relations -----------------------------------------

#[test]
fn every_input_is_tested_for_an_issuance_including_the_sponsors() {
    let target = reviewed_target();

    for subject in demonstration_live_shape_set().shapes() {
        let fragment =
            issuance_absence_fragment(&target, subject).expect("the absence fragment assembles");
        assert_eq!(
            schedules(&fragment, OpcodeId::InspectInputIssuance),
            usize::from(subject.inputs()),
            "{subject:?} leaves an input untested for an issuance",
        );
    }
}

#[test]
fn an_input_carrying_an_issuance_has_no_successful_path() {
    // The reviewed introspection's two forms, separated. The absent form
    // consumes its marker and leaves the stack as it found it; the
    // present form pushes six items whose top is a thirty-two byte
    // blinding nonce, and the comparison against the empty marker leaves
    // it nowhere to go.
    let fragment = issuance_absence_fragment(&reviewed_target(), plain(1, 1))
        .expect("the absence fragment assembles");
    let result = walk(&fragment);

    assert_eq!(
        result.success().iter().cloned().collect::<Vec<_>>(),
        vec![AbstractStackState::from_main(Vec::new())],
        "the issuance-present form survived the comparison",
    );
    assert!(result.nonaborting_failure().is_empty());
}

#[test]
fn the_forbidden_object_families_have_no_position_to_occupy() {
    // §10.8's roots, reserves, projected events and burn records are
    // objects, and an object needs a position. Every position of every
    // admitted shape is held by exactly one family the coordinator
    // authenticates, and the exact counts leave none over — so the
    // absence is a property of the census rather than a list of tests
    // that would go out of date the moment the architecture named one
    // more family.
    for subject in demonstration_live_shape_set().shapes() {
        let ranges = live_family_ranges(subject);
        assert_eq!(family_range_defects(&ranges), Vec::new());

        let positions: usize = ranges
            .ranges()
            .map(|range| usize::from(range.count()))
            .sum();
        assert_eq!(
            positions,
            usize::from(subject.inputs()) + usize::from(subject.outputs()),
        );
    }
}

// --- The composed explicit coordinator --------------------------------

#[test]
fn the_explicit_coordinator_carries_every_global_fragment_it_claims() {
    let target = reviewed_target();
    let symbols = symbols();
    let constructor = explicit();
    let subject = shape(2, 2, 1, SponsorChangePresence::Present);
    let program = live_coordinator_program(&target, &symbols, &constructor, subject)
        .expect("the coordinator program emits");

    for fragment in [
        destination_closure_fragment(&target, &symbols, subject)
            .expect("the closure fragment assembles"),
        live_sponsor_isolation_fragment(&target, &symbols, subject)
            .expect("the isolation fragment assembles"),
        issuance_absence_fragment(&target, subject).expect("the absence fragment assembles"),
        explicit_conservation_fragment(&target, subject)
            .expect("the conservation fragment assembles"),
    ] {
        let (whole, part) = (program.instructions(), fragment.instructions());
        assert!(
            !part.is_empty() && whole.windows(part.len()).any(|window| window == part),
            "the coordinator does not carry a fragment its slot census claims",
        );
    }
}

#[test]
fn the_private_coordinator_carries_every_global_fragment_but_the_arithmetic() {
    // §10.6 admits no amount inspection, so the private coordinator
    // carries the destination closure, the sponsor isolation and the
    // issuance absence — none of which depends on how a value is carried
    // — and not the conservation.
    let target = reviewed_target();
    let symbols = symbols();
    let representation = LiveTransferRepresentationPlan::PrivateCommitted;
    let shapes = demonstration_live_shape_set();
    let closure = owner_key_encoding_closure(target.definition().authorization());
    let owner = OwnerKey::new(&closure, closure.approved(), vec![0x11; 32])
        .expect("the fixture is the approved encoding at its exact width");
    let constructor = derive_live_receipt_constructor(
        &target,
        &live_transfer_plan(),
        representation,
        owner,
        shapes.clone(),
        static_transfer_leaf_set(representation, &shapes),
    )
    .expect("the private constructor derives");

    let subject = shape(2, 2, 1, SponsorChangePresence::Present);
    let program = live_coordinator_program(&target, &symbols, &constructor, subject)
        .expect("the coordinator program emits");

    assert!(
        !opens_an_amount(&program),
        "the private coordinator opens an amount",
    );
    assert_eq!(
        schedules(&program, OpcodeId::InspectOutputAsset),
        usize::from(subject.receipt_outputs()) + 2,
    );
    assert_eq!(
        schedules(&program, OpcodeId::InspectInputIssuance),
        usize::from(subject.inputs()),
    );
}
