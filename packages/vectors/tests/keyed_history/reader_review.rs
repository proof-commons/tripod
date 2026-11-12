//! Review controls: byte-derived evidence beside declared-model subjects.
use super::reader::{changed, edit, incomplete, input, observe, point};
use super::{fixture, model};
use std::collections::BTreeSet;
use vectors::keyed_history_reader::{self as r, Address, Key, Membership, Missing, Query, Refusal};

pub(super) fn controls() {
    let source = fixture::build(fixture::graph("source"));
    let multi = fixture::build(fixture::graph("multi"));
    query_and_unexamined(&source);
    classification(&source);
    counters(&multi);
    mapped(&multi);
    precedence(&multi);
}
const fn h(event: u32) -> r::Handle {
    r::Handle {
        key: Key([42; 32]),
        output: r::Output { event, position: 0 },
    }
}
fn singleton(reading: &r::Reading, event: u32) {
    assert_eq!(
        reading.query(h(event)),
        Query::Handle {
            handle: h(event),
            membership: Membership::Complete(BTreeSet::from([h(event)]))
        }
    );
}
fn query_and_unexamined(fixture: &fixture::Fixture) {
    let accepted = input(fixture.reader_inputs());
    singleton(&observe(&accepted).unwrap(), 10);
    let mut subject = accepted.clone();
    subject.keys.clear();
    let result = observe(&subject).unwrap();
    assert_eq!(result.query(h(10)), Query::UnresolvedKey(h(10).output));
    assert!(result.memberships().is_empty());
    assert!(result.live().is_empty());
    assert_eq!(result.unexamined(), &[(10, BTreeSet::new())].into());
    let mut subject = accepted;
    edit(
        &mut subject,
        10,
        |tx| {
            let mut outputs = tx.outputs().to_vec();
            outputs.swap(0, 1);
            changed(tx, tx.inputs().to_vec(), outputs, tx.witnesses().to_vec())
        },
        true,
    );
    assert!(observe(&subject).unwrap().unexamined()[&10].is_empty());
    subject.retrieved.clear();
    let result = observe(&subject).unwrap();
    assert!(result.memberships().is_empty());
    assert!(matches!(
        result.unexamined()[&10].iter().next(),
        Some(Missing::Parent {
            event: 10,
            input: 0,
            ..
        })
    ));
    assert_eq!(result.unexamined()[&10].len(), 1);
    assert_eq!(result.query(h(10)), Query::NotHandle);
}
/// Replaces only the public funding output program and follows its new address.
fn funding(accepted: &r::Input, program: Vec<u8>) -> r::Input {
    let mut subject = accepted.clone();
    let (old, bytes) = subject.retrieved.iter().next().unwrap();
    let old = *old;
    let parent = transaction::TargetTransaction::decode(bytes).unwrap();
    let mut outputs = parent.outputs().to_vec();
    let row = &outputs[1];
    outputs[1] = transaction::TargetOutput::new(row.asset(), row.value(), row.nonce(), program);
    let bytes = changed(
        &parent,
        parent.inputs().to_vec(),
        outputs,
        parent.witnesses().to_vec(),
    )
    .encode();
    let address = Address(fixture::replay::txid(&bytes));
    subject.retrieved.remove(&old);
    subject.retrieved.insert(address, bytes);
    subject.assertions.inclusion.insert(address);
    subject.assertions.retrieval.insert(address);
    edit(
        &mut subject,
        10,
        |tx| {
            changed(
                tx,
                vec![point(&tx.inputs()[0], address, 1)],
                tx.outputs().to_vec(),
                tx.witnesses().to_vec(),
            )
        },
        true,
    );
    subject
}
fn classification(fixture: &fixture::Fixture) {
    let accepted = input(fixture.reader_inputs());
    let mut ordinary = vec![0x51, 32];
    ordinary.extend([9; 32]);
    let subject = funding(&accepted, ordinary);
    singleton(&observe(&subject).unwrap(), 10);
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let recipe = vectors::keyed_constructor::construct_keyed(
        &target,
        tapscript::keyed_constructor::KeyedSemanticKey::new([43; 32]),
        &vectors::keyed_constructor::PublicKeyedCurve,
    )
    .unwrap();
    let mut subject = funding(&accepted, recipe.output_program);
    // Put the other-key output at its birth position; retain all other coordinates.
    let address = *subject.retrieved.keys().next().unwrap();
    let parent = transaction::TargetTransaction::decode(&subject.retrieved[&address]).unwrap();
    let mut outputs = parent.outputs().to_vec();
    outputs.swap(0, 1);
    let bytes = changed(
        &parent,
        parent.inputs().to_vec(),
        outputs,
        parent.witnesses().to_vec(),
    )
    .encode();
    let new_address = Address(fixture::replay::txid(&bytes));
    subject.retrieved.remove(&address);
    subject.retrieved.insert(new_address, bytes);
    subject.assertions.inclusion.insert(new_address);
    subject.assertions.retrieval.insert(new_address);
    edit(
        &mut subject,
        10,
        |tx| {
            changed(
                tx,
                vec![point(&tx.inputs()[0], new_address, 0)],
                tx.outputs().to_vec(),
                tx.witnesses().to_vec(),
            )
        },
        true,
    );
    singleton(&observe(&subject).unwrap(), 10);
    // The model knows the omitted key; inclusion is the reader's stated limit.
    let graph = vec![fixture::event(9, &[], 43), fixture::event(10, &[9], 42)];
    assert_eq!(
        model::analyze(&graph).unwrap_err(),
        model::Refusal::WrongSuccessor {
            input: fixture::handle(43, 9),
            spender: 10
        }
    );
    subject.keys.insert(Key([43; 32]));
    assert_eq!(
        observe(&subject),
        Err(Refusal::WrongSuccessor {
            event: 10,
            input: 0
        })
    );
}
fn counters(fixture: &fixture::Fixture) {
    let accepted = input(fixture.reader_inputs());
    let control = observe(&accepted).unwrap();
    let mut unused = accepted.clone();
    unused.keys.insert(Key([43; 32]));
    assert_eq!(observe(&unused), Ok(control.clone()));
    let mut reordered = accepted.clone();
    reordered.retrieved = accepted
        .retrieved
        .iter()
        .rev()
        .map(|(k, v)| (*k, v.clone()))
        .collect();
    assert_eq!(observe(&reordered), Ok(control.clone()));
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let birth = tapscript::keyed_pattern::born_first_fragment(&target)
        .unwrap()
        .encode(&target);
    let continuation = tapscript::keyed_pattern::continue_first_fragment(&target)
        .unwrap()
        .encode(&target);
    for row in control.relations() {
        let (bytes, count, peak) = match row.relation {
            r::Relation::BornFirst => (&row.leaf[34..44], 5, 3),
            r::Relation::ContinueFirst => (&row.leaf[44..51], 6, 4),
        };
        assert_eq!(
            bytes,
            if row.relation == r::Relation::BornFirst {
                birth.as_slice()
            } else {
                continuation.as_slice()
            }
        );
        assert_eq!(
            (row.primitive_count, row.peak, row.maximum_width),
            (count, peak, 32)
        );
    }
    assert_eq!(control.leaves().len(), 2);
    for leaf in control.leaves() {
        assert_eq!(
            (leaf.primitive_count, leaf.peak, leaf.maximum_width),
            (12, 4, 32)
        );
    }
}
fn mapped(fixture: &fixture::Fixture) {
    let accepted = input(fixture.reader_inputs());
    let mut graph = fixture.graph.clone();
    graph.push(graph[0].clone());
    let mut subject = accepted.clone();
    subject.association.push(subject.association[0]);
    assert_eq!(
        model::analyze(&graph).unwrap_err(),
        model::Refusal::DuplicateEvent(10)
    );
    assert_eq!(observe(&subject), Err(Refusal::DuplicateEvent(10)));
    graph.last_mut().unwrap().outputs[0] = Some([43; 32]);
    subject.association.last_mut().unwrap().1 = Address([9; 32]);
    assert_eq!(
        model::analyze(&graph).unwrap_err(),
        model::Refusal::ConflictingEvent(10)
    );
    assert_eq!(observe(&subject), Err(Refusal::ConflictingEvent(10)));
    let mut graph = fixture.graph.clone();
    graph.retain(|event| event.id != 10);
    let mut subject = accepted.clone();
    subject.bytes.remove(&10);
    assert_eq!(
        model::analyze(&graph).unwrap_err(),
        model::Refusal::MissingEvent(model::Output {
            event: 10,
            position: 0
        })
    );
    assert_eq!(incomplete(&observe(&subject).unwrap(), 20).len(), 1);
    graph.last_mut().unwrap().outputs[0] = None;
    edit(&mut subject, 20, wrong_program, true);
    assert_eq!(
        model::analyze(&graph).unwrap_err(),
        model::Refusal::MissingEvent(model::Output {
            event: 10,
            position: 0
        })
    );
    assert_eq!(
        observe(&subject),
        Err(Refusal::WrongSuccessor {
            event: 20,
            input: 1
        })
    );
    domains_and_consumption(fixture, &accepted);
    cycles_and_clauses(fixture, &accepted);
}
fn wrong_program(tx: &transaction::TargetTransaction) -> transaction::TargetTransaction {
    let mut outputs = tx.outputs().to_vec();
    let row = &outputs[0];
    outputs[0] = transaction::TargetOutput::new(row.asset(), row.value(), row.nonce(), vec![0x6a]);
    changed(tx, tx.inputs().to_vec(), outputs, tx.witnesses().to_vec())
}
fn domains_and_consumption(fixture: &fixture::Fixture, accepted: &r::Input) {
    for duplicate in [false, true] {
        let mut graph = fixture.graph.clone();
        let edges = &mut graph
            .iter_mut()
            .find(|event| event.id == 20)
            .unwrap()
            .inputs;
        if duplicate {
            edges[1] = edges[0];
        } else {
            edges[0].position = 99;
        }
        let mut subject = accepted.clone();
        edit(
            &mut subject,
            20,
            |tx| {
                let mut rows = tx.inputs().to_vec();
                if duplicate {
                    rows[1] = rows[0].clone();
                } else {
                    rows[0] = point(&rows[0], Address(*rows[0].outpoint().txid().internal()), 99);
                }
                changed(tx, rows, tx.outputs().to_vec(), tx.witnesses().to_vec())
            },
            true,
        );
        let failure = model::analyze(&graph).unwrap_err();
        let observed = observe(&subject).unwrap_err();
        if duplicate {
            assert_eq!(
                failure,
                model::Refusal::DuplicateConsumption(model::Output {
                    event: 10,
                    position: 0
                })
            );
            assert!(matches!(
                observed,
                Refusal::DuplicateConsumption { position: 0, .. }
            ));
        } else {
            assert_eq!(
                failure,
                model::Refusal::MissingOutput(model::Output {
                    event: 10,
                    position: 99
                })
            );
            assert!(matches!(
                observed,
                Refusal::MissingOutput {
                    event: 20,
                    input: 0,
                    position: 99,
                    ..
                }
            ));
        }
    }
    let mut graph = fixture.graph.clone();
    graph.push(fixture::event(21, &[10], 42));
    let mut subject = accepted.clone();
    let tx = transaction::TargetTransaction::decode(&subject.bytes[&20]).unwrap();
    let bytes = transaction::TargetTransaction::new(
        3,
        vec![tx.inputs()[0].clone()],
        tx.outputs().to_vec(),
        tx.lock_time(),
        vec![tx.witnesses()[0].clone()],
    )
    .unwrap()
    .encode();
    subject
        .association
        .push((21, Address(fixture::replay::txid(&bytes))));
    subject.bytes.insert(21, bytes);
    assert_eq!(
        model::analyze(&graph).unwrap_err(),
        model::Refusal::DuplicateConsumption(model::Output {
            event: 10,
            position: 0
        })
    );
    assert!(matches!(
        observe(&subject),
        Err(Refusal::DuplicateConsumption { position: 0, .. })
    ));
}
fn cycles_and_clauses(fixture: &fixture::Fixture, accepted: &r::Input) {
    let mut graph = fixture.graph.clone();
    graph[0].inputs.push(model::Output {
        event: 20,
        position: 0,
    });
    let address = accepted
        .association
        .iter()
        .find(|(id, _)| *id == 20)
        .unwrap()
        .1;
    let mut subject = accepted.clone();
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
    assert_eq!(model::analyze(&graph).unwrap_err(), model::Refusal::Cycle);
    assert_eq!(observe(&subject), Err(Refusal::Cycle));
    let mut graph = fixture.graph.clone();
    graph.last_mut().unwrap().outputs[0] = None;
    let mut subject = accepted.clone();
    edit(&mut subject, 20, wrong_program, true);
    assert_eq!(
        model::analyze(&graph).unwrap_err(),
        model::Refusal::WrongSuccessor {
            input: fixture::handle(42, 10),
            spender: 20
        }
    );
    assert_eq!(
        observe(&subject),
        Err(Refusal::WrongSuccessor {
            event: 20,
            input: 0
        })
    );
    let boundary = fixture::existing(2);
    let mut graph = boundary.graph.clone();
    graph.last_mut().unwrap().inputs[1].position = 1;
    let mut subject = input(boundary.reader_inputs());
    edit(
        &mut subject,
        1000,
        |tx| {
            let mut rows = tx.inputs().to_vec();
            rows[1] = point(&rows[1], Address(*rows[1].outpoint().txid().internal()), 1);
            changed(tx, rows, tx.outputs().to_vec(), tx.witnesses().to_vec())
        },
        true,
    );
    assert_eq!(
        model::analyze(&graph).unwrap_err(),
        model::Refusal::LaterOutputConsumed(model::Output {
            event: 1,
            position: 1
        })
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
fn precedence(fixture: &fixture::Fixture) {
    let accepted = input(fixture.reader_inputs());
    let mut subject = accepted;
    edit(
        &mut subject,
        20,
        |tx| {
            let mut outputs = tx.outputs().to_vec();
            let row = &outputs[0];
            let mut program = row.program().to_vec();
            program[0] = 0;
            outputs[0] =
                transaction::TargetOutput::new(row.asset(), row.value(), row.nonce(), program);
            let mut witnesses = tx.witnesses().to_vec();
            witnesses[0] = transaction::InputWitness::new(vec![]);
            changed(tx, tx.inputs().to_vec(), outputs, witnesses)
        },
        true,
    );
    assert_eq!(
        observe(&subject),
        Err(Refusal::WrongVersion {
            event: 20,
            input: 0
        })
    );
    let graph = vec![
        fixture::event(10, &[], 42),
        fixture::event(11, &[], 42),
        fixture::event(20, &[10], 42),
        fixture::event(21, &[11], 42),
    ];
    let separate = fixture::build(graph);
    let accepted = input(separate.reader_inputs());
    assert!(observe(&accepted).is_ok());
    let mut graph = separate.graph;
    graph[2].outputs[0] = None;
    graph[3].outputs[0] = None;
    let mut subject = accepted;
    edit(&mut subject, 21, wrong_program, true);
    edit(&mut subject, 20, wrong_program, true);
    assert_eq!(
        model::analyze(&graph).unwrap_err(),
        model::Refusal::WrongSuccessor {
            input: fixture::handle(42, 10),
            spender: 20
        }
    );
    assert_eq!(
        observe(&subject),
        Err(Refusal::WrongSuccessor {
            event: 20,
            input: 0
        })
    );
    subject.association.reverse();
    assert_eq!(
        observe(&subject),
        Err(Refusal::WrongSuccessor {
            event: 20,
            input: 0
        })
    );
}
