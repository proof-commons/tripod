#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod backend;
pub mod bundle;
pub mod carrier;
pub mod deployment;
pub mod error;
pub mod graph;
pub mod relocate;
pub mod symbol;
pub mod taptree;

pub use bundle::{
    CandidateLinkedBundle, LinkObligation, LinkedArtifactStatus, LinkedConstructor,
    LinkedResourceFormula, OutstandingLinkObligations, link_candidate,
};
pub use carrier::CarrierClosure;
pub use deployment::{LinkDeploymentParameters, SelfCommitmentStrategy};
pub use error::LinkRefusal;
pub use graph::{
    FrozenReferenceGraph, ReferenceClass, ReferenceEdge, ReferenceEdgeId, ReferenceNode, SccId,
    StronglyConnectedComponent, leaf_symbol,
};
pub use relocate::LinkedLeafProgram;
pub use symbol::{
    DefinitionCensus, DefinitionOrigin, SymbolDefinition, SymbolType, SymbolValue, declared_type,
};
pub use taptree::{
    ControlPathRecipe, DeterministicTaptree, ORACLE_LEAF_BUDGET, TapLeafInput, TaptreeInput,
    TreeObjective, enumerated_minimum_cost, exact_minimum_cost,
};

#[cfg(test)]
mod tests;
