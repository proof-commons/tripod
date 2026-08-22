//! The live-receipt output instance: tree, tweak, and program (§7.5,
//! §12.6).
//!
//! # Why this crate hashes the tree and does not tweak the key
//!
//! [`crate::taproot`] states the boundary for the compact-ASH pin: the
//! merkle root is a hash of public data and belongs here, and the tweak
//! that turns it into an output key is curve arithmetic this crate does
//! not have and must not grow — the workspace's only implementation of
//! it belongs to the independent host oracle, and a builder that
//! borrowed it would be producing one opinion and counting it twice.
//!
//! The live transfer cannot simply pin its way past that, and the
//! difference is worth stating. A compact-ASH spend consumes an output
//! that already exists on chain, so its program is an observed public
//! fact. A live transfer *creates* receipts, and §12.3 refuses a request
//! that names constructor bytes or a target program — so the destination
//! program has to be derived, from the linked constructor the
//! destination owner selects, and deriving it needs the tweak.
//!
//! # So the tweak arrives as a capability
//!
//! [`LiveCurveCapability`] is an adapter in the same sense
//! [`crate::sponsor::SponsorCapability`] is: the caller supplies it, no
//! secret crosses the boundary, and a `None` answer is an ordinary
//! outcome rather than a panic. What the capability supplies is public
//! point arithmetic over public inputs — an x-only internal key, a
//! merkle root, an offered owner key — and there is no method on it that
//! could carry a scalar.
//!
//! That is what lets this crate discharge the link's
//! `TaprootOutputKeyUndischarged` and `OwnerKeyCurvePointMembershipUnverified`
//! rather than carry them forward again: the arithmetic is performed,
//! and it is performed by the package that owns it.

use std::collections::{BTreeMap, BTreeSet};

use linker::LinkedLiveConstructor;
use linker::live_backend::LiveTransferLeafRole;
use target_elements::{LeafVersion, ReviewedElementsTapscriptDefinition};

use crate::error::TransactionRefusal;
use crate::taproot::{
    CONTROL_BASE_BYTES, DIGEST_BYTES, Digest32, OutputKeyParity, TAPROOT_LEAF_MASK,
    TAPROOT_PROGRAM_BYTES, TAPROOT_WITNESS_VERSION, leaf_hash, subtree_hash,
    witness_program_script,
};

/// One taproot output key, with the parity a control block carries.
///
/// Both halves, because a control block that stated the wrong parity
/// would authenticate a leaf of a tree committed under the other key,
/// and returning the key alone would make the parity something a caller
/// had to remember to ask for separately.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TweakedOutputKey {
    key: [u8; TAPROOT_PROGRAM_BYTES],
    parity: OutputKeyParity,
}

impl TweakedOutputKey {
    /// The key and parity one tweak determined.
    #[must_use]
    pub const fn new(key: [u8; TAPROOT_PROGRAM_BYTES], parity: OutputKeyParity) -> Self {
        Self { key, parity }
    }

    /// The x-only output key.
    #[must_use]
    pub const fn key(&self) -> &[u8; TAPROOT_PROGRAM_BYTES] {
        &self.key
    }

    /// The output key's parity.
    #[must_use]
    pub const fn parity(&self) -> OutputKeyParity {
        self.parity
    }
}

/// The public point arithmetic a live construction needs.
///
/// Two questions, both about public data, and neither of them one this
/// crate is entitled to answer for itself. An implementation belongs to
/// whichever package owns the target's curve; the evidence layer wires
/// one in, and this crate's own tests supply a fixture one that says so.
///
/// `None` from either method is an ordinary outcome — an offered key
/// that is not a point, or a tweak with no output key — and construction
/// refuses rather than proceeding on a value it did not get.
pub trait LiveCurveCapability {
    /// Whether an offered owner key names a point of the target's curve.
    ///
    /// §1.8's second conjunct. The approved *encoding* is checked where
    /// the owner metadata is built; this is the question that check
    /// explicitly cannot answer, and an owner key that fails it never
    /// enters a signing request.
    fn owner_key_is_a_curve_point(&self, owner: &[u8]) -> bool;

    /// The output key one internal key and one merkle root determine.
    ///
    /// The taproot tweak: the internal key offset by the generator
    /// multiple of a tagged hash over the key and the root.
    fn output_key(&self, internal_key: &[u8], merkle_root: &Digest32) -> Option<TweakedOutputKey>;
}

/// One linked live constructor's committed tree, hashed.
///
/// The live counterpart of [`crate::taproot::CommittedTree`], keyed by
/// the live leaf roles, and built from the same structural recipes by
/// the same recursion. What differs is only which leaves hang off the
/// tree; a second copy of the hashing would have been a second chance to
/// order a branch's children wrongly in exactly one of them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommittedLiveTree {
    leaf_version: LeafVersion,
    programs: BTreeMap<LiveTransferLeafRole, Vec<u8>>,
    leaves: BTreeMap<LiveTransferLeafRole, Digest32>,
    paths: BTreeMap<LiveTransferLeafRole, Vec<Digest32>>,
    root: Digest32,
}

impl CommittedLiveTree {
    /// The leaf version every committed leaf carries.
    #[must_use]
    pub const fn leaf_version(&self) -> LeafVersion {
        self.leaf_version
    }

    /// Every committed leaf's exact program bytes, in canonical order.
    ///
    /// The same bytes the leaf hashes were taken over, kept rather than
    /// re-encoded, so the script a witness carries and the script the
    /// tree commits to cannot come to differ.
    #[must_use]
    pub const fn leaf_programs(&self) -> &BTreeMap<LiveTransferLeafRole, Vec<u8>> {
        &self.programs
    }

    /// Every leaf's tapleaf hash, in canonical order.
    #[must_use]
    pub const fn leaf_hashes(&self) -> &BTreeMap<LiveTransferLeafRole, Digest32> {
        &self.leaves
    }

    /// Every leaf's merkle path, from the leaf upward.
    #[must_use]
    pub const fn paths(&self) -> &BTreeMap<LiveTransferLeafRole, Vec<Digest32>> {
        &self.paths
    }

    /// The tree's merkle root.
    #[must_use]
    pub const fn merkle_root(&self) -> Digest32 {
        self.root
    }
}

/// One owner's live-receipt instance under one representation.
///
/// Everything a transfer needs about the object it creates and the
/// objects it consumes: the tree that authenticates a spend, the key
/// that commits to it, and the program an output pays to. Derived, never
/// supplied — §12.3 refuses a request that names any of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveReceiptInstance {
    tree: CommittedLiveTree,
    internal_key: Vec<u8>,
    output_key: TweakedOutputKey,
    program: Vec<u8>,
}

impl LiveReceiptInstance {
    /// The committed tree.
    #[must_use]
    pub const fn tree(&self) -> &CommittedLiveTree {
        &self.tree
    }

    /// The unspendable internal key the constructor inherited.
    #[must_use]
    pub fn internal_key(&self) -> &[u8] {
        &self.internal_key
    }

    /// The tweaked output key and its parity.
    #[must_use]
    pub const fn output_key(&self) -> &TweakedOutputKey {
        &self.output_key
    }

    /// The witness-version-one program an output carrying this receipt
    /// pays to.
    #[must_use]
    pub fn program(&self) -> &[u8] {
        &self.program
    }

    /// One leaf's control block.
    ///
    /// The version-and-parity byte, the x-only internal key, and the
    /// merkle path — the layout a script-path spend supplies.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::MissingLiveLeaf`] for a leaf this tree does
    /// not commit to.
    pub fn control_block(&self, leaf: LiveTransferLeafRole) -> Result<Vec<u8>, TransactionRefusal> {
        let path = self
            .tree
            .paths
            .get(&leaf)
            .ok_or(TransactionRefusal::MissingLiveLeaf(leaf))?;
        let mut block =
            Vec::with_capacity(CONTROL_BASE_BYTES + path.len().saturating_mul(DIGEST_BYTES));
        block.push(
            (self.tree.leaf_version.get() & TAPROOT_LEAF_MASK) | self.output_key.parity.bit(),
        );
        block.extend_from_slice(&self.internal_key);
        for node in path {
            block.extend_from_slice(node);
        }
        Ok(block)
    }
}

/// Hash one linked live constructor's committed tree.
///
/// # Errors
///
/// [`TransactionRefusal::MissingLiveLeaf`] when the tree commits to a
/// leaf the constructor linked no program for, or when a recipe's
/// sibling sets do not reconstruct a tree; and
/// [`TransactionRefusal::LiveBundleIsNotACandidate`] for a constructor
/// with no leaves at all. Both are structurally unreachable from a tree
/// this workspace's linker assembled, and are typed rather than asserted
/// because the recipes cross a package boundary.
pub fn commit_live_tree(
    target: &ReviewedElementsTapscriptDefinition,
    constructor: &LinkedLiveConstructor,
) -> Result<CommittedLiveTree, TransactionRefusal> {
    let leaf_version = constructor.taptree().leaf_version();
    let recipes = constructor.taptree().recipes();

    let mut programs = BTreeMap::new();
    let mut leaves = BTreeMap::new();
    for leaf in recipes.keys() {
        let linked = constructor
            .program(*leaf)
            .ok_or(TransactionRefusal::MissingLiveLeaf(*leaf))?;
        let script = linked.program().encode(target);
        leaves.insert(*leaf, leaf_hash(leaf_version, &script));
        programs.insert(*leaf, script);
    }

    let mut hashes: BTreeMap<BTreeSet<LiveTransferLeafRole>, Digest32> = BTreeMap::new();
    for (leaf, hash) in &leaves {
        hashes.insert(BTreeSet::from([*leaf]), *hash);
    }

    let mut paths = BTreeMap::new();
    let mut root = *leaves
        .values()
        .next()
        .ok_or(TransactionRefusal::LiveBundleIsNotACandidate)?;
    for (leaf, recipe) in recipes {
        let mut path = Vec::with_capacity(recipe.siblings().len());
        let mut node = BTreeSet::from([*leaf]);
        for sibling in recipe.siblings() {
            path.push(hash_live_subtree(sibling, recipes, &leaves, &mut hashes)?);
            node.extend(sibling.iter().copied());
        }
        root = hash_live_subtree(&node, recipes, &leaves, &mut hashes)?;
        paths.insert(*leaf, path);
    }

    Ok(CommittedLiveTree {
        leaf_version,
        programs,
        leaves,
        paths,
        root,
    })
}

/// Derive one owner's live-receipt instance.
///
/// The tree is hashed here; the tweak comes from the capability, and the
/// program is assembled from the key the capability returned. Nothing
/// about the result is a claim that a target would accept a spend of it
/// — that is a run, and §1.11 admits no verdict without one.
///
/// # Errors
///
/// Any failure of [`commit_live_tree`];
/// [`TransactionRefusal::LiveInternalKeyMalformed`] when the linked
/// internal key is not the reviewed x-only width; and
/// [`TransactionRefusal::DestinationOutputKeyUndetermined`] when the
/// capability determines no output key for the tree.
pub fn derive_live_receipt_instance(
    target: &ReviewedElementsTapscriptDefinition,
    constructor: &LinkedLiveConstructor,
    curve: &dyn LiveCurveCapability,
) -> Result<LiveReceiptInstance, TransactionRefusal> {
    let tree = commit_live_tree(target, constructor)?;
    let internal_key = constructor.internal_key().bytes().to_vec();
    if internal_key.len() != TAPROOT_PROGRAM_BYTES {
        return Err(TransactionRefusal::LiveInternalKeyMalformed {
            offered: internal_key.len(),
        });
    }

    let output_key = curve
        .output_key(&internal_key, &tree.root)
        .ok_or(TransactionRefusal::DestinationOutputKeyUndetermined)?;
    let program = witness_program_script(target, TAPROOT_WITNESS_VERSION, output_key.key())?;

    Ok(LiveReceiptInstance {
        tree,
        internal_key,
        output_key,
        program,
    })
}

/// One live subtree's hash, under this crate's own refusal.
fn hash_live_subtree(
    node: &BTreeSet<LiveTransferLeafRole>,
    recipes: &BTreeMap<LiveTransferLeafRole, linker::ControlPathRecipe<LiveTransferLeafRole>>,
    leaves: &BTreeMap<LiveTransferLeafRole, Digest32>,
    hashes: &mut BTreeMap<BTreeSet<LiveTransferLeafRole>, Digest32>,
) -> Result<Digest32, TransactionRefusal> {
    subtree_hash(node, recipes, leaves, hashes).map_err(|leaf| {
        leaf.map_or(
            TransactionRefusal::LiveBundleIsNotACandidate,
            TransactionRefusal::MissingLiveLeaf,
        )
    })
}
