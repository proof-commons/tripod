//! Mutation fixtures: each deliberate manifest corruption must fail
//! draft validation.

use crate::*;

fn leak<T>(values: Vec<T>) -> &'static [T] {
    Box::leak(values.into_boxed_slice())
}

fn mutate_asset(id: AssetId, mutate: impl FnOnce(&mut AssetSpec)) -> Architecture {
    let mut assets = ARCHITECTURE.assets.to_vec();

    let asset = assets.iter_mut().find(|asset| asset.id == id).unwrap();

    mutate(asset);

    let mut architecture = ARCHITECTURE;
    architecture.assets = leak(assets);
    architecture
}

fn mutate_operation(id: OperationId, mutate: impl FnOnce(&mut OperationSpec)) -> Architecture {
    let mut operations = ARCHITECTURE.operations.to_vec();

    let operation = operations
        .iter_mut()
        .find(|operation| operation.id == id)
        .unwrap();

    mutate(operation);

    let mut architecture = ARCHITECTURE;
    architecture.operations = leak(operations);
    architecture
}

#[test]
fn mutation_removing_settlement_from_u_destruction_fails() {
    let mutated = mutate_asset(AssetId::U, |asset| {
        asset.destruction_operations = &[OperationId::Redeem, OperationId::Clear];
    });

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_removing_a_witness_fails() {
    let witnesses = ARCHITECTURE
        .witnesses
        .iter()
        .filter(|witness| witness.id != WitnessId::CanonicalDelta)
        .copied()
        .collect::<Vec<_>>();

    let mut mutated = ARCHITECTURE;
    mutated.witnesses = leak(witnesses);

    assert!(validate_draft(&mutated).is_err());
}

fn mutate_input_authorization(
    operation: OperationId,
    object: ObjectId,
    authorization: InputAuthorization,
) -> Architecture {
    mutate_operation(operation, |operation| {
        let mut inputs = operation.inputs.to_vec();

        for input in &mut inputs {
            if input.object == object {
                input.authorization = authorization;
            }
        }

        operation.inputs = leak(inputs);
    })
}

fn mutate_root(id: RootId, mutate: impl FnOnce(&mut RootSpec)) -> Architecture {
    let mut roots = ARCHITECTURE.roots.to_vec();

    let root = roots.iter_mut().find(|root| root.id == id).unwrap();

    mutate(root);

    let mut architecture = ARCHITECTURE;
    architecture.roots = leak(roots);
    architecture
}

fn mutate_object(id: ObjectId, mutate: impl FnOnce(&mut ObjectSpec)) -> Architecture {
    let mut objects = ARCHITECTURE.objects.to_vec();

    let object = objects.iter_mut().find(|object| object.id == id).unwrap();

    mutate(object);

    let mut architecture = ARCHITECTURE;
    architecture.objects = leak(objects);
    architecture
}

fn mutate_bound(id: BoundId, mutate: impl FnOnce(&mut BoundSpec)) -> Architecture {
    let mut bounds = ARCHITECTURE.bounds.to_vec();

    let bound = bounds.iter_mut().find(|bound| bound.id == id).unwrap();

    mutate(bound);

    let mut architecture = ARCHITECTURE;
    architecture.bounds = leak(bounds);
    architecture
}

#[test]
fn mutation_fixed_bound_without_a_value_fails() {
    let mutated = mutate_bound(BoundId::BurnInputMax, |bound| {
        bound.requires_deployment_calibration = false;
        bound.default_value = None;
    });

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_fixed_bound_with_zero_value_fails() {
    let mutated = mutate_bound(BoundId::BurnInputMax, |bound| {
        bound.requires_deployment_calibration = false;
        bound.default_value = Some(0);
    });

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn fixed_bound_with_positive_value_validates() {
    let mutated = mutate_bound(BoundId::BurnInputMax, |bound| {
        bound.requires_deployment_calibration = false;
    });

    assert!(validate_draft(&mutated).is_ok());
}

#[test]
fn mutation_changing_state_root_asset_away_from_its_object_fails() {
    // PACE is also a closed amount-one asset, so the generic root
    // checks alone would accept this contradiction; the root/object
    // closure check must reject it.
    let mutated = mutate_root(RootId::State, |root| {
        root.asset = AssetId::Pace;
    });

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_changing_a_root_role_fails() {
    let mutated = mutate_root(RootId::Pace, |root| {
        root.role = RootRole::StateIdentity;
    });

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_declaring_a_fixed_amount_on_the_reserve_root_fails() {
    let mutated = mutate_root(RootId::Resv, |root| {
        root.fixed_amount = Some(1);
    });

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_changing_a_root_object_asset_fails() {
    let mutated = mutate_object(ObjectId::State, |object| {
        object.asset = AssetId::Pace;
    });

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_changing_a_root_object_lifecycle_fails() {
    let mutated = mutate_object(ObjectId::State, |object| {
        object.lifecycle = LifecycleClass::OpenOffer;
    });

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_marking_a_rootless_object_constant_root_fails() {
    let mutated = mutate_object(ObjectId::Ash, |object| {
        object.lifecycle = LifecycleClass::ConstantRoot;
    });

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_making_a_root_input_signer_backed_fails() {
    let mutated = mutate_input_authorization(
        OperationId::Cycle,
        ObjectId::State,
        InputAuthorization::SponsorOwner,
    );

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_making_a_root_input_permissionless_fails() {
    let mutated = mutate_input_authorization(
        OperationId::AdmitDeposits,
        ObjectId::Resv,
        InputAuthorization::Permissionless,
    );

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_making_a_root_input_owner_authorized_fails() {
    let mutated = mutate_input_authorization(
        OperationId::Redeem,
        ObjectId::State,
        InputAuthorization::InputOwner,
    );

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_making_a_receipt_input_a_companion_fails() {
    let mutated = mutate_input_authorization(
        OperationId::Redeem,
        ObjectId::ReceiptLive,
        InputAuthorization::CovenantCompanion,
    );

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_making_a_non_root_entitlement_input_a_companion_fails() {
    let mutated = mutate_input_authorization(
        OperationId::SettleDistribution,
        ObjectId::DepositEntitlement,
        InputAuthorization::CovenantCompanion,
    );

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_adding_a_companion_input_for_a_forbidden_root_fails() {
    let mutated = mutate_operation(OperationId::Clear, |operation| {
        let mut inputs = operation.inputs.to_vec();

        inputs.push(InputSpec {
            object: ObjectId::Pace,
            minimum: 1,
            maximum: MaxCount::Exact(1),
            authorization: InputAuthorization::CovenantCompanion,
        });

        operation.inputs = leak(inputs);
    });

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_declaring_cadence_band_outside_cycle_fails() {
    let mutated = mutate_operation(OperationId::Burn, |operation| {
        operation.authorization = PermissionClass::CadenceBand;
    });

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_removing_owner_authorization_from_transfer_fails() {
    let mutated = mutate_input_authorization(
        OperationId::TransferLive,
        ObjectId::ReceiptLive,
        InputAuthorization::Permissionless,
    );

    assert!(validate_draft(&mutated).is_err());
}

fn mutate_input_minimum(operation: OperationId, object: ObjectId, minimum: u16) -> Architecture {
    mutate_operation(operation, |operation| {
        let mut inputs = operation.inputs.to_vec();

        for input in &mut inputs {
            if input.object == object {
                input.minimum = minimum;
            }
        }

        operation.inputs = leak(inputs);
    })
}

fn mutate_output_minimum(operation: OperationId, object: ObjectId, minimum: u16) -> Architecture {
    mutate_operation(operation, |operation| {
        let mut outputs = operation.outputs.to_vec();

        for output in &mut outputs {
            if output.object == object {
                output.minimum = minimum;
            }
        }

        operation.outputs = leak(outputs);
    })
}

#[test]
fn mutation_relaxing_cycle_state_input_minimum_fails() {
    let mutated = mutate_input_minimum(OperationId::Cycle, ObjectId::State, 0);

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_relaxing_burn_ash_output_minimum_fails() {
    let mutated = mutate_output_minimum(OperationId::Burn, ObjectId::Ash, 0);

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_relaxing_transfer_receipt_input_minimum_fails() {
    let mutated = mutate_input_minimum(OperationId::TransferLive, ObjectId::ReceiptLive, 0);

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_relaxing_settlement_entitlement_minimum_fails() {
    let mutated = mutate_input_minimum(
        OperationId::SettleDistribution,
        ObjectId::DepositEntitlement,
        0,
    );

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_relaxing_compaction_ash_minimum_fails() {
    let mutated = mutate_input_minimum(OperationId::CompactAsh, ObjectId::Ash, 1);

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn minimum_mutations_change_the_semantic_hash() {
    let mutated = mutate_output_minimum(OperationId::Burn, ObjectId::Ash, 0);

    assert_ne!(
        semantic_hash(&ARCHITECTURE).unwrap(),
        semantic_hash(&mutated).unwrap(),
    );
}

#[test]
fn mutation_changing_an_input_maximum_fails() {
    let mutated = mutate_operation(OperationId::CompactAsh, |operation| {
        let mut inputs = operation.inputs.to_vec();

        for input in &mut inputs {
            if input.object == ObjectId::Ash {
                // Minimum two ASH inputs with an exact maximum of
                // one is unsatisfiable.
                input.maximum = MaxCount::Exact(1);
            }
        }

        operation.inputs = leak(inputs);
    });

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_removing_a_destruction_tag_fails() {
    let mutated = mutate_operation(OperationId::Redeem, |operation| {
        let mut deltas = operation.canonical_deltas.to_vec();

        for delta in &mut deltas {
            if delta.kind == DeltaKind::Destruction {
                delta.destruction_tag = None;
            }
        }

        operation.canonical_deltas = leak(deltas);
    });

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_declaring_a_wrong_delta_asset_fails() {
    let mutated = mutate_operation(OperationId::Redeem, |operation| {
        let mut deltas = operation.canonical_deltas.to_vec();

        for delta in &mut deltas {
            delta.asset = AssetId::Lbtc;
        }

        operation.canonical_deltas = leak(deltas);
    });

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_omitting_an_authority_root_fails() {
    let mutated = mutate_operation(OperationId::Cycle, |operation| {
        let roots = operation
            .roots
            .iter()
            .filter(|root| root.root != RootId::Pace)
            .copied()
            .collect::<Vec<_>>();

        operation.roots = leak(roots);
    });

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_adding_an_unauthorized_residue_reader_fails() {
    let mut quantities = ARCHITECTURE.quantities.to_vec();

    for quantity in &mut quantities {
        if quantity.id == QuantityId::HistoricalLiveResidue {
            let mut readers = quantity.readers.to_vec();
            readers.push(ReaderId::ConsumerFormula);
            quantity.readers = leak(readers);
        }
    }

    let mut mutated = ARCHITECTURE;
    mutated.quantities = leak(quantities);

    assert!(validate_draft(&mutated).is_err());
}

// Projection-policy mutations: each deliberate drift from the pinned
// projection policy must fail draft validation.

fn mutate_projections(id: OperationId, projections: Vec<ProjectionSpec>) -> Architecture {
    mutate_operation(id, |operation| {
        operation.projections = leak(projections);
    })
}

#[test]
fn mutation_removing_transition_certificate_projection_fails() {
    let mutated = mutate_projections(OperationId::Cycle, Vec::new());

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_making_burn_projection_optional_on_burn_fails() {
    let mutated = mutate_projections(
        OperationId::Burn,
        vec![
            ProjectionSpec {
                projection: ProjectionId::TransitionCertificate,
                rule: ProjectionRule::Required,
            },
            ProjectionSpec {
                projection: ProjectionId::BurnEvent,
                rule: ProjectionRule::Optional,
            },
        ],
    );

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_forbidding_burn_projection_on_burn_fails() {
    let mutated = mutate_projections(
        OperationId::Burn,
        vec![
            ProjectionSpec {
                projection: ProjectionId::TransitionCertificate,
                rule: ProjectionRule::Required,
            },
            ProjectionSpec {
                projection: ProjectionId::BurnEvent,
                rule: ProjectionRule::Forbidden,
            },
        ],
    );

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_permitting_burn_projection_on_compaction_fails() {
    let mutated = mutate_projections(
        OperationId::CompactAsh,
        vec![
            ProjectionSpec {
                projection: ProjectionId::TransitionCertificate,
                rule: ProjectionRule::Required,
            },
            ProjectionSpec {
                projection: ProjectionId::BurnEvent,
                rule: ProjectionRule::Optional,
            },
        ],
    );

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_permitting_clear_projection_on_burn_fails() {
    let mutated = mutate_projections(
        OperationId::Burn,
        vec![
            ProjectionSpec {
                projection: ProjectionId::TransitionCertificate,
                rule: ProjectionRule::Required,
            },
            ProjectionSpec {
                projection: ProjectionId::BurnEvent,
                rule: ProjectionRule::Required,
            },
            ProjectionSpec {
                projection: ProjectionId::ClearEvent,
                rule: ProjectionRule::Optional,
            },
        ],
    );

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_permitting_residue_projection_outside_settlement_fails() {
    let mutated = mutate_projections(
        OperationId::Redeem,
        vec![
            ProjectionSpec {
                projection: ProjectionId::TransitionCertificate,
                rule: ProjectionRule::Required,
            },
            ProjectionSpec {
                projection: ProjectionId::DistributionResidue,
                rule: ProjectionRule::Optional,
            },
        ],
    );

    assert!(validate_draft(&mutated).is_err());
}

#[test]
fn mutation_duplicating_a_projection_declaration_fails() {
    let mutated = mutate_projections(
        OperationId::Cycle,
        vec![
            ProjectionSpec {
                projection: ProjectionId::TransitionCertificate,
                rule: ProjectionRule::Required,
            },
            ProjectionSpec {
                projection: ProjectionId::TransitionCertificate,
                rule: ProjectionRule::Required,
            },
        ],
    );

    assert!(validate_draft(&mutated).is_err());
}
