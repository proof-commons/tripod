//! Predicate joins over owner-produced identities and type facts.
use crate::{
    engine_proof::FactKey,
    engine_records::{OperationKey, RelationKey},
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExpressionKey {
    Fact(FactKey),
    Relation { relation: RelationKey, role: u16 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expression {
    pub operation: Option<OperationKey>,
    pub boolean: bool,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Records {
    pub predicates: Vec<(RelationKey, ExpressionKey)>,
    pub expressions: BTreeMap<ExpressionKey, Expression>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    UnknownExpression {
        relation: RelationKey,
        expression: ExpressionKey,
    },
    NonBoolean {
        relation: RelationKey,
        expression: ExpressionKey,
    },
    OutsideScope {
        relation: RelationKey,
        expression: ExpressionKey,
    },
}
pub fn validate_predicates(records: &Records) -> Result<(), Error> {
    let span = tracing::debug_span!(target: "compiler::engine_foundation", "validate_predicates", predicate_count = crate::instrumentation::count(records.predicates.len()), expression_count = crate::instrumentation::count(records.expressions.len()));
    let _entered = span.enter();
    for (relation, expression) in &records.predicates {
        let declaration =
            records
                .expressions
                .get(expression)
                .ok_or_else(|| Error::UnknownExpression {
                    relation: relation.clone(),
                    expression: expression.clone(),
                })?;
        if !declaration.boolean {
            return Err(Error::NonBoolean {
                relation: relation.clone(),
                expression: expression.clone(),
            });
        }
        if declaration
            .operation
            .is_some_and(|operation| operation != relation.operation)
        {
            return Err(Error::OutsideScope {
                relation: relation.clone(),
                expression: expression.clone(),
            });
        }
    }
    Ok(())
}
