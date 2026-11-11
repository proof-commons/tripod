//! Fixed public keyed recipe, construction material and supplied curve capability.
//!
//! The semantic key is opaque public deployment data, not a signing key or a
//! recursive output-key literal. Its push/drop adds no spending predicate.
//! Abstract validation cannot establish a concrete spend or key-path closure.
//! Transaction commitment and materialization remain downstream. No hashing or
//! curve implementation is introduced here.

use target_elements::{LeafVersion, ReviewedElementsTapscriptDefinition};

use crate::{TapscriptError, TapscriptProgram};

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

/// Schema-1 keyed construction policy, parameterized only by public semantic bytes.
///
/// The leaf is minimal push32(key), DROP, born-first, continue-first, TRUE,
/// at the reviewed target's leaf version. The internal x coordinate is SHA256
/// of the 65-byte SEC1 uncompressed secp256k1 generator (04 || x || y), lifted
/// with even y. The complete tree is this sole leaf, with no sibling or branch.
/// Its root is tagged SHA256("TapLeaf/elements", version || CompactSize(script
/// length) || script). The tweak is tagged SHA256("TapTweak/elements", internal
/// x || root). Its big-endian 256-bit integer t must be below the group order n;
/// Q = P + t*G and the parity bit is Q's y oddness. No scalar reduction; zero is valid, overflow
/// and infinity refuse. The control is (version | output parity) || internal x,
/// with no siblings; the output is witness v1 with the tweaked x-only 32 bytes.
/// No caller chooses another version, tree, domain, internal key or control.
///
/// Schema 1 names these typed rules, not a serialized descriptor format.
/// [`KeyedSemanticKey`] alone is data; this type additionally commits to policy.
/// Discrete-log/NUMS and SHA256 preimage assumptions remain conditional; generated
/// key separation needs second-preimage resistance, and adversarial pairs need
/// collision resistance. Tests do not prove those assumptions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyedLeafRecipe {
    key: KeyedSemanticKey,
}

impl KeyedLeafRecipe {
    /// All 32-byte public identities are allowed, including zero.
    #[must_use]
    pub const fn new(key: [u8; 32]) -> Self {
        Self {
            key: KeyedSemanticKey::new(key),
        }
    }

    /// Exact public identity; never interpreted as a signing key.
    #[must_use]
    pub const fn key(&self) -> &[u8; 32] {
        self.key.bytes()
    }

    /// Public semantic identity retained by this policy.
    #[must_use]
    pub const fn semantic_key(&self) -> KeyedSemanticKey {
        self.key
    }

    /// Fixed typed recipe schema, independent of conformance JSON.
    #[must_use]
    pub const fn schema(&self) -> u32 {
        1
    }

    /// Fixed internal-key derivation; arithmetic is supplied downstream.
    #[must_use]
    pub const fn internal_key_derivation(&self) -> KeyedInternalKeyDerivation {
        KeyedInternalKeyDerivation::Sha256UncompressedGenerator
    }

    /// Build the sole leaf, retaining its policy and reviewed version.
    /// # Errors
    /// Returns the existing typed leaf-construction refusal unchanged.
    pub fn construct(
        &self,
        target: &ReviewedElementsTapscriptDefinition,
    ) -> Result<KeyedConstructionMaterial, TapscriptError> {
        let _span = tracing::debug_span!(target: "tapscript::keyed_constructor", "construct_keyed_material").entered();
        let result = construct_material(target, *self);
        match &result {
            Ok(_) => tracing::debug!(target: "tapscript::keyed_constructor",
                outcome = "completed", "keyed material construction finished"),
            Err(error) => tracing::debug!(target: "tapscript::keyed_constructor",
                outcome = "refused", refusal_kind = error.refusal_kind(),
                "keyed material construction finished"),
        }
        result
    }
}

impl From<KeyedSemanticKey> for KeyedLeafRecipe {
    fn from(key: KeyedSemanticKey) -> Self {
        Self { key }
    }
}

// Same-crate composition uses this core so only the outer boundary reports.
pub(crate) fn construct_material(
    target: &ReviewedElementsTapscriptDefinition,
    recipe: KeyedLeafRecipe,
) -> Result<KeyedConstructionMaterial, TapscriptError> {
    Ok(KeyedConstructionMaterial {
        recipe,
        version: target.definition().leaf_version(),
        program: keyed_leaf_program(target, recipe.semantic_key())?,
    })
}

/// Public provenance for the fixed NUMS x coordinate, not proof of unknowability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedInternalKeyDerivation {
    /// SHA256 of SEC1's 65-byte uncompressed secp256k1 generator.
    Sha256UncompressedGenerator,
}

/// Immutable sole-leaf construction material, not an admitted commitment.
///
/// Available even for SOURCE-only bundles. There is exactly one script path
/// containing both clauses and an unavoidable taproot key path, whose inability
/// to sign remains conditional on the recipe's cryptographic assumptions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedConstructionMaterial {
    recipe: KeyedLeafRecipe,
    version: LeafVersion,
    program: TapscriptProgram,
}

impl KeyedConstructionMaterial {
    /// Public recipe retained with its leaf.
    #[must_use]
    pub const fn recipe(&self) -> &KeyedLeafRecipe {
        &self.recipe
    }
    /// Reviewed target version used for the sole leaf and control.
    #[must_use]
    pub const fn leaf_version(&self) -> LeafVersion {
        self.version
    }
    /// Exact key/drop, birth, continuation and final true instructions.
    #[must_use]
    pub const fn program(&self) -> &TapscriptProgram {
        &self.program
    }
    /// Complete depth-zero path: no caller-supplied sibling is permitted.
    #[must_use]
    pub const fn siblings(&self) -> &[[u8; 32]] {
        &[]
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
pub(crate) fn keyed_leaf_program(
    target: &ReviewedElementsTapscriptDefinition,
    key: KeyedSemanticKey,
) -> Result<TapscriptProgram, TapscriptError> {
    crate::keyed_pattern::compose_keyed_leaf(target, key.bytes())
}
