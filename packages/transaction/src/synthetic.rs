//! The synthetic test-ASH origin, and what a report must say about it
//! (§15.9).
//!
//! # Why development ASH is synthetic at all
//!
//! An ASH is supposed to come from a burn, and burn is not implemented.
//! So the development fixtures create one another way — a disposable
//! test asset standing for the closed protocol asset, the candidate
//! constructor, deterministic public test values, and a test-only
//! funding ceremony — and every report that touches such an instance
//! has to say what it is not.
//!
//! # The disclaimers are a set, not a sentence
//!
//! §15.9 gives five statements a report must carry. They are typed
//! individually so that a report cannot carry four of them, and they
//! are attached to the instance's *origin* rather than added by a
//! caller, so a report about a synthetic instance cannot be written
//! without them.
//!
//! # The ceremony is not in this crate
//!
//! Creating the disposable asset is a delegated capability: it needs an
//! issuance, and this package constructs none. What lives here is the
//! typed record of where an instance came from and what that means —
//! which is the part a report needs and the part a builder must not be
//! able to overstate.

use std::collections::BTreeSet;

use crate::taproot::AshInstanceOrigin;

/// One statement a report about a synthetic instance must carry.
///
/// The five of §15.9, one variant each. A report carrying some of them
/// would be a report that had decided which of its own disclaimers
/// mattered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SyntheticDisclaimer {
    /// The instance is a synthetic target fixture.
    SyntheticTargetFixture,
    /// It was not produced by burn.
    NotProducedByBurn,
    /// It is not evidence of burn lineage.
    NotEvidenceOfBurnLineage,
    /// It is not an attestation event.
    NotAnAttestationEvent,
    /// It authorizes nothing of value.
    AuthorizesNothingOfValue,
}

impl SyntheticDisclaimer {
    /// The complete census of disclaimers.
    pub const ALL: &'static [Self] = &[
        Self::SyntheticTargetFixture,
        Self::NotProducedByBurn,
        Self::NotEvidenceOfBurnLineage,
        Self::NotAnAttestationEvent,
        Self::AuthorizesNothingOfValue,
    ];

    /// The disclaimers an instance of this origin carries.
    ///
    /// An observed target output carries none, because it is whatever
    /// the target says it is and this crate has no standing to
    /// characterize it further. A synthetic one carries all five.
    #[must_use]
    pub fn for_origin(origin: AshInstanceOrigin) -> BTreeSet<Self> {
        match origin {
            AshInstanceOrigin::SyntheticTestFunding => Self::ALL.iter().copied().collect(),
            _ => BTreeSet::new(),
        }
    }
}

/// What the test-only funding ceremony must supply, and who supplies
/// it.
///
/// A census rather than an implementation. Each entry is a step the
/// ceremony performs outside this package, and naming them is what lets
/// a fixture state its own provenance instead of appearing from
/// nowhere.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FundingCeremonyStep {
    /// Issue the disposable test asset that stands for the closed
    /// protocol asset.
    ///
    /// Delegated: an issuance is a field this crate neither constructs
    /// nor encodes.
    IssueDisposableTestAsset,
    /// Compute the constructor's output program from the committed
    /// tree and the internal key.
    ///
    /// Delegated, and delegated on purpose: the tweak is the half of
    /// the taproot output key this crate does not compute, and the
    /// package that does compute it is the independent oracle rather
    /// than this one.
    DeriveConstructorOutputProgram,
    /// Pay the disposable asset to that program, once per ASH input the
    /// fixture needs.
    FundEachAshInput,
    /// Record the resulting outpoints and their public fields as the
    /// construction view.
    RecordPublicView,
}

impl FundingCeremonyStep {
    /// The complete census of ceremony steps.
    pub const ALL: &'static [Self] = &[
        Self::IssueDisposableTestAsset,
        Self::DeriveConstructorOutputProgram,
        Self::FundEachAshInput,
        Self::RecordPublicView,
    ];
}
