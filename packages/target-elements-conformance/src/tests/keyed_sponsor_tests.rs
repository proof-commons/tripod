//! Independent Rust digest encoding comparison; ECDSA backend is shared, not
//! independent curve evidence. Keys below are disposable public test material.
use elements::bitcoin::hashes::Hash;
use elements::{EcdsaSighashType, Script, Transaction, confidential, sighash::SighashCache};
use secp256k1_zkp::{Message, PublicKey, Secp256k1, SecretKey, ecdsa::Signature};
use transaction::keyed_sponsor::{
    KeyedSponsorRefusal as E, keyed_sponsor_digest, verify_keyed_sponsor,
};
use transaction::{
    AssetField, AssetId, InputWitness, NonceField, Outpoint, TargetInput, TargetOutput,
    TargetTransaction, Txid, ValueField,
};

fn output(program: Vec<u8>) -> TargetOutput {
    TargetOutput::new(
        AssetField::Explicit(AssetId::from_internal([3; 32])),
        ValueField::Explicit(1000),
        NonceField::Null,
        program,
    )
}
fn fixture(key_byte: u8, index: usize, count: usize) -> (Vec<u8>, TargetOutput, Vec<Vec<u8>>) {
    let secp = Secp256k1::new();
    let secret = SecretKey::from_slice(&[key_byte; 32]).unwrap();
    let public = PublicKey::from_secret_key(&secp, &secret).serialize();
    let hash = elements::hashes::hash160::Hash::hash(&public).to_byte_array();
    let mut program = vec![0, 20];
    program.extend(hash);
    let spent = output(program);
    let tx = TargetTransaction::new(
        2,
        (0..count)
            .map(|n| {
                TargetInput::new(
                    Outpoint::new(
                        Txid::from_internal(std::array::from_fn(|byte| {
                            u8::try_from(byte + n).unwrap()
                        })),
                        0x0102_0304 + u32::try_from(n).unwrap(),
                    )
                    .unwrap(),
                    17 + u32::try_from(n).unwrap(),
                )
            })
            .collect(),
        vec![
            output(vec![0x51, 32].into_iter().chain([42; 32]).collect()),
            output(vec![]),
        ],
        9,
        vec![InputWitness::new(vec![]); count],
    )
    .unwrap();
    let frozen = tx.encode_without_witness();
    let digest = reference(&frozen, index, &spent);
    let mut sig = secp
        .sign_ecdsa(&Message::from_digest(digest), &secret)
        .serialize_der()
        .to_vec();
    sig.push(1);
    (frozen, spent, vec![sig, public.to_vec()])
}
fn reference(bytes: &[u8], index: usize, spent: &TargetOutput) -> [u8; 32] {
    let tx: Transaction = elements::encode::deserialize(bytes).unwrap();
    let code = Script::new_p2pkh(&elements::PubkeyHash::from_slice(&spent.program()[2..]).unwrap());
    let ValueField::Explicit(value) = spent.value() else {
        panic!("explicit fixture")
    };
    SighashCache::new(&tx)
        .segwitv0_sighash(
            index,
            &code,
            confidential::Value::Explicit(value),
            EcdsaSighashType::All,
        )
        .to_byte_array()
}
#[test]
fn keyed_sponsor_digest_and_verification_match_reference_for_keys_and_positions() {
    for key in [1, 2] {
        for (index, count) in [(0, 1), (0, 2), (1, 2)] {
            let (bytes, spent, witness) = fixture(key, index, count);
            assert_eq!(
                keyed_sponsor_digest(&bytes, index, &spent).unwrap(),
                reference(&bytes, index, &spent)
            );
            assert_eq!(
                verify_keyed_sponsor(&bytes, index, &spent, &witness),
                Ok(())
            );
            let sig = Signature::from_der(&witness[0][..witness[0].len() - 1]).unwrap();
            assert!(
                Secp256k1::verification_only()
                    .verify_ecdsa(
                        &Message::from_digest(reference(&bytes, index, &spent)),
                        &sig,
                        &PublicKey::from_slice(&witness[1]).unwrap()
                    )
                    .is_ok()
            );
            let mut changed = bytes;
            changed[0] ^= 1;
            assert_eq!(
                verify_keyed_sponsor(&changed, index, &spent, &witness),
                Err(E::InvalidSignature)
            );
        }
    }
}
#[test]
fn keyed_sponsor_witness_refusals_have_accepted_twins() {
    let (bytes, spent, control) = fixture(1, 1, 2);
    let run = |w: &[Vec<u8>]| verify_keyed_sponsor(&bytes, 1, &spent, w);
    assert_eq!(run(&control), Ok(()));
    assert_eq!(run(&control[..1]), Err(E::WitnessShape));
    let mut subject = control.clone();
    subject[0] = vec![1; 73];
    assert_eq!(run(&subject), Err(E::SignatureLength));
    let mut subject = control.clone();
    *subject[0].last_mut().unwrap() = 2;
    assert_eq!(run(&subject), Err(E::HashType));
    let mut subject = control.clone();
    subject[0][0] = 0x31;
    assert_eq!(run(&subject), Err(E::StrictDer));
    let mut subject = control.clone();
    subject[1] = PublicKey::from_slice(&subject[1])
        .unwrap()
        .serialize_uncompressed()
        .to_vec();
    assert_eq!(run(&subject), Err(E::CompressedPublicKey));
    let mut subject = control.clone();
    subject[1] = fixture(2, 1, 2).2[1].clone();
    assert_eq!(run(&subject), Err(E::PublicKeyHash));
    let sig = Signature::from_der(&control[0][..control[0].len() - 1]).unwrap();
    let mut compact = sig.serialize_compact();
    let order = secp256k1_zkp::constants::CURVE_ORDER;
    let mut borrow = 0_i16;
    for i in (0..32).rev() {
        let d = i16::from(order[i]) - i16::from(compact[32 + i]) - borrow;
        compact[32 + i] = u8::try_from(d.rem_euclid(256)).unwrap();
        borrow = i16::from(d < 0);
    }
    let mut high = Signature::from_compact(&compact)
        .unwrap()
        .serialize_der()
        .to_vec();
    high.push(1);
    let mut subject = control.clone();
    subject[0] = high;
    assert_eq!(
        run(&subject),
        Err(if subject[0].len() > 72 {
            E::SignatureLength
        } else {
            E::HighS
        })
    );
    // A compact high-S signature with small r keeps DER below the byte ceiling.
    compact[..32].fill(0);
    compact[31] = 1;
    let mut high = Signature::from_compact(&compact)
        .unwrap()
        .serialize_der()
        .to_vec();
    high.push(1);
    subject[0] = high;
    assert_eq!(run(&subject), Err(E::HighS));
    assert_eq!(run(&control), Ok(()));
}

#[test]
fn keyed_sponsor_digest_forms_refuse_without_guessing() {
    let (bytes, spent, witness) = fixture(1, 1, 2);
    assert_eq!(verify_keyed_sponsor(&bytes, 1, &spent, &witness), Ok(()));
    let tx = TargetTransaction::decode(&bytes).unwrap();
    let rebuild = |outputs, witnesses| {
        TargetTransaction::new(
            tx.version(),
            tx.inputs().to_vec(),
            outputs,
            tx.lock_time(),
            witnesses,
        )
        .unwrap()
    };
    let witnessed = rebuild(tx.outputs().to_vec(), vec![InputWitness::new(witness); 2]);
    assert_eq!(
        keyed_sponsor_digest(&witnessed.encode(), 1, &spent),
        Err(E::NonCanonicalFrozenBytes)
    );
    let mut nonced = tx.outputs().to_vec();
    nonced[0] = TargetOutput::new(
        nonced[0].asset(),
        nonced[0].value(),
        NonceField::Commitment([2; 33]),
        nonced[0].program().to_vec(),
    );
    assert_eq!(
        keyed_sponsor_digest(
            &rebuild(nonced, tx.witnesses().to_vec()).encode_without_witness(),
            1,
            &spent
        ),
        Err(E::UnsupportedForm)
    );
    let committed = TargetOutput::new(
        spent.asset(),
        ValueField::Commitment([8; 33]),
        spent.nonce(),
        spent.program().to_vec(),
    );
    assert_eq!(
        keyed_sponsor_digest(&bytes, 1, &committed),
        Err(E::UnsupportedForm)
    );
    let version_one = output(vec![0x51, 32].into_iter().chain([7; 32]).collect());
    assert_eq!(
        keyed_sponsor_digest(&bytes, 1, &version_one),
        Err(E::Program)
    );
    assert_eq!(keyed_sponsor_digest(&bytes, 2, &spent), Err(E::InputIndex));
    assert!(matches!(
        keyed_sponsor_digest(&bytes[..bytes.len() - 1], 1, &spent),
        Err(E::Decode(_))
    ));
    assert_eq!(
        keyed_sponsor_digest(&bytes, 1, &spent).unwrap(),
        reference(&bytes, 1, &spent)
    );
}

#[test]
fn keyed_sponsor_noncanonical_der_and_witness_shapes_have_exact_refusals() {
    let (bytes, spent, control) = fixture(1, 0, 1);
    let run = |w: &[Vec<u8>]| verify_keyed_sponsor(&bytes, 0, &spent, w);
    assert_eq!(run(&control), Ok(()));
    let mut subject = control.clone();
    subject[0].clear();
    assert_eq!(run(&subject), Err(E::SignatureLength));
    let mut subject = control.clone();
    subject.push(vec![]);
    assert_eq!(run(&subject), Err(E::WitnessShape));
    let mut subject = control.clone();
    subject[1] = vec![0xff; 33];
    subject[1][0] = 2;
    assert_eq!(run(&subject), Err(E::CompressedPublicKey));
    // Negative r is accepted as zero by the backend's DER parser. Its re-encoding
    // differs, so this subject specifically exercises the canonicality check.
    let der = vec![0x30, 6, 2, 1, 0x80, 2, 1, 1];
    let parsed = Signature::from_der(&der).unwrap();
    assert_ne!(parsed.serialize_der().as_ref(), der);
    let mut subject = control.clone();
    subject[0] = der.into_iter().chain([1]).collect();
    assert_eq!(run(&subject), Err(E::StrictDer));
    assert_eq!(run(&control), Ok(()));
}
