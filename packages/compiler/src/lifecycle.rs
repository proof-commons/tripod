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

// The analysis stages have no non-test consumer until the P2-012
// analyzed program; unit tests exercise them until then. Remove with
// the first real consumer.
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use petgraph::{
    algo::has_path_connecting,
    graph::{DiGraph, NodeIndex},
};
use realization::{
    LifecycleEdge, LifecycleNode, LifecycleNodeId, ProofKind, Relation, RepresentationMode,
};

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
    pub graph: DiGraph<LifecycleNode, LifecycleEdge, u32>,
    pub node_by_id: BTreeMap<LifecycleNodeId, NodeIndex<u32>>,
    pub choices: Vec<RepresentationChoice>,
    pub requirements: Vec<LifecycleRequirement>,
}

impl CompilerLifecycleAnalysis {
    /// True when every requirement's exit is available in scope: the
    /// deployment-lifecycle-complete claim, expected false for pilots.
    #[must_use]
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

    // Representation variables from the exact representation relations.
    let mut choices = Vec::new();
    let mut choice_objects = BTreeSet::new();

    for node in relations.graph.node_weights() {
        if let Relation::Representation { object, allowed } = &node.source.relation {
            let mut candidates = allowed.iter().copied().collect::<Vec<_>>();
            candidates.sort();

            choice_objects.insert(*object);
            choices.push(RepresentationChoice {
                id: RepresentationChoiceId {
                    operation: node.source.id.operation(),
                    object: *object,
                },
                candidates,
            });
        }
    }

    choices.sort_by(|left, right| left.id.cmp(&right.id));

    // Every in-scope lifecycle-exit relation needs a representation
    // variable for its object.
    for node in relations.graph.node_weights() {
        if let Relation::LifecycleExit { object, .. } = &node.source.relation
            && !choice_objects.contains(object)
        {
            return Err(CompileError::MissingRepresentationChoice {
                operation: node.source.id.operation(),
                object: *object,
            });
        }
    }

    // Rebuild the lifecycle graph, restricted to the chosen objects.
    let nodes = projection
        .nodes
        .iter()
        .filter(|node| choice_objects.contains(&lifecycle_object(&node.id)))
        .cloned()
        .collect::<Vec<_>>();
    let mut graph = DiGraph::<LifecycleNode, LifecycleEdge, u32>::with_capacity(
        nodes.len(),
        projection.edges.len(),
    );
    let mut node_by_id = BTreeMap::new();

    for node_weight in nodes {
        let id = node_weight.id.clone();
        let node = graph.add_node(node_weight);
        node_by_id.insert(id, node);
    }

    for edge in &projection.edges {
        let (Some(source), Some(target)) = (
            node_by_id.get(&edge.source).copied(),
            node_by_id.get(&edge.target).copied(),
        ) else {
            continue;
        };

        graph.add_edge(source, target, edge.edge);
    }

    // Every supported representation must reach every required exit of
    // its object; the exit's scope membership decides the status.
    let mut requirements = Vec::new();

    for (id, node) in &node_by_id {
        let LifecycleNodeId::Representation { object, mode } = id else {
            continue;
        };

        for (exit_id, exit_node) in &node_by_id {
            let LifecycleNodeId::RequiredExit {
                object: exit_object,
                operation,
            } = exit_id
            else {
                continue;
            };

            if exit_object != object {
                continue;
            }

            if !has_path_connecting(&graph, *node, *exit_node, None) {
                return Err(CompileError::MissingLifecyclePath {
                    object: *object,
                    representation: *mode,
                    exit: *operation,
                });
            }

            let status = if input.scope().operations().contains(operation) {
                LifecycleExitStatus::AvailableInCompilerScope
            } else {
                LifecycleExitStatus::DeclaredOutsideCompilerScope
            };

            requirements.push(LifecycleRequirement {
                object: *object,
                representation: *mode,
                exit: *operation,
                status,
            });
        }
    }

    requirements.sort();

    Ok(CompilerLifecycleAnalysis {
        graph,
        node_by_id,
        choices,
        requirements,
    })
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

const fn lifecycle_object(node: &LifecycleNodeId) -> architecture::ObjectId {
    match node {
        LifecycleNodeId::Representation { object, .. }
        | LifecycleNodeId::RequiredExit { object, .. } => *object,
    }
}
