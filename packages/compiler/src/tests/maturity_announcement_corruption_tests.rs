//! Corrupt retained obligations, without claiming execution of future transactions.
//! Public validation rebinds the input; cached analyzed validation additionally
//! permits coherent factor damage that a bound input cannot represent.

use std::{collections::BTreeSet, sync::LazyLock};

use architecture::{AssetId, BoundId, ObjectId, OperationId, RootId};
use realization::{
    ArchitectureBinding, Count, DisclosureNodeId, FactId, Relation, RelationId, RelationKind,
    RelationSubject, RepresentationMode, StateField, TransactionSide,
};

use super::announcement_input;
use crate::{
    AnnouncementConstructorRole, AnnouncementMetadataRequirement,
    AnnouncementRecoveryInputRole as RecoveryInput, AnnouncementRecoveryStep,
    AnnouncementRequirementMutation as RequirementMutation,
    AnnouncementRootHistoryCheck as History, CompileError, StateFieldLaw, StateFieldLawKind,
    StateLawOperand,
    analyzed::{ScopedAnalyzedProgram, analyze_scoped_program},
    coverage_graph::CoverageNodeId,
    maturity_announcement_plan::{
        MaturityAnnouncementClause as Clause, MaturityAnnouncementRepresentationPlan as Mode,
        MaturityAnnouncementRepresentationProjection as Projection,
        ValidatedMaturityAnnouncementOperationPlan as Plan, derive_plan,
        plan_maturity_announcement_target_operation, validate_analyzed_maturity_announcement_plan,
        validate_evidence_closure, validate_maturity_announcement_plan,
        validate_representation_equivalence, validate_sponsor_erasure,
    },
    operation_plan::{
        CardinalityCeiling, LayoutRequirement, OperandId, OperandRole, PlacementSearchLimits,
        ProofDisposition, RelationActivity, RelationMutation as Mutation, SourceRequirement,
        TargetCoverageObligation, TargetOperationSource, TargetRelationRequirement,
    },
};

const OPERATION: OperationId = OperationId::AnnounceMaturity;
static ANALYZED: LazyLock<ScopedAnalyzedProgram> =
    LazyLock::new(|| analyze_scoped_program(&announcement_input(), limits()).unwrap());
static PLAN: LazyLock<Plan> = LazyLock::new(|| derive_plan(&ANALYZED).unwrap());

fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        10_000_000_u64.try_into().unwrap(),
        1_000_000_u64.try_into().unwrap(),
    )
}

fn projection(plan: &mut Plan) -> &mut Projection {
    plan.representations_mut().get_mut(&Mode::Explicit).unwrap()
}

fn relation(kind: RelationKind, subject: RelationSubject) -> RelationId {
    RelationId::new(OPERATION, kind, subject)
}

fn policy(kind: RelationKind) -> RelationId {
    relation(kind, RelationSubject::Operation)
}

fn family(kind: RelationKind, side: TransactionSide, object: ObjectId) -> RelationId {
    relation(kind, RelationSubject::ObjectFamily { side, object })
}

fn state_count() -> RelationId {
    family(
        RelationKind::Cardinality,
        TransactionSide::Input,
        ObjectId::State,
    )
}

fn row<'a>(plan: &'a mut Plan, id: &RelationId) -> &'a mut TargetRelationRequirement {
    projection(plan).relations_mut().get_mut(id).unwrap()
}

fn defect(clause: Clause) -> CompileError {
    CompileError::MaturityAnnouncementContractDefect { clause }
}

fn refuses(plan: &Plan, error: CompileError) {
    assert_eq!(
        validate_maturity_announcement_plan(plan, &announcement_input()),
        Err(error)
    );
}

fn corrupt(change: impl FnOnce(&mut Plan), error: CompileError) {
    let mut plan = PLAN.clone();
    change(&mut plan);
    refuses(&plan, error);
}

#[test]
fn unmodified_plan_passes_every_validator() {
    validate_maturity_announcement_plan(&PLAN, &announcement_input()).unwrap();
    validate_analyzed_maturity_announcement_plan(&ANALYZED, &PLAN).unwrap();
    validate_evidence_closure(&PLAN).unwrap();
    validate_representation_equivalence(&PLAN).unwrap();
    validate_sponsor_erasure(&PLAN).unwrap();
    assert_eq!(
        plan_maturity_announcement_target_operation(&announcement_input(), limits()).unwrap(),
        *PLAN
    );
}

#[test]
fn wrong_operation_is_rejected() {
    corrupt(
        |plan| *plan.operation_mut() = OperationId::Clear,
        CompileError::TargetOperationOutOfScope {
            operation: OperationId::Clear,
        },
    );
}

fn corrupt_source(change: impl FnOnce(&mut ScopedAnalyzedProgram)) {
    let mut analyzed = ANALYZED.clone();
    change(&mut analyzed);
    corrupt(
        |plan| *plan.source_mut() = TargetOperationSource::of(&analyzed),
        CompileError::TargetPlanSourceMismatch,
    );
}

#[test]
fn architecture_binding_mismatch_is_rejected() {
    let mut architecture = architecture::ARCHITECTURE;
    architecture.document.realization_version = "0.5.0-dev";
    let binding = ArchitectureBinding::from_architecture(&architecture).unwrap();
    corrupt_source(|analyzed| analyzed.source.architecture = binding);
}

#[test]
fn realization_body_mismatch_is_rejected() {
    corrupt_source(|analyzed| {
        analyzed
            .source
            .realization
            .relations
            .nodes
            .iter_mut()
            .find(|declaration| declaration.id == state_count())
            .unwrap()
            .relation = Relation::OperatorAuthorization;
    });
}

#[test]
fn scope_mismatch_is_rejected() {
    corrupt_source(|analyzed| {
        analyzed.source.compilation_scope =
            crate::CompilationScope::from_operations([OperationId::Clear]).unwrap();
    });
}

#[test]
fn declaration_asset_projection_mismatch_is_rejected() {
    corrupt(
        |plan| *plan.state_mut().asset_mut() = AssetId::Lbtc,
        defect(Clause::ClassClosure),
    );
}

#[test]
fn relation_omission_is_rejected() {
    corrupt(
        |plan| {
            projection(plan).relations_mut().remove(&state_count());
        },
        CompileError::TargetPlanRelationCensusMismatch {
            relation: state_count(),
        },
    );
}

#[test]
fn duplicate_relation_value_under_an_existing_key_is_rejected() {
    // Maps preclude a duplicate key. A duplicate value must not replace another obligation.
    corrupt(
        |plan| {
            let duplicate = row(plan, &state_count()).clone();
            projection(plan)
                .relations_mut()
                .insert(policy(RelationKind::Authorization), duplicate);
        },
        CompileError::TargetPlanRelationRequirementMismatch,
    );
}

#[test]
fn extra_relation_identity_is_rejected() {
    let extra = RelationId::new(
        OperationId::Clear,
        RelationKind::Authorization,
        RelationSubject::Operation,
    );
    corrupt(
        |plan| {
            let mut added = row(plan, &state_count()).clone();
            added.relation = extra.clone();
            projection(plan)
                .relations_mut()
                .insert(extra.clone(), added);
        },
        CompileError::TargetPlanRelationCensusMismatch {
            relation: extra.clone(),
        },
    );
}

#[test]
fn altered_relation_requirement_body_is_rejected() {
    corrupt(
        |plan| row(plan, &state_count()).proof = ProofDisposition::StaticallyValidated,
        CompileError::TargetPlanRelationRequirementMismatch,
    );
}

#[test]
fn case_mode_change_is_rejected() {
    corrupt(
        |plan| {
            projection(plan)
                .cases_mut()
                .values_mut()
                .next()
                .unwrap()
                .id
                .representations
                .insert(ObjectId::State, RepresentationMode::PrivateCommitted);
        },
        CompileError::TargetPlanCaseCensusMismatch,
    );
}

#[test]
fn relation_case_activity_flip_is_rejected() {
    corrupt(
        |plan| {
            row(plan, &state_count())
                .cases
                .values_mut()
                .next()
                .unwrap()
                .activity = RelationActivity::Vacuous;
        },
        CompileError::TargetPlanRelationRequirementMismatch,
    );
}

#[test]
fn altered_discharge_boundary_is_rejected() {
    corrupt(
        |plan| {
            row(plan, &state_count())
                .cases
                .values_mut()
                .next()
                .unwrap()
                .boundaries
                .clear();
        },
        CompileError::TargetPlanRelationRequirementMismatch,
    );
}

#[test]
fn missing_representation_is_rejected() {
    corrupt(
        |plan| {
            plan.representations_mut().remove(&Mode::PublicCommitted);
        },
        defect(Clause::RepresentationApproval),
    );
}

#[test]
fn representation_key_value_disagreement_is_rejected() {
    corrupt(
        |plan| *projection(plan).mode_mut() = Mode::PublicCommitted,
        defect(Clause::RepresentationApproval),
    );
    assert_eq!(Mode::of(RepresentationMode::PrivateCommitted), None);
    assert_eq!(Mode::ALL, &[Mode::Explicit, Mode::PublicCommitted]);
}

#[test]
fn missing_factor_is_rejected() {
    let mut analyzed = ANALYZED.clone();
    analyzed.proof_plans.values_mut().for_each(|candidate| {
        candidate.operations.remove(&OPERATION);
    });
    assert_eq!(
        validate_analyzed_maturity_announcement_plan(&analyzed, &PLAN),
        Err(CompileError::MissingMaturityAnnouncementRepresentation {
            operation: OPERATION,
            representation: Mode::Explicit
        })
    );
}

#[test]
fn coherent_relation_and_edge_omission_from_every_factor_is_rejected() {
    let mut analyzed = ANALYZED.clone();
    let omitted = state_count();
    for candidate in analyzed.proof_plans.values_mut() {
        candidate.relation_requirements.remove(&omitted);
        let factor = candidate.operations.get_mut(&OPERATION).unwrap();
        factor
            .relation_cases
            .retain(|key, _| key.relation != omitted);
        factor
            .coverage
            .requirements
            .retain(|key, _| key.relation != omitted);
        factor.coverage_dependencies.edges.retain(|edge| {
            [&edge.source, &edge.target].into_iter().all(|node| {
                !matches!(node, CoverageNodeId::RelationCase(key) if key.relation == omitted)
            })
        });
    }
    let plan = derive_plan(&analyzed).unwrap();
    assert_eq!(
        validate_analyzed_maturity_announcement_plan(&analyzed, &plan),
        Err(CompileError::TargetPlanRelationCensusMismatch { relation: omitted })
    );
}

#[test]
fn missing_carrier_is_rejected() {
    corrupt(
        |plan| {
            projection(plan).carriers_mut().pop_first();
        },
        CompileError::TargetPlanCarrierCensusMismatch,
    );
}

#[test]
fn missing_layout_is_rejected() {
    corrupt(
        |plan| {
            projection(plan).layout_mut().pop_first();
        },
        CompileError::TargetPlanLayoutCensusMismatch,
    );
}

#[test]
fn missing_capability_is_rejected() {
    corrupt(
        |plan| {
            projection(plan).capabilities_mut().pop_first();
        },
        CompileError::TargetPlanCapabilityCensusMismatch,
    );
}

#[test]
fn cross_mode_semantic_divergence_is_rejected() {
    let mut plan = PLAN.clone();
    row(&mut plan, &state_count()).source_requirements.clear();
    assert_eq!(
        validate_representation_equivalence(&plan),
        Err(CompileError::TargetPlanRelationRequirementMismatch)
    );
    refuses(&plan, CompileError::TargetPlanRelationRequirementMismatch);
}

#[test]
fn case_evidence_cannot_disagree_with_relation_evidence() {
    let mut plan = PLAN.clone();
    row(&mut plan, &policy(RelationKind::Authorization))
        .cases
        .values_mut()
        .next()
        .unwrap()
        .external_evidence
        .clear();
    assert_eq!(
        validate_evidence_closure(&plan),
        Err(CompileError::TargetPlanEvidenceCensusMismatch)
    );
    refuses(&plan, CompileError::TargetPlanRelationRequirementMismatch);
}

#[test]
fn aggregate_evidence_must_equal_the_active_union() {
    let mut plan = PLAN.clone();
    projection(&mut plan).evidence_mut().clear();
    assert_eq!(
        validate_evidence_closure(&plan),
        Err(CompileError::TargetPlanEvidenceCensusMismatch)
    );
    refuses(&plan, CompileError::TargetPlanEvidenceCensusMismatch);
}

#[test]
fn coherent_case_and_coverage_evidence_loss_still_fails_the_join() {
    let mut analyzed = ANALYZED.clone();
    let authorization = policy(RelationKind::Authorization);
    for candidate in analyzed.proof_plans.values_mut() {
        let factor = candidate.operations.get_mut(&OPERATION).unwrap();
        for (key, bundle) in &mut factor.relation_cases {
            if key.relation == authorization {
                bundle.external_evidence.clear();
                bundle.coverage.external_evidence.clear();
            }
        }
        for (key, coverage) in &mut factor.coverage.requirements {
            if key.relation == authorization {
                coverage.external_evidence.clear();
            }
        }
    }
    let plan = derive_plan(&analyzed).unwrap();
    assert_eq!(
        validate_analyzed_maturity_announcement_plan(&analyzed, &plan),
        Err(CompileError::TargetPlanEvidenceCensusMismatch)
    );
}

type Catalogue = std::collections::BTreeMap<RelationId, BTreeSet<Mutation>>;

fn family_catalogue() -> Catalogue {
    let mut expected = Catalogue::new();
    for side in [TransactionSide::Input, TransactionSide::Output] {
        expected.insert(
            family(RelationKind::Cardinality, side, ObjectId::State),
            BTreeSet::from([
                Mutation::CardinalityBelowMinimum,
                Mutation::CardinalityAboveMaximum {
                    ceiling: CardinalityCeiling::Declared(Count::ONE),
                },
            ]),
        );
        expected.insert(
            family(RelationKind::Cardinality, side, ObjectId::PlainLbtc),
            BTreeSet::from([Mutation::CardinalityAboveMaximum {
                ceiling: if side == TransactionSide::Input {
                    CardinalityCeiling::Bound(BoundId::FeeSponsorInputMax)
                } else {
                    CardinalityCeiling::Declared(Count::ONE)
                },
            }]),
        );
        for object in [ObjectId::State, ObjectId::PlainLbtc] {
            expected.insert(
                family(RelationKind::Recognition, side, object),
                BTreeSet::from([
                    Mutation::WrongRecognizedAsset,
                    Mutation::WrongRecognizedObject,
                ]),
            );
        }
        expected.insert(
            relation(
                RelationKind::AllowedObjectFamilies,
                RelationSubject::TransactionSide { side },
            ),
            BTreeSet::from([Mutation::UndeclaredObjectFamily]),
        );
    }
    expected
}

fn policy_catalogue() -> Catalogue {
    [
        (
            policy(RelationKind::CanonicalDeltaPolicy),
            vec![
                Mutation::MissingCanonicalDeltaFamily,
                Mutation::UnexpectedCanonicalDeltaFamily,
                Mutation::DuplicateCanonicalSourceOrDestination,
            ],
        ),
        (
            policy(RelationKind::RootPolicy),
            vec![Mutation::WrongRootEffect],
        ),
        (
            policy(RelationKind::ProjectionPolicy),
            vec![
                Mutation::MissingRequiredProjection,
                Mutation::ForbiddenProjectionPresent,
            ],
        ),
        (
            policy(RelationKind::OpenFlowPolicy),
            vec![Mutation::UndeclaredOpenFlow],
        ),
        (
            relation(RelationKind::SponsorIsolation, RelationSubject::Sponsor),
            vec![
                Mutation::SponsorProtocolOverlap,
                Mutation::MissingSponsorAuthorization,
            ],
        ),
        (
            relation(
                RelationKind::SponsorEnvelopeMultiplicity,
                RelationSubject::Sponsor,
            ),
            vec![Mutation::SponsorEnvelopeMultiplicityExceeded],
        ),
        (
            relation(
                RelationKind::Representation,
                RelationSubject::Representation {
                    object: ObjectId::State,
                },
            ),
            vec![
                Mutation::UnsupportedRepresentation,
                Mutation::UnauthenticatedRepresentation,
            ],
        ),
    ]
    .into_iter()
    .map(|(id, mutations)| (id, mutations.into_iter().collect()))
    .collect()
}

fn catalogue() -> Catalogue {
    let mut expected = family_catalogue();
    expected.extend(policy_catalogue());
    for id in [
        policy(RelationKind::Authorization),
        policy(RelationKind::Constructibility),
        relation(
            RelationKind::SubstrateConservation,
            RelationSubject::Asset {
                asset: AssetId::Lbtc,
            },
        ),
    ] {
        expected.insert(
            id,
            BTreeSet::from([
                Mutation::ExternalEvidenceMissing,
                Mutation::ExternalEvidenceFailed,
                Mutation::ExternalEvidenceIdentityMismatch,
            ]),
        );
    }
    for exit in [
        OperationId::AdmitDeposits,
        OperationId::Cycle,
        OperationId::Redeem,
        OperationId::ReceiptRelabel,
        OperationId::Clear,
        OPERATION,
    ] {
        expected.insert(
            relation(
                RelationKind::Lifecycle,
                RelationSubject::LifecycleExit {
                    object: ObjectId::State,
                    exit,
                },
            ),
            BTreeSet::from([Mutation::RequiredLifecycleExitMissing]),
        );
    }
    expected
}

#[test]
fn every_relation_and_sponsor_case_has_the_literal_mutation_catalogue() {
    let expected = catalogue();
    assert_eq!(expected.len(), 26);
    for projection in PLAN.representations() {
        for case in projection.cases() {
            for (id, mutations) in &expected {
                let actual: BTreeSet<_> = projection
                    .coverage()
                    .filter_map(|row| {
                        if row.id.relation != *id || row.id.case != case.id {
                            return None;
                        }
                        match &row.obligation {
                            TargetCoverageObligation::Negative(negative) => Some(negative.mutation),
                            TargetCoverageObligation::Positive(_) => None,
                        }
                    })
                    .collect();
                let inactive = case.id.sponsor == crate::case::SponsorCase::Absent
                    && matches!(
                        id.subject(),
                        RelationSubject::ObjectFamily {
                            object: ObjectId::PlainLbtc,
                            ..
                        }
                    );
                assert_eq!(
                    actual,
                    if inactive {
                        BTreeSet::new()
                    } else {
                        mutations.clone()
                    },
                    "{id:?}, {:?}",
                    case.id
                );
            }
        }
    }
}

fn require_negative_rows(kinds: &[RelationKind]) {
    let mut exercised = BTreeSet::new();
    for mode in Mode::ALL {
        let projection = PLAN.projection(*mode).unwrap();
        for original in projection
            .coverage()
            .filter(|row| kinds.contains(&row.id.relation.kind()))
        {
            let TargetCoverageObligation::Negative(negative) = &original.obligation else {
                continue;
            };
            exercised.insert(negative.mutation);
            let mut plan = PLAN.clone();
            plan.representations_mut()
                .get_mut(mode)
                .unwrap()
                .coverage_mut()
                .remove(&original.id);
            assert_eq!(
                validate_analyzed_maturity_announcement_plan(&ANALYZED, &plan),
                Err(CompileError::TargetPlanCoverageCensusMismatch),
                "{:?}",
                original.id
            );
        }
    }
    let expected: BTreeSet<_> = catalogue()
        .into_iter()
        .filter(|(id, _)| kinds.contains(&id.kind()))
        .flat_map(|(_, mutations)| mutations)
        .collect();
    assert_eq!(exercised, expected);
    assert!(!exercised.is_empty());
}

#[test]
fn every_cardinality_negative_is_required() {
    require_negative_rows(&[RelationKind::Cardinality]);
}

#[test]
fn every_recognition_and_family_negative_is_required() {
    require_negative_rows(&[
        RelationKind::Recognition,
        RelationKind::AllowedObjectFamilies,
    ]);
}

#[test]
fn every_sponsor_and_open_flow_negative_is_required() {
    require_negative_rows(&[
        RelationKind::SponsorIsolation,
        RelationKind::SponsorEnvelopeMultiplicity,
        RelationKind::OpenFlowPolicy,
    ]);
}

#[test]
fn every_root_and_projection_negative_is_required() {
    require_negative_rows(&[RelationKind::RootPolicy, RelationKind::ProjectionPolicy]);
}

#[test]
fn every_external_evidence_negative_is_required() {
    require_negative_rows(&[
        RelationKind::Authorization,
        RelationKind::SubstrateConservation,
        RelationKind::Constructibility,
    ]);
}

#[test]
fn every_representation_and_lifecycle_negative_is_required() {
    require_negative_rows(&[RelationKind::Representation, RelationKind::Lifecycle]);
}

#[test]
fn empty_canonical_policy_keeps_all_three_derived_negatives() {
    assert!(PLAN.canonical().expected().is_empty());
    assert_eq!(
        catalogue()[&policy(RelationKind::CanonicalDeltaPolicy)],
        BTreeSet::from([
            Mutation::MissingCanonicalDeltaFamily,
            Mutation::UnexpectedCanonicalDeltaFamily,
            Mutation::DuplicateCanonicalSourceOrDestination,
        ])
    );
    require_negative_rows(&[RelationKind::CanonicalDeltaPolicy]);
    // No focused runtime witness is claimed: the expected family set is empty;
    // duplicate partition references fail observation normalization first.
}

#[test]
fn unrelated_mutation_classes_have_no_announcement_rows() {
    let present: BTreeSet<_> = catalogue().into_values().flatten().collect();
    for absent in [
        Mutation::AmountMismatch,
        Mutation::MissingRequiredOwner,
        Mutation::UnexpectedProtocolSecret,
        Mutation::ConstructibilityWitnessUnavailable,
        Mutation::PermissionlessPrivateDependency,
        Mutation::ExpressionPredicateFalse,
    ] {
        assert!(!present.contains(&absent));
    }
}

fn omit_history(plan: &mut Plan, omitted: History) {
    plan.history_mut().checks = Box::leak(
        History::ALL
            .iter()
            .copied()
            .filter(|check| *check != omitted)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
}

fn mutate_metadata(plan: &mut Plan, mutation: RequirementMutation) {
    let metadata = plan.transition_mut();
    match mutation {
        // The maturity enum has only Unannounced; corrupt its required subject.
        RequirementMutation::InvalidPredecessorMaturity => {
            metadata.fields[5].input = metadata.input_cycle.clone();
        }
        RequirementMutation::LeadWindowBelow => {
            metadata.minimum_lead = metadata.maximum_lead.clone();
        }
        RequirementMutation::LeadWindowAbove => {
            metadata.maximum_lead = metadata.minimum_lead.clone();
        }
        RequirementMutation::LeadWindowOverflowing => {
            metadata.input_cycle = metadata.requested_cycle.clone();
        }
        RequirementMutation::ChangedPreservedField => {
            metadata.fields[0].law.kind = StateFieldLawKind::ZeroAmount;
        }
        RequirementMutation::WrongSuccessorMaturity => {
            metadata.fields[5].law = StateFieldLaw {
                kind: StateFieldLawKind::Copy,
                operands: Vec::new(),
            };
        }
        // A lead selector cannot carry an arbitrary bound ID. Substitute a foreign bound fact.
        RequirementMutation::AlteredLeadBound => {
            metadata.minimum_lead = FactId::BoundValue {
                bound: BoundId::FeeSponsorInputMax,
            };
        }
        _ => panic!("not a metadata mutation"),
    }
}

fn mutate_requirement(plan: &mut Plan, mutation: RequirementMutation) {
    match mutation {
        RequirementMutation::UnauthenticatedPredecessorField => {
            plan.constructor_mut().predecessor = AnnouncementConstructorRole::ReconstructSuccessor;
        }
        RequirementMutation::InvalidPredecessorMaturity
        | RequirementMutation::LeadWindowBelow
        | RequirementMutation::LeadWindowAbove
        | RequirementMutation::LeadWindowOverflowing
        | RequirementMutation::ChangedPreservedField
        | RequirementMutation::WrongSuccessorMaturity
        | RequirementMutation::AlteredLeadBound => mutate_metadata(plan, mutation),
        RequirementMutation::ConstructorDiscontinuity => {
            plan.constructor_mut().successor = AnnouncementConstructorRole::AuthenticatePredecessor;
        }
        RequirementMutation::StaticPolicyDiscontinuity => {
            // Static and policy continuity are singleton types. Remove their public recipe dependency.
            plan.recovery_mut().inputs = Box::leak(
                RecoveryInput::ALL
                    .iter()
                    .copied()
                    .filter(|role| *role != RecoveryInput::StaticConstructorRecipeOrReference)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            );
        }
        RequirementMutation::MissingRecoveryInput => {
            plan.recovery_mut().inputs = &RecoveryInput::ALL[1..];
        }
        RequirementMutation::ReconstructionMismatch => {
            plan.recovery_mut().steps = &AnnouncementRecoveryStep::ALL[..5];
        }
        RequirementMutation::RootHistoryStale => omit_history(plan, History::RejectStaleHistory),
        RequirementMutation::RootHistoryWrongEndpoint => {
            plan.history_mut().succession.root = RootId::Resv;
        }
        RequirementMutation::RootHistoryWrongCertificate => {
            plan.history_mut().succession.certificate_relation = policy(RelationKind::RootPolicy);
        }
        RequirementMutation::RootHistoryIntermediate => {
            omit_history(plan, History::EveryIntermediateEdgeValid);
        }
        RequirementMutation::RootHistoryReorgInvalid => {
            omit_history(plan, History::CheckpointAndReorgBinding);
        }
    }
}

fn requirement_corruption(mutation: RequirementMutation) {
    corrupt(
        |plan| mutate_requirement(plan, mutation),
        defect(Clause::Transition),
    );
}

#[test]
fn unauthenticated_predecessor_field_requirement_is_rejected() {
    requirement_corruption(RequirementMutation::UnauthenticatedPredecessorField);
}

#[test]
fn invalid_predecessor_maturity_requirement_is_rejected() {
    requirement_corruption(RequirementMutation::InvalidPredecessorMaturity);
}

#[test]
fn lead_window_below_requirement_is_rejected() {
    requirement_corruption(RequirementMutation::LeadWindowBelow);
}

#[test]
fn lead_window_above_requirement_is_rejected() {
    requirement_corruption(RequirementMutation::LeadWindowAbove);
}

#[test]
fn lead_window_overflowing_requirement_is_rejected() {
    requirement_corruption(RequirementMutation::LeadWindowOverflowing);
}

#[test]
fn changed_preserved_field_requirement_is_rejected() {
    requirement_corruption(RequirementMutation::ChangedPreservedField);
}

#[test]
fn wrong_successor_maturity_requirement_is_rejected() {
    requirement_corruption(RequirementMutation::WrongSuccessorMaturity);
}

#[test]
fn altered_lead_bound_requirement_is_rejected() {
    requirement_corruption(RequirementMutation::AlteredLeadBound);
}

#[test]
fn constructor_discontinuity_requirement_is_rejected() {
    requirement_corruption(RequirementMutation::ConstructorDiscontinuity);
}

#[test]
fn static_policy_discontinuity_requirement_is_rejected() {
    requirement_corruption(RequirementMutation::StaticPolicyDiscontinuity);
}

#[test]
fn missing_recovery_input_requirement_is_rejected() {
    requirement_corruption(RequirementMutation::MissingRecoveryInput);
}

#[test]
fn reconstruction_mismatch_requirement_is_rejected() {
    requirement_corruption(RequirementMutation::ReconstructionMismatch);
}

#[test]
fn root_history_stale_requirement_is_rejected() {
    requirement_corruption(RequirementMutation::RootHistoryStale);
}

#[test]
fn root_history_wrong_endpoint_requirement_is_rejected() {
    requirement_corruption(RequirementMutation::RootHistoryWrongEndpoint);
}

#[test]
fn root_history_wrong_certificate_requirement_is_rejected() {
    requirement_corruption(RequirementMutation::RootHistoryWrongCertificate);
}

#[test]
fn root_history_intermediate_requirement_is_rejected() {
    requirement_corruption(RequirementMutation::RootHistoryIntermediate);
}

#[test]
fn root_history_reorg_invalid_requirement_is_rejected() {
    requirement_corruption(RequirementMutation::RootHistoryReorgInvalid);
}

#[test]
fn requirement_mutation_census_is_exact_and_separate() {
    assert_eq!(
        RequirementMutation::ALL,
        &[
            RequirementMutation::UnauthenticatedPredecessorField,
            RequirementMutation::InvalidPredecessorMaturity,
            RequirementMutation::LeadWindowBelow,
            RequirementMutation::LeadWindowAbove,
            RequirementMutation::LeadWindowOverflowing,
            RequirementMutation::ChangedPreservedField,
            RequirementMutation::WrongSuccessorMaturity,
            RequirementMutation::AlteredLeadBound,
            RequirementMutation::ConstructorDiscontinuity,
            RequirementMutation::StaticPolicyDiscontinuity,
            RequirementMutation::MissingRecoveryInput,
            RequirementMutation::ReconstructionMismatch,
            RequirementMutation::RootHistoryStale,
            RequirementMutation::RootHistoryWrongEndpoint,
            RequirementMutation::RootHistoryWrongCertificate,
            RequirementMutation::RootHistoryIntermediate,
            RequirementMutation::RootHistoryReorgInvalid,
        ]
    );
}

#[test]
fn supplemental_requirement_orphan_is_rejected() {
    corrupt(
        |plan| {
            plan.recovery_mut().source_facts[0] = FactId::StateField {
                operation: OperationId::Clear,
                side: TransactionSide::Input,
                field: StateField::Omega,
            }
        },
        defect(Clause::Transition),
    );
}

#[test]
fn duplicate_supplemental_requirement_is_rejected() {
    corrupt(
        |plan| {
            let fields = &mut plan.transition_mut().fields;
            fields[0] = fields[1].clone();
        },
        defect(Clause::Transition),
    );
    corrupt(
        |plan| {
            plan.history_mut().checks =
                &[History::ExactlyOneStateEdge, History::ExactlyOneStateEdge];
        },
        defect(Clause::Transition),
    );
}

#[test]
fn supplemental_requirement_insertion_is_rejected() {
    corrupt(
        |plan| {
            plan.recovery_mut().inputs = Box::leak(
                RecoveryInput::ALL
                    .iter()
                    .copied()
                    .chain([RecoveryInput::SuccessorNonce])
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            );
        },
        defect(Clause::Transition),
    );
}

#[test]
fn succession_dependency_omission_is_rejected() {
    corrupt(
        |plan| plan.history_mut().succession.input_fields = None,
        defect(Clause::Transition),
    );
    corrupt(
        |plan| plan.history_mut().succession.output_fields = None,
        defect(Clause::Transition),
    );
}

#[test]
fn public_fact_duplication_is_rejected() {
    corrupt(
        |plan| plan.facts_mut()[0] = AnnouncementMetadataRequirement::public_facts()[1].clone(),
        defect(Clause::PublicFacts),
    );
}

fn erased_fact() -> FactId {
    FactId::FamilyAmount {
        operation: OPERATION,
        side: TransactionSide::Input,
        object: ObjectId::PlainLbtc,
    }
}

fn erased_source() -> SourceRequirement {
    let mut source = PLAN
        .projection(Mode::Explicit)
        .unwrap()
        .relation(&state_count())
        .unwrap()
        .source_requirements
        .iter()
        .next()
        .unwrap()
        .clone();
    source.operand = OperandId::new(
        state_count(),
        OperandRole::ObjectFamilyAmount {
            side: TransactionSide::Input,
            object: ObjectId::PlainLbtc,
        },
    );
    source
}

fn erased_layout() -> LayoutRequirement {
    let mut layout = PLAN
        .projection(Mode::Explicit)
        .unwrap()
        .layout()
        .find(|requirement| matches!(requirement, LayoutRequirement::MakeSourceAvailable { .. }))
        .unwrap()
        .clone();
    if let LayoutRequirement::MakeSourceAvailable { source, .. } = &mut layout {
        *source = erased_source();
    }
    layout
}

fn erasure_corruption(change: impl FnOnce(&mut Plan), earlier_error: CompileError) {
    let mut plan = PLAN.clone();
    change(&mut plan);
    // Independent traversal must fire even where earlier factor equality also refuses the plan.
    assert_eq!(
        validate_sponsor_erasure(&plan),
        Err(CompileError::SponsorValueRead)
    );
    refuses(&plan, earlier_error);
}

#[test]
fn sponsor_value_in_global_layout_is_rejected() {
    erasure_corruption(
        |plan| {
            projection(plan).layout_mut().insert(erased_layout());
        },
        CompileError::TargetPlanLayoutCensusMismatch,
    );
}

#[test]
fn sponsor_value_in_relation_case_layout_is_rejected() {
    // TargetRelationRequirement owns its layout through cases, with no separate relation layout.
    erasure_corruption(
        |plan| {
            row(plan, &state_count())
                .cases
                .values_mut()
                .next()
                .unwrap()
                .layout_requirements
                .insert(erased_layout());
        },
        CompileError::TargetPlanRelationRequirementMismatch,
    );
}

#[test]
fn sponsor_value_in_carrier_alternative_layout_is_rejected() {
    erasure_corruption(
        |plan| {
            let carriers = projection(plan).carriers_mut();
            let mut carrier = carriers.pop_first().unwrap();
            let mut alternative = carrier.alternatives.pop_first().unwrap();
            alternative.layout.insert(erased_layout());
            carrier.alternatives.insert(alternative);
            carriers.insert(carrier);
        },
        CompileError::TargetPlanCarrierCensusMismatch,
    );
}

#[test]
fn sponsor_value_in_relation_sources_is_rejected() {
    erasure_corruption(
        |plan| {
            row(plan, &state_count())
                .source_requirements
                .insert(erased_source());
        },
        CompileError::TargetPlanRelationRequirementMismatch,
    );
}

#[test]
fn sponsor_value_in_active_case_sources_is_rejected() {
    erasure_corruption(
        |plan| {
            row(plan, &state_count())
                .cases
                .values_mut()
                .next()
                .unwrap()
                .active_sources
                .insert(erased_source());
        },
        CompileError::TargetPlanRelationRequirementMismatch,
    );
}

#[test]
fn sponsor_value_in_coverage_operands_is_rejected() {
    erasure_corruption(
        |plan| {
            let positive = projection(plan)
                .coverage_mut()
                .values_mut()
                .find_map(|row| match &mut row.obligation {
                    TargetCoverageObligation::Positive(positive) => Some(positive),
                    TargetCoverageObligation::Negative(_) => None,
                })
                .unwrap();
            positive.operands.push(erased_source().operand);
        },
        CompileError::TargetPlanCoverageCensusMismatch,
    );
}

fn comparison(plan: &mut Plan) -> &mut crate::operation_plan::SemanticProjectionRequirement {
    projection(plan)
        .coverage_mut()
        .values_mut()
        .find_map(|row| row.projection.as_mut())
        .unwrap()
}

#[test]
fn sponsor_value_in_projection_operands_is_rejected() {
    erasure_corruption(
        |plan| comparison(plan).operands.push(erased_source().operand),
        CompileError::TargetPlanCoverageCensusMismatch,
    );
}

#[test]
fn sponsor_value_in_projection_sources_is_rejected() {
    erasure_corruption(
        |plan| comparison(plan).sources.push(erased_source()),
        CompileError::TargetPlanCoverageCensusMismatch,
    );
}

#[test]
fn sponsor_value_in_coverage_carrier_layout_is_rejected() {
    erasure_corruption(
        |plan| {
            let carrier = &mut projection(plan)
                .coverage_mut()
                .values_mut()
                .find(|row| !row.carrier.is_empty())
                .unwrap()
                .carrier;
            let mut alternative = carrier.pop_first().unwrap();
            alternative.layout.insert(erased_layout());
            carrier.insert(alternative);
        },
        CompileError::TargetPlanCoverageCensusMismatch,
    );
}

fn source_erasure(change: impl FnOnce(&mut ScopedAnalyzedProgram)) {
    let mut analyzed = ANALYZED.clone();
    change(&mut analyzed);
    erasure_corruption(
        |plan| *plan.source_mut() = TargetOperationSource::of(&analyzed),
        CompileError::TargetPlanSourceMismatch,
    );
}

#[test]
fn sponsor_value_in_disclosure_facts_is_rejected() {
    source_erasure(|analyzed| {
        analyzed
            .source
            .realization
            .disclosure
            .nodes
            .push(DisclosureNodeId::Fact(erased_fact()));
    });
}

#[test]
fn sponsor_value_in_disclosure_edge_without_a_node_is_rejected() {
    source_erasure(|analyzed| {
        analyzed.source.realization.disclosure.edges.push(
            realization::DisclosureDependencyProjection {
                source: DisclosureNodeId::Fact(erased_fact()),
                target: DisclosureNodeId::Relation(state_count()),
                edge: realization::DisclosureEdge::RelationOperand,
            },
        );
    });
}

#[test]
fn sponsor_value_in_declassification_is_rejected() {
    source_erasure(|analyzed| {
        analyzed
            .source
            .realization
            .declassification
            .required_public
            .insert(erased_fact(), BTreeSet::new());
    });
    source_erasure(|analyzed| {
        analyzed
            .source
            .realization
            .declassification
            .newly_disclosed
            .insert(erased_fact(), BTreeSet::new());
    });
    source_erasure(|analyzed| {
        analyzed
            .source
            .realization
            .declassification
            .retained_private
            .insert(erased_fact());
    });
}

#[test]
fn sponsor_value_in_supplemental_sources_is_rejected() {
    let changes: [fn(&mut Plan); 10] = [
        |plan| plan.transition_mut().input_cycle = erased_fact(),
        |plan| plan.transition_mut().requested_cycle = erased_fact(),
        |plan| plan.transition_mut().minimum_lead = erased_fact(),
        |plan| plan.transition_mut().maximum_lead = erased_fact(),
        |plan| plan.transition_mut().fields[0].input = erased_fact(),
        |plan| plan.transition_mut().fields[0].output = erased_fact(),
        |plan| {
            plan.transition_mut().fields[5].law.operands[0] = StateLawOperand::Fact(erased_fact());
        },
        |plan| plan.history_mut().succession.input_fields.as_mut().unwrap()[0] = erased_fact(),
        |plan| {
            plan.history_mut()
                .succession
                .output_fields
                .as_mut()
                .unwrap()[0] = erased_fact();
        },
        |plan| plan.recovery_mut().source_facts[0] = erased_fact(),
    ];
    for change in changes {
        erasure_corruption(change, defect(Clause::Transition));
    }
    erasure_corruption(
        |plan| plan.recovery_mut().result_facts[0] = erased_fact(),
        defect(Clause::Transition),
    );
    erasure_corruption(
        |plan| plan.facts_mut()[0] = erased_fact(),
        defect(Clause::PublicFacts),
    );
}

#[test]
fn private_factor_selection_is_rejected() {
    let mut analyzed = ANALYZED.clone();
    let selection = crate::lifecycle::RepresentationChoiceId {
        operation: OPERATION,
        object: ObjectId::State,
    };
    for candidate in analyzed.proof_plans.values_mut() {
        candidate
            .proof_plan
            .representations
            .insert(selection.clone(), RepresentationMode::PrivateCommitted);
    }
    assert_eq!(
        validate_analyzed_maturity_announcement_plan(&analyzed, &PLAN),
        Err(CompileError::MissingMaturityAnnouncementRepresentation {
            operation: OPERATION,
            representation: Mode::Explicit
        })
    );
}

#[test]
fn aggregate_evidence_cannot_add_an_unused_role() {
    let mut plan = PLAN.clone();
    projection(&mut plan)
        .evidence_mut()
        .insert(crate::target::ExternalEvidenceRole::ConfidentialValueConservation);
    assert_eq!(
        validate_evidence_closure(&plan),
        Err(CompileError::TargetPlanEvidenceCensusMismatch)
    );
    refuses(&plan, CompileError::TargetPlanEvidenceCensusMismatch);
}

#[test]
fn coverage_evidence_must_agree_with_its_case() {
    let mut plan = PLAN.clone();
    projection(&mut plan)
        .coverage_mut()
        .values_mut()
        .find(|row| !row.external_evidence.is_empty())
        .unwrap()
        .external_evidence
        .clear();
    assert_eq!(
        validate_evidence_closure(&plan),
        Err(CompileError::TargetPlanEvidenceCensusMismatch)
    );
    refuses(&plan, CompileError::TargetPlanCoverageCensusMismatch);
}

#[test]
fn singleton_conditions_and_computed_boundaries_admit_no_alternate_row() {
    assert_eq!(
        crate::AnnouncementPredecessorMaturity::ALL,
        &[crate::AnnouncementPredecessorMaturity::Unannounced]
    );
    assert_eq!(
        crate::AnnouncementStaticContinuity::ALL,
        &[crate::AnnouncementStaticContinuity::SameLinkedStaticSubtree]
    );
    assert_eq!(
        crate::AnnouncementPolicyContinuity::ALL,
        &[crate::AnnouncementPolicyContinuity::SameLeafVersionAndInternalKeyPolicy]
    );
    assert_eq!(
        crate::AnnouncementDuty::CurrentRootFreshness.boundary(),
        crate::AnnouncementRequirementBoundary::RootHistoryReport
    );
    assert_eq!(
        crate::AnnouncementDuty::PublicReconstruction.boundary(),
        crate::AnnouncementRequirementBoundary::PublicRecoveryReport
    );
}

#[test]
fn public_fact_contract_precedes_factor_comparison() {
    corrupt(
        |plan| {
            plan.facts_mut()[0] = AnnouncementMetadataRequirement::public_facts()[1].clone();
            projection(plan).relations_mut().remove(&state_count());
        },
        defect(Clause::PublicFacts),
    );
}
