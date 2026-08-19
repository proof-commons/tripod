//! Typed target programs, their exact serialization, and the parser
//! that is their validation boundary.
//!
//! # What construction actually establishes
//!
//! [`TapscriptProgram`] validates its work limit and nothing else, and
//! the absence of the other checks is deliberate rather than an
//! omission. A program is built from typed instructions against the
//! *reviewed* contract, so:
//!
//! - the execution domain and the contract revision are fixed by the
//!   reviewed wrapper, which has no public constructor and admits only
//!   a contract typed-equal to the one the target package derives;
//! - every [`OpcodeId`] has a contract gated to that domain, because
//!   the target validator refuses a definition where one does not;
//! - every [`StackItem`] is within the literal bound, because no
//!   constructor produces one that is not;
//! - every item therefore has a minimal push form, because the
//!   reviewed rule names a form for every payload within that bound.
//!
//! Re-testing any of those would add a branch no input can reach, and
//! an unreachable branch in a validator reads as a check that is
//! running when it is not.
//!
//! # The parser is not a script reader
//!
//! [`TapscriptProgram::decode`] accepts exactly the reviewed subset:
//! reviewed primitive bytes and reviewed push forms in their minimal
//! encoding. An unknown byte, a nonminimal push, an oversized literal,
//! and a push that runs off the end all fail with their own typed
//! reason. It is a boundary for correlating first-party programs with
//! target-native results, not a general Elements script parser, and
//! widening it would quietly make it one.
//!
//! # Refusing more than the target does
//!
//! The target itself refuses a nonminimal push only under the
//! standardness rules a node applies to what it relays. The parser here
//! refuses it always. That is stricter than consensus on purpose: a
//! first-party program that no node forwards is of no use, and the
//! round-trip property below only holds for the minimal form.
//!
//! # The work bound binds the parse, not the result
//!
//! [`MAXIMUM_PROGRAM_INSTRUCTIONS`] is checked inside the parse loop, so
//! the work and the allocation a script can cause are bounded by the
//! limit rather than by the script's own length. Checking it only on the
//! finished sequence would make the limit a property of the answer
//! instead of a property of the parse, and an oversized script would pay
//! for itself in full before being refused.

use std::collections::BTreeMap;

use target_elements::{ByteOrder, OpcodeId, PushFormSpec, ReviewedElementsTapscriptDefinition};

use crate::error::TapscriptError;
use crate::instruction::{StackItem, TapscriptInstruction};

/// The greatest number of instructions one program may carry.
///
/// A first-party bound, not a target one: the reviewed execution domain
/// enforces no script-size limit at all, so a program that ran away
/// would be refused by nothing until it reached a node. The abstract
/// validator's cost is linear in this figure, which is why it is
/// stated here rather than left implicit.
pub const MAXIMUM_PROGRAM_INSTRUCTIONS: u64 = 10_000;

/// A validated sequence of typed target instructions.
///
/// Fields are private: a program is what its constructor accepted, and
/// there is no literal form that would let a caller assemble one
/// without the work-limit check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TapscriptProgram {
    instructions: Vec<TapscriptInstruction>,
}

impl TapscriptProgram {
    /// Validates a sequence of typed instructions.
    ///
    /// # Errors
    ///
    /// [`TapscriptError::InstructionLimitExceeded`] when the sequence
    /// is longer than [`MAXIMUM_PROGRAM_INSTRUCTIONS`].
    pub fn new(instructions: Vec<TapscriptInstruction>) -> Result<Self, TapscriptError> {
        // No target parameter: every fact construction is asked to
        // establish is already established by the types of the
        // instructions, and the one reachable check is a first-party
        // work limit rather than a target fact. A parameter the
        // function could not read would suggest a check it does not
        // make.
        let offered = u64::try_from(instructions.len()).unwrap_or(u64::MAX);
        if offered > MAXIMUM_PROGRAM_INSTRUCTIONS {
            return Err(TapscriptError::InstructionLimitExceeded {
                maximum: MAXIMUM_PROGRAM_INSTRUCTIONS,
            });
        }
        Ok(Self { instructions })
    }

    /// The typed instructions, in program order.
    #[must_use]
    pub fn instructions(&self) -> &[TapscriptInstruction] {
        &self.instructions
    }

    /// How many instructions the program carries.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.instructions.len()
    }

    /// Whether the program carries no instructions.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    /// The program's exact target bytes.
    ///
    /// Every opcode byte and every push form is resolved from the
    /// reviewed contract, so no target number is restated here. The
    /// result is deterministic: it depends on the instruction sequence
    /// and on nothing else — not on map iteration, declaration order,
    /// host, thread count, or environment.
    #[must_use]
    pub fn encode(&self, target: &ReviewedElementsTapscriptDefinition) -> Vec<u8> {
        let mut bytes = Vec::new();
        for instruction in &self.instructions {
            encode_instruction(target, instruction, &mut bytes);
        }
        bytes
    }

    /// How many target bytes the program's exact encoding occupies.
    ///
    /// # The same encoding, counted rather than kept
    ///
    /// This is [`Self::encode`] measured, not a second account of what an
    /// encoding costs. One instruction is laid out at a time into a
    /// buffer that is reused, so the figure is the encoding's own by
    /// construction and cannot drift from it, and the program is never
    /// assembled in full to be measured.
    ///
    /// Saturating: the figure is diagnostic, and a saturated one is
    /// visibly pinned where a wrapped one would read as a small honest
    /// program.
    #[must_use]
    pub fn encoded_length(&self, target: &ReviewedElementsTapscriptDefinition) -> u64 {
        let mut instruction_bytes = Vec::new();
        let mut total = 0_u64;
        for instruction in &self.instructions {
            instruction_bytes.clear();
            encode_instruction(target, instruction, &mut instruction_bytes);
            total =
                total.saturating_add(u64::try_from(instruction_bytes.len()).unwrap_or(u64::MAX));
        }
        total
    }

    /// Parses exactly the reviewed subset of target script.
    ///
    /// # Errors
    ///
    /// [`TapscriptError::UnknownOpcodeByte`] for a byte that is neither
    /// a reviewed primitive nor a push form,
    /// [`TapscriptError::TruncatedInstruction`] when a push runs off
    /// the end of the script, [`TapscriptError::OversizedStackItem`]
    /// when a push states a width above the target's literal bound,
    /// [`TapscriptError::NonMinimalPush`] when a payload is carried in
    /// a form that is not its minimal one, and
    /// [`TapscriptError::InstructionLimitExceeded`] when the script
    /// holds more instructions than a program may carry.
    pub fn decode(
        target: &ReviewedElementsTapscriptDefinition,
        bytes: &[u8],
    ) -> Result<Self, TapscriptError> {
        let primitives = primitive_bytes(target);
        let pushes = target.definition().pushes();
        let mut instructions = Vec::new();
        let mut offset = 0;

        while offset < bytes.len() {
            // The work bound is enforced here rather than by the
            // constructor at the end. A script arrives from an untrusted
            // source, and a parser that read all of it before applying
            // its own limit would let the input decide how much work and
            // how much allocation the limit was supposed to bound. There
            // are still bytes left and the program is already full, so
            // the answer cannot change: it is the limit, whatever those
            // bytes turn out to be.
            if u64::try_from(instructions.len()).unwrap_or(u64::MAX) >= MAXIMUM_PROGRAM_INSTRUCTIONS
            {
                return Err(TapscriptError::InstructionLimitExceeded {
                    maximum: MAXIMUM_PROGRAM_INSTRUCTIONS,
                });
            }

            let opcode = bytes[offset];
            offset += 1;

            let Some(spec) = pushes.form_for_opcode(opcode) else {
                let id = primitives
                    .get(&opcode)
                    .ok_or(TapscriptError::UnknownOpcodeByte(opcode))?;
                instructions.push(TapscriptInstruction::Opcode(*id));
                continue;
            };

            // The payload either travels inside the opcode, or its
            // width does, or a width prefix states it. The three cases
            // come from the contract, so the parser learns them rather
            // than deciding them.
            let payload = if let Some(literal) = spec.payload_for(opcode) {
                literal
            } else {
                let width = match spec.width_in_opcode(opcode) {
                    Some(width) => width,
                    None => read_width(spec, bytes, &mut offset)?,
                };
                let maximum = pushes.maximum_payload_bytes();
                let end = offset
                    .checked_add(width)
                    .ok_or(TapscriptError::TruncatedInstruction)?;
                // Truncation is decided before the literal bound, as
                // the target decides it: a script that ends mid-push is
                // malformed whatever width it claimed.
                if end > bytes.len() {
                    return Err(TapscriptError::TruncatedInstruction);
                }
                if width > maximum {
                    return Err(TapscriptError::OversizedStackItem {
                        offered: width,
                        maximum,
                    });
                }
                let payload = bytes[offset..end].to_vec();
                offset = end;
                payload
            };

            // The payload's width is already within the literal bound,
            // so the rule can only answer with a form, and the question
            // left is whether the script used that form.
            let minimal = pushes
                .minimal_form(&payload)
                .map_err(|_| TapscriptError::NonMinimalPush)?;
            if minimal != spec.form() {
                return Err(TapscriptError::NonMinimalPush);
            }

            instructions.push(TapscriptInstruction::Push(StackItem::new(target, payload)?));
        }

        Self::new(instructions)
    }
}

/// The reviewed primitive each target byte names.
fn primitive_bytes(target: &ReviewedElementsTapscriptDefinition) -> BTreeMap<u8, OpcodeId> {
    target
        .definition()
        .opcodes()
        .values()
        .map(|spec| (spec.code(), spec.id()))
        .collect()
}

/// The target byte of one reviewed primitive.
///
/// Total by construction: the target validator refuses a definition
/// that omits a contract for any member of the reviewed census, so the
/// reviewed contract states a byte for every identity.
fn opcode_byte(target: &ReviewedElementsTapscriptDefinition, id: OpcodeId) -> u8 {
    target
        .definition()
        .opcodes()
        .get(&id)
        .expect("the reviewed contract states a byte for every primitive")
        .code()
}

/// Appends the exact encoding of one typed instruction.
///
/// The single place the layout of an instruction is decided, so the
/// bytes a program writes and the bytes it is measured at are the same
/// bytes.
fn encode_instruction(
    target: &ReviewedElementsTapscriptDefinition,
    instruction: &TapscriptInstruction,
    bytes: &mut Vec<u8>,
) {
    match instruction {
        TapscriptInstruction::Opcode(id) => bytes.push(opcode_byte(target, *id)),
        TapscriptInstruction::Push(item) => encode_push(target, item, bytes),
    }
}

/// Appends the minimal encoding of one literal.
///
/// Total by construction: every stack item is within the literal bound,
/// and the reviewed rule names a minimal form for every payload within
/// it, so neither resolution can fail.
fn encode_push(
    target: &ReviewedElementsTapscriptDefinition,
    item: &StackItem,
    bytes: &mut Vec<u8>,
) {
    let pushes = target.definition().pushes();
    let form = pushes
        .minimal_form(item.bytes())
        .expect("every admissible literal has a minimal form");
    let spec = pushes
        .form(form)
        .expect("the reviewed contract states every form the rule names");
    let opcode = spec
        .opcode_for(item.bytes())
        .expect("the minimal form carries the payload it was chosen for");

    bytes.push(opcode);
    // A form that carries its payload inside the opcode is complete
    // already. Any other form states a width prefix of its declared
    // size — zero bytes for a form whose opcode is the width — and then
    // the payload itself.
    if spec.payload_for(opcode).is_some() {
        return;
    }
    if let Some(byte_order) = spec.width_prefix_order() {
        bytes.extend_from_slice(&width_bytes(
            item.len(),
            spec.width_prefix_bytes(),
            byte_order,
        ));
    }
    bytes.extend_from_slice(item.bytes());
}

/// One payload width, laid out as a prefix of `count` bytes.
fn width_bytes(width: usize, count: usize, byte_order: ByteOrder) -> Vec<u8> {
    let mut prefix: Vec<u8> = (0..count)
        .map(|index| u8::try_from((width >> (index * 8)) & 0xff).unwrap_or(0))
        .collect();
    if byte_order == ByteOrder::BigEndian {
        prefix.reverse();
    }
    prefix
}

/// Reads one width prefix, advancing `offset` past it.
fn read_width(
    spec: &PushFormSpec,
    bytes: &[u8],
    offset: &mut usize,
) -> Result<usize, TapscriptError> {
    let count = spec.width_prefix_bytes();
    let byte_order = spec.width_prefix_order().unwrap_or(ByteOrder::LittleEndian);

    let end = offset
        .checked_add(count)
        .ok_or(TapscriptError::TruncatedInstruction)?;
    if end > bytes.len() {
        return Err(TapscriptError::TruncatedInstruction);
    }

    let mut digits = bytes[*offset..end].to_vec();
    if byte_order == ByteOrder::BigEndian {
        digits.reverse();
    }
    *offset = end;

    let mut width = 0_usize;
    for (index, digit) in digits.into_iter().enumerate() {
        width |= usize::from(digit) << (index * 8);
    }
    Ok(width)
}
