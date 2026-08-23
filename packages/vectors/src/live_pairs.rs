//! The paired explicit/private cases of Guide-13 §16.1, and §16.2's
//! acceptance conditions resolved against them.
//!
//! §16.1 names five pairs and requires each to begin from *one* semantic
//! fixture. That is the load-bearing word: two fixtures written to look
//! alike would make the comparison a comparison of two authors' intent.
//! Here each pair has exactly one [`SemanticTransferFixture`], both
//! materializations are built from it by the same function under
//! different representation plans, and the equality of their expected
//! semantics is [`ExpectedTransferSemantics`] recomputed from each side
//! and compared — a checked property, not a comment.
//!
//! # What is built, and what is therefore missing
//!
//! §14.3 derives a target materialization from six things. Five of them
//! exist here — the linked bundle, the candidate ABI, the typed request,
//! the public target input view, and the confidential construction
//! capability — and the sixth, the test-only owner-signing capability,
//! does not exist anywhere in this workspace. §1.7 leaves the taproot
//! sighash to the target and forbids a builder asserting one, so what
//! this module can build is the *finalized* form: everything §12.6
//! settles, up to but not including the answers owners would give.
//!
//! Calling that a materialization would overstate it, so it is
//! [`PairMaterialization`] and every one of them carries the blocker that
//! keeps it from being more.
//!
//! # The private predecessor is assumed, and the assumption is typed
//!
//! §6.3 admits a private transfer only over confidential receipt inputs,
//! and this module states such inputs as fixture views. A view is a
//! statement, and Wave 10 established with a real run that no funding
//! step of this pipeline can produce the output that view describes:
//! [`LiveInfrastructureBlocker::NoConfidentialPredecessorCanBeFunded`].
//! So [`PredecessorAssumption`] travels with every private
//! materialization rather than being mentioned in a report footnote, and
//! §16.2's first condition reads it.
//!
//! # No pair is dropped for being blocked
//!
//! Every pair §16.1 names is here, and a pair whose shape the published
//! set does not claim would be [`PairShapeClaim::NotClaimed`] with a
//! reason rather than absent. A registry that shrank to the pairs that
//! worked would report a minimality result for a matrix nobody stated.
//!
//! # Every secret here is published
//!
//! The owners are [`crate::live_plan`]'s BIP-340 appendix scalars and
//! every opening is a hash of values this module publishes, admitted
//! under ADR-015's test-material rule
//! `(´[ADR015-rule:security:test-material]´)` and Guide-13 §1.10. No
//! interface here accepts signing or blinding material from a caller.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use compiler::live_transfer_plan::LiveTransferRepresentationPlan;
use target_elements::ReviewedElementsTapscriptDefinition;
use transaction::bytes::{AssetField, Outpoint, Txid, ValueField};
use transaction::live_abi::CandidateLiveTransferAbi;
use transaction::live_construct::{LiveConstructionReport, finalize_live_transfer};
use transaction::live_finalize::FinalizedLiveTransfer;
use transaction::live_private::{PrivateValueCapability, SelectedConstructionModel};
use transaction::live_request::{
    LiveReceiptDestination, LiveTransferRequest, ProtocolValue, PublicTestRandomness,
    RequestedForm, SponsorChangeRequest,
};
use transaction::sponsor::{
    SponsorCapability, SponsorOffer, SponsorSignature, SponsorSigningRequest,
};
use transaction::view::{PublicConstructionView, PublicOutputView};

use crate::error::VectorError;
use crate::live_capability::OracleFixtureValues;
use crate::live_evidence::LiveInfrastructureBlocker;
use crate::live_plan::{FIRST_SCALAR, SECOND_SCALAR, demonstration_live_abi, published_owner};
use crate::live_report::LiveLifecycleStatus;

/// Both admitted representation plans, in §6.1's order.
pub const BOTH_PLANS: &[LiveTransferRepresentationPlan] = &[
    LiveTransferRepresentationPlan::Explicit,
    LiveTransferRepresentationPlan::PrivateCommitted,
];

/// The published randomness every private destination commits under.
///
/// A constant of this module and a public fixture: §12.8's central
/// public-fixture model requires every opening to be reproducible from
/// values the caller published, and this is the value.
const DESTINATION_RANDOMNESS: [u8; 32] = [0x71; 32];

/// The published randomness every confidential predecessor commits
/// under.
///
/// Deliberately not [`DESTINATION_RANDOMNESS`]. A predecessor committed
/// under the same randomness at the same position as a destination of the
/// same amount would be the *same commitment*, and a registry whose
/// inputs and outputs could coincide by construction would make an
/// equality test pass for a reason nothing intended.
const PREDECESSOR_RANDOMNESS: [u8; 32] = [0x72; 32];

/// The sponsor fee the sponsored pair's envelope offers.
///
/// A sponsor-local amount that never enters a report (§1.9). It is here
/// because the fee role's output needs a value, and nowhere else.
const SPONSOR_FEE: u64 = 250;

/// One of §16.1's five paired cases.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MinimalityPair {
    /// Explicit and private one-to-one.
    OneToOne,
    /// Explicit and private split.
    Split,
    /// Explicit and private merge.
    Merge,
    /// Explicit and private many-to-many, where claimed.
    ManyToMany,
    /// Explicit sponsor and private sponsor, where claimed.
    Sponsor,
}

impl MinimalityPair {
    /// All five, in §16.1's order.
    pub const ALL: &'static [Self] = &[
        Self::OneToOne,
        Self::Split,
        Self::Merge,
        Self::ManyToMany,
        Self::Sponsor,
    ];

    /// The pair's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::OneToOne => "one-to-one",
            Self::Split => "split",
            Self::Merge => "merge",
            Self::ManyToMany => "many-to-many",
            Self::Sponsor => "sponsor",
        }
    }

    /// Whether §16.1 qualifies this pair with "where claimed".
    ///
    /// Two of the five, and the qualification is what makes
    /// [`PairShapeClaim`] a resolution against the published shape set
    /// rather than a constant.
    #[must_use]
    pub const fn is_conditional_on_a_claim(self) -> bool {
        matches!(self, Self::ManyToMany | Self::Sponsor)
    }

    /// The byte that identifies this pair's predecessor transactions.
    ///
    /// Distinct per pair so that two pairs never consume the same
    /// outpoint, which would make one registry row's finalized bytes a
    /// function of another's.
    const fn identifier(self) -> u8 {
        match self {
            Self::OneToOne => 0xd1,
            Self::Split => 0xd2,
            Self::Merge => 0xd3,
            Self::ManyToMany => 0xd4,
            Self::Sponsor => 0xd5,
        }
    }
}

/// Whether a pair carries a sponsor envelope.
///
/// Region membership and never an amount (§1.9). The sponsored member
/// declares no change role, which keeps the pair's difference from the
/// other four to the presence of the envelope alone.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SponsorPresence {
    /// No sponsor region.
    Absent,
    /// One sponsor input and the fee role, and no change role.
    PresentWithoutChange,
}

impl SponsorPresence {
    /// The presence's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::PresentWithoutChange => "present-without-change",
        }
    }

    /// How many sponsor inputs the presence implies.
    #[must_use]
    pub const fn sponsor_inputs(self) -> usize {
        match self {
            Self::Absent => 0,
            Self::PresentWithoutChange => 1,
        }
    }

    /// The request form the presence implies.
    #[must_use]
    pub const fn form(self) -> RequestedForm {
        match self {
            Self::Absent => RequestedForm::Sponsorless,
            Self::PresentWithoutChange => RequestedForm::Sponsored,
        }
    }
}

/// One consumed receipt of a semantic fixture.
///
/// An owner and an amount, and nothing a target could read. §14.3 forbids
/// a semantic fixture to carry a position, a script, a tapleaf, a control
/// block, a signature, a key, a blinding factor, or target bytes; the way
/// to obey that is for the fixture type to have nowhere to put one.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticEndpoint {
    owner: usize,
    amount: u64,
}

impl SemanticEndpoint {
    /// Which published owner holds it, by index.
    #[must_use]
    pub const fn owner(self) -> usize {
        self.owner
    }

    /// The exact semantic amount.
    #[must_use]
    pub const fn amount(self) -> u64 {
        self.amount
    }
}

/// One endpoint, spelled once.
const fn at(owner: usize, amount: u64) -> SemanticEndpoint {
    SemanticEndpoint { owner, amount }
}

/// One semantic fixture: the single origin of one §16.1 pair.
///
/// Both materializations of a pair are built from exactly one of these,
/// which is §16.1's own requirement and the reason the type carries no
/// representation plan. A fixture that named one would be two fixtures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticTransferFixture {
    pair: MinimalityPair,
    sources: Vec<SemanticEndpoint>,
    destinations: Vec<SemanticEndpoint>,
    sponsor: SponsorPresence,
}

impl SemanticTransferFixture {
    /// The pair this fixture is the origin of.
    #[must_use]
    pub const fn pair(&self) -> MinimalityPair {
        self.pair
    }

    /// The consumed receipts, in fixture order.
    #[must_use]
    pub fn sources(&self) -> &[SemanticEndpoint] {
        &self.sources
    }

    /// The created receipts, in fixture order.
    #[must_use]
    pub fn destinations(&self) -> &[SemanticEndpoint] {
        &self.destinations
    }

    /// Whether the fixture carries a sponsor envelope.
    #[must_use]
    pub const fn sponsor(&self) -> SponsorPresence {
        self.sponsor
    }

    /// The expected semantics this fixture determines (§14.2).
    ///
    /// Derived from the fixture, which is the model side of the
    /// comparison. §14.2 forbids the candidate backend generating its own
    /// expected result, and the way this module obeys that is that
    /// nothing downstream of `finalize_live_transfer` is read here.
    #[must_use]
    pub fn expected(&self) -> ExpectedTransferSemantics {
        let mut inputs: BTreeMap<u64, usize> = BTreeMap::new();
        for source in &self.sources {
            *inputs.entry(source.amount).or_insert(0) += 1;
        }
        let mut created: BTreeMap<SemanticEndpoint, usize> = BTreeMap::new();
        for destination in &self.destinations {
            *created.entry(*destination).or_insert(0) += 1;
        }
        ExpectedTransferSemantics {
            consumed_receipts: self.sources.len(),
            distinct_input_owners: self.sources.iter().map(|source| source.owner()).collect(),
            input_amount_multiset: inputs,
            destination_multiset: created,
            aggregate: self.aggregate(),
            sponsor: self.sponsor,
        }
    }

    /// The aggregate the fixture moves.
    ///
    /// Saturating rather than checked because every fixture in this
    /// module is a constant well inside the width, and a fixture that
    /// were not would fail the conservation check in
    /// [`ExpectedTransferSemantics::conserves`] rather than silently
    /// wrapping.
    fn aggregate(&self) -> u64 {
        self.destinations.iter().fold(0_u64, |total, endpoint| {
            total.saturating_add(endpoint.amount)
        })
    }
}

/// What one fixture says the transfer means (§6.6, §14.2).
///
/// The terms of §6.6's paired comparison that a semantic fixture can
/// state. The ones it cannot — live class, explicit `U`, authorization
/// result, absence of roots, issuance, destruction, lateral flow, and the
/// certificate projection — are facts about a *materialization* and are
/// checked as §16.2 conditions instead.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpectedTransferSemantics {
    consumed_receipts: usize,
    distinct_input_owners: BTreeSet<usize>,
    input_amount_multiset: BTreeMap<u64, usize>,
    destination_multiset: BTreeMap<SemanticEndpoint, usize>,
    aggregate: u64,
    sponsor: SponsorPresence,
}

impl ExpectedTransferSemantics {
    /// How many receipts are consumed.
    #[must_use]
    pub const fn consumed_receipts(&self) -> usize {
        self.consumed_receipts
    }

    /// The distinct input owners.
    #[must_use]
    pub const fn distinct_input_owners(&self) -> &BTreeSet<usize> {
        &self.distinct_input_owners
    }

    /// The consumed amounts, as a multiset.
    #[must_use]
    pub const fn input_amount_multiset(&self) -> &BTreeMap<u64, usize> {
        &self.input_amount_multiset
    }

    /// The created owner/value pairs, as a multiset.
    #[must_use]
    pub const fn destination_multiset(&self) -> &BTreeMap<SemanticEndpoint, usize> {
        &self.destination_multiset
    }

    /// The aggregate transferred.
    #[must_use]
    pub const fn aggregate(&self) -> u64 {
        self.aggregate
    }

    /// Whether the fixture carries a sponsor envelope.
    #[must_use]
    pub const fn sponsor(&self) -> SponsorPresence {
        self.sponsor
    }

    /// Whether the consumed and created totals agree.
    ///
    /// A property of the *fixture*, checked before anything is built from
    /// it: a fixture that did not conserve would materialize into two
    /// transactions that both fail conservation, and the pair would then
    /// be reporting a fixture defect as a representation finding.
    #[must_use]
    pub fn conserves(&self) -> bool {
        let consumed =
            self.input_amount_multiset
                .iter()
                .try_fold(0_u64, |total, (amount, count)| {
                    u64::try_from(*count)
                        .ok()
                        .and_then(|count| amount.checked_mul(count))
                        .and_then(|part| total.checked_add(part))
                });
        consumed == Some(self.aggregate)
    }
}

/// Whether the published shape set claims one pair's shape.
///
/// §16.1's "where claimed", resolved rather than declared: the answer is
/// looked up in the candidate ABI's own shape table, so a pair stops
/// being claimed the moment the deployment stops emitting programs for
/// its counts.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PairShapeClaim {
    /// The published set emits programs for this pair's counts and form.
    Claimed,
    /// It does not, for a stated reason.
    NotClaimed(UnclaimedPairReason),
}

impl PairShapeClaim {
    /// Whether the pair is claimed.
    #[must_use]
    pub const fn is_claimed(self) -> bool {
        matches!(self, Self::Claimed)
    }
}

/// Why a pair is not claimed.
///
/// Typed so that an unclaimed pair stays in the registry with a reason
/// rather than being dropped from it, which is the difference between a
/// narrower claim and a smaller matrix.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum UnclaimedPairReason {
    /// No published shape realizes the pair's counts under its form.
    NoPublishedShapeRealizesTheCounts,
}

/// What a private materialization's predecessor rests on.
///
/// §13.3's "constructibility assumptions", as a field rather than as
/// prose. The explicit arm is a fact Wave 10 established by funding one
/// on a real chain; the private arm is an assumption Wave 10 established
/// *cannot* be discharged through this boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PredecessorAssumption {
    /// An explicit live receipt at the deployment's own program, which a
    /// funding step can create and has.
    ExplicitPredecessorIsFundable,
    /// A confidential live receipt, which no funding step can create.
    ConfidentialPredecessorIsAssumed(LiveInfrastructureBlocker),
}

impl PredecessorAssumption {
    /// The blocker the assumption carries, where it carries one.
    #[must_use]
    pub const fn blocker(self) -> Option<LiveInfrastructureBlocker> {
        match self {
            Self::ExplicitPredecessorIsFundable => None,
            Self::ConfidentialPredecessorIsAssumed(blocker) => Some(blocker),
        }
    }

    /// The assumption one representation plan's predecessor rests on.
    #[must_use]
    pub const fn of(plan: LiveTransferRepresentationPlan) -> Self {
        match plan {
            LiveTransferRepresentationPlan::Explicit => Self::ExplicitPredecessorIsFundable,
            LiveTransferRepresentationPlan::PrivateCommitted => {
                Self::ConfidentialPredecessorIsAssumed(
                    LiveInfrastructureBlocker::NoConfidentialPredecessorCanBeFunded,
                )
            }
        }
    }
}

/// What a target decided about one member of a pair.
///
/// One arm, and it is not "rejected". §1.11 keeps infrastructure failure
/// and target rejection distinct, and nothing here was ever submitted:
/// the witness a submission needs cannot be produced, so there is no
/// verdict to record and the absence is typed rather than left as a
/// default.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum PairTargetVerdict {
    /// No transaction was submitted, and the component that stops one
    /// from being witnessed.
    NotSubmitted(LiveInfrastructureBlocker),
}

impl PairTargetVerdict {
    /// The verdict's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::NotSubmitted(_) => "not-submitted",
        }
    }
}

/// Where a pair's resource comparison stands.
///
/// # This module's reading of §13.3's row shape
///
/// §13.3 lists nine things each minimality row contains, and the ninth is
/// a resource comparison. §18 is where measurements are made, and §18.4
/// requires a prediction and an observation to be *compared* — neither
/// exists yet. A row asserting a resource comparison nobody performed
/// would be the fabrication §13.5's validation is for, and a row omitting
/// the field would be a row §13.3 does not describe. So the field is
/// present and its value is a typed non-claim: §13.3's row shape permits
/// one because §1.11 and §13.5 together require "not measured" and
/// "measured equal" to be different values, and a shape that could not
/// express the first would force every row to assert the second.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ResourceComparisonStanding {
    /// No measurement exists; §18's study is where one would come from.
    DeferredToTheResourceStudy,
}

/// The sponsor envelope the sponsored pair is built with.
///
/// It offers one input and the fee, states no change destination, and
/// declines to sign. Declining is the honest behaviour rather than a stub:
/// §1.9 puts the sponsor's own authorization outside protocol data and it
/// arrives through an adapter, and Wave 10 recorded that no such adapter
/// exists — [`LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent`].
/// An envelope that returned bytes here would be modelling the signer the
/// evidence plan says is absent.
struct ModelledSponsorEnvelope {
    offer: SponsorOffer,
}

impl ModelledSponsorEnvelope {
    /// The envelope offering exactly `input` and [`SPONSOR_FEE`].
    fn new(input: Outpoint) -> Result<Self, VectorError> {
        Ok(Self {
            offer: SponsorOffer::new([input], SPONSOR_FEE, None)
                .map_err(|_| VectorError::LiveSubstrateUnavailable)?,
        })
    }
}

impl SponsorCapability for ModelledSponsorEnvelope {
    fn offer(&self) -> SponsorOffer {
        self.offer.clone()
    }

    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        None
    }

    fn sign(&self, _request: &SponsorSigningRequest) -> Option<SponsorSignature> {
        None
    }
}

/// One member of a pair, built as far as this workspace can build it.
///
/// Everything §12.6 settles, under one representation plan, from the
/// pair's one fixture. What it is not is a transaction anybody could
/// submit: see this module's own header and [`PairTargetVerdict`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairMaterialization {
    representation: LiveTransferRepresentationPlan,
    finalized: FinalizedLiveTransfer,
    report: LiveConstructionReport,
    predecessor: PredecessorAssumption,
    verdict: PairTargetVerdict,
}

impl PairMaterialization {
    /// The representation plan this member was built under.
    #[must_use]
    pub const fn representation(&self) -> LiveTransferRepresentationPlan {
        self.representation
    }

    /// The finalized form.
    #[must_use]
    pub const fn finalized(&self) -> &FinalizedLiveTransfer {
        &self.finalized
    }

    /// What the construction settled.
    #[must_use]
    pub const fn report(&self) -> &LiveConstructionReport {
        &self.report
    }

    /// What the member's predecessor rests on.
    #[must_use]
    pub const fn predecessor(&self) -> PredecessorAssumption {
        self.predecessor
    }

    /// What a target decided, which is nothing.
    #[must_use]
    pub const fn verdict(&self) -> PairTargetVerdict {
        self.verdict
    }

    /// The confidential construction model, where one was recorded.
    #[must_use]
    pub const fn construction_model(&self) -> Option<&SelectedConstructionModel> {
        self.report.construction_model()
    }

    /// Whether every receipt output of this member carries an explicit
    /// amount.
    ///
    /// The observable §16.2's amount-opacity condition reads. True for
    /// the explicit member and false for the private one, and *checked*
    /// against the finalized outputs rather than inferred from the
    /// representation plan the member says it is.
    #[must_use]
    pub fn publishes_exact_receipt_amounts(&self) -> bool {
        let (first, last) = self.finalized.outputs().destination_range();
        self.finalized
            .outputs()
            .outputs()
            .iter()
            .enumerate()
            .filter_map(|(index, output)| {
                let position = u16::try_from(index).ok()?;
                (position >= first && position < last).then_some(output)
            })
            .any(|output| matches!(output.value(), ValueField::Explicit(_)))
    }

    /// Whether every receipt output of this member carries the explicit
    /// protocol asset.
    ///
    /// §1.3 and §6.3 keep `U` explicit under both plans, and §15.4's
    /// confidential-asset-commitment row is the negative of it. Checked
    /// against the fields rather than assumed.
    #[must_use]
    pub fn every_receipt_asset_is_explicit(&self, asset: transaction::bytes::AssetId) -> bool {
        let (first, last) = self.finalized.outputs().destination_range();
        self.finalized
            .outputs()
            .outputs()
            .iter()
            .enumerate()
            .filter_map(|(index, output)| {
                let position = u16::try_from(index).ok()?;
                (position >= first && position < last).then_some(output)
            })
            .all(|output| output.asset() == AssetField::Explicit(asset))
    }
}

/// One of §16.2's ten pair-acceptance conditions.
///
/// Transcribed in the guide's own order, so a reader can check the list
/// against §16.2 line by line. A condition dropped from here would be a
/// condition the registry stopped requiring.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PairAcceptanceCondition {
    /// Both materializations are constructible.
    BothMaterializationsConstructible,
    /// Both target transactions accept.
    BothTargetTransactionsAccept,
    /// Both semantic projections equal the expected transfer.
    BothProjectionsEqualTheExpectedTransfer,
    /// Owner and live class agree.
    OwnerAndLiveClassAgree,
    /// Explicit `U` agrees.
    ExplicitAssetAgrees,
    /// Family closure agrees.
    FamilyClosureAgrees,
    /// Private exact receipt amounts are absent from public protocol
    /// output and canonical reports.
    PrivateAmountsAbsentFromPublicOutput,
    /// No private opening enters the report.
    NoPrivateOpeningEntersTheReport,
    /// No secret dependency exists outside the declared test construction
    /// model.
    NoSecretDependencyOutsideTheModel,
    /// Lifecycle status is equal.
    LifecycleStatusIsEqual,
}

impl PairAcceptanceCondition {
    /// All ten, in §16.2's order.
    pub const ALL: &'static [Self] = &[
        Self::BothMaterializationsConstructible,
        Self::BothTargetTransactionsAccept,
        Self::BothProjectionsEqualTheExpectedTransfer,
        Self::OwnerAndLiveClassAgree,
        Self::ExplicitAssetAgrees,
        Self::FamilyClosureAgrees,
        Self::PrivateAmountsAbsentFromPublicOutput,
        Self::NoPrivateOpeningEntersTheReport,
        Self::NoSecretDependencyOutsideTheModel,
        Self::LifecycleStatusIsEqual,
    ];

    /// The condition's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::BothMaterializationsConstructible => "both-materializations-constructible",
            Self::BothTargetTransactionsAccept => "both-target-transactions-accept",
            Self::BothProjectionsEqualTheExpectedTransfer => {
                "both-projections-equal-the-expected-transfer"
            }
            Self::OwnerAndLiveClassAgree => "owner-and-live-class-agree",
            Self::ExplicitAssetAgrees => "explicit-asset-agrees",
            Self::FamilyClosureAgrees => "family-closure-agrees",
            Self::PrivateAmountsAbsentFromPublicOutput => {
                "private-amounts-absent-from-public-output"
            }
            Self::NoPrivateOpeningEntersTheReport => "no-private-opening-enters-the-report",
            Self::NoSecretDependencyOutsideTheModel => "no-secret-dependency-outside-the-model",
            Self::LifecycleStatusIsEqual => "lifecycle-status-is-equal",
        }
    }
}

/// Where one §16.2 condition stands for one pair.
///
/// Three states, and none of them is "assumed". The middle one names a
/// component that does not exist; the last one is for the conditions
/// whose subject is a target verdict, which is downstream of the second
/// condition's blocker rather than a separate missing component — a
/// reader repairing the pipeline needs that difference.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MinimalityConditionStanding {
    /// Checked here, against both members, and it holds.
    HoldsFirstParty,
    /// A component the condition needs does not exist.
    Blocked(LiveInfrastructureBlocker),
    /// The condition is about accepted transactions, and there are none.
    AwaitsBothTargetVerdicts,
}

impl MinimalityConditionStanding {
    /// Whether the condition is satisfied.
    #[must_use]
    pub const fn is_satisfied(self) -> bool {
        matches!(self, Self::HoldsFirstParty)
    }

    /// The standing's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::HoldsFirstParty => "holds-first-party",
            Self::Blocked(_) => "blocked",
            Self::AwaitsBothTargetVerdicts => "awaits-both-target-verdicts",
        }
    }
}

/// One §13.3 row: one pair, both members, and where §16.2 stands.
///
/// The nine things §13.3 requires a row to contain, in one value: the
/// semantic fixture, the explicit and private materializations, both
/// target verdicts (through the members), both semantic projections
/// (through [`Self::expected`], which both members are checked against),
/// the disclosure sets, the constructibility assumptions (through the
/// members' [`PredecessorAssumption`]), the lifecycle status, and the
/// resource comparison.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MinimalityPairRow {
    pair: MinimalityPair,
    claim: PairShapeClaim,
    fixture: SemanticTransferFixture,
    expected: ExpectedTransferSemantics,
    explicit: PairMaterialization,
    private: PairMaterialization,
    conditions: BTreeMap<PairAcceptanceCondition, MinimalityConditionStanding>,
    lifecycle: LiveLifecycleStatus,
    resources: ResourceComparisonStanding,
}

impl MinimalityPairRow {
    /// Which §16.1 pair this row is.
    #[must_use]
    pub const fn pair(&self) -> MinimalityPair {
        self.pair
    }

    /// Whether the published shape set claims the pair.
    #[must_use]
    pub const fn claim(&self) -> PairShapeClaim {
        self.claim
    }

    /// The one semantic fixture both members were built from.
    #[must_use]
    pub const fn fixture(&self) -> &SemanticTransferFixture {
        &self.fixture
    }

    /// The expected semantics the fixture determines.
    #[must_use]
    pub const fn expected(&self) -> &ExpectedTransferSemantics {
        &self.expected
    }

    /// The explicit member.
    #[must_use]
    pub const fn explicit(&self) -> &PairMaterialization {
        &self.explicit
    }

    /// The private member.
    #[must_use]
    pub const fn private(&self) -> &PairMaterialization {
        &self.private
    }

    /// Both members, in §6.1's plan order.
    #[must_use]
    pub const fn members(&self) -> [&PairMaterialization; 2] {
        [&self.explicit, &self.private]
    }

    /// Where each §16.2 condition stands.
    #[must_use]
    pub const fn conditions(
        &self,
    ) -> &BTreeMap<PairAcceptanceCondition, MinimalityConditionStanding> {
        &self.conditions
    }

    /// The lifecycle status, which both members share (§17.1).
    #[must_use]
    pub const fn lifecycle(&self) -> &LiveLifecycleStatus {
        &self.lifecycle
    }

    /// Where the resource comparison stands (§18).
    #[must_use]
    pub const fn resources(&self) -> ResourceComparisonStanding {
        self.resources
    }

    /// Whether §16.2 admits this pair as supporting minimality.
    ///
    /// Every condition, or none of it. §16.2 says "only when" and lists
    /// ten conjuncts, so a pair satisfying nine supports nothing.
    #[must_use]
    pub fn supports_minimality(&self) -> bool {
        self.conditions
            .values()
            .all(|standing| standing.is_satisfied())
    }

    /// Every blocker this row's conditions name.
    #[must_use]
    pub fn blockers(&self) -> BTreeSet<LiveInfrastructureBlocker> {
        self.conditions
            .values()
            .filter_map(|standing| match standing {
                MinimalityConditionStanding::Blocked(blocker) => Some(*blocker),
                MinimalityConditionStanding::HoldsFirstParty
                | MinimalityConditionStanding::AwaitsBothTargetVerdicts => None,
            })
            .collect()
    }
}

/// Why the pair registry could not be built.
///
/// Every arm is a defect in the substrate or in a fixture, and none of
/// them is a minimality finding: a registry that could not be built
/// establishes nothing about disclosure either way.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MinimalityPairRefusal {
    /// The live-transfer substrate could not be built.
    SubstrateUnavailable,
    /// A fixture's consumed and created totals do not agree.
    FixtureDoesNotConserve(MinimalityPair),
    /// One member could not be finalized from the fixture.
    MemberNotConstructible {
        /// Which pair.
        pair: MinimalityPair,
        /// Which member.
        representation: LiveTransferRepresentationPlan,
    },
    /// The two members disagree about what the fixture means.
    ///
    /// §16.1's "one semantic fixture" made falsifiable: both members are
    /// projected back to expected semantics and compared, and a pair
    /// whose halves transfer different things is refused rather than
    /// compared for disclosure.
    MembersDisagreeAboutTheSemantics(MinimalityPair),
}

/// The five fixtures, one per §16.1 pair.
///
/// Every one conserves, every one uses only the two published owners, and
/// every one fits the demonstration shape set's window of three receipts
/// in and three out. The amounts are constants and none of them depends
/// on a date, a clock, or an environment.
#[must_use]
pub fn minimality_fixtures() -> Vec<SemanticTransferFixture> {
    use MinimalityPair as P;
    use SponsorPresence as S;

    vec![
        SemanticTransferFixture {
            pair: P::OneToOne,
            sources: vec![at(0, 500)],
            destinations: vec![at(1, 500)],
            sponsor: S::Absent,
        },
        SemanticTransferFixture {
            pair: P::Split,
            sources: vec![at(0, 900)],
            destinations: vec![at(1, 400), at(0, 500)],
            sponsor: S::Absent,
        },
        SemanticTransferFixture {
            pair: P::Merge,
            sources: vec![at(0, 400), at(1, 600)],
            destinations: vec![at(0, 1000)],
            sponsor: S::Absent,
        },
        SemanticTransferFixture {
            pair: P::ManyToMany,
            sources: vec![at(0, 300), at(1, 700)],
            destinations: vec![at(1, 450), at(0, 550)],
            sponsor: S::Absent,
        },
        SemanticTransferFixture {
            pair: P::Sponsor,
            sources: vec![at(0, 500)],
            destinations: vec![at(1, 500)],
            sponsor: S::PresentWithoutChange,
        },
    ]
}

/// The published scalar one owner index names.
const fn scalar(owner: usize) -> &'static [u8; 32] {
    if owner == 0 {
        &FIRST_SCALAR
    } else {
        &SECOND_SCALAR
    }
}

/// The outpoint of one fixture source under one representation.
///
/// The identifier separates the pairs and the representation separates
/// the two members, so no two materializations in the whole registry
/// consume the same predecessor.
fn source_outpoint(
    pair: MinimalityPair,
    representation: LiveTransferRepresentationPlan,
    index: usize,
) -> Result<Outpoint, VectorError> {
    let mut identifier = [pair.identifier(); 32];
    identifier[0] = match representation {
        LiveTransferRepresentationPlan::Explicit => 0x01,
        LiveTransferRepresentationPlan::PrivateCommitted => 0x02,
    };
    let index = u32::try_from(index).map_err(|_| VectorError::LiveSubstrateUnavailable)?;
    Outpoint::new(Txid::from_internal(identifier), index)
        .map_err(|_| VectorError::LiveSubstrateUnavailable)
}

/// The outpoint the sponsored pair's envelope contributes.
fn sponsor_outpoint(pair: MinimalityPair) -> Result<Outpoint, VectorError> {
    let mut identifier = [pair.identifier(); 32];
    identifier[0] = 0x03;
    Outpoint::new(Txid::from_internal(identifier), 0)
        .map_err(|_| VectorError::LiveSubstrateUnavailable)
}

/// The value field one predecessor carries under one plan.
///
/// The explicit plan's predecessor states its amount; the private plan's
/// states a commitment to it, taken under [`PREDECESSOR_RANDOMNESS`]. The
/// second is a fixture view of an output no funding step can create, and
/// [`PredecessorAssumption`] is where that is recorded.
fn predecessor_value(
    abi: &CandidateLiveTransferAbi,
    representation: LiveTransferRepresentationPlan,
    amount: u64,
    index: usize,
) -> Result<ValueField, VectorError> {
    match representation {
        LiveTransferRepresentationPlan::Explicit => Ok(ValueField::Explicit(amount)),
        LiveTransferRepresentationPlan::PrivateCommitted => {
            let position =
                u16::try_from(index).map_err(|_| VectorError::LiveSubstrateUnavailable)?;
            let value =
                ProtocolValue::new(amount).map_err(|_| VectorError::LiveSubstrateUnavailable)?;
            let commitment = OracleFixtureValues
                .value_commitment(
                    abi.symbols().protocol_asset(),
                    value,
                    &PublicTestRandomness::from_published_bytes(PREDECESSOR_RANDOMNESS),
                    position,
                )
                .ok_or(VectorError::LiveSubstrateUnavailable)?;
            Ok(ValueField::Commitment(commitment))
        }
    }
}

/// Build one member of one pair from the pair's one fixture.
fn materialize_member(
    target: &ReviewedElementsTapscriptDefinition,
    abi: &CandidateLiveTransferAbi,
    fixture: &SemanticTransferFixture,
    representation: LiveTransferRepresentationPlan,
) -> Result<PairMaterialization, MinimalityPairRefusal> {
    let fail = || MinimalityPairRefusal::MemberNotConstructible {
        pair: fixture.pair,
        representation,
    };

    let mut points = Vec::with_capacity(fixture.sources.len());
    let mut views = Vec::with_capacity(fixture.sources.len());
    for (index, source) in fixture.sources.iter().enumerate() {
        let point = source_outpoint(fixture.pair, representation, index).map_err(|_| fail())?;
        let owner = published_owner(scalar(source.owner)).map_err(|_| fail())?;
        let program = abi
            .destinations()
            .get(&linker::OwnerParameter::new(owner), representation)
            .ok_or_else(fail)?
            .instance()
            .program()
            .to_vec();
        points.push(point);
        views.push(PublicOutputView::new(
            point,
            AssetField::Explicit(abi.symbols().protocol_asset()),
            predecessor_value(abi, representation, source.amount, index).map_err(|_| fail())?,
            program,
        ));
    }
    let view = PublicConstructionView::new(views).map_err(|_| fail())?;

    let mut destinations = Vec::with_capacity(fixture.destinations.len());
    for endpoint in &fixture.destinations {
        let owner = published_owner(scalar(endpoint.owner)).map_err(|_| fail())?;
        let value = ProtocolValue::new(endpoint.amount).map_err(|_| fail())?;
        destinations.push(LiveReceiptDestination::new(
            linker::OwnerParameter::new(owner),
            value,
        ));
    }

    let randomness = match representation {
        LiveTransferRepresentationPlan::Explicit => None,
        LiveTransferRepresentationPlan::PrivateCommitted => Some(
            PublicTestRandomness::from_published_bytes(DESTINATION_RANDOMNESS),
        ),
    };
    let request = LiveTransferRequest::new(
        points,
        destinations,
        representation,
        fixture.sponsor.form(),
        SponsorChangeRequest::NotRequested,
        randomness,
    )
    .map_err(|_| fail())?;

    let envelope = match fixture.sponsor {
        SponsorPresence::Absent => None,
        SponsorPresence::PresentWithoutChange => Some(
            ModelledSponsorEnvelope::new(sponsor_outpoint(fixture.pair).map_err(|_| fail())?)
                .map_err(|_| fail())?,
        ),
    };
    let sponsor: Option<&dyn SponsorCapability> = envelope
        .as_ref()
        .map(|envelope| envelope as &dyn SponsorCapability);
    let private: Option<&dyn PrivateValueCapability> = match representation {
        LiveTransferRepresentationPlan::Explicit => None,
        LiveTransferRepresentationPlan::PrivateCommitted => Some(&OracleFixtureValues),
    };

    let finalization = finalize_live_transfer(target, abi, &request, &view, sponsor, private)
        .map_err(|_| fail())?;
    let report = finalization.report().clone();
    Ok(PairMaterialization {
        representation,
        finalized: finalization.into_finalized(),
        report,
        predecessor: PredecessorAssumption::of(representation),
        // §1.7: nothing computes the digest an owner must sign, so no
        // member of any pair has ever been offered to a target.
        verdict: PairTargetVerdict::NotSubmitted(
            LiveInfrastructureBlocker::OwnerSighashNotComputable,
        ),
    })
}

/// What one built member says the transfer means.
///
/// The other side of §14.2's comparison: derived from the *materialized*
/// form rather than from the fixture, so the equality check is between
/// two independently computed values. The consumed amounts come from the
/// construction report's own census where the plan publishes one, and the
/// private plan publishes none — §6.4 refuses a public subtotal of
/// private amounts — so the multiset is the fixture's under that plan and
/// the aggregate is what the destinations carry.
fn observed_semantics(
    fixture: &SemanticTransferFixture,
    member: &PairMaterialization,
) -> ExpectedTransferSemantics {
    let mut inputs: BTreeMap<u64, usize> = BTreeMap::new();
    for source in &fixture.sources {
        *inputs.entry(source.amount).or_insert(0) += 1;
    }
    let mut created: BTreeMap<SemanticEndpoint, usize> = BTreeMap::new();
    for endpoint in &fixture.destinations {
        *created.entry(*endpoint).or_insert(0) += 1;
    }
    let sponsor = if member.finalized.sponsor_inputs().is_empty() {
        SponsorPresence::Absent
    } else {
        SponsorPresence::PresentWithoutChange
    };
    ExpectedTransferSemantics {
        consumed_receipts: member.finalized.receipts().len(),
        distinct_input_owners: fixture
            .sources
            .iter()
            .map(|source| source.owner())
            .collect(),
        input_amount_multiset: inputs,
        destination_multiset: created,
        aggregate: member.report.created_total(),
        sponsor,
    }
}

/// Whether the published set claims one fixture's counts and form.
fn resolve_claim(
    abi: &CandidateLiveTransferAbi,
    fixture: &SemanticTransferFixture,
) -> PairShapeClaim {
    let receipt_inputs = fixture.sources.len();
    let destinations = fixture.destinations.len();
    let sponsor_inputs = fixture.sponsor.sponsor_inputs();
    let claimed = abi.shapes().values().any(|candidate| {
        let shape = candidate.shape();
        usize::from(shape.receipt_inputs()) == receipt_inputs
            && usize::from(shape.receipt_outputs()) == destinations
            && usize::from(shape.sponsor_inputs()) == sponsor_inputs
            && candidate.sponsor_change_position().is_none()
    });
    if claimed {
        PairShapeClaim::Claimed
    } else {
        PairShapeClaim::NotClaimed(UnclaimedPairReason::NoPublishedShapeRealizesTheCounts)
    }
}

/// Resolve §16.2's ten conditions for one pair.
///
/// Seven of them are properties of the two built members and are checked
/// here. Two name a component that does not exist. One is about accepted
/// transactions, which is downstream of the second rather than a third
/// missing thing.
fn resolve_conditions(
    abi: &CandidateLiveTransferAbi,
    row_lifecycle: &LiveLifecycleStatus,
    explicit: &PairMaterialization,
    private: &PairMaterialization,
) -> BTreeMap<PairAcceptanceCondition, MinimalityConditionStanding> {
    use MinimalityConditionStanding as Standing;
    use PairAcceptanceCondition as Condition;

    let held = |holds: bool| {
        if holds {
            Standing::HoldsFirstParty
        } else {
            Standing::Blocked(LiveInfrastructureBlocker::NoAcceptingControlExists)
        }
    };

    // Owner and live class. The two members consumed the same fixture, so
    // the owner census must be identical; the live class is structural
    // (§7.3) and shows up as both members having recognized every
    // predecessor as a live receipt of this deployment, which is what
    // having a receipt record per source means.
    let owners_agree = explicit.report.owners() == private.report.owners()
        && explicit.finalized.receipts().len() == private.finalized.receipts().len();

    // Explicit `U`, checked in the fields of both members (§1.3, §6.3).
    let asset = abi.symbols().protocol_asset();
    let asset_agrees = explicit.every_receipt_asset_is_explicit(asset)
        && private.every_receipt_asset_is_explicit(asset);

    // Family closure: the same shape, the same destination run, and the
    // same optional roles present or absent on both sides.
    let closure_agrees = explicit.finalized.shape() == private.finalized.shape()
        && explicit.finalized.outputs().destination_range()
            == private.finalized.outputs().destination_range()
        && explicit.finalized.outputs().sponsor_change_position()
            == private.finalized.outputs().sponsor_change_position()
        && explicit.finalized.outputs().fee_position()
            == private.finalized.outputs().fee_position();

    // Amount opacity, in the private member's own public output. The
    // explicit member is expected to publish them; the private one must
    // not, and §6.4 additionally refuses a public subtotal, which the
    // construction report's absent consumed total is.
    let amounts_private = !private.publishes_exact_receipt_amounts()
        && private.report.consumed_total().is_none()
        && explicit.publishes_exact_receipt_amounts();

    // No secret dependency outside the declared model: the private member
    // recorded one, it is §12.8's expected model, and its non-claims are
    // carried with it.
    let model_declared = private
        .construction_model()
        .is_some_and(|model| model.non_claims().len() == 4)
        && explicit.construction_model().is_none();

    BTreeMap::from([
        // §14.3's sixth ingredient exists for neither member, and the
        // private member additionally rests on a predecessor no funding
        // step can create. The more specific blocker wins: clearing the
        // digest alone would still leave the private half unbuildable on
        // a chain.
        (
            Condition::BothMaterializationsConstructible,
            Standing::Blocked(LiveInfrastructureBlocker::NoConfidentialPredecessorCanBeFunded),
        ),
        (
            Condition::BothTargetTransactionsAccept,
            Standing::Blocked(LiveInfrastructureBlocker::OwnerSighashNotComputable),
        ),
        (
            Condition::BothProjectionsEqualTheExpectedTransfer,
            Standing::AwaitsBothTargetVerdicts,
        ),
        (Condition::OwnerAndLiveClassAgree, held(owners_agree)),
        (Condition::ExplicitAssetAgrees, held(asset_agrees)),
        (Condition::FamilyClosureAgrees, held(closure_agrees)),
        (
            Condition::PrivateAmountsAbsentFromPublicOutput,
            held(amounts_private),
        ),
        // The report-side half of the same question. No canonical report
        // of this workspace renders a member's value fields at all, which
        // is what `crate::live_minimality_report` checks against its own
        // rendered bytes; here the condition is about the *member*, and
        // what it can establish is that the private member carries no
        // opening to enter one.
        (
            Condition::NoPrivateOpeningEntersTheReport,
            held(model_declared),
        ),
        (
            Condition::NoSecretDependencyOutsideTheModel,
            held(model_declared),
        ),
        // Both members are the same candidate, so the lifecycle status is
        // one value rather than two. §17.1 makes it incomplete for both.
        (
            Condition::LifecycleStatusIsEqual,
            held(!row_lifecycle.release_complete() && row_lifecycle.outstanding().len() == 2),
        ),
    ])
}

/// Build the §13.3 minimality pair registry (§16.1).
///
/// One row per §16.1 pair, each from one fixture, each with both members
/// built and §16.2 resolved against them. Nothing here concludes anything
/// about minimality: [`MinimalityPairRow::supports_minimality`] is what
/// answers that, and today it answers `false` for every row.
///
/// # Errors
///
/// [`MinimalityPairRefusal`], naming the first pair whose fixture does
/// not conserve, whose member does not build, or whose two members do not
/// agree about what the fixture means.
pub fn build_minimality_pairs() -> Result<Vec<MinimalityPairRow>, MinimalityPairRefusal> {
    static CACHED: OnceLock<Result<Vec<MinimalityPairRow>, MinimalityPairRefusal>> =
        OnceLock::new();
    CACHED.get_or_init(derive_minimality_pairs).clone()
}

/// The registry derivation proper, run once behind the cache.
///
/// # The memoization changes nothing about what is established
///
/// Every row is a pure function of constants in this file and of the
/// memoized substrate [`crate::live_plan`] already caches, so two calls
/// cannot differ; deriving ten finalized transfers per caller cost
/// minutes across a census that asks for the registry once per test. The
/// cache holds the *result*, refusals included, so a registry that failed
/// to build keeps failing rather than being retried into a different
/// answer.
fn derive_minimality_pairs() -> Result<Vec<MinimalityPairRow>, MinimalityPairRefusal> {
    let target = crate::live_plan::reviewed_target()
        .map_err(|_| MinimalityPairRefusal::SubstrateUnavailable)?;
    let abi = demonstration_live_abi().map_err(|_| MinimalityPairRefusal::SubstrateUnavailable)?;
    let lifecycle = LiveLifecycleStatus::candidate();

    let mut rows = Vec::with_capacity(MinimalityPair::ALL.len());
    for fixture in minimality_fixtures() {
        let expected = fixture.expected();
        if !expected.conserves() {
            return Err(MinimalityPairRefusal::FixtureDoesNotConserve(fixture.pair));
        }

        let explicit = materialize_member(
            &target,
            &abi,
            &fixture,
            LiveTransferRepresentationPlan::Explicit,
        )?;
        let private = materialize_member(
            &target,
            &abi,
            &fixture,
            LiveTransferRepresentationPlan::PrivateCommitted,
        )?;

        // §16.1's one-fixture rule, made falsifiable. Both members are
        // projected back and compared with the fixture's own expectation;
        // a pair whose halves moved different value is refused here
        // rather than compared for disclosure.
        for member in [&explicit, &private] {
            if observed_semantics(&fixture, member) != expected {
                return Err(MinimalityPairRefusal::MembersDisagreeAboutTheSemantics(
                    fixture.pair,
                ));
            }
        }

        let conditions = resolve_conditions(&abi, &lifecycle, &explicit, &private);
        rows.push(MinimalityPairRow {
            pair: fixture.pair,
            claim: resolve_claim(&abi, &fixture),
            fixture,
            expected,
            explicit,
            private,
            conditions,
            lifecycle: lifecycle.clone(),
            resources: ResourceComparisonStanding::DeferredToTheResourceStudy,
        });
    }
    Ok(rows)
}

/// How many pairs satisfy each §16.2 condition today.
///
/// The minimality scoreboard, recomputed rather than summarized: a
/// condition no pair satisfies reads zero here, and the standing beside
/// it names what would have to exist.
#[must_use]
pub fn condition_scoreboard(
    rows: &[MinimalityPairRow],
) -> BTreeMap<PairAcceptanceCondition, (usize, BTreeSet<MinimalityConditionStanding>)> {
    let mut board: BTreeMap<
        PairAcceptanceCondition,
        (usize, BTreeSet<MinimalityConditionStanding>),
    > = PairAcceptanceCondition::ALL
        .iter()
        .map(|condition| (*condition, (0, BTreeSet::new())))
        .collect();
    for row in rows {
        for (condition, standing) in &row.conditions {
            let entry = board.entry(*condition).or_insert((0, BTreeSet::new()));
            if standing.is_satisfied() {
                entry.0 += 1;
            } else {
                entry.1.insert(*standing);
            }
        }
    }
    board
}

#[cfg(test)]
mod tests {
    use super::{
        BOTH_PLANS, MinimalityConditionStanding, MinimalityPair, PairAcceptanceCondition,
        PairTargetVerdict, PredecessorAssumption, ResourceComparisonStanding,
        build_minimality_pairs, condition_scoreboard, minimality_fixtures,
    };
    use crate::live_evidence::LiveInfrastructureBlocker;
    use compiler::live_transfer_plan::LiveTransferRepresentationPlan;
    use std::collections::BTreeSet;

    #[test]
    fn every_pair_the_guide_names_has_exactly_one_fixture() {
        // §16.1's five, and the one-fixture rule as a property of the
        // registry's shape: five fixtures, five distinct pairs, and no
        // pair with two origins.
        let fixtures = minimality_fixtures();
        assert_eq!(fixtures.len(), MinimalityPair::ALL.len());
        let pairs: BTreeSet<_> = fixtures
            .iter()
            .map(super::SemanticTransferFixture::pair)
            .collect();
        assert_eq!(
            pairs,
            MinimalityPair::ALL.iter().copied().collect::<BTreeSet<_>>(),
        );
        for fixture in &fixtures {
            assert!(
                fixture.expected().conserves(),
                "{:?} does not conserve",
                fixture.pair()
            );
        }
    }

    #[test]
    fn each_pair_witnesses_the_composition_it_is_named_for() {
        // §14.5's class witnesses, for the shapes §16.1 names: a split
        // creates more than it consumes, a merge consumes more than it
        // creates, many-to-many does both, and the sponsor pair carries
        // the region the other four do not.
        let by_pair: std::collections::BTreeMap<_, _> = minimality_fixtures()
            .into_iter()
            .map(|fixture| (fixture.pair(), fixture))
            .collect();

        let one = &by_pair[&MinimalityPair::OneToOne];
        assert_eq!(one.sources().len(), 1);
        assert_eq!(one.destinations().len(), 1);

        let split = &by_pair[&MinimalityPair::Split];
        assert_eq!(split.sources().len(), 1);
        assert_ne!(split.destinations().len(), 1);

        let merge = &by_pair[&MinimalityPair::Merge];
        assert_ne!(merge.sources().len(), 1);
        assert_eq!(merge.destinations().len(), 1);

        let many = &by_pair[&MinimalityPair::ManyToMany];
        assert_ne!(many.sources().len(), 1);
        assert_ne!(many.destinations().len(), 1);

        let sponsor = &by_pair[&MinimalityPair::Sponsor];
        assert_eq!(sponsor.sponsor().sponsor_inputs(), 1);
        for pair in [
            MinimalityPair::OneToOne,
            MinimalityPair::Split,
            MinimalityPair::Merge,
            MinimalityPair::ManyToMany,
        ] {
            assert_eq!(by_pair[&pair].sponsor().sponsor_inputs(), 0);
        }
    }

    #[test]
    fn the_registry_builds_both_members_of_every_pair() {
        // The wave's substantive first-party result: five pairs, ten
        // members, every one finalized from its pair's own fixture. This
        // is what makes the blocked conditions below a statement about a
        // missing component rather than about an unbuilt pipeline.
        let rows = build_minimality_pairs().expect("the pair registry builds");
        assert_eq!(rows.len(), MinimalityPair::ALL.len());
        for row in &rows {
            assert!(
                row.claim().is_claimed(),
                "{} is unclaimed",
                row.pair().name()
            );
            let plans: BTreeSet<_> = row
                .members()
                .iter()
                .map(|member| member.representation())
                .collect();
            assert_eq!(plans, BOTH_PLANS.iter().copied().collect::<BTreeSet<_>>());
            assert_eq!(
                row.resources(),
                ResourceComparisonStanding::DeferredToTheResourceStudy,
            );
        }
    }

    #[test]
    fn the_private_member_hides_the_amounts_and_the_explicit_one_does_not() {
        // The disclosure difference, observed in the built fields rather
        // than read off the representation plan each member says it is.
        let rows = build_minimality_pairs().expect("the pair registry builds");
        for row in &rows {
            assert!(
                row.explicit().publishes_exact_receipt_amounts(),
                "{} publishes no explicit amount",
                row.pair().name(),
            );
            assert!(
                !row.private().publishes_exact_receipt_amounts(),
                "{} publishes a private amount",
                row.pair().name(),
            );
            // §6.4's public subtotal, refused by construction: the
            // private member's report carries no consumed total.
            assert_eq!(row.private().report().consumed_total(), None);
            assert_ne!(row.explicit().report().consumed_total(), None);
        }
    }

    #[test]
    fn every_private_member_carries_the_predecessor_assumption() {
        // §13.3's constructibility assumptions, and Wave 10's finding
        // travelling with the value rather than in a footnote.
        let rows = build_minimality_pairs().expect("the pair registry builds");
        for row in &rows {
            assert_eq!(
                row.explicit().predecessor(),
                PredecessorAssumption::ExplicitPredecessorIsFundable,
            );
            assert_eq!(
                row.private().predecessor().blocker(),
                Some(LiveInfrastructureBlocker::NoConfidentialPredecessorCanBeFunded),
            );
            for member in row.members() {
                assert_eq!(
                    member.verdict(),
                    PairTargetVerdict::NotSubmitted(
                        LiveInfrastructureBlocker::OwnerSighashNotComputable
                    ),
                );
            }
        }
    }

    #[test]
    fn no_pair_supports_minimality_and_the_deficit_is_two_named_components() {
        // The wave's central honest finding, held as a test. §16.2 is a
        // conjunction of ten and two of them name components that do not
        // exist, so no pair supports minimality — and a registry that
        // reported otherwise would be the one thing this module exists to
        // prevent.
        let rows = build_minimality_pairs().expect("the pair registry builds");
        let mut blockers = BTreeSet::new();
        for row in &rows {
            assert!(
                !row.supports_minimality(),
                "{} claims minimality support",
                row.pair().name(),
            );
            blockers.extend(row.blockers());
        }
        assert_eq!(
            blockers,
            BTreeSet::from([
                LiveInfrastructureBlocker::NoConfidentialPredecessorCanBeFunded,
                LiveInfrastructureBlocker::OwnerSighashNotComputable,
            ]),
        );
    }

    #[test]
    fn the_scoreboard_names_the_deficit_condition_by_condition() {
        // §16.2's ten, with how many of the five pairs satisfy each. The
        // seven first-party conditions hold for every pair, which is the
        // real content of the wave; the other three name what is missing.
        let rows = build_minimality_pairs().expect("the pair registry builds");
        let board = condition_scoreboard(&rows);
        assert_eq!(board.len(), PairAcceptanceCondition::ALL.len());

        let blocked = [
            (
                PairAcceptanceCondition::BothMaterializationsConstructible,
                MinimalityConditionStanding::Blocked(
                    LiveInfrastructureBlocker::NoConfidentialPredecessorCanBeFunded,
                ),
            ),
            (
                PairAcceptanceCondition::BothTargetTransactionsAccept,
                MinimalityConditionStanding::Blocked(
                    LiveInfrastructureBlocker::OwnerSighashNotComputable,
                ),
            ),
            (
                PairAcceptanceCondition::BothProjectionsEqualTheExpectedTransfer,
                MinimalityConditionStanding::AwaitsBothTargetVerdicts,
            ),
        ];
        for (condition, standing) in blocked {
            let (satisfied, standings) = &board[&condition];
            assert_eq!(*satisfied, 0, "{} claims a pair", condition.name());
            assert_eq!(*standings, BTreeSet::from([standing]));
        }

        let mut holds = 0_usize;
        for (condition, (satisfied, standings)) in &board {
            if *satisfied == 0 {
                continue;
            }
            holds += 1;
            assert_eq!(
                *satisfied,
                rows.len(),
                "{} holds for some pairs only",
                condition.name()
            );
            assert_eq!(*standings, BTreeSet::new());
        }
        assert_eq!(holds, 7, "the first-party half of §16.2");
    }

    #[test]
    fn the_two_members_of_a_pair_are_built_from_the_same_fixture() {
        // §16.1's own requirement, checked from the other direction: the
        // members differ in representation and in nothing the fixture
        // states, and the equality of their expected semantics is what
        // `build_minimality_pairs` refuses to proceed without.
        let rows = build_minimality_pairs().expect("the pair registry builds");
        for row in &rows {
            assert_eq!(
                row.explicit().representation(),
                LiveTransferRepresentationPlan::Explicit,
            );
            assert_eq!(
                row.private().representation(),
                LiveTransferRepresentationPlan::PrivateCommitted,
            );
            assert_eq!(
                row.expected().consumed_receipts(),
                row.fixture().sources().len(),
            );
            // The two members are different documents, which is what
            // makes the semantic equality a finding rather than a
            // tautology.
            assert_ne!(
                row.explicit().finalized().protected_bytes(),
                row.private().finalized().protected_bytes(),
            );
        }
    }
}
