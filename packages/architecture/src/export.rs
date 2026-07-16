//! Serde export model for the typed architecture manifest.
//!
//! The typed Rust architecture remains normative. These owned DTOs are
//! deterministic derivative representations used for the two published
//! artifacts, `architecture.json` and `architecture.toml`.
//!
//! Field order matters only for readability; all semantically set-like
//! collections are sorted during conversion so the export is
//! deterministic regardless of declaration order in `spec.rs`.

use serde::{Deserialize, Serialize};

use crate::canonical::{
    BEHAVIOURAL_HASH_ALGORITHM, SEMANTIC_HASH_ALGORITHM, export_behavioural_hash_hex,
    export_body_hash_hex, hex,
};
use crate::ids::{
    AllocatorId, BoundId, DeallocatorId, InputAuthorization, OpenFlowKind, OperationId,
    PermissionClass, PublicationStatus, QuantityId, ReaderId, ValueFlowClass, WitnessId,
};
use crate::spec::{
    Architecture, CanonicalDeltaSpec, DataOutputSpec, InputSpec, MaxCount, OperationSpec,
    OutputSpec, QuantitySpec, owner_bearing,
};

/// One row of the generated input-authorization evidence table
/// `[tbl:manifest:input-authorization-evidence]`: the evidence class
/// backing an input-authorization mode at each assurance layer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorizationEvidenceExport {
    pub input_authorization: String,
    pub model_evidence: String,
    pub compiler_evidence: String,
    pub deployment_evidence: String,
}

/// One row of the generated operation-authorization evidence table
/// `[tbl:manifest:operation-authorization-evidence]`.
///
/// The evidence class backing an operation-level permission class at
/// each assurance layer. Input participation and operation
/// authorization are distinct claims and are exported separately.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationAuthorizationEvidenceExport {
    pub permission_class: String,
    pub model_evidence: String,
    pub compiler_evidence: String,
    pub deployment_evidence: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishedArchitecture {
    /// Publication status is envelope metadata: a draft-to-final
    /// transition must not change the semantic architecture hash when
    /// architecture semantics are unchanged.
    pub publication_status: String,

    /// Schema 17 lifts the version fields out of the hashed body and
    /// into the envelope: a schema or version movement must not move
    /// either hash, so neither field may be a hash input.
    pub architecture_schema_version: u32,
    pub realization_version: String,

    pub semantic_hash_algorithm: String,
    pub semantic_hash: String,

    /// The behavioural hash covers the behavioural arrays only
    /// (dependencies, decisions, evidence tables, and envelope
    /// excluded). The denotation gate pins it: it moves only as a
    /// declared, recorded denotation change.
    pub behavioural_hash_algorithm: String,
    pub behavioural_hash: String,

    pub architecture: ArchitectureExport,
}

impl PublishedArchitecture {
    pub fn from_architecture(architecture: &Architecture) -> Result<Self, serde_json::Error> {
        let body = ArchitectureExport::from_architecture(architecture);

        Ok(Self {
            publication_status: architecture.document.status.as_str().to_owned(),
            architecture_schema_version: architecture.document.architecture_schema_version,
            realization_version: architecture.document.realization_version.to_owned(),
            semantic_hash_algorithm: SEMANTIC_HASH_ALGORITHM.to_owned(),
            semantic_hash: export_body_hash_hex(&body)?,
            behavioural_hash_algorithm: BEHAVIOURAL_HASH_ALGORITHM.to_owned(),
            behavioural_hash: export_behavioural_hash_hex(&body)?,
            architecture: body,
        })
    }

    /// Verifies the embedded semantic hash against the canonical JSON
    /// body only. Envelope metadata (publication status, versions,
    /// declared hash algorithms) is intentionally excluded from the
    /// hash input; use [`Self::validate_envelope`] when ingesting a
    /// published artifact.
    pub fn verify_body_hash(&self) -> Result<bool, serde_json::Error> {
        Ok(export_body_hash_hex(&self.architecture)? == self.semantic_hash)
    }

    /// Verifies the embedded behavioural hash against the behavioural
    /// arrays of the body.
    pub fn verify_behavioural_hash(&self) -> Result<bool, serde_json::Error> {
        Ok(export_behavioural_hash_hex(&self.architecture)? == self.behavioural_hash)
    }

    /// Canonical presentation rendering of the JSON artifact
    /// (`architecture.json`): pretty-printed, no trailing newline.
    /// The generator, the stale-artifact checker, and the conformance
    /// tests all compare against these exact bytes.
    ///
    /// # Errors
    ///
    /// Returns the underlying serialization error.
    pub fn to_artifact_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Canonical presentation rendering of the TOML artifact
    /// (`architecture.toml`). The same bytes are attached verbatim as
    /// the realization document's `app:realization:architecture` appendix.
    ///
    /// # Errors
    ///
    /// Returns the underlying serialization error.
    pub fn to_artifact_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Hash-envelope verification only: both declared hash algorithms
    /// must be the supported algorithms and both digests must verify
    /// over the body. Deliberately says nothing about schema, status,
    /// or version fields — those are envelope metadata outside the
    /// hashes, checked by [`Self::validate_envelope`].
    ///
    /// # Errors
    ///
    /// Returns the first hash-envelope failure.
    pub fn verify_hashes(&self) -> Result<(), EnvelopeError> {
        if self.semantic_hash_algorithm != SEMANTIC_HASH_ALGORITHM {
            return Err(EnvelopeError::UnsupportedHashAlgorithm);
        }

        if !self.verify_body_hash()? {
            return Err(EnvelopeError::HashMismatch);
        }

        if self.behavioural_hash_algorithm != BEHAVIOURAL_HASH_ALGORITHM {
            return Err(EnvelopeError::UnsupportedBehaviouralHashAlgorithm);
        }

        if !self.verify_behavioural_hash()? {
            return Err(EnvelopeError::BehaviouralHashMismatch);
        }

        Ok(())
    }

    /// Full publication-envelope validation: the hash envelope
    /// verifies ([`Self::verify_hashes`]) **and** the metadata
    /// envelope carries supported values — the schema version this
    /// crate implements, a recognized publication status, and a
    /// nonblank realization version. Hash exclusion makes these fields
    /// caller-controlled, so ingestion must not accept arbitrary
    /// values merely because the digests verify. Artifact ingestion
    /// must call this, not merely [`Self::verify_body_hash`].
    ///
    /// This establishes **self-consistency of an untrusted envelope
    /// only**. Both hashes are unkeyed digests of the embedded body:
    /// anyone can produce a different — or semantically invalid — body
    /// and recompute matching hashes. Passing here does not establish
    /// that the body is a valid architecture, that its identifiers are
    /// recognized, or that it equals any trusted architecture. For
    /// those claims see [`Self::validate_against_expected`].
    ///
    /// # Errors
    ///
    /// Returns the first envelope failure.
    pub fn validate_envelope(&self) -> Result<(), EnvelopeError> {
        self.verify_hashes()?;

        if self.architecture_schema_version != crate::spec::ARCHITECTURE_SCHEMA_VERSION {
            return Err(EnvelopeError::UnsupportedSchemaVersion(
                self.architecture_schema_version,
            ));
        }

        if !PublicationStatus::ALL
            .iter()
            .any(|status| status.as_str() == self.publication_status)
        {
            return Err(EnvelopeError::UnrecognizedPublicationStatus(
                self.publication_status.clone(),
            ));
        }

        if self.realization_version.trim().is_empty() {
            return Err(EnvelopeError::MissingRealizationVersion);
        }

        // The realization version is the tracked binding to the
        // compiler line: major.minor with a zero patch and an optional
        // prerelease marker. Nothing else enters this field.
        if !crate::spec::realization_version_well_formed(&self.realization_version) {
            return Err(EnvelopeError::MalformedRealizationVersion(
                self.realization_version.clone(),
            ));
        }

        Ok(())
    }

    /// Release-envelope validation: a supported envelope
    /// ([`Self::validate_envelope`]) whose publication status is
    /// `final`.
    ///
    /// Like [`Self::validate_envelope`], this is a **self-consistency
    /// check on untrusted input**, not a semantic or authenticity
    /// claim: a fabricated final artifact with recomputed hashes
    /// passes. Release tooling ingesting a published artifact must
    /// call this **and then** establish identity against an
    /// independently derived expected value —
    /// [`Self::validate_against_expected`] with
    /// `Self::from_architecture(&ARCHITECTURE)`, or a comparison of
    /// the semantic hash against an independently trusted expected
    /// hash — before treating the body as the architecture.
    ///
    /// # Errors
    ///
    /// Returns the first envelope failure, or
    /// [`EnvelopeError::NotFinal`] for a non-final artifact.
    pub fn validate_release_envelope(&self) -> Result<(), EnvelopeError> {
        self.validate_envelope()?;

        if self.publication_status != PublicationStatus::Final.as_str() {
            return Err(EnvelopeError::NotFinal);
        }

        Ok(())
    }

    /// Trusted-identity validation: a supported envelope
    /// ([`Self::validate_envelope`]) whose complete value — body and
    /// envelope metadata — equals `expected`, an independently derived
    /// publication (normally `Self::from_architecture(&ARCHITECTURE)`
    /// in first-party tooling). This is the step that turns envelope
    /// self-consistency into an authenticity claim; unkeyed hashes
    /// alone cannot.
    ///
    /// # Errors
    ///
    /// Returns the first envelope failure, or
    /// [`EnvelopeError::UnexpectedPublication`] when any field differs
    /// from `expected`.
    pub fn validate_against_expected(&self, expected: &Self) -> Result<(), EnvelopeError> {
        self.validate_envelope()?;

        if self != expected {
            return Err(EnvelopeError::UnexpectedPublication);
        }

        Ok(())
    }
}

/// Publication-envelope validation failure.
#[derive(Debug)]
pub enum EnvelopeError {
    UnsupportedHashAlgorithm,
    HashMismatch,
    UnsupportedBehaviouralHashAlgorithm,
    BehaviouralHashMismatch,
    UnsupportedSchemaVersion(u32),
    UnrecognizedPublicationStatus(String),
    MissingRealizationVersion,
    MalformedRealizationVersion(String),
    NotFinal,
    UnexpectedPublication,
    Serialization(serde_json::Error),
}

impl From<serde_json::Error> for EnvelopeError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error)
    }
}

impl core::fmt::Display for EnvelopeError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnsupportedHashAlgorithm => {
                formatter.write_str("unsupported semantic-hash algorithm identifier")
            }
            Self::HashMismatch => formatter.write_str("embedded semantic hash failed verification"),
            Self::UnsupportedBehaviouralHashAlgorithm => {
                formatter.write_str("unsupported behavioural-hash algorithm identifier")
            }
            Self::BehaviouralHashMismatch => {
                formatter.write_str("embedded behavioural hash failed verification")
            }
            Self::UnsupportedSchemaVersion(version) => {
                write!(
                    formatter,
                    "unsupported architecture schema version {version}"
                )
            }
            Self::UnrecognizedPublicationStatus(status) => {
                write!(formatter, "unrecognized publication status {status:?}")
            }
            Self::MissingRealizationVersion => formatter.write_str("realization version is empty"),
            Self::MalformedRealizationVersion(version) => {
                write!(
                    formatter,
                    "realization version {version:?} is not a tracked major.minor.0 binding"
                )
            }
            Self::NotFinal => formatter.write_str("publication status is not final"),
            Self::UnexpectedPublication => formatter
                .write_str("publication does not equal the independently derived expected value"),
            Self::Serialization(error) => write!(formatter, "serialization failure: {error}"),
        }
    }
}

impl std::error::Error for EnvelopeError {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureExport {
    pub target_network: String,

    pub specification: SpecificationExport,

    pub assets: Vec<AssetExport>,
    pub roots: Vec<RootExport>,
    pub objects: Vec<ObjectExport>,
    pub operations: Vec<OperationExport>,
    pub quantities: Vec<QuantityExport>,
    pub witnesses: Vec<WitnessExport>,
    pub clauses: Vec<ClauseExport>,
    pub dependencies: Vec<DependencyExport>,
    pub decisions: Vec<DecisionExport>,
    pub bounds: Vec<BoundExport>,
    pub amount_limits: Vec<AmountLimitExport>,
    pub tags: Vec<TagExport>,
    pub input_authorization_evidence: Vec<AuthorizationEvidenceExport>,
    pub operation_authorization_evidence: Vec<OperationAuthorizationEvidenceExport>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecificationExport {
    pub version: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor_set_hash: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetExport {
    pub code: u16,
    pub id: String,
    pub class: String,
    pub role: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_amount: Option<u64>,

    pub reissuable: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_operation: Option<String>,

    pub destruction_operations: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RootExport {
    pub code: u16,
    pub id: String,
    pub asset: String,
    pub role: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_amount: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RootUseExport {
    pub root: String,
    pub use_kind: String,
}

// Serde does not enforce `deny_unknown_fields` on internally tagged
// enums: unknown (and explicitly null) variant fields are buffered and
// discarded, so a mutated publication would still verify its body hash
// and compare equal to the typed expected value. Every tagged enum in
// this module therefore deserializes through a `*Repr` struct that
// rejects unknown fields and explicit `null`s, while serialization
// stays derived so canonical bytes are unchanged.

/// One optional variant field distinguishing three wire states:
/// absent, explicitly `null`, and a value. Serde derives cannot make
/// this distinction with `Option<T>` alone, and a discarded `null`
/// would survive body-hash verification exactly like an unknown field.
#[derive(Default)]
enum VariantField<T> {
    #[default]
    Absent,
    Null,
    Value(T),
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for VariantField<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(match Option::<T>::deserialize(deserializer)? {
            Some(value) => Self::Value(value),
            None => Self::Null,
        })
    }
}

impl<T> VariantField<T> {
    fn required(self, container: &str, kind: &str, field: &str) -> Result<T, String> {
        match self {
            Self::Value(value) => Ok(value),
            Self::Absent | Self::Null => Err(format!(
                "{container} kind {kind:?} requires field {field:?}"
            )),
        }
    }

    fn forbid(&self, container: &str, kind: &str, field: &str) -> Result<(), String> {
        match self {
            Self::Absent => Ok(()),
            Self::Null | Self::Value(_) => Err(format!(
                "{container} kind {kind:?} does not accept field {field:?}"
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    try_from = "AllocatorExportRepr"
)]
pub enum AllocatorExport {
    Genesis,
    External,
    Operation { operation: String },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AllocatorExportRepr {
    kind: String,
    #[serde(default)]
    operation: VariantField<String>,
}

impl TryFrom<AllocatorExportRepr> for AllocatorExport {
    type Error = String;

    fn try_from(repr: AllocatorExportRepr) -> Result<Self, Self::Error> {
        match repr.kind.as_str() {
            "genesis" => {
                repr.operation.forbid("allocator", "genesis", "operation")?;
                Ok(Self::Genesis)
            }
            "external" => {
                repr.operation
                    .forbid("allocator", "external", "operation")?;
                Ok(Self::External)
            }
            "operation" => Ok(Self::Operation {
                operation: repr
                    .operation
                    .required("allocator", "operation", "operation")?,
            }),
            other => Err(format!("unknown allocator kind {other:?}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    try_from = "DeallocatorExportRepr"
)]
pub enum DeallocatorExport {
    None,
    ExternalSpend,
    Operation { operation: String },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeallocatorExportRepr {
    kind: String,
    #[serde(default)]
    operation: VariantField<String>,
}

impl TryFrom<DeallocatorExportRepr> for DeallocatorExport {
    type Error = String;

    fn try_from(repr: DeallocatorExportRepr) -> Result<Self, Self::Error> {
        match repr.kind.as_str() {
            "none" => {
                repr.operation.forbid("deallocator", "none", "operation")?;
                Ok(Self::None)
            }
            "external-spend" => {
                repr.operation
                    .forbid("deallocator", "external-spend", "operation")?;
                Ok(Self::ExternalSpend)
            }
            "operation" => Ok(Self::Operation {
                operation: repr
                    .operation
                    .required("deallocator", "operation", "operation")?,
            }),
            other => Err(format!("unknown deallocator kind {other:?}")),
        }
    }
}

/// One derived recognition/authorization path of an object: either a
/// declared operation input consuming the object, or an external
/// wallet-level spend.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    try_from = "ObjectAuthorizationExportRepr"
)]
pub enum ObjectAuthorizationExport {
    Operation {
        operation: String,
        authorization: String,
    },

    ExternalSpend {
        authorization: String,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectAuthorizationExportRepr {
    kind: String,
    #[serde(default)]
    operation: VariantField<String>,
    #[serde(default)]
    authorization: VariantField<String>,
}

impl TryFrom<ObjectAuthorizationExportRepr> for ObjectAuthorizationExport {
    type Error = String;

    fn try_from(repr: ObjectAuthorizationExportRepr) -> Result<Self, Self::Error> {
        const CONTAINER: &str = "authorization path";
        match repr.kind.as_str() {
            "operation" => Ok(Self::Operation {
                operation: repr
                    .operation
                    .required(CONTAINER, "operation", "operation")?,
                authorization: repr.authorization.required(
                    CONTAINER,
                    "operation",
                    "authorization",
                )?,
            }),
            "external-spend" => {
                repr.operation
                    .forbid(CONTAINER, "external-spend", "operation")?;
                Ok(Self::ExternalSpend {
                    authorization: repr.authorization.required(
                        CONTAINER,
                        "external-spend",
                        "authorization",
                    )?,
                })
            }
            other => Err(format!("unknown {CONTAINER} kind {other:?}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectExport {
    pub code: u16,
    pub id: String,
    pub asset: String,
    pub lifecycle: String,
    pub accounting_domain: String,

    pub allocators: Vec<AllocatorExport>,
    pub mutators: Vec<String>,
    pub deallocators: Vec<DeallocatorExport>,

    /// Derived from operation input declarations; not stored
    /// separately in the typed manifest.
    pub authorization_paths: Vec<ObjectAuthorizationExport>,

    pub witnesses: Vec<String>,
    pub consensus_value_authoritative: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IssuanceExport {
    pub asset: String,
    pub authority: String,
    pub condition: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    try_from = "MaximumExportRepr"
)]
pub enum MaximumExport {
    Exact { value: u16 },
    Bound { bound: String },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MaximumExportRepr {
    kind: String,
    #[serde(default)]
    value: VariantField<u16>,
    #[serde(default)]
    bound: VariantField<String>,
}

impl TryFrom<MaximumExportRepr> for MaximumExport {
    type Error = String;

    fn try_from(repr: MaximumExportRepr) -> Result<Self, Self::Error> {
        match repr.kind.as_str() {
            "exact" => {
                repr.bound.forbid("maximum", "exact", "bound")?;
                Ok(Self::Exact {
                    value: repr.value.required("maximum", "exact", "value")?,
                })
            }
            "bound" => {
                repr.value.forbid("maximum", "bound", "value")?;
                Ok(Self::Bound {
                    bound: repr.bound.required("maximum", "bound", "bound")?,
                })
            }
            other => Err(format!("unknown maximum kind {other:?}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputExport {
    pub object: String,
    pub minimum: u16,
    pub authorization: String,
    pub maximum: MaximumExport,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputExport {
    pub object: String,
    pub minimum: u16,
    pub maximum: MaximumExport,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalDeltaExport {
    pub asset: String,
    pub kind: String,
    pub condition: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destruction_tag: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataOutputExport {
    pub kind: String,
    pub tag: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,

    pub minimum: u16,
    pub condition: String,
    pub maximum: MaximumExport,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionExport {
    pub projection: String,
    pub rule: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationExport {
    pub code: u16,
    pub id: String,
    pub kind: String,
    pub authorization: String,

    pub open_flows: Vec<String>,
    pub reads: Vec<String>,
    pub writes: Vec<String>,
    pub witnesses: Vec<String>,
    pub value_flows: Vec<String>,
    pub bounds: Vec<String>,

    pub roots: Vec<RootUseExport>,
    pub issuances: Vec<IssuanceExport>,
    pub inputs: Vec<InputExport>,
    pub outputs: Vec<OutputExport>,
    pub canonical_deltas: Vec<CanonicalDeltaExport>,
    pub data_outputs: Vec<DataOutputExport>,
    pub projections: Vec<ProjectionExport>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", try_from = "ReaderExportRepr")]
pub enum ReaderExport {
    Operation { operation: String },
    AttestationIndexer,
    InvariantChecker,
    ExternalAuditor,
    ConsumerFormula,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReaderExportRepr {
    kind: String,
    #[serde(default)]
    operation: VariantField<String>,
}

impl TryFrom<ReaderExportRepr> for ReaderExport {
    type Error = String;

    fn try_from(repr: ReaderExportRepr) -> Result<Self, Self::Error> {
        if repr.kind == "operation" {
            return Ok(Self::Operation {
                operation: repr
                    .operation
                    .required("reader", "operation", "operation")?,
            });
        }
        repr.operation.forbid("reader", &repr.kind, "operation")?;
        match repr.kind.as_str() {
            "attestation-indexer" => Ok(Self::AttestationIndexer),
            "invariant-checker" => Ok(Self::InvariantChecker),
            "external-auditor" => Ok(Self::ExternalAuditor),
            "consumer-formula" => Ok(Self::ConsumerFormula),
            other => Err(format!("unknown reader kind {other:?}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuantityExport {
    pub code: u16,
    pub id: String,
    pub kind: String,
    pub reads: Vec<String>,
    pub writers: Vec<String>,
    pub readers: Vec<ReaderExport>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WitnessExport {
    pub code: u16,
    pub id: String,
    pub semantic_tag: String,
}

/// One row of the generated invariant-clause registry. The `id` is
/// the realization document's frozen clause label; the document's
/// clause table is generated from this array, not re-typed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClauseExport {
    pub code: u16,
    pub id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyExport {
    pub code: u16,
    pub id: String,
    pub verification_required: bool,
    pub rationale: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionExport {
    pub code: u16,
    pub id: String,
    pub status: String,
    pub rationale: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoundExport {
    pub code: u16,
    pub id: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<u64>,

    pub requires_deployment_calibration: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AmountLimitExport {
    pub code: u16,
    pub id: String,
    pub asset: String,
    pub value: u64,
    pub unit: String,
    pub rationale: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TagExport {
    pub code: u16,
    pub id: String,
    pub participates_in_attestation: bool,
}

impl ArchitectureExport {
    pub fn from_architecture(architecture: &Architecture) -> Self {
        let mut assets = architecture
            .assets
            .iter()
            .map(|asset| AssetExport {
                code: asset.id.code(),
                id: asset.id.as_str().to_owned(),
                class: asset.class.as_str().to_owned(),
                role: asset.role.as_str().to_owned(),
                fixed_amount: asset.fixed_amount,
                reissuable: asset.reissuable,
                authority: asset
                    .authority
                    .map(|authority| authority.as_str().to_owned()),
                issue_operation: asset
                    .issue_operation
                    .map(|operation| operation.as_str().to_owned()),
                destruction_operations: sorted_ids(asset.destruction_operations, |operation| {
                    operation.as_str()
                }),
            })
            .collect::<Vec<_>>();

        assets.sort_by_key(|asset| asset.code);

        let mut roots = architecture
            .roots
            .iter()
            .map(|root| RootExport {
                code: root.id.code(),
                id: root.id.as_str().to_owned(),
                asset: root.asset.as_str().to_owned(),
                role: root.role.as_str().to_owned(),
                fixed_amount: root.fixed_amount,
            })
            .collect::<Vec<_>>();

        roots.sort_by_key(|root| root.code);

        let mut objects = architecture
            .objects
            .iter()
            .map(|object| object_export(architecture, object))
            .collect::<Vec<_>>();

        objects.sort_by_key(|object| object.code);

        let mut operations = architecture
            .operations
            .iter()
            .map(operation_export)
            .collect::<Vec<_>>();

        operations.sort_by_key(|operation| operation.code);

        let mut quantities = architecture
            .quantities
            .iter()
            .map(quantity_export)
            .collect::<Vec<_>>();

        quantities.sort_by_key(|quantity| quantity.code);

        let mut witnesses = architecture
            .witnesses
            .iter()
            .map(|witness| WitnessExport {
                code: witness.id.code(),
                id: witness.id.as_str().to_owned(),
                semantic_tag: witness.semantic_tag.to_owned(),
            })
            .collect::<Vec<_>>();

        witnesses.sort_by_key(|witness| witness.code);

        let mut clauses = architecture
            .clauses
            .iter()
            .map(|clause| ClauseExport {
                code: clause.code(),
                id: clause.as_str().to_owned(),
            })
            .collect::<Vec<_>>();

        clauses.sort_by_key(|clause| clause.code);

        let mut dependencies = architecture
            .dependencies
            .iter()
            .map(|dependency| DependencyExport {
                code: dependency.id.code(),
                id: dependency.id.as_str().to_owned(),
                verification_required: dependency.verification_required,
                rationale: dependency.rationale.to_owned(),
            })
            .collect::<Vec<_>>();

        dependencies.sort_by_key(|dependency| dependency.code);

        let mut decisions = architecture
            .decisions
            .iter()
            .map(|decision| {
                let mut rationale = decision
                    .rationale
                    .iter()
                    .map(|line| (*line).to_owned())
                    .collect::<Vec<_>>();

                rationale.sort();

                DecisionExport {
                    code: decision.id.code(),
                    id: decision.id.as_str().to_owned(),
                    status: decision.status.as_str().to_owned(),
                    rationale,
                }
            })
            .collect::<Vec<_>>();

        decisions.sort_by_key(|decision| decision.code);

        let mut bounds = architecture
            .bounds
            .iter()
            .map(|bound| BoundExport {
                code: bound.id.code(),
                id: bound.id.as_str().to_owned(),
                default_value: bound.default_value,
                requires_deployment_calibration: bound.requires_deployment_calibration,
            })
            .collect::<Vec<_>>();

        bounds.sort_by_key(|bound| bound.code);

        let mut amount_limits = architecture
            .amount_limits
            .iter()
            .map(|limit| AmountLimitExport {
                code: limit.id.code(),
                id: limit.id.as_str().to_owned(),
                asset: limit.asset.as_str().to_owned(),
                value: limit.value,
                unit: limit.unit.to_owned(),
                rationale: limit.rationale.to_owned(),
            })
            .collect::<Vec<_>>();

        amount_limits.sort_by_key(|limit| limit.code);

        let mut tags = architecture
            .tags
            .iter()
            .map(|tag| TagExport {
                code: tag.id.code(),
                id: tag.id.as_str().to_owned(),
                participates_in_attestation: tag.participates_in_attestation,
            })
            .collect::<Vec<_>>();

        tags.sort_by_key(|tag| tag.code);

        let input_authorization_evidence = InputAuthorization::ALL
            .iter()
            .map(|authorization| {
                let evidence = authorization.evidence();

                AuthorizationEvidenceExport {
                    input_authorization: authorization.as_str().to_owned(),
                    model_evidence: evidence.model.as_str().to_owned(),
                    compiler_evidence: evidence.compiler.as_str().to_owned(),
                    deployment_evidence: evidence.deployment.as_str().to_owned(),
                }
            })
            .collect::<Vec<_>>();

        let operation_authorization_evidence = PermissionClass::ALL
            .iter()
            .map(|class| {
                let evidence = class.evidence();

                OperationAuthorizationEvidenceExport {
                    permission_class: class.as_str().to_owned(),
                    model_evidence: evidence.model.as_str().to_owned(),
                    compiler_evidence: evidence.compiler.as_str().to_owned(),
                    deployment_evidence: evidence.deployment.as_str().to_owned(),
                }
            })
            .collect::<Vec<_>>();

        Self {
            target_network: architecture.document.target_network.to_owned(),

            specification: SpecificationExport {
                version: architecture.document.specification.version.to_owned(),
                anchor_set_hash: architecture
                    .document
                    .specification
                    .anchor_set_hash
                    .map(|hash| hex(&hash)),
            },

            assets,
            roots,
            objects,
            operations,
            quantities,
            witnesses,
            clauses,
            dependencies,
            decisions,
            bounds,
            amount_limits,
            tags,
            input_authorization_evidence,
            operation_authorization_evidence,
        }
    }
}

fn sorted_ids<T: Copy>(values: &[T], name: fn(T) -> &'static str) -> Vec<String> {
    let mut output = values
        .iter()
        .map(|value| name(*value).to_owned())
        .collect::<Vec<_>>();

    output.sort();
    output.dedup();
    output
}

fn object_export(architecture: &Architecture, object: &crate::spec::ObjectSpec) -> ObjectExport {
    let mut allocators = object
        .allocators
        .iter()
        .copied()
        .map(allocator_export)
        .collect::<Vec<_>>();

    allocators.sort();

    let mut deallocators = object
        .deallocators
        .iter()
        .copied()
        .map(deallocator_export)
        .collect::<Vec<_>>();

    deallocators.sort();

    ObjectExport {
        code: object.id.code(),
        id: object.id.as_str().to_owned(),
        asset: object.asset.as_str().to_owned(),
        lifecycle: object.lifecycle.as_str().to_owned(),
        accounting_domain: object.accounting_domain.as_str().to_owned(),

        allocators,
        mutators: sorted_ids(object.mutators, |operation| operation.as_str()),
        deallocators,

        authorization_paths: object_authorization_paths(architecture, object),
        witnesses: sorted_ids(object.witnesses, WitnessId::as_str),
        consensus_value_authoritative: object.consensus_value_authoritative,
    }
}

/// Derives the object's authorization paths from the operation input
/// declarations referencing it, plus an external-spend path for
/// externally spent objects.
fn object_authorization_paths(
    architecture: &Architecture,
    object: &crate::spec::ObjectSpec,
) -> Vec<ObjectAuthorizationExport> {
    let mut paths = Vec::new();

    for operation in architecture.operations {
        for input in operation.inputs {
            if input.object == object.id {
                paths.push(ObjectAuthorizationExport::Operation {
                    operation: operation.id.as_str().to_owned(),
                    authorization: input.authorization.as_str().to_owned(),
                });
            }
        }
    }

    if object.deallocators.contains(&DeallocatorId::ExternalSpend) {
        let authorization = if owner_bearing(object.id) {
            InputAuthorization::InputOwner
        } else {
            InputAuthorization::Permissionless
        };

        paths.push(ObjectAuthorizationExport::ExternalSpend {
            authorization: authorization.as_str().to_owned(),
        });
    }

    paths.sort();
    paths.dedup();
    paths
}

fn allocator_export(allocator: AllocatorId) -> AllocatorExport {
    match allocator {
        AllocatorId::Genesis => AllocatorExport::Genesis,
        AllocatorId::External => AllocatorExport::External,
        AllocatorId::Operation(operation) => AllocatorExport::Operation {
            operation: operation.as_str().to_owned(),
        },
    }
}

fn deallocator_export(deallocator: DeallocatorId) -> DeallocatorExport {
    match deallocator {
        DeallocatorId::None => DeallocatorExport::None,
        DeallocatorId::ExternalSpend => DeallocatorExport::ExternalSpend,
        DeallocatorId::Operation(operation) => DeallocatorExport::Operation {
            operation: operation.as_str().to_owned(),
        },
    }
}

fn operation_export(operation: &OperationSpec) -> OperationExport {
    let mut roots = operation
        .roots
        .iter()
        .map(|root| RootUseExport {
            root: root.root.as_str().to_owned(),
            use_kind: root.use_kind.as_str().to_owned(),
        })
        .collect::<Vec<_>>();

    roots.sort_by(|left, right| left.root.cmp(&right.root));

    let mut issuances = operation
        .issuances
        .iter()
        .map(|issuance| IssuanceExport {
            asset: issuance.asset.as_str().to_owned(),
            authority: issuance.authority.as_str().to_owned(),
            condition: issuance.condition.as_str().to_owned(),
        })
        .collect::<Vec<_>>();

    issuances.sort_by(|left, right| left.asset.cmp(&right.asset));

    let mut inputs = operation
        .inputs
        .iter()
        .copied()
        .map(input_export)
        .collect::<Vec<_>>();

    inputs.sort_by(|left, right| left.object.cmp(&right.object));

    let mut outputs = operation
        .outputs
        .iter()
        .copied()
        .map(output_export)
        .collect::<Vec<_>>();

    outputs.sort_by(|left, right| left.object.cmp(&right.object));

    let mut canonical_deltas = operation
        .canonical_deltas
        .iter()
        .copied()
        .map(canonical_delta_export)
        .collect::<Vec<_>>();

    canonical_deltas.sort_by(|left, right| {
        (&left.asset, &left.kind, &left.condition).cmp(&(
            &right.asset,
            &right.kind,
            &right.condition,
        ))
    });

    let mut data_outputs = operation
        .data_outputs
        .iter()
        .copied()
        .map(data_output_export)
        .collect::<Vec<_>>();

    data_outputs.sort_by(|left, right| {
        (&left.tag, &left.kind, &left.condition).cmp(&(&right.tag, &right.kind, &right.condition))
    });

    let mut projections = operation
        .projections
        .iter()
        .map(|projection| ProjectionExport {
            projection: projection.projection.as_str().to_owned(),
            rule: projection.rule.as_str().to_owned(),
        })
        .collect::<Vec<_>>();

    projections.sort_by(|left, right| left.projection.cmp(&right.projection));

    OperationExport {
        code: operation.id.code(),
        id: operation.id.as_str().to_owned(),
        kind: operation.kind.as_str().to_owned(),
        authorization: operation.authorization.as_str().to_owned(),

        open_flows: sorted_ids(operation.open_flows, OpenFlowKind::as_str),
        reads: sorted_ids(operation.reads, QuantityId::as_str),
        writes: sorted_ids(operation.writes, QuantityId::as_str),
        witnesses: sorted_ids(operation.witnesses, WitnessId::as_str),
        value_flows: sorted_ids(operation.value_flows, ValueFlowClass::as_str),
        bounds: sorted_ids(operation.bounds, BoundId::as_str),

        roots,
        issuances,
        inputs,
        outputs,
        canonical_deltas,
        data_outputs,
        projections,
    }
}

fn maximum_export(value: MaxCount) -> MaximumExport {
    match value {
        MaxCount::Exact(value) => MaximumExport::Exact { value },
        MaxCount::Bound(bound) => MaximumExport::Bound {
            bound: bound.as_str().to_owned(),
        },
    }
}

fn input_export(value: InputSpec) -> InputExport {
    InputExport {
        object: value.object.as_str().to_owned(),
        minimum: value.minimum,
        authorization: value.authorization.as_str().to_owned(),
        maximum: maximum_export(value.maximum),
    }
}

fn output_export(value: OutputSpec) -> OutputExport {
    OutputExport {
        object: value.object.as_str().to_owned(),
        minimum: value.minimum,
        maximum: maximum_export(value.maximum),
    }
}

fn canonical_delta_export(value: CanonicalDeltaSpec) -> CanonicalDeltaExport {
    CanonicalDeltaExport {
        asset: value.asset.as_str().to_owned(),
        kind: value.kind.as_str().to_owned(),
        condition: value.condition.as_str().to_owned(),
        destruction_tag: value.destruction_tag.map(|tag| tag.as_str().to_owned()),
    }
}

fn data_output_export(value: DataOutputSpec) -> DataOutputExport {
    DataOutputExport {
        kind: value.kind.as_str().to_owned(),
        tag: value.tag.as_str().to_owned(),
        asset: value.asset.map(|asset| asset.as_str().to_owned()),
        minimum: value.minimum,
        condition: value.condition.as_str().to_owned(),
        maximum: maximum_export(value.maximum),
    }
}

fn quantity_export(quantity: &QuantitySpec) -> QuantityExport {
    let mut reads = quantity
        .reads
        .iter()
        .map(|data| data.as_str().to_owned())
        .collect::<Vec<_>>();

    reads.sort();

    let mut readers = quantity
        .readers
        .iter()
        .copied()
        .map(reader_export)
        .collect::<Vec<_>>();

    readers.sort();

    QuantityExport {
        code: quantity.id.code(),
        id: quantity.id.as_str().to_owned(),
        kind: quantity.kind.as_str().to_owned(),
        reads,
        writers: sorted_ids(quantity.writers, OperationId::as_str),
        readers,
    }
}

fn reader_export(reader: ReaderId) -> ReaderExport {
    match reader {
        ReaderId::Operation(operation) => ReaderExport::Operation {
            operation: operation.as_str().to_owned(),
        },
        ReaderId::AttestationIndexer => ReaderExport::AttestationIndexer,
        ReaderId::InvariantChecker => ReaderExport::InvariantChecker,
        ReaderId::ExternalAuditor => ReaderExport::ExternalAuditor,
        ReaderId::ConsumerFormula => ReaderExport::ConsumerFormula,
    }
}
