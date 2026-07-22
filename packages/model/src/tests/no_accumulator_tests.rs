//! No-accumulator and architecture-conformance tests.
//!
//! Implements `´test:verification:no-accumulator´` against the typed
//! manifest in `tripod-architecture`.

use std::collections::BTreeSet;

use architecture::ARCHITECTURE;

use super::distribution_fixtures;
use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;

#[test]
fn implementation_conforms_to_the_typed_architecture() {
    validate_architecture_conformance().unwrap();
}

#[test]
fn no_attestation_accumulator_root_is_declared() {
    // An attestation-root role is not representable in the typed
    // manifest; the declared root set is exactly the expected roots.
    assert_eq!(ARCHITECTURE.roots.len(), architecture::RootId::ALL.len());

    for expected in architecture::RootId::ALL {
        assert!(ARCHITECTURE.root(*expected).is_some());
    }

    assert!(ARCHITECTURE.decisions.iter().any(|decision| {
        decision.id == architecture::DecisionId::NoOnchainAttestationAccumulator
            && decision.status == architecture::DecisionStatus::Closed
    }));
}

#[test]
fn historical_residue_is_audit_only_in_the_manifest() {
    for quantity in [
        architecture::QuantityId::HistoricalLiveResidue,
        architecture::QuantityId::HistoricalTimeLockedResidue,
    ] {
        let spec = ARCHITECTURE.quantity(quantity).unwrap();

        assert!(spec.allows(architecture::ReaderId::InvariantChecker));
        assert!(spec.allows(architecture::ReaderId::ExternalAuditor));
        assert!(!spec.allows(architecture::ReaderId::AttestationIndexer));
        assert!(!spec.allows(architecture::ReaderId::ConsumerFormula));

        for operation in architecture::OperationId::ALL {
            assert!(!spec.allows(architecture::ReaderId::Operation(*operation)));
        }
    }
}

#[test]
fn every_declared_bound_is_calibrated() {
    let world = test_fixtures::world();

    validate_bound_conformance(&world.constants).unwrap();
}

#[test]
fn genesis_world_has_no_attestation_singleton() {
    let world = test_fixtures::world();

    assert_no_attestation_singleton(&world).unwrap();
}

#[test]
fn branch_and_operation_maps_are_inverse() {
    for operation in architecture::OperationId::ALL {
        assert_eq!(branch_operation(operation_branch(*operation)), *operation);
    }
}

#[test]
fn declared_assets_round_trip() {
    for asset in architecture::AssetId::ALL {
        assert_eq!(declared_asset(asset_of(*asset)), Some(*asset));
    }

    assert_eq!(declared_asset(Asset::Foreign(7)), None);
}

#[test]
fn full_operation_manifest_conformance() {
    architecture::validate_draft(&ARCHITECTURE).unwrap();
    validate_architecture_conformance().unwrap();

    let world = test_fixtures::world();

    for operation in ARCHITECTURE.operations {
        // Bijective branch/operation identifier mapping.
        assert_eq!(
            branch_operation(operation_branch(operation.id)),
            operation.id,
        );

        // Every declared finite bound resolves to a nonzero runtime
        // value.
        for bound in operation.bounds {
            assert!(bound_value(&world.constants, *bound) > 0);
        }

        // Input authorization legality against the object schemas:
        // root inputs are covenant companions; non-root inputs use
        // owner, refund, sponsor, or permissionless modes.
        for input in operation.inputs {
            match input.authorization {
                architecture::InputAuthorization::SponsorOwner => {
                    assert_eq!(input.object, architecture::ObjectId::PlainLbtc);
                }

                architecture::InputAuthorization::RefundKey => {
                    assert_eq!(input.object, architecture::ObjectId::DepositRequest);
                }

                architecture::InputAuthorization::CovenantCompanion => {
                    assert!(matches!(
                        input.object,
                        architecture::ObjectId::State
                            | architecture::ObjectId::Resv
                            | architecture::ObjectId::Pace
                            | architecture::ObjectId::EntitlementAuthority
                            | architecture::ObjectId::DistributionAuthority,
                    ));
                }

                architecture::InputAuthorization::InputOwner
                | architecture::InputAuthorization::Permissionless => {}
            }
        }
    }
}

#[test]
#[allow(clippy::cognitive_complexity)]
fn manifest_derived_shape_policy_matches_all_branches() {
    let world = test_fixtures::world();

    let resolve = |maximum: architecture::MaxCount| match maximum {
        architecture::MaxCount::Exact(value) => usize::from(value),
        architecture::MaxCount::Bound(bound) => bound_value(&world.constants, bound),
    };

    for operation in architecture::OperationId::ALL {
        let spec = ARCHITECTURE.operation(*operation).unwrap();

        let policy = crate::shape::shape_policy(&world.constants, operation_branch(*operation));

        assert_eq!(policy.allowed_inputs.len(), spec.inputs.len());
        assert_eq!(policy.allowed_outputs.len(), spec.outputs.len());

        assert_eq!(policy.min_inputs.len(), spec.inputs.len());
        assert_eq!(policy.max_inputs.len(), spec.inputs.len());
        assert_eq!(policy.min_outputs.len(), spec.outputs.len());
        assert_eq!(policy.max_outputs.len(), spec.outputs.len());

        assert_eq!(
            policy.allowed_inputs,
            policy.max_inputs.keys().copied().collect::<BTreeSet<_>>(),
        );

        assert_eq!(
            policy.allowed_outputs,
            policy.max_outputs.keys().copied().collect::<BTreeSet<_>>(),
        );

        // Every declared minimum and maximum is carried exactly.
        for input in spec.inputs {
            let kind = crate::manifest::object_kind_of(input.object);

            assert_eq!(
                policy.min_inputs.get(&kind),
                Some(&usize::from(input.minimum)),
                "operation {} input {}",
                operation,
                input.object,
            );

            assert_eq!(
                policy.max_inputs.get(&kind),
                Some(&resolve(input.maximum)),
                "operation {} input {}",
                operation,
                input.object,
            );
        }

        for output in spec.outputs {
            let kind = crate::manifest::object_kind_of(output.object);

            assert_eq!(
                policy.min_outputs.get(&kind),
                Some(&usize::from(output.minimum)),
                "operation {} output {}",
                operation,
                output.object,
            );

            assert_eq!(
                policy.max_outputs.get(&kind),
                Some(&resolve(output.maximum)),
                "operation {} output {}",
                operation,
                output.object,
            );
        }
    }

    // Representative pins against the frozen v13 shape expectations.
    let cycle = crate::shape::shape_policy(&world.constants, BranchKind::Cycle);

    assert!(cycle.allowed_outputs.contains(&ObjectKind::CpfpAnchor));
    assert_eq!(cycle.max_outputs.get(&ObjectKind::ReceiptLive), Some(&1));
    assert_eq!(cycle.min_inputs.get(&ObjectKind::State), Some(&1));
    assert_eq!(cycle.min_outputs.get(&ObjectKind::Pace), Some(&1));
    assert_eq!(
        cycle.max_inputs.get(&ObjectKind::PlainLbtc),
        Some(&world.constants.fee_sponsor_input_max),
    );

    let settle = crate::shape::shape_policy(&world.constants, BranchKind::SettleDistribution);

    assert_eq!(
        settle.max_inputs.get(&ObjectKind::DepositEntitlement),
        Some(&world.constants.settlement_batch_max),
    );
    assert_eq!(
        settle.min_inputs.get(&ObjectKind::DepositEntitlement),
        Some(&1),
    );
    assert!(!settle.allowed_inputs.contains(&ObjectKind::State));

    let burn = crate::shape::shape_policy(&world.constants, BranchKind::Burn);

    assert_eq!(burn.min_outputs.get(&ObjectKind::Ash), Some(&1));
    assert_eq!(burn.max_outputs.get(&ObjectKind::Ash), Some(&1));
    assert_eq!(burn.min_inputs.get(&ObjectKind::ReceiptLive), Some(&1));
    assert_eq!(
        burn.max_inputs.get(&ObjectKind::ReceiptLive),
        Some(&world.constants.burn_input_max),
    );

    let compact = crate::shape::shape_policy(&world.constants, BranchKind::CompactAsh);

    assert_eq!(compact.min_inputs.get(&ObjectKind::Ash), Some(&2));
}

#[test]
fn manifest_derived_root_policy_matches_all_branches() {
    let redeem = branch_policy(BranchKind::Redeem);

    assert_eq!(redeem.state, RootUse::Succession);
    assert_eq!(redeem.resv, RootUse::SuccessionOrTermination);
    assert_eq!(redeem.pace, RootUse::Forbidden);

    let cycle = branch_policy(BranchKind::Cycle);

    assert_eq!(cycle.state, RootUse::Succession);
    assert_eq!(cycle.resv, RootUse::Succession);
    assert_eq!(cycle.pace, RootUse::Succession);
    assert_eq!(cycle.distribution_authority, RootUse::Succession);
    assert_eq!(cycle.entitlement_authority, RootUse::Forbidden);

    let admit = branch_policy(BranchKind::AdmitDeposits);

    assert_eq!(admit.entitlement_authority, RootUse::Succession);
    assert_eq!(admit.distribution_authority, RootUse::Forbidden);

    for branch in [
        BranchKind::CreateRequest,
        BranchKind::CancelRequest,
        BranchKind::SettleDistribution,
        BranchKind::TransferLive,
        BranchKind::TransferTimeLocked,
        BranchKind::Burn,
        BranchKind::CompactAsh,
    ] {
        let policy = branch_policy(branch);

        assert_eq!(policy.state, RootUse::Forbidden);
        assert_eq!(policy.resv, RootUse::Forbidden);
        assert_eq!(policy.pace, RootUse::Forbidden);
    }

    let burn = branch_policy(BranchKind::Burn);
    assert!(burn.allows_burn_projection);
    assert!(!burn.allows_clear_projection);

    let clear = branch_policy(BranchKind::Clear);
    assert!(clear.allows_clear_projection);
    assert!(!clear.allows_burn_projection);

    let settle = branch_policy(BranchKind::SettleDistribution);
    assert!(settle.allows_distribution_residue_projection);
}

#[test]
fn manifest_derived_open_flow_policy_matches_all_branches() {
    use crate::kernel::branch_allows_open_flow;

    // Representative pins.
    assert!(branch_allows_open_flow(
        BranchKind::CreateRequest,
        OpenFlowKind::RequestCreation,
    ));

    assert!(!branch_allows_open_flow(
        BranchKind::CreateRequest,
        OpenFlowKind::FeeSponsor,
    ));

    assert!(branch_allows_open_flow(
        BranchKind::Redeem,
        OpenFlowKind::Redemption,
    ));

    assert!(!branch_allows_open_flow(
        BranchKind::Redeem,
        OpenFlowKind::RequestRefund,
    ));

    assert!(branch_allows_open_flow(
        BranchKind::AdmitDeposits,
        OpenFlowKind::DepositAdmission,
    ));

    assert!(!branch_allows_open_flow(
        BranchKind::AdmitDeposits,
        OpenFlowKind::FeeSponsor,
    ));

    assert!(branch_allows_open_flow(
        BranchKind::Cycle,
        OpenFlowKind::ReserveCarry,
    ));

    // The derived allow-list is exactly the manifest declaration.
    for operation in architecture::OperationId::ALL {
        let spec = ARCHITECTURE.operation(*operation).unwrap();

        let allowed = architecture::OpenFlowKind::ALL
            .iter()
            .filter(|kind| spec.open_flows.contains(kind))
            .count();

        assert_eq!(allowed, spec.open_flows.len());
    }
}

#[test]
fn manifest_derived_value_flow_policy_matches_all_branches() {
    for operation in architecture::OperationId::ALL {
        let spec = ARCHITECTURE.operation(*operation).unwrap();

        assert_eq!(
            expected_value_flow_classes(operation_branch(*operation)).len(),
            spec.value_flows.len(),
        );
    }

    // Representative pins.
    assert!(
        expected_value_flow_classes(BranchKind::Redeem)
            .contains(&ValueFlowClass::FormulaBoundPayout)
    );

    assert!(
        expected_value_flow_classes(BranchKind::AdmitDeposits)
            .contains(&ValueFlowClass::PreauthorizedServiceBudget)
    );

    assert!(
        !expected_value_flow_classes(BranchKind::CreateRequest)
            .contains(&ValueFlowClass::SponsorEnvelope)
    );

    assert!(
        expected_value_flow_classes(BranchKind::Clear)
            .contains(&ValueFlowClass::OwnerlessTerminalSink)
    );
}

#[test]
fn manifest_delta_conditions_match_real_certificates() {
    // Every accepted transition already passes the postcommit
    // manifest-delta conformance check; this fixture additionally pins
    // the expected delta signatures of representative certificates.
    let (cycled, cycle) = distribution_fixtures::world_with_distribution(&[(ALICE, BOB, 1_000)]);

    let cycle_certificate = cycled
        .history
        .transitions
        .iter()
        .rev()
        .find(|certificate| certificate.branch == BranchKind::Cycle)
        .unwrap();

    assert!(
        cycle_certificate
            .canonical_partition
            .canonical_deltas()
            .iter()
            .any(|delta| { delta.asset == Asset::U && delta.kind == DeltaKind::Issuance })
    );

    assert!(
        cycle_certificate
            .canonical_partition
            .canonical_deltas()
            .iter()
            .any(|delta| { delta.asset == Asset::DistCtl && delta.kind == DeltaKind::Issuance })
    );

    let entitlements = distribution_fixtures::find_all_entitlements_for_cycle(&cycled, cycle);

    let settled = distribution_fixtures::settle_batch(&cycled, cycle, entitlements);

    let settle_certificate = settled.history.transitions.last().unwrap();

    assert_eq!(settle_certificate.branch, BranchKind::SettleDistribution);

    assert!(
        settle_certificate
            .canonical_partition
            .canonical_deltas()
            .iter()
            .any(|delta| {
                delta.asset == Asset::Ent
                    && delta.kind == DeltaKind::Destruction
                    && delta.destruction_tag == Some(Tag::Entitlement)
            })
    );

    // Single-batch settlement is terminal: the control closes.
    assert!(
        settle_certificate
            .canonical_partition
            .canonical_deltas()
            .iter()
            .any(|delta| {
                delta.asset == Asset::DistCtl
                    && delta.kind == DeltaKind::Destruction
                    && delta.destruction_tag == Some(Tag::DistributionControlClose)
            })
    );

    assert!(settle_certificate.distribution_residue.is_some());

    check_invariant(&settled).unwrap();
}

#[test]
fn manifest_data_outputs_match_real_transitions() {
    // The precommit data-output family gate is derived from the
    // manifest, so any accepted transition's data outputs come from
    // declared families. Pin the declared families per operation
    // against the destruction deltas they authenticate.
    for operation in ARCHITECTURE.operations {
        for delta in operation.canonical_deltas {
            if delta.kind != architecture::DeltaKind::Destruction {
                continue;
            }

            let tag = delta.destruction_tag.unwrap();

            let matching = operation
                .data_outputs
                .iter()
                .filter(|output| {
                    output.kind == architecture::DataOutputKind::Destruction
                        && output.tag == tag
                        && output.asset == Some(delta.asset)
                })
                .count();

            assert_eq!(matching, 1, "operation {}", operation.id);
        }

        for output in operation.data_outputs {
            match output.kind {
                architecture::DataOutputKind::BurnRecord => {
                    assert_eq!(operation.id, architecture::OperationId::Burn);
                }

                architecture::DataOutputKind::Destruction => {
                    assert!(operation.canonical_deltas.iter().any(|delta| {
                        delta.kind == architecture::DeltaKind::Destruction
                            && delta.destruction_tag == Some(output.tag)
                    }));
                }
            }
        }
    }
}

#[test]
fn runtime_constants_match_a_calibrated_deployment_profile() {
    // Constants built from the profile's calibrated values conform;
    // any divergence between published calibration evidence and the
    // running limits is rejected.
    let world = test_fixtures::world();

    let calibrated_bounds = ARCHITECTURE
        .bounds
        .iter()
        .map(|bound| architecture::BoundCalibration {
            bound: bound.id,
            value: u64::try_from(bound_value(&world.constants, bound.id)).unwrap(),
            evidence_hash: [0xA1; 32],
            script_bundle_hash: [0xA2; 32],
            measured_weight: 150_000,
            measured_witness_bytes: 40_000,
            measured_opcode_cost: 20_000,
        })
        .collect::<Vec<_>>();

    let profile = architecture::DeploymentProfile {
        schema_version: architecture::DEPLOYMENT_PROFILE_SCHEMA_VERSION,
        status: architecture::PublicationStatus::Draft,
        architecture_semantic_hash: architecture::semantic_hash(&ARCHITECTURE).unwrap(),
        network_id: [0xC1; 32],
        genesis_id: [0xC2; 32],
        script_limits: architecture::ScriptLimits {
            max_weight: 400_000,
            max_witness_bytes: 100_000,
            max_opcode_cost: 50_000,
        },
        calibrated_bounds,
        dependency_evidence: Vec::new(),
        artifacts: architecture::ArtifactHashes {
            normative_rust: [0; 32],
            compiler_configuration: [0; 32],
            emitted_script_bundle: [0; 32],
            reference_indexer: [0; 32],
            architecture_json: [0; 32],
            architecture_toml: [0; 32],
            canonical_wire_vectors: [0; 32],
        },
        test_evidence: architecture::TestEvidence {
            unit_test_report_hash: [0; 32],
            property_test_report_hash: [0; 32],
            independent_event_projection_report_hash: [0; 32],
            independent_attestation_query_report_hash: [0; 32],
            independent_receipt_accounting_report_hash: [0; 32],
            script_integration_report_hash: [0; 32],
        },
    };

    validate_profile_bound_conformance(&world.constants, &profile).unwrap();

    // A drifted calibration is rejected.
    let mut drifted = profile.clone();

    for calibration in &mut drifted.calibrated_bounds {
        if calibration.bound == architecture::BoundId::BurnInputMax {
            calibration.value += 1;
        }
    }

    assert_eq!(
        validate_profile_bound_conformance(&world.constants, &drifted),
        Err(Guard::BadConstant),
    );

    // Fixed (non-calibrated) bounds conform against the manifest's
    // declared value: a matching runtime passes, a diverging runtime
    // fails, and a fixed bound with no declared value is a defect,
    // never a free runtime constant.
    let runtime_burn_input_max = u64::try_from(crate::manifest::bound_value(
        &world.constants,
        architecture::BoundId::BurnInputMax,
    ))
    .unwrap();

    let fixed_bound = |default_value| {
        [architecture::BoundSpec {
            id: architecture::BoundId::BurnInputMax,
            default_value,
            requires_deployment_calibration: false,
        }]
    };

    crate::manifest::validate_bounds_against_authority(
        &fixed_bound(Some(runtime_burn_input_max)),
        &world.constants,
        &profile,
    )
    .unwrap();

    assert_eq!(
        crate::manifest::validate_bounds_against_authority(
            &fixed_bound(Some(runtime_burn_input_max + 1)),
            &world.constants,
            &profile,
        ),
        Err(Guard::BadConstant),
    );

    assert_eq!(
        crate::manifest::validate_bounds_against_authority(
            &fixed_bound(None),
            &world.constants,
            &profile,
        ),
        Err(Guard::BadConstant),
    );

    // A draft profile with no verified dependencies is never a
    // deployment release, even though the abstract model is green.
    assert!(architecture::validate_deployment_release(&ARCHITECTURE, &profile).is_err());
}
