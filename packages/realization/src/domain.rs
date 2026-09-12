//! Target-independent semantic domains.
//!
//! Counts and protocol amounts are deliberately distinct even though
//! both currently have integer representations. A count describes
//! cardinality. A protocol amount participates in economic value
//! relations and carries the v13 amount-domain bound.
//!
//! A cycle is an ordinal: it names a position in the cycle sequence,
//! neither a count nor an amount. Ordinals compare and advance, they
//! carry no cardinality, and they take part in no value relation, so
//! a cycle never stands in for either neighbouring domain.

use crate::RealizationError;

/// Exclusive upper bound of the v13 protocol-amount domain.
pub const PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE: u64 = 1_u64 << 51;

/// A protocol amount satisfying `0 <= v < 2^51`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProtocolAmount(u64);

impl ProtocolAmount {
    /// Zero protocol amount.
    pub const ZERO: Self = Self(0);

    /// One protocol amount.
    pub const ONE: Self = Self(1);

    /// Construct an amount after checking the v13 domain.
    pub const fn new(value: u64) -> Result<Self, RealizationError> {
        if value < PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE {
            Ok(Self(value))
        } else {
            Err(RealizationError::AmountOutOfDomain { value })
        }
    }

    /// Return the underlying exact integer value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Return whether this amount is zero.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Add two protocol amounts and re-establish the domain bound.
    pub fn checked_add(self, other: Self) -> Result<Self, RealizationError> {
        let value = self
            .0
            .checked_add(other.0)
            .ok_or(RealizationError::AmountOverflow)?;

        Self::new(value)
    }

    /// Subtract one protocol amount from another.
    pub fn checked_sub(self, other: Self) -> Result<Self, RealizationError> {
        let value = self
            .0
            .checked_sub(other.0)
            .ok_or(RealizationError::AmountUnderflow)?;

        Self::new(value)
    }

    /// Sum protocol amounts with checked arithmetic.
    pub fn checked_sum(values: impl IntoIterator<Item = Self>) -> Result<Self, RealizationError> {
        values.into_iter().try_fold(Self::ZERO, Self::checked_add)
    }
}

/// A nonnegative semantic cardinality.
///
/// A count deliberately carries no protocol-amount interpretation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Count(u64);

impl Count {
    /// Zero count.
    pub const ZERO: Self = Self(0);

    /// One count.
    pub const ONE: Self = Self(1);

    /// Construct a count.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Return the underlying exact integer.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Return whether this count is zero.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Add two counts.
    pub fn checked_add(self, other: Self) -> Result<Self, RealizationError> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or(RealizationError::CountOverflow)
    }
}

/// Target-independent value-representation capability.
///
/// These modes denote the same semantic value. They describe which
/// target proofs may later be selected; they do not claim that the
/// current workspace already implements those target proofs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RepresentationMode {
    /// Amount and closed asset identity are directly encoded.
    Explicit,

    /// Amount remains private under a target value commitment.
    PrivateCommitted,

    /// Amount is public through an authenticated opening while target
    /// commitment algebra remains available.
    PublicCommitted,
}

/// A cycle ordinal.
///
/// The cycle domain is the whole `u64`, matching the model's `Cycle`
/// alias, so construction is total. An announcement lead is expressed
/// in the same domain, because a lead is a distance between ordinals;
/// that is why `checked_add` takes a second cycle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cycle(u64);

impl Cycle {
    /// The least cycle ordinal.
    pub const ZERO: Self = Self(0);

    /// The greatest representable cycle ordinal.
    pub const MAX: Self = Self(u64::MAX);

    /// Construct a cycle ordinal.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Return the underlying exact integer ordinal.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Advance a cycle ordinal, failing closed on overflow.
    pub fn checked_add(self, other: Self) -> Result<Self, RealizationError> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or(RealizationError::CycleOverflow)
    }
}
