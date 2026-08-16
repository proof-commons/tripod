//! Typed target instructions and the stack items they push.
//!
//! # There is no raw instruction
//!
//! A [`TapscriptInstruction`] is a reviewed primitive identity or a
//! checked literal, and the safe construction API offers nothing else:
//! no raw opcode byte, no raw instruction, no raw program. Untrusted
//! bytes enter through the parser in [`crate::program`] and either
//! become typed instructions or fail, so there is no path by which a
//! byte nobody reviewed reaches a program that is then treated as
//! validated.
//!
//! # What a stack item is, and what it is not
//!
//! A [`StackItem`] is a byte string the target admits as a literal. Its
//! typed constructors exist where the target contract fixes an
//! encoding, and they resolve the width, the byte order, and the
//! canonical form from that contract rather than restating them.
//!
//! They carry no protocol meaning whatever. There is no receipt value
//! here, no owner, no amount of anything: those are the attestation
//! contract's semantics, and a constructor named for one would put
//! protocol meaning inside a package whose entire subject is the
//! target.

use target_elements::{
    ByteOrder, CanonicalEncodingRule, EncodingClass, EncodingSpec, OpcodeId, PayloadWidth,
    ReviewedElementsTapscriptDefinition,
};

use crate::error::TapscriptError;

/// One typed instruction of a target program.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TapscriptInstruction {
    /// A reviewed primitive.
    Opcode(OpcodeId),
    /// A literal pushed onto the stack.
    Push(StackItem),
}

/// A byte string the target admits as a stack literal.
///
/// The field is private and every constructor checks the target's
/// literal bound, so an oversized item does not exist to be pushed.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StackItem {
    bytes: Vec<u8>,
}

impl StackItem {
    /// Accepts an arbitrary byte string as a literal.
    ///
    /// # Errors
    ///
    /// [`TapscriptError::OversizedStackItem`] when the bytes exceed the
    /// largest literal the reviewed contract accepts.
    pub fn new(
        target: &ReviewedElementsTapscriptDefinition,
        bytes: Vec<u8>,
    ) -> Result<Self, TapscriptError> {
        let maximum = target.definition().pushes().maximum_payload_bytes();
        if bytes.len() > maximum {
            return Err(TapscriptError::OversizedStackItem {
                offered: bytes.len(),
                maximum,
            });
        }
        Ok(Self { bytes })
    }

    /// The empty item.
    ///
    /// The target uses it as both a false and an absent-field marker,
    /// and it needs no bound check because nothing is narrower.
    #[must_use]
    pub const fn empty() -> Self {
        Self { bytes: Vec::new() }
    }

    /// Encodes `value` as the script language's variable-width number.
    ///
    /// The encoding is the canonical minimal one, so the item is
    /// already in the form the target's minimality rule requires.
    ///
    /// # Errors
    ///
    /// [`TapscriptError::ScriptNumberOutOfRange`] when the value needs
    /// more bytes than the reviewed contract admits for a script
    /// number.
    pub fn script_number(
        target: &ReviewedElementsTapscriptDefinition,
        value: i64,
    ) -> Result<Self, TapscriptError> {
        let spec = encoding(target, EncodingClass::ScriptNumber);
        let bytes = order_bytes(spec, script_number_bytes(value));
        if !admits_width(spec.payload(), bytes.len()) {
            return Err(TapscriptError::ScriptNumberOutOfRange { offered: value });
        }
        Ok(Self { bytes })
    }

    /// Encodes `value` as the target's signed fixed-width integer.
    #[must_use]
    pub fn signed_le64(target: &ReviewedElementsTapscriptDefinition, value: i64) -> Self {
        Self {
            bytes: order_bytes(
                encoding(target, EncodingClass::SignedLittleEndian64),
                value.to_le_bytes().to_vec(),
            ),
        }
    }

    /// Encodes `value` as the target's unsigned 32-bit integer.
    #[must_use]
    pub fn unsigned_le32(target: &ReviewedElementsTapscriptDefinition, value: u32) -> Self {
        Self {
            bytes: order_bytes(
                encoding(target, EncodingClass::UnsignedLittleEndian32),
                value.to_le_bytes().to_vec(),
            ),
        }
    }

    /// Encodes `value` as the target's unsigned 64-bit integer.
    #[must_use]
    pub fn unsigned_le64(target: &ReviewedElementsTapscriptDefinition, value: u64) -> Self {
        Self {
            bytes: order_bytes(
                encoding(target, EncodingClass::UnsignedLittleEndian64),
                value.to_le_bytes().to_vec(),
            ),
        }
    }

    /// Accepts the payload of one reviewed encoding class.
    ///
    /// The bytes are the field's payload without any prefix byte, and
    /// they are checked against the width the reviewed contract fixes
    /// for the class, and against its canonical form where the contract
    /// requires a minimal one.
    ///
    /// # Errors
    ///
    /// [`TapscriptError::MalformedEncodedItem`] when the width is not
    /// one the class admits, [`TapscriptError::NonMinimalScriptNumber`]
    /// when the class requires a minimal encoding and the bytes are not
    /// in it, and [`TapscriptError::OversizedStackItem`] when the bytes
    /// exceed the target's literal bound.
    pub fn encoded(
        target: &ReviewedElementsTapscriptDefinition,
        class: EncodingClass,
        bytes: Vec<u8>,
    ) -> Result<Self, TapscriptError> {
        let spec = encoding(target, class);
        if !admits_width(spec.payload(), bytes.len()) {
            return Err(TapscriptError::MalformedEncodedItem { class });
        }
        if spec.canonicality() == CanonicalEncodingRule::Minimal
            && !is_minimal_script_number(&order_bytes(spec, bytes.clone()))
        {
            return Err(TapscriptError::NonMinimalScriptNumber);
        }
        Self::new(target, bytes)
    }

    /// The item's exact bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// How many bytes the item occupies on the stack.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Whether the item is the empty one.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// The script number this item carries, where it carries one.
    ///
    /// # Only the canonical form counts
    ///
    /// The answer is `None` for any item the target's own minimality
    /// rule would reject, and for any item wider than the reviewed
    /// script-number encoding admits. A non-minimal item is one the
    /// target aborts on rather than one that carries a number, and
    /// reporting a value for it would let a consumer reason about a
    /// program the target never runs.
    ///
    /// This is the inverse of [`Self::script_number`] over that
    /// canonical domain, and it exists so a consumer can settle a
    /// literal a program pushed — a slice bound, for instance — instead
    /// of treating every pushed number as an unknown.
    #[must_use]
    pub fn script_number_value(&self, target: &ReviewedElementsTapscriptDefinition) -> Option<i64> {
        let spec = encoding(target, EncodingClass::ScriptNumber);
        if !admits_width(spec.payload(), self.bytes.len()) {
            return None;
        }
        let little_endian = order_bytes(spec, self.bytes.clone());
        if !is_minimal_script_number(&little_endian) {
            return None;
        }
        script_number_from_bytes(&little_endian)
    }
}

/// The value of a minimal little-endian script number.
///
/// Sign and magnitude, mirroring [`script_number_bytes`]: the top bit
/// of the last byte is the sign and the rest is the magnitude, written
/// least significant byte first. Anything wide enough to overflow the
/// magnitude has no value here rather than a wrapped one.
fn script_number_from_bytes(little_endian: &[u8]) -> Option<i64> {
    let [rest @ .., top] = little_endian else {
        return Some(0);
    };
    if rest.len() >= 8 {
        return None;
    }

    let mut magnitude: u64 = 0;
    for (index, byte) in rest.iter().enumerate() {
        magnitude |= u64::from(*byte) << (8 * index);
    }
    magnitude |= u64::from(top & 0x7f) << (8 * rest.len());

    let value = i64::try_from(magnitude).ok()?;
    Some(if top & 0x80 == 0 { value } else { -value })
}

/// The reviewed contract of one encoding class.
///
/// Total by construction: the target validator refuses a definition
/// whose encoding registry is not the complete census, so the reviewed
/// contract states every class.
fn encoding(
    target: &ReviewedElementsTapscriptDefinition,
    class: EncodingClass,
) -> &'_ EncodingSpec {
    target
        .definition()
        .encodings()
        .get(&class)
        .expect("the reviewed contract states every encoding class")
}

/// Lays `little_endian` out in the order the class declares.
fn order_bytes(spec: &EncodingSpec, little_endian: Vec<u8>) -> Vec<u8> {
    match spec.byte_order() {
        Some(ByteOrder::BigEndian) => little_endian.into_iter().rev().collect(),
        Some(ByteOrder::LittleEndian) | None => little_endian,
    }
}

/// Whether a payload width is one the declared width admits.
const fn admits_width(payload: PayloadWidth, width: usize) -> bool {
    match payload {
        PayloadWidth::Absent => width == 0,
        PayloadWidth::Exact(exact) => width == exact.get(),
        PayloadWidth::Bounded { minimum, maximum } => minimum <= width && width <= maximum.get(),
    }
}

/// The minimal little-endian script-number encoding of `value`.
///
/// Sign and magnitude rather than two's complement: the magnitude is
/// written least significant byte first, and the sign occupies the top
/// bit of the last byte, which is why a magnitude whose own top bit is
/// set needs one further byte to carry the sign.
fn script_number_bytes(value: i64) -> Vec<u8> {
    if value == 0 {
        return Vec::new();
    }

    let negative = value < 0;
    let mut magnitude = value.unsigned_abs();
    let mut bytes = Vec::new();
    while magnitude > 0 {
        let low = u8::try_from(magnitude & 0xff).unwrap_or(0);
        bytes.push(low);
        magnitude >>= 8;
    }

    match bytes.last_mut() {
        Some(top) if *top & 0x80 == 0 => {
            if negative {
                *top |= 0x80;
            }
        }
        _ => bytes.push(if negative { 0x80 } else { 0x00 }),
    }
    bytes
}

/// Whether `bytes` is a minimal little-endian script number.
///
/// A trailing zero byte is never minimal, and neither is a trailing
/// sign byte whose magnitude byte below it already had room for the
/// sign.
fn is_minimal_script_number(bytes: &[u8]) -> bool {
    match bytes {
        [] => true,
        [.., last] => {
            if last & 0x7f != 0 {
                return true;
            }
            matches!(bytes, [.., second_last, _] if second_last & 0x80 != 0)
        }
    }
}
