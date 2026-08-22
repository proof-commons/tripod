//! The confidential construction model, and what it does not claim
//! (§12.8).
//!
//! # A model is recorded, not assumed
//!
//! §12.8 gives four construction models a test materializer could
//! record, and names one as the initial expected model. Recording it is
//! the point: a private transaction built one way and reported as though
//! it had been built another is the failure this vocabulary exists to
//! prevent, and the four members are here so that the selection is a
//! value a report can carry rather than an assumption a reader has to
//! reconstruct.
//!
//! # The non-claims are typed for the same reason
//!
//! §12.8 says in as many words that the expected model demonstrates
//! target feasibility and semantic equivalence and does *not*
//! demonstrate a production multi-owner privacy protocol. A sentence in
//! a doc comment is not something a report can be checked against, so
//! [`PrivateConstructionNonClaim`] is a census and
//! [`SelectedConstructionModel`] carries the whole of it. A reader who
//! finds a private construction here and wonders what it establishes
//! reads the value rather than trusting the prose.
//!
//! # Central public-fixture construction, exactly
//!
//! The openings are values the caller published before construction
//! began (§1.10, [`crate::live_request::PublicTestRandomness`]). One
//! party knows every opening, which is what makes the construction
//! central; every opening is a fixture, which is what makes it public;
//! and nothing here generates, stores, or hands back a blinding scalar,
//! which is what keeps it from being the production interface §1.10
//! refuses.

use std::collections::BTreeSet;

use crate::bytes::{AssetId, COMMITMENT_BYTES};
use crate::error::TransactionRefusal;
use crate::live_request::{ProtocolValue, PublicTestRandomness};

/// One construction model a confidential test materializer may record
/// (§12.8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConfidentialConstructionModel {
    /// One party holds every opening, and every opening is a published
    /// fixture.
    CentralPublicFixtureConstruction,
    /// Owners exchange openings with each other cooperatively.
    CooperativeOwnerOpeningExchange,
    /// Blinding is produced by a protocol no single party can open.
    DistributedBlindingProtocol,
    /// An external wallet materializes the confidential fields.
    ExternalWalletMaterializer,
}

impl ConfidentialConstructionModel {
    /// The complete census, in §12.8's own order.
    pub const ALL: &'static [Self] = &[
        Self::CentralPublicFixtureConstruction,
        Self::CooperativeOwnerOpeningExchange,
        Self::DistributedBlindingProtocol,
        Self::ExternalWalletMaterializer,
    ];

    /// The model §12.8 expects initially, and the only one implemented.
    pub const EXPECTED: Self = Self::CentralPublicFixtureConstruction;
}

/// What a private construction under the expected model demonstrates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrivateConstructionDemonstration {
    /// The target's confidential fields can be built for this ABI.
    TargetFeasibility,
    /// The private and explicit representations project to the same
    /// semantics.
    SemanticEquivalence,
}

impl PrivateConstructionDemonstration {
    /// Everything the expected model demonstrates, and no more.
    pub const ALL: &'static [Self] = &[Self::TargetFeasibility, Self::SemanticEquivalence];
}

/// One thing a private construction here does not establish.
///
/// Typed rather than written down, so that a report carrying a private
/// construction carries its non-claims with it and cannot be summarized
/// into a stronger statement than the construction supports.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrivateConstructionNonClaim {
    /// Not a production multi-owner privacy protocol.
    ///
    /// §12.8's own sentence. One party holds every opening, so the
    /// construction establishes nothing about what several parties who
    /// do not trust each other could build.
    NotAProductionMultiOwnerPrivacyProtocol,
    /// No opening here is secret.
    ///
    /// Every one is a published fixture the caller supplied. A
    /// disclosure argument made against these openings would be an
    /// argument about public values.
    NoOpeningIsSecret,
    /// No range proof is produced, and none is checked.
    ///
    /// The field form this crate builds carries value commitments and
    /// empty proof fields. Whether a target accepts a confidential
    /// output without one is a target question, and this construction
    /// does not answer it.
    NoRangeProofIsProducedOrChecked,
    /// The confidential field form is settled on the target and nowhere
    /// else.
    ///
    /// This crate can state which prefix a commitment carries. Whether
    /// the target reads that prefix the way this crate encodes it is
    /// established by a target-native run, and neither confidential
    /// prefix has had one.
    FieldFormSettledOnlyOnTheTarget,
}

impl PrivateConstructionNonClaim {
    /// The complete census, in declaration order.
    pub const ALL: &'static [Self] = &[
        Self::NotAProductionMultiOwnerPrivacyProtocol,
        Self::NoOpeningIsSecret,
        Self::NoRangeProofIsProducedOrChecked,
        Self::FieldFormSettledOnlyOnTheTarget,
    ];
}

/// The construction model one private build recorded, with its
/// non-claims.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectedConstructionModel {
    model: ConfidentialConstructionModel,
    demonstrates: BTreeSet<PrivateConstructionDemonstration>,
    non_claims: BTreeSet<PrivateConstructionNonClaim>,
}

impl SelectedConstructionModel {
    /// The model this materializer implements, with everything it does
    /// and does not establish.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::ConfidentialConstructionModelNotAdmitted`]
    /// for any model other than [`ConfidentialConstructionModel::EXPECTED`].
    /// The other three are named by the vocabulary because §12.8 names
    /// them, and refused by this constructor because nothing here
    /// implements them — a materializer that silently recorded a model
    /// it had not performed would be the exact failure the vocabulary is
    /// for.
    pub fn record(model: ConfidentialConstructionModel) -> Result<Self, TransactionRefusal> {
        if model != ConfidentialConstructionModel::EXPECTED {
            return Err(TransactionRefusal::ConfidentialConstructionModelNotAdmitted);
        }
        Ok(Self {
            model,
            demonstrates: PrivateConstructionDemonstration::ALL
                .iter()
                .copied()
                .collect(),
            non_claims: PrivateConstructionNonClaim::ALL.iter().copied().collect(),
        })
    }

    /// The recorded model.
    #[must_use]
    pub const fn model(&self) -> ConfidentialConstructionModel {
        self.model
    }

    /// What the construction demonstrates.
    #[must_use]
    pub const fn demonstrates(&self) -> &BTreeSet<PrivateConstructionDemonstration> {
        &self.demonstrates
    }

    /// What it does not.
    #[must_use]
    pub const fn non_claims(&self) -> &BTreeSet<PrivateConstructionNonClaim> {
        &self.non_claims
    }
}

/// The confidential value arithmetic a private construction needs.
///
/// A second adapter, kept apart from
/// [`crate::live_taproot::LiveCurveCapability`] because the explicit
/// representation needs none of it: a trait carrying both would make
/// every explicit build declare an implementation of arithmetic it never
/// calls.
///
/// The blinding scalar is derived by the implementation from the
/// published randomness and the output's own position, and never crosses
/// this boundary in either direction. There is no method here that
/// returns an opening.
pub trait PrivateValueCapability {
    /// The value commitment one destination's amount takes.
    ///
    /// `None` is an ordinary outcome — a derived scalar out of range, or
    /// a commitment at the group identity — and construction refuses
    /// rather than proceeding on a value it did not get.
    fn value_commitment(
        &self,
        asset: AssetId,
        value: ProtocolValue,
        randomness: &PublicTestRandomness,
        position: u16,
    ) -> Option<[u8; COMMITMENT_BYTES]>;
}
