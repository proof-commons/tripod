//! First-party negative evidence for the STATE announcement safety
//! matrix.
//!
//! §4.2 states what discharges a negative row whose boundary precedes
//! the target, and every clause of it removes a way of appearing to have
//! evidence: a canonical malformed typed input, the *exact owning*
//! validator, a typed refusal naming the intended class, a focused
//! positive control, a focused negative test, and a validated report
//! whose constructor is private. This module meets that policy for the
//! §16 rows whose declared boundary is a pre-target one.
//!
//! # Why the control is half the evidence
//!
//! A validator that refuses everything refuses a changed input too. The
//! argument only works because the changed input and the honest one
//! differ by exactly one thing and the *same call* accepts the second:
//! [`validate_maturity_first_party`] runs the owning entry point twice
//! per case and refuses to conclude anything if the control did not
//! pass.
//!
//! # What a discharge here is not
//!
//! Not a target verdict. §4.3 keeps three answers apart — the safe
//! constructor refused, an unsafe raw mutation exists, and the target
//! was not asked — and everything this module produces is the first of
//! them. A row discharged here still names the runtime relation it
//! anticipates, and nothing here submits anything. A pre-target class is
//! never submitted merely because an executor is available.
//!
//! # A row whose owner has no public entry is carried, never discharged
//! # by proximity
//!
//! The census covers every pre-target row of the matrix, and a row this
//! bite does not discharge carries a typed reason rather than a case
//! filed against the nearest validator that happens to refuse something.
//! Two kinds of reason are kept apart, because they mean opposite things
//! to a reader: a row whose owning entry point was located and whose
//! discharge waits on the substrate that entry point needs is work
//! outstanding, and a row whose stated change no public entry accepts is
//! work nothing can do as the tree stands. Collapsing the two would make
//! the second look like a backlog item and the first look like a
//! finding.
//!
//! The largest group of the second kind is the rows whose stated change
//! sits at the ABI's classification and role layout. That layout is
//! derived from the validated view and has no public constructor, so
//! there is no call a caller can make that offers one. Offering the
//! change instead as a raw mutation of the finalized bytes would answer
//! the row at the transaction layer it does not state, and would collapse
//! rows that differ in what they add onto the two or three classes a
//! byte comparison distinguishes.
//!
//! # Every secret here is published
//!
//! The chain signs with the published test signer material the
//! conformance layer publishes, and the deployment it runs over is the
//! one whose committed operator key that signer holds: a deployment
//! committing public fill is one no signature can be produced for, and a
//! chain over it would stop at the authorization by construction rather
//! than by any property of the chain. No interface here accepts signing
//! material from a caller.

use std::sync::LazyLock;

use linker::CandidateLinkedMaturityBundle;
use realization::{
    Cycle, Maturity, MaturityTransitionRefusal, StateMetadata, StateRepresentationNonce,
};
use tapscript::{
    CandidateStateConstructor, OperatorKey, OperatorKeyRejection, StateNonceBudget,
    operator_key_encoding_closure,
};
use target_elements::{EncodingClass, ReviewedElementsTapscriptDefinition, TargetContractVersion};
use transaction::bytes::{AssetField, AssetId, Outpoint, Txid, ValueField};
use transaction::error::TransactionRefusal;
use transaction::live_request::{RequestedForm, SponsorChangeRequest};
use transaction::operator_right::{BranchContext, OperatorRightRegistry};
use transaction::operator_signing::{
    OPERATOR_SIGHASH_TYPE_BYTE, OperatorSigningInput, OperatorSigningRefusal,
    OperatorSigningRequest, OperatorSigningResponse,
};
use transaction::script_path_signing::LiveDeployment;
use transaction::state_abi::derive_maturity_announcement_abi;
use transaction::state_construct::construct_maturity_announcement;
use transaction::state_finalize::{FinalizedMaturityAnnouncement, finalize_maturity_announcement};
use transaction::state_request::MaturityAnnouncementRequest;
use transaction::state_signing::OperatorSigningStarted;
use transaction::state_view::{
    MaturityViewStatement, PublicMaturityStateView, ValidatedMaturityStateView,
};

use crate::error::VectorError;
use crate::live_capability::OracleLiveCurve;
use crate::matrix::{EvidenceBoundary, MutationLayer};
use crate::maturity_closure::{
    MaturityDeployment, OracleStateCurve, closure_target, linked_maturity_bundle,
};
use crate::maturity_operator::{OPERATOR_HANDLE, OperatorVerifier};
use crate::maturity_safety::{
    MaturityCanonicalControl, MaturityMutationLocator, MaturitySafetyRow, MaturitySafetySection,
    rows,
};

/// The deployment every chain here is taken over.
///
/// The one whose committed operator key a published signer holds; the
/// others commit public fill, and a signature under fill is not
/// something a test can arrange.
const DEPLOYMENT: MaturityDeployment = MaturityDeployment::PublishedSignerHeld;

/// The transaction the caller states the current STATE output sits in.
const PREDECESSOR_TXID: [u8; 32] = [0x71; 32];

/// The branch identifier the caller binds the current root at.
const BRANCH: [u8; 32] = [0x9b; 32];

/// The checkpoint ordinal beside it.
const CHECKPOINT: u64 = 12;

/// The auxiliary the published signer masks its scalar with.
///
/// A stated value rather than randomness, which is what makes the
/// honest signature reproducible from the inputs this module publishes.
const AUXILIARY: [u8; 32] = [0; 32];

/// One entry point that owns a pre-target refusal of this chain.
///
/// One member per owning entry point, and the refusal type differs
/// between them: two of the four answer in the transaction crate's own
/// vocabulary and two answer in the vocabularies of the layers that own
/// the key encoding and the frozen signing selection. A census that
/// named only the crate would have had to flatten those two into a
/// wrapper that no call actually returns.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityFirstPartyValidator {
    /// Announcement construction over a validated view and a request.
    SemanticConstruction,
    /// The operator authorization over a frozen signing request.
    OperatorAuthorization,
    /// The approved operator key encoding.
    OperatorKeyEncoding,
    /// The freeze of the operator signing request.
    OperatorRequestFreeze,
}

impl MaturityFirstPartyValidator {
    /// Every owning entry point this census drives.
    pub const ALL: &'static [Self] = &[
        Self::SemanticConstruction,
        Self::OperatorAuthorization,
        Self::OperatorKeyEncoding,
        Self::OperatorRequestFreeze,
    ];

    /// The entry point's own name, as a reader would call it.
    #[must_use]
    pub const fn entry_point(self) -> &'static str {
        match self {
            Self::SemanticConstruction => "construct_maturity_announcement",
            Self::OperatorAuthorization => "OperatorSigningStarted::authorize",
            Self::OperatorKeyEncoding => "OperatorKey::new",
            Self::OperatorRequestFreeze => "OperatorSigningRequest::freeze",
        }
    }

    /// The package the entry point lives in.
    #[must_use]
    pub const fn owning_package(self) -> &'static str {
        match self {
            Self::SemanticConstruction | Self::OperatorAuthorization => "transaction",
            Self::OperatorKeyEncoding | Self::OperatorRequestFreeze => "tapscript",
        }
    }
}

/// The one change a case makes to the honest chain.
///
/// Named in the rows' own words, and applied at exactly one place: the
/// world the view states, the announced cycle, the operator response
/// set, the offered key encoding, or the frozen signing selection. A
/// change offered to a validator whose input it does not name leaves
/// that input untouched, which the discharge refuses as a change that
/// changed nothing rather than reporting a refusal about something else.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityFirstPartyChange {
    /// State a predecessor whose maturity is already announced.
    AnnouncePredecessorThatAlreadyAnnounced,
    /// State a predecessor whose maturity is complete.
    AnnouncePredecessorWhoseMaturityIsComplete,
    /// Announce the cycle one below the minimum lead.
    AnnounceOneCycleBelowTheMinimumLead,
    /// Announce the cycle one above the maximum lead.
    AnnounceOneCycleAboveTheMaximumLead,
    /// Announce the predecessor's own current cycle.
    AnnounceTheCurrentCycleFromOutsideTheLead,
    /// State a predecessor cycle whose minimum lead overflows.
    StateAPredecessorCycleWhoseMinimumLeadOverflows,
    /// State a predecessor cycle whose maximum lead overflows.
    StateAPredecessorCycleWhoseMaximumLeadOverflows,
    /// Offer no operator response at all.
    OmitTheOperatorResponse,
    /// Offer the committed key under a class the contract does not
    /// approve.
    OfferAnUnapprovedKeyEncoding,
    /// Offer the approved class with a payload of another width.
    OfferApprovedKeyBytesOfAnotherWidth,
    /// Answer under a capability revision the profile refuses.
    AnswerUnderAnotherCapabilityRevision,
    /// Answer an input the request did not freeze.
    AnswerAnInputTheRequestDidNotFreeze,
    /// Answer the frozen input twice.
    AnswerTwiceForTheFrozenInput,
    /// Answer one more position than was frozen.
    AnswerOneMorePositionThanWasFrozen,
    /// Freeze a leaf script the stated selection does not hash to.
    FreezeALeafScriptTheSelectionDoesNotHashTo,
}

/// What one owning entry point said when it refused.
///
/// One member per owning entry point's refusal type rather than one per
/// crate, because a crate is not the unit that answers: the key encoding
/// and the frozen selection are refused in two different vocabularies of
/// the same package. Carried whole rather than flattened onto the
/// transaction root, so that which layer spoke stays readable in the
/// value a report renders.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MaturityFirstPartyObservation {
    /// The transaction crate's own refusal root.
    Transaction(TransactionRefusal),
    /// The approved-key encoding's shape refusal.
    OperatorKey(OperatorKeyRejection),
    /// The operator signing boundary's own finding.
    OperatorSigning(OperatorSigningRefusal),
}

/// An owning entry point this census located and does not yet drive.
///
/// Each member names a call that exists and is public. The substrate it
/// needs — a metadata byte string, an offered static tree, a link source
/// tuple, a response record — is not the substrate this bite builds, and
/// naming the owner is what keeps the outstanding work checkable instead
/// of remembered.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum MaturityDeferredOwner {
    /// The canonical metadata decode over offered bytes.
    MetadataPatternFromBytes,
    /// The offered static subtree's validation.
    StaticSubtreeConstruction,
    /// The constructor derivation over a static subtree.
    StateConstructorDerivation,
    /// The link of a candidate bundle over its sources.
    StateCandidateLink,
    /// The check of an offered transaction against a finalized one.
    OfferedTransactionCheck,
    /// The typed response shape validation.
    ResponseShapeValidation,
}

impl MaturityDeferredOwner {
    /// The entry point's own name.
    #[must_use]
    pub const fn entry_point(self) -> &'static str {
        match self {
            Self::MetadataPatternFromBytes => "StateMetadataPattern::from_bytes",
            Self::StaticSubtreeConstruction => "StateStaticSubtree::new",
            Self::StateConstructorDerivation => "CandidateStateConstructor::derive",
            Self::StateCandidateLink => "link_state_candidate",
            Self::OfferedTransactionCheck => "FinalizedMaturityAnnouncement::check_offered",
            Self::ResponseShapeValidation => "validate_response_shape",
        }
    }
}

/// Why one pre-target row carries no case.
///
/// The first member is the census's largest group and its own finding:
/// a row whose stated change is at the ABI's layout names a change no
/// public call accepts, so either the row's stated layer is wrong or the
/// layout needs a constructor a caller can offer one to. The last member
/// is work outstanding and says whose; the middle three are properties
/// of the typed interface that no later bite changes by itself.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum MaturityCarriedReason {
    /// No entry point accepts an ABI layout of a caller's own.
    LayoutIsNotAcceptedFromACaller,
    /// The typed input has no malformed inhabitant to offer.
    TheTypedInputAdmitsNoMalformedEncoding,
    /// The value is derived from the request rather than accepted
    /// beside it.
    TheValueIsDerivedNotAccepted,
    /// The owning validator publishes no entry a caller can call.
    TheOwningValidatorHasNoPublicEntry,
    /// The owner is located and the discharge waits on its substrate.
    OwnerLocatedDischargeDeferred(MaturityDeferredOwner),
}

/// One row's discharge: its owner, its change, and the class it expects.
#[derive(Clone, Copy, Debug)]
pub struct MaturityFirstPartyDischarge {
    validator: MaturityFirstPartyValidator,
    change: MaturityFirstPartyChange,
    expected: fn(&MaturityFirstPartyObservation) -> bool,
    expected_name: &'static str,
}

impl PartialEq for MaturityFirstPartyDischarge {
    /// Compared by what a reader can check, never by function address.
    ///
    /// The expected-class predicate travels as a function pointer, whose
    /// address is not a meaningful identity; the class *name* beside it
    /// is, and it is the value a report renders anyway.
    fn eq(&self, other: &Self) -> bool {
        self.validator == other.validator
            && self.change == other.change
            && self.expected_name == other.expected_name
    }
}

impl Eq for MaturityFirstPartyDischarge {}

impl MaturityFirstPartyDischarge {
    /// Which entry point owns the refusal.
    #[must_use]
    pub const fn validator(&self) -> MaturityFirstPartyValidator {
        self.validator
    }

    /// The one change the discharge makes.
    #[must_use]
    pub const fn change(&self) -> MaturityFirstPartyChange {
        self.change
    }

    /// The refusal class the discharge expects, by name.
    #[must_use]
    pub const fn expected_class(&self) -> &'static str {
        self.expected_name
    }
}

/// What the census says about one pre-target row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityFirstPartyDisposition {
    /// The row is discharged against its owning entry point.
    Discharge(MaturityFirstPartyDischarge),
    /// The row carries a typed reason instead.
    Carried(MaturityCarriedReason),
}

/// One pre-target row of the §16 matrix, as this census answers it.
///
/// The row is identified by its table and its name together rather than
/// by a rendered string, because three names repeat across the matrix's
/// tables and a name alone is therefore not a row identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MaturityFirstPartyCase {
    section: MaturitySafetySection,
    name: &'static str,
    disposition: MaturityFirstPartyDisposition,
}

impl MaturityFirstPartyCase {
    /// The §16 table the row comes from.
    #[must_use]
    pub const fn section(&self) -> MaturitySafetySection {
        self.section
    }

    /// The row's stable name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// What the census says about the row.
    #[must_use]
    pub const fn disposition(&self) -> MaturityFirstPartyDisposition {
        self.disposition
    }

    /// The discharge, where the row has one.
    #[must_use]
    pub const fn discharge(&self) -> Option<MaturityFirstPartyDischarge> {
        match self.disposition {
            MaturityFirstPartyDisposition::Discharge(discharge) => Some(discharge),
            MaturityFirstPartyDisposition::Carried(_) => None,
        }
    }

    /// The typed reason, where the row carries one.
    #[must_use]
    pub const fn carried_reason(&self) -> Option<MaturityCarriedReason> {
        match self.disposition {
            MaturityFirstPartyDisposition::Carried(reason) => Some(reason),
            MaturityFirstPartyDisposition::Discharge(_) => None,
        }
    }
}

/// Why one offered case does not discharge its row.
///
/// Every arm is a way the offered evidence falls short of §4.2, and none
/// of them is a defect in the validator: a validator that refused the
/// control has said something true about the control, and what fails is
/// the claim that the refusal was about the change.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MaturityFirstPartyRefusal {
    /// §16 names no such row in that table.
    RowNotInTheMatrix(MaturitySafetySection, &'static str),
    /// The row's boundary needs a target execution, so a first-party
    /// refusal is not what answers it.
    RowIsNotPreTarget(MaturitySafetySection, &'static str),
    /// The announcement substrate could not be built.
    SubstrateUnavailable,
    /// The honest chain could not be carried to the owner's input.
    ///
    /// Then there is nothing to change, and a refusal of the changed
    /// input would say nothing about the builder.
    ScenarioNotConstructible,
    /// The change left the offered input unchanged.
    ///
    /// The whole argument is that the two inputs differ by exactly one
    /// thing; two identical inputs differ by nothing, and the validator
    /// would have to answer them the same way.
    ChangeChangedNothing,
    /// The validator accepted the changed input.
    ChangedInputWasAccepted,
    /// The validator refused, naming a class other than the row's.
    RefusalNamesAnotherClass(MaturityFirstPartyObservation),
    /// The validator refused the control too.
    ///
    /// Then the refusal is not attributable to the change: the input was
    /// unacceptable before it was changed.
    ControlWasRefused(MaturityFirstPartyObservation),
    /// The row carries a typed reason and no owning entry to run.
    RowCarriedWithoutAPublicOwner(MaturityCarriedReason),
}

impl From<VectorError> for MaturityFirstPartyRefusal {
    fn from(_: VectorError) -> Self {
        Self::SubstrateUnavailable
    }
}

/// One pre-target negative row, discharged.
///
/// # What holding one of these establishes
///
/// That the exact entry point the row's boundary belongs to was run
/// twice — once on an input changed in one stated place and once on the
/// honest input that change is a single difference of — that it refused
/// the first naming the row's own class, and that it accepted the
/// second. All of that is recomputed by
/// [`validate_maturity_first_party`], which is the only constructor;
/// every field here is a conclusion of that run and none of them is a
/// value a caller offered. The control the row names is recorded only
/// after that control was accepted, so a reader checks that the control
/// passed rather than assuming it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedMaturityFirstPartyEvidence {
    section: MaturitySafetySection,
    name: &'static str,
    validator: MaturityFirstPartyValidator,
    change: MaturityFirstPartyChange,
    control: MaturityCanonicalControl,
    observed: MaturityFirstPartyObservation,
}

impl ValidatedMaturityFirstPartyEvidence {
    /// The §16 table the discharged row comes from.
    #[must_use]
    pub const fn section(&self) -> MaturitySafetySection {
        self.section
    }

    /// The row this evidence answers.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Which entry point produced the refusal.
    #[must_use]
    pub const fn validator(&self) -> MaturityFirstPartyValidator {
        self.validator
    }

    /// The one change the discharge made.
    #[must_use]
    pub const fn change(&self) -> MaturityFirstPartyChange {
        self.change
    }

    /// The accepted control this discharge departed from.
    #[must_use]
    pub const fn control(&self) -> MaturityCanonicalControl {
        self.control
    }

    /// The refusal the owning entry point raised.
    #[must_use]
    pub const fn observed(&self) -> &MaturityFirstPartyObservation {
        &self.observed
    }
}

// --- The substrate -------------------------------------------------------

/// The predecessor world one run of the chain is taken over.
///
/// The nonce and the program are derived consequences of the metadata
/// rather than statements beside it: they are recomputed from the
/// metadata by the same search the link itself ran, so a world that
/// differs in one semantic field is one change and not three.
#[derive(Clone, Debug, Eq, PartialEq)]
struct MaturityWorld {
    metadata: StateMetadata,
    nonce: StateRepresentationNonce,
    program: Vec<u8>,
    announced: Cycle,
}

/// The reviewed contract, the linked bundle, the honest world and the
/// window that bundle's deployment fixes.
struct MaturitySubstrate {
    target: ReviewedElementsTapscriptDefinition,
    bundle: CandidateLinkedMaturityBundle,
    honest: MaturityWorld,
    earliest: Cycle,
    latest: Cycle,
    maximum: Cycle,
}

/// Build the substrate once, through the real curve.
fn build_substrate() -> Option<MaturitySubstrate> {
    let target = closure_target().ok()?;
    let bundle = linked_maturity_bundle(DEPLOYMENT).ok()?;
    let retained = bundle.instances().first()?;
    let bounds = bundle.deployment().lead_bounds().bounds();
    let metadata = retained.metadata().semantic;
    let (earliest, latest) = bounds.window(metadata.cycle).ok()?;
    let announced = Cycle::new(earliest.get().checked_add(1)?);
    if announced > latest {
        return None;
    }
    let honest = MaturityWorld {
        metadata,
        nonce: retained.metadata().representation,
        program: retained.constructor().output_program(),
        announced,
    };
    Some(MaturitySubstrate {
        target,
        bundle,
        honest,
        earliest,
        latest,
        maximum: bounds.maximum(),
    })
}

/// The substrate, linked once and shared.
fn substrate() -> Result<&'static MaturitySubstrate, MaturityFirstPartyRefusal> {
    static SUBSTRATE: LazyLock<Option<MaturitySubstrate>> = LazyLock::new(build_substrate);
    SUBSTRATE
        .as_ref()
        .ok_or(MaturityFirstPartyRefusal::SubstrateUnavailable)
}

/// The world a changed predecessor metadata produces, nonce and program
/// recomputed from it by the same search the link ran.
fn world_over(
    substrate: &MaturitySubstrate,
    metadata: StateMetadata,
) -> Result<MaturityWorld, MaturityFirstPartyRefusal> {
    let derived = CandidateStateConstructor::derive(
        &substrate.target,
        &metadata,
        substrate.bundle.static_subtree(),
        substrate.bundle.policy().internal_key(),
        StateNonceBudget::default(),
        &OracleStateCurve,
    )
    .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    Ok(MaturityWorld {
        metadata,
        nonce: derived.nonce(),
        program: derived.output_program(),
        announced: substrate.honest.announced,
    })
}

/// The world one change asks the chain to be run over.
///
/// A change this validator's input does not name returns the honest
/// world unchanged, which the discharge refuses as a change that changed
/// nothing.
fn changed_world(
    substrate: &MaturitySubstrate,
    change: MaturityFirstPartyChange,
) -> Result<MaturityWorld, MaturityFirstPartyRefusal> {
    let honest = &substrate.honest;
    let mut metadata = honest.metadata;
    match change {
        MaturityFirstPartyChange::AnnouncePredecessorThatAlreadyAnnounced => {
            metadata.maturity = Maturity::Announced {
                cycle: honest.announced,
            };
        }
        MaturityFirstPartyChange::AnnouncePredecessorWhoseMaturityIsComplete => {
            metadata.maturity = Maturity::Complete;
        }
        MaturityFirstPartyChange::StateAPredecessorCycleWhoseMinimumLeadOverflows => {
            metadata.cycle = Cycle::new(u64::MAX);
        }
        MaturityFirstPartyChange::StateAPredecessorCycleWhoseMaximumLeadOverflows => {
            let cycle = u64::MAX
                .saturating_sub(substrate.maximum.get())
                .saturating_add(1);
            metadata.cycle = Cycle::new(cycle);
        }
        MaturityFirstPartyChange::AnnounceOneCycleBelowTheMinimumLead => {
            let announced = Cycle::new(substrate.earliest.get().saturating_sub(1));
            return Ok(MaturityWorld {
                announced,
                ..honest.clone()
            });
        }
        MaturityFirstPartyChange::AnnounceOneCycleAboveTheMaximumLead => {
            let announced = Cycle::new(substrate.latest.get().saturating_add(1));
            return Ok(MaturityWorld {
                announced,
                ..honest.clone()
            });
        }
        MaturityFirstPartyChange::AnnounceTheCurrentCycleFromOutsideTheLead => {
            return Ok(MaturityWorld {
                announced: honest.metadata.cycle,
                ..honest.clone()
            });
        }
        _ => return Ok(honest.clone()),
    }
    world_over(substrate, metadata)
}

/// The seven statements of a view over one world.
fn statements(
    substrate: &MaturitySubstrate,
    world: &MaturityWorld,
) -> Result<Vec<MaturityViewStatement>, MaturityFirstPartyRefusal> {
    let unavailable = || MaturityFirstPartyRefusal::SubstrateUnavailable;
    let parameters = DEPLOYMENT.parameters().map_err(|_| unavailable())?;
    let outpoint =
        Outpoint::new(Txid::from_internal(PREDECESSOR_TXID), 0).map_err(|_| unavailable())?;
    let branch = BranchContext::new(BRANCH, CHECKPOINT).map_err(|_| unavailable())?;
    Ok(vec![
        MaturityViewStatement::CurrentStateOutpoint(outpoint),
        MaturityViewStatement::AssetAndAmount(
            AssetField::Explicit(AssetId::from_internal(*parameters.singleton())),
            ValueField::Explicit(1),
        ),
        MaturityViewStatement::PredecessorMetadata(world.metadata),
        MaturityViewStatement::PredecessorRepresentationNonce(world.nonce),
        MaturityViewStatement::CurrentRootBinding(branch),
        MaturityViewStatement::PredecessorProgram(world.program.clone()),
        MaturityViewStatement::AcceptedLinkedBundle(Box::new(substrate.bundle.clone())),
    ])
}

/// The validated view over one world, with its one relation computed.
fn validated_view(
    substrate: &MaturitySubstrate,
    world: &MaturityWorld,
) -> Result<ValidatedMaturityStateView, MaturityFirstPartyRefusal> {
    PublicMaturityStateView::new(statements(substrate, world)?)
        .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?
        .validate(&substrate.target, &OracleStateCurve)
        .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)
}

/// Run construction over one world, which is the owning call itself.
fn construct_over(
    substrate: &MaturitySubstrate,
    world: &MaturityWorld,
) -> Result<Result<(), TransactionRefusal>, MaturityFirstPartyRefusal> {
    let validated = validated_view(substrate, world)?;
    let abi = derive_maturity_announcement_abi(&substrate.target, &validated)
        .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    let request = MaturityAnnouncementRequest::new(
        world.announced,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
    )
    .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    Ok(construct_maturity_announcement(
        &substrate.target,
        &abi,
        &validated,
        &request,
        &OracleStateCurve,
    )
    .map(|_| ()))
}

/// The honest announcement, finalized.
fn honest_finalized(
    substrate: &MaturitySubstrate,
) -> Result<FinalizedMaturityAnnouncement, MaturityFirstPartyRefusal> {
    let world = &substrate.honest;
    let validated = validated_view(substrate, world)?;
    let unbuildable = || MaturityFirstPartyRefusal::ScenarioNotConstructible;
    let abi = derive_maturity_announcement_abi(&substrate.target, &validated)
        .map_err(|_| unbuildable())?;
    let request = MaturityAnnouncementRequest::new(
        world.announced,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
    )
    .map_err(|_| unbuildable())?;
    let construction = construct_maturity_announcement(
        &substrate.target,
        &abi,
        &validated,
        &request,
        &OracleStateCurve,
    )
    .map_err(|_| unbuildable())?;
    Ok(finalize_maturity_announcement(construction))
}

/// One frozen operator request over the honest candidate.
fn open_signing<'finalized>(
    substrate: &MaturitySubstrate,
    finalized: &'finalized FinalizedMaturityAnnouncement,
) -> Result<OperatorSigningStarted<'finalized>, MaturityFirstPartyRefusal> {
    OperatorSigningStarted::open(
        finalized,
        &substrate.target,
        &OracleLiveCurve::new(substrate.target.clone()),
    )
    .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)
}

/// The well-formed answer one frozen request admits.
///
/// Every field is read off the request itself, so a change below alters
/// exactly the one term it means to alter.
fn honest_response(
    state: &OperatorSigningStarted<'_>,
) -> Result<OperatorSigningResponse, MaturityFirstPartyRefusal> {
    let bound = state.request().binding();
    let signature = OPERATOR_HANDLE
        .material()
        .map_err(|_| MaturityFirstPartyRefusal::SubstrateUnavailable)?
        .sign(state.request().message().with_vector_grown(), &AUXILIARY)
        .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?
        .to_vec();
    Ok(OperatorSigningResponse::new(
        state.request().input_index(),
        signature,
        OPERATOR_SIGHASH_TYPE_BYTE,
        state.protected_bytes().to_vec(),
        bound.key().clone(),
        bound.deployment().clone(),
        bound.capability_revision(),
    ))
}

/// The response set one change offers.
fn changed_responses(
    state: &OperatorSigningStarted<'_>,
    honest: &OperatorSigningResponse,
    change: MaturityFirstPartyChange,
) -> Vec<OperatorSigningResponse> {
    let frozen = state.request().input_index();
    let bound = state.request().binding();
    let elsewhere = |response: &OperatorSigningResponse| {
        OperatorSigningResponse::new(
            frozen.saturating_add(1),
            response.signature().to_vec(),
            OPERATOR_SIGHASH_TYPE_BYTE,
            state.protected_bytes().to_vec(),
            bound.key().clone(),
            bound.deployment().clone(),
            bound.capability_revision(),
        )
    };
    match change {
        MaturityFirstPartyChange::OmitTheOperatorResponse => Vec::new(),
        MaturityFirstPartyChange::AnswerAnInputTheRequestDidNotFreeze => vec![elsewhere(honest)],
        MaturityFirstPartyChange::AnswerTwiceForTheFrozenInput => {
            vec![honest.clone(), honest.clone()]
        }
        MaturityFirstPartyChange::AnswerOneMorePositionThanWasFrozen => {
            vec![honest.clone(), elsewhere(honest)]
        }
        MaturityFirstPartyChange::AnswerUnderAnotherCapabilityRevision => {
            vec![OperatorSigningResponse::new(
                frozen,
                honest.signature().to_vec(),
                OPERATOR_SIGHASH_TYPE_BYTE,
                state.protected_bytes().to_vec(),
                bound.key().clone(),
                bound.deployment().clone(),
                TargetContractVersion::V1,
            )]
        }
        _ => vec![honest.clone()],
    }
}

// --- The four discharges -------------------------------------------------

/// Construction over the honest world, then over the changed one.
fn discharge_construction(
    substrate: &MaturitySubstrate,
    change: MaturityFirstPartyChange,
) -> Result<MaturityFirstPartyObservation, MaturityFirstPartyRefusal> {
    let changed = changed_world(substrate, change)?;
    if changed == substrate.honest {
        return Err(MaturityFirstPartyRefusal::ChangeChangedNothing);
    }
    construct_over(substrate, &substrate.honest)?.map_err(|refusal| {
        MaturityFirstPartyRefusal::ControlWasRefused(MaturityFirstPartyObservation::Transaction(
            refusal,
        ))
    })?;
    let refusal = construct_over(substrate, &changed)?
        .err()
        .ok_or(MaturityFirstPartyRefusal::ChangedInputWasAccepted)?;
    Ok(MaturityFirstPartyObservation::Transaction(refusal))
}

/// Authorization over the honest response set, then over the changed one.
fn discharge_authorization(
    substrate: &MaturitySubstrate,
    change: MaturityFirstPartyChange,
) -> Result<MaturityFirstPartyObservation, MaturityFirstPartyRefusal> {
    let finalized = honest_finalized(substrate)?;
    let opened = open_signing(substrate, &finalized)?;
    let honest = honest_response(&opened)?;
    let changed = changed_responses(&opened, &honest, change);
    let control = vec![honest];
    if changed == control {
        return Err(MaturityFirstPartyRefusal::ChangeChangedNothing);
    }
    authorize_once(substrate, &finalized, control)?.map_err(|refusal| {
        MaturityFirstPartyRefusal::ControlWasRefused(MaturityFirstPartyObservation::Transaction(
            refusal,
        ))
    })?;
    let refusal = authorize_once(substrate, &finalized, changed)?
        .err()
        .ok_or(MaturityFirstPartyRefusal::ChangedInputWasAccepted)?;
    Ok(MaturityFirstPartyObservation::Transaction(refusal))
}

/// One authorization over a fresh frozen request and a fresh registry.
///
/// Fresh on both sides because the call consumes the started state and
/// the registry records what it observed: a second run over a spent
/// state or a used token would be answering a different question.
fn authorize_once(
    substrate: &MaturitySubstrate,
    finalized: &FinalizedMaturityAnnouncement,
    responses: Vec<OperatorSigningResponse>,
) -> Result<Result<(), TransactionRefusal>, MaturityFirstPartyRefusal> {
    let state = open_signing(substrate, finalized)?;
    let mut registry = OperatorRightRegistry::default();
    let right = registry
        .issue(state.construction_right_scope(), state.protected_bytes())
        .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    Ok(
        match state.authorize(&mut registry, right, responses, &OperatorVerifier) {
            Ok(_) => Ok(()),
            Err(failure) => Err(failure.refusal),
        },
    )
}

/// The approved key encoding over the honest bytes, then the changed
/// offer.
fn discharge_key_encoding(
    substrate: &MaturitySubstrate,
    change: MaturityFirstPartyChange,
) -> Result<MaturityFirstPartyObservation, MaturityFirstPartyRefusal> {
    let closure = operator_key_encoding_closure(substrate.target.definition().authorization());
    let committed = substrate.bundle.deployment().operator().key();
    let honest = (closure.approved(), committed.bytes().to_vec());
    let changed = match change {
        MaturityFirstPartyChange::OfferAnUnapprovedKeyEncoding => {
            (EncodingClass::CompressedPublicKey, honest.1.clone())
        }
        MaturityFirstPartyChange::OfferApprovedKeyBytesOfAnotherWidth => {
            let mut bytes = honest.1.clone();
            bytes.pop();
            (honest.0, bytes)
        }
        _ => honest.clone(),
    };
    if changed == honest {
        return Err(MaturityFirstPartyRefusal::ChangeChangedNothing);
    }
    OperatorKey::new(&closure, honest.0, honest.1).map_err(|rejection| {
        MaturityFirstPartyRefusal::ControlWasRefused(MaturityFirstPartyObservation::OperatorKey(
            rejection,
        ))
    })?;
    let rejection = OperatorKey::new(&closure, changed.0, changed.1)
        .err()
        .ok_or(MaturityFirstPartyRefusal::ChangedInputWasAccepted)?;
    Ok(MaturityFirstPartyObservation::OperatorKey(rejection))
}

/// The freeze over the honest selection, then over the changed one.
fn discharge_request_freeze(
    substrate: &MaturitySubstrate,
    change: MaturityFirstPartyChange,
) -> Result<MaturityFirstPartyObservation, MaturityFirstPartyRefusal> {
    let finalized = honest_finalized(substrate)?;
    let leaf = finalized.executing_leaf(&substrate.target);
    let honest = leaf.leaf_script().to_vec();
    let changed = match change {
        MaturityFirstPartyChange::FreezeALeafScriptTheSelectionDoesNotHashTo => {
            let mut script = honest.clone();
            script.pop();
            script
        }
        _ => honest.clone(),
    };
    if changed == honest {
        return Err(MaturityFirstPartyRefusal::ChangeChangedNothing);
    }
    freeze_once(substrate, &finalized, honest).map_err(|refusal| {
        MaturityFirstPartyRefusal::ControlWasRefused(
            MaturityFirstPartyObservation::OperatorSigning(refusal),
        )
    })?;
    let refusal = freeze_once(substrate, &finalized, changed)
        .err()
        .ok_or(MaturityFirstPartyRefusal::ChangedInputWasAccepted)?;
    Ok(MaturityFirstPartyObservation::OperatorSigning(refusal))
}

/// One freeze over one offered leaf script.
fn freeze_once(
    substrate: &MaturitySubstrate,
    finalized: &FinalizedMaturityAnnouncement,
    leaf_script: Vec<u8>,
) -> Result<(), OperatorSigningRefusal> {
    let leaf = finalized.executing_leaf(&substrate.target);
    let spent = finalized.spent_output();
    let binding = substrate.bundle.deployment().operator();
    // The binding retains the printed identity and the message uses
    // internal order, so the seed offered here is the binding's genesis
    // reversed, exactly as the chain's own freeze offers it.
    let mut genesis = *binding.deployment().genesis_id();
    genesis.reverse();
    let input = OperatorSigningInput::new(
        u32::from(spent.position()),
        *leaf.tapleaf_hash(),
        leaf.leaf_version(),
        leaf_script,
        leaf.control_block().to_vec(),
    );
    OperatorSigningRequest::freeze(
        &substrate.target,
        binding,
        finalized.protected().clone(),
        vec![spent.census_entry()],
        LiveDeployment::new(genesis),
        input,
        &OracleLiveCurve::new(substrate.target.clone()),
    )
    .map(|_| ())
}

// --- The expected classes ------------------------------------------------

/// The transition refusal one semantic-request row expects.
fn transition_is(
    observation: &MaturityFirstPartyObservation,
    expected: MaturityTransitionRefusal,
) -> bool {
    matches!(
        observation,
        MaturityFirstPartyObservation::Transaction(
            TransactionRefusal::MaturitySuccessorTransitionRefused { refusal },
        ) if *refusal == expected
    )
}

/// `PredecessorAlreadyAnnounced`, spelled once.
fn already_announced(observation: &MaturityFirstPartyObservation) -> bool {
    transition_is(
        observation,
        MaturityTransitionRefusal::PredecessorAlreadyAnnounced,
    )
}

/// `PredecessorMaturityComplete`, spelled once.
fn maturity_complete(observation: &MaturityFirstPartyObservation) -> bool {
    transition_is(
        observation,
        MaturityTransitionRefusal::PredecessorMaturityComplete,
    )
}

/// `AnnouncementBelowMinimum`, spelled once.
fn below_minimum(observation: &MaturityFirstPartyObservation) -> bool {
    transition_is(
        observation,
        MaturityTransitionRefusal::AnnouncementBelowMinimum,
    )
}

/// `AnnouncementAboveMaximum`, spelled once.
fn above_maximum(observation: &MaturityFirstPartyObservation) -> bool {
    transition_is(
        observation,
        MaturityTransitionRefusal::AnnouncementAboveMaximum,
    )
}

/// `CycleArithmeticOverflow`, spelled once.
fn cycle_overflow(observation: &MaturityFirstPartyObservation) -> bool {
    transition_is(
        observation,
        MaturityTransitionRefusal::CycleArithmeticOverflow,
    )
}

/// The operator boundary's own finding inside an authorization refusal.
fn operator_finding(
    observation: &MaturityFirstPartyObservation,
) -> Option<&OperatorSigningRefusal> {
    match observation {
        MaturityFirstPartyObservation::Transaction(
            TransactionRefusal::MaturityOperatorAuthorizationRefused { refusal },
        ) => Some(refusal),
        MaturityFirstPartyObservation::OperatorSigning(refusal) => Some(refusal),
        _ => None,
    }
}

/// `MissingResponse`, spelled once.
fn missing_response(observation: &MaturityFirstPartyObservation) -> bool {
    matches!(
        operator_finding(observation),
        Some(OperatorSigningRefusal::MissingResponse { .. })
    )
}

/// `DuplicateResponse`, spelled once.
fn duplicate_response(observation: &MaturityFirstPartyObservation) -> bool {
    matches!(
        operator_finding(observation),
        Some(OperatorSigningRefusal::DuplicateResponse { .. })
    )
}

/// `UnexpectedResponse`, spelled once.
fn unexpected_response(observation: &MaturityFirstPartyObservation) -> bool {
    matches!(
        operator_finding(observation),
        Some(OperatorSigningRefusal::UnexpectedResponse { .. })
    )
}

/// `WrongInput`, spelled once.
fn wrong_input(observation: &MaturityFirstPartyObservation) -> bool {
    matches!(
        operator_finding(observation),
        Some(OperatorSigningRefusal::WrongInput { .. })
    )
}

/// `WrongProfile`, spelled once.
fn wrong_profile(observation: &MaturityFirstPartyObservation) -> bool {
    matches!(
        operator_finding(observation),
        Some(OperatorSigningRefusal::WrongProfile(_))
    )
}

/// `WrongLeaf`, spelled once.
fn wrong_leaf(observation: &MaturityFirstPartyObservation) -> bool {
    matches!(
        operator_finding(observation),
        Some(OperatorSigningRefusal::WrongLeaf { .. })
    )
}

/// `AlternateEncodingOfApprovedKey`, spelled once.
const fn alternate_key_encoding(observation: &MaturityFirstPartyObservation) -> bool {
    matches!(
        observation,
        MaturityFirstPartyObservation::OperatorKey(
            OperatorKeyRejection::AlternateEncodingOfApprovedKey { .. }
        )
    )
}

/// `WrongWidth`, spelled once.
const fn wrong_key_width(observation: &MaturityFirstPartyObservation) -> bool {
    matches!(
        observation,
        MaturityFirstPartyObservation::OperatorKey(OperatorKeyRejection::WrongWidth { .. })
    )
}

// --- The census ----------------------------------------------------------

/// One discharge entry, spelled once.
const fn discharge(
    validator: MaturityFirstPartyValidator,
    change: MaturityFirstPartyChange,
    expected: fn(&MaturityFirstPartyObservation) -> bool,
    expected_name: &'static str,
) -> MaturityFirstPartyDischarge {
    MaturityFirstPartyDischarge {
        validator,
        change,
        expected,
        expected_name,
    }
}

/// One census entry: the row it answers, and how.
type DischargeEntry = (
    MaturitySafetySection,
    &'static str,
    MaturityFirstPartyDischarge,
);

/// The seven maturity-window rows, owned by one call.
///
/// All seven are refused where the successor is derived, and the five
/// findings that call carries answer them: two turn on the predecessor's
/// status, three on where the announced cycle falls, and two on a window
/// whose arithmetic does not close. Two rows reaching one finding stay
/// two rows, because what separates them is the change and not the word
/// the layer answered with.
fn window_discharges() -> Vec<DischargeEntry> {
    use MaturityFirstPartyChange as Change;
    use MaturityFirstPartyValidator as V;
    use MaturitySafetySection as S;

    vec![
        (
            S::WindowFault,
            "predecessor-already-announced",
            discharge(
                V::SemanticConstruction,
                Change::AnnouncePredecessorThatAlreadyAnnounced,
                already_announced,
                "PredecessorAlreadyAnnounced",
            ),
        ),
        (
            S::WindowFault,
            "predecessor-complete",
            discharge(
                V::SemanticConstruction,
                Change::AnnouncePredecessorWhoseMaturityIsComplete,
                maturity_complete,
                "PredecessorMaturityComplete",
            ),
        ),
        (
            S::WindowFault,
            "one-below-minimum",
            discharge(
                V::SemanticConstruction,
                Change::AnnounceOneCycleBelowTheMinimumLead,
                below_minimum,
                "AnnouncementBelowMinimum",
            ),
        ),
        (
            S::WindowFault,
            "one-above-maximum",
            discharge(
                V::SemanticConstruction,
                Change::AnnounceOneCycleAboveTheMaximumLead,
                above_maximum,
                "AnnouncementAboveMaximum",
            ),
        ),
        (
            S::WindowFault,
            "equal-to-current-cycle-where-outside-the-lead",
            discharge(
                V::SemanticConstruction,
                Change::AnnounceTheCurrentCycleFromOutsideTheLead,
                below_minimum,
                "AnnouncementBelowMinimum",
            ),
        ),
        (
            S::WindowFault,
            "lower-bound-addition-overflow",
            discharge(
                V::SemanticConstruction,
                Change::StateAPredecessorCycleWhoseMinimumLeadOverflows,
                cycle_overflow,
                "CycleArithmeticOverflow",
            ),
        ),
        (
            S::WindowFault,
            "upper-bound-addition-overflow",
            discharge(
                V::SemanticConstruction,
                Change::StateAPredecessorCycleWhoseMaximumLeadOverflows,
                cycle_overflow,
                "CycleArithmeticOverflow",
            ),
        ),
    ]
}

/// The eight pre-target operator rows, across three entry points.
///
/// Separate from the window half because they are not one call's rows:
/// the response set is answered where responses are collected, the
/// approved key encoding where a key is offered at all, and the leaf
/// selection where the request is frozen. Filing all eight under the
/// collecting call would have named a validator that never sees two of
/// the changes.
fn operator_discharges() -> Vec<DischargeEntry> {
    use MaturityFirstPartyChange as Change;
    use MaturityFirstPartyValidator as V;
    use MaturitySafetySection as S;

    vec![
        (
            S::OperatorFault,
            "missing-operator-signature",
            discharge(
                V::OperatorAuthorization,
                Change::OmitTheOperatorResponse,
                missing_response,
                "MissingResponse",
            ),
        ),
        (
            S::OperatorFault,
            "unknown-key-encoding",
            discharge(
                V::OperatorKeyEncoding,
                Change::OfferAnUnapprovedKeyEncoding,
                alternate_key_encoding,
                "AlternateEncodingOfApprovedKey",
            ),
        ),
        (
            S::OperatorFault,
            "malformed-approved-key",
            discharge(
                V::OperatorKeyEncoding,
                Change::OfferApprovedKeyBytesOfAnotherWidth,
                wrong_key_width,
                "WrongWidth",
            ),
        ),
        (
            S::OperatorFault,
            "wrong-profile",
            discharge(
                V::OperatorAuthorization,
                Change::AnswerUnderAnotherCapabilityRevision,
                wrong_profile,
                "WrongProfile",
            ),
        ),
        (
            S::OperatorFault,
            "wrong-input",
            discharge(
                V::OperatorAuthorization,
                Change::AnswerAnInputTheRequestDidNotFreeze,
                wrong_input,
                "WrongInput",
            ),
        ),
        (
            S::OperatorFault,
            "wrong-leaf",
            discharge(
                V::OperatorRequestFreeze,
                Change::FreezeALeafScriptTheSelectionDoesNotHashTo,
                wrong_leaf,
                "WrongLeaf",
            ),
        ),
        (
            S::OperatorFault,
            "duplicate-response",
            discharge(
                V::OperatorAuthorization,
                Change::AnswerTwiceForTheFrozenInput,
                duplicate_response,
                "DuplicateResponse",
            ),
        ),
        (
            S::OperatorFault,
            "unexpected-response",
            discharge(
                V::OperatorAuthorization,
                Change::AnswerOneMorePositionThanWasFrozen,
                unexpected_response,
                "UnexpectedResponse",
            ),
        ),
    ]
}

/// The pre-target rows this bite discharges, by table and name.
fn discharges() -> Vec<DischargeEntry> {
    let mut entries = window_discharges();
    entries.extend(operator_discharges());
    entries
}

/// Why one pre-target row this bite does not discharge carries no case.
///
/// Total over the pre-target rows, and decided from the row's own facts
/// rather than from a list beside them: a row whose stated change sits
/// at the ABI layout is carried for that reason whichever table it comes
/// from, and a row whose owner is located is carried naming the owner.
fn carried_reason(row: &MaturitySafetyRow) -> MaturityCarriedReason {
    use MaturityCarriedReason as Why;
    use MaturityDeferredOwner as Owner;

    if matches!(row.mutation(), Some(MutationLayer::AbiLayout)) {
        return Why::LayoutIsNotAcceptedFromACaller;
    }
    match row.section() {
        MaturitySafetySection::WindowFault => {
            if row.name() == "malformed-cycle-encoding" {
                Why::TheTypedInputAdmitsNoMalformedEncoding
            } else {
                Why::TheValueIsDerivedNotAccepted
            }
        }
        MaturitySafetySection::ProtocolReportFault => {
            if matches!(
                row.locator(),
                Some(MaturityMutationLocator::ProtocolRequestField)
            ) {
                Why::TheOwningValidatorHasNoPublicEntry
            } else {
                Why::OwnerLocatedDischargeDeferred(Owner::ResponseShapeValidation)
            }
        }
        MaturitySafetySection::MetadataFault => {
            Why::OwnerLocatedDischargeDeferred(Owner::MetadataPatternFromBytes)
        }
        _ => match row.refusing_layer() {
            Some(EvidenceBoundary::ConstructorDerivationRejection) => {
                if matches!(row.locator(), Some(MaturityMutationLocator::BranchOrder)) {
                    Why::OwnerLocatedDischargeDeferred(Owner::StateConstructorDerivation)
                } else {
                    Why::OwnerLocatedDischargeDeferred(Owner::StaticSubtreeConstruction)
                }
            }
            Some(EvidenceBoundary::LinkerRejection) => {
                Why::OwnerLocatedDischargeDeferred(Owner::StateCandidateLink)
            }
            _ => Why::OwnerLocatedDischargeDeferred(Owner::OfferedTransactionCheck),
        },
    }
}

/// Whether one row's declared boundary precedes the target.
fn is_pre_target(row: &MaturitySafetyRow) -> bool {
    row.refusing_layer()
        .is_some_and(EvidenceBoundary::is_pre_target)
}

/// The §16 row one case names, if that table has it.
fn matrix_row(section: MaturitySafetySection, name: &str) -> Option<&'static MaturitySafetyRow> {
    rows()
        .iter()
        .find(|row| row.section() == section && row.name() == name)
}

/// One entry per pre-target row of the matrix, in the matrix's order.
///
/// Total by construction: the census walks the matrix rather than a list
/// of its own, so a row that starts declaring a pre-target boundary
/// appears here without anything being remembered, and a row that stops
/// declaring one disappears.
#[must_use]
pub fn maturity_first_party_cases() -> Vec<MaturityFirstPartyCase> {
    let table = discharges();
    rows()
        .iter()
        .filter(|row| is_pre_target(row))
        .map(|row| {
            let disposition = table
                .iter()
                .find(|(section, name, _)| *section == row.section() && *name == row.name())
                .map_or_else(
                    || MaturityFirstPartyDisposition::Carried(carried_reason(row)),
                    |(_, _, discharge)| MaturityFirstPartyDisposition::Discharge(*discharge),
                );
            MaturityFirstPartyCase {
                section: row.section(),
                name: row.name(),
                disposition,
            }
        })
        .collect()
}

/// Discharge one pre-target negative row, per §4.2.
///
/// The owning entry point is run twice: once on the honest input, which
/// must be accepted, and once on the input changed in the case's one
/// stated place, which must be refused naming the case's own class.
///
/// # Errors
///
/// [`MaturityFirstPartyRefusal`], naming which of §4.2's conditions the
/// offered case does not meet, or the typed reason a carried row has no
/// owning entry to run.
///
/// # Panics
///
/// Panics only where the finalized candidate's own executing-leaf read
/// panics, which a linked bundle cannot arrange: the leaf's role, its
/// control path and its committed hash are each read back out of the
/// subtree that produced them.
pub fn validate_maturity_first_party(
    case: &MaturityFirstPartyCase,
) -> Result<ValidatedMaturityFirstPartyEvidence, MaturityFirstPartyRefusal> {
    let row = matrix_row(case.section, case.name).ok_or(
        MaturityFirstPartyRefusal::RowNotInTheMatrix(case.section, case.name),
    )?;
    if !is_pre_target(row) {
        return Err(MaturityFirstPartyRefusal::RowIsNotPreTarget(
            case.section,
            case.name,
        ));
    }
    let discharge = match case.disposition {
        MaturityFirstPartyDisposition::Carried(reason) => {
            return Err(MaturityFirstPartyRefusal::RowCarriedWithoutAPublicOwner(
                reason,
            ));
        }
        MaturityFirstPartyDisposition::Discharge(discharge) => discharge,
    };
    let substrate = substrate()?;
    let observed = match discharge.validator {
        MaturityFirstPartyValidator::SemanticConstruction => {
            discharge_construction(substrate, discharge.change)
        }
        MaturityFirstPartyValidator::OperatorAuthorization => {
            discharge_authorization(substrate, discharge.change)
        }
        MaturityFirstPartyValidator::OperatorKeyEncoding => {
            discharge_key_encoding(substrate, discharge.change)
        }
        MaturityFirstPartyValidator::OperatorRequestFreeze => {
            discharge_request_freeze(substrate, discharge.change)
        }
    }?;
    if !(discharge.expected)(&observed) {
        return Err(MaturityFirstPartyRefusal::RefusalNamesAnotherClass(
            observed,
        ));
    }
    Ok(ValidatedMaturityFirstPartyEvidence {
        section: case.section,
        name: case.name,
        validator: discharge.validator,
        change: discharge.change,
        control: row.control(),
        observed,
    })
}

/// Every pre-target row this bite discharges, discharged.
///
/// The carried rows are not run and are not failures: they carry a typed
/// reason instead, and the census test is what holds the two halves to
/// the matrix's own row count.
///
/// # Errors
///
/// Whatever [`validate_maturity_first_party`] refuses, on the first
/// discharged case that does not meet §4.2.
///
/// # Panics
///
/// Everything [`validate_maturity_first_party`] panics on, for its
/// reasons.
pub fn discharge_maturity_first_party()
-> Result<Vec<ValidatedMaturityFirstPartyEvidence>, MaturityFirstPartyRefusal> {
    maturity_first_party_cases()
        .iter()
        .filter(|case| case.discharge().is_some())
        .map(validate_maturity_first_party)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        MaturityCarriedReason, MaturityFirstPartyCase, MaturityFirstPartyChange,
        MaturityFirstPartyDischarge, MaturityFirstPartyDisposition, MaturityFirstPartyRefusal,
        MaturityFirstPartyValidator, ValidatedMaturityFirstPartyEvidence, discharge,
        discharge_maturity_first_party, is_pre_target, matrix_row, maturity_first_party_cases,
        missing_response, validate_maturity_first_party,
    };
    use crate::matrix::EvidenceBoundary;
    use crate::maturity_safety::{MaturityCanonicalControl, MaturitySafetySection, rows};
    use std::collections::{BTreeMap, BTreeSet};

    /// One case built for a test, naming a row and a disposition.
    fn case(
        section: MaturitySafetySection,
        name: &'static str,
        disposition: MaturityFirstPartyDisposition,
    ) -> MaturityFirstPartyCase {
        MaturityFirstPartyCase {
            section,
            name,
            disposition,
        }
    }

    #[test]
    fn the_census_covers_the_pre_target_rows_exactly_and_nothing_else() {
        // Both directions, against the matrix rather than against a
        // remembered figure: every pre-target row has exactly one entry,
        // and every entry names a pre-target row. A row that started or
        // stopped declaring a pre-target boundary fails here rather than
        // ageing into a completeness claim nobody rechecks.
        let cases = maturity_first_party_cases();
        let expected: Vec<_> = rows()
            .iter()
            .filter(|row| is_pre_target(row))
            .map(|row| (row.section(), row.name()))
            .collect();
        let named: Vec<_> = cases
            .iter()
            .map(|case| (case.section(), case.name()))
            .collect();
        assert_eq!(named, expected);

        let distinct: BTreeSet<_> = named.iter().collect();
        assert_eq!(distinct.len(), named.len());
        for case in &cases {
            let row = matrix_row(case.section(), case.name())
                .unwrap_or_else(|| panic!("{} is not a §16 row", case.name()));
            assert!(is_pre_target(row), "{row} does not precede the target");
        }

        // The per-boundary shape of that census, recomputed. A change
        // that moved rows between boundaries would pass the totals above
        // and fail here.
        let mut by_boundary: BTreeMap<EvidenceBoundary, usize> = BTreeMap::new();
        for row in rows().iter().filter(|row| is_pre_target(row)) {
            if let Some(boundary) = row.refusing_layer() {
                *by_boundary.entry(boundary).or_default() += 1;
            }
        }
        assert_eq!(
            by_boundary,
            BTreeMap::from([
                (EvidenceBoundary::SemanticRequestRejection, 24),
                (EvidenceBoundary::ConstructorDerivationRejection, 12),
                (EvidenceBoundary::LinkerRejection, 8),
                (EvidenceBoundary::AbiConstructionRejection, 39),
            ]),
        );
        assert_eq!(cases.len(), 83);
    }

    #[test]
    fn the_two_dispositions_partition_the_census() {
        // A carried row is work stated, not work lost, and the two
        // halves are held to the matrix's own count so that neither can
        // grow by quietly shrinking the other.
        let cases = maturity_first_party_cases();
        let discharged: Vec<_> = cases
            .iter()
            .filter_map(MaturityFirstPartyCase::discharge)
            .collect();
        let carried: Vec<_> = cases
            .iter()
            .filter_map(MaturityFirstPartyCase::carried_reason)
            .collect();
        assert_eq!(discharged.len() + carried.len(), cases.len());
        assert_eq!(discharged.len(), 15);

        // The layout rows are the census's largest carried group, and
        // they are carried for a reason that is not deferral.
        let layout = carried
            .iter()
            .filter(|reason| **reason == MaturityCarriedReason::LayoutIsNotAcceptedFromACaller)
            .count();
        assert_eq!(layout, 21);

        // Every deferred row names the owner it waits on, so a reader
        // can tell outstanding work from work nothing can do.
        let owners: BTreeSet<_> = carried
            .iter()
            .filter_map(|reason| match reason {
                MaturityCarriedReason::OwnerLocatedDischargeDeferred(owner) => Some(*owner),
                _ => None,
            })
            .collect();
        assert_eq!(owners.len(), 6);
    }

    #[test]
    fn every_discharge_meets_the_policy_against_its_own_control() {
        // §4.2 performed over this bite's half of the census. Each case
        // runs its owning entry point twice and must be refused naming
        // its own class while the honest input is accepted; a case whose
        // refusal came from somewhere else fails here rather than being
        // filed as coverage.
        let cases = maturity_first_party_cases();
        let discharged: Vec<_> = cases
            .iter()
            .filter(|case| case.discharge().is_some())
            .map(|case| {
                validate_maturity_first_party(case)
                    .unwrap_or_else(|refusal| panic!("{case:?} did not meet §4.2: {refusal:?}"))
            })
            .collect();
        assert_eq!(discharged.len(), 15);
        assert_eq!(
            discharge_maturity_first_party().expect("the census discharges"),
            discharged,
        );

        // All four owning entry points were really driven, so a census
        // that quietly lost one would fail rather than report a smaller
        // matrix.
        let validators: BTreeSet<_> = discharged
            .iter()
            .map(ValidatedMaturityFirstPartyEvidence::validator)
            .collect();
        assert_eq!(
            validators,
            MaturityFirstPartyValidator::ALL
                .iter()
                .copied()
                .collect::<BTreeSet<_>>(),
        );
    }

    #[test]
    fn every_discharge_records_the_control_its_row_names() {
        // Read off the evidence rather than assumed. The control is
        // recorded only where the same entry point accepted it, so this
        // reads a computed fact and not a restatement of the row.
        for evidence in discharge_maturity_first_party().expect("the census discharges") {
            let row = matrix_row(evidence.section(), evidence.name())
                .unwrap_or_else(|| panic!("{} is not a §16 row", evidence.name()));
            assert_eq!(evidence.control(), row.control());
        }
    }

    #[test]
    fn a_change_that_changes_nothing_is_refused_as_such() {
        // A change the owner's input does not name leaves that input
        // untouched, and two identical inputs differ by nothing: the
        // validator would answer them the same way, so no refusal of the
        // second could be attributed to the change.
        let offered = case(
            MaturitySafetySection::WindowFault,
            "one-below-minimum",
            MaturityFirstPartyDisposition::Discharge(discharge(
                MaturityFirstPartyValidator::SemanticConstruction,
                MaturityFirstPartyChange::OmitTheOperatorResponse,
                missing_response,
                "MissingResponse",
            )),
        );
        assert_eq!(
            validate_maturity_first_party(&offered),
            Err(MaturityFirstPartyRefusal::ChangeChangedNothing),
        );
    }

    #[test]
    fn a_validator_that_is_not_the_rows_owner_does_not_discharge_it() {
        // §4.2's clause that a refusal from another layer does not
        // satisfy a row, performed. The key encoding really does refuse
        // the offered class, and that refusal answers a different
        // question than the row asked, so the case is refused for naming
        // another class rather than passing on a refusal it did not earn.
        let offered = case(
            MaturitySafetySection::OperatorFault,
            "missing-operator-signature",
            MaturityFirstPartyDisposition::Discharge(discharge(
                MaturityFirstPartyValidator::OperatorKeyEncoding,
                MaturityFirstPartyChange::OfferAnUnapprovedKeyEncoding,
                missing_response,
                "MissingResponse",
            )),
        );
        assert!(matches!(
            validate_maturity_first_party(&offered),
            Err(MaturityFirstPartyRefusal::RefusalNamesAnotherClass(_)),
        ));
    }

    #[test]
    fn a_carried_row_is_refused_naming_its_reason_rather_than_discharged() {
        // A carried entry has no owning entry to run, and asking for one
        // returns the reason rather than a verdict: the census answers
        // the question it can answer and says so.
        let carried = maturity_first_party_cases()
            .into_iter()
            .find(|case| case.carried_reason().is_some())
            .expect("the census carries rows");
        let reason = carried.carried_reason().expect("the case carries a reason");
        assert_eq!(
            validate_maturity_first_party(&carried),
            Err(MaturityFirstPartyRefusal::RowCarriedWithoutAPublicOwner(
                reason
            )),
        );
    }

    #[test]
    fn the_evidence_is_read_through_its_public_readers_alone() {
        // The evidence type has no public constructor, which is a
        // compile-time property no test can assert; what a test can do is
        // read every field through the readers that exist and construct
        // nothing, so that a constructor added later would be visible
        // beside a test that never needed one.
        for evidence in discharge_maturity_first_party().expect("the census discharges") {
            let discharge: MaturityFirstPartyDischarge = maturity_first_party_cases()
                .iter()
                .find(|case| case.section() == evidence.section() && case.name() == evidence.name())
                .and_then(MaturityFirstPartyCase::discharge)
                .expect("a discharged row has its discharge in the census");
            assert_eq!(evidence.validator(), discharge.validator());
            assert_eq!(evidence.change(), discharge.change());
            assert!((discharge.expected)(evidence.observed()));
            assert_ne!(
                evidence.control(),
                MaturityCanonicalControl::TheRowIsTheControl,
            );
        }
    }
}
