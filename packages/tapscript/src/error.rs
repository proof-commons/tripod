//! The adapter's typed error root.
//!
//! # Only failures that can happen
//!
//! A variant exists only where some branch in this crate constructs it,
//! and several failures a reader of the guide's error sketch might
//! expect are deliberately absent because no input reaches them:
//!
//! - an unsupported instruction, an unsupported target contract
//!   revision, and an unreviewed target definition are unconstructible.
//!   Every entry point accepts the reviewed Elements contract, which
//!   has no public constructor, states a contract for every member of
//!   the reviewed primitive census, and gates each one to the domain it
//!   describes.
//! - a duplicate opcode byte is unconstructible for the same reason:
//!   the target validator refuses a definition where two primitives
//!   claim one byte, so the parser's byte table cannot collide.
//! - trailing instruction bytes are not a separate outcome of this
//!   parser. Every byte belongs to the instruction that consumed it, so
//!   a partial instruction at the end of a script is exactly the
//!   truncation case and is reported as one.
//!
//! # The set-level assessment
//!
//! Four of the variants below belong to the capability adapter, because
//! four of its failures are reachable. The set-level
//! assessment builds one map per published census, and the two ways
//! either map can disagree with the census it was built from — a member
//! assessed twice, and a key set that is not the census — are checks
//! that run on every call and that a test drives to failure. The
//! capability census and the external-evidence-role census get their
//! own variants rather than a shared one carrying a kind: a reader of a
//! failure should not have to decode which census broke.
//!
//! Several failure classes a reader might expect are deliberately
//! absent, because no input reaches them:
//!
//! - a missing target primitive is an *assessment*, not an error. It is
//!   a fact about the target that the caller must see next to every
//!   other capability's disposition, not an exception that discards the
//!   rest of the census.
//! - a missing target evidence requirement is unconstructible. The
//!   target validator refuses a contract whose evidence registry is not
//!   the complete census, so no validated definition exists that omits
//!   one. A variant for it would read as a check that is running when
//!   nothing can trigger it.
//! - an unsupported target contract revision is unconstructible for the
//!   same reason: the revision type has no unchecked constructor.
//! - an unreviewed target definition cannot arrive here, because the
//!   public assessment entry points accept only the reviewed Elements
//!   contract, and the target package alone constructs that wrapper.

use std::fmt;

use compiler::target::{ExternalEvidenceRole, RequiredCapability};
use target_elements::{EncodingClass, OperandContract, StackValueType};

/// A typed adapter failure.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TapscriptError {
    /// A literal exceeded the largest one the target accepts on its
    /// stack.
    OversizedStackItem {
        /// The width that was offered.
        offered: usize,
        /// The largest width the target accepts.
        maximum: usize,
    },

    /// A script number needed more bytes than the target's script
    /// number admits, so it has no representation to push.
    ScriptNumberOutOfRange {
        /// The value that was offered.
        offered: i64,
    },

    /// A payload was not a width the encoding class admits, so it is
    /// not a value of that class at all.
    MalformedEncodedItem {
        /// The class the payload was offered for.
        class: EncodingClass,
    },

    /// A payload of a class the target requires to be minimally encoded
    /// was not in its minimal form.
    NonMinimalScriptNumber,

    /// A program carried more instructions than one program may.
    InstructionLimitExceeded {
        /// The limit that was exceeded.
        maximum: u64,
    },

    /// A script byte named neither a reviewed primitive nor a push
    /// form. It is refused rather than skipped: a byte nobody reviewed
    /// carries semantics this package cannot speak for.
    UnknownOpcodeByte(u8),

    /// A push ran off the end of the script, so the script ends in the
    /// middle of an instruction.
    TruncatedInstruction,

    /// An instruction needed more operands than the stack reaching it
    /// carries.
    ///
    /// The index is ephemeral diagnostic context naming where in this
    /// program the defect is. It is not an identity, it does not
    /// survive an edit, and it appears in no projection.
    StackUnderflow {
        /// Where in the program the instruction sits.
        instruction: usize,
    },

    /// An operand was not a value the instruction's declared operand
    /// admits.
    StackTypeMismatch {
        /// Where in the program the instruction sits.
        instruction: usize,
        /// The operand position the contract declares.
        ///
        /// The whole position rather than one type: a signature or key
        /// position admits alternatives, and naming one of them would
        /// misreport what the program was refused against.
        expected: OperandContract,
        /// The operand the stack carries.
        actual: StackValueType,
    },

    /// A validation reached its state budget, so there is no complete
    /// state set to return and no partial one is offered.
    AbstractStateLimitExceeded {
        /// The limit that was reached.
        maximum: u64,
    },

    /// A state grew deeper than the target admits.
    StackLimitExceeded {
        /// The limit that was exceeded.
        maximum: u64,
    },

    /// A result carried more alternatives than the budget admits.
    ResultAlternativeLimitExceeded {
        /// The limit that was exceeded.
        maximum: u64,
    },

    /// A payload was carried in a form that is not its minimal one.
    ///
    /// Stricter than the target's own validity rules, which enforce
    /// minimality only under the standardness rules a node applies to
    /// what it relays.
    NonMinimalPush,

    /// One compiler capability was assessed more than once.
    ///
    /// Two assessments of one capability are two answers to one
    /// question, and nothing downstream could choose between them.
    DuplicateCapabilityAssessment(RequiredCapability),

    /// The assessed capabilities are not exactly the required ones.
    ///
    /// Both directions matter. A missing assessment silently drops a
    /// requirement the analysis published, which is the weakening this
    /// adapter exists to prevent; an unexpected one answers a question
    /// no analysis asked, which would let an assessment set look
    /// broader than the analysis behind it.
    CapabilityAssessmentCensusMismatch {
        /// Required capabilities with no assessment, in census order.
        missing: Vec<RequiredCapability>,
        /// Assessed capabilities nothing required, in census order.
        unexpected: Vec<RequiredCapability>,
    },

    /// One compiler evidence role was assessed more than once.
    ///
    /// As for a capability: two assessments of one role are two answers
    /// to one question, and nothing downstream could choose between
    /// them.
    DuplicateEvidenceAssessment(ExternalEvidenceRole),

    /// The assessed evidence roles are not exactly the required ones.
    ///
    /// Both directions matter, for the reason the capability census
    /// gives. A missing role assessment is precisely the silent drop
    /// this census was added to prevent: the compiler's evidence
    /// boundary would reach the adapter and stop there.
    EvidenceAssessmentCensusMismatch {
        /// Required roles with no assessment, in census order.
        missing: Vec<ExternalEvidenceRole>,
        /// Assessed roles nothing required, in census order.
        unexpected: Vec<ExternalEvidenceRole>,
    },

    /// One operation requirement was assessed more than once.
    ///
    /// The plan publishes its censuses as sets, so a repeat means the
    /// plan itself carried a duplicate. Reported rather than collapsed:
    /// a census that quietly deduplicated would report a smaller total
    /// than the plan it claims to answer.
    DuplicateOperationRequirement,
}

impl fmt::Display for TapscriptError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OversizedStackItem { offered, maximum } => write!(
                formatter,
                "a literal of {offered} bytes exceeds the {maximum} byte limit",
            ),
            Self::ScriptNumberOutOfRange { offered } => {
                write!(formatter, "{offered} is outside the script number range")
            }
            Self::MalformedEncodedItem { class } => {
                write!(formatter, "the payload is not a valid {class:?}")
            }
            Self::NonMinimalScriptNumber => {
                write!(formatter, "the script number is not minimally encoded")
            }
            Self::InstructionLimitExceeded { maximum } => {
                write!(
                    formatter,
                    "a program carries at most {maximum} instructions"
                )
            }
            Self::UnknownOpcodeByte(opcode) => {
                write!(
                    formatter,
                    "byte {opcode:#04x} names no reviewed instruction"
                )
            }
            Self::TruncatedInstruction => {
                write!(formatter, "the script ends inside an instruction")
            }
            Self::NonMinimalPush => {
                write!(formatter, "the literal is not pushed in its minimal form")
            }
            Self::StackUnderflow { instruction } => write!(
                formatter,
                "instruction {instruction} needs more operands than the stack carries",
            ),
            Self::StackTypeMismatch {
                instruction,
                expected,
                actual,
            } => write!(
                formatter,
                "instruction {instruction} expects {expected:?} and the stack carries {actual:?}",
            ),
            Self::AbstractStateLimitExceeded { maximum } => {
                write!(formatter, "a validation may visit at most {maximum} states")
            }
            Self::StackLimitExceeded { maximum } => {
                write!(formatter, "a stack may hold at most {maximum} items")
            }
            Self::ResultAlternativeLimitExceeded { maximum } => write!(
                formatter,
                "a result may carry at most {maximum} alternatives",
            ),
            Self::DuplicateOperationRequirement => write!(
                formatter,
                "one operation requirement was assessed more than once",
            ),
            Self::DuplicateCapabilityAssessment(capability) => {
                write!(formatter, "capability {capability:?} was assessed twice")
            }
            Self::CapabilityAssessmentCensusMismatch {
                missing,
                unexpected,
            } => write!(
                formatter,
                "capability assessment census mismatch: {} missing, {} unexpected",
                missing.len(),
                unexpected.len(),
            ),
            Self::DuplicateEvidenceAssessment(role) => {
                write!(formatter, "evidence role {role:?} was assessed twice")
            }
            Self::EvidenceAssessmentCensusMismatch {
                missing,
                unexpected,
            } => write!(
                formatter,
                "evidence assessment census mismatch: {} missing, {} unexpected",
                missing.len(),
                unexpected.len(),
            ),
        }
    }
}

impl std::error::Error for TapscriptError {}
