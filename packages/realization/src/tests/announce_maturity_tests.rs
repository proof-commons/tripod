use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    ARCHITECTURE, Architecture, AssetId, BoundId, DeltaKind, MaxCount, ObjectId, OperationId,
    OperationSpec, ProjectionId, RootId,
};

use crate::{
    AnnouncementLeadBound, ArchitectureBinding, ArchitectureMismatchField, AvailabilityClass,
    CardinalityMaximum, ConstructibilityClass, ConstructibilityDependencyDeclaration,
    ConstructibilityEdge, ConstructibilityEdgeRole, ConstructibilityNodeId, Count, DisclosureNode,
    DisclosureReason, FactId, InitialVisibility, LifecycleDependencyDeclaration, LifecycleEdge,
    LifecycleNodeId, ObservedAsset, ObservedCanonicalFlow, ObservedCanonicalPartition,
    ObservedObject, ObservedObjectKind, ObservedObjectRef, ObservedRootEffect,
    ObservedRootEffectKind, ObservedSide, ObservedValue, OperationObservation,
    OperationRealization, OwnerId, ProofAlternativeId, ProofKind, ProtocolAmount, RealizationError,
    RealizationScope, Relation, RelationDeclaration, RelationDependencyDeclaration, RelationEdge,
    RelationId, RelationKind, RelationStatus, RelationSubject, RepresentationMode,
    RequirementStrength, ScopedRealizationSpec, StateField, TransactionSide, WitnessRole, derive,
    derive::assemble_scoped_realization,
};

pub(super) const OP: OperationId = OperationId::AnnounceMaturity;

pub(super) fn scope() -> RealizationScope {
    RealizationScope::from_operations([OP]).unwrap()
}

pub(super) fn realization() -> ScopedRealizationSpec {
    derive(&ARCHITECTURE, scope()).unwrap()
}

pub(super) fn rebuild(
    declaration: OperationRealization,
) -> Result<ScopedRealizationSpec, RealizationError> {
    assemble_scoped_realization(
        &ARCHITECTURE,
        ArchitectureBinding::from_architecture(&ARCHITECTURE).unwrap(),
        scope(),
        BTreeMap::from([(OP, declaration)]),
    )
}

pub(super) fn id(kind: RelationKind, subject: RelationSubject) -> RelationId {
    RelationId::new(OP, kind, subject)
}

fn family_id(kind: RelationKind, side: TransactionSide, object: ObjectId) -> RelationId {
    id(kind, RelationSubject::ObjectFamily { side, object })
}

fn expected_relation(
    kind: RelationKind,
    subject: RelationSubject,
    relation: Relation,
) -> RelationDeclaration {
    let id = id(kind, subject);
    let proofs = match kind {
        RelationKind::Representation | RelationKind::Lifecycle => BTreeSet::new(),
        RelationKind::SubstrateConservation => BTreeSet::from([ProofKind::SubstrateConservation]),
        _ => BTreeSet::from([ProofKind::ManifestShape]),
    };
    let proof_alternatives = proofs
        .into_iter()
        .map(|proof| ProofAlternativeId::new(id.clone(), proof))
        .collect();
    RelationDeclaration {
        id,
        relation,
        proof_alternatives,
    }
}

fn expected_families(row: &OperationSpec) -> Vec<RelationDeclaration> {
    let inputs = row
        .inputs
        .iter()
        .map(|v| (TransactionSide::Input, v.object, v.minimum, v.maximum));
    let outputs = row
        .outputs
        .iter()
        .map(|v| (TransactionSide::Output, v.object, v.minimum, v.maximum));
    let mut expected = Vec::new();
    for (side, object, minimum, maximum) in inputs.chain(outputs) {
        let subject = RelationSubject::ObjectFamily { side, object };
        let observed = match side {
            TransactionSide::Input => ObservedSide::Input,
            TransactionSide::Output => ObservedSide::Output,
        };
        expected.push(expected_relation(
            RelationKind::Cardinality,
            subject.clone(),
            Relation::Cardinality {
                side: observed,
                object,
                minimum: Count::new(u64::from(minimum)),
                maximum: match maximum {
                    MaxCount::Exact(n) => CardinalityMaximum::Exact(Count::new(u64::from(n))),
                    MaxCount::Bound(bound) => CardinalityMaximum::Bound(bound),
                },
            },
        ));
        expected.push(expected_relation(
            RelationKind::Recognition,
            subject,
            Relation::Recognition {
                side: observed,
                object,
                asset: ARCHITECTURE.object(object).unwrap().asset,
            },
        ));
    }
    for (side, allowed) in [
        (
            TransactionSide::Input,
            row.inputs.iter().map(|v| v.object).collect(),
        ),
        (
            TransactionSide::Output,
            row.outputs.iter().map(|v| v.object).collect(),
        ),
    ] {
        expected.push(expected_relation(
            RelationKind::AllowedObjectFamilies,
            RelationSubject::TransactionSide { side },
            Relation::AllowedObjectFamilies {
                side: match side {
                    TransactionSide::Input => ObservedSide::Input,
                    TransactionSide::Output => ObservedSide::Output,
                },
                allowed,
            },
        ));
    }
    expected
}

fn expected_policies(row: &OperationSpec) -> Vec<RelationDeclaration> {
    [
        (RelationKind::Authorization, Relation::OperatorAuthorization),
        (
            RelationKind::Constructibility,
            Relation::Constructibility {
                class: ConstructibilityClass::Operator,
            },
        ),
        (
            RelationKind::OpenFlowPolicy,
            Relation::OpenFlowPolicy {
                allowed: row.open_flows.iter().copied().collect(),
            },
        ),
        (
            RelationKind::CanonicalDeltaPolicy,
            Relation::CanonicalDeltaPolicy {
                expected: BTreeSet::new(),
            },
        ),
        (
            RelationKind::RootPolicy,
            Relation::RootPolicy {
                expected: RootId::ALL
                    .iter()
                    .map(|root| (*root, row.root_use(*root)))
                    .collect(),
            },
        ),
        (
            RelationKind::ProjectionPolicy,
            Relation::ProjectionPolicy {
                expected: ProjectionId::ALL
                    .iter()
                    .map(|projection| (*projection, row.projection_rule(*projection)))
                    .collect(),
            },
        ),
    ]
    .into_iter()
    .map(|(kind, body)| expected_relation(kind, RelationSubject::Operation, body))
    .collect()
}

fn expected_relations(row: &OperationSpec) -> BTreeMap<RelationId, RelationDeclaration> {
    assert_eq!(row.canonical_deltas, &[]);
    assert_eq!(row.authorization, architecture::PermissionClass::Operator);
    let mut expected = expected_families(row);
    expected.extend(expected_policies(row));
    expected.extend([
        expected_relation(
            RelationKind::SponsorIsolation,
            RelationSubject::Sponsor,
            Relation::SponsorIsolation,
        ),
        expected_relation(
            RelationKind::SponsorEnvelopeMultiplicity,
            RelationSubject::Sponsor,
            Relation::SponsorEnvelopeMultiplicity {
                maximum: Count::ONE,
            },
        ),
        expected_relation(
            RelationKind::SubstrateConservation,
            RelationSubject::Asset {
                asset: AssetId::Lbtc,
            },
            Relation::SubstrateConservation {
                asset: AssetId::Lbtc,
            },
        ),
        expected_relation(
            RelationKind::Representation,
            RelationSubject::Representation {
                object: ObjectId::State,
            },
            Relation::Representation {
                object: ObjectId::State,
                allowed: modes(),
            },
        ),
    ]);
    expected.extend(
        ARCHITECTURE
            .object(ObjectId::State)
            .unwrap()
            .mutators
            .iter()
            .map(|exit| {
                expected_relation(
                    RelationKind::Lifecycle,
                    RelationSubject::LifecycleExit {
                        object: ObjectId::State,
                        exit: *exit,
                    },
                    Relation::LifecycleExit {
                        object: ObjectId::State,
                        exit: *exit,
                    },
                )
            }),
    );
    expected.into_iter().map(|r| (r.id.clone(), r)).collect()
}

fn modes() -> BTreeSet<RepresentationMode> {
    BTreeSet::from([
        RepresentationMode::Explicit,
        RepresentationMode::PublicCommitted,
    ])
}

type ExpectedEdge = (RelationId, RelationId, RelationEdge);

fn expected_edges() -> BTreeSet<RelationDependencyDeclaration> {
    let representation = id(
        RelationKind::Representation,
        RelationSubject::Representation {
            object: ObjectId::State,
        },
    );
    expected_recognition_edges()
        .into_iter()
        .chain(expected_sponsor_edges())
        .chain(expected_constructibility_edges(&representation))
        .chain(expected_exit_edges(&representation))
        .map(
            |(prerequisite, dependent, edge)| RelationDependencyDeclaration {
                prerequisite,
                dependent,
                edge,
            },
        )
        .collect()
}

fn expected_recognition_edges() -> Vec<ExpectedEdge> {
    let mut edges = Vec::new();
    for side in [TransactionSide::Input, TransactionSide::Output] {
        for object in [ObjectId::State, ObjectId::PlainLbtc] {
            edges.push((
                family_id(RelationKind::Recognition, side, object),
                family_id(RelationKind::Cardinality, side, object),
                RelationEdge::RecognitionBeforeCardinality,
            ));
        }
    }
    edges
}

fn expected_sponsor_edges() -> Vec<ExpectedEdge> {
    let operation = |kind| id(kind, RelationSubject::Operation);
    let sponsor = id(RelationKind::SponsorIsolation, RelationSubject::Sponsor);
    let multiplicity = id(
        RelationKind::SponsorEnvelopeMultiplicity,
        RelationSubject::Sponsor,
    );
    let mut edges = Vec::new();
    for side in [TransactionSide::Input, TransactionSide::Output] {
        edges.push((
            family_id(RelationKind::Recognition, side, ObjectId::PlainLbtc),
            sponsor.clone(),
            RelationEdge::StaticRequirement,
        ));
    }
    edges.extend([
        (
            operation(RelationKind::OpenFlowPolicy),
            sponsor.clone(),
            RelationEdge::OpenFlowPolicyBeforeSponsor,
        ),
        (
            operation(RelationKind::OpenFlowPolicy),
            multiplicity.clone(),
            RelationEdge::StaticRequirement,
        ),
        (
            multiplicity,
            sponsor.clone(),
            RelationEdge::StaticRequirement,
        ),
        (
            sponsor.clone(),
            operation(RelationKind::Constructibility),
            RelationEdge::SponsorBeforeConstructibility,
        ),
        (
            sponsor,
            id(
                RelationKind::SubstrateConservation,
                RelationSubject::Asset {
                    asset: AssetId::Lbtc,
                },
            ),
            RelationEdge::SponsorBeforeOperation,
        ),
    ]);
    edges
}

fn expected_constructibility_edges(representation: &RelationId) -> Vec<ExpectedEdge> {
    let operation = |kind| id(kind, RelationSubject::Operation);
    vec![
        (
            operation(RelationKind::Authorization),
            id(
                RelationKind::AllowedObjectFamilies,
                RelationSubject::TransactionSide {
                    side: TransactionSide::Input,
                },
            ),
            RelationEdge::AuthorizationBeforeClosure,
        ),
        (
            operation(RelationKind::Authorization),
            operation(RelationKind::Constructibility),
            RelationEdge::AuthorizationBeforeConstructibility,
        ),
        (
            operation(RelationKind::Constructibility),
            representation.clone(),
            RelationEdge::StaticRequirement,
        ),
        (
            family_id(
                RelationKind::Recognition,
                TransactionSide::Output,
                ObjectId::State,
            ),
            representation.clone(),
            RelationEdge::StaticRequirement,
        ),
        (
            operation(RelationKind::RootPolicy),
            operation(RelationKind::Constructibility),
            RelationEdge::RootPolicyBeforeOperation,
        ),
        (
            operation(RelationKind::ProjectionPolicy),
            operation(RelationKind::Constructibility),
            RelationEdge::ProjectionPolicyBeforeOperation,
        ),
    ]
}

fn expected_exit_edges(
    representation: &RelationId,
) -> impl Iterator<Item = (RelationId, RelationId, RelationEdge)> + '_ {
    ARCHITECTURE
        .object(ObjectId::State)
        .unwrap()
        .mutators
        .iter()
        .map(|exit| {
            (
                representation.clone(),
                id(
                    RelationKind::Lifecycle,
                    RelationSubject::LifecycleExit {
                        object: ObjectId::State,
                        exit: *exit,
                    },
                ),
                RelationEdge::RepresentationBeforeLifecycle,
            )
        })
}

fn assert_constructibility(declaration: &OperationRealization) {
    let operation = ConstructibilityNodeId::Operation(OP);
    let witnesses = [
        (
            WitnessRole::OperatorAuthorization,
            AvailabilityClass::Operator,
            ConstructibilityEdgeRole::RequiredWitness,
            RequirementStrength::Required,
        ),
        (
            WitnessRole::SponsorAuthorization,
            AvailabilityClass::SponsorLocal,
            ConstructibilityEdgeRole::SponsorOnly,
            RequirementStrength::Optional,
        ),
    ];
    let mut nodes = BTreeSet::from([operation.clone()]);
    let mut edges = BTreeSet::new();
    for (role, availability, edge_role, strength) in witnesses {
        let source = ConstructibilityNodeId::Witness {
            operation: OP,
            role,
            availability,
        };
        nodes.insert(source.clone());
        edges.insert(ConstructibilityDependencyDeclaration {
            source,
            target: operation.clone(),
            edge: ConstructibilityEdge {
                role: edge_role,
                strength,
            },
        });
    }
    assert_eq!(
        declaration
            .constructibility_nodes
            .iter()
            .map(|n| n.id.clone())
            .collect::<BTreeSet<_>>(),
        nodes
    );
    assert_eq!(
        declaration
            .constructibility_edges
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>(),
        edges
    );
}

fn assert_lifecycle(declaration: &OperationRealization) {
    let mut nodes = BTreeSet::new();
    let mut edges = BTreeSet::new();
    for mode in modes() {
        let source = LifecycleNodeId::Representation {
            object: ObjectId::State,
            mode,
        };
        nodes.insert(source.clone());
        for exit in ARCHITECTURE.object(ObjectId::State).unwrap().mutators {
            let target = LifecycleNodeId::RequiredExit {
                object: ObjectId::State,
                operation: *exit,
            };
            nodes.insert(target.clone());
            edges.insert(LifecycleDependencyDeclaration {
                source: source.clone(),
                target,
                edge: LifecycleEdge::RequiresExit,
            });
        }
    }
    assert_eq!(
        declaration
            .lifecycle_nodes
            .iter()
            .map(|n| n.id.clone())
            .collect::<BTreeSet<_>>(),
        nodes
    );
    assert_eq!(
        declaration
            .lifecycle_edges
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>(),
        edges
    );
}

fn public_facts() -> BTreeMap<FactId, BTreeSet<DisclosureReason>> {
    let mut facts: BTreeMap<_, _> = StateField::ALL
        .iter()
        .map(|field| {
            (
                FactId::StateField {
                    operation: OP,
                    field: *field,
                },
                BTreeSet::from([DisclosureReason::PublicState]),
            )
        })
        .collect();
    facts.insert(
        FactId::RequestedAnnouncementCycle { operation: OP },
        BTreeSet::from([DisclosureReason::PublicRequest]),
    );
    for bound in [
        AnnouncementLeadBound::Minimum,
        AnnouncementLeadBound::Maximum,
    ] {
        facts.insert(
            FactId::AnnouncementLead {
                operation: OP,
                bound,
            },
            BTreeSet::from([DisclosureReason::PublicRequest]),
        );
    }
    facts
}

#[test]
fn derive_matches_architecture_spec() {
    let realization = realization();
    let declaration = &realization.operations[&OP];
    let row = ARCHITECTURE.operation(OP).unwrap();
    let actual: BTreeMap<_, _> = declaration
        .relations
        .iter()
        .map(|r| (r.id.clone(), r.clone()))
        .collect();
    assert_eq!(actual, expected_relations(row));
    assert!(
        !declaration
            .relations
            .iter()
            .any(|r| matches!(r.relation, Relation::AmountConservation { .. }))
    );
    assert_eq!(declaration.expressions.as_slice(), &[]);
    assert_eq!(
        declaration
            .relation_dependencies
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>(),
        expected_edges()
    );
    assert_constructibility(declaration);
    assert_lifecycle(declaration);
    assert_eq!(
        declaration
            .disclosure_nodes
            .iter()
            .map(DisclosureNode::id)
            .collect::<BTreeSet<_>>(),
        public_facts()
            .into_keys()
            .map(crate::DisclosureNodeId::Fact)
            .collect()
    );
    assert_eq!(declaration.disclosure_edges.as_slice(), &[]);
    assert_eq!(declaration.disclosure_seeds.as_slice(), &[]);
    assert_eq!(realization.relation_graph.node_count(), 26);
    assert_eq!(realization.relation_graph.edge_count(), 23);
    assert_eq!(realization.lifecycle_graph.node_count(), 8);
    assert_eq!(realization.lifecycle_graph.edge_count(), 12);
    assert_eq!(realization.disclosure_graph.node_count(), 9);
}

pub(super) fn assert_permutations(baseline: &ScopedRealizationSpec) {
    let reversals: [fn(&mut OperationRealization); 10] = [
        |d| d.expressions.reverse(),
        |d| d.relations.reverse(),
        |d| d.relation_dependencies.reverse(),
        |d| d.constructibility_nodes.reverse(),
        |d| d.constructibility_edges.reverse(),
        |d| d.lifecycle_nodes.reverse(),
        |d| d.lifecycle_edges.reverse(),
        |d| d.disclosure_nodes.reverse(),
        |d| d.disclosure_edges.reverse(),
        |d| d.disclosure_seeds.reverse(),
    ];
    for reverse in reversals {
        let mut operations = baseline.operations.clone();
        for declaration in operations.values_mut() {
            reverse(declaration);
        }
        let actual = assemble_scoped_realization(
            &ARCHITECTURE,
            ArchitectureBinding::from_architecture(&ARCHITECTURE).unwrap(),
            baseline.scope.clone(),
            operations,
        )
        .unwrap();
        assert_eq!(actual.project(), baseline.project());
    }
}

#[test]
fn declaration_is_permutation_stable() {
    assert_permutations(&realization());
}

pub(super) fn observation() -> OperationObservation {
    OperationObservation {
        operation: OP,
        objects: [ObservedSide::Input, ObservedSide::Output]
            .map(|side| ObservedObject {
                reference: ObservedObjectRef { side, ordinal: 0 },
                kind: ObservedObjectKind::Declared(ObjectId::State),
                asset: ObservedAsset::Declared(AssetId::Pid),
                value: ObservedValue::Protocol(ProtocolAmount::ONE),
                owner: None,
                representation: RepresentationMode::Explicit,
            })
            .to_vec(),
        protocol_signers: BTreeSet::new(),
        sponsor_signers: BTreeSet::new(),
        canonical_partition: ObservedCanonicalPartition::default(),
        open_flows: Vec::new(),
        root_effects: vec![ObservedRootEffect {
            root: RootId::State,
            effect: ObservedRootEffectKind::Succession,
        }],
        projections: BTreeSet::from([ProjectionId::TransitionCertificate]),
        bounds: BTreeMap::from([(BoundId::FeeSponsorInputMax, Count::new(16))]),
    }
}

pub(super) fn status(observation: &OperationObservation, relation: &RelationId) -> RelationStatus {
    realization()
        .evaluate_operation(observation)
        .unwrap()
        .verdict(relation)
        .unwrap()
        .status
        .clone()
}

pub(super) fn root_id() -> RelationId {
    id(RelationKind::RootPolicy, RelationSubject::Operation)
}

#[test]
fn foreign_root_participation_rejects() {
    for root in RootId::ALL.iter().filter(|root| **root != RootId::State) {
        let mut observed = observation();
        observed.root_effects.push(ObservedRootEffect {
            root: *root,
            effect: ObservedRootEffectKind::Succession,
        });
        assert_eq!(
            status(&observed, &root_id()),
            RelationStatus::Failed {
                reason: crate::RelationFailure::RootPolicy
            }
        );
    }
}

fn output_cardinality() -> RelationId {
    family_id(
        RelationKind::Cardinality,
        TransactionSide::Output,
        ObjectId::State,
    )
}

#[test]
fn two_state_outputs_reject() {
    let mut observed = observation();
    let mut extra = observed.objects[1].clone();
    extra.reference.ordinal = 1;
    observed.objects.push(extra);
    assert_eq!(
        status(&observed, &output_cardinality()),
        RelationStatus::Failed {
            reason: crate::RelationFailure::CardinalityAboveMaximum,
        }
    );
}

#[test]
fn zero_state_outputs_reject() {
    let mut observed = observation();
    observed
        .objects
        .retain(|object| object.reference.side != ObservedSide::Output);
    assert_eq!(
        observed.root_effects[0].effect,
        ObservedRootEffectKind::Succession
    );
    assert_eq!(
        status(&observed, &output_cardinality()),
        RelationStatus::Failed {
            reason: crate::RelationFailure::CardinalityBelowMinimum,
        }
    );
}

#[test]
fn operator_authorization_is_external_evidence() {
    for signers in [BTreeSet::new(), BTreeSet::from([OwnerId([1; 32])])] {
        let mut observed = observation();
        observed.protocol_signers = signers;
        let realization = realization();
        let report = realization.evaluate_operation(&observed).unwrap();

        let expected: BTreeMap<_, _> = realization.operations[&OP]
            .relations
            .iter()
            .map(|declaration| {
                let status = match &declaration.relation {
                    Relation::OperatorAuthorization
                    | Relation::Constructibility {
                        class: ConstructibilityClass::Operator,
                    } => RelationStatus::EvidenceRequired {
                        requirement: crate::ExternalEvidenceRequirement::OperatorAuthorization {
                            operation: OP,
                        },
                    },
                    Relation::SubstrateConservation { asset } => RelationStatus::EvidenceRequired {
                        requirement: crate::ExternalEvidenceRequirement::SubstrateConservation {
                            operation: OP,
                            asset: *asset,
                        },
                    },
                    Relation::LifecycleExit { .. } => RelationStatus::StaticallyValidated,
                    _ => RelationStatus::Passed,
                };
                (declaration.id.clone(), status)
            })
            .collect();
        let actual = report
            .verdicts
            .iter()
            .map(|verdict| (verdict.relation.clone(), verdict.status.clone()))
            .collect::<BTreeMap<_, _>>();

        assert_eq!(actual, expected);
        assert_eq!(report.verdicts.len(), 26);
        assert!(report.is_conformant());
        assert!(!report.is_evidence_complete());
        assert_eq!(
            report
                .verdict(&id(RelationKind::Authorization, RelationSubject::Operation))
                .unwrap()
                .status,
            RelationStatus::EvidenceRequired {
                requirement: crate::ExternalEvidenceRequirement::OperatorAuthorization {
                    operation: OP,
                },
            }
        );
        assert_eq!(
            report
                .verdict(&id(
                    RelationKind::Constructibility,
                    RelationSubject::Operation,
                ))
                .unwrap()
                .status,
            RelationStatus::EvidenceRequired {
                requirement: crate::ExternalEvidenceRequirement::OperatorAuthorization {
                    operation: OP,
                },
            }
        );
        assert_eq!(
            report
                .verdict(&id(
                    RelationKind::AllowedObjectFamilies,
                    RelationSubject::TransactionSide {
                        side: TransactionSide::Input,
                    },
                ))
                .unwrap()
                .status,
            RelationStatus::Passed,
        );
        assert_eq!(
            report
                .verdict(&id(
                    RelationKind::Representation,
                    RelationSubject::Representation {
                        object: ObjectId::State,
                    },
                ))
                .unwrap()
                .status,
            RelationStatus::Passed,
        );

        let exits = ARCHITECTURE.object(ObjectId::State).unwrap().mutators;
        assert_eq!(exits.len(), 6);
        for exit in exits {
            assert_eq!(
                report
                    .verdict(&id(
                        RelationKind::Lifecycle,
                        RelationSubject::LifecycleExit {
                            object: ObjectId::State,
                            exit: *exit,
                        },
                    ))
                    .unwrap()
                    .status,
                RelationStatus::StaticallyValidated,
            );
        }
        assert!(
            !report
                .verdicts
                .iter()
                .any(|verdict| matches!(verdict.status, RelationStatus::Blocked { .. }))
        );

        let evidence_multiset = report.required_external_evidence().cloned().fold(
            BTreeMap::new(),
            |mut counts, requirement| {
                *counts.entry(requirement).or_insert(0_usize) += 1;
                counts
            },
        );
        assert_eq!(
            evidence_multiset,
            BTreeMap::from([
                (
                    crate::ExternalEvidenceRequirement::OperatorAuthorization { operation: OP },
                    2,
                ),
                (
                    crate::ExternalEvidenceRequirement::SubstrateConservation {
                        operation: OP,
                        asset: AssetId::Lbtc,
                    },
                    1,
                ),
            ]),
        );
    }
}

#[test]
fn failed_input_recognition_still_blocks_input_cardinality() {
    let mut observed = observation();
    observed.objects[0].asset = ObservedAsset::Declared(AssetId::U);

    let report = realization().evaluate_operation(&observed).unwrap();
    let recognition = family_id(
        RelationKind::Recognition,
        TransactionSide::Input,
        ObjectId::State,
    );
    let cardinality = family_id(
        RelationKind::Cardinality,
        TransactionSide::Input,
        ObjectId::State,
    );

    assert_eq!(
        report.verdict(&recognition).unwrap().status,
        RelationStatus::Failed {
            reason: crate::RelationFailure::ObjectRecognition,
        },
    );
    assert_eq!(
        report.verdict(&cardinality).unwrap().status,
        RelationStatus::Blocked {
            prerequisites: vec![recognition],
        },
    );
}

fn compact_ash_pilot_observation() -> OperationObservation {
    let input0 = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 0,
    };
    let input1 = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 1,
    };
    let output0 = ObservedObjectRef {
        side: ObservedSide::Output,
        ordinal: 0,
    };
    let ash = |side, ordinal, value| ObservedObject {
        reference: ObservedObjectRef { side, ordinal },
        kind: ObservedObjectKind::Declared(ObjectId::Ash),
        asset: ObservedAsset::Declared(AssetId::U),
        value: ObservedValue::Protocol(ProtocolAmount::new(value).unwrap()),
        owner: None,
        representation: RepresentationMode::Explicit,
    };

    OperationObservation {
        operation: OperationId::CompactAsh,
        objects: vec![
            ash(ObservedSide::Input, 0, 40),
            ash(ObservedSide::Input, 1, 60),
            ash(ObservedSide::Output, 0, 100),
        ],
        protocol_signers: BTreeSet::new(),
        sponsor_signers: BTreeSet::new(),
        canonical_partition: ObservedCanonicalPartition {
            issuances: Vec::new(),
            flows: vec![ObservedCanonicalFlow {
                asset: AssetId::U,
                sources: vec![input0, input1],
                destinations: vec![output0],
                movement_kind: Some(DeltaKind::OwnerlessLateral),
                destructions: Vec::new(),
            }],
        },
        open_flows: Vec::new(),
        root_effects: Vec::new(),
        projections: BTreeSet::from([ProjectionId::TransitionCertificate]),
        bounds: BTreeMap::from([
            (BoundId::AshBatchMax, Count::new(64)),
            (BoundId::FeeSponsorInputMax, Count::new(16)),
        ]),
    }
}

fn live_transfer_pilot_observation() -> OperationObservation {
    let input0 = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 0,
    };
    let output0 = ObservedObjectRef {
        side: ObservedSide::Output,
        ordinal: 0,
    };
    let output1 = ObservedObjectRef {
        side: ObservedSide::Output,
        ordinal: 1,
    };
    let receipt = |side, ordinal, value, owner| ObservedObject {
        reference: ObservedObjectRef { side, ordinal },
        kind: ObservedObjectKind::Declared(ObjectId::ReceiptLive),
        asset: ObservedAsset::Declared(AssetId::U),
        value: ObservedValue::Protocol(ProtocolAmount::new(value).unwrap()),
        owner: Some(owner),
        representation: RepresentationMode::Explicit,
    };

    OperationObservation {
        operation: OperationId::TransferLive,
        objects: vec![
            receipt(ObservedSide::Input, 0, 100, OwnerId([1; 32])),
            receipt(ObservedSide::Output, 0, 40, OwnerId([2; 32])),
            receipt(ObservedSide::Output, 1, 60, OwnerId([3; 32])),
        ],
        protocol_signers: BTreeSet::from([OwnerId([1; 32])]),
        sponsor_signers: BTreeSet::new(),
        canonical_partition: ObservedCanonicalPartition {
            issuances: Vec::new(),
            flows: vec![ObservedCanonicalFlow {
                asset: AssetId::U,
                sources: vec![input0],
                destinations: vec![output0, output1],
                movement_kind: Some(DeltaKind::Lateral),
                destructions: Vec::new(),
            }],
        },
        open_flows: Vec::new(),
        root_effects: Vec::new(),
        projections: BTreeSet::from([ProjectionId::TransitionCertificate]),
        bounds: BTreeMap::from([
            (BoundId::TransferInputMax, Count::new(64)),
            (BoundId::TransferOutputMax, Count::new(64)),
            (BoundId::FeeSponsorInputMax, Count::new(16)),
        ]),
    }
}

#[test]
fn phase1_pilot_reports_remain_deterministic_and_unblocked() {
    let realization = derive(&ARCHITECTURE, RealizationScope::phase1_pilots()).unwrap();

    for observation in [
        compact_ash_pilot_observation(),
        live_transfer_pilot_observation(),
    ] {
        let baseline = realization.evaluate_operation(&observation).unwrap();
        let repeated = realization.evaluate_operation(&observation).unwrap();

        assert!(
            !baseline
                .verdicts
                .iter()
                .any(|verdict| matches!(verdict.status, RelationStatus::Blocked { .. }))
        );
        assert_eq!(repeated, baseline);
    }
}

#[test]
fn sponsor_reads_carry_no_amount() {
    let realization = realization();
    let is_amount = |fact: &FactId| {
        matches!(
            fact,
            FactId::FamilyAmount {
                object: ObjectId::PlainLbtc,
                ..
            }
        )
    };
    assert!(
        realization
            .expression_graph
            .node_weights()
            .all(|d| !matches!(&d.node, crate::ExpressionNode::Fact(f) if is_amount(f)))
    );
    assert!(
        realization
            .disclosure_graph
            .node_weights()
            .all(|n| !matches!(n, DisclosureNode::Fact { id, .. } if is_amount(id)))
    );
    assert!(
        realization
            .constructibility_node_by_id
            .keys()
            .all(|n| !matches!(n, ConstructibilityNodeId::Fact { fact, .. } if is_amount(fact)))
    );
    let analysis = realization.declassification();
    assert!(
        !analysis
            .required_public
            .keys()
            .chain(analysis.newly_disclosed.keys())
            .chain(analysis.retained_private.iter())
            .any(is_amount)
    );
}

pub(super) fn omit_exit(declaration: &mut OperationRealization, exit: OperationId) {
    let relation = id(
        RelationKind::Lifecycle,
        RelationSubject::LifecycleExit {
            object: ObjectId::State,
            exit,
        },
    );
    declaration.relations.retain(|r| r.id != relation);
    declaration
        .relation_dependencies
        .retain(|e| e.prerequisite != relation && e.dependent != relation);
    let node = LifecycleNodeId::RequiredExit {
        object: ObjectId::State,
        operation: exit,
    };
    declaration.lifecycle_nodes.retain(|n| n.id != node);
    declaration
        .lifecycle_edges
        .retain(|e| e.source != node && e.target != node);
}

#[test]
fn lifecycle_pin_refuses_a_coherent_omission() {
    for exit in ARCHITECTURE.object(ObjectId::State).unwrap().mutators {
        let mut declaration = realization().operations.remove(&OP).unwrap();
        omit_exit(&mut declaration, *exit);
        assert_eq!(
            rebuild(declaration).unwrap_err(),
            RealizationError::MissingLifecycleExitNode {
                object: ObjectId::State,
                exit: *exit
            }
        );
    }
}

#[test]
fn state_object_check_refuses_a_mutated_object() {
    let mutations: [fn(&mut architecture::ObjectSpec); 3] = [
        |o| o.mutators = &[],
        |o| o.deallocators = &[],
        |o| o.consensus_value_authoritative = false,
    ];
    for mutate in mutations {
        let mut architecture = ARCHITECTURE;
        let mut objects = architecture.objects.to_vec();
        mutate(
            objects
                .iter_mut()
                .find(|o| o.id == ObjectId::State)
                .unwrap(),
        );
        architecture.objects = objects.leak();
        weld_rejects(&architecture, ArchitectureMismatchField::StateObject);
    }
}

#[test]
fn public_facts_are_declared_and_public() {
    let realization = realization();
    assert_eq!(realization.declassification.required_public, public_facts());
    assert!(realization.declassification.newly_disclosed.is_empty());
    assert!(realization.declassification.retained_private.is_empty());
    assert_eq!(realization.operations[&OP].disclosure_seeds.as_slice(), &[]);
    for node in realization.disclosure_graph.node_weights() {
        assert!(matches!(
            node,
            DisclosureNode::Fact {
                initial_visibility: InitialVisibility::Public,
                ..
            }
        ));
    }
}

#[test]
fn explicit_scope_derives_the_announcement() {
    assert_eq!(realization().scope.operations(), &[OP]);
    assert_eq!(
        RealizationScope::phase1_pilots().operations(),
        &[OperationId::TransferLive, OperationId::CompactAsh]
    );
}

fn mutated(mutate: impl FnOnce(&mut OperationSpec)) -> Architecture {
    let mut architecture = ARCHITECTURE;
    let mut operations = architecture.operations.to_vec();
    mutate(operations.iter_mut().find(|o| o.id == OP).unwrap());
    architecture.operations = operations.leak();
    architecture
}

fn weld_rejects(architecture: &Architecture, field: ArchitectureMismatchField) {
    assert_eq!(
        crate::validate::validate_announce_maturity_architecture(architecture),
        Err(RealizationError::ArchitectureOperationMismatch {
            operation: OP,
            field
        })
    );
}

type Mutation = (ArchitectureMismatchField, fn(&mut OperationSpec));

fn shape_mutations() -> Vec<Mutation> {
    use ArchitectureMismatchField as F;
    vec![
        (F::OperationKind, |o| {
            o.kind = architecture::OperationKind::ClientProtocol;
        }),
        (F::Authorization, |o| {
            o.authorization = architecture::PermissionClass::Permissionless;
        }),
        (F::Issuances, |o| {
            o.issuances = vec![architecture::IssuanceSpec {
                asset: AssetId::U,
                authority: AssetId::U,
                condition: architecture::IssuanceCondition::PositiveAdmittedPrincipal,
            }]
            .leak();
        }),
        (F::Reads, |o| {
            o.reads = vec![architecture::QuantityId::Floor].leak();
        }),
        (F::Writes, |o| {
            o.writes = vec![architecture::QuantityId::Floor].leak();
        }),
        (F::InputFamilies, |o| o.inputs = &[]),
        (F::OutputFamilies, |o| o.outputs = &[]),
        (F::Bounds, |o| o.bounds = &[]),
        (F::OpenFlows, |o| o.open_flows = &[]),
        (F::ValueFlows, |o| o.value_flows = &[]),
        (F::RootPolicy, |o| o.roots = &[]),
        (F::CanonicalDeltas, |o| {
            o.canonical_deltas = ARCHITECTURE
                .operation(OperationId::TransferLive)
                .unwrap()
                .canonical_deltas;
        }),
        (F::DataOutputs, |o| {
            o.data_outputs = vec![architecture::DataOutputSpec {
                kind: architecture::DataOutputKind::Destruction,
                tag: architecture::TagId::Burn,
                asset: Some(AssetId::U),
                minimum: 1,
                maximum: MaxCount::Exact(1),
                condition: architecture::DeltaCondition::Always,
            }]
            .leak();
        }),
        (F::ProjectionPolicy, |o| o.projections = &[]),
        (F::Witnesses, |o| o.witnesses = &[]),
    ]
}

#[test]
fn architecture_validator_rejects_every_shape_field_mutation() {
    crate::validate::validate_announce_maturity_architecture(&ARCHITECTURE).unwrap();
    for (field, mutate) in shape_mutations() {
        weld_rejects(&mutated(mutate), field);
    }
}

#[test]
fn architecture_validator_rejects_every_input_component_mutation() {
    let mutations: [fn(&mut architecture::InputSpec); 3] = [
        |input| input.minimum += 1,
        |input| input.maximum = MaxCount::Exact(2),
        |input| input.authorization = architecture::InputAuthorization::Permissionless,
    ];
    for (object, field) in [
        (ObjectId::State, ArchitectureMismatchField::StateInput),
        (ObjectId::PlainLbtc, ArchitectureMismatchField::SponsorInput),
    ] {
        for mutate in mutations {
            let architecture = mutated(|operation| {
                let mut inputs = operation.inputs.to_vec();
                mutate(inputs.iter_mut().find(|i| i.object == object).unwrap());
                operation.inputs = inputs.leak();
            });
            weld_rejects(&architecture, field);
        }
    }
}

#[test]
fn architecture_validator_rejects_every_output_component_mutation() {
    let mutations: [fn(&mut architecture::OutputSpec); 2] = [
        |output| output.minimum += 1,
        |output| output.maximum = MaxCount::Exact(2),
    ];
    for (object, field) in [
        (ObjectId::State, ArchitectureMismatchField::StateOutput),
        (
            ObjectId::PlainLbtc,
            ArchitectureMismatchField::SponsorOutput,
        ),
    ] {
        for mutate in mutations {
            let architecture = mutated(|operation| {
                let mut outputs = operation.outputs.to_vec();
                mutate(outputs.iter_mut().find(|o| o.object == object).unwrap());
                operation.outputs = outputs.leak();
            });
            weld_rejects(&architecture, field);
        }
    }
}

#[test]
fn architecture_validator_checks_every_root_entry() {
    for root in RootId::ALL {
        let architecture = mutated(|operation| {
            let mut roots = operation.roots.to_vec();
            roots.retain(|r| r.root != *root);
            roots.push(architecture::RootUseSpec {
                root: *root,
                use_kind: if *root == RootId::State {
                    architecture::RootUse::Forbidden
                } else {
                    architecture::RootUse::Succession
                },
            });
            operation.roots = roots.leak();
        });
        weld_rejects(&architecture, ArchitectureMismatchField::RootPolicy);
    }
}

#[test]
fn architecture_validator_checks_every_projection_entry() {
    for projection in ProjectionId::ALL {
        let architecture = mutated(|operation| {
            let mut projections = operation.projections.to_vec();
            projections.retain(|p| p.projection != *projection);
            projections.push(architecture::ProjectionSpec {
                projection: *projection,
                rule: if *projection == ProjectionId::TransitionCertificate {
                    architecture::ProjectionRule::Forbidden
                } else {
                    architecture::ProjectionRule::Required
                },
            });
            operation.projections = projections.leak();
        });
        weld_rejects(&architecture, ArchitectureMismatchField::ProjectionPolicy);
    }
}

#[test]
fn architecture_validator_reports_the_missing_operation() {
    let mut architecture = ARCHITECTURE;
    architecture.operations = architecture
        .operations
        .iter()
        .copied()
        .filter(|o| o.id != OP)
        .collect::<Vec<_>>()
        .leak();
    assert_eq!(
        crate::validate::validate_announce_maturity_architecture(&architecture),
        Err(RealizationError::MissingArchitectureOperation(OP))
    );
}
