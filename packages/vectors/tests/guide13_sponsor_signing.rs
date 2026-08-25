//! The sponsor envelope signer, wired into the live-transfer lane once.
//!
//! # What this is and what it is not
//!
//! The Phase-5 handoff card's sponsor-envelope-signing row names one
//! integration test as the cheapest honest first step: finalize an
//! explicit sponsored control, send its exact sponsor request, replay the
//! returned witness through the sponsor capability, and verify byte
//! binding. That is all this file does, and each of the four is a
//! separate observable rather than a claim the others imply.
//!
//! The lane now takes one further step, and it is the step the carried
//! residual's own clearing rule names. That rule, at
//! `vectors::live_evidence::LiveInfrastructureBlocker`, asks for the
//! sponsor owner's target authorization and says a returned byte stack
//! is not that until a target has ACCEPTED a control carrying it. So the
//! replayed control is submitted, and what the target answers is
//! recorded at the layer the target typed it at.
//!
//! A blocker still moves on an observed result and never on a capability
//! existing — "the capability is reachable from a lane" remains the
//! capability existing. What can move it is the acceptance, and only the
//! acceptance.
//!
//! # What one acceptance is not
//!
//! It is the sponsor envelope's WIRE, established end to end, and one
//! target acceptance of a control carrying a sponsor witness. It is NOT
//! production multi-party sponsor signing. One fixed regtest key signs
//! here — deterministic, single-party, and published — and a single key
//! answering a request is not a ceremony. Every value this lane touches
//! is public disposable test material under ADR-015.
//!
//! # Why it needs a node
//!
//! Both halves of the round trip come from the real adapter. The sponsor
//! coin is one the adapter created and remembers, and the signature is
//! one the adapter produced over the exact finalized bytes it was handed.
//! A mock could return bytes, and bytes a mock chose would prove nothing
//! about the capability this row is about.
//!
//! Run it as:
//!
//! ```text
//! TRIPOD_SPONSOR_EXECUTOR=<adapter> \
//! TRIPOD_SPONSOR_NETWORK_ID=<64 hex> \
//! TRIPOD_SPONSOR_GENESIS_ID=<64 hex> \
//!   cargo test -p tripod-vectors --test guide13_sponsor_signing -- --ignored --nocapture
//! ```
//!
//! # The reserve asset is learned rather than assumed
//!
//! This lane once could not submit anything, and the reason was exactly
//! one thing: the deployment's reserve asset was a fixture constant no
//! chain had issued, while the sponsor coin the adapter funds is one of
//! its own. A control built that way names a fee output in an asset its
//! sponsor input does not carry.
//!
//! The reserve is now a value the run LEARNS. The sponsor-funding
//! request deliberately names no asset — what a development network
//! uses as its reserve is the network's own fact — and the executor
//! reports which asset it funded in. That answer is read, checked for a
//! single identity across every coin, and welded into the deployment
//! before anything is linked against it.
//!
//! Which is why the sponsor region is funded BEFORE the receipts. The
//! reserve is pushed as a literal by §10.7's isolation fragments, so it
//! is committed in the taptree and the destination programs move with
//! it. Receipts funded first would be paid to the programs of a
//! deployment this lane is about to stop using.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use linker::live_backend::LiveTransferRepresentationPlan;
use target_elements::{
    ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding, LeafVersion,
    reviewed_elements_tapscript, validate_reviewed_development_binding,
};
use target_elements_conformance::executor::{
    DEFAULT_EXECUTOR_TIMEOUT, ExecutorConfiguration, ExecutorDiagnostics, ExecutorTrust,
    OperationStep, PlanRefused, TargetOperationPlanner, execute_operations,
};
use target_elements_conformance::protocol::{
    NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId, OperationSubject,
    TargetFundingSubject, TargetSponsorFundingSubject, TargetSponsorSigningSubject,
    TargetSubmissionSubject, WireOutpoint, WireSighashProfile,
};
use transaction::bytes::{AssetField, AssetId, Outpoint, Txid, ValueField};
use transaction::live_abi::CandidateLiveTransferAbi;
use transaction::live_construct::{complete_live_transfer, finalize_live_transfer};
use transaction::live_request::{
    LiveReceiptDestination, LiveTransferRequest, ProtocolValue, RequestedForm, SponsorChangeRequest,
};
use transaction::live_signing::{LiveOwnerResponse, authorize_live_transfer};
use transaction::sponsor::{
    SponsorCapability, SponsorOffer, SponsorSignature, SponsorSigningRequest,
};
use transaction::view::{PublicConstructionView, PublicOutputView};
use vectors::live_evidence::UNAUTHORIZING_SIGNATURE;
use vectors::live_plan::{
    FIRST_SCALAR, SECOND_SCALAR, demonstration_live_abi, live_abi_for_asset, published_owner,
    reviewed_target,
};

/// What each funded receipt is asked to hold.
const RECEIPT_AMOUNT: u64 = 5_000;

/// What the sponsor coin is asked to hold.
///
/// Exact rather than a minimum, and equal to the fee the offer states,
/// because the sponsor asks for no change: a coin holding more than the
/// offer spends leaves the reserve asset unbalanced, and the caller is
/// the side that knows what it is about to build.
const SPONSOR_FEE: u64 = 250;

/// How the staged envelope answers a signing request.
enum Answers {
    /// Record what was asked and answer with a placeholder, so that the
    /// request the adapter must be handed can be collected at all.
    ///
    /// The transaction this arm completes is discarded. A
    /// placeholder-authorized control is not evidence and must never be
    /// able to become any.
    Recording(RefCell<Vec<(u16, Vec<u8>)>>),
    /// Answer with what the adapter returned, keyed by input position.
    Replaying(BTreeMap<u16, SponsorSignature>),
}

/// The sponsor envelope this lane stages, in both of its passes.
///
/// One type rather than two, because the offer and the change
/// destination must be identical across the passes: a second spelling of
/// the offer could disagree with the first about what was finalized, and
/// then the bytes the adapter signed would not be the bytes replayed
/// into.
struct StagedEnvelope {
    offer: SponsorOffer,
    answers: Answers,
}

impl SponsorCapability for StagedEnvelope {
    fn offer(&self) -> SponsorOffer {
        self.offer.clone()
    }

    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        None
    }

    fn sign(&self, request: &SponsorSigningRequest) -> Option<SponsorSignature> {
        match &self.answers {
            Answers::Recording(recorded) => {
                recorded
                    .borrow_mut()
                    .push((request.input(), request.transaction().to_vec()));
                Some(SponsorSignature::new(
                    request.transaction().to_vec(),
                    vec![Vec::new(), Vec::new()],
                ))
            }
            Answers::Replaying(answers) => answers.get(&request.input()).cloned(),
        }
    }
}

/// Everything one round trip through the adapter's signer produced.
///
/// Kept as observations rather than as a verdict: the test asserts on
/// these, and the planner that fills them decides nothing.
#[derive(Clone, Debug, Default)]
struct RoundTrip {
    /// The sponsor input's position in the finalized control.
    input: u16,
    /// The exact bytes the sponsor request carried.
    sent: Vec<u8>,
    /// The bytes the adapter echoed back as what it signed.
    echoed: Vec<u8>,
    /// The witness stack the adapter returned.
    witness: Vec<Vec<u8>>,
    /// The control completed by replaying that witness.
    replayed: Vec<u8>,
    /// The control completed in the recording pass, for comparison.
    placeholder: Vec<u8>,
    /// What a signature bound to one mutated byte was refused with.
    mutated_refusal: Option<String>,
}

/// What the target answered when handed the sponsor-signed control.
///
/// Kept as observations for the same reason [`RoundTrip`] is: the
/// planner records what happened and judges none of it.
#[derive(Clone, Debug)]
struct Submission {
    /// The exact bytes handed to the target.
    sent: Vec<u8>,
    /// The layer the target's answer was typed at.
    layer: ObservedOutcomeLayer,
    /// The target's own words, when it refused.
    detail: Option<String>,
    /// The identity the target reported on acceptance.
    txid: Option<String>,
    /// The target's own copy of the mined transaction.
    read_back: Option<Vec<u8>>,
    /// The block the target mined it into.
    block: Option<(String, u32)>,
}

/// The finalization a sponsor request was formed against.
///
/// Kept whole rather than rebuilt, because a rebuilt finalization is a
/// different one and the signature would then be bound to other bytes.
type StagedFinalization = (
    transaction::live_signing::AuthorizedLiveTransfer,
    transaction::live_construct::LiveConstructionReport,
);

/// A staged control: its finalization, every sponsor request the builder
/// issued against it, and the placeholder completion to compare against.
type StagedControl = (StagedFinalization, Vec<(u16, Vec<u8>)>, Vec<u8>);

/// Which step the lane is on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Issue,
    FundSponsor,
    FundReceipts,
    SignSponsor,
    Submit,
    Done,
}

/// Why the lane stopped before the round trip completed.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Refusal {
    SubstrateUnavailable,
    IssuanceNamedNoAsset,
    RelinkRefused,
    FundingCreatedNoPredecessor,
    MalformedFundedOutpoint,
    ControlNotConstructible,
    /// The adapter refused, or answered at a layer below acceptance.
    AuthorizationDidNotHappen(ObservedOutcomeLayer),
    /// The adapter accepted and returned no witness.
    AuthorizationCarriedNoWitness,
    /// The adapter echoed bytes that are not the bytes it was sent.
    AuthorizationBoundToOtherBytes {
        sent: usize,
        echoed: usize,
    },
    /// The completed control did not ask the envelope for exactly one
    /// sponsor authorization.
    UnexpectedSponsorRequestCount(usize),
    /// A signature bound to mutated bytes was accepted, which would mean
    /// the binding check this test is about does not hold.
    MutatedBindingAccepted,
    /// The sponsor coins came back in more than one asset, so there is
    /// no single reserve identity to weld the deployment to.
    SponsorAssetsDisagree,
    /// The sponsor-funding step reported no reserve asset at all.
    SponsorFundingNamedNoReserve,
    /// The target refused the sponsor-signed control, or answered at a
    /// layer below acceptance.
    SubmissionDidNotHappen(ObservedOutcomeLayer),
    /// The target accepted and named no transaction identity.
    SubmissionCarriedNoIdentity,
    /// The target's own copy of the mined transaction is not the bytes
    /// that were submitted.
    ReadbackDisagreesWithSubmittedBytes {
        submitted: usize,
        read_back: usize,
    },
}

/// The lane: issue, fund receipts, fund a sponsor coin, sign.
struct SponsorSigningPlanner {
    stage: Stage,
    abi: CandidateLiveTransferAbi,
    issued_asset: Option<String>,
    /// The reserve identity the chain reported, welded into the
    /// deployment before anything is linked against it.
    reserve: Option<AssetId>,
    explicit_program: Vec<u8>,
    receipts: Vec<ObservedCoin>,
    sponsor: Option<ObservedCoin>,
    submission: Option<Submission>,
    /// The finalization the sponsor request was formed against, kept so
    /// the replay pass completes the same one rather than a rebuild.
    staged: Option<StagedFinalization>,
    round: Option<RoundTrip>,
    refusal: Option<Refusal>,
}

/// One coin as the node reported it.
///
/// The three message-bearing fields are the node's own and never this
/// lane's expectations, for the reason the live-transfer lane records:
/// a control built over a spent-output triple nobody asked the chain
/// about is a control about nothing.
#[derive(Clone, Debug)]
struct ObservedCoin {
    outpoint: Outpoint,
    asset: AssetId,
    amount: u64,
    program: Vec<u8>,
}

impl SponsorSigningPlanner {
    fn new() -> Result<Self, Refusal> {
        let abi = demonstration_live_abi().map_err(|_| Refusal::SubstrateUnavailable)?;
        let explicit_program = destination_program(&abi)?;
        Ok(Self {
            stage: Stage::Issue,
            abi,
            issued_asset: None,
            reserve: None,
            explicit_program,
            receipts: Vec::new(),
            sponsor: None,
            submission: None,
            staged: None,
            round: None,
            refusal: None,
        })
    }

    const fn refuse(&mut self, refusal: Refusal) -> PlanRefused {
        self.refusal = Some(refusal);
        self.stage = Stage::Done;
        PlanRefused
    }

    /// Remember the asset the chain just issued.
    ///
    /// The deployment is NOT relinked here, because half of what it is
    /// welded to is still unknown: the reserve arrives from the
    /// sponsor-funding answer, and a link taken now would be a link
    /// against the fixture reserve.
    fn remember_issued_asset(&mut self, response: &NativeOperationResponse) -> Result<(), Refusal> {
        let asset = response
            .issued_asset
            .clone()
            .ok_or(Refusal::IssuanceNamedNoAsset)?;
        asset_of(&asset).ok_or(Refusal::IssuanceNamedNoAsset)?;
        self.issued_asset = Some(asset);
        Ok(())
    }

    /// Weld the deployment to BOTH assets the chain reported.
    ///
    /// The link the whole lane waited for. Until it happens the
    /// deployment names a reserve no chain has issued, and the fee
    /// output of any control built from it is payable in an asset its
    /// sponsor input does not carry.
    fn relink(&mut self) -> Result<(), Refusal> {
        let printed = self
            .issued_asset
            .clone()
            .ok_or(Refusal::IssuanceNamedNoAsset)?;
        let protocol = asset_of(&printed).ok_or(Refusal::IssuanceNamedNoAsset)?;
        let reserve = self.reserve.ok_or(Refusal::SponsorFundingNamedNoReserve)?;
        let abi = live_abi_for_asset(*protocol.internal(), *reserve.internal())
            .map_err(|_| Refusal::RelinkRefused)?;
        self.explicit_program = destination_program(&abi)?;
        self.abi = abi;
        Ok(())
    }

    /// Read the sponsor coins and settle the ONE reserve identity.
    ///
    /// Mirrors the compact-ASH lane's own settlement, single-identity
    /// check included: one reserve is what gets linked into the leaves,
    /// so coins that disagree about it leave nothing to link. The
    /// executor funds in the chain's policy asset and reports which one
    /// it used, which is the only honest source for a value the request
    /// deliberately does not name.
    fn settle_sponsor_funding(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), Refusal> {
        let coins = Self::settle_funding(response)?;
        let mut reserve: Option<AssetId> = None;
        for coin in &coins {
            match reserve {
                Some(known) if known != coin.asset => return Err(Refusal::SponsorAssetsDisagree),
                Some(_) => {}
                None => reserve = Some(coin.asset),
            }
        }
        let reserve = reserve.ok_or(Refusal::FundingCreatedNoPredecessor)?;
        let coin = coins
            .into_iter()
            .next()
            .ok_or(Refusal::FundingCreatedNoPredecessor)?;
        self.reserve = Some(reserve);
        self.sponsor = Some(coin);
        self.relink()
    }

    /// One funding step's coins, taken from the node's report of them.
    fn settle_funding(response: &NativeOperationResponse) -> Result<Vec<ObservedCoin>, Refusal> {
        if response.funded_outputs.is_empty() {
            return Err(Refusal::FundingCreatedNoPredecessor);
        }
        let mut coins = Vec::with_capacity(response.funded_outputs.len());
        for funded in &response.funded_outputs {
            coins.push(ObservedCoin {
                outpoint: outpoint_of(&funded.outpoint).ok_or(Refusal::MalformedFundedOutpoint)?,
                asset: asset_of(&funded.asset).ok_or(Refusal::MalformedFundedOutpoint)?,
                amount: funded.amount_satoshis,
                program: decode_hex(&funded.script).ok_or(Refusal::MalformedFundedOutpoint)?,
            });
        }
        Ok(coins)
    }

    /// The funding step for the explicit constructor's program.
    fn funding_step(&self, name: &str, issue: bool) -> OperationStep {
        OperationStep::new(
            name,
            OperationSubject::Funding(Box::new(TargetFundingSubject {
                issue_asset: issue,
                asset: if issue {
                    None
                } else {
                    self.issued_asset.clone()
                },
                output_program: self.explicit_program.clone(),
                outputs: 2,
                amount_per_output: RECEIPT_AMOUNT,
            })),
        )
    }

    /// The explicit sponsored control, finalized and owner-authorized,
    /// with every sponsor request the builder issues collected.
    ///
    /// Returns the recorded requests and the placeholder-completed bytes
    /// alongside the authorized value, so that the second pass replays
    /// into the same finalization rather than into a rebuilt one.
    fn stage_control(&self) -> Result<StagedControl, Refusal> {
        let sponsor_coin = self
            .sponsor
            .as_ref()
            .ok_or(Refusal::ControlNotConstructible)?;
        let mut points = Vec::with_capacity(self.receipts.len());
        let mut views = Vec::with_capacity(self.receipts.len());
        let mut total = 0_u64;
        for coin in &self.receipts {
            points.push(coin.outpoint);
            views.push(PublicOutputView::new(
                coin.outpoint,
                AssetField::Explicit(coin.asset),
                ValueField::Explicit(coin.amount),
                coin.program.clone(),
            ));
            total = total
                .checked_add(coin.amount)
                .ok_or(Refusal::ControlNotConstructible)?;
        }
        // The sponsor coin is SHOWN and not merely named. Construction
        // refuses a sponsor input it cannot see, and refuses one whose
        // asset is not the deployment's reserve — which is exactly the
        // mismatch this lane exists to have removed, so letting the
        // builder check it is the point rather than a formality.
        views.push(PublicOutputView::new(
            sponsor_coin.outpoint,
            AssetField::Explicit(sponsor_coin.asset),
            ValueField::Explicit(sponsor_coin.amount),
            sponsor_coin.program.clone(),
        ));
        let view =
            PublicConstructionView::new(views).map_err(|_| Refusal::ControlNotConstructible)?;

        let request = LiveTransferRequest::new(
            points,
            [
                destination(&SECOND_SCALAR, total / 2)?,
                destination(&FIRST_SCALAR, total - total / 2)?,
            ],
            LiveTransferRepresentationPlan::Explicit,
            RequestedForm::Sponsored,
            SponsorChangeRequest::NotRequested,
            None,
        )
        .map_err(|_| Refusal::ControlNotConstructible)?;

        let envelope = StagedEnvelope {
            offer: SponsorOffer::new([sponsor_coin.outpoint], SPONSOR_FEE, None)
                .map_err(|_| Refusal::ControlNotConstructible)?,
            answers: Answers::Recording(RefCell::new(Vec::new())),
        };

        let target = reviewed_target().map_err(|_| Refusal::SubstrateUnavailable)?;
        let finalization =
            finalize_live_transfer(&target, &self.abi, &request, &view, Some(&envelope), None)
                .map_err(|_| Refusal::ControlNotConstructible)?;
        let report = finalization.report().clone();
        let finalized = finalization.into_finalized();

        // The owners authorize with bytes that authorize nothing. The
        // sponsor round trip is about the sponsor's signature over the
        // finalized bytes, and those bytes are settled before any owner
        // response is read; an owner signature would change what is
        // signed over here not at all.
        let responses: Vec<_> = finalized
            .signing_requests()
            .iter()
            .map(|signing| {
                (
                    signing.input(),
                    LiveOwnerResponse::to(signing, UNAUTHORIZING_SIGNATURE.to_vec()),
                )
            })
            .collect();
        let authorized = authorize_live_transfer(finalized, responses)
            .map_err(|_| Refusal::ControlNotConstructible)?;

        let placeholder =
            complete_live_transfer(&target, authorized.clone(), report.clone(), Some(&envelope))
                .map_err(|_| Refusal::ControlNotConstructible)?
                .bytes();

        let Answers::Recording(recorded) = &envelope.answers else {
            // Unreachable: the value was just built with this arm.
            return Err(Refusal::ControlNotConstructible);
        };
        let recorded = recorded.borrow().clone();
        Ok(((authorized, report), recorded, placeholder))
    }

    /// The sponsor signing step, carrying the exact finalized bytes.
    fn sign_step(&mut self) -> Result<OperationStep, Refusal> {
        let (staged, recorded, placeholder) = self.stage_control()?;
        if recorded.len() != 1 {
            return Err(Refusal::UnexpectedSponsorRequestCount(recorded.len()));
        }
        let (input, sent) = recorded[0].clone();
        let sponsor_coin = self
            .sponsor
            .clone()
            .ok_or(Refusal::ControlNotConstructible)?;

        self.round = Some(RoundTrip {
            input,
            sent: sent.clone(),
            placeholder,
            ..RoundTrip::default()
        });
        self.staged = Some(staged);

        Ok(OperationStep::new(
            "authorize-sponsor-input",
            OperationSubject::SponsorSigning(Box::new(TargetSponsorSigningSubject {
                finalized_transaction: sent,
                sponsor_input_index: input,
                sponsor_outpoint: WireOutpoint {
                    txid: txid_to_wire(&sponsor_coin.outpoint.txid()),
                    vout: sponsor_coin.outpoint.index(),
                },
                sighash_profile: WireSighashProfile::AllInputsAllOutputs,
            })),
        ))
    }
}

impl SponsorSigningPlanner {
    /// Hand the target the sponsor-signed control, exactly as replayed.
    ///
    /// The bytes are the replay's own — the completion carrying the
    /// adapter's witness — and never a rebuild. A rebuilt control is a
    /// different control, and submitting one would report a verdict
    /// about bytes no sponsor ever signed.
    fn submit_step(&mut self) -> Result<OperationStep, Refusal> {
        let round = self.round.clone().ok_or(Refusal::ControlNotConstructible)?;
        if round.replayed.is_empty() {
            return Err(Refusal::ControlNotConstructible);
        }
        Ok(OperationStep::new(
            "submit-sponsor-signed-control",
            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: round.replayed,
            })),
        ))
    }

    /// What the target did with it, recorded before it is judged.
    ///
    /// A refusal is recorded at the layer the target typed it at and in
    /// the target's own words, because which layer refused is the whole
    /// answer: the submission path asks the mempool first, so a relay
    /// refusal and a consensus refusal are different findings and only
    /// one of them is about standardness.
    fn settle_submission(&mut self, response: &NativeOperationResponse) -> Result<(), Refusal> {
        let round = self.round.clone().ok_or(Refusal::ControlNotConstructible)?;
        let read_back = response
            .mined_readback
            .as_ref()
            .map(|readback| readback.raw_transaction.clone());
        self.submission = Some(Submission {
            sent: round.replayed.clone(),
            layer: response.observed_layer,
            detail: response.observed_detail.clone(),
            txid: response.accepted_txid.clone(),
            read_back: read_back.clone(),
            block: response
                .mined_readback
                .as_ref()
                .map(|readback| (readback.block_hash.clone(), readback.block_height)),
        });

        if response.observed_layer != ObservedOutcomeLayer::Accepted {
            return Err(Refusal::SubmissionDidNotHappen(response.observed_layer));
        }
        if response.accepted_txid.is_none() {
            return Err(Refusal::SubmissionCarriedNoIdentity);
        }

        // The readback is the acceptance checked rather than believed:
        // the target's own copy of what it mined, against the bytes
        // that were handed to it.
        let read_back = read_back.ok_or(Refusal::SubmissionCarriedNoIdentity)?;
        if read_back != round.replayed {
            return Err(Refusal::ReadbackDisagreesWithSubmittedBytes {
                submitted: round.replayed.len(),
                read_back: read_back.len(),
            });
        }
        Ok(())
    }

    /// What the adapter answered: checked for binding, then replayed.
    fn settle_signature(&mut self, response: &NativeOperationResponse) -> Result<(), Refusal> {
        if response.observed_layer != ObservedOutcomeLayer::Accepted {
            return Err(Refusal::AuthorizationDidNotHappen(response.observed_layer));
        }
        let echoed = response
            .signature_bound_to
            .clone()
            .ok_or(Refusal::AuthorizationCarriedNoWitness)?;
        if response.sponsor_witness.is_empty() {
            return Err(Refusal::AuthorizationCarriedNoWitness);
        }
        let mut round = self.round.clone().ok_or(Refusal::ControlNotConstructible)?;
        if echoed != round.sent {
            return Err(Refusal::AuthorizationBoundToOtherBytes {
                sent: round.sent.len(),
                echoed: echoed.len(),
            });
        }
        round.echoed.clone_from(&echoed);
        round.witness.clone_from(&response.sponsor_witness);

        let (authorized, report) = self
            .staged
            .clone()
            .ok_or(Refusal::ControlNotConstructible)?;
        let sponsor_coin = self
            .sponsor
            .clone()
            .ok_or(Refusal::ControlNotConstructible)?;
        let target = reviewed_target().map_err(|_| Refusal::SubstrateUnavailable)?;
        let offer = SponsorOffer::new([sponsor_coin.outpoint], SPONSOR_FEE, None)
            .map_err(|_| Refusal::ControlNotConstructible)?;

        // The replay: the adapter's own witness, bound to the adapter's
        // own echo, back through the same capability the construction
        // asks — and into the same finalization the request was formed
        // against, never a rebuilt one.
        let replaying = StagedEnvelope {
            offer: offer.clone(),
            answers: Answers::Replaying(BTreeMap::from([(
                round.input,
                SponsorSignature::new(echoed.clone(), response.sponsor_witness.clone()),
            )])),
        };
        round.replayed = complete_live_transfer(
            &target,
            authorized.clone(),
            report.clone(),
            Some(&replaying),
        )
        .map_err(|_| Refusal::ControlNotConstructible)?
        .bytes();

        // The control for the binding claim: one byte of the echo moved
        // and the same witness. Completion must refuse, or "bound to
        // these exact bytes" is a sentence nothing enforces.
        let mut mutated = echoed;
        if let Some(first) = mutated.first_mut() {
            *first ^= 0x01;
        }
        let refusing = StagedEnvelope {
            offer,
            answers: Answers::Replaying(BTreeMap::from([(
                round.input,
                SponsorSignature::new(mutated, response.sponsor_witness.clone()),
            )])),
        };
        match complete_live_transfer(&target, authorized, report, Some(&refusing)) {
            Ok(_) => return Err(Refusal::MutatedBindingAccepted),
            Err(refusal) => round.mutated_refusal = Some(format!("{refusal:?}")),
        }

        self.round = Some(round);
        Ok(())
    }
}

impl TargetOperationPlanner for SponsorSigningPlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        if let Some((_case, response)) = previous {
            match self.stage {
                Stage::Issue => {
                    if let Err(refusal) = self.remember_issued_asset(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::FundSponsor;
                }
                // The sponsor region is funded BEFORE the receipts, and
                // the order is the whole repair. The reserve arrives in
                // this answer; the deployment is welded to it; and only
                // then are the receipts funded — to destination programs
                // that moved when the reserve did. Funding them first
                // would pay them to the programs of a deployment this
                // lane is about to stop using.
                Stage::FundSponsor => {
                    if let Err(refusal) = self.settle_sponsor_funding(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::FundReceipts;
                }
                Stage::FundReceipts => match Self::settle_funding(response) {
                    Ok(coins) => {
                        self.receipts = coins;
                        self.stage = Stage::SignSponsor;
                    }
                    Err(refusal) => return Err(self.refuse(refusal)),
                },
                Stage::SignSponsor => {
                    if let Err(refusal) = self.settle_signature(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::Submit;
                }
                Stage::Submit => {
                    if let Err(refusal) = self.settle_submission(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::Done;
                }
                Stage::Done => {}
            }
        }

        let step = match self.stage {
            Stage::Issue => self.funding_step("issue-protocol-asset", true),
            Stage::FundSponsor => OperationStep::new(
                "fund-sponsor-region",
                OperationSubject::SponsorFunding(Box::new(TargetSponsorFundingSubject {
                    sponsor_outputs: 1,
                    amount_per_sponsor_output: SPONSOR_FEE,
                })),
            ),
            Stage::FundReceipts => self.funding_step("fund-explicit-constructor", false),
            Stage::SignSponsor => match self.sign_step() {
                Ok(step) => step,
                Err(refusal) => return Err(self.refuse(refusal)),
            },
            Stage::Submit => match self.submit_step() {
                Ok(step) => step,
                Err(refusal) => return Err(self.refuse(refusal)),
            },
            Stage::Done => return Ok(None),
        };
        Ok(Some(step))
    }
}

/// The explicit constructor's destination program for the first
/// published owner.
fn destination_program(abi: &CandidateLiveTransferAbi) -> Result<Vec<u8>, Refusal> {
    Ok(abi
        .destinations()
        .get(
            &linker::OwnerParameter::new(
                published_owner(&FIRST_SCALAR).map_err(|_| Refusal::SubstrateUnavailable)?,
            ),
            LiveTransferRepresentationPlan::Explicit,
        )
        .ok_or(Refusal::SubstrateUnavailable)?
        .instance()
        .program()
        .to_vec())
}

fn destination(scalar: &[u8; 32], amount: u64) -> Result<LiveReceiptDestination, Refusal> {
    let owner = published_owner(scalar).map_err(|_| Refusal::SubstrateUnavailable)?;
    let value = ProtocolValue::new(amount).map_err(|_| Refusal::ControlNotConstructible)?;
    Ok(LiveReceiptDestination::new(
        linker::OwnerParameter::new(owner),
        value,
    ))
}

/// One wire outpoint, as this crate's own type.
fn outpoint_of(wire: &WireOutpoint) -> Option<Outpoint> {
    let raw = decode_hex(&wire.txid)?;
    let mut internal = <[u8; 32]>::try_from(raw.as_slice()).ok()?;
    // A target prints a transaction identity in the reverse of the order
    // it hashes it in.
    internal.reverse();
    Outpoint::new(Txid::from_internal(internal), wire.vout).ok()
}

/// One transaction identity, back in the spelling a target prints.
fn txid_to_wire(txid: &Txid) -> String {
    let mut bytes = *txid.internal();
    bytes.reverse();
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        text.push(char::from_digit(u32::from(byte >> 4), 16).unwrap_or('0'));
        text.push(char::from_digit(u32::from(byte & 0x0f), 16).unwrap_or('0'));
    }
    text
}

fn asset_of(text: &str) -> Option<AssetId> {
    let mut raw = decode_hex(text)?;
    raw.reverse();
    <[u8; 32]>::try_from(raw.as_slice())
        .ok()
        .map(AssetId::from_internal)
}

fn decode_hex(text: &str) -> Option<Vec<u8>> {
    let raw = text.as_bytes();
    let (pairs, remainder) = raw.as_chunks::<2>();
    if !remainder.is_empty() {
        return None;
    }
    let mut bytes = Vec::with_capacity(pairs.len());
    for pair in pairs {
        let digits = std::str::from_utf8(pair).ok()?;
        bytes.push(u8::from_str_radix(digits, 16).ok()?);
    }
    Some(bytes)
}

/// Whether `needle` occurs in `haystack` as a contiguous run.
fn contains_run(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn environment(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

fn identifier(text: &str) -> [u8; 32] {
    let mut bytes = [0_u8; 32];
    let raw = text.as_bytes();
    let (pairs, _) = raw.as_chunks::<2>();
    for (slot, pair) in bytes.iter_mut().zip(pairs) {
        if let Ok(digits) = std::str::from_utf8(pair)
            && let Ok(value) = u8::from_str_radix(digits, 16)
        {
            *slot = value;
        }
    }
    bytes
}

/// Everything up to the round trip: the environment, the deployment
/// binding, the executor, and one run of the lane.
///
/// Separated from the assertions because a function that both arranges
/// a run and judges it makes the judging hard to read past the
/// arranging.
fn run_the_lane() -> (RoundTrip, Submission, usize, Option<PathBuf>) {
    let executor =
        environment("TRIPOD_SPONSOR_EXECUTOR").expect("TRIPOD_SPONSOR_EXECUTOR names the adapter");
    let network = environment("TRIPOD_SPONSOR_NETWORK_ID")
        .expect("TRIPOD_SPONSOR_NETWORK_ID names the chain");
    let genesis = environment("TRIPOD_SPONSOR_GENESIS_ID")
        .expect("TRIPOD_SPONSOR_GENESIS_ID names the genesis block");
    let report = environment("TRIPOD_SPONSOR_REPORT").map(PathBuf::from);

    let target = reviewed_elements_tapscript().expect("the reviewed target validates");
    let binding = validate_reviewed_development_binding(
        &target,
        DevelopmentDeploymentBinding::new(
            target.definition().version(),
            DeploymentEnvironment::Development,
            identifier(&network),
            identifier(&genesis),
            ActivationDeclaration::new(true, LeafVersion::TAPSCRIPT, []),
            None,
        ),
    )
    .expect("the development binding validates");

    let timeout = environment("TRIPOD_SPONSOR_TIMEOUT_SECONDS")
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(DEFAULT_EXECUTOR_TIMEOUT, Duration::from_secs);
    let diagnostics = report
        .as_deref()
        .and_then(Path::parent)
        .unwrap_or_else(|| Path::new("."));
    let configuration = ExecutorConfiguration::new(
        Path::new(&executor),
        ExecutorTrust::ReviewedNonMock,
        timeout,
        ExecutorDiagnostics::in_directory(diagnostics),
    );

    let mut planner = SponsorSigningPlanner::new().expect("the candidate substrate builds");
    let outcome = execute_operations(&target, &binding, &configuration, &mut planner);

    // The submission's own record is read BEFORE the refusal is
    // asserted away, because a refused submission is exactly the
    // observation a typed stop is made of: the target's layer and its
    // own words have to survive the assertion that stops the run.
    if let Some(submission) = &planner.submission {
        assert!(
            submission.layer == ObservedOutcomeLayer::Accepted,
            "the target refused the sponsor-signed control at {:?}: {:?}",
            submission.layer,
            submission.detail,
        );
    }
    assert!(
        planner.refusal.is_none(),
        "the lane stopped before the submission: {:?}",
        planner.refusal
    );
    outcome.expect("the operation run completed");
    let round = planner
        .round
        .clone()
        .expect("the sponsor round trip completed");
    let submission = planner
        .submission
        .clone()
        .expect("the sponsor-signed control was submitted");
    (round, submission, planner.receipts.len(), report)
}

/// The four sub-steps the handoff card names, each observed separately.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_sponsor_envelope_signer_round_trips_through_the_adapter() {
    let started = Instant::now();
    let (round, submission, receipts, report) = run_the_lane();

    // (a) An explicit sponsored control was finalized. The builder asked
    // the envelope for exactly one sponsor authorization — the request
    // count is checked where it is collected — and asked for it at the
    // position the sponsor suffix occupies after every receipt input.
    assert!(
        !round.sent.is_empty(),
        "the finalized control carried no bytes"
    );
    assert_eq!(
        usize::from(round.input),
        receipts,
        "the sponsor input is not the suffix member following every receipt input"
    );

    // (b) The exact request reached the adapter. The echo is compared
    // byte for byte and not by length, because two serializations of one
    // shape have the same length and are not the same bytes.
    assert_eq!(
        round.echoed, round.sent,
        "the adapter signed bytes other than the ones it was sent"
    );

    // (c) The returned witness replays through the sponsor capability
    // and reaches the completed control.
    assert!(!round.witness.is_empty(), "the adapter returned no witness");
    assert!(
        !round.replayed.is_empty(),
        "the replayed control carried no bytes"
    );
    assert_ne!(
        round.replayed, round.placeholder,
        "the replay changed nothing, so no returned witness reached the control"
    );
    for item in &round.witness {
        assert!(
            contains_run(&round.replayed, item),
            "a returned witness item is absent from the replayed control"
        );
    }

    // (d) The binding is enforced rather than announced. The same
    // witness, bound to one mutated byte, is refused.
    let refusal = round
        .mutated_refusal
        .clone()
        .expect("a signature bound to mutated bytes was refused");
    assert!(
        refusal.contains("SponsorSignatureBindingMismatch"),
        "the refusal names something other than the binding: {refusal}"
    );

    // (e) A target ACCEPTED the sponsor-signed control. This is the
    // observation the residual's own clearing rule asks for, and the
    // four above could never have produced it: a returned byte stack is
    // not the sponsor owner's target authorization until a target has
    // accepted a control carrying it.
    //
    // The acceptance is relay-crossing rather than merely valid. The
    // submission path asks `testmempoolaccept` first and only mines
    // what the mempool allowed, so an accepted verdict here is a
    // transaction the node would relay — the boundary no sponsored
    // control had ever been offered to.
    assert_eq!(
        submission.layer,
        ObservedOutcomeLayer::Accepted,
        "the target did not accept the sponsor-signed control: {:?}",
        submission.detail,
    );
    let txid = submission
        .txid
        .clone()
        .expect("the acceptance names a transaction identity");
    assert_eq!(
        submission.sent, round.replayed,
        "the bytes submitted are not the bytes the replay produced"
    );
    let read_back = submission
        .read_back
        .clone()
        .expect("the acceptance carries the target's own copy");
    assert_eq!(
        read_back, submission.sent,
        "the target's copy of the mined transaction is not what was submitted"
    );
    let (block_hash, block_height) = submission
        .block
        .clone()
        .expect("the acceptance names the block it was mined into");

    // The run says in its own bytes what it did and what it did not
    // establish, where a lane can read it afterwards. Nothing is
    // printed: what a run found belongs in an artifact rather than in a
    // scrollback nobody keeps.
    let record = format!(
        "sponsor_round_trip\n\
         sent_bytes {}\n\
         echoed_bytes {}\n\
         witness_items {}\n\
         witness_item_bytes {:?}\n\
         replayed_bytes {}\n\
         placeholder_bytes {}\n\
         sponsor_input {}\n\
         mutated_binding_refusal {}\n\
         submitted_bytes {}\n\
         observed_layer {:?}\n\
         accepted_txid {}\n\
         readback_matches_submitted true\n\
         block_hash {}\n\
         block_height {}\n\
         relay_boundary_crossed true\n\
         establishes_sponsor_envelope_wire true\n\
         establishes_multi_party_sponsor_signing false\n\
         wall_seconds {:.1}\n",
        round.sent.len(),
        round.echoed.len(),
        round.witness.len(),
        round.witness.iter().map(Vec::len).collect::<Vec<_>>(),
        round.replayed.len(),
        round.placeholder.len(),
        round.input,
        refusal,
        submission.sent.len(),
        submission.layer,
        txid,
        block_hash,
        block_height,
        started.elapsed().as_secs_f64(),
    );
    if let Some(path) = report.as_deref() {
        std::fs::write(path, &record).expect("the run record is writable");
    }

    // The run says in its own bytes what it did NOT establish, in the
    // place a later reader will look. One fixed regtest key signed
    // once; that is a wire and a target acceptance, and it is not a
    // multi-party ceremony.
    assert!(
        record.contains("establishes_multi_party_sponsor_signing false"),
        "the run record does not say what it left unestablished",
    );
}

/// Wiring the signer does not clear the residual, and this says so where
/// a change would have to notice.
///
/// It runs in the ordinary lane rather than behind the node gate, on
/// purpose: the claim is about what this repository still carries, and a
/// claim only a node can check is one nobody checks.
#[test]
fn wiring_the_signer_leaves_the_carried_residual_standing() {
    assert!(
        vectors::live_evidence::carried_residuals().contains(
            &vectors::live_evidence::LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent
        ),
        "the sponsor residual left the carried set, and no submitted and accepted \
         control carrying a sponsor witness exists to have moved it"
    );
}
