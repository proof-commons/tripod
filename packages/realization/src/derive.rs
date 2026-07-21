//! Deterministic derivation of one explicitly scoped realization.

use std::collections::BTreeMap;

use architecture::{Architecture, OperationId};
use petgraph::graph::{DiGraph, NodeIndex};

use crate::{
    ArchitectureBinding, ConstructibilityEdge, ConstructibilityGraphProjection,
    ConstructibilityNode, ConstructibilityNodeId, DeclassificationAnalysis, DependencyEdge,
    DisclosureEdge, DisclosureGraphProjection, DisclosureNode, ExprId, ExpressionDeclaration,
    LifecycleEdge, LifecycleGraphProjection, LifecycleNode, LifecycleNodeId, OperationObservation,
    OperationRealization, RealizationError, RealizationScope, RelationDeclaration, RelationEdge,
    RelationGraphProjection, RelationId,
    constructibility::{build_constructibility_graph, project_constructibility_graph},
    declarations,
    declassification::{analyze_disclosure, build_disclosure_graph, project_disclosure_graph},
    expression::{ExpressionGraphProjection, build_expression_graph, project_expression_graph},
    lifecycle::{build_lifecycle_graph, project_lifecycle_graph},
    relation::{build_relation_graph, project_relation_graph},
};

/// Explicitly scoped target-independent realization.
#[derive(Clone, Debug)]
pub struct ScopedRealizationSpec {
    pub architecture: ArchitectureBinding,
    pub scope: RealizationScope,
    pub operations: BTreeMap<OperationId, OperationRealization>,
    pub(crate) expression_graph: DiGraph<ExpressionDeclaration, DependencyEdge, u32>,
    pub(crate) relation_graph: DiGraph<RelationDeclaration, RelationEdge, u32>,
    pub(crate) relation_node_by_id: BTreeMap<RelationId, NodeIndex<u32>>,
    pub(crate) expression_evaluation_order: Vec<ExprId>,
    pub(crate) relation_evaluation_order: Vec<RelationId>,
    pub(crate) constructibility_graph: DiGraph<ConstructibilityNode, ConstructibilityEdge, u32>,
    pub(crate) constructibility_node_by_id: BTreeMap<ConstructibilityNodeId, NodeIndex<u32>>,
    pub(crate) lifecycle_graph: DiGraph<LifecycleNode, LifecycleEdge, u32>,
    pub(crate) lifecycle_node_by_id: BTreeMap<LifecycleNodeId, NodeIndex<u32>>,
    pub(crate) disclosure_graph: DiGraph<DisclosureNode, DisclosureEdge, u32>,
    pub declassification: DeclassificationAnalysis,
}

impl ScopedRealizationSpec {
    /// Return one declared operation by stable architecture-owned ID.
    #[must_use]
    pub fn operation(&self, id: OperationId) -> Option<&OperationRealization> {
        self.operations.get(&id)
    }

    /// Return one relation by stable realization ID.
    #[must_use]
    pub fn relation(&self, id: &RelationId) -> Option<&RelationDeclaration> {
        self.relation_node_by_id
            .get(id)
            .copied()
            .map(|node| &self.relation_graph[node])
    }

    /// Iterate relations in deterministic evaluation order.
    pub fn relations(&self) -> impl Iterator<Item = &RelationDeclaration> {
        self.relation_evaluation_order
            .iter()
            .filter_map(|id| self.relation(id))
    }

    /// Evaluate one observed operation against this realization.
    pub fn evaluate_operation(
        &self,
        observation: &OperationObservation,
    ) -> Result<crate::ConformanceReport, RealizationError> {
        crate::evaluate::evaluate_operation(
            &self.relation_graph,
            &self.relation_node_by_id,
            &self.relation_evaluation_order,
            observation,
        )
    }

    /// Project this realization into stable typed values.
    #[must_use]
    pub fn project(&self) -> ScopedRealizationProjection {
        project_scoped_realization(self)
    }
}

/// Stable typed projection of a scoped realization for deterministic comparison.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopedRealizationProjection {
    pub architecture: ArchitectureBinding,
    pub scope: RealizationScope,
    pub operations: Vec<(OperationId, OperationRealization)>,
    pub expressions: ExpressionGraphProjection,
    pub relations: RelationGraphProjection,
    pub constructibility: ConstructibilityGraphProjection,
    pub lifecycle: LifecycleGraphProjection,
    pub disclosure: DisclosureGraphProjection,
    pub declassification: DeclassificationAnalysis,
    pub expression_evaluation_order: Vec<ExprId>,
    pub relation_evaluation_order: Vec<RelationId>,
}

/// Derive one target-independent realization from architecture and scope.
pub fn derive(
    architecture: &Architecture,
    scope: RealizationScope,
) -> Result<ScopedRealizationSpec, RealizationError> {
    let binding = ArchitectureBinding::from_architecture(architecture)?;

    scope.validate_against(architecture)?;

    let mut operations = BTreeMap::new();

    for operation in scope.operations() {
        let declaration = declarations::derive_operation(architecture, *operation)?;

        if operations.insert(*operation, declaration).is_some() {
            return Err(RealizationError::DuplicateOperationDeclaration(*operation));
        }
    }

    let expression_declarations = operations
        .values()
        .flat_map(|operation| operation.expressions.iter().cloned())
        .collect::<Vec<_>>();
    let (expression_graph, _expression_node_by_id, expression_evaluation_order) =
        build_expression_graph(expression_declarations)?;

    let relation_declarations = operations
        .values()
        .flat_map(|operation| operation.relations.iter().cloned())
        .collect::<Vec<_>>();
    let relation_dependencies = operations
        .values()
        .flat_map(|operation| operation.relation_dependencies.iter().cloned())
        .collect::<Vec<_>>();
    let (relation_graph, relation_node_by_id, relation_evaluation_order) =
        build_relation_graph(relation_declarations, relation_dependencies)?;

    let constructibility_nodes = operations
        .values()
        .flat_map(|operation| operation.constructibility_nodes.iter().cloned())
        .collect::<Vec<_>>();
    let constructibility_edges = operations
        .values()
        .flat_map(|operation| operation.constructibility_edges.iter().cloned())
        .collect::<Vec<_>>();
    let (constructibility_graph, constructibility_node_by_id, _constructibility_order) =
        build_constructibility_graph(constructibility_nodes, constructibility_edges)?;

    let lifecycle_nodes = operations
        .values()
        .flat_map(|operation| operation.lifecycle_nodes.iter().cloned())
        .collect::<Vec<_>>();
    let lifecycle_edges = operations
        .values()
        .flat_map(|operation| operation.lifecycle_edges.iter().cloned())
        .collect::<Vec<_>>();
    let (lifecycle_graph, lifecycle_node_by_id, _lifecycle_order) =
        build_lifecycle_graph(lifecycle_nodes, lifecycle_edges)?;

    let disclosure_nodes = operations
        .values()
        .flat_map(|operation| operation.disclosure_nodes.iter().cloned())
        .collect::<Vec<_>>();
    let disclosure_edges = operations
        .values()
        .flat_map(|operation| operation.disclosure_edges.iter().cloned())
        .collect::<Vec<_>>();
    let disclosure_seeds = operations
        .values()
        .flat_map(|operation| operation.disclosure_seeds.iter().cloned())
        .collect::<Vec<_>>();
    let (disclosure_graph, disclosure_node_by_id) =
        build_disclosure_graph(disclosure_nodes, disclosure_edges)?;
    let declassification =
        analyze_disclosure(&disclosure_graph, &disclosure_node_by_id, &disclosure_seeds)?;

    let result = ScopedRealizationSpec {
        architecture: binding,
        scope,
        operations,
        expression_graph,
        relation_graph,
        relation_node_by_id,
        expression_evaluation_order,
        relation_evaluation_order,
        constructibility_graph,
        constructibility_node_by_id,
        lifecycle_graph,
        lifecycle_node_by_id,
        disclosure_graph,
        declassification,
    };

    crate::validate::validate_scoped_realization(architecture, &result)?;

    Ok(result)
}

/// Project a scoped realization into stable typed values.
#[must_use]
pub fn project_scoped_realization(
    realization: &ScopedRealizationSpec,
) -> ScopedRealizationProjection {
    ScopedRealizationProjection {
        architecture: realization.architecture.clone(),
        scope: realization.scope.clone(),
        operations: realization
            .operations
            .iter()
            .map(|(operation, declaration)| (*operation, declaration.clone()))
            .collect(),
        expressions: project_expression_graph(&realization.expression_graph),
        relations: project_relation_graph(&realization.relation_graph),
        constructibility: project_constructibility_graph(&realization.constructibility_graph),
        lifecycle: project_lifecycle_graph(&realization.lifecycle_graph),
        disclosure: project_disclosure_graph(&realization.disclosure_graph),
        declassification: realization.declassification.clone(),
        expression_evaluation_order: realization.expression_evaluation_order.clone(),
        relation_evaluation_order: realization.relation_evaluation_order.clone(),
    }
}
