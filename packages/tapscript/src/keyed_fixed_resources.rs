//! Derived fixed-form resources, never native measurements or policy admission.

use crate::keyed_fixed_constructor::KeyedFixedLeafRecipe;
use crate::keyed_resources::KeyedResourceOverflow;
use crate::{AbstractLimits, AbstractStackState, TapscriptError, program_stack_profile};
use target_elements::ReviewedElementsTapscriptDefinition;

/// Exact schedule counts and qualified-domain width, without a transaction cap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedFixedResources {
    pub script_bytes: u64,
    pub instructions: u64,
    pub primitives: u64,
    pub peak_stack: u64,
    /// 40-byte SHA-256 state header plus the 33 buffered asset/prefix bytes.
    pub maximum_element_bytes: u64,
    pub script_witness_bytes: u64,
    pub input_witness_bytes: u64,
    /// Witness bytes plus the target's validation-budget offset, not CPU time.
    pub validation_budget: u64,
}

/// Project the emitted schedule using the existing abstract stack machinery.
/// # Errors
/// Returns the reviewed literal or program construction refusal.
pub fn resources(
    target: &ReviewedElementsTapscriptDefinition,
    recipe: KeyedFixedLeafRecipe,
) -> Result<KeyedFixedResources, TapscriptError> {
    let _span =
        tracing::debug_span!(target: "tapscript::keyed_fixed_resources", "resources").entered();
    let material = recipe.construct(target)?;
    let program = material.program();
    let stack = program_stack_profile(
        target,
        program,
        &AbstractStackState::from_main(vec![]),
        AbstractLimits::for_target(target),
    );
    let script_bytes = program.encoded_length(target);
    let control_bytes =
        1 + crate::STATE_NUMS_KEY.len() as u64 + 32 * material.siblings().len() as u64;
    // This fixed schedule and depth-zero control each fit one CompactSize byte.
    // The script stack has two items; the other three witness vectors are empty.
    let script_witness_bytes = 1 + 1 + script_bytes + 1 + control_bytes;
    let input_witness_bytes = script_witness_bytes + 3;
    let asset_message_bytes = recipe.parameters().asset.len() as u64 + 1;
    // Serialized SHA-256 context: state32, byte-count8, buffered message modulo64.
    let maximum_element_bytes = 32 + 8 + asset_message_bytes % 64;
    let result = KeyedFixedResources {
        script_bytes,
        instructions: program.instructions().len() as u64,
        primitives: program
            .instructions()
            .iter()
            .filter(|i| matches!(i, crate::TapscriptInstruction::Opcode(_)))
            .count() as u64,
        peak_stack: stack.peak_main() + stack.peak_alternate(),
        maximum_element_bytes,
        script_witness_bytes,
        input_witness_bytes,
        validation_budget: script_witness_bytes
            + target
                .definition()
                .resources()
                .consensus()
                .validation_budget_offset(),
    };
    tracing::debug!(target: "tapscript::keyed_fixed_resources", script_bytes = result.script_bytes, peak_stack = result.peak_stack, "derived fixed-form resources");
    Ok(result)
}

/// Complete stripped bytes and other witnesses must be supplied exactly once.
/// This predicts weight; it supplies no admitted member ceiling.
/// # Errors
/// Returns overflow rather than wrapping any size.
pub fn transaction_weight(
    target: &ReviewedElementsTapscriptDefinition,
    members: u64,
    base_bytes: u64,
    other_witness_bytes: u64,
) -> Result<u64, KeyedResourceOverflow> {
    let _span = tracing::debug_span!(target: "tapscript::keyed_fixed_resources", "transaction_weight", members).entered();
    let result = base_bytes
        .checked_mul(
            target
                .definition()
                .resources()
                .consensus()
                .witness_scale_factor(),
        )
        .and_then(|base| base.checked_add(other_witness_bytes))
        .and_then(|other| {
            members
                .checked_mul(143)
                .and_then(|witness| other.checked_add(witness))
        })
        .ok_or(KeyedResourceOverflow);
    tracing::debug!(target: "tapscript::keyed_fixed_resources", outcome = if result.is_ok() { "completed" } else { "refused" }, "fixed-form weight derived");
    result
}
