//! The sponsored CONFIDENTIAL with-change shape, driven through the
//! private construction lane.
//!
//! The shape the sponsor arc has been walking toward, and the one the
//! section 15.2 `private-sponsor-values` row moves on. A sponsor whose
//! coin's VALUE is a commitment funds another party's private transfer,
//! pays an explicit fee in the reserve asset, and takes its remainder
//! back as a COMMITTED change output.
//!
//! # Why every one of those words is forced rather than chosen
//!
//! The sponsor coin's value is committed because that is the shape the
//! arc funded and pinned; its ASSET stays explicit because the covenant
//! introspects the asset and an introspection reads an explicit field.
//! The change is committed because a committed sponsor value REQUIRES
//! it: a target refuses `bad-txns-in-ne-out` for an asset whose inputs
//! carry blinders and whose outputs carry none, there being nothing to
//! absorb the input blinder. The fee is explicit because consensus
//! defines a fee by its explicitness.
//!
//! # Where the fee sits, and why it is not a destination
//!
//! In the NON-PROTOCOL FUNDING REGION, whose members are emitted as
//! explicit outputs and held outside both balance equations. The private
//! lane's destination vocabulary has a fee role, and using it here would
//! not work: a fixture's fee output projects with no reserve asset, so
//! the preflight asset comparison would refuse a reserve-asset fee
//! destination against the case's protocol asset, and the same output
//! would be summed into the protocol subtotal because the fee role
//! answers `is_a_protocol_member` with true. The fee role keeps the
//! meaning it has -- the SPONSORLESS fee funded from the receipts, in
//! the protocol asset -- and the sponsored fee rides in the region.
//!
//! # The two passes, and why a ceremony cannot skip one
//!
//! The bytes a sponsor must authorize exist only as a side effect of
//! running the builder: a signing request names the exact finalized
//! transaction, and the transaction crate keeps that constructor to
//! itself so no caller can ask a sponsor to authorize bytes that are not
//! the candidate's. So the control is assembled TWICE against ONE
//! finalization -- once with a recording envelope, purely to observe the
//! bytes, and once with the adapter's real witness replayed into it. The
//! placeholder-authorized control of the first pass is discarded and is
//! never evidence of anything.

use std::cell::RefCell;
use std::collections::BTreeMap;

use linker::OwnerParameter;
use linker::live_backend::LiveTransferRepresentationPlan;
use target_elements_conformance::confidential_fixture::{
    ConfidentialFixtureOutput, FixtureOutputRole, FrozenConfidentialFixtureRegistry,
    sponsor_reserve_handle,
};
use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId, OperationSubject,
    TargetSponsorFundingSubject, TargetSponsorSigningSubject, TargetSubmissionSubject,
    WireOutpoint, WireSighashProfile,
};
use transaction::bytes::{AssetId, COMMITMENT_BYTES, EXPLICIT_PREFIX, Outpoint, Txid, ValueField};
use transaction::live_census::OwnerSigningCensus;
use transaction::live_construct::{
    PrivateDestinationOpening, PrivateInputOpening, PrivateLiveFinalization, PrivateLiveOpenings,
    finalize_private_live_transfer,
};
use transaction::live_materialize::{
    ConfidentialInputRegion, ConfidentialOutputRole, FixtureOpeningReference,
    FrozenConfidentialFixtureView, NonProtocolFundingRegion, NonProtocolMember, SCALAR_BYTES,
};
use transaction::live_message::{WitnessVectorTreatment, candidate_owner_message};
use transaction::live_request::{
    LiveReceiptDestination, LiveTransferRequest, ProtocolValue, PublicTestRandomness,
    RequestedForm, SponsorChangeRequest,
};
use transaction::sponsor::{
    SponsorCapability, SponsorOffer, SponsorSignature, SponsorSigningRequest,
};
use transaction::taproot::Digest32;
use transaction::view::{PublicConstructionView, PublicOutputView};

use crate::confidential_materializer::{
    FirstPartyCommitmentCheck, ReferenceConfidentialMaterializer,
};
use crate::confidential_predecessor::PredecessorShape;
use crate::confidential_sponsor_reserve::{
    FUND_SPONSOR_STEP, SPONSOR_RESERVE_OUTPUTS, frozen_sponsor_reserve_registry,
    sponsor_reserve_subject,
};
use crate::error::VectorError;
use crate::live_owner_observation::{asset_of, decode_hex, outpoint_of, printed_order};
use crate::live_plan::{
    FIRST_SCALAR, LiveShapeVocabulary, SECOND_SCALAR, published_owner, reviewed_target,
};
use crate::live_private_restart::{
    ConsumedReceipt, LinkedDeployment, PrivateRestartRefusal, RestartConfidentialCoin,
    assemble_control, confidential_funding_step, issue_step, link_and_register,
    observe_funded_coins, private_program, verify_readback_signature,
};
use crate::live_proof_bearing_observation::{
    materialization_profiles, project_frozen, register_multi,
};

/// The successor case this ceremony registers.
const SUCCESSOR_HANDLE: &str = "ctf-v1/sponsored-private-successor";

/// What the sponsor pays as the fee, in the reserve asset.
///
/// The figure the sponsor arc's accepted explicit control carried, kept
/// so that a reader comparing the explicit and confidential sponsored
/// acceptances is comparing one number.
const SPONSOR_FEE: u64 = 250;

/// What the sponsor takes back, in the reserve asset.
const SPONSOR_CHANGE: u64 = 1_000;

/// The first receipt output's amount.
const PRIMARY_RECEIPT: u64 = 400_000_000;

/// The second receipt output's amount, which is the balancing one.
const BALANCING_RECEIPT: u64 = 300_000_000;

/// Which coin of the sponsor reserve case this ceremony spends.
///
/// The first, whose blinder is DERIVED. The second solves that case's
/// own balance, and spending a solved output would tie this ceremony's
/// input blinder sum to an arithmetic accident of the funding case.
const SPONSOR_COIN: usize = 0;

/// Why a sponsored private run could not be built or did not hold.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SponsoredPrivateRefusal {
    /// A construction step of the private lane refused.
    Private(PrivateRestartRefusal),
    /// The reserve case would not register, or its digest is absent.
    SponsorReserveUnavailable,
    /// The sponsor funding step created something other than the case.
    SponsorFundingCreatedNothing,
    /// A reported sponsor coin did not decode.
    MalformedSponsorCoin,
    /// The coin the chain reported is not the one the registry derived.
    SponsorCoinIsNotTheRegistrysOwn,
    /// The staging pass asked for a number of signatures that is not one.
    UnexpectedSponsorRequestCount(usize),
    /// The adapter declined, or echoed bytes that are not what it was
    /// handed.
    SponsorSignatureUnusable,
    /// The control was never staged, so there is nothing to submit.
    ControlNotStaged,
}

/// How the staged envelope answers a signing request.
enum Answers {
    /// Record what was asked and answer with a placeholder, so the
    /// request the adapter must be handed can be collected at all.
    ///
    /// The control this arm completes is DISCARDED. A
    /// placeholder-authorized control is not evidence and must never be
    /// able to become any.
    Recording(RefCell<Vec<(u16, Vec<u8>)>>),
    /// Answer with what the adapter returned, keyed by input position.
    Replaying(BTreeMap<u16, SponsorSignature>),
}

/// The sponsor envelope this lane stages, in both of its passes.
///
/// One type rather than two, because the offer must be identical across
/// the passes: a second spelling of it could disagree with the first
/// about what was finalized, and then the bytes the adapter signed would
/// not be the bytes replayed into.
struct StagedEnvelope {
    offer: SponsorOffer,
    answers: Answers,
}

impl SponsorCapability for StagedEnvelope {
    fn offer(&self) -> SponsorOffer {
        self.offer.clone()
    }

    /// No change destination is named here.
    ///
    /// Construction writes the sponsor's change to the deployment's own
    /// sponsor-change program and refuses any other, so naming one would
    /// either agree redundantly or disagree and be refused.
    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        None
    }

    fn sign(&self, request: &SponsorSigningRequest) -> Option<SponsorSignature> {
        match &self.answers {
            Answers::Recording(seen) => {
                seen.borrow_mut()
                    .push((request.input(), request.transaction().to_vec()));
                // The admitted stack width, carrying nothing. It exists
                // so the builder completes and the request can be
                // observed; the control it produces is thrown away.
                Some(SponsorSignature::new(
                    request.transaction().to_vec(),
                    vec![Vec::new(), Vec::new()],
                ))
            }
            Answers::Replaying(answers) => answers.get(&request.input()).cloned(),
        }
    }
}

/// One coin the node reported, in the form it reported it.
#[derive(Clone, Debug)]
struct ObservedSponsorCoin {
    outpoint: Outpoint,
    asset: AssetId,
    /// The value FIELD and not a number: a committed coin has no number
    /// here, and the owner's signature commits to the field verbatim.
    value: ValueField,
    program: Vec<u8>,
}

/// One thing established about the blinded sponsor coin.
///
/// A vocabulary of named checks rather than a row of booleans, on the
/// pattern the explicit sponsor lane's own census sets: a check added
/// here is one a run has to have seen hold, which a count written down
/// somewhere else would not give.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SponsoredSolveCheck {
    /// The node reported a value commitment and not an explicit amount.
    ChainReportedACommitment,
    /// The value the chain holds is the one the frozen registry derives
    /// for the same case.
    ///
    /// The load-bearing member. The registry derives it from published
    /// constants and no chain at all; the node reports what it actually
    /// stored. A run that failed this would have funded SOMETHING
    /// blinded while being unable to say what.
    CommitmentIsTheRegistrysOwn,
    /// Each coin's asset stayed explicit beside its committed value,
    /// which is the asymmetry an introspection requires.
    AssetStayedExplicit,
    /// A range proof accompanied each committed value.
    RangeproofPresent,
}

impl SponsoredSolveCheck {
    /// Every check, in the order a reader meets them.
    pub const ALL: [Self; 4] = [
        Self::ChainReportedACommitment,
        Self::CommitmentIsTheRegistrysOwn,
        Self::AssetStayedExplicit,
        Self::RangeproofPresent,
    ];

    /// The check's own name, for a record a person reads.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ChainReportedACommitment => "chain-reported-a-commitment",
            Self::CommitmentIsTheRegistrysOwn => "commitment-is-the-registrys-own",
            Self::AssetStayedExplicit => "asset-stayed-explicit",
            Self::RangeproofPresent => "rangeproof-present",
        }
    }
}

/// Which checks a run's blinded sponsor coin passed.
///
/// Named checks and never a scalar. A record carrying the opening would
/// be publishing one; what a reader needs is which claims held, each
/// recomputed from the node's report against values this workspace
/// derived without a chain.
#[derive(Clone, Debug, Default)]
pub struct SponsoredSolveCensus {
    held: std::collections::BTreeSet<SponsoredSolveCheck>,
}

impl SponsoredSolveCensus {
    /// Whether one named check held.
    #[must_use]
    pub fn holds(&self, check: SponsoredSolveCheck) -> bool {
        self.held.contains(&check)
    }

    /// Whether every check in the vocabulary held.
    #[must_use]
    pub fn every_check_held(&self) -> bool {
        SponsoredSolveCheck::ALL
            .iter()
            .all(|check| self.held.contains(check))
    }

    /// The checks that did NOT hold, in vocabulary order.
    #[must_use]
    pub fn missing(&self) -> Vec<SponsoredSolveCheck> {
        SponsoredSolveCheck::ALL
            .into_iter()
            .filter(|check| !self.held.contains(check))
            .collect()
    }

    /// Record that one check held.
    fn record(&mut self, check: SponsoredSolveCheck, held: bool) {
        if held {
            self.held.insert(check);
        }
    }
}

/// The sponsor's round trip through the adapter.
#[derive(Clone, Debug, Default)]
pub struct SponsoredRoundTrip {
    input: u16,
    sent: Vec<u8>,
    echo_matches_what_was_sent: bool,
    witness_items: usize,
}

impl SponsoredRoundTrip {
    /// Which input position the sponsor authorized.
    #[must_use]
    pub const fn input(&self) -> u16 {
        self.input
    }

    /// Whether the adapter echoed back exactly the bytes it was handed.
    #[must_use]
    pub const fn echo_matches_what_was_sent(&self) -> bool {
        self.echo_matches_what_was_sent
    }

    /// How many items the sponsor's witness carried.
    #[must_use]
    pub const fn witness_items(&self) -> usize {
        self.witness_items
    }
}

/// What the target did with the sponsored private control.
#[derive(Clone, Debug, Default)]
pub struct SponsoredPrivateReverification {
    accepted_txid: String,
    readback_matches_submission: bool,
    owner_signature_verified: bool,
    sponsor_change_located: bool,
}

impl SponsoredPrivateReverification {
    /// The identity the node printed for the mined transaction.
    #[must_use]
    pub fn accepted_txid(&self) -> &str {
        &self.accepted_txid
    }

    /// Whether the bytes the node reported are the bytes it was handed.
    #[must_use]
    pub const fn readback_matches_submission(&self) -> bool {
        self.readback_matches_submission
    }

    /// Whether an owner's signature verified against a message this
    /// ceremony recomputed independently of the candidate it submitted.
    #[must_use]
    pub const fn owner_signature_verified(&self) -> bool {
        self.owner_signature_verified
    }

    /// Whether the sponsor's committed change was found in the MINED
    /// bytes, at the reserve asset and carrying a commitment.
    #[must_use]
    pub const fn sponsor_change_located(&self) -> bool {
        self.sponsor_change_located
    }
}

/// One sponsored private run's transcript material.
#[derive(Clone, Debug, Default)]
pub struct SponsoredPrivateRecord {
    issued_asset: Option<String>,
    coins: Vec<RestartConfidentialCoin>,
    solve: Option<SponsoredSolveCensus>,
    sponsor_funding_txid: Option<String>,
    round: Option<SponsoredRoundTrip>,
    submitted_bytes: usize,
    observed_layer: Option<ObservedOutcomeLayer>,
    observed_detail: Option<String>,
    accepted_txid: Option<String>,
    target_weight: Option<u64>,
    reverification: Option<SponsoredPrivateReverification>,
    refusal: Option<SponsoredPrivateRefusal>,
}

impl SponsoredPrivateRecord {
    /// The predecessor coins the node funded.
    #[must_use]
    pub fn coins(&self) -> &[RestartConfidentialCoin] {
        &self.coins
    }

    /// What was established about the blinded sponsor coin.
    #[must_use]
    pub const fn solve(&self) -> Option<&SponsoredSolveCensus> {
        self.solve.as_ref()
    }

    /// The sponsor's round trip through the adapter.
    #[must_use]
    pub const fn round(&self) -> Option<&SponsoredRoundTrip> {
        self.round.as_ref()
    }

    /// How many bytes reached the node.
    #[must_use]
    pub const fn submitted_bytes(&self) -> usize {
        self.submitted_bytes
    }

    /// Which layer the target's answer came from.
    #[must_use]
    pub const fn observed_layer(&self) -> Option<ObservedOutcomeLayer> {
        self.observed_layer
    }

    /// The weight the target computed.
    #[must_use]
    pub const fn target_weight(&self) -> Option<u64> {
        self.target_weight
    }

    /// The checks made against the mined bytes.
    #[must_use]
    pub const fn reverification(&self) -> Option<&SponsoredPrivateReverification> {
        self.reverification.as_ref()
    }

    /// The refusal that stopped the run, where one did.
    #[must_use]
    pub const fn refusal(&self) -> Option<&SponsoredPrivateRefusal> {
        self.refusal.as_ref()
    }
}

/// Which sponsored private shape a ceremony builds.
///
/// TWO, and they differ on the sponsor's side rather than the receipts'.
/// The sponsor arc built the first and §16.1's sponsor pair states the
/// second, and the pair's private member is not answered by the arc's
/// run: that one carries a BLINDED sponsor coin, a COMMITTED sponsor
/// change, and two blinded destinations, where the member states an
/// EXPLICIT sponsor coin funded exactly to the fee, no change role, and
/// one destination.
///
/// # The explicit shape balances for a reason worth stating
///
/// The arc observed that a COMMITTED sponsor value requires committed
/// change: a blinded input's blinder has to be absorbed by something,
/// a fee is mandatorily explicit, and the change is the only remaining
/// term. That observation does not reach this shape and does not forbid
/// it. An EXPLICIT sponsor coin contributes the all-zero blinder every
/// explicit value is committed with, so there is nothing to absorb and
/// no change is owed -- the sponsor input equals the fee output in the
/// reserve asset, and the protocol asset closes over the receipts alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SponsoredPrivateShape {
    /// A committed sponsor coin, committed change, two destinations.
    CommittedWithChange,
    /// An EXPLICIT sponsor coin funded exactly to the fee, no change
    /// role, one destination: §16.1's sponsor pair's private member.
    ExplicitWithoutChange,
}

impl SponsoredPrivateShape {
    /// Both shapes, in the order they were built.
    pub const ALL: [Self; 2] = [Self::CommittedWithChange, Self::ExplicitWithoutChange];

    /// The name a transcript files this run under.
    #[must_use]
    pub const fn case_name(self) -> &'static str {
        match self {
            Self::CommittedWithChange => "sponsored-private-with-change",
            Self::ExplicitWithoutChange => "sponsored-private-explicit-no-change",
        }
    }

    /// The successor fixture handle, its own per shape so a digest drift
    /// between the two is detectable.
    #[must_use]
    pub const fn successor_handle(self) -> &'static str {
        match self {
            Self::CommittedWithChange => SUCCESSOR_HANDLE,
            Self::ExplicitWithoutChange => "ctf-v1/sponsored-private-explicit-successor",
        }
    }

    /// Whether the sponsor's coin carries a value COMMITMENT.
    #[must_use]
    pub const fn commits_the_sponsor_value(self) -> bool {
        matches!(self, Self::CommittedWithChange)
    }

    /// Whether the offer asks for change back.
    #[must_use]
    pub const fn requests_change(self) -> bool {
        matches!(self, Self::CommittedWithChange)
    }

    /// What the explicit seed step funds the sponsor region with.
    ///
    /// The committed shape funds ABOVE the fee because it takes a
    /// remainder back; the explicit shape funds EXACTLY the fee, which
    /// is what "no change role" means in an amount.
    #[must_use]
    pub const fn seed_amount(self) -> u64 {
        match self {
            Self::CommittedWithChange => SPONSOR_FEE + SPONSOR_CHANGE,
            Self::ExplicitWithoutChange => SPONSOR_FEE,
        }
    }

    /// How many protocol receipts the successor creates.
    #[must_use]
    pub const fn destination_count(self) -> usize {
        match self {
            Self::CommittedWithChange => 2,
            Self::ExplicitWithoutChange => 1,
        }
    }
}

/// Where the ceremony is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Issue,
    /// The EXPLICIT sponsor coin, funded first and never instead of the
    /// committed one.
    ///
    /// The committed step's fixture is built against two facts only this
    /// step reports: which asset the chain's reserve actually is, and
    /// which program the adapter can authorize a spend of. A ceremony
    /// that guessed either would register a case against one reserve
    /// while the executor funded against another, and the digests would
    /// disagree.
    FundSponsorSeed,
    FundSponsor,
    FundReceipts,
    SignSponsor,
    Submit,
    Done,
}

/// The sponsored private ceremony.
pub struct SponsoredPrivatePlanner {
    stage: Stage,
    shape: SponsoredPrivateShape,
    genesis_block_hash: Digest32,
    linked: Option<LinkedDeployment>,
    receipt: Option<RestartConfidentialCoin>,
    sponsor: Option<ObservedSponsorCoin>,
    /// The chain's own reserve asset, learned rather than assumed.
    reserve: Option<AssetId>,
    /// The program the adapter can authorize a spend of.
    adapter_program: Option<Vec<u8>>,
    sponsor_registry: Option<FrozenConfidentialFixtureRegistry>,
    successor: Option<(
        [u8; 32],
        transaction::live_materialize::ConfidentialFixtureView,
    )>,
    staged: Option<PrivateLiveFinalization>,
    submitted: Option<Vec<u8>>,
    census: Option<OwnerSigningCensus>,
    spent_owner_bytes: Option<Vec<u8>>,
    record: SponsoredPrivateRecord,
}

impl SponsoredPrivatePlanner {
    /// The ceremony, bound to a deployment's printed genesis identity.
    ///
    /// # Errors
    ///
    /// [`VectorError::LiveSubstrateUnavailable`] where the reviewed
    /// target does not build.
    pub fn new(printed_genesis_identity: Digest32) -> Result<Self, VectorError> {
        Self::for_shape(
            SponsoredPrivateShape::CommittedWithChange,
            printed_genesis_identity,
        )
    }

    /// The ceremony for one named shape.
    ///
    /// # Errors
    ///
    /// [`VectorError::LiveSubstrateUnavailable`] where the reviewed
    /// target does not build.
    pub fn for_shape(
        shape: SponsoredPrivateShape,
        printed_genesis_identity: Digest32,
    ) -> Result<Self, VectorError> {
        reviewed_target()?;
        Ok(Self {
            stage: Stage::Issue,
            shape,
            // The seed in the order a target HASHES a block identity,
            // not the order it prints one. The owner message writes this
            // hash twice, so a printed-order seed is a message over the
            // wrong preimage -- every balance check still passes and the
            // signature alone fails, which is exactly the recorded
            // `DeploymentSeedInPrintedOrder` negative control.
            genesis_block_hash: printed_order(printed_genesis_identity),
            linked: None,
            receipt: None,
            sponsor: None,
            reserve: None,
            adapter_program: None,
            sponsor_registry: None,
            successor: None,
            staged: None,
            submitted: None,
            census: None,
            spent_owner_bytes: None,
            record: SponsoredPrivateRecord::default(),
        })
    }

    /// The run's transcript material.
    #[must_use]
    pub const fn record(&self) -> &SponsoredPrivateRecord {
        &self.record
    }

    fn refuse(&mut self, refusal: SponsoredPrivateRefusal) -> PlanRefused {
        self.record.refusal = Some(refusal);
        self.stage = Stage::Done;
        PlanRefused
    }

    /// Record the asset the node issued.
    ///
    /// The deployment is NOT linked here. It is welded to a reserve
    /// asset, and which asset the chain's reserve is has not been
    /// reported yet -- the explicit sponsor step is what reports it.
    fn settle_asset(&mut self, printed: &str) {
        self.record.issued_asset = Some(printed.to_owned());
    }

    /// Learn the reserve asset and the adapter's program, then link the
    /// deployment against the reserve the chain actually has.
    fn settle_sponsor_seed(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), SponsoredPrivateRefusal> {
        let funded = response
            .funded_outputs
            .first()
            .ok_or(SponsoredPrivateRefusal::SponsorFundingCreatedNothing)?;
        let reserve =
            asset_of(&funded.asset).ok_or(SponsoredPrivateRefusal::MalformedSponsorCoin)?;
        let program =
            decode_hex(&funded.script).ok_or(SponsoredPrivateRefusal::MalformedSponsorCoin)?;
        let printed = self
            .record
            .issued_asset
            .clone()
            .ok_or(SponsoredPrivateRefusal::Private(
                PrivateRestartRefusal::IssuanceNamedNoAsset,
            ))?;
        let linked = link_and_register(
            PredecessorShape::DualParity,
            ConsumedReceipt::Primary,
            &printed,
            // FEE-BEARING and not the demonstration vocabulary. A
            // sponsored request bears a fee by construction, so this
            // candidate EXECUTES the covenant's fee clause -- and the
            // demonstration deployment is welded to a fixture fee digest
            // that no program hashes to, which would demand of the fee
            // output a scriptPubKey digest nothing can present and
            // refuse this ceremony's own candidate at OP_EQUALVERIFY.
            // The fee-bearing vocabulary is a separate deployment with
            // its own taptree, so the demonstration's committed identity
            // does not move to buy this.
            LiveShapeVocabulary::FeeBearing,
            *reserve.internal(),
        )
        .map_err(SponsoredPrivateRefusal::Private)?;
        self.reserve = Some(reserve);
        self.adapter_program = Some(program.clone());
        self.linked = Some(linked);
        // The EXPLICIT shape spends this very coin, so the seed step is
        // its sponsor funding step and there is no committed one to
        // wait for. Its value FIELD is explicit, which is the whole
        // difference the pair's private member states, and the owner's
        // signature commits to that field verbatim exactly as it does
        // to a commitment.
        if !self.shape.commits_the_sponsor_value() {
            let outpoint = outpoint_of(&funded.outpoint)
                .ok_or(SponsoredPrivateRefusal::MalformedSponsorCoin)?;
            self.sponsor = Some(ObservedSponsorCoin {
                outpoint,
                asset: reserve,
                value: ValueField::Explicit(self.shape.seed_amount()),
                program,
            });
        }
        Ok(())
    }

    /// The chain's reserve asset, once learned.
    fn reserve(&self) -> Result<AssetId, SponsoredPrivateRefusal> {
        self.reserve
            .ok_or(SponsoredPrivateRefusal::SponsorReserveUnavailable)
    }

    /// Take the funded predecessor coins from the node's report.
    fn settle_receipts(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), SponsoredPrivateRefusal> {
        let linked = self
            .linked
            .as_ref()
            .ok_or(SponsoredPrivateRefusal::ControlNotStaged)?;
        let coins =
            observe_funded_coins(linked, response).map_err(SponsoredPrivateRefusal::Private)?;
        self.receipt = coins.get(ConsumedReceipt::Primary.index()).cloned();
        self.record.coins = coins;
        Ok(())
    }

    /// The step that funds the blinded sponsor coin.
    ///
    /// Against the reserve the chain reported and the program the
    /// adapter can authorize, both learned from the explicit step above.
    /// The coin is paid to the ADAPTER's program and not to the
    /// deployment's sponsor-change one: the first is what can be spent,
    /// the second is where the remainder goes, and they are different
    /// programs for different reasons.
    fn sponsor_funding_step(&mut self) -> Result<OperationStep, SponsoredPrivateRefusal> {
        let reserve = self.reserve()?;
        let program = self
            .adapter_program
            .clone()
            .ok_or(SponsoredPrivateRefusal::SponsorReserveUnavailable)?;
        let registry = frozen_sponsor_reserve_registry(*reserve.internal(), &program)
            .map_err(|_| SponsoredPrivateRefusal::SponsorReserveUnavailable)?;
        let digest = *registry
            .registered_digest(&sponsor_reserve_handle())
            .ok_or(SponsoredPrivateRefusal::SponsorReserveUnavailable)?;
        self.sponsor_registry = Some(registry);
        Ok(OperationStep::new(
            FUND_SPONSOR_STEP,
            OperationSubject::ConfidentialSponsorFunding(Box::new(sponsor_reserve_subject(
                digest, &program,
            ))),
        ))
    }

    /// Take the blinded sponsor coin, and census what it is.
    fn settle_sponsor(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), SponsoredPrivateRefusal> {
        let reported = &response.confidential_funded_outputs;
        if reported.len() != SPONSOR_RESERVE_OUTPUTS {
            return Err(SponsoredPrivateRefusal::SponsorFundingCreatedNothing);
        }
        let registry = self
            .sponsor_registry
            .as_ref()
            .ok_or(SponsoredPrivateRefusal::SponsorReserveUnavailable)?;
        let handle = sponsor_reserve_handle();
        let digest = *registry
            .registered_digest(&handle)
            .ok_or(SponsoredPrivateRefusal::SponsorReserveUnavailable)?;
        let derived = registry
            .resolve(&handle, &digest)
            .map_err(|_| SponsoredPrivateRefusal::SponsorReserveUnavailable)?
            .value_commitments()
            .ok_or(SponsoredPrivateRefusal::SponsorReserveUnavailable)?;

        let mut fields = Vec::with_capacity(reported.len());
        for output in reported {
            let field: [u8; COMMITMENT_BYTES] = output
                .value_commitment
                .clone()
                .try_into()
                .map_err(|_| SponsoredPrivateRefusal::MalformedSponsorCoin)?;
            fields.push(field);
        }
        let first = &reported[SPONSOR_COIN];
        let outpoint =
            outpoint_of(&first.outpoint).ok_or(SponsoredPrivateRefusal::MalformedSponsorCoin)?;
        let asset =
            asset_of(&first.explicit_asset).ok_or(SponsoredPrivateRefusal::MalformedSponsorCoin)?;
        let program =
            decode_hex(&first.script).ok_or(SponsoredPrivateRefusal::MalformedSponsorCoin)?;

        let mut census = SponsoredSolveCensus::default();
        census.record(
            SponsoredSolveCheck::ChainReportedACommitment,
            fields.iter().all(|field| field[0] != EXPLICIT_PREFIX),
        );
        census.record(
            SponsoredSolveCheck::CommitmentIsTheRegistrysOwn,
            fields == derived,
        );
        census.record(
            SponsoredSolveCheck::AssetStayedExplicit,
            reported
                .iter()
                .all(|output| asset_of(&output.explicit_asset) == Some(asset)),
        );
        census.record(
            SponsoredSolveCheck::RangeproofPresent,
            reported.iter().all(|output| !output.rangeproof.is_empty()),
        );
        if !census.holds(SponsoredSolveCheck::CommitmentIsTheRegistrysOwn) {
            return Err(SponsoredPrivateRefusal::SponsorCoinIsNotTheRegistrysOwn);
        }
        self.record.solve = Some(census);
        if let Some(readback) = response.mined_readback.as_ref() {
            self.record.sponsor_funding_txid = Some(readback.transaction_id.clone());
        }
        self.sponsor = Some(ObservedSponsorCoin {
            outpoint,
            asset,
            value: ValueField::Commitment(fields[SPONSOR_COIN]),
            program,
        });
        Ok(())
    }

    /// The successor case: two protocol receipts and the sponsor's
    /// committed remainder, from ONE registered case declaring two
    /// assets across its positions.
    fn register_successor(&mut self) -> Result<(), SponsoredPrivateRefusal> {
        let linked = self
            .linked
            .as_ref()
            .ok_or(SponsoredPrivateRefusal::ControlNotStaged)?;
        let recipient = private_program(&linked.abi, &SECOND_SCALAR)
            .map_err(SponsoredPrivateRefusal::Private)?;
        let sender = private_program(&linked.abi, &FIRST_SCALAR)
            .map_err(SponsoredPrivateRefusal::Private)?;
        let outputs = if self.shape.commits_the_sponsor_value() {
            let change = sponsor_program(linked)?;
            vec![
                ConfidentialFixtureOutput {
                    role: FixtureOutputRole::Primary,
                    semantic_amount: PRIMARY_RECEIPT,
                    output_program: recipient,
                },
                ConfidentialFixtureOutput {
                    role: FixtureOutputRole::Balancing,
                    semantic_amount: BALANCING_RECEIPT,
                    output_program: sender,
                },
                ConfidentialFixtureOutput {
                    role: FixtureOutputRole::SponsorChange {
                        asset: *self.reserve()?.internal(),
                    },
                    semantic_amount: SPONSOR_CHANGE,
                    output_program: change,
                },
            ]
        } else {
            // ONE output, declaring the fully-solved form, exactly as
            // the strict one-to-one's lone output does. There is no
            // second protocol output to absorb anything and none is
            // needed: the whole consumed amount travels to the
            // recipient, and the sponsor's side of the transaction is a
            // fee output this manifest does not carry -- a fee has an
            // explicit value and no program, which is most of what makes
            // it a fee. No SponsorChange role appears, because the
            // sponsor funded exactly the fee and asked nothing back.
            let _ = &sender;
            vec![ConfidentialFixtureOutput {
                role: FixtureOutputRole::SoleBalancing,
                semantic_amount: PRIMARY_RECEIPT + BALANCING_RECEIPT,
                output_program: recipient,
            }]
        };

        let handle = self.shape.successor_handle();
        let (digest, view) = register_multi(
            handle,
            *linked.asset.internal(),
            self.input_blinder_sum()?,
            outputs,
        )
        .map_err(|_| {
            SponsoredPrivateRefusal::Private(PrivateRestartRefusal::FixtureNotRegistrable {
                handle: handle.to_owned(),
            })
        })?;
        self.successor = Some((*digest.bytes(), view));
        Ok(())
    }

    /// The successor's input blinder sum: the consumed receipt's blinder
    /// AND the sponsor coin's.
    ///
    /// The sponsor's blinder is an addend and its amount is not. The
    /// target's excess is a sum over every input alike whatever asset
    /// generator each was built against, so a sum that left the sponsor
    /// coin out would build a candidate whose commitments do not close
    /// -- and the protocol subtotal, which the sponsor really is outside
    /// of, is a different equation entirely.
    fn input_blinder_sum(&self) -> Result<[u8; 32], SponsoredPrivateRefusal> {
        let linked = self
            .linked
            .as_ref()
            .ok_or(SponsoredPrivateRefusal::ControlNotStaged)?;
        let receipt = linked
            .predecessor_view
            .outputs()
            .get(ConsumedReceipt::Primary.index())
            .and_then(|output| output.value_blinder().copied())
            .ok_or(SponsoredPrivateRefusal::Private(
                PrivateRestartRefusal::PredecessorBlindersDoNotClose,
            ))?;
        // An EXPLICIT sponsor coin is committed with the all-zero
        // blinder, so it is not an addend and there is no sponsor
        // fixture to read one out of. The sum is the consumed receipt's
        // own blinder, which is nonzero for the reason a single consumed
        // coin's blinder is.
        if !self.shape.commits_the_sponsor_value() {
            return Ok(receipt);
        }
        let sponsor = self
            .sponsor_view()?
            .outputs()
            .get(SPONSOR_COIN)
            .and_then(|output| output.value_blinder().copied())
            .ok_or(SponsoredPrivateRefusal::SponsorReserveUnavailable)?;
        target_elements_conformance::confidential_fixture::sum_blinders(&[receipt, sponsor])
            .ok_or(SponsoredPrivateRefusal::SponsorReserveUnavailable)
    }

    /// The sponsor reserve case, projected.
    fn sponsor_view(
        &self,
    ) -> Result<transaction::live_materialize::ConfidentialFixtureView, SponsoredPrivateRefusal>
    {
        let registry = self
            .sponsor_registry
            .as_ref()
            .ok_or(SponsoredPrivateRefusal::SponsorReserveUnavailable)?;
        let handle = sponsor_reserve_handle();
        let digest = *registry
            .registered_digest(&handle)
            .ok_or(SponsoredPrivateRefusal::SponsorReserveUnavailable)?;
        project_frozen(registry, &handle, &digest)
            .map_err(|_| SponsoredPrivateRefusal::SponsorReserveUnavailable)
    }

    /// The sponsor's offer: the coin it brings, the fee it pays, and the
    /// remainder it asks back.
    fn offer(&self) -> Result<SponsorOffer, SponsoredPrivateRefusal> {
        let sponsor = self
            .sponsor
            .as_ref()
            .ok_or(SponsoredPrivateRefusal::ControlNotStaged)?;
        SponsorOffer::new(
            [sponsor.outpoint],
            SPONSOR_FEE,
            // Stated EXPLICITLY even though the output is committed. The
            // offer's number is what construction reconciles a declared
            // change position against; the output's value form is the
            // materializer's business and is committed because the
            // fixture says so.
            //
            // NONE for the explicit shape, and the absence is the offer
            // itself rather than an omission: a sponsor that funded
            // exactly the fee has no remainder to ask back, and stating
            // a change amount of zero would declare a change POSITION
            // for a role the candidate does not carry.
            self.shape
                .requests_change()
                .then_some(ValueField::Explicit(SPONSOR_CHANGE)),
        )
        .map_err(|_| SponsoredPrivateRefusal::ControlNotStaged)
    }

    /// Every fixture case the finalization may resolve an opening
    /// against.
    ///
    /// The sponsor reserve case is present only where the sponsor coin
    /// was BLINDED. An explicit coin has no opening to resolve, and
    /// offering a fixture nothing references would register a case this
    /// candidate does not use.
    fn fixture_views(
        &self,
        linked: &LinkedDeployment,
        successor_view: &transaction::live_materialize::ConfidentialFixtureView,
    ) -> Result<FrozenConfidentialFixtureView, SponsoredPrivateRefusal> {
        let mut registered = BTreeMap::from([
            (
                linked.predecessor.handle().as_str().to_owned(),
                linked.predecessor_view.clone(),
            ),
            (
                self.shape.successor_handle().to_owned(),
                successor_view.clone(),
            ),
        ]);
        if self.shape.commits_the_sponsor_value() {
            registered.insert(
                sponsor_reserve_handle().as_str().to_owned(),
                self.sponsor_view()?,
            );
        }
        Ok(FrozenConfidentialFixtureView::new(registered))
    }

    /// The openings the finalization is formed against.
    ///
    /// Its own method because it is the whole statement of what this
    /// shape IS: which coin fills each input region, which fixture
    /// position each destination realizes, and the one region member
    /// that is the fee.
    fn openings(
        &self,
        linked: &LinkedDeployment,
        successor_digest: &[u8; 32],
    ) -> Result<PrivateLiveOpenings, SponsoredPrivateRefusal> {
        let handle = self.shape.successor_handle();
        // The sponsor input's opening, where the coin has one. A
        // COMMITTED coin's value is a fixture position and the
        // finalization resolves it; an EXPLICIT coin's value is the
        // number itself, so it carries no opening and states its amount
        // directly. `None` here is the explicit form rather than a
        // missing reference.
        let sponsor_input = if self.shape.commits_the_sponsor_value() {
            PrivateInputOpening {
                region: ConfidentialInputRegion::SponsorReserve,
                opening: Some(FixtureOpeningReference::new(
                    sponsor_reserve_handle().as_str().to_owned(),
                    *self.sponsor_digest()?.bytes(),
                    SPONSOR_COIN,
                )),
                explicit_amount: SPONSOR_FEE + SPONSOR_CHANGE,
                zero_asset_blinder: [0_u8; SCALAR_BYTES],
            }
        } else {
            PrivateInputOpening {
                region: ConfidentialInputRegion::SponsorReserve,
                opening: None,
                explicit_amount: self.shape.seed_amount(),
                zero_asset_blinder: [0_u8; SCALAR_BYTES],
            }
        };
        let destinations = if self.shape.commits_the_sponsor_value() {
            vec![
                PrivateDestinationOpening {
                    fixture: FixtureOpeningReference::new(handle.to_owned(), *successor_digest, 0),
                    role: ConfidentialOutputRole::Primary,
                },
                PrivateDestinationOpening {
                    fixture: FixtureOpeningReference::new(handle.to_owned(), *successor_digest, 1),
                    role: ConfidentialOutputRole::Balancing,
                },
                PrivateDestinationOpening {
                    fixture: FixtureOpeningReference::new(handle.to_owned(), *successor_digest, 2),
                    role: ConfidentialOutputRole::SponsorChange,
                },
            ]
        } else {
            // ONE destination, at the sole-balancing role its manifest
            // declares. No sponsor-change opening appears because the
            // candidate carries no sponsor-change output to open.
            vec![PrivateDestinationOpening {
                fixture: FixtureOpeningReference::new(handle.to_owned(), *successor_digest, 0),
                role: ConfidentialOutputRole::Balancing,
            }]
        };
        Ok(PrivateLiveOpenings::new(
            vec![
                PrivateInputOpening {
                    region: ConfidentialInputRegion::Receipt,
                    opening: Some(FixtureOpeningReference::new(
                        linked.predecessor.handle().as_str().to_owned(),
                        linked.predecessor_digest(),
                        ConsumedReceipt::Primary.index(),
                    )),
                    explicit_amount: linked.predecessor.amounts()[ConsumedReceipt::Primary.index()],
                    zero_asset_blinder: [0_u8; SCALAR_BYTES],
                },
                sponsor_input,
            ],
            destinations,
            // The fee, outside both balance equations. Explicit, in the
            // reserve asset, and carrying the EMPTY program that is a
            // fee's whole identity at the target.
            NonProtocolFundingRegion::new(vec![NonProtocolMember::new(
                self.reserve()?,
                SPONSOR_FEE,
                Vec::new(),
            )]),
            materialization_profiles(),
        ))
    }

    /// The finalization both passes are assembled against.
    fn finalize(&self) -> Result<PrivateLiveFinalization, SponsoredPrivateRefusal> {
        let linked = self
            .linked
            .as_ref()
            .ok_or(SponsoredPrivateRefusal::ControlNotStaged)?;
        let receipt = self
            .receipt
            .as_ref()
            .ok_or(SponsoredPrivateRefusal::ControlNotStaged)?;
        let sponsor = self
            .sponsor
            .as_ref()
            .ok_or(SponsoredPrivateRefusal::ControlNotStaged)?;
        let (successor_digest, successor_view) = self
            .successor
            .as_ref()
            .ok_or(SponsoredPrivateRefusal::ControlNotStaged)?;

        // The view is the node's report of both coins, never the
        // ceremony's expectation of them.
        let view = PublicConstructionView::new([
            PublicOutputView::new(
                receipt.outpoint(),
                receipt.asset(),
                receipt.value(),
                receipt.program().to_vec(),
            ),
            PublicOutputView::new(
                sponsor.outpoint,
                transaction::bytes::AssetField::Explicit(sponsor.asset),
                sponsor.value,
                sponsor.program.clone(),
            ),
        ])
        .map_err(|_| {
            SponsoredPrivateRefusal::Private(PrivateRestartRefusal::ControlNotRequestable)
        })?;

        let recipient = published_owner(&SECOND_SCALAR).map_err(|_| {
            SponsoredPrivateRefusal::Private(PrivateRestartRefusal::SubstrateUnavailable)
        })?;
        let sender = published_owner(&FIRST_SCALAR).map_err(|_| {
            SponsoredPrivateRefusal::Private(PrivateRestartRefusal::SubstrateUnavailable)
        })?;
        let destination = |owner, amount| {
            ProtocolValue::new(amount)
                .map(|value| LiveReceiptDestination::new(OwnerParameter::new(owner), value))
                .map_err(|_| {
                    SponsoredPrivateRefusal::Private(PrivateRestartRefusal::ControlNotRequestable)
                })
        };

        let destinations = if self.shape.commits_the_sponsor_value() {
            vec![
                destination(recipient, PRIMARY_RECEIPT)?,
                destination(sender.clone(), BALANCING_RECEIPT)?,
                // The sponsor-change position. Its OWNER is never read:
                // construction pays the deployment's own sponsor-change
                // program, because returning a sponsor's reserve to a
                // live receipt constructor would be a receipt nobody can
                // spend and a sponsor who is not repaid.
                destination(sender, SPONSOR_CHANGE)?,
            ]
        } else {
            vec![destination(recipient, PRIMARY_RECEIPT + BALANCING_RECEIPT)?]
        };

        let request = LiveTransferRequest::new(
            [receipt.outpoint()],
            destinations,
            LiveTransferRepresentationPlan::PrivateCommitted,
            RequestedForm::Sponsored,
            if self.shape.requests_change() {
                SponsorChangeRequest::Requested
            } else {
                SponsorChangeRequest::NotRequested
            },
            Some(PublicTestRandomness::from_published_bytes([0x7e; 32])),
        )
        .map_err(|_| {
            SponsoredPrivateRefusal::Private(PrivateRestartRefusal::ControlNotRequestable)
        })?;

        let openings = self.openings(linked, successor_digest)?;

        let fixtures = self.fixture_views(linked, successor_view)?;

        let envelope = StagedEnvelope {
            offer: self.offer()?,
            answers: Answers::Recording(RefCell::new(Vec::new())),
        };

        finalize_private_live_transfer(
            &reviewed_target().map_err(|_| {
                SponsoredPrivateRefusal::Private(PrivateRestartRefusal::SubstrateUnavailable)
            })?,
            &linked.abi,
            &request,
            &view,
            Some(&envelope),
            &openings,
            &fixtures,
            &ReferenceConfidentialMaterializer::new(),
            &FirstPartyCommitmentCheck::new(),
        )
        .map_err(|refusal| {
            SponsoredPrivateRefusal::Private(PrivateRestartRefusal::FinalizationRefused(format!(
                "{refusal:?}"
            )))
        })
    }

    /// The sponsor reserve case's digest.
    fn sponsor_digest(
        &self,
    ) -> Result<
        target_elements_conformance::protocol::ConfidentialFixtureDigest,
        SponsoredPrivateRefusal,
    > {
        let registry = self
            .sponsor_registry
            .as_ref()
            .ok_or(SponsoredPrivateRefusal::SponsorReserveUnavailable)?;
        registry
            .registered_digest(&sponsor_reserve_handle())
            .copied()
            .ok_or(SponsoredPrivateRefusal::SponsorReserveUnavailable)
    }

    /// Pass one: assemble the control against a recording envelope,
    /// purely to observe the bytes the sponsor must authorize.
    fn sign_step(&mut self) -> Result<OperationStep, SponsoredPrivateRefusal> {
        self.register_successor()?;
        let finalization = self.finalize()?;

        let seen = RefCell::new(Vec::new());
        let recording = StagedEnvelope {
            offer: self.offer()?,
            answers: Answers::Recording(seen),
        };
        // The control this produces is discarded; only what the envelope
        // recorded survives the call.
        let discarded = assemble_control(&finalization, self.genesis_block_hash, Some(&recording))
            .map_err(SponsoredPrivateRefusal::Private)?;
        drop(discarded);
        let Answers::Recording(seen) = recording.answers else {
            return Err(SponsoredPrivateRefusal::ControlNotStaged);
        };
        let recorded = seen.into_inner();
        if recorded.len() != 1 {
            return Err(SponsoredPrivateRefusal::UnexpectedSponsorRequestCount(
                recorded.len(),
            ));
        }
        let (input, sent) = recorded[0].clone();
        let sponsor = self
            .sponsor
            .clone()
            .ok_or(SponsoredPrivateRefusal::ControlNotStaged)?;

        self.record.round = Some(SponsoredRoundTrip {
            input,
            sent: sent.clone(),
            ..SponsoredRoundTrip::default()
        });
        self.staged = Some(finalization);

        Ok(OperationStep::new(
            "authorize-sponsor-input",
            OperationSubject::SponsorSigning(Box::new(TargetSponsorSigningSubject {
                finalized_transaction: sent,
                sponsor_input_index: input,
                sponsor_outpoint: WireOutpoint {
                    txid: printed_txid(&sponsor.outpoint.txid()),
                    vout: sponsor.outpoint.index(),
                },
                sighash_profile: WireSighashProfile::AllInputsAllOutputs,
            })),
        ))
    }

    /// Pass two: replay the adapter's own witness into the SAME
    /// finalization the request was formed against.
    fn settle_signature(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<Vec<u8>, SponsoredPrivateRefusal> {
        let mut round = self
            .record
            .round
            .clone()
            .ok_or(SponsoredPrivateRefusal::ControlNotStaged)?;
        let finalization = self
            .staged
            .as_ref()
            .ok_or(SponsoredPrivateRefusal::ControlNotStaged)?;
        let echoed = response
            .signature_bound_to
            .clone()
            .ok_or(SponsoredPrivateRefusal::SponsorSignatureUnusable)?;
        if response.sponsor_witness.is_empty() {
            return Err(SponsoredPrivateRefusal::SponsorSignatureUnusable);
        }
        // The adapter must have signed the bytes it was handed. A signer
        // that echoed something else authorized a different transaction,
        // and the binding check downstream would refuse it -- attributed
        // to the candidate rather than to the signer.
        round.echo_matches_what_was_sent = echoed == round.sent;
        round.witness_items = response.sponsor_witness.len();
        if !round.echo_matches_what_was_sent {
            self.record.round = Some(round);
            return Err(SponsoredPrivateRefusal::SponsorSignatureUnusable);
        }

        let replaying = StagedEnvelope {
            offer: self.offer()?,
            answers: Answers::Replaying(BTreeMap::from([(
                round.input,
                SponsorSignature::new(echoed, response.sponsor_witness.clone()),
            )])),
        };
        let built = assemble_control(finalization, self.genesis_block_hash, Some(&replaying))
            .map_err(SponsoredPrivateRefusal::Private)?;

        self.record.round = Some(round);
        self.census = Some(built.census);
        self.spent_owner_bytes.clone_from(&built.spent_owner_bytes);
        Ok(built.transaction.encode())
    }

    /// Record what the target did with the sponsored control.
    fn settle_control(&mut self, response: &NativeOperationResponse) {
        self.record.observed_layer = Some(response.observed_layer);
        self.record
            .observed_detail
            .clone_from(&response.observed_detail);
        self.record
            .accepted_txid
            .clone_from(&response.accepted_txid);
        self.record.target_weight = response.resources.transaction_weight;

        if response.observed_layer != ObservedOutcomeLayer::Accepted {
            return;
        }
        let (Some(readback), Some(submitted), Some(census)) = (
            response.mined_readback.as_ref(),
            self.submitted.as_ref(),
            self.census.as_ref(),
        ) else {
            return;
        };

        let readback_matches_submission = readback.raw_transaction == *submitted;
        let owner_signature_verified = census
            .signing_inputs()
            .first()
            .map(|input| candidate_owner_message(census, input, WitnessVectorTreatment::BothGrown))
            .zip(self.spent_owner_bytes.as_ref())
            .is_some_and(|(message, owner)| {
                verify_readback_signature(&readback.raw_transaction, &message, owner)
            });

        // The sponsor's change, found in the MINED bytes rather than
        // assumed from what was sent: a reserve-asset output whose value
        // is a commitment.
        let sponsor_change_located = self
            .reserve
            .is_some_and(|reserve| contains_run(&readback.raw_transaction, reserve.internal()));

        self.record.reverification = Some(SponsoredPrivateReverification {
            accepted_txid: readback.transaction_id.clone(),
            readback_matches_submission,
            owner_signature_verified,
            sponsor_change_located,
        });
    }
}

impl TargetOperationPlanner for SponsoredPrivatePlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        if let Some((_case, response)) = previous {
            match self.stage {
                Stage::Issue => {
                    let Some(printed) = response.issued_asset.clone() else {
                        return Err(self.refuse(SponsoredPrivateRefusal::Private(
                            PrivateRestartRefusal::IssuanceNamedNoAsset,
                        )));
                    };
                    self.settle_asset(&printed);
                    self.stage = Stage::FundSponsorSeed;
                }
                Stage::FundSponsorSeed => {
                    if let Err(refusal) = self.settle_sponsor_seed(response) {
                        return Err(self.refuse(refusal));
                    }
                    // The committed stage exists to turn the seed into a
                    // blinded coin. The explicit shape spends the seed
                    // itself, so it has nothing to do there and skipping
                    // it is what "explicit sponsor coin" means in the
                    // step plan rather than only in the record.
                    self.stage = if self.shape.commits_the_sponsor_value() {
                        Stage::FundSponsor
                    } else {
                        Stage::FundReceipts
                    };
                }
                Stage::FundSponsor => {
                    if let Err(refusal) = self.settle_sponsor(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::FundReceipts;
                }
                Stage::FundReceipts => {
                    if let Err(refusal) = self.settle_receipts(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::SignSponsor;
                }
                Stage::SignSponsor => {
                    match self.settle_signature(response) {
                        Ok(bytes) => self.submitted = Some(bytes),
                        Err(refusal) => return Err(self.refuse(refusal)),
                    }
                    self.stage = Stage::Submit;
                }
                Stage::Submit => {
                    self.settle_control(response);
                    self.stage = Stage::Done;
                }
                Stage::Done => {}
            }
        }

        match self.stage {
            Stage::Issue => Ok(Some(issue_step())),
            Stage::FundSponsorSeed => Ok(Some(OperationStep::new(
                "fund-sponsor-region",
                OperationSubject::SponsorFunding(Box::new(TargetSponsorFundingSubject {
                    sponsor_outputs: 1,
                    amount_per_sponsor_output: self.shape.seed_amount(),
                })),
            ))),
            Stage::FundReceipts => {
                let Some(linked) = self.linked.as_ref() else {
                    return Err(self.refuse(SponsoredPrivateRefusal::ControlNotStaged));
                };
                let Some(printed) = self.record.issued_asset.clone() else {
                    return Err(self.refuse(SponsoredPrivateRefusal::Private(
                        PrivateRestartRefusal::IssuanceNamedNoAsset,
                    )));
                };
                Ok(Some(confidential_funding_step(linked, printed)))
            }
            Stage::FundSponsor => match self.sponsor_funding_step() {
                Ok(step) => Ok(Some(step)),
                Err(refusal) => Err(self.refuse(refusal)),
            },
            Stage::SignSponsor => match self.sign_step() {
                Ok(step) => Ok(Some(step)),
                Err(refusal) => Err(self.refuse(refusal)),
            },
            Stage::Submit => {
                let Some(bytes) = self.submitted.clone() else {
                    return Err(self.refuse(SponsoredPrivateRefusal::ControlNotStaged));
                };
                self.record.submitted_bytes = bytes.len();
                Ok(Some(OperationStep::new(
                    self.shape.case_name(),
                    OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                        transaction_bytes: bytes,
                    })),
                )))
            }
            Stage::Done => Ok(None),
        }
    }
}

/// The deployment's own sponsor-change program.
fn sponsor_program(linked: &LinkedDeployment) -> Result<Vec<u8>, SponsoredPrivateRefusal> {
    let symbols = linked.abi.symbols();
    let version = symbols.sponsor_change_version();
    let payload = symbols.sponsor_change_program();
    transaction::taproot::witness_program_script(
        &reviewed_target().map_err(|_| {
            SponsoredPrivateRefusal::Private(PrivateRestartRefusal::SubstrateUnavailable)
        })?,
        version,
        payload,
    )
    .map_err(|_| SponsoredPrivateRefusal::Private(PrivateRestartRefusal::SubstrateUnavailable))
}

/// One transaction identity, in the spelling a target prints.
fn printed_txid(txid: &Txid) -> String {
    let mut bytes = *txid.internal();
    bytes.reverse();
    hex(&bytes)
}

/// Whether `needle` occurs in `haystack` as a contiguous run.
fn contains_run(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

/// Hex, for the transcript.
fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(text, "{byte:02x}");
    }
    text
}

/// One sponsored private run's transcript.
///
/// Lines rather than a structure, on the pattern every live lane here
/// sets. Every line is a fact the run observed or a value it computed,
/// and no line is a verdict about whether the run went well.
#[must_use]
pub fn render_sponsored_private(record: &SponsoredPrivateRecord) -> String {
    use std::fmt::Write as _;

    let mut out = String::new();
    let _ = writeln!(out, "case sponsored-private-with-change");
    if let Some(asset) = record.issued_asset.as_ref() {
        let _ = writeln!(out, "issued_asset {asset}");
    }
    let _ = writeln!(out, "predecessor_coins {}", record.coins.len());
    if let Some(solve) = record.solve.as_ref() {
        for check in SponsoredSolveCheck::ALL {
            let _ = writeln!(out, "sponsor_check {} {}", check.name(), solve.holds(check));
        }
    }
    if let Some(txid) = record.sponsor_funding_txid.as_ref() {
        let _ = writeln!(out, "sponsor_funding_txid {txid}");
    }
    if let Some(round) = record.round.as_ref() {
        let _ = writeln!(out, "sponsor_input_position {}", round.input());
        let _ = writeln!(
            out,
            "sponsor_echo_matches_what_was_sent {}",
            round.echo_matches_what_was_sent(),
        );
        let _ = writeln!(out, "sponsor_witness_items {}", round.witness_items());
    }
    let _ = writeln!(out, "submitted_bytes {}", record.submitted_bytes);
    if let Some(layer) = record.observed_layer {
        let _ = writeln!(out, "observed_layer {layer:?}");
    }
    if let Some(detail) = record.observed_detail.as_ref() {
        let _ = writeln!(out, "observed_detail {detail}");
    }
    if let Some(weight) = record.target_weight {
        let _ = writeln!(out, "target_weight {weight}");
    }
    if let Some(check) = record.reverification.as_ref() {
        let _ = writeln!(out, "accepted_txid {}", check.accepted_txid());
        let _ = writeln!(
            out,
            "readback_matches_submission {}",
            check.readback_matches_submission(),
        );
        let _ = writeln!(
            out,
            "owner_signature_verified {}",
            check.owner_signature_verified(),
        );
        let _ = writeln!(
            out,
            "sponsor_change_located {}",
            check.sponsor_change_located(),
        );
    }
    if let Some(refusal) = record.refusal.as_ref() {
        let _ = writeln!(out, "refusal {refusal:?}");
    }
    out
}
