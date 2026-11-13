//! Typed, deterministic response stand-in. Script verdicts are scripted, not native evidence.
//! Funding and signatures bind actual bytes; the existing bridge owns authority controls.
use super::{bridge_mock, driver};
use crate::keyed_reproduction::fixture as prior;
use elements::secp256k1_zkp::{Message, PublicKey, Secp256k1, SecretKey};
use target_elements_conformance::executor::OperationStep;
use target_elements_conformance::protocol::*;
use transaction::TargetTransaction;
use vectors::keyed_final_capture::*;

pub struct Mock {
    funding: bridge_mock::Session,
    coins: std::collections::BTreeMap<(String, u32), transaction::TargetOutput>,
    sponsor_program: Vec<u8>,
    pub rows: Vec<CaptureRow>,
    pub exchanges: Vec<(NativeOperationRequest, NativeOperationResponse)>,
}
impl Mock {
    pub fn new() -> Self {
        Self {
            funding: bridge_mock::Session::new(),
            coins: std::collections::BTreeMap::new(),
            sponsor_program: vec![],
            rows: vec![],
            exchanges: vec![],
        }
    }
    fn register(&mut self, bytes: &[u8]) {
        let parent = TargetTransaction::decode(bytes).unwrap();
        let txid = transaction::Txid::from_internal(prior::txid(bytes)).to_target_display();
        for (index, output) in parent.outputs().iter().enumerate() {
            if output.program() == self.sponsor_program {
                self.coins.insert(
                    (txid.clone(), u32::try_from(index).unwrap()),
                    output.clone(),
                );
            }
        }
    }
}
impl driver::Exchange for Mock {
    fn before_submission(&mut self, row: CaptureRow) {
        self.rows.push(row);
    }
    fn exchange(&mut self, step: OperationStep) -> NativeOperationResponse {
        let mut response = NativeOperationResponse {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case: step.case().clone(),
            observed_layer: ObservedOutcomeLayer::Accepted,
            observed_detail: Some("typed scripted mock; no native execution".into()),
            issued_asset: None,
            funded_outputs: vec![],
            confidential_funded_outputs: vec![],
            mined_readback: None,
            accepted_txid: None,
            sponsor_witness: vec![],
            script_path_witness: vec![],
            signer_public_key: None,
            signed_profile: None,
            signing_genesis: None,
            signature_bound_to: None,
            resources: NativeResourceObservation::default(),
        };
        match step.subject() {
            OperationSubject::SponsorFunding(_) => {
                let bytes = self.funding.fund();
                let (expected, answer) = self.funding.exchanges.last().unwrap();
                assert_eq!(expected, &step);
                response = answer.clone();
                let parent = TargetTransaction::decode(&bytes).unwrap();
                self.sponsor_program = parent.outputs()[1].program().to_vec();
                self.register(&bytes);
            }
            OperationSubject::SponsorSigning(subject) => {
                let output = &self.coins[&(
                    subject.sponsor_outpoint.txid.clone(),
                    subject.sponsor_outpoint.vout,
                )];
                let digest = prior::reference_digest(
                    &subject.finalized_transaction,
                    usize::from(subject.sponsor_input_index),
                    output,
                );
                let secp = Secp256k1::new();
                let secret = SecretKey::from_slice(&[1; 32]).unwrap();
                let mut signature = secp
                    .sign_ecdsa(&Message::from_digest(digest), &secret)
                    .serialize_der()
                    .to_vec();
                signature.push(1);
                response.sponsor_witness = vec![
                    signature,
                    PublicKey::from_secret_key(&secp, &secret)
                        .serialize()
                        .to_vec(),
                ];
                response.signature_bound_to = Some(subject.finalized_transaction.clone());
            }
            OperationSubject::Submission(subject) => {
                let row = self.rows.last().unwrap();
                assert_eq!(row.submitted, subject.transaction_bytes);
                response.observed_layer = row.expected;
                if row.expected == ObservedOutcomeLayer::Accepted {
                    let tx: elements::Transaction =
                        elements::encode::deserialize(&subject.transaction_bytes).unwrap();
                    response.accepted_txid = Some(tx.txid().to_string());
                    response.mined_readback = Some(MinedFundingReadback {
                        transaction_id: tx.txid().to_string(),
                        witness_transaction_id: tx.wtxid().to_string(),
                        block_hash: format!("mock-block-{}", self.exchanges.len()),
                        block_height: u32::try_from(self.exchanges.len()).unwrap() + 100,
                        raw_transaction: subject.transaction_bytes.clone(),
                    });
                    self.register(&subject.transaction_bytes);
                }
            }
            _ => panic!("closed capture operation census"),
        }
        self.exchanges.push((
            NativeOperationRequest {
                schema: NATIVE_PROTOCOL_SCHEMA,
                case: step.case().clone(),
                subject: step.subject().clone(),
            },
            response.clone(),
        ));
        response
    }
}
