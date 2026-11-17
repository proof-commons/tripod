//! Complete target-independent Clear declaration.
//!
//! The owner census implements (`[PLAN-tab:realization:burn-clear-graphs]`).

use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    Architecture, AssetId, BoundId, ObjectId, OpenFlowKind, OperationId, ProjectionId,
    ProjectionRule, RootId, RootUse,
};

use crate::{
    AvailabilityClass, CardinalityMaximum, ConstructibilityClass,
    ConstructibilityDependencyDeclaration, ConstructibilityEdge, ConstructibilityEdgeRole,
    ConstructibilityNode, ConstructibilityNodeId, Count, DisclosureDependencyDeclaration,
    DisclosureEdge, DisclosureNode, DisclosureNodeId, DisclosureReason, DisclosureSeed, ExprId,
    ExpressionDeclaration, ExpressionNode, FactId, InitialVisibility,
    LifecycleDependencyDeclaration, LifecycleEdge, LifecycleNode, LifecycleNodeId, ObservedSide,
    OperationRealization, ProofAlternativeId, ProofKind, RealizationError, Relation,
    RelationDeclaration, RelationDependencyDeclaration, RelationEdge, RelationId, RelationKind,
    RelationSubject, RepresentationMode, RequirementStrength, SemanticType, StateField,
    TransactionSide, WitnessRole,
};

/// Weld the inherited architecture before admitting the declaration.
///
/// # Errors
/// Returns the exact architecture mismatch when a Clear coordinate differs.
pub fn derive(architecture: &Architecture) -> Result<OperationRealization, RealizationError> {
    crate::validate::validate_clear_architecture(architecture)?;
    Ok(declaration())
}

/// Canonical owner census, also consumed by completeness validation.
pub fn declaration() -> OperationRealization {
    let relations = relations();
    let relation_dependencies = dependencies();
    let (constructibility_nodes, constructibility_edges) = construction();
    let (lifecycle_nodes, lifecycle_edges) = lifecycle();
    let (disclosure_nodes, disclosure_edges, disclosure_seeds) = disclosure();
    OperationRealization {
        operation: OperationId::Clear,
        expressions: expressions(),
        relations,
        relation_dependencies,
        constructibility_nodes,
        constructibility_edges,
        lifecycle_nodes,
        lifecycle_edges,
        disclosure_nodes,
        disclosure_edges,
        disclosure_seeds,
    }
}

fn id(kind: RelationKind, subject: RelationSubject) -> RelationId {
    RelationId::new(OperationId::Clear, kind, subject)
}

fn op(kind: RelationKind) -> RelationId {
    id(kind, RelationSubject::Operation)
}

fn family(kind: RelationKind, side: TransactionSide, object: ObjectId) -> RelationId {
    id(kind, RelationSubject::ObjectFamily { side, object })
}

fn authorization() -> RelationId {
    op(RelationKind::Authorization)
}

fn sponsor(kind: RelationKind) -> RelationId {
    id(kind, RelationSubject::Sponsor)
}

fn representation(object: ObjectId) -> RelationId {
    id(
        RelationKind::Representation,
        RelationSubject::Representation { object },
    )
}

fn exit(object: ObjectId, operation: OperationId) -> RelationId {
    id(
        RelationKind::Lifecycle,
        RelationSubject::LifecycleExit {
            object,
            exit: operation,
        },
    )
}

fn row(id: RelationId, relation: Relation, proof: Option<ProofKind>) -> RelationDeclaration {
    RelationDeclaration {
        proof_alternatives: proof
            .into_iter()
            .map(|kind| ProofAlternativeId::new(id.clone(), kind))
            .collect(),
        id,
        relation,
    }
}

fn families() -> [(TransactionSide, ObjectId, Count, CardinalityMaximum); 6] {
    use TransactionSide::{Input, Output};
    [
        (
            Input,
            ObjectId::State,
            Count::ONE,
            CardinalityMaximum::Exact(Count::ONE),
        ),
        (
            Output,
            ObjectId::State,
            Count::ONE,
            CardinalityMaximum::Exact(Count::ONE),
        ),
        (
            Input,
            ObjectId::Ash,
            Count::ONE,
            CardinalityMaximum::Bound(BoundId::AshBatchMax),
        ),
        (
            Output,
            ObjectId::Ash,
            Count::ZERO,
            CardinalityMaximum::Exact(Count::ONE),
        ),
        (
            Input,
            ObjectId::PlainLbtc,
            Count::ZERO,
            CardinalityMaximum::Bound(BoundId::FeeSponsorInputMax),
        ),
        (
            Output,
            ObjectId::PlainLbtc,
            Count::ZERO,
            CardinalityMaximum::Exact(Count::ONE),
        ),
    ]
}

fn expressions() -> Vec<ExpressionDeclaration> {
    let mut expressions = Vec::new();
    for field in StateField::ALL {
        for side in [TransactionSide::Input, TransactionSide::Output] {
            let fact = FactId::StateField {
                operation: OperationId::Clear,
                side,
                field: *field,
            };
            expressions.push(ExpressionDeclaration {
                id: ExprId::fact(fact.clone()),
                ty: fact.semantic_type(),
                node: ExpressionNode::Fact(fact),
            });
        }
        if *field != StateField::YL {
            let relation = id(
                RelationKind::ClearAssignment,
                RelationSubject::StateField { field: *field },
            );
            expressions.push(ExpressionDeclaration {
                id: ExprId::relation(relation, crate::ExpressionRole::Predicate),
                ty: SemanticType::Bool,
                node: crate::burn_clear_laws::copy_expression(*field),
            });
        }
    }
    expressions
}

#[allow(clippy::too_many_lines)] // The complete relation census belongs in one owner.
fn relations() -> Vec<RelationDeclaration> {
    let mut rows = crate::burn_clear_laws::burn_clear_laws(OperationId::Clear)
        .iter()
        .map(crate::burn_clear_laws::BurnClearLawDeclaration::relation_declaration)
        .collect::<Vec<_>>();
    for (side, object, minimum, maximum) in families() {
        let observed = match side {
            TransactionSide::Input => ObservedSide::Input,
            TransactionSide::Output => ObservedSide::Output,
        };
        rows.push(row(
            family(RelationKind::Cardinality, side, object),
            Relation::Cardinality {
                side: observed,
                object,
                minimum,
                maximum,
            },
            Some(ProofKind::ManifestShape),
        ));
        rows.push(row(
            family(RelationKind::Recognition, side, object),
            Relation::Recognition {
                side: observed,
                object,
                asset: if object == ObjectId::State {
                    AssetId::Pid
                } else if object == ObjectId::Ash {
                    AssetId::U
                } else {
                    AssetId::Lbtc
                },
            },
            Some(ProofKind::ManifestShape),
        ));
    }
    for side in [TransactionSide::Input, TransactionSide::Output] {
        rows.push(row(
            id(
                RelationKind::AllowedObjectFamilies,
                RelationSubject::TransactionSide { side },
            ),
            Relation::AllowedObjectFamilies {
                side: match side {
                    TransactionSide::Input => ObservedSide::Input,
                    TransactionSide::Output => ObservedSide::Output,
                },
                allowed: BTreeSet::from([ObjectId::State, ObjectId::Ash, ObjectId::PlainLbtc]),
            },
            Some(ProofKind::ManifestShape),
        ));
    }
    rows.extend([
        row(
            authorization(),
            Relation::PermissionlessAuthorization,
            Some(ProofKind::PublicConstructibility),
        ),
        row(
            sponsor(RelationKind::SponsorIsolation),
            Relation::SponsorIsolation,
            Some(ProofKind::ManifestShape),
        ),
        row(
            sponsor(RelationKind::SponsorEnvelopeMultiplicity),
            Relation::SponsorEnvelopeMultiplicity {
                maximum: Count::ONE,
            },
            Some(ProofKind::ManifestShape),
        ),
        row(
            id(
                RelationKind::SubstrateConservation,
                RelationSubject::Asset {
                    asset: AssetId::Lbtc,
                },
            ),
            Relation::SubstrateConservation {
                asset: AssetId::Lbtc,
            },
            Some(ProofKind::SubstrateConservation),
        ),
        row(
            op(RelationKind::OpenFlowPolicy),
            Relation::OpenFlowPolicy {
                allowed: BTreeSet::from([OpenFlowKind::FeeSponsor]),
            },
            Some(ProofKind::ManifestShape),
        ),
        row(
            op(RelationKind::RootPolicy),
            Relation::RootPolicy {
                expected: RootId::ALL
                    .iter()
                    .map(|root| {
                        (
                            *root,
                            if *root == RootId::State {
                                RootUse::Succession
                            } else {
                                RootUse::Forbidden
                            },
                        )
                    })
                    .collect(),
            },
            Some(ProofKind::ManifestShape),
        ),
        row(
            op(RelationKind::ProjectionPolicy),
            Relation::ProjectionPolicy {
                expected: ProjectionId::ALL
                    .iter()
                    .map(|projection| {
                        (
                            *projection,
                            match projection {
                                ProjectionId::TransitionCertificate | ProjectionId::ClearEvent => {
                                    ProjectionRule::Required
                                }
                                ProjectionId::BurnEvent | ProjectionId::DistributionResidue => {
                                    ProjectionRule::Forbidden
                                }
                            },
                        )
                    })
                    .collect::<BTreeMap<_, _>>(),
            },
            Some(ProofKind::ManifestShape),
        ),
        row(
            op(RelationKind::Constructibility),
            Relation::Constructibility {
                class: ConstructibilityClass::PublicPermissionless,
            },
            Some(ProofKind::PublicConstructibility),
        ),
        row(
            representation(ObjectId::Ash),
            Relation::Representation {
                object: ObjectId::Ash,
                allowed: BTreeSet::from([
                    RepresentationMode::Explicit,
                    RepresentationMode::PublicCommitted,
                ]),
            },
            None,
        ),
        row(
            representation(ObjectId::State),
            Relation::Representation {
                object: ObjectId::State,
                allowed: BTreeSet::from([
                    RepresentationMode::Explicit,
                    RepresentationMode::PublicCommitted,
                ]),
            },
            None,
        ),
    ]);
    for (object, exits) in exits() {
        for operation in exits {
            rows.push(row(
                exit(object, operation),
                Relation::LifecycleExit {
                    object,
                    exit: operation,
                },
                None,
            ));
        }
    }
    rows
}

fn exits() -> [(ObjectId, Vec<OperationId>); 2] {
    [
        (
            ObjectId::Ash,
            vec![OperationId::CompactAsh, OperationId::Clear],
        ),
        (
            ObjectId::State,
            vec![
                OperationId::AdmitDeposits,
                OperationId::Cycle,
                OperationId::Redeem,
                OperationId::ReceiptRelabel,
                OperationId::Clear,
                OperationId::AnnounceMaturity,
            ],
        ),
    ]
}

fn dependency(
    prerequisite: RelationId,
    dependent: RelationId,
    edge: RelationEdge,
) -> RelationDependencyDeclaration {
    RelationDependencyDeclaration {
        prerequisite,
        dependent,
        edge,
    }
}

#[allow(clippy::too_many_lines)] // The complete dependency census belongs in one owner.
fn dependencies() -> Vec<RelationDependencyDeclaration> {
    use RelationEdge as E;
    use RelationKind as K;
    let mut edges = crate::burn_clear_laws::clear_law_dependencies();
    for (side, object, _, _) in families() {
        let recognition = family(K::Recognition, side, object);
        let cardinality = family(K::Cardinality, side, object);
        edges.push(dependency(
            recognition.clone(),
            cardinality.clone(),
            E::RecognitionBeforeCardinality,
        ));
        if object == ObjectId::PlainLbtc {
            edges.push(dependency(
                recognition,
                sponsor(K::SponsorIsolation),
                E::StaticRequirement,
            ));
            continue;
        }
        if side == TransactionSide::Input {
            edges.push(dependency(
                recognition,
                op(K::ClearClamp),
                E::StaticRequirement,
            ));
            edges.push(dependency(
                cardinality,
                op(K::ClearClamp),
                E::StaticRequirement,
            ));
        } else {
            edges.push(dependency(
                recognition.clone(),
                op(K::ClearBinding),
                E::StaticRequirement,
            ));
            edges.push(dependency(
                cardinality,
                op(K::ClearBinding),
                E::StaticRequirement,
            ));
            edges.push(dependency(
                recognition,
                representation(object),
                E::StaticRequirement,
            ));
        }
    }
    edges.extend([
        dependency(
            authorization(),
            id(
                K::AllowedObjectFamilies,
                RelationSubject::TransactionSide {
                    side: TransactionSide::Input,
                },
            ),
            E::AuthorizationBeforeClosure,
        ),
        dependency(
            authorization(),
            op(K::Constructibility),
            E::AuthorizationBeforeConstructibility,
        ),
        dependency(
            op(K::OpenFlowPolicy),
            sponsor(K::SponsorIsolation),
            E::OpenFlowPolicyBeforeSponsor,
        ),
        dependency(
            op(K::OpenFlowPolicy),
            sponsor(K::SponsorEnvelopeMultiplicity),
            E::StaticRequirement,
        ),
        dependency(
            sponsor(K::SponsorEnvelopeMultiplicity),
            sponsor(K::SponsorIsolation),
            E::StaticRequirement,
        ),
        dependency(
            sponsor(K::SponsorIsolation),
            op(K::Constructibility),
            E::SponsorBeforeConstructibility,
        ),
        dependency(
            sponsor(K::SponsorIsolation),
            id(
                K::SubstrateConservation,
                RelationSubject::Asset {
                    asset: AssetId::Lbtc,
                },
            ),
            E::SponsorBeforeOperation,
        ),
        dependency(
            op(K::RootPolicy),
            op(K::Constructibility),
            E::RootPolicyBeforeOperation,
        ),
        dependency(
            op(K::ProjectionPolicy),
            op(K::Constructibility),
            E::ProjectionPolicyBeforeOperation,
        ),
    ]);
    for (object, exits) in exits() {
        edges.push(dependency(
            op(K::Constructibility),
            representation(object),
            E::StaticRequirement,
        ));
        for operation in exits {
            edges.push(dependency(
                representation(object),
                exit(object, operation),
                E::RepresentationBeforeLifecycle,
            ));
        }
    }
    edges
}

fn facts() -> Vec<(FactId, AvailabilityClass)> {
    let mut facts = Vec::new();
    for (side, object) in [
        (TransactionSide::Input, ObjectId::State),
        (TransactionSide::Output, ObjectId::State),
        (TransactionSide::Input, ObjectId::Ash),
        (TransactionSide::Output, ObjectId::Ash),
    ] {
        facts.push((
            FactId::FamilyCount {
                operation: OperationId::Clear,
                side,
                object,
            },
            AvailabilityClass::Public,
        ));
    }
    facts.push((
        FactId::FamilyAmount {
            operation: OperationId::Clear,
            side: TransactionSide::Input,
            object: ObjectId::Ash,
        },
        AvailabilityClass::Public,
    ));
    for bound in [BoundId::AshBatchMax, BoundId::FeeSponsorInputMax] {
        facts.push((FactId::BoundValue { bound }, AvailabilityClass::Public));
    }
    for field in StateField::ALL {
        facts.push((
            FactId::StateField {
                operation: OperationId::Clear,
                side: TransactionSide::Input,
                field: *field,
            },
            AvailabilityClass::Public,
        ));
    }
    facts
}

fn construction() -> (
    Vec<ConstructibilityNode>,
    Vec<ConstructibilityDependencyDeclaration>,
) {
    let operation = ConstructibilityNodeId::Operation(OperationId::Clear);
    let mut nodes = vec![ConstructibilityNode {
        id: operation.clone(),
    }];
    let mut edges = Vec::new();
    for (fact, availability) in facts() {
        let source = ConstructibilityNodeId::Fact {
            operation: OperationId::Clear,
            fact,
            availability,
        };
        nodes.push(ConstructibilityNode { id: source.clone() });
        edges.push(ConstructibilityDependencyDeclaration {
            source,
            target: operation.clone(),
            edge: ConstructibilityEdge {
                role: ConstructibilityEdgeRole::RequiredFact,
                strength: RequirementStrength::Required,
            },
        });
    }
    let source = ConstructibilityNodeId::Witness {
        operation: OperationId::Clear,
        role: WitnessRole::SponsorAuthorization,
        availability: AvailabilityClass::SponsorLocal,
    };
    nodes.push(ConstructibilityNode { id: source.clone() });
    edges.push(ConstructibilityDependencyDeclaration {
        source,
        target: operation,
        edge: ConstructibilityEdge {
            role: ConstructibilityEdgeRole::SponsorOnly,
            strength: RequirementStrength::Optional,
        },
    });
    (nodes, edges)
}

fn lifecycle() -> (Vec<LifecycleNode>, Vec<LifecycleDependencyDeclaration>) {
    let mut nodes = BTreeSet::new();
    let mut edges = Vec::new();
    for (object, exits) in exits() {
        for mode in [
            RepresentationMode::Explicit,
            RepresentationMode::PublicCommitted,
        ] {
            let source = LifecycleNodeId::Representation { object, mode };
            nodes.insert(source.clone());
            for operation in &exits {
                let target = LifecycleNodeId::RequiredExit {
                    object,
                    operation: *operation,
                };
                nodes.insert(target.clone());
                edges.push(LifecycleDependencyDeclaration {
                    source: source.clone(),
                    target,
                    edge: LifecycleEdge::RequiresExit,
                });
            }
        }
    }
    (
        nodes.into_iter().map(|id| LifecycleNode { id }).collect(),
        edges,
    )
}

#[allow(clippy::too_many_lines)] // Keep the complete disclosure graph beside its fact census.
fn disclosure() -> (
    Vec<DisclosureNode>,
    Vec<DisclosureDependencyDeclaration>,
    Vec<DisclosureSeed>,
) {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for relation in [
        op(RelationKind::Constructibility),
        representation(ObjectId::Ash),
        representation(ObjectId::State),
        op(RelationKind::ClearClamp),
        op(RelationKind::ClearFloor),
        op(RelationKind::ClearFlow),
        op(RelationKind::ClearBinding),
    ] {
        nodes.push(DisclosureNode::Relation { id: relation });
    }
    for field in StateField::ALL {
        nodes.push(DisclosureNode::Relation {
            id: id(
                RelationKind::ClearAssignment,
                RelationSubject::StateField { field: *field },
            ),
        });
    }
    for (fact, availability) in facts() {
        nodes.push(DisclosureNode::Fact {
            id: fact.clone(),
            initial_visibility: if availability == AvailabilityClass::Public {
                InitialVisibility::Public
            } else {
                InitialVisibility::Private
            },
        });
        edges.push(DisclosureDependencyDeclaration {
            source: DisclosureNodeId::Fact(fact.clone()),
            target: DisclosureNodeId::Relation(op(RelationKind::Constructibility)),
            edge: DisclosureEdge::ConstructibilityInput,
        });
        if let FactId::FamilyAmount {
            object: ObjectId::Ash,
            ..
        } = &fact
        {
            edges.push(DisclosureDependencyDeclaration {
                source: DisclosureNodeId::Fact(fact.clone()),
                target: DisclosureNodeId::Relation(op(RelationKind::ClearClamp)),
                edge: DisclosureEdge::RelationOperand,
            });
            edges.push(DisclosureDependencyDeclaration {
                source: DisclosureNodeId::Fact(fact.clone()),
                target: DisclosureNodeId::Relation(op(RelationKind::ClearFlow)),
                edge: DisclosureEdge::RelationOperand,
            });
            edges.push(DisclosureDependencyDeclaration {
                source: DisclosureNodeId::Fact(fact.clone()),
                target: DisclosureNodeId::Relation(representation(ObjectId::Ash)),
                edge: DisclosureEdge::PublicObservableInput,
            });
        }
        if matches!(
            &fact,
            FactId::BoundValue {
                bound: BoundId::AshBatchMax,
            }
        ) {
            edges.push(DisclosureDependencyDeclaration {
                source: DisclosureNodeId::Fact(fact.clone()),
                target: DisclosureNodeId::Relation(op(RelationKind::ClearClamp)),
                edge: DisclosureEdge::RelationOperand,
            });
        }
        if let FactId::StateField { field, .. } = &fact {
            let mut targets = vec![id(
                RelationKind::ClearAssignment,
                RelationSubject::StateField { field: *field },
            )];
            if matches!(field, StateField::YL | StateField::YT) {
                targets.push(op(RelationKind::ClearClamp));
            }
            if matches!(field, StateField::YL | StateField::YT | StateField::Omega) {
                targets.push(op(RelationKind::ClearFloor));
            }
            for target in targets {
                edges.push(DisclosureDependencyDeclaration {
                    source: DisclosureNodeId::Fact(fact.clone()),
                    target: DisclosureNodeId::Relation(target),
                    edge: DisclosureEdge::RelationOperand,
                });
            }
        }
    }
    for field in StateField::ALL {
        let fact = FactId::StateField {
            operation: OperationId::Clear,
            side: TransactionSide::Output,
            field: *field,
        };
        nodes.push(DisclosureNode::Fact {
            id: fact.clone(),
            initial_visibility: InitialVisibility::Public,
        });
        edges.push(DisclosureDependencyDeclaration {
            source: DisclosureNodeId::Fact(fact.clone()),
            target: DisclosureNodeId::Relation(id(
                RelationKind::ClearAssignment,
                RelationSubject::StateField { field: *field },
            )),
            edge: DisclosureEdge::RelationOperand,
        });
        if matches!(field, StateField::YL | StateField::YT | StateField::Omega) {
            edges.push(DisclosureDependencyDeclaration {
                source: DisclosureNodeId::Fact(fact.clone()),
                target: DisclosureNodeId::Relation(op(RelationKind::ClearFloor)),
                edge: DisclosureEdge::RelationOperand,
            });
            edges.push(DisclosureDependencyDeclaration {
                source: DisclosureNodeId::Fact(fact),
                target: DisclosureNodeId::Relation(op(RelationKind::ClearBinding)),
                edge: DisclosureEdge::PublicObservableInput,
            });
        }
    }
    let seeds = vec![
        DisclosureSeed {
            node: DisclosureNodeId::Relation(representation(ObjectId::State)),
            reason: DisclosureReason::PublicState,
        },
        DisclosureSeed {
            node: DisclosureNodeId::Relation(representation(ObjectId::Ash)),
            reason: DisclosureReason::PublicInterface,
        },
        DisclosureSeed {
            node: DisclosureNodeId::Relation(op(RelationKind::ClearBinding)),
            reason: DisclosureReason::PublicEvent,
        },
        DisclosureSeed {
            node: DisclosureNodeId::Relation(op(RelationKind::Constructibility)),
            reason: DisclosureReason::PermissionlessConstructibility {
                operation: OperationId::Clear,
                relation: op(RelationKind::Constructibility),
            },
        },
    ];
    (nodes, edges, seeds)
}
