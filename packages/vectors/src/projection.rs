//! The accepted semantic projection of Guide-12 §17.4.
//!
//! §1.4 makes target acceptance and semantic acceptance two verdicts,
//! and says the successful condition is acceptance *and* projection
//! equality. This module is the second half of that conjunction: the
//! exact set of facts an accepted transaction is compared on.
//!
//! # Why the sponsor region carries no amount
//!
//! §17.4 closes by ruling that individual sponsor amounts and openings
//! are absent, and §1.6 requires sponsor opacity to survive lowering.
//! [`SponsorRegion`] therefore has nowhere to put an amount: it records
//! existence, membership, and whether a change role exists, and nothing
//! else. A field that could hold a sponsor amount would eventually hold
//! one — which is why the change role is a presence and not a residual.

use realization::ProtocolAmount;

/// A semantic handle for one object in a fixture's world.
///
/// Deliberately not a target index. §17.2 excludes target input and
/// output indices from a semantic fixture, and a handle that happened to
/// equal an index would be one refactor away from being read as one.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticObjectId(u32);

impl SemanticObjectId {
    /// The handle numbered `ordinal` in a fixture's own numbering.
    #[must_use]
    pub const fn new(ordinal: u32) -> Self {
        Self(ordinal)
    }

    /// The handle's ordinal.
    #[must_use]
    pub const fn ordinal(self) -> u32 {
        self.0
    }
}

/// Whether the projected objects are ownerless.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OwnershipStatus {
    /// Compact ASH consumes and creates ownerless objects.
    Ownerless,
    /// Some projected object carries an owner.
    Owned,
}

/// Whether a root succession appears in the projection.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RootSuccession {
    /// No root is consumed or created. Required for compact ASH.
    Absent,
    /// A root edge appears.
    Present,
}

/// Whether an issuance appears in the projection.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IssuanceClaim {
    /// No issuance. Required for compact ASH.
    Absent,
    /// An issuance appears.
    Present,
}

/// Whether a destruction appears in the projection.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DestructionClaim {
    /// No destruction. Required for compact ASH.
    Absent,
    /// A destruction appears.
    Present,
}

/// Whether a specialized event is projected.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EventClaim {
    /// No specialized event role. Required for compact ASH.
    Absent,
    /// A specialized event is projected.
    Specialized,
}

/// The canonical movement kind the flow is filed under.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CanonicalFlowKind {
    /// The compact-ASH flow: ownerless, lateral.
    OwnerlessLateral,
    /// An owner-controlled lateral movement.
    OwnerControlledLateral,
    /// Issuance.
    Issuance,
    /// Destruction.
    Destruction,
    /// A burn record.
    Burn,
    /// A clear exit.
    Clear,
}

/// Whether a sponsor region returns a residual to its sponsor.
///
/// # Why this is a projected fact and not a construction detail
///
/// §18.1 names `sponsor-change-present` and `sponsor-change-absent` as
/// two classes, which makes the presence of a change role a property a
/// run has to be able to witness. A projection that recorded only
/// existence and membership could not tell the two apart, so a run of
/// either row would be equally good evidence for both — the row would
/// carry the class name without carrying the property the name asserts
/// (`G13-R10`).
///
/// It is a presence and never an amount, for the reason this module's
/// own documentation gives: §17.4 rules individual sponsor amounts and
/// openings absent, and a residual's *size* is exactly such an amount.
/// That a residual exists is structural; how much sits in it is not
/// projected here and has nowhere to go if it were.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SponsorChange {
    /// The sponsor region returns nothing: its whole contribution is
    /// the fee.
    Absent,
    /// The sponsor region carries a change role.
    Present,
}

impl SponsorChange {
    /// Whether a change role exists.
    #[must_use]
    pub const fn is_present(self) -> bool {
        matches!(self, Self::Present)
    }
}

/// The sponsor region's existence, membership, and change role.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SponsorRegion {
    present: bool,
    members: u16,
    change: SponsorChange,
}

impl SponsorRegion {
    /// A sponsorless projection.
    ///
    /// Change is absent because there is no region to carry one, which
    /// is a fact rather than a default: §9.1 refuses a change role with
    /// no sponsor region, so this is the only value the field can take
    /// here.
    pub const ABSENT: Self = Self {
        present: false,
        members: 0,
        change: SponsorChange::Absent,
    };

    /// A sponsored projection with `members` members and this change
    /// role.
    #[must_use]
    pub const fn present(members: u16, change: SponsorChange) -> Self {
        Self {
            present: true,
            members,
            change,
        }
    }

    /// Whether a sponsor region exists at all.
    #[must_use]
    pub const fn is_present(self) -> bool {
        self.present
    }

    /// How many members the region has.
    #[must_use]
    pub const fn members(self) -> u16 {
        self.members
    }

    /// Whether the region returns a residual to its sponsor.
    #[must_use]
    pub const fn change(self) -> SponsorChange {
        self.change
    }
}

/// How the transition certificate is derived.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TransitionCertificate {
    /// Derived from the projected relations by an independent verifier.
    DerivedFromRelations,
    /// Absent, which compact ASH never projects.
    Absent,
}

/// The thirteen comparisons §17.4 requires for an accepted transaction.
///
/// Every field is a semantic fact. None is a byte, an index, a program,
/// or a witness: comparing those would compare the construction with
/// itself rather than with the model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedProjection {
    input_family: Vec<(SemanticObjectId, ProtocolAmount)>,
    successor: (SemanticObjectId, ProtocolAmount),
    explicit_u: ProtocolAmount,
    aggregate: ProtocolAmount,
    ownership: OwnershipStatus,
    roots: RootSuccession,
    issuance: IssuanceClaim,
    destruction: DestructionClaim,
    flow: CanonicalFlowKind,
    sponsor: SponsorRegion,
    event: EventClaim,
    certificate: TransitionCertificate,
    operation: &'static str,
}

impl AcceptedProjection {
    /// Assemble a projection.
    ///
    /// Takes every field, because §17.4 says verdict equality alone is
    /// insufficient and a partially populated projection would be a
    /// verdict wearing a projection's name.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        input_family: Vec<(SemanticObjectId, ProtocolAmount)>,
        successor: (SemanticObjectId, ProtocolAmount),
        explicit_u: ProtocolAmount,
        aggregate: ProtocolAmount,
        sponsor: SponsorRegion,
        certificate: TransitionCertificate,
        operation: &'static str,
    ) -> Self {
        Self {
            input_family,
            successor,
            explicit_u,
            aggregate,
            // §12.10 and §17.4 fix these five for every compact-ASH
            // projection. They are constructor-fixed rather than
            // caller-supplied so that a comparison cannot be made to
            // pass by asking for the wrong absence.
            ownership: OwnershipStatus::Ownerless,
            roots: RootSuccession::Absent,
            issuance: IssuanceClaim::Absent,
            destruction: DestructionClaim::Absent,
            flow: CanonicalFlowKind::OwnerlessLateral,
            sponsor,
            event: EventClaim::Absent,
            certificate,
            operation,
        }
    }

    /// The exact ASH input family.
    #[must_use]
    pub fn input_family(&self) -> &[(SemanticObjectId, ProtocolAmount)] {
        &self.input_family
    }

    /// The exact successor ASH family member.
    #[must_use]
    pub const fn successor(&self) -> (SemanticObjectId, ProtocolAmount) {
        self.successor
    }

    /// The exact explicit `U`.
    #[must_use]
    pub const fn explicit_u(&self) -> ProtocolAmount {
        self.explicit_u
    }

    /// The exact aggregate amount.
    #[must_use]
    pub const fn aggregate(&self) -> ProtocolAmount {
        self.aggregate
    }

    /// The ownerless status.
    #[must_use]
    pub const fn ownership(&self) -> OwnershipStatus {
        self.ownership
    }

    /// Whether any root succession is projected.
    #[must_use]
    pub const fn roots(&self) -> RootSuccession {
        self.roots
    }

    /// Whether any issuance is projected.
    #[must_use]
    pub const fn issuance(&self) -> IssuanceClaim {
        self.issuance
    }

    /// Whether any destruction is projected.
    #[must_use]
    pub const fn destruction(&self) -> DestructionClaim {
        self.destruction
    }

    /// The canonical movement kind.
    #[must_use]
    pub const fn flow(&self) -> CanonicalFlowKind {
        self.flow
    }

    /// The sponsor region's existence and membership.
    #[must_use]
    pub const fn sponsor(&self) -> SponsorRegion {
        self.sponsor
    }

    /// Whether a specialized event is projected.
    #[must_use]
    pub const fn event(&self) -> EventClaim {
        self.event
    }

    /// How the transition certificate is derived.
    #[must_use]
    pub const fn certificate(&self) -> TransitionCertificate {
        self.certificate
    }

    /// The exact operation identity.
    #[must_use]
    pub const fn operation(&self) -> &'static str {
        self.operation
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AcceptedProjection, CanonicalFlowKind, DestructionClaim, EventClaim, IssuanceClaim,
        OwnershipStatus, RootSuccession, SemanticObjectId, SponsorChange, SponsorRegion,
        TransitionCertificate,
    };
    use realization::ProtocolAmount;

    fn amount(value: u64) -> ProtocolAmount {
        ProtocolAmount::new(value).expect("the fixture amount is in the v13 domain")
    }

    fn sample(sponsor: SponsorRegion) -> AcceptedProjection {
        AcceptedProjection::new(
            vec![
                (SemanticObjectId::new(0), amount(120)),
                (SemanticObjectId::new(1), amount(180)),
            ],
            (SemanticObjectId::new(2), amount(300)),
            amount(300),
            amount(300),
            sponsor,
            TransitionCertificate::DerivedFromRelations,
            "compact-ash",
        )
    }

    #[test]
    fn the_five_absences_are_fixed_by_the_constructor() {
        let projection = sample(SponsorRegion::ABSENT);
        assert_eq!(projection.ownership(), OwnershipStatus::Ownerless);
        assert_eq!(projection.roots(), RootSuccession::Absent);
        assert_eq!(projection.issuance(), IssuanceClaim::Absent);
        assert_eq!(projection.destruction(), DestructionClaim::Absent);
        assert_eq!(projection.event(), EventClaim::Absent);
        assert_eq!(projection.flow(), CanonicalFlowKind::OwnerlessLateral);
    }

    #[test]
    fn a_sponsor_region_states_membership_and_never_an_amount() {
        // §17.4: individual sponsor amounts and openings are absent.
        // The region has nowhere to put one, and its rendering must not
        // acquire one by accident either.
        let sponsored = sample(SponsorRegion::present(2, SponsorChange::Absent));
        assert!(sponsored.sponsor().is_present());
        assert_eq!(sponsored.sponsor().members(), 2);

        let rendered = format!("{:?}", sponsored.sponsor());
        assert!(rendered.contains("members"));
        for forbidden in ["amount", "value", "opening", "blinder", "fee"] {
            assert!(
                !rendered.contains(forbidden),
                "the sponsor region rendered {forbidden}"
            );
        }
    }

    #[test]
    fn two_projections_differing_in_one_semantic_fact_are_unequal() {
        // §17.4's point: verdict equality is insufficient, so the
        // projection has to actually discriminate.
        let base = sample(SponsorRegion::ABSENT);
        let sponsored = sample(SponsorRegion::present(1, SponsorChange::Absent));
        assert_ne!(base, sponsored);

        let shifted = AcceptedProjection::new(
            vec![
                (SemanticObjectId::new(0), amount(121)),
                (SemanticObjectId::new(1), amount(179)),
            ],
            (SemanticObjectId::new(2), amount(300)),
            amount(300),
            amount(300),
            SponsorRegion::ABSENT,
            TransitionCertificate::DerivedFromRelations,
            "compact-ash",
        );
        assert_ne!(
            base, shifted,
            "an equal aggregate must not hide a changed family"
        );
    }
}
