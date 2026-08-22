//! Confidential-value and issuance capability descriptions.
//!
//! Both contracts state target facts and stop there. Nothing here maps
//! an issuance field onto a protocol asset role, and nothing here
//! marks a protocol relation discharged: whole-transaction value
//! conservation is a claim about the target's consensus rules, not a
//! proof that any particular relation holds.

use std::collections::{BTreeMap, BTreeSet};

use crate::capability::census_enum;
use crate::encoding::EncodingClass;
use crate::evidence::TargetEvidenceRequirementId;
use crate::opcode::OpcodeId;

census_enum! {
    /// One confidential-value claim the target might support.
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

census_enum! {
    /// One field an issuance carries.
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

/// How a curve point's encoding selects between the two y values that
/// share an x coordinate.
///
/// The distinction is the reason a pattern may not be carried over from
/// one family of primitives to the other by analogy: the two
/// conventions pick the same y only by coincidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum PointParityConvention {
    /// The prefix records whether y is a quadratic residue.
    ///
    /// This is what the target's own confidential encodings use, for
    /// both value commitments and asset generators.
    QuadraticResidue,
    /// The prefix records whether y is odd.
    ///
    /// This is what a compressed public key uses, and therefore what
    /// the curve-checking primitives accept.
    CompressedOddness,
    /// No prefix is carried and the even y is implied.
    ///
    /// This is what an x-only key uses.
    ImpliedEvenY,
}

/// The two bytes a standard compressed curve point is written with.
///
/// The same pair the encoding registry gives its compressed
/// public-key class, restated here because this table is transcribed
/// from the reviewed target rather than derived from the registry. The
/// `G12-R10` witness pins the two against each other.
const COMPRESSED_POINT_PREFIXES: (u8, u8) = (2, 3);

/// The reviewed encoding of one confidential field.
///
/// The convention the committed prefix follows is not stated
/// separately: it is read off the prefix pair, so a field cannot claim
/// a convention its bytes do not use.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConfidentialFieldEncoding {
    explicit_width: usize,
    committed_width: usize,
    explicit_prefix: u8,
    committed_prefixes: (u8, u8),
}

impl ConfidentialFieldEncoding {
    /// States one field's encoding.
    #[must_use]
    pub const fn new(
        explicit_width: usize,
        committed_width: usize,
        explicit_prefix: u8,
        committed_prefixes: (u8, u8),
    ) -> Self {
        Self {
            explicit_width,
            committed_width,
            explicit_prefix,
            committed_prefixes,
        }
    }

    /// The serialized width of the explicit form, prefix included.
    #[must_use]
    pub const fn explicit_width(&self) -> usize {
        self.explicit_width
    }

    /// The serialized width of the committed form, prefix included.
    #[must_use]
    pub const fn committed_width(&self) -> usize {
        self.committed_width
    }

    /// The byte marking the explicit form.
    #[must_use]
    pub const fn explicit_prefix(&self) -> u8 {
        self.explicit_prefix
    }

    /// The two bytes marking the committed form.
    ///
    /// The pair is ordered as the encoder emits it, low bit clear
    /// first. What that bit records is the field's own convention,
    /// reported by [`Self::parity`].
    #[must_use]
    pub const fn committed_prefixes(&self) -> (u8, u8) {
        self.committed_prefixes
    }

    /// Which y the committed prefix selects.
    ///
    /// Read off the prefix pair rather than asserted alongside it. A
    /// field the target commits to itself carries a per-field constant
    /// exclusive-or the squareness of y; a point the target only
    /// transports carries the standard compressed pair instead, whose
    /// low bit records oddness. The pair therefore settles the
    /// convention, and no field can name the one its bytes do not use.
    ///
    /// [`PointParityConvention::ImpliedEvenY`] never arises here: it
    /// belongs to an encoding that carries no prefix at all.
    #[must_use]
    pub const fn parity(&self) -> PointParityConvention {
        if self.committed_prefixes.0 == COMPRESSED_POINT_PREFIXES.0
            && self.committed_prefixes.1 == COMPRESSED_POINT_PREFIXES.1
        {
            PointParityConvention::CompressedOddness
        } else {
            PointParityConvention::QuadraticResidue
        }
    }

    /// Whether a byte is one this field admits.
    #[must_use]
    pub const fn admits_prefix(&self, prefix: u8) -> bool {
        prefix == 0
            || prefix == self.explicit_prefix
            || prefix == self.committed_prefixes.0
            || prefix == self.committed_prefixes.1
    }
}

/// The reviewed recipe taking an asset identifier to its generator.
///
/// Stated so a consumer can see what a program would have to perform to
/// derive a generator on-script, and therefore why it cannot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetGeneratorDerivation {
    identifier_width: usize,
    hash_prefix_width: usize,
    curve_map_evaluations: usize,
    point_additions: usize,
    blinded_form_adds_base_multiple: bool,
}

impl AssetGeneratorDerivation {
    /// States the derivation.
    #[must_use]
    pub const fn new(
        identifier_width: usize,
        hash_prefix_width: usize,
        curve_map_evaluations: usize,
        point_additions: usize,
        blinded_form_adds_base_multiple: bool,
    ) -> Self {
        Self {
            identifier_width,
            hash_prefix_width,
            curve_map_evaluations,
            point_additions,
            blinded_form_adds_base_multiple,
        }
    }

    /// The width of the asset identifier the recipe consumes.
    #[must_use]
    pub const fn identifier_width(&self) -> usize {
        self.identifier_width
    }

    /// The width of each generation's tagged hash prefix.
    #[must_use]
    pub const fn hash_prefix_width(&self) -> usize {
        self.hash_prefix_width
    }

    /// How many times the recipe maps a field element to the curve.
    #[must_use]
    pub const fn curve_map_evaluations(&self) -> usize {
        self.curve_map_evaluations
    }

    /// How many point additions the recipe performs.
    #[must_use]
    pub const fn point_additions(&self) -> usize {
        self.point_additions
    }

    /// Whether the blinded form adds a multiple of the base point.
    #[must_use]
    pub const fn blinded_form_adds_base_multiple(&self) -> bool {
        self.blinded_form_adds_base_multiple
    }
}

/// Which term of the commitment relation carries which generator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CommitmentTermRoles {
    /// The blinding scalar multiplies the conventional base point and
    /// the amount multiplies the asset generator, and the two are
    /// added.
    ///
    /// This is the reviewed convention. Both terms are positive.
    BlindOnBaseAmountOnAssetGenerator,
    /// The roles are exchanged.
    BlindOnAssetGeneratorAmountOnBase,
}

/// How the opening scalar is read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ScalarByteOrder {
    /// Most significant byte first.
    BigEndian,
    /// Least significant byte first.
    LittleEndian,
}

/// The reviewed commitment relation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommitmentRelation {
    roles: CommitmentTermRoles,
    scalar_width: usize,
    scalar_order: ScalarByteOrder,
    admits_zero_scalar: bool,
    rejects_scalar_at_or_above_group_order: bool,
    rejects_identity_result: bool,
}

impl CommitmentRelation {
    /// States the relation.
    #[must_use]
    pub const fn new(
        roles: CommitmentTermRoles,
        scalar_width: usize,
        scalar_order: ScalarByteOrder,
        admits_zero_scalar: bool,
        rejects_scalar_at_or_above_group_order: bool,
        rejects_identity_result: bool,
    ) -> Self {
        Self {
            roles,
            scalar_width,
            scalar_order,
            admits_zero_scalar,
            rejects_scalar_at_or_above_group_order,
            rejects_identity_result,
        }
    }

    /// Which generator each term uses.
    #[must_use]
    pub const fn roles(&self) -> CommitmentTermRoles {
        self.roles
    }

    /// The width of the opening scalar.
    #[must_use]
    pub const fn scalar_width(&self) -> usize {
        self.scalar_width
    }

    /// How the opening scalar's bytes are ordered.
    #[must_use]
    pub const fn scalar_order(&self) -> ScalarByteOrder {
        self.scalar_order
    }

    /// Whether an all-zero scalar is admitted.
    ///
    /// It is, and consensus relies on it: an explicit amount enters the
    /// balance as a commitment under a zero blinder.
    #[must_use]
    pub const fn admits_zero_scalar(&self) -> bool {
        self.admits_zero_scalar
    }

    /// Whether a scalar at or above the group order is refused.
    #[must_use]
    pub const fn rejects_scalar_at_or_above_group_order(&self) -> bool {
        self.rejects_scalar_at_or_above_group_order
    }

    /// Whether a commitment landing on the identity is refused.
    #[must_use]
    pub const fn rejects_identity_result(&self) -> bool {
        self.rejects_identity_result
    }
}

/// How the target closes its confidential-value balance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ConservationForm {
    /// Inputs less outputs must be the identity exactly, with no
    /// excess term and no kernel.
    ExactTallyToIdentity,
    /// A non-zero excess is carried and separately authorized.
    ExcessCarried,
}

/// When the target demands a proof alongside a blinded field, and what
/// that proof binds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofRequirements {
    range_proof_required_for: BTreeSet<EncodingClass>,
    range_proof_binds_output_script: bool,
    range_proof_excludes_zero_when_spendable: bool,
    surjection_proof_required_for: BTreeSet<EncodingClass>,
}

impl ProofRequirements {
    /// States the proof requirements.
    #[must_use]
    pub fn new(
        range_proof_required_for: impl IntoIterator<Item = EncodingClass>,
        range_proof_binds_output_script: bool,
        range_proof_excludes_zero_when_spendable: bool,
        surjection_proof_required_for: impl IntoIterator<Item = EncodingClass>,
    ) -> Self {
        Self {
            range_proof_required_for: range_proof_required_for.into_iter().collect(),
            range_proof_binds_output_script,
            range_proof_excludes_zero_when_spendable,
            surjection_proof_required_for: surjection_proof_required_for.into_iter().collect(),
        }
    }

    /// The value encodings a range proof is required for.
    ///
    /// The requirement is exact in both directions: the reviewed target
    /// also refuses a range proof attached to an encoding absent here.
    #[must_use]
    pub const fn range_proof_required_for(&self) -> &BTreeSet<EncodingClass> {
        &self.range_proof_required_for
    }

    /// Whether the range proof commits to the output script.
    #[must_use]
    pub const fn range_proof_binds_output_script(&self) -> bool {
        self.range_proof_binds_output_script
    }

    /// Whether a proven minimum of zero is refused for a spendable
    /// output.
    #[must_use]
    pub const fn range_proof_excludes_zero_when_spendable(&self) -> bool {
        self.range_proof_excludes_zero_when_spendable
    }

    /// The asset encodings a surjection proof is required for.
    #[must_use]
    pub const fn surjection_proof_required_for(&self) -> &BTreeSet<EncodingClass> {
        &self.surjection_proof_required_for
    }
}

/// The reviewed whole-transaction conservation rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConservationContract {
    form: ConservationForm,
    explicit_values_join_as_zero_blinded: bool,
    issuance_contributes_to_input_side: bool,
    unspendable_zero_output_leaves_tally: bool,
    proofs: ProofRequirements,
}

impl ConservationContract {
    /// States the conservation contract.
    #[must_use]
    pub const fn new(
        form: ConservationForm,
        explicit_values_join_as_zero_blinded: bool,
        issuance_contributes_to_input_side: bool,
        unspendable_zero_output_leaves_tally: bool,
        proofs: ProofRequirements,
    ) -> Self {
        Self {
            form,
            explicit_values_join_as_zero_blinded,
            issuance_contributes_to_input_side,
            unspendable_zero_output_leaves_tally,
            proofs,
        }
    }

    /// The shape of the balance check.
    #[must_use]
    pub const fn form(&self) -> ConservationForm {
        self.form
    }

    /// Whether an explicit amount joins the tally as a zero-blinded
    /// commitment rather than being handled separately.
    #[must_use]
    pub const fn explicit_values_join_as_zero_blinded(&self) -> bool {
        self.explicit_values_join_as_zero_blinded
    }

    /// Whether an issuance adds pseudo-inputs to the input side.
    #[must_use]
    pub const fn issuance_contributes_to_input_side(&self) -> bool {
        self.issuance_contributes_to_input_side
    }

    /// Whether an admitted zero-value explicit output is omitted from
    /// the tally rather than committed into it.
    #[must_use]
    pub const fn unspendable_zero_output_leaves_tally(&self) -> bool {
        self.unspendable_zero_output_leaves_tally
    }

    /// The proofs a blinded field must carry.
    #[must_use]
    pub const fn proofs(&self) -> &ProofRequirements {
        &self.proofs
    }
}

census_enum! {
    /// One reason the reviewed target cannot carry an authenticated public
    /// opening inside a program.
    ///
    /// These are review results, not design intent. Each names a specific
    /// missing correspondence rather than a general difficulty, so a later
    /// candidate can be judged against them one at a time.
    #[non_exhaustive]
    pub enum OpeningBlocker {
        /// No primitive maps an asset identifier to its generator.
        ///
        /// The recipe needs two curve maps and a point addition, and the
        /// reviewed language performs neither.
        GeneratorNotDerivableOnScript,
        /// The confidential encodings and the curve primitives disagree
        /// about which y a prefix selects.
        ///
        /// Commitments and generators record whether y is a quadratic
        /// residue; the curve primitives accept only the compressed
        /// public-key prefixes, which record whether y is odd. No reviewed
        /// primitive converts between them.
        EncodingDomainMismatch,
        /// Nothing binds a witness-supplied parity byte to the point the
        /// commitment names.
        ///
        /// A program can assemble an operand from an exposed x coordinate,
        /// but the parity it supplies is unchecked, so the relation holds
        /// for the point or its negation.
        SuppliedParityUnbound,
    }
}

/// What the review established about carrying an opening on-script.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpeningFeasibility {
    reachable: bool,
    blockers: BTreeSet<OpeningBlocker>,
    candidate_primitives: BTreeSet<OpcodeId>,
}

impl OpeningFeasibility {
    /// States the feasibility result.
    #[must_use]
    pub fn new(
        reachable: bool,
        blockers: impl IntoIterator<Item = OpeningBlocker>,
        candidate_primitives: impl IntoIterator<Item = OpcodeId>,
    ) -> Self {
        Self {
            reachable,
            blockers: blockers.into_iter().collect(),
            candidate_primitives: candidate_primitives.into_iter().collect(),
        }
    }

    /// Whether the reviewed language can carry a complete opening.
    #[must_use]
    pub const fn reachable(&self) -> bool {
        self.reachable
    }

    /// Every blocker the review named.
    #[must_use]
    pub const fn blockers(&self) -> &BTreeSet<OpeningBlocker> {
        &self.blockers
    }

    /// The primitives a candidate would draw on.
    ///
    /// Their presence is not a claim that a pattern exists: it is the
    /// list a later candidate must build from, and the blockers state
    /// why the list is not yet sufficient.
    #[must_use]
    pub const fn candidate_primitives(&self) -> &BTreeSet<OpcodeId> {
        &self.candidate_primitives
    }
}

/// The reviewed confidential-value facts that are not capability
/// claims.
///
/// These are target facts held apart from [`TargetDefinition`], which
/// this review does not extend: growing that schema is a versioned
/// change, and this wave states facts rather than revising a contract
/// shape.
///
/// [`TargetDefinition`]: crate::definition::TargetDefinition
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfidentialReviewFacts {
    value: ConfidentialFieldEncoding,
    asset: ConfidentialFieldEncoding,
    nonce: ConfidentialFieldEncoding,
    generator: AssetGeneratorDerivation,
    relation: CommitmentRelation,
    conservation: ConservationContract,
    opening: OpeningFeasibility,
}

impl ConfidentialReviewFacts {
    /// States the reviewed facts.
    #[must_use]
    pub const fn new(
        value: ConfidentialFieldEncoding,
        asset: ConfidentialFieldEncoding,
        nonce: ConfidentialFieldEncoding,
        generator: AssetGeneratorDerivation,
        relation: CommitmentRelation,
        conservation: ConservationContract,
        opening: OpeningFeasibility,
    ) -> Self {
        Self {
            value,
            asset,
            nonce,
            generator,
            relation,
            conservation,
            opening,
        }
    }

    /// The value field's encoding.
    #[must_use]
    pub const fn value(&self) -> ConfidentialFieldEncoding {
        self.value
    }

    /// The asset field's encoding.
    #[must_use]
    pub const fn asset(&self) -> ConfidentialFieldEncoding {
        self.asset
    }

    /// The nonce field's encoding.
    #[must_use]
    pub const fn nonce(&self) -> ConfidentialFieldEncoding {
        self.nonce
    }

    /// The asset generator recipe.
    #[must_use]
    pub const fn generator(&self) -> AssetGeneratorDerivation {
        self.generator
    }

    /// The commitment relation.
    #[must_use]
    pub const fn relation(&self) -> CommitmentRelation {
        self.relation
    }

    /// The conservation rule.
    #[must_use]
    pub const fn conservation(&self) -> &ConservationContract {
        &self.conservation
    }

    /// The on-script opening result.
    #[must_use]
    pub const fn opening(&self) -> &OpeningFeasibility {
        &self.opening
    }
}

/// Builds the reviewed confidential-value facts.
#[must_use]
pub fn reviewed_confidential_review_facts() -> ConfidentialReviewFacts {
    ConfidentialReviewFacts::new(
        // Prefix, then eight bytes of amount; or a commitment point
        // whose prefix records the squareness of its y coordinate.
        ConfidentialFieldEncoding::new(9, 33, 1, (8, 9)),
        // Prefix, then the thirty-two byte identifier; or a generator
        // under the same squareness convention, shifted by two.
        ConfidentialFieldEncoding::new(33, 33, 1, (10, 11)),
        // The nonce shares the shape but not the convention. Its
        // committed form is a point the target transports rather than
        // one it commits to, written with the standard compressed
        // pair, so its prefix records oddness. This field is the
        // reviewed table's own instance of the mismatch the opening
        // blockers rest on.
        ConfidentialFieldEncoding::new(33, 33, 1, (2, 3)),
        // Two tagged hashes over the same identifier, each mapped to
        // the curve, then added. The blinded form prepends a multiple
        // of the base point, which is why a second addition exists
        // there and not here.
        AssetGeneratorDerivation::new(32, 16, 2, 1, true),
        CommitmentRelation::new(
            CommitmentTermRoles::BlindOnBaseAmountOnAssetGenerator,
            32,
            ScalarByteOrder::BigEndian,
            true,
            true,
            true,
        ),
        ConservationContract::new(
            ConservationForm::ExactTallyToIdentity,
            true,
            true,
            true,
            ProofRequirements::new(
                [EncodingClass::ConfidentialValue],
                true,
                true,
                [EncodingClass::ConfidentialAsset],
            ),
        ),
        OpeningFeasibility::new(
            false,
            OpeningBlocker::ALL.iter().copied(),
            [
                OpcodeId::InspectInputValue,
                OpcodeId::InspectOutputValue,
                OpcodeId::InspectInputAsset,
                OpcodeId::InspectOutputAsset,
                OpcodeId::EcMulScalarVerify,
                OpcodeId::TweakVerify,
                OpcodeId::Concatenate,
            ],
        ),
    )
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
