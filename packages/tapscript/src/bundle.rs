//! The candidate relocatable backend bundle (Guide-12 §11, §13).
//!
//! # What this module emits, and what it refuses to be
//!
//! [`emit_candidate_bundle`] assembles one
//! [`CandidateRelocatableTapscriptBundle`]: the static ASH constructor,
//! the typed coordinator and member programs, their concrete
//! placements, the typed symbols and relocations, the witness roles,
//! the resource formulas, and the operation verdict census the gate of
//! §8.3 was run against. Nothing is emitted before that gate passes.
//!
//! The type is a *candidate* and cannot be talked into being anything
//! else. Three separate facts hold it there, and none of them is a
//! sentence in a comment:
//!
//! - construction refuses a plan whose lifecycle is release-complete
//!   ([`BundleRefusal::PlanClaimsReleaseComplete`]), so a bundle only
//!   exists while an exit is outstanding, and the outstanding set it
//!   carries is structurally non-empty ([`OutstandingLifecycle`]);
//! - [`BackendArtifactStatus`] is read, never written: there is no
//!   field, constructor argument, or setter through which a caller
//!   could claim `CandidateOperationProven` or `ProductionApproved`.
//!   §13.5 admits the first of those only after full operation
//!   evidence, and the artifact that would carry that evidence does not
//!   exist yet, so no field reserves a place for it (§1.10);
//! - [`KeyPathPolicy`] and [`InternalKeyPolicy`] have one variant each,
//!   so an accepted key-path escape and an operator-held internal key
//!   are unrepresentable rather than merely unused (§11.3, §11.4).
//!
//! # Relationship to the accepted Guide-10 constructor prototype
//!
//! §1.8 forbids a prototype becoming an operation pattern silently, so
//! the relationship is stated. The Guide-10 metadata-constructor
//! prototype is *followed* in one respect only: the technique of
//! binding a constructor to a target contract, a leaf version, an
//! internal-key policy, and a finite leaf set decided before emission,
//! rather than assembling leaves per request. That technique is what
//! [`StaticAshConstructor`] reuses.
//!
//! It is *departed from* in every respect that carried its semantics.
//! The prototype's synthetic continuity counter, its representation
//! nonce, and its mutable metadata leaf are all absent, because compact
//! ASH has no mutable semantic metadata for them to carry and §11.1
//! refuses them being present merely because the prototype had them.
//! Its wide-floor arithmetic is likewise absent: the aggregate here is
//! the exact fixed-width sum Wave 6 scheduled. No promotion is claimed:
//! the five conditions §1.8 attaches to a promotion include measured
//! resource behaviour in the complete transaction and passing
//! relation-indexed target evidence, and neither exists yet.
//!
//! # Why the placements are recomputed rather than declared
//!
//! Every placement in this module is derived from the thing it
//! describes and then checked against it. Input and output positions
//! are laid out from the shape and required to account for every
//! position exactly once. A shared member leaf is admitted only after
//! each sharing shape's own program has been rebuilt and compared
//! instruction by instruction (§11.2). Relocation sites are not found
//! by looking for bytes that resemble a symbol — two distinct symbols
//! can resolve to the same literal, and a version number is
//! indistinguishable from an index — they are found by rebuilding the
//! program with exactly one symbol replaced and taking the positions
//! that moved. A site's target role is then read from the introspection
//! primitive that dominates it, and a symbol sitting under the wrong
//! primitive is a refusal rather than a relocation.
//!
//! # No literal here is minted, and no digest is taken
//!
//! The symbols keep Wave 6's discipline: the bundle is laid out against
//! the symbol values it is handed, records where they sit, and states
//! that a later layer resolves them. It mints no identity of its own —
//! no bundle hash, no leaf hash, no program hash — because §1.10 admits
//! a digest only once a real consumer of one exists.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;

use compiler::operation_plan::{
    CarrierRole, LifecycleRequirement, RelationCaseKey, ValidatedTargetOperationPlan,
};
use target_elements::{
    EncodingClass, LeafVersion, OpcodeId, PayloadWidth, ResourceDimension,
    ReviewedElementsTapscriptDefinition, TargetContractVersion, TargetEvidenceRequirementId,
};

use crate::capability::BackendPatternId;
use crate::error::TapscriptError;
use crate::instruction::{StackItem, TapscriptInstruction};
use crate::operation_assessment::{EmissionRefusal, OperationAssessmentSet, assess_operation_plan};
use crate::pattern::{AbiAssumption, CompactAshSymbols, coordinator_program, member_program};
use crate::policy::{
    AshRepresentationSelection, CompactAshBackendPolicy, ExactTargetProjection, TieBreak,
};
use crate::program::TapscriptProgram;
use crate::shape::{CandidateShapeSet, CompactAshShape, SponsorChangePresence};
use crate::stack::{AbstractLimits, AbstractStackState, resource_projection, validate_program};

// --- Program and leaf roles ------------------------------------------

/// Which typed role one emitted program plays (§13.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProgramRole {
    /// The single program spending the canonical anchor input.
    Coordinator,
    /// The program every non-anchor member of the ASH family spends.
    Member,
}

/// The identity of one leaf of the static constructor's program set.
///
/// The identity is exactly what the program depends on, which is what
/// makes leaf sharing checkable rather than asserted: a coordinator
/// program is a function of the whole shape, and a member program is a
/// function of the batch size alone, so two shapes with equal batch
/// sizes name one member leaf by construction. §11.2 permits that only
/// with a typed proof, and [`SharedLeafProof`] is where the recomputed
/// comparison is recorded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LeafRole {
    /// The coordinator leaf of one exact shape.
    Coordinator {
        /// The shape whose counts the program authenticates.
        shape: CompactAshShape,
    },
    /// The member leaf serving every shape of one batch size.
    Member {
        /// The ASH batch size whose member range the program bounds.
        ash_inputs: u8,
    },
}

impl LeafRole {
    /// Which program role this leaf carries.
    #[must_use]
    pub const fn program_role(self) -> ProgramRole {
        match self {
            Self::Coordinator { .. } => ProgramRole::Coordinator,
            Self::Member { .. } => ProgramRole::Member,
        }
    }
}

// --- Concrete layout --------------------------------------------------

/// Which side of the transaction one field belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FieldSide {
    /// An input field.
    Input,
    /// An output field.
    Output,
}

/// The role of one input position in the candidate layout (§10.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputRole {
    /// The canonical anchor, which is also an ASH source.
    Coordinator,
    /// A non-anchor ASH source.
    Member,
    /// A member of the isolated sponsor suffix.
    Sponsor,
}

/// The role of one output position in the candidate layout (§10.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OutputRole {
    /// The successor ASH object.
    Successor,
    /// The optional sponsor-change role.
    SponsorChange,
    /// The target's own fee role.
    TargetFee,
}

/// One contiguous run of input positions holding one role.
///
/// Half-open, and exact: the range is computed from the shape and the
/// whole census is then required to account for every position of the
/// transaction exactly once.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InputPlacement {
    role: InputRole,
    first: u16,
    end: u16,
}

impl InputPlacement {
    /// The role these positions hold.
    #[must_use]
    pub const fn role(self) -> InputRole {
        self.role
    }

    /// The first position of the run.
    #[must_use]
    pub const fn first(self) -> u16 {
        self.first
    }

    /// One past the last position of the run.
    #[must_use]
    pub const fn end(self) -> u16 {
        self.end
    }

    /// How many positions the run covers.
    #[must_use]
    pub const fn count(self) -> u16 {
        self.end.saturating_sub(self.first)
    }
}

/// One output position and the role it holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutputPlacement {
    role: OutputRole,
    position: u16,
}

impl OutputPlacement {
    /// The role this position holds.
    #[must_use]
    pub const fn role(self) -> OutputRole {
        self.role
    }

    /// The exact position.
    #[must_use]
    pub const fn position(self) -> u16 {
        self.position
    }
}

/// The exact transaction layout of one admitted shape (§10).
///
/// Both censuses are exhaustive and disjoint by construction: a
/// position claimed twice and a position claimed by nobody are separate
/// typed refusals, so a layout that quietly left a hole does not exist.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConcreteLayout {
    shape: CompactAshShape,
    inputs: Vec<InputPlacement>,
    outputs: Vec<OutputPlacement>,
}

impl ConcreteLayout {
    /// The shape this layout belongs to.
    #[must_use]
    pub const fn shape(&self) -> CompactAshShape {
        self.shape
    }

    /// Every input run, in position order.
    #[must_use]
    pub fn inputs(&self) -> &[InputPlacement] {
        &self.inputs
    }

    /// Every output position, in position order.
    #[must_use]
    pub fn outputs(&self) -> &[OutputPlacement] {
        &self.outputs
    }

    /// The run holding one input role, if the shape has one.
    #[must_use]
    pub fn input_run(&self, role: InputRole) -> Option<InputPlacement> {
        self.inputs
            .iter()
            .copied()
            .find(|placement| placement.role == role)
    }

    /// The position holding one output role, if the shape has one.
    #[must_use]
    pub fn output_position(&self, role: OutputRole) -> Option<u16> {
        self.outputs
            .iter()
            .find(|placement| placement.role == role)
            .map(|placement| placement.position)
    }
}

// --- Witness roles ----------------------------------------------------

/// One component a spend of a protocol leaf must supply (§13.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WitnessComponent {
    /// The leaf program itself.
    LeafScript,
    /// The control block proving the leaf is in the committed tree.
    ControlBlock,
}

/// What a spend of one leaf must put on the witness stack.
///
/// The data-item count is established rather than declared: the program
/// is walked from the empty stack, and a program that schedules from
/// empty is a program no caller supplies a datum to. That is the
/// property §12.8 needs and the one a permissionless path rests on — a
/// leaf requiring a witness datum is a leaf whose spend somebody has to
/// be able to produce.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WitnessRole {
    leaf: LeafRole,
    components: BTreeSet<WitnessComponent>,
    data_items: usize,
}

impl WitnessRole {
    /// The leaf this role belongs to.
    #[must_use]
    pub const fn leaf(&self) -> LeafRole {
        self.leaf
    }

    /// Every component the spend carries, in canonical order.
    #[must_use]
    pub const fn components(&self) -> &BTreeSet<WitnessComponent> {
        &self.components
    }

    /// How many witness data items the program consumes.
    ///
    /// Zero for every leaf of this candidate, established by scheduling
    /// the program from the empty stack rather than by assertion.
    #[must_use]
    pub const fn data_items(&self) -> usize {
        self.data_items
    }
}

// --- Symbols and relocations ------------------------------------------

/// One typed link-time role of the candidate bundle (§13.4).
///
/// Both halves of §13.4's list are present: the roles a later layer
/// must resolve, and the roles this bundle itself defines and hands the
/// linker to place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BundleSymbol {
    /// The closed protocol asset the ASH family carries.
    ClosedAsset,
    /// The reserve asset every sponsor and fee role carries.
    ReserveAsset,
    /// The ASH constructor's witness program.
    ///
    /// Named here and settled nowhere. It is the taproot output over
    /// the taptree this bundle's own leaves are committed in, so no
    /// layer this side of a deployed tree can hand over its bytes — and
    /// no program needs them, because the leaves that compare against
    /// it read it from the input they are spending. The symbol exists
    /// so that dependency is visible to the linker's cycle analysis
    /// rather than invisible for being unwritten.
    AshConstructorProgram,
    /// The sponsor-change role's witness program.
    SponsorChangeProgram,
    /// The version the sponsor-change program is read at.
    SponsorChangeProgramVersion,
    /// The target fee role's program digest.
    TargetFeeRoleProgramDigest,
    /// The unspendable taproot internal key (§11.3).
    UnspendableInternalKey,
    /// The reviewed tapscript leaf version.
    TargetLeafVersion,
    /// The candidate ASH batch bound.
    CandidateAshBound,
    /// The candidate sponsor-region bound.
    CandidateSponsorBound,
    /// The coordinator program of one shape, defined here.
    CoordinatorProgram {
        /// The shape whose coordinator this is.
        shape: CompactAshShape,
    },
    /// The member program of one batch size, defined here.
    MemberProgram {
        /// The batch size whose member leaf this is.
        ash_inputs: u8,
    },
}

/// Who settles one symbol's value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SymbolBinding {
    /// This bundle settles it, from the reviewed contract or from its
    /// own candidate shape set.
    DefinedByBundle,
    /// A later layer settles it, which is exactly
    /// [`AbiAssumption::SymbolsResolvedAtLink`].
    ResolvedAtLink,
    /// No layer settles it: the referring programs read the value from
    /// the target at spend time.
    ///
    /// Not a weaker [`Self::ResolvedAtLink`] but a different claim. A
    /// symbol bound this way has no link-time value, so there is
    /// nothing for a deployment to supply, nothing to substitute, and
    /// no relocation site — and a caller reading the census cannot
    /// mistake it for a parameter somebody forgot to fill in.
    ReadFromTargetAtSpendTime,
}

/// What one symbol occupies wherever it is placed (§13.4).
///
/// There is no variable-width case, and that absence is the §13.4
/// prohibition rather than an oversight: a byte patch is admissible
/// only against [`Self::Fixed`], and construction refuses a patch
/// stated against a width only the resolved value settles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SymbolWidth {
    /// A width the reviewed contract fixes for every resolution.
    Fixed {
        /// The exact payload width in bytes.
        bytes: usize,
    },
    /// A width only the resolved value settles.
    ValueDetermined {
        /// The width the value this bundle was laid out against takes.
        bytes: usize,
    },
    /// The symbol reaches no serialized field of this bundle.
    Unserialized,
}

/// How one symbol is carried where it is placed (§13.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelocationEncoding {
    /// The payload of one reviewed encoding class, pushed as a literal.
    EncodedPayload {
        /// The reviewed class fixing the payload's form.
        class: EncodingClass,
    },
    /// The script language's minimal number encoding.
    ScriptNumber,
    /// A field of the taproot control block the linker writes.
    ControlBlockField,
    /// A complete tapscript leaf program.
    TapscriptLeafScript,
    /// A typed constructor parameter with no target encoding at all.
    TypedParameter,
}

/// What one site's value is compared with or used as (§13.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TargetRole {
    /// Compared with an introspected asset field.
    AssetComparand {
        /// Which side the introspected field is on.
        side: FieldSide,
    },
    /// Compared with an introspected program field.
    ProgramComparand {
        /// Which side the introspected field is on.
        side: FieldSide,
    },
    /// Compared with the version an introspected program is read at.
    ProgramVersionComparand {
        /// Which side the introspected field is on.
        side: FieldSide,
    },
    /// Compared with the digest standing in for the fee role's program.
    FeeProgramDigestComparand,
    /// Bound as the taproot internal key.
    TaprootInternalKey,
    /// Bound as the leaf version of every emitted leaf.
    TaprootLeafVersion,
    /// Bound as a bound of the candidate shape set.
    CandidateShapeBound,
    /// Placed as one leaf of the committed program set.
    CommittedLeafScript,
}

/// Where one relocation applies (§13.4).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RelocationSite {
    /// Exact push positions inside one emitted program.
    ProgramInstructions {
        /// The leaf whose program carries them.
        leaf: LeafRole,
        /// The exact instruction indices, non-empty.
        indices: BTreeSet<usize>,
    },
    /// A binding of the static constructor, outside any program.
    ConstructorBinding,
}

/// How one symbol reaches its site (§13.4).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubstitutionMode {
    /// Substituted into the typed program before serialization.
    ///
    /// §13.4's preferred form, and the only one this bundle uses: the
    /// program is re-serialized after substitution, so a resolution
    /// whose width differs from the one laid out against costs nothing
    /// and corrupts nothing.
    StructuredBeforeSerialization,
    /// Fixed-width byte patching, with the placeholder to overwrite.
    ///
    /// Admissible only against [`SymbolWidth::Fixed`], and only where
    /// the placeholder is exactly that width. Both are checked, so a
    /// variable-width patch cannot be stated.
    FixedWidthBytePatch {
        /// The exact bytes the unresolved artifact carries.
        placeholder: StackItem,
    },
}

/// One relocation of one symbol (§13.4).
///
/// States every item the rule lists: the source symbol, the target
/// role, the semantic location, the width, the encoding, the
/// multiplicity, and the placeholder where a byte patch is unavoidable.
/// The multiplicity is the exact site count, computed when the
/// relocation is built rather than declared alongside it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Relocation {
    symbol: BundleSymbol,
    role: TargetRole,
    site: RelocationSite,
    width: SymbolWidth,
    encoding: RelocationEncoding,
    multiplicity: NonZeroUsize,
    substitution: SubstitutionMode,
}

impl Relocation {
    /// State a byte-patching relocation, if §13.4 admits one.
    ///
    /// The only constructor this type offers, and the reason it exists
    /// is that §13.4's prohibition has to be something that runs. A
    /// patch is admissible only where the width is fixed by the
    /// reviewed contract for every resolution, and only where the
    /// placeholder is exactly that width. Everything this bundle emits
    /// substitutes before serialization instead and is built inside
    /// this module, where no patch is reachable at all.
    ///
    /// # Errors
    ///
    /// [`BundleRefusal::VariableWidthBytePatch`] when the width is one
    /// only the resolved value settles, and
    /// [`BundleRefusal::PlaceholderWidthMismatch`] when the placeholder
    /// is not that width.
    pub fn byte_patch(
        symbol: BundleSymbol,
        role: TargetRole,
        site: RelocationSite,
        width: SymbolWidth,
        encoding: RelocationEncoding,
        multiplicity: NonZeroUsize,
        placeholder: StackItem,
    ) -> Result<Self, BundleRefusal> {
        let SymbolWidth::Fixed { bytes } = width else {
            return Err(BundleRefusal::VariableWidthBytePatch { symbol });
        };
        if placeholder.len() != bytes {
            return Err(BundleRefusal::PlaceholderWidthMismatch { symbol });
        }

        Ok(Self {
            symbol,
            role,
            site,
            width,
            encoding,
            multiplicity,
            substitution: SubstitutionMode::FixedWidthBytePatch { placeholder },
        })
    }

    /// The symbol whose value this relocation places.
    #[must_use]
    pub const fn symbol(&self) -> BundleSymbol {
        self.symbol
    }

    /// What the placed value is used as.
    #[must_use]
    pub const fn role(&self) -> TargetRole {
        self.role
    }

    /// The semantic location.
    #[must_use]
    pub const fn site(&self) -> &RelocationSite {
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

/// One leaf's dependency on a symbol it reads from the target.
///
/// The counterpart of [`Relocation`] for a value no layer supplies. A
/// relocation records where a later layer writes bytes in; this records
/// where the program fetches them itself, and the two are exclusive by
/// construction — a site cannot both be patched and be introspected.
///
/// Recorded rather than inferred, because the dependency is otherwise
/// invisible. The whole hazard of a self-committing constructor is that
/// a program which stopped carrying a literal for it looks, to anything
/// reading relocations alone, like a program that never needed it. The
/// linker's cycle analysis needs the edge, so the bundle states it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IntrospectionReference {
    symbol: BundleSymbol,
    leaf: LeafRole,
    role: TargetRole,
    sites: NonZeroUsize,
}

impl IntrospectionReference {
    /// The symbol whose value the program reads.
    #[must_use]
    pub const fn symbol(&self) -> BundleSymbol {
        self.symbol
    }

    /// The leaf whose program reads it.
    #[must_use]
    pub const fn leaf(&self) -> LeafRole {
        self.leaf
    }

    /// What the read value is, and which side it is read from.
    ///
    /// The side is the read's, not the comparison's. Every read here is
    /// of the leaf's own input, including the ones whose result is
    /// compared against an output — the successor's program test reads
    /// an input and compares an output with it.
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

/// What one symbol is, independent of where it is placed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolEntry {
    binding: SymbolBinding,
    width: SymbolWidth,
    encoding: RelocationEncoding,
}

impl SymbolEntry {
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

// --- Resources --------------------------------------------------------

/// The exact resource cost of one emitted program (§13.3).
///
/// Read from the resource projection, which charges the exact encoded
/// byte length including every push opcode and width prefix, never a
/// sum of opcode costs. The initial-stack-item count is the witness
/// role's, so the two cannot disagree.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProgramResources {
    dimensions: BTreeMap<ResourceDimension, u64>,
}

impl ProgramResources {
    /// Every charged dimension, in canonical order.
    #[must_use]
    pub const fn dimensions(&self) -> &BTreeMap<ResourceDimension, u64> {
        &self.dimensions
    }

    /// One dimension's exact figure, where this program charges it.
    #[must_use]
    pub fn charged(&self, dimension: ResourceDimension) -> Option<u64> {
        self.dimensions.get(&dimension).copied()
    }
}

/// Which later layer owes a resource dimension this bundle cannot
/// establish (§13.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceObligation {
    /// The abstract walk would have to track a peak it does not track.
    AbstractExecutionWalk,
    /// The linker settles it once a taptree exists.
    Linker,
    /// Only a complete transaction settles it.
    CompleteTransaction,
}

/// How one dimension varies across the candidate shape set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceModel {
    /// An exact affine function of the shape's three counts.
    ///
    /// Valid over exactly the candidate shape set and nowhere else: the
    /// coefficients are read off differences within that set and then
    /// recomputed against every member of it. A shape outside the set
    /// has no program, so it has no figure for this to predict.
    Affine {
        /// The constant term.
        base: i64,
        /// The cost of one further ASH source.
        per_ash_input: i64,
        /// The cost of the first sponsor input and the fee role it
        /// forces.
        per_sponsor_input: i64,
        /// The cost of the optional sponsor-change role.
        per_sponsor_change: i64,
    },
    /// No affine model reproduces every measurement exactly.
    ///
    /// Never a partial fit: the exact per-shape figures stand alone
    /// rather than a model that is right about most of them (§1.11).
    ExactTableOnly,
}

/// The resource behaviour of one program role across the shape set.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShapeResourceFormula {
    role: ProgramRole,
    dimension: ResourceDimension,
    model: ResourceModel,
    measurements: BTreeMap<CompactAshShape, u64>,
}

impl ShapeResourceFormula {
    /// The program role this formula describes.
    #[must_use]
    pub const fn role(&self) -> ProgramRole {
        self.role
    }

    /// The dimension this formula describes.
    #[must_use]
    pub const fn dimension(&self) -> ResourceDimension {
        self.dimension
    }

    /// The established model.
    #[must_use]
    pub const fn model(&self) -> ResourceModel {
        self.model
    }

    /// The exact measurement at every shape, in canonical order.
    #[must_use]
    pub const fn measurements(&self) -> &BTreeMap<CompactAshShape, u64> {
        &self.measurements
    }

    /// What the model predicts for one shape, where a model exists.
    #[must_use]
    pub fn predict(&self, shape: CompactAshShape) -> Option<i64> {
        predict_model(self.model, shape)
    }
}

/// What one model predicts at one shape.
fn predict_model(model: ResourceModel, shape: CompactAshShape) -> Option<i64> {
    let ResourceModel::Affine {
        base,
        per_ash_input,
        per_sponsor_input,
        per_sponsor_change,
    } = model
    else {
        return None;
    };
    let (ash, sponsors, change) = shape_axes(shape);

    base.checked_add(per_ash_input.checked_mul(ash)?)?
        .checked_add(per_sponsor_input.checked_mul(sponsors)?)?
        .checked_add(per_sponsor_change.checked_mul(change)?)
}

// --- The static constructor -------------------------------------------

/// The internal-key policy of the candidate constructor (§11.3).
///
/// One variant, so an operator key, a release key, a wallet key, a
/// generated-and-discarded key, and a caller-selected key are
/// unrepresentable rather than merely unused. The key's bytes are
/// [`BundleSymbol::UnspendableInternalKey`] and are resolved at link:
/// minting one here would be the speculative identity §1.10 refuses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InternalKeyPolicy {
    /// A public unspendable internal key, with its assumption stated.
    UnspendableWithResidualDiscreteLogAssumption,
}

/// The key-path policy of the candidate constructor (§11.4).
///
/// One variant: the candidate provides no accepted key-path escape, and
/// there is no value of this type that says otherwise.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyPathPolicy {
    /// No key-path spend is accepted by this constructor.
    NoAcceptedEscape,
}

/// The explicit value policy of the candidate constructor (§11.1).
///
/// One variant, matching the one representation Guide 11 fixed. The
/// enforcement is the prefix comparison every recognition fragment
/// opens with, which is why the policy names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExplicitValuePolicy {
    /// Every value field is required explicit by prefix equality.
    RequireExplicitPrefixEquality,
}

/// An assumption the constructor rests on that it does not establish.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConstructorAssumption {
    /// The unspendability of the internal key rests on the residual
    /// discrete-log assumption, which no program discharges (§11.3).
    ResidualDiscreteLogOnUnspendableInternalKey,
    /// The resolved internal key must be verifiable as unspendable from
    /// public data alone, which the linker owes (§11.3).
    InternalKeyUnspendabilityVerifiableFromPublicData,
}

/// Why one leaf may serve several shapes (§11.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SharingGround {
    /// Each sharing shape's own program was rebuilt from that shape and
    /// compared with the shared leaf's, instruction by instruction, and
    /// found typed-equal — so the shared leaf enforces the exact same
    /// relation for every shape using it.
    IdenticalTypedProgramRecomputedPerShape,
}

/// One admitted leaf sharing, and the comparison that admitted it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SharedLeafProof {
    leaf: LeafRole,
    shapes: BTreeSet<CompactAshShape>,
    ground: SharingGround,
}

impl SharedLeafProof {
    /// The shared leaf.
    #[must_use]
    pub const fn leaf(&self) -> LeafRole {
        self.leaf
    }

    /// Every shape the leaf serves, in canonical order.
    #[must_use]
    pub const fn shapes(&self) -> &BTreeSet<CompactAshShape> {
        &self.shapes
    }

    /// What established the sharing.
    #[must_use]
    pub const fn ground(&self) -> SharingGround {
        self.ground
    }
}

/// One emitted leaf: its program, cost, and witness role.
///
/// Not ordered, deliberately: a leaf is reached by its [`LeafRole`],
/// which is the identity everything else keys on, and an ordering over
/// programs would invite one being used as a sort key where the role is
/// the thing that means something.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeafProgram {
    leaf: LeafRole,
    program: TapscriptProgram,
    resources: ProgramResources,
    witness: WitnessRole,
    shapes: BTreeSet<CompactAshShape>,
}

impl LeafProgram {
    /// The leaf's identity.
    #[must_use]
    pub const fn leaf(&self) -> LeafRole {
        self.leaf
    }

    /// The typed program.
    #[must_use]
    pub const fn program(&self) -> &TapscriptProgram {
        &self.program
    }

    /// The program's exact resource cost.
    #[must_use]
    pub const fn resources(&self) -> &ProgramResources {
        &self.resources
    }

    /// What a spend of this leaf must supply.
    #[must_use]
    pub const fn witness(&self) -> &WitnessRole {
        &self.witness
    }

    /// Every shape this leaf serves, in canonical order.
    #[must_use]
    pub const fn shapes(&self) -> &BTreeSet<CompactAshShape> {
        &self.shapes
    }
}

/// The static compact-ASH constructor (§11).
///
/// Every element §11.1 binds is present: the reviewed contract it was
/// built against, the explicit asset and value policies, the static
/// program set, the internal-key policy, the leaf version, and the
/// candidate shape set. Nothing else is: no counter, no nonce, and no
/// metadata leaf.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaticAshConstructor {
    contract: TargetContractVersion,
    leaf_version: LeafVersion,
    representation: AshRepresentationSelection,
    value_policy: ExplicitValuePolicy,
    internal_key: InternalKeyPolicy,
    key_path: KeyPathPolicy,
    assumptions: BTreeSet<ConstructorAssumption>,
    shapes: CandidateShapeSet,
    leaves: BTreeMap<LeafRole, LeafProgram>,
    sharing: BTreeSet<SharedLeafProof>,
}

impl StaticAshConstructor {
    /// The reviewed contract revision this constructor is bound to.
    #[must_use]
    pub const fn contract(&self) -> TargetContractVersion {
        self.contract
    }

    /// The leaf version every emitted leaf carries.
    #[must_use]
    pub const fn leaf_version(&self) -> LeafVersion {
        self.leaf_version
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

    /// The internal-key policy.
    #[must_use]
    pub const fn internal_key(&self) -> InternalKeyPolicy {
        self.internal_key
    }

    /// The key-path policy.
    #[must_use]
    pub const fn key_path(&self) -> KeyPathPolicy {
        self.key_path
    }

    /// Every assumption the constructor rests on.
    #[must_use]
    pub const fn assumptions(&self) -> &BTreeSet<ConstructorAssumption> {
        &self.assumptions
    }

    /// The candidate shape set.
    #[must_use]
    pub const fn shapes(&self) -> &CandidateShapeSet {
        &self.shapes
    }

    /// Every emitted leaf, in canonical order.
    #[must_use]
    pub const fn leaves(&self) -> &BTreeMap<LeafRole, LeafProgram> {
        &self.leaves
    }

    /// One leaf by identity.
    #[must_use]
    pub fn leaf(&self, leaf: LeafRole) -> Option<&LeafProgram> {
        self.leaves.get(&leaf)
    }

    /// The two leaves one shape spends through.
    ///
    /// The pair §11.2 requires of every admitted shape, returned
    /// together so a caller cannot read one without the other.
    #[must_use]
    pub fn shape_leaves(&self, shape: CompactAshShape) -> Option<(&LeafProgram, &LeafProgram)> {
        let coordinator = self.leaves.get(&LeafRole::Coordinator { shape })?;
        let member = self.leaves.get(&LeafRole::Member {
            ash_inputs: shape.ash_inputs(),
        })?;
        Some((coordinator, member))
    }

    /// Every admitted leaf sharing, in canonical order.
    #[must_use]
    pub const fn sharing(&self) -> &BTreeSet<SharedLeafProof> {
        &self.sharing
    }
}

// --- Relation placement -----------------------------------------------

/// Where one abstract carrier is concretely discharged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConcreteCarrierSite {
    /// The coordinator leaf, spending the canonical anchor input.
    CoordinatorLeaf,
    /// The member leaf, at every member position of the shape.
    MemberLeaf,
    /// The bundle's own structure rather than any program.
    BundleStructure,
    /// Outside the bundle entirely: the target's own rules.
    OutsideBundle,
}

/// One relation-case placed on concrete carriers (§13.1).
///
/// The provenance is retained rather than reduced to the answer: how
/// many assignments the compiler offered, how many this bundle can
/// actually realize, and which sites the selected one names. An
/// alternative naming a carrier this candidate emits no program for is
/// not realizable, and that is a fact about the candidate rather than a
/// defect in the plan.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConcreteRelationPlacement {
    relation_case: RelationCaseKey,
    sites: BTreeSet<ConcreteCarrierSite>,
    offered: NonZeroUsize,
    realizable: NonZeroUsize,
}

impl ConcreteRelationPlacement {
    /// The relation-case this placement discharges.
    #[must_use]
    pub const fn relation_case(&self) -> &RelationCaseKey {
        &self.relation_case
    }

    /// Every concrete site the selected assignment names.
    #[must_use]
    pub const fn sites(&self) -> &BTreeSet<ConcreteCarrierSite> {
        &self.sites
    }

    /// How many assignments the compiler offered.
    #[must_use]
    pub const fn offered(&self) -> NonZeroUsize {
        self.offered
    }

    /// How many of them this candidate can realize.
    #[must_use]
    pub const fn realizable(&self) -> NonZeroUsize {
        self.realizable
    }
}

// --- Status and lifecycle ---------------------------------------------

/// The status vocabulary backend artifacts are distinguished by (§13.5).
///
/// Guide 12 may promote the compact-ASH patterns and bundle to
/// [`Self::CandidateOperationProven`], and only after full operation
/// evidence. No such evidence exists yet, so no bundle reaches it and
/// no field of a bundle reserves a place for one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BackendArtifactStatus {
    /// Emitted and internally checked; no operation evidence.
    Prototype,
    /// Full operation evidence passed for exactly these patterns.
    CandidateOperationProven,
    /// Approved for production deployment.
    ProductionApproved,
}

/// The outstanding lifecycle exits of a candidate bundle (§1.9, §11.5).
///
/// Structurally non-empty: the least requirement is a field of its own,
/// so a bundle whose lifecycle is complete has no representation. That
/// is what makes "this is a candidate" a fact about the type rather
/// than a sentence a later edit could drop.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OutstandingLifecycle {
    least: LifecycleRequirement,
    rest: BTreeSet<LifecycleRequirement>,
}

impl OutstandingLifecycle {
    /// Every outstanding requirement, in canonical order.
    pub fn requirements(&self) -> impl Iterator<Item = &LifecycleRequirement> {
        std::iter::once(&self.least).chain(self.rest.iter())
    }

    /// How many exits are outstanding, which is never zero.
    #[must_use]
    pub fn count(&self) -> NonZeroUsize {
        NonZeroUsize::MIN.saturating_add(self.rest.len())
    }
}

// --- Refusals ---------------------------------------------------------

/// Why a candidate bundle was not emitted.
///
/// Construction failure, never target rejection (§1.5): every variant
/// names something wrong with what this backend was asked to build, and
/// none of them is a statement about what a target would do with it.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum BundleRefusal {
    /// The plan's lifecycle is complete, so it is not this type's
    /// subject: a release-complete plan needs the final bundle Guide 12
    /// must not construct (§1.9).
    PlanClaimsReleaseComplete,
    /// The operation assessment did not complete.
    Assessment(TapscriptError),
    /// A fragment or program did not build.
    Program(TapscriptError),
    /// The emission gate of §8.3 refused for one shape.
    EmissionRefused {
        /// The shape whose pattern census was gated.
        shape: CompactAshShape,
        /// Why the gate refused.
        refusal: EmissionRefusal,
    },
    /// The policy's preference and the admitted census are not the same
    /// set, so one of them names a pattern the other does not.
    PatternSelectionMismatch {
        /// Preferred but not admitted.
        preferred_only: BTreeSet<BackendPatternId>,
        /// Admitted but not preferred.
        admitted_only: BTreeSet<BackendPatternId>,
    },
    /// The candidate shape set is empty, so there is nothing to emit.
    EmptyCandidateShapeSet,
    /// A leaf's program does not schedule from the empty stack, so its
    /// witness role is not the one this bundle can establish.
    LeafDoesNotScheduleFromEmptyStack {
        /// The leaf.
        leaf: LeafRole,
    },
    /// A leaf's program reaches no successful state, so no spend of it
    /// could ever succeed.
    LeafNeverSucceeds {
        /// The leaf.
        leaf: LeafRole,
    },
    /// A shape sharing a leaf does not rebuild to that leaf's exact
    /// program, so the sharing §11.2 requires is not established.
    SharedLeafRelationMismatch {
        /// The shared leaf.
        leaf: LeafRole,
        /// The shape whose rebuild disagreed.
        shape: CompactAshShape,
    },
    /// A layout position was claimed by two roles.
    LayoutPositionClaimedTwice {
        /// The shape.
        shape: CompactAshShape,
        /// Which side.
        side: FieldSide,
        /// The position.
        position: u16,
    },
    /// A layout position was claimed by nobody.
    LayoutPositionUnaccounted {
        /// The shape.
        shape: CompactAshShape,
        /// Which side.
        side: FieldSide,
        /// The position.
        position: u16,
    },
    /// A probe program did not have the same instruction count as the
    /// program it probes, so the difference does not locate a symbol.
    ProbeProgramShapeChanged {
        /// The leaf being probed.
        leaf: LeafRole,
        /// The symbol being probed.
        symbol: BundleSymbol,
    },
    /// A relocation site is not a pushed literal.
    RelocationSiteIsNotAPush {
        /// The leaf.
        leaf: LeafRole,
        /// The instruction index.
        index: usize,
    },
    /// A relocation site is not dominated by an introspection primitive
    /// whose field the symbol could be compared with.
    RelocationSiteContextMismatch {
        /// The leaf.
        leaf: LeafRole,
        /// The instruction index.
        index: usize,
        /// The symbol placed there.
        symbol: BundleSymbol,
    },
    /// A byte patch was stated against a width only the resolved value
    /// settles, which §13.4 prohibits.
    VariableWidthBytePatch {
        /// The symbol.
        symbol: BundleSymbol,
    },
    /// A byte patch's placeholder is not the relocation's width.
    PlaceholderWidthMismatch {
        /// The symbol.
        symbol: BundleSymbol,
    },
    /// The plan states no operation-global carrier, so the anchor
    /// family this backend places against cannot be read from it.
    NoOperationAnchor,
    /// One relation-case's carrier requirement appears twice.
    DuplicateCarrierRequirement,
    /// No offered carrier assignment names only carriers this candidate
    /// emits a program for.
    NoRealizableCarrierAssignment {
        /// The relation-case with nowhere to go.
        relation_case: RelationCaseKey,
    },
    /// Several realizable assignments remain and the policy states no
    /// tie-break, so there is no accountable ground to pick one.
    TiedCarrierAssignments {
        /// The relation-case.
        relation_case: RelationCaseKey,
        /// How many assignments tied.
        tied: usize,
    },
}

// --- The bundle -------------------------------------------------------

/// The candidate relocatable backend bundle (§13.1).
///
/// Carries every item §13.1 lists, and reaches no further: the
/// validated plan, the exact target projection, the backend policy, the
/// candidate shape set, the static constructor, the typed programs, the
/// selected pattern identities, the concrete relation placements and
/// candidate layouts, the witness roles, the symbols and relocations,
/// the resource formulas, the carrier provenance, the explicit status,
/// and the outstanding clear lifecycle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateRelocatableTapscriptBundle {
    plan: ValidatedTargetOperationPlan,
    policy: CompactAshBackendPolicy,
    assessment: OperationAssessmentSet,
    constructor: StaticAshConstructor,
    layouts: BTreeMap<CompactAshShape, ConcreteLayout>,
    placements: BTreeMap<RelationCaseKey, ConcreteRelationPlacement>,
    symbols: BTreeMap<BundleSymbol, SymbolEntry>,
    unresolved: CompactAshSymbols,
    relocations: BTreeSet<Relocation>,
    introspections: BTreeSet<IntrospectionReference>,
    formulas: BTreeMap<ProgramRole, BTreeMap<ResourceDimension, ShapeResourceFormula>>,
    outstanding_dimensions: BTreeMap<ResourceDimension, ResourceObligation>,
    patterns: BTreeSet<BackendPatternId>,
    abi: BTreeSet<AbiAssumption>,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
    lifecycle: OutstandingLifecycle,
}

impl CandidateRelocatableTapscriptBundle {
    /// The validated operation plan this bundle was emitted for.
    ///
    /// Carried whole rather than named, for the same reason the plan
    /// carries its own typed source: a reference to a plan would be a
    /// plan this value cannot check, and every consumer would have to
    /// trust that the thing referred to still said what it said.
    #[must_use]
    pub const fn plan(&self) -> &ValidatedTargetOperationPlan {
        &self.plan
    }

    /// The backend policy this bundle was emitted under.
    #[must_use]
    pub const fn policy(&self) -> &CompactAshBackendPolicy {
        &self.policy
    }

    /// The exact reviewed target projection.
    ///
    /// Read from the policy rather than stored again: a second copy
    /// could disagree with the one the selection was made under.
    #[must_use]
    pub const fn target_projection(&self) -> &ExactTargetProjection {
        self.policy.projection()
    }

    /// The operation verdict census the emission gate ran against.
    #[must_use]
    pub const fn assessment(&self) -> &OperationAssessmentSet {
        &self.assessment
    }

    /// The static ASH constructor.
    #[must_use]
    pub const fn constructor(&self) -> &StaticAshConstructor {
        &self.constructor
    }

    /// The candidate shape set, read from the constructor.
    #[must_use]
    pub const fn shapes(&self) -> &CandidateShapeSet {
        self.constructor.shapes()
    }

    /// Every shape's exact transaction layout, in canonical order.
    #[must_use]
    pub const fn layouts(&self) -> &BTreeMap<CompactAshShape, ConcreteLayout> {
        &self.layouts
    }

    /// One shape's layout.
    #[must_use]
    pub fn layout(&self, shape: CompactAshShape) -> Option<&ConcreteLayout> {
        self.layouts.get(&shape)
    }

    /// Every relation-case's concrete placement, in canonical order.
    #[must_use]
    pub const fn placements(&self) -> &BTreeMap<RelationCaseKey, ConcreteRelationPlacement> {
        &self.placements
    }

    /// The typed symbol table, in canonical order.
    #[must_use]
    pub const fn symbols(&self) -> &BTreeMap<BundleSymbol, SymbolEntry> {
        &self.symbols
    }

    /// The symbol values this bundle was laid out against.
    ///
    /// Not resolutions. The programs were assembled with these values
    /// so that their exact shape and cost could be measured, and the
    /// relocations record where a later layer replaces them. Reading
    /// them as settled would be reading the placeholder as the answer.
    #[must_use]
    pub const fn unresolved_symbols(&self) -> &CompactAshSymbols {
        &self.unresolved
    }

    /// Every relocation, in canonical order.
    #[must_use]
    pub const fn relocations(&self) -> &BTreeSet<Relocation> {
        &self.relocations
    }

    /// Every relocation naming one symbol.
    pub fn relocations_for(&self, symbol: BundleSymbol) -> impl Iterator<Item = &Relocation> {
        self.relocations
            .iter()
            .filter(move |relocation| relocation.symbol == symbol)
    }

    /// Every value a program reads from the target, in canonical order.
    #[must_use]
    pub const fn introspections(&self) -> &BTreeSet<IntrospectionReference> {
        &self.introspections
    }

    /// Every introspection reference to one symbol.
    pub fn introspections_for(
        &self,
        symbol: BundleSymbol,
    ) -> impl Iterator<Item = &IntrospectionReference> {
        self.introspections
            .iter()
            .filter(move |reference| reference.symbol == symbol)
    }

    /// The resource formulas, by program role and dimension.
    #[must_use]
    pub const fn formulas(
        &self,
    ) -> &BTreeMap<ProgramRole, BTreeMap<ResourceDimension, ShapeResourceFormula>> {
        &self.formulas
    }

    /// The resource dimensions this bundle does not establish, and who
    /// owes each of them.
    #[must_use]
    pub const fn outstanding_dimensions(&self) -> &BTreeMap<ResourceDimension, ResourceObligation> {
        &self.outstanding_dimensions
    }

    /// The selected proof-pattern identities, in canonical order.
    #[must_use]
    pub const fn selected_patterns(&self) -> &BTreeSet<BackendPatternId> {
        &self.patterns
    }

    /// Every ABI assumption the emitted programs rest on.
    #[must_use]
    pub const fn abi_assumptions(&self) -> &BTreeSet<AbiAssumption> {
        &self.abi
    }

    /// Every target evidence requirement the bundle's correctness rests
    /// on, which no part of this wave discharges.
    #[must_use]
    pub const fn target_evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }

    /// The outstanding lifecycle exits, which are never none.
    #[must_use]
    pub const fn outstanding_lifecycle(&self) -> &OutstandingLifecycle {
        &self.lifecycle
    }

    /// This artifact's status (§13.5).
    ///
    /// Always [`BackendArtifactStatus::Prototype`], and read-only: the
    /// promotion §13.5 admits requires full operation evidence, the
    /// artifact carrying that evidence does not exist yet, and a setter
    /// or a field for it would let a bundle claim the promotion without
    /// the evidence.
    #[must_use]
    pub const fn status(&self) -> BackendArtifactStatus {
        BackendArtifactStatus::Prototype
    }

    /// The exact total encoded program bytes of every emitted leaf.
    ///
    /// The measure the policy's own objective is stated in, computed
    /// from the same projection the per-leaf figures come from.
    #[must_use]
    pub fn total_script_bytes(&self) -> u64 {
        self.constructor
            .leaves
            .values()
            .filter_map(|leaf| leaf.resources.charged(ResourceDimension::ScriptBytes))
            .fold(0, u64::saturating_add)
    }
}

// --- Emission ---------------------------------------------------------

/// Emit the candidate relocatable bundle for one validated plan.
///
/// The order is the one §8.1 fixes: the operation is assessed first,
/// the emission gate of §8.3 runs against each shape's pattern census,
/// and only then is anything built. A refusal returns no partial
/// bundle.
///
/// # Errors
///
/// [`BundleRefusal::PlanClaimsReleaseComplete`] for a plan that is not
/// this type's subject; [`BundleRefusal::EmissionRefused`] when the
/// §8.3 gate refuses; and any of the construction refusals
/// [`BundleRefusal`] names when a program, a placement, a relocation,
/// or a layout does not check out.
pub fn emit_candidate_bundle(
    target: &ReviewedElementsTapscriptDefinition,
    plan: &ValidatedTargetOperationPlan,
    policy: CompactAshBackendPolicy,
    symbols: CompactAshSymbols,
) -> Result<CandidateRelocatableTapscriptBundle, BundleRefusal> {
    let lifecycle = outstanding_lifecycle(plan)?;
    let assessment = assess_operation_plan(target, plan).map_err(BundleRefusal::Assessment)?;

    let shapes: Vec<CompactAshShape> = policy.cardinality().shapes().collect();
    if shapes.is_empty() {
        return Err(BundleRefusal::EmptyCandidateShapeSet);
    }

    let mut patterns = BTreeSet::new();
    let mut abi = BTreeSet::new();
    let mut evidence = BTreeSet::new();
    for shape in &shapes {
        let census = crate::pattern::operation_patterns(target, &symbols, *shape)
            .map_err(BundleRefusal::Program)?;
        assessment.emission_admissible(&census).map_err(|refusal| {
            BundleRefusal::EmissionRefused {
                shape: *shape,
                refusal,
            }
        })?;
        for (id, pattern) in &census {
            patterns.insert(*id);
            abi.extend(pattern.abi().iter().copied());
            evidence.extend(pattern.evidence().iter().copied());
        }
    }

    let preferred: BTreeSet<BackendPatternId> =
        policy.pattern_preference().iter().copied().collect();
    if preferred != patterns {
        return Err(BundleRefusal::PatternSelectionMismatch {
            preferred_only: preferred.difference(&patterns).copied().collect(),
            admitted_only: patterns.difference(&preferred).copied().collect(),
        });
    }

    let (leaves, sharing) = leaf_programs(target, &symbols, &shapes)?;
    let layouts = layouts(&shapes)?;
    let relocations = all_relocations(target, &symbols, &shapes, &leaves)?;
    let introspections = introspection_references(&leaves);
    let symbol_table = symbol_table(target, &symbols, &leaves);
    let placements = relation_placements(plan, &policy)?;
    let formulas = resource_formulas(&leaves, &shapes);

    let contract = target.definition();
    let constructor = StaticAshConstructor {
        contract: contract.version(),
        leaf_version: contract.leaf_version(),
        representation: policy.representation(),
        value_policy: ExplicitValuePolicy::RequireExplicitPrefixEquality,
        internal_key: InternalKeyPolicy::UnspendableWithResidualDiscreteLogAssumption,
        key_path: KeyPathPolicy::NoAcceptedEscape,
        assumptions: BTreeSet::from([
            ConstructorAssumption::ResidualDiscreteLogOnUnspendableInternalKey,
            ConstructorAssumption::InternalKeyUnspendabilityVerifiableFromPublicData,
        ]),
        shapes: policy.cardinality().clone(),
        leaves,
        sharing,
    };

    Ok(CandidateRelocatableTapscriptBundle {
        plan: plan.clone(),
        policy,
        assessment,
        constructor,
        layouts,
        placements,
        symbols: symbol_table,
        unresolved: symbols,
        relocations,
        introspections,
        formulas,
        outstanding_dimensions: outstanding_dimensions(),
        patterns,
        abi,
        evidence,
        lifecycle,
    })
}

/// The plan's outstanding exits, refusing a release-complete plan.
fn outstanding_lifecycle(
    plan: &ValidatedTargetOperationPlan,
) -> Result<OutstandingLifecycle, BundleRefusal> {
    let mut outstanding: BTreeSet<LifecycleRequirement> =
        plan.lifecycle().outstanding().cloned().collect();

    let Some(least) = outstanding.iter().next().cloned() else {
        return Err(BundleRefusal::PlanClaimsReleaseComplete);
    };
    outstanding.remove(&least);

    Ok(OutstandingLifecycle {
        least,
        rest: outstanding,
    })
}

/// The dimensions §13.3 keeps separately typed and this wave does not
/// establish, each with the layer that owes it.
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
    symbols: &CompactAshSymbols,
    shapes: &[CompactAshShape],
) -> Result<(BTreeMap<LeafRole, LeafProgram>, BTreeSet<SharedLeafProof>), BundleRefusal> {
    let mut leaves: BTreeMap<LeafRole, LeafProgram> = BTreeMap::new();
    let mut member_shapes: BTreeMap<u8, BTreeSet<CompactAshShape>> = BTreeMap::new();

    for shape in shapes {
        let role = LeafRole::Coordinator { shape: *shape };
        let program =
            coordinator_program(target, symbols, *shape).map_err(BundleRefusal::Program)?;
        leaves.insert(
            role,
            leaf_program(target, role, program, BTreeSet::from([*shape]))?,
        );

        let member = member_program(target, symbols, *shape).map_err(BundleRefusal::Program)?;
        let member_role = LeafRole::Member {
            ash_inputs: shape.ash_inputs(),
        };
        member_shapes
            .entry(shape.ash_inputs())
            .or_default()
            .insert(*shape);

        // §11.2 admits sharing only where one typed proof establishes
        // that the shared leaf enforces the exact same relation for
        // every shape using it. The proof is the rebuild: this shape's
        // own program, built from this shape, compared instruction by
        // instruction with the leaf already admitted for the batch
        // size. Equal typed instructions is equal enforcement, because
        // a program is nothing but its instructions.
        match leaves.get(&member_role) {
            Some(existing) if existing.program != member => {
                return Err(BundleRefusal::SharedLeafRelationMismatch {
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
    for (ash_inputs, served) in member_shapes {
        let role = LeafRole::Member { ash_inputs };
        if let Some(leaf) = leaves.get_mut(&role) {
            leaf.shapes.clone_from(&served);
        }
        if served.len() > 1 {
            sharing.insert(SharedLeafProof {
                leaf: role,
                shapes: served,
                ground: SharingGround::IdenticalTypedProgramRecomputedPerShape,
            });
        }
    }

    Ok((leaves, sharing))
}

/// One leaf's record: its cost and its witness role, both computed.
fn leaf_program(
    target: &ReviewedElementsTapscriptDefinition,
    leaf: LeafRole,
    program: TapscriptProgram,
    shapes: BTreeSet<CompactAshShape>,
) -> Result<LeafProgram, BundleRefusal> {
    let empty = AbstractStackState::from_main(Vec::new());
    let limits = AbstractLimits::for_target(target);
    let result = validate_program(target, &program, &empty, limits)
        .map_err(|_| BundleRefusal::LeafDoesNotScheduleFromEmptyStack { leaf })?;
    if result.success().is_empty() {
        return Err(BundleRefusal::LeafNeverSucceeds { leaf });
    }

    let mut dimensions = resource_projection(target, &program);
    dimensions.insert(
        ResourceDimension::InitialStackItems,
        u64::try_from(empty.depth()).unwrap_or(u64::MAX),
    );

    Ok(LeafProgram {
        leaf,
        resources: ProgramResources { dimensions },
        witness: WitnessRole {
            leaf,
            components: BTreeSet::from([
                WitnessComponent::LeafScript,
                WitnessComponent::ControlBlock,
            ]),
            data_items: empty.depth(),
        },
        program,
        shapes,
    })
}

// --- Layout -----------------------------------------------------------

/// Every shape's exact layout.
fn layouts(
    shapes: &[CompactAshShape],
) -> Result<BTreeMap<CompactAshShape, ConcreteLayout>, BundleRefusal> {
    let mut layouts = BTreeMap::new();
    for shape in shapes {
        layouts.insert(*shape, layout_of(*shape)?);
    }
    Ok(layouts)
}

/// One shape's layout, with both position censuses checked exact.
fn layout_of(shape: CompactAshShape) -> Result<ConcreteLayout, BundleRefusal> {
    let (ash_first, ash_end) = shape.ash_range();
    let (sponsor_first, sponsor_end) = shape.sponsor_range();

    let mut inputs = vec![InputPlacement {
        role: InputRole::Coordinator,
        first: u16::from(ash_first),
        end: u16::from(ash_first) + 1,
    }];
    inputs.push(InputPlacement {
        role: InputRole::Member,
        first: u16::from(ash_first) + 1,
        end: u16::from(ash_end),
    });
    if shape.sponsored() {
        inputs.push(InputPlacement {
            role: InputRole::Sponsor,
            first: u16::from(sponsor_first),
            end: u16::from(sponsor_end),
        });
    }

    let mut outputs = vec![OutputPlacement {
        role: OutputRole::Successor,
        position: 0,
    }];
    let mut next = 1;
    if shape.sponsor_change() == SponsorChangePresence::Present {
        outputs.push(OutputPlacement {
            role: OutputRole::SponsorChange,
            position: next,
        });
        next += 1;
    }
    if shape.sponsored() {
        outputs.push(OutputPlacement {
            role: OutputRole::TargetFee,
            position: next,
        });
    }

    census(shape, FieldSide::Input, shape.inputs(), |position| {
        inputs
            .iter()
            .filter(|placement| placement.first <= position && position < placement.end)
            .count()
    })?;
    census(shape, FieldSide::Output, shape.outputs(), |position| {
        outputs
            .iter()
            .filter(|placement| placement.position == position)
            .count()
    })?;

    Ok(ConcreteLayout {
        shape,
        inputs,
        outputs,
    })
}

/// Require every position below `total` to be claimed exactly once.
fn census(
    shape: CompactAshShape,
    side: FieldSide,
    total: u16,
    claims: impl Fn(u16) -> usize,
) -> Result<(), BundleRefusal> {
    for position in 0..total {
        match claims(position) {
            1 => {}
            0 => {
                return Err(BundleRefusal::LayoutPositionUnaccounted {
                    shape,
                    side,
                    position,
                });
            }
            _ => {
                return Err(BundleRefusal::LayoutPositionClaimedTwice {
                    shape,
                    side,
                    position,
                });
            }
        }
    }
    Ok(())
}

// --- Symbols and relocations ------------------------------------------

/// Every symbol pushed into a program, in canonical order.
const PROGRAM_SYMBOLS: &[BundleSymbol] = &[
    BundleSymbol::ClosedAsset,
    BundleSymbol::ReserveAsset,
    BundleSymbol::SponsorChangeProgram,
    BundleSymbol::SponsorChangeProgramVersion,
    BundleSymbol::TargetFeeRoleProgramDigest,
];

/// The symbol set with exactly one symbol replaced.
///
/// The probe's value has to differ from the real one and nothing else
/// has to hold: the programs are rebuilt from the same builders over
/// the same shape, so their instruction counts are equal by
/// construction and the positions that moved are exactly the positions
/// that symbol occupies. A width-preserving probe is not required
/// because no byte offset is taken.
fn probe_symbols(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &CompactAshSymbols,
    symbol: BundleSymbol,
) -> Result<CompactAshSymbols, BundleRefusal> {
    let flip =
        |item: &StackItem| -> Vec<u8> { item.bytes().iter().map(|byte| byte ^ 0xff).collect() };
    let other = |value: i64| -> i64 { if value == 1 { 2 } else { 1 } };

    let mut closed = symbols.closed_asset().bytes().to_vec();
    let mut reserve = symbols.reserve_asset().bytes().to_vec();
    let mut change = symbols.sponsor_change_program().bytes().to_vec();
    let mut change_version = symbols.sponsor_change_version();
    let mut digest = symbols.fee_program_digest().bytes().to_vec();

    match symbol {
        BundleSymbol::ClosedAsset => closed = flip(symbols.closed_asset()),
        BundleSymbol::ReserveAsset => reserve = flip(symbols.reserve_asset()),
        BundleSymbol::SponsorChangeProgram => change = flip(symbols.sponsor_change_program()),
        BundleSymbol::SponsorChangeProgramVersion => change_version = other(change_version),
        BundleSymbol::TargetFeeRoleProgramDigest => digest = flip(symbols.fee_program_digest()),
        _ => {}
    }

    CompactAshSymbols::new(target, closed, reserve, change, change_version, digest)
        .map_err(BundleRefusal::Program)
}

/// Every relocation of the bundle: program sites and constructor
/// bindings alike.
fn all_relocations(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &CompactAshSymbols,
    shapes: &[CompactAshShape],
    leaves: &BTreeMap<LeafRole, LeafProgram>,
) -> Result<BTreeSet<Relocation>, BundleRefusal> {
    let mut relocations = BTreeSet::new();

    for symbol in PROGRAM_SYMBOLS {
        let probe = probe_symbols(target, symbols, *symbol)?;
        for (role, leaf) in leaves {
            let rebuilt = rebuild(target, &probe, *role, shapes)?;
            let Some(rebuilt) = rebuilt else {
                continue;
            };
            if rebuilt.len() != leaf.program.len() {
                return Err(BundleRefusal::ProbeProgramShapeChanged {
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

            // One symbol can hold two roles in one program: the closed
            // asset is compared with an input asset in the recognition
            // fragments and with output 0's asset in the successor
            // fragment, and those are different target roles even
            // though the pushed bytes are one symbol. So the sites are
            // grouped by the role the instructions establish for them,
            // and each group is its own relocation with its own exact
            // multiplicity.
            for (target_role, sites) in site_roles(&leaf.program, &indices, *role, *symbol)? {
                let Some(multiplicity) = NonZeroUsize::new(sites.len()) else {
                    continue;
                };
                relocations.insert(Relocation {
                    symbol: *symbol,
                    role: target_role,
                    site: RelocationSite::ProgramInstructions {
                        leaf: *role,
                        indices: sites,
                    },
                    width: symbol_width(target, symbols, *symbol),
                    encoding: symbol_encoding(*symbol),
                    multiplicity,
                    substitution: SubstitutionMode::StructuredBeforeSerialization,
                });
            }
        }
    }

    relocations.extend(constructor_relocations(target, symbols, leaves));
    Ok(relocations)
}

/// One leaf's program, rebuilt from a probe symbol set.
fn rebuild(
    target: &ReviewedElementsTapscriptDefinition,
    probe: &CompactAshSymbols,
    leaf: LeafRole,
    shapes: &[CompactAshShape],
) -> Result<Option<TapscriptProgram>, BundleRefusal> {
    let program = match leaf {
        LeafRole::Coordinator { shape } => coordinator_program(target, probe, shape),
        LeafRole::Member { ash_inputs } => {
            let Some(shape) = shapes.iter().find(|shape| shape.ash_inputs() == ash_inputs) else {
                return Ok(None);
            };
            member_program(target, probe, *shape)
        }
    };
    program.map(Some).map_err(BundleRefusal::Program)
}

/// What field one introspection primitive reads.
#[derive(Clone, Copy, PartialEq, Eq)]
enum InspectedField {
    Asset,
    Program,
    Value,
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
                _ => return None,
            })
        })
}

/// The target roles one symbol's sites hold, grouped by role.
///
/// Read from the instructions rather than declared: each site must be a
/// pushed literal dominated by an introspection primitive whose field
/// the symbol could be compared with. A symbol sitting under the wrong
/// primitive — an asset comparand under a value introspection, say — is
/// a refusal, which is what makes this a check rather than a label.
fn site_roles(
    program: &TapscriptProgram,
    indices: &BTreeSet<usize>,
    leaf: LeafRole,
    symbol: BundleSymbol,
) -> Result<BTreeMap<TargetRole, BTreeSet<usize>>, BundleRefusal> {
    let mut roles: BTreeMap<TargetRole, BTreeSet<usize>> = BTreeMap::new();

    for index in indices {
        if !matches!(
            program.instructions().get(*index),
            Some(TapscriptInstruction::Push(_))
        ) {
            return Err(BundleRefusal::RelocationSiteIsNotAPush {
                leaf,
                index: *index,
            });
        }
        let context = introspection_context(program, *index);
        let role = match (symbol, context) {
            (
                BundleSymbol::ClosedAsset | BundleSymbol::ReserveAsset,
                Some((side, InspectedField::Asset)),
            ) => TargetRole::AssetComparand { side },
            (BundleSymbol::SponsorChangeProgram, Some((side, InspectedField::Program))) => {
                TargetRole::ProgramComparand { side }
            }
            (BundleSymbol::SponsorChangeProgramVersion, Some((side, InspectedField::Program))) => {
                TargetRole::ProgramVersionComparand { side }
            }
            (
                BundleSymbol::TargetFeeRoleProgramDigest,
                Some((FieldSide::Output, InspectedField::Program)),
            ) => TargetRole::FeeProgramDigestComparand,
            _ => {
                return Err(BundleRefusal::RelocationSiteContextMismatch {
                    leaf,
                    index: *index,
                    symbol,
                });
            }
        };
        roles.entry(role).or_default().insert(*index);
    }

    Ok(roles)
}

/// Every place a leaf reads the ASH constructor's program from the
/// target instead of carrying a literal for it.
///
/// Read from the emitted instructions, on the same principle as
/// [`site_roles`]: the census is what the programs do, never a claim
/// made beside them. The idiom is exact — this leaf's own input index,
/// then that input's witness program — and nothing else in the
/// candidate emits it, so counting occurrences counts the reads.
fn introspection_references(
    leaves: &BTreeMap<LeafRole, LeafProgram>,
) -> BTreeSet<IntrospectionReference> {
    let mut references = BTreeSet::new();

    for (role, leaf) in leaves {
        let sites = leaf
            .program
            .instructions()
            .windows(2)
            .filter(|pair| {
                matches!(
                    (&pair[0], &pair[1]),
                    (
                        TapscriptInstruction::Opcode(OpcodeId::PushCurrentInputIndex),
                        TapscriptInstruction::Opcode(OpcodeId::InspectInputScriptPubKey),
                    )
                )
            })
            .count();
        let Some(sites) = NonZeroUsize::new(sites) else {
            continue;
        };

        references.insert(IntrospectionReference {
            symbol: BundleSymbol::AshConstructorProgram,
            leaf: *role,
            role: TargetRole::ProgramComparand {
                side: FieldSide::Input,
            },
            sites,
        });
    }

    references
}

/// The relocations of the symbols no program pushes.
fn constructor_relocations(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &CompactAshSymbols,
    leaves: &BTreeMap<LeafRole, LeafProgram>,
) -> BTreeSet<Relocation> {
    let mut relocations = BTreeSet::new();
    let one = NonZeroUsize::MIN;

    for (symbol, role) in [
        (
            BundleSymbol::UnspendableInternalKey,
            TargetRole::TaprootInternalKey,
        ),
        (
            BundleSymbol::TargetLeafVersion,
            TargetRole::TaprootLeafVersion,
        ),
        (
            BundleSymbol::CandidateAshBound,
            TargetRole::CandidateShapeBound,
        ),
        (
            BundleSymbol::CandidateSponsorBound,
            TargetRole::CandidateShapeBound,
        ),
    ] {
        relocations.insert(Relocation {
            symbol,
            role,
            site: RelocationSite::ConstructorBinding,
            width: symbol_width(target, symbols, symbol),
            encoding: symbol_encoding(symbol),
            multiplicity: one,
            substitution: SubstitutionMode::StructuredBeforeSerialization,
        });
    }

    for (leaf, program) in leaves {
        let symbol = match leaf {
            LeafRole::Coordinator { shape } => BundleSymbol::CoordinatorProgram { shape: *shape },
            LeafRole::Member { ash_inputs } => BundleSymbol::MemberProgram {
                ash_inputs: *ash_inputs,
            },
        };
        relocations.insert(Relocation {
            symbol,
            role: TargetRole::CommittedLeafScript,
            site: RelocationSite::ConstructorBinding,
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
const fn symbol_class(symbol: BundleSymbol) -> Option<EncodingClass> {
    Some(match symbol {
        BundleSymbol::ClosedAsset | BundleSymbol::ReserveAsset => EncodingClass::ExplicitAsset,
        BundleSymbol::AshConstructorProgram | BundleSymbol::SponsorChangeProgram => {
            EncodingClass::WitnessProgram
        }
        BundleSymbol::TargetFeeRoleProgramDigest => EncodingClass::ScriptPubKeySha256,
        BundleSymbol::SponsorChangeProgramVersion => EncodingClass::ScriptNumber,
        BundleSymbol::UnspendableInternalKey => EncodingClass::XOnlyPublicKey,
        _ => return None,
    })
}

/// How one symbol is carried.
const fn symbol_encoding(symbol: BundleSymbol) -> RelocationEncoding {
    match symbol_class(symbol) {
        Some(EncodingClass::ScriptNumber) => RelocationEncoding::ScriptNumber,
        Some(class) => RelocationEncoding::EncodedPayload { class },
        None => match symbol {
            BundleSymbol::TargetLeafVersion => RelocationEncoding::ControlBlockField,
            BundleSymbol::CoordinatorProgram { .. } | BundleSymbol::MemberProgram { .. } => {
                RelocationEncoding::TapscriptLeafScript
            }
            _ => RelocationEncoding::TypedParameter,
        },
    }
}

/// What one symbol occupies, read from the reviewed contract where the
/// contract fixes it and from the laid-out value where it does not.
fn symbol_width(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &CompactAshSymbols,
    symbol: BundleSymbol,
) -> SymbolWidth {
    let laid_out = |item: &StackItem| item.len();
    let bytes = match symbol {
        BundleSymbol::ClosedAsset => laid_out(symbols.closed_asset()),
        BundleSymbol::ReserveAsset => laid_out(symbols.reserve_asset()),
        BundleSymbol::SponsorChangeProgram => laid_out(symbols.sponsor_change_program()),
        BundleSymbol::TargetFeeRoleProgramDigest => laid_out(symbols.fee_program_digest()),
        BundleSymbol::SponsorChangeProgramVersion => {
            StackItem::script_number(target, symbols.sponsor_change_version())
                .map_or(0, |item| item.len())
        }
        BundleSymbol::UnspendableInternalKey | BundleSymbol::TargetLeafVersion => 0,
        // The ASH constructor's program has no link-time value at all,
        // so there is no laid-out width to read and no field of this
        // bundle it occupies.
        BundleSymbol::AshConstructorProgram
        | BundleSymbol::CandidateAshBound
        | BundleSymbol::CandidateSponsorBound
        | BundleSymbol::CoordinatorProgram { .. }
        | BundleSymbol::MemberProgram { .. } => return SymbolWidth::Unserialized,
    };

    if symbol == BundleSymbol::TargetLeafVersion {
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
fn symbol_table(
    target: &ReviewedElementsTapscriptDefinition,
    symbols: &CompactAshSymbols,
    leaves: &BTreeMap<LeafRole, LeafProgram>,
) -> BTreeMap<BundleSymbol, SymbolEntry> {
    let mut table = BTreeMap::new();

    let mut admit = |symbol: BundleSymbol, binding: SymbolBinding, width: SymbolWidth| {
        table.insert(
            symbol,
            SymbolEntry {
                binding,
                width,
                encoding: symbol_encoding(symbol),
            },
        );
    };

    for symbol in PROGRAM_SYMBOLS
        .iter()
        .copied()
        .chain([BundleSymbol::UnspendableInternalKey])
    {
        admit(
            symbol,
            SymbolBinding::ResolvedAtLink,
            symbol_width(target, symbols, symbol),
        );
    }
    for symbol in [
        BundleSymbol::TargetLeafVersion,
        BundleSymbol::CandidateAshBound,
        BundleSymbol::CandidateSponsorBound,
    ] {
        admit(
            symbol,
            SymbolBinding::DefinedByBundle,
            symbol_width(target, symbols, symbol),
        );
    }
    admit(
        BundleSymbol::AshConstructorProgram,
        SymbolBinding::ReadFromTargetAtSpendTime,
        symbol_width(target, symbols, BundleSymbol::AshConstructorProgram),
    );
    for (leaf, program) in leaves {
        let symbol = match leaf {
            LeafRole::Coordinator { shape } => BundleSymbol::CoordinatorProgram { shape: *shape },
            LeafRole::Member { ash_inputs } => BundleSymbol::MemberProgram {
                ash_inputs: *ash_inputs,
            },
        };
        admit(
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

// --- Relation placement -----------------------------------------------

/// Place every relation-case on concrete carriers.
fn relation_placements(
    plan: &ValidatedTargetOperationPlan,
    policy: &CompactAshBackendPolicy,
) -> Result<BTreeMap<RelationCaseKey, ConcreteRelationPlacement>, BundleRefusal> {
    // Which family the operation is anchored in is the compiler's own
    // statement, read from its operation-global carrier. A target
    // package that named the object itself would be deciding semantic
    // identity, which §1.2 puts above this boundary.
    let anchor = plan
        .carriers()
        .flat_map(|requirement| requirement.alternatives.iter())
        .flat_map(|alternative| alternative.carriers.iter())
        .find_map(|placed| match &placed.carrier {
            CarrierRole::OperationGlobal { anchor, .. } => Some(*anchor),
            _ => None,
        })
        .ok_or(BundleRefusal::NoOperationAnchor)?;

    let site = |carrier: &CarrierRole| -> Option<ConcreteCarrierSite> {
        match carrier {
            CarrierRole::OperationGlobal { .. } => Some(ConcreteCarrierSite::CoordinatorLeaf),
            CarrierRole::InputFamilyCoordinator { object } if *object == anchor => {
                Some(ConcreteCarrierSite::CoordinatorLeaf)
            }
            CarrierRole::EveryInputFamilyMember { object } if *object == anchor => {
                Some(ConcreteCarrierSite::MemberLeaf)
            }
            // A family outside the anchor is the sponsor region, which
            // this candidate places no protocol leaf on: §12.9 has the
            // coordinator prove the region rather than each sponsor
            // input prove itself. An assignment naming one is therefore
            // not realizable here, which is a property of the candidate
            // and not a defect in the plan.
            CarrierRole::InputFamilyCoordinator { .. }
            | CarrierRole::EveryInputFamilyMember { .. } => None,
            CarrierRole::BackendStructural { .. } => Some(ConcreteCarrierSite::BundleStructure),
            CarrierRole::ExternalEvidence { .. } => Some(ConcreteCarrierSite::OutsideBundle),
        }
    };

    let mut placements = BTreeMap::new();
    for requirement in plan.carriers() {
        let offered = NonZeroUsize::new(requirement.alternatives.len()).ok_or_else(|| {
            BundleRefusal::NoRealizableCarrierAssignment {
                relation_case: requirement.relation_case.clone(),
            }
        })?;

        let realizable: Vec<_> = requirement
            .alternatives
            .iter()
            .filter(|alternative| {
                alternative
                    .carriers
                    .iter()
                    .all(|placed| site(&placed.carrier).is_some())
            })
            .collect();

        let count = NonZeroUsize::new(realizable.len()).ok_or_else(|| {
            BundleRefusal::NoRealizableCarrierAssignment {
                relation_case: requirement.relation_case.clone(),
            }
        })?;

        // One survivor needs no tie-break. Several do, and §7.3 permits
        // a least-key selection only where the policy states one — so a
        // policy that states none refuses rather than takes the first.
        let selected = if count.get() == 1 {
            realizable[0]
        } else {
            match policy.tie_break() {
                TieBreak::LexicographicLeastKey { .. } => {
                    realizable.iter().copied().min().ok_or_else(|| {
                        BundleRefusal::NoRealizableCarrierAssignment {
                            relation_case: requirement.relation_case.clone(),
                        }
                    })?
                }
                TieBreak::NoneStated => {
                    return Err(BundleRefusal::TiedCarrierAssignments {
                        relation_case: requirement.relation_case.clone(),
                        tied: count.get(),
                    });
                }
            }
        };

        let placement = ConcreteRelationPlacement {
            relation_case: requirement.relation_case.clone(),
            sites: selected
                .carriers
                .iter()
                .filter_map(|placed| site(&placed.carrier))
                .collect(),
            offered,
            realizable: count,
        };
        if placements
            .insert(requirement.relation_case.clone(), placement)
            .is_some()
        {
            return Err(BundleRefusal::DuplicateCarrierRequirement);
        }
    }

    Ok(placements)
}

// --- Resource formulas ------------------------------------------------

/// One shape's three counts, as the formula's axes.
fn shape_axes(shape: CompactAshShape) -> (i64, i64, i64) {
    (
        i64::from(shape.ash_inputs()),
        i64::from(shape.sponsor_inputs()),
        match shape.sponsor_change() {
            SponsorChangePresence::Absent => 0,
            SponsorChangePresence::Present => 1,
        },
    )
}

/// Every role's resource behaviour across the candidate shape set.
fn resource_formulas(
    leaves: &BTreeMap<LeafRole, LeafProgram>,
    shapes: &[CompactAshShape],
) -> BTreeMap<ProgramRole, BTreeMap<ResourceDimension, ShapeResourceFormula>> {
    let mut formulas: BTreeMap<ProgramRole, BTreeMap<ResourceDimension, ShapeResourceFormula>> =
        BTreeMap::new();

    let mut dimensions = BTreeSet::new();
    for leaf in leaves.values() {
        dimensions.extend(leaf.resources.dimensions.keys().copied());
    }

    for role in [ProgramRole::Coordinator, ProgramRole::Member] {
        for dimension in &dimensions {
            let mut measurements = BTreeMap::new();
            for shape in shapes {
                let key = match role {
                    ProgramRole::Coordinator => LeafRole::Coordinator { shape: *shape },
                    ProgramRole::Member => LeafRole::Member {
                        ash_inputs: shape.ash_inputs(),
                    },
                };
                if let Some(measure) = leaves
                    .get(&key)
                    .and_then(|leaf| leaf.resources.charged(*dimension))
                {
                    measurements.insert(*shape, measure);
                }
            }
            if measurements.is_empty() {
                continue;
            }
            let model = fit_model(&measurements);
            formulas.entry(role).or_default().insert(
                *dimension,
                ShapeResourceFormula {
                    role,
                    dimension: *dimension,
                    model,
                    measurements,
                },
            );
        }
    }

    formulas
}

/// Fit the affine model of one measurement table (§13.3).
///
/// Public because the linker refits it. Substitution changes the
/// pushed literals' widths, so the exact encoded byte length of every
/// linked program differs from the pre-link one and the coefficients
/// read off the pre-link table no longer describe anything. §1.12
/// keeps one authored source per semantic object, so the linker calls
/// this rather than reimplementing the fit and risking two answers to
/// one question.
#[must_use]
pub fn fit_shape_model(measurements: &BTreeMap<CompactAshShape, u64>) -> ResourceModel {
    fit_model(measurements)
}

/// What one fitted model predicts at one shape, where a model exists.
///
/// The companion of [`fit_shape_model`], public for the same reason:
/// a refitted model has to be checkable by whoever refitted it.
#[must_use]
pub fn predict_shape_model(model: ResourceModel, shape: CompactAshShape) -> Option<i64> {
    predict_model(model, shape)
}

/// The affine model of one measurement table, or none.
///
/// The coefficients are exact differences taken inside the table, and
/// the model is then recomputed at every measured shape. A model that
/// misses one shape is discarded whole: there is no best-fit here, and
/// a partial answer would be the truncated result §1.11 refuses.
fn fit_model(measurements: &BTreeMap<CompactAshShape, u64>) -> ResourceModel {
    let Some(per_ash_input) = unit_difference(measurements, 0) else {
        return ResourceModel::ExactTableOnly;
    };
    let Some(per_sponsor_input) = unit_difference(measurements, 1) else {
        return ResourceModel::ExactTableOnly;
    };
    let Some(per_sponsor_change) = unit_difference(measurements, 2) else {
        return ResourceModel::ExactTableOnly;
    };

    let Some((shape, measure)) = measurements.iter().next() else {
        return ResourceModel::ExactTableOnly;
    };
    let Ok(measure) = i64::try_from(*measure) else {
        return ResourceModel::ExactTableOnly;
    };
    let (ash, sponsors, change) = shape_axes(*shape);
    let Some(base) = per_ash_input
        .checked_mul(ash)
        .and_then(|term| {
            per_sponsor_input
                .checked_mul(sponsors)
                .map(|next| (term, next))
        })
        .and_then(|(term, next)| {
            per_sponsor_change
                .checked_mul(change)
                .map(|last| (term, next, last))
        })
        .and_then(|(term, next, last)| {
            measure
                .checked_sub(term)
                .and_then(|value| value.checked_sub(next))
                .and_then(|value| value.checked_sub(last))
        })
    else {
        return ResourceModel::ExactTableOnly;
    };

    let model = ResourceModel::Affine {
        base,
        per_ash_input,
        per_sponsor_input,
        per_sponsor_change,
    };

    for (shape, measure) in measurements {
        let expected = i64::try_from(*measure).ok();
        if predict_model(model, *shape) != expected {
            return ResourceModel::ExactTableOnly;
        }
    }
    model
}

/// The measured difference of one axis, from a pair differing on that
/// axis by exactly one and agreeing on the others.
fn unit_difference(measurements: &BTreeMap<CompactAshShape, u64>, axis: usize) -> Option<i64> {
    for (low, low_measure) in measurements {
        for (high, high_measure) in measurements {
            let a = shape_axes(*low);
            let b = shape_axes(*high);
            let deltas = [b.0 - a.0, b.1 - a.1, b.2 - a.2];
            let differs = deltas.iter().enumerate().all(|(index, delta)| {
                if index == axis {
                    *delta == 1
                } else {
                    *delta == 0
                }
            });
            if differs {
                return i64::try_from(*high_measure)
                    .ok()?
                    .checked_sub(i64::try_from(*low_measure).ok()?);
            }
        }
    }
    None
}
