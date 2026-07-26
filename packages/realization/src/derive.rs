//! Deterministic derivation of one explicitly scoped realization.

use std::collections::BTreeMap;

use architecture::{Architecture, OperationId};
use petgraph::graph::{DiGraph, NodeIndex};

use crate::{
    ArchitectureBinding, ConstructibilityEdge, ConstructibilityGraphProjection,
    ConstructibilityNode, ConstructibilityNodeId, DeclassificationAnalysis, DependencyEdge,
    DisclosureEdge, DisclosureGraphProjection, DisclosureNode, ExprId, ExpressionDeclaration,
    LifecycleEdge, LifecycleGraphProjection, LifecycleNode, LifecycleNodeId, OperationObservation,
    OperationRealization, OperationRealizationProjection, RealizationError, RealizationScope,
    RelationDeclaration, RelationEdge, RelationGraphProjection, RelationId,
    constructibility::{build_constructibility_graph, project_constructibility_graph},
    declarations,
    declassification::{analyze_disclosure, build_disclosure_graph, project_disclosure_graph},
    expression::{ExpressionGraphProjection, build_expression_graph, project_expression_graph},
    lifecycle::{build_lifecycle_graph, project_lifecycle_graph},
    relation::{build_relation_graph, project_relation_graph},
};

/// Explicitly scoped target-independent realization.
///
/// A value of this type is validated on construction and stays
/// internally consistent afterwards: every field that the derived
/// graphs summarize is private, so an external consumer can read the
/// source declarations and projections but never desynchronize them
/// from the graphs.
///
/// ```compile_fail
/// let mut spec = realization::derive(
///     &architecture::ARCHITECTURE,
///     realization::RealizationScope::phase1_pilots(),
/// )
/// .unwrap();
/// spec.operations.clear();
/// ```
///
/// ```compile_fail
/// let mut spec = realization::derive(
///     &architecture::ARCHITECTURE,
///     realization::RealizationScope::phase1_pilots(),
/// )
/// .unwrap();
/// spec.scope = realization::RealizationScope::phase1_pilots();
/// ```
///
/// ```compile_fail
/// let mut spec = realization::derive(
///     &architecture::ARCHITECTURE,
///     realization::RealizationScope::phase1_pilots(),
/// )
/// .unwrap();
/// let replacement = spec.architecture().clone();
/// spec.architecture = replacement;
/// ```
///
/// ```compile_fail
/// let mut spec = realization::derive(
///     &architecture::ARCHITECTURE,
///     realization::RealizationScope::phase1_pilots(),
/// )
/// .unwrap();
/// let replacement = spec.declassification().clone();
/// spec.declassification = replacement;
/// ```
#[derive(Clone, Debug)]
pub struct ScopedRealizationSpec {
    pub(crate) architecture: ArchitectureBinding,
    pub(crate) scope: RealizationScope,
    pub(crate) operations: BTreeMap<OperationId, OperationRealization>,
    pub(crate) expression_graph: DiGraph<ExpressionDeclaration, DependencyEdge, u32>,
    pub(crate) expression_node_by_id: BTreeMap<ExprId, NodeIndex<u32>>,
    pub(crate) relation_graph: DiGraph<RelationDeclaration, RelationEdge, u32>,
    pub(crate) relation_node_by_id: BTreeMap<RelationId, NodeIndex<u32>>,
    pub(crate) expression_evaluation_order: Vec<ExprId>,
    pub(crate) relation_evaluation_order: Vec<RelationId>,
    pub(crate) constructibility_graph: DiGraph<ConstructibilityNode, ConstructibilityEdge, u32>,
    pub(crate) constructibility_node_by_id: BTreeMap<ConstructibilityNodeId, NodeIndex<u32>>,
    pub(crate) lifecycle_graph: DiGraph<LifecycleNode, LifecycleEdge, u32>,
    pub(crate) lifecycle_node_by_id: BTreeMap<LifecycleNodeId, NodeIndex<u32>>,
    pub(crate) disclosure_graph: DiGraph<DisclosureNode, DisclosureEdge, u32>,
    pub(crate) declassification: DeclassificationAnalysis,
}

impl ScopedRealizationSpec {
    /// Return the pinned architecture identity this realization binds.
    #[must_use]
    pub fn architecture(&self) -> &ArchitectureBinding {
        &self.architecture
    }

    /// Return the explicit operation scope this realization covers.
    #[must_use]
    pub fn scope(&self) -> &RealizationScope {
        &self.scope
    }

    /// Iterate declared operations in stable operation-ID order.
    pub fn operations(&self) -> impl Iterator<Item = (OperationId, &OperationRealization)> {
        self.operations
            .iter()
            .map(|(operation, declaration)| (*operation, declaration))
    }

    /// Return the declassification analysis of the disclosure graph.
    #[must_use]
    pub fn declassification(&self) -> &DeclassificationAnalysis {
        &self.declassification
    }

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
            &self.expression_graph,
            &self.expression_node_by_id,
            &self.expression_evaluation_order,
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
///
/// Every graph-shaped declaration appears exactly once, in its
/// canonical graph projection; the per-operation rows carry only
/// canonical operation-owned facts. Source declaration order never
/// reaches this value, so two realizations with permuted set-like
/// declaration collections project equal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopedRealizationProjection {
    pub architecture: ArchitectureBinding,
    pub scope: RealizationScope,
    pub operations: Vec<OperationRealizationProjection>,
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

    assemble_scoped_realization(architecture, binding, scope, operations)
}

/// Build and validate one realization from explicit operation
/// declarations.
///
/// Crate-private on purpose: this is the single graph-assembly and
/// validation path shared by [`derive`] and by test fixtures that
/// permute or mutate declarations — every constructed value passes
/// through the same builders and `validate_scoped_realization`.
pub(crate) fn assemble_scoped_realization(
    architecture: &Architecture,
    binding: ArchitectureBinding,
    scope: RealizationScope,
    operations: BTreeMap<OperationId, OperationRealization>,
) -> Result<ScopedRealizationSpec, RealizationError> {
    for (operation, declaration) in &operations {
        crate::validate::validate_operation_ownership(*operation, declaration)?;
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
    // Weld the disclosure graph to the relation census before any
    // disclosure conclusion is drawn. Ownership validation runs per
    // operation and cannot see this: a phantom relation carries the
    // right operation, so it passes ownership while naming a relation
    // the semantic graph never declared. Without this check a
    // declassification requirement could be derived for a relation
    // that does not exist, and the compiler-input boundary would carry
    // a requirement its own relation census cannot support.
    crate::validate::validate_disclosure_relation_census(
        &disclosure_nodes,
        &disclosure_edges,
        &disclosure_seeds,
        &relation_node_by_id,
    )?;

    let (disclosure_graph, disclosure_node_by_id) =
        build_disclosure_graph(disclosure_nodes, disclosure_edges)?;
    let declassification =
        analyze_disclosure(&disclosure_graph, &disclosure_node_by_id, &disclosure_seeds)?;

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
            .values()
            .map(OperationRealization::project)
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
