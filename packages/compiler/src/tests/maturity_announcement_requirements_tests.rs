use std::collections::BTreeSet;

use architecture::{OperationId, ProjectionId, ProjectionRule, RootId, RootUse};
use realization::{
    AnnouncementLeadBound, DisclosureNode, FactId, InitialVisibility, Relation, StateField,
    TransactionSide,
};

use crate::{
    AnnouncementConstructorRole, AnnouncementDuty, AnnouncementMetadataRequirement,
    AnnouncementPolicyContinuity, AnnouncementPredecessorMaturity, AnnouncementPublicationRole,
    AnnouncementRecoveryInputRole, AnnouncementRecoveryStep, AnnouncementRequirementBoundary,
    AnnouncementRootHistoryCheck, AnnouncementStaticContinuity, ConstructorContinuityRequirement,
    PublicRecoveryRequirement, RootHistoryRequirement, StateFieldLaw, StateFieldLawKind,
    StateFieldRequirement, StateFieldRequirementError, StateLawOperand, StateSuccessionRequirement,
};

#[test]
fn metadata_fields_are_exhaustive_and_symbolic() {
    let requirement = AnnouncementMetadataRequirement::required();
    assert_eq!(requirement.fields.len(), 6);
    assert_eq!(
        requirement
            .fields
            .each_ref()
            .map(|row| row.field)
            .as_slice(),
        StateField::ALL
    );
    for row in &requirement.fields {
        assert_eq!(row.input, field_key(TransactionSide::Input, row.field));
        assert_eq!(row.output, field_key(TransactionSide::Output, row.field));
        let expected = if row.field == StateField::Maturity {
            StateFieldLaw {
                kind: StateFieldLawKind::AnnounceRequestedCycle,
                operands: vec![StateLawOperand::Fact(requirement.requested_cycle.clone())],
            }
        } else {
            StateFieldLaw {
                kind: StateFieldLawKind::Copy,
                operands: Vec::new(),
            }
        };
        assert_eq!(row.law, expected);
        row.validate().unwrap();
    }
    assert_eq!(
        requirement.predecessor_maturity,
        AnnouncementPredecessorMaturity::Unannounced
    );
    assert_eq!(
        requirement.input_cycle,
        field_key(TransactionSide::Input, StateField::Cycle)
    );
    assert_eq!(
        requirement.requested_cycle,
        FactId::RequestedAnnouncementCycle {
            operation: OperationId::AnnounceMaturity,
        }
    );
}

fn field_key(side: TransactionSide, field: StateField) -> FactId {
    FactId::StateField {
        operation: OperationId::AnnounceMaturity,
        side,
        field,
    }
}

fn assert_side_fields(fields: &[FactId; 6], side: TransactionSide) {
    let expected = StateField::ALL
        .iter()
        .map(|field| field_key(side, *field))
        .collect::<Vec<_>>();
    assert_eq!(fields.as_slice(), expected);
    assert_eq!(fields.iter().collect::<BTreeSet<_>>().len(), 6);
}

#[test]
fn announcement_admissibility_names_architecture_leads_outside_field_laws() {
    let requirement = AnnouncementMetadataRequirement::required();
    for (fact, selector, id) in [
        (
            &requirement.minimum_lead,
            AnnouncementLeadBound::Minimum,
            architecture::BoundId::MaturityLeadMin,
        ),
        (
            &requirement.maximum_lead,
            AnnouncementLeadBound::Maximum,
            architecture::BoundId::MaturityLeadMax,
        ),
    ] {
        assert_eq!(
            *fact,
            FactId::AnnouncementLead {
                operation: OperationId::AnnounceMaturity,
                bound: selector
            }
        );
        assert_eq!(selector.bound_id(), id);
        assert_eq!(id.unit(), architecture::BoundUnit::Cycle);
        assert!(requirement.fields.iter().all(|row| {
            !row.law
                .operands
                .contains(&StateLawOperand::Fact(fact.clone()))
        }));
    }
}

#[test]
fn fifteen_public_facts_equal_the_declaration() {
    let input = super::proof_tests::announcement_input();
    let declaration = input
        .realization()
        .operation(OperationId::AnnounceMaturity)
        .unwrap();
    let declared = declaration
        .disclosure_nodes
        .iter()
        .map(|node| match node {
            DisclosureNode::Fact {
                id,
                initial_visibility,
            } => {
                assert_eq!(*initial_visibility, InitialVisibility::Public);
                id.clone()
            }
            DisclosureNode::Relation { .. } => panic!("announcement publishes only facts"),
        })
        .collect::<Vec<_>>();
    let facts = AnnouncementMetadataRequirement::public_facts();
    assert_eq!(facts.len(), 15);
    assert_eq!(facts.iter().collect::<BTreeSet<_>>().len(), 15);
    assert_eq!(facts.as_slice(), declared);
}

#[test]
fn recovery_roles_and_steps_have_no_payloads() {
    use AnnouncementRecoveryInputRole as Input;
    use AnnouncementRecoveryStep as Step;
    let PublicRecoveryRequirement {
        publication,
        source_facts,
        result_facts,
        inputs,
        steps,
    } = PublicRecoveryRequirement::REQUIRED;
    assert_eq!(
        publication,
        AnnouncementPublicationRole::AcceptedTransactionWitnessAndOutputs
    );
    assert_eq!(AnnouncementPublicationRole::ALL, &[publication]);
    assert_eq!(
        inputs,
        &[
            Input::SuccessorNonce,
            Input::MetadataSchema,
            Input::StaticConstructorRecipeOrReference,
            Input::SuccessorOutputPosition,
            Input::TargetLeafVersion,
            Input::TargetInternalKeyPolicy,
        ]
    );
    assert_eq!(inputs.len(), 6);
    assert_eq!(inputs, AnnouncementRecoveryInputRole::ALL);
    let expected_sources = StateField::ALL
        .iter()
        .map(|field| field_key(TransactionSide::Input, *field))
        .chain([FactId::RequestedAnnouncementCycle {
            operation: OperationId::AnnounceMaturity,
        }])
        .collect::<Vec<_>>();
    assert_eq!(source_facts.as_slice(), expected_sources);
    assert_side_fields(&result_facts, TransactionSide::Output);
    assert_eq!(
        steps,
        &[
            Step::LocateAcceptedTransaction,
            Step::VerifyBytesAndDeploymentBinding,
            Step::DecodePublicWitness,
            Step::DeriveSuccessorMetadata,
            Step::ReconstructSuccessorConstructor,
            Step::CompareActualStateOutput,
        ]
    );
}

#[test]
fn succession_names_declared_policies_and_only_state_endpoints() {
    let StateSuccessionRequirement {
        root,
        input_fields,
        output_fields,
        root_relation,
        certificate_relation,
    } = StateSuccessionRequirement::REQUIRED;
    assert_eq!(root, RootId::State);
    assert_side_fields(&input_fields.unwrap(), TransactionSide::Input);
    assert_side_fields(&output_fields.unwrap(), TransactionSide::Output);
    let input = super::proof_tests::announcement_input();
    let declaration = input
        .realization()
        .operation(OperationId::AnnounceMaturity)
        .unwrap();
    let roots = declaration
        .relations
        .iter()
        .find(|row| row.id == root_relation)
        .unwrap();
    let certificate = declaration
        .relations
        .iter()
        .find(|row| row.id == certificate_relation)
        .unwrap();
    let Relation::RootPolicy { expected } = &roots.relation else {
        panic!("root policy")
    };
    assert_eq!(expected.len(), RootId::ALL.len());
    for id in RootId::ALL {
        assert_eq!(
            expected[id],
            if *id == root {
                RootUse::Succession
            } else {
                RootUse::Forbidden
            }
        );
    }
    let Relation::ProjectionPolicy { expected } = &certificate.relation else {
        panic!("projection policy")
    };
    assert_eq!(expected.len(), ProjectionId::ALL.len());
    for id in ProjectionId::ALL {
        assert_eq!(
            expected[id],
            if *id == ProjectionId::TransitionCertificate {
                ProjectionRule::Required
            } else {
                ProjectionRule::Forbidden
            }
        );
    }
}

#[test]
fn constructor_endpoints_and_continuity_are_separate_payload_free_roles() {
    let ConstructorContinuityRequirement {
        predecessor,
        successor,
        static_continuity,
        policy_continuity,
    } = ConstructorContinuityRequirement::REQUIRED;
    assert_eq!(
        predecessor,
        AnnouncementConstructorRole::AuthenticatePredecessor
    );
    assert_eq!(successor, AnnouncementConstructorRole::ReconstructSuccessor);
    assert_eq!(AnnouncementConstructorRole::ALL, &[predecessor, successor]);
    assert_eq!(
        static_continuity,
        AnnouncementStaticContinuity::SameLinkedStaticSubtree
    );
    assert_eq!(
        policy_continuity,
        AnnouncementPolicyContinuity::SameLeafVersionAndInternalKeyPolicy
    );
    assert_eq!(AnnouncementStaticContinuity::ALL, &[static_continuity]);
    assert_eq!(AnnouncementPolicyContinuity::ALL, &[policy_continuity]);
}

#[test]
fn history_checks_include_invalid_intermediates_and_non_claims() {
    use AnnouncementRootHistoryCheck as Check;
    let RootHistoryRequirement { succession, checks } = RootHistoryRequirement::REQUIRED;
    assert_eq!(succession, StateSuccessionRequirement::REQUIRED);
    assert_eq!(
        checks,
        &[
            Check::ExactlyOneStateEdge,
            Check::EndpointAndCertificateAgreement,
            Check::CurrentPredecessor,
            Check::EveryIntermediateEdgeValid,
            Check::RejectStaleHistory,
            Check::RejectTerminatedHistory,
            Check::RejectForeignRootHistory,
            Check::CheckpointAndReorgBinding,
            Check::NoSyntheticOriginCanonicalityClaim,
        ]
    );
    assert_eq!(checks.iter().collect::<BTreeSet<_>>().len(), 9);
}

#[test]
fn boundary_has_five_disjoint_complete_duty_groups() {
    use AnnouncementRequirementBoundary as Boundary;
    assert_eq!(
        Boundary::ALL,
        &[
            Boundary::Runtime,
            Boundary::LinkedConstructor,
            Boundary::TargetEvidence,
            Boundary::RootHistoryReport,
            Boundary::PublicRecoveryReport,
        ]
    );
    let mut union = BTreeSet::new();
    for boundary in Boundary::ALL {
        let expected = expected_duties(*boundary);
        let actual = AnnouncementDuty::ALL
            .iter()
            .copied()
            .filter(|duty| duty.boundary() == *boundary)
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
        for duty in expected {
            assert!(union.insert(*duty));
        }
    }
    assert_eq!(union, AnnouncementDuty::ALL.iter().copied().collect());
    assert_eq!(union.len(), 18);
}

fn expected_duties(boundary: AnnouncementRequirementBoundary) -> &'static [AnnouncementDuty] {
    use AnnouncementDuty as Duty;
    match boundary {
        AnnouncementRequirementBoundary::Runtime => &[
            Duty::PredecessorMetadataAuthentication,
            Duty::PredecessorMaturity,
            Duty::AnnouncementLeadWindow,
            Duty::SuccessorMetadataDerivation,
            Duty::SuccessorConstructorReconstruction,
            Duty::OperatorAuthorization,
            Duty::TransactionStructure,
            Duty::RootAndProjectionClosure,
            Duty::SponsorIsolation,
        ],
        AnnouncementRequirementBoundary::LinkedConstructor => &[
            Duty::MetadataLeafCommitment,
            Duty::StaticSubtreeContinuity,
            Duty::ConstructorTargetPolicy,
        ],
        AnnouncementRequirementBoundary::TargetEvidence => &[
            Duty::SelectedSignatureSemantics,
            Duty::WholeTransactionConservation,
            Duty::TaprootCommitmentAndControlPath,
        ],
        AnnouncementRequirementBoundary::RootHistoryReport => {
            &[Duty::CurrentRootFreshness, Duty::RootHistoryEdgeSequence]
        }
        AnnouncementRequirementBoundary::PublicRecoveryReport => &[Duty::PublicReconstruction],
    }
}

#[test]
fn field_law_and_precondition_censuses_are_closed() {
    use StateFieldLawKind as Kind;
    assert_eq!(
        Kind::ALL,
        &[
            Kind::Copy,
            Kind::CheckedAmountAdd,
            Kind::CheckedAmountSubtract,
            Kind::ZeroAmount,
            Kind::NextCycle,
            Kind::AnnounceRequestedCycle,
            Kind::CycleLiveSupply,
            Kind::CycleTimeLockedSupply,
            Kind::CycleMaturity,
            Kind::RedemptionBacking,
            Kind::ClearLiveSupply
        ]
    );
    assert_eq!(Kind::ALL.iter().copied().collect::<BTreeSet<_>>().len(), 11);
    assert_eq!(
        Kind::ALL
            .iter()
            .map(|kind| kind.operand_count())
            .collect::<Vec<_>>(),
        [0, 1, 1, 0, 0, 1, 6, 6, 1, 3, 2]
    );
    assert_eq!(
        AnnouncementPredecessorMaturity::ALL,
        &[AnnouncementPredecessorMaturity::Unannounced]
    );
}

#[test]
fn requirement_values_are_deterministic_and_equal() {
    assert_repeated(AnnouncementMetadataRequirement::required);
    assert_repeated(AnnouncementMetadataRequirement::public_facts);
    assert_repeated(|| StateSuccessionRequirement::REQUIRED);
    assert_repeated(|| ConstructorContinuityRequirement::REQUIRED);
    assert_repeated(|| RootHistoryRequirement::REQUIRED);
    assert_repeated(|| PublicRecoveryRequirement::REQUIRED);
    for field in StateField::ALL {
        assert_repeated(|| StateFieldRequirement::for_field(*field));
    }
    for duty in AnnouncementDuty::ALL {
        assert_repeated(|| duty.boundary());
    }
}

fn assert_repeated<T: Clone + std::fmt::Debug + Eq>(make: impl Fn() -> T) {
    let first = make();
    assert_eq!(first, make());
    assert_eq!(first, first.clone());
}

#[test]
fn field_map_agrees_with_the_owned_transition_at_both_window_endpoints() {
    use realization::{AnnouncementLeadBounds, Cycle, Maturity, ProtocolAmount, StateMetadata};
    let predecessor = StateMetadata {
        omega: ProtocolAmount::new(11).unwrap(),
        y_l: ProtocolAmount::new(7).unwrap(),
        y_t: ProtocolAmount::new(5).unwrap(),
        q: ProtocolAmount::new(3).unwrap(),
        cycle: Cycle::new(13),
        maturity: Maturity::Unannounced,
    };
    let bounds = AnnouncementLeadBounds::new(Cycle::new(2), Cycle::new(4)).unwrap();
    for requested in [Cycle::new(15), Cycle::new(17)] {
        let successor = realization::announce_maturity(&predecessor, requested, bounds).unwrap();
        let StateMetadata {
            omega,
            y_l,
            y_t,
            q,
            cycle,
            maturity,
        } = successor;
        assert_eq!(
            (omega, y_l, y_t, q, cycle),
            (
                predecessor.omega,
                predecessor.y_l,
                predecessor.y_t,
                predecessor.q,
                predecessor.cycle
            )
        );
        assert_eq!(maturity, Maturity::Announced { cycle: requested });
    }
}

#[test]
fn field_laws_reject_swapped_sides_and_mismatched_keys() {
    let base = StateFieldRequirement::for_field(StateField::Omega);
    let mut swapped = base.clone();
    std::mem::swap(&mut swapped.input, &mut swapped.output);
    assert_eq!(
        swapped.validate(),
        Err(StateFieldRequirementError::InvalidInput)
    );
    let mut wrong_output_side = base.clone();
    wrong_output_side.output = base.input.clone();
    assert_eq!(
        wrong_output_side.validate(),
        Err(StateFieldRequirementError::InvalidOutput)
    );
    let mut wrong_input_field = base.clone();
    wrong_input_field.input = field_key(TransactionSide::Input, StateField::Q);
    assert_eq!(
        wrong_input_field.validate(),
        Err(StateFieldRequirementError::InvalidInput)
    );
    let mut wrong_output_field = base.clone();
    wrong_output_field.output = field_key(TransactionSide::Output, StateField::Q);
    assert_eq!(
        wrong_output_field.validate(),
        Err(StateFieldRequirementError::InvalidOutput)
    );
    let mut wrong_operation = base;
    wrong_operation.output = FactId::StateField {
        operation: OperationId::Cycle,
        side: TransactionSide::Output,
        field: StateField::Omega,
    };
    assert_eq!(
        wrong_operation.validate(),
        Err(StateFieldRequirementError::InvalidOutput)
    );
}

#[test]
fn every_law_checks_its_signature_arity() {
    for kind in StateFieldLawKind::ALL {
        let mut row = StateFieldRequirement::for_field(StateField::Omega);
        row.law.kind = *kind;
        row.law.operands = vec![StateLawOperand::Fact(row.input.clone()); kind.operand_count()];
        row.validate().unwrap();
        row.law.operands.push(StateLawOperand::PublishedParameter(
            realization::StateLawParameter::Zeta,
        ));
        assert_eq!(
            row.validate(),
            Err(StateFieldRequirementError::OperandCount)
        );
        row.law.operands.pop();
        if row.law.operands.pop().is_some() {
            assert_eq!(
                row.validate(),
                Err(StateFieldRequirementError::OperandCount)
            );
        }
    }
}

#[test]
fn reusable_side_sets_allow_absence_and_keys_allow_every_state_exit() {
    let mut succession = StateSuccessionRequirement::REQUIRED;
    succession.input_fields = None;
    assert_side_fields(
        &succession.output_fields.clone().unwrap(),
        TransactionSide::Output,
    );
    succession.output_fields = None;
    assert_eq!(
        (succession.input_fields, succession.output_fields),
        (None, None)
    );
    for operation in [
        OperationId::AdmitDeposits,
        OperationId::Cycle,
        OperationId::Redeem,
        OperationId::ReceiptRelabel,
        OperationId::Clear,
        OperationId::AnnounceMaturity,
    ] {
        for field in StateField::ALL {
            let row = StateFieldRequirement {
                field: *field,
                input: FactId::StateField {
                    operation,
                    side: TransactionSide::Input,
                    field: *field,
                },
                output: FactId::StateField {
                    operation,
                    side: TransactionSide::Output,
                    field: *field,
                },
                law: StateFieldLaw {
                    kind: StateFieldLawKind::Copy,
                    operands: Vec::new(),
                },
            };
            row.validate().unwrap();
        }
    }
}

#[test]
fn cycle_laws_can_name_other_fields_and_the_published_split_parameter() {
    let mut row = StateFieldRequirement::for_field(StateField::YL);
    row.input = FactId::StateField {
        operation: OperationId::Cycle,
        side: TransactionSide::Input,
        field: StateField::YL,
    };
    row.output = FactId::StateField {
        operation: OperationId::Cycle,
        side: TransactionSide::Output,
        field: StateField::YL,
    };
    let operands = [
        StateField::Omega,
        StateField::YT,
        StateField::Q,
        StateField::Cycle,
        StateField::Maturity,
    ]
    .map(|field| {
        StateLawOperand::Fact(FactId::StateField {
            operation: OperationId::Cycle,
            side: TransactionSide::Input,
            field,
        })
    })
    .into_iter()
    .chain([StateLawOperand::PublishedParameter(
        realization::StateLawParameter::Zeta,
    )])
    .collect::<Vec<_>>();
    row.law = StateFieldLaw {
        kind: StateFieldLawKind::CycleLiveSupply,
        operands: operands.clone(),
    };
    row.validate().unwrap();
    assert_eq!(row.law.operands, operands);
    assert!(
        AnnouncementMetadataRequirement::required()
            .fields
            .iter()
            .all(|field| field
                .law
                .operands
                .iter()
                .all(|operand| !matches!(operand, StateLawOperand::PublishedParameter(_))))
    );
}
