//! Public-byte fixtures and their association checks, never the expected-set model.
use super::model::{Event, EventId, Handle, History, Key, Output, analyze};
pub(super) use crate::keyed_reproduction::fixture as replay;
use std::collections::{BTreeMap, BTreeSet};
use transaction::{
    AssetField, AssetId, InputWitness, NonceField, TargetInput, TargetOutput, TargetTransaction,
    ValueField,
};

/// Test-local public inputs and constructed bytes for a predeclared graph.
pub(super) struct Fixture {
    pub graph: Vec<Event>,
    pub expected: History,
    pub packets: BTreeMap<EventId, replay::Packet>,
    pub bytes: BTreeMap<EventId, Vec<u8>>,
    pub association: Vec<(EventId, [u8; 32])>,
    pub external_inputs: BTreeMap<EventId, Vec<([u8; 32], u32)>>,
    pub sponsor_premises: BTreeMap<EventId, SponsorPremise>,
    pub boundary_premises: BTreeSet<EventId>,
}

/// Supplied funding bytes and selected output, never authority about membership.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SponsorPremise {
    pub parent: Vec<u8>,
    pub txid: [u8; 32],
    pub vout: u32,
}

/// Explicit caller readings, not observations or a classification result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PolicyReading {
    pub fee: u64,
    pub asset: [u8; 32],
    pub provenance: String,
    pub rates: [u64; 3],
    pub minimums: Vec<u64>,
    pub ceilings: [u64; 3],
    pub package_before: u64,
    pub signature_bound: usize,
}

/// Reader-facing data only: no graph, expected set, packet or classification flag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReaderFixture {
    /// Hypothetical inclusion/retrieval assertions, not chain observations.
    pub assertions: vectors::keyed_history_reader::Assertions,
    pub bytes: BTreeMap<EventId, Vec<u8>>,
    pub association: Vec<(EventId, [u8; 32])>,
    pub keys: BTreeSet<Key>,
    pub retrieved: BTreeMap<[u8; 32], Vec<u8>>,
    pub boundary_premises: BTreeSet<EventId>,
    pub policy_readings: BTreeMap<EventId, PolicyReading>,
}

impl Fixture {
    const fn empty(graph: Vec<Event>, expected: History) -> Self {
        Self {
            graph,
            expected,
            packets: BTreeMap::new(),
            bytes: BTreeMap::new(),
            association: Vec::new(),
            external_inputs: BTreeMap::new(),
            sponsor_premises: BTreeMap::new(),
            boundary_premises: BTreeSet::new(),
        }
    }

    /// Projects public evidence without handing the reader construction answers.
    pub(super) fn reader_inputs(&self) -> ReaderFixture {
        let addresses = self
            .association
            .iter()
            .map(|(_, address)| *address)
            .chain(self.sponsor_premises.values().map(|premise| premise.txid))
            .map(vectors::keyed_history_reader::Address)
            .collect::<BTreeSet<_>>();
        ReaderFixture {
            assertions: vectors::keyed_history_reader::Assertions {
                inclusion: addresses.clone(),
                retrieval: addresses,
            },
            bytes: self.bytes.clone(),
            association: self.association.clone(),
            keys: self.packets.values().map(|packet| packet.key).collect(),
            retrieved: self
                .sponsor_premises
                .values()
                .map(|premise| (premise.txid, premise.parent.clone()))
                .collect(),
            boundary_premises: self.boundary_premises.clone(),
            policy_readings: self
                .packets
                .iter()
                .map(|(id, packet)| {
                    (
                        *id,
                        PolicyReading {
                            fee: packet.fee,
                            asset: packet.asset,
                            provenance: packet.provenance.clone(),
                            rates: packet.rates,
                            minimums: packet.minimums.clone(),
                            ceilings: packet.ceilings,
                            package_before: packet.package_before,
                            signature_bound: packet.signature_bound,
                        },
                    )
                })
                .collect(),
        }
    }
}

/// Association failures are fixture-binding failures, never covenant refusals.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum AssociationRefusal {
    DuplicateName(EventId),
    DuplicateAddress,
    UnknownName(EventId),
    MissingName(EventId),
    MissingBytes(EventId),
    AddressMismatch(EventId),
    InvalidBytes(EventId),
    InputEdges(EventId),
    OutputKeys(EventId),
}

pub(super) fn event(id: EventId, inputs: &[EventId], key: u8) -> Event {
    Event {
        id,
        inputs: inputs
            .iter()
            .map(|id| Output {
                event: *id,
                position: 0,
            })
            .collect(),
        outputs: vec![Some([key; 32]), None, None],
    }
}

pub(super) fn graph(name: &str) -> Vec<Event> {
    match name {
        "source" => vec![event(10, &[], 42)],
        "single" => vec![event(10, &[], 42), event(20, &[10], 42)],
        "ordinary-edge" => {
            let mut next = event(20, &[], 42);
            next.inputs.push(Output {
                event: 10,
                position: 1,
            });
            vec![event(10, &[], 42), next]
        }
        "multi" => vec![
            event(10, &[], 42),
            event(11, &[], 42),
            event(20, &[10, 11], 42),
        ],
        "path" => vec![
            event(10, &[], 42),
            event(11, &[], 42),
            event(20, &[10], 42),
            event(30, &[20, 11], 42),
            event(40, &[30], 42),
        ],
        "two-keys" => vec![
            event(10, &[], 42),
            event(11, &[], 43),
            event(20, &[10], 42),
            event(21, &[11], 43),
        ],
        "wide" => {
            let mut sources: Vec<_> = (10..263).map(|id| event(id, &[], 42)).collect();
            sources.push(event(1000, &(10..263).collect::<Vec<_>>(), 42));
            sources
        }
        "later" => {
            let mut source = event(10, &[], 42);
            source.outputs[1] = Some([42; 32]);
            vec![source]
        }
        _ => panic!("named fixture subject"),
    }
}

fn sponsor_parent(template: &[u8], salt: u32) -> Vec<u8> {
    let old = TargetTransaction::decode(template).unwrap();
    let input = TargetInput::new(
        transaction::Outpoint::new(old.inputs()[0].outpoint().txid(), salt + 1000).unwrap(),
        salt + 31,
    );
    let outputs = [
        1_000_003 + u64::from(salt) * 17,
        1_000_019 + u64::from(salt) * 23,
    ]
    .into_iter()
    .map(|value| {
        TargetOutput::new(
            AssetField::Explicit(AssetId::from_internal([3; 32])),
            ValueField::Explicit(value),
            NonceField::Null,
            old.outputs()[1].program().to_vec(),
        )
    })
    .collect();
    TargetTransaction::new(2, vec![input], outputs, 0, vec![InputWitness::new(vec![])])
        .unwrap()
        .encode()
}

pub(super) fn build(graph: Vec<Event>) -> Fixture {
    // Expectations exist before construction and are not read from any byte.
    let expected = super::timed("model", || analyze(&graph).unwrap());
    let template = super::timed("template", || replay::case(0));
    let mut phases = [std::time::Duration::ZERO; 3];
    let mut fixture = Fixture::empty(graph, expected);
    let mut parents = BTreeMap::new();
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let mut materials = BTreeMap::new();
    for id in fixture.expected.order() {
        let declared = fixture.graph.iter().find(|event| event.id == *id).unwrap();
        let key = declared.outputs[0].unwrap();
        let mut packet = template.clone();
        packet.key = key;
        let input_keys = declared_input_keys(&fixture.graph, declared);
        packet.source = input_keys.iter().all(Option::is_none);
        let commitment = materials.entry((key, packet.source)).or_insert_with(|| {
            transaction::keyed_taproot::commit_keyed_bundle(
                &target,
                &replay::linked(key, packet.source),
            )
            .unwrap()
        });
        packet.inputs.clear();
        let mut total = 0;
        for (position, (input, input_key)) in declared.inputs.iter().zip(&input_keys).enumerate() {
            if let Some(input_key) = input_key {
                assert_eq!(
                    (*input_key, input.position),
                    (key, 0),
                    "declared member is same-key output zero"
                );
            } else {
                assert_eq!(
                    position + 1,
                    declared.inputs.len(),
                    "one graph-local sponsor, last"
                );
            }
            let parent: &TargetTransaction = &parents[&input.event];
            let ValueField::Explicit(value) = parent.outputs()[input.position].value() else {
                panic!("explicit fixture")
            };
            total += value;
            let bytes = fixture.bytes[&input.event].clone();
            packet.inputs.push(replay::Input {
                txid: replay::txid(&bytes),
                parent: bytes,
                vout: u32::try_from(input.position).unwrap(),
                sequence: 200 + *id + u32::try_from(position).unwrap(),
                member: input_key.is_some(),
            });
        }
        let (external, sponsor_value) = append_external_sponsor(
            &mut packet,
            &template,
            *id,
            input_keys.iter().all(Option::is_some),
        );
        total += sponsor_value;
        let sponsor = packet.inputs.last().unwrap();
        fixture.sponsor_premises.insert(
            *id,
            SponsorPremise {
                parent: sponsor.parent.clone(),
                txid: sponsor.txid,
                vout: sponsor.vout,
            },
        );
        fixture.external_inputs.insert(*id, external);
        packet.successor = 1000 + u64::from(*id);
        packet.change = total - packet.successor - packet.fee;
        let structural = declared.outputs[1].is_some();
        let repeat = fixture.graph.len() <= 5 || !packet.source;
        let snapshot = build_packet(&mut packet, structural, commitment, &mut phases, repeat);
        let transaction = TargetTransaction::decode(&snapshot.complete).unwrap();
        fixture
            .association
            .push((*id, replay::txid(&snapshot.complete)));
        fixture.bytes.insert(*id, snapshot.complete);
        fixture.packets.insert(*id, packet);
        parents.insert(*id, transaction);
    }
    phase_totals(phases);
    fingerprint(&fixture.bytes);
    fixture
}

pub(super) fn check_association(
    fixture: &Fixture,
    offers: &[(EventId, [u8; 32])],
) -> Result<(), AssociationRefusal> {
    let mut names = BTreeMap::new();
    let mut addresses = BTreeSet::new();
    for (id, address) in offers {
        if names.insert(*id, *address).is_some() {
            return Err(AssociationRefusal::DuplicateName(*id));
        }
        if !addresses.insert(*address) {
            return Err(AssociationRefusal::DuplicateAddress);
        }
        if !fixture.graph.iter().any(|event| event.id == *id) {
            return Err(AssociationRefusal::UnknownName(*id));
        }
    }
    let mut decoded = BTreeMap::new();
    for event in &fixture.graph {
        let address = names
            .get(&event.id)
            .ok_or(AssociationRefusal::MissingName(event.id))?;
        let bytes = fixture
            .bytes
            .get(&event.id)
            .ok_or(AssociationRefusal::MissingBytes(event.id))?;
        let transaction = TargetTransaction::decode(bytes)
            .map_err(|_| AssociationRefusal::InvalidBytes(event.id))?;
        if transaction.encode() != *bytes {
            return Err(AssociationRefusal::InvalidBytes(event.id));
        }
        if replay::txid(bytes) != *address {
            return Err(AssociationRefusal::AddressMismatch(event.id));
        }
        decoded.insert(event.id, transaction);
    }
    // Shared public leaf recipe plus conformance tweak arithmetic: substrate
    // authentication, not independent recurrence or recipe-correctness evidence.
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let keys: BTreeSet<Key> = fixture
        .graph
        .iter()
        .flat_map(|event| event.outputs.iter().flatten().copied())
        .collect();
    let programs: BTreeMap<_, _> = keys
        .into_iter()
        .map(|key| {
            let descriptor = vectors::keyed_constructor::construct_keyed(
                &target,
                tapscript::keyed_constructor::KeyedSemanticKey::new(key),
                &vectors::keyed_constructor::PublicKeyedCurve,
            )
            .unwrap();
            (key, descriptor.output_program)
        })
        .collect();
    for event in &fixture.graph {
        let transaction = &decoded[&event.id];
        let expected_inputs: Vec<_> = event
            .inputs
            .iter()
            .map(|input| (names[&input.event], u32::try_from(input.position).unwrap()))
            .chain(fixture.external_inputs[&event.id].iter().copied())
            .collect();
        let actual: Vec<_> = transaction
            .inputs()
            .iter()
            .map(|input| {
                (
                    *input.outpoint().txid().internal(),
                    input.outpoint().index(),
                )
            })
            .collect();
        if expected_inputs != actual {
            return Err(AssociationRefusal::InputEdges(event.id));
        }
        let actual_keys: Vec<_> = transaction
            .outputs()
            .iter()
            .map(|output| {
                programs
                    .iter()
                    .find(|(_, program)| program.as_slice() == output.program())
                    .map(|(key, _)| *key)
            })
            .collect();
        if actual_keys != event.outputs {
            return Err(AssociationRefusal::OutputKeys(event.id));
        }
    }
    Ok(())
}

pub(super) const fn handle(key: u8, id: EventId) -> Handle {
    Handle {
        key: [key; 32],
        output: Output {
            event: id,
            position: 0,
        },
    }
}

/// Joins the unchanged reproduction bytes to exact supplied boundary declarations.
pub(super) fn existing(count: usize) -> Fixture {
    // Declare exact boundary-output shapes before generating any public bytes.
    let mut graph: Vec<_> = (0..u32::try_from(count).unwrap())
        .map(|id| Event {
            id,
            inputs: Vec::new(),
            outputs: vec![Some([42; 32]), Some([42; 32])],
        })
        .collect();
    graph.push(event(
        1000,
        &(0..u32::try_from(count).unwrap()).collect::<Vec<_>>(),
        42,
    ));
    let expected = super::timed("model", || analyze(&graph).unwrap());
    let packet = replay::case(count);
    let (snapshot, _) = replay::rebuild(&packet, true);
    assert_eq!(replay::rebuild(&packet, true).0, snapshot);
    let mut bytes = BTreeMap::new();
    let mut association = Vec::new();
    let mut external_inputs = BTreeMap::new();
    let mut boundary_premises = BTreeSet::new();
    for (id, input) in (0..u32::try_from(count).unwrap()).zip(&packet.inputs) {
        let parent = TargetTransaction::decode(&input.parent).unwrap();
        external_inputs.insert(
            id,
            parent
                .inputs()
                .iter()
                .map(|row| (*row.outpoint().txid().internal(), row.outpoint().index()))
                .collect(),
        );
        bytes.insert(id, input.parent.clone());
        association.push((id, input.txid));
        boundary_premises.insert(id);
    }
    let sponsor = packet.inputs.last().unwrap();
    let sponsor_premises = BTreeMap::from([(
        1000,
        SponsorPremise {
            parent: sponsor.parent.clone(),
            txid: sponsor.txid,
            vout: sponsor.vout,
        },
    )]);
    external_inputs.insert(1000, vec![(sponsor.txid, sponsor.vout)]);
    association.push((1000, replay::txid(&snapshot.complete)));
    bytes.insert(1000, snapshot.complete);
    fingerprint(&bytes);
    Fixture {
        graph,
        expected,
        packets: BTreeMap::from([(1000, packet)]),
        bytes,
        association,
        external_inputs,
        sponsor_premises,
        boundary_premises,
    }
}

fn append_external_sponsor(
    packet: &mut replay::Packet,
    template: &replay::Packet,
    id: EventId,
    needed: bool,
) -> (Vec<([u8; 32], u32)>, u64) {
    let mut total = 0;
    let mut external = Vec::new();
    if needed {
        let sponsor = sponsor_parent(&template.inputs[0].parent, id);
        let decoded = TargetTransaction::decode(&sponsor).unwrap();
        let ValueField::Explicit(value) = decoded.outputs()[1].value() else {
            panic!("explicit fixture")
        };
        total = value;
        external.push((replay::txid(&sponsor), 1));
        packet.inputs.push(replay::Input {
            txid: replay::txid(&sponsor),
            parent: sponsor,
            vout: 1,
            sequence: 700 + id,
            member: false,
        });
    }
    (external, total)
}

fn declared_input_keys(graph: &[Event], event: &Event) -> Vec<Option<Key>> {
    event
        .inputs
        .iter()
        .map(|input| {
            graph
                .iter()
                .find(|parent| parent.id == input.event)
                .unwrap()
                .outputs[input.position]
        })
        .collect()
}

fn build_packet(
    packet: &mut replay::Packet,
    structural: bool,
    commitment: &transaction::keyed_taproot::KeyedCommitment,
    phases: &mut [std::time::Duration; 3],
    repeat: bool,
) -> replay::Snapshot {
    if structural {
        packet.witness.clear();
        packet.change_program = commitment.output_program().to_vec();
    } else {
        super::accumulate(&mut phases[0], || {
            replay::record_signature_with_commitment(packet, commitment);
        });
    }
    let (snapshot, _) = super::accumulate(&mut phases[1], || {
        replay::rebuild_with_commitment(packet, !structural, commitment)
    });
    // Reproduction is pinned on small controls and every MERGE, including 253 inputs.
    // Repeating the same SOURCE form 253 times adds no width-specific check.
    if repeat {
        // No signing occurs in this second reconstruction: recorded witness only.
        assert_eq!(
            super::accumulate(&mut phases[2], || replay::rebuild_with_commitment(
                packet,
                !structural,
                commitment
            ))
            .0,
            snapshot
        );
    }
    snapshot
}

fn phase_totals(phases: [std::time::Duration; 3]) {
    for (phase, elapsed) in ["fixture-sign", "fixture-rebuild", "fixture-repeat"]
        .into_iter()
        .zip(phases)
    {
        use std::io::Write;
        writeln!(
            std::io::stdout().lock(),
            "history phase={phase} wall={:.9}s",
            elapsed.as_secs_f64()
        )
        .unwrap();
    }
}

// Before/after measurement evidence, not a regenerated test expectation.
fn fingerprint(bytes: &BTreeMap<EventId, Vec<u8>>) {
    use std::io::Write;
    let mut framed = Vec::new();
    for (id, transaction) in bytes {
        framed.extend_from_slice(&id.to_le_bytes());
        framed.extend_from_slice(&u64::try_from(transaction.len()).unwrap().to_le_bytes());
        framed.extend_from_slice(transaction);
    }
    let digest = target_elements_conformance::constructor::tagged::sha256(&framed);
    writeln!(
        std::io::stdout().lock(),
        "history fixture events={} framed_bytes={} digest={digest:02x?}",
        bytes.len(),
        framed.len()
    )
    .unwrap();
}
