//! Draft/release validation and structural declaration tests.

use crate::*;

#[test]
fn draft_manifest_is_valid() {
    validate_draft(&ARCHITECTURE).unwrap();
}

#[test]
fn architecture_release_validation_accepts_the_pinned_final_manifest() {
    validate_architecture_release(&ARCHITECTURE).unwrap();
}

#[test]
fn architecture_release_validation_rejects_an_unpinned_draft() {
    let mut architecture = ARCHITECTURE;

    architecture.document.status = PublicationStatus::Draft;
    architecture.document.specification.anchor_set_hash = None;

    let errors = validate_architecture_release(&architecture).unwrap_err();

    assert!(errors.contains(&ManifestError::UnpinnedSpecification));
    assert!(errors.contains(&ManifestError::DraftPublication));
}

#[test]
fn no_attestation_accumulator_is_declared() {
    assert!(ARCHITECTURE.decisions.iter().any(|decision| {
        decision.id == DecisionId::NoOnchainAttestationAccumulator
            && decision.status == DecisionStatus::Closed
    }));

    // An attestation-root role is not representable; the declared
    // root set is exactly the five expected roots.
    assert_eq!(ARCHITECTURE.roots.len(), RootId::ALL.len());

    for expected in RootId::ALL {
        assert!(ARCHITECTURE.root(*expected).is_some());
    }
}

#[test]
fn every_object_is_declared() {
    for expected in ObjectId::ALL {
        assert!(ARCHITECTURE.object(*expected).is_some());
    }
}

#[test]
fn historical_residue_is_audit_only() {
    for quantity in [
        QuantityId::HistoricalLiveResidue,
        QuantityId::HistoricalTimeLockedResidue,
    ] {
        let spec = ARCHITECTURE.quantity(quantity).unwrap();

        assert!(spec.allows(ReaderId::InvariantChecker));
        assert!(spec.allows(ReaderId::ExternalAuditor));
        assert!(!spec.allows(ReaderId::AttestationIndexer));
        assert!(!spec.allows(ReaderId::ConsumerFormula));

        for operation in OperationId::ALL {
            assert!(!spec.allows(ReaderId::Operation(*operation)));
        }
    }
}

#[test]
fn all_enum_ids_are_declared() {
    for expected in AssetId::ALL {
        assert!(ARCHITECTURE.asset(*expected).is_some());
    }

    for expected in RootId::ALL {
        assert!(ARCHITECTURE.root(*expected).is_some());
    }

    for expected in ObjectId::ALL {
        assert!(ARCHITECTURE.object(*expected).is_some());
    }

    for expected in OperationId::ALL {
        assert!(ARCHITECTURE.operation(*expected).is_some());
    }

    for expected in QuantityId::ALL {
        assert!(ARCHITECTURE.quantity(*expected).is_some());
    }

    for expected in WitnessId::ALL {
        assert!(
            ARCHITECTURE
                .witnesses
                .iter()
                .any(|witness| witness.id == *expected)
        );
    }

    for expected in DependencyId::ALL {
        assert!(
            ARCHITECTURE
                .dependencies
                .iter()
                .any(|dependency| dependency.id == *expected)
        );
    }

    for expected in DecisionId::ALL {
        assert!(
            ARCHITECTURE
                .decisions
                .iter()
                .any(|decision| decision.id == *expected)
        );
    }

    for expected in BoundId::ALL {
        assert!(ARCHITECTURE.bound(*expected).is_some());
    }

    for expected in AmountLimitId::ALL {
        assert!(ARCHITECTURE.amount_limit(*expected).is_some());
    }

    for expected in TagId::ALL {
        assert!(ARCHITECTURE.tags.iter().any(|tag| tag.id == *expected));
    }
}

// Document-metadata validation (M-review finding 8): envelope fields
// are outside both hashes, so blanks and mismatches must be caught by
// the validator itself, per field.

#[test]
fn document_with_wrong_schema_version_is_rejected() {
    let mut architecture = ARCHITECTURE;
    architecture.document.architecture_schema_version = 0;

    let errors = validate_draft(&architecture).unwrap_err();
    assert!(errors.contains(&ManifestError::InvalidDocument {
        reason: "architecture schema version is not the supported schema",
    }));
}

#[test]
fn document_with_blank_realization_version_is_rejected() {
    let mut architecture = ARCHITECTURE;
    architecture.document.realization_version = "  ";

    let errors = validate_draft(&architecture).unwrap_err();
    assert!(errors.contains(&ManifestError::InvalidDocument {
        reason: "realization version is blank",
    }));
}

#[test]
fn document_with_malformed_realization_version_is_rejected() {
    let mut architecture = ARCHITECTURE;
    architecture.document.realization_version = "13z";

    let errors = validate_draft(&architecture).unwrap_err();
    assert!(errors.contains(&ManifestError::InvalidDocument {
        reason: "realization version is not a tracked major.minor.0 binding",
    }));
}

#[test]
fn document_with_blank_specification_version_is_rejected() {
    let mut architecture = ARCHITECTURE;
    architecture.document.specification.version = "";

    let errors = validate_draft(&architecture).unwrap_err();
    assert!(errors.contains(&ManifestError::InvalidDocument {
        reason: "specification version is blank",
    }));
}

#[test]
fn document_with_blank_target_network_is_rejected() {
    let mut architecture = ARCHITECTURE;
    architecture.document.target_network = " ";

    let errors = validate_draft(&architecture).unwrap_err();
    assert!(errors.contains(&ManifestError::InvalidDocument {
        reason: "target network is blank",
    }));
}

#[test]
fn all_zero_anchor_set_hash_is_not_a_release_pin() {
    let mut architecture = ARCHITECTURE;
    architecture.document.status = PublicationStatus::Final;
    architecture.document.specification.anchor_set_hash = Some([0_u8; 32]);

    let errors = validate_architecture_release(&architecture).unwrap_err();
    assert!(errors.contains(&ManifestError::UnpinnedSpecification));
}

// Duplicate set-like declarations (M-review finding 9): the export
// layer deduplicates these arrays, so a duplicated declaration would
// be hash-invisible; the validator must reject it instead.

#[test]
fn duplicate_operation_bound_declaration_is_rejected() {
    let mut architecture = ARCHITECTURE;

    let mut operations = architecture.operations.to_vec();
    let operation = operations
        .iter_mut()
        .find(|operation| !operation.bounds.is_empty())
        .expect("some operation declares a bound");
    let operation_id = operation.id;

    let mut bounds = operation.bounds.to_vec();
    bounds.push(bounds[0]);
    operation.bounds = bounds.leak();
    architecture.operations = operations.leak();

    let errors = validate_draft(&architecture).unwrap_err();
    assert!(errors.contains(&ManifestError::InvalidOperation {
        operation: operation_id,
        reason: "duplicate bound declaration",
    }));
}

#[test]
fn duplicate_operation_read_declaration_is_rejected() {
    let mut architecture = ARCHITECTURE;

    let mut operations = architecture.operations.to_vec();
    let operation = operations
        .iter_mut()
        .find(|operation| !operation.reads.is_empty())
        .expect("some operation declares a read");
    let operation_id = operation.id;

    let mut reads = operation.reads.to_vec();
    reads.push(reads[0]);
    operation.reads = reads.leak();
    architecture.operations = operations.leak();

    let errors = validate_draft(&architecture).unwrap_err();
    assert!(errors.contains(&ManifestError::InvalidOperation {
        operation: operation_id,
        reason: "duplicate read declaration",
    }));
}

#[test]
fn duplicate_canonical_delta_declaration_is_rejected() {
    let mut architecture = ARCHITECTURE;

    let mut operations = architecture.operations.to_vec();
    let operation = operations
        .iter_mut()
        .find(|operation| !operation.canonical_deltas.is_empty())
        .expect("some operation declares a canonical delta");
    let operation_id = operation.id;

    let mut deltas = operation.canonical_deltas.to_vec();
    deltas.push(deltas[0]);
    operation.canonical_deltas = deltas.leak();
    architecture.operations = operations.leak();

    let errors = validate_draft(&architecture).unwrap_err();
    assert!(errors.contains(&ManifestError::InvalidOperation {
        operation: operation_id,
        reason: "duplicate canonical-delta declaration",
    }));
}

#[test]
fn duplicate_object_mutator_declaration_is_rejected() {
    let mut architecture = ARCHITECTURE;

    let mut objects = architecture.objects.to_vec();
    let object = objects
        .iter_mut()
        .find(|object| !object.mutators.is_empty())
        .expect("some object declares a mutator");
    let object_id = object.id;

    let mut mutators = object.mutators.to_vec();
    mutators.push(mutators[0]);
    object.mutators = mutators.leak();
    architecture.objects = objects.leak();

    let errors = validate_draft(&architecture).unwrap_err();
    assert!(errors.contains(&ManifestError::InvalidObject {
        object: object_id,
        reason: "duplicate mutator declaration",
    }));
}

#[test]
fn duplicate_quantity_reader_declaration_is_rejected() {
    let mut architecture = ARCHITECTURE;

    let mut quantities = architecture.quantities.to_vec();
    let quantity = quantities
        .iter_mut()
        .find(|quantity| !quantity.readers.is_empty())
        .expect("some quantity declares a reader");
    let quantity_id = quantity.id;

    let mut readers = quantity.readers.to_vec();
    readers.push(readers[0]);
    quantity.readers = readers.leak();
    architecture.quantities = quantities.leak();

    let errors = validate_draft(&architecture).unwrap_err();
    assert!(errors.contains(&ManifestError::InvalidQuantity {
        quantity: quantity_id,
        reason: "duplicate reader declaration",
    }));
}

#[test]
fn duplicate_asset_destruction_operation_is_rejected() {
    let mut architecture = ARCHITECTURE;

    let mut assets = architecture.assets.to_vec();
    let asset = assets
        .iter_mut()
        .find(|asset| !asset.destruction_operations.is_empty())
        .expect("some asset declares a destruction operation");
    let asset_id = asset.id;

    let mut operations = asset.destruction_operations.to_vec();
    operations.push(operations[0]);
    asset.destruction_operations = operations.leak();
    architecture.assets = assets.leak();

    let errors = validate_draft(&architecture).unwrap_err();
    assert!(errors.contains(&ManifestError::InvalidAsset {
        asset: asset_id,
        reason: "duplicate destruction-operation declaration",
    }));
}

#[test]
fn duplicate_issuance_declaration_is_rejected() {
    let mut architecture = ARCHITECTURE;

    let mut operations = architecture.operations.to_vec();
    let operation = operations
        .iter_mut()
        .find(|operation| !operation.issuances.is_empty())
        .expect("some operation declares an issuance");
    let operation_id = operation.id;

    let mut issuances = operation.issuances.to_vec();
    issuances.push(issuances[0]);
    operation.issuances = issuances.leak();
    architecture.operations = operations.leak();

    let errors = validate_draft(&architecture).unwrap_err();
    assert!(errors.contains(&ManifestError::InvalidOperation {
        operation: operation_id,
        reason: "duplicate issuance declaration for one asset",
    }));
}

#[test]
fn duplicate_data_output_declaration_is_rejected() {
    let mut architecture = ARCHITECTURE;

    let mut operations = architecture.operations.to_vec();
    let operation = operations
        .iter_mut()
        .find(|operation| !operation.data_outputs.is_empty())
        .expect("some operation declares a data output");
    let operation_id = operation.id;

    let mut data_outputs = operation.data_outputs.to_vec();
    data_outputs.push(data_outputs[0]);
    operation.data_outputs = data_outputs.leak();
    architecture.operations = operations.leak();

    let errors = validate_draft(&architecture).unwrap_err();
    assert!(errors.contains(&ManifestError::InvalidOperation {
        operation: operation_id,
        reason: "duplicate data-output declaration",
    }));
}
