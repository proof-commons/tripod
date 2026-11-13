//! Offline freeze offers, not native admission or a stable descriptor format.
//!
//! Source digests pin selected source files and their non-test child modules,
//! not a particular semantic key or a transitive dependency graph.
//! A new source revision requires explicit re-freeze and review; runtime values
//! are inventoried separately and cannot masquerade as frozen evidence.
//! `SourceDrift` checks compile-time included bytes from the build checkout, not
//! the runtime tree, executable, toolchain or resolved dependency graph. A moved
//! or copied checkout must rebuild vectors before this evidence counts.

use std::collections::BTreeMap;
use target_elements_conformance::constructor::tagged::sha256;

mod pins;

/// Exact ASCII bytes fixed by the adoption before generating any new fixture.
pub const SECOND_KEY_PHRASE: &str = "Tripod Wave 1 supplied-key independence control";

/// Canonical suffix after push32(key); no clause reads the discarded key bytes.
pub const LEAF_SUFFIX: [u8; 19] = [
    0x75, 0xcd, 0xc7, 0x75, 4, 0, 0, 0, 0, 0x88, 0x75, 0xcd, 0xca, 0, 0xd1, 0x7b, 0x88, 0x88, 0x51,
];

pub use pins::SECOND_KEY;

/// Reviewed once, with exact source identities in each offered row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FrozenItem {
    /// Architecture's keyed declaration contract.
    Declarations,
    /// Realization's typed SOURCE/MERGE derivation.
    Realization,
    /// Sealed key-free compiler plans and their engine.
    Plans,
    /// Proof zero and placement zero per selected operation, key-free source rule.
    Selection,
    /// The fixed instruction/literal grammar and zero-relocation acceptance.
    Grammar,
    /// Schema-1 leaf/control function and production arithmetic.
    Recipe,
    /// Reviewed target definition and encoding.
    Target,
    /// Request/input ABI and explicit external obligations.
    Abi,
    /// Workspace versions and transaction dependency roles.
    Dependencies,
    /// Ordered construction, frozen bytes and separate ordinary signing.
    FundingSigning,
    /// Measured-envelope constructor admission and independent scaling formula.
    PolicyEnvelope,
    /// Constructor offers and authenticated introspection-reference census.
    References,
}

/// Values recomputed per key or run, never inherited from a previous capture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComputedItem {
    /// Canonical key-instantiated leaf bytes.
    Leaf,
    /// Tagged leaf hash.
    LeafHash,
    /// Single-leaf root.
    Root,
    /// Unreduced tagged tweak.
    Tweak,
    /// Witness-v1 output program.
    OutputProgram,
    /// Tweaked point parity.
    Parity,
    /// Depth-zero control block.
    Control,
    /// Sealed production commitment.
    Commitment,
    /// Actual parent/outpoint and request binding.
    RequestAssociation,
    /// Independently reconstructed per-key handle sets.
    HandleSets,
    /// Recorded ordinary funding witnesses, not covenant signatures.
    Signatures,
    /// Complete and stripped transaction identities.
    TransactionIdentities,
    /// Asset, fee, provenance, rates, minimums, ceilings, package weight and
    /// sponsor signature bound supplied for this run.
    PolicyReading,
    /// SOURCE/MERGE scope, ordered inputs, sequences, values and output destinations.
    ConstructionChoices,
    /// Per-key emission/link projection, recipe offers and closure occurrences.
    EmissionAndClosure,
    /// Serialized whole-transaction weight and virtual size, including DER length.
    FinalResources,
}

/// Complete variable inventory; order is descriptive, not a wire encoding.
pub const COMPUTED_ITEMS: [ComputedItem; 16] = [
    ComputedItem::Leaf,
    ComputedItem::LeafHash,
    ComputedItem::Root,
    ComputedItem::Tweak,
    ComputedItem::OutputProgram,
    ComputedItem::Parity,
    ComputedItem::Control,
    ComputedItem::Commitment,
    ComputedItem::RequestAssociation,
    ComputedItem::HandleSets,
    ComputedItem::Signatures,
    ComputedItem::TransactionIdentities,
    ComputedItem::PolicyReading,
    ComputedItem::ConstructionChoices,
    ComputedItem::EmissionAndClosure,
    ComputedItem::FinalResources,
];

/// Exact file bytes, rather than an unstable debug rendering or a Git identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceIdentity {
    /// Repository-relative source path.
    pub path: String,
    /// SHA-256 of the entire source file at freeze.
    pub digest: [u8; 32],
}

/// Structural measured envelope, not an asset choice or a fresh policy reading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Envelope {
    /// Construction-policy handle ceiling, not a script law.
    pub handles: u64,
    /// Maximum ordinary sponsor signature including sighash byte.
    pub signature_bytes: u64,
    /// Relay weight ceiling in weight units.
    pub weight_units: u64,
    /// One P2WPKH ordinary sponsor.
    pub sponsors: u64,
    /// Successor, ordinary change and explicit fee.
    pub outputs: u64,
}

/// Mutable offline offer. Validation binds it to a caller's explicit key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreezeManifest {
    /// All twelve reviewed categories with their exact source pins.
    pub frozen: BTreeMap<FrozenItem, Vec<SourceIdentity>>,
    /// Values that remain per-instance, not frozen observations.
    pub computed: Vec<ComputedItem>,
    /// Public key data, not a signing key.
    pub key: [u8; 32],
    /// Canonical leaf identity for that key, independently spelled here.
    pub leaf_digest: [u8; 32],
    /// Reviewed constructor schema.
    pub recipe_schema: u32,
    /// Reviewed target definition version.
    pub target_version: u32,
    /// Explicit measured structural envelope.
    pub envelope: Envelope,
}

/// One condition per refusal; none is covenant execution or native admission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FreezeRefusal {
    /// Text supplied a key of the wrong byte width.
    KeyWidth,
    /// Hexadecimal input has a non-hex character.
    KeyEncoding,
    /// An included source has drifted from the reviewed pin.
    SourceDrift(String),
    /// Offered source inventory differs from the reviewed rows.
    FrozenInventory,
    /// Variable inventory is missing or substitutes a frozen fact.
    ComputedInventory,
    /// Key offer differs from the caller's key.
    KeyBinding,
    /// Leaf identity differs from the canonical template instantiation.
    LeafBinding,
    /// Offered constructor schema differs.
    Recipe,
    /// Offered target version differs.
    Target,
    /// Offered envelope differs from the recorded one.
    Policy,
}

/// Offline ownership of a refusal, never a covenant or native verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FreezeBoundary {
    /// Untrusted textual public key input.
    TextKey,
    /// Reviewed source and inventory identity.
    SourceFreeze,
    /// Per-key instantiation of the frozen recipe, target and policy.
    ArtifactBinding,
}

impl FreezeRefusal {
    /// Exhaustive boundary mapping.
    #[must_use]
    pub const fn boundary(&self) -> FreezeBoundary {
        match self {
            Self::KeyWidth | Self::KeyEncoding => FreezeBoundary::TextKey,
            Self::SourceDrift(_) | Self::FrozenInventory | Self::ComputedInventory => {
                FreezeBoundary::SourceFreeze
            }
            Self::KeyBinding | Self::LeafBinding | Self::Recipe | Self::Target | Self::Policy => {
                FreezeBoundary::ArtifactBinding
            }
        }
    }
    /// Exhaustive diagnostic classification for both conformance boundaries.
    #[must_use]
    pub const fn refusal_kind(&self) -> &'static str {
        match self {
            Self::KeyWidth => "key_width",
            Self::KeyEncoding => "key_encoding",
            Self::SourceDrift(_) => "source_drift",
            Self::FrozenInventory => "frozen_inventory",
            Self::ComputedInventory => "computed_inventory",
            Self::KeyBinding => "key_binding",
            Self::LeafBinding => "leaf_binding",
            Self::Recipe => "recipe",
            Self::Target => "target",
            Self::Policy => "policy",
        }
    }
}

/// Canonical template prediction; it never invokes an emitter or decoder.
#[must_use]
pub fn predicted_leaf(key: [u8; 32]) -> Vec<u8> {
    std::iter::once(32).chain(key).chain(LEAF_SUFFIX).collect()
}

impl FreezeManifest {
    /// Construct an untrusted offer from the retained review pins, not a verdict.
    #[must_use]
    pub fn offer(key: [u8; 32]) -> Self {
        let mut frozen = BTreeMap::<_, Vec<_>>::new();
        for (kind, path, _, digest) in pins::SOURCES {
            frozen.entry(*kind).or_default().push(SourceIdentity {
                path: (*path).into(),
                digest: *digest,
            });
        }
        Self {
            frozen,
            computed: COMPUTED_ITEMS.to_vec(),
            key,
            leaf_digest: sha256(&predicted_leaf(key)),
            recipe_schema: 1,
            target_version: 2,
            envelope: Envelope {
                handles: 1564,
                signature_bytes: 72,
                weight_units: 400_000,
                sponsors: 1,
                outputs: 3,
            },
        }
    }
}

/// Validate a freeze offer. No producer receipt, signature or chain fact is trusted.
/// # Errors
/// Returns the exact inventory/binding condition that differs, in source-first order.
pub fn validate_manifest(offered: &FreezeManifest, key: [u8; 32]) -> Result<(), FreezeRefusal> {
    let _span =
        tracing::info_span!(target: "vectors::keyed_final_manifest", "validate_keyed_freeze")
            .entered();
    let result = validate_core(offered, key);
    match &result {
        Ok(()) => tracing::info!(target: "vectors::keyed_final_manifest", "keyed freeze validated"),
        Err(error) => {
            tracing::debug!(target: "vectors::keyed_final_manifest", refusal_kind = error.refusal_kind(), "keyed freeze refused");
        }
    }
    result
}

fn validate_core(offered: &FreezeManifest, key: [u8; 32]) -> Result<(), FreezeRefusal> {
    for (_, path, bytes, digest) in pins::SOURCES {
        if sha256(bytes) != *digest {
            return Err(FreezeRefusal::SourceDrift((*path).into()));
        }
    }
    let expected = FreezeManifest::offer(key);
    if offered.frozen != expected.frozen {
        return Err(FreezeRefusal::FrozenInventory);
    }
    if offered.computed != expected.computed {
        return Err(FreezeRefusal::ComputedInventory);
    }
    if offered.key != key {
        return Err(FreezeRefusal::KeyBinding);
    }
    if offered.leaf_digest != expected.leaf_digest {
        return Err(FreezeRefusal::LeafBinding);
    }
    if offered.recipe_schema != expected.recipe_schema {
        return Err(FreezeRefusal::Recipe);
    }
    if offered.target_version != expected.target_version {
        return Err(FreezeRefusal::Target);
    }
    if offered.envelope != expected.envelope {
        return Err(FreezeRefusal::Policy);
    }
    Ok(())
}

/// Parse 64 hexadecimal ASCII digits as 32 bytes of public key data.
/// Upper- and lower-case digits are accepted: this test convenience is not a
/// unique textual representation of a key.
/// # Errors
/// Refuses wrong width before invalid hex; accepts zero and high-bit values.
pub fn parse_key(text: &str) -> Result<[u8; 32], FreezeRefusal> {
    let _span =
        tracing::debug_span!(target: "vectors::keyed_final_manifest", "parse_keyed_freeze_key")
            .entered();
    let result = parse_core(text);
    match &result {
        Ok(_) => {
            tracing::debug!(target: "vectors::keyed_final_manifest", "keyed freeze key parsed");
        }
        Err(error) => {
            tracing::debug!(target: "vectors::keyed_final_manifest", refusal_kind = error.refusal_kind(), "keyed freeze key refused");
        }
    }
    result
}

fn parse_core(text: &str) -> Result<[u8; 32], FreezeRefusal> {
    if text.len() != 64 {
        return Err(FreezeRefusal::KeyWidth);
    }
    let mut result = [0; 32];
    for (out, digits) in result.iter_mut().zip(text.as_bytes().as_chunks::<2>().0) {
        *out = hex(digits[0])? * 16 + hex(digits[1])?;
    }
    Ok(result)
}

const fn hex(byte: u8) -> Result<u8, FreezeRefusal> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(FreezeRefusal::KeyEncoding),
    }
}
