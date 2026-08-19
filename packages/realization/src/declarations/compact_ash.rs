//! Target-independent compact-ASH declaration.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    Architecture, AssetId, BoundId, DeltaKind, ObjectId, OpenFlowKind, OperationId, ProjectionId,
    ProjectionRule, RootId, RootUse,
};

use crate::{
    AvailabilityClass, CardinalityMaximum, ConstructibilityDependencyDeclaration,
    ConstructibilityEdge, ConstructibilityEdgeRole, ConstructibilityNode, ConstructibilityNodeId,
    Count, DisclosureDependencyDeclaration, DisclosureEdge, DisclosureNode, DisclosureNodeId,
    DisclosureReason, DisclosureSeed, ExpectedCanonicalDelta, FactId, InitialVisibility,
    LifecycleDependencyDeclaration, LifecycleEdge, LifecycleNode, LifecycleNodeId,
    OperationRealization, ProofAlternativeId, ProofKind, RealizationError, Relation,
    RelationDeclaration, RelationDependencyDeclaration, RelationEdge, RelationId, RelationKind,
    RelationSubject, RepresentationMode, RequirementStrength, TransactionSide, WitnessRole,
    validate::validate_compact_ash_architecture,
};

#[allow(clippy::too_many_lines)]
pub fn derive(architecture: &Architecture) -> Result<OperationRealization, RealizationError> {
    validate_compact_ash_architecture(architecture)?;

    let ids = Ids::new();
    let relations = relation_declarations(&ids);
    let relation_dependencies = relation_dependencies(&ids);
    let (constructibility_nodes, constructibility_edges) = constructibility_declarations(&ids);
    let (lifecycle_nodes, lifecycle_edges) = lifecycle_declarations();
    let (disclosure_nodes, disclosure_edges, disclosure_seeds) = disclosure_declarations(&ids);

    Ok(OperationRealization {
        operation: OperationId::CompactAsh,
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
    conservation: RelationId,
    permissionless: RelationId,
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
    compact_lifecycle: RelationId,
    clear_lifecycle: RelationId,
}

impl Ids {
    fn new() -> Self {
        Self {
            input_cardinality: ash_family(RelationKind::Cardinality, TransactionSide::Input),
            output_cardinality: ash_family(RelationKind::Cardinality, TransactionSide::Output),
            sponsor_input_cardinality: sponsor_family(
                RelationKind::Cardinality,
                TransactionSide::Input,
            ),
            sponsor_output_cardinality: sponsor_family(
                RelationKind::Cardinality,
                TransactionSide::Output,
            ),
            input_recognition: ash_family(RelationKind::Recognition, TransactionSide::Input),
            output_recognition: ash_family(RelationKind::Recognition, TransactionSide::Output),
            sponsor_input_recognition: sponsor_family(
                RelationKind::Recognition,
                TransactionSide::Input,
            ),
            sponsor_output_recognition: sponsor_family(
                RelationKind::Recognition,
                TransactionSide::Output,
            ),
            conservation: relation_id(
                RelationKind::Conservation,
                RelationSubject::Asset { asset: AssetId::U },
            ),
            permissionless: relation_id(RelationKind::Authorization, RelationSubject::Operation),
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
                    object: ObjectId::Ash,
                },
            ),
            compact_lifecycle: lifecycle(OperationId::CompactAsh),
            clear_lifecycle: lifecycle(OperationId::Clear),
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
    let ash_objects = BTreeSet::from([ObjectId::Ash]);

    vec![
        declaration(
            ids.input_cardinality.clone(),
            Relation::Cardinality {
                side: crate::ObservedSide::Input,
                object: ObjectId::Ash,
                minimum: Count::new(2),
                maximum: CardinalityMaximum::Bound(BoundId::AshBatchMax),
            },
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.output_cardinality.clone(),
            Relation::Cardinality {
                side: crate::ObservedSide::Output,
                object: ObjectId::Ash,
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
        declaration(
            ids.input_recognition.clone(),
            Relation::Recognition {
                side: crate::ObservedSide::Input,
                object: ObjectId::Ash,
                asset: AssetId::U,
            },
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.output_recognition.clone(),
            Relation::Recognition {
                side: crate::ObservedSide::Output,
                object: ObjectId::Ash,
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
            ids.permissionless.clone(),
            Relation::PermissionlessAuthorization,
            [ProofKind::PublicConstructibility],
        ),
        declaration(
            ids.input_closure.clone(),
            Relation::AllowedObjectFamilies {
                side: crate::ObservedSide::Input,
                allowed: BTreeSet::from([ObjectId::Ash, ObjectId::PlainLbtc]),
            },
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.output_closure.clone(),
            Relation::AllowedObjectFamilies {
                side: crate::ObservedSide::Output,
                allowed: BTreeSet::from([ObjectId::Ash, ObjectId::PlainLbtc]),
            },
            [ProofKind::ManifestShape],
        ),
        declaration(
            ids.conservation.clone(),
            Relation::AmountConservation {
                asset: AssetId::U,
                input_objects: ash_objects.clone(),
                output_objects: ash_objects,
            },
            [ProofKind::PublicArithmetic],
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
                    kind: DeltaKind::OwnerlessLateral,
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
                class: crate::ConstructibilityClass::PublicPermissionless,
            },
            [ProofKind::PublicConstructibility],
        ),
        // A mode constraint, not a second arithmetic proof: the
        // conservation relation owns how value preservation is proved,
        // and an independent proof alternative here could contradict
        // the mode this relation admits.
        relation_only(
            ids.representation.clone(),
            Relation::Representation {
                object: ObjectId::Ash,
                allowed: BTreeSet::from([
                    RepresentationMode::Explicit,
                    RepresentationMode::PublicCommitted,
                ]),
            },
        ),
        relation_only(
            ids.compact_lifecycle.clone(),
            Relation::LifecycleExit {
                object: ObjectId::Ash,
                exit: OperationId::CompactAsh,
            },
        ),
        relation_only(
            ids.clear_lifecycle.clone(),
            Relation::LifecycleExit {
                object: ObjectId::Ash,
                exit: OperationId::Clear,
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
            &ids.permissionless,
            &ids.input_closure,
            RelationEdge::AuthorizationBeforeClosure,
        ),
        dep(
            &ids.permissionless,
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
            &ids.constructibility,
            &ids.representation,
            RelationEdge::StaticRequirement,
        ),
        dep(
            &ids.output_recognition,
            &ids.representation,
            RelationEdge::StaticRequirement,
        ),
        dep(
            &ids.representation,
            &ids.compact_lifecycle,
            RelationEdge::RepresentationBeforeLifecycle,
        ),
        dep(
            &ids.representation,
            &ids.clear_lifecycle,
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

fn constructibility_declarations(
    ids: &Ids,
) -> (
    Vec<ConstructibilityNode>,
    Vec<ConstructibilityDependencyDeclaration>,
) {
    let operation = ConstructibilityNodeId::Operation(OperationId::CompactAsh);
    let amount = fact_node(
        FactId::FamilyAmount {
            operation: OperationId::CompactAsh,
            side: TransactionSide::Input,
            object: ObjectId::Ash,
        },
        AvailabilityClass::Public,
    );
    let count = fact_node(
        FactId::FamilyCount {
            operation: OperationId::CompactAsh,
            side: TransactionSide::Input,
            object: ObjectId::Ash,
        },
        AvailabilityClass::Public,
    );
    let bound = fact_node(
        FactId::BoundValue {
            bound: BoundId::AshBatchMax,
        },
        AvailabilityClass::Public,
    );
    let sponsor = ConstructibilityNodeId::Witness {
        operation: OperationId::CompactAsh,
        role: WitnessRole::SponsorAuthorization,
        availability: AvailabilityClass::SponsorLocal,
    };
    let nodes = vec![
        ConstructibilityNode {
            id: operation.clone(),
        },
        ConstructibilityNode { id: amount.clone() },
        ConstructibilityNode { id: count.clone() },
        ConstructibilityNode { id: bound.clone() },
        ConstructibilityNode {
            id: sponsor.clone(),
        },
    ];
    let edges = vec![
        cedge(
            amount,
            operation.clone(),
            ConstructibilityEdgeRole::RequiredFact,
            RequirementStrength::Required,
        ),
        cedge(
            count,
            operation.clone(),
            ConstructibilityEdgeRole::RequiredFact,
            RequirementStrength::Required,
        ),
        cedge(
            bound,
            operation.clone(),
            ConstructibilityEdgeRole::RequiredFact,
            RequirementStrength::Required,
        ),
        cedge(
            sponsor,
            operation,
            ConstructibilityEdgeRole::SponsorOnly,
            RequirementStrength::Optional,
        ),
    ];

    let _ = ids;
    (nodes, edges)
}

fn lifecycle_declarations() -> (Vec<LifecycleNode>, Vec<LifecycleDependencyDeclaration>) {
    lifecycle_for(
        ObjectId::Ash,
        [
            RepresentationMode::Explicit,
            RepresentationMode::PublicCommitted,
        ],
        [OperationId::CompactAsh, OperationId::Clear],
    )
}

fn disclosure_declarations(
    ids: &Ids,
) -> (
    Vec<DisclosureNode>,
    Vec<DisclosureDependencyDeclaration>,
    Vec<DisclosureSeed>,
) {
    let input_amount = FactId::FamilyAmount {
        operation: OperationId::CompactAsh,
        side: TransactionSide::Input,
        object: ObjectId::Ash,
    };
    let output_amount = FactId::FamilyAmount {
        operation: OperationId::CompactAsh,
        side: TransactionSide::Output,
        object: ObjectId::Ash,
    };
    let input_count = FactId::FamilyCount {
        operation: OperationId::CompactAsh,
        side: TransactionSide::Input,
        object: ObjectId::Ash,
    };
    let bound = FactId::BoundValue {
        bound: BoundId::AshBatchMax,
    };
    let nodes = vec![
        disclosure_fact(input_amount.clone(), InitialVisibility::Public),
        disclosure_fact(output_amount.clone(), InitialVisibility::Public),
        disclosure_fact(input_count.clone(), InitialVisibility::Public),
        disclosure_fact(bound.clone(), InitialVisibility::Public),
        DisclosureNode::Relation {
            id: ids.constructibility.clone(),
        },
        DisclosureNode::Relation {
            id: ids.representation.clone(),
        },
    ];
    let edges = vec![
        dedge(
            DisclosureNodeId::Fact(input_amount),
            DisclosureNodeId::Relation(ids.constructibility.clone()),
            DisclosureEdge::ConstructibilityInput,
        ),
        dedge(
            DisclosureNodeId::Fact(input_count),
            DisclosureNodeId::Relation(ids.constructibility.clone()),
            DisclosureEdge::ConstructibilityInput,
        ),
        dedge(
            DisclosureNodeId::Fact(bound),
            DisclosureNodeId::Relation(ids.constructibility.clone()),
            DisclosureEdge::ConstructibilityInput,
        ),
        dedge(
            DisclosureNodeId::Fact(output_amount),
            DisclosureNodeId::Relation(ids.representation.clone()),
            DisclosureEdge::PublicObservableInput,
        ),
    ];
    let seeds = vec![
        DisclosureSeed {
            node: DisclosureNodeId::Relation(ids.constructibility.clone()),
            reason: DisclosureReason::PermissionlessConstructibility {
                operation: OperationId::CompactAsh,
                relation: ids.constructibility.clone(),
            },
        },
        DisclosureSeed {
            node: DisclosureNodeId::Relation(ids.representation.clone()),
            reason: DisclosureReason::PublicInterface,
        },
    ];

    (nodes, edges, seeds)
}

fn relation_id(kind: RelationKind, subject: RelationSubject) -> RelationId {
    RelationId::new(OperationId::CompactAsh, kind, subject)
}

/// One relation of the compacted ASH object family.
fn ash_family(kind: RelationKind, side: TransactionSide) -> RelationId {
    relation_id(
        kind,
        RelationSubject::ObjectFamily {
            side,
            object: ObjectId::Ash,
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
            object: ObjectId::Ash,
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

fn fact_node(fact: FactId, availability: AvailabilityClass) -> ConstructibilityNodeId {
    ConstructibilityNodeId::Fact {
        operation: OperationId::CompactAsh,
        fact,
        availability,
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
