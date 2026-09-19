//! Contract and mutation oracles for STATE structural fragments.
//!
//! The byte strings below are public, meaningless fixture substitutions.
//! Their requirements remain unresolved even when a walk succeeds.

use std::collections::{BTreeMap, BTreeSet};

use compiler::operation_plan::RequiredSourceKind;
use target_elements::{
    ElementsCapability, EncodingClass, FailureCause, OpcodeId, StackValueType,
    TargetEvidenceRequirementId,
};

use super::reviewed_target;
use crate::instruction::{StackItem, TapscriptInstruction};
use crate::pattern::{number, op, prefix};
use crate::program::TapscriptProgram;
use crate::stack::{
    AbstractExecutionResult, AbstractLimits, AbstractStackState, resource_projection,
    validate_program,
};
use crate::state_pattern::{
    StateDisclosure, StateExternalEvidenceRole, StatePattern, StatePatternBindings,
    StatePatternConstructibility, StatePatternId, StatePatternOwner, StatePatternRecipe,
    StatePatternRefusal, StatePatternResidual, StatePatternSymbol, StatePatternWitness,
    StateStructuralEvidence, build_state_pattern, state_structural_fragment,
    state_structural_patterns,
};

fn values() -> BTreeMap<StatePatternSymbol, StackItem> {
    use StatePatternSymbol as S;
    let target = reviewed_target();
    BTreeMap::from([
        (
            S::StateAsset,
            StackItem::new(&target, vec![0x11; 32]).unwrap(),
        ),
        (S::StateAmount, StackItem::signed_le64(&target, 1)),
    ])
}

fn bindings() -> StatePatternBindings {
    StatePatternBindings::new(&reviewed_target(), values()).unwrap()
}

fn pattern(id: StatePatternId) -> StatePattern {
    let target = reviewed_target();
    let bindings = bindings();
    let fragment = state_structural_fragment(&target, &bindings, id).unwrap();
    build_state_pattern(&target, &bindings, id, fragment).unwrap()
}

fn recipe() -> StatePatternRecipe {
    state_structural_patterns(&reviewed_target(), &bindings()).unwrap()
}

fn walk(fragment: &TapscriptProgram) -> AbstractExecutionResult {
    let target = reviewed_target();
    validate_program(
        &target,
        fragment,
        &AbstractStackState::from_main(Vec::new()),
        AbstractLimits::for_target(&target),
    )
    .unwrap()
}

fn opcode_count(fragment: &TapscriptProgram, opcode: OpcodeId) -> usize {
    fragment
        .instructions()
        .iter()
        .filter(|instruction| **instruction == op(opcode))
        .count()
}

fn assert_contract(
    id: StatePatternId,
    owner: StatePatternOwner,
    expected: &BTreeSet<ElementsCapability>,
) {
    let record = pattern(id);
    let walked = walk(record.fragment());
    let empty = AbstractStackState::from_main(Vec::new());
    assert_eq!(record.id(), id);
    assert_eq!(record.owner(), owner);
    assert_eq!(record.precondition(), &empty);
    assert_eq!(record.success(), &BTreeSet::from([empty]));
    assert_eq!(record.nonaborting_failure(), &BTreeSet::new());
    assert_eq!(record.success(), walked.success());
    assert_eq!(record.nonaborting_failure(), walked.nonaborting_failure());
    assert_eq!(record.aborts(), walked.aborts());
    assert!(record.aborts().contains(&FailureCause::UnequalOperands));
    assert_eq!(record.signature_forms(), &BTreeMap::new());
    assert!(!record.reaches_unverified_success());
    assert_eq!(record.witness(), StatePatternWitness::NoWitnessItem);
    assert_eq!(
        record.constructibility(),
        StatePatternConstructibility::CheckedUnresolvedConsumers
    );
    assert_eq!(record.prerequisites(), expected);
    assert_eq!(
        record.resources(),
        &resource_projection(&reviewed_target(), record.fragment())
    );
    assert!(record.resources().values().any(|&cost| cost > 0));
}

// Three instructions, one introspected field, one comparison. Everything the
// coordinator used to carry in front of this pin was a claim about positions
// the operation does not own.
#[test]
fn coordinator_contract_owns_only_the_executing_input_position() {
    use ElementsCapability as C;
    assert_contract(
        StatePatternId::StateCoordinatorRoleV1,
        StatePatternOwner::CoordinatorRole,
        &BTreeSet::from([C::CurrentInputIndexInspection, C::ByteStringEquality]),
    );
    let record = pattern(StatePatternId::StateCoordinatorRoleV1);
    assert_eq!(record.fragment().instructions().len(), 3);
    assert_eq!(
        opcode_count(record.fragment(), OpcodeId::PushCurrentInputIndex),
        1
    );
    assert!(record.consumers().is_empty());
    for absent in [
        OpcodeId::InspectNumInputs,
        OpcodeId::InspectNumOutputs,
        OpcodeId::InspectInputIssuance,
        OpcodeId::InspectOutputAsset,
    ] {
        assert_eq!(opcode_count(record.fragment(), absent), 0);
    }
}

#[test]
fn predecessor_contract_owns_exact_asset_amount_and_script_version() {
    use ElementsCapability as C;
    assert_contract(
        StatePatternId::StateInputRecognitionV1,
        StatePatternOwner::PredecessorRecognition,
        &BTreeSet::from([
            C::InputAssetInspection,
            C::InputValueInspection,
            C::InputProgramInspection,
            C::ByteStringEquality,
            // Discarding the introspected program is a rearrangement.
            C::StackRearrangement,
        ]),
    );
    let record = pattern(StatePatternId::StateInputRecognitionV1);
    assert_eq!(
        opcode_count(record.fragment(), OpcodeId::InspectInputValue),
        1
    );
    assert_eq!(
        opcode_count(record.fragment(), OpcodeId::InspectInputScriptPubKey),
        1
    );
    assert_eq!(record.metadata().structural(), &BTreeSet::new());
}

// Replace a single introspected field with a checked observation. Other fields
// retain their abstract alternatives, so a false check must kill every path.
fn observed(
    fragment: &TapscriptProgram,
    opcode: OpcodeId,
    occurrence: usize,
    replacement: Vec<TapscriptInstruction>,
) -> TapscriptProgram {
    let mut instructions = fragment.instructions().to_vec();
    let index = instructions
        .iter()
        .enumerate()
        .filter(|(_, instruction)| **instruction == op(opcode))
        .nth(occurrence)
        .map(|(index, _)| index)
        .unwrap();
    let start = if matches!(opcode, OpcodeId::PushCurrentInputIndex) {
        index
    } else {
        index - 1
    };
    instructions.splice(start..=index, replacement);
    TapscriptProgram::new(instructions).unwrap()
}

fn field(payload: StackItem, class: EncodingClass) -> Vec<TapscriptInstruction> {
    vec![
        TapscriptInstruction::Push(payload),
        TapscriptInstruction::Push(prefix(&reviewed_target(), class).unwrap()),
    ]
}

fn asset(byte: u8) -> Vec<TapscriptInstruction> {
    field(
        StackItem::new(&reviewed_target(), vec![byte; 32]).unwrap(),
        EncodingClass::ExplicitAsset,
    )
}

// A second object of this family spent at input one cannot run this leaf: the
// pin refuses every index but zero. No count is involved, and none is needed.
#[test]
fn the_singleton_at_an_input_other_than_zero_is_refused_by_the_pin() {
    let record = pattern(StatePatternId::StateCoordinatorRoleV1);
    for index in [1, 2, 7, 255] {
        let fragment = observed(
            record.fragment(),
            OpcodeId::PushCurrentInputIndex,
            0,
            vec![number(&reviewed_target(), index).unwrap()],
        );
        assert!(walk(&fragment).always_aborts());
    }
    let exact = observed(
        record.fragment(),
        OpcodeId::PushCurrentInputIndex,
        0,
        vec![number(&reviewed_target(), 0).unwrap()],
    );
    assert_eq!(walk(&exact).success(), record.success());
}

#[test]
fn predecessor_rejects_wrong_asset_and_amount() {
    let record = pattern(StatePatternId::StateInputRecognitionV1);
    let wrong_asset = observed(
        record.fragment(),
        OpcodeId::InspectInputAsset,
        0,
        asset(0x99),
    );
    assert!(walk(&wrong_asset).always_aborts());
    let wrong_amount = field(
        StackItem::signed_le64(&reviewed_target(), 2),
        EncodingClass::ExplicitValue,
    );
    let fragment = observed(
        record.fragment(),
        OpcodeId::InspectInputValue,
        0,
        wrong_amount,
    );
    assert!(walk(&fragment).always_aborts());
}

// Recognition checks the consumed script version and nothing about the program
// bytes: any program observed at input zero walks to the same empty success
// stack. Binding that program is the semantic authentication component's work,
// which verifies the tweak over the internal key and the authenticated
// metadata rather than comparing the bytes against a literal.
#[test]
fn predecessor_rejects_a_wrong_script_version_and_admits_any_program() {
    let record = pattern(StatePatternId::StateInputRecognitionV1);
    let wrong_version = observed(
        record.fragment(),
        OpcodeId::InspectInputScriptPubKey,
        0,
        vec![
            TapscriptInstruction::Push(StackItem::new(&reviewed_target(), vec![0x33; 32]).unwrap()),
            number(&reviewed_target(), 0).unwrap(),
        ],
    );
    assert!(walk(&wrong_version).always_aborts());
    for byte in [0x33, 0x99] {
        let admitted = observed(
            record.fragment(),
            OpcodeId::InspectInputScriptPubKey,
            0,
            vec![
                TapscriptInstruction::Push(
                    StackItem::new(&reviewed_target(), vec![byte; 32]).unwrap(),
                ),
                number(&reviewed_target(), 1).unwrap(),
            ],
        );
        assert!(!walk(&admitted).always_aborts());
    }
}

#[test]
fn exact_predecessor_observations_preserve_the_empty_success_stack() {
    let record = pattern(StatePatternId::StateInputRecognitionV1);
    let with_asset = observed(
        record.fragment(),
        OpcodeId::InspectInputAsset,
        0,
        asset(0x11),
    );
    let with_amount = observed(
        &with_asset,
        OpcodeId::InspectInputValue,
        0,
        field(
            StackItem::signed_le64(&reviewed_target(), 1),
            EncodingClass::ExplicitValue,
        ),
    );
    let complete = observed(
        &with_amount,
        OpcodeId::InspectInputScriptPubKey,
        0,
        vec![
            TapscriptInstruction::Push(StackItem::new(&reviewed_target(), vec![0x33; 32]).unwrap()),
            number(&reviewed_target(), 1).unwrap(),
        ],
    );
    assert_eq!(walk(&complete).success(), record.success());
}

// No instruction of the recipe pushes the consumed program.
#[test]
fn no_structural_fragment_pushes_a_predecessor_program_literal() {
    let target = reviewed_target();
    let literal = TapscriptInstruction::Push(StackItem::new(&target, vec![0x33; 32]).unwrap());
    let recipe = recipe();
    for record in recipe.components() {
        assert!(!record.fragment().instructions().contains(&literal));
    }
    assert_eq!(
        recipe.consumers().keys().copied().collect::<Vec<_>>(),
        StatePatternSymbol::ALL
    );
}

// Every introspection the structural side performs names position zero, and
// the primitives that could reach another position are absent outright. A
// transaction may therefore carry any number of further inputs and outputs, of
// any other asset, in any arrangement — a sponsor suffix of any length, no
// sponsor at all, a fee output that is not last, several of each — and none of
// it is observable here, let alone refusable.
#[test]
fn the_structural_side_introspects_position_zero_and_nothing_else() {
    let target = reviewed_target();
    for record in recipe().components() {
        let instructions = record.fragment().instructions();
        for (index, instruction) in instructions.iter().enumerate() {
            let TapscriptInstruction::Opcode(opcode) = instruction else {
                continue;
            };
            if !matches!(
                opcode,
                OpcodeId::InspectInputAsset
                    | OpcodeId::InspectInputValue
                    | OpcodeId::InspectInputScriptPubKey
            ) {
                continue;
            }
            let Some(TapscriptInstruction::Push(position)) = index
                .checked_sub(1)
                .and_then(|index| instructions.get(index))
            else {
                panic!("introspection without a literal position")
            };
            assert_eq!(position.script_number_value(&target), Some(0));
        }
        for absent in [
            OpcodeId::InspectNumInputs,
            OpcodeId::InspectNumOutputs,
            OpcodeId::InspectInputIssuance,
            OpcodeId::InspectOutputAsset,
            OpcodeId::InspectOutputValue,
            OpcodeId::InspectOutputScriptPubKey,
        ] {
            assert_eq!(opcode_count(record.fragment(), absent), 0);
        }
    }
}

fn refuse(
    id: StatePatternId,
    instructions: Vec<TapscriptInstruction>,
    expected: StatePatternRefusal,
) {
    let result = build_state_pattern(
        &reviewed_target(),
        &bindings(),
        id,
        TapscriptProgram::new(instructions).unwrap(),
    );
    assert_eq!(result, Err(expected));
}

#[test]
fn unconsumed_boolean_is_refused_for_every_identity() {
    for &id in StatePatternId::ALL {
        let mut instructions = pattern(id).fragment().instructions().to_vec();
        // Recognition ends by discarding the introspected program, so the
        // verifying comparison it must not lose is the one before that.
        if id == StatePatternId::StateInputRecognitionV1 {
            assert_eq!(instructions.pop(), Some(op(OpcodeId::Drop)));
        }
        assert_eq!(instructions.pop(), Some(op(OpcodeId::EqualVerify)));
        instructions.push(op(OpcodeId::Equal));
        let weakened = TapscriptProgram::new(instructions.clone()).unwrap();
        assert_ne!(walk(&weakened).success(), pattern(id).success());
        refuse(id, instructions, StatePatternRefusal::FragmentMismatch);
    }
}

#[test]
fn every_changed_comparand_is_refused_before_it_can_inherit_an_identity() {
    let target = reviewed_target();
    for &id in StatePatternId::ALL {
        let canonical = pattern(id);
        for (index, instruction) in canonical.fragment().instructions().iter().enumerate() {
            let TapscriptInstruction::Push(item) = instruction else {
                continue;
            };
            let mut bytes = item.bytes().to_vec();
            bytes.push(0x7e);
            let mut instructions = canonical.fragment().instructions().to_vec();
            instructions[index] =
                TapscriptInstruction::Push(StackItem::new(&target, bytes).unwrap());
            refuse(id, instructions, StatePatternRefusal::FragmentMismatch);
        }
    }
}

#[test]
fn deleting_any_instruction_refuses_the_structural_identity() {
    for &id in StatePatternId::ALL {
        let canonical = pattern(id);
        for index in 0..canonical.fragment().instructions().len() {
            let mut instructions = canonical.fragment().instructions().to_vec();
            instructions.remove(index);
            if instructions.is_empty() {
                continue;
            }
            refuse(id, instructions, StatePatternRefusal::FragmentMismatch);
        }
    }
}

#[test]
fn recipe_metadata_is_the_component_union_in_every_dimension() {
    let recipe = recipe();
    let components = recipe.components();
    assert_eq!(
        recipe.metadata().sources(),
        &components
            .iter()
            .flat_map(|record| record.metadata().sources().iter().copied())
            .collect()
    );
    assert_eq!(
        recipe.metadata().evidence(),
        &components
            .iter()
            .flat_map(|record| record.metadata().evidence().iter().copied())
            .collect()
    );
    assert_eq!(
        recipe.metadata().disclosure(),
        &components
            .iter()
            .flat_map(|record| record.metadata().disclosure().iter().copied())
            .collect()
    );
    assert_eq!(
        recipe.metadata().residuals(),
        &components
            .iter()
            .flat_map(|record| record.metadata().residuals().iter().copied())
            .collect()
    );
    assert_eq!(
        recipe.metadata().structural(),
        &components
            .iter()
            .flat_map(|record| record.metadata().structural().iter().copied())
            .collect()
    );
    assert_eq!(
        recipe.metadata().disclosure(),
        &BTreeSet::from([
            StateDisclosure::AssetIdentities,
            StateDisclosure::PredecessorCommitment,
        ])
    );
    assert!(
        recipe
            .metadata()
            .residuals()
            .contains(&StatePatternResidual::CurrentStateRootFreshness)
    );
}

#[test]
fn every_opcode_evidence_dependency_survives_the_recipe_union() {
    let target = reviewed_target();
    let recipe = recipe();
    for record in recipe.components() {
        for instruction in record.fragment().instructions() {
            if let TapscriptInstruction::Opcode(opcode) = instruction {
                for evidence in target.definition().opcodes()[opcode].evidence() {
                    assert!(record.metadata().evidence().contains(evidence));
                    assert!(recipe.metadata().evidence().contains(evidence));
                }
            }
        }
    }
}

#[test]
fn root_freshness_and_semantic_metadata_remain_distinct_residuals() {
    let record = pattern(StatePatternId::StateInputRecognitionV1);
    let residuals = record.metadata().residuals();
    for residual in [
        StatePatternResidual::CurrentStateRootFreshness,
        StatePatternResidual::SemanticMetadataAuthentication,
    ] {
        assert!(residuals.contains(&residual));
    }
    assert!(
        record
            .metadata()
            .sources()
            .contains(&RequiredSourceKind::AuthenticatedInputObject)
    );
    assert!(record.metadata().structural().is_empty());
}

// The one absence the recipe still claims, and the three things that carry it:
// the pin is in these bytes, the other two are facts the recipe names as
// external roles rather than checks.
#[test]
fn singleton_exclusivity_is_claimed_once_and_names_its_two_external_facts() {
    let record = pattern(StatePatternId::StateCoordinatorRoleV1);
    assert_eq!(
        record.metadata().structural(),
        &BTreeSet::from([StateStructuralEvidence::SingletonExclusiveToPositionZero])
    );
    assert_eq!(
        record.metadata().external(),
        &BTreeSet::from([
            StateExternalEvidenceRole::SubstrateConservation,
            StateExternalEvidenceRole::SingletonNonReissuable,
            StateExternalEvidenceRole::SingletonIssuedUnderConstructor,
        ])
    );
    assert!(
        record
            .metadata()
            .residuals()
            .contains(&StatePatternResidual::SubstrateConservation)
    );
    assert_eq!(StateStructuralEvidence::ALL.len(), 1);
    assert_eq!(
        recipe().metadata().external(),
        &StateExternalEvidenceRole::ALL.iter().copied().collect()
    );
}

// An external role is an obligation this walk does not discharge; it is not
// target evidence the emitted opcodes require, and it is not a structural fact.
#[test]
fn external_roles_are_not_reported_as_structural_or_opcode_evidence() {
    let target = reviewed_target();
    let recipe = recipe();
    let mut opcode_evidence: BTreeSet<TargetEvidenceRequirementId> = BTreeSet::new();
    for record in recipe.components() {
        for instruction in record.fragment().instructions() {
            if let TapscriptInstruction::Opcode(opcode) = instruction {
                opcode_evidence.extend(target.definition().opcodes()[opcode].evidence());
            }
        }
    }
    assert!(!opcode_evidence.is_empty());
    assert!(
        !recipe
            .metadata()
            .evidence()
            .contains(&TargetEvidenceRequirementId::FeeOutputForm)
    );
    assert!(
        !recipe
            .metadata()
            .evidence()
            .contains(&TargetEvidenceRequirementId::FeelessTransactionAdmission)
    );
    assert!(
        !recipe
            .metadata()
            .structural()
            .iter()
            .any(|fact| *fact != StateStructuralEvidence::SingletonExclusiveToPositionZero)
    );
}

#[test]
fn identity_and_owner_censuses_are_deterministic_and_distinct() {
    assert_eq!(StatePatternId::ALL.len(), 2);
    assert_eq!(StatePatternOwner::ALL.len(), 2);
    assert!(StatePatternId::ALL.windows(2).all(|pair| pair[0] < pair[1]));
    let owners: BTreeSet<_> = StatePatternId::ALL
        .iter()
        .map(|&id| pattern(id).owner())
        .collect();
    assert_eq!(owners, StatePatternOwner::ALL.iter().copied().collect());
    assert_eq!(recipe(), recipe());
}

#[test]
fn consumers_are_unique_nonempty_exact_push_sites_and_still_unresolved() {
    let recipe = recipe();
    let substitutions = values();
    assert_eq!(recipe.consumers().len(), 2);
    assert_eq!(
        recipe.consumers().keys().copied().collect::<Vec<_>>(),
        StatePatternSymbol::ALL
    );
    let mut all_sites = BTreeSet::new();
    for (&symbol, requirement) in recipe.consumers() {
        assert_eq!(requirement.symbol, symbol);
        assert_ne!(requirement.sites, BTreeMap::new());
        for (&id, sites) in &requirement.sites {
            assert_ne!(sites, &BTreeSet::new());
            let record = recipe
                .components()
                .iter()
                .find(|record| record.id() == id)
                .unwrap();
            for &index in sites {
                assert!(all_sites.insert((id, index)));
                assert_eq!(
                    record.fragment().instructions()[index],
                    TapscriptInstruction::Push(substitutions[&symbol].clone())
                );
            }
            assert!(
                record
                    .metadata()
                    .residuals()
                    .contains(&StatePatternResidual::UnresolvedConsumers)
            );
        }
    }
    // Both consumers are recognition's; the pin resolves nothing.
    assert_eq!(all_sites.len(), 2);
}

#[test]
fn consumer_sites_union_every_component_site_without_inventing_a_consumer() {
    let recipe = recipe();
    for (&symbol, requirement) in recipe.consumers() {
        let expected: BTreeMap<_, _> = recipe
            .components()
            .iter()
            .filter_map(|record| record.consumers().get(&symbol))
            .flat_map(|component| component.sites.clone())
            .collect();
        assert_eq!(requirement.sites, expected);
    }
    for record in recipe.components() {
        for symbol in record.consumers().keys() {
            assert!(recipe.consumers().contains_key(symbol));
        }
    }
}

#[test]
fn missing_and_unused_bindings_are_refused() {
    let target = reviewed_target();
    for symbol in StatePatternSymbol::ALL {
        let mut short = values();
        short.remove(symbol);
        assert_eq!(
            StatePatternBindings::new(&target, short),
            Err(StatePatternRefusal::ConsumerCensus)
        );
    }
    assert_eq!(
        StatePatternBindings::new(&target, BTreeMap::new()),
        Err(StatePatternRefusal::ConsumerCensus)
    );
}

#[test]
fn malformed_symbol_domains_are_refused() {
    use StatePatternSymbol as S;
    let target = reviewed_target();
    for (symbol, item) in [
        (
            S::StateAsset,
            StackItem::new(&target, vec![0x11; 31]).unwrap(),
        ),
        (S::StateAmount, StackItem::signed_le64(&target, 0)),
        (S::StateAmount, StackItem::signed_le64(&target, -1)),
    ] {
        let mut malformed = values();
        malformed.insert(symbol, item);
        assert_eq!(
            StatePatternBindings::new(&target, malformed),
            Err(StatePatternRefusal::InvalidBinding(symbol))
        );
    }
}

#[test]
fn recipes_refuse_missing_duplicate_reordered_and_incompatible_components() {
    let components: Vec<_> = StatePatternId::ALL.iter().map(|&id| pattern(id)).collect();
    for broken in [
        vec![components[0].clone()],
        vec![components[1].clone()],
        vec![components[1].clone(), components[0].clone()],
        vec![components[0].clone(), components[0].clone()],
        Vec::new(),
    ] {
        assert_eq!(
            StatePatternRecipe::new(broken),
            Err(StatePatternRefusal::ComponentRecipe)
        );
    }
    let target = reviewed_target();
    let mut other = values();
    other.insert(
        StatePatternSymbol::StateAsset,
        StackItem::new(&target, vec![0x77; 32]).unwrap(),
    );
    let other = StatePatternBindings::new(&target, other).unwrap();
    let incompatible = StatePatternId::StateInputRecognitionV1;
    let fragment = state_structural_fragment(&target, &other, incompatible).unwrap();
    let mismatched = build_state_pattern(&target, &other, incompatible, fragment).unwrap();
    assert_eq!(
        StatePatternRecipe::new(vec![components[0].clone(), mismatched]),
        Err(StatePatternRefusal::ComponentRecipe)
    );
    StatePatternRecipe::new(components).unwrap();
}

#[test]
fn boolean_producers_are_immediately_verified_and_equal_uses_the_verifying_form() {
    for record in recipe().components() {
        assert_eq!(opcode_count(record.fragment(), OpcodeId::Equal), 0);
        assert!(opcode_count(record.fragment(), OpcodeId::EqualVerify) > 0);
    }
}

#[test]
fn every_structural_fragment_preserves_extra_witness_as_forbidden_residue() {
    let target = reviewed_target();
    let extra = AbstractStackState::from_main(vec![StackValueType::Bytes {
        minimum: 1,
        maximum: 1,
    }]);
    for record in recipe().components() {
        let execution = validate_program(
            &target,
            record.fragment(),
            &extra,
            AbstractLimits::for_target(&target),
        )
        .unwrap();
        assert_ne!(execution.success(), record.success());
        assert!(
            execution
                .success()
                .iter()
                .all(|state| !state.main().is_empty())
        );
    }
}

#[test]
fn structural_refusal_declaration_has_exact_exercised_and_unreachable_census() {
    super::state_program_tests::assert_refusal_census(
        include_str!("../state_pattern.rs"),
        "StatePatternRefusal",
        &[
            ("ConsumerCensus", missing_and_unused_bindings_are_refused),
            ("InvalidBinding", malformed_symbol_domains_are_refused),
            (
                "FragmentMismatch",
                deleting_any_instruction_refuses_the_structural_identity,
            ),
            (
                "ComponentRecipe",
                recipes_refuse_missing_duplicate_reordered_and_incompatible_components,
            ),
        ],
        &[
            (
                "Program",
                "Emission is a fixed handful of instructions over checked bindings, and a changed fragment is refused by recipe comparison before it is walked; reaching this needs an internal emission seam.",
            ),
            (
                "InvalidContract",
                "Exact canonical emission precedes the walk; needs an internal contract checker accepting a walked mutation.",
            ),
        ],
    );
}
