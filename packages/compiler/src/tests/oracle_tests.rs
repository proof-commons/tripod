//! Independent exhaustive proof-search oracle (Guide-3 §14).
//!
//! The oracle enumerates the complete Cartesian product of proof and
//! representation assignments with an iterative odometer — never the
//! production recursive search — and validates only complete
//! assignments through the same typed hard constraints. For every
//! instance the production feasible set must equal the oracle's,
//! compared as complete typed candidate sets.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{ObjectId, OperationId};
use proptest::prelude::*;
use realization::{ProofAlternativeId, ProofKind, Relation, RelationId, RepresentationMode};

use super::bound_input;
use crate::{
    BoundCompilerInput, CompileError,
    capability::{CapabilityView, RequiredCapability},
    constructibility::{build_constructibility_analysis, validate_source_constructibility},
    disclosure::{derive_disclosure, validate_disclosure},
    lifecycle::{RepresentationChoiceId, build_lifecycle_analysis, proof_supports_representation},
    proof::{
        ProofPlanCandidate, RelationObligationClass, classify_obligations, enumerate_feasible_plans,
    },
    relation::build_relation_analysis,
    source::{derive_source_requirements, proof_capabilities},
};

/// Complete Cartesian enumeration with complete-assignment validation.
///
/// One deliberately linear test-only function: splitting it would
/// share structure with the production search it must stay
/// independent of.
#[allow(clippy::too_many_lines)]
fn oracle_enumerate(
    input: &BoundCompilerInput,
    view: &CapabilityView,
) -> Result<Vec<ProofPlanCandidate>, CompileError> {
    let relations = build_relation_analysis(input)?;
    let constructibility = build_constructibility_analysis(input)?;
    let lifecycle = build_lifecycle_analysis(input, &relations)?;
    let obligations = classify_obligations(&relations)?;

    let declarations = relations
        .graph
        .node_weights()
        .map(|node| (node.source.id.clone(), node.source.clone()))
        .collect::<BTreeMap<_, _>>();

    let mut proof_variables: Vec<(RelationId, Vec<ProofAlternativeId>)> = Vec::new();
    let mut external_evidence = BTreeSet::new();
    let mut fixed_capabilities = BTreeSet::new();
    let mut fixed_sources = Vec::new();

    for obligation in &obligations {
        match &obligation.class {
            RelationObligationClass::ProofRequired { alternatives } => {
                proof_variables.push((obligation.relation.clone(), alternatives.clone()));
            }
            RelationObligationClass::ExternalEvidence {
                requirement,
                required_capabilities,
                source_requirements,
                ..
            } => {
                external_evidence.insert(requirement.clone());
                fixed_capabilities.extend(required_capabilities.iter().copied());
                fixed_sources.extend(source_requirements.iter().cloned());
            }
            RelationObligationClass::StaticallyValidated => {}
        }
    }

    if !view.supports(&fixed_capabilities) {
        return Ok(Vec::new());
    }

    // Odometer over proofs then representations.
    let domains: Vec<usize> = proof_variables
        .iter()
        .map(|(_, alternatives)| alternatives.len())
        .chain(
            lifecycle
                .choices
                .iter()
                .map(|choice| choice.candidates.len()),
        )
        .collect();
    let mut odometer = vec![0_usize; domains.len()];
    let mut feasible = Vec::new();

    'outer: loop {
        // Materialize the complete assignment.
        let proofs = proof_variables
            .iter()
            .enumerate()
            .map(|(index, (relation, alternatives))| {
                (relation.clone(), alternatives[odometer[index]].clone())
            })
            .collect::<BTreeMap<_, _>>();
        let representations = lifecycle
            .choices
            .iter()
            .enumerate()
            .map(|(index, choice)| {
                (
                    choice.id.clone(),
                    choice.candidates[odometer[proof_variables.len() + index]],
                )
            })
            .collect::<BTreeMap<_, _>>();

        // Validate the complete assignment.
        let mut valid = true;
        let mut required_capabilities = fixed_capabilities.clone();
        let mut source_requirements = fixed_sources.clone();

        for (relation, alternative) in &proofs {
            let declaration = &declarations[relation];
            let capabilities = proof_capabilities(declaration, alternative.proof());

            if !view.supports(&capabilities) {
                valid = false;
                break;
            }

            let Ok(rows) = derive_source_requirements(declaration, alternative.proof()) else {
                valid = false;
                break;
            };

            if validate_source_constructibility(&constructibility, relation, &rows).is_err() {
                valid = false;
                break;
            }

            // Representation compatibility for amount relations.
            if let Relation::AmountConservation {
                input_objects,
                output_objects,
                ..
            } = &declaration.relation
            {
                for (choice, mode) in &representations {
                    if choice.operation == relation.operation()
                        && (input_objects.contains(&choice.object)
                            || output_objects.contains(&choice.object))
                        && !proof_supports_representation(alternative.proof(), *mode)
                    {
                        valid = false;
                    }
                }
            }

            if !valid {
                break;
            }

            required_capabilities.extend(capabilities);
            source_requirements.extend(rows);
        }

        if valid {
            source_requirements.sort();
            source_requirements.dedup();

            if let Ok(disclosure) =
                derive_disclosure(input.realization().declassification(), &representations)
                && validate_disclosure(&disclosure).is_ok()
            {
                let lifecycle_rows = lifecycle
                    .requirements
                    .iter()
                    .filter(|requirement| {
                        representations.iter().any(|(choice, mode)| {
                            choice.object == requirement.object
                                && *mode == requirement.representation
                        })
                    })
                    .cloned()
                    .collect::<Vec<_>>();

                feasible.push(ProofPlanCandidate {
                    proofs,
                    representations,
                    required_capabilities,
                    source_requirements,
                    external_evidence: external_evidence.clone(),
                    lifecycle: lifecycle_rows,
                    disclosure,
                });
            }
        }

        // Advance the odometer.
        for position in (0..domains.len()).rev() {
            odometer[position] += 1;

            if odometer[position] < domains[position] {
                continue 'outer;
            }

            odometer[position] = 0;
        }

        break;
    }

    feasible.sort();
    feasible.dedup();
    Ok(feasible)
}

#[test]
fn production_search_equals_the_oracle_on_every_pilot_scope() {
    for scope in [
        vec![OperationId::CompactAsh],
        vec![OperationId::TransferLive],
        vec![OperationId::CompactAsh, OperationId::TransferLive],
    ] {
        let input = bound_input(&scope);
        let production =
            enumerate_feasible_plans(&input, &CapabilityView::Unconstrained).expect("plans");
        let oracle = oracle_enumerate(&input, &CapabilityView::Unconstrained).expect("oracle");

        assert_eq!(production.candidates, oracle, "scope {scope:?}");
    }
}

#[test]
fn greedy_first_choice_selection_is_globally_invalid() {
    // Per-relation first choice picks public arithmetic for live
    // conservation (its alternatives sort with PublicArithmetic first).
    let input = bound_input(&[OperationId::TransferLive]);
    let relations = build_relation_analysis(&input).expect("relations");
    let conservation = relations
        .project()
        .nodes
        .into_iter()
        .map(|node| node.source)
        .find(|declaration| matches!(declaration.relation, Relation::AmountConservation { .. }))
        .expect("live conservation");
    let mut alternatives = conservation
        .proof_alternatives
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    alternatives.sort();
    assert_eq!(alternatives[0].proof(), ProofKind::PublicArithmetic);

    let plans = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained).expect("plans");
    let live_choice = RepresentationChoiceId {
        operation: OperationId::TransferLive,
        object: ObjectId::ReceiptLive,
    };

    // The greedy local assignment (first alternative) is infeasible
    // with the private representation…
    assert!(!plans.candidates.iter().any(|candidate| {
        candidate.representations[&live_choice] == RepresentationMode::PrivateCommitted
            && candidate.proofs[&conservation.id] == alternatives[0]
    }));

    // …while a feasible global private plan exists with the non-first
    // alternative: exact global planning, not local selection.
    assert!(plans.candidates.iter().any(|candidate| {
        candidate.representations[&live_choice] == RepresentationMode::PrivateCommitted
            && candidate.proofs[&conservation.id].proof() == ProofKind::ConfidentialConservation
    }));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// Random capability views: production equals oracle, whether the
    /// outcome is a feasible set or a typed no-feasible-plan failure.
    #[test]
    fn random_capability_views_agree_with_the_oracle(
        subset in proptest::collection::btree_set(
            prop_oneof![
                Just(RequiredCapability::AuthenticatedObjectRecognition),
                Just(RequiredCapability::AuthenticatedFamilyCardinality),
                Just(RequiredCapability::AuthenticatedCanonicalPartition),
                Just(RequiredCapability::AuthenticatedOpenFlowPartition),
                Just(RequiredCapability::AuthenticatedRootEffects),
                Just(RequiredCapability::AuthenticatedProjectionSet),
                Just(RequiredCapability::ExactPublicAmountArithmetic),
                Just(RequiredCapability::ConfidentialValueConservation),
                Just(RequiredCapability::OwnerAuthorization),
                Just(RequiredCapability::PublicConstructibility),
                Just(RequiredCapability::WholeTransactionValueConservation),
            ],
            0..11,
        ),
    ) {
        let input = bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
        let view = CapabilityView::Available(subset);

        match enumerate_feasible_plans(&input, &view) {
            Ok(production) => {
                let oracle = oracle_enumerate(&input, &view).expect("oracle");
                prop_assert_eq!(production.candidates, oracle);
            }
            Err(CompileError::NoFeasibleProofPlan { .. }) => {
                let oracle = oracle_enumerate(&input, &view).expect("oracle");
                prop_assert!(oracle.is_empty(), "production blocked but oracle feasible");
            }
            Err(other) => prop_assert!(false, "unexpected error {other:?}"),
        }
    }
}
