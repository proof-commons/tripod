//! Standalone Keyed Provenance Covenant manifest. No attestation census is borrowed.
//!
//! The manifest binds paper law, not a deployment, key encoding, funding floor,
//! finite input bound, or finality depth. Its binding is not a deployment or realization claim.

use sha2::{Digest as _, Sha256};

use crate::semantic::{
    KeyedObjectId, KeyedOperationId, SemanticObjectId, SemanticOperationId, SemanticProfileId,
};

pub const KEYED_SCHEMA_VERSION: u32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum KeyedClause {
    BornFirst = 1,
    ContinueFirst = 2,
}

/// A symbolic parameter fixed at covenant generation, not a chosen key encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum KeyParameterId {
    CovenantKey = 1,
}

pub const KEYED_ANCHORS: &[&str] = &[
    "def:kpc-model:history",
    "def:kpc-model:keyed-membership",
    "def:kpc-model:first-output",
    "op:kpc-operations:source",
    "op:kpc-operations:merge",
    "def:kpc-operations:classification",
    "def:kpc-interface:membership-reading",
    "rem:kpc-interface:record-evidence",
    "sec:kpc:interface",
    "thm:kpc-interface:membership-correctness",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedDocumentBinding {
    pub paper: String,
    pub version: String,
    pub anchors: Vec<String>,
    pub anchor_set_hash: [u8; 32],
}

impl KeyedDocumentBinding {
    /// Build the independent paper binding using the existing anchor-set recipe.
    ///
    /// # Errors
    /// Returns a malformed anchor error if the built-in anchor vocabulary is invalid.
    pub fn paper() -> Result<Self, crate::AnchorNameDefect> {
        let anchors = crate::ValidatedAnchorSet::new(KEYED_ANCHORS.iter().copied())?;
        Ok(Self {
            paper: "papers/lineage/".to_owned(),
            version: "1.0.0".to_owned(),
            anchors: anchors
                .iter()
                .map(|name| name.as_str().to_owned())
                .collect(),
            anchor_set_hash: crate::anchor_set_hash(&anchors),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedManifest {
    pub profile: SemanticProfileId,
    pub schema_version: u32,
    pub document: KeyedDocumentBinding,
    pub key: KeyParameterId,
    pub objects: Vec<SemanticObjectId>,
    pub operations: Vec<SemanticOperationId>,
    pub clauses: Vec<KeyedClause>,
}

impl KeyedManifest {
    /// The stable SOURCE/MERGE manifest declared by the Keyed Provenance Covenant paper.
    pub fn contract() -> Result<Self, crate::AnchorNameDefect> {
        Ok(Self {
            profile: SemanticProfileId::KeyedProvenance,
            schema_version: KEYED_SCHEMA_VERSION,
            document: KeyedDocumentBinding::paper()?,
            key: KeyParameterId::CovenantKey,
            objects: vec![SemanticObjectId::Keyed(KeyedObjectId::Handle)],
            operations: vec![
                SemanticOperationId::Keyed(KeyedOperationId::Source),
                SemanticOperationId::Keyed(KeyedOperationId::Merge),
            ],
            clauses: vec![KeyedClause::BornFirst, KeyedClause::ContinueFirst],
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyedManifestError {
    ProfileBinding,
    SchemaVersion,
    PaperBinding,
    ObjectCensus,
    OperationCensus,
    ClauseCensus,
}

impl KeyedManifestError {
    pub const fn kind(self) -> &'static str {
        match self {
            Self::ProfileBinding => "profile_binding",
            Self::SchemaVersion => "schema_version",
            Self::PaperBinding => "paper_binding",
            Self::ObjectCensus => "object_census",
            Self::OperationCensus => "operation_census",
            Self::ClauseCensus => "clause_census",
        }
    }
}

/// Only the architecture validation entry can construct this wrapper.
#[derive(Clone, Copy, Debug)]
pub struct ValidatedKeyedManifest<'a>(pub(crate) &'a KeyedManifest);

impl ValidatedKeyedManifest<'_> {
    pub const fn manifest(&self) -> &KeyedManifest {
        self.0
    }

    /// Canonical version-2 bytes under ruling 173: domain, schema, profile, key,
    /// length-framed paper/version, anchor-set hash and sorted identifier rows.
    /// All numeric fields are big-endian; text lengths and row counts are u64.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = b"tripod keyed provenance covenant manifest v2\n".to_vec();
        bytes.extend(self.0.schema_version.to_be_bytes());
        bytes.extend((self.0.profile as u16).to_be_bytes());
        bytes.extend((self.0.key as u16).to_be_bytes());
        for text in [&self.0.document.paper, &self.0.document.version] {
            bytes.extend(u64::try_from(text.len()).unwrap_or(u64::MAX).to_be_bytes());
            bytes.extend(text.as_bytes());
        }
        bytes.extend(self.0.document.anchor_set_hash);
        let mut objects = self.0.objects.clone();
        objects.sort();
        let mut operations = self.0.operations.clone();
        operations.sort();
        let mut clauses = self.0.clauses.clone();
        clauses.sort();
        for rows in [
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
    pub fn binding(&self) -> KeyedArchitectureBinding {
        let span = tracing::debug_span!(target: "architecture::keyed", "keyed_binding");
        let _entered = span.enter();
        let bytes = self.canonical_bytes();
        let digest = Sha256::digest(&bytes).into();
        tracing::debug!(target: "architecture::keyed", outcome = "completed", byte_count = crate::trace_count(bytes.len()), "keyed binding computed");
        KeyedArchitectureBinding {
            profile: self.0.profile,
            schema_version: self.0.schema_version,
            digest,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedArchitectureBinding {
    pub profile: SemanticProfileId,
    pub schema_version: u32,
    pub digest: [u8; 32],
}
