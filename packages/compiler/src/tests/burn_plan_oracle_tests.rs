//! Source-based Burn oracle. Expectations come from the declared operation
//! and literal Wave-3 coordinates, never from a Burn plan projection.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{ObjectId, OperationId};
use realization::{RelationId, RelationKind, RelationSubject, RepresentationMode, TransactionSide};

use crate::{
    CompileError,
    analyzed::analyze_scoped_program,
    burn_operation_plan::{derive_burn_plan, validate_burn_plan},
    carrier::{CarrierEligibility, CarrierQuantification, CarrierRole},
    case::{ExecutionCaseId, SponsorCase},
    coverage::{CoverageBoundary, CoveragePurpose},
    layout::LayoutRequirement,
    lifecycle::{LifecycleExitStatus, LifecycleRequirement, RepresentationChoiceId},
    placement::{CarrierMultiplicity, PlacedCarrier, PlacementSearchLimits, RelationCaseKey},
    requirement::ProofDisposition,
    source::{BurnClearCoordinate, OperandRole},
};

fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(10_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
    )
}

fn expected_case(sponsor: SponsorCase) -> ExecutionCaseId {
    ExecutionCaseId {
        operation: OperationId::Burn,
        sponsor,
        representations: BTreeMap::from([(ObjectId::Ash, RepresentationMode::Explicit)]),
        side_representations: Box::new(BTreeMap::from([
            (
                (TransactionSide::Input, ObjectId::ReceiptLive),
                RepresentationMode::Explicit,
            ),
            (
                (TransactionSide::Output, ObjectId::ReceiptLive),
                RepresentationMode::Explicit,
            ),
        ])),
    }
}

fn expected_choice(object: ObjectId, side: Option<TransactionSide>) -> RepresentationChoiceId {
    RepresentationChoiceId {
        operation: OperationId::Burn,
        object,
        side,
    }
}

fn expected_outstanding() -> BTreeSet<LifecycleRequirement> {
    use LifecycleExitStatus::DeclaredOutsideCompilerScope;
    use ObjectId::{Ash, ReceiptLive};
    use OperationId::{Clear, CompactAsh, Redeem};
    [(ReceiptLive, Redeem), (Ash, CompactAsh), (Ash, Clear)]
        .into_iter()
        .map(|(object, exit)| LifecycleRequirement {
            object,
            representation: RepresentationMode::Explicit,
            exit,
            status: DeclaredOutsideCompilerScope,
        })
        .collect()
}

fn omission_atoms() -> [RelationId; 4] {
    use RelationKind as Kind;
    use RelationSubject as Subject;
    [
        RelationId::new(OperationId::Burn, Kind::BurnBinding, Subject::Operation),
        RelationId::new(OperationId::Burn, Kind::DataOutputs, Subject::Operation),
        RelationId::new(
            OperationId::Burn,
            Kind::Representation,
            Subject::Representation {
                object: ObjectId::Ash,
            },
        ),
        RelationId::new(
            OperationId::Burn,
            Kind::Lifecycle,
            Subject::LifecycleExit {
                object: ObjectId::ReceiptLive,
                exit: OperationId::Burn,
            },
        ),
    ]
}

// Enumerate subsets of the declaration-derived eligible roles. This is a
// test-only decision search; it never reads the facade's carrier projection.
fn carrier_subsets(eligibility: &CarrierEligibility) -> BTreeSet<BTreeSet<PlacedCarrier>> {
    let entries = &eligibility.eligible;
    let mut feasible = BTreeSet::new();
    for mask in 1_usize..(1_usize << entries.len()) {
        let subset = entries
            .iter()
            .enumerate()
            .filter(|(bit, _)| mask & (1 << bit) != 0)
            .map(|(_, entry)| PlacedCarrier {
                carrier: entry.carrier.clone(),
                quantification: entry.quantification,
            })
            .collect::<BTreeSet<_>>();
        let quantified = subset.iter().all(|entry| match eligibility.multiplicity {
            CarrierMultiplicity::EveryMember => matches!(
                entry.quantification,
                CarrierQuantification::PerMember | CarrierQuantification::CompleteFamilyProof
            ),
            _ => entry.quantification == CarrierQuantification::Single,
        });
        let runtime = subset.iter().all(|entry| {
            !matches!(
                entry.carrier,
                CarrierRole::BackendStructural { .. } | CarrierRole::ExternalEvidence { .. }
            )
        });
        if quantified && runtime {
            let allowed_size = match eligibility.multiplicity {
                CarrierMultiplicity::ExactlyOne => subset.len() == 1,
                CarrierMultiplicity::DeliberateDuplication => subset.len() == entries.len(),
                CarrierMultiplicity::EveryMember | CarrierMultiplicity::AtLeastOne => true,
            };
            if allowed_size {
                feasible.insert(subset);
            }
        }
    }
    feasible
        .iter()
        .filter(|outer| {
            !feasible
                .iter()
                .any(|inner| inner != *outer && inner.is_subset(outer))
        })
        .cloned()
        .collect()
}

fn layout_kind(row: &LayoutRequirement) -> &'static str {
    match row {
        LayoutRequirement::AuthenticateFamilyCensus { .. } => "family census",
        LayoutRequirement::CompleteAndDisjointFamilies { .. } => "disjoint families",
        LayoutRequirement::CanonicalCoordinator { .. } => "coordinator",
        LayoutRequirement::MakeSourceAvailable { .. } => "source routing",
        LayoutRequirement::IsolateSponsorRegion { .. } => "sponsor region",
        LayoutRequirement::EnforceRepresentation { .. } => "representation",
        LayoutRequirement::EnforceSideRepresentation { .. } => "side representation",
        LayoutRequirement::SecretFreeOperationPath { .. } => "secret free",
    }
}

fn layout_case(row: &LayoutRequirement) -> Option<ExecutionCaseId> {
    match row {
        LayoutRequirement::MakeSourceAvailable { case, .. }
        | LayoutRequirement::IsolateSponsorRegion { case, .. }
        | LayoutRequirement::SecretFreeOperationPath { case, .. } => Some(case.clone()),
        _ => None,
    }
}

fn coverage_kind(boundary: CoverageBoundary, purpose: &CoveragePurpose) -> &'static str {
    match purpose {
        CoveragePurpose::ReportAnchorRequired { .. }
        | CoveragePurpose::ReportAnchorChanged { .. } => "report",
        CoveragePurpose::StructuralPresence | CoveragePurpose::StructuralRejection(_) => {
            "structural"
        }
        _ if boundary == CoverageBoundary::BackendStructural => "structural",
        CoveragePurpose::ActiveAccept
        | CoveragePurpose::InactiveAccept
        | CoveragePurpose::CarrierExecution(_)
        | CoveragePurpose::AcceptedProjection
        | CoveragePurpose::ExternalEvidenceVerified { .. } => "positive",
        CoveragePurpose::FocusedReject(_) | CoveragePurpose::ExternalEvidenceRejected { .. } => {
            "negative"
        }
    }
}

fn expected_layout_partition() -> BTreeMap<(Option<ExecutionCaseId>, &'static str), usize> {
    // Counts pinned from the validated plan, not derived from declarations.
    BTreeMap::from([
        ((None, "coordinator"), 3),
        ((None, "disjoint families"), 12),
        ((None, "family census"), 15),
        ((None, "representation"), 1),
        ((None, "side representation"), 2),
        (
            (Some(expected_case(SponsorCase::Absent)), "source routing"),
            28,
        ),
        (
            (Some(expected_case(SponsorCase::Absent)), "sponsor region"),
            2,
        ),
        (
            (Some(expected_case(SponsorCase::Present)), "source routing"),
            37,
        ),
        (
            (Some(expected_case(SponsorCase::Present)), "sponsor region"),
            2,
        ),
    ])
}

fn expected_coverage_partition() -> BTreeMap<(ExecutionCaseId, &'static str), usize> {
    // Counts pinned from the validated plan, not derived from declarations.
    BTreeMap::from([
        ((expected_case(SponsorCase::Absent), "positive"), 79),
        ((expected_case(SponsorCase::Absent), "negative"), 40),
        ((expected_case(SponsorCase::Absent), "structural"), 32),
        ((expected_case(SponsorCase::Absent), "report"), 4),
        ((expected_case(SponsorCase::Present), "positive"), 90),
        ((expected_case(SponsorCase::Present), "negative"), 46),
        ((expected_case(SponsorCase::Present), "structural"), 32),
        ((expected_case(SponsorCase::Present), "report"), 4),
    ])
}

#[test]
#[allow(clippy::too_many_lines)] // The literal source, mode, exit, anchor and subset claims form one oracle.
fn burn_plan_matches_small_oracle() {
    let control = super::source_tests::burn_control();
    let analyzed =
        analyze_scoped_program(&control.input, limits()).expect("complete Burn analysis");
    let plan = derive_burn_plan(&analyzed).expect("explicit Burn plan");
    assert_eq!(validate_burn_plan(&analyzed, &plan), Ok(()));

    // The realization declaration is the independent source of relation
    // identities and proof approvals. Its literal 32-row contract is
    // pinned by realization's own conformance test.
    let declared = control
        .input
        .realization()
        .project()
        .relations
        .nodes
        .into_iter()
        .filter(|row| row.id.operation() == OperationId::Burn)
        .map(|row| (row.id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(declared.len(), 32);
    assert_eq!(
        plan.relations()
            .map(|row| row.relation.clone())
            .collect::<BTreeSet<_>>(),
        declared.keys().cloned().collect(),
    );
    for row in plan.relations() {
        let approved = &declared[&row.relation].proof_alternatives;
        match &row.proof {
            ProofDisposition::Selected { proof } => assert!(approved.contains(proof)),
            ProofDisposition::ExternalEvidence { approved_proof, .. } => {
                assert!(approved.contains(approved_proof));
            }
            ProofDisposition::StaticallyValidated => assert!(approved.is_empty()),
        }
    }

    let expected_cases = [SponsorCase::Absent, SponsorCase::Present]
        .into_iter()
        .map(expected_case)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        plan.cases()
            .map(|case| case.id.clone())
            .collect::<BTreeSet<_>>(),
        expected_cases,
    );
    assert_eq!(
        plan.representation()
            .selected()
            .map(|(choice, mode)| (choice.clone(), mode))
            .collect::<BTreeMap<_, _>>(),
        BTreeMap::from([
            (
                expected_choice(ObjectId::ReceiptLive, Some(TransactionSide::Input)),
                RepresentationMode::Explicit
            ),
            (
                expected_choice(ObjectId::ReceiptLive, Some(TransactionSide::Output)),
                RepresentationMode::Explicit
            ),
            (
                expected_choice(ObjectId::Ash, None),
                RepresentationMode::Explicit
            ),
        ]),
    );
    assert_eq!(
        plan.lifecycle().implemented().collect::<BTreeSet<_>>(),
        BTreeSet::from([OperationId::TransferLive, OperationId::Burn]),
    );
    assert_eq!(
        plan.lifecycle()
            .outstanding()
            .cloned()
            .collect::<BTreeSet<_>>(),
        expected_outstanding(),
    );

    let mut anchors = BTreeSet::new();
    let mut anchor_count = 0;
    for row in plan.coverage() {
        let CoveragePurpose::ReportAnchorRequired { operand } = &row.id.purpose else {
            continue;
        };
        assert_eq!(row.id.boundary, CoverageBoundary::ReportObligation);
        let OperandRole::BurnClear(coordinate) = operand.role() else {
            panic!("Burn report anchor uses another role");
        };
        anchor_count += 1;
        anchors.insert((row.id.case.clone(), *coordinate));
    }
    assert_eq!(anchor_count, 4);
    assert_eq!(
        anchors,
        expected_cases
            .iter()
            .flat_map(|case| {
                [
                    BurnClearCoordinate::BurnEventType,
                    BurnClearCoordinate::BurnFreshAmountEvent,
                ]
                .into_iter()
                .map(|coordinate| (case.clone(), coordinate))
            })
            .collect(),
    );

    // Independently enumerate assignments from the lower-stage eligible
    // roles, then compare each Burn relation-case with the public facade.
    // The production carrier projection and placement search are not used
    // to form the expected assignments.
    let cases = crate::analyzed_operation::operation_execution_cases(
        &control.relations,
        &control.candidate,
        OperationId::Burn,
    )
    .expect("Burn execution cases");
    let relation_cases = crate::placement::classify_relation_cases(&control.relations, &cases)
        .expect("Burn relation cases");
    let eligibility =
        crate::carrier::relation_case_eligibility(&control.relations, &relation_cases)
            .expect("declaration-derived Burn eligibility");
    let mut expected_carriers = BTreeMap::new();
    for eligible in &eligibility {
        let key = RelationCaseKey {
            relation: eligible.relation.clone(),
            case: eligible.case.clone(),
        };
        let subsets = carrier_subsets(eligible);
        assert!(!subsets.is_empty(), "Burn runtime obligation: {key:?}");
        expected_carriers.insert(key, subsets);
    }
    let actual_carriers = plan
        .carriers()
        .map(|row| {
            (
                row.relation_case.clone(),
                row.alternatives
                    .iter()
                    .map(|alternative| alternative.carriers.clone())
                    .collect::<BTreeSet<_>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    // A four-obligation subinstance of this Burn declaration stays within
    // ruling 227's bounded power-set domain. The product counts every
    // nonempty-or-empty subset choice before feasibility filtering.
    let tiny = eligibility
        .iter()
        .filter(|row| row.eligible.len() <= 3)
        .take(4)
        .collect::<Vec<_>>();
    assert_eq!(tiny.len(), 4);
    let domain = tiny
        .iter()
        .map(|row| row.eligible.len())
        .collect::<Vec<_>>();
    let product = domain.iter().map(|size| 1_usize << size).product::<usize>();
    assert_eq!(domain, [2, 2, 2, 1]);
    assert_eq!(product, 128);
    let mut expected_tiny = BTreeSet::from([BTreeMap::new()]);
    for row in &tiny {
        assert!(!carrier_subsets(row).is_empty());
        let key = RelationCaseKey {
            relation: row.relation.clone(),
            case: row.case.clone(),
        };
        expected_tiny = expected_tiny
            .iter()
            .flat_map(|assignments| {
                carrier_subsets(row).into_iter().map(|carriers| {
                    let mut next = assignments.clone();
                    next.insert(key.clone(), carriers);
                    next
                })
            })
            .collect();
    }
    let tiny_keys = tiny
        .iter()
        .map(|row| RelationCaseKey {
            relation: row.relation.clone(),
            case: row.case.clone(),
        })
        .collect::<BTreeSet<_>>();
    let factor = &analyzed.proof_plans[&control.candidate].operations[&OperationId::Burn];
    let actual_tiny = factor
        .feasible_placements
        .iter()
        .map(|placement| {
            placement
                .assignments
                .iter()
                .filter(|(key, _)| tiny_keys.contains(*key))
                .map(|(key, carriers)| (key.clone(), carriers.clone()))
                .collect::<BTreeMap<_, _>>()
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(actual_tiny, expected_tiny);

    let layout_partition = plan.layout().fold(BTreeMap::new(), |mut counts, row| {
        *counts
            .entry((layout_case(row), layout_kind(row)))
            .or_insert(0_usize) += 1;
        counts
    });
    let coverage_partition = plan.coverage().fold(BTreeMap::new(), |mut counts, row| {
        *counts
            .entry((
                row.id.case.clone(),
                coverage_kind(row.id.boundary, &row.id.purpose),
            ))
            .or_insert(0_usize) += 1;
        counts
    });
    assert_eq!(actual_carriers, expected_carriers);
    assert_eq!(layout_partition, expected_layout_partition());
    assert_eq!(coverage_partition, expected_coverage_partition());

    // Four literal relation atoms are retained for the focused corruption
    // subjects below; relation-set equality was checked above.
    let atoms = omission_atoms();
    assert!(atoms.iter().all(|atom| declared.contains_key(atom)));

    // Each refusal subject is exactly one relation away from the accepted
    // control and reaches the owning Burn relation-census boundary.
    for atom in atoms {
        let mut offered = plan.clone();
        assert!(offered.relations_mut().remove(&atom).is_some());
        assert_eq!(
            validate_burn_plan(&analyzed, &offered),
            Err(CompileError::TargetPlanRelationCensusMismatch { relation: atom }),
        );
    }
}
