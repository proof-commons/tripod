//! Exhaustive adapter tables use real owner registry keys and independent expectations.
use super::{
    Prepared, refusal,
    tests::{bound, limits},
};
use crate::{
    engine_placement as p, engine_proof as proof, engine_records as r, engine_source as s,
};
use crate::{
    engine_program as core,
    keyed_plan::{Defect as D, KeyedPlanError as E, Stage as S},
};
use architecture::semantic::KeyedOperationId as K;
type Rows = Vec<(core::Error, E)>;
struct Keys {
    other_relation: r::RelationKey,
    other_id: realization::relation::KeyedRelationId,
    other_case: p::Key,
    relation: r::RelationKey,
    id: realization::relation::KeyedRelationId,
    case: p::Key,
    alternative: proof::AlternativeKey,
    row: s::SourceRow,
    evidence: s::Evidence,
    carrier: p::CarrierKey,
    layout: p::LayoutKey,
    authorization: crate::engine_constructibility::Authorization,
}
impl Keys {
    fn both(&self, stage: S, defect: D) -> E {
        let relations = std::collections::BTreeSet::from([self.id, self.other_id]);
        E::Refused {
            stage,
            defect,
            operations: relations
                .iter()
                .map(|id| id.operation)
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect(),
            relations: relations.into_iter().collect(),
            unrestored_key_count: 0,
        }
    }

    fn refused(&self, stage: S, defect: D) -> E {
        E::Refused {
            stage,
            defect,
            relations: vec![self.id],
            operations: vec![self.id.operation],
            unrestored_key_count: 0,
        }
    }
    fn operation(&self, stage: S, defect: D) -> E {
        E::Refused {
            stage,
            defect,
            relations: Vec::new(),
            operations: vec![self.id.operation],
            unrestored_key_count: 0,
        }
    }
    fn empty(stage: S, defect: D) -> E {
        E::Refused {
            stage,
            defect,
            relations: Vec::new(),
            operations: Vec::new(),
            unrestored_key_count: 0,
        }
    }
}
#[test]
fn keyed_refusal_table_restores_every_adapter_arm() {
    let input = bound(&[K::Source, K::Merge]);
    let prepared = Prepared::new(&input).unwrap();
    assert!(prepared.analyze(limits()).is_ok());
    for table in &prepared.records.sources.tables {
        let other_table = prepared
            .records
            .sources
            .tables
            .iter()
            .find(|other| other.relation.operation != table.relation.operation)
            .unwrap();
        let other_relation = other_table.relation.clone();
        let other_id = prepared.relation(&other_relation).unwrap();
        let other_case = prepared
            .records
            .cases
            .rows
            .iter()
            .find(|row| row.key.relation == other_relation)
            .unwrap()
            .key
            .clone();
        let relation = table.relation.clone();
        let id = prepared.relation(&relation).unwrap();
        let case = prepared
            .records
            .cases
            .rows
            .iter()
            .find(|row| row.key.relation == relation)
            .unwrap()
            .key
            .clone();
        let attestation = crate::tests::bound_input(&[architecture::OperationId::CompactAsh]);
        let (legacy, _) =
            crate::engine_attestation::program::test_preparation(&attestation).unwrap();
        let keys = Keys {
            other_relation,
            other_id,
            other_case,
            id,
            case,
            alternative: proof::AlternativeKey {
                relation: relation.clone(),
                namespace: table.proof_namespace,
                kind: table.proof_kind,
            },
            row: table.rows[0].clone(),
            evidence: prepared.evidence.keys().next().unwrap().clone(),
            carrier: prepared.identities.carriers.keys().next().unwrap().clone(),
            layout: legacy.cases.layout_scopes.keys().next().unwrap().clone(),
            authorization: legacy.constructibility.operations[0].1[0].clone(),
            relation,
        };
        let rows = [
            relations(&keys),
            sources(&keys),
            joins(&keys),
            proofs(&keys),
            predicates(&keys),
            construction(&keys),
            lifecycle(&keys),
            requirements(&keys),
            cases(&keys),
            placement(&keys),
            coverage(&keys),
            graph(&keys),
            wrappers(&keys),
        ];
        for (error, expected) in rows.into_iter().flatten() {
            assert_eq!(prepared.refusal(error.clone()), expected, "{error:?}");
        }
        for (error, expected) in lowering_rows(&keys) {
            assert_eq!(
                refusal::lowering(input.owner(), error.clone()),
                expected,
                "{error:?}"
            );
        }
    }
}

fn relations(k: &Keys) -> Rows {
    vec![
        (
            core::Error::Relations(r::RelationRecordError::ForeignProofOwner {
                relation: k.relation.clone(),
                alternative_relation: k.other_relation.clone(),
            }),
            k.both(S::Relations, D::ForeignProofOwner),
        ),
        (
            core::Error::Relations(r::RelationRecordError::DuplicateRelation {
                relation: k.relation.clone(),
            }),
            k.refused(S::Relations, D::DuplicateRelation),
        ),
        (
            core::Error::Relations(r::RelationRecordError::UnknownEndpoint {
                relation: k.relation.clone(),
            }),
            k.refused(S::Relations, D::UnknownEndpoint),
        ),
        (
            core::Error::Relations(r::RelationRecordError::EscapesScope {
                prerequisite: k.relation.clone(),
                dependent: k.other_relation.clone(),
            }),
            k.both(S::Relations, D::EscapesScope),
        ),
        (
            core::Error::Relations(r::RelationRecordError::DuplicateDependency {
                prerequisite: k.relation.clone(),
                dependent: k.other_relation.clone(),
                kind: r::DependencyKind {
                    namespace: 2,
                    code: 0,
                },
            }),
            k.both(S::Relations, D::DuplicateDependency),
        ),
        (
            core::Error::Relations(r::RelationRecordError::Cycle {
                components: vec![r::CycleComponent {
                    members: vec![k.relation.clone()],
                    internal_edges: Vec::new(),
                }],
            }),
            k.refused(S::Relations, D::Cycle),
        ),
        (
            core::Error::Relations(r::RelationRecordError::CensusMismatch {
                missing: vec![k.relation.clone()],
                unexpected: vec![k.other_relation.clone()],
            }),
            k.both(S::Relations, D::CensusMismatch),
        ),
    ]
}

fn sources(k: &Keys) -> Rows {
    vec![
        (
            core::Error::Sources(Box::new(s::SourceError::DuplicateOperand(
                k.row.operand.clone(),
            ))),
            k.refused(S::Sources, D::DuplicateOperand),
        ),
        (
            core::Error::Sources(Box::new(s::SourceError::MissingOperand(
                k.row.operand.clone(),
            ))),
            k.refused(S::Sources, D::MissingOperand),
        ),
        (
            core::Error::Sources(Box::new(s::SourceError::ForeignOperand(
                k.row.operand.clone(),
            ))),
            k.refused(S::Sources, D::ForeignOperand),
        ),
        (
            core::Error::Sources(Box::new(s::SourceError::DuplicateRow(
                k.row.operand.clone(),
            ))),
            k.refused(S::Sources, D::DuplicateRow),
        ),
        (
            core::Error::Sources(Box::new(s::SourceError::UndecidableRegion(
                k.relation.operation,
            ))),
            E::UndecidableRegion(k.id.operation),
        ),
    ]
}

fn joins(k: &Keys) -> Rows {
    vec![
        (
            core::Error::StageJoin(core::StageJoinError::RepresentationChoices),
            Keys::empty(S::Join, D::RepresentationChoices),
        ),
        (
            core::Error::StageJoin(core::StageJoinError::LifecycleRequirements),
            Keys::empty(S::Join, D::LifecycleRequirements),
        ),
        (
            core::Error::StageJoin(core::StageJoinError::Constructibility(
                k.alternative.clone(),
            )),
            k.refused(S::Join, D::Constructibility),
        ),
        (
            core::Error::StageJoin(core::StageJoinError::SourceRows(k.alternative.clone())),
            k.refused(S::Join, D::SourceRows),
        ),
        (
            core::Error::StageJoin(core::StageJoinError::DuplicateSourceTable(
                k.alternative.clone(),
            )),
            k.refused(S::Join, D::DuplicateSourceTable),
        ),
        (
            core::Error::StageJoin(core::StageJoinError::UnnamedSourceTable(
                k.alternative.clone(),
            )),
            k.refused(S::Join, D::UnnamedSourceTable),
        ),
        (
            core::Error::ProofSourceMismatch(k.alternative.clone()),
            k.refused(S::Join, D::ProofSource),
        ),
        (
            core::Error::StageCensus {
                stage: core::Stage::Source,
                missing: vec![k.relation.clone()],
                unexpected: vec![k.other_relation.clone()],
            },
            k.both(S::Sources, D::CensusMismatch),
        ),
        (
            core::Error::StageCensus {
                stage: core::Stage::Proof,
                missing: vec![k.relation.clone()],
                unexpected: vec![k.other_relation.clone()],
            },
            k.both(S::Proof, D::CensusMismatch),
        ),
        (
            core::Error::StageCensus {
                stage: core::Stage::Requirement,
                missing: vec![k.relation.clone()],
                unexpected: vec![k.other_relation.clone()],
            },
            k.both(S::Requirements, D::CensusMismatch),
        ),
        (
            core::Error::StageCensus {
                stage: core::Stage::Case,
                missing: vec![k.relation.clone()],
                unexpected: vec![k.other_relation.clone()],
            },
            k.both(S::Cases, D::CensusMismatch),
        ),
    ]
}

fn proofs(k: &Keys) -> Rows {
    vec![
        (
            core::Error::Proof(proof::Error::NoFeasiblePlan(vec![k.relation.clone()])),
            E::NoFeasiblePlan(vec![k.id]),
        ),
        (
            core::Error::Proof(proof::Error::FixedSource(k.relation.clone())),
            E::FixedSource(k.id),
        ),
        (
            core::Error::Proof(proof::Error::StateLimit(7)),
            E::SearchLimit {
                stage: S::Proof,
                candidates: false,
                limit: 7,
            },
        ),
        (
            core::Error::Proof(proof::Error::CandidateLimit(7)),
            E::SearchLimit {
                stage: S::Proof,
                candidates: true,
                limit: 7,
            },
        ),
    ]
}

fn predicates(k: &Keys) -> Rows {
    use crate::engine_foundation as f;

    vec![
        (
            core::Error::Predicates(f::Error::UnknownExpression {
                relation: k.relation.clone(),
                expression: f::ExpressionKey::Relation {
                    relation: k.relation.clone(),
                    role: 0,
                },
            }),
            k.refused(S::Predicates, D::UnknownExpression),
        ),
        (
            core::Error::Predicates(f::Error::NonBoolean {
                relation: k.relation.clone(),
                expression: f::ExpressionKey::Relation {
                    relation: k.relation.clone(),
                    role: 0,
                },
            }),
            k.refused(S::Predicates, D::NonBoolean),
        ),
        (
            core::Error::Predicates(f::Error::OutsideScope {
                relation: k.relation.clone(),
                expression: f::ExpressionKey::Relation {
                    relation: k.relation.clone(),
                    role: 0,
                },
            }),
            k.refused(S::Predicates, D::OutsideScope),
        ),
    ]
}

fn construction(k: &Keys) -> Rows {
    use crate::engine_constructibility as c;
    let node = c::NodeKey::operation(k.relation.operation);
    vec![
        (
            core::Error::Constructibility(Box::new(
                c::RecordError::DuplicateConstructibilityNode { node: node.clone() },
            )),
            k.operation(S::Constructibility, D::DuplicateConstructibilityNode),
        ),
        (
            core::Error::Constructibility(Box::new(
                c::RecordError::DuplicateConstructibilityDependency { node: node.clone() },
            )),
            k.operation(S::Constructibility, D::DuplicateConstructibilityDependency),
        ),
        (
            core::Error::Constructibility(Box::new(c::RecordError::UnknownConstructibilityNode {
                node: node.clone(),
            })),
            k.operation(S::Constructibility, D::UnknownConstructibilityNode),
        ),
        (
            core::Error::Constructibility(Box::new(c::RecordError::ConstructibilityCycle {
                components: vec![vec![node.clone()]],
            })),
            k.operation(S::Constructibility, D::ConstructibilityCycle),
        ),
        (
            core::Error::Constructibility(Box::new(
                c::RecordError::ConstructibilityCrossOperationDependency {
                    operation: k.relation.operation,
                    node: node.clone(),
                },
            )),
            k.operation(
                S::Constructibility,
                D::ConstructibilityCrossOperationDependency,
            ),
        ),
        (
            core::Error::Constructibility(Box::new(
                c::RecordError::PermissionlessPrivateDependency {
                    operation: k.relation.operation,
                    node: node.clone(),
                    path: vec![node.clone()],
                },
            )),
            k.operation(S::Constructibility, D::PermissionlessPrivateDependency),
        ),
        (
            core::Error::Constructibility(Box::new(
                c::RecordError::ConstructibilityWitnessUnavailable {
                    operation: k.relation.operation,
                    authorization: k.authorization.clone(),
                    node: node.clone(),
                    path: vec![node.clone()],
                },
            )),
            k.operation(S::Constructibility, D::ConstructibilityWitnessUnavailable),
        ),
        (
            core::Error::Constructibility(Box::new(c::RecordError::SponsorDependencyEscaped {
                operation: k.relation.operation,
                node: node.clone(),
                path: vec![node],
            })),
            k.operation(S::Constructibility, D::SponsorDependencyEscaped),
        ),
    ]
}

fn lifecycle(k: &Keys) -> Rows {
    use crate::engine_lifecycle as l;
    let object = crate::engine_lifecycle::ObjectKey {
        namespace: 2,
        code: 0,
    };
    vec![
        (
            core::Error::Lifecycle(l::LifecycleRecordError::MissingRepresentationChoice {
                operation: k.relation.operation,
                object,
            }),
            k.operation(S::Lifecycle, D::MissingRepresentationChoice),
        ),
        (
            core::Error::Lifecycle(l::LifecycleRecordError::MissingLifecyclePath {
                object,
                representation: l::ModeKey {
                    namespace: 2,
                    code: 0,
                },
                exit: k.relation.operation,
            }),
            k.operation(S::Lifecycle, D::MissingLifecyclePath),
        ),
    ]
}

fn requirements(k: &Keys) -> Rows {
    use crate::engine_requirement as q;

    vec![
        (
            core::Error::Requirements(q::SelectionError::Missing(k.relation.clone())),
            k.refused(S::Requirements, D::MissingSelection),
        ),
        (
            core::Error::Requirements(q::SelectionError::Unexpected(k.relation.clone())),
            k.refused(S::Requirements, D::UnexpectedSelection),
        ),
        (
            core::Error::Requirements(q::SelectionError::Unapproved(k.relation.clone())),
            k.refused(S::Requirements, D::UnapprovedSelection),
        ),
        (
            core::Error::Requirements(q::SelectionError::Duplicate(k.relation.clone())),
            k.refused(S::Requirements, D::DuplicateSelection),
        ),
        (
            core::Error::Requirements(q::SelectionError::OwnerRefusal(Box::new(
                crate::CompileError::MissingOwnerIdentity,
            ))),
            E::Compiler(Box::new(crate::CompileError::MissingOwnerIdentity)),
        ),
        (
            core::Error::Aggregate(Box::new(q::Error::Capability {
                relation: Some(k.relation.clone()),
                capability: crate::capability::RequiredCapability::PublicConstructibility,
            })),
            k.refused(S::Requirements, D::CapabilityMismatch),
        ),
        (
            core::Error::Aggregate(Box::new(q::Error::Source {
                relation: Some(k.relation.clone()),
                row: Box::new(k.row.clone()),
            })),
            k.refused(S::Requirements, D::SourceMismatch),
        ),
        (
            core::Error::Aggregate(Box::new(q::Error::Evidence {
                relation: Some(k.relation.clone()),
                evidence: k.evidence.clone(),
            })),
            k.refused(S::Requirements, D::EvidenceMismatch),
        ),
    ]
}

fn cases(k: &Keys) -> Rows {
    use crate::engine_cases as c;
    let choice = proof::ChoiceKey {
        operation: k.relation.operation,
        object: crate::engine_lifecycle::ObjectKey {
            namespace: 2,
            code: 0,
        },
    };
    vec![
        (
            core::Error::Cases(c::Error::MissingRepresentationChoice(choice)),
            k.operation(S::Cases, D::MissingRepresentationChoice),
        ),
        (
            core::Error::Cases(c::Error::UnsupportedRepresentation {
                choice,
                mode: crate::engine_lifecycle::ModeKey {
                    namespace: 2,
                    code: 0,
                },
            }),
            k.operation(S::Cases, D::UnsupportedRepresentation),
        ),
        (
            core::Error::Cases(c::Error::CaseCensusMismatch {
                missing: vec![k.case.clone()],
                unexpected: vec![k.other_case.clone()],
            }),
            k.both(S::Cases, D::CensusMismatch),
        ),
        (
            core::Error::Cases(c::Error::DuplicateCase(k.case.clone())),
            k.refused(S::Cases, D::DuplicateCase),
        ),
        (
            core::Error::Cases(c::Error::CaseFactsMismatch(k.case.clone())),
            k.refused(S::Cases, D::CaseFactsMismatch),
        ),
        (
            core::Error::Cases(c::Error::OwnerRefusal(Box::new(
                crate::CompileError::MissingOwnerIdentity,
            ))),
            E::Compiler(Box::new(crate::CompileError::MissingOwnerIdentity)),
        ),
    ]
}

fn placement(k: &Keys) -> Rows {
    let wrap = |error| {
        core::Error::Operations(Box::new(crate::engine_operation::Error::Placement(
            Box::new(error),
        )))
    };
    vec![
        (
            wrap(p::Error::NoEligibleCarrier(k.case.clone())),
            k.refused(S::Placement, D::NoEligibleCarrier),
        ),
        (
            wrap(p::Error::GlobalRelationHasOnlyLocalCarrier(k.case.clone())),
            k.refused(S::Placement, D::GlobalRelationHasOnlyLocalCarrier),
        ),
        (
            wrap(p::Error::UnconditionalRelationOnOptionalCarrier(
                k.case.clone(),
            )),
            k.refused(S::Placement, D::UnconditionalRelationOnOptionalCarrier),
        ),
        (
            wrap(p::Error::NonCanonicalCarrierAssignment(k.case.clone())),
            k.refused(S::Placement, D::NonCanonicalCarrierAssignment),
        ),
        (
            wrap(p::Error::MissingLayoutRequirement(k.case.clone())),
            k.refused(S::Placement, D::MissingLayoutRequirement),
        ),
        (
            wrap(p::Error::DuplicatePlacedCarrier {
                key: k.case.clone(),
                carrier: k.carrier.clone(),
            }),
            k.refused(S::Placement, D::DuplicatePlacedCarrier),
        ),
        (
            wrap(p::Error::UnpermittedCarrierPlacement {
                key: k.case.clone(),
                carrier: k.carrier.clone(),
            }),
            k.refused(S::Placement, D::UnpermittedCarrierPlacement),
        ),
        (
            wrap(p::Error::PlacementCensusMismatch {
                missing: vec![k.case.clone()],
                unexpected: vec![k.other_case.clone()],
            }),
            k.both(S::Placement, D::CensusMismatch),
        ),
        (
            wrap(p::Error::UnexpectedLayoutRequirement(vec![
                k.layout.clone(),
            ])),
            E::Refused {
                stage: S::Placement,
                defect: D::UnexpectedLayoutRequirement,
                relations: Vec::new(),
                operations: Vec::new(),
                unrestored_key_count: 1,
            },
        ),
        (
            wrap(p::Error::StateLimit(7)),
            E::SearchLimit {
                stage: S::Placement,
                candidates: false,
                limit: 7,
            },
        ),
        (
            wrap(p::Error::CandidateLimit(7)),
            E::SearchLimit {
                stage: S::Placement,
                candidates: true,
                limit: 7,
            },
        ),
    ]
}

fn coverage(k: &Keys) -> Rows {
    use crate::engine_coverage as c;
    let wrap = |error| {
        core::Error::Operations(Box::new(crate::engine_operation::Error::Coverage(
            Box::new(error),
        )))
    };
    vec![
        (
            wrap(c::Error::MissingInactive(k.case.clone())),
            k.refused(S::Coverage, D::MissingInactive),
        ),
        (
            wrap(c::Error::UnexpectedRejection(k.case.clone())),
            k.refused(S::Coverage, D::UnexpectedRejection),
        ),
        (
            wrap(c::Error::MissingPositive {
                key: k.case.clone(),
                boundary: crate::coverage::CoverageBoundary::ExternalEvidence,
            }),
            k.refused(S::Coverage, D::MissingPositive),
        ),
        (
            wrap(c::Error::MissingNegative {
                key: k.case.clone(),
                boundary: crate::coverage::CoverageBoundary::ExternalEvidence,
            }),
            k.refused(S::Coverage, D::MissingNegative),
        ),
        (
            wrap(c::Error::CensusMismatch {
                missing: vec![k.case.clone()],
                unexpected: vec![k.other_case.clone()],
            }),
            k.both(S::Coverage, D::CensusMismatch),
        ),
        (
            wrap(c::Error::DuplicateRequirement(c::Requirement {
                key: k.case.clone(),
                boundary: crate::coverage::CoverageBoundary::ExternalEvidence,
                purpose: c::Purpose::AcceptedProjection,
                inactive: false,
            })),
            k.refused(S::Coverage, D::DuplicateRequirement),
        ),
        (
            core::Error::Operations(Box::new(crate::engine_operation::Error::Join(
                crate::engine_coverage_join::Error::UnexpectedCarrier(k.case.clone()),
            ))),
            k.refused(S::Coverage, D::UnexpectedCarrier),
        ),
        (
            core::Error::Operations(Box::new(crate::engine_operation::Error::Join(
                crate::engine_coverage_join::Error::MissingCarrier(k.case.clone()),
            ))),
            k.refused(S::Coverage, D::MissingCarrier),
        ),
        (
            core::Error::Operations(Box::new(crate::engine_operation::Error::Join(
                crate::engine_coverage_join::Error::MissingProjection(k.case.clone()),
            ))),
            k.refused(S::Coverage, D::MissingProjection),
        ),
        (
            core::Error::Operations(Box::new(crate::engine_operation::Error::Join(
                crate::engine_coverage_join::Error::UnexpectedLayout(k.case.clone()),
            ))),
            k.refused(S::Coverage, D::UnexpectedLayout),
        ),
        (
            core::Error::Operations(Box::new(crate::engine_operation::Error::Join(
                crate::engine_coverage_join::Error::MissingLayout(k.case.clone()),
            ))),
            k.refused(S::Coverage, D::MissingLayout),
        ),
    ]
}

fn graph(k: &Keys) -> Rows {
    use crate::engine_coverage_graph as g;
    let symbol = g::Key::Relation(k.case.clone());
    let edge = g::Dependency {
        source: symbol.clone(),
        target: g::Key::Relation(k.other_case.clone()),
        edge: crate::coverage_graph::CoverageEdge::RelationPrerequisite,
    };
    let wrap =
        |error| core::Error::Operations(Box::new(crate::engine_operation::Error::Graph(error)));
    let expected = |defect| E::CoverageGraph {
        defect,
        symbols: vec![crate::keyed_plan::CoverageSymbol::Relation(k.id)],
        unrestored_key_count: 0,
    };
    let both = |defect| E::CoverageGraph {
        defect,
        symbols: std::collections::BTreeSet::from([
            crate::keyed_plan::CoverageSymbol::Relation(k.id),
            crate::keyed_plan::CoverageSymbol::Relation(k.other_id),
        ])
        .into_iter()
        .collect(),
        unrestored_key_count: 0,
    };
    vec![
        (
            wrap(g::Error::UnknownSymbol(Box::new(symbol.clone()))),
            expected(D::UnknownSymbol),
        ),
        (
            wrap(g::Error::DuplicateSymbol(Box::new(symbol.clone()))),
            expected(D::DuplicateSymbol),
        ),
        (
            wrap(g::Error::EndpointClass(Box::new(edge.clone()))),
            both(D::EndpointClass),
        ),
        (
            wrap(g::Error::CrossOperation(Box::new(edge.clone()))),
            both(D::CrossOperation),
        ),
        (
            wrap(g::Error::DuplicateDependency(Box::new(edge.clone()))),
            both(D::DuplicateDependency),
        ),
        (
            wrap(g::Error::Cycle(vec![g::Cycle {
                members: vec![symbol],
                internal_edges: vec![edge],
            }])),
            expected(D::Cycle),
        ),
    ]
}

fn wrappers(k: &Keys) -> Rows {
    vec![
        (
            core::Error::Operations(Box::new(crate::engine_operation::Error::Prerequisite(
                Box::new((k.case.clone(), k.other_case.clone())),
            ))),
            k.both(S::Coverage, D::Prerequisite),
        ),
        (
            core::Error::Operations(Box::new(crate::engine_operation::Error::CaseFacts {
                key: k.case.clone(),
                defect: crate::analyzed_operation::RelationCaseDefect::ForeignOperation,
            })),
            k.refused(S::Cases, D::CaseFactsMismatch),
        ),
        (
            core::Error::Operations(Box::new(crate::engine_operation::Error::CaseCensus {
                missing: vec![k.case.clone()],
                unexpected: vec![k.other_case.clone()],
            })),
            k.both(S::Cases, D::CensusMismatch),
        ),
        (core::Error::SourceMismatch, E::PublicationMismatch),
        (core::Error::ProgramMismatch, E::PublicationMismatch),
        (
            core::Error::Lowering(Box::new(r::LoweringError::Attestation(
                crate::CompileError::MissingOwnerIdentity,
            ))),
            Keys::empty(S::Lowering, D::InvalidRecords),
        ),
    ]
}

fn lowering_rows(k: &Keys) -> Vec<(crate::engine_keyed::KeyedLoweringError, E)> {
    use crate::{
        engine_keyed::{KeyedLoweringError as L, ShellField},
        engine_requirement::ProofBundleError as B,
    };
    vec![
        (
            L::UnsupportedShell {
                operation: k.id.operation,
                field: ShellField::Expressions,
                count: 1,
            },
            E::UnsupportedShell {
                operation: k.id.operation,
                field: ShellField::Expressions,
                count: 1,
            },
        ),
        (
            L::MissingSourceTable(k.relation.clone()),
            k.refused(S::Lowering, D::MissingSourceTable),
        ),
        (
            L::RuntimeSourcePayload(k.relation.clone()),
            k.refused(S::Lowering, D::RuntimeSourcePayload),
        ),
        (
            L::ProofBundle(B::Constructibility(k.alternative.clone())),
            E::ProofBundle {
                relation: k.id,
                reason: None,
            },
        ),
        (
            L::ProofBundle(B::Source {
                alternative: k.alternative.clone(),
                reason: crate::proof::LocalProofRejection::SourceDerivation,
            }),
            E::ProofBundle {
                relation: k.id,
                reason: Some(crate::proof::LocalProofRejection::SourceDerivation),
            },
        ),
    ]
}

#[test]
fn partial_keyed_restoration_keeps_known_subjects_and_unknown_counts() {
    use crate::{engine_constructibility as c, engine_coverage_graph as g};
    let input = bound(&[K::Source]);
    let prepared = Prepared::new(&input).unwrap();
    let row = &prepared.records.cases.rows[0];
    let known = row.key.relation.clone();
    let id = prepared.relation(&known).unwrap();
    let mut unknown = known.clone();
    unknown.subject.push(u16::MAX);
    let unknown_operation = r::OperationKey {
        namespace: u16::MAX,
        code: u16::MAX,
    };
    let mut unknown_case = row.key.clone();
    unknown_case.relation = unknown.clone();
    let expected = |stage, defect, relations, operations| E::Refused {
        stage,
        defect,
        relations,
        operations,
        unrestored_key_count: 1,
    };
    let rows = vec![
        (
            core::Error::Relations(r::RelationRecordError::CensusMismatch {
                missing: vec![known.clone(), unknown.clone()],
                unexpected: vec![known.clone(), unknown.clone()],
            }),
            expected(
                S::Relations,
                D::CensusMismatch,
                vec![id],
                vec![id.operation],
            ),
        ),
        (
            core::Error::Proof(proof::Error::NoFeasiblePlan(vec![known, unknown.clone()])),
            expected(S::Proof, D::NoFeasiblePlan, vec![id], vec![id.operation]),
        ),
        (
            core::Error::Proof(proof::Error::FixedSource(unknown)),
            expected(S::Proof, D::FixedSource, Vec::new(), Vec::new()),
        ),
        (
            core::Error::Sources(Box::new(s::SourceError::UndecidableRegion(
                unknown_operation,
            ))),
            expected(S::Sources, D::UndecidableRegion, Vec::new(), Vec::new()),
        ),
        (
            core::Error::Constructibility(Box::new(
                c::RecordError::ConstructibilityCrossOperationDependency {
                    operation: row.key.relation.operation,
                    node: c::NodeKey::operation(unknown_operation),
                },
            )),
            expected(
                S::Constructibility,
                D::ConstructibilityCrossOperationDependency,
                Vec::new(),
                vec![id.operation],
            ),
        ),
        (
            core::Error::Operations(Box::new(crate::engine_operation::Error::Graph(
                g::Error::EndpointClass(Box::new(g::Dependency {
                    source: g::Key::Relation(row.key.clone()),
                    target: g::Key::Relation(unknown_case),
                    edge: crate::coverage_graph::CoverageEdge::RelationPrerequisite,
                })),
            ))),
            E::CoverageGraph {
                defect: D::EndpointClass,
                symbols: vec![crate::keyed_plan::CoverageSymbol::Relation(id)],
                unrestored_key_count: 1,
            },
        ),
    ];
    for (offered, expected) in rows {
        assert_eq!(prepared.refusal(offered), expected);
    }
}

#[test]
fn unknown_lowering_keys_keep_their_named_defect() {
    use crate::engine_keyed::KeyedLoweringError as L;
    use crate::engine_requirement::ProofBundleError as B;
    let input = bound(&[K::Source]);
    let prepared = Prepared::new(&input).unwrap();
    let mut key = prepared.records.sources.tables[0].relation.clone();
    assert!(matches!(
        refusal::lowering(input.owner(), L::MissingSourceTable(key.clone())),
        E::Refused {
            unrestored_key_count: 0,
            ..
        }
    ));
    key.subject.push(u16::MAX);
    let alternative = proof::AlternativeKey {
        relation: key.clone(),
        namespace: 2,
        kind: 0,
    };
    for (error, defect) in [
        (L::MissingSourceTable(key.clone()), D::MissingSourceTable),
        (L::RuntimeSourcePayload(key), D::RuntimeSourcePayload),
        (
            L::ProofBundle(B::Constructibility(alternative.clone())),
            D::Constructibility,
        ),
        (
            L::ProofBundle(B::Source {
                alternative,
                reason: crate::proof::LocalProofRejection::SourceDerivation,
            }),
            D::SourceRows,
        ),
    ] {
        assert_eq!(
            refusal::lowering(input.owner(), error),
            E::Refused {
                stage: S::Lowering,
                defect,
                relations: Vec::new(),
                operations: Vec::new(),
                unrestored_key_count: 1,
            }
        );
    }
}
