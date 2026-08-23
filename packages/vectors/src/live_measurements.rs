//! §18.2's complete transactions and §18.3's seventeen dimensions.
//!
//! §18.2 names fifteen cases and asks for a *complete transaction* per
//! case; §18.3 names seventeen dimensions and asks for them to be
//! recorded separately. This module builds the transactions through the
//! same construction pipeline a deployment would use — request,
//! finalization, owner authorization, completion — and reads every figure
//! off the finalized bytes or off the linked artifacts those bytes point
//! at. Nothing here is an estimate, and nothing is transcribed from an
//! earlier wave's notes.
//!
//! # Fifteen cases, seventeen transactions
//!
//! Two of §18.2's bullets name two things each — "private split and merge
//! where claimed" and "sponsor change present and absent" — so a case is
//! one bullet and carries one or two members. The alternative was to
//! flatten the bullets into seventeen case names, which would have made
//! the list stop matching the section it transcribes.
//!
//! # Why a dimension is a standing rather than a number
//!
//! Because several of §18.3's seventeen cannot be numbers here, and
//! §18.4 forbids reading an absent observation as zero or as agreement.
//! Three kinds of not-a-number arise, and they are different findings:
//!
//! - a dimension the case's own shape has no member for at all — a
//!   one-to-one transfer commits no member leaf, so its member-byte
//!   figure is *absent* rather than zero;
//! - a dimension whose subject cannot exist in this workspace — no
//!   confidential proof is serialized (§12.8's model commits values and
//!   the encoder writes an empty range proof for every output), and no
//!   owner signature can be produced at all (§1.7);
//! - a dimension whose subject is a target's own verdict, which a
//!   transaction nobody could witness has not earned (§1.11).
//!
//! [`DimensionStanding`] has one arm per kind, and the arithmetic that
//! reads the table has to match on all of them.
//!
//! # The signature position is a slot, and the slot is what is measured
//!
//! Nothing in this workspace computes the taproot sighash an owner would
//! sign, so every receipt input's witness carries opaque bytes of a
//! signature's width. That is stated at every use rather than hidden: the
//! weight and virtual size below are the exact figures for a transaction
//! whose signature positions are *filled*, which is the figure a resource
//! study wants, and [`DimensionStanding::WitnessSlotOnly`] is what keeps
//! it from reading as a measurement of a signature that exists.

use std::collections::{BTreeMap, BTreeSet};

use linker::{CandidateLinkedLiveTransferBundle, OwnerParameter};
use tapscript::upstream::LiveTransferRepresentationPlan;
use tapscript::{
    AbstractLimits, LiveTransferLeafRole, LiveTransferShape, SponsorChangePresence, StackItem,
    TapscriptInstruction, TapscriptProgram, live_program_precondition, resource_projection,
    validate_program,
};
use target_elements::{ResourceDimension, ReviewedElementsTapscriptDefinition};
use transaction::bytes::{AssetField, Outpoint, Txid, ValueField};
use transaction::live_abi::CandidateLiveTransferAbi;
use transaction::live_construct::{
    CandidateLiveTransferTransaction, complete_live_transfer, finalize_live_transfer,
};
use transaction::live_finalize::FinalizedLiveTransfer;
use transaction::live_private::PrivateValueCapability;
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
use crate::live_plan::{
    FIRST_SCALAR, SECOND_SCALAR, demonstration_live_abi, demonstration_live_bundle,
    published_owner, reviewed_target,
};

/// The amount one measured predecessor carries.
///
/// A constant, and not a clock, a counter, or an environment value: two
/// runs of this study measure the same bytes, which is what makes a
/// figure here comparable with one a target reported for those bytes.
pub const MEASURED_RECEIPT_UNIT: u64 = 1_000;

/// The fee a measured sponsor declares.
pub const MEASURED_SPONSOR_FEE: u64 = 250;

/// The change a measured sponsor asks back.
pub const MEASURED_SPONSOR_CHANGE: u64 = 500;

/// The published randomness a measured private destination consumes.
///
/// Public test material in the sense `(´[ADR015-rule:security:test-material]´)`
/// fixes: a value this study published, reproducible because it is an
/// input rather than something generated here.
pub const MEASURED_DESTINATION_RANDOMNESS: [u8; 32] = [0x3d; 32];

/// The published randomness a measured private predecessor was committed
/// under.
pub const MEASURED_PREDECESSOR_RANDOMNESS: [u8; 32] = [0x4d; 32];

/// One of §18.3's seventeen dimensions.
///
/// Transcribed in the section's own order, so a reader can check the list
/// against §18.3 line by line. A dimension dropped from here would be a
/// dimension this study stopped recording.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LiveResourceRecord {
    /// The linked coordinator leaf's encoded length.
    CoordinatorBytes,
    /// The linked member leaf's encoded length.
    MemberBytes,
    /// Every linked leaf of every constructor this transfer spends.
    ConstructorBytes,
    /// The greatest control-path depth any spent leaf reaches.
    TaptreeDepth,
    /// Every spent input's control block, summed.
    ControlBytes,
    /// The owner-signature positions' total width.
    OwnerSignatureWitnessBytes,
    /// The confidential proofs the outputs carry.
    ConfidentialProofBytes,
    /// The witness items one spent input presents before evaluation.
    InitialWitnessItems,
    /// The deepest main stack any reachable state holds.
    PeakMainStack,
    /// The deepest alternate stack any reachable state holds.
    PeakAlternateStack,
    /// The widest single element the spend can place on a stack.
    LargestElement,
    /// The validation budget the spent programs charge.
    ValidationBudget,
    /// The complete transaction's exact weight.
    CompleteWeight,
    /// The complete transaction's virtual size.
    VirtualSize,
    /// What consensus decided about these bytes.
    ConsensusVerdict,
    /// What relay policy decided about these bytes.
    RelayPolicyVerdict,
    /// How long construction and execution took.
    ConstructionAndExecutionTime,
}

impl LiveResourceRecord {
    /// All seventeen, in §18.3's order.
    pub const ALL: &'static [Self] = &[
        Self::CoordinatorBytes,
        Self::MemberBytes,
        Self::ConstructorBytes,
        Self::TaptreeDepth,
        Self::ControlBytes,
        Self::OwnerSignatureWitnessBytes,
        Self::ConfidentialProofBytes,
        Self::InitialWitnessItems,
        Self::PeakMainStack,
        Self::PeakAlternateStack,
        Self::LargestElement,
        Self::ValidationBudget,
        Self::CompleteWeight,
        Self::VirtualSize,
        Self::ConsensusVerdict,
        Self::RelayPolicyVerdict,
        Self::ConstructionAndExecutionTime,
    ];

    /// The dimension's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::CoordinatorBytes => "coordinator-bytes",
            Self::MemberBytes => "member-bytes",
            Self::ConstructorBytes => "constructor-bytes",
            Self::TaptreeDepth => "taptree-depth",
            Self::ControlBytes => "control-bytes",
            Self::OwnerSignatureWitnessBytes => "owner-signature-witness-bytes",
            Self::ConfidentialProofBytes => "confidential-proof-bytes",
            Self::InitialWitnessItems => "initial-witness-items",
            Self::PeakMainStack => "peak-main-stack",
            Self::PeakAlternateStack => "peak-alternate-stack",
            Self::LargestElement => "largest-element",
            Self::ValidationBudget => "validation-budget",
            Self::CompleteWeight => "complete-weight",
            Self::VirtualSize => "virtual-size",
            Self::ConsensusVerdict => "consensus-verdict",
            Self::RelayPolicyVerdict => "relay-policy-verdict",
            Self::ConstructionAndExecutionTime => "construction-and-execution-time",
        }
    }

    /// Whether §13.5 excludes this dimension from canonical bytes.
    ///
    /// One does: §18.3 calls construction and execution time a
    /// noncanonical diagnostic in the same breath that it asks for it,
    /// and §13.5 puts elapsed time outside the canonical bytes of every
    /// report. A study that recorded it in the document would make two
    /// runs of the same measurements produce two different documents.
    #[must_use]
    pub const fn is_noncanonical_diagnostic(self) -> bool {
        matches!(self, Self::ConstructionAndExecutionTime)
    }
}

/// Why one dimension of one measurement carries no figure.
///
/// Every arm names a component that does not exist, and none of them is
/// a measurement of zero. §18.4's rule is the reason the type exists at
/// all: an absent observation read as zero is an absent observation
/// reported as agreement.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum LiveResourceNonClaim {
    /// The encoder writes an empty range proof for every output.
    ///
    /// §12.8's construction model commits values and produces no proof,
    /// and the serializer writes one empty surjection and one empty range
    /// prefix per output because every output here is unblinded. So a
    /// private transfer's proof-byte figure is not zero-because-measured
    /// but zero-because-there-is-no-proof, and the two must not read
    /// alike. Closing this is the business of the confidential funding
    /// concept, which is a charter and not substrate.
    NoConfidentialProofIsSerialized,
    /// No target has judged these bytes, and the named component is why.
    NoTargetVerdictExists(LiveInfrastructureBlocker),
    /// A relay-policy verdict is downstream of a consensus one.
    ///
    /// Distinguished from the consensus arm because they call for
    /// different repairs: the consensus verdict is blocked by a missing
    /// digest, and the relay verdict is blocked by there being no
    /// accepted transaction for a policy to have an opinion about.
    NoRelayVerdictWithoutAConsensusOne,
}

impl LiveResourceNonClaim {
    /// The non-claim's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::NoConfidentialProofIsSerialized => "no-confidential-proof-is-serialized",
            Self::NoTargetVerdictExists(_) => "no-target-verdict-exists",
            Self::NoRelayVerdictWithoutAConsensusOne => "no-relay-verdict-without-a-consensus-one",
        }
    }
}

/// Where one dimension of one measurement stands.
///
/// Five arms, and only one of them is a number that means what a number
/// usually means. See this module's header for why the other four exist.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum DimensionStanding {
    /// An exact figure, recomputed from bytes this measurement produced.
    Measured(u64),
    /// The exact width of a slot whose contents cannot exist.
    ///
    /// The figure is a real byte count and the thing it counts is not a
    /// signature. Both halves matter: the transaction's weight depends on
    /// this width, and no claim about a signature may be read from it.
    WitnessSlotOnly(u64),
    /// This shape commits no member of this kind at all.
    ///
    /// Absent rather than zero. A one-to-one transfer has no nonzero
    /// receipt position, so it commits no member leaf, and a zero here
    /// would report a leaf of no bytes rather than no leaf.
    AbsentFromThisShape,
    /// No figure exists, for this reason.
    NotClaimable(LiveResourceNonClaim),
    /// Excluded from canonical bytes as a noncanonical diagnostic.
    ///
    /// §18.3 asks for construction and execution time and calls it
    /// noncanonical in the same sentence, and §13.5 keeps elapsed time
    /// out of every report's canonical bytes. The standing records that
    /// the dimension was considered and deliberately carries no figure,
    /// which is a different statement from carrying none by omission.
    NoncanonicalDiagnostic,
}

impl DimensionStanding {
    /// The figure, where the standing carries one that means a
    /// measurement.
    ///
    /// A slot width is deliberately not returned here. It is a real
    /// count, and a caller summing "the measured dimensions" must not
    /// pick it up without having matched on the arm that says what it is.
    #[must_use]
    pub const fn measured(self) -> Option<u64> {
        match self {
            Self::Measured(figure) => Some(figure),
            Self::WitnessSlotOnly(_)
            | Self::AbsentFromThisShape
            | Self::NotClaimable(_)
            | Self::NoncanonicalDiagnostic => None,
        }
    }

    /// The standing's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Measured(_) => "measured",
            Self::WitnessSlotOnly(_) => "witness-slot-only",
            Self::AbsentFromThisShape => "absent-from-this-shape",
            Self::NotClaimable(_) => "not-claimable",
            Self::NoncanonicalDiagnostic => "noncanonical-diagnostic",
        }
    }
}

/// One of §18.2's fifteen cases.
///
/// Transcribed in the section's own order. Two of them name two things,
/// and those carry two members apiece — see this module's header.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LiveResourceCase {
    /// Explicit one-to-one.
    ExplicitOneToOne,
    /// Explicit split.
    ExplicitSplit,
    /// Explicit merge.
    ExplicitMerge,
    /// Explicit many-to-many.
    ExplicitManyToMany,
    /// Private one-to-one.
    PrivateOneToOne,
    /// Private split and merge, where the published set claims them.
    PrivateSplitAndMerge,
    /// The widest receipt-input family the tested candidate admits.
    MaximumInputFamily,
    /// The widest destination family the tested candidate admits.
    MaximumOutputFamily,
    /// The most distinct semantic owners the tested deployment links.
    MaximumDistinctOwners,
    /// One owner authorizing several receipt inputs.
    RepeatedOwner,
    /// The sponsorless form of one transfer.
    Sponsorless,
    /// The sponsored form of the same transfer.
    Sponsored,
    /// Sponsor change present, and sponsor change absent.
    SponsorChangePresentAndAbsent,
    /// The most proof forms the tested candidate can ask for.
    LargestProofForms,
    /// The spend whose control path is deepest in a committed tree.
    DeepestControlPath,
}

impl LiveResourceCase {
    /// All fifteen, in §18.2's order.
    pub const ALL: &'static [Self] = &[
        Self::ExplicitOneToOne,
        Self::ExplicitSplit,
        Self::ExplicitMerge,
        Self::ExplicitManyToMany,
        Self::PrivateOneToOne,
        Self::PrivateSplitAndMerge,
        Self::MaximumInputFamily,
        Self::MaximumOutputFamily,
        Self::MaximumDistinctOwners,
        Self::RepeatedOwner,
        Self::Sponsorless,
        Self::Sponsored,
        Self::SponsorChangePresentAndAbsent,
        Self::LargestProofForms,
        Self::DeepestControlPath,
    ];

    /// The case's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ExplicitOneToOne => "explicit-one-to-one",
            Self::ExplicitSplit => "explicit-split",
            Self::ExplicitMerge => "explicit-merge",
            Self::ExplicitManyToMany => "explicit-many-to-many",
            Self::PrivateOneToOne => "private-one-to-one",
            Self::PrivateSplitAndMerge => "private-split-and-merge",
            Self::MaximumInputFamily => "maximum-input-family",
            Self::MaximumOutputFamily => "maximum-output-family",
            Self::MaximumDistinctOwners => "maximum-distinct-owners",
            Self::RepeatedOwner => "repeated-owner",
            Self::Sponsorless => "sponsorless",
            Self::Sponsored => "sponsored",
            Self::SponsorChangePresentAndAbsent => "sponsor-change-present-and-absent",
            Self::LargestProofForms => "largest-proof-forms",
            Self::DeepestControlPath => "deepest-control-path",
        }
    }

    /// A short identifier separating one case's predecessors from every
    /// other case's.
    ///
    /// The outpoints a measurement consumes are built from this, so no
    /// two measurements in the whole study spend the same predecessor and
    /// no measurement's bytes can be another's by accident.
    const fn identifier(self) -> u8 {
        match self {
            Self::ExplicitOneToOne => 0x11,
            Self::ExplicitSplit => 0x12,
            Self::ExplicitMerge => 0x13,
            Self::ExplicitManyToMany => 0x14,
            Self::PrivateOneToOne => 0x15,
            Self::PrivateSplitAndMerge => 0x16,
            Self::MaximumInputFamily => 0x17,
            Self::MaximumOutputFamily => 0x18,
            Self::MaximumDistinctOwners => 0x19,
            Self::RepeatedOwner => 0x1a,
            Self::Sponsorless => 0x1b,
            Self::Sponsored => 0x1c,
            Self::SponsorChangePresentAndAbsent => 0x1d,
            Self::LargestProofForms => 0x1e,
            Self::DeepestControlPath => 0x1f,
        }
    }
}

/// What sponsor region one measured transaction carries.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MeasuredSponsorRole {
    /// No sponsor region, and therefore no fee output.
    Absent,
    /// One sponsor input funding the fee, with no change output.
    PresentWithoutChange,
    /// One sponsor input funding the fee, with a change output.
    PresentWithChange,
}

impl MeasuredSponsorRole {
    /// The form a request under this role asks for.
    const fn form(self) -> RequestedForm {
        match self {
            Self::Absent => RequestedForm::Sponsorless,
            Self::PresentWithoutChange | Self::PresentWithChange => RequestedForm::Sponsored,
        }
    }

    /// Whether the request asks for the change role.
    const fn change(self) -> SponsorChangeRequest {
        match self {
            Self::Absent | Self::PresentWithoutChange => SponsorChangeRequest::NotRequested,
            Self::PresentWithChange => SponsorChangeRequest::Requested,
        }
    }
}

/// One complete transaction a case asks to be built.
///
/// Owners are named by index into the two published scalars, because the
/// tested deployment links constructors for exactly those two and a third
/// index would name an owner nothing was linked for.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeasurementRecipe {
    case: LiveResourceCase,
    member: u8,
    representation: LiveTransferRepresentationPlan,
    sources: Vec<(usize, u64)>,
    destinations: Vec<(usize, u64)>,
    sponsor: MeasuredSponsorRole,
}

impl MeasurementRecipe {
    /// The case this recipe serves.
    #[must_use]
    pub const fn case(&self) -> LiveResourceCase {
        self.case
    }

    /// Which member of the case this is.
    #[must_use]
    pub const fn member(&self) -> u8 {
        self.member
    }

    /// The representation plan the transaction is built under.
    #[must_use]
    pub const fn representation(&self) -> LiveTransferRepresentationPlan {
        self.representation
    }

    /// The sponsor region the transaction carries.
    #[must_use]
    pub const fn sponsor(&self) -> MeasuredSponsorRole {
        self.sponsor
    }

    /// How many receipts the transaction consumes.
    #[must_use]
    pub const fn receipt_inputs(&self) -> usize {
        self.sources.len()
    }

    /// How many receipts the transaction creates.
    #[must_use]
    pub const fn destinations(&self) -> usize {
        self.destinations.len()
    }
}

/// One complete transaction, measured across §18.3's seventeen
/// dimensions.
///
/// The bytes are carried because §18.4's comparison is *over the same
/// exact bytes*: a figure compared against a target's own is only
/// comparable if both sides can be shown to be about one serialization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionMeasurement {
    recipe: MeasurementRecipe,
    shape: LiveTransferShape,
    bytes: Vec<u8>,
    spent_leaves: BTreeSet<LiveTransferLeafRole>,
    dimensions: BTreeMap<LiveResourceRecord, DimensionStanding>,
}

impl TransactionMeasurement {
    /// The recipe this measurement realized.
    #[must_use]
    pub const fn recipe(&self) -> &MeasurementRecipe {
        &self.recipe
    }

    /// The shape the construction selected.
    #[must_use]
    pub const fn shape(&self) -> LiveTransferShape {
        self.shape
    }

    /// The exact complete transaction bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Every committed leaf this transaction's inputs spend.
    #[must_use]
    pub const fn spent_leaves(&self) -> &BTreeSet<LiveTransferLeafRole> {
        &self.spent_leaves
    }

    /// Where each of §18.3's seventeen dimensions stands.
    #[must_use]
    pub const fn dimensions(&self) -> &BTreeMap<LiveResourceRecord, DimensionStanding> {
        &self.dimensions
    }

    /// One dimension's standing.
    #[must_use]
    pub fn standing(&self, record: LiveResourceRecord) -> Option<DimensionStanding> {
        self.dimensions.get(&record).copied()
    }

    /// One dimension's figure, where it is a measurement.
    #[must_use]
    pub fn figure(&self, record: LiveResourceRecord) -> Option<u64> {
        self.standing(record).and_then(DimensionStanding::measured)
    }
}

/// One §18.2 case and its one or two complete transactions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseMeasurement {
    case: LiveResourceCase,
    members: Vec<TransactionMeasurement>,
}

impl CaseMeasurement {
    /// Which case this is.
    #[must_use]
    pub const fn case(&self) -> LiveResourceCase {
        self.case
    }

    /// Every complete transaction the case asked for.
    #[must_use]
    pub fn members(&self) -> &[TransactionMeasurement] {
        &self.members
    }
}

/// Why the resource study could not be measured.
///
/// Every arm is a defect in the substrate or in a recipe, and none of
/// them is a resource finding: a study that could not build its
/// transactions establishes nothing about their cost.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ResourceStudyRefusal {
    /// The live-transfer substrate could not be built.
    SubstrateUnavailable,
    /// One recipe's transaction could not be finalized.
    MemberNotFinalizable {
        /// Which case.
        case: LiveResourceCase,
        /// Which member of it.
        member: u8,
    },
    /// One recipe's transaction could not be completed into bytes.
    MemberNotCompletable {
        /// Which case.
        case: LiveResourceCase,
        /// Which member of it.
        member: u8,
    },
    /// The tested deployment carries no leaf to measure a deepest
    /// control path over.
    NoCommittedLeafToMeasure,
}

/// The sponsor envelope every measured sponsored transaction is built
/// with.
///
/// It offers one input and the fee, states a change destination where the
/// recipe asks for one, and answers the signing request with bytes of a
/// signature's width that authorize nothing. §1.9 puts the sponsor's own
/// authorization outside protocol data and Wave 10 recorded that no
/// adapter produces one — so a resource study either measures the
/// sponsored form with a filled slot or does not measure it at all, and
/// the second would leave four of §18.2's fifteen cases unmeasured for a
/// reason that has nothing to do with resources.
struct MeasuredSponsorEnvelope {
    offer: SponsorOffer,
}

impl MeasuredSponsorEnvelope {
    /// The envelope one recipe asks for.
    fn new(input: Outpoint, role: MeasuredSponsorRole) -> Result<Self, VectorError> {
        let residual = matches!(role, MeasuredSponsorRole::PresentWithChange)
            .then_some(ValueField::Explicit(MEASURED_SPONSOR_CHANGE));
        Ok(Self {
            offer: SponsorOffer::new([input], MEASURED_SPONSOR_FEE, residual)
                .map_err(|_| VectorError::LiveSubstrateUnavailable)?,
        })
    }
}

impl SponsorCapability for MeasuredSponsorEnvelope {
    fn offer(&self) -> SponsorOffer {
        self.offer.clone()
    }

    /// No destination of its own, which is not the same as no change.
    ///
    /// The residual is stated in the offer above; what this method
    /// answers is *where* it pays, and the deployment already declares
    /// one admitted sponsor-change program and refuses any other. So an
    /// envelope naming a program of its own could only ever name the
    /// deployment's — which the builder substitutes when this is absent —
    /// or a program the construction refuses. Deferring is the honest
    /// spelling of a sponsor that accepts the deployment's terms, and it
    /// is what makes the measured change output the one a deployment
    /// would actually create.
    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        None
    }

    fn sign(&self, request: &SponsorSigningRequest) -> Option<SponsorSignature> {
        Some(SponsorSignature::new(
            request.transaction().to_vec(),
            vec![UNAUTHORIZING_SIGNATURE.to_vec()],
        ))
    }
}

/// Every recipe §18.2's fifteen cases ask for, in the section's order.
///
/// # Errors
///
/// [`ResourceStudyRefusal::SubstrateUnavailable`] when the tested bundle
/// cannot be built, and
/// [`ResourceStudyRefusal::NoCommittedLeafToMeasure`] when it carries no
/// committed leaf for the deepest-control-path case to be about.
pub fn measurement_recipes() -> Result<Vec<MeasurementRecipe>, ResourceStudyRefusal> {
    let mut recipes = transfer_shape_recipes();
    recipes.extend(family_and_owner_recipes());
    recipes.extend(sponsor_recipes());
    recipes.push(deepest_recipe(deepest_committed_shape()?));
    Ok(recipes)
}

/// §18.2's first six bullets: the four explicit compositions and the
/// private one-to-one, split, and merge.
///
/// Grouped because they are one question asked six ways — what one
/// composition of §5.3 costs under each representation plan — and
/// because the four amounts each carries are only readable beside the
/// others.
fn transfer_shape_recipes() -> Vec<MeasurementRecipe> {
    use LiveResourceCase as C;
    use LiveTransferRepresentationPlan::{Explicit, PrivateCommitted};
    use MeasuredSponsorRole::Absent;

    vec![
        recipe(
            C::ExplicitOneToOne,
            0,
            Explicit,
            &[(0, 1000)],
            &[(1, 1000)],
            Absent,
        ),
        recipe(
            C::ExplicitSplit,
            0,
            Explicit,
            &[(0, 1000)],
            &[(1, 400), (0, 600)],
            Absent,
        ),
        recipe(
            C::ExplicitMerge,
            0,
            Explicit,
            &[(0, 400), (1, 600)],
            &[(0, 1000)],
            Absent,
        ),
        recipe(
            C::ExplicitManyToMany,
            0,
            Explicit,
            &[(0, 300), (1, 700)],
            &[(1, 450), (0, 550)],
            Absent,
        ),
        recipe(
            C::PrivateOneToOne,
            0,
            PrivateCommitted,
            &[(0, 1000)],
            &[(1, 1000)],
            Absent,
        ),
        recipe(
            C::PrivateSplitAndMerge,
            0,
            PrivateCommitted,
            &[(0, 1000)],
            &[(1, 400), (0, 600)],
            Absent,
        ),
        recipe(
            C::PrivateSplitAndMerge,
            1,
            PrivateCommitted,
            &[(0, 400), (1, 600)],
            &[(0, 1000)],
            Absent,
        ),
    ]
}

/// §18.2's family and owner bullets: the widest input family, the widest
/// output family, the most distinct owners, and the repeated owner.
///
/// The two "maximum" cases are maximum *for the tested candidate*, whose
/// published set admits three receipts in and three out. The distinct-owner
/// maximum is two, because the tested deployment links constructors for
/// exactly the two published owners — a ceiling of the test material and
/// not of the design, and [`LiveResourceCase::MaximumDistinctOwners`] is
/// where the study says which.
fn family_and_owner_recipes() -> Vec<MeasurementRecipe> {
    use LiveResourceCase as C;
    use LiveTransferRepresentationPlan::Explicit;
    use MeasuredSponsorRole::Absent;

    vec![
        recipe(
            C::MaximumInputFamily,
            0,
            Explicit,
            &[(0, 300), (1, 300), (0, 400)],
            &[(1, 1000)],
            Absent,
        ),
        recipe(
            C::MaximumOutputFamily,
            0,
            Explicit,
            &[(0, 1000)],
            &[(1, 300), (0, 300), (1, 400)],
            Absent,
        ),
        recipe(
            C::MaximumDistinctOwners,
            0,
            Explicit,
            &[(0, 500), (1, 500)],
            &[(0, 400), (1, 600)],
            Absent,
        ),
        recipe(
            C::RepeatedOwner,
            0,
            Explicit,
            &[(0, 400), (0, 600)],
            &[(1, 1000)],
            Absent,
        ),
    ]
}

/// §18.2's sponsor bullets, and the largest proof forms.
///
/// The sponsorless and sponsored members move the same value between the
/// same owners, so laying them side by side is what makes the sponsor
/// region's cost readable; the change pair does the same for the change
/// role over a narrower transfer, so the two differences are not being
/// read off one subtraction.
fn sponsor_recipes() -> Vec<MeasurementRecipe> {
    use LiveResourceCase as C;
    use LiveTransferRepresentationPlan::{Explicit, PrivateCommitted};
    use MeasuredSponsorRole as S;

    vec![
        recipe(
            C::Sponsorless,
            0,
            Explicit,
            &[(0, 500), (1, 500)],
            &[(1, 500), (0, 500)],
            S::Absent,
        ),
        recipe(
            C::Sponsored,
            0,
            Explicit,
            &[(0, 500), (1, 500)],
            &[(1, 500), (0, 500)],
            S::PresentWithoutChange,
        ),
        recipe(
            C::SponsorChangePresentAndAbsent,
            0,
            Explicit,
            &[(0, 1000)],
            &[(1, 1000)],
            S::PresentWithChange,
        ),
        recipe(
            C::SponsorChangePresentAndAbsent,
            1,
            Explicit,
            &[(0, 1000)],
            &[(1, 1000)],
            S::PresentWithoutChange,
        ),
        recipe(
            C::LargestProofForms,
            0,
            PrivateCommitted,
            &[(0, 900)],
            &[(0, 300), (1, 300), (0, 300)],
            S::Absent,
        ),
    ]
}

/// One recipe, spelled once.
fn recipe(
    case: LiveResourceCase,
    member: u8,
    representation: LiveTransferRepresentationPlan,
    sources: &[(usize, u64)],
    destinations: &[(usize, u64)],
    sponsor: MeasuredSponsorRole,
) -> MeasurementRecipe {
    MeasurementRecipe {
        case,
        member,
        representation,
        sources: sources.to_vec(),
        destinations: destinations.to_vec(),
        sponsor,
    }
}

/// The shape whose committed leaf sits deepest in a tested tree.
///
/// Computed from the committed trees rather than assumed, because which
/// leaf is deepest is a property of the linker's tie-break over a leaf set
/// nobody laid out by hand. Only coordinator leaves are considered: a
/// member leaf is reached by a transfer of *some* shape carrying that
/// input count, and the coordinator of that same shape is spent alongside
/// it, so a transfer built for the deepest coordinator reaches at least as
/// deep as one built for the deepest member.
///
/// # Errors
///
/// [`ResourceStudyRefusal::SubstrateUnavailable`] when the bundle cannot
/// be built, and [`ResourceStudyRefusal::NoCommittedLeafToMeasure`] when
/// it carries no coordinator leaf at all.
pub fn deepest_committed_shape() -> Result<LiveTransferShape, ResourceStudyRefusal> {
    let bundle =
        demonstration_live_bundle().map_err(|_| ResourceStudyRefusal::SubstrateUnavailable)?;

    let mut deepest: Option<(u32, LiveTransferShape)> = None;
    for constructor in bundle.constructors().values() {
        if constructor.representation() != LiveTransferRepresentationPlan::Explicit {
            continue;
        }
        for (leaf, recipe) in constructor.taptree().recipes() {
            let LiveTransferLeafRole::Coordinator { shape, .. } = leaf else {
                continue;
            };
            // Ties are broken toward the *earlier* shape in the leaf
            // ordering, so two runs over the same tree name the same
            // shape. A strict comparison keeps the first one seen.
            if deepest.is_none_or(|(depth, _)| recipe.depth() > depth) {
                deepest = Some((recipe.depth(), *shape));
            }
        }
    }

    deepest
        .map(|(_, shape)| shape)
        .ok_or(ResourceStudyRefusal::NoCommittedLeafToMeasure)
}

/// The recipe realizing one shape exactly.
///
/// Owners cycle through the two published scalars, so a shape with more
/// receipt inputs than published owners repeats one — which is a fact
/// about the tested deployment rather than about the shape, and
/// [`LiveResourceCase::MaximumDistinctOwners`] is where it is recorded.
fn deepest_recipe(shape: LiveTransferShape) -> MeasurementRecipe {
    let inputs = usize::from(shape.receipt_inputs());
    let outputs = usize::from(shape.receipt_outputs());
    let total = MEASURED_RECEIPT_UNIT.saturating_mul(inputs as u64);
    let each = total / outputs as u64;

    let sources: Vec<(usize, u64)> = (0..inputs)
        .map(|index| (index % 2, MEASURED_RECEIPT_UNIT))
        .collect();
    let destinations: Vec<(usize, u64)> = (0..outputs)
        .map(|index| {
            let amount = if index + 1 == outputs {
                total - each * (outputs as u64 - 1)
            } else {
                each
            };
            (index % 2, amount)
        })
        .collect();

    let sponsor = match (shape.sponsored(), shape.sponsor_change()) {
        (false, _) => MeasuredSponsorRole::Absent,
        (true, SponsorChangePresence::Absent) => MeasuredSponsorRole::PresentWithoutChange,
        (true, SponsorChangePresence::Present) => MeasuredSponsorRole::PresentWithChange,
    };

    recipe(
        LiveResourceCase::DeepestControlPath,
        0,
        LiveTransferRepresentationPlan::Explicit,
        &sources,
        &destinations,
        sponsor,
    )
}

/// Build and measure every §18.2 case.
///
/// One entry per case in §18.2's order, each carrying its one or two
/// complete transactions. Nothing is skipped: a case that could not be
/// built refuses the whole study rather than being dropped from it, so
/// the case count is §18.2's count whatever any of them answered.
///
/// # Errors
///
/// [`ResourceStudyRefusal`], naming the first case whose transaction did
/// not finalize or did not complete.
pub fn measure_resource_cases() -> Result<Vec<CaseMeasurement>, ResourceStudyRefusal> {
    let target = reviewed_target().map_err(|_| ResourceStudyRefusal::SubstrateUnavailable)?;
    let abi = demonstration_live_abi().map_err(|_| ResourceStudyRefusal::SubstrateUnavailable)?;
    let bundle =
        demonstration_live_bundle().map_err(|_| ResourceStudyRefusal::SubstrateUnavailable)?;

    let mut walks: BTreeMap<LiveTransferLeafRole, ProgramWalk> = BTreeMap::new();
    let mut cases: Vec<CaseMeasurement> = Vec::with_capacity(LiveResourceCase::ALL.len());

    for recipe in measurement_recipes()? {
        let measured = measure_one(&target, &abi, &bundle, &recipe, &mut walks)?;
        match cases.last_mut() {
            Some(last) if last.case == recipe.case => last.members.push(measured),
            _ => cases.push(CaseMeasurement {
                case: recipe.case,
                members: vec![measured],
            }),
        }
    }

    Ok(cases)
}

/// Build one recipe's complete transaction and read its dimensions.
fn measure_one(
    target: &ReviewedElementsTapscriptDefinition,
    abi: &CandidateLiveTransferAbi,
    bundle: &CandidateLinkedLiveTransferBundle,
    recipe: &MeasurementRecipe,
    walks: &mut BTreeMap<LiveTransferLeafRole, ProgramWalk>,
) -> Result<TransactionMeasurement, ResourceStudyRefusal> {
    let not_final = || ResourceStudyRefusal::MemberNotFinalizable {
        case: recipe.case,
        member: recipe.member,
    };
    let not_complete = || ResourceStudyRefusal::MemberNotCompletable {
        case: recipe.case,
        member: recipe.member,
    };

    let mut points = Vec::with_capacity(recipe.sources.len());
    let mut views = Vec::with_capacity(recipe.sources.len());
    for (index, (owner, amount)) in recipe.sources.iter().enumerate() {
        let point = measured_outpoint(recipe, 0x01, index).map_err(|_| not_final())?;
        let key = published_owner(scalar(*owner)).map_err(|_| not_final())?;
        let program = abi
            .destinations()
            .get(&OwnerParameter::new(key), recipe.representation)
            .ok_or_else(not_final)?
            .instance()
            .program()
            .to_vec();
        points.push(point);
        views.push(PublicOutputView::new(
            point,
            AssetField::Explicit(abi.symbols().protocol_asset()),
            predecessor_value(abi, recipe.representation, *amount, index)
                .map_err(|_| not_final())?,
            program,
        ));
    }
    let view = PublicConstructionView::new(views).map_err(|_| not_final())?;

    let mut destinations = Vec::with_capacity(recipe.destinations.len());
    for (owner, amount) in &recipe.destinations {
        let key = published_owner(scalar(*owner)).map_err(|_| not_final())?;
        let value = ProtocolValue::new(*amount).map_err(|_| not_final())?;
        destinations.push(LiveReceiptDestination::new(OwnerParameter::new(key), value));
    }

    let randomness = match recipe.representation {
        LiveTransferRepresentationPlan::Explicit => None,
        LiveTransferRepresentationPlan::PrivateCommitted => Some(
            PublicTestRandomness::from_published_bytes(MEASURED_DESTINATION_RANDOMNESS),
        ),
    };
    let request = LiveTransferRequest::new(
        points,
        destinations,
        recipe.representation,
        recipe.sponsor.form(),
        recipe.sponsor.change(),
        randomness,
    )
    .map_err(|_| not_final())?;

    let envelope = match recipe.sponsor {
        MeasuredSponsorRole::Absent => None,
        role => Some(
            MeasuredSponsorEnvelope::new(
                measured_outpoint(recipe, 0x03, 0).map_err(|_| not_final())?,
                role,
            )
            .map_err(|_| not_final())?,
        ),
    };
    let sponsor: Option<&dyn SponsorCapability> = envelope
        .as_ref()
        .map(|envelope| envelope as &dyn SponsorCapability);
    let private: Option<&dyn PrivateValueCapability> = match recipe.representation {
        LiveTransferRepresentationPlan::Explicit => None,
        LiveTransferRepresentationPlan::PrivateCommitted => Some(&OracleFixtureValues),
    };

    let finalization = finalize_live_transfer(target, abi, &request, &view, sponsor, private)
        .map_err(|_| not_final())?;
    let report = finalization.report().clone();
    let finalized = finalization.into_finalized();

    // Every owner "signs" with bytes that authorize nothing, so the
    // signature *positions* are filled and the widths below are the ones
    // a real transfer would carry. Nothing about a signature follows.
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
    let authorized = authorize_live_transfer(finalized, responses).map_err(|_| not_complete())?;
    let finalized = authorized.finalized().clone();
    let built =
        complete_live_transfer(target, authorized, report, sponsor).map_err(|_| not_complete())?;

    let dimensions = read_dimensions(target, bundle, recipe, &finalized, &built, walks);
    let spent_leaves = finalized
        .receipts()
        .iter()
        .map(transaction::live_finalize::ReceiptInputRecord::leaf)
        .collect();

    Ok(TransactionMeasurement {
        recipe: recipe.clone(),
        shape: finalized.shape(),
        bytes: built.bytes(),
        spent_leaves,
        dimensions,
    })
}

/// The leaf-derived figures one complete transaction's inputs settle.
///
/// A struct rather than a tuple because it has ten members and every one
/// of them is a different dimension of §18.3; a ten-tuple would let two
/// of them be swapped at the call site without anything noticing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct SpentLeafFigures {
    coordinator_bytes: Option<u64>,
    member_bytes: Option<u64>,
    constructor_bytes: u64,
    control_bytes: u64,
    deepest_path: u64,
    signature_bytes: u64,
    widest_element: u64,
    peak_main: u64,
    peak_alternate: u64,
    validation_budget: u64,
}

/// Read every figure the spent leaves settle.
///
/// Each is read off the *linked* programs the transaction's own control
/// blocks authenticate, so a relink that moved a leaf moves these with
/// it, and none of them is looked up from a table this study kept.
fn read_spent_leaves(
    target: &ReviewedElementsTapscriptDefinition,
    bundle: &CandidateLinkedLiveTransferBundle,
    recipe: &MeasurementRecipe,
    finalized: &FinalizedLiveTransfer,
    walks: &mut BTreeMap<LiveTransferLeafRole, ProgramWalk>,
) -> SpentLeafFigures {
    let mut figures = SpentLeafFigures::default();
    let mut constructors: BTreeSet<(OwnerParameter, LiveTransferRepresentationPlan)> =
        BTreeSet::new();

    for record in finalized.receipts() {
        let leaf = record.leaf();
        let script = record.leaf_script().len() as u64;
        match leaf {
            LiveTransferLeafRole::Coordinator { .. } => {
                figures.coordinator_bytes =
                    Some(figures.coordinator_bytes.unwrap_or(0).max(script));
            }
            LiveTransferLeafRole::Member { .. } => {
                figures.member_bytes = Some(figures.member_bytes.unwrap_or(0).max(script));
            }
        }

        let control = record.control_block().len() as u64;
        figures.control_bytes = figures.control_bytes.saturating_add(control);
        // A control block is a parity-and-version byte, an x-only
        // internal key, and one thirty-two-byte node per level. The depth
        // is read back out of the width rather than looked up, so it is
        // the depth these exact bytes carry.
        figures.deepest_path = figures.deepest_path.max(control.saturating_sub(33) / 32);

        figures.signature_bytes = figures
            .signature_bytes
            .saturating_add(UNAUTHORIZING_SIGNATURE.len() as u64);
        constructors.insert((record.owner().clone(), recipe.representation));

        let walk = walk_for(target, bundle, leaf, walks);
        figures.peak_main = figures.peak_main.max(walk.peak_main);
        figures.peak_alternate = figures.peak_alternate.max(walk.peak_alternate);
        figures.widest_element = figures.widest_element.max(walk.widest_push);
        figures.validation_budget = figures
            .validation_budget
            .saturating_add(walk.validation_budget);
    }

    figures.constructor_bytes = constructors
        .iter()
        .filter_map(|(owner, representation)| bundle.constructor(owner, *representation))
        .map(|constructor| {
            constructor
                .programs()
                .values()
                .filter_map(|program| program.charged(ResourceDimension::ScriptBytes))
                .fold(0_u64, u64::saturating_add)
        })
        .fold(0_u64, u64::saturating_add);

    figures
}

/// Read §18.3's seventeen dimensions off one complete transaction.
fn read_dimensions(
    target: &ReviewedElementsTapscriptDefinition,
    bundle: &CandidateLinkedLiveTransferBundle,
    recipe: &MeasurementRecipe,
    finalized: &FinalizedLiveTransfer,
    built: &CandidateLiveTransferTransaction,
    walks: &mut BTreeMap<LiveTransferLeafRole, ProgramWalk>,
) -> BTreeMap<LiveResourceRecord, DimensionStanding> {
    use DimensionStanding as Standing;
    use LiveResourceRecord as Record;

    let leaves = read_spent_leaves(target, bundle, recipe, finalized, walks);

    // The witness items one input presents, from the ABI's own order: the
    // owner signature, the leaf, and the control block that authenticates
    // it. Read off the built witnesses rather than written as three.
    let mut initial_items = 0_u64;
    let mut widest_element = leaves.widest_element;
    for witness in built.transaction().witnesses() {
        initial_items = initial_items.max(witness.stack().len() as u64);
        for item in witness.stack() {
            widest_element = widest_element.max(item.len() as u64);
        }
    }

    let proofs = match recipe.representation {
        // An explicit output has no proof form at all, so the dimension
        // has no member here rather than a figure of zero.
        LiveTransferRepresentationPlan::Explicit => Standing::AbsentFromThisShape,
        LiveTransferRepresentationPlan::PrivateCommitted => {
            Standing::NotClaimable(LiveResourceNonClaim::NoConfidentialProofIsSerialized)
        }
    };

    BTreeMap::from([
        (
            Record::CoordinatorBytes,
            leaves
                .coordinator_bytes
                .map_or(Standing::AbsentFromThisShape, Standing::Measured),
        ),
        (
            Record::MemberBytes,
            leaves
                .member_bytes
                .map_or(Standing::AbsentFromThisShape, Standing::Measured),
        ),
        (
            Record::ConstructorBytes,
            Standing::Measured(leaves.constructor_bytes),
        ),
        (
            Record::TaptreeDepth,
            Standing::Measured(leaves.deepest_path),
        ),
        (
            Record::ControlBytes,
            Standing::Measured(leaves.control_bytes),
        ),
        (
            Record::OwnerSignatureWitnessBytes,
            Standing::WitnessSlotOnly(leaves.signature_bytes),
        ),
        (Record::ConfidentialProofBytes, proofs),
        (
            Record::InitialWitnessItems,
            Standing::Measured(initial_items),
        ),
        (Record::PeakMainStack, Standing::Measured(leaves.peak_main)),
        (
            Record::PeakAlternateStack,
            Standing::Measured(leaves.peak_alternate),
        ),
        (Record::LargestElement, Standing::Measured(widest_element)),
        (
            Record::ValidationBudget,
            Standing::Measured(leaves.validation_budget),
        ),
        (
            Record::CompleteWeight,
            Standing::Measured(built.transaction().weight()),
        ),
        (
            Record::VirtualSize,
            Standing::Measured(built.transaction().virtual_size()),
        ),
        // §1.7: no owner signature can be produced, so these bytes were
        // never offered to a target and have earned no verdict. The
        // native lane's own submitted bytes are the exception, and they
        // are compared in `crate::live_comparison` rather than borrowed
        // here — those are different bytes.
        (
            Record::ConsensusVerdict,
            Standing::NotClaimable(LiveResourceNonClaim::NoTargetVerdictExists(
                LiveInfrastructureBlocker::OwnerSighashNotComputable,
            )),
        ),
        (
            Record::RelayPolicyVerdict,
            Standing::NotClaimable(LiveResourceNonClaim::NoRelayVerdictWithoutAConsensusOne),
        ),
        (
            Record::ConstructionAndExecutionTime,
            Standing::NoncanonicalDiagnostic,
        ),
    ])
}

/// What one abstract walk of one linked leaf established.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ProgramWalk {
    peak_main: u64,
    peak_alternate: u64,
    widest_push: u64,
    validation_budget: u64,
}

/// The walk of one leaf, computed once and reused.
///
/// Memoized because the walk validates every prefix of a program and the
/// study spends the same leaves in many cases; the result is a pure
/// function of the leaf and the target, so a cached answer is the answer.
fn walk_for(
    target: &ReviewedElementsTapscriptDefinition,
    bundle: &CandidateLinkedLiveTransferBundle,
    leaf: LiveTransferLeafRole,
    walks: &mut BTreeMap<LiveTransferLeafRole, ProgramWalk>,
) -> ProgramWalk {
    if let Some(walked) = walks.get(&leaf) {
        return *walked;
    }
    let walked = bundle
        .constructors()
        .values()
        .find_map(|constructor| constructor.program(leaf))
        .map_or_else(ProgramWalk::default, |linked| {
            walk_program(target, linked.program())
        });
    walks.insert(leaf, walked);
    walked
}

/// Walk one linked program and record what its stacks reach.
///
/// The peaks are the *validator's* own, taken by validating every prefix
/// of the program and reading the deepest state any of them reaches —
/// which is the same technique the prototype lane uses, and for the same
/// reason: a hand count of pushes and pops is a second implementation of
/// the interpreter, and this study is not entitled to one.
///
/// The alternate peak has no precedent to copy: nothing in this
/// workspace had measured one before, and a dimension §18.3 names cannot
/// be left out because no earlier wave needed it. What it measures to is
/// zero at every leaf this study spends — no live program moves an item
/// across — and that is a finding rather than a gap, which is why it is
/// walked for rather than assumed: the walk is the same one that reports
/// the main peak, and the main peak is what shows it ran.
fn walk_program(
    target: &ReviewedElementsTapscriptDefinition,
    program: &TapscriptProgram,
) -> ProgramWalk {
    let initial = live_program_precondition(target);
    let mut walk = ProgramWalk {
        peak_main: initial.main().len() as u64,
        peak_alternate: initial.alternate().len() as u64,
        widest_push: 0,
        validation_budget: resource_projection(target, program)
            .get(&ResourceDimension::ValidationBudget)
            .copied()
            .unwrap_or(0),
    };

    for instruction in program.instructions() {
        if let TapscriptInstruction::Push(item) = instruction {
            walk.widest_push = walk.widest_push.max(pushed_width(item));
        }
    }

    for length in 1..=program.len() {
        let Ok(prefix) = TapscriptProgram::new(program.instructions()[..length].to_vec()) else {
            continue;
        };
        let Ok(outcome) = validate_program(
            target,
            &prefix,
            &initial,
            AbstractLimits::for_target(target),
        ) else {
            continue;
        };
        for state in outcome
            .success()
            .iter()
            .chain(outcome.nonaborting_failure())
        {
            walk.peak_main = walk.peak_main.max(state.main().len() as u64);
            walk.peak_alternate = walk.peak_alternate.max(state.alternate().len() as u64);
        }
    }

    walk
}

/// How wide one pushed literal is.
fn pushed_width(item: &StackItem) -> u64 {
    item.bytes().len() as u64
}

/// The published scalar one owner index names.
const fn scalar(owner: usize) -> &'static [u8; 32] {
    if owner == 0 {
        &FIRST_SCALAR
    } else {
        &SECOND_SCALAR
    }
}

/// The outpoint one measurement's predecessor sits at.
///
/// The case identifier separates the cases, the member separates a case's
/// two halves, and the role byte separates a receipt from a sponsor coin,
/// so no two measurements in the study spend the same predecessor.
fn measured_outpoint(
    recipe: &MeasurementRecipe,
    role: u8,
    index: usize,
) -> Result<Outpoint, VectorError> {
    let mut identifier = [recipe.case.identifier(); 32];
    identifier[0] = role;
    identifier[1] = recipe.member;
    let index = u32::try_from(index).map_err(|_| VectorError::LiveSubstrateUnavailable)?;
    Outpoint::new(Txid::from_internal(identifier), index)
        .map_err(|_| VectorError::LiveSubstrateUnavailable)
}

/// The value field one measured predecessor carries under one plan.
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
                    &PublicTestRandomness::from_published_bytes(MEASURED_PREDECESSOR_RANDOMNESS),
                    position,
                )
                .ok_or(VectorError::LiveSubstrateUnavailable)?;
            Ok(ValueField::Commitment(commitment))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CaseMeasurement, DimensionStanding, LiveResourceCase, LiveResourceNonClaim,
        LiveResourceRecord, MeasuredSponsorRole, deepest_committed_shape, measure_resource_cases,
        measurement_recipes,
    };
    use crate::live_evidence::LiveInfrastructureBlocker;
    use std::collections::BTreeSet;
    use std::sync::OnceLock;
    use tapscript::upstream::LiveTransferRepresentationPlan;

    /// The measured study, built once for the whole module.
    ///
    /// Seventeen complete transactions is real construction work, and a
    /// module that rebuilt them per test would spend it per test. The
    /// result is a pure function of constants in this file, so a cached
    /// answer is the answer.
    fn study() -> &'static [CaseMeasurement] {
        static CACHED: OnceLock<Vec<CaseMeasurement>> = OnceLock::new();
        CACHED.get_or_init(|| measure_resource_cases().expect("every case is measurable"))
    }

    #[test]
    fn the_study_measures_section_eighteen_twos_fifteen_cases() {
        // The case census, checked against §18.2's own list rather than
        // against a count. A case dropped from `ALL` would silently
        // shorten the study, so the two lists are compared in order.
        let measured: Vec<_> = study().iter().map(CaseMeasurement::case).collect();
        assert_eq!(measured, LiveResourceCase::ALL.to_vec());

        let members: usize = study().iter().map(|case| case.members().len()).sum();
        assert_eq!(members, 17);
        assert_eq!(measurement_recipes().expect("the recipes build").len(), 17);
    }

    #[test]
    fn the_two_double_cases_are_the_two_bullets_that_name_two_things() {
        // §18.2's bullets, and which of them carry two members. Asserted
        // rather than described, because a third double case appearing
        // would mean this module had stopped transcribing the section.
        for case in study() {
            let expected = match case.case() {
                LiveResourceCase::PrivateSplitAndMerge
                | LiveResourceCase::SponsorChangePresentAndAbsent => 2,
                _ => 1,
            };
            assert_eq!(
                case.members().len(),
                expected,
                "{} carries the wrong member count",
                case.case().name(),
            );
        }
    }

    #[test]
    fn every_measured_transaction_is_a_different_serialization() {
        // §18.2's cases overlap by design — a many-to-many transfer is
        // also a two-owner one — so the study has to show it built
        // seventeen transactions rather than seventeen names for fewer.
        // Distinct bytes is the strongest available statement of that.
        let mut seen: BTreeSet<Vec<u8>> = BTreeSet::new();
        let mut count = 0;
        for case in study() {
            for member in case.members() {
                assert_ne!(member.bytes().len(), 0);
                assert!(
                    seen.insert(member.bytes().to_vec()),
                    "{} member {} repeats another measurement's bytes",
                    case.case().name(),
                    member.recipe().member(),
                );
                count += 1;
            }
        }
        assert_eq!(seen.len(), count);
    }

    #[test]
    fn every_measurement_records_all_seventeen_dimensions_separately() {
        // §18.3 says "record separately", so every measurement carries a
        // standing for every dimension. A missing key would be a
        // dimension the study quietly stopped recording, and reading a
        // missing key as zero is exactly what §18.4 forbids.
        for case in study() {
            for member in case.members() {
                assert_eq!(member.dimensions().len(), LiveResourceRecord::ALL.len());
                for record in LiveResourceRecord::ALL {
                    assert!(
                        member.standing(*record).is_some(),
                        "{} does not record {}",
                        case.case().name(),
                        record.name(),
                    );
                }
            }
        }
    }

    #[test]
    fn the_typed_non_claims_are_where_the_blockers_put_them() {
        // The three dimensions no measurement here can carry a figure
        // for, and one that only a private measurement cannot. Each is
        // asserted as its own typed value rather than as an absence, so
        // a later wave that produced a real figure would have to change
        // this test in order to record it.
        for case in study() {
            for member in case.members() {
                assert_eq!(
                    member.standing(LiveResourceRecord::ConsensusVerdict),
                    Some(DimensionStanding::NotClaimable(
                        LiveResourceNonClaim::NoTargetVerdictExists(
                            LiveInfrastructureBlocker::OwnerSighashNotComputable,
                        ),
                    )),
                );
                assert_eq!(
                    member.standing(LiveResourceRecord::RelayPolicyVerdict),
                    Some(DimensionStanding::NotClaimable(
                        LiveResourceNonClaim::NoRelayVerdictWithoutAConsensusOne,
                    )),
                );
                assert_eq!(
                    member.standing(LiveResourceRecord::ConstructionAndExecutionTime),
                    Some(DimensionStanding::NoncanonicalDiagnostic),
                );

                let proofs = member
                    .standing(LiveResourceRecord::ConfidentialProofBytes)
                    .expect("every measurement records the proof dimension");
                match member.recipe().representation() {
                    LiveTransferRepresentationPlan::Explicit => {
                        assert_eq!(proofs, DimensionStanding::AbsentFromThisShape);
                    }
                    LiveTransferRepresentationPlan::PrivateCommitted => {
                        assert_eq!(
                            proofs,
                            DimensionStanding::NotClaimable(
                                LiveResourceNonClaim::NoConfidentialProofIsSerialized,
                            ),
                        );
                    }
                }

                // The signature dimension is a slot and never a
                // measurement, so a caller summing measured figures
                // cannot pick it up by accident.
                let signature = member
                    .standing(LiveResourceRecord::OwnerSignatureWitnessBytes)
                    .expect("every measurement records the signature dimension");
                assert_eq!(signature.measured(), None);
                assert_eq!(
                    signature,
                    DimensionStanding::WitnessSlotOnly(
                        64 * member.recipe().receipt_inputs() as u64,
                    ),
                );
            }
        }
    }

    #[test]
    fn a_one_to_one_transfer_commits_no_member_leaf_and_says_so() {
        // The absence §18.3 has to be able to express. A one-input
        // transfer has no nonzero receipt position, so it spends no
        // member leaf; a figure of zero here would report a leaf of no
        // bytes rather than no leaf, and the two are different findings.
        let one_to_one = case_named(LiveResourceCase::ExplicitOneToOne);
        let member = &one_to_one.members()[0];

        assert_eq!(
            member.standing(LiveResourceRecord::MemberBytes),
            Some(DimensionStanding::AbsentFromThisShape),
        );
        assert_ne!(member.figure(LiveResourceRecord::CoordinatorBytes), Some(0));
        assert_eq!(member.spent_leaves().len(), 1);

        // A merge spends both roles, so the same dimension carries a
        // figure there — which is what makes the absence above a
        // property of the shape rather than of this module.
        let merge = case_named(LiveResourceCase::ExplicitMerge);
        assert!(
            merge.members()[0]
                .figure(LiveResourceRecord::MemberBytes)
                .is_some_and(|bytes| bytes > 0),
        );
        assert_eq!(merge.members()[0].spent_leaves().len(), 2);
    }

    #[test]
    fn the_sponsored_form_costs_more_than_the_sponsorless_one_it_matches() {
        // The comparison §18.2 pairs those two bullets to make. Both
        // members move the same value between the same owners, so the
        // difference is the sponsor region and the fee output it forces —
        // and it is read off two complete serializations rather than
        // predicted from a formula.
        let weight = |case: LiveResourceCase| {
            case_named(case).members()[0]
                .figure(LiveResourceRecord::CompleteWeight)
                .expect("the case is weighed")
        };

        let sponsorless = weight(LiveResourceCase::Sponsorless);
        let sponsored = weight(LiveResourceCase::Sponsored);
        assert!(
            sponsored > sponsorless,
            "a sponsor input and a fee output cannot be free: {sponsored} against {sponsorless}",
        );

        // Change present against change absent, over the same transfer.
        let change = case_named(LiveResourceCase::SponsorChangePresentAndAbsent);
        let present = change.members()[0]
            .figure(LiveResourceRecord::CompleteWeight)
            .expect("the present member is weighed");
        let absent = change.members()[1]
            .figure(LiveResourceRecord::CompleteWeight)
            .expect("the absent member is weighed");
        assert_eq!(
            change.members()[0].recipe().sponsor(),
            MeasuredSponsorRole::PresentWithChange,
        );
        assert_eq!(
            change.members()[1].recipe().sponsor(),
            MeasuredSponsorRole::PresentWithoutChange,
        );
        assert!(present > absent, "a change output cannot be free");
    }

    #[test]
    fn the_weight_and_the_virtual_size_agree_with_the_targets_own_identity() {
        // Two figures from one serialization, and the target's own
        // relation between them. `GetVirtualTransactionSize` divides the
        // weight by four rounding up, so a study reporting a pair that
        // does not satisfy that has read one of them from somewhere else.
        for case in study() {
            for member in case.members() {
                let weight = member
                    .figure(LiveResourceRecord::CompleteWeight)
                    .expect("every measurement weighs");
                let virtual_size = member
                    .figure(LiveResourceRecord::VirtualSize)
                    .expect("every measurement sizes");
                assert_eq!(virtual_size, weight.div_ceil(4));
                assert_ne!(weight, 0);
            }
        }
    }

    #[test]
    fn the_control_bytes_carry_the_depth_the_taptree_dimension_reports() {
        // The two leaf-derived dimensions are read from one artifact —
        // the control blocks these exact bytes carry — so they cannot
        // disagree with each other. A control block is a
        // parity-and-version byte, an x-only key, and one node per level,
        // so a transaction's control total is thirty-three per input plus
        // thirty-two per level of every input's own path.
        //
        // The paths are *not* all the same length, and that is the point
        // of stating the relation this way rather than as one width times
        // the input count: a coordinator leaf and a member leaf sit at
        // different depths in the same committed tree, so a merge carries
        // two control blocks of two different widths and an assertion
        // written over their average would be an assertion about no leaf.
        let mut uneven = false;
        for case in study() {
            for member in case.members() {
                let control = member
                    .figure(LiveResourceRecord::ControlBytes)
                    .expect("every measurement carries control bytes");
                let depth = member
                    .figure(LiveResourceRecord::TaptreeDepth)
                    .expect("every measurement carries a depth");
                let inputs = member.recipe().receipt_inputs() as u64;

                assert_ne!(control, 0);
                let path_bytes = control
                    .checked_sub(33 * inputs)
                    .expect("every control block carries its version byte and internal key");
                assert_eq!(path_bytes % 32, 0, "a merkle path is whole nodes");

                let levels = path_bytes / 32;
                assert!(
                    levels <= depth * inputs,
                    "no input's path is deeper than the deepest one reported",
                );
                assert!(levels >= depth, "the deepest path is one of the paths");
                uneven |= levels != depth * inputs;

                assert!(
                    depth <= 8,
                    "the tested deployment declares a maximum depth of eight",
                );
            }
        }
        assert!(
            uneven,
            "no measured transaction spent leaves at two different depths, \
             so this relation was never tested where it could fail",
        );
    }

    #[test]
    fn the_deepest_control_path_case_reaches_the_deepest_committed_leaf() {
        // The case is chosen by measuring the committed trees rather than
        // by naming a shape, so this asserts the chosen one is actually
        // the deepest: no other measured transaction reaches further.
        let deepest = case_named(LiveResourceCase::DeepestControlPath);
        let reached = deepest.members()[0]
            .figure(LiveResourceRecord::TaptreeDepth)
            .expect("the deepest-path case carries a depth");

        assert_eq!(
            deepest.members()[0].shape(),
            deepest_committed_shape().expect("the deepest shape resolves"),
        );
        for case in study() {
            for member in case.members() {
                let depth = member
                    .figure(LiveResourceRecord::TaptreeDepth)
                    .expect("every measurement carries a depth");
                assert!(
                    depth <= reached,
                    "{} reaches {depth}, past the deepest-path case's {reached}",
                    case.case().name(),
                );
            }
        }
    }

    #[test]
    fn the_initial_witness_is_the_three_items_the_abi_orders() {
        // §12's witness handoff: the owner signature the leaf checks,
        // then the leaf, then the control block. Read off the built
        // witnesses rather than written as three, so a change to the
        // handoff moves this figure instead of leaving it stale.
        for case in study() {
            for member in case.members() {
                assert_eq!(
                    member.figure(LiveResourceRecord::InitialWitnessItems),
                    Some(3),
                );
                // The widest element is at least a leaf script, which is
                // wider than a signature — so a study reporting sixty-four
                // here would have missed the leaf item entirely.
                let widest = member
                    .figure(LiveResourceRecord::LargestElement)
                    .expect("every measurement carries a widest element");
                assert!(widest > 64, "the widest element is at least a leaf script");
            }
        }
    }

    #[test]
    fn the_alternate_stack_is_measured_at_zero_and_the_walk_that_says_so_ran() {
        // §18.3 names both peaks, and this workspace had measured neither
        // for a live leaf before. The alternate peak turns out to be zero
        // at every measured leaf — no live program moves an item across —
        // and that is a *measurement* rather than an absence only if the
        // walk that produced it actually ran.
        //
        // So the walk is made to prove itself on the main stack, where it
        // has something to find: the precondition is one item, and a
        // reachable state deeper than that can only have come from
        // interpreting instructions. A walk that silently refused every
        // prefix would report the precondition's own depth everywhere,
        // and this refuses that reading — which is what stops the zero
        // beside it from being the "absent read as zero" §18.4 forbids.
        let mut deepest_main = 0_u64;
        for case in study() {
            for member in case.members() {
                let main = member
                    .figure(LiveResourceRecord::PeakMainStack)
                    .expect("every measurement carries a main peak");
                assert!(main >= 1, "the precondition alone is one item");
                deepest_main = deepest_main.max(main);

                assert_eq!(
                    member.figure(LiveResourceRecord::PeakAlternateStack),
                    Some(0),
                    "{} reached an alternate stack no live program builds",
                    case.case().name(),
                );

                assert!(
                    member
                        .figure(LiveResourceRecord::ValidationBudget)
                        .is_some_and(|budget| budget > 0),
                );
            }
        }
        assert!(
            deepest_main > 1,
            "no measured leaf ever grew its stack past the precondition, \
             so the walk that reported these peaks never interpreted anything",
        );
    }

    #[test]
    fn the_measured_table_is_the_one_this_study_reports() {
        // The study's headline figures, pinned. Every other test here
        // asserts a *relation* between measurements — sponsored costs
        // more, virtual size follows weight, a merge spends two leaves —
        // and a table of relations can drift wholesale while every
        // relation still holds. These are the numbers themselves, so a
        // change to a leaf, to the ABI, or to the witness handoff moves
        // this test and has to be looked at.
        let mut rows = Vec::new();
        for case in study() {
            for member in case.members() {
                rows.push(format!(
                    "{} {} weight={} vsize={} depth={} peak={} budget={}",
                    case.case().name(),
                    member.recipe().member(),
                    member
                        .figure(LiveResourceRecord::CompleteWeight)
                        .unwrap_or(0),
                    member.figure(LiveResourceRecord::VirtualSize).unwrap_or(0),
                    member.figure(LiveResourceRecord::TaptreeDepth).unwrap_or(0),
                    member
                        .figure(LiveResourceRecord::PeakMainStack)
                        .unwrap_or(0),
                    member
                        .figure(LiveResourceRecord::ValidationBudget)
                        .unwrap_or(0),
                ));
            }
        }
        assert_eq!(rows.join("\n"), "HARVEST");
    }

    /// One case's measurement, by name.
    fn case_named(case: LiveResourceCase) -> &'static CaseMeasurement {
        study()
            .iter()
            .find(|measured| measured.case() == case)
            .expect("every §18.2 case is measured")
    }
}
