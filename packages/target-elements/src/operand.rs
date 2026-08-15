//! What one primitive accepts in one operand position.
//!
//! # Why an operand is not just a stack type
//!
//! Most operands are one shape: a primitive either gets a 64-bit
//! little-endian item there or it fails. A signature operand is not
//! like that. The reviewed target accepts an empty item *and* a
//! 64-byte signature in the same position, and takes a different path
//! for each — the empty one is a documented, non-aborting failure, not
//! a malformed operand. A public-key operand is not like that either:
//! the reviewed encoding is one 32-byte form, and any *other* nonempty
//! form is the target's forward-compatibility path, which succeeds
//! without verifying anything.
//!
//! Declaring those positions as one exact stack type could not state
//! either behaviour (Guide-10 `rule:guide10:signature-abstraction`).
//! An exact 64-byte signature type excludes the empty item, so the
//! empty-signature path was refused as a type mismatch before its own
//! failure effect could apply; declaring the position as the encoded
//! signature and letting an empty item through instead would have said
//! the exact-width encoding admits emptiness, which it does not. The
//! same held for keys: an exact x-only type put the unknown-key path
//! outside the operand domain entirely, so the abstract validator
//! rejected the very case the target documents as succeeding.
//!
//! So the alternatives live in the operand contract, where the target's
//! own branch structure is, rather than being flattened into a single
//! type that has to be either too narrow or too wide.

use std::collections::BTreeSet;

use crate::encoding::EncodingClass;
use crate::opcode::StackValueType;

/// What one primitive accepts in one operand position.
///
/// # Exhaustive on purpose
///
/// Unlike the stack-type vocabulary this is not marked non-exhaustive.
/// Each alternative here selects different target behaviour, so a
/// consumer matching on it with a catch-all arm would be silently
/// treating a position it has not been taught as an ordinary one. A new
/// alternative should break such a consumer, and does.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OperandContract {
    /// Exactly one stack type, which is the ordinary case.
    Exact(StackValueType),

    /// Any one of several stack types, with no further structure.
    ///
    /// For a position whose admissible forms are alternatives of equal
    /// standing. A signature or key position is not this: its
    /// alternatives select different target behaviour, which is what
    /// the two variants below carry and this one does not.
    OneOf(BTreeSet<StackValueType>),

    /// A signature position.
    Signature {
        /// The encoding a nonempty signature must carry.
        nonempty_encoding: EncodingClass,
        /// Whether an empty item is admissible here.
        ///
        /// Admissible does not mean successful. Where this holds, the
        /// empty item reaches the primitive's own empty-signature
        /// failure effect — which pushes a false, or aborts in the
        /// verifying form — rather than being refused as an operand of
        /// the wrong type.
        empty_allowed: bool,
    },

    /// A public-key position.
    PublicKey {
        /// The one key encoding the target verifies against.
        recognized_encoding: EncodingClass,
        /// Whether a nonempty key of an unrecognized form is
        /// admissible.
        ///
        /// This is the target's forward-compatibility rule, and it is
        /// a success path rather than a tolerance: the check succeeds
        /// without verifying anything. An empty key is never admitted
        /// by it — emptiness is rejected outright, which is why the
        /// two are separate conditions and not one "unrecognized" one.
        unknown_nonempty_allowed: bool,
    },
}

impl OperandContract {
    /// The ordinary single-type operand.
    #[must_use]
    pub const fn exact(value: StackValueType) -> Self {
        Self::Exact(value)
    }

    /// The stack types this position names explicitly.
    ///
    /// The unknown-key form is deliberately absent: it is every
    /// nonempty encoding the target does *not* recognize, which is not
    /// a stack type and cannot be enumerated as one.
    #[must_use]
    pub fn named_types(&self) -> BTreeSet<StackValueType> {
        match self {
            Self::Exact(value) => BTreeSet::from([value.clone()]),
            Self::OneOf(values) => values.clone(),
            Self::Signature {
                nonempty_encoding,
                empty_allowed,
            } => {
                let mut types = BTreeSet::from([StackValueType::Encoded(*nonempty_encoding)]);
                if *empty_allowed {
                    types.insert(StackValueType::Empty);
                }
                types
            }
            Self::PublicKey {
                recognized_encoding,
                ..
            } => BTreeSet::from([StackValueType::Encoded(*recognized_encoding)]),
        }
    }

    /// The encoding classes this position names.
    #[must_use]
    pub fn named_encodings(&self) -> BTreeSet<EncodingClass> {
        self.named_types()
            .iter()
            .filter_map(|value| match value {
                StackValueType::Encoded(class)
                | StackValueType::EncodedPayload(class)
                | StackValueType::EncodingPrefix(class) => Some(*class),
                _ => None,
            })
            .collect()
    }

    /// The single stack type this position admits, where it admits one.
    ///
    /// A position with alternatives has no single type, and says so
    /// rather than offering the first of them.
    #[must_use]
    pub const fn as_exact(&self) -> Option<&StackValueType> {
        match self {
            Self::Exact(value) => Some(value),
            _ => None,
        }
    }
}

impl From<StackValueType> for OperandContract {
    fn from(value: StackValueType) -> Self {
        Self::Exact(value)
    }
}

/// What an incoming operand can be, in a signature position.
///
/// Both fields can hold at once: an abstract state that has not decided
/// the item's width admits both paths, and a validator that picked one
/// would be inventing a fact the state does not carry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SignatureOperandFacts {
    /// Whether the item can be the empty one.
    pub can_be_empty: bool,
    /// Whether the item can be a nonempty signature.
    pub can_be_nonempty: bool,
}

/// What an incoming operand can be, in a public-key position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublicKeyOperandFacts {
    /// Whether the item can be the empty one, which the target
    /// rejects.
    pub can_be_empty: bool,
    /// Whether the item can be the recognized key encoding, which the
    /// target verifies against.
    pub can_be_recognized: bool,
    /// Whether the item can be a nonempty key of an unrecognized form,
    /// which the target succeeds on without verifying.
    pub can_be_unknown_nonempty: bool,
}
