//! Public triple recipe; the target amount ceiling belongs to S6 construction.

use realization::keyed_fixed::KeyedFixedParameters;
use sha2::{Digest, Sha256};
use target_elements::{LeafVersion, ReviewedElementsTapscriptDefinition};

use crate::{TapscriptError, TapscriptProgram};

/// Fixed-form recipe with a positive supplied value and no deployment default.
///
/// The sole depth-zero leaf uses the NUMS derivation and Elements tags specified
/// in docs/lineage/realization.md. No caller selects a tree or internal key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyedFixedLeafRecipe {
    parameters: KeyedFixedParameters,
}

impl KeyedFixedLeafRecipe {
    /// Retain public data; this does not admit a transaction amount.
    #[must_use]
    pub const fn new(parameters: KeyedFixedParameters) -> Self {
        Self { parameters }
    }

    /// Exact supplied triple, including the opaque key (zero is allowed).
    #[must_use]
    pub const fn parameters(&self) -> KeyedFixedParameters {
        self.parameters
    }

    /// A || explicit-asset prefix || LE64(v) || explicit-value prefix.
    #[must_use]
    pub fn transcript(&self) -> [u8; 42] {
        let mut bytes = [0; 42];
        bytes[..32].copy_from_slice(&self.parameters.asset);
        bytes[32] = 1;
        bytes[33..41].copy_from_slice(&self.parameters.value.get().to_le_bytes());
        bytes[41] = 1;
        bytes
    }

    /// SHA-256 of the complete prefix-qualified transcript.
    #[must_use]
    pub fn pair_digest(&self) -> [u8; 32] {
        Sha256::digest(self.transcript()).into()
    }

    /// Emit the literal 36-instruction schedule, without witness arguments.
    /// # Errors
    /// Returns the reviewed literal or program construction refusal.
    pub fn construct(
        &self,
        target: &ReviewedElementsTapscriptDefinition,
    ) -> Result<KeyedFixedConstructionMaterial, TapscriptError> {
        let _span = tracing::debug_span!(target: "tapscript::keyed_fixed_constructor", "construct")
            .entered();
        let result = crate::keyed_fixed_pattern::compose(target, *self).map(|program| {
            KeyedFixedConstructionMaterial {
                recipe: *self,
                version: target.definition().leaf_version(),
                program,
            }
        });
        tracing::debug!(target: "tapscript::keyed_fixed_constructor", outcome = if result.is_ok() { "completed" } else { "refused" }, instruction_count = 36_u64, "fixed-form material constructed");
        result
    }
}

/// Sealed emitted material; commitment and parent authentication are downstream.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedFixedConstructionMaterial {
    recipe: KeyedFixedLeafRecipe,
    version: LeafVersion,
    program: TapscriptProgram,
}

impl KeyedFixedConstructionMaterial {
    /// Public triple recipe retained without substitution.
    #[must_use]
    pub const fn recipe(&self) -> &KeyedFixedLeafRecipe {
        &self.recipe
    }
    /// Reviewed leaf version, distinct from the streaming opcode byte.
    #[must_use]
    pub const fn leaf_version(&self) -> LeafVersion {
        self.version
    }
    /// Exact instruction sequence.
    #[must_use]
    pub const fn program(&self) -> &TapscriptProgram {
        &self.program
    }
    /// The sole leaf has no siblings.
    #[must_use]
    pub const fn siblings(&self) -> &[[u8; 32]] {
        &[]
    }
}
