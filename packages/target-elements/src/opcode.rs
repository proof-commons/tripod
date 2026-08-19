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
use crate::operand::OperandContract;
use crate::success::{
    ResultValue, SuccessCase, SuccessCondition, SuccessContract, SuccessContractDefect,
    SuccessStackEffect,
};

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
    /// The payload of an encoded field whose form is selected by the
    /// prefix item pushed immediately above it.
    ///
    /// # Why this exists alongside the success alternatives
    ///
    /// A primitive that reads *one* field states its explicit and
    /// confidential forms as separate success cases, because the whole
    /// result differs. The issuance primitive reads *four* fields at
    /// once, two of which are amounts that are independently explicit
    /// or blinded. Its condition is issuance presence, not
    /// confidentiality, and enumerating the product of the two amounts'
    /// forms as four cases would name a condition the target does not
    /// branch on. So the field's admissible forms are stated on the
    /// item itself, and the prefix pushed above it is what a program
    /// reads to tell them apart.
    EncodedPayloadAlternatives(BTreeSet<EncodingClass>),
    /// The prefix byte selecting among the alternative forms of one
    /// encoded field, pushed as its own item.
    EncodingPrefixAlternatives(BTreeSet<EncodingClass>),
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
    ///
    /// A documented path rather than a malformed operand: in the
    /// pushing form it consumes the operands and pushes a false, and
    /// it applies whatever the key is, because the target settles
    /// emptiness before it looks at the key's form.
    EmptySignature,
    /// The offered signature did not verify against a recognized key.
    ///
    /// Reachable only where the key is the recognized encoding. Where
    /// the key is an unrecognized nonempty form there is no
    /// verification to fail, which is a success rather than a silent
    /// pass of this cause.
    InvalidSignature,
    /// The offered public key was the empty item.
    ///
    /// Its own cause rather than a shade of a rejected encoding: the
    /// empty key is refused outright while every other unrecognized
    /// nonempty form succeeds without verification, so collapsing the
    /// two would put the target's forward-compatibility path and its
    /// hardest rejection under one name.
    EmptyPublicKey,
    /// A public key carried an encoding the primitive rejects.
    InvalidPublicKeyEncoding,
    /// An elliptic-curve relation did not hold.
    InvalidCurveRelation,
    /// A relative timelock was not satisfied.
    UnsatisfiedTimelock,
    /// A timelock operand was negative.
    NegativeTimelock,
    /// The script-path validation budget was exhausted.
    ValidationBudgetExhausted,
    /// A computed byte string was wider than the target admits.
    ///
    /// Distinct from an operand of the wrong width: the operands were
    /// each admissible and the *result* was not, which is a bound the
    /// reviewed domain does check on a computed value even though it
    /// does not check one generally.
    ResultSizeExceeded,
    /// A requested slice did not lie within the operand.
    SliceOutOfRange,
    /// Two operands the primitive compares were not equal.
    ///
    /// Its own cause rather than a shade of a false result: the
    /// pushing form of the comparison reports inequality as a
    /// successful false, and only the verifying form treats it as a
    /// failure at all.
    UnequalOperands,
    /// A verified operand was the target's false.
    FalseVerification,
    /// Two operands a bitwise primitive combines were of different
    /// widths.
    ///
    /// The bitwise primitives pair their operands byte for byte and
    /// refuse to guess at a shorter one, so a width disagreement ends
    /// evaluation rather than padding either side.
    MismatchedOperandWidths,
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
    operands: Vec<OperandContract>,
    success: SuccessContract,
    failure: FailureContract,
}

impl StackContract {
    /// Builds a stack contract.
    ///
    /// `operands` is ordered deepest first, matching the order in
    /// which a program pushes them. `success` states every valid
    /// successful form and what each one does to the stack.
    #[must_use]
    pub const fn new(
        operands: Vec<OperandContract>,
        success: SuccessContract,
        failure: FailureContract,
    ) -> Self {
        Self {
            operands,
            success,
            failure,
        }
    }

    /// The operands, deepest first.
    #[must_use]
    pub fn operands(&self) -> &[OperandContract] {
        &self.operands
    }

    /// The successful behavior, in all of its valid forms.
    #[must_use]
    pub const fn success(&self) -> &SuccessContract {
        &self.success
    }

    /// The failure behavior.
    #[must_use]
    pub const fn failure(&self) -> &FailureContract {
        &self.failure
    }

    /// Whether any declared operand or result has an incoherent width.
    #[must_use]
    pub fn has_malformed_width(&self) -> bool {
        let operand_types = self
            .operands
            .iter()
            .flat_map(OperandContract::named_types)
            .collect::<Vec<_>>();
        operand_types
            .iter()
            .chain(self.success.result_types().iter())
            .any(|value| matches!(value, StackValueType::Bytes { minimum, maximum } if minimum > maximum))
    }

    /// Why the successful behavior is not a coherent relation over the
    /// declared operands, if it is not.
    #[must_use]
    pub fn success_defect(&self) -> Option<SuccessContractDefect> {
        self.success.defect(self.operands.len())
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
    ///
    /// "At any point during its execution" is not the same as the depth
    /// the primitive settles at, and for two reviewed primitives the
    /// two figures differ. A verifying signature form reaches its
    /// branching counterpart's depth before the implicit verification
    /// consumes the truth value, so it declares one item above where it
    /// leaves the stack — which is the figure a scheduler must size
    /// against. The transient rule and its target source location are
    /// stated with the weld that derives this field.
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

    /// Copies the top item.
    Duplicate,
    /// Copies the top two items, as a pair.
    DuplicateTwo,
    /// Copies the second item to the top.
    CopyOver,
    /// Exchanges the top two items.
    Swap,
    /// Moves the third item to the top.
    Rotate,
    /// Removes the second item.
    RemoveSecond,
    /// Inserts a copy of the top item below the second.
    Tuck,
    /// Removes the top item.
    Drop,
    /// Removes the top two items.
    DropTwo,

    /// Compares two items for byte equality.
    Equal,
    /// Compares two items for byte equality and requires it.
    EqualVerify,
    /// Requires the top item to be true.
    Verify,

    /// Joins two items into one.
    Concatenate,
    /// Pushes the width of the top item above it.
    Size,
    /// Extracts a slice of one item.
    Substring,

    /// Combines two equal-width items bit by bit, conjunctively.
    BitwiseAnd,
    /// Combines two equal-width items bit by bit, exclusively.
    BitwiseXor,
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
        Self::Duplicate,
        Self::DuplicateTwo,
        Self::CopyOver,
        Self::Swap,
        Self::Rotate,
        Self::RemoveSecond,
        Self::Tuck,
        Self::Drop,
        Self::DropTwo,
        Self::Equal,
        Self::EqualVerify,
        Self::Verify,
        Self::Concatenate,
        Self::Size,
        Self::Substring,
        Self::BitwiseAnd,
        Self::BitwiseXor,
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

/// A stack contract whose one successful form consumes every operand
/// it declares.
///
/// The common shape by a wide margin. Stating the consumed count from
/// the operand list rather than by hand keeps the two from drifting
/// apart in a registry this long.
fn consuming(
    operands: Vec<StackValueType>,
    results: Vec<StackValueType>,
    failure: FailureContract,
) -> StackContract {
    let consumed_operands = operands.len();
    StackContract::new(
        operands.into_iter().map(OperandContract::Exact).collect(),
        SuccessContract::Fixed {
            consumed_operands,
            results,
        },
        failure,
    )
}

/// One alternative successful form that consumes every operand.
fn case(
    condition: SuccessCondition,
    consumed_operands: usize,
    results: Vec<StackValueType>,
) -> SuccessCase {
    SuccessCase::new(
        condition,
        SuccessStackEffect::new(consumed_operands, results),
    )
}

/// The two forms a whole asset or value field arrives in.
///
/// Both push a payload and then a prefix, so the prefix ends up on
/// top; what differs is the payload's width and meaning. An explicit
/// amount is eight bytes and a blinded one is thirty-two, so a
/// consumer that read only the explicit form would size the wrong
/// item.
fn explicit_or_confidential_field(
    explicit: EncodingClass,
    confidential: EncodingClass,
) -> SuccessContract {
    use StackValueType as S;

    SuccessContract::Alternatives {
        cases: vec![
            case(
                SuccessCondition::ExplicitEncoding,
                1,
                vec![S::EncodedPayload(explicit), S::EncodingPrefix(explicit)],
            ),
            case(
                SuccessCondition::ConfidentialEncoding,
                1,
                vec![
                    S::EncodedPayload(confidential),
                    S::EncodingPrefix(confidential),
                ],
            ),
        ],
    }
}

/// The two forms a locking program arrives in.
///
/// A witness program is pushed as it stands with its version above it.
/// Anything else is replaced by a digest of the program with a
/// negative version marker above it, and the marker is what a program
/// must branch on: the digest is not a program and cannot be treated
/// as one.
fn witness_or_digest_program() -> SuccessContract {
    use StackValueType as S;

    SuccessContract::Alternatives {
        cases: vec![
            case(
                SuccessCondition::WitnessProgram,
                1,
                vec![S::Encoded(EncodingClass::WitnessProgram), S::ScriptNumber],
            ),
            case(
                SuccessCondition::NonWitnessProgram,
                1,
                vec![
                    S::Encoded(EncodingClass::ScriptPubKeySha256),
                    S::ScriptNumber,
                ],
            ),
        ],
    }
}

/// The forms one issuance amount can arrive in.
fn issuance_amount_forms() -> BTreeSet<EncodingClass> {
    [
        EncodingClass::ExplicitValue,
        EncodingClass::ConfidentialValue,
    ]
    .into_iter()
    .collect()
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
            consuming(
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
            consuming(
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
            consuming(
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
            consuming(
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
                vec![OperandContract::Exact(S::ScriptNumber)],
                // The payload is pushed first and the prefix second,
                // so the prefix ends up on top. Both explicit and
                // confidential assets carry a payload of the same
                // width, which is why the prefix is the only thing
                // that distinguishes them on the stack — and why the
                // two forms must still be named, since the payload's
                // *meaning* differs entirely.
                explicit_or_confidential_field(E::ExplicitAsset, E::ConfidentialAsset),
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
                vec![OperandContract::Exact(S::ScriptNumber)],
                // An explicit amount reaches the stack little-endian
                // even though the transaction field stores it
                // big-endian, and it is eight bytes wide; a
                // confidential amount reaches it as a thirty-two byte
                // payload. The prefix is what tells them apart, and it
                // is pushed last. An absent amount is pushed in the
                // explicit form, as eight zero bytes under the
                // explicit prefix, so it needs no third form.
                explicit_or_confidential_field(E::ExplicitValue, E::ConfidentialValue),
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
                vec![OperandContract::Exact(S::ScriptNumber)],
                witness_or_digest_program(),
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
            consuming(
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
                vec![OperandContract::Exact(S::ScriptNumber)],
                // An input carrying an issuance pushes six items; an
                // input carrying none pushes a single empty item.
                //
                // The push order is the reviewed one and it is not the
                // order the fields are usually named in: the
                // inflation-keys amount goes first, the issued amount
                // second, then the entropy, then the blinding nonce.
                // The nonce is last so that an empty stack top means
                // exactly "no issuance" — which is why the absent form
                // pushes the null value marker and nothing else.
                //
                // Either amount may be explicit or blinded, and the
                // two are independent of each other. That is a
                // property of each amount rather than of the issuance,
                // so it is stated on the payload items instead of
                // multiplying out into conditions the target does not
                // branch on.
                SuccessContract::Alternatives {
                    cases: vec![
                        case(
                            SuccessCondition::IssuancePresent,
                            1,
                            vec![
                                S::EncodedPayloadAlternatives(issuance_amount_forms()),
                                S::EncodingPrefixAlternatives(issuance_amount_forms()),
                                S::EncodedPayloadAlternatives(issuance_amount_forms()),
                                S::EncodingPrefixAlternatives(issuance_amount_forms()),
                                S::Encoded(E::IssuanceEntropy),
                                S::Encoded(E::IssuanceBlindingNonce),
                            ],
                        ),
                        case(
                            SuccessCondition::IssuanceAbsent,
                            1,
                            vec![S::Encoded(E::NullValue)],
                        ),
                    ],
                },
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
            consuming(
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
                vec![OperandContract::Exact(S::ScriptNumber)],
                explicit_or_confidential_field(E::ExplicitAsset, E::ConfidentialAsset),
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
                vec![OperandContract::Exact(S::ScriptNumber)],
                explicit_or_confidential_field(E::ExplicitValue, E::ConfidentialValue),
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
                vec![OperandContract::Exact(S::ScriptNumber)],
                // The one asymmetric case among the reviewed
                // introspection primitives: the nonce arrives as a
                // single item with its prefix byte still attached,
                // where assets and values arrive split in two. An
                // absent nonce arrives as the empty item, which is a
                // third form rather than a degenerate explicit one.
                SuccessContract::Alternatives {
                    cases: vec![
                        case(
                            SuccessCondition::ExplicitEncoding,
                            1,
                            vec![S::Encoded(E::ExplicitNonce)],
                        ),
                        case(
                            SuccessCondition::ConfidentialEncoding,
                            1,
                            vec![S::Encoded(E::ConfidentialNonce)],
                        ),
                        case(
                            SuccessCondition::NullEncoding,
                            1,
                            vec![S::Encoded(E::NullNonce)],
                        ),
                    ],
                },
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
                vec![OperandContract::Exact(S::ScriptNumber)],
                witness_or_digest_program(),
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
            consuming(vec![], vec![unsigned32()], gated([])),
            plain(1),
            &[R::OpcodeSemantics, R::TransactionIntrospectionSemantics],
        ),
        spec(
            O::InspectLockTime,
            0xd3,
            consuming(vec![], vec![unsigned32()], gated([])),
            plain(1),
            &[R::OpcodeSemantics, R::TransactionIntrospectionSemantics],
        ),
        spec(
            O::InspectNumInputs,
            0xd4,
            consuming(
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
            consuming(
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
            consuming(
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
            consuming(
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
            consuming(
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
            consuming(
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
            consuming(
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
            consuming(
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
            consuming(
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
            consuming(
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
            consuming(
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
            consuming(
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
            consuming(
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
            consuming(
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
            consuming(
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
            consuming(
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
        // The tweak position admits on width alone, and the reason is
        // the target's own guard: `vchTweak.size() != 32` is the entire
        // operand check, and what the bytes mean is decided afterwards
        // by `CheckPayToContract`
        // (`src/script/interpreter.cpp:2206-2220`). Declaring it as one
        // exact encoding refused the digest a program actually derives
        // there, which the target accepts
        // (´[PLAN-rule:guide10:tweak-totality]´).
        spec(
            O::TweakVerify,
            0xe4,
            StackContract::new(
                vec![
                    OperandContract::Exact(S::Encoded(E::CompressedPublicKey)),
                    OperandContract::WidthOnly {
                        bytes: NonZeroUsize::new(32).expect("thirty-two is not zero"),
                        intent: E::TaprootTweak,
                    },
                    OperandContract::Exact(S::Encoded(E::XOnlyPublicKey)),
                ],
                SuccessContract::Fixed {
                    consumed_operands: 3,
                    results: vec![],
                },
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
    use crate::evidence::TargetEvidenceRequirementId as R;
    use OpcodeId as O;
    use StackValueType as S;

    vec![
        // -- Signature verification ----------------------------
        //
        // Three reviewed branches, none of which collapses into
        // another (Guide-10 rule:guide10:signature-abstraction).
        //
        // An empty signature is a documented path, not a malformed
        // operand: the pushing form consumes the operands and pushes a
        // false, and the verifying form aborts. A nonempty signature
        // that does not verify against a recognized key aborts in both
        // forms, so a backend cannot treat "verification failed" as a
        // branchable condition. And a nonempty key of an unrecognized
        // form is not a rejection at all: the check succeeds without
        // verifying anything, which is the target's own
        // forward-compatibility rule. Only the empty key is refused
        // outright.
        //
        // The operand positions carry those alternatives because no
        // single stack type can: an exact 64-byte signature type
        // excludes the empty item the target accepts there, and an
        // exact x-only key type excludes the unrecognized forms the
        // target succeeds on.
        spec(
            O::CheckSig,
            0xac,
            signature_check(schnorr_signature(), any_public_key(), vec![S::Bool], false),
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
            // The verifying form leaves no branchable result: an empty
            // signature that would have pushed a false aborts here
            // instead.
            signature_check(schnorr_signature(), any_public_key(), vec![], true),
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
            signature_check_from_stack(vec![S::Bool], false),
            budgeted(-2),
            &[R::OpcodeSemantics, R::SignatureSemantics],
        ),
        spec(
            O::CheckSigFromStackVerify,
            0xc2,
            signature_check_from_stack(vec![], true),
            budgeted(-2),
            &[R::OpcodeSemantics, R::SignatureSemantics],
        ),
    ]
}

/// The signature operand of the reviewed signature primitives.
const fn schnorr_signature() -> OperandContract {
    OperandContract::Signature {
        nonempty_encoding: EncodingClass::SchnorrSignature,
        empty_allowed: true,
    }
}

/// The public-key operand of the reviewed signature primitives.
const fn any_public_key() -> OperandContract {
    OperandContract::PublicKey {
        recognized_encoding: EncodingClass::XOnlyPublicKey,
        unknown_nonempty_allowed: true,
    }
}

/// The two successful forms and five failure paths every reviewed
/// signature primitive shares.
///
/// `aborts_on_empty_signature` is what separates the verifying forms
/// from the pushing ones, and it is the only thing that does: both push
/// nothing else and both fail on everything else alike.
fn signature_check(
    signature: OperandContract,
    public_key: OperandContract,
    results: Vec<StackValueType>,
    aborts_on_empty_signature: bool,
) -> StackContract {
    signature_check_over(
        vec![signature, public_key],
        results,
        aborts_on_empty_signature,
    )
}

/// The same, for the forms that verify a signature over a stack item.
fn signature_check_from_stack(
    results: Vec<StackValueType>,
    aborts_on_empty_signature: bool,
) -> StackContract {
    signature_check_over(
        vec![
            schnorr_signature(),
            OperandContract::Exact(any_bytes()),
            any_public_key(),
        ],
        results,
        aborts_on_empty_signature,
    )
}

/// One signature primitive's complete stack contract.
fn signature_check_over(
    operands: Vec<OperandContract>,
    results: Vec<StackValueType>,
    aborts_on_empty_signature: bool,
) -> StackContract {
    let consumed_operands = operands.len();
    let empty_signature = if aborts_on_empty_signature {
        FailureOutcome::AbortEvaluation
    } else {
        FailureOutcome::ConsumeOperandsPushFalse
    };

    StackContract::new(
        operands,
        SuccessContract::Alternatives {
            cases: vec![
                case(
                    SuccessCondition::RecognizedKeyVerifiedSignature,
                    consumed_operands,
                    results.clone(),
                ),
                case(
                    SuccessCondition::UnknownKeyTypeUnverified,
                    consumed_operands,
                    results,
                ),
            ],
        },
        FailureContract::new([
            abort(FailureCause::StackUnderflow),
            abort(FailureCause::EmptyPublicKey),
            abort(FailureCause::InvalidPublicKeyEncoding),
            abort(FailureCause::ValidationBudgetExhausted),
            abort(FailureCause::InvalidSignature),
            FailureEffect::new(FailureCause::EmptySignature, empty_signature),
        ]),
    )
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
                // Five bytes, not four. The operand is compared against
                // an unsigned thirty-two bit sequence field, and the
                // flag bit that disables the check sits above that
                // field's range, so the target reads this one operand at
                // the wider width. Typing it as the ordinary script
                // number said a five-byte operand is malformed, which
                // made the disable-flag behavior unstateable and would
                // have refused a program the target accepts.
                vec![OperandContract::Exact(S::Encoded(
                    EncodingClass::LockTimeScriptNumber,
                ))],
                // The operand is inspected and left in place, and
                // nothing is pushed above it: a successful check
                // leaves the stack exactly as it found it. Recording
                // this as a consuming form with no results would have
                // described a net reduction of one, contradicting the
                // resource row's growth of zero and mis-scheduling
                // every program that uses a relative timelock.
                SuccessContract::RetainsOperands { results: vec![] },
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

/// A rearranging contract over `operands` positions that constrain
/// nothing.
///
/// The ordinary stack operations share this whole shape: every operand
/// is any item at all, every result is one of those items carried
/// through, and the only way they fail is by not being given enough to
/// work with.
fn rearranging(operands: usize, consumed_operands: usize, results: &[usize]) -> StackContract {
    StackContract::new(
        vec![OperandContract::AnyItem; operands],
        SuccessContract::OperandResolved {
            consumed_operands,
            results: results
                .iter()
                .copied()
                .map(ResultValue::OperandCopy)
                .collect(),
        },
        gated([abort(FailureCause::StackUnderflow)]),
    )
}

/// Part of the reviewed primitive registry.
///
/// # Why these are not typed by what they move
///
/// A duplicate does not know what it duplicated. Each position here
/// constrains nothing and each result names the position it came from,
/// so a fixed-width integer stays a fixed-width integer across a swap
/// instead of being widened to an anonymous byte string
/// `(´[PLAN-rule:guide10:primitive-admission]´)`.
fn stack_opcodes() -> Vec<(OpcodeId, OpcodeSpec)> {
    use crate::evidence::TargetEvidenceRequirementId as R;
    use OpcodeId as O;

    let evidence: &[TargetEvidenceRequirementId] =
        &[R::OpcodeSemantics, R::StackRearrangementSemantics];

    vec![
        // Results are listed in push order, and each entry is the
        // deepest-first index of the declared operand it copies.
        spec(
            O::Duplicate,
            0x76,
            rearranging(1, 1, &[0, 0]),
            plain(1),
            evidence,
        ),
        spec(
            O::DuplicateTwo,
            0x6e,
            rearranging(2, 2, &[0, 1, 0, 1]),
            plain(2),
            evidence,
        ),
        spec(
            O::CopyOver,
            0x78,
            rearranging(2, 2, &[0, 1, 0]),
            plain(1),
            evidence,
        ),
        spec(
            O::Swap,
            0x7c,
            rearranging(2, 2, &[1, 0]),
            plain(0),
            evidence,
        ),
        spec(
            O::Rotate,
            0x7b,
            rearranging(3, 3, &[1, 2, 0]),
            plain(0),
            evidence,
        ),
        spec(
            O::RemoveSecond,
            0x77,
            rearranging(2, 2, &[1]),
            plain(-1),
            evidence,
        ),
        spec(
            O::Tuck,
            0x7d,
            rearranging(2, 2, &[1, 0, 1]),
            plain(1),
            evidence,
        ),
        spec(O::Drop, 0x75, rearranging(1, 1, &[]), plain(-1), evidence),
        spec(
            O::DropTwo,
            0x6d,
            rearranging(2, 2, &[]),
            plain(-2),
            evidence,
        ),
    ]
}

/// Part of the reviewed primitive registry.
fn verification_opcodes() -> Vec<(OpcodeId, OpcodeSpec)> {
    use crate::evidence::TargetEvidenceRequirementId as R;
    use FailureCause as C;
    use OpcodeId as O;
    use StackValueType as S;

    let evidence: &[TargetEvidenceRequirementId] = &[R::OpcodeSemantics, R::VerificationSemantics];

    vec![
        // Equality compares the operands byte for byte, with no
        // numeric interpretation whatever: two script numbers of equal
        // value in different encodings are unequal here. That is why
        // the operands constrain nothing — there is no type for which
        // this primitive means something different.
        spec(
            O::Equal,
            0x87,
            StackContract::new(
                vec![OperandContract::AnyItem, OperandContract::AnyItem],
                // Inequality is a successful false, exactly as it is
                // for the fixed-width comparisons. Recording it as a
                // failure would make a program that legitimately
                // branches on inequality look like one that failed.
                SuccessContract::Fixed {
                    consumed_operands: 2,
                    results: vec![S::Bool],
                },
                gated([abort(C::StackUnderflow)]),
            ),
            plain(-1),
            evidence,
        ),
        // The verifying form consumes both operands and pushes
        // nothing, and inequality ends evaluation. The transient truth
        // value the target pushes before popping it again is not a
        // state a program can observe, so it is not in the contract.
        spec(
            O::EqualVerify,
            0x88,
            StackContract::new(
                vec![OperandContract::AnyItem, OperandContract::AnyItem],
                SuccessContract::Fixed {
                    consumed_operands: 2,
                    results: vec![],
                },
                gated([abort(C::StackUnderflow), abort(C::UnequalOperands)]),
            ),
            plain(-2),
            evidence,
        ),
        spec(
            O::Verify,
            0x69,
            StackContract::new(
                vec![OperandContract::AnyItem],
                SuccessContract::Fixed {
                    consumed_operands: 1,
                    results: vec![],
                },
                // A false operand aborts and is *not* consumed. The
                // distinction does not reach the contract because
                // nothing survives to observe the depth, which is why
                // this is an aborting effect rather than a retaining
                // one.
                gated([abort(C::StackUnderflow), abort(C::FalseVerification)]),
            ),
            plain(-1),
            evidence,
        ),
    ]
}

/// Part of the reviewed primitive registry.
fn byte_string_opcodes() -> Vec<(OpcodeId, OpcodeSpec)> {
    use crate::evidence::TargetEvidenceRequirementId as R;
    use FailureCause as C;
    use OpcodeId as O;
    use StackValueType as S;

    let evidence: &[TargetEvidenceRequirementId] = &[R::OpcodeSemantics, R::ByteStringSemantics];

    vec![
        // Concatenation is the one reviewed primitive that checks the
        // literal bound against a value it computed. The operands may
        // each be admissible and their join not be, so the bound is a
        // failure of this primitive rather than of whatever pushed the
        // operands.
        spec(
            O::Concatenate,
            0x7e,
            StackContract::new(
                vec![OperandContract::AnyItem, OperandContract::AnyItem],
                SuccessContract::Fixed {
                    consumed_operands: 2,
                    results: vec![any_bytes()],
                },
                gated([abort(C::StackUnderflow), abort(C::ResultSizeExceeded)]),
            ),
            plain(-1),
            evidence,
        ),
        // The width is pushed *above* the item, which stays where it
        // was. A contract that consumed the operand would have every
        // caller of this primitive scheduling one item too few.
        spec(
            O::Size,
            0x82,
            StackContract::new(
                vec![OperandContract::AnyItem],
                SuccessContract::OperandResolved {
                    consumed_operands: 1,
                    results: vec![
                        ResultValue::OperandCopy(0),
                        ResultValue::Computed(S::ScriptNumber),
                    ],
                },
                gated([abort(C::StackUnderflow)]),
            ),
            plain(1),
            evidence,
        ),
        // The slice must lie wholly within the operand: a start at or
        // past the end, a negative bound, or a length running past the
        // end all end evaluation rather than clamping. The lazy
        // variant that clamps instead is a different target byte and
        // is not reviewed here.
        //
        // Because it never clamps, the successful result is exactly as
        // wide as the length operand asks for — every other case is one
        // of the aborts below. The contract states that dependency
        // rather than reporting an unconstrained byte string a caller
        // would then have to guess the width of.
        spec(
            O::Substring,
            0x7f,
            StackContract::new(
                vec![
                    OperandContract::AnyItem,
                    OperandContract::Exact(S::ScriptNumber),
                    OperandContract::Exact(S::ScriptNumber),
                ],
                SuccessContract::OperandResolved {
                    consumed_operands: 3,
                    results: vec![ResultValue::ComputedWidthFromOperand {
                        width_operand: 2,
                        unsettled: any_bytes(),
                    }],
                },
                gated([
                    abort(C::StackUnderflow),
                    abort(C::MalformedScriptNumber),
                    abort(C::SliceOutOfRange),
                ]),
            ),
            plain(-2),
            evidence,
        ),
        spec(O::BitwiseAnd, 0x84, bitwise(), plain(-1), evidence),
        spec(O::BitwiseXor, 0x86, bitwise(), plain(-1), evidence),
    ]
}

/// The contract the reviewed bitwise primitives share.
///
/// They pair their operands byte for byte and refuse a width
/// disagreement outright rather than padding either side, so the
/// result is exactly as wide as both operands were.
fn bitwise() -> StackContract {
    StackContract::new(
        vec![OperandContract::AnyItem, OperandContract::AnyItem],
        SuccessContract::Fixed {
            consumed_operands: 2,
            results: vec![any_bytes()],
        },
        gated([
            abort(FailureCause::StackUnderflow),
            abort(FailureCause::MismatchedOperandWidths),
        ]),
    )
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
        stack_opcodes(),
        verification_opcodes(),
        byte_string_opcodes(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// The largest byte string the target admits as a literal script push,
/// as an initial witness item, and as a concatenated result.
///
/// The reviewed execution domain does not re-check this bound against
/// computed results *generally* — an arithmetic or hashing result is
/// not measured against it. Concatenation is the one reviewed
/// primitive that does check it, refusing a join whose width exceeds
/// this bound even though both operands were admissible, so the bound
/// is no longer only an entry-point rule.
pub const MAX_STACK_ELEMENT_BYTES: usize = 520;
