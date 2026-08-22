//! The candidate relocatable explicit live-transfer bundle (§1.12, §11).
//!
//! # What this module emits
//!
//! [`emit_candidate_live_bundle`] assembles one
//! [`CandidateRelocatableLiveTransferBundle`]: the validated
//! live-transfer plan, the static live-receipt constructor, the candidate
//! shape set, the typed coordinator and member programs with their
//! witness roles and exact resource projections, every shape's family
//! ranges, the typed symbols and relocations, the selected pattern
//! identities with the residuals they carry, and the candidate lifecycle.
//! Nothing is emitted before each program has been walked and held to
//! §10.9, and before each shape's positions have been shown to be covered
//! exactly once.
//!
//! # What is consumed from Phase 4, and what had to be new
//!
//! §1.1 asks that a Phase-4 boundary change only where live transfer
//! presents a fact it cannot express, and that each generalization record
//! the fact. The vocabulary of [`crate::bundle`] is consumed whole:
//! [`SymbolBinding`], [`SymbolWidth`], [`RelocationEncoding`],
//! [`TargetRole`], [`SubstitutionMode`], [`WitnessComponent`],
//! [`FieldSide`], [`BackendArtifactStatus`], [`InternalKeyPolicy`],
//! [`KeyPathPolicy`] and [`ConstructorAssumption`] are the same types the
//! compact-ASH bundle uses, so a linker reads one census of each.
//!
//! One of them gained a member. [`TargetRole::SignatureKeyOperand`] did
//! not exist because compact ASH is permissionless and no leaf of it
//! carries a key; §10.2 pushes the committed owner's key as the operand
//! of a signature primitive, which is not a comparand of any introspected
//! field. The alternative was a second role enum, and then two censuses
//! for one linker.
//!
//! Three types are new, and each is new because it is keyed to something
//! compact ASH does not have: [`LiveBundleSymbol`], because the symbol
//! set differs; [`LiveRelocationSite`] and [`LiveIntrospectionReference`],
//! because a site names a [`LiveTransferLeafRole`], which is a different
//! type from [`crate::bundle::LeafRole`] for the reason §11.3 gives — the
//! leaves of two representations are disjoint sets and a leaf of one must
//! have no way of reaching the other's constructor.
//!
//! # Explicit only, and it says so
//!
//! [`emit_candidate_live_bundle`] refuses a constructor built for the
//! private-committed plan. §11.3 keeps the two coordinators distinct
//! until one complete typed proof establishes a sound shared program, and
//! no such proof exists; §10.6's conservation is not built at all. A
//! bundle that accepted the private constructor would be advertising
//! programs whose representation-specific obligation is missing, which is
//! the one thing a bundle must not do quietly.
//!
//! # Nothing here is final and no digest is minted
//!
//! [`BackendArtifactStatus`] is read and never written, the lifecycle the
//! bundle carries is the constructor's structurally-incomplete one, and
//! §1.13 names `LiveTransferBundleHash` among the identities Guide 13
//! mints none of. The bundle's identity is the typed value.
//!
//! The relocations are found the way [`crate::bundle`] finds them, and
//! for the same reason: the programs are rebuilt with exactly one symbol
//! replaced and the positions that moved are taken. Searching for bytes
//! that resemble a symbol would confuse two symbols that resolved to the
//! same literal, and a version number is indistinguishable from an index.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;

use compiler::live_transfer_plan::{
    LiveTransferRepresentationPlan, ValidatedLiveTransferOperationPlan,
};
use target_elements::{
    EncodingClass, LeafVersion, OpcodeId, PayloadWidth, ResourceDimension,
    ReviewedElementsTapscriptDefinition, TargetContractVersion, TargetEvidenceRequirementId,
};

use crate::authorization::OwnerProfileDisposition;
use crate::bundle::{
    BackendArtifactStatus, ConstructorAssumption, FieldSide, InternalKeyPolicy, KeyPathPolicy,
    RelocationEncoding, ResourceObligation, SubstitutionMode, SymbolBinding, SymbolWidth,
    TargetRole, WitnessComponent,
};
use crate::error::TapscriptError;
use crate::instruction::{StackItem, TapscriptInstruction};
use crate::live_constructor::{
    CandidateTransferLifecycle, LiveConstructorRefusal, LiveTransferLeafRole, OwnerKey,
    OwnerKeyRejection, StaticLiveReceiptConstructor, derive_live_receipt_constructor,
};
use crate::live_pattern::{
    FinalStackDefect, LiveProgramRefusal, LiveTransferPatternId, LiveTransferSymbols,
    RecognitionResidual, final_stack_defects, has_member_position, live_coordinator_program,
    live_member_program, live_owner_profile_disposition, live_program_precondition,
    live_transfer_patterns,
};
use crate::live_plan::{
    CompleteFamilyRanges, FamilyRangeDefect, family_range_defects, live_family_ranges,
};
use crate::live_shape::{LiveTransferShape, LiveTransferShapeSet};
use crate::program::TapscriptProgram;
use crate::stack::{AbstractLimits, resource_projection, validate_program};

// --- Symbols ----------------------------------------------------------

/// One typed link-time role of the candidate live-transfer bundle.
///
/// Both halves of §11.2's list are present: the roles a later layer must
/// resolve, and the roles this bundle itself defines and hands the linker
/// to place.
///
/// # The live-receipt constructor's own program is absent
///
/// Compact ASH names its constructor's program as a symbol nobody settles
/// because its leaves *compare* against it, read from the input they are
/// spending, and the linker's cycle analysis needs that edge. No
/// live-transfer fragment does: §10.1's recognition rests on the leaf
/// commitment instead — a leaf runs only from a taptree its input's
/// program commits to — so there is no comparison and no read to record.
/// A symbol with neither would be a reserved field, which §1.10 refuses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiveBundleSymbol {
    /// The explicit protocol asset every receipt carries.
    ProtocolAsset,
    /// The explicit reserve asset every sponsor and fee role carries.
    ReserveAsset,
    /// The version a destination's program is read at.
    DestinationProgramVersion,
    /// The sponsor-change role's witness program.
    SponsorChangeProgram,
    /// The version the sponsor-change program is read at.
    SponsorChangeProgramVersion,
    /// The target fee role's program digest.
    TargetFeeRoleProgramDigest,
    /// The committed owner's public key.
    ///
    /// Defined by this bundle rather than resolved at link: the
    /// constructor commits one owner (§7.2), the bundle carries that
    /// constructor, and §7.6 gives a request no parameter through which
    /// another key could arrive.
    OwnerPublicKey,
    /// The unspendable taproot internal key (§7.5).
    UnspendableInternalKey,
    /// The reviewed tapscript leaf version.
    TargetLeafVersion,
    /// The selected owner sighash profile (§11.2).
    ///
    /// A symbol with no relocation, and it is here rather than omitted
    /// because it is genuinely consumed: [`CandidateRelocatableLiveTransferBundle::sighash_profile`]
    /// carries the review's answer about it. It reaches no site because
    /// the profile decides which message the target builds, and no byte
    /// an authorization fragment pushes depends on that message — the
    /// fragment pushes a key and verifies whatever the witness offers
    /// against it.
    SelectedSighashProfile,
    /// The candidate receipt-input bound.
    CandidateReceiptInputBound,
    /// The candidate receipt-output bound.
    CandidateReceiptOutputBound,
    /// The candidate sponsor-region bound.
    CandidateSponsorBound,
    /// The coordinator program of one shape, defined here.
    CoordinatorProgram {
        /// The shape whose coordinator this is.
        shape: LiveTransferShape,
    },
    /// The member program of one receipt-input count, defined here.
    MemberProgram {
        /// The receipt-input count whose member range it bounds.
        receipt_inputs: u8,
    },
}

/// The symbols some emitted program pushes a literal for.
///
/// Exactly the ones a relocation can have program sites for, which is why
/// the probe loop reads this list and not [`LiveBundleSymbol`]'s whole
/// census: probing a symbol no fragment pushes would rebuild every
/// program only to find nothing had moved.
const PROGRAM_SYMBOLS: &[LiveBundleSymbol] = &[
    LiveBundleSymbol::ProtocolAsset,
    LiveBundleSymbol::ReserveAsset,
    LiveBundleSymbol::DestinationProgramVersion,
    LiveBundleSymbol::SponsorChangeProgram,
    LiveBundleSymbol::SponsorChangeProgramVersion,
    LiveBundleSymbol::TargetFeeRoleProgramDigest,
    LiveBundleSymbol::OwnerPublicKey,
];

/// What one symbol is, independent of where it is placed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LiveSymbolEntry {
    binding: SymbolBinding,
    width: SymbolWidth,
    encoding: RelocationEncoding,
}

impl LiveSymbolEntry {
    /// Who settles the symbol's value.
    #[must_use]
    pub const fn binding(self) -> SymbolBinding {
        self.binding
    }

    /// What the symbol occupies.
    #[must_use]
    pub const fn width(self) -> SymbolWidth {
        self.width
    }

    /// How the symbol is carried.
    #[must_use]
    pub const fn encoding(self) -> RelocationEncoding {
        self.encoding
    }
}

// --- Relocations ------------------------------------------------------

/// Where one live-transfer relocation applies.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LiveRelocationSite {
    /// Exact push positions inside one emitted program.
    ProgramInstructions {
        /// The leaf whose program carries them.
        leaf: LiveTransferLeafRole,
        /// The exact instruction indices, non-empty.
        indices: BTreeSet<usize>,
    },
    /// A binding of the static constructor, outside any program.
    ConstructorBinding,
}

/// One relocation of one symbol.
///
/// Every item §11.2's linker needs: the source symbol, the target role,
/// the semantic location, the width, the encoding, the exact
/// multiplicity, and how the value reaches the site. The multiplicity is
/// computed when the relocation is built rather than declared beside it.
///
/// Every relocation this bundle emits substitutes into the typed program
/// before serialization, so a resolution whose width differs from the one
/// laid out against costs nothing and corrupts nothing. There is no byte
/// patch here at all — not refused at construction, but unreachable,
/// because nothing in this module states one.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LiveRelocation {
    symbol: LiveBundleSymbol,
    role: TargetRole,
    site: LiveRelocationSite,
    width: SymbolWidth,
    encoding: RelocationEncoding,
    multiplicity: NonZeroUsize,
    substitution: SubstitutionMode,
}

impl LiveRelocation {
    /// The symbol whose value this relocation places.
    #[must_use]
    pub const fn symbol(&self) -> LiveBundleSymbol {
        self.symbol
    }

    /// What the placed value is used as.
    #[must_use]
    pub const fn role(&self) -> TargetRole {
        self.role
    }

    /// The semantic location.
    #[must_use]
    pub const fn site(&self) -> &LiveRelocationSite {
        &self.site
    }

    /// The symbol's width at this site.
    #[must_use]
    pub const fn width(&self) -> SymbolWidth {
        self.width
    }

    /// How the value is carried at this site.
    #[must_use]
    pub const fn encoding(&self) -> RelocationEncoding {
        self.encoding
    }

    /// How many places this relocation covers.
    #[must_use]
    pub const fn multiplicity(&self) -> NonZeroUsize {
        self.multiplicity
    }

    /// How the value reaches the site.
    #[must_use]
    pub const fn substitution(&self) -> &SubstitutionMode {
        &self.substitution
    }
}

/// One leaf's dependency on a value it reads from the target.
///
/// The counterpart of [`LiveRelocation`] for a value no layer supplies.
/// The live-transfer leaves have none today — see [`LiveBundleSymbol`]
/// for why the constructor's own program is not among the symbols — and
/// the type exists so that a leaf which acquired one would have somewhere
/// to record it rather than acquiring it invisibly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LiveIntrospectionReference {
    symbol: LiveBundleSymbol,
    leaf: LiveTransferLeafRole,
    role: TargetRole,
    sites: NonZeroUsize,
}

impl LiveIntrospectionReference {
    /// The symbol whose value the program reads.
    #[must_use]
    pub const fn symbol(&self) -> LiveBundleSymbol {
        self.symbol
    }

    /// The leaf whose program reads it.
    #[must_use]
    pub const fn leaf(&self) -> LiveTransferLeafRole {
        self.leaf
    }

    /// What the read value is, and which side it is read from.
    #[must_use]
    pub const fn role(&self) -> TargetRole {
        self.role
    }

    /// How many places in that program read it.
    #[must_use]
    pub const fn sites(&self) -> NonZeroUsize {
        self.sites
    }
}

// --- Leaves -----------------------------------------------------------

/// One emitted leaf: its program, its cost, and its witness role.
///
/// All three computed from the program rather than declared beside it.
/// The witness role in particular is established by walking: the program
/// is scheduled from the one-item precondition §10.2 leaves it, and the
/// data-item count is that precondition's depth, so the record and the
/// schedule cannot disagree about what a spender supplies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveLeafProgram {
    leaf: LiveTransferLeafRole,
    program: TapscriptProgram,
    resources: BTreeMap<ResourceDimension, u64>,
    witness: BTreeSet<WitnessComponent>,
    data_items: usize,
    shapes: BTreeSet<LiveTransferShape>,
}

impl LiveLeafProgram {
    /// The leaf this record belongs to.
    #[must_use]
    pub const fn leaf(&self) -> LiveTransferLeafRole {
        self.leaf
    }

    /// The typed program.
    #[must_use]
    pub const fn program(&self) -> &TapscriptProgram {
        &self.program
    }

    /// The exact resource cost, by dimension.
    #[must_use]
    pub const fn resources(&self) -> &BTreeMap<ResourceDimension, u64> {
        &self.resources
    }

    /// Every component a spend of this leaf carries.
    #[must_use]
    pub const fn witness(&self) -> &BTreeSet<WitnessComponent> {
        &self.witness
    }

    /// How many witness data items the program consumes.
    ///
    /// One for every live-transfer leaf: the owner signature of §10.2,
    /// established by scheduling the program from that precondition
    /// rather than by assertion.
    #[must_use]
    pub const fn data_items(&self) -> usize {
        self.data_items
    }

    /// Every admitted shape this leaf serves, in canonical order.
    #[must_use]
    pub const fn shapes(&self) -> &BTreeSet<LiveTransferShape> {
        &self.shapes
    }
}

/// Why one member leaf may be shared by several shapes (§11.3).
///
/// One ground, and it is a recomputation rather than an argument: each
/// sharing shape's own member program is rebuilt from that shape and
/// compared instruction by instruction with the leaf already admitted for
/// the receipt-input count. Equal typed instructions is equal
/// enforcement, because a program is nothing but its instructions.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LiveSharedLeafProof {
    leaf: LiveTransferLeafRole,
    shapes: BTreeSet<LiveTransferShape>,
}

impl LiveSharedLeafProof {
    /// The shared leaf.
    #[must_use]
    pub const fn leaf(&self) -> LiveTransferLeafRole {
        self.leaf
    }

    /// Every shape the leaf serves, in canonical order.
    #[must_use]
    pub const fn shapes(&self) -> &BTreeSet<LiveTransferShape> {
        &self.shapes
    }
}

// --- Refusals ---------------------------------------------------------

/// Why a candidate live-transfer bundle was not emitted.
///
/// Construction failure, never target rejection (§1.11): every variant
/// names something wrong with what this backend was asked to build.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LiveBundleRefusal {
    /// The constructor is built for a representation this bundle does not
    /// emit.
    ///
    /// §11.3 keeps the explicit and private coordinators distinct until
    /// one complete typed proof admits a shared program, and §10.6's
    /// conservation is not built. Accepting the private constructor would
    /// advertise programs whose representation-specific obligation is
    /// missing.
    RepresentationNotExplicit {
        /// The representation the constructor selected.
        selected: LiveTransferRepresentationPlan,
    },
    /// A program or fragment did not assemble.
    Program(LiveProgramRefusal),
    /// A fragment builder refused.
    Fragment(TapscriptError),
    /// A probe constructor could not be derived.
    ///
    /// Only reachable if the reference constructor's own parts stopped
    /// deriving one, which would be a defect in this crate rather than a
    /// property of the plan.
    Probe(LiveConstructorRefusal),
    /// A probe owner key was not admissible metadata.
    ///
    /// Likewise unreachable for a constructed [`OwnerKey`]: the probe
    /// keeps the approved encoding and its exact width and changes only
    /// the bytes. Refusing rather than unwrapping keeps that reasoning
    /// out of this module's soundness.
    ProbeOwnerRefused {
        /// What §7.2's gate reported.
        rejection: OwnerKeyRejection,
    },
    /// A leaf's program does not schedule from the precondition §10.2
    /// leaves it.
    LeafDoesNotSchedule {
        /// The leaf.
        leaf: LiveTransferLeafRole,
    },
    /// A leaf's program does not satisfy §10.9.
    ///
    /// Emission runs the final-stack census rather than trusting it: a
    /// bundle carrying a program with a surviving non-aborting failure or
    /// an unverified signature form would be publishing exactly the
    /// artifact §10.9 exists to refuse.
    LeafFailsTheFinalStackRule {
        /// The leaf.
        leaf: LiveTransferLeafRole,
        /// Every way it fails, in canonical order.
        defects: Vec<FinalStackDefect>,
    },
    /// A shape's positions are not covered exactly once.
    FamilyRangesIncomplete {
        /// The shape.
        shape: LiveTransferShape,
        /// Every defect the census reported.
        defects: Vec<FamilyRangeDefect>,
    },
    /// A shape sharing a member leaf does not rebuild to that leaf's
    /// exact program.
    SharedLeafRelationMismatch {
        /// The shared leaf.
        leaf: LiveTransferLeafRole,
        /// The shape whose rebuild disagreed.
        shape: LiveTransferShape,
    },
    /// A probe program did not have the same instruction count as the
    /// program it probes, so the difference does not locate a symbol.
    ProbeProgramShapeChanged {
        /// The leaf being probed.
        leaf: LiveTransferLeafRole,
        /// The symbol being probed.
        symbol: LiveBundleSymbol,
    },
    /// A relocation site is not a pushed literal.
    RelocationSiteIsNotAPush {
        /// The leaf.
        leaf: LiveTransferLeafRole,
        /// The instruction index.
        index: usize,
    },
    /// A relocation site is not in a context the symbol could occupy.
    RelocationSiteContextMismatch {
        /// The leaf.
        leaf: LiveTransferLeafRole,
        /// The instruction index.
        index: usize,
        /// The symbol placed there.
        symbol: LiveBundleSymbol,
    },
}

impl From<LiveProgramRefusal> for LiveBundleRefusal {
    fn from(refusal: LiveProgramRefusal) -> Self {
        Self::Program(refusal)
    }
}

impl From<TapscriptError> for LiveBundleRefusal {
    fn from(error: TapscriptError) -> Self {
        Self::Fragment(error)
    }
}

// --- The bundle -------------------------------------------------------

/// The candidate relocatable explicit live-transfer bundle (§1.12).
///
/// Every field is private and there is no public constructor: the sole
/// route to a value is [`emit_candidate_live_bundle`], which builds each
/// program, walks it, holds it to §10.9, and derives the placements from
/// the things they describe. A bundle assembled from arbitrary fields
/// would be a request rather than an emission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateRelocatableLiveTransferBundle {
    plan: ValidatedLiveTransferOperationPlan,
    constructor: StaticLiveReceiptConstructor,
    contract: TargetContractVersion,
    leaf_version: LeafVersion,
    profile: OwnerProfileDisposition,
    leaves: BTreeMap<LiveTransferLeafRole, LiveLeafProgram>,
    sharing: BTreeSet<LiveSharedLeafProof>,
    ranges: BTreeMap<LiveTransferShape, CompleteFamilyRanges>,
    symbols: BTreeMap<LiveBundleSymbol, LiveSymbolEntry>,
    unresolved: LiveTransferSymbols,
    relocations: BTreeSet<LiveRelocation>,
    introspections: BTreeSet<LiveIntrospectionReference>,
    outstanding_dimensions: BTreeMap<ResourceDimension, ResourceObligation>,
    patterns: BTreeSet<LiveTransferPatternId>,
    residuals: BTreeSet<RecognitionResidual>,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
    internal_key: InternalKeyPolicy,
    key_path: KeyPathPolicy,
    assumptions: BTreeSet<ConstructorAssumption>,
}

impl CandidateRelocatableLiveTransferBundle {
    /// The validated live-transfer plan this bundle was emitted for.
    #[must_use]
    pub const fn plan(&self) -> &ValidatedLiveTransferOperationPlan {
        &self.plan
    }

    /// The static live-receipt constructor.
    #[must_use]
    pub const fn constructor(&self) -> &StaticLiveReceiptConstructor {
        &self.constructor
    }

    /// The candidate shape set, read from the constructor.
    #[must_use]
    pub const fn shapes(&self) -> &LiveTransferShapeSet {
        self.constructor.shapes()
    }

    /// The representation every emitted program carries.
    #[must_use]
    pub const fn representation(&self) -> LiveTransferRepresentationPlan {
        LiveTransferRepresentationPlan::Explicit
    }

    /// The reviewed contract revision the programs are built against.
    #[must_use]
    pub const fn contract(&self) -> TargetContractVersion {
        self.contract
    }

    /// The leaf version every emitted leaf carries.
    #[must_use]
    pub const fn leaf_version(&self) -> LeafVersion {
        self.leaf_version
    }

    /// What the review establishes about the selected sighash profile.
    ///
    /// Retained rather than discarded because §11.6 has the linked
    /// candidate carry the selection, and because a caller reading a
    /// verified signature as authorization over §1.7's protected data is
    /// reading past this.
    #[must_use]
    pub const fn sighash_profile(&self) -> &OwnerProfileDisposition {
        &self.profile
    }

    /// Every emitted leaf, in canonical order.
    #[must_use]
    pub const fn leaves(&self) -> &BTreeMap<LiveTransferLeafRole, LiveLeafProgram> {
        &self.leaves
    }

    /// One leaf's record.
    #[must_use]
    pub fn leaf(&self, leaf: LiveTransferLeafRole) -> Option<&LiveLeafProgram> {
        self.leaves.get(&leaf)
    }

    /// Every recomputed member-leaf sharing proof.
    #[must_use]
    pub const fn sharing(&self) -> &BTreeSet<LiveSharedLeafProof> {
        &self.sharing
    }

    /// Every shape's family ranges, in canonical order.
    #[must_use]
    pub const fn family_ranges(&self) -> &BTreeMap<LiveTransferShape, CompleteFamilyRanges> {
        &self.ranges
    }

    /// The typed symbol table, in canonical order.
    #[must_use]
    pub const fn symbols(&self) -> &BTreeMap<LiveBundleSymbol, LiveSymbolEntry> {
        &self.symbols
    }

    /// The symbol values this bundle was laid out against.
    ///
    /// Not resolutions. The programs were assembled with these values so
    /// that their exact shape and cost could be measured, and the
    /// relocations record where a later layer replaces them.
    #[must_use]
    pub const fn unresolved_symbols(&self) -> &LiveTransferSymbols {
        &self.unresolved
    }

    /// Every relocation, in canonical order.
    #[must_use]
    pub const fn relocations(&self) -> &BTreeSet<LiveRelocation> {
        &self.relocations
    }

    /// Every relocation naming one symbol.
    pub fn relocations_for(
        &self,
        symbol: LiveBundleSymbol,
    ) -> impl Iterator<Item = &LiveRelocation> {
        self.relocations
            .iter()
            .filter(move |relocation| relocation.symbol == symbol)
    }

    /// Every value a program reads from the target, in canonical order.
    #[must_use]
    pub const fn introspections(&self) -> &BTreeSet<LiveIntrospectionReference> {
        &self.introspections
    }

    /// The resource dimensions this bundle does not establish, and who
    /// owes each of them.
    #[must_use]
    pub const fn outstanding_dimensions(&self) -> &BTreeMap<ResourceDimension, ResourceObligation> {
        &self.outstanding_dimensions
    }

    /// The selected proof-pattern identities, in canonical order.
    #[must_use]
    pub const fn selected_patterns(&self) -> &BTreeSet<LiveTransferPatternId> {
        &self.patterns
    }

    /// Every residual the selected patterns carry.
    ///
    /// The union rather than a summary: a consumer deciding how far to
    /// trust this bundle needs the whole list, and a bundle that reported
    /// none would be claiming §10.4 whole and the sighash profile
    /// reviewed.
    #[must_use]
    pub const fn residuals(&self) -> &BTreeSet<RecognitionResidual> {
        &self.residuals
    }

    /// Every target evidence requirement the bundle's correctness rests
    /// on, which no part of this wave discharges.
    #[must_use]
    pub const fn target_evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }

    /// The candidate lifecycle, read from the constructor.
    ///
    /// Structurally incomplete: its outstanding count is a [`NonZeroUsize`],
    /// so a bundle whose lifecycle was complete has no representation.
    #[must_use]
    pub const fn lifecycle(&self) -> &CandidateTransferLifecycle {
        self.constructor.lifecycle()
    }

    /// The inherited internal-key policy (§7.5).
    #[must_use]
    pub const fn internal_key(&self) -> InternalKeyPolicy {
        self.internal_key
    }

    /// The inherited key-path policy (§7.5).
    #[must_use]
    pub const fn key_path(&self) -> KeyPathPolicy {
        self.key_path
    }

    /// Every assumption the bundle rests on and does not establish.
    #[must_use]
    pub const fn assumptions(&self) -> &BTreeSet<ConstructorAssumption> {
        &self.assumptions
    }

    /// This artifact's status.
    ///
    /// Always [`BackendArtifactStatus::Prototype`], and read-only: the
    /// promotion the status vocabulary admits requires full operation
    /// evidence, the artifact carrying that evidence does not exist, and a
    /// setter or a field for it would let a bundle claim the promotion
    /// without the evidence.
    #[must_use]
    pub const fn status(&self) -> BackendArtifactStatus {
        BackendArtifactStatus::Prototype
    }

    /// The exact total encoded program bytes of every emitted leaf.
    #[must_use]
    pub fn total_script_bytes(&self) -> u64 {
        self.leaves
            .values()
            .filter_map(|leaf| leaf.resources.get(&ResourceDimension::ScriptBytes).copied())
            .fold(0, u64::saturating_add)
    }
}

// --- Emission ---------------------------------------------------------

/// Emit the candidate relocatable explicit live-transfer bundle.
///
/// The order is the one the checks depend on: the representation is
/// settled first, then every leaf is built and walked and held to §10.9,
/// then every shape's positions are covered, and only then are the
/// placements derived. A refusal returns no partial bundle.
///
/// # Errors
///
/// [`LiveBundleRefusal::RepresentationNotExplicit`] for a constructor
/// this bundle does not emit for;
/// [`LiveBundleRefusal::LeafFailsTheFinalStackRule`] and
/// [`LiveBundleRefusal::FamilyRangesIncomplete`] when an emitted program
/// or an admitted shape does not check out; and any of the assembly,
/// probe, and placement refusals [`LiveBundleRefusal`] names.
pub fn emit_candidate_live_bundle(
    target: &ReviewedElementsTapscriptDefinition,
    plan: &ValidatedLiveTransferOperationPlan,
    constructor: &StaticLiveReceiptConstructor,
    symbols: LiveTransferSymbols,
) -> Result<CandidateRelocatableLiveTransferBundle, LiveBundleRefusal> {
    if constructor.representation() != LiveTransferRepresentationPlan::Explicit {
        return Err(LiveBundleRefusal::RepresentationNotExplicit {
            selected: constructor.representation(),
        });
    }

    let shapes: Vec<LiveTransferShape> = constructor.shapes().shapes().collect();
    let (leaves, sharing) = leaf_programs(target, &symbols, constructor, &shapes)?;

    let mut ranges = BTreeMap::new();
    for shape in &shapes {
        let census = live_family_ranges(*shape);
        let defects = family_range_defects(&census);
        if !defects.is_empty() {
            return Err(LiveBundleRefusal::FamilyRangesIncomplete {
                shape: *shape,
                defects,
            });
        }
        ranges.insert(*shape, census);
    }

    let mut patterns = BTreeSet::new();
    let mut residuals = BTreeSet::new();
    let mut evidence = BTreeSet::new();
    for shape in &shapes {
        // Exactly the identities the shape calls for, because
        // `live_transfer_patterns` is what `patterns_for` describes: a
        // pattern silently absent from one shape's emission would be
        // visible here as an identity the union never gained.
        let census = live_transfer_patterns(target, &symbols, constructor, *shape)?;
        for (id, pattern) in &census {
            patterns.insert(*id);
            residuals.extend(pattern.residuals().iter().copied());
            evidence.extend(pattern.evidence().iter().copied());
        }
    }

    let relocations = all_relocations(target, &symbols, plan, constructor, &leaves)?;
    let symbol_table = symbol_table(target, &symbols, constructor, &leaves);

    Ok(CandidateRelocatableLiveTransferBundle {
        plan: plan.clone(),
        contract: constructor.contract(),
        leaf_version: constructor.leaf_version(),
        profile: live_owner_profile_disposition(target),
        constructor: constructor.clone(),
        leaves,
        sharing,
        ranges,
        symbols: symbol_table,
        unresolved: symbols,
        relocations,
        // Empty, and stated rather than omitted: no live-transfer
        // fragment reads a symbol's value from the target, so there is no
        // edge for the linker's cycle analysis to carry.
        introspections: BTreeSet::new(),
        outstanding_dimensions: outstanding_dimensions(),
        patterns,
        residuals,
        evidence,
        internal_key: constructor.internal_key(),
        key_path: constructor.key_path(),
        assumptions: constructor.assumptions().clone(),
    })
}

/// The dimensions this wave does not establish, each with its owner.
fn outstanding_dimensions() -> BTreeMap<ResourceDimension, ResourceObligation> {
    BTreeMap::from([
        (
            ResourceDimension::PeakStackItems,
            ResourceObligation::AbstractExecutionWalk,
        ),
        (
            ResourceDimension::StackElementBytes,
            ResourceObligation::AbstractExecutionWalk,
        ),
        (
            ResourceDimension::ControlPathDepth,
            ResourceObligation::Linker,
        ),
        (
            ResourceDimension::WitnessBytes,
            ResourceObligation::CompleteTransaction,
        ),
        (
            ResourceDimension::TransactionWeight,
            ResourceObligation::CompleteTransaction,
        ),
        (
            ResourceDimension::PackageLimit,
            ResourceObligation::CompleteTransaction,
        ),
    ])
}

// --- Leaves -----------------------------------------------------------

/// Build every leaf, deduplicating the shared member leaves.
fn leaf_programs(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &LiveTransferSymbols,
    constructor: &StaticLiveReceiptConstructor,
    shapes: &[LiveTransferShape],
) -> Result<
    (
        BTreeMap<LiveTransferLeafRole, LiveLeafProgram>,
        BTreeSet<LiveSharedLeafProof>,
    ),
    LiveBundleRefusal,
> {
    let representation = constructor.representation();
    let mut leaves: BTreeMap<LiveTransferLeafRole, LiveLeafProgram> = BTreeMap::new();
    let mut member_shapes: BTreeMap<u8, BTreeSet<LiveTransferShape>> = BTreeMap::new();

    for shape in shapes {
        let role = LiveTransferLeafRole::Coordinator {
            representation,
            shape: *shape,
        };
        let program = live_coordinator_program(target, symbols, constructor, *shape)?;
        leaves.insert(
            role,
            leaf_program(target, role, program, BTreeSet::from([*shape]))?,
        );

        if !has_member_position(*shape) {
            continue;
        }
        let member_role = LiveTransferLeafRole::Member {
            representation,
            receipt_inputs: shape.receipt_inputs(),
        };
        let member = live_member_program(target, symbols, constructor, shape.receipt_inputs())?;
        member_shapes
            .entry(shape.receipt_inputs())
            .or_default()
            .insert(*shape);

        // §11.3 admits sharing only where the shared leaf enforces the
        // exact same relation for every shape using it. The proof is the
        // rebuild: this shape's own program, compared instruction by
        // instruction with the leaf already admitted for the count.
        match leaves.get(&member_role) {
            Some(existing) if existing.program != member => {
                return Err(LiveBundleRefusal::SharedLeafRelationMismatch {
                    leaf: member_role,
                    shape: *shape,
                });
            }
            Some(_) => {}
            None => {
                leaves.insert(
                    member_role,
                    leaf_program(target, member_role, member, BTreeSet::new())?,
                );
            }
        }
    }

    let mut sharing = BTreeSet::new();
    for (receipt_inputs, served) in member_shapes {
        let role = LiveTransferLeafRole::Member {
            representation,
            receipt_inputs,
        };
        if let Some(leaf) = leaves.get_mut(&role) {
            leaf.shapes.clone_from(&served);
        }
        if served.len() > 1 {
            sharing.insert(LiveSharedLeafProof {
                leaf: role,
                shapes: served,
            });
        }
    }

    Ok((leaves, sharing))
}

/// One leaf's record: its cost and its witness role, both computed.
fn leaf_program(
    target: &ReviewedElementsTapscriptDefinition,
    leaf: LiveTransferLeafRole,
    program: TapscriptProgram,
    shapes: BTreeSet<LiveTransferShape>,
) -> Result<LiveLeafProgram, LiveBundleRefusal> {
    let initial = live_program_precondition(target);
    validate_program(
        target,
        &program,
        &initial,
        AbstractLimits::for_target(target),
    )
    .map_err(|_| LiveBundleRefusal::LeafDoesNotSchedule { leaf })?;

    let defects = final_stack_defects(target, &program, &initial)
        .map_err(|_| LiveBundleRefusal::LeafDoesNotSchedule { leaf })?;
    if !defects.is_empty() {
        return Err(LiveBundleRefusal::LeafFailsTheFinalStackRule { leaf, defects });
    }

    let mut resources = resource_projection(target, &program);
    resources.insert(
        ResourceDimension::InitialStackItems,
        u64::try_from(initial.depth()).unwrap_or(u64::MAX),
    );

    Ok(LiveLeafProgram {
        leaf,
        resources,
        witness: BTreeSet::from([WitnessComponent::LeafScript, WitnessComponent::ControlBlock]),
        data_items: initial.depth(),
        program,
        shapes,
    })
}

// --- Relocations ------------------------------------------------------

/// The symbol set with exactly one symbol replaced.
///
/// The probe's value has to differ from the real one and nothing else has
/// to hold: the programs are rebuilt from the same builders over the same
/// shape, so their instruction counts are equal by construction and the
/// positions that moved are exactly the positions that symbol occupies.
fn probe_symbols(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &LiveTransferSymbols,
    symbol: LiveBundleSymbol,
) -> Result<LiveTransferSymbols, LiveBundleRefusal> {
    let flip =
        |item: &StackItem| -> Vec<u8> { item.bytes().iter().map(|byte| byte ^ 0xff).collect() };
    let other = |value: i64| -> i64 { if value == 1 { 2 } else { 1 } };

    let mut protocol = symbols.protocol_asset().bytes().to_vec();
    let mut reserve = symbols.reserve_asset().bytes().to_vec();
    let mut destination_version = symbols.destination_program_version();
    let mut change = symbols.sponsor_change_program().bytes().to_vec();
    let mut change_version = symbols.sponsor_change_version();
    let mut digest = symbols.fee_program_digest().bytes().to_vec();

    match symbol {
        LiveBundleSymbol::ProtocolAsset => protocol = flip(symbols.protocol_asset()),
        LiveBundleSymbol::ReserveAsset => reserve = flip(symbols.reserve_asset()),
        LiveBundleSymbol::DestinationProgramVersion => {
            destination_version = other(destination_version);
        }
        LiveBundleSymbol::SponsorChangeProgram => change = flip(symbols.sponsor_change_program()),
        LiveBundleSymbol::SponsorChangeProgramVersion => change_version = other(change_version),
        LiveBundleSymbol::TargetFeeRoleProgramDigest => digest = flip(symbols.fee_program_digest()),
        _ => {}
    }

    Ok(LiveTransferSymbols::new(
        target,
        protocol,
        reserve,
        destination_version,
        change,
        change_version,
        digest,
    )?)
}

/// The constructor with exactly the owner key replaced.
///
/// The owner is not a member of the symbol set, so probing it means
/// deriving a second constructor from the same plan, shapes, and leaves.
/// That is the right shape for the probe as well as the only available
/// one: the owner reaches a fragment only by way of a constructor that
/// admitted it (§7.6), so a probe that reached around the constructor
/// would be probing a route no request has.
fn probe_constructor(
    target: &ReviewedElementsTapscriptDefinition,
    plan: &ValidatedLiveTransferOperationPlan,
    constructor: &StaticLiveReceiptConstructor,
) -> Result<StaticLiveReceiptConstructor, LiveBundleRefusal> {
    let owner = constructor.owner();
    let flipped: Vec<u8> = owner.bytes().iter().map(|byte| byte ^ 0xff).collect();
    let probe = OwnerKey::new(constructor.owner_encoding(), owner.encoding(), flipped)
        .map_err(|rejection| LiveBundleRefusal::ProbeOwnerRefused { rejection })?;

    derive_live_receipt_constructor(
        target,
        plan,
        constructor.representation(),
        probe,
        constructor.shapes().clone(),
        constructor.leaves().collect(),
    )
    .map_err(LiveBundleRefusal::Probe)
}

/// One leaf's program, rebuilt from a probe symbol set and constructor.
fn rebuild(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &LiveTransferSymbols,
    constructor: &StaticLiveReceiptConstructor,
    leaf: LiveTransferLeafRole,
) -> Result<TapscriptProgram, LiveBundleRefusal> {
    Ok(match leaf {
        LiveTransferLeafRole::Coordinator { shape, .. } => {
            live_coordinator_program(target, symbols, constructor, shape)?
        }
        LiveTransferLeafRole::Member { receipt_inputs, .. } => {
            live_member_program(target, symbols, constructor, receipt_inputs)?
        }
    })
}

/// Every relocation of the bundle: program sites and constructor
/// bindings alike.
fn all_relocations(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &LiveTransferSymbols,
    plan: &ValidatedLiveTransferOperationPlan,
    constructor: &StaticLiveReceiptConstructor,
    leaves: &BTreeMap<LiveTransferLeafRole, LiveLeafProgram>,
) -> Result<BTreeSet<LiveRelocation>, LiveBundleRefusal> {
    let mut relocations = BTreeSet::new();
    let owner_probe = probe_constructor(target, plan, constructor)?;

    for symbol in PROGRAM_SYMBOLS {
        let (probed_symbols, probed_constructor) = if *symbol == LiveBundleSymbol::OwnerPublicKey {
            (symbols.clone(), &owner_probe)
        } else {
            (probe_symbols(target, symbols, *symbol)?, constructor)
        };

        for (role, leaf) in leaves {
            let rebuilt = rebuild(target, &probed_symbols, probed_constructor, *role)?;
            if rebuilt.len() != leaf.program.len() {
                return Err(LiveBundleRefusal::ProbeProgramShapeChanged {
                    leaf: *role,
                    symbol: *symbol,
                });
            }
            let indices: BTreeSet<usize> = leaf
                .program
                .instructions()
                .iter()
                .zip(rebuilt.instructions())
                .enumerate()
                .filter(|(_, (here, there))| here != there)
                .map(|(index, _)| index)
                .collect();
            if indices.is_empty() {
                continue;
            }

            // One symbol can hold two roles in one program: the protocol
            // asset is compared with an input asset in the recognition
            // fragment and with a destination's asset in the closure, and
            // those are different target roles even though the pushed
            // bytes are one symbol. So the sites are grouped by the role
            // the instructions establish for them.
            for (target_role, sites) in site_roles(&leaf.program, &indices, *role, *symbol)? {
                let Some(multiplicity) = NonZeroUsize::new(sites.len()) else {
                    continue;
                };
                relocations.insert(LiveRelocation {
                    symbol: *symbol,
                    role: target_role,
                    site: LiveRelocationSite::ProgramInstructions {
                        leaf: *role,
                        indices: sites,
                    },
                    width: symbol_width(target, symbols, constructor, *symbol),
                    encoding: symbol_encoding(*symbol),
                    multiplicity,
                    substitution: SubstitutionMode::StructuredBeforeSerialization,
                });
            }
        }
    }

    relocations.extend(constructor_relocations(
        target,
        symbols,
        constructor,
        leaves,
    ));
    Ok(relocations)
}

/// What field one introspection primitive reads.
#[derive(Clone, Copy, PartialEq, Eq)]
enum InspectedField {
    Asset,
    Program,
    Value,
    Issuance,
}

/// The introspection primitive dominating one instruction, if any.
fn introspection_context(
    program: &TapscriptProgram,
    index: usize,
) -> Option<(FieldSide, InspectedField)> {
    program.instructions()[..index]
        .iter()
        .rev()
        .find_map(|instruction| {
            let TapscriptInstruction::Opcode(id) = instruction else {
                return None;
            };
            Some(match id {
                OpcodeId::InspectInputAsset => (FieldSide::Input, InspectedField::Asset),
                OpcodeId::InspectOutputAsset => (FieldSide::Output, InspectedField::Asset),
                OpcodeId::InspectInputScriptPubKey => (FieldSide::Input, InspectedField::Program),
                OpcodeId::InspectOutputScriptPubKey => (FieldSide::Output, InspectedField::Program),
                OpcodeId::InspectInputValue => (FieldSide::Input, InspectedField::Value),
                OpcodeId::InspectOutputValue => (FieldSide::Output, InspectedField::Value),
                OpcodeId::InspectInputIssuance => (FieldSide::Input, InspectedField::Issuance),
                _ => return None,
            })
        })
}

/// The target roles one symbol's sites hold, grouped by role.
///
/// Read from the instructions rather than declared: each site must be a
/// pushed literal in a context the symbol could occupy. The owner key is
/// the one symbol whose context is not an introspection — it is the
/// operand of a signature primitive — so its site is required to be
/// followed by one, which is the same kind of check read forwards.
fn site_roles(
    program: &TapscriptProgram,
    indices: &BTreeSet<usize>,
    leaf: LiveTransferLeafRole,
    symbol: LiveBundleSymbol,
) -> Result<BTreeMap<TargetRole, BTreeSet<usize>>, LiveBundleRefusal> {
    let mut roles: BTreeMap<TargetRole, BTreeSet<usize>> = BTreeMap::new();

    for index in indices {
        if !matches!(
            program.instructions().get(*index),
            Some(TapscriptInstruction::Push(_))
        ) {
            return Err(LiveBundleRefusal::RelocationSiteIsNotAPush {
                leaf,
                index: *index,
            });
        }

        let role = match symbol {
            LiveBundleSymbol::OwnerPublicKey
                if matches!(
                    program.instructions().get(index + 1),
                    Some(TapscriptInstruction::Opcode(
                        OpcodeId::CheckSig | OpcodeId::CheckSigVerify
                    ))
                ) =>
            {
                TargetRole::SignatureKeyOperand
            }
            _ => match (symbol, introspection_context(program, *index)) {
                (
                    LiveBundleSymbol::ProtocolAsset | LiveBundleSymbol::ReserveAsset,
                    Some((side, InspectedField::Asset)),
                ) => TargetRole::AssetComparand { side },
                (LiveBundleSymbol::SponsorChangeProgram, Some((side, InspectedField::Program))) => {
                    TargetRole::ProgramComparand { side }
                }
                (
                    LiveBundleSymbol::DestinationProgramVersion
                    | LiveBundleSymbol::SponsorChangeProgramVersion,
                    Some((side, InspectedField::Program)),
                ) => TargetRole::ProgramVersionComparand { side },
                (
                    LiveBundleSymbol::TargetFeeRoleProgramDigest,
                    Some((FieldSide::Output, InspectedField::Program)),
                ) => TargetRole::FeeProgramDigestComparand,
                _ => {
                    return Err(LiveBundleRefusal::RelocationSiteContextMismatch {
                        leaf,
                        index: *index,
                        symbol,
                    });
                }
            },
        };
        roles.entry(role).or_default().insert(*index);
    }

    Ok(roles)
}

/// The relocations of the symbols no program pushes.
fn constructor_relocations(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &LiveTransferSymbols,
    constructor: &StaticLiveReceiptConstructor,
    leaves: &BTreeMap<LiveTransferLeafRole, LiveLeafProgram>,
) -> BTreeSet<LiveRelocation> {
    let mut relocations = BTreeSet::new();
    let one = NonZeroUsize::MIN;

    for (symbol, role) in [
        (
            LiveBundleSymbol::UnspendableInternalKey,
            TargetRole::TaprootInternalKey,
        ),
        (
            LiveBundleSymbol::TargetLeafVersion,
            TargetRole::TaprootLeafVersion,
        ),
        (
            LiveBundleSymbol::CandidateReceiptInputBound,
            TargetRole::CandidateShapeBound,
        ),
        (
            LiveBundleSymbol::CandidateReceiptOutputBound,
            TargetRole::CandidateShapeBound,
        ),
        (
            LiveBundleSymbol::CandidateSponsorBound,
            TargetRole::CandidateShapeBound,
        ),
    ] {
        relocations.insert(LiveRelocation {
            symbol,
            role,
            site: LiveRelocationSite::ConstructorBinding,
            width: symbol_width(target, symbols, constructor, symbol),
            encoding: symbol_encoding(symbol),
            multiplicity: one,
            substitution: SubstitutionMode::StructuredBeforeSerialization,
        });
    }

    for (leaf, program) in leaves {
        let symbol = match leaf {
            LiveTransferLeafRole::Coordinator { shape, .. } => {
                LiveBundleSymbol::CoordinatorProgram { shape: *shape }
            }
            LiveTransferLeafRole::Member { receipt_inputs, .. } => {
                LiveBundleSymbol::MemberProgram {
                    receipt_inputs: *receipt_inputs,
                }
            }
        };
        relocations.insert(LiveRelocation {
            symbol,
            role: TargetRole::CommittedLeafScript,
            site: LiveRelocationSite::ConstructorBinding,
            width: SymbolWidth::ValueDetermined {
                bytes: usize::try_from(program.program.encoded_length(target))
                    .unwrap_or(usize::MAX),
            },
            encoding: RelocationEncoding::TapscriptLeafScript,
            multiplicity: one,
            substitution: SubstitutionMode::StructuredBeforeSerialization,
        });
    }

    relocations
}

/// The reviewed encoding class one symbol's payload belongs to.
const fn symbol_class(symbol: LiveBundleSymbol) -> Option<EncodingClass> {
    Some(match symbol {
        LiveBundleSymbol::ProtocolAsset | LiveBundleSymbol::ReserveAsset => {
            EncodingClass::ExplicitAsset
        }
        LiveBundleSymbol::SponsorChangeProgram => EncodingClass::WitnessProgram,
        LiveBundleSymbol::TargetFeeRoleProgramDigest => EncodingClass::ScriptPubKeySha256,
        LiveBundleSymbol::DestinationProgramVersion
        | LiveBundleSymbol::SponsorChangeProgramVersion => EncodingClass::ScriptNumber,
        LiveBundleSymbol::OwnerPublicKey | LiveBundleSymbol::UnspendableInternalKey => {
            EncodingClass::XOnlyPublicKey
        }
        _ => return None,
    })
}

/// How one symbol is carried.
const fn symbol_encoding(symbol: LiveBundleSymbol) -> RelocationEncoding {
    match symbol_class(symbol) {
        Some(EncodingClass::ScriptNumber) => RelocationEncoding::ScriptNumber,
        Some(class) => RelocationEncoding::EncodedPayload { class },
        None => match symbol {
            LiveBundleSymbol::TargetLeafVersion => RelocationEncoding::ControlBlockField,
            LiveBundleSymbol::CoordinatorProgram { .. }
            | LiveBundleSymbol::MemberProgram { .. } => RelocationEncoding::TapscriptLeafScript,
            _ => RelocationEncoding::TypedParameter,
        },
    }
}

/// What one symbol occupies, read from the reviewed contract where the
/// contract fixes it and from the laid-out value where it does not.
fn symbol_width(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &LiveTransferSymbols,
    constructor: &StaticLiveReceiptConstructor,
    symbol: LiveBundleSymbol,
) -> SymbolWidth {
    let script_number =
        |value: i64| StackItem::script_number(target, value).map_or(0, |item| item.len());
    let bytes = match symbol {
        LiveBundleSymbol::ProtocolAsset => symbols.protocol_asset().len(),
        LiveBundleSymbol::ReserveAsset => symbols.reserve_asset().len(),
        LiveBundleSymbol::SponsorChangeProgram => symbols.sponsor_change_program().len(),
        LiveBundleSymbol::TargetFeeRoleProgramDigest => symbols.fee_program_digest().len(),
        LiveBundleSymbol::OwnerPublicKey => constructor.owner().width(),
        LiveBundleSymbol::DestinationProgramVersion => {
            script_number(symbols.destination_program_version())
        }
        LiveBundleSymbol::SponsorChangeProgramVersion => {
            script_number(symbols.sponsor_change_version())
        }
        LiveBundleSymbol::UnspendableInternalKey | LiveBundleSymbol::TargetLeafVersion => 0,
        // A typed parameter or a program: no field of this bundle carries
        // a serialized form of it.
        LiveBundleSymbol::SelectedSighashProfile
        | LiveBundleSymbol::CandidateReceiptInputBound
        | LiveBundleSymbol::CandidateReceiptOutputBound
        | LiveBundleSymbol::CandidateSponsorBound
        | LiveBundleSymbol::CoordinatorProgram { .. }
        | LiveBundleSymbol::MemberProgram { .. } => return SymbolWidth::Unserialized,
    };

    if symbol == LiveBundleSymbol::TargetLeafVersion {
        // One byte, and fixed: the reviewed contract states exactly one
        // leaf version, so no resolution can widen it.
        return SymbolWidth::Fixed { bytes: 1 };
    }

    match symbol_class(symbol).map(|class| class_width(target, class)) {
        Some(Some(exact)) => SymbolWidth::Fixed { bytes: exact },
        _ => SymbolWidth::ValueDetermined { bytes },
    }
}

/// The exact payload width one encoding class fixes, where it fixes one.
fn class_width(
    target: &ReviewedElementsTapscriptDefinition,
    class: EncodingClass,
) -> Option<usize> {
    match target.definition().encodings().get(&class)?.payload() {
        PayloadWidth::Exact(exact) => Some(exact.get()),
        PayloadWidth::Absent | PayloadWidth::Bounded { .. } => None,
    }
}

/// The typed symbol table.
///
/// Total over the symbols this bundle names, which is every member of
/// [`LiveBundleSymbol`] the candidate reaches: the six link-time values,
/// the owner key and the four constructor bindings, and one entry per
/// emitted leaf. A symbol with no entry would be a relocation naming
/// something the table does not describe.
fn symbol_table(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &LiveTransferSymbols,
    constructor: &StaticLiveReceiptConstructor,
    leaves: &BTreeMap<LiveTransferLeafRole, LiveLeafProgram>,
) -> BTreeMap<LiveBundleSymbol, LiveSymbolEntry> {
    let mut table = BTreeMap::new();
    let mut record = |symbol: LiveBundleSymbol, binding: SymbolBinding, width: SymbolWidth| {
        table.insert(
            symbol,
            LiveSymbolEntry {
                binding,
                width,
                encoding: symbol_encoding(symbol),
            },
        );
    };

    for symbol in [
        LiveBundleSymbol::ProtocolAsset,
        LiveBundleSymbol::ReserveAsset,
        LiveBundleSymbol::DestinationProgramVersion,
        LiveBundleSymbol::SponsorChangeProgram,
        LiveBundleSymbol::SponsorChangeProgramVersion,
        LiveBundleSymbol::TargetFeeRoleProgramDigest,
        LiveBundleSymbol::UnspendableInternalKey,
    ] {
        let width = symbol_width(target, symbols, constructor, symbol);
        record(symbol, SymbolBinding::ResolvedAtLink, width);
    }
    for symbol in [
        // Settled by this bundle's own constructor, contract, and shape
        // set: there is nothing for a deployment to supply.
        LiveBundleSymbol::OwnerPublicKey,
        LiveBundleSymbol::TargetLeafVersion,
        LiveBundleSymbol::SelectedSighashProfile,
        LiveBundleSymbol::CandidateReceiptInputBound,
        LiveBundleSymbol::CandidateReceiptOutputBound,
        LiveBundleSymbol::CandidateSponsorBound,
    ] {
        let width = symbol_width(target, symbols, constructor, symbol);
        record(symbol, SymbolBinding::DefinedByBundle, width);
    }
    for (leaf, program) in leaves {
        let symbol = match leaf {
            LiveTransferLeafRole::Coordinator { shape, .. } => {
                LiveBundleSymbol::CoordinatorProgram { shape: *shape }
            }
            LiveTransferLeafRole::Member { receipt_inputs, .. } => {
                LiveBundleSymbol::MemberProgram {
                    receipt_inputs: *receipt_inputs,
                }
            }
        };
        // The leaf's own exact encoded length, which is the width its
        // relocation places it at. `symbol_width` would answer
        // `Unserialized` here, and the two answers would disagree about
        // one symbol: a leaf script reaches no *comparand* field of this
        // bundle, but it is very much serialized — it is the committed
        // program itself.
        record(
            symbol,
            SymbolBinding::DefinedByBundle,
            SymbolWidth::ValueDetermined {
                bytes: usize::try_from(program.program.encoded_length(target))
                    .unwrap_or(usize::MAX),
            },
        );
    }

    table
}
