//! Test-only schedule and mutation controls for the unchanged byte core.
use crate::keyed_stack_core::{Facts, InputFact, ProgramFact, Refusal, evaluate, fragment, leaf};
mod constructed;
mod members;

const BIRTH: &[u8] = &[0xcd, 0xc7, 0x75, 4, 0, 0, 0, 0, 0x88, 0x75];
const CONTINUATION: &[u8] = &[0xcd, 0xca, 0, 0xd1, 0x7b, 0x88, 0x88];
const KEY: [u8; 32] = [0x42; 32];
fn grammar() -> Vec<u8> {
    let mut out = vec![32];
    out.extend(KEY);
    out.push(0x75);
    out.extend(BIRTH);
    out.extend(CONTINUATION);
    out.push(0x51);
    out
}
fn facts() -> Facts {
    let program = ProgramFact::Witness {
        version: 1,
        payload: vec![0x31; 32],
    };
    Facts {
        enabled: true,
        current: Some(1),
        inputs: vec![
            InputFact {
                txid: [0x11; 32],
                index: 9,
                flags: 0,
                program: program.clone(),
            },
            InputFact {
                txid: [0x22; 32],
                index: 0,
                flags: 3,
                program: program.clone(),
            },
        ],
        outputs: vec![program],
    }
}

#[test]
fn model_separates_birth_continuation_and_leaf_with_independent_counts() {
    let target = target_elements::reviewed_elements_tapscript().expect("reviewed target");
    assert_eq!(
        tapscript::keyed_pattern::born_first_fragment(&target)
            .expect("birth")
            .encode(&target),
        BIRTH
    );
    assert_eq!(
        tapscript::keyed_pattern::continue_first_fragment(&target)
            .expect("continuation")
            .encode(&target),
        CONTINUATION
    );
    let context = facts();
    let birth = fragment(BIRTH, &context).expect("birth model");
    let continuation = fragment(CONTINUATION, &context).expect("continuation model");
    let composed = leaf(&grammar(), &context, &KEY).expect("leaf model");
    let txid = vec![0x22; 32];
    let zero = vec![0; 4];
    let payload = vec![0x31; 32];
    assert_eq!(
        birth.main,
        vec![
            vec![],
            vec![vec![1]],
            vec![txid.clone(), zero.clone(), vec![3]],
            vec![txid.clone(), zero.clone()],
            vec![txid.clone(), zero.clone(), zero],
            vec![txid],
            vec![]
        ]
    );
    assert_eq!(
        continuation.main,
        vec![
            vec![],
            vec![vec![1]],
            vec![payload.clone(), vec![1]],
            vec![payload.clone(), vec![1], vec![]],
            vec![payload.clone(), vec![1], payload.clone(), vec![1]],
            vec![payload.clone(), payload.clone(), vec![1], vec![1]],
            vec![payload.clone(), payload],
            vec![]
        ]
    );
    assert_eq!(composed.main[1], vec![KEY.to_vec()]);
    assert_eq!(
        (birth.primitive_count, birth.peak, birth.maximum_width),
        (5, 3, 32)
    );
    assert_eq!(
        (
            continuation.primitive_count,
            continuation.peak,
            continuation.maximum_width
        ),
        (6, 4, 32)
    );
    assert_eq!(
        (
            composed.primitive_count,
            composed.peak,
            composed.maximum_width
        ),
        (12, 4, 32)
    );
    for (schedule, model) in [
        (tapscript::keyed_resources::KeyedSchedule::Birth, &birth),
        (
            tapscript::keyed_resources::KeyedSchedule::Continuation,
            &continuation,
        ),
        (tapscript::keyed_resources::KeyedSchedule::Leaf, &composed),
    ] {
        assert_eq!(
            tapscript::keyed_resources::keyed_resource_projection(&target, schedule)
                .expect("projection")
                .primitive_count,
            model.primitive_count
        );
    }
    assert_eq!(composed.main[15], Vec::<Vec<u8>>::new());
    assert_eq!(composed.main[16], vec![vec![1]]);
    assert_eq!(&composed.main[2..=8], birth.main.as_slice());
    assert_eq!(&composed.main[8..=15], continuation.main.as_slice());
    // The first input is deliberately ineligible: current-input introspection
    // selects input one, and flags are not required to be zero.
    let mut bad = context.clone();
    bad.current = Some(0);
    assert_eq!(fragment(BIRTH, &bad), Err(Refusal::Unequal));
    let mut bad = context.clone();
    bad.outputs[0] = ProgramFact::Witness {
        version: 2,
        payload: vec![0x31; 32],
    };
    assert_eq!(fragment(CONTINUATION, &bad), Err(Refusal::Unequal));
    let mut bad = context;
    bad.outputs[0] = ProgramFact::Witness {
        version: 1,
        payload: vec![0x32; 32],
    };
    assert_eq!(fragment(CONTINUATION, &bad), Err(Refusal::Unequal));
}

#[test]
fn model_pins_seven_mutation_refusals_against_one_shared_control() {
    let original = grammar();
    let target = target_elements::reviewed_elements_tapscript().expect("target");
    let accepted = leaf(&original, &facts(), &KEY).expect("shared accepted control");
    let original_schedule =
        tapscript::TapscriptProgram::decode(&target, &original).expect("control");
    let baseline = abstract_prefixes(&original_schedule);
    let mut subjects = Vec::new();
    for (name, offset, instruction, refusal) in [
        ("flags drop", 36, 4, Refusal::Unequal),
        ("txid drop", 43, 7, Refusal::Residue),
        ("rotation", 48, 12, Refusal::Unequal),
        ("final true", 51, 15, Refusal::Residue),
    ] {
        let mut bytes = original.clone();
        bytes.remove(offset);
        subjects.push((name, bytes, instruction, refusal));
    }
    let mut wrong_width = original.clone();
    wrong_width.splice(37..42, [0]);
    subjects.push(("index width", wrong_width, 5, Refusal::Unequal));
    let mut residue = original.clone();
    residue.insert(51, 0);
    subjects.push(("extra residue", residue, 15, Refusal::Residue));
    let mut key = original.clone();
    key[1] ^= 1;
    subjects.push(("key push", key, 0, Refusal::RecipeKey));
    for (name, subject, instruction, refusal) in subjects {
        assert_eq!(leaf(&subject, &facts(), &KEY), Err(refusal), "{name}");
        let schedule = tapscript::TapscriptProgram::decode(&target, &subject).expect("mutant");
        assert_ne!(schedule.encode(&target), original, "{name}");
        assert_ne!(
            schedule.instructions(),
            original_schedule.instructions(),
            "{name}"
        );
        let changed = abstract_prefixes(&schedule);
        assert_eq!(
            &changed[..=instruction],
            &baseline[..=instruction],
            "unchanged prefixes: {name}"
        );
        if name == "key push" {
            assert_eq!(changed, baseline); // width-only analysis cannot see a changed key.
            assert_eq!(
                evaluate(&subject, &facts())
                    .expect("raw execution succeeds")
                    .main
                    .last(),
                accepted.main.last()
            );
        } else if name == "final true" {
            assert_ne!(
                changed.last(),
                baseline.last(),
                "final stack, not vector length"
            );
        } else {
            assert_ne!(
                changed[instruction + 1],
                baseline[instruction + 1],
                "aligned affected prefix: {name}"
            );
        }
    }
}

#[test]
fn clean_stack_comparison_omissions_accept_the_wrong_fact_that_control_refuses() {
    let target = target_elements::reviewed_elements_tapscript().expect("target");
    let original = grammar();
    let baseline = abstract_prefixes(
        &tapscript::TapscriptProgram::decode(&target, &original).expect("control"),
    );
    let mut mutations = Vec::new();
    for (offset, wrong_program) in [
        (
            49,
            ProgramFact::Witness {
                version: 2,
                payload: vec![0x31; 32],
            },
        ),
        (
            50,
            ProgramFact::Witness {
                version: 1,
                payload: vec![0x32; 32],
            },
        ),
    ] {
        let mut subject = original.clone();
        subject.splice(offset..=offset, [0x75, 0x75]);
        let mut adjacent = facts();
        adjacent.outputs[0] = wrong_program;
        assert!(leaf(&original, &facts(), &KEY).is_ok());
        assert!(leaf(&subject, &facts(), &KEY).is_ok());
        assert_eq!(leaf(&original, &adjacent, &KEY), Err(Refusal::Unequal));
        assert!(leaf(&subject, &adjacent, &KEY).is_ok());
        let schedule =
            tapscript::TapscriptProgram::decode(&target, &subject).expect("clean-stack mutant");
        assert_ne!(schedule.encode(&target), original);
        // Both finish with the same abstract stack/failures; the literal
        // schedule pin and paired concrete fact expose the omitted predicate.
        assert_eq!(abstract_prefixes(&schedule).last(), baseline.last());
        mutations.push(subject);
    }
    assert_ne!(mutations[0], mutations[1]);
}

#[test]
fn model_domain_bounds_and_unsupported_opcodes_are_explicit() {
    let control = facts();
    assert!(leaf(&grammar(), &control, &KEY).is_ok());
    for op in [0xac, 0x6b, 0x6c, 0xff] {
        assert_eq!(evaluate(&[op], &control), Err(Refusal::Unsupported(op)));
    }
    assert_eq!(evaluate(&[32, 1], &control), Err(Refusal::Truncated));
    for op in [0x75, 0x7b, 0x88] {
        assert_eq!(evaluate(&[op], &control), Err(Refusal::Underflow));
    }
    assert!(evaluate(&[0, 0xc7], &control).is_ok());
    assert_eq!(evaluate(&[1, 0x81, 0xc7], &control), Err(Refusal::Index));
    let mut no_outputs = control.clone();
    no_outputs.outputs.clear();
    assert_eq!(fragment(CONTINUATION, &no_outputs), Err(Refusal::Index));
    assert!(evaluate(&[0xcd], &control).is_ok());
    let mut invalid_current = control.clone();
    invalid_current.current = Some(i32::MAX as u32 + 1);
    assert_eq!(
        evaluate(&[0xcd], &invalid_current),
        Err(Refusal::Fact("current_index"))
    );
    assert_eq!(evaluate(&[0; 129], &control), Err(Refusal::Bound));
    let mut missing = control.clone();
    missing.enabled = false;
    assert_eq!(fragment(BIRTH, &missing), Err(Refusal::Domain));
    assert!(evaluate(&[1, 0x42], &missing).is_ok());
    assert!(evaluate(&[1, 0x42, 0x75], &control).is_ok());
    assert_eq!(evaluate(&[1, 0x42, 0x75], &missing), Err(Refusal::Domain));
    missing = control.clone();
    missing.current = None;
    assert_eq!(fragment(BIRTH, &missing), Err(Refusal::Context));
    missing = control.clone();
    missing.current = Some(2);
    assert_eq!(fragment(BIRTH, &missing), Err(Refusal::Index));
    assert_eq!(evaluate(&[1, 0, 0xc7], &control), Err(Refusal::Number));
    for width in [2, 40] {
        let mut context = control.clone();
        let p = ProgramFact::Witness {
            version: 16,
            payload: vec![0x31; width],
        };
        context.inputs[1].program = p.clone();
        context.outputs[0] = p;
        assert_eq!(
            fragment(CONTINUATION, &context)
                .expect("payload domain")
                .maximum_width,
            width
        );
    }
    for (version, width) in [(1, 1), (1, 41), (17, 32)] {
        let mut invalid = control.clone();
        invalid.inputs[1].program = ProgramFact::Witness {
            version,
            payload: vec![0x31; width],
        };
        assert_eq!(
            fragment(CONTINUATION, &invalid),
            Err(Refusal::Fact("witness_program"))
        );
    }
    let mut digest = control;
    digest.inputs[1].program = ProgramFact::Other { sha256: [0x41; 32] };
    digest.outputs[0] = digest.inputs[1].program.clone();
    assert!(fragment(CONTINUATION, &digest).is_ok());
}

// Test comparison only: the model above never calls this schedule oracle.
fn abstract_prefixes(
    program: &tapscript::TapscriptProgram,
) -> Vec<Result<tapscript::AbstractExecutionResult, tapscript::TapscriptError>> {
    let target = target_elements::reviewed_elements_tapscript().expect("target");
    (0..=program.instructions().len())
        .map(|n| {
            let prefix = tapscript::TapscriptProgram::new(program.instructions()[..n].to_vec())
                .expect("prefix");
            tapscript::validate_program(
                &target,
                &prefix,
                &tapscript::AbstractStackState::from_main(vec![]),
                tapscript::AbstractLimits::for_target(&target),
            )
        })
        .collect()
}

#[test]
fn resource_weight_matches_target_serialization_and_admitted_marginal_input() {
    use transaction::bytes::{
        AssetField, AssetId, InputWitness, NonceField, Outpoint, TargetInput, TargetOutput,
        TargetTransaction, Txid, ValueField,
    };
    let target = target_elements::reviewed_elements_tapscript().expect("target");
    let mut small_weights = Vec::new();
    for count in [1_u32, 2, 252, 253] {
        let inputs = (0..count)
            .map(|n| {
                TargetInput::new(
                    Outpoint::new(Txid::from_internal([0x11; 32]), n).expect("outpoint"),
                    u32::MAX,
                )
            })
            .collect();
        let witness = InputWitness::new(vec![grammar(), vec![0xc4; 33]]);
        let cost = crate::keyed_cost::witness_cost(&witness);
        assert_eq!(
            (cost.script_witness_bytes, cost.input_witness_bytes),
            (88, 91)
        );
        let output = TargetOutput::new(
            AssetField::Explicit(AssetId::from_internal([1; 32])),
            ValueField::Explicit(1),
            NonceField::Null,
            vec![0x51],
        );
        let tx = TargetTransaction::new(
            2,
            inputs,
            vec![output],
            0,
            vec![witness; usize::try_from(count).expect("count")],
        )
        .expect("serialization fixture, not admitted transaction");
        let base = tx.encode_without_witness();
        let other = u64::try_from(tx.output_witness_bytes().len()).expect("output fields");
        assert_eq!(base[4], 0); // the flag is a base byte in both encodings.
        assert_eq!(tx.encode()[4], 1);
        assert_eq!(tx.witness_bytes(), 91 * u64::from(count) + other);
        assert_eq!(
            tapscript::keyed_resources::keyed_transaction_weight(
                &target,
                u64::from(count),
                u64::try_from(base.len()).expect("base"),
                other
            ),
            Ok(tx.weight())
        );
        if count <= 2 {
            small_weights.push(tx.weight());
        }
    }
    assert_eq!(
        small_weights[1] - small_weights[0],
        crate::keyed_scaling::INPUT_WEIGHT
    );
    assert_eq!(crate::keyed_scaling::INPUT_WEIGHT, 255);
}

#[test]
fn unavailable_program_is_an_explicit_model_fact_refusal() {
    let control = facts();
    assert!(fragment(CONTINUATION, &control).is_ok());
    let mut subject = control;
    subject.inputs[1].program = ProgramFact::Unavailable;
    assert_eq!(
        fragment(CONTINUATION, &subject),
        Err(Refusal::Fact("missing_program"))
    );
}
