//! Decode and index once. Graph consistency precedes content authentication.
use super::{Address, ByteSource, EventId, Input, Refusal};
use std::collections::{BTreeMap, BTreeSet};
use target_elements_conformance::constructor::tagged::sha256;
use transaction::TargetTransaction;
use transaction::TransactionRefusal;

pub struct Index {
    pub names: BTreeMap<EventId, Address>,
    pub aliases: BTreeMap<Address, EventId>,
    pub transactions: BTreeMap<EventId, TargetTransaction>,
    pub parents: BTreeMap<Address, TargetTransaction>,
    pub order: Vec<EventId>,
}

fn decode(subject: ByteSource, bytes: &[u8]) -> Result<TargetTransaction, Refusal> {
    let transaction = TargetTransaction::decode(bytes).map_err(|source| match source {
        TransactionRefusal::IssuanceInputRefused
        | TransactionRefusal::PeginInputRefused
        | TransactionRefusal::IssuanceProofRefused
        | TransactionRefusal::SurjectionProofRefused
        | TransactionRefusal::RangeProofRefused => Refusal::UnsupportedForm { subject, source },
        _ => Refusal::MalformedBytes { subject, source },
    })?;
    if transaction.encode() != bytes {
        return Err(Refusal::NonCanonical(subject));
    }
    Ok(transaction)
}

pub fn address(transaction: &TargetTransaction) -> Address {
    Address(sha256(&sha256(&transaction.encode_without_witness())))
}

fn names(input: &Input) -> Result<BTreeMap<EventId, Address>, Refusal> {
    let mut groups: BTreeMap<_, Vec<_>> = BTreeMap::new();
    for (id, address) in &input.association {
        groups.entry(*id).or_default().push(*address);
    }
    let mut names = BTreeMap::new();
    for (id, group) in groups {
        if group.len() > 1 {
            return Err(if group.iter().all(|value| *value == group[0]) {
                Refusal::DuplicateEvent(id)
            } else {
                Refusal::ConflictingEvent(id)
            });
        }
        names.insert(id, group[0]);
    }
    let mut addresses = BTreeSet::new();
    for address in names.values() {
        if !addresses.insert(*address) {
            return Err(Refusal::ConflictingAlias(*address));
        }
    }
    Ok(names)
}

pub fn load(input: &Input) -> Result<Index, Refusal> {
    let names = names(input)?;
    let mut transactions = BTreeMap::new();
    let mut parents = BTreeMap::new();
    for (id, bytes) in &input.bytes {
        let offered = names.get(id).ok_or(Refusal::UnassociatedBytes(*id))?;
        let decoded = decode(ByteSource::Event(*id), bytes)?;
        parents.insert(*offered, decoded.clone());
        transactions.insert(*id, decoded);
    }
    for (offered, bytes) in &input.retrieved {
        if let Some(existing) = parents.get(offered) {
            if existing.encode() != *bytes {
                return Err(Refusal::ConflictingParent(*offered));
            }
        } else {
            parents.insert(*offered, decode(ByteSource::Retrieved(*offered), bytes)?);
        }
    }
    let aliases = names.iter().map(|(id, address)| (*address, *id)).collect();
    let order = topology(&transactions, &parents, &aliases)?;
    for (id, decoded) in &transactions {
        check_address(ByteSource::Event(*id), names[id], decoded)?;
    }
    for offered in input.retrieved.keys() {
        check_address(ByteSource::Retrieved(*offered), *offered, &parents[offered])?;
    }
    Ok(Index {
        names,
        aliases,
        transactions,
        parents,
        order,
    })
}

fn check_address(
    subject: ByteSource,
    offered: Address,
    transaction: &TargetTransaction,
) -> Result<(), Refusal> {
    let computed = address(transaction);
    if offered != computed {
        return Err(Refusal::ContentAddress {
            subject,
            offered,
            computed,
        });
    }
    Ok(())
}

fn topology(
    transactions: &BTreeMap<EventId, TargetTransaction>,
    parents: &BTreeMap<Address, TargetTransaction>,
    aliases: &BTreeMap<Address, EventId>,
) -> Result<Vec<EventId>, Refusal> {
    for (event, tx) in transactions {
        for (input, row) in tx.inputs().iter().enumerate() {
            let point = row.outpoint();
            if let Some(parent) = parents.get(&Address(*point.txid().internal()))
                && usize::try_from(point.index())
                    .ok()
                    .and_then(|i| parent.outputs().get(i))
                    .is_none()
            {
                return Err(Refusal::MissingOutput {
                    event: *event,
                    input,
                    address: Address(*point.txid().internal()),
                    position: point.index(),
                });
            }
        }
    }
    let mut spent = BTreeSet::new();
    let mut counts = BTreeMap::new();
    let mut children: BTreeMap<EventId, BTreeSet<EventId>> = BTreeMap::new();
    let mut ready = BTreeSet::new();
    for (event, tx) in transactions {
        let mut dependencies = BTreeSet::new();
        for row in tx.inputs() {
            let point = (
                Address(*row.outpoint().txid().internal()),
                row.outpoint().index(),
            );
            if !spent.insert(point) {
                return Err(Refusal::DuplicateConsumption {
                    address: point.0,
                    position: point.1,
                });
            }
            if let Some(parent) = aliases.get(&point.0)
                && transactions.contains_key(parent)
            {
                dependencies.insert(*parent);
            }
        }
        counts.insert(*event, dependencies.len());
        if dependencies.is_empty() {
            ready.insert(*event);
        }
        for parent in dependencies {
            children.entry(parent).or_default().insert(*event);
        }
    }
    let mut order = Vec::new();
    while let Some(event) = ready.pop_first() {
        order.push(event);
        for child in children.get(&event).into_iter().flatten() {
            let count = counts.get_mut(child).expect("indexed child");
            *count -= 1;
            if *count == 0 {
                ready.insert(*child);
            }
        }
    }
    if order.len() != transactions.len() {
        return Err(Refusal::Cycle);
    }
    Ok(order)
}
