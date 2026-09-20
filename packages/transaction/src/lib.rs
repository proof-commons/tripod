#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod abi;
pub mod bytes;
pub mod construct;
pub mod error;
pub mod live_abi;
pub mod live_accepted;
pub mod live_census;
pub mod live_construct;
pub mod live_finalize;
pub mod live_handoff;
pub mod live_materialize;
pub mod live_message;
pub mod live_private;
pub mod live_request;
pub mod live_signing;
pub mod live_taproot;
pub mod operator_membership;
pub mod operator_right;
pub mod operator_signing;
pub mod request;
pub mod script_path_signing;
pub mod sponsor;
pub mod state_abi;
pub mod state_construct;
pub mod state_finalize;
pub mod state_request;
pub mod state_view;
pub mod synthetic;
pub mod taproot;
pub mod view;

pub use abi::{
    AbiObligation, AbiStatus, CandidateTransactionAbi, CanonicalOrdering, CoordinatorRule,
    DeploymentSymbols, OutstandingAbiObligations, PackageLimits, SequenceConstraint, ShapeAbi,
    TargetTransactionVersion, WitnessItem, derive_candidate_abi,
};
pub use bytes::{
    AssetField, AssetId, InputWitness, NonceField, Outpoint, TargetInput, TargetOutput,
    TargetTransaction, TransactionIdentityParseError, Txid, ValueField, compact_size,
};
pub use construct::{
    CandidateCompleteTransaction, ConstructionReport, RoleCensus, SettledResources, check_weight,
    construct,
};
pub use error::TransactionRefusal;
pub use live_abi::{
    CandidateLiveTransferAbi, DestinationConstructorTable, InheritedLinkObligations,
    LIVE_TRANSFER_LOCK_TIME, LiveAbiObligation, LiveAbiStatus, LiveCanonicalOrdering,
    LiveCoordinatorRule, LiveDeploymentSymbols, LiveDestinationConstructor, LiveShapeAbi,
    LiveTransactionForm, LiveWitnessItem, OutstandingLiveAbiObligations, derive_live_transfer_abi,
};
pub use live_accepted::{
    AcceptedOwnerAuthorizations, AcceptedResultRefusal, OfferedOwnerAuthorization,
};
pub use live_census::{
    AnnexDisposition, FinalizedSigningField, IssuanceDisposition, LiveDeployment,
    OWNER_CODESEPARATOR_POSITION, OWNER_KEY_VERSION_BYTE, OWNER_SIGHASH_TYPE_BYTE,
    OWNER_SIGNATURE_BYTES, OWNER_SPEND_TYPE_BYTE, OwnerCensusRefusal, OwnerSigningCensus,
    OwnerSigningInputCensus, OwnerSigningInputRequest, ProofFinalizedReceiptInput,
    ProofFinalizedSigningCandidate, SpentOutputCensusEntry, check_signature_width, check_type_byte,
    spend_type_byte,
};
pub use live_construct::{
    CandidateLiveTransferTransaction, ExplicitDestinationRole, LIVE_TRANSFER_SEQUENCE,
    LiveConstructionReport, LiveFinalization, LiveOwnerCensus, complete_live_transfer,
    finalize_live_transfer, finalize_live_transfer_declaring,
};
pub use live_finalize::{
    FinalizedFact, FinalizedLiveTransfer, FinalizedOutputCensus, LiveSigningRequest,
    ReceiptInputRecord,
};
pub use live_message::{
    CandidateMessagePair, KEY_PATH_SPEND_TYPE_BYTE, TAP_SIGHASH_TAG, WitnessVectorTreatment,
    candidate_key_path_message, candidate_message_pair, candidate_owner_message,
};
pub use live_private::{
    ConfidentialConstructionModel, PrivateConstructionDemonstration, PrivateConstructionNonClaim,
    PrivateValueCapability, SelectedConstructionModel,
};
pub use live_request::{
    LiveReceiptDestination, LiveTransferRequest, ProtocolValue, PublicTestRandomness,
    RequestedForm, SelectableRequestFacet, SponsorChangeRequest, UnselectableRequestFacet,
};
pub use live_signing::{
    AuthorizedLiveTransfer, LiveOwnerResponse, OwnerAuthorizationLevels, authorize_live_transfer,
};
pub use live_taproot::{
    CommittedLiveTree, LiveCurveCapability, LiveReceiptInstance, TweakedOutputKey,
    commit_live_tree, derive_live_receipt_instance,
};
pub use operator_membership::{
    OperatorMappingRefusal, OperatorMembershipMapping, OperatorMembershipRefusal,
    OperatorMembershipRequest, produce_operator_membership,
};

pub use operator_right::{
    BranchContext, CachedOperatorAuthorization, ConstructionRight, NonEquivocationEntry,
    NonEquivocationEvent, NonEquivocationRecord, OperatorRightOutcome, OperatorRightRegistry,
    RightFailure, RightIdentity, RightRefusal, RightScope, StateCheckpointPolicy,
    StateContinuation, StateContinuationStanding, StateContinuityEvidence, StateThreadAnchor,
    StateThreadContinuations, StateThreadOrigin, StateThreadProvenance, StateThreadRefusal,
};
pub use operator_signing::{
    OPERATOR_CODESEPARATOR_POSITION, OPERATOR_KEY_VERSION_BYTE, OPERATOR_SIGHASH_TYPE_BYTE,
    OPERATOR_SIGNATURE_BYTES, OPERATOR_SPEND_TYPE_BYTE, OperatorAuthorizedCandidate,
    OperatorEvidenceStanding, OperatorSigningInput, OperatorSigningRefusal, OperatorSigningRequest,
    OperatorSigningResponse, ScriptPathSignatureVerifier, ScriptPathVerifierRejection,
    authorize_operator, authorize_operator_under_right,
};
pub use request::CompactAshRequest;
pub use script_path_signing::{
    ScriptPathCensusRefusal, ScriptPathSigningCensus, ScriptPathSigningInputCensus,
};
pub use sponsor::{
    SighashProfile, SignerRole, SponsorCapability, SponsorOffer, SponsorSignature,
    SponsorSigningRequest,
};
pub use state_abi::{
    CandidateMaturityAnnouncementAbi, InheritedStateLinkObligations, MaturityAbiObligation,
    MaturityAbiStatus, MaturityCoordinatorRule, MaturityInputLayout, MaturityOutputLayout,
    MaturityOutputPlacementRule, MaturityOutputRole, MaturityRelayVerdict,
    MaturitySponsorSuffixRule, MaturityWitnessRole, MaturityWitnessSource,
    OutstandingMaturityAbiObligations, WitnessClassification, derive_maturity_announcement_abi,
};
pub use state_construct::{
    MATURITY_ANNOUNCEMENT_LOCK_TIME, MaturityConstruction, construct_maturity_announcement,
};
pub use state_finalize::{
    FinalizedMaturityAnnouncement, MaturityExecutingLeaf, MaturityFinalizedFact,
    MaturityFinalizedOutputCensus, MaturitySpentOutputRecord, finalize_maturity_announcement,
};
pub use state_request::{
    MaturityAnnouncementRequest, SelectableMaturityRequestFacet, UnselectableMaturityRequestFacet,
};
pub use state_view::{
    MaturityViewEntry, MaturityViewResidual, MaturityViewStatement, PublicMaturityStateView,
    ValidatedMaturityStateView,
};
pub use synthetic::{FundingCeremonyStep, SyntheticDisclaimer};
pub use taproot::{
    AshInstanceOrigin, CommittedTree, Digest32, OutputKeyParity, PinnedAshInstance, branch_hash,
    commit_tree, leaf_hash, tagged_hash, witness_program_script,
};
pub use view::{PublicConstructionView, PublicOutputView};

#[cfg(test)]
mod tests;
