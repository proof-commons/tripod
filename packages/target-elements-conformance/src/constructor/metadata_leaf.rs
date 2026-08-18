//! The metadata leaf's script, and why it can never be spent.
//!
//! # An empty script is not unspendable
//!
//! The metadata leaf exists to commit to metadata bytes, not to be
//! executed. That does not make it safe to leave empty: the target ends
//! evaluation by checking that exactly one true item remains, and an
//! initial witness item survives an empty script untouched. A leaf
//! carrying no script is spendable by anybody who supplies a true
//! witness item `(´[PLAN-rule:guide10:metadata-unspendable]´)`.
//!
//! So the leaf carries a script whose every execution ends in an abort,
//! whatever the witness was.
//!
//! # No unconditional-abort primitive exists to use
//!
//! The reviewed census has no primitive that simply ends evaluation.
//! What it has is Boolean verification, which aborts on a false operand
//! and is the only reviewed primitive declaring that cause. The leaf is
//! therefore built out of it: push the metadata, push the false, and
//! verify. The verification finds a literal false and ends evaluation
//! before the target ever reaches its final-stack rule, so nothing the
//! witness contained can matter.
//!
//! # The bytes come from a typed program
//!
//! The script is assembled by the reviewed program builder rather than
//! written out here, so its encoding is the contract's and not this
//! module's opinion of it. That is the one place the oracle deliberately
//! does depend on the typed builder, and the reason is the opposite of
//! the usual one: these bytes are an *input* to the construction rather
//! than an expectation about it, and a hand-assembled input would be
//! testing the oracle against bytes no target ever sees.

use tapscript::instruction::{StackItem, TapscriptInstruction};
use tapscript::program::TapscriptProgram;
use target_elements::{OpcodeId, ReviewedElementsTapscriptDefinition};

use crate::error::NativeConformanceError;

/// The metadata leaf's script, for one metadata encoding.
///
/// # Errors
///
/// [`NativeConformanceError::FixtureNotExpressible`] when the metadata
/// is wider than the target admits as one literal push.
pub fn metadata_leaf_script(
    target: &ReviewedElementsTapscriptDefinition,
    metadata: &[u8],
) -> Result<Vec<u8>, NativeConformanceError> {
    Ok(metadata_leaf_program(target, metadata)?.encode(target))
}

/// The metadata leaf's typed program.
///
/// # Errors
///
/// [`NativeConformanceError::FixtureNotExpressible`] when the metadata
/// is wider than the target admits as one literal push.
pub fn metadata_leaf_program(
    target: &ReviewedElementsTapscriptDefinition,
    metadata: &[u8],
) -> Result<TapscriptProgram, NativeConformanceError> {
    let committed = StackItem::new(target, metadata.to_vec())
        .map_err(|_| NativeConformanceError::FixtureNotExpressible)?;

    TapscriptProgram::new(vec![
        // The commitment. It is never read: what commits to the
        // metadata is the leaf hash covering these bytes, and putting
        // them on the stack is how they get into the script at all.
        TapscriptInstruction::Push(committed),
        // The literal false, and the verification that ends evaluation
        // on it. Nothing after this instruction can execute, and
        // nothing before it can consume the false.
        TapscriptInstruction::Push(StackItem::empty()),
        TapscriptInstruction::Opcode(OpcodeId::Verify),
    ])
    .map_err(|_| NativeConformanceError::FixtureNotExpressible)
}
