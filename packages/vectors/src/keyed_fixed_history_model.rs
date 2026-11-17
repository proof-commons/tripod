//! Independent symbolic graph oracle: no transaction bytes or reader calls.
use std::collections::{BTreeMap, BTreeSet};

pub type EventId = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Triple {
    pub key: [u8; 32],
    pub asset: [u8; 32],
    pub value: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Output {
    pub event: EventId,
    pub position: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Handle {
    pub triple: Triple,
    pub output: Output,
}

/// Declarations are independent premises about a finite history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub id: EventId,
    pub inputs: Vec<Output>,
    pub outputs: Vec<Option<Triple>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    DuplicateEvent(EventId),
    ConflictingEvent(EventId),
    MissingEvent(Output),
    MissingOutput(Output),
    DuplicateConsumption(Output),
    Cycle,
    LaterOutputConsumed(Output),
    WrongSuccessor { input: Handle, spender: EventId },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct History {
    sets: BTreeMap<Handle, BTreeSet<Handle>>,
    live: BTreeSet<Handle>,
    successors: BTreeMap<Handle, Handle>,
    order: Vec<EventId>,
}

impl History {
    #[must_use]
    pub const fn sets(&self) -> &BTreeMap<Handle, BTreeSet<Handle>> {
        &self.sets
    }
    #[must_use]
    pub const fn live(&self) -> &BTreeSet<Handle> {
        &self.live
    }
    #[must_use]
    pub const fn successors(&self) -> &BTreeMap<Handle, Handle> {
        &self.successors
    }
    #[must_use]
    pub fn order(&self) -> &[EventId] {
        &self.order
    }
}

/// Apply the paper recurrence to a declared acyclic graph.
/// # Errors
/// Refuses invalid graph identity, consumption or first-output clauses.
/// # Panics
/// Panics only if a dependency disappears after this function validates the immutable graph.
pub fn analyze(events: &[Event]) -> Result<History, Refusal> {
    let declared = declarations(events)?;
    let order = topology(&declared)?;
    let mut history = History {
        sets: BTreeMap::new(),
        live: BTreeSet::new(),
        successors: BTreeMap::new(),
        order,
    };
    for id in &history.order {
        let event = declared[id];
        let first = event.outputs.first().copied().flatten();
        let mut records = BTreeSet::new();
        let mut members = Vec::new();
        for output in &event.inputs {
            if let Some(triple) = declared[&output.event].outputs[output.position] {
                if output.position != 0 {
                    return Err(Refusal::LaterOutputConsumed(*output));
                }
                let member = Handle {
                    triple,
                    output: *output,
                };
                if first != Some(triple) {
                    return Err(Refusal::WrongSuccessor {
                        input: member,
                        spender: *id,
                    });
                }
                records.extend(history.sets.get(&member).expect("folded dependency"));
                members.push(member);
            }
        }
        if let Some(triple) = first {
            let handle = Handle {
                triple,
                output: Output {
                    event: *id,
                    position: 0,
                },
            };
            if members.is_empty() {
                records.insert(handle);
            }
            for member in members {
                history.live.remove(&member);
                history.successors.insert(member, handle);
            }
            history.sets.insert(handle, records);
            history.live.insert(handle);
        }
    }
    Ok(history)
}

fn declarations(events: &[Event]) -> Result<BTreeMap<EventId, &Event>, Refusal> {
    let mut groups: BTreeMap<EventId, Vec<&Event>> = BTreeMap::new();
    for event in events {
        groups.entry(event.id).or_default().push(event);
    }
    let mut declared = BTreeMap::new();
    for (id, group) in groups {
        if group.len() != 1 {
            return Err(if group.iter().all(|event| *event == group[0]) {
                Refusal::DuplicateEvent(id)
            } else {
                Refusal::ConflictingEvent(id)
            });
        }
        declared.insert(id, group[0]);
    }
    let mut consumed = BTreeSet::new();
    for event in declared.values() {
        for output in &event.inputs {
            let Some(parent) = declared.get(&output.event) else {
                return Err(Refusal::MissingEvent(*output));
            };
            if output.position >= parent.outputs.len() {
                return Err(Refusal::MissingOutput(*output));
            }
            if !consumed.insert(*output) {
                return Err(Refusal::DuplicateConsumption(*output));
            }
        }
    }
    Ok(declared)
}

fn topology(declared: &BTreeMap<EventId, &Event>) -> Result<Vec<EventId>, Refusal> {
    let mut remaining = BTreeMap::new();
    let mut children: BTreeMap<EventId, BTreeSet<EventId>> = BTreeMap::new();
    let mut ready = BTreeSet::new();
    for event in declared.values() {
        let parents: BTreeSet<_> = event.inputs.iter().map(|input| input.event).collect();
        remaining.insert(event.id, parents.len());
        if parents.is_empty() {
            ready.insert(event.id);
        }
        for parent in parents {
            children.entry(parent).or_default().insert(event.id);
        }
    }
    let mut order = Vec::new();
    while let Some(id) = ready.pop_first() {
        order.push(id);
        for child in children.get(&id).into_iter().flatten() {
            let remaining = remaining.get_mut(child).expect("indexed child");
            *remaining -= 1;
            if *remaining == 0 {
                ready.insert(*child);
            }
        }
    }
    if order.len() != declared.len() {
        return Err(Refusal::Cycle);
    }
    Ok(order)
}
