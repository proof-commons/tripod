//! Complete SOURCE singleton or union recurrence from authenticated bytes.
use super::{
    Address, Class, EventId, Handle, Input, Membership, Missing, Obligation, Output, ParentFact,
    Reading, Refusal, authentication, identity::Index,
};
use std::collections::{BTreeMap, BTreeSet};
use transaction::TargetTransaction;

fn class(output: &transaction::TargetOutput, recipes: &authentication::Recipes) -> Class {
    authentication::program_triple(output.program(), recipes).map_or_else(
        || {
            if authentication::witness_version(output.program()) == Some(1) {
                Class::Unresolved
            } else {
                Class::Ordinary
            }
        },
        |triple| {
            if authentication::qualified(output, triple) {
                Class::Qualified(triple)
            } else {
                Class::Unqualified(triple)
            }
        },
    )
}

fn assertions(address: Address, input: &Input, missing: &mut BTreeSet<Missing>) {
    if !input.assertions.inclusion.contains(&address) {
        missing.insert(Missing::Inclusion(address));
    }
    if !input.assertions.retrieval.contains(&address) {
        missing.insert(Missing::Retrieval(address));
    }
}

fn obligations(event: EventId, tx: &TargetTransaction, reading: &mut Reading) {
    for (input, row) in tx.inputs().iter().enumerate() {
        for fact in [
            ParentFact::Inclusion,
            ParentFact::Unspentness,
            ParentFact::Retrieval,
            ParentFact::Finality,
        ] {
            reading.obligations.insert(Obligation {
                event,
                input,
                address: Address(*row.outpoint().txid().internal()),
                position: row.outpoint().index(),
                fact,
            });
        }
    }
}

pub fn reconstruct(
    input: &Input,
    index: &Index,
    recipes: &authentication::Recipes,
) -> Result<Reading, Refusal> {
    let mut reading = Reading {
        triples: input.triples.clone(),
        outputs: index
            .transactions
            .iter()
            .map(|(id, tx)| {
                (
                    *id,
                    tx.outputs()
                        .iter()
                        .map(|output| class(output, recipes))
                        .collect(),
                )
            })
            .collect(),
        memberships: BTreeMap::new(),
        live: BTreeSet::new(),
        successors: BTreeMap::new(),
        obligations: BTreeSet::new(),
        unexamined: BTreeMap::new(),
        transactions: index.transactions.clone(),
        event_bytes: input.bytes.values().map(Vec::len).sum(),
        edges: 0,
    };
    for event in &index.order {
        event_fold(*event, input, index, recipes, &mut reading)?;
    }
    Ok(reading)
}

fn event_fold(
    event: EventId,
    input: &Input,
    index: &Index,
    recipes: &authentication::Recipes,
    reading: &mut Reading,
) -> Result<(), Refusal> {
    let tx = &index.transactions[&event];
    obligations(event, tx, reading);
    reading.edges += tx.inputs().len();
    let mut missing = BTreeSet::new();
    assertions(index.names[&event], input, &mut missing);
    let mut members = Vec::new();
    for (position, row) in tx.inputs().iter().enumerate() {
        let point = row.outpoint();
        let address = Address(*point.txid().internal());
        let Some(parent) = index.parents.get(&address) else {
            missing.insert(Missing::Parent {
                event,
                input: position,
                address,
                position: point.index(),
            });
            continue;
        };
        assertions(address, input, &mut missing);
        let output = &parent.outputs()[usize::try_from(point.index()).expect("checked position")];
        match class(output, recipes) {
            Class::Ordinary | Class::Unresolved => (),
            Class::Unqualified(_) => {
                return Err(Refusal::WrongParentPair {
                    event,
                    input: position,
                });
            }
            Class::Qualified(triple) => {
                if point.index() != 0 {
                    return Err(Refusal::LaterBirth {
                        event,
                        input: position,
                        address,
                        position: point.index(),
                    });
                }
                authentication::member(event, position, tx, &recipes[&triple])?;
                if let Some(parent_event) = index.aliases.get(&address) {
                    let handle = Handle {
                        triple,
                        output: Output {
                            event: *parent_event,
                            position: 0,
                        },
                    };
                    match reading.memberships.get(&handle) {
                        Some(Membership::Incomplete(dependencies)) => {
                            missing.extend(dependencies.iter().cloned());
                        }
                        Some(Membership::Complete(_)) => (),
                        None => {
                            missing.insert(Missing::Event(*parent_event));
                        }
                    }
                    members.push(handle);
                } else {
                    missing.insert(Missing::Membership {
                        event,
                        input: position,
                        address,
                    });
                }
            }
        }
    }
    finish(event, members, missing, reading);
    Ok(())
}

fn finish(event: EventId, members: Vec<Handle>, missing: BTreeSet<Missing>, reading: &mut Reading) {
    if let Some(Class::Qualified(triple)) = reading.outputs[&event].first() {
        let handle = Handle {
            triple: *triple,
            output: Output { event, position: 0 },
        };
        let membership = if missing.is_empty() {
            let mut records = BTreeSet::new();
            if members.is_empty() {
                records.insert(handle);
            }
            for member in &members {
                if let Membership::Complete(set) = reading
                    .memberships
                    .get(member)
                    .expect("completed dependency")
                {
                    records.extend(set);
                }
            }
            Membership::Complete(records)
        } else {
            Membership::Incomplete(missing)
        };
        for member in members {
            reading.live.remove(&member);
            reading.successors.insert(member, handle);
        }
        reading.live.insert(handle);
        reading.memberships.insert(handle, membership);
    } else {
        reading.unexamined.insert(event, missing);
    }
}
