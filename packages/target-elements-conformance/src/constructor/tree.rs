//! The typed tree a constructor fixture states, and the exact target
//! values it determines.
//!
//! # An independent reference, on purpose
//!
//! Everything here is computed from a reading of the target's own
//! source, using this package's own hashing and curve arithmetic. It
//! asks the typed program builder nothing, and it asks the upstream
//! functional-test framework nothing
//! `(´[PLAN-rule:guide10:constructor-oracle]´)`.
//!
//! That independence is the entire value: an expectation computed by
//! the same code that produces the thing under test agrees with itself
//! no matter how wrong both are. What this module produces is a second
//! opinion, to be compared against the framework's construction and
//! against a target-native verdict.
//!
//! # What is a target fact and what is a prototype choice
//!
//! The tag strings, the leaf version, the child ordering, the framing,
//! and the output-program encoding are target facts, transcribed with
//! provenance. The metadata schema and the choice of internal key are
//! prototype choices. The two are kept separately labelled throughout,
//! because only the first kind can be wrong about the target.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use target_elements::LeafVersion;

use crate::constructor::curve::{
    self, CurvePoint, FIELD_ELEMENT_BYTES, PointDecodingDefect, is_valid_scalar, lift_x,
};
use crate::constructor::tagged::{
    Digest32, TAP_BRANCH_TAG, TAP_LEAF_TAG, TAP_TWEAK_TAG, compact_size, tagged_hash,
};

/// The witness version byte a taproot output program carries.
///
/// Provenance: the target's segregated-witness version 1 program, which
/// the script writes as the small-integer opcode for one.
const WITNESS_VERSION_ONE: u8 = 0x51;

/// The parity bit's position in a control block's first byte.
///
/// Provenance: `src/script/interpreter.cpp:3234`, which passes
/// `control[0] & 1` as the parity to `CheckTapTweak`, and
/// `src/script/interpreter.h:280`, `TAPROOT_LEAF_MASK = 0xfe`, which is
/// how the leaf version is recovered from the same byte.
const PARITY_MASK: u8 = 0x01;

/// How many bytes a control block carries before its path.
///
/// Provenance: `src/script/interpreter.h:283`,
/// `TAPROOT_CONTROL_BASE_SIZE = 33` — one byte of version and parity,
/// then the 32-byte internal key.
pub const CONTROL_BASE_BYTES: usize = 33;

/// How many path nodes a control block may carry.
///
/// Provenance: `src/script/interpreter.h:285`,
/// `TAPROOT_CONTROL_MAX_NODE_COUNT = 128`.
pub const CONTROL_MAXIMUM_NODES: usize = 128;

/// One node of a taproot tree.
///
/// # Why the tree is stated and not a root
///
/// A fixture that stated only a merkle root would be satisfiable by any
/// tree with that root, and the executor could then materialize a
/// different tree than the one under test without anything noticing.
/// The complete tree is stated, and the executor materializes exactly
/// it `(´[PLAN-rule:guide10:taptree-fixture]´)`.
///
/// # The leaf version is a byte here
///
/// A fixture states a leaf version the way the wire carries it, as a
/// byte, and validation checks it against the reviewed contract. Typing
/// the field as the reviewed version instead would make an unreviewed
/// one unstateable, and a fixture that deliberately states an
/// unreviewed version is exactly how the contract's leaf-version rule
/// gets tested.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum FixtureTapTree {
    /// A leaf carrying one exact script under one exact version.
    Leaf {
        /// The leaf version byte.
        version: u8,
        /// The exact script bytes.
        script: Vec<u8>,
    },
    /// A branch over two subtrees.
    ///
    /// The order stated here is the fixture's own, and it is not the
    /// order the branch hash uses: the target sorts the two child
    /// hashes. Keeping the stated order distinct from the hashed order
    /// is deliberate, so that a fixture written with the children the
    /// other way round produces the same root and is visibly the same
    /// tree.
    Branch {
        /// The first subtree as the fixture states it.
        left: Box<Self>,
        /// The second subtree as the fixture states it.
        right: Box<Self>,
    },
}

/// Why a tree does not determine the values a constructor needs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeDefect {
    /// The tree does not contain the executing leaf.
    ExecutingLeafAbsent,
    /// What was offered as the executing leaf is a branch.
    ///
    /// Distinct from an absent leaf: a branch is not a leaf that is
    /// missing, it is a node a spend cannot execute at all, and
    /// reporting the two the same way would send a reader looking for
    /// the wrong defect.
    ExecutingLeafIsNotALeaf,
    /// The tree contains the executing leaf more than once, so the
    /// control path is not determined.
    ExecutingLeafRepeated,
    /// The path from the root to the executing leaf is longer than a
    /// control block can carry.
    PathTooDeep {
        /// How many nodes the path needs.
        needed: usize,
    },
}

/// Why an internal key and a tree determine no output key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TweakDefect {
    /// The internal key is not the x coordinate of any curve point.
    InternalKeyNotOnCurve(PointDecodingDefect),
    /// The tweak is at or above the group order, so it is not a
    /// multiplier at all.
    ///
    /// Overflow is the whole of the rule, and zero is not part of it.
    /// A zero tweak is a multiplier whose product is the identity and
    /// whose sum is therefore the internal key unchanged, so it has an
    /// output key like any other value below the order. Refusing it
    /// here classified a construction the target accepts as one with
    /// no answer, which is the defect `G13-R07` records; the sum that
    /// really has no encoding is reported by
    /// [`Self::TweakedKeyIsIdentity`] instead, after the addition that
    /// establishes it.
    ///
    /// A hash of public data lands here with negligible probability,
    /// and negligible is not never — which is exactly why the
    /// constructor has to state a policy for it
    /// `(´[PLAN-rule:guide10:tweak-totality]´)`.
    TweakNotAScalar,
    /// The tweaked key is the identity, which has no x-only encoding.
    ///
    /// This is the tweak being the internal key's own negated discrete
    /// logarithm. Nobody can produce it deliberately without solving
    /// for that logarithm, and it is representable here because a
    /// construction that silently returned some other point would be
    /// worse than one that has no answer.
    TweakedKeyIsIdentity,
}

/// The exact target values one constructor instance determines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstructedOutput {
    merkle_root: Digest32,
    tweak: Digest32,
    output_key: [u8; FIELD_ELEMENT_BYTES],
    parity: u8,
    output_program: Vec<u8>,
    executing_leaf_hash: Digest32,
    control_block: Vec<u8>,
}

impl ConstructedOutput {
    /// The tree's merkle root.
    #[must_use]
    pub const fn merkle_root(&self) -> &Digest32 {
        &self.merkle_root
    }

    /// The tweak the internal key and the root determine.
    #[must_use]
    pub const fn tweak(&self) -> &Digest32 {
        &self.tweak
    }

    /// The tweaked output key, x-only.
    #[must_use]
    pub const fn output_key(&self) -> &[u8; FIELD_ELEMENT_BYTES] {
        &self.output_key
    }

    /// The output key's parity bit.
    ///
    /// A witness, not an authority: it is what a control block carries
    /// so that a verifier can recover the point, and a verifier that
    /// took it on trust rather than checking the tweak against it would
    /// accept a spend the target rejects
    /// `(´[PLAN-rule:guide10:parity]´)`.
    #[must_use]
    pub const fn parity(&self) -> u8 {
        self.parity
    }

    /// The output program the target expects to see on the successor
    /// output.
    #[must_use]
    pub fn output_program(&self) -> &[u8] {
        &self.output_program
    }

    /// The executing leaf's hash.
    #[must_use]
    pub const fn executing_leaf_hash(&self) -> &Digest32 {
        &self.executing_leaf_hash
    }

    /// The control block that authenticates the executing leaf.
    #[must_use]
    pub fn control_block(&self) -> &[u8] {
        &self.control_block
    }
}

impl FixtureTapTree {
    /// A leaf under the reviewed tapscript leaf version.
    #[must_use]
    pub const fn leaf(script: Vec<u8>) -> Self {
        Self::Leaf {
            version: LeafVersion::TAPSCRIPT.get(),
            script,
        }
    }

    /// A leaf carrying one exact script under one exact version byte.
    #[must_use]
    pub const fn leaf_of_version(version: u8, script: Vec<u8>) -> Self {
        Self::Leaf { version, script }
    }

    /// A branch over two subtrees.
    #[must_use]
    pub fn branch(left: Self, right: Self) -> Self {
        Self::Branch {
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// This node's hash: a leaf hash, or a branch hash over its
    /// children's.
    #[must_use]
    pub fn node_hash(&self) -> Digest32 {
        match self {
            Self::Leaf { version, script } => leaf_hash_of_version_byte(*version, script),
            Self::Branch { left, right } => branch_hash(&left.node_hash(), &right.node_hash()),
        }
    }

    /// Every leaf hash in the tree, with repetitions.
    #[must_use]
    pub fn leaf_hashes(&self) -> Vec<Digest32> {
        match self {
            Self::Leaf { version, script } => vec![leaf_hash_of_version_byte(*version, script)],
            Self::Branch { left, right } => {
                let mut hashes = left.leaf_hashes();
                hashes.extend(right.leaf_hashes());
                hashes
            }
        }
    }

    /// How many leaves the tree carries.
    #[must_use]
    pub fn leaf_count(&self) -> usize {
        self.leaf_hashes().len()
    }

    /// The sibling hashes between one leaf and the root, deepest first.
    ///
    /// # Errors
    ///
    /// [`TreeDefect`] when the leaf is absent, appears more than once,
    /// or sits deeper than a control block can address.
    pub fn path_to(&self, leaf: &Digest32) -> Result<Vec<Digest32>, TreeDefect> {
        let occurrences = self
            .leaf_hashes()
            .into_iter()
            .filter(|found| found == leaf)
            .count();
        match occurrences {
            0 => return Err(TreeDefect::ExecutingLeafAbsent),
            1 => {}
            _ => return Err(TreeDefect::ExecutingLeafRepeated),
        }

        let path = self.siblings_to(leaf).unwrap_or_default();
        if path.len() > CONTROL_MAXIMUM_NODES {
            return Err(TreeDefect::PathTooDeep { needed: path.len() });
        }
        Ok(path)
    }

    /// The sibling hashes to one leaf, where this subtree contains it.
    ///
    /// Total by construction for the caller above, which has already
    /// established that the leaf occurs exactly once.
    fn siblings_to(&self, leaf: &Digest32) -> Option<Vec<Digest32>> {
        match self {
            Self::Leaf { version, script } => {
                (leaf_hash_of_version_byte(*version, script) == *leaf).then(Vec::new)
            }
            Self::Branch { left, right } => {
                if let Some(mut path) = left.siblings_to(leaf) {
                    path.push(right.node_hash());
                    return Some(path);
                }
                let mut path = right.siblings_to(leaf)?;
                path.push(left.node_hash());
                Some(path)
            }
        }
    }

    /// The distinct leaf hashes in the tree.
    #[must_use]
    pub fn distinct_leaf_hashes(&self) -> BTreeSet<Digest32> {
        self.leaf_hashes().into_iter().collect()
    }
}

/// One leaf's hash.
///
/// The preimage is the leaf version byte, the script's length as a
/// compact size, and the script. Provenance: the target's script-path
/// validation hashes the leaf under `HASHER_TAPLEAF_ELEMENTS`
/// (`src/script/interpreter.cpp:550`).
#[must_use]
pub fn leaf_hash(version: LeafVersion, script: &[u8]) -> Digest32 {
    leaf_hash_of_version_byte(version.get(), script)
}

/// One leaf's hash, under a leaf version this contract has not
/// reviewed.
///
/// # Why an unreviewed version byte is reachable at all
///
/// The reviewed contract admits exactly one leaf version, and a fixture
/// may state only that one. Published target test vectors are not
/// fixtures: they carry whatever leaf version they were written with,
/// and reproducing them is how this oracle is checked against something
/// other than itself. Refusing to hash their version byte would make
/// the strongest available cross-check unavailable.
///
/// The version byte is hashed as given. The target masks the parity bit
/// out of a control block's first byte before hashing it
/// (`src/script/interpreter.cpp:3293`, `TAPROOT_LEAF_MASK = 0xfe`), so
/// a caller passing a byte taken from a control block must mask it
/// first; a caller passing a leaf version passes an even byte already.
#[must_use]
pub fn leaf_hash_of_version_byte(version: u8, script: &[u8]) -> Digest32 {
    let mut preimage = vec![version];
    preimage.extend_from_slice(&compact_size(script.len()));
    preimage.extend_from_slice(script);
    tagged_hash(TAP_LEAF_TAG, &preimage)
}

/// One branch's hash, over its children in canonical order.
///
/// The target sorts the two child hashes as byte strings before hashing
/// them, so a branch has one hash whichever way round its children were
/// built. Reproducing that ordering is load-bearing rather than
/// cosmetic: a constructor that hashed children in caller order would
/// compute a root the target does not agree with, for half of all
/// inputs `(´[PLAN-rule:guide10:tapbranch-order]´)`.
#[must_use]
pub fn branch_hash(left: &Digest32, right: &Digest32) -> Digest32 {
    let (first, second) = if left <= right {
        (left, right)
    } else {
        (right, left)
    };
    let mut preimage = Vec::with_capacity(64);
    preimage.extend_from_slice(first);
    preimage.extend_from_slice(second);
    tagged_hash(TAP_BRANCH_TAG, &preimage)
}

/// The tweak one internal key and one merkle root determine.
///
/// Provenance: `src/pubkey.cpp:246-257`, which hashes the x-only
/// internal key followed by the merkle root under
/// `HASHER_TAPTWEAK_ELEMENTS`.
#[must_use]
pub fn tweak(internal_key: &[u8; FIELD_ELEMENT_BYTES], merkle_root: &Digest32) -> Digest32 {
    let mut preimage = Vec::with_capacity(64);
    preimage.extend_from_slice(internal_key);
    preimage.extend_from_slice(merkle_root);
    tagged_hash(TAP_TWEAK_TAG, &preimage)
}

/// The tweak one internal key alone determines, with no tree.
///
/// The merkle root is *omitted* from the preimage rather than written
/// as zeroes: an output with no scripts hashes only the key
/// (`src/pubkey.cpp:250`). A constructor that hashed 32 zero bytes
/// there would compute a different key for the treeless case and would
/// never discover it, because no tree produces a zero root.
///
/// The constructor prototype never builds such an output — it always
/// has a tree — and this exists because the published target vectors
/// include the case, so the oracle can be checked against it.
#[must_use]
pub fn tweak_without_tree(internal_key: &[u8; FIELD_ELEMENT_BYTES]) -> Digest32 {
    tagged_hash(TAP_TWEAK_TAG, internal_key)
}

/// The output key one internal key and one tweak determine.
///
/// # Errors
///
/// [`TweakDefect`] when the internal key is not on the curve, the tweak
/// is at or above the group order, or the sum is the identity. A tweak
/// of zero is none of those: it is a multiplier, and the addition it
/// leads to is the identity added to the internal key.
pub fn tweaked_key(
    internal_key: &[u8; FIELD_ELEMENT_BYTES],
    tweak: &Digest32,
) -> Result<([u8; FIELD_ELEMENT_BYTES], u8), TweakDefect> {
    let internal = lift_x(internal_key).map_err(TweakDefect::InternalKeyNotOnCurve)?;
    if !is_valid_scalar(tweak) {
        return Err(TweakDefect::TweakNotAScalar);
    }

    let offset = curve::multiply_point(tweak, &curve::generator());
    let sum = curve::add(&CurvePoint::Affine(internal), &offset);
    match sum {
        CurvePoint::Identity => Err(TweakDefect::TweakedKeyIsIdentity),
        CurvePoint::Affine(point) => Ok((point.x_only_bytes(), point.parity_bit())),
    }
}

/// The output program one x-only key determines.
///
/// A witness-version-one program: the version opcode, the payload
/// length, and the key.
#[must_use]
pub fn output_program(output_key: &[u8; FIELD_ELEMENT_BYTES]) -> Vec<u8> {
    let mut program = Vec::with_capacity(2 + FIELD_ELEMENT_BYTES);
    program.push(WITNESS_VERSION_ONE);
    program.push(u8::try_from(FIELD_ELEMENT_BYTES).unwrap_or(u8::MAX));
    program.extend_from_slice(output_key);
    program
}

/// The control block authenticating one leaf under one internal key.
///
/// The first byte carries the leaf version with the output key's parity
/// in its low bit, then the internal key, then the path from the leaf
/// to the root.
#[must_use]
pub fn control_block(
    version: LeafVersion,
    parity: u8,
    internal_key: &[u8; FIELD_ELEMENT_BYTES],
    path: &[Digest32],
) -> Vec<u8> {
    control_block_of_version_byte(version.get(), parity, internal_key, path)
}

/// The control block authenticating one leaf under a leaf version byte
/// this contract has not necessarily reviewed.
#[must_use]
pub fn control_block_of_version_byte(
    version: u8,
    parity: u8,
    internal_key: &[u8; FIELD_ELEMENT_BYTES],
    path: &[Digest32],
) -> Vec<u8> {
    let mut block = Vec::with_capacity(CONTROL_BASE_BYTES + path.len() * FIELD_ELEMENT_BYTES);
    block.push(version | (parity & PARITY_MASK));
    block.extend_from_slice(internal_key);
    for node in path {
        block.extend_from_slice(node);
    }
    block
}

/// Why a constructor instance determines no output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstructionDefect {
    /// The tree does not determine a control path.
    Tree(TreeDefect),
    /// The internal key and tree determine no output key.
    Tweak(TweakDefect),
}

/// Whether a different representation nonce could repair this defect.
///
/// # What a nonce actually moves
///
/// A nonce changes the metadata leaf, and through it the merkle root
/// and the tweak. Everything downstream of the root can therefore come
/// out differently on the next attempt, and everything upstream of it
/// cannot.
///
/// So exactly two defects are worth retrying. The tweak is a hash of
/// the root, so a tweak that overflows the group order is a property
/// of this root and not of the instance; and the tweaked key being the
/// identity depends on the same hash. Both are negligibly rare and
/// both move.
///
/// Zero appears in neither list, because a zero tweak is no longer a
/// defect to classify: it has an output key, so nothing retries it and
/// nothing refuses it `(´[PLAN-rule:guide10:tweak-totality]´)`.
///
/// Nothing else does. Every [`TreeDefect`] is a property of the static
/// subtree and the executing leaf — whether the tree contains that
/// leaf, contains it once, and sits within the control depth — and the
/// nonce does not touch any of them. Neither is
/// [`TweakDefect::InternalKeyNotOnCurve`]: the internal key is an
/// input, decoded before the root is consulted at all, so no number of
/// nonces makes it a curve point.
///
/// # Why it is one function
///
/// Both retry implementations classified this for themselves, and both
/// short-circuited on tree defects only, so an invalid internal key was
/// ground against every nonce in the bound and then reported as
/// exhaustion — a caller reading that diagnostic would raise the retry
/// limit against a failure no limit can repair. One predicate is what
/// keeps the two policies from drifting again
/// `(´[PLAN-rule:guide11-exec:retry-classification]´)`.
#[must_use]
pub const fn retryable(defect: ConstructionDefect) -> bool {
    matches!(
        defect,
        ConstructionDefect::Tweak(TweakDefect::TweakNotAScalar | TweakDefect::TweakedKeyIsIdentity)
    )
}

/// Every exact value one constructor instance determines.
///
/// # Errors
///
/// [`ConstructionDefect`] when the tree does not contain the executing
/// leaf exactly once, or when no output key exists.
pub fn construct(
    internal_key: &[u8; FIELD_ELEMENT_BYTES],
    tree: &FixtureTapTree,
    executing_leaf: &FixtureTapTree,
) -> Result<ConstructedOutput, ConstructionDefect> {
    let FixtureTapTree::Leaf { version, script } = executing_leaf else {
        return Err(ConstructionDefect::Tree(
            TreeDefect::ExecutingLeafIsNotALeaf,
        ));
    };
    let executing_leaf_hash = leaf_hash_of_version_byte(*version, script);
    let path = tree
        .path_to(&executing_leaf_hash)
        .map_err(ConstructionDefect::Tree)?;

    let merkle_root = tree.node_hash();
    let tweak = tweak(internal_key, &merkle_root);
    let (output_key, parity) =
        tweaked_key(internal_key, &tweak).map_err(ConstructionDefect::Tweak)?;
    let control_block = control_block_of_version_byte(*version, parity, internal_key, &path);

    Ok(ConstructedOutput {
        merkle_root,
        tweak,
        output_program: output_program(&output_key),
        control_block,
        output_key,
        parity,
        executing_leaf_hash,
    })
}
