//! Typed STATE semantic metadata for the maturity-announcement
//! operation.
//!
//! The metadata is the semantic STATE census and nothing else: the
//! pool quantities, the cycle ordinal, and the closed maturity
//! status, together with the one total transition that announces a
//! maturity cycle and the closed sum of reasons it refuses.
//!
//! Representation, nonce, and encoding are deliberately absent. A
//! nonce is a target-representation artefact that semantic projection
//! erases, and an encoding fixes byte order and discriminants that no
//! semantic statement depends on; admitting either here would let a
//! representation choice travel as if it were semantic STATE. The
//! transition therefore returns a semantic successor only, and
//! representation work happens strictly after it succeeds.
//!
//! The semantic census is stated in terms of this crate's own
//! domains, so the layer stays inside the architecture-to-realization
//! dependency direction: the pipeline reads typed metadata without
//! any pipeline crate depending on the executable model. Agreement
//! with the model's own pool state is a conformance obligation proved
//! against the model, not an import.

use thiserror::Error;

use crate::{Cycle, ProtocolAmount, RealizationError};

/// The closed maturity status carried by semantic STATE.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Maturity {
    /// No maturity cycle has been announced.
    Unannounced,

    /// A maturity cycle has been announced and not yet reached.
    Announced {
        /// The announced maturity cycle ordinal.
        cycle: Cycle,
    },

    /// Maturity has completed.
    Complete,
}

/// Semantic STATE metadata: one field per semantic STATE quantity,
/// plus the cycle ordinal and the maturity status.
///
/// The field set is exhaustive and public: a transition that forgets
/// a field cannot compile, and a consumer reads every semantic
/// quantity without a getter wall, because no field carries an
/// invariant that construction could violate on its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StateMetadata {
    /// The pool backing quantity.
    pub omega: ProtocolAmount,

    /// The live receipt quantity.
    pub y_l: ProtocolAmount,

    /// The time-locked receipt quantity.
    pub y_t: ProtocolAmount,

    /// The pending entitlement quantity.
    pub q: ProtocolAmount,

    /// The current cycle ordinal.
    pub cycle: Cycle,

    /// The maturity status.
    pub maturity: Maturity,
}

impl StateMetadata {
    /// Return this metadata with the maturity status replaced and
    /// every other semantic field copied through unchanged.
    ///
    /// The copy-through is spelled field by field rather than by
    /// update syntax, so a field added to the census forces this
    /// function to be revisited instead of silently inheriting.
    #[must_use]
    pub const fn with_maturity(self, maturity: Maturity) -> Self {
        Self {
            omega: self.omega,
            y_l: self.y_l,
            y_t: self.y_t,
            q: self.q,
            cycle: self.cycle,
            maturity,
        }
    }
}

/// The inclusive announcement lead window, in cycles.
///
/// The bounds are a typed parameter of the transition rather than a
/// constant of this crate: the admissible lead is configuration that
/// callers own, and a transition that read it from a global could not
/// be exercised against two different configurations at once. The
/// invariant the constructor establishes — a nonzero minimum that does
/// not exceed the maximum — is what makes the window nonempty, so the
/// fields stay private.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnnouncementLeadBounds {
    minimum: Cycle,
    maximum: Cycle,
}

impl AnnouncementLeadBounds {
    /// Construct lead bounds, refusing a zero minimum and an inverted
    /// pair.
    ///
    /// A zero minimum would admit an announcement for the current
    /// cycle, which is not a lead at all.
    pub fn new(minimum: Cycle, maximum: Cycle) -> Result<Self, RealizationError> {
        if minimum == Cycle::ZERO || minimum > maximum {
            return Err(RealizationError::InvalidAnnouncementLeadBounds {
                minimum: minimum.get(),
                maximum: maximum.get(),
            });
        }

        Ok(Self { minimum, maximum })
    }

    /// Return the minimum announcement lead.
    #[must_use]
    pub const fn minimum(self) -> Cycle {
        self.minimum
    }

    /// Return the maximum announcement lead.
    #[must_use]
    pub const fn maximum(self) -> Cycle {
        self.maximum
    }

    /// Return the inclusive window of announceable cycles for a
    /// current cycle: both endpoints are admissible.
    ///
    /// Both endpoints are computed with checked arithmetic, so a
    /// current cycle near the top of the domain refuses rather than
    /// wrapping into an apparently valid window.
    pub fn window(self, current: Cycle) -> Result<(Cycle, Cycle), MaturityTransitionRefusal> {
        let earliest = current
            .checked_add(self.minimum)
            .map_err(|_| MaturityTransitionRefusal::CycleArithmeticOverflow)?;

        let latest = current
            .checked_add(self.maximum)
            .map_err(|_| MaturityTransitionRefusal::CycleArithmeticOverflow)?;

        Ok((earliest, latest))
    }
}

/// The closed reason a maturity announcement is refused.
///
/// The sum is distinct from `RealizationError` because it is the
/// vocabulary of one transition: a caller matching it exhaustively
/// learns exactly which transition preconditions exist, which a
/// crate-wide error enum cannot tell it.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq, Hash)]
pub enum MaturityTransitionRefusal {
    /// The predecessor already carries an announced maturity cycle.
    #[error("the predecessor maturity is already announced")]
    PredecessorAlreadyAnnounced,

    /// The predecessor maturity is already complete.
    #[error("the predecessor maturity is complete")]
    PredecessorMaturityComplete,

    /// The announced cycle is earlier than the minimum lead admits.
    #[error("the announced cycle is below the minimum announcement lead")]
    AnnouncementBelowMinimum,

    /// The announced cycle is later than the maximum lead admits.
    #[error("the announced cycle is above the maximum announcement lead")]
    AnnouncementAboveMaximum,

    /// Checked cycle arithmetic overflowed while deriving the window.
    #[error("cycle arithmetic overflowed while deriving the announcement window")]
    CycleArithmeticOverflow,
}

impl MaturityTransitionRefusal {
    /// Every refusal, in declaration order.
    ///
    /// A test walks this slice, so a variant added without a test
    /// that reaches it is visible rather than silently unexercised.
    pub const ALL: &'static [Self] = &[
        Self::PredecessorAlreadyAnnounced,
        Self::PredecessorMaturityComplete,
        Self::AnnouncementBelowMinimum,
        Self::AnnouncementAboveMaximum,
        Self::CycleArithmeticOverflow,
    ];

    /// Return the stable kebab-case name of this refusal.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::PredecessorAlreadyAnnounced => "predecessor-already-announced",
            Self::PredecessorMaturityComplete => "predecessor-maturity-complete",
            Self::AnnouncementBelowMinimum => "announcement-below-minimum",
            Self::AnnouncementAboveMaximum => "announcement-above-maximum",
            Self::CycleArithmeticOverflow => "cycle-arithmetic-overflow",
        }
    }
}

/// Announce a maturity cycle, deriving the one semantic successor.
///
/// The function is total: every input either yields a complete
/// successor or one refusal, and no path panics or returns a
/// partially updated predecessor. The successor differs from the
/// predecessor in the maturity field alone; every other semantic
/// field is copied through by `StateMetadata::with_maturity`.
///
/// The order of checks is the order of the operation's preconditions:
/// the predecessor must be `Unannounced`, the window is derived with
/// checked arithmetic, and the requested cycle must lie inside it.
/// No representation nonce is derived here, because representation
/// grinding is meaningful only once the semantic transition holds.
pub fn announce_maturity(
    predecessor: &StateMetadata,
    announced_cycle: Cycle,
    bounds: AnnouncementLeadBounds,
) -> Result<StateMetadata, MaturityTransitionRefusal> {
    match predecessor.maturity {
        Maturity::Unannounced => {}
        Maturity::Announced { .. } => {
            return Err(MaturityTransitionRefusal::PredecessorAlreadyAnnounced);
        }
        Maturity::Complete => {
            return Err(MaturityTransitionRefusal::PredecessorMaturityComplete);
        }
    }

    let (earliest, latest) = bounds.window(predecessor.cycle)?;

    if announced_cycle < earliest {
        return Err(MaturityTransitionRefusal::AnnouncementBelowMinimum);
    }

    if announced_cycle > latest {
        return Err(MaturityTransitionRefusal::AnnouncementAboveMaximum);
    }

    let maturity = Maturity::Announced {
        cycle: announced_cycle,
    };

    Ok(predecessor.with_maturity(maturity))
}
