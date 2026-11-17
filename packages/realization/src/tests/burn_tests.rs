use std::collections::{BTreeMap, BTreeSet};

use architecture::{ARCHITECTURE, AssetId, BoundId, ObjectId, OperationId};

use crate::*;

fn realized() -> ScopedRealizationSpec {
    derive(
        &ARCHITECTURE,
        RealizationScope::from_operations([OperationId::Burn]).unwrap(),
    )
    .unwrap()
}

fn rid(kind: RelationKind, subject: RelationSubject) -> RelationId {
    RelationId::new(OperationId::Burn, kind, subject)
}

fn op(kind: RelationKind) -> RelationId {
    rid(kind, RelationSubject::Operation)
}

fn family(kind: RelationKind, side: TransactionSide, object: ObjectId) -> RelationId {
    rid(kind, RelationSubject::ObjectFamily { side, object })
}

fn census_mismatch(kind: RelationKind) -> RealizationError {
    RealizationError::ArchitectureRelationMismatch {
        relation: rid(
            kind,
            if kind == RelationKind::Representation {
                RelationSubject::Representation {
                    object: ObjectId::Ash,
                }
            } else {
                RelationSubject::Operation
            },
        ),
    }
}

fn mismatch(field: ArchitectureMismatchField) -> RealizationError {
    RealizationError::ArchitectureOperationMismatch {
        operation: OperationId::Burn,
        field,
    }
}

fn control() -> (OperationObservation, BurnClearObservation) {
    let (mut common, mut extension) = super::burn_clear_relation_tests::burn();
    for bound in [
        BoundId::BurnInputMax,
        BoundId::BurnChangeMax,
        BoundId::FeeSponsorInputMax,
    ] {
        common.bounds.insert(bound, Count::new(4));
    }
    extension.context =
        BurnClearContext::bind_observation(common.clone(), [1; 32], [2; 32]).unwrap();
    (common, extension)
}

fn expected_row(
    id: RelationId,
    relation: Relation,
    proof: Option<ProofKind>,
) -> RelationDeclaration {
    let proof_alternatives = proof.map_or_else(BTreeSet::new, |kind| {
        BTreeSet::from([ProofAlternativeId::new(id.clone(), kind)])
    });
    RelationDeclaration {
        id,
        relation,
        proof_alternatives,
    }
}

fn observed_side(side: TransactionSide) -> ObservedSide {
    match side {
        TransactionSide::Input => ObservedSide::Input,
        TransactionSide::Output => ObservedSide::Output,
    }
}

// Literal B family rows of (`[PLAN-tab:realization:burn-clear-relations]`).
fn expected_family_rows() -> Vec<RelationDeclaration> {
    use ObjectId::{Ash, PlainLbtc, ReceiptLive};
    use TransactionSide::{Input, Output};
    let mut rows = Vec::new();
    for (side, object, asset, minimum, maximum) in [
        (
            Input,
            ReceiptLive,
            AssetId::U,
            1,
            CardinalityMaximum::Bound(BoundId::BurnInputMax),
        ),
        (
            Output,
            ReceiptLive,
            AssetId::U,
            0,
            CardinalityMaximum::Bound(BoundId::BurnChangeMax),
        ),
        (
            Output,
            Ash,
            AssetId::U,
            1,
            CardinalityMaximum::Exact(Count::ONE),
        ),
        (
            Input,
            PlainLbtc,
            AssetId::Lbtc,
            0,
            CardinalityMaximum::Bound(BoundId::FeeSponsorInputMax),
        ),
        (
            Output,
            PlainLbtc,
            AssetId::Lbtc,
            0,
            CardinalityMaximum::Exact(Count::ONE),
        ),
    ] {
        rows.push(expected_row(
            family(RelationKind::Cardinality, side, object),
            Relation::Cardinality {
                side: observed_side(side),
                object,
                minimum: Count::new(minimum),
                maximum,
            },
            Some(ProofKind::ManifestShape),
        ));
        rows.push(expected_row(
            family(RelationKind::Recognition, side, object),
            Relation::Recognition {
                side: observed_side(side),
                object,
                asset,
            },
            Some(ProofKind::ManifestShape),
        ));
    }
    for (side, allowed) in [
        (Input, BTreeSet::from([ReceiptLive, PlainLbtc])),
        (Output, BTreeSet::from([ReceiptLive, Ash, PlainLbtc])),
    ] {
        rows.push(expected_row(
            rid(
                RelationKind::AllowedObjectFamilies,
                RelationSubject::TransactionSide { side },
            ),
            Relation::AllowedObjectFamilies {
                side: observed_side(side),
                allowed,
            },
            Some(ProofKind::ManifestShape),
        ));
    }
    rows
}

// The B policy clauses and proof paragraph of the adopted relation/graph tables.
#[allow(clippy::too_many_lines)] // Keep the literal policy bodies beside their proof kinds.
fn expected_policy_rows() -> Vec<RelationDeclaration> {
    use ProofKind::{ManifestShape, PublicArithmetic, SignerMembership};
    use RelationKind as K;
    use architecture::{DeltaCondition, DeltaKind, ProjectionId, ProjectionRule, RootId, RootUse};
    let mut rows = vec![
        expected_row(
            family(
                K::Authorization,
                TransactionSide::Input,
                ObjectId::ReceiptLive,
            ),
            Relation::OwnerAuthorization {
                object: ObjectId::ReceiptLive,
            },
            Some(SignerMembership),
        ),
        expected_row(
            rid(
                K::Conservation,
                RelationSubject::Asset { asset: AssetId::U },
            ),
            Relation::AmountConservation {
                asset: AssetId::U,
                input_objects: BTreeSet::from([ObjectId::ReceiptLive]),
                output_objects: BTreeSet::from([ObjectId::ReceiptLive, ObjectId::Ash]),
            },
            Some(PublicArithmetic),
        ),
        expected_row(
            rid(K::SponsorIsolation, RelationSubject::Sponsor),
            Relation::SponsorIsolation,
            Some(ManifestShape),
        ),
        expected_row(
            rid(K::SponsorEnvelopeMultiplicity, RelationSubject::Sponsor),
            Relation::SponsorEnvelopeMultiplicity {
                maximum: Count::ONE,
            },
            Some(ManifestShape),
        ),
        expected_row(
            rid(
                K::SubstrateConservation,
                RelationSubject::Asset {
                    asset: AssetId::Lbtc,
                },
            ),
            Relation::SubstrateConservation {
                asset: AssetId::Lbtc,
            },
            Some(ProofKind::SubstrateConservation),
        ),
    ];
    for (kind, body, proof) in [
        (K::BurnBinding, Relation::BurnBinding, ManifestShape),
        (
            K::CanonicalDeltaPolicy,
            Relation::CanonicalDeltaPolicy {
                expected: BTreeSet::from([ExpectedCanonicalDelta {
                    asset: AssetId::U,
                    kind: DeltaKind::Lateral,
                    destruction_tag: None,
                }]),
            },
            ManifestShape,
        ),
        (
            K::OpenFlowPolicy,
            Relation::OpenFlowPolicy {
                allowed: BTreeSet::from([architecture::OpenFlowKind::FeeSponsor]),
            },
            ManifestShape,
        ),
        (
            K::RootPolicy,
            Relation::RootPolicy {
                expected: BTreeMap::from([
                    (RootId::State, RootUse::Forbidden),
                    (RootId::Resv, RootUse::Forbidden),
                    (RootId::Pace, RootUse::Forbidden),
                    (RootId::EntAuth, RootUse::Forbidden),
                    (RootId::DistAuth, RootUse::Forbidden),
                ]),
            },
            ManifestShape,
        ),
        (
            K::ProjectionPolicy,
            Relation::ProjectionPolicy {
                expected: BTreeMap::from([
                    (
                        ProjectionId::TransitionCertificate,
                        ProjectionRule::Required,
                    ),
                    (ProjectionId::BurnEvent, ProjectionRule::Required),
                    (ProjectionId::ClearEvent, ProjectionRule::Forbidden),
                    (ProjectionId::DistributionResidue, ProjectionRule::Forbidden),
                ]),
            },
            ManifestShape,
        ),
        (
            K::Constructibility,
            Relation::Constructibility {
                class: ConstructibilityClass::OwnersOf {
                    object: ObjectId::ReceiptLive,
                },
            },
            SignerMembership,
        ),
        (
            K::DataOutputs,
            Relation::DataOutputs {
                expected: vec![architecture::DataOutputSpec {
                    kind: architecture::DataOutputKind::BurnRecord,
                    tag: architecture::TagId::Burn,
                    asset: None,
                    minimum: 0,
                    maximum: architecture::MaxCount::Bound(BoundId::BurnRecordMax),
                    condition: DeltaCondition::Always,
                }],
            },
            ManifestShape,
        ),
    ] {
        rows.push(expected_row(op(kind), body, Some(proof)));
    }
    rows
}

// The representation and exit sets are expanded from the adopted B rows.
fn expected_mode_and_exit_rows() -> Vec<RelationDeclaration> {
    use RepresentationMode::{Explicit, PrivateCommitted, PublicCommitted};
    let mut rows = Vec::new();
    for (side, allowed) in [
        (TransactionSide::Input, BTreeSet::from([Explicit])),
        (
            TransactionSide::Output,
            BTreeSet::from([Explicit, PrivateCommitted]),
        ),
    ] {
        rows.push(expected_row(
            rid(
                RelationKind::SideRepresentation,
                RelationSubject::SideRepresentation {
                    side,
                    object: ObjectId::ReceiptLive,
                },
            ),
            Relation::SideRepresentation {
                side,
                object: ObjectId::ReceiptLive,
                allowed,
            },
            None,
        ));
    }
    rows.push(expected_row(
        rid(
            RelationKind::Representation,
            RelationSubject::Representation {
                object: ObjectId::Ash,
            },
        ),
        Relation::Representation {
            object: ObjectId::Ash,
            allowed: BTreeSet::from([Explicit, PublicCommitted]),
        },
        None,
    ));
    for (object, exit) in [
        (ObjectId::ReceiptLive, OperationId::TransferLive),
        (ObjectId::ReceiptLive, OperationId::Burn),
        (ObjectId::ReceiptLive, OperationId::Redeem),
        (ObjectId::Ash, OperationId::CompactAsh),
        (ObjectId::Ash, OperationId::Clear),
    ] {
        rows.push(expected_row(
            rid(
                RelationKind::Lifecycle,
                RelationSubject::LifecycleExit { object, exit },
            ),
            Relation::LifecycleExit { object, exit },
            None,
        ));
    }
    rows
}

#[test]
fn relation_bodies_and_proofs_match_the_adopted_literal_rows() {
    // (`[PLAN-tab:realization:burn-clear-relations]`) and the proof paragraph of
    // (`[PLAN-tab:realization:burn-clear-graphs]`); no expected row reads the owner.
    let mut expected = expected_family_rows();
    expected.extend(expected_policy_rows());
    expected.extend(expected_mode_and_exit_rows());
    expected.sort_by(|left, right| left.id.cmp(&right.id));
    assert_eq!(expected.len(), 32);
    let mut actual = realized().operations[&OperationId::Burn].relations.clone();
    actual.sort_by(|left, right| left.id.cmp(&right.id));
    assert_eq!(actual, expected);
}

#[test]
fn lifecycle_rows_match_the_adopted_literal_product() {
    // (`[PLAN-tab:realization:burn-clear-graphs]`), Lifecycle row.
    use LifecycleNodeId::{Representation, RequiredExit};
    use ObjectId::{Ash, ReceiptLive};
    use OperationId::{Burn, Clear, CompactAsh, Redeem, TransferLive};
    use RepresentationMode::{Explicit, PrivateCommitted, PublicCommitted};
    let realized = realized();
    let declaration = &realized.operations[&Burn];
    let mut expected_nodes = [
        Representation {
            object: ReceiptLive,
            mode: Explicit,
        },
        Representation {
            object: ReceiptLive,
            mode: PrivateCommitted,
        },
        RequiredExit {
            object: ReceiptLive,
            operation: TransferLive,
        },
        RequiredExit {
            object: ReceiptLive,
            operation: Burn,
        },
        RequiredExit {
            object: ReceiptLive,
            operation: Redeem,
        },
        Representation {
            object: Ash,
            mode: Explicit,
        },
        Representation {
            object: Ash,
            mode: PublicCommitted,
        },
        RequiredExit {
            object: Ash,
            operation: CompactAsh,
        },
        RequiredExit {
            object: Ash,
            operation: Clear,
        },
    ];
    expected_nodes.sort();
    let mut actual_nodes = declaration
        .lifecycle_nodes
        .iter()
        .map(|row| row.id.clone())
        .collect::<Vec<_>>();
    actual_nodes.sort();
    assert_eq!(actual_nodes, expected_nodes);
    let mut expected_edges = [
        (ReceiptLive, Explicit, TransferLive),
        (ReceiptLive, Explicit, Burn),
        (ReceiptLive, Explicit, Redeem),
        (ReceiptLive, PrivateCommitted, TransferLive),
        (ReceiptLive, PrivateCommitted, Burn),
        (ReceiptLive, PrivateCommitted, Redeem),
        (Ash, Explicit, CompactAsh),
        (Ash, Explicit, Clear),
        (Ash, PublicCommitted, CompactAsh),
        (Ash, PublicCommitted, Clear),
    ]
    .map(|(object, mode, operation)| LifecycleDependencyDeclaration {
        source: Representation { object, mode },
        target: RequiredExit { object, operation },
        edge: LifecycleEdge::RequiresExit,
    });
    expected_edges.sort();
    let mut actual_edges = declaration.lifecycle_edges.clone();
    actual_edges.sort();
    assert_eq!(actual_edges, expected_edges);
}

fn expected_disclosure_facts() -> Vec<(FactId, InitialVisibility)> {
    // Constructor/disclosure facts paragraph of (`[PLAN-tab:realization:burn-clear-graphs]`).
    use InitialVisibility::{Private, Public};
    use ObjectId::{Ash, ReceiptLive};
    use TransactionSide::{Input, Output};
    let mut facts = Vec::new();
    for (side, object, visibility) in [
        (Input, ReceiptLive, Public),
        (Output, ReceiptLive, Private),
        (Output, Ash, Public),
    ] {
        facts.push((
            FactId::FamilyCount {
                operation: OperationId::Burn,
                side,
                object,
            },
            Public,
        ));
        facts.push((
            FactId::FamilyAmount {
                operation: OperationId::Burn,
                side,
                object,
            },
            visibility,
        ));
    }
    for bound in [
        BoundId::BurnInputMax,
        BoundId::BurnChangeMax,
        BoundId::BurnRecordMax,
        BoundId::FeeSponsorInputMax,
    ] {
        facts.push((FactId::BoundValue { bound }, Public));
    }
    facts
}

#[test]
#[allow(clippy::too_many_lines)] // Retain all literal disclosure rows in one comparison.
fn disclosure_rows_and_visibility_match_the_adopted_literals() {
    // (`[PLAN-tab:realization:burn-clear-graphs]`), Disclosure row and facts paragraph.
    use DisclosureNodeId::{Fact, Relation as RelationNode};
    let construction = op(RelationKind::Constructibility);
    let conservation = rid(
        RelationKind::Conservation,
        RelationSubject::Asset { asset: AssetId::U },
    );
    let ash = rid(
        RelationKind::Representation,
        RelationSubject::Representation {
            object: ObjectId::Ash,
        },
    );
    let change = rid(
        RelationKind::SideRepresentation,
        RelationSubject::SideRepresentation {
            side: TransactionSide::Output,
            object: ObjectId::ReceiptLive,
        },
    );
    let binding = op(RelationKind::BurnBinding);
    let facts = expected_disclosure_facts();
    let mut nodes = facts
        .iter()
        .map(|(id, initial_visibility)| DisclosureNode::Fact {
            id: id.clone(),
            initial_visibility: *initial_visibility,
        })
        .collect::<Vec<_>>();
    nodes.extend(
        [
            construction.clone(),
            conservation.clone(),
            ash.clone(),
            change.clone(),
            binding.clone(),
        ]
        .map(|id| DisclosureNode::Relation { id }),
    );
    let mut edges = facts
        .into_iter()
        .map(|(fact, _)| DisclosureDependencyDeclaration {
            source: Fact(fact),
            target: RelationNode(construction.clone()),
            edge: DisclosureEdge::ConstructibilityInput,
        })
        .collect::<Vec<_>>();
    for (side, object, target, edge) in [
        (
            TransactionSide::Input,
            ObjectId::ReceiptLive,
            conservation.clone(),
            DisclosureEdge::RelationOperand,
        ),
        (
            TransactionSide::Output,
            ObjectId::ReceiptLive,
            conservation.clone(),
            DisclosureEdge::RelationOperand,
        ),
        (
            TransactionSide::Output,
            ObjectId::Ash,
            conservation,
            DisclosureEdge::RelationOperand,
        ),
        (
            TransactionSide::Output,
            ObjectId::ReceiptLive,
            change,
            DisclosureEdge::PublicObservableInput,
        ),
        (
            TransactionSide::Output,
            ObjectId::Ash,
            ash.clone(),
            DisclosureEdge::PublicObservableInput,
        ),
        (
            TransactionSide::Output,
            ObjectId::Ash,
            binding.clone(),
            DisclosureEdge::PublicObservableInput,
        ),
    ] {
        edges.push(DisclosureDependencyDeclaration {
            source: Fact(FactId::FamilyAmount {
                operation: OperationId::Burn,
                side,
                object,
            }),
            target: RelationNode(target),
            edge,
        });
    }
    let mut seeds = [
        DisclosureSeed {
            node: RelationNode(ash),
            reason: DisclosureReason::PublicInterface,
        },
        DisclosureSeed {
            node: RelationNode(binding),
            reason: DisclosureReason::PublicEvent,
        },
    ];
    nodes.sort_by_key(DisclosureNode::id);
    edges.sort();
    seeds.sort();
    let declaration = realized().operations.remove(&OperationId::Burn).unwrap();
    let mut actual_nodes = declaration.disclosure_nodes;
    let mut actual_edges = declaration.disclosure_edges;
    let mut actual_seeds = declaration.disclosure_seeds;
    actual_nodes.sort_by_key(DisclosureNode::id);
    actual_edges.sort();
    actual_seeds.sort();
    assert_eq!(actual_nodes, nodes);
    assert_eq!(actual_edges, edges);
    assert_eq!(actual_seeds, seeds);
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

#[test]
fn complete_burn_evaluates_with_private_change_and_independent_overclaim() {
    let realized = realized();
    let (common, extension) = control();
    let bound = bind(&common, &extension);
    let report = realized.evaluate_operation_with_burn_clear(&bound).unwrap();
    assert!(report.is_conformant(), "{report:?}");
    assert_eq!(report.verdicts.len(), 32);
    assert!(!report.is_evidence_complete());
    assert_eq!(
        report
            .verdicts
            .iter()
            .find(|row| row.relation
                == rid(
                    RelationKind::SubstrateConservation,
                    RelationSubject::Asset {
                        asset: AssetId::Lbtc
                    }
                ))
            .unwrap()
            .status,
        RelationStatus::EvidenceRequired {
            requirement: ExternalEvidenceRequirement::SubstrateConservation {
                operation: OperationId::Burn,
                asset: AssetId::Lbtc,
            },
        }
    );
    assert_eq!(
        evaluate_record_credit(&bound).unwrap(),
        RecordCredit::Overclaim {
            total: 13,
            fresh_amount: ProtocolAmount::new(10).unwrap()
        }
    );
    assert_eq!(
        realized.evaluate_operation(&common),
        Err(RealizationError::BurnClearObservation(
            BurnClearRefusal::MissingExtension
        ))
    );
    let mut credited = extension;
    let BurnClearPayload::Burn { records, .. } = &mut credited.payload else {
        unreachable!()
    };
    records[0].amount = ProtocolAmount::new(10).unwrap();
    let credited = bind(&common, &credited);
    assert_eq!(
        evaluate_record_credit(&credited).unwrap(),
        RecordCredit::Credited { total: 10 }
    );
    assert_eq!(
        realized
            .evaluate_operation_with_burn_clear(&credited)
            .unwrap(),
        report
    );
}

#[test]
fn unequal_burn_totals_fail_delta_and_block_conservation_binding_and_data() {
    let realized = realized();
    let (mut common, extension) = control();
    assert!(
        realized
            .evaluate_operation_with_burn_clear(&bind(&common, &extension))
            .unwrap()
            .is_conformant()
    );
    // Change only fresh ASH: I=12, A=9, C=2.
    common.objects[1].value = ObservedValue::Protocol(ProtocolAmount::new(9).unwrap());
    let report = realized
        .evaluate_operation_with_burn_clear(&bind(&common, &extension))
        .unwrap();
    let conservation = rid(
        RelationKind::Conservation,
        RelationSubject::Asset { asset: AssetId::U },
    );
    for (relation, status) in [
        (
            op(RelationKind::CanonicalDeltaPolicy),
            RelationStatus::Failed {
                reason: RelationFailure::CanonicalDeltaPolicy,
            },
        ),
        (
            conservation.clone(),
            RelationStatus::Blocked {
                prerequisites: vec![op(RelationKind::CanonicalDeltaPolicy)],
            },
        ),
        (
            op(RelationKind::BurnBinding),
            RelationStatus::Blocked {
                prerequisites: vec![conservation],
            },
        ),
        (
            op(RelationKind::DataOutputs),
            RelationStatus::Blocked {
                prerequisites: vec![op(RelationKind::BurnBinding)],
            },
        ),
    ] {
        assert_eq!(
            report
                .verdicts
                .iter()
                .find(|row| row.relation == relation)
                .unwrap()
                .status,
            status
        );
    }
}

#[test]
fn zero_fresh_ash_refuses_at_observation_binding_before_recognition() {
    let realized = realized();
    let (mut common, mut extension) = control();
    assert!(
        realized
            .evaluate_operation_with_burn_clear(&bind(&common, &extension))
            .unwrap()
            .is_conformant()
    );
    common.objects[1].value = ObservedValue::Protocol(ProtocolAmount::ZERO);
    extension.context =
        BurnClearContext::bind_observation(common.clone(), [1; 32], [2; 32]).unwrap();
    assert_eq!(
        validate_burn_clear_observation(&common, &extension.context, Some(&extension)),
        Err(BurnClearRefusal::AshBinding)
    );
}

#[test]
fn private_input_is_not_admitted_by_the_input_change_mode_union() {
    let realized = realized();
    let (mut common, extension) = control();
    assert!(
        realized
            .evaluate_operation_with_burn_clear(&bind(&common, &extension))
            .unwrap()
            .is_conformant()
    );
    require_lifecycle_exit(
        &realized.lifecycle_graph,
        &realized.lifecycle_node_by_id,
        ObjectId::ReceiptLive,
        RepresentationMode::PrivateCommitted,
        OperationId::Burn,
    )
    .unwrap();
    common.objects[0].representation = RepresentationMode::PrivateCommitted;
    let report = realized
        .evaluate_operation_with_burn_clear(&bind(&common, &extension))
        .unwrap();
    let input = rid(
        RelationKind::SideRepresentation,
        RelationSubject::SideRepresentation {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
    );
    assert_eq!(
        report
            .verdicts
            .iter()
            .find(|row| row.relation == input)
            .unwrap()
            .status,
        RelationStatus::Failed {
            reason: RelationFailure::BurnClearLaw
        }
    );
    assert_eq!(
        report
            .verdicts
            .iter()
            .find(|row| row.relation == op(RelationKind::BurnBinding))
            .unwrap()
            .status,
        RelationStatus::Blocked {
            prerequisites: vec![input]
        }
    );
    assert_eq!(
        report
            .verdicts
            .iter()
            .find(|row| row.relation == op(RelationKind::DataOutputs))
            .unwrap()
            .status,
        RelationStatus::Blocked {
            prerequisites: vec![op(RelationKind::BurnBinding)]
        }
    );
}

#[test]
fn fresh_ash_keeps_both_public_modes_and_refuses_private_commitments() {
    let realized = realized();
    let (mut common, extension) = control();
    for mode in [
        RepresentationMode::Explicit,
        RepresentationMode::PublicCommitted,
    ] {
        common.objects[1].representation = mode;
        assert!(
            realized
                .evaluate_operation_with_burn_clear(&bind(&common, &extension))
                .unwrap()
                .is_conformant()
        );
        for exit in [OperationId::CompactAsh, OperationId::Clear] {
            require_lifecycle_exit(
                &realized.lifecycle_graph,
                &realized.lifecycle_node_by_id,
                ObjectId::Ash,
                mode,
                exit,
            )
            .unwrap();
        }
    }
    common.objects[1].representation = RepresentationMode::PrivateCommitted;
    let report = realized
        .evaluate_operation_with_burn_clear(&bind(&common, &extension))
        .unwrap();
    assert_eq!(
        report
            .verdicts
            .iter()
            .find(|row| row.relation
                == rid(
                    RelationKind::Representation,
                    RelationSubject::Representation {
                        object: ObjectId::Ash
                    }
                ))
            .unwrap()
            .status,
        RelationStatus::Failed {
            reason: RelationFailure::Representation
        }
    );
}

#[test]
fn complete_burn_distinguishes_disagreement_blocked_and_missing_operand() {
    let realized = realized();
    let (mut common, extension) = control();
    assert!(
        realized
            .evaluate_operation_with_burn_clear(&bind(&common, &extension))
            .unwrap()
            .is_conformant()
    );
    common.protocol_signers.clear();
    let report = realized
        .evaluate_operation_with_burn_clear(&bind(&common, &extension))
        .unwrap();
    let authorization = family(
        RelationKind::Authorization,
        TransactionSide::Input,
        ObjectId::ReceiptLive,
    );
    assert_eq!(
        report
            .verdicts
            .iter()
            .find(|row| row.relation == authorization)
            .unwrap()
            .status,
        RelationStatus::Failed {
            reason: RelationFailure::MissingOwnerAuthorization
        }
    );
    assert_eq!(
        report
            .verdicts
            .iter()
            .find(|row| row.relation == op(RelationKind::BurnBinding))
            .unwrap()
            .status,
        RelationStatus::Blocked {
            prerequisites: vec![authorization]
        }
    );
    let (mut common, extension) = control();
    common.bounds.remove(&BoundId::BurnInputMax);
    assert_eq!(
        realized.evaluate_operation_with_burn_clear(&bind(&common, &extension)),
        Err(RealizationError::MissingBoundValue(BoundId::BurnInputMax))
    );
}

#[test]
fn burn_retains_no_change_and_opaque_sponsor_controls() {
    let realized = realized();
    let (mut common, extension) = control();
    common.objects[0].value = ObservedValue::Protocol(ProtocolAmount::new(10).unwrap());
    let change = common.objects.pop().unwrap().reference;
    common
        .region
        .members
        .retain(|reference| *reference != change);
    common.region.canonical_partition.flows[0]
        .destinations
        .retain(|reference| *reference != change);
    assert!(
        realized
            .evaluate_operation_with_burn_clear(&bind(&common, &extension))
            .unwrap()
            .is_conformant()
    );
    let sponsor_owner = OwnerId([7; 32]);
    let source = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 1,
    };
    let destination = ObservedObjectRef {
        side: ObservedSide::Output,
        ordinal: 1,
    };
    for reference in [source, destination] {
        common.claim_object(ObservedObject {
            reference,
            kind: ObservedObjectKind::Declared(ObjectId::PlainLbtc),
            asset: ObservedAsset::Declared(AssetId::Lbtc),
            value: ObservedValue::SponsorOpaque,
            owner: Some(sponsor_owner),
            representation: RepresentationMode::Explicit,
        });
    }
    common.sponsor_signers.insert(sponsor_owner);
    common.region.open_flows.push(ObservedOpenFlow {
        kind: architecture::OpenFlowKind::FeeSponsor,
        sources: vec![source],
        destinations: vec![destination],
        fee: ProtocolAmount::ONE,
    });
    assert!(
        realized
            .evaluate_operation_with_burn_clear(&bind(&common, &extension))
            .unwrap()
            .is_conformant()
    );
    common.sponsor_signers.clear();
    let report = realized
        .evaluate_operation_with_burn_clear(&bind(&common, &extension))
        .unwrap();
    assert_eq!(
        report
            .verdicts
            .iter()
            .find(
                |row| row.relation == rid(RelationKind::SponsorIsolation, RelationSubject::Sponsor)
            )
            .unwrap()
            .status,
        RelationStatus::Failed {
            reason: RelationFailure::SponsorIsolation
        }
    );
}

#[test]
fn burn_refuses_state_and_destruction() {
    let realized = realized();
    let (mut common, extension) = control();
    assert!(
        realized
            .evaluate_operation_with_burn_clear(&bind(&common, &extension))
            .unwrap()
            .is_conformant()
    );
    common.region.canonical_partition.flows[0]
        .destructions
        .push(ObservedDestructionLeg {
            tag: architecture::TagId::Recon,
            amount: ProtocolAmount::ONE,
        });
    let report = realized
        .evaluate_operation_with_burn_clear(&bind(&common, &extension))
        .unwrap();
    assert_eq!(
        report
            .verdicts
            .iter()
            .find(|row| row.relation == op(RelationKind::CanonicalDeltaPolicy))
            .unwrap()
            .status,
        RelationStatus::Failed {
            reason: RelationFailure::CanonicalDeltaPolicy
        }
    );
    let (mut common, extension) = control();
    common.claim_object(ObservedObject {
        reference: ObservedObjectRef {
            side: ObservedSide::Output,
            ordinal: 2,
        },
        kind: ObservedObjectKind::Declared(ObjectId::State),
        asset: ObservedAsset::Declared(AssetId::Pid),
        value: ObservedValue::Protocol(ProtocolAmount::ONE),
        owner: None,
        representation: RepresentationMode::Explicit,
    });
    let report = realized
        .evaluate_operation_with_burn_clear(&bind(&common, &extension))
        .unwrap();
    assert_eq!(
        report
            .verdicts
            .iter()
            .find(|row| row.relation
                == rid(
                    RelationKind::AllowedObjectFamilies,
                    RelationSubject::TransactionSide {
                        side: TransactionSide::Output
                    }
                ))
            .unwrap()
            .status,
        RelationStatus::Failed {
            reason: RelationFailure::UndeclaredObjectFamily
        }
    );
}

#[test]
#[allow(clippy::too_many_lines)] // Keep the independent census and its mutations together.
fn shell_census_has_literal_identities_counts_and_disclosure_result() {
    use RelationKind as K;
    use TransactionSide::{Input, Output};
    let realized = realized();
    let declaration = realized.operations.get(&OperationId::Burn).unwrap();
    let mut expected = BTreeSet::new();
    for (side, object) in [
        (Input, ObjectId::ReceiptLive),
        (Output, ObjectId::ReceiptLive),
        (Output, ObjectId::Ash),
        (Input, ObjectId::PlainLbtc),
        (Output, ObjectId::PlainLbtc),
    ] {
        expected.insert(family(K::Recognition, side, object));
        expected.insert(family(K::Cardinality, side, object));
    }
    for kind in [
        K::OpenFlowPolicy,
        K::CanonicalDeltaPolicy,
        K::RootPolicy,
        K::ProjectionPolicy,
        K::Constructibility,
        K::DataOutputs,
        K::BurnBinding,
    ] {
        expected.insert(op(kind));
    }
    for side in [Input, Output] {
        expected.insert(rid(
            K::AllowedObjectFamilies,
            RelationSubject::TransactionSide { side },
        ));
        expected.insert(rid(
            K::SideRepresentation,
            RelationSubject::SideRepresentation {
                side,
                object: ObjectId::ReceiptLive,
            },
        ));
    }
    expected.extend([
        family(K::Authorization, Input, ObjectId::ReceiptLive),
        rid(
            K::Conservation,
            RelationSubject::Asset { asset: AssetId::U },
        ),
        rid(
            K::SubstrateConservation,
            RelationSubject::Asset {
                asset: AssetId::Lbtc,
            },
        ),
        rid(K::SponsorIsolation, RelationSubject::Sponsor),
        rid(K::SponsorEnvelopeMultiplicity, RelationSubject::Sponsor),
        rid(
            K::Representation,
            RelationSubject::Representation {
                object: ObjectId::Ash,
            },
        ),
    ]);
    for (object, exits) in [
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
    ] {
        for exit in exits {
            expected.insert(rid(
                K::Lifecycle,
                RelationSubject::LifecycleExit { object, exit },
            ));
        }
    }
    assert_eq!(declaration.relations.len(), 32);
    assert_eq!(
        declaration
            .relations
            .iter()
            .map(|row| row.id.clone())
            .collect::<BTreeSet<_>>(),
        expected
    );
    assert_eq!(declaration.expressions, []);
    assert_eq!(declaration.relation_dependencies.len(), 42);
    assert_eq!(declaration.constructibility_nodes.len(), 13);
    assert_eq!(declaration.constructibility_edges.len(), 12);
    assert_eq!(declaration.lifecycle_nodes.len(), 9);
    assert_eq!(declaration.lifecycle_edges.len(), 10);
    assert_eq!(declaration.disclosure_nodes.len(), 15);
    assert_eq!(declaration.disclosure_edges.len(), 16);
    assert_eq!(declaration.disclosure_seeds.len(), 2);
    let private_change = FactId::FamilyAmount {
        operation: OperationId::Burn,
        side: Output,
        object: ObjectId::ReceiptLive,
    };
    assert_eq!(
        realized.declassification.retained_private,
        BTreeSet::from([private_change.clone()])
    );
    assert!(
        !realized
            .declassification
            .required_public
            .contains_key(&private_change)
    );
    assert_eq!(
        realized.declassification.required_public,
        BTreeMap::from([(
            FactId::FamilyAmount {
                operation: OperationId::Burn,
                side: Output,
                object: ObjectId::Ash
            },
            BTreeSet::from([
                DisclosureReason::PublicInterface,
                DisclosureReason::PublicEvent
            ])
        )])
    );
}

#[test]
#[allow(clippy::too_many_lines)] // Independent literal endpoint list.
fn complete_dependency_census_keeps_each_literal_endpoint_and_edge_kind() {
    use RelationEdge as E;
    use RelationKind as K;
    use TransactionSide::{Input, Output};
    let ids = BTreeMap::from([
        ("ri", family(K::Recognition, Input, ObjectId::ReceiptLive)),
        ("ci", family(K::Cardinality, Input, ObjectId::ReceiptLive)),
        ("ro", family(K::Recognition, Output, ObjectId::ReceiptLive)),
        ("co", family(K::Cardinality, Output, ObjectId::ReceiptLive)),
        ("ra", family(K::Recognition, Output, ObjectId::Ash)),
        ("ca", family(K::Cardinality, Output, ObjectId::Ash)),
        ("rsi", family(K::Recognition, Input, ObjectId::PlainLbtc)),
        ("csi", family(K::Cardinality, Input, ObjectId::PlainLbtc)),
        ("rso", family(K::Recognition, Output, ObjectId::PlainLbtc)),
        ("cso", family(K::Cardinality, Output, ObjectId::PlainLbtc)),
        (
            "auth",
            family(K::Authorization, Input, ObjectId::ReceiptLive),
        ),
        (
            "value",
            rid(
                K::Conservation,
                RelationSubject::Asset { asset: AssetId::U },
            ),
        ),
        ("delta", op(K::CanonicalDeltaPolicy)),
        ("binding", op(K::BurnBinding)),
        ("data", op(K::DataOutputs)),
        ("construct", op(K::Constructibility)),
        ("roots", op(K::RootPolicy)),
        ("projections", op(K::ProjectionPolicy)),
        ("open", op(K::OpenFlowPolicy)),
        (
            "sponsor",
            rid(K::SponsorIsolation, RelationSubject::Sponsor),
        ),
        (
            "multiplicity",
            rid(K::SponsorEnvelopeMultiplicity, RelationSubject::Sponsor),
        ),
        (
            "substrate",
            rid(
                K::SubstrateConservation,
                RelationSubject::Asset {
                    asset: AssetId::Lbtc,
                },
            ),
        ),
        (
            "closure",
            rid(
                K::AllowedObjectFamilies,
                RelationSubject::TransactionSide { side: Input },
            ),
        ),
        (
            "input_mode",
            rid(
                K::SideRepresentation,
                RelationSubject::SideRepresentation {
                    side: Input,
                    object: ObjectId::ReceiptLive,
                },
            ),
        ),
        (
            "change_mode",
            rid(
                K::SideRepresentation,
                RelationSubject::SideRepresentation {
                    side: Output,
                    object: ObjectId::ReceiptLive,
                },
            ),
        ),
        (
            "ash_mode",
            rid(
                K::Representation,
                RelationSubject::Representation {
                    object: ObjectId::Ash,
                },
            ),
        ),
        (
            "live_transfer",
            rid(
                K::Lifecycle,
                RelationSubject::LifecycleExit {
                    object: ObjectId::ReceiptLive,
                    exit: OperationId::TransferLive,
                },
            ),
        ),
        (
            "live_burn",
            rid(
                K::Lifecycle,
                RelationSubject::LifecycleExit {
                    object: ObjectId::ReceiptLive,
                    exit: OperationId::Burn,
                },
            ),
        ),
        (
            "live_redeem",
            rid(
                K::Lifecycle,
                RelationSubject::LifecycleExit {
                    object: ObjectId::ReceiptLive,
                    exit: OperationId::Redeem,
                },
            ),
        ),
        (
            "ash_compact",
            rid(
                K::Lifecycle,
                RelationSubject::LifecycleExit {
                    object: ObjectId::Ash,
                    exit: OperationId::CompactAsh,
                },
            ),
        ),
        (
            "ash_clear",
            rid(
                K::Lifecycle,
                RelationSubject::LifecycleExit {
                    object: ObjectId::Ash,
                    exit: OperationId::Clear,
                },
            ),
        ),
    ]);
    let expected = [
        ("input_mode", "binding", E::StaticRequirement),
        ("binding", "data", E::StaticRequirement),
        ("ri", "ci", E::RecognitionBeforeCardinality),
        ("ro", "co", E::RecognitionBeforeCardinality),
        ("ra", "ca", E::RecognitionBeforeCardinality),
        ("rsi", "csi", E::RecognitionBeforeCardinality),
        ("rso", "cso", E::RecognitionBeforeCardinality),
        ("ri", "value", E::RecognitionBeforeValue),
        ("ro", "value", E::RecognitionBeforeValue),
        ("ra", "value", E::RecognitionBeforeValue),
        ("ci", "value", E::CardinalityBeforeValue),
        ("co", "value", E::CardinalityBeforeValue),
        ("ca", "value", E::CardinalityBeforeValue),
        ("ri", "binding", E::StaticRequirement),
        ("ro", "binding", E::StaticRequirement),
        ("ra", "binding", E::StaticRequirement),
        ("ci", "binding", E::StaticRequirement),
        ("co", "binding", E::StaticRequirement),
        ("ca", "binding", E::StaticRequirement),
        ("ro", "change_mode", E::StaticRequirement),
        ("ra", "ash_mode", E::StaticRequirement),
        ("rsi", "sponsor", E::StaticRequirement),
        ("rso", "sponsor", E::StaticRequirement),
        ("delta", "value", E::CanonicalDeltaBeforeValue),
        ("auth", "closure", E::AuthorizationBeforeClosure),
        ("auth", "construct", E::AuthorizationBeforeConstructibility),
        ("auth", "binding", E::StaticRequirement),
        ("value", "binding", E::StaticRequirement),
        ("open", "sponsor", E::OpenFlowPolicyBeforeSponsor),
        ("open", "multiplicity", E::StaticRequirement),
        ("multiplicity", "sponsor", E::StaticRequirement),
        ("sponsor", "construct", E::SponsorBeforeConstructibility),
        ("sponsor", "substrate", E::SponsorBeforeOperation),
        ("roots", "construct", E::RootPolicyBeforeOperation),
        (
            "projections",
            "construct",
            E::ProjectionPolicyBeforeOperation,
        ),
        ("construct", "change_mode", E::StaticRequirement),
        ("construct", "ash_mode", E::StaticRequirement),
        (
            "change_mode",
            "live_transfer",
            E::RepresentationBeforeLifecycle,
        ),
        ("change_mode", "live_burn", E::RepresentationBeforeLifecycle),
        (
            "change_mode",
            "live_redeem",
            E::RepresentationBeforeLifecycle,
        ),
        ("ash_mode", "ash_compact", E::RepresentationBeforeLifecycle),
        ("ash_mode", "ash_clear", E::RepresentationBeforeLifecycle),
    ]
    .into_iter()
    .map(|(source, target, edge)| RelationDependencyDeclaration {
        prerequisite: ids[source].clone(),
        dependent: ids[target].clone(),
        edge,
    })
    .collect::<BTreeSet<_>>();
    let declaration = crate::declarations::burn::derive(&ARCHITECTURE).unwrap();
    assert_eq!(
        declaration
            .relation_dependencies
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>(),
        expected
    );
    for (index, _) in declaration.relation_dependencies.iter().enumerate() {
        let mut omitted = declaration.clone();
        omitted.relation_dependencies.remove(index);
        assert_eq!(
            crate::validate::validate_burn_declaration(&omitted),
            Err(census_mismatch(RelationKind::BurnBinding))
        );
    }
}

#[test]
fn every_object_coordinate_is_welded_for_ash_and_live_receipts() {
    let changes: [fn(&mut architecture::ObjectSpec); 8] = [
        |row| {
            row.asset = AssetId::Lbtc;
        },
        |row| {
            row.lifecycle = architecture::LifecycleClass::ExternalWallet;
        },
        |row| {
            row.accounting_domain = architecture::AccountingDomain::None;
        },
        |row| {
            row.consensus_value_authoritative = false;
        },
        |row| {
            let mut values = row.allocators.to_vec();
            values[0] = architecture::AllocatorId::External;
            row.allocators = values.leak();
        },
        |row| {
            let mut values = row.mutators.to_vec();
            values[0] = OperationId::CreateRequest;
            row.mutators = values.leak();
        },
        |row| {
            let mut values = row.deallocators.to_vec();
            values[0] = architecture::DeallocatorId::ExternalSpend;
            row.deallocators = values.leak();
        },
        |row| {
            let mut values = row.witnesses.to_vec();
            values[0] = architecture::WitnessId::FloorNondecrease;
            row.witnesses = values.leak();
        },
    ];
    crate::declarations::burn::derive(&ARCHITECTURE).unwrap();
    for (object, field) in [
        (ObjectId::Ash, ArchitectureMismatchField::AshOutput),
        (
            ObjectId::ReceiptLive,
            ArchitectureMismatchField::ReceiptOutput,
        ),
    ] {
        for change in changes {
            let mut architecture = ARCHITECTURE;
            let mut objects = architecture.objects.to_vec();
            change(objects.iter_mut().find(|row| row.id == object).unwrap());
            architecture.objects = objects.leak();
            assert_eq!(
                crate::declarations::burn::derive(&architecture).unwrap_err(),
                mismatch(field)
            );
        }
    }
}

#[test]
fn overlapping_burn_pilots_have_literal_unions() {
    realized();
    for (pilot, objects) in [
        (
            OperationId::CompactAsh,
            &[ObjectId::Ash, ObjectId::ReceiptLive][..],
        ),
        (
            OperationId::TransferLive,
            &[ObjectId::Ash, ObjectId::ReceiptLive][..],
        ),
    ] {
        let scope = RealizationScope::from_operations([OperationId::Burn, pilot]).unwrap();
        let mixed = derive(&ARCHITECTURE, scope).unwrap();
        mixed.validate_against(&ARCHITECTURE).unwrap();
        super::derivation_tests::assert_literal_lifecycle(&mixed, objects);
        super::derivation_tests::assert_pair_disclosure_union(&mixed, [OperationId::Burn, pilot]);
        assert_eq!(mixed.operations.len(), 2);
        assert!(mixed.lifecycle_graph.edge_indices().any(|edge| {
            let (source, target) = mixed.lifecycle_graph.edge_endpoints(edge).unwrap();
            mixed.lifecycle_graph[source].id
                == LifecycleNodeId::Representation {
                    object: ObjectId::ReceiptLive,
                    mode: RepresentationMode::PrivateCommitted,
                }
                && mixed.lifecycle_graph[target].id
                    == LifecycleNodeId::RequiredExit {
                        object: ObjectId::ReceiptLive,
                        operation: OperationId::Burn,
                    }
        }));
        let mut expected_private = BTreeSet::from([FactId::FamilyAmount {
            operation: OperationId::Burn,
            side: TransactionSide::Output,
            object: ObjectId::ReceiptLive,
        }]);
        if pilot == OperationId::TransferLive {
            for side in [TransactionSide::Input, TransactionSide::Output] {
                expected_private.insert(FactId::FamilyAmount {
                    operation: OperationId::TransferLive,
                    side,
                    object: ObjectId::ReceiptLive,
                });
            }
        }
        assert_eq!(mixed.declassification.retained_private, expected_private);
    }
}

#[test]
#[allow(clippy::too_many_lines)] // Literal constructor facts and witnesses.
fn constructor_facts_and_witnesses_are_exact_and_public_ash_has_no_creator_secret() {
    use TransactionSide::{Input, Output};
    let public = AvailabilityClass::Public;
    let owner = AvailabilityClass::InputOwners {
        object: ObjectId::ReceiptLive,
    };
    let expected = [
        (
            FactId::FamilyCount {
                operation: OperationId::Burn,
                side: Input,
                object: ObjectId::ReceiptLive,
            },
            public,
        ),
        (
            FactId::FamilyCount {
                operation: OperationId::Burn,
                side: Output,
                object: ObjectId::ReceiptLive,
            },
            public,
        ),
        (
            FactId::FamilyCount {
                operation: OperationId::Burn,
                side: Output,
                object: ObjectId::Ash,
            },
            public,
        ),
        (
            FactId::FamilyAmount {
                operation: OperationId::Burn,
                side: Input,
                object: ObjectId::ReceiptLive,
            },
            public,
        ),
        (
            FactId::FamilyAmount {
                operation: OperationId::Burn,
                side: Output,
                object: ObjectId::ReceiptLive,
            },
            owner,
        ),
        (
            FactId::FamilyAmount {
                operation: OperationId::Burn,
                side: Output,
                object: ObjectId::Ash,
            },
            public,
        ),
        (
            FactId::BoundValue {
                bound: BoundId::BurnInputMax,
            },
            public,
        ),
        (
            FactId::BoundValue {
                bound: BoundId::BurnChangeMax,
            },
            public,
        ),
        (
            FactId::BoundValue {
                bound: BoundId::BurnRecordMax,
            },
            public,
        ),
        (
            FactId::BoundValue {
                bound: BoundId::FeeSponsorInputMax,
            },
            public,
        ),
    ];
    let operation = ConstructibilityNodeId::Operation(OperationId::Burn);
    let mut nodes = BTreeSet::from([operation.clone()]);
    let mut edges = BTreeSet::new();
    for (fact, availability) in expected {
        let source = ConstructibilityNodeId::Fact {
            operation: OperationId::Burn,
            fact,
            availability,
        };
        nodes.insert(source.clone());
        edges.insert(ConstructibilityDependencyDeclaration {
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
            owner,
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
    let declaration = crate::declarations::burn::derive(&ARCHITECTURE).unwrap();
    assert_eq!(
        declaration
            .constructibility_nodes
            .iter()
            .map(|node| node.id.clone())
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
    let mut secret = declaration;
    secret.constructibility_nodes.push(ConstructibilityNode {
        id: ConstructibilityNodeId::Witness {
            operation: OperationId::Burn,
            role: WitnessRole::OperatorAuthorization,
            availability: AvailabilityClass::Operator,
        },
    });
    assert_eq!(
        crate::validate::validate_burn_declaration(&secret),
        Err(census_mismatch(RelationKind::Constructibility))
    );
}

#[test]
fn every_shell_requirement_refuses_omission_and_permutation_is_accepted() {
    use RelationKind as K;
    type Change = (K, fn(&mut OperationRealization));
    let declaration = crate::declarations::burn::derive(&ARCHITECTURE).unwrap();
    crate::validate::validate_burn_declaration(&declaration).unwrap();
    for removed in &declaration.relations {
        let mut subject = declaration.clone();
        subject.relations.retain(|row| row.id != removed.id);
        assert_eq!(
            crate::validate::validate_burn_declaration(&subject),
            Err(RealizationError::MissingArchitectureRelation {
                operation: OperationId::Burn,
                kind: removed.id.kind(),
                subject: removed.id.subject().clone(),
            })
        );
        let mut duplicate = declaration.clone();
        duplicate.relations.push(removed.clone());
        assert_eq!(
            crate::validate::validate_burn_declaration(&duplicate),
            Err(RealizationError::SurplusArchitectureRelation {
                relation: removed.id.clone()
            })
        );
    }
    let removals: [Change; 8] = [
        (K::BurnBinding, |row| {
            row.relation_dependencies.pop();
        }),
        (K::Constructibility, |row| {
            row.constructibility_nodes.pop();
        }),
        (K::Constructibility, |row| {
            row.constructibility_edges.pop();
        }),
        (K::Representation, |row| {
            row.lifecycle_nodes.pop();
        }),
        (K::Representation, |row| {
            row.lifecycle_edges.pop();
        }),
        (K::Representation, |row| {
            row.disclosure_nodes.pop();
        }),
        (K::Representation, |row| {
            row.disclosure_edges.pop();
        }),
        (K::Representation, |row| {
            row.disclosure_seeds.pop();
        }),
    ];
    for (field, remove) in removals {
        let mut subject = declaration.clone();
        remove(&mut subject);
        assert_eq!(
            crate::validate::validate_burn_declaration(&subject),
            Err(census_mismatch(field))
        );
    }
    let mut permuted = declaration;
    permuted.relations.reverse();
    permuted.relation_dependencies.reverse();
    permuted.constructibility_nodes.reverse();
    permuted.constructibility_edges.reverse();
    permuted.lifecycle_nodes.reverse();
    permuted.lifecycle_edges.reverse();
    permuted.disclosure_nodes.reverse();
    permuted.disclosure_edges.reverse();
    permuted.disclosure_seeds.reverse();
    let scope = RealizationScope::from_operations([OperationId::Burn]).unwrap();
    let actual = crate::derive::assemble_scoped_realization(
        &ARCHITECTURE,
        ArchitectureBinding::from_architecture(&ARCHITECTURE).unwrap(),
        scope,
        BTreeMap::from([(OperationId::Burn, permuted)]),
    )
    .unwrap();
    assert_eq!(
        project_scoped_realization(&actual),
        project_scoped_realization(&realized())
    );
}

#[test]
fn coherent_removal_of_public_ash_constructor_requirement_refuses_at_assembly() {
    let mut declaration = crate::declarations::burn::derive(&ARCHITECTURE).unwrap();
    let fact = FactId::FamilyAmount {
        operation: OperationId::Burn,
        side: TransactionSide::Output,
        object: ObjectId::Ash,
    };
    let node = ConstructibilityNodeId::Fact {
        operation: OperationId::Burn,
        fact,
        availability: AvailabilityClass::Public,
    };
    declaration
        .constructibility_nodes
        .retain(|row| row.id != node);
    declaration
        .constructibility_edges
        .retain(|row| row.source != node);
    let error = crate::derive::assemble_scoped_realization(
        &ARCHITECTURE,
        ArchitectureBinding::from_architecture(&ARCHITECTURE).unwrap(),
        RealizationScope::from_operations([OperationId::Burn]).unwrap(),
        BTreeMap::from([(OperationId::Burn, declaration)]),
    )
    .unwrap_err();
    assert_eq!(error, census_mismatch(RelationKind::Constructibility));
}

#[test]
fn same_length_graph_substitutions_refuse_at_the_complete_census() {
    use RelationKind as K;
    type Change = (K, fn(&mut OperationRealization));
    let declaration = crate::declarations::burn::derive(&ARCHITECTURE).unwrap();
    crate::validate::validate_burn_declaration(&declaration).unwrap();
    let substitutions: [Change; 8] = [
        (K::BurnBinding, |row| {
            row.relation_dependencies[0] = row.relation_dependencies[1].clone();
        }),
        (K::Constructibility, |row| {
            row.constructibility_nodes[0] = row.constructibility_nodes[1].clone();
        }),
        (K::Constructibility, |row| {
            row.constructibility_edges[0] = row.constructibility_edges[1].clone();
        }),
        (K::Representation, |row| {
            row.lifecycle_nodes[0] = row.lifecycle_nodes[1].clone();
        }),
        (K::Representation, |row| {
            row.lifecycle_edges[0] = row.lifecycle_edges[1].clone();
        }),
        (K::Representation, |row| {
            row.disclosure_nodes[0] = row.disclosure_nodes[1].clone();
        }),
        (K::Representation, |row| {
            row.disclosure_edges[0].edge = DisclosureEdge::RelationOperand;
        }),
        (K::Representation, |row| {
            row.disclosure_seeds[0].reason = DisclosureReason::PublicState;
        }),
    ];
    for (field, substitute) in substitutions {
        let mut subject = declaration.clone();
        substitute(&mut subject);
        assert_eq!(
            crate::validate::validate_burn_declaration(&subject),
            Err(census_mismatch(field))
        );
    }
}

fn assemble_burn(
    declaration: OperationRealization,
) -> Result<ScopedRealizationSpec, RealizationError> {
    crate::derive::assemble_scoped_realization(
        &ARCHITECTURE,
        ArchitectureBinding::from_architecture(&ARCHITECTURE).unwrap(),
        RealizationScope::from_operations([OperationId::Burn]).unwrap(),
        BTreeMap::from([(OperationId::Burn, declaration)]),
    )
}

#[test]
fn coherent_removal_of_the_live_burn_exit_refuses_at_assembly() {
    let mut declaration = crate::declarations::burn::derive(&ARCHITECTURE).unwrap();
    assemble_burn(declaration.clone()).unwrap();
    let subject = RelationSubject::LifecycleExit {
        object: ObjectId::ReceiptLive,
        exit: OperationId::Burn,
    };
    let relation = rid(RelationKind::Lifecycle, subject.clone());
    let exit = LifecycleNodeId::RequiredExit {
        object: ObjectId::ReceiptLive,
        operation: OperationId::Burn,
    };
    declaration.relations.retain(|row| row.id != relation);
    declaration
        .relation_dependencies
        .retain(|row| row.dependent != relation);
    declaration.lifecycle_nodes.retain(|row| row.id != exit);
    // Both Explicit and PrivateCommitted paths disappear with their requirement.
    declaration.lifecycle_edges.retain(|row| row.target != exit);
    assert_eq!(
        assemble_burn(declaration).unwrap_err(),
        RealizationError::MissingArchitectureRelation {
            operation: OperationId::Burn,
            kind: RelationKind::Lifecycle,
            subject,
        }
    );
}

#[test]
fn coherent_removal_of_a_disclosure_fact_refuses_at_assembly() {
    let mut declaration = crate::declarations::burn::derive(&ARCHITECTURE).unwrap();
    assemble_burn(declaration.clone()).unwrap();
    let node = DisclosureNodeId::Fact(FactId::FamilyAmount {
        operation: OperationId::Burn,
        side: TransactionSide::Output,
        object: ObjectId::ReceiptLive,
    });
    declaration.disclosure_nodes.retain(|row| row.id() != node);
    declaration
        .disclosure_edges
        .retain(|row| row.source != node && row.target != node);
    assert_eq!(
        assemble_burn(declaration).unwrap_err(),
        census_mismatch(RelationKind::Representation)
    );
}

#[test]
fn inserted_expression_refuses_at_the_complete_burn_census() {
    let mut declaration = crate::declarations::burn::derive(&ARCHITECTURE).unwrap();
    assemble_burn(declaration.clone()).unwrap();
    let fact = FactId::BoundValue {
        bound: BoundId::BurnInputMax,
    };
    declaration.expressions.push(ExpressionDeclaration {
        id: ExprId::fact(fact.clone()),
        ty: SemanticType::Count,
        node: ExpressionNode::Fact(fact),
    });
    assert_eq!(
        assemble_burn(declaration).unwrap_err(),
        census_mismatch(RelationKind::BurnBinding)
    );
}

fn altered(change: impl FnOnce(&mut architecture::OperationSpec)) -> architecture::Architecture {
    let mut architecture = ARCHITECTURE;
    let mut operations = architecture.operations.to_vec();
    change(
        operations
            .iter_mut()
            .find(|row| row.id == OperationId::Burn)
            .unwrap(),
    );
    architecture.operations = operations.leak();
    architecture
}

#[test]
#[allow(clippy::too_many_lines)] // Retain each weld coordinate and its exact error together.
fn architecture_weld_refuses_each_changed_coordinate_and_duplicates() {
    use ArchitectureMismatchField as F;
    type Change = (F, fn(&mut architecture::OperationSpec));
    let cases: [Change; 18] = [
        (F::OperationKind, |row| {
            row.kind = architecture::OperationKind::ClientProtocol;
        }),
        (F::RootPolicy, |row| {
            row.roots = vec![architecture::RootUseSpec {
                root: architecture::RootId::State,
                use_kind: architecture::RootUse::Succession,
            }]
            .leak();
        }),
        (F::Reads, |row| {
            row.reads = vec![architecture::QuantityId::Floor].leak();
        }),
        (F::Writes, |row| {
            row.writes = vec![architecture::QuantityId::Floor].leak();
        }),
        (F::Issuances, |row| {
            row.issuances = vec![architecture::IssuanceSpec {
                asset: AssetId::U,
                authority: AssetId::U,
                condition: architecture::IssuanceCondition::PositiveAdmittedPrincipal,
            }]
            .leak();
        }),
        (F::Authorization, |row| {
            row.authorization = architecture::PermissionClass::Permissionless;
        }),
        (F::InputFamilies, |row| {
            let mut rows = row.inputs.to_vec();
            rows[0].minimum = 0;
            row.inputs = rows.leak();
        }),
        (F::InputFamilies, |row| {
            let mut rows = row.inputs.to_vec();
            rows[0].authorization = architecture::InputAuthorization::Permissionless;
            row.inputs = rows.leak();
        }),
        (F::OutputFamilies, |row| {
            let mut rows = row.outputs.to_vec();
            rows[0].maximum = architecture::MaxCount::Exact(2);
            row.outputs = rows.leak();
        }),
        (F::CanonicalDelta, |row| {
            let mut rows = row.canonical_deltas.to_vec();
            rows[0].kind = architecture::DeltaKind::OwnerlessLateral;
            row.canonical_deltas = rows.leak();
        }),
        (F::DataOutputs, |row| {
            let mut rows = row.data_outputs.to_vec();
            rows[0].tag = architecture::TagId::Recon;
            row.data_outputs = rows.leak();
        }),
        (F::OpenFlows, |row| {
            let mut values = row.open_flows.to_vec();
            values[0] = architecture::OpenFlowKind::Redemption;
            row.open_flows = values.leak();
        }),
        (F::ValueFlows, |row| {
            let mut values = row.value_flows.to_vec();
            values[0] = architecture::ValueFlowClass::FormulaBoundPayout;
            row.value_flows = values.leak();
        }),
        (F::Witnesses, |row| {
            row.witnesses = row.witnesses[..5].to_vec().leak();
        }),
        (F::Bounds, |row| {
            let mut values = row.bounds.to_vec();
            values[0] = BoundId::AshBatchMax;
            row.bounds = values.leak();
        }),
        (F::ProjectionPolicy, |row| {
            let mut values = row.projections.to_vec();
            values
                .iter_mut()
                .find(|value| value.projection == architecture::ProjectionId::BurnEvent)
                .unwrap()
                .rule = architecture::ProjectionRule::Optional;
            row.projections = values.leak();
        }),
        (F::InputFamilies, |row| {
            let mut rows = row.inputs.to_vec();
            rows.push(rows[0]);
            row.inputs = rows.leak();
        }),
        (F::Witnesses, |row| {
            let mut rows = row.witnesses.to_vec();
            rows.push(rows[0]);
            row.witnesses = rows.leak();
        }),
    ];
    crate::validate::validate_burn_architecture(&ARCHITECTURE).unwrap();
    for (field, change) in cases {
        assert_eq!(
            crate::declarations::burn::derive(&altered(change)).unwrap_err(),
            mismatch(field)
        );
    }
    let permuted = altered(|row| {
        let mut inputs = row.inputs.to_vec();
        inputs.reverse();
        row.inputs = inputs.leak();
        let mut outputs = row.outputs.to_vec();
        outputs.reverse();
        row.outputs = outputs.leak();
        let mut witnesses = row.witnesses.to_vec();
        witnesses.reverse();
        row.witnesses = witnesses.leak();
    });
    crate::declarations::burn::derive(&permuted).unwrap();
}
