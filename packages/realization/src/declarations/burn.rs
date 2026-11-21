//! Complete target-independent Burn declaration.
//!
//! The owner census implements (`[PLAN-tab:realization:burn-clear-graphs]`).
//! Private change retains its deferred Burn exit; it never admits private inputs.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    Architecture, AssetId, BoundId, ObjectId, OpenFlowKind, OperationId, ProjectionId,
    ProjectionRule, RootId, RootUse,
};

use crate::{
    AvailabilityClass, CardinalityMaximum, ConstructibilityClass,
    ConstructibilityDependencyDeclaration, ConstructibilityEdge, ConstructibilityEdgeRole,
    ConstructibilityNode, ConstructibilityNodeId, Count, DisclosureDependencyDeclaration,
    DisclosureEdge, DisclosureNode, DisclosureNodeId, DisclosureReason, DisclosureSeed, FactId,
    InitialVisibility, LifecycleDependencyDeclaration, LifecycleEdge, LifecycleNode,
    LifecycleNodeId, ObservedSide, OperationRealization, ProofAlternativeId, ProofKind,
    RealizationError, Relation, RelationDeclaration, RelationDependencyDeclaration, RelationEdge,
    RelationId, RelationKind, RelationSubject, RepresentationMode, RequirementStrength,
    TransactionSide, WitnessRole,
};

pub fn derive(architecture: &Architecture) -> Result<OperationRealization, RealizationError> {
    crate::validate::validate_burn_architecture(architecture)?;
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
        operation: OperationId::Burn,
        // Burn has no stored-field assignment or unused expression.
        expressions: Vec::new(),
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
    RelationId::new(OperationId::Burn, kind, subject)
}

fn op(kind: RelationKind) -> RelationId {
    id(kind, RelationSubject::Operation)
}

fn family(kind: RelationKind, side: TransactionSide, object: ObjectId) -> RelationId {
    id(kind, RelationSubject::ObjectFamily { side, object })
}

fn conservation() -> RelationId {
    id(
        RelationKind::Conservation,
        RelationSubject::Asset { asset: AssetId::U },
    )
}

fn authorization() -> RelationId {
    family(
        RelationKind::Authorization,
        TransactionSide::Input,
        ObjectId::ReceiptLive,
    )
}

fn sponsor(kind: RelationKind) -> RelationId {
    id(kind, RelationSubject::Sponsor)
}

fn representation(object: ObjectId) -> RelationId {
    if object == ObjectId::ReceiptLive {
        id(
            RelationKind::SideRepresentation,
            RelationSubject::SideRepresentation {
                side: TransactionSide::Output,
                object,
            },
        )
    } else {
        id(
            RelationKind::Representation,
            RelationSubject::Representation { object },
        )
    }
}

fn exit(object: ObjectId, exit: OperationId) -> RelationId {
    id(
        RelationKind::Lifecycle,
        RelationSubject::LifecycleExit { object, exit },
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

fn families() -> [(TransactionSide, ObjectId, Count, CardinalityMaximum); 5] {
    use TransactionSide::{Input, Output};
    [
        (
            Input,
            ObjectId::ReceiptLive,
            Count::ONE,
            CardinalityMaximum::Bound(BoundId::BurnInputMax),
        ),
        (
            Output,
            ObjectId::ReceiptLive,
            Count::ZERO,
            CardinalityMaximum::Bound(BoundId::BurnChangeMax),
        ),
        (
            Output,
            ObjectId::Ash,
            Count::ONE,
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

#[allow(clippy::too_many_lines)] // Keep the complete relation census in one owner.
fn relations() -> Vec<RelationDeclaration> {
    let mut rows = crate::burn_clear_laws::burn_clear_laws(OperationId::Burn)
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
                asset: if object == ObjectId::PlainLbtc {
                    AssetId::Lbtc
                } else {
                    AssetId::U
                },
            },
            Some(ProofKind::ManifestShape),
        ));
    }
    for (side, observed, allowed) in [
        (
            TransactionSide::Input,
            ObservedSide::Input,
            BTreeSet::from([ObjectId::ReceiptLive, ObjectId::PlainLbtc]),
        ),
        (
            TransactionSide::Output,
            ObservedSide::Output,
            BTreeSet::from([ObjectId::ReceiptLive, ObjectId::Ash, ObjectId::PlainLbtc]),
        ),
    ] {
        rows.push(row(
            id(
                RelationKind::AllowedObjectFamilies,
                RelationSubject::TransactionSide { side },
            ),
            Relation::AllowedObjectFamilies {
                side: observed,
                allowed,
            },
            Some(ProofKind::ManifestShape),
        ));
    }
    rows.extend([
        row(
            authorization(),
            Relation::OwnerAuthorization {
                object: ObjectId::ReceiptLive,
            },
            Some(ProofKind::SignerMembership),
        ),
        RelationDeclaration {
            proof_alternatives: [
                ProofKind::PublicArithmetic,
                ProofKind::ConfidentialConservation,
            ]
            .into_iter()
            .map(|proof| ProofAlternativeId::new(conservation(), proof))
            .collect(),
            id: conservation(),
            relation: Relation::AmountConservation {
                asset: AssetId::U,
                input_objects: BTreeSet::from([ObjectId::ReceiptLive]),
                output_objects: BTreeSet::from([ObjectId::ReceiptLive, ObjectId::Ash]),
            },
        },
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
                    .map(|root| (*root, RootUse::Forbidden))
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
                                ProjectionId::TransitionCertificate | ProjectionId::BurnEvent => {
                                    ProjectionRule::Required
                                }
                                ProjectionId::ClearEvent | ProjectionId::DistributionResidue => {
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
                class: ConstructibilityClass::OwnersOf {
                    object: ObjectId::ReceiptLive,
                },
            },
            Some(ProofKind::SignerMembership),
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
            ObjectId::ReceiptLive,
            vec![
                OperationId::TransferLive,
                OperationId::Burn,
                OperationId::Redeem,
            ],
        ),
        (
            ObjectId::Ash,
            vec![OperationId::CompactAsh, OperationId::Clear],
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

#[allow(clippy::too_many_lines)] // Keep the complete dependency census in one owner.
fn dependencies() -> Vec<RelationDependencyDeclaration> {
    use RelationEdge as E;
    use RelationKind as K;
    let mut edges = crate::burn_clear_laws::burn_law_dependencies();
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
        } else {
            edges.extend([
                dependency(
                    recognition.clone(),
                    conservation(),
                    E::RecognitionBeforeValue,
                ),
                dependency(
                    cardinality.clone(),
                    conservation(),
                    E::CardinalityBeforeValue,
                ),
                dependency(
                    recognition.clone(),
                    op(K::BurnBinding),
                    E::StaticRequirement,
                ),
                dependency(cardinality, op(K::BurnBinding), E::StaticRequirement),
            ]);
            if side == TransactionSide::Output {
                edges.push(dependency(
                    recognition,
                    representation(object),
                    E::StaticRequirement,
                ));
            }
        }
    }
    edges.extend([
        dependency(
            op(K::CanonicalDeltaPolicy),
            conservation(),
            E::CanonicalDeltaBeforeValue,
        ),
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
        dependency(authorization(), op(K::BurnBinding), E::StaticRequirement),
        dependency(conservation(), op(K::BurnBinding), E::StaticRequirement),
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
        (TransactionSide::Input, ObjectId::ReceiptLive),
        (TransactionSide::Output, ObjectId::ReceiptLive),
        (TransactionSide::Output, ObjectId::Ash),
    ] {
        facts.push((
            FactId::FamilyCount {
                operation: OperationId::Burn,
                side,
                object,
            },
            AvailabilityClass::Public,
        ));
        facts.push((
            FactId::FamilyAmount {
                operation: OperationId::Burn,
                side,
                object,
            },
            if side == TransactionSide::Output && object == ObjectId::ReceiptLive {
                AvailabilityClass::InputOwners { object }
            } else {
                AvailabilityClass::Public
            },
        ));
    }
    for bound in [
        BoundId::BurnInputMax,
        BoundId::BurnChangeMax,
        BoundId::BurnRecordMax,
        BoundId::FeeSponsorInputMax,
    ] {
        facts.push((FactId::BoundValue { bound }, AvailabilityClass::Public));
    }
    facts
}

fn construction() -> (
    Vec<ConstructibilityNode>,
    Vec<ConstructibilityDependencyDeclaration>,
) {
    let operation = ConstructibilityNodeId::Operation(OperationId::Burn);
    let mut nodes = vec![ConstructibilityNode {
        id: operation.clone(),
    }];
    let mut edges = Vec::new();
    for (fact, availability) in facts() {
        let source = ConstructibilityNodeId::Fact {
            operation: OperationId::Burn,
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
    for (role, availability, edge_role, strength) in [
        (
            WitnessRole::ProtocolOwnerAuthorization,
            AvailabilityClass::InputOwners {
                object: ObjectId::ReceiptLive,
            },
            ConstructibilityEdgeRole::RequiredWitness,
            RequirementStrength::Required,
        ),
        (
            WitnessRole::SponsorAuthorization,
            AvailabilityClass::SponsorLocal,
            ConstructibilityEdgeRole::SponsorOnly,
            RequirementStrength::Optional,
        ),
    ] {
        let source = ConstructibilityNodeId::Witness {
            operation: OperationId::Burn,
            role,
            availability,
        };
        nodes.push(ConstructibilityNode { id: source.clone() });
        edges.push(ConstructibilityDependencyDeclaration {
            source,
            target: operation.clone(),
            edge: ConstructibilityEdge {
                role: edge_role,
                strength,
            },
        });
    }
    (nodes, edges)
}

fn lifecycle() -> (Vec<LifecycleNode>, Vec<LifecycleDependencyDeclaration>) {
    let mut nodes = BTreeSet::new();
    let mut edges = Vec::new();
    for (object, exits) in exits() {
        let second = if object == ObjectId::Ash {
            RepresentationMode::PublicCommitted
        } else {
            RepresentationMode::PrivateCommitted
        };
        for mode in [RepresentationMode::Explicit, second] {
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

fn disclosure() -> (
    Vec<DisclosureNode>,
    Vec<DisclosureDependencyDeclaration>,
    Vec<DisclosureSeed>,
) {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for relation in [
        op(RelationKind::Constructibility),
        conservation(),
        representation(ObjectId::Ash),
        representation(ObjectId::ReceiptLive),
        op(RelationKind::BurnBinding),
    ] {
        nodes.push(DisclosureNode::Relation { id: relation });
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
        if let FactId::FamilyAmount { side, object, .. } = &fact {
            edges.push(DisclosureDependencyDeclaration {
                source: DisclosureNodeId::Fact(fact.clone()),
                target: DisclosureNodeId::Relation(conservation()),
                edge: DisclosureEdge::RelationOperand,
            });
            if *side == TransactionSide::Output {
                edges.push(DisclosureDependencyDeclaration {
                    source: DisclosureNodeId::Fact(fact.clone()),
                    target: DisclosureNodeId::Relation(representation(*object)),
                    edge: DisclosureEdge::PublicObservableInput,
                });
            }
            if *object == ObjectId::Ash {
                edges.push(DisclosureDependencyDeclaration {
                    source: DisclosureNodeId::Fact(fact),
                    target: DisclosureNodeId::Relation(op(RelationKind::BurnBinding)),
                    edge: DisclosureEdge::PublicObservableInput,
                });
            }
        }
    }
    let seeds = vec![
        DisclosureSeed {
            node: DisclosureNodeId::Relation(representation(ObjectId::Ash)),
            reason: DisclosureReason::PublicInterface,
        },
        DisclosureSeed {
            node: DisclosureNodeId::Relation(op(RelationKind::BurnBinding)),
            reason: DisclosureReason::PublicEvent,
        },
    ];
    (nodes, edges, seeds)
}
