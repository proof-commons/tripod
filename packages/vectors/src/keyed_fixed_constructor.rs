//! Offline fixed-form commitment and complete public reconstruction.
//! No native observation, target amount admission or transaction construction.

use realization::keyed_fixed::KeyedFixedParameters;
use tapscript::keyed_constructor::{KeyedCurveCapability, KeyedTweakOutcome};
use tapscript::keyed_fixed_constructor::KeyedFixedLeafRecipe;
use target_elements::ReviewedElementsTapscriptDefinition;
use target_elements_conformance::constructor::curve::generator;
use target_elements_conformance::constructor::tagged::sha256;
use target_elements_conformance::constructor::{
    UNSPENDABLE_INTERNAL_KEY, control_block, leaf_hash, output_program, tweak,
};

pub use crate::keyed_constructor::PublicKeyedCurve;

/// Mutable offer. Only reconstruction establishes agreement with this recipe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedFixedDescriptor {
    pub parameters: KeyedFixedParameters,
    pub transcript: [u8; 42],
    pub pair_digest: [u8; 32],
    pub leaf: Vec<u8>,
    pub leaf_version: u8,
    pub internal_key: [u8; 32],
    pub root: [u8; 32],
    pub tweak: [u8; 32],
    pub output_key: [u8; 32],
    pub parity: bool,
    pub output_program: Vec<u8>,
    pub control: Vec<u8>,
}

/// Construction and authentication boundaries, never node outcomes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyedFixedConstructorRefusal {
    LeafConstruction,
    InternalKeyPolicy,
    InternalKeyNotAPoint,
    TweakAboveGroupOrder,
    TweakedPointIsIdentity,
    DescriptorMismatch,
    PredecessorCommitment,
}

/// Reconstruct the recipe using existing public arithmetic, with no key search.
/// # Errors
/// Returns exact leaf, NUMS or arithmetic refusal; never changes the triple.
pub fn construct(
    target: &ReviewedElementsTapscriptDefinition,
    parameters: KeyedFixedParameters,
    curve: &impl KeyedCurveCapability,
) -> Result<KeyedFixedDescriptor, KeyedFixedConstructorRefusal> {
    let _span =
        tracing::debug_span!(target: "vectors::keyed_fixed_constructor", "construct").entered();
    let result = construct_core(target, parameters, curve);
    tracing::debug!(target: "vectors::keyed_fixed_constructor", outcome = if result.is_ok() { "completed" } else { "refused" }, refusal = ?result.as_ref().err(), "fixed-form commitment constructed");
    result
}

fn construct_core(
    target: &ReviewedElementsTapscriptDefinition,
    parameters: KeyedFixedParameters,
    curve: &impl KeyedCurveCapability,
) -> Result<KeyedFixedDescriptor, KeyedFixedConstructorRefusal> {
    use KeyedFixedConstructorRefusal as E;
    let recipe = KeyedFixedLeafRecipe::new(parameters);
    let material = recipe.construct(target).map_err(|_| E::LeafConstruction)?;
    let internal_key = sha256(&generator().uncompressed_bytes());
    if internal_key != UNSPENDABLE_INTERNAL_KEY {
        return Err(E::InternalKeyPolicy);
    }
    if !curve.internal_key_is_a_point(&internal_key) {
        return Err(E::InternalKeyNotAPoint);
    }
    let leaf = material.program().encode(target);
    let version = material.leaf_version();
    let root = leaf_hash(version, &leaf);
    let tweak = tweak(&internal_key, &root);
    let (output_key, parity) = match curve.output_key(&internal_key, &root) {
        KeyedTweakOutcome::OutputKey { key, parity } => (key, parity),
        KeyedTweakOutcome::InternalKeyNotAPoint => return Err(E::InternalKeyNotAPoint),
        KeyedTweakOutcome::TweakAboveGroupOrder => return Err(E::TweakAboveGroupOrder),
        KeyedTweakOutcome::TweakedPointIsIdentity => return Err(E::TweakedPointIsIdentity),
    };
    Ok(KeyedFixedDescriptor {
        parameters,
        transcript: recipe.transcript(),
        pair_digest: recipe.pair_digest(),
        leaf,
        leaf_version: version.get(),
        internal_key,
        root,
        tweak,
        output_key,
        parity,
        output_program: output_program(&output_key),
        control: control_block(version, u8::from(parity), &internal_key, &[]),
    })
}

/// Recompute every field from the supplied triple, then compare the whole offer.
/// # Errors
/// Returns a construction refusal or a complete-descriptor mismatch.
pub fn reconstruct(
    target: &ReviewedElementsTapscriptDefinition,
    offered: &KeyedFixedDescriptor,
    curve: &impl KeyedCurveCapability,
) -> Result<KeyedFixedDescriptor, KeyedFixedConstructorRefusal> {
    let _span =
        tracing::debug_span!(target: "vectors::keyed_fixed_constructor", "reconstruct").entered();
    let expected = construct(target, offered.parameters, curve)?;
    if expected != *offered {
        return Err(KeyedFixedConstructorRefusal::DescriptorMismatch);
    }
    Ok(expected)
}

/// Compares supplied program bytes with the reconstructed program for the triple.
///
/// This authenticates no parent. The caller must authenticate the predecessor and
/// its fields independently; equality here only checks its supplied program bytes.
/// # Errors
/// Refuses descriptor drift or a different predecessor commitment.
pub fn authenticate_predecessor(
    target: &ReviewedElementsTapscriptDefinition,
    offered: &KeyedFixedDescriptor,
    predecessor: &[u8],
    curve: &impl KeyedCurveCapability,
) -> Result<(), KeyedFixedConstructorRefusal> {
    let expected = reconstruct(target, offered, curve)?;
    if expected.output_program != predecessor {
        return Err(KeyedFixedConstructorRefusal::PredecessorCommitment);
    }
    Ok(())
}
