//! Typed expression declarations and direct Petgraph construction.

use std::collections::BTreeMap;

use petgraph::{
    algo::{kosaraju_scc, toposort},
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};

use crate::{Count, ExprId, FactId, ProtocolAmount, RealizationError, SemanticType, SemanticValue};

/// Typed dependency edge in an expression graph.
///
/// Edge direction is:
///
/// ```text
/// dependency -> consumer
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DependencyEdge {
    Operand { position: u32 },
    Left,
    Right,
    RequiredOwners,
    PresentedSigners,
    Condition,
}

/// One typed expression node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpressionNode {
    Fact(FactId),
    Bool(bool),
    Count(Count),
    Amount(ProtocolAmount),

    /// Checked sum of values of one numeric semantic type.
    CheckedSum {
        ty: SemanticType,
        terms: Vec<ExprId>,
    },

    /// Exact equality of two values of one semantic type.
    Equal {
        left: ExprId,
        right: ExprId,
    },

    /// Ordered comparison over counts or protocol amounts.
    LessOrEqual {
        left: ExprId,
        right: ExprId,
    },

    /// Logical conjunction.
    All {
        terms: Vec<ExprId>,
    },

    /// Every owner in `required` occurs in `presented`.
    OwnerSubset {
        required: ExprId,
        presented: ExprId,
    },
}

/// One stable expression declaration stored directly as a Petgraph node weight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpressionDeclaration {
    pub id: ExprId,
    pub ty: SemanticType,
    pub node: ExpressionNode,
}

/// Stable typed projection of one expression dependency edge.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExpressionDependencyProjection {
    pub source: ExprId,
    pub target: ExprId,
    pub edge: DependencyEdge,
}

/// Canonical typed projection of one expression graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpressionGraphProjection {
    pub nodes: Vec<ExpressionDeclaration>,
    pub edges: Vec<ExpressionDependencyProjection>,
}

/// Primitive fact values supplied to the evaluator.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FactValues {
    values: BTreeMap<FactId, SemanticValue>,
}

impl FactValues {
    /// Insert one fact after checking its declared semantic type.
    pub fn insert(&mut self, fact: FactId, value: SemanticValue) -> Result<(), RealizationError> {
        let expected = fact.semantic_type();
        let actual = value.semantic_type();

        if expected != actual {
            return Err(RealizationError::FactTypeMismatch {
                fact,
                expected,
                actual,
            });
        }

        if self.values.insert(fact.clone(), value).is_some() {
            return Err(RealizationError::DuplicateFactValue(fact));
        }

        Ok(())
    }

    /// Read one supplied fact.
    #[must_use]
    pub fn get(&self, fact: &FactId) -> Option<&SemanticValue> {
        self.values.get(fact)
    }

    /// Iterate facts in stable fact-ID order.
    pub fn iter(&self) -> impl Iterator<Item = (&FactId, &SemanticValue)> {
        self.values.iter()
    }
}

/// Complete result of expression evaluation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvaluatedExpressions {
    values: BTreeMap<ExprId, SemanticValue>,
}

impl EvaluatedExpressions {
    /// Read one evaluated expression.
    #[must_use]
    pub fn get(&self, id: &ExprId) -> Option<&SemanticValue> {
        self.values.get(id)
    }

    /// Read a boolean expression.
    pub fn bool(&self, id: &ExprId) -> Result<bool, RealizationError> {
        let value =
            self.values
                .get(id)
                .ok_or_else(|| RealizationError::UnknownEvaluatedExpression {
                    expression: id.clone(),
                })?;

        value
            .as_bool()
            .ok_or_else(|| RealizationError::ExpressionTypeMismatch {
                expression: id.clone(),
                expected: SemanticType::Bool,
                actual: value.semantic_type(),
            })
    }

    /// Iterate evaluated expressions in stable expression-ID order.
    pub fn iter(&self) -> impl Iterator<Item = (&ExprId, &SemanticValue)> {
        self.values.iter()
    }
}

impl FactId {
    /// Semantic type of one primitive fact.
    #[must_use]
    pub const fn semantic_type(&self) -> SemanticType {
        match self {
            Self::FamilyCount { .. } | Self::BoundValue { .. } => SemanticType::Count,

            Self::FamilyAmount { .. } => SemanticType::Amount,

            Self::InputOwners { .. } | Self::Signers { .. } => SemanticType::OwnerSet,

            Self::ProjectionPresent { .. }
            | Self::FamilyRecognized { .. }
            | Self::SponsorIsolated { .. }
            | Self::ProtocolSecretUsed { .. } => SemanticType::Bool,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct PendingDependency {
    dependency: ExprId,
    consumer: ExprId,
    edge: DependencyEdge,
}

/// Build a direct Petgraph expression dependency graph.
///
/// Nodes and edges are inserted in stable typed-key order. Petgraph computes
/// topology and SCC membership; first-party code translates local indices back
/// to stable IDs and canonicalizes unordered SCC output.
#[allow(clippy::type_complexity)]
pub(crate) fn build_expression_graph(
    declarations: impl IntoIterator<Item = ExpressionDeclaration>,
) -> Result<
    (
        DiGraph<ExpressionDeclaration, DependencyEdge, u32>,
        BTreeMap<ExprId, NodeIndex<u32>>,
        Vec<ExprId>,
    ),
    RealizationError,
> {
    let mut by_id = BTreeMap::new();

    for declaration in declarations {
        let id = declaration.id.clone();

        if by_id.insert(id.clone(), declaration).is_some() {
            return Err(RealizationError::DuplicateExpression(id));
        }
    }

    for declaration in by_id.values() {
        validate_expression_declaration(declaration, &by_id)?;
    }

    let mut dependencies = Vec::new();
    for declaration in by_id.values() {
        dependencies.extend(expression_dependencies(declaration)?);
    }
    dependencies.sort();

    let mut graph = DiGraph::<ExpressionDeclaration, DependencyEdge, u32>::with_capacity(
        by_id.len(),
        dependencies.len(),
    );
    let mut node_by_id = BTreeMap::new();

    for declaration in by_id.values() {
        let node = graph.add_node(declaration.clone());
        node_by_id.insert(declaration.id.clone(), node);
    }

    for dependency in dependencies {
        let source = node_by_id[&dependency.dependency];
        let target = node_by_id[&dependency.consumer];
        graph.add_edge(source, target, dependency.edge);
    }

    let evaluation_order = toposort(&graph, None)
        .map_err(|_cycle| RealizationError::ExpressionDependencyCycle {
            components: cyclic_expression_components(&graph),
        })?
        .into_iter()
        .map(|node| graph[node].id.clone())
        .collect();

    Ok((graph, node_by_id, evaluation_order))
}

/// Project a direct Petgraph expression graph into stable typed values.
#[must_use]
pub(crate) fn project_expression_graph(
    graph: &DiGraph<ExpressionDeclaration, DependencyEdge, u32>,
) -> ExpressionGraphProjection {
    let mut nodes = graph.node_weights().cloned().collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.id.cmp(&right.id));

    let mut edges = graph
        .edge_references()
        .map(|edge| ExpressionDependencyProjection {
            source: graph[edge.source()].id.clone(),
            target: graph[edge.target()].id.clone(),
            edge: *edge.weight(),
        })
        .collect::<Vec<_>>();
    edges.sort();

    ExpressionGraphProjection { nodes, edges }
}

/// Evaluate a direct Petgraph expression graph from primitive facts.
#[cfg(test)]
pub(crate) fn evaluate_expressions(
    graph: &DiGraph<ExpressionDeclaration, DependencyEdge, u32>,
    node_by_id: &BTreeMap<ExprId, NodeIndex<u32>>,
    evaluation_order: &[ExprId],
    facts: &FactValues,
) -> Result<EvaluatedExpressions, RealizationError> {
    let mut values = BTreeMap::new();

    for id in evaluation_order {
        let node = node_by_id.get(id).copied().ok_or_else(|| {
            RealizationError::UnknownEvaluatedExpression {
                expression: id.clone(),
            }
        })?;
        let declaration = &graph[node];
        let value = evaluate_node(&declaration.node, facts, &values)?;

        if value.semantic_type() != declaration.ty {
            return Err(RealizationError::ExpressionTypeMismatch {
                expression: declaration.id.clone(),
                expected: declaration.ty,
                actual: value.semantic_type(),
            });
        }

        values.insert(declaration.id.clone(), value);
    }

    Ok(EvaluatedExpressions { values })
}

fn expression_dependencies(
    declaration: &ExpressionDeclaration,
) -> Result<Vec<PendingDependency>, RealizationError> {
    let consumer = declaration.id.clone();
    let dependency = |id: &ExprId, edge| PendingDependency {
        dependency: id.clone(),
        consumer: consumer.clone(),
        edge,
    };

    match &declaration.node {
        ExpressionNode::Fact(_)
        | ExpressionNode::Bool(_)
        | ExpressionNode::Count(_)
        | ExpressionNode::Amount(_) => Ok(Vec::new()),

        ExpressionNode::CheckedSum { terms, .. } | ExpressionNode::All { terms } => terms
            .iter()
            .enumerate()
            .map(|(position, term)| {
                let position = u32::try_from(position).map_err(|_| {
                    RealizationError::TooManyExpressionOperands {
                        expression: declaration.id.clone(),
                    }
                })?;

                Ok(dependency(term, DependencyEdge::Operand { position }))
            })
            .collect(),

        ExpressionNode::Equal { left, right } | ExpressionNode::LessOrEqual { left, right } => {
            Ok(vec![
                dependency(left, DependencyEdge::Left),
                dependency(right, DependencyEdge::Right),
            ])
        }

        ExpressionNode::OwnerSubset {
            required,
            presented,
        } => Ok(vec![
            dependency(required, DependencyEdge::RequiredOwners),
            dependency(presented, DependencyEdge::PresentedSigners),
        ]),
    }
}

fn cyclic_expression_components(
    graph: &DiGraph<ExpressionDeclaration, DependencyEdge, u32>,
) -> Vec<Vec<ExprId>> {
    let mut components = kosaraju_scc(graph)
        .into_iter()
        .filter(|component| {
            component.len() > 1
                || component
                    .first()
                    .is_some_and(|node| graph.find_edge(*node, *node).is_some())
        })
        .map(|component| {
            let mut ids = component
                .into_iter()
                .map(|node| graph[node].id.clone())
                .collect::<Vec<_>>();

            ids.sort();
            ids
        })
        .collect::<Vec<_>>();

    components.sort();
    components
}

fn validate_expression_declaration(
    declaration: &ExpressionDeclaration,
    declarations: &BTreeMap<ExprId, ExpressionDeclaration>,
) -> Result<(), RealizationError> {
    if let ExpressionNode::Fact(fact) = &declaration.node
        && declaration.id != ExprId::Fact(fact.clone())
    {
        return Err(RealizationError::FactExpressionIdentityMismatch {
            expression: declaration.id.clone(),
            fact: fact.clone(),
        });
    }

    for dependency in expression_dependencies(declaration)? {
        if !declarations.contains_key(&dependency.dependency) {
            return Err(RealizationError::UnknownExpressionDependency {
                expression: declaration.id.clone(),
                dependency: dependency.dependency,
            });
        }
    }

    let inferred = infer_node_type(declaration, declarations)?;

    if declaration.ty != inferred {
        return Err(RealizationError::ExpressionTypeMismatch {
            expression: declaration.id.clone(),
            expected: declaration.ty,
            actual: inferred,
        });
    }

    Ok(())
}

fn infer_node_type(
    declaration: &ExpressionDeclaration,
    declarations: &BTreeMap<ExprId, ExpressionDeclaration>,
) -> Result<SemanticType, RealizationError> {
    let dependency_type = |id: &ExprId| {
        declarations
            .get(id)
            .map(|dependency| dependency.ty)
            .ok_or_else(|| RealizationError::UnknownExpressionDependency {
                expression: declaration.id.clone(),
                dependency: id.clone(),
            })
    };

    match &declaration.node {
        ExpressionNode::Fact(fact) => Ok(fact.semantic_type()),
        ExpressionNode::Bool(_) => Ok(SemanticType::Bool),
        ExpressionNode::Count(_) => Ok(SemanticType::Count),
        ExpressionNode::Amount(_) => Ok(SemanticType::Amount),

        ExpressionNode::CheckedSum { ty, terms } => {
            if !matches!(ty, SemanticType::Count | SemanticType::Amount) {
                return Err(RealizationError::InvalidSumType(*ty));
            }

            for term in terms {
                let actual = dependency_type(term)?;

                if actual != *ty {
                    return Err(RealizationError::ExpressionOperandTypeMismatch {
                        expression: term.clone(),
                        expected: *ty,
                        actual,
                    });
                }
            }

            Ok(*ty)
        }

        ExpressionNode::Equal { left, right } | ExpressionNode::LessOrEqual { left, right } => {
            let left_type = dependency_type(left)?;
            let right_type = dependency_type(right)?;

            if left_type != right_type {
                return Err(RealizationError::BinaryOperandTypeMismatch {
                    left: left.clone(),
                    right: right.clone(),
                    left_type,
                    right_type,
                });
            }

            if matches!(declaration.node, ExpressionNode::LessOrEqual { .. })
                && !matches!(left_type, SemanticType::Count | SemanticType::Amount)
            {
                return Err(RealizationError::InvalidOrderedType(left_type));
            }

            Ok(SemanticType::Bool)
        }

        ExpressionNode::All { terms } => {
            for term in terms {
                let actual = dependency_type(term)?;

                if actual != SemanticType::Bool {
                    return Err(RealizationError::ExpressionOperandTypeMismatch {
                        expression: term.clone(),
                        expected: SemanticType::Bool,
                        actual,
                    });
                }
            }

            Ok(SemanticType::Bool)
        }

        ExpressionNode::OwnerSubset {
            required,
            presented,
        } => {
            for operand in [required, presented] {
                let actual = dependency_type(operand)?;

                if actual != SemanticType::OwnerSet {
                    return Err(RealizationError::ExpressionOperandTypeMismatch {
                        expression: operand.clone(),
                        expected: SemanticType::OwnerSet,
                        actual,
                    });
                }
            }

            Ok(SemanticType::Bool)
        }
    }
}

#[cfg(test)]
fn evaluate_node(
    node: &ExpressionNode,
    facts: &FactValues,
    values: &BTreeMap<ExprId, SemanticValue>,
) -> Result<SemanticValue, RealizationError> {
    let dependency = |dependency_id: &ExprId| {
        values
            .get(dependency_id)
            .ok_or_else(|| RealizationError::UnknownEvaluatedExpression {
                expression: dependency_id.clone(),
            })
    };

    match node {
        ExpressionNode::Fact(fact) => facts
            .get(fact)
            .cloned()
            .ok_or_else(|| RealizationError::MissingFactValue(fact.clone())),

        ExpressionNode::Bool(value) => Ok(SemanticValue::Bool(*value)),
        ExpressionNode::Count(value) => Ok(SemanticValue::Count(*value)),
        ExpressionNode::Amount(value) => Ok(SemanticValue::Amount(*value)),

        ExpressionNode::CheckedSum { ty, terms } => evaluate_checked_sum(*ty, terms, values),

        ExpressionNode::Equal { left, right } => {
            Ok(SemanticValue::Bool(dependency(left)? == dependency(right)?))
        }

        ExpressionNode::LessOrEqual { left, right } => evaluate_less_or_equal(left, right, values),

        ExpressionNode::All { terms } => evaluate_all(terms, values),

        ExpressionNode::OwnerSubset {
            required,
            presented,
        } => evaluate_owner_subset(required, presented, values),
    }
}

#[cfg(test)]
fn evaluated_value<'a>(
    values: &'a BTreeMap<ExprId, SemanticValue>,
    id: &ExprId,
) -> Result<&'a SemanticValue, RealizationError> {
    values
        .get(id)
        .ok_or_else(|| RealizationError::UnknownEvaluatedExpression {
            expression: id.clone(),
        })
}

#[cfg(test)]
fn evaluate_checked_sum(
    ty: SemanticType,
    terms: &[ExprId],
    values: &BTreeMap<ExprId, SemanticValue>,
) -> Result<SemanticValue, RealizationError> {
    match ty {
        SemanticType::Count => {
            let mut total = Count::ZERO;

            for term in terms {
                let value = evaluated_value(values, term)?;
                let count =
                    value
                        .as_count()
                        .ok_or_else(|| RealizationError::ExpressionTypeMismatch {
                            expression: term.clone(),
                            expected: SemanticType::Count,
                            actual: value.semantic_type(),
                        })?;

                total = total.checked_add(count)?;
            }

            Ok(SemanticValue::Count(total))
        }

        SemanticType::Amount => {
            let mut total = ProtocolAmount::ZERO;

            for term in terms {
                let value = evaluated_value(values, term)?;
                let amount =
                    value
                        .as_amount()
                        .ok_or_else(|| RealizationError::ExpressionTypeMismatch {
                            expression: term.clone(),
                            expected: SemanticType::Amount,
                            actual: value.semantic_type(),
                        })?;

                total = total.checked_add(amount)?;
            }

            Ok(SemanticValue::Amount(total))
        }

        other => Err(RealizationError::InvalidSumType(other)),
    }
}

#[cfg(test)]
fn evaluate_less_or_equal(
    left: &ExprId,
    right: &ExprId,
    values: &BTreeMap<ExprId, SemanticValue>,
) -> Result<SemanticValue, RealizationError> {
    let left_value = evaluated_value(values, left)?;
    let right_value = evaluated_value(values, right)?;

    let result = match (left_value, right_value) {
        (SemanticValue::Count(left), SemanticValue::Count(right)) => left <= right,
        (SemanticValue::Amount(left), SemanticValue::Amount(right)) => left <= right,
        _ => {
            return Err(RealizationError::BinaryOperandTypeMismatch {
                left: left.clone(),
                right: right.clone(),
                left_type: left_value.semantic_type(),
                right_type: right_value.semantic_type(),
            });
        }
    };

    Ok(SemanticValue::Bool(result))
}

#[cfg(test)]
fn evaluate_all(
    terms: &[ExprId],
    values: &BTreeMap<ExprId, SemanticValue>,
) -> Result<SemanticValue, RealizationError> {
    let mut result = true;

    for term in terms {
        let value = evaluated_value(values, term)?;
        let boolean = value
            .as_bool()
            .ok_or_else(|| RealizationError::ExpressionTypeMismatch {
                expression: term.clone(),
                expected: SemanticType::Bool,
                actual: value.semantic_type(),
            })?;

        result &= boolean;
    }

    Ok(SemanticValue::Bool(result))
}

#[cfg(test)]
fn evaluate_owner_subset(
    required: &ExprId,
    presented: &ExprId,
    values: &BTreeMap<ExprId, SemanticValue>,
) -> Result<SemanticValue, RealizationError> {
    let required_value = evaluated_value(values, required)?;
    let presented_value = evaluated_value(values, presented)?;
    let required_owners =
        required_value
            .as_owner_set()
            .ok_or_else(|| RealizationError::ExpressionTypeMismatch {
                expression: required.clone(),
                expected: SemanticType::OwnerSet,
                actual: required_value.semantic_type(),
            })?;
    let presented_owners =
        presented_value
            .as_owner_set()
            .ok_or_else(|| RealizationError::ExpressionTypeMismatch {
                expression: presented.clone(),
                expected: SemanticType::OwnerSet,
                actual: presented_value.semantic_type(),
            })?;

    Ok(SemanticValue::Bool(
        required_owners.is_subset(presented_owners),
    ))
}
