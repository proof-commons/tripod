//! The one mapping from this package's own workload onto the executor's
//! step records (Guide-12 §16.2).
//!
//! [`crate::plan::RequiredTargetWork`] says what a target must do before any coverage
//! row can be discharged, in this package's vocabulary. The executor
//! boundary says what a target *can* be asked to do, in a target-generic
//! one: issue an asset, pay outputs to a witness program, submit a
//! transaction. This module is the single place the two meet, and
//! nothing else in the crate speaks the executor's vocabulary.
//!
//! # Nothing here tells the executor what to expect
//!
//! An [`OperationStep`] carries an identity and a subject and has no
//! member for an expected layer, an expected identity, or a class. The
//! planner therefore cannot hand the target the answer it is about to be
//! graded against, and the type is what says so rather than a convention
//! `(´[PLAN-rule:guide11-exec:request-subject]´)`.
//!
//! # Why the plan is consulted between steps
//!
//! A transaction cannot be built until the coins it spends exist, and
//! those coins are created by an earlier step of the same run against the
//! same chain. So the ceremony's answers are inputs to everything after
//! them: the asset the target issued decides which bundle is linked, the
//! bundle decides the constructor's output program, and the outpoints the
//! target reported decide what each vector spends.

use std::collections::{BTreeMap, BTreeSet};

use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId, OperationStepKind,
    OperationSubject, TargetFundingSubject, TargetSponsorFundingSubject,
    TargetSponsorSigningSubject, TargetSubmissionSubject, WireOutpoint, WireSighashProfile,
};
use transaction::{FundingCeremonyStep, Outpoint, SponsorSignature, Txid};

use crate::bundle::{FixtureBundle, ceremony_bundle};
use crate::divergence::{AmountBeyondTargetBound, target_amount_standing};
use crate::error::VectorError;
use crate::fixture::{CompactAshSemanticCase, positive_semantic_census};
use crate::materialize::{
    AshFunding, MaterializedTargetVector, SponsorCoin, SponsorSigningTask, TargetVectorId,
    has_candidate_program, materialize, materialize_sponsored, sponsor_signing_requests, vector_id,
};
use crate::mutation::{MutatedVector, NegativeMutation, apply};

/// What each ceremony output carries while the asset is being issued.
///
/// The issuance step has to pay *somewhere*, and it pays before the
/// bundle that decides the real program can be linked at all. The
/// output it creates is deliberately not used by anything: it exists
/// because an issuance attaches to a transaction that has outputs, and
/// the asset identity is what the step is for.
const ISSUANCE_PROBE_AMOUNT: u64 = 1;

/// What each sponsor coin holds, in the reserve asset's smallest unit.
///
/// # Why an exact figure and not a minimum
///
/// It is the fee a sponsored vector declares. The reserve asset has to
/// balance across the transaction and the fee output is the only place
/// it goes, so the coin and the fee are one number; a coin holding more
/// would leave the reserve unbalanced and the target would refuse the
/// transaction for a reason that has nothing to do with what the row is
/// about.
///
/// The size is chosen to sit comfortably above any relay threshold a
/// development deployment sets, so that a submission is answered on its
/// merits rather than for a fee nobody meant to make marginal. It is
/// not a semantic value and no fixture states it.
const SPONSOR_COIN_UNITS: u64 = 100_000;

/// Why a plan could not state its next step.
///
/// The executor boundary's [`PlanRefused`] is a marker: a plan that
/// cannot continue knows why in its own vocabulary and that package
/// holds none for it. This is that vocabulary, read off the planner
/// after the run is refused.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PlanRefusal {
    /// The issuance step reached no target verdict, so no asset exists.
    IssuanceDidNotHappen(ObservedOutcomeLayer),
    /// The target accepted the issuance but named no asset.
    IssuanceNamedNoAsset,
    /// The asset the target chose is not 32 bytes of hex.
    IssuedAssetUnreadable(String),
    /// A funding step reached no target verdict.
    FundingDidNotHappen(ObservedOutcomeLayer),
    /// The target created a coin its own bound says it cannot state.
    ///
    /// The opposite surprise to a funding failure, and refused just as
    /// hard. A row is asked for only because the reviewed bound says
    /// the target cannot hold that amount; a target that holds it
    /// anyway has falsified the reviewed fact the classification was
    /// derived from, and continuing would build a run on a reading of
    /// the target that is known to be wrong.
    AmountBeyondBoundWasFunded {
        /// The vector the coin was cut for.
        vector: TargetVectorId,
        /// The amount the step asked the target to state.
        stated: u64,
        /// The bound the reviewed target facts publish.
        bound: u64,
    },
    /// A funding step created a different number of outputs than asked.
    FundingCardinalityWrong {
        /// How many outputs the step asked for.
        wanted: usize,
        /// How many it reported.
        reported: usize,
    },
    /// A funded output's outpoint is not one the target admits.
    FundedOutpointUnreadable(String),
    /// The target stored a script other than the program asked for.
    ///
    /// The whole content of the derive step: the program was computed
    /// here from this bundle's own tree, and this is the target saying
    /// what it actually created.
    ProgramMismatch {
        /// What the planner asked the target to pay to.
        asked: String,
        /// What the target reported storing.
        stored: String,
    },
    /// The sponsor-funding step reached no target verdict.
    SponsorFundingDidNotHappen(ObservedOutcomeLayer),
    /// The executor created a different number of sponsor coins than
    /// asked.
    SponsorCardinalityWrong {
        /// How many coins the step asked for.
        wanted: usize,
        /// How many it reported.
        reported: usize,
    },
    /// A sponsor coin's asset is not 32 bytes of hex.
    SponsorAssetUnreadable(String),
    /// The executor paid its sponsor coins in more than one asset.
    ///
    /// The reserve is one identity that gets linked into every leaf, so
    /// coins carrying two of them cannot all be spent by one bundle.
    /// Picking whichever came first would link against an asset half
    /// the coins do not hold.
    SponsorAssetsDisagree,
    /// A sponsor coin's outpoint is not one the target admits.
    SponsorOutpointUnreadable(String),
    /// A sponsor coin's program is not readable as bytes.
    SponsorProgramUnreadable(String),
    /// An authorization step reached no target verdict.
    AuthorizationDidNotHappen(ObservedOutcomeLayer),
    /// The executor authorized bytes other than the ones it was handed.
    ///
    /// The caller's own comparison of the echo, made before the
    /// authorization is put anywhere near a construction. The builder
    /// makes it again from its own side; this one catches it at the
    /// step that produced it, so the transcript names the exchange that
    /// went wrong rather than the construction that later refused.
    AuthorizationBoundToOtherBytes {
        /// The vector whose construction asked for it.
        vector: TargetVectorId,
        /// How many bytes were handed over.
        sent: usize,
        /// How many the executor said it authorized.
        echoed: usize,
    },
    /// An accepted authorization carried no witness stack.
    AuthorizationCarriedNoWitness(TargetVectorId),
    /// The target weighed a submission differently than the ABI did.
    ///
    /// The one resource dimension both sides state for the same bytes,
    /// so it is the one place §20.5's comparison can actually be made.
    /// A disagreement means the ABI's weight arithmetic and the
    /// target's do not describe the same transaction, which falsifies
    /// the prediction every other candidate measurement rests on.
    WeightObservationDisagrees {
        /// The vector whose submission was weighed.
        vector: TargetVectorId,
        /// What the transaction layer settled before submission.
        predicted: u64,
        /// What the target reported for the same bytes.
        observed: u64,
    },
    /// The ceremony-bound bundle could not be built or materialized.
    Bundle(Box<VectorError>),
}

/// What one submitted vector produced.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubmissionOutcome {
    vector: TargetVectorId,
    layer: ObservedOutcomeLayer,
    detail: Option<String>,
    accepted_txid: Option<String>,
    bytes: Vec<u8>,
    predicted_weight: u64,
    observed_weight: Option<u64>,
    witness_bytes: u64,
    virtual_size: u64,
}

impl SubmissionOutcome {
    /// The vector that was submitted.
    #[must_use]
    pub const fn vector(&self) -> TargetVectorId {
        self.vector
    }

    /// Where the target put it.
    ///
    /// §1.4 and §1.5 both live here: a consensus refusal, a script-path
    /// refusal, a relay refusal, and an acceptance are four different
    /// facts, and the two non-verdicts are not target facts at all.
    #[must_use]
    pub const fn layer(&self) -> ObservedOutcomeLayer {
        self.layer
    }

    /// What the target or the adapter said, verbatim and unmapped.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    /// The identity the target gave the transaction, where it took one.
    #[must_use]
    pub fn accepted_txid(&self) -> Option<&str> {
        self.accepted_txid.as_deref()
    }

    /// The exact bytes that were submitted.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The weight the ABI computed before anything was submitted.
    ///
    /// A prediction in the sense §20.5 means: settled by the
    /// transaction layer from the bytes it built, with no target
    /// involved.
    #[must_use]
    pub const fn predicted_weight(&self) -> u64 {
        self.predicted_weight
    }

    /// The weight the target reported for the same bytes.
    ///
    /// Absent where the executor made no such observation, which is not
    /// the same fact as a weight of zero. An executor that reports none
    /// leaves the comparison unmade rather than passing it by default.
    #[must_use]
    pub const fn observed_weight(&self) -> Option<u64> {
        self.observed_weight
    }

    /// The serialized witness bytes the ABI settled.
    ///
    /// One of §20.3's measures, and unlike the weight it has no
    /// observed counterpart: the target reports a weight and never a
    /// witness subtotal, so this figure stands alone rather than
    /// entering the comparison.
    #[must_use]
    pub const fn witness_bytes(&self) -> u64 {
        self.witness_bytes
    }

    /// The virtual size the ABI settled.
    #[must_use]
    pub const fn virtual_size(&self) -> u64 {
        self.virtual_size
    }

    /// Whether prediction and observation agree, where both exist.
    ///
    /// `None` when nothing was observed. §20.5 makes a mismatch fail the
    /// resource report even for a transaction the target accepted, so
    /// the three cases stay distinct here rather than collapsing an
    /// unmade comparison into a passing one.
    #[must_use]
    pub const fn weight_agrees(&self) -> Option<bool> {
        match self.observed_weight {
            Some(observed) => Some(observed == self.predicted_weight),
            None => None,
        }
    }
}

/// One target-bound divergence, as the target answered it.
///
/// # Why the answer is recorded and not just the classification
///
/// The classification is a comparison of two numbers this workspace
/// holds, and on its own it establishes nothing about a running node.
/// The divergence becomes a fact about the target when the target is
/// asked to create the coin and does not. Both halves are therefore
/// kept: what was derived, and what came back.
///
/// The layer is deliberately not constrained to one value. The reviewed
/// bound says the amount cannot be stated; which of the target's gates
/// says so first is the target's business, and on this path an
/// adapter's own reserve arithmetic can answer ahead of every one of
/// them. What is required is that the step did not succeed, and the
/// layer and detail record which way it failed
/// `(´[PLAN-rule:guide12-exec:failure-layers]´)`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservedDivergence {
    vector: TargetVectorId,
    beyond: AmountBeyondTargetBound,
    layer: ObservedOutcomeLayer,
    detail: Option<String>,
}

impl ObservedDivergence {
    /// The vector whose funding the target's bound forbids.
    #[must_use]
    pub const fn vector(&self) -> TargetVectorId {
        self.vector
    }

    /// The amount, its place in the row, and the bound it overshoots.
    #[must_use]
    pub const fn beyond(&self) -> AmountBeyondTargetBound {
        self.beyond
    }

    /// Where the target put the step that was asked anyway.
    #[must_use]
    pub const fn layer(&self) -> ObservedOutcomeLayer {
        self.layer
    }

    /// What the target or the adapter said, verbatim and unmapped.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

/// What one submitted mutation produced.
///
/// # Why this is not a [`SubmissionOutcome`]
///
/// A submission outcome answers a positive row, and
/// `CompactAshEvidencePlan::discharge` reads those to move positive
/// coverage. A mutation answers a negative requirement and must never
/// be able to move a positive row by being mistaken for one, so it is a
/// different type in a different list rather than a flag on the same
/// one.
///
/// The expected boundary is not stored. It is a function of the
/// mutation, which is stored, and a copy taken at submission time could
/// disagree with the matrix later — which is exactly the drift the
/// lookup exists to prevent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MutantOutcome {
    origin: TargetVectorId,
    mutation: NegativeMutation,
    layer: ObservedOutcomeLayer,
    detail: Option<String>,
    accepted_txid: Option<String>,
    bytes: Vec<u8>,
}

impl MutantOutcome {
    /// The accepted vector this mutation was made from.
    #[must_use]
    pub const fn origin(&self) -> TargetVectorId {
        self.origin
    }

    /// What was changed.
    #[must_use]
    pub const fn mutation(&self) -> NegativeMutation {
        self.mutation
    }

    /// Where the target put the mutated transaction.
    #[must_use]
    pub const fn layer(&self) -> ObservedOutcomeLayer {
        self.layer
    }

    /// What the target or the adapter said, verbatim and unmapped.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    /// The identity the target gave the transaction, where a mutation was
    /// accepted rather than refused.
    #[must_use]
    pub fn accepted_txid(&self) -> Option<&str> {
        self.accepted_txid.as_deref()
    }

    /// The exact bytes that were submitted.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(test)]
impl MutantOutcome {
    /// One mutation outcome, stated directly, for the crate's tests.
    pub(crate) fn for_tests(
        origin: TargetVectorId,
        mutation: NegativeMutation,
        layer: ObservedOutcomeLayer,
        bytes: Vec<u8>,
    ) -> Self {
        Self {
            origin,
            mutation,
            layer,
            detail: None,
            accepted_txid: None,
            bytes,
        }
    }
}

/// Everything one operation run established.
///
/// Built only by [`CompactAshOperationPlanner`], and only from answers
/// the executor recorded. Nothing here is an intention.
/// Construction is crate-private so callers cannot mint a record for a run
/// that never happened.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationTranscript {
    issued_asset: Option<[u8; 32]>,
    reserve_asset: Option<[u8; 32]>,
    constructor_program: Option<Vec<u8>>,
    funded: BTreeMap<TargetVectorId, Vec<Outpoint>>,
    /// The mutation subject's own extra replicas, one per submittable
    /// arm, keyed by replica number.
    ///
    /// Replica zero is the control's, and lands in [`Self::funded`]
    /// exactly as every other row's single funding does. Every other
    /// vector in the census is funded once and needs no entry here at
    /// all. The subject alone is funded once per arm this run will
    /// actually submit, so that an accepted mutation spends only its
    /// own coins and abandons nothing behind it; this is the record of
    /// which replica became which arm's own funding, read by
    /// `stage_mutants` in the same deterministic plan order the
    /// replicas were scheduled in.
    arm_funded: BTreeMap<usize, Vec<Outpoint>>,
    coins: BTreeMap<Outpoint, u64>,
    submissions: Vec<SubmissionOutcome>,
    mutants: Vec<MutantOutcome>,
    mutation_subject: Option<TargetVectorId>,
    divergences: Vec<ObservedDivergence>,
    refusal: Option<PlanRefusal>,
}

impl OperationTranscript {
    /// An empty transcript for the planner to populate from recorded answers.
    pub(crate) const fn empty() -> Self {
        Self {
            issued_asset: None,
            reserve_asset: None,
            constructor_program: None,
            funded: BTreeMap::new(),
            arm_funded: BTreeMap::new(),
            coins: BTreeMap::new(),
            submissions: Vec::new(),
            mutants: Vec::new(),
            mutation_subject: None,
            divergences: Vec::new(),
            refusal: None,
        }
    }

    /// The disposable asset the target issued, where it issued one.
    #[must_use]
    pub const fn issued_asset(&self) -> Option<[u8; 32]> {
        self.issued_asset
    }

    /// The asset the executor paid its sponsor coins in.
    ///
    /// Observed rather than chosen, like the issued asset, and for a
    /// reason of the same kind: which asset a development network
    /// treats as its reserve is the network's own fact. The bundle this
    /// run executed is linked against it.
    #[must_use]
    pub const fn reserve_asset(&self) -> Option<[u8; 32]> {
        self.reserve_asset
    }

    /// The constructor output program the ceremony funded.
    ///
    /// This is the real pin: the taproot output key of the
    /// ceremony-bound bundle's own committed tree, which the target
    /// created outputs at.
    #[must_use]
    pub fn constructor_program(&self) -> Option<&[u8]> {
        self.constructor_program.as_deref()
    }

    /// The coins the ceremony created for each vector.
    #[must_use]
    pub const fn funded(&self) -> &BTreeMap<TargetVectorId, Vec<Outpoint>> {
        &self.funded
    }

    /// Which replica of the mutation subject's coins funded each
    /// submittable arm, by replica number.
    ///
    /// Empty for a run that reserved no subject. Replica zero is the
    /// control's own and is not here, only in [`Self::funded`]; every
    /// other replica is numbered by counting forward through
    /// [`crate::mutation::NegativeMutation::ALL`] in order and
    /// skipping every arm this run does not submit, so replica one is
    /// the first submittable arm in that order, replica two the
    /// second, and so on.
    #[must_use]
    pub const fn arm_funded(&self) -> &BTreeMap<usize, Vec<Outpoint>> {
        &self.arm_funded
    }

    /// What the target said it put in each coin it created.
    ///
    /// The amount is read back off the target's own answer rather than
    /// carried over from the request. They should be the same number and
    /// this is the only record of the one the target actually reported,
    /// which is what a later comparison has to be made against if it is
    /// to be a comparison at all.
    #[must_use]
    pub const fn coins(&self) -> &BTreeMap<Outpoint, u64> {
        &self.coins
    }

    /// What each submitted vector produced, in submission order.
    #[must_use]
    pub fn submissions(&self) -> &[SubmissionOutcome] {
        &self.submissions
    }

    /// What each submitted mutation produced, in submission order.
    ///
    /// Empty for a run that accepted nothing: a mutation is made from an
    /// accepted transaction, so a run with no acceptance has nothing to
    /// mutate and says so by carrying no negative results rather than by
    /// carrying unattributable ones.
    #[must_use]
    pub fn mutants(&self) -> &[MutantOutcome] {
        &self.mutants
    }

    /// The vector this run held back to mutate, if it chose one.
    ///
    /// Its own submission is the control: every mutation was offered
    /// before it, against coins nothing had spent. A control the target
    /// refused invalidates those mutations, and naming the subject is
    /// what lets a reader check which submission the control was.
    #[must_use]
    pub const fn mutation_subject(&self) -> Option<TargetVectorId> {
        self.mutation_subject
    }

    /// Every row the target's own bound refused to fund.
    ///
    /// Empty for a run whose census the target admits whole. A
    /// non-empty list is evidence of a divergence between the
    /// protocol's amount domain and this target's, and is never a
    /// submission result: nothing was built, so nothing was judged.
    #[must_use]
    pub fn divergences(&self) -> &[ObservedDivergence] {
        &self.divergences
    }

    /// Why the plan stopped, where it stopped early.
    #[must_use]
    pub const fn refusal(&self) -> Option<&PlanRefusal> {
        self.refusal.as_ref()
    }

    /// A transcript assembled directly, for the crate's own tests.
    ///
    /// Not public, and for the reason the executor package gives for its
    /// own such constructor: a transcript is what a run established, and
    /// a caller able to state one without a run could hand the report
    /// validation a run that never happened. The crate's own tests need
    /// exactly that — a pair of records that do *not* correspond is the
    /// thing report validation exists to refuse, and no honest run
    /// produces one to test against.
    #[cfg(test)]
    pub(crate) fn for_tests(parts: TranscriptParts) -> Self {
        Self {
            issued_asset: parts.issued_asset,
            reserve_asset: parts.reserve_asset,
            constructor_program: parts.constructor_program,
            submissions: parts.submissions,
            mutants: parts.mutants,
            refusal: parts.refusal,
            ..Self::empty()
        }
    }
}

/// The parts a test-assembled [`OperationTranscript`] is stated from.
///
/// A struct rather than a long argument list, so that a test states
/// which field it is exercising by name.
#[cfg(test)]
pub(crate) struct TranscriptParts {
    pub(crate) issued_asset: Option<[u8; 32]>,
    pub(crate) reserve_asset: Option<[u8; 32]>,
    pub(crate) constructor_program: Option<Vec<u8>>,
    pub(crate) submissions: Vec<SubmissionOutcome>,
    pub(crate) mutants: Vec<MutantOutcome>,
    pub(crate) refusal: Option<PlanRefusal>,
}

#[cfg(test)]
impl SubmissionOutcome {
    /// One submission outcome, stated directly, for the crate's tests.
    pub(crate) fn for_tests(
        vector: TargetVectorId,
        layer: ObservedOutcomeLayer,
        accepted_txid: Option<&str>,
        bytes: Vec<u8>,
        predicted_weight: u64,
        observed_weight: Option<u64>,
    ) -> Self {
        Self {
            vector,
            layer,
            detail: None,
            accepted_txid: accepted_txid.map(str::to_owned),
            bytes,
            predicted_weight,
            observed_weight,
            witness_bytes: 0,
            virtual_size: 0,
        }
    }
}

/// What the ceremony is asking a funding step for.
///
/// # Why an expectation exists here and nowhere near the step
///
/// The [`OperationStep`] this produces carries an identity and a
/// subject, exactly as before: no expected layer crosses the boundary,
/// and the executor is told nothing about which answer would be the
/// interesting one. The expectation is the planner's own reading of
/// what it is about to ask, kept on this side so that an answer can be
/// compared against something rather than merely recorded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FundingExpectation {
    /// A coin the vector will spend.
    Coin,
    /// A coin the reviewed target bound says the target cannot state.
    ///
    /// Asked anyway, and asked exactly once per divergent row. The
    /// classification is arithmetic over two numbers this workspace
    /// holds; without putting it to the target there is no observation
    /// behind the claim, only a calculation asserting one.
    BeyondBound(AmountBeyondTargetBound),
}

/// One funding request the schedule holds.
#[derive(Clone, Copy, Debug)]
struct PlannedFunding {
    vector: TargetVectorId,
    member: usize,
    amount: u64,
    expectation: FundingExpectation,
    /// Which copy of this vector's input set this request funds.
    ///
    /// Zero for every row this run funds once, which is every row but
    /// the mutation subject. The subject earns one replica per arm
    /// this run will actually submit, plus its own replica zero, so
    /// that every arm spends coins nothing else has touched.
    /// `settle_funding` reads this to decide where an answer's
    /// outpoint belongs, and `stage_mutants` reads the transcript's
    /// own record of it to build each arm's own subject.
    replica: usize,
}

/// Where the planner is.
#[derive(Clone, Debug)]
enum Stage {
    /// Ask the target to issue the disposable asset.
    Issue,
    /// Ask the executor for the sponsor coins, and learn the reserve.
    ///
    /// # Why this comes before the bundle is linked
    ///
    /// The reserve asset is substituted into all twelve leaves, so the
    /// committed tree — and the program every ASH input pays to — is a
    /// function of it just as it is of the closed asset. A bundle
    /// cannot be linked until both are known, and neither is a value
    /// anyone chooses: the closed asset is what the issuance produced,
    /// and the reserve is what the executor reported paying its sponsor
    /// coins in.
    ///
    /// That is why a run with no sponsored row still takes this step.
    /// It is not the sponsored rows that need it; it is the link.
    FundSponsor,
    /// Ask it to create one output at the derived constructor program.
    Derive,
    /// Work through the per-input funding schedule.
    Fund(usize),
    /// Work through the authorizations the sponsored rows need.
    Sign(usize),
    /// Work through the materialized vectors.
    Submit(usize),
    /// Work through the mutations of the reserved subject.
    ///
    /// Before the subject itself, so every mutation is offered while the
    /// coins it spends are still unspent. A mutation submitted after its
    /// own subject is refused for double-spending and establishes
    /// nothing about the mutation.
    SubmitMutant(usize),
    /// Submit the reserved subject, un-mutated, as the control.
    SubmitControl,
    /// Nothing left to ask.
    Done,
}

/// One sponsored row, waiting for the authorizations it needs.
#[derive(Clone, Debug)]
struct PendingSponsored {
    case: CompactAshSemanticCase,
    funding: AshFunding,
    coins: Vec<SponsorCoin>,
    tasks: Vec<SponsorSigningTask>,
    signatures: BTreeMap<Outpoint, SponsorSignature>,
}

/// The compact-ASH operation plan, as the executor consults it.
///
/// # How the thirteen work items map
///
/// The plan's [`RequiredTargetWork`] census is four ceremony steps, one
/// per sponsorless vector after that, and — for a row the reviewed
/// target bound forbids — a divergence probe in the submission's place.
/// The submissions map one-to-one onto submission steps. The ceremony
/// does not, and the mismatch is deliberate rather than an
/// approximation:
///
/// - `IssueDisposableTestAsset` is one funding step that issues.
/// - `DeriveConstructorOutputProgram` is one funding step at the program
///   this planner derived, whose answer is the target saying what it
///   stored there.
/// - `FundEachAshInput` is *many* funding steps, which is what the
///   census entry says: once per ASH input the fixture needs. A single
///   step could not express it, because a funding step states one amount
///   and the fixtures state different amounts per input. A row the
///   reviewed target bound cannot state gets one step instead of one
///   per input, and that step's whole purpose is the refusal it
///   collects `(´[PLAN-rule:guide12-exec:failure-layers]´)`.
/// - `RecordPublicView` is no step at all. Recording the outpoints and
///   their public fields is what this planner does with what the target
///   already reported; asking the target to do it would be asking it to
///   observe itself.
///
/// [`RequiredTargetWork`]: crate::plan::RequiredTargetWork
pub struct CompactAshOperationPlanner {
    stage: Stage,
    cases: Vec<CompactAshSemanticCase>,
    schedule: Vec<PlannedFunding>,
    unfundable: BTreeSet<TargetVectorId>,
    bundle: Option<FixtureBundle>,
    probe_program: Vec<u8>,
    program: Vec<u8>,
    sponsor_coins: Vec<SponsorCoin>,
    pending: Vec<PendingSponsored>,
    /// The flattened authorization schedule: which pending row, and
    /// which of its tasks. Flattened so a stage is one index, exactly as
    /// the funding schedule is.
    sign_schedule: Vec<(usize, usize)>,
    vectors: Vec<MaterializedTargetVector>,
    /// The mutated transactions this run will submit, settled once the
    /// positive submissions have said which vector was accepted.
    mutants: Vec<MutatedVector>,
    /// The vector this run has chosen to hold back and mutate, fixed
    /// before funding is scheduled.
    ///
    /// Read here rather than recomputed later, because the funding
    /// schedule needs to know which row earns extra replicas before
    /// any of them is asked for. [`Self::choose_mutation_subject`]
    /// reads it back once the vector actually exists, to move it to
    /// the end of the submission order and settle
    /// [`OperationTranscript::mutation_subject`].
    subject: Option<TargetVectorId>,
    transcript: OperationTranscript,
}

/// The mutations this run will actually offer, in plan order.
///
/// [`NegativeMutation::ALL`], filtered to the arms whose class both
/// names a boundary and does not expect it before the target — the
/// same two checks [`CompactAshOperationPlanner::stage_mutants`] makes
/// per arm, so a class this workspace's own matrix moves cannot leave
/// the funding schedule and the staging loop counting the submittable
/// set two different ways. `settle_sponsor_funding` reads only this
/// list's length, to know how many replicas the subject earns;
/// `stage_mutants` recomputes the same two checks itself, arm by arm,
/// so it can say *why* an arm that fails them was not submitted.
fn submittable_mutations() -> Vec<NegativeMutation> {
    NegativeMutation::ALL
        .iter()
        .copied()
        .filter(|mutation| {
            crate::abi_validation::precedes_the_target(*mutation).is_ok_and(|withheld| !withheld)
        })
        .collect()
}

impl CompactAshOperationPlanner {
    /// A planner for the sponsorless positive census.
    ///
    /// # Errors
    ///
    /// Any refusal from building the positive semantic census, which is
    /// a construction failure and never a target verdict.
    pub fn new() -> Result<Self, VectorError> {
        // Every row this candidate has a program for, which now
        // includes the sponsored ones: what used to exclude them was an
        // authorization nothing could produce, and the executor
        // boundary produces one.
        let cases: Vec<CompactAshSemanticCase> = positive_semantic_census()?
            .into_iter()
            .filter(has_candidate_program)
            .collect();
        let canonical = crate::bundle::fixture_bundle()?;
        let probe_program = canonical
            .pin()
            .output_script(canonical.target())
            .map_err(|cause| VectorError::TargetMaterializationFailed {
                vector: vector_id(&cases[0]),
                cause,
            })?;
        Ok(Self {
            stage: Stage::Issue,
            cases,
            schedule: Vec::new(),
            unfundable: BTreeSet::new(),
            bundle: None,
            probe_program,
            program: Vec::new(),
            sponsor_coins: Vec::new(),
            pending: Vec::new(),
            sign_schedule: Vec::new(),
            vectors: Vec::new(),
            mutants: Vec::new(),
            subject: None,
            transcript: OperationTranscript::empty(),
        })
    }

    /// Everything the run established.
    #[must_use]
    pub const fn transcript(&self) -> &OperationTranscript {
        &self.transcript
    }

    /// The ceremony-bound bundle, once the asset is known.
    #[must_use]
    pub const fn bundle(&self) -> Option<&FixtureBundle> {
        self.bundle.as_ref()
    }

    /// The vectors materialized against the ceremony's own coins.
    #[must_use]
    pub fn vectors(&self) -> &[MaterializedTargetVector] {
        &self.vectors
    }

    fn refuse(&mut self, refusal: PlanRefusal) -> PlanRefused {
        self.transcript.refusal = Some(refusal);
        self.stage = Stage::Done;
        PlanRefused
    }

    /// The issuance step: one output, issuing the disposable asset.
    ///
    /// # Why it pays to the canonical program
    ///
    /// A funding step states a program, and the program the ceremony is
    /// actually for cannot be computed yet: it belongs to a bundle that
    /// is linked against the asset this very step is asking the target
    /// to choose. So the output goes to the canonical pinned program,
    /// which is a well-formed witness program of the right shape and
    /// which nothing in this run ever spends. The step's answer is what
    /// it is for — the identity the target chose — and the output it
    /// happens to create is not used by anything.
    fn issue_step(&self) -> OperationStep {
        OperationStep::new(
            "issue-disposable-test-asset",
            OperationSubject::Funding(Box::new(TargetFundingSubject {
                issue_asset: true,
                // Absent exactly when the step issues: the target has not
                // chosen the identity yet, so nothing here can name it.
                asset: None,
                output_program: self.probe_program.clone(),
                outputs: 1,
                amount_per_output: ISSUANCE_PROBE_AMOUNT,
            })),
        )
    }

    fn derive_step(&self) -> OperationStep {
        OperationStep::new(
            "derive-constructor-output-program",
            OperationSubject::Funding(Box::new(TargetFundingSubject {
                issue_asset: false,
                asset: self.transcript.issued_asset.map(hex_of),
                output_program: self.program.clone(),
                outputs: 1,
                amount_per_output: ISSUANCE_PROBE_AMOUNT,
            })),
        )
    }

    fn fund_step(&self, index: usize) -> Option<OperationStep> {
        let planned = self.schedule.get(index)?;
        // Every row but the mutation subject is funded once, so its
        // ordinal and member already name a step uniquely. The subject's
        // extra replicas share both, and the executor refuses a plan
        // that asks for one step identity twice -- so a non-zero
        // replica earns its own suffix, which doubles as the funding
        // derivation the transcript keeps visible: this step's name
        // says which arm its answer will fund.
        let mut name = format!(
            "fund-ash-input/{}/{}",
            planned.vector.fixture().ordinal(),
            planned.member
        );
        if planned.replica > 0 {
            use std::fmt::Write as _;
            let _ = write!(name, "/arm-{}", planned.replica);
        }
        Some(OperationStep::new(
            &name,
            OperationSubject::Funding(Box::new(TargetFundingSubject {
                issue_asset: false,
                asset: self.transcript.issued_asset.map(hex_of),
                output_program: self.program.clone(),
                outputs: 1,
                amount_per_output: planned.amount,
            })),
        ))
    }

    /// How many sponsor coins this census needs.
    ///
    /// One per sponsor member of every row that has a program, and at
    /// least one whatever the census holds: a run with no sponsored row
    /// still has to learn which asset the executor treats as its
    /// reserve, because the bundle is linked against it either way. That
    /// spare coin is used by nothing, exactly as the issuance step's
    /// output is.
    fn sponsor_coins_wanted(&self) -> usize {
        let needed: usize = self
            .cases
            .iter()
            .map(|case| usize::from(case.sponsor().members()))
            .sum();
        needed.max(1)
    }

    fn sponsor_step(&self) -> OperationStep {
        OperationStep::new(
            "fund-sponsor-region",
            OperationSubject::SponsorFunding(Box::new(TargetSponsorFundingSubject {
                sponsor_outputs: u8::try_from(self.sponsor_coins_wanted()).unwrap_or(u8::MAX),
                amount_per_sponsor_output: SPONSOR_COIN_UNITS,
            })),
        )
    }

    fn sign_step(&self, index: usize) -> Option<OperationStep> {
        let &(row, task) = self.sign_schedule.get(index)?;
        let pending = self.pending.get(row)?;
        let task = pending.tasks.get(task)?;
        let outpoint = task.coin().outpoint();
        Some(OperationStep::new(
            &format!(
                "authorize-sponsor-input/{}/{}",
                vector_id(&pending.case).fixture().ordinal(),
                task.request().input()
            ),
            OperationSubject::SponsorSigning(Box::new(TargetSponsorSigningSubject {
                finalized_transaction: task.request().transaction().to_vec(),
                sponsor_input_index: task.request().input(),
                sponsor_outpoint: WireOutpoint {
                    txid: txid_to_wire(&outpoint.txid()),
                    vout: outpoint.index(),
                },
                sighash_profile: WireSighashProfile::AllInputsAllOutputs,
            })),
        ))
    }

    fn submit_step(&self, index: usize) -> Option<OperationStep> {
        let vector = self.vectors.get(index)?;
        Some(OperationStep::new(
            &format!("submit-vector/{}", vector.id().fixture().ordinal()),
            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: vector.bytes().to_vec(),
            })),
        ))
    }

    /// Read the issued asset. The bundle waits for the reserve.
    fn settle_issuance(&mut self, response: &NativeOperationResponse) -> Result<(), PlanRefusal> {
        if response.observed_layer != ObservedOutcomeLayer::Accepted {
            return Err(PlanRefusal::IssuanceDidNotHappen(response.observed_layer));
        }
        let named = response
            .issued_asset
            .clone()
            .ok_or(PlanRefusal::IssuanceNamedNoAsset)?;
        let asset = asset_from_hex(&named)
            .ok_or_else(|| PlanRefusal::IssuedAssetUnreadable(named.clone()))?;
        self.transcript.issued_asset = Some(asset);
        Ok(())
    }

    /// Read the sponsor coins, link the bundle, and build the schedule.
    fn settle_sponsor_funding(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), PlanRefusal> {
        if response.observed_layer != ObservedOutcomeLayer::Accepted {
            return Err(PlanRefusal::SponsorFundingDidNotHappen(
                response.observed_layer,
            ));
        }
        let wanted = self.sponsor_coins_wanted();
        if response.funded_outputs.len() != wanted {
            return Err(PlanRefusal::SponsorCardinalityWrong {
                wanted,
                reported: response.funded_outputs.len(),
            });
        }

        // One reserve identity across every coin, because one is what
        // gets linked into the leaves.
        let mut reserve: Option<[u8; 32]> = None;
        let mut coins = Vec::with_capacity(wanted);
        for output in &response.funded_outputs {
            let asset = asset_from_hex(&output.asset)
                .ok_or_else(|| PlanRefusal::SponsorAssetUnreadable(output.asset.clone()))?;
            match reserve {
                Some(known) if known != asset => return Err(PlanRefusal::SponsorAssetsDisagree),
                Some(_) => {}
                None => reserve = Some(asset),
            }
            let outpoint = outpoint_from_wire(&output.outpoint.txid, output.outpoint.vout)
                .ok_or_else(|| {
                    PlanRefusal::SponsorOutpointUnreadable(output.outpoint.txid.clone())
                })?;
            let program = bytes_from_hex(&output.script)
                .ok_or_else(|| PlanRefusal::SponsorProgramUnreadable(output.script.clone()))?;
            coins.push(SponsorCoin::new(outpoint, output.amount_satoshis, program));
        }
        let reserve = reserve.ok_or(PlanRefusal::SponsorCardinalityWrong {
            wanted,
            reported: 0,
        })?;
        self.transcript.reserve_asset = Some(reserve);
        self.sponsor_coins = coins;

        let asset = self
            .transcript
            .issued_asset
            .ok_or(PlanRefusal::IssuanceNamedNoAsset)?;

        // The bundle is linked against the two assets the run observed,
        // and its pin is derived from its own tree. Neither could have
        // been stated before these answers arrived.
        let bundle = ceremony_bundle(asset, reserve)
            .map_err(|cause| PlanRefusal::Bundle(Box::new(cause.into())))?;
        let program = bundle
            .pin()
            .output_script(bundle.target())
            .map_err(|cause| {
                PlanRefusal::Bundle(Box::new(VectorError::TargetMaterializationFailed {
                    vector: vector_id(&self.cases[0]),
                    cause,
                }))
            })?;
        self.program = program;
        self.bundle = Some(bundle);

        self.schedule_funding();
        Ok(())
    }

    /// Build the funding schedule and choose the mutation subject.
    ///
    /// One funding request per ASH input, in fixture order and then
    /// member order, because materialization pairs the coins with the
    /// amounts in exactly that order.
    ///
    /// Except for a row the reviewed target bound says cannot be funded
    /// at all. That row gets exactly one step — the first input the
    /// bound forbids — and none of the rest, because the remaining
    /// coins would be cut for a transaction that is never built. The
    /// one step is asked so that the divergence is observed rather than
    /// assumed.
    ///
    /// Except also for the mutation subject, chosen here rather than
    /// after materialization: the widest fundable row, ties broken
    /// toward the lowest ordinal, exactly the rule
    /// [`Self::choose_mutation_subject`] used to apply once the vectors
    /// existed. Nothing that rule reads is learned from a target
    /// answer — a case's own ash-input count and whether its bound
    /// forbids it are both known from the census alone — so the choice
    /// can be made before anything is asked for, and the subject earns
    /// one extra replica of its whole input set per arm this run will
    /// actually submit, plus its own at replica zero. Every other row
    /// is funded once.
    fn schedule_funding(&mut self) {
        self.schedule.clear();
        self.unfundable.clear();

        for case in &self.cases {
            if target_amount_standing(case).unfundable_input().is_some() {
                self.unfundable.insert(vector_id(case));
            }
        }

        self.subject = self
            .cases
            .iter()
            .enumerate()
            .filter(|(_, case)| !self.unfundable.contains(&vector_id(case)))
            .max_by_key(|(index, case)| (case.ash_inputs(), std::cmp::Reverse(*index)))
            .map(|(_, case)| vector_id(case));

        let submittable = submittable_mutations().len();
        for case in &self.cases {
            let vector = vector_id(case);
            let Some((member, beyond)) = target_amount_standing(case).unfundable_input() else {
                let replicas = if Some(vector) == self.subject {
                    1 + submittable
                } else {
                    1
                };
                for replica in 0..replicas {
                    for (member, amount) in case.inputs().iter().enumerate() {
                        self.schedule.push(PlannedFunding {
                            vector,
                            member,
                            amount: amount.get(),
                            expectation: FundingExpectation::Coin,
                            replica,
                        });
                    }
                }
                continue;
            };
            self.schedule.push(PlannedFunding {
                vector,
                member,
                amount: beyond.stated(),
                expectation: FundingExpectation::BeyondBound(beyond),
                replica: 0,
            });
        }
    }

    /// Check the target stored the program the planner asked for.
    fn settle_derive(&mut self, response: &NativeOperationResponse) -> Result<(), PlanRefusal> {
        if response.observed_layer != ObservedOutcomeLayer::Accepted {
            return Err(PlanRefusal::FundingDidNotHappen(response.observed_layer));
        }
        let output =
            response
                .funded_outputs
                .first()
                .ok_or(PlanRefusal::FundingCardinalityWrong {
                    wanted: 1,
                    reported: response.funded_outputs.len(),
                })?;
        let asked = hex_of_slice(&self.program);
        if output.script != asked {
            return Err(PlanRefusal::ProgramMismatch {
                asked,
                stored: output.script.clone(),
            });
        }
        self.transcript.constructor_program = Some(self.program.clone());
        Ok(())
    }

    /// Record one funded coin against the vector it was cut for.
    ///
    /// # Why one row's refusal is not the run's refusal
    ///
    /// An unexpected funding failure still stops everything: the coins
    /// a later step depends on would not exist, and continuing would
    /// submit transactions against outputs no chain holds. The one
    /// exception is a step the reviewed target bound already said could
    /// not succeed. There the refusal is the answer the step was asked
    /// for, the row it belongs to is excluded from everything
    /// downstream, and the rest of the census carries on — which is the
    /// difference between a target that cannot state one amount and a
    /// ceremony that has broken.
    fn settle_funding(
        &mut self,
        index: usize,
        response: &NativeOperationResponse,
    ) -> Result<(), PlanRefusal> {
        let planned = self.schedule[index];
        if let FundingExpectation::BeyondBound(beyond) = planned.expectation {
            if response.observed_layer == ObservedOutcomeLayer::Accepted {
                return Err(PlanRefusal::AmountBeyondBoundWasFunded {
                    vector: planned.vector,
                    stated: beyond.stated(),
                    bound: beyond.bound(),
                });
            }
            self.transcript.divergences.push(ObservedDivergence {
                vector: planned.vector,
                beyond,
                layer: response.observed_layer,
                detail: response.observed_detail.clone(),
            });
            return Ok(());
        }
        if response.observed_layer != ObservedOutcomeLayer::Accepted {
            return Err(PlanRefusal::FundingDidNotHappen(response.observed_layer));
        }
        if response.funded_outputs.len() != 1 {
            return Err(PlanRefusal::FundingCardinalityWrong {
                wanted: 1,
                reported: response.funded_outputs.len(),
            });
        }
        let output = &response.funded_outputs[0];
        let outpoint = outpoint_from_wire(&output.outpoint.txid, output.outpoint.vout)
            .ok_or_else(|| PlanRefusal::FundedOutpointUnreadable(output.outpoint.txid.clone()))?;
        // Replica zero is every row's ordinary funding, the control's
        // included. Every other replica is a mutation subject arm's own
        // coins, and lands in its own record so it is never mistaken for
        // the control's — see `PlannedFunding::replica`.
        if planned.replica == 0 {
            self.transcript
                .funded
                .entry(planned.vector)
                .or_default()
                .push(outpoint);
        } else {
            self.transcript
                .arm_funded
                .entry(planned.replica)
                .or_default()
                .push(outpoint);
        }
        self.transcript
            .coins
            .insert(outpoint, output.amount_satoshis);
        Ok(())
    }

    /// Materialize every vector the ceremony actually created coins for.
    ///
    /// A row the target's bound refused is skipped rather than built
    /// against a short funding record. There is nothing to build: the
    /// coins do not exist, and a transaction naming outpoints no chain
    /// created is the very thing Wave 11 stopped producing.
    ///
    /// A sponsored row is not built here either, and for an unrelated
    /// reason: it cannot be, until somebody has authorized the input it
    /// carries. What happens instead is that the construction is run far
    /// enough to learn what those authorizations have to be about, and
    /// the row waits in [`Self::pending`] for them.
    fn materialize_all(&mut self) -> Result<(), PlanRefusal> {
        let bundle = self
            .bundle
            .clone()
            .ok_or(PlanRefusal::IssuanceNamedNoAsset)?;
        let mut vectors = Vec::with_capacity(self.cases.len());
        let mut pending = Vec::new();
        // The sponsor coins are handed out one region at a time, in
        // census order, so no two rows are built against one coin.
        let mut spare = self.sponsor_coins.clone();
        for case in &self.cases {
            let id = vector_id(case);
            if self.unfundable.contains(&id) {
                continue;
            }
            let outpoints = self.transcript.funded.get(&id).cloned().unwrap_or_default();
            let funding = AshFunding::new(id, outpoints)
                .map_err(|cause| PlanRefusal::Bundle(Box::new(cause)))?;
            let members = usize::from(case.sponsor().members());
            if members == 0 {
                let vector = materialize(&bundle, case, &funding)
                    .map_err(|cause| PlanRefusal::Bundle(Box::new(cause)))?;
                vectors.push(vector);
                continue;
            }
            if spare.len() < members {
                return Err(PlanRefusal::SponsorCardinalityWrong {
                    wanted: members,
                    reported: spare.len(),
                });
            }
            let coins: Vec<SponsorCoin> = spare.drain(..members).collect();
            let tasks = sponsor_signing_requests(&bundle, case, &funding, &coins)
                .map_err(|cause| PlanRefusal::Bundle(Box::new(cause)))?;
            pending.push(PendingSponsored {
                case: case.clone(),
                funding,
                coins,
                tasks,
                signatures: BTreeMap::new(),
            });
        }

        self.sign_schedule = pending
            .iter()
            .enumerate()
            .flat_map(|(row, entry)| (0..entry.tasks.len()).map(move |task| (row, task)))
            .collect();
        self.pending = pending;
        self.vectors = vectors;
        Ok(())
    }

    /// Record one authorization the executor produced.
    ///
    /// The echo is compared here, before the authorization is put
    /// anywhere near a construction. The builder compares it again from
    /// its own side and would refuse a mismatch; making it twice is
    /// deliberate, because the two refusals name different things — this
    /// one names the exchange that produced the wrong bytes, and the
    /// builder's names the construction that could not use them.
    fn settle_authorization(
        &mut self,
        index: usize,
        response: &NativeOperationResponse,
    ) -> Result<(), PlanRefusal> {
        let &(row, task) = self
            .sign_schedule
            .get(index)
            .ok_or(PlanRefusal::IssuanceNamedNoAsset)?;
        let pending = self
            .pending
            .get_mut(row)
            .ok_or(PlanRefusal::IssuanceNamedNoAsset)?;
        let vector = vector_id(&pending.case);
        let task = pending
            .tasks
            .get(task)
            .ok_or(PlanRefusal::AuthorizationCarriedNoWitness(vector))?;

        if response.observed_layer != ObservedOutcomeLayer::Accepted {
            return Err(PlanRefusal::AuthorizationDidNotHappen(
                response.observed_layer,
            ));
        }
        let bound = response
            .signature_bound_to
            .as_ref()
            .ok_or(PlanRefusal::AuthorizationCarriedNoWitness(vector))?;
        let sent = task.request().transaction();
        if bound.as_slice() != sent {
            return Err(PlanRefusal::AuthorizationBoundToOtherBytes {
                vector,
                sent: sent.len(),
                echoed: bound.len(),
            });
        }
        if response.sponsor_witness.is_empty() {
            return Err(PlanRefusal::AuthorizationCarriedNoWitness(vector));
        }

        let outpoint = task.coin().outpoint();
        pending.signatures.insert(
            outpoint,
            SponsorSignature::new(bound.clone(), response.sponsor_witness.clone()),
        );
        Ok(())
    }

    /// Which stage follows the funding schedule.
    ///
    /// Authorizations where a sponsored row is waiting for one, and
    /// submission where none is: a census of sponsorless rows must not
    /// stall in a stage with nothing to ask.
    const fn after_funding(&self) -> Stage {
        if self.sign_schedule.is_empty() {
            Stage::Submit(0)
        } else {
            Stage::Sign(0)
        }
    }

    /// Build the sponsored rows where there was nothing to authorize.
    ///
    /// Reached only when the schedule is empty, which means the census
    /// held no sponsored row at all; the loop then runs over nothing.
    /// It exists so the two paths out of funding both leave every
    /// admitted row built.
    fn finish_if_signed(&mut self) -> Result<(), PlanRefusal> {
        if self.sign_schedule.is_empty() {
            self.finish_sponsored()?;
        }
        Ok(())
    }

    /// Build every sponsored row now that its authorizations are in.
    ///
    /// The construction runs a second time, against the same inputs, and
    /// the authorization it is handed carries the bytes it was produced
    /// against. The builder compares those with what this run settles
    /// on, which is what makes running it twice sound rather than
    /// assumed: a second run that differed would be refused here rather
    /// than submitted.
    fn finish_sponsored(&mut self) -> Result<(), PlanRefusal> {
        let bundle = self
            .bundle
            .clone()
            .ok_or(PlanRefusal::IssuanceNamedNoAsset)?;
        for entry in &self.pending {
            let vector = materialize_sponsored(
                &bundle,
                &entry.case,
                &entry.funding,
                &entry.coins,
                &entry.signatures,
            )
            .map_err(|cause| PlanRefusal::Bundle(Box::new(cause)))?;
            self.vectors.push(vector);
        }
        // Every vector this run will submit now exists, so the one held
        // back to be mutated can be settled and moved to the end.
        self.choose_mutation_subject();
        Ok(())
    }

    /// Move the reserved subject to the end of the submission order.
    ///
    /// # Why the subject is submitted after its own mutations
    ///
    /// A mutation of an *already submitted* vector spends coins the
    /// target has already seen spent, so the target refuses it for
    /// double-spending whatever the mutation did. Those refusals look
    /// like negative evidence and are worth nothing: the first Wave-13
    /// run produced eight of them, seven reading `missing-inputs` and
    /// one `txn-already-known`, and not one was attributable to the
    /// mutation it was supposed to be about.
    ///
    /// So the subject is held back. Its mutations are offered while its
    /// own replicas are still unspent, and the un-mutated subject
    /// follows as the control: a refused mutation and an accepted
    /// control differ by exactly the mutation, which is the whole
    /// argument. A control the target refuses invalidates every
    /// mutation submitted before it, and the transcript records the
    /// control so a reader can tell.
    ///
    /// # Why the choice itself is not made here
    ///
    /// It used to be: the widest row among the materialized vectors,
    /// ties broken toward the lowest ordinal. `self.subject` now
    /// carries that same choice, made before funding was scheduled so
    /// the subject's extra replicas could be asked for — this function
    /// only has to find the vector that choice named and move it.
    fn choose_mutation_subject(&mut self) {
        let Some(id) = self.subject else {
            return;
        };
        let Some(position) = self.vectors.iter().position(|vector| vector.id() == id) else {
            return;
        };
        let subject = self.vectors.remove(position);
        self.transcript.mutation_subject = Some(subject.id());
        self.vectors.push(subject);
    }

    /// Which vector this run holds back to mutate, if it chose one.
    const fn control_index(&self) -> Option<usize> {
        match self.vectors.len() {
            0 => None,
            length => Some(length - 1),
        }
    }

    /// Build the mutations of the reserved subject.
    ///
    /// Every arm is staged whether or not anything has been accepted
    /// yet, because attributability no longer waits on one shared
    /// acceptance: each submittable arm was funded its own replica of
    /// the subject's input set, so each is materialized from its own
    /// coins here and stands or falls on its own answer. An arm with
    /// nothing to act on in this shape, or whose replica did not fund,
    /// is recorded as a fixture construction failure, which is what it
    /// is: no target was asked, so no target refused, and the row it
    /// would have answered stays outstanding.
    ///
    /// # An arm whose boundary is before the target is not submitted
    ///
    /// A class whose §1.5 boundary is a pre-target one expects
    /// first-party code to refuse the shape, so no answer the target
    /// could give would satisfy it: submitting it asks a question whose
    /// every answer is the wrong one. Such an arm earns no funding
    /// replica in the first place — see `settle_sponsor_funding` — and
    /// is filtered here for the same reason, so the two cannot disagree
    /// about which arms this run submits.
    fn stage_mutants(&mut self) {
        let Some(asset) = self.transcript.issued_asset else {
            return;
        };
        let Some(bundle) = self.bundle.clone() else {
            return;
        };
        let Some(subject_id) = self.transcript.mutation_subject else {
            return;
        };
        let Some(case) = self.cases.iter().find(|case| vector_id(case) == subject_id) else {
            return;
        };

        // Replica zero is the control's, so arm numbering starts at
        // one and counts forward in exactly the plan order
        // `submittable_mutations` filters — the two must agree, or an
        // arm here would read another arm's coins.
        let mut replica = 0_usize;
        for mutation in NegativeMutation::ALL {
            // A class this module cannot look up is a drift between the
            // matrix and the arms, and staging past it would submit a
            // vector nothing states an expectation for.
            let Ok(_) = mutation.expected_boundary() else {
                self.transcript.mutants.push(MutantOutcome {
                    origin: subject_id,
                    mutation: *mutation,
                    layer: ObservedOutcomeLayer::FixtureConstructionFailure,
                    detail: Some("§18 names no class by this arm's name".into()),
                    accepted_txid: None,
                    bytes: Vec::new(),
                });
                continue;
            };
            if crate::abi_validation::precedes_the_target(*mutation).is_ok_and(|withheld| withheld)
            {
                self.transcript.mutants.push(MutantOutcome {
                    origin: subject_id,
                    mutation: *mutation,
                    layer: ObservedOutcomeLayer::FixtureConstructionFailure,
                    detail: Some(
                        "not submitted: the class is settled at the ABI-validation boundary".into(),
                    ),
                    accepted_txid: None,
                    bytes: Vec::new(),
                });
                continue;
            }

            replica += 1;
            let outpoints = self
                .transcript
                .arm_funded
                .get(&replica)
                .cloned()
                .unwrap_or_default();
            let Ok(funding) = AshFunding::new(subject_id, outpoints) else {
                self.transcript.mutants.push(MutantOutcome {
                    origin: subject_id,
                    mutation: *mutation,
                    layer: ObservedOutcomeLayer::FixtureConstructionFailure,
                    detail: Some(
                        "not submitted: this arm's own replica was not funded to the subject's shape"
                            .into(),
                    ),
                    accepted_txid: None,
                    bytes: Vec::new(),
                });
                continue;
            };
            let Ok(arm_subject) = materialize(&bundle, case, &funding) else {
                self.transcript.mutants.push(MutantOutcome {
                    origin: subject_id,
                    mutation: *mutation,
                    layer: ObservedOutcomeLayer::FixtureConstructionFailure,
                    detail: Some("this arm's own replica did not materialize".into()),
                    accepted_txid: None,
                    bytes: Vec::new(),
                });
                continue;
            };
            match apply(&arm_subject, asset, *mutation) {
                Ok(mutant) => self.mutants.push(mutant),
                Err(_) => self.transcript.mutants.push(MutantOutcome {
                    origin: subject_id,
                    mutation: *mutation,
                    layer: ObservedOutcomeLayer::FixtureConstructionFailure,
                    detail: Some("the subject's shape gave this mutation nothing to act on".into()),
                    accepted_txid: None,
                    bytes: Vec::new(),
                }),
            }
        }
    }

    fn mutant_step(&self, index: usize) -> Option<OperationStep> {
        let mutant = self.mutants.get(index)?;
        Some(OperationStep::new(
            &format!("submit-mutant/{}", mutant.mutation().class_name()),
            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                transaction_bytes: mutant.bytes().to_vec(),
            })),
        ))
    }

    fn settle_mutant(&mut self, index: usize, response: &NativeOperationResponse) {
        if let Some(mutant) = self.mutants.get(index) {
            self.transcript.mutants.push(MutantOutcome {
                origin: mutant.origin(),
                mutation: mutant.mutation(),
                layer: response.observed_layer,
                detail: response.observed_detail.clone(),
                accepted_txid: response.accepted_txid.clone(),
                bytes: mutant.bytes().to_vec(),
            });
        }
    }

    /// Record one submission answer, and say whether the plan survives it.
    ///
    /// # Errors
    ///
    /// [`PlanRefusal::WeightObservationDisagrees`] where the target
    /// weighed the submitted bytes differently than the ABI did. The
    /// observed *layer* is never a refusal — every layer the target can
    /// reach is a fact about the target, and the two that are not target
    /// facts are recorded as themselves rather than dropped
    /// `(´[PLAN-rule:guide12-exec:failure-layers]´)`. A weight
    /// disagreement is not a layer: it is the two sides of the boundary
    /// contradicting each other about bytes they both hold.
    fn settle_submission(
        &mut self,
        index: usize,
        response: &NativeOperationResponse,
    ) -> Result<(), PlanRefusal> {
        if let Some(vector) = self.vectors.get(index) {
            let predicted_weight = vector.weight();
            let observed_weight = response.resources.transaction_weight;
            self.transcript.submissions.push(SubmissionOutcome {
                vector: vector.id(),
                layer: response.observed_layer,
                detail: response.observed_detail.clone(),
                accepted_txid: response.accepted_txid.clone(),
                bytes: vector.bytes().to_vec(),
                predicted_weight,
                observed_weight,
                witness_bytes: vector.witness_bytes(),
                virtual_size: vector.virtual_size(),
            });

            // §20.5: a resource mismatch fails the resource report even
            // where the transaction was accepted. The ABI settled this
            // weight from the same bytes the target weighed, so a
            // disagreement is not a tolerance to widen — one of the two
            // is wrong about bytes both of them hold, and continuing
            // would build the rest of the run on whichever it was.
            //
            // The refusal is returned rather than filed here, so that the
            // one place a refusal is recorded is also the one place the
            // plan is stopped. Filing it here and returning nothing is
            // exactly the state `G13-R03` was about: a transcript that
            // says refused under a run that says completed.
            if observed_weight.is_some_and(|observed| observed != predicted_weight) {
                return Err(PlanRefusal::WeightObservationDisagrees {
                    vector: vector.id(),
                    predicted: predicted_weight,
                    observed: observed_weight.unwrap_or_default(),
                });
            }
        }
        Ok(())
    }
}

impl TargetOperationPlanner for CompactAshOperationPlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        // Settle the previous answer before deciding what to ask next.
        // The stage is what says which answer this was; the case identity
        // is echoed back by the executor and is not re-parsed here,
        // because reading the stage out of a name the planner itself
        // chose would be this package agreeing with itself.
        if let Some((_case, response)) = previous {
            match self.stage {
                Stage::Issue => {
                    if let Err(refusal) = self.settle_issuance(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::FundSponsor;
                }
                Stage::FundSponsor => {
                    if let Err(refusal) = self.settle_sponsor_funding(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::Derive;
                }
                Stage::Derive => {
                    if let Err(refusal) = self.settle_derive(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::Fund(0);
                }
                Stage::Fund(index) => {
                    if let Err(refusal) = self.settle_funding(index, response) {
                        return Err(self.refuse(refusal));
                    }
                    let next = index + 1;
                    if next < self.schedule.len() {
                        self.stage = Stage::Fund(next);
                    } else {
                        if let Err(refusal) = self.materialize_all() {
                            return Err(self.refuse(refusal));
                        }
                        self.stage = self.after_funding();
                        if let Err(refusal) = self.finish_if_signed() {
                            return Err(self.refuse(refusal));
                        }
                    }
                }
                Stage::Sign(index) => {
                    if let Err(refusal) = self.settle_authorization(index, response) {
                        return Err(self.refuse(refusal));
                    }
                    let next = index + 1;
                    if next < self.sign_schedule.len() {
                        self.stage = Stage::Sign(next);
                    } else {
                        if let Err(refusal) = self.finish_sponsored() {
                            return Err(self.refuse(refusal));
                        }
                        self.stage = Stage::Submit(0);
                    }
                }
                Stage::Submit(index) => {
                    if let Err(refusal) = self.settle_submission(index, response) {
                        return Err(self.refuse(refusal));
                    }
                    let next = index + 1;
                    // The reserved subject is the last vector and is not
                    // submitted here: its mutations go first, while its
                    // coins are still unspent.
                    self.stage = if Some(next) < self.control_index() {
                        Stage::Submit(next)
                    } else {
                        self.stage_mutants();
                        if self.mutants.is_empty() {
                            Stage::SubmitControl
                        } else {
                            Stage::SubmitMutant(0)
                        }
                    };
                }
                Stage::SubmitMutant(index) => {
                    self.settle_mutant(index, response);
                    let next = index + 1;
                    // Every arm was funded its own replica, so an
                    // acceptance here spends only that arm's coins and
                    // every later arm is offered exactly as planned —
                    // there is no shared subject left to abandon the
                    // rest for.
                    self.stage = if next < self.mutants.len() {
                        Stage::SubmitMutant(next)
                    } else {
                        Stage::SubmitControl
                    };
                }
                Stage::SubmitControl => {
                    if let Some(index) = self.control_index()
                        && let Err(refusal) = self.settle_submission(index, response)
                    {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::Done;
                }
                Stage::Done => {}
            }
        }

        Ok(match self.stage {
            Stage::Issue => Some(self.issue_step()),
            Stage::FundSponsor => Some(self.sponsor_step()),
            Stage::Derive => Some(self.derive_step()),
            Stage::Fund(index) => self.fund_step(index),
            Stage::Sign(index) => self.sign_step(index),
            Stage::Submit(index) => self.submit_step(index),
            Stage::SubmitMutant(index) => self.mutant_step(index),
            Stage::SubmitControl => self
                .control_index()
                .and_then(|index| self.submit_step(index)),
            Stage::Done => None,
        })
    }
}

/// The target's own spelling of a 32-byte identity.
///
/// A target prints an asset identity in the reverse of the order it
/// commits to it in, exactly as it does for a transaction identity. The
/// transcript holds the committed order, because that is the order the
/// linked programs introspect and the order the constructor writes into
/// an explicit asset field; the reversal happens here and in
/// [`asset_from_hex`], at the two points where the target's spelling
/// crosses the boundary.
fn hex_of(bytes: [u8; 32]) -> String {
    let mut printed = bytes;
    printed.reverse();
    hex_of_slice(&printed)
}

fn hex_of_slice(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        // The width is fixed and the sink is a `String`, so the write
        // cannot fail; the result is consumed rather than unwrapped so
        // that no formatting path can panic.
        let _ = write!(text, "{byte:02x}");
    }
    text
}

fn bytes_from_hex(text: &str) -> Option<Vec<u8>> {
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

/// One asset identity, from the spelling the target printed.
///
/// Reversed into the committed order. See [`hex_of`].
fn asset_from_hex(text: &str) -> Option<[u8; 32]> {
    let mut bytes = bytes_from_hex(text)?;
    bytes.reverse();
    <[u8; 32]>::try_from(bytes.as_slice()).ok()
}

/// The target's own spelling of a transaction identity.
///
/// The reverse of the order this crate's `Txid` holds, for the reason
/// [`outpoint_from_wire`] states from the other direction.
fn txid_to_wire(txid: &Txid) -> String {
    let mut printed = *txid.internal();
    printed.reverse();
    hex_of_slice(&printed)
}

/// One outpoint, from the identity the target printed.
///
/// A target prints a transaction identity in the reverse of the order it
/// hashes it in, and this crate's `Txid` holds the hashed order. The
/// reversal is performed here, once, at the boundary where the target's
/// spelling arrives.
fn outpoint_from_wire(txid: &str, vout: u32) -> Option<Outpoint> {
    let mut bytes = bytes_from_hex(txid)?;
    bytes.reverse();
    let internal = <[u8; 32]>::try_from(bytes.as_slice()).ok()?;
    Outpoint::new(Txid::from_internal(internal), vout).ok()
}

/// The ceremony census, restated nowhere.
///
/// The planner's step names are its own, and the census entry each one
/// serves is stated here so a reader can check the mapping the module
/// documentation describes against something mechanical.
#[must_use]
pub const fn ceremony_census() -> &'static [FundingCeremonyStep] {
    FundingCeremonyStep::ALL
}

/// Whether a step identity names a funding step.
#[must_use]
pub fn is_funding(case: &OperationCaseId) -> bool {
    case.operation == OperationStepKind::Fund
}

#[cfg(test)]
mod tests {
    use super::{
        CompactAshOperationPlanner, PlanRefusal, hex_of_slice, is_funding, submittable_mutations,
    };
    use target_elements_conformance::executor::TargetOperationPlanner;
    use target_elements_conformance::protocol::{
        FundedOutput, MinedFundingReadback, NATIVE_PROTOCOL_SCHEMA, NativeOperationResponse,
        NativeResourceObservation, ObservedOutcomeLayer, OperationCaseId, OperationStepKind,
        OperationSubject, WireOutpoint,
    };

    /// A disposable asset identity, in the target's own spelling.
    ///
    /// A fixed value naming a coin on no chain, which authorizes
    /// nothing `(´[ADR015-rule:security:test-material]´)`.
    const ISSUED: &str = "11223344556677889900aabbccddeeff11223344556677889900aabbccddeeff";

    /// A disposable reserve identity, in the target's own spelling.
    ///
    /// Distinct from [`ISSUED`], because a fake target that answered one
    /// asset for both would let a planner confusing the two pass
    /// `(´[ADR015-rule:security:test-material]´)`.
    const RESERVE: &str = "aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899";

    /// The program a sponsor coin lands at, as the fake target reports
    /// it: version zero over a twenty-byte payload.
    const SPONSOR_PROGRAM: &str = "0014000102030405060708090a0b0c0d0e0f10111213";

    fn resources() -> NativeResourceObservation {
        NativeResourceObservation {
            script_bytes: 0,
            initial_stack_items: 0,
            peak_stack_items: None,
            peak_altstack_items: None,
            maximum_element_bytes: None,
            validation_budget_used: None,
            transaction_weight: None,
        }
    }

    /// One coin identity per funding answer, so no two collide.
    fn coin(sequence: u32) -> WireOutpoint {
        let mut bytes = [0_u8; 32];
        bytes[0..4].copy_from_slice(&sequence.to_be_bytes());
        WireOutpoint {
            txid: hex_of_slice(&bytes),
            vout: 0,
        }
    }

    /// The bound the reviewed target facts publish.
    fn bound() -> u64 {
        target_elements::reviewed_stated_amount_bound().maximum()
    }

    /// What the fake target should do with the step it was handed.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Answer {
        /// Do what the step asked.
        Perform,
        /// Report that the step did not happen.
        Refuse(ObservedOutcomeLayer),
    }

    /// A fake target that answers exactly what a step asks for.
    ///
    /// It creates no chain and validates nothing. Its whole job is to
    /// produce well-shaped answers so that the *planner's* reaction to
    /// them can be tested, which is the half of the exchange this
    /// module owns.
    struct FakeTarget<F: FnMut(&OperationCaseId, u64) -> Answer> {
        sequence: u32,
        decide: F,
        /// What this target reports weighing a submission at.
        ///
        /// A function of the bytes rather than a constant, so a test can
        /// stage a target that agrees with the ABI by decoding what it
        /// was handed, and one that disagrees by a stated amount.
        weigh: fn(&[u8]) -> Option<u64>,
    }

    impl<F: FnMut(&OperationCaseId, u64) -> Answer> FakeTarget<F> {
        fn answer(
            &mut self,
            case: &OperationCaseId,
            subject: &OperationSubject,
        ) -> NativeOperationResponse {
            let mut response = NativeOperationResponse {
                schema: NATIVE_PROTOCOL_SCHEMA,
                case: case.clone(),
                observed_layer: ObservedOutcomeLayer::Accepted,
                observed_detail: None,
                issued_asset: None,
                funded_outputs: Vec::new(),
                confidential_funded_outputs: Vec::new(),
                mined_readback: None,
                accepted_txid: None,
                sponsor_witness: Vec::new(),
                signature_bound_to: None,
                resources: resources(),
            };
            match subject {
                OperationSubject::Funding(funding) => {
                    if let Answer::Refuse(layer) = (self.decide)(case, funding.amount_per_output) {
                        response.observed_layer = layer;
                        response.observed_detail = Some(
                            "the step asks for more of the asset than this run holds".to_owned(),
                        );
                        return response;
                    }
                    self.sequence += 1;
                    if funding.issue_asset {
                        response.issued_asset = Some(ISSUED.to_owned());
                    }
                    response.funded_outputs = (0..u32::from(funding.outputs))
                        .map(|index| FundedOutput {
                            outpoint: coin(self.sequence * 16 + index),
                            asset: ISSUED.to_owned(),
                            amount_satoshis: funding.amount_per_output,
                            script: hex_of_slice(&funding.output_program),
                        })
                        .collect();
                }
                OperationSubject::SponsorFunding(sponsor) => {
                    if let Answer::Refuse(layer) =
                        (self.decide)(case, sponsor.amount_per_sponsor_output)
                    {
                        response.observed_layer = layer;
                        response.observed_detail =
                            Some("this run holds no reserve to sponsor from".to_owned());
                        return response;
                    }
                    self.sequence += 1;
                    // The reserve asset and the program are the fake
                    // target's own answers, exactly as a real adapter's
                    // are: the planner states neither.
                    response.funded_outputs = (0..u32::from(sponsor.sponsor_outputs))
                        .map(|index| FundedOutput {
                            outpoint: coin(self.sequence * 16 + index),
                            asset: RESERVE.to_owned(),
                            amount_satoshis: sponsor.amount_per_sponsor_output,
                            script: SPONSOR_PROGRAM.to_owned(),
                        })
                        .collect();
                }
                OperationSubject::SponsorSigning(signing) => {
                    // Two items, because that is the stack shape the
                    // admitted sponsor program class takes, and the echo
                    // is the bytes it was handed.
                    response.sponsor_witness = vec![vec![0x30; 71], vec![0x02; 33]];
                    response.signature_bound_to = Some(signing.finalized_transaction.clone());
                }
                OperationSubject::Submission(submission) => {
                    // A submission is offered to the same decision as
                    // every other step, so a run in which the target
                    // refuses what it is handed can be staged. The
                    // amount is zero because a submission asks for no
                    // coin; the case is what a decision distinguishes.
                    if let Answer::Refuse(layer) = (self.decide)(case, 0) {
                        response.observed_layer = layer;
                        response.observed_detail =
                            Some("this run refuses the transaction it was handed".to_owned());
                        return response;
                    }
                    response.accepted_txid = Some(ISSUED.to_owned());
                    // And the readback an acceptance owes. This fake
                    // target confirms what it accepts, so it reports
                    // the bytes it was handed back at an identity of
                    // its own choosing — which is the shape a real
                    // adapter produces and not a claim about a chain.
                    response.mined_readback = Some(MinedFundingReadback {
                        transaction_id: ISSUED.to_owned(),
                        witness_transaction_id: ISSUED.to_owned(),
                        block_hash: ISSUED.to_owned(),
                        block_height: 1,
                        raw_transaction: submission.transaction_bytes.clone(),
                    });
                    // The weight this fake target reports for the bytes
                    // it was handed, which a test chooses so that
                    // agreement and disagreement can both be staged.
                    response.resources.transaction_weight =
                        (self.weigh)(&submission.transaction_bytes);
                }
                // This fake target materializes no confidential
                // representation and states so. A step it cannot perform
                // is an infrastructure failure carrying no observation,
                // which is what an unimplemented step honestly is — and
                // the compact-ASH planner never states one, so reaching
                // this arm at all would be a planner defect rather than
                // a target answer.
                OperationSubject::ConfidentialFunding(_) => {
                    response.observed_layer = ObservedOutcomeLayer::ExecutorInfrastructureFailure;
                    response.observed_detail = Some(
                        "this fake target performs no confidential funding ceremony".to_owned(),
                    );
                }
            }
            response
        }
    }

    /// Drive a planner to completion against a fake target.
    ///
    /// Returns the planner and whether the plan ran out of steps rather
    /// than refusing.
    fn run<F: FnMut(&OperationCaseId, u64) -> Answer>(
        decide: F,
    ) -> (CompactAshOperationPlanner, bool) {
        run_weighing(decide, |_| None)
    }

    /// The same drive, with a chosen answer to "what does this weigh".
    fn run_weighing<F: FnMut(&OperationCaseId, u64) -> Answer>(
        decide: F,
        weigh: fn(&[u8]) -> Option<u64>,
    ) -> (CompactAshOperationPlanner, bool) {
        let mut planner = CompactAshOperationPlanner::new().expect("the planner builds");
        let mut target = FakeTarget {
            sequence: 0,
            decide,
            weigh,
        };
        let mut previous: Option<(OperationCaseId, NativeOperationResponse)> = None;
        loop {
            let borrowed = previous.as_ref().map(|(case, response)| (case, response));
            let step = match planner.next_step(borrowed) {
                Ok(Some(step)) => step,
                Ok(None) => return (planner, true),
                Err(_) => return (planner, false),
            };
            let response = target.answer(step.case(), step.subject());
            response
                .validate_shape()
                .expect("the fake target answers in a shape the protocol defines");
            previous = Some((step.case().clone(), response));
        }
    }

    #[test]
    fn a_run_that_accepts_something_submits_every_mutation_of_it() {
        // The negative half, end to end against a fake target: once a
        // positive row is accepted, each mutation is submitted and each
        // answer is recorded against the mutation that produced it.
        let (planner, finished) = run(refuse_beyond_bound);
        assert!(finished, "the plan ran out of steps rather than refusing");
        let transcript = planner.transcript();

        let staged = super::NegativeMutation::ALL.len();
        assert_eq!(
            transcript.mutants().len(),
            staged,
            "every mutation must produce an outcome, applied or refused",
        );
        // Each one names the mutation it was made from, and no mutation
        // is recorded twice.
        let named: std::collections::BTreeSet<_> = transcript
            .mutants()
            .iter()
            .map(super::MutantOutcome::mutation)
            .collect();
        assert_eq!(named.len(), staged, "a mutation was recorded twice");

        // And a mutation never lands among the positive submissions,
        // which is what keeps it from discharging a positive row.
        let submitted: Vec<_> = transcript
            .submissions()
            .iter()
            .map(super::SubmissionOutcome::bytes)
            .collect();
        for mutant in transcript.mutants() {
            if mutant.layer() == ObservedOutcomeLayer::FixtureConstructionFailure {
                continue;
            }
            assert!(
                !submitted.contains(&mutant.bytes()),
                "a mutated transaction was recorded as a positive submission",
            );
        }

        // The fake target answers every performed submission with
        // `ISSUED` as the accepted txid, and every submittable arm here
        // was funded its own replica: none of them spends coins another
        // arm touched, so every one of them is accepted rather than
        // just the first. Only the pre-target and drift arms carry no
        // identity, because they were never submitted at all.
        let accepted: Vec<_> = transcript
            .mutants()
            .iter()
            .filter(|mutant| mutant.layer() == ObservedOutcomeLayer::Accepted)
            .collect();
        assert_eq!(
            accepted.len(),
            submittable_mutations().len(),
            "every arm funded its own replica, so every submittable arm is accepted",
        );
        for mutant in &accepted {
            assert_eq!(
                mutant.accepted_txid(),
                Some(ISSUED),
                "an accepted mutation must carry the txid the target reported",
            );
        }
        for mutant in transcript.mutants() {
            if mutant.layer() != ObservedOutcomeLayer::Accepted {
                assert_eq!(
                    mutant.accepted_txid(),
                    None,
                    "{:?} was not accepted and must carry no txid",
                    mutant.mutation(),
                );
            }
        }
    }

    #[test]
    fn a_mutation_whose_boundary_precedes_the_target_is_never_offered() {
        // Wave 13's finding, kept from recurring. The wrong-sequence arm
        // expects the constructor to refuse the shape, and the target has
        // no rule to refuse it by — so offering it would buy an
        // acceptance that spends the subject and abandons every arm
        // behind it. The row is still recorded, carrying the reason.
        let (planner, finished) = run(refuse_beyond_bound);
        assert!(finished, "the plan ran out of steps rather than refusing");
        let transcript = planner.transcript();

        for mutation in super::NegativeMutation::ALL {
            let boundary = mutation.expected_boundary().expect("the class is named");
            if !boundary.is_pre_target() {
                continue;
            }
            let outcome = transcript
                .mutants()
                .iter()
                .find(|mutant| mutant.mutation() == *mutation)
                .unwrap_or_else(|| panic!("{mutation:?} has no recorded outcome"));
            assert_eq!(
                outcome.layer(),
                ObservedOutcomeLayer::FixtureConstructionFailure,
                "{mutation:?} reached the target despite a pre-target boundary",
            );
            assert!(
                outcome.bytes().is_empty(),
                "{mutation:?} carries bytes, which only a submitted arm does",
            );
            assert_eq!(
                outcome.accepted_txid(),
                None,
                "{mutation:?} was never submitted and must carry no txid",
            );
        }
    }

    #[test]
    fn every_mutation_is_offered_before_the_control_that_proves_its_coins() {
        // The honesty property of the whole negative half, and the one
        // the first Wave-13 run got wrong. A mutation of an already
        // submitted vector spends coins the target has seen spent, so it
        // is refused for double-spending whatever the mutation did. The
        // subject is therefore held back: its mutations go first, and it
        // follows as the control.
        let trace = std::cell::RefCell::new(Vec::new());
        let (planner, finished) = run(|case, amount| {
            trace.borrow_mut().push(case.step.clone());
            refuse_beyond_bound(case, amount)
        });
        assert!(finished, "the plan ran out of steps rather than refusing");

        let steps = trace.borrow();
        let last_mutant = steps
            .iter()
            .rposition(|step| step.starts_with("submit-mutant/"))
            .expect("the run offered no mutation at all");
        let last_vector = steps
            .iter()
            .rposition(|step| step.starts_with("submit-vector/"))
            .expect("the run submitted no vector at all");
        assert!(
            last_mutant < last_vector,
            "a mutation was offered after the control whose acceptance makes it attributable",
        );

        // And the control is the subject those mutations were made from,
        // so the two really are the same transaction but for the change.
        let transcript = planner.transcript();
        let subject = transcript
            .mutation_subject()
            .expect("the run reserved no subject to mutate");
        assert_eq!(
            transcript
                .submissions()
                .last()
                .map(super::SubmissionOutcome::vector),
            Some(subject),
            "the last submission is not the mutation subject",
        );
        for mutant in transcript.mutants() {
            assert_eq!(
                mutant.origin(),
                subject,
                "a mutation was made from a vector that is not the control",
            );
        }
    }

    /// The answer a run gives to a step the reviewed bound forbids.
    fn refuse_beyond_bound(case: &OperationCaseId, amount: u64) -> Answer {
        if is_funding(case) && amount > bound() {
            Answer::Refuse(ObservedOutcomeLayer::ExecutorInfrastructureFailure)
        } else {
            Answer::Perform
        }
    }

    /// The weight of the bytes, decoded independently of the ABI.
    ///
    /// A second reader of the same bytes rather than an echo of the
    /// prediction: a fake target that simply repeated what it was told
    /// would make the comparison agree with itself.
    fn decoded_weight(bytes: &[u8]) -> Option<u64> {
        transaction::TargetTransaction::decode(bytes)
            .ok()
            .map(|transaction| transaction.weight())
    }

    /// One weight unit more than the bytes actually weigh.
    fn overstated_weight(bytes: &[u8]) -> Option<u64> {
        decoded_weight(bytes).map(|weight| weight + 1)
    }

    #[test]
    fn a_target_that_weighs_a_submission_as_the_abi_did_agrees() {
        // §20.5's comparison in its passing case. The fake target
        // decodes the bytes it was handed and reports their weight, so
        // the agreement is between two independent readings of one byte
        // string rather than between the ABI and an echo of itself.
        let (planner, finished) = run_weighing(refuse_beyond_bound, decoded_weight);
        assert!(finished, "the plan ran out of steps rather than refusing");

        let transcript = planner.transcript();
        assert_eq!(transcript.refusal(), None);
        assert!(
            !transcript.submissions().is_empty(),
            "the run submitted nothing, so nothing was weighed",
        );
        for submission in transcript.submissions() {
            assert_eq!(
                submission.weight_agrees(),
                Some(true),
                "vector {:?} was weighed at {:?} against a predicted {}",
                submission.vector(),
                submission.observed_weight(),
                submission.predicted_weight(),
            );
            assert!(submission.predicted_weight() > 0);
        }
    }

    #[test]
    fn a_target_that_weighs_a_submission_differently_refuses_the_plan() {
        // The negative control, and the one that matters: §20.5 makes a
        // mismatch fail even where the transaction was accepted, so
        // this fake target accepts everything and misweighs it by one.
        // A comparison that only ever ran on agreeing figures would
        // agree with itself.
        let (planner, _) = run_weighing(refuse_beyond_bound, overstated_weight);

        let transcript = planner.transcript();
        let refusal = transcript
            .refusal()
            .expect("a misweighed submission refuses the plan");
        assert!(
            matches!(refusal, PlanRefusal::WeightObservationDisagrees { .. }),
            "the refusal must name the weight disagreement, not something else: {refusal:?}",
        );

        let disagreeing = transcript
            .submissions()
            .iter()
            .find(|submission| submission.weight_agrees() == Some(false))
            .expect("the misweighed submission is recorded with its two figures");
        assert_eq!(
            disagreeing.observed_weight(),
            Some(disagreeing.predicted_weight() + 1),
        );
    }

    /// `G13-R03`: a fatal plan refusal actually refuses the plan.
    ///
    /// A weight disagreement is fatal, so the submission stages refuse on
    /// it exactly as every other stage refuses on its own fatal answers.
    /// The state this forbids is the one the row named: a run that ends
    /// by running out of steps while its transcript carries a fatal
    /// refusal, so that the control plane says the operation completed
    /// and the operation report says it was refused.
    ///
    /// Three properties at once, which is what makes the state
    /// contradictory rather than merely surprising: the mismatch is
    /// staged, the run refuses, and nothing was submitted after it.
    #[test]
    fn a_misweighed_submission_stops_the_run_rather_than_completing_it() {
        let (planner, finished) = run_weighing(refuse_beyond_bound, overstated_weight);
        let transcript = planner.transcript();

        // The fixture really did stage the mismatch this row is about.
        let refusal = transcript
            .refusal()
            .expect("the fake target misweighs every submission by one");
        assert!(
            matches!(refusal, PlanRefusal::WeightObservationDisagrees { .. }),
            "the staged refusal must be the weight disagreement: {refusal:?}",
        );

        assert!(
            !finished,
            "the plan ran out of steps while carrying a fatal refusal, so the executor result says completed and the transcript says refused",
        );

        let disagreed = transcript
            .submissions()
            .iter()
            .position(|submission| submission.weight_agrees() == Some(false))
            .expect("the misweighed submission is recorded");
        assert_eq!(
            disagreed,
            transcript.submissions().len() - 1,
            "the planner emitted further submissions after a refusal it had already recorded",
        );
        assert!(
            transcript.mutants().is_empty(),
            "the planner staged mutations after a refusal it had already recorded",
        );
    }

    #[test]
    fn a_target_that_weighs_nothing_leaves_the_comparison_unmade() {
        // The third case, kept distinct from agreement. An executor
        // reports no weight where it observes none, and reading that
        // absence as a match would let a lane that stopped observing
        // keep reporting a passing resource comparison.
        let (planner, finished) = run(refuse_beyond_bound);
        assert!(finished, "the plan ran out of steps rather than refusing");

        let transcript = planner.transcript();
        assert_eq!(transcript.refusal(), None);
        for submission in transcript.submissions() {
            assert_eq!(submission.observed_weight(), None);
            assert_eq!(submission.weight_agrees(), None);
        }
    }

    #[test]
    fn the_divergent_row_is_asked_for_once_and_the_run_continues() {
        // The whole behaviour in one run: the step the bound forbids is
        // asked, refused, recorded, and the remaining rows are funded
        // and submitted anyway.
        let (planner, completed) = run(refuse_beyond_bound);
        assert!(completed, "an expected refusal must not stop the run");

        let transcript = planner.transcript();
        assert_eq!(transcript.refusal(), None);
        assert_eq!(transcript.divergences().len(), 1);

        let divergence = &transcript.divergences()[0];
        assert_eq!(
            divergence.vector().fixture().name(),
            "values-summing-to-two-pow-51-minus-one"
        );
        assert_eq!(divergence.beyond().stated(), (1 << 51) - 2);
        assert_eq!(divergence.beyond().bound(), bound());
        assert_eq!(
            divergence.layer(),
            ObservedOutcomeLayer::ExecutorInfrastructureFailure
        );
        assert!(divergence.detail().is_some(), "a refusal states a reason");

        // Twelve submissions and not thirteen: eight sponsorless rows
        // and four sponsored ones, with the divergent row absent
        // because it was never built and there was nothing to hand
        // over.
        assert_eq!(transcript.submissions().len(), 12);
        assert_eq!(planner.vectors().len(), 12);
        assert_eq!(
            transcript
                .submissions()
                .iter()
                .filter(|outcome| outcome.vector().sponsors() > 0)
                .count(),
            4,
            "the sponsored rows are submitted too"
        );
        for submission in transcript.submissions() {
            assert_ne!(
                submission.vector().fixture().name(),
                "values-summing-to-two-pow-51-minus-one",
                "a row with no coins was submitted anyway"
            );
        }
    }

    #[test]
    fn the_divergent_row_costs_one_funding_step_and_not_its_whole_family() {
        // Its family has two members and only the forbidden one is
        // asked for: the second coin would be cut for a transaction
        // that is never built.
        let recorded = std::cell::RefCell::new(Vec::new());
        let (_, completed) = run(|case, amount| {
            if is_funding(case) {
                recorded.borrow_mut().push(amount);
            }
            refuse_beyond_bound(case, amount)
        });
        assert!(completed);
        let asked = recorded.into_inner();
        assert_eq!(asked.iter().filter(|amount| **amount > bound()).count(), 1);

        // The schedule's own length, recomputed from the census rather
        // than pinned as a bare number: every runnable row's inputs,
        // except the divergent row which contributes one, plus the two
        // ceremony steps that also fund. The sponsor-funding step is
        // not among these -- it is its own kind and `is_funding` does
        // not match it, which is the point of giving it one.
        //
        // Except also for the mutation subject, whose whole input set is
        // asked for once more per submittable arm: the same widest
        // fundable row, ties broken to the lowest census index, that
        // `settle_sponsor_funding` picks before any of this is asked
        // for.
        let census = crate::fixture::positive_semantic_census().expect("the census builds");
        let cases: Vec<_> = census
            .iter()
            .filter(|case| crate::materialize::has_candidate_program(case))
            .collect();
        let scheduled: usize = cases
            .iter()
            .map(|case| {
                if crate::divergence::target_amount_standing(case).is_unfundable() {
                    1
                } else {
                    case.inputs().len()
                }
            })
            .sum();
        let subject = cases
            .iter()
            .enumerate()
            .filter(|(_, case)| !crate::divergence::target_amount_standing(case).is_unfundable())
            .max_by_key(|(index, case)| (case.ash_inputs(), std::cmp::Reverse(*index)))
            .map(|(_, case)| *case);
        let subject_extra = subject.map_or(0, |case| {
            case.inputs().len() * submittable_mutations().len()
        });
        assert_eq!(asked.len(), scheduled + 2 + subject_extra);
    }

    #[test]
    fn a_target_that_funds_the_forbidden_amount_refuses_the_plan() {
        // The other half of the expectation. The step is asked only
        // because the reviewed bound says it cannot succeed, so a
        // target that succeeds has falsified the fact the
        // classification was derived from, and the run must not
        // continue on a reading of the target known to be wrong.
        let (planner, completed) = run(|_, _| Answer::Perform);
        assert!(!completed, "the surprise must refuse the run");
        match planner.transcript().refusal() {
            Some(PlanRefusal::AmountBeyondBoundWasFunded {
                vector,
                stated,
                bound: reported,
            }) => {
                assert_eq!(
                    vector.fixture().name(),
                    "values-summing-to-two-pow-51-minus-one"
                );
                assert_eq!(*stated, (1 << 51) - 2);
                assert_eq!(*reported, bound());
            }
            other => panic!("the plan refused with {other:?}"),
        }
    }

    #[test]
    fn an_unexpected_funding_refusal_still_refuses_the_whole_plan() {
        // The conservative rule is not weakened generally: only the row
        // the bound names may fail. Any other failure leaves coins
        // missing that later steps depend on.
        let (planner, completed) = run(|case, amount| {
            if is_funding(case) && amount == 180 {
                Answer::Refuse(ObservedOutcomeLayer::ExecutorInfrastructureFailure)
            } else {
                Answer::Perform
            }
        });
        assert!(!completed);
        assert_eq!(
            planner.transcript().refusal(),
            Some(&PlanRefusal::FundingDidNotHappen(
                ObservedOutcomeLayer::ExecutorInfrastructureFailure
            ))
        );
        assert!(
            planner.transcript().divergences().is_empty(),
            "an ordinary failure is not a divergence"
        );
    }

    #[test]
    fn no_step_the_planner_states_carries_an_expectation() {
        // §16.2's forbidden direction, checked against the rendering of
        // every step this planner actually produces rather than against
        // the field list. The divergence machinery added an expectation
        // to the *plan*, and this is what says none of it leaked into
        // what crosses the boundary.
        let mut planner = CompactAshOperationPlanner::new().expect("the planner builds");
        let mut target = FakeTarget {
            sequence: 0,
            decide: refuse_beyond_bound,
            weigh: |_| None,
        };
        let mut previous: Option<(OperationCaseId, NativeOperationResponse)> = None;
        let mut steps = 0_usize;
        while let Ok(Some(step)) =
            planner.next_step(previous.as_ref().map(|(case, response)| (case, response)))
        {
            let rendered = format!("{step:?}").to_lowercase();
            for forbidden in ["expect", "refus", "bound", "diverg", "beyond", "accept"] {
                assert!(!rendered.contains(forbidden), "a step rendered {forbidden}");
            }
            assert!(matches!(
                step.case().operation,
                OperationStepKind::Fund
                    | OperationStepKind::Submit
                    | OperationStepKind::FundSponsor
                    | OperationStepKind::SignSponsor
            ));
            let response = target.answer(step.case(), step.subject());
            previous = Some((step.case().clone(), response));
            steps += 1;
        }
        assert!(steps > 0, "the planner stated no steps at all");
    }
}
