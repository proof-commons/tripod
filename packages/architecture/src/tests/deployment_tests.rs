//! Deployment-release validation tests: a fully populated synthetic
//! final profile must validate, and every missing or mismatched field
//! must be rejected.

use crate::*;

/// A final, pinned architecture for release testing.
fn release_architecture() -> Architecture {
    let mut architecture = ARCHITECTURE;

    architecture.document.status = PublicationStatus::Final;
    architecture.document.specification.anchor_set_hash = Some([0x11; 32]);

    architecture
}

/// A fully populated synthetic final deployment profile bound to the
/// release architecture.
fn release_profile(architecture: &Architecture) -> DeploymentProfile {
    let calibrated_bounds = architecture
        .bounds
        .iter()
        .map(|bound| BoundCalibration {
            bound: bound.id,
            value: bound.default_value.unwrap_or(32),
            evidence_hash: [0xA1; 32],
            script_bundle_hash: [0xA2; 32],
            measured_weight: 150_000,
            measured_witness_bytes: 40_000,
            measured_opcode_cost: 20_000,
        })
        .collect();

    let dependency_evidence = architecture
        .dependencies
        .iter()
        .map(|dependency| DependencyEvidence {
            dependency: dependency.id,
            status: VerificationStatus::Verified,
            evidence_hash: [0xB1; 32],
            tool_version: "elements-23.2.1".to_owned(),
            test_name: format!("verify-{}", dependency.id),
        })
        .collect();

    DeploymentProfile {
        schema_version: DEPLOYMENT_PROFILE_SCHEMA_VERSION,
        status: PublicationStatus::Final,

        architecture_semantic_hash: semantic_hash(architecture).unwrap(),

        network_id: [0xC1; 32],
        genesis_id: [0xC2; 32],

        script_limits: ScriptLimits {
            max_weight: 400_000,
            max_witness_bytes: 100_000,
            max_opcode_cost: 50_000,
        },

        calibrated_bounds,
        dependency_evidence,

        artifacts: ArtifactHashes {
            normative_rust: [0xD1; 32],
            compiler_configuration: [0xD2; 32],
            emitted_script_bundle: [0xD3; 32],
            reference_indexer: [0xD4; 32],
            architecture_json: [0xD5; 32],
            architecture_toml: [0xD6; 32],
            canonical_wire_vectors: [0xD7; 32],
        },

        test_evidence: TestEvidence {
            unit_test_report_hash: [0xE1; 32],
            property_test_report_hash: [0xE2; 32],
            independent_event_projection_report_hash: [0xE3; 32],
            independent_attestation_query_report_hash: [0xE5; 32],
            independent_receipt_accounting_report_hash: [0xE6; 32],
            script_integration_report_hash: [0xE4; 32],
        },
    }
}

#[test]
fn fully_populated_final_profile_validates() {
    let architecture = release_architecture();
    let profile = release_profile(&architecture);

    validate_deployment_release(&architecture, &profile).unwrap();
}

#[test]
fn draft_architecture_is_rejected_for_deployment() {
    let architecture = release_architecture();
    let profile = release_profile(&architecture);

    let mut draft = architecture;
    draft.document.status = PublicationStatus::Draft;

    // The profile hash was bound to the final architecture; both the
    // release failure and the hash mismatch must surface.
    let errors = validate_deployment_release(&draft, &profile).unwrap_err();

    assert!(errors.contains(&DeploymentError::ArchitectureNotReleasable));
}

#[test]
fn unpinned_specification_is_rejected_for_deployment() {
    let mut architecture = release_architecture();
    architecture.document.specification.anchor_set_hash = None;

    let profile = release_profile(&architecture);

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(errors.contains(&DeploymentError::ArchitectureNotReleasable));
}

#[test]
fn architecture_hash_mismatch_is_rejected() {
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);
    profile.architecture_semantic_hash = [0x99; 32];

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(errors.contains(&DeploymentError::ArchitectureHashMismatch));
}

#[test]
fn draft_profile_is_rejected() {
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);
    profile.status = PublicationStatus::Draft;

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(errors.contains(&DeploymentError::ProfileNotFinal));
}

#[test]
fn unsupported_schema_version_is_rejected() {
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);
    profile.schema_version = 0;

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(errors.contains(&DeploymentError::UnsupportedSchemaVersion));
}

#[test]
fn zero_network_and_genesis_ids_are_rejected() {
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);
    profile.network_id = [0; 32];
    profile.genesis_id = [0; 32];

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(errors.contains(&DeploymentError::ZeroNetworkId));
    assert!(errors.contains(&DeploymentError::ZeroGenesisId));
}

#[test]
fn missing_bound_calibration_is_rejected() {
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);
    profile
        .calibrated_bounds
        .retain(|calibration| calibration.bound != BoundId::BurnInputMax);

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(errors.contains(&DeploymentError::MissingBoundCalibration(
        BoundId::BurnInputMax,
    )));
}

#[test]
fn duplicate_bound_calibration_is_rejected() {
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);
    let duplicate = profile.calibrated_bounds[0].clone();
    let bound = duplicate.bound;
    profile.calibrated_bounds.push(duplicate);

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(errors.contains(&DeploymentError::DuplicateBoundCalibration(bound)));
}

#[test]
fn zero_calibrated_bound_is_rejected() {
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);

    for calibration in &mut profile.calibrated_bounds {
        if calibration.bound == BoundId::AdmissionBatchMax {
            calibration.value = 0;
        }
    }

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(errors.contains(&DeploymentError::ZeroCalibratedValue(
        BoundId::AdmissionBatchMax,
    )));
}

#[test]
fn calibrated_bound_below_manifest_minimum_is_rejected() {
    // ASH compaction requires at least two ASH inputs, so calibrating
    // ASH_BATCH_MAX to one contradicts the manifest.
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);

    for calibration in &mut profile.calibrated_bounds {
        if calibration.bound == BoundId::AshBatchMax {
            calibration.value = 1;
        }
    }

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(
        errors.contains(&DeploymentError::CalibratedValueBelowManifestMinimum(
            BoundId::AshBatchMax,
        ))
    );
}

#[test]
fn missing_bound_evidence_hash_is_rejected() {
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);

    for calibration in &mut profile.calibrated_bounds {
        if calibration.bound == BoundId::TransferInputMax {
            calibration.evidence_hash = [0; 32];
        }
    }

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(errors.contains(&DeploymentError::MissingBoundEvidence(
        BoundId::TransferInputMax,
    )));
}

#[test]
fn measurement_exceeding_script_limits_is_rejected() {
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);

    for calibration in &mut profile.calibrated_bounds {
        if calibration.bound == BoundId::SettlementBatchMax {
            calibration.measured_weight = profile.script_limits.max_weight + 1;
        }
    }

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(
        errors.contains(&DeploymentError::MeasurementExceedsScriptLimit(
            BoundId::SettlementBatchMax,
        ))
    );
}

// Census rules: every profile entry has exactly one interpretation.
// Calibrations for bounds that require no calibration are rejected
// (never silently ignored); duplicates are rejected regardless of
// entry order and regardless of the verification_required gate;
// whitespace-only evidence fields are as missing as empty ones.

#[test]
fn unexpected_bound_calibration_is_rejected() {
    let mut architecture = release_architecture();

    // Reclassify one bound as fixed (no deployment calibration): the
    // synthetic profile still carries a calibration entry for it,
    // which must now be rejected as unexpected, not silently skipped.
    let mut bounds = architecture.bounds.to_vec();
    bounds[0].requires_deployment_calibration = false;
    let bounds = bounds.leak();
    architecture.bounds = bounds;

    let profile = release_profile(&architecture);

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(errors.contains(&DeploymentError::UnexpectedBoundCalibration(bounds[0].id,)));
}

#[test]
fn duplicate_optional_dependency_evidence_is_rejected() {
    // ValueCommitmentOpening is the declared optional dependency
    // (verification_required = false). Duplicated optional evidence
    // must be rejected even though the dependency itself is optional.
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);
    let duplicate = profile
        .dependency_evidence
        .iter()
        .find(|evidence| evidence.dependency == DependencyId::ValueCommitmentOpening)
        .expect("synthetic profile covers every declared dependency")
        .clone();
    profile.dependency_evidence.insert(0, duplicate);

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(
        errors.contains(&DeploymentError::DuplicateDependencyEvidence(
            DependencyId::ValueCommitmentOpening,
        ))
    );
}

#[test]
fn duplicate_entries_are_rejected_in_any_order() {
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);
    let duplicate_bound = profile.calibrated_bounds[2].clone();
    let bound = duplicate_bound.bound;
    profile.calibrated_bounds.insert(0, duplicate_bound);

    let duplicate_evidence = profile.dependency_evidence[3].clone();
    let dependency = duplicate_evidence.dependency;
    profile.dependency_evidence.insert(0, duplicate_evidence);

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(errors.contains(&DeploymentError::DuplicateBoundCalibration(bound)));
    assert!(errors.contains(&DeploymentError::DuplicateDependencyEvidence(dependency)));
}

#[test]
fn whitespace_only_dependency_fields_are_rejected() {
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);

    for evidence in &mut profile.dependency_evidence {
        if evidence.dependency == DependencyId::SighashProfile {
            evidence.tool_version = "   ".to_owned();
            evidence.test_name = "\t\n".to_owned();
        }
    }

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(
        errors.contains(&DeploymentError::MissingDependencyToolVersion(
            DependencyId::SighashProfile,
        ))
    );
    assert!(errors.contains(&DeploymentError::MissingDependencyTestName(
        DependencyId::SighashProfile,
    )));
}

#[test]
fn present_optional_evidence_must_be_well_formed() {
    // The optional-evidence policy: presence implies well-formedness.
    // Optional evidence may carry any verification status, but a
    // blank tool version or zero hash is rejected.
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);

    for evidence in &mut profile.dependency_evidence {
        if evidence.dependency == DependencyId::ValueCommitmentOpening {
            evidence.status = VerificationStatus::Pending;
            evidence.evidence_hash = [0; 32];
            evidence.tool_version = " ".to_owned();
        }
    }

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    // Pending status alone is allowed for an optional dependency…
    assert!(!errors.contains(&DeploymentError::DependencyNotVerified(
        DependencyId::ValueCommitmentOpening,
    )));
    // …but malformed fields are not.
    assert!(
        errors.contains(&DeploymentError::MissingDependencyEvidenceHash(
            DependencyId::ValueCommitmentOpening,
        ))
    );
    assert!(
        errors.contains(&DeploymentError::MissingDependencyToolVersion(
            DependencyId::ValueCommitmentOpening,
        ))
    );
}

#[test]
fn absent_optional_evidence_is_allowed() {
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);
    profile
        .dependency_evidence
        .retain(|evidence| evidence.dependency != DependencyId::ValueCommitmentOpening);

    validate_deployment_release(&architecture, &profile).unwrap();
}

#[test]
fn missing_required_dependency_is_rejected() {
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);
    profile
        .dependency_evidence
        .retain(|evidence| evidence.dependency != DependencyId::WeldEnforcement);

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(errors.contains(&DeploymentError::MissingDependencyEvidence(
        DependencyId::WeldEnforcement,
    )));
}

#[test]
fn pending_and_failed_dependencies_are_rejected() {
    let architecture = release_architecture();

    for status in [VerificationStatus::Pending, VerificationStatus::Failed] {
        let mut profile = release_profile(&architecture);

        for evidence in &mut profile.dependency_evidence {
            if evidence.dependency == DependencyId::PackageRelay {
                evidence.status = status;
            }
        }

        let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

        assert!(errors.contains(&DeploymentError::DependencyNotVerified(
            DependencyId::PackageRelay,
        )));
    }
}

#[test]
fn missing_dependency_evidence_fields_are_rejected() {
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);

    for evidence in &mut profile.dependency_evidence {
        if evidence.dependency == DependencyId::SighashProfile {
            evidence.evidence_hash = [0; 32];
            evidence.tool_version = String::new();
            evidence.test_name = String::new();
        }
    }

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(
        errors.contains(&DeploymentError::MissingDependencyEvidenceHash(
            DependencyId::SighashProfile,
        ))
    );
    assert!(
        errors.contains(&DeploymentError::MissingDependencyToolVersion(
            DependencyId::SighashProfile,
        ))
    );
    assert!(errors.contains(&DeploymentError::MissingDependencyTestName(
        DependencyId::SighashProfile,
    )));
}

#[test]
fn missing_artifact_hashes_are_rejected() {
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);
    profile.artifacts.compiler_configuration = [0; 32];
    profile.artifacts.emitted_script_bundle = [0; 32];
    profile.artifacts.reference_indexer = [0; 32];

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(errors.contains(&DeploymentError::MissingArtifactHash(
        "compiler-configuration",
    )));
    assert!(errors.contains(&DeploymentError::MissingArtifactHash(
        "emitted-script-bundle",
    )));
    assert!(errors.contains(&DeploymentError::MissingArtifactHash("reference-indexer")));
}

#[test]
fn missing_test_report_hashes_are_rejected() {
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);
    profile.test_evidence.script_integration_report_hash = [0; 32];

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(errors.contains(&DeploymentError::MissingTestReportHash(
        "script-integration-report",
    )));
}

// Each independent-evidence report hash is separately required: the
// event-projection, attestation-query, and receipt-accounting claims
// have separate witnesses and none may stand in for another.

#[test]
fn missing_event_projection_report_hash_is_rejected() {
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);
    profile
        .test_evidence
        .independent_event_projection_report_hash = [0; 32];

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(errors.contains(&DeploymentError::MissingTestReportHash(
        "independent-event-projection-report",
    )));
}

#[test]
fn missing_attestation_query_report_hash_is_rejected() {
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);
    profile
        .test_evidence
        .independent_attestation_query_report_hash = [0; 32];

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(errors.contains(&DeploymentError::MissingTestReportHash(
        "independent-attestation-query-report",
    )));
}

#[test]
fn missing_receipt_accounting_report_hash_is_rejected() {
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);
    profile
        .test_evidence
        .independent_receipt_accounting_report_hash = [0; 32];

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(errors.contains(&DeploymentError::MissingTestReportHash(
        "independent-receipt-accounting-report",
    )));
}

#[test]
fn profile_schema_version_two_is_required() {
    assert_eq!(DEPLOYMENT_PROFILE_SCHEMA_VERSION, 2);

    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);
    profile.schema_version = 1;

    let errors = validate_deployment_release(&architecture, &profile).unwrap_err();

    assert!(errors.contains(&DeploymentError::UnsupportedSchemaVersion));
}

#[test]
fn final_architecture_alone_is_not_deployment_ready() {
    // The v13 draft architecture with an empty evidence profile must
    // never validate as a deployment release: finality of the
    // architecture is necessary but not sufficient.
    let architecture = release_architecture();

    let mut profile = release_profile(&architecture);
    profile.calibrated_bounds.clear();
    profile.dependency_evidence.clear();

    assert!(validate_deployment_release(&architecture, &profile).is_err());
}

#[test]
fn deployment_profile_hash_is_stable_and_domain_separated() {
    let architecture = release_architecture();
    let profile = release_profile(&architecture);

    assert_eq!(
        deployment_profile_hash(&profile).unwrap(),
        deployment_profile_hash(&profile).unwrap(),
    );

    // The profile hash is not the architecture hash: the domains are
    // separated even when the profile embeds the architecture hash.
    assert_ne!(
        deployment_profile_hash(&profile).unwrap(),
        semantic_hash(&architecture).unwrap(),
    );

    assert_eq!(
        DEPLOYMENT_HASH_ALGORITHM,
        "sha256-canonical-json-deployment-v1"
    );
}

#[test]
fn deployment_profile_hash_changes_with_content() {
    let architecture = release_architecture();
    let profile = release_profile(&architecture);

    let mut modified = profile.clone();
    modified.genesis_id = [0xC3; 32];

    assert_ne!(
        deployment_profile_hash(&profile).unwrap(),
        deployment_profile_hash(&modified).unwrap(),
    );
}

// The profile hash commits each independent-evidence report
// separately: changing exactly one report hash changes the profile
// hash, so the three claims cannot be conflated in the commitment.

#[test]
fn profile_hash_changes_when_event_report_changes() {
    let architecture = release_architecture();
    let profile = release_profile(&architecture);

    let mut modified = profile.clone();
    modified
        .test_evidence
        .independent_event_projection_report_hash = [0xF1; 32];

    assert_ne!(
        deployment_profile_hash(&profile).unwrap(),
        deployment_profile_hash(&modified).unwrap(),
    );
}

#[test]
fn profile_hash_changes_when_query_report_changes() {
    let architecture = release_architecture();
    let profile = release_profile(&architecture);

    let mut modified = profile.clone();
    modified
        .test_evidence
        .independent_attestation_query_report_hash = [0xF2; 32];

    assert_ne!(
        deployment_profile_hash(&profile).unwrap(),
        deployment_profile_hash(&modified).unwrap(),
    );
}

#[test]
fn profile_hash_changes_when_accounting_report_changes() {
    let architecture = release_architecture();
    let profile = release_profile(&architecture);

    let mut modified = profile.clone();
    modified
        .test_evidence
        .independent_receipt_accounting_report_hash = [0xF3; 32];

    assert_ne!(
        deployment_profile_hash(&profile).unwrap(),
        deployment_profile_hash(&modified).unwrap(),
    );
}
