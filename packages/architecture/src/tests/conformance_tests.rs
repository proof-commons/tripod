//! Cross-declaration conformance tests: destruction lifecycle,
//! issuance/delta/data-output correspondence, input authorization,
//! object lifecycle paths, root cardinality, quantity read
//! bidirectionality, and finite-bound resolution.

use std::collections::BTreeSet;

use crate::*;

#[test]
fn asset_destruction_operations_match_delta_declarations() {
    validate_draft(&ARCHITECTURE).unwrap();

    let u = ARCHITECTURE.asset(AssetId::U).unwrap();

    assert_eq!(
        u.destruction_operations
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            OperationId::Clear,
            OperationId::Redeem,
            OperationId::SettleDistribution,
        ]),
    );
}

#[test]
fn asset_destruction_operations_are_derived_exactly() {
    for asset in ARCHITECTURE.assets {
        let derived = ARCHITECTURE
            .operations
            .iter()
            .filter(|operation| {
                operation
                    .canonical_deltas
                    .iter()
                    .any(|delta| delta.asset == asset.id && delta.kind == DeltaKind::Destruction)
            })
            .map(|operation| operation.id)
            .collect::<BTreeSet<_>>();

        let declared = asset
            .destruction_operations
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();

        assert_eq!(declared, derived, "asset {}", asset.id);
    }
}

#[test]
fn issuance_specs_match_issuance_deltas() {
    for operation in ARCHITECTURE.operations {
        for issuance in operation.issuances {
            let matching = operation
                .canonical_deltas
                .iter()
                .filter(|delta| {
                    delta.kind == DeltaKind::Issuance
                        && delta.asset == issuance.asset
                        && delta.condition.as_str() == issuance.condition.as_str()
                        && delta.destruction_tag.is_none()
                })
                .count();

            assert_eq!(matching, 1, "operation {}", operation.id);
        }

        let issuance_deltas = operation
            .canonical_deltas
            .iter()
            .filter(|delta| delta.kind == DeltaKind::Issuance)
            .count();

        assert_eq!(
            issuance_deltas,
            operation.issuances.len(),
            "operation {}",
            operation.id,
        );
    }
}

#[test]
fn destruction_deltas_match_data_outputs() {
    for operation in ARCHITECTURE.operations {
        for delta in operation.canonical_deltas {
            if delta.kind != DeltaKind::Destruction {
                continue;
            }

            let tag = delta.destruction_tag.expect("destruction tag");

            let matching = operation
                .data_outputs
                .iter()
                .filter(|output| {
                    output.kind == DataOutputKind::Destruction
                        && output.tag == tag
                        && output.asset == Some(delta.asset)
                        && output.condition == delta.condition
                })
                .count();

            assert_eq!(matching, 1, "operation {}", operation.id);
        }
    }
}

#[test]
fn every_input_has_legal_authorization() {
    let root_object = |root: RootId| match root {
        RootId::State => ObjectId::State,
        RootId::Resv => ObjectId::Resv,
        RootId::Pace => ObjectId::Pace,
        RootId::EntAuth => ObjectId::EntitlementAuthority,
        RootId::DistAuth => ObjectId::DistributionAuthority,
    };

    let root_objects = RootId::ALL
        .iter()
        .map(|root| root_object(*root))
        .collect::<BTreeSet<_>>();

    for operation in ARCHITECTURE.operations {
        for input in operation.inputs {
            if root_objects.contains(&input.object) {
                // Every root input is a covenant companion regardless
                // of the operation's primary permission class.
                assert_eq!(
                    input.authorization,
                    InputAuthorization::CovenantCompanion,
                    "operation {} object {}",
                    operation.id,
                    input.object,
                );

                continue;
            }

            match input.authorization {
                InputAuthorization::SponsorOwner => {
                    assert_eq!(input.object, ObjectId::PlainLbtc);
                }

                InputAuthorization::RefundKey => {
                    assert_eq!(input.object, ObjectId::DepositRequest);
                }

                InputAuthorization::InputOwner => {
                    assert!(owner_bearing(input.object));
                }

                InputAuthorization::CovenantCompanion => {
                    panic!(
                        "non-root object {} claims companion status in {}",
                        input.object, operation.id,
                    );
                }

                InputAuthorization::Permissionless => {}
            }
        }
    }
}

#[test]
fn object_authorization_paths_are_derived() {
    let export = ArchitectureExport::from_architecture(&ARCHITECTURE);

    let plain = export
        .objects
        .iter()
        .find(|object| object.id == "PLAIN_LBTC")
        .unwrap();

    assert!(plain.authorization_paths.iter().any(|path| {
        matches!(
            path,
            export::ObjectAuthorizationExport::ExternalSpend { authorization }
                if authorization == "input-owner"
        )
    }));

    assert!(plain.authorization_paths.iter().any(|path| {
        matches!(
            path,
            export::ObjectAuthorizationExport::Operation { operation, authorization }
                if operation == "cycle" && authorization == "sponsor-owner"
        )
    }));

    let state = export
        .objects
        .iter()
        .find(|object| object.id == "STATE")
        .unwrap();

    assert!(state.authorization_paths.iter().any(|path| {
        matches!(
            path,
            export::ObjectAuthorizationExport::Operation { operation, authorization }
                if operation == "clear" && authorization == "covenant-companion"
        )
    }));

    assert!(state.authorization_paths.iter().any(|path| {
        matches!(
            path,
            export::ObjectAuthorizationExport::Operation { operation, authorization }
                if operation == "cycle" && authorization == "covenant-companion"
        )
    }));
}

/// The entitlement's `owner` is a routing destination, not a consent
/// gate: the object is not owner-bearing, derives no external-spend
/// authorization path, and its sole consumption path is permissionless
/// settlement.
#[test]
fn deposit_entitlement_derives_no_external_spend_authorization() {
    assert!(!owner_bearing(ObjectId::DepositEntitlement));

    let export = ArchitectureExport::from_architecture(&ARCHITECTURE);

    let entitlement = export
        .objects
        .iter()
        .find(|object| object.id == "DEPOSIT_ENTITLEMENT")
        .unwrap();

    assert!(!entitlement.authorization_paths.iter().any(|path| {
        matches!(
            path,
            export::ObjectAuthorizationExport::ExternalSpend { .. }
        )
    }));

    assert_eq!(
        entitlement.authorization_paths,
        vec![export::ObjectAuthorizationExport::Operation {
            operation: "settle-distribution".to_owned(),
            authorization: "permissionless".to_owned(),
        }],
    );
}

#[test]
fn object_allocator_paths_are_complete() {
    for object in ARCHITECTURE.objects {
        for allocator in object.allocators {
            if let AllocatorId::Operation(operation) = allocator {
                let spec = ARCHITECTURE.operation(*operation).unwrap();

                assert!(
                    spec.outputs.iter().any(|output| output.object == object.id),
                    "object {} allocator {}",
                    object.id,
                    operation,
                );
            }
        }
    }
}

#[test]
fn object_mutator_paths_are_complete() {
    for object in ARCHITECTURE.objects {
        for mutator in object.mutators {
            let spec = ARCHITECTURE.operation(*mutator).unwrap();

            assert!(
                spec.inputs.iter().any(|input| input.object == object.id),
                "object {} mutator {}",
                object.id,
                mutator,
            );
        }
    }
}

#[test]
fn object_deallocator_paths_are_complete() {
    for object in ARCHITECTURE.objects {
        for deallocator in object.deallocators {
            if let DeallocatorId::Operation(operation) = deallocator {
                let spec = ARCHITECTURE.operation(*operation).unwrap();

                assert!(
                    spec.inputs.iter().any(|input| input.object == object.id),
                    "object {} deallocator {}",
                    object.id,
                    operation,
                );
            }
        }
    }
}

#[test]
fn root_use_matches_root_cardinality() {
    let root_object = |root: RootId| match root {
        RootId::State => ObjectId::State,
        RootId::Resv => ObjectId::Resv,
        RootId::Pace => ObjectId::Pace,
        RootId::EntAuth => ObjectId::EntitlementAuthority,
        RootId::DistAuth => ObjectId::DistributionAuthority,
    };

    for operation in ARCHITECTURE.operations {
        for root in RootId::ALL {
            let object = root_object(*root);

            let input = operation.inputs.iter().find(|input| input.object == object);
            let output = operation
                .outputs
                .iter()
                .find(|output| output.object == object);

            match operation.root_use(*root) {
                RootUse::Forbidden => {
                    assert!(input.is_none() && output.is_none());
                }

                RootUse::Succession => {
                    assert_eq!(input.unwrap().minimum, 1);
                    assert_eq!(output.unwrap().minimum, 1);
                }

                RootUse::SuccessionOrTermination => {
                    assert_eq!(input.unwrap().minimum, 1);
                    assert_eq!(output.unwrap().minimum, 0);
                }
            }
        }
    }
}

#[test]
fn quantity_readers_and_operation_reads_are_bidirectional() {
    for operation in ARCHITECTURE.operations {
        for quantity in operation.reads {
            let spec = ARCHITECTURE.quantity(*quantity).unwrap();
            assert!(spec.allows(ReaderId::Operation(operation.id)));
        }
    }

    for quantity in ARCHITECTURE.quantities {
        for reader in quantity.readers {
            if let ReaderId::Operation(operation) = reader {
                let spec = ARCHITECTURE.operation(*operation).unwrap();
                assert!(spec.reads.contains(&quantity.id));
            }
        }
    }
}

#[test]
fn all_bounds_resolve_and_cover_minima() {
    for operation in ARCHITECTURE.operations {
        let cardinalities = operation
            .inputs
            .iter()
            .map(|input| (input.minimum, input.maximum))
            .chain(
                operation
                    .outputs
                    .iter()
                    .map(|output| (output.minimum, output.maximum)),
            )
            .chain(
                operation
                    .data_outputs
                    .iter()
                    .map(|output| (output.minimum, output.maximum)),
            );

        for (minimum, maximum) in cardinalities {
            match maximum {
                MaxCount::Exact(value) => assert!(minimum <= value),

                MaxCount::Bound(bound) => {
                    let spec = ARCHITECTURE.bound(bound).unwrap();

                    if let Some(default) = spec.default_value {
                        assert!(u64::from(minimum) <= default);
                    }
                }
            }
        }
    }
}

#[test]
fn no_duplicate_input_or_output_object_declarations() {
    for operation in ARCHITECTURE.operations {
        let inputs = operation
            .inputs
            .iter()
            .map(|input| input.object)
            .collect::<BTreeSet<_>>();

        assert_eq!(inputs.len(), operation.inputs.len());

        let outputs = operation
            .outputs
            .iter()
            .map(|output| output.object)
            .collect::<BTreeSet<_>>();

        assert_eq!(outputs.len(), operation.outputs.len());
    }
}

#[test]
fn authorization_evidence_table_is_generated_for_every_mode() {
    let export = ArchitectureExport::from_architecture(&ARCHITECTURE);

    assert_eq!(
        export.input_authorization_evidence.len(),
        InputAuthorization::ALL.len(),
    );

    for authorization in InputAuthorization::ALL {
        let row = export
            .input_authorization_evidence
            .iter()
            .find(|row| row.input_authorization == authorization.as_str())
            .unwrap();

        let evidence = authorization.evidence();

        assert_eq!(row.model_evidence, evidence.model.as_str());
        assert_eq!(row.compiler_evidence, evidence.compiler.as_str());
        assert_eq!(row.deployment_evidence, evidence.deployment.as_str());
    }

    // A covenant companion carries no independent signature, so its
    // evidence must never claim signer, checksig, or sighash backing.
    let companion = InputAuthorization::CovenantCompanion.evidence();

    assert_eq!(companion.model, AuthorizationEvidenceKind::ModelBranchShape);
    assert_ne!(companion.model, AuthorizationEvidenceKind::ModelSignerSet);
    assert_eq!(
        companion.compiler,
        AuthorizationEvidenceKind::CompilerCovenantPredicate,
    );
    assert_ne!(
        companion.compiler,
        AuthorizationEvidenceKind::CompilerChecksig,
    );
    assert_eq!(
        companion.deployment,
        AuthorizationEvidenceKind::DeploymentScriptSemantics,
    );
    assert_ne!(
        companion.deployment,
        AuthorizationEvidenceKind::DeploymentSighash,
    );

    // Signer-bearing input modes remain signer-backed; permissionless
    // claims none.
    for authorization in [
        InputAuthorization::InputOwner,
        InputAuthorization::RefundKey,
        InputAuthorization::SponsorOwner,
    ] {
        let evidence = authorization.evidence();

        assert_eq!(evidence.model, AuthorizationEvidenceKind::ModelSignerSet);
        assert_eq!(
            evidence.compiler,
            AuthorizationEvidenceKind::CompilerChecksig,
        );
        assert_eq!(
            evidence.deployment,
            AuthorizationEvidenceKind::DeploymentSighash,
        );
    }

    assert_eq!(
        InputAuthorization::Permissionless.evidence().model,
        AuthorizationEvidenceKind::NonePermissionless,
    );
}

#[test]
fn operation_authorization_evidence_table_is_generated_for_every_class() {
    let export = ArchitectureExport::from_architecture(&ARCHITECTURE);

    assert_eq!(
        export.operation_authorization_evidence.len(),
        PermissionClass::ALL.len(),
    );

    for class in PermissionClass::ALL {
        let row = export
            .operation_authorization_evidence
            .iter()
            .find(|row| row.permission_class == class.as_str())
            .unwrap();

        let evidence = class.evidence();

        assert_eq!(row.model_evidence, evidence.model.as_str());
        assert_eq!(row.compiler_evidence, evidence.compiler.as_str());
        assert_eq!(row.deployment_evidence, evidence.deployment.as_str());
    }

    // Cadence authorization is a composite band policy, not an
    // ordinary CHECKSIG claim.
    let cadence = PermissionClass::CadenceBand.evidence();

    assert_eq!(cadence.model, AuthorizationEvidenceKind::ModelCadenceBand);
    assert_eq!(
        cadence.compiler,
        AuthorizationEvidenceKind::CompilerCadenceLeaves,
    );
    assert_eq!(
        cadence.deployment,
        AuthorizationEvidenceKind::DeploymentCsvSemantics,
    );

    // Signer-backed operation classes remain signer-backed.
    for class in [
        PermissionClass::ClientAuthorized,
        PermissionClass::RefundKey,
        PermissionClass::ReceiptOwners,
        PermissionClass::Operator,
    ] {
        let evidence = class.evidence();

        assert_eq!(evidence.model, AuthorizationEvidenceKind::ModelSignerSet);
        assert_eq!(
            evidence.compiler,
            AuthorizationEvidenceKind::CompilerChecksig,
        );
        assert_eq!(
            evidence.deployment,
            AuthorizationEvidenceKind::DeploymentSighash,
        );
    }

    assert_eq!(
        PermissionClass::Permissionless.evidence().model,
        AuthorizationEvidenceKind::NonePermissionless,
    );
}
