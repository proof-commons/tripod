//! Signature, sighash, and relative-timelock dimensions.
//!
//! This module states what the target *offers*. It does not choose
//! what the project will use. No sighash profile is selected here, no
//! cadence band is encoded here, and nothing here asserts that a
//! target signature check establishes any protocol authorization: a
//! verified signature is one later evidence layer, not a proof that
//! the right party approved the right thing.

use std::collections::BTreeSet;

use crate::encoding::EncodingClass;
use crate::evidence::TargetEvidenceRequirementId;
use crate::opcode::FailureOutcome;

/// One dimension a sighash may commit to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SighashDimension {
    /// Every output of the transaction.
    AllOutputs,
    /// The single output at the signing input's index.
    SingleOutput,
    /// Every input of the transaction.
    AllInputs,
    /// Only the signing input, permitting other inputs to be added.
    InputExtensionPermitted,
    /// The issuance fields of the inputs.
    Issuance,
    /// The transaction version field.
    Version,
    /// The transaction locktime field.
    LockTime,
    /// The hash of the executing leaf.
    TapleafHash,
    /// The taproot internal key.
    InternalKey,
    /// The full set of outputs being spent.
    SpentOutputs,
}

impl SighashDimension {
    /// The complete census of sighash dimensions.
    pub const ALL: &'static [Self] = &[
        Self::AllOutputs,
        Self::SingleOutput,
        Self::AllInputs,
        Self::InputExtensionPermitted,
        Self::Issuance,
        Self::Version,
        Self::LockTime,
        Self::TapleafHash,
        Self::InternalKey,
        Self::SpentOutputs,
    ];
}

/// Which sighash dimensions the review actually established.
///
/// # Why this is split rather than a set of booleans
///
/// The source review covered the signature primitives themselves —
/// their operands, their two distinct failure modes, and their budget
/// accounting — but it did not cover the sighash type enumeration or
/// the message construction. Recording the dimensions as a set of
/// booleans would have forced a yes-or-no answer to questions the
/// review never asked, and the resulting contract would have read as
/// authoritative.
///
/// So each dimension is classified as reviewed or not reviewed, and
/// every dimension is currently *not* reviewed. Absence from the
/// reviewed set means "this package has not established it", never
/// "the target does not offer it". A downstream selector must treat an
/// unreviewed dimension as unavailable and say why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SighashCapability {
    reviewed: BTreeSet<SighashDimension>,
    unreviewed: BTreeSet<SighashDimension>,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
}

impl SighashCapability {
    /// States the sighash review.
    #[must_use]
    pub fn new(
        reviewed: impl IntoIterator<Item = SighashDimension>,
        unreviewed: impl IntoIterator<Item = SighashDimension>,
        evidence: impl IntoIterator<Item = TargetEvidenceRequirementId>,
    ) -> Self {
        Self {
            reviewed: reviewed.into_iter().collect(),
            unreviewed: unreviewed.into_iter().collect(),
            evidence: evidence.into_iter().collect(),
        }
    }

    /// The dimensions the review established.
    #[must_use]
    pub const fn reviewed(&self) -> &BTreeSet<SighashDimension> {
        &self.reviewed
    }

    /// The dimensions the review did not reach.
    #[must_use]
    pub const fn unreviewed(&self) -> &BTreeSet<SighashDimension> {
        &self.unreviewed
    }

    /// The evidence a deployment must produce for the sighash.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }

    /// Any dimension classified both ways.
    #[must_use]
    pub fn contradictory(&self) -> Option<SighashDimension> {
        self.reviewed.intersection(&self.unreviewed).next().copied()
    }

    /// Any dimension classified neither way.
    #[must_use]
    pub fn unclassified(&self) -> Option<SighashDimension> {
        SighashDimension::ALL
            .iter()
            .find(|dimension| {
                !self.reviewed.contains(dimension) && !self.unreviewed.contains(dimension)
            })
            .copied()
    }
}

/// What the target does with a public key whose encoding it does not
/// recognize.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum UnknownPublicKeyTypeRule {
    /// The check reports success *without performing any
    /// verification*, provided the offered signature is non-empty.
    ///
    /// This is the reviewed behavior and it is a sharp edge worth
    /// stating loudly: an unknown key type turns signature
    /// verification into a no-op that returns true. Any backend that
    /// relies on a signature check for authorization must constrain
    /// the key encoding itself rather than trusting the check's
    /// result. The edge is kept as an upstream friction
    /// `(´[PLAN-obs:upstream:eg-008]´)`, and this variant is why a
    /// backend has to ask which rule the deployment runs under.
    SucceedsWithoutVerification,
    /// The check refuses the key.
    Rejected,
}

/// The reviewed contract of the target's signature primitives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignaturePrimitiveContract {
    public_key_encoding: EncodingClass,
    signature_encoding: EncodingClass,
    empty_signature: FailureOutcome,
    invalid_signature: FailureOutcome,
    unknown_public_key_type: UnknownPublicKeyTypeRule,
    budget_per_check: u64,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
}

impl SignaturePrimitiveContract {
    /// States the signature primitive contract.
    #[must_use]
    pub fn new(
        public_key_encoding: EncodingClass,
        signature_encoding: EncodingClass,
        empty_signature: FailureOutcome,
        invalid_signature: FailureOutcome,
        unknown_public_key_type: UnknownPublicKeyTypeRule,
        budget_per_check: u64,
        evidence: impl IntoIterator<Item = TargetEvidenceRequirementId>,
    ) -> Self {
        Self {
            public_key_encoding,
            signature_encoding,
            empty_signature,
            invalid_signature,
            unknown_public_key_type,
            budget_per_check,
            evidence: evidence.into_iter().collect(),
        }
    }

    /// The encoding a public key operand carries.
    #[must_use]
    pub const fn public_key_encoding(&self) -> EncodingClass {
        self.public_key_encoding
    }

    /// The encoding a signature operand carries.
    #[must_use]
    pub const fn signature_encoding(&self) -> EncodingClass {
        self.signature_encoding
    }

    /// What an empty signature does.
    #[must_use]
    pub const fn empty_signature(&self) -> FailureOutcome {
        self.empty_signature
    }

    /// What a non-empty signature that does not verify does.
    #[must_use]
    pub const fn invalid_signature(&self) -> FailureOutcome {
        self.invalid_signature
    }

    /// What an unrecognized public key encoding does.
    #[must_use]
    pub const fn unknown_public_key_type(&self) -> UnknownPublicKeyTypeRule {
        self.unknown_public_key_type
    }

    /// The validation budget one check can consume.
    #[must_use]
    pub const fn budget_per_check(&self) -> u64 {
        self.budget_per_check
    }

    /// The evidence a deployment must produce for the primitive.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }
}

/// Whether a relative timelock counts blocks or elapsed time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TimelockMode {
    /// The lock counts confirmations.
    BlockHeight,
    /// The lock counts fixed time intervals.
    TimeInterval,
}

/// How a sequence field's bits are divided.
///
/// A value of its own because the three masks are meaningless apart:
/// each covers part of the same word, and a validator has to be able
/// to check that they do not overlap.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SequenceFieldLayout {
    /// The bit that disables the lock entirely.
    pub disable_flag: u32,
    /// The bit that selects block or time mode.
    pub mode_flag: u32,
    /// The bits carrying the lock's magnitude.
    pub value_mask: u32,
    /// The left shift converting a time-mode magnitude to seconds.
    pub time_interval_shift: u8,
}

impl SequenceFieldLayout {
    /// Whether any two of the fields claim the same bit, which would
    /// make them indistinguishable to a decoder.
    #[must_use]
    pub const fn has_overlapping_fields(self) -> bool {
        (self.disable_flag & self.mode_flag) != 0
            || (self.disable_flag & self.value_mask) != 0
            || (self.mode_flag & self.value_mask) != 0
    }
}

/// The reviewed relative-timelock dimensions.
///
/// These are target facts. The protocol's cadence band — the window in
/// which an operator acts and after which anyone may — is not encoded
/// here and must not be: a downstream planner builds that band out of
/// these primitives, and the target has no opinion about it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelativeTimelockContract {
    modes: BTreeSet<TimelockMode>,
    layout: SequenceFieldLayout,
    minimum_transaction_version: u32,
    unsatisfied: FailureOutcome,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
}

impl RelativeTimelockContract {
    /// States the relative-timelock contract.
    #[must_use]
    pub fn new(
        modes: impl IntoIterator<Item = TimelockMode>,
        layout: SequenceFieldLayout,
        minimum_transaction_version: u32,
        unsatisfied: FailureOutcome,
        evidence: impl IntoIterator<Item = TargetEvidenceRequirementId>,
    ) -> Self {
        Self {
            modes: modes.into_iter().collect(),
            layout,
            minimum_transaction_version,
            unsatisfied,
            evidence: evidence.into_iter().collect(),
        }
    }

    /// How the sequence field's bits are divided.
    #[must_use]
    pub const fn layout(&self) -> SequenceFieldLayout {
        self.layout
    }

    /// The modes the target supports.
    #[must_use]
    pub const fn modes(&self) -> &BTreeSet<TimelockMode> {
        &self.modes
    }

    /// The sequence bit that disables the lock entirely.
    #[must_use]
    pub const fn sequence_disable_flag(&self) -> u32 {
        self.layout.disable_flag
    }

    /// The sequence bit that selects block or time mode.
    #[must_use]
    pub const fn sequence_mode_flag(&self) -> u32 {
        self.layout.mode_flag
    }

    /// The sequence bits carrying the lock's magnitude.
    #[must_use]
    pub const fn sequence_value_mask(&self) -> u32 {
        self.layout.value_mask
    }

    /// The left shift converting a time-mode magnitude to seconds.
    #[must_use]
    pub const fn time_interval_shift(&self) -> u8 {
        self.layout.time_interval_shift
    }

    /// The lowest transaction version at which the lock is enforced.
    ///
    /// Below this version the check does not merely pass — the lock is
    /// not enforced at all, so a program relying on it must constrain
    /// the version too.
    #[must_use]
    pub const fn minimum_transaction_version(&self) -> u32 {
        self.minimum_transaction_version
    }

    /// What an unsatisfied lock does.
    #[must_use]
    pub const fn unsatisfied(&self) -> FailureOutcome {
        self.unsatisfied
    }

    /// The evidence a deployment must produce for the lock.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }

    /// Whether the flag and mask bits overlap, which would make the
    /// fields indistinguishable.
    #[must_use]
    pub const fn has_overlapping_fields(&self) -> bool {
        self.layout.has_overlapping_fields()
    }
}

/// The target's authorization dimensions, taken together.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorizationContract {
    signature: SignaturePrimitiveContract,
    sighash: SighashCapability,
    relative_timelock: RelativeTimelockContract,
}

impl AuthorizationContract {
    /// States the authorization contract.
    #[must_use]
    pub const fn new(
        signature: SignaturePrimitiveContract,
        sighash: SighashCapability,
        relative_timelock: RelativeTimelockContract,
    ) -> Self {
        Self {
            signature,
            sighash,
            relative_timelock,
        }
    }

    /// The signature primitive contract.
    #[must_use]
    pub const fn signature(&self) -> &SignaturePrimitiveContract {
        &self.signature
    }

    /// The sighash dimensions.
    #[must_use]
    pub const fn sighash(&self) -> &SighashCapability {
        &self.sighash
    }

    /// The relative-timelock dimensions.
    #[must_use]
    pub const fn relative_timelock(&self) -> &RelativeTimelockContract {
        &self.relative_timelock
    }
}

/// Builds the reviewed authorization contract.
pub(crate) fn reviewed_authorization() -> AuthorizationContract {
    use TargetEvidenceRequirementId as R;

    AuthorizationContract::new(
        SignaturePrimitiveContract::new(
            EncodingClass::XOnlyPublicKey,
            EncodingClass::SchnorrSignature,
            // An empty signature consumes the operands and pushes a
            // false, which a program can branch on.
            FailureOutcome::ConsumeOperandsPushFalse,
            // A non-empty signature that does not verify aborts. There
            // is no branchable "verification failed".
            FailureOutcome::AbortEvaluation,
            UnknownPublicKeyTypeRule::SucceedsWithoutVerification,
            crate::opcode::VALIDATION_BUDGET_PER_CHECK,
            [R::SignatureSemantics],
        ),
        // Every dimension is unreviewed. The review reached the
        // signature primitives but not the sighash construction, and
        // saying so is the only honest option available.
        SighashCapability::new(
            [],
            SighashDimension::ALL.iter().copied(),
            [R::SighashSemantics],
        ),
        RelativeTimelockContract::new(
            [TimelockMode::BlockHeight, TimelockMode::TimeInterval],
            SequenceFieldLayout {
                disable_flag: 0x8000_0000,
                mode_flag: 0x0040_0000,
                value_mask: 0x0000_ffff,
                time_interval_shift: 9,
            },
            2,
            FailureOutcome::AbortEvaluation,
            [R::RelativeTimelockSemantics],
        ),
    )
}
