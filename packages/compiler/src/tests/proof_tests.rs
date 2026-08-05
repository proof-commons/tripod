//! Obligation classification and exact-search tests (Guide-3 §17.6, §15).

use std::num::NonZeroU64;

use architecture::{ObjectId, OperationId};
use realization::{ProofKind, Relation, RelationKind, RepresentationMode};

use super::{bound_input, phase1_realization};
use crate::{
    AnalysisPolicy, CompilationScope, CompileError, ProofSearchLimits, bind_input,
    capability::{CapabilityView, RequiredCapability},
    lifecycle::RepresentationChoiceId,
    proof::{RelationObligationClass, classify_obligations, enumerate_feasible_plans},
    relation::build_relation_analysis,
};

fn limited_input(
    operations: &[OperationId],
    states: u64,
    candidates: u64,
) -> crate::BoundCompilerInput {
    bind_input(
        &architecture::ARCHITECTURE,
        phase1_realization(),
        CompilationScope::from_operations(operations.iter().copied()).expect("scope"),
        AnalysisPolicy::strict(ProofSearchLimits::new(
            NonZeroU64::new(states).expect("nonzero"),
            NonZeroU64::new(candidates).expect("nonzero"),
        )),
    )
    .expect("bind input")
}

// --- obligation classification (§11) ---

#[test]
fn every_pilot_relation_is_classified_exactly_once() {
    let input = bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let relations = build_relation_analysis(&input).expect("relations");
    let obligations = classify_obligations(&relations).expect("classify");

    assert_eq!(obligations.len(), relations.project().nodes.len());

    for obligation in &obligations {
        let declaration = relations
            .project()
            .nodes
            .into_iter()
            .find(|node| node.source.id == obligation.relation)
            .expect("relation exists")
            .source;

        match &declaration.relation {
            Relation::LifecycleExit { .. } => assert_eq!(
                obligation.class,
                RelationObligationClass::StaticallyValidated,
                "{:?}",
                obligation.relation,
            ),
            Relation::SubstrateConservation { .. } => assert!(
                matches!(
                    obligation.class,
                    RelationObligationClass::ExternalEvidence { .. }
                ),
                "{:?}",
                obligation.relation,
            ),
            _ => {
                let RelationObligationClass::ProofRequired { alternatives } = &obligation.class
                else {
                    panic!(
                        "runtime relation {:?} must be proof-required",
                        obligation.relation
                    );
                };
                assert!(!alternatives.is_empty());
                assert!(alternatives.is_sorted());
            }
        }

        assert!(!obligation.operands.is_empty());
    }
}

// --- exact search over the real pilots (§15) ---

#[test]
fn pilot_search_retains_both_live_transfer_strategies() {
    let input = bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let plans = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained).expect("plans");

    assert!(!plans.candidates.is_empty());
    assert!(plans.candidates.is_sorted());

    let live_choice = RepresentationChoiceId {
        operation: OperationId::TransferLive,
        object: ObjectId::ReceiptLive,
    };
    let conservation_proof = |candidate: &crate::proof::ProofPlanCandidate| {
        candidate
            .proofs
            .iter()
            .find(|(relation, _)| {
                relation.operation() == OperationId::TransferLive
                    && relation.kind() == RelationKind::Conservation
            })
            .map(|(_, alternative)| alternative.proof())
            .expect("live conservation is proof-required")
    };

    // The explicit strategy: explicit representation + public arithmetic.
    assert!(plans.candidates.iter().any(|candidate| {
        candidate.representations[&live_choice] == RepresentationMode::Explicit
            && conservation_proof(candidate) == ProofKind::PublicArithmetic
    }));

    // The private strategy: private-committed + confidential conservation.
    assert!(plans.candidates.iter().any(|candidate| {
        candidate.representations[&live_choice] == RepresentationMode::PrivateCommitted
            && conservation_proof(candidate) == ProofKind::ConfidentialConservation
    }));

    // The invalid pairing never appears: private-committed values with
    // public arithmetic would need exact public amounts.
    assert!(!plans.candidates.iter().any(|candidate| {
        candidate.representations[&live_choice] == RepresentationMode::PrivateCommitted
            && conservation_proof(candidate) == ProofKind::PublicArithmetic
    }));

    for candidate in &plans.candidates {
        // T2 substrate conservation survives every plan…
        assert!(!candidate.external_evidence.is_empty());
        // …no fake proof is chosen for static or external relations…
        assert!(candidate.proofs.keys().all(|relation| {
            relation.kind() != RelationKind::SubstrateConservation
                && relation.kind() != RelationKind::Lifecycle
        }));
        // …and disclosure matches the representation strategy.
        let private =
            candidate.representations[&live_choice] == RepresentationMode::PrivateCommitted;
        let discloses_live_amount = candidate
            .disclosure
            .added_required_public
            .keys()
            .any(|fact| {
                matches!(fact, realization::FactId::FamilyAmount { object, .. }
                if *object == ObjectId::ReceiptLive)
            });

        assert_eq!(discloses_live_amount, !private);
    }

    // Repeated analysis is equal.
    let again = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained).expect("plans");
    assert_eq!(plans, again);
}

#[test]
fn compact_ash_plans_add_no_disclosure() {
    let input = bound_input(&[OperationId::CompactAsh]);
    let plans = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained).expect("plans");

    // Both ASH representations are feasible, and nothing new discloses.
    assert!(plans.candidates.len() >= 2);
    for candidate in &plans.candidates {
        assert!(candidate.disclosure.added_required_public.is_empty());
    }
}

// --- capability pruning and failure classes (§17.6, §13.5-6) ---

#[test]
fn a_missing_capability_blocks_rather_than_weakens() {
    let input = bound_input(&[OperationId::CompactAsh]);

    // Withhold everything: the prepass reports blocked relations, and
    // no relation is silently dropped or weakened.
    let error = enumerate_feasible_plans(
        &input,
        &CapabilityView::Available(std::collections::BTreeSet::default()),
    )
    .unwrap_err();
    let CompileError::NoFeasibleProofPlan { blocked_relations } = error else {
        panic!("must be no-feasible-plan");
    };
    assert!(!blocked_relations.is_empty());
    assert!(blocked_relations.is_sorted());
}

#[test]
fn exhaustion_returns_no_partial_result_and_differs_from_infeasibility() {
    // State exhaustion.
    let input = limited_input(&[OperationId::CompactAsh], 3, 10_000);
    assert!(matches!(
        enumerate_feasible_plans(&input, &CapabilityView::Unconstrained).unwrap_err(),
        CompileError::ProofSearchStateLimitExceeded { maximum: 3 },
    ));

    // Candidate exhaustion.
    let input = limited_input(&[OperationId::CompactAsh], 1_000_000, 1);
    assert!(matches!(
        enumerate_feasible_plans(&input, &CapabilityView::Unconstrained).unwrap_err(),
        CompileError::ProofCandidateLimitExceeded { maximum: 1 },
    ));
}

#[test]
fn capability_requirements_are_published_per_candidate() {
    let input = bound_input(&[OperationId::TransferLive]);
    let plans = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained).expect("plans");

    for candidate in &plans.candidates {
        assert!(
            candidate
                .required_capabilities
                .contains(&RequiredCapability::OwnerAuthorization),
            "live transfer always requires owner authorization",
        );
        assert!(!candidate.source_requirements.is_empty());
        assert!(candidate.source_requirements.is_sorted());
    }
}
