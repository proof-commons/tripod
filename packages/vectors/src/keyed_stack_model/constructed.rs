//! Model-only execution of extracted constructed bytes. No native acceptance.
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

fn context(packet: &fixture::Packet, tx: &TargetTransaction, current: usize) -> Facts {
    // Actual constructed input order and outpoints select public parent outputs.
    // Parent content addresses are recomputed, not taken as trusted model facts.
    let inputs = tx
        .inputs()
        .iter()
        .map(|input| {
            let outpoint = input.outpoint();
            let parent = packet
                .inputs
                .iter()
                .find(|row| {
                    transaction::Txid::from_internal(fixture::txid(&row.parent)) == outpoint.txid()
                })
                .unwrap();
            let decoded = TargetTransaction::decode(&parent.parent).unwrap();
            InputFact {
                txid: parent.txid,
                index: outpoint.index(),
                flags: 0,
                program: program(
                    decoded.outputs()[usize::try_from(outpoint.index()).unwrap()].program(),
                ),
            }
        })
        .collect();
    Facts {
        enabled: true,
        current: Some(u32::try_from(current).unwrap()),
        inputs,
        outputs: tx.outputs().iter().map(|o| program(o.program())).collect(),
    }
}

fn run(
    packet: &fixture::Packet,
    tx: &TargetTransaction,
    commitment: &KeyedCommitment,
    index: usize,
) -> Result<Result<(), Refusal>, A> {
    let stack = tx.witnesses()[index].stack();
    assert_eq!(
        stack.len(),
        2,
        "no execution arguments or covenant signature"
    );
    authenticate_keyed_path(commitment, &stack[0], &stack[1])?;
    Ok(leaf(&stack[0], &context(packet, tx, index), &packet.key).map(|_| ()))
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
            let parent = TargetTransaction::decode(&packet.inputs[index].parent).unwrap();
            authenticate_keyed_predecessor(
                &target,
                &oracle,
                parent.outputs()[0].program(),
                &PublicKeyedCurve,
            )
            .unwrap();
            assert_eq!(run(&packet, &tx, &commitment, index), Ok(Ok(())));
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
        for index in 0..2 {
            assert_eq!(run(&packet, &tx, &commitment, index), Ok(Ok(())));
            assert_eq!(
                run(&packet, &subject, &commitment, index),
                Ok(Err(Refusal::Unequal))
            );
            let facts = context(&packet, &subject, index);
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
    for (coordinate, expected) in [(0, A::WrongParity), (1, A::WrongLeaf)] {
        let mut witnesses = tx.witnesses().to_vec();
        let mut stack = witnesses[1].stack().to_vec();
        if coordinate == 0 {
            stack[1][0] ^= 1;
        } else {
            stack[0][1] ^= 1;
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
        assert_eq!(run(&packet, &tx, &commitment, 1), Ok(Ok(())));
        assert_eq!(run(&packet, &subject, &commitment, 0), Ok(Ok(())));
        assert_eq!(run(&packet, &subject, &commitment, 1), Err(expected));
    }
}
