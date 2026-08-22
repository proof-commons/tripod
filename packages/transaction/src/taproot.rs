//! Committed-tree hashing, control blocks, and the pinned instance.
//!
//! # What this discharges, and what it does not
//!
//! The linker left `TaprootOutputKeyUndischarged`: it computed neither
//! the merkle root, nor the tweak, nor the output key. This module
//! discharges the first of those three and deliberately not the other
//! two.
//!
//! The merkle root is discharged because the witness needs it. A
//! control block carries the path from the leaf to the root, each
//! element a tagged hash of the linked programs, and a crate that could
//! not compute those could state a witness layout but not a witness.
//! Hashing is all it takes, and this workspace's digest is pure Rust.
//!
//! The tweak and the output key are *not* discharged, and the reason is
//! not that the arithmetic is hard. The object a compact-ASH
//! transaction spends already exists: the ASH inputs are target outputs
//! that were created before this crate ran, by a burn in production or
//! by the test funding ceremony in development. Their program is an
//! observed public fact, and §15.6 hands the permissionless constructor
//! "exact target asset and program data" rather than asking it to
//! derive one. A builder that recomputed the output key would not be
//! obtaining the program — it would be *checking* the program it was
//! given, and that check needs curve arithmetic this crate does not
//! own and must not borrow from the independent oracle it is checked
//! against.
//!
//! So the program is pinned, the pin's own status is a typed
//! obligation, and the one bit the pin cannot be derived from — the
//! parity of the output key, which a control block carries and an
//! x-only program does not state — is pinned alongside it.
//!
//! # The tags are the target's, not Bitcoin's
//!
//! The reviewed target's tag strings differ from the upstream Bitcoin
//! ones by an `/elements` suffix. A tree built on the Bitcoin tags
//! would produce well-formed control blocks that commit to nothing the
//! target recognizes, and no amount of internal consistency would
//! reveal it.

use std::collections::{BTreeMap, BTreeSet};

use linker::backend::{LeafRole, StackItem};
use linker::{CandidateLinkedBundle, ControlPathRecipe, TreeLeaf};
use sha2::{Digest, Sha256};
use target_elements::{
    LeafVersion, PushForm, PushOpcodeMapping, ReviewedElementsTapscriptDefinition,
};

use crate::bytes::compact_size;
use crate::error::TransactionRefusal;

/// How many bytes one digest occupies.
pub const DIGEST_BYTES: usize = 32;

/// One 32-byte digest.
pub type Digest32 = [u8; DIGEST_BYTES];

/// The tag the target hashes a leaf under.
///
/// Provenance: `HASHER_TAPLEAF_ELEMENTS = TaggedHash("TapLeaf/elements")`
/// in `src/script/interpreter.cpp`, consumed by `ComputeTapleafHash`.
pub const TAP_LEAF_TAG: &str = "TapLeaf/elements";

/// The tag the target hashes a branch under.
///
/// Provenance: `HASHER_TAPBRANCH_ELEMENTS =
/// TaggedHash("TapBranch/elements")` in the same file, consumed by
/// `ComputeTapbranchHash`.
pub const TAP_BRANCH_TAG: &str = "TapBranch/elements";

/// The bits of a control block's first byte that carry the leaf
/// version.
///
/// Provenance: `TAPROOT_LEAF_MASK` in `src/script/interpreter.cpp`. The
/// bit the mask clears is the output key's parity, which is why the two
/// live in one byte.
pub const TAPROOT_LEAF_MASK: u8 = 0xfe;

/// How many bytes a control block carries before its path.
///
/// Provenance: `TAPROOT_CONTROL_BASE_SIZE`: the version-and-parity byte
/// plus the x-only internal key.
pub const CONTROL_BASE_BYTES: usize = 33;

/// The witness version a taproot output program is read at.
pub const TAPROOT_WITNESS_VERSION: u8 = 1;

/// How many bytes a taproot witness program occupies.
pub const TAPROOT_PROGRAM_BYTES: usize = 32;

/// The tagged hash of `message` under `tag`.
///
/// The published construction: the tag's own digest twice, then the
/// message. The fixed-width prefix pair is what makes the domain
/// separation independent of the tag's length.
#[must_use]
pub fn tagged_hash(tag: &str, message: &[u8]) -> Digest32 {
    let prefix = Sha256::digest(tag.as_bytes());
    let mut hasher = Sha256::new();
    hasher.update(prefix);
    hasher.update(prefix);
    hasher.update(message);
    hasher.finalize().into()
}

/// The hash of one committed leaf.
///
/// Provenance: `ComputeTapleafHash`, whose preimage is the leaf version
/// byte, the script's compact-size length, and the script.
#[must_use]
pub fn leaf_hash(leaf_version: LeafVersion, script: &[u8]) -> Digest32 {
    let mut message = vec![leaf_version.get()];
    message.extend_from_slice(&compact_size(script.len() as u64));
    message.extend_from_slice(script);
    tagged_hash(TAP_LEAF_TAG, &message)
}

/// The hash of one branch over two child hashes.
///
/// Provenance: `ComputeTapbranchHash`, which orders the two children
/// lexicographically before hashing. The ordering is what lets a
/// control block carry a bare path with no side bits.
#[must_use]
pub fn branch_hash(left: Digest32, right: Digest32) -> Digest32 {
    let mut message = Vec::with_capacity(2 * DIGEST_BYTES);
    if left < right {
        message.extend_from_slice(&left);
        message.extend_from_slice(&right);
    } else {
        message.extend_from_slice(&right);
        message.extend_from_slice(&left);
    }
    tagged_hash(TAP_BRANCH_TAG, &message)
}

/// The parity of an output key's implicit y coordinate.
///
/// A witness program states a point by its x coordinate alone, so a
/// spend has to say which of the two y coordinates the key was tweaked
/// to. This is that bit, and it is the one fact about a deployed
/// instance that cannot be read off its program.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OutputKeyParity {
    /// The output key's y coordinate is even.
    Even,
    /// The output key's y coordinate is odd.
    Odd,
}

impl OutputKeyParity {
    /// The bit this parity contributes to a control block.
    #[must_use]
    pub const fn bit(self) -> u8 {
        match self {
            Self::Even => 0,
            Self::Odd => 1,
        }
    }
}

/// Where a pinned instance's program came from.
///
/// Recorded because the two origins carry different weight and a report
/// that could not tell them apart would let a development fixture read
/// as a production object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AshInstanceOrigin {
    /// The instance was funded by the test-only ceremony, standing for
    /// a burn that is not implemented.
    SyntheticTestFunding,
    /// The instance was observed as an existing target output.
    ObservedTargetOutput,
}

/// The deployed ASH instance an ABI builds against.
///
/// # Why a pin rather than a derivation
///
/// The fix wave established that identity introspection gives the
/// programs *sameness* — a coordinator can require that its successor
/// carries the same program as the input it spends — and not named
/// identity. Sameness does not say *which* program, so someone has to,
/// and the ABI's consumers would otherwise each answer differently.
/// This type is the single place the answer lives.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PinnedAshInstance {
    program: [u8; TAPROOT_PROGRAM_BYTES],
    parity: OutputKeyParity,
    leaf_version: LeafVersion,
    internal_key: Vec<u8>,
    origin: AshInstanceOrigin,
}

impl PinnedAshInstance {
    /// Pin the instance these public facts describe.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::MalformedPinnedProgram`] when the witness
    /// program is not the reviewed taproot width.
    pub fn new(
        program: &[u8],
        parity: OutputKeyParity,
        leaf_version: LeafVersion,
        internal_key: Vec<u8>,
        origin: AshInstanceOrigin,
    ) -> Result<Self, TransactionRefusal> {
        let program = <[u8; TAPROOT_PROGRAM_BYTES]>::try_from(program).map_err(|_| {
            TransactionRefusal::MalformedPinnedProgram {
                offered: program.len(),
            }
        })?;
        Ok(Self {
            program,
            parity,
            leaf_version,
            internal_key,
            origin,
        })
    }

    /// The pinned witness program's payload.
    #[must_use]
    pub const fn program(&self) -> &[u8; TAPROOT_PROGRAM_BYTES] {
        &self.program
    }

    /// The pinned output key's parity.
    #[must_use]
    pub const fn parity(&self) -> OutputKeyParity {
        self.parity
    }

    /// The leaf version the pinned instance commits leaves under.
    #[must_use]
    pub const fn leaf_version(&self) -> LeafVersion {
        self.leaf_version
    }

    /// The internal key the pinned instance was tweaked from.
    #[must_use]
    pub fn internal_key(&self) -> &[u8] {
        &self.internal_key
    }

    /// Where the pinned instance came from.
    #[must_use]
    pub const fn origin(&self) -> AshInstanceOrigin {
        self.origin
    }

    /// Check the pin against the bundle that will build against it.
    ///
    /// The two facts a pin can be checked against without curve
    /// arithmetic are checked: the internal key it names and the leaf
    /// version it commits under must be the ones the link resolved. The
    /// third fact — that the program *is* the tweak of that key by this
    /// tree's root — is exactly what is not checked, and is the whole
    /// content of the outstanding obligation.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::PinnedInternalKeyMismatch`] and
    /// [`TransactionRefusal::PinnedLeafVersionMismatch`].
    pub fn check_against(&self, bundle: &CandidateLinkedBundle) -> Result<(), TransactionRefusal> {
        let linked: &StackItem = bundle.constructor().internal_key();
        if linked.bytes() != self.internal_key.as_slice() {
            return Err(TransactionRefusal::PinnedInternalKeyMismatch);
        }
        let linked_version = bundle.constructor().leaf_version();
        if linked_version != self.leaf_version {
            return Err(TransactionRefusal::PinnedLeafVersionMismatch {
                pinned: self.leaf_version.get(),
                linked: linked_version.get(),
            });
        }
        Ok(())
    }

    /// The exact output script this instance is spent from and paid to.
    ///
    /// Both roles at once, and deliberately one function: compact ASH
    /// consolidates a family of ASH inputs into one successor of the
    /// same constructor instance, so the successor's program is the
    /// inputs' program. A separate successor builder would be a second
    /// place for the two to disagree.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::MalformedDeploymentSymbol`] when the
    /// reviewed push contract does not carry the two forms a witness
    /// program's script is built from. Unreachable against the reviewed
    /// target, and typed rather than asserted because the contract is
    /// data this crate reads rather than a constant it restates.
    pub fn output_script(
        &self,
        target: &ReviewedElementsTapscriptDefinition,
    ) -> Result<Vec<u8>, TransactionRefusal> {
        witness_program_script(target, TAPROOT_WITNESS_VERSION, &self.program)
    }
}

/// The script of a witness program of `version` over `payload`.
///
/// Every byte is resolved from the reviewed push contract rather than
/// transcribed: the payload's length prefix comes from the direct
/// form, whose opcode byte *is* the width, and the version opcode from
/// whichever form actually carries that version.
///
/// Version zero and the rest come from *different* forms, and getting
/// that wrong is the classic way to build an unspendable script. Zero
/// is the empty push, whose opcode is not one less than the opcode for
/// one; every other version is the small-number form, whose opcode is
/// its value plus a fixed offset. Deriving zero from the small-number
/// offset would produce a script that pushes the number zero rather
/// than the empty item, and it would be well formed.
///
/// # Errors
///
/// [`TransactionRefusal::MalformedDeploymentSymbol`] when a form is
/// absent from the contract or does not carry the mapping its kind
/// requires.
pub fn witness_program_script(
    target: &ReviewedElementsTapscriptDefinition,
    version: u8,
    payload: &[u8],
) -> Result<Vec<u8>, TransactionRefusal> {
    let pushes = target.definition().pushes();
    let version_opcode = if version == 0 {
        let empty =
            pushes
                .form(PushForm::Empty)
                .ok_or(TransactionRefusal::MalformedDeploymentSymbol {
                    symbol: "push form: empty",
                })?;
        if !matches!(empty.mapping(), PushOpcodeMapping::LiteralPayload(payload) if payload.is_empty())
        {
            return Err(TransactionRefusal::MalformedDeploymentSymbol {
                symbol: "push form: empty",
            });
        }
        empty.first_opcode()
    } else {
        let small = pushes.form(PushForm::SmallNumber).ok_or(
            TransactionRefusal::MalformedDeploymentSymbol {
                symbol: "push form: small number",
            },
        )?;
        let PushOpcodeMapping::NumericPayload { offset } = small.mapping() else {
            return Err(TransactionRefusal::MalformedDeploymentSymbol {
                symbol: "push form: small number",
            });
        };
        let opcode = offset.saturating_add(version);
        if !small.occupies(opcode) {
            return Err(TransactionRefusal::MalformedDeploymentSymbol {
                symbol: "push form: small number",
            });
        }
        opcode
    };
    let direct =
        pushes
            .form(PushForm::Direct)
            .ok_or(TransactionRefusal::MalformedDeploymentSymbol {
                symbol: "push form: direct",
            })?;
    if !matches!(direct.mapping(), PushOpcodeMapping::WidthInOpcode) {
        return Err(TransactionRefusal::MalformedDeploymentSymbol {
            symbol: "push form: direct",
        });
    }
    let width =
        u8::try_from(payload.len()).map_err(|_| TransactionRefusal::MalformedPinnedProgram {
            offered: payload.len(),
        })?;
    if !direct.occupies(width) {
        return Err(TransactionRefusal::MalformedPinnedProgram {
            offered: payload.len(),
        });
    }

    let mut script = Vec::with_capacity(2 + payload.len());
    script.push(version_opcode);
    script.push(width);
    script.extend_from_slice(payload);
    Ok(script)
}

/// The committed tree, hashed.
///
/// Built from the linker's structural recipes and the linked program
/// bytes. The linker names each sibling by the set of leaves beneath
/// it, which is enough to rebuild the tree exactly — a node's children
/// are read off the path of any leaf it contains — and is not enough to
/// hash one, which is why the hashing lives here rather than there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommittedTree {
    leaf_version: LeafVersion,
    programs: BTreeMap<LeafRole, Vec<u8>>,
    leaves: BTreeMap<LeafRole, Digest32>,
    paths: BTreeMap<LeafRole, Vec<Digest32>>,
    root: Digest32,
}

impl CommittedTree {
    /// The leaf version every committed leaf carries.
    #[must_use]
    pub const fn leaf_version(&self) -> LeafVersion {
        self.leaf_version
    }

    /// Every committed leaf's exact program bytes, in canonical order.
    ///
    /// The same bytes the leaf hashes were taken over, kept rather than
    /// re-encoded. A witness carries the leaf program, and re-encoding
    /// it at assembly time would be a second opportunity for the script
    /// in the witness and the script the tree commits to to differ.
    #[must_use]
    pub const fn leaf_programs(&self) -> &BTreeMap<LeafRole, Vec<u8>> {
        &self.programs
    }

    /// Every leaf's tapleaf hash, in canonical order.
    #[must_use]
    pub const fn leaf_hashes(&self) -> &BTreeMap<LeafRole, Digest32> {
        &self.leaves
    }

    /// Every leaf's merkle path, from the leaf upward.
    #[must_use]
    pub const fn paths(&self) -> &BTreeMap<LeafRole, Vec<Digest32>> {
        &self.paths
    }

    /// The tree's merkle root.
    ///
    /// The half of `TaprootOutputKeyUndischarged` this crate settles.
    /// The tweak that would turn it into an output key is the half it
    /// does not.
    #[must_use]
    pub const fn merkle_root(&self) -> Digest32 {
        self.root
    }

    /// One leaf's control block, against a pinned instance.
    ///
    /// Provenance: the control block a script-path spend supplies is
    /// the version-and-parity byte, the x-only internal key, and the
    /// merkle path — the layout `VerifyTaprootCommitment` reads.
    ///
    /// # Errors
    ///
    /// [`TransactionRefusal::MissingLeaf`] for a leaf this tree does
    /// not commit to.
    pub fn control_block(
        &self,
        leaf: LeafRole,
        pin: &PinnedAshInstance,
    ) -> Result<Vec<u8>, TransactionRefusal> {
        let path = self
            .paths
            .get(&leaf)
            .ok_or(TransactionRefusal::MissingLeaf(leaf))?;
        let mut block =
            Vec::with_capacity(CONTROL_BASE_BYTES + path.len().saturating_mul(DIGEST_BYTES));
        block.push((self.leaf_version.get() & TAPROOT_LEAF_MASK) | pin.parity().bit());
        block.extend_from_slice(pin.internal_key());
        for node in path {
            block.extend_from_slice(node);
        }
        Ok(block)
    }
}

/// Hash the committed tree of one linked bundle.
///
/// # Errors
///
/// [`TransactionRefusal::MissingLeaf`] when the tree commits to a leaf
/// the bundle linked no program for, or when a recipe's sibling sets do
/// not reconstruct a tree — the second is structurally unreachable from
/// a tree this workspace's linker assembled, and is typed rather than
/// asserted because the recipes cross a package boundary.
pub fn commit_tree(
    target: &ReviewedElementsTapscriptDefinition,
    bundle: &CandidateLinkedBundle,
) -> Result<CommittedTree, TransactionRefusal> {
    let leaf_version = bundle.taptree().leaf_version();
    let recipes = bundle.taptree().recipes();

    let mut programs = BTreeMap::new();
    let mut leaves = BTreeMap::new();
    for leaf in recipes.keys() {
        let linked = bundle
            .program(*leaf)
            .ok_or(TransactionRefusal::MissingLeaf(*leaf))?;
        let script = linked.program().encode(target);
        leaves.insert(*leaf, leaf_hash(leaf_version, &script));
        programs.insert(*leaf, script);
    }

    let mut hashes: BTreeMap<BTreeSet<LeafRole>, Digest32> = BTreeMap::new();
    for (leaf, hash) in &leaves {
        hashes.insert(BTreeSet::from([*leaf]), *hash);
    }

    let mut paths = BTreeMap::new();
    let mut root = *leaves
        .values()
        .next()
        .ok_or(TransactionRefusal::BundleIsNotACandidate)?;
    for (leaf, recipe) in recipes {
        let mut path = Vec::with_capacity(recipe.siblings().len());
        let mut node = BTreeSet::from([*leaf]);
        for sibling in recipe.siblings() {
            path.push(hash_subtree(sibling, recipes, &leaves, &mut hashes)?);
            node.extend(sibling.iter().copied());
        }
        root = hash_subtree(&node, recipes, &leaves, &mut hashes)?;
        paths.insert(*leaf, path);
    }

    Ok(CommittedTree {
        leaf_version,
        programs,
        leaves,
        paths,
        root,
    })
}

/// One compact-ASH subtree's hash, under this crate's own refusal.
///
/// The generic recursion below, with the leaf it could not account for
/// named as a [`TransactionRefusal::MissingLeaf`]. An empty node is
/// impossible from a linked tree and is reported as a bundle that is not
/// a candidate rather than asserted away.
fn hash_subtree(
    node: &BTreeSet<LeafRole>,
    recipes: &BTreeMap<LeafRole, ControlPathRecipe>,
    leaves: &BTreeMap<LeafRole, Digest32>,
    hashes: &mut BTreeMap<BTreeSet<LeafRole>, Digest32>,
) -> Result<Digest32, TransactionRefusal> {
    subtree_hash(node, recipes, leaves, hashes).map_err(|leaf| {
        leaf.map_or(
            TransactionRefusal::BundleIsNotACandidate,
            TransactionRefusal::MissingLeaf,
        )
    })
}

/// The hash of the subtree whose leaves are exactly `node`, or the leaf
/// the recipes could not account for.
///
/// A node's children are recovered from any leaf beneath it: that
/// leaf's path climbs through nodes of strictly growing leaf sets, and
/// the step that first reaches `node` names the two children.
///
/// Generic over the leaf role, and shared with the live-transfer tree
/// (§11.4, §12.6) rather than transcribed for it. The tree structure is
/// the same structure whichever operation's leaves hang off it, and a
/// second copy of this recursion would be a second chance to get the
/// sibling ordering wrong in exactly one of them.
///
/// The error carries the offending leaf and not a refusal, because the
/// two callers name that leaf under different variants of
/// [`TransactionRefusal`] — which is the only thing they disagree about.
pub(crate) fn subtree_hash<L: TreeLeaf>(
    node: &BTreeSet<L>,
    recipes: &BTreeMap<L, ControlPathRecipe<L>>,
    leaves: &BTreeMap<L, Digest32>,
    hashes: &mut BTreeMap<BTreeSet<L>, Digest32>,
) -> Result<Digest32, Option<L>> {
    if let Some(hash) = hashes.get(node) {
        return Ok(*hash);
    }
    let member = *node.iter().next().ok_or(None)?;
    if node.len() == 1 {
        let hash = *leaves.get(&member).ok_or(Some(member))?;
        hashes.insert(node.clone(), hash);
        return Ok(hash);
    }

    let recipe = recipes.get(&member).ok_or(Some(member))?;
    let mut below = BTreeSet::from([member]);
    for sibling in recipe.siblings() {
        let mut above = below.clone();
        above.extend(sibling.iter().copied());
        if &above == node {
            let hash = branch_hash(
                subtree_hash(&below, recipes, leaves, hashes)?,
                subtree_hash(sibling, recipes, leaves, hashes)?,
            );
            hashes.insert(node.clone(), hash);
            return Ok(hash);
        }
        below = above;
    }

    Err(Some(member))
}
