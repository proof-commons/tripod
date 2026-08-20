//! Machine-checked stack schedules for the compact-ASH proof patterns.
//!
//! # What makes a pattern operation-proven here
//!
//! A pattern is admitted to [`BackendPatternId`] only where its own
//! fragment has been walked through the abstract validator and the
//! resulting success, non-aborting failure, and abort sets check the
//! claim the pattern makes. Every assertion below is one of those three
//! sets compared exactly, never a containment: a containment check
//! would pass for a fragment that reached states the pattern never
//! claimed.
//!
//! # The three properties every schedule owes
//!
//! §12.11 asks each program for the same three things, and each is
//! checked here for every pattern rather than for a chosen example:
//! exactly one canonical successful shape; no state reached through a
//! non-aborting failure surviving into it; and no unconsumed
//! arithmetic or comparison flag left behind. The validator carries the
//! reached-through-failure bit itself, so the second is a checked
//! property rather than a claim.
//!
//! # Negative vectors
//!
//! Each pattern's fragment is mutated in the two ways a fragment can be
//! built wrong — a verification dropped, a literal corrupted — and the
//! mutant is required to lose the pattern's successful form or to gain
//! an abort. A mutation that changed nothing observable would be
//! coverage that proves nothing, so a mutant that still reaches exactly
//! the pattern's contract fails the test.
//!
//! # The symbols are fixtures
//!
//! The byte strings below are distinguishable placeholders standing in
//! for link-time symbols, not claims about any real object. They are
//! test material in the sense `(´[ADR015-rule:security:test-material]´)`
//! fixes: public, meaningless, and never a key.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU8;

use target_elements::{
    ElementsCapability, FailureCause, OpcodeId, ResourceDimension, StackValueType,
};

use crate::capability::BackendPatternId;
use crate::instruction::TapscriptInstruction;
use crate::pattern::{
    AMOUNT_OPERAND_BYTES, AbiAssumption, BackendPattern, CompactAshSymbols, MutationOutcome,
    PatternMutation, PatternOwner, ash_input_recognition_fragment, carries_authorization,
    coordinator_program, member_program, operand_type, operation_patterns, output_census_fragment,
    sponsor_isolation_fragment, successor_recognition_fragment,
};
use crate::program::TapscriptProgram;
use crate::shape::{
    CompactAshShape, CompactAshShapeBounds, SponsorChangePresence, demonstration_shape_set,
};
use crate::stack::{AbstractLimits, AbstractStackState, validate_program};
use crate::tests::{pattern_symbols, reviewed_target};

/// Placeholder link-time symbols, each byte string distinct so a
/// fragment that compared against the wrong one would be visible.
fn symbols() -> CompactAshSymbols {
    pattern_symbols(&reviewed_target())
}

/// A shape with `ash` sources, `sponsors` sponsor inputs, and `change`.
fn shape(ash: u8, sponsors: u8, change: SponsorChangePresence) -> CompactAshShape {
    let bounds = CompactAshShapeBounds::new(NonZeroU8::new(4).expect("nonzero"), 1)
        .expect("four is above the minimum");
    CompactAshShape::new(
        bounds,
        NonZeroU8::new(ash).expect("nonzero"),
        sponsors,
        change,
    )
    .expect("a valid fixture shape")
}

/// The sponsored, change-bearing shape every fragment is non-trivial
/// on.
fn full_shape() -> CompactAshShape {
    shape(3, 1, SponsorChangePresence::Present)
}

/// The causes every domain-gated primitive can abort on regardless of
/// what it is handed.
fn domain_abort() -> BTreeSet<FailureCause> {
    BTreeSet::from([FailureCause::UnsupportedExecutionDomain])
}

/// The single successful state a fragment ending empty reaches.
fn empty_success() -> BTreeSet<AbstractStackState> {
    BTreeSet::from([AbstractStackState::from_main(Vec::new())])
}

/// Every pattern of the full shape.
fn patterns() -> std::collections::BTreeMap<BackendPatternId, BackendPattern> {
    let target = reviewed_target();
    operation_patterns(&target, &symbols(), full_shape()).expect("every pattern schedules")
}

// --- The census, and what admission means ----------------------------

#[test]
fn every_admitted_identity_has_a_complete_record_and_no_identity_lacks_one() {
    // §8.4: an identity exists because a complete record does. Both
    // directions, so neither an unbacked variant nor an unnamed record
    // can appear.
    let patterns = patterns();

    assert_eq!(
        patterns.keys().copied().collect::<Vec<_>>(),
        BackendPatternId::ALL.to_vec(),
    );
    assert_eq!(patterns.len(), 8);

    for (id, pattern) in &patterns {
        assert_eq!(pattern.id(), *id);
        assert!(!pattern.abi().is_empty(), "{id:?} states its assumptions");
        assert!(!pattern.sources().is_empty(), "{id:?} routes some fact");
        assert!(!pattern.evidence().is_empty(), "{id:?} names its evidence");
        assert!(
            pattern.resources().script_bytes() > 0 || pattern.fragment().is_empty(),
            "{id:?} is charged its exact bytes",
        );
        assert_eq!(
            pattern
                .resources()
                .dimensions()
                .keys()
                .copied()
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([
                ResourceDimension::ScriptBytes,
                ResourceDimension::OperationCost,
                ResourceDimension::ValidationBudget,
            ]),
        );
    }
}

#[test]
fn every_pattern_owns_a_distinct_relation() {
    // A pattern census in which two identities claimed the same
    // relation would be two answers to one obligation.
    let owners = patterns()
        .values()
        .map(BackendPattern::owner)
        .collect::<BTreeSet<PatternOwner>>();

    assert_eq!(owners.len(), 8);
}

#[test]
fn no_pattern_leaves_a_state_reached_through_a_failure() {
    // §12.11: no non-aborting failure state may survive into the
    // successful ones, and none may survive at all where every flag is
    // verified. The validator separates them by construction, so this
    // is a checked property of each schedule.
    for (id, pattern) in patterns() {
        assert!(
            pattern.failure().nonaborting().is_empty(),
            "{id:?} leaves an unconsumed flag",
        );
        assert!(
            pattern
                .stack()
                .success()
                .intersection(pattern.failure().nonaborting())
                .next()
                .is_none(),
            "{id:?} rejoins success through a failure",
        );
    }
}

#[test]
fn every_pattern_reaches_exactly_one_successful_shape() {
    // A schedule with two successful shapes would leave a caller
    // unable to schedule the next fragment against it, which is the
    // mis-scheduling the alternative-retaining validator exists to
    // expose.
    for (id, pattern) in patterns() {
        assert_eq!(
            pattern.stack().success().len(),
            1,
            "{id:?} reaches {:?}",
            pattern.stack().success(),
        );
    }
}

// --- The individual schedules, hand-walked ---------------------------

#[test]
fn the_coordinator_leaf_succeeds_at_the_anchor_and_aborts_anywhere_else() {
    // `PushCurrentInputIndex; Push 0; EQUALVERIFY`. The index is the
    // target's own, so a coordinator leaf spent at any other input
    // reaches the inequality abort — which is exactly §10.3's "a
    // coordinator leaf at a later input rejects".
    let pattern = patterns()[&BackendPatternId::CompactAshCoordinatorRoleV1].clone();

    assert_eq!(pattern.stack().success(), &empty_success());
    assert_eq!(
        pattern.failure().aborts(),
        &domain_abort()
            .into_iter()
            .chain([
                FailureCause::IntrospectionContextUnavailable,
                FailureCause::UnequalOperands,
            ])
            .collect(),
    );
    assert_eq!(
        pattern.prerequisites(),
        &BTreeSet::from([
            ElementsCapability::CurrentInputIndexInspection,
            ElementsCapability::ByteStringEquality,
        ]),
    );
}

#[test]
fn the_member_leaf_checks_both_ends_of_its_range_and_consumes_both_flags() {
    // `1 ≤ index < n`, each comparison's Boolean verified immediately.
    // Both bounds are false-verification aborts rather than surviving
    // states, so a member leaf at input 0 and one in the sponsor suffix
    // both end the spend (§10.3, §12.7).
    let pattern = patterns()[&BackendPatternId::CompactAshMemberRoleV1].clone();

    assert_eq!(pattern.stack().success(), &empty_success());
    assert_eq!(
        pattern.failure().aborts(),
        &BTreeSet::from([
            FailureCause::UnsupportedExecutionDomain,
            FailureCause::IntrospectionContextUnavailable,
            FailureCause::MalformedScriptNumber,
            FailureCause::FalseVerification,
        ]),
    );

    // Two verifications, and the comparison whose result each consumes.
    let verifies = pattern
        .fragment()
        .instructions()
        .iter()
        .filter(|instruction| matches!(instruction, TapscriptInstruction::Opcode(OpcodeId::Verify)))
        .count();
    assert_eq!(verifies, 2);
}

#[test]
fn recognition_leaves_exactly_one_amount_operand_and_nothing_else() {
    // §12.1 in one fragment: the linked asset, the linked program, the
    // explicit encoding, and the amount domain — with the amount left
    // for the aggregate rather than introspected a second time.
    let pattern = patterns()[&BackendPatternId::CompactAshObjectRecognitionV1].clone();

    assert_eq!(
        pattern.stack().success(),
        &BTreeSet::from([AbstractStackState::from_main(vec![operand_type(
            AMOUNT_OPERAND_BYTES
        )])]),
    );

    // The slice is what makes the operand's width a program fact. Its
    // out-of-range abort is therefore part of the pattern's failure
    // behaviour and not an accident.
    assert!(
        pattern
            .failure()
            .aborts()
            .contains(&FailureCause::SliceOutOfRange),
    );
    // Both domain bounds and both prefix comparisons can fail.
    assert!(
        pattern
            .failure()
            .aborts()
            .contains(&FailureCause::FalseVerification),
    );
    assert!(
        pattern
            .failure()
            .aborts()
            .contains(&FailureCause::UnequalOperands),
    );

    assert!(
        pattern
            .abi()
            .contains(&AbiAssumption::ExplicitFormEstablishedByPrefixEquality),
        "the one narrowing the abstract walk cannot make is stated",
    );
}

#[test]
fn the_successor_is_recognized_the_same_way_the_sources_are() {
    // §12.2 is §12.1 on the output side, and the schedules agree: same
    // successful shape, same failure behaviour. A successor recognized
    // more weakly than a source would be the asymmetry §12.5's "the
    // only U output" depends on not existing.
    let target = reviewed_target();
    let symbols = symbols();
    let source = ash_input_recognition_fragment(&target, &symbols, 0).expect("fragment");
    let successor = successor_recognition_fragment(&target, &symbols).expect("fragment");
    let limits = AbstractLimits::for_target(&target);
    let initial = AbstractStackState::from_main(Vec::new());

    let source = validate_program(&target, &source, &initial, limits).expect("schedule");
    let successor = validate_program(&target, &successor, &initial, limits).expect("schedule");

    assert_eq!(source.success(), successor.success());
    assert_eq!(source.aborts(), successor.aborts());
    assert!(successor.nonaborting_failure().is_empty());
}

#[test]
fn the_shape_fragment_compares_the_targets_own_counts() {
    // §12.3: caller-proposed counts are witnesses at most. This
    // fragment has no witness at all — the counts are compiled in from
    // the shape and compared against the target's introspected ones.
    let pattern = patterns()[&BackendPatternId::CompactAshShapeV1].clone();

    assert_eq!(pattern.stack().success(), &empty_success());
    assert_eq!(
        pattern.prerequisites(),
        &BTreeSet::from([
            ElementsCapability::InputCountInspection,
            ElementsCapability::OutputCountInspection,
            ElementsCapability::ByteStringEquality,
        ]),
    );

    // The full shape has three sources, one sponsor, one successor, one
    // change role, and one fee role.
    assert_eq!(full_shape().inputs(), 4);
    assert_eq!(full_shape().outputs(), 3);
}

#[test]
fn the_aggregate_consumes_every_flag_and_ends_on_an_exact_comparison() {
    // §12.4: each addition's success flag is verified immediately, so
    // an overflow aborts rather than leaving a false; the closing
    // byte-equality is exact because both operands are canonical
    // eight-byte encodings.
    let pattern = patterns()[&BackendPatternId::CompactAshExplicitSumV1].clone();

    assert_eq!(
        pattern.stack().initial(),
        &AbstractStackState::from_main(vec![operand_type(AMOUNT_OPERAND_BYTES); 4]),
        "the successor's amount, then one per source",
    );
    assert_eq!(pattern.stack().success(), &empty_success());
    assert!(pattern.failure().nonaborting().is_empty());
    assert_eq!(
        pattern.failure().aborts(),
        &BTreeSet::from([
            FailureCause::UnsupportedExecutionDomain,
            FailureCause::FalseVerification,
            FailureCause::UnequalOperands,
        ]),
    );

    // Two additions for three sources, each with its own verification.
    let additions = pattern
        .fragment()
        .instructions()
        .iter()
        .filter(|instruction| matches!(instruction, TapscriptInstruction::Opcode(OpcodeId::Add64)))
        .count();
    assert_eq!(additions, 2);
    assert_eq!(pattern.fragment().len(), 5);
}

#[test]
fn an_unverified_addition_is_visible_as_a_surviving_failure_state() {
    // The negative control for the aggregate. Without the verification
    // the overflow path survives with both operands retained and a
    // false above them — three items where the checked schedule leaves
    // one, which is the depth error §12.11 exists to prevent.
    let target = reviewed_target();
    let program = TapscriptProgram::new(vec![TapscriptInstruction::Opcode(OpcodeId::Add64)])
        .expect("one instruction");
    let initial = AbstractStackState::from_main(vec![operand_type(AMOUNT_OPERAND_BYTES); 2]);
    let result = validate_program(
        &target,
        &program,
        &initial,
        AbstractLimits::for_target(&target),
    )
    .expect("schedule");

    let failure_depths = result
        .nonaborting_failure()
        .iter()
        .map(|state| state.main().len())
        .collect::<Vec<_>>();
    assert_eq!(failure_depths, vec![3]);
    assert!(!result.nonaborting_failure().is_empty());
}

#[test]
fn the_output_census_recognizes_the_fee_role_by_form_and_never_by_amount() {
    // §10.6 and Wave 5: the target replaces a program that is not a
    // witness program by a digest of it under a negative version
    // marker, so the fee role reads on the stack as exactly that pair.
    // The fragment introspects no output *value* at all, which is what
    // makes "not an ordinary output whose amount is zero" a property of
    // the emitted instructions rather than a comment.
    let pattern = patterns()[&BackendPatternId::CompactAshCanonicalPartitionV1].clone();

    assert_eq!(pattern.stack().success(), &empty_success());
    assert!(
        !pattern
            .prerequisites()
            .contains(&ElementsCapability::OutputValueInspection),
        "no output amount is read to recognize a role",
    );
    assert_eq!(
        pattern.prerequisites(),
        &BTreeSet::from([
            ElementsCapability::OutputAssetInspection,
            ElementsCapability::OutputProgramInspection,
            ElementsCapability::ByteStringEquality,
        ]),
    );

    // The negative version marker is pushed as a literal, so a witness
    // program at the fee position fails the comparison.
    let target = reviewed_target();
    let marker = crate::instruction::StackItem::script_number(&target, -1).expect("marker");
    assert!(
        pattern
            .fragment()
            .instructions()
            .contains(&TapscriptInstruction::Push(marker)),
    );
}

#[test]
fn a_sponsorless_shape_has_no_fee_role_and_an_empty_sponsor_region() {
    // §10.6 and the reviewed zero-fee representation: a sponsorless
    // compact-ASH transaction pays nothing and therefore carries no fee
    // output, because the target refuses a zero-valued one. Both
    // fragments are empty for that shape, and their obligation is
    // discharged by the exact counts the shape fragment pins rather
    // than by a test that could be omitted.
    let target = reviewed_target();
    let symbols = symbols();
    let sponsorless = shape(2, 0, SponsorChangePresence::Absent);

    assert_eq!(sponsorless.outputs(), 1);
    assert!(
        output_census_fragment(&target, &symbols, sponsorless)
            .expect("fragment")
            .is_empty(),
    );
    assert!(
        sponsor_isolation_fragment(&target, &symbols, sponsorless)
            .expect("fragment")
            .is_empty(),
    );

    // And on a sponsored shape both are non-trivial, so the emptiness
    // above is a property of the shape rather than of the builder.
    assert!(
        !output_census_fragment(&target, &symbols, full_shape())
            .expect("fragment")
            .is_empty(),
    );
    assert!(
        !sponsor_isolation_fragment(&target, &symbols, full_shape())
            .expect("fragment")
            .is_empty(),
    );
}

#[test]
fn sponsor_isolation_reads_the_asset_and_never_the_value() {
    // §12.9 and §1.6. The reviewed profile forces the sponsor asset
    // explicit and leaves the sponsor value free to stay a commitment,
    // so the fragment checks the asset and does not introspect the
    // value at all. There is no amount comparison to remove because
    // there is no amount on the stack.
    let pattern = patterns()[&BackendPatternId::CompactAshSponsorIsolationV1].clone();

    assert_eq!(pattern.stack().success(), &empty_success());
    assert!(
        !pattern
            .prerequisites()
            .contains(&ElementsCapability::InputValueInspection),
        "no sponsor amount is read",
    );
    assert_eq!(
        pattern.prerequisites(),
        &BTreeSet::from([
            ElementsCapability::InputAssetInspection,
            ElementsCapability::ByteStringEquality,
        ]),
    );
    assert!(
        pattern
            .abi()
            .contains(&AbiAssumption::ExternalWholeTransactionConservation),
        "whole-transaction conservation stays an external target claim",
    );
}

#[test]
fn no_emitted_program_carries_any_authorization_primitive() {
    // §12.8, checked on the typed instructions rather than on a
    // capability mapping: the claim is about what is emitted. Every
    // program of every demonstration shape is audited, coordinator and
    // member alike, and no signature or cadence primitive appears in
    // any of them.
    let target = reviewed_target();
    let symbols = symbols();

    for shape in demonstration_shape_set().shapes() {
        for program in [
            coordinator_program(&target, &symbols, shape).expect("coordinator"),
            member_program(&target, &symbols, shape).expect("member"),
        ] {
            assert!(!carries_authorization(&program), "shape {shape:?}");

            let result = validate_program(
                &target,
                &program,
                &AbstractStackState::from_main(Vec::new()),
                AbstractLimits::for_target(&target),
            )
            .expect("the whole program schedules");

            // §12.11: one canonical true item on success, and nothing
            // else — no residue, no unconsumed flag.
            assert_eq!(
                result.success(),
                &BTreeSet::from([AbstractStackState::from_main(vec![StackValueType::Bytes {
                    minimum: 1,
                    maximum: 1,
                }])]),
                "shape {shape:?}",
            );
            assert!(result.nonaborting_failure().is_empty(), "shape {shape:?}");
            assert!(
                result
                    .success()
                    .iter()
                    .all(|state| state.alternate().is_empty()),
                "no alternate-stack residue",
            );

            // No signature-shaped cause can arise, because no
            // signature-shaped instruction was emitted.
            for cause in [FailureCause::InvalidSignature, FailureCause::EmptyPublicKey] {
                assert!(!result.aborts().contains(&cause), "shape {shape:?}");
            }
        }
    }
}

// --- Negative vectors -------------------------------------------------

#[test]
fn the_negative_vector_census_is_exactly_this_and_names_what_the_walk_cannot_decide() {
    // The negative half of every pattern's vectors, as an exact
    // classification rather than a blanket "the mutant differs".
    //
    // The result is the interesting one. Dropping a verification is
    // decided by the walk every time it applies: the fragment loses its
    // successful shape or its abort. Corrupting a literal's *value* is
    // decided nowhere, because every literal in these fragments is
    // compared with a target-supplied value the walk holds no bytes
    // for — so those claims rest on the link step resolving the symbol,
    // which is what `SymbolsResolvedAtLink` records. Widening a literal
    // is decided wherever the mutated literal feeds an operand whose
    // width the contract fixes, and the mutant then stops being a
    // schedulable program at all.
    //
    // Writing the census out means a pattern that quietly stopped
    // depending on its own verification fails here rather than passing.
    let target = reviewed_target();
    let mut census = BTreeMap::new();
    for (id, pattern) in &patterns() {
        for mutation in PatternMutation::ALL {
            census.insert(
                (*id, *mutation),
                pattern.negative_vector(&target, *mutation),
            );
        }
    }

    for (id, mutation) in census.keys() {
        let outcome = census[&(*id, *mutation)];

        match mutation {
            PatternMutation::DropFinalVerification => assert_eq!(
                outcome,
                MutationOutcome::ContractChanged,
                "{id:?}: dropping its verification must cost it its contract",
            ),
            PatternMutation::CorruptFinalLiteral => assert!(
                matches!(
                    outcome,
                    MutationOutcome::AbstractlyIndistinguishable | MutationOutcome::Inapplicable,
                ),
                "{id:?}: {outcome:?} — a value corruption decidable here would be a \
                 surprise worth recording rather than a pass",
            ),
            PatternMutation::WidenFinalLiteral => assert!(matches!(
                outcome,
                MutationOutcome::Refused
                    | MutationOutcome::ContractChanged
                    | MutationOutcome::AbstractlyIndistinguishable
                    | MutationOutcome::Inapplicable,
            ),),
        }
    }

    // Exactly the two patterns whose last literal feeds an arithmetic
    // comparison refuse a widened one outright: those operands have a
    // width the contract fixes, and a nine-byte item is not a program
    // the validator will describe. Every other pattern's last literal
    // is compared by byte equality, which admits any width, so widening
    // it is just another wrong literal — including the permissionless
    // pattern, whose fragment is the whole coordinator program and
    // whose last literal is the fee role's digest.
    let refused = census
        .iter()
        .filter(|(_, outcome)| **outcome == MutationOutcome::Refused)
        .map(|((id, _), _)| *id)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        refused,
        BTreeSet::from([
            BackendPatternId::CompactAshMemberRoleV1,
            BackendPatternId::CompactAshObjectRecognitionV1,
        ]),
    );

    // No pattern is left with no negative vector at all.
    for id in BackendPatternId::ALL {
        assert!(
            PatternMutation::ALL
                .iter()
                .any(|mutation| census[&(*id, *mutation)] != MutationOutcome::Inapplicable),
            "{id:?} has no negative vector",
        );
    }
}

#[test]
fn a_pattern_with_no_applicable_mutation_is_reported_rather_than_passed() {
    // The one place a missing mutant is honest: a fragment with no
    // pushed literal has none to corrupt. `mutated` returns `None`
    // rather than an unchanged program, so a caller cannot mistake
    // "no such vector" for "the vector passed".
    let target = reviewed_target();
    let program = TapscriptProgram::new(vec![TapscriptInstruction::Opcode(
        OpcodeId::PushCurrentInputIndex,
    )])
    .expect("one instruction");
    let pattern = crate::pattern::build_pattern(
        &target,
        BackendPatternId::CompactAshCoordinatorRoleV1,
        PatternOwner::CoordinatorRole,
        program,
        AbstractStackState::from_main(Vec::new()),
        BTreeSet::new(),
        BTreeSet::new(),
        BTreeSet::new(),
        BTreeSet::new(),
    )
    .expect("it schedules");

    assert!(
        pattern
            .mutated(&target, PatternMutation::CorruptFinalLiteral)
            .is_none(),
    );
    assert!(
        pattern
            .mutated(&target, PatternMutation::DropFinalVerification)
            .is_none(),
    );
}

// --- Resources --------------------------------------------------------

#[test]
fn a_larger_batch_costs_strictly_more_bytes_and_the_cost_is_exact() {
    // §9.2 asks what shape specialization costs. The resource
    // projection charges the exact encoded bytes including every push,
    // so the answer is measured rather than modelled: one more source
    // is one more recognition fragment plus one more checked addition.
    let target = reviewed_target();
    let symbols = symbols();

    let mut previous = 0_u64;
    for ash in 2..=4_u8 {
        let program = coordinator_program(
            &target,
            &symbols,
            shape(ash, 0, SponsorChangePresence::Absent),
        )
        .expect("coordinator");
        let bytes = program.encoded_length(&target);

        assert!(bytes > previous, "{ash} sources cost {bytes}");
        previous = bytes;
    }
}
