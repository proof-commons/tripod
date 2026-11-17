//! Independent record reconstruction and adjacent model subjects, not native runs.
use elements::hashes::sha256;
use realization::keyed_fixed::{KeyedFixedParameters, KeyedFixedValue};
use tapscript::keyed_constructor::{KeyedCurveCapability, KeyedTweakOutcome};
use vectors::keyed_fixed_constructor::{
    KeyedFixedConstructorRefusal, PublicKeyedCurve, authenticate_predecessor, construct,
    reconstruct,
};
use vectors::keyed_fixed_stack_model::{self as model, Facts, Field, Input, Output, ProgramFact};

fn hash(bytes: &[u8]) -> [u8; 32] {
    sha256::Hash::hash(bytes).to_byte_array()
}
fn tagged(tag: &[u8], message: &[u8]) -> [u8; 32] {
    hash(&[hash(tag).to_vec(), hash(tag).to_vec(), message.to_vec()].concat())
}
fn parameters() -> KeyedFixedParameters {
    KeyedFixedParameters {
        key: [0; 32],
        asset: [0x23; 32],
        value: KeyedFixedValue::new(0x0102_0304_0506_0708).unwrap(),
    }
}
fn independent_leaf(q: KeyedFixedParameters) -> (Vec<u8>, Vec<u8>) {
    let transcript = [
        q.asset.to_vec(),
        vec![1],
        q.value.get().to_le_bytes().to_vec(),
        vec![1],
    ]
    .concat();
    let leaf = [
        vec![32],
        q.key.to_vec(),
        vec![0x75, 32],
        hash(&transcript).to_vec(),
        vec![
            0x76, 0, 0xce, 0x7e, 0xc4, 0, 0xcf, 0x7e, 0xc6, 0x88, 0xcd, 0xc8, 0x7e, 0xc4, 0xcd,
            0xc9, 0x7e, 0xc6, 0x88, 0xcd, 0xc7, 0x75, 4, 0, 0, 0, 0, 0x88, 0x75, 0xcd, 0xca, 0,
            0xd1, 0x7b, 0x88, 0x88, 0x51,
        ],
    ]
    .concat();
    (transcript, leaf)
}

#[test]
fn independent_transcript_tags_curve_control_and_triple_binding() {
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let q = parameters();
    let mut triples = vec![q];
    let mut changed = q;
    changed.key[0] = 1;
    triples.push(changed);
    let mut changed = q;
    changed.asset[0] ^= 1;
    triples.push(changed);
    let mut changed = q;
    changed.value = KeyedFixedValue::new(q.value.get() + 1).unwrap();
    triples.push(changed);
    let mut programs = std::collections::BTreeSet::new();
    for q in triples {
        let descriptor = construct(&target, q, &PublicKeyedCurve).unwrap();
        let (transcript, leaf) = independent_leaf(q);
        assert_eq!(descriptor.transcript.as_slice(), transcript);
        assert_eq!(descriptor.pair_digest, hash(&transcript));
        assert_eq!(descriptor.leaf, leaf);
        let internal = hash(
            &[
                vec![4],
                tapscript::STATE_GENERATOR_X.to_vec(),
                tapscript::STATE_GENERATOR_Y.to_vec(),
            ]
            .concat(),
        );
        assert_eq!(internal, tapscript::STATE_NUMS_KEY);
        assert_eq!(descriptor.internal_key, internal);
        let root = tagged(b"TapLeaf/elements", &[vec![0xc4, 104], leaf].concat());
        assert_eq!(descriptor.root, root);
        assert_eq!(
            descriptor.tweak,
            tagged(b"TapTweak/elements", &[internal, root].concat())
        );
        let KeyedTweakOutcome::OutputKey { key, parity } =
            transaction::keyed_curve::KeyedCurve.output_key(&internal, &root)
        else {
            panic!("public independent curve");
        };
        assert_eq!(descriptor.output_key, key);
        assert_eq!(descriptor.parity, parity);
        assert_eq!(
            descriptor.output_program,
            [vec![0x51, 32], key.to_vec()].concat()
        );
        assert_eq!(
            descriptor.control,
            [vec![0xc4 | u8::from(parity)], internal.to_vec()].concat()
        );
        assert_eq!(
            reconstruct(&target, &descriptor, &PublicKeyedCurve),
            Ok(descriptor.clone())
        );
        let mut bad = descriptor.clone();
        bad.control[0] ^= 1;
        assert_eq!(
            reconstruct(&target, &bad, &PublicKeyedCurve),
            Err(KeyedFixedConstructorRefusal::DescriptorMismatch)
        );
        let mut bad = descriptor.clone();
        bad.control.extend([0; 32]);
        assert_eq!(
            reconstruct(&target, &bad, &PublicKeyedCurve),
            Err(KeyedFixedConstructorRefusal::DescriptorMismatch)
        );
        let mut bad = descriptor.clone();
        bad.leaf[1] ^= 1;
        assert_eq!(
            reconstruct(&target, &bad, &PublicKeyedCurve),
            Err(KeyedFixedConstructorRefusal::DescriptorMismatch)
        );
        programs.insert(descriptor.output_program);
    }
    assert_eq!(programs.len(), 4);
}

#[test]
fn every_stack_boundary_matches_the_record_schedule() {
    use model::Item::{Bytes, Context};
    let (transcript, leaf) = independent_leaf(parameters());
    let digest = Bytes(hash(&transcript).to_vec());
    let asset = Bytes(vec![0x23; 32]);
    let prefix = Bytes(vec![1]);
    let joined_asset = Bytes([vec![0x23; 32], vec![1]].concat());
    let context = Context([vec![0x23; 32], vec![1]].concat());
    let value = Bytes(vec![8, 7, 6, 5, 4, 3, 2, 1]);
    let joined_value = Bytes(vec![8, 7, 6, 5, 4, 3, 2, 1, 1]);
    let zero = Bytes(vec![]);
    let one = Bytes(vec![1]);
    let txid = Bytes(vec![6; 32]);
    let birth = Bytes(vec![0; 4]);
    let program = Bytes(vec![7; 32]);
    let key = Bytes(vec![0; 32]);
    let flags = Bytes(vec![3]);
    // Rows are the record's stack, bottom first, after each literal instruction.
    // These are not obtained from the emitter, decoder or model transition code.
    let rows: [Vec<&model::Item>; 37] = [
        vec![],
        vec![&key],
        vec![],
        vec![&digest],
        vec![&digest, &digest],
        vec![&digest, &digest, &zero],
        vec![&digest, &digest, &asset, &prefix],
        vec![&digest, &digest, &joined_asset],
        vec![&digest, &digest, &context],
        vec![&digest, &digest, &context, &zero],
        vec![&digest, &digest, &context, &value, &prefix],
        vec![&digest, &digest, &context, &joined_value],
        vec![&digest, &digest, &digest],
        vec![&digest],
        vec![&digest, &one],
        vec![&digest, &asset, &prefix],
        vec![&digest, &joined_asset],
        vec![&digest, &context],
        vec![&digest, &context, &one],
        vec![&digest, &context, &value, &prefix],
        vec![&digest, &context, &joined_value],
        vec![&digest, &digest],
        vec![],
        vec![&one],
        vec![&txid, &birth, &flags],
        vec![&txid, &birth],
        vec![&txid, &birth, &birth],
        vec![&txid],
        vec![],
        vec![&one],
        vec![&program, &one],
        vec![&program, &one, &zero],
        vec![&program, &one, &program, &one],
        vec![&program, &program, &one, &one],
        vec![&program, &program],
        vec![],
        vec![&one],
    ];
    let expected: Vec<Vec<model::Item>> = rows
        .into_iter()
        .map(|row| row.into_iter().cloned().collect())
        .collect();
    assert_eq!(model::evaluate(&leaf, &facts()).unwrap().stacks, expected);
}

#[test]
fn zero_value_alias_refuses_while_unrelated_outputs_remain_unconstrained() {
    let (_, leaf) = independent_leaf(parameters());
    let control = facts();
    assert!(model::evaluate(&leaf, &control).is_ok());
    for input in [false, true] {
        let mut subject = control.clone();
        let output = if input {
            &mut subject.inputs[1].output
        } else {
            &mut subject.outputs[0]
        };
        // The record gives null and explicit zero the same introspected bytes.
        output.value.payload = vec![0; 8];
        assert_eq!(
            model::evaluate(&leaf, &subject),
            Err(model::Refusal {
                instruction: if input { 21 } else { 12 },
                kind: "unequal"
            })
        );
    }
    let mut with_surplus = control;
    let mut ordinary = with_surplus.outputs[0].clone();
    ordinary.asset.payload[0] ^= 1;
    ordinary.value.payload = vec![255; 8];
    ordinary.program = ProgramFact::Unavailable;
    with_surplus.outputs.push(ordinary);
    assert!(model::evaluate(&leaf, &with_surplus).is_ok());
}

#[test]
fn arithmetic_refusals_and_predecessor_authentication_are_explicit() {
    struct Outcome(KeyedTweakOutcome);
    impl KeyedCurveCapability for Outcome {
        fn internal_key_is_a_point(&self, _: &[u8; 32]) -> bool {
            true
        }
        fn output_key(&self, _: &[u8; 32], _: &[u8; 32]) -> KeyedTweakOutcome {
            self.0
        }
    }
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let q = parameters();
    let control = construct(&target, q, &PublicKeyedCurve).unwrap();
    for (outcome, refusal) in [
        (
            KeyedTweakOutcome::InternalKeyNotAPoint,
            KeyedFixedConstructorRefusal::InternalKeyNotAPoint,
        ),
        (
            KeyedTweakOutcome::TweakAboveGroupOrder,
            KeyedFixedConstructorRefusal::TweakAboveGroupOrder,
        ),
        (
            KeyedTweakOutcome::TweakedPointIsIdentity,
            KeyedFixedConstructorRefusal::TweakedPointIsIdentity,
        ),
    ] {
        // Capability-level adjacent subjects exercise propagation, not rare hashes.
        assert_eq!(construct(&target, q, &Outcome(outcome)), Err(refusal));
    }
    assert_eq!(
        authenticate_predecessor(
            &target,
            &control,
            &control.output_program,
            &PublicKeyedCurve
        ),
        Ok(())
    );
    let mut predecessor = control.output_program.clone();
    predecessor[2] ^= 1;
    assert_eq!(
        authenticate_predecessor(&target, &control, &predecessor, &PublicKeyedCurve),
        Err(KeyedFixedConstructorRefusal::PredecessorCommitment)
    );
}

fn facts() -> Facts {
    let q = parameters();
    let output = Output {
        program: ProgramFact::Witness {
            version: 1,
            payload: vec![7; 32],
        },
        asset: Field {
            payload: q.asset.to_vec(),
            prefix: 1,
        },
        value: Field {
            payload: q.value.get().to_le_bytes().to_vec(),
            prefix: 1,
        },
    };
    Facts {
        current: 1,
        inputs: vec![
            Input {
                txid: [5; 32],
                birth: 9,
                flags: 0,
                output: output.clone(),
            },
            Input {
                txid: [6; 32],
                birth: 0,
                flags: 3,
                output: output.clone(),
            },
        ],
        outputs: vec![output],
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Whole-leaf adjacent subjects retain their exact refusal sites.
fn model_checks_both_forms_birth_flags_and_absent_successor_at_exact_sites() {
    let (_, leaf) = independent_leaf(parameters());
    let control = facts();
    let trace = model::evaluate(&leaf, &control).unwrap();
    assert_eq!(
        (
            trace.stacks.len(),
            trace.primitives,
            trace.peak,
            trace.width
        ),
        (37, 29, 5, 73)
    );
    assert_eq!(
        trace.stacks.last(),
        Some(&vec![model::Item::Bytes(vec![1])])
    );
    for input in [false, true] {
        for asset in [false, true] {
            for prefix in if asset { [10, 11] } else { [8, 9] } {
                let mut subject = control.clone();
                let output = if input {
                    &mut subject.inputs[1].output
                } else {
                    &mut subject.outputs[0]
                };
                let field = if asset {
                    &mut output.asset
                } else {
                    &mut output.value
                };
                field.prefix = prefix;
                field.payload.resize(32, 0);
                assert_eq!(
                    model::evaluate(&leaf, &subject),
                    Err(model::Refusal {
                        instruction: if input { 21 } else { 12 },
                        kind: "unequal"
                    })
                );
            }
            let mut subject = control.clone();
            let output = if input {
                &mut subject.inputs[1].output
            } else {
                &mut subject.outputs[0]
            };
            let field = if asset {
                &mut output.asset
            } else {
                &mut output.value
            };
            field.payload[0] ^= 1;
            assert_eq!(
                model::evaluate(&leaf, &subject),
                Err(model::Refusal {
                    instruction: if input { 21 } else { 12 },
                    kind: "unequal"
                })
            );
        }
    }
    let mut subject = control.clone();
    subject.outputs.clear();
    assert_eq!(
        model::evaluate(&leaf, &subject),
        Err(model::Refusal {
            instruction: 5,
            kind: "output-index"
        })
    );
    let mut subject = control.clone();
    subject.inputs[1].birth = 1;
    assert_eq!(
        model::evaluate(&leaf, &subject),
        Err(model::Refusal {
            instruction: 26,
            kind: "unequal"
        })
    );
    for flags in [0, 1, 2, 3, 255] {
        let mut subject = control.clone();
        subject.inputs[1].flags = flags;
        assert!(model::evaluate(&leaf, &subject).is_ok());
    }
    let mut subject = control.clone();
    subject.outputs[0].program = ProgramFact::Witness {
        version: 2,
        payload: vec![7; 32],
    };
    assert_eq!(
        model::evaluate(&leaf, &subject),
        Err(model::Refusal {
            instruction: 33,
            kind: "unequal"
        })
    );
    let mut subject = control;
    subject.outputs[0].program = ProgramFact::Witness {
        version: 1,
        payload: vec![8; 32],
    };
    assert_eq!(
        model::evaluate(&leaf, &subject),
        Err(model::Refusal {
            instruction: 34,
            kind: "unequal"
        })
    );
}
