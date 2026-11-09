//! Authorization-case constructibility analysis (Guide-3 Tranche D).
//!
//! The compiler rebuilds the realization's stable constructibility
//! projection into its own direct Petgraph graph and revalidates,
//! per operation and per owner-derived authorization case, that every
//! required ancestor's availability is dischargeable, that sponsor-local
//! dependencies stay confined to optional sponsor-only subtrees, and
//! that no dependency crosses operations. Witness-bearing source
//! requirements are then cross-checked against the graph so a proof
//! can never assume a witness constructibility does not provide.

// One item-level allowance remains: the Petgraph storage is retained
// beside the derived analysis rather than read by it. The analysis
// answers its questions from the resolved maps.

use std::collections::{BTreeMap, BTreeSet};

use petgraph::graph::{DiGraph, NodeIndex};
use realization::{
    AvailabilityClass, ConstructibilityAuthorization, ConstructibilityEdge, ConstructibilityNode,
    ConstructibilityNodeId, WitnessRole,
};

use crate::{
    CompileError,
    input::BoundCompilerInput,
    source::{RequiredSourceKind, SourceRequirement},
};

/// Constructibility analysis of one authorization case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstructibilityCaseAnalysis {
    pub authorization: ConstructibilityAuthorization,
    pub required_nodes: Vec<ConstructibilityNodeId>,
    pub optional_nodes: Vec<ConstructibilityNodeId>,
    pub required_availability: BTreeSet<AvailabilityClass>,
}

/// Constructibility analysis of one operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationConstructibilityAnalysis {
    pub operation: architecture::OperationId,
    pub cases: Vec<ConstructibilityCaseAnalysis>,
}

/// Internal compiler constructibility analysis.
#[derive(Debug)]
pub struct CompilerConstructibilityAnalysis {
    #[allow(dead_code)]
    pub graph: DiGraph<ConstructibilityNode, ConstructibilityEdge, u32>,
    pub node_by_id: BTreeMap<ConstructibilityNodeId, NodeIndex<u32>>,
    pub operations: Vec<OperationConstructibilityAnalysis>,
}

/// Build and validate the scoped constructibility analysis.
pub fn build_constructibility_analysis(
    input: &BoundCompilerInput,
) -> Result<CompilerConstructibilityAnalysis, CompileError> {
    crate::engine_attestation::constructibility::lower(input.owner())?.analyze()
}

/// Cross-check witness-bearing source requirements against the graph:
/// a proof may not assume a witness constructibility does not provide.
pub fn validate_source_constructibility(
    analysis: &CompilerConstructibilityAnalysis,
    relation: &realization::RelationId,
    rows: &[SourceRequirement],
) -> Result<(), CompileError> {
    let operation = relation.operation();
    let optional = optional_witnesses(analysis, operation);

    for row in rows {
        let expected = match row.source {
            RequiredSourceKind::InputOwnerWitness => Some(WitnessRole::ProtocolOwnerAuthorization),
            RequiredSourceKind::OperatorWitness => Some(WitnessRole::OperatorAuthorization),
            RequiredSourceKind::SponsorLocalWitness => Some(WitnessRole::SponsorAuthorization),
            _ => None,
        };
        let Some(role) = expected else {
            continue;
        };

        let witness = ConstructibilityNodeId::Witness {
            operation,
            role,
            availability: row.availability,
        };

        if !analysis.node_by_id.contains_key(&witness) {
            return Err(CompileError::SourceConstructibilityMismatch {
                relation: relation.clone(),
                operand: row.operand.clone(),
            });
        }

        // A sponsor-local witness must belong to an optional subtree.
        if row.availability == AvailabilityClass::SponsorLocal && !optional.contains(&witness) {
            return Err(CompileError::SourceConstructibilityMismatch {
                relation: relation.clone(),
                operand: row.operand.clone(),
            });
        }
    }

    Ok(())
}

fn optional_witnesses(
    analysis: &CompilerConstructibilityAnalysis,
    operation: architecture::OperationId,
) -> BTreeSet<ConstructibilityNodeId> {
    analysis
        .operations
        .iter()
        .filter(|row| row.operation == operation)
        .flat_map(|row| row.cases.iter())
        .flat_map(|case| case.optional_nodes.iter().cloned())
        .collect()
}

/// Whether one authorization case can discharge one required dependency.
///
/// The check `analyze_operation` makes for every required ancestor, and
/// the only place the two constructibility refusals are constructed.
/// Published so that a first-party negative case can drive the exact
/// owning validator rather than a second spelling of it: an evidence
/// package holding a refusal from a mirror of this check would establish
/// that the mirror refuses.
///
/// The path is a closure because it is only ever walked on the refusing
/// branch, and a caller stating a dependency out of a graph it has not
/// built has no walk to offer.
///
/// # Errors
///
/// [`CompileError::PermissionlessPrivateDependency`] where the case is a
/// permissionless one, and
/// [`CompileError::ConstructibilityWitnessUnavailable`] otherwise. A
/// node carrying no availability class is discharged by every case and
/// answers `Ok`.
pub fn validate_required_dependency(
    operation: architecture::OperationId,
    authorization: &ConstructibilityAuthorization,
    node: &ConstructibilityNodeId,
    path: impl FnOnce() -> Vec<ConstructibilityNodeId>,
) -> Result<(), CompileError> {
    let lowered =
        crate::engine_attestation::constructibility::dependency_allowed(authorization, node);
    if lowered {
        return Ok(());
    }
    Err(unavailable_error(
        operation,
        authorization,
        node.clone(),
        path(),
    ))
}

fn unavailable_error(
    operation: architecture::OperationId,
    authorization: &ConstructibilityAuthorization,
    node: ConstructibilityNodeId,
    path: Vec<ConstructibilityNodeId>,
) -> CompileError {
    match authorization {
        ConstructibilityAuthorization::Permissionless
        | ConstructibilityAuthorization::CadencePermissionless => {
            CompileError::PermissionlessPrivateDependency {
                operation,
                node,
                path,
            }
        }
        _ => CompileError::ConstructibilityWitnessUnavailable {
            operation,
            authorization: authorization.clone(),
            node,
            path,
        },
    }
}
