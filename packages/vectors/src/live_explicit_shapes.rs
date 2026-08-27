//! The positive explicit shapes of §15.1, each as its own candidate
//! against a real node.
//!
//! # What this module is for
//!
//! §15.1 names sixteen positive explicit classes and the evidence plan
//! answered none of them. Not one was blocked on a missing component:
//! the owner message is computed, the explicit lane has had a
//! first-party spend accepted, and every one of the sixteen was waiting
//! on a run of its own shape that nobody had taken.
//!
//! This module takes those runs. It is the explicit-lane sibling of
//! [`crate::live_multi_shapes`], which did the same for the private
//! table, and it reuses the explicit spine
//! [`crate::live_owner_observation`] already established — issue a
//! disposable asset, relink the deployment against it, fund the
//! explicit destination constructor, finalize a candidate over the
//! funded coins, census the owners, sign, submit, and read the mined
//! bytes back.
//!
//! # A shape is what varies, and it is the only thing that varies
//!
//! The spine funds two coins and creates two outputs, fixed. Everything
//! a §15.1 row is *about* is one of those numbers or one of the owners
//! attached to them, so the ceremony takes the shape as a parameter:
//! how many coins each published owner is funded, how many destinations
//! are created and for whom, what each destination is worth, and in
//! which order the receipts are offered.
//!
//! Nothing else moves between shapes. Same deployment, same disposable
//! asset per run, same signing material, same message model, same
//! submission path — so two runs that differ are runs whose shapes
//! differ.
//!
//! # A row moves on an acceptance of its own shape and on nothing else
//!
//! The ceremony existing answers nothing. What answers a row is the
//! artifact a run produces: an identity the target computed for a
//! transaction of that row's own shape, the node's own copy of it read
//! back equal to the bytes it was handed, and every signature in that
//! copy verified against a message this workspace recomputed. Those
//! three are separate observations and the record keeps them separate.
//!
//! The recorded identities live in [`run_of_record`], and
//! [`crate::live_evidence`] cites them from there, so a reader
//! following a row's answer arrives at a value one execution against a
//! real node produced rather than at a claim in a source file.
//!
//! # What an explicit acceptance does not establish
//!
//! It says nothing about the proof-bearing lane, for the reason
//! [`crate::live_owner_observation`] states at length: an explicit
//! candidate's output-witness entries are default-constructed and so
//! recoverable from the preimage, and a candidate carrying real range
//! proofs is not.
//!
//! It also discharges no negative row. An accepted transfer is a
//! positive control; a refusal becomes attributable *because* of it,
//! but attributability is not an answer.
//!
//! # Every key here is published test material
//!
//! The two signing scalars are the BIP-340 specification's own appendix
//! secret keys, ADR-015 public disposable test material at each use.
//! They authorize nothing on any network anybody uses, and the chain
//! each run is taken against is created and destroyed by that run.

use std::fmt::Write as _;

use linker::live_backend::LiveTransferRepresentationPlan;
use target_elements_conformance::constructor::curve::FIELD_ELEMENT_BYTES;
use target_elements_conformance::executor::{OperationStep, PlanRefused, TargetOperationPlanner};
use target_elements_conformance::owner_key_oracle::verify_owner_signature;
use target_elements_conformance::protocol::{
    FundedOutput, MinedFundingReadback, NativeOperationResponse, ObservedOutcomeLayer,
    OperationCaseId, OperationSubject, TargetFundingSubject, TargetSubmissionSubject,
};
use transaction::bytes::{AssetField, AssetId, Outpoint, TargetTransaction, ValueField};
use transaction::live_abi::CandidateLiveTransferAbi;
use transaction::live_census::{LiveDeployment, OWNER_SIGNATURE_BYTES, OwnerSigningCensus};
use transaction::live_construct::{
    ExplicitDestinationRole, LiveConstructionReport, complete_live_transfer,
    finalize_live_transfer_declaring,
};
use transaction::live_finalize::FinalizedLiveTransfer;
use transaction::live_message::{WitnessVectorTreatment, candidate_owner_message};
use transaction::live_request::{
    LiveReceiptDestination, LiveTransferRequest, ProtocolValue, RequestedForm, SponsorChangeRequest,
};
use transaction::live_signing::{LiveOwnerResponse, authorize_live_transfer};
use transaction::taproot::Digest32;
use transaction::view::{PublicConstructionView, PublicOutputView};

use crate::error::VectorError;
use crate::live_capability::OracleLiveCurve;
use crate::live_owner_observation::{asset_of, decode_hex, outpoint_of, printed_order};
use crate::live_plan::{
    FEE_PROGRAM_DIGEST, FIRST_SCALAR, LiveShapeVocabulary, PROTOCOL_ASSET, RESERVE_ASSET,
    SECOND_SCALAR, live_abi_for_vocabulary, published_owner, reviewed_target, signing_material,
};

/// What each funded receipt holds.
///
/// The same figure every other live lane funds with. Keeping it equal
/// is what makes two runs' weights comparable; a shape that needed a
/// different amount would be varying two things at once.
const RECEIPT_AMOUNT: u64 = 5_000;

/// The fee the self-paying shape pays out of its own receipts.
///
/// The confidential sponsorless cell's figure, so the two sponsorless
/// halves of the owner fee matrix differ in their REPRESENTATION and not
/// in their arithmetic.
const SELF_PAID_FEE_AMOUNT: u64 = 250;

/// The fee-program digest one vocabulary's deployment is welded to.
///
/// The demonstration deployment is welded to a digest no program hashes
/// to, kept deliberately so its committed taptree does not move; nothing
/// had ever executed the clause that reads it. A fee-bearing deployment
/// must supply the digest of the fee program it actually constructs,
/// because the clause is about to be executed for the first time.
fn fee_program_digest_for(vocabulary: LiveShapeVocabulary) -> [u8; 32] {
    match vocabulary {
        LiveShapeVocabulary::Demonstration => FEE_PROGRAM_DIGEST,
        LiveShapeVocabulary::FeeBearing => crate::bundle::fee_program_digest(),
    }
}

/// The auxiliary value every signature here is taken with.
///
/// A published constant rather than randomness, on the spine's own
/// reasoning: BIP-340 masks the scalar with it before deriving the
/// nonce, so fixing it is what makes each signature reproducible from
/// values that are all written down. ADR-015 public disposable test
/// material.
const SHAPE_AUXILIARY: [u8; FIELD_ELEMENT_BYTES] = [0x37; FIELD_ELEMENT_BYTES];

/// Which published owner a coin or a destination belongs to.
///
/// Two members because the demonstration deployment publishes two
/// owners. The name travels into the transcript, so a reader can see
/// which owner a run put where without recomputing a key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShapeOwner {
    /// The first published owner.
    First,
    /// The second published owner.
    Second,
}

impl ShapeOwner {
    /// This owner's signing scalar. ADR-015 public disposable test
    /// material: the BIP-340 appendix keys, and nothing else.
    const fn scalar(self) -> &'static [u8; FIELD_ELEMENT_BYTES] {
        match self {
            Self::First => &FIRST_SCALAR,
            Self::Second => &SECOND_SCALAR,
        }
    }

    /// The name this owner is written down under.
    const fn name(self) -> &'static str {
        match self {
            Self::First => "first",
            Self::Second => "second",
        }
    }
}

/// The published owner a semantic fixture's owner INDEX names.
///
/// The deployment publishes two owners and a fixture states an index, so
/// this is the whole of the map between the two vocabularies. Anything
/// past the second is the second: the arc's fixture uses indices zero and
/// one, and a fixture that used a third would be refused by the linker
/// rather than silently mapped here — no third owner's constructor is in
/// this deployment's ABI at all.
const fn published_shape_owner(index: usize) -> ShapeOwner {
    if index == 0 {
        ShapeOwner::First
    } else {
        ShapeOwner::Second
    }
}

/// How a shape divides the consumed total among its destinations.
///
/// Two rules and no third, because §15.1 asks for exactly two things:
/// most rows are about cardinality or ownership and want the division
/// to be unremarkable, and one row is about the values themselves.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ValueRule {
    /// Equal shares, the remainder going to the last destination.
    EvenShares,
    /// The smallest destination the request type admits, and the rest.
    ///
    /// The smallest is one: [`ProtocolValue`] refuses zero by name, so
    /// one is the boundary rather than a small number somebody chose.
    /// The last destination takes the remainder, which for a two-output
    /// division is the largest value this shape's inputs can express.
    BoundaryExtremes,
}

/// One positive explicit class of §15.1, as a shape a run can take.
///
/// Each member names the row it is the shape of. Several members share
/// a cardinality and differ in what they are *about* — a two-input
/// transfer under one owner and a two-input transfer under two owners
/// have the same width and answer different rows — so the member is the
/// subject and never merely the counts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ExplicitShape {
    /// One receipt consumed, one created.
    OneToOne,
    /// One receipt consumed, split into two.
    SplitIntoTwo,
    /// Two receipts consumed, merged into one.
    MergedIntoOne,
    /// Two receipts consumed, two created.
    SeveralToSeveral,
    /// Two receipts under the SAME owner, three created.
    ///
    /// The subject is the repetition rather than the width: one owner
    /// authorizes two separate inputs, each at its own position and each
    /// over its own message.
    RepeatedOwner,
    /// Two receipts under two DISTINCT published owners.
    SeveralDistinctOwners,
    /// Two destinations, both belonging to one owner.
    OneDestinationOwner,
    /// Two destinations belonging to two distinct owners.
    SeveralDestinationOwners,
    /// Destinations at the boundary values the request type admits.
    SemanticBoundaryValues,
    /// The receipts offered in the reverse of their canonical order.
    CanonicalInputNormalization,
    /// A transfer carrying no sponsor region at all.
    Sponsorless,
    /// As many receipt inputs as the candidate's bounds admit.
    MaximumInputs,
    /// As many receipt outputs as the candidate's bounds admit.
    MaximumOutputs,
    /// A sponsorless transfer paying its OWN fee out of its receipts.
    ///
    /// The fourteenth shape and the only one that is not a §15.1 row.
    /// The explicit positive table is complete at sixteen rows and none
    /// of the sixteen is a self-paying transfer, so this shape answers
    /// the owner fee matrix rather than the table — see [`Self::row_name`].
    ///
    /// It is also the only shape built against the FEE-BEARING
    /// vocabulary. The demonstration bounds leave the fee axis off, so
    /// the demonstration deployment admits no shape bearing a fee, and
    /// widening those bounds would move its committed taptree and every
    /// digest recorded against it.
    SelfPaidFee,
    /// The EXPLICIT member of the pairs arc's §16.1 one-to-one pair.
    ///
    /// The fifteenth shape and the second that is not a §15.1 row. It is
    /// not a class of the explicit positive table at all: it is one
    /// materialization of ONE semantic fixture whose other
    /// materialization the private lane builds, and the two are a pair
    /// only because everything about them that is not the representation
    /// plan is read off that one fixture (§16.1).
    ///
    /// So its owners, its cardinalities and its AMOUNT are derived from
    /// [`crate::live_pair_arc::pair_arc_fixture`] rather than written
    /// here. A literal copied into this file would be a second author's
    /// statement of the fixture, which is the substitution §16.1's "one
    /// semantic fixture" exists to forbid.
    ///
    /// It is the one shape whose funding amount is not
    /// `RECEIPT_AMOUNT`, and that is the fixture's doing: the private
    /// member has to consume a coin the confidential predecessor already
    /// carries, so the fixture takes that coin's amount and the EXPLICIT
    /// side — the side a funding step can be asked for any amount — is
    /// the one that follows. Every other shape keeps
    /// `RECEIPT_AMOUNT` and therefore keeps its recorded bytes.
    PairedOneToOne,
}

impl ExplicitShape {
    /// Every shape this module runs.
    pub const ALL: &'static [Self] = &[
        Self::OneToOne,
        Self::SplitIntoTwo,
        Self::MergedIntoOne,
        Self::SeveralToSeveral,
        Self::RepeatedOwner,
        Self::SeveralDistinctOwners,
        Self::OneDestinationOwner,
        Self::SeveralDestinationOwners,
        Self::SemanticBoundaryValues,
        Self::CanonicalInputNormalization,
        Self::Sponsorless,
        Self::MaximumInputs,
        Self::MaximumOutputs,
        Self::SelfPaidFee,
        // APPENDED, and the position is load-bearing for the same reason
        // the private lane's own appended shape says: this array is read
        // positionally by tests that pin recorded bytes at an index, so a
        // shape added after a run goes on the end.
        Self::PairedOneToOne,
    ];

    /// The §15.1 row this shape is the shape of, where it is one.
    ///
    /// An OPTION, and the absent case is a fact about the table rather
    /// than a gap in this enum. §15.1's explicit positive table is
    /// complete at sixteen rows, and a sponsorless transfer that pays
    /// its own fee is not one of the sixteen classes: it answers the
    /// owner fee matrix, and its evidence is the run of record below.
    /// Naming it a row anyway would either invent a seventeenth or
    /// borrow another row's name for a run that did not answer it.
    #[must_use]
    pub const fn row_name(self) -> Option<&'static str> {
        match self {
            // Neither is a §15.1 class. The self-paying shape answers the
            // owner fee matrix; the paired member answers §16.1, whose
            // subject is a PAIR rather than a positive class, and naming
            // a row here would borrow one for a run that did not answer
            // it.
            Self::SelfPaidFee | Self::PairedOneToOne => None,
            Self::OneToOne => Some("one-input-to-one-output"),
            Self::SplitIntoTwo => Some("one-input-split-into-two"),
            Self::MergedIntoOne => Some("several-inputs-merged-into-one"),
            Self::SeveralToSeveral => Some("several-inputs-to-several-outputs"),
            Self::RepeatedOwner => Some("repeated-owner"),
            Self::SeveralDistinctOwners => Some("several-distinct-owners"),
            Self::OneDestinationOwner => Some("one-destination-owner"),
            Self::SeveralDestinationOwners => Some("several-destination-owners"),
            Self::SemanticBoundaryValues => Some("semantic-boundary-values"),
            Self::CanonicalInputNormalization => Some("canonical-input-normalization"),
            Self::Sponsorless => Some("sponsorless"),
            Self::MaximumInputs => Some("candidate-maximum-inputs"),
            Self::MaximumOutputs => Some("candidate-maximum-outputs"),
        }
    }

    /// The name a run's transcript is written under.
    #[must_use]
    pub const fn case_name(self) -> &'static str {
        match self {
            Self::OneToOne => "explicit-one-to-one",
            Self::SplitIntoTwo => "explicit-split",
            Self::MergedIntoOne => "explicit-merge",
            Self::SeveralToSeveral => "explicit-several-to-several",
            Self::RepeatedOwner => "explicit-repeated-owner",
            Self::SeveralDistinctOwners => "explicit-several-owners",
            Self::OneDestinationOwner => "explicit-one-destination-owner",
            Self::SeveralDestinationOwners => "explicit-several-destination-owners",
            Self::SemanticBoundaryValues => "explicit-boundary-values",
            Self::CanonicalInputNormalization => "explicit-normalization",
            Self::Sponsorless => "explicit-sponsorless",
            Self::MaximumInputs => "explicit-maximum-inputs",
            Self::MaximumOutputs => "explicit-maximum-outputs",
            Self::SelfPaidFee => "explicit-self-paid-fee",
            Self::PairedOneToOne => "explicit-paired-one-to-one",
        }
    }

    /// What each of this shape's funded receipts holds.
    ///
    /// `RECEIPT_AMOUNT` for every shape but the paired member, whose
    /// figure is the arc fixture's own source amount. The accessor exists
    /// so that the ONE shape whose amount the fixture fixes can say so
    /// without moving any other shape's bytes: fourteen shapes answer
    /// with the constant they always funded with.
    #[must_use]
    pub fn funded_amount(self) -> u64 {
        match self {
            Self::PairedOneToOne => crate::live_pair_arc::pair_arc_fixture()
                .sources()
                .first()
                .map_or(RECEIPT_AMOUNT, |endpoint| endpoint.amount()),
            _ => RECEIPT_AMOUNT,
        }
    }

    /// How many coins each published owner is funded, in order.
    ///
    /// A shape whose second entry is zero is funded in one step; the
    /// distinct-owners shape is the one that needs two, because two
    /// owners' explicit constructors are two different programs and a
    /// funding step names one program.
    #[must_use]
    pub const fn funded_coins(self) -> (u8, u8) {
        match self {
            Self::OneToOne
            | Self::SplitIntoTwo
            | Self::SemanticBoundaryValues
            | Self::Sponsorless
            | Self::SeveralDestinationOwners
            | Self::SelfPaidFee
            | Self::PairedOneToOne
            | Self::MaximumOutputs => (1, 0),
            Self::MergedIntoOne
            | Self::SeveralToSeveral
            | Self::RepeatedOwner
            | Self::OneDestinationOwner
            | Self::CanonicalInputNormalization => (2, 0),
            Self::SeveralDistinctOwners => (1, 1),
            Self::MaximumInputs => (3, 0),
        }
    }

    /// The owner each destination is created for, in output order.
    #[must_use]
    pub fn destination_owners(self) -> Vec<ShapeOwner> {
        match self {
            // The paired member reads its destinations off the arc's one
            // fixture rather than off a literal here, which is what makes
            // it a MATERIALIZATION of that fixture rather than a shape
            // that happens to resemble one.
            Self::PairedOneToOne => crate::live_pair_arc::pair_arc_fixture()
                .destinations()
                .iter()
                .map(|endpoint| published_shape_owner(endpoint.owner()))
                .collect(),
            // The single-destination shapes, spelled in one arm: what
            // separates them is their INPUT side or the order their
            // receipts are offered in, and none of that is here.
            Self::OneToOne
            | Self::MergedIntoOne
            | Self::CanonicalInputNormalization
            | Self::Sponsorless
            | Self::MaximumInputs => {
                vec![ShapeOwner::Second]
            }
            Self::SplitIntoTwo
            | Self::SeveralToSeveral
            | Self::SeveralDistinctOwners
            | Self::SeveralDestinationOwners
            | Self::SemanticBoundaryValues => {
                vec![ShapeOwner::Second, ShapeOwner::First]
            }
            // The destination, then the FEE entry. The fee names the
            // second owner only so the literal has the shape its
            // neighbours have -- a fee output carries no program at all,
            // which is most of what makes it a fee, and this owner is
            // never read. The confidential ceremony's fee entry says the
            // same thing for the same reason.
            Self::OneDestinationOwner | Self::SelfPaidFee => {
                vec![ShapeOwner::Second, ShapeOwner::Second]
            }
            Self::RepeatedOwner | Self::MaximumOutputs => {
                vec![ShapeOwner::Second, ShapeOwner::First, ShapeOwner::Second]
            }
        }
    }

    /// How the consumed total is divided.
    const fn value_rule(self) -> ValueRule {
        match self {
            Self::SemanticBoundaryValues => ValueRule::BoundaryExtremes,
            _ => ValueRule::EvenShares,
        }
    }

    /// Whether the receipts are offered in the reverse of the order the
    /// node reported them.
    #[must_use]
    pub const fn offers_reversed_receipts(self) -> bool {
        matches!(self, Self::CanonicalInputNormalization)
    }

    /// How many receipts this shape consumes.
    #[must_use]
    pub const fn input_count(self) -> usize {
        let (first, second) = self.funded_coins();
        (first as usize) + (second as usize)
    }

    /// How many destinations this shape creates.
    ///
    /// The self-paying shape's fee entry is counted here, because it IS
    /// a destination entry: it occupies an output position and states an
    /// amount. What separates it from the others is its declared role.
    #[must_use]
    pub fn output_count(self) -> usize {
        self.destination_owners().len()
    }

    /// The fee this shape pays out of its own receipts, where it pays one.
    ///
    /// Matched to the confidential ceremony's fee so the two sponsorless
    /// cells of the owner fee matrix are comparable at the amount as well
    /// as at the shape. A fee of zero is refused outright by the target,
    /// so there is no zero case to represent here.
    #[must_use]
    pub const fn self_paid_fee(self) -> Option<u64> {
        match self {
            Self::SelfPaidFee => Some(SELF_PAID_FEE_AMOUNT),
            _ => None,
        }
    }

    /// Which deployment vocabulary this shape's ceremony links against.
    ///
    /// Only the self-paying shape leaves the demonstration vocabulary,
    /// and the separation is what keeps every recorded demonstration
    /// digest and the demonstration taptree untouched by the fee axis.
    #[must_use]
    pub const fn vocabulary(self) -> LiveShapeVocabulary {
        match self {
            Self::SelfPaidFee => LiveShapeVocabulary::FeeBearing,
            _ => LiveShapeVocabulary::Demonstration,
        }
    }

    /// The role of each destination entry, in output order.
    ///
    /// EMPTY for every shape that declares nothing, which is what the
    /// explicit lane's other thirteen shapes do: an empty declaration is
    /// every entry a receipt output, and passing it leaves those runs
    /// building exactly the bytes they built before this shape existed.
    #[must_use]
    pub fn declared_roles(self) -> Vec<ExplicitDestinationRole> {
        let Some(_) = self.self_paid_fee() else {
            return Vec::new();
        };
        let mut roles = vec![ExplicitDestinationRole::ReceiptOutput; self.output_count()];
        // The fee is LAST, which is where the sponsorless shape's fee
        // position is: immediately after the destinations, no change role
        // being able to sit between them without a sponsor region.
        if let Some(last) = roles.last_mut() {
            *last = ExplicitDestinationRole::Fee;
        }
        roles
    }
}

/// Why a shape run stopped before the node answered.
///
/// A construction refusal is this workspace declining to build
/// something, and it is never a target verdict. The two are kept apart
/// so a run that stopped here cannot be read as a chain having said
/// anything.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ExplicitShapeRefusal {
    /// The candidate substrate did not build.
    SubstrateUnavailable,
    /// The issuing step named no asset to relink against.
    IssuanceNamedNoAsset,
    /// The relink against the issued asset was refused.
    RelinkRefused,
    /// The funding step created no predecessor.
    FundingCreatedNoPredecessor,
    /// The node reported a funded output this ceremony could not read.
    MalformedFundedOutput,
    /// The node funded a different number of coins than the shape asked
    /// for.
    FundedWidthDisagrees,
    /// The candidate did not finalize over the funded coins.
    CandidateNotConstructible,
    /// An owner census over the finalized candidate was refused.
    CensusRefused,
    /// A signing request named an owner this deployment does not
    /// publish.
    SigningOwnerUnpublished,
    /// The signing material refused to sign.
    SigningRefused,
    /// The target accepted a transaction and reported no copy of it.
    AcceptanceCarriedNoReadback,
    /// The node's own copy did not decode.
    ReadbackDidNotDecode,
    /// The node's own copy carried no witness at an input position.
    ReadbackCarriesNoWitness,
}

/// One funded coin, as the node reported it and as the shape expected
/// it.
#[derive(Clone, Debug)]
pub struct ObservedShapeCoin {
    outpoint: Outpoint,
    asset: AssetField,
    value: ValueField,
    program: Vec<u8>,
    owner: ShapeOwner,
    matches_expectation: bool,
}

impl ObservedShapeCoin {
    /// Whether the node reported the coin the ceremony asked for.
    #[must_use]
    pub const fn matches_expectation(&self) -> bool {
        self.matches_expectation
    }

    /// The owner whose explicit constructor this coin was paid to.
    #[must_use]
    pub const fn owner(&self) -> ShapeOwner {
        self.owner
    }

    /// The outpoint the node reported.
    #[must_use]
    pub const fn outpoint(&self) -> Outpoint {
        self.outpoint
    }
}

/// One input's signature, taken out of the node's own copy and checked
/// against a message this workspace recomputed.
///
/// Per input rather than per transaction, and deliberately: a
/// multi-input candidate whose first signature verified would tell a
/// reader nothing about its second, and the shapes this module runs are
/// mostly multi-input.
#[derive(Clone, Debug)]
pub struct InputVerification {
    input_index: usize,
    owner: ShapeOwner,
    recomputed_message: [u8; 32],
    signature_bytes: usize,
    verified: bool,
}

impl InputVerification {
    /// Whether the accepted signature verifies against the recomputed
    /// message.
    #[must_use]
    pub const fn verified(&self) -> bool {
        self.verified
    }

    /// Which input this is.
    #[must_use]
    pub const fn input_index(&self) -> usize {
        self.input_index
    }
}

/// What reading the accepted transaction back established.
#[derive(Clone, Debug)]
pub struct ShapeReverification {
    accepted_txid: String,
    witness_txid: String,
    block_height: u32,
    readback_matches_submission: bool,
    inputs: Vec<InputVerification>,
}

impl ShapeReverification {
    /// Whether the bytes the node reported are the bytes it was handed.
    #[must_use]
    pub const fn readback_matches_submission(&self) -> bool {
        self.readback_matches_submission
    }

    /// Whether every input's accepted signature verifies against its own
    /// recomputed message.
    #[must_use]
    pub fn every_input_verified(&self) -> bool {
        !self.inputs.is_empty() && self.inputs.iter().all(InputVerification::verified)
    }

    /// The identity the target computed.
    #[must_use]
    pub fn accepted_txid(&self) -> &str {
        &self.accepted_txid
    }

    /// The per-input verifications.
    #[must_use]
    pub fn inputs(&self) -> &[InputVerification] {
        &self.inputs
    }
}

/// What one negative case replaces in the witness it offers.
///
/// §15.3's witness-content rows change what the witness OFFERS rather
/// than what the builder assembled, and §10.2 types the signature
/// position as an unconstrained item precisely so that the target is the
/// thing that refuses an empty or a malformed offering. So these
/// mutations are applied after the candidate is finalized and censused,
/// and the bytes reach a node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ExplicitWitnessMutation {
    /// The signature position offers nothing at all.
    EmptySignature,
    /// The signature position offers bytes of the right width that are
    /// not a signature.
    ///
    /// The width is kept so that a refusal cannot be attributed to the
    /// length: what changed is the CONTENT of a well-sized offering, and
    /// a run that also shortened it would have moved two things.
    MalformedSignature,
}

impl ExplicitWitnessMutation {
    /// Every mutation this ceremony offers.
    pub const ALL: &'static [Self] = &[Self::EmptySignature, Self::MalformedSignature];

    /// The §15.3 row this mutation is the mutation of.
    #[must_use]
    pub const fn row_name(self) -> &'static str {
        match self {
            Self::EmptySignature => "empty-signature",
            Self::MalformedSignature => "malformed-signature",
        }
    }

    /// The name this case is written down under.
    #[must_use]
    pub const fn case_name(self) -> &'static str {
        match self {
            Self::EmptySignature => "empty-signature",
            Self::MalformedSignature => "malformed-signature",
        }
    }

    /// The bytes this mutation offers in place of a signature.
    ///
    /// The malformed offering is a fixed published pattern of the
    /// selected width. ADR-015 public disposable test material: it
    /// authorizes nothing, which is the entire point of offering it.
    fn offering(self, width: usize) -> Vec<u8> {
        match self {
            Self::EmptySignature => Vec::new(),
            Self::MalformedSignature => vec![0xff; width],
        }
    }
}

/// What the target did with one mutated candidate.
#[derive(Clone, Debug)]
pub struct NegativeObservation {
    mutation: ExplicitWitnessMutation,
    submitted_bytes: usize,
    offered_signature_bytes: usize,
    layer: ObservedOutcomeLayer,
    detail: Option<String>,
    accepted_txid: Option<String>,
    differs_from_control_in_one_item: bool,
    control_difference_bytes: usize,
    mutant_difference_bytes: usize,
}

impl NegativeObservation {
    /// Which mutation this was.
    #[must_use]
    pub const fn mutation(&self) -> ExplicitWitnessMutation {
        self.mutation
    }

    /// The layer the target answered at.
    #[must_use]
    pub const fn layer(&self) -> ObservedOutcomeLayer {
        self.layer
    }

    /// Whether the target refused it.
    #[must_use]
    pub const fn refused(&self) -> bool {
        !matches!(self.layer, ObservedOutcomeLayer::Accepted)
    }

    /// What the target said.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    /// Whether these bytes differ from the control's, and do so only
    /// within the witness item the mutation replaced.
    ///
    /// Computed by comparing the two submissions rather than argued from
    /// the code that built them. The size bound is where the content is:
    /// any two differing strings differ in one contiguous region, so
    /// what has to be checked is that the region FITS inside one
    /// signature item and its length prefix.
    #[must_use]
    pub const fn differs_from_control_in_one_item(&self) -> bool {
        self.differs_from_control_in_one_item
    }
}

/// One shape run's record.
#[derive(Clone, Debug)]
pub struct ExplicitShapeRecord {
    shape: ExplicitShape,
    issued_asset: Option<String>,
    relinked: bool,
    coins: Vec<ObservedShapeCoin>,
    offered_outpoints: Vec<Outpoint>,
    canonical_outpoints: Vec<Outpoint>,
    destination_amounts: Vec<u64>,
    destination_owners: Vec<ShapeOwner>,
    input_owners: Vec<ShapeOwner>,
    submitted_bytes: usize,
    observed_weight: Option<u64>,
    observed_layer: Option<ObservedOutcomeLayer>,
    observed_detail: Option<String>,
    accepted_txid: Option<String>,
    reverification: Option<ShapeReverification>,
    control_bytes: Vec<u8>,
    offered_signature_bytes: usize,
    negatives: Vec<NegativeObservation>,
    refusal: Option<ExplicitShapeRefusal>,
}

impl ExplicitShapeRecord {
    /// The shape this run took.
    #[must_use]
    pub const fn shape(&self) -> ExplicitShape {
        self.shape
    }

    /// The disposable asset this run issued and relinked against.
    ///
    /// Read by a caller that has to hand the SAME asset to a second
    /// ceremony: §6.6 asks a §16.1 pair's two members to carry one exact
    /// explicit `U`, and a second issuance would be a second asset.
    #[must_use]
    pub fn issued_asset(&self) -> Option<&str> {
        self.issued_asset.as_deref()
    }

    /// The coins the node funded.
    #[must_use]
    pub fn coins(&self) -> &[ObservedShapeCoin] {
        &self.coins
    }

    /// How many bytes the ceremony handed the node.
    #[must_use]
    pub const fn submitted_bytes(&self) -> usize {
        self.submitted_bytes
    }

    /// The weight the TARGET reported for the submitted transaction.
    ///
    /// Absent where the adapter reported none, which is an unmade
    /// measurement rather than a weight of zero.
    #[must_use]
    pub const fn observed_weight(&self) -> Option<u64> {
        self.observed_weight
    }

    /// The layer the target answered at.
    #[must_use]
    pub const fn observed_layer(&self) -> Option<ObservedOutcomeLayer> {
        self.observed_layer
    }

    /// The identity the target computed, where it accepted.
    #[must_use]
    pub fn accepted_txid(&self) -> Option<&str> {
        self.accepted_txid.as_deref()
    }

    /// What reading the accepted transaction back established.
    #[must_use]
    pub const fn reverification(&self) -> Option<&ShapeReverification> {
        self.reverification.as_ref()
    }

    /// What the target did with each mutated candidate.
    #[must_use]
    pub fn negatives(&self) -> &[NegativeObservation] {
        &self.negatives
    }

    /// The construction refusal that stopped the run, if one did.
    #[must_use]
    pub const fn refusal(&self) -> Option<ExplicitShapeRefusal> {
        self.refusal
    }

    /// How many receipts were consumed.
    #[must_use]
    pub const fn input_count(&self) -> usize {
        self.input_owners.len()
    }

    /// How many destinations were created.
    #[must_use]
    pub const fn output_count(&self) -> usize {
        self.destination_amounts.len()
    }

    /// Whether the receipts were offered in a non-canonical order.
    ///
    /// The normalization row's own observable. It is read off the two
    /// recorded orders rather than declared by the shape, so a run in
    /// which the offered order happened to be canonical says so.
    #[must_use]
    pub fn offered_order_differs(&self) -> bool {
        self.offered_outpoints != self.canonical_outpoints
    }

    /// What this run establishes nothing about.
    #[must_use]
    pub const fn non_claims() -> [&'static str; 4] {
        [
            "evidences no negative case: an accepted transfer is a positive control, and a \
             refusal is attributable because of it rather than answered by it",
            "evidences nothing about the proof-bearing lane: an explicit candidate's \
             output-witness entries are default-constructed and a proof-bearing one's are not",
            "evidences no sponsor row: this ceremony builds no sponsor region at all",
            "claims nothing about any deployment but the one this run created and destroyed",
        ]
    }
}

/// What the ceremony is doing next.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Stage {
    /// Issue the disposable asset the deployment is then linked against.
    Issue,
    /// Pay the issued asset to the first owner's explicit constructor.
    FundFirst,
    /// Pay the issued asset to the second owner's explicit constructor.
    FundSecond,
    /// Submit the case at this position of the case list.
    Submit(usize),
    /// Nothing further.
    Done,
}

/// The §15.1 shape ceremony.
pub struct ExplicitShapePlanner {
    stage: Stage,
    shape: ExplicitShape,
    cases: Vec<CaseKind>,
    abi: CandidateLiveTransferAbi,
    genesis_block_hash: Digest32,
    pending: Option<Vec<u8>>,
    record: ExplicitShapeRecord,
}

/// One submission a run makes.
///
/// A run that submits only its control is a positive shape run; a run
/// that submits the control AND its mutants is the negative half, and
/// the two live in one ceremony because that is what makes a refusal
/// attributable: same chain, same deployment, same funded coins, same
/// finalized candidate, and one witness item different.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CaseKind {
    /// The unmutated candidate.
    Control,
    /// The same candidate with one witness item replaced.
    Mutated(ExplicitWitnessMutation),
}

impl ExplicitShapePlanner {
    /// The ceremony for one shape, bound to one deployment's printed
    /// genesis identity.
    ///
    /// The identity arrives in the spelling the target prints and is
    /// reversed into the seed the target hashes with, the same relation
    /// the explicit spine applies at the same term.
    ///
    /// # Errors
    ///
    /// [`VectorError::LiveSubstrateUnavailable`] when the candidate ABI
    /// is unavailable.
    pub fn for_shape(
        shape: ExplicitShape,
        printed_genesis_identity: Digest32,
    ) -> Result<Self, VectorError> {
        let abi = live_abi_for_vocabulary(
            shape.vocabulary(),
            PROTOCOL_ASSET,
            RESERVE_ASSET,
            fee_program_digest_for(shape.vocabulary()),
        )?;
        Ok(Self {
            stage: Stage::Issue,
            shape,
            cases: vec![CaseKind::Control],
            abi,
            genesis_block_hash: printed_order(printed_genesis_identity),
            pending: None,
            record: ExplicitShapeRecord {
                shape,
                issued_asset: None,
                relinked: false,
                coins: Vec::new(),
                offered_outpoints: Vec::new(),
                canonical_outpoints: Vec::new(),
                destination_amounts: Vec::new(),
                destination_owners: Vec::new(),
                input_owners: Vec::new(),
                submitted_bytes: 0,
                observed_weight: None,
                observed_layer: None,
                observed_detail: None,
                accepted_txid: None,
                reverification: None,
                control_bytes: Vec::new(),
                offered_signature_bytes: 0,
                negatives: Vec::new(),
                refusal: None,
            },
        })
    }

    /// The ceremony for §15.3's witness-content negatives.
    ///
    /// One run submits THREE candidates to one node on one chain: the
    /// one-input one-output candidate with its signature position
    /// offering nothing, the same one again with the position offering
    /// bytes of the selected width that are not a signature, and THEN
    /// the unmutated control.
    ///
    /// # Why all three are in one run
    ///
    /// A refusal is attributable to a row's own class only when the
    /// UNMUTATED form is accepted and the mutated form is refused, and
    /// "accepted" has to mean accepted by the same node, on the same
    /// chain, over the same funded coins, in the same session. A control
    /// accepted last week on a chain that no longer exists would leave a
    /// refusal explicable by anything that changed in between.
    ///
    /// # Why the mutants go FIRST, which is not a preference
    ///
    /// The order was the other way round and the run that took it is
    /// what corrected it. A witness-content mutation changes the WITNESS
    /// and the witness is not part of a transaction's identity, so a
    /// mutant has the SAME identity as its control. With the control
    /// submitted and mined first, both mutants came back refused
    /// `txn-already-known` at a layer before script evaluation: a true
    /// refusal, naming the identity that was already on the chain, and
    /// attributable to the submission order rather than to anything the
    /// witness offered. Nothing about §15.3 could be read off it.
    ///
    /// So the mutants are offered while no transaction of that identity
    /// exists yet, and the control follows. Each verdict then belongs to
    /// the bytes that earned it.
    ///
    /// The control is the one-to-one shape for a stated reason: it has
    /// exactly one input, so there is exactly one signature position and
    /// no question about which one moved.
    ///
    /// # Errors
    ///
    /// [`VectorError::LiveSubstrateUnavailable`] when the candidate ABI
    /// is unavailable.
    pub fn for_witness_negatives(printed_genesis_identity: Digest32) -> Result<Self, VectorError> {
        let mut planner = Self::for_shape(ExplicitShape::OneToOne, printed_genesis_identity)?;
        planner.cases = ExplicitWitnessMutation::ALL
            .iter()
            .copied()
            .map(CaseKind::Mutated)
            .chain(std::iter::once(CaseKind::Control))
            .collect();
        Ok(planner)
    }

    /// The ceremony's own record of the run.
    #[must_use]
    pub const fn record(&self) -> &ExplicitShapeRecord {
        &self.record
    }

    /// Record one refusal and stop.
    const fn refuse(&mut self, refusal: ExplicitShapeRefusal) -> PlanRefused {
        self.record.refusal = Some(refusal);
        self.stage = Stage::Done;
        PlanRefused
    }

    /// One owner's explicit destination program under the current link.
    fn program_for(&self, owner: ShapeOwner) -> Result<Vec<u8>, ExplicitShapeRefusal> {
        explicit_destination_program(&self.abi, owner)
            .map_err(|_| ExplicitShapeRefusal::SubstrateUnavailable)
    }

    /// A funding step paying `count` coins to one owner's constructor.
    fn funding_step(
        &self,
        name: &'static str,
        issue: bool,
        owner: ShapeOwner,
        count: u8,
    ) -> Result<OperationStep, ExplicitShapeRefusal> {
        Ok(OperationStep::new(
            name,
            OperationSubject::Funding(Box::new(TargetFundingSubject {
                issue_asset: issue,
                asset: if issue {
                    None
                } else {
                    self.record.issued_asset.clone()
                },
                output_program: self.program_for(owner)?,
                outputs: count,
                amount_per_output: self.shape.funded_amount(),
            })),
        ))
    }

    /// Link the deployment against the asset the target issued.
    fn relink(&mut self, response: &NativeOperationResponse) -> Result<(), ExplicitShapeRefusal> {
        let asset = response
            .issued_asset
            .clone()
            .ok_or(ExplicitShapeRefusal::IssuanceNamedNoAsset)?;
        let identity = asset_of(&asset).ok_or(ExplicitShapeRefusal::IssuanceNamedNoAsset)?;
        let vocabulary = self.shape.vocabulary();
        let abi = live_abi_for_vocabulary(
            vocabulary,
            *identity.internal(),
            RESERVE_ASSET,
            fee_program_digest_for(vocabulary),
        )
        .map_err(|_| ExplicitShapeRefusal::RelinkRefused)?;
        self.record.issued_asset = Some(asset);
        self.record.relinked = true;
        self.abi = abi;
        Ok(())
    }

    /// Take one funding answer's coins, as the node reported them.
    ///
    /// Every field is the node's. The spent asset, value and program are
    /// terms of the owner message, so a signature over what the builder
    /// assumed rather than over what the chain holds would be a
    /// signature over a different message than the one the target forms.
    fn settle_funding(
        &mut self,
        owner: ShapeOwner,
        response: &NativeOperationResponse,
    ) -> Result<(), ExplicitShapeRefusal> {
        if response.funded_outputs.is_empty() {
            return Err(ExplicitShapeRefusal::FundingCreatedNoPredecessor);
        }
        let expected_asset = self
            .record
            .issued_asset
            .as_deref()
            .and_then(asset_of)
            .ok_or(ExplicitShapeRefusal::IssuanceNamedNoAsset)?;
        let program = self.program_for(owner)?;
        let expected_amount = self.shape.funded_amount();
        for funded in &response.funded_outputs {
            let coin = observed_coin(funded, expected_asset, &program, owner, expected_amount)?;
            self.record.coins.push(coin);
        }
        Ok(())
    }

    /// The public view the candidate is constructed against.
    fn view(&self) -> Result<PublicConstructionView, ExplicitShapeRefusal> {
        PublicConstructionView::new(self.record.coins.iter().map(|coin| {
            PublicOutputView::new(coin.outpoint, coin.asset, coin.value, coin.program.clone())
        }))
        .map_err(|_| ExplicitShapeRefusal::CandidateNotConstructible)
    }

    /// The outpoints the shape offers, in the order it offers them.
    fn offered_outpoints(&self) -> Vec<Outpoint> {
        let mut points: Vec<Outpoint> = self
            .record
            .coins
            .iter()
            .map(ObservedShapeCoin::outpoint)
            .collect();
        if self.shape.offers_reversed_receipts() {
            points.reverse();
        }
        points
    }

    /// The destination values this shape asks for.
    ///
    /// Computed from the total the funded coins actually carry rather
    /// than from what the ceremony asked to be funded, because the
    /// division has to conserve against the chain's coins and not
    /// against an expectation.
    fn destination_amounts(&self, total: u64, outputs: usize) -> Option<Vec<u64>> {
        let width = u64::try_from(outputs).ok()?;
        if width == 0 || total < width {
            return None;
        }
        // A self-paying shape divides only what is LEFT after the fee,
        // and states the fee as its last entry. Dividing the whole total
        // and calling one share the fee would make the fee a function of
        // the coin the node happened to fund.
        if let Some(fee) = self.shape.self_paid_fee() {
            let remaining = total.checked_sub(fee)?;
            if remaining == 0 || outputs < 2 {
                return None;
            }
            let destinations = outputs - 1;
            let width = u64::try_from(destinations).ok()?;
            let share = remaining / width;
            if share == 0 {
                return None;
            }
            let mut amounts = vec![share; destinations];
            let last = amounts.last_mut()?;
            *last = remaining - share * (width - 1);
            amounts.push(fee);
            return Some(amounts);
        }
        Some(match self.shape.value_rule() {
            ValueRule::EvenShares => {
                let share = total / width;
                let mut amounts = vec![share; outputs];
                let last = amounts.last_mut()?;
                *last = total - share * (width - 1);
                amounts
            }
            ValueRule::BoundaryExtremes => {
                let mut amounts = vec![1_u64; outputs];
                let last = amounts.last_mut()?;
                *last = total - (width - 1);
                amounts
            }
        })
    }

    /// The finalized candidate this shape's request produces, with the
    /// construction report completion needs.
    fn finalize(
        &mut self,
    ) -> Result<(FinalizedLiveTransfer, LiveConstructionReport), ExplicitShapeRefusal> {
        let view = self.view()?;
        let points = self.offered_outpoints();
        let total = self
            .record
            .coins
            .iter()
            .try_fold(0_u64, |sum, coin| match coin.value {
                ValueField::Explicit(amount) => sum.checked_add(amount),
                // Any value field but an explicit one is a coin this
                // ceremony cannot total, and this is the explicit lane.
                // A guessed total would be a conservation claim about
                // bytes nothing here can read.
                _ => None,
            })
            .ok_or(ExplicitShapeRefusal::CandidateNotConstructible)?;

        let owners = self.shape.destination_owners();
        let amounts = self
            .destination_amounts(total, owners.len())
            .ok_or(ExplicitShapeRefusal::CandidateNotConstructible)?;

        let mut destinations = Vec::with_capacity(owners.len());
        for (owner, amount) in owners.iter().copied().zip(amounts.iter().copied()) {
            let published = published_owner(owner.scalar())
                .map_err(|_| ExplicitShapeRefusal::SubstrateUnavailable)?;
            let value = ProtocolValue::new(amount)
                .map_err(|_| ExplicitShapeRefusal::CandidateNotConstructible)?;
            destinations.push(LiveReceiptDestination::new(
                linker::OwnerParameter::new(published),
                value,
            ));
        }

        let request = LiveTransferRequest::new(
            points.clone(),
            destinations,
            LiveTransferRepresentationPlan::Explicit,
            RequestedForm::Sponsorless,
            SponsorChangeRequest::NotRequested,
            None,
        )
        .map_err(|_| ExplicitShapeRefusal::CandidateNotConstructible)?;

        let target = reviewed_target().map_err(|_| ExplicitShapeRefusal::SubstrateUnavailable)?;
        let finalization = finalize_live_transfer_declaring(
            &target,
            &self.abi,
            &request,
            &view,
            None,
            None,
            &self.shape.declared_roles(),
        )
        .map_err(|_| ExplicitShapeRefusal::CandidateNotConstructible)?;

        self.record.offered_outpoints = points;
        self.record.canonical_outpoints = request.receipts().iter().copied().collect();
        self.record.destination_amounts = amounts;
        self.record.destination_owners = owners;
        let report = finalization.report().clone();
        Ok((finalization.into_finalized(), report))
    }

    /// The owner census of one finalized candidate, under one
    /// deployment.
    fn census(
        finalized: &FinalizedLiveTransfer,
        genesis: Digest32,
    ) -> Result<OwnerSigningCensus, ExplicitShapeRefusal> {
        let target = reviewed_target().map_err(|_| ExplicitShapeRefusal::SubstrateUnavailable)?;
        let curve = OracleLiveCurve::new(
            reviewed_target().map_err(|_| ExplicitShapeRefusal::SubstrateUnavailable)?,
        );
        OwnerSigningCensus::from_explicit_finalized(
            &target,
            finalized,
            LiveDeployment::new(genesis),
            &curve,
        )
        .map_err(|_| ExplicitShapeRefusal::CensusRefused)
    }

    /// Which published owner one signing request names.
    fn owner_of(request_owner: &linker::OwnerParameter) -> Option<ShapeOwner> {
        for candidate in [ShapeOwner::First, ShapeOwner::Second] {
            let published = published_owner(candidate.scalar()).ok()?;
            if linker::OwnerParameter::new(published) == *request_owner {
                return Some(candidate);
            }
        }
        None
    }

    /// The shape's candidate bytes, signed by whichever owner holds each
    /// input.
    fn candidate_bytes(
        &mut self,
        mutation: Option<ExplicitWitnessMutation>,
    ) -> Result<(Vec<u8>, usize), ExplicitShapeRefusal> {
        let (finalized, report) = self.finalize()?;
        let census = Self::census(&finalized, self.genesis_block_hash)?;

        let mut input_owners = Vec::new();
        let mut responses = Vec::new();
        let mut offered_bytes = 0_usize;
        for signing in finalized.signing_requests() {
            let owner = Self::owner_of(signing.owner())
                .ok_or(ExplicitShapeRefusal::SigningOwnerUnpublished)?;
            let input = census
                .signing_inputs()
                .iter()
                .find(|entry| entry.input_index() == u32::from(signing.input()))
                .ok_or(ExplicitShapeRefusal::CandidateNotConstructible)?;
            let message =
                candidate_owner_message(&census, input, WitnessVectorTreatment::BothGrown);
            let material = signing_material(owner.scalar())
                .map_err(|_| ExplicitShapeRefusal::SubstrateUnavailable)?;
            let signature = material
                .sign(&message, &SHAPE_AUXILIARY)
                .map_err(|_| ExplicitShapeRefusal::SigningRefused)?
                .to_vec();
            // The mutation replaces what the FIRST input's signature
            // position offers and leaves every other input alone, so a
            // multi-input control could not have its refusal attributed
            // to a second change. These cases run over a one-input
            // control anyway, and the rule is written into the code
            // rather than left to the shape's choice.
            let offered = match mutation {
                Some(change) if signing.input() == 0 => {
                    let replacement = change.offering(signature.len());
                    offered_bytes = replacement.len();
                    replacement
                }
                _ => signature,
            };
            input_owners.push((usize::from(signing.input()), owner));
            responses.push((signing.input(), LiveOwnerResponse::to(&signing, offered)));
        }

        input_owners.sort_unstable_by_key(|(index, _)| *index);
        self.record.input_owners = input_owners.into_iter().map(|(_, owner)| owner).collect();

        let report_target =
            reviewed_target().map_err(|_| ExplicitShapeRefusal::SubstrateUnavailable)?;
        let authorized = authorize_live_transfer(finalized, responses)
            .map_err(|_| ExplicitShapeRefusal::CandidateNotConstructible)?;
        let built = complete_live_transfer(&report_target, authorized, report, None)
            .map_err(|_| ExplicitShapeRefusal::CandidateNotConstructible)?;
        Ok((built.bytes(), offered_bytes))
    }

    /// Record what the target did with one case.
    ///
    /// Whatever the layer was. Nothing here compares the answer against
    /// what the case intended: a mutant the node ACCEPTED would be a
    /// finding written into the transcript rather than a panic that hid
    /// it, which is the same discipline the explicit spine applies to
    /// its own controls.
    fn settle_case(
        &mut self,
        position: usize,
        response: &NativeOperationResponse,
    ) -> Result<(), ExplicitShapeRefusal> {
        let submitted = self
            .pending
            .take()
            .ok_or(ExplicitShapeRefusal::CandidateNotConstructible)?;
        let case = self
            .cases
            .get(position)
            .copied()
            .ok_or(ExplicitShapeRefusal::CandidateNotConstructible)?;
        if let CaseKind::Mutated(mutation) = case {
            // How many BYTES of the two submissions differ. The control
            // and the mutant are the same finalized candidate with one
            // witness item replaced, so a difference confined to that
            // item is what makes the target's change of mind
            // attributable -- and it is measured against the control's
            // actual bytes rather than asserted from the code.
            let control = &self.record.control_bytes;
            let (control_run, mutant_run) = difference_runs(control, &submitted);
            let one_item = (control_run > 0 || mutant_run > 0)
                && control_run <= WITNESS_ITEM_BOUND
                && mutant_run <= WITNESS_ITEM_BOUND;
            self.record.negatives.push(NegativeObservation {
                mutation,
                submitted_bytes: submitted.len(),
                offered_signature_bytes: self.record.offered_signature_bytes,
                layer: response.observed_layer,
                detail: response.observed_detail.clone(),
                accepted_txid: response.accepted_txid.clone(),
                differs_from_control_in_one_item: one_item,
                control_difference_bytes: control_run,
                mutant_difference_bytes: mutant_run,
            });
            return Ok(());
        }

        // The control's own bytes, which a mutant-first run has already
        // computed locally. Asserting the two agree would be comparing a
        // value with itself, so it is simply set.
        self.record.control_bytes.clone_from(&submitted);
        self.record.observed_layer = Some(response.observed_layer);
        // The target's OWN weight, read off `decoderawtransaction` by
        // the adapter rather than computed here. §20.5 wants the
        // comparison made against the node's figure, and a weight this
        // ceremony calculated for itself would be comparing a prediction
        // with itself.
        self.record.observed_weight = response.resources.transaction_weight;
        self.record
            .observed_detail
            .clone_from(&response.observed_detail);
        self.record
            .accepted_txid
            .clone_from(&response.accepted_txid);

        if matches!(response.observed_layer, ObservedOutcomeLayer::Accepted) {
            let readback = response
                .mined_readback
                .as_ref()
                .ok_or(ExplicitShapeRefusal::AcceptanceCarriedNoReadback)?;
            let record = self.reverify(readback, &submitted)?;
            self.record.reverification = Some(record);
        }
        Ok(())
    }

    /// Read the accepted transaction back and verify every input's
    /// signature against a recomputed message.
    fn reverify(
        &mut self,
        readback: &MinedFundingReadback,
        submitted: &[u8],
    ) -> Result<ShapeReverification, ExplicitShapeRefusal> {
        let decoded = TargetTransaction::decode(&readback.raw_transaction)
            .map_err(|_| ExplicitShapeRefusal::ReadbackDidNotDecode)?;
        let (finalized, _report) = self.finalize()?;
        let census = Self::census(&finalized, self.genesis_block_hash)?;
        let target = reviewed_target().map_err(|_| ExplicitShapeRefusal::SubstrateUnavailable)?;

        let mut inputs = Vec::new();
        for signing in finalized.signing_requests() {
            let index = usize::from(signing.input());
            let owner = Self::owner_of(signing.owner())
                .ok_or(ExplicitShapeRefusal::SigningOwnerUnpublished)?;
            let entry = census
                .signing_inputs()
                .iter()
                .find(|entry| entry.input_index() == u32::from(signing.input()))
                .ok_or(ExplicitShapeRefusal::CandidateNotConstructible)?;
            let recomputed =
                candidate_owner_message(&census, entry, WitnessVectorTreatment::BothGrown);
            let signature = decoded
                .witnesses()
                .get(index)
                .and_then(|witness| witness.stack().first())
                .cloned()
                .ok_or(ExplicitShapeRefusal::ReadbackCarriesNoWitness)?;
            let published = published_owner(owner.scalar())
                .map_err(|_| ExplicitShapeRefusal::SubstrateUnavailable)?;
            let verified =
                verify_owner_signature(&target, published.bytes(), &recomputed, &signature).is_ok();
            inputs.push(InputVerification {
                input_index: index,
                owner,
                recomputed_message: recomputed,
                signature_bytes: signature.len(),
                verified,
            });
        }
        inputs.sort_unstable_by_key(|entry| entry.input_index);

        Ok(ShapeReverification {
            accepted_txid: readback.transaction_id.clone(),
            witness_txid: readback.witness_transaction_id.clone(),
            block_height: readback.block_height,
            readback_matches_submission: readback.raw_transaction == submitted,
            inputs,
        })
    }
}

impl TargetOperationPlanner for ExplicitShapePlanner {
    fn next_step(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<OperationStep>, PlanRefused> {
        self.advance(previous)?;
        self.emit_step()
    }
}

impl ExplicitShapePlanner {
    /// Settle the previous step's answer and choose the next stage.
    ///
    /// Split from the step the ceremony emits because the two are
    /// different jobs: this one reads what the target said, and the
    /// other decides what to ask next.
    fn advance(
        &mut self,
        previous: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<(), PlanRefused> {
        let (_first_coins, second_coins) = self.shape.funded_coins();
        if let Some((_case, response)) = previous {
            match self.stage {
                Stage::Issue => {
                    if let Err(refusal) = self.relink(response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::FundFirst;
                }
                Stage::FundFirst => {
                    if let Err(refusal) = self.settle_funding(ShapeOwner::First, response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = if second_coins > 0 {
                        Stage::FundSecond
                    } else {
                        Stage::Submit(0)
                    };
                }
                Stage::FundSecond => {
                    if let Err(refusal) = self.settle_funding(ShapeOwner::Second, response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = Stage::Submit(0);
                }
                Stage::Submit(position) => {
                    if let Err(refusal) = self.settle_case(position, response) {
                        return Err(self.refuse(refusal));
                    }
                    self.stage = if position + 1 < self.cases.len() {
                        Stage::Submit(position + 1)
                    } else {
                        Stage::Done
                    };
                }
                Stage::Done => {}
            }
        }

        Ok(())
    }

    /// The step the ceremony asks for next.
    fn emit_step(&mut self) -> Result<Option<OperationStep>, PlanRefused> {
        let (first_coins, second_coins) = self.shape.funded_coins();
        match self.stage {
            Stage::Issue => match self.funding_step(
                "issue-protocol-asset",
                true,
                ShapeOwner::First,
                first_coins,
            ) {
                Ok(step) => Ok(Some(step)),
                Err(refusal) => Err(self.refuse(refusal)),
            },
            Stage::FundFirst => match self.funding_step(
                "fund-first-owner-constructor",
                false,
                ShapeOwner::First,
                first_coins,
            ) {
                Ok(step) => Ok(Some(step)),
                Err(refusal) => Err(self.refuse(refusal)),
            },
            Stage::FundSecond => match self.funding_step(
                "fund-second-owner-constructor",
                false,
                ShapeOwner::Second,
                second_coins,
            ) {
                Ok(step) => Ok(Some(step)),
                Err(refusal) => Err(self.refuse(refusal)),
            },
            Stage::Submit(position) => {
                if self.record.coins.len() != self.shape.input_count() {
                    return Err(self.refuse(ExplicitShapeRefusal::FundedWidthDisagrees));
                }
                let Some(case) = self.cases.get(position).copied() else {
                    return Err(self.refuse(ExplicitShapeRefusal::CandidateNotConstructible));
                };
                let (mutation, name) = match case {
                    CaseKind::Control => (None, self.shape.case_name()),
                    CaseKind::Mutated(change) => (Some(change), change.case_name()),
                };
                // The comparison a mutant is measured against is the
                // control's bytes, and with the mutants offered first
                // the control has not been built yet. So it is built
                // here -- locally, submitted to nothing -- rather than
                // the measurement being skipped or deferred.
                if mutation.is_some() && self.record.control_bytes.is_empty() {
                    match self.candidate_bytes(None) {
                        Ok((bytes, _)) => self.record.control_bytes = bytes,
                        Err(refusal) => return Err(self.refuse(refusal)),
                    }
                }
                match self.candidate_bytes(mutation) {
                    Ok((bytes, offered)) => {
                        if matches!(case, CaseKind::Control) {
                            self.record.submitted_bytes = bytes.len();
                        } else {
                            self.record.offered_signature_bytes = offered;
                        }
                        self.pending = Some(bytes.clone());
                        Ok(Some(OperationStep::new(
                            name,
                            OperationSubject::Submission(Box::new(TargetSubmissionSubject {
                                transaction_bytes: bytes,
                            })),
                        )))
                    }
                    Err(refusal) => Err(self.refuse(refusal)),
                }
            }
            Stage::Done => Ok(None),
        }
    }
}

/// One owner's explicit destination program.
fn explicit_destination_program(
    abi: &CandidateLiveTransferAbi,
    owner: ShapeOwner,
) -> Result<Vec<u8>, VectorError> {
    Ok(abi
        .destinations()
        .get(
            &linker::OwnerParameter::new(published_owner(owner.scalar())?),
            LiveTransferRepresentationPlan::Explicit,
        )
        .ok_or(VectorError::LiveSubstrateUnavailable)?
        .instance()
        .program()
        .to_vec())
}

/// One funded coin, as reported and as expected.
fn observed_coin(
    funded: &FundedOutput,
    expected_asset: AssetId,
    expected_program: &[u8],
    owner: ShapeOwner,
    expected_amount: u64,
) -> Result<ObservedShapeCoin, ExplicitShapeRefusal> {
    let outpoint =
        outpoint_of(&funded.outpoint).ok_or(ExplicitShapeRefusal::MalformedFundedOutput)?;
    let asset = asset_of(&funded.asset).ok_or(ExplicitShapeRefusal::MalformedFundedOutput)?;
    let program = decode_hex(&funded.script).ok_or(ExplicitShapeRefusal::MalformedFundedOutput)?;
    let matches_expectation = asset == expected_asset
        && funded.amount_satoshis == expected_amount
        && program == expected_program;
    Ok(ObservedShapeCoin {
        outpoint,
        asset: AssetField::Explicit(asset),
        value: ValueField::Explicit(funded.amount_satoshis),
        program,
        owner,
        matches_expectation,
    })
}

/// How many bytes of each submission lie between their common prefix
/// and their common suffix.
///
/// The pair is what the attributability measurement is actually made
/// of, and saying so precisely matters because the obvious phrasing
/// overclaims. ANY two distinct byte strings differ in exactly one
/// contiguous region, since the region between the maximal common
/// prefix and the maximal common suffix is contiguous by construction.
/// So "they differ in one run" is not a property worth checking — it is
/// true of every pair that differs at all.
///
/// What has content is the region's SIZE. The control and a mutant are
/// the same finalized candidate with one witness item replaced, so the
/// difference must fit inside that item and its length prefix. A
/// difference wider than that would mean something other than the
/// signature position moved, and the refusal could no longer be
/// attributed to what the witness offered.
fn difference_runs(control: &[u8], mutant: &[u8]) -> (usize, usize) {
    let prefix = control
        .iter()
        .zip(mutant.iter())
        .take_while(|(left, right)| left == right)
        .count();
    let remaining = control.len().min(mutant.len()) - prefix;
    let suffix = control
        .iter()
        .rev()
        .zip(mutant.iter().rev())
        .take_while(|(left, right)| left == right)
        .count()
        .min(remaining);
    (
        control.len() - prefix - suffix,
        mutant.len() - prefix - suffix,
    )
}

/// The widest a witness-content difference may be.
///
/// One selected signature plus the single byte that states its length.
/// A difference within this bound sits inside the item the mutation
/// replaced; one beyond it does not.
const WITNESS_ITEM_BOUND: usize = OWNER_SIGNATURE_BYTES + 1;

/// Hex, for the transcript.
fn hex(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(text, "{byte:02x}");
    }
    text
}

/// The read-back and per-input verification lines, and the negative
/// cases' lines.
///
/// Split out of the renderer because the renderer had grown past what
/// one function is allowed to be, and this is the seam: everything here
/// is about what happened AFTER a submission was answered.
fn render_reverification(out: &mut String, record: &ExplicitShapeRecord) {
    if let Some(check) = &record.reverification {
        let _ = writeln!(out, "readback_txid {}", check.accepted_txid);
        let _ = writeln!(out, "readback_witness_txid {}", check.witness_txid);
        let _ = writeln!(out, "readback_block_height {}", check.block_height);
        let _ = writeln!(
            out,
            "readback_matches_submission {}",
            check.readback_matches_submission
        );
        for input in &check.inputs {
            let _ = writeln!(
                out,
                "input {} owner {} signature_bytes {} verified {} message {}",
                input.input_index,
                input.owner.name(),
                input.signature_bytes,
                input.verified,
                hex(&input.recomputed_message)
            );
        }
        let _ = writeln!(out, "every_input_verified {}", check.every_input_verified());
    }
    for negative in &record.negatives {
        let _ = writeln!(
            out,
            "negative {} row {} submitted_bytes {} offered_signature_bytes {} layer {:?} \
             control_difference_bytes {} mutant_difference_bytes {} \
             differs_from_control_in_one_item {} accepted_txid {} detail {}",
            negative.mutation.case_name(),
            negative.mutation.row_name(),
            negative.submitted_bytes,
            negative.offered_signature_bytes,
            negative.layer,
            negative.control_difference_bytes,
            negative.mutant_difference_bytes,
            negative.differs_from_control_in_one_item,
            negative.accepted_txid.as_deref().unwrap_or("none"),
            negative.detail.as_deref().unwrap_or("none"),
        );
    }
}

/// One shape run's transcript.
///
/// Lines rather than a structure, on the pattern every live lane here
/// sets: the artifact is read by people and diffed by machines. Every
/// line is a fact the run observed or a value it computed, and no line
/// is a verdict about whether the run went well.
#[must_use]
pub fn render_explicit_shape(record: &ExplicitShapeRecord) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "explicit_shape {}", record.shape.case_name());
    let _ = writeln!(
        out,
        "row_name {}",
        record.shape.row_name().unwrap_or("none")
    );
    let _ = writeln!(
        out,
        "issued_asset {}",
        record.issued_asset.as_deref().unwrap_or("none")
    );
    let _ = writeln!(out, "relinked {}", record.relinked);
    let _ = writeln!(out, "funded_coins {}", record.coins.len());
    for coin in &record.coins {
        let _ = writeln!(
            out,
            "coin owner {} matches_expectation {}",
            coin.owner.name(),
            coin.matches_expectation
        );
    }
    let _ = writeln!(out, "input_count {}", record.input_count());
    for owner in &record.input_owners {
        let _ = writeln!(out, "input_owner {}", owner.name());
    }
    let _ = writeln!(out, "output_count {}", record.output_count());
    for (owner, amount) in record
        .destination_owners
        .iter()
        .zip(record.destination_amounts.iter())
    {
        let _ = writeln!(out, "destination owner {} value {amount}", owner.name());
    }
    let _ = writeln!(
        out,
        "offered_order_differs {}",
        record.offered_order_differs()
    );
    let _ = writeln!(out, "submitted_bytes {}", record.submitted_bytes);
    match record.observed_weight {
        Some(weight) => {
            let _ = writeln!(out, "observed_weight {weight}");
        }
        None => {
            let _ = writeln!(out, "observed_weight none");
        }
    }
    let _ = writeln!(
        out,
        "observed_layer {}",
        record
            .observed_layer
            .map_or_else(|| "none".to_owned(), |layer| format!("{layer:?}"))
    );
    let _ = writeln!(
        out,
        "observed_detail {}",
        record.observed_detail.as_deref().unwrap_or("none")
    );
    let _ = writeln!(
        out,
        "accepted_txid {}",
        record.accepted_txid.as_deref().unwrap_or("none")
    );
    render_reverification(&mut out, record);
    let _ = writeln!(
        out,
        "construction_refusal {}",
        record
            .refusal
            .map_or_else(|| "none".to_owned(), |refusal| format!("{refusal:?}"))
    );
    let _ = writeln!(
        out,
        "evidences_no_negative_case {}",
        record.negatives.is_empty()
    );
    let _ = writeln!(out, "builds_no_sponsor_region true");
    for claim in ExplicitShapeRecord::non_claims() {
        let _ = writeln!(out, "does_not_establish {claim}");
    }
    out
}

/// What one execution of every shape against a real node produced.
///
/// Hand-recorded from the transcripts that run wrote, on the pattern the
/// private lane's own register sets and for the same reason: a matrix
/// row cites a value a chain produced, and a value a chain produced has
/// to be written down somewhere a reader can reach it.
///
/// The target was the pinned Elements node the live lane binds itself
/// to, on a disposable development chain each run created and
/// destroyed. Every run issued its own asset and every one issued the
/// same identity, which is what a deterministic disposable chain does
/// and not a sign that one run was reported thirteen times.
///
/// # Thirteen runs and TEN identities, which is a finding rather than a
/// defect
///
/// Three pairs of runs produced the same identity, and in each case for
/// the same reason: the two rows are two CLASSES of one transaction
/// rather than two transactions. §15.1 names sixteen classes and a
/// single transfer is an instance of several of them at once — a
/// one-input one-output sponsorless transfer is simultaneously the
/// `one-input-to-one-output` class and the `sponsorless` class, and
/// building a second, gratuitously different transfer so that each row
/// could cite its own hex string would be dressing one fact up as two.
///
/// So the identity is shared and the sharing is stated:
///
/// - `one-input-to-one-output` and `sponsorless` share
///   [`run_of_record::ONE_TO_ONE_ACCEPTED_TXID`]. The accepted bytes are both.
/// - `one-input-split-into-two` and `several-destination-owners` share
///   [`run_of_record::SPLIT_ACCEPTED_TXID`]. A split into two destinations belonging to
///   two distinct owners is both.
/// - `several-inputs-merged-into-one` and
///   `canonical-input-normalization` share [`run_of_record::MERGE_ACCEPTED_TXID`], and
///   this pair is the strongest of the three rather than the weakest.
///   The normalization run offered its two receipts in the REVERSE of
///   their canonical order and the merge run offered them in it; the two
///   built byte-identical transactions and the node computed one
///   identity for them. The collision IS the normalization, observed
///   rather than asserted, and a run that had produced a second identity
///   would have been evidence that the request does not normalize.
///
/// What the rule this register is held to actually forbids is citing an
/// acceptance of a DIFFERENT shape. None of these does: in each pair the
/// accepted bytes are an instance of both rows' classes.
pub mod run_of_record {
    /// The disposable asset every run issued.
    pub const ISSUED_ASSET: &str =
        "d74fc8d4d85f8251aa653f5404ea646f56d34b8f506a98279ce2926d05ca93fb";

    /// One receipt consumed, one created, sponsorless.
    ///
    /// Cited by `one-input-to-one-output` and by `sponsorless`. The
    /// smallest submission this lane has made, 593 bytes, for the
    /// structural reason that an explicit transfer carries no range
    /// proof at all.
    pub const ONE_TO_ONE_ACCEPTED_TXID: &str =
        "872a2294da5ea650a7a74ffd8a5932210930ab70d6a08a991eb3ea471ee29abb";

    crate::recorded_acceptance::mint_recorded_acceptance!(
        one_to_one_accepted,
        ONE_TO_ONE_ACCEPTED_TXID
    );

    /// How many bytes the one-to-one handed the node.
    pub const ONE_TO_ONE_SUBMITTED_BYTES: usize = 593;

    /// One receipt consumed, TWO created for two distinct owners.
    ///
    /// Cited by `one-input-split-into-two` and by
    /// `several-destination-owners`.
    pub const SPLIT_ACCEPTED_TXID: &str =
        "0fcf267058e83e06e87a950bbeec920a15e3641ef7f76c55df0a8fff544e64c1";

    crate::recorded_acceptance::mint_recorded_acceptance!(split_accepted, SPLIT_ACCEPTED_TXID);

    /// How many bytes the split handed the node.
    pub const SPLIT_SUBMITTED_BYTES: usize = 751;

    /// TWO receipts consumed, ONE created.
    ///
    /// Cited by `several-inputs-merged-into-one` and by
    /// `canonical-input-normalization`, the second because the
    /// normalization run offered the same two receipts in the reverse
    /// order and the node computed this same identity for what it built.
    pub const MERGE_ACCEPTED_TXID: &str =
        "7a0ac33f0268e48ebeb1316dbc262c8d40569ba5c96274d1b8262f394c6f7c39";

    crate::recorded_acceptance::mint_recorded_acceptance!(merge_accepted, MERGE_ACCEPTED_TXID);

    /// How many bytes the merge handed the node.
    pub const MERGE_SUBMITTED_BYTES: usize = 1_006;

    /// TWO receipts consumed, TWO created.
    pub const SEVERAL_TO_SEVERAL_ACCEPTED_TXID: &str =
        "40cb6c4ee284ed38555a4840198c8130d1e2c3246b57b9d8b93842c3c6730029";

    crate::recorded_acceptance::mint_recorded_acceptance!(
        several_to_several_accepted,
        SEVERAL_TO_SEVERAL_ACCEPTED_TXID
    );

    /// TWO receipts under ONE owner, three outputs created.
    ///
    /// The repetition is the subject: one published owner authorized two
    /// separate inputs, each at its own position and each over its own
    /// recomputed message, and both signatures verify out of the node's
    /// own copy.
    pub const REPEATED_OWNER_ACCEPTED_TXID: &str =
        "3f833570061c28f1b6cae0cd2abda65ed2c573bf62f418e847836a7999382114";

    crate::recorded_acceptance::mint_recorded_acceptance!(
        repeated_owner_accepted,
        REPEATED_OWNER_ACCEPTED_TXID
    );

    /// TWO receipts under two DISTINCT published owners.
    ///
    /// Its destinations are the several-to-several run's exactly, and
    /// the identities differ anyway — because the SPENT programs differ,
    /// one coin having been paid to each owner's explicit constructor.
    /// That the two runs diverge on their input side alone is what makes
    /// this run about its input owners.
    pub const SEVERAL_DISTINCT_OWNERS_ACCEPTED_TXID: &str =
        "f87e1ef327f69fe1f6de5f763cc73d14edbe9425372f7a451d79d8e30e42b660";

    crate::recorded_acceptance::mint_recorded_acceptance!(
        several_distinct_owners_accepted,
        SEVERAL_DISTINCT_OWNERS_ACCEPTED_TXID
    );

    /// TWO destinations, both created for ONE owner.
    pub const ONE_DESTINATION_OWNER_ACCEPTED_TXID: &str =
        "c9bd2bd7ea47f2e7df3d95751d008e2db2448d6b9611425114b06e09d7a2a0a8";

    crate::recorded_acceptance::mint_recorded_acceptance!(
        one_destination_owner_accepted,
        ONE_DESTINATION_OWNER_ACCEPTED_TXID
    );

    /// Destinations at the boundary values the request type admits.
    ///
    /// One and the remainder. The smallest is ONE because
    /// [`super::ProtocolValue`] refuses zero by name, so the value is
    /// the boundary the type states rather than a small number somebody
    /// picked — and the node accepted it, which is the fact worth
    /// having: nothing on this chain turned a one-unit output away.
    pub const BOUNDARY_VALUES_ACCEPTED_TXID: &str =
        "a53626927129a23c37681973eede8d23996743a1fb04adc71853e2bfe0d608be";

    crate::recorded_acceptance::mint_recorded_acceptance!(
        boundary_values_accepted,
        BOUNDARY_VALUES_ACCEPTED_TXID
    );

    /// THREE receipts consumed, the candidate's stated input bound.
    ///
    /// The widest submission of this table at 1421 bytes.
    pub const MAXIMUM_INPUTS_ACCEPTED_TXID: &str =
        "fce6e069897f841297803e36da5d51f3e7b4e15422ff0083a1cac4735147c112";

    crate::recorded_acceptance::mint_recorded_acceptance!(
        maximum_inputs_accepted,
        MAXIMUM_INPUTS_ACCEPTED_TXID
    );

    /// THREE destinations created, the candidate's stated output bound.
    pub const MAXIMUM_OUTPUTS_ACCEPTED_TXID: &str =
        "6a5617cc547f0fe22cefa261ed2fb885a82a9e7293aefad76be5c35f0b531613";

    crate::recorded_acceptance::mint_recorded_acceptance!(
        maximum_outputs_accepted,
        MAXIMUM_OUTPUTS_ACCEPTED_TXID
    );

    // --- The fourteenth shape: the sponsorless self-paid fee ----------
    //
    // The last cell of the owner fee matrix, and the only row here that
    // is not a §15.1 row at all. Its figures are kept in this register
    // rather than in the §15 tables because §15.1's explicit positive
    // table is complete at sixteen rows and none of the sixteen is a
    // self-paying transfer.

    /// The identity the target computed for the self-paying run.
    ///
    /// An OPTION carrying a value: the node answered, and this is what
    /// it answered. The confidential lane's fee-bearing identity is
    /// written the same way for the same reason -- a record of what a
    /// node answered holds a digest only once one has, and a placeholder
    /// shaped like an identity would be indistinguishable from an
    /// observation.
    ///
    /// It collides with no other row's identity, which the collision
    /// census checks rather than assumes.
    pub const SELF_PAID_FEE_ACCEPTED_IDENTITY: Option<&str> =
        Some("72fad04b346a8ea93c97cd415d24434529d8a5d9ae148a13da3694047e117328");

    /// The witness identity of the same accepted transaction.
    ///
    /// Recorded beside the identity because they DIFFER, and the
    /// difference is the ordinary one: the transaction carries a
    /// witness, so the two hashes are taken over different bytes.
    pub const SELF_PAID_FEE_WITNESS_IDENTITY: &str =
        "fafc82e7396f27e6379572cc958c710412b1ae0e0d69912a04212aa79c7d2822";

    /// The disposable asset the self-paying run issued.
    pub const SELF_PAID_FEE_ISSUED_ASSET: &str =
        "d74fc8d4d85f8251aa653f5404ea646f56d34b8f506a98279ce2926d05ca93fb";

    /// How many bytes the self-paying candidate handed the node.
    pub const SELF_PAID_FEE_SUBMITTED_BYTES: usize = 782;

    /// The weight the TARGET reported, read off `decoderawtransaction`.
    ///
    /// The node's own figure rather than one computed here, so §20.5's
    /// comparison is against an observation.
    pub const SELF_PAID_FEE_TARGET_WEIGHT: u64 = 1_304;

    /// What the one consumed receipt held.
    pub const SELF_PAID_FEE_CONSUMED: u64 = super::RECEIPT_AMOUNT;

    /// The fee the self-paying run paid, in the PROTOCOL asset.
    ///
    /// The protocol asset because the fee is funded out of the consumed
    /// receipts and Elements balances per asset: a reserve-asset fee
    /// beside no reserve-asset input dies at the tally.
    pub const SELF_PAID_FEE_AMOUNT: u64 = super::SELF_PAID_FEE_AMOUNT;

    /// What reached the one destination, the fee having been taken.
    pub const SELF_PAID_FEE_DESTINATION: u64 = 4_750;

    /// Whether the self-paying candidate crossed RELAY and then a block.
    ///
    /// TRUE, and this is the figure the wave existed to obtain. The
    /// adapter offers every submission to `testmempoolaccept` first and
    /// records an acceptance only when the mempool ALLOWED it and a
    /// block then included it, so an accepted layer here is a relay
    /// verdict and a consensus verdict together.
    ///
    /// It matters because this is the FIRST sponsorless form to face
    /// relay on its own. Its predecessors paid no fee and travelled as
    /// package children, which is why the ABI builds a sponsorless form
    /// at the topology-restricted version; a form that pays its own fee
    /// needs no package parent, and the open question was whether that
    /// version would still relay standalone. It did.
    pub const SELF_PAID_FEE_CROSSED_RELAY_AND_BLOCK: bool = true;

    /// Seconds of wall time the self-paying run took.
    pub const SELF_PAID_FEE_WALL_SECONDS: f64 = 6.0;
}

impl ExplicitShape {
    /// The identity the target computed for this shape's accepted
    /// transaction.
    ///
    /// An OPTION, and the absent case is a shape whose acceptance this
    /// register does not yet hold rather than a shape that failed. The
    /// confidential lane's fee-bearing identity is written the same way
    /// and for the same reason: an identity is recorded when a node has
    /// answered, and a placeholder that looked like one would be a claim
    /// nothing observed.
    ///
    /// Three identities are each shared by two shapes, and
    /// [`run_of_record`] states which and why.
    #[must_use]
    pub const fn observed_identity(self) -> Option<&'static str> {
        match self {
            Self::OneToOne | Self::Sponsorless => Some(run_of_record::ONE_TO_ONE_ACCEPTED_TXID),
            Self::SplitIntoTwo | Self::SeveralDestinationOwners => {
                Some(run_of_record::SPLIT_ACCEPTED_TXID)
            }
            Self::MergedIntoOne | Self::CanonicalInputNormalization => {
                Some(run_of_record::MERGE_ACCEPTED_TXID)
            }
            Self::SeveralToSeveral => Some(run_of_record::SEVERAL_TO_SEVERAL_ACCEPTED_TXID),
            Self::RepeatedOwner => Some(run_of_record::REPEATED_OWNER_ACCEPTED_TXID),
            Self::SeveralDistinctOwners => {
                Some(run_of_record::SEVERAL_DISTINCT_OWNERS_ACCEPTED_TXID)
            }
            Self::OneDestinationOwner => Some(run_of_record::ONE_DESTINATION_OWNER_ACCEPTED_TXID),
            Self::SemanticBoundaryValues => Some(run_of_record::BOUNDARY_VALUES_ACCEPTED_TXID),
            Self::MaximumInputs => Some(run_of_record::MAXIMUM_INPUTS_ACCEPTED_TXID),
            Self::MaximumOutputs => Some(run_of_record::MAXIMUM_OUTPUTS_ACCEPTED_TXID),
            Self::SelfPaidFee => run_of_record::SELF_PAID_FEE_ACCEPTED_IDENTITY,
            // The arc's own run, cited from the arc's module rather than
            // copied here: the identity belongs to the ledger that
            // observed it, and this lane's own run of record never
            // submitted this shape.
            Self::PairedOneToOne => {
                crate::live_pair_arc::run_of_record::EXPLICIT_MEMBER_ACCEPTED_IDENTITY
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use transaction::live_construct::ExplicitDestinationRole::Fee;

    use super::{ExplicitShape, LiveShapeVocabulary, run_of_record};

    #[test]
    fn every_shape_names_a_distinct_row_of_the_explicit_table() {
        // The shape table and the matrix table are two authorities, and
        // a shape that answered a row twice would be one of them
        // disagreeing with itself.
        let rows: BTreeSet<&str> = ExplicitShape::ALL
            .iter()
            .filter_map(|shape| shape.row_name())
            .collect();
        assert_eq!(rows.len(), 13);
        assert_eq!(ExplicitShape::ALL.len(), 15);

        // And exactly TWO shapes name no row. The count is asserted
        // rather than left implied, because the interesting failure is a
        // third shape quietly acquiring the absent case: §15.1's table
        // is complete, so a shape naming no row is a claim about the
        // matrix and every one of them has to be deliberate. The two are
        // the self-paying shape, which answers the owner fee matrix, and
        // the arc's paired member, which answers §16.1.
        let rowless = ExplicitShape::ALL
            .iter()
            .filter(|shape| shape.row_name().is_none())
            .count();
        assert_eq!(rowless, 2);
    }

    #[test]
    fn every_recorded_identity_is_a_target_identity() {
        for shape in ExplicitShape::ALL {
            let Some(identity) = shape.observed_identity() else {
                continue;
            };
            assert_eq!(
                identity.len(),
                64,
                "{shape:?} cites something that is not a target identity",
            );
            assert!(identity.chars().all(|digit| digit.is_ascii_hexdigit()));
        }
        assert_eq!(run_of_record::ISSUED_ASSET.len(), 64);
    }

    #[test]
    fn exactly_three_identities_are_shared_and_the_sharing_is_the_documented_one() {
        // The collision census, spelled rather than derived. A pair that
        // started sharing an identity because a shape stopped differing
        // would fail here, which is the whole point: two rows may share
        // an acceptance only where the accepted bytes are an instance of
        // both classes, and that is a judgement rather than an accident.
        let mut by_identity: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for shape in ExplicitShape::ALL {
            let (Some(identity), Some(row)) = (shape.observed_identity(), shape.row_name()) else {
                continue;
            };
            by_identity.entry(identity).or_default().insert(row);
        }
        assert_eq!(by_identity.len(), 10, "thirteen runs, ten identities");

        // The fourteenth shape is deliberately OUTSIDE this census, and
        // its exclusion is the census's own subject: this is a count of
        // which §15.1 ROWS share an acceptance, and a shape naming no row
        // has none to share. What is asserted about it instead is that
        // its identity collides with nothing, which is the property the
        // census exists to protect -- one acceptance may never be cited
        // for work it did not do.
        if let Some(identity) = ExplicitShape::SelfPaidFee.observed_identity() {
            assert!(
                !by_identity.contains_key(identity),
                "the self-paying run cites an identity another shape already claims",
            );
        }

        let shared: BTreeSet<BTreeSet<&str>> = by_identity
            .values()
            .filter(|rows| rows.len() > 1)
            .cloned()
            .collect();
        assert_eq!(
            shared,
            BTreeSet::from([
                BTreeSet::from(["one-input-to-one-output", "sponsorless"]),
                BTreeSet::from(["one-input-split-into-two", "several-destination-owners"]),
                BTreeSet::from([
                    "canonical-input-normalization",
                    "several-inputs-merged-into-one"
                ]),
            ]),
        );
    }

    #[test]
    fn the_self_paying_run_conserves_what_it_consumed() {
        // The fee is a TERM of this equality rather than a residue: the
        // consumed receipt funds the destination AND the fee, which is
        // exactly the relation the explicit conservation leaf checks in
        // the covenant that ran. Recomputed here from the recorded
        // figures so a register edited on one side fails.
        assert_eq!(
            run_of_record::SELF_PAID_FEE_DESTINATION + run_of_record::SELF_PAID_FEE_AMOUNT,
            run_of_record::SELF_PAID_FEE_CONSUMED,
        );
        assert_eq!(
            run_of_record::SELF_PAID_FEE_AMOUNT,
            ExplicitShape::SelfPaidFee
                .self_paid_fee()
                .expect("the self-paying shape states a fee"),
        );
    }

    #[test]
    fn the_self_paying_shape_declares_its_fee_last_and_only_once() {
        // The declaration the ceremony hands construction, checked here
        // rather than trusted: the fee sits immediately after the
        // destinations, which is where a SPONSORLESS shape's fee
        // position is -- no change role can precede it without a sponsor
        // region.
        let roles = ExplicitShape::SelfPaidFee.declared_roles();
        assert_eq!(roles.len(), ExplicitShape::SelfPaidFee.output_count());
        assert_eq!(
            roles.iter().filter(|role| **role == Fee).count(),
            1,
            "the target admits one fee position",
        );
        assert_eq!(roles.last(), Some(&Fee));

        // And every OTHER shape declares nothing at all, which is what
        // leaves their runs building the bytes they built before this
        // shape existed.
        for shape in ExplicitShape::ALL {
            if *shape == ExplicitShape::SelfPaidFee {
                continue;
            }
            assert!(
                shape.declared_roles().is_empty(),
                "{shape:?} declares a role it has no fee to declare",
            );
        }
    }

    #[test]
    fn only_the_self_paying_shape_leaves_the_demonstration_vocabulary() {
        // The separation that keeps the demonstration taptree and every
        // digest recorded against it untouched by the fee axis.
        for shape in ExplicitShape::ALL {
            let expected = if *shape == ExplicitShape::SelfPaidFee {
                LiveShapeVocabulary::FeeBearing
            } else {
                LiveShapeVocabulary::Demonstration
            };
            assert_eq!(shape.vocabulary(), expected, "{shape:?}");
        }
    }

    #[test]
    fn the_shape_cardinalities_are_the_ones_the_runs_reported() {
        // Read back off the shape rather than off the transcript, so a
        // shape edited after its run fails here instead of quietly
        // citing an identity for something else.
        assert_eq!(ExplicitShape::OneToOne.input_count(), 1);
        assert_eq!(ExplicitShape::OneToOne.output_count(), 1);
        assert_eq!(ExplicitShape::MergedIntoOne.input_count(), 2);
        assert_eq!(ExplicitShape::MergedIntoOne.output_count(), 1);
        assert_eq!(ExplicitShape::MaximumInputs.input_count(), 3);
        assert_eq!(ExplicitShape::MaximumOutputs.output_count(), 3);
        assert_eq!(ExplicitShape::SeveralDistinctOwners.funded_coins(), (1, 1));
        assert!(ExplicitShape::CanonicalInputNormalization.offers_reversed_receipts());
    }
}

/// What the witness-content negative run observed.
///
/// §15.3's two witness-content rows, answered by ONE run that submitted
/// three candidates to one node on one chain: two mutants first, then
/// the unmutated control.
///
/// # The order is part of the evidence, and the first attempt got it
/// wrong
///
/// This register exists in the shape it does because a run falsified the
/// obvious ordering. A witness-content mutation changes the witness, and
/// the witness is not part of a transaction's identity, so a mutant and
/// its control have the SAME identity. Submitting the control first and
/// mining it made both mutants come back refused `txn-already-known` at
/// a layer before script evaluation — a true refusal about an identity
/// already on the chain, attributable to the submission order and to
/// nothing the witness offered.
///
/// Read as an answer, that would have been the exact error the
/// attributability rule exists to prevent: a refusal counted for a row
/// whose class had nothing to do with it. The mutants now go first, and
/// the target's answers changed with the order — which is itself the
/// demonstration that the earlier answers were about the order.
pub mod witness_negatives_run_of_record {
    use target_elements_conformance::protocol::ObservedOutcomeLayer;

    /// The identity the target computed for the accepted control.
    ///
    /// The same one-input one-output candidate the positive table cites,
    /// accepted again here after both mutants had been refused — which
    /// is what makes each refusal attributable rather than merely
    /// recorded.
    pub const CONTROL_ACCEPTED_TXID: &str =
        "872a2294da5ea650a7a74ffd8a5932210930ab70d6a08a991eb3ea471ee29abb";

    crate::recorded_acceptance::mint_recorded_acceptance!(control_accepted, CONTROL_ACCEPTED_TXID);

    /// What the target said to a signature position offering nothing.
    ///
    /// Its own words, verbatim. The offering was empty, so the check
    /// that consumed it failed rather than the signature being judged
    /// invalid — which is why this row and the malformed one are
    /// distinguishable at all.
    pub const EMPTY_SIGNATURE_REFUSAL: &str =
        "mandatory-script-verify-flag-failed (Script failed an OP_CHECKSIGVERIFY operation)";

    /// How many bytes the empty-signature candidate handed the node.
    ///
    /// Sixty-four fewer than the control, which is the signature that is
    /// no longer there.
    pub const EMPTY_SIGNATURE_SUBMITTED_BYTES: usize = 529;

    /// What the target said to a well-sized offering that is not a
    /// signature.
    ///
    /// A DIFFERENT verdict from the empty case, and the difference is
    /// what makes each row its own: the width was kept, so the check
    /// consumed an item and judged it, and the target named the judgement
    /// rather than the arity.
    pub const MALFORMED_SIGNATURE_REFUSAL: &str =
        "mandatory-script-verify-flag-failed (Invalid Schnorr signature)";

    /// How many bytes the malformed-signature candidate handed the node.
    ///
    /// Exactly the control's count, because only the CONTENT of a
    /// well-sized item moved.
    pub const MALFORMED_SIGNATURE_SUBMITTED_BYTES: usize = 593;

    /// What the earlier, control-first ordering produced.
    ///
    /// Kept rather than deleted, because a register that recorded only
    /// the ordering that worked would lose the reason the ordering
    /// matters, and a reader reversing it would rediscover this the
    /// expensive way.
    pub const REFUSAL_UNDER_CONTROL_FIRST_ORDER: &str = "txn-already-known";

    /// The layer BOTH witness mutants were refused at, TYPED.
    ///
    /// One constant for the two rows because the layer is the same fact
    /// for both: each offering reached the leaf and was judged there, and
    /// what separates the rows is the target's WORDS — the arity check
    /// against the signature judgement — not where it spoke. Typed so the
    /// classifier can check the layer instead of assuming it.
    pub const WITNESS_REFUSAL_OBSERVED_LAYER: ObservedOutcomeLayer =
        ObservedOutcomeLayer::ScriptPathRejection;
}
