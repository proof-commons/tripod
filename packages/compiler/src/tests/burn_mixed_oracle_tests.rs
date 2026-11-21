//! Literal mixed Burn side, evidence and bounded placement oracle.
//!
//! The expectations below come from the adopted Burn declaration and the
//! carrier scope rules, not from the facade or the production classifiers.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{AssetId, BoundId, ObjectId, OperationId};
use realization::{
    AvailabilityClass, CardinalityMaximum, Count, ExternalEvidenceRequirement, ProofKind, Relation,
    RelationId, RelationKind, RelationSubject, RepresentationMode, TransactionSide,
};

use crate::{
    CompileError,
    analyzed::analyze_scoped_program,
    burn_operation_plan::{
        ExternalEvidenceRole, ValidatedBurnOperationPlan, derive_mixed_burn_plan,
        plan_burn_target_operation, plan_burn_target_operation_with_output_selection,
        validate_mixed_burn_plan,
    },
    carrier::{CarrierQuantification, CarrierRole},
    case::{ExecutionCaseId, SponsorCase},
    layout::LayoutRequirement,
    lifecycle::RepresentationChoiceId,
    placement::{PlacedCarrier, PlacementSearchLimits, RelationCaseKey},
    requirement::ProofDisposition,
    source::{
        OperandId, OperandRole, RequiredSourceKind, RequirementActivation, SourceRequirement,
    },
};

const BURN: OperationId = OperationId::Burn;

const fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(10_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
    )
}

const fn choice(object: ObjectId, side: Option<TransactionSide>) -> RepresentationChoiceId {
    RepresentationChoiceId {
        operation: BURN,
        object,
        side,
    }
}

fn expected_modes() -> BTreeMap<RepresentationChoiceId, RepresentationMode> {
    BTreeMap::from([
        (
            choice(ObjectId::ReceiptLive, Some(TransactionSide::Input)),
            RepresentationMode::Explicit,
        ),
        (
            choice(ObjectId::ReceiptLive, Some(TransactionSide::Output)),
            RepresentationMode::PrivateCommitted,
        ),
        (choice(ObjectId::Ash, None), RepresentationMode::Explicit),
    ])
}

fn expected_case(sponsor: SponsorCase) -> ExecutionCaseId {
    ExecutionCaseId {
        operation: BURN,
        sponsor,
        representations: BTreeMap::from([(ObjectId::Ash, RepresentationMode::Explicit)]),
        side_representations: Box::new(BTreeMap::from([
            (
                (TransactionSide::Input, ObjectId::ReceiptLive),
                RepresentationMode::Explicit,
            ),
            (
                (TransactionSide::Output, ObjectId::ReceiptLive),
                RepresentationMode::PrivateCommitted,
            ),
        ])),
    }
}

const fn relation(kind: RelationKind, subject: RelationSubject) -> RelationId {
    RelationId::new(BURN, kind, subject)
}

const fn family(kind: RelationKind, side: TransactionSide, object: ObjectId) -> RelationId {
    relation(kind, RelationSubject::ObjectFamily { side, object })
}

const fn conservation() -> RelationId {
    relation(
        RelationKind::Conservation,
        RelationSubject::Asset { asset: AssetId::U },
    )
}

fn expected_conservation_sources() -> BTreeSet<SourceRequirement> {
    [
        (
            TransactionSide::Input,
            ObjectId::ReceiptLive,
            RequiredSourceKind::AuthenticatedConsensusValue,
        ),
        (
            TransactionSide::Output,
            ObjectId::Ash,
            RequiredSourceKind::AuthenticatedConsensusValue,
        ),
        (
            TransactionSide::Output,
            ObjectId::ReceiptLive,
            RequiredSourceKind::AuthenticatedCommitmentRelation,
        ),
    ]
    .into_iter()
    .map(|(side, object, source)| SourceRequirement {
        operand: OperandId::new(
            conservation(),
            OperandRole::ObjectFamilyAmount { side, object },
        ),
        source,
        availability: AvailabilityClass::Public,
        activation: RequirementActivation::Always,
    })
    .collect()
}

const fn placed(carrier: CarrierRole, quantification: CarrierQuantification) -> PlacedCarrier {
    PlacedCarrier {
        carrier,
        quantification,
    }
}

fn small_options(
    case: ExecutionCaseId,
) -> BTreeMap<RelationCaseKey, BTreeSet<BTreeSet<PlacedCarrier>>> {
    use CarrierQuantification::{CompleteFamilyProof, PerMember, Single};
    let global = CarrierRole::OperationGlobal {
        operation: BURN,
        anchor: ObjectId::ReceiptLive,
    };
    let member = CarrierRole::EveryInputFamilyMember {
        object: ObjectId::ReceiptLive,
    };
    let coordinator = CarrierRole::InputFamilyCoordinator {
        object: ObjectId::ReceiptLive,
    };
    BTreeMap::from([
        (
            RelationCaseKey {
                relation: relation(RelationKind::BurnBinding, RelationSubject::Operation),
                case: case.clone(),
            },
            BTreeSet::from([BTreeSet::from([placed(global.clone(), Single)])]),
        ),
        (
            RelationCaseKey {
                relation: relation(
                    RelationKind::AllowedObjectFamilies,
                    RelationSubject::TransactionSide {
                        side: TransactionSide::Input,
                    },
                ),
                case: case.clone(),
            },
            BTreeSet::from([BTreeSet::from([placed(global.clone(), Single)])]),
        ),
        (
            RelationCaseKey {
                relation: family(
                    RelationKind::Authorization,
                    TransactionSide::Input,
                    ObjectId::ReceiptLive,
                ),
                case: case.clone(),
            },
            BTreeSet::from([BTreeSet::from([placed(member.clone(), PerMember)])]),
        ),
        (
            RelationCaseKey {
                relation: family(
                    RelationKind::Recognition,
                    TransactionSide::Input,
                    ObjectId::ReceiptLive,
                ),
                case,
            },
            BTreeSet::from([
                BTreeSet::from([placed(global, CompleteFamilyProof)]),
                BTreeSet::from([placed(coordinator, CompleteFamilyProof)]),
                BTreeSet::from([placed(member, PerMember)]),
            ]),
        ),
    ])
}

// Cross only four literal local option sets. The expected three members are
// built without reading eligibility, placement or facade output.
fn small_product(
    options: &BTreeMap<RelationCaseKey, BTreeSet<BTreeSet<PlacedCarrier>>>,
) -> BTreeSet<BTreeMap<RelationCaseKey, BTreeSet<PlacedCarrier>>> {
    let mut product = BTreeSet::from([BTreeMap::new()]);
    for (key, alternatives) in options {
        product = product
            .iter()
            .flat_map(|assignments| {
                alternatives.iter().map(|carriers| {
                    let mut next = assignments.clone();
                    next.insert(key.clone(), carriers.clone());
                    next
                })
            })
            .collect();
    }
    product
}

fn check_side_and_evidence(plan: &ValidatedBurnOperationPlan) {
    assert_eq!(
        plan.representation()
            .selected()
            .map(|(id, mode)| (id.clone(), mode))
            .collect::<BTreeMap<_, _>>(),
        expected_modes(),
    );
    assert_eq!(
        plan.cases()
            .map(|row| row.id.clone())
            .collect::<BTreeSet<_>>(),
        [SponsorCase::Absent, SponsorCase::Present]
            .into_iter()
            .map(expected_case)
            .collect(),
    );
    let row = plan.relation(&conservation()).expect("Burn U conservation");
    assert!(matches!(
        &row.proof,
        ProofDisposition::Selected { proof }
            if proof.proof() == ProofKind::ConfidentialConservation
    ));
    assert_eq!(row.source_requirements, expected_conservation_sources());
    let obligation = ExternalEvidenceRequirement::ConfidentialValueConservation {
        operation: BURN,
        asset: AssetId::U,
    };
    assert_eq!(row.external_evidence, BTreeSet::from([obligation.clone()]));
    for sponsor in [SponsorCase::Absent, SponsorCase::Present] {
        let case = &row.cases[&expected_case(sponsor)];
        assert_eq!(case.external_evidence, BTreeSet::from([obligation.clone()]));
    }
    assert_eq!(
        plan.external_evidence().collect::<BTreeSet<_>>(),
        BTreeSet::from([
            ExternalEvidenceRole::ConfidentialValueConservation,
            ExternalEvidenceRole::SubstrateConservation,
        ]),
    );
}

fn with_case_identity(mut layout: LayoutRequirement, case: &ExecutionCaseId) -> LayoutRequirement {
    match &mut layout {
        LayoutRequirement::MakeSourceAvailable { case: row, .. }
        | LayoutRequirement::IsolateSponsorRegion { case: row, .. }
        | LayoutRequirement::SecretFreeOperationPath { case: row, .. } => *row = case.clone(),
        _ => {}
    }
    layout
}

#[test]
#[allow(clippy::too_many_lines)] // One literal model joins the side, evidence and four-factor oracle.
fn mixed_burn_plan_matches_small_oracle() {
    let control = super::source_tests::burn_mixed_control();
    let analyzed = analyze_scoped_program(&control.input, limits()).expect("complete analysis");
    let plan = derive_mixed_burn_plan(&analyzed).expect("mixed Burn plan");
    assert_eq!(validate_mixed_burn_plan(&analyzed, &plan), Ok(()));
    assert_eq!(plan.operation(), BURN);
    check_side_and_evidence(&plan);

    // The realized declaration is an independent source of relation identity,
    // side admission and optional output cardinality. Literal expectations
    // above and below are formed without invoking compiler classifiers.
    let declared = control
        .input
        .realization()
        .project()
        .relations
        .nodes
        .into_iter()
        .filter(|row| row.id.operation() == BURN)
        .map(|row| (row.id.clone(), row.relation))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(declared.len(), 32);
    assert_eq!(
        plan.relations()
            .map(|row| row.relation.clone())
            .collect::<BTreeSet<_>>(),
        declared.keys().cloned().collect(),
    );
    for (side, expected) in [
        (
            TransactionSide::Input,
            BTreeSet::from([RepresentationMode::Explicit]),
        ),
        (
            TransactionSide::Output,
            BTreeSet::from([
                RepresentationMode::Explicit,
                RepresentationMode::PrivateCommitted,
            ]),
        ),
    ] {
        let id = relation(
            RelationKind::SideRepresentation,
            RelationSubject::SideRepresentation {
                side,
                object: ObjectId::ReceiptLive,
            },
        );
        assert!(matches!(
            &declared[&id],
            Relation::SideRepresentation { allowed, .. } if *allowed == expected
        ));
    }
    let optional_change = family(
        RelationKind::Cardinality,
        TransactionSide::Output,
        ObjectId::ReceiptLive,
    );
    assert!(matches!(
        &declared[&optional_change],
        Relation::Cardinality {
            minimum: Count::ZERO,
            maximum: CardinalityMaximum::Bound(BoundId::BurnChangeMax),
            ..
        }
    ));
    assert!(plan.relation(&optional_change).is_some());

    let options = small_options(expected_case(SponsorCase::Absent));
    let expected = small_product(&options);
    assert_eq!(expected.len(), 3);
    let keys = options.keys().cloned().collect::<BTreeSet<_>>();
    let mut checked = 0;
    for candidate in analyzed.proof_plans.values() {
        let modes = &candidate.proof_plan.representations;
        if !expected_modes()
            .iter()
            .all(|(id, mode)| modes.get(id) == Some(mode))
            || candidate
                .proof_plan
                .proofs
                .get(&conservation())
                .is_none_or(|proof| proof.proof() != ProofKind::ConfidentialConservation)
        {
            continue;
        }
        let factor = &candidate.operations[&BURN];
        let actual = factor
            .feasible_placements
            .enumerated()
            .expect("Burn retains bounded enumeration")
            .iter()
            .map(|placement| {
                placement
                    .assignments
                    .iter()
                    .filter(|(key, _)| keys.contains(*key))
                    .map(|(key, carriers)| (key.clone(), carriers.clone()))
                    .collect::<BTreeMap<_, _>>()
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(actual, expected);
        checked += 1;
    }
    assert!(checked > 0);

    // The deleted carrier is selected from the publication: this subject is
    // a pin for fresh-validation refusal, not an independent carrier row.
    let mut changed = plan.clone();
    let pinned = changed.carriers().next().expect("carrier row").clone();
    assert!(changed.carriers_mut().remove(&pinned));
    assert_eq!(
        validate_mixed_burn_plan(&analyzed, &changed),
        Err(CompileError::TargetPlanCarrierCensusMismatch),
    );

    let mut sponsor_amount = plan;
    let source = SourceRequirement {
        operand: OperandId::new(
            conservation(),
            OperandRole::ObjectFamilyAmount {
                side: TransactionSide::Input,
                object: ObjectId::PlainLbtc,
            },
        ),
        source: RequiredSourceKind::AuthenticatedConsensusValue,
        availability: AvailabilityClass::Public,
        activation: RequirementActivation::Always,
    };
    assert!(
        sponsor_amount
            .layout_mut()
            .insert(LayoutRequirement::MakeSourceAvailable {
                relation: conservation(),
                case: expected_case(SponsorCase::Absent),
                carrier: CarrierRole::OperationGlobal {
                    operation: BURN,
                    anchor: ObjectId::ReceiptLive,
                },
                source,
            })
    );
    assert_eq!(
        validate_mixed_burn_plan(&analyzed, &sponsor_amount),
        Err(CompileError::SponsorValueRead),
    );
}

#[test]
fn normalized_input_with_private_change_is_admitted() {
    let control = super::source_tests::burn_mixed_control();
    let mixed = plan_burn_target_operation_with_output_selection(
        &control.input,
        limits(),
        RepresentationMode::PrivateCommitted,
    )
    .expect("public mixed Burn selection");
    check_side_and_evidence(&mixed);
    let explicit =
        plan_burn_target_operation(&control.input, limits()).expect("former public explicit route");
    assert!(matches!(
        &explicit.relation(&conservation()).expect("explicit conservation").proof,
        ProofDisposition::Selected { proof } if proof.proof() == ProofKind::PublicArithmetic
    ));
    let optional_change = family(
        RelationKind::Cardinality,
        TransactionSide::Output,
        ObjectId::ReceiptLive,
    );
    assert!(explicit.relation(&optional_change).is_some());
    assert!(mixed.relation(&optional_change).is_some());
    assert_eq!(
        plan_burn_target_operation_with_output_selection(
            &control.input,
            limits(),
            RepresentationMode::Explicit,
        ),
        Ok(explicit.clone()),
    );
    for id in [
        family(
            RelationKind::Cardinality,
            TransactionSide::Output,
            ObjectId::Ash,
        ),
        family(
            RelationKind::Recognition,
            TransactionSide::Output,
            ObjectId::Ash,
        ),
        relation(
            RelationKind::Representation,
            RelationSubject::Representation {
                object: ObjectId::Ash,
            },
        ),
        relation(RelationKind::DataOutputs, RelationSubject::Operation),
    ] {
        let old = explicit
            .relation(&id)
            .expect("explicit ASH constructor row");
        let new = mixed.relation(&id).expect("mixed ASH constructor row");
        assert_eq!(new.proof, old.proof);
        assert_eq!(new.representation, old.representation);
        assert_eq!(new.source_requirements, old.source_requirements);
        assert_eq!(new.required_capabilities, old.required_capabilities);
        assert_eq!(new.external_evidence, old.external_evidence);
        assert_eq!(new.lifecycle, old.lifecycle);
        for sponsor in [SponsorCase::Absent, SponsorCase::Present] {
            let old_case = &old.cases[&ExecutionCaseId {
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
                ..expected_case(sponsor)
            }];
            let new_case = &new.cases[&expected_case(sponsor)];
            let mut normalized = new_case.clone();
            normalized.key.case = old_case.key.case.clone();
            normalized.layout_requirements = normalized
                .layout_requirements
                .into_iter()
                .map(|row| with_case_identity(row, &old_case.key.case))
                .collect();
            assert_eq!(&normalized, old_case);
        }
    }
    assert_eq!(
        plan_burn_target_operation_with_output_selection(
            &control.input,
            limits(),
            RepresentationMode::PublicCommitted,
        ),
        Err(CompileError::TargetPlanRepresentationMismatch {
            object: ObjectId::ReceiptLive,
        }),
    );
}

#[test]
fn private_input_remains_refused_after_private_change_activation() {
    let control = super::source_tests::burn_mixed_control();
    let analyzed = analyze_scoped_program(&control.input, limits()).expect("complete analysis");
    let plan = plan_burn_target_operation_with_output_selection(
        &control.input,
        limits(),
        RepresentationMode::PrivateCommitted,
    )
    .expect("normalized public mixed route");
    assert_eq!(validate_mixed_burn_plan(&analyzed, &plan), Ok(()));
    let mut changed = plan;
    changed.selections_mut().insert(
        choice(ObjectId::ReceiptLive, Some(TransactionSide::Input)),
        RepresentationMode::PrivateCommitted,
    );
    assert_eq!(
        validate_mixed_burn_plan(&analyzed, &changed),
        Err(CompileError::TargetPlanRepresentationMismatch {
            object: ObjectId::ReceiptLive,
        }),
    );
}
