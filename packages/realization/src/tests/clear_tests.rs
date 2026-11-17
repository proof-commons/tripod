use std::collections::{BTreeMap, BTreeSet};

use architecture::{ARCHITECTURE, AssetId, BoundId, ObjectId, OperationId, TagId};

use crate::*;

fn realized() -> ScopedRealizationSpec {
    derive(
        &ARCHITECTURE,
        RealizationScope::from_operations([OperationId::Clear]).unwrap(),
    )
    .unwrap()
}

fn id(kind: RelationKind, subject: RelationSubject) -> RelationId {
    RelationId::new(OperationId::Clear, kind, subject)
}

fn op(kind: RelationKind) -> RelationId {
    id(kind, RelationSubject::Operation)
}

fn mismatch(field: ArchitectureMismatchField) -> RealizationError {
    RealizationError::ArchitectureOperationMismatch {
        operation: OperationId::Clear,
        field,
    }
}

fn census_mismatch(kind: RelationKind) -> RealizationError {
    RealizationError::ArchitectureRelationMismatch {
        relation: id(
            kind,
            if kind == RelationKind::ClearAssignment {
                RelationSubject::StateField {
                    field: StateField::Omega,
                }
            } else if kind == RelationKind::Representation {
                RelationSubject::Representation {
                    object: ObjectId::Ash,
                }
            } else {
                RelationSubject::Operation
            },
        ),
    }
}

fn control(
    b: u64,
    yl: u64,
    yt: u64,
    x: u64,
    next_yl: u64,
    residual: u64,
) -> (OperationObservation, BurnClearObservation) {
    let (mut common, mut extension) =
        super::burn_clear_relation_tests::clear(b, yl, yt, x, next_yl, residual);
    common
        .bounds
        .insert(BoundId::FeeSponsorInputMax, Count::new(4));
    extension.context =
        BurnClearContext::bind_observation(common.clone(), [1; 32], [2; 32]).unwrap();
    (common, extension)
}

fn bind(
    common: &OperationObservation,
    extension: &BurnClearObservation,
) -> BoundBurnClearObservation {
    let mut extension = extension.clone();
    extension.context =
        BurnClearContext::bind_observation(common.clone(), [1; 32], [2; 32]).unwrap();
    validate_burn_clear_observation(common, &extension.context, Some(&extension)).unwrap()
}

fn assert_row(
    shell: &OperationRealization,
    identity: RelationId,
    body: Relation,
    proof: Option<ProofKind>,
) {
    let actual = shell
        .relations
        .iter()
        .find(|row| row.id == identity)
        .unwrap();
    let expected = proof.map_or_else(BTreeSet::new, |kind| {
        BTreeSet::from([ProofAlternativeId::new(identity.clone(), kind)])
    });
    assert_eq!(
        actual,
        &RelationDeclaration {
            id: identity,
            relation: body,
            proof_alternatives: expected,
        }
    );
}

#[test]
#[allow(clippy::too_many_lines)] // Keep the literal complete-shell comparison in one focused test.
fn clear_shell_matches_plan_literals() {
    use RelationEdge as E;
    use RelationKind as K;
    use TransactionSide::{Input, Output};
    let shell = crate::declarations::clear::declaration();
    assert_eq!(shell.operation, OperationId::Clear);
    assert_eq!(shell.expressions.len(), 17);
    let mut expected_expressions = Vec::new();
    for field in StateField::ALL {
        for side in [Input, Output] {
            let fact = FactId::StateField {
                operation: OperationId::Clear,
                side,
                field: *field,
            };
            expected_expressions.push(ExpressionDeclaration {
                id: ExprId::fact(fact.clone()),
                ty: fact.semantic_type(),
                node: ExpressionNode::Fact(fact),
            });
        }
        if *field != StateField::YL {
            let relation = id(
                K::ClearAssignment,
                RelationSubject::StateField { field: *field },
            );
            expected_expressions.push(ExpressionDeclaration {
                id: ExprId::relation(relation, ExpressionRole::Predicate),
                ty: SemanticType::Bool,
                node: ExpressionNode::Equal {
                    left: ExprId::fact(FactId::StateField {
                        operation: OperationId::Clear,
                        side: Input,
                        field: *field,
                    }),
                    right: ExprId::fact(FactId::StateField {
                        operation: OperationId::Clear,
                        side: Output,
                        field: *field,
                    }),
                },
            });
        }
    }
    let mut actual_expressions = shell.expressions.clone();
    actual_expressions.sort_by_key(|row| row.id.clone());
    expected_expressions.sort_by_key(|row| row.id.clone());
    assert_eq!(actual_expressions, expected_expressions);
    let ids = shell
        .relations
        .iter()
        .map(|row| row.id.clone())
        .collect::<BTreeSet<_>>();
    let mut expected = BTreeSet::new();
    for side in [Input, Output] {
        for object in [ObjectId::State, ObjectId::Ash, ObjectId::PlainLbtc] {
            for kind in [K::Cardinality, K::Recognition] {
                expected.insert(id(kind, RelationSubject::ObjectFamily { side, object }));
            }
        }
        expected.insert(id(
            K::AllowedObjectFamilies,
            RelationSubject::TransactionSide { side },
        ));
    }
    for kind in [
        K::Authorization,
        K::CanonicalDeltaPolicy,
        K::OpenFlowPolicy,
        K::RootPolicy,
        K::ProjectionPolicy,
        K::Constructibility,
        K::DataOutputs,
        K::ClearClamp,
        K::ClearFloor,
        K::ClearFlow,
        K::ClearBinding,
    ] {
        expected.insert(op(kind));
    }
    for kind in [K::SponsorIsolation, K::SponsorEnvelopeMultiplicity] {
        expected.insert(id(kind, RelationSubject::Sponsor));
    }
    expected.insert(id(
        K::SubstrateConservation,
        RelationSubject::Asset {
            asset: AssetId::Lbtc,
        },
    ));
    for object in [ObjectId::State, ObjectId::Ash] {
        expected.insert(id(
            K::Representation,
            RelationSubject::Representation { object },
        ));
    }
    for field in StateField::ALL {
        expected.insert(id(
            K::ClearAssignment,
            RelationSubject::StateField { field: *field },
        ));
    }
    for object in [ObjectId::State, ObjectId::Ash] {
        let exits: &[OperationId] = if object == ObjectId::State {
            &[
                OperationId::AdmitDeposits,
                OperationId::Cycle,
                OperationId::Redeem,
                OperationId::ReceiptRelabel,
                OperationId::Clear,
                OperationId::AnnounceMaturity,
            ]
        } else {
            &[OperationId::CompactAsh, OperationId::Clear]
        };
        for exit in exits {
            expected.insert(id(
                K::Lifecycle,
                RelationSubject::LifecycleExit {
                    object,
                    exit: *exit,
                },
            ));
        }
    }
    assert_eq!(ids, expected);
    assert_eq!(shell.relations.len(), expected.len());
    for (side, object, minimum, maximum, asset) in [
        (
            Input,
            ObjectId::State,
            Count::ONE,
            CardinalityMaximum::Exact(Count::ONE),
            AssetId::Pid,
        ),
        (
            Output,
            ObjectId::State,
            Count::ONE,
            CardinalityMaximum::Exact(Count::ONE),
            AssetId::Pid,
        ),
        (
            Input,
            ObjectId::Ash,
            Count::ONE,
            CardinalityMaximum::Bound(BoundId::AshBatchMax),
            AssetId::U,
        ),
        (
            Output,
            ObjectId::Ash,
            Count::ZERO,
            CardinalityMaximum::Exact(Count::ONE),
            AssetId::U,
        ),
        (
            Input,
            ObjectId::PlainLbtc,
            Count::ZERO,
            CardinalityMaximum::Bound(BoundId::FeeSponsorInputMax),
            AssetId::Lbtc,
        ),
        (
            Output,
            ObjectId::PlainLbtc,
            Count::ZERO,
            CardinalityMaximum::Exact(Count::ONE),
            AssetId::Lbtc,
        ),
    ] {
        let observed = if side == Input {
            ObservedSide::Input
        } else {
            ObservedSide::Output
        };
        assert_row(
            &shell,
            id(
                K::Cardinality,
                RelationSubject::ObjectFamily { side, object },
            ),
            Relation::Cardinality {
                side: observed,
                object,
                minimum,
                maximum,
            },
            Some(ProofKind::ManifestShape),
        );
        assert_row(
            &shell,
            id(
                K::Recognition,
                RelationSubject::ObjectFamily { side, object },
            ),
            Relation::Recognition {
                side: observed,
                object,
                asset,
            },
            Some(ProofKind::ManifestShape),
        );
    }
    for side in [Input, Output] {
        assert_row(
            &shell,
            id(
                K::AllowedObjectFamilies,
                RelationSubject::TransactionSide { side },
            ),
            Relation::AllowedObjectFamilies {
                side: if side == Input {
                    ObservedSide::Input
                } else {
                    ObservedSide::Output
                },
                allowed: BTreeSet::from([ObjectId::State, ObjectId::Ash, ObjectId::PlainLbtc]),
            },
            Some(ProofKind::ManifestShape),
        );
    }
    assert_row(
        &shell,
        op(K::Authorization),
        Relation::PermissionlessAuthorization,
        Some(ProofKind::PublicConstructibility),
    );
    assert_row(
        &shell,
        op(K::CanonicalDeltaPolicy),
        Relation::ClearCanonicalDeltaPolicy,
        Some(ProofKind::ManifestShape),
    );
    assert_row(
        &shell,
        id(K::SponsorIsolation, RelationSubject::Sponsor),
        Relation::SponsorIsolation,
        Some(ProofKind::ManifestShape),
    );
    assert_row(
        &shell,
        id(K::SponsorEnvelopeMultiplicity, RelationSubject::Sponsor),
        Relation::SponsorEnvelopeMultiplicity {
            maximum: Count::ONE,
        },
        Some(ProofKind::ManifestShape),
    );
    assert_row(
        &shell,
        id(
            K::SubstrateConservation,
            RelationSubject::Asset {
                asset: AssetId::Lbtc,
            },
        ),
        Relation::SubstrateConservation {
            asset: AssetId::Lbtc,
        },
        Some(ProofKind::SubstrateConservation),
    );
    assert_row(
        &shell,
        op(K::OpenFlowPolicy),
        Relation::OpenFlowPolicy {
            allowed: BTreeSet::from([architecture::OpenFlowKind::FeeSponsor]),
        },
        Some(ProofKind::ManifestShape),
    );
    assert_row(
        &shell,
        op(K::RootPolicy),
        Relation::RootPolicy {
            expected: architecture::RootId::ALL
                .iter()
                .map(|root| {
                    (
                        *root,
                        if *root == architecture::RootId::State {
                            architecture::RootUse::Succession
                        } else {
                            architecture::RootUse::Forbidden
                        },
                    )
                })
                .collect::<BTreeMap<_, _>>(),
        },
        Some(ProofKind::ManifestShape),
    );
    assert_row(
        &shell,
        op(K::ProjectionPolicy),
        Relation::ProjectionPolicy {
            expected: architecture::ProjectionId::ALL
                .iter()
                .map(|projection| {
                    (
                        *projection,
                        if matches!(
                            projection,
                            architecture::ProjectionId::TransitionCertificate
                                | architecture::ProjectionId::ClearEvent
                        ) {
                            architecture::ProjectionRule::Required
                        } else {
                            architecture::ProjectionRule::Forbidden
                        },
                    )
                })
                .collect::<BTreeMap<_, _>>(),
        },
        Some(ProofKind::ManifestShape),
    );
    assert_row(
        &shell,
        op(K::Constructibility),
        Relation::Constructibility {
            class: ConstructibilityClass::PublicPermissionless,
        },
        Some(ProofKind::PublicConstructibility),
    );
    assert_row(
        &shell,
        op(K::DataOutputs),
        Relation::DataOutputs {
            expected: vec![architecture::DataOutputSpec {
                kind: architecture::DataOutputKind::Destruction,
                tag: TagId::Recon,
                asset: Some(AssetId::U),
                minimum: 1,
                maximum: architecture::MaxCount::Exact(1),
                condition: architecture::DeltaCondition::Always,
            }],
        },
        Some(ProofKind::ManifestShape),
    );
    for (kind, body, proof) in [
        (
            K::ClearClamp,
            Relation::ClearClamp,
            ProofKind::PublicArithmetic,
        ),
        (
            K::ClearFloor,
            Relation::ClearFloor,
            ProofKind::PublicArithmetic,
        ),
        (
            K::ClearFlow,
            Relation::ClearFlow,
            ProofKind::PublicArithmetic,
        ),
        (
            K::ClearBinding,
            Relation::ClearBinding,
            ProofKind::ManifestShape,
        ),
    ] {
        assert_row(&shell, op(kind), body, Some(proof));
    }
    for field in StateField::ALL {
        let identity = id(
            K::ClearAssignment,
            RelationSubject::StateField { field: *field },
        );
        assert_row(
            &shell,
            identity.clone(),
            Relation::ClearAssignment {
                field: *field,
                expression: (*field != StateField::YL)
                    .then(|| ExprId::relation(identity, ExpressionRole::Predicate)),
            },
            Some(ProofKind::PublicArithmetic),
        );
    }
    for object in [ObjectId::Ash, ObjectId::State] {
        assert_row(
            &shell,
            id(
                K::Representation,
                RelationSubject::Representation { object },
            ),
            Relation::Representation {
                object,
                allowed: BTreeSet::from([
                    RepresentationMode::Explicit,
                    RepresentationMode::PublicCommitted,
                ]),
            },
            None,
        );
        let exits: &[OperationId] = if object == ObjectId::Ash {
            &[OperationId::CompactAsh, OperationId::Clear]
        } else {
            &[
                OperationId::AdmitDeposits,
                OperationId::Cycle,
                OperationId::Redeem,
                OperationId::ReceiptRelabel,
                OperationId::Clear,
                OperationId::AnnounceMaturity,
            ]
        };
        for exit in exits {
            assert_row(
                &shell,
                id(
                    K::Lifecycle,
                    RelationSubject::LifecycleExit {
                        object,
                        exit: *exit,
                    },
                ),
                Relation::LifecycleExit {
                    object,
                    exit: *exit,
                },
                None,
            );
        }
    }
    let dep = |prerequisite, dependent, edge| RelationDependencyDeclaration {
        prerequisite,
        dependent,
        edge,
    };
    let mut dependencies = Vec::new();
    let clamp = op(K::ClearClamp);
    let floor = op(K::ClearFloor);
    let binding = op(K::ClearBinding);
    for target in [
        op(K::CanonicalDeltaPolicy),
        op(K::ClearFlow),
        op(K::DataOutputs),
    ] {
        dependencies.push(dep(clamp.clone(), target, E::StaticRequirement));
    }
    for field in StateField::ALL {
        let assignment = id(
            K::ClearAssignment,
            RelationSubject::StateField { field: *field },
        );
        dependencies.push(dep(clamp.clone(), assignment.clone(), E::StaticRequirement));
        dependencies.push(dep(assignment, floor.clone(), E::StaticRequirement));
    }
    for source in [floor, op(K::ClearFlow), op(K::DataOutputs)] {
        dependencies.push(dep(source, binding.clone(), E::StaticRequirement));
    }
    for side in [Input, Output] {
        for object in [ObjectId::State, ObjectId::Ash, ObjectId::PlainLbtc] {
            let recognition = id(
                K::Recognition,
                RelationSubject::ObjectFamily { side, object },
            );
            let cardinality = id(
                K::Cardinality,
                RelationSubject::ObjectFamily { side, object },
            );
            dependencies.push(dep(
                recognition.clone(),
                cardinality.clone(),
                E::RecognitionBeforeCardinality,
            ));
            if object == ObjectId::PlainLbtc {
                dependencies.push(dep(
                    recognition,
                    id(K::SponsorIsolation, RelationSubject::Sponsor),
                    E::StaticRequirement,
                ));
            } else if side == Input {
                dependencies.push(dep(recognition, clamp.clone(), E::StaticRequirement));
                dependencies.push(dep(cardinality, clamp.clone(), E::StaticRequirement));
            } else {
                dependencies.push(dep(
                    recognition.clone(),
                    binding.clone(),
                    E::StaticRequirement,
                ));
                dependencies.push(dep(cardinality, binding.clone(), E::StaticRequirement));
                dependencies.push(dep(
                    recognition,
                    id(
                        K::Representation,
                        RelationSubject::Representation { object },
                    ),
                    E::StaticRequirement,
                ));
            }
        }
    }
    dependencies.extend([
        dep(
            op(K::Authorization),
            id(
                K::AllowedObjectFamilies,
                RelationSubject::TransactionSide { side: Input },
            ),
            E::AuthorizationBeforeClosure,
        ),
        dep(
            op(K::Authorization),
            op(K::Constructibility),
            E::AuthorizationBeforeConstructibility,
        ),
        dep(
            op(K::OpenFlowPolicy),
            id(K::SponsorIsolation, RelationSubject::Sponsor),
            E::OpenFlowPolicyBeforeSponsor,
        ),
        dep(
            op(K::OpenFlowPolicy),
            id(K::SponsorEnvelopeMultiplicity, RelationSubject::Sponsor),
            E::StaticRequirement,
        ),
        dep(
            id(K::SponsorEnvelopeMultiplicity, RelationSubject::Sponsor),
            id(K::SponsorIsolation, RelationSubject::Sponsor),
            E::StaticRequirement,
        ),
        dep(
            id(K::SponsorIsolation, RelationSubject::Sponsor),
            op(K::Constructibility),
            E::SponsorBeforeConstructibility,
        ),
        dep(
            id(K::SponsorIsolation, RelationSubject::Sponsor),
            id(
                K::SubstrateConservation,
                RelationSubject::Asset {
                    asset: AssetId::Lbtc,
                },
            ),
            E::SponsorBeforeOperation,
        ),
        dep(
            op(K::RootPolicy),
            op(K::Constructibility),
            E::RootPolicyBeforeOperation,
        ),
        dep(
            op(K::ProjectionPolicy),
            op(K::Constructibility),
            E::ProjectionPolicyBeforeOperation,
        ),
    ]);
    for object in [ObjectId::Ash, ObjectId::State] {
        let representation = id(
            K::Representation,
            RelationSubject::Representation { object },
        );
        dependencies.push(dep(
            op(K::Constructibility),
            representation.clone(),
            E::StaticRequirement,
        ));
        let exits: &[OperationId] = if object == ObjectId::Ash {
            &[OperationId::CompactAsh, OperationId::Clear]
        } else {
            &[
                OperationId::AdmitDeposits,
                OperationId::Cycle,
                OperationId::Redeem,
                OperationId::ReceiptRelabel,
                OperationId::Clear,
                OperationId::AnnounceMaturity,
            ]
        };
        for exit in exits {
            dependencies.push(dep(
                representation.clone(),
                id(
                    K::Lifecycle,
                    RelationSubject::LifecycleExit {
                        object,
                        exit: *exit,
                    },
                ),
                E::RepresentationBeforeLifecycle,
            ));
        }
    }
    dependencies.sort();
    let mut actual_dependencies = shell.relation_dependencies.clone();
    actual_dependencies.sort();
    assert_eq!(actual_dependencies, dependencies);
    let mut facts = Vec::new();
    for (side, object) in [
        (Input, ObjectId::State),
        (Output, ObjectId::State),
        (Input, ObjectId::Ash),
        (Output, ObjectId::Ash),
    ] {
        facts.push(FactId::FamilyCount {
            operation: OperationId::Clear,
            side,
            object,
        });
    }
    facts.push(FactId::FamilyAmount {
        operation: OperationId::Clear,
        side: Input,
        object: ObjectId::Ash,
    });
    for bound in [BoundId::AshBatchMax, BoundId::FeeSponsorInputMax] {
        facts.push(FactId::BoundValue { bound });
    }
    for field in StateField::ALL {
        facts.push(FactId::StateField {
            operation: OperationId::Clear,
            side: Input,
            field: *field,
        });
    }
    let operation_node = ConstructibilityNodeId::Operation(OperationId::Clear);
    let mut construction_nodes = BTreeSet::from([operation_node.clone()]);
    let mut construction_edges = BTreeSet::new();
    for fact in &facts {
        let source = ConstructibilityNodeId::Fact {
            operation: OperationId::Clear,
            fact: fact.clone(),
            availability: AvailabilityClass::Public,
        };
        construction_nodes.insert(source.clone());
        construction_edges.insert(ConstructibilityDependencyDeclaration {
            source,
            target: operation_node.clone(),
            edge: ConstructibilityEdge {
                role: ConstructibilityEdgeRole::RequiredFact,
                strength: RequirementStrength::Required,
            },
        });
    }
    let sponsor = ConstructibilityNodeId::Witness {
        operation: OperationId::Clear,
        role: WitnessRole::SponsorAuthorization,
        availability: AvailabilityClass::SponsorLocal,
    };
    construction_nodes.insert(sponsor.clone());
    construction_edges.insert(ConstructibilityDependencyDeclaration {
        source: sponsor,
        target: operation_node,
        edge: ConstructibilityEdge {
            role: ConstructibilityEdgeRole::SponsorOnly,
            strength: RequirementStrength::Optional,
        },
    });
    assert_eq!(
        shell
            .constructibility_nodes
            .iter()
            .map(|node| node.id.clone())
            .collect::<BTreeSet<_>>(),
        construction_nodes
    );
    assert_eq!(
        shell
            .constructibility_edges
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>(),
        construction_edges
    );
    let mut lifecycle_nodes = BTreeSet::new();
    let mut lifecycle_edges = BTreeSet::new();
    for object in [ObjectId::Ash, ObjectId::State] {
        let exits: &[OperationId] = if object == ObjectId::Ash {
            &[OperationId::CompactAsh, OperationId::Clear]
        } else {
            &[
                OperationId::AdmitDeposits,
                OperationId::Cycle,
                OperationId::Redeem,
                OperationId::ReceiptRelabel,
                OperationId::Clear,
                OperationId::AnnounceMaturity,
            ]
        };
        for mode in [
            RepresentationMode::Explicit,
            RepresentationMode::PublicCommitted,
        ] {
            let source = LifecycleNodeId::Representation { object, mode };
            lifecycle_nodes.insert(source.clone());
            for exit in exits {
                let target = LifecycleNodeId::RequiredExit {
                    object,
                    operation: *exit,
                };
                lifecycle_nodes.insert(target.clone());
                lifecycle_edges.insert(LifecycleDependencyDeclaration {
                    source: source.clone(),
                    target,
                    edge: LifecycleEdge::RequiresExit,
                });
            }
        }
    }
    assert_eq!(
        shell
            .lifecycle_nodes
            .iter()
            .map(|node| node.id.clone())
            .collect::<BTreeSet<_>>(),
        lifecycle_nodes
    );
    assert_eq!(
        shell
            .lifecycle_edges
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>(),
        lifecycle_edges
    );
    let construction_relation = op(K::Constructibility);
    let ash_relation = id(
        K::Representation,
        RelationSubject::Representation {
            object: ObjectId::Ash,
        },
    );
    let state_relation = id(
        K::Representation,
        RelationSubject::Representation {
            object: ObjectId::State,
        },
    );
    let binding_relation = op(K::ClearBinding);
    let mut disclosure_nodes = Vec::new();
    let mut disclosure_edges = Vec::new();
    for relation in [
        construction_relation.clone(),
        ash_relation.clone(),
        state_relation.clone(),
        clamp.clone(),
        op(K::ClearFloor),
        op(K::ClearFlow),
        binding_relation.clone(),
    ] {
        disclosure_nodes.push(DisclosureNode::Relation { id: relation });
    }
    for field in StateField::ALL {
        disclosure_nodes.push(DisclosureNode::Relation {
            id: id(
                K::ClearAssignment,
                RelationSubject::StateField { field: *field },
            ),
        });
    }
    for fact in &facts {
        disclosure_nodes.push(DisclosureNode::Fact {
            id: fact.clone(),
            initial_visibility: InitialVisibility::Public,
        });
        disclosure_edges.push(DisclosureDependencyDeclaration {
            source: DisclosureNodeId::Fact(fact.clone()),
            target: DisclosureNodeId::Relation(construction_relation.clone()),
            edge: DisclosureEdge::ConstructibilityInput,
        });
        if let FactId::FamilyAmount {
            object: ObjectId::Ash,
            ..
        } = fact
        {
            disclosure_edges.push(DisclosureDependencyDeclaration {
                source: DisclosureNodeId::Fact(fact.clone()),
                target: DisclosureNodeId::Relation(clamp.clone()),
                edge: DisclosureEdge::RelationOperand,
            });
            disclosure_edges.push(DisclosureDependencyDeclaration {
                source: DisclosureNodeId::Fact(fact.clone()),
                target: DisclosureNodeId::Relation(op(K::ClearFlow)),
                edge: DisclosureEdge::RelationOperand,
            });
            disclosure_edges.push(DisclosureDependencyDeclaration {
                source: DisclosureNodeId::Fact(fact.clone()),
                target: DisclosureNodeId::Relation(ash_relation.clone()),
                edge: DisclosureEdge::PublicObservableInput,
            });
        }
        if matches!(
            fact,
            FactId::BoundValue {
                bound: BoundId::AshBatchMax
            }
        ) {
            disclosure_edges.push(DisclosureDependencyDeclaration {
                source: DisclosureNodeId::Fact(fact.clone()),
                target: DisclosureNodeId::Relation(clamp.clone()),
                edge: DisclosureEdge::RelationOperand,
            });
        }
        if let FactId::StateField { field, .. } = fact {
            disclosure_edges.push(DisclosureDependencyDeclaration {
                source: DisclosureNodeId::Fact(fact.clone()),
                target: DisclosureNodeId::Relation(id(
                    K::ClearAssignment,
                    RelationSubject::StateField { field: *field },
                )),
                edge: DisclosureEdge::RelationOperand,
            });
            if matches!(field, StateField::YL | StateField::YT) {
                disclosure_edges.push(DisclosureDependencyDeclaration {
                    source: DisclosureNodeId::Fact(fact.clone()),
                    target: DisclosureNodeId::Relation(clamp.clone()),
                    edge: DisclosureEdge::RelationOperand,
                });
            }
            if matches!(field, StateField::Omega | StateField::YL | StateField::YT) {
                disclosure_edges.push(DisclosureDependencyDeclaration {
                    source: DisclosureNodeId::Fact(fact.clone()),
                    target: DisclosureNodeId::Relation(op(K::ClearFloor)),
                    edge: DisclosureEdge::RelationOperand,
                });
            }
        }
    }
    for field in StateField::ALL {
        let fact = FactId::StateField {
            operation: OperationId::Clear,
            side: Output,
            field: *field,
        };
        disclosure_nodes.push(DisclosureNode::Fact {
            id: fact.clone(),
            initial_visibility: InitialVisibility::Public,
        });
        disclosure_edges.push(DisclosureDependencyDeclaration {
            source: DisclosureNodeId::Fact(fact.clone()),
            target: DisclosureNodeId::Relation(id(
                K::ClearAssignment,
                RelationSubject::StateField { field: *field },
            )),
            edge: DisclosureEdge::RelationOperand,
        });
        if matches!(field, StateField::Omega | StateField::YL | StateField::YT) {
            disclosure_edges.push(DisclosureDependencyDeclaration {
                source: DisclosureNodeId::Fact(fact.clone()),
                target: DisclosureNodeId::Relation(op(K::ClearFloor)),
                edge: DisclosureEdge::RelationOperand,
            });
            disclosure_edges.push(DisclosureDependencyDeclaration {
                source: DisclosureNodeId::Fact(fact),
                target: DisclosureNodeId::Relation(binding_relation.clone()),
                edge: DisclosureEdge::PublicObservableInput,
            });
        }
    }
    let mut disclosure_seeds = vec![
        DisclosureSeed {
            node: DisclosureNodeId::Relation(state_relation),
            reason: DisclosureReason::PublicState,
        },
        DisclosureSeed {
            node: DisclosureNodeId::Relation(ash_relation),
            reason: DisclosureReason::PublicInterface,
        },
        DisclosureSeed {
            node: DisclosureNodeId::Relation(binding_relation),
            reason: DisclosureReason::PublicEvent,
        },
        DisclosureSeed {
            node: DisclosureNodeId::Relation(construction_relation.clone()),
            reason: DisclosureReason::PermissionlessConstructibility {
                operation: OperationId::Clear,
                relation: construction_relation,
            },
        },
    ];
    disclosure_nodes.sort_by_key(DisclosureNode::id);
    disclosure_edges.sort();
    disclosure_seeds.sort();
    let mut actual_nodes = shell.disclosure_nodes.clone();
    let mut actual_edges = shell.disclosure_edges.clone();
    let mut actual_seeds = shell.disclosure_seeds.clone();
    actual_nodes.sort_by_key(DisclosureNode::id);
    actual_edges.sort();
    actual_seeds.sort();
    assert_eq!(actual_nodes, disclosure_nodes);
    assert_eq!(actual_edges, disclosure_edges);
    assert_eq!(actual_seeds, disclosure_seeds);
    assert_eq!(
        shell
            .relations
            .iter()
            .find(|row| row.id == op(K::Authorization))
            .unwrap()
            .relation,
        Relation::PermissionlessAuthorization
    );
    assert_eq!(
        shell
            .relations
            .iter()
            .find(|row| row.id == op(K::Constructibility))
            .unwrap()
            .relation,
        Relation::Constructibility {
            class: ConstructibilityClass::PublicPermissionless
        }
    );
    assert_eq!(
        shell
            .relations
            .iter()
            .find(|row| row.id == op(K::ClearClamp))
            .unwrap()
            .relation,
        Relation::ClearClamp
    );
    assert_eq!(
        shell
            .relations
            .iter()
            .find(|row| row.id == op(K::ClearFlow))
            .unwrap()
            .relation,
        Relation::ClearFlow
    );
    assert_eq!(
        shell
            .relations
            .iter()
            .find(|row| row.id == op(K::ClearBinding))
            .unwrap()
            .relation,
        Relation::ClearBinding
    );
    assert_eq!(
        shell
            .constructibility_nodes
            .iter()
            .filter(|row| matches!(row.id, ConstructibilityNodeId::Witness { .. }))
            .count(),
        1
    );
    assert!(shell.constructibility_nodes.iter().all(|row| !matches!(
        row.id,
        ConstructibilityNodeId::Witness {
            role: WitnessRole::ProtocolOwnerAuthorization,
            ..
        }
    )));
    assert_eq!(shell.lifecycle_edges.len(), 16);
    assert_eq!(shell.disclosure_seeds.len(), 4);
    let public = realized();
    assert_eq!(public.operations[&OperationId::Clear], shell);
    let constructor_reason = DisclosureReason::PermissionlessConstructibility {
        operation: OperationId::Clear,
        relation: op(K::Constructibility),
    };
    let mut required_public = BTreeMap::new();
    for fact in facts {
        let mut reasons = BTreeSet::from([constructor_reason.clone()]);
        if let FactId::StateField { .. } = &fact {
            reasons.insert(DisclosureReason::PublicState);
        }
        if let FactId::FamilyAmount {
            object: ObjectId::Ash,
            ..
        } = &fact
        {
            reasons.insert(DisclosureReason::PublicInterface);
        }
        required_public.insert(fact, reasons);
    }
    for field in StateField::ALL {
        required_public.insert(
            FactId::StateField {
                operation: OperationId::Clear,
                side: Output,
                field: *field,
            },
            if matches!(field, StateField::Omega | StateField::YL | StateField::YT) {
                BTreeSet::from([DisclosureReason::PublicState, DisclosureReason::PublicEvent])
            } else {
                BTreeSet::from([DisclosureReason::PublicState])
            },
        );
    }
    assert_eq!(public.declassification.required_public, required_public);
    assert!(public.declassification.newly_disclosed.is_empty());
    assert!(public.declassification.retained_private.is_empty());
    let mut witness_obligations = BTreeMap::from([
        (
            architecture::WitnessId::CanonicalDelta,
            BTreeSet::from([
                op(K::CanonicalDeltaPolicy),
                op(K::ClearClamp),
                op(K::ClearFlow),
                op(K::DataOutputs),
                id(
                    K::AllowedObjectFamilies,
                    RelationSubject::TransactionSide { side: Input },
                ),
                id(
                    K::AllowedObjectFamilies,
                    RelationSubject::TransactionSide { side: Output },
                ),
            ]),
        ),
        (
            architecture::WitnessId::AshLineage,
            BTreeSet::from([
                op(K::ClearClamp),
                op(K::ClearBinding),
                op(K::ClearFlow),
                id(
                    K::Recognition,
                    RelationSubject::ObjectFamily {
                        side: Input,
                        object: ObjectId::Ash,
                    },
                ),
                id(
                    K::Recognition,
                    RelationSubject::ObjectFamily {
                        side: Output,
                        object: ObjectId::Ash,
                    },
                ),
                id(
                    K::Lifecycle,
                    RelationSubject::LifecycleExit {
                        object: ObjectId::Ash,
                        exit: OperationId::CompactAsh,
                    },
                ),
                id(
                    K::Lifecycle,
                    RelationSubject::LifecycleExit {
                        object: ObjectId::Ash,
                        exit: OperationId::Clear,
                    },
                ),
            ]),
        ),
        (
            architecture::WitnessId::StateSuccession,
            BTreeSet::from([
                op(K::RootPolicy),
                op(K::ClearBinding),
                id(
                    K::Cardinality,
                    RelationSubject::ObjectFamily {
                        side: Input,
                        object: ObjectId::State,
                    },
                ),
                id(
                    K::Recognition,
                    RelationSubject::ObjectFamily {
                        side: Input,
                        object: ObjectId::State,
                    },
                ),
                id(
                    K::Cardinality,
                    RelationSubject::ObjectFamily {
                        side: Output,
                        object: ObjectId::State,
                    },
                ),
                id(
                    K::Recognition,
                    RelationSubject::ObjectFamily {
                        side: Output,
                        object: ObjectId::State,
                    },
                ),
            ]),
        ),
        (
            architecture::WitnessId::FloorNondecrease,
            BTreeSet::from([op(K::ClearClamp), op(K::ClearFloor)]),
        ),
        (
            architecture::WitnessId::ValueFlowClosure,
            BTreeSet::from([
                op(K::CanonicalDeltaPolicy),
                op(K::OpenFlowPolicy),
                op(K::ClearFlow),
                id(K::SponsorIsolation, RelationSubject::Sponsor),
                id(K::SponsorEnvelopeMultiplicity, RelationSubject::Sponsor),
                id(
                    K::AllowedObjectFamilies,
                    RelationSubject::TransactionSide { side: Input },
                ),
                id(
                    K::AllowedObjectFamilies,
                    RelationSubject::TransactionSide { side: Output },
                ),
            ]),
        ),
        (
            architecture::WitnessId::NativeFeeAuction,
            BTreeSet::from([
                op(K::OpenFlowPolicy),
                op(K::Constructibility),
                id(K::SponsorIsolation, RelationSubject::Sponsor),
                id(K::SponsorEnvelopeMultiplicity, RelationSubject::Sponsor),
            ]),
        ),
    ]);
    for field in StateField::ALL {
        let assignment = id(
            K::ClearAssignment,
            RelationSubject::StateField { field: *field },
        );
        witness_obligations
            .get_mut(&architecture::WitnessId::StateSuccession)
            .unwrap()
            .insert(assignment.clone());
        witness_obligations
            .get_mut(&architecture::WitnessId::FloorNondecrease)
            .unwrap()
            .insert(assignment);
    }
    assert_eq!(
        ARCHITECTURE
            .operation(OperationId::Clear)
            .unwrap()
            .witnesses
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        witness_obligations.keys().copied().collect()
    );
    assert!(
        witness_obligations
            .values()
            .all(|relations| relations.is_subset(&ids))
    );
}

#[test]
#[allow(clippy::too_many_lines)] // Keep each changed architecture coordinate beside its exact refusal.
fn clear_weld_refuses_changed_coordinates() {
    use ArchitectureMismatchField as F;
    assert_eq!(
        crate::validate::validate_clear_architecture(&ARCHITECTURE),
        Ok(())
    );
    let mut architecture = ARCHITECTURE;
    let mut rows = architecture.operations.to_vec();
    let clear = rows
        .iter_mut()
        .find(|row| row.id == OperationId::Clear)
        .unwrap();
    let mut roots = clear.roots.to_vec();
    roots.push(architecture::RootUseSpec {
        root: architecture::RootId::Resv,
        use_kind: architecture::RootUse::Succession,
    });
    clear.roots = roots.leak();
    architecture.operations = rows.leak();
    assert_eq!(
        crate::validate::validate_clear_architecture(&architecture),
        Err(mismatch(F::RootPolicy))
    );
    let mut architecture = ARCHITECTURE;
    let mut rows = architecture.operations.to_vec();
    let clear = rows
        .iter_mut()
        .find(|row| row.id == OperationId::Clear)
        .unwrap();
    let mut witnesses = clear.witnesses.to_vec();
    witnesses[0] = architecture::WitnessId::ReceiptClassClosure;
    clear.witnesses = witnesses.leak();
    architecture.operations = rows.leak();
    assert_eq!(
        crate::validate::validate_clear_architecture(&architecture),
        Err(mismatch(F::Witnesses))
    );
    let mut architecture = ARCHITECTURE;
    let mut rows = architecture.operations.to_vec();
    let clear = rows
        .iter_mut()
        .find(|row| row.id == OperationId::Clear)
        .unwrap();
    let mut outputs = clear.data_outputs.to_vec();
    outputs[0].tag = TagId::Burn;
    clear.data_outputs = outputs.leak();
    architecture.operations = rows.leak();
    assert_eq!(
        crate::validate::validate_clear_architecture(&architecture),
        Err(mismatch(F::DataOutputs))
    );
    let mut shell = crate::declarations::clear::declaration();
    shell
        .relations
        .retain(|row| row.id != op(RelationKind::RootPolicy));
    assert_eq!(
        crate::validate::validate_clear_declaration(&shell),
        Err(RealizationError::MissingArchitectureRelation {
            operation: OperationId::Clear,
            kind: RelationKind::RootPolicy,
            subject: RelationSubject::Operation,
        })
    );
    let mut architecture = ARCHITECTURE;
    let mut rows = architecture.operations.to_vec();
    let clear = rows
        .iter_mut()
        .find(|row| row.id == OperationId::Clear)
        .unwrap();
    let mut outputs = clear.data_outputs.to_vec();
    outputs[0].kind = architecture::DataOutputKind::BurnRecord;
    clear.data_outputs = outputs.leak();
    architecture.operations = rows.leak();
    assert_eq!(
        crate::validate::validate_clear_architecture(&architecture),
        Err(mismatch(F::DataOutputs))
    );
    let mut architecture = ARCHITECTURE;
    let mut rows = architecture.operations.to_vec();
    let clear = rows
        .iter_mut()
        .find(|row| row.id == OperationId::Clear)
        .unwrap();
    let mut deltas = clear.canonical_deltas.to_vec();
    deltas[0].destruction_tag = Some(TagId::Burn);
    clear.canonical_deltas = deltas.leak();
    architecture.operations = rows.leak();
    assert_eq!(
        crate::validate::validate_clear_architecture(&architecture),
        Err(mismatch(F::CanonicalDelta))
    );
    let mut architecture = ARCHITECTURE;
    let mut rows = architecture.operations.to_vec();
    let clear = rows
        .iter_mut()
        .find(|row| row.id == OperationId::Clear)
        .unwrap();
    let mut deltas = clear.canonical_deltas.to_vec();
    deltas[1].condition = architecture::DeltaCondition::Always;
    clear.canonical_deltas = deltas.leak();
    architecture.operations = rows.leak();
    assert_eq!(
        crate::validate::validate_clear_architecture(&architecture),
        Err(mismatch(F::CanonicalDelta))
    );
    let mut shell = crate::declarations::clear::declaration();
    let relation = id(
        RelationKind::ClearAssignment,
        RelationSubject::StateField {
            field: StateField::Omega,
        },
    );
    let expression = ExprId::relation(relation.clone(), ExpressionRole::Predicate);
    let node = shell
        .expressions
        .iter_mut()
        .find(|node| node.id == expression)
        .unwrap();
    let ExpressionNode::Equal { left, right } = &mut node.node else {
        panic!("copy relation has Equal predicate");
    };
    *right = left.clone();
    assert_eq!(
        crate::validate::validate_operation_ownership(OperationId::Clear, &shell),
        Err(RealizationError::ArchitectureRelationMismatch { relation })
    );
    let mut shell = crate::declarations::clear::declaration();
    let fact = FactId::FamilyAmount {
        operation: OperationId::Clear,
        side: TransactionSide::Input,
        object: ObjectId::Ash,
    };
    let required = ConstructibilityNodeId::Fact {
        operation: OperationId::Clear,
        fact: fact.clone(),
        availability: AvailabilityClass::Public,
    };
    shell
        .constructibility_nodes
        .retain(|row| row.id != required);
    shell
        .constructibility_edges
        .retain(|row| row.source != required);
    shell
        .disclosure_nodes
        .retain(|row| row.id() != DisclosureNodeId::Fact(fact.clone()));
    shell
        .disclosure_edges
        .retain(|row| row.source != DisclosureNodeId::Fact(fact.clone()));
    let error = crate::derive::assemble_scoped_realization(
        &ARCHITECTURE,
        ArchitectureBinding::from_architecture(&ARCHITECTURE).unwrap(),
        RealizationScope::from_operations([OperationId::Clear]).unwrap(),
        BTreeMap::from([(OperationId::Clear, shell)]),
    )
    .unwrap_err();
    assert_eq!(error, census_mismatch(RelationKind::Constructibility));
}

#[test]
#[allow(clippy::too_many_lines)] // Keep omitted and changed shell subjects beside the accepted permutation.
fn clear_shell_refuses_omission_and_accepts_permutation() {
    let shell = crate::declarations::clear::declaration();
    assert_eq!(crate::validate::validate_clear_declaration(&shell), Ok(()));
    let mut permuted = shell.clone();
    permuted.relations.reverse();
    permuted.relation_dependencies.reverse();
    permuted.expressions.reverse();
    permuted.constructibility_nodes.reverse();
    permuted.constructibility_edges.reverse();
    permuted.lifecycle_nodes.reverse();
    permuted.lifecycle_edges.reverse();
    permuted.disclosure_nodes.reverse();
    permuted.disclosure_edges.reverse();
    permuted.disclosure_seeds.reverse();
    assert_eq!(
        crate::validate::validate_clear_declaration(&permuted),
        Ok(())
    );
    let mut omitted = shell.clone();
    let public_ash = ConstructibilityNodeId::Fact {
        operation: OperationId::Clear,
        fact: FactId::FamilyAmount {
            operation: OperationId::Clear,
            side: TransactionSide::Input,
            object: ObjectId::Ash,
        },
        availability: AvailabilityClass::Public,
    };
    omitted
        .constructibility_edges
        .retain(|row| row.source != public_ash);
    assert_eq!(
        crate::validate::validate_clear_declaration(&omitted),
        Err(census_mismatch(RelationKind::Constructibility))
    );
    assert_eq!(
        crate::derive::assemble_scoped_realization(
            &ARCHITECTURE,
            ArchitectureBinding::from_architecture(&ARCHITECTURE).unwrap(),
            RealizationScope::from_operations([OperationId::Clear]).unwrap(),
            BTreeMap::from([(OperationId::Clear, omitted)]),
        )
        .unwrap_err(),
        census_mismatch(RelationKind::Constructibility)
    );
    let mut missing_seed = shell.clone();
    missing_seed
        .disclosure_seeds
        .retain(|row| row.reason != DisclosureReason::PublicEvent);
    assert_eq!(
        crate::validate::validate_clear_declaration(&missing_seed),
        Err(census_mismatch(RelationKind::Representation))
    );
    let mut changed = shell.clone();
    let copy = ExprId::relation(
        id(
            RelationKind::ClearAssignment,
            RelationSubject::StateField {
                field: StateField::Omega,
            },
        ),
        ExpressionRole::Predicate,
    );
    let node = changed
        .expressions
        .iter_mut()
        .find(|node| node.id == copy)
        .unwrap();
    let ExpressionNode::Equal { left, right } = &mut node.node else {
        panic!("copy predicate must be Equal");
    };
    *right = left.clone();
    assert_eq!(
        crate::validate::validate_clear_declaration(&changed),
        Err(census_mismatch(RelationKind::ClearAssignment))
    );
    let mut surplus = shell.clone();
    let duplicate = surplus.relations[0].clone();
    surplus.relations.push(duplicate);
    assert_eq!(
        crate::validate::validate_clear_declaration(&surplus),
        Err(RealizationError::SurplusArchitectureRelation {
            relation: shell.relations[0].id.clone()
        })
    );
    let state_exit = id(
        RelationKind::Lifecycle,
        RelationSubject::LifecycleExit {
            object: ObjectId::State,
            exit: OperationId::Clear,
        },
    );
    let mut coherent = shell.clone();
    coherent.relations.retain(|row| row.id != state_exit);
    coherent
        .relation_dependencies
        .retain(|edge| edge.dependent != state_exit && edge.prerequisite != state_exit);
    coherent.lifecycle_nodes.retain(|node| {
        node.id
            != LifecycleNodeId::RequiredExit {
                object: ObjectId::State,
                operation: OperationId::Clear,
            }
    });
    coherent.lifecycle_edges.retain(|edge| {
        edge.target
            != LifecycleNodeId::RequiredExit {
                object: ObjectId::State,
                operation: OperationId::Clear,
            }
    });
    assert_eq!(
        crate::validate::validate_clear_declaration(&coherent),
        Err(RealizationError::MissingArchitectureRelation {
            operation: OperationId::Clear,
            kind: RelationKind::Lifecycle,
            subject: RelationSubject::LifecycleExit {
                object: ObjectId::State,
                exit: OperationId::Clear
            }
        })
    );
    let mut missing = shell;
    missing.lifecycle_edges.pop();
    assert_eq!(
        crate::validate::validate_clear_declaration(&missing),
        Err(census_mismatch(RelationKind::Representation))
    );
}

#[test]
#[allow(clippy::too_many_lines)] // Compare both literal residual branches and successor fields together.
fn complete_clear_accepts_full_and_partial_residual() {
    let realized = realized();
    for (b, yl, yt, x, next_yl, residual) in [(10, 6, 3, 6, 0, 4), (2, 6, 3, 2, 4, 0)] {
        let (common, extension) = control(b, yl, yt, x, next_yl, residual);
        let BurnClearPayload::Clear {
            predecessor,
            successor,
            destruction,
            ..
        } = &extension.payload
        else {
            panic!("Clear control must carry its typed payload");
        };
        assert_eq!(predecessor.metadata.omega, ProtocolAmount::new(20).unwrap());
        assert_eq!(predecessor.metadata.y_l, ProtocolAmount::new(yl).unwrap());
        assert_eq!(predecessor.metadata.y_t, ProtocolAmount::new(yt).unwrap());
        assert_eq!(successor.metadata.omega, ProtocolAmount::new(20).unwrap());
        assert_eq!(
            successor.metadata.y_l,
            ProtocolAmount::new(next_yl).unwrap()
        );
        assert_eq!(successor.metadata.y_t, ProtocolAmount::new(yt).unwrap());
        assert_eq!(successor.metadata.q, ProtocolAmount::new(2).unwrap());
        assert_eq!(successor.metadata.cycle, Cycle::new(7));
        assert_eq!(successor.metadata.maturity, Maturity::Unannounced);
        assert_eq!(destruction.amount, ProtocolAmount::new(x).unwrap());
        assert_eq!(destruction.tag, TagId::Recon);
        assert_eq!(destruction.asset, AssetId::U);
        assert_eq!(
            common
                .objects
                .iter()
                .filter(
                    |object| object.kind == ObservedObjectKind::Declared(ObjectId::Ash)
                        && object.reference.side == ObservedSide::Output
                )
                .count(),
            usize::from(residual > 0)
        );
        let report = realized
            .evaluate_operation_with_burn_clear(&bind(&common, &extension))
            .unwrap();
        assert!(report.is_conformant(), "{report:?}");
        assert_eq!(
            report
                .verdict(&op(RelationKind::ClearClamp))
                .unwrap()
                .status,
            RelationStatus::Passed
        );
        assert_eq!(
            report
                .verdict(&op(RelationKind::ClearBinding))
                .unwrap()
                .status,
            RelationStatus::Passed
        );
        for kind in [
            RelationKind::CanonicalDeltaPolicy,
            RelationKind::ClearFloor,
            RelationKind::ClearFlow,
            RelationKind::DataOutputs,
        ] {
            assert_eq!(
                report.verdict(&op(kind)).unwrap().status,
                RelationStatus::Passed
            );
        }
        for field in StateField::ALL {
            assert_eq!(
                report
                    .verdict(&id(
                        RelationKind::ClearAssignment,
                        RelationSubject::StateField { field: *field }
                    ))
                    .unwrap()
                    .status,
                RelationStatus::Passed
            );
        }
        assert_eq!(
            report
                .verdict(&id(
                    RelationKind::SubstrateConservation,
                    RelationSubject::Asset {
                        asset: AssetId::Lbtc
                    }
                ))
                .unwrap()
                .status,
            RelationStatus::EvidenceRequired {
                requirement: ExternalEvidenceRequirement::SubstrateConservation {
                    operation: OperationId::Clear,
                    asset: AssetId::Lbtc
                }
            }
        );
        assert!(
            common
                .objects
                .iter()
                .all(|object| object.kind != ObservedObjectKind::Declared(ObjectId::Resv))
        );
    }
}

#[test]
fn complete_clear_clamp_failure_blocks_dependents() {
    let realized = realized();
    for (b, yl, yt, x, next_yl, residual, rejected_yl) in
        [(10, 2, 0, 1, 1, 9, 0), (10, 2, 0, 1, 1, 9, 1)]
    {
        let (common, mut extension) = control(b, yl, yt, x, next_yl, residual);
        assert!(
            realized
                .evaluate_operation_with_burn_clear(&bind(&common, &extension))
                .unwrap()
                .is_conformant()
        );
        if let BurnClearPayload::Clear { predecessor, .. } = &mut extension.payload {
            predecessor.metadata.y_l = ProtocolAmount::new(rejected_yl).unwrap();
        }
        let report = realized
            .evaluate_operation_with_burn_clear(&bind(&common, &extension))
            .unwrap();
        assert_eq!(
            report
                .verdict(&op(RelationKind::ClearClamp))
                .unwrap()
                .status,
            RelationStatus::Failed {
                reason: RelationFailure::BurnClearLaw
            }
        );
        assert_eq!(
            report
                .verdict(&op(RelationKind::CanonicalDeltaPolicy))
                .unwrap()
                .status,
            RelationStatus::Blocked {
                prerequisites: vec![op(RelationKind::ClearClamp)]
            }
        );
        for kind in [RelationKind::ClearFlow, RelationKind::DataOutputs] {
            assert_eq!(
                report.verdict(&op(kind)).unwrap().status,
                RelationStatus::Blocked {
                    prerequisites: vec![op(RelationKind::ClearClamp)]
                }
            );
        }
        let mut assignments = Vec::new();
        for field in StateField::ALL {
            let assignment = id(
                RelationKind::ClearAssignment,
                RelationSubject::StateField { field: *field },
            );
            assert_eq!(
                report.verdict(&assignment).unwrap().status,
                RelationStatus::Blocked {
                    prerequisites: vec![op(RelationKind::ClearClamp)]
                }
            );
            assignments.push(assignment);
        }
        assignments.sort();
        assert_eq!(
            report
                .verdict(&op(RelationKind::ClearFloor))
                .unwrap()
                .status,
            RelationStatus::Blocked {
                prerequisites: assignments
            }
        );
        let mut binding_prerequisites = vec![
            op(RelationKind::ClearFloor),
            op(RelationKind::ClearFlow),
            op(RelationKind::DataOutputs),
        ];
        binding_prerequisites.sort();
        assert_eq!(
            report
                .verdict(&op(RelationKind::ClearBinding))
                .unwrap()
                .status,
            RelationStatus::Blocked {
                prerequisites: binding_prerequisites
            }
        );
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Four pairwise literal reason and private-set controls belong together.
fn overlapping_clear_scopes_have_literal_unions() {
    realized();
    let clear_reason = DisclosureReason::PermissionlessConstructibility {
        operation: OperationId::Clear,
        relation: op(RelationKind::Constructibility),
    };
    for (other, objects) in [
        (
            OperationId::CompactAsh,
            &[ObjectId::Ash, ObjectId::State][..],
        ),
        (
            OperationId::Burn,
            &[ObjectId::Ash, ObjectId::State, ObjectId::ReceiptLive][..],
        ),
        (
            OperationId::AnnounceMaturity,
            &[ObjectId::Ash, ObjectId::State][..],
        ),
        (
            OperationId::TransferLive,
            &[ObjectId::Ash, ObjectId::State, ObjectId::ReceiptLive][..],
        ),
    ] {
        let mixed = derive(
            &ARCHITECTURE,
            RealizationScope::from_operations([OperationId::Clear, other]).unwrap(),
        )
        .unwrap();
        mixed.validate_against(&ARCHITECTURE).unwrap();
        super::derivation_tests::assert_literal_lifecycle(&mixed, objects);
        super::derivation_tests::assert_pair_disclosure_union(&mixed, [OperationId::Clear, other]);
        assert_eq!(mixed.operations.len(), 2);
        if other == OperationId::CompactAsh {
            let compact_reason = DisclosureReason::PermissionlessConstructibility {
                operation: OperationId::CompactAsh,
                relation: RelationId::new(
                    OperationId::CompactAsh,
                    RelationKind::Constructibility,
                    RelationSubject::Operation,
                ),
            };
            assert_eq!(
                mixed.declassification.required_public[&FactId::BoundValue {
                    bound: BoundId::AshBatchMax,
                }],
                BTreeSet::from([clear_reason.clone(), compact_reason])
            );
        }
        if other == OperationId::Burn {
            assert_eq!(
                mixed.declassification.required_public[&FactId::BoundValue {
                    bound: BoundId::FeeSponsorInputMax,
                }],
                BTreeSet::from([clear_reason.clone()])
            );
        }
        let expected_private = if other == OperationId::Burn {
            BTreeSet::from([FactId::FamilyAmount {
                operation: OperationId::Burn,
                side: TransactionSide::Output,
                object: ObjectId::ReceiptLive,
            }])
        } else if other == OperationId::TransferLive {
            [TransactionSide::Input, TransactionSide::Output]
                .into_iter()
                .map(|side| FactId::FamilyAmount {
                    operation: OperationId::TransferLive,
                    side,
                    object: ObjectId::ReceiptLive,
                })
                .collect()
        } else {
            BTreeSet::new()
        };
        assert_eq!(mixed.declassification.retained_private, expected_private);
        assert!(mixed.declassification.newly_disclosed.is_empty());
    }
}
