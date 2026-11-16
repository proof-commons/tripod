//! External consumer of the default keyed production entry.
use architecture::keyed_fixed::{KeyedFixedManifest, KeyedFixedOperationId as K};
use compiler::{AnalysisPolicy, ProofSearchLimits, bind_input, keyed_fixed_plan::*};

#[test]
fn external_keyed_fixed_plans_publish_all_seven_stages_and_universal_clauses() {
    for (scope, relations, evidence, assignments, coverage_nodes) in [
        (vec![K::Source], 4, 4, 0, 28),
        (vec![K::Merge], 6, 3, 3, 39),
        (vec![K::Source, K::Merge], 10, 7, 3, 67),
    ] {
        let manifest = KeyedFixedManifest::contract().expect("manifest");
        let realization = realization::keyed_fixed::KeyedFixedRealizationDraft::derive(&manifest)
            .expect("realization")
            .validate(&manifest)
            .expect("validation");
        let declarations = realization.project();
        let states = std::num::NonZeroU64::new(1_000_000).unwrap();
        let candidates = std::num::NonZeroU64::new(100_000).unwrap();
        let requested_scope = scope.clone();
        let input = bind_input(
            &manifest,
            realization,
            scope,
            AnalysisPolicy::strict(ProofSearchLimits::new(states, candidates)),
        )
        .expect("input");
        let limits = PlacementSearchLimits::new(states, candidates);
        let plan = plan(&input, limits).expect("production plan");
        revalidate(&input, limits, &plan).expect("independent validation");
        let projection = plan.project();
        assert_eq!(projection, &expected_projection(&requested_scope, limits));
        assert_eq!(projection.stages.relations, relations);
        assert_eq!(projection.stages.source_tables, relations);
        assert_eq!(projection.stages.construction_nodes, 0);
        assert_eq!(projection.stages.lifecycle_nodes, 0);
        assert_eq!(projection.stages.proof_plans, 1);
        assert_eq!(projection.stages.placement_assignments, assignments);
        assert_eq!(projection.stages.coverage_nodes, coverage_nodes);
        assert_eq!(projection.evidence.len(), evidence);
        let published = projection
            .plans
            .iter()
            .flat_map(|plan| plan.operations.values())
            .flat_map(|operation| &operation.relations)
            .map(|(id, row)| (*id, row.declaration))
            .collect::<std::collections::BTreeMap<_, _>>();
        let expected = declarations
            .into_iter()
            .filter(|row| projection.scope.contains(&row.id.operation))
            .map(|row| (row.id, row.relation))
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(published, expected);
        inspect_every_relation(projection);
        // This public construction above takes only public declarations, scope and limits:
        // no creator secret or evidence witness is supplied. Premises remain obligations.
        let runtime = projection
            .plans
            .iter()
            .flat_map(|p| p.operations.values())
            .flat_map(|o| &o.relations)
            .filter(|(_, row)| !row.runtime.is_empty())
            .collect::<Vec<_>>();
        assert_eq!(runtime.len(), assignments);
        for (id, row) in runtime {
            assert!(matches!(
                id.kind,
                realization::keyed_fixed_relation::KeyedFixedRelationKind::BornFirst
                    | realization::keyed_fixed_relation::KeyedFixedRelationKind::OwnQualification
                    | realization::keyed_fixed_relation::KeyedFixedRelationKind::SuccessorQualification
            ));
            assert_eq!(
                row.runtime,
                vec![RuntimeClause {
                    side: realization::TransactionSide::Input,
                    multiplicity: CarrierMultiplicity::EveryMember
                }]
            );
        }
        for obligation in &projection.evidence {
            if let EvidencePremises::MembershipReading {
                inclusion,
                retrieval,
                finality_for_final_reading,
                ..
            } = obligation.premises
            {
                assert!(inclusion && retrieval && finality_for_final_reading);
            }
        }
        for carrier in projection
            .plans
            .iter()
            .flat_map(|p| p.operations.values())
            .flat_map(|o| &o.placements)
            .flat_map(|p| p.values())
            .flatten()
        {
            assert_eq!(carrier.operation, K::Merge);
            assert_eq!(carrier.quantification, CarrierQuantification::PerMember);
        }
    }
}

// Independent paper-derived publication: no declaration constructor, lowering,
// restoration, shared analyzer or producer projection supplies expected rows.
#[allow(clippy::too_many_lines)] // Keep the complete independent publication expectation together.
fn expected_projection(scope: &[K], limits: PlacementSearchLimits) -> KeyedFixedPlanProjection {
    use architecture::keyed_fixed::{KeyedFixedArchitectureBinding, KeyedFixedObjectId as Obj};
    use realization::TransactionSide::{Input, Output};
    use realization::keyed_fixed_relation::{
        KeyedFixedCardinality as Count, KeyedFixedRelation as R, KeyedFixedRelationId as Id,
        KeyedFixedRelationKind as Kind, RecordRecurrence as Rec,
    };
    use std::collections::{BTreeMap, BTreeSet};
    let mut operations = BTreeMap::new();
    let mut requirements = BTreeMap::new();
    let mut selected_relations = BTreeSet::new();
    let mut all_evidence = BTreeSet::new();
    for operation in scope {
        let rows = if *operation == K::Source {
            vec![
                (
                    Kind::InputFamily,
                    R::HandleFamily {
                        side: Input,
                        cardinality: Count::Zero,
                    },
                    Some(EvidencePremises::HandleClassification {
                        side: Input,
                        minimum: 0,
                        maximum: Some(0),
                    }),
                ),
                (
                    Kind::OutputFamily,
                    R::HandleFamily {
                        side: Output,
                        cardinality: Count::One,
                    },
                    Some(EvidencePremises::HandleClassification {
                        side: Output,
                        minimum: 1,
                        maximum: Some(1),
                    }),
                ),
                (
                    Kind::SourceClassification,
                    R::SourceClassification,
                    Some(EvidencePremises::SourceClassification),
                ),
                (
                    Kind::MembershipReading,
                    R::MembershipReading {
                        recurrence: Rec::SourceSingleton,
                    },
                    Some(EvidencePremises::MembershipReading {
                        recurrence: Rec::SourceSingleton,
                        inclusion: true,
                        retrieval: true,
                        finality_for_final_reading: true,
                    }),
                ),
            ]
        } else {
            vec![
                (
                    Kind::InputFamily,
                    R::HandleFamily {
                        side: Input,
                        cardinality: Count::OneOrMore,
                    },
                    Some(EvidencePremises::HandleClassification {
                        side: Input,
                        minimum: 1,
                        maximum: None,
                    }),
                ),
                (
                    Kind::OutputFamily,
                    R::HandleFamily {
                        side: Output,
                        cardinality: Count::One,
                    },
                    Some(EvidencePremises::HandleClassification {
                        side: Output,
                        minimum: 1,
                        maximum: Some(1),
                    }),
                ),
                (Kind::BornFirst, R::BornFirst, None),
                (Kind::OwnQualification, R::OwnQualification, None),
                (
                    Kind::SuccessorQualification,
                    R::SuccessorQualification,
                    None,
                ),
                (
                    Kind::MembershipReading,
                    R::MembershipReading {
                        recurrence: Rec::ConsumedUnion,
                    },
                    Some(EvidencePremises::MembershipReading {
                        recurrence: Rec::ConsumedUnion,
                        inclusion: true,
                        retrieval: true,
                        finality_for_final_reading: true,
                    }),
                ),
            ]
        };
        let mut relations = BTreeMap::new();
        let mut assignments = BTreeMap::new();
        let mut nodes = BTreeMap::new();
        let mut edges = Vec::new();
        for (kind, declaration, premises) in rows {
            let id = Id {
                operation: *operation,
                object: Obj::Handle,
                kind,
            };
            let runtime = premises.is_none();
            let boundary = if runtime {
                CoverageBoundary::RuntimeCarrier
            } else {
                CoverageBoundary::ExternalEvidence
            };
            let requirement = |purpose| CoverageRequirement {
                relation: id,
                boundary,
                purpose,
                inactive: false,
            };
            let carrier = RuntimeCarrier {
                operation: *operation,
                object: Obj::Handle,
                quantification: CarrierQuantification::PerMember,
            };
            let evidence = premises.map(|premises| EvidenceObligation {
                relation: id,
                requirement: declaration,
                premises,
            });
            let mut positive = vec![requirement(CoveragePurpose::AcceptedProjection)];
            let negative = if let Some(evidence) = &evidence {
                positive.push(requirement(CoveragePurpose::Verified(evidence.clone())));
                [
                    EvidenceDefect::Missing,
                    EvidenceDefect::Failed,
                    EvidenceDefect::IdentityMismatch,
                ]
                .map(|defect| {
                    requirement(CoveragePurpose::Rejected {
                        evidence: evidence.clone(),
                        defect,
                    })
                })
                .to_vec()
            } else {
                positive.push(requirement(CoveragePurpose::ClauseAccepted));
                positive.push(requirement(CoveragePurpose::CarrierExecution(
                    carrier.clone(),
                )));
                selected_relations.insert(id);
                assignments.insert(id, vec![carrier.clone()]);
                vec![requirement(CoveragePurpose::ClauseRejected)]
            };
            positive.sort();
            let evidence = evidence.into_iter().collect::<BTreeSet<_>>();
            all_evidence.extend(evidence.clone());
            requirements.insert(id, evidence.clone());
            let mut node = |symbol: CoverageSymbol, class, relation, active| {
                nodes.insert(
                    symbol.clone(),
                    CoverageNode {
                        id: symbol,
                        operation: *operation,
                        class,
                        relation,
                        active,
                    },
                );
            };
            node(
                CoverageSymbol::Relation(id),
                CoverageNodeClass::RelationCase,
                Some(id),
                Some(true),
            );
            for req in positive.iter().chain(&negative) {
                node(
                    CoverageSymbol::Requirement(Box::new(req.clone())),
                    CoverageNodeClass::Requirement,
                    None,
                    None,
                );
            }
            let mut edge = |source, edge| {
                edges.push(CoverageDependency {
                    source,
                    target: CoverageSymbol::Relation(id),
                    edge,
                });
            };
            edge(
                CoverageSymbol::Requirement(Box::new(requirement(
                    CoveragePurpose::AcceptedProjection,
                ))),
                CoverageEdge::ProjectionEnablesRelation,
            );
            for evidence in &evidence {
                node(
                    CoverageSymbol::Evidence(evidence.clone()),
                    CoverageNodeClass::ExternalEvidence,
                    None,
                    None,
                );
                edge(
                    CoverageSymbol::Evidence(evidence.clone()),
                    CoverageEdge::EvidenceEnablesRelation,
                );
            }
            if runtime {
                let symbol = CoverageSymbol::Carrier {
                    relation: id,
                    carrier: carrier.clone(),
                };
                node(symbol.clone(), CoverageNodeClass::Carrier, None, None);
                edge(symbol, CoverageEdge::CarrierEnablesRelation);
            }
            relations.insert(
                id,
                RelationPlan {
                    declaration,
                    active: true,
                    runtime: if runtime {
                        vec![RuntimeClause {
                            side: Input,
                            multiplicity: CarrierMultiplicity::EveryMember,
                        }]
                    } else {
                        Vec::new()
                    },
                    boundaries: BTreeSet::from([boundary]),
                    positive,
                    negative,
                    evidence,
                    alternatives: if runtime {
                        BTreeSet::from([BTreeSet::from([carrier])])
                    } else {
                        BTreeSet::new()
                    },
                    projections: BTreeSet::from([boundary]),
                },
            );
        }
        edges.sort_by(|a, b| (&a.source, &a.target, a.edge).cmp(&(&b.source, &b.target, b.edge)));
        operations.insert(
            *operation,
            OperationPlan {
                relations,
                placements: vec![assignments],
                coverage_nodes: nodes.into_values().collect(),
                coverage_edges: edges,
                search: PlacementSearchReport {
                    states_visited: if *operation == K::Source { 1 } else { 4 },
                    complete_assignments: 1,
                    feasible_placements: 1,
                    retained_options: if *operation == K::Source { 0 } else { 3 },
                },
            },
        );
    }
    let merge = scope.contains(&K::Merge);
    let source = scope.contains(&K::Source);
    KeyedFixedPlanProjection {
        binding: KeyedFixedArchitectureBinding {
            profile: architecture::semantic::SemanticProfileId::KeyedFixedForm,
            schema_version: 3,
            digest: [
                168, 83, 97, 146, 4, 54, 228, 90, 227, 38, 214, 131, 218, 63, 196, 87, 34, 102,
                156, 162, 181, 242, 65, 139, 201, 120, 241, 131, 243, 66, 47, 193,
            ],
        },
        scope: scope.to_vec(),
        plans: vec![ProofPlan {
            selected_relations,
            requirements,
            operations,
        }],
        evidence: all_evidence,
        stages: StageCensus {
            relations: usize::from(source) * 4 + usize::from(merge) * 6,
            source_tables: usize::from(source) * 4 + usize::from(merge) * 6,
            construction_nodes: 0,
            lifecycle_nodes: 0,
            proof_plans: 1,
            placement_assignments: usize::from(merge) * 3,
            coverage_nodes: usize::from(source) * 28 + usize::from(merge) * 39,
        },
        proof_limits: ProofSearchLimits::new(
            std::num::NonZeroU64::new(1_000_000).unwrap(),
            std::num::NonZeroU64::new(100_000).unwrap(),
        ),
        placement_limits: limits,
        proof_search: ProofSearchReport {
            states_visited: if merge { 4 } else { 1 },
            complete_assignments: 1,
            feasible_assignments: 1,
            ..ProofSearchReport::default()
        },
    }
}

#[test]
fn external_keyed_refusals_revalidate_scope_and_enforce_search_limits() {
    let bind = |op| {
        let manifest = KeyedFixedManifest::contract().unwrap();
        let source = realization::keyed_fixed::KeyedFixedRealizationDraft::derive(&manifest)
            .unwrap()
            .validate(&manifest)
            .unwrap();
        bind_input(
            &manifest,
            source,
            vec![op],
            AnalysisPolicy::strict(ProofSearchLimits::new(
                std::num::NonZeroU64::new(1_000_000).unwrap(),
                std::num::NonZeroU64::new(100_000).unwrap(),
            )),
        )
        .unwrap()
    };
    let source = bind(K::Source);
    let merge = bind(K::Merge);
    let limits = PlacementSearchLimits::new(
        std::num::NonZeroU64::new(1_000_000).unwrap(),
        std::num::NonZeroU64::new(100_000).unwrap(),
    );
    let control = plan(&source, limits).unwrap();
    assert!(revalidate(&source, limits, &control).is_ok());
    assert_eq!(
        revalidate(&merge, limits, &control),
        Err(KeyedFixedPlanError::PublicationMismatch)
    );
    assert!(plan(&merge, limits).is_ok());
    let one = std::num::NonZeroU64::new(1).unwrap();
    assert_eq!(
        plan(&merge, PlacementSearchLimits::new(one, one)).err(),
        Some(KeyedFixedPlanError::SearchLimit {
            stage: Stage::Placement,
            candidates: false,
            limit: 1
        })
    );
}

fn inspect_every_relation(projection: &KeyedFixedPlanProjection) {
    for operation in projection
        .plans
        .iter()
        .flat_map(|plan| plan.operations.values())
    {
        let mut evidence_rows = 0;
        let mut runtime_rows = 0;
        for (id, row) in &operation.relations {
            assert!(row.active);
            assert!(!row.positive.is_empty() && !row.negative.is_empty());
            if row.runtime.is_empty() {
                evidence_rows += 1;
                assert_eq!(row.evidence.len(), 1);
                assert!(row.alternatives.is_empty());
                assert!(row.boundaries.contains(&CoverageBoundary::ExternalEvidence));
            } else {
                runtime_rows += 1;
                assert_eq!(id.operation, K::Merge);
                assert_eq!(row.runtime.len(), 1);
                assert!(row.evidence.is_empty());
                assert!(row.boundaries.contains(&CoverageBoundary::RuntimeCarrier));
            }
        }
        let op = operation.relations.keys().next().unwrap().operation;
        assert_eq!(
            (evidence_rows, runtime_rows),
            if op == K::Source { (4, 0) } else { (3, 3) }
        );
    }
}
