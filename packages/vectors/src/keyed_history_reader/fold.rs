//! Recurrence derived from actual outpoints and authenticated output programs.
use super::{
    Address, Class, EventId, Handle, Input, Member, Membership, Missing, Obligation, Output,
    ParentFact, Reading, Refusal, authentication, identity::Index,
};
use std::collections::{BTreeMap, BTreeSet};
use transaction::TargetTransaction;

pub fn reconstruct(
    input: &Input,
    index: &Index,
    recipes: &authentication::Recipes,
) -> Result<Reading, Refusal> {
    let mut reading = Reading {
        outputs: index
            .transactions
            .iter()
            .map(|(id, tx)| {
                (
                    *id,
                    tx.outputs()
                        .iter()
                        .map(|output| class(output.program(), recipes))
                        .collect(),
                )
            })
            .collect(),
        memberships: BTreeMap::new(),
        live: BTreeSet::new(),
        successors: BTreeMap::new(),
        relations: Vec::new(),
        obligations: BTreeSet::new(),
        unexamined: BTreeMap::new(),
        leaves: Vec::new(),
        boundary_premises: input.boundary_premises.clone(),
    };
    for event in &index.order {
        event_fold(*event, input, index, recipes, &mut reading)?;
    }
    Ok(reading)
}
fn class(program: &[u8], recipes: &authentication::Recipes) -> Class {
    authentication::key(program, recipes).map_or_else(
        || {
            if authentication::witness_version(program) == Some(1) {
                Class::Unresolved
            } else {
                Class::Ordinary
            }
        },
        Class::Keyed,
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

fn event_fold(
    event: EventId,
    input: &Input,
    index: &Index,
    recipes: &authentication::Recipes,
    reading: &mut Reading,
) -> Result<(), Refusal> {
    let tx = &index.transactions[&event];
    obligations(event, tx, reading);
    let facts = authentication::context(index, tx);
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
        // Canonical indexing checked this output before content authentication.
        let output = &parent.outputs()[usize::try_from(point.index()).expect("checked position")];
        match class(output.program(), recipes) {
            // For the supplied key, nonmatching bytes are outside its consumed handles.
            // Excluding an unsupplied key's handle relies on inclusion in one valid
            // history: continue-first would otherwise forbid the supplied successor.
            Class::Ordinary | Class::Unresolved => (),
            Class::Keyed(key) => {
                let member = Member {
                    key,
                    address,
                    position: point.index(),
                };
                let (relations, leaf) =
                    authentication::member(event, position, member, tx, &recipes[&key], &facts)?;
                reading.relations.extend(relations);
                reading.leaves.push(leaf);
                if let Some(parent_event) = index.aliases.get(&address) {
                    let handle = Handle {
                        key,
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
    if let Some(Class::Keyed(key)) = reading.outputs[&event].first() {
        let handle = Handle {
            key: *key,
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
