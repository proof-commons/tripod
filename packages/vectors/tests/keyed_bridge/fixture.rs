//! Disposable mock funding/signing support. No native acceptance is asserted.
//! The mock decodes through rust-elements and transaction, not the Python framework.
//! It requires explicit fields for assertions, whereas the script compares serialized fields.
//! Its registration compares the readback txid but has no separate accepted-txid argument;
//! block hashes are test labels, not hexadecimal chain identities. These differences
//! limit mock evidence; the first native pilot must exercise the framework decoder.
use crate::keyed_reproduction::fixture as prior;
use elements::bitcoin::hashes::{Hash, hash160};
use elements::secp256k1_zkp::{Message, PublicKey, Secp256k1, SecretKey};
use std::cell::{Cell, RefCell};
use std::io::Write as _;
use target_elements_conformance::constructor::tagged::sha256;
use target_elements_conformance::executor::OperationStep;
use target_elements_conformance::protocol::*;
use transaction::keyed_construct::*;
use transaction::keyed_inputs::*;
use transaction::keyed_request::*;
use transaction::keyed_taproot::commit_keyed_bundle;
use transaction::*;
use vectors::keyed_final_native::*;

#[derive(Clone)]
pub struct Session {
    pub exchanges: Vec<(OperationStep, NativeOperationResponse)>,
    pub finals: Vec<FinalSubject>,
    coins: std::collections::BTreeMap<(String, u32), TargetOutput>,
    program: Vec<u8>,
    change_coins: std::collections::BTreeSet<(String, u32)>,
    serial: u32,
    submitted: std::collections::BTreeSet<Vec<u8>>,
}
const fn output(value: u64, program: Vec<u8>) -> TargetOutput {
    TargetOutput::new(
        AssetField::Explicit(AssetId::from_internal([3; 32])),
        ValueField::Explicit(value),
        NonceField::Null,
        program,
    )
}
fn readback(bytes: &[u8], height: u32) -> MinedFundingReadback {
    let t: elements::Transaction = elements::encode::deserialize(bytes).unwrap();
    MinedFundingReadback {
        transaction_id: t.txid().to_string(),
        witness_transaction_id: t.wtxid().to_string(),
        block_hash: format!("mock-block-{height}"),
        block_height: height,
        raw_transaction: bytes.to_vec(),
    }
}
fn response(step: &OperationStep) -> NativeOperationResponse {
    NativeOperationResponse {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: step.case().clone(),
        observed_layer: ObservedOutcomeLayer::Accepted,
        observed_detail: Some("mock only; no native claim".into()),
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
    }
}
impl Session {
    pub fn new() -> Self {
        let secp = Secp256k1::new();
        let secret = SecretKey::from_slice(&[1; 32]).unwrap();
        let public = PublicKey::from_secret_key(&secp, &secret).serialize();
        let mut program = vec![0, 20];
        program.extend_from_slice(hash160::Hash::hash(&public).as_byte_array());
        Self {
            exchanges: vec![],
            finals: vec![],
            coins: std::collections::BTreeMap::new(),
            program,
            serial: 0,
            change_coins: std::collections::BTreeSet::new(),
            submitted: std::collections::BTreeSet::new(),
        }
    }
    pub fn fund(&mut self) -> Vec<u8> {
        self.serial += 1;
        let step = OperationStep::new(
            &format!("fund-{}", self.serial),
            OperationSubject::SponsorFunding(Box::new(TargetSponsorFundingSubject {
                sponsor_outputs: 1,
                amount_per_sponsor_output: 1_000_000,
            })),
        );
        // Asymmetric output index 1, not a coinbase/issuance parent.
        let tx = TargetTransaction::new(
            2,
            vec![TargetInput::new(
                Outpoint::new(
                    Txid::from_internal([u8::try_from(self.serial).unwrap(); 32]),
                    7,
                )
                .unwrap(),
                900 + self.serial,
            )],
            vec![
                output(17, vec![0x51]),
                output(1_000_000, self.program.clone()),
            ],
            self.serial,
            vec![InputWitness::new(vec![])],
        )
        .unwrap();
        let raw = tx.encode();
        let rb = readback(&raw, 100 + self.serial);
        let funded = FundedOutput {
            outpoint: WireOutpoint {
                txid: rb.transaction_id.clone(),
                vout: 1,
            },
            asset: "03".repeat(32),
            amount_satoshis: 1_000_000,
            script: crate::common::hex_bytes(&self.program),
        };
        assert_eq!(
            target_elements_conformance::sponsor_parent::bind_sponsor_parent(&rb, &funded).unwrap(),
            raw
        );
        self.coins
            .insert((rb.transaction_id.clone(), 1), tx.outputs()[1].clone());
        let mut answer = response(&step);
        answer.funded_outputs.push(funded);
        answer.mined_readback = Some(rb);
        self.exchanges.push((step, answer));
        raw
    }
    fn sign(&mut self, step: OperationStep) -> Vec<Vec<u8>> {
        let OperationSubject::SponsorSigning(subject) = step.subject() else {
            panic!("signing step")
        };
        let coin = self.checked_sponsor(subject).unwrap();
        let index = usize::from(subject.sponsor_input_index);
        let digest = prior::reference_digest(&subject.finalized_transaction, index, coin);
        assert_eq!(
            digest,
            transaction::keyed_sponsor::keyed_sponsor_digest(
                &subject.finalized_transaction,
                index,
                coin
            )
            .unwrap()
        );
        let secp = Secp256k1::new();
        let secret = SecretKey::from_slice(&[1; 32]).unwrap();
        let mut sig = secp
            .sign_ecdsa(&Message::from_digest(digest), &secret)
            .serialize_der()
            .to_vec();
        sig.push(1);
        let witness = vec![
            sig,
            PublicKey::from_secret_key(&secp, &secret)
                .serialize()
                .to_vec(),
        ];
        let mut answer = response(&step);
        answer.sponsor_witness.clone_from(&witness);
        answer.signature_bound_to = Some(subject.finalized_transaction.clone());
        self.exchanges.push((step, answer));
        witness
    }
    pub fn event(
        &mut self,
        key: [u8; 32],
        parents: &[(Vec<u8>, u32, bool)],
        name: &str,
    ) -> Vec<u8> {
        let target = target_elements::reviewed_elements_tapscript().unwrap();
        let source = parents.iter().all(|p| !p.2);
        let commitment = commit_keyed_bundle(&target, &prior::linked(key, source)).unwrap();
        let signer = parents.iter().position(|p| !p.2).unwrap();
        assert_eq!(parents.iter().filter(|p| !p.2).count(), 1);
        let decoded: Vec<_> = parents
            .iter()
            .map(|(bytes, _, _)| TargetTransaction::decode(bytes).unwrap())
            .collect();
        let total: u64 = parents
            .iter()
            .zip(&decoded)
            .map(
                |((_, vout, _), t)| match t.outputs()[*vout as usize].value() {
                    ValueField::Explicit(v) => v,
                    _ => panic!("explicit"),
                },
            )
            .sum();
        let fee = 10_000;
        let successor = 1_000;
        let change = total - fee - successor;
        let point = Outpoint::new(
            Txid::from_internal(prior::txid(&parents[signer].0)),
            parents[signer].1,
        )
        .unwrap();
        let cap = Replay {
            point,
            change,
            program: self.program.clone(),
            offers: Cell::new(0),
            signs: Cell::new(0),
            frozen: RefCell::new(None),
            witness: RefCell::new(vec![]),
            captured: RefCell::new(None),
        };
        let inputs: Vec<_> = parents
            .iter()
            .enumerate()
            .map(|(i, (bytes, vout, member))| KeyedInputOffer {
                input_index: u32::try_from(i).unwrap(),
                parent: bytes,
                outpoint: Outpoint::new(Txid::from_internal(prior::txid(bytes)), *vout).unwrap(),
                sequence: 0x8000_0000 | (1000 + u32::try_from(i).unwrap()),
                flags: 0,
                role: if *member {
                    KeyedInputRole::Member
                } else {
                    KeyedInputRole::Collateral
                },
                capability: if *member { None } else { Some(&cap) },
            })
            .collect();
        let source_request;
        let merge_request;
        let request = if source {
            source_request = keyed_source_request(&target, key, &commitment, &inputs).unwrap();
            KeyedConstructionRequest::Source(&source_request)
        } else {
            merge_request = keyed_merge_request(&target, key, &commitment, &inputs).unwrap();
            KeyedConstructionRequest::Merge(&merge_request)
        };
        let offer = KeyedOutputOffer {
            policy_asset: AssetId::from_internal([3; 32]),
            successor_value: successor,
            additional_outputs: vec![output(change, self.program.clone())],
            fee: Some(fee),
        };
        let structure = construct_keyed(&target, request, offer.clone()).unwrap();
        let frozen = freeze_final_subject(request, &structure, signer).unwrap();
        assert_eq!(
            freeze_final_subject(request, &structure, usize::from(u16::MAX) + 1).unwrap_err(),
            BridgeRefusal::SigningIndex
        );
        cap.frozen.replace(Some(frozen.clone()));
        cap.witness.replace(self.sign(frozen.signing_step()));
        let policy = KeyedPolicy::new(
            "mock explicit policy reading; not observed relay policy".into(),
            AssetId::from_internal([3; 32]),
            [100; 3],
            vec![100, 100, 1],
            [400_000; 3],
            0,
            72,
        )
        .unwrap();
        let admitted = admit_keyed(&target, request, offer, &policy).unwrap();
        assert_eq!(cap.offers.get(), 1);
        assert_eq!(cap.signs.get(), 1);
        let final_subject = complete_final_subject(&frozen, request, &admitted).unwrap();
        assert_eq!(final_subject.frozen().inputs().len(), parents.len());
        let script = if source {
            commitment.output_program()
        } else {
            commitment.leaf()
        };
        self.submit(final_subject, script, name)
    }
    fn submit(&mut self, final_subject: FinalSubject, script: &[u8], name: &str) -> Vec<u8> {
        let second = *final_subject.frozen().key() == vectors::keyed_final_manifest::SECOND_KEY;
        assert!(second || *final_subject.frozen().key() == [42; 32]);
        let expected = match (name, second) {
            ("source-a", false) => {
                "9fe13dcc13850d10a65568cfb0bad753bc2f82e57cd0f84a2ec0e0ea5eca5e0a"
            }
            ("source-a", true) => {
                "91cf8e6d3d69bb0587611e446b47fdbe134dd55ccd20523efb5c879287775db6"
            }
            ("single-ordinary-edge", false) => {
                "5b43befe181f4d2f1f3f1b109bdb8f7e37f207d56418d693235c364286838a59"
            }
            ("single-ordinary-edge", true) => {
                "4e024ce41e130de19c49ba1a257002ed21ce92c8676faa84ed88814cc51913fe"
            }
            ("source-b", false) => {
                "e76f7b900257bf3b540587b13a9b57e7cb5661b1741bbd8b9ed34e73b919d50e"
            }
            ("source-b", true) => {
                "6c2bd6ec12090416a82d2e014216c5be6f1195129021df2053f4d553b7ac12ce"
            }
            ("multi-nonzero-members", false) => {
                "cab7142cd5ee22ca2093ecab68c9a98b1e9dbcdbb1131743884eb4b97e28fe2d"
            }
            ("multi-nonzero-members", true) => {
                "a491bf34f385af3a9c803d570bc5306dc85153edd819b2fa798643d420d0e976"
            }
            _ => panic!("unnamed mock subject"),
        };
        assert_eq!(
            crate::common::hex_bytes(&final_subject.digest().0),
            expected
        );
        let step = final_subject.submission_step();
        let wire = NativeOperationRequest {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case: step.case().clone(),
            subject: step.subject().clone(),
        };
        final_subject
            .validate_request(OperationRevision::Nine, &wire)
            .unwrap();
        let mut wrong = wire.clone();
        if let OperationSubject::Submission(s) = &mut wrong.subject {
            s.transaction_bytes[0] ^= 1;
        }
        assert_eq!(
            final_subject.validate_request(OperationRevision::Nine, &wrong),
            Err(BridgeRefusal::SubmissionBytes)
        );
        let mut wrong_case = wire.clone();
        wrong_case.case.step.push_str("-unsealed");
        assert_eq!(
            final_subject.validate_request(OperationRevision::Nine, &wrong_case),
            Err(BridgeRefusal::SubmissionCase)
        );
        let mut wrong_schema = wire.clone();
        wrong_schema.schema = 8;
        assert_eq!(
            final_subject.validate_request(OperationRevision::Nine, &wrong_schema),
            Err(BridgeRefusal::SubmissionSchema)
        );
        let mut wrong_script = wire;
        if let OperationSubject::Submission(subject) = &mut wrong_script.subject {
            let offset = subject
                .transaction_bytes
                .windows(script.len())
                .position(|window| window == script)
                .unwrap();
            subject.transaction_bytes[offset] ^= 1;
        }
        assert_eq!(
            final_subject.validate_request(OperationRevision::Nine, &wrong_script),
            Err(BridgeRefusal::SubmissionBytes)
        );
        let bytes = final_subject.bytes().to_vec();
        let rb = readback(&bytes, 200 + u32::try_from(self.finals.len()).unwrap());
        // Only this session's submission can register mined owned change.
        self.submitted.insert(bytes.clone());
        self.register(&bytes, &rb, ObservedOutcomeLayer::Accepted)
            .unwrap();
        let mut answer = response(&step);
        answer.accepted_txid = Some(rb.transaction_id.clone());
        answer.mined_readback = Some(rb);
        self.exchanges.push((step, answer));
        writeln!(
            std::io::stdout().lock(),
            "bridge subject {name} key={} digest={} bytes={}",
            crate::common::hex_bytes(&final_subject.frozen().key()[..4]),
            crate::common::hex_bytes(&sha256(&bytes)),
            bytes.len()
        )
        .unwrap();
        self.finals.push(final_subject);
        bytes
    }
}
struct Replay {
    point: Outpoint,
    change: u64,
    program: Vec<u8>,
    offers: Cell<usize>,
    signs: Cell<usize>,
    frozen: RefCell<Option<FrozenFinalSubject>>,
    witness: RefCell<Vec<Vec<u8>>>,
    captured: RefCell<Option<SponsorSigningRequest>>,
}
impl SponsorCapability for Replay {
    fn offer(&self) -> SponsorOffer {
        self.offers.set(self.offers.get() + 1);
        SponsorOffer::new(
            [self.point],
            10_000,
            Some(ValueField::Explicit(self.change)),
        )
        .unwrap()
    }
    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        Some((0, self.program[2..].to_vec()))
    }
    fn sign(&self, r: &SponsorSigningRequest) -> Option<SponsorSignature> {
        self.signs.set(self.signs.get() + 1);
        self.captured.replace(Some(r.clone()));
        self.frozen
            .borrow()
            .as_ref()
            .unwrap()
            .bind_signing_request(r)
            .unwrap();
        Some(SponsorSignature::new(
            r.transaction().to_vec(),
            self.witness.borrow().clone(),
        ))
    }
}
pub fn history(key: [u8; 32]) -> Session {
    let mut session = Session::new();
    let funding = session.fund();
    let a = session.event(key, &[(funding, 1, false)], "source-a");
    let single = session.event(
        key,
        &[(a.clone(), 1, false), (a, 0, true)],
        "single-ordinary-edge",
    );
    let funding = session.fund();
    let b = session.event(key, &[(funding, 1, false)], "source-b");
    session.event(
        key,
        &[(single.clone(), 0, true), (single, 1, false), (b, 0, true)],
        "multi-nonzero-members",
    );
    session
}

pub fn refusals() {
    use transaction::keyed_request::KeyedRequestRefusal;
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let key = [42; 32];
    let commitment = commit_keyed_bundle(&target, &prior::linked(key, false)).unwrap();
    let parent = prior::parent(commitment.output_program(), 71);
    let point = Outpoint::new(Txid::from_internal(prior::txid(&parent)), 0).unwrap();
    let offer = |bytes, point, flags, role| KeyedInputOffer {
        input_index: 0,
        parent: bytes,
        outpoint: point,
        sequence: 1000,
        flags,
        role,
        capability: None,
    };
    let accepted = offer(parent.as_slice(), point, 0, KeyedInputRole::Member);
    keyed_merge_request(&target, key, &commitment, &[accepted]).unwrap();
    let cases = [
        (offer(&[], point, 0, KeyedInputRole::Member), "decode"),
        (
            offer(
                &parent,
                Outpoint::new(Txid::from_internal([9; 32]), 0).unwrap(),
                0,
                KeyedInputRole::Member,
            ),
            "parent_hash",
        ),
        (
            offer(&parent, point, 0, KeyedInputRole::Collateral),
            "false_collateral_role",
        ),
        (
            offer(&parent, point, 0x40, KeyedInputRole::Member),
            "unsupported_spending_input_form",
        ),
    ];
    for (subject, expected) in cases {
        let error = keyed_merge_request(&target, key, &commitment, &[subject]).unwrap_err();
        let KeyedRequestRefusal::Input { refusal, .. } = error else {
            panic!("input boundary: {error:?}")
        };
        assert_eq!(refusal.refusal_kind(), expected);
    }
    let foreign = commit_keyed_bundle(&target, &prior::linked([43; 32], false)).unwrap();
    let error = keyed_merge_request(
        &target,
        key,
        &foreign,
        &[offer(&parent, point, 0, KeyedInputRole::Member)],
    )
    .unwrap_err();
    assert_eq!(error.refusal_kind(), "link");
}

/// Test-local mock refusals; the real executor has its own boundary controls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MockAuthorityRefusal {
    NotSubmitted,
    NotAccepted,
    NotMined,
    Size,
    Decode,
    NonCanonical,
    Bytes,
    Identity,
    WitnessIdentity,
    SigningDecode,
    SigningIndex,
    SigningOutpoint,
    Assertion,
    Coin,
    Asset,
    Value,
    Program,
}
impl Session {
    fn register(
        &mut self,
        raw: &[u8],
        rb: &MinedFundingReadback,
        layer: ObservedOutcomeLayer,
    ) -> Result<(), MockAuthorityRefusal> {
        use MockAuthorityRefusal as R;
        if !self.submitted.contains(raw) {
            return Err(R::NotSubmitted);
        }
        if layer != ObservedOutcomeLayer::Accepted {
            return Err(R::NotAccepted);
        }
        if rb.block_height == 0 || rb.block_hash.is_empty() {
            return Err(R::NotMined);
        }
        if rb.raw_transaction.len()
            > target_elements_conformance::sponsor_parent::MAXIMUM_SPONSOR_PARENT_BYTES
        {
            return Err(R::Size);
        }
        let tx: elements::Transaction =
            elements::encode::deserialize(&rb.raw_transaction).map_err(|_| R::Decode)?;
        if elements::encode::serialize(&tx) != rb.raw_transaction {
            return Err(R::NonCanonical);
        }
        if rb.raw_transaction != raw {
            return Err(R::Bytes);
        }
        if tx.txid().to_string() != rb.transaction_id {
            return Err(R::Identity);
        }
        if tx.wtxid().to_string() != rb.witness_transaction_id {
            return Err(R::WitnessIdentity);
        }
        let decoded = TargetTransaction::decode(raw).map_err(|_| R::Decode)?;
        for (i, output) in decoded.outputs().iter().enumerate() {
            if output.program() == self.program {
                self.change_coins
                    .insert((rb.transaction_id.clone(), u32::try_from(i).unwrap()));
                self.coins.insert(
                    (rb.transaction_id.clone(), u32::try_from(i).unwrap()),
                    output.clone(),
                );
            }
        }
        self.submitted.remove(raw);
        Ok(())
    }
    fn checked_sponsor(
        &self,
        subject: &TargetSponsorSigningSubject,
    ) -> Result<&TargetOutput, MockAuthorityRefusal> {
        use MockAuthorityRefusal as R;
        let tx = TargetTransaction::decode(&subject.finalized_transaction)
            .map_err(|_| R::SigningDecode)?;
        let input = tx
            .inputs()
            .get(usize::from(subject.sponsor_input_index))
            .ok_or(R::SigningIndex)?;
        if input.outpoint().txid().to_target_display() != subject.sponsor_outpoint.txid
            || input.outpoint().index() != subject.sponsor_outpoint.vout
        {
            return Err(R::SigningOutpoint);
        }

        let coin = self
            .coins
            .get(&(
                subject.sponsor_outpoint.txid.clone(),
                subject.sponsor_outpoint.vout,
            ))
            .ok_or(R::Coin)?;
        if self.change_coins.contains(&(
            subject.sponsor_outpoint.txid.clone(),
            subject.sponsor_outpoint.vout,
        )) && subject.spent_output.is_none()
        {
            return Err(R::Assertion);
        }
        if let Some(stated) = &subject.spent_output {
            let AssetField::Explicit(asset) = coin.asset() else {
                return Err(R::Asset);
            };
            let ValueField::Explicit(value) = coin.value() else {
                return Err(R::Value);
            };
            let mut asset_field = vec![1];
            asset_field.extend_from_slice(asset.internal());
            let mut value_field = vec![1];
            value_field.extend_from_slice(&value.to_be_bytes());
            if stated.asset_field != asset_field {
                return Err(R::Asset);
            }
            if stated.value_field != value_field {
                return Err(R::Value);
            }
            if stated.program != coin.program() {
                return Err(R::Program);
            }
        }
        Ok(coin)
    }
}

/// Independent mock authority controls beside its accepted session-owned output.
pub fn authority_controls() {
    use MockAuthorityRefusal as R;
    type Mutate = fn(&mut WireSpentOutput);
    let mut session = Session::new();
    let raw = session.fund();
    session.coins.clear();
    let rb = readback(&raw, 111);
    assert_eq!(
        session.register(&raw, &rb, ObservedOutcomeLayer::Accepted),
        Err(R::NotSubmitted)
    );
    session.submitted.insert(raw.clone());
    let pending = session.clone();
    assert_eq!(
        session.register(&raw, &rb, ObservedOutcomeLayer::RelayPolicyRejection),
        Err(R::NotAccepted)
    );
    let mut unmined = rb.clone();
    unmined.block_height = 0;
    assert_eq!(
        session.register(&raw, &unmined, ObservedOutcomeLayer::Accepted),
        Err(R::NotMined)
    );
    session
        .register(&raw, &rb, ObservedOutcomeLayer::Accepted)
        .unwrap();
    assert_eq!(session.coins.len(), 1);
    let mut asset = vec![1];
    asset.extend_from_slice(&[3; 32]);
    let mut value = vec![1];
    value.extend_from_slice(&1_000_000_u64.to_be_bytes());
    let control = TargetSponsorSigningSubject {
        finalized_transaction: TargetTransaction::new(
            2,
            vec![TargetInput::new(
                Outpoint::new(Txid::from_internal(prior::txid(&raw)), 1).unwrap(),
                19,
            )],
            vec![output(999_000, session.program.clone())],
            0,
            vec![InputWitness::new(vec![])],
        )
        .unwrap()
        .encode_without_witness(),
        sponsor_input_index: 0,
        sponsor_outpoint: WireOutpoint {
            txid: rb.transaction_id.clone(),
            vout: 1,
        },
        spent_output: Some(WireSpentOutput {
            asset_field: asset,
            value_field: value,
            program: session.program.clone(),
        }),
        sighash_profile: WireSighashProfile::AllInputsAllOutputs,
    };
    assert!(session.checked_sponsor(&control).is_ok());
    assert_eq!(Session::new().checked_sponsor(&control), Err(R::Coin));
    let mut other_output = control.clone();
    other_output.sponsor_outpoint.vout = 0;
    assert_eq!(
        session.checked_sponsor(&other_output),
        Err(R::SigningOutpoint)
    );
    let mut absent = control.clone();
    absent.sponsor_input_index = 1;
    assert_eq!(session.checked_sponsor(&absent), Err(R::SigningIndex));
    let mut missing = control.clone();
    missing.spent_output = None;
    assert_eq!(session.checked_sponsor(&missing), Err(R::Assertion));
    let mut malformed = control.clone();
    malformed.finalized_transaction.clear();
    assert_eq!(session.checked_sponsor(&malformed), Err(R::SigningDecode));
    for (expected, mutate) in [
        (
            R::Asset,
            (|s: &mut WireSpentOutput| s.asset_field[1] ^= 1) as Mutate,
        ),
        (
            R::Value,
            (|s: &mut WireSpentOutput| s.value_field[8] ^= 1) as Mutate,
        ),
        (
            R::Program,
            (|s: &mut WireSpentOutput| s.program[2] ^= 1) as Mutate,
        ),
    ] {
        let mut subject = control.clone();
        mutate(subject.spent_output.as_mut().unwrap());
        assert_eq!(session.checked_sponsor(&subject), Err(expected));
    }
    authority_readback_controls(pending, &raw, &rb);
    // Neither a second session nor an outpoint offer grants registration authority.
    assert_eq!(
        Session::new().register(&raw, &rb, ObservedOutcomeLayer::Accepted),
        Err(R::NotSubmitted)
    );
}

fn authority_readback_controls(mut pending: Session, raw: &[u8], rb: &MinedFundingReadback) {
    use MockAuthorityRefusal as R;
    for (expected, subject) in [
        (
            R::Size,
            MinedFundingReadback {
                raw_transaction: vec![
                    0;
                    target_elements_conformance::sponsor_parent::MAXIMUM_SPONSOR_PARENT_BYTES
                        + 1
                ],
                ..rb.clone()
            },
        ),
        (
            R::Decode,
            MinedFundingReadback {
                raw_transaction: vec![],
                ..rb.clone()
            },
        ),
        (
            R::Identity,
            MinedFundingReadback {
                transaction_id: "different".into(),
                ..rb.clone()
            },
        ),
        (
            R::WitnessIdentity,
            MinedFundingReadback {
                witness_transaction_id: "different".into(),
                ..rb.clone()
            },
        ),
    ] {
        let mut twin = pending.clone();
        assert_eq!(
            twin.register(raw, &subject, ObservedOutcomeLayer::Accepted),
            Err(expected)
        );
        assert!(twin.coins.is_empty());
    }
    let mut changed = TargetTransaction::decode(raw).unwrap();
    // A second canonical transaction is a byte mismatch, not malformed input.
    changed = TargetTransaction::new(
        changed.version(),
        changed.inputs().to_vec(),
        changed.outputs().to_vec(),
        changed.lock_time() + 1,
        changed.witnesses().to_vec(),
    )
    .unwrap();
    let other = readback(&changed.encode(), 111);
    assert_eq!(
        pending.register(raw, &other, ObservedOutcomeLayer::Accepted),
        Err(R::Bytes)
    );
}

/// Funding uses the pinned elements decoder; construction owns its narrower
/// no-issuance/no-peg-in parent envelope. Neither success establishes inclusion.
///
/// # Panics
/// Panics only if a fixed fixture or its expected owner boundary changes; this
/// test helper takes no caller-supplied values.
pub fn decoder_boundary() {
    use transaction::keyed_request::KeyedRequestRefusal;
    let mut session = Session::new();
    let control = session.fund();
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let key = [42; 32];
    let commitment = commit_keyed_bundle(&target, &prior::linked(key, true)).unwrap();
    let request = |bytes: &[u8]| {
        keyed_source_request(
            &target,
            key,
            &commitment,
            &[KeyedInputOffer {
                input_index: 0,
                parent: bytes,
                outpoint: Outpoint::new(
                    Txid::from_internal(
                        elements::encode::deserialize::<elements::Transaction>(bytes)
                            .unwrap()
                            .txid()
                            .to_byte_array(),
                    ),
                    1,
                )
                .unwrap(),
                sequence: 1000,
                flags: 0,
                role: KeyedInputRole::Collateral,
                capability: None,
            }],
        )
        .map(|_| ())
    };
    assert_eq!(request(&control), Ok(()));
    let mut parent: elements::Transaction = elements::encode::deserialize(&control).unwrap();
    parent.input[0].is_pegin = true;
    let subject = elements::encode::serialize(&parent);
    let rb = readback(&subject, 101);
    let mut funded = session.exchanges[0].1.funded_outputs[0].clone();
    funded.outpoint.txid.clone_from(&rb.transaction_id);
    assert_eq!(
        target_elements_conformance::sponsor_parent::bind_sponsor_parent(&rb, &funded).unwrap(),
        subject
    );
    let error = request(&subject).unwrap_err();
    assert_eq!(
        error,
        KeyedRequestRefusal::Input {
            index: 0,
            refusal: KeyedInputRefusal::UnsupportedForm(UnsupportedParentForm::PegIn)
        }
    );
}

/// Reconstruct the same public subject; no private request fields are forged.
fn with_subject<T>(
    subject: &FinalSubject,
    inspect: impl FnOnce(
        KeyedConstructionRequest<'_, '_>,
        &KeyedStructuralTransaction<'_, '_>,
        &KeyedAdmittedTransaction,
        &SponsorSigningRequest,
    ) -> T,
) -> T {
    let target = target_elements::reviewed_elements_tapscript().unwrap();
    let frozen = subject.frozen();
    let key = *frozen.key();
    let commitment = commit_keyed_bundle(
        &target,
        &prior::linked(key, frozen.kind() == FinalKind::Source),
    )
    .unwrap();
    let decoded = TargetTransaction::decode(subject.bytes()).unwrap();
    let signer = frozen.inputs().iter().position(|r| !r.member()).unwrap();
    let ValueField::Explicit(change) = decoded.outputs()[1].value() else {
        panic!("explicit")
    };
    let cap = Replay {
        point: frozen.inputs()[signer].parent().outpoint(),
        change,
        program: decoded.outputs()[1].program().to_vec(),
        offers: Cell::new(0),
        signs: Cell::new(0),
        frozen: RefCell::new(Some(frozen.clone())),
        witness: RefCell::new(decoded.witnesses()[signer].stack().to_vec()),
        captured: RefCell::new(None),
    };
    let inputs: Vec<_> = frozen
        .inputs()
        .iter()
        .map(|r| KeyedInputOffer {
            input_index: r.index(),
            parent: r.parent().parent_bytes(),
            outpoint: r.parent().outpoint(),
            sequence: r.sequence(),
            flags: 0,
            role: if r.member() {
                KeyedInputRole::Member
            } else {
                KeyedInputRole::Collateral
            },
            capability: if r.member() { None } else { Some(&cap) },
        })
        .collect();
    let source;
    let merge;
    let request = if frozen.kind() == FinalKind::Source {
        source = keyed_source_request(&target, key, &commitment, &inputs).unwrap();
        KeyedConstructionRequest::Source(&source)
    } else {
        merge = keyed_merge_request(&target, key, &commitment, &inputs).unwrap();
        KeyedConstructionRequest::Merge(&merge)
    };
    let offer = KeyedOutputOffer {
        policy_asset: AssetId::from_internal([3; 32]),
        successor_value: 1000,
        additional_outputs: vec![decoded.outputs()[1].clone()],
        fee: Some(10_000),
    };
    let structure = construct_keyed(&target, request, offer.clone()).unwrap();
    let policy = KeyedPolicy::new(
        "mock explicit policy reading; not observed relay policy".into(),
        AssetId::from_internal([3; 32]),
        [100; 3],
        vec![100, 100, 1],
        [400_000; 3],
        0,
        72,
    )
    .unwrap();
    let admitted = admit_keyed(&target, request, offer, &policy).unwrap();
    assert_eq!(admitted.transaction().encode(), subject.bytes());
    let signing = cap.captured.borrow().clone().unwrap();
    inspect(request, &structure, &admitted, &signing)
}

/// Public mispairings swap one sealed argument; the compact role is the named exception.
pub fn pairing_controls() {
    use BridgeRefusal as R;
    let history = history([42; 32]);
    let other = history_for_second_key();
    let a = &history.finals[0];
    with_subject(a, |request_a, structure_a, admitted_a, signing_a| {
        let frozen = freeze_final_subject(request_a, structure_a, 0).unwrap();
        frozen.bind_signing_request(signing_a).unwrap();
        complete_final_subject(&frozen, request_a, admitted_a).unwrap();
        with_subject(&history.finals[1], |request, structure, _, _| {
            assert_eq!(
                freeze_final_subject(request, structure_a, 0).unwrap_err(),
                R::Census
            );
            freeze_final_subject(request, structure, 0).unwrap();
            assert_eq!(
                freeze_final_subject(request, structure, 1).unwrap_err(),
                R::SigningRole
            );
        });
        with_subject(&other.finals[0], |_, structure, _, _| {
            assert_eq!(
                freeze_final_subject(request_a, structure, 0).unwrap_err(),
                R::OutputZero
            );
        });
        with_subject(&history.finals[3], |_, _, _, signing| {
            assert_eq!(
                freeze_final_subject(request_a, structure_a, 2).unwrap_err(),
                R::SigningIndex
            );
            assert_eq!(frozen.bind_signing_request(signing), Err(R::SigningIndex));
        });
        with_subject(&history.finals[2], |request_b, _, admitted_b, signing_b| {
            assert_eq!(frozen.bind_signing_request(signing_b), Err(R::SigningBytes));
            assert_eq!(
                complete_final_subject(&frozen, request_a, admitted_b).unwrap_err(),
                R::SignedBytes
            );
            assert_eq!(
                complete_final_subject(&frozen, request_b, admitted_a).unwrap_err(),
                R::CapabilitySnapshot
            );
        });
        let compact = compact_signing_request();
        assert_eq!(compact.role(), SignerRole::SponsorSuffixMember);
        assert_eq!(
            frozen.bind_signing_request(&compact),
            Err(R::SigningProfile)
        );
    });
}
fn history_for_second_key() -> Session {
    history(vectors::keyed_final_manifest::SECOND_KEY)
}
fn compact_signing_request() -> SponsorSigningRequest {
    use vectors::materialize::{
        AshFunding, SponsorCoin, has_candidate_program, needs_authorization,
        sponsor_signing_requests, vector_id,
    };
    let case = vectors::fixture::positive_semantic_census()
        .unwrap()
        .into_iter()
        .find(|c| needs_authorization(c) && has_candidate_program(c))
        .unwrap();
    let bundle = vectors::bundle::fixture_bundle().unwrap();
    let funding = AshFunding::unexecutable_placeholder(vector_id(&case));
    let mut program = vec![0, 20];
    program.extend_from_slice(&[1; 20]);
    let coin = SponsorCoin::new(
        Outpoint::new(Txid::from_internal([0xc0; 32]), 0).unwrap(),
        100_000,
        program,
    );
    sponsor_signing_requests(&bundle, &case, &funding, &[coin]).unwrap()[0]
        .request()
        .clone()
}

pub(super) fn recorded_revision_control() {
    let subject = history([42; 32]).finals.remove(0);
    let step = subject.submission_step();
    let historical = NativeOperationRequest {
        schema: 8,
        case: step.case().clone(),
        subject: step.subject().clone(),
    };
    subject
        .validate_request(OperationRevision::Eight, &historical)
        .unwrap();
    assert_eq!(
        subject.validate_request(OperationRevision::Nine, &historical),
        Err(BridgeRefusal::SubmissionSchema)
    );
}
