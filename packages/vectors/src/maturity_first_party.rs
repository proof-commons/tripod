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
//! No row is of the first kind at this tip, and the vocabulary for it
//! stays anyway. Every owner this census once deferred to is driven
//! here, so the set of deferred owners it produces is empty and the test
//! asserts that emptiness as a set rather than as a smaller count. A row
//! that declares a pre-target boundary before its owner is driven is
//! what the next table transcribed here can produce, and it would have
//! to be carried as outstanding work rather than as work nothing can do;
//! a vocabulary dropped for being briefly empty would leave whoever met
//! that row with no way to say which of the two it was.
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

use std::collections::BTreeSet;
use std::num::{NonZeroU32, NonZeroU64};
use std::sync::LazyLock;

use architecture::ARCHITECTURE;
use linker::{
    CandidateLinkedMaturityBundle, LinkRefusal, LinkedStateLeafProgram, StateConsumerCensus,
    StateLinkRefusal, StateLinkSources, StateSingletonAsset, check_linked_state_program,
    collect_state_definitions, link_state_candidate, resolve_state_census, substitute_state,
};
use realization::{
    Cycle, Maturity, MaturityTransitionRefusal, STATE_METADATA_BYTES, STATE_METADATA_DOMAIN,
    STATE_METADATA_SCHEMA, StateMetadata, StateRepresentationNonce,
};
use tapscript::upstream::StateSingletonDeclaration;
use tapscript::{
    CandidateStateConstructor, OperatorKey, OperatorKeyRejection, StackItem, StateBranchSide,
    StateConstructorRefusal, StateLeafRole, StateMetadataPattern, StateNonceBudget,
    StateStaticLeaf, StateStaticNode, StateStaticSubtree, TapscriptInstruction, TapscriptProgram,
    operator_key_encoding_closure,
};
use target_elements::{EncodingClass, ReviewedElementsTapscriptDefinition, TargetContractVersion};
use target_elements_conformance::conservation::ConservationRowId;
use target_elements_conformance::fixture::{NativeCaseGroup, NativeCaseId};
use target_elements_conformance::protocol::{
    ConservationOpening, ExecutorCapability, MinedFundingReadback, NATIVE_PROTOCOL_SCHEMA,
    NativeConservationResponse, NativeExecutionResponse, NativeOperationResponse,
    NativeResourceObservation, NativeVerdict, ObservedOutcomeLayer, OperationCaseId,
    OperationStepKind, ResponseShapeDefect, validate_response_shape,
};
use target_elements_conformance::provenance::{ExpectedExecutorProvenance, ProvenanceDefect};
use target_elements_conformance::{executor as native_executor, protocol as native_protocol};
use transaction::bytes::{
    AssetField, AssetId, Outpoint, TargetOutput, TargetTransaction, Txid, ValueField,
};
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
    MaturityDeployment, MaturitySources, OracleStateCurve, closure_target, linked_maturity_bundle,
    maturity_sources,
};
use crate::maturity_operator::{OPERATOR_HANDLE, OperatorVerifier};
use crate::maturity_safety::{
    MaturityCanonicalControl, MaturityIntendedCarrier, MaturityMutationClass,
    MaturityMutationLocator, MaturityRelationStanding, MaturitySafetyRow, MaturitySafetySection,
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
/// One member per owning entry point rather than one per refusal type or
/// one per crate, because neither of those is the unit that answers. Two
/// members answer in the transaction crate's own vocabulary and two in
/// the vocabularies of the layers that own the key encoding and the
/// frozen signing selection; four of the constructor's members share one
/// refusal root and are still four members, because what separates them
/// is the input each call accepts — offered bytes, an offered tree, an
/// offered leaf program, an offered pair of outer children — and not the
/// word each answers with. Collapsing them onto their shared root would
/// name a module where the policy asks for a call.
///
/// # Why the link is two members and the typed records are three
///
/// The same reading, carried into the two layers this census reached
/// last. A candidate bundle's link and the check of one linked program
/// are two calls taking two inputs — a tuple of seven sources, and a
/// program beside the census that claims to describe it — and the second
/// exists precisely so that a caller holding a program the linker did
/// not build can put it the same questions. The conformance layer's
/// three shape checks answer in one defect vocabulary and read three
/// different records, so which record was offered is exactly what the
/// member has to say.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityFirstPartyValidator {
    /// Compare the offered successor with the authorized one.
    AuthorizedSuccessor,
    /// Reconstruct the complete offered STATE program.
    CompleteStateView,
    /// Check unique weighted STATE declarations.
    WeightedStateDeclarations,
    /// Bind the complete STATE tree under its depth cap.
    CompleteStateBounds,
    /// Compare the observed protocol environment with its binding.
    ProtocolEnvironment,
    /// Compare a transcript's provenance with its retained expectation.
    TranscriptProvenance,
    /// Bind the signing response to the requested bytes.
    ScriptPathBinding,
    /// Constructor continuity against a retained constructor.
    ConstructorContinuity,
    /// Consumed symbols against offered definitions.
    SymbolResolution,
    /// Announcement construction over a validated view and a request.
    SemanticConstruction,
    /// The operator authorization over a frozen signing request.
    OperatorAuthorization,
    /// The approved operator key encoding.
    OperatorKeyEncoding,
    /// The freeze of the operator signing request.
    OperatorRequestFreeze,
    /// The canonical metadata decode over offered bytes.
    MetadataPatternDecode,
    /// The metadata leaf's unspendability over an offered program.
    MetadataLeafPattern,
    /// The validation of an offered static subtree.
    StaticSubtreeValidation,
    /// The fixed outer branch side over an offered pair of children.
    CanonicalBranchSide,
    /// The link of a candidate bundle over its seven sources.
    StateCandidateLink,
    /// The check of one linked program against the census that claims to
    /// describe it.
    LinkedProgramCheck,
    /// The check of an offered transaction against the finalized one.
    OfferedTransactionCheck,
    /// The shape check over a script-execution response record.
    ResponseShapeValidation,
    /// The shape check over an operation-step response record.
    OperationResponseShape,
    /// The shape check over a conservation-row response record.
    ConservationResponseShape,
}

impl MaturityFirstPartyValidator {
    /// Every owning entry point the historical census drives.
    /// Supplementary comparisons retain their entries separately.
    pub const ALL: &'static [Self] = &[
        Self::SemanticConstruction,
        Self::OperatorAuthorization,
        Self::OperatorKeyEncoding,
        Self::OperatorRequestFreeze,
        Self::MetadataPatternDecode,
        Self::MetadataLeafPattern,
        Self::StaticSubtreeValidation,
        Self::CanonicalBranchSide,
        Self::StateCandidateLink,
        Self::LinkedProgramCheck,
        Self::OfferedTransactionCheck,
        Self::ResponseShapeValidation,
        Self::OperationResponseShape,
        Self::ConservationResponseShape,
    ];

    /// The entry point's own name, as a reader would call it.
    #[must_use]
    pub const fn entry_point(self) -> &'static str {
        match self {
            Self::AuthorizedSuccessor => "OperatorAuthorizedMaturityAnnouncement::check_offered",
            Self::CompleteStateView => "PublicMaturityStateView::validate",
            Self::WeightedStateDeclarations => "TaptreeInput<StateLeafRole>::new",
            Self::CompleteStateBounds => "StateLinkedTaptree::bind",
            Self::ProtocolEnvironment => "compare_environment",
            Self::TranscriptProvenance => "ExecutionTranscript::provenance_agreement",
            Self::ScriptPathBinding => "bind_script_path_response",
            Self::ConstructorContinuity => "CandidateStateConstructor::continuity",
            Self::SymbolResolution => "resolve_state_census",
            Self::SemanticConstruction => "construct_maturity_announcement",
            Self::OperatorAuthorization => "OperatorSigningStarted::authorize",
            Self::OperatorKeyEncoding => "OperatorKey::new",
            Self::OperatorRequestFreeze => "OperatorSigningRequest::freeze",
            Self::MetadataPatternDecode => "StateMetadataPattern::from_bytes",
            Self::MetadataLeafPattern => "StateMetadataPattern::validate",
            Self::StaticSubtreeValidation => "StateStaticSubtree::new",
            Self::CanonicalBranchSide => "StateBranchSide::check",
            Self::StateCandidateLink => "link_state_candidate",
            Self::LinkedProgramCheck => "check_linked_state_program",
            Self::OfferedTransactionCheck => "FinalizedMaturityAnnouncement::check_offered",
            Self::ResponseShapeValidation => "validate_response_shape",
            Self::OperationResponseShape => "NativeOperationResponse::validate_shape",
            Self::ConservationResponseShape => "NativeConservationResponse::validate_shape",
        }
    }

    /// The package the entry point lives in.
    #[must_use]
    pub const fn owning_package(self) -> &'static str {
        match self {
            Self::SemanticConstruction
            | Self::AuthorizedSuccessor
            | Self::CompleteStateView
            | Self::OperatorAuthorization
            | Self::OfferedTransactionCheck => "transaction",
            Self::ConstructorContinuity
            | Self::OperatorKeyEncoding
            | Self::OperatorRequestFreeze
            | Self::MetadataPatternDecode
            | Self::MetadataLeafPattern
            | Self::StaticSubtreeValidation
            | Self::CanonicalBranchSide => "tapscript",
            Self::SymbolResolution
            | Self::StateCandidateLink
            | Self::LinkedProgramCheck
            | Self::WeightedStateDeclarations
            | Self::CompleteStateBounds => "linker",
            Self::ResponseShapeValidation
            | Self::ProtocolEnvironment
            | Self::TranscriptProvenance
            | Self::ScriptPathBinding
            | Self::OperationResponseShape
            | Self::ConservationResponseShape => "target-elements-conformance",
        }
    }
}

/// The one change a case makes to the honest chain.
///
/// Named in the rows' own words, and applied at exactly one place: the
/// world the view states, the announced cycle, the operator response
/// set, the offered key encoding, the frozen signing selection, one
/// range of the canonical metadata bytes, one leaf of the offered static
/// tree, the literal the metadata leaf verifies, which of the two outer
/// children is offered as the metadata child, which constructor the link
/// is given, one instruction of the offered linked program, one output
/// of the offered transaction, or one member of one offered response
/// record. A change offered to a validator whose input it does not name
/// leaves that input untouched, which the discharge refuses as a change
/// that changed nothing rather than reporting a refusal about something
/// else.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityFirstPartyChange {
    /// Change only the offered announced cycle after authorization.
    ChangeAuthorizedSuccessorCycle,
    /// Commit the static subtree alone instead of the complete outer pair.
    OmitMetadataCommitmentBranch,
    /// Add a second declaration of one role with a conflicting weight.
    AddConflictingWeightedDeclaration,
    /// Lower the positive cap below the complete tree depth.
    LowerCompleteTreeDepthCap,
    /// Change only the observed protocol schema.
    ChangeObservedSchema,
    /// Change only the observed network identifier.
    ChangeObservedNetwork,
    /// Add one reported topic absent from the independent expectation.
    AddUnexpectedProvenanceTopic,
    /// Change only the transaction bytes echoed by a signing response.
    ChangeSignatureBoundBytes,
    /// Change one static leaf against the retained constructor.
    ChangeRetainedStaticLeaf,
    /// Add a static leaf against the retained constructor.
    AddRetainedStaticLeaf,
    /// Remove the consumed metadata schema definition.
    OmitConsumedMetadataSchema,
    /// Remove the consumed minimum lead definition.
    OmitConsumedMinimumLead,
    /// Remove the consumed operator definition.
    OmitConsumedOperator,
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
    /// Omit one semantic field from the metadata encoding.
    OmitOneMetadataField,
    /// Offer one semantic field of the metadata encoding twice.
    DuplicateOneMetadataField,
    /// Present the schema revision ahead of the domain separator.
    ReorderTheMetadataFraming,
    /// State a metadata schema revision the decode does not support.
    StateAnUnsupportedMetadataSchema,
    /// State a separator that does not identify STATE metadata.
    StateAnotherMetadataDomainSeparator,
    /// State a maturity discriminant the schema does not define.
    StateAMaturityTagTheSchemaDoesNotDefine,
    /// Set a reserved metadata byte nonzero.
    SetAReservedMetadataByteNonzero,
    /// Append a byte past the encoding's fixed width.
    AppendAByteAfterTheFixedWidth,
    /// Offer a static subtree that also holds the metadata leaf.
    OfferAStaticSubtreeHoldingTheMetadataLeaf,
    /// Offer a static subtree carrying no operation leaf.
    OfferAStaticSubtreeWithoutTheOperationLeaf,
    /// Declare one leaf identity twice with one definition.
    DeclareOneLeafIdentityTwiceIdentically,
    /// Declare one leaf identity under two roles.
    DeclareOneLeafIdentityUnderTwoRoles,
    /// Offer a metadata leaf whose verification can survive.
    OfferAMetadataLeafWhoseVerificationCanSurvive,
    /// State the metadata child on the side the order does not put it
    /// on.
    StateTheMetadataChildOnTheOtherSide,
    /// Offer the link the constructor it retained from its own run.
    OfferTheConstructorTheLinkItselfRetained,
    /// Offer a constructor applied over another deployment's bundle.
    OfferAConstructorLinkedForAnotherDeployment,
    /// Restore one relocated site to the literal the record pushes.
    RestoreOneRelocatedSiteToItsPristinePush,
    /// Write one linked value at a site no relocation covers.
    WriteOneLinkedValueAtASiteNoRelocationCovers,
    /// Offer a transaction whose output the construction did not fix.
    OfferATransactionWhoseOutputTheConstructionDidNotFix,
    /// Report a final stack beside a verdict saying nothing ran.
    ReportAStackBesideAVerdictThatNothingRan,
    /// Omit the identity an accepted submission took.
    OmitTheIdentityAnAcceptedSubmissionTook,
    /// Omit the mined readback an accepted submission owes.
    OmitTheMinedReadbackAnAcceptedSubmissionOwes,
    /// Carry an accepted identity on a refused submission.
    CarryAnAcceptedIdentityOnARefusedSubmission,
    /// Carry a witness on a refused signing step.
    CarryAWitnessOnARefusedSigningStep,
    /// Carry an opening on a refused conservation row.
    CarryAnOpeningOnARefusedConservationRow,
}

/// What one owning entry point said when it refused.
///
/// One member per owning entry point's refusal type rather than one per
/// crate, because a crate is not the unit that answers: the key encoding
/// and the frozen selection are refused in two different vocabularies of
/// the same package. One member is therefore not one validator either —
/// four of the census's entry points answer in the constructor's single
/// root — so this vocabulary stays a list of the words the chain can
/// speak, and the validator beside it is what says which call spoke.
/// Carried whole rather than flattened onto the transaction root, so
/// that which layer spoke stays readable in the value a report renders.
///
/// # Why the linker speaks twice here
///
/// Because it really does answer in two roots. The link of a candidate
/// bundle answers in the shared link root, which wraps the state
/// refusal; the check of one linked program answers in the state root
/// directly. Unwrapping the first into the second would drop every
/// refusal the shared root can raise that the state root has no member
/// for, and inventing the wrapper around the second would put words in a
/// call's mouth. Both are therefore carried as the call returned them,
/// and the expected-class predicates match through the wrapper where
/// there is one.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MaturityFirstPartyObservation {
    /// Exact unsupported schema returned by environment binding.
    UnsupportedProtocolSchema {
        /// The schema on the refused observation.
        offered: u32,
    },
    /// The environment binding returned its typed mismatch.
    EnvironmentBindingMismatch,
    /// The transcript's own typed provenance defect.
    Provenance(target_elements_conformance::provenance::ProvenanceDefect),
    /// The transaction crate's own refusal root.
    Transaction(TransactionRefusal),
    /// The approved-key encoding's shape refusal.
    OperatorKey(OperatorKeyRejection),
    /// The operator signing boundary's own finding.
    OperatorSigning(OperatorSigningRefusal),
    /// The constructor boundary's own refusal root.
    Constructor(StateConstructorRefusal),
    /// The link entry's own root, which wraps the state refusal.
    ///
    /// Boxed, and the box is not a hedge about the value: this
    /// vocabulary travels inside the refusal every call in this module
    /// returns by value, and the linker's roots are the widest words any
    /// of those calls can answer with. The payload is carried whole
    /// either way.
    Link(Box<LinkRefusal>),
    /// The state link's own root, as the linked-program check returns it.
    ///
    /// Boxed for the reason the entry's root above is.
    StateLink(Box<StateLinkRefusal>),
    /// The conformance layer's own defect vocabulary for a response
    /// record's shape.
    ResponseShape(ResponseShapeDefect),
}

/// An owning entry point this census located and does not yet drive.
///
/// Each member names a call that exists and is public. The substrate it
/// needs — a metadata byte string, an offered static tree, a link source
/// tuple, a response record — is not the substrate the bite that named
/// it built, and naming the owner is what keeps the outstanding work
/// checkable instead of remembered. A member no row names is a located
/// owner whose rows have since been answered, and it stands as the
/// honest fallback for a row that might declare its boundary later; the
/// set the census actually produces, which the census test recomputes,
/// is what says which work is outstanding.
///
/// # Why the vocabulary stays where the set it produces is empty
///
/// No row names any member at this tip, and the census test asserts that
/// emptiness as a set rather than as a count, so a row that starts
/// deferring fails there rather than passing a smaller total. The type
/// stays because deferral is something this census must still be able to
/// say: the next row to declare a pre-target boundary before its owner
/// is driven needs the word, and a vocabulary retired for being briefly
/// empty would have to be reinvented by whoever met that row, with
/// nothing left to tell them that an owner located and an owner absent
/// had ever been two different answers.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum MaturityDeferredOwner {
    /// The canonical metadata decode over offered bytes.
    ///
    /// No row names it at this tip: the eight encoding rows are
    /// discharged against this call and the ninth carries a reason of
    /// its own.
    MetadataPatternFromBytes,
    /// The offered static subtree's validation.
    ///
    /// No row names it at this tip: four rows are discharged against
    /// this call and the rest carry reasons of their own.
    StaticSubtreeConstruction,
    /// The constructor derivation over a static subtree.
    ///
    /// No row names it at this tip, and no row did own it: the
    /// derivation fixes the outer branch side itself and retries a nonce
    /// that violates it, so the three rows once deferred here are
    /// answered by the side check or carry a reason of their own.
    StateConstructorDerivation,
    /// The link of a candidate bundle over its sources.
    ///
    /// No row names it at this tip: two of the eight rows once deferred
    /// here are discharged against this call, two against the check of
    /// one linked program, and the remaining four carry reasons of their
    /// own, because what each of them would change is an artifact the
    /// link produces rather than one it accepts.
    StateCandidateLink,
    /// The check of an offered transaction against a finalized one.
    ///
    /// No row names it at this tip: the one row deferred here is
    /// discharged against this call.
    OfferedTransactionCheck,
    /// The typed response shape validation.
    ///
    /// No row names it at this tip: one of the seven rows once deferred
    /// here is discharged against this call, five against the shape
    /// checks of the two other response records, and the last carries a
    /// reason of its own.
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
/// layout needs a constructor a caller can offer one to. The deferral
/// member is work outstanding and says whose; every other member is a
/// property of the typed interface that no later bite changes by itself.
///
/// # Why each of those names a mechanism rather than an absence
///
/// A row whose change no owner can see is a question about the row's
/// stated layer, and a reader can only ask it if the reason says what
/// swallowed the change: an order normalized before anything commits to
/// it, a leastness produced by a search instead of checked on an offer,
/// an owner that derives from what it is given and retains nothing to
/// compare it against, a typed input with no term for the change to land
/// on, a census the owner resolves for itself, and a bound measured over
/// what the owner built are six different answers, and a reader told
/// only that the change was invisible would have to rediscover which.
/// Collapsing them would make each look like the others, which is the
/// same defect this census keeps deferral and impossibility apart to
/// avoid.
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
    /// Leastness is a property of the search, not of an offer.
    ///
    /// The nonce a canonical encoding carries is read back as the four
    /// bytes it was written from, and nothing pre-target asks whether a
    /// smaller one would have been admitted: the one scan that decides
    /// leastness produces the nonce rather than checking one, and the
    /// call that commits a caller's nonce says in its own documentation
    /// that it claims no leastness. A row changing a nonce for a larger
    /// admitted one therefore offers an input every pre-target owner
    /// accepts.
    LeastnessIsAPropertyOfTheSearchNotOfAnOffer,
    /// The offered order is normalized before anything commits to it.
    ///
    /// A branch hashes its two children in sorted order, and a leaf's
    /// path records the sibling's hash rather than the side it sat on,
    /// so the two orders of one branch produce the same root, the same
    /// leaf hashes and the same paths. A row that states an order
    /// therefore states something the commitment cannot distinguish, and
    /// a refusal of it would have to be invented rather than observed.
    TheOfferedOrderIsNormalizedBeforeItIsCommitted,
    /// The owner derives from the offer and retains nothing to compare
    /// it against.
    ///
    /// A row whose change is that the offered object is the wrong one,
    /// rather than that it is malformed, asks for a comparison against a
    /// retained object. The owner here is handed one object and derives
    /// from it; a different well-formed object is a different derivation
    /// and not a refusal. The comparison exists further along, where a
    /// retained object is in hand, which is a different boundary than
    /// the row declares.
    TheOwnerHasNoRetainedObjectToCompareTheOfferAgainst,
    /// The typed input carries no term the row's change names.
    ///
    /// The change states two declarations of one thing disagreeing about
    /// a term, and the typed input the owner accepts has no such term to
    /// disagree about. Staging the disagreement on a term the input does
    /// have would answer a different row, and the two rows would then be
    /// separated only by a word in their names.
    TheTypedInputCarriesNoTermTheChangeNames,
    /// The census the owner reads is resolved by the owner itself.
    ///
    /// A row asking for a symbol left unresolved asks for a resolved
    /// census with a gap in it. There is no way to offer one: the type
    /// publishes readers and no constructor, the resolution that produces
    /// one refuses a consumed key nothing defines before any relocation
    /// stage is reached, and the collection feeding that resolution takes
    /// five whole typed sources with no way to omit a value. So the
    /// artifact the row would disturb is made by the call that reads it,
    /// and the refusal the row names answers a census this layer cannot
    /// hand anybody.
    TheResolvedCensusIsResolvedByTheOwnerItself,
    /// The bound is measured over what the owner built.
    ///
    /// A row stating that a candidate falls outside a bound asks for a
    /// bound check over an offered candidate. Every bound the link states
    /// is computed over an artifact the link made — the census it
    /// resolved, the graph it assembled from its own applied constructor,
    /// the resources it measured over the leaf it substituted — so a
    /// caller can change what the link is given and never what the link
    /// measures. That is a question about where the row's boundary sits,
    /// and it stays visible only while the reason says which.
    TheBoundIsMeasuredOverWhatTheOwnerBuilt,
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
    /// Supplementary keys or retained evidence differ from the recomputed ledger.
    SupplementaryLedgerMismatch,
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
    comparison: Option<MaturityRetainedComparison>,
}

impl ValidatedMaturityFirstPartyEvidence {
    /// The exact class established by a supplementary comparison, if applicable.
    pub(crate) fn supplementary_class(&self) -> Option<&'static str> {
        supplemental_discharge(self.section, self.name).map(|case| case.expected_class())
    }
    /// Exact objects retained by a supplementary comparison.
    #[must_use]
    pub const fn comparison(&self) -> Option<&MaturityRetainedComparison> {
        self.comparison.as_ref()
    }
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
    /// The sources that link was taken over, kept beside their product.
    ///
    /// The link's own entry point takes seven values and the bundle
    /// publishes four of them, so a discharge against that entry needs
    /// the sources themselves; holding them here is what keeps the
    /// honest control a second run of the same link rather than a
    /// reconstruction of one.
    sources: MaturitySources,
    bundle: CandidateLinkedMaturityBundle,
    /// A bundle linked over a deployment fixing other values.
    ///
    /// Every value a deployment fixes differs, so its applied
    /// constructor commits a leaf built for another link — which is the
    /// one thing a row about a leaf from another bundle needs and the
    /// substrate's own bundle cannot supply.
    second: CandidateLinkedMaturityBundle,
    honest: MaturityWorld,
    earliest: Cycle,
    latest: Cycle,
    maximum: Cycle,
}

/// Build the substrate once, through the real curve.
fn build_substrate() -> Option<MaturitySubstrate> {
    let target = closure_target().ok()?;
    let sources = maturity_sources(
        DEPLOYMENT,
        crate::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
    )
    .ok()?;
    let bundle = sources.link(&OracleStateCurve).ok()?;
    let second = linked_maturity_bundle(
        MaturityDeployment::Second,
        crate::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
    )
    .ok()?;
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
        sources,
        bundle,
        second,
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

// --- The constructor inputs ----------------------------------------------

/// The width the encoding writes one semantic amount at.
///
/// Read off the integer type the codec encodes with rather than copied
/// as a figure, so a widened field moves this with it.
const METADATA_AMOUNT_BYTES: usize = size_of::<u64>();

/// The width the encoding writes the schema revision at.
const METADATA_SCHEMA_BYTES: usize = size_of::<u32>();

/// The amount fields written before the maturity discriminant.
const METADATA_AMOUNTS_BEFORE_THE_TAG: usize = 5;

/// A maturity discriminant the canonical schema does not define.
///
/// The schema defines three, so the first undefined one is the fourth.
const UNDEFINED_MATURITY_TAG: u8 = 3;

/// The position the canonical metadata leaf verifies its literal at.
///
/// The leaf is three instructions — the metadata push, the literal, the
/// verification — and the literal is what decides that the leaf aborts.
const METADATA_LEAF_LITERAL: usize = 1;

/// The constructor the linked bundle retained for the predecessor.
fn retained_constructor(
    substrate: &MaturitySubstrate,
) -> Result<&CandidateStateConstructor, MaturityFirstPartyRefusal> {
    Ok(substrate
        .bundle
        .instances()
        .first()
        .ok_or(MaturityFirstPartyRefusal::SubstrateUnavailable)?
        .constructor())
}

/// The metadata bytes one change offers the decode.
///
/// Every arm rebuilds the canonical encoding with exactly one range
/// changed, and the ranges are computed from the codec's own published
/// domain, width and integer types rather than from figures written
/// beside them. A change this input does not name returns the honest
/// bytes, which the discharge refuses as a change that changed nothing.
fn changed_metadata_bytes(honest: &[u8], change: MaturityFirstPartyChange) -> Option<Vec<u8>> {
    use MaturityFirstPartyChange as Change;

    let domain = STATE_METADATA_DOMAIN.len();
    let schema_end = domain + METADATA_SCHEMA_BYTES;
    let field_end = schema_end + METADATA_AMOUNT_BYTES;
    let tag = schema_end + METADATA_AMOUNTS_BEFORE_THE_TAG * METADATA_AMOUNT_BYTES;
    let reserved = STATE_METADATA_BYTES.checked_sub(METADATA_AMOUNT_BYTES)?;
    let mut bytes = honest.to_vec();
    match change {
        Change::OmitOneMetadataField => {
            let mut trimmed = honest.get(..schema_end)?.to_vec();
            trimmed.extend_from_slice(honest.get(field_end..)?);
            bytes = trimmed;
        }
        Change::DuplicateOneMetadataField => {
            let mut doubled = honest.get(..field_end)?.to_vec();
            doubled.extend_from_slice(honest.get(schema_end..field_end)?);
            doubled.extend_from_slice(honest.get(field_end..)?);
            bytes = doubled;
        }
        Change::ReorderTheMetadataFraming => {
            let mut reordered = honest.get(domain..schema_end)?.to_vec();
            reordered.extend_from_slice(honest.get(..domain)?);
            reordered.extend_from_slice(honest.get(schema_end..)?);
            bytes = reordered;
        }
        Change::StateAnUnsupportedMetadataSchema => {
            let unsupported = STATE_METADATA_SCHEMA.checked_add(1)?;
            bytes
                .get_mut(domain..schema_end)?
                .copy_from_slice(&unsupported.to_be_bytes());
        }
        Change::StateAnotherMetadataDomainSeparator => {
            *bytes.first_mut()? ^= 1;
        }
        Change::StateAMaturityTagTheSchemaDoesNotDefine => {
            *bytes.get_mut(tag)? = UNDEFINED_MATURITY_TAG;
        }
        Change::SetAReservedMetadataByteNonzero => {
            *bytes.get_mut(reserved)? = 1;
        }
        Change::AppendAByteAfterTheFixedWidth => {
            bytes.push(0);
        }
        _ => {}
    }
    Some(bytes)
}

/// The offered tree with its operation leaf re-roled as a support leaf.
///
/// The leaf itself stays where it is and keeps its program: what changes
/// is the role the subtree reads to decide that an operation is on the
/// tree at all, which is the one thing the row states.
fn support_instead_of_the_operation(node: &StateStaticNode, spare: u32) -> StateStaticNode {
    match node {
        StateStaticNode::Leaf { identity, leaf } if leaf.role == StateLeafRole::Announcement => {
            let mut demoted = leaf.clone();
            demoted.role = StateLeafRole::Support(spare);
            StateStaticNode::Leaf {
                identity: *identity,
                leaf: demoted,
            }
        }
        StateStaticNode::Branch(left, right) => StateStaticNode::Branch(
            Box::new(support_instead_of_the_operation(left, spare)),
            Box::new(support_instead_of_the_operation(right, spare)),
        ),
        support @ StateStaticNode::Leaf { .. } => support.clone(),
    }
}

/// The static node tree one change offers the subtree validation.
///
/// A change that adds a leaf adds the branch that holds it, because a
/// tree has no other way to carry one; everything else about the offered
/// tree is the honest tree's. A change this input does not name returns
/// the honest tree.
fn changed_static_tree(
    substrate: &MaturitySubstrate,
    change: MaturityFirstPartyChange,
) -> Option<StateStaticNode> {
    use MaturityFirstPartyChange as Change;

    let subtree = substrate.bundle.static_subtree();
    let honest = subtree.tree();
    let first = subtree.leaves().first()?;
    let spare = subtree
        .leaves()
        .iter()
        .map(|entry| entry.identity)
        .max()?
        .checked_add(1)?;
    let beside = |identity: u32, leaf: StateStaticLeaf| {
        StateStaticNode::Branch(
            Box::new(honest.clone()),
            Box::new(StateStaticNode::Leaf { identity, leaf }),
        )
    };
    match change {
        Change::OfferAStaticSubtreeHoldingTheMetadataLeaf => Some(beside(
            spare,
            StateStaticLeaf {
                role: StateLeafRole::MetadataCommitment,
                program: retained_constructor(substrate).ok()?.leaf_program().clone(),
                version: substrate.target.definition().leaf_version().get(),
            },
        )),
        Change::OfferAStaticSubtreeWithoutTheOperationLeaf => {
            Some(support_instead_of_the_operation(honest, spare))
        }
        Change::DeclareOneLeafIdentityTwiceIdentically => {
            Some(beside(first.identity, first.leaf.clone()))
        }
        Change::DeclareOneLeafIdentityUnderTwoRoles => {
            let mut disagreeing = first.leaf.clone();
            // Stated against the leaf's own role rather than as a fixed
            // one, so the second declaration differs from the first
            // whatever role the first carries.
            disagreeing.role = match first.leaf.role {
                StateLeafRole::Announcement => StateLeafRole::Support(spare),
                _ => StateLeafRole::Announcement,
            };
            Some(beside(first.identity, disagreeing))
        }
        _ => Some(honest.clone()),
    }
}

/// The metadata leaf program one change offers the pattern validation.
///
/// The canonical leaf pushes the metadata, pushes an empty item and
/// verifies it, and an empty item is what makes that verification abort
/// on every admitted stack. The one change is the literal: an item with
/// a byte in it is one the verification accepts, which is the row's own
/// statement that the metadata leaf can be spent.
fn surviving_leaf_program(
    substrate: &MaturitySubstrate,
    honest: &TapscriptProgram,
) -> Option<TapscriptProgram> {
    let mut instructions = honest.instructions().to_vec();
    let literal = instructions.get_mut(METADATA_LEAF_LITERAL)?;
    *literal = TapscriptInstruction::Push(StackItem::new(&substrate.target, vec![1]).ok()?);
    TapscriptProgram::new(instructions).ok()
}

// --- The eight discharges ------------------------------------------------

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

/// The canonical decode over the honest bytes, then the changed ones.
///
/// The owner answers every noncanonical encoding with one class, so the
/// eight rows filed here are separated by the change each states and not
/// by the word the layer answered with — the same reading that keeps two
/// window rows reaching one arithmetic finding two rows. The finer
/// eight-way vocabulary exists one crate down, in the codec this call
/// wraps, and reaching for it would name a layer that is not the row's
/// owner, which §4.2 does not admit.
fn discharge_metadata_decode(
    substrate: &MaturitySubstrate,
    change: MaturityFirstPartyChange,
) -> Result<MaturityFirstPartyObservation, MaturityFirstPartyRefusal> {
    let honest = retained_constructor(substrate)?.metadata_bytes().to_vec();
    let changed = changed_metadata_bytes(&honest, change)
        .ok_or(MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    if changed == honest {
        return Err(MaturityFirstPartyRefusal::ChangeChangedNothing);
    }
    StateMetadataPattern::from_bytes(&substrate.target, &honest).map_err(|refusal| {
        MaturityFirstPartyRefusal::ControlWasRefused(MaturityFirstPartyObservation::Constructor(
            refusal,
        ))
    })?;
    let refusal = StateMetadataPattern::from_bytes(&substrate.target, &changed)
        .err()
        .ok_or(MaturityFirstPartyRefusal::ChangedInputWasAccepted)?;
    Ok(MaturityFirstPartyObservation::Constructor(refusal))
}

/// The subtree validation over the honest tree, then the changed one.
fn discharge_static_subtree(
    substrate: &MaturitySubstrate,
    change: MaturityFirstPartyChange,
) -> Result<MaturityFirstPartyObservation, MaturityFirstPartyRefusal> {
    let honest = substrate.bundle.static_subtree().tree().clone();
    let changed = changed_static_tree(substrate, change)
        .ok_or(MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    if changed == honest {
        return Err(MaturityFirstPartyRefusal::ChangeChangedNothing);
    }
    StateStaticSubtree::new(&substrate.target, Some(honest)).map_err(|refusal| {
        MaturityFirstPartyRefusal::ControlWasRefused(MaturityFirstPartyObservation::Constructor(
            refusal,
        ))
    })?;
    let refusal = StateStaticSubtree::new(&substrate.target, Some(changed))
        .err()
        .ok_or(MaturityFirstPartyRefusal::ChangedInputWasAccepted)?;
    Ok(MaturityFirstPartyObservation::Constructor(refusal))
}

/// The leaf pattern over the honest program, then the changed one.
///
/// Both runs commit the same metadata, so the only difference the
/// validation can answer to is the program: a control refused here would
/// have said the retained leaf was never the canonical one.
fn discharge_metadata_leaf(
    substrate: &MaturitySubstrate,
    change: MaturityFirstPartyChange,
) -> Result<MaturityFirstPartyObservation, MaturityFirstPartyRefusal> {
    let retained = retained_constructor(substrate)?;
    let honest = retained.leaf_program().clone();
    let changed = match change {
        MaturityFirstPartyChange::OfferAMetadataLeafWhoseVerificationCanSurvive => {
            surviving_leaf_program(substrate, &honest)
                .ok_or(MaturityFirstPartyRefusal::ScenarioNotConstructible)?
        }
        _ => honest.clone(),
    };
    if changed == honest {
        return Err(MaturityFirstPartyRefusal::ChangeChangedNothing);
    }
    let metadata = retained.encoded_metadata();
    StateMetadataPattern::validate(&substrate.target, metadata, &honest).map_err(|refusal| {
        MaturityFirstPartyRefusal::ControlWasRefused(MaturityFirstPartyObservation::Constructor(
            refusal,
        ))
    })?;
    let refusal = StateMetadataPattern::validate(&substrate.target, metadata, &changed)
        .err()
        .ok_or(MaturityFirstPartyRefusal::ChangedInputWasAccepted)?;
    Ok(MaturityFirstPartyObservation::Constructor(refusal))
}

/// The fixed outer side over the honest pair, then the changed one.
///
/// The pair is the derivation's own: the metadata leaf hash is read back
/// off the retained constructor's control recipe for the non-executing
/// role, and the static root off the subtree that recipe's sibling names.
/// The change is which of the two is offered as the metadata child,
/// which is the one thing this call reads.
fn discharge_branch_side(
    substrate: &MaturitySubstrate,
    change: MaturityFirstPartyChange,
) -> Result<MaturityFirstPartyObservation, MaturityFirstPartyRefusal> {
    let retained = retained_constructor(substrate)?;
    let recipe = retained
        .control_recipe(StateLeafRole::MetadataCommitment)
        .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    let honest = (
        recipe.executing_leaf_hash,
        *retained.static_subtree().root(),
    );
    let changed = match change {
        MaturityFirstPartyChange::StateTheMetadataChildOnTheOtherSide => (honest.1, honest.0),
        _ => honest,
    };
    if changed == honest {
        return Err(MaturityFirstPartyRefusal::ChangeChangedNothing);
    }
    StateBranchSide::MetadataLeftStaticRight
        .check(&honest.0, &honest.1)
        .map_err(|refusal| {
            MaturityFirstPartyRefusal::ControlWasRefused(
                MaturityFirstPartyObservation::Constructor(refusal),
            )
        })?;
    let refusal = StateBranchSide::MetadataLeftStaticRight
        .check(&changed.0, &changed.1)
        .err()
        .ok_or(MaturityFirstPartyRefusal::ChangedInputWasAccepted)?;
    Ok(MaturityFirstPartyObservation::Constructor(refusal))
}

// --- The link and the linked program -------------------------------------

/// The two link sources the bundle does not publish.
///
/// Rebuilt where the sources themselves took them rather than read back
/// off an artifact: the issued identifier is deployment data, and the
/// declaration is the architecture's own. A reader checks both against
/// the layer that owns them instead of against a copy kept here.
///
/// # Errors
///
/// [`MaturityFirstPartyRefusal::SubstrateUnavailable`] where the
/// deployment's values or the architecture's declaration do not resolve.
fn link_deployment_values()
-> Result<(StateSingletonAsset, StateSingletonDeclaration), MaturityFirstPartyRefusal> {
    let unavailable = || MaturityFirstPartyRefusal::SubstrateUnavailable;
    let parameters = DEPLOYMENT.parameters().map_err(|_| unavailable())?;
    let specification = ARCHITECTURE
        .asset(architecture::AssetId::Pid)
        .ok_or_else(unavailable)?;
    let declaration = StateSingletonDeclaration::from_architecture_asset(specification)
        .map_err(|_| unavailable())?;
    Ok((
        StateSingletonAsset::new(*parameters.singleton()),
        declaration,
    ))
}

/// One link over one offered constructor, every other source the honest
/// one.
fn link_once(
    substrate: &MaturitySubstrate,
    constructor: &CandidateStateConstructor,
) -> Result<Result<(), LinkRefusal>, MaturityFirstPartyRefusal> {
    let (singleton, declaration) = link_deployment_values()?;
    let sources = StateLinkSources::new(
        substrate.sources.record(),
        substrate.bundle.deployment(),
        constructor,
        &singleton,
        &declaration,
        &substrate.honest.metadata,
        &OracleStateCurve,
    );
    Ok(link_state_candidate(&substrate.target, &sources).map(|_| ()))
}

/// The constructor one change offers the link.
///
/// Both offers are constructors a link produced rather than constructors
/// built here, which is what the rows state: one is this bundle's own
/// application, whose subtree commits the substituted program instead of
/// the record's, and the other is the application of a bundle linked
/// over a deployment fixing other values. A change this input does not
/// name returns the honest constructor.
fn changed_constructor(
    substrate: &MaturitySubstrate,
    change: MaturityFirstPartyChange,
) -> Result<&CandidateStateConstructor, MaturityFirstPartyRefusal> {
    match change {
        MaturityFirstPartyChange::OfferTheConstructorTheLinkItselfRetained => {
            retained_constructor(substrate)
        }
        MaturityFirstPartyChange::OfferAConstructorLinkedForAnotherDeployment => Ok(substrate
            .second
            .instances()
            .first()
            .ok_or(MaturityFirstPartyRefusal::SubstrateUnavailable)?
            .constructor()),
        _ => Ok(substrate.sources.constructor()),
    }
}

/// The link over the honest sources, then over the changed constructor.
fn discharge_candidate_link(
    substrate: &MaturitySubstrate,
    change: MaturityFirstPartyChange,
) -> Result<MaturityFirstPartyObservation, MaturityFirstPartyRefusal> {
    let honest = substrate.sources.constructor();
    let changed = changed_constructor(substrate, change)?;
    if changed == honest {
        return Err(MaturityFirstPartyRefusal::ChangeChangedNothing);
    }
    link_once(substrate, honest)?.map_err(|refusal| {
        MaturityFirstPartyRefusal::ControlWasRefused(MaturityFirstPartyObservation::Link(Box::new(
            refusal,
        )))
    })?;
    let refusal = link_once(substrate, changed)?
        .err()
        .ok_or(MaturityFirstPartyRefusal::ChangedInputWasAccepted)?;
    Ok(MaturityFirstPartyObservation::Link(Box::new(refusal)))
}

/// The linked program and the census that claims to describe it.
///
/// Both are produced by the calls a link produces them with: the census
/// resolves the record's consumers against the deployment's definitions,
/// and the substitution rebuilds the program from it. What this module
/// offers the check is the program alone, which is exactly the input
/// that check exists to accept.
///
/// # Errors
///
/// [`MaturityFirstPartyRefusal::SubstrateUnavailable`] from the
/// deployment values, and
/// [`MaturityFirstPartyRefusal::ScenarioNotConstructible`] where the
/// collection, the resolution or the substitution refuses.
fn linked_program(
    substrate: &MaturitySubstrate,
) -> Result<LinkedStateLeafProgram, MaturityFirstPartyRefusal> {
    let (singleton, declaration) = link_deployment_values()?;
    let unbuildable = || MaturityFirstPartyRefusal::ScenarioNotConstructible;
    let record = substrate.sources.record();
    let constructor = substrate.sources.constructor();
    let definitions = collect_state_definitions(
        &substrate.target,
        substrate.bundle.deployment(),
        constructor,
        &singleton,
        &declaration,
        record.schedule(),
    )
    .map_err(|_| unbuildable())?;
    let consumers = StateConsumerCensus::from_sources(record, constructor);
    let resolved = resolve_state_census(&definitions, &consumers).map_err(|_| unbuildable())?;
    substitute_state(&substrate.target, record, &resolved).map_err(|_| unbuildable())
}

/// The program one change offers the linked-program check.
///
/// One instruction either way. Restoring a relocated site to the literal
/// the record pushes is a relocation the offered program does not carry;
/// writing a linked value at a site the census does not cover is that
/// same relocation carried once more than the census accounts for, and
/// the site is chosen by what the program holds rather than by an index
/// written here. A change this input does not name returns the honest
/// program.
fn changed_linked_program(
    linked: &LinkedStateLeafProgram,
    change: MaturityFirstPartyChange,
) -> Option<TapscriptProgram> {
    let census = linked.relocations();
    let first = census.relocations().first()?;
    let mut instructions = linked.program().instructions().to_vec();
    match change {
        MaturityFirstPartyChange::RestoreOneRelocatedSiteToItsPristinePush => {
            *instructions.get_mut(first.site())? =
                TapscriptInstruction::Push(first.pre_value().clone());
        }
        MaturityFirstPartyChange::WriteOneLinkedValueAtASiteNoRelocationCovers => {
            let covered = census.sites();
            let elsewhere = instructions
                .iter()
                .enumerate()
                .find(|(site, instruction)| {
                    !covered.contains(site)
                        && matches!(
                            instruction,
                            TapscriptInstruction::Push(item) if item != first.linked_value()
                        )
                })
                .map(|(site, _)| site)?;
            *instructions.get_mut(elsewhere)? =
                TapscriptInstruction::Push(first.linked_value().clone());
        }
        _ => {}
    }
    TapscriptProgram::new(instructions).ok()
}

/// The check over the honest linked program, then over the changed one.
fn discharge_linked_program(
    substrate: &MaturitySubstrate,
    change: MaturityFirstPartyChange,
) -> Result<MaturityFirstPartyObservation, MaturityFirstPartyRefusal> {
    let linked = linked_program(substrate)?;
    let honest = linked.program().clone();
    let changed = changed_linked_program(&linked, change)
        .ok_or(MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    if changed == honest {
        return Err(MaturityFirstPartyRefusal::ChangeChangedNothing);
    }
    let record = substrate.sources.record();
    let census = linked.relocations();
    check_linked_state_program(&substrate.target, record, &honest, census)
        .map(|_| ())
        .map_err(|refusal| {
            MaturityFirstPartyRefusal::ControlWasRefused(MaturityFirstPartyObservation::StateLink(
                Box::new(refusal),
            ))
        })?;
    let refusal = check_linked_state_program(&substrate.target, record, &changed, census)
        .err()
        .ok_or(MaturityFirstPartyRefusal::ChangedInputWasAccepted)?;
    Ok(MaturityFirstPartyObservation::StateLink(Box::new(refusal)))
}

// --- The offered transaction ---------------------------------------------

/// One offering, built through the public transaction constructor.
///
/// A rebuild rather than a clone of the finalized bytes, because what
/// the row states is a transaction a caller wrote: a control that was
/// the finalized object itself would differ from the changed offering in
/// authorship as well as in the one field.
fn rebuilt_offering(
    protected: &TargetTransaction,
    outputs: Vec<TargetOutput>,
) -> Result<TargetTransaction, MaturityFirstPartyRefusal> {
    TargetTransaction::new(
        protected.version(),
        protected.inputs().to_vec(),
        outputs,
        protected.lock_time(),
        protected.witnesses().to_vec(),
    )
    .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)
}

/// The outputs one change offers, the fixed output rewritten.
///
/// The asset, the value and the nonce are the finalized output's own, so
/// the one difference is the program the caller paid to — which is the
/// output the construction fixes and the row's own statement of what a
/// raw transaction does differently.
fn caller_written_outputs(protected: &TargetTransaction) -> Option<Vec<TargetOutput>> {
    let mut outputs = protected.outputs().to_vec();
    let fixed = protected.outputs().first()?;
    let mut program = fixed.program().to_vec();
    *program.last_mut()? ^= 1;
    *outputs.first_mut()? = TargetOutput::new(fixed.asset(), fixed.value(), fixed.nonce(), program);
    Some(outputs)
}

/// The check over the honest offering, then over the changed one.
fn discharge_offered_transaction(
    substrate: &MaturitySubstrate,
    change: MaturityFirstPartyChange,
) -> Result<MaturityFirstPartyObservation, MaturityFirstPartyRefusal> {
    let finalized = honest_finalized(substrate)?;
    let protected = finalized.protected();
    let honest = rebuilt_offering(protected, protected.outputs().to_vec())?;
    let changed = match change {
        MaturityFirstPartyChange::OfferATransactionWhoseOutputTheConstructionDidNotFix => {
            let outputs = caller_written_outputs(protected)
                .ok_or(MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
            rebuilt_offering(protected, outputs)?
        }
        _ => honest.clone(),
    };
    if changed == honest {
        return Err(MaturityFirstPartyRefusal::ChangeChangedNothing);
    }
    finalized.check_offered(&honest).map_err(|refusal| {
        MaturityFirstPartyRefusal::ControlWasRefused(MaturityFirstPartyObservation::Transaction(
            refusal,
        ))
    })?;
    let refusal = finalized
        .check_offered(&changed)
        .err()
        .ok_or(MaturityFirstPartyRefusal::ChangedInputWasAccepted)?;
    Ok(MaturityFirstPartyObservation::Transaction(refusal))
}

// --- The offered response records ----------------------------------------

/// The identity a target reports for a transaction it took.
///
/// A stated stand-in rather than a value copied from a run: the shape
/// rules read which members a record carries and never what they say, so
/// a record here states presence and claims nothing about a chain.
const ACCEPTED_IDENTITY: &str = "accepted-transaction-identity";

/// The block a readback names, on the same reading.
const MINED_BLOCK: &str = "mined-block-identity";

/// The caller's own name for the submission step these records answer.
const SUBMISSION_STEP: &str = "announcement-submission";

/// The caller's own name for the signing step.
const SIGNING_STEP: &str = "announcement-script-path-signing";

/// The conservation row these records answer.
const CONSERVATION_ROW: &str = "announcement-conservation";

/// What an opening's asset and blinding factors stand for.
///
/// The same reading as the identity above: an opening is refused here
/// for being present beside a refusal, and no rule reads the factors, so
/// a stand-in is what this record can honestly carry.
const OPENING_FIGURE: &str = "opening-stand-in";

/// The executor interface every response here is read against.
///
/// Stack reporting is advertised so that a record carrying a stack is
/// answered for the rule its row names rather than for reporting
/// something its executor said it never observes.
fn advertised_capabilities() -> BTreeSet<ExecutorCapability> {
    BTreeSet::from([ExecutorCapability::FinalStackReporting])
}

/// The well-formed response of a run that never happened.
fn honest_execution_response() -> NativeExecutionResponse {
    NativeExecutionResponse {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: NativeCaseId::new(NativeCaseGroup::ExecutionDomain, None, 1),
        verdict: NativeVerdict::InfrastructureError,
        final_stack: None,
        final_altstack: None,
        observed_failure: None,
        resources: NativeResourceObservation::default(),
    }
}

/// The shape check over the honest response, then over the changed one.
fn discharge_execution_response(
    change: MaturityFirstPartyChange,
) -> Result<MaturityFirstPartyObservation, MaturityFirstPartyRefusal> {
    let honest = honest_execution_response();
    let mut changed = honest.clone();
    if matches!(
        change,
        MaturityFirstPartyChange::ReportAStackBesideAVerdictThatNothingRan
    ) {
        changed.final_stack = Some(vec![vec![1]]);
    }
    if changed == honest {
        return Err(MaturityFirstPartyRefusal::ChangeChangedNothing);
    }
    let capabilities = advertised_capabilities();
    validate_response_shape(&honest, &capabilities).map_err(|defect| {
        MaturityFirstPartyRefusal::ControlWasRefused(MaturityFirstPartyObservation::ResponseShape(
            defect,
        ))
    })?;
    let defect = validate_response_shape(&changed, &capabilities)
        .err()
        .ok_or(MaturityFirstPartyRefusal::ChangedInputWasAccepted)?;
    Ok(MaturityFirstPartyObservation::ResponseShape(defect))
}

/// One operation-step record at one kind and one observed layer.
///
/// Every member a step may carry is absent here, and each honest record
/// below states the ones its own kind owes. A change is therefore one
/// member added or one removed, against a record whose other members
/// were never in question.
fn operation_response(
    operation: OperationStepKind,
    step: &str,
    observed_layer: ObservedOutcomeLayer,
) -> NativeOperationResponse {
    NativeOperationResponse {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: OperationCaseId {
            operation,
            step: step.to_owned(),
        },
        observed_layer,
        observed_detail: None,
        issued_asset: None,
        funded_outputs: Vec::new(),
        confidential_funded_outputs: Vec::new(),
        mined_readback: None,
        accepted_txid: None,
        sponsor_witness: Vec::new(),
        script_path_witness: Vec::new(),
        signer_public_key: None,
        signed_profile: None,
        signing_genesis: None,
        signature_bound_to: None,
        resources: NativeResourceObservation::default(),
    }
}

/// What a node reports for a transaction it has confirmed.
fn mined_readback() -> MinedFundingReadback {
    MinedFundingReadback {
        transaction_id: ACCEPTED_IDENTITY.to_owned(),
        witness_transaction_id: ACCEPTED_IDENTITY.to_owned(),
        block_hash: MINED_BLOCK.to_owned(),
        block_height: 1,
        raw_transaction: Vec::new(),
    }
}

/// The honest record one operation-response change departs from.
///
/// Three of them, because the rows are about two answers and two kinds:
/// what an accepted submission owes, what a refused one may not carry,
/// and what a refused signing step may not carry. A change this input
/// does not name departs from the accepted submission and leaves it
/// untouched.
fn honest_operation_response(change: MaturityFirstPartyChange) -> NativeOperationResponse {
    match change {
        MaturityFirstPartyChange::CarryAnAcceptedIdentityOnARefusedSubmission => {
            operation_response(
                OperationStepKind::Submit,
                SUBMISSION_STEP,
                ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
            )
        }
        MaturityFirstPartyChange::CarryAWitnessOnARefusedSigningStep => operation_response(
            OperationStepKind::SignScriptPath,
            SIGNING_STEP,
            ObservedOutcomeLayer::ScriptPathRejection,
        ),
        _ => {
            let mut accepted = operation_response(
                OperationStepKind::Submit,
                SUBMISSION_STEP,
                ObservedOutcomeLayer::Accepted,
            );
            accepted.accepted_txid = Some(ACCEPTED_IDENTITY.to_owned());
            accepted.mined_readback = Some(mined_readback());
            accepted
        }
    }
}

/// The record one change offers the operation-step shape check.
fn changed_operation_response(
    honest: &NativeOperationResponse,
    change: MaturityFirstPartyChange,
) -> NativeOperationResponse {
    let mut changed = honest.clone();
    match change {
        MaturityFirstPartyChange::OmitTheIdentityAnAcceptedSubmissionTook => {
            changed.accepted_txid = None;
        }
        MaturityFirstPartyChange::OmitTheMinedReadbackAnAcceptedSubmissionOwes => {
            changed.mined_readback = None;
        }
        MaturityFirstPartyChange::CarryAnAcceptedIdentityOnARefusedSubmission => {
            changed.accepted_txid = Some(ACCEPTED_IDENTITY.to_owned());
        }
        MaturityFirstPartyChange::CarryAWitnessOnARefusedSigningStep => {
            // One item, because what the rule reads here is that a
            // refused step carried an authorization at all; the width a
            // witness must have is checked behind an acceptance.
            changed.script_path_witness = vec![vec![1]];
        }
        _ => {}
    }
    changed
}

/// The shape check over the honest record, then over the changed one.
fn discharge_operation_response(
    change: MaturityFirstPartyChange,
) -> Result<MaturityFirstPartyObservation, MaturityFirstPartyRefusal> {
    let honest = honest_operation_response(change);
    let changed = changed_operation_response(&honest, change);
    if changed == honest {
        return Err(MaturityFirstPartyRefusal::ChangeChangedNothing);
    }
    honest.validate_shape().map_err(|defect| {
        MaturityFirstPartyRefusal::ControlWasRefused(MaturityFirstPartyObservation::ResponseShape(
            defect,
        ))
    })?;
    let defect = changed
        .validate_shape()
        .err()
        .ok_or(MaturityFirstPartyRefusal::ChangedInputWasAccepted)?;
    Ok(MaturityFirstPartyObservation::ResponseShape(defect))
}

/// The well-formed record of a conservation row the target refused.
fn honest_conservation_response() -> NativeConservationResponse {
    NativeConservationResponse {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: ConservationRowId {
            ordinal: 1,
            name: CONSERVATION_ROW.to_owned(),
        },
        observed_layer: ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
        observed_detail: None,
        transaction_bytes: None,
        observed_value_commitments: Vec::new(),
        observed_asset_commitments: Vec::new(),
        observed_openings: Vec::new(),
    }
}

/// The shape check over the honest row record, then over the changed one.
fn discharge_conservation_response(
    change: MaturityFirstPartyChange,
) -> Result<MaturityFirstPartyObservation, MaturityFirstPartyRefusal> {
    let honest = honest_conservation_response();
    let mut changed = honest.clone();
    if matches!(
        change,
        MaturityFirstPartyChange::CarryAnOpeningOnARefusedConservationRow
    ) {
        changed.observed_openings = vec![ConservationOpening {
            vout: 0,
            amount_satoshis: 1,
            asset: OPENING_FIGURE.to_owned(),
            amount_blinder: OPENING_FIGURE.to_owned(),
            asset_blinder: OPENING_FIGURE.to_owned(),
        }];
    }
    if changed == honest {
        return Err(MaturityFirstPartyRefusal::ChangeChangedNothing);
    }
    honest.validate_shape().map_err(|defect| {
        MaturityFirstPartyRefusal::ControlWasRefused(MaturityFirstPartyObservation::ResponseShape(
            defect,
        ))
    })?;
    let defect = changed
        .validate_shape()
        .err()
        .ok_or(MaturityFirstPartyRefusal::ChangedInputWasAccepted)?;
    Ok(MaturityFirstPartyObservation::ResponseShape(defect))
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

/// `MetadataEncodingRefused`, spelled once.
const fn metadata_encoding_refused(observation: &MaturityFirstPartyObservation) -> bool {
    matches!(
        observation,
        MaturityFirstPartyObservation::Constructor(
            StateConstructorRefusal::MetadataEncodingRefused
        )
    )
}

/// `MetadataLeafNotUnspendable`, spelled once.
const fn metadata_leaf_not_unspendable(observation: &MaturityFirstPartyObservation) -> bool {
    matches!(
        observation,
        MaturityFirstPartyObservation::Constructor(
            StateConstructorRefusal::MetadataLeafNotUnspendable
        )
    )
}

/// `StaticSubtreeIncomplete`, spelled once.
const fn static_subtree_incomplete(observation: &MaturityFirstPartyObservation) -> bool {
    matches!(
        observation,
        MaturityFirstPartyObservation::Constructor(
            StateConstructorRefusal::StaticSubtreeIncomplete
        )
    )
}

/// `DuplicateLeaf`, spelled once.
const fn duplicate_leaf(observation: &MaturityFirstPartyObservation) -> bool {
    matches!(
        observation,
        MaturityFirstPartyObservation::Constructor(StateConstructorRefusal::DuplicateLeaf)
    )
}

/// `ConflictingLeaf`, spelled once.
const fn conflicting_leaf(observation: &MaturityFirstPartyObservation) -> bool {
    matches!(
        observation,
        MaturityFirstPartyObservation::Constructor(StateConstructorRefusal::ConflictingLeaf)
    )
}

/// `CanonicalBranchSideNotSatisfied`, spelled once.
const fn canonical_branch_side(observation: &MaturityFirstPartyObservation) -> bool {
    matches!(
        observation,
        MaturityFirstPartyObservation::Constructor(
            StateConstructorRefusal::CanonicalBranchSideNotSatisfied
        )
    )
}

/// The link entry's own word, where the entry is what spoke.
fn link_refusal(observation: &MaturityFirstPartyObservation) -> Option<&LinkRefusal> {
    match observation {
        MaturityFirstPartyObservation::Link(refusal) => Some(refusal.as_ref()),
        _ => None,
    }
}

/// The state link's own word, where the linked-program check spoke.
fn state_link_refusal(observation: &MaturityFirstPartyObservation) -> Option<&StateLinkRefusal> {
    match observation {
        MaturityFirstPartyObservation::StateLink(refusal) => Some(refusal.as_ref()),
        _ => None,
    }
}

/// `SuppliedConstructorCommitsAnotherProgram`, spelled once.
///
/// Read through the shared root the link answers in rather than beside
/// it, because that wrapper is part of what the call said.
fn supplied_constructor_commits_another_program(
    observation: &MaturityFirstPartyObservation,
) -> bool {
    matches!(
        link_refusal(observation),
        Some(LinkRefusal::StateLink(
            StateLinkRefusal::SuppliedConstructorCommitsAnotherProgram
        ))
    )
}

/// `RelocationNotApplied`, spelled once.
fn relocation_not_applied(observation: &MaturityFirstPartyObservation) -> bool {
    matches!(
        state_link_refusal(observation),
        Some(StateLinkRefusal::RelocationNotApplied { .. })
    )
}

/// `UntrackedProgramMutation`, spelled once.
fn untracked_program_mutation(observation: &MaturityFirstPartyObservation) -> bool {
    matches!(
        state_link_refusal(observation),
        Some(StateLinkRefusal::UntrackedProgramMutation { .. })
    )
}

/// `OutputMutatedAfterSigning`, spelled once.
const fn output_mutated_after_signing(observation: &MaturityFirstPartyObservation) -> bool {
    matches!(
        observation,
        MaturityFirstPartyObservation::Transaction(
            TransactionRefusal::OutputMutatedAfterSigning { .. }
        )
    )
}

/// `InfrastructureResponseCarriesObservation`, spelled once.
const fn infrastructure_response_carries_observation(
    observation: &MaturityFirstPartyObservation,
) -> bool {
    matches!(
        observation,
        MaturityFirstPartyObservation::ResponseShape(
            ResponseShapeDefect::InfrastructureResponseCarriesObservation
        )
    )
}

/// `AcceptedOperationOmitsObservation`, spelled once.
const fn accepted_operation_omits_observation(observation: &MaturityFirstPartyObservation) -> bool {
    matches!(
        observation,
        MaturityFirstPartyObservation::ResponseShape(
            ResponseShapeDefect::AcceptedOperationOmitsObservation
        )
    )
}

/// `RefusedOperationCarriesObservation`, spelled once.
const fn refused_operation_carries_observation(
    observation: &MaturityFirstPartyObservation,
) -> bool {
    matches!(
        observation,
        MaturityFirstPartyObservation::ResponseShape(
            ResponseShapeDefect::RefusedOperationCarriesObservation
        )
    )
}

/// `RefusedConservationCarriesOpenings`, spelled once.
const fn refused_conservation_carries_openings(
    observation: &MaturityFirstPartyObservation,
) -> bool {
    matches!(
        observation,
        MaturityFirstPartyObservation::ResponseShape(
            ResponseShapeDefect::RefusedConservationCarriesOpenings
        )
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

/// The eight metadata-encoding rows, owned by one decode.
///
/// Eight rows and eight changes against one class. What a reader checks
/// here is that no two rows state the same change to the same canonical
/// bytes: the class they share is the owner's whole vocabulary for a
/// noncanonical encoding, so it separates nothing, and the change is
/// what each row is.
fn metadata_discharges() -> Vec<DischargeEntry> {
    use MaturityFirstPartyChange as Change;
    use MaturityFirstPartyValidator as V;
    use MaturitySafetySection as S;

    let refused = |change| {
        discharge(
            V::MetadataPatternDecode,
            change,
            metadata_encoding_refused,
            "MetadataEncodingRefused",
        )
    };
    vec![
        (
            S::MetadataFault,
            "omit-one-field",
            refused(Change::OmitOneMetadataField),
        ),
        (
            S::MetadataFault,
            "duplicate-one-field",
            refused(Change::DuplicateOneMetadataField),
        ),
        (
            S::MetadataFault,
            "reorder-fields",
            refused(Change::ReorderTheMetadataFraming),
        ),
        (
            S::MetadataFault,
            "unknown-schema",
            refused(Change::StateAnUnsupportedMetadataSchema),
        ),
        (
            S::MetadataFault,
            "wrong-domain-separator",
            refused(Change::StateAnotherMetadataDomainSeparator),
        ),
        (
            S::MetadataFault,
            "noncanonical-enum-tag",
            refused(Change::StateAMaturityTagTheSchemaDoesNotDefine),
        ),
        (
            S::MetadataFault,
            "nonzero-reserved-field",
            refused(Change::SetAReservedMetadataByteNonzero),
        ),
        (
            S::MetadataFault,
            "trailing-bytes",
            refused(Change::AppendAByteAfterTheFixedWidth),
        ),
    ]
}

/// The six constructor rows, across three entry points.
///
/// Not one call's rows, and not the call the rows' boundary is named
/// after: the derivation accepts a validated subtree and a semantic
/// metadata and nothing else a caller can misstate, so the offered tree
/// is answered where a tree is validated, the metadata leaf where a leaf
/// program is validated, and the outer side where that side is checked —
/// which the derivation itself only reaches through a retryable refusal
/// it answers by moving to the next nonce.
fn constructor_discharges() -> Vec<DischargeEntry> {
    use MaturityFirstPartyChange as Change;
    use MaturityFirstPartyValidator as V;
    use MaturitySafetySection as S;

    vec![
        (
            S::PredecessorConstructorFault,
            "metadata-leaf-duplicated",
            discharge(
                V::StaticSubtreeValidation,
                Change::OfferAStaticSubtreeHoldingTheMetadataLeaf,
                static_subtree_incomplete,
                "StaticSubtreeIncomplete",
            ),
        ),
        (
            S::PredecessorConstructorFault,
            "operation-leaf-missing",
            discharge(
                V::StaticSubtreeValidation,
                Change::OfferAStaticSubtreeWithoutTheOperationLeaf,
                static_subtree_incomplete,
                "StaticSubtreeIncomplete",
            ),
        ),
        (
            S::SuccessorConstructorFault,
            "spendable-metadata-leaf",
            discharge(
                V::MetadataLeafPattern,
                Change::OfferAMetadataLeafWhoseVerificationCanSurvive,
                metadata_leaf_not_unspendable,
                "MetadataLeafNotUnspendable",
            ),
        ),
        (
            S::TotalityFault,
            "metadata-child-on-wrong-side",
            discharge(
                V::CanonicalBranchSide,
                Change::StateTheMetadataChildOnTheOtherSide,
                canonical_branch_side,
                "CanonicalBranchSideNotSatisfied",
            ),
        ),
        (
            S::AbiLinkerFault,
            "duplicate-tree-leaf",
            discharge(
                V::StaticSubtreeValidation,
                Change::DeclareOneLeafIdentityTwiceIdentically,
                duplicate_leaf,
                "DuplicateLeaf",
            ),
        ),
        (
            S::AbiLinkerFault,
            "conflicting-leaf-role",
            discharge(
                V::StaticSubtreeValidation,
                Change::DeclareOneLeafIdentityUnderTwoRoles,
                conflicting_leaf,
                "ConflictingLeaf",
            ),
        ),
    ]
}

/// The four linker rows a caller can offer an input for, across two
/// entry points.
///
/// Not one call's rows either, and the split is the same reading the
/// constructor half uses: the link accepts a tuple of sources and the
/// linked-program check accepts a program beside the census that claims
/// to describe it, so a row about which constructor was supplied is
/// answered where constructors are supplied, and a row about an
/// instruction of the linked program is answered where a program is
/// checked. Two rows reaching one class stay two rows, because what
/// separates them is which constructor was offered: one retained from
/// this bundle's own run, and one applied over a bundle linked for
/// another deployment.
fn linker_discharges() -> Vec<DischargeEntry> {
    use MaturityFirstPartyChange as Change;
    use MaturityFirstPartyValidator as V;
    use MaturitySafetySection as S;

    let committed_elsewhere = |change| {
        discharge(
            V::StateCandidateLink,
            change,
            supplied_constructor_commits_another_program,
            "SuppliedConstructorCommitsAnotherProgram",
        )
    };
    vec![
        (
            S::PredecessorConstructorFault,
            "stale-constructor-from-another-bundle",
            committed_elsewhere(Change::OfferTheConstructorTheLinkItselfRetained),
        ),
        (
            S::AbiLinkerFault,
            "leaf-from-another-bundle",
            committed_elsewhere(Change::OfferAConstructorLinkedForAnotherDeployment),
        ),
        (
            S::AbiLinkerFault,
            "relocation-omitted",
            discharge(
                V::LinkedProgramCheck,
                Change::RestoreOneRelocatedSiteToItsPristinePush,
                relocation_not_applied,
                "RelocationNotApplied",
            ),
        ),
        (
            S::AbiLinkerFault,
            "relocation-applied-twice",
            discharge(
                V::LinkedProgramCheck,
                Change::WriteOneLinkedValueAtASiteNoRelocationCovers,
                untracked_program_mutation,
                "UntrackedProgramMutation",
            ),
        ),
    ]
}

/// The one row about a transaction offered in place of the built one.
///
/// Filed against the check that compares an offering with the finalized
/// candidate rather than against the construction, because the row is
/// not about a construction that went wrong: it is about a caller who
/// did not use one, and the only first-party call that can say so is the
/// one holding the transaction the construction produced.
fn offered_transaction_discharges() -> Vec<DischargeEntry> {
    vec![(
        MaturitySafetySection::AbiLinkerFault,
        "raw-transaction-bypassing-safe-construction",
        discharge(
            MaturityFirstPartyValidator::OfferedTransactionCheck,
            MaturityFirstPartyChange::OfferATransactionWhoseOutputTheConstructionDidNotFix,
            output_mutated_after_signing,
            "OutputMutatedAfterSigning",
        ),
    )]
}

/// The six protocol rows whose subject is a typed response record.
///
/// Three entry points and one defect vocabulary, which is why the
/// validator beside each row is what says who spoke: the three calls
/// read three different records, and a reader who knew only the class
/// would not know which record was offered. The request half of that
/// table is not here — its subject is the wire message a request arrives
/// as, and no first-party call accepts one — and neither is the signing
/// echo, whose comparison happens inside a binding this workspace
/// publishes no entry to.
fn protocol_discharges() -> Vec<DischargeEntry> {
    use MaturityFirstPartyChange as Change;
    use MaturityFirstPartyValidator as V;
    use MaturitySafetySection as S;

    let submission = |change, expected: fn(&MaturityFirstPartyObservation) -> bool, name| {
        discharge(V::OperationResponseShape, change, expected, name)
    };
    vec![
        (
            S::ProtocolReportFault,
            "infrastructure-response-carrying-target-observation",
            discharge(
                V::ResponseShapeValidation,
                Change::ReportAStackBesideAVerdictThatNothingRan,
                infrastructure_response_carries_observation,
                "InfrastructureResponseCarriesObservation",
            ),
        ),
        (
            S::ProtocolReportFault,
            "accepted-submission-without-identity",
            submission(
                Change::OmitTheIdentityAnAcceptedSubmissionTook,
                accepted_operation_omits_observation,
                "AcceptedOperationOmitsObservation",
            ),
        ),
        (
            S::ProtocolReportFault,
            "accepted-submission-without-mined-readback-where-required",
            submission(
                Change::OmitTheMinedReadbackAnAcceptedSubmissionOwes,
                accepted_operation_omits_observation,
                "AcceptedOperationOmitsObservation",
            ),
        ),
        (
            S::ProtocolReportFault,
            "rejected-submission-carrying-accepted-identity",
            submission(
                Change::CarryAnAcceptedIdentityOnARefusedSubmission,
                refused_operation_carries_observation,
                "RefusedOperationCarriesObservation",
            ),
        ),
        (
            S::ProtocolReportFault,
            "rejected-signing-response-carrying-witness",
            submission(
                Change::CarryAWitnessOnARefusedSigningStep,
                refused_operation_carries_observation,
                "RefusedOperationCarriesObservation",
            ),
        ),
        (
            S::ProtocolReportFault,
            "conservation-rejection-carrying-accepted-only-openings",
            discharge(
                V::ConservationResponseShape,
                Change::CarryAnOpeningOnARefusedConservationRow,
                refused_conservation_carries_openings,
                "RefusedConservationCarriesOpenings",
            ),
        ),
    ]
}

/// The pre-target rows this census discharges, by table and name.
fn discharges() -> Vec<DischargeEntry> {
    let mut entries = window_discharges();
    entries.extend(operator_discharges());
    entries.extend(metadata_discharges());
    entries.extend(constructor_discharges());
    entries.extend(linker_discharges());
    entries.extend(offered_transaction_discharges());
    entries.extend(protocol_discharges());
    entries
}

/// Why one pre-target row this census does not discharge carries no
/// case.
///
/// Total over the pre-target rows, and decided from the row's own facts
/// rather than from a list beside them: a row whose stated change sits
/// at the ABI layout is carried for that reason whichever table it comes
/// from, a row whose owner is located is carried naming the owner, and a
/// row whose change an owner accepts is carried naming the mechanism
/// that accepted it. The last kind is decided the same way — the
/// metadata table's nonce row is the one carried by the linked
/// constructor rather than by the announcement leaf, the linker's rows
/// are separated by where their change would land, and the constructor
/// rows are separated below.
///
/// # Why the protocol table splits on a relation
///
/// Because that is the fact that separates the two rows left standing
/// there. The request rows are carried because no first-party call
/// accepts a wire message, which their locator says. One response row is
/// carried for the same reason and cannot say so with a locator: the
/// comparison that owns it — a signing echo against the bytes the
/// request named — happens inside a binding the executor performs and
/// publishes no entry to, and what distinguishes that row from the five
/// response rows discharged beside it is that it is the only one naming
/// a published relation of its own.
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
            ) || matches!(row.relation(), MaturityRelationStanding::Declared { .. })
            {
                Why::TheOwningValidatorHasNoPublicEntry
            } else {
                Why::OwnerLocatedDischargeDeferred(Owner::ResponseShapeValidation)
            }
        }
        MaturitySafetySection::MetadataFault => {
            if matches!(row.carrier(), MaturityIntendedCarrier::LinkedConstructor) {
                Why::LeastnessIsAPropertyOfTheSearchNotOfAnOffer
            } else {
                Why::OwnerLocatedDischargeDeferred(Owner::MetadataPatternFromBytes)
            }
        }
        _ => match row.refusing_layer() {
            Some(EvidenceBoundary::ConstructorDerivationRejection) => constructor_reason(row),
            Some(EvidenceBoundary::LinkerRejection) => linker_reason(row),
            _ => Why::OwnerLocatedDischargeDeferred(Owner::OfferedTransactionCheck),
        },
    }
}

/// Why one linker row this census does not discharge carries no case.
///
/// Both reasons say that the artifact the row would disturb is one the
/// link makes rather than one it takes, and they are two reasons because
/// a reader asking why has to be told which artifact: a resolved census
/// the link resolves for itself before any relocation reads it, or a
/// bound the link measures over the leaf it substituted and the graph it
/// assembled. Where the change would land is what separates them, and
/// the row states it.
const fn linker_reason(row: &MaturitySafetyRow) -> MaturityCarriedReason {
    if matches!(
        row.locator(),
        Some(MaturityMutationLocator::LinkerRelocation)
    ) {
        return MaturityCarriedReason::TheResolvedCensusIsResolvedByTheOwnerItself;
    }
    MaturityCarriedReason::TheBoundIsMeasuredOverWhatTheOwnerBuilt
}

/// Why one constructor row this census does not discharge carries no
/// case.
///
/// Decided from the row's own facts in the order that separates them: a
/// row that states an order is answered by the order being normalized
/// whatever else it says, a row whose class is a mismatch with a subject
/// asks for a comparison rather than a form, and the two rows left are
/// separated by the table they come from, because one states a term the
/// constructor's typed leaf does not carry and the other states a leaf
/// the constructor derives rather than accepts.
const fn constructor_reason(row: &MaturitySafetyRow) -> MaturityCarriedReason {
    use MaturityCarriedReason as Why;

    if matches!(row.locator(), Some(MaturityMutationLocator::BranchOrder)) {
        return Why::TheOfferedOrderIsNormalizedBeforeItIsCommitted;
    }
    if matches!(
        row.relation(),
        MaturityRelationStanding::Declared {
            class: MaturityMutationClass::ExternalReportSubjectMismatch,
            ..
        }
    ) {
        return Why::TheOwnerHasNoRetainedObjectToCompareTheOfferAgainst;
    }
    match row.section() {
        MaturitySafetySection::AbiLinkerFault => Why::TheTypedInputCarriesNoTermTheChangeNames,
        MaturitySafetySection::PredecessorConstructorFault => Why::TheValueIsDerivedNotAccepted,
        _ => Why::OwnerLocatedDischargeDeferred(MaturityDeferredOwner::StaticSubtreeConstruction),
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
    if supplemental_discharge(case.section, case.name).is_some() {
        return validate_supplementary_comparison(case, row, discharge);
    }
    let observed = match discharge.validator {
        MaturityFirstPartyValidator::ConstructorContinuity
        | MaturityFirstPartyValidator::SymbolResolution
        | MaturityFirstPartyValidator::AuthorizedSuccessor
        | MaturityFirstPartyValidator::CompleteStateView
        | MaturityFirstPartyValidator::WeightedStateDeclarations
        | MaturityFirstPartyValidator::CompleteStateBounds
        | MaturityFirstPartyValidator::ProtocolEnvironment
        | MaturityFirstPartyValidator::TranscriptProvenance
        | MaturityFirstPartyValidator::ScriptPathBinding => {
            return Err(MaturityFirstPartyRefusal::ScenarioNotConstructible);
        }
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
        MaturityFirstPartyValidator::MetadataPatternDecode => {
            discharge_metadata_decode(substrate, discharge.change)
        }
        MaturityFirstPartyValidator::MetadataLeafPattern => {
            discharge_metadata_leaf(substrate, discharge.change)
        }
        MaturityFirstPartyValidator::StaticSubtreeValidation => {
            discharge_static_subtree(substrate, discharge.change)
        }
        MaturityFirstPartyValidator::CanonicalBranchSide => {
            discharge_branch_side(substrate, discharge.change)
        }
        MaturityFirstPartyValidator::StateCandidateLink => {
            discharge_candidate_link(substrate, discharge.change)
        }
        MaturityFirstPartyValidator::LinkedProgramCheck => {
            discharge_linked_program(substrate, discharge.change)
        }
        MaturityFirstPartyValidator::OfferedTransactionCheck => {
            discharge_offered_transaction(substrate, discharge.change)
        }
        MaturityFirstPartyValidator::ResponseShapeValidation => {
            discharge_execution_response(discharge.change)
        }
        MaturityFirstPartyValidator::OperationResponseShape => {
            discharge_operation_response(discharge.change)
        }
        MaturityFirstPartyValidator::ConservationResponseShape => {
            discharge_conservation_response(discharge.change)
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
        comparison: None,
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

/// Test staging for constructor continuity; never a production constructor bypass.
/// Both subjects are derived normally, with a changed static tree on the offered side.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityConstructorComparisonTestStaging {
    control: CandidateStateConstructor,
    malformed: CandidateStateConstructor,
}

impl MaturityConstructorComparisonTestStaging {
    /// The retained constructor used as the accepted control.
    #[must_use]
    pub const fn control(&self) -> &CandidateStateConstructor {
        &self.control
    }

    /// The constructor whose static tree disagrees with the retained one.
    #[must_use]
    pub const fn malformed(&self) -> &CandidateStateConstructor {
        &self.malformed
    }

    fn new(change: MaturityFirstPartyChange) -> Result<Self, MaturityFirstPartyRefusal> {
        let substrate = substrate()?;
        let control = retained_constructor(substrate)?.clone();
        let subtree = control.static_subtree();
        let first = subtree
            .leaves()
            .first()
            .ok_or(MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
        let identity = subtree
            .leaves()
            .iter()
            .map(|entry| entry.identity)
            .max()
            .and_then(|value| value.checked_add(1))
            .ok_or(MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
        let mut leaf = first.leaf.clone();
        let tree = match change {
            MaturityFirstPartyChange::ChangeRetainedStaticLeaf => {
                let mut instructions = leaf.program.instructions().to_vec();
                instructions.push(TapscriptInstruction::Push(
                    StackItem::new(&substrate.target, vec![1])
                        .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?,
                ));
                leaf.program = TapscriptProgram::new(instructions)
                    .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
                replace_static_leaf(subtree.tree(), first.identity, &leaf)
            }
            MaturityFirstPartyChange::AddRetainedStaticLeaf => {
                leaf.role = StateLeafRole::Support(identity);
                leaf.program = TapscriptProgram::new(vec![TapscriptInstruction::Push(
                    StackItem::new(&substrate.target, vec![1])
                        .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?,
                )])
                .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
                StateStaticNode::Branch(
                    Box::new(subtree.tree().clone()),
                    Box::new(StateStaticNode::Leaf { identity, leaf }),
                )
            }
            _ => return Err(MaturityFirstPartyRefusal::ChangeChangedNothing),
        };
        let changed = StateStaticSubtree::new(&substrate.target, Some(tree))
            .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
        let malformed = CandidateStateConstructor::derive(
            &substrate.target,
            &control.encoded_metadata().semantic,
            &changed,
            substrate.bundle.policy().internal_key(),
            StateNonceBudget::default(),
            &OracleStateCurve,
        )
        .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
        Ok(Self { control, malformed })
    }

    fn observe(&self) -> Result<MaturityFirstPartyObservation, MaturityFirstPartyRefusal> {
        if self.control.static_subtree().root() == self.malformed.static_subtree().root() {
            return Err(MaturityFirstPartyRefusal::ChangeChangedNothing);
        }
        self.control.continuity(&self.control).map_err(|refusal| {
            MaturityFirstPartyRefusal::ControlWasRefused(
                MaturityFirstPartyObservation::Constructor(refusal),
            )
        })?;
        let observed = self
            .control
            .continuity(&self.malformed)
            .err()
            .ok_or(MaturityFirstPartyRefusal::ChangedInputWasAccepted)?;
        let observed = MaturityFirstPartyObservation::Constructor(observed);
        if observed
            != MaturityFirstPartyObservation::Constructor(StateConstructorRefusal::ConflictingLeaf)
        {
            return Err(MaturityFirstPartyRefusal::RefusalNamesAnotherClass(
                observed,
            ));
        }
        Ok(observed)
    }
}

fn replace_static_leaf(
    tree: &StateStaticNode,
    wanted: u32,
    replacement: &StateStaticLeaf,
) -> StateStaticNode {
    match tree {
        StateStaticNode::Leaf { identity, .. } if *identity == wanted => StateStaticNode::Leaf {
            identity: *identity,
            leaf: replacement.clone(),
        },
        StateStaticNode::Leaf { .. } => tree.clone(),
        StateStaticNode::Branch(left, right) => StateStaticNode::Branch(
            Box::new(replace_static_leaf(left, wanted, replacement)),
            Box::new(replace_static_leaf(right, wanted, replacement)),
        ),
    }
}

/// Test staging for symbol resolution using only public definition constructors.
/// The malformed census omits exactly one consumed definition; consumers stay fixed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturitySymbolResolutionTestStaging {
    control: linker::StateDefinitionCensus,
    malformed: linker::StateDefinitionCensus,
    consumers: StateConsumerCensus,
    missing: linker::StateLinkSymbol,
}

impl MaturitySymbolResolutionTestStaging {
    /// The complete definitions offered to resolution.
    #[must_use]
    pub const fn control(&self) -> &linker::StateDefinitionCensus {
        &self.control
    }

    /// The same definitions with one consumed symbol omitted.
    #[must_use]
    pub const fn malformed(&self) -> &linker::StateDefinitionCensus {
        &self.malformed
    }

    /// The exact symbol whose absence must be reported.
    #[must_use]
    pub const fn missing(&self) -> linker::StateLinkSymbol {
        self.missing
    }

    fn new(missing: linker::StateLinkSymbol) -> Result<Self, MaturityFirstPartyRefusal> {
        let substrate = substrate()?;
        let (singleton, declaration) = link_deployment_values()?;
        let constructor = substrate.sources.constructor();
        let control = collect_state_definitions(
            &substrate.target,
            substrate.bundle.deployment(),
            constructor,
            &singleton,
            &declaration,
            substrate.sources.record().schedule(),
        )
        .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
        let consumers = StateConsumerCensus::from_sources(substrate.sources.record(), constructor);
        let mut malformed = linker::StateDefinitionCensus::default();
        for (&symbol, definition) in control.definitions() {
            if symbol != missing {
                malformed
                    .define(symbol, definition.value().clone(), definition.origin())
                    .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
            }
        }
        Ok(Self {
            control,
            malformed,
            consumers,
            missing,
        })
    }

    fn observe(&self) -> Result<MaturityFirstPartyObservation, MaturityFirstPartyRefusal> {
        if self.control == self.malformed {
            return Err(MaturityFirstPartyRefusal::ChangeChangedNothing);
        }
        resolve_state_census(&self.control, &self.consumers).map_err(|refusal| {
            MaturityFirstPartyRefusal::ControlWasRefused(MaturityFirstPartyObservation::StateLink(
                Box::new(refusal),
            ))
        })?;
        let observed = resolve_state_census(&self.malformed, &self.consumers)
            .err()
            .ok_or(MaturityFirstPartyRefusal::ChangedInputWasAccepted)?;
        let observed = MaturityFirstPartyObservation::StateLink(Box::new(observed));
        let expected = MaturityFirstPartyObservation::StateLink(Box::new(
            StateLinkRefusal::MissingDefinition(self.missing),
        ));
        if observed != expected {
            return Err(MaturityFirstPartyRefusal::RefusalNamesAnotherClass(
                observed,
            ));
        }
        Ok(observed)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EntryPair<T> {
    control: T,
    malformed: T,
}

impl<T: Eq> EntryPair<T> {
    fn differs(&self) -> Result<(), MaturityFirstPartyRefusal> {
        if self.control == self.malformed {
            Err(MaturityFirstPartyRefusal::ChangeChangedNothing)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum MaturityEntrySubjects {
    Successor {
        finalized: Box<FinalizedMaturityAnnouncement>,
        responses: Vec<OperatorSigningResponse>,
        offers: Box<EntryPair<StateMetadata>>,
    },
    CompleteProgram {
        offers: Box<EntryPair<PublicMaturityStateView>>,
        static_root: [u8; 32],
        static_program: Vec<u8>,
    },
    Weights(Box<EntryPair<Vec<linker::TapLeafInput<StateLeafRole>>>>),
    Bounds {
        constructor: Box<CandidateStateConstructor>,
        tree: Box<linker::DeterministicTaptree<StateLeafRole>>,
        caps: EntryPair<NonZeroU32>,
    },
    Environment {
        binding: Box<target_elements::ReviewedDevelopmentBinding>,
        offers: Box<EntryPair<native_protocol::ExecutorEnvironmentObservation>>,
    },
    Provenance {
        binding: Box<target_elements::ReviewedDevelopmentBinding>,
        environment: Box<native_protocol::ExecutorEnvironmentObservation>,
        configuration: Box<native_executor::ExecutorConfiguration>,
        offers: Box<EntryPair<native_protocol::ExecutorHandshake>>,
    },
    Signing {
        subject: Box<native_protocol::OperationSubject>,
        environment: Box<native_protocol::ExecutorEnvironmentObservation>,
        offers: Box<EntryPair<NativeOperationResponse>>,
    },
}

/// Retained control and changed offers at an existing first-party entry.
/// Construction uses public typed inputs; observations are recomputed from both offers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityEntryComparisonTestStaging {
    change: MaturityFirstPartyChange,
    subjects: MaturityEntrySubjects,
}

impl MaturityEntryComparisonTestStaging {
    /// The single coordinate changed by this pair.
    #[must_use]
    pub const fn change(&self) -> MaturityFirstPartyChange {
        self.change
    }

    fn new(change: MaturityFirstPartyChange) -> Result<Self, MaturityFirstPartyRefusal> {
        use MaturityFirstPartyChange as Change;
        let subjects = match change {
            Change::ChangeAuthorizedSuccessorCycle => successor_entry_pair()?,
            Change::OmitMetadataCommitmentBranch => complete_program_pair()?,
            Change::AddConflictingWeightedDeclaration => {
                let leaf = linker::TapLeafInput::new(StateLeafRole::Announcement, NonZeroU64::MIN);
                MaturityEntrySubjects::Weights(Box::new(EntryPair {
                    control: vec![leaf],
                    malformed: vec![
                        leaf,
                        linker::TapLeafInput::new(leaf.leaf(), leaf.weight().saturating_add(1)),
                    ],
                }))
            }
            Change::LowerCompleteTreeDepthCap => complete_bounds_pair()?,
            Change::ChangeObservedSchema | Change::ChangeObservedNetwork => {
                let (binding, control) = entry_environment()?;
                let mut malformed = control.clone();
                if change == Change::ChangeObservedSchema {
                    malformed.schema = NATIVE_PROTOCOL_SCHEMA + 1;
                } else {
                    malformed.network_id[0] ^= 1;
                }
                MaturityEntrySubjects::Environment {
                    binding: Box::new(binding),
                    offers: Box::new(EntryPair { control, malformed }),
                }
            }
            Change::AddUnexpectedProvenanceTopic => provenance_entry_pair()?,
            Change::ChangeSignatureBoundBytes => signing_entry_pair()?,
            _ => return Err(MaturityFirstPartyRefusal::ScenarioNotConstructible),
        };
        Ok(Self { change, subjects })
    }

    fn observe(&self) -> Result<MaturityFirstPartyObservation, MaturityFirstPartyRefusal> {
        use MaturityEntrySubjects as Subjects;
        let target = &substrate()?.target;
        let observed = match &self.subjects {
            Subjects::Successor {
                finalized,
                responses,
                offers,
            } => observe_successor_entry(finalized, responses, offers),
            Subjects::CompleteProgram { offers, .. } => {
                offers.differs()?;
                entry_outcome(
                    offers
                        .control
                        .validate(target, &OracleStateCurve)
                        .map(|_| ()),
                    offers
                        .malformed
                        .validate(target, &OracleStateCurve)
                        .map(|_| ()),
                    MaturityFirstPartyObservation::Transaction,
                )
            }
            Subjects::Weights(offers) => {
                offers.differs()?;
                let call = |leaves: &[linker::TapLeafInput<StateLeafRole>]| {
                    linker::TaptreeInput::new(
                        leaves.iter().copied(),
                        target.definition().leaf_version(),
                        linker::TreeObjective::MinimumTotalWeightedDepth,
                        NonZeroU32::MIN.saturating_add(127),
                    )
                    .map(|_| ())
                };
                entry_outcome(call(&offers.control), call(&offers.malformed), |error| {
                    MaturityFirstPartyObservation::Link(Box::new(error))
                })
            }
            Subjects::Bounds {
                constructor,
                tree,
                caps,
            } => {
                caps.differs()?;
                let call = |cap| {
                    linker::StateLinkedTaptree::bind(
                        target,
                        tree.as_ref().clone(),
                        constructor,
                        cap,
                    )
                    .map(|_| ())
                };
                entry_outcome(call(caps.control), call(caps.malformed), |error| {
                    MaturityFirstPartyObservation::Link(Box::new(error))
                })
            }
            Subjects::Environment { binding, offers } => {
                offers.differs()?;
                let call = |offer| {
                    environment_entry_outcome(&native_executor::compare_environment(
                        target, binding, offer,
                    ))
                };
                entry_outcome(call(&offers.control)?, call(&offers.malformed)?, |error| {
                    error
                })
            }
            Subjects::Provenance {
                binding,
                environment,
                configuration,
                offers,
            } => {
                offers.differs()?;
                let call =
                    |offer| provenance_entry_outcome(binding, environment, configuration, offer);
                entry_outcome(
                    call(&offers.control)?,
                    call(&offers.malformed)?,
                    MaturityFirstPartyObservation::Provenance,
                )
            }
            Subjects::Signing {
                subject,
                environment,
                offers,
            } => observe_signing_entry(subject, environment, offers),
        }?;
        if !entry_refusal_matches(self.change, &observed) {
            return Err(MaturityFirstPartyRefusal::RefusalNamesAnotherClass(
                observed,
            ));
        }
        Ok(observed)
    }
}

fn observe_successor_entry(
    finalized: &FinalizedMaturityAnnouncement,
    responses: &[OperatorSigningResponse],
    offers: &EntryPair<StateMetadata>,
) -> Result<MaturityFirstPartyObservation, MaturityFirstPartyRefusal> {
    offers.differs()?;
    let state = open_signing(substrate()?, finalized)?;
    let mut registry = OperatorRightRegistry::default();
    let right = registry
        .issue(state.construction_right_scope(), state.protected_bytes())
        .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    let authorized = state
        .authorize(&mut registry, right, responses.to_vec(), &OperatorVerifier)
        .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    entry_outcome(
        authorized.check_offered(finalized.protected(), offers.control),
        authorized.check_offered(finalized.protected(), offers.malformed),
        MaturityFirstPartyObservation::Transaction,
    )
}

fn observe_signing_entry(
    subject: &native_protocol::OperationSubject,
    environment: &native_protocol::ExecutorEnvironmentObservation,
    offers: &EntryPair<NativeOperationResponse>,
) -> Result<MaturityFirstPartyObservation, MaturityFirstPartyRefusal> {
    offers.differs()?;
    for offer in [&offers.control, &offers.malformed] {
        offer
            .validate_shape()
            .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    }
    entry_outcome(
        native_executor::bind_script_path_response(subject, &offers.control, environment),
        native_executor::bind_script_path_response(subject, &offers.malformed, environment),
        MaturityFirstPartyObservation::ResponseShape,
    )
}

fn entry_outcome<E>(
    control: Result<(), E>,
    malformed: Result<(), E>,
    observation: impl Fn(E) -> MaturityFirstPartyObservation,
) -> Result<MaturityFirstPartyObservation, MaturityFirstPartyRefusal> {
    control.map_err(|error| MaturityFirstPartyRefusal::ControlWasRefused(observation(error)))?;
    malformed
        .err()
        .map(observation)
        .ok_or(MaturityFirstPartyRefusal::ChangedInputWasAccepted)
}

fn successor_entry_pair() -> Result<MaturityEntrySubjects, MaturityFirstPartyRefusal> {
    let finalized = honest_finalized(substrate()?)?;
    let state = open_signing(substrate()?, &finalized)?;
    let responses = vec![honest_response(&state)?];
    let control = finalized.construction().successor_metadata();
    let mut malformed = control;
    let Maturity::Announced { cycle } = control.maturity else {
        return Err(MaturityFirstPartyRefusal::ScenarioNotConstructible);
    };
    malformed.maturity = Maturity::Announced {
        cycle: Cycle::new(
            cycle
                .get()
                .checked_add(1)
                .ok_or(MaturityFirstPartyRefusal::ScenarioNotConstructible)?,
        ),
    };
    Ok(MaturityEntrySubjects::Successor {
        finalized: Box::new(finalized),
        responses,
        offers: Box::new(EntryPair { control, malformed }),
    })
}

fn static_only_program(root: &[u8; 32]) -> Result<Vec<u8>, MaturityFirstPartyRefusal> {
    use tapscript::{StateCurveCapability, StateTweakOutcome};
    let key = substrate()?.bundle.policy().internal_key();
    let StateTweakOutcome::OutputKey { key, .. } = OracleStateCurve.output_key(key.key(), root)
    else {
        return Err(MaturityFirstPartyRefusal::ScenarioNotConstructible);
    };
    let mut program = vec![0x51, 0x20];
    program.extend_from_slice(&key);
    Ok(program)
}

fn complete_program_pair() -> Result<MaturityEntrySubjects, MaturityFirstPartyRefusal> {
    let substrate = substrate()?;
    let control = PublicMaturityStateView::new(statements(substrate, &substrate.honest)?)
        .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    let static_root = *substrate.bundle.static_subtree().root();
    let static_program = static_only_program(&static_root)?;
    let world = MaturityWorld {
        program: static_program.clone(),
        ..substrate.honest.clone()
    };
    let malformed = PublicMaturityStateView::new(statements(substrate, &world)?)
        .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    Ok(MaturityEntrySubjects::CompleteProgram {
        offers: Box::new(EntryPair { control, malformed }),
        static_root,
        static_program,
    })
}

fn complete_bounds_pair() -> Result<MaturityEntrySubjects, MaturityFirstPartyRefusal> {
    let staged = MaturityConstructorComparisonTestStaging::new(
        MaturityFirstPartyChange::AddRetainedStaticLeaf,
    )?;
    let constructor = staged.malformed;
    let target = &substrate()?.target;
    let input = linker::state_static_taptree_input(
        constructor
            .static_subtree()
            .leaves()
            .iter()
            .map(|entry| entry.leaf.role),
        target.definition().leaf_version(),
        NonZeroU32::MIN.saturating_add(127),
    )
    .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    let tree = linker::assemble_state_static(&input)
        .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    let control = NonZeroU32::new(
        tree.depth()
            .checked_add(1)
            .ok_or(MaturityFirstPartyRefusal::ScenarioNotConstructible)?,
    )
    .ok_or(MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    let malformed =
        NonZeroU32::new(tree.depth()).ok_or(MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    Ok(MaturityEntrySubjects::Bounds {
        constructor: Box::new(constructor),
        tree: Box::new(tree),
        caps: EntryPair { control, malformed },
    })
}

fn entry_environment() -> Result<
    (
        target_elements::ReviewedDevelopmentBinding,
        native_protocol::ExecutorEnvironmentObservation,
    ),
    MaturityFirstPartyRefusal,
> {
    use target_elements::{
        ActivationDeclaration, DeploymentEnvironment, DevelopmentDeploymentBinding,
        validate_reviewed_development_binding,
    };
    let target = &substrate()?.target;
    let definition = target.definition();
    let binding = validate_reviewed_development_binding(
        target,
        DevelopmentDeploymentBinding::new(
            definition.version(),
            DeploymentEnvironment::Development,
            [0x11; 32],
            [0x22; 32],
            ActivationDeclaration::new(true, definition.leaf_version(), []),
            None,
        ),
    )
    .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    let observation = native_protocol::ExecutorEnvironmentObservation {
        schema: NATIVE_PROTOCOL_SCHEMA,
        environment: native_protocol::WireEnvironment::Development,
        chain_name: "elementsregtest".to_owned(),
        network_id: [0x11; 32],
        genesis_id: [0x22; 32],
        active_domains: BTreeSet::from([native_protocol::WireExecutionDomain::Tapscript]),
        active_leaf_versions: BTreeSet::from([definition.leaf_version().get()]),
    };
    Ok((binding, observation))
}

const fn environment_entry_outcome(
    result: &Result<(), target_elements_conformance::NativeConformanceError>,
) -> Result<Result<(), MaturityFirstPartyObservation>, MaturityFirstPartyRefusal> {
    use target_elements_conformance::NativeConformanceError as Error;
    match result {
        Ok(()) => Ok(Ok(())),
        Err(Error::UnsupportedProtocolSchema { offered }) => Ok(Err(
            MaturityFirstPartyObservation::UnsupportedProtocolSchema { offered: *offered },
        )),
        Err(Error::EnvironmentBindingMismatch) => Ok(Err(
            MaturityFirstPartyObservation::EnvironmentBindingMismatch,
        )),
        Err(_) => Err(MaturityFirstPartyRefusal::ScenarioNotConstructible),
    }
}

fn provenance_entry_pair() -> Result<MaturityEntrySubjects, MaturityFirstPartyRefusal> {
    use std::{path::Path, time::Duration};
    let (binding, environment) = entry_environment()?;
    // Synthetic source provenance for a scripted comparison, never a repository citation.
    let tip = "ab".repeat(20);
    let base = "cd".repeat(20);
    let expected = ExpectedExecutorProvenance::new(&tip, &base, ["fix/control"])
        .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    let configuration = native_executor::ExecutorConfiguration::new(
        Path::new("unused-scripted-executor"),
        native_executor::ExecutorTrust::Mock,
        Duration::from_secs(1),
        native_executor::ExecutorDiagnostics::in_directory(Path::new(
            "unused-scripted-diagnostics",
        )),
    )
    .with_expected_provenance(expected);
    let control = native_protocol::ExecutorHandshake {
        protocol_schema: NATIVE_PROTOCOL_SCHEMA,
        adapter_name: "scripted-control".to_owned(),
        adapter_version: "1".to_owned(),
        framework_revision: None,
        node_name: "scripted-node".to_owned(),
        node_version: "1".to_owned(),
        binary_reported_revision: Some(tip.clone()),
        intended_executed_tip: Some(tip),
        upstream_base: Some(base),
        included_local_topics: BTreeSet::from(["fix/control".to_owned()]),
        supported_domains: environment.active_domains.clone(),
        supported_leaf_versions: environment.active_leaf_versions.clone(),
        capabilities: BTreeSet::new(),
        confidential_funding: None,
    };
    let mut malformed = control.clone();
    malformed
        .included_local_topics
        .insert("fix/unexpected".to_owned());
    Ok(MaturityEntrySubjects::Provenance {
        binding: Box::new(binding),
        environment: Box::new(environment),
        configuration: Box::new(configuration),
        offers: Box::new(EntryPair { control, malformed }),
    })
}

struct EmptyEntryPlan;

impl native_executor::TargetOperationPlanner for EmptyEntryPlan {
    fn next_step(
        &mut self,
        _: Option<(&OperationCaseId, &NativeOperationResponse)>,
    ) -> Result<Option<native_executor::OperationStep>, native_executor::PlanRefused> {
        Ok(None)
    }
}

fn provenance_entry_outcome(
    binding: &target_elements::ReviewedDevelopmentBinding,
    environment: &native_protocol::ExecutorEnvironmentObservation,
    configuration: &native_executor::ExecutorConfiguration,
    handshake: &native_protocol::ExecutorHandshake,
) -> Result<Result<(), ProvenanceDefect>, MaturityFirstPartyRefusal> {
    let mut bytes = serde_json::to_vec(handshake)
        .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    bytes.push(b'\n');
    serde_json::to_writer(&mut bytes, environment)
        .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    bytes.push(b'\n');
    let transcript = native_executor::run_protocol(
        &substrate()?.target,
        binding,
        configuration,
        native_executor::NativeWorkload::Operations(&mut EmptyEntryPlan),
        Vec::new(),
        &mut std::io::Cursor::new(bytes),
    )
    .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    transcript
        .provenance_agreement()
        .ok_or(MaturityFirstPartyRefusal::ScenarioNotConstructible)
        .map(|result| result.map(|_| ()))
}

fn signing_entry_pair() -> Result<MaturityEntrySubjects, MaturityFirstPartyRefusal> {
    use native_protocol::{
        OperationSubject, TargetScriptPathSigningSubject, WireSighashProfile, WireSpentOutput,
        WireTapleaf,
    };
    let finalized = honest_finalized(substrate()?)?;
    let state = open_signing(substrate()?, &finalized)?;
    let signature = honest_response(&state)?;
    let leaf = finalized.executing_leaf(&substrate()?.target);
    let spent = finalized.spent_output();
    let (AssetField::Explicit(asset), ValueField::Explicit(amount)) =
        (spent.asset(), spent.value())
    else {
        return Err(MaturityFirstPartyRefusal::ScenarioNotConstructible);
    };
    let mut asset_field = vec![1];
    asset_field.extend_from_slice(asset.internal());
    let mut value_field = vec![1];
    value_field.extend_from_slice(&amount.to_be_bytes());
    let subject = TargetScriptPathSigningSubject {
        finalized_transaction: finalized.protected_bytes().to_vec(),
        input_index: 0,
        spent_outputs: vec![WireSpentOutput {
            asset_field,
            value_field,
            program: spent.program().to_vec(),
        }],
        executing_leaf: WireTapleaf {
            leaf_version: leaf.leaf_version().get(),
            script: leaf.leaf_script().to_vec(),
            control_block: leaf.control_block().to_vec(),
        },
        sighash_profile: WireSighashProfile::AllInputsAllOutputs,
        signer: OPERATOR_HANDLE,
    };
    let (_, environment) = entry_environment()?;
    let mut control = operation_response(
        OperationStepKind::SignScriptPath,
        "first-party-binding",
        ObservedOutcomeLayer::Accepted,
    );
    control.script_path_witness = vec![signature.signature().to_vec()];
    control.signer_public_key = Some(
        subject
            .signer
            .x_only_public_key()
            .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?,
    );
    control.signed_profile = Some(subject.sighash_profile);
    control.signing_genesis = Some(environment.genesis_id);
    control.signature_bound_to = Some(subject.finalized_transaction.clone());
    let mut malformed = control.clone();
    let bytes = malformed
        .signature_bound_to
        .as_mut()
        .ok_or(MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    let byte = bytes
        .first_mut()
        .ok_or(MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
    *byte ^= 1;
    Ok(MaturityEntrySubjects::Signing {
        subject: Box::new(OperationSubject::ScriptPathSigning(Box::new(subject))),
        environment: Box::new(environment),
        offers: Box::new(EntryPair { control, malformed }),
    })
}

/// Actual paired subjects, retained beside the checked refusal rather than a success flag.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaturityRetainedComparison {
    /// Exact offers through the existing entry points added after bounded consolidation.
    Entries(Box<MaturityEntryComparisonTestStaging>),
    /// Static-root continuity over normally derived constructors.
    Constructor(Box<MaturityConstructorComparisonTestStaging>),
    /// Completeness of definitions against the same consumed symbols.
    Symbols(Box<MaturitySymbolResolutionTestStaging>),
}

impl MaturityRetainedComparison {
    fn observe(&self) -> Result<MaturityFirstPartyObservation, MaturityFirstPartyRefusal> {
        match self {
            Self::Entries(pair) => pair.observe(),
            Self::Constructor(pair) => pair.observe(),
            Self::Symbols(pair) => pair.observe(),
        }
    }
}

fn supplemental_discharge(
    section: MaturitySafetySection,
    name: &str,
) -> Option<MaturityFirstPartyDischarge> {
    use MaturityFirstPartyChange as Change;
    use MaturityFirstPartyValidator as Validator;
    use MaturitySafetySection as Section;
    if let Some((validator, change, expected_name)) = entry_discharge(section, name) {
        return Some(discharge(validator, change, entry_refusal, expected_name));
    }
    let (validator, change, expected_name) = match (section, name) {
        (Section::PredecessorConstructorFault, "wrong-static-subtree") => (
            Validator::ConstructorContinuity,
            Change::ChangeRetainedStaticLeaf,
            "ConflictingLeaf",
        ),
        (Section::PredecessorConstructorFault, "extra-escape-leaf") => (
            Validator::ConstructorContinuity,
            Change::AddRetainedStaticLeaf,
            "ConflictingLeaf",
        ),
        (Section::AbiLinkerFault, "unresolved-metadata-schema-symbol") => (
            Validator::SymbolResolution,
            Change::OmitConsumedMetadataSchema,
            "MissingDefinition(MetadataSchema)",
        ),
        (Section::AbiLinkerFault, "unresolved-lead-bound-symbol") => (
            Validator::SymbolResolution,
            Change::OmitConsumedMinimumLead,
            "MissingDefinition(MaturityLeadMin)",
        ),
        (Section::AbiLinkerFault, "unresolved-operator-symbol") => (
            Validator::SymbolResolution,
            Change::OmitConsumedOperator,
            "MissingDefinition(CommittedOperatorKey)",
        ),
        _ => return None,
    };
    Some(discharge(
        validator,
        change,
        supplemental_refusal,
        expected_name,
    ))
}

fn entry_discharge(
    section: MaturitySafetySection,
    name: &str,
) -> Option<(
    MaturityFirstPartyValidator,
    MaturityFirstPartyChange,
    &'static str,
)> {
    use MaturityFirstPartyChange as Change;
    use MaturityFirstPartyValidator as Validator;
    use MaturitySafetySection as Section;
    Some(match (section, name) {
        (Section::WindowFault, "request-cycle-differs-from-successor") => (
            Validator::AuthorizedSuccessor,
            Change::ChangeAuthorizedSuccessorCycle,
            "MaturitySuccessorMetadataChangedAfterSigning",
        ),
        (Section::PredecessorConstructorFault, "metadata-leaf-missing") => (
            Validator::CompleteStateView,
            Change::OmitMetadataCommitmentBranch,
            "MaturityViewProgramNotReconstructed",
        ),
        (Section::AbiLinkerFault, "conflicting-leaf-weight") => (
            Validator::WeightedStateDeclarations,
            Change::AddConflictingWeightedDeclaration,
            "DuplicateStateTreeLeaf(Announcement)",
        ),
        (Section::AbiLinkerFault, "candidate-outside-bounds") => (
            Validator::CompleteStateBounds,
            Change::LowerCompleteTreeDepthCap,
            "StateTreeDepthExceeded",
        ),
        (Section::ProtocolReportFault, "wrong-schema") => (
            Validator::ProtocolEnvironment,
            Change::ChangeObservedSchema,
            "UnsupportedProtocolSchema",
        ),
        (Section::ProtocolReportFault, "wrong-environment") => (
            Validator::ProtocolEnvironment,
            Change::ChangeObservedNetwork,
            "EnvironmentBindingMismatch",
        ),
        (Section::ProtocolReportFault, "wrong-provenance") => (
            Validator::TranscriptProvenance,
            Change::AddUnexpectedProvenanceTopic,
            "TopicCensusIsNotTheExpectedCensus",
        ),
        (Section::ProtocolReportFault, "signing-response-bound-to-other-bytes") => (
            Validator::ScriptPathBinding,
            Change::ChangeSignatureBoundBytes,
            "ScriptPathTransactionMismatch",
        ),
        _ => return None,
    })
}

fn entry_refusal(observation: &MaturityFirstPartyObservation) -> bool {
    use MaturityFirstPartyObservation as Observation;
    matches!(
        observation,
        Observation::Transaction(
            TransactionRefusal::MaturitySuccessorMetadataChangedAfterSigning
                | TransactionRefusal::MaturityViewProgramNotReconstructed { .. }
        ) | Observation::UnsupportedProtocolSchema { .. }
            | Observation::EnvironmentBindingMismatch
            | Observation::Provenance(ProvenanceDefect::TopicCensusIsNotTheExpectedCensus)
            | Observation::ResponseShape(ResponseShapeDefect::ScriptPathTransactionMismatch)
    ) || matches!(observation, Observation::Link(error) if matches!(error.as_ref(),
        LinkRefusal::DuplicateStateTreeLeaf(StateLeafRole::Announcement)
        | LinkRefusal::StateTreeDepthExceeded { .. }))
}

fn entry_refusal_matches(
    change: MaturityFirstPartyChange,
    observed: &MaturityFirstPartyObservation,
) -> bool {
    use MaturityFirstPartyChange as Change;
    use MaturityFirstPartyObservation as Observation;
    match (change, observed) {
        (
            Change::ChangeAuthorizedSuccessorCycle,
            Observation::Transaction(
                TransactionRefusal::MaturitySuccessorMetadataChangedAfterSigning,
            ),
        )
        | (
            Change::OmitMetadataCommitmentBranch,
            Observation::Transaction(TransactionRefusal::MaturityViewProgramNotReconstructed {
                ..
            }),
        )
        | (Change::ChangeObservedNetwork, Observation::EnvironmentBindingMismatch)
        | (
            Change::AddUnexpectedProvenanceTopic,
            Observation::Provenance(ProvenanceDefect::TopicCensusIsNotTheExpectedCensus),
        )
        | (
            Change::ChangeSignatureBoundBytes,
            Observation::ResponseShape(ResponseShapeDefect::ScriptPathTransactionMismatch),
        ) => true,
        (Change::ChangeObservedSchema, Observation::UnsupportedProtocolSchema { offered }) => {
            *offered == NATIVE_PROTOCOL_SCHEMA + 1
        }
        (Change::AddConflictingWeightedDeclaration, Observation::Link(error)) => {
            matches!(
                error.as_ref(),
                LinkRefusal::DuplicateStateTreeLeaf(StateLeafRole::Announcement)
            )
        }
        (Change::LowerCompleteTreeDepthCap, Observation::Link(error)) => {
            matches!(error.as_ref(), LinkRefusal::StateTreeDepthExceeded { depth, maximum, .. }
                if *maximum > 0 && depth.checked_sub(1) == Some(*maximum))
        }
        _ => false,
    }
}

fn supplemental_refusal(observation: &MaturityFirstPartyObservation) -> bool {
    matches!(
        observation,
        MaturityFirstPartyObservation::Constructor(StateConstructorRefusal::ConflictingLeaf)
    ) || matches!(observation,
        MaturityFirstPartyObservation::StateLink(refusal)
            if matches!(refusal.as_ref(), StateLinkRefusal::MissingDefinition(_))
    )
}

fn validate_supplementary_comparison(
    case: &MaturityFirstPartyCase,
    row: &MaturitySafetyRow,
    discharge: MaturityFirstPartyDischarge,
) -> Result<ValidatedMaturityFirstPartyEvidence, MaturityFirstPartyRefusal> {
    use MaturityFirstPartyChange as Change;
    use linker::StateLinkSymbol as Symbol;
    if supplemental_discharge(case.section, case.name) != Some(discharge) {
        return Err(MaturityFirstPartyRefusal::ScenarioNotConstructible);
    }
    let comparison = if entry_discharge(case.section, case.name).is_some() {
        MaturityRetainedComparison::Entries(Box::new(MaturityEntryComparisonTestStaging::new(
            discharge.change,
        )?))
    } else {
        match discharge.change {
            Change::ChangeRetainedStaticLeaf | Change::AddRetainedStaticLeaf => {
                MaturityRetainedComparison::Constructor(Box::new(
                    MaturityConstructorComparisonTestStaging::new(discharge.change)?,
                ))
            }
            change => {
                let symbol = match change {
                    Change::OmitConsumedMetadataSchema => Symbol::MetadataSchema,
                    Change::OmitConsumedMinimumLead => Symbol::MaturityLeadMin,
                    Change::OmitConsumedOperator => Symbol::CommittedOperatorKey,
                    _ => return Err(MaturityFirstPartyRefusal::ChangeChangedNothing),
                };
                MaturityRetainedComparison::Symbols(Box::new(
                    MaturitySymbolResolutionTestStaging::new(symbol)?,
                ))
            }
        }
    };
    let observed = comparison.observe()?;
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
        comparison: Some(comparison),
    })
}

/// A boundary retaining an observable obligation, including entries still required.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityRetainedBoundary {
    /// Decode the request's offered cycle encoding.
    RequestDecoding,
    /// Compare an offered successor with the finalized successor.
    SuccessorComparison,
    /// Compare the selected nonce with the canonical search result.
    NonceSearch,
    /// Reconstruct the complete tree including the metadata leaf.
    CompleteTree,
    /// Classify the objects and flows offered to the ABI.
    AbiClassification,
    /// Validate the offered coordinator, positions and cardinalities.
    AbiRoleValidation,
    /// Validate weighted leaf declarations.
    WeightedTreeInput,
    /// Check the complete linked tree and resource bounds.
    LinkedBounds,
    /// Read bounded newline-terminated requests.
    RequestFraming,
    /// Decode the offered JSON record.
    RequestJson,
    /// Refuse unknown request fields.
    RequestFields,
    /// Bind the offered protocol revision.
    SchemaBinding,
    /// Bind the observed environment.
    EnvironmentBinding,
    /// Bind the expected provenance.
    ProvenanceBinding,
    /// Bind the signing response to the requested transaction.
    SigningBinding,
    /// Compare static roots when staging cannot produce a usable pair.
    ConstructorContinuity,
    /// Resolve symbols when staging cannot produce a usable pair.
    SymbolResolution,
}

impl MaturityRetainedBoundary {
    /// The observable boundary; an absent entry is explicitly marked required.
    #[must_use]
    pub const fn entry_point(self) -> &'static str {
        match self {
            Self::RequestDecoding => "request decoding: required typed cycle decoder",
            Self::SuccessorComparison => "OperatorAuthorizedMaturityAnnouncement::check_offered",
            Self::NonceSearch => "CandidateStateConstructor::derive: canonical search comparison",
            Self::CompleteTree => {
                "PublicMaturityStateView::validate: complete program reconstruction"
            }
            Self::AbiClassification => {
                "ABI classification: required transaction::validate_maturity_abi_classification"
            }
            Self::AbiRoleValidation => {
                "ABI role validation: required transaction::validate_maturity_abi_roles"
            }
            Self::WeightedTreeInput => "TaptreeInput<StateLeafRole>::new",
            Self::LinkedBounds => "StateLinkedTaptree::bind: complete tree and resource bounds",
            Self::RequestFraming => "RequestReader.read",
            Self::RequestJson => "request JSON decoding",
            Self::RequestFields => "require_execution_request_fields",
            Self::SchemaBinding => "compare_environment: protocol schema binding",
            Self::EnvironmentBinding => "compare_environment: environment binding",
            Self::ProvenanceBinding => "ExecutionTranscript::provenance_agreement",
            Self::SigningBinding => "bind_script_path_response",
            Self::ConstructorContinuity => "CandidateStateConstructor::continuity",
            Self::SymbolResolution => "resolve_state_census",
        }
    }
}

/// A formal absence-of-coordinate closure, limited to ordering of unchanged contents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityAbsenceOfCoordinate {
    /// Exchanging children leaves the committed branch and paths unchanged.
    BranchOrder,
    /// Reordering declarations leaves canonical assembly unchanged.
    SourceDeclarationOrder,
}

impl MaturityAbsenceOfCoordinate {
    /// Why no malformed committed coordinate exists, and the argument's limit.
    #[must_use]
    pub const fn argument(self) -> &'static str {
        match self {
            Self::BranchOrder => {
                "absence of coordinate: sorted child hashes erase offered order; topology and leaf changes excluded"
            }
            Self::SourceDeclarationOrder => {
                "absence of coordinate: canonical assembly erases declaration order; topology and leaf changes excluded"
            }
        }
    }
}

/// Actual order variants and their canonical assembly, retained for the scope argument.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityOrderNormalization {
    first: StateStaticSubtree,
    reversed: StateStaticSubtree,
    declarations: Vec<StateLeafRole>,
    assembly: linker::DeterministicTaptree<StateLeafRole>,
    reversed_assembly: linker::DeterministicTaptree<StateLeafRole>,
}

impl MaturityOrderNormalization {
    fn new() -> Result<Self, MaturityFirstPartyRefusal> {
        let staged = MaturityConstructorComparisonTestStaging::new(
            MaturityFirstPartyChange::AddRetainedStaticLeaf,
        )?;
        let first = staged.malformed.static_subtree().clone();
        let StateStaticNode::Branch(left, right) = first.tree() else {
            return Err(MaturityFirstPartyRefusal::ScenarioNotConstructible);
        };
        let target = &substrate()?.target;
        let reversed = StateStaticSubtree::new(
            target,
            Some(StateStaticNode::Branch(right.clone(), left.clone())),
        )
        .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
        let declarations: Vec<_> = first.leaves().iter().map(|entry| entry.leaf.role).collect();
        let build_tree = |roles: Vec<StateLeafRole>| {
            let input = linker::state_static_taptree_input(
                roles,
                target.definition().leaf_version(),
                std::num::NonZeroU32::MIN.saturating_add(127),
            )
            .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)?;
            linker::assemble_state_static(&input)
                .map_err(|_| MaturityFirstPartyRefusal::ScenarioNotConstructible)
        };
        let assembly = build_tree(declarations.clone())?;
        let reversed_assembly = build_tree(declarations.iter().rev().copied().collect())?;
        let proof = Self {
            first,
            reversed,
            declarations,
            assembly,
            reversed_assembly,
        };
        proof.validate()?;
        Ok(proof)
    }

    fn validate(&self) -> Result<(), MaturityFirstPartyRefusal> {
        let leaves = |tree: &StateStaticSubtree| {
            tree.leaves()
                .iter()
                .map(|entry| {
                    (
                        entry.identity,
                        (entry.leaf.clone(), entry.hash, entry.siblings.clone()),
                    )
                })
                .collect::<std::collections::BTreeMap<_, _>>()
        };
        if self.first.tree() == self.reversed.tree()
            || self.first.root() != self.reversed.root()
            || leaves(&self.first) != leaves(&self.reversed)
            || self.declarations.len() < 2
            || self.assembly != self.reversed_assembly
        {
            return Err(MaturityFirstPartyRefusal::ScenarioNotConstructible);
        }
        Ok(())
    }
}

/// A valid retained construction and protocol opening, without a claimed malformed sibling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityRetainedSubject {
    announcement: Box<FinalizedMaturityAnnouncement>,
    opening: target_elements_conformance::protocol::HandshakeRequest,
}

impl MaturityRetainedSubject {
    /// Construction retaining its request, view, ABI and successor.
    #[must_use]
    pub const fn announcement(&self) -> &FinalizedMaturityAnnouncement {
        &self.announcement
    }

    /// The typed protocol opening retained for request-boundary work.
    #[must_use]
    pub const fn opening(&self) -> &target_elements_conformance::protocol::HandshakeRequest {
        &self.opening
    }
}

/// The supplementary standing, separate from the historical evidence-plan classification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaturitySupplementaryDisposition {
    /// The exact pair was accepted and refused at its owning entry.
    Discharged(Box<ValidatedMaturityFirstPartyEvidence>),
    /// Canonical assembly leaves no coordinate for the stated order fault.
    OutOfContract {
        /// Typed formal closure, never an absent-entry argument.
        reason: MaturityAbsenceOfCoordinate,
        /// Retained inputs and canonical outputs supporting the argument.
        proof: Box<MaturityOrderNormalization>,
    },
    /// Observable work remains at the named boundary.
    Retained {
        /// Where the exact fault must be compared.
        boundary: MaturityRetainedBoundary,
        /// The valid subject retained for subsequent staging.
        subject: Box<MaturityRetainedSubject>,
        /// A failed attempted comparison, preserved rather than converted into credit.
        attempted: Option<Box<MaturityFirstPartyRefusal>>,
    },
}

/// One keyed disposition preserving the original mechanism and relation link.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturitySupplementaryRow {
    section: MaturitySafetySection,
    name: &'static str,
    previous: MaturityCarriedReason,
    relation: MaturityRelationStanding,
    disposition: MaturitySupplementaryDisposition,
}

impl MaturitySupplementaryRow {
    /// The complete identity; row names alone are not unique.
    #[must_use]
    pub const fn key(&self) -> (MaturitySafetySection, &'static str) {
        (self.section, self.name)
    }

    /// The inherited reason, including the distinct deferred-owner variant.
    #[must_use]
    pub const fn previous(&self) -> MaturityCarriedReason {
        self.previous
    }

    /// The existing published relation or its explicit non-link.
    #[must_use]
    pub const fn relation(&self) -> &MaturityRelationStanding {
        &self.relation
    }

    /// Evidence, formal argument, or retained observable obligation.
    #[must_use]
    pub const fn disposition(&self) -> &MaturitySupplementaryDisposition {
        &self.disposition
    }
}

fn retained_boundary(case: &MaturityFirstPartyCase) -> MaturityRetainedBoundary {
    use MaturityRetainedBoundary as Boundary;
    use MaturitySafetySection as Section;
    match (case.section, case.name) {
        (Section::WindowFault, "malformed-cycle-encoding") => Boundary::RequestDecoding,
        (Section::WindowFault, _) => Boundary::SuccessorComparison,
        (Section::MetadataFault, _) => Boundary::NonceSearch,
        (Section::PredecessorConstructorFault, "metadata-leaf-missing") => Boundary::CompleteTree,
        (Section::PredecessorConstructorFault, _) => Boundary::ConstructorContinuity,
        (Section::AbsenceFault, _) => Boundary::AbiClassification,
        (Section::AbiLinkerFault, "conflicting-leaf-weight") => Boundary::WeightedTreeInput,
        (Section::AbiLinkerFault, "candidate-outside-bounds") => Boundary::LinkedBounds,
        (Section::AbiLinkerFault, name) if name.starts_with("unresolved-") => {
            Boundary::SymbolResolution
        }
        (Section::AbiLinkerFault, _) => Boundary::AbiRoleValidation,
        (_, "blank-request" | "oversized-request" | "unterminated-request") => {
            Boundary::RequestFraming
        }
        (_, "malformed-json") => Boundary::RequestJson,
        (_, "unknown-request-field") => Boundary::RequestFields,
        (_, "wrong-schema") => Boundary::SchemaBinding,
        (_, "wrong-environment") => Boundary::EnvironmentBinding,
        (_, "wrong-provenance") => Boundary::ProvenanceBinding,
        _ => Boundary::SigningBinding,
    }
}

/// Derive dispositions for exactly the inherited first-party-required keys.
///
/// Comparison discharges remain distinct from the historical 78 admitted answers.
/// Production derives the validated ledger once. The evidence fold checks its
/// required keys and previous reasons before applying the retained evidence.
///
/// # Errors
/// Returns failure to retain a valid subject or validate a normalization argument.
/// Failed comparison attempts instead retain their boundary and exact failure.
///
/// # Panics
/// Panics only under the finalized construction invariants documented by
/// [`validate_maturity_first_party`], which a linked bundle cannot violate.
pub fn maturity_first_party_dispositions()
-> Result<Vec<MaturitySupplementaryRow>, MaturityFirstPartyRefusal> {
    derive_first_party_dispositions(true)
}

fn derive_first_party_dispositions(
    include_entries: bool,
) -> Result<Vec<MaturitySupplementaryRow>, MaturityFirstPartyRefusal> {
    let subject = MaturityRetainedSubject {
        announcement: Box::new(honest_finalized(substrate()?)?),
        opening: target_elements_conformance::protocol::HandshakeRequest::default(),
    };
    let normalization = MaturityOrderNormalization::new()?;
    maturity_first_party_cases()
        .into_iter()
        .filter_map(|case| case.carried_reason().map(|previous| (case, previous)))
        .map(|(case, previous)| {
            let row = matrix_row(case.section, case.name).ok_or(
                MaturityFirstPartyRefusal::RowNotInTheMatrix(case.section, case.name),
            )?;
            let retain = |attempted| MaturitySupplementaryDisposition::Retained {
                boundary: retained_boundary(&case),
                subject: Box::new(subject.clone()),
                attempted,
            };
            let disposition = if let Some(discharge) =
                supplemental_discharge(case.section, case.name).filter(|_| {
                    include_entries || entry_discharge(case.section, case.name).is_none()
                }) {
                let offered = MaturityFirstPartyCase {
                    disposition: MaturityFirstPartyDisposition::Discharge(discharge),
                    ..case
                };
                match validate_maturity_first_party(&offered) {
                    Ok(evidence) => {
                        MaturitySupplementaryDisposition::Discharged(Box::new(evidence))
                    }
                    Err(refusal) => retain(Some(Box::new(refusal))),
                }
            } else if case.section == MaturitySafetySection::TotalityFault {
                let reason = match case.name {
                    "caller-supplied-branch-order" => MaturityAbsenceOfCoordinate::BranchOrder,
                    "source-order-dependent-tree" => {
                        MaturityAbsenceOfCoordinate::SourceDeclarationOrder
                    }
                    _ => return Err(MaturityFirstPartyRefusal::ScenarioNotConstructible),
                };
                MaturitySupplementaryDisposition::OutOfContract {
                    reason,
                    proof: Box::new(normalization.clone()),
                }
            } else {
                retain(None)
            };
            Ok(MaturitySupplementaryRow {
                section: case.section,
                name: case.name,
                previous,
                relation: row.relation().clone(),
                disposition,
            })
        })
        .collect()
}

/// Supplementary evidence derived here or admitted against complete keyed recomputation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ValidatedMaturitySupplementaryLedger(Vec<MaturitySupplementaryRow>);

impl ValidatedMaturitySupplementaryLedger {
    pub(crate) fn rows(&self) -> &[MaturitySupplementaryRow] {
        &self.0
    }
}

/// Derive and admit the production ledger once, retaining its validated evidence.
pub(crate) fn derive_maturity_first_party_dispositions()
-> Result<ValidatedMaturitySupplementaryLedger, MaturityFirstPartyRefusal> {
    maturity_first_party_dispositions().map(ValidatedMaturitySupplementaryLedger)
}

/// Reconstruct the bounded consolidation before existing-entry admission.
#[cfg(test)]
pub(crate) fn derive_close_safety_dispositions()
-> Result<ValidatedMaturitySupplementaryLedger, MaturityFirstPartyRefusal> {
    derive_first_party_dispositions(false).map(ValidatedMaturitySupplementaryLedger)
}

/// Test-held reference check of every offered key and complete retained evidence.
#[cfg(test)]
pub(crate) fn admit_maturity_first_party_dispositions(
    offered: Vec<MaturitySupplementaryRow>,
) -> Result<ValidatedMaturitySupplementaryLedger, MaturityFirstPartyRefusal> {
    let expected = maturity_first_party_dispositions()?;
    let keyed = |rows: &[MaturitySupplementaryRow]| {
        rows.iter()
            .map(|row| (row.key(), row.clone()))
            .collect::<std::collections::BTreeMap<_, _>>()
    };
    let actual = keyed(&offered);
    if actual.len() != offered.len() || actual != keyed(&expected) {
        return Err(MaturityFirstPartyRefusal::SupplementaryLedgerMismatch);
    }
    Ok(ValidatedMaturitySupplementaryLedger(offered))
}

#[cfg(test)]
mod supplementary_tests {
    #[test]
    fn supplementary_join_refuses_missing_duplicate_surplus_and_misbound_evidence() {
        use super::{MaturityFirstPartyRefusal, admit_maturity_first_party_dispositions};
        let honest = ledger();
        assert!(admit_maturity_first_party_dispositions(honest.clone()).is_ok());
        let refuse = |offered| {
            assert_eq!(
                admit_maturity_first_party_dispositions(offered),
                Err(MaturityFirstPartyRefusal::SupplementaryLedgerMismatch),
            );
        };
        let mut missing = honest.clone();
        missing.pop();
        refuse(missing);
        let mut duplicate = honest.clone();
        duplicate.push(honest[0].clone());
        refuse(duplicate);
        let mut surplus = honest.clone();
        let mut extra = honest[0].clone();
        extra.name = "unregistered-supplementary-row";
        surplus.push(extra);
        refuse(surplus);
        let mut misbound = honest.clone();
        let comparison_positions: Vec<_> = honest
            .iter()
            .enumerate()
            .filter_map(|(position, row)| {
                matches!(
                    row.disposition(),
                    super::MaturitySupplementaryDisposition::Discharged(_)
                )
                .then_some(position)
            })
            .collect();
        misbound[comparison_positions[0]].disposition =
            honest[comparison_positions[1]].disposition.clone();
        refuse(misbound);
        let mut wrong_section = honest.clone();
        wrong_section[0].section = super::MaturitySafetySection::Positive;
        refuse(wrong_section);
        let mut wrong_reason = honest;
        wrong_reason[0].previous = super::MaturityCarriedReason::TheValueIsDerivedNotAccepted;
        refuse(wrong_reason);
    }
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    fn ledger() -> Vec<MaturitySupplementaryRow> {
        maturity_first_party_dispositions().expect("valid retained subjects")
    }

    fn entry_pair(
        section: MaturitySafetySection,
        name: &str,
    ) -> MaturityEntryComparisonTestStaging {
        let (validator, change, class) = entry_discharge(section, name).expect("named entry");
        let pair = MaturityEntryComparisonTestStaging::new(change).expect("staged offers");
        let observed = pair
            .observe()
            .expect("control accepted and changed offer refused");
        assert!(entry_refusal_matches(change, &observed));
        let row = ledger()
            .into_iter()
            .find(|row| row.key() == (section, name))
            .expect("row");
        let MaturitySupplementaryDisposition::Discharged(evidence) = row.disposition() else {
            panic!("pair was not admitted: {:?}", row.disposition());
        };
        assert_eq!(evidence.validator(), validator);
        assert_eq!(evidence.observed(), &observed);
        assert_eq!(evidence.supplementary_class(), Some(class));
        assert_eq!(
            evidence.comparison(),
            Some(&MaturityRetainedComparison::Entries(Box::new(pair.clone())))
        );
        pair
    }

    #[test]
    fn entry_pair_successor_cycle() {
        let pair = entry_pair(
            MaturitySafetySection::WindowFault,
            "request-cycle-differs-from-successor",
        );
        let MaturityEntrySubjects::Successor {
            finalized, offers, ..
        } = pair.subjects
        else {
            panic!("successor pair")
        };
        assert_eq!(
            offers.control,
            finalized.construction().successor_metadata()
        );
        let mut restored = offers.malformed;
        assert_ne!(restored.maturity, offers.control.maturity);
        restored.maturity = offers.control.maturity;
        assert_eq!(restored, offers.control);
    }

    #[test]
    fn entry_pair_metadata_omission() {
        let pair = entry_pair(
            MaturitySafetySection::PredecessorConstructorFault,
            "metadata-leaf-missing",
        );
        let MaturityEntrySubjects::CompleteProgram {
            offers,
            static_root,
            static_program,
        } = pair.subjects
        else {
            panic!("program pair")
        };
        assert_eq!(
            static_root,
            *substrate()
                .expect("substrate")
                .bundle
                .static_subtree()
                .root()
        );
        assert_eq!(
            static_program,
            static_only_program(&static_root).expect("static-only commitment")
        );
        assert_eq!(offers.malformed.predecessor_program(), static_program);
        let world = MaturityWorld {
            program: offers.control.predecessor_program().to_vec(),
            ..substrate().expect("substrate").honest.clone()
        };
        assert_eq!(
            offers.control,
            PublicMaturityStateView::new(
                statements(substrate().expect("substrate"), &world).expect("view statements"),
            )
            .expect("control view")
        );
        assert_eq!(
            offers.control.predecessor_metadata(),
            offers.malformed.predecessor_metadata()
        );
        assert_eq!(
            offers.control.predecessor_representation_nonce(),
            offers.malformed.predecessor_representation_nonce()
        );
        assert_eq!(
            offers.control.accepted_linked_bundle(),
            offers.malformed.accepted_linked_bundle()
        );
        assert_eq!(
            offers.control.current_state_outpoint(),
            offers.malformed.current_state_outpoint()
        );
        assert_eq!(
            offers.control.current_root_binding(),
            offers.malformed.current_root_binding()
        );
        assert_eq!(offers.control.asset(), offers.malformed.asset());
        assert_eq!(offers.control.value(), offers.malformed.value());
    }

    #[test]
    fn entry_pair_conflicting_weight() {
        let pair = entry_pair(
            MaturitySafetySection::AbiLinkerFault,
            "conflicting-leaf-weight",
        );
        let MaturityEntrySubjects::Weights(offers) = pair.subjects else {
            panic!("weight pair")
        };
        assert_eq!(offers.control, offers.malformed[..1]);
        assert_eq!(offers.malformed.len(), 2);
        assert_eq!(offers.malformed[0].leaf(), offers.malformed[1].leaf());
        assert_ne!(offers.malformed[0].weight(), offers.malformed[1].weight());
        let accepted_change = MaturityEntryComparisonTestStaging {
            change: MaturityFirstPartyChange::AddConflictingWeightedDeclaration,
            subjects: MaturityEntrySubjects::Weights(Box::new(EntryPair {
                control: offers.control,
                malformed: vec![offers.malformed[1]],
            })),
        };
        assert_eq!(
            accepted_change.observe(),
            Err(MaturityFirstPartyRefusal::ChangedInputWasAccepted)
        );
    }

    #[test]
    fn entry_pair_depth_cap() {
        let pair = entry_pair(
            MaturitySafetySection::AbiLinkerFault,
            "candidate-outside-bounds",
        );
        let MaturityEntrySubjects::Bounds { tree, caps, .. } = pair.subjects else {
            panic!("bounds pair")
        };
        assert_eq!(caps.control.get(), tree.depth() + 1);
        assert_eq!(caps.malformed.get(), tree.depth());
        assert!(caps.malformed.get() > 0);
    }

    #[test]
    fn entry_pair_schema() {
        let pair = entry_pair(MaturitySafetySection::ProtocolReportFault, "wrong-schema");
        let MaturityEntrySubjects::Environment { offers, .. } = pair.subjects else {
            panic!("environment pair")
        };
        let mut restored = offers.malformed.clone();
        assert_ne!(restored.schema, offers.control.schema);
        restored.schema = offers.control.schema;
        assert_eq!(restored, offers.control);
    }

    #[test]
    fn entry_pair_environment() {
        let pair = entry_pair(
            MaturitySafetySection::ProtocolReportFault,
            "wrong-environment",
        );
        let MaturityEntrySubjects::Environment { offers, .. } = pair.subjects else {
            panic!("environment pair")
        };
        let mut restored = offers.malformed.clone();
        assert_ne!(restored.network_id, offers.control.network_id);
        restored.network_id = offers.control.network_id;
        assert_eq!(restored, offers.control);
    }

    #[test]
    fn entry_pair_provenance() {
        let pair = entry_pair(
            MaturitySafetySection::ProtocolReportFault,
            "wrong-provenance",
        );
        let MaturityEntrySubjects::Provenance {
            configuration,
            offers,
            ..
        } = pair.subjects
        else {
            panic!("provenance pair")
        };
        assert!(configuration.expected_provenance().is_some());
        let mut restored = offers.malformed.clone();
        assert!(restored.included_local_topics.remove("fix/unexpected"));
        assert_eq!(restored, offers.control);
    }

    #[test]
    fn entry_pair_signature_binding() {
        let pair = entry_pair(
            MaturitySafetySection::ProtocolReportFault,
            "signing-response-bound-to-other-bytes",
        );
        let MaturityEntrySubjects::Signing {
            subject, offers, ..
        } = pair.subjects
        else {
            panic!("signing pair")
        };
        let native_protocol::OperationSubject::ScriptPathSigning(subject) = subject.as_ref() else {
            panic!("script-path subject")
        };
        assert_eq!(
            offers.control.signature_bound_to.as_deref(),
            Some(subject.finalized_transaction.as_slice())
        );
        let mut restored = offers.malformed.clone();
        let before = offers
            .control
            .signature_bound_to
            .as_ref()
            .expect("control bytes");
        let after = restored.signature_bound_to.as_ref().expect("changed bytes");
        assert_eq!(before.len(), after.len());
        assert_eq!(
            before
                .iter()
                .zip(after)
                .filter(|(left, right)| left != right)
                .count(),
            1
        );
        restored
            .signature_bound_to
            .clone_from(&offers.control.signature_bound_to);
        assert_eq!(restored, offers.control);
    }

    fn change_pair<T: Clone>(pair: &mut EntryPair<T>, reverse: bool) {
        if reverse {
            std::mem::swap(&mut pair.control, &mut pair.malformed);
        } else {
            pair.malformed.clone_from(&pair.control);
        }
    }

    #[test]
    fn entry_pairs_refuse_copied_objects_invalid_controls_and_wrong_classes() {
        let pairs: Vec<_> = ledger()
            .into_iter()
            .filter_map(|row| {
                let MaturitySupplementaryDisposition::Discharged(evidence) = row.disposition else {
                    return None;
                };
                let Some(MaturityRetainedComparison::Entries(pair)) = evidence.comparison else {
                    return None;
                };
                Some(*pair)
            })
            .collect();
        assert_eq!(pairs.len(), 8);
        for original in pairs {
            for reverse in [false, true] {
                let mut changed = original.clone();
                match &mut changed.subjects {
                    MaturityEntrySubjects::Successor { offers, .. } => change_pair(offers, reverse),
                    MaturityEntrySubjects::CompleteProgram { offers, .. } => {
                        change_pair(offers, reverse);
                    }
                    MaturityEntrySubjects::Weights(offers) => change_pair(offers, reverse),
                    MaturityEntrySubjects::Bounds { caps, .. } => change_pair(caps, reverse),
                    MaturityEntrySubjects::Environment { offers, .. } => {
                        change_pair(offers, reverse);
                    }
                    MaturityEntrySubjects::Provenance { offers, .. } => {
                        change_pair(offers, reverse);
                    }
                    MaturityEntrySubjects::Signing { offers, .. } => change_pair(offers, reverse),
                }
                let result = changed.observe();
                if reverse {
                    assert!(
                        matches!(result, Err(MaturityFirstPartyRefusal::ControlWasRefused(_))),
                        "{result:?}"
                    );
                } else {
                    assert_eq!(result, Err(MaturityFirstPartyRefusal::ChangeChangedNothing));
                }
            }
            let other_environment_change = match original.change {
                MaturityFirstPartyChange::ChangeObservedSchema => {
                    Some(MaturityFirstPartyChange::ChangeObservedNetwork)
                }
                MaturityFirstPartyChange::ChangeObservedNetwork => {
                    Some(MaturityFirstPartyChange::ChangeObservedSchema)
                }
                _ => None,
            };
            let mut changed = original;
            if let Some(change) = other_environment_change {
                changed.change = change;
                assert!(matches!(
                    changed.observe(),
                    Err(MaturityFirstPartyRefusal::RefusalNamesAnotherClass(_))
                ));
            }
            changed.change = MaturityFirstPartyChange::OmitConsumedOperator;
            assert!(matches!(
                changed.observe(),
                Err(MaturityFirstPartyRefusal::RefusalNamesAnotherClass(_))
            ));
        }
    }

    #[test]
    fn safety_disposition_record_matches_the_typed_ledger() {
        let backlog = include_str!("../../../plans/backlog.md");
        let table = backlog
            .split("**Table (First-party Safety rows)**")
            .nth(1)
            .expect("record table")
            .split("**Result (First-party census)**")
            .next()
            .expect("table end");
        let recorded: BTreeMap<_, _> = table
            .lines()
            .filter(|line| line.starts_with("| `"))
            .map(|line| {
                let columns: Vec<_> = line.split('|').map(str::trim).collect();
                assert_eq!(columns.len(), 7);
                assert_ne!(columns[3], "");
                assert_ne!(columns[5], "");
                (
                    columns[1].trim_matches('`').to_owned(),
                    (columns[2].trim_matches('`'), columns[4]),
                )
            })
            .collect();
        let rows = ledger();
        assert_eq!(recorded.len(), 43);
        assert_eq!(
            table.lines().filter(|line| line.starts_with("| `")).count(),
            43
        );
        for row in &rows {
            let key = format!("{} / {}", row.section.section(), row.name);
            let (entry, disposition) = recorded.get(&key).expect("exact row key");
            match row.disposition() {
                MaturitySupplementaryDisposition::Discharged(evidence) => {
                    assert_eq!(*disposition, "comparison");
                    assert_eq!(*entry, evidence.validator().entry_point());
                }
                MaturitySupplementaryDisposition::OutOfContract { reason, .. } => {
                    assert_eq!(*disposition, "absence of coordinate");
                    assert_eq!(
                        *entry,
                        match reason {
                            MaturityAbsenceOfCoordinate::BranchOrder => "StateStaticSubtree::new",
                            MaturityAbsenceOfCoordinate::SourceDeclarationOrder =>
                                "assemble_state_static",
                        }
                    );
                }
                MaturitySupplementaryDisposition::Retained { boundary, .. } => {
                    assert_eq!(*disposition, "retained");
                    assert_eq!(*entry, boundary.entry_point());
                }
            }
        }
    }

    #[test]
    fn supplementary_safety_keys_and_census_are_exact() {
        let rows = ledger();
        let inherited: BTreeMap<_, _> = maturity_first_party_cases()
            .into_iter()
            .filter_map(|case| {
                case.carried_reason()
                    .map(|reason| ((case.section(), case.name()), reason))
            })
            .collect();
        let actual: BTreeMap<_, _> = rows.iter().map(|row| (row.key(), row.previous())).collect();
        assert_eq!(actual, inherited);
        assert_eq!(actual.len(), rows.len());
        assert_eq!(rows.len(), 43);
        let mut census = [0; 3];
        for row in &rows {
            assert_eq!(
                row.relation(),
                matrix_row(row.section, row.name).expect("row").relation()
            );
            match row.disposition() {
                MaturitySupplementaryDisposition::Discharged(_) => census[0] += 1,
                MaturitySupplementaryDisposition::OutOfContract { .. } => census[1] += 1,
                MaturitySupplementaryDisposition::Retained { .. } => census[2] += 1,
            }
        }
        assert_eq!(census, [13, 2, 28]);
        assert_eq!(census.iter().sum::<usize>(), 43);
        assert_eq!(
            inherited
                .values()
                .filter(|reason| matches!(
                    reason,
                    MaturityCarriedReason::OwnerLocatedDischargeDeferred(_)
                ))
                .count(),
            0
        );
        assert_eq!(
            inherited
                .values()
                .filter(|reason| !matches!(
                    reason,
                    MaturityCarriedReason::OwnerLocatedDischargeDeferred(_)
                ))
                .count(),
            43
        );
    }

    #[test]
    fn supplementary_comparisons_retain_exact_objects_and_refusals() {
        let rows = ledger();
        let mut constructors = 0;
        let mut symbols = BTreeSet::new();
        for row in &rows {
            let MaturitySupplementaryDisposition::Discharged(evidence) = row.disposition() else {
                continue;
            };
            assert_eq!((evidence.section(), evidence.name()), row.key());
            let comparison = evidence.comparison().expect("an actual retained pair");
            assert_eq!(
                &comparison.observe().expect("pair admission"),
                evidence.observed()
            );
            match comparison {
                MaturityRetainedComparison::Entries(pair) => {
                    assert_eq!(pair.change(), evidence.change());
                    assert!(entry_refusal_matches(pair.change(), evidence.observed()));
                }
                MaturityRetainedComparison::Constructor(pair) => {
                    constructors += 1;
                    assert_ne!(pair.control(), pair.malformed());
                    assert_ne!(
                        pair.control().static_subtree().root(),
                        pair.malformed().static_subtree().root()
                    );
                    assert_eq!(
                        pair.control().encoded_metadata().semantic,
                        pair.malformed().encoded_metadata().semantic
                    );
                    assert_eq!(
                        evidence.validator(),
                        MaturityFirstPartyValidator::ConstructorContinuity
                    );
                    assert_eq!(
                        evidence.observed(),
                        &MaturityFirstPartyObservation::Constructor(
                            StateConstructorRefusal::ConflictingLeaf,
                        )
                    );
                    let before = pair.control().static_subtree().leaves();
                    let after = pair.malformed().static_subtree().leaves();
                    if row.name == "extra-escape-leaf" {
                        assert_eq!(after.len(), before.len() + 1);
                        for leaf in before {
                            assert!(
                                after.iter().any(|other| other.identity == leaf.identity
                                    && other.leaf == leaf.leaf)
                            );
                        }
                    } else {
                        assert_eq!(after.len(), before.len());
                        assert_eq!(
                            before
                                .iter()
                                .zip(after)
                                .filter(|(left, right)| left.leaf != right.leaf)
                                .count(),
                            1
                        );
                    }
                }
                MaturityRetainedComparison::Symbols(pair) => {
                    assert!(symbols.insert(pair.missing()));
                    assert_eq!(pair.control().len(), pair.malformed().len() + 1);
                    assert!(pair.control().definition(pair.missing()).is_some());
                    assert!(pair.malformed().definition(pair.missing()).is_none());
                    for (&key, definition) in pair.malformed().definitions() {
                        assert_eq!(pair.control().definition(key), Some(definition));
                    }
                    assert_eq!(
                        evidence.validator(),
                        MaturityFirstPartyValidator::SymbolResolution
                    );
                    assert_eq!(
                        evidence.observed(),
                        &MaturityFirstPartyObservation::StateLink(Box::new(
                            StateLinkRefusal::MissingDefinition(pair.missing()),
                        ))
                    );
                }
            }
        }
        assert_eq!(constructors, 2);
        assert_eq!(
            symbols,
            BTreeSet::from([
                linker::StateLinkSymbol::MetadataSchema,
                linker::StateLinkSymbol::MaturityLeadMin,
                linker::StateLinkSymbol::CommittedOperatorKey,
            ])
        );
    }

    #[test]
    fn supplementary_admission_refuses_copied_pairs_and_wrong_bindings() {
        let mut constructor = MaturityConstructorComparisonTestStaging::new(
            MaturityFirstPartyChange::ChangeRetainedStaticLeaf,
        )
        .expect("staging");
        constructor.malformed = constructor.control.clone();
        assert_eq!(
            constructor.observe(),
            Err(MaturityFirstPartyRefusal::ChangeChangedNothing)
        );
        let pair =
            MaturitySymbolResolutionTestStaging::new(linker::StateLinkSymbol::MetadataSchema)
                .expect("staging");
        let mut copied = pair.clone();
        copied.malformed = copied.control.clone();
        assert_eq!(
            copied.observe(),
            Err(MaturityFirstPartyRefusal::ChangeChangedNothing)
        );
        let mut refused_control = pair.clone();
        refused_control.control = pair.malformed.clone();
        refused_control.malformed = pair.control.clone();
        assert!(matches!(
            refused_control.observe(),
            Err(MaturityFirstPartyRefusal::ControlWasRefused(_))
        ));
        let mut wrong_symbol = pair;
        wrong_symbol.missing = linker::StateLinkSymbol::CommittedOperatorKey;
        assert!(matches!(
            wrong_symbol.observe(),
            Err(MaturityFirstPartyRefusal::RefusalNamesAnotherClass(_))
        ));
        let mut accepted =
            MaturitySymbolResolutionTestStaging::new(linker::StateLinkSymbol::MetadataSchema)
                .expect("staging");
        accepted.malformed = linker::StateDefinitionCensus::default();
        for (&symbol, definition) in accepted.control.definitions() {
            let value = match definition.value() {
                linker::StateSymbolValue::MetadataSchema(value) => {
                    linker::StateSymbolValue::MetadataSchema(value + 1)
                }
                value => value.clone(),
            };
            accepted
                .malformed
                .define(symbol, value, definition.origin())
                .expect("unique definition");
        }
        assert_eq!(
            accepted.observe(),
            Err(MaturityFirstPartyRefusal::ChangedInputWasAccepted)
        );
        let section = MaturitySafetySection::AbiLinkerFault;
        let mut case = MaturityFirstPartyCase {
            section,
            name: "unresolved-metadata-schema-symbol",
            disposition: MaturityFirstPartyDisposition::Discharge(
                supplemental_discharge(section, "unresolved-operator-symbol").expect("case"),
            ),
        };
        assert_eq!(
            validate_maturity_first_party(&case),
            Err(MaturityFirstPartyRefusal::ScenarioNotConstructible)
        );
        case.section = MaturitySafetySection::PredecessorConstructorFault;
        case.name = "wrong-static-subtree";
        assert_eq!(
            validate_maturity_first_party(&case),
            Err(MaturityFirstPartyRefusal::ScenarioNotConstructible)
        );
    }

    #[test]
    fn absence_of_coordinate_is_only_unchanged_content_ordering() {
        let rows = ledger();
        let mut names = BTreeSet::new();
        for row in &rows {
            let MaturitySupplementaryDisposition::OutOfContract { reason, proof } =
                row.disposition()
            else {
                continue;
            };
            names.insert(row.key());
            assert!(reason.argument().contains("absence of coordinate"));
            assert!(
                reason
                    .argument()
                    .contains("topology and leaf changes excluded")
            );
            assert_ne!(proof.first.tree(), proof.reversed.tree());
            assert_eq!(proof.first.root(), proof.reversed.root());
            for leaf in proof.first.leaves() {
                let reversed = proof
                    .reversed
                    .leaves()
                    .iter()
                    .find(|other| other.identity == leaf.identity)
                    .expect("same leaf");
                assert_eq!(leaf.leaf, reversed.leaf);
                assert_eq!(leaf.hash, reversed.hash);
                assert_eq!(leaf.siblings, reversed.siblings);
            }
            assert_eq!(proof.assembly, proof.reversed_assembly);
            assert_eq!(proof.assembly.recipes(), proof.reversed_assembly.recipes());
            proof.validate().expect("normalization");
            let mut altered = proof.as_ref().clone();
            altered.reversed = retained_constructor(substrate().expect("substrate"))
                .expect("constructor")
                .static_subtree()
                .clone();
            assert!(altered.validate().is_err());
        }
        assert_eq!(
            names,
            BTreeSet::from([
                (
                    MaturitySafetySection::TotalityFault,
                    "caller-supplied-branch-order"
                ),
                (
                    MaturitySafetySection::TotalityFault,
                    "source-order-dependent-tree"
                ),
            ])
        );
    }

    #[test]
    fn retained_obligations_have_valid_subjects_and_named_boundaries() {
        let rows = ledger();
        let mut retained = 0;
        let mut classification = 0;
        let mut roles = 0;
        for row in &rows {
            let MaturitySupplementaryDisposition::Retained {
                boundary,
                subject,
                attempted,
            } = row.disposition()
            else {
                continue;
            };
            retained += 1;
            assert!(attempted.is_none(), "failed staging: {attempted:?}");
            assert_ne!(boundary.entry_point(), "");
            subject
                .announcement()
                .check_offered(subject.announcement().protected())
                .expect("valid retained construction");
            assert_ne!(subject.announcement().protected_bytes().len(), 0);
            assert_eq!(subject.opening().schema, NATIVE_PROTOCOL_SCHEMA);
            match row.section {
                MaturitySafetySection::AbsenceFault => {
                    classification += 1;
                    assert_eq!(*boundary, MaturityRetainedBoundary::AbiClassification);
                    assert!(boundary.entry_point().contains("required transaction::"));
                }
                MaturitySafetySection::AbiLinkerFault
                    if matches!(
                        row.previous,
                        MaturityCarriedReason::LayoutIsNotAcceptedFromACaller
                    ) =>
                {
                    roles += 1;
                    assert_eq!(*boundary, MaturityRetainedBoundary::AbiRoleValidation);
                    assert!(boundary.entry_point().contains("required transaction::"));
                }
                _ => {}
            }
        }
        assert_eq!(retained, 28);
        assert_eq!(classification, 15);
        assert_eq!(roles, 6);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MaturityCarriedReason, MaturityFirstPartyCase, MaturityFirstPartyChange,
        MaturityFirstPartyDischarge, MaturityFirstPartyDisposition, MaturityFirstPartyRefusal,
        MaturityFirstPartyValidator, ValidatedMaturityFirstPartyEvidence,
        accepted_operation_omits_observation, canonical_branch_side, discharge,
        discharge_maturity_first_party, duplicate_leaf,
        infrastructure_response_carries_observation, is_pre_target, matrix_row,
        maturity_first_party_cases, metadata_encoding_refused, metadata_leaf_not_unspendable,
        missing_response, output_mutated_after_signing, relocation_not_applied,
        supplied_constructor_commits_another_program, validate_maturity_first_party,
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
        assert_eq!(discharged.len(), 40);

        // The layout rows are the census's largest carried group, and
        // they are carried for a reason that is not deferral.
        let layout = carried
            .iter()
            .filter(|reason| **reason == MaturityCarriedReason::LayoutIsNotAcceptedFromACaller)
            .count();
        assert_eq!(layout, 21);

        // Every deferred row names the owner it waits on, so a reader
        // can tell outstanding work from work nothing can do. The set is
        // the census's own rather than the type's, and it is empty:
        // every entry point this census once deferred to is driven, so a
        // row naming one of them would be outstanding work nobody had
        // noticed rather than a member that happens to exist. Asserted
        // as a set so that the row appears here rather than as a count
        // that a second change could restore.
        let owners: BTreeSet<_> = carried
            .iter()
            .filter_map(|reason| match reason {
                MaturityCarriedReason::OwnerLocatedDischargeDeferred(owner) => Some(*owner),
                _ => None,
            })
            .collect();
        assert_eq!(owners, BTreeSet::new());
    }

    #[test]
    fn each_mechanism_reason_stands_on_exactly_the_rows_it_names() {
        // The reasons that name a mechanism rather than a deferral,
        // checked against the census rather than against a remembered
        // figure. Each says what swallowed the row's change, and each
        // would be falsified by the tree moving under it: a leastness
        // check on an offer, a branch that stopped sorting its children,
        // a retained object arriving at the derivation, or a weight term
        // entering the typed leaf would each fail here.
        let mut named: BTreeMap<MaturityCarriedReason, Vec<&'static str>> = BTreeMap::new();
        for case in &maturity_first_party_cases() {
            if let Some(reason) = case.carried_reason() {
                named.entry(reason).or_default().push(case.name());
            }
        }
        let standing = |reason: &MaturityCarriedReason| named.get(reason).map(Vec::as_slice);
        assert_eq!(
            standing(&MaturityCarriedReason::LeastnessIsAPropertyOfTheSearchNotOfAnOffer),
            Some(["correct-semantic-metadata-with-noncanonical-representation-nonce"].as_slice()),
        );
        assert_eq!(
            standing(&MaturityCarriedReason::TheOfferedOrderIsNormalizedBeforeItIsCommitted),
            Some(
                [
                    "caller-supplied-branch-order",
                    "source-order-dependent-tree"
                ]
                .as_slice()
            ),
        );
        assert_eq!(
            standing(&MaturityCarriedReason::TheOwnerHasNoRetainedObjectToCompareTheOfferAgainst),
            Some(["wrong-static-subtree", "extra-escape-leaf"].as_slice()),
        );
        assert_eq!(
            standing(&MaturityCarriedReason::TheTypedInputCarriesNoTermTheChangeNames),
            Some(["conflicting-leaf-weight"].as_slice()),
        );

        // The two reasons this bite adds, on the rows whose change the
        // link swallows by making the artifact it checks. A resolution
        // that began accepting a census, or a bound that began taking an
        // offered candidate, would fail here.
        assert_eq!(
            standing(&MaturityCarriedReason::TheResolvedCensusIsResolvedByTheOwnerItself),
            Some(
                [
                    "unresolved-metadata-schema-symbol",
                    "unresolved-lead-bound-symbol",
                    "unresolved-operator-symbol"
                ]
                .as_slice()
            ),
        );
        assert_eq!(
            standing(&MaturityCarriedReason::TheBoundIsMeasuredOverWhatTheOwnerBuilt),
            Some(["candidate-outside-bounds"].as_slice()),
        );

        // The one response row carried beside the eight request rows,
        // for the reason they carry: the comparison that owns it runs
        // inside a binding no public entry reaches.
        assert!(
            named[&MaturityCarriedReason::TheOwningValidatorHasNoPublicEntry]
                .contains(&"signing-response-bound-to-other-bytes"),
        );

        // The one row this bite carries under a reason the census
        // already had: the metadata leaf is derived from the metadata
        // rather than accepted beside it, which is what the window rows
        // sharing this reason say about their own derived values.
        assert!(
            named[&MaturityCarriedReason::TheValueIsDerivedNotAccepted]
                .contains(&"metadata-leaf-missing"),
        );
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
        assert_eq!(discharged.len(), 40);
        assert_eq!(
            discharge_maturity_first_party().expect("the census discharges"),
            discharged,
        );

        // Every owning entry point was really driven, so a census that
        // quietly lost one would fail rather than report a smaller
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
    fn the_subtree_validation_does_not_discharge_a_metadata_encoding_row() {
        // Each of the four tests below performs §4.2's clause that a
        // refusal from another layer does not satisfy a row, against one
        // of this bite's owners. The wrong owner is given its own change
        // so that it really refuses: what fails is not the call but the
        // claim that its refusal answered this row, and the case is
        // refused for naming another class rather than passing on a
        // refusal it did not earn.
        let offered = case(
            MaturitySafetySection::MetadataFault,
            "omit-one-field",
            MaturityFirstPartyDisposition::Discharge(discharge(
                MaturityFirstPartyValidator::StaticSubtreeValidation,
                MaturityFirstPartyChange::OfferAStaticSubtreeWithoutTheOperationLeaf,
                metadata_encoding_refused,
                "MetadataEncodingRefused",
            )),
        );
        assert!(matches!(
            validate_maturity_first_party(&offered),
            Err(MaturityFirstPartyRefusal::RefusalNamesAnotherClass(_)),
        ));
    }

    #[test]
    fn the_metadata_decode_does_not_discharge_a_static_subtree_row() {
        let offered = case(
            MaturitySafetySection::AbiLinkerFault,
            "duplicate-tree-leaf",
            MaturityFirstPartyDisposition::Discharge(discharge(
                MaturityFirstPartyValidator::MetadataPatternDecode,
                MaturityFirstPartyChange::OmitOneMetadataField,
                duplicate_leaf,
                "DuplicateLeaf",
            )),
        );
        assert!(matches!(
            validate_maturity_first_party(&offered),
            Err(MaturityFirstPartyRefusal::RefusalNamesAnotherClass(_)),
        ));
    }

    #[test]
    fn the_leaf_pattern_does_not_discharge_the_branch_side_row() {
        let offered = case(
            MaturitySafetySection::TotalityFault,
            "metadata-child-on-wrong-side",
            MaturityFirstPartyDisposition::Discharge(discharge(
                MaturityFirstPartyValidator::MetadataLeafPattern,
                MaturityFirstPartyChange::OfferAMetadataLeafWhoseVerificationCanSurvive,
                canonical_branch_side,
                "CanonicalBranchSideNotSatisfied",
            )),
        );
        assert!(matches!(
            validate_maturity_first_party(&offered),
            Err(MaturityFirstPartyRefusal::RefusalNamesAnotherClass(_)),
        ));
    }

    #[test]
    fn the_branch_side_does_not_discharge_the_metadata_leaf_row() {
        let offered = case(
            MaturitySafetySection::SuccessorConstructorFault,
            "spendable-metadata-leaf",
            MaturityFirstPartyDisposition::Discharge(discharge(
                MaturityFirstPartyValidator::CanonicalBranchSide,
                MaturityFirstPartyChange::StateTheMetadataChildOnTheOtherSide,
                metadata_leaf_not_unspendable,
                "MetadataLeafNotUnspendable",
            )),
        );
        assert!(matches!(
            validate_maturity_first_party(&offered),
            Err(MaturityFirstPartyRefusal::RefusalNamesAnotherClass(_)),
        ));
    }

    #[test]
    fn the_link_does_not_discharge_a_linked_program_row() {
        // The same clause against this bite's six owners, and the two
        // linker owners are where it earns the most: both calls answer
        // about one link, so a row filed against the wrong one of them
        // would look right in every rendering. It fails here, because
        // the refusal it produced is the other call's.
        let offered = case(
            MaturitySafetySection::AbiLinkerFault,
            "relocation-omitted",
            MaturityFirstPartyDisposition::Discharge(discharge(
                MaturityFirstPartyValidator::StateCandidateLink,
                MaturityFirstPartyChange::OfferTheConstructorTheLinkItselfRetained,
                relocation_not_applied,
                "RelocationNotApplied",
            )),
        );
        assert!(matches!(
            validate_maturity_first_party(&offered),
            Err(MaturityFirstPartyRefusal::RefusalNamesAnotherClass(_)),
        ));
    }

    #[test]
    fn the_linked_program_check_does_not_discharge_a_link_row() {
        let offered = case(
            MaturitySafetySection::PredecessorConstructorFault,
            "stale-constructor-from-another-bundle",
            MaturityFirstPartyDisposition::Discharge(discharge(
                MaturityFirstPartyValidator::LinkedProgramCheck,
                MaturityFirstPartyChange::RestoreOneRelocatedSiteToItsPristinePush,
                supplied_constructor_commits_another_program,
                "SuppliedConstructorCommitsAnotherProgram",
            )),
        );
        assert!(matches!(
            validate_maturity_first_party(&offered),
            Err(MaturityFirstPartyRefusal::RefusalNamesAnotherClass(_)),
        ));
    }

    #[test]
    fn the_offered_transaction_check_does_not_discharge_a_response_row() {
        let offered = case(
            MaturitySafetySection::ProtocolReportFault,
            "infrastructure-response-carrying-target-observation",
            MaturityFirstPartyDisposition::Discharge(discharge(
                MaturityFirstPartyValidator::OfferedTransactionCheck,
                MaturityFirstPartyChange::OfferATransactionWhoseOutputTheConstructionDidNotFix,
                infrastructure_response_carries_observation,
                "InfrastructureResponseCarriesObservation",
            )),
        );
        assert!(matches!(
            validate_maturity_first_party(&offered),
            Err(MaturityFirstPartyRefusal::RefusalNamesAnotherClass(_)),
        ));
    }

    #[test]
    fn the_response_shape_check_does_not_discharge_the_offered_transaction_row() {
        let offered = case(
            MaturitySafetySection::AbiLinkerFault,
            "raw-transaction-bypassing-safe-construction",
            MaturityFirstPartyDisposition::Discharge(discharge(
                MaturityFirstPartyValidator::ResponseShapeValidation,
                MaturityFirstPartyChange::ReportAStackBesideAVerdictThatNothingRan,
                output_mutated_after_signing,
                "OutputMutatedAfterSigning",
            )),
        );
        assert!(matches!(
            validate_maturity_first_party(&offered),
            Err(MaturityFirstPartyRefusal::RefusalNamesAnotherClass(_)),
        ));
    }

    #[test]
    fn the_operation_response_check_does_not_discharge_a_metadata_row() {
        let offered = case(
            MaturitySafetySection::MetadataFault,
            "omit-one-field",
            MaturityFirstPartyDisposition::Discharge(discharge(
                MaturityFirstPartyValidator::OperationResponseShape,
                MaturityFirstPartyChange::OmitTheIdentityAnAcceptedSubmissionTook,
                metadata_encoding_refused,
                "MetadataEncodingRefused",
            )),
        );
        assert!(matches!(
            validate_maturity_first_party(&offered),
            Err(MaturityFirstPartyRefusal::RefusalNamesAnotherClass(_)),
        ));
    }

    #[test]
    fn the_conservation_response_check_does_not_discharge_a_submission_row() {
        // The pair this clause is sharpest on: two records in one
        // vocabulary, so the class alone would not say which was
        // offered, and only the validator beside the row does.
        let offered = case(
            MaturitySafetySection::ProtocolReportFault,
            "accepted-submission-without-identity",
            MaturityFirstPartyDisposition::Discharge(discharge(
                MaturityFirstPartyValidator::ConservationResponseShape,
                MaturityFirstPartyChange::CarryAnOpeningOnARefusedConservationRow,
                accepted_operation_omits_observation,
                "AcceptedOperationOmitsObservation",
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
