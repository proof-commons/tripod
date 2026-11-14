//! Fresh-process construction consumes recorded ordinary signatures, never keys for signing.
use super::{decoded_manifest as manifest, hex};
use serde_json::Value;
use std::collections::BTreeMap;
use transaction::keyed_construct::{
    KeyedConstructionError, KeyedConstructionRequest, KeyedOutputOffer, KeyedPolicy, admit_keyed,
    construct_keyed,
};
use transaction::keyed_inputs::{KeyedInputOffer, KeyedInputRole};
use transaction::keyed_request::{keyed_merge_request, keyed_source_request};
use transaction::{
    AssetField, Outpoint, SponsorCapability, SponsorOffer, SponsorSignature, SponsorSigningRequest,
    TargetTransaction, ValueField,
};

#[allow(
    dead_code,
    reason = "Only the existing public linker fixture is reused here; its other tests retain the remaining helpers."
)]
#[path = "../../tests/keyed_reproduction/fixture.rs"]
#[allow(
    clippy::duplicate_mod,
    reason = "Reuse the unchanged test fixture; its other consumer is private inside the protected model module."
)]
mod public;

struct Recorded<'a> {
    tx: &'a TargetTransaction,
    signer: usize,
    fee: u64,
}
impl SponsorCapability for Recorded<'_> {
    fn offer(&self) -> SponsorOffer {
        SponsorOffer::new(
            [self.tx.inputs()[self.signer].outpoint()],
            self.fee,
            Some(self.tx.outputs()[1].value()),
        )
        .unwrap()
    }
    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        Some((0, self.tx.outputs()[1].program()[2..].to_vec()))
    }
    fn sign(&self, request: &SponsorSigningRequest) -> Option<SponsorSignature> {
        assert_eq!(usize::from(request.input()), self.signer);
        assert_eq!(request.transaction(), self.tx.encode_without_witness());
        Some(SponsorSignature::new(
            request.transaction().to_vec(),
            self.tx.witnesses()[self.signer].stack().to_vec(),
        ))
    }
}
fn value(field: ValueField) -> u64 {
    let ValueField::Explicit(n) = field else {
        panic!("explicit capture amount")
    };
    n
}
#[allow(
    clippy::too_many_lines,
    reason = "Keep public request construction and recorded-witness replay in one test-only per-case procedure."
)]
pub(super) fn check(case: &Value, key_index: usize, chunk: usize) {
    let _span = tracing::info_span!("reproduce_final_bytes", key_index, chunk).entered();
    let session = &case["sessions"][key_index];
    let manifest = manifest(case).unwrap();
    let rows: BTreeMap<_, _> = manifest
        .rows
        .iter()
        .map(|row| (hex(&row.digest), row))
        .collect();
    let funding: Vec<_> = session["parents"]
        .as_array()
        .unwrap()
        .iter()
        .map(|parent| super::bytes(&parent["raw_transaction"]).unwrap())
        .collect();
    let parents: BTreeMap<_, &[u8]> = funding
        .iter()
        .map(|raw| {
            let tx: elements::Transaction = elements::encode::deserialize(raw).unwrap();
            (
                crate::keyed_history_reader::Address(tx.txid().to_byte_array()),
                raw.as_slice(),
            )
        })
        .chain(
            session["observations"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|observation| observation["layer"] == "accepted")
                .map(|observation| {
                    let id: elements::Txid = observation["readback"]["transaction_id"]
                        .as_str()
                        .unwrap()
                        .parse()
                        .unwrap();
                    (
                        crate::keyed_history_reader::Address(id.to_byte_array()),
                        rows[observation["digest"].as_str().unwrap()]
                            .submitted
                            .as_slice(),
                    )
                }),
        )
        .collect();
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let commitments: BTreeMap<_, _> = [[42; 32], crate::keyed_final_manifest::SECOND_KEY]
        .into_iter()
        .map(|key| {
            (
                key,
                (
                    transaction::keyed_taproot::commit_keyed_bundle(
                        &target,
                        &public::linked(key, true),
                    )
                    .unwrap(),
                    transaction::keyed_taproot::commit_keyed_bundle(
                        &target,
                        &public::linked(key, false),
                    )
                    .unwrap(),
                ),
            )
        })
        .collect();
    for observation in session["observations"]
        .as_array()
        .unwrap()
        .iter()
        .skip(chunk * 128)
        .take(128)
    {
        let row = rows[observation["digest"].as_str().unwrap()];
        let reference: elements::Transaction =
            elements::encode::deserialize(&row.submitted).unwrap();
        assert_eq!(elements::encode::serialize(&reference), row.submitted);
        let structural =
            row.boundary == crate::keyed_final_capture::CaptureBoundary::StructuralEnvelope;
        if !row.sealed_control && !structural {
            continue;
        }
        let key = row.key;
        let (source, merge) = &commitments[&key];
        let tx = TargetTransaction::decode(&row.submitted).unwrap();
        let decoded: Vec<_> = tx
            .inputs()
            .iter()
            .map(|i| {
                let raw =
                    parents[&crate::keyed_history_reader::Address(*i.outpoint().txid().internal())];
                let parent: elements::Transaction = elements::encode::deserialize(raw).unwrap();
                assert_eq!(
                    parent.txid().to_byte_array(),
                    *i.outpoint().txid().internal()
                );
                TargetTransaction::decode(raw).unwrap()
            })
            .collect();
        let members: Vec<_> = tx
            .inputs()
            .iter()
            .zip(&decoded)
            .map(|(i, parent)| {
                parent.outputs()[i.outpoint().index() as usize].program() == source.output_program()
            })
            .collect();
        let signers: Vec<_> = members
            .iter()
            .enumerate()
            .filter_map(|(i, member)| (!member).then_some(i))
            .collect();
        let [signer] = signers.as_slice() else {
            panic!("one recorded sponsor")
        };
        let fee = value(tx.outputs().last().unwrap().value());
        let cap = Recorded {
            tx: &tx,
            signer: *signer,
            fee,
        };
        let offers: Vec<_> = tx
            .inputs()
            .iter()
            .enumerate()
            .map(|(index, i)| KeyedInputOffer {
                input_index: u32::try_from(index).unwrap(),
                parent: parents
                    [&crate::keyed_history_reader::Address(*i.outpoint().txid().internal())],
                outpoint: Outpoint::new(i.outpoint().txid(), i.outpoint().index()).unwrap(),
                sequence: i.sequence(),
                flags: 0,
                role: if members[index] {
                    KeyedInputRole::Member
                } else {
                    KeyedInputRole::Collateral
                },
                capability: if members[index] {
                    None
                } else {
                    Some(&cap as &dyn SponsorCapability)
                },
            })
            .collect();
        let source_request;
        let merge_request;
        let request = if members.iter().any(|member| *member) {
            merge_request = keyed_merge_request(&target, key, merge, &offers).unwrap();
            KeyedConstructionRequest::Merge(&merge_request)
        } else {
            source_request = keyed_source_request(&target, key, source, &offers).unwrap();
            KeyedConstructionRequest::Source(&source_request)
        };
        let AssetField::Explicit(asset) = tx.outputs()[0].asset() else {
            panic!("explicit capture asset")
        };
        let offer = KeyedOutputOffer {
            policy_asset: asset,
            successor_value: value(tx.outputs()[0].value()),
            additional_outputs: vec![tx.outputs()[1].clone()],
            fee: Some(fee),
        };
        let policy = KeyedPolicy::new(
            "replay of capture construction readings; native policy stays per subject".into(),
            asset,
            [100; 3],
            vec![100, 100, 1],
            [400_000; 3],
            0,
            72,
        )
        .unwrap();
        if structural {
            assert!(matches!(
                admit_keyed(&target, request, offer.clone(), &policy),
                Err(KeyedConstructionError::HandleCountExceedsPolicy {
                    offered: 1565,
                    maximum: 1564
                })
            ));
            let structure = construct_keyed(&target, request, offer).unwrap();
            let built = structure.transaction();
            transaction::keyed_sponsor::verify_keyed_sponsor(
                &built.encode_without_witness(),
                *signer,
                &decoded[*signer].outputs()[tx.inputs()[*signer].outpoint().index() as usize],
                tx.witnesses()[*signer].stack(),
            )
            .unwrap();
            let mut witnesses = built.witnesses().to_vec();
            witnesses[*signer] = tx.witnesses()[*signer].clone();
            let rebuilt = TargetTransaction::new(
                built.version(),
                built.inputs().to_vec(),
                built.outputs().to_vec(),
                built.lock_time(),
                witnesses,
            )
            .unwrap();
            assert_eq!(rebuilt.encode(), row.submitted);
        } else {
            let result = admit_keyed(&target, request, offer, &policy).unwrap();
            assert_eq!(result.transaction().encode(), row.submitted);
        }
    }
}
