//! Comparisons are outside the library reader; expectations never cross its API.
use super::{fixture, model};
use reader::{Membership, Missing, Query, Refusal};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use vectors::keyed_history_reader as reader;

const fn handle(value: model::Handle) -> reader::Handle {
    reader::Handle {
        key: reader::Key(value.key),
        output: reader::Output {
            event: value.output.event,
            position: value.output.position,
        },
    }
}
/// Only the deliberately answer-free projection crosses this conversion.
pub(super) fn input(fixture: fixture::ReaderFixture) -> reader::Input {
    reader::Input {
        bytes: fixture.bytes,
        association: fixture
            .association
            .into_iter()
            .map(|(id, address)| (id, reader::Address(address)))
            .collect(),
        keys: fixture.keys.into_iter().map(reader::Key).collect(),
        retrieved: fixture
            .retrieved
            .into_iter()
            .map(|(address, bytes)| (reader::Address(address), bytes))
            .collect(),
        boundary_premises: fixture.boundary_premises,
        // Hypothetical fixture assertions only; all four facts remain obligations.
        assertions: fixture.assertions,
    }
}

pub(super) fn compare(fixture: &fixture::Fixture) {
    let projection = fixture.reader_inputs();
    assert_eq!(projection.policy_readings.len(), fixture.packets.len());
    let input = input(projection);
    let observed = observe(&input).unwrap();
    compare_substrate(&input);
    assert_eq!(
        observed.live(),
        &fixture
            .expected
            .live()
            .iter()
            .copied()
            .map(handle)
            .collect()
    );
    let mut successors = BTreeMap::new();
    for event in &fixture.graph {
        for (position, key) in event.outputs.iter().enumerate() {
            for key in [key.unwrap_or([42; 32]), [250; 32]] {
                let query = model::Handle {
                    key,
                    output: model::Output {
                        event: event.id,
                        position,
                    },
                };
                compare_query(fixture, &observed, query);
                if let Some(next) = fixture.expected.successor(query) {
                    successors.insert(handle(query), handle(next));
                }
            }
        }
        compare_query(
            fixture,
            &observed,
            model::Handle {
                key: [42; 32],
                output: model::Output {
                    event: event.id,
                    position: event.outputs.len(),
                },
            },
        );
    }
    compare_query(
        fixture,
        &observed,
        model::Handle {
            key: [42; 32],
            output: model::Output {
                event: u32::MAX,
                position: 0,
            },
        },
    );
    assert_eq!(observed.successors(), &successors);
    assert_eq!(observed.boundary_premises(), &input.boundary_premises);
    for evidence in observed.relations().as_chunks::<2>().0 {
        assert_eq!(evidence[0].relation, reader::Relation::BornFirst);
        assert_eq!(evidence[1].relation, reader::Relation::ContinueFirst);
        assert_eq!(evidence[0].member, evidence[1].member);
        assert_eq!(evidence[0].input, evidence[1].input);
    }
    let count: usize = fixture
        .graph
        .iter()
        .map(|event| {
            event
                .inputs
                .iter()
                .filter(|point| {
                    point.position == 0
                        && fixture
                            .graph
                            .iter()
                            .find(|parent| parent.id == point.event)
                            .unwrap()
                            .outputs[0]
                            .is_some()
                })
                .count()
        })
        .sum();
    assert_eq!(observed.relations().len(), 2 * count);
    compare_relation_identities(fixture, &observed);
    compare_obligations(&input, &observed);
    let mut permuted = input;
    permuted.association.reverse();
    assert_eq!(observe(&permuted), Ok(observed));
}
fn compare_obligations(input: &reader::Input, observed: &reader::Reading) {
    let input_count: usize = input
        .bytes
        .values()
        .map(|bytes| {
            transaction::TargetTransaction::decode(bytes)
                .unwrap()
                .inputs()
                .len()
        })
        .sum();
    assert_eq!(observed.obligations().len(), 4 * input_count);
    let expected_obligations: BTreeSet<_> = input
        .bytes
        .iter()
        .flat_map(|(event, bytes)| {
            let tx: elements::Transaction = elements::encode::deserialize(bytes).unwrap();
            tx.input
                .into_iter()
                .enumerate()
                .flat_map(move |(index, row)| {
                    [
                        reader::ParentFact::Inclusion,
                        reader::ParentFact::Unspentness,
                        reader::ParentFact::Retrieval,
                        reader::ParentFact::Finality,
                    ]
                    .map(|fact| reader::Obligation {
                        event: *event,
                        input: index,
                        address: reader::Address(row.previous_output.txid.to_byte_array()),
                        position: row.previous_output.vout,
                        fact,
                    })
                })
        })
        .collect();
    assert_eq!(observed.obligations(), &expected_obligations);
}

fn compare_query(fixture: &fixture::Fixture, observed: &reader::Reading, query: model::Handle) {
    match fixture.expected.query(query) {
        model::OutputStatus::MissingEvent => assert_eq!(
            observed.query(handle(query)),
            Query::MissingEvent(query.output.event)
        ),
        model::OutputStatus::MissingOutput => assert_eq!(
            observed.query(handle(query)),
            Query::MissingOutput(handle(query).output)
        ),
        model::OutputStatus::NotHandle => {
            assert_eq!(observed.query(handle(query)), Query::NotHandle);
        }
        model::OutputStatus::WrongKey(actual) => assert_eq!(
            observed.query(handle(query)),
            Query::WrongKey {
                offered: handle(query),
                actual: reader::Key(actual)
            }
        ),
        model::OutputStatus::Handle(_) => {
            let Query::Handle { membership, .. } = observed.query(handle(query)) else {
                panic!("recognized handle");
            };
            if fixture.boundary_premises.is_empty() {
                assert_eq!(
                    membership,
                    Membership::Complete(
                        fixture
                            .expected
                            .records(query)
                            .unwrap()
                            .iter()
                            .copied()
                            .map(handle)
                            .collect()
                    )
                );
            } else {
                let Membership::Incomplete(missing) = membership else {
                    panic!("boundary is not SOURCE authority");
                };
                let unresolved: BTreeSet<_> = missing
                    .iter()
                    .map(|value| match value {
                        Missing::Parent {
                            event, input: 0, ..
                        } => *event,
                        _ => panic!("only unsupplied boundary dependencies: {value:?}"),
                    })
                    .collect();
                let expected: BTreeSet<_> = fixture
                    .expected
                    .records(query)
                    .unwrap()
                    .iter()
                    .map(|handle| handle.output.event)
                    .collect();
                assert_eq!(unresolved, expected);
                if query.output.event == 1000 {
                    assert_eq!(unresolved, fixture.boundary_premises);
                }
                let dependencies = expected
                    .into_iter()
                    .map(|event| {
                        let parent: elements::Transaction =
                            elements::encode::deserialize(&fixture.bytes[&event]).unwrap();
                        assert_eq!(parent.input.len(), 1);
                        let point = parent.input[0].previous_output;
                        Missing::Parent {
                            event,
                            input: 0,
                            address: reader::Address(point.txid.to_byte_array()),
                            position: point.vout,
                        }
                    })
                    .collect();
                assert_eq!(missing, dependencies);
            }
        }
    }
}

pub(super) fn edit(
    input: &mut reader::Input,
    event: u32,
    change: impl FnOnce(&transaction::TargetTransaction) -> transaction::TargetTransaction,
    readdress: bool,
) {
    let tx = transaction::TargetTransaction::decode(&input.bytes[&event]).unwrap();
    let bytes = change(&tx).encode();
    if readdress {
        let address = reader::Address(fixture::replay::txid(&bytes));
        input
            .association
            .iter_mut()
            .find(|(id, _)| *id == event)
            .unwrap()
            .1 = address;
        input.assertions.inclusion.insert(address);
        input.assertions.retrieval.insert(address);
    }
    input.bytes.insert(event, bytes);
}
pub(super) fn changed(
    tx: &transaction::TargetTransaction,
    inputs: Vec<transaction::TargetInput>,
    outputs: Vec<transaction::TargetOutput>,
    witnesses: Vec<transaction::InputWitness>,
) -> transaction::TargetTransaction {
    transaction::TargetTransaction::new(tx.version(), inputs, outputs, tx.lock_time(), witnesses)
        .unwrap()
}
pub(super) fn point(
    row: &transaction::TargetInput,
    txid: reader::Address,
    vout: u32,
) -> transaction::TargetInput {
    transaction::TargetInput::new(
        transaction::Outpoint::new(transaction::Txid::from_internal(txid.0), vout).unwrap(),
        row.sequence(),
    )
}
pub(super) fn incomplete(reading: &reader::Reading, event: u32) -> BTreeSet<Missing> {
    let Query::Handle {
        membership: Membership::Incomplete(missing),
        ..
    } = reading.query(handle(fixture::handle(42, event)))
    else {
        panic!("incomplete membership");
    };
    missing
}

pub(super) fn negatives() {
    let fixture = fixture::build(fixture::graph("multi"));
    let accepted = input(fixture.reader_inputs());
    let control = observe(&accepted).unwrap();
    assert!(matches!(
        control.query(handle(fixture::handle(42, 20))),
        Query::Handle {
            membership: Membership::Complete(_),
            ..
        }
    ));
    completeness(&accepted);
    consistency(&accepted);
    continuation(&accepted);
    commitment(&accepted);
    remaining_evidence(&accepted);
    retrieval_consistency(&accepted);
    mixed_fault_precedence(&accepted);
    boundary_subjects();
}
fn completeness(accepted: &reader::Input) {
    let mut subject = accepted.clone();
    subject.bytes.remove(&10);
    let missing = incomplete(&observe(&subject).unwrap(), 20);
    assert!(matches!(
        missing.iter().next(),
        Some(Missing::Parent {
            event: 20,
            input: 0,
            ..
        })
    ));
    assert_eq!(missing.len(), 1);
    subject.boundary_premises.insert(10);
    assert_eq!(incomplete(&observe(&subject).unwrap(), 20), missing);
    for inclusion in [true, false] {
        let mut subject = accepted.clone();
        let address = subject
            .association
            .iter()
            .find(|(id, _)| *id == 10)
            .unwrap()
            .1;
        if inclusion {
            subject.assertions.inclusion.remove(&address);
        } else {
            subject.assertions.retrieval.remove(&address);
        }
        assert_eq!(
            incomplete(&observe(&subject).unwrap(), 20),
            BTreeSet::from([if inclusion {
                Missing::Inclusion(address)
            } else {
                Missing::Retrieval(address)
            }])
        );
    }
}

fn consistency(accepted: &reader::Input) {
    let mut subject = accepted.clone();
    subject.association.push(subject.association[0]);
    assert_eq!(observe(&subject), Err(Refusal::DuplicateEvent(10)));
    subject.association.last_mut().unwrap().1 = reader::Address([9; 32]);
    assert_eq!(observe(&subject), Err(Refusal::ConflictingEvent(10)));
    let mut subject = accepted.clone();
    subject.association[1].1 = subject.association[0].1;
    assert_eq!(
        observe(&subject),
        Err(Refusal::ConflictingAlias(subject.association[0].1))
    );
    let mut subject = accepted.clone();
    subject.bytes.get_mut(&20).unwrap()[0] ^= 1;
    assert!(matches!(
        observe(&subject),
        Err(Refusal::ContentAddress {
            subject: reader::ByteSource::Event(20),
            ..
        })
    ));
    let mut subject = accepted.clone();
    edit(
        &mut subject,
        20,
        |tx| {
            let mut rows = tx.inputs().to_vec();
            rows[1] = rows[0].clone();
            changed(tx, rows, tx.outputs().to_vec(), tx.witnesses().to_vec())
        },
        true,
    );
    assert!(matches!(
        observe(&subject),
        Err(Refusal::DuplicateConsumption { position: 0, .. })
    ));
    let mut subject = accepted.clone();
    edit(
        &mut subject,
        20,
        |tx| {
            let mut rows = tx.inputs().to_vec();
            rows[0] = point(
                &rows[0],
                reader::Address(*rows[0].outpoint().txid().internal()),
                99,
            );
            changed(tx, rows, tx.outputs().to_vec(), tx.witnesses().to_vec())
        },
        true,
    );
    assert!(matches!(
        observe(&subject),
        Err(Refusal::MissingOutput {
            event: 20,
            input: 0,
            position: 99,
            ..
        })
    ));
    consistency_graph(accepted);
}
fn consistency_graph(accepted: &reader::Input) {
    let mut subject = accepted.clone();
    let address = subject
        .association
        .iter()
        .find(|(id, _)| *id == 20)
        .unwrap()
        .1;
    edit(
        &mut subject,
        10,
        |tx| {
            changed(
                tx,
                vec![point(&tx.inputs()[0], address, 0)],
                tx.outputs().to_vec(),
                tx.witnesses().to_vec(),
            )
        },
        false,
    );
    assert_eq!(observe(&subject), Err(Refusal::Cycle));
    // Two-fault subject: cycle plus wrong successor. Graph wins before hashing.
    edit(
        &mut subject,
        20,
        |tx| {
            let mut outputs = tx.outputs().to_vec();
            outputs.swap(0, 1);
            changed(tx, tx.inputs().to_vec(), outputs, tx.witnesses().to_vec())
        },
        false,
    );
    assert_eq!(observe(&subject), Err(Refusal::Cycle));
    subject.association.reverse();
    assert_eq!(observe(&subject), Err(Refusal::Cycle));
    let mut subject = accepted.clone();
    subject.bytes.get_mut(&20).unwrap()[41] |= 0x80;
    assert!(matches!(
        observe(&subject),
        Err(Refusal::UnsupportedForm {
            subject: reader::ByteSource::Event(20),
            ..
        })
    ));
}
fn continuation(accepted: &reader::Input) {
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let other = vectors::keyed_constructor::construct_keyed(
        &target,
        tapscript::keyed_constructor::KeyedSemanticKey::new([43; 32]),
        &vectors::keyed_constructor::PublicKeyedCurve,
    )
    .unwrap();
    for change in 0..4 {
        let mut subject = accepted.clone();
        edit(
            &mut subject,
            20,
            |tx| {
                let mut outputs = tx.outputs().to_vec();
                if change == 2 {
                    outputs.swap(0, 1);
                } else {
                    let row = &outputs[0];
                    let mut program = row.program().to_vec();
                    match change {
                        0 => program.clone_from(&other.output_program),
                        1 => program[0] = 0,
                        _ => program[2] ^= 1,
                    }
                    outputs[0] = transaction::TargetOutput::new(
                        row.asset(),
                        row.value(),
                        row.nonce(),
                        program,
                    );
                }
                changed(tx, tx.inputs().to_vec(), outputs, tx.witnesses().to_vec())
            },
            true,
        );
        assert_eq!(
            observe(&subject),
            Err(if change == 0 || change == 3 {
                Refusal::WrongSuccessor {
                    event: 20,
                    input: 0,
                }
            } else {
                Refusal::WrongVersion {
                    event: 20,
                    input: 0,
                }
            })
        );
    }
}
fn commitment(accepted: &reader::Input) {
    for change in 0..4 {
        let mut subject = accepted.clone();
        edit(
            &mut subject,
            20,
            |tx| {
                let mut witnesses = tx.witnesses().to_vec();
                let mut stack = witnesses[1].stack().to_vec();
                match change {
                    0 => stack[0][1] ^= 1,
                    1 => stack[1][0] ^= 1,
                    2 => stack[1][0] ^= 2,
                    _ => stack[1][1] ^= 1,
                }
                witnesses[1] = transaction::InputWitness::new(stack);
                changed(tx, tx.inputs().to_vec(), tx.outputs().to_vec(), witnesses)
            },
            true,
        );
        let expected = match change {
            0 => Refusal::UncommittedLeaf {
                event: 20,
                input: 1,
            },
            1 => Refusal::WrongParity {
                event: 20,
                input: 1,
            },
            2 => Refusal::ControlVersion {
                event: 20,
                input: 1,
            },
            _ => Refusal::ControlRecipe {
                event: 20,
                input: 1,
            },
        };
        assert_eq!(observe(&subject), Err(expected));
        // Input zero's accepted proof is unchanged by this one-member mutation.
        assert_eq!(
            transaction::TargetTransaction::decode(&subject.bytes[&20])
                .unwrap()
                .witnesses()[0],
            transaction::TargetTransaction::decode(&accepted.bytes[&20])
                .unwrap()
                .witnesses()[0]
        );
    }
}
fn boundary_subjects() {
    let fixture = fixture::existing(2);
    let accepted = input(fixture.reader_inputs());
    let reading = observe(&accepted).unwrap();
    assert_eq!(reading.relations().len(), 4);
    assert_eq!(incomplete(&reading, 1000).len(), 2);
    let mut subject = accepted;
    subject.boundary_premises.clear();
    assert_eq!(
        incomplete(&observe(&subject).unwrap(), 1000),
        incomplete(&reading, 1000)
    );
    edit(
        &mut subject,
        1000,
        |tx| {
            let mut inputs = tx.inputs().to_vec();
            inputs[1] = point(
                &inputs[1],
                reader::Address(*inputs[1].outpoint().txid().internal()),
                1,
            );
            changed(tx, inputs, tx.outputs().to_vec(), tx.witnesses().to_vec())
        },
        true,
    );
    assert!(matches!(
        observe(&subject),
        Err(Refusal::LaterBirth {
            event: 1000,
            input: 1,
            position: 1,
            ..
        })
    ));
}

pub(super) fn trace_calls(fixture: &fixture::Fixture) {
    let mut offered = input(fixture.reader_inputs());
    assert!(observe(&offered).is_ok());
    offered.association.push(offered.association[0]);
    assert_eq!(observe(&offered), Err(Refusal::DuplicateEvent(10)));
}

fn compare_substrate(input: &reader::Input) {
    for (event, bytes) in &input.bytes {
        let reference_tx: elements::Transaction = elements::encode::deserialize(bytes).unwrap();
        let local = transaction::TargetTransaction::decode(bytes).unwrap();
        let offered = input
            .association
            .iter()
            .find(|(id, _)| id == event)
            .unwrap()
            .1;
        assert_eq!(reference_tx.txid().to_byte_array(), offered.0);
        assert_eq!(elements::encode::serialize(&reference_tx), *bytes);
        assert_eq!(reference_tx.input.len(), local.inputs().len());
        assert_eq!(reference_tx.output.len(), local.outputs().len());
        for ((reference, row), witness) in reference_tx
            .input
            .iter()
            .zip(local.inputs())
            .zip(local.witnesses())
        {
            assert_eq!(
                reference.previous_output.txid.to_byte_array(),
                *row.outpoint().txid().internal()
            );
            assert_eq!(reference.previous_output.vout, row.outpoint().index());
            assert_eq!(reference.witness.script_witness, witness.stack());
        }
        for (reference, row) in reference_tx.output.iter().zip(local.outputs()) {
            assert_eq!(reference.script_pubkey.as_bytes(), row.program());
            if let transaction::ValueField::Explicit(value) = row.value() {
                assert_eq!(reference.value.explicit(), Some(value));
            }
        }
    }
}

fn remaining_evidence(accepted: &reader::Input) {
    let mut subject = accepted.clone();
    subject.association.retain(|(id, _)| *id != 10);
    assert_eq!(observe(&subject), Err(Refusal::UnassociatedBytes(10)));
    let mut subject = accepted.clone();
    subject.bytes.insert(20, vec![0]);
    assert!(matches!(
        observe(&subject),
        Err(Refusal::MalformedBytes {
            subject: reader::ByteSource::Event(20),
            ..
        })
    ));
    let mut subject = accepted.clone();
    let parent = subject.bytes.remove(&10).unwrap();
    let txid = reader::Address(fixture::replay::txid(&parent));
    subject.association.retain(|(id, _)| *id != 10);
    // Retrieval-bag label is not a role. Actual outpoints decide the parent use.
    subject.retrieved.insert(txid, parent);
    assert_eq!(
        incomplete(&observe(&subject).unwrap(), 20),
        BTreeSet::from([Missing::Membership {
            event: 20,
            input: 0,
            address: txid
        }])
    );
    let mut subject = accepted.clone();
    edit(
        &mut subject,
        20,
        |tx| {
            let mut witnesses = tx.witnesses().to_vec();
            witnesses[1] = transaction::InputWitness::new(vec![witnesses[1].stack()[0].clone()]);
            changed(tx, tx.inputs().to_vec(), tx.outputs().to_vec(), witnesses)
        },
        true,
    );
    assert_eq!(
        observe(&subject),
        Err(Refusal::WitnessShape {
            event: 20,
            input: 1
        })
    );
    for (flag, source) in [
        (0x80, transaction::TransactionRefusal::IssuanceInputRefused),
        (0x40, transaction::TransactionRefusal::PeginInputRefused),
    ] {
        let mut subject = accepted.clone();
        subject.bytes.get_mut(&20).unwrap()[41] |= flag;
        assert_eq!(
            observe(&subject),
            Err(Refusal::UnsupportedForm {
                subject: reader::ByteSource::Event(20),
                source
            })
        );
    }
}

fn retrieval_consistency(accepted: &reader::Input) {
    let mut subject = accepted.clone();
    let (address, bytes) = subject.retrieved.iter().next().unwrap();
    let address = *address;
    let mut conflicting = bytes.clone();
    conflicting[0] ^= 1;
    // Conflict with an event's already-indexed bytes, independent of a bag label.
    let event_address = subject.association[0].1;
    subject.retrieved.insert(event_address, conflicting);
    assert_eq!(
        observe(&subject),
        Err(Refusal::ConflictingParent(event_address))
    );
    let mut subject = accepted.clone();
    subject.retrieved.get_mut(&address).unwrap()[0] ^= 1;
    assert!(
        matches!(observe(&subject), Err(Refusal::ContentAddress { subject: reader::ByteSource::Retrieved(offered), .. }) if offered == address)
    );
    let mut subject = accepted.clone();
    let parent = subject.bytes.remove(&10).unwrap();
    let txid = reader::Address(fixture::replay::txid(&parent));
    subject.retrieved.insert(txid, parent);
    assert_eq!(
        incomplete(&observe(&subject).unwrap(), 20),
        BTreeSet::from([Missing::Event(10)])
    );
}

fn mixed_fault_precedence(accepted: &reader::Input) {
    let mut subject = accepted.clone();
    subject.bytes.remove(&10);
    assert_eq!(incomplete(&observe(&subject).unwrap(), 20).len(), 1);
    edit(
        &mut subject,
        20,
        |tx| {
            let mut outputs = tx.outputs().to_vec();
            let row = &outputs[0];
            let mut program = row.program().to_vec();
            program[2] ^= 1;
            outputs[0] =
                transaction::TargetOutput::new(row.asset(), row.value(), row.nonce(), program);
            changed(tx, tx.inputs().to_vec(), outputs, tx.witnesses().to_vec())
        },
        true,
    );
    // The other supplied member can decisively refuse even with one missing parent.
    assert_eq!(
        observe(&subject),
        Err(Refusal::WrongSuccessor {
            event: 20,
            input: 1
        })
    );
}

fn compare_relation_identities(fixture: &fixture::Fixture, observed: &reader::Reading) {
    let names: BTreeMap<_, _> = fixture.association.iter().copied().collect();
    let mut expected = BTreeSet::new();
    for event in &fixture.graph {
        let tx: elements::Transaction =
            elements::encode::deserialize(&fixture.bytes[&event.id]).unwrap();
        for edge in &event.inputs {
            let parent = fixture
                .graph
                .iter()
                .find(|parent| parent.id == edge.event)
                .unwrap();
            let Some(key) = parent.outputs[edge.position] else {
                continue;
            };
            let address = names[&edge.event];
            let index = tx
                .input
                .iter()
                .position(|row| {
                    row.previous_output.txid.to_byte_array() == address
                        && usize::try_from(row.previous_output.vout).unwrap() == edge.position
                })
                .unwrap();
            for relation in [reader::Relation::BornFirst, reader::Relation::ContinueFirst] {
                expected.insert((
                    event.id,
                    index,
                    key,
                    address,
                    u32::try_from(edge.position).unwrap(),
                    relation,
                ));
            }
        }
    }
    let actual: BTreeSet<_> = observed
        .relations()
        .iter()
        .map(|row| {
            (
                row.event,
                row.input,
                row.member.key.0,
                row.member.address.0,
                row.member.position,
                row.relation,
            )
        })
        .collect();
    assert_eq!(actual, expected);
    assert_eq!(actual.len(), observed.relations().len());
}

/// Timed reader work, separate from fixture construction and comparison work.
pub(super) fn observe(input: &reader::Input) -> Result<reader::Reading, reader::Refusal> {
    let start = std::time::Instant::now();
    let result = reader::read(input);
    writeln!(
        std::io::stdout().lock(),
        "reader derivation events={} recipes={} wall={:.6}s",
        input.bytes.len(),
        input.keys.len(),
        start.elapsed().as_secs_f64()
    )
    .unwrap();
    result
}
