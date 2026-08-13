//! Reviewed target primitives and their complete typed contracts.
//!
//! Each primitive admitted here states its operands, its successful
//! results, *and* its failure effects. Describing an opcode only by
//! what it does when it succeeds would be the most dangerous kind of
//! incomplete contract, because the reviewed target does not fail
//! uniformly: some primitives abort script evaluation, some consume
//! their operands and push a false, and the fixed-width arithmetic
//! primitives leave their operands in place and push a false above
//! them. A backend that assumed a uniform failure mode would emit
//! programs whose stack depth is wrong exactly on the failing path.
//!
//! # Opcode bytes are data
//!
//! An identity's position in the [`OpcodeId`] declaration is not its
//! target byte. The byte lives in the specification and is validated
//! against the reviewed contract; a test states the expected bytes
//! independently rather than reading them back out of this registry.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;

use crate::encoding::{ByteOrder, EncodingClass};
use crate::evidence::TargetEvidenceRequirementId;

/// The script execution domain a primitive is available in.
///
/// Only the reviewed domain is declared. Legacy script, segwit v0, and
/// the separate Simplicity leaf are deliberately absent: this package
/// has not reviewed them, and declaring a domain it cannot describe
/// would be a claim rather than a contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ExecutionDomain {
    /// The taproot script-path execution domain.
    Tapscript,
}

impl ExecutionDomain {
    /// The complete census of reviewed execution domains.
    pub const ALL: &'static [Self] = &[Self::Tapscript];
}

/// A validated tapleaf version byte.
///
/// Not every byte is representable merely because the target field is
/// one byte wide. A leaf version selects the semantics of everything
/// executed beneath it, so accepting an arbitrary byte would let a
/// caller construct a contract describing semantics this package never
/// reviewed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LeafVersion(u8);

impl LeafVersion {
    /// The reviewed tapscript leaf version.
    ///
    /// This value is target-specific and is *not* the corresponding
    /// upstream Bitcoin constant. Transcribing the Bitcoin value here
    /// would have produced a contract that validates cleanly and
    /// describes the wrong leaf.
    pub const TAPSCRIPT: Self = Self(0xc4);

    /// Every leaf version this contract describes.
    pub const REVIEWED: &'static [Self] = &[Self::TAPSCRIPT];

    /// Accepts a leaf version byte this contract has reviewed.
    ///
    /// # Errors
    ///
    /// Returns [`TargetError::UnreviewedLeafVersion`] for any other
    /// byte.
    ///
    /// [`TargetError::UnreviewedLeafVersion`]:
    ///     crate::error::TargetError::UnreviewedLeafVersion
    pub fn new(value: u8) -> Result<Self, crate::error::TargetError> {
        let candidate = Self(value);
        if Self::REVIEWED.contains(&candidate) {
            Ok(candidate)
        } else {
            Err(crate::error::TargetError::UnreviewedLeafVersion { offered: value })
        }
    }

    /// The leaf version byte.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// The target-level representation of one stack item.
///
/// These are target stack shapes. No attestation-contract object,
/// receipt, owner, or settlement value appears here or may be
/// introduced here; those are protocol semantics owned upstream.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum StackValueType {
    /// A truth value in the target's canonical form.
    Bool,
    /// The script language's variable-width signed number.
    ScriptNumber,
    /// An unconstrained byte string within an inclusive width range.
    Bytes {
        /// The smallest admissible width.
        minimum: usize,
        /// The largest admissible width.
        maximum: usize,
    },
    /// A signed integer of exactly the given width.
    SignedFixedWidth {
        /// The exact width in bytes.
        bytes: NonZeroUsize,
        /// The order of those bytes.
        byte_order: ByteOrder,
    },
    /// An unsigned integer of exactly the given width.
    UnsignedFixedWidth {
        /// The exact width in bytes.
        bytes: NonZeroUsize,
        /// The order of those bytes.
        byte_order: ByteOrder,
    },
    /// A whole encoded field, prefix byte included, as one item.
    Encoded(EncodingClass),
    /// The payload of an encoded field, pushed *without* its prefix
    /// byte as its own item.
    ///
    /// The reviewed asset and value introspection primitives split a
    /// field this way, pushing the payload and then the prefix as two
    /// separate stack items. The nonce primitive does not. Collapsing
    /// the two shapes into one would misstate the stack depth of every
    /// program that inspects an asset or a value.
    EncodedPayload(EncodingClass),
    /// The prefix byte of an encoded field, pushed as its own item.
    EncodingPrefix(EncodingClass),
    /// The target's empty stack item, used as both a false and an
    /// absent-field marker.
    Empty,
}

/// Why a target primitive failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FailureCause {
    /// Fewer operands were present than the primitive consumes.
    StackUnderflow,
    /// An operand was not the exact width the primitive requires.
    InvalidOperandWidth,
    /// A script-number operand exceeded the admissible width, or was
    /// not minimally encoded where minimality is enforced.
    MalformedScriptNumber,
    /// A result could not be represented as a script number.
    ScriptNumberRangeExceeded,
    /// The primitive is not available in the executing domain.
    UnsupportedExecutionDomain,
    /// The introspection context was unavailable.
    IntrospectionContextUnavailable,
    /// An introspection index named no such input or output.
    IntrospectionIndexOutOfRange,
    /// A serialized hash state could not be loaded.
    HashContextLoad,
    /// A hash state could not absorb the offered data.
    HashContextWrite,
    /// A signed fixed-width operation overflowed its width.
    ArithmeticOverflow,
    /// A division had a zero divisor.
    DivisionByZero,
    /// The offered signature was empty.
    EmptySignature,
    /// The offered signature did not verify.
    InvalidSignature,
    /// A public key was absent or carried an encoding the primitive
    /// rejects.
    InvalidPublicKeyEncoding,
    /// An elliptic-curve relation did not hold.
    InvalidCurveRelation,
    /// A relative timelock was not satisfied.
    UnsatisfiedTimelock,
    /// A timelock operand was negative.
    NegativeTimelock,
    /// The script-path validation budget was exhausted.
    ValidationBudgetExhausted,
}

/// What the target does when a primitive fails.
///
/// The three outcomes are genuinely distinct and must not be
/// collapsed: aborting ends evaluation, consuming operands and pushing
/// a false leaves a shallower stack, and retaining operands and
/// pushing a false leaves a *deeper* stack than the successful path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FailureOutcome {
    /// Script evaluation ends and the spend is invalid.
    AbortEvaluation,
    /// The operands are consumed and a false is pushed in their place.
    ConsumeOperandsPushFalse,
    /// The operands are left untouched and a false is pushed above
    /// them.
    ///
    /// This is the reviewed behavior of fixed-width arithmetic on
    /// overflow and of division by zero. A caller that assumed
    /// operands were consumed would compute the wrong stack depth on
    /// exactly the path where it matters.
    RetainOperandsPushFalse,
}

/// One failure cause paired with the effect it produces.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FailureEffect {
    cause: FailureCause,
    outcome: FailureOutcome,
}

impl FailureEffect {
    /// States that `cause` produces `outcome`.
    #[must_use]
    pub const fn new(cause: FailureCause, outcome: FailureOutcome) -> Self {
        Self { cause, outcome }
    }

    /// The cause.
    #[must_use]
    pub const fn cause(self) -> FailureCause {
        self.cause
    }

    /// The effect that cause produces.
    #[must_use]
    pub const fn outcome(self) -> FailureOutcome {
        self.outcome
    }
}

/// The complete failure behavior of one primitive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailureContract {
    effects: BTreeSet<FailureEffect>,
}

impl FailureContract {
    /// Builds a failure contract from its effects.
    #[must_use]
    pub fn new(effects: impl IntoIterator<Item = FailureEffect>) -> Self {
        Self {
            effects: effects.into_iter().collect(),
        }
    }

    /// The declared effects, in stable cause order.
    #[must_use]
    pub const fn effects(&self) -> &BTreeSet<FailureEffect> {
        &self.effects
    }

    /// Whether any effect is declared at all.
    ///
    /// Every reviewed primitive has at least one failure path, so an
    /// empty contract is an incomplete transcription rather than a
    /// primitive that cannot fail.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }

    /// Whether one cause is declared with two different outcomes,
    /// which would make the contract self-contradictory.
    #[must_use]
    pub fn contradictory_cause(&self) -> Option<FailureCause> {
        let mut seen: BTreeSet<FailureCause> = BTreeSet::new();
        for effect in &self.effects {
            if !seen.insert(effect.cause()) {
                return Some(effect.cause());
            }
        }
        None
    }
}

/// The operand and result behavior of one primitive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackContract {
    operands: Vec<StackValueType>,
    success_results: Vec<StackValueType>,
    failure: FailureContract,
}

impl StackContract {
    /// Builds a stack contract.
    ///
    /// `operands` is ordered deepest first, matching the order in
    /// which a program pushes them. `success_results` is ordered in
    /// push order, so its last element ends up on top.
    #[must_use]
    pub const fn new(
        operands: Vec<StackValueType>,
        success_results: Vec<StackValueType>,
        failure: FailureContract,
    ) -> Self {
        Self {
            operands,
            success_results,
            failure,
        }
    }

    /// The operands, deepest first.
    #[must_use]
    pub fn operands(&self) -> &[StackValueType] {
        &self.operands
    }

    /// The successful results, in push order.
    #[must_use]
    pub fn success_results(&self) -> &[StackValueType] {
        &self.success_results
    }

    /// The failure behavior.
    #[must_use]
    pub const fn failure(&self) -> &FailureContract {
        &self.failure
    }

    /// Whether any declared operand or result has an incoherent width.
    #[must_use]
    pub fn has_malformed_width(&self) -> bool {
        self.operands
            .iter()
            .chain(self.success_results.iter())
            .any(|value| matches!(value, StackValueType::Bytes { minimum, maximum } if minimum > maximum))
    }
}

/// The resource units one primitive consumes.
///
/// The units are kept apart on purpose. Script bytes, the script-path
/// validation budget, and stack depth are measured in different things
/// and bound different limits; combining them into one weighted score
/// would produce a number that no target limit is expressed in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpcodeResourceCost {
    script_bytes: u64,
    operation_cost: u64,
    validation_budget: u64,
    maximum_stack_growth: i64,
    maximum_altstack_growth: i64,
}

impl OpcodeResourceCost {
    /// States the cost of one primitive.
    #[must_use]
    pub const fn new(
        script_bytes: u64,
        operation_cost: u64,
        validation_budget: u64,
        maximum_stack_growth: i64,
        maximum_altstack_growth: i64,
    ) -> Self {
        Self {
            script_bytes,
            operation_cost,
            validation_budget,
            maximum_stack_growth,
            maximum_altstack_growth,
        }
    }

    /// The bytes the primitive occupies in a script.
    #[must_use]
    pub const fn script_bytes(self) -> u64 {
        self.script_bytes
    }

    /// The operation-budget units the primitive consumes.
    ///
    /// The reviewed execution domain enforces no per-script operation
    /// budget, so every reviewed primitive states zero here. That is a
    /// reviewed target fact, not an unfilled field: the dimension
    /// exists because a different domain or a future policy could
    /// charge it.
    #[must_use]
    pub const fn operation_cost(self) -> u64 {
        self.operation_cost
    }

    /// The script-path validation budget the primitive can consume.
    ///
    /// This is the greatest amount the primitive can charge. The
    /// reviewed signature primitives charge only when the offered
    /// signature is non-empty, so a program that supplies an empty
    /// signature is charged nothing; the contract states the maximum
    /// because that is the figure a bound must be checked against.
    #[must_use]
    pub const fn validation_budget(self) -> u64 {
        self.validation_budget
    }

    /// The greatest increase in main-stack depth the primitive can
    /// cause at any point during its execution.
    ///
    /// Negative values mean the primitive can only shrink the stack.
    #[must_use]
    pub const fn maximum_stack_growth(self) -> i64 {
        self.maximum_stack_growth
    }

    /// The greatest increase in alternate-stack depth the primitive
    /// can cause.
    ///
    /// Every reviewed primitive states zero: none of them touches the
    /// alternate stack.
    #[must_use]
    pub const fn maximum_altstack_growth(self) -> i64 {
        self.maximum_altstack_growth
    }
}

/// A stable key naming one reviewed target primitive.
///
/// Admission is by review, not by availability: a primitive the target
/// implements but this package has not reviewed does not appear here,
/// and its absence means "not reviewed", never "not available".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum OpcodeId {
    /// Begins a streaming hash over an initial chunk.
    Sha256Initialize,
    /// Absorbs a further chunk into a streaming hash state.
    Sha256Update,
    /// Absorbs a final chunk and produces the digest.
    Sha256Finalize,

    /// Pushes the outpoint of one input.
    InspectInputOutpoint,
    /// Pushes the asset of one spent output.
    InspectInputAsset,
    /// Pushes the value of one spent output.
    InspectInputValue,
    /// Pushes the program of one spent output.
    InspectInputScriptPubKey,
    /// Pushes the sequence field of one input.
    InspectInputSequence,
    /// Pushes the issuance fields of one input.
    InspectInputIssuance,

    /// Pushes the index of the input being validated.
    PushCurrentInputIndex,

    /// Pushes the asset of one output.
    InspectOutputAsset,
    /// Pushes the value of one output.
    InspectOutputValue,
    /// Pushes the nonce of one output.
    InspectOutputNonce,
    /// Pushes the program of one output.
    InspectOutputScriptPubKey,

    /// Pushes the transaction version.
    InspectVersion,
    /// Pushes the transaction locktime.
    InspectLockTime,
    /// Pushes the input count.
    InspectNumInputs,
    /// Pushes the output count.
    InspectNumOutputs,
    /// Pushes the transaction weight.
    TxWeight,

    /// Adds two signed fixed-width operands.
    Add64,
    /// Subtracts the top signed fixed-width operand from the deeper.
    Sub64,
    /// Multiplies two signed fixed-width operands.
    Mul64,
    /// Divides the deeper signed fixed-width operand by the top,
    /// producing a remainder and a quotient.
    Div64,
    /// Negates one signed fixed-width operand.
    Neg64,

    /// Orders two signed fixed-width operands strictly.
    LessThan64,
    /// Orders two signed fixed-width operands inclusively.
    LessThanOrEqual64,
    /// Orders two signed fixed-width operands strictly, reversed.
    GreaterThan64,
    /// Orders two signed fixed-width operands inclusively, reversed.
    GreaterThanOrEqual64,

    /// Widens a script number to a signed fixed-width value.
    ScriptNumToLe64,
    /// Narrows a signed fixed-width value to a script number.
    Le64ToScriptNum,
    /// Widens an unsigned 32-bit value to a signed fixed-width value.
    Le32ToLe64,

    /// Verifies that a point is a scalar multiple of a generator.
    EcMulScalarVerify,
    /// Verifies a pay-to-contract tweak relation.
    TweakVerify,

    /// Verifies a signature over the transaction sighash.
    CheckSig,
    /// Verifies a signature over the transaction sighash and requires
    /// success.
    CheckSigVerify,
    /// Verifies a signature over a message taken from the stack.
    CheckSigFromStack,
    /// Verifies a signature over a message taken from the stack and
    /// requires success.
    CheckSigFromStackVerify,

    /// Requires a relative timelock to have matured.
    CheckSequenceVerify,
}

impl OpcodeId {
    /// The complete census of reviewed primitives.
    ///
    /// Declaration order here is a grouping convenience. It is not the
    /// target byte order, and a consumer must read the byte from the
    /// specification rather than infer it from this position.
    pub const ALL: &'static [Self] = &[
        Self::Sha256Initialize,
        Self::Sha256Update,
        Self::Sha256Finalize,
        Self::InspectInputOutpoint,
        Self::InspectInputAsset,
        Self::InspectInputValue,
        Self::InspectInputScriptPubKey,
        Self::InspectInputSequence,
        Self::InspectInputIssuance,
        Self::PushCurrentInputIndex,
        Self::InspectOutputAsset,
        Self::InspectOutputValue,
        Self::InspectOutputNonce,
        Self::InspectOutputScriptPubKey,
        Self::InspectVersion,
        Self::InspectLockTime,
        Self::InspectNumInputs,
        Self::InspectNumOutputs,
        Self::TxWeight,
        Self::Add64,
        Self::Sub64,
        Self::Mul64,
        Self::Div64,
        Self::Neg64,
        Self::LessThan64,
        Self::LessThanOrEqual64,
        Self::GreaterThan64,
        Self::GreaterThanOrEqual64,
        Self::ScriptNumToLe64,
        Self::Le64ToScriptNum,
        Self::Le32ToLe64,
        Self::EcMulScalarVerify,
        Self::TweakVerify,
        Self::CheckSig,
        Self::CheckSigVerify,
        Self::CheckSigFromStack,
        Self::CheckSigFromStackVerify,
        Self::CheckSequenceVerify,
    ];
}

/// The complete typed contract of one reviewed primitive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpcodeSpec {
    id: OpcodeId,
    code: u8,
    domains: BTreeSet<ExecutionDomain>,
    stack: StackContract,
    resources: OpcodeResourceCost,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
}

impl OpcodeSpec {
    /// States the contract of one primitive.
    #[must_use]
    pub fn new(
        id: OpcodeId,
        code: u8,
        domains: impl IntoIterator<Item = ExecutionDomain>,
        stack: StackContract,
        resources: OpcodeResourceCost,
        evidence: impl IntoIterator<Item = TargetEvidenceRequirementId>,
    ) -> Self {
        Self {
            id,
            code,
            domains: domains.into_iter().collect(),
            stack,
            resources,
            evidence: evidence.into_iter().collect(),
        }
    }

    /// The primitive's stable identity.
    #[must_use]
    pub const fn id(&self) -> OpcodeId {
        self.id
    }

    /// The primitive's target byte.
    #[must_use]
    pub const fn code(&self) -> u8 {
        self.code
    }

    /// The domains the primitive executes in.
    #[must_use]
    pub const fn domains(&self) -> &BTreeSet<ExecutionDomain> {
        &self.domains
    }

    /// The primitive's operand, result, and failure behavior.
    #[must_use]
    pub const fn stack(&self) -> &StackContract {
        &self.stack
    }

    /// The primitive's resource cost.
    #[must_use]
    pub const fn resources(&self) -> OpcodeResourceCost {
        self.resources
    }

    /// The evidence a deployment must produce for this primitive.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }
}

// ---------------------------------------------------------------
// The reviewed registry.
// ---------------------------------------------------------------

/// Eight bytes, the width of the target's fixed-width integers.
const fn eight() -> NonZeroUsize {
    NonZeroUsize::new(8).expect("8 is not zero")
}

/// Four bytes, the width of the target's 32-bit fields.
const fn four() -> NonZeroUsize {
    NonZeroUsize::new(4).expect("4 is not zero")
}

/// A signed little-endian 64-bit stack item.
const fn signed64() -> StackValueType {
    StackValueType::SignedFixedWidth {
        bytes: eight(),
        byte_order: ByteOrder::LittleEndian,
    }
}

/// An unsigned little-endian 64-bit stack item.
const fn unsigned64() -> StackValueType {
    StackValueType::UnsignedFixedWidth {
        bytes: eight(),
        byte_order: ByteOrder::LittleEndian,
    }
}

/// An unsigned little-endian 32-bit stack item.
const fn unsigned32() -> StackValueType {
    StackValueType::UnsignedFixedWidth {
        bytes: four(),
        byte_order: ByteOrder::LittleEndian,
    }
}

/// The failure effects every domain-gated primitive shares.
fn gated(extra: impl IntoIterator<Item = FailureEffect>) -> FailureContract {
    let mut effects = vec![FailureEffect::new(
        FailureCause::UnsupportedExecutionDomain,
        FailureOutcome::AbortEvaluation,
    )];
    effects.extend(extra);
    FailureContract::new(effects)
}

/// An aborting failure effect.
const fn abort(cause: FailureCause) -> FailureEffect {
    FailureEffect::new(cause, FailureOutcome::AbortEvaluation)
}

/// A failure that leaves the operands in place and pushes a false.
const fn retain(cause: FailureCause) -> FailureEffect {
    FailureEffect::new(cause, FailureOutcome::RetainOperandsPushFalse)
}

/// The cost of a primitive that charges no validation budget.
const fn plain(stack_growth: i64) -> OpcodeResourceCost {
    OpcodeResourceCost::new(1, 0, 0, stack_growth, 0)
}

/// The cost of a primitive that can charge the per-signature budget.
const fn budgeted(stack_growth: i64) -> OpcodeResourceCost {
    OpcodeResourceCost::new(1, 0, VALIDATION_BUDGET_PER_CHECK, stack_growth, 0)
}

/// The script-path validation budget one signature or curve check can
/// consume.
pub const VALIDATION_BUDGET_PER_CHECK: u64 = 50;

/// The failure effects shared by every index-taking introspection
/// primitive.
fn introspection_failures() -> Vec<FailureEffect> {
    vec![
        abort(FailureCause::StackUnderflow),
        abort(FailureCause::MalformedScriptNumber),
        abort(FailureCause::IntrospectionContextUnavailable),
        abort(FailureCause::IntrospectionIndexOutOfRange),
    ]
}

/// Builds one specification, gated to the reviewed domain.
fn spec(
    id: OpcodeId,
    code: u8,
    stack: StackContract,
    resources: OpcodeResourceCost,
    evidence: &[TargetEvidenceRequirementId],
) -> (OpcodeId, OpcodeSpec) {
    (
        id,
        OpcodeSpec::new(
            id,
            code,
            [ExecutionDomain::Tapscript],
            stack,
            resources,
            evidence.iter().copied(),
        ),
    )
}

/// A serialized streaming hash state on the stack.
const fn hash_state() -> StackValueType {
    StackValueType::Encoded(EncodingClass::Sha256Context)
}

/// An unconstrained byte string operand.
const fn any_bytes() -> StackValueType {
    StackValueType::Bytes {
        minimum: 0,
        maximum: MAX_STACK_ELEMENT_BYTES,
    }
}

/// Part of the reviewed primitive registry.
fn hashing_opcodes() -> Vec<(OpcodeId, OpcodeSpec)> {
    use crate::encoding::EncodingClass as E;
    use crate::evidence::TargetEvidenceRequirementId as R;
    use FailureCause as C;
    use OpcodeId as O;
    use StackValueType as S;

    vec![
        // -- Streaming hashing ---------------------------------
        spec(
            O::Sha256Initialize,
            0xc4,
            StackContract::new(
                vec![any_bytes()],
                vec![hash_state()],
                gated([abort(C::StackUnderflow), abort(C::HashContextWrite)]),
            ),
            plain(0),
            &[R::OpcodeSemantics, R::StreamingHashSemantics],
        ),
        spec(
            O::Sha256Update,
            0xc5,
            StackContract::new(
                vec![hash_state(), any_bytes()],
                vec![hash_state()],
                gated([
                    abort(C::StackUnderflow),
                    abort(C::HashContextLoad),
                    abort(C::HashContextWrite),
                ]),
            ),
            plain(-1),
            &[R::OpcodeSemantics, R::StreamingHashSemantics],
        ),
        spec(
            O::Sha256Finalize,
            0xc6,
            StackContract::new(
                vec![hash_state(), any_bytes()],
                vec![S::Encoded(E::Sha256Digest)],
                gated([
                    abort(C::StackUnderflow),
                    abort(C::HashContextLoad),
                    abort(C::HashContextWrite),
                ]),
            ),
            plain(-1),
            &[R::OpcodeSemantics, R::StreamingHashSemantics],
        ),
    ]
}
/// Part of the reviewed primitive registry.
fn input_introspection_opcodes() -> Vec<(OpcodeId, OpcodeSpec)> {
    use crate::encoding::EncodingClass as E;
    use crate::evidence::TargetEvidenceRequirementId as R;
    use OpcodeId as O;
    use StackValueType as S;

    vec![
        // -- Input introspection -------------------------------
        spec(
            O::InspectInputOutpoint,
            0xc7,
            StackContract::new(
                vec![S::ScriptNumber],
                vec![
                    S::Encoded(E::OutPointTxid),
                    S::Encoded(E::OutPointIndex),
                    S::Encoded(E::OutPointFlags),
                ],
                gated(introspection_failures()),
            ),
            plain(2),
            &[R::OpcodeSemantics, R::InputIntrospectionSemantics],
        ),
        spec(
            O::InspectInputAsset,
            0xc8,
            StackContract::new(
                vec![S::ScriptNumber],
                // The payload is pushed first and the prefix second,
                // so the prefix ends up on top. Both explicit and
                // confidential assets carry a payload of the same
                // width, which is why the prefix is the only thing
                // that distinguishes them on the stack.
                vec![
                    S::EncodedPayload(E::ExplicitAsset),
                    S::EncodingPrefix(E::ExplicitAsset),
                ],
                gated(introspection_failures()),
            ),
            plain(1),
            &[
                R::OpcodeSemantics,
                R::InputIntrospectionSemantics,
                R::EncodingSemantics,
            ],
        ),
        spec(
            O::InspectInputValue,
            0xc9,
            StackContract::new(
                vec![S::ScriptNumber],
                // An explicit amount reaches the stack little-endian
                // even though the transaction field stores it
                // big-endian, and a confidential amount reaches it as
                // a wider payload. The prefix is what tells them
                // apart, and it is pushed last.
                vec![
                    S::EncodedPayload(E::ExplicitValue),
                    S::EncodingPrefix(E::ExplicitValue),
                ],
                gated(introspection_failures()),
            ),
            plain(1),
            &[
                R::OpcodeSemantics,
                R::InputIntrospectionSemantics,
                R::EncodingSemantics,
            ],
        ),
        spec(
            O::InspectInputScriptPubKey,
            0xca,
            StackContract::new(
                vec![S::ScriptNumber],
                // A witness program is pushed with its version; any
                // other program is replaced by a digest paired with a
                // negative version marker.
                vec![S::Encoded(E::WitnessProgram), S::ScriptNumber],
                gated(introspection_failures()),
            ),
            plain(1),
            &[
                R::OpcodeSemantics,
                R::InputIntrospectionSemantics,
                R::EncodingSemantics,
            ],
        ),
    ]
}

/// Part of the reviewed primitive registry: the input fields whose
/// results are single items rather than a split payload and prefix.
fn input_sequence_and_issuance_opcodes() -> Vec<(OpcodeId, OpcodeSpec)> {
    use crate::encoding::EncodingClass as E;
    use crate::evidence::TargetEvidenceRequirementId as R;
    use OpcodeId as O;
    use StackValueType as S;

    vec![
        spec(
            O::InspectInputSequence,
            0xcb,
            StackContract::new(
                vec![S::ScriptNumber],
                vec![S::Encoded(E::Sequence)],
                gated(introspection_failures()),
            ),
            plain(0),
            &[R::OpcodeSemantics, R::InputIntrospectionSemantics],
        ),
        spec(
            O::InspectInputIssuance,
            0xcc,
            StackContract::new(
                vec![S::ScriptNumber],
                // An input carrying an issuance pushes six items; an
                // input carrying none pushes a single empty item. The
                // successful result stated here is the issuance-present
                // shape, and the absent shape is the reason the empty
                // marker is a declared stack type rather than an
                // afterthought.
                vec![
                    S::EncodedPayload(E::ExplicitValue),
                    S::EncodingPrefix(E::ExplicitValue),
                    S::EncodedPayload(E::ExplicitValue),
                    S::EncodingPrefix(E::ExplicitValue),
                    S::Encoded(E::IssuanceEntropy),
                    S::Encoded(E::IssuanceBlindingNonce),
                ],
                gated(introspection_failures()),
            ),
            plain(5),
            &[
                R::OpcodeSemantics,
                R::InputIntrospectionSemantics,
                R::IssuanceIntrospection,
            ],
        ),
    ]
}
/// Part of the reviewed primitive registry.
fn current_index_opcodes() -> Vec<(OpcodeId, OpcodeSpec)> {
    use crate::evidence::TargetEvidenceRequirementId as R;
    use FailureCause as C;
    use OpcodeId as O;
    use StackValueType as S;

    vec![
        // -- Current index -------------------------------------
        spec(
            O::PushCurrentInputIndex,
            0xcd,
            StackContract::new(
                vec![],
                vec![S::ScriptNumber],
                gated([abort(C::IntrospectionContextUnavailable)]),
            ),
            plain(1),
            &[R::OpcodeSemantics, R::InputIntrospectionSemantics],
        ),
    ]
}
/// Part of the reviewed primitive registry.
fn output_introspection_opcodes() -> Vec<(OpcodeId, OpcodeSpec)> {
    use crate::encoding::EncodingClass as E;
    use crate::evidence::TargetEvidenceRequirementId as R;
    use OpcodeId as O;
    use StackValueType as S;

    vec![
        // -- Output introspection ------------------------------
        spec(
            O::InspectOutputAsset,
            0xce,
            StackContract::new(
                vec![S::ScriptNumber],
                vec![
                    S::EncodedPayload(E::ExplicitAsset),
                    S::EncodingPrefix(E::ExplicitAsset),
                ],
                gated(introspection_failures()),
            ),
            plain(1),
            &[
                R::OpcodeSemantics,
                R::OutputIntrospectionSemantics,
                R::EncodingSemantics,
            ],
        ),
        spec(
            O::InspectOutputValue,
            0xcf,
            StackContract::new(
                vec![S::ScriptNumber],
                vec![
                    S::EncodedPayload(E::ExplicitValue),
                    S::EncodingPrefix(E::ExplicitValue),
                ],
                gated(introspection_failures()),
            ),
            plain(1),
            &[
                R::OpcodeSemantics,
                R::OutputIntrospectionSemantics,
                R::EncodingSemantics,
            ],
        ),
        spec(
            O::InspectOutputNonce,
            0xd0,
            StackContract::new(
                vec![S::ScriptNumber],
                // The one asymmetric case among the reviewed
                // introspection primitives: the nonce arrives as a
                // single item with its prefix byte still attached,
                // where assets and values arrive split in two.
                vec![S::Encoded(E::ExplicitNonce)],
                gated(introspection_failures()),
            ),
            plain(0),
            &[
                R::OpcodeSemantics,
                R::OutputIntrospectionSemantics,
                R::EncodingSemantics,
            ],
        ),
        spec(
            O::InspectOutputScriptPubKey,
            0xd1,
            StackContract::new(
                vec![S::ScriptNumber],
                vec![S::Encoded(E::WitnessProgram), S::ScriptNumber],
                gated(introspection_failures()),
            ),
            plain(1),
            &[
                R::OpcodeSemantics,
                R::OutputIntrospectionSemantics,
                R::EncodingSemantics,
            ],
        ),
    ]
}
/// Part of the reviewed primitive registry.
fn transaction_introspection_opcodes() -> Vec<(OpcodeId, OpcodeSpec)> {
    use crate::evidence::TargetEvidenceRequirementId as R;
    use FailureCause as C;
    use OpcodeId as O;
    use StackValueType as S;

    vec![
        // -- Transaction introspection -------------------------
        spec(
            O::InspectVersion,
            0xd2,
            StackContract::new(vec![], vec![unsigned32()], gated([])),
            plain(1),
            &[R::OpcodeSemantics, R::TransactionIntrospectionSemantics],
        ),
        spec(
            O::InspectLockTime,
            0xd3,
            StackContract::new(vec![], vec![unsigned32()], gated([])),
            plain(1),
            &[R::OpcodeSemantics, R::TransactionIntrospectionSemantics],
        ),
        spec(
            O::InspectNumInputs,
            0xd4,
            StackContract::new(
                vec![],
                vec![S::ScriptNumber],
                gated([abort(C::IntrospectionContextUnavailable)]),
            ),
            plain(1),
            &[R::OpcodeSemantics, R::TransactionIntrospectionSemantics],
        ),
        spec(
            O::InspectNumOutputs,
            0xd5,
            StackContract::new(
                vec![],
                vec![S::ScriptNumber],
                gated([abort(C::IntrospectionContextUnavailable)]),
            ),
            plain(1),
            &[R::OpcodeSemantics, R::TransactionIntrospectionSemantics],
        ),
        spec(
            O::TxWeight,
            0xd6,
            StackContract::new(
                vec![],
                vec![unsigned64()],
                gated([abort(C::IntrospectionContextUnavailable)]),
            ),
            plain(1),
            &[R::OpcodeSemantics, R::TransactionIntrospectionSemantics],
        ),
    ]
}
/// Part of the reviewed primitive registry.
fn arithmetic_opcodes() -> Vec<(OpcodeId, OpcodeSpec)> {
    use crate::evidence::TargetEvidenceRequirementId as R;
    use FailureCause as C;
    use OpcodeId as O;
    use StackValueType as S;

    vec![
        // -- Fixed-width arithmetic ----------------------------
        //
        // The arithmetic primitives push their result *and* a success
        // flag, and on overflow they leave both operands in place and
        // push a false above them. That asymmetry is the single most
        // important reviewed fact in this registry.
        spec(
            O::Add64,
            0xd7,
            StackContract::new(
                vec![signed64(), signed64()],
                vec![signed64(), S::Bool],
                gated([
                    abort(C::StackUnderflow),
                    abort(C::InvalidOperandWidth),
                    retain(C::ArithmeticOverflow),
                ]),
            ),
            plain(1),
            &[R::OpcodeSemantics, R::ArithmeticSemantics],
        ),
        spec(
            O::Sub64,
            0xd8,
            StackContract::new(
                vec![signed64(), signed64()],
                vec![signed64(), S::Bool],
                gated([
                    abort(C::StackUnderflow),
                    abort(C::InvalidOperandWidth),
                    retain(C::ArithmeticOverflow),
                ]),
            ),
            plain(1),
            &[R::OpcodeSemantics, R::ArithmeticSemantics],
        ),
        spec(
            O::Mul64,
            0xd9,
            StackContract::new(
                vec![signed64(), signed64()],
                vec![signed64(), S::Bool],
                gated([
                    abort(C::StackUnderflow),
                    abort(C::InvalidOperandWidth),
                    retain(C::ArithmeticOverflow),
                ]),
            ),
            plain(1),
            &[R::OpcodeSemantics, R::ArithmeticSemantics],
        ),
        spec(
            O::Div64,
            0xda,
            StackContract::new(
                vec![signed64(), signed64()],
                // Remainder first, then quotient, then the flag. The
                // remainder is normalized non-negative, so this is
                // Euclidean rather than truncating division.
                vec![signed64(), signed64(), S::Bool],
                gated([
                    abort(C::StackUnderflow),
                    abort(C::InvalidOperandWidth),
                    retain(C::DivisionByZero),
                    retain(C::ArithmeticOverflow),
                ]),
            ),
            plain(1),
            &[R::OpcodeSemantics, R::ArithmeticSemantics],
        ),
        spec(
            O::Neg64,
            0xdb,
            StackContract::new(
                vec![signed64()],
                vec![signed64(), S::Bool],
                gated([
                    abort(C::StackUnderflow),
                    abort(C::InvalidOperandWidth),
                    retain(C::ArithmeticOverflow),
                ]),
            ),
            plain(1),
            &[R::OpcodeSemantics, R::ArithmeticSemantics],
        ),
    ]
}
/// Part of the reviewed primitive registry.
fn comparison_opcodes() -> Vec<(OpcodeId, OpcodeSpec)> {
    use crate::evidence::TargetEvidenceRequirementId as R;
    use FailureCause as C;
    use OpcodeId as O;
    use StackValueType as S;

    vec![
        // -- Fixed-width comparison ----------------------------
        //
        // Unlike the arithmetic primitives these always consume both
        // operands and push exactly one item. The false they can push
        // is the comparison's answer, not a failure flag, so it is a
        // successful result rather than a failure effect.
        spec(
            O::LessThan64,
            0xdc,
            StackContract::new(
                vec![signed64(), signed64()],
                vec![S::Bool],
                gated([abort(C::StackUnderflow), abort(C::InvalidOperandWidth)]),
            ),
            plain(-1),
            &[R::OpcodeSemantics, R::ComparisonSemantics],
        ),
        spec(
            O::LessThanOrEqual64,
            0xdd,
            StackContract::new(
                vec![signed64(), signed64()],
                vec![S::Bool],
                gated([abort(C::StackUnderflow), abort(C::InvalidOperandWidth)]),
            ),
            plain(-1),
            &[R::OpcodeSemantics, R::ComparisonSemantics],
        ),
        spec(
            O::GreaterThan64,
            0xde,
            StackContract::new(
                vec![signed64(), signed64()],
                vec![S::Bool],
                gated([abort(C::StackUnderflow), abort(C::InvalidOperandWidth)]),
            ),
            plain(-1),
            &[R::OpcodeSemantics, R::ComparisonSemantics],
        ),
        spec(
            O::GreaterThanOrEqual64,
            0xdf,
            StackContract::new(
                vec![signed64(), signed64()],
                vec![S::Bool],
                gated([abort(C::StackUnderflow), abort(C::InvalidOperandWidth)]),
            ),
            plain(-1),
            &[R::OpcodeSemantics, R::ComparisonSemantics],
        ),
    ]
}
/// Part of the reviewed primitive registry.
fn conversion_opcodes() -> Vec<(OpcodeId, OpcodeSpec)> {
    use crate::evidence::TargetEvidenceRequirementId as R;
    use FailureCause as C;
    use OpcodeId as O;
    use StackValueType as S;

    vec![
        // -- Conversions ---------------------------------------
        spec(
            O::ScriptNumToLe64,
            0xe0,
            StackContract::new(
                vec![S::ScriptNumber],
                vec![signed64()],
                gated([abort(C::StackUnderflow), abort(C::MalformedScriptNumber)]),
            ),
            plain(0),
            &[R::OpcodeSemantics, R::ConversionSemantics],
        ),
        spec(
            O::Le64ToScriptNum,
            0xe1,
            StackContract::new(
                vec![signed64()],
                vec![S::ScriptNumber],
                // Narrowing aborts rather than pushing a false: a
                // value outside the script number's range has no
                // representation to push.
                gated([
                    abort(C::StackUnderflow),
                    abort(C::InvalidOperandWidth),
                    abort(C::ScriptNumberRangeExceeded),
                ]),
            ),
            plain(0),
            &[R::OpcodeSemantics, R::ConversionSemantics],
        ),
        spec(
            O::Le32ToLe64,
            0xe2,
            StackContract::new(
                // The operand is read unsigned and zero-extended, so
                // this widening never produces a negative result.
                vec![unsigned32()],
                vec![signed64()],
                gated([abort(C::StackUnderflow), abort(C::InvalidOperandWidth)]),
            ),
            plain(0),
            &[R::OpcodeSemantics, R::ConversionSemantics],
        ),
    ]
}
/// Part of the reviewed primitive registry.
fn curve_opcodes() -> Vec<(OpcodeId, OpcodeSpec)> {
    use crate::encoding::EncodingClass as E;
    use crate::evidence::TargetEvidenceRequirementId as R;
    use FailureCause as C;
    use OpcodeId as O;
    use StackValueType as S;

    vec![
        // -- Elliptic-curve checks -----------------------------
        spec(
            O::EcMulScalarVerify,
            0xe3,
            StackContract::new(
                vec![
                    S::Encoded(E::CompressedPublicKey),
                    S::Encoded(E::CompressedPublicKey),
                    S::Encoded(E::EcScalar),
                ],
                // A verify-style primitive: it drops its operands and
                // pushes nothing at all.
                vec![],
                gated([
                    abort(C::StackUnderflow),
                    abort(C::InvalidPublicKeyEncoding),
                    abort(C::ValidationBudgetExhausted),
                    abort(C::InvalidCurveRelation),
                ]),
            ),
            budgeted(-3),
            &[R::OpcodeSemantics, R::EllipticCurveSemantics],
        ),
        spec(
            O::TweakVerify,
            0xe4,
            StackContract::new(
                vec![
                    S::Encoded(E::CompressedPublicKey),
                    S::Encoded(E::TaprootTweak),
                    S::Encoded(E::XOnlyPublicKey),
                ],
                vec![],
                gated([
                    abort(C::StackUnderflow),
                    abort(C::InvalidPublicKeyEncoding),
                    abort(C::ValidationBudgetExhausted),
                    abort(C::InvalidCurveRelation),
                ]),
            ),
            budgeted(-3),
            &[R::OpcodeSemantics, R::EllipticCurveSemantics],
        ),
    ]
}
/// Part of the reviewed primitive registry.
fn signature_opcodes() -> Vec<(OpcodeId, OpcodeSpec)> {
    use crate::encoding::EncodingClass as E;
    use crate::evidence::TargetEvidenceRequirementId as R;
    use FailureCause as C;
    use OpcodeId as O;
    use StackValueType as S;

    vec![
        // -- Signature verification ----------------------------
        //
        // The reviewed failure split matters: an empty signature
        // consumes the operands and pushes a false, while a non-empty
        // signature that does not verify aborts. A backend cannot
        // treat "verification failed" as a branchable condition.
        spec(
            O::CheckSig,
            0xac,
            StackContract::new(
                vec![
                    S::Encoded(E::SchnorrSignature),
                    S::Encoded(E::XOnlyPublicKey),
                ],
                vec![S::Bool],
                FailureContract::new([
                    abort(C::StackUnderflow),
                    abort(C::InvalidPublicKeyEncoding),
                    abort(C::ValidationBudgetExhausted),
                    abort(C::InvalidSignature),
                    FailureEffect::new(C::EmptySignature, FailureOutcome::ConsumeOperandsPushFalse),
                ]),
            ),
            budgeted(-1),
            &[
                R::OpcodeSemantics,
                R::SignatureSemantics,
                R::SighashSemantics,
            ],
        ),
        spec(
            O::CheckSigVerify,
            0xad,
            StackContract::new(
                vec![
                    S::Encoded(E::SchnorrSignature),
                    S::Encoded(E::XOnlyPublicKey),
                ],
                vec![],
                // The verifying form leaves no branchable result: an
                // empty signature that would have pushed a false
                // aborts here instead.
                FailureContract::new([
                    abort(C::StackUnderflow),
                    abort(C::InvalidPublicKeyEncoding),
                    abort(C::ValidationBudgetExhausted),
                    abort(C::InvalidSignature),
                    abort(C::EmptySignature),
                ]),
            ),
            budgeted(-1),
            &[
                R::OpcodeSemantics,
                R::SignatureSemantics,
                R::SighashSemantics,
            ],
        ),
        spec(
            O::CheckSigFromStack,
            0xc1,
            StackContract::new(
                vec![
                    S::Encoded(E::SchnorrSignature),
                    any_bytes(),
                    S::Encoded(E::XOnlyPublicKey),
                ],
                vec![S::Bool],
                FailureContract::new([
                    abort(C::StackUnderflow),
                    abort(C::InvalidPublicKeyEncoding),
                    abort(C::ValidationBudgetExhausted),
                    abort(C::InvalidSignature),
                    FailureEffect::new(C::EmptySignature, FailureOutcome::ConsumeOperandsPushFalse),
                ]),
            ),
            budgeted(-2),
            &[R::OpcodeSemantics, R::SignatureSemantics],
        ),
        spec(
            O::CheckSigFromStackVerify,
            0xc2,
            StackContract::new(
                vec![
                    S::Encoded(E::SchnorrSignature),
                    any_bytes(),
                    S::Encoded(E::XOnlyPublicKey),
                ],
                vec![],
                FailureContract::new([
                    abort(C::StackUnderflow),
                    abort(C::InvalidPublicKeyEncoding),
                    abort(C::ValidationBudgetExhausted),
                    abort(C::InvalidSignature),
                    abort(C::EmptySignature),
                ]),
            ),
            budgeted(-2),
            &[R::OpcodeSemantics, R::SignatureSemantics],
        ),
    ]
}
/// Part of the reviewed primitive registry.
fn timelock_opcodes() -> Vec<(OpcodeId, OpcodeSpec)> {
    use crate::evidence::TargetEvidenceRequirementId as R;
    use FailureCause as C;
    use OpcodeId as O;
    use StackValueType as S;

    vec![
        // -- Relative timelock ---------------------------------
        spec(
            O::CheckSequenceVerify,
            0xb2,
            StackContract::new(
                // The operand is inspected and left in place; this
                // primitive pushes and pops nothing.
                vec![S::ScriptNumber],
                vec![],
                FailureContract::new([
                    abort(C::StackUnderflow),
                    abort(C::MalformedScriptNumber),
                    abort(C::NegativeTimelock),
                    abort(C::UnsatisfiedTimelock),
                ]),
            ),
            plain(0),
            &[R::OpcodeSemantics, R::RelativeTimelockSemantics],
        ),
    ]
}

/// Builds the reviewed primitive registry.
///
/// Every entry was transcribed from a reviewed reading of the upstream
/// interpreter, one primitive at a time. The review record lives in the
/// human reference; it does not appear in these types.
pub(crate) fn reviewed_opcodes() -> BTreeMap<OpcodeId, OpcodeSpec> {
    [
        hashing_opcodes(),
        input_introspection_opcodes(),
        input_sequence_and_issuance_opcodes(),
        current_index_opcodes(),
        output_introspection_opcodes(),
        transaction_introspection_opcodes(),
        arithmetic_opcodes(),
        comparison_opcodes(),
        conversion_opcodes(),
        curve_opcodes(),
        signature_opcodes(),
        timelock_opcodes(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// The largest byte string the target admits as a literal script push
/// and as an initial witness item.
///
/// The reviewed execution domain does not re-check this bound against
/// computed results, so it is stated as the literal-push and initial
/// witness bound rather than as a universal stack item bound.
pub const MAX_STACK_ELEMENT_BYTES: usize = 520;
