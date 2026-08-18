//! Target capabilities and their prerequisite structure.
//!
//! # A capability names a target fact, never a protocol proof
//!
//! Every name below describes something the target can do. None of
//! them describes an attestation-contract obligation being met. That a
//! program *can* inspect an output's asset says nothing about whether
//! object recognition, family closure, canonical partitioning, or
//! permissionless constructibility has been proved; those are
//! downstream questions and this vocabulary must not be read as
//! answering them.
//!
//! # Support is not a boolean
//!
//! [`StaticCapabilityStatus`] distinguishes a reviewed capability from
//! an incomplete one and from an unsupported one, and none of the
//! three means "deployment-evidenced". `Reviewed` means the typed
//! static contract has been reviewed against upstream source. It does
//! not mean a node was ever asked.

use std::collections::{BTreeMap, BTreeSet};

use crate::encoding::EncodingClass;
use crate::evidence::TargetEvidenceRequirementId;
use crate::opcode::OpcodeId;

/// A stable key naming one target capability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ElementsCapability {
    /// Scripts execute in the reviewed domain at all.
    TapscriptExecution,
    /// The reviewed leaf version selects that domain.
    RequiredLeafVersion,

    /// A program can read the input count.
    InputCountInspection,
    /// A program can read the output count.
    OutputCountInspection,
    /// A program can read the index of the input being validated.
    CurrentInputIndexInspection,

    /// A program can read an input's outpoint.
    InputOutpointInspection,
    /// A program can read a spent output's asset.
    InputAssetInspection,
    /// A program can read a spent output's value.
    InputValueInspection,
    /// A program can read a spent output's program.
    InputProgramInspection,
    /// A program can read an input's sequence.
    InputSequenceInspection,
    /// A program can read an input's issuance fields.
    InputIssuanceInspection,

    /// A program can read an output's asset.
    OutputAssetInspection,
    /// A program can read an output's value.
    OutputValueInspection,
    /// A program can read an output's nonce.
    OutputNonceInspection,
    /// A program can read an output's program.
    OutputProgramInspection,

    /// A program can read the transaction version.
    TransactionVersionInspection,
    /// A program can read the transaction locktime.
    TransactionLockTimeInspection,
    /// A program can read the transaction weight.
    TransactionWeightInspection,

    /// A program can add, subtract, multiply, divide, and negate
    /// signed fixed-width values.
    SignedFixedWidthArithmetic,
    /// A program can order signed fixed-width values.
    SignedFixedWidthComparison,
    /// A program can convert between the script number and
    /// fixed-width forms.
    ScriptNumberConversion,

    /// A program can hash a message in streamed chunks.
    StreamingSha256,
    /// A program can verify a scalar multiplication.
    EcScalarVerification,
    /// A program can verify a pay-to-contract tweak.
    TweakVerification,

    /// A program can verify a signature over the transaction sighash.
    SignatureVerification,
    /// A program can verify a signature over a message it supplies.
    StackMessageSignatureVerification,
    /// A signature can be made to commit to the transaction's outputs.
    OutputCommittingSighash,
    /// A signature can be made to control which inputs may be added.
    InputCommitmentControl,
    /// A program can require a relative timelock.
    RelativeTimelock,

    /// A program can duplicate, reorder, and discard stack items.
    StackRearrangement,
    /// A program can compare two items for byte equality.
    ByteStringEquality,
    /// A program can require a truth value and abort otherwise.
    BooleanVerification,
    /// A program can join two items into one.
    ByteStringConcatenation,
    /// A program can read the width of an item.
    ByteStringWidth,
    /// A program can extract a slice of an item.
    ByteStringSlicing,
    /// A program can combine two equal-width items bit by bit.
    BitwiseByteLogic,
    /// A program can order two byte strings lexicographically with one
    /// reviewed primitive.
    ///
    /// No such primitive exists. The capability is named so that the
    /// absence is a stated, evidenced row rather than a gap a reader
    /// has to notice, and so that a construction depending on it must
    /// confront the status instead of assuming a familiar opcode
    /// `(´[PLAN-rule:guide10:primitive-admission]´)`.
    CanonicalByteOrdering,

    /// The target's rules conserve value across a transaction.
    ConfidentialValueConservation,
    /// A program can establish equality of two commitments.
    CommitmentEquality,
    /// A program can bind a commitment to a claimed amount.
    AuthenticatedValueOpening,
    /// A program can read amounts carried in the clear.
    ExplicitValueInspection,
    /// A program can read issuance fields.
    IssuanceIntrospection,
    /// A program can distinguish a reissuance from an issuance.
    ReissuanceIntrospection,

    /// The target's own resource bounds are known.
    ConsensusResourceLimits,
    /// A deployment's stricter bounds are known.
    PolicyResourceLimits,
}

impl ElementsCapability {
    /// The complete census of target capabilities.
    pub const ALL: &'static [Self] = &[
        Self::TapscriptExecution,
        Self::RequiredLeafVersion,
        Self::InputCountInspection,
        Self::OutputCountInspection,
        Self::CurrentInputIndexInspection,
        Self::InputOutpointInspection,
        Self::InputAssetInspection,
        Self::InputValueInspection,
        Self::InputProgramInspection,
        Self::InputSequenceInspection,
        Self::InputIssuanceInspection,
        Self::OutputAssetInspection,
        Self::OutputValueInspection,
        Self::OutputNonceInspection,
        Self::OutputProgramInspection,
        Self::TransactionVersionInspection,
        Self::TransactionLockTimeInspection,
        Self::TransactionWeightInspection,
        Self::SignedFixedWidthArithmetic,
        Self::SignedFixedWidthComparison,
        Self::ScriptNumberConversion,
        Self::StreamingSha256,
        Self::EcScalarVerification,
        Self::TweakVerification,
        Self::SignatureVerification,
        Self::StackMessageSignatureVerification,
        Self::OutputCommittingSighash,
        Self::InputCommitmentControl,
        Self::RelativeTimelock,
        Self::StackRearrangement,
        Self::ByteStringEquality,
        Self::BooleanVerification,
        Self::ByteStringConcatenation,
        Self::ByteStringWidth,
        Self::ByteStringSlicing,
        Self::BitwiseByteLogic,
        Self::CanonicalByteOrdering,
        Self::ConfidentialValueConservation,
        Self::CommitmentEquality,
        Self::AuthenticatedValueOpening,
        Self::ExplicitValueInspection,
        Self::IssuanceIntrospection,
        Self::ReissuanceIntrospection,
        Self::ConsensusResourceLimits,
        Self::PolicyResourceLimits,
    ];
}

/// What the static review established about one capability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum StaticCapabilityStatus {
    /// The typed static contract has been reviewed against upstream
    /// source. This is not deployment evidence.
    Reviewed,
    /// The target offers something toward it, but the review did not
    /// establish enough to rely on it.
    Incomplete,
    /// No reviewed mechanism establishes it.
    Unsupported,
}

impl StaticCapabilityStatus {
    /// The status's place in the total order
    /// `Unsupported < Incomplete < Reviewed`.
    ///
    /// # Why this is not the derived ordering
    ///
    /// The derived comparison follows declaration order, which runs
    /// the other way and would silently invert every closure check.
    /// The strength order is a semantic decision, so it is written out
    /// here and reordering the variants for readability cannot change
    /// what the validator enforces.
    #[must_use]
    pub const fn strength(self) -> u8 {
        match self {
            Self::Unsupported => 0,
            Self::Incomplete => 1,
            Self::Reviewed => 2,
        }
    }

    /// Whether this status is no stronger than `other`.
    #[must_use]
    pub const fn at_most(self, other: Self) -> bool {
        self.strength() <= other.strength()
    }
}

/// The complete typed contract of one capability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityContract {
    capability: ElementsCapability,
    prerequisites: BTreeSet<ElementsCapability>,
    opcodes: BTreeSet<OpcodeId>,
    encodings: BTreeSet<EncodingClass>,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
    status: StaticCapabilityStatus,
}

impl CapabilityContract {
    /// States one capability contract.
    #[must_use]
    pub fn new(
        capability: ElementsCapability,
        prerequisites: impl IntoIterator<Item = ElementsCapability>,
        opcodes: impl IntoIterator<Item = OpcodeId>,
        encodings: impl IntoIterator<Item = EncodingClass>,
        evidence: impl IntoIterator<Item = TargetEvidenceRequirementId>,
        status: StaticCapabilityStatus,
    ) -> Self {
        Self {
            capability,
            prerequisites: prerequisites.into_iter().collect(),
            opcodes: opcodes.into_iter().collect(),
            encodings: encodings.into_iter().collect(),
            evidence: evidence.into_iter().collect(),
            status,
        }
    }

    /// The capability this contract describes.
    #[must_use]
    pub const fn capability(&self) -> ElementsCapability {
        self.capability
    }

    /// The capabilities it depends upon.
    #[must_use]
    pub const fn prerequisites(&self) -> &BTreeSet<ElementsCapability> {
        &self.prerequisites
    }

    /// The primitives it is built from.
    #[must_use]
    pub const fn opcodes(&self) -> &BTreeSet<OpcodeId> {
        &self.opcodes
    }

    /// The encodings it depends upon.
    #[must_use]
    pub const fn encodings(&self) -> &BTreeSet<EncodingClass> {
        &self.encodings
    }

    /// The evidence a deployment must produce for it.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }

    /// What the static review established.
    #[must_use]
    pub const fn status(&self) -> StaticCapabilityStatus {
        self.status
    }
}

/// Reports the capabilities caught in a prerequisite cycle.
///
/// # Why this reports a residual rather than each strongly connected
/// component separately
///
/// The crate depends on no graph library and will not acquire one to
/// answer a question this narrow. Kahn's algorithm settles the
/// question exactly: the capabilities it cannot emit are precisely
/// those on a cycle or depending on one, which is the complete set a
/// reviewer must look at. Decomposing that set into individual
/// components would name the same capabilities in smaller groups
/// without changing what has to be fixed.
///
/// Returns an empty vector when the prerequisite relation is acyclic.
#[must_use]
pub fn prerequisite_cycle_residual(
    contracts: &BTreeMap<ElementsCapability, CapabilityContract>,
) -> Vec<ElementsCapability> {
    let mut remaining: BTreeMap<ElementsCapability, BTreeSet<ElementsCapability>> = contracts
        .iter()
        .map(|(key, contract)| {
            let pending = contract
                .prerequisites()
                .iter()
                .filter(|prerequisite| contracts.contains_key(prerequisite))
                .copied()
                .collect();
            (*key, pending)
        })
        .collect();

    loop {
        let ready: Vec<ElementsCapability> = remaining
            .iter()
            .filter(|(_, pending)| pending.is_empty())
            .map(|(key, _)| *key)
            .collect();

        if ready.is_empty() {
            // Whatever is left cannot be ordered, so it is on a cycle
            // or reaches one.
            return remaining.into_keys().collect();
        }

        for key in ready {
            remaining.remove(&key);
            for pending in remaining.values_mut() {
                pending.remove(&key);
            }
        }
    }
}

/// Every capability reachable from `capability` by following
/// prerequisites, excluding the capability itself.
///
/// Cycle-safe: a capability already visited is not expanded again, so
/// a cyclic registry yields a finite set rather than looping. The
/// cycle itself is reported separately by
/// [`prerequisite_cycle_residual`].
#[must_use]
pub fn transitive_prerequisites(
    contracts: &BTreeMap<ElementsCapability, CapabilityContract>,
    capability: ElementsCapability,
) -> BTreeSet<ElementsCapability> {
    let mut reached: BTreeSet<ElementsCapability> = BTreeSet::new();
    let mut pending: Vec<ElementsCapability> = contracts
        .get(&capability)
        .map(|contract| contract.prerequisites().iter().copied().collect())
        .unwrap_or_default();

    while let Some(next) = pending.pop() {
        if next == capability || !reached.insert(next) {
            continue;
        }
        if let Some(contract) = contracts.get(&next) {
            pending.extend(contract.prerequisites().iter().copied());
        }
    }

    reached
}

/// Reports each capability whose status exceeds a transitive
/// prerequisite's, paired with the weakest prerequisite responsible.
///
/// # Why the status must be closed and not merely resolvable
///
/// A capability is a claim that a program can do something. Realizing
/// it needs everything it depends upon, so a row marked reviewed above
/// a prerequisite marked unsupported is not a strong claim standing on
/// a weak one — it is a claim that cannot be true. The existing
/// prerequisite checks establish that the edges resolve and that the
/// graph can be ordered; neither says anything about strength.
///
/// One offending prerequisite is named per capability. Naming every
/// one would list the same repair several times over, and the weakest
/// is the one that has to move first.
#[must_use]
pub fn status_closure_violations(
    contracts: &BTreeMap<ElementsCapability, CapabilityContract>,
) -> Vec<(ElementsCapability, ElementsCapability)> {
    let mut violations = Vec::new();

    for (capability, contract) in contracts {
        let offender = transitive_prerequisites(contracts, *capability)
            .into_iter()
            .filter_map(|prerequisite| {
                let status = contracts.get(&prerequisite)?.status();
                (!contract.status().at_most(status)).then_some((status.strength(), prerequisite))
            })
            .min();

        if let Some((_, prerequisite)) = offender {
            violations.push((*capability, prerequisite));
        }
    }

    violations
}

/// Builds the reviewed capability registry.
/// Builds one capability contract.
fn entry(
    capability: ElementsCapability,
    prerequisites: &[ElementsCapability],
    opcodes: &[OpcodeId],
    encodings: &[EncodingClass],
    evidence: &[TargetEvidenceRequirementId],
    status: StaticCapabilityStatus,
) -> (ElementsCapability, CapabilityContract) {
    (
        capability,
        CapabilityContract::new(
            capability,
            prerequisites.iter().copied(),
            opcodes.iter().copied(),
            encodings.iter().copied(),
            evidence.iter().copied(),
            status,
        ),
    )
}

/// Every capability below the domain requires the domain, and the
/// domain requires the leaf version that selects it. Stating the edge
/// in that direction once keeps the orientation unambiguous: an edge
/// runs from a capability to what it requires.
const BASE: &[ElementsCapability] = &[ElementsCapability::TapscriptExecution];

/// The domain, the leaf version, and input introspection.
fn foundation_and_input_capabilities() -> Vec<(ElementsCapability, CapabilityContract)> {
    use ElementsCapability as P;
    use EncodingClass as E;
    use OpcodeId as O;
    use StaticCapabilityStatus::Reviewed;
    use TargetEvidenceRequirementId as R;

    let base = BASE;

    vec![
        entry(
            P::RequiredLeafVersion,
            &[],
            &[],
            &[],
            &[R::LeafVersionActivation],
            Reviewed,
        ),
        entry(
            P::TapscriptExecution,
            &[P::RequiredLeafVersion],
            &[],
            &[],
            &[R::TapscriptExecutionDomain],
            Reviewed,
        ),
        entry(
            P::InputCountInspection,
            base,
            &[O::InspectNumInputs],
            &[E::ScriptNumber],
            &[R::TransactionIntrospectionSemantics],
            Reviewed,
        ),
        entry(
            P::OutputCountInspection,
            base,
            &[O::InspectNumOutputs],
            &[E::ScriptNumber],
            &[R::TransactionIntrospectionSemantics],
            Reviewed,
        ),
        entry(
            P::CurrentInputIndexInspection,
            base,
            &[O::PushCurrentInputIndex],
            &[E::ScriptNumber],
            &[R::InputIntrospectionSemantics],
            Reviewed,
        ),
        entry(
            P::InputOutpointInspection,
            base,
            &[O::InspectInputOutpoint],
            &[E::OutPointTxid, E::OutPointIndex, E::OutPointFlags],
            &[R::InputIntrospectionSemantics, R::EncodingSemantics],
            Reviewed,
        ),
        entry(
            P::InputAssetInspection,
            base,
            &[O::InspectInputAsset],
            &[E::ExplicitAsset, E::ConfidentialAsset],
            &[R::InputIntrospectionSemantics, R::EncodingSemantics],
            Reviewed,
        ),
        entry(
            P::InputValueInspection,
            base,
            &[O::InspectInputValue],
            &[E::ExplicitValue, E::ConfidentialValue],
            &[R::InputIntrospectionSemantics, R::EncodingSemantics],
            Reviewed,
        ),
        entry(
            P::InputProgramInspection,
            base,
            &[O::InspectInputScriptPubKey],
            &[E::WitnessProgram, E::ScriptPubKeySha256],
            &[R::InputIntrospectionSemantics, R::EncodingSemantics],
            Reviewed,
        ),
        entry(
            P::InputSequenceInspection,
            base,
            &[O::InspectInputSequence],
            &[E::Sequence],
            &[R::InputIntrospectionSemantics],
            Reviewed,
        ),
    ]
}

/// Output and transaction introspection.
fn output_and_transaction_capabilities() -> Vec<(ElementsCapability, CapabilityContract)> {
    use ElementsCapability as P;
    use EncodingClass as E;
    use OpcodeId as O;
    use StaticCapabilityStatus::Reviewed;
    use TargetEvidenceRequirementId as R;

    let base = BASE;

    vec![
        entry(
            P::InputIssuanceInspection,
            base,
            &[O::InspectInputIssuance],
            &[
                E::ExplicitValue,
                E::NullValue,
                E::IssuanceEntropy,
                E::IssuanceBlindingNonce,
            ],
            &[R::InputIntrospectionSemantics, R::IssuanceIntrospection],
            Reviewed,
        ),
        entry(
            P::OutputAssetInspection,
            base,
            &[O::InspectOutputAsset],
            &[E::ExplicitAsset, E::ConfidentialAsset],
            &[R::OutputIntrospectionSemantics, R::EncodingSemantics],
            Reviewed,
        ),
        entry(
            P::OutputValueInspection,
            base,
            &[O::InspectOutputValue],
            &[E::ExplicitValue, E::ConfidentialValue],
            &[R::OutputIntrospectionSemantics, R::EncodingSemantics],
            Reviewed,
        ),
        entry(
            P::OutputNonceInspection,
            base,
            &[O::InspectOutputNonce],
            &[E::ExplicitNonce, E::ConfidentialNonce, E::NullNonce],
            &[R::OutputIntrospectionSemantics, R::EncodingSemantics],
            Reviewed,
        ),
        entry(
            P::OutputProgramInspection,
            base,
            &[O::InspectOutputScriptPubKey],
            &[E::WitnessProgram, E::ScriptPubKeySha256],
            &[R::OutputIntrospectionSemantics, R::EncodingSemantics],
            Reviewed,
        ),
        entry(
            P::TransactionVersionInspection,
            base,
            &[O::InspectVersion],
            &[E::UnsignedLittleEndian32],
            &[R::TransactionIntrospectionSemantics],
            Reviewed,
        ),
        entry(
            P::TransactionLockTimeInspection,
            base,
            &[O::InspectLockTime],
            &[E::UnsignedLittleEndian32],
            &[R::TransactionIntrospectionSemantics],
            Reviewed,
        ),
        entry(
            P::TransactionWeightInspection,
            base,
            &[O::TxWeight],
            &[E::UnsignedLittleEndian64],
            &[R::TransactionIntrospectionSemantics],
            Reviewed,
        ),
        entry(
            P::SignedFixedWidthArithmetic,
            base,
            &[O::Add64, O::Sub64, O::Mul64, O::Div64, O::Neg64],
            &[E::SignedLittleEndian64],
            &[R::ArithmeticSemantics],
            Reviewed,
        ),
        entry(
            P::SignedFixedWidthComparison,
            base,
            &[
                O::LessThan64,
                O::LessThanOrEqual64,
                O::GreaterThan64,
                O::GreaterThanOrEqual64,
            ],
            &[E::SignedLittleEndian64],
            &[R::ComparisonSemantics],
            Reviewed,
        ),
    ]
}

/// Arithmetic, comparison, conversion, hashing, and curve checks.
fn computation_capabilities() -> Vec<(ElementsCapability, CapabilityContract)> {
    use ElementsCapability as P;
    use EncodingClass as E;
    use OpcodeId as O;
    use StaticCapabilityStatus::{Incomplete, Reviewed};
    use TargetEvidenceRequirementId as R;

    let base = BASE;

    vec![
        entry(
            P::ScriptNumberConversion,
            base,
            &[O::ScriptNumToLe64, O::Le64ToScriptNum, O::Le32ToLe64],
            &[
                E::ScriptNumber,
                E::SignedLittleEndian64,
                E::UnsignedLittleEndian32,
            ],
            &[R::ConversionSemantics],
            Reviewed,
        ),
        entry(
            P::StreamingSha256,
            base,
            &[O::Sha256Initialize, O::Sha256Update, O::Sha256Finalize],
            &[E::Sha256Context, E::Sha256Digest],
            &[R::StreamingHashSemantics],
            Reviewed,
        ),
        entry(
            P::EcScalarVerification,
            base,
            &[O::EcMulScalarVerify],
            &[E::CompressedPublicKey, E::EcScalar],
            &[R::EllipticCurveSemantics],
            Reviewed,
        ),
        entry(
            P::TweakVerification,
            base,
            &[O::TweakVerify],
            &[E::CompressedPublicKey, E::XOnlyPublicKey, E::TaprootTweak],
            &[R::EllipticCurveSemantics],
            Reviewed,
        ),
        entry(
            P::SignatureVerification,
            base,
            &[O::CheckSig, O::CheckSigVerify],
            &[E::XOnlyPublicKey, E::SchnorrSignature],
            &[R::SignatureSemantics],
            Reviewed,
        ),
        entry(
            P::StackMessageSignatureVerification,
            base,
            &[O::CheckSigFromStack, O::CheckSigFromStackVerify],
            &[E::XOnlyPublicKey, E::SchnorrSignature],
            &[R::SignatureSemantics],
            Reviewed,
        ),
        // The signature primitives were reviewed; the sighash
        // construction was not. Marking these Reviewed because the
        // check exists would be exactly the inference the contract is
        // shaped to prevent.
        entry(
            P::OutputCommittingSighash,
            &[P::SignatureVerification],
            &[O::CheckSig],
            &[E::SchnorrSignature],
            &[R::SighashSemantics],
            Incomplete,
        ),
        entry(
            P::InputCommitmentControl,
            &[P::SignatureVerification],
            &[O::CheckSig],
            &[E::SchnorrSignature],
            &[R::SighashSemantics],
            Incomplete,
        ),
        entry(
            P::RelativeTimelock,
            &[P::TapscriptExecution, P::TransactionVersionInspection],
            &[O::CheckSequenceVerify],
            &[E::LockTimeScriptNumber, E::Sequence],
            &[R::RelativeTimelockSemantics],
            Reviewed,
        ),
        // An external consensus claim: no primitive demonstrates it.
        entry(
            P::ConfidentialValueConservation,
            &[],
            &[],
            &[E::ExplicitValue, E::ConfidentialValue],
            &[R::ConfidentialValueConservation],
            Incomplete,
        ),
    ]
}

/// Signatures, timelocks, confidential values, issuance, and resources.
fn authorization_and_value_capabilities() -> Vec<(ElementsCapability, CapabilityContract)> {
    use ElementsCapability as P;
    use EncodingClass as E;
    use OpcodeId as O;
    use StaticCapabilityStatus::{Reviewed, Unsupported};
    use TargetEvidenceRequirementId as R;

    vec![
        entry(
            P::CommitmentEquality,
            &[],
            &[],
            &[E::ConfidentialValue],
            &[R::CommitmentEquality],
            Unsupported,
        ),
        // Never complete. The curve and hash primitives exist; that is
        // not an opening proof.
        entry(
            P::AuthenticatedValueOpening,
            &[P::EcScalarVerification, P::StreamingSha256],
            &[],
            &[E::ConfidentialValue],
            &[R::CommitmentEquality],
            Unsupported,
        ),
        entry(
            P::ExplicitValueInspection,
            &[P::InputValueInspection, P::OutputValueInspection],
            &[O::InspectInputValue, O::InspectOutputValue],
            &[E::ExplicitValue],
            &[R::EncodingSemantics],
            Reviewed,
        ),
        entry(
            P::IssuanceIntrospection,
            &[P::InputIssuanceInspection],
            &[O::InspectInputIssuance],
            &[E::IssuanceEntropy, E::IssuanceBlindingNonce],
            &[R::IssuanceIntrospection],
            Reviewed,
        ),
        entry(
            P::ReissuanceIntrospection,
            &[P::IssuanceIntrospection],
            &[O::InspectInputIssuance],
            &[E::IssuanceBlindingNonce],
            &[R::IssuanceIntrospection],
            Reviewed,
        ),
        entry(
            P::ConsensusResourceLimits,
            &[],
            &[],
            &[],
            &[R::ConsensusResourceLimits],
            Reviewed,
        ),
        entry(
            P::PolicyResourceLimits,
            &[],
            &[],
            &[],
            &[R::PolicyResourceLimits],
            Reviewed,
        ),
    ]
}

/// The compound-proof capabilities: stack rearrangement, equality,
/// verification, and the byte-string operations.
///
/// # The one that is not there
///
/// [`ElementsCapability::CanonicalByteOrdering`] names no primitive and
/// is [`StaticCapabilityStatus::Unsupported`]. The reviewed target has
/// no byte-lexicographic comparison: its ordering primitives read
/// fixed-width signed integers, and its script-number ordering reads a
/// number, neither of which orders a thirty-two byte digest. A
/// construction needing canonical ordering must build it from the
/// primitives that do exist and prove the construction, which is a
/// different claim from having the capability
/// `(´[PLAN-rule:guide10:tapbranch-order]´)`.
fn compound_proof_capabilities() -> Vec<(ElementsCapability, CapabilityContract)> {
    use ElementsCapability as P;
    use OpcodeId as O;
    use StaticCapabilityStatus::{Reviewed, Unsupported};
    use TargetEvidenceRequirementId as R;

    let base = BASE;

    vec![
        entry(
            P::StackRearrangement,
            base,
            &[
                O::Duplicate,
                O::DuplicateTwo,
                O::CopyOver,
                O::Swap,
                O::Rotate,
                O::RemoveSecond,
                O::Tuck,
                O::Drop,
                O::DropTwo,
            ],
            &[],
            &[R::StackRearrangementSemantics],
            Reviewed,
        ),
        entry(
            P::ByteStringEquality,
            base,
            &[O::Equal, O::EqualVerify],
            &[],
            &[R::VerificationSemantics],
            Reviewed,
        ),
        entry(
            P::BooleanVerification,
            base,
            &[O::Verify],
            &[],
            &[R::VerificationSemantics],
            Reviewed,
        ),
        entry(
            P::ByteStringConcatenation,
            base,
            &[O::Concatenate],
            &[],
            &[R::ByteStringSemantics],
            Reviewed,
        ),
        entry(
            P::ByteStringWidth,
            base,
            &[O::Size],
            &[],
            &[R::ByteStringSemantics],
            Reviewed,
        ),
        entry(
            P::ByteStringSlicing,
            base,
            &[O::Substring],
            &[],
            &[R::ByteStringSemantics],
            Reviewed,
        ),
        entry(
            P::BitwiseByteLogic,
            base,
            &[O::BitwiseAnd, O::BitwiseXor],
            &[],
            &[R::ByteStringSemantics],
            Reviewed,
        ),
        entry(
            P::CanonicalByteOrdering,
            base,
            &[],
            &[],
            &[R::ByteStringSemantics],
            Unsupported,
        ),
    ]
}

/// Builds the reviewed capability registry.
pub(crate) fn reviewed_capabilities() -> BTreeMap<ElementsCapability, CapabilityContract> {
    [
        foundation_and_input_capabilities(),
        output_and_transaction_capabilities(),
        computation_capabilities(),
        authorization_and_value_capabilities(),
        compound_proof_capabilities(),
    ]
    .into_iter()
    .flatten()
    .collect()
}
