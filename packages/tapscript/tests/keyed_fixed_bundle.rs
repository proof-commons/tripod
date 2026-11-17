//! Public fixed-form consumers, with record literals independent of emission.
use architecture::keyed_fixed::{KeyedFixedManifest, KeyedFixedOperationId as K};
use compiler::{
    AnalysisPolicy, ProofSearchLimits, bind_input,
    keyed_fixed_plan::{KeyedFixedPlan, PlacementSearchLimits, plan},
};
use realization::keyed_fixed::{KeyedFixedParameters, KeyedFixedValue};
use tapscript::keyed_fixed_bundle::*;

fn fixture(scope: Vec<K>) -> KeyedFixedPlan {
    let manifest = KeyedFixedManifest::contract().unwrap();
    let source = realization::keyed_fixed::KeyedFixedRealizationDraft::derive(&manifest)
        .unwrap()
        .validate(&manifest)
        .unwrap();
    let states = std::num::NonZeroU64::new(1_000_000).unwrap();
    let candidates = std::num::NonZeroU64::new(100_000).unwrap();
    let input = bind_input(
        &manifest,
        source,
        scope,
        AnalysisPolicy::strict(ProofSearchLimits::new(states, candidates)),
    )
    .unwrap();
    plan(&input, PlacementSearchLimits::new(states, candidates)).unwrap()
}

fn selection(plan: &KeyedFixedPlan) -> KeyedFixedBundleSelection {
    KeyedFixedBundleSelection {
        binding: plan.project().binding.clone(),
        scope: plan.project().scope.clone(),
        proof: 0,
        placements: plan.project().plans[0]
            .operations
            .keys()
            .map(|id| (*id, 0))
            .collect(),
    }
}

fn recipe() -> KeyedFixedLeafRecipe {
    KeyedFixedLeafRecipe::new(KeyedFixedParameters {
        key: [0; 32],
        asset: [0x23; 32],
        value: KeyedFixedValue::new(0x0102_0304_0506_0708).unwrap(),
    })
}

#[test]
fn all_scopes_retain_plan_and_literal_schedule() {
    use sha2::{Digest, Sha256};
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    // The record specifies asset payload, prefix, little-endian value, prefix.
    let transcript = [vec![0x23; 32], vec![1, 8, 7, 6, 5, 4, 3, 2, 1, 1]].concat();
    let digest = Sha256::digest(&transcript);
    let expected = [
        vec![32],
        vec![0; 32],
        vec![0x75, 32],
        digest.to_vec(),
        vec![
            0x76, 0, 0xce, 0x7e, 0xc4, 0, 0xcf, 0x7e, 0xc6, 0x88, 0xcd, 0xc8, 0x7e, 0xc4, 0xcd,
            0xc9, 0x7e, 0xc6, 0x88, 0xcd, 0xc7, 0x75, 4, 0, 0, 0, 0, 0x88, 0x75, 0xcd, 0xca, 0,
            0xd1, 0x7b, 0x88, 0x88, 0x51,
        ],
    ]
    .concat();
    assert_eq!(recipe().transcript().as_slice(), transcript);
    assert_eq!(recipe().pair_digest().as_slice(), digest.as_slice());
    for (scope, clauses, evidence) in [
        (vec![K::Source], 0, 4),
        (vec![K::Merge], 3, 3),
        (vec![K::Source, K::Merge], 3, 7),
    ] {
        let plan = fixture(scope);
        let request = selection(&plan);
        let bundle = emit_keyed_fixed_bundle(&target, &plan, request.clone(), recipe()).unwrap();
        let view = bundle.project();
        assert_eq!(view.selection, request);
        assert_eq!(view.selected, plan.project().plans[0]);
        assert_eq!(view.fragments.len(), clauses);
        assert_eq!(view.evidence.len(), evidence);
        assert_eq!(view.successor.program().encode(&target), expected);
        assert_eq!(view.successor.program().instructions().len(), 36);
    }
    let resource = tapscript::keyed_fixed_resources::resources(&target, recipe()).unwrap();
    assert_eq!(
        (
            resource.script_bytes,
            resource.instructions,
            resource.primitives,
            resource.peak_stack,
            resource.maximum_element_bytes
        ),
        (104, 36, 29, 5, 73)
    );
    assert_eq!(
        (
            resource.script_witness_bytes,
            resource.input_witness_bytes,
            resource.validation_budget
        ),
        (140, 143, 190)
    );
    assert_eq!(
        tapscript::keyed_fixed_resources::transaction_weight(&target, 1, 41, 0),
        Ok(307)
    );
    assert!(tapscript::keyed_fixed_resources::transaction_weight(&target, u64::MAX, 0, 0).is_err());
}

#[test]
fn selections_and_member_coverage_refuse_changed_coordinates() {
    use tapscript::keyed_fixed_members::*;
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let plan = fixture(vec![K::Merge]);
    let control = selection(&plan);
    let bundle = emit_keyed_fixed_bundle(&target, &plan, control.clone(), recipe()).unwrap();
    let mut subject = control.clone();
    subject.binding.digest[0] ^= 1;
    assert_eq!(
        emit_keyed_fixed_bundle(&target, &plan, subject, recipe()),
        Err(KeyedFixedBundleRefusal::BindingMismatch)
    );
    let mut subject = control.clone();
    subject.scope.clear();
    assert_eq!(
        emit_keyed_fixed_bundle(&target, &plan, subject, recipe()),
        Err(KeyedFixedBundleRefusal::ScopeMismatch)
    );
    let mut subject = control.clone();
    subject.proof = usize::MAX;
    assert_eq!(
        emit_keyed_fixed_bundle(&target, &plan, subject, recipe()),
        Err(KeyedFixedBundleRefusal::ProofSelection { index: usize::MAX })
    );
    let mut subject = control;
    subject.placements.clear();
    assert_eq!(
        emit_keyed_fixed_bundle(&target, &plan, subject, recipe()),
        Err(KeyedFixedBundleRefusal::PlacementScopeMismatch)
    );
    let outpoint = InputOutpoint {
        txid: [7; 32],
        output_index: 0,
    };
    let input = KeyedFixedInputContext {
        input_index: 0,
        outpoint,
        role: KeyedFixedInputRole::ConsumedHandle,
        predecessor_program: vec![1],
    };
    let member = KeyedFixedMemberBinding {
        input_index: 0,
        outpoint,
        recipe: recipe(),
        offline_assertion: OfflinePredecessorAssertion {
            outpoint,
            predecessor_program: vec![1],
            recipe: recipe(),
        },
        relations: bundle.project().fragments.keys().copied().collect(),
    };
    let receipt = bind_keyed_fixed_members(
        &bundle,
        std::slice::from_ref(&input),
        std::slice::from_ref(&member),
    )
    .unwrap();
    assert_eq!(receipt.coverage().len(), 3);
    let mut missing = member.clone();
    let relation = missing.relations.pop().unwrap();
    assert_eq!(
        bind_keyed_fixed_members(&bundle, std::slice::from_ref(&input), &[missing]),
        Err(KeyedFixedMemberRefusal::MissingRelation {
            input_index: 0,
            relation
        })
    );
    let mut duplicate = member;
    duplicate.relations.push(relation);
    assert_eq!(
        bind_keyed_fixed_members(&bundle, std::slice::from_ref(&input), &[duplicate]),
        Err(KeyedFixedMemberRefusal::DuplicateRelation {
            input_index: 0,
            relation
        })
    );
    assert_eq!(
        bind_keyed_fixed_members(&bundle, &[input], &[]),
        Err(KeyedFixedMemberRefusal::MissingMember(0))
    );
}
