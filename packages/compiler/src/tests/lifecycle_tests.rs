//! Representation and lifecycle tests (Guide-3 §17.4).

use architecture::{ObjectId, OperationId};
use realization::{ProofKind, RepresentationMode};

use super::bound_input;
use crate::{
    lifecycle::{
        LifecycleExitStatus, RepresentationChoiceId, build_lifecycle_analysis,
        proof_supports_representation,
    },
    relation::build_relation_analysis,
};

#[test]
fn pilot_representation_choices_come_from_realization() {
    let input = bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let relations = build_relation_analysis(&input).expect("relations");
    let analysis = build_lifecycle_analysis(&input, &relations).expect("lifecycle");

    let candidates = |operation: OperationId, object: ObjectId| {
        analysis
            .choices
            .iter()
            .find(|choice| choice.id == RepresentationChoiceId { operation, object })
            .expect("choice exists")
            .candidates
            .clone()
    };

    assert_eq!(
        candidates(OperationId::CompactAsh, ObjectId::Ash),
        [
            RepresentationMode::Explicit,
            RepresentationMode::PublicCommitted,
        ],
    );
    assert_eq!(
        candidates(OperationId::TransferLive, ObjectId::ReceiptLive),
        [
            RepresentationMode::Explicit,
            RepresentationMode::PrivateCommitted,
        ],
    );
}

#[test]
fn pilot_lifecycle_is_valid_in_scope_but_deployment_incomplete() {
    let input = bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let relations = build_relation_analysis(&input).expect("relations");
    let analysis = build_lifecycle_analysis(&input, &relations).expect("lifecycle");

    let status = |object: ObjectId, mode: RepresentationMode, exit: OperationId| {
        analysis
            .requirements
            .iter()
            .find(|requirement| {
                requirement.object == object
                    && requirement.representation == mode
                    && requirement.exit == exit
            })
            .expect("requirement exists")
            .status
    };

    // Every ASH representation reaches compact (in scope) and clear
    // (an explicit future obligation).
    for mode in [
        RepresentationMode::Explicit,
        RepresentationMode::PublicCommitted,
    ] {
        assert_eq!(
            status(ObjectId::Ash, mode, OperationId::CompactAsh),
            LifecycleExitStatus::AvailableInCompilerScope,
        );
        assert_eq!(
            status(ObjectId::Ash, mode, OperationId::Clear),
            LifecycleExitStatus::DeclaredOutsideCompilerScope,
        );
    }

    // Every live-receipt representation reaches transfer (in scope)
    // plus burn and redeem (future obligations).
    for mode in [
        RepresentationMode::Explicit,
        RepresentationMode::PrivateCommitted,
    ] {
        assert_eq!(
            status(ObjectId::ReceiptLive, mode, OperationId::TransferLive),
            LifecycleExitStatus::AvailableInCompilerScope,
        );
        for exit in [OperationId::Burn, OperationId::Redeem] {
            assert_eq!(
                status(ObjectId::ReceiptLive, mode, exit),
                LifecycleExitStatus::DeclaredOutsideCompilerScope,
            );
        }
    }

    // Semantically analyzable in scope, deliberately not
    // deployment-lifecycle complete.
    assert!(!analysis.is_deployment_complete());
}

#[test]
fn proof_representation_compatibility_is_exact() {
    // Public arithmetic rejects private-committed values…
    assert!(!proof_supports_representation(
        ProofKind::PublicArithmetic,
        RepresentationMode::PrivateCommitted,
    ));
    assert!(proof_supports_representation(
        ProofKind::PublicArithmetic,
        RepresentationMode::Explicit,
    ));
    assert!(proof_supports_representation(
        ProofKind::PublicArithmetic,
        RepresentationMode::PublicCommitted,
    ));

    // …while confidential conservation accepts them and rejects
    // explicit values.
    assert!(proof_supports_representation(
        ProofKind::ConfidentialConservation,
        RepresentationMode::PrivateCommitted,
    ));
    assert!(!proof_supports_representation(
        ProofKind::ConfidentialConservation,
        RepresentationMode::Explicit,
    ));

    // Shape and signer proofs are representation-agnostic within the
    // relation-approved set.
    for mode in [
        RepresentationMode::Explicit,
        RepresentationMode::PublicCommitted,
        RepresentationMode::PrivateCommitted,
    ] {
        assert!(proof_supports_representation(
            ProofKind::ManifestShape,
            mode
        ));
        assert!(proof_supports_representation(
            ProofKind::SignerMembership,
            mode
        ));
    }
}
