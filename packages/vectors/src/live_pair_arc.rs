//! The PAIRS ARC: one §16.1 semantic fixture materialized twice, both
//! materializations submitted to one node, and the relation over the two
//! accepted identities observed.
//!
//! # Why an arc of its own, and why it is not an amendment
//!
//! Wave seven's ledger records a run whose strict chain never reached a
//! pairs step, and that record stands unamended: it was the correct
//! record of its own run. What was unreachable was unreachable IN THAT
//! ORDER, so the pairs step is chartered here as a new arc with its own
//! entry condition and its own ledger, and nothing in
//! [`crate::live_restart`] or [`crate::live_closeout`] moves. Editing a
//! recorded ledger to make a later run fit it would be backdating, and it
//! is the one thing this module may not do.
//!
//! # The entry condition, cited rather than asserted
//!
//! The old order's blocker was the sponsor envelope's signer. That
//! blocker is discharged — a real node accepted a sponsor-signed control
//! — and the arc's entry condition CITES that acceptance by the identity
//! the target computed for it
//! ([`crate::live_sponsor_shapes::sponsored_run_of_record::SPONSORED_ACCEPTED_TXID`])
//! rather than restating the discharge in prose. A reader can check the
//! condition against a chain.
//!
//! # What makes the two members a PAIR
//!
//! §16.1's load-bearing sentence is that each pair begins from ONE
//! semantic fixture. Two ceremonies whose shapes merely match are not a
//! pair however alike they come out, which is the refutation the
//! minimality wave recorded and the reason
//! [`crate::live_pairs::PairTargetVerdict`] carries a member for a run of
//! a member's own SHAPE that denies the member was accepted.
//!
//! So this module states ONE [`SemanticTransferFixture`] and both
//! materializations read it: the explicit member's owners, cardinality
//! and funding AMOUNT are read from [`pair_arc_fixture`] inside
//! [`crate::live_explicit_shapes::ExplicitShape::PairedOneToOne`], and
//! the private member's destinations are read from the same function
//! inside [`crate::live_multi_shapes::PrivateShape::PairedOneToOne`].
//! Neither shape carries a literal of its own, so there is no second
//! author of the fixture to disagree with the first.
//!
//! # One run, one node, one issuance
//!
//! Both members are submitted by ONE ceremony against ONE issued asset,
//! and that is a requirement rather than a convenience: §6.6 asks a
//! pair's members to carry the same exact explicit `U`, and two runs
//! would issue two assets. The arc therefore drives the explicit lane's
//! ceremony to completion and then hands the private lane the asset the
//! first one issued
//! ([`crate::live_multi_shapes::MultiShapePlanner::for_shape_against_issued_asset`]).
//!
//! # The observation, and what it is careful not to claim
//!
//! The row this arc answers wants the PUBLIC PROTOCOL PROJECTIONS of the
//! two accepted transactions compared. They are computed here from the
//! node's own copy of each accepted transaction and never from what the
//! ceremony meant to build.
//!
//! The projections are not identical and must not be reported as if they
//! were: the explicit member publishes its exact amounts and the private
//! member publishes commitments. That difference is the whole subject of
//! the disclosure-minimality matrix, so every §6.6 term carries its own
//! [`ProjectionTermStanding`] saying HOW it was found equal — in the
//! node's copy, in public constructor metadata, or with the private side
//! withholding the exact value under §16.2's declared construction model.
//! A single boolean would let a reader take the third case for the first.
//!
//! # Every secret here is published
//!
//! The owners are [`crate::live_plan`]'s BIP-340 appendix scalars and
//! every opening is derived from values the confidential lane publishes,
//! admitted under ADR-015's test-material rule
//! `(´[ADR015-rule:security:test-material]´)`.

use std::collections::BTreeSet;

use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::protocol::{
    NativeOperationResponse, ObservedOutcomeLayer, OperationCaseId,
};
use transaction::bytes::{AssetField, TargetTransaction, ValueField};
use transaction::taproot::Digest32;

use compiler::live_transfer_plan::LiveTransferRepresentationPlan;

use crate::confidential_predecessor::PREDECESSOR_AMOUNTS;
use crate::error::VectorError;
use crate::live_explicit_shapes::{ExplicitShape, ExplicitShapePlanner};
use crate::live_multi_shapes::{MultiShapePlanner, PrivateShape};
use crate::live_owner_observation::asset_of;
use crate::live_pairs::{
    MinimalityPair, SemanticEndpoint, SemanticTransferFixture, SponsorPresence,
};
use crate::live_plan::{
    FEE_PROGRAM_DIGEST, FIRST_SCALAR, LiveShapeVocabulary, RESERVE_ASSET, SECOND_SCALAR,
    live_abi_for_vocabulary, published_owner,
};

/// The published owner index a fixture endpoint names, as a scalar.
///
/// The deployment publishes two owners and a fixture states an index, so
/// this is the whole of the map between the two vocabularies. It is here
/// rather than in either lane because BOTH lanes read it, and a map
/// spelled twice would be two maps.
#[must_use]
pub const fn published_scalar(owner: usize) -> [u8; 32] {
    if owner == 0 {
        FIRST_SCALAR
    } else {
        SECOND_SCALAR
    }
}

/// Which published owner holds the fixture's source.
pub const PAIR_ARC_SOURCE_OWNER: usize = 0;

/// Which published owner the fixture's destination is created for.
pub const PAIR_ARC_DESTINATION_OWNER: usize = 1;

/// The exact amount the arc's one fixture moves.
///
/// Taken from the confidential predecessor's PRIMARY coin rather than
/// chosen, and the direction is forced. The private member must consume a
/// coin that a registered predecessor already carries, and that
/// predecessor's amounts are recorded constants a run has already written
/// down; the explicit member's coin is created by a funding step, which
/// can be asked for any amount. So the side that cannot move sets the
/// figure and the side that can follows it — and no recorded digest moves
/// to buy the agreement.
pub const PAIR_ARC_AMOUNT: u64 = PREDECESSOR_AMOUNTS[0];

/// The arc's ONE semantic fixture.
///
/// A one-to-one transfer: the first published owner's receipt consumed,
/// the second published owner's receipt created, the whole amount
/// travelling and nothing held back. It carries no representation plan,
/// no position, no script and no blinding factor, which is §14.3's
/// requirement of a semantic fixture and the reason both members can be
/// built from it without either one being the other's model.
#[must_use]
pub fn pair_arc_fixture() -> SemanticTransferFixture {
    SemanticTransferFixture::stated(
        MinimalityPair::OneToOne,
        vec![SemanticEndpoint::new(
            PAIR_ARC_SOURCE_OWNER,
            PAIR_ARC_AMOUNT,
        )],
        vec![SemanticEndpoint::new(
            PAIR_ARC_DESTINATION_OWNER,
            PAIR_ARC_AMOUNT,
        )],
        SponsorPresence::Absent,
    )
}

/// One member of the arc's pair.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PairArcMember {
    /// The explicit materialization.
    Explicit,
    /// The private materialization.
    Private,
}

impl PairArcMember {
    /// Both members, explicit first — the order the arc submits them in.
    pub const ALL: &'static [Self] = &[Self::Explicit, Self::Private];

    /// The member's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Explicit => "explicit",
            Self::Private => "private",
        }
    }

    /// The representation plan this member materializes the fixture
    /// under.
    #[must_use]
    pub const fn plan(self) -> LiveTransferRepresentationPlan {
        match self {
            Self::Explicit => LiveTransferRepresentationPlan::Explicit,
            Self::Private => LiveTransferRepresentationPlan::PrivateCommitted,
        }
    }

    /// The step name this member's submission is asked for under.
    #[must_use]
    pub const fn case_name(self) -> &'static str {
        match self {
            Self::Explicit => "explicit-paired-one-to-one",
            Self::Private => "private-paired-one-to-one",
        }
    }
}

/// The arc's entry condition.
///
/// One condition and one citation. The old order's pairs step was blocked
/// on the sponsor envelope's signer; that blocker is discharged, and the
/// discharge is cited by the identity a target computed for the accepted
/// sponsor-signed control rather than restated. A condition carrying an
/// identity can be checked against a chain by somebody who does not trust
/// this crate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PairArcEntryCondition {
    cited_acceptance: &'static str,
}

impl PairArcEntryCondition {
    /// The condition, resolved against the sponsor lane's run of record.
    #[must_use]
    pub const fn stands() -> Self {
        Self {
            cited_acceptance:
                crate::live_sponsor_shapes::sponsored_run_of_record::SPONSORED_ACCEPTED_TXID,
        }
    }

    /// The acceptance the condition rests on.
    #[must_use]
    pub const fn cited_acceptance(self) -> &'static str {
        self.cited_acceptance
    }

    /// Whether the citation is a target identity at all.
    ///
    /// Checked rather than assumed: an entry condition citing something
    /// that is not an identity would be citing nothing.
    #[must_use]
    pub fn cites_a_target_identity(self) -> bool {
        self.cited_acceptance.len() == 64
            && self
                .cited_acceptance
                .chars()
                .all(|digit| digit.is_ascii_hexdigit())
    }
}

/// One member's PUBLIC PROTOCOL PROJECTION, recomputed from the node's
/// own copy of the accepted transaction.
///
/// Every field is read out of the accepted bytes. Nothing the ceremony
/// intended enters, which is what makes the comparison a comparison of
/// two chains' facts rather than of two builders' plans.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicProtocolProjection {
    member: PairArcMember,
    consumed_positions: usize,
    receipt_outputs: usize,
    fee_outputs: usize,
    unrecognized_outputs: usize,
    destination_owners: Vec<usize>,
    explicit_asset: Option<String>,
    published_destination_amounts: Vec<Option<u64>>,
}

impl PublicProtocolProjection {
    /// Which member this is the projection of.
    #[must_use]
    pub const fn member(&self) -> PairArcMember {
        self.member
    }

    /// How many input positions the accepted transaction carries.
    #[must_use]
    pub const fn consumed_positions(&self) -> usize {
        self.consumed_positions
    }

    /// How many outputs sit at a published owner's receipt destination
    /// program.
    #[must_use]
    pub const fn receipt_outputs(&self) -> usize {
        self.receipt_outputs
    }

    /// How many outputs meet the target's own fee-role test.
    #[must_use]
    pub const fn fee_outputs(&self) -> usize {
        self.fee_outputs
    }

    /// How many outputs are neither.
    ///
    /// The term that makes "absence of roots, issuance, destruction and
    /// certificates" an OBSERVATION: an accepted transaction whose every
    /// output is a receipt of a published owner or the fee role carries
    /// none of them, and one that carried any would be counted here.
    #[must_use]
    pub const fn unrecognized_outputs(&self) -> usize {
        self.unrecognized_outputs
    }

    /// Which published owner each receipt output was created for, in
    /// output order.
    #[must_use]
    pub fn destination_owners(&self) -> &[usize] {
        &self.destination_owners
    }

    /// The one explicit asset every output names, where they agree.
    #[must_use]
    pub fn explicit_asset(&self) -> Option<&str> {
        self.explicit_asset.as_deref()
    }

    /// The exact amount each receipt output PUBLISHES, in output order.
    ///
    /// `None` where the output carries a commitment. This is the field
    /// the whole minimality matrix is about, and it is a vector of
    /// options rather than a vector of amounts precisely so that a
    /// withheld value has no number in it to be read.
    #[must_use]
    pub fn published_destination_amounts(&self) -> &[Option<u64>] {
        &self.published_destination_amounts
    }

    /// Whether this member publishes every destination's exact amount.
    #[must_use]
    pub fn publishes_every_exact_amount(&self) -> bool {
        !self.published_destination_amounts.is_empty()
            && self
                .published_destination_amounts
                .iter()
                .all(Option::is_some)
    }
}

/// How one §6.6 term was found to agree between the two members.
///
/// There is deliberately no member meaning "assumed". A term whose
/// agreement nobody established is [`Self::Disagrees`], and a projection
/// carrying one of those is not an equality.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ProjectionTermStanding {
    /// Both members' accepted bytes were read and they agree.
    ///
    /// The strongest member: the fact is in two chains' copies of two
    /// transactions and in nothing this workspace says.
    EqualInTheNodesOwnCopy,
    /// The term is equal, and its equality is read off PUBLIC
    /// constructor metadata rather than off the accepted bytes.
    ///
    /// Kept apart from the member above because the evidence is
    /// different in kind. §16.3 classes owners as public constructor
    /// metadata, and an input's owner is public because the output it
    /// spends is on a chain — but this arc reads it from the deployment's
    /// own programs rather than fetching the spent outputs, and a reader
    /// deserves to know which.
    EqualByPublicConstructorMetadata,
    /// The explicit member publishes the exact term and the private
    /// member does not.
    ///
    /// THE MINIMALITY FINDING, and never a defect. The two members agree
    /// about the term at the fixture; the private member's accepted bytes
    /// carry a commitment where the explicit member's carry a number, so
    /// its side of the agreement rests on §16.2's ninth condition — the
    /// declared test construction model — rather than on a published
    /// value. Reporting this as [`Self::EqualInTheNodesOwnCopy`] would
    /// claim the chain publishes an amount it does not.
    EqualAndWithheldByThePrivateMember,
    /// Neither member carries the structure at all, and the absence was
    /// observed in both accepted copies.
    AbsentFromBothMembers,
    /// The two members do not agree.
    ///
    /// Present so that a disagreement has somewhere to go. A projection
    /// carrying one is not an equality and moves no row.
    Disagrees,
}

impl ProjectionTermStanding {
    /// Whether this standing is an agreement.
    #[must_use]
    pub const fn agrees(self) -> bool {
        !matches!(self, Self::Disagrees)
    }

    /// The standing's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::EqualInTheNodesOwnCopy => "equal-in-the-nodes-own-copy",
            Self::EqualByPublicConstructorMetadata => "equal-by-public-constructor-metadata",
            Self::EqualAndWithheldByThePrivateMember => "equal-and-withheld-by-the-private-member",
            Self::AbsentFromBothMembers => "absent-from-both-members",
            Self::Disagrees => "disagrees",
        }
    }
}

/// The eleven terms §6.6 names, in the guide's own order.
///
/// Transcribed rather than summarized, so a reader can check the list
/// against §6.6 line by line. A term dropped from here would be a term
/// the comparison stopped making.
pub const REPRESENTATION_EQUIVALENCE_TERMS: [&str; 11] = [
    "input-semantic-amount-multiset",
    "destination-owner-value-multiset",
    "distinct-input-owner-set",
    "live-class",
    "exact-explicit-u",
    "authorization-result",
    "absence-of-roots",
    "absence-of-issuance-and-destruction",
    "lateral-flow",
    "sponsor-relation-and-shape",
    "transition-certificate-projection",
];

/// The relation the arc observed over its two accepted identities.
///
/// One term per §6.6 line, each with the standing that says how it was
/// found to agree. The observation is an equality only when every term
/// agrees, which is [`Self::projections_are_equal`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionEqualityObservation {
    terms: Vec<(&'static str, ProjectionTermStanding)>,
    explicit: PublicProtocolProjection,
    private: PublicProtocolProjection,
}

impl ProjectionEqualityObservation {
    /// Every term and how it stands.
    #[must_use]
    pub fn terms(&self) -> &[(&'static str, ProjectionTermStanding)] {
        &self.terms
    }

    /// The explicit member's projection.
    #[must_use]
    pub const fn explicit(&self) -> &PublicProtocolProjection {
        &self.explicit
    }

    /// The private member's projection.
    #[must_use]
    pub const fn private(&self) -> &PublicProtocolProjection {
        &self.private
    }

    /// Whether every §6.6 term agrees.
    #[must_use]
    pub fn projections_are_equal(&self) -> bool {
        self.terms.len() == REPRESENTATION_EQUIVALENCE_TERMS.len()
            && self.terms.iter().all(|(_, standing)| standing.agrees())
    }

    /// How many terms the private member withholds the exact value of.
    ///
    /// The disclosure delta, as a number a report can carry. Zero would
    /// mean the private representation published everything the explicit
    /// one did, which would make the pair evidence of nothing.
    #[must_use]
    pub fn terms_withheld_by_the_private_member(&self) -> usize {
        self.terms
            .iter()
            .filter(|(_, standing)| {
                *standing == ProjectionTermStanding::EqualAndWithheldByThePrivateMember
            })
            .count()
    }
}

/// One member's acceptance, as the arc recorded it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedMember {
    member: PairArcMember,
    accepted_txid: String,
    block_height: Option<u32>,
    submitted_bytes: usize,
    target_weight: Option<u64>,
    readback_matches_submission: bool,
    every_input_verified: bool,
    projection: PublicProtocolProjection,
}

impl AcceptedMember {
    /// Which member.
    #[must_use]
    pub const fn member(&self) -> PairArcMember {
        self.member
    }

    /// The identity the target computed.
    #[must_use]
    pub fn accepted_txid(&self) -> &str {
        &self.accepted_txid
    }

    /// The block the acceptance was mined into.
    #[must_use]
    pub const fn block_height(&self) -> Option<u32> {
        self.block_height
    }

    /// How many bytes were handed to the node.
    #[must_use]
    pub const fn submitted_bytes(&self) -> usize {
        self.submitted_bytes
    }

    /// The weight the TARGET computed.
    #[must_use]
    pub const fn target_weight(&self) -> Option<u64> {
        self.target_weight
    }

    /// Whether the node's copy is byte-equal to the submission.
    #[must_use]
    pub const fn readback_matches_submission(&self) -> bool {
        self.readback_matches_submission
    }

    /// Whether every input's accepted signature verified against an
    /// independently recomputed message.
    #[must_use]
    pub const fn every_input_verified(&self) -> bool {
        self.every_input_verified
    }

    /// The projection recomputed from the node's own copy.
    #[must_use]
    pub const fn projection(&self) -> &PublicProtocolProjection {
        &self.projection
    }

    /// Whether this member meets the acceptance bar every recorded run
    /// of this campaign meets.
    #[must_use]
    pub fn meets_the_acceptance_bar(&self) -> bool {
        self.accepted_txid.len() == 64
            && self.readback_matches_submission
            && self.every_input_verified
            && self.block_height.is_some()
    }
}

/// Why the arc stopped before it had a ledger.
///
/// A construction or observation refusal, and never a target verdict.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PairArcRefusal {
    /// The reviewed substrate or the deployment did not build.
    SubstrateUnavailable,
    /// The fixture's consumed and created totals do not agree.
    FixtureDoesNotConserve,
    /// The explicit half completed without naming the issued asset.
    IssuanceNamedNoAsset,
    /// A member reached the node and was not accepted.
    MemberNotAccepted(PairArcMember),
    /// An acceptance carried no readback to recompute a projection from.
    AcceptanceCarriedNoReadback(PairArcMember),
    /// The node's own copy did not decode.
    ReadbackDidNotDecode(PairArcMember),
    /// One half's own ceremony refused before the node.
    MemberCeremonyRefused(PairArcMember),
}

/// The arc's LEDGER: its entry condition, both accepted members, and the
/// relation observed over them.
///
/// Its own record, and not an amendment of any other. Nothing here is
/// written unless both members were accepted, which is why the two member
/// fields are not options: a ledger with a member missing would be a
/// ledger of something that is not a pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairArcLedger {
    entry: PairArcEntryCondition,
    issued_asset: String,
    explicit: AcceptedMember,
    private: AcceptedMember,
    observation: ProjectionEqualityObservation,
}

impl PairArcLedger {
    /// The entry condition the arc opened on.
    #[must_use]
    pub const fn entry(&self) -> PairArcEntryCondition {
        self.entry
    }

    /// The one asset both members carry.
    #[must_use]
    pub fn issued_asset(&self) -> &str {
        &self.issued_asset
    }

    /// The explicit member's acceptance.
    #[must_use]
    pub const fn explicit(&self) -> &AcceptedMember {
        &self.explicit
    }

    /// The private member's acceptance.
    #[must_use]
    pub const fn private(&self) -> &AcceptedMember {
        &self.private
    }

    /// The relation over the two accepted identities.
    #[must_use]
    pub const fn observation(&self) -> &ProjectionEqualityObservation {
        &self.observation
    }

    /// Whether this ledger carries everything the row's rule asks for.
    ///
    /// Recomputed rather than declared: both members accepted at the bar
    /// every recorded run meets, both carrying the same asset, and every
    /// §6.6 term agreeing.
    #[must_use]
    pub fn supports_the_projection_equality_row(&self) -> bool {
        self.entry.cites_a_target_identity()
            && self.explicit.meets_the_acceptance_bar()
            && self.private.meets_the_acceptance_bar()
            && self.explicit.accepted_txid != self.private.accepted_txid
            && self.observation.projections_are_equal()
            && self.observation.terms_withheld_by_the_private_member() > 0
    }
}

/// What the arc is doing next.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Half {
    /// The explicit member's ceremony.
    Explicit,
    /// The private member's ceremony.
    Private,
    /// Nothing further.
    Done,
}

/// The transcript one arc run produces.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PairArcRecord {
    fixture_conserves: bool,
    issued_asset: Option<String>,
    explicit: Option<AcceptedMember>,
    private: Option<AcceptedMember>,
    ledger: Option<PairArcLedger>,
    refusal: Option<PairArcRefusal>,
}

impl PairArcRecord {
    /// Whether the arc's one fixture conserves.
    #[must_use]
    pub const fn fixture_conserves(&self) -> bool {
        self.fixture_conserves
    }

    /// The disposable asset both members carry.
    #[must_use]
    pub fn issued_asset(&self) -> Option<&str> {
        self.issued_asset.as_deref()
    }

    /// The explicit member's acceptance, where there was one.
    #[must_use]
    pub const fn explicit(&self) -> Option<&AcceptedMember> {
        self.explicit.as_ref()
    }

    /// The private member's acceptance, where there was one.
    #[must_use]
    pub const fn private(&self) -> Option<&AcceptedMember> {
        self.private.as_ref()
    }

    /// The arc's ledger, where the run produced one.
    #[must_use]
    pub const fn ledger(&self) -> Option<&PairArcLedger> {
        self.ledger.as_ref()
    }

    /// The refusal, where the arc stopped.
    #[must_use]
    pub const fn refusal(&self) -> Option<PairArcRefusal> {
        self.refusal
    }
}

/// The pairs arc's ceremony: one fixture, two members, one node.
pub struct PairArcPlanner {
    half: Half,
    genesis_block_hash: Digest32,
    explicit: ExplicitShapePlanner,
    private: Option<MultiShapePlanner>,
    record: PairArcRecord,
}

impl PairArcPlanner {
    /// The arc, bound to a deployment's printed genesis identity.
    ///
    /// # Errors
    ///
    /// [`VectorError::LiveSubstrateUnavailable`] where the reviewed
    /// target does not build, and where the arc's own fixture does not
    /// conserve — a fixture that did not would materialize into two
    /// transactions that both fail conservation, and the pair would then
    /// report a fixture defect as a representation finding.
    pub fn new(printed_genesis_identity: Digest32) -> Result<Self, VectorError> {
        let fixture = pair_arc_fixture();
        if !fixture.expected().conserves() {
            return Err(VectorError::LiveSubstrateUnavailable);
        }
        let explicit = ExplicitShapePlanner::for_shape(
            ExplicitShape::PairedOneToOne,
            printed_genesis_identity,
        )?;
        Ok(Self {
            half: Half::Explicit,
            genesis_block_hash: printed_genesis_identity,
            explicit,
            private: None,
            record: PairArcRecord {
                fixture_conserves: true,
                ..PairArcRecord::default()
            },
        })
    }

    /// The transcript.
    #[must_use]
    pub const fn record(&self) -> &PairArcRecord {
        &self.record
    }

    /// Record a refusal and stop.
    fn refuse(&mut self, refusal: PairArcRefusal) -> PlanRefused {
        if self.record.refusal.is_none() {
            self.record.refusal = Some(refusal);
        }
        self.half = Half::Done;
        PlanRefused
    }

    /// Take one member's acceptance out of the response it arrived in.
    ///
    /// Everything here is the NODE's: the identity, the height, the
    /// weight, and the bytes the projection is recomputed from.
    fn observe_member(
        &mut self,
        member: PairArcMember,
        response: &NativeOperationResponse,
        submitted_bytes: usize,
        every_input_verified: bool,
        readback_matches_submission: bool,
    ) -> Result<(), PairArcRefusal> {
        if !matches!(response.observed_layer, ObservedOutcomeLayer::Accepted) {
            return Err(PairArcRefusal::MemberNotAccepted(member));
        }
        let readback = response
            .mined_readback
            .as_ref()
            .ok_or(PairArcRefusal::AcceptanceCarriedNoReadback(member))?;
        let decoded = TargetTransaction::decode(&readback.raw_transaction)
            .map_err(|_| PairArcRefusal::ReadbackDidNotDecode(member))?;
        let printed = self
            .record
            .issued_asset
            .clone()
            .ok_or(PairArcRefusal::IssuanceNamedNoAsset)?;
        let projection = project_public_protocol(member, &decoded, &printed)
            .map_err(|()| PairArcRefusal::SubstrateUnavailable)?;
        let accepted = AcceptedMember {
            member,
            accepted_txid: readback.transaction_id.clone(),
            block_height: Some(readback.block_height),
            submitted_bytes,
            target_weight: response.resources.transaction_weight,
            readback_matches_submission,
            every_input_verified,
            projection,
        };
        match member {
            PairArcMember::Explicit => self.record.explicit = Some(accepted),
            PairArcMember::Private => self.record.private = Some(accepted),
        }
        Ok(())
    }

    /// Settle the explicit half and open the private one.
    fn open_the_private_half(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), PairArcRefusal> {
        if let Some(refusal) = self.explicit.record().refusal() {
            let _ = refusal;
            return Err(PairArcRefusal::MemberCeremonyRefused(
                PairArcMember::Explicit,
            ));
        }
        self.record.issued_asset = self
            .explicit
            .record()
            .issued_asset()
            .map(std::borrow::ToOwned::to_owned);
        let check = self
            .explicit
            .record()
            .reverification()
            .ok_or(PairArcRefusal::MemberNotAccepted(PairArcMember::Explicit))?;
        let every_input_verified = check.every_input_verified();
        let readback_matches = check.readback_matches_submission();
        let submitted_bytes = self.explicit.record().submitted_bytes();
        self.observe_member(
            PairArcMember::Explicit,
            response,
            submitted_bytes,
            every_input_verified,
            readback_matches,
        )?;

        let printed = self
            .record
            .issued_asset
            .clone()
            .ok_or(PairArcRefusal::IssuanceNamedNoAsset)?;
        let private = MultiShapePlanner::for_shape_against_issued_asset(
            PrivateShape::PairedOneToOne,
            self.genesis_block_hash,
            &printed,
        )
        .map_err(|_| PairArcRefusal::SubstrateUnavailable)?;
        self.private = Some(private);
        self.half = Half::Private;
        Ok(())
    }

    /// Settle the private half and close the ledger.
    fn close_the_ledger(
        &mut self,
        response: &NativeOperationResponse,
    ) -> Result<(), PairArcRefusal> {
        let private = self
            .private
            .as_ref()
            .ok_or(PairArcRefusal::SubstrateUnavailable)?;
        if private.record().refusal().is_some() {
            return Err(PairArcRefusal::MemberCeremonyRefused(
                PairArcMember::Private,
            ));
        }
        let check = private
            .record()
            .reverification()
            .ok_or(PairArcRefusal::MemberNotAccepted(PairArcMember::Private))?;
        let verified = check.verified();
        let readback_matches = check.readback_matches_submission();
        let submitted_bytes = private.record().submitted_bytes();
        self.observe_member(
            PairArcMember::Private,
            response,
            submitted_bytes,
            verified,
            readback_matches,
        )?;

        let (Some(explicit), Some(private)) =
            (self.record.explicit.clone(), self.record.private.clone())
        else {
            return Err(PairArcRefusal::SubstrateUnavailable);
        };
        let issued_asset = self
            .record
            .issued_asset
            .clone()
            .ok_or(PairArcRefusal::IssuanceNamedNoAsset)?;
        let observation =
            compare_public_protocol_projections(&explicit.projection, &private.projection);
        self.record.ledger = Some(PairArcLedger {
            entry: PairArcEntryCondition::stands(),
            issued_asset,
            explicit,
            private,
            observation,
        });
        Ok(())
    }
}

impl TargetOperationPlanner for PairArcPlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        match self.half {
            Half::Explicit => {
                let step = self.explicit.next_step(previous)?;
                if let Some(step) = step {
                    return Ok(Some(step));
                }
                let Some((_case, response)) = previous else {
                    return Err(self.refuse(PairArcRefusal::SubstrateUnavailable));
                };
                // The explicit ceremony is complete and the response in
                // hand is its submission's. Cloning it out of the borrow
                // keeps the arc free to write its own record.
                let response = response.clone();
                if let Err(refusal) = self.open_the_private_half(&response) {
                    return Err(self.refuse(refusal));
                }
                if self.private.is_none() {
                    return Err(self.refuse(PairArcRefusal::SubstrateUnavailable));
                }
                let private = self
                    .private
                    .as_mut()
                    .expect("the private half was just opened");
                private.next_step(None)
            }
            Half::Private => {
                let private = match self.private.as_mut() {
                    Some(private) => private,
                    None => return Err(self.refuse(PairArcRefusal::SubstrateUnavailable)),
                };
                let step = private.next_step(previous)?;
                if let Some(step) = step {
                    return Ok(Some(step));
                }
                let Some((_case, response)) = previous else {
                    return Err(self.refuse(PairArcRefusal::SubstrateUnavailable));
                };
                let response = response.clone();
                if let Err(refusal) = self.close_the_ledger(&response) {
                    return Err(self.refuse(refusal));
                }
                self.half = Half::Done;
                Ok(None)
            }
            Half::Done => Ok(None),
        }
    }
}

/// Recompute one member's public protocol projection from an accepted
/// transaction's own bytes.
///
/// # Errors
///
/// `()` where the deployment does not link against the named asset, which
/// is the only way the owner programs this reads against can be
/// unavailable.
fn project_public_protocol(
    member: PairArcMember,
    decoded: &TargetTransaction,
    printed_asset: &str,
) -> Result<PublicProtocolProjection, ()> {
    let programs = destination_programs(printed_asset, member.plan())?;

    let mut receipt_outputs = 0_usize;
    let mut fee_outputs = 0_usize;
    let mut unrecognized_outputs = 0_usize;
    let mut destination_owners = Vec::new();
    let mut published_destination_amounts = Vec::new();
    let mut assets: BTreeSet<String> = BTreeSet::new();

    for output in decoded.outputs() {
        if let AssetField::Explicit(asset) = output.asset() {
            assets.insert(hex(asset.internal()));
        }
        if output.is_fee() {
            fee_outputs += 1;
            continue;
        }
        let Some(owner) = programs
            .iter()
            .position(|program| program.as_slice() == output.program())
        else {
            unrecognized_outputs += 1;
            continue;
        };
        receipt_outputs += 1;
        destination_owners.push(owner);
        published_destination_amounts.push(match output.value() {
            ValueField::Explicit(amount) => Some(amount),
            // A commitment publishes no exact amount, and neither would
            // any value form this crate has yet to learn: `None` says
            // "this projection read no exact value here", which is the
            // honest answer in both cases.
            _ => None,
        });
    }

    // ONE asset or none. Two would mean the transaction moves more than
    // one family, and a projection stating one of them would be stating
    // half a fact.
    let explicit_asset = if assets.len() == 1 {
        assets.into_iter().next()
    } else {
        None
    };

    Ok(PublicProtocolProjection {
        member,
        consumed_positions: decoded.inputs().len(),
        receipt_outputs,
        fee_outputs,
        unrecognized_outputs,
        destination_owners,
        explicit_asset,
        published_destination_amounts,
    })
}

/// The two published owners' destination programs under one plan, in
/// owner-index order.
fn destination_programs(
    printed_asset: &str,
    plan: LiveTransferRepresentationPlan,
) -> Result<Vec<Vec<u8>>, ()> {
    let asset = asset_of(printed_asset).ok_or(())?;
    let abi = live_abi_for_vocabulary(
        LiveShapeVocabulary::Demonstration,
        *asset.internal(),
        RESERVE_ASSET,
        FEE_PROGRAM_DIGEST,
    )
    .map_err(|_| ())?;
    let mut programs = Vec::new();
    for index in 0..2_usize {
        let scalar = published_scalar(index);
        let owner = published_owner(&scalar).map_err(|_| ())?;
        let program = abi
            .destinations()
            .get(&linker::OwnerParameter::new(owner), plan)
            .ok_or(())?
            .instance()
            .program()
            .to_vec();
        programs.push(program);
    }
    Ok(programs)
}

/// Compare two accepted members' public protocol projections, term by
/// §6.6 term.
///
/// The comparison is the arc's whole observation, and it is written to be
/// PRECISE rather than generous: a term the private member withholds is
/// recorded as withheld, not as equal, and a term that disagrees is
/// recorded as a disagreement rather than dropped.
#[must_use]
pub fn compare_public_protocol_projections(
    explicit: &PublicProtocolProjection,
    private: &PublicProtocolProjection,
) -> ProjectionEqualityObservation {
    use ProjectionTermStanding as S;

    let fixture = pair_arc_fixture();
    let expected = fixture.expected();

    // The two sides' shapes, as the node's copies report them.
    let same_width = explicit.consumed_positions == private.consumed_positions
        && explicit.receipt_outputs == private.receipt_outputs;
    let same_owners = explicit.destination_owners == private.destination_owners;
    let same_asset =
        explicit.explicit_asset.is_some() && explicit.explicit_asset == private.explicit_asset;
    let both_clean = explicit.unrecognized_outputs == 0
        && private.unrecognized_outputs == 0
        && explicit.fee_outputs == 0
        && private.fee_outputs == 0;

    // The explicit member's published amounts are the fixture's; the
    // private member's are absent, which is the point of the pair.
    let explicit_amounts_are_the_fixtures = explicit.published_destination_amounts
        == fixture
            .destinations()
            .iter()
            .map(|endpoint| Some(endpoint.amount()))
            .collect::<Vec<_>>();
    let private_withholds = private
        .published_destination_amounts
        .iter()
        .all(Option::is_none)
        && !private.published_destination_amounts.is_empty();
    let amounts_agree_with_disclosure_asymmetry =
        explicit_amounts_are_the_fixtures && private_withholds;

    let consumed_agrees = same_width && explicit.consumed_positions == expected.consumed_receipts();
    let owners_agree = same_owners
        && explicit.destination_owners
            == fixture
                .destinations()
                .iter()
                .map(|endpoint| endpoint.owner())
                .collect::<Vec<_>>();

    let terms = vec![
        // §6.6 term 1. The explicit member publishes the consumed
        // amounts through the coins it spends; the private member spends
        // commitments. The counts agree in both copies and the exact
        // values are published on one side only.
        (
            REPRESENTATION_EQUIVALENCE_TERMS[0],
            if consumed_agrees && amounts_agree_with_disclosure_asymmetry {
                S::EqualAndWithheldByThePrivateMember
            } else {
                S::Disagrees
            },
        ),
        // §6.6 term 2. The OWNERS agree in both copies — the destination
        // programs are the deployment's own and were matched in each
        // accepted transaction — and the VALUES are published by the
        // explicit member alone.
        (
            REPRESENTATION_EQUIVALENCE_TERMS[1],
            if owners_agree && amounts_agree_with_disclosure_asymmetry {
                S::EqualAndWithheldByThePrivateMember
            } else {
                S::Disagrees
            },
        ),
        // §6.6 term 3. Public constructor metadata: the fixture states
        // one source owner and both ceremonies fund that owner's
        // constructor, which is public data about a program rather than
        // a field of either accepted transaction.
        (
            REPRESENTATION_EQUIVALENCE_TERMS[2],
            if expected.distinct_input_owners().len() == 1 && consumed_agrees {
                S::EqualByPublicConstructorMetadata
            } else {
                S::Disagrees
            },
        ),
        // §6.6 term 4. Every receipt output of both members sits at a
        // published owner's LIVE receipt destination program under that
        // member's own plan, matched in the node's copy.
        (
            REPRESENTATION_EQUIVALENCE_TERMS[3],
            if same_owners && both_clean {
                S::EqualInTheNodesOwnCopy
            } else {
                S::Disagrees
            },
        ),
        // §6.6 term 5. One explicit asset, and the SAME one, read out of
        // both accepted copies. This is the term one issuance buys: two
        // runs would carry two assets and this term would disagree.
        (
            REPRESENTATION_EQUIVALENCE_TERMS[4],
            if same_asset {
                S::EqualInTheNodesOwnCopy
            } else {
                S::Disagrees
            },
        ),
        // §6.6 term 6. Both accepted, which is a target verdict about
        // the authorization each carried; the arc's ledger additionally
        // holds each member to a witness verified against a recomputed
        // message.
        (
            REPRESENTATION_EQUIVALENCE_TERMS[5],
            S::EqualInTheNodesOwnCopy,
        ),
        // §6.6 terms 7, 8 and 11. Nothing but receipt outputs of
        // published owners appears in either copy, and the decoder
        // refuses an issuance or peg-in input outright — so a copy that
        // decoded carries none.
        (
            REPRESENTATION_EQUIVALENCE_TERMS[6],
            if both_clean {
                S::AbsentFromBothMembers
            } else {
                S::Disagrees
            },
        ),
        (
            REPRESENTATION_EQUIVALENCE_TERMS[7],
            if both_clean {
                S::AbsentFromBothMembers
            } else {
                S::Disagrees
            },
        ),
        // §6.6 term 9. One asset in both copies, so nothing flowed
        // laterally into another family.
        (
            REPRESENTATION_EQUIVALENCE_TERMS[8],
            if same_asset && both_clean {
                S::EqualInTheNodesOwnCopy
            } else {
                S::Disagrees
            },
        ),
        // §6.6 term 10. The fixture declares no sponsor region and
        // neither copy carries a fee or a sponsor output.
        (
            REPRESENTATION_EQUIVALENCE_TERMS[9],
            if expected.sponsor() == SponsorPresence::Absent && both_clean {
                S::AbsentFromBothMembers
            } else {
                S::Disagrees
            },
        ),
        (
            REPRESENTATION_EQUIVALENCE_TERMS[10],
            if both_clean {
                S::AbsentFromBothMembers
            } else {
                S::Disagrees
            },
        ),
    ];

    ProjectionEqualityObservation {
        terms,
        explicit: explicit.clone(),
        private: private.clone(),
    }
}

/// One byte string, lower-case.
fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    bytes.iter().fold(String::new(), |mut text, byte| {
        let _ = write!(text, "{byte:02x}");
        text
    })
}

/// The transcript, one fact per line.
#[must_use]
pub fn render_pair_arc(record: &PairArcRecord) -> String {
    use std::fmt::Write as _;

    let mut out = String::new();
    let _ = writeln!(out, "run pairs-arc");
    let _ = writeln!(out, "fixture_conserves {}", record.fixture_conserves());
    let _ = writeln!(out, "fixture_amount {PAIR_ARC_AMOUNT}");
    let _ = writeln!(
        out,
        "entry_condition_cites {}",
        PairArcEntryCondition::stands().cited_acceptance(),
    );
    let _ = writeln!(
        out,
        "issued_asset {}",
        record.issued_asset().unwrap_or("absent"),
    );
    for member in PairArcMember::ALL {
        let observed = match member {
            PairArcMember::Explicit => record.explicit(),
            PairArcMember::Private => record.private(),
        };
        let Some(observed) = observed else {
            let _ = writeln!(out, "member {} absent", member.name());
            continue;
        };
        let _ = writeln!(
            out,
            "member {} accepted_txid {} height {} submitted_bytes {} weight {} readback_equal {} inputs_verified {}",
            member.name(),
            observed.accepted_txid(),
            observed
                .block_height()
                .map_or_else(|| "absent".to_owned(), |height| height.to_string()),
            observed.submitted_bytes(),
            observed
                .target_weight()
                .map_or_else(|| "absent".to_owned(), |weight| weight.to_string()),
            observed.readback_matches_submission(),
            observed.every_input_verified(),
        );
        let projection = observed.projection();
        let _ = writeln!(
            out,
            "projection {} consumed {} receipts {} fee {} unrecognized {} owners {:?} publishes_amounts {}",
            member.name(),
            projection.consumed_positions(),
            projection.receipt_outputs(),
            projection.fee_outputs(),
            projection.unrecognized_outputs(),
            projection.destination_owners(),
            projection.publishes_every_exact_amount(),
        );
    }
    if let Some(ledger) = record.ledger() {
        for (term, standing) in ledger.observation().terms() {
            let _ = writeln!(out, "term {term} {}", standing.name());
        }
        let _ = writeln!(
            out,
            "projections_are_equal {}",
            ledger.observation().projections_are_equal(),
        );
        let _ = writeln!(
            out,
            "terms_withheld_by_the_private_member {}",
            ledger.observation().terms_withheld_by_the_private_member(),
        );
        let _ = writeln!(
            out,
            "supports_the_projection_equality_row {}",
            ledger.supports_the_projection_equality_row(),
        );
    } else {
        let _ = writeln!(out, "ledger absent");
    }
    let _ = writeln!(
        out,
        "refusal {}",
        record
            .refusal()
            .map_or_else(|| "none".to_owned(), |refusal| format!("{refusal:?}")),
    );
    // What this run did NOT establish, in its own bytes.
    let _ = writeln!(out, "evidences_no_negative_case true");
    let _ = writeln!(out, "builds_no_sponsor_region true");
    out
}

/// What the arc's own run against a real node observed.
///
/// Empty until the arc has run. Both identities are options for the
/// reason the self-paying explicit shape's is: a constant carrying a
/// placeholder would be a claim, and `None` is the absence of one.
pub mod run_of_record {
    /// The identity the target computed for the accepted EXPLICIT member.
    pub const EXPLICIT_MEMBER_ACCEPTED_IDENTITY: Option<&str> = None;

    /// The identity the target computed for the accepted PRIVATE member.
    pub const PRIVATE_MEMBER_ACCEPTED_IDENTITY: Option<&str> = None;

    /// Whether the arc has produced a ledger at all.
    ///
    /// The one flag the evidence matrix reads. It is FALSE until a run
    /// writes the two identities above, and the classifier's paired
    /// branch declines the row while it is.
    pub const A_PAIR_ARC_LEDGER_EXISTS: bool = false;
}

#[cfg(test)]
mod tests {
    use super::{
        PAIR_ARC_AMOUNT, PAIR_ARC_DESTINATION_OWNER, PAIR_ARC_SOURCE_OWNER, PairArcEntryCondition,
        PairArcMember, REPRESENTATION_EQUIVALENCE_TERMS, pair_arc_fixture, published_scalar,
        run_of_record,
    };
    use crate::confidential_predecessor::PREDECESSOR_AMOUNTS;
    use crate::live_explicit_shapes::ExplicitShape;
    use crate::live_multi_shapes::PrivateShape;
    use crate::live_pairs::MinimalityPair;
    use crate::live_plan::{FIRST_SCALAR, SECOND_SCALAR};

    #[test]
    fn the_arc_states_one_fixture_and_it_conserves() {
        let fixture = pair_arc_fixture();
        assert_eq!(fixture.pair(), MinimalityPair::OneToOne);
        assert_eq!(fixture.sources().len(), 1);
        assert_eq!(fixture.destinations().len(), 1);
        assert!(fixture.expected().conserves());
        assert_eq!(fixture.sources()[0].owner(), PAIR_ARC_SOURCE_OWNER);
        assert_eq!(
            fixture.destinations()[0].owner(),
            PAIR_ARC_DESTINATION_OWNER
        );
        assert_eq!(fixture.expected().aggregate(), PAIR_ARC_AMOUNT);
    }

    #[test]
    fn both_members_are_materializations_of_that_one_fixture() {
        // §16.1's load-bearing requirement, checked rather than
        // described: each side's shape is asked what it will build and
        // the answer is held against the fixture. A shape that grew a
        // literal of its own would fail here.
        let fixture = pair_arc_fixture();

        let explicit = ExplicitShape::PairedOneToOne;
        assert_eq!(explicit.input_count(), fixture.sources().len());
        assert_eq!(explicit.output_count(), fixture.destinations().len());
        assert_eq!(explicit.funded_amount(), fixture.sources()[0].amount());
        assert_eq!(explicit.row_name(), None);

        let private = PrivateShape::PairedOneToOne;
        assert_eq!(private.output_count(), fixture.destinations().len());
        assert_eq!(private.input_count(), fixture.sources().len());
        assert_eq!(private.row_name(), None);
    }

    #[test]
    fn the_fixture_amount_is_the_coin_the_private_member_must_consume() {
        // The direction the amount travels in, made falsifiable. The
        // private member spends the dual-parity predecessor's PRIMARY
        // coin, whose amount a recorded run already wrote down; the
        // fixture takes that figure and the explicit funding step
        // follows it.
        assert_eq!(PAIR_ARC_AMOUNT, PREDECESSOR_AMOUNTS[0]);
    }

    #[test]
    fn the_two_members_take_the_two_representation_plans() {
        let plans: Vec<_> = PairArcMember::ALL
            .iter()
            .map(|member| member.plan())
            .collect();
        assert_eq!(plans.len(), 2);
        assert_ne!(plans[0], plans[1]);
    }

    #[test]
    fn the_entry_condition_cites_a_target_identity() {
        let entry = PairArcEntryCondition::stands();
        assert!(entry.cites_a_target_identity());
        assert_eq!(
            entry.cited_acceptance(),
            crate::live_sponsor_shapes::sponsored_run_of_record::SPONSORED_ACCEPTED_TXID,
        );
    }

    #[test]
    fn the_owner_index_map_is_the_published_pair() {
        assert_eq!(published_scalar(0), FIRST_SCALAR);
        assert_eq!(published_scalar(1), SECOND_SCALAR);
    }

    #[test]
    fn the_comparison_transcribes_every_term_of_the_representation_rule() {
        // Eleven, which is §6.6's own count. A term dropped here would
        // be a term the comparison stopped making, and the count is what
        // catches that.
        assert_eq!(REPRESENTATION_EQUIVALENCE_TERMS.len(), 11);
        let mut sorted = REPRESENTATION_EQUIVALENCE_TERMS.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), REPRESENTATION_EQUIVALENCE_TERMS.len());
    }

    #[test]
    fn the_run_of_record_is_honest_about_what_has_not_run() {
        // The two constants and the flag move together, and this is what
        // keeps them from drifting: a flag saying a ledger exists while
        // no identity is recorded would be the claim the whole run of
        // record discipline exists to prevent.
        let both = run_of_record::EXPLICIT_MEMBER_ACCEPTED_IDENTITY.is_some()
            && run_of_record::PRIVATE_MEMBER_ACCEPTED_IDENTITY.is_some();
        assert_eq!(run_of_record::A_PAIR_ARC_LEDGER_EXISTS, both);
    }
}
