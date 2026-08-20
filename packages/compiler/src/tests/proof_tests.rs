//! Obligation classification and exact-search tests (Guide-3 §17.6, §15).

use std::{collections::BTreeSet, num::NonZeroU64};

use architecture::{ObjectId, OperationId};
use realization::{ProofKind, Relation, RelationKind, RepresentationMode};

use super::{bound_input, phase1_realization};
use crate::{
    AnalysisPolicy, CompilationScope, CompileError, ProofSearchLimits, bind_input,
    capability::{CapabilityView, RequiredCapability},
    constructibility::build_constructibility_analysis,
    lifecycle::RepresentationChoiceId,
    proof::{
        LocalProofRejection, ProofSearchReport, RelationObligationClass, classify_obligations,
        enumerate_feasible_plans, locally_feasible,
    },
    relation::build_relation_analysis,
    source::{OperandRole, RequiredSourceKind},
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
            Relation::LifecycleExit { .. } | Relation::Representation { .. } => assert_eq!(
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
                assert_ne!(alternatives.as_slice(), []);
                assert!(alternatives.is_sorted());
            }
        }

        assert_ne!(obligation.operands, [] as [crate::source::OperandId; 0]);
    }
}

// --- exact search over the real pilots (§15) ---

#[test]
fn pilot_search_retains_both_live_transfer_strategies() {
    let input = bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let plans = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained).expect("plans");

    assert_ne!(
        plans.candidates,
        [] as [crate::proof::ProofPlanCandidate; 0]
    );
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
    assert_ne!(blocked_relations, [] as [realization::RelationId; 0]);
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
        assert_ne!(
            candidate.source_requirements,
            [] as [crate::source::SourceRequirement; 0]
        );
        assert!(candidate.source_requirements.is_sorted());
    }
}

// --- T6: external evidence fixes capabilities and sources (§4-5) ---

/// Every current capability except whole-transaction conservation.
fn capabilities_without_whole_transaction() -> BTreeSet<RequiredCapability> {
    BTreeSet::from([
        RequiredCapability::AuthenticatedObjectRecognition,
        RequiredCapability::AuthenticatedFamilyCardinality,
        RequiredCapability::AuthenticatedCanonicalPartition,
        RequiredCapability::AuthenticatedOpenFlowPartition,
        RequiredCapability::AuthenticatedRootEffects,
        RequiredCapability::AuthenticatedProjectionSet,
        RequiredCapability::ExactPublicAmountArithmetic,
        RequiredCapability::ConfidentialValueConservation,
        RequiredCapability::OwnerAuthorization,
        RequiredCapability::OperatorAuthorization,
        RequiredCapability::RefundAuthorization,
        RequiredCapability::PublicConstructibility,
    ])
}

fn substrate_conservation_relation(input: &crate::BoundCompilerInput) -> realization::RelationId {
    let relations = build_relation_analysis(input).expect("relations");
    relations
        .project()
        .nodes
        .into_iter()
        .map(|node| node.source.id)
        .find(|relation| relation.kind() == RelationKind::SubstrateConservation)
        .expect("pilot declares substrate conservation")
}

#[test]
fn a_missing_whole_transaction_capability_blocks_substrate_conservation() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let input = bound_input(&[operation]);
        let error = enumerate_feasible_plans(
            &input,
            &CapabilityView::Available(capabilities_without_whole_transaction()),
        )
        .unwrap_err();

        let CompileError::NoFeasibleProofPlan { blocked_relations } = error else {
            panic!("{operation:?} must be infeasible without whole-transaction conservation");
        };

        assert!(
            blocked_relations.contains(&substrate_conservation_relation(&input)),
            "{operation:?} must name the substrate-conservation relation",
        );
        assert!(blocked_relations.is_sorted());
    }
}

#[test]
fn adding_the_whole_transaction_capability_restores_feasibility() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let input = bound_input(&[operation]);
        let mut available = capabilities_without_whole_transaction();
        available.insert(RequiredCapability::WholeTransactionValueConservation);

        let plans = enumerate_feasible_plans(&input, &CapabilityView::Available(available))
            .expect("feasible with the whole-transaction capability");

        assert!(!plans.candidates.is_empty(), "{operation:?}");
    }
}

#[test]
fn external_evidence_carries_its_capability_and_source_into_every_candidate() {
    let input = bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let plans = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained).expect("plans");

    assert_ne!(
        plans.candidates,
        [] as [crate::proof::ProofPlanCandidate; 0]
    );

    for candidate in &plans.candidates {
        // The evidence requirement itself is retained unresolved…
        assert!(
            candidate
                .external_evidence
                .iter()
                .any(|requirement| matches!(
                    requirement,
                    realization::ExternalEvidenceRequirement::SubstrateConservation { .. }
                ))
        );

        // …its capability now reaches the plan…
        assert!(
            candidate
                .required_capabilities
                .contains(&RequiredCapability::WholeTransactionValueConservation),
        );

        // …and so does its typed external source row.
        assert!(candidate.source_requirements.iter().any(|row| {
            row.source == RequiredSourceKind::ExternalEvidence
                && matches!(row.operand.role(), OperandRole::ExternalEvidence { .. })
        }));
    }
}

#[test]
fn fixed_external_requirements_name_no_sponsor_amount() {
    let input = bound_input(&[OperationId::TransferLive]);
    let relations = build_relation_analysis(&input).expect("relations");
    let obligations = classify_obligations(&relations).expect("classify");

    let mut seen = false;

    for obligation in &obligations {
        let RelationObligationClass::ExternalEvidence {
            source_requirements,
            required_capabilities,
            proof,
            ..
        } = &obligation.class
        else {
            continue;
        };

        seen = true;
        assert_eq!(proof.proof(), ProofKind::SubstrateConservation);
        assert_eq!(
            *required_capabilities,
            BTreeSet::from([RequiredCapability::WholeTransactionValueConservation]),
        );

        for row in source_requirements {
            // Sponsor erasure is structural: no fixed row may name a
            // sponsor amount, and none is rewritten into exact public
            // sponsor arithmetic.
            assert!(!crate::source::is_sponsor_amount_operand(
                row.operand.role()
            ));
            assert!(!matches!(
                row.operand.role(),
                OperandRole::ObjectFamilyAmount { .. }
            ));
            assert_ne!(row.source, RequiredSourceKind::AuthenticatedConsensusValue);
        }
    }

    assert!(seen, "live transfer declares external evidence");
}

#[test]
fn an_unexpected_external_evidence_alternative_is_rejected() {
    let input = bound_input(&[OperationId::CompactAsh]);
    let mut relations = build_relation_analysis(&input).expect("relations");
    let node = relations
        .graph
        .node_weights_mut()
        .find(|node| node.source.id.kind() == RelationKind::SubstrateConservation)
        .expect("substrate conservation");
    let relation = node.source.id.clone();

    // A second alternative the realization never approved: the
    // compiler refuses rather than picking one.
    node.source
        .proof_alternatives
        .insert(realization::ProofAlternativeId::new(
            relation.clone(),
            ProofKind::PublicArithmetic,
        ));

    assert_eq!(
        classify_obligations(&relations).unwrap_err(),
        CompileError::InvalidExternalEvidenceProofAlternatives { relation },
    );
}

// --- T7: representation is a static mode constraint (§7-8) ---

#[test]
fn representation_relations_carry_no_proof_variable() {
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let input = bound_input(&[operation]);
        let relations = build_relation_analysis(&input).expect("relations");
        let obligations = classify_obligations(&relations).expect("classify");

        let representations = obligations
            .iter()
            .filter(|obligation| obligation.relation.kind() == RelationKind::Representation)
            .collect::<Vec<_>>();

        assert!(!representations.is_empty(), "{operation:?}");

        for obligation in &representations {
            assert_eq!(
                obligation.class,
                RelationObligationClass::StaticallyValidated,
                "{:?}",
                obligation.relation,
            );
        }

        // No candidate selects a proof for the mode constraint.
        let plans =
            enumerate_feasible_plans(&input, &CapabilityView::Unconstrained).expect("plans");

        for candidate in &plans.candidates {
            for obligation in &representations {
                assert!(
                    !candidate.proofs.contains_key(&obligation.relation),
                    "{:?}",
                    obligation.relation,
                );
            }
        }
    }
}

#[test]
fn live_transfer_conservation_agrees_with_every_selected_mode() {
    let input = bound_input(&[OperationId::TransferLive]);
    let plans = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained).expect("plans");
    let live_choice = RepresentationChoiceId {
        operation: OperationId::TransferLive,
        object: ObjectId::ReceiptLive,
    };

    assert_ne!(
        plans.candidates,
        [] as [crate::proof::ProofPlanCandidate; 0]
    );

    for candidate in &plans.candidates {
        let (conservation, alternative) = candidate
            .proofs
            .iter()
            .find(|(relation, _)| relation.kind() == RelationKind::Conservation)
            .expect("live conservation is proof-required");

        let expected = match candidate.representations[&live_choice] {
            RepresentationMode::Explicit => ProofKind::PublicArithmetic,
            RepresentationMode::PrivateCommitted => ProofKind::ConfidentialConservation,
            other @ RepresentationMode::PublicCommitted => {
                panic!("live transfer never selects {other:?}")
            }
        };
        assert_eq!(alternative.proof(), expected);

        // Capabilities and source rows follow the selected proof, and
        // the opposite capability appears only if another selected
        // proof legitimately requires it.
        let (present, absent) = match expected {
            ProofKind::PublicArithmetic => (
                RequiredCapability::ExactPublicAmountArithmetic,
                RequiredCapability::ConfidentialValueConservation,
            ),
            _ => (
                RequiredCapability::ConfidentialValueConservation,
                RequiredCapability::ExactPublicAmountArithmetic,
            ),
        };
        assert!(candidate.required_capabilities.contains(&present));

        let otherwise_required = candidate
            .proofs
            .iter()
            .filter(|(relation, _)| *relation != conservation)
            .any(|(relation, other)| {
                crate::source::proof_capabilities(&declaration_of(&input, relation), other.proof())
                    .contains(&absent)
            });
        assert_eq!(
            candidate.required_capabilities.contains(&absent),
            otherwise_required,
        );

        let expected_source = match expected {
            ProofKind::PublicArithmetic => RequiredSourceKind::AuthenticatedConsensusValue,
            _ => RequiredSourceKind::AuthenticatedCommitmentRelation,
        };
        let amount_rows = candidate
            .source_requirements
            .iter()
            .filter(|row| {
                row.operand.relation() == conservation
                    && matches!(row.operand.role(), OperandRole::ObjectFamilyAmount { .. })
            })
            .collect::<Vec<_>>();

        assert_ne!(amount_rows, [] as [&crate::source::SourceRequirement; 0]);
        for row in amount_rows {
            assert_eq!(row.source, expected_source);
            assert_ne!(row.source, RequiredSourceKind::AuthenticatedFamilyCensus);
        }
    }
}

#[test]
fn compact_ash_keeps_both_modes_under_public_arithmetic() {
    let input = bound_input(&[OperationId::CompactAsh]);
    let plans = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained).expect("plans");
    let ash_choice = RepresentationChoiceId {
        operation: OperationId::CompactAsh,
        object: ObjectId::Ash,
    };

    let mut modes = BTreeSet::new();

    for candidate in &plans.candidates {
        let alternative = candidate
            .proofs
            .iter()
            .find(|(relation, _)| relation.kind() == RelationKind::Conservation)
            .map(|(_, alternative)| alternative.proof())
            .expect("ash conservation is proof-required");

        // Compact ASH declares only public arithmetic, and both of its
        // approved modes remain compatible with it.
        assert_eq!(alternative, ProofKind::PublicArithmetic);
        modes.insert(candidate.representations[&ash_choice]);
    }

    assert_eq!(
        modes,
        BTreeSet::from([
            RepresentationMode::Explicit,
            RepresentationMode::PublicCommitted,
        ]),
    );
}

fn declaration_of(
    input: &crate::BoundCompilerInput,
    relation: &realization::RelationId,
) -> realization::RelationDeclaration {
    build_relation_analysis(input)
        .expect("relations")
        .project()
        .nodes
        .into_iter()
        .map(|node| node.source)
        .find(|declaration| declaration.id == *relation)
        .expect("relation exists")
}

// --- S2-05/SR3-07: exact-search budgets and typed rejection causes ---

#[test]
fn a_single_state_budget_refuses_a_search_that_needs_more() {
    let input = limited_input(&[OperationId::CompactAsh], 1, 10_000);

    assert!(matches!(
        enumerate_feasible_plans(&input, &CapabilityView::Unconstrained).unwrap_err(),
        CompileError::ProofSearchStateLimitExceeded { maximum: 1 },
    ));
}

#[test]
fn the_state_budget_counts_every_visited_state_and_its_boundary_is_exact() {
    let generous = limited_input(&[OperationId::CompactAsh], 1_000_000, 10_000);
    let visited = enumerate_feasible_plans(&generous, &CapabilityView::Unconstrained)
        .expect("plans")
        .search
        .states_visited;

    assert!(
        visited > 1,
        "the pilot search visits more than the root state",
    );

    // Inclusive rule: a budget of exactly the visited count completes,
    // so the last permitted state is the one that finishes the search
    // rather than the one that is refused.
    let exact = limited_input(&[OperationId::CompactAsh], visited, 10_000);
    assert_eq!(
        enumerate_feasible_plans(&exact, &CapabilityView::Unconstrained)
            .expect("the exact budget completes")
            .search
            .states_visited,
        visited,
    );

    // One state short is a typed failure with no partial result.
    let short = limited_input(&[OperationId::CompactAsh], visited - 1, 10_000);
    let error = enumerate_feasible_plans(&short, &CapabilityView::Unconstrained).unwrap_err();
    assert!(
        matches!(
            error,
            CompileError::ProofSearchStateLimitExceeded { maximum }
                if maximum == visited - 1,
        ),
        "one state short exhausts: {error:?}",
    );

    // One state spare changes nothing: the budget bounds the walk, it
    // does not participate in it.
    let spare = limited_input(&[OperationId::CompactAsh], visited + 1, 10_000);
    assert_eq!(
        enumerate_feasible_plans(&spare, &CapabilityView::Unconstrained)
            .expect("a spare budget completes")
            .search
            .states_visited,
        visited,
    );
}

#[test]
fn a_constructibility_rejection_is_never_counted_as_a_capability_rejection() {
    let input = bound_input(&[OperationId::CompactAsh]);
    let declarations = pilot_declarations(&input);
    let intact = build_constructibility_analysis(&input).expect("constructibility");

    // Every capability stays available; only the witness census is
    // corrupted, so the sole cause any alternative can now fail under is
    // witness constructibility.
    let mut starved = build_constructibility_analysis(&input).expect("constructibility");
    starved
        .node_by_id
        .retain(|node, _| !matches!(node, realization::ConstructibilityNodeId::Witness { .. }));

    let mut report = ProofSearchReport::default();
    let mut observed = 0_u64;

    for declaration in &declarations {
        for alternative in &declaration.proof_alternatives {
            if locally_feasible(
                declaration,
                alternative,
                &CapabilityView::Unconstrained,
                &intact,
            )
            .is_err()
            {
                continue;
            }

            let Err(rejection) = locally_feasible(
                declaration,
                alternative,
                &CapabilityView::Unconstrained,
                &starved,
            ) else {
                continue;
            };

            assert_eq!(
                rejection,
                LocalProofRejection::Constructibility,
                "a starved witness census rejects for constructibility only",
            );

            report.record_local_rejection(rejection);
            observed += 1;
        }
    }

    assert!(
        observed > 0,
        "at least one pilot alternative depends on a witness",
    );
    assert_eq!(
        report.rejected_by_capability, 0,
        "capability rejection stays zero when every capability is available",
    );
    assert_eq!(report.rejected_by_source, 0);
    assert_eq!(report.rejected_by_constructibility, observed);
}

#[test]
fn a_missing_capability_is_counted_as_a_capability_rejection() {
    let input = bound_input(&[OperationId::CompactAsh]);
    let constructibility = build_constructibility_analysis(&input).expect("constructibility");
    let declarations = pilot_declarations(&input);

    let mut report = ProofSearchReport::default();
    let mut observed = 0_u64;

    for declaration in &declarations {
        for alternative in &declaration.proof_alternatives {
            let Err(rejection) = locally_feasible(
                declaration,
                alternative,
                &CapabilityView::Available(BTreeSet::default()),
                &constructibility,
            ) else {
                continue;
            };

            assert_eq!(rejection, LocalProofRejection::MissingCapability);
            report.record_local_rejection(rejection);
            observed += 1;
        }
    }

    assert!(observed > 0);
    assert_eq!(report.rejected_by_capability, observed);
    assert_eq!(report.rejected_by_constructibility, 0);
    assert_eq!(report.rejected_by_source, 0);
}

fn pilot_declarations(input: &crate::BoundCompilerInput) -> Vec<realization::RelationDeclaration> {
    build_relation_analysis(input)
        .expect("relations")
        .project()
        .nodes
        .into_iter()
        .map(|node| node.source)
        .collect()
}
