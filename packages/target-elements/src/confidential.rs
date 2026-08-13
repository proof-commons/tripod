//! Confidential-value and issuance capability descriptions.
//!
//! Both contracts state target facts and stop there. Nothing here maps
//! an issuance field onto a protocol asset role, and nothing here
//! marks a protocol relation discharged: whole-transaction value
//! conservation is a claim about the target's consensus rules, not a
//! proof that any particular relation holds.

use std::collections::{BTreeMap, BTreeSet};

use crate::encoding::EncodingClass;
use crate::evidence::TargetEvidenceRequirementId;
use crate::opcode::OpcodeId;

/// One confidential-value claim the target might support.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ConfidentialValueCapability {
    /// The target's own rules conserve value across a transaction.
    ConsensusValueConservation,
    /// A program can establish that two commitments are equal.
    CommitmentEquality,
    /// A program can read an amount carried in the clear.
    ExplicitValueInspection,
    /// A program can read a blinded amount's commitment.
    ConfidentialValueInspection,
    /// A program can establish that a commitment opens to a claimed
    /// amount, in a way that binds the claim.
    AuthenticatedOpening,
}

impl ConfidentialValueCapability {
    /// The complete census of confidential-value claims.
    pub const ALL: &'static [Self] = &[
        Self::ConsensusValueConservation,
        Self::CommitmentEquality,
        Self::ExplicitValueInspection,
        Self::ConfidentialValueInspection,
        Self::AuthenticatedOpening,
    ];
}

/// What the review established about one confidential-value claim.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ConfidentialCapabilityState {
    /// A reviewed script primitive establishes it directly.
    PrimitiveReviewed,
    /// The target is relied upon for it outside the script language,
    /// so no program instruction demonstrates it and a deployment must
    /// evidence it separately.
    ExternalConsensusClaim,
    /// No reviewed mechanism establishes it.
    ///
    /// This is not a statement that the target cannot do it. It is a
    /// statement that this package has found no reviewed way, and a
    /// consumer must therefore treat the claim as unavailable.
    Unsupported,
}

/// The target's confidential-value capabilities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfidentialValueContract {
    states: BTreeMap<ConfidentialValueCapability, ConfidentialCapabilityState>,
    participating_encodings: BTreeSet<EncodingClass>,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
}

impl ConfidentialValueContract {
    /// States the confidential-value contract.
    #[must_use]
    pub fn new(
        states: impl IntoIterator<Item = (ConfidentialValueCapability, ConfidentialCapabilityState)>,
        participating_encodings: impl IntoIterator<Item = EncodingClass>,
        evidence: impl IntoIterator<Item = TargetEvidenceRequirementId>,
    ) -> Self {
        Self {
            states: states.into_iter().collect(),
            participating_encodings: participating_encodings.into_iter().collect(),
            evidence: evidence.into_iter().collect(),
        }
    }

    /// The state of each claim.
    #[must_use]
    pub const fn states(
        &self,
    ) -> &BTreeMap<ConfidentialValueCapability, ConfidentialCapabilityState> {
        &self.states
    }

    /// The value encodings that participate in conservation.
    #[must_use]
    pub const fn participating_encodings(&self) -> &BTreeSet<EncodingClass> {
        &self.participating_encodings
    }

    /// The evidence a deployment must produce.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }

    /// The first claim the contract leaves unclassified.
    #[must_use]
    pub fn unclassified(&self) -> Option<ConfidentialValueCapability> {
        ConfidentialValueCapability::ALL
            .iter()
            .find(|claim| !self.states.contains_key(claim))
            .copied()
    }
}

/// One field an issuance carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum IssuanceField {
    /// The amount of the asset being issued.
    AssetAmount,
    /// The amount of reissuance authority being created.
    InflationKeysAmount,
    /// The entropy binding the issued asset's identity.
    AssetEntropy,
    /// The blinding nonce, whose value distinguishes an issuance from
    /// a reissuance.
    BlindingNonce,
}

impl IssuanceField {
    /// The complete census of issuance fields.
    pub const ALL: &'static [Self] = &[
        Self::AssetAmount,
        Self::InflationKeysAmount,
        Self::AssetEntropy,
        Self::BlindingNonce,
    ];
}

/// The target's issuance and reissuance facts.
///
/// These are target facts only. No field here is mapped to a protocol
/// asset, authority, or distribution control; that mapping belongs to
/// a backend and a transaction layout, both of which are downstream of
/// this package and neither of which exists yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IssuanceContract {
    fields: BTreeSet<IssuanceField>,
    introspection: OpcodeId,
    absent_marker: EncodingClass,
    outpoint_flag_reports_issuance: bool,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
}

impl IssuanceContract {
    /// States the issuance contract.
    #[must_use]
    pub fn new(
        fields: impl IntoIterator<Item = IssuanceField>,
        introspection: OpcodeId,
        absent_marker: EncodingClass,
        outpoint_flag_reports_issuance: bool,
        evidence: impl IntoIterator<Item = TargetEvidenceRequirementId>,
    ) -> Self {
        Self {
            fields: fields.into_iter().collect(),
            introspection,
            absent_marker,
            outpoint_flag_reports_issuance,
            evidence: evidence.into_iter().collect(),
        }
    }

    /// The fields an issuance carries.
    #[must_use]
    pub const fn fields(&self) -> &BTreeSet<IssuanceField> {
        &self.fields
    }

    /// The primitive that reads them.
    #[must_use]
    pub const fn introspection(&self) -> OpcodeId {
        self.introspection
    }

    /// The encoding marking an input that carries no issuance.
    #[must_use]
    pub const fn absent_marker(&self) -> EncodingClass {
        self.absent_marker
    }

    /// Whether the outpoint flag byte also reports issuance presence.
    #[must_use]
    pub const fn outpoint_flag_reports_issuance(&self) -> bool {
        self.outpoint_flag_reports_issuance
    }

    /// The evidence a deployment must produce.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }
}

/// Builds the reviewed confidential-value contract.
pub(crate) fn reviewed_confidential_values() -> ConfidentialValueContract {
    use ConfidentialCapabilityState as S;
    use ConfidentialValueCapability as V;
    use TargetEvidenceRequirementId as R;

    ConfidentialValueContract::new(
        [
            // The target conserves value by its own consensus rules.
            // No script primitive demonstrates it, so a deployment
            // must evidence it and no relation may be marked
            // discharged on the strength of this line.
            (V::ConsensusValueConservation, S::ExternalConsensusClaim),
            // No reviewed primitive establishes equality of two
            // commitments.
            (V::CommitmentEquality, S::Unsupported),
            (V::ExplicitValueInspection, S::PrimitiveReviewed),
            (V::ConfidentialValueInspection, S::PrimitiveReviewed),
            // Deliberately unsupported. Low-level curve and hash
            // primitives exist, and their existence is not an opening
            // proof: marking this complete because the parts are
            // present is precisely the error the contract is shaped to
            // prevent.
            (V::AuthenticatedOpening, S::Unsupported),
        ],
        [
            EncodingClass::ExplicitValue,
            EncodingClass::ConfidentialValue,
            EncodingClass::NullValue,
        ],
        [R::ConfidentialValueConservation, R::CommitmentEquality],
    )
}

/// Builds the reviewed issuance contract.
pub(crate) fn reviewed_issuance() -> IssuanceContract {
    IssuanceContract::new(
        IssuanceField::ALL.iter().copied(),
        OpcodeId::InspectInputIssuance,
        EncodingClass::NullValue,
        true,
        [TargetEvidenceRequirementId::IssuanceIntrospection],
    )
}
