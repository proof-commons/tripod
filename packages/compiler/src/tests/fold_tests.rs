//! Checked constant-folding tests (P2-006).
//!
//! The reference oracle here is a deliberately simple, non-folding
//! recursive evaluator written independently of the production fold
//! code. Facts evaluate to Unknown; everything else evaluates from
//! constants with checked arithmetic. For generated constant DAGs the
//! oracle outcome and the fold outcome must agree exactly — value for
//! value, failure class for failure class, unknown for unfolded.

use std::collections::BTreeMap;

use architecture::{BoundId, OperationId};
use proptest::prelude::*;
use realization::{
    Count, DependencyEdge, ExprId, ExpressionDeclaration, ExpressionDependencyProjection,
    ExpressionNode, ExpressionRole, FactId, PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE, ProtocolAmount,
    RelationId, RelationKind, RelationSubject, SemanticType, SemanticValue,
};

use crate::{
    CompileError,
    expression::{CompilerExpressionAnalysis, build_expression_graph},
    fold::fold_expressions,
};

const ASH_SCOPE: &[OperationId] = &[OperationId::CompactAsh];

fn expr(index: u32) -> ExprId {
    // Distinct in-scope IDs: use distinct roles/relations per index.
    let kinds = [
        RelationKind::Cardinality,
        RelationKind::Recognition,
        RelationKind::Authorization,
        RelationKind::Conservation,
        RelationKind::RootPolicy,
        RelationKind::AllowedObjectFamilies,
        RelationKind::CanonicalDeltaPolicy,
        RelationKind::OpenFlowPolicy,
    ];
    let roles = [
        ExpressionRole::Predicate,
        ExpressionRole::Minimum,
        ExpressionRole::Maximum,
        ExpressionRole::Expected,
        ExpressionRole::InputTotal,
        ExpressionRole::OutputTotal,
        ExpressionRole::Condition,
    ];
    let kind = kinds[(index as usize / roles.len()) % kinds.len()];
    let role = roles[index as usize % roles.len()];

    ExprId::relation(
        RelationId::new(OperationId::CompactAsh, kind, RelationSubject::Operation),
        role,
    )
}

fn decl(id: ExprId, ty: SemanticType, node: ExpressionNode) -> ExpressionDeclaration {
    ExpressionDeclaration { id, ty, node }
}

fn count(value: u64) -> ExpressionNode {
    ExpressionNode::Count(Count::new(value))
}

fn amount(value: u64) -> ExpressionNode {
    ExpressionNode::Amount(ProtocolAmount::new(value).expect("amount fixture"))
}

/// Derive dependency edges from node structure, as the realization does.
fn edges_of(declarations: &[ExpressionDeclaration]) -> Vec<ExpressionDependencyProjection> {
    let mut edges = Vec::new();

    for declaration in declarations {
        let mut push = |dependency: &ExprId, edge: DependencyEdge| {
            edges.push(ExpressionDependencyProjection {
                source: dependency.clone(),
                target: declaration.id.clone(),
                edge,
            });
        };

        match &declaration.node {
            ExpressionNode::Fact(_)
            | ExpressionNode::Bool(_)
            | ExpressionNode::Count(_)
            | ExpressionNode::Amount(_) => {}
            ExpressionNode::CheckedSum { terms, .. } | ExpressionNode::All { terms } => {
                for (position, term) in terms.iter().enumerate() {
                    push(
                        term,
                        DependencyEdge::Operand {
                            position: u32::try_from(position).expect("position"),
                        },
                    );
                }
            }
            ExpressionNode::Equal { left, right } | ExpressionNode::LessOrEqual { left, right } => {
                push(left, DependencyEdge::Left);
                push(right, DependencyEdge::Right);
            }
            ExpressionNode::OwnerSubset {
                required,
                presented,
            } => {
                push(required, DependencyEdge::RequiredOwners);
                push(presented, DependencyEdge::PresentedSigners);
            }
        }
    }

    edges
}

fn build(declarations: &[ExpressionDeclaration]) -> CompilerExpressionAnalysis {
    build_expression_graph(ASH_SCOPE, declarations, &edges_of(declarations)).expect("graph builds")
}

fn fold(
    declarations: &[ExpressionDeclaration],
) -> Result<CompilerExpressionAnalysis, CompileError> {
    let mut analysis = build(declarations);
    fold_expressions(&mut analysis)?;
    Ok(analysis)
}

fn folded_value(analysis: &CompilerExpressionAnalysis, id: &ExprId) -> Option<SemanticValue> {
    analysis
        .project()
        .nodes
        .into_iter()
        .find(|node| &node.id == id)
        .expect("node present")
        .folded_value
}

// --- independent reference oracle ---

#[derive(Clone, Debug, PartialEq, Eq)]
enum ReferenceOutcome {
    Value(SemanticValue),
    Unknown,
    Failure,
}

fn reference_evaluate(
    id: &ExprId,
    declarations: &BTreeMap<ExprId, ExpressionDeclaration>,
) -> ReferenceOutcome {
    use ReferenceOutcome::{Failure, Unknown, Value};

    let recurse = |dependency: &ExprId| reference_evaluate(dependency, declarations);

    match &declarations[id].node {
        ExpressionNode::Fact(_) | ExpressionNode::OwnerSubset { .. } => Unknown,
        ExpressionNode::Bool(value) => Value(SemanticValue::Bool(*value)),
        ExpressionNode::Count(value) => Value(SemanticValue::Count(*value)),
        ExpressionNode::Amount(value) => Value(SemanticValue::Amount(*value)),

        ExpressionNode::CheckedSum { ty, terms } => {
            let mut total: u128 = 0;
            let mut unknown = false;

            for term in terms {
                match recurse(term) {
                    Value(SemanticValue::Count(value)) if *ty == SemanticType::Count => {
                        total += u128::from(value.get());
                    }
                    Value(SemanticValue::Amount(value)) if *ty == SemanticType::Amount => {
                        total += u128::from(value.get());
                    }
                    Unknown => unknown = true,
                    Failure | Value(_) => return Failure,
                }
            }

            if unknown {
                return Unknown;
            }

            match ty {
                SemanticType::Count => u64::try_from(total).map_or(Failure, |value| {
                    Value(SemanticValue::Count(Count::new(value)))
                }),
                SemanticType::Amount => {
                    if total < u128::from(PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE) {
                        Value(SemanticValue::Amount(
                            ProtocolAmount::new(u64::try_from(total).expect("in domain"))
                                .expect("in domain"),
                        ))
                    } else {
                        Failure
                    }
                }
                _ => Failure,
            }
        }

        ExpressionNode::Equal { left, right } => match (recurse(left), recurse(right)) {
            (Failure, _) | (_, Failure) => Failure,
            (Unknown, _) | (_, Unknown) => Unknown,
            (Value(left_value), Value(right_value)) => {
                if left_value.semantic_type() == right_value.semantic_type() {
                    Value(SemanticValue::Bool(left_value == right_value))
                } else {
                    Failure
                }
            }
        },

        ExpressionNode::LessOrEqual { left, right } => match (recurse(left), recurse(right)) {
            (Failure, _) | (_, Failure) => Failure,
            (Unknown, _) | (_, Unknown) => Unknown,
            (Value(SemanticValue::Count(left)), Value(SemanticValue::Count(right))) => {
                Value(SemanticValue::Bool(left <= right))
            }
            (Value(SemanticValue::Amount(left)), Value(SemanticValue::Amount(right))) => {
                Value(SemanticValue::Bool(left <= right))
            }
            _ => Failure,
        },

        ExpressionNode::All { terms } => {
            let mut result = true;
            let mut unknown = false;

            for term in terms {
                match recurse(term) {
                    Value(SemanticValue::Bool(value)) => result &= value,
                    Unknown => unknown = true,
                    Failure | Value(_) => return Failure,
                }
            }

            if unknown {
                Unknown
            } else {
                Value(SemanticValue::Bool(result))
            }
        }
    }
}

/// Compare the fold outcome for every declaration with the oracle.
fn assert_agrees_with_oracle(declarations: &[ExpressionDeclaration]) {
    let by_id = declarations
        .iter()
        .map(|declaration| (declaration.id.clone(), declaration.clone()))
        .collect::<BTreeMap<_, _>>();
    let any_failure = by_id
        .keys()
        .any(|id| reference_evaluate(id, &by_id) == ReferenceOutcome::Failure);

    match fold(declarations) {
        Err(_) => assert!(any_failure, "fold failed but the oracle sees no failure"),
        Ok(analysis) => {
            assert!(!any_failure, "oracle sees a failure the fold missed");

            for node in analysis.project().nodes {
                let expected = match reference_evaluate(&node.id, &by_id) {
                    ReferenceOutcome::Value(value) => Some(value),
                    ReferenceOutcome::Unknown => None,
                    ReferenceOutcome::Failure => unreachable!("no failure in this branch"),
                };

                assert_eq!(node.folded_value, expected, "disagreement on {:?}", node.id);
            }
        }
    }
}

// --- literals and closed operations ---

#[test]
fn literals_fold() {
    let ids = [expr(0), expr(1), expr(2), expr(3)];
    let declarations = [
        decl(
            ids[0].clone(),
            SemanticType::Bool,
            ExpressionNode::Bool(true),
        ),
        decl(
            ids[1].clone(),
            SemanticType::Bool,
            ExpressionNode::Bool(false),
        ),
        decl(ids[2].clone(), SemanticType::Count, count(0)),
        decl(ids[3].clone(), SemanticType::Amount, amount(1)),
    ];
    let analysis = fold(&declarations).expect("folds");

    assert_eq!(
        folded_value(&analysis, &ids[0]),
        Some(SemanticValue::Bool(true))
    );
    assert_eq!(
        folded_value(&analysis, &ids[1]),
        Some(SemanticValue::Bool(false))
    );
    assert_eq!(
        folded_value(&analysis, &ids[2]),
        Some(SemanticValue::Count(Count::ZERO))
    );
    assert_eq!(
        folded_value(&analysis, &ids[3]),
        Some(SemanticValue::Amount(ProtocolAmount::ONE))
    );
}

#[test]
fn closed_checked_sums_fold() {
    let (a, b, total) = (expr(0), expr(1), expr(2));
    let declarations = [
        decl(a.clone(), SemanticType::Count, count(1)),
        decl(b.clone(), SemanticType::Count, count(2)),
        decl(
            total.clone(),
            SemanticType::Count,
            ExpressionNode::CheckedSum {
                ty: SemanticType::Count,
                terms: vec![a, b],
            },
        ),
    ];
    let analysis = fold(&declarations).expect("folds");

    assert_eq!(
        folded_value(&analysis, &total),
        Some(SemanticValue::Count(Count::new(3)))
    );

    // Empty sums fold to the typed zero.
    let empty_count = expr(3);
    let empty_amount = expr(4);
    let declarations = [
        decl(
            empty_count.clone(),
            SemanticType::Count,
            ExpressionNode::CheckedSum {
                ty: SemanticType::Count,
                terms: Vec::new(),
            },
        ),
        decl(
            empty_amount.clone(),
            SemanticType::Amount,
            ExpressionNode::CheckedSum {
                ty: SemanticType::Amount,
                terms: Vec::new(),
            },
        ),
    ];
    let analysis = fold(&declarations).expect("folds");
    assert_eq!(
        folded_value(&analysis, &empty_count),
        Some(SemanticValue::Count(Count::ZERO))
    );
    assert_eq!(
        folded_value(&analysis, &empty_amount),
        Some(SemanticValue::Amount(ProtocolAmount::ZERO))
    );
}

#[test]
fn count_overflow_fails() {
    let (a, b, total) = (expr(0), expr(1), expr(2));
    let declarations = [
        decl(a.clone(), SemanticType::Count, count(u64::MAX)),
        decl(b.clone(), SemanticType::Count, count(1)),
        decl(
            total.clone(),
            SemanticType::Count,
            ExpressionNode::CheckedSum {
                ty: SemanticType::Count,
                terms: vec![a, b],
            },
        ),
    ];

    assert_eq!(
        fold(&declarations).unwrap_err(),
        CompileError::ConstantFoldOverflow { expression: total },
    );
}

#[test]
fn amount_domain_exit_fails_with_the_offending_total() {
    let maximum = PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 1;
    let (a, b, total) = (expr(0), expr(1), expr(2));
    let declarations = [
        decl(a.clone(), SemanticType::Amount, amount(maximum)),
        decl(b.clone(), SemanticType::Amount, amount(1)),
        decl(
            total.clone(),
            SemanticType::Amount,
            ExpressionNode::CheckedSum {
                ty: SemanticType::Amount,
                terms: vec![a.clone(), b],
            },
        ),
    ];

    assert_eq!(
        fold(&declarations).unwrap_err(),
        CompileError::ConstantFoldAmountOutOfDomain {
            expression: total,
            value: maximum + 1,
        },
    );

    // The maximum valid amount itself folds.
    let alone = expr(3);
    let declarations = [
        decl(a.clone(), SemanticType::Amount, amount(maximum)),
        decl(
            alone.clone(),
            SemanticType::Amount,
            ExpressionNode::CheckedSum {
                ty: SemanticType::Amount,
                terms: vec![a],
            },
        ),
    ];
    let analysis = fold(&declarations).expect("folds");
    assert_eq!(
        folded_value(&analysis, &alone),
        Some(SemanticValue::Amount(ProtocolAmount::new(maximum).unwrap()))
    );
}

#[test]
fn closed_equality_and_ordering_fold() {
    let (a, b, eq, le, gt) = (expr(0), expr(1), expr(2), expr(3), expr(4));
    let declarations = [
        decl(a.clone(), SemanticType::Count, count(2)),
        decl(b.clone(), SemanticType::Count, count(3)),
        decl(
            eq.clone(),
            SemanticType::Bool,
            ExpressionNode::Equal {
                left: a.clone(),
                right: a.clone(),
            },
        ),
        decl(
            le.clone(),
            SemanticType::Bool,
            ExpressionNode::LessOrEqual {
                left: a.clone(),
                right: b.clone(),
            },
        ),
        decl(
            gt.clone(),
            SemanticType::Bool,
            ExpressionNode::LessOrEqual { left: b, right: a },
        ),
    ];
    let analysis = fold(&declarations).expect("folds");

    assert_eq!(
        folded_value(&analysis, &eq),
        Some(SemanticValue::Bool(true))
    );
    assert_eq!(
        folded_value(&analysis, &le),
        Some(SemanticValue::Bool(true))
    );
    assert_eq!(
        folded_value(&analysis, &gt),
        Some(SemanticValue::Bool(false))
    );
}

// --- conjunction discipline ---

fn bound_fact() -> ExpressionDeclaration {
    let fact = FactId::BoundValue {
        bound: BoundId::AshBatchMax,
    };

    decl(
        ExprId::fact(fact.clone()),
        SemanticType::Count,
        ExpressionNode::Fact(fact),
    )
}

fn unknown_boolean(id: ExprId) -> [ExpressionDeclaration; 2] {
    // fact ≤ fact: well-typed, boolean, and unknowable at compile time.
    let fact = bound_fact();

    [
        fact.clone(),
        decl(
            id,
            SemanticType::Bool,
            ExpressionNode::LessOrEqual {
                left: fact.id.clone(),
                right: fact.id,
            },
        ),
    ]
}

#[test]
fn conjunction_folds_only_when_fully_closed() {
    // Empty and all-constant conjunctions fold.
    let (t, f, all_true, with_false, empty) = (expr(0), expr(1), expr(2), expr(3), expr(4));
    let declarations = [
        decl(t.clone(), SemanticType::Bool, ExpressionNode::Bool(true)),
        decl(f.clone(), SemanticType::Bool, ExpressionNode::Bool(false)),
        decl(
            all_true.clone(),
            SemanticType::Bool,
            ExpressionNode::All {
                terms: vec![t.clone(), t.clone()],
            },
        ),
        decl(
            with_false.clone(),
            SemanticType::Bool,
            ExpressionNode::All { terms: vec![t, f] },
        ),
        decl(
            empty.clone(),
            SemanticType::Bool,
            ExpressionNode::All { terms: Vec::new() },
        ),
    ];
    let analysis = fold(&declarations).expect("folds");
    assert_eq!(
        folded_value(&analysis, &all_true),
        Some(SemanticValue::Bool(true))
    );
    assert_eq!(
        folded_value(&analysis, &with_false),
        Some(SemanticValue::Bool(false))
    );
    assert_eq!(
        folded_value(&analysis, &empty),
        Some(SemanticValue::Bool(true))
    );
}

#[test]
fn false_and_unknown_remains_unfolded() {
    let unknown = expr(0);
    let conjunction = expr(1);
    let f = expr(2);
    let mut declarations = unknown_boolean(unknown.clone()).to_vec();
    declarations.push(decl(
        f.clone(),
        SemanticType::Bool,
        ExpressionNode::Bool(false),
    ));
    declarations.push(decl(
        conjunction.clone(),
        SemanticType::Bool,
        ExpressionNode::All {
            terms: vec![f, unknown],
        },
    ));

    let analysis = fold(&declarations).expect("folds without deciding");
    assert_eq!(folded_value(&analysis, &conjunction), None);
}

#[test]
fn false_and_overflowing_constant_dependency_fails() {
    // A short-circuit fold would hide the checked-arithmetic failure
    // behind the constant false. It must fail instead.
    let (a, b, total, is_zero, f, conjunction) =
        (expr(0), expr(1), expr(2), expr(3), expr(4), expr(5));
    let declarations = [
        decl(a.clone(), SemanticType::Count, count(u64::MAX)),
        decl(b.clone(), SemanticType::Count, count(1)),
        decl(
            total.clone(),
            SemanticType::Count,
            ExpressionNode::CheckedSum {
                ty: SemanticType::Count,
                terms: vec![a.clone(), b],
            },
        ),
        decl(
            is_zero.clone(),
            SemanticType::Bool,
            ExpressionNode::Equal {
                left: total.clone(),
                right: a,
            },
        ),
        decl(f.clone(), SemanticType::Bool, ExpressionNode::Bool(false)),
        decl(
            conjunction,
            SemanticType::Bool,
            ExpressionNode::All {
                terms: vec![f, is_zero],
            },
        ),
    ];

    assert_eq!(
        fold(&declarations).unwrap_err(),
        CompileError::ConstantFoldOverflow { expression: total },
    );
}

// --- facts stay unfolded ---

#[test]
fn facts_and_owner_subsets_remain_unfolded() {
    let fact = bound_fact();
    let comparison = expr(0);
    let mut declarations = unknown_boolean(comparison.clone()).to_vec();

    // An unrelated literal does not alter another expression's fold.
    declarations.push(decl(expr(1), SemanticType::Count, count(7)));

    let analysis = fold(&declarations).expect("folds");
    assert_eq!(folded_value(&analysis, &fact.id), None);
    assert_eq!(folded_value(&analysis, &comparison), None);
}

// --- oracle agreement and permutation properties ---

#[test]
fn repeated_folding_is_equal() {
    let (a, b, total) = (expr(0), expr(1), expr(2));
    let declarations = [
        decl(a.clone(), SemanticType::Count, count(1)),
        decl(b.clone(), SemanticType::Count, count(2)),
        decl(
            total,
            SemanticType::Count,
            ExpressionNode::CheckedSum {
                ty: SemanticType::Count,
                terms: vec![a, b],
            },
        ),
    ];

    let first = fold(&declarations).expect("folds").project();
    let second = fold(&declarations).expect("folds").project();
    assert_eq!(first, second);
}

proptest! {
    #[test]
    fn generated_constant_sums_agree_with_the_oracle(
        values in proptest::collection::vec(0_u64..=u64::MAX / 2, 1..6),
        include_fact in any::<bool>(),
    ) {
        let mut declarations = Vec::new();
        let mut terms = Vec::new();

        for (index, value) in values.iter().enumerate() {
            let id = expr(u32::try_from(index).expect("index"));
            declarations.push(decl(id.clone(), SemanticType::Count, count(*value)));
            terms.push(id);
        }

        if include_fact {
            let fact = bound_fact();
            terms.push(fact.id.clone());
            declarations.push(fact);
        }

        let total = expr(40);
        declarations.push(decl(
            total,
            SemanticType::Count,
            ExpressionNode::CheckedSum {
                ty: SemanticType::Count,
                terms,
            },
        ));

        assert_agrees_with_oracle(&declarations);
    }

    #[test]
    fn declaration_permutations_preserve_folded_projection(
        permutation in Just((0..5_u32).collect::<Vec<_>>()).prop_shuffle(),
    ) {
        let build_declarations = |order: &[u32]| {
            let (a, b, total, eq, t) = (expr(0), expr(1), expr(2), expr(3), expr(4));
            let all = [
                decl(a.clone(), SemanticType::Count, count(1)),
                decl(b.clone(), SemanticType::Count, count(2)),
                decl(
                    total.clone(),
                    SemanticType::Count,
                    ExpressionNode::CheckedSum {
                        ty: SemanticType::Count,
                        terms: vec![a, b],
                    },
                ),
                decl(
                    eq,
                    SemanticType::Bool,
                    ExpressionNode::Equal {
                        left: total.clone(),
                        right: total,
                    },
                ),
                decl(t, SemanticType::Bool, ExpressionNode::Bool(true)),
            ];

            order
                .iter()
                .map(|index| all[*index as usize].clone())
                .collect::<Vec<_>>()
        };

        let baseline = fold(&build_declarations(&[0, 1, 2, 3, 4]))
            .expect("baseline folds")
            .project();
        let permuted = fold(&build_declarations(&permutation))
            .expect("permutation folds")
            .project();

        prop_assert_eq!(baseline, permuted);
    }
}
