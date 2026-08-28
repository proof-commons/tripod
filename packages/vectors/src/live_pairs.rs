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
//! step of THIS pipeline can produce the output that view describes:
//! [`LiveInfrastructureBlocker::NoConfidentialPredecessorCanBeFunded`].
//! So [`PredecessorAssumption`] travels with every private
//! materialization rather than being mentioned in a report footnote, and
//! §16.2's first condition reads it.
//!
//! That word has stopped being true, and the blocker no longer stands on
//! any of §16.2's conditions. A confidential predecessor IS fundable:
//! the confidential funding arm exists, a deterministic materializer
//! builds the exact explicit-asset/confidential-value form, one
//! predecessor of that form has been submitted, accepted, mined, and
//! read back raw ([`crate::confidential_predecessor`]), and the private
//! lane has since spent confidential predecessors in six accepted
//! shapes. The guide's own closeout carries
//! `NoConfidentialPredecessorCanBeFunded` in its CLEARED set.
//!
//! [`PredecessorAssumption`] still travels with every private
//! materialization, because what it records — that this pipeline states
//! its confidential inputs as fixture views rather than consuming a
//! predecessor it funded — remains exactly true. What changed is that
//! this is no longer a component's absence and is therefore no longer a
//! blocker. This pipeline does not call the private lane's own entry
//! point; that is an attempt not yet made, and §16.2's ninth condition
//! is where the declared test construction model is accounted for.
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
use transaction::live_construct::{
    LiveConstructionReport, complete_live_transfer, finalize_live_transfer,
};
use transaction::live_finalize::FinalizedLiveTransfer;
use transaction::live_private::{PrivateValueCapability, SelectedConstructionModel};
use transaction::live_request::{
    LiveReceiptDestination, LiveTransferRequest, ProtocolValue, PublicTestRandomness,
    RequestedForm, SponsorChangeRequest,
};
use transaction::live_signing::{LiveOwnerResponse, authorize_live_transfer};
use transaction::sponsor::{
    SponsorCapability, SponsorOffer, SponsorSignature, SponsorSigningRequest,
};
use transaction::view::{PublicConstructionView, PublicOutputView};

use crate::error::VectorError;
use crate::live_capability::OracleFixtureValues;
use crate::live_evidence::{LiveInfrastructureBlocker, UNAUTHORIZING_SIGNATURE};
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
    /// One endpoint: a published owner index and an exact amount.
    ///
    /// Public because the pairs ARC states its own fixture in its own
    /// module and has to state it in THIS type. A second endpoint type
    /// would be a second definition of what a semantic fixture is, and
    /// §16.1's requirement that both members begin from one fixture is
    /// checkable only while there is one.
    #[must_use]
    pub const fn new(owner: usize, amount: u64) -> Self {
        Self { owner, amount }
    }

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
    /// One fixture, stated by a caller that materializes it itself.
    ///
    /// The registry below states five of these and submits none of them.
    /// The pairs arc states ONE and submits both of its materializations,
    /// so the constructor is public — and it is the same constructor, so
    /// the arc's fixture is the same KIND of object the registry's are
    /// and answers [`Self::expected`] and
    /// [`ExpectedTransferSemantics::conserves`] the same way.
    #[must_use]
    pub const fn stated(
        pair: MinimalityPair,
        sources: Vec<SemanticEndpoint>,
        destinations: Vec<SemanticEndpoint>,
        sponsor: SponsorPresence,
    ) -> Self {
        Self {
            pair,
            sources,
            destinations,
            sponsor,
        }
    }

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
/// cannot be discharged through THIS boundary.
///
/// "Through this boundary" is now the whole of the claim, and it is a
/// smaller claim than it was. A confidential predecessor is fundable and
/// one has been funded, mined, and read back through the confidential
/// funding arm. What has not happened is this pipeline consuming one:
/// its funding step has no confidential form, and its private
/// materialization does not take the transaction-wide path that would
/// consume an opening. The assumption is therefore still carried, and it
/// is carried for a reason that no longer has a name of its own in this
/// vocabulary.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PredecessorAssumption {
    /// An explicit live receipt at the deployment's own program, which a
    /// funding step can create and has.
    ExplicitPredecessorIsFundable,
    /// A confidential live receipt, which no funding step of this
    /// pipeline can create.
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
    /// This member was not submitted, and a recorded run of ITS OWN
    /// SHAPE was accepted at this identity.
    ///
    /// The first half of that sentence is the load-bearing one. This
    /// variant exists so that a member with shape evidence behind it
    /// stops carrying a blocker that has been cleared, WITHOUT the
    /// registry acquiring a way to say a member was accepted. There is
    /// no such variant, and there is none because no member was.
    NotSubmittedShapeAcceptedElsewhere {
        /// The accepted run's case name.
        case: &'static str,
        /// The identity the target computed for that run.
        accepted_identity: &'static str,
    },
    /// This member was not submitted and no recorded run has its shape.
    NotSubmittedNoRunOfThisShape {
        /// How the nearest recorded run differs.
        because: &'static str,
    },
    /// THIS MEMBER was submitted, and a target ACCEPTED it.
    ///
    /// The variant this type's own doc says does not exist, and the doc
    /// was right for as long as it stood: no member had been submitted,
    /// and a registry with a way to say one had would have been an
    /// invitation to fill it with a shape sibling's acceptance.
    ///
    /// It is minted with the ONE pair whose members were submitted. The
    /// pairs arc materialized this registry's own one-to-one fixture
    /// twice and handed both materializations to one node against one
    /// issued asset ([`crate::live_pair_arc`]). What separates it from
    /// [`Self::NotSubmittedShapeAcceptedElsewhere`] is exactly the
    /// distinction that member exists to draw, and the two are still
    /// spelled apart: this one does NOT begin `not-submitted`, because
    /// the half of that sentence which was equally true of every member
    /// has stopped being true of these two.
    Accepted {
        /// The arc case that submitted it.
        case: &'static str,
        /// The identity the target computed for it.
        accepted_identity: &'static str,
    },
}

impl PairTargetVerdict {
    /// The verdict's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::NotSubmitted(_) => "not-submitted",
            // Three spellings and not one, because a reader of the
            // rendered word has to be able to tell a member with shape
            // evidence behind it from a member with none. All three
            // begin `not-submitted`, which is the half that is equally
            // true of every member.
            Self::NotSubmittedShapeAcceptedElsewhere { .. } => {
                "not-submitted-shape-accepted-elsewhere"
            }
            Self::NotSubmittedNoRunOfThisShape { .. } => "not-submitted-no-run-of-this-shape",
            // The one spelling that does not begin `not-submitted`,
            // because the member it names was.
            Self::Accepted { .. } => "accepted",
        }
    }
}

/// Where a pair's resource comparison stands.
///
/// # This module's reading of §13.3's row shape
///
/// §13.3 lists nine things each minimality row contains, and the ninth is
/// a resource comparison. Until §18's study existed there was nothing to
/// put there, and this field held a typed non-claim saying so. The study
/// exists now, and that non-claim is gone rather than kept beside the
/// measurement — a standing that says "no measurement exists" is false
/// once one does, and leaving it available would let a later row defer to
/// a study that has already reported.
///
/// # Why a measured comparison still carries a caveat in its own name
///
/// Because the private member's weight is a weight of a serialization
/// whose proof slots are *empty*. §12.8's construction model commits
/// values and produces no range proof, and the encoder writes one empty
/// range prefix per output — so the figure is exact for the bytes this
/// workspace can build and is not the figure a confidential transfer
/// would carry. The variant is named for that, so a reader cannot take
/// the number without the condition on it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ResourceComparisonStanding {
    /// Both members were completed and weighed, and the private
    /// serialization counts no confidential proof.
    MeasuredWithNoConfidentialProof {
        /// The explicit member's complete transaction weight.
        explicit_weight: u64,
        /// The private member's complete transaction weight, over a
        /// serialization whose proof slots are empty.
        private_weight: u64,
    },
    /// One member has no complete transaction, and this is why.
    ///
    /// A finalized transfer is not a complete one: every witness position
    /// has to be filled before there are bytes to weigh, and a member
    /// whose sponsor suffix nothing can authorize has none. The blocker
    /// names the component, so a reader can tell a pair that costs
    /// nothing to measure from one that could not be measured.
    NoCompleteTransactionExists(LiveInfrastructureBlocker),
}

impl ResourceComparisonStanding {
    /// The standing's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::MeasuredWithNoConfidentialProof { .. } => "measured-with-no-confidential-proof",
            Self::NoCompleteTransactionExists(_) => "no-complete-transaction-exists",
        }
    }

    /// The two weights, where both members were weighed.
    #[must_use]
    pub const fn weights(self) -> Option<(u64, u64)> {
        match self {
            Self::MeasuredWithNoConfidentialProof {
                explicit_weight,
                private_weight,
            } => Some((explicit_weight, private_weight)),
            Self::NoCompleteTransactionExists(_) => None,
        }
    }
}

/// The sponsor envelope the sponsored pair is built with.
///
/// It offers one input and the fee, states no change destination, and
/// declines to sign. Declining is the honest behaviour rather than a stub:
/// §1.9 puts the sponsor's own authorization outside protocol data and it
/// arrives through an adapter, and no adapter signer is wired into this
/// evidence lane — [`LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent`].
/// An envelope that returned bytes here would be modelling the signer this
/// lane does not have.
///
/// That blocker is no longer a CARRIED residual of the evidence plan, a
/// target having accepted a sponsored control elsewhere, and the name is
/// still the right one here for the reason the variant's own doc gives:
/// it names a condition, and this lane is in it. What would be wrong is
/// to read the cleared residual as meaning THIS envelope now signs. It
/// does not, and a modelled signature would be exactly the fabrication
/// the declining behaviour exists to refuse.
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
    complete_weight: Option<u64>,
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

    /// This member's complete transaction weight, where one exists
    /// (§18.2).
    ///
    /// §13.3's ninth field needs a figure, and a figure needs a complete
    /// transaction: a finalized transfer has no witnesses attached and so
    /// has no weight. Every witness position is filled with
    /// [`crate::live_evidence::UNAUTHORIZING_SIGNATURE`] to reach one,
    /// which is what makes the number a real weight and not a claim about
    /// any signature.
    ///
    /// Absent for a member no signer can complete at all — the sponsored
    /// pair, whose suffix needs an authorization
    /// [`LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent`] records
    /// as having no producer IN THIS LANE. Absent rather than zero, on
    /// §18.4's rule.
    ///
    /// The residual has cleared for the evidence plan and this figure is
    /// still absent, which is not a contradiction: a producer exists and
    /// is not wired here, so this lane still completes no sponsored
    /// member and has no weight to report. Reporting one would mean
    /// weighing a transaction whose sponsor witness this lane invented.
    #[must_use]
    pub const fn complete_weight(&self) -> Option<u64> {
        self.complete_weight
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

/// Whether a recorded run accepted a transaction of one member's own
/// shape.
///
/// # This is a narrower fact than §16.2's second condition asks for
///
/// §16.2 asks that both target transactions accept, and NO member of
/// this registry has ever been submitted to a target. §1.7 leaves the
/// owner digest to the target, this pipeline offers nothing to one, and
/// [`PairMaterialization::verdict`] says so for every member.
///
/// What the campaign has instead is an acceptance per SHAPE: real
/// transactions of each shape, submitted to a real node, accepted,
/// mined, and recorded at the identity the target itself computed. That
/// is also the repo's own standard for moving a row — an observed
/// acceptance of the row's own shape and nothing else — so it is the
/// standard applied here.
///
/// The type is spelled to keep the two apart and the distinction is the
/// point: a member whose SHAPE was accepted is not a member that was
/// accepted, and no reader of this registry should be able to slide
/// from one to the other.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PairShapeAcceptance {
    /// A recorded run of this member's exact shape was accepted.
    ObservedForThisShape {
        /// The run's own case name, so a reader can check the citation
        /// against the module that recorded it.
        case: &'static str,
        /// The identity the target computed for that run.
        accepted_identity: &'static str,
    },
    /// No recorded run has this member's shape.
    NoRunOfThisShape {
        /// The nearest recorded run and how its shape differs, so the
        /// absence is a statement rather than a silence.
        because: &'static str,
    },
    /// THIS MEMBER was submitted and accepted at this identity.
    ///
    /// The member this type's own doc says there is no way to say. There
    /// was none while no member had been submitted, and the narrowness
    /// was the point: a registry able to say a member was accepted, with
    /// nothing that had been, would have been an invitation to file a
    /// shape sibling's acceptance under it.
    ///
    /// One pair has changed that. The pairs arc materialized §16.1's
    /// one-to-one fixture TWICE — this registry's own fixture, read from
    /// the arc so there is only one — and submitted both materializations
    /// to one node against one issued asset, both accepted in the validated
    /// native-v2/revision-7 corpus. What may occupy this member is a
    /// submission of the pair's own member and nothing else;
    /// the four other pairs keep
    /// [`Self::ObservedForThisShape`], which says less and is what is
    /// true of them.
    ObservedForThisMember {
        /// The arc case that submitted it.
        case: &'static str,
        /// The identity the target computed for it.
        accepted_identity: &'static str,
    },
}

impl PairShapeAcceptance {
    /// Whether a run of this shape was accepted.
    #[must_use]
    pub const fn is_observed(self) -> bool {
        matches!(
            self,
            Self::ObservedForThisShape { .. } | Self::ObservedForThisMember { .. }
        )
    }

    /// Whether the MEMBER itself was submitted and accepted.
    ///
    /// Strictly narrower than [`Self::is_observed`], and the two are kept
    /// apart because the whole registry was built around the difference:
    /// a member whose SHAPE was accepted is not a member that was
    /// accepted, and no reader may slide from one to the other.
    #[must_use]
    pub const fn is_the_member_itself(self) -> bool {
        matches!(self, Self::ObservedForThisMember { .. })
    }

    /// The identity, where there is one.
    #[must_use]
    pub const fn accepted_identity(self) -> Option<&'static str> {
        match self {
            Self::ObservedForThisShape {
                accepted_identity, ..
            }
            | Self::ObservedForThisMember {
                accepted_identity, ..
            } => Some(accepted_identity),
            Self::NoRunOfThisShape { .. } => None,
        }
    }
}

fn current_acceptance_identity_at(ceremony: &str, index: usize, expected: usize) -> &'static str {
    let corpus = crate::live_corpus_native_v2_r7::run_of_record()
        .expect("the reviewed native-v2/revision-7 corpus validates");
    let acceptances = corpus
        .acceptance_projections(ceremony)
        .unwrap_or_else(|| panic!("the current corpus omits {ceremony}"));
    assert_eq!(
        acceptances.len(),
        expected,
        "{ceremony} carries a different current acceptance census",
    );
    acceptances
        .get(index)
        .unwrap_or_else(|| panic!("{ceremony} omits current acceptance {index}"))
        .identity_display()
}

fn current_acceptance_identity(ceremony: &str) -> &'static str {
    current_acceptance_identity_at(ceremony, 0, 1)
}

/// The recorded acceptance of one member's shape, or its absence.
///
/// A table over typed projections of the validated current corpus. Every arm
/// selects a ceremony rather than repeating an identity, so there is no second
/// transcription surface beside the archive.
///
/// Shapes are compared on what a target can see: how many receipts are
/// consumed, how many receipt outputs are created, which non-receipt
/// roles are present, and — where a pair's subject is the owners —
/// whether the consumed receipts stand under distinct owners.
///
/// # Panics
///
/// Panics only if the embedded reviewed native-v2/revision-7 corpus no
/// longer supplies the exact acceptance projection selected by this pair.
#[must_use]
pub fn recorded_acceptance(
    pair: MinimalityPair,
    representation: LiveTransferRepresentationPlan,
) -> PairShapeAcceptance {
    use LiveTransferRepresentationPlan as Plan;
    use MinimalityPair as P;
    use PairShapeAcceptance as A;

    match (pair, representation) {
        // 1 -> 1, sponsorless -- and the ONE pair whose own MEMBERS have
        // been submitted. The identities cited are the pairs arc's, and
        // what they answer is narrower and stronger than every other arm
        // here: not a run of the member's shape, but the member.
        //
        (P::OneToOne, Plan::Explicit) => A::ObservedForThisMember {
            case: "explicit-paired-one-to-one",
            accepted_identity: current_acceptance_identity_at("pairs-arc", 0, 2),
        },
        (P::OneToOne, Plan::PrivateCommitted) => A::ObservedForThisMember {
            case: "private-paired-one-to-one",
            accepted_identity: current_acceptance_identity_at("pairs-arc", 1, 2),
        },
        // 1 -> 2, both created outputs being RECEIPTS.
        (P::Split, Plan::Explicit) => A::ObservedForThisShape {
            case: "explicit-split",
            accepted_identity: current_acceptance_identity("explicit-split"),
        },
        // The private lane HAS now run this member's own shape. It did
        // not before, and the two shapes it had run near this one are
        // still not it: the recorded split creates THREE outputs, and
        // the only other one-in-two-out private run is the fee-bearing
        // shape whose second output is a FEE. Neither was answered by
        // being close, so a shape of exactly two created receipts was
        // built and submitted, and the node accepted it.
        //
        // What makes the acceptance this member's rather than another
        // near miss is measured rather than argued: the run carried TWO
        // output-witness range proofs over TWO created outputs, and a
        // fee output carries no range proof at all. Two proofs over two
        // outputs is the measured form of "both created outputs being
        // RECEIPTS".
        (P::Split, Plan::PrivateCommitted) => A::ObservedForThisShape {
            case: "private-pure-split",
            accepted_identity: current_acceptance_identity("multi-pure-split"),
        },
        // 2 -> 1.
        (P::Merge, Plan::Explicit) => A::ObservedForThisShape {
            case: "explicit-merge",
            accepted_identity: current_acceptance_identity("explicit-merge"),
        },
        (P::Merge, Plan::PrivateCommitted) => A::ObservedForThisShape {
            case: "private-merge",
            accepted_identity: current_acceptance_identity("multi-private-merge"),
        },
        // 2 -> 2, and this fixture's two consumed receipts stand under
        // two DISTINCT published owners. On the private lane the run of
        // exactly that shape is the one filed under the owners rather
        // than under the cardinality; the run the private lane files as
        // `many-to-many` creates three outputs and is a different shape
        // from this member. Citing the matching run under a name that
        // does not match the pair's is the honest way round, and it is
        // stated here rather than left for a reader to notice.
        (P::ManyToMany, Plan::Explicit) => A::ObservedForThisShape {
            case: "explicit-several-to-several",
            accepted_identity: current_acceptance_identity("explicit-several-to-several"),
        },
        (P::ManyToMany, Plan::PrivateCommitted) => A::ObservedForThisShape {
            case: "private-several-distinct-owners",
            accepted_identity: current_acceptance_identity("multi-several-owners"),
        },
        // The sponsored pair. Its explicit half is exactly the sponsor
        // lane's change-absent control: a sponsor coin funded to the fee
        // and no change role, which is what this registry's request
        // states with `SponsorChangeRequest::NotRequested`.
        (P::Sponsor, Plan::Explicit) => A::ObservedForThisShape {
            case: "sponsored-change-absent",
            accepted_identity: current_acceptance_identity("sponsored-change-absent"),
        },
        // Its private half HAS now been run as its own shape. The one
        // sponsored private successor recorded before it is still not
        // this member: that one carries a BLINDED sponsor coin, a
        // COMMITTED sponsor change, and two blinded destinations, where
        // this member states an explicit sponsor coin funded exactly to
        // the fee, no change role, and one destination.
        //
        // The arc's observation that a COMMITTED sponsor value requires
        // committed change never reached this member and never forbade
        // it: an explicit sponsor coin brings the all-zero blinder, so
        // there is nothing for a change term to absorb. Running it is
        // what settled that, and the acceptance rules the change out by
        // arithmetic rather than by inspection -- the reserve
        // sub-equation is `sponsor_input == fee + change`, this coin was
        // funded to exactly the fee, and a node that accepted it cannot
        // have been handed a change output.
        (P::Sponsor, Plan::PrivateCommitted) => A::ObservedForThisShape {
            case: "sponsored-private-explicit-no-change",
            accepted_identity: current_acceptance_identity("sponsored-private-explicit-no-change"),
        },
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
    /// Both members' SHAPES were run against a real target and accepted,
    /// at these identities.
    ///
    /// The satisfied standing for the one condition whose subject is a
    /// target verdict. It is a separate member from
    /// [`Self::HoldsFirstParty`] and not a special case of it, because
    /// what stands behind it is not a check this crate performed: it is
    /// two transactions a real node accepted and mined, recorded at the
    /// identities the node itself computed.
    ///
    /// # What it does not say
    ///
    /// That either member was submitted. Neither was, and
    /// [`PairTargetVerdict`] says so per member. This standing is the
    /// repo's own row rule applied to a pair — an observed acceptance
    /// of the shape, and nothing else — and a reader who needs the
    /// stronger claim will not find it here.
    HoldsOnObservedShapeAcceptances {
        /// The identity of the accepted run of the explicit member's
        /// shape.
        explicit_identity: &'static str,
        /// The identity of the accepted run of the private member's
        /// shape.
        private_identity: &'static str,
    },
    /// One lane has no recorded run of this pair's shape.
    ///
    /// Distinct from [`Self::AwaitsBothTargetVerdicts`], which says
    /// there are no verdicts at all. This says a verdict exists on one
    /// side and not the other, and names the side — which is the
    /// difference between a pair waiting on a pipeline and a pair
    /// waiting on ONE run.
    AwaitsARunOfThisShape {
        /// Which lane lacks it, in the plan's own spelling.
        lane: &'static str,
        /// Why no recorded run has the shape.
        because: &'static str,
    },
    /// BOTH MEMBERS were submitted to a real target and accepted, at
    /// these identities.
    ///
    /// The standing [`Self::HoldsOnObservedShapeAcceptances`]'s own doc
    /// says a reader will not find, and it was right until one pair's
    /// members were submitted. This says what that one cannot: not that
    /// two runs of the members' shapes were accepted, but that these two
    /// materializations of THIS pair's one fixture were.
    ///
    /// Only the pairs arc's pair may occupy it. The other four keep the
    /// weaker standing, which is what is true of them.
    HoldsOnAcceptedMembers {
        /// The identity the target computed for the accepted explicit
        /// member.
        explicit_identity: &'static str,
        /// The identity the target computed for the accepted private
        /// member.
        private_identity: &'static str,
    },
}

impl MinimalityConditionStanding {
    /// Whether the condition is satisfied.
    #[must_use]
    pub const fn is_satisfied(self) -> bool {
        matches!(
            self,
            Self::HoldsFirstParty
                | Self::HoldsOnObservedShapeAcceptances { .. }
                | Self::HoldsOnAcceptedMembers { .. }
        )
    }

    /// Whether what satisfies this standing is a target's verdict rather
    /// than a check this crate ran.
    ///
    /// Asked separately from [`Self::is_satisfied`] so that a report can
    /// say how much of §16.2 rests on observation and how much on
    /// first-party recomputation, which are different kinds of evidence
    /// and are never summed here.
    #[must_use]
    pub const fn rests_on_observation(self) -> bool {
        matches!(
            self,
            Self::HoldsOnObservedShapeAcceptances { .. } | Self::HoldsOnAcceptedMembers { .. }
        )
    }

    /// Whether what satisfies it is an acceptance of the MEMBERS
    /// themselves.
    ///
    /// Strictly narrower than [`Self::rests_on_observation`], and asked
    /// separately for the reason that one is asked separately from
    /// satisfaction: the two are different strengths of evidence, and a
    /// report that summed them would let a reader take a shape sibling's
    /// acceptance for a member's.
    #[must_use]
    pub const fn rests_on_accepted_members(self) -> bool {
        matches!(self, Self::HoldsOnAcceptedMembers { .. })
    }

    /// The standing's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::HoldsFirstParty => "holds-first-party",
            Self::Blocked(_) => "blocked",
            Self::AwaitsBothTargetVerdicts => "awaits-both-target-verdicts",
            Self::HoldsOnObservedShapeAcceptances { .. } => "holds-on-observed-shape-acceptances",
            Self::AwaitsARunOfThisShape { .. } => "awaits-a-run-of-this-shape",
            Self::HoldsOnAcceptedMembers { .. } => "holds-on-accepted-members",
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
                // A pair awaiting a run of its shape names NO blocker,
                // and that is the honest answer rather than an omission:
                // what it lacks is a run, and a run that has not been
                // attempted is not a component that does not exist.
                MinimalityConditionStanding::HoldsFirstParty
                | MinimalityConditionStanding::HoldsOnObservedShapeAcceptances { .. }
                | MinimalityConditionStanding::HoldsOnAcceptedMembers { .. }
                | MinimalityConditionStanding::AwaitsARunOfThisShape { .. }
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
        // The one-to-one fixture is the PAIRS ARC's, read from the arc
        // rather than restated here. This registry states five fixtures
        // and submits none of them; the arc states one and submits both
        // of its materializations, and if the two files each spelled a
        // one-to-one fixture the workspace would have TWO of them — so
        // the arc's members would be materializations of a fixture this
        // registry's row is not about. There is one, and this is where
        // the registry reads it.
        crate::live_pair_arc::pair_arc_fixture(),
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
    // The sponsor coin is stated before the view is sealed, because a
    // sponsored build reads its sponsor inputs from this same view and
    // the reserve asset is the one thing it checks about them. The
    // fixture named this outpoint in the offer long before it showed
    // the builder anything about it.
    let sponsor_point = match fixture.sponsor {
        SponsorPresence::Absent => None,
        SponsorPresence::PresentWithoutChange => {
            Some(sponsor_outpoint(fixture.pair).map_err(|_| fail())?)
        }
    };
    if let Some(point) = sponsor_point {
        views.push(PublicOutputView::new(
            point,
            AssetField::Explicit(abi.symbols().reserve_asset()),
            ValueField::Explicit(SPONSOR_FEE),
            abi.symbols().sponsor_change_program().to_vec(),
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

    let envelope = match sponsor_point {
        None => None,
        Some(point) => Some(ModelledSponsorEnvelope::new(point).map_err(|_| fail())?),
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
    let finalized = finalization.into_finalized();
    let complete_weight = complete_weight_of(target, &finalized, &report, sponsor);
    Ok(PairMaterialization {
        representation,
        finalized,
        report,
        complete_weight,
        predecessor: PredecessorAssumption::of(representation),
        // §1.7: this pipeline computes no owner digest and submits
        // nothing, so no member of any pair has ever been offered to a
        // target. That has not changed and this field keeps saying so.
        //
        // What changed is the second half of the sentence. It used to
        // name `OwnerSighashNotComputable` as the component in the way,
        // and that blocker is carried by zero rows of the census: the
        // digest IS computed and both lanes carry acceptances. A field
        // naming a cleared residual reads as a pipeline one repair from
        // a run when the repair in question was made waves ago.
        //
        // So the member now records what is actually known about its
        // shape — a run of it was accepted, or none has been — while
        // the variant names keep the member's own status unambiguous.
        // Every one of them begins `NotSubmitted`, and there is no
        // variant that says otherwise.
        verdict: member_verdict(fixture.pair, representation),
    })
}

/// Where §16.2's second condition stands for one pair.
///
/// Split out of the condition resolver because it is the one conjunct
/// whose subject is a TARGET verdict rather than a property of the built
/// members, and it is answered from the two lanes' runs of record.
fn acceptance_standing(pair: MinimalityPair) -> MinimalityConditionStanding {
    use MinimalityConditionStanding as Standing;

    let explicit_shape = recorded_acceptance(pair, LiveTransferRepresentationPlan::Explicit);
    let private_shape = recorded_acceptance(pair, LiveTransferRepresentationPlan::PrivateCommitted);
    match (explicit_shape, private_shape) {
        // BOTH MEMBERS accepted, which only the pairs arc's pair reaches.
        // It is matched FIRST because it is strictly stronger, and a
        // fall-through order that let it be read as a shape acceptance
        // would throw the distinction away at the one place it is worth
        // something.
        (
            PairShapeAcceptance::ObservedForThisMember {
                accepted_identity: explicit_identity,
                ..
            },
            PairShapeAcceptance::ObservedForThisMember {
                accepted_identity: private_identity,
                ..
            },
        ) => Standing::HoldsOnAcceptedMembers {
            explicit_identity,
            private_identity,
        },
        (
            PairShapeAcceptance::ObservedForThisShape {
                accepted_identity: explicit_identity,
                ..
            },
            PairShapeAcceptance::ObservedForThisShape {
                accepted_identity: private_identity,
                ..
            },
        ) => Standing::HoldsOnObservedShapeAcceptances {
            explicit_identity,
            private_identity,
        },
        (PairShapeAcceptance::NoRunOfThisShape { because }, _) => Standing::AwaitsARunOfThisShape {
            lane: "explicit",
            because,
        },
        (_, PairShapeAcceptance::NoRunOfThisShape { because }) => Standing::AwaitsARunOfThisShape {
            lane: "private-committed",
            because,
        },
        // One member accepted and the other only shape-observed. No pair
        // is in this state and the arm is not decoration: it is what
        // keeps the registry from silently promoting a HALF-submitted
        // pair to the stronger standing the moment a second arc is built
        // and stops half way.
        (
            PairShapeAcceptance::ObservedForThisMember { .. },
            PairShapeAcceptance::ObservedForThisShape { .. },
        ) => Standing::AwaitsARunOfThisShape {
            lane: "private-committed",
            because: "the explicit member was submitted and accepted and the private member was not",
        },
        (
            PairShapeAcceptance::ObservedForThisShape { .. },
            PairShapeAcceptance::ObservedForThisMember { .. },
        ) => Standing::AwaitsARunOfThisShape {
            lane: "explicit",
            because: "the private member was submitted and accepted and the explicit member was not",
        },
    }
}

/// One member's target verdict, from what the runs of record recorded.
///
/// Split out of the materializer because it is a different question from
/// building a member: what this answers is what a target said, and the
/// materializer answers what this workspace can construct.
fn member_verdict(
    pair: MinimalityPair,
    representation: LiveTransferRepresentationPlan,
) -> PairTargetVerdict {
    match recorded_acceptance(pair, representation) {
        PairShapeAcceptance::ObservedForThisShape {
            case,
            accepted_identity,
        } => PairTargetVerdict::NotSubmittedShapeAcceptedElsewhere {
            case,
            accepted_identity,
        },
        PairShapeAcceptance::NoRunOfThisShape { because } => {
            PairTargetVerdict::NotSubmittedNoRunOfThisShape { because }
        }
        PairShapeAcceptance::ObservedForThisMember {
            case,
            accepted_identity,
        } => PairTargetVerdict::Accepted {
            case,
            accepted_identity,
        },
    }
}

/// One member's complete transaction weight, where every position can be
/// filled.
///
/// §13.3's resource field wants a figure about a *transaction*, and a
/// finalized transfer is not one: its witnesses are not attached, so it
/// has no serialization and no weight. This fills every owner position
/// with bytes that authorize nothing and asks the sponsor for its own,
/// then weighs the result — the same route `crate::live_measurements`
/// takes, so the two studies weigh the same kind of thing.
///
/// `None` where completion refuses, which today is the sponsored pair and
/// for one reason: the modelled envelope declines to sign, because §1.9
/// puts that authorization outside protocol data and Wave 10 recorded
/// that no adapter produces it. Returning `None` rather than a weight of
/// zero is §18.4's rule applied to a first-party figure.
fn complete_weight_of(
    target: &ReviewedElementsTapscriptDefinition,
    finalized: &FinalizedLiveTransfer,
    report: &LiveConstructionReport,
    sponsor: Option<&dyn SponsorCapability>,
) -> Option<u64> {
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
    let authorized = authorize_live_transfer(finalized.clone(), responses).ok()?;
    let built = complete_live_transfer(target, authorized, report.clone(), sponsor).ok()?;
    Some(built.transaction().weight())
}

/// Where one pair's resource comparison stands (§13.3, §18.2).
///
/// Derived from the two members rather than declared: a pair whose halves
/// were both weighed carries both figures, and one whose half could not
/// be completed carries the component that stopped it.
const fn resolve_resources(
    explicit: &PairMaterialization,
    private: &PairMaterialization,
) -> ResourceComparisonStanding {
    match (explicit.complete_weight(), private.complete_weight()) {
        (Some(explicit_weight), Some(private_weight)) => {
            ResourceComparisonStanding::MeasuredWithNoConfidentialProof {
                explicit_weight,
                private_weight,
            }
        }
        _ => ResourceComparisonStanding::NoCompleteTransactionExists(
            LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent,
        ),
    }
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
/// Nine of them are properties of the two built members and are
/// recomputed here. The tenth is about accepted transactions, and it is
/// answered from the two lanes' runs of record — per pair, because the
/// answer differs per pair.
///
/// It read "seven checked, two blocked, one downstream" for as long as
/// the two blockers stood, and both had gone stale where they stood:
/// one named a residual the guide's own closeout lists as CLEARED, and
/// the other borrowed it to hold a second condition shut. Nothing about
/// the members changed to make the count nine; what changed is that the
/// standings were recomputed against what the campaign had observed
/// instead of being carried forward.
fn resolve_conditions(
    abi: &CandidateLiveTransferAbi,
    pair: MinimalityPair,
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

    // §16.2's second condition, recomputed from the two lanes' runs of
    // record rather than declared. Both members' shapes must have been
    // accepted; a pair with one side unrun names the side.
    let acceptance = acceptance_standing(pair);

    BTreeMap::from([
        // Both members finalize. That is not an assumption here: the
        // registry refuses to exist otherwise —
        // `MinimalityPairRefusal::MemberNotConstructible` is returned
        // the moment either half fails to build — so a row that can be
        // read at all is a row whose two materializations were both
        // constructed.
        //
        // What kept this blocked was never the construction. It was the
        // predecessor the private member rests on: a fixture view of a
        // confidential receipt, carried under a blocker whose own name
        // asserted that no such predecessor could be funded. That
        // assertion is now FALSE by observation, and observed is the
        // word that matters. A confidential predecessor has been
        // funded, mined, and read back raw; the private lane has spent
        // confidential predecessors in six accepted shapes; and the
        // guide's own closeout records
        // `NoConfidentialPredecessorCanBeFunded` in its cleared set
        // rather than its carried one.
        //
        // So the blocker cannot stand here. It was the last artifact in
        // this workspace still naming a residual the closeout had
        // cleared, which is a row moving in one artifact and not the
        // other — the exact condition this crate elsewhere calls a
        // defect.
        //
        // The residual that IS true is smaller and is not a blocker:
        // this pipeline does not call the private lane's own entry
        // point, so its private member is built through the declared
        // test construction model rather than through
        // `finalize_private_live_transfer`. That is an attempt not yet
        // made rather than a component that does not exist, it is what
        // §16.2's ninth condition is FOR, and that condition is
        // resolved on its own below rather than borrowed to hold this
        // one shut.
        (
            Condition::BothMaterializationsConstructible,
            Standing::HoldsFirstParty,
        ),
        // Recomputed from the two lanes' runs of record. This named
        // `OwnerSighashNotComputable` and then
        // `NoConfidentialPredecessorCanBeFunded`, and both were stale
        // by the time they were read: the digest is computed, the
        // profile is established, the predecessor is funded, and both
        // lanes carry observed acceptances.
        //
        // What replaces them is not a better blocker but an ANSWER,
        // and the answer differs per pair. Three pairs have a recorded
        // acceptance of each member's shape and stand at
        // `HoldsOnObservedShapeAcceptances` carrying both identities.
        // Two do not, and they stand at `AwaitsARunOfThisShape` naming
        // the lane and the difference. Neither standing claims a member
        // was submitted; `PairTargetVerdict` is where that is said, and
        // it says no for all ten members.
        (Condition::BothTargetTransactionsAccept, acceptance),
        // Recomputed, and it was always recomputable. Both members are
        // projected back to `ExpectedTransferSemantics` and compared
        // with the fixture's own expectation in
        // `derive_minimality_pairs`, and a pair whose halves moved
        // different value is REFUSED rather than returned — so every
        // row that exists has had this conjunct checked against both of
        // its members before anything else was asked.
        //
        // It stood at `AwaitsBothTargetVerdicts` on the reading that a
        // projection is a projection OF an accepted transaction. That
        // reading double-counts: §16.2 lists "both target transactions
        // accept" as a separate conjunct immediately above this one, so
        // holding this one shut on the same fact made one missing
        // acceptance close two conditions and made the scoreboard say
        // the deficit was wider than it was.
        (
            Condition::BothProjectionsEqualTheExpectedTransfer,
            Standing::HoldsFirstParty,
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
/// answers that, and it answers per row.
///
/// Today it answers `true` for the three pairs whose two shapes were
/// both run against a real target and accepted — one-to-one, merge, and
/// many-to-many — and `false` for split and sponsor, each of which is
/// missing a run of its PRIVATE member's shape and says so through the
/// standing on §16.2's second condition. It answered `false` for all
/// five until the runs existed.
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

        let conditions = resolve_conditions(&abi, fixture.pair, &lifecycle, &explicit, &private);
        let resources = resolve_resources(&explicit, &private);
        rows.push(MinimalityPairRow {
            pair: fixture.pair,
            claim: resolve_claim(&abi, &fixture),
            fixture,
            expected,
            explicit,
            private,
            conditions,
            lifecycle: lifecycle.clone(),
            resources,
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
        PairTargetVerdict, PredecessorAssumption, ResourceComparisonStanding, SponsorPresence,
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
            // §13.3's ninth field, now a measurement. The sponsorless
            // pairs carry both members' complete weights; the sponsored
            // pair carries the component that stops either half from
            // reaching a complete transaction at all.
            match row.fixture().sponsor() {
                SponsorPresence::Absent => {
                    let (explicit, private) = row
                        .resources()
                        .weights()
                        .expect("a sponsorless pair weighs both members");
                    assert_ne!(explicit, 0);
                    assert_ne!(private, 0);
                }
                SponsorPresence::PresentWithoutChange => {
                    assert_eq!(
                        row.resources(),
                        ResourceComparisonStanding::NoCompleteTransactionExists(
                            LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent,
                        ),
                    );
                }
            }
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
            // ONE pair's members were submitted, and the verdicts say
            // exactly which. The one-to-one pair's two members carry
            // `Accepted` on the arc's own identities; every other member
            // still begins `NotSubmitted`, and the split is asserted in
            // both directions so a verdict that spread would fail here.
            let submitted = row.pair() == MinimalityPair::OneToOne;
            for member in row.members() {
                if submitted {
                    assert!(matches!(
                        member.verdict(),
                        PairTargetVerdict::Accepted { .. }
                    ));
                    assert_eq!(member.verdict().name(), "accepted");
                } else {
                    assert!(matches!(
                        member.verdict(),
                        PairTargetVerdict::NotSubmittedShapeAcceptedElsewhere { .. }
                            | PairTargetVerdict::NotSubmittedNoRunOfThisShape { .. },
                    ));
                    assert!(member.verdict().name().starts_with("not-submitted"));
                }
            }
        }
    }

    #[test]
    fn exactly_the_pairs_whose_two_shapes_ran_support_minimality() {
        // The wave's central finding, held as a test rather than
        // written in a report. §16.2 is a conjunction of ten, and the
        // one conjunct that is not a property of the built members is
        // answered per pair from the two lanes' runs of record — so
        // "does this pair support minimality" now has five answers
        // rather than one.
        //
        // Three pairs have a recorded acceptance of BOTH members'
        // shapes and satisfy all ten. Two do not, and each is missing
        // the same side: a run of its PRIVATE member's shape. The names
        // are spelled here so that a pair moved by an edit rather than
        // by a run fails, which is the whole point of spelling them.
        let rows = build_minimality_pairs().expect("the pair registry builds");
        let supporting: BTreeSet<_> = rows
            .iter()
            .filter(|row| row.supports_minimality())
            .map(|row| row.pair().name())
            .collect();
        assert_eq!(
            supporting,
            BTreeSet::from(["one-to-one", "merge", "many-to-many", "split", "sponsor"]),
        );

        for row in &rows {
            let standing = row.conditions()[&PairAcceptanceCondition::BothTargetTransactionsAccept];
            if row.supports_minimality() {
                // A supporting pair rests on observation for exactly one
                // conjunct and on recomputation for the other nine, and
                // the two are never summed.
                assert!(standing.rests_on_observation(), "{}", row.pair().name());
                let observed = row
                    .conditions()
                    .values()
                    .filter(|standing| standing.rests_on_observation())
                    .count();
                assert_eq!(observed, 1, "{}", row.pair().name());
                continue;
            }
            // A pair that does not support minimality names its failing
            // conjunct, and the failure is the acceptance one for both
            // of them. Nine of ten is nothing: §16.2 says "only when".
            assert!(!standing.is_satisfied(), "{}", row.pair().name());
            let unsatisfied: Vec<_> = row
                .conditions()
                .iter()
                .filter(|(_, standing)| !standing.is_satisfied())
                .map(|(condition, _)| condition.name())
                .collect();
            assert_eq!(
                unsatisfied,
                vec!["both-target-transactions-accept"],
                "{} fails somewhere else too",
                row.pair().name(),
            );
            let MinimalityConditionStanding::AwaitsARunOfThisShape { lane, because } = standing
            else {
                panic!("{} fails without naming a lane", row.pair().name());
            };
            assert_eq!(lane, "private-committed", "{}", row.pair().name());
            assert_ne!(because.len(), 0);
        }

        // No pair names a blocker any more. Every blocker this registry
        // ever carried named a component that does not exist, and the
        // last of them — `NoConfidentialPredecessorCanBeFunded` — is in
        // the guide closeout's CLEARED set. What the two unsupported
        // pairs lack is a run, and a run not yet attempted is not a
        // missing component.
        let mut blockers = BTreeSet::new();
        for row in &rows {
            blockers.extend(row.blockers());
        }
        assert_eq!(blockers, BTreeSet::new());
    }

    #[test]
    fn the_scoreboard_names_the_deficit_condition_by_condition() {
        // §16.2's ten, with how many of the five pairs satisfy each.
        // Nine hold for every pair; the tenth holds for three and names
        // the missing lane for the other two.
        let rows = build_minimality_pairs().expect("the pair registry builds");
        let board = condition_scoreboard(&rows);
        assert_eq!(board.len(), PairAcceptanceCondition::ALL.len());

        let mut universal = 0_usize;
        for (condition, (satisfied, standings)) in &board {
            if *condition == PairAcceptanceCondition::BothTargetTransactionsAccept {
                assert_eq!(*satisfied, 5, "the acceptance conjunct");
                // NO pair is unsatisfied any more, so the deficit set is
                // empty. It is asserted empty rather than dropped: a
                // later shape whose run stopped reproducing would put a
                // standing back here, and a test that had stopped
                // looking would not notice.
                assert_eq!(standings.len(), 0);
                for standing in standings {
                    assert!(matches!(
                        standing,
                        MinimalityConditionStanding::AwaitsARunOfThisShape {
                            lane: "private-committed",
                            ..
                        },
                    ));
                }
                continue;
            }
            universal += 1;
            assert_eq!(
                *satisfied,
                rows.len(),
                "{} holds for some pairs only",
                condition.name()
            );
            assert_eq!(*standings, BTreeSet::new());
        }
        assert_eq!(universal, 9, "the recomputed half of §16.2");
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
