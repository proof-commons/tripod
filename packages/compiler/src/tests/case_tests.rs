//! Execution-case derivation and census tests (Guide-5 §16.1).

use std::collections::BTreeMap;

use architecture::{ObjectId, OperationId};
use realization::RepresentationMode;

use super::bound_input;
use crate::{
    CompileError,
    capability::CapabilityView,
    case::{
        ExecutionCaseId, SponsorCase, case_census, derive_execution_cases, execution_cases,
        is_active, sponsor_cases, validate_case_census,
    },
    lifecycle::RepresentationChoiceId,
    proof::{ProofPlanCandidate, enumerate_feasible_plans},
    relation::{CompilerRelationAnalysis, build_relation_analysis},
    source::{RequiredSourceKind, RequirementActivation},
};

fn analysis(operations: &[OperationId]) -> (CompilerRelationAnalysis, Vec<ProofPlanCandidate>) {
    let input = bound_input(operations);
    let relations = build_relation_analysis(&input).expect("relations");
    let plans = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained).expect("plans");

    (relations, plans.candidates)
}

fn representation(
    object: ObjectId,
    mode: RepresentationMode,
) -> BTreeMap<ObjectId, RepresentationMode> {
    BTreeMap::from([(object, mode)])
}

// --- pilot case census (§16.1, §4.4) ---

#[test]
fn compact_ash_census_is_explicit_and_public_committed_by_sponsor() {
    let (relations, candidates) = analysis(&[OperationId::CompactAsh]);
    let census = case_census(&relations, &candidates).expect("census");

    let expected = [
        (RepresentationMode::Explicit, SponsorCase::Absent),
        (RepresentationMode::Explicit, SponsorCase::Present),
        (RepresentationMode::PublicCommitted, SponsorCase::Absent),
        (RepresentationMode::PublicCommitted, SponsorCase::Present),
    ]
    .into_iter()
    .map(|(mode, sponsor)| ExecutionCaseId {
        operation: OperationId::CompactAsh,
        sponsor,
        representations: representation(ObjectId::Ash, mode),
    })
    .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(census, expected);
    assert_eq!(census.len(), 4);
}

#[test]
fn live_transfer_census_is_explicit_and_private_committed_by_sponsor() {
    let (relations, candidates) = analysis(&[OperationId::TransferLive]);
    let census = case_census(&relations, &candidates).expect("census");

    let expected = [
        (RepresentationMode::Explicit, SponsorCase::Absent),
        (RepresentationMode::Explicit, SponsorCase::Present),
        (RepresentationMode::PrivateCommitted, SponsorCase::Absent),
        (RepresentationMode::PrivateCommitted, SponsorCase::Present),
    ]
    .into_iter()
    .map(|(mode, sponsor)| ExecutionCaseId {
        operation: OperationId::TransferLive,
        sponsor,
        representations: representation(ObjectId::ReceiptLive, mode),
    })
    .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(census, expected);
    assert_eq!(census.len(), 4);
}

#[test]
fn no_private_committed_ash_case_and_no_public_committed_live_case() {
    let (relations, candidates) = analysis(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let census = case_census(&relations, &candidates).expect("census");

    for case in &census {
        let mode = |object: ObjectId| case.representations.get(&object).copied();

        match case.operation {
            OperationId::CompactAsh => {
                assert_ne!(
                    mode(ObjectId::Ash),
                    Some(RepresentationMode::PrivateCommitted),
                    "compact ASH declares no private-committed representation",
                );
            }
            OperationId::TransferLive => {
                assert_ne!(
                    mode(ObjectId::ReceiptLive),
                    Some(RepresentationMode::PublicCommitted),
                    "live transfer declares no public-committed representation",
                );
            }
            other => panic!("unexpected operation {other:?} in census"),
        }
    }

    // Two pilots, four semantic cases each.
    assert_eq!(census.len(), 8);
}

#[test]
fn one_candidate_carries_only_its_own_two_sponsor_cases() {
    let (relations, candidates) = analysis(&[OperationId::TransferLive]);

    for candidate in &candidates {
        let cases = execution_cases(&relations, candidate).expect("cases");

        assert_eq!(cases.len(), 2, "one representation, two sponsor cases");
        assert_eq!(cases[0].id.representations, cases[1].id.representations);

        let sponsors = cases.iter().map(|case| case.id.sponsor).collect::<Vec<_>>();
        assert_eq!(sponsors, vec![SponsorCase::Absent, SponsorCase::Present]);
    }
}

#[test]
fn both_pilots_admit_the_optional_sponsor_flow() {
    let (relations, _) = analysis(&[OperationId::CompactAsh, OperationId::TransferLive]);

    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        assert_eq!(
            sponsor_cases(&relations, operation),
            vec![SponsorCase::Absent, SponsorCase::Present],
        );
    }
}

// --- activation filtering (§4.2, Tranche A step 3) ---

#[test]
fn sponsor_local_sources_are_active_only_when_the_sponsor_is_present() {
    let (relations, candidates) = analysis(&[OperationId::TransferLive]);
    let candidate = candidates.first().expect("a feasible candidate");
    let cases = execution_cases(&relations, candidate).expect("cases");

    let sponsor_local = |case: &crate::case::ExecutionCase| {
        case.active_sources
            .iter()
            .any(|row| row.source == RequiredSourceKind::SponsorLocalWitness)
    };

    let absent = cases
        .iter()
        .find(|case| case.id.sponsor == SponsorCase::Absent)
        .expect("sponsorless case");
    let present = cases
        .iter()
        .find(|case| case.id.sponsor == SponsorCase::Present)
        .expect("sponsored case");

    assert!(!sponsor_local(absent));
    assert!(sponsor_local(present));

    // Activation never adds rows: the sponsorless case is a subset.
    for row in &absent.active_sources {
        assert!(present.active_sources.contains(row));
    }
    assert!(absent.active_sources.len() < present.active_sources.len());
}

#[test]
fn active_sources_belong_to_the_case_operation_only() {
    let (relations, candidates) = analysis(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let candidate = candidates.first().expect("a feasible candidate");

    for case in execution_cases(&relations, candidate).expect("cases") {
        for row in &case.active_sources {
            assert_eq!(row.operand.relation().operation(), case.id.operation);
        }
    }
}

/// One case fixing two object families' representations independently.
///
/// Synthetic: each pilot decides one representation today, so no derived
/// case can distinguish an object-keyed condition from a mode-only one.
/// A second independently represented family is exactly the shape that
/// makes the distinction observable.
fn two_object_case() -> ExecutionCaseId {
    ExecutionCaseId {
        operation: OperationId::TransferLive,
        sponsor: SponsorCase::Absent,
        representations: BTreeMap::from([
            (ObjectId::Ash, RepresentationMode::Explicit),
            (ObjectId::ReceiptLive, RepresentationMode::PrivateCommitted),
        ]),
    }
}

#[test]
fn a_representation_conditional_row_reads_its_own_objects_mode() {
    let (relations, candidates) = analysis(&[OperationId::TransferLive]);
    let candidate = candidates.first().expect("a feasible candidate");
    let cases = execution_cases(&relations, candidate).expect("cases");
    let row = cases
        .first()
        .expect("a case")
        .active_sources
        .first()
        .expect("an active row")
        .clone();

    let conditional = |object: ObjectId| {
        let mut row = row.clone();
        row.activation = RequirementActivation::WhenRepresentation {
            object,
            mode: RepresentationMode::PrivateCommitted,
        };
        row
    };
    let case = two_object_case();

    // The mode belongs to the receipt family. A row conditioned on the
    // ash family being privately committed is inactive here, even though
    // *some* family is.
    assert!(!is_active(&case, &conditional(ObjectId::Ash)));
    assert!(is_active(&case, &conditional(ObjectId::ReceiptLive)));
}

// --- census validation (§15) ---

#[test]
fn a_repeated_case_identity_is_rejected() {
    let (relations, candidates) = analysis(&[OperationId::CompactAsh]);
    let candidate = candidates.first().expect("a feasible candidate");
    let cases = derive_execution_cases(&relations, candidate);
    let mut doubled = cases.clone();
    doubled.push(cases[0].clone());

    assert_eq!(
        validate_case_census(&relations, candidate, &doubled),
        Err(CompileError::DuplicateExecutionCase {
            case: cases[0].id.clone(),
        }),
    );
}

#[test]
fn an_extra_representation_decision_breaks_the_case_census() {
    let (relations, candidates) = analysis(&[OperationId::CompactAsh]);
    let mut candidate = candidates.first().expect("a feasible candidate").clone();

    // A representation decision for an object the operation declares no
    // representation relation for: the derived case carries a dimension
    // the census does not require.
    candidate.representations.insert(
        RepresentationChoiceId {
            operation: OperationId::CompactAsh,
            object: ObjectId::ReceiptLive,
        },
        RepresentationMode::Explicit,
    );

    let cases = derive_execution_cases(&relations, &candidate);
    let error = validate_case_census(&relations, &candidate, &cases).expect_err("census defect");

    let CompileError::ExecutionCaseCensusMismatch {
        missing,
        unexpected,
    } = error
    else {
        panic!("expected a census mismatch");
    };

    assert_eq!(missing.len(), 2);
    assert_eq!(unexpected.len(), 2);
}

#[test]
fn a_missing_representation_decision_is_reported() {
    let (relations, candidates) = analysis(&[OperationId::CompactAsh]);
    let mut candidate = candidates.first().expect("a feasible candidate").clone();

    candidate.representations.remove(&RepresentationChoiceId {
        operation: OperationId::CompactAsh,
        object: ObjectId::Ash,
    });

    let cases = derive_execution_cases(&relations, &candidate);

    assert_eq!(
        validate_case_census(&relations, &candidate, &cases),
        Err(CompileError::MissingRepresentationChoice {
            operation: OperationId::CompactAsh,
            object: ObjectId::Ash,
        }),
    );
}

// --- determinism (§11) ---

#[test]
fn repeated_case_derivation_is_equal() {
    let (relations, candidates) = analysis(&[OperationId::CompactAsh, OperationId::TransferLive]);

    for candidate in &candidates {
        let first = execution_cases(&relations, candidate).expect("first");
        let second = execution_cases(&relations, candidate).expect("second");

        assert_eq!(first, second);
        assert!(first.iter().map(|case| &case.id).is_sorted());
    }

    let first = case_census(&relations, &candidates).expect("first census");
    let mut permuted = candidates;
    permuted.reverse();
    let second = case_census(&relations, &permuted).expect("permuted census");

    assert_eq!(first, second);
}
