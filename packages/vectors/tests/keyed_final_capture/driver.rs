//! Same public construction for live transport and recorded-response replay.
//! No prototype spend builder; the live worker retains request borrows across signing.
use crate::keyed_reproduction::fixture as prior;
use std::cell::{Cell, RefCell};
use target_elements_conformance::constructor::tagged::sha256;
use target_elements_conformance::executor::OperationStep;
use target_elements_conformance::protocol::*;
use transaction::keyed_construct::*;
use transaction::keyed_inputs::*;
use transaction::keyed_request::*;
use transaction::keyed_taproot::commit_keyed_bundle;
use transaction::*;
use vectors::keyed_final_capture::*;
use vectors::keyed_final_native::*;

/// Synchronous public construction's sole connection to a transport or replay.
pub trait Exchange {
    /// Return the response for this exact frozen operation.
    fn exchange(&mut self, step: OperationStep) -> NativeOperationResponse;
    /// Retain exact bytes and manifest row before the transport sees a submission.
    fn before_submission(&mut self, row: CaptureRow);
}

struct Session<I> {
    io: I,
    program: Vec<u8>,
    asset: AssetId,
    serial: u32,
    finals: Vec<FinalSubject>,
    target: CaptureTarget,
    foreign: Option<Vec<u8>>,
    policy: Option<vectors::maturity_native::sponsored::Policy>,
}
const fn output(asset: AssetId, value: u64, program: Vec<u8>) -> TargetOutput {
    TargetOutput::new(
        AssetField::Explicit(asset),
        ValueField::Explicit(value),
        NonceField::Null,
        program,
    )
}
impl<I: Exchange> Session<I> {
    fn fund(&mut self) -> (Vec<u8>, u32) {
        self.serial += 1;
        let step = OperationStep::new(
            &format!("fund-{}", self.serial),
            OperationSubject::SponsorFunding(Box::new(TargetSponsorFundingSubject {
                sponsor_outputs: 1,
                amount_per_sponsor_output: 1_000_000,
            })),
        );
        let answer = self.io.exchange(step);
        assert_eq!(answer.observed_layer, ObservedOutcomeLayer::Accepted);
        let [coin] = answer.funded_outputs.as_slice() else {
            panic!("one funded output")
        };
        let readback = answer.mined_readback.as_ref().expect("funding readback");
        let bytes =
            target_elements_conformance::sponsor_parent::bind_sponsor_parent(readback, coin)
                .expect("bound canonical funding parent");
        let parent = TargetTransaction::decode(bytes).expect("canonical parent");
        let out = &parent.outputs()[coin.outpoint.vout as usize];
        let AssetField::Explicit(asset) = out.asset() else {
            panic!("explicit policy asset")
        };
        if !self.program.is_empty() {
            assert_eq!(self.program, out.program());
            assert_eq!(self.asset, asset);
        }
        self.program = out.program().to_vec();
        self.asset = asset;
        (bytes.to_vec(), coin.outpoint.vout)
    }
    fn sign(&mut self, step: OperationStep) -> Vec<Vec<u8>> {
        let OperationSubject::SponsorSigning(subject) = step.subject() else {
            panic!("signing subject")
        };
        let frozen = subject.finalized_transaction.clone();
        let answer = self.io.exchange(step);
        assert_eq!(answer.observed_layer, ObservedOutcomeLayer::Accepted);
        assert_eq!(
            answer.signature_bound_to.as_deref(),
            Some(frozen.as_slice())
        );
        answer.sponsor_witness
    }
    #[allow(
        clippy::too_many_lines,
        reason = "The request and capability borrows stay alive across the transport signing phase."
    )]
    fn event(
        &mut self,
        key: [u8; 32],
        parents: &[(Vec<u8>, u32, bool)],
        schema: CaptureSchema,
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
        let fee = if matches!(schema, CaptureSchema::ScalingMerge(_)) {
            20_000
        } else {
            10_000
        };
        let successor = 1_000;
        let change = total - fee - successor;
        let point = Outpoint::new(
            Txid::from_internal(prior::txid(&parents[signer].0)),
            parents[signer].1,
        )
        .unwrap();
        let cap = Replay {
            point,
            fee,
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
            policy_asset: self.asset,
            successor_value: successor,
            additional_outputs: vec![output(self.asset, change, self.program.clone())],
            fee: Some(fee),
        };
        if schema == CaptureSchema::ScalingMerge(1565) {
            assert!(matches!(
                admit_keyed(
                    &target,
                    request,
                    offer.clone(),
                    &KeyedPolicy::new(
                        "preflight before structural serialization".into(),
                        self.asset,
                        [100; 3],
                        vec![100, 100, 1],
                        [400_000; 3],
                        0,
                        72
                    )
                    .unwrap()
                ),
                Err(KeyedConstructionError::HandleCountExceedsPolicy {
                    offered: 1565,
                    maximum: 1564
                })
            ));
            assert_eq!(cap.signs.get(), 0);
            println!(
                "scaling preflight members=1565 maximum=1564 refused-before-serialization=true"
            );
        }
        let structure = construct_keyed(&target, request, offer.clone()).unwrap();
        let frozen = freeze_final_subject(request, &structure, signer).unwrap();
        assert_eq!(
            freeze_final_subject(request, &structure, usize::from(u16::MAX) + 1).unwrap_err(),
            BridgeRefusal::SigningIndex
        );
        cap.frozen.replace(Some(frozen.clone()));
        cap.witness.replace(self.sign(frozen.signing_step()));
        let policy = KeyedPolicy::new(
            "declared capture construction readings; native policy remains separately observed"
                .into(),
            self.asset,
            [100; 3],
            vec![100, 100, 1],
            [400_000; 3],
            0,
            72,
        )
        .unwrap();
        if schema == CaptureSchema::ScalingMerge(1565) {
            let witness = cap.witness.borrow().clone();
            transaction::keyed_sponsor::verify_keyed_sponsor(
                frozen.frozen_bytes(),
                signer,
                frozen.inputs()[signer].parent().output(),
                &witness,
            )
            .unwrap();
            let tx = structure.transaction();
            let mut witnesses = tx.witnesses().to_vec();
            witnesses[signer] = InputWitness::new(witness);
            let tx = TargetTransaction::new(
                tx.version(),
                tx.inputs().to_vec(),
                tx.outputs().to_vec(),
                tx.lock_time(),
                witnesses,
            )
            .unwrap();
            return self.raw_submission(key, schema, tx.encode());
        }
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
        self.submit(final_subject, script, schema)
    }

    fn submit(&mut self, subject: FinalSubject, _script: &[u8], schema: CaptureSchema) -> Vec<u8> {
        if let CaptureTarget::Funding(point) = self.target
            && (schema == CaptureSchema::FundingControl
                || matches!(schema, CaptureSchema::FundingPoint(_)))
        {
            return self.funding_submission(
                &subject,
                if schema == CaptureSchema::FundingControl {
                    0
                } else {
                    point
                },
            );
        }
        self.mutants(&subject, schema);
        if schema == CaptureSchema::LaterParent {
            return self.later_parent(&subject);
        }
        if schema == CaptureSchema::KeyPathControl {
            return self.key_path_pair(&subject);
        }
        let bytes = subject.bytes().to_vec();
        let row = CaptureRow {
            schema,
            key: *subject.frozen().key(),
            freeze: capture_freeze_digest(*subject.frozen().key()).expect("freeze"),
            digest: subject.digest().0,
            submitted: bytes.clone(),
            boundary: schema.boundary(),
            expected: schema.outcome(),
            sealed_control: true,
        };
        self.io.before_submission(row);
        let answer = self.io.exchange(subject.submission_step());
        assert_eq!(answer.observed_layer, ObservedOutcomeLayer::Accepted);
        let readback = answer
            .mined_readback
            .as_ref()
            .expect("mined accepted subject");
        assert_eq!(readback.raw_transaction, bytes);
        assert_eq!(
            readback.transaction_id,
            Txid::from_internal(prior::txid(&bytes)).to_target_display()
        );
        // Target acceptance stays recorded. Failed registration stops this pilot,
        // because the next event must spend its owned change successfully.
        if let Some(detail) = &answer.observed_detail
            && let Ok(value) = serde_json::from_str::<serde_json::Value>(detail)
        {
            assert!(
                value.get("sponsor_registration_failure").is_none()
                    && value
                        .get("detail")
                        .and_then(|facts| facts.get("sponsor_registration_failure"))
                        .is_none(),
                "accepted, but registration failed: {detail}"
            );
        }
        self.finals.push(subject);
        bytes
    }
}

/// The four sealed controls, also compared with the eight historical mock pins.
/// # Panics
/// Panics if construction or a transport response violates the reviewed subject.
pub fn controls<I: Exchange>(io: I, key: [u8; 32]) -> I {
    campaign(io, key, CaptureTarget::Cost)
}

/// Run the reviewed schema for one supplied key, retaining ordinary-edge order.
/// # Panics
/// Panics if construction or a transport response violates the reviewed subject.
pub fn campaign<I: Exchange>(io: I, key: [u8; 32], target: CaptureTarget) -> I {
    if let CaptureTarget::Funding(point) = target {
        let mut s = Session {
            io,
            program: vec![],
            asset: AssetId::from_internal([0; 32]),
            serial: 0,
            finals: vec![],
            target,
            foreign: None,
            policy: None,
        };
        let mut parents = vec![];
        if point >= 12 {
            let (bytes, vout) = s.fund();
            let parent = s.event(key, &[(bytes, vout, false)], CaptureSchema::SourceA);
            parents.push((parent, 0, true));
        }
        let (bytes, vout) = s.fund();
        parents.push((bytes, vout, false));
        s.event(key, &parents, CaptureSchema::FundingControl);
        if point % 12 != 0 {
            let mut parents = vec![];
            if point >= 12 {
                let (bytes, vout) = s.fund();
                let parent = s.event(key, &[(bytes, vout, false)], CaptureSchema::SourceB);
                parents.push((parent, 0, true));
            }
            let (bytes, vout) = s.fund();
            parents.push((bytes, vout, false));
            s.event(key, &parents, CaptureSchema::FundingPoint(point));
        }
        return s.io;
    }
    if let CaptureTarget::Scaling(count) = target {
        return envelope::campaign(io, key, count);
    }
    let mut s = Session {
        io,
        program: vec![],
        asset: AssetId::from_internal([0; 32]),
        serial: 0,
        finals: vec![],
        target,
        foreign: None,
        policy: None,
    };
    let (funding, index) = s.fund();
    let a = s.event(key, &[(funding, index, false)], CaptureSchema::SourceA);
    // This is the pilot's first purpose: accepted framework decode, registration,
    // and a signing request for that mined change, before any campaign mutant.
    let single = s.event(
        key,
        &[(a.clone(), 1, false), (a, 0, true)],
        CaptureSchema::SingleOrdinaryEdge,
    );
    let (funding, index) = s.fund();
    let b = s.event(key, &[(funding, index, false)], CaptureSchema::SourceB);
    if target == CaptureTarget::Continuation {
        let other = if key == [42; 32] {
            vectors::keyed_final_manifest::SECOND_KEY
        } else {
            [42; 32]
        };
        let (funding, index) = s.fund();
        s.foreign = Some(s.event(
            other,
            &[(funding, index, false)],
            CaptureSchema::ForeignSource,
        ));
    }
    let multi = s.event(
        key,
        &[(single.clone(), 0, true), (single, 1, false), (b, 0, true)],
        CaptureSchema::MultiNonzeroMembers,
    );
    if target == CaptureTarget::Commitment {
        s.event(
            key,
            &[(multi.clone(), 0, true), (multi, 1, false)],
            CaptureSchema::KeyPathControl,
        );
    }
    if target == CaptureTarget::Birth {
        let (funding, index) = s.fund();
        let parent = s.event(key, &[(funding, index, false)], CaptureSchema::LaterParent);
        s.event(
            key,
            &[(parent.clone(), 0, true), (parent, 2, false)],
            CaptureSchema::LaterControl,
        );
    }
    s.io
}
#[path = "mutations.rs"]
mod mutations;
struct Replay {
    point: Outpoint,
    fee: u64,
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
            self.fee,
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

#[path = "envelope.rs"]
pub(super) mod envelope;
