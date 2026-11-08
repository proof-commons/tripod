//! Born-first schedule for the keyed provenance experiment.
//!
//! The executing input selects its authenticated outpoint. Flags are discarded;
//! the spent output index is compared as four bytes, never as a script number.
//! The fragment consumes no initial items and leaves no successful residue.
//! Abstract stack validation cannot decide a concrete birth outcome: the native
//! A0–A3 subjects must establish that separately. A standalone experiment appends
//! final truth; this fragment itself neither pushes truth nor checks continuation.

use target_elements::{EncodingClass, OpcodeId, ReviewedElementsTapscriptDefinition};

use crate::{StackItem, TapscriptError, TapscriptInstruction, TapscriptProgram};

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
