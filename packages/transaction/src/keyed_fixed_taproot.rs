//! Fixed-form keyed commitment and local authentication, not chain recognition.
//!
//! One script path contains the fixed-form conjuncts. The unavoidable key path is conditional
//! on NUMS/discrete-log and SHA256 preimage assumptions; generated-key separation
//! needs second-preimage resistance and adversarial pairs need collision resistance.

use linker::keyed_fixed_bundle::LinkedKeyedFixedBundle;
use sha2::{Digest, Sha256};
use tapscript::keyed_constructor::KeyedTweakOutcome;
use tapscript::keyed_fixed_constructor::KeyedFixedLeafRecipe;
use target_elements::ReviewedElementsTapscriptDefinition;

use crate::TransactionRefusal;
use crate::keyed_curve::{apply_tweak, tweak};
use crate::taproot::{CONTROL_BASE_BYTES, TAPROOT_LEAF_MASK, leaf_hash, witness_program_script};

/// Constructor-policy/arithmetic refusal, never a script execution result.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedFixedCommitmentRefusal {
    InternalKeyDerivationMismatch,
    InternalKeyNotAPoint,
    TweakAboveGroupOrder,
    TweakedPointIsIdentity,
    Target(TransactionRefusal),
}
impl KeyedFixedCommitmentRefusal {
    /// Static ADR-026 refusal classification.
    #[must_use]
    pub fn refusal_kind(&self) -> &'static str {
        crate::trace_support::TraceRefusal::tracing_kind(self)
    }
}

impl crate::trace_support::TraceRefusal for KeyedFixedCommitmentRefusal {
    fn tracing_kind(&self) -> &'static str {
        match self {
            Self::InternalKeyDerivationMismatch => "internal_key_derivation_mismatch",
            Self::InternalKeyNotAPoint => "internal_key_not_a_point",
            Self::TweakAboveGroupOrder => "tweak_above_group_order",
            Self::TweakedPointIsIdentity => "tweaked_point_is_identity",
            Self::Target(_) => "target",
        }
    }
}

/// Local commitment mismatch, distinct from construction policy and execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedFixedAuthenticationRefusal {
    WrongProgram,
    WrongLeaf,
    WrongVersion,
    WrongParity,
    WrongControl,
}

impl KeyedFixedAuthenticationRefusal {
    /// Static ADR-026 refusal classification.
    #[must_use]
    pub fn refusal_kind(&self) -> &'static str {
        crate::trace_support::TraceRefusal::tracing_kind(self)
    }
}

impl crate::trace_support::TraceRefusal for KeyedFixedAuthenticationRefusal {
    fn tracing_kind(&self) -> &'static str {
        match self {
            Self::WrongProgram => "wrong_program",
            Self::WrongLeaf => "wrong_leaf",
            Self::WrongVersion => "wrong_version",
            Self::WrongParity => "wrong_parity",
            Self::WrongControl => "wrong_control",
        }
    }
}

/// Complete path census under the recipe's explicit cryptographic assumptions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyedFixedSpendPath {
    ScriptWithBothClauses,
    ConditionalNumsKeyPath,
}

/// Immutable commitment computed only from a linked fixed-form bundle.
///
/// ```compile_fail,E0451
/// use linker::keyed_fixed_bundle::LinkedKeyedFixedBundle;
/// use transaction::keyed_fixed_taproot::KeyedFixedCommitment;
/// fn forge(linked: LinkedKeyedFixedBundle) -> KeyedFixedCommitment {
///     KeyedFixedCommitment { linked, internal_key: [0; 32], root: [0; 32],
///         tweak: [0; 32], output_key: [0; 32], parity: false,
///         output_program: Vec::new(), control: [0; 33] }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedFixedCommitment {
    linked: LinkedKeyedFixedBundle,
    internal_key: [u8; 32],
    root: [u8; 32],
    tweak: [u8; 32],
    output_key: [u8; 32],
    parity: bool,
    output_program: Vec<u8>,
    control: [u8; CONTROL_BASE_BYTES],
}

impl KeyedFixedCommitment {
    /// Retained binding, selection, obligations and sealed emission.
    #[must_use]
    pub const fn linked(&self) -> &LinkedKeyedFixedBundle {
        &self.linked
    }
    /// Normative construction recipe.
    #[must_use]
    pub const fn recipe(&self) -> &KeyedFixedLeafRecipe {
        self.linked.project().successor.recipe()
    }
    /// Exact linked leaf bytes, without re-emission.
    #[must_use]
    pub fn leaf(&self) -> &[u8] {
        self.linked.bytes()
    }
    /// Reviewed leaf version.
    #[must_use]
    pub const fn leaf_version(&self) -> u8 {
        self.linked.project().successor.leaf_version().get()
    }
    /// Public NUMS x, recomputed from the uncompressed generator.
    #[must_use]
    pub const fn internal_key(&self) -> &[u8; 32] {
        &self.internal_key
    }
    /// The sole leaf hash is the complete root; there are no branches.
    #[must_use]
    pub const fn root(&self) -> &[u8; 32] {
        &self.root
    }
    /// Big-endian tweak bytes, never reduced modulo the order.
    #[must_use]
    pub const fn tweak(&self) -> &[u8; 32] {
        &self.tweak
    }
    /// Tweaked x-only key.
    #[must_use]
    pub const fn output_key(&self) -> &[u8; 32] {
        &self.output_key
    }
    /// Oddness of Q's y coordinate.
    #[must_use]
    pub const fn parity(&self) -> bool {
        self.parity
    }
    /// Full witness-v1 scriptPubKey, not only its x-only payload.
    #[must_use]
    pub fn output_program(&self) -> &[u8] {
        &self.output_program
    }
    /// Depth-zero `(version | parity) || internal x` control.
    #[must_use]
    pub const fn control(&self) -> &[u8; CONTROL_BASE_BYTES] {
        &self.control
    }
    /// Exactly one script path plus the conditional key path, not an execution proof.
    #[must_use]
    pub const fn paths(&self) -> [KeyedFixedSpendPath; 2] {
        [
            KeyedFixedSpendPath::ScriptWithBothClauses,
            KeyedFixedSpendPath::ConditionalNumsKeyPath,
        ]
    }
}

/// Sealed borrowed local equality receipt, produced only inside transaction.
///
/// The public equality check returns unit. The keyed request path retains this
/// receipt only inside a parent-bound input after canonical decoding and selected-
/// outpoint checks; callers cannot supply it as evidence of a real predecessor.
/// No linked bundle is cloned.
/// ```compile_fail,E0451
/// use transaction::keyed_fixed_taproot::{AuthenticatedKeyedFixedPredecessor, KeyedFixedCommitment};
/// fn forge(commitment: &KeyedFixedCommitment) -> AuthenticatedKeyedFixedPredecessor<'_> {
///     AuthenticatedKeyedFixedPredecessor { commitment }
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuthenticatedKeyedFixedPredecessor<'a> {
    commitment: &'a KeyedFixedCommitment,
}

impl AuthenticatedKeyedFixedPredecessor<'_> {
    /// The locally compared commitment; the request input retains parent provenance.
    #[must_use]
    pub const fn commitment(&self) -> &KeyedFixedCommitment {
        self.commitment
    }
}

/// Compute fixed-form commitment using production arithmetic and existing hashing.
///
/// # Errors
/// Returns a typed constructor-policy/arithmetic or unchanged target refusal.
pub fn commit_keyed_fixed_bundle(
    target: &ReviewedElementsTapscriptDefinition,
    linked: &LinkedKeyedFixedBundle,
) -> Result<KeyedFixedCommitment, KeyedFixedCommitmentRefusal> {
    let _span = tracing::debug_span!(target: "transaction::keyed_fixed_taproot", "commit_keyed_fixed_bundle", family = "keyed").entered();
    let result = commit_core(target, linked);
    match &result {
        Ok(_) => {
            tracing::debug!(target: "transaction::keyed_fixed_taproot", family = "keyed", outcome = "completed", script_path_count = 1_u64, key_path_count = 1_u64, "keyed fixed commitment completed");
        }
        Err(error) => {
            tracing::debug!(target: "transaction::keyed_fixed_taproot", family = "keyed", outcome = "refused", refusal_kind = error.refusal_kind(), "keyed fixed commitment refused");
        }
    }
    result
}

// The public entry fixes the production adapter. Tests inject only here to
// capture exceptional returns without searching for cryptographic preimages.
fn commit_core(
    target: &ReviewedElementsTapscriptDefinition,
    linked: &LinkedKeyedFixedBundle,
) -> Result<KeyedFixedCommitment, KeyedFixedCommitmentRefusal> {
    let material = &linked.project().successor;
    let mut generator = [0; 65];
    generator[0] = 4;
    generator[1..33].copy_from_slice(&tapscript::STATE_GENERATOR_X);
    generator[33..].copy_from_slice(&tapscript::STATE_GENERATOR_Y);
    // Reuse public SEC1 constants only, not STATE metadata or covenant semantics.
    let internal_key: [u8; 32] = Sha256::digest(generator).into();
    if internal_key != tapscript::STATE_NUMS_KEY {
        return Err(KeyedFixedCommitmentRefusal::InternalKeyDerivationMismatch);
    }
    if secp256k1_zkp::XOnlyPublicKey::from_slice(&internal_key).is_err() {
        return Err(KeyedFixedCommitmentRefusal::InternalKeyNotAPoint);
    }
    let root = leaf_hash(material.leaf_version(), linked.bytes());
    let tweak = tweak(&internal_key, &root);
    let (output_key, parity) = match apply_tweak(&internal_key, tweak) {
        KeyedTweakOutcome::OutputKey { key, parity } => (key, parity),
        KeyedTweakOutcome::InternalKeyNotAPoint => {
            return Err(KeyedFixedCommitmentRefusal::InternalKeyNotAPoint);
        }
        KeyedTweakOutcome::TweakAboveGroupOrder => {
            return Err(KeyedFixedCommitmentRefusal::TweakAboveGroupOrder);
        }
        KeyedTweakOutcome::TweakedPointIsIdentity => {
            return Err(KeyedFixedCommitmentRefusal::TweakedPointIsIdentity);
        }
    };
    let output_program = witness_program_script(target, 1, &output_key)
        .map_err(KeyedFixedCommitmentRefusal::Target)?;
    Ok(KeyedFixedCommitment {
        linked: linked.clone(),
        internal_key,
        root,
        tweak,
        output_key,
        parity,
        output_program,
        control: keyed_fixed_control(material.leaf_version().get(), parity, &internal_key),
    })
}

// KeyedFixed sole-leaf builder; no ASH role, pin, sibling or caller policy is involved.
fn keyed_fixed_control(
    version: u8,
    parity: bool,
    internal_key: &[u8; 32],
) -> [u8; CONTROL_BASE_BYTES] {
    let mut control = [0; CONTROL_BASE_BYTES];
    control[0] = (version & TAPROOT_LEAF_MASK) | u8::from(parity);
    control[1..].copy_from_slice(internal_key);
    control
}

/// Check a supplied predecessor's full program against a sealed commitment.
///
/// This returns only equality, not a receipt. Parent/outpoint and chain evidence
/// are still owed by the caller; the public check cannot mint an authenticated input.
/// # Errors
/// Returns `WrongProgram` for any other program, including another key's valid recipe.
pub fn authenticate_keyed_fixed_predecessor(
    commitment: &KeyedFixedCommitment,
    predecessor_program: &[u8],
) -> Result<(), KeyedFixedAuthenticationRefusal> {
    let _span = tracing::debug_span!(target: "transaction::keyed_fixed_taproot", "authenticate_keyed_fixed_predecessor", family = "keyed").entered();
    let result =
        authenticate_keyed_fixed_predecessor_core(commitment, predecessor_program).map(|_| ());
    match &result {
        Ok(()) => {
            tracing::debug!(target: "transaction::keyed_fixed_taproot", family = "keyed", outcome = "completed", "keyed predecessor authenticated");
        }
        Err(error) => {
            tracing::debug!(target: "transaction::keyed_fixed_taproot", family = "keyed", outcome = "refused", refusal_kind = error.refusal_kind(), "keyed predecessor refused");
        }
    }
    result
}

// The public equality entry discards this receipt. The request path retains it
// only with canonical parent and selected-outpoint provenance; never accept one
// from callers.
pub(crate) fn authenticate_keyed_fixed_predecessor_core<'a>(
    commitment: &'a KeyedFixedCommitment,
    predecessor_program: &[u8],
) -> Result<AuthenticatedKeyedFixedPredecessor<'a>, KeyedFixedAuthenticationRefusal> {
    if commitment.output_program() == predecessor_program {
        Ok(AuthenticatedKeyedFixedPredecessor { commitment })
    } else {
        Err(KeyedFixedAuthenticationRefusal::WrongProgram)
    }
}

/// Check a proposed script/control pair against the sealed canonical commitment.
///
/// No script execution or policy admission occurs. This accepts only the recipe's
/// sole script path, not arbitrary taproot trees.
/// # Errors
/// Refuses a wrong leaf, version, parity or control at local commitment authentication.
pub fn authenticate_keyed_fixed_path(
    commitment: &KeyedFixedCommitment,
    leaf: &[u8],
    control: &[u8],
) -> Result<(), KeyedFixedAuthenticationRefusal> {
    let _span = tracing::debug_span!(target: "transaction::keyed_fixed_taproot", "authenticate_keyed_fixed_path", family = "keyed").entered();
    let result = path_core(commitment, leaf, control);
    match &result {
        Ok(()) => {
            tracing::debug!(target: "transaction::keyed_fixed_taproot", family = "keyed", outcome = "completed", "keyed path authenticated");
        }
        Err(error) => {
            tracing::debug!(target: "transaction::keyed_fixed_taproot", family = "keyed", outcome = "refused", refusal_kind = error.refusal_kind(), "keyed path refused");
        }
    }
    result
}

fn path_core(
    c: &KeyedFixedCommitment,
    leaf: &[u8],
    control: &[u8],
) -> Result<(), KeyedFixedAuthenticationRefusal> {
    if leaf != c.leaf() {
        return Err(KeyedFixedAuthenticationRefusal::WrongLeaf);
    }
    if control.len() != CONTROL_BASE_BYTES {
        return Err(KeyedFixedAuthenticationRefusal::WrongControl);
    }
    if control[0] & TAPROOT_LEAF_MASK != c.leaf_version() {
        return Err(KeyedFixedAuthenticationRefusal::WrongVersion);
    }
    if control[0] & 1 != u8::from(c.parity()) {
        return Err(KeyedFixedAuthenticationRefusal::WrongParity);
    }
    if control != c.control() {
        return Err(KeyedFixedAuthenticationRefusal::WrongControl);
    }
    Ok(())
}
