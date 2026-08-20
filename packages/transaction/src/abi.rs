//! The candidate compact-ASH transaction ABI (§15.3, §15.4).
//!
//! # Derived, never parsed
//!
//! §15.3 fixes the five inputs an ABI derives from — the candidate
//! linked bundle, the exact reviewed target, the candidate shape
//! assignment, the fixed explicit representation, and the typed
//! transaction-form decision — and adds that it parses no script bytes,
//! reports, plans, or reference prose. Every field below is read from a
//! typed value handed in, and the one place bytes appear is the
//! committed tree's hashing, which consumes programs the linker built
//! rather than bytes anyone wrote down.
//!
//! # A candidate, held there by three facts
//!
//! §1.9 keeps the candidate and final states distinct, and this type
//! stays on the candidate side without relying on a reader noticing a
//! comment:
//!
//! - [`AbiStatus`] is read, never written; there is no field, argument,
//!   or setter through which a caller could claim more, and the
//!   evidence a promotion would rest on is Wave 11's;
//! - the ABI is derived only from a bundle whose own status is a
//!   prototype, and a bundle claiming more is a refusal;
//! - [`OutstandingAbiObligations`] is structurally non-empty, and the
//!   least of them is the taproot output key the linker did not
//!   discharge and this layer discharges only halfway.
//!
//! # No digest, and no room for one
//!
//! §1.10 admits a digest only once a real consumer of one appears, and
//! §15.10 says in as many words that the ABI carries no identity digest
//! by default. There is no ABI hash here and no field that would hold
//! one.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;

use linker::backend::{
    AshRepresentationSelection, BundleSymbol, CompactAshShape, ConcreteLayout,
    ConcreteRelationPlacement, ConstructorAssumption, ExplicitValuePolicy, InputPlacement,
    InputRole, InternalKeyPolicy, KeyPathPolicy, LeafRole, OutputRole, StackItem, WitnessRole,
};
use linker::error::RelationCaseKey;
use linker::{CandidateLinkedBundle, LinkObligation, SymbolValue};
use target_elements::{
    LeafVersion, ResourceDimension, ReviewedElementsTapscriptDefinition, TargetContractVersion,
    TransactionForm, TransactionFormReview,
};

use crate::bytes::AssetId;
use crate::error::TransactionRefusal;
use crate::taproot::{CommittedTree, PinnedAshInstance, commit_tree};

/// The status vocabulary a derived ABI is distinguished by (§15.10).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AbiStatus {
    /// Derived and internally checked; no operation evidence.
    Candidate,
}

/// One obligation the ABI creates or inherits and does not discharge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AbiObligation {
    /// The taproot output key is pinned rather than recomputed.
    ///
    /// Inherited from the link, and narrowed rather than cleared. The
    /// merkle root *is* computed here, because the control blocks the
    /// witness carries are made of it. The tweak that turns that root
    /// and the internal key into an output key is not, and so the
    /// equality between the pinned program and this tree is the exact
    /// thing nothing in this crate establishes.
    ///
    /// It is not a gap that a bigger crate would close. The object
    /// being spent already exists on the target, its program is an
    /// observed public fact under §15.6, and the arithmetic that would
    /// check it belongs to the independent oracle this crate's output
    /// is compared against — an oracle a builder must not call.
    PinnedOutputKeyUnverifiedAgainstTree,
    /// The internal key was not verified unspendable from public data.
    ///
    /// Inherited from the link unchanged. The constructor names the
    /// assumption; discharging it needs curve arithmetic over a real
    /// key, so it is recorded rather than claimed.
    InternalKeyUnspendabilityUnverified,
    /// The clear lifecycle is absent, so no instance this ABI builds
    /// for can be retired.
    ClearLifecycleAbsent,
    /// No target node has executed anything this ABI states.
    ///
    /// The ABI is a statement about bytes, and every claim that those
    /// bytes are accepted belongs to a run rather than to a
    /// derivation.
    TargetExecutionEvidenceAbsent,
}

/// The outstanding ABI obligations, which are never none.
///
/// Structurally non-empty for the same reason the link's are: the least
/// obligation is a field of its own, so an ABI owing nothing has no
/// representation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OutstandingAbiObligations {
    least: AbiObligation,
    rest: BTreeSet<AbiObligation>,
}

impl OutstandingAbiObligations {
    /// Every outstanding obligation, in canonical order.
    pub fn obligations(&self) -> impl Iterator<Item = &AbiObligation> {
        std::iter::once(&self.least).chain(self.rest.iter())
    }

    /// How many are outstanding, which is never zero.
    #[must_use]
    pub fn count(&self) -> NonZeroUsize {
        NonZeroUsize::MIN.saturating_add(self.rest.len())
    }

    /// Whether one obligation is outstanding.
    #[must_use]
    pub fn holds(&self, obligation: AbiObligation) -> bool {
        self.least == obligation || self.rest.contains(&obligation)
    }
}

/// The rule fixing which input is the coordinator (§10.3).
///
/// A rule rather than a selection. §15.5 forbids a request from
/// choosing the coordinator, and this type is why it cannot: the index
/// is a constant of the ABI and the canonical sort is what decides
/// which outpoint lands on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoordinatorRule {
    index: u16,
}

impl CoordinatorRule {
    /// The input index the coordinator program requires of itself.
    #[must_use]
    pub const fn index(self) -> u16 {
        self.index
    }
}

/// How the ABI orders each input family (§10.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CanonicalOrdering {
    /// Ascending by transaction identifier, then by output index.
    ///
    /// The target's own outpoint order, which is what makes it
    /// canonical rather than merely deterministic: it is a total order
    /// on outpoints that two independent builders reach without
    /// agreeing on anything else first.
    AscendingOutpoint,
}

/// What the ABI fixes about the sequence field.
///
/// An ABI convention, and typed as one. No compact-ASH program inspects
/// a sequence field or executes a relative timelock, so nothing in the
/// protocol relation constrains this and the target constrains it only
/// through rules the candidate does not engage. Recording it as a
/// convention is the difference between a layout the project may revise
/// and one it may not.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SequenceConstraint {
    /// Every input carries the final sequence, which engages no
    /// relative timelock and leaves the lock-time field inert.
    FinalOnEveryInput,
}

impl SequenceConstraint {
    /// The exact sequence field this constraint fixes.
    ///
    /// Provenance: `CTxIn::SEQUENCE_FINAL`
    /// (`src/primitives/transaction.h`).
    #[must_use]
    pub const fn sequence(self) -> u32 {
        match self {
            Self::FinalOnEveryInput => 0xffff_ffff,
        }
    }
}

/// Which transaction version one form is built at, and why.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TargetTransactionVersion {
    /// The ordinary version, which engages no topology policy.
    ///
    /// Provenance: `CTransaction::CURRENT_VERSION`. Used for the
    /// sponsored form, whose own fee reaches the relay floor and which
    /// therefore needs no package to travel.
    Standard,
    /// The topology-restricted version.
    ///
    /// Provenance: `TRUC_VERSION` (`src/policy/truc_policy.h`). Used
    /// for the sponsorless form, whose reviewed relay verdict is
    /// admitted-under-condition and one of whose two conditions is a
    /// topology-restricted package. Selecting it is what turns that
    /// recorded condition into a constructible path, and it is a policy
    /// choice rather than a consensus requirement: consensus admits the
    /// sponsorless form at either version.
    TopologyRestricted,
}

impl TargetTransactionVersion {
    /// The exact version field.
    #[must_use]
    pub const fn version(self) -> u32 {
        match self {
            Self::Standard => 2,
            Self::TopologyRestricted => 3,
        }
    }
}

/// The package limits the topology-restricted path imposes.
///
/// Settles `PackageLimit`. Every figure is a virtual-size or a count
/// rather than a weight, because that is the unit the relay path
/// applies them in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageLimits {
    transactions: u32,
    parent_virtual_size: u64,
    child_virtual_size: u64,
}

impl PackageLimits {
    /// The reviewed limits.
    ///
    /// Provenance, in `src/policy/truc_policy.h`:
    /// `TRUC_ANCESTOR_LIMIT` and `TRUC_DESCENDANT_LIMIT`, which are
    /// both two and together admit exactly one parent and one child;
    /// `TRUC_MAX_VSIZE`; and `TRUC_CHILD_MAX_VSIZE`.
    #[must_use]
    pub const fn reviewed() -> Self {
        Self {
            transactions: 2,
            parent_virtual_size: 10_000,
            child_virtual_size: 1_000,
        }
    }

    /// How many transactions one package may hold.
    #[must_use]
    pub const fn transactions(self) -> u32 {
        self.transactions
    }

    /// The largest virtual size the sponsorless parent may reach.
    #[must_use]
    pub const fn parent_virtual_size(self) -> u64 {
        self.parent_virtual_size
    }

    /// The largest virtual size its fee-paying child may reach.
    #[must_use]
    pub const fn child_virtual_size(self) -> u64 {
        self.child_virtual_size
    }
}

/// One item a script-path witness carries, in stack order (§15.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum WitnessItem {
    /// The leaf program being executed.
    LeafScript,
    /// The control block proving the leaf is in the committed tree.
    ControlBlock,
}

/// The public deployment constants the ABI reads from the link.
///
/// Read from the linked definition census rather than taken as
/// parameters. §1.10 refuses a speculative identity, and a transaction
/// layer that accepted its own asset identifiers would be able to build
/// a transaction against an asset the programs it embeds never
/// mentioned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeploymentSymbols {
    closed_asset: AssetId,
    reserve_asset: AssetId,
    sponsor_change_program: Vec<u8>,
    sponsor_change_version: u8,
}

impl DeploymentSymbols {
    /// The closed protocol asset the ASH family carries.
    #[must_use]
    pub const fn closed_asset(&self) -> AssetId {
        self.closed_asset
    }

    /// The reserve asset every sponsor and fee role carries.
    #[must_use]
    pub const fn reserve_asset(&self) -> AssetId {
        self.reserve_asset
    }

    /// The sponsor-change role's witness program payload.
    #[must_use]
    pub fn sponsor_change_program(&self) -> &[u8] {
        &self.sponsor_change_program
    }

    /// The version the sponsor-change witness program is read at.
    #[must_use]
    pub const fn sponsor_change_version(&self) -> u8 {
        self.sponsor_change_version
    }
}

/// Everything the ABI states about one admitted shape (§15.4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShapeAbi {
    shape: CompactAshShape,
    layout: ConcreteLayout,
    ash_range: (u16, u16),
    sponsor_range: (u16, u16),
    successor_position: u16,
    sponsor_change_position: Option<u16>,
    fee_position: Option<u16>,
    tapleaf: BTreeMap<InputRole, LeafRole>,
    form: TransactionForm,
    version: TargetTransactionVersion,
}

impl ShapeAbi {
    /// The shape this describes.
    #[must_use]
    pub const fn shape(&self) -> CompactAshShape {
        self.shape
    }

    /// The exact layout the backend fixed for it.
    #[must_use]
    pub const fn layout(&self) -> &ConcreteLayout {
        &self.layout
    }

    /// The half-open ASH input range.
    #[must_use]
    pub const fn ash_range(&self) -> (u16, u16) {
        self.ash_range
    }

    /// The half-open sponsor suffix, empty for a sponsorless shape.
    #[must_use]
    pub const fn sponsor_range(&self) -> (u16, u16) {
        self.sponsor_range
    }

    /// Where the successor ASH output sits.
    #[must_use]
    pub const fn successor_position(&self) -> u16 {
        self.successor_position
    }

    /// Where the sponsor-change output sits, if the shape carries one.
    #[must_use]
    pub const fn sponsor_change_position(&self) -> Option<u16> {
        self.sponsor_change_position
    }

    /// Where the target's fee output sits, if the shape carries one.
    #[must_use]
    pub const fn fee_position(&self) -> Option<u16> {
        self.fee_position
    }

    /// Which committed leaf each input role spends through.
    #[must_use]
    pub const fn tapleaf(&self) -> &BTreeMap<InputRole, LeafRole> {
        &self.tapleaf
    }

    /// Which reviewed transaction form this shape takes.
    #[must_use]
    pub const fn form(&self) -> TransactionForm {
        self.form
    }

    /// The version the ABI builds this shape at.
    #[must_use]
    pub const fn version(&self) -> TargetTransactionVersion {
        self.version
    }
}

/// The candidate compact-ASH transaction ABI (§15.10).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateTransactionAbi {
    contract: TargetContractVersion,
    leaf_version: LeafVersion,
    pin: PinnedAshInstance,
    tree: CommittedTree,
    shapes: BTreeMap<CompactAshShape, ShapeAbi>,
    coordinator: CoordinatorRule,
    ordering: CanonicalOrdering,
    witness_order: Vec<WitnessItem>,
    sequence: SequenceConstraint,
    lock_time: u32,
    representation: AshRepresentationSelection,
    value_policy: ExplicitValuePolicy,
    internal_key_policy: InternalKeyPolicy,
    key_path: KeyPathPolicy,
    assumptions: BTreeSet<ConstructorAssumption>,
    symbols: DeploymentSymbols,
    witness_roles: BTreeMap<LeafRole, WitnessRole>,
    placements: BTreeMap<RelationCaseKey, ConcreteRelationPlacement>,
    forms: BTreeMap<TransactionForm, TransactionFormReview>,
    package_limits: PackageLimits,
    bounds: BTreeMap<ResourceDimension, u64>,
    obligations: OutstandingAbiObligations,
}

impl CandidateTransactionAbi {
    /// The reviewed contract revision the ABI is bound to.
    #[must_use]
    pub const fn contract(&self) -> TargetContractVersion {
        self.contract
    }

    /// The leaf version every committed leaf carries.
    #[must_use]
    pub const fn leaf_version(&self) -> LeafVersion {
        self.leaf_version
    }

    /// The deployed instance this ABI builds against.
    #[must_use]
    pub const fn pin(&self) -> &PinnedAshInstance {
        &self.pin
    }

    /// The hashed committed tree.
    #[must_use]
    pub const fn tree(&self) -> &CommittedTree {
        &self.tree
    }

    /// Every admitted shape's ABI, in canonical order.
    #[must_use]
    pub const fn shapes(&self) -> &BTreeMap<CompactAshShape, ShapeAbi> {
        &self.shapes
    }

    /// One shape's ABI.
    #[must_use]
    pub fn shape(&self, shape: CompactAshShape) -> Option<&ShapeAbi> {
        self.shapes.get(&shape)
    }

    /// The coordinator rule.
    #[must_use]
    pub const fn coordinator(&self) -> CoordinatorRule {
        self.coordinator
    }

    /// How each input family is ordered.
    #[must_use]
    pub const fn ordering(&self) -> CanonicalOrdering {
        self.ordering
    }

    /// The witness item order, bottom of the stack first.
    #[must_use]
    pub fn witness_order(&self) -> &[WitnessItem] {
        &self.witness_order
    }

    /// The sequence constraint.
    #[must_use]
    pub const fn sequence(&self) -> SequenceConstraint {
        self.sequence
    }

    /// The lock-time field every candidate transaction carries.
    #[must_use]
    pub const fn lock_time(&self) -> u32 {
        self.lock_time
    }

    /// The selected ASH representation.
    #[must_use]
    pub const fn representation(&self) -> AshRepresentationSelection {
        self.representation
    }

    /// The explicit value policy.
    #[must_use]
    pub const fn value_policy(&self) -> ExplicitValuePolicy {
        self.value_policy
    }

    /// The internal-key policy, which admits one value.
    #[must_use]
    pub const fn internal_key_policy(&self) -> InternalKeyPolicy {
        self.internal_key_policy
    }

    /// The key-path policy, which admits one value.
    #[must_use]
    pub const fn key_path(&self) -> KeyPathPolicy {
        self.key_path
    }

    /// Every assumption the constructor rests on.
    #[must_use]
    pub const fn assumptions(&self) -> &BTreeSet<ConstructorAssumption> {
        &self.assumptions
    }

    /// The public deployment constants.
    #[must_use]
    pub const fn symbols(&self) -> &DeploymentSymbols {
        &self.symbols
    }

    /// Every leaf's witness handoff, carried from the link.
    #[must_use]
    pub const fn witness_roles(&self) -> &BTreeMap<LeafRole, WitnessRole> {
        &self.witness_roles
    }

    /// Every relation-case's concrete placement.
    #[must_use]
    pub const fn placements(&self) -> &BTreeMap<RelationCaseKey, ConcreteRelationPlacement> {
        &self.placements
    }

    /// The reviewed transaction-form verdicts.
    #[must_use]
    pub const fn target_policy_status(&self) -> &BTreeMap<TransactionForm, TransactionFormReview> {
        &self.forms
    }

    /// The package limits the topology-restricted path imposes.
    #[must_use]
    pub const fn package_limits(&self) -> PackageLimits {
        self.package_limits
    }

    /// The candidate bounds, per dimension.
    ///
    /// The linked programs' own charged figures, carried rather than
    /// recomputed. Whole-transaction dimensions are absent here and
    /// settled per constructed transaction, because a weight is a fact
    /// about a transaction and not about an ABI.
    #[must_use]
    pub const fn candidate_bounds(&self) -> &BTreeMap<ResourceDimension, u64> {
        &self.bounds
    }

    /// The obligations this ABI carries.
    #[must_use]
    pub const fn outstanding_obligations(&self) -> &OutstandingAbiObligations {
        &self.obligations
    }

    /// This artifact's status (§15.10).
    ///
    /// Always [`AbiStatus::Candidate`], and read-only.
    #[must_use]
    pub const fn status(&self) -> AbiStatus {
        AbiStatus::Candidate
    }
}

/// Derive the candidate ABI (§15.3).
///
/// # Errors
///
/// [`TransactionRefusal::BundleIsNotACandidate`] for a bundle claiming
/// more than a prototype link,
/// [`TransactionRefusal::ContractRevisionMismatch`] when the bundle and
/// the target disagree about the reviewed revision, and every refusal
/// the pin check, the tree hashing, and the layout reading raise. No
/// partial ABI is returned by any path.
pub fn derive_candidate_abi(
    target: &ReviewedElementsTapscriptDefinition,
    bundle: &CandidateLinkedBundle,
    pin: PinnedAshInstance,
) -> Result<CandidateTransactionAbi, TransactionRefusal> {
    if bundle.status() != linker::LinkedArtifactStatus::Prototype {
        return Err(TransactionRefusal::BundleIsNotACandidate);
    }
    if bundle.constructor().contract() != target.definition().version() {
        return Err(TransactionRefusal::ContractRevisionMismatch);
    }
    pin.check_against(bundle)?;

    let tree = commit_tree(target, bundle)?;
    let symbols = read_symbols(bundle)?;

    let mut shapes = BTreeMap::new();
    for shape in bundle.shapes().shapes() {
        shapes.insert(shape, shape_abi(bundle, shape)?);
    }

    let coordinator = coordinator_rule(&shapes)?;

    let mut bounds = BTreeMap::new();
    for program in bundle.programs().values() {
        for (dimension, charged) in program.dimensions() {
            let entry = bounds.entry(*dimension).or_insert(0);
            *entry = (*entry).max(*charged);
        }
    }

    Ok(CandidateTransactionAbi {
        contract: bundle.constructor().contract(),
        leaf_version: bundle.constructor().leaf_version(),
        pin,
        tree,
        shapes,
        coordinator,
        ordering: CanonicalOrdering::AscendingOutpoint,
        // Bottom of the stack first, which is the order a spend pushes
        // them and the reverse of the order the target pops them.
        witness_order: vec![WitnessItem::LeafScript, WitnessItem::ControlBlock],
        sequence: SequenceConstraint::FinalOnEveryInput,
        lock_time: 0,
        representation: bundle.constructor().representation(),
        value_policy: bundle.constructor().value_policy(),
        internal_key_policy: bundle.constructor().internal_key_policy(),
        key_path: bundle.constructor().key_path(),
        assumptions: bundle.constructor().assumptions().clone(),
        symbols,
        witness_roles: bundle.witness_roles().clone(),
        placements: bundle.placements().clone(),
        forms: bundle.target_projection().forms().clone(),
        package_limits: PackageLimits::reviewed(),
        bounds,
        obligations: obligations(bundle),
    })
}

/// The obligations an ABI over this bundle carries.
fn obligations(bundle: &CandidateLinkedBundle) -> OutstandingAbiObligations {
    let mut rest = BTreeSet::from([
        AbiObligation::ClearLifecycleAbsent,
        AbiObligation::TargetExecutionEvidenceAbsent,
    ]);
    if bundle
        .outstanding_obligations()
        .holds(LinkObligation::InternalKeyUnspendabilityUnverified)
    {
        rest.insert(AbiObligation::InternalKeyUnspendabilityUnverified);
    }
    OutstandingAbiObligations {
        least: AbiObligation::PinnedOutputKeyUnverifiedAgainstTree,
        rest,
    }
}

/// The coordinator rule every shape's layout agrees on.
fn coordinator_rule(
    shapes: &BTreeMap<CompactAshShape, ShapeAbi>,
) -> Result<CoordinatorRule, TransactionRefusal> {
    let mut index = None;
    for abi in shapes.values() {
        let placed = abi
            .layout
            .input_run(InputRole::Coordinator)
            .ok_or(TransactionRefusal::MissingInputRole {
                shape: abi.shape,
                role: InputRole::Coordinator,
            })?
            .first();
        // §10.3 fixes the coordinator at input zero, and the coordinator
        // program requires that index of itself. A layout that placed it
        // anywhere else would emit a transaction whose anchor leaf
        // rejects, so the disagreement is a refusal rather than a
        // parameter.
        if placed != 0 {
            return Err(TransactionRefusal::CoordinatorNotAtAnchor { placed });
        }
        match index {
            None => index = Some(placed),
            Some(fixed) if fixed == placed => {}
            Some(_) => return Err(TransactionRefusal::CoordinatorNotAtAnchor { placed }),
        }
    }
    Ok(CoordinatorRule {
        index: index.unwrap_or_default(),
    })
}

/// Read one shape's ABI from its linked layout.
fn shape_abi(
    bundle: &CandidateLinkedBundle,
    shape: CompactAshShape,
) -> Result<ShapeAbi, TransactionRefusal> {
    let layout = bundle
        .layouts()
        .get(&shape)
        .ok_or(TransactionRefusal::MissingLayout(shape))?
        .clone();

    let coordinator_run =
        layout
            .input_run(InputRole::Coordinator)
            .ok_or(TransactionRefusal::MissingInputRole {
                shape,
                role: InputRole::Coordinator,
            })?;
    let member_run = layout.input_run(InputRole::Member);
    let sponsor_run = layout.input_run(InputRole::Sponsor);

    let ash_end = member_run.map_or(coordinator_run.end(), InputPlacement::end);
    let sponsor_range = sponsor_run.map_or((ash_end, ash_end), |run| (run.first(), run.end()));

    let successor_position = layout.output_position(OutputRole::Successor).ok_or(
        TransactionRefusal::MissingOutputRole {
            shape,
            role: OutputRole::Successor,
        },
    )?;

    let coordinator_leaf = LeafRole::Coordinator { shape };
    let member_leaf = LeafRole::Member {
        ash_inputs: shape.ash_inputs(),
    };
    for leaf in [coordinator_leaf, member_leaf] {
        if bundle.taptree().recipes().contains_key(&leaf) {
            continue;
        }
        // A shape with one ASH input has no member position, so its
        // member leaf need not be committed; any other absence is a
        // shape whose family cannot be spent.
        if leaf == member_leaf && member_run.is_none() {
            continue;
        }
        return Err(TransactionRefusal::MissingLeaf(leaf));
    }

    let mut tapleaf = BTreeMap::from([(InputRole::Coordinator, coordinator_leaf)]);
    if member_run.is_some() {
        tapleaf.insert(InputRole::Member, member_leaf);
    }

    let form = if sponsor_range.0 == sponsor_range.1 {
        TransactionForm::Sponsorless
    } else {
        TransactionForm::Sponsored
    };

    Ok(ShapeAbi {
        shape,
        ash_range: (coordinator_run.first(), ash_end),
        sponsor_range,
        successor_position,
        sponsor_change_position: layout.output_position(OutputRole::SponsorChange),
        fee_position: layout.output_position(OutputRole::TargetFee),
        tapleaf,
        form,
        version: match form {
            TransactionForm::Sponsorless => TargetTransactionVersion::TopologyRestricted,
            _ => TargetTransactionVersion::Standard,
        },
        layout,
    })
}

/// Read the public deployment constants from the linked census.
fn read_symbols(bundle: &CandidateLinkedBundle) -> Result<DeploymentSymbols, TransactionRefusal> {
    let closed_asset = asset(bundle, BundleSymbol::ClosedAsset, "closed asset")?;
    let reserve_asset = asset(bundle, BundleSymbol::ReserveAsset, "reserve asset")?;

    let sponsor_change_program = match value(
        bundle,
        BundleSymbol::SponsorChangeProgram,
        "sponsor change program",
    )? {
        SymbolValue::WitnessProgram(item) => item.bytes().to_vec(),
        _ => {
            return Err(TransactionRefusal::MalformedDeploymentSymbol {
                symbol: "sponsor change program",
            });
        }
    };
    let sponsor_change_version = match value(
        bundle,
        BundleSymbol::SponsorChangeProgramVersion,
        "sponsor change version",
    )? {
        SymbolValue::ScriptNumber(number) => {
            u8::try_from(*number).map_err(|_| TransactionRefusal::MalformedDeploymentSymbol {
                symbol: "sponsor change version",
            })?
        }
        _ => {
            return Err(TransactionRefusal::MalformedDeploymentSymbol {
                symbol: "sponsor change version",
            });
        }
    };

    Ok(DeploymentSymbols {
        closed_asset,
        reserve_asset,
        sponsor_change_program,
        sponsor_change_version,
    })
}

/// One symbol's linked value.
fn value<'bundle>(
    bundle: &'bundle CandidateLinkedBundle,
    symbol: BundleSymbol,
    rendered: &'static str,
) -> Result<&'bundle SymbolValue, TransactionRefusal> {
    bundle
        .definitions()
        .definition(symbol)
        .map(linker::SymbolDefinition::value)
        .ok_or(TransactionRefusal::MissingDeploymentSymbol { symbol: rendered })
}

/// One symbol's linked value, read as an asset identifier.
fn asset(
    bundle: &CandidateLinkedBundle,
    symbol: BundleSymbol,
    rendered: &'static str,
) -> Result<AssetId, TransactionRefusal> {
    let SymbolValue::Asset(item) = value(bundle, symbol, rendered)? else {
        return Err(TransactionRefusal::MalformedDeploymentSymbol { symbol: rendered });
    };
    let item: &StackItem = item;
    AssetId::from_slice(item.bytes())
        .map_err(|_| TransactionRefusal::MalformedDeploymentSymbol { symbol: rendered })
}
