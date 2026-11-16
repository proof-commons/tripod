//! Standalone fixed-form Keyed Provenance Covenant manifest. No attestation census is borrowed.
//!
//! The manifest binds paper law, not a deployment, key encoding, funding floor,
//! finite input bound, or finality depth. Its binding is not a deployment or realization claim.

use sha2::{Digest as _, Sha256};

pub mod vocabulary;
use crate::semantic::SemanticProfileId;
pub use vocabulary::{
    FixedParameterId, KeyedFixedClause, KeyedFixedObjectId, KeyedFixedOperationId,
};

pub const KEYED_FIXED_SCHEMA_VERSION: u32 = 3;

pub const KEYED_FIXED_ANCHORS: &[&str] = &[
    "def:kpcfixed-model:history",
    "def:kpcfixed-model:keyed-membership",
    "def:kpcfixed-model:first-output",
    "op:kpcfixed-operations:source",
    "op:kpcfixed-operations:merge",
    "def:kpcfixed-operations:classification",
    "def:kpcfixed-interface:membership-reading",
    "rem:kpcfixed-interface:record-evidence",
    "sec:kpcfixed:interface",
    "thm:kpcfixed-interface:membership-correctness",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedFixedDocumentBinding {
    pub paper: String,
    pub version: String,
    pub anchors: Vec<String>,
    pub anchor_set_hash: [u8; 32],
}

impl KeyedFixedDocumentBinding {
    /// Build the independent paper binding using the existing anchor-set recipe.
    ///
    /// # Errors
    /// Returns a malformed anchor error if the built-in anchor vocabulary is invalid.
    pub fn paper() -> Result<Self, crate::AnchorNameDefect> {
        let anchors = crate::ValidatedAnchorSet::new(KEYED_FIXED_ANCHORS.iter().copied())?;
        Ok(Self {
            paper: "papers/lineage-fixed-form/".to_owned(),
            version: "2.1.0".to_owned(),
            anchors: anchors
                .iter()
                .map(|name| name.as_str().to_owned())
                .collect(),
            anchor_set_hash: crate::anchor_set_hash(&anchors),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedFixedManifest {
    pub profile: SemanticProfileId,
    pub schema_version: u32,
    pub document: KeyedFixedDocumentBinding,
    pub parameters: Vec<FixedParameterId>,
    pub objects: Vec<KeyedFixedObjectId>,
    pub operations: Vec<KeyedFixedOperationId>,
    pub clauses: Vec<KeyedFixedClause>,
}

impl KeyedFixedManifest {
    /// The stable SOURCE/MERGE manifest declared by the fixed-form Keyed Provenance Covenant paper.
    pub fn contract() -> Result<Self, crate::AnchorNameDefect> {
        Ok(Self {
            profile: SemanticProfileId::KeyedFixedForm,
            schema_version: KEYED_FIXED_SCHEMA_VERSION,
            document: KeyedFixedDocumentBinding::paper()?,
            parameters: vec![
                FixedParameterId::Key,
                FixedParameterId::Asset,
                FixedParameterId::Value,
            ],
            objects: vec![KeyedFixedObjectId::Handle],
            operations: vec![KeyedFixedOperationId::Source, KeyedFixedOperationId::Merge],
            clauses: vec![
                KeyedFixedClause::BornFirst,
                KeyedFixedClause::OwnQualification,
                KeyedFixedClause::SuccessorQualification,
            ],
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyedFixedManifestError {
    ProfileBinding,
    SchemaVersion,
    PaperBinding,
    ParameterCensus,
    ObjectCensus,
    OperationCensus,
    ClauseCensus,
}

impl KeyedFixedManifestError {
    pub const fn kind(self) -> &'static str {
        match self {
            Self::ProfileBinding => "profile_binding",
            Self::SchemaVersion => "schema_version",
            Self::PaperBinding => "paper_binding",
            Self::ParameterCensus => "parameter_census",
            Self::ObjectCensus => "object_census",
            Self::OperationCensus => "operation_census",
            Self::ClauseCensus => "clause_census",
        }
    }
}

/// Only the architecture validation entry can construct this wrapper.
#[derive(Clone, Copy, Debug)]
pub struct ValidatedKeyedFixedManifest<'a>(pub(crate) &'a KeyedFixedManifest);

impl ValidatedKeyedFixedManifest<'_> {
    pub const fn manifest(&self) -> &KeyedFixedManifest {
        self.0
    }

    /// Canonical schema-3 bytes under ruling 209: domain, schema, profile,
    /// length-framed paper/version, anchor-set hash and sorted identifier rows.
    /// Tables are parameters, objects, operations and clauses, in that order.
    /// All numeric fields are big-endian; text lengths and row counts are u64.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = b"tripod keyed provenance fixed-form manifest schema 3\n".to_vec();
        bytes.extend(self.0.schema_version.to_be_bytes());
        bytes.extend((self.0.profile as u16).to_be_bytes());
        for text in [&self.0.document.paper, &self.0.document.version] {
            bytes.extend(u64::try_from(text.len()).unwrap_or(u64::MAX).to_be_bytes());
            bytes.extend(text.as_bytes());
        }
        bytes.extend(self.0.document.anchor_set_hash);
        let mut parameters = self.0.parameters.clone();
        parameters.sort();
        let mut objects = self.0.objects.clone();
        objects.sort();
        let mut operations = self.0.operations.clone();
        operations.sort();
        let mut clauses = self.0.clauses.clone();
        clauses.sort();
        for rows in [
            parameters
                .iter()
                .map(|id| [0, *id as u16])
                .collect::<Vec<_>>(),
            objects.iter().map(|id| id.encoding()).collect::<Vec<_>>(),
            operations.iter().map(|id| id.encoding()).collect(),
            clauses.iter().map(|clause| [0, *clause as u16]).collect(),
        ] {
            bytes.extend(u64::try_from(rows.len()).unwrap_or(u64::MAX).to_be_bytes());
            for row in rows {
                for code in row {
                    bytes.extend(code.to_be_bytes());
                }
            }
        }
        bytes
    }

    /// Independent keyed declaration binding, never an attestation publication identity.
    pub fn binding(&self) -> KeyedFixedArchitectureBinding {
        let span = tracing::debug_span!(target: "architecture::keyed_fixed", "keyed_fixed_binding");
        let _entered = span.enter();
        let bytes = self.canonical_bytes();
        let digest = Sha256::digest(&bytes).into();
        tracing::debug!(target: "architecture::keyed_fixed", outcome = "completed", byte_count = crate::trace_count(bytes.len()), "keyed binding computed");
        KeyedFixedArchitectureBinding {
            profile: self.0.profile,
            schema_version: self.0.schema_version,
            digest,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedFixedArchitectureBinding {
    pub profile: SemanticProfileId,
    pub schema_version: u32,
    pub digest: [u8; 32],
}
