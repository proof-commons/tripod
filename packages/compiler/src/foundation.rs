//! The internal compiler analysis foundation (Guide-2 aggregate).
//!
//! Combines the exact scoped relation DAG and expression graph and
//! validates the expression-predicate bindings between them. This is
//! deliberately crate-private: a relation graph and a fold result are
//! partial compiler analysis, and no partial analyzed program is
//! exposed before the complete pilot result exists under P2-012.

#[cfg(test)]
use crate::{
    CompileError, expression::build_expression_analysis, input::BoundCompilerInput,
    relation::build_relation_analysis,
};
use crate::{
    expression::{CompilerExpressionAnalysis, CompilerExpressionGraphProjection},
    relation::{CompilerRelationAnalysis, CompilerRelationGraphProjection},
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
#[cfg(test)]
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
#[cfg(test)]
pub fn validate_predicate_bindings(
    relations: &CompilerRelationAnalysis,
    expressions: &CompilerExpressionAnalysis,
) -> Result<(), CompileError> {
    crate::engine_attestation::foundation::lower(relations, expressions).validate()
}
