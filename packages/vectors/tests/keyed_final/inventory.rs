//! Exhaustive classification of the public test packet and reproduced snapshot.
use crate::keyed_reproduction::fixture::{Input, Packet, Snapshot};
use vectors::keyed_final_manifest::{COMPUTED_ITEMS, ComputedItem as C};

fn classified<T: ?Sized>(_: &T, item: C, seen: &mut Vec<C>) {
    assert!(COMPUTED_ITEMS.contains(&item));
    if !seen.contains(&item) {
        seen.push(item);
    }
}

/// No rest patterns: adding a packet, input or snapshot field requires a decision.
pub fn check(packet: &Packet, snapshot: &Snapshot) {
    let Packet {
        key,
        source,
        inputs,
        asset,
        successor,
        change,
        fee,
        change_program,
        provenance,
        rates,
        minimums,
        ceilings,
        package_before,
        signature_bound,
        witness,
    } = packet;
    let Snapshot {
        linked_bytes,
        linked_projection,
        closure,
        commitment,
        receipt,
        coverage,
        evidence,
        stripped,
        complete,
        witnesses,
        obligations,
    } = snapshot;
    let mut seen = Vec::new();
    classified(key, C::Leaf, &mut seen);
    classified(source, C::ConstructionChoices, &mut seen);
    for Input {
        parent,
        txid,
        vout,
        sequence,
        member,
    } in inputs
    {
        classified(parent, C::RequestAssociation, &mut seen);
        classified(txid, C::TransactionIdentities, &mut seen);
        classified(vout, C::RequestAssociation, &mut seen);
        classified(sequence, C::ConstructionChoices, &mut seen);
        classified(member, C::RequestAssociation, &mut seen);
    }
    classified(asset, C::PolicyReading, &mut seen);
    classified(successor, C::ConstructionChoices, &mut seen);
    classified(change, C::ConstructionChoices, &mut seen);
    classified(change_program, C::ConstructionChoices, &mut seen);
    classified(fee, C::PolicyReading, &mut seen);
    classified(provenance, C::PolicyReading, &mut seen);
    classified(rates, C::PolicyReading, &mut seen);
    classified(minimums, C::PolicyReading, &mut seen);
    classified(ceilings, C::PolicyReading, &mut seen);
    classified(package_before, C::PolicyReading, &mut seen);
    classified(signature_bound, C::PolicyReading, &mut seen);
    classified(witness, C::Signatures, &mut seen);
    classified(linked_bytes, C::Leaf, &mut seen);
    classified(linked_projection, C::EmissionAndClosure, &mut seen);
    classified(closure, C::EmissionAndClosure, &mut seen);
    classified(commitment, C::Commitment, &mut seen);
    classified(receipt, C::RequestAssociation, &mut seen);
    classified(coverage, C::RequestAssociation, &mut seen);
    classified(evidence, C::RequestAssociation, &mut seen);
    classified(stripped, C::TransactionIdentities, &mut seen);
    classified(complete, C::TransactionIdentities, &mut seen);
    classified(witnesses, C::Signatures, &mut seen);
    classified(obligations, C::RequestAssociation, &mut seen);
    let tx = transaction::TargetTransaction::decode(complete).unwrap();
    classified(&tx.weight(), C::FinalResources, &mut seen);
    classified(&tx.weight().div_ceil(4), C::FinalResources, &mut seen);
    assert_eq!(seen.len(), 9);
}
