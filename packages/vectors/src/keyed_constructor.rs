//! Offline, provisional keyed constructor experiment and public descriptor.
//!
//! This conformance-owned API supplies the tapscript capability from the existing
//! public constructor arithmetic. It selects no production linker/transaction
//! home and borrows no STATE metadata law. Reconstruction authenticates the
//! complete canonical recipe, never a key-shaped literal in an arbitrary leaf.
//! NUMS closure remains conditional on discrete-log hardness and SHA-256 preimage
//! resistance. Key separation additionally needs tagged SHA-256 second-preimage
//! resistance for a generated key and collision resistance for adversarial pairs.
//! Recipe one uses the minimal 32-byte data push and the existing
//! `TapLeaf/elements`, `TapBranch/elements` and `TapTweak/elements` domains.
//! The tree is a sole leaf, so no branch or sibling hash is added to its root.

use serde::{Deserialize, Serialize};
use tapscript::keyed_constructor::{
    KeyedCurveCapability, KeyedSemanticKey, KeyedTweakOutcome, keyed_leaf_program,
};
use target_elements::ReviewedElementsTapscriptDefinition;
use target_elements_conformance::constructor::curve::{generator, lift_x};
use target_elements_conformance::constructor::tagged::sha256;
use target_elements_conformance::constructor::{
    FixtureTapTree, TweakDefect, UNSPENDABLE_INTERNAL_KEY, control_block, leaf_hash,
    output_program, tweak, tweaked_key,
};

/// Supplied public arithmetic, not a target observation or independent verdict.
pub struct PublicKeyedCurve;

impl KeyedCurveCapability for PublicKeyedCurve {
    fn internal_key_is_a_point(&self, key: &[u8; 32]) -> bool {
        lift_x(key).is_ok()
    }
    fn output_key(&self, key: &[u8; 32], root: &[u8; 32]) -> KeyedTweakOutcome {
        match tweaked_key(key, &tweak(key, root)) {
            Ok((key, parity)) => KeyedTweakOutcome::OutputKey {
                key,
                parity: parity == 1,
            },
            Err(TweakDefect::InternalKeyNotOnCurve(_)) => KeyedTweakOutcome::InternalKeyNotAPoint,
            Err(TweakDefect::TweakNotAScalar) => KeyedTweakOutcome::TweakAboveGroupOrder,
            Err(TweakDefect::TweakedKeyIsIdentity) => KeyedTweakOutcome::TweakedPointIsIdentity,
        }
    }
}

/// Fixed public NUMS derivation; no generated-and-discarded secret is involved.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyedInternalKeyDerivation {
    /// X coordinate is SHA256 of the uncompressed secp256k1 generator.
    Sha256UncompressedGenerator,
}

/// Public control recipe; a single-leaf tree requires an empty sibling path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyedControlRecipe {
    /// Oddness of the tweaked point, checked rather than trusted.
    pub parity: bool,
    /// Sibling hashes, deepest first.
    pub siblings: Vec<[u8; 32]>,
    /// Exact version/parity, internal key and path encoding.
    pub bytes: Vec<u8>,
}

/// Public data sufficient to reconstruct and authenticate the exact recipe.
///
/// Fields are inspectable and mutable offers, not admitted authority. Use
/// [`reconstruct_keyed`] before treating an offered descriptor as canonical.
/// JSON is an experiment carrier, not a stable interchange contract.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyedConstructorDescriptor {
    /// Experiment recipe generation, currently one.
    pub schema: u32,
    /// Opaque public semantic bytes; never interpreted as a curve key.
    pub semantic_key: [u8; 32],
    /// Exact canonical key/drop, birth, continuation and true encoding.
    pub leaf_bytes: Vec<u8>,
    /// Reviewed Elements leaf version.
    pub leaf_version: u8,
    /// Public NUMS provenance.
    pub internal_key_derivation: KeyedInternalKeyDerivation,
    /// The fixed NUMS x coordinate, not a deployment choice.
    pub internal_key: [u8; 32],
    /// Complete committed tree, never a supplied root alone.
    pub tree: FixtureTapTree,
    /// Target-tagged leaf commitment.
    pub leaf_hash: [u8; 32],
    /// Root of the complete tree; identical to the sole leaf hash.
    pub merkle_root: [u8; 32],
    /// Public tweaked x-only output key.
    pub output_key: [u8; 32],
    /// Exact witness-version-one scriptPubKey.
    pub output_program: Vec<u8>,
    /// Authenticated leaf control recipe.
    pub control: KeyedControlRecipe,
}

/// Typed offline construction/policy refusal, never a native script verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyedConstructorRefusal {
    /// A different internal key or derivation was offered.
    InternalKeyPolicy,
    /// The complete tree is not exactly one script leaf.
    LeafCensus,
    /// A field differs from the reconstructed canonical recipe.
    DescriptorMismatch,
    /// The reviewed typed leaf could not be encoded.
    LeafConstruction,
    /// NUMS point cannot be lifted by the supplied capability.
    InternalKeyNotAPoint,
    /// Tweak overflow is terminal; the semantic key is never ground.
    TweakAboveGroupOrder,
    /// Point sum is infinity; no output is substituted.
    TweakedPointIsIdentity,
    /// The funded predecessor does not commit to these exact control bytes.
    PredecessorCommitment,
}

/// Construct from public semantic bytes using the fixed policy and supplied curve.
/// # Errors
/// Returns a leaf, NUMS-policy or precise curve refusal without changing the key.
pub fn construct_keyed(
    target: &ReviewedElementsTapscriptDefinition,
    key: KeyedSemanticKey,
    curve: &impl KeyedCurveCapability,
) -> Result<KeyedConstructorDescriptor, KeyedConstructorRefusal> {
    let _span = tracing::info_span!("construct_keyed").entered();
    let result = construct(target, key, curve);
    tracing::info!(constructed = result.is_ok(), refusal = ?result.as_ref().err(), "keyed construction finished");
    result
}

fn construct(
    target: &ReviewedElementsTapscriptDefinition,
    key: KeyedSemanticKey,
    curve: &impl KeyedCurveCapability,
) -> Result<KeyedConstructorDescriptor, KeyedConstructorRefusal> {
    let internal_key = sha256(&generator().uncompressed_bytes());
    if internal_key != UNSPENDABLE_INTERNAL_KEY {
        return Err(KeyedConstructorRefusal::InternalKeyPolicy);
    }
    if !curve.internal_key_is_a_point(&internal_key) {
        return Err(KeyedConstructorRefusal::InternalKeyNotAPoint);
    }
    let leaf_bytes = keyed_leaf_program(target, key)
        .map_err(|_| KeyedConstructorRefusal::LeafConstruction)?
        .encode(target);
    let version = target.definition().leaf_version();
    let tree = FixtureTapTree::leaf_of_version(version.get(), leaf_bytes.clone());
    let leaf_hash = leaf_hash(version, &leaf_bytes);
    let merkle_root = tree.node_hash();
    let siblings = tree
        .path_to(&leaf_hash)
        .map_err(|_| KeyedConstructorRefusal::LeafCensus)?;
    let (output_key, parity) = match curve.output_key(&internal_key, &merkle_root) {
        KeyedTweakOutcome::OutputKey { key, parity } => (key, parity),
        KeyedTweakOutcome::InternalKeyNotAPoint => {
            return Err(KeyedConstructorRefusal::InternalKeyNotAPoint);
        }
        KeyedTweakOutcome::TweakAboveGroupOrder => {
            return Err(KeyedConstructorRefusal::TweakAboveGroupOrder);
        }
        KeyedTweakOutcome::TweakedPointIsIdentity => {
            return Err(KeyedConstructorRefusal::TweakedPointIsIdentity);
        }
    };
    let bytes = control_block(version, u8::from(parity), &internal_key, &siblings);
    Ok(KeyedConstructorDescriptor {
        schema: 1,
        semantic_key: *key.bytes(),
        leaf_bytes,
        leaf_version: version.get(),
        internal_key_derivation: KeyedInternalKeyDerivation::Sha256UncompressedGenerator,
        internal_key,
        tree,
        leaf_hash,
        merkle_root,
        output_key,
        output_program: output_program(&output_key),
        control: KeyedControlRecipe {
            parity,
            siblings,
            bytes,
        },
    })
}

/// Recompute every commitment from the public offer, without creator state.
/// # Errors
/// Refuses foreign internal keys, extra leaves and any noncanonical field.
pub fn reconstruct_keyed(
    target: &ReviewedElementsTapscriptDefinition,
    offered: &KeyedConstructorDescriptor,
    curve: &impl KeyedCurveCapability,
) -> Result<KeyedConstructorDescriptor, KeyedConstructorRefusal> {
    let _span = tracing::debug_span!("reconstruct_keyed").entered();
    if offered.internal_key != UNSPENDABLE_INTERNAL_KEY {
        return Err(KeyedConstructorRefusal::InternalKeyPolicy);
    }
    if !matches!(offered.tree, FixtureTapTree::Leaf { .. }) {
        return Err(KeyedConstructorRefusal::LeafCensus);
    }
    let rebuilt = construct_keyed(target, KeyedSemanticKey::new(offered.semantic_key), curve)?;
    if rebuilt != *offered {
        return Err(KeyedConstructorRefusal::DescriptorMismatch);
    }
    Ok(rebuilt)
}

/// Authenticate the full public recipe and its sole control against a predecessor.
///
/// The caller obtains `predecessor_program` from its funded output. Offline tests
/// use explicit synthetic funding, never label it a mined observation.
/// # Errors
/// Refuses recipe drift or a different predecessor commitment.
pub fn authenticate_keyed_predecessor(
    target: &ReviewedElementsTapscriptDefinition,
    descriptor: &KeyedConstructorDescriptor,
    predecessor_program: &[u8],
    curve: &impl KeyedCurveCapability,
) -> Result<(), KeyedConstructorRefusal> {
    let _span = tracing::debug_span!("authenticate_keyed_predecessor").entered();
    let rebuilt = reconstruct_keyed(target, descriptor, curve)?;
    if rebuilt.output_program != predecessor_program {
        return Err(KeyedConstructorRefusal::PredecessorCommitment);
    }
    Ok(())
}

/// Complete path inventory: one script path plus the unavoidable key path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyedSpendPath {
    /// The sole key-committing leaf, with both predicates and its control.
    CanonicalScriptLeaf,
    /// Present in taproot; inability to sign remains conditional NUMS reasoning.
    ConditionalNumsKeyPath,
}

/// Native rows prepared here but neither executed nor admitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyedNativeSubject {
    /// Flip only bit zero of the same control; expect commitment refusal.
    WrongControlParity,
    /// Change the offered leaf with the original control; expect commitment refusal.
    UncommittedLeaf,
    /// Replace script-path witness with a key-path attempt on the same coin.
    KeyPathAttempt,
}

impl KeyedNativeSubject {
    /// One changed coordinate, with the canonical script spend as control.
    #[must_use]
    pub const fn changed_coordinate(self) -> &'static str {
        match self {
            Self::WrongControlParity => {
                "control byte zero parity bit only; leaf and transaction fixed"
            }
            Self::UncommittedLeaf => {
                "offered leaf bytes only; original control and transaction fixed"
            }
            Self::KeyPathAttempt => {
                "witness path selection only; same funded constructor and transaction"
            }
        }
    }
    /// Declared target boundary, not an observed result.
    #[must_use]
    pub const fn expected_boundary(self) -> &'static str {
        match self {
            Self::WrongControlParity | Self::UncommittedLeaf => {
                "taproot script commitment refusal before either clause"
            }
            Self::KeyPathAttempt => "native key-path signature refusal, separately classified",
        }
    }
    /// Derived collateral permitted beside the otherwise fixed control.
    #[must_use]
    pub const fn collateral(self) -> &'static str {
        "witness bytes, witness transaction identity, size/weight and observation; no non-witness transaction change"
    }
    /// No native evidence exists in this offline manifest.
    #[must_use]
    pub const fn standing(self) -> &'static str {
        "capture outstanding; no native outcome admitted"
    }
}

/// Candidate path census and native work still required for measurement (c).
pub struct KeyedConstructorManifest {
    /// Exact recipe, not an accepted run.
    descriptor: KeyedConstructorDescriptor,
    /// Exactly one script leaf and the single taproot key path.
    paths: [KeyedSpendPath; 2],
    /// Target-bound subjects; every outcome is outstanding.
    native_subjects: [KeyedNativeSubject; 3],
}

impl KeyedConstructorManifest {
    /// Exact reconstructed public recipe.
    #[must_use]
    pub const fn descriptor(&self) -> &KeyedConstructorDescriptor {
        &self.descriptor
    }
    /// Complete path inventory, including the conditional key path.
    #[must_use]
    pub const fn paths(&self) -> &[KeyedSpendPath; 2] {
        &self.paths
    }
    /// The three described but unexecuted native subjects.
    #[must_use]
    pub const fn native_subjects(&self) -> &[KeyedNativeSubject; 3] {
        &self.native_subjects
    }
    /// Reconstruct and bind the census to an actual offered predecessor program.
    /// # Errors
    /// Refuses a noncanonical descriptor or unbound predecessor commitment.
    pub fn new(
        target: &ReviewedElementsTapscriptDefinition,
        descriptor: &KeyedConstructorDescriptor,
        predecessor_program: &[u8],
        curve: &impl KeyedCurveCapability,
    ) -> Result<Self, KeyedConstructorRefusal> {
        authenticate_keyed_predecessor(target, descriptor, predecessor_program, curve)?;
        Ok(Self {
            descriptor: descriptor.clone(),
            paths: [
                KeyedSpendPath::CanonicalScriptLeaf,
                KeyedSpendPath::ConditionalNumsKeyPath,
            ],
            native_subjects: [
                KeyedNativeSubject::WrongControlParity,
                KeyedNativeSubject::UncommittedLeaf,
                KeyedNativeSubject::KeyPathAttempt,
            ],
        })
    }
}
