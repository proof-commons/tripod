//! Semantic namespaces for independently owned contract declarations.
//!
//! These identifiers do not extend the attestation enums or their ALL censuses.
//! Encoding arms are vocabulary, never a selection of contract policy.

use crate::{ObjectId, OperationId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum SemanticProfileId {
    Attestation = 1,
    KeyedProvenance = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum KeyedOperationId {
    Source = 1,
    Merge = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum KeyedObjectId {
    Handle = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticOperationId {
    Attestation(OperationId),
    Keyed(KeyedOperationId),
}

impl SemanticOperationId {
    /// Namespace and local code; neither component may be discarded.
    pub const fn encoding(self) -> [u16; 2] {
        match self {
            Self::Attestation(id) => [SemanticProfileId::Attestation as u16, id.code()],
            Self::Keyed(id) => [SemanticProfileId::KeyedProvenance as u16, id as u16],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticObjectId {
    Attestation(ObjectId),
    Keyed(KeyedObjectId),
}

impl SemanticObjectId {
    pub const fn encoding(self) -> [u16; 2] {
        match self {
            Self::Attestation(id) => [SemanticProfileId::Attestation as u16, id.code()],
            Self::Keyed(id) => [SemanticProfileId::KeyedProvenance as u16, id as u16],
        }
    }
}
