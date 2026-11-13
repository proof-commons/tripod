//! Parameterized public-input histories; ordinary sponsor parents are premises.
use crate::keyed_reproduction::fixture::{self as prior, Packet, Snapshot};
use std::collections::{BTreeMap, BTreeSet};
use vectors::keyed_history_reader::{Address, Assertions, Input, Key};

/// Declare the same two SOURCE events followed by their MERGE for every key.
///
/// # Panics
/// Panics if the public pipeline cannot construct these explicit offline inputs.
pub fn graph(key: [u8; 32]) -> (Vec<Packet>, Vec<Snapshot>) {
    // Graph edges are fixed before producing bytes: 10 -> 30 and 20 -> 30.
    let mut packets = Vec::new();
    let mut snapshots = Vec::new();
    for salt in [701, 809] {
        let mut packet = prior::case(0);
        packet.key = key;
        let parent = prior::parent(&packet.change_program, salt);
        packet.inputs[0].txid = prior::txid(&parent);
        packet.inputs[0].parent = parent;
        packet.inputs[0].sequence = salt;
        prior::record_signature(&mut packet);
        snapshots.push(prior::rebuild(&packet, true).0);
        packets.push(packet);
    }
    let mut merge = prior::case(2);
    merge.key = key;
    for (row, source) in merge.inputs.iter_mut().zip(&snapshots) {
        row.parent.clone_from(&source.complete);
        row.txid = prior::txid(&row.parent);
    }
    let sponsor = prior::parent(&merge.change_program, 919);
    merge.inputs[2].txid = prior::txid(&sponsor);
    merge.inputs[2].parent = sponsor;
    merge.change = 991_000; // 2 * 1000 + 1_000_000 - successor 1000 - fee 10_000.
    prior::record_signature(&mut merge);
    snapshots.push(prior::rebuild(&merge, true).0);
    packets.push(merge);
    (packets, snapshots)
}

/// Bytes and explicit assertions only; no graph, role or expected set crosses.
#[must_use]
pub fn reader_input(key: [u8; 32], packets: &[Packet], snapshots: &[Snapshot]) -> Input {
    let bytes: BTreeMap<_, _> = [10, 20, 30]
        .into_iter()
        .zip(snapshots.iter().map(|s| s.complete.clone()))
        .collect();
    let association: Vec<_> = bytes
        .iter()
        .map(|(id, b)| (*id, Address(prior::txid(b))))
        .collect();
    let retrieved: BTreeMap<_, _> = packets
        .iter()
        .flat_map(|p| &p.inputs)
        .map(|row| (Address(row.txid), row.parent.clone()))
        .collect();
    let assertions: BTreeSet<_> = association
        .iter()
        .map(|(_, a)| *a)
        .chain(retrieved.keys().copied())
        .collect();
    Input {
        bytes,
        association,
        keys: BTreeSet::from([Key(key)]),
        retrieved,
        boundary_premises: BTreeSet::new(),
        assertions: Assertions {
            inclusion: assertions.clone(),
            retrieval: assertions,
        },
    }
}
