//! Operator-only fixed-form native sessions. Every request and answer is retained.

use super::{POLICY_ASSET, linked, parameters};
use architecture::keyed_fixed::KeyedFixedOperationId;
use elements::hashes::sha256d;
use realization::keyed_fixed::{KeyedFixedParameters, KeyedFixedValue};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write as _;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use target_elements::*;
use target_elements_conformance::constructor::tagged::sha256 as tagged_sha256;
use target_elements_conformance::executor::{
    ExecutorConfiguration, ExecutorDiagnostics, ExecutorTrust, NativeOperationCapture,
    OperationStep, PlanRefused, TargetOperationPlanner, execute_operations_captured,
};
use target_elements_conformance::protocol::{
    NATIVE_PROTOCOL_SCHEMA, NativeOperationRequest, NativeOperationResponse, ObservedOutcomeLayer,
    OperationCaseId, OperationSubject, TargetSponsorFundingSubject, TargetSponsorSigningSubject,
    TargetSubmissionSubject, WireOutpoint, WireSighashProfile, WireSpentOutput,
};
use target_elements_conformance::provenance::ExpectedExecutorProvenance;
use transaction::bytes::OutputWitness;
use transaction::keyed_fixed_construct::{
    KeyedFixedConstructionError, KeyedFixedConstructionRequest, KeyedFixedOutputOffer,
    KeyedFixedPolicy, admit_keyed_fixed, construct_keyed_fixed,
};
use transaction::keyed_fixed_inputs::{KeyedFixedInputOffer, KeyedFixedInputRole};
use transaction::keyed_fixed_request::{keyed_fixed_merge_request, keyed_fixed_source_request};
use transaction::keyed_fixed_taproot::commit_keyed_fixed_bundle;
use transaction::{
    AssetField, AssetId, InputWitness, NonceField, Outpoint, SignerRole, SponsorCapability,
    SponsorOffer, SponsorSignature, SponsorSigningRequest, TargetInput, TargetOutput,
    TargetTransaction, Txid, ValueField,
};
use vectors::keyed_fixed_capture::{
    CaptureFamily, CaptureProvenance, CaptureSubject, JournalBinding, JournalRow, PairControl,
    bind_capture_freeze, bind_journal, expected_control, expected_layer, subjects,
};
use vectors::keyed_fixed_history_reader as reader;
use vectors::keyed_fixed_manifest::FreezeManifest;
use vectors::keyed_fixed_native::{ObservedFreeze, bind_native_freeze, bind_structural_subject};

const BLINDER_ONE: [u8; 32] = {
    let mut bytes = [0; 32];
    bytes[31] = 1;
    bytes
};

fn hex32(text: &str) -> [u8; 32] {
    vectors::maturity_observer::decode_hex(text)
        .unwrap()
        .try_into()
        .unwrap()
}

fn field<'a>(value: &'a serde_json::Value, name: &str) -> &'a str {
    value[name].as_str().expect("prepared provenance field")
}

fn store<T: Serialize>(path: &Path, value: &T) {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    serde_json::to_writer(&mut file, value).unwrap();
    writeln!(file).unwrap();
    file.sync_all().unwrap();
    std::fs::File::open(path.parent().unwrap())
        .unwrap()
        .sync_all()
        .unwrap();
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Submission {
    subject: CaptureSubject,
    key: [u8; 32],
    asset: [u8; 32],
    value: u64,
    request: NativeOperationRequest,
    response: NativeOperationResponse,
    submitted: Vec<u8>,
    parents: Vec<Vec<u8>>,
    freeze: [u8; 32],
    sponsor_index: usize,
    control: Option<Box<Self>>,
}

struct Wire {
    steps: mpsc::Sender<Option<OperationStep>>,
    answers: mpsc::Receiver<NativeOperationResponse>,
    directory: PathBuf,
    ordinal: usize,
    completed: Vec<Submission>,
}

impl Wire {
    fn ask(&mut self, step: OperationStep) -> NativeOperationResponse {
        self.ordinal += 1;
        let request = NativeOperationRequest {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case: step.case().clone(),
            subject: step.subject().clone(),
        };
        let prefix = format!("exchange-{:05}", self.ordinal);
        store(
            &self.directory.join(format!("{prefix}-request.json")),
            &request,
        );
        let started = Instant::now();
        self.steps
            .send(Some(step))
            .expect("native executor is live");
        let response = self.answers.recv().expect("native response or refusal");
        store(
            &self.directory.join(format!("{prefix}-response.json")),
            &response,
        );
        if self.ordinal % 64 == 0 {
            tracing::info!(target: "vectors::native_test",
                completed_count = u64::try_from(self.ordinal).unwrap_or(u64::MAX),
                "fixed-form native exchanges retained");
        }
        tracing::debug!(target: "vectors::native_test", %prefix,
            wall_us = u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX),
            "fixed-form native exchange retained");
        response
    }

    fn submission(
        &mut self,
        subject: CaptureSubject,
        parameters: KeyedFixedParameters,
        submitted: Vec<u8>,
        parents: Vec<Vec<u8>>,
        sponsor_index: usize,
    ) -> NativeOperationResponse {
        let freeze = FreezeManifest::offer(parameters)
            .unwrap()
            .validate(parameters)
            .unwrap();
        let identity = freeze.identity();
        let encoding_path = self.directory.join(format!(
            "freeze-{}-{}.bin",
            parameters.key[0],
            parameters.value.get()
        ));
        if encoding_path.exists() {
            assert_eq!(std::fs::read(&encoding_path).unwrap(), freeze.encoding());
        } else {
            std::fs::write(&encoding_path, freeze.encoding()).unwrap();
            std::fs::File::open(&encoding_path)
                .unwrap()
                .sync_all()
                .unwrap();
        }
        bind_native_freeze(&freeze, Some(ObservedFreeze(identity))).unwrap();
        bind_capture_freeze(&freeze, Some(identity)).unwrap();
        let step = OperationStep::new(
            &format!("fixed-submit-{}", self.ordinal + 1),
            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: submitted.clone(),
            })),
        );
        let request = NativeOperationRequest {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case: step.case().clone(),
            subject: step.subject().clone(),
        };
        let prefix = format!("subject-{:05}", self.ordinal + 1);
        std::fs::write(
            self.directory.join(format!("{prefix}.transaction")),
            &submitted,
        )
        .unwrap();
        for (index, parent) in parents.iter().enumerate() {
            std::fs::write(
                self.directory
                    .join(format!("{prefix}-parent-{index}.transaction")),
                parent,
            )
            .unwrap();
        }
        let response = self.ask(step);
        let predicted = expected_layer(subject).expect("submitted census subject");
        assert_eq!(
            response.observed_layer, predicted,
            "stage drift for {subject:?}; node detail: {:?}",
            response.observed_detail
        );
        let control = expected_control(subject).map(|wanted| {
            Box::new(
                self.completed
                    .iter()
                    .rev()
                    .find(|row| row.subject == wanted)
                    .expect("accepted paired control precedes negative")
                    .clone(),
            )
        });
        let row = Submission {
            subject,
            key: parameters.key,
            asset: parameters.asset,
            value: parameters.value.get(),
            request,
            response: response.clone(),
            submitted,
            parents,
            freeze: identity,
            sponsor_index,
            control,
        };
        store(&self.directory.join(format!("{prefix}.json")), &row);
        if response.observed_layer == ObservedOutcomeLayer::Accepted {
            self.completed.push(row);
        }
        response
    }
}

struct Planner {
    steps: mpsc::Receiver<Option<OperationStep>>,
    answers: Option<mpsc::Sender<NativeOperationResponse>>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl Planner {
    fn new(family: CaptureFamily, directory: &Path) -> Self {
        let (send_step, steps) = mpsc::channel();
        let (answers, receive_answer) = mpsc::channel();
        let directory = directory.to_owned();
        let worker = std::thread::spawn(move || {
            let mut wire = Wire {
                steps: send_step,
                answers: receive_answer,
                directory,
                ordinal: 0,
                completed: Vec::new(),
            };
            campaign(&mut wire, family);
            wire.steps.send(None).unwrap();
        });
        Self {
            steps,
            answers: Some(answers),
            worker: Some(worker),
        }
    }
}

impl TargetOperationPlanner for Planner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        if let Some((_, response)) = previous {
            self.answers
                .as_ref()
                .ok_or(PlanRefused)?
                .send(response.clone())
                .map_err(|_| PlanRefused)?;
        }
        self.steps.recv().map_err(|_| PlanRefused)
    }
}

impl Drop for Planner {
    fn drop(&mut self) {
        self.answers.take();
        if let Some(worker) = self.worker.take() {
            worker.join().expect("fixed-form campaign worker");
        }
    }
}

struct Sponsor {
    point: Outpoint,
    fee: u64,
    change: u64,
    program: Vec<u8>,
    witness: RefCell<Vec<Vec<u8>>>,
}

impl SponsorCapability for Sponsor {
    fn offer(&self) -> SponsorOffer {
        SponsorOffer::new(
            [self.point],
            self.fee,
            Some(ValueField::Explicit(self.change)),
        )
        .unwrap()
    }
    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        Some((0, self.program[2..].to_vec()))
    }
    fn sign(&self, request: &SponsorSigningRequest) -> Option<SponsorSignature> {
        assert_eq!(request.role(), SignerRole::KeyedCollateral);
        Some(SponsorSignature::new(
            request.transaction().to_vec(),
            self.witness.borrow().clone(),
        ))
    }
}

const fn output(asset: AssetId, value: u64, program: Vec<u8>) -> TargetOutput {
    TargetOutput::new(
        AssetField::Explicit(asset),
        ValueField::Explicit(value),
        NonceField::Null,
        program,
    )
}

fn txid(bytes: &[u8]) -> Txid {
    let parent = TargetTransaction::decode(bytes).unwrap();
    Txid::from_internal(sha256d::Hash::hash(&parent.encode_without_witness()).to_byte_array())
}

fn fund_sponsor(wire: &mut Wire) -> (Vec<u8>, u32, AssetId, Vec<u8>) {
    let answer = wire.ask(OperationStep::new(
        &format!("fixed-fund-{}", wire.ordinal + 1),
        OperationSubject::SponsorFunding(Box::new(TargetSponsorFundingSubject {
            sponsor_outputs: 1,
            amount_per_sponsor_output: 1_000_000,
        })),
    ));
    assert_eq!(answer.observed_layer, ObservedOutcomeLayer::Accepted);
    let [coin] = answer.funded_outputs.as_slice() else {
        panic!("one sponsor coin")
    };
    let bytes = target_elements_conformance::sponsor_parent::bind_sponsor_parent(
        answer
            .mined_readback
            .as_ref()
            .expect("mined sponsor parent"),
        coin,
    )
    .unwrap()
    .to_vec();
    let parent = TargetTransaction::decode(&bytes).unwrap();
    let spent = &parent.outputs()[coin.outpoint.vout as usize];
    let AssetField::Explicit(asset) = spent.asset() else {
        panic!("explicit policy asset")
    };
    assert_eq!(asset.internal(), &POLICY_ASSET);
    (bytes, coin.outpoint.vout, asset, spent.program().to_vec())
}

fn signing_subject(
    frozen: Vec<u8>,
    index: usize,
    point: Outpoint,
    spent: &TargetOutput,
) -> TargetSponsorSigningSubject {
    let AssetField::Explicit(asset) = spent.asset() else {
        panic!("explicit sponsor asset")
    };
    let ValueField::Explicit(value) = spent.value() else {
        panic!("explicit sponsor value")
    };
    let mut asset_field = vec![1];
    asset_field.extend_from_slice(asset.internal());
    let mut value_field = vec![1];
    value_field.extend_from_slice(&value.to_be_bytes());
    TargetSponsorSigningSubject {
        finalized_transaction: frozen,
        sponsor_input_index: u16::try_from(index).unwrap(),
        sponsor_outpoint: WireOutpoint {
            txid: point.txid().to_target_display(),
            vout: point.index(),
        },
        spent_output: Some(WireSpentOutput {
            asset_field,
            value_field,
            program: spent.program().to_vec(),
        }),
        sighash_profile: WireSighashProfile::AllInputsAllOutputs,
    }
}

#[allow(clippy::too_many_lines)] // Retain one borrowed request through structural construction, signing and admission.
#[allow(clippy::needless_pass_by_value)] // Own the optional replacement so retained parent bytes remain independent of the caller.
fn event_with_replacement(
    wire: &mut Wire,
    subject: CaptureSubject,
    q: KeyedFixedParameters,
    parents: &[(Vec<u8>, u32, bool)],
    replacement: Option<(Vec<u8>, u32, u64, bool)>,
) -> Vec<u8> {
    let target = reviewed_elements_tapscript().unwrap();
    let source = parents.iter().all(|parent| !parent.2);
    let operation = if source {
        KeyedFixedOperationId::Source
    } else {
        KeyedFixedOperationId::Merge
    };
    let commitment = commit_keyed_fixed_bundle(&target, &linked(q, operation)).unwrap();
    let decoded: Vec<_> = parents
        .iter()
        .map(|(bytes, _, _)| TargetTransaction::decode(bytes).unwrap())
        .collect();
    let sponsor_index = parents.iter().position(|parent| !parent.2).unwrap();
    assert_eq!(parents.iter().filter(|parent| !parent.2).count(), 1);
    let total: u64 = parents
        .iter()
        .zip(&decoded)
        .map(
            |((_, vout, _), parent)| match parent.outputs()[*vout as usize].value() {
                ValueField::Explicit(value) => value,
                ValueField::Commitment(_) => q.value.get(),
                // The byte owner reserves future forms; a capture cannot infer their amount.
                _ => panic!("unsupported future value form"),
            },
        )
        .sum();
    let fee = 10_000;
    let value = q.value.get();
    let change = total - value - fee;
    let sponsor_parent = &decoded[sponsor_index];
    let sponsor_spent = &sponsor_parent.outputs()[parents[sponsor_index].1 as usize];
    let sponsor = Sponsor {
        point: Outpoint::new(txid(&parents[sponsor_index].0), parents[sponsor_index].1).unwrap(),
        fee,
        change,
        program: sponsor_spent.program().to_vec(),
        witness: RefCell::new(vec![]),
    };
    let inputs: Vec<_> = parents
        .iter()
        .enumerate()
        .map(|(index, (bytes, vout, member))| KeyedFixedInputOffer {
            input_index: u32::try_from(index).unwrap(),
            parent: bytes,
            outpoint: Outpoint::new(txid(bytes), *vout).unwrap(),
            sequence: 0x8000_0000 | (1000 + u32::try_from(index).unwrap()),
            flags: 0,
            role: if *member {
                KeyedFixedInputRole::Member
            } else {
                KeyedFixedInputRole::Collateral
            },
            capability: if *member { None } else { Some(&sponsor) },
        })
        .collect();
    let source_request;
    let merge_request;
    let request = if source {
        source_request = keyed_fixed_source_request(&target, q, &commitment, &inputs).unwrap();
        KeyedFixedConstructionRequest::Source(&source_request)
    } else {
        merge_request = keyed_fixed_merge_request(&target, q, &commitment, &inputs).unwrap();
        KeyedFixedConstructionRequest::Merge(&merge_request)
    };
    let AssetField::Explicit(asset) = sponsor_spent.asset() else {
        panic!("explicit sponsor asset")
    };
    let offer = KeyedFixedOutputOffer {
        policy_asset: asset,
        successor_value: value,
        additional_outputs: vec![output(asset, change, sponsor.program.clone())],
        fee: Some(fee),
    };
    if subject == CaptureSubject::WeightMerge(1300) {
        let policy = KeyedFixedPolicy::new(
            "default-rate fixed-form capture reading".into(),
            asset,
            ([100; 4], true),
            vec![15, 1, 1],
            [400_000; 3],
            0,
            72,
        )
        .unwrap();
        assert!(matches!(
            admit_keyed_fixed(&target, request, offer, &policy),
            Err(KeyedFixedConstructionError::HandleCountExceedsPolicy {
                offered: 1300,
                maximum: 1299,
            })
        ));
        store(
            &wire.directory.join("typed-weight-1300.json"),
            &serde_json::json!({"members":1300,"maximum":1299,"boundary":"construction_policy","submitted":false}),
        );
        return vec![];
    }
    let base_transaction = if matches!(
        subject,
        CaptureSubject::StructuralWeight1300 | CaptureSubject::DirectBlock1300
    ) {
        let built_inputs: Vec<_> = inputs
            .iter()
            .map(|input| TargetInput::new(input.outpoint, input.sequence))
            .collect();
        let built_witnesses: Vec<_> = inputs
            .iter()
            .map(|input| {
                if input.role == KeyedFixedInputRole::Member {
                    InputWitness::new(vec![
                        commitment.leaf().to_vec(),
                        commitment.control().to_vec(),
                    ])
                } else {
                    InputWitness::new(vec![])
                }
            })
            .collect();
        TargetTransaction::new(
            2,
            built_inputs,
            vec![
                output(asset, value, commitment.output_program().to_vec()),
                output(asset, change, sponsor.program.clone()),
                output(asset, fee, vec![]),
            ],
            0,
            built_witnesses,
        )
        .unwrap()
    } else {
        construct_keyed_fixed(&target, request, offer.clone())
            .unwrap()
            .transaction()
            .clone()
    };
    let mut changed_outputs = base_transaction.outputs().to_vec();
    let mut changed_inputs = base_transaction.inputs().to_vec();
    let mut submitted_parents = parents.to_vec();
    if let Some((bytes, vout, semantic_value, confidential)) = &replacement {
        changed_inputs[0] = transaction::TargetInput::new(
            Outpoint::new(txid(bytes), *vout).unwrap(),
            changed_inputs[0].sequence(),
        );
        submitted_parents[0] = (bytes.clone(), *vout, true);
        if *confidential {
            // The incoming blinder is retained on change, so the target's
            // Pedersen sum is valid before the input-pair instruction runs.
            let material =
                target_elements_conformance::confidential_oracles::ReferenceProofMaterial::new();
            let blinder = BLINDER_ONE;
            let seed = [9_u8; 32];
            let commitment = material
                .value_commitment(&q.asset, change, &blinder)
                .unwrap();
            let nonce = material.nonce_commitment(&seed).unwrap();
            changed_outputs[1] = TargetOutput::new(
                AssetField::Explicit(asset),
                ValueField::Commitment(commitment),
                NonceField::Commitment(nonce),
                sponsor.program.clone(),
            );
        } else if *semantic_value != value {
            let adjusted = change
                .checked_add_signed(
                    i64::try_from(*semantic_value).unwrap() - i64::try_from(value).unwrap(),
                )
                .unwrap();
            changed_outputs[1] = output(asset, adjusted, sponsor.program.clone());
        }
    }
    match subject {
        CaptureSubject::ValueMinus | CaptureSubject::ValuePlus => {
            let decrease = subject == CaptureSubject::ValueMinus;
            let successor = if decrease { value - 1 } else { value + 1 };
            let residual = if decrease { change + 1 } else { change - 1 };
            changed_outputs[0] = output(asset, successor, commitment.output_program().to_vec());
            changed_outputs[1] = output(asset, residual, sponsor.program.clone());
        }
        CaptureSubject::SurplusDestination(number) => {
            let destination = vec![0, 20]
                .into_iter()
                .chain(std::iter::repeat_n(number + 1, 20))
                .collect();
            changed_outputs[1] = output(asset, change - 1_000, sponsor.program.clone());
            changed_outputs.insert(2, output(asset, 1_000, destination));
        }
        CaptureSubject::ConfidentialValue => {
            let material =
                target_elements_conformance::confidential_oracles::ReferenceProofMaterial::new();
            let first_blinder = BLINDER_ONE;
            let mut second_blinder = elements::secp256k1_zkp::constants::CURVE_ORDER;
            second_blinder[31] -= 1;
            changed_outputs[0] = TargetOutput::new(
                AssetField::Explicit(asset),
                ValueField::Commitment(
                    material
                        .value_commitment(&q.asset, value, &first_blinder)
                        .unwrap(),
                ),
                NonceField::Commitment(material.nonce_commitment(&[2_u8; 32]).unwrap()),
                commitment.output_program().to_vec(),
            );
            changed_outputs[1] = TargetOutput::new(
                AssetField::Explicit(asset),
                ValueField::Commitment(
                    material
                        .value_commitment(&q.asset, change, &second_blinder)
                        .unwrap(),
                ),
                NonceField::Commitment(material.nonce_commitment(&[3_u8; 32]).unwrap()),
                sponsor.program.clone(),
            );
        }
        CaptureSubject::WrongProgram => {
            let mut wrong = commitment.output_program().to_vec();
            wrong[2] ^= 1;
            changed_outputs[0] = output(asset, value, wrong);
        }
        CaptureSubject::WrongKey => {
            let foreign = parameters(43, value);
            let foreign_program = commit_keyed_fixed_bundle(&target, &linked(foreign, operation))
                .unwrap()
                .output_program()
                .to_vec();
            changed_outputs[0] = output(asset, value, foreign_program);
        }
        CaptureSubject::DisplacedSuccessor => {
            let mut wrong = commitment.output_program().to_vec();
            wrong[2] ^= 1;
            changed_outputs[0] = output(asset, value, wrong);
            changed_outputs[1] = output(asset, value, commitment.output_program().to_vec());
            changed_outputs.insert(2, output(asset, change - value, sponsor.program.clone()));
        }
        CaptureSubject::PresentNonceFloor(_) => {
            let nonce = elements::secp256k1_zkp::PublicKey::from_secret_key(
                &elements::secp256k1_zkp::Secp256k1::new(),
                &elements::secp256k1_zkp::SecretKey::from_slice(&[7; 32]).unwrap(),
            )
            .serialize();
            changed_outputs[0] = TargetOutput::new(
                AssetField::Explicit(asset),
                ValueField::Explicit(value),
                NonceField::Commitment(nonce),
                commitment.output_program().to_vec(),
            );
        }
        CaptureSubject::ZeroFeeFloor => {
            changed_outputs[1] = output(asset, change + fee, sponsor.program.clone());
            assert!(changed_outputs.pop().is_some());
        }
        CaptureSubject::PublicSource(_)
        | CaptureSubject::OneMemberMerge(_)
        | CaptureSubject::TwoMemberMerge(_)
        | CaptureSubject::MalformedPaymentValue
        | CaptureSubject::MalformedPaymentValueForm
        | CaptureSubject::LaterPosition
        | CaptureSubject::MalformedControl
        | CaptureSubject::KeyPathAttempt
        | CaptureSubject::Floor(_)
        | CaptureSubject::WeightSource
        | CaptureSubject::WeightMerge(_)
        | CaptureSubject::StructuralWeight1300
        | CaptureSubject::DirectBlock1300
        | CaptureSubject::HistoryLinear(_)
        | CaptureSubject::HistoryBranch(_)
        | CaptureSubject::HistoryMissingBranch
        | CaptureSubject::LegacyConfidentialValue
        | CaptureSubject::LegacyPublicCopy => {}
    }
    let changed = changed_outputs != base_transaction.outputs()
        || changed_inputs != base_transaction.inputs();
    let to_sign = TargetTransaction::with_output_witnesses(
        base_transaction.version(),
        changed_inputs,
        changed_outputs.clone(),
        base_transaction.lock_time(),
        base_transaction.witnesses().to_vec(),
        if subject == CaptureSubject::ConfidentialValue {
            let material =
                target_elements_conformance::confidential_oracles::ReferenceProofMaterial::new();
            let first_blinder = BLINDER_ONE;
            let mut second_blinder = elements::secp256k1_zkp::constants::CURVE_ORDER;
            second_blinder[31] -= 1;
            let ValueField::Commitment(first) = changed_outputs[0].value() else {
                panic!("committed successor")
            };
            let ValueField::Commitment(second) = changed_outputs[1].value() else {
                panic!("committed change")
            };
            vec![
                OutputWitness::range_proof_only(
                    material
                        .range_proof(
                            &q.asset,
                            value,
                            &first_blinder,
                            &[2_u8; 32],
                            &first,
                            commitment.output_program(),
                        )
                        .unwrap(),
                ),
                OutputWitness::range_proof_only(
                    material
                        .range_proof(
                            &q.asset,
                            change,
                            &second_blinder,
                            &[3_u8; 32],
                            &second,
                            &sponsor.program,
                        )
                        .unwrap(),
                ),
                OutputWitness::empty(),
            ]
        } else if replacement
            .as_ref()
            .is_some_and(|(_, _, _, confidential)| *confidential)
        {
            let material =
                target_elements_conformance::confidential_oracles::ReferenceProofMaterial::new();
            let blinder = BLINDER_ONE;
            let seed = [9_u8; 32];
            let ValueField::Commitment(commitment) = changed_outputs[1].value() else {
                panic!("committed change")
            };
            vec![
                OutputWitness::empty(),
                OutputWitness::range_proof_only(
                    material
                        .range_proof(
                            &q.asset,
                            change,
                            &blinder,
                            &seed,
                            &commitment,
                            &sponsor.program,
                        )
                        .unwrap(),
                ),
                OutputWitness::empty(),
            ]
        } else if changed {
            (0..changed_outputs.len())
                .map(|_| OutputWitness::empty())
                .collect()
        } else {
            base_transaction.output_witnesses().to_vec()
        },
    )
    .unwrap();
    let frozen = to_sign.encode_without_witness();
    let signing = signing_subject(frozen.clone(), sponsor_index, sponsor.point, sponsor_spent);
    let answer = wire.ask(OperationStep::new(
        &format!("fixed-sign-{}", wire.ordinal + 1),
        OperationSubject::SponsorSigning(Box::new(signing)),
    ));
    assert_eq!(answer.observed_layer, ObservedOutcomeLayer::Accepted);
    assert_eq!(
        answer.signature_bound_to.as_deref(),
        Some(frozen.as_slice())
    );
    sponsor.witness.replace(answer.sponsor_witness);
    let mut transaction = if subject == CaptureSubject::Floor(14)
        || matches!(
            subject,
            CaptureSubject::StructuralWeight1300 | CaptureSubject::DirectBlock1300
        )
        || changed
    {
        let mut witnesses = to_sign.witnesses().to_vec();
        witnesses[sponsor_index] = InputWitness::new(sponsor.witness.borrow().clone());
        TargetTransaction::with_output_witnesses(
            to_sign.version(),
            to_sign.inputs().to_vec(),
            to_sign.outputs().to_vec(),
            to_sign.lock_time(),
            witnesses,
            to_sign.output_witnesses().to_vec(),
        )
        .unwrap()
    } else {
        let policy = KeyedFixedPolicy::new(
            "default-rate fixed-form capture reading".into(),
            asset,
            ([100; 4], true),
            vec![15, 1, 1],
            [400_000; 3],
            0,
            72,
        )
        .unwrap();
        admit_keyed_fixed(&target, request, offer, &policy)
            .unwrap()
            .transaction()
            .clone()
    };
    if matches!(
        subject,
        CaptureSubject::MalformedControl | CaptureSubject::KeyPathAttempt
    ) {
        let member_index = parents.iter().position(|parent| parent.2).unwrap();
        let mut witnesses = transaction.witnesses().to_vec();
        let stack = if subject == CaptureSubject::KeyPathAttempt {
            vec![vec![0; 64]]
        } else {
            let mut stack = witnesses[member_index].stack().to_vec();
            stack.last_mut().unwrap()[0] ^= 1;
            stack
        };
        witnesses[member_index] = InputWitness::new(stack);
        transaction = TargetTransaction::with_output_witnesses(
            transaction.version(),
            transaction.inputs().to_vec(),
            transaction.outputs().to_vec(),
            transaction.lock_time(),
            witnesses,
            transaction.output_witnesses().to_vec(),
        )
        .unwrap();
    }
    let bytes = transaction.encode();
    let freeze = FreezeManifest::offer(q).unwrap().validate(q).unwrap();
    let structural_parents: Vec<_> = submitted_parents
        .iter()
        .map(|(bytes, _, _)| (TargetTransaction::decode(bytes).unwrap(), bytes.clone()))
        .collect();
    bind_structural_subject(
        &freeze,
        Some(ObservedFreeze(freeze.identity())),
        transaction,
        &bytes,
        &structural_parents,
        sponsor_index,
    )
    .unwrap()
    .verify_sponsor()
    .unwrap();
    let answer = wire.submission(
        subject,
        q,
        bytes.clone(),
        submitted_parents
            .iter()
            .map(|parent| parent.0.clone())
            .collect(),
        sponsor_index,
    );
    let observed_weight = answer
        .resources
        .transaction_weight
        .expect("native transaction weight");
    assert_eq!(observed_weight, transaction_weight(&bytes));
    let independent: elements::Transaction = elements::encode::deserialize(&bytes).unwrap();
    assert_eq!(elements::encode::serialize(&independent), bytes);
    assert_eq!(
        observed_weight,
        u64::try_from(independent.weight()).unwrap()
    );
    if matches!(
        subject,
        CaptureSubject::WeightSource
            | CaptureSubject::WeightMerge(_)
            | CaptureSubject::StructuralWeight1300
            | CaptureSubject::DirectBlock1300
    ) {
        let members = u64::try_from(parents.len() - 1).unwrap();
        let prefix = if members + 1 < 253 { 1 } else { 3 };
        let upper = 4 * (239 + prefix + 41 * members) + 117 + 143 * members;
        let actual_signature = sponsor.witness.borrow()[0].len();
        assert!(actual_signature <= 72);
        assert_eq!(
            observed_weight,
            upper - (72 - u64::try_from(actual_signature).unwrap())
        );
    }
    let detail: serde_json::Value = serde_json::from_str(
        answer
            .observed_detail
            .as_deref()
            .expect("maturity observation detail"),
    )
    .unwrap();
    match subject {
        CaptureSubject::Floor(14)
        | CaptureSubject::StructuralWeight1300
        | CaptureSubject::DirectBlock1300 => {
            assert_eq!(detail["relay_allowed"], false);
            assert_eq!(detail["route"], "direct-block");
            assert!(detail["relay_detail"].is_string());
            assert_eq!(answer.observed_layer, ObservedOutcomeLayer::Accepted);
            assert_eq!(answer.mined_readback.unwrap().raw_transaction, bytes);
        }
        CaptureSubject::ValueMinus
        | CaptureSubject::ValuePlus
        | CaptureSubject::DisplacedSuccessor
        | CaptureSubject::WrongProgram
        | CaptureSubject::WrongKey
        | CaptureSubject::MalformedControl
        | CaptureSubject::MalformedPaymentValue
        | CaptureSubject::MalformedPaymentValueForm
        | CaptureSubject::LaterPosition
        | CaptureSubject::ConfidentialValue
        | CaptureSubject::KeyPathAttempt => {
            assert_eq!(
                answer.observed_layer,
                ObservedOutcomeLayer::ScriptPathRejection
            );
        }
        CaptureSubject::ZeroFeeFloor => {
            assert_eq!(detail["relay_allowed"], false);
            assert_eq!(detail["route"], "direct-block");
            assert_eq!(answer.observed_layer, ObservedOutcomeLayer::Accepted);
            assert!(detail["relay_detail"].is_string());
            assert_eq!(answer.mined_readback.unwrap().raw_transaction, bytes);
        }
        CaptureSubject::PublicSource(_)
        | CaptureSubject::OneMemberMerge(_)
        | CaptureSubject::TwoMemberMerge(_)
        | CaptureSubject::SurplusDestination(_)
        | CaptureSubject::Floor(_)
        | CaptureSubject::PresentNonceFloor(_)
        | CaptureSubject::WeightSource
        | CaptureSubject::WeightMerge(_)
        | CaptureSubject::HistoryLinear(_)
        | CaptureSubject::HistoryBranch(_)
        | CaptureSubject::HistoryMissingBranch
        | CaptureSubject::LegacyConfidentialValue
        | CaptureSubject::LegacyPublicCopy => {
            assert_eq!(detail["relay_allowed"], true);
            assert_eq!(answer.observed_layer, ObservedOutcomeLayer::Accepted);
            assert_eq!(answer.mined_readback.unwrap().raw_transaction, bytes);
        }
    }
    bytes
}

fn event(
    wire: &mut Wire,
    subject: CaptureSubject,
    q: KeyedFixedParameters,
    parents: &[(Vec<u8>, u32, bool)],
) -> Vec<u8> {
    event_with_replacement(wire, subject, q, parents, None)
}

fn transaction_weight(bytes: &[u8]) -> u64 {
    let decoded = TargetTransaction::decode(bytes).unwrap();
    decoded.weight()
}

fn funded_source(wire: &mut Wire, q: KeyedFixedParameters, subject: CaptureSubject) -> Vec<u8> {
    let (funding, vout, _, _) = fund_sponsor(wire);
    event(wire, subject, q, &[(funding, vout, false)])
}

fn funded_merge(
    wire: &mut Wire,
    q: KeyedFixedParameters,
    members: Vec<Vec<u8>>,
    subject: CaptureSubject,
) -> Vec<u8> {
    let (funding, vout, _, _) = fund_sponsor(wire);
    let mut parents: Vec<_> = members.into_iter().map(|bytes| (bytes, 0, true)).collect();
    parents.push((funding, vout, false));
    event(wire, subject, q, &parents)
}

#[allow(clippy::too_many_lines)] // The proof-bearing payment and its ordinary signature share one exact frozen transaction.
fn payment(
    wire: &mut Wire,
    q: KeyedFixedParameters,
    amount: u64,
    later: bool,
    confidential: bool,
) -> (Vec<u8>, u32) {
    let (parent_bytes, vout, asset, sponsor_program) = fund_sponsor(wire);
    let parent = TargetTransaction::decode(&parent_bytes).unwrap();
    let point = Outpoint::new(txid(&parent_bytes), vout).unwrap();
    let target = reviewed_elements_tapscript().unwrap();
    let program = commit_keyed_fixed_bundle(&target, &linked(q, KeyedFixedOperationId::Source))
        .unwrap()
        .output_program()
        .to_vec();
    let fee = 10_000;
    let change = 1_000_000 - amount - fee;
    let (payment_output, change_output, payment_proof, change_proof) = if confidential {
        let material =
            target_elements_conformance::confidential_oracles::ReferenceProofMaterial::new();
        let first_blinder = BLINDER_ONE;
        let mut second_blinder = elements::secp256k1_zkp::constants::CURVE_ORDER;
        second_blinder[31] -= 1;
        let first_seed = [2_u8; 32];
        let second_seed = [3_u8; 32];
        let first_commitment = material
            .value_commitment(&q.asset, amount, &first_blinder)
            .unwrap();
        let second_commitment = material
            .value_commitment(&q.asset, change, &second_blinder)
            .unwrap();
        (
            TargetOutput::new(
                AssetField::Explicit(asset),
                ValueField::Commitment(first_commitment),
                NonceField::Commitment(material.nonce_commitment(&first_seed).unwrap()),
                program.clone(),
            ),
            TargetOutput::new(
                AssetField::Explicit(asset),
                ValueField::Commitment(second_commitment),
                NonceField::Commitment(material.nonce_commitment(&second_seed).unwrap()),
                sponsor_program.clone(),
            ),
            OutputWitness::range_proof_only(
                material
                    .range_proof(
                        &q.asset,
                        amount,
                        &first_blinder,
                        &first_seed,
                        &first_commitment,
                        &program,
                    )
                    .unwrap(),
            ),
            OutputWitness::range_proof_only(
                material
                    .range_proof(
                        &q.asset,
                        change,
                        &second_blinder,
                        &second_seed,
                        &second_commitment,
                        &sponsor_program,
                    )
                    .unwrap(),
            ),
        )
    } else {
        (
            output(asset, amount, program),
            output(asset, change, sponsor_program),
            OutputWitness::empty(),
            OutputWitness::empty(),
        )
    };
    let (outputs, output_witnesses, payment_index) = if later {
        (
            vec![change_output, payment_output, output(asset, fee, vec![])],
            vec![change_proof, payment_proof, OutputWitness::empty()],
            1,
        )
    } else {
        (
            vec![payment_output, change_output, output(asset, fee, vec![])],
            vec![payment_proof, change_proof, OutputWitness::empty()],
            0,
        )
    };
    let unsigned = TargetTransaction::with_output_witnesses(
        2,
        vec![transaction::TargetInput::new(point, 0x8000_0001)],
        outputs,
        0,
        vec![InputWitness::new(vec![])],
        output_witnesses,
    )
    .unwrap();
    let frozen = unsigned.encode_without_witness();
    let answer = wire.ask(OperationStep::new(
        &format!("fixed-payment-sign-{}", wire.ordinal + 1),
        OperationSubject::SponsorSigning(Box::new(signing_subject(
            frozen.clone(),
            0,
            point,
            &parent.outputs()[vout as usize],
        ))),
    ));
    assert_eq!(answer.observed_layer, ObservedOutcomeLayer::Accepted);
    assert_eq!(
        answer.signature_bound_to.as_deref(),
        Some(frozen.as_slice())
    );
    let signed = TargetTransaction::with_output_witnesses(
        unsigned.version(),
        unsigned.inputs().to_vec(),
        unsigned.outputs().to_vec(),
        unsigned.lock_time(),
        vec![InputWitness::new(answer.sponsor_witness)],
        unsigned.output_witnesses().to_vec(),
    )
    .unwrap();
    let bytes = signed.encode();
    let prefix = format!("payment-{:05}", wire.ordinal + 1);
    std::fs::write(wire.directory.join(format!("{prefix}.transaction")), &bytes).unwrap();
    std::fs::write(
        wire.directory.join(format!("{prefix}-parent.transaction")),
        &parent_bytes,
    )
    .unwrap();
    let answer = wire.ask(OperationStep::new(
        &format!("fixed-payment-submit-{}", wire.ordinal + 1),
        OperationSubject::Submission(Box::new(TargetSubmissionSubject {
            transaction_bytes: bytes.clone(),
        })),
    ));
    assert_eq!(answer.observed_layer, ObservedOutcomeLayer::Accepted);
    assert_eq!(
        answer.mined_readback.as_ref().unwrap().raw_transaction,
        bytes
    );
    store(
        &wire.directory.join(format!("{prefix}.json")),
        &serde_json::json!({"key":q.key,"asset":q.asset,"value":q.value.get(),
            "payment_index":payment_index,"response":answer}),
    );
    (bytes, payment_index)
}

fn weight_case(wire: &mut Wire, count: u16, structural: bool) {
    let q = parameters(42, 60);
    let members: Vec<_> = (0..count)
        .map(|_| funded_source(wire, q, CaptureSubject::PublicSource(42)))
        .collect();
    if count == 1300 {
        if structural {
            funded_merge(wire, q, members.clone(), CaptureSubject::WeightMerge(1300));
            funded_merge(wire, q, members, CaptureSubject::StructuralWeight1300);
        } else {
            funded_merge(wire, q, members, CaptureSubject::WeightMerge(1300));
        }
    } else {
        funded_merge(wire, q, members, CaptureSubject::WeightMerge(count));
    }
}

fn weight_family(wire: &mut Wire) {
    let q = parameters(42, 60);
    funded_source(wire, q, CaptureSubject::WeightSource);
    for count in [1, 2, 251, 252, 253, 650, 1299, 1300] {
        weight_case(wire, count, true);
    }
}

fn address(bytes: &[u8]) -> reader::Address {
    let transaction = TargetTransaction::decode(bytes).unwrap();
    reader::Address(sha256d::Hash::hash(&transaction.encode_without_witness()).to_byte_array())
}

#[allow(clippy::too_many_lines)] // The complete graph and five repeated reads share exact funded bytes.
fn history_graph(wire: &mut Wire, events: usize, branching: bool) {
    let q = parameters(42, 60);
    let shape = vectors::keyed_fixed_capture::workload_shape(events, branching);
    assert_eq!(shape.events(), events);
    let subject = if branching {
        CaptureSubject::HistoryBranch(u8::try_from(events).unwrap())
    } else {
        CaptureSubject::HistoryLinear(u8::try_from(events).unwrap())
    };
    let triple = reader::Triple {
        key: q.key,
        asset: q.asset,
        value: q.value.get(),
    };
    let mut input = reader::Input {
        bytes: BTreeMap::new(),
        association: Vec::new(),
        triples: BTreeSet::from([triple]),
        retrieved: BTreeMap::new(),
        assertions: reader::Assertions::default(),
    };
    let mut accepted: Vec<Vec<u8>> = Vec::new();
    let mut expected = Vec::<BTreeSet<reader::Handle>>::new();
    for (index, parent_indices) in shape.parents.iter().enumerate() {
        let (funding, vout, _, _) = fund_sponsor(wire);
        let funding_address = address(&funding);
        input.retrieved.insert(funding_address, funding.clone());
        input.assertions.inclusion.insert(funding_address);
        input.assertions.retrieval.insert(funding_address);
        let mut parents: Vec<_> = parent_indices
            .iter()
            .map(|parent| (accepted[*parent].clone(), 0, true))
            .collect();
        parents.push((funding, vout, false));
        let bytes = event(wire, subject, q, &parents);
        let id = u32::try_from(index + 1).unwrap();
        let event_address = address(&bytes);
        input.bytes.insert(id, bytes.clone());
        input.association.push((id, event_address));
        input.assertions.inclusion.insert(event_address);
        input.assertions.retrieval.insert(event_address);
        let handle = reader::Handle {
            triple,
            output: reader::Output {
                event: id,
                position: 0,
            },
        };
        let set = if parent_indices.is_empty() {
            BTreeSet::from([handle])
        } else {
            parent_indices
                .iter()
                .flat_map(|parent| expected[*parent].iter().copied())
                .collect()
        };
        expected.push(set);
        accepted.push(bytes);
    }
    let started = Instant::now();
    let reading = reader::read(&input).unwrap();
    let cold_micros = started.elapsed().as_micros();
    let mut warm_micros = [0_u128; 5];
    for sample in &mut warm_micros {
        let started = Instant::now();
        assert_eq!(
            reader::read(&input).unwrap().memberships(),
            reading.memberships()
        );
        *sample = started.elapsed().as_micros();
    }
    for (index, set) in expected.iter().enumerate() {
        let handle = reader::Handle {
            triple,
            output: reader::Output {
                event: u32::try_from(index + 1).unwrap(),
                position: 0,
            },
        };
        assert_eq!(
            reading.query(handle),
            reader::Query::Handle {
                handle,
                membership: reader::Membership::Complete(set.clone()),
            }
        );
    }
    let filename = format!(
        "history-{}-{events}.json",
        if branching { "branch" } else { "linear" }
    );
    store(
        &wire.directory.join(filename),
        &serde_json::json!({
            "events": events,
            "edges": shape.edges(),
            "unique_bytes": reading.event_bytes(),
            "records": reading.memberships().len(),
            "cold_read_micros": cold_micros,
            "warm_read_micros": warm_micros,
        }),
    );
    tracing::info!(target: "vectors::native_test", event_count = u64::try_from(events).unwrap_or(u64::MAX),
        branching, "funded history read retained");
    if events == 4 && branching {
        input.bytes.remove(&2);
        let incomplete = reader::read(&input).unwrap();
        assert!(
            incomplete
                .memberships()
                .values()
                .any(|membership| { matches!(membership, reader::Membership::Incomplete(_)) })
        );
        store(
            &wire.directory.join("history-missing-branch.json"),
            &serde_json::json!({"subject":"history_missing_branch","boundary":"incomplete_history"}),
        );
    }
}

#[derive(Serialize, Deserialize)]
struct LegacyRecord {
    subject: CaptureSubject,
    freeze: [u8; 32],
    submitted: Vec<u8>,
    parents: Vec<Vec<u8>>,
    response: NativeOperationResponse,
}

fn legacy_submit(wire: &mut Wire, subject: CaptureSubject, bytes: Vec<u8>, parents: Vec<Vec<u8>>) {
    let freeze = vectors::keyed_final_native::capture_freeze_digest([42; 32]).unwrap();
    let prefix = format!("legacy-{:05}", wire.ordinal + 1);
    std::fs::write(wire.directory.join(format!("{prefix}.transaction")), &bytes).unwrap();
    for (index, parent) in parents.iter().enumerate() {
        std::fs::write(
            wire.directory
                .join(format!("{prefix}-parent-{index}.transaction")),
            parent,
        )
        .unwrap();
    }
    let response = wire.ask(OperationStep::new(
        &format!("legacy-submit-{}", wire.ordinal + 1),
        OperationSubject::Submission(Box::new(TargetSubmissionSubject {
            transaction_bytes: bytes.clone(),
        })),
    ));
    assert_eq!(response.observed_layer, ObservedOutcomeLayer::Accepted);
    assert_eq!(
        response.mined_readback.as_ref().unwrap().raw_transaction,
        bytes
    );
    store(
        &wire.directory.join(format!("{prefix}.json")),
        &LegacyRecord {
            subject,
            freeze,
            submitted: bytes,
            parents,
            response,
        },
    );
}

fn legacy_linked(key: [u8; 32], source: bool) -> linker::keyed_bundle::LinkedKeyedBundle {
    use architecture::{
        keyed::KeyedManifest,
        semantic::{KeyedOperationId, SemanticOperationId},
    };
    use compiler::{
        AnalysisPolicy, ProofSearchLimits, bind_input,
        keyed_plan::{PlacementSearchLimits, plan},
    };
    use linker::{keyed_bundle::link_keyed_bundle, keyed_reference::keyed_parameter_offers};
    use tapscript::{
        keyed_bundle::{KeyedBundleSelection, emit_keyed_bundle},
        keyed_constructor::KeyedLeafRecipe,
    };
    let target = reviewed_elements_tapscript().unwrap();
    let manifest = KeyedManifest::contract().unwrap();
    let declaration = realization::keyed::KeyedRealizationDraft::derive(&manifest)
        .unwrap()
        .validate(&manifest)
        .unwrap();
    let limit = std::num::NonZeroU64::new(100_000).unwrap();
    let operation = if source {
        KeyedOperationId::Source
    } else {
        KeyedOperationId::Merge
    };
    let input = bind_input(
        &manifest,
        declaration,
        vec![SemanticOperationId::Keyed(operation)],
        AnalysisPolicy::strict(ProofSearchLimits::new(limit, limit)),
    )
    .unwrap();
    let plan = plan(&input, PlacementSearchLimits::new(limit, limit)).unwrap();
    let projection = plan.project();
    let selection = KeyedBundleSelection {
        binding: projection.binding.clone(),
        scope: projection.scope.clone(),
        proof: 0,
        placements: projection.plans[0]
            .operations
            .keys()
            .map(|id| (*id, 0))
            .collect(),
    };
    let recipe = KeyedLeafRecipe::new(key);
    let emission = emit_keyed_bundle(&target, &plan, selection, recipe).unwrap();
    link_keyed_bundle(&target, &emission, &keyed_parameter_offers(&target, recipe)).unwrap()
}

#[allow(clippy::too_many_lines)] // The old-leaf request and its sponsor borrow remain live through signing.
#[allow(clippy::needless_pass_by_value)] // Own old-leaf parent bytes because the independent journal retains them.
fn legacy_event(
    wire: &mut Wire,
    member: Option<(Vec<u8>, u32)>,
    replacement: Option<(Vec<u8>, u32)>,
    confidential: bool,
) -> Vec<u8> {
    use transaction::keyed_construct::{
        KeyedConstructionRequest, KeyedOutputOffer, construct_keyed,
    };
    use transaction::keyed_inputs::{KeyedInputOffer, KeyedInputRole};
    use transaction::keyed_request::{keyed_merge_request, keyed_source_request};
    use transaction::keyed_taproot::commit_keyed_bundle;

    let key = [42_u8; 32];
    let target = reviewed_elements_tapscript().unwrap();
    let source = member.is_none();
    let commitment = commit_keyed_bundle(&target, &legacy_linked(key, source)).unwrap();
    let (sponsor_bytes, sponsor_vout, asset, sponsor_program) = fund_sponsor(wire);
    let sponsor_index = usize::from(!source);
    let fee = 10_000;
    let value = 1_000;
    let change = 1_000_000 - fee - if source { value } else { 0 };
    let sponsor_parent = TargetTransaction::decode(&sponsor_bytes).unwrap();
    let sponsor_point = Outpoint::new(txid(&sponsor_bytes), sponsor_vout).unwrap();
    let sponsor = Sponsor {
        point: sponsor_point,
        fee,
        change,
        program: sponsor_program.clone(),
        witness: RefCell::new(vec![]),
    };
    let mut retained_parents = Vec::new();
    let mut inputs = Vec::new();
    if let Some((bytes, vout)) = &member {
        retained_parents.push(bytes.clone());
        inputs.push(KeyedInputOffer {
            input_index: 0,
            parent: bytes,
            outpoint: Outpoint::new(txid(bytes), *vout).unwrap(),
            sequence: 0x8000_03e8,
            flags: 0,
            role: KeyedInputRole::Member,
            capability: None,
        });
    }
    retained_parents.push(sponsor_bytes.clone());
    inputs.push(KeyedInputOffer {
        input_index: u32::try_from(sponsor_index).unwrap(),
        parent: &sponsor_bytes,
        outpoint: sponsor_point,
        sequence: 0x8000_03e9,
        flags: 0,
        role: KeyedInputRole::Collateral,
        capability: Some(&sponsor),
    });
    let source_request;
    let merge_request;
    let request = if source {
        source_request = keyed_source_request(&target, key, &commitment, &inputs).unwrap();
        KeyedConstructionRequest::Source(&source_request)
    } else {
        merge_request = keyed_merge_request(&target, key, &commitment, &inputs).unwrap();
        KeyedConstructionRequest::Merge(&merge_request)
    };
    let structure = construct_keyed(
        &target,
        request,
        KeyedOutputOffer {
            policy_asset: asset,
            successor_value: value,
            additional_outputs: vec![output(asset, change, sponsor_program.clone())],
            fee: Some(fee),
        },
    )
    .unwrap();
    let mut inputs = structure.transaction().inputs().to_vec();
    if let Some((actual, vout)) = &replacement {
        inputs[0] = transaction::TargetInput::new(
            Outpoint::new(txid(actual), *vout).unwrap(),
            inputs[0].sequence(),
        );
        retained_parents[0].clone_from(actual);
    }
    let mut outputs = structure.transaction().outputs().to_vec();
    let mut proofs = vec![OutputWitness::empty(); 3];
    if confidential {
        let material =
            target_elements_conformance::confidential_oracles::ReferenceProofMaterial::new();
        let first_blinder = BLINDER_ONE;
        let first_seed = [2_u8; 32];
        let first_commitment = material
            .value_commitment(&POLICY_ASSET, value, &first_blinder)
            .unwrap();
        outputs[0] = TargetOutput::new(
            AssetField::Explicit(asset),
            ValueField::Commitment(first_commitment),
            NonceField::Commitment(material.nonce_commitment(&first_seed).unwrap()),
            commitment.output_program().to_vec(),
        );
        proofs[0] = OutputWitness::range_proof_only(
            material
                .range_proof(
                    &POLICY_ASSET,
                    value,
                    &first_blinder,
                    &first_seed,
                    &first_commitment,
                    commitment.output_program(),
                )
                .unwrap(),
        );
        if source {
            let mut opposite = elements::secp256k1_zkp::constants::CURVE_ORDER;
            opposite[31] -= 1;
            let second_seed = [3_u8; 32];
            let second_commitment = material
                .value_commitment(&POLICY_ASSET, change, &opposite)
                .unwrap();
            outputs[1] = TargetOutput::new(
                AssetField::Explicit(asset),
                ValueField::Commitment(second_commitment),
                NonceField::Commitment(material.nonce_commitment(&second_seed).unwrap()),
                sponsor_program.clone(),
            );
            proofs[1] = OutputWitness::range_proof_only(
                material
                    .range_proof(
                        &POLICY_ASSET,
                        change,
                        &opposite,
                        &second_seed,
                        &second_commitment,
                        &sponsor_program,
                    )
                    .unwrap(),
            );
        }
    }
    let unsigned = TargetTransaction::with_output_witnesses(
        2,
        inputs,
        outputs,
        0,
        structure.transaction().witnesses().to_vec(),
        proofs,
    )
    .unwrap();
    let frozen = unsigned.encode_without_witness();
    let response = wire.ask(OperationStep::new(
        &format!("legacy-sign-{}", wire.ordinal + 1),
        OperationSubject::SponsorSigning(Box::new(signing_subject(
            frozen.clone(),
            sponsor_index,
            sponsor_point,
            &sponsor_parent.outputs()[sponsor_vout as usize],
        ))),
    ));
    assert_eq!(response.observed_layer, ObservedOutcomeLayer::Accepted);
    assert_eq!(
        response.signature_bound_to.as_deref(),
        Some(frozen.as_slice())
    );
    let mut witnesses = unsigned.witnesses().to_vec();
    witnesses[sponsor_index] = InputWitness::new(response.sponsor_witness);
    let signed = TargetTransaction::with_output_witnesses(
        unsigned.version(),
        unsigned.inputs().to_vec(),
        unsigned.outputs().to_vec(),
        unsigned.lock_time(),
        witnesses,
        unsigned.output_witnesses().to_vec(),
    )
    .unwrap();
    transaction::keyed_sponsor::verify_keyed_sponsor(
        &unsigned.encode(),
        sponsor_index,
        &sponsor_parent.outputs()[sponsor_vout as usize],
        signed.witnesses()[sponsor_index].stack(),
    )
    .unwrap();
    let bytes = signed.encode();
    legacy_submit(
        wire,
        CaptureSubject::LegacyConfidentialValue,
        bytes.clone(),
        retained_parents,
    );
    bytes
}

#[allow(clippy::too_many_lines)] // The public copy and new sponsor signing are one retained control.
fn legacy_copy_campaign(wire: &mut Wire) {
    let confidential_source = legacy_event(wire, None, None, true);
    let explicit_control = legacy_event(wire, None, None, false);
    legacy_event(
        wire,
        Some((explicit_control, 0)),
        Some((confidential_source.clone(), 0)),
        true,
    );
    let public_input = wire.directory.join("legacy-public-input.transaction");
    let public_output = wire.directory.join("legacy-public-copy.transaction");
    std::fs::write(&public_input, &confidential_source).unwrap();
    let started = Instant::now();
    let copied = std::process::Command::new("/usr/bin/timeout")
        .args(["--kill-after=5", "60", "/usr/bin/cp"])
        .arg(&public_input)
        .arg(&public_output)
        .output()
        .unwrap();
    std::fs::write(wire.directory.join("legacy-copy.stdout"), &copied.stdout).unwrap();
    std::fs::write(wire.directory.join("legacy-copy.stderr"), &copied.stderr).unwrap();
    store(
        &wire.directory.join("legacy-copy-process.json"),
        &serde_json::json!({"command":["/usr/bin/timeout","--kill-after=5","60","/usr/bin/cp",
            public_input,public_output],"exit":copied.status.code(),
            "wall_micros":started.elapsed().as_micros()}),
    );
    assert!(copied.status.success());
    let copied_bytes = std::fs::read(&public_output).unwrap();
    assert_eq!(copied_bytes, confidential_source);
    let original = TargetTransaction::decode(&copied_bytes).unwrap();
    let (sponsor_bytes, sponsor_vout, _, sponsor_program) = fund_sponsor(wire);
    let sponsor_parent = TargetTransaction::decode(&sponsor_bytes).unwrap();
    assert_eq!(original.outputs()[1].program(), sponsor_program);
    let point = Outpoint::new(txid(&sponsor_bytes), sponsor_vout).unwrap();
    let unsigned = TargetTransaction::with_output_witnesses(
        original.version(),
        vec![transaction::TargetInput::new(point, 0x8000_03e9)],
        original.outputs().to_vec(),
        original.lock_time(),
        vec![InputWitness::new(vec![])],
        original.output_witnesses().to_vec(),
    )
    .unwrap();
    let frozen = unsigned.encode_without_witness();
    let response = wire.ask(OperationStep::new(
        &format!("legacy-copy-sign-{}", wire.ordinal + 1),
        OperationSubject::SponsorSigning(Box::new(signing_subject(
            frozen.clone(),
            0,
            point,
            &sponsor_parent.outputs()[sponsor_vout as usize],
        ))),
    ));
    assert_eq!(response.observed_layer, ObservedOutcomeLayer::Accepted);
    assert_eq!(
        response.signature_bound_to.as_deref(),
        Some(frozen.as_slice())
    );
    let signed = TargetTransaction::with_output_witnesses(
        unsigned.version(),
        unsigned.inputs().to_vec(),
        unsigned.outputs().to_vec(),
        unsigned.lock_time(),
        vec![InputWitness::new(response.sponsor_witness)],
        unsigned.output_witnesses().to_vec(),
    )
    .unwrap();
    transaction::keyed_sponsor::verify_keyed_sponsor(
        &unsigned.encode(),
        0,
        &sponsor_parent.outputs()[sponsor_vout as usize],
        signed.witnesses()[0].stack(),
    )
    .unwrap();
    legacy_submit(
        wire,
        CaptureSubject::LegacyPublicCopy,
        signed.encode(),
        vec![sponsor_bytes],
    );
}

#[allow(clippy::too_many_lines)] // The six exact census families share one exhaustive dispatch without a wildcard case.
fn campaign(wire: &mut Wire, family: CaptureFamily) {
    let selected = subjects(family);
    match family {
        CaptureFamily::Pilot => {
            assert_eq!(
                selected,
                [
                    CaptureSubject::PublicSource(42),
                    CaptureSubject::OneMemberMerge(42),
                    CaptureSubject::Floor(15),
                    CaptureSubject::Floor(14),
                    CaptureSubject::WeightMerge(252),
                ]
            );
            let q = parameters(42, 60);
            let (funding, vout, _, _) = fund_sponsor(wire);
            let source = event(wire, selected[0], q, &[(funding, vout, false)]);
            let (funding, vout, _, _) = fund_sponsor(wire);
            event(
                wire,
                selected[1],
                q,
                &[(source, 0, true), (funding, vout, false)],
            );
            let (funding, vout, _, _) = fund_sponsor(wire);
            let floor = KeyedFixedParameters {
                key: [42; 32],
                asset: POLICY_ASSET,
                value: KeyedFixedValue::new(14).unwrap(),
            };
            let floor_control = KeyedFixedParameters {
                value: KeyedFixedValue::new(15).unwrap(),
                ..floor
            };
            event(wire, selected[2], floor_control, &[(funding, vout, false)]);
            let (funding, vout, _, _) = fund_sponsor(wire);
            event(wire, selected[3], floor, &[(funding, vout, false)]);
            weight_case(wire, 252, false);
        }
        CaptureFamily::Clauses => {
            for key in [42, 43] {
                let q = parameters(key, if key == 42 { 60 } else { 1_000 });
                let (funding, vout, _, _) = fund_sponsor(wire);
                let source = event(
                    wire,
                    CaptureSubject::PublicSource(key),
                    q,
                    &[(funding, vout, false)],
                );
                let (funding, vout, _, _) = fund_sponsor(wire);
                event(
                    wire,
                    CaptureSubject::OneMemberMerge(key),
                    q,
                    &[(source, 0, true), (funding, vout, false)],
                );
                let (funding, vout, _, _) = fund_sponsor(wire);
                let first = event(
                    wire,
                    CaptureSubject::PublicSource(key),
                    q,
                    &[(funding, vout, false)],
                );
                let (funding, vout, _, _) = fund_sponsor(wire);
                let second = event(
                    wire,
                    CaptureSubject::PublicSource(key),
                    q,
                    &[(funding, vout, false)],
                );
                let (funding, vout, _, _) = fund_sponsor(wire);
                event(
                    wire,
                    CaptureSubject::TwoMemberMerge(key),
                    q,
                    &[(first, 0, true), (second, 0, true), (funding, vout, false)],
                );
            }
            let q = parameters(42, 60);
            for subject in [
                CaptureSubject::SurplusDestination(0),
                CaptureSubject::SurplusDestination(1),
                CaptureSubject::ValueMinus,
                CaptureSubject::ValuePlus,
            ] {
                let member = funded_source(wire, q, CaptureSubject::PublicSource(42));
                let (funding, vout, _, _) = fund_sponsor(wire);
                event(
                    wire,
                    subject,
                    q,
                    &[(member, 0, true), (funding, vout, false)],
                );
            }
            for subject in [
                CaptureSubject::DisplacedSuccessor,
                CaptureSubject::WrongProgram,
                CaptureSubject::WrongKey,
                CaptureSubject::MalformedControl,
                CaptureSubject::KeyPathAttempt,
            ] {
                let member = funded_source(wire, q, CaptureSubject::PublicSource(42));
                let (funding, vout, _, _) = fund_sponsor(wire);
                event(
                    wire,
                    subject,
                    q,
                    &[(member, 0, true), (funding, vout, false)],
                );
            }
            for (subject, amount, later, confidential) in [
                (CaptureSubject::MalformedPaymentValue, 59, false, false),
                (CaptureSubject::LaterPosition, 60, true, false),
            ] {
                let (payment, payment_vout) = payment(wire, q, amount, later, confidential);
                let control_parent = funded_source(wire, q, CaptureSubject::PublicSource(42));
                let (funding, vout, _, _) = fund_sponsor(wire);
                event_with_replacement(
                    wire,
                    subject,
                    q,
                    &[(control_parent, 0, true), (funding, vout, false)],
                    Some((payment, payment_vout, amount, confidential)),
                );
            }
        }
        CaptureFamily::Policy => {
            let floor_14 = KeyedFixedParameters {
                key: [42; 32],
                asset: POLICY_ASSET,
                value: KeyedFixedValue::new(14).unwrap(),
            };
            let floor_15 = KeyedFixedParameters {
                value: KeyedFixedValue::new(15).unwrap(),
                ..floor_14
            };
            funded_source(wire, floor_15, CaptureSubject::Floor(15));
            funded_source(wire, floor_14, CaptureSubject::Floor(14));
            funded_source(wire, floor_14, CaptureSubject::ZeroFeeFloor);
        }
        CaptureFamily::Scaling => weight_family(wire),
        CaptureFamily::Histories => {
            for events in [4, 16, 64] {
                history_graph(wire, events, false);
                history_graph(wire, events, true);
            }
        }
        CaptureFamily::LegacyCopy => legacy_copy_campaign(wire),
    }
}

fn provenance(prepared: &serde_json::Value, freeze_identity: [u8; 32]) -> CaptureProvenance {
    CaptureProvenance {
        freeze_identity,
        executable_digest: hex32(field(prepared, "capture_binary")),
        executor_digest: hex32(field(prepared, "executor")),
        dependency_lock_digest: hex32(field(prepared, "dependency_lock")),
        daemon_digest: hex32(field(prepared, "daemon_digest")),
        framework_revision: field(prepared, "framework").into(),
        zk_library_digest: hex32(field(prepared, "zk_library_digest")),
        image_identity: field(prepared, "image").into(),
    }
}

#[derive(Clone, Debug, Serialize)]
struct RuntimeObserved {
    daemon_digest: [u8; 32],
    zk_library_digest: [u8; 32],
    executor_digest: [u8; 32],
    executable_digest: [u8; 32],
    dependency_lock_digest: [u8; 32],
    zk_library_loading: &'static str,
}

fn runtime_observed(prepared: &serde_json::Value) -> RuntimeObserved {
    let paths = [
        ("/elements/bin/elementsd", "daemon_digest"),
        ("/elements/lib/libsecp256k1.so", "zk_library_digest"),
        ("/src/scripts/elements-native-executor.py", "executor"),
        ("/capture-build/keyed_fixed_capture", "capture_binary"),
        ("/capture-build/Cargo.lock", "dependency_lock"),
    ];
    let digests = paths.map(|(path, field_name)| {
        let digest = tagged_sha256(&std::fs::read(path).unwrap());
        assert_eq!(
            digest,
            hex32(field(prepared, field_name)),
            "runtime provenance drift: {path}"
        );
        digest
    });
    let executable_digest =
        tagged_sha256(&std::fs::read(std::env::current_exe().unwrap()).unwrap());
    assert_eq!(executable_digest, digests[3]);
    assert!(std::env::var_os("ELEMENTS_NATIVE_EXECUTOR_ZK_LIBRARY").is_none());
    RuntimeObserved {
        daemon_digest: digests[0],
        zk_library_digest: digests[1],
        executor_digest: digests[2],
        executable_digest,
        dependency_lock_digest: digests[4],
        zk_library_loading: "present-not-loaded",
    }
}

fn journal(
    submission: &Submission,
    prepared: &serde_json::Value,
    handshake: &target_elements_conformance::protocol::ExecutorHandshake,
    runtime: &RuntimeObserved,
) -> JournalRow {
    let expected = provenance(prepared, submission.freeze);
    let mut observed = expected.clone();
    observed.framework_revision = handshake.framework_revision.clone().unwrap();
    observed.daemon_digest = runtime.daemon_digest;
    observed.zk_library_digest = runtime.zk_library_digest;
    observed.executor_digest = runtime.executor_digest;
    observed.executable_digest = runtime.executable_digest;
    observed.dependency_lock_digest = runtime.dependency_lock_digest;
    JournalRow {
        request: submission.request.clone(),
        response: submission.response.clone(),
        submitted: submission.submitted.clone(),
        submitted_digest: tagged_sha256(&submission.submitted),
        parent_digests: submission
            .parents
            .iter()
            .map(|bytes| tagged_sha256(bytes))
            .collect(),
        parent_bytes: submission.parents.clone(),
        readback: submission
            .response
            .mined_readback
            .as_ref()
            .map(|readback| readback.raw_transaction.clone()),
        expected_provenance: expected,
        observed_provenance: observed,
    }
}

fn binding() -> ReviewedDevelopmentBinding {
    let target = reviewed_elements_tapscript().unwrap();
    validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            hex32("75e823f7c5c70ddfbd9584f90f67298f2570907947829828066c7047b21b53b1"),
            hex32("209577bda6bf4b5804bd46f8621580dd6d4e8bfa2d190e1c50e932492baca07d"),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .unwrap()
}

fn executor(directory: &Path, prepared: &serde_json::Value) -> PathBuf {
    let tip = field(prepared, "framework");
    let script = format!(
        "#!/bin/sh\nset -eu\nexec /usr/bin/python3 /src/scripts/elements-native-executor.py --framework /elements/src/test/functional --elementsd /elements/bin/elementsd --elements-cli /elements/bin/elements-cli --fee-policy node-defaults --maturity-observation --enable-wallet --datadir '{}/regtest' --network-id 75e823f7c5c70ddfbd9584f90f67298f2570907947829828066c7047b21b53b1 --intended-executed-tip {tip} --upstream-base {tip} \"$@\"\n",
        directory.display()
    );
    let path = directory.join("executor");
    std::fs::write(&path, script).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    path
}

/// Execute one operator-selected family against an isolated node.
/// # Panics
/// Panics if preparation, provenance, any response, or journal admission refuses.
#[allow(clippy::too_many_lines)] // Keep preparation, admission and the final marker in one ordered lifecycle.
pub fn run(family: CaptureFamily) {
    let directory = Path::new("/capture");
    let timing_file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join("phase-timing.jsonl"))
        .unwrap();
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(timing_file)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let _span = tracing::info_span!(target: "vectors::native_test", "keyed_fixed_capture", ?family)
        .entered();
    assert_eq!(
        std::env::var("TRIPOD_FIXED_CAPTURE_DIR").unwrap(),
        "/capture"
    );
    assert_eq!(
        std::env::var("TRIPOD_FIXED_FEE_POLICY").unwrap(),
        "node-defaults"
    );
    let prepared: serde_json::Value =
        serde_json::from_slice(&std::fs::read("/capture-build/prepared.json").unwrap()).unwrap();
    assert_eq!(field(&prepared, "trust"), "reviewed_non_mock");
    let runtime = runtime_observed(&prepared);
    store(&directory.join("observed-provenance.json"), &runtime);
    let census = subjects(family)
        .into_iter()
        .map(|subject| serde_json::json!({"subject":subject,"boundary":subject.boundary()}))
        .collect::<Vec<_>>();
    store(
        &directory.join("CENSUS.json"),
        &serde_json::json!({"family":family,"subjects":census}),
    );
    let adapter = executor(directory, &prepared);
    let config = ExecutorConfiguration::new(
        &adapter,
        ExecutorTrust::ReviewedNonMock,
        Duration::from_secs(600),
        ExecutorDiagnostics::in_directory(&directory.join("diagnostics")),
    )
    .with_expected_provenance(
        ExpectedExecutorProvenance::new(
            field(&prepared, "framework"),
            field(&prepared, "framework"),
            [],
        )
        .unwrap(),
    );
    let mut planner = Planner::new(family, directory);
    let mut capture = NativeOperationCapture::default();
    let started = Instant::now();
    let result = execute_operations_captured(
        &reviewed_elements_tapscript().unwrap(),
        &binding(),
        &config,
        &mut planner,
        &mut capture,
    );
    drop(planner);
    store(
        &directory.join("transport-result.json"),
        &format!("{result:?}"),
    );
    let transcript = result.expect("native transport; partial evidence retained");
    transcript.provenance_agreement().unwrap().unwrap();
    let handshake = capture.handshake().unwrap();
    assert_eq!(handshake.protocol_schema, NATIVE_PROTOCOL_SCHEMA);
    assert_eq!(
        handshake.framework_revision.as_deref(),
        Some(field(&prepared, "framework"))
    );
    assert_eq!(handshake.node_version, field(&prepared, "daemon_version"));
    store(&directory.join("executor-handshake.json"), handshake);
    tracing::info!(target: "vectors::native_test", phase = "transport",
        wall_us = u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX),
        "fixed-form transport completed");
    let admission_started = Instant::now();
    if family == CaptureFamily::LegacyCopy {
        let expected = vectors::keyed_final_native::capture_freeze_digest([42; 32]).unwrap();
        let mut rows = Vec::new();
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy();
            if name.starts_with("legacy-")
                && name.as_bytes().get(7).is_some_and(u8::is_ascii_digit)
                && path
                    .extension()
                    .is_some_and(|extension| extension == "json")
            {
                let row: LegacyRecord =
                    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
                assert_eq!(row.freeze, expected);
                assert_eq!(row.response.observed_layer, ObservedOutcomeLayer::Accepted);
                assert_eq!(
                    row.response.mined_readback.unwrap().raw_transaction,
                    row.submitted
                );
                rows.push(row.subject);
            }
        }
        assert_eq!(rows.len(), 4);
        assert!(rows.contains(&CaptureSubject::LegacyConfidentialValue));
        assert!(rows.contains(&CaptureSubject::LegacyPublicCopy));
        store(
            &directory.join("legacy-provenance.json"),
            &serde_json::json!({
                "legacy_freeze":expected,
                "capture_binary":field(&prepared,"capture_binary"),
                "executor":field(&prepared,"executor"),
                "daemon_digest":field(&prepared,"daemon_digest"),
                "framework":field(&prepared,"framework"),
                "zk_library_digest":field(&prepared,"zk_library_digest"),
                "image":field(&prepared,"image"),
                "observed_framework":handshake.framework_revision,
                "observed_daemon":handshake.node_version,
            }),
        );
        std::fs::write(
            directory.join("admitted"),
            b"old-leaf copy journal admitted separately\n",
        )
        .unwrap();
        return;
    }
    let mut seen = Vec::new();
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if !path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("subject-")
            || path.extension().is_none_or(|extension| extension != "json")
        {
            continue;
        }
        let row: Submission = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        seen.push(row.subject);
        let parameters = KeyedFixedParameters {
            key: row.key,
            asset: row.asset,
            value: KeyedFixedValue::new(row.value).unwrap(),
        };
        let freeze = FreezeManifest::offer(parameters)
            .unwrap()
            .validate(parameters)
            .unwrap();
        assert_eq!(row.freeze, freeze.identity());
        let prepared_provenance = provenance(&prepared, row.freeze);
        let observed_row = journal(&row, &prepared, handshake, &runtime);
        let control = row.control.as_ref().map(|control| {
            let control_parameters = KeyedFixedParameters {
                key: control.key,
                asset: control.asset,
                value: KeyedFixedValue::new(control.value).unwrap(),
            };
            let freeze = FreezeManifest::offer(control_parameters)
                .unwrap()
                .validate(control_parameters)
                .unwrap();
            assert_eq!(control.freeze, freeze.identity());
            (freeze, journal(control, &prepared, handshake, &runtime))
        });
        let binding = JournalBinding {
            family,
            subject: row.subject,
            sponsor_index: row.sponsor_index,
            prepared_provenance: &prepared_provenance,
            control: control
                .as_ref()
                .map(|(control_freeze, control_row)| PairControl {
                    subject: row.control.as_ref().unwrap().subject,
                    freeze: control_freeze,
                    row: control_row,
                    sponsor_index: row.control.as_ref().unwrap().sponsor_index,
                }),
        };
        bind_journal(&freeze, &observed_row, &binding).unwrap_or_else(|refusal| {
            panic!(
                "journal admission refused row {} subject {:?}: {refusal:?}",
                path.display(),
                row.subject
            )
        });
    }
    for selected in subjects(family) {
        if selected == CaptureSubject::WeightMerge(1300) {
            assert!(directory.join("typed-weight-1300.json").is_file());
        } else if selected == CaptureSubject::HistoryMissingBranch {
            assert!(directory.join("history-missing-branch.json").is_file());
        } else {
            assert!(
                seen.contains(&selected),
                "unsubmitted selected subject: {selected:?}"
            );
        }
    }
    std::fs::write(
        directory.join("admitted"),
        b"fixed-form native journal admitted\n",
    )
    .unwrap();
    tracing::info!(target: "vectors::native_test", phase = "admission",
        wall_us = u64::try_from(admission_started.elapsed().as_micros()).unwrap_or(u64::MAX),
        "fixed-form admission completed");
    tracing::info!(target: "vectors::native_test", ?family,
        subject_count = u64::try_from(seen.len()).unwrap_or(u64::MAX),
        "fixed-form family admitted");
    tracing::info!(target: "vectors::native_test", ?family,
        wall_us = u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX),
        "fixed-form native family completed");
}
