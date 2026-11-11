use super::*;
use crate::keyed_bundle::{KeyedBundleSelection, emit_keyed_bundle};
use architecture::{
    keyed::KeyedManifest,
    semantic::{KeyedOperationId as K, SemanticOperationId},
};
use compiler::{
    AnalysisPolicy, ProofSearchLimits, bind_input,
    keyed_plan::{KeyedPlan, PlacementSearchLimits, plan},
};
fn fixture(scope: Vec<SemanticOperationId>) -> KeyedPlan {
    let manifest = KeyedManifest::contract().expect("manifest");
    let source = realization::keyed::KeyedRealizationDraft::derive(&manifest)
        .expect("derive")
        .validate(&manifest)
        .expect("validate");
    let states = std::num::NonZeroU64::new(1_000_000).expect("positive");
    let candidates = std::num::NonZeroU64::new(100_000).expect("positive");
    let input = bind_input(
        &manifest,
        source,
        scope,
        AnalysisPolicy::strict(ProofSearchLimits::new(states, candidates)),
    )
    .expect("bind");
    plan(&input, PlacementSearchLimits::new(states, candidates)).expect("plan")
}

pub fn context() -> (
    KeyedEmissionBundle,
    Vec<KeyedInputContext>,
    Vec<KeyedMemberBinding>,
) {
    let target = target_elements::reviewed_elements_tapscript().expect("target");
    let plan = fixture(vec![
        SemanticOperationId::Keyed(K::Source),
        SemanticOperationId::Keyed(K::Merge),
    ]);
    let selection = KeyedBundleSelection {
        binding: plan.project().binding.clone(),
        scope: plan.project().scope.clone(),
        proof: 0,
        placements: plan.project().plans[0]
            .operations
            .keys()
            .map(|id| (*id, 0))
            .collect(),
    };
    let recipe = KeyedLeafRecipe::new([42; 32]);
    let bundle = emit_keyed_bundle(&target, &plan, selection, recipe).expect("bundle");
    let inputs = (0u32..3)
        .map(|i| KeyedInputContext {
            input_index: i,
            outpoint: InputOutpoint {
                txid: [u8::try_from(i).expect("small"); 32],
                output_index: 0,
            },
            role: if i == 0 {
                KeyedInputRole::Sponsor
            } else {
                KeyedInputRole::ConsumedHandle
            },
            predecessor_program: vec![0x51, 32, 42],
        })
        .collect::<Vec<_>>();
    let members = inputs
        .iter()
        .filter(|i| i.role == KeyedInputRole::ConsumedHandle)
        .map(|i| KeyedMemberBinding {
            input_index: i.input_index,
            outpoint: i.outpoint,
            recipe,
            offline_assertion: OfflinePredecessorAssertion {
                outpoint: i.outpoint,
                predecessor_program: i.predecessor_program.clone(),
                recipe,
            },
            relations: bundle.project().fragments.keys().copied().collect(),
        })
        .collect();
    (bundle, inputs, members)
}

#[test]
fn supplied_roles_cover_both_clauses_at_nonzero_indices_and_permute() {
    let (bundle, mut inputs, mut members) = context();
    let control = bind_keyed_members(&bundle, &inputs, &members).expect("control");
    assert_eq!(control.coverage().len(), 2);
    for row in control.coverage().values() {
        assert_eq!(
            row.iter().map(|m| m.input_index).collect::<Vec<_>>(),
            [1, 2]
        );
    }
    assert_eq!(control.evidence(), &bundle.project().evidence);
    inputs.reverse();
    members.reverse();
    for member in &mut members {
        member.relations.reverse();
    }
    assert_eq!(bind_keyed_members(&bundle, &inputs, &members), Ok(control));
}

#[test]
fn supplied_member_census_refuses_each_single_coordinate_defect() {
    use KeyedMemberRefusal as E;
    let (bundle, inputs, members) = context();
    let check = |subject: Vec<KeyedMemberBinding>, expected| {
        assert!(bind_keyed_members(&bundle, &inputs, &members).is_ok());
        assert_eq!(
            bind_keyed_members(&bundle, &inputs, &subject),
            Err(expected)
        );
    };
    check(members[..1].to_vec(), E::MissingMember(2));
    let mut duplicate = members.clone();
    duplicate.push(members[1].clone());
    check(duplicate, E::DuplicateMember(2));
    let mut omitted = members.clone();
    let relation = omitted[1].relations.remove(0);
    check(
        omitted,
        E::MissingRelation {
            input_index: 2,
            relation,
        },
    );
    let mut repeated = members.clone();
    repeated[1].relations.push(relation);
    check(
        repeated,
        E::DuplicateRelation {
            input_index: 2,
            relation,
        },
    );
    let mut stale = members.clone();
    stale[1].input_index = 3;
    check(stale, E::UnknownInput(3));
    let mut wrong = members.clone();
    wrong[1].outpoint = members[0].outpoint;
    check(wrong, E::WrongMember(2));
    let mut sponsor = members.clone();
    sponsor[1].input_index = 0;
    check(sponsor, E::NotConsumedHandle(0));
    let mut offline = members.clone();
    offline[1].offline_assertion.predecessor_program.push(0);
    check(offline, E::OfflineSubjectMismatch(2));
    let mut duplicate = inputs.clone();
    duplicate.push(inputs[1].clone());
    assert_eq!(
        bind_keyed_members(&bundle, &duplicate, &members),
        Err(E::DuplicateInput(1))
    );
    let mut missing = inputs.clone();
    missing.remove(1);
    assert_eq!(
        bind_keyed_members(&bundle, &missing, &members),
        Err(E::MissingInputIndex(1))
    );
}

#[test]
fn keyed_assessment_requires_only_its_primitive_census_and_retains_obligations() {
    use target_elements::ElementsCapability as C;
    let (bundle, _, _) = context();
    let target = target_elements::reviewed_elements_tapscript().expect("target");
    let assessment =
        crate::keyed_assessment::assess_keyed_bundle(&target, &bundle).expect("reviewed");
    assert_eq!(
        assessment
            .capabilities()
            .keys()
            .copied()
            .collect::<BTreeSet<_>>(),
        [
            C::TapscriptExecution,
            C::RequiredLeafVersion,
            C::CurrentInputIndexInspection,
            C::InputOutpointInspection,
            C::InputProgramInspection,
            C::OutputProgramInspection,
            C::StackRearrangement,
            C::ByteStringEquality,
        ]
        .into_iter()
        .collect()
    );
    assert_eq!(*assessment.recipe(), *bundle.project().successor.recipe());
    assert_eq!(*assessment.evidence(), bundle.project().evidence);
    assert_eq!(assessment.resources().primitive_count, 12);
    assert!(assessment.resources().dimensions.values().any(|v| matches!(
        v,
        crate::keyed_resources::KeyedResourceTerm::DeploymentRequired
    )));
}

#[test]
fn missing_member_refusal_is_permutation_stable() {
    let (bundle, mut inputs, members) = context();
    assert!(bind_keyed_members(&bundle, &inputs, &members).is_ok());
    assert_eq!(
        bind_keyed_members(&bundle, &inputs, &[]),
        Err(KeyedMemberRefusal::MissingMember(1))
    );
    inputs.reverse();
    assert_eq!(
        bind_keyed_members(&bundle, &inputs, &[]),
        Err(KeyedMemberRefusal::MissingMember(1))
    );
}

#[test]
fn offline_disjuncts_and_real_reader_substitution_refuse() {
    let (bundle, inputs, members) = context();
    assert!(bind_keyed_members(&bundle, &inputs, &members).is_ok());
    let mut duplicate = inputs.clone();
    duplicate[2].outpoint = duplicate[1].outpoint;
    assert_eq!(
        bind_keyed_members(&bundle, &duplicate, &members),
        Err(KeyedMemberRefusal::DuplicateOutpoint(inputs[1].outpoint))
    );
    let mut outpoint = members.clone();
    outpoint[1].offline_assertion.outpoint.output_index = 1;
    assert_eq!(
        bind_keyed_members(&bundle, &inputs, &outpoint),
        Err(KeyedMemberRefusal::OfflineSubjectMismatch(2))
    );
    let mut recipe = members.clone();
    recipe[1].offline_assertion.recipe = KeyedLeafRecipe::new([43; 32]);
    assert_eq!(
        bind_keyed_members(&bundle, &inputs, &recipe),
        Err(KeyedMemberRefusal::OfflineSubjectMismatch(2))
    );
    let reader = bundle
        .project()
        .selected
        .operations
        .values()
        .flat_map(|operation| operation.relations.keys())
        .find(|id| id.kind == realization::relation::KeyedRelationKind::MembershipReading)
        .copied()
        .expect("real reader");
    let mut subject = members;
    subject[1].relations.push(reader);
    assert_eq!(
        bind_keyed_members(&bundle, &inputs, &subject),
        Err(KeyedMemberRefusal::UnexpectedRelation {
            input_index: 2,
            relation: reader
        })
    );
}

#[test]
fn source_accepts_sponsors_only_and_merge_requires_a_consumed_member() {
    let (merge, mut inputs, members) = context();
    assert!(bind_keyed_members(&merge, &inputs, &members).is_ok());
    for input in &mut inputs {
        input.role = KeyedInputRole::Sponsor;
        input.predecessor_program.clear();
    }
    assert_eq!(
        bind_keyed_members(&merge, &inputs, &[]),
        Err(KeyedMemberRefusal::NoConsumedMembers)
    );
    assert_eq!(
        bind_keyed_members(&merge, &[], &[]),
        Err(KeyedMemberRefusal::NoConsumedMembers)
    );
    let target = target_elements::reviewed_elements_tapscript().expect("target");
    let plan = fixture(vec![SemanticOperationId::Keyed(K::Source)]);
    let selection = KeyedBundleSelection {
        binding: plan.project().binding.clone(),
        scope: plan.project().scope.clone(),
        proof: 0,
        placements: plan.project().plans[0]
            .operations
            .keys()
            .map(|id| (*id, 0))
            .collect(),
    };
    let source = emit_keyed_bundle(&target, &plan, selection, KeyedLeafRecipe::new([42; 32]))
        .expect("source");
    assert!(bind_keyed_members(&source, &inputs, &[]).is_ok());
    assert!(bind_keyed_members(&source, &[], &[]).is_ok());
    assert_eq!(
        bind_keyed_members(&source, &inputs, &members[..1]),
        Err(KeyedMemberRefusal::NoRuntimeAssignments)
    );
    inputs[1].role = KeyedInputRole::ConsumedHandle;
    assert_eq!(
        bind_keyed_members(&source, &inputs, &[]),
        Err(KeyedMemberRefusal::NoRuntimeAssignments)
    );
}

#[test]
fn receipt_retains_the_successor_recipe_and_normalized_member_subjects() {
    let (bundle, inputs, members) = context();
    let control = bind_keyed_members(&bundle, &inputs, &members).expect("control");
    assert_eq!(
        control.successor_recipe(),
        bundle.project().successor.recipe()
    );
    assert_eq!(
        control.members()[&1].offline_assertion(),
        &members[0].offline_assertion
    );
    assert_eq!(
        control.members()[&1].relations(),
        &members[0].relations.iter().copied().collect()
    );
    let target = target_elements::reviewed_elements_tapscript().expect("target");
    let plan = fixture(vec![
        SemanticOperationId::Keyed(K::Source),
        SemanticOperationId::Keyed(K::Merge),
    ]);
    let other = emit_keyed_bundle(
        &target,
        &plan,
        bundle.project().selection.clone(),
        KeyedLeafRecipe::new([43; 32]),
    )
    .expect("other recipe");
    let subject =
        bind_keyed_members(&other, &inputs, &members).expect("binding adds no successor equality");
    assert_eq!(control.selection(), subject.selection());
    assert_ne!(control, subject);
}
