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

use std::collections::BTreeSet;
use std::sync::LazyLock;

use tapscript::pattern::fragment_prerequisites;
use tapscript::{
    FinalStackDefect, MAXIMUM_PROGRAM_INSTRUCTIONS, TapscriptInstruction, TapscriptProgram,
    final_stack_defects,
};
use target_elements::{ElementsCapability, OpcodeId, OperandContract, ResourceDimension};

use super::state_relocate_tests::{resolved_census, second_resolved_census};
use crate::tests::{record, reviewed_target};
use crate::{
    StateLinkRefusal, StateLinkedResources, StateResourceGap, measure_state_resources,
    measure_state_totals, substitute_state,
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

/// Whether the composed program schedules one primitive.
fn schedules(program: &TapscriptProgram, id: OpcodeId) -> bool {
    program.instructions().iter().any(
        |instruction| matches!(instruction, TapscriptInstruction::Opcode(scheduled) if *scheduled == id),
    )
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
    assert_eq!(totals.totals().len(), 5);
}

// The peak stack is the walk's own, and the alternate stack never grows,
// so the combined figure is a depth rather than a bound on one.
#[test]
fn the_peak_stack_is_the_walk_s_own_combined_depth() {
    let composed = record();
    let totals = demonstration_resources();
    let peak = totals
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
        StateLinkRefusal::LinkedProgramFailsTheFinalStackRule {
            defects: vec![FinalStackDefect::DoesNotEndOnTheCanonicalTrueItem],
        },
        StateLinkRefusal::ResourceProjectionDisagreement {
            dimension: ResourceDimension::ScriptBytes,
            diagnostic: 0,
            checked: 1,
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

// The measurement is over a linked program and says so in its type: a
// program nobody linked has no measurement here.
#[test]
fn the_totals_are_measured_over_the_linked_program() {
    let target = reviewed_target();
    let composed = record();
    let linked = substitute_state(&target, &composed, &second_resolved_census())
        .expect("the second deployment links");

    let totals = measure_state_totals(&target, &composed, linked.program())
        .expect("the linked program measures");
    assert_eq!(totals, *second_resources().totals());

    // The pristine program measures to the same figures, which is what
    // the fixed widths mean and why the record's projection is
    // comparable with the linked one's at all.
    let pristine = measure_state_totals(&target, &composed, composed.program())
        .expect("the pristine program measures");
    assert_eq!(pristine, totals);
}
