//! Born-first and continue-first schedules for the keyed provenance experiment.
//!
//! The executing input selects its authenticated outpoint. Flags are discarded;
//! the spent output index is compared as four bytes, never as a script number.
//! The fragment consumes no initial items and leaves no successful residue.
//! Abstract stack validation cannot decide a concrete birth outcome: the native
//! A0–A3 subjects must establish that separately. A standalone experiment appends
//! final truth; this fragment itself neither pushes truth nor checks continuation.
//! The continuation fragment compares the authenticated current program against
//! output zero, version first and payload second, following the comparison
//! schedule in `pattern.rs`. Abstract validation cannot decide a concrete
//! continuation outcome; native control/negative subjects must establish it.

use target_elements::{EncodingClass, OpcodeId, ReviewedElementsTapscriptDefinition};

use crate::{StackItem, TapscriptError, TapscriptInstruction, TapscriptProgram};

// Shared composition for the provisional constructor and the plan adapter.
// Keep the instruction order and bytes identical to the admitted recipe.
pub(crate) fn compose_keyed_leaf(
    target: &ReviewedElementsTapscriptDefinition,
    key: &[u8; 32],
) -> Result<TapscriptProgram, TapscriptError> {
    let mut instructions = vec![
        TapscriptInstruction::Push(StackItem::new(target, key.to_vec())?),
        TapscriptInstruction::Opcode(OpcodeId::Drop),
    ];
    instructions.extend_from_slice(born_first_fragment(target)?.instructions());
    instructions.extend_from_slice(continue_first_fragment(target)?.instructions());
    instructions.push(TapscriptInstruction::Push(StackItem::script_number(
        target, 1,
    )?));
    TapscriptProgram::new(instructions)
}

/// Require that the output spent by the executing input was born at position zero.
///
/// There is no current-input-zero or zero-flags condition.
///
/// # Errors
/// Returns the checked literal or program construction refusal if the reviewed
/// encoding or program bounds do not admit this schedule.
pub fn born_first_fragment(
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<TapscriptProgram, TapscriptError> {
    let span = tracing::debug_span!(target: "tapscript::keyed_pattern", "born_first_fragment");
    let _entered = span.enter();
    let result =
        StackItem::encoded(target, EncodingClass::OutPointIndex, vec![0; 4]).and_then(|zero| {
            TapscriptProgram::new(vec![
                TapscriptInstruction::Opcode(OpcodeId::PushCurrentInputIndex),
                TapscriptInstruction::Opcode(OpcodeId::InspectInputOutpoint),
                TapscriptInstruction::Opcode(OpcodeId::Drop),
                TapscriptInstruction::Push(zero),
                TapscriptInstruction::Opcode(OpcodeId::EqualVerify),
                TapscriptInstruction::Opcode(OpcodeId::Drop),
            ])
        });
    match &result {
        Ok(_) => tracing::debug!(
            target: "tapscript::keyed_pattern",
            outcome = "completed",
            instruction_count = 6_u64,
            "birth fragment constructed"
        ),
        Err(error) => tracing::debug!(
            target: "tapscript::keyed_pattern",
            outcome = "refused",
            refusal_kind = error.refusal_kind(),
            "birth fragment refused"
        ),
    }
    result
}

/// Require output zero to have the executing input's version and program bytes.
///
/// This equality-only schedule consumes no initial items and leaves no residue.
/// It imposes no asset, value, amount or current-input-position condition.
///
/// # Errors
/// Returns the checked numeric literal or program construction refusal when the
/// reviewed encoding or program bounds do not admit this schedule.
pub fn continue_first_fragment(
    target: &ReviewedElementsTapscriptDefinition,
) -> Result<TapscriptProgram, TapscriptError> {
    let span = tracing::debug_span!(target: "tapscript::keyed_pattern", "continue_first_fragment");
    let _entered = span.enter();
    let result = StackItem::script_number(target, 0).and_then(|zero| {
        TapscriptProgram::new(vec![
            TapscriptInstruction::Opcode(OpcodeId::PushCurrentInputIndex),
            TapscriptInstruction::Opcode(OpcodeId::InspectInputScriptPubKey),
            TapscriptInstruction::Push(zero),
            TapscriptInstruction::Opcode(OpcodeId::InspectOutputScriptPubKey),
            TapscriptInstruction::Opcode(OpcodeId::Rotate),
            TapscriptInstruction::Opcode(OpcodeId::EqualVerify),
            TapscriptInstruction::Opcode(OpcodeId::EqualVerify),
        ])
    });
    match &result {
        Ok(_) => tracing::debug!(target: "tapscript::keyed_pattern", outcome = "completed",
            instruction_count = 7_u64, "continuation fragment constructed"),
        Err(error) => tracing::debug!(target: "tapscript::keyed_pattern", outcome = "refused",
            refusal_kind = error.refusal_kind(), "continuation fragment refused"),
    }
    result
}
