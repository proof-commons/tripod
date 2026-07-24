//! Typed operation declarations.

use architecture::OperationId;

use crate::{
    ConstructibilityDependencyDeclaration, ConstructibilityNode, DisclosureDependencyDeclaration,
    DisclosureNode, DisclosureSeed, ExpressionDeclaration, LifecycleDependencyDeclaration,
    LifecycleNode, RelationDeclaration, RelationDependencyDeclaration,
};

/// One target-independent operation realization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationRealization {
    pub operation: OperationId,
    pub expressions: Vec<ExpressionDeclaration>,
    pub relations: Vec<RelationDeclaration>,
    pub relation_dependencies: Vec<RelationDependencyDeclaration>,
    pub constructibility_nodes: Vec<ConstructibilityNode>,
    pub constructibility_edges: Vec<ConstructibilityDependencyDeclaration>,
    pub lifecycle_nodes: Vec<LifecycleNode>,
    pub lifecycle_edges: Vec<LifecycleDependencyDeclaration>,
    pub disclosure_nodes: Vec<DisclosureNode>,
    pub disclosure_edges: Vec<DisclosureDependencyDeclaration>,
    pub disclosure_seeds: Vec<DisclosureSeed>,
}

/// Canonical projection of one declared operation.
///
/// The stable scoped projection represents every graph-shaped
/// declaration exactly once, in the canonical graph projections.
/// The per-operation row therefore carries only the operation
/// identity and the canonically sorted disclosure seeds — the one
/// operation-owned collection no graph projection represents. Raw
/// source-order declaration vectors never enter the stable
/// projection, so permuting a set-like declaration collection cannot
/// move it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationRealizationProjection {
    pub operation: OperationId,
    pub disclosure_seeds: Vec<DisclosureSeed>,
}

impl OperationRealization {
    /// Project this declaration into its canonical stable row.
    #[must_use]
    pub fn project(&self) -> OperationRealizationProjection {
        let mut disclosure_seeds = self.disclosure_seeds.clone();
        disclosure_seeds.sort();
        OperationRealizationProjection {
            operation: self.operation,
            disclosure_seeds,
        }
    }
}
