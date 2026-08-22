//! Oracles for the live-transfer recognition and authorization patterns
//! (Guide-13 §10).
//!
//! # What is checked against what
//!
//! Nothing here asks a pattern to confirm its own reasoning. The
//! owner-key width is read from the *target's* encoding registry, so a
//! test that passed because this module and the registry agreed on a
//! wrong width could not exist. The §1.8 negatives are not read off a
//! table: every mutation the census names is applied to the emitted
//! fragment, the mutant is walked, and the outcome the walk reports is
//! required to be the one the census claims. And the §10.3 slot census
//! is validated against the fragments a real coordinator and a real
//! member program are built from, rather than against a second list.
//!
//! # The owner keys below are public test material
//!
//! Fixed byte patterns of the approved encoding's exact width:
//! distinguishable, meaningless, and behind none of them a secret. They
//! are public disposable test data in the sense
//! `(´[ADR015-rule:security:test-material]´)` fixes, and no test here
//! signs anything — a signature over a target sighash is the conformance
//! package's business, and this module's subject is which forms the
//! emitted bytes leave reachable.

use std::collections::BTreeSet;
use std::num::NonZeroU8;

use compiler::live_transfer_plan::LiveTransferRepresentationPlan;
use target_elements::{
    EncodingClass, FailureCause, OpcodeId, PayloadWidth, ReviewedElementsTapscriptDefinition,
};

use super::{live_transfer_plan, live_transfer_symbols, reviewed_target};
use crate::authorization::{
    OwnerKeyEncodingClosure, OwnerKeyNegative, OwnerKeyObligation, OwnerProfileDisposition,
    owner_key_encoding_closure,
};
use crate::instruction::{StackItem, TapscriptInstruction};
use crate::live_constructor::{
    LiveProgramRole, OwnerKey, OwnerKeyRejection, StaticLiveReceiptConstructor,
    derive_live_receipt_constructor, static_transfer_leaf_set,
};
use crate::live_pattern::{
    CoordinatorGlobalCheck, FinalStackDefect, GlobalCheckStatus, LiveFragmentId,
    LiveProgramRefusal, LiveTransferPatternId, LiveTransferSymbols, LiveWitnessRole,
    NegativeDisposition, OutstandingGlobalPattern, OwnerAssignmentRejection, OwnerKeyMutation,
    OwnerKeyMutationGate, OwnerKeyMutationOutcome, OwnerKeyOracle, PlacementDefect,
    ReceiptOwnerAssignment, RecognitionCarrier, RecognitionResidual, RecognizedFact,
    coordinator_placements, emitted_fragments, every_emitted_fragment, final_stack_defects,
    has_member_position, live_coordinator_program, live_member_program,
    live_owner_profile_disposition, live_program_precondition, live_transfer_patterns,
    local_recognition_fragment, mutated_owner_authorization_fragment, mutation_gate,
    negative_disposition, owner_authorization_fragment, owner_authorization_precondition,
    owner_key_mutation_outcome, owner_key_obligation, patterns_for, recognition_establishments,
    validate_coordinator_placements,
};
use crate::live_plan::{has_sponsor_region, live_sponsor_isolation_fragment};
use crate::live_shape::{LiveTransferShape, demonstration_live_shape_set};
use crate::pattern::final_truth_fragment;
use crate::program::TapscriptProgram;
use crate::shape::SponsorChangePresence;
use crate::stack::{AbstractLimits, SignatureSuccessForm, validate_program};

// --- Fixtures ---------------------------------------------------------

/// The reviewed contract's owner-key encoding closure.
fn closure() -> OwnerKeyEncodingClosure {
    owner_key_encoding_closure(reviewed_target().definition().authorization())
}

/// The approved encoding's exact width, read from the target's registry.
///
/// # Panics
///
/// If the approved owner-key encoding stops fixing one exact width, in
/// which case these fixtures have nothing to be the right width of.
fn approved_width() -> usize {
    match closure().approved().v1_shape().payload() {
        PayloadWidth::Exact(width) => width.get(),
        other => panic!("the approved owner-key encoding fixes no exact width: {other:?}"),
    }
}

/// One owner key of the approved encoding, filled with `fill`.
///
/// # Panics
///
/// If the fixture is not the approved encoding at its exact width, which
/// would make this helper rather than the pattern the thing under test.
fn owner(fill: u8) -> OwnerKey {
    OwnerKey::new(
        &closure(),
        closure().approved(),
        vec![fill; approved_width()],
    )
    .expect("the fixture is the approved encoding at its exact width")
}

/// The link-time symbol set, shared with the plan oracles.
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

/// One shape of `receipt_inputs` receipts, sponsorless.
///
/// # Panics
///
/// If the demonstration bounds stop admitting it, which would make the
/// fixture rather than the pattern the thing under test.
fn shape(receipt_inputs: u8) -> LiveTransferShape {
    LiveTransferShape::new(
        demonstration_live_shape_set().bounds(),
        count(receipt_inputs),
        count(2),
        0,
        SponsorChangePresence::Absent,
    )
    .expect("the demonstration bounds admit the fixture shape")
}

/// One shape of `receipt_inputs` receipts, with a sponsor envelope and
/// its change role.
///
/// # Panics
///
/// If the demonstration bounds stop admitting it, which would make the
/// fixture rather than the pattern the thing under test.
fn sponsored_shape(receipt_inputs: u8) -> LiveTransferShape {
    LiveTransferShape::new(
        demonstration_live_shape_set().bounds(),
        count(receipt_inputs),
        count(2),
        1,
        SponsorChangePresence::Present,
    )
    .expect("the demonstration bounds admit the sponsored fixture shape")
}

/// The reference constructor for one representation plan.
///
/// # Panics
///
/// If the reference parts stop deriving a constructor, which Wave 4's
/// own oracles would have caught first.
fn constructor(representation: LiveTransferRepresentationPlan) -> StaticLiveReceiptConstructor {
    let shapes = demonstration_live_shape_set();
    let leaves = static_transfer_leaf_set(representation, &shapes);

    derive_live_receipt_constructor(
        &reviewed_target(),
        &live_transfer_plan(),
        representation,
        owner(0x11),
        shapes,
        leaves,
    )
    .expect("the reference subject derives")
}

/// The explicit-plan reference constructor.
fn explicit() -> StaticLiveReceiptConstructor {
    constructor(LiveTransferRepresentationPlan::Explicit)
}

/// Whether `whole` carries `part` as a contiguous run.
fn carries(whole: &TapscriptProgram, part: &TapscriptProgram) -> bool {
    let (whole, part) = (whole.instructions(), part.instructions());
    !part.is_empty() && whole.windows(part.len()).any(|window| window == part)
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

/// Whether a program schedules one reviewed primitive.
fn calls(program: &TapscriptProgram, id: OpcodeId) -> bool {
    program
        .instructions()
        .contains(&TapscriptInstruction::Opcode(id))
}

/// The reviewed prefix byte of one encoding class.
///
/// # Panics
///
/// If the class states no prefix, which the value classes do.
fn class_prefix(target: &ReviewedElementsTapscriptDefinition, class: EncodingClass) -> u8 {
    target
        .definition()
        .encodings()
        .get(&class)
        .and_then(|spec| spec.prefixes().iter().next().copied())
        .expect("the value encodings are prefix discriminated")
}

// --- §10.1: local predecessor recognition -----------------------------

#[test]
fn recognition_compares_the_exact_linked_asset_and_nothing_like_it() {
    let target = reviewed_target();
    let symbols = symbols();
    let fragment =
        local_recognition_fragment(&target, &symbols, LiveTransferRepresentationPlan::Explicit)
            .expect("the recognition fragment assembles");

    // Three literals and no others: the explicit-asset prefix, the exact
    // linked asset, and the selected plan's value prefix. A fragment
    // that compared against anything else would have a fourth.
    let literals = pushed(&fragment);
    assert_eq!(literals.len(), 3);
    assert_eq!(
        literals[0].bytes(),
        &[class_prefix(&target, EncodingClass::ExplicitAsset)],
    );
    assert_eq!(literals[1], *symbols.protocol_asset());
    assert_eq!(
        literals[2].bytes(),
        &[class_prefix(&target, EncodingClass::ExplicitValue)],
    );
}

#[test]
fn recognition_reads_the_value_form_and_never_an_amount() {
    let target = reviewed_target();
    let fragment = local_recognition_fragment(
        &target,
        &symbols(),
        LiveTransferRepresentationPlan::Explicit,
    )
    .expect("the recognition fragment assembles");

    // §10.6 admits no amount inspection in the private plan, and the
    // same fragment serves both plans, so it can read no amount in
    // either. Its absence is checked on the emitted primitives rather
    // than argued: no arithmetic, no ordering, no slicing.
    for arithmetic in [
        OpcodeId::Add64,
        OpcodeId::Sub64,
        OpcodeId::LessThan64,
        OpcodeId::GreaterThan64,
        OpcodeId::Substring,
        OpcodeId::Le64ToScriptNum,
    ] {
        assert!(
            !calls(&fragment, arithmetic),
            "recognition schedules {arithmetic:?}, which reads an amount",
        );
    }

    // And it leaves nothing behind for a later fragment to read as one.
    let result = validate_program(
        &target,
        &fragment,
        &crate::stack::AbstractStackState::from_main(Vec::new()),
        AbstractLimits::for_target(&target),
    )
    .expect("the recognition fragment validates");
    for state in result.success() {
        assert!(
            state.main().is_empty(),
            "recognition left an operand behind: {state:?}",
        );
    }
}

#[test]
fn recognition_speaks_only_about_the_input_the_leaf_runs_in() {
    let fragment = local_recognition_fragment(
        &reviewed_target(),
        &symbols(),
        LiveTransferRepresentationPlan::Explicit,
    )
    .expect("the recognition fragment assembles");

    // Every introspection is indexed by the target's own current-input
    // index. A compiled-in index would be a cross-input claim, and §10.1
    // admits no cross-input shortcut — see the module documentation for
    // why an owner-parameterized constructor could not support one.
    let instructions = fragment.instructions();
    for (position, instruction) in instructions.iter().enumerate() {
        let TapscriptInstruction::Opcode(
            id @ (OpcodeId::InspectInputAsset | OpcodeId::InspectInputValue),
        ) = instruction
        else {
            continue;
        };
        assert_eq!(
            position
                .checked_sub(1)
                .and_then(|before| instructions.get(before)),
            Some(&TapscriptInstruction::Opcode(
                OpcodeId::PushCurrentInputIndex
            )),
            "{id:?} is indexed by something other than the current input",
        );
    }
}

#[test]
fn the_value_form_follows_the_selected_representation() {
    let target = reviewed_target();
    let symbols = symbols();

    let explicit =
        local_recognition_fragment(&target, &symbols, LiveTransferRepresentationPlan::Explicit)
            .expect("the explicit fragment assembles");
    let private = local_recognition_fragment(
        &target,
        &symbols,
        LiveTransferRepresentationPlan::PrivateCommitted,
    )
    .expect("the private fragment assembles");

    assert_ne!(explicit, private);
    assert_eq!(
        pushed(&private)[2].bytes(),
        &[class_prefix(&target, EncodingClass::ConfidentialValue)],
    );
}

#[test]
fn every_recognized_fact_has_a_carrier() {
    let establishments = recognition_establishments();

    assert_eq!(establishments.len(), RecognizedFact::ALL.len());
    for fact in RecognizedFact::ALL {
        assert!(
            establishments.contains_key(fact),
            "§10.1 names {fact:?} and this wave says nothing about it",
        );
    }
}

#[test]
fn the_constructor_of_another_input_is_carried_as_a_residual() {
    let establishments = recognition_establishments();
    let constructor_fact = &establishments[&RecognizedFact::LiveReceiptConstructor];

    // The honest limit of a local recognition: the leaf's own commitment
    // settles its own input, and this wave establishes nothing about any
    // other. A carrier of `EmittedInstructions` here would be the claim
    // the module documentation explains cannot be made.
    assert_eq!(
        constructor_fact.carrier(),
        RecognitionCarrier::LeafCommitment
    );
    assert!(
        constructor_fact
            .residuals()
            .any(|residual| residual == RecognitionResidual::LinkedDestinationConstructorIdentity),
    );
}

#[test]
fn the_live_class_is_carried_by_the_type_and_not_by_a_check() {
    let establishments = recognition_establishments();

    // §7.3: there is no class field to compare against, because a
    // live-transfer leaf is not a leaf of any other operation's
    // constructor. A carrier that claimed emitted instructions would be
    // claiming bytes that do not exist.
    assert_eq!(
        establishments[&RecognizedFact::LiveClass].carrier(),
        RecognitionCarrier::ConstructorTyping,
    );
}

// --- §10.2 and §1.8: owner authorization ------------------------------

#[test]
fn the_owner_key_is_pushed_as_the_approved_encoding_at_its_exact_width() {
    let target = reviewed_target();
    let owner = owner(0x11);
    let fragment = owner_authorization_fragment(&target, &owner).expect("the fragment assembles");

    let literals = pushed(&fragment);
    assert_eq!(literals.len(), 1);
    assert_eq!(literals[0].len(), approved_width());
    assert_eq!(literals[0].bytes(), owner.bytes());
}

#[test]
fn the_authorization_fragment_reaches_only_the_verified_form() {
    // The whole of §1.8, as an executed statement rather than a claim:
    // the key is a literal of the approved encoding at its exact width,
    // so the target's forward-compatibility form is not something this
    // program can reach at all.
    let target = reviewed_target();
    let fragment =
        owner_authorization_fragment(&target, &owner(0x11)).expect("the fragment assembles");
    let result = validate_program(
        &target,
        &fragment,
        &owner_authorization_precondition(&target),
        AbstractLimits::for_target(&target),
    )
    .expect("the authorization fragment validates");

    assert!(!result.reaches_unverified_signature_success());
    assert_eq!(
        result
            .signature_forms()
            .values()
            .cloned()
            .collect::<Vec<_>>(),
        vec![BTreeSet::from([
            SignatureSuccessForm::RecognizedKeyVerified
        ])],
    );
    // A verification that can happen can also fail, which is the other
    // half of the same finding.
    assert!(result.aborts().contains(&FailureCause::InvalidSignature));
}

#[test]
fn the_authorization_fragment_rejects_the_empty_signature_and_branches_on_nothing() {
    let target = reviewed_target();
    let fragment =
        owner_authorization_fragment(&target, &owner(0x11)).expect("the fragment assembles");
    let result = validate_program(
        &target,
        &fragment,
        &owner_authorization_precondition(&target),
        AbstractLimits::for_target(&target),
    )
    .expect("the authorization fragment validates");

    // The verifying form aborts on the empty offering rather than
    // pushing a false somebody could reduce later.
    assert!(result.aborts().contains(&FailureCause::EmptySignature));
    assert!(result.nonaborting_failure().is_empty());
    assert_eq!(
        result.success().iter().cloned().collect::<Vec<_>>(),
        vec![crate::stack::AbstractStackState::from_main(Vec::new())],
    );
}

#[test]
fn the_target_obliges_the_pattern_to_authenticate_the_encoding_itself() {
    // Read from the constructor's retained closure rather than assumed:
    // this is the reviewed target's own unknown-key rule, and it is why
    // the key is a literal at all.
    assert_eq!(
        owner_key_obligation(&explicit()),
        OwnerKeyObligation::AuthenticateEncodingIndependently,
    );
}

#[test]
fn the_empty_key_mutant_has_no_successful_path() {
    let outcome =
        owner_key_mutation_outcome(&reviewed_target(), &owner(0x11), OwnerKeyMutation::EmptyKey)
            .expect("the mutant walks");

    assert_eq!(outcome, OwnerKeyMutationOutcome::NoSuccessfulPath);
}

#[test]
fn the_unknown_key_mutant_reopens_the_forward_compatibility_path() {
    // The negative that gives the pattern its content. Replace the
    // approved-encoding literal with a width the target does not
    // recognize and the program keeps a successful form — one that
    // verified nothing.
    let outcome = owner_key_mutation_outcome(
        &reviewed_target(),
        &owner(0x11),
        OwnerKeyMutation::UnknownNonemptyKeyType,
    )
    .expect("the mutant walks");

    assert_eq!(outcome, OwnerKeyMutationOutcome::ReachesUnverifiedSuccess);
}

#[test]
fn an_approved_width_mutant_is_indistinguishable_and_names_its_oracle() {
    let target = reviewed_target();
    let owner = owner(0x11);

    // Both mutations keep the approved encoding's exact width, so the
    // walk reaches the pattern's own contract for each. Reporting that
    // as a pass would be the mistake; each names the oracle that decides
    // it instead.
    assert_eq!(
        owner_key_mutation_outcome(&target, &owner, OwnerKeyMutation::MalformedApprovedKey)
            .expect("the mutant walks"),
        OwnerKeyMutationOutcome::AbstractlyIndistinguishable {
            oracle: OwnerKeyOracle::CurvePointMembership,
        },
    );
    assert_eq!(
        owner_key_mutation_outcome(&target, &owner, OwnerKeyMutation::ApprovedKeyOfAnotherOwner)
            .expect("the mutant walks"),
        OwnerKeyMutationOutcome::AbstractlyIndistinguishable {
            oracle: OwnerKeyOracle::SignatureVerification,
        },
    );
}

#[test]
fn the_owner_metadata_gate_refuses_exactly_the_two_encoding_mutations() {
    // Not read off the census: the refusals the gate is claimed to make
    // are executed against §7.2's own constructor.
    assert_eq!(
        OwnerKey::new(&closure(), closure().approved(), Vec::new()),
        Err(OwnerKeyRejection::OwnerOmitted),
    );
    assert!(matches!(
        OwnerKey::new(
            &closure(),
            EncodingClass::CompressedPublicKey,
            vec![0x02; approved_width()],
        ),
        Err(OwnerKeyRejection::AlternateEncodingOfApprovedKey { .. }),
    ));

    // And the two it admits are admitted, which is where §7.2's
    // authority ends and an oracle's begins.
    for mutation in [
        OwnerKeyMutation::MalformedApprovedKey,
        OwnerKeyMutation::ApprovedKeyOfAnotherOwner,
    ] {
        assert_eq!(
            mutation_gate(mutation),
            OwnerKeyMutationGate::AdmittedByOwnerMetadata,
        );
    }
    for mutation in [
        OwnerKeyMutation::EmptyKey,
        OwnerKeyMutation::UnknownNonemptyKeyType,
    ] {
        assert_eq!(
            mutation_gate(mutation),
            OwnerKeyMutationGate::RefusedByOwnerMetadata,
        );
    }
}

#[test]
fn every_owner_key_negative_is_routed_to_an_executable_case() {
    let target = reviewed_target();
    let owner = owner(0x11);

    for negative in OwnerKeyNegative::ALL {
        match negative_disposition(*negative) {
            NegativeDisposition::FragmentMutation { mutation } => {
                // Executed, not asserted: the mutant is built and walked,
                // and a mutation the walk could not schedule would fail
                // here rather than pass by being unreachable.
                let mutant = mutated_owner_authorization_fragment(&target, &owner, mutation)
                    .expect("the mutant assembles");
                assert_ne!(
                    mutant,
                    owner_authorization_fragment(&target, &owner).expect("the fragment assembles"),
                    "{negative:?} names a mutation that changes nothing",
                );
                owner_key_mutation_outcome(&target, &owner, mutation).expect("the mutant walks");
            }
            NegativeDisposition::WitnessSubstitution { oracle } => {
                // The two cases no property of the emitted bytes could
                // decide: one signature is as well formed as another.
                assert_eq!(oracle, OwnerKeyOracle::SignatureVerification);
            }
        }
    }
}

// --- §10.3: coordinator and member placement --------------------------

#[test]
fn the_coordinator_binds_itself_to_input_zero_before_anything_else() {
    let target = reviewed_target();
    let program = live_coordinator_program(&target, &symbols(), &explicit(), shape(2))
        .expect("the coordinator program emits");
    let anchor = crate::pattern::coordinator_role_fragment(&target).expect("the anchor assembles");

    assert_eq!(
        &program.instructions()[..anchor.instructions().len()],
        anchor.instructions(),
    );
}

#[test]
fn a_member_leaf_bounds_itself_to_the_nonzero_receipt_positions() {
    let target = reviewed_target();
    let program =
        live_member_program(&target, &symbols(), &explicit(), 3).expect("the member program emits");
    let range = crate::live_pattern::live_member_role_fragment(&target, 3)
        .expect("the range fragment assembles");

    assert_eq!(
        &program.instructions()[..range.instructions().len()],
        range.instructions(),
    );
}

#[test]
fn the_coordinator_performs_the_local_pair_like_every_other_input() {
    // §10.3's last sentence, checked on the emitted bytes: the
    // coordinator is not exempt from the recognition and authorization
    // every receipt input runs.
    let target = reviewed_target();
    let symbols = symbols();
    let constructor = explicit();
    let program = live_coordinator_program(&target, &symbols, &constructor, shape(2))
        .expect("the coordinator program emits");

    let recognition = local_recognition_fragment(&target, &symbols, constructor.representation())
        .expect("the recognition fragment assembles");
    let authorization =
        owner_authorization_fragment(&target, constructor.owner()).expect("the fragment assembles");

    assert!(carries(&program, &recognition));
    assert!(carries(&program, &authorization));
}

#[test]
fn a_member_program_does_not_reprove_the_transaction_shape() {
    // A member leaf serves every shape of one receipt-input count, so it
    // has no single pair of counts to authenticate. Emitting one anyway
    // would tie the leaf to a shape and cost every other shape a leaf.
    let program = live_member_program(&reviewed_target(), &symbols(), &explicit(), 3)
        .expect("the member program emits");

    assert!(!calls(&program, OpcodeId::InspectNumInputs));
    assert!(!calls(&program, OpcodeId::InspectNumOutputs));
}

#[test]
fn the_one_to_one_shape_has_no_member_program() {
    let refusal = live_member_program(&reviewed_target(), &symbols(), &explicit(), 1)
        .expect_err("a one-receipt transfer has no nonzero receipt position");

    assert!(matches!(
        refusal,
        LiveProgramRefusal::ShapeHasNoMemberPosition { .. },
    ));
}

#[test]
fn a_shape_the_constructor_does_not_admit_is_refused() {
    let outside = LiveTransferShape::new(
        crate::live_shape::LiveTransferShapeBounds::new(count(9), count(9), 0),
        count(9),
        count(9),
        0,
        SponsorChangePresence::Absent,
    )
    .expect("the wider bounds admit it");

    assert_eq!(
        live_coordinator_program(&reviewed_target(), &symbols(), &explicit(), outside),
        Err(LiveProgramRefusal::ShapeNotAdmitted { shape: outside }),
    );
}

#[test]
fn every_global_check_is_either_emitted_or_owed() {
    validate_coordinator_placements(&coordinator_placements(), &every_emitted_fragment())
        .expect("the slot census stands up");
}

#[test]
fn the_slot_census_still_reports_everything_this_candidate_does_not_build() {
    let placements = coordinator_placements();
    let owed = placements
        .values()
        .flat_map(crate::live_pattern::GlobalCheckPlacement::outstanding)
        .collect::<BTreeSet<_>>();

    // Both members, and no fewer. A census that had quietly stopped
    // naming one would be a coordinator with a check nobody owes.
    assert_eq!(
        owed,
        OutstandingGlobalPattern::ALL.iter().copied().collect(),
    );
    // And the two are the ones this candidate genuinely cannot emit:
    // another representation's conservation, and a value no in-script
    // comparison reaches.
    assert_eq!(
        owed,
        BTreeSet::from([
            OutstandingGlobalPattern::PrivateConservation,
            OutstandingGlobalPattern::DestinationConstructorIdentity,
        ]),
    );
}

#[test]
fn the_four_checks_that_are_not_settled_whole_name_what_they_owe() {
    let placements = coordinator_placements();
    let partial = placements
        .values()
        .filter(|placement| placement.status() == GlobalCheckStatus::Partial)
        .map(crate::live_pattern::GlobalCheckPlacement::check)
        .collect::<BTreeSet<_>>();

    // The conservation, because §10.6 is Wave 7's; and the three whose
    // object families could wear a destination's shape, because the
    // destination constructor's exact bytes are not recomputed in script.
    assert_eq!(
        partial,
        BTreeSet::from([
            CoordinatorGlobalCheck::RepresentationSpecificConservation,
            CoordinatorGlobalCheck::LiveClassOutputClosure,
            CoordinatorGlobalCheck::RootsAbsent,
            CoordinatorGlobalCheck::SpecializedEventsAbsent,
        ]),
    );

    // Nothing is Outstanding any more, which is the flip this wave is
    // for: every check has emitted bytes behind it.
    assert!(
        !placements
            .values()
            .any(|placement| placement.status() == GlobalCheckStatus::Outstanding),
    );
    assert_eq!(
        placements
            .values()
            .filter(|placement| placement.status() == GlobalCheckStatus::Established)
            .count(),
        CoordinatorGlobalCheck::ALL.len() - partial.len(),
    );
}

#[test]
fn a_slot_claiming_a_fragment_no_role_emits_is_refused() {
    // The validator has teeth: a census may not claim credit for bytes
    // nothing emits.
    let defect = validate_coordinator_placements(
        &coordinator_placements(),
        &BTreeSet::from([LiveFragmentId::FinalTruth]),
    )
    .expect_err("a slot claims fragments this set does not hold");

    assert!(matches!(defect, PlacementDefect::FragmentNotEmitted { .. }));
}

#[test]
fn the_coordinator_role_carries_the_counts_and_the_member_role_does_not() {
    let explicit_plan = LiveTransferRepresentationPlan::Explicit;
    assert!(
        emitted_fragments(LiveProgramRole::Coordinator, explicit_plan)
            .contains(&LiveFragmentId::Cardinality),
    );
    assert!(
        !emitted_fragments(LiveProgramRole::Member, explicit_plan)
            .contains(&LiveFragmentId::Cardinality)
    );
    // Both perform the local pair, which is §10.3's own sentence.
    for role in [LiveProgramRole::Coordinator, LiveProgramRole::Member] {
        let fragments = emitted_fragments(role, explicit_plan);
        assert!(fragments.contains(&LiveFragmentId::LocalRecognition));
        assert!(fragments.contains(&LiveFragmentId::OwnerAuthorization));
    }
}

// --- §1.6: three signatures from one owner are not three owners -------

#[test]
fn one_owner_holding_three_receipts_is_one_semantic_owner_and_three_signatures() {
    let assignment = ReceiptOwnerAssignment::new(vec![owner(0x11), owner(0x11), owner(0x11)])
        .expect("the assignment names three receipt inputs");

    assert_eq!(assignment.distinct_semantic_owners().len(), 1);
    assert_eq!(assignment.concrete_receipt_inputs(), 3);
    assert_eq!(assignment.concrete_owner_signatures(), 3);
}

#[test]
fn two_owners_across_three_receipts_are_two_semantic_owners() {
    let assignment = ReceiptOwnerAssignment::new(vec![owner(0x11), owner(0x22), owner(0x11)])
        .expect("the assignment names three receipt inputs");

    assert_eq!(assignment.distinct_semantic_owners().len(), 2);
    assert_eq!(assignment.concrete_owner_signatures(), 3);
    assert_eq!(assignment.owner_at(1), Some(&owner(0x22)));
}

#[test]
fn a_transfer_consuming_no_receipt_is_not_a_narrow_transfer() {
    assert_eq!(
        ReceiptOwnerAssignment::new(Vec::new()),
        Err(OwnerAssignmentRejection::NoReceiptInput),
    );
}

// --- §10.9: the final stack -------------------------------------------

#[test]
fn every_composed_program_satisfies_the_final_stack_rule() {
    let target = reviewed_target();
    let symbols = symbols();

    for representation in [
        LiveTransferRepresentationPlan::Explicit,
        LiveTransferRepresentationPlan::PrivateCommitted,
    ] {
        let constructor = constructor(representation);
        let programs = [
            live_coordinator_program(&target, &symbols, &constructor, shape(2))
                .expect("the coordinator program emits"),
            live_member_program(&target, &symbols, &constructor, 2)
                .expect("the member program emits"),
        ];

        for program in programs {
            assert_eq!(
                final_stack_defects(&target, &program, &live_program_precondition(&target))
                    .expect("the program walks"),
                Vec::new(),
                "{representation:?} program does not satisfy §10.9",
            );
        }
    }
}

#[test]
fn a_program_that_does_not_end_on_the_canonical_true_item_is_a_defect() {
    let target = reviewed_target();
    let fragment =
        owner_authorization_fragment(&target, &owner(0x11)).expect("the fragment assembles");

    let defects = final_stack_defects(&target, &fragment, &live_program_precondition(&target))
        .expect("the fragment walks");

    assert!(defects.contains(&FinalStackDefect::DoesNotEndOnTheCanonicalTrueItem));
}

#[test]
fn a_program_whose_key_is_not_the_approved_encoding_reports_the_forward_path() {
    // The §10.9 rule and the §1.8 rule meeting: a composed program that
    // can succeed without verifying is a program whose final truth means
    // nothing, and the defect census says so.
    let target = reviewed_target();
    let mut instructions = mutated_owner_authorization_fragment(
        &target,
        &owner(0x11),
        OwnerKeyMutation::UnknownNonemptyKeyType,
    )
    .expect("the mutant assembles")
    .instructions()
    .to_vec();
    instructions.extend_from_slice(
        final_truth_fragment(&target)
            .expect("the canonical true item assembles")
            .instructions(),
    );
    let program = TapscriptProgram::new(instructions).expect("the mutant program is short");

    let defects = final_stack_defects(&target, &program, &live_program_precondition(&target))
        .expect("the mutant walks");

    assert!(
        defects.iter().any(|defect| matches!(
            defect,
            FinalStackDefect::UnverifiedSignatureSuccessPath { .. }
        )),
        "the mutant program reports no forward-compatibility defect: {defects:?}",
    );
}

#[test]
fn a_false_comparison_followed_by_verify_has_no_abstract_success_path() {
    // §10.5's sentence, proved through composed instructions rather than
    // over one primitive: the canonical true item after the verification
    // is unreachable, so no program can reduce a false comparison to a
    // truth by appending one.
    let target = reviewed_target();
    let composed = |left: i64, right: i64| {
        let mut instructions = vec![
            TapscriptInstruction::Push(StackItem::signed_le64(&target, left)),
            TapscriptInstruction::Push(StackItem::signed_le64(&target, right)),
            TapscriptInstruction::Opcode(OpcodeId::LessThan64),
            TapscriptInstruction::Opcode(OpcodeId::Verify),
        ];
        instructions.extend_from_slice(
            final_truth_fragment(&target)
                .expect("the canonical true item assembles")
                .instructions(),
        );
        validate_program(
            &target,
            &TapscriptProgram::new(instructions).expect("the fixture is short"),
            &crate::stack::AbstractStackState::from_main(Vec::new()),
            AbstractLimits::for_target(&target),
        )
        .expect("the fixture validates")
    };

    assert!(composed(2, 1).always_aborts());
    // And the true direction is not vacuously aborting, which is what
    // makes the negative worth having.
    assert!(!composed(1, 2).always_aborts());
}

// --- The pattern census -----------------------------------------------

#[test]
fn every_pattern_identity_has_a_record_built_by_walking_its_fragment() {
    // A sponsored shape with a member position, which is the one shape
    // that calls for every identity: it has a nonzero receipt position,
    // it has a sponsor region, and the explicit plan reads amounts.
    let subject = sponsored_shape(2);
    let patterns = live_transfer_patterns(&reviewed_target(), &symbols(), &explicit(), subject)
        .expect("the census builds");

    assert_eq!(
        patterns_for(subject, LiveTransferRepresentationPlan::Explicit),
        LiveTransferPatternId::ALL.iter().copied().collect(),
    );
    assert_eq!(
        patterns.keys().copied().collect::<BTreeSet<_>>(),
        patterns_for(subject, LiveTransferRepresentationPlan::Explicit),
    );
    for pattern in patterns.values() {
        assert!(
            !pattern.prerequisites().is_empty(),
            "{:?} was minted over a fragment that schedules nothing",
            pattern.id(),
        );
        assert!(!pattern.evidence().is_empty());
    }
}

#[test]
fn a_sponsorless_shape_gets_no_sponsor_record_rather_than_an_empty_one() {
    // The sponsor-isolation fragment of a shape with no sponsor region is
    // zero instructions, because the region's absence is established by
    // the exact counts. A record over it would carry an empty
    // prerequisite census and an empty resource formula and would read,
    // from the outside, as a pattern somebody had checked.
    let subject = shape(2);
    let patterns = live_transfer_patterns(&reviewed_target(), &symbols(), &explicit(), subject)
        .expect("the census builds");

    assert!(!has_sponsor_region(subject));
    assert!(!patterns.contains_key(&LiveTransferPatternId::LiveSponsorIsolationV1));
    assert_eq!(
        live_sponsor_isolation_fragment(&reviewed_target(), &symbols(), subject)
            .expect("the fragment assembles")
            .instructions(),
        [],
    );
}

#[test]
fn the_private_plan_gets_no_conservation_record_rather_than_the_explicit_one() {
    // §10.6 admits no amount inspection at all, so the private plan does
    // not borrow the explicit plan's arithmetic. What it gets instead is
    // a named remainder on the coordinator's slot, which Wave 7 fills.
    let subject = shape(2);
    let constructor = constructor(LiveTransferRepresentationPlan::PrivateCommitted);
    let patterns = live_transfer_patterns(&reviewed_target(), &symbols(), &constructor, subject)
        .expect("the census builds");

    assert!(!patterns.contains_key(&LiveTransferPatternId::LiveExplicitConservationV1));
    assert_eq!(
        patterns.keys().copied().collect::<BTreeSet<_>>(),
        patterns_for(subject, LiveTransferRepresentationPlan::PrivateCommitted),
    );
    assert!(
        coordinator_placements()[&CoordinatorGlobalCheck::RepresentationSpecificConservation]
            .outstanding()
            .any(|pattern| pattern == OutstandingGlobalPattern::PrivateConservation),
    );
}

#[test]
fn the_one_to_one_shape_gets_no_member_record_rather_than_a_borrowed_one() {
    // A member record for a shape with no nonzero receipt position would
    // have to be either an unsatisfiable range check or the
    // coordinator's own program under the member identity. It is
    // neither: the identity is simply not called for.
    let subject = shape(1);
    let patterns = live_transfer_patterns(&reviewed_target(), &symbols(), &explicit(), subject)
        .expect("the census builds");

    assert!(!has_member_position(subject));
    assert!(!patterns.contains_key(&LiveTransferPatternId::LiveMemberRoleV1));
    assert!(!patterns.contains_key(&LiveTransferPatternId::LiveMemberProgramV1));
    assert_eq!(
        patterns.keys().copied().collect::<BTreeSet<_>>(),
        patterns_for(subject, LiveTransferRepresentationPlan::Explicit),
    );
    // And what it does hold is the coordinator's, under the
    // coordinator's own identity.
    assert!(patterns.contains_key(&LiveTransferPatternId::LiveCoordinatorProgramV1));
}

#[test]
fn no_pattern_in_the_census_can_succeed_without_verifying() {
    let patterns = live_transfer_patterns(&reviewed_target(), &symbols(), &explicit(), shape(2))
        .expect("the census builds");

    for pattern in patterns.values() {
        assert!(
            !pattern.reaches_unverified_success(),
            "{:?} has a success path that verified nothing",
            pattern.id(),
        );
    }
}

#[test]
fn every_pattern_asserting_a_signature_carries_the_unreviewed_profile_residual() {
    // §9.2's review is incomplete and this wave does not pretend
    // otherwise: a pattern that reads an owner signature is a candidate
    // statement until the profile is established.
    let patterns = live_transfer_patterns(&reviewed_target(), &symbols(), &explicit(), shape(2))
        .expect("the census builds");

    for pattern in patterns.values() {
        if pattern.witness() == LiveWitnessRole::OwnerSignature {
            assert!(
                pattern
                    .residuals()
                    .contains(&RecognitionResidual::SighashProfileUnreviewed),
                "{:?} asserts a signature and claims the profile is settled",
                pattern.id(),
            );
        }
    }
}

#[test]
fn the_unreviewed_profile_residual_is_a_computed_answer_and_not_a_caveat() {
    // The residual every signature-asserting pattern carries, checked
    // against the reviewed contract's own sighash capability rather than
    // taken on trust. A contract that reviewed the dimensions would move
    // this to `Established` and the residual would be the thing out of
    // date — which is the direction a caveat could never fail in.
    let disposition = live_owner_profile_disposition(&reviewed_target());

    assert!(
        matches!(
            disposition,
            OwnerProfileDisposition::ReviewIncomplete { .. }
        ),
        "the profile review is claimed complete: {disposition:?}",
    );
}

#[test]
fn the_owner_public_key_is_disclosed_exactly_where_it_is_pushed() {
    let patterns = live_transfer_patterns(&reviewed_target(), &symbols(), &explicit(), shape(2))
        .expect("the census builds");

    for pattern in patterns.values() {
        let pushes_the_key = pushed(pattern.fragment())
            .iter()
            .any(|item| item.bytes() == owner(0x11).bytes());
        assert_eq!(
            pushes_the_key,
            pattern
                .disclosure()
                .contains(&crate::live_pattern::LiveDisclosure::OwnerPublicKey),
            "{:?} disagrees with its own bytes about disclosing the owner key",
            pattern.id(),
        );
    }
}
