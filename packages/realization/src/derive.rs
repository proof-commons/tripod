//! Deterministic derivation of one explicitly scoped realization.

use std::collections::BTreeMap;

use architecture::{Architecture, OperationId};
use petgraph::graph::{DiGraph, NodeIndex};

use crate::{
    ArchitectureBinding, DependencyEdge, ExprId, ExpressionDeclaration, OperationRealization,
    RealizationError, RealizationScope, RelationDeclaration, RelationEdge, RelationId,
    declarations, expression::build_expression_graph, relation::build_relation_graph,
};

/// Explicitly scoped target-independent realization.
#[derive(Clone, Debug)]
pub struct ScopedRealizationSpec {
    pub architecture: ArchitectureBinding,
    pub scope: RealizationScope,
    pub operations: BTreeMap<OperationId, OperationRealization>,
    pub expression_graph: DiGraph<ExpressionDeclaration, DependencyEdge, u32>,
    pub expression_node_by_id: BTreeMap<ExprId, NodeIndex<u32>>,
    pub relation_graph: DiGraph<RelationDeclaration, RelationEdge, u32>,
    pub relation_node_by_id: BTreeMap<RelationId, NodeIndex<u32>>,
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
    let (expression_graph, expression_node_by_id, expression_evaluation_order) =
        build_expression_graph(expression_declarations)?;

    let relation_declarations = operations
        .values()
        .flat_map(|operation| operation.relations.iter().cloned())
        .collect::<Vec<_>>();
    let (relation_graph, relation_node_by_id, relation_evaluation_order) =
        build_relation_graph(relation_declarations)?;

    let result = ScopedRealizationSpec {
        architecture: binding,
        scope,
        operations,
        expression_graph,
        expression_node_by_id,
        relation_graph,
        relation_node_by_id,
        expression_evaluation_order,
        relation_evaluation_order,
    };

    crate::validate::validate_scoped_realization(architecture, &result)?;

    Ok(result)
}
