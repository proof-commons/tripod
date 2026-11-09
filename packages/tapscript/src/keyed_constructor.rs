//! Provisional keyed leaf and supplied public curve capability.
//!
//! The semantic key is opaque public deployment data, not a signing key or a
//! recursive output-key literal. Its push/drop adds no spending predicate.
//! Abstract validation cannot establish a concrete spend or key-path closure.
//! This default-off API does not select a production constructor home.

use target_elements::{OpcodeId, ReviewedElementsTapscriptDefinition};

use crate::keyed_pattern::{born_first_fragment, continue_first_fragment};
use crate::{StackItem, TapscriptError, TapscriptInstruction, TapscriptProgram};

/// Opaque fixed-width public semantic identity. Every 32-byte value is admitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyedSemanticKey([u8; 32]);

impl KeyedSemanticKey {
    /// Accept deployment data without a signing-key or nonzero requirement.
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// The exact public bytes committed by the leaf.
    #[must_use]
    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Caller-supplied public arithmetic, independent of STATE metadata semantics.
pub trait KeyedCurveCapability {
    /// Check liftability only; this says nothing about scalar knowledge.
    fn internal_key_is_a_point(&self, key: &[u8; 32]) -> bool;
    /// Compute the Elements tagged tweak and point sum; zero tweak is valid.
    fn output_key(&self, internal_key: &[u8; 32], root: &[u8; 32]) -> KeyedTweakOutcome;
}

/// Closed outcomes of the supplied public tweak operation; no nonce grinding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyedTweakOutcome {
    /// A finite output point and its y oddness.
    OutputKey {
        /// X-only output key.
        key: [u8; 32],
        /// Whether the output point has odd y.
        parity: bool,
    },
    /// Hash cannot be interpreted as a scalar without reduction.
    TweakAboveGroupOrder,
    /// The point sum is infinity.
    TweakedPointIsIdentity,
    /// The internal point cannot be lifted.
    InternalKeyNotAPoint,
}

/// Commit the key, then enforce birth and continuation, then leave final true.
/// # Errors
/// Returns the reviewed literal or program construction refusal.
pub fn keyed_leaf_program(
    target: &ReviewedElementsTapscriptDefinition,
    key: KeyedSemanticKey,
) -> Result<TapscriptProgram, TapscriptError> {
    let _span = tracing::debug_span!("keyed_leaf_program").entered();
    let result = (|| {
        let mut instructions = vec![
            TapscriptInstruction::Push(StackItem::new(target, key.bytes().to_vec())?),
            TapscriptInstruction::Opcode(OpcodeId::Drop),
        ];
        instructions.extend_from_slice(born_first_fragment(target)?.instructions());
        instructions.extend_from_slice(continue_first_fragment(target)?.instructions());
        instructions.push(TapscriptInstruction::Push(StackItem::script_number(
            target, 1,
        )?));
        TapscriptProgram::new(instructions)
    })();
    tracing::debug!(
        completed = result.is_ok(),
        "keyed leaf construction finished"
    );
    result
}
