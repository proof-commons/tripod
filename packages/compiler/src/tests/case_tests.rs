//! Execution-case derivation and census tests (Guide-5 §16.1).

use std::collections::BTreeMap;

use architecture::{ObjectId, OpenFlowKind, OperationId};
use realization::RepresentationMode;

use super::bound_input;
use crate::{
    CompileError,
    capability::CapabilityView,
    case::{
        ExecutionCaseId, SponsorCase, case_census, derive_execution_cases, execution_cases,
        is_active, is_sponsor_region_family, sponsor_cases, validate_case_census,
    },
    lifecycle::RepresentationChoiceId,
    proof::{ProofPlanCandidate, enumerate_feasible_plans},
    relation::{CompilerRelationAnalysis, build_relation_analysis},
    source::{RequiredSourceKind, RequirementActivation},
    sponsor_region::{OrdinaryLbtcRole, ordinary_lbtc_role, validate_decidable_sponsor_regions},
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

// --- S2-01: sponsor identity is a flow role, not an object family ---

#[test]
fn both_pilots_use_ordinary_lbtc_only_as_the_fee_sponsor_region() {
    // The invariance evidence for this change: the pilots declare fee
    // sponsorship and no protocol flow that could claim ordinary L-BTC,
    // so the corrected rule reduces exactly to the family test it
    // replaces and every pilot analysis is unchanged.
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let (relations, _) = analysis(&[operation]);

        assert_eq!(
            ordinary_lbtc_role(&relations, operation),
            OrdinaryLbtcRole::SponsorRegion,
            "{operation:?}",
        );
        assert!(is_sponsor_region_family(
            ObjectId::PlainLbtc,
            ordinary_lbtc_role(&relations, operation),
        ));
    }
}

#[test]
fn a_protocol_claimed_ordinary_lbtc_family_is_not_a_sponsor_region() {
    // The defect S2-01 names, stated directly: the same family in an
    // operation whose flows claim it for the protocol is not the
    // optional sponsor region, so no relation over it may be made
    // sponsor-conditional and no amount over it may be erased.
    assert!(!is_sponsor_region_family(
        ObjectId::PlainLbtc,
        OrdinaryLbtcRole::ProtocolClaimed,
    ));
    assert!(!OrdinaryLbtcRole::ProtocolClaimed.is_sponsor_region());

    // Silence is read as erasure, which is safe only because the
    // protocol-claimed case is refused rather than resolved.
    assert!(OrdinaryLbtcRole::Absent.is_sponsor_region());
    assert!(OrdinaryLbtcRole::SponsorRegion.is_sponsor_region());

    // No other family is ever the sponsor region, whatever the role.
    for role in [
        OrdinaryLbtcRole::Absent,
        OrdinaryLbtcRole::SponsorRegion,
        OrdinaryLbtcRole::ProtocolClaimed,
    ] {
        assert!(!is_sponsor_region_family(ObjectId::ReceiptLive, role));
    }
}

#[test]
fn an_operation_claiming_ordinary_lbtc_for_the_protocol_is_refused() {
    // An operation whose declared open flows include one that can hold
    // a protocol L-BTC amount cannot have its sponsor region decided
    // from the declarations the compiler is given, so binding a scope
    // containing it is a typed refusal rather than a guess.
    let (relations, _) = analysis(&[OperationId::CompactAsh]);

    let protocol_claiming = [
        OpenFlowKind::RequestCreation,
        OpenFlowKind::RequestRefund,
        OpenFlowKind::DepositAdmission,
        OpenFlowKind::Redemption,
    ];

    for flow in protocol_claiming {
        let synthetic = relations_declaring_open_flows(&[flow, OpenFlowKind::FeeSponsor]);

        assert_eq!(
            ordinary_lbtc_role(&synthetic, OperationId::CompactAsh),
            OrdinaryLbtcRole::ProtocolClaimed,
            "{flow:?}",
        );
        assert_eq!(
            validate_decidable_sponsor_regions(&synthetic, &[OperationId::CompactAsh]),
            Err(CompileError::UndecidableSponsorRegion {
                operation: OperationId::CompactAsh,
            }),
            "{flow:?}",
        );
    }

    // The reserve carry moves the reserve asset; its ordinary-L-BTC
    // change is generic sponsor change, so it claims nothing.
    let carry =
        relations_declaring_open_flows(&[OpenFlowKind::ReserveCarry, OpenFlowKind::FeeSponsor]);

    assert_eq!(
        ordinary_lbtc_role(&carry, OperationId::CompactAsh),
        OrdinaryLbtcRole::SponsorRegion,
    );
    assert_eq!(
        validate_decidable_sponsor_regions(&carry, &[OperationId::CompactAsh]),
        Ok(()),
    );

    // And the real pilot scope passes the gate.
    assert_eq!(
        validate_decidable_sponsor_regions(&relations, &[OperationId::CompactAsh]),
        Ok(()),
    );
}

/// One pilot relation analysis with its open-flow policy replaced.
fn relations_declaring_open_flows(allowed: &[OpenFlowKind]) -> CompilerRelationAnalysis {
    let input = bound_input(&[OperationId::CompactAsh]);
    let source = input.realization().project();

    let nodes = source
        .relations
        .nodes
        .iter()
        .map(|node| {
            let mut node = node.clone();

            if matches!(node.relation, realization::Relation::OpenFlowPolicy { .. }) {
                node.relation = realization::Relation::OpenFlowPolicy {
                    allowed: allowed.iter().copied().collect(),
                };
            }

            node
        })
        .collect::<Vec<_>>();

    crate::relation::build_relation_graph(
        input.scope().operations(),
        &nodes,
        &source.relations.edges,
    )
    .expect("the synthetic relation graph builds")
}

// --- §6.5 mixed representation is refused, not planned ---

/// One synthetic conservation relation over two object families.
///
/// Hand-built rather than taken from a pilot, because no pilot declares
/// one: every operation the realization declares conserves one family
/// per side, so the mixed case is unreachable through the production
/// path today. That is precisely why it is worth stating — an
/// unreachable refusal nobody exercises is indistinguishable from a
/// refusal that does not work, and the operation this guards is one a
/// later wave is expected to add.
fn two_family_conservation() -> realization::RelationDeclaration {
    realization::RelationDeclaration {
        id: realization::RelationId::new(
            OperationId::TransferLive,
            realization::RelationKind::Conservation,
            realization::RelationSubject::Asset {
                asset: architecture::AssetId::U,
            },
        ),
        relation: realization::Relation::AmountConservation {
            asset: architecture::AssetId::U,
            input_objects: std::collections::BTreeSet::from([
                ObjectId::ReceiptLive,
                ObjectId::ReceiptTimeLocked,
            ]),
            output_objects: std::collections::BTreeSet::from([ObjectId::ReceiptLive]),
        },
        proof_alternatives: std::collections::BTreeSet::new(),
    }
}

fn case_fixing(
    representations: BTreeMap<ObjectId, RepresentationMode>,
) -> crate::case::ExecutionCaseId {
    ExecutionCaseId {
        operation: OperationId::TransferLive,
        sponsor: SponsorCase::Absent,
        representations,
    }
}

#[test]
fn a_homogeneous_case_decides_the_conserved_amounts_either_way() {
    use crate::case::{ConservedAmountVisibility, conserved_amount_visibility};

    let declaration = two_family_conservation();
    let both = |mode| {
        case_fixing(BTreeMap::from([
            (ObjectId::ReceiptLive, mode),
            (ObjectId::ReceiptTimeLocked, mode),
        ]))
    };

    assert_eq!(
        conserved_amount_visibility(&declaration, &both(RepresentationMode::Explicit)),
        Ok(ConservedAmountVisibility::Readable),
    );
    assert_eq!(
        conserved_amount_visibility(&declaration, &both(RepresentationMode::PrivateCommitted)),
        Ok(ConservedAmountVisibility::Committed),
    );

    // A publicly committed amount is published through an authenticated
    // opening, so the arithmetic discharge still stands. Grouping it
    // with the private mode because both involve commitments would move
    // a discharge the analysis can perform to a target nobody asked.
    assert_eq!(
        conserved_amount_visibility(&declaration, &both(RepresentationMode::PublicCommitted)),
        Ok(ConservedAmountVisibility::Readable),
    );
}

#[test]
fn a_mixed_case_is_refused_rather_than_planned_as_either_half() {
    use crate::case::conserved_amount_visibility;

    let declaration = two_family_conservation();

    // Both orderings, because a rule that read the first family it saw
    // would refuse one of them and plan the other.
    for (live, locked) in [
        (
            RepresentationMode::Explicit,
            RepresentationMode::PrivateCommitted,
        ),
        (
            RepresentationMode::PrivateCommitted,
            RepresentationMode::Explicit,
        ),
    ] {
        let case = case_fixing(BTreeMap::from([
            (ObjectId::ReceiptLive, live),
            (ObjectId::ReceiptTimeLocked, locked),
        ]));

        assert_eq!(
            conserved_amount_visibility(&declaration, &case),
            Err(CompileError::MixedRepresentationConservation {
                relation: declaration.id.clone(),
            }),
            "{live:?} with {locked:?}",
        );
    }
}

#[test]
fn a_relation_that_conserves_nothing_reads_no_representation() {
    use crate::case::{ConservedAmountVisibility, conserved_amount_visibility};

    // The mixed case above, over a relation with no amounts: the answer
    // must not depend on the representations at all, or every relation
    // in a mixed case would be refused rather than the one relation the
    // mixing actually breaks.
    let declaration = realization::RelationDeclaration {
        id: realization::RelationId::new(
            OperationId::TransferLive,
            realization::RelationKind::SponsorIsolation,
            realization::RelationSubject::Operation,
        ),
        relation: realization::Relation::SponsorIsolation,
        proof_alternatives: std::collections::BTreeSet::new(),
    };
    let case = case_fixing(BTreeMap::from([
        (ObjectId::ReceiptLive, RepresentationMode::Explicit),
        (
            ObjectId::ReceiptTimeLocked,
            RepresentationMode::PrivateCommitted,
        ),
    ]));

    assert_eq!(
        conserved_amount_visibility(&declaration, &case),
        Ok(ConservedAmountVisibility::Readable),
    );
}
