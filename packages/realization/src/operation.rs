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
