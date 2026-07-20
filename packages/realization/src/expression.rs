//! Typed expression declarations.
//!
//! The expression vocabulary is intentionally small. Phase 1 needs
//! checked sums, exact equality, ordered comparison, conjunction, and
//! owner-set inclusion. More expressive forms are added only when a
//! concrete semantic operation requires them.

use std::collections::BTreeMap;

use petgraph::{
    algo::{kosaraju_scc, toposort},
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};

use crate::{Count, ExprId, FactId, ProtocolAmount, RealizationError, SemanticType, SemanticValue};

/// Typed dependency edge in an expression graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DependencyEdge {
    Operand { position: u32 },
    Left,
    Right,
    RequiredOwners,
    PresentedSigners,
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

impl ExpressionNode {
    fn dependency_edges(
        &self,
        expression: &ExprId,
    ) -> Result<Vec<(ExprId, DependencyEdge)>, RealizationError> {
        match self {
            Self::Fact(_) | Self::Bool(_) | Self::Count(_) | Self::Amount(_) => Ok(Vec::new()),

            Self::CheckedSum { terms, .. } | Self::All { terms } => terms
                .iter()
                .enumerate()
                .map(|(position, term)| {
                    Ok((
                        term.clone(),
                        DependencyEdge::Operand {
                            position: operand_position(expression, position, terms.len())?,
                        },
                    ))
                })
                .collect(),

            Self::Equal { left, right } | Self::LessOrEqual { left, right } => Ok(vec![
                (left.clone(), DependencyEdge::Left),
                (right.clone(), DependencyEdge::Right),
            ]),

            Self::OwnerSubset {
                required,
                presented,
            } => Ok(vec![
                (required.clone(), DependencyEdge::RequiredOwners),
                (presented.clone(), DependencyEdge::PresentedSigners),
            ]),
        }
    }
}

/// One named expression declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpressionDeclaration {
    pub id: ExprId,
    pub ty: SemanticType,
    pub node: ExpressionNode,
}

/// Frozen deterministic expression registry.
#[derive(Clone, Debug)]
pub struct ExpressionRegistry {
    declarations: BTreeMap<ExprId, ExpressionDeclaration>,
    graph: DiGraph<ExprId, DependencyEdge, u32>,
    node_by_id: BTreeMap<ExprId, NodeIndex<u32>>,
    evaluation_order: Vec<ExprId>,
}

impl ExpressionRegistry {
    /// Validate and freeze expression declarations.
    pub fn new(
        declarations: impl IntoIterator<Item = ExpressionDeclaration>,
    ) -> Result<Self, RealizationError> {
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

        let edge_declarations = expression_dependencies(&by_id)?;
        let mut graph = DiGraph::<ExprId, DependencyEdge, u32>::with_capacity(
            by_id.len(),
            edge_declarations.len(),
        );
        let mut node_by_id = BTreeMap::new();

        for id in by_id.keys() {
            let node = graph.add_node(id.clone());
            node_by_id.insert(id.clone(), node);
        }

        for dependency in &edge_declarations {
            let source = node_by_id[&dependency.dependency];
            let target = node_by_id[&dependency.consumer];
            graph.add_edge(source, target, dependency.edge);
        }

        let evaluation_order = topological_expression_order(&graph)?;

        Ok(Self {
            declarations: by_id,
            graph,
            node_by_id,
            evaluation_order,
        })
    }

    /// Look up one declaration.
    #[must_use]
    pub fn get(&self, id: &ExprId) -> Option<&ExpressionDeclaration> {
        self.declarations.get(id)
    }

    /// Iterate declarations in stable expression-ID order.
    pub fn iter(&self) -> impl Iterator<Item = (&ExprId, &ExpressionDeclaration)> {
        self.declarations.iter()
    }

    /// Canonical dependency-before-consumer evaluation order.
    #[must_use]
    pub fn evaluation_order(&self) -> &[ExprId] {
        &self.evaluation_order
    }

    /// Direct Petgraph dependency graph.
    #[must_use]
    pub const fn dependency_graph(&self) -> &DiGraph<ExprId, DependencyEdge, u32> {
        &self.graph
    }

    /// Return the local Petgraph node for a stable expression ID.
    #[must_use]
    pub fn node_index(&self, id: &ExprId) -> Option<NodeIndex<u32>> {
        self.node_by_id.get(id).copied()
    }

    /// Number of expressions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.declarations.len()
    }

    /// Return whether the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.declarations.is_empty()
    }

    /// Evaluate the complete registry from primitive facts.
    pub fn evaluate(&self, facts: &FactValues) -> Result<EvaluatedExpressions, RealizationError> {
        let mut values = BTreeMap::new();

        for id in &self.evaluation_order {
            let declaration = self.declarations.get(id).ok_or_else(|| {
                RealizationError::UnknownEvaluatedExpression {
                    expression: id.clone(),
                }
            })?;

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
}

impl PartialEq for ExpressionRegistry {
    fn eq(&self, other: &Self) -> bool {
        self.declarations == other.declarations
            && canonical_graph_edges(&self.graph) == canonical_graph_edges(&other.graph)
            && self.evaluation_order == other.evaluation_order
    }
}

impl Eq for ExpressionRegistry {}

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
struct ExpressionDependency {
    dependency: ExprId,
    consumer: ExprId,
    edge: DependencyEdge,
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

    for (dependency, _) in declaration.node.dependency_edges(&declaration.id)? {
        if !declarations.contains_key(&dependency) {
            return Err(RealizationError::UnknownExpressionDependency {
                expression: declaration.id.clone(),
                dependency,
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

fn expression_dependencies(
    declarations: &BTreeMap<ExprId, ExpressionDeclaration>,
) -> Result<Vec<ExpressionDependency>, RealizationError> {
    let mut dependencies = Vec::new();

    for declaration in declarations.values() {
        for (dependency, edge) in declaration.node.dependency_edges(&declaration.id)? {
            dependencies.push(ExpressionDependency {
                dependency,
                consumer: declaration.id.clone(),
                edge,
            });
        }
    }

    dependencies.sort();
    Ok(dependencies)
}

fn topological_expression_order(
    graph: &DiGraph<ExprId, DependencyEdge, u32>,
) -> Result<Vec<ExprId>, RealizationError> {
    match toposort(graph, None) {
        Ok(nodes) => Ok(nodes.into_iter().map(|node| graph[node].clone()).collect()),
        Err(_cycle) => Err(RealizationError::ExpressionDependencyCycle {
            components: cyclic_components(graph),
        }),
    }
}

fn cyclic_components(graph: &DiGraph<ExprId, DependencyEdge, u32>) -> Vec<Vec<ExprId>> {
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
                .map(|node| graph[node].clone())
                .collect::<Vec<_>>();

            ids.sort();
            ids
        })
        .collect::<Vec<_>>();

    components.sort();
    components
}

fn canonical_graph_edges(
    graph: &DiGraph<ExprId, DependencyEdge, u32>,
) -> Vec<(ExprId, ExprId, DependencyEdge)> {
    let mut edges = graph
        .edge_references()
        .map(|edge| {
            (
                graph[edge.source()].clone(),
                graph[edge.target()].clone(),
                *edge.weight(),
            )
        })
        .collect::<Vec<_>>();

    edges.sort();
    edges
}

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

fn operand_position(
    expression: &ExprId,
    position: usize,
    _count: usize,
) -> Result<u32, RealizationError> {
    u32::try_from(position).map_err(|_| RealizationError::TooManyExpressionOperands {
        expression: expression.clone(),
    })
}
