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
//! It is NOT a discharge of the carried residual. The residual's own
//! clearing rule, at `vectors::live_evidence::LiveInfrastructureBlocker`,
//! asks for the sponsor owner's target authorization and says a returned
//! byte stack is not that until a target has ACCEPTED a control carrying
//! it. Nothing here submits anything, so nothing here can satisfy that
//! rule. A blocker moves on an observed result and never on a capability
//! existing — and "the capability is now reachable from a lane" is still
//! the capability existing.
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
//! # The control is not submittable, and that is reported rather than hidden
//!
//! The deployment's reserve asset is a fixture constant no chain has
//! issued, while the sponsor coin the adapter funds is one of its own.
//! The control this test completes therefore names a fee output in an
//! asset its sponsor input does not carry. That costs the round trip
//! nothing — the signature and its binding are over the bytes either way
//! — and it is one more reason the residual stays: a submission step
//! needs a reserve asset the chain knows about, which nothing in this
//! lane has.

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
    TargetFundingSubject, TargetSponsorFundingSubject, TargetSponsorSigningSubject, WireOutpoint,
    WireSighashProfile,
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
    FundReceipts,
    FundSponsor,
    SignSponsor,
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
}

/// The lane: issue, fund receipts, fund a sponsor coin, sign.
struct SponsorSigningPlanner {
    stage: Stage,
    abi: CandidateLiveTransferAbi,
    issued_asset: Option<String>,
    explicit_program: Vec<u8>,
    receipts: Vec<ObservedCoin>,
    sponsor: Option<ObservedCoin>,
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
            explicit_program,
            receipts: Vec::new(),
            sponsor: None,
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

    /// Weld the deployment to the asset the chain just issued.
    fn relink(&mut self, response: &NativeOperationResponse) -> Result<(), Refusal> {
        let asset = response
            .issued_asset
            .clone()
            .ok_or(Refusal::IssuanceNamedNoAsset)?;
        let identity = asset_of(&asset).ok_or(Refusal::IssuanceNamedNoAsset)?;
        let abi = live_abi_for_asset(*identity.internal()).map_err(|_| Refusal::RelinkRefused)?;
        self.explicit_program = destination_program(&abi)?;
        self.issued_asset = Some(asset);
        self.abi = abi;
        Ok(())
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
                    if let Err(refusal) = self.relink(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::FundReceipts;
                }
                Stage::FundReceipts => match Self::settle_funding(response) {
                    Ok(coins) => {
                        self.receipts = coins;
                        self.stage = Stage::FundSponsor;
                    }
                    Err(refusal) => return Err(self.refuse(refusal)),
                },
                Stage::FundSponsor => match Self::settle_funding(response) {
                    Ok(coins) => {
                        let Some(coin) = coins.into_iter().next() else {
                            return Err(self.refuse(Refusal::FundingCreatedNoPredecessor));
                        };
                        self.sponsor = Some(coin);
                        self.stage = Stage::SignSponsor;
                    }
                    Err(refusal) => return Err(self.refuse(refusal)),
                },
                Stage::SignSponsor => {
                    if let Err(refusal) = self.settle_signature(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::Done;
                }
                Stage::Done => {}
            }
        }

        let step = match self.stage {
            Stage::Issue => self.funding_step("issue-protocol-asset", true),
            Stage::FundReceipts => self.funding_step("fund-explicit-constructor", false),
            Stage::FundSponsor => OperationStep::new(
                "fund-sponsor-region",
                OperationSubject::SponsorFunding(Box::new(TargetSponsorFundingSubject {
                    sponsor_outputs: 1,
                    amount_per_sponsor_output: SPONSOR_FEE,
                })),
            ),
            Stage::SignSponsor => match self.sign_step() {
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

/// The four sub-steps the handoff card names, each observed separately.
#[test]
#[ignore = "needs a live Elements node and an executor adapter"]
fn the_sponsor_envelope_signer_round_trips_through_the_adapter() {
    let started = Instant::now();
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

    assert!(
        planner.refusal.is_none(),
        "the lane stopped before the round trip: {:?}",
        planner.refusal
    );
    outcome.expect("the operation run completed");
    let round = planner
        .round
        .clone()
        .expect("the sponsor round trip completed");

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
        planner.receipts.len(),
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
         submitted_anything false\n\
         clears_the_sponsor_residual false\n\
         wall_seconds {:.1}\n",
        round.sent.len(),
        round.echoed.len(),
        round.witness.len(),
        round.witness.iter().map(Vec::len).collect::<Vec<_>>(),
        round.replayed.len(),
        round.placeholder.len(),
        round.input,
        refusal,
        started.elapsed().as_secs_f64(),
    );
    if let Some(path) = report.as_deref() {
        std::fs::write(path, &record).expect("the run record is writable");
    }
    assert!(
        record.contains("clears_the_sponsor_residual false"),
        "the run record does not say what it left standing",
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
