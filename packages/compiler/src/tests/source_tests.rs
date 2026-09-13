//! Operand, capability, and source-requirement tests (Guide-3 §17.1–2).

use std::collections::BTreeSet;

use architecture::{ObjectId, OperationId};
use realization::{
    AvailabilityClass, ProofKind, Relation, RelationDeclaration, RelationId, RelationKind,
    RelationSubject,
};

use architecture::{
    ObjectId::{PlainLbtc, State},
    OperationId::AnnounceMaturity,
};

use super::{announcement_input, bound_input};
use crate::{
    CompileError,
    capability::{CapabilityView, RequiredCapability},
    relation::build_relation_analysis,
    source::{
        OperandRole, RequiredSourceKind, RequirementActivation, derive_source_requirements,
        is_sponsor_amount_operand, relation_operands,
    },
};

fn declaration(
    operation: OperationId,
    kind: RelationKind,
    relation: Relation,
) -> RelationDeclaration {
    RelationDeclaration {
        id: RelationId::new(operation, kind, RelationSubject::Operation),
        relation,
        proof_alternatives: BTreeSet::new(),
    }
}

// --- capability view (§17.1) ---

#[test]
fn capability_views_prune_fail_closed() {
    let required = BTreeSet::from([
        RequiredCapability::ExactPublicAmountArithmetic,
        RequiredCapability::AuthenticatedObjectRecognition,
    ]);

    assert!(CapabilityView::Unconstrained.supports(&required));
    assert!(CapabilityView::Available(required.clone()).supports(&required));
    assert!(
        CapabilityView::Available(BTreeSet::from([
            RequiredCapability::ExactPublicAmountArithmetic,
            RequiredCapability::AuthenticatedObjectRecognition,
            RequiredCapability::OwnerAuthorization,
        ]))
        .supports(&required)
    );
    // A missing capability rejects; nothing is invented.
    assert!(
        !CapabilityView::Available(BTreeSet::from([
            RequiredCapability::ExactPublicAmountArithmetic,
        ]))
        .supports(&required)
    );
    assert!(!CapabilityView::Available(BTreeSet::new()).supports(&required));
}

// --- operand census over the real pilots (§17.2) ---

#[test]
fn every_pilot_relation_has_a_complete_sourced_operand_census() {
    let input = bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let analysis = build_relation_analysis(&input).expect("relations");

    for node in analysis.project().nodes {
        let operands = relation_operands(&node.source).expect("operands derive");
        assert!(
            !operands.is_empty(),
            "relation {:?} has no operands",
            node.source.id
        );

        // Every approved proof alternative yields one authenticated
        // source row per operand, canonically sorted.
        for alternative in &node.source.proof_alternatives {
            let rows =
                derive_source_requirements(&node.source, alternative.proof()).expect("sources");

            assert_eq!(rows.len(), operands.len(), "{:?}", node.source.id);
            assert!(rows.is_sorted());

            for row in &rows {
                assert!(operands.contains(&row.operand));
                assert!(!is_sponsor_amount_operand(row.operand.role()));
            }
        }
    }
}

#[test]
fn conservation_proofs_classify_amount_sources_by_proof() {
    let conservation = declaration(
        OperationId::TransferLive,
        RelationKind::Conservation,
        Relation::AmountConservation {
            asset: architecture::AssetId::U,
            input_objects: BTreeSet::from([ObjectId::ReceiptLive]),
            output_objects: BTreeSet::from([ObjectId::ReceiptLive]),
        },
    );

    // Public arithmetic requires public exact amounts…
    let public = derive_source_requirements(&conservation, ProofKind::PublicArithmetic).unwrap();
    assert!(public.iter().all(|row| {
        !matches!(row.operand.role(), OperandRole::ObjectFamilyAmount { .. })
            || row.source == RequiredSourceKind::AuthenticatedConsensusValue
    }));

    // …while confidential conservation requires commitment relations,
    // never exact public amounts.
    let confidential =
        derive_source_requirements(&conservation, ProofKind::ConfidentialConservation).unwrap();
    assert!(confidential.iter().all(|row| {
        !matches!(row.operand.role(), OperandRole::ObjectFamilyAmount { .. })
            || row.source == RequiredSourceKind::AuthenticatedCommitmentRelation
    }));
    assert!(
        confidential
            .iter()
            .all(|row| row.source != RequiredSourceKind::AuthenticatedConsensusValue)
    );
}

#[test]
fn authorization_sources_match_their_class() {
    // Owner authorization requires the owner census and an input-owner
    // witness of the exact family.
    let owner = declaration(
        OperationId::TransferLive,
        RelationKind::Authorization,
        Relation::OwnerAuthorization {
            object: ObjectId::ReceiptLive,
        },
    );
    let rows = derive_source_requirements(&owner, ProofKind::SignerMembership).unwrap();
    assert!(rows.iter().any(|row| {
        row.source == RequiredSourceKind::InputOwnerWitness
            && row.availability
                == AvailabilityClass::InputOwners {
                    object: ObjectId::ReceiptLive,
                }
    }));

    // Permissionless authorization requires no private witness at all.
    let permissionless = declaration(
        OperationId::CompactAsh,
        RelationKind::Authorization,
        Relation::PermissionlessAuthorization,
    );
    let rows =
        derive_source_requirements(&permissionless, ProofKind::PublicConstructibility).unwrap();
    assert!(
        rows.iter()
            .all(|row| row.availability == AvailabilityClass::Public),
        "permissionless authorization must not require a private witness",
    );
}

#[test]
fn sponsor_isolation_reads_no_sponsor_amount() {
    let sponsor = declaration(
        OperationId::CompactAsh,
        RelationKind::SponsorIsolation,
        Relation::SponsorIsolation,
    );
    let operands = relation_operands(&sponsor).unwrap();

    assert!(
        operands
            .iter()
            .all(|operand| { !matches!(operand.role(), OperandRole::ObjectFamilyAmount { .. }) })
    );

    // The sponsor witness stays sponsor-local and sponsor-activated.
    let rows = derive_source_requirements(&sponsor, ProofKind::ManifestShape).unwrap();
    let witness = rows
        .iter()
        .find(|row| row.source == RequiredSourceKind::SponsorLocalWitness)
        .expect("sponsor witness row");
    assert_eq!(witness.availability, AvailabilityClass::SponsorLocal);
    assert_eq!(
        witness.activation,
        RequirementActivation::WhenSponsorPresent
    );
}

#[test]
fn a_sponsor_amount_operand_is_structurally_rejected() {
    let poisoned = declaration(
        OperationId::CompactAsh,
        RelationKind::Conservation,
        Relation::AmountConservation {
            asset: architecture::AssetId::Lbtc,
            input_objects: BTreeSet::from([ObjectId::PlainLbtc]),
            output_objects: BTreeSet::from([ObjectId::PlainLbtc]),
        },
    );

    assert_eq!(
        relation_operands(&poisoned).unwrap_err(),
        CompileError::SponsorValueRead,
    );
}

#[test]
fn evidence_bounds_and_predicates_stay_typed() {
    // External substrate conservation remains a typed evidence operand.
    let substrate = declaration(
        OperationId::CompactAsh,
        RelationKind::SubstrateConservation,
        Relation::SubstrateConservation {
            asset: architecture::AssetId::Lbtc,
        },
    );
    let rows = derive_source_requirements(&substrate, ProofKind::SubstrateConservation).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].source, RequiredSourceKind::ExternalEvidence);
    assert!(matches!(
        rows[0].operand.role(),
        OperandRole::ExternalEvidence { .. }
    ));

    // Runtime bounds remain architecture-owned.
    let cardinality = declaration(
        OperationId::CompactAsh,
        RelationKind::Cardinality,
        Relation::Cardinality {
            side: realization::ObservedSide::Input,
            object: ObjectId::Ash,
            minimum: realization::Count::new(2),
            maximum: realization::CardinalityMaximum::Bound(architecture::BoundId::AshBatchMax),
        },
    );
    let rows = derive_source_requirements(&cardinality, ProofKind::ManifestShape).unwrap();
    assert!(rows.iter().any(|row| {
        row.source == RequiredSourceKind::RuntimeArchitectureBound
            && matches!(
                row.operand.role(),
                OperandRole::RuntimeBound {
                    bound: architecture::BoundId::AshBatchMax,
                }
            )
    }));

    // Expression predicates retain the exact source expression.
    let expression = realization::ExprId::relation(
        RelationId::new(
            OperationId::CompactAsh,
            RelationKind::CanonicalDeltaPolicy,
            RelationSubject::Operation,
        ),
        realization::ExpressionRole::Predicate,
    );
    let predicate = declaration(
        OperationId::CompactAsh,
        RelationKind::CanonicalDeltaPolicy,
        Relation::ExpressionPredicate {
            expression: expression.clone(),
        },
    );
    let operands = relation_operands(&predicate).unwrap();
    assert!(operands.iter().any(|operand| {
        matches!(
            operand.role(),
            OperandRole::ExpressionResult { expression: e } if *e == expression
        )
    }));
}

#[test]
fn operator_authorization_requires_an_external_evidence_operand() {
    let declaration = declaration(
        OperationId::AnnounceMaturity,
        RelationKind::Authorization,
        Relation::OperatorAuthorization,
    );
    assert_operator_source(&declaration);
    let operands = relation_operands(&declaration).unwrap();
    assert_eq!(operands.len(), 1);
    assert_eq!(
        operands[0].role(),
        &OperandRole::ExternalEvidence {
            requirement: realization::ExternalEvidenceRequirement::OperatorAuthorization {
                operation: OperationId::AnnounceMaturity
            }
        }
    );
}

#[test]
fn operator_constructibility_requires_an_external_evidence_operand() {
    let declaration = declaration(
        OperationId::AnnounceMaturity,
        RelationKind::Constructibility,
        Relation::Constructibility {
            class: realization::ConstructibilityClass::Operator,
        },
    );
    assert_operator_source(&declaration);
    let operands = relation_operands(&declaration).unwrap();
    assert_eq!(operands.len(), 1);
    assert_eq!(
        operands[0].role(),
        &OperandRole::ExternalEvidence {
            requirement: realization::ExternalEvidenceRequirement::OperatorAuthorization {
                operation: OperationId::AnnounceMaturity
            }
        }
    );
}

fn assert_operator_source(declaration: &RelationDeclaration) {
    assert_eq!(
        crate::source::proof_capabilities(declaration, ProofKind::ManifestShape),
        BTreeSet::from([
            RequiredCapability::AuthenticatedObjectRecognition,
            RequiredCapability::OperatorAuthorization
        ]),
    );
    assert_eq!(
        derive_source_requirements(declaration, ProofKind::ManifestShape).unwrap(),
        vec![crate::source::SourceRequirement {
            operand: crate::source::OperandId::new(
                declaration.id.clone(),
                OperandRole::ExternalEvidence {
                    requirement: realization::ExternalEvidenceRequirement::OperatorAuthorization {
                        operation: OperationId::AnnounceMaturity
                    },
                }
            ),
            source: RequiredSourceKind::ExternalEvidence,
            availability: AvailabilityClass::Public,
            activation: RequirementActivation::Always,
        }]
    );
}

fn announcement_retained_operands(id: &RelationId) -> Vec<OperandRole> {
    use OperandRole::{
        CanonicalPartition, ExternalEvidence, ObjectFamilyCount, ObjectFamilyMembers, OpenFlowSet,
        ProjectionSet, RootEffects, RuntimeBound, SponsorRegion, SponsorSignerSet,
    };
    use RelationKind::{
        AllowedObjectFamilies, Authorization, CanonicalDeltaPolicy, Cardinality, Constructibility,
        Lifecycle, OpenFlowPolicy, ProjectionPolicy, Recognition, RootPolicy,
        SponsorEnvelopeMultiplicity, SponsorIsolation, SubstrateConservation,
    };
    use architecture::{AssetId, BoundId};
    use realization::{ExternalEvidenceRequirement as Evidence, TransactionSide::Input};
    let operation = AnnounceMaturity;
    match (id.kind(), id.subject()) {
        (Cardinality | Recognition, RelationSubject::ObjectFamily { side, object }) => {
            assert!([State, PlainLbtc].contains(object));
            let (side, object) = (*side, *object);
            let mut roles = if id.kind() == Cardinality {
                vec![ObjectFamilyCount { side, object }]
            } else {
                vec![ObjectFamilyMembers { side, object }]
            };
            if (id.kind(), side, object) == (Cardinality, Input, PlainLbtc) {
                roles.push(RuntimeBound {
                    bound: BoundId::FeeSponsorInputMax,
                });
            }
            roles
        }
        (AllowedObjectFamilies, RelationSubject::TransactionSide { side }) => [State, PlainLbtc]
            .map(|object| ObjectFamilyMembers {
                side: *side,
                object,
            })
            .into(),
        (Authorization | Constructibility, RelationSubject::Operation) => vec![ExternalEvidence {
            requirement: Evidence::OperatorAuthorization { operation },
        }],
        (
            SubstrateConservation,
            RelationSubject::Asset {
                asset: AssetId::Lbtc,
            },
        ) => vec![ExternalEvidence {
            requirement: Evidence::SubstrateConservation {
                operation,
                asset: AssetId::Lbtc,
            },
        }],
        (SponsorIsolation, RelationSubject::Sponsor) => vec![SponsorRegion, SponsorSignerSet],
        (SponsorEnvelopeMultiplicity, RelationSubject::Sponsor)
        | (OpenFlowPolicy, RelationSubject::Operation) => vec![OpenFlowSet],
        (CanonicalDeltaPolicy, RelationSubject::Operation) => vec![CanonicalPartition],
        (RootPolicy, RelationSubject::Operation) => vec![RootEffects],
        (ProjectionPolicy, RelationSubject::Operation) => vec![ProjectionSet],
        (RelationKind::Representation, RelationSubject::Representation { object: State })
        | (Lifecycle, RelationSubject::LifecycleExit { object: State, .. }) => vec![],
        other => panic!("unexpected announcement relation {other:?}"),
    }
}

#[test]
fn announcement_sources_pin_retention_and_vacuous_bound_erasure() {
    use crate::{
        analyzed_operation::analyze_operation,
        case::{SponsorCase, execution_cases},
        constructibility::build_constructibility_analysis,
        placement::{DischargeBoundary, PlacementSearchLimits},
        proof::enumerate_feasible_plans,
        requirement::derive_relation_requirements,
    };
    let input = announcement_input();
    let relations = build_relation_analysis(&input).unwrap();
    let construction = build_constructibility_analysis(&input).unwrap();
    let plans = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained).unwrap();
    for candidate in plans.candidates {
        assert_eq!(candidate.source_requirements.len(), 23);
        let owned = derive_relation_requirements(&relations, &construction, &candidate).unwrap();
        assert_eq!(owned.len(), 26);
        for (id, rows) in &owned {
            let actual: BTreeSet<_> = rows
                .source_requirements
                .iter()
                .map(|row| row.operand.role().clone())
                .collect();
            assert_eq!(
                actual,
                announcement_retained_operands(id).into_iter().collect(),
                "{id:?}"
            );
        }
        let cases = execution_cases(&relations, &candidate).unwrap();
        let absent = cases
            .iter()
            .find(|case| case.id.sponsor == SponsorCase::Absent)
            .unwrap();
        assert_eq!(absent.active_sources.len(), 15);
        let bound = absent
            .active_sources
            .iter()
            .find(|row| matches!(row.operand.role(), OperandRole::RuntimeBound { .. }))
            .unwrap();
        assert_eq!(bound.source, RequiredSourceKind::RuntimeArchitectureBound);
        assert_eq!(bound.activation, RequirementActivation::Always);
        let limits = PlacementSearchLimits::new(
            10_000_000_u64.try_into().unwrap(),
            1_000_000_u64.try_into().unwrap(),
        );
        let analyzed =
            analyze_operation(&relations, &owned, &candidate, AnnounceMaturity, limits).unwrap();
        let rows: Vec<_> = analyzed
            .relation_cases
            .values()
            .filter(|row| row.key.case.sponsor == SponsorCase::Absent)
            .collect();
        assert_eq!(
            rows.iter()
                .map(|row| row.active_sources.len())
                .sum::<usize>(),
            14
        );
        assert_eq!(
            rows.iter()
                .filter(|row| row.boundaries.contains(&DischargeBoundary::RuntimeCarrier))
                .map(|row| row.active_sources.len())
                .sum::<usize>(),
            11
        );
        assert!(rows.iter().all(|row| !row.active_sources.contains(bound)));
    }
}
