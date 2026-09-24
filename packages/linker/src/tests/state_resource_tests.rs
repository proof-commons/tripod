//! What the linked maturity leaf costs, asserted against the real
//! program.
//!
//! # The figures are read, never restated
//!
//! No total below is written down. Each is compared with the record's
//! own projection, with the encoder's own count, or with the reviewed
//! contract's own per-primitive figures, so a test that passed on a
//! program the emitters no longer produce is not available here.
//!
//! # The fixtures are the relocation bite's
//!
//! The second deployment's sources live beside the relocation tests
//! because a resource measurement needs a linked program and the link is
//! what produces one.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

use realization::{STATE_METADATA_LAYOUT, STATE_METADATA_VARIABLE_BYTES};
use tapscript::pattern::fragment_prerequisites;
use tapscript::{
    AbstractLimits, FinalStackDefect, MAXIMUM_PROGRAM_INSTRUCTIONS, StateAnnouncementId,
    StateAnnouncementProgram, StateProgramComponent, StateProgramWitness,
    StateWitnessLoweringRefusal, StateWitnessSchedule, TapscriptInstruction, TapscriptProgram,
    final_stack_defects, legalize_state_witness_schedule, program_stack_profile,
};
use target_elements::{
    ElementsCapability, EncodingClass, OpcodeId, OperandContract, PayloadWidth, ResourceBound,
    ResourceDimension, StackValueType,
};

use super::state_relocate_tests::{resolved_census, second_resolved_census};
use crate::state_resource::{compare_initial_arguments, projection_disagreement};
use crate::tests::{
    bridge_for_record, declaration, record, record_for_schedule, reviewed_target, singleton,
    state_constructor,
};
use crate::{
    InitialArgumentBound, InitialArgumentOutcome, InitialArgumentVerdict, InitialArgumentWidth,
    LinkedStateLeafProgram, StateConsumerCensus, StateLinkRefusal, StateLinkedResources,
    StateResourceGap, collect_state_definitions, compare_initial_argument_widths,
    measure_state_resources, measure_state_totals, resolve_state_census, substitute_state,
};

/// The streaming hash primitives, as this file's own list.
const STREAMING_HASH: [OpcodeId; 3] = [
    OpcodeId::Sha256Initialize,
    OpcodeId::Sha256Update,
    OpcodeId::Sha256Finalize,
];

/// The demonstration deployment's measurement, taken once.
///
/// The peak-stack figure is the abstract walk's own profile over every
/// prefix, so the measurement is paid for once here and handed out by
/// clone rather than recomputed by each test.
fn demonstration_resources() -> StateLinkedResources {
    static RESOURCES: LazyLock<StateLinkedResources> = LazyLock::new(|| {
        let target = reviewed_target();
        let composed = record();
        let linked = substitute_state(&target, &composed, &resolved_census())
            .expect("the demonstration deployment links");
        measure_state_resources(&target, &composed, &linked)
            .expect("the demonstration measurement completes")
    });
    RESOURCES.clone()
}

/// The second deployment's measurement, taken once.
fn second_resources() -> StateLinkedResources {
    static RESOURCES: LazyLock<StateLinkedResources> = LazyLock::new(|| {
        let target = reviewed_target();
        let composed = record();
        let linked = substitute_state(&target, &composed, &second_resolved_census())
            .expect("the second deployment links");
        measure_state_resources(&target, &composed, &linked)
            .expect("the second measurement completes")
    });
    RESOURCES.clone()
}

/// The linked variable schedule, with its own record-bound bridge.
fn variable_linked() -> (StateAnnouncementProgram, LinkedStateLeafProgram) {
    static LINKED: LazyLock<(StateAnnouncementProgram, LinkedStateLeafProgram)> =
        LazyLock::new(|| {
            let target = reviewed_target();
            let record = record_for_schedule(StateWitnessSchedule::VariableMetadata);
            let constructor = state_constructor();
            let definitions = collect_state_definitions(
                &target,
                &bridge_for_record(&record),
                &constructor,
                &singleton(),
                &declaration(),
                record.schedule(),
            )
            .expect("variable definitions collect");
            let consumers = StateConsumerCensus::from_sources(&record, &constructor);
            let resolved = resolve_state_census(&definitions, &consumers)
                .expect("variable definitions resolve against their consumers");
            assert_eq!(definitions.len(), 14);
            assert_eq!(
                definitions.len(),
                record.consumers().len() + constructor.reference_declarations().len()
            );
            assert_eq!(resolved.entries().len(), definitions.len());
            assert_eq!(resolved.push_site_count(), 16);
            assert_eq!(
                resolved.push_site_count(),
                record
                    .consumers()
                    .values()
                    .map(|consumer| consumer.sites.len())
                    .sum::<usize>()
            );
            let linked = substitute_state(&target, &record, &resolved)
                .expect("the variable schedule substitutes into a linked program");
            (record, linked)
        });
    LINKED.clone()
}

/// The variable linked program's measurement, taken once.
fn variable_resources() -> StateLinkedResources {
    static RESOURCES: LazyLock<StateLinkedResources> = LazyLock::new(|| {
        let (record, linked) = variable_linked();
        measure_state_resources(&reviewed_target(), &record, &linked)
            .expect("the variable linked program measures")
    });
    RESOURCES.clone()
}

/// Whether the composed program schedules one primitive.
fn schedules(program: &TapscriptProgram, id: OpcodeId) -> bool {
    program.instructions().iter().any(
        |instruction| matches!(instruction, TapscriptInstruction::Opcode(scheduled) if *scheduled == id),
    )
}

/// Read an exact width directly from the fixture's declared type.
fn exact_width(declared: &StackValueType) -> u64 {
    let width = match declared {
        StackValueType::Bytes { minimum, maximum } if minimum == maximum => *minimum,
        StackValueType::Encoded(class) => match class.v1_shape().payload() {
            PayloadWidth::Exact(width) => width.get(),
            other => panic!("the fixture's encoded argument must be exact: {other:?}"),
        },
        other => panic!("the fixture's argument must be exact: {other:?}"),
    };
    u64::try_from(width).expect("the declared width fits a resource unit")
}

// --- (a) The checked totals ---------------------------------------------

// Every checked total of the linked demonstration program, against the
// source that fixes it: the encoder for the bytes, the record's own
// projection for the three dimensions it carries, the declared witness
// for the initial stack, and the program itself for the instructions.
#[test]
fn the_checked_totals_are_the_linked_program_s_own() {
    let target = reviewed_target();
    let composed = record();
    let resources = demonstration_resources();
    let totals = resources.totals();

    let linked = substitute_state(&target, &composed, &resolved_census())
        .expect("the demonstration deployment links");
    assert_eq!(
        totals.instructions(),
        u64::try_from(composed.program().len()).expect("the instruction count is a magnitude")
    );
    assert_eq!(
        totals.total(ResourceDimension::ScriptBytes),
        Some(linked.program().encoded_length(&target))
    );
    assert_eq!(
        totals.total(ResourceDimension::InitialStackItems),
        Some(u64::try_from(composed.precondition().depth()).expect("the depth is a magnitude"))
    );
    assert_eq!(
        totals.total(ResourceDimension::InitialStackItems),
        Some(u64::try_from(composed.witness().len()).expect("the witness is a magnitude"))
    );
    assert_eq!(
        totals.total(ResourceDimension::InitialWitnessItemBytes),
        composed.precondition().main().iter().map(exact_width).max()
    );

    // The reviewed domain charges no operation budget for any primitive,
    // and the validation budget is charged per signature or curve check,
    // so the two totals are the contract's own figures summed.
    let budget: u64 = composed
        .program()
        .instructions()
        .iter()
        .filter_map(|instruction| match instruction {
            TapscriptInstruction::Opcode(id) => target.definition().opcodes().get(id),
            TapscriptInstruction::Push(_) => None,
        })
        .map(|spec| spec.resources().validation_budget())
        .sum();
    assert_eq!(
        totals.total(ResourceDimension::ValidationBudget),
        Some(budget)
    );
    assert_eq!(totals.total(ResourceDimension::OperationCost), Some(0));

    // Nothing is pinned, so no figure here is a saturated one wearing a
    // total's clothes.
    assert!(totals.totals().values().all(|total| *total != u64::MAX));

    // The five dimensions no program fixes are absent rather than zero.
    for dimension in [
        ResourceDimension::TransactionWeight,
        ResourceDimension::WitnessBytes,
        ResourceDimension::StackElementBytes,
        ResourceDimension::ControlPathDepth,
        ResourceDimension::PackageLimit,
    ] {
        assert_eq!(totals.total(dimension), None, "{dimension:?}");
    }
    assert_eq!(totals.totals().len(), 6);
}

// The peak stack is the walk's own, and the alternate stack never grows,
// so the combined figure is a depth rather than a bound on one.
#[test]
fn the_peak_stack_is_the_walk_s_own_combined_depth() {
    for (composed, resources) in [
        (record(), demonstration_resources()),
        (variable_linked().0, variable_resources()),
    ] {
        let peak = resources
            .totals()
            .total(ResourceDimension::PeakStackItems)
            .expect("the peak stack is measured");

        assert!(peak >= u64::try_from(composed.precondition().depth()).expect("a magnitude"));
        for state in composed.execution().success() {
            assert!(peak >= u64::try_from(state.depth()).expect("a magnitude"));
            // No reviewed primitive moves an item to the alternate stack, so
            // adding the two peaks is exact rather than an over-count.
            assert_eq!(state.alternate(), []);
        }
    }
}

// Equality is admitted; the next byte is refused. An explicit unbounded
// policy statement admits both exact widths.
#[test]
fn the_initial_argument_comparator_observes_the_policy_boundary() {
    let arguments = [
        (
            StateProgramWitness::SuccessorNonce,
            InitialArgumentWidth::Exact(80),
        ),
        (
            StateProgramWitness::RequestedCycle,
            InitialArgumentWidth::Exact(81),
        ),
    ];
    let bounded = compare_initial_argument_widths(&arguments, ResourceBound::Maximum(80));
    assert_eq!(
        bounded.bound(),
        InitialArgumentBound::Stated(ResourceBound::Maximum(80))
    );
    assert_eq!(
        bounded.arguments()[0].outcome(),
        InitialArgumentOutcome::WithinBound
    );
    assert_eq!(
        bounded.arguments()[1].outcome(),
        InitialArgumentOutcome::OverBound
    );
    assert_eq!(bounded.over_bound_positions(), &BTreeSet::from([1]));
    assert_eq!(bounded.verdict(), InitialArgumentVerdict::Refused);

    let exact_boundary =
        compare_initial_argument_widths(&arguments[..1], ResourceBound::Maximum(80));
    assert_eq!(exact_boundary.verdict(), InitialArgumentVerdict::Admitted);
    let unbounded = compare_initial_argument_widths(&arguments, ResourceBound::Unbounded);
    assert_eq!(unbounded.verdict(), InitialArgumentVerdict::Admitted);
    assert!(unbounded.over_bound_positions().is_empty());
}

// A type without an exact width stays visible in the returned value and
// does not become admitted merely because a policy says unbounded.
#[test]
fn a_non_exact_declaration_has_a_typed_refused_entry() {
    let declared = StackValueType::ScriptNumber;
    let admission = compare_initial_argument_widths(
        &[(
            StateProgramWitness::RequestedCycle,
            InitialArgumentWidth::NoExactWidth(declared.clone()),
        )],
        ResourceBound::Unbounded,
    );
    assert_eq!(admission.arguments().len(), 1);
    assert_eq!(admission.arguments()[0].position(), 0);
    assert_eq!(
        admission.arguments()[0].role(),
        StateProgramWitness::RequestedCycle
    );
    assert_eq!(
        admission.arguments()[0].width(),
        &InitialArgumentWidth::NoExactWidth(declared)
    );
    assert_eq!(
        admission.arguments()[0].outcome(),
        InitialArgumentOutcome::NoExactWidth
    );
    assert!(admission.over_bound_positions().is_empty());
    assert_eq!(admission.verdict(), InitialArgumentVerdict::Refused);
}

// A missing policy statement is not the reviewed assertion Unbounded.
#[test]
fn no_bound_stated_does_not_establish_admission() {
    let admission = compare_initial_arguments(
        &[(
            StateProgramWitness::SuccessorNonce,
            InitialArgumentWidth::Exact(4),
        )],
        InitialArgumentBound::NoBoundStated,
    );
    assert_eq!(admission.bound(), InitialArgumentBound::NoBoundStated);
    assert_eq!(
        admission.arguments()[0].outcome(),
        InitialArgumentOutcome::NoBoundStated
    );
    assert_eq!(admission.verdict(), InitialArgumentVerdict::Refused);
    assert!(admission.over_bound_positions().is_empty());
}

// The historical argument over the relay policy is measured, while the
// replay-only schedule still resolves to the same linked program.
#[test]
fn the_linked_whole_schedule_retains_its_policy_refusal() {
    let target = reviewed_target();
    let record = record();
    let linked = substitute_state(&target, &record, &resolved_census())
        .expect("the historical program still links");
    let resources = demonstration_resources();
    let admission = resources.initial_argument_admission();
    let bound = target.definition().resources().policy().bounds()
        [&ResourceDimension::InitialWitnessItemBytes];
    let metadata_position = record
        .witness()
        .iter()
        .position(|(role, _)| *role == StateProgramWitness::PredecessorMetadata)
        .expect("the record declares predecessor metadata");

    assert!(record.schedule().is_replay_only());
    assert_eq!(
        linked.program().encode(&target),
        record.program().encode(&target)
    );
    assert_eq!(record.precondition().depth(), 7);
    assert_eq!(record.precondition().alternate(), []);
    assert_eq!(admission.arguments().len(), record.precondition().depth());
    assert_eq!(admission.bound(), InitialArgumentBound::Stated(bound));
    for (position, ((role, _), declared)) in record
        .witness()
        .iter()
        .zip(record.precondition().main())
        .enumerate()
    {
        let entry = &admission.arguments()[position];
        assert_eq!(entry.position(), position);
        assert_eq!(entry.role(), *role);
        assert_eq!(
            entry.width(),
            &InitialArgumentWidth::Exact(exact_width(declared))
        );
    }
    assert_eq!(
        admission.over_bound_positions(),
        &BTreeSet::from([metadata_position])
    );
    assert_eq!(
        admission.arguments()[metadata_position].outcome(),
        InitialArgumentOutcome::OverBound
    );
    assert_eq!(admission.verdict(), InitialArgumentVerdict::Refused);
    assert_eq!(
        admission.arguments()[metadata_position].width(),
        &InitialArgumentWidth::Exact(86)
    );
    assert_eq!(bound, ResourceBound::Maximum(80));
}

// All three premises for the resource refusal's reachability account
// are read from the lowering, the composed record, and the reviewed type.
#[test]
fn the_linked_variable_schedule_has_seven_admitted_arguments() {
    let target = reviewed_target();
    let (record, linked) = variable_linked();
    let admission = variable_resources().initial_argument_admission().clone();
    let bound = target.definition().resources().policy().bounds()
        [&ResourceDimension::InitialWitnessItemBytes];
    let metadata_position = record
        .witness()
        .iter()
        .position(|(role, _)| *role == StateProgramWitness::PredecessorMetadata)
        .expect("the record declares predecessor metadata");
    let signature_position = record
        .witness()
        .iter()
        .position(|(role, _)| *role == StateProgramWitness::OperatorSignature)
        .expect("the record declares the operator signature");
    let variable_width = exact_width(&record.precondition().main()[metadata_position]);
    let signature_width = exact_width(&record.precondition().main()[signature_position]);

    assert!(!record.schedule().is_replay_only());
    assert_eq!(record.precondition().depth(), 7);
    assert_eq!(record.precondition().alternate(), []);
    assert_eq!(admission.arguments().len(), record.precondition().depth());
    assert_eq!(admission.verdict(), InitialArgumentVerdict::Admitted);
    assert!(admission.over_bound_positions().is_empty());
    assert!(
        admission
            .arguments()
            .iter()
            .all(|entry| entry.outcome() == InitialArgumentOutcome::WithinBound)
    );
    assert_eq!(
        variable_width,
        u64::try_from(STATE_METADATA_VARIABLE_BYTES).expect("a width")
    );
    assert_eq!(signature_width, 64);
    assert!(signature_width > variable_width);
    assert_eq!(
        record.precondition().main()[signature_position],
        StackValueType::Encoded(EncodingClass::SchnorrSignature)
    );
    assert!(
        matches!(EncodingClass::SchnorrSignature.v1_shape().payload(), PayloadWidth::Exact(width) if width.get() == 64)
    );
    assert_eq!(
        variable_resources()
            .totals()
            .total(ResourceDimension::InitialWitnessItemBytes),
        Some(signature_width)
    );
    assert_eq!(admission.bound(), InitialArgumentBound::Stated(bound));
    assert_eq!(linked.program().len(), record.program().len());

    let narrower = ResourceBound::Maximum(variable_width - 1);
    assert_eq!(
        legalize_state_witness_schedule(record.schedule(), &STATE_METADATA_LAYOUT, narrower),
        Err(StateWitnessLoweringRefusal::VariableRegionTooWide {
            width: STATE_METADATA_VARIABLE_BYTES,
            bound: variable_width - 1,
        })
    );
}

// Both schedules' figures come from their linked encoding and walk; the
// extra bytes and instructions are the variable restoration prologue.
#[test]
fn both_linked_schedules_recompute_their_totals_and_prologue_difference() {
    let target = reviewed_target();
    let whole_record = record();
    let whole_linked = substitute_state(&target, &whole_record, &resolved_census())
        .expect("the whole schedule links");
    let (variable_record, variable_linked) = variable_linked();
    let whole = demonstration_resources();
    let variable = variable_resources();

    for (record, linked, resources) in [
        (&whole_record, &whole_linked, &whole),
        (&variable_record, &variable_linked, &variable),
    ] {
        let program = linked.program();
        let totals = resources.totals();
        let profile = program_stack_profile(
            &target,
            program,
            record.precondition(),
            AbstractLimits::for_target(&target),
        );
        assert_eq!(totals.totals().len(), 6);
        assert_eq!(
            totals.instructions(),
            u64::try_from(program.len()).expect("instruction count")
        );
        assert_eq!(
            totals.total(ResourceDimension::ScriptBytes),
            Some(program.encoded_length(&target))
        );
        assert_eq!(
            totals.total(ResourceDimension::InitialStackItems),
            Some(u64::try_from(record.precondition().depth()).expect("depth"))
        );
        assert_eq!(
            totals.total(ResourceDimension::PeakStackItems),
            Some(profile.peak_main() + profile.peak_alternate())
        );
        assert_eq!(
            totals.total(ResourceDimension::InitialWitnessItemBytes),
            record.precondition().main().iter().map(exact_width).max()
        );
    }

    let range = variable_record.components()
        [&StateProgramComponent::Semantic(StateAnnouncementId::MetadataAuthentication)]
        .clone();
    let prologue = &variable_linked.program().instructions()[range.start..range.start + 7];
    let prologue_program =
        TapscriptProgram::new(prologue.to_vec()).expect("the prologue is a program");
    let push_widths: Vec<_> = prologue
        .iter()
        .filter_map(|instruction| match instruction {
            TapscriptInstruction::Push(item) => Some(item.len()),
            TapscriptInstruction::Opcode(_) => None,
        })
        .collect();
    assert_eq!(push_widths, [25, 8]);
    assert_eq!(prologue.len() - push_widths.len(), 5);
    assert_eq!(prologue_program.encoded_length(&target), 40);
    assert_eq!(
        variable.totals().instructions() - whole.totals().instructions(),
        u64::try_from(prologue.len()).expect("prologue length")
    );
    assert_eq!(
        variable
            .totals()
            .total(ResourceDimension::ScriptBytes)
            .expect("variable script")
            - whole
                .totals()
                .total(ResourceDimension::ScriptBytes)
                .expect("whole script"),
        prologue_program.encoded_length(&target)
    );
}

// --- (b) The second deployment ------------------------------------------

// Every width is fixed by its encoding, so a second deployment's linked
// program costs exactly what the first's does.
#[test]
fn the_second_deployment_costs_what_the_first_does() {
    let first = demonstration_resources();
    let second = second_resources();

    assert_eq!(first.totals().totals(), second.totals().totals());
    assert_eq!(
        first.totals().instructions(),
        second.totals().instructions()
    );
    assert_eq!(first.observations(), second.observations());
    assert_eq!(first.gaps(), second.gaps());
    assert_eq!(first.diagnostic(), second.diagnostic());
}

// --- (c) The final-stack walk -------------------------------------------

// The walk is empty over the linked program, and over the pristine one
// it was substituted from, so the link neither introduced a defect nor
// inherited a clean report from a program with one.
#[test]
fn the_final_stack_walk_is_empty_on_both_sides_of_the_link() {
    let target = reviewed_target();
    let composed = record();

    assert_eq!(demonstration_resources().final_stack(), []);
    assert_eq!(second_resources().final_stack(), []);

    let pristine = final_stack_defects(&target, composed.program(), composed.precondition())
        .expect("the pristine program schedules");
    assert_eq!(pristine, []);
}

// --- (d) The native observations ----------------------------------------

// Present is the linked program's own prerequisite census, absent is the
// rest of the reviewed contract's roster, and the two partition it.
#[test]
fn the_observations_partition_the_capability_roster() {
    let target = reviewed_target();
    let composed = record();
    let resources = demonstration_resources();
    let observations = resources.observations();

    let linked = substitute_state(&target, &composed, &resolved_census())
        .expect("the demonstration deployment links");
    assert_eq!(
        observations.present(),
        &fragment_prerequisites(linked.program())
    );
    // Substitution moves payloads and no primitive, so the record's own
    // census is the linked program's.
    assert_eq!(observations.present(), composed.prerequisites());

    let roster: BTreeSet<ElementsCapability> = ElementsCapability::ALL.iter().copied().collect();
    assert!(!observations.present().is_empty());
    assert!(!observations.absent().is_empty());
    assert!(observations.present().is_disjoint(observations.absent()));
    assert_eq!(
        observations
            .present()
            .union(observations.absent())
            .copied()
            .collect::<BTreeSet<_>>(),
        roster
    );
}

// --- (e) The gaps -------------------------------------------------------

// Each gap is the emitter fact it comes from, recomputed here: the hash
// primitives carry no capability, the observations read primitives and
// not the literals the domains arrive as, the contract's figures for the
// hash primitives are per-opcode constants, and a budgeted primitive's
// figure is the most it can charge.
#[test]
fn every_named_gap_is_a_fact_of_the_emitters() {
    let target = reviewed_target();
    let composed = record();
    let resources = demonstration_resources();

    assert_eq!(
        resources.gaps(),
        &BTreeSet::from([
            StateResourceGap::Sha256PrimitivesCarryNoCapability,
            StateResourceGap::HashDomainSeparationIsNotObserved,
            StateResourceGap::StreamingHashCostIsPerOpcodeNotPerByte,
            StateResourceGap::ValidationBudgetIsAPerPrimitiveMaximum,
        ])
    );

    // The leaf streams hashes, and the capability that names streaming
    // hashing is reported absent regardless.
    for id in STREAMING_HASH {
        assert!(schedules(composed.program(), id), "{id:?}");
        let cost = target
            .definition()
            .opcodes()
            .get(&id)
            .expect("the reviewed contract states every primitive")
            .resources();
        assert_eq!(cost.operation_cost(), 0);
        assert_eq!(cost.validation_budget(), 0);
    }
    assert!(
        resources
            .observations()
            .absent()
            .contains(&ElementsCapability::StreamingSha256)
    );

    // The observations are derived from primitives alone: stripping
    // every literal, and with it every domain tag the leaf pushes,
    // leaves the census unchanged.
    let opcodes: Vec<TapscriptInstruction> = composed
        .program()
        .instructions()
        .iter()
        .filter(|instruction| matches!(instruction, TapscriptInstruction::Opcode(_)))
        .cloned()
        .collect();
    let stripped = TapscriptProgram::new(opcodes).expect("the primitives are a program");
    assert_eq!(
        &fragment_prerequisites(&stripped),
        resources.observations().present()
    );

    // Something in the leaf charges the validation budget, and charges
    // it only where its own operand contract admits a charge.
    assert!(composed.program().instructions().iter().any(|instruction| {
        let TapscriptInstruction::Opcode(id) = instruction else {
            return false;
        };
        let spec = target
            .definition()
            .opcodes()
            .get(id)
            .expect("the reviewed contract states every primitive");
        spec.resources().validation_budget() > 0
            && spec.stack().operands().iter().any(|operand| {
                matches!(
                    operand,
                    OperandContract::Signature {
                        empty_allowed: true,
                        ..
                    }
                )
            })
    }));
}

// --- (f) The diagnostic comparison --------------------------------------

// For every dimension the record's projection carries, the checked total
// equals it and no figure is pinned: the tests observe the equality, and
// the saturated case is an admission the module states rather than one
// anything here reaches.
#[test]
fn the_checked_totals_agree_with_the_record_s_projection() {
    let resources = demonstration_resources();
    let diagnostic = resources.diagnostic();
    assert!(!diagnostic.is_empty());

    for (&dimension, &projected) in diagnostic {
        assert_ne!(projected, u64::MAX, "{dimension:?} is pinned");
        assert_eq!(
            resources.totals().total(dimension),
            Some(projected),
            "{dimension:?}"
        );
    }
    assert_eq!(diagnostic, record().resources());
}

// --- (g) The overflow refusal, as arithmetic -----------------------------

// Why no program reaches the overflow refusal, computed from the
// reviewed contract rather than asserted: the largest per-primitive
// figure, charged at every instruction a program may carry, still fits.
#[test]
fn no_admissible_program_can_overflow_a_checked_total() {
    let target = reviewed_target();
    let heaviest = target
        .definition()
        .opcodes()
        .values()
        .map(|spec| {
            spec.resources()
                .operation_cost()
                .max(spec.resources().validation_budget())
        })
        .max()
        .expect("the reviewed contract states at least one primitive");

    assert!(heaviest > 0);
    assert!(
        heaviest
            .checked_mul(MAXIMUM_PROGRAM_INSTRUCTIONS)
            .is_some_and(|greatest| greatest < u64::MAX)
    );
}

// --- (h) The closed root, for the resource half --------------------------

/// Which test reaches one resource refusal, or why nothing can.
///
/// Called from the census bite's walk over the whole closed root, which
/// names these variants one by one.
pub(super) fn resource_reachability(refusal: &StateLinkRefusal) -> &'static str {
    match refusal {
        StateLinkRefusal::ResourceTotalOverflow { .. } => {
            "unreachable: no_admissible_program_can_overflow_a_checked_total computes why"
        }
        StateLinkRefusal::InitialArgumentNotAdmitted { .. } => {
            "unreachable: the_linked_variable_schedule_has_seven_admitted_arguments checks \
             that lowering refuses an over-wide variable span before composition, the \
             composed variable metadata has its reviewed exact width, and the signature \
             is the greatest variable argument under the reviewed bound"
        }
        StateLinkRefusal::LinkedProgramFailsTheFinalStackRule { .. } => {
            "unreachable: a linked program comes from the substitution, which admits only the \
             pristine program with equal-width payloads, and that program's walk is empty"
        }
        StateLinkRefusal::ResourceProjectionDisagreement { .. } => {
            "unreachable: both figures come from one encoder and one set of per-primitive costs, \
             differing only in saturation, which the overflow arithmetic excludes"
        }
        _ => "accounted for elsewhere in the closed root",
    }
}

// Every resource refusal is declared with a reason another test
// recomputes.
#[test]
fn every_resource_refusal_is_reached_or_declared() {
    let refusals = [
        StateLinkRefusal::ResourceTotalOverflow {
            dimension: ResourceDimension::ValidationBudget,
        },
        StateLinkRefusal::InitialArgumentNotAdmitted {
            position: 4,
            role: StateProgramWitness::PredecessorMetadata,
            width: InitialArgumentWidth::Exact(81),
            bound: InitialArgumentBound::Stated(ResourceBound::Maximum(80)),
        },
        StateLinkRefusal::LinkedProgramFailsTheFinalStackRule {
            defects: vec![FinalStackDefect::DoesNotEndOnTheCanonicalTrueItem],
        },
        StateLinkRefusal::ResourceProjectionDisagreement {
            dimension: ResourceDimension::ScriptBytes,
            diagnostic: 0,
            checked: Some(1),
        },
    ];

    let accounts: BTreeSet<&str> = refusals.iter().map(resource_reachability).collect();
    assert_eq!(accounts.len(), refusals.len());
    assert!(
        accounts
            .iter()
            .all(|account| account.starts_with("unreachable"))
    );
}

#[test]
fn an_unmeasured_projected_dimension_refuses_with_no_checked_figure() {
    let resources = demonstration_resources();
    let totals = resources.totals();
    let mut projection = BTreeMap::new();
    projection.insert(ResourceDimension::WitnessBytes, 0);
    assert_eq!(
        projection_disagreement(totals, &projection),
        Err(StateLinkRefusal::ResourceProjectionDisagreement {
            dimension: ResourceDimension::WitnessBytes,
            diagnostic: 0,
            checked: None,
        })
    );

    projection.insert(ResourceDimension::WitnessBytes, u64::MAX);
    assert_eq!(
        projection_disagreement(totals, &projection),
        Err(StateLinkRefusal::ResourceProjectionDisagreement {
            dimension: ResourceDimension::WitnessBytes,
            diagnostic: u64::MAX,
            checked: None,
        })
    );

    projection.clear();
    projection.insert(ResourceDimension::ScriptBytes, u64::MAX);
    let measured_pinned = projection_disagreement(totals, &projection);
    assert_eq!(measured_pinned, Ok(()));
    let record_projection = projection_disagreement(totals, record().resources());
    assert_eq!(record_projection, Ok(()));
}

// The measurement is over a linked program and says so in its type: a
// program nobody linked has no measurement here.
#[test]
fn the_totals_are_measured_over_the_linked_program() {
    let target = reviewed_target();
    let composed = record();
    let linked = substitute_state(&target, &composed, &second_resolved_census())
        .expect("the second deployment links");
    let (variable_record, variable_linked) = variable_linked();

    for (record, linked, resources) in [
        (&composed, &linked, second_resources()),
        (&variable_record, &variable_linked, variable_resources()),
    ] {
        let totals = measure_state_totals(&target, record, linked.program())
            .expect("the linked program measures");
        assert_eq!(totals, *resources.totals());

        // The pristine program measures to the same figures, which is
        // what the fixed widths mean and why the record's projection is
        // comparable with the linked one's at all.
        let pristine = measure_state_totals(&target, record, record.program())
            .expect("the pristine program measures");
        assert_eq!(pristine, totals);
    }
}
