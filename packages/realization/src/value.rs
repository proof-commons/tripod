//! Runtime values used by the small target-independent expression
//! evaluator.
//!
//! These values are semantic observations. They contain no target
//! encoding, stack representation, transaction index, or backend
//! witness format.

use std::collections::BTreeSet;

use crate::{Count, Cycle, Maturity, ProtocolAmount};

/// Opaque owner identity used by realization observations.
///
/// The realization needs equality and set inclusion only. It assigns
/// no cryptographic or target encoding semantics to these bytes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OwnerId(pub [u8; 32]);

/// Type of one expression value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticType {
    Bool,
    Count,
    Amount,
    Cycle,
    Maturity,
    OwnerSet,
}

/// Evaluated target-independent value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticValue {
    Bool(bool),
    Count(Count),
    Amount(ProtocolAmount),
    Cycle(Cycle),
    Maturity(Maturity),
    OwnerSet(BTreeSet<OwnerId>),
}

impl SemanticValue {
    /// Type of this value.
    #[must_use]
    pub const fn semantic_type(&self) -> SemanticType {
        match self {
            Self::Bool(_) => SemanticType::Bool,
            Self::Count(_) => SemanticType::Count,
            Self::Amount(_) => SemanticType::Amount,
            Self::Cycle(_) => SemanticType::Cycle,
            Self::Maturity(_) => SemanticType::Maturity,
            Self::OwnerSet(_) => SemanticType::OwnerSet,
        }
    }

    /// Read this value as a boolean.
    #[must_use]
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// Read this value as a count.
    #[must_use]
    pub const fn as_count(&self) -> Option<Count> {
        match self {
            Self::Count(value) => Some(*value),
            _ => None,
        }
    }

    /// Read this value as a protocol amount.
    #[must_use]
    pub const fn as_amount(&self) -> Option<ProtocolAmount> {
        match self {
            Self::Amount(value) => Some(*value),
            _ => None,
        }
    }

    /// Read this value as a cycle ordinal.
    #[must_use]
    pub const fn as_cycle(&self) -> Option<Cycle> {
        match self {
            Self::Cycle(value) => Some(*value),
            _ => None,
        }
    }

    /// Read this value as a maturity status.
    #[must_use]
    pub const fn as_maturity(&self) -> Option<Maturity> {
        match self {
            Self::Maturity(value) => Some(*value),
            _ => None,
        }
    }

    /// Read this value as an owner set.
    #[must_use]
    pub fn as_owner_set(&self) -> Option<&BTreeSet<OwnerId>> {
        match self {
            Self::OwnerSet(value) => Some(value),
            _ => None,
        }
    }
}
