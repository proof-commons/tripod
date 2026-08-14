//! Deployment-evidence profile for one concrete realization.
//!
//! Architecture release and deployment release are distinct assurance
//! boundaries. `validate_architecture_release` proves only that the
//! typed manifest is final, pinned, and internally consistent; it
//! makes no claim that any Elements deployment has calibrated its
//! finite bounds, verified its substrate dependencies, or produced the
//! implementation artifacts. Those claims are carried by a
//! [`DeploymentProfile`], which binds calibrated limits, verified
//! dependencies, and artifact hashes to one architecture semantic hash
//! and one network/genesis identity, and is checked by
//! [`validate_deployment_release`].
//!
//! The profile has its own canonical hash under a domain-separated
//! algorithm identifier; the architecture hash algorithm is never
//! reused for profile bytes.
//!
//! Validation precedes identity (ADR-016). The profile hash is defined
//! only over a [`ValidatedDeploymentProfile`], which
//! [`validate_deployment_profile`] alone constructs, so "hashable"
//! cannot be mistaken for "valid" once a release consumer appears. The
//! unchecked canonical projection stays crate-private for mutation
//! tests. The identity remains dormant: no consumer, publication, or
//! generated artifact carries a deployment-profile hash today.

use core::fmt;

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::canonical::{canonicalize_json, hex};
use crate::ids::{BoundId, DependencyId, PublicationStatus};
use crate::spec::{Architecture, MaxCount};
use crate::validate::validate_architecture_release;

/// Canonical deployment-profile hash algorithm identifier. Domain
/// separation from the architecture manifest hash is provided both by
/// this identifier and by a domain prefix over the hashed bytes.
pub const DEPLOYMENT_HASH_ALGORITHM: &str = "sha256-canonical-json-deployment-v1";

const DEPLOYMENT_HASH_DOMAIN: &[u8] = b"tripod deployment profile JSON v1\n";

/// Supported deployment-profile schema version.
///
/// Schema version 2 splits the independent-indexer evidence into
/// separate event-projection, attestation-query, and
/// receipt-accounting report hashes, so the three claims are
/// separately committed. Schema version 1 profiles are rejected;
/// before final release, direct replacement is consistent with the
/// stated versioning policy.
pub const DEPLOYMENT_PROFILE_SCHEMA_VERSION: u32 = 2;

/// Verification status of one substrate dependency in a deployment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VerificationStatus {
    Pending,
    Verified,
    Failed,
}

impl VerificationStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Verified => "verified",
            Self::Failed => "failed",
        }
    }
}

/// Calibrated value and measurement evidence for one declared finite
/// bound. Every bound with `requires_deployment_calibration` must
/// appear exactly once in a release profile.
///
/// The calibration binds the exact emitted script bundle it measured
/// (`script_bundle_hash` must equal the profile's released
/// emitted-script-bundle artifact). Schema-2 residual: the calibration
/// cannot yet bind the transaction ABI/configuration under which the
/// measurement was taken, so bundle equality alone does not prove the
/// measured transaction shape used the final ABI. A future profile
/// schema must add that binding before any production release.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundCalibration {
    pub bound: BoundId,
    pub value: u64,

    pub evidence_hash: [u8; 32],
    pub script_bundle_hash: [u8; 32],

    pub measured_weight: u64,
    pub measured_witness_bytes: u64,
    pub measured_opcode_cost: u64,
}

/// Verification evidence for one substrate dependency.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyEvidence {
    pub dependency: DependencyId,
    pub status: VerificationStatus,

    pub evidence_hash: [u8; 32],
    pub tool_version: String,
    pub test_name: String,
}

/// Hashes of the implementation artifacts a deployment publishes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactHashes {
    pub normative_rust: [u8; 32],
    pub compiler_configuration: [u8; 32],
    pub emitted_script_bundle: [u8; 32],
    pub reference_indexer: [u8; 32],
    pub architecture_json: [u8; 32],
    pub architecture_toml: [u8; 32],
    pub canonical_wire_vectors: [u8; 32],
}

/// Hashes of the required test reports.
///
/// Event recognition, query computation, and residue accounting are
/// separate claims and require separate witnesses, so the independent
/// evidence names one report per claim: a candidate indexer can
/// recognize the wrong events yet accidentally compute the right
/// aggregate, and offsetting residue events can preserve totals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestEvidence {
    pub unit_test_report_hash: [u8; 32],
    pub property_test_report_hash: [u8; 32],

    /// Independent comparison of raw recognized burn and clear
    /// projections before grouping.
    pub independent_event_projection_report_hash: [u8; 32],

    /// Independent comparison of canonical attestation query terms
    /// and bytes, bound to context and manifest hash.
    pub independent_attestation_query_report_hash: [u8; 32],

    /// Independent receipt-accounting and residue-event comparison.
    /// This is external-auditor evidence, not attestation-indexer
    /// evidence.
    pub independent_receipt_accounting_report_hash: [u8; 32],

    pub script_integration_report_hash: [u8; 32],
}

/// Deployment-declared script resource maxima against which bound
/// calibrations are measured.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScriptLimits {
    pub max_weight: u64,
    pub max_witness_bytes: u64,
    pub max_opcode_cost: u64,
}

/// Deployment-evidence profile for one concrete realization on one
/// network.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeploymentProfile {
    pub schema_version: u32,
    pub status: PublicationStatus,

    pub architecture_semantic_hash: [u8; 32],

    pub network_id: [u8; 32],
    pub genesis_id: [u8; 32],

    pub script_limits: ScriptLimits,

    pub calibrated_bounds: Vec<BoundCalibration>,
    pub dependency_evidence: Vec<DependencyEvidence>,

    pub artifacts: ArtifactHashes,
    pub test_evidence: TestEvidence,
}

/// Deployment-release validation failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeploymentError {
    UnsupportedSchemaVersion,
    ArchitectureNotReleasable,
    ArchitectureHashMismatch,
    ProfileNotFinal,
    ZeroNetworkId,
    ZeroGenesisId,
    ZeroScriptLimit,

    MissingBoundCalibration(BoundId),
    DuplicateBoundCalibration(BoundId),
    UnexpectedBoundCalibration(BoundId),
    ZeroCalibratedValue(BoundId),
    CalibratedValueBelowManifestMinimum(BoundId),
    MissingBoundEvidence(BoundId),
    BoundCalibrationBundleMismatch(BoundId),
    MissingBoundMeasurement(BoundId),
    MeasurementExceedsScriptLimit(BoundId),

    MissingDependencyEvidence(DependencyId),
    DuplicateDependencyEvidence(DependencyId),
    DependencyNotVerified(DependencyId),
    MissingDependencyEvidenceHash(DependencyId),
    MissingDependencyToolVersion(DependencyId),
    MissingDependencyTestName(DependencyId),

    MissingArtifactHash(&'static str),
    MissingTestReportHash(&'static str),
}

impl fmt::Display for DeploymentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion => {
                formatter.write_str("unsupported deployment-profile schema version")
            }
            Self::ArchitectureNotReleasable => {
                formatter.write_str("architecture fails release validation")
            }
            Self::ArchitectureHashMismatch => {
                formatter.write_str("profile is bound to a different architecture hash")
            }
            Self::ProfileNotFinal => formatter.write_str("deployment profile is not final"),
            Self::ZeroNetworkId => formatter.write_str("network id is zero"),
            Self::ZeroGenesisId => formatter.write_str("genesis id is zero"),
            Self::ZeroScriptLimit => formatter.write_str("script limit is zero"),
            Self::MissingBoundCalibration(bound) => {
                write!(formatter, "missing calibration for bound {bound}")
            }
            Self::DuplicateBoundCalibration(bound) => {
                write!(formatter, "duplicate calibration for bound {bound}")
            }
            Self::UnexpectedBoundCalibration(bound) => {
                write!(
                    formatter,
                    "calibration entry for {bound}, which does not require deployment calibration"
                )
            }
            Self::ZeroCalibratedValue(bound) => {
                write!(formatter, "zero calibrated value for bound {bound}")
            }
            Self::CalibratedValueBelowManifestMinimum(bound) => {
                write!(formatter, "calibrated {bound} below a manifest minimum")
            }
            Self::MissingBoundEvidence(bound) => {
                write!(formatter, "missing calibration evidence for bound {bound}")
            }
            Self::BoundCalibrationBundleMismatch(bound) => {
                write!(
                    formatter,
                    "calibration for bound {bound} measured a different script bundle \
                     than the released emitted-script-bundle artifact"
                )
            }
            Self::MissingBoundMeasurement(bound) => {
                write!(formatter, "missing script measurement for bound {bound}")
            }
            Self::MeasurementExceedsScriptLimit(bound) => {
                write!(formatter, "measured {bound} exceeds a script limit")
            }
            Self::MissingDependencyEvidence(dependency) => {
                write!(formatter, "missing evidence for dependency {dependency}")
            }
            Self::DuplicateDependencyEvidence(dependency) => {
                write!(formatter, "duplicate evidence for dependency {dependency}")
            }
            Self::DependencyNotVerified(dependency) => {
                write!(formatter, "dependency {dependency} is not verified")
            }
            Self::MissingDependencyEvidenceHash(dependency) => {
                write!(
                    formatter,
                    "missing evidence hash for dependency {dependency}"
                )
            }
            Self::MissingDependencyToolVersion(dependency) => {
                write!(
                    formatter,
                    "missing tool version for dependency {dependency}"
                )
            }
            Self::MissingDependencyTestName(dependency) => {
                write!(formatter, "missing test name for dependency {dependency}")
            }
            Self::MissingArtifactHash(artifact) => {
                write!(formatter, "missing artifact hash: {artifact}")
            }
            Self::MissingTestReportHash(report) => {
                write!(formatter, "missing test report hash: {report}")
            }
        }
    }
}

impl std::error::Error for DeploymentError {}

/// The minima a bound's runtime value must dominate.
///
/// Every declared input, output, and data-output minimum whose
/// maximum references the bound contributes. Public so runtime bound
/// conformance (model constants) and deployment calibration validate
/// against the same typed authority instead of hardcoding minima.
pub fn manifest_minimum_for_bound(architecture: &Architecture, bound: BoundId) -> u64 {
    let mut minimum = 0_u64;

    for operation in architecture.operations {
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

        for (declared_minimum, maximum) in cardinalities {
            if maximum == MaxCount::Bound(bound) {
                minimum = minimum.max(u64::from(declared_minimum));
            }
        }
    }

    minimum
}

fn is_zero(hash: &[u8; 32]) -> bool {
    hash.iter().all(|byte| *byte == 0)
}

/// Deployment-release validation.
///
/// The architecture being final is necessary but never sufficient: a
/// deployment is release-ready only when this validation passes for a
/// final profile bound to the final architecture.
pub fn validate_deployment_release(
    architecture: &Architecture,
    profile: &DeploymentProfile,
) -> Result<(), Vec<DeploymentError>> {
    let mut errors = Vec::new();

    if profile.schema_version != DEPLOYMENT_PROFILE_SCHEMA_VERSION {
        errors.push(DeploymentError::UnsupportedSchemaVersion);
    }

    // Architecture binding: the architecture must itself be
    // releasable, and the profile must bind to its exact semantic
    // hash.
    match validate_architecture_release(architecture) {
        Ok(release) => match crate::canonical::semantic_hash(&release.draft()) {
            Ok(hash) if hash == profile.architecture_semantic_hash => {}
            _ => errors.push(DeploymentError::ArchitectureHashMismatch),
        },

        // An unreleasable architecture has no architecture identity to
        // compare the binding against (ADR-016 puts validation before
        // identity), so no binding verdict is claimed here; the profile
        // already fails closed on `ArchitectureNotReleasable`.
        Err(_) => errors.push(DeploymentError::ArchitectureNotReleasable),
    }

    if profile.status != PublicationStatus::Final {
        errors.push(DeploymentError::ProfileNotFinal);
    }

    // Network binding.
    if is_zero(&profile.network_id) {
        errors.push(DeploymentError::ZeroNetworkId);
    }

    if is_zero(&profile.genesis_id) {
        errors.push(DeploymentError::ZeroGenesisId);
    }

    if profile.script_limits.max_weight == 0
        || profile.script_limits.max_witness_bytes == 0
        || profile.script_limits.max_opcode_cost == 0
    {
        errors.push(DeploymentError::ZeroScriptLimit);
    }

    validate_bound_calibrations(architecture, profile, &mut errors);
    validate_dependency_evidence(architecture, profile, &mut errors);
    validate_artifact_hashes(profile, &mut errors);
    validate_test_evidence(profile, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_bound_calibrations(
    architecture: &Architecture,
    profile: &DeploymentProfile,
    errors: &mut Vec<DeploymentError>,
) {
    // Census, entry direction: every profile calibration must reference
    // a declared bound that actually requires deployment calibration.
    // A stray entry is rejected, never silently ignored — a final
    // profile has exactly one interpretation.
    for calibration in &profile.calibrated_bounds {
        let declared = architecture
            .bounds
            .iter()
            .find(|bound| bound.id == calibration.bound);

        if !declared.is_some_and(|bound| bound.requires_deployment_calibration) {
            errors.push(DeploymentError::UnexpectedBoundCalibration(
                calibration.bound,
            ));
        }
    }

    for bound in architecture.bounds {
        let entries = profile
            .calibrated_bounds
            .iter()
            .filter(|calibration| calibration.bound == bound.id)
            .collect::<Vec<_>>();

        if !bound.requires_deployment_calibration {
            continue;
        }

        let calibration = match entries.len() {
            0 => {
                errors.push(DeploymentError::MissingBoundCalibration(bound.id));
                continue;
            }

            1 => entries[0],

            _ => {
                errors.push(DeploymentError::DuplicateBoundCalibration(bound.id));
                continue;
            }
        };

        if calibration.value == 0 {
            errors.push(DeploymentError::ZeroCalibratedValue(bound.id));
        }

        if calibration.value < manifest_minimum_for_bound(architecture, bound.id) {
            errors.push(DeploymentError::CalibratedValueBelowManifestMinimum(
                bound.id,
            ));
        }

        if is_zero(&calibration.evidence_hash) || is_zero(&calibration.script_bundle_hash) {
            errors.push(DeploymentError::MissingBoundEvidence(bound.id));
        } else if calibration.script_bundle_hash != profile.artifacts.emitted_script_bundle {
            // Calibration evidence is meaningful only for the exact
            // bundle being released: a measurement taken against any
            // other script bundle is stale, whatever its values say.
            errors.push(DeploymentError::BoundCalibrationBundleMismatch(bound.id));
        }

        if calibration.measured_weight == 0
            || calibration.measured_witness_bytes == 0
            || calibration.measured_opcode_cost == 0
        {
            errors.push(DeploymentError::MissingBoundMeasurement(bound.id));
        }

        if calibration.measured_weight > profile.script_limits.max_weight
            || calibration.measured_witness_bytes > profile.script_limits.max_witness_bytes
            || calibration.measured_opcode_cost > profile.script_limits.max_opcode_cost
        {
            errors.push(DeploymentError::MeasurementExceedsScriptLimit(bound.id));
        }
    }
}

fn validate_dependency_evidence(
    architecture: &Architecture,
    profile: &DeploymentProfile,
    errors: &mut Vec<DeploymentError>,
) {
    // The profile multiplicity rule: a required dependency appears
    // exactly once; optional evidence appears at most once; presence
    // implies well-formedness (nonzero hash, non-blank tool version
    // and test name); requirement additionally implies verified
    // status. Duplicates are rejected before — and regardless of —
    // the verification_required gate.
    for dependency in architecture.dependencies {
        let entries = profile
            .dependency_evidence
            .iter()
            .filter(|evidence| evidence.dependency == dependency.id)
            .collect::<Vec<_>>();

        if entries.len() > 1 {
            errors.push(DeploymentError::DuplicateDependencyEvidence(dependency.id));
            continue;
        }

        let Some(evidence) = entries.first() else {
            if dependency.verification_required {
                errors.push(DeploymentError::MissingDependencyEvidence(dependency.id));
            }
            continue;
        };

        if dependency.verification_required && evidence.status != VerificationStatus::Verified {
            errors.push(DeploymentError::DependencyNotVerified(dependency.id));
        }

        if is_zero(&evidence.evidence_hash) {
            errors.push(DeploymentError::MissingDependencyEvidenceHash(
                dependency.id,
            ));
        }

        if evidence.tool_version.trim().is_empty() {
            errors.push(DeploymentError::MissingDependencyToolVersion(dependency.id));
        }

        if evidence.test_name.trim().is_empty() {
            errors.push(DeploymentError::MissingDependencyTestName(dependency.id));
        }
    }
}

fn validate_artifact_hashes(profile: &DeploymentProfile, errors: &mut Vec<DeploymentError>) {
    let artifacts = &profile.artifacts;

    let required: &[(&'static str, &[u8; 32])] = &[
        ("normative-rust", &artifacts.normative_rust),
        ("compiler-configuration", &artifacts.compiler_configuration),
        ("emitted-script-bundle", &artifacts.emitted_script_bundle),
        ("reference-indexer", &artifacts.reference_indexer),
        ("architecture-json", &artifacts.architecture_json),
        ("architecture-toml", &artifacts.architecture_toml),
        ("canonical-wire-vectors", &artifacts.canonical_wire_vectors),
    ];

    for (name, hash) in required {
        if is_zero(hash) {
            errors.push(DeploymentError::MissingArtifactHash(name));
        }
    }
}

fn validate_test_evidence(profile: &DeploymentProfile, errors: &mut Vec<DeploymentError>) {
    let evidence = &profile.test_evidence;

    let required: &[(&'static str, &[u8; 32])] = &[
        ("unit-test-report", &evidence.unit_test_report_hash),
        ("property-test-report", &evidence.property_test_report_hash),
        (
            "independent-event-projection-report",
            &evidence.independent_event_projection_report_hash,
        ),
        (
            "independent-attestation-query-report",
            &evidence.independent_attestation_query_report_hash,
        ),
        (
            "independent-receipt-accounting-report",
            &evidence.independent_receipt_accounting_report_hash,
        ),
        (
            "script-integration-report",
            &evidence.script_integration_report_hash,
        ),
    ];

    for (name, hash) in required {
        if is_zero(hash) {
            errors.push(DeploymentError::MissingTestReportHash(name));
        }
    }
}

// -------------------------------------------------------------------------
// Canonical profile hash
// -------------------------------------------------------------------------

fn hash_value(hash: &[u8; 32]) -> Value {
    Value::String(hex(hash))
}

fn object(entries: Vec<(&'static str, Value)>) -> Value {
    let mut map = Map::new();

    for (key, value) in entries {
        map.insert(key.to_owned(), value);
    }

    Value::Object(map)
}

fn profile_value(profile: &DeploymentProfile) -> Value {
    let mut bounds = profile
        .calibrated_bounds
        .iter()
        .map(|calibration| {
            object(vec![
                (
                    "bound",
                    Value::String(calibration.bound.as_str().to_owned()),
                ),
                ("value", Value::from(calibration.value)),
                ("evidence_hash", hash_value(&calibration.evidence_hash)),
                (
                    "script_bundle_hash",
                    hash_value(&calibration.script_bundle_hash),
                ),
                ("measured_weight", Value::from(calibration.measured_weight)),
                (
                    "measured_witness_bytes",
                    Value::from(calibration.measured_witness_bytes),
                ),
                (
                    "measured_opcode_cost",
                    Value::from(calibration.measured_opcode_cost),
                ),
            ])
        })
        .collect::<Vec<_>>();

    bounds.sort_by_key(ToString::to_string);

    let mut dependencies = profile
        .dependency_evidence
        .iter()
        .map(|evidence| {
            object(vec![
                (
                    "dependency",
                    Value::String(evidence.dependency.as_str().to_owned()),
                ),
                ("status", Value::String(evidence.status.as_str().to_owned())),
                ("evidence_hash", hash_value(&evidence.evidence_hash)),
                ("tool_version", Value::String(evidence.tool_version.clone())),
                ("test_name", Value::String(evidence.test_name.clone())),
            ])
        })
        .collect::<Vec<_>>();

    dependencies.sort_by_key(ToString::to_string);

    object(vec![
        ("schema_version", Value::from(profile.schema_version)),
        ("status", Value::String(profile.status.as_str().to_owned())),
        (
            "architecture_semantic_hash",
            hash_value(&profile.architecture_semantic_hash),
        ),
        ("network_id", hash_value(&profile.network_id)),
        ("genesis_id", hash_value(&profile.genesis_id)),
        (
            "script_limits",
            object(vec![
                ("max_weight", Value::from(profile.script_limits.max_weight)),
                (
                    "max_witness_bytes",
                    Value::from(profile.script_limits.max_witness_bytes),
                ),
                (
                    "max_opcode_cost",
                    Value::from(profile.script_limits.max_opcode_cost),
                ),
            ]),
        ),
        ("calibrated_bounds", Value::Array(bounds)),
        ("dependency_evidence", Value::Array(dependencies)),
        (
            "artifacts",
            object(vec![
                (
                    "normative_rust",
                    hash_value(&profile.artifacts.normative_rust),
                ),
                (
                    "compiler_configuration",
                    hash_value(&profile.artifacts.compiler_configuration),
                ),
                (
                    "emitted_script_bundle",
                    hash_value(&profile.artifacts.emitted_script_bundle),
                ),
                (
                    "reference_indexer",
                    hash_value(&profile.artifacts.reference_indexer),
                ),
                (
                    "architecture_json",
                    hash_value(&profile.artifacts.architecture_json),
                ),
                (
                    "architecture_toml",
                    hash_value(&profile.artifacts.architecture_toml),
                ),
                (
                    "canonical_wire_vectors",
                    hash_value(&profile.artifacts.canonical_wire_vectors),
                ),
            ]),
        ),
        (
            "test_evidence",
            object(vec![
                (
                    "unit_test_report_hash",
                    hash_value(&profile.test_evidence.unit_test_report_hash),
                ),
                (
                    "property_test_report_hash",
                    hash_value(&profile.test_evidence.property_test_report_hash),
                ),
                (
                    "independent_event_projection_report_hash",
                    hash_value(
                        &profile
                            .test_evidence
                            .independent_event_projection_report_hash,
                    ),
                ),
                (
                    "independent_attestation_query_report_hash",
                    hash_value(
                        &profile
                            .test_evidence
                            .independent_attestation_query_report_hash,
                    ),
                ),
                (
                    "independent_receipt_accounting_report_hash",
                    hash_value(
                        &profile
                            .test_evidence
                            .independent_receipt_accounting_report_hash,
                    ),
                ),
                (
                    "script_integration_report_hash",
                    hash_value(&profile.test_evidence.script_integration_report_hash),
                ),
            ]),
        ),
    ])
}

/// A deployment profile that has passed release validation.
///
/// The wrapper is the type-level record of the identity rule stated in
/// ADR-016: a complete typed object is validated first, then projected
/// canonically, and only then does it bear an identity. Because the
/// only constructor is [`validate_deployment_profile`], holding one of
/// these is proof that [`validate_deployment_release`] accepted the
/// profile against the architecture it binds, so no caller can mint a
/// canonical profile identity for a draft, mis-bound, uncalibrated, or
/// otherwise unreleasable profile.
///
/// The identity itself remains dormant: nothing in the workspace
/// consumes a profile hash or publishes one.
#[derive(Clone, Copy, Debug)]
pub struct ValidatedDeploymentProfile<'a> {
    architecture: &'a Architecture,
    profile: &'a DeploymentProfile,
}

impl<'a> ValidatedDeploymentProfile<'a> {
    /// The architecture the profile was validated against.
    pub const fn architecture(&self) -> &'a Architecture {
        self.architecture
    }

    /// The validated profile.
    pub const fn profile(&self) -> &'a DeploymentProfile {
        self.profile
    }
}

/// Validate a deployment release and carry the verdict in the type.
///
/// This is the sole entry to profile identity: the returned wrapper is
/// the only value [`deployment_profile_hash`] accepts.
pub fn validate_deployment_profile<'a>(
    architecture: &'a Architecture,
    profile: &'a DeploymentProfile,
) -> Result<ValidatedDeploymentProfile<'a>, Vec<DeploymentError>> {
    validate_deployment_release(architecture, profile)?;

    Ok(ValidatedDeploymentProfile {
        architecture,
        profile,
    })
}

/// Canonical semantic hash of a validated deployment profile,
/// domain-separated from the architecture manifest hash.
///
/// Validation precedes identity: the argument type is reachable only
/// through [`validate_deployment_profile`], so an invalid profile has
/// no hash under this API.
pub fn deployment_profile_hash(
    validated: &ValidatedDeploymentProfile<'_>,
) -> Result<[u8; 32], serde_json::Error> {
    unchecked_deployment_profile_hash(validated.profile)
}

/// Canonical profile bytes without the validation precondition.
///
/// Crate-private on purpose: mutation tests need to observe which
/// fields the canonical projection commits to, including on profiles
/// that deliberately fail release validation. No public caller can
/// reach a profile identity without validating first.
pub(crate) fn unchecked_deployment_profile_hash(
    profile: &DeploymentProfile,
) -> Result<[u8; 32], serde_json::Error> {
    let canonical = canonicalize_json(profile_value(profile));

    let bytes = serde_json::to_vec(&canonical)?;

    let mut hasher = Sha256::new();

    hasher.update(DEPLOYMENT_HASH_DOMAIN);
    hasher.update(&bytes);

    let digest = hasher.finalize();

    let mut result = [0_u8; 32];
    result.copy_from_slice(&digest);

    Ok(result)
}

/// Hex form of [`deployment_profile_hash`].
pub fn deployment_profile_hash_hex(
    validated: &ValidatedDeploymentProfile<'_>,
) -> Result<String, serde_json::Error> {
    Ok(hex(&deployment_profile_hash(validated)?))
}
