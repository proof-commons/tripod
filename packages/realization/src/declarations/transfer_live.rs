//! Target-independent live-receipt transfer declaration.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    Architecture, AssetId, BoundId, DeltaKind, ObjectId, OpenFlowKind, OperationId, ProjectionId,
    ProjectionRule, RootId, RootUse,
};

use crate::{
    AvailabilityClass, CardinalityMaximum, ConstructibilityDependencyDeclaration,
    ConstructibilityEdge, ConstructibilityEdgeRole, ConstructibilityNode, ConstructibilityNodeId,
    Count, DisclosureDependencyDeclaration, DisclosureEdge, DisclosureNode, DisclosureNodeId,
    DisclosureSeed, ExpectedCanonicalDelta, FactId, InitialVisibility,
    LifecycleDependencyDeclaration, LifecycleEdge, LifecycleNode, LifecycleNodeId,
    OperationRealization, ProofAlternativeId, ProofKind, RealizationError, Relation,
    RelationDeclaration, RelationDependencyDeclaration, RelationEdge, RelationId, RelationKind,
    RelationSubject, RepresentationMode, RequirementStrength, TransactionSide, WitnessRole,
    validate::validate_live_transfer_architecture,
};

#[allow(clippy::too_many_lines)]
pub fn derive(architecture: &Architecture) -> Result<OperationRealization, RealizationError> {
    validate_live_transfer_architecture(architecture)?;

    let ids = Ids::new();
    let relations = relation_declarations(&ids);
    let relation_dependencies = relation_dependencies(&ids);
    let (constructibility_nodes, constructibility_edges) = constructibility_declarations();
    let (lifecycle_nodes, lifecycle_edges) = lifecycle_declarations();
    let (disclosure_nodes, disclosure_edges, disclosure_seeds) = disclosure_declarations(&ids);

    Ok(OperationRealization {
        operation: OperationId::TransferLive,
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
    })
}

struct Ids {
    input_cardinality: RelationId,
    output_cardinality: RelationId,
    sponsor_input_cardinality: RelationId,
    sponsor_output_cardinality: RelationId,
    input_recognition: RelationId,
    output_recognition: RelationId,
    sponsor_input_recognition: RelationId,
    sponsor_output_recognition: RelationId,
    authorization: RelationId,
    input_closure: RelationId,
    output_closure: RelationId,
    conservation: RelationId,
    sponsor: RelationId,
    sponsor_multiplicity: RelationId,
    substrate_conservation: RelationId,
    open_flow_policy: RelationId,
    canonical_delta_policy: RelationId,
    roots: RelationId,
    projections: RelationId,
    constructibility: RelationId,
    representation: RelationId,
    transfer_exit: RelationId,
    burn_exit: RelationId,
    redeem_exit: RelationId,
}

impl Ids {
    fn new() -> Self {
        Self {
            input_cardinality: object_relation(RelationKind::Cardinality, TransactionSide::Input),
            output_cardinality: object_relation(RelationKind::Cardinality, TransactionSide::Output),
            sponsor_input_cardinality: sponsor_cardinality(TransactionSide::Input),
            sponsor_output_cardinality: sponsor_cardinality(TransactionSide::Output),
            input_recognition: object_relation(RelationKind::Recognition, TransactionSide::Input),
            output_recognition: object_relation(RelationKind::Recognition, TransactionSide::Output),
            sponsor_input_recognition: sponsor_recognition(TransactionSide::Input),
            sponsor_output_recognition: sponsor_recognition(TransactionSide::Output),
            authorization: object_relation(RelationKind::Authorization, TransactionSide::Input),
            input_closure: side_relation(
                RelationKind::AllowedObjectFamilies,
                TransactionSide::Input,
            ),
            output_closure: side_relation(
                RelationKind::AllowedObjectFamilies,
                TransactionSide::Output,
            ),
            conservation: id(
                RelationKind::Conservation,
                RelationSubject::Asset { asset: AssetId::U },
            ),
            sponsor: id(RelationKind::SponsorIsolation, RelationSubject::Sponsor),
            sponsor_multiplicity: id(
                RelationKind::SponsorEnvelopeMultiplicity,
                RelationSubject::Sponsor,
            ),
            substrate_conservation: id(
                RelationKind::SubstrateConservation,
                RelationSubject::Asset {
                    asset: AssetId::Lbtc,
                },
            ),
            open_flow_policy: id(RelationKind::OpenFlowPolicy, RelationSubject::Operation),
            canonical_delta_policy: id(
                RelationKind::CanonicalDeltaPolicy,
                RelationSubject::Operation,
            ),
            roots: id(RelationKind::RootPolicy, RelationSubject::Operation),
            projections: id(RelationKind::ProjectionPolicy, RelationSubject::Operation),
            constructibility: id(RelationKind::Constructibility, RelationSubject::Operation),
            representation: id(
                RelationKind::Representation,
                RelationSubject::Representation {
                    object: ObjectId::ReceiptLive,
                },
            ),
            transfer_exit: lifecycle(OperationId::TransferLive),
            burn_exit: lifecycle(OperationId::Burn),
            redeem_exit: lifecycle(OperationId::Redeem),
        }
    }
}

#[allow(clippy::too_many_lines)]
fn relation_declarations(ids: &Ids) -> Vec<RelationDeclaration> {
    let expected_roots = RootId::ALL
        .iter()
        .map(|root| (*root, RootUse::Forbidden))
        .collect::<BTreeMap<_, _>>();
    let expected_projections = ProjectionId::ALL
        .iter()
        .map(|projection| {
            let rule = if *projection == ProjectionId::TransitionCertificate {
                ProjectionRule::Required
            } else {
                ProjectionRule::Forbidden
            };
            (*projection, rule)
        })
        .collect::<BTreeMap<_, _>>();

    vec![
        declaration(
            ids.input_cardinality.clone(),
            Relation::Cardinality {
                side: crate::ObservedSide::Input,
                object: ObjectId::ReceiptLive,
                minimum: Count::ONE,
                maximum: CardinalityMaximum::Bound(BoundId::TransferInputMax),
            },
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.output_cardinality.clone(),
            Relation::Cardinality {
                side: crate::ObservedSide::Output,
                object: ObjectId::ReceiptLive,
                minimum: Count::ONE,
                maximum: CardinalityMaximum::Bound(BoundId::TransferOutputMax),
            },
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.sponsor_input_cardinality.clone(),
            Relation::Cardinality {
                side: crate::ObservedSide::Input,
                object: ObjectId::PlainLbtc,
                minimum: Count::ZERO,
                maximum: CardinalityMaximum::Bound(BoundId::FeeSponsorInputMax),
            },
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.sponsor_output_cardinality.clone(),
            Relation::Cardinality {
                side: crate::ObservedSide::Output,
                object: ObjectId::PlainLbtc,
                minimum: Count::ZERO,
                maximum: CardinalityMaximum::Exact(Count::ONE),
            },
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.input_recognition.clone(),
            Relation::Recognition {
                side: crate::ObservedSide::Input,
                object: ObjectId::ReceiptLive,
                asset: AssetId::U,
            },
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.output_recognition.clone(),
            Relation::Recognition {
                side: crate::ObservedSide::Output,
                object: ObjectId::ReceiptLive,
                asset: AssetId::U,
            },
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.sponsor_input_recognition.clone(),
            Relation::Recognition {
                side: crate::ObservedSide::Input,
                object: ObjectId::PlainLbtc,
                asset: AssetId::Lbtc,
            },
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.sponsor_output_recognition.clone(),
            Relation::Recognition {
                side: crate::ObservedSide::Output,
                object: ObjectId::PlainLbtc,
                asset: AssetId::Lbtc,
            },
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.authorization.clone(),
            Relation::OwnerAuthorization {
                object: ObjectId::ReceiptLive,
            },
            [ProofKind::SignerMembership],
        ),
        declaration(
            ids.input_closure.clone(),
            Relation::AllowedObjectFamilies {
                side: crate::ObservedSide::Input,
                allowed: BTreeSet::from([ObjectId::ReceiptLive, ObjectId::PlainLbtc]),
            },
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.output_closure.clone(),
            Relation::AllowedObjectFamilies {
                side: crate::ObservedSide::Output,
                allowed: BTreeSet::from([ObjectId::ReceiptLive, ObjectId::PlainLbtc]),
            },
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.conservation.clone(),
            Relation::AmountConservation {
                asset: AssetId::U,
                input_objects: BTreeSet::from([ObjectId::ReceiptLive]),
                output_objects: BTreeSet::from([ObjectId::ReceiptLive]),
            },
            [
                ProofKind::PublicArithmetic,
                ProofKind::ConfidentialConservation,
            ],
        ),
        declaration(
            ids.sponsor.clone(),
            Relation::SponsorIsolation,
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.substrate_conservation.clone(),
            Relation::SubstrateConservation {
                asset: AssetId::Lbtc,
            },
            [ProofKind::SubstrateConservation],
        ),
        declaration(
            ids.sponsor_multiplicity.clone(),
            Relation::SponsorEnvelopeMultiplicity {
                maximum: Count::ONE,
            },
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.open_flow_policy.clone(),
            Relation::OpenFlowPolicy {
                allowed: BTreeSet::from([OpenFlowKind::FeeSponsor]),
            },
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.canonical_delta_policy.clone(),
            Relation::CanonicalDeltaPolicy {
                expected: BTreeSet::from([ExpectedCanonicalDelta {
                    asset: AssetId::U,
                    kind: DeltaKind::Lateral,
                    destruction_tag: None,
                }]),
            },
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.roots.clone(),
            Relation::RootPolicy {
                expected: expected_roots,
            },
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.projections.clone(),
            Relation::ProjectionPolicy {
                expected: expected_projections,
            },
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.constructibility.clone(),
            Relation::Constructibility {
                class: crate::ConstructibilityClass::OwnersOf {
                    object: ObjectId::ReceiptLive,
                },
            },
            [ProofKind::SignerMembership],
        ),
        // A mode constraint, not a second arithmetic proof: the
        // conservation relation owns how value preservation is proved,
        // and an independent proof alternative here could contradict
        // the mode this relation admits.
        relation_only(
            ids.representation.clone(),
            Relation::Representation {
                object: ObjectId::ReceiptLive,
                allowed: BTreeSet::from([
                    RepresentationMode::Explicit,
                    RepresentationMode::PrivateCommitted,
                ]),
            },
        ),
        relation_only(
            ids.transfer_exit.clone(),
            Relation::LifecycleExit {
                object: ObjectId::ReceiptLive,
                exit: OperationId::TransferLive,
            },
        ),
        relation_only(
            ids.burn_exit.clone(),
            Relation::LifecycleExit {
                object: ObjectId::ReceiptLive,
                exit: OperationId::Burn,
            },
        ),
        relation_only(
            ids.redeem_exit.clone(),
            Relation::LifecycleExit {
                object: ObjectId::ReceiptLive,
                exit: OperationId::Redeem,
            },
        ),
    ]
}

#[allow(clippy::too_many_lines)]
fn relation_dependencies(ids: &Ids) -> Vec<RelationDependencyDeclaration> {
    vec![
        dep(
            &ids.input_recognition,
            &ids.input_cardinality,
            RelationEdge::RecognitionBeforeCardinality,
        ),
        dep(
            &ids.output_recognition,
            &ids.output_cardinality,
            RelationEdge::RecognitionBeforeCardinality,
        ),
        dep(
            &ids.sponsor_input_recognition,
            &ids.sponsor_input_cardinality,
            RelationEdge::RecognitionBeforeCardinality,
        ),
        dep(
            &ids.sponsor_output_recognition,
            &ids.sponsor_output_cardinality,
            RelationEdge::RecognitionBeforeCardinality,
        ),
        dep(
            &ids.input_cardinality,
            &ids.conservation,
            RelationEdge::CardinalityBeforeValue,
        ),
        dep(
            &ids.output_cardinality,
            &ids.conservation,
            RelationEdge::CardinalityBeforeValue,
        ),
        dep(
            &ids.input_recognition,
            &ids.conservation,
            RelationEdge::RecognitionBeforeValue,
        ),
        dep(
            &ids.output_recognition,
            &ids.conservation,
            RelationEdge::RecognitionBeforeValue,
        ),
        dep(
            &ids.canonical_delta_policy,
            &ids.conservation,
            RelationEdge::CanonicalDeltaBeforeValue,
        ),
        dep(
            &ids.authorization,
            &ids.input_closure,
            RelationEdge::AuthorizationBeforeClosure,
        ),
        dep(
            &ids.authorization,
            &ids.constructibility,
            RelationEdge::AuthorizationBeforeConstructibility,
        ),
        dep(
            &ids.open_flow_policy,
            &ids.sponsor,
            RelationEdge::OpenFlowPolicyBeforeSponsor,
        ),
        dep(
            &ids.open_flow_policy,
            &ids.sponsor_multiplicity,
            RelationEdge::StaticRequirement,
        ),
        dep(
            &ids.sponsor_multiplicity,
            &ids.sponsor,
            RelationEdge::StaticRequirement,
        ),
        dep(
            &ids.sponsor_input_recognition,
            &ids.sponsor,
            RelationEdge::StaticRequirement,
        ),
        dep(
            &ids.sponsor_output_recognition,
            &ids.sponsor,
            RelationEdge::StaticRequirement,
        ),
        dep(
            &ids.sponsor,
            &ids.constructibility,
            RelationEdge::SponsorBeforeConstructibility,
        ),
        dep(
            &ids.sponsor,
            &ids.substrate_conservation,
            RelationEdge::SponsorBeforeOperation,
        ),
        dep(
            &ids.conservation,
            &ids.representation,
            RelationEdge::StaticRequirement,
        ),
        dep(
            &ids.constructibility,
            &ids.representation,
            RelationEdge::StaticRequirement,
        ),
        dep(
            &ids.representation,
            &ids.transfer_exit,
            RelationEdge::RepresentationBeforeLifecycle,
        ),
        dep(
            &ids.representation,
            &ids.burn_exit,
            RelationEdge::RepresentationBeforeLifecycle,
        ),
        dep(
            &ids.representation,
            &ids.redeem_exit,
            RelationEdge::RepresentationBeforeLifecycle,
        ),
        dep(
            &ids.roots,
            &ids.constructibility,
            RelationEdge::RootPolicyBeforeOperation,
        ),
        dep(
            &ids.projections,
            &ids.constructibility,
            RelationEdge::ProjectionPolicyBeforeOperation,
        ),
    ]
}

fn constructibility_declarations() -> (
    Vec<ConstructibilityNode>,
    Vec<ConstructibilityDependencyDeclaration>,
) {
    let operation = ConstructibilityNodeId::Operation(OperationId::TransferLive);
    let owner_witness = ConstructibilityNodeId::Witness {
        operation: OperationId::TransferLive,
        role: WitnessRole::ProtocolOwnerAuthorization,
        availability: AvailabilityClass::InputOwners {
            object: ObjectId::ReceiptLive,
        },
    };
    let sponsor = ConstructibilityNodeId::Witness {
        operation: OperationId::TransferLive,
        role: WitnessRole::SponsorAuthorization,
        availability: AvailabilityClass::SponsorLocal,
    };
    let nodes = vec![
        ConstructibilityNode {
            id: operation.clone(),
        },
        ConstructibilityNode {
            id: owner_witness.clone(),
        },
        ConstructibilityNode {
            id: sponsor.clone(),
        },
    ];
    let edges = vec![
        cedge(
            owner_witness,
            operation.clone(),
            ConstructibilityEdgeRole::RequiredWitness,
            RequirementStrength::Required,
        ),
        cedge(
            sponsor,
            operation,
            ConstructibilityEdgeRole::SponsorOnly,
            RequirementStrength::Optional,
        ),
    ];
    (nodes, edges)
}

fn lifecycle_declarations() -> (Vec<LifecycleNode>, Vec<LifecycleDependencyDeclaration>) {
    lifecycle_for(
        ObjectId::ReceiptLive,
        [
            RepresentationMode::Explicit,
            RepresentationMode::PrivateCommitted,
        ],
        [
            OperationId::TransferLive,
            OperationId::Burn,
            OperationId::Redeem,
        ],
    )
}

fn disclosure_declarations(
    ids: &Ids,
) -> (
    Vec<DisclosureNode>,
    Vec<DisclosureDependencyDeclaration>,
    Vec<DisclosureSeed>,
) {
    let live_input_amount = FactId::FamilyAmount {
        operation: OperationId::TransferLive,
        side: TransactionSide::Input,
        object: ObjectId::ReceiptLive,
    };
    let live_output_amount = FactId::FamilyAmount {
        operation: OperationId::TransferLive,
        side: TransactionSide::Output,
        object: ObjectId::ReceiptLive,
    };
    let nodes = vec![
        disclosure_fact(live_input_amount.clone(), InitialVisibility::Private),
        disclosure_fact(live_output_amount.clone(), InitialVisibility::Private),
        DisclosureNode::Relation {
            id: ids.conservation.clone(),
        },
    ];
    let edges = vec![
        dedge(
            DisclosureNodeId::Fact(live_input_amount),
            DisclosureNodeId::Relation(ids.conservation.clone()),
            DisclosureEdge::RelationOperand,
        ),
        dedge(
            DisclosureNodeId::Fact(live_output_amount),
            DisclosureNodeId::Relation(ids.conservation.clone()),
            DisclosureEdge::RelationOperand,
        ),
    ];
    (nodes, edges, Vec::new())
}

fn id(kind: RelationKind, subject: RelationSubject) -> RelationId {
    RelationId::new(OperationId::TransferLive, kind, subject)
}

fn object_relation(kind: RelationKind, side: TransactionSide) -> RelationId {
    id(
        kind,
        RelationSubject::ObjectFamily {
            side,
            object: ObjectId::ReceiptLive,
        },
    )
}

fn side_relation(kind: RelationKind, side: TransactionSide) -> RelationId {
    id(kind, RelationSubject::TransactionSide { side })
}

fn sponsor_cardinality(side: TransactionSide) -> RelationId {
    id(
        kind_cardinality(),
        RelationSubject::ObjectFamily {
            side,
            object: ObjectId::PlainLbtc,
        },
    )
}

fn sponsor_recognition(side: TransactionSide) -> RelationId {
    id(
        RelationKind::Recognition,
        RelationSubject::ObjectFamily {
            side,
            object: ObjectId::PlainLbtc,
        },
    )
}

const fn kind_cardinality() -> RelationKind {
    RelationKind::Cardinality
}

fn lifecycle(exit: OperationId) -> RelationId {
    id(
        RelationKind::Lifecycle,
        RelationSubject::LifecycleExit {
            object: ObjectId::ReceiptLive,
            exit,
        },
    )
}

fn declaration<const A: usize>(
    id: RelationId,
    relation: Relation,
    proof_kinds: [ProofKind; A],
) -> RelationDeclaration {
    let proof_alternatives = proof_kinds
        .into_iter()
        .map(|kind| ProofAlternativeId::new(id.clone(), kind))
        .collect();
    RelationDeclaration {
        id,
        relation,
        proof_alternatives,
    }
}

fn relation_only(id: RelationId, relation: Relation) -> RelationDeclaration {
    RelationDeclaration {
        id,
        relation,
        proof_alternatives: BTreeSet::new(),
    }
}

fn dep(
    prerequisite: &RelationId,
    dependent: &RelationId,
    edge: RelationEdge,
) -> RelationDependencyDeclaration {
    RelationDependencyDeclaration {
        prerequisite: prerequisite.clone(),
        dependent: dependent.clone(),
        edge,
    }
}

fn cedge(
    source: ConstructibilityNodeId,
    target: ConstructibilityNodeId,
    role: ConstructibilityEdgeRole,
    strength: RequirementStrength,
) -> ConstructibilityDependencyDeclaration {
    ConstructibilityDependencyDeclaration {
        source,
        target,
        edge: ConstructibilityEdge { role, strength },
    }
}

fn lifecycle_for<const M: usize, const E: usize>(
    object: ObjectId,
    modes: [RepresentationMode; M],
    exits: [OperationId; E],
) -> (Vec<LifecycleNode>, Vec<LifecycleDependencyDeclaration>) {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for mode in modes {
        let source = LifecycleNodeId::Representation { object, mode };
        nodes.push(LifecycleNode { id: source.clone() });
        for exit in exits {
            let target = LifecycleNodeId::RequiredExit {
                object,
                operation: exit,
            };
            nodes.push(LifecycleNode { id: target.clone() });
            edges.push(LifecycleDependencyDeclaration {
                source: source.clone(),
                target,
                edge: LifecycleEdge::RequiresExit,
            });
        }
    }
    nodes.sort_by(|left, right| left.id.cmp(&right.id));
    nodes.dedup_by(|left, right| left.id == right.id);
    (nodes, edges)
}

fn disclosure_fact(id: FactId, initial_visibility: InitialVisibility) -> DisclosureNode {
    DisclosureNode::Fact {
        id,
        initial_visibility,
    }
}

fn dedge(
    source: DisclosureNodeId,
    target: DisclosureNodeId,
    edge: DisclosureEdge,
) -> DisclosureDependencyDeclaration {
    DisclosureDependencyDeclaration {
        source,
        target,
        edge,
    }
}
