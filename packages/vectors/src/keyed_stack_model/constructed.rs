//! Model-only execution of extracted constructed bytes. No native acceptance.
//! The cross-tree fixture include is test-only and is registered as an excluded
//! test input in vectors/meson.build; moving it must update both consumers.
use super::*;
#[path = "../../tests/keyed_reproduction/fixture.rs"]
mod fixture;
use crate::keyed_constructor::{PublicKeyedCurve, authenticate_keyed_predecessor, construct_keyed};
use target_elements_conformance::constructor::tagged::sha256;
use transaction::keyed_taproot::{
    KeyedAuthenticationRefusal as A, KeyedCommitment, authenticate_keyed_path,
};
use transaction::{TargetOutput, TargetTransaction};

fn program(bytes: &[u8]) -> ProgramFact {
    if let [version, length, payload @ ..] = bytes {
        let version = match *version {
            0 => Some(0),
            0x51..=0x60 => Some(version - 0x50),
            _ => None,
        };
        if let Some(version) = version
            && usize::from(*length) == payload.len()
            && (2..=40).contains(&payload.len())
        {
            return ProgramFact::Witness {
                version,
                payload: payload.to_vec(),
            };
        }
    }
    ProgramFact::Other {
        sha256: sha256(bytes),
    }
}

type Parents = std::collections::BTreeMap<transaction::Txid, TargetTransaction>;

fn parents(packet: &fixture::Packet) -> Parents {
    packet
        .inputs
        .iter()
        .map(|row| {
            (
                transaction::Txid::from_internal(fixture::txid(&row.parent)),
                TargetTransaction::decode(&row.parent).unwrap(),
            )
        })
        .collect()
}

fn context(parents: &Parents, tx: &TargetTransaction) -> Facts {
    // Rehash and decode each public parent once, then resolve actual outpoints.
    let inputs = tx
        .inputs()
        .iter()
        .map(|input| {
            let outpoint = input.outpoint();
            let decoded = &parents[&outpoint.txid()];
            InputFact {
                txid: *outpoint.txid().internal(),
                index: outpoint.index(),
                // The canonical decoder rejects issuance/peg-in and Outpoint excludes
                // flag bits. Zero follows from that domain, not a supplied flag fact.
                flags: 0,
                program: program(
                    decoded.outputs()[usize::try_from(outpoint.index()).unwrap()].program(),
                ),
            }
        })
        .collect();
    Facts {
        enabled: true,
        current: None,
        inputs,
        outputs: tx.outputs().iter().map(|o| program(o.program())).collect(),
    }
}

fn run(
    packet: &fixture::Packet,
    tx: &TargetTransaction,
    commitment: &KeyedCommitment,
    index: usize,
    facts: &Facts,
) -> Result<Result<(), Refusal>, A> {
    let stack = tx.witnesses()[index].stack();
    assert_eq!(
        stack.len(),
        2,
        "no execution arguments or covenant signature"
    );
    authenticate_keyed_path(commitment, &stack[0], &stack[1])?;
    let mut facts = facts.clone();
    facts.current = Some(u32::try_from(index).unwrap());
    Ok(leaf(&stack[0], &facts, &packet.key).map(|_| ()))
}

fn changed_outputs(tx: &TargetTransaction, outputs: Vec<TargetOutput>) -> TargetTransaction {
    // Synthetic decoded-transaction subject, never forged as an admitted artifact.
    TargetTransaction::new(
        tx.version(),
        tx.inputs().to_vec(),
        outputs,
        tx.lock_time(),
        tx.witnesses().to_vec(),
    )
    .unwrap()
}

#[test]
fn constructed_final_leaf_checks_every_member_from_serialized_contexts() {
    for count in [2, 253] {
        let packet = fixture::case(count);
        let (snapshot, commitment) = fixture::rebuild(&packet, true);
        let tx = TargetTransaction::decode(&snapshot.complete).unwrap();
        let parents = parents(&packet);
        let facts = context(&parents, &tx);
        assert_eq!(tx.encode(), snapshot.complete);
        let target = target_elements::reviewed_elements_tapscript().unwrap();
        let oracle = construct_keyed(
            &target,
            tapscript::keyed_constructor::KeyedSemanticKey::new(packet.key),
            &PublicKeyedCurve,
        )
        .unwrap();
        assert_eq!(commitment.output_program(), oracle.output_program);
        assert_eq!(commitment.control().as_slice(), oracle.control.bytes);
        assert_eq!(commitment.leaf(), oracle.leaf_bytes);
        for index in 0..count {
            let outpoint = tx.inputs()[index].outpoint();
            let parent = &parents[&outpoint.txid()];
            authenticate_keyed_predecessor(
                &target,
                &oracle,
                parent.outputs()[usize::try_from(outpoint.index()).unwrap()].program(),
                &PublicKeyedCurve,
            )
            .unwrap();
            assert_eq!(run(&packet, &tx, &commitment, index, &facts), Ok(Ok(())));
        }
        assert!(
            snapshot
                .obligations
                .contains("MemberExecutionAndRelationEvidence")
        );
    }
}

#[test]
fn constructed_successor_mutants_refuse_at_continuation() {
    let packet = fixture::case(2);
    let (snapshot, commitment) = fixture::rebuild(&packet, true);
    let tx = TargetTransaction::decode(&snapshot.complete).unwrap();
    let parents = parents(&packet);
    let facts = context(&parents, &tx);
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let other =
        transaction::keyed_taproot::commit_keyed_bundle(&target, &fixture::linked([43; 32], false))
            .unwrap();
    let mut key = tx.outputs().to_vec();
    let output = &key[0];
    key[0] = TargetOutput::new(
        output.asset(),
        output.value(),
        output.nonce(),
        other.output_program().to_vec(),
    );
    let mut version = tx.outputs().to_vec();
    let mut script = version[0].program().to_vec();
    script[0] = 0; // Same payload; witness version alone differs.
    version[0] = TargetOutput::new(
        version[0].asset(),
        version[0].value(),
        version[0].nonce(),
        script,
    );
    let mut displaced = tx.outputs().to_vec();
    displaced.swap(0, 1);
    for outputs in [key, version, displaced] {
        let subject = changed_outputs(&tx, outputs);
        let subject_facts = context(&parents, &subject);
        for index in 0..2 {
            assert_eq!(run(&packet, &tx, &commitment, index, &facts), Ok(Ok(())));
            assert_eq!(
                run(&packet, &subject, &commitment, index, &subject_facts),
                Ok(Err(Refusal::Unequal))
            );
            let mut facts = subject_facts.clone();
            facts.current = Some(u32::try_from(index).unwrap());
            assert!(fragment(BIRTH, &facts).is_ok());
            assert_eq!(fragment(CONTINUATION, &facts), Err(Refusal::Unequal));
        }
    }
}

#[test]
fn constructed_path_mutants_refuse_authentication_before_execution() {
    let packet = fixture::case(2);
    let (snapshot, commitment) = fixture::rebuild(&packet, true);
    let tx = TargetTransaction::decode(&snapshot.complete).unwrap();
    let parents = parents(&packet);
    let facts = context(&parents, &tx);
    for (coordinate, expected) in [(0, A::WrongParity), (1, A::WrongLeaf), (2, A::WrongVersion)] {
        let mut witnesses = tx.witnesses().to_vec();
        let mut stack = witnesses[1].stack().to_vec();
        if coordinate == 0 {
            stack[1][0] ^= 1;
        } else if coordinate == 1 {
            stack[0][1] ^= 1;
        } else {
            stack[1][0] ^= 2;
        }
        witnesses[1] = transaction::InputWitness::new(stack);
        let subject = TargetTransaction::new(
            tx.version(),
            tx.inputs().to_vec(),
            tx.outputs().to_vec(),
            tx.lock_time(),
            witnesses,
        )
        .unwrap();
        assert_eq!(run(&packet, &tx, &commitment, 1, &facts), Ok(Ok(())));
        assert_eq!(run(&packet, &subject, &commitment, 0, &facts), Ok(Ok(())));
        assert_eq!(
            run(&packet, &subject, &commitment, 1, &facts),
            Err(expected)
        );
    }
}

#[test]
fn constructed_context_changes_only_the_second_members_birth_coordinate() {
    let packet = fixture::case(2);
    let (snapshot, commitment) = fixture::rebuild(&packet, true);
    let tx = TargetTransaction::decode(&snapshot.complete).unwrap();
    let parents = parents(&packet);
    let control = context(&parents, &tx);
    let mut subject = control.clone();
    // Synthetic model context, not a forged request or admitted transaction.
    subject.inputs[1].index = 1;
    for index in 0..2 {
        assert_eq!(run(&packet, &tx, &commitment, index, &control), Ok(Ok(())));
    }
    assert_eq!(run(&packet, &tx, &commitment, 0, &subject), Ok(Ok(())));
    assert_eq!(
        run(&packet, &tx, &commitment, 1, &subject),
        Ok(Err(Refusal::Unequal))
    );
    subject.current = Some(1);
    assert_eq!(fragment(BIRTH, &subject), Err(Refusal::Unequal));
    assert!(fragment(CONTINUATION, &subject).is_ok());
}
