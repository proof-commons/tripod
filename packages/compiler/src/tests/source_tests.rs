//! Operand, capability, and source-requirement tests (Guide-3 §17.1–2).

use std::collections::BTreeSet;

use architecture::{ObjectId, OperationId};
use realization::{
    AvailabilityClass, ProofKind, Relation, RelationDeclaration, RelationId, RelationKind,
    RelationSubject,
};

use super::bound_input;
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
