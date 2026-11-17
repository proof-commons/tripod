//! Seven focused complete-reading controls for the fixed-form edition.
use realization::keyed_fixed::{KeyedFixedParameters, KeyedFixedValue};
use std::collections::{BTreeMap, BTreeSet};
use target_elements_conformance::constructor::tagged::sha256;
use transaction::{
    AssetField, AssetId, InputWitness, NonceField, Outpoint, TargetInput, TargetOutput,
    TargetTransaction, Txid, ValueField,
};
use vectors::keyed_fixed_constructor::{PublicKeyedCurve, construct};
use vectors::keyed_fixed_evidence::{self as evidence, Finality, Report, Request};
use vectors::keyed_fixed_history_model as model;
use vectors::keyed_fixed_history_reader::{
    self as reader, Address, Handle, Input, Membership, Missing, Output, Query, Refusal, Triple,
};

// An opaque example asset; membership never consults a deployment policy asset.
const FIXTURE_ASSET: [u8; 32] = [3; 32];

const fn triple(seed: u8) -> Triple {
    Triple {
        key: [seed; 32],
        asset: FIXTURE_ASSET,
        value: 1_000,
    }
}

fn descriptor(triple: Triple) -> vectors::keyed_fixed_constructor::KeyedFixedDescriptor {
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    construct(
        &target,
        KeyedFixedParameters {
            key: triple.key,
            asset: triple.asset,
            value: KeyedFixedValue::new(triple.value).unwrap(),
        },
        &PublicKeyedCurve,
    )
    .unwrap()
}

const fn output(triple: Triple, program: Vec<u8>, nonce: NonceField) -> TargetOutput {
    TargetOutput::new(
        AssetField::Explicit(AssetId::from_internal(triple.asset)),
        ValueField::Explicit(triple.value),
        nonce,
        program,
    )
}

fn address(tx: &TargetTransaction) -> Address {
    let stripped = tx.encode_without_witness();
    Address(sha256(&sha256(&stripped)))
}

fn tx(
    parents: &[(Address, u32)],
    outputs: Vec<TargetOutput>,
    witness: Vec<InputWitness>,
    salt: u32,
) -> TargetTransaction {
    let inputs = if parents.is_empty() {
        vec![TargetInput::new(
            Outpoint::new(Txid::from_internal([u8::try_from(salt).unwrap(); 32]), salt).unwrap(),
            123,
        )]
    } else {
        parents
            .iter()
            .map(|(addr, index)| {
                TargetInput::new(
                    Outpoint::new(Txid::from_internal(addr.0), *index).unwrap(),
                    123,
                )
            })
            .collect()
    };
    let witnesses = if parents.is_empty() {
        vec![InputWitness::new(vec![])]
    } else {
        witness
    };
    TargetTransaction::new(2, inputs, outputs, 0, witnesses).unwrap()
}

fn funding(salt: u32) -> TargetTransaction {
    tx(
        &[],
        vec![TargetOutput::new(
            AssetField::Explicit(AssetId::from_internal([3; 32])),
            ValueField::Explicit(50_000),
            NonceField::Null,
            vec![0x51],
        )],
        vec![],
        salt,
    )
}

fn add_event(input: &mut Input, id: u32, tx: &TargetTransaction) {
    let addr = address(tx);
    input.association.push((id, addr));
    input.bytes.insert(id, tx.encode());
    input.assertions.inclusion.insert(addr);
    input.assertions.retrieval.insert(addr);
}

fn add_parent(input: &mut Input, tx: &TargetTransaction) {
    let addr = address(tx);
    input.retrieved.insert(addr, tx.encode());
    input.assertions.inclusion.insert(addr);
    input.assertions.retrieval.insert(addr);
}

const fn handle(triple: Triple, event: u32) -> Handle {
    Handle {
        triple,
        output: Output { event, position: 0 },
    }
}

const fn from_model_handle(value: model::Handle) -> Handle {
    Handle {
        triple: Triple {
            key: value.triple.key,
            asset: value.triple.asset,
            value: value.triple.value,
        },
        output: Output {
            event: value.output.event,
            position: value.output.position,
        },
    }
}

struct Fixture {
    input: Input,
    events: BTreeMap<u32, TargetTransaction>,
    triple: Triple,
}

fn fixture() -> Fixture {
    let triple = triple(7);
    let recipe = descriptor(triple);
    let mut input = Input {
        bytes: BTreeMap::new(),
        association: Vec::new(),
        triples: BTreeSet::from([triple]),
        retrieved: BTreeMap::new(),
        assertions: reader::Assertions::default(),
    };
    let mut events = BTreeMap::new();
    for (id, salt) in [(1, 11), (2, 12)] {
        let fund = funding(salt);
        add_parent(&mut input, &fund);
        let source = tx(
            &[(address(&fund), 0)],
            vec![output(
                triple,
                recipe.output_program.clone(),
                NonceField::Null,
            )],
            vec![InputWitness::new(vec![])],
            id,
        );
        let source = if id == 2 {
            TargetTransaction::new(
                2,
                source.inputs().to_vec(),
                source.outputs().to_vec(),
                1,
                source.witnesses().to_vec(),
            )
            .unwrap()
        } else {
            source
        };
        add_event(&mut input, id, &source);
        events.insert(id, source);
    }
    let witness = InputWitness::new(vec![recipe.leaf.clone(), recipe.control.clone()]);
    let join = tx(
        &[(address(&events[&1]), 0), (address(&events[&2]), 0)],
        vec![
            output(triple, recipe.output_program.clone(), NonceField::Null),
            output(triple, vec![0x6a, 0x01, 0x44], NonceField::Null),
        ],
        vec![witness.clone(), witness.clone()],
        3,
    );
    add_event(&mut input, 3, &join);
    events.insert(3, join);
    let continuation = tx(
        &[(address(&events[&3]), 0)],
        vec![output(triple, recipe.output_program, NonceField::Null)],
        vec![witness],
        4,
    );
    add_event(&mut input, 4, &continuation);
    events.insert(4, continuation);
    Fixture {
        input,
        events,
        triple,
    }
}

fn records(reading: &reader::Reading, h: Handle) -> BTreeSet<Handle> {
    match reading.query(h) {
        Query::Handle {
            membership: Membership::Complete(set),
            ..
        } => set,
        other => panic!("not complete: {other:?}"),
    }
}

fn accepted_records(report: Report) -> BTreeSet<Handle> {
    match report {
        Report::Complete {
            accepted_records, ..
        } => accepted_records,
        other => panic!("not a complete report: {other:?}"),
    }
}

fn refused(
    result: Result<reader::Reading, Refusal>,
    expected: &Refusal,
    boundary: reader::Boundary,
) {
    let actual = result.unwrap_err();
    assert_eq!(&actual, expected);
    assert_eq!(actual.boundary(), boundary);
}

#[test]
#[allow(clippy::too_many_lines)] // One graph control compares every declared handle and its no-record extension.
fn fixed_reader_matches_independent_graph() {
    let f = fixture();
    let reading = reader::read(&f.input).unwrap();
    let q = model::Triple {
        key: f.triple.key,
        asset: f.triple.asset,
        value: f.triple.value,
    };
    let events = [
        model::Event {
            id: 1,
            inputs: vec![],
            outputs: vec![Some(q)],
        },
        model::Event {
            id: 2,
            inputs: vec![],
            outputs: vec![Some(q)],
        },
        model::Event {
            id: 3,
            inputs: vec![
                model::Output {
                    event: 1,
                    position: 0,
                },
                model::Output {
                    event: 2,
                    position: 0,
                },
            ],
            outputs: vec![Some(q), None],
        },
        model::Event {
            id: 4,
            inputs: vec![model::Output {
                event: 3,
                position: 0,
            }],
            outputs: vec![Some(q)],
        },
    ];
    let oracle = model::analyze(&events).unwrap();
    for (name, expected) in [
        (1, BTreeSet::from([handle(f.triple, 1)])),
        (2, BTreeSet::from([handle(f.triple, 2)])),
        (
            3,
            BTreeSet::from([handle(f.triple, 1), handle(f.triple, 2)]),
        ),
        (
            4,
            BTreeSet::from([handle(f.triple, 1), handle(f.triple, 2)]),
        ),
    ] {
        assert_eq!(records(&reading, handle(f.triple, name)), expected);
        let key = model::Handle {
            triple: q,
            output: model::Output {
                event: name,
                position: 0,
            },
        };
        let from_model: BTreeSet<_> = oracle.sets()[&key]
            .iter()
            .map(|record| handle(f.triple, record.output.event))
            .collect();
        assert_eq!(expected, from_model);
    }
    let expected_successors = BTreeMap::from([
        (handle(f.triple, 1), handle(f.triple, 3)),
        (handle(f.triple, 2), handle(f.triple, 3)),
        (handle(f.triple, 3), handle(f.triple, 4)),
    ]);
    let model_successors: BTreeMap<_, _> = oracle
        .successors()
        .iter()
        .map(|(before, after)| (from_model_handle(*before), from_model_handle(*after)))
        .collect();
    assert_eq!(*reading.successors(), expected_successors);
    assert_eq!(model_successors, expected_successors);
    let expected_live = BTreeSet::from([handle(f.triple, 4)]);
    let model_live: BTreeSet<_> = oracle
        .live()
        .iter()
        .copied()
        .map(from_model_handle)
        .collect();
    assert_eq!(*reading.live(), expected_live);
    assert_eq!(model_live, expected_live);
    let mut prefix = f.input.clone();
    prefix.bytes.remove(&4);
    prefix.association.retain(|(id, _)| *id != 4);
    assert_eq!(
        records(&reader::read(&prefix).unwrap(), handle(f.triple, 3)),
        records(&reading, handle(f.triple, 3))
    );
    let mut with_payment = f.input.clone();
    let fund = funding(41);
    add_parent(&mut with_payment, &fund);
    let payment = tx(
        &[(address(&fund), 0)],
        vec![output(f.triple, vec![0x51], NonceField::Null)],
        vec![InputWitness::new(vec![])],
        41,
    );
    add_event(&mut with_payment, 5, &payment);
    assert_eq!(
        reader::read(&with_payment)
            .unwrap()
            .query(handle(f.triple, 5)),
        Query::NotHandle
    );
    let mut extended = events.to_vec();
    extended.push(model::Event {
        id: 5,
        inputs: vec![],
        outputs: vec![None],
    });
    assert_eq!(model::analyze(&extended).unwrap().sets(), oracle.sets());
}

#[test]
#[allow(clippy::too_many_lines)] // One membership control keeps the pair, nonce, birth and cross-triple boundaries together.
fn fixed_reader_authenticates_exact_membership() {
    let f = fixture();
    let recipe = descriptor(f.triple);
    let fund = funding(99);
    let cases = [
        (
            output(
                f.triple,
                recipe.output_program.clone(),
                NonceField::Commitment([2; 33]),
            ),
            true,
        ),
        (
            output(f.triple, recipe.output_program.clone(), NonceField::Null),
            true,
        ),
        (
            output(f.triple, vec![0x51, 0x20, 1], NonceField::Null),
            false,
        ),
        (
            TargetOutput::new(
                AssetField::Explicit(AssetId::from_internal([4; 32])),
                ValueField::Explicit(f.triple.value),
                NonceField::Null,
                recipe.output_program.clone(),
            ),
            false,
        ),
        (
            TargetOutput::new(
                AssetField::Explicit(AssetId::from_internal(f.triple.asset)),
                ValueField::Explicit(f.triple.value + 1),
                NonceField::Null,
                recipe.output_program.clone(),
            ),
            false,
        ),
        (
            TargetOutput::new(
                AssetField::Commitment([10; 33]),
                ValueField::Explicit(f.triple.value),
                NonceField::Null,
                recipe.output_program.clone(),
            ),
            false,
        ),
        (
            TargetOutput::new(
                AssetField::Explicit(AssetId::from_internal(f.triple.asset)),
                ValueField::Commitment([8; 33]),
                NonceField::Null,
                recipe.output_program.clone(),
            ),
            false,
        ),
    ];
    for (index, (candidate, member)) in cases.into_iter().enumerate() {
        let mut input = f.input.clone();
        add_parent(&mut input, &fund);
        let subject = tx(
            &[(address(&fund), 0)],
            vec![candidate],
            vec![InputWitness::new(vec![])],
            100 + u32::try_from(index).unwrap(),
        );
        add_event(&mut input, 100, &subject);
        let query = reader::read(&input).unwrap().query(handle(f.triple, 100));
        let expected = if member {
            Query::Handle {
                handle: handle(f.triple, 100),
                membership: Membership::Complete(BTreeSet::from([handle(f.triple, 100)])),
            }
        } else {
            Query::NotHandle
        };
        assert_eq!(query, expected, "case {index}");
    }
    let mut later = f.input.clone();
    add_parent(&mut later, &fund);
    let subject = tx(
        &[(address(&fund), 0)],
        vec![
            output(f.triple, vec![0x51], NonceField::Null),
            output(f.triple, recipe.output_program.clone(), NonceField::Null),
        ],
        vec![InputWitness::new(vec![])],
        120,
    );
    add_event(&mut later, 120, &subject);
    assert_eq!(
        reader::read(&later).unwrap().query(Handle {
            triple: f.triple,
            output: Output {
                event: 120,
                position: 1
            }
        }),
        Query::NotHandle
    );
    let foreign = triple(8);
    let mut input = f.input.clone();
    input.triples.insert(foreign);
    assert_eq!(
        reader::read(&input).unwrap().query(handle(foreign, 1)),
        Query::WrongTriple {
            offered: handle(foreign, 1),
            actual: f.triple
        }
    );
    let mut wrong = f.input.clone();
    let version = TargetOutput::new(
        AssetField::Explicit(AssetId::from_internal(f.triple.asset)),
        ValueField::Explicit(f.triple.value),
        NonceField::Null,
        vec![0x52, 0x20].into_iter().chain([0; 32]).collect(),
    );
    let old = &f.events[&3];
    let subject = tx(
        &[(address(&f.events[&1]), 0), (address(&f.events[&2]), 0)],
        vec![version],
        old.witnesses().to_vec(),
        3,
    );
    wrong.bytes.insert(3, subject.encode());
    wrong.association.retain(|(id, _)| *id != 3);
    wrong.association.push((3, address(&subject)));
    refused(
        reader::read(&wrong),
        &Refusal::WrongVersion { event: 3, input: 0 },
        reader::Boundary::Clause,
    );
    let foreign_recipe = descriptor(foreign);
    let foreign_output = output(foreign, foreign_recipe.output_program, NonceField::Null);
    let foreign_subject = tx(
        &[(address(&f.events[&1]), 0), (address(&f.events[&2]), 0)],
        vec![foreign_output],
        old.witnesses().to_vec(),
        3,
    );
    let mut foreign_input = f.input.clone();
    foreign_input.triples.insert(foreign);
    foreign_input.bytes.insert(3, foreign_subject.encode());
    foreign_input.association.retain(|(id, _)| *id != 3);
    foreign_input
        .association
        .push((3, address(&foreign_subject)));
    refused(
        reader::read(&foreign_input),
        &Refusal::WrongSuccessor { event: 3, input: 0 },
        reader::Boundary::Clause,
    );
    let mut annex = f.input.clone();
    let annex_witness = InputWitness::new(vec![
        recipe.leaf.clone(),
        recipe.control.clone(),
        vec![0x50, 0x01],
    ]);
    let join_with_annex = tx(
        &[(address(&f.events[&1]), 0), (address(&f.events[&2]), 0)],
        f.events[&3].outputs().to_vec(),
        vec![annex_witness, f.events[&3].witnesses()[1].clone()],
        3,
    );
    annex.bytes.insert(3, join_with_annex.encode());
    annex.association.retain(|(id, _)| *id != 3);
    annex.association.push((3, address(&join_with_annex)));
    assert_eq!(
        records(&reader::read(&annex).unwrap(), handle(f.triple, 3)),
        BTreeSet::from([handle(f.triple, 1), handle(f.triple, 2)])
    );
    let mut collateral = f.input.clone();
    collateral.bytes.remove(&3);
    collateral.bytes.remove(&4);
    collateral.association.retain(|(id, _)| *id < 3);
    let p = triple(9);
    let p_recipe = descriptor(p);
    let p_fund = funding(201);
    add_parent(&mut collateral, &p_fund);
    let p_source = tx(
        &[(address(&p_fund), 0)],
        vec![output(p, p_recipe.output_program, NonceField::Null)],
        vec![InputWitness::new(vec![])],
        5,
    );
    add_event(&mut collateral, 5, &p_source);
    let collateral_spend = tx(
        &[(address(&f.events[&1]), 0), (address(&p_source), 0)],
        vec![output(f.triple, recipe.output_program, NonceField::Null)],
        vec![
            InputWitness::new(vec![recipe.leaf, recipe.control]),
            InputWitness::new(vec![]),
        ],
        6,
    );
    add_event(&mut collateral, 6, &collateral_spend);
    assert_eq!(
        records(&reader::read(&collateral).unwrap(), handle(f.triple, 6)),
        BTreeSet::from([handle(f.triple, 1)])
    );
    collateral.triples.insert(p);
    refused(
        reader::read(&collateral),
        &Refusal::WrongSuccessor { event: 6, input: 1 },
        reader::Boundary::Clause,
    );
}

#[test]
fn fixed_reader_requires_all_retrieval_fields_and_branches() {
    let f = fixture();
    assert_eq!(
        records(&reader::read(&f.input).unwrap(), handle(f.triple, 3)).len(),
        2
    );
    let mut cut = f.input.clone();
    cut.bytes.remove(&2);
    let reading = reader::read(&cut).unwrap();
    let Query::Handle {
        membership: Membership::Incomplete(missing),
        ..
    } = reading.query(handle(f.triple, 3))
    else {
        panic!("branch cut became complete");
    };
    assert_eq!(
        missing,
        BTreeSet::from([Missing::Parent {
            event: 3,
            input: 1,
            address: address(&f.events[&2]),
            position: 0
        }])
    );
    let mut no_parent = f.input.clone();
    no_parent.retrieved.remove(&address(&funding(11)));
    let Query::Handle {
        membership: Membership::Incomplete(missing),
        ..
    } = reader::read(&no_parent).unwrap().query(handle(f.triple, 3))
    else {
        panic!("funding cut became complete");
    };
    assert_eq!(
        missing,
        BTreeSet::from([Missing::Parent {
            event: 1,
            input: 0,
            address: address(&funding(11)),
            position: 0
        }])
    );
    for remove_inclusion in [true, false] {
        let mut input = f.input.clone();
        if remove_inclusion {
            input.assertions.inclusion.remove(&address(&f.events[&2]));
        } else {
            input.assertions.retrieval.remove(&address(&f.events[&2]));
        }
        let Query::Handle {
            membership: Membership::Incomplete(missing),
            ..
        } = reader::read(&input).unwrap().query(handle(f.triple, 3))
        else {
            panic!("assertion cut became complete");
        };
        let expected = if remove_inclusion {
            Missing::Inclusion(address(&f.events[&2]))
        } else {
            Missing::Retrieval(address(&f.events[&2]))
        };
        assert_eq!(missing, BTreeSet::from([expected]));
    }
    assert_eq!(
        records(&reader::read(&f.input).unwrap(), handle(f.triple, 1)),
        BTreeSet::from([handle(f.triple, 1)])
    );
}

#[test]
#[allow(clippy::too_many_lines)] // One identity control keeps contradictory mappings, byte digests and graph refusals beside the valid fixture.
fn fixed_reader_refuses_invalid_identity_and_dependencies() {
    let f = fixture();
    assert!(reader::read(&f.input).is_ok());
    let mut conflict = f.input.clone();
    conflict.association.push((1, Address([9; 32])));
    refused(
        reader::read(&conflict),
        &Refusal::ConflictingEvent(1),
        reader::Boundary::EvidenceConsistency,
    );
    let mut alias = f.input.clone();
    alias.association.retain(|(id, _)| *id != 2);
    alias.association.push((2, address(&f.events[&1])));
    refused(
        reader::read(&alias),
        &Refusal::ConflictingAlias(address(&f.events[&1])),
        reader::Boundary::EvidenceConsistency,
    );
    let fund = funding(11);
    // The reviewed independent serialization includes the zero flag byte.
    let correct = Address([
        125, 16, 80, 212, 200, 85, 112, 238, 136, 107, 169, 106, 73, 207, 219, 81, 245, 24, 135,
        128, 84, 250, 163, 146, 248, 124, 149, 47, 243, 214, 142, 90,
    ]);
    let flag_omitted = Address([
        68, 59, 187, 192, 64, 51, 58, 182, 146, 51, 69, 70, 148, 152, 90, 61, 123, 175, 7, 189,
        123, 196, 198, 92, 238, 23, 216, 11, 243, 16, 197, 175,
    ]);
    assert_eq!(address(&fund), correct);
    let mut wrong_digest = f.input.clone();
    wrong_digest.retrieved.insert(flag_omitted, fund.encode());
    refused(
        reader::read(&wrong_digest),
        &Refusal::ContentAddress {
            subject: reader::ByteSource::Retrieved(flag_omitted),
            offered: flag_omitted,
            computed: correct,
        },
        reader::Boundary::ContentAddressing,
    );
    let mut parent_conflict = f.input.clone();
    parent_conflict
        .retrieved
        .insert(address(&f.events[&1]), f.events[&2].encode());
    refused(
        reader::read(&parent_conflict),
        &Refusal::ConflictingParent(address(&f.events[&1])),
        reader::Boundary::EvidenceConsistency,
    );
    let mut duplicate = f.input.clone();
    let recipe = descriptor(f.triple);
    let witness = InputWitness::new(vec![recipe.leaf, recipe.control]);
    let double = tx(
        &[(address(&f.events[&1]), 0), (address(&f.events[&1]), 0)],
        vec![output(f.triple, recipe.output_program, NonceField::Null)],
        vec![witness.clone(), witness],
        3,
    );
    duplicate.bytes.insert(3, double.encode());
    duplicate.association.retain(|(id, _)| *id != 3);
    duplicate.association.push((3, address(&double)));
    refused(
        reader::read(&duplicate),
        &Refusal::DuplicateConsumption {
            address: address(&f.events[&1]),
            position: 0,
        },
        reader::Boundary::EvidenceConsistency,
    );
    let first_address = Address([71; 32]);
    let second_address = Address([72; 32]);
    let first = tx(
        &[(second_address, 0)],
        vec![output(f.triple, vec![0x51], NonceField::Null)],
        vec![InputWitness::new(vec![])],
        1,
    );
    let second = tx(
        &[(first_address, 0)],
        vec![output(f.triple, vec![0x51], NonceField::Null)],
        vec![InputWitness::new(vec![])],
        2,
    );
    let cycle = Input {
        bytes: BTreeMap::from([(1, first.encode()), (2, second.encode())]),
        association: vec![(1, first_address), (2, second_address)],
        triples: BTreeSet::from([f.triple]),
        retrieved: BTreeMap::new(),
        assertions: reader::Assertions::default(),
    };
    refused(
        reader::read(&cycle),
        &Refusal::Cycle,
        reader::Boundary::EvidenceConsistency,
    );
}

const fn all_sources(_: &TargetTransaction) -> bool {
    true
}
fn high_source(tx: &TargetTransaction) -> bool {
    tx.outputs()[0].value() == ValueField::Explicit(1_001)
}
fn attributed_source(tx: &TargetTransaction) -> bool {
    tx.inputs()[0].outpoint().index() == 1
}
const fn eligible_source(tx: &TargetTransaction) -> bool {
    tx.lock_time() == 2
}
fn source_only(tx: &TargetTransaction) -> bool {
    tx.outputs().len() == 1 && tx.outputs()[0].program().len() == 34
}

#[test]
fn fixed_reader_filters_are_source_folds() {
    let f = fixture();
    let reading = reader::read(&f.input).unwrap();
    let h = handle(f.triple, 3);
    let all = evidence::report(Request {
        reading: Ok(&reading),
        handle: h,
        source_predicate: &all_sources,
        finality: Finality::Unclaimed,
    });
    let attribution = evidence::report(Request {
        reading: Ok(&reading),
        handle: h,
        source_predicate: &source_only,
        finality: Finality::Unclaimed,
    });
    let expected_full = BTreeSet::from([handle(f.triple, 1), handle(f.triple, 2)]);
    assert_eq!(accepted_records(all), expected_full);
    for predicate in [
        high_source as fn(&TargetTransaction) -> bool,
        attributed_source,
        eligible_source,
    ] {
        let rejected = evidence::report(Request {
            reading: Ok(&reading),
            handle: h,
            source_predicate: &predicate,
            finality: Finality::Unclaimed,
        });
        assert_eq!(accepted_records(rejected), BTreeSet::new());
    }
    assert_eq!(accepted_records(attribution), expected_full);
    assert_eq!(records(&reading, h), expected_full);
    assert!(!records(&reading, h).contains(&h));
    assert_eq!(records(&reading, handle(f.triple, 4)), records(&reading, h));
    let selected_lock_time = 0;
    let accepts_first = |tx: &TargetTransaction| tx.lock_time() == selected_lock_time;
    for (event, expected) in [
        (1, BTreeSet::from([handle(f.triple, 1)])),
        (3, BTreeSet::from([handle(f.triple, 1)])),
        (4, BTreeSet::from([handle(f.triple, 1)])),
    ] {
        let report = evidence::report(Request {
            reading: Ok(&reading),
            handle: handle(f.triple, event),
            source_predicate: &accepts_first,
            finality: Finality::Unclaimed,
        });
        assert_eq!(accepted_records(report), expected);
    }
    let mut with_payment = f.input.clone();
    let fund = funding(72);
    add_parent(&mut with_payment, &fund);
    let payment = tx(
        &[(address(&fund), 0)],
        vec![output(f.triple, vec![0x6a, 0x01, 0x55], NonceField::Null)],
        vec![InputWitness::new(vec![])],
        72,
    );
    add_event(&mut with_payment, 5, &payment);
    let with_payment = reader::read(&with_payment).unwrap();
    assert_eq!(records(&with_payment, h), records(&reading, h));
    assert_eq!(with_payment.query(handle(f.triple, 5)), Query::NotHandle);
    assert_eq!(
        accepted_records(evidence::report(Request {
            reading: Ok(&with_payment),
            handle: h,
            source_predicate: &source_only,
            finality: Finality::Unclaimed,
        })),
        expected_full
    );
}

#[test]
fn fixed_reader_reports_premises_without_finality() {
    let f = fixture();
    let reading = reader::read(&f.input).unwrap();
    let complete = evidence::report(Request {
        reading: Ok(&reading),
        handle: handle(f.triple, 3),
        source_predicate: &all_sources,
        finality: Finality::Unclaimed,
    });
    assert!(matches!(
        complete,
        Report::Complete {
            finality: Finality::Unclaimed,
            ..
        }
    ));
    let mut cut = f.input.clone();
    cut.assertions.retrieval.remove(&address(&f.events[&2]));
    let incomplete = reader::read(&cut).unwrap();
    assert!(matches!(
        evidence::report(Request {
            reading: Ok(&incomplete),
            handle: handle(f.triple, 3),
            source_predicate: &all_sources,
            finality: Finality::Unclaimed
        }),
        Report::Incomplete(_)
    ));
    assert!(matches!(
        evidence::report(Request {
            reading: Ok(&reading),
            handle: Handle {
                triple: f.triple,
                output: Output {
                    event: 1,
                    position: 1
                }
            },
            source_predicate: &all_sources,
            finality: Finality::Unclaimed
        }),
        Report::NotHandle(Query::MissingOutput(_))
    ));
    let mut payment_input = f.input.clone();
    let fund = funding(61);
    add_parent(&mut payment_input, &fund);
    let payment = tx(
        &[(address(&fund), 0)],
        vec![output(f.triple, vec![0x51], NonceField::Null)],
        vec![InputWitness::new(vec![])],
        61,
    );
    add_event(&mut payment_input, 61, &payment);
    let payment_reading = reader::read(&payment_input).unwrap();
    assert!(matches!(
        evidence::report(Request {
            reading: Ok(&payment_reading),
            handle: handle(f.triple, 61),
            source_predicate: &all_sources,
            finality: Finality::Unclaimed
        }),
        Report::NotHandle(Query::NotHandle)
    ));
    let mut absent_event = f.input.clone();
    absent_event.bytes.remove(&2);
    let absent_reading = reader::read(&absent_event).unwrap();
    assert_eq!(
        evidence::report(Request {
            reading: Ok(&absent_reading),
            handle: handle(f.triple, 2),
            source_predicate: &all_sources,
            finality: Finality::Unclaimed,
        }),
        Report::Incomplete(BTreeSet::from([Missing::Event(2)]))
    );
    let unsupplied = triple(99);
    assert_eq!(
        evidence::report(Request {
            reading: Ok(&reading),
            handle: handle(unsupplied, 1),
            source_predicate: &all_sources,
            finality: Finality::Unclaimed,
        }),
        Report::UnprovidedTriple(unsupplied)
    );
    let mut conflict = f.input.clone();
    conflict.association.push((1, Address([9; 32])));
    let invalid = reader::read(&conflict).unwrap_err();
    assert_eq!(
        evidence::report(Request {
            reading: Err(&invalid),
            handle: handle(f.triple, 3),
            source_predicate: &all_sources,
            finality: Finality::Unclaimed
        }),
        Report::Invalid(invalid)
    );
}

#[test]
fn fixed_reader_workloads_and_path_boundary() {
    let f = fixture();
    let reading = reader::read(&f.input).unwrap();
    tracing::info!(target: "vectors::keyed_fixed_history", events = f.events.len(), edges = reading.edges(), bytes = reading.event_bytes(), "branching workload");
    assert_eq!(f.events.len(), 4);
    assert_eq!(reading.edges(), 5);
    assert_eq!(reading.event_bytes(), 1_043);
    assert_eq!(records(&reading, handle(f.triple, 4)).len(), 2);
    let mut path = f.input.clone();
    path.bytes.remove(&2);
    let path_reading = reader::read(&path).unwrap();
    assert!(matches!(
        path_reading.query(handle(f.triple, 4)),
        Query::Handle {
            membership: Membership::Incomplete(_),
            ..
        }
    ));
    let mut linear = f.input.clone();
    linear.bytes.remove(&2);
    linear.bytes.remove(&3);
    linear.bytes.remove(&4);
    linear.association.retain(|(id, _)| *id == 1);
    let recipe = descriptor(f.triple);
    let witness = InputWitness::new(vec![recipe.leaf, recipe.control]);
    let mut parent = address(&f.events[&1]);
    for id in 10..13 {
        let next = tx(
            &[(parent, 0)],
            vec![output(
                f.triple,
                recipe.output_program.clone(),
                NonceField::Null,
            )],
            vec![witness.clone()],
            id,
        );
        add_event(&mut linear, id, &next);
        parent = address(&next);
    }
    let linear_reading = reader::read(&linear).unwrap();
    tracing::info!(target: "vectors::keyed_fixed_history", events = linear.bytes.len(), edges = linear_reading.edges(), bytes = linear_reading.event_bytes(), "linear seed workload");
    assert_eq!(linear.bytes.len(), 4);
    assert_eq!(linear_reading.edges(), 4);
    assert_eq!(linear_reading.event_bytes(), 955);
    assert_eq!(
        records(&linear_reading, handle(f.triple, 12)),
        BTreeSet::from([handle(f.triple, 1)])
    );
}
