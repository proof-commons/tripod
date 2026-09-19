//! Contract and mutation oracles for STATE structural fragments.
//!
//! The byte strings below are public, meaningless fixture substitutions.
//! Their requirements remain unresolved even when a walk succeeds.

use std::collections::{BTreeMap, BTreeSet};

use compiler::operation_plan::RequiredSourceKind;
use target_elements::{
    ElementsCapability, EncodingClass, FailureCause, OpcodeId, ResourceDimension,
    TargetEvidenceRequirementId,
};

use super::reviewed_target;
use crate::error::TapscriptError;
use crate::instruction::{StackItem, TapscriptInstruction};
use crate::live_plan::reads_a_value_field;
use crate::live_shape::FeePresence;
use crate::pattern::{number, op, prefix};
use crate::program::TapscriptProgram;
use crate::shape::SponsorChangePresence;
use crate::stack::{
    AbstractExecutionResult, AbstractLimits, AbstractStackState, resource_projection,
    validate_program,
};
use crate::state_pattern::{
    StateAnnouncementShape, StateDisclosure, StateExternalEvidenceRole, StatePattern,
    StatePatternBindings, StatePatternConstructibility, StatePatternId, StatePatternOwner,
    StatePatternRecipe, StatePatternRefusal, StatePatternResidual, StatePatternSymbol,
    StatePatternWitness, StateStructuralEvidence, build_state_pattern, state_structural_fragment,
    state_structural_patterns,
};

fn sponsored() -> StateAnnouncementShape {
    StateAnnouncementShape::new(2, SponsorChangePresence::Present, FeePresence::Present)
}

fn plain() -> StateAnnouncementShape {
    StateAnnouncementShape::new(0, SponsorChangePresence::Absent, FeePresence::Absent)
}

fn values(shape: StateAnnouncementShape) -> BTreeMap<StatePatternSymbol, StackItem> {
    use StatePatternSymbol as S;
    let target = reviewed_target();
    let item = |byte| StackItem::new(&target, vec![byte; 32]).unwrap();
    let mut result = BTreeMap::from([
        (S::StateAsset, item(0x11)),
        (S::StateAmount, StackItem::signed_le64(&target, 1)),
        (
            S::FeeSponsorInputMax,
            StackItem::script_number(&target, 3).unwrap(),
        ),
    ]);
    if shape.sponsor_inputs() > 0 || shape.has_change() || shape.has_fee() {
        result.insert(S::ReserveAsset, item(0x22));
    }
    if shape.has_change() {
        result.extend([
            (
                S::SponsorChangeProgram,
                StackItem::new(&target, vec![0x44; 20]).unwrap(),
            ),
            (
                S::SponsorChangeVersion,
                StackItem::script_number(&target, 0).unwrap(),
            ),
        ]);
    }
    if shape.has_fee() {
        result.insert(S::FeeProgramDigest, item(0x55));
    }
    result
}

fn bindings(shape: StateAnnouncementShape) -> StatePatternBindings {
    StatePatternBindings::new(&reviewed_target(), shape, values(shape)).unwrap()
}

fn pattern(id: StatePatternId) -> StatePattern {
    let target = reviewed_target();
    let bindings = bindings(sponsored());
    let fragment = state_structural_fragment(&target, &bindings, id).unwrap();
    build_state_pattern(&target, &bindings, id, fragment).unwrap()
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
    assert!(
        record
            .aborts()
            .contains(&FailureCause::IntrospectionIndexOutOfRange)
    );
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

fn partition_primitives() -> BTreeSet<ElementsCapability> {
    use ElementsCapability as C;
    BTreeSet::from([
        C::InputCountInspection,
        C::OutputCountInspection,
        C::InputAssetInspection,
        C::OutputAssetInspection,
        C::OutputProgramInspection,
        C::ByteStringEquality,
        C::ScriptNumberConversion,
        C::SignedFixedWidthComparison,
        C::BooleanVerification,
    ])
}

#[test]
fn coordinator_contract_owns_input_zero_and_complete_partition() {
    let mut prerequisites = partition_primitives();
    prerequisites.insert(ElementsCapability::CurrentInputIndexInspection);
    assert_contract(
        StatePatternId::StateCoordinatorRoleV1,
        StatePatternOwner::CoordinatorRole,
        &prerequisites,
    );
    let record = pattern(StatePatternId::StateCoordinatorRoleV1);
    assert_eq!(
        opcode_count(record.fragment(), OpcodeId::PushCurrentInputIndex),
        1
    );
    assert_eq!(
        opcode_count(record.fragment(), OpcodeId::InspectInputAsset),
        3
    );
    assert_eq!(
        opcode_count(record.fragment(), OpcodeId::InspectOutputAsset),
        3
    );
}

#[test]
fn cardinality_contract_owns_exact_counts_and_singleton_partition() {
    assert_contract(
        StatePatternId::StateCardinalityV1,
        StatePatternOwner::FamilyCardinality,
        &partition_primitives(),
    );
    let record = pattern(StatePatternId::StateCardinalityV1);
    assert_eq!(
        opcode_count(record.fragment(), OpcodeId::InspectNumInputs),
        1
    );
    assert_eq!(
        opcode_count(record.fragment(), OpcodeId::InspectNumOutputs),
        1
    );
    assert_eq!(
        opcode_count(record.fragment(), OpcodeId::LessThanOrEqual64),
        1
    );
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

#[test]
fn sponsor_contract_has_no_value_primitive() {
    assert_contract(
        StatePatternId::StateSponsorIsolationV1,
        StatePatternOwner::SponsorIsolation,
        &partition_primitives(),
    );
    let record = pattern(StatePatternId::StateSponsorIsolationV1);
    assert!(!reads_a_value_field(record.fragment()));
    assert_eq!(
        opcode_count(record.fragment(), OpcodeId::InspectOutputScriptPubKey),
        2
    );
}

#[test]
fn issuance_contract_checks_every_input_and_names_structural_evidence() {
    let mut prerequisites = partition_primitives();
    prerequisites.insert(ElementsCapability::InputIssuanceInspection);
    assert_contract(
        StatePatternId::StateIssuanceAbsenceV1,
        StatePatternOwner::AbsenceRelations,
        &prerequisites,
    );
    let record = pattern(StatePatternId::StateIssuanceAbsenceV1);
    assert_eq!(
        opcode_count(record.fragment(), OpcodeId::InspectInputIssuance),
        3
    );
    assert_eq!(
        record.metadata().structural(),
        &StateStructuralEvidence::ALL.iter().copied().collect()
    );
    assert!(
        record
            .metadata()
            .evidence()
            .contains(&TargetEvidenceRequirementId::IssuanceIntrospection)
    );
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
    let start = if matches!(
        opcode,
        OpcodeId::InspectNumInputs | OpcodeId::InspectNumOutputs | OpcodeId::PushCurrentInputIndex
    ) {
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

#[test]
fn coordinator_rejects_nonzero_executing_input() {
    let record = pattern(StatePatternId::StateCoordinatorRoleV1);
    let fragment = observed(
        record.fragment(),
        OpcodeId::PushCurrentInputIndex,
        0,
        vec![number(&reviewed_target(), 1).unwrap()],
    );
    assert!(walk(&fragment).always_aborts());
}

#[test]
fn cardinality_rejects_extra_and_missing_positions_on_both_sides() {
    let record = pattern(StatePatternId::StateCardinalityV1);
    for opcode in [OpcodeId::InspectNumInputs, OpcodeId::InspectNumOutputs] {
        for count in [0, 1, 2, 4] {
            let fragment = observed(
                record.fragment(),
                opcode,
                0,
                vec![number(&reviewed_target(), count).unwrap()],
            );
            assert!(walk(&fragment).always_aborts());
        }
    }
}

#[test]
fn cardinality_rejects_a_second_state_in_any_sponsor_position() {
    let record = pattern(StatePatternId::StateCardinalityV1);
    for position in [1, 2] {
        for opcode in [OpcodeId::InspectInputAsset, OpcodeId::InspectOutputAsset] {
            let fragment = observed(record.fragment(), opcode, position, asset(0x11));
            assert!(walk(&fragment).always_aborts());
        }
    }
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

// No instruction of the recipe pushes the consumed program, at any shape.
#[test]
fn no_structural_fragment_pushes_a_predecessor_program_literal() {
    let target = reviewed_target();
    for shape in [sponsored(), plain()] {
        let recipe = state_structural_patterns(&target, &bindings(shape)).unwrap();
        let literal = StackItem::new(&target, vec![0x33; 32]).unwrap();
        for record in recipe.components() {
            assert!(
                !record
                    .fragment()
                    .instructions()
                    .contains(&TapscriptInstruction::Push(literal.clone()))
            );
        }
        assert_eq!(
            recipe.consumers().keys().copied().collect::<BTreeSet<_>>(),
            values(shape).keys().copied().collect()
        );
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

#[test]
fn sponsor_rejects_wrong_reserve_asset_at_every_suffix_position() {
    let record = pattern(StatePatternId::StateSponsorIsolationV1);
    for position in [1, 2] {
        let fragment = observed(
            record.fragment(),
            OpcodeId::InspectInputAsset,
            position,
            asset(0x99),
        );
        assert!(walk(&fragment).always_aborts());
    }
}

#[test]
fn sponsor_rejects_wrong_change_and_fee_programs() {
    let record = pattern(StatePatternId::StateSponsorIsolationV1);
    for (occurrence, width, version) in [(0, 20, 0), (1, 32, -1)] {
        let replacement = vec![
            TapscriptInstruction::Push(
                StackItem::new(&reviewed_target(), vec![0x99; width]).unwrap(),
            ),
            number(&reviewed_target(), version).unwrap(),
        ];
        let fragment = observed(
            record.fragment(),
            OpcodeId::InspectOutputScriptPubKey,
            occurrence,
            replacement,
        );
        assert!(walk(&fragment).always_aborts());
    }
}

#[test]
fn issuance_on_the_coordinator_or_any_sponsor_aborts() {
    let record = pattern(StatePatternId::StateIssuanceAbsenceV1);
    for occurrence in 0..3 {
        let issued = vec![
            TapscriptInstruction::Push(
                StackItem::new(&reviewed_target(), vec![0x77; 32]).unwrap()
            );
            6
        ];
        let fragment = observed(
            record.fragment(),
            OpcodeId::InspectInputIssuance,
            occurrence,
            issued,
        );
        assert!(walk(&fragment).always_aborts());
        let absent = observed(
            record.fragment(),
            OpcodeId::InspectInputIssuance,
            occurrence,
            vec![TapscriptInstruction::Push(StackItem::empty())],
        );
        assert_eq!(walk(&absent).success(), record.success());
    }
}

fn refuse(
    id: StatePatternId,
    instructions: Vec<TapscriptInstruction>,
    expected: StatePatternRefusal,
) {
    let result = build_state_pattern(
        &reviewed_target(),
        &bindings(sponsored()),
        id,
        TapscriptProgram::new(instructions).unwrap(),
    );
    assert_eq!(result, Err(expected));
}

#[test]
fn emitter_refuses_sponsor_value_reads_even_when_the_results_are_dropped() {
    for opcode in [OpcodeId::InspectInputValue, OpcodeId::InspectOutputValue] {
        let id = StatePatternId::StateSponsorIsolationV1;
        let mut instructions = pattern(id).fragment().instructions().to_vec();
        instructions.extend([
            number(&reviewed_target(), 1).unwrap(),
            op(opcode),
            op(OpcodeId::DropTwo),
        ]);
        refuse(id, instructions, StatePatternRefusal::SponsorValueRead);
    }
}

#[test]
fn omitted_issuance_checks_cannot_inherit_the_record() {
    let id = StatePatternId::StateIssuanceAbsenceV1;
    let record = pattern(id);
    for occurrence in 0..3 {
        let mut instructions = record.fragment().instructions().to_vec();
        let index = instructions
            .iter()
            .enumerate()
            .filter(|(_, instruction)| **instruction == op(OpcodeId::InspectInputIssuance))
            .nth(occurrence)
            .map(|(index, _)| index)
            .unwrap();
        instructions.drain(index - 1..index + 3);
        refuse(id, instructions, StatePatternRefusal::FragmentMismatch);
    }
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
    for &id in StatePatternId::ALL {
        let mut instructions = pattern(id).fragment().instructions().to_vec();
        let index = instructions.iter().position(|instruction| matches!(instruction, TapscriptInstruction::Push(item) if item.bytes() == [0x11; 32])).unwrap();
        instructions[index] =
            TapscriptInstruction::Push(StackItem::new(&reviewed_target(), vec![0x99; 32]).unwrap());
        refuse(id, instructions, StatePatternRefusal::FragmentMismatch);
    }
}

#[test]
fn recipe_metadata_is_the_component_union_in_all_five_dimensions() {
    let recipe = state_structural_patterns(&reviewed_target(), &bindings(sponsored())).unwrap();
    let components = recipe.components();
    assert_eq!(components.len(), 5);
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
    assert!(
        recipe
            .metadata()
            .disclosure()
            .contains(&StateDisclosure::PredecessorCommitment)
    );
    assert!(
        recipe
            .metadata()
            .disclosure()
            .contains(&StateDisclosure::IssuanceAbsence)
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
    let recipe = state_structural_patterns(&target, &bindings(sponsored())).unwrap();
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
    assert!(
        !record
            .metadata()
            .structural()
            .contains(&StateStructuralEvidence::NoSecondState)
    );
}

#[test]
fn identity_and_owner_censuses_are_deterministic_and_distinct() {
    assert_eq!(StatePatternId::ALL.len(), 5);
    assert_eq!(StatePatternOwner::ALL.len(), 5);
    assert_eq!(
        StatePatternId::ALL
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len(),
        5
    );
    assert!(StatePatternId::ALL.windows(2).all(|pair| pair[0] < pair[1]));
    let owners: BTreeSet<_> = StatePatternId::ALL
        .iter()
        .map(|&id| pattern(id).owner())
        .collect();
    assert_eq!(owners, StatePatternOwner::ALL.iter().copied().collect());
    let first = state_structural_patterns(&reviewed_target(), &bindings(sponsored())).unwrap();
    let second = state_structural_patterns(&reviewed_target(), &bindings(sponsored())).unwrap();
    assert_eq!(first, second);
}

#[test]
fn consumers_are_unique_nonempty_exact_push_sites_and_still_unresolved() {
    let recipe = state_structural_patterns(&reviewed_target(), &bindings(sponsored())).unwrap();
    let substitutions = values(sponsored());
    assert_eq!(recipe.consumers().len(), 7);
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
}

#[test]
fn consumer_sites_union_every_component_site_without_inventing_a_consumer() {
    let recipe = state_structural_patterns(&reviewed_target(), &bindings(sponsored())).unwrap();
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
fn symbol_requirements_do_not_depend_on_fixture_byte_uniqueness() {
    let shape = sponsored();
    let mut substitutions = values(shape);
    substitutions.insert(
        StatePatternSymbol::FeeProgramDigest,
        substitutions[&StatePatternSymbol::StateAsset].clone(),
    );
    let supplied = StatePatternBindings::new(&reviewed_target(), shape, substitutions).unwrap();
    let recipe = state_structural_patterns(&reviewed_target(), &supplied).unwrap();
    assert_eq!(recipe.consumers().len(), 7);
    assert_ne!(
        recipe.consumers()[&StatePatternSymbol::StateAsset].sites,
        recipe.consumers()[&StatePatternSymbol::FeeProgramDigest].sites
    );
}

#[test]
fn sponsorless_recipe_omits_every_unused_sponsor_consumer() {
    let recipe = state_structural_patterns(&reviewed_target(), &bindings(plain())).unwrap();
    assert_eq!(
        recipe.consumers().keys().copied().collect::<BTreeSet<_>>(),
        BTreeSet::from([
            StatePatternSymbol::StateAsset,
            StatePatternSymbol::StateAmount,
            StatePatternSymbol::FeeSponsorInputMax,
        ])
    );
    assert!(
        !recipe
            .metadata()
            .disclosure()
            .contains(&StateDisclosure::SponsorPrograms)
    );
    assert!(
        recipe
            .metadata()
            .evidence()
            .contains(&TargetEvidenceRequirementId::FeelessTransactionAdmission)
    );
    assert!(
        !recipe
            .metadata()
            .evidence()
            .contains(&TargetEvidenceRequirementId::FeeOutputForm)
    );
}

#[test]
fn optional_change_and_fee_are_independent_roles_with_exact_counts() {
    for sponsors in [0, 1, 3] {
        for change in [
            SponsorChangePresence::Absent,
            SponsorChangePresence::Present,
        ] {
            for fee in [FeePresence::Absent, FeePresence::Present] {
                let shape = StateAnnouncementShape::new(sponsors, change, fee);
                let recipe =
                    state_structural_patterns(&reviewed_target(), &bindings(shape)).unwrap();
                assert_eq!(shape.inputs(), u16::from(sponsors) + 1);
                assert_eq!(
                    shape.outputs(),
                    1 + u16::from(shape.has_change()) + u16::from(shape.has_fee())
                );
                let absence = recipe.components().last().unwrap();
                assert_eq!(
                    opcode_count(absence.fragment(), OpcodeId::InspectInputIssuance),
                    usize::from(shape.inputs())
                );
                assert_eq!(
                    opcode_count(absence.fragment(), OpcodeId::InspectOutputAsset),
                    usize::from(shape.outputs())
                );
                assert_eq!(
                    recipe
                        .consumers()
                        .contains_key(&StatePatternSymbol::SponsorChangeProgram),
                    shape.has_change()
                );
                assert_eq!(
                    recipe
                        .consumers()
                        .contains_key(&StatePatternSymbol::FeeProgramDigest),
                    shape.has_fee()
                );
            }
        }
    }
}

#[test]
fn missing_and_unused_bindings_are_refused() {
    let target = reviewed_target();
    for &symbol in StatePatternSymbol::ALL {
        let mut substitutions = values(sponsored());
        substitutions.remove(&symbol);
        assert_eq!(
            StatePatternBindings::new(&target, sponsored(), substitutions),
            Err(StatePatternRefusal::ConsumerCensus)
        );
    }
    let mut substitutions = values(plain());
    substitutions.insert(
        StatePatternSymbol::ReserveAsset,
        StackItem::new(&target, vec![0x22; 32]).unwrap(),
    );
    assert_eq!(
        StatePatternBindings::new(&target, plain(), substitutions),
        Err(StatePatternRefusal::ConsumerCensus)
    );
}

#[test]
fn overlapping_assets_and_exceeded_fee_bounds_are_refused() {
    let target = reviewed_target();
    let shape = sponsored();
    let mut substitutions = values(shape);
    substitutions.insert(
        StatePatternSymbol::ReserveAsset,
        substitutions[&StatePatternSymbol::StateAsset].clone(),
    );
    assert_eq!(
        StatePatternBindings::new(&target, shape, substitutions),
        Err(StatePatternRefusal::AssetFamiliesOverlap)
    );
    let mut substitutions = values(shape);
    substitutions.insert(
        StatePatternSymbol::FeeSponsorInputMax,
        StackItem::script_number(&target, 1).unwrap(),
    );
    assert_eq!(
        StatePatternBindings::new(&target, shape, substitutions),
        Err(StatePatternRefusal::InvalidBinding(
            StatePatternSymbol::FeeSponsorInputMax
        ))
    );
}

#[test]
fn malformed_symbol_domains_are_refused() {
    let target = reviewed_target();
    for &symbol in StatePatternSymbol::ALL {
        let mut substitutions = values(sponsored());
        substitutions.insert(symbol, StackItem::new(&target, vec![0xff; 7]).unwrap());
        if symbol == StatePatternSymbol::SponsorChangeProgram {
            substitutions.insert(symbol, StackItem::empty());
        }
        assert_eq!(
            StatePatternBindings::new(&target, sponsored(), substitutions),
            Err(StatePatternRefusal::InvalidBinding(symbol))
        );
    }
}

#[test]
fn fee_bound_encoding_does_not_override_the_abstract_work_limit() {
    let target = reviewed_target();
    for sponsors in [0, 3, 255] {
        let shape = StateAnnouncementShape::new(
            sponsors,
            SponsorChangePresence::Absent,
            FeePresence::Absent,
        );
        let mut substitutions = values(shape);
        substitutions.insert(
            StatePatternSymbol::FeeSponsorInputMax,
            StackItem::script_number(&target, i64::from(sponsors)).unwrap(),
        );
        let supplied = StatePatternBindings::new(&target, shape, substitutions).unwrap();
        assert_eq!(supplied.shape(), shape);
        assert_eq!(shape.inputs(), 1 + u16::from(sponsors));
        let result = state_structural_patterns(&target, &supplied);
        if sponsors == 255 {
            assert_eq!(
                result,
                Err(StatePatternRefusal::Program(
                    TapscriptError::AbstractStateLimitExceeded { maximum: 4096 },
                ))
            );
        } else {
            assert_eq!(result.unwrap().components().len(), 5);
        }
    }
}

#[test]
fn recipes_refuse_missing_duplicate_reordered_and_incompatible_components() {
    let full = state_structural_patterns(&reviewed_target(), &bindings(sponsored())).unwrap();
    let mut missing = full.components().clone();
    missing.pop();
    assert_eq!(
        StatePatternRecipe::new(missing),
        Err(StatePatternRefusal::ComponentRecipe)
    );
    let mut duplicate = full.components().clone();
    duplicate[1] = duplicate[0].clone();
    assert_eq!(
        StatePatternRecipe::new(duplicate),
        Err(StatePatternRefusal::ComponentRecipe)
    );
    let mut reversed = full.components().clone();
    reversed.reverse();
    assert_eq!(
        StatePatternRecipe::new(reversed),
        Err(StatePatternRefusal::ComponentRecipe)
    );
    let mut mixed = full.components().clone();
    let other = state_structural_patterns(&reviewed_target(), &bindings(plain())).unwrap();
    mixed[0] = other.components()[0].clone();
    assert_eq!(
        StatePatternRecipe::new(mixed),
        Err(StatePatternRefusal::ComponentRecipe)
    );
}

#[test]
fn boolean_producers_are_immediately_verified_and_equal_uses_the_verifying_form() {
    for &id in StatePatternId::ALL {
        let record = pattern(id);
        assert_eq!(opcode_count(record.fragment(), OpcodeId::Equal), 0);
        for pair in record.fragment().instructions().windows(2) {
            if pair[0] == op(OpcodeId::LessThanOrEqual64) {
                assert_eq!(pair[1], op(OpcodeId::Verify));
            }
        }
    }
}

#[test]
fn resources_grow_with_authenticated_sponsor_positions() {
    let target = reviewed_target();
    let small = state_structural_patterns(&target, &bindings(plain())).unwrap();
    let large = state_structural_patterns(&target, &bindings(sponsored())).unwrap();
    for (small_record, large_record) in small.components().iter().zip(large.components()) {
        if small_record.id() == StatePatternId::StateInputRecognitionV1 {
            assert_eq!(small_record.resources(), large_record.resources());
        } else {
            assert!(
                small_record.fragment().instructions().len()
                    < large_record.fragment().instructions().len()
            );
            let growth: Vec<ResourceDimension> = small_record
                .resources()
                .iter()
                .filter_map(|(&dimension, &cost)| {
                    (large_record
                        .resources()
                        .get(&dimension)
                        .copied()
                        .unwrap_or_default()
                        > cost)
                        .then_some(dimension)
                })
                .collect();
            assert_ne!(growth, []);
        }
    }
}

#[test]
fn external_root_role_is_not_reported_as_structural_or_opcode_evidence() {
    let record = pattern(StatePatternId::StateInputRecognitionV1);
    assert_eq!(
        record.metadata().external(),
        &BTreeSet::from([StateExternalEvidenceRole::CurrentStateRootFreshness,])
    );
    let recipe = state_structural_patterns(&reviewed_target(), &bindings(sponsored())).unwrap();
    let expected = recipe
        .components()
        .iter()
        .flat_map(|component| component.metadata().external().iter().copied())
        .collect();
    assert_eq!(recipe.metadata().external(), &expected);
}

#[test]
fn reserve_asset_equality_does_not_discharge_family_role_authentication() {
    let record = pattern(StatePatternId::StateIssuanceAbsenceV1);
    let reserve = observed(
        record.fragment(),
        OpcodeId::InspectInputAsset,
        1,
        asset(0x22),
    );
    assert_eq!(walk(&reserve).success(), record.success());
    assert!(
        record
            .metadata()
            .sources()
            .contains(&RequiredSourceKind::AuthenticatedFamilyCensus)
    );
    assert!(
        record
            .metadata()
            .residuals()
            .contains(&StatePatternResidual::AuthenticatedFamilyRoles)
    );
    assert!(
        record
            .metadata()
            .residuals()
            .contains(&StatePatternResidual::SuccessorConstructorAuthentication)
    );
}

#[test]
fn every_structural_fragment_preserves_extra_witness_as_forbidden_residue() {
    for shape in [plain(), sponsored()] {
        let recipe = state_structural_patterns(&reviewed_target(), &bindings(shape)).unwrap();
        for record in recipe.components() {
            assert_eq!(record.precondition().main(), []);
            for width in [0, 1, 31, 32, 33, 85, 86, 87] {
                super::state_program_tests::assert_witness_changes_success(
                    record.fragment(),
                    vec![target_elements::StackValueType::Bytes {
                        minimum: width,
                        maximum: width,
                    }],
                    record.success(),
                );
            }
        }
    }
}

#[test]
fn removing_comparison_verify_leaves_boolean_residue_and_refuses_structural_identity() {
    let target = reviewed_target();
    for &id in StatePatternId::ALL {
        let record = pattern(id);
        let Some(index) = record
            .fragment()
            .instructions()
            .iter()
            .position(|instruction| *instruction == op(OpcodeId::LessThanOrEqual64))
        else {
            continue;
        };
        let mut instructions = record.fragment().instructions().to_vec();
        assert_eq!(instructions.remove(index + 1), op(OpcodeId::Verify));
        let changed = TapscriptProgram::new(instructions).unwrap();
        let result = walk(&changed);
        assert_eq!(
            result.success(),
            &BTreeSet::from([AbstractStackState::from_main(vec![
                target_elements::StackValueType::Bool
            ])])
        );
        assert_eq!(
            build_state_pattern(&target, &bindings(sponsored()), id, changed),
            Err(StatePatternRefusal::FragmentMismatch)
        );
    }
}

#[test]
fn structural_refusal_declaration_has_exact_exercised_and_unreachable_census() {
    super::state_program_tests::assert_refusal_census(
        include_str!("../state_pattern.rs"),
        "StatePatternRefusal",
        &[
            (
                "Program",
                fee_bound_encoding_does_not_override_the_abstract_work_limit,
            ),
            ("ConsumerCensus", missing_and_unused_bindings_are_refused),
            ("InvalidBinding", malformed_symbol_domains_are_refused),
            (
                "AssetFamiliesOverlap",
                overlapping_assets_and_exceeded_fee_bounds_are_refused,
            ),
            (
                "SponsorValueRead",
                emitter_refuses_sponsor_value_reads_even_when_the_results_are_dropped,
            ),
            (
                "FragmentMismatch",
                omitted_issuance_checks_cannot_inherit_the_record,
            ),
            (
                "ComponentRecipe",
                recipes_refuse_missing_duplicate_reordered_and_incompatible_components,
            ),
        ],
        &[(
            "InvalidContract",
            "Exact canonical emission precedes the walk; needs an internal contract checker accepting a walked mutation.",
        )],
    );
}

fn composed_for_shape(shape: StateAnnouncementShape) -> crate::StateAnnouncementProgram {
    let target = reviewed_target();
    let structural = state_structural_patterns(&target, &bindings(shape)).unwrap();
    let fixture = super::state_program_tests::fixtures();
    let program = crate::state_announcement_program(
        &target,
        &structural,
        &fixture.semantic,
        &fixture.operator,
    )
    .unwrap();
    crate::build_state_announcement_program(
        &target,
        &structural,
        &fixture.semantic,
        &fixture.operator,
        program,
    )
    .unwrap()
}

fn assert_value_positions(program: &TapscriptProgram) {
    let target = reviewed_target();
    let mut count = 0;
    for (index, instruction) in program.instructions().iter().enumerate() {
        if matches!(
            instruction,
            TapscriptInstruction::Opcode(
                OpcodeId::InspectInputValue | OpcodeId::InspectOutputValue
            )
        ) {
            let Some(TapscriptInstruction::Push(position)) = index
                .checked_sub(1)
                .and_then(|index| program.instructions().get(index))
            else {
                panic!("value read without a literal position")
            };
            assert_eq!(
                position.script_number_value(&target),
                Some(0),
                "value inspection at instruction {index}"
            );
            count += 1;
        }
    }
    assert!(count <= 2);
}

#[test]
fn value_inspection_census_reads_only_state_zero_in_both_transaction_shapes() {
    for shape in [plain(), sponsored()] {
        let recipe = state_structural_patterns(&reviewed_target(), &bindings(shape)).unwrap();
        for record in recipe.components() {
            assert_value_positions(record.fragment());
            assert_eq!(
                opcode_count(record.fragment(), OpcodeId::InspectInputValue),
                usize::from(record.id() == StatePatternId::StateInputRecognitionV1)
            );
            assert_eq!(
                opcode_count(record.fragment(), OpcodeId::InspectOutputValue),
                0
            );
        }
        let composed = composed_for_shape(shape);
        assert_value_positions(composed.program());
        assert_eq!(
            opcode_count(composed.program(), OpcodeId::InspectInputValue),
            1
        );
        assert_eq!(
            opcode_count(composed.program(), OpcodeId::InspectOutputValue),
            1
        );
    }
}

#[test]
fn issuance_at_each_input_aborts_both_shapes_and_their_composed_programs() {
    let target = reviewed_target();
    for shape in [plain(), sponsored()] {
        let recipe = state_structural_patterns(&target, &bindings(shape)).unwrap();
        let absence = recipe
            .components()
            .iter()
            .find(|record| record.id() == StatePatternId::StateIssuanceAbsenceV1)
            .unwrap();
        let composed = composed_for_shape(shape);
        assert_eq!(
            absence.metadata().structural(),
            &StateStructuralEvidence::ALL.iter().copied().collect()
        );
        assert_eq!(
            &composed.metadata().structural,
            absence.metadata().structural()
        );
        for position in 0..usize::from(shape.inputs()) {
            let issued =
                vec![
                    TapscriptInstruction::Push(StackItem::new(&target, vec![0x77; 32]).unwrap());
                    6
                ];
            let changed = observed(
                composed.program(),
                OpcodeId::InspectInputIssuance,
                position,
                issued.clone(),
            );
            let result = validate_program(
                &target,
                &changed,
                composed.precondition(),
                AbstractLimits::for_target(&target),
            )
            .unwrap();
            assert!(result.always_aborts(), "input {position} in {shape:?}");
            if shape == plain() {
                let changed = observed(
                    absence.fragment(),
                    OpcodeId::InspectInputIssuance,
                    position,
                    issued,
                );
                assert!(walk(&changed).always_aborts());
            }
            let absent = observed(
                composed.program(),
                OpcodeId::InspectInputIssuance,
                position,
                vec![TapscriptInstruction::Push(StackItem::empty())],
            );
            let result = validate_program(
                &target,
                &absent,
                composed.precondition(),
                AbstractLimits::for_target(&target),
            )
            .unwrap();
            assert_eq!(result.success(), composed.execution().success());
        }
    }
}
