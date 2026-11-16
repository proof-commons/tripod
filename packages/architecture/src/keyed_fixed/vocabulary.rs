//! Fixed-form local identifiers. The profile namespace has one architecture owner.

use crate::semantic::SemanticProfileId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum KeyedFixedOperationId {
    Source = 1,
    Merge = 2,
}

impl KeyedFixedOperationId {
    pub const fn encoding(self) -> [u16; 2] {
        [SemanticProfileId::KeyedFixedForm as u16, self as u16]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum KeyedFixedObjectId {
    Handle = 1,
}

impl KeyedFixedObjectId {
    pub const fn encoding(self) -> [u16; 2] {
        [SemanticProfileId::KeyedFixedForm as u16, self as u16]
    }
}

/// Symbolic generation parameters; no deployment values enter the manifest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum FixedParameterId {
    Key = 1,
    Asset = 2,
    Value = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum KeyedFixedClause {
    BornFirst = 1,
    OwnQualification = 2,
    SuccessorQualification = 3,
}
