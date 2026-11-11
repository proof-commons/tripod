//! Production bundles with offline authentication and independent model execution.
//! No native observation, serialized-input binding or funding admission is claimed.
use super::*;
use crate::keyed_constructor::{
    KeyedConstructorDescriptor, KeyedConstructorRefusal, PublicKeyedCurve,
    authenticate_keyed_predecessor, construct_keyed,
};
use architecture::{
    keyed::KeyedManifest,
    semantic::{KeyedOperationId as K, SemanticOperationId as O},
};
use compiler::{
    AnalysisPolicy, ProofSearchLimits, bind_input,
    keyed_plan::{PlacementSearchLimits, plan},
};
use realization::relation::{KeyedRelationId, KeyedRelationKind};
use std::collections::BTreeMap;
use tapscript::{keyed_bundle::*, keyed_constructor::KeyedSemanticKey, keyed_members::*};

fn descriptor(key: u8) -> KeyedConstructorDescriptor {
    construct_keyed(
        &target_elements::reviewed_elements_tapscript().expect("target"),
        KeyedSemanticKey::new([key; 32]),
        &PublicKeyedCurve,
    )
    .expect("public construction")
}
fn bundle() -> KeyedEmissionBundle {
    let manifest = KeyedManifest::contract().expect("manifest");
    let realization = realization::keyed::KeyedRealizationDraft::derive(&manifest)
        .expect("derive")
        .validate(&manifest)
        .expect("validate");
    let states = std::num::NonZeroU64::new(1_000_000).expect("positive");
    let candidates = std::num::NonZeroU64::new(100_000).expect("positive");
    let input = bind_input(
        &manifest,
        realization,
        vec![O::Keyed(K::Merge)],
        AnalysisPolicy::strict(ProofSearchLimits::new(states, candidates)),
    )
    .expect("bind");
    let plan = plan(&input, PlacementSearchLimits::new(states, candidates)).expect("plan");
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
    emit_keyed_bundle(
        &target_elements::reviewed_elements_tapscript().expect("target"),
        &plan,
        selection,
        KeyedLeafRecipe::new([42; 32]),
    )
    .expect("bundle")
}
fn program(d: &KeyedConstructorDescriptor) -> ProgramFact {
    // These facts are explicitly supplied from the authenticated public descriptor;
    // the byte model intentionally does not parse transactions or scriptPubKeys.
    ProgramFact::Witness {
        version: 1,
        payload: d.output_key.to_vec(),
    }
}
fn context(d: &KeyedConstructorDescriptor) -> Facts {
    Facts {
        enabled: true,
        current: Some(1),
        inputs: vec![
            InputFact {
                txid: [0; 32],
                index: 7,
                flags: 0,
                program: ProgramFact::Other { sha256: [0; 32] },
            },
            InputFact {
                txid: [1; 32],
                index: 0,
                flags: 0,
                program: program(d),
            },
            InputFact {
                txid: [2; 32],
                index: 0,
                flags: 0,
                program: program(d),
            },
        ],
        outputs: vec![program(d)],
    }
}

fn bind_offline(
    bundle: &KeyedEmissionBundle,
    facts: &Facts,
    descriptors: &BTreeMap<u32, KeyedConstructorDescriptor>,
) -> KeyedMemberReceipt {
    let target = target_elements::reviewed_elements_tapscript().expect("target");
    let mut inputs = Vec::new();
    let mut members = Vec::new();
    for (position, fact) in facts.inputs.iter().enumerate() {
        let input_index = u32::try_from(position).expect("small fixture");
        let outpoint = InputOutpoint {
            txid: fact.txid,
            output_index: fact.index,
        };
        let Some(d) = descriptors.get(&input_index) else {
            inputs.push(KeyedInputContext {
                input_index,
                outpoint,
                role: KeyedInputRole::Sponsor,
                predecessor_program: vec![],
            });
            continue;
        };
        authenticate_keyed_predecessor(&target, d, &d.output_program, &PublicKeyedCurve)
            .expect("offline commitment before model execution");
        assert_eq!(fact.program.rows(), program(d).rows());
        let recipe = KeyedLeafRecipe::new(d.semantic_key);
        inputs.push(KeyedInputContext {
            input_index,
            outpoint,
            role: KeyedInputRole::ConsumedHandle,
            predecessor_program: d.output_program.clone(),
        });
        members.push(KeyedMemberBinding {
            input_index,
            outpoint,
            recipe,
            offline_assertion: OfflinePredecessorAssertion {
                outpoint,
                predecessor_program: d.output_program.clone(),
                recipe,
            },
            relations: bundle.project().fragments.keys().copied().collect(),
        });
    }
    bind_keyed_members(bundle, &inputs, &members).expect("complete supplied binding")
}

fn run(
    bundle: &KeyedEmissionBundle,
    facts: &Facts,
    descriptors: &BTreeMap<u32, KeyedConstructorDescriptor>,
) -> BTreeMap<(KeyedRelationId, u32), Result<(), Refusal>> {
    let receipt = bind_offline(bundle, facts, descriptors);
    let target = target_elements::reviewed_elements_tapscript().expect("target");
    let mut results = BTreeMap::new();
    for (relation, members) in receipt.coverage() {
        for member in members {
            let mut local = facts.clone();
            local.current = Some(member.input_index);
            results.insert(
                (*relation, member.input_index),
                fragment(
                    &bundle.project().fragments[relation].encode(&target),
                    &local,
                )
                .map(|_| ()),
            );
        }
    }
    results
}

#[test]
fn production_bundle_checks_both_clauses_for_every_member_and_a_single_changed_member() {
    let bundle = bundle();
    let d = descriptor(42);
    let other = descriptor(43);
    let facts = context(&d);
    let descriptors = [(1, d.clone()), (2, d.clone())].into_iter().collect();
    let control = run(&bundle, &facts, &descriptors);
    let target = target_elements::reviewed_elements_tapscript().expect("target");
    assert_eq!(
        d.leaf_bytes,
        bundle.project().successor.program().encode(&target)
    );
    assert_eq!(control.len(), 4);
    assert!(control.values().all(Result::is_ok));
    for (index, d) in &descriptors {
        let mut local = facts.clone();
        local.current = Some(*index);
        assert!(leaf(&d.leaf_bytes, &local, &d.semantic_key).is_ok());
    }
    let mut later = facts.clone();
    later.inputs[2].index = 1;
    for ((id, index), outcome) in run(&bundle, &later, &descriptors) {
        assert_eq!(
            outcome,
            if index == 2 && id.kind == KeyedRelationKind::BornFirst {
                Err(Refusal::Unequal)
            } else {
                Ok(())
            }
        );
    }
    let mut wrong = facts;
    wrong.inputs[2].program = program(&other);
    let descriptors = [(1, d), (2, other)].into_iter().collect();
    for ((id, index), outcome) in run(&bundle, &wrong, &descriptors) {
        assert_eq!(
            outcome,
            if index == 2 && id.kind == KeyedRelationKind::ContinueFirst {
                Err(Refusal::Unequal)
            } else {
                Ok(())
            }
        );
    }
}

#[test]
fn member_permutation_and_collateral_flags_preserve_execution() {
    let bundle = bundle();
    let d = descriptor(42);
    let facts = context(&d);
    let descriptors = [(1, d.clone()), (2, d.clone())].into_iter().collect();
    assert!(
        run(&bundle, &facts, &descriptors)
            .values()
            .all(Result::is_ok)
    );
    let mut permuted = facts.clone();
    permuted.inputs.swap(0, 2);
    let moved = [(0, d.clone()), (1, d)].into_iter().collect();
    assert!(run(&bundle, &permuted, &moved).values().all(Result::is_ok));
    // OutPointFlags is exactly one byte (target encoding contract). The pinned
    // interpreter's GetOutpointFlag shifts issuance bit 31 and peg-in bit 30
    // right by 24, then OP_INSPECTINPUTOUTPOINT pushes that byte.
    for flags in [0x80, 0x40, 0xc0] {
        let mut subject = facts.clone();
        subject.inputs[2].flags = flags;
        assert_eq!(
            run(&bundle, &subject, &descriptors),
            run(&bundle, &facts, &descriptors)
        );
    }
}

#[test]
fn successor_script_version_and_position_fail_only_continuation() {
    let bundle = bundle();
    let d = descriptor(42);
    let facts = context(&d);
    let descriptors = [(1, d.clone()), (2, d)].into_iter().collect();
    let control = run(&bundle, &facts, &descriptors);
    assert!(control.values().all(Result::is_ok));
    let mut script = facts.clone();
    script.outputs[0] = ProgramFact::Other { sha256: [99; 32] };
    let mut version = facts.clone();
    if let ProgramFact::Witness { version, .. } = &mut version.outputs[0] {
        *version = 0;
    }
    let mut displaced = facts;
    displaced
        .outputs
        .insert(0, ProgramFact::Other { sha256: [99; 32] });
    for subject in [script, version, displaced] {
        for ((id, _), outcome) in run(&bundle, &subject, &descriptors) {
            assert_eq!(
                outcome,
                if id.kind == KeyedRelationKind::ContinueFirst {
                    Err(Refusal::Unequal)
                } else {
                    Ok(())
                }
            );
        }
    }
}

#[test]
fn commitment_subjects_refuse_before_the_execution_callback() {
    let target = target_elements::reviewed_elements_tapscript().expect("target");
    let d = descriptor(42);
    let facts = context(&d);
    let execute = |subject: &KeyedConstructorDescriptor| {
        authenticate_keyed_predecessor(&target, subject, &d.output_program, &PublicKeyedCurve)?;
        Ok::<_, KeyedConstructorRefusal>(leaf(&subject.leaf_bytes, &facts, &subject.semantic_key))
    };
    assert!(execute(&d).expect("authenticated").is_ok());
    let mut parity = d.clone();
    parity.control.parity = !parity.control.parity;
    assert_eq!(
        execute(&parity),
        Err(KeyedConstructorRefusal::DescriptorMismatch)
    );
    let mut script = d.clone();
    // Replace the version EQUALVERIFY with two DROPs. Under the control facts
    // the model still succeeds; only the oracle rejects this uncommitted leaf.
    let version_comparison = script.leaf_bytes.len() - 3;
    assert_eq!(script.leaf_bytes[version_comparison], 0x88);
    script
        .leaf_bytes
        .splice(version_comparison..=version_comparison, [0x75, 0x75]);
    assert!(leaf(&script.leaf_bytes, &facts, &script.semantic_key).is_ok());
    assert_eq!(
        execute(&script),
        Err(KeyedConstructorRefusal::DescriptorMismatch)
    );
    let other = descriptor(43);
    assert_eq!(
        authenticate_keyed_predecessor(&target, &other, &other.output_program, &PublicKeyedCurve),
        Ok(())
    );
    assert_eq!(
        execute(&other),
        Err(KeyedConstructorRefusal::PredecessorCommitment)
    );
}

#[test]
fn additional_output_after_successor_leaves_both_clauses_unchanged() {
    let bundle = bundle();
    let d = descriptor(42);
    let facts = context(&d);
    let descriptors = [(1, d.clone()), (2, d)].into_iter().collect();
    let control = run(&bundle, &facts, &descriptors);
    assert!(control.values().all(Result::is_ok));
    let mut open = facts;
    open.outputs.push(ProgramFact::Other { sha256: [99; 32] });
    assert_eq!(run(&bundle, &open, &descriptors), control);
}
