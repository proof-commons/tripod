//! State and sponsor structure for maturity announcements.
//!
//! Operator authorization requires external evidence. State admits explicit
//! and public committed representations, with six lifecycle exits and nine
//! public facts. The typed State transition owns the maturity law: this
//! declaration omits the window predicate and contains no model or target facts.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    Architecture, AssetId, BoundId, ObjectId, OpenFlowKind, OperationId, ProjectionId,
    ProjectionRule, RootId, RootUse,
};

use crate::{
    AnnouncementLeadBound, AvailabilityClass, CardinalityMaximum,
    ConstructibilityDependencyDeclaration, ConstructibilityEdge, ConstructibilityEdgeRole,
    ConstructibilityNode, ConstructibilityNodeId, Count, DisclosureNode, FactId, InitialVisibility,
    LifecycleDependencyDeclaration, LifecycleEdge, LifecycleNode, LifecycleNodeId,
    OperationRealization, ProofAlternativeId, ProofKind, RealizationError, Relation,
    RelationDeclaration, RelationDependencyDeclaration, RelationEdge, RelationId, RelationKind,
    RelationSubject, RepresentationMode, RequirementStrength, StateField, TransactionSide,
    WitnessRole, validate::validate_announce_maturity_architecture,
};

const EXITS: [OperationId; 6] = [
    OperationId::AdmitDeposits,
    OperationId::Cycle,
    OperationId::Redeem,
    OperationId::ReceiptRelabel,
    OperationId::Clear,
    OperationId::AnnounceMaturity,
];

pub fn derive(architecture: &Architecture) -> Result<OperationRealization, RealizationError> {
    validate_announce_maturity_architecture(architecture)?;

    let ids = Ids::new();
    let relations = relation_declarations(&ids);
    let relation_dependencies = relation_dependencies(&ids);
    let (constructibility_nodes, constructibility_edges) = constructibility_declarations();
    let (lifecycle_nodes, lifecycle_edges) = lifecycle_declarations();
    let disclosure_nodes = disclosure_declarations();

    Ok(OperationRealization {
        operation: OperationId::AnnounceMaturity,
        expressions: Vec::new(),
        relations,
        relation_dependencies,
        constructibility_nodes,
        constructibility_edges,
        lifecycle_nodes,
        lifecycle_edges,
        disclosure_nodes,
        disclosure_edges: Vec::new(),
        disclosure_seeds: Vec::new(),
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
    sponsor: RelationId,
    sponsor_multiplicity: RelationId,
    substrate_conservation: RelationId,
    open_flow_policy: RelationId,
    canonical_delta_policy: RelationId,
    roots: RelationId,
    projections: RelationId,
    constructibility: RelationId,
    representation: RelationId,
    exits: [RelationId; 6],
}

impl Ids {
    fn new() -> Self {
        Self {
            input_cardinality: state_family(RelationKind::Cardinality, TransactionSide::Input),
            output_cardinality: state_family(RelationKind::Cardinality, TransactionSide::Output),
            sponsor_input_cardinality: sponsor_family(
                RelationKind::Cardinality,
                TransactionSide::Input,
            ),
            sponsor_output_cardinality: sponsor_family(
                RelationKind::Cardinality,
                TransactionSide::Output,
            ),
            input_recognition: state_family(RelationKind::Recognition, TransactionSide::Input),
            output_recognition: state_family(RelationKind::Recognition, TransactionSide::Output),
            sponsor_input_recognition: sponsor_family(
                RelationKind::Recognition,
                TransactionSide::Input,
            ),
            sponsor_output_recognition: sponsor_family(
                RelationKind::Recognition,
                TransactionSide::Output,
            ),
            authorization: relation_id(RelationKind::Authorization, RelationSubject::Operation),
            input_closure: relation_id(
                RelationKind::AllowedObjectFamilies,
                RelationSubject::TransactionSide {
                    side: TransactionSide::Input,
                },
            ),
            output_closure: relation_id(
                RelationKind::AllowedObjectFamilies,
                RelationSubject::TransactionSide {
                    side: TransactionSide::Output,
                },
            ),
            sponsor: relation_id(RelationKind::SponsorIsolation, RelationSubject::Sponsor),
            sponsor_multiplicity: relation_id(
                RelationKind::SponsorEnvelopeMultiplicity,
                RelationSubject::Sponsor,
            ),
            substrate_conservation: relation_id(
                RelationKind::SubstrateConservation,
                RelationSubject::Asset {
                    asset: AssetId::Lbtc,
                },
            ),
            open_flow_policy: relation_id(RelationKind::OpenFlowPolicy, RelationSubject::Operation),
            canonical_delta_policy: relation_id(
                RelationKind::CanonicalDeltaPolicy,
                RelationSubject::Operation,
            ),
            roots: relation_id(RelationKind::RootPolicy, RelationSubject::Operation),
            projections: relation_id(RelationKind::ProjectionPolicy, RelationSubject::Operation),
            constructibility: relation_id(
                RelationKind::Constructibility,
                RelationSubject::Operation,
            ),
            representation: relation_id(
                RelationKind::Representation,
                RelationSubject::Representation {
                    object: ObjectId::State,
                },
            ),
            exits: EXITS.map(lifecycle),
        }
    }
}

fn relation_declarations(ids: &Ids) -> Vec<RelationDeclaration> {
    cardinality_relations(ids)
        .into_iter()
        .chain(recognition_relations(ids))
        .chain(structural_relations(ids))
        .chain(policy_relations(ids))
        .chain(lifecycle_relations(ids))
        .collect()
}

fn cardinality_relations(ids: &Ids) -> Vec<RelationDeclaration> {
    vec![
        declaration(
            ids.input_cardinality.clone(),
            Relation::Cardinality {
                side: crate::ObservedSide::Input,
                object: ObjectId::State,
                minimum: Count::ONE,
                maximum: CardinalityMaximum::Exact(Count::ONE),
            },
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.output_cardinality.clone(),
            Relation::Cardinality {
                side: crate::ObservedSide::Output,
                object: ObjectId::State,
                minimum: Count::ONE,
                maximum: CardinalityMaximum::Exact(Count::ONE),
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
    ]
}

fn recognition_relations(ids: &Ids) -> Vec<RelationDeclaration> {
    vec![
        declaration(
            ids.input_recognition.clone(),
            Relation::Recognition {
                side: crate::ObservedSide::Input,
                object: ObjectId::State,
                asset: AssetId::Pid,
            },
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.output_recognition.clone(),
            Relation::Recognition {
                side: crate::ObservedSide::Output,
                object: ObjectId::State,
                asset: AssetId::Pid,
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
    ]
}

fn structural_relations(ids: &Ids) -> Vec<RelationDeclaration> {
    vec![
        declaration(
            ids.authorization.clone(),
            Relation::OperatorAuthorization,
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.input_closure.clone(),
            Relation::AllowedObjectFamilies {
                side: crate::ObservedSide::Input,
                allowed: BTreeSet::from([ObjectId::State, ObjectId::PlainLbtc]),
            },
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.output_closure.clone(),
            Relation::AllowedObjectFamilies {
                side: crate::ObservedSide::Output,
                allowed: BTreeSet::from([ObjectId::State, ObjectId::PlainLbtc]),
            },
            [ProofKind::ManifestShape],
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
    ]
}

fn policy_relations(ids: &Ids) -> Vec<RelationDeclaration> {
    let expected_roots = RootId::ALL
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
                expected: BTreeSet::new(),
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
                class: crate::ConstructibilityClass::Operator,
            },
            [ProofKind::ManifestShape],
        ),
        relation_only(
            ids.representation.clone(),
            Relation::Representation {
                object: ObjectId::State,
                allowed: BTreeSet::from([
                    RepresentationMode::Explicit,
                    RepresentationMode::PublicCommitted,
                ]),
            },
        ),
    ]
}

fn lifecycle_relations(ids: &Ids) -> Vec<RelationDeclaration> {
    ids.exits
        .iter()
        .zip(EXITS)
        .map(|(id, exit)| {
            relation_only(
                id.clone(),
                Relation::LifecycleExit {
                    object: ObjectId::State,
                    exit,
                },
            )
        })
        .collect()
}

fn relation_dependencies(ids: &Ids) -> Vec<RelationDependencyDeclaration> {
    let mut edges = [
        (
            &ids.input_recognition,
            &ids.input_cardinality,
            RelationEdge::RecognitionBeforeCardinality,
        ),
        (
            &ids.output_recognition,
            &ids.output_cardinality,
            RelationEdge::RecognitionBeforeCardinality,
        ),
        (
            &ids.sponsor_input_recognition,
            &ids.sponsor_input_cardinality,
            RelationEdge::RecognitionBeforeCardinality,
        ),
        (
            &ids.sponsor_output_recognition,
            &ids.sponsor_output_cardinality,
            RelationEdge::RecognitionBeforeCardinality,
        ),
        (
            &ids.authorization,
            &ids.input_closure,
            RelationEdge::AuthorizationBeforeClosure,
        ),
        (
            &ids.authorization,
            &ids.constructibility,
            RelationEdge::AuthorizationBeforeConstructibility,
        ),
        (
            &ids.open_flow_policy,
            &ids.sponsor,
            RelationEdge::OpenFlowPolicyBeforeSponsor,
        ),
        (
            &ids.open_flow_policy,
            &ids.sponsor_multiplicity,
            RelationEdge::StaticRequirement,
        ),
        (
            &ids.sponsor_multiplicity,
            &ids.sponsor,
            RelationEdge::StaticRequirement,
        ),
        (
            &ids.sponsor_input_recognition,
            &ids.sponsor,
            RelationEdge::StaticRequirement,
        ),
        (
            &ids.sponsor_output_recognition,
            &ids.sponsor,
            RelationEdge::StaticRequirement,
        ),
        (
            &ids.sponsor,
            &ids.constructibility,
            RelationEdge::SponsorBeforeConstructibility,
        ),
        (
            &ids.sponsor,
            &ids.substrate_conservation,
            RelationEdge::SponsorBeforeOperation,
        ),
        (
            &ids.constructibility,
            &ids.representation,
            RelationEdge::StaticRequirement,
        ),
        (
            &ids.output_recognition,
            &ids.representation,
            RelationEdge::StaticRequirement,
        ),
        (
            &ids.roots,
            &ids.constructibility,
            RelationEdge::RootPolicyBeforeOperation,
        ),
        (
            &ids.projections,
            &ids.constructibility,
            RelationEdge::ProjectionPolicyBeforeOperation,
        ),
    ]
    .into_iter()
    .map(|(source, target, edge)| dep(source, target, edge))
    .collect::<Vec<_>>();
    edges.extend(ids.exits.iter().map(|exit| {
        dep(
            &ids.representation,
            exit,
            RelationEdge::RepresentationBeforeLifecycle,
        )
    }));
    edges
}

fn constructibility_declarations() -> (
    Vec<ConstructibilityNode>,
    Vec<ConstructibilityDependencyDeclaration>,
) {
    let operation = ConstructibilityNodeId::Operation(OperationId::AnnounceMaturity);
    let operator_witness = ConstructibilityNodeId::Witness {
        operation: OperationId::AnnounceMaturity,
        role: WitnessRole::OperatorAuthorization,
        availability: AvailabilityClass::Operator,
    };
    let sponsor = ConstructibilityNodeId::Witness {
        operation: OperationId::AnnounceMaturity,
        role: WitnessRole::SponsorAuthorization,
        availability: AvailabilityClass::SponsorLocal,
    };
    let nodes = vec![
        ConstructibilityNode {
            id: operation.clone(),
        },
        ConstructibilityNode {
            id: operator_witness.clone(),
        },
        ConstructibilityNode {
            id: sponsor.clone(),
        },
    ];
    let edges = vec![
        cedge(
            operator_witness,
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
        ObjectId::State,
        [
            RepresentationMode::Explicit,
            RepresentationMode::PublicCommitted,
        ],
        EXITS,
    )
}

fn disclosure_declarations() -> Vec<DisclosureNode> {
    let operation = OperationId::AnnounceMaturity;
    StateField::ALL
        .iter()
        .map(|field| FactId::StateField {
            operation,
            field: *field,
        })
        .chain([FactId::RequestedAnnouncementCycle { operation }])
        .chain(
            [
                AnnouncementLeadBound::Minimum,
                AnnouncementLeadBound::Maximum,
            ]
            .map(|bound| FactId::AnnouncementLead { operation, bound }),
        )
        .map(|id| DisclosureNode::Fact {
            id,
            initial_visibility: InitialVisibility::Public,
        })
        .collect()
}

fn relation_id(kind: RelationKind, subject: RelationSubject) -> RelationId {
    RelationId::new(OperationId::AnnounceMaturity, kind, subject)
}

/// One relation of the State object family.
fn state_family(kind: RelationKind, side: TransactionSide) -> RelationId {
    relation_id(
        kind,
        RelationSubject::ObjectFamily {
            side,
            object: ObjectId::State,
        },
    )
}

/// One relation of the optional sponsor object family.
fn sponsor_family(kind: RelationKind, side: TransactionSide) -> RelationId {
    relation_id(
        kind,
        RelationSubject::ObjectFamily {
            side,
            object: ObjectId::PlainLbtc,
        },
    )
}

fn lifecycle(exit: OperationId) -> RelationId {
    relation_id(
        RelationKind::Lifecycle,
        RelationSubject::LifecycleExit {
            object: ObjectId::State,
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
