#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod abi;
pub mod bytes;
pub mod construct;
pub mod error;
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
    CandidateCompleteTransaction, ConstructionReport, RoleCensus, SettledResources, construct,
};
pub use error::TransactionRefusal;
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
