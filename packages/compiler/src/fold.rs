//! Conservative checked constant folding (P2-006).
//!
//! An expression folds only when its complete value is derivable from
//! constants — never from algebraic identities over unknown operands.
//! `false ∧ unknown` stays unfolded: the unknown branch might be
//! missing or fail checked arithmetic, and short-circuit folding would
//! change failure behavior. Checked count and protocol-amount
//! arithmetic is used throughout; an overflow or amount-domain failure
//! fails compilation with a focused typed error rather than leaving
//! the expression unfolded.
//!
//! Facts never fold. An architecture bound value is a runtime or
//! deployment value, not a compile-time constant. Source nodes, IDs,
//! and dependency edges all remain in place as provenance: the fold
//! result is derived analysis attached beside the source, never a
//! replacement for it.

// The analysis stages have no non-test consumer until proof
// planning (P2-007) and the P2-012 analyzed program; unit tests
// exercise them until then. Remove with the first real consumer.
#![allow(dead_code)]

use std::collections::BTreeMap;

use realization::{Count, ExprId, ExpressionNode, ProtocolAmount, SemanticType, SemanticValue};

use crate::{CompileError, expression::CompilerExpressionAnalysis};

/// Fold every fully closed constant expression in place.
///
/// Processes the internal graph in topological order; each node's
/// optional folded value is written into its [`crate::expression::CompilerExpressionNode`].
pub fn fold_expressions(analysis: &mut CompilerExpressionAnalysis) -> Result<(), CompileError> {
    let mut folded: BTreeMap<ExprId, SemanticValue> = BTreeMap::new();

    for id in &analysis.evaluation_order {
        let node = analysis.node_by_id.get(id).copied().ok_or_else(|| {
            CompileError::ConstantFoldMissingOperand {
                expression: id.clone(),
                operand: id.clone(),
            }
        })?;
        let declaration = analysis.graph[node].source.clone();
        let value = fold_node(&declaration.id, &declaration.node, &folded)?;

        if let Some(value) = value {
            let actual = value.semantic_type();

            if actual != declaration.ty {
                return Err(CompileError::ConstantFoldTypeMismatch {
                    expression: declaration.id.clone(),
                    expected: declaration.ty,
                    actual,
                });
            }

            folded.insert(declaration.id, value.clone());
            analysis.graph[node].folded_value = Some(value);
        }
    }

    Ok(())
}

/// Fold one node from already folded operands, or return `None` when
/// any operand is unknown.
fn fold_node(
    id: &ExprId,
    node: &ExpressionNode,
    folded: &BTreeMap<ExprId, SemanticValue>,
) -> Result<Option<SemanticValue>, CompileError> {
    match node {
        // A fact is a runtime or deployment value; never a constant.
        ExpressionNode::Fact(_) => Ok(None),

        ExpressionNode::Bool(value) => Ok(Some(SemanticValue::Bool(*value))),
        ExpressionNode::Count(value) => Ok(Some(SemanticValue::Count(*value))),
        ExpressionNode::Amount(value) => Ok(Some(SemanticValue::Amount(*value))),

        ExpressionNode::CheckedSum { ty, terms } => fold_checked_sum(id, *ty, terms, folded),

        ExpressionNode::Equal { left, right } => {
            let (Some(left_value), Some(right_value)) = (folded.get(left), folded.get(right))
            else {
                return Ok(None);
            };

            if left_value.semantic_type() != right_value.semantic_type() {
                return Err(CompileError::ConstantFoldTypeMismatch {
                    expression: id.clone(),
                    expected: left_value.semantic_type(),
                    actual: right_value.semantic_type(),
                });
            }

            Ok(Some(SemanticValue::Bool(left_value == right_value)))
        }

        ExpressionNode::LessOrEqual { left, right } => {
            let (Some(left_value), Some(right_value)) = (folded.get(left), folded.get(right))
            else {
                return Ok(None);
            };

            let result = match (left_value, right_value) {
                (SemanticValue::Count(left), SemanticValue::Count(right)) => left <= right,
                (SemanticValue::Amount(left), SemanticValue::Amount(right)) => left <= right,
                _ => {
                    return Err(CompileError::ConstantFoldTypeMismatch {
                        expression: id.clone(),
                        expected: left_value.semantic_type(),
                        actual: right_value.semantic_type(),
                    });
                }
            };

            Ok(Some(SemanticValue::Bool(result)))
        }

        // Fold only when every term is a folded boolean. No
        // short-circuit: `false ∧ unknown` stays unfolded. The empty
        // conjunction folds to true, matching the source evaluator.
        ExpressionNode::All { terms } => {
            let mut result = true;

            for term in terms {
                let Some(value) = folded.get(term) else {
                    return Ok(None);
                };
                let Some(boolean) = value.as_bool() else {
                    return Err(CompileError::ConstantFoldTypeMismatch {
                        expression: id.clone(),
                        expected: SemanticType::Bool,
                        actual: value.semantic_type(),
                    });
                };

                result &= boolean;
            }

            Ok(Some(SemanticValue::Bool(result)))
        }

        // Owner sets currently arise only from facts, so both operands
        // constant is unreachable from real declarations; the arm is
        // still exact for completeness.
        ExpressionNode::OwnerSubset {
            required,
            presented,
        } => {
            let (Some(required_value), Some(presented_value)) =
                (folded.get(required), folded.get(presented))
            else {
                return Ok(None);
            };
            let (Some(required_owners), Some(presented_owners)) = (
                required_value.as_owner_set(),
                presented_value.as_owner_set(),
            ) else {
                return Err(CompileError::ConstantFoldTypeMismatch {
                    expression: id.clone(),
                    expected: SemanticType::OwnerSet,
                    actual: required_value.semantic_type(),
                });
            };

            Ok(Some(SemanticValue::Bool(
                required_owners.is_subset(presented_owners),
            )))
        }
    }
}

fn fold_checked_sum(
    id: &ExprId,
    ty: SemanticType,
    terms: &[ExprId],
    folded: &BTreeMap<ExprId, SemanticValue>,
) -> Result<Option<SemanticValue>, CompileError> {
    // Every term must be a folded constant of the declared type before
    // any arithmetic decision is made.
    let mut values = Vec::with_capacity(terms.len());

    for term in terms {
        let Some(value) = folded.get(term) else {
            return Ok(None);
        };

        if value.semantic_type() != ty {
            return Err(CompileError::ConstantFoldTypeMismatch {
                expression: id.clone(),
                expected: ty,
                actual: value.semantic_type(),
            });
        }

        values.push(value);
    }

    match ty {
        SemanticType::Count => {
            let mut total = Count::ZERO;

            for value in values {
                let term = value.as_count().expect("type checked above");
                total =
                    total
                        .checked_add(term)
                        .map_err(|_| CompileError::ConstantFoldOverflow {
                            expression: id.clone(),
                        })?;
            }

            Ok(Some(SemanticValue::Count(total)))
        }

        SemanticType::Amount => {
            // Distinguish u64 overflow from an amount-domain failure so
            // the diagnostic names the offending total where one exists.
            let mut total: u128 = 0;

            for value in values {
                let term = value.as_amount().expect("type checked above");
                total += u128::from(term.get());
            }

            let value = u64::try_from(total).map_err(|_| CompileError::ConstantFoldOverflow {
                expression: id.clone(),
            })?;
            let amount = ProtocolAmount::new(value).map_err(|_| {
                CompileError::ConstantFoldAmountOutOfDomain {
                    expression: id.clone(),
                    value,
                }
            })?;

            Ok(Some(SemanticValue::Amount(amount)))
        }

        other => Err(CompileError::ConstantFoldTypeMismatch {
            expression: id.clone(),
            expected: SemanticType::Count,
            actual: other,
        }),
    }
}
