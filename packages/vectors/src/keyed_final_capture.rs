//! Test-local final-capture manifest and offline evidence joins.
//!
//! Mock and native runs share schemas, not complete transaction identities.
//! Every digest below names the bytes of this run; no chain premise is discharged.
use serde::{Deserialize, Serialize};
use target_elements_conformance::constructor::tagged::sha256;
use target_elements_conformance::protocol::{
    ExecutorHandshake, NativeOperationRequest, NativeOperationResponse, ObservedOutcomeLayer,
    OperationSubject,
};

use crate::keyed_final_native::{
    CaptureProvenance, JournalRefusal, ProvenanceRefusal, bind_capture_journal,
    bind_capture_provenance, capture_freeze_digest,
};

/// Independently launched capture, never an ordinary gate's native workload.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureTarget {
    /// Birth position and flags, beside the final constructor's controls.
    Birth,
    /// Program, version, key and output-position continuation.
    Continuation,
    /// Leaf/control authentication and the unavoidable key path.
    Commitment,
    /// Available observations of the exact final transactions.
    Cost,
}

/// Reviewed schema identity. A schema is not a transaction digest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureSchema {
    /// First SOURCE, then its change is spent in the single-member event.
    SourceA,
    /// SOURCE for the second disjoint member.
    SourceB,
    /// Member at input one, ordinary graph-local change at input zero.
    SingleOrdinaryEdge,
    /// Two members, with the sponsor between them.
    MultiNonzeroMembers,
    /// Extra SOURCE of the other key, supplying the changed-member subject.
    ForeignSource,
    /// Untrusted but balanced setup with identical keyed outputs zero and one.
    LaterParent,
    /// Sealed spend of the setup's output zero, beside its output-one mutant.
    LaterControl,
    /// An explicitly untrusted later-born parent/output subject.
    LaterBorn,
    /// An explicitly untrusted flagged input serialization.
    Flagged,
    /// Another key's successor program.
    WrongKey,
    /// One changed successor payload byte.
    WrongScript,
    /// One changed successor witness version.
    WrongVersion,
    /// Successor moved away from output zero.
    DisplacedSuccessor,
    /// One changed member beside another unchanged member.
    ChangedMember,
    /// One changed control parity bit.
    WrongParity,
    /// A script not committed by the consumed output.
    UncommittedLeaf,
    /// An invalid key-path signature, not evidence of key unspendability.
    KeyPathAttempt,
    /// Unadmitted single-input script-path twin of the key-path probe.
    KeyPathControl,
}

/// Boundary the reviewed subject intends to reach, not an executor assertion.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureBoundary {
    /// A complete target acceptance, still conditional on external premises.
    Acceptance,
    /// The birth-position predicate.
    Birth,
    /// Target transaction form before script evaluation.
    TransactionForm,
    /// The continuation predicate.
    Continuation,
    /// Authentication of the revealed script/control pair.
    Commitment,
    /// Key-path verification, where no covenant clause executes.
    KeyPath,
}

impl CaptureSchema {
    /// The expected boundary is a reviewed schema property, not a report field.
    #[must_use]
    pub const fn boundary(self) -> CaptureBoundary {
        match self {
            Self::SourceA
            | Self::SourceB
            | Self::SingleOrdinaryEdge
            | Self::MultiNonzeroMembers
            | Self::ForeignSource
            | Self::LaterParent
            | Self::LaterControl
            | Self::KeyPathControl => CaptureBoundary::Acceptance,
            Self::LaterBorn => CaptureBoundary::Birth,
            Self::Flagged => CaptureBoundary::TransactionForm,
            Self::WrongKey
            | Self::WrongScript
            | Self::WrongVersion
            | Self::DisplacedSuccessor
            | Self::ChangedMember => CaptureBoundary::Continuation,
            Self::WrongParity | Self::UncommittedLeaf => CaptureBoundary::Commitment,
            Self::KeyPathAttempt => CaptureBoundary::KeyPath,
        }
    }
    /// Expected protocol layer; a diagnostic does not replace this target verdict.
    #[must_use]
    pub const fn outcome(self) -> ObservedOutcomeLayer {
        match self.boundary() {
            CaptureBoundary::Acceptance => ObservedOutcomeLayer::Accepted,
            CaptureBoundary::TransactionForm => {
                ObservedOutcomeLayer::ConsensusRejectionBeforeScript
            }
            CaptureBoundary::Birth
            | CaptureBoundary::Continuation
            | CaptureBoundary::Commitment => ObservedOutcomeLayer::ScriptPathRejection,
            CaptureBoundary::KeyPath => ObservedOutcomeLayer::KeyPathRejection,
        }
    }
}

/// Reviewed census, identical for mock and native runs under each public key.
#[must_use]
pub fn schemas(target: CaptureTarget) -> Vec<CaptureSchema> {
    use CaptureSchema as S;
    let mut result = vec![
        S::SourceA,
        S::SingleOrdinaryEdge,
        S::SourceB,
        S::MultiNonzeroMembers,
    ];
    result.extend_from_slice(match target {
        CaptureTarget::Birth => &[S::LaterParent, S::LaterControl, S::LaterBorn, S::Flagged],
        CaptureTarget::Continuation => &[
            S::ForeignSource,
            S::WrongKey,
            S::WrongScript,
            S::WrongVersion,
            S::DisplacedSuccessor,
            S::ChangedMember,
        ],
        CaptureTarget::Commitment => &[
            S::WrongParity,
            S::UncommittedLeaf,
            S::KeyPathControl,
            S::KeyPathAttempt,
        ],
        CaptureTarget::Cost => &[],
    });
    result
}

/// One row flushed before its submission. Negative bytes are explicitly untrusted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureRow {
    /// Reviewed subject schema, checked against the target's census.
    pub schema: CaptureSchema,
    /// Public supplied key.
    pub key: [u8; 32],
    /// This key's checked source-freeze identity.
    pub freeze: [u8; 32],
    /// SHA-256 of the exact complete submitted bytes.
    pub digest: [u8; 32],
    /// Exact bytes retained before the executor receives them.
    pub submitted: Vec<u8>,
    /// Expected boundary, redundantly bound to the reviewed schema.
    pub boundary: CaptureBoundary,
    /// Expected target outcome, never supplied to the executor.
    pub expected: ObservedOutcomeLayer,
    /// True only for a public sealed-construction control.
    pub sealed_control: bool,
}

/// Append-only run manifest; final digest is recorded after its last row is flushed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureManifest {
    /// Internal format version; this is no maintained interchange contract.
    pub schema: u32,
    /// Independently dispatched target.
    pub target: CaptureTarget,
    /// Rows in submission order, including explicitly untrusted negatives.
    pub rows: Vec<CaptureRow>,
}

/// Canonical test-local JSON identity of a completed manifest.
/// # Errors
/// Preserves the serializer's refusal without inventing an identity.
pub fn manifest_digest(manifest: &CaptureManifest) -> Result<[u8; 32], serde_json::Error> {
    serde_json::to_vec(manifest).map(|bytes| sha256(&bytes))
}

/// Capture evidence refusal. None is itself a covenant refusal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CaptureRefusal {
    /// The caller supplied a different internal manifest version.
    Version,
    /// The manifest names a different dispatched target.
    Target,
    /// The completed manifest differs from its retained provenance digest.
    ManifestDigest,
    /// A reviewed (schema,key) row is absent.
    MissingSchema,
    /// A (schema,key) identity appears twice.
    DuplicateSchema,
    /// A row is outside the reviewed target census.
    Schema,
    /// A row names neither representative key.
    Key,
    /// The source-freeze inventory refuses or disagrees.
    Freeze,
    /// The expected boundary differs from the reviewed schema.
    Boundary,
    /// The expected outcome differs from the reviewed schema.
    ExpectedOutcome,
    /// A negative is presented as sealed, or a control as unsealed.
    Origin,
    /// Retained bytes do not reproduce the row's digest.
    SubmittedDigest,
    /// Two rows name the same submitted bytes.
    DuplicateDigest,
    /// A submission has no manifest row.
    ObservationWithoutRow,
    /// A manifest row has no submission/response pair.
    RowWithoutObservation,
    /// A submission is observed more than once.
    DuplicateObservation,
    /// The executor's target layer differs from the expected one.
    ObservedOutcome,
    /// Mined bytes differ from the exact submitted bytes.
    ReadbackBytes,
    /// The bridge's provenance owner refused; its condition is retained.
    Provenance(ProvenanceRefusal),
    /// The bridge's journal owner refused; its condition is retained.
    Journal(JournalRefusal),
}

impl CaptureRefusal {
    /// Exhaustive condition inventory at the capture-admission boundary.
    #[must_use]
    pub const fn refusal_kind(&self) -> &'static str {
        match self {
            Self::Version => "version",
            Self::Target => "target",
            Self::ManifestDigest => "manifest_digest",
            Self::MissingSchema => "missing_schema",
            Self::DuplicateSchema => "duplicate_schema",
            Self::Schema => "schema",
            Self::Key => "key",
            Self::Freeze => "freeze",
            Self::Boundary => "boundary",
            Self::ExpectedOutcome => "expected_outcome",
            Self::Origin => "origin",
            Self::SubmittedDigest => "submitted_digest",
            Self::DuplicateDigest => "duplicate_digest",
            Self::ObservationWithoutRow => "observation_without_row",
            Self::RowWithoutObservation => "row_without_observation",
            Self::DuplicateObservation => "duplicate_observation",
            Self::ObservedOutcome => "observed_outcome",
            Self::ReadbackBytes => "readback_bytes",
            Self::Provenance(_) => "provenance",
            Self::Journal(_) => "journal",
        }
    }
}

/// Bind byte-bound capture joins.
///
/// Full offline admission also replays construction
/// from the retained funding and signing responses; this join alone proves no
/// sealed-construction origin, script execution, chain truth or native provenance.
///
/// The manifest is closed after construction; its digest is a run-provenance
/// coordinate, not a value guessed before funding. Build/executor provenance is
/// separately checked before starting a session. All raw observations are retained.
/// # Errors
/// Preserves provenance and journal refusals before validating manifest and joins.
pub fn bind_capture_manifest(
    target: CaptureTarget,
    expected: &CaptureProvenance,
    observed: &CaptureProvenance,
    handshake: &ExecutorHandshake,
    manifest: &CaptureManifest,
    retained_manifest_digest: [u8; 32],
    exchanges: &[(NativeOperationRequest, NativeOperationResponse)],
) -> Result<(), CaptureRefusal> {
    let _span =
        tracing::debug_span!(target: "vectors::keyed_final_capture", "bind_capture_manifest")
            .entered();
    let result = (|| {
        bind_capture_provenance(expected, observed).map_err(CaptureRefusal::Provenance)?;
        bind_capture_journal(observed, handshake, exchanges).map_err(CaptureRefusal::Journal)?;
        check_manifest(target, manifest, retained_manifest_digest)?;
        check_joins(manifest, exchanges)
    })();
    match &result {
        Ok(()) => {
            tracing::debug!(target: "vectors::keyed_final_capture", "capture admission completed");
        }
        Err(error) => {
            tracing::debug!(target: "vectors::keyed_final_capture", refusal_kind=error.refusal_kind(), "capture admission refused");
        }
    }
    result
}

fn check_manifest(
    target: CaptureTarget,
    manifest: &CaptureManifest,
    digest: [u8; 32],
) -> Result<(), CaptureRefusal> {
    use CaptureRefusal as R;
    if manifest.schema != 1 {
        return Err(R::Version);
    }
    if manifest.target != target {
        return Err(R::Target);
    }
    if manifest_digest(manifest).ok() != Some(digest) {
        return Err(R::ManifestDigest);
    }
    let reviewed = schemas(target);
    let keys = [[42; 32], crate::keyed_final_manifest::SECOND_KEY];
    let mut identities = Vec::new();
    let mut digests = Vec::new();
    for row in &manifest.rows {
        if !reviewed.contains(&row.schema) {
            return Err(R::Schema);
        }
        if !keys.contains(&row.key) {
            return Err(R::Key);
        }
        if identities.contains(&(row.schema, row.key)) {
            return Err(R::DuplicateSchema);
        }
        identities.push((row.schema, row.key));
        if capture_freeze_digest(row.key).ok() != Some(row.freeze) {
            return Err(R::Freeze);
        }
        if row.boundary != row.schema.boundary() {
            return Err(R::Boundary);
        }
        if row.expected != row.schema.outcome() {
            return Err(R::ExpectedOutcome);
        }
        if row.sealed_control
            != (row.boundary == CaptureBoundary::Acceptance
                && !matches!(
                    row.schema,
                    CaptureSchema::LaterParent | CaptureSchema::KeyPathControl
                ))
        {
            return Err(R::Origin);
        }
        if sha256(&row.submitted) != row.digest {
            return Err(R::SubmittedDigest);
        }
        if digests.contains(&row.digest) {
            return Err(R::DuplicateDigest);
        }
        digests.push(row.digest);
    }
    if keys.iter().any(|key| {
        reviewed
            .iter()
            .any(|schema| !identities.contains(&(*schema, *key)))
    }) {
        return Err(R::MissingSchema);
    }
    Ok(())
}

fn check_joins(
    manifest: &CaptureManifest,
    exchanges: &[(NativeOperationRequest, NativeOperationResponse)],
) -> Result<(), CaptureRefusal> {
    use CaptureRefusal as R;
    let mut seen = Vec::new();
    for (index, (request, response)) in exchanges.iter().enumerate() {
        let OperationSubject::Submission(subject) = &request.subject else {
            continue;
        };
        let digest = sha256(&subject.transaction_bytes);
        let row = manifest
            .rows
            .iter()
            .find(|row| row.digest == digest)
            .ok_or(R::ObservationWithoutRow)?;
        if seen.contains(&digest) {
            return Err(R::DuplicateObservation);
        }
        seen.push(digest);
        if response.observed_layer != row.expected {
            return Err(R::ObservedOutcome);
        }
        if response.observed_layer == ObservedOutcomeLayer::Accepted {
            let readback = response.mined_readback.as_ref().ok_or(R::Journal(JournalRefusal::Shape {
                index,
                refusal: target_elements_conformance::protocol::ResponseShapeDefect::AcceptedOperationOmitsObservation,
            }))?;
            if readback.raw_transaction != row.submitted {
                return Err(R::ReadbackBytes);
            }
        }
    }
    if seen.len() != manifest.rows.len() {
        return Err(R::RowWithoutObservation);
    }
    Ok(())
}
