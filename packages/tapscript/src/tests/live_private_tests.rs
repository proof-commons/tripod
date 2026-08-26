//! Oracles for the private-committed plan (Guide-13 §6.3, §6.4, §6.6).
//!
//! # What is checked against what
//!
//! Nothing here asks the private plan to confirm its own reasoning. The
//! opacity claims are read off the emitted instruction lists, and every
//! one of them is shown to be non-vacuous by running the same predicate
//! over the explicit conservation — which opens amounts, because §6.2 is
//! exactly that it does. A property only the private programs satisfy is
//! a property; a property everything satisfies is a sentence.
//!
//! The §6.3 census is checked in both directions. Every condition names
//! something that carries it, every fragment it names is one a private
//! program actually emits, and the one line no fragment may carry names
//! none — the last being §10.6's own rule rather than a convention this
//! module chose.
//!
//! The prefix arithmetic is checked against the *registry*, over every
//! prefix-discriminated class the reviewed contract states, so a class
//! that gained or lost a form would move the test rather than pass it.
//! Its refusal is checked over a byte set instead, because every class
//! the contract states happens to discriminate and a refusal reachable
//! only through the registry would be a refusal nobody has watched work.
//!
//! # No confidential fixture is needed here, and none is invented
//!
//! Not one test below fixes a commitment, a blinder, or an opening. It
//! does not need to: a value's *form* is a prefix the reviewed registry
//! declares, and the private plan reads nothing else from the field. The
//! commitment arithmetic that would validate a fixture expectation is the
//! conformance package's first-party oracle (§9.3), it establishes no
//! target acceptance, and this crate takes no dependency that would put
//! curve arithmetic behind a tapscript oracle.

use std::collections::BTreeSet;
use std::num::NonZeroU8;

use compiler::live_transfer_plan::LiveTransferRepresentationPlan;
use compiler::target::ExternalEvidenceRole;
use target_elements::{EncodingClass, OpcodeId, ReviewedElementsTapscriptDefinition};

use super::{live_transfer_plan, live_transfer_symbols, reviewed_target};
use crate::authorization::owner_key_encoding_closure;
use crate::instruction::{StackItem, TapscriptInstruction};
use crate::live_bundle::emit_candidate_live_bundle;
use crate::live_constructor::{
    LiveProgramRole, OwnerKey, StaticLiveReceiptConstructor, derive_live_receipt_constructor,
    static_transfer_leaf_set,
};
use crate::live_pattern::{
    LiveDisclosure, LiveFragmentId, LiveTransferPatternId, LiveTransferSymbols, emitted_fragments,
    final_stack_defects, has_member_position, live_coordinator_program, live_member_program,
    live_program_precondition, live_transfer_patterns, local_recognition_fragment, patterns_for,
};
use crate::live_plan::{explicit_conservation_fragment, opens_an_amount, reads_a_value_field};
use crate::live_private::{
    ComparisonSource, PrivateAmountProhibition, PrivateConditionCarrier, PrivatePlanNonClaim,
    PrivateSoundnessCondition, ProhibitionDisposition, RepresentationComparisonAxis, ValueFieldUse,
    absorber_position, crossing_destination_form_fragment, discriminating_mask,
    opens_no_value_payload, prefix_mask, private_destination_form_fragment,
    private_soundness_establishments, prohibition_dispositions, representation_comparison_axes,
    value_field_uses,
};
use crate::live_shape::{LiveTransferShape, demonstration_live_shape_set};
use crate::program::TapscriptProgram;
use crate::shape::SponsorChangePresence::{self, Absent, Present};

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

/// The reference constructor for one representation plan.
///
/// # Panics
///
/// If the reference parts stop deriving a constructor, which Wave 4's own
/// oracles would have caught first.
fn constructor(representation: LiveTransferRepresentationPlan) -> StaticLiveReceiptConstructor {
    let target = reviewed_target();
    let shapes = demonstration_live_shape_set();
    let leaves = static_transfer_leaf_set(representation, &shapes);
    let closure = owner_key_encoding_closure(target.definition().authorization());
    let owner = OwnerKey::new(&closure, closure.approved(), vec![0x11; 32])
        .expect("the fixture is the approved encoding at its exact width");

    derive_live_receipt_constructor(
        &target,
        &live_transfer_plan(),
        representation,
        owner,
        shapes,
        leaves,
    )
    .expect("the reference subject derives")
}

/// Every program the private plan emits, over every admitted shape.
///
/// Coordinators and members alike. A property held by the coordinators
/// alone would be a property with a hole exactly where a spend of a
/// nonzero receipt position runs.
///
/// # Panics
///
/// If a program the constructor admits stops emitting, which is a defect
/// in the emitter rather than a finding about the private plan.
fn private_programs() -> Vec<TapscriptProgram> {
    let target = reviewed_target();
    let symbols = symbols();
    let subject = constructor(LiveTransferRepresentationPlan::PrivateCommitted);
    let mut programs = Vec::new();

    for shape in demonstration_live_shape_set().shapes() {
        programs.push(
            live_coordinator_program(&target, &symbols, &subject, shape)
                .expect("the private coordinator emits"),
        );
        if has_member_position(shape) {
            programs.push(
                live_member_program(&target, &symbols, &subject, shape.receipt_inputs())
                    .expect("the private member emits"),
            );
        }
    }

    programs
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

/// Whether one program schedules a reviewed primitive at all.
fn schedules(program: &TapscriptProgram, id: OpcodeId) -> bool {
    program
        .instructions()
        .contains(&TapscriptInstruction::Opcode(id))
}

/// Every prefix the reviewed registry declares for one class.
fn declared_prefixes(
    target: &ReviewedElementsTapscriptDefinition,
    class: EncodingClass,
) -> BTreeSet<u8> {
    target
        .definition()
        .encodings()
        .get(&class)
        .map(|spec| spec.prefixes().clone())
        .unwrap_or_default()
}

// --- Prefix discrimination --------------------------------------------

#[test]
fn every_prefix_discriminated_class_the_contract_states_discriminates() {
    // Read off the registry rather than listed here, so a class that
    // gained or lost a form moves this test instead of passing it. The
    // masked comparison this crate emits is only a form test while every
    // class it is used on admits exactly its declared prefixes.
    let target = reviewed_target();
    let mut checked = 0;

    for class in EncodingClass::ALL {
        let declared = declared_prefixes(&target, *class);
        if declared.is_empty() {
            continue;
        }
        checked += 1;

        let mask = prefix_mask(&target, *class)
            .unwrap_or_else(|defect| panic!("{class:?} does not discriminate: {defect:?}"));
        assert_eq!(
            (0..=u8::MAX)
                .filter(|byte| mask.admits(*byte))
                .collect::<BTreeSet<_>>(),
            declared,
            "the mask for {class:?} does not admit exactly its declared prefixes",
        );
    }

    assert!(checked > 0, "the registry states no prefixed class at all");
}

#[test]
fn a_prefix_set_that_no_mask_admits_exactly_is_refused_rather_than_widened() {
    // The refusal, watched working. `{0x01, 0x02, 0x03}` varies in two
    // bits, so any mask covering them admits `0x00` as well — and a form
    // test that accepts an undeclared form is not a form test. Fixing
    // this by widening is the failure the refusal exists to prevent.
    assert_eq!(
        discriminating_mask(&BTreeSet::from([0x01, 0x02, 0x03])),
        None
    );
    assert_eq!(discriminating_mask(&BTreeSet::new()), None);

    // And the neighbouring set that *does* tile is accepted, so the
    // refusal is discriminating rather than merely cautious.
    let tiled = discriminating_mask(&BTreeSet::from([0x00, 0x01, 0x02, 0x03]))
        .expect("four bytes over two varying bits tile exactly");
    assert_eq!(tiled.admitted_count(), 4);
    assert!(!tiled.is_exact_byte());

    // A single prefix fixes every bit, which is what keeps the explicit
    // plan's emitted bytes the equality they always were.
    let single = discriminating_mask(&BTreeSet::from([0x01])).expect("one byte tiles exactly");
    assert!(single.is_exact_byte());
    assert_eq!(single.value(), 0x01);
    assert_eq!(single.admitted_count(), 1);
}

// --- §6.3: what makes the private relation sound ----------------------

#[test]
fn every_soundness_condition_names_something_that_carries_it() {
    let establishments = private_soundness_establishments();

    assert_eq!(establishments.len(), PrivateSoundnessCondition::ALL.len());
    for condition in PrivateSoundnessCondition::ALL {
        let establishment = &establishments[condition];
        assert_eq!(establishment.condition(), *condition);
        assert!(
            establishment.fragments().count() + establishment.external_evidence().count() > 0,
            "{condition:?} is carried by nothing at all",
        );
    }
}

#[test]
fn the_target_conservation_line_names_no_fragment_and_only_external_evidence() {
    // §10.6's rule, executed. No private-conservation pattern is minted
    // for target consensus behaviour, and §6.3 refuses a local program
    // that claims CT conservation because consensus eventually accepts —
    // so a census listing a fragment beside this line would be recording
    // exactly the artifact both rules exist to refuse.
    let establishments = private_soundness_establishments();
    let equation = &establishments[&PrivateSoundnessCondition::TargetConfidentialConservation];

    assert_eq!(equation.fragments().count(), 0);
    assert_eq!(
        equation.carrier(),
        PrivateConditionCarrier::ExternalTargetEvidence,
    );
    assert_eq!(
        equation.external_evidence().collect::<BTreeSet<_>>(),
        BTreeSet::from([ExternalEvidenceRole::ConfidentialValueConservation]),
    );

    // And it is the only line carried by nothing this crate emits, which
    // is what keeps the closure a closure rather than a gesture at one.
    for condition in PrivateSoundnessCondition::ALL {
        if *condition == PrivateSoundnessCondition::TargetConfidentialConservation {
            continue;
        }
        assert!(
            establishments[condition].fragments().count() > 0,
            "{condition:?} emits nothing, so the private plan closes nothing around the equation",
        );
    }
}

#[test]
fn every_fragment_the_soundness_census_names_is_one_a_private_program_emits() {
    // The census may not claim credit for bytes the private plan does not
    // carry. Checked against the emitter's own account of both roles,
    // which is what `validate_coordinator_placements` checks the §10.3
    // slots against.
    let private = LiveTransferRepresentationPlan::PrivateCommitted;
    let emitted = [LiveProgramRole::Coordinator, LiveProgramRole::Member]
        .into_iter()
        .flat_map(|role| emitted_fragments(role, private))
        .collect::<BTreeSet<_>>();

    for establishment in private_soundness_establishments().values() {
        for fragment in establishment.fragments() {
            assert!(
                emitted.contains(&fragment),
                "{:?} names {fragment:?}, which no private program emits",
                establishment.condition(),
            );
        }
    }

    // And the explicit plan's arithmetic is not among them, which is the
    // claim the whole census rests on.
    assert!(!emitted.contains(&LiveFragmentId::ExplicitConservation));
    assert!(emitted.contains(&LiveFragmentId::PrivateDestinationForm));
}

#[test]
fn the_private_plan_emits_a_one_to_one_a_split_and_a_merge() {
    // §1.5's prohibited implications run between these three, in both
    // directions: a one-to-one private transfer working implies nothing
    // about a split, and a split implies nothing about a merge. So each
    // is emitted and held to §10.9 by name rather than left to a loop
    // over the shape set, where a class silently absent from the bounds
    // would take its own oracle with it.
    let target = reviewed_target();
    let symbols = symbols();
    let subject = constructor(LiveTransferRepresentationPlan::PrivateCommitted);

    for (receipt_inputs, receipt_outputs) in [(1, 1), (1, 3), (3, 1), (3, 3)] {
        let cardinality = shape(receipt_inputs, receipt_outputs, 0, Absent);
        let coordinator = live_coordinator_program(&target, &symbols, &subject, cardinality)
            .expect("the private coordinator emits");

        assert_eq!(
            final_stack_defects(&target, &coordinator, &live_program_precondition(&target))
                .expect("the coordinator schedules"),
            Vec::new(),
            "the private coordinator of ({receipt_inputs}, {receipt_outputs}) fails §10.9",
        );
        assert!(!opens_an_amount(&coordinator));
        assert!(opens_no_value_payload(&coordinator));

        // And the form check covers every destination of the shape,
        // which is what makes the split and the redistribution closed
        // rather than closed at the first output.
        assert_eq!(
            value_field_uses(
                &private_destination_form_fragment(&target, cardinality)
                    .expect("the form fragment assembles")
            )
            .len(),
            usize::from(receipt_outputs),
        );
    }
}

// --- §6.4: amount opacity over the emitted bytes ----------------------

#[test]
fn no_private_program_opens_an_amount_or_leaves_a_value_payload() {
    // §6.4's first prohibition, over every program the private plan emits
    // for every shape it admits — coordinators and members alike.
    for program in private_programs() {
        assert!(!opens_an_amount(&program));
        assert!(
            opens_no_value_payload(&program),
            "a private program leaves a value payload where a primitive could take it",
        );
    }

    // Neither claim is vacuous. The explicit conservation reads value
    // fields, opens their payloads, and fails the whitelist — because
    // §6.2 is exactly that it does.
    let conservation = explicit_conservation_fragment(&reviewed_target(), shape(2, 2, 0, Absent))
        .expect("the conservation fragment assembles");
    assert!(reads_a_value_field(&conservation));
    assert!(opens_an_amount(&conservation));
    assert!(!opens_no_value_payload(&conservation));
}

#[test]
fn the_payload_whitelist_tells_a_form_check_from_an_opening() {
    // The oracle itself, over the two fragments whose behaviour is known.
    // A whitelist that reported everything as checked would satisfy the
    // test above and mean nothing.
    let target = reviewed_target();

    let recognition = local_recognition_fragment(
        &target,
        &symbols(),
        LiveTransferRepresentationPlan::PrivateCommitted,
    )
    .expect("the recognition fragment assembles");
    assert_eq!(
        value_field_uses(&recognition),
        vec![ValueFieldUse::FormCheckedThenDropped],
    );

    let subject = shape(2, 3, 0, Absent);
    let form =
        private_destination_form_fragment(&target, subject).expect("the form fragment assembles");
    assert_eq!(
        value_field_uses(&form),
        vec![ValueFieldUse::FormCheckedThenDropped; usize::from(subject.receipt_outputs())],
    );

    // Every read of the explicit conservation is an opening, and there is
    // one per position on both sides.
    let conservation =
        explicit_conservation_fragment(&target, subject).expect("the conservation assembles");
    assert_eq!(
        value_field_uses(&conservation),
        vec![
            ValueFieldUse::PayloadOpened;
            usize::from(subject.receipt_inputs()) + usize::from(subject.receipt_outputs())
        ],
    );

    // And a program that reads no value field at all reports nothing
    // rather than reporting itself clean by default.
    let nothing = TapscriptProgram::new(Vec::new()).expect("the empty program assembles");
    assert_eq!(value_field_uses(&nothing), Vec::new());
}

#[test]
fn no_private_program_schedules_arithmetic_or_publishes_a_subtotal() {
    // §6.4's subtotal and zero-comparison prohibitions share one
    // mechanism: no value payload reaches any primitive, so there is no
    // operand a subtotal or a comparison with zero could be built from.
    // The stronger statement is checkable too — the private plan
    // schedules no arithmetic primitive at all.
    for program in private_programs() {
        for arithmetic in [
            OpcodeId::Add64,
            OpcodeId::Sub64,
            OpcodeId::Mul64,
            OpcodeId::Div64,
            OpcodeId::Neg64,
            OpcodeId::Substring,
        ] {
            assert!(
                !schedules(&program, arithmetic),
                "a private program schedules {arithmetic:?}",
            );
        }
    }
}

#[test]
fn the_private_plan_publishes_no_amount_and_no_amount_domain() {
    // §6.4's diagnostics prohibition, over the surface this crate
    // publishes. The explicit plan holds every receipt amount to the
    // semantic domain and its disclosure census says so; the private plan
    // reads no amount, so it holds none to a bound and claims none — a
    // disclosure census naming the domain here would be publishing a
    // check nobody emitted.
    let target = reviewed_target();
    let subject = shape(2, 2, 0, Absent);
    let private = live_transfer_patterns(
        &target,
        &symbols(),
        &constructor(LiveTransferRepresentationPlan::PrivateCommitted),
        subject,
    )
    .expect("the private census builds");

    for pattern in private.values() {
        assert!(
            !pattern
                .disclosure()
                .contains(&LiveDisclosure::SemanticAmountDomain),
            "{:?} claims to publish the semantic amount domain",
            pattern.id(),
        );
    }

    // Not vacuous: the explicit plan's conservation does publish it, and
    // the domain literal it pushes appears in no private program.
    let explicit = live_transfer_patterns(
        &target,
        &symbols(),
        &constructor(LiveTransferRepresentationPlan::Explicit),
        subject,
    )
    .expect("the explicit census builds");
    let domain_literals =
        pushed(explicit[&LiveTransferPatternId::LiveExplicitConservationV1].fragment())
            .into_iter()
            .filter(|item| item.bytes().len() == 8)
            .collect::<BTreeSet<_>>();
    assert!(!domain_literals.is_empty());

    for program in private_programs() {
        for literal in pushed(&program) {
            assert!(
                !domain_literals.contains(&literal),
                "a private program pushes an amount-domain literal",
            );
        }
    }
}

#[test]
fn the_destination_form_check_is_a_function_of_the_destination_count_alone() {
    // §6.4's ordering prohibition. The loop is indexed by position, and
    // the fragment depends on nothing else — so two shapes agreeing on
    // their destination count emit the same bytes, and no private amount
    // or blinding value could order anything, because none reaches the
    // emitter at all.
    let target = reviewed_target();

    assert_eq!(
        private_destination_form_fragment(&target, shape(1, 3, 0, Absent))
            .expect("the form fragment assembles"),
        private_destination_form_fragment(&target, shape(3, 3, 1, Present))
            .expect("the form fragment assembles"),
    );
    // And the count does move it, so the equality above is not the
    // fragment being constant.
    assert_ne!(
        private_destination_form_fragment(&target, shape(1, 2, 0, Absent))
            .expect("the form fragment assembles"),
        private_destination_form_fragment(&target, shape(1, 3, 0, Absent))
            .expect("the form fragment assembles"),
    );
}

#[test]
fn the_private_plan_mints_no_conservation_pattern_and_no_equality_witness() {
    // §6.4's last prohibition and §10.6's rule are the same statement
    // read twice: a public equality witness duplicating target CT
    // conservation would need a pattern to carry it, and the census mints
    // none for any shape the plan admits.
    for subject in demonstration_live_shape_set().shapes() {
        let selected = patterns_for(subject, LiveTransferRepresentationPlan::PrivateCommitted);
        assert!(!selected.contains(&LiveTransferPatternId::LiveExplicitConservationV1));
        assert!(selected.contains(&LiveTransferPatternId::LivePrivateDestinationFormV1));
    }
}

#[test]
fn every_prohibition_has_a_disposition_and_the_named_ones_are_named() {
    let dispositions = prohibition_dispositions();

    assert_eq!(dispositions.len(), PrivateAmountProhibition::ALL.len());
    // Exactly one prohibition is owed by a surface this crate does not
    // emit, and it is the canonical-report one. A census that had
    // quietly moved a second prohibition into that bucket would be this
    // wave checking less than it claims.
    assert_eq!(
        dispositions
            .iter()
            .filter(|(_, how)| **how == ProhibitionDisposition::NamedNonClaim)
            .map(|(prohibition, _)| *prohibition)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([PrivateAmountProhibition::FixtureOpeningsInCanonicalReports]),
    );
}

// --- §6.6: the comparable projections ---------------------------------

#[test]
fn every_comparison_axis_has_a_source_and_the_amount_axes_come_from_fixtures() {
    let axes = representation_comparison_axes();

    assert_eq!(axes.len(), RepresentationComparisonAxis::ALL.len());
    for axis in RepresentationComparisonAxis::ALL {
        let projection = axes[axis];
        assert_eq!(projection.axis(), *axis);
        if axis.carries_semantic_amounts() {
            // The whole of §6.6's discipline. An axis carrying semantic
            // amounts that a private fixture could only state by opening
            // a commitment would be an axis the comparison cannot use —
            // and a minimality argument that opened them would have
            // disproved itself in the making.
            assert_eq!(
                projection.source(),
                ComparisonSource::SemanticFixture,
                "{axis:?} carries amounts and is read from the wrong place",
            );
        }
    }

    // Two axes carry amounts and no more, so the check above is not
    // quantifying over an empty set.
    assert_eq!(
        RepresentationComparisonAxis::ALL
            .iter()
            .filter(|axis| axis.carries_semantic_amounts())
            .count(),
        2,
    );
}

// --- §1.5, §1.12, §9.3: the production non-claims ---------------------

#[test]
fn the_non_claims_that_can_be_witnessed_are_witnessed() {
    // A non-claim is a statement, but three of these leave a trace in the
    // artifacts, and a trace that disagreed with the statement would make
    // the census decorative.
    let target = reviewed_target();
    let private = emit_candidate_live_bundle(
        &target,
        &live_transfer_plan(),
        &constructor(LiveTransferRepresentationPlan::PrivateCommitted),
        symbols(),
    )
    .expect("the private plan emits a candidate bundle");

    // ProductionReadiness: the artifact is a prototype and the lifecycle
    // is structurally incomplete.
    assert_eq!(
        private.status(),
        crate::bundle::BackendArtifactStatus::Prototype,
    );
    // The outstanding count is a `NonZeroUsize`, so a bundle whose
    // lifecycle was complete has no representation at all.
    assert!(private.lifecycle().outstanding().get() > 0);

    // TargetAcceptance: every emitted pattern still names the target
    // evidence its correctness rests on, none of which this wave
    // discharges.
    assert!(!private.target_evidence().is_empty());

    // ConfidentialProofValidity: no program here builds or checks a
    // commitment. The curve primitives the target offers are scheduled by
    // nothing the private plan emits.
    for program in private_programs() {
        assert!(!schedules(&program, OpcodeId::EcMulScalarVerify));
        assert!(!schedules(&program, OpcodeId::TweakVerify));
    }

    // SharedCoordinatorProgram: §11.3's disjointness, as bytes. The two
    // representations' coordinators differ for every admitted shape.
    let symbols = symbols();
    let (explicit, committed) = (
        constructor(LiveTransferRepresentationPlan::Explicit),
        constructor(LiveTransferRepresentationPlan::PrivateCommitted),
    );
    for subject in demonstration_live_shape_set().shapes() {
        assert_ne!(
            live_coordinator_program(&target, &symbols, &explicit, subject)
                .expect("the explicit coordinator emits"),
            live_coordinator_program(&target, &symbols, &committed, subject)
                .expect("the private coordinator emits"),
        );
    }

    // And the census names every non-claim exactly once.
    assert_eq!(
        PrivatePlanNonClaim::ALL
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len(),
        PrivatePlanNonClaim::ALL.len(),
    );
}

// --- §6.5: the exit crossing's positional value-form obligation -------

#[test]
fn the_absorber_is_the_last_destination_of_every_shape_that_has_one() {
    // A declaration is only a declaration if it names a position for
    // every shape it governs. Recomputed over a spread of receipt-output
    // counts rather than asserted for one.
    let target = reviewed_target();

    for receipt_outputs in 1_u8..=3 {
        let subject = shape(1, receipt_outputs, 0, Absent);
        let (first, end) = subject.destination_range();

        assert_eq!(
            absorber_position(subject),
            Some(end - 1),
            "the declared absorber is the last destination",
        );
        let position = absorber_position(subject).expect("a destination range with a last member");
        assert!(
            position >= first && position < end,
            "the absorber is INSIDE the destination range, which is what leaves the §10.4 \
             closure argument untouched",
        );
        // Emitting is part of the claim: a position that named itself
        // but could not be built would be a comment.
        crossing_destination_form_fragment(&target, subject)
            .expect("the crossing fragment builds for every admitted destination count");
    }
}

#[test]
fn the_crossing_fragment_requires_the_explicit_form_everywhere_but_the_absorber() {
    // The property is read off the EMITTED BYTES rather than off the
    // source, because the bytes are what a node executes. Each
    // destination position contributes one introspection, one form
    // comparison and one drop, so the two fragments differ in exactly
    // the comparison bytes and in nothing else.
    let target = reviewed_target();
    let subject = shape(1, 3, 0, Absent);
    let absorber = absorber_position(subject).expect("a last destination");

    let crossing =
        crossing_destination_form_fragment(&target, subject).expect("the crossing fragment builds");
    let private =
        private_destination_form_fragment(&target, subject).expect("the private fragment builds");

    // Same length: same positions, same reads, same drops. Only the
    // form each position is held to differs.
    assert_eq!(
        crossing.instructions().len(),
        private.instructions().len(),
        "the crossing fragment adds no position and drops none; it only changes which form \
         each position is held to",
    );
    assert_ne!(
        crossing.instructions(),
        private.instructions(),
        "a crossing fragment identical to the private one would hold the explicit \
         destinations to the confidential form",
    );

    // The absorber's own position, emitted alone, IS the private
    // fragment's treatment of a one-destination shape — which is the
    // statement that the absorber is an ordinary blinded destination
    // sitting at a declared index, and not a new kind of output.
    let sole = shape(1, 1, 0, Absent);
    assert_eq!(
        crossing_destination_form_fragment(&target, sole)
            .expect("builds")
            .instructions(),
        private_destination_form_fragment(&target, sole)
            .expect("builds")
            .instructions(),
        "a one-destination exit crossing is all absorber, so it is the private fragment",
    );
    assert_eq!(absorber_position(sole), Some(0));
    assert_eq!(
        absorber, 2,
        "the three-destination shape absorbs at index two"
    );
}

#[test]
fn the_crossing_fragment_opens_no_value_payload() {
    // §6.4 is not weakened because a destination became readable. The
    // explicit destinations' amounts are readable in principle and this
    // fragment still does not read them: every introspection is form-
    // checked and dropped, so there is no payload left for any primitive
    // to take. That is what keeps this a FORM obligation and stops it
    // being mistaken for a conservation one.
    let target = reviewed_target();

    for receipt_outputs in 1_u8..=3 {
        let subject = shape(1, receipt_outputs, 0, Absent);
        let fragment = crossing_destination_form_fragment(&target, subject).expect("builds");

        assert!(
            opens_no_value_payload(&fragment),
            "the crossing fragment must open no value payload at {receipt_outputs} \
             destinations",
        );
        for use_of_field in value_field_uses(&fragment) {
            assert_eq!(use_of_field, ValueFieldUse::FormCheckedThenDropped);
        }
    }
}
