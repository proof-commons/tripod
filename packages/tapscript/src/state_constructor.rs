//! Candidate STATE construction from canonical metadata and an exact static tree.
//!
//! Curve arithmetic is supplied by the caller. Host nonce leastness is evidence,
//! not a target observation; native metadata-leaf evidence remains outstanding.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use realization::{
    EncodedStateMetadata, STATE_METADATA_SCHEMA, StateField, StateMetadata,
    StateRepresentationNonce, TransactionSide, decode_state_metadata, encode_state_metadata,
};
use sha2::{Digest, Sha256};
use target_elements::{
    LeafVersion, OpcodeId, ResourceDimension, ReviewedElementsTapscriptDefinition,
    TargetContractVersion,
};
use thiserror::Error;

use crate::{
    AbstractExecutionResult, AbstractLimits, AbstractStackState, InternalKeyPolicy, KeyPathPolicy,
    ProgramStackProfile, StackItem, TapscriptInstruction, TapscriptProgram, program_stack_profile,
    resource_projection, validate_program,
};

/// Public curve arithmetic supplied by a downstream implementation.
pub trait StateCurveCapability {
    /// Validate a point; this establishes nothing about knowledge of its scalar.
    fn internal_key_is_a_point(&self, x_only: &[u8; 32]) -> bool;
    /// Compute the Elements tagged tweak and point sum, admitting a zero tweak.
    fn output_key(&self, internal_key: &[u8; 32], merkle_root: &[u8; 32]) -> StateTweakOutcome;
}

/// The complete result of the public tweak operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateTweakOutcome {
    /// A valid output point, with oddness of its y coordinate.
    OutputKey { key: [u8; 32], parity: bool },
    /// The tweak cannot be read as a scalar without reduction.
    TweakAboveGroupOrder,
    /// The point sum has no x-only encoding.
    TweakedPointIsIdentity,
    /// The internal key cannot be lifted.
    InternalKeyNotAPoint,
}

/// A closed construction refusal.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum StateConstructorRefusal {
    /// Construction refused: metadata-encoding-refused.
    #[error("metadata-encoding-refused")]
    MetadataEncodingRefused,
    /// Construction refused: metadata-leaf-not-unspendable.
    #[error("metadata-leaf-not-unspendable")]
    MetadataLeafNotUnspendable,
    /// Construction refused: static-subtree-empty.
    #[error("static-subtree-empty")]
    StaticSubtreeEmpty,
    /// Construction refused: static-subtree-incomplete.
    #[error("static-subtree-incomplete")]
    StaticSubtreeIncomplete,
    /// Construction refused: duplicate-leaf.
    #[error("duplicate-leaf")]
    DuplicateLeaf,
    /// Construction refused: conflicting-leaf.
    #[error("conflicting-leaf")]
    ConflictingLeaf,
    /// Construction refused: wrong-leaf-version.
    #[error("wrong-leaf-version")]
    WrongLeafVersion,
    /// Construction refused: wrong-internal-key.
    #[error("wrong-internal-key")]
    WrongInternalKey,
    /// Construction refused: internal-key-not-a-point.
    #[error("internal-key-not-a-point")]
    InternalKeyNotAPoint,
    /// Construction refused: canonical-branch-side-not-satisfied.
    #[error("canonical-branch-side-not-satisfied")]
    CanonicalBranchSideNotSatisfied,
    /// Construction refused: representation-search-exhausted.
    #[error("representation-search-exhausted")]
    RepresentationSearchExhausted,
    /// Construction refused: tweak-above-group-order.
    #[error("tweak-above-group-order")]
    TweakAboveGroupOrder,
    /// Construction refused: tweaked-point-is-identity.
    #[error("tweaked-point-is-identity")]
    TweakedPointIsIdentity,
    /// Construction refused: control-path-too-deep.
    #[error("control-path-too-deep")]
    ControlPathTooDeep,
    /// Construction refused: executing-leaf-absent.
    #[error("executing-leaf-absent")]
    ExecutingLeafAbsent,
    /// Construction refused: executing-leaf-repeated.
    #[error("executing-leaf-repeated")]
    ExecutingLeafRepeated,
    /// Construction refused: constructor-reference-cycle-unresolved.
    #[error("constructor-reference-cycle-unresolved")]
    ConstructorReferenceCycleUnresolved,
    /// Construction refused: resource-bound-exceeded.
    #[error("resource-bound-exceeded")]
    ResourceBoundExceeded,
}

impl StateConstructorRefusal {
    /// Every refusal in declaration order.
    pub const ALL: &'static [Self] = &[
        Self::MetadataEncodingRefused,
        Self::MetadataLeafNotUnspendable,
        Self::StaticSubtreeEmpty,
        Self::StaticSubtreeIncomplete,
        Self::DuplicateLeaf,
        Self::ConflictingLeaf,
        Self::WrongLeafVersion,
        Self::WrongInternalKey,
        Self::InternalKeyNotAPoint,
        Self::CanonicalBranchSideNotSatisfied,
        Self::RepresentationSearchExhausted,
        Self::TweakAboveGroupOrder,
        Self::TweakedPointIsIdentity,
        Self::ControlPathTooDeep,
        Self::ExecutingLeafAbsent,
        Self::ExecutingLeafRepeated,
        Self::ConstructorReferenceCycleUnresolved,
        Self::ResourceBoundExceeded,
    ];

    /// Whether changing only the representation nonce can repair this refusal.
    #[must_use]
    pub const fn retryable(self) -> bool {
        matches!(
            self,
            Self::TweakAboveGroupOrder
                | Self::TweakedPointIsIdentity
                | Self::CanonicalBranchSideNotSatisfied
        )
    }

    /// The stable diagnostic name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::MetadataEncodingRefused => "metadata-encoding-refused",
            Self::MetadataLeafNotUnspendable => "metadata-leaf-not-unspendable",
            Self::StaticSubtreeEmpty => "static-subtree-empty",
            Self::StaticSubtreeIncomplete => "static-subtree-incomplete",
            Self::DuplicateLeaf => "duplicate-leaf",
            Self::ConflictingLeaf => "conflicting-leaf",
            Self::WrongLeafVersion => "wrong-leaf-version",
            Self::WrongInternalKey => "wrong-internal-key",
            Self::InternalKeyNotAPoint => "internal-key-not-a-point",
            Self::CanonicalBranchSideNotSatisfied => "canonical-branch-side-not-satisfied",
            Self::RepresentationSearchExhausted => "representation-search-exhausted",
            Self::TweakAboveGroupOrder => "tweak-above-group-order",
            Self::TweakedPointIsIdentity => "tweaked-point-is-identity",
            Self::ControlPathTooDeep => "control-path-too-deep",
            Self::ExecutingLeafAbsent => "executing-leaf-absent",
            Self::ExecutingLeafRepeated => "executing-leaf-repeated",
            Self::ConstructorReferenceCycleUnresolved => "constructor-reference-cycle-unresolved",
            Self::ResourceBoundExceeded => "resource-bound-exceeded",
        }
    }
}

/// Mirrors `packages/target-elements-conformance/src/constructor/tagged.rs`.
fn tagged_hash(tag: &[u8], message: &[u8]) -> [u8; 32] {
    let prefix = Sha256::digest(tag);
    let mut hash = Sha256::new();
    hash.update(prefix);
    hash.update(prefix);
    hash.update(message);
    hash.finalize().into()
}

/// Mirrors the compact-size framing in the prototype's `constructor/tagged.rs`.
fn compact_size(length: usize) -> Vec<u8> {
    if let Ok(short) = u8::try_from(length)
        && short < 0xfd
    {
        return vec![short];
    }
    if let Ok(short) = u16::try_from(length) {
        let mut bytes = vec![0xfd];
        bytes.extend(short.to_le_bytes());
        return bytes;
    }
    if let Ok(word) = u32::try_from(length) {
        let mut bytes = vec![0xfe];
        bytes.extend(word.to_le_bytes());
        return bytes;
    }
    let mut bytes = vec![0xff];
    bytes.extend(length.to_le_bytes());
    bytes
}

/// Mirrors the leaf preimage in the prototype's `constructor/tree.rs`.
fn leaf_hash(version: LeafVersion, script: &[u8]) -> [u8; 32] {
    let mut preimage = vec![version.get()];
    preimage.extend(compact_size(script.len()));
    preimage.extend(script);
    tagged_hash(b"TapLeaf/elements", &preimage)
}

/// Mirrors the sorted branch in the prototype's `constructor/tree.rs`.
fn branch_hash(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let (first, second) = if left <= right {
        (left, right)
    } else {
        (right, left)
    };
    tagged_hash(
        b"TapBranch/elements",
        &[first.as_slice(), second.as_slice()].concat(),
    )
}

/// Mirrors the tweak preimage in the prototype's `constructor/tree.rs`.
fn tweak_hash(key: &[u8; 32], root: &[u8; 32]) -> [u8; 32] {
    tagged_hash(
        b"TapTweak/elements",
        &[key.as_slice(), root.as_slice()].concat(),
    )
}

/// Build the canonical three-instruction commitment leaf.
///
/// # Errors
/// Returns a metadata encoding or unspendability refusal.
pub fn state_metadata_leaf_program(
    target: &ReviewedElementsTapscriptDefinition,
    metadata: &EncodedStateMetadata,
) -> Result<TapscriptProgram, StateConstructorRefusal> {
    let bytes = encode_state_metadata(&metadata.semantic, metadata.representation);
    let item = StackItem::new(target, bytes)
        .map_err(|_| StateConstructorRefusal::MetadataEncodingRefused)?;
    let program = TapscriptProgram::new(vec![
        TapscriptInstruction::Push(item),
        TapscriptInstruction::Push(StackItem::empty()),
        TapscriptInstruction::Opcode(OpcodeId::Verify),
    ])
    .map_err(|_| StateConstructorRefusal::ResourceBoundExceeded)?;
    StateMetadataPattern::validate(target, metadata, &program)?;
    Ok(program)
}

/// The metadata pattern's evidence and protected source.
///
/// The existing backend pattern identities and owners describe compact ASH only.
/// This record uses the same abstract executor and resource projection without
/// assigning this leaf an unrelated semantic owner. Initial stack contents are
/// untouched: the two literal pushes determine the false operand of Verify.
/// Stacks too deep for those pushes abort at the target resource limit instead.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateMetadataPattern {
    /// The exact typed fragment.
    pub program: TapscriptProgram,
    /// Exact canonical bytes protected by the leaf hash.
    pub protected_source: Vec<u8>,
    /// Empty-stack execution; the literal false is independent of the prefix.
    pub execution: AbstractExecutionResult,
    /// Exact resource dimensions.
    pub resources: BTreeMap<ResourceDimension, u64>,
    /// Stack peaks above an empty initial stack.
    pub stack: ProgramStackProfile,
}

impl StateMetadataPattern {
    /// Identity of the exact metadata commitment fragment.
    pub const IDENTITY: &'static str = "state-metadata-commitment-v1";
    /// Realization owns the semantic metadata protected here.
    pub const SEMANTIC_OWNER: &'static str = "realization::StateMetadata";
    /// Every admitted initial stack either reaches Verify or hits a resource abort.
    pub const PRECONDITION: &'static str = "arbitrary target-admitted initial stack";
    /// Construction requires only canonical public bytes and reviewed primitives.
    pub const CONSTRUCTIBILITY: &'static str = "host-derived from canonical STATE metadata";
    /// The canonical metadata is public when the leaf program is disclosed.
    pub const DISCLOSURE: &'static str = "exact canonical metadata bytes";
    /// No metadata witness is an executing witness.
    pub const WITNESS_ROLE: &'static str = "non-executing-metadata-commitment";
    /// Both target-native evidence rows remain outstanding.
    pub const NATIVE_EVIDENCE: &'static str =
        "outstanding: accepted-spend absence and attempted-spend rejection";
    /// The abstract contract is not native evidence.
    pub const RESIDUAL: &'static str =
        "target-native behavior and hash collision resistance remain assumptions";
    /// The fragment uses reviewed literal pushes and Boolean verification.
    pub const PREREQUISITES: &'static str =
        "canonical literal pushes; Verify aborts on the empty item";

    /// Decode only the canonical STATE schema and build its pattern.
    ///
    /// # Errors
    /// Returns `MetadataEncodingRefused` for any noncanonical encoding.
    pub fn from_bytes(
        target: &ReviewedElementsTapscriptDefinition,
        bytes: &[u8],
    ) -> Result<Self, StateConstructorRefusal> {
        let metadata = decode_state_metadata(bytes)
            .map_err(|_| StateConstructorRefusal::MetadataEncodingRefused)?;
        let program = state_metadata_leaf_program(target, &metadata)?;
        Self::validate(target, &metadata, &program)
    }

    /// Validate both the commitment bytes and unconditional abort form.
    ///
    /// # Errors
    /// Returns `MetadataLeafNotUnspendable` for a different program or surviving state.
    pub fn validate(
        target: &ReviewedElementsTapscriptDefinition,
        metadata: &EncodedStateMetadata,
        program: &TapscriptProgram,
    ) -> Result<Self, StateConstructorRefusal> {
        let bytes = encode_state_metadata(&metadata.semantic, metadata.representation);
        let instructions = program.instructions();
        let exact = matches!(instructions, [TapscriptInstruction::Push(item), TapscriptInstruction::Push(empty), TapscriptInstruction::Opcode(OpcodeId::Verify)] if item.bytes() == bytes && empty.bytes().is_empty());
        let initial = AbstractStackState::from_main(Vec::new());
        let limits = AbstractLimits::for_target(target);
        let execution = validate_program(target, program, &initial, limits)
            .map_err(|_| StateConstructorRefusal::MetadataLeafNotUnspendable)?;
        if !exact || !execution.always_aborts() {
            return Err(StateConstructorRefusal::MetadataLeafNotUnspendable);
        }
        Ok(Self {
            program: program.clone(),
            protected_source: bytes,
            execution,
            resources: resource_projection(target, program),
            stack: program_stack_profile(target, program, &initial, limits),
        })
    }
}

/// A metadata commitment or a specific static operation role.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StateLeafRole {
    /// The aborting canonical metadata leaf.
    MetadataCommitment,
    /// The maturity-announcement operation.
    Announcement,
    /// One support leaf required by the supplied strategy.
    Support(u32),
}

/// One offered static leaf, before version and set validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateStaticLeaf {
    /// The semantic role, distinct from leaf identity.
    pub role: StateLeafRole,
    /// The exact typed program.
    pub program: TapscriptProgram,
    /// An offered version byte, checked against the target.
    pub version: u8,
}

/// A fully stated static tree; no opaque root substitutes for a leaf set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StateStaticNode {
    /// An identity and its complete leaf definition.
    Leaf {
        identity: u32,
        leaf: StateStaticLeaf,
    },
    /// Two children whose hashes use ordinary target ordering.
    Branch(Box<Self>, Box<Self>),
}

/// One validated static leaf and its internal path, deepest sibling first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateStaticLeafEntry {
    /// Caller-supplied identity, used to detect conflicting declarations.
    pub identity: u32,
    /// The exact definition.
    pub leaf: StateStaticLeaf,
    /// The target leaf hash.
    pub hash: [u8; 32],
    /// Siblings inside the static subtree only.
    pub siblings: Vec<[u8; 32]>,
}

/// A nonempty, complete static subtree with validated identities and versions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateStaticSubtree {
    tree: StateStaticNode,
    root: [u8; 32],
    leaves: Vec<StateStaticLeafEntry>,
}

impl StateStaticSubtree {
    /// Validate a whole offered tree, retaining all reconstruction inputs.
    ///
    /// # Errors
    /// Refuses empty, incomplete, duplicate, conflicting, wrong-version or deep trees.
    pub fn new(
        target: &ReviewedElementsTapscriptDefinition,
        tree: Option<StateStaticNode>,
    ) -> Result<Self, StateConstructorRefusal> {
        let tree = tree.ok_or(StateConstructorRefusal::StaticSubtreeEmpty)?;
        let mut leaves = Vec::new();
        let root = collect_static(target, &tree, 0, &mut leaves)?;
        if !leaves
            .iter()
            .any(|entry| entry.leaf.role == StateLeafRole::Announcement)
        {
            return Err(StateConstructorRefusal::StaticSubtreeIncomplete);
        }
        Ok(Self { tree, root, leaves })
    }

    /// The supplied node tree.
    #[must_use]
    pub const fn tree(&self) -> &StateStaticNode {
        &self.tree
    }
    /// The static root.
    #[must_use]
    pub const fn root(&self) -> &[u8; 32] {
        &self.root
    }
    /// All leaves with their internal paths.
    #[must_use]
    pub fn leaves(&self) -> &[StateStaticLeafEntry] {
        &self.leaves
    }

    /// Locate one executing hash without silently choosing an occurrence.
    ///
    /// # Errors
    /// Refuses an absent or repeated executing leaf.
    pub fn path_to(&self, hash: &[u8; 32]) -> Result<&[[u8; 32]], StateConstructorRefusal> {
        let mut matches = self.leaves.iter().filter(|entry| entry.hash == *hash);
        let entry = matches
            .next()
            .ok_or(StateConstructorRefusal::ExecutingLeafAbsent)?;
        if matches.next().is_some() {
            return Err(StateConstructorRefusal::ExecutingLeafRepeated);
        }
        Ok(&entry.siblings)
    }
}

fn collect_static(
    target: &ReviewedElementsTapscriptDefinition,
    node: &StateStaticNode,
    depth: usize,
    leaves: &mut Vec<StateStaticLeafEntry>,
) -> Result<[u8; 32], StateConstructorRefusal> {
    if depth > 128 {
        return Err(StateConstructorRefusal::ControlPathTooDeep);
    }
    if leaves.len() >= 4096 {
        return Err(StateConstructorRefusal::ResourceBoundExceeded);
    }
    match node {
        StateStaticNode::Leaf { identity, leaf } => {
            if let Some(previous) = leaves.iter().find(|entry| entry.identity == *identity) {
                return Err(if previous.leaf == *leaf {
                    StateConstructorRefusal::DuplicateLeaf
                } else {
                    StateConstructorRefusal::ConflictingLeaf
                });
            }
            if leaf.version != target.definition().leaf_version().get() {
                return Err(StateConstructorRefusal::WrongLeafVersion);
            }
            if leaf.role == StateLeafRole::MetadataCommitment {
                return Err(StateConstructorRefusal::StaticSubtreeIncomplete);
            }
            let hash = leaf_hash(
                target.definition().leaf_version(),
                &leaf.program.encode(target),
            );
            leaves.push(StateStaticLeafEntry {
                identity: *identity,
                leaf: leaf.clone(),
                hash,
                siblings: Vec::new(),
            });
            Ok(hash)
        }
        StateStaticNode::Branch(left, right) => {
            let start = leaves.len();
            let left_hash = collect_static(target, left, depth + 1, leaves)?;
            let middle = leaves.len();
            let right_hash = collect_static(target, right, depth + 1, leaves)?;
            for entry in &mut leaves[start..middle] {
                entry.siblings.push(right_hash);
            }
            for entry in &mut leaves[middle..] {
                entry.siblings.push(left_hash);
            }
            Ok(branch_hash(&left_hash, &right_hash))
        }
    }
}

/// The only admitted outer branch side.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StateBranchSide {
    /// Metadata is the left child and its hash cannot exceed the static root.
    MetadataLeftStaticRight,
}

impl StateBranchSide {
    /// Check the fixed side before attempting any curve operation.
    ///
    /// # Errors
    /// Returns `CanonicalBranchSideNotSatisfied` when metadata sorts second.
    pub fn check(
        self,
        metadata: &[u8; 32],
        static_root: &[u8; 32],
    ) -> Result<(), StateConstructorRefusal> {
        if metadata <= static_root {
            Ok(())
        } else {
            Err(StateConstructorRefusal::CanonicalBranchSideNotSatisfied)
        }
    }
}

/// A bounded scan starting at zero, never beyond the reviewed 4,096 candidates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StateNonceBudget(u32);

impl Default for StateNonceBudget {
    fn default() -> Self {
        Self(4096)
    }
}

impl StateNonceBudget {
    /// Narrow the reviewed budget for a caller's resource limit.
    ///
    /// # Errors
    /// Returns `ResourceBoundExceeded` for zero or more than 4,096 attempts.
    pub const fn new(attempts: u32) -> Result<Self, StateConstructorRefusal> {
        if attempts == 0 || attempts > 4096 {
            return Err(StateConstructorRefusal::ResourceBoundExceeded);
        }
        Ok(Self(attempts))
    }
    /// The number of candidate nonces, starting at zero.
    #[must_use]
    pub const fn attempts(self) -> u32 {
        self.0
    }
}

/// The public secp256k1 generator's x coordinate.
pub const STATE_GENERATOR_X: [u8; 32] = [
    0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac, 0x55, 0xa0, 0x62, 0x95, 0xce, 0x87, 0x0b, 0x07,
    0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9, 0x59, 0xf2, 0x81, 0x5b, 0x16, 0xf8, 0x17, 0x98,
];
/// The public secp256k1 generator's y coordinate.
pub const STATE_GENERATOR_Y: [u8; 32] = [
    0x48, 0x3a, 0xda, 0x77, 0x26, 0xa3, 0xc4, 0x65, 0x5d, 0xa4, 0xfb, 0xfc, 0x0e, 0x11, 0x08, 0xa8,
    0xfd, 0x17, 0xb4, 0x48, 0xa6, 0x85, 0x54, 0x19, 0x9c, 0x47, 0xd0, 0x8f, 0xfb, 0x10, 0xd4, 0xb8,
];
/// The NUMS x coordinate, recomputed from the uncompressed generator on admission.
pub const STATE_NUMS_KEY: [u8; 32] = [
    0x50, 0x92, 0x9b, 0x74, 0xc1, 0xa0, 0x49, 0x54, 0xb7, 0x8b, 0x4b, 0x60, 0x35, 0xe9, 0x7a, 0x5e,
    0x07, 0x8a, 0x5a, 0x0f, 0x28, 0xec, 0x96, 0xd5, 0x47, 0xbf, 0xee, 0x9a, 0xce, 0x80, 0x3a, 0xc0,
];

/// A publicly derived internal key with its residual policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StateInternalKeyPolicy([u8; 32]);

impl StateInternalKeyPolicy {
    /// Knowledge of a private scalar is not ruled out by point validation.
    pub const RESIDUAL: &'static str = "discrete-log hardness and SHA-256 preimage resistance; no proof that a key path is impossible";
    /// The existing internal-key policy, without strengthening its claim.
    pub const INTERNAL: InternalKeyPolicy =
        InternalKeyPolicy::UnspendableWithResidualDiscreteLogAssumption;
    /// The constructor provides no accepted key-path escape.
    pub const KEY_PATH: KeyPathPolicy = KeyPathPolicy::NoAcceptedEscape;

    /// Recompute the public derivation and ask the capability to validate the point.
    ///
    /// # Errors
    /// Refuses any other key or a capability's failed point validation.
    pub fn new(
        offered: [u8; 32],
        curve: &impl StateCurveCapability,
    ) -> Result<Self, StateConstructorRefusal> {
        let mut hash = Sha256::new();
        hash.update([0x04]);
        hash.update(STATE_GENERATOR_X);
        hash.update(STATE_GENERATOR_Y);
        let derived: [u8; 32] = hash.finalize().into();
        if offered != derived || derived != STATE_NUMS_KEY {
            return Err(StateConstructorRefusal::WrongInternalKey);
        }
        if !curve.internal_key_is_a_point(&offered) {
            return Err(StateConstructorRefusal::InternalKeyNotAPoint);
        }
        Ok(Self(offered))
    }
    /// The exact admitted key bytes.
    #[must_use]
    pub const fn key(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Identity of this constructor recipe, independent of any repository revision.
///
/// A generation names a recipe, not a version of some datum inside one. Two
/// generations differ in how the tree is shaped and what its leaves commit to,
/// so an object built under one is not an object built under another with a
/// field changed; that is why this is an identity here rather than a number in
/// the metadata. The census holds a single member because §17.3 admits no
/// constructor migration and none is implemented, so no second recipe can be
/// reached from an object built under this one. The guide's own status lists
/// migration among the outstanding items, which is where a second member would
/// come from if it ever came.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StateConstructorGeneration {
    /// Canonical metadata under a fixed-side depth-one outer tree.
    CanonicalMetadataV1,
}

/// Every input dependency the constructor hands to the linker.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StateConstructorReference {
    /// Canonical metadata schema revision.
    MetadataSchema(u32),
    /// Exact static root, with no migration implied.
    StaticSubtreeRoot([u8; 32]),
    /// Target leaf version.
    LeafVersion(LeafVersion),
    /// Admitted internal key and its residual policy.
    InternalKeyPolicy(StateInternalKeyPolicy),
    /// The fixed outer ordering.
    BranchSide(StateBranchSide),
    /// Host search bound.
    NonceBudget(StateNonceBudget),
    /// Reviewed target contract revision.
    TargetPolicy(TargetContractVersion),
}

/// One typed reference and the references on which it depends.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateReferenceDeclaration {
    /// The dependent reference.
    pub reference: StateConstructorReference,
    /// Dependencies; order is immaterial to the local census.
    pub dependencies: Vec<StateConstructorReference>,
}

/// A deterministic local component census, without authenticated cuts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateReferenceCensus {
    /// Strongly connected components, ordered by their smallest reference.
    pub components: Vec<Vec<StateConstructorReference>>,
    cyclic: bool,
}

impl StateReferenceCensus {
    /// Compute reachability and strongly connected components of the declarations.
    ///
    /// # Errors
    /// Returns `ResourceBoundExceeded` for more than 64 distinct references.
    pub fn new(
        declarations: &[StateReferenceDeclaration],
    ) -> Result<Self, StateConstructorRefusal> {
        use StateConstructorReference as Reference;
        let mut reachable = BTreeMap::<Reference, BTreeSet<Reference>>::new();
        for declaration in declarations {
            reachable
                .entry(declaration.reference)
                .or_default()
                .extend(&declaration.dependencies);
            for dependency in &declaration.dependencies {
                reachable.entry(*dependency).or_default();
            }
            if reachable.len() > 64 {
                return Err(StateConstructorRefusal::ResourceBoundExceeded);
            }
        }
        let nodes: Vec<_> = reachable.keys().copied().collect();
        for via in &nodes {
            let onward = reachable[via].clone();
            for paths in reachable.values_mut() {
                if paths.contains(via) {
                    paths.extend(&onward);
                }
            }
        }
        let cyclic = nodes.iter().any(|node| reachable[node].contains(node));
        let mut remaining: BTreeSet<_> = nodes.into_iter().collect();
        let mut components = Vec::new();
        while let Some(first) = remaining.pop_first() {
            let mut component = vec![first];
            let peers: Vec<_> = remaining
                .iter()
                .copied()
                .filter(|other| {
                    reachable[&first].contains(other) && reachable[other].contains(&first)
                })
                .collect();
            for peer in peers {
                remaining.remove(&peer);
                component.push(peer);
            }
            components.push(component);
        }
        Ok(Self { components, cyclic })
    }
    /// Refuse every cycle; no cut is selected or authenticated here.
    ///
    /// # Errors
    /// Returns `ConstructorReferenceCycleUnresolved` for any cyclic component.
    pub const fn require_acyclic(&self) -> Result<(), StateConstructorRefusal> {
        if self.cyclic {
            Err(StateConstructorRefusal::ConstructorReferenceCycleUnresolved)
        } else {
            Ok(())
        }
    }
}

/// Typed control-block reconstruction inputs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateControlRecipe {
    /// The executing role, or the non-executing metadata commitment role.
    pub role: StateLeafRole,
    /// Reviewed execution version.
    pub leaf_version: LeafVersion,
    /// Whether the output point has odd y.
    pub parity: bool,
    /// Exact internal key.
    pub internal_key: [u8; 32],
    /// Leaf authenticated by this path.
    pub executing_leaf_hash: [u8; 32],
    /// Ordered siblings, deepest first.
    pub siblings: Vec<[u8; 32]>,
}

impl StateControlRecipe {
    /// Derive bytes instead of treating stored bytes as reconstruction authority.
    ///
    /// # Errors
    /// Returns `ControlPathTooDeep` beyond 128 sibling nodes.
    pub fn control_bytes(&self) -> Result<Vec<u8>, StateConstructorRefusal> {
        if self.siblings.len() > 128 {
            return Err(StateConstructorRefusal::ControlPathTooDeep);
        }
        let mut bytes = vec![self.leaf_version.get() | u8::from(self.parity)];
        bytes.extend(self.internal_key);
        for sibling in &self.siblings {
            bytes.extend(sibling);
        }
        Ok(bytes)
    }
}

/// A typed semantic field's commitment, not an observed or accepted fact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateFieldCommitment {
    /// Transaction side whose metadata is being constructed.
    pub side: TransactionSide,
    /// The realization-owned semantic selector.
    pub field: StateField,
    /// Half-open canonical encoding range.
    pub range: Range<usize>,
    /// Exact committed bytes at that range.
    pub bytes: Vec<u8>,
}

/// Host evidence for the least admitted nonce within the chosen budget.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateNonceEvidence {
    /// Every rejected lower nonce, in scan order, with its repairable refusal.
    pub rejected: Vec<(StateRepresentationNonce, StateConstructorRefusal)>,
    /// The first admitted nonce.
    pub selected: StateRepresentationNonce,
}

impl StateNonceEvidence {
    /// Exhaustion does not rule out an admissible nonce beyond the bound.
    pub const RESIDUAL: &'static str =
        "host leastness only; a later admissible nonce may exist beyond the search budget";
}

/// A fallibly derived candidate STATE output and its reconstruction evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateStateConstructor {
    metadata: EncodedStateMetadata,
    pattern: StateMetadataPattern,
    static_subtree: StateStaticSubtree,
    internal_key: StateInternalKeyPolicy,
    leaf_version: LeafVersion,
    target_policy: TargetContractVersion,
    budget: StateNonceBudget,
    metadata_hash: [u8; 32],
    merkle_root: [u8; 32],
    output_key: [u8; 32],
    parity: bool,
    evidence: StateNonceEvidence,
}

impl CandidateStateConstructor {
    /// Grind from zero, retaining every rejected lower representation.
    ///
    /// # Errors
    /// Returns a nonretryable defect immediately, or search exhaustion at the bound.
    pub fn derive(
        target: &ReviewedElementsTapscriptDefinition,
        metadata: &StateMetadata,
        static_subtree: &StateStaticSubtree,
        policy: StateInternalKeyPolicy,
        budget: StateNonceBudget,
        curve: &impl StateCurveCapability,
    ) -> Result<Self, StateConstructorRefusal> {
        StateInternalKeyPolicy::new(*policy.key(), curve)?;
        validate_static_paths(static_subtree)?;
        let mut rejected = Vec::new();
        for attempt in 0..budget.attempts() {
            let representation = StateRepresentationNonce::new(attempt);
            let encoded = EncodedStateMetadata {
                semantic: *metadata,
                representation,
            };
            let program = state_metadata_leaf_program(target, &encoded)?;
            let hash = leaf_hash(target.definition().leaf_version(), &program.encode(target));
            let result = StateBranchSide::MetadataLeftStaticRight
                .check(&hash, static_subtree.root())
                .and_then(|()| {
                    let root = branch_hash(&hash, static_subtree.root());
                    match curve.output_key(policy.key(), &root) {
                        StateTweakOutcome::OutputKey { key, parity } => Ok((root, key, parity)),
                        StateTweakOutcome::TweakAboveGroupOrder => {
                            Err(StateConstructorRefusal::TweakAboveGroupOrder)
                        }
                        StateTweakOutcome::TweakedPointIsIdentity => {
                            Err(StateConstructorRefusal::TweakedPointIsIdentity)
                        }
                        StateTweakOutcome::InternalKeyNotAPoint => {
                            Err(StateConstructorRefusal::InternalKeyNotAPoint)
                        }
                    }
                });
            match result {
                Ok((merkle_root, output_key, parity)) => {
                    return Ok(Self {
                        metadata: encoded,
                        pattern: StateMetadataPattern::validate(target, &encoded, &program)?,
                        static_subtree: static_subtree.clone(),
                        internal_key: policy,
                        leaf_version: target.definition().leaf_version(),
                        target_policy: target.definition().version(),
                        budget,
                        metadata_hash: hash,
                        merkle_root,
                        output_key,
                        parity,
                        evidence: StateNonceEvidence {
                            rejected,
                            selected: representation,
                        },
                    });
                }
                Err(refusal) if refusal.retryable() => rejected.push((representation, refusal)),
                Err(refusal) => return Err(refusal),
            }
        }
        Err(StateConstructorRefusal::RepresentationSearchExhausted)
    }

    /// The semantic metadata and selected representation nonce.
    #[must_use]
    pub const fn encoded_metadata(&self) -> &EncodedStateMetadata {
        &self.metadata
    }
    /// Exact canonical committed bytes.
    #[must_use]
    pub fn metadata_bytes(&self) -> &[u8] {
        &self.pattern.protected_source
    }
    /// The selected representation nonce.
    #[must_use]
    pub const fn nonce(&self) -> StateRepresentationNonce {
        self.metadata.representation
    }
    /// The typed metadata commitment program.
    #[must_use]
    pub const fn leaf_program(&self) -> &TapscriptProgram {
        &self.pattern.program
    }
    /// Abstract pattern evidence and outstanding native obligations.
    #[must_use]
    pub const fn pattern(&self) -> &StateMetadataPattern {
        &self.pattern
    }
    /// The shared static subtree.
    #[must_use]
    pub const fn static_subtree(&self) -> &StateStaticSubtree {
        &self.static_subtree
    }
    /// The complete outer root.
    #[must_use]
    pub const fn merkle_root(&self) -> &[u8; 32] {
        &self.merkle_root
    }
    /// The Elements tagged tweak digest, before scalar interpretation.
    #[must_use]
    pub fn tweak_hash(&self) -> [u8; 32] {
        tweak_hash(self.internal_key.key(), &self.merkle_root)
    }
    /// The capability-supplied x-only output key.
    #[must_use]
    pub const fn output_key(&self) -> &[u8; 32] {
        &self.output_key
    }
    /// Whether the output point has odd y.
    #[must_use]
    pub const fn parity(&self) -> bool {
        self.parity
    }
    /// Witness-version-one output program over the output key.
    #[must_use]
    pub fn output_program(&self) -> Vec<u8> {
        let mut bytes = vec![0x51, 0x20];
        bytes.extend(self.output_key);
        bytes
    }
    /// The host's rejected lower candidates and selected nonce.
    #[must_use]
    pub const fn evidence(&self) -> &StateNonceEvidence {
        &self.evidence
    }
    /// The closed recipe generation identity.
    #[must_use]
    pub const fn generation(&self) -> StateConstructorGeneration {
        StateConstructorGeneration::CanonicalMetadataV1
    }

    /// Derive a unique role's control recipe.
    ///
    /// # Errors
    /// Refuses an absent or repeated executing role, or an overdeep control path.
    pub fn control_recipe(
        &self,
        role: StateLeafRole,
    ) -> Result<StateControlRecipe, StateConstructorRefusal> {
        let metadata_hash = self.metadata_hash;
        let (executing_leaf_hash, siblings) = if role == StateLeafRole::MetadataCommitment {
            (metadata_hash, vec![*self.static_subtree.root()])
        } else {
            let mut matches = self
                .static_subtree
                .leaves
                .iter()
                .filter(|entry| entry.leaf.role == role);
            let entry = matches
                .next()
                .ok_or(StateConstructorRefusal::ExecutingLeafAbsent)?;
            if matches.next().is_some() {
                return Err(StateConstructorRefusal::ExecutingLeafRepeated);
            }
            let mut path = entry.siblings.clone();
            path.push(metadata_hash);
            (entry.hash, path)
        };
        let recipe = StateControlRecipe {
            role,
            leaf_version: self.leaf_version,
            parity: self.parity,
            internal_key: *self.internal_key.key(),
            executing_leaf_hash,
            siblings,
        };
        recipe.control_bytes()?;
        Ok(recipe)
    }

    /// The six field commitments on the requested side.
    #[must_use]
    pub fn field_commitments(&self, side: TransactionSide) -> Vec<StateFieldCommitment> {
        let fields = [
            (StateField::Omega, 25..33),
            (StateField::YL, 33..41),
            (StateField::YT, 41..49),
            (StateField::Q, 49..57),
            (StateField::Cycle, 57..65),
            (StateField::Maturity, 65..74),
        ];
        fields
            .into_iter()
            .map(|(field, range)| StateFieldCommitment {
                side,
                field,
                bytes: self.metadata_bytes()[range.clone()].to_vec(),
                range,
            })
            .collect()
    }

    /// The exact dependencies of this construction, in canonical variant order.
    #[must_use]
    pub fn reference_declarations(&self) -> Vec<StateReferenceDeclaration> {
        use StateConstructorReference as Reference;
        [
            Reference::MetadataSchema(STATE_METADATA_SCHEMA),
            Reference::StaticSubtreeRoot(*self.static_subtree.root()),
            Reference::LeafVersion(self.leaf_version),
            Reference::InternalKeyPolicy(self.internal_key),
            Reference::BranchSide(StateBranchSide::MetadataLeftStaticRight),
            Reference::NonceBudget(self.budget),
            Reference::TargetPolicy(self.target_policy),
        ]
        .into_iter()
        .map(|reference| StateReferenceDeclaration {
            reference,
            dependencies: Vec::new(),
        })
        .collect()
    }

    /// Check the fixed construction parameters shared by adjacent states.
    ///
    /// # Errors
    /// Refuses a different static root, internal key, or leaf version.
    pub fn continuity(&self, other: &Self) -> Result<(), StateConstructorRefusal> {
        if self.static_subtree.root() != other.static_subtree.root() {
            return Err(StateConstructorRefusal::ConflictingLeaf);
        }
        if self.internal_key != other.internal_key {
            return Err(StateConstructorRefusal::WrongInternalKey);
        }
        if self.leaf_version != other.leaf_version {
            return Err(StateConstructorRefusal::WrongLeafVersion);
        }
        Ok(())
    }
}

fn validate_static_paths(subtree: &StateStaticSubtree) -> Result<(), StateConstructorRefusal> {
    let mut roles = BTreeSet::new();
    for entry in &subtree.leaves {
        if !roles.insert(entry.leaf.role) {
            return Err(StateConstructorRefusal::ExecutingLeafRepeated);
        }
        if subtree.path_to(&entry.hash)?.len() >= 128 {
            return Err(StateConstructorRefusal::ControlPathTooDeep);
        }
    }
    Ok(())
}

/// Reconstruct the predecessor with the supplied static strategy.
///
/// # Errors
/// Returns the constructor's precise refusal without changing semantic metadata.
pub fn predecessor_recipe(
    target: &ReviewedElementsTapscriptDefinition,
    metadata: &StateMetadata,
    static_subtree: &StateStaticSubtree,
    policy: StateInternalKeyPolicy,
    budget: StateNonceBudget,
    curve: &impl StateCurveCapability,
) -> Result<CandidateStateConstructor, StateConstructorRefusal> {
    CandidateStateConstructor::derive(target, metadata, static_subtree, policy, budget, curve)
}

/// Construct the semantically derived successor with the same static strategy.
///
/// The caller supplies realization's transition result, not an independently
/// authenticated witness blob. Witness authentication is a later program's job.
///
/// # Errors
/// Returns the constructor's precise refusal without changing semantic metadata.
pub fn successor_recipe(
    target: &ReviewedElementsTapscriptDefinition,
    metadata: &StateMetadata,
    static_subtree: &StateStaticSubtree,
    policy: StateInternalKeyPolicy,
    budget: StateNonceBudget,
    curve: &impl StateCurveCapability,
) -> Result<CandidateStateConstructor, StateConstructorRefusal> {
    CandidateStateConstructor::derive(target, metadata, static_subtree, policy, budget, curve)
}
