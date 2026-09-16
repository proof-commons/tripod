//! Composed recipe contracts and shared checked fixture substitutions.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use sha2::{Digest, Sha256};
use target_elements::{
    EncodingClass, FailureCause, LeafVersion, OpcodeId, ResourceDimension, StackValueType,
};

use super::reviewed_target;
use crate::live_shape::FeePresence;
use crate::pattern::{fragment_prerequisites, number, op};
use crate::shape::SponsorChangePresence;
use crate::state_announcement::*;
use crate::state_constructor::{STATE_NUMS_KEY, StateLeafRole, StateStaticSubtree};
use crate::state_operator::*;
use crate::state_pattern::*;
use crate::state_program::*;
use crate::{
    AbstractLimits, AbstractStackState, SignatureSuccessForm, StackItem, TapscriptInstruction,
    TapscriptProgram, resource_projection, validate_program,
};

pub(super) struct Fixtures {
    pub(super) structural: StatePatternRecipe,
    pub(super) semantic: StateAnnouncementRecipe,
    pub(super) operator: StateOperatorPattern,
    pub(super) program: StateAnnouncementProgram,
}

fn structural_values() -> BTreeMap<StatePatternSymbol, StackItem> {
    use StatePatternSymbol as S;
    let target = reviewed_target();
    let item = |byte| StackItem::new(&target, vec![byte; 32]).unwrap();
    BTreeMap::from([
        (S::StateAsset, item(0x11)),
        (S::StateAmount, StackItem::signed_le64(&target, 1)),
        (S::PredecessorProgram, item(0x33)),
        (
            S::FeeSponsorInputMax,
            StackItem::script_number(&target, 3).unwrap(),
        ),
        (S::ReserveAsset, item(0x22)),
        (
            S::SponsorChangeProgram,
            StackItem::new(&target, vec![0x44; 20]).unwrap(),
        ),
        (
            S::SponsorChangeVersion,
            StackItem::script_number(&target, 0).unwrap(),
        ),
        (S::FeeProgramDigest, item(0x55)),
    ])
}

fn semantic_values() -> BTreeMap<StateAnnouncementSymbol, StackItem> {
    use StateAnnouncementSymbol as S;
    let target = reviewed_target();
    let item = |bytes| StackItem::new(&target, bytes).unwrap();
    BTreeMap::from([
        (S::InternalKey, item(STATE_NUMS_KEY.to_vec())),
        (S::MaturityLeadMin, StackItem::unsigned_le64(&target, 2)),
        (S::MaturityLeadMax, StackItem::unsigned_le64(&target, 4)),
        (S::StateAsset, item(vec![0x11; 32])),
        (S::StateAmount, StackItem::signed_le64(&target, 1)),
    ])
}

pub(super) fn fixtures() -> &'static Fixtures {
    static FIXTURES: OnceLock<Fixtures> = OnceLock::new();
    FIXTURES.get_or_init(|| {
        let target = reviewed_target();
        let bindings = StatePatternBindings::new(
            &target,
            StateAnnouncementShape::new(2, SponsorChangePresence::Present, FeePresence::Present),
            structural_values(),
        )
        .unwrap();
        let structural = state_structural_patterns(&target, &bindings).unwrap();
        let bindings = StateAnnouncementBindings::new(&target, semantic_values()).unwrap();
        let semantic = state_announcement_patterns(&target, &bindings).unwrap();
        let bindings = StateOperatorBindings::new(
            &target,
            &BTreeMap::from([(
                StateOperatorSymbol::CommittedOperatorKey,
                StackItem::encoded(&target, EncodingClass::XOnlyPublicKey, vec![0x33; 32]).unwrap(),
            )]),
        )
        .unwrap();
        let operator = build_state_operator_pattern(
            &target,
            &bindings,
            state_operator_fragment(&bindings).unwrap(),
        )
        .unwrap();
        let raw = state_announcement_program(&target, &structural, &semantic, &operator).unwrap();
        let program =
            build_state_announcement_program(&target, &structural, &semantic, &operator, raw)
                .unwrap();
        Fixtures {
            structural,
            semantic,
            operator,
            program,
        }
    })
}

pub(super) fn production_tree() -> StateStaticSubtree {
    production_static_subtree(&reviewed_target(), &fixtures().program).unwrap()
}

pub(super) fn tagged_hash(tag: &[u8], bytes: &[u8]) -> [u8; 32] {
    let tag = Sha256::digest(tag);
    let mut hash = Sha256::new();
    hash.update(tag);
    hash.update(tag);
    hash.update(bytes);
    hash.finalize().into()
}

pub(super) fn independent_leaf_hash(bytes: &[u8]) -> [u8; 32] {
    let mut preimage = vec![0xc4];
    match bytes.len() {
        0..=252 => preimage.push(u8::try_from(bytes.len()).unwrap()),
        253..=65_535 => {
            preimage.push(0xfd);
            preimage.extend(u16::try_from(bytes.len()).unwrap().to_le_bytes());
        }
        _ => panic!("fixture script exceeds the test CompactSize range"),
    }
    preimage.extend(bytes);
    tagged_hash(b"TapLeaf/elements", &preimage)
}

fn admit(
    instructions: Vec<TapscriptInstruction>,
) -> Result<StateAnnouncementProgram, StateProgramRefusal> {
    let f = fixtures();
    build_state_announcement_program(
        &reviewed_target(),
        &f.structural,
        &f.semantic,
        &f.operator,
        TapscriptProgram::new(instructions).unwrap(),
    )
}

#[test]
fn composed_contract_has_only_canonical_true_and_verified_operator_success() {
    let target = reviewed_target();
    let record = &fixtures().program;
    let walk = validate_program(
        &target,
        record.program(),
        record.precondition(),
        AbstractLimits::for_target(&target),
    )
    .unwrap();
    assert_eq!(record.execution(), &walk);
    assert_eq!(
        walk.success(),
        &BTreeSet::from([AbstractStackState::from_main(vec![StackValueType::Bytes {
            minimum: 1,
            maximum: 1
        }])])
    );
    assert!(walk.nonaborting_failure().is_empty());
    assert_eq!(
        record.program().instructions().last(),
        Some(&number(&target, 1).unwrap())
    );
    assert_eq!(
        walk.signature_forms(),
        &BTreeMap::from([(
            1,
            BTreeSet::from([SignatureSuccessForm::RecognizedKeyVerified])
        )])
    );
    let mut aborts = fixtures().operator.execution().aborts().clone();
    for component in fixtures().structural.components() {
        aborts.extend(component.aborts());
    }
    for component in fixtures().semantic.components() {
        aborts.extend(component.aborts());
    }
    assert_eq!(walk.aborts(), &aborts);
    assert!(walk.aborts().contains(&FailureCause::InvalidSignature));
    assert_eq!(
        record.constructibility(),
        StatePatternConstructibility::CheckedUnresolvedConsumers
    );
}

#[test]
fn witness_uses_component_types_in_declared_deepest_first_order() {
    use StateProgramWitness as W;
    let record = &fixtures().program;
    assert_eq!(
        record
            .witness()
            .iter()
            .map(|(role, _)| *role)
            .collect::<Vec<_>>(),
        vec![
            W::SuccessorOutputKeyPrefix,
            W::SuccessorNonce,
            W::RequestedCycle,
            W::StaticSubtreeRoot,
            W::PredecessorMetadata,
            W::PredecessorOutputKeyPrefix,
            W::OperatorSignature,
        ]
    );
    let mut types: Vec<_> = [1, 4, 8, 32, 86, 1]
        .into_iter()
        .map(|bytes| StackValueType::Bytes {
            minimum: bytes,
            maximum: bytes,
        })
        .collect();
    types.push(StackValueType::Encoded(EncodingClass::SchnorrSignature));
    assert_eq!(record.precondition().main(), types);
    assert_eq!(record.precondition().alternate(), []);
}

#[test]
fn component_ranges_cover_instructions_once_except_the_named_partition_alias() {
    use StateProgramComponent as C;
    let f = fixtures();
    let instructions = f.program.program().instructions();
    let map = f.program.components();
    assert_eq!(map.len(), 15);
    assert_eq!(
        &instructions[map[&C::Operator(f.operator.id())].clone()],
        f.operator.fragment().instructions()
    );
    for component in f.structural.components() {
        assert_eq!(
            &instructions[map[&C::Structural(component.id())].clone()],
            component.fragment().instructions()
        );
    }
    for component in f.semantic.components() {
        assert_eq!(
            &instructions[map[&C::Semantic(*component.id())].clone()],
            component.fragment().instructions()
        );
    }
    let mut covered = BTreeSet::new();
    for (id, range) in map {
        if *id == C::Structural(StatePatternId::StateSponsorIsolationV1) {
            continue;
        }
        for index in range.clone() {
            assert!(covered.insert(index));
        }
    }
    assert_eq!(covered, (0..instructions.len()).collect());
    assert_eq!(
        map[&C::Structural(StatePatternId::StateCardinalityV1)],
        map[&C::Structural(StatePatternId::StateSponsorIsolationV1)]
    );
    for (adapter, expected) in [
        (StateProgramAdapter::LeadWindow, vec![op(OpcodeId::Rotate)]),
        (StateProgramAdapter::CopyThrough, vec![op(OpcodeId::Swap)]),
        (
            StateProgramAdapter::SuccessorReconstruction,
            vec![op(OpcodeId::Rotate)],
        ),
        (
            StateProgramAdapter::FinalTruth,
            vec![op(OpcodeId::Drop), number(&reviewed_target(), 1).unwrap()],
        ),
    ] {
        assert_eq!(instructions[map[&C::Adapter(adapter)].clone()], expected);
    }
}

#[test]
fn metadata_is_exactly_the_family_unions_and_adapter_evidence() {
    let f = fixtures();
    let s = f.structural.metadata();
    let a = f.semantic.metadata();
    let o = &f.operator;
    let actual = f.program.metadata();
    let sources: BTreeSet<_> = s
        .sources()
        .iter()
        .chain(&a.sources)
        .chain(o.sources())
        .copied()
        .collect();
    assert_eq!(actual.sources, sources);
    let mut evidence: BTreeSet<_> = s
        .evidence()
        .iter()
        .chain(&a.evidence)
        .chain(o.evidence())
        .copied()
        .collect();
    let target = reviewed_target();
    for opcode in [OpcodeId::Rotate, OpcodeId::Swap, OpcodeId::Drop] {
        evidence.extend(target.definition().opcodes()[&opcode].evidence());
    }
    assert_eq!(actual.evidence, evidence);
    let disclosure: BTreeSet<_> = s
        .disclosure()
        .iter()
        .copied()
        .map(StateProgramDisclosure::Structural)
        .chain(
            a.disclosure
                .iter()
                .copied()
                .map(StateProgramDisclosure::Semantic),
        )
        .chain(
            o.disclosure()
                .iter()
                .copied()
                .map(StateProgramDisclosure::Operator),
        )
        .collect();
    assert_eq!(actual.disclosure, disclosure);
    let residuals: BTreeSet<_> = s
        .residuals()
        .iter()
        .copied()
        .map(StateProgramResidual::Structural)
        .chain(
            a.residuals
                .iter()
                .copied()
                .map(StateProgramResidual::Semantic),
        )
        .chain(
            o.residuals()
                .iter()
                .copied()
                .map(StateProgramResidual::Operator),
        )
        .collect();
    assert_eq!(actual.residuals, residuals);
    assert_eq!(&actual.structural, s.structural());
    assert_eq!(&actual.external, s.external());
}

#[test]
fn every_consumer_is_the_fixture_push_at_the_reindexed_component_site() {
    use StateProgramComponent as C;
    use StateProgramSymbol as S;
    let f = fixtures();
    let mut expected: BTreeMap<StateProgramSymbol, BTreeSet<usize>> = BTreeMap::new();
    for (&symbol, consumer) in f.structural.consumers() {
        for (&id, sites) in &consumer.sites {
            let start = f.program.components()[&C::Structural(id)].start;
            expected
                .entry(S::Structural(symbol))
                .or_default()
                .extend(sites.iter().map(|site| start + site));
        }
    }
    for (&symbol, consumer) in f.semantic.consumers() {
        let symbol = symbol
            .structural()
            .map_or(S::Semantic(symbol), S::Structural);
        for (&id, sites) in &consumer.sites {
            let start = f.program.components()[&C::Semantic(id)].start;
            expected
                .entry(symbol)
                .or_default()
                .extend(sites.iter().map(|site| start + site));
        }
    }
    for (&symbol, sites) in f.operator.consumers() {
        let start = f.program.components()[&C::Operator(f.operator.id())].start;
        expected
            .entry(S::Operator(symbol))
            .or_default()
            .extend(sites.iter().map(|site| start + site));
    }
    assert_eq!(f.program.consumers().len(), expected.len());
    let structural = structural_values();
    let semantic = semantic_values();
    for (&symbol, consumer) in f.program.consumers() {
        assert_eq!(consumer.sites, expected[&symbol]);
        let item = match symbol {
            S::Structural(symbol) => &structural[&symbol],
            S::Semantic(symbol) => &semantic[&symbol],
            S::Operator(_) => {
                let TapscriptInstruction::Push(item) = &f.operator.fragment().instructions()[0]
                else {
                    panic!("operator fixture must push its key")
                };
                item
            }
        };
        assert_eq!(&consumer.item, item);
        assert!(!consumer.sites.is_empty());
        for &site in &consumer.sites {
            assert_eq!(
                f.program.program().instructions()[site],
                TapscriptInstruction::Push(item.clone())
            );
        }
    }
}

#[test]
fn either_mismatched_shared_binding_is_refused() {
    let f = fixtures();
    let target = reviewed_target();
    for (symbol, replacement) in [
        (
            StateAnnouncementSymbol::StateAsset,
            StackItem::new(&target, vec![0x77; 32]).unwrap(),
        ),
        (
            StateAnnouncementSymbol::StateAmount,
            StackItem::signed_le64(&target, 2),
        ),
    ] {
        let mut values = semantic_values();
        values.insert(symbol, replacement);
        let bindings = StateAnnouncementBindings::new(&target, values).unwrap();
        let semantic = state_announcement_patterns(&target, &bindings).unwrap();
        assert_eq!(
            state_announcement_program(&target, &f.structural, &semantic, &f.operator),
            Err(StateProgramRefusal::ConsumerCensus)
        );
        assert_eq!(
            build_state_announcement_program(
                &target,
                &f.structural,
                &semantic,
                &f.operator,
                f.program.program().clone()
            ),
            Err(StateProgramRefusal::ConsumerCensus)
        );
    }
}

#[test]
fn removing_any_component_or_adapter_cannot_inherit_the_recipe() {
    let f = fixtures();
    for range in f.program.components().values() {
        let mut instructions = f.program.program().instructions().to_vec();
        instructions.drain(range.clone());
        assert_eq!(
            admit(instructions),
            Err(StateProgramRefusal::ComponentRecipe)
        );
    }
}

#[test]
fn moving_any_component_or_adapter_cannot_inherit_the_recipe() {
    let f = fixtures();
    for range in f.program.components().values() {
        let mut instructions = f.program.program().instructions().to_vec();
        let removed: Vec<_> = instructions.drain(range.clone()).collect();
        if range.start == 0 {
            instructions.extend(removed);
        } else {
            instructions.splice(0..0, removed);
        }
        assert_eq!(
            admit(instructions),
            Err(StateProgramRefusal::ComponentRecipe)
        );
    }
}

#[test]
fn missing_or_reordered_family_records_are_refused_before_composition() {
    let f = fixtures();
    for index in 0..f.structural.components().len() {
        let mut components = f.structural.components().clone();
        components.remove(index);
        assert_eq!(
            StatePatternRecipe::new(components),
            Err(StatePatternRefusal::ComponentRecipe)
        );
        let mut components = f.structural.components().clone();
        components.swap(index, (index + 1) % StatePatternId::ALL.len());
        assert_eq!(
            StatePatternRecipe::new(components),
            Err(StatePatternRefusal::ComponentRecipe)
        );
    }
    for index in 0..f.semantic.components().len() {
        let mut components = f.semantic.components().clone();
        components.remove(index);
        assert_eq!(
            StateAnnouncementRecipe::new(components),
            Err(StateAnnouncementRefusal::ComponentRecipe)
        );
        let mut components = f.semantic.components().clone();
        components.swap(index, (index + 1) % StateAnnouncementId::ALL.len());
        assert_eq!(
            StateAnnouncementRecipe::new(components),
            Err(StateAnnouncementRefusal::ComponentRecipe)
        );
    }
}

#[test]
fn projected_resources_follow_target_costs_and_stay_within_each_limit() {
    let target = reviewed_target();
    let record = &fixtures().program;
    assert_eq!(
        record.resources(),
        &resource_projection(&target, record.program())
    );
    assert_eq!(
        record.prerequisites(),
        &fragment_prerequisites(record.program())
    );
    assert_eq!(record.resources().len(), 3);
    for (dimension, charged) in record.resources() {
        // The reviewed target charges no per-script operation budget.
        if *dimension == ResourceDimension::OperationCost {
            assert_eq!(*charged, 0);
        } else {
            assert!(*charged > 0);
        }
        if let Some(maximum) = target
            .definition()
            .resources()
            .consensus()
            .bounds()
            .get(dimension)
            .and_then(|bound| bound.maximum())
        {
            assert!(*charged <= maximum);
        }
    }
}

#[test]
fn production_subtree_is_exactly_the_complete_announcement_leaf() {
    let tree = production_tree();
    let [entry] = tree.leaves() else {
        panic!("exactly one production leaf")
    };
    assert_eq!(entry.identity, 0);
    assert_eq!(entry.leaf.role, StateLeafRole::Announcement);
    assert_eq!(entry.leaf.version, LeafVersion::TAPSCRIPT.get());
    assert_eq!(&entry.leaf.program, fixtures().program.program());
    assert_eq!(entry.siblings, [] as [[u8; 32]; 0]);
    let hash = independent_leaf_hash(&entry.leaf.program.encode(&reviewed_target()));
    assert_eq!(entry.hash, hash);
    assert_eq!(tree.root(), &hash);
}

#[test]
fn deleting_each_instruction_refuses_the_composed_identity() {
    let instructions = fixtures().program.program().instructions();
    for index in 0..instructions.len() {
        let mut changed = instructions.to_vec();
        changed.remove(index);
        assert_eq!(
            admit(changed),
            Err(StateProgramRefusal::ComponentRecipe),
            "instruction {index}"
        );
    }
}

pub(super) fn assert_witness_changes_success(
    program: &TapscriptProgram,
    witness: Vec<StackValueType>,
    success: &BTreeSet<AbstractStackState>,
) {
    let target = reviewed_target();
    if let Ok(walk) = validate_program(
        &target,
        program,
        &AbstractStackState::from_main(witness),
        AbstractLimits::for_target(&target),
    ) {
        assert!(
            walk.success().is_disjoint(success),
            "wrong witness inherited success: {walk:?}"
        );
    }
}

#[test]
fn missing_or_extra_composed_witness_items_never_leave_canonical_success() {
    let record = &fixtures().program;
    for index in 0..record.witness().len() {
        let mut witness = record.precondition().main().to_vec();
        witness.remove(index);
        assert_witness_changes_success(record.program(), witness, record.execution().success());
    }
    for index in 0..=record.witness().len() {
        let mut witness = record.precondition().main().to_vec();
        witness.insert(
            index,
            StackValueType::Bytes {
                minimum: 0,
                maximum: 0,
            },
        );
        assert_witness_changes_success(record.program(), witness, record.execution().success());
    }
}

#[test]
fn closing_adapter_deletion_and_extra_items_break_final_contract() {
    let target = reviewed_target();
    let record = &fixtures().program;
    let closing = record.components()
        [&StateProgramComponent::Adapter(StateProgramAdapter::FinalTruth)]
        .clone();
    let mut absent = record.program().instructions().to_vec();
    absent.drain(closing);
    let mut extra = record.program().instructions().to_vec();
    extra.push(number(&target, 1).unwrap());
    for instructions in [absent, extra] {
        let changed = TapscriptProgram::new(instructions.clone()).unwrap();
        let walk = validate_program(
            &target,
            &changed,
            record.precondition(),
            AbstractLimits::for_target(&target),
        )
        .unwrap();
        assert!(!walk.success().is_empty());
        assert!(walk.success().is_disjoint(record.execution().success()));
        assert_eq!(
            admit(instructions),
            Err(StateProgramRefusal::ComponentRecipe)
        );
    }
}

#[test]
fn alternate_witness_residue_cannot_inherit_the_final_stack() {
    let target = reviewed_target();
    let record = &fixtures().program;
    let witness = AbstractStackState::new(
        record.precondition().main().to_vec(),
        vec![StackValueType::Bool],
    );
    let result = validate_program(
        &target,
        record.program(),
        &witness,
        AbstractLimits::for_target(&target),
    )
    .unwrap();
    assert!(!result.success().is_empty());
    assert!(result.success().is_disjoint(record.execution().success()));
    assert!(
        result
            .success()
            .iter()
            .all(|state| state.alternate() == [StackValueType::Bool])
    );
}

#[test]
fn every_composed_boolean_producer_is_followed_by_verification() {
    let instructions = fixtures().program.program().instructions();
    for (index, instruction) in instructions.iter().enumerate() {
        if matches!(
            instruction,
            TapscriptInstruction::Opcode(
                OpcodeId::Add64
                    | OpcodeId::Sub64
                    | OpcodeId::Mul64
                    | OpcodeId::Div64
                    | OpcodeId::Neg64
                    | OpcodeId::LessThan64
                    | OpcodeId::LessThanOrEqual64
                    | OpcodeId::GreaterThan64
                    | OpcodeId::GreaterThanOrEqual64
            )
        ) {
            assert_eq!(instructions.get(index + 1), Some(&op(OpcodeId::Verify)));
        }
        assert!(!matches!(
            instruction,
            TapscriptInstruction::Opcode(
                OpcodeId::Equal | OpcodeId::CheckSig | OpcodeId::CheckSigFromStack
            )
        ));
    }
}

#[test]
fn deleting_last_comparison_verify_is_refused_and_exposes_a_boolean_at_its_site() {
    let target = reviewed_target();
    let record = &fixtures().program;
    let instructions = record.program().instructions();
    let index = instructions
        .iter()
        .rposition(|item| *item == op(OpcodeId::GreaterThanOrEqual64))
        .unwrap();
    assert_eq!(instructions[index + 1], op(OpcodeId::Verify));
    let prefix = TapscriptProgram::new(instructions[..=index].to_vec()).unwrap();
    let before_verify = validate_program(
        &target,
        &prefix,
        record.precondition(),
        AbstractLimits::for_target(&target),
    )
    .unwrap();
    assert!(!before_verify.success().is_empty());
    assert!(
        before_verify
            .success()
            .iter()
            .all(|state| state.main().last() == Some(&StackValueType::Bool))
    );
    let mut changed = instructions.to_vec();
    changed.remove(index + 1);
    assert_eq!(admit(changed), Err(StateProgramRefusal::ComponentRecipe));
}

#[test]
fn appended_pushes_exceed_stack_bound_but_recipe_guard_precedes_resource_check() {
    let target = reviewed_target();
    let maximum = target.definition().resources().consensus().bounds()
        [&ResourceDimension::PeakStackItems]
        .maximum()
        .unwrap();
    let record = &fixtures().program;
    let mut instructions = record.program().instructions().to_vec();
    // Canonical success already has one item; these pushes reach maximum + 1.
    instructions.extend((0..maximum).map(|_| number(&target, 1).unwrap()));
    let changed = TapscriptProgram::new(instructions.clone()).unwrap();
    assert_eq!(
        validate_program(
            &target,
            &changed,
            record.precondition(),
            AbstractLimits::for_target(&target)
        ),
        Err(crate::TapscriptError::StackLimitExceeded { maximum })
    );
    assert_eq!(
        admit(instructions),
        Err(StateProgramRefusal::ComponentRecipe)
    );
}

// Read declarations instead of maintaining a second enum that could silently drift.
pub(super) fn declared_variants<'a>(source: &'a str, name: &str) -> BTreeSet<&'a str> {
    let marker = format!("pub enum {name} {{");
    let body = source
        .split_once(&marker)
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    body.lines()
        .filter_map(|line| {
            let tail = line.strip_prefix("    ")?;
            if !tail.starts_with(|c: char| c.is_ascii_uppercase()) {
                return None;
            }
            Some(tail.split(['(', '{', ',']).next().unwrap().trim())
        })
        .collect()
}

pub(super) fn assert_refusal_census(
    source: &str,
    name: &str,
    exercised: &[(&str, fn())],
    unavailable: &[(&str, &str)],
) {
    let reached: BTreeSet<_> = exercised
        .iter()
        .map(|(variant, test)| {
            test();
            *variant
        })
        .collect();
    let gaps: BTreeSet<_> = unavailable
        .iter()
        .map(|(variant, reason)| {
            assert_ne!(*reason, "");
            *variant
        })
        .collect();
    assert!(reached.is_disjoint(&gaps));
    assert_eq!(
        reached.union(&gaps).copied().collect::<BTreeSet<_>>(),
        declared_variants(source, name)
    );
    assert_eq!(exercised.len(), reached.len());
    assert_eq!(unavailable.len(), gaps.len());
}

#[test]
fn program_refusal_declaration_has_exact_exercised_and_unreachable_census() {
    assert_refusal_census(
        include_str!("../state_program.rs"),
        "StateProgramRefusal",
        &[
            (
                "ComponentRecipe",
                deleting_each_instruction_refuses_the_composed_identity,
            ),
            (
                "ConsumerCensus",
                either_mismatched_shared_binding_is_refused,
            ),
        ],
        &[
            (
                "InvalidContract",
                "Exact assembly and fixed witness precede the contract check; needs an internal contract validation seam.",
            ),
            (
                "ResourceLimit",
                "All three projected dimensions are unbounded; needs bounded projections and an internal resource validation seam.",
            ),
            (
                "Program",
                "Checked complete recipes supply no failing emission or walk input; needs a fallible assembly test seam.",
            ),
            (
                "Subtree",
                "The fixed single valid announcement leaf supplies no malformed subtree; needs a subtree construction seam.",
            ),
        ],
    );
}
