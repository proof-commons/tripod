#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod abi;
pub mod bytes;
pub mod construct;
pub mod error;
pub mod live_abi;
pub mod live_construct;
pub mod live_finalize;
pub mod live_private;
pub mod live_request;
pub mod live_signing;
pub mod live_taproot;
pub mod request;
pub mod sponsor;
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
    TargetTransaction, Txid, ValueField, compact_size,
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
pub use live_construct::{
    CandidateLiveTransferTransaction, LIVE_TRANSFER_SEQUENCE, LiveConstructionReport,
    LiveFinalization, LiveOwnerCensus, complete_live_transfer, finalize_live_transfer,
};
pub use live_finalize::{
    FinalizedFact, FinalizedLiveTransfer, FinalizedOutputCensus, LiveSigningRequest,
    ReceiptInputRecord,
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
pub use request::CompactAshRequest;
pub use sponsor::{
    SighashProfile, SignerRole, SponsorCapability, SponsorOffer, SponsorSignature,
    SponsorSigningRequest,
};
pub use synthetic::{FundingCeremonyStep, SyntheticDisclaimer};
pub use taproot::{
    AshInstanceOrigin, CommittedTree, Digest32, OutputKeyParity, PinnedAshInstance, branch_hash,
    commit_tree, leaf_hash, tagged_hash, witness_program_script,
};
pub use view::{PublicConstructionView, PublicOutputView};

#[cfg(test)]
mod tests;
