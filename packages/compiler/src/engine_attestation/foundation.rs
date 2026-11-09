//! Attestation predicate facts and lossless refusal identities.
use crate::{
    CompileError, engine_foundation as core,
    expression::{CompilerExpressionAnalysis, expression_operation},
    relation::CompilerRelationAnalysis,
};
use std::collections::BTreeMap;

pub struct Lowered {
    pub records: core::Records,
    relations: BTreeMap<crate::engine_records::RelationKey, realization::RelationId>,
    expressions: BTreeMap<core::ExpressionKey, realization::ExprId>,
    types: BTreeMap<core::ExpressionKey, realization::SemanticType>,
}

pub fn lower(
    relations: &CompilerRelationAnalysis,
    expressions: &CompilerExpressionAnalysis,
) -> Lowered {
    let span = tracing::debug_span!(target: "compiler::engine_attestation", "lower_predicates", relation_count = crate::instrumentation::count(relations.graph.node_count()), expression_count = crate::instrumentation::count(expressions.graph.node_count()));
    let _entered = span.enter();
    let mut lowered = Lowered {
        records: core::Records::default(),
        relations: BTreeMap::new(),
        expressions: BTreeMap::new(),
        types: BTreeMap::new(),
    };
    for node in relations.graph.node_weights() {
        if let realization::Relation::ExpressionPredicate { expression } = &node.source.relation {
            let relation = super::relation_key(&node.source.id);
            let key = expression_key(expression);
            lowered
                .relations
                .insert(relation.clone(), node.source.id.clone());
            lowered.expressions.insert(key.clone(), expression.clone());
            lowered.records.predicates.push((relation, key));
        }
    }
    for node in expressions.graph.node_weights() {
        let id = &node.source.id;
        let key = expression_key(id);
        lowered.expressions.insert(key.clone(), id.clone());
        lowered.types.insert(key.clone(), node.source.ty);
        lowered.records.expressions.insert(
            key,
            core::Expression {
                operation: expression_operation(id).map(super::operation_key),
                boolean: node.source.ty == realization::SemanticType::Bool,
            },
        );
    }
    lowered
}

impl Lowered {
    pub fn validate(&self) -> Result<(), CompileError> {
        core::validate_predicates(&self.records).map_err(|error| match error {
            core::Error::UnknownExpression {
                relation,
                expression,
            } => CompileError::UnknownPredicateExpression {
                relation: self.relations[&relation].clone(),
                expression: self.expressions[&expression].clone(),
            },
            core::Error::NonBoolean {
                relation,
                expression,
            } => CompileError::NonBooleanPredicateExpression {
                relation: self.relations[&relation].clone(),
                expression: self.expressions[&expression].clone(),
                actual: self.types[&expression],
            },
            core::Error::OutsideScope {
                relation,
                expression,
            } => CompileError::PredicateExpressionOutsideScope {
                relation: self.relations[&relation].clone(),
                expression: self.expressions[&expression].clone(),
            },
        })
    }
}

fn expression_key(id: &realization::ExprId) -> core::ExpressionKey {
    match id {
        realization::ExprId::Fact(fact) => {
            core::ExpressionKey::Fact(crate::engine_proof::FactKey {
                namespace: super::ATTESTATION,
                subject: super::constructibility::fact_key(fact),
            })
        }
        realization::ExprId::Relation { relation, role } => core::ExpressionKey::Relation {
            relation: super::relation_key(relation),
            role: *role as u16,
        },
    }
}
