//! The internal compiler analysis foundation (Guide-2 aggregate).
//!
//! Combines the exact scoped relation DAG and expression graph and
//! validates the expression-predicate bindings between them. This is
//! deliberately crate-private: a relation graph and a fold result are
//! partial compiler analysis, and no partial analyzed program is
//! exposed before the complete pilot result exists under P2-012.

// The analysis stages have no non-test consumer until proof
// planning (P2-007) and the P2-012 analyzed program; unit tests
// exercise them until then. Remove with the first real consumer.
#![allow(dead_code)]

use crate::{
    CompileError,
    expression::{
        CompilerExpressionAnalysis, CompilerExpressionGraphProjection, build_expression_analysis,
        expression_operation,
    },
    input::BoundCompilerInput,
    relation::{
        CompilerRelationAnalysis, CompilerRelationGraphProjection, build_relation_analysis,
    },
};

/// Internal aggregate of the Guide-2 analyses.
#[derive(Debug)]
pub struct CompilerAnalysisFoundation {
    pub relations: CompilerRelationAnalysis,
    pub expressions: CompilerExpressionAnalysis,
}

/// Stable projection of the complete foundation, for tests and the
/// future P2-012 analyzed program.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompilerAnalysisFoundationProjection {
    pub relations: CompilerRelationGraphProjection,
    pub expressions: CompilerExpressionGraphProjection,
}

impl CompilerAnalysisFoundation {
    #[must_use]
    pub fn project(&self) -> CompilerAnalysisFoundationProjection {
        CompilerAnalysisFoundationProjection {
            relations: self.relations.project(),
            expressions: self.expressions.project(),
        }
    }
}

/// Build and cross-validate the complete analysis foundation.
pub fn analyze_foundation(
    input: &BoundCompilerInput,
) -> Result<CompilerAnalysisFoundation, CompileError> {
    let relations = build_relation_analysis(input)?;
    let mut expressions = build_expression_analysis(input)?;

    validate_predicate_bindings(&relations, &expressions)?;
    crate::fold::fold_expressions(&mut expressions)?;

    Ok(CompilerAnalysisFoundation {
        relations,
        expressions,
    })
}

/// Expression-predicate binding at the compiler boundary.
///
/// The folded boolean result never substitutes for the predicate
/// identity: the relation retains the exact source `ExprId`, because a
/// later target or evidence consumer must know which source relation
/// was discharged.
pub fn validate_predicate_bindings(
    relations: &CompilerRelationAnalysis,
    expressions: &CompilerExpressionAnalysis,
) -> Result<(), CompileError> {
    for node in relations.graph.node_weights() {
        let realization::Relation::ExpressionPredicate { expression } = &node.source.relation
        else {
            continue;
        };

        let Some(expression_node) = expressions.node_by_id.get(expression).copied() else {
            return Err(CompileError::UnknownPredicateExpression {
                relation: node.source.id.clone(),
                expression: expression.clone(),
            });
        };
        let declaration = &expressions.graph[expression_node].source;

        if declaration.ty != realization::SemanticType::Bool {
            return Err(CompileError::NonBooleanPredicateExpression {
                relation: node.source.id.clone(),
                expression: expression.clone(),
                actual: declaration.ty,
            });
        }

        match expression_operation(expression) {
            None => {}
            Some(operation) if operation == node.source.id.operation() => {}
            Some(_) => {
                return Err(CompileError::PredicateExpressionOutsideScope {
                    relation: node.source.id.clone(),
                    expression: expression.clone(),
                });
            }
        }
    }

    Ok(())
}
