//! Representation and lifecycle analysis (Guide-3 Tranche E).
//!
//! Representation choices come from the realization's representation
//! relations — the candidate set is the relation's exact allowed set,
//! never extended. The lifecycle graph is rebuilt from the stable
//! realization projection, and every supported representation must
//! reach every required exit of its object. An exit outside compiler
//! scope is not an error in a partial pilot: it is an explicit future
//! obligation, so the analysis distinguishes semantically-valid-in-
//! scope from deployment-lifecycle-complete — the pilots satisfy the
//! first and deliberately fail the second.

// The item-level allowances below cover the Petgraph storage, which is
// retained beside the derived requirements rather than read by them,
// and a deployment-completeness query the analysis does not ask: the
// analyzed program states lifecycle completeness per plan (§5.4).

use std::collections::BTreeMap;

use petgraph::graph::{DiGraph, NodeIndex};
use realization::{LifecycleEdge, LifecycleNode, LifecycleNodeId, ProofKind, RepresentationMode};

use crate::{CompileError, input::BoundCompilerInput, relation::CompilerRelationAnalysis};

/// One representation decision variable.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RepresentationChoiceId {
    pub operation: architecture::OperationId,
    pub object: architecture::ObjectId,
}

/// One representation variable with its realization-approved candidates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepresentationChoice {
    pub id: RepresentationChoiceId,
    pub candidates: Vec<RepresentationMode>,
}

/// Whether one required exit is reachable within compiler scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LifecycleExitStatus {
    AvailableInCompilerScope,
    DeclaredOutsideCompilerScope,
}

/// One representation-specific lifecycle obligation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LifecycleRequirement {
    pub object: architecture::ObjectId,
    pub representation: RepresentationMode,
    pub exit: architecture::OperationId,
    pub status: LifecycleExitStatus,
}

/// Internal lifecycle analysis.
#[derive(Debug)]
pub struct CompilerLifecycleAnalysis {
    #[allow(dead_code)]
    pub graph: DiGraph<LifecycleNode, LifecycleEdge, u32>,
    #[allow(dead_code)]
    pub node_by_id: BTreeMap<LifecycleNodeId, NodeIndex<u32>>,
    pub choices: Vec<RepresentationChoice>,
    pub requirements: Vec<LifecycleRequirement>,
}

impl CompilerLifecycleAnalysis {
    /// True when every requirement's exit is available in scope: the
    /// deployment-lifecycle-complete claim, expected false for pilots.
    #[must_use]
    #[allow(dead_code)]
    pub fn is_deployment_complete(&self) -> bool {
        self.requirements
            .iter()
            .all(|requirement| requirement.status == LifecycleExitStatus::AvailableInCompilerScope)
    }
}

/// Build the scoped representation and lifecycle analysis.
pub fn build_lifecycle_analysis(
    input: &BoundCompilerInput,
    relations: &CompilerRelationAnalysis,
) -> Result<CompilerLifecycleAnalysis, CompileError> {
    let projection = input.realization().project().lifecycle;
    build_lifecycle_analysis_from_projection(input, relations, &projection)
}

/// Compare reachability in an offered compact lifecycle projection.
///
/// This runs the existing traversal with the bound input's relations and scope.
/// It checks paths between retained nodes, not completeness of the offered node
/// census. Exact path-removal comparisons must retain every node and every
/// unrelated edge. Success makes no deployment-completeness claim.
///
/// # Errors
///
/// Propagates relation-analysis errors and the existing traversal's
/// [`CompileError::MissingRepresentationChoice`] or
/// [`CompileError::MissingLifecyclePath`] unchanged.
pub fn validate_compact_lifecycle_projection(
    input: &BoundCompilerInput,
    offered: &realization::LifecycleGraphProjection,
) -> Result<(), CompileError> {
    let relations = crate::relation::build_relation_analysis(input)?;
    build_lifecycle_analysis_from_projection(input, &relations, offered).map(|_| ())
}

/// Build the analysis over a supplied lifecycle projection; the shared traversal behind both routes.
pub fn build_lifecycle_analysis_from_projection(
    input: &BoundCompilerInput,
    relations: &CompilerRelationAnalysis,
    projection: &realization::LifecycleGraphProjection,
) -> Result<CompilerLifecycleAnalysis, CompileError> {
    crate::engine_attestation::lifecycle::lower(
        input.scope().operations(),
        relations.graph.node_weights().map(|node| &node.source),
        projection,
    )
    .analyze()
}

/// Proof/representation compatibility (§9.6).
///
/// Public arithmetic needs exact public values; confidential
/// conservation needs a commitment. Shape, signer, constructibility,
/// and external-evidence proofs are representation-agnostic within the
/// relation's own allowed set (which is enforced separately by the
/// representation relation itself).
#[must_use]
pub const fn proof_supports_representation(proof: ProofKind, mode: RepresentationMode) -> bool {
    match proof {
        ProofKind::PublicArithmetic => matches!(
            mode,
            RepresentationMode::Explicit | RepresentationMode::PublicCommitted
        ),
        ProofKind::ConfidentialConservation => matches!(
            mode,
            RepresentationMode::PrivateCommitted | RepresentationMode::PublicCommitted
        ),
        ProofKind::ManifestShape
        | ProofKind::SignerMembership
        | ProofKind::PublicConstructibility
        | ProofKind::SubstrateConservation => true,
    }
}
