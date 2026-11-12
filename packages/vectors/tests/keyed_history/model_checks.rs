//! Standalone standard-library-only harness. No Cargo externs are provided.
#[path = "../../src/keyed_history_model.rs"]
mod model;

use model::{Boundary, Event, Handle, Output, OutputStatus, Refusal, analyze};
use std::collections::BTreeSet;
use std::io::Write;

fn output(event: u32, position: usize) -> Output {
    Output { event, position }
}
fn handle(key: u8, event: u32) -> Handle {
    Handle {
        key: [key; 32],
        output: output(event, 0),
    }
}
fn event(id: u32, inputs: &[u32], key: u8) -> Event {
    Event {
        id,
        inputs: inputs.iter().map(|id| output(*id, 0)).collect(),
        outputs: vec![Some([key; 32]), None],
    }
}

fn statements() {
    let graph = vec![
        event(1, &[], 7),
        event(2, &[], 7),
        event(3, &[1, 2], 7),
        event(4, &[3], 7),
        event(5, &[], 8),
    ];
    let history = analyze(&graph).unwrap();
    let one = BTreeSet::from([handle(7, 1)]);
    let union = BTreeSet::from([handle(7, 1), handle(7, 2)]);
    assert_eq!(history.records(handle(7, 1)), Some(&one));
    assert_eq!(history.records(handle(7, 3)), Some(&union));
    assert_eq!(history.records(handle(7, 4)), Some(&union));
    assert!(
        !history
            .records(handle(7, 3))
            .unwrap()
            .contains(&handle(7, 3)),
        "MERGE introduces no record"
    );
    assert!(one.is_subset(history.records(handle(7, 4)).unwrap()));
    assert_eq!(history.successor(handle(7, 1)), Some(handle(7, 3)));
    assert_eq!(history.successor(handle(7, 3)), Some(handle(7, 4)));
    assert_eq!(
        history.live(),
        &BTreeSet::from([handle(7, 4), handle(8, 5)])
    );
    assert!(
        history
            .records(handle(7, 4))
            .unwrap()
            .is_disjoint(history.records(handle(8, 5)).unwrap())
    );
    assert_eq!(history.order(), &[1, 2, 3, 4, 5]);
    let mut permutation = graph.clone();
    permutation.reverse();
    assert_eq!(analyze(&permutation), Ok(history));
    // Equal symbolic local names in separate keyed histories remain distinct.
    let other = analyze(&[event(1, &[], 8)]).unwrap();
    assert_eq!(
        other.records(handle(8, 1)),
        Some(&BTreeSet::from([handle(8, 1)]))
    );
    assert_ne!(handle(7, 1), handle(8, 1));
    assert_eq!(other.records(handle(7, 1)), None);
    assert_eq!(
        other.query(handle(8, 1)),
        OutputStatus::Handle(handle(8, 1))
    );
    assert_eq!(other.query(handle(7, 1)), OutputStatus::WrongKey([8; 32]));
    assert_eq!(other.query(handle(8, 99)), OutputStatus::MissingEvent);
    assert_eq!(
        other.query(Handle {
            key: [8; 32],
            output: output(1, 2)
        }),
        OutputStatus::MissingOutput
    );
    assert_eq!(
        other.query(Handle {
            key: [8; 32],
            output: output(1, 1)
        }),
        OutputStatus::NotHandle
    );
    // A focused changed dependency alters the union: this comparison can fail.
    let mut changed = graph;
    changed[2].inputs.pop();
    assert_eq!(analyze(&changed).unwrap().records(handle(7, 4)), Some(&one));
    assert_ne!(one, union);
}

fn refusals() {
    let control = vec![event(1, &[], 7), event(2, &[1], 7)];
    assert!(analyze(&control).is_ok());
    let mut subject = control.clone();
    subject.push(control[0].clone());
    assert_eq!(analyze(&subject), Err(Refusal::DuplicateEvent(1)));
    assert_eq!(
        analyze(&subject).unwrap_err().boundary(),
        Boundary::ValidHistory
    );
    subject[2].outputs[1] = Some([7; 32]);
    assert_eq!(analyze(&subject), Err(Refusal::ConflictingEvent(1)));
    assert_eq!(
        analyze(&subject).unwrap_err().boundary(),
        Boundary::ValidHistory
    );
    subject = control.clone();
    subject[1].inputs.push(output(1, 0));
    assert_eq!(
        analyze(&subject),
        Err(Refusal::DuplicateConsumption(output(1, 0)))
    );
    assert_eq!(
        Refusal::DuplicateConsumption(output(1, 0)).boundary(),
        Boundary::ValidHistory
    );
    subject = control.clone();
    subject.push(event(3, &[1], 7));
    assert_eq!(
        analyze(&subject),
        Err(Refusal::DuplicateConsumption(output(1, 0)))
    );
    subject = control.clone();
    subject[0].inputs.push(output(2, 0));
    assert_eq!(analyze(&subject), Err(Refusal::Cycle));
    assert_eq!(Refusal::Cycle.boundary(), Boundary::Graph);
    subject = control.clone();
    subject[1].inputs[0].event = 99;
    assert_eq!(analyze(&subject), Err(Refusal::MissingEvent(output(99, 0))));
    assert_eq!(analyze(&subject).unwrap_err().boundary(), Boundary::Graph);
    subject = control.clone();
    subject[1].inputs[0].position = 2;
    assert_eq!(analyze(&subject), Err(Refusal::MissingOutput(output(1, 2))));
    assert_eq!(
        analyze(&subject).unwrap_err().boundary(),
        Boundary::ValidHistory
    );
    subject = control.clone();
    subject[1].outputs[0] = Some([8; 32]);
    assert_eq!(
        analyze(&subject),
        Err(Refusal::WrongSuccessor {
            input: handle(7, 1),
            spender: 2
        })
    );
    assert_eq!(
        analyze(&subject).unwrap_err().boundary(),
        Boundary::ContinueFirst
    );
    subject = control.clone();
    subject[0].outputs[1] = Some([7; 32]);
    let accepted = analyze(&subject).unwrap();
    assert_eq!(
        accepted.records(Handle {
            key: [7; 32],
            output: output(1, 1)
        }),
        None
    );
    assert!(accepted.records(handle(7, 1)).is_some());
    subject[1].inputs[0].position = 1;
    assert_eq!(
        analyze(&subject),
        Err(Refusal::LaterOutputConsumed(output(1, 1)))
    );
    assert_eq!(
        analyze(&subject).unwrap_err().boundary(),
        Boundary::BornFirst
    );
    // Ordinary later outputs have no covenant or record set, but can be spent.
    subject[0].outputs[1] = None;
    let ordinary = analyze(&subject).unwrap();
    assert_eq!(
        ordinary.records(handle(7, 2)),
        Some(&BTreeSet::from([handle(7, 2)]))
    );
    assert_eq!(
        ordinary.live(),
        &BTreeSet::from([handle(7, 1), handle(7, 2)])
    );

    // Each edge case has an accepted twin and an exact boundary.
    let mut mixed = control.clone();
    mixed.push(event(5, &[], 7));
    mixed[1].inputs.push(output(5, 0));
    assert!(analyze(&mixed).is_ok());
    mixed[2].outputs[0] = Some([8; 32]);
    assert_eq!(
        analyze(&mixed),
        Err(Refusal::WrongSuccessor {
            input: handle(8, 5),
            spender: 2
        })
    );
    assert_eq!(
        analyze(&mixed).unwrap_err().boundary(),
        Boundary::ContinueFirst
    );
    mixed[1].outputs[0] = Some([8; 32]);
    assert_eq!(
        analyze(&mixed),
        Err(Refusal::WrongSuccessor {
            input: handle(7, 1),
            spender: 2
        })
    );
    assert_eq!(
        analyze(&mixed).unwrap_err().boundary(),
        Boundary::ContinueFirst
    );
    for first in [vec![None, None], Vec::new()] {
        let mut changed = control.clone();
        assert!(analyze(&changed).is_ok());
        changed[1].outputs = first;
        assert_eq!(
            analyze(&changed),
            Err(Refusal::WrongSuccessor {
                input: handle(7, 1),
                spender: 2
            })
        );
        assert_eq!(
            analyze(&changed).unwrap_err().boundary(),
            Boundary::ContinueFirst
        );
    }
    let mut self_edge = vec![event(1, &[], 7)];
    assert!(analyze(&self_edge).is_ok());
    self_edge[0].inputs.push(output(1, 0));
    assert_eq!(analyze(&self_edge), Err(Refusal::Cycle));
    assert_eq!(analyze(&self_edge).unwrap_err().boundary(), Boundary::Graph);
    let empty = Event {
        id: 1,
        inputs: Vec::new(),
        outputs: Vec::new(),
    };
    assert!(
        analyze(std::slice::from_ref(&empty))
            .unwrap()
            .live()
            .is_empty()
    );
    assert_eq!(
        analyze(&[empty, event(2, &[1], 7)]),
        Err(Refusal::MissingOutput(output(1, 0)))
    );

    let mut two_faults = vec![
        event(1, &[], 7),
        event(2, &[1], 8),
        event(3, &[4], 7),
        event(4, &[3], 7),
    ];
    assert_eq!(analyze(&two_faults), Err(Refusal::Cycle));
    two_faults.reverse();
    assert_eq!(analyze(&two_faults), Err(Refusal::Cycle));
    two_faults.retain(|event| event.id < 3);
    assert_eq!(
        analyze(&two_faults),
        Err(Refusal::WrongSuccessor {
            input: handle(7, 1),
            spender: 2
        })
    );
    let mut duplicates = vec![
        event(1, &[], 7),
        event(1, &[], 8),
        event(2, &[], 7),
        event(2, &[], 7),
    ];
    assert_eq!(analyze(&duplicates), Err(Refusal::ConflictingEvent(1)));
    duplicates.reverse();
    assert_eq!(analyze(&duplicates), Err(Refusal::ConflictingEvent(1)));
}

fn main() {
    let start = std::time::Instant::now();
    statements();
    refusals();
    writeln!(
        std::io::stdout().lock(),
        "model statements and refusal controls passed wall={:.6}s",
        start.elapsed().as_secs_f64()
    )
    .unwrap();
}
