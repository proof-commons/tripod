//! Why a link did not produce a bundle.
//!
//! Construction failure, never target rejection (§1.5). Every variant
//! below names something wrong with what the linker was asked to
//! resolve, and none of them is a statement about what a target node
//! would do with the result. A refusal returns no partial bundle
//! (§1.11): there is no variant carrying a half-linked artifact and no
//! entry point that returns one alongside a diagnostic.

use std::collections::BTreeSet;

use tapscript::upstream::{ExternalEvidenceRole, LiveTransferRepresentationPlan};
use tapscript::{
    BundleSymbol, FinalStackDefect, LeafRole, LiveBundleSymbol, LiveProgramRefusal,
    LiveTransferLeafRole, OwnerProfileDisposition, TapscriptError,
};
use target_elements::ResourceDimension;

use crate::graph::{ReferenceEdgeId, ReferenceNode, SccId};
use crate::live_symbol::{LiveLinkRole, LiveLinkSymbol, LiveSymbolType};
use crate::symbol::SymbolType;

/// One relation-case identity, as the linker reports it.
///
/// The compiler's own key, carried rather than rendered: §14.3 states
/// that display strings are not symbol identity, and the same holds for
/// the relation-case keys the carrier census is indexed by.
pub type RelationCaseKey = tapscript::upstream::RelationCaseKey;

/// Why a link refused.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LinkRefusal {
    /// A deployment parameter is not the width or form the reviewed
    /// contract fixes for the field it will be compared with.
    InvalidDeploymentParameters(TapscriptError),

    // --- Pass one: definitions ---------------------------------------
    /// Two definitions claim one typed symbol key (§14.3).
    DuplicateSymbolDefinition(BundleSymbol),
    /// A symbol the bundle references has no definition (§14.3).
    MissingSymbol(BundleSymbol),
    /// A definition's type is not the type the symbol declares (§14.3).
    IncompatibleSymbolType {
        /// The symbol.
        symbol: BundleSymbol,
        /// What the symbol's role requires.
        expected: SymbolType,
        /// What the definition carries.
        actual: SymbolType,
    },
    /// One symbol reached the census under two definition origins, so
    /// which layer settles it is ambiguous (§14.3).
    AmbiguousSymbol(BundleSymbol),

    // --- Pass two: references ----------------------------------------
    /// A reference names a node the definition census does not hold.
    UnknownReferenceTarget(ReferenceEdgeId),
    /// One reference edge's exact site count does not fit the graph's
    /// observable count type.
    ///
    /// The edge is refused rather than reported with a saturated
    /// count. The current backend emission limits rule this out; the
    /// refusal keeps a future wider emitter or constructor from making
    /// the count lossy.
    ReferenceSiteCountOverflow {
        /// The referring node.
        referrer: ReferenceNode,
        /// The referenced node.
        referent: ReferenceNode,
    },
    /// A reference's expected target type is not the resolved
    /// definition's type.
    ReferenceTypeMismatch {
        /// The reference.
        edge: ReferenceEdgeId,
        /// What the reference expects.
        expected: SymbolType,
        /// What the definition carries.
        actual: SymbolType,
    },

    // --- Cycle policy -------------------------------------------------
    /// A cyclic edge carries no explicit resolution strategy, so the
    /// component is refused rather than accepted (§14.4).
    UnsupportedReferenceCycle {
        /// The component.
        component: SccId,
        /// Its members, normalized by stable key.
        members: BTreeSet<ReferenceNode>,
        /// The edges inside it.
        edges: BTreeSet<ReferenceEdgeId>,
    },
    /// A cycle demands a static value that is a function of itself, and
    /// no admitted strategy resolves it (§14.4).
    ///
    /// Distinct from [`Self::UnsupportedReferenceCycle`]: the component
    /// is not merely unclassified, it demands a fixed point that only
    /// repeated hashing until bytes stabilize could search for, and
    /// §14.4 prohibits that search outright.
    ImpossibleStaticFixedPoint {
        /// The component.
        component: SccId,
        /// The symbol whose value would have to contain itself.
        symbol: BundleSymbol,
    },
    /// A declared strategy says the value is reconstructed in-program,
    /// and a link-time literal for it reaches an emitted leaf (§14.4).
    CycleStrategyContradictedByRelocation {
        /// The symbol.
        symbol: BundleSymbol,
        /// A leaf whose program pushes the literal.
        leaf: LeafRole,
    },

    // --- Relocation ---------------------------------------------------
    /// A relocation's symbol has no definition to substitute.
    UnresolvedRelocation(BundleSymbol),
    /// A leaf's relinked program did not rebuild.
    InvalidLinkedProgram {
        /// The leaf.
        leaf: LeafRole,
        /// Why the rebuild failed.
        cause: TapscriptError,
    },
    /// A leaf's relinked program did not change at a site the bundle
    /// records a relocation for, so the substitution did not happen.
    RelocationNotApplied {
        /// The leaf.
        leaf: LeafRole,
        /// The symbol whose site did not move.
        symbol: BundleSymbol,
    },
    /// A leaf's relinked program changed at an instruction no
    /// relocation covers, so the link mutated something untracked.
    UntrackedProgramMutation {
        /// The leaf.
        leaf: LeafRole,
        /// The instruction index.
        index: usize,
    },
    /// A relinked program did not survive the §13.2 round trip.
    RoundTripMismatch(LeafRole),

    // --- Taptree ------------------------------------------------------
    /// A leaf carries a version other than the one every leaf carries.
    InvalidLeafVersion(LeafRole),
    /// The taptree input holds no leaf, so there is nothing to commit.
    EmptyLeafSet,
    /// One leaf identity was declared more than once, so the leaf set
    /// does not know its own size and declaration order would decide
    /// which declaration survived (§14.5).
    DuplicateTreeLeaf(LeafRole),
    /// The deterministic tree exceeds the declared maximum depth.
    TreeDepthExceeded {
        /// The deepest leaf.
        leaf: LeafRole,
        /// The depth it reached.
        depth: u32,
        /// The declared maximum.
        maximum: u32,
    },
    /// The deterministic tree is not the optimum the declared objective
    /// names, checked against an independent exact oracle (§14.5).
    NonOptimalTree {
        /// What the constructed tree costs.
        constructed: u128,
        /// What the oracle says the optimum is.
        optimum: u128,
    },
    /// A tree measurement did not fit the exact domain the objective is
    /// computed in, so no cost is reported rather than a saturated one
    /// (§14.5). The leaf budget rules this out; it refuses instead of
    /// returning a number that is not any tree's cost.
    TreeCostOverflow,
    /// Two orderings of the same leaf set produced different trees, so
    /// construction is not independent of declaration order (§14.5).
    NonDeterministicTree,
    /// The exact tree oracle's leaf budget was exceeded, so the
    /// comparison §14.5 requires was not made and no tree is returned
    /// (§1.11).
    ///
    /// Raised when *no* route the caller admits reaches the leaf set:
    /// past the oracle's budget with unequal weights, or past it with
    /// equal weights under a policy that admits the oracle alone.
    TreeOracleBudgetExceeded {
        /// How many leaves were offered.
        leaves: usize,
        /// The budget.
        budget: usize,
    },
    /// The tree construction's own leaf budget was exceeded, so the
    /// module's exact-domain argument no longer covers the arithmetic and
    /// no tree is returned (§14.5, §1.11).
    ///
    /// Distinct from [`Self::TreeOracleBudgetExceeded`]: that one says no
    /// admitted route establishes the optimum, this one says the
    /// construction itself is past the bound its `u128` sufficiency
    /// argument rests on, so no number it produced would be known to be
    /// any tree's cost.
    TreeLeafBudgetExceeded {
        /// How many leaves were offered.
        leaves: usize,
        /// The budget.
        budget: usize,
    },

    // --- Live-transfer symbols (§11.2) --------------------------------
    /// Two definitions claim one typed live symbol key (§11.2).
    DuplicateLiveSymbolDefinition(Box<LiveLinkSymbol>),
    /// A live symbol the bundle references has no definition (§11.2).
    MissingLiveSymbol(Box<LiveLinkSymbol>),
    /// A live definition's type is not the type the symbol declares
    /// (§11.2).
    IncompatibleLiveSymbolType {
        /// The symbol.
        symbol: Box<LiveLinkSymbol>,
        /// What the symbol's role requires.
        expected: LiveSymbolType,
        /// What the definition carries.
        actual: LiveSymbolType,
    },
    /// One live symbol reached the census under two definition origins,
    /// so which layer settles it is ambiguous (§11.2).
    AmbiguousLiveSymbol(Box<LiveLinkSymbol>),
    /// The bundle and the link disagree about what the review
    /// establishes for the selected sighash profile (§1.7, §11.2).
    ///
    /// Both read the reviewed contract's own sighash capability, so a
    /// disagreement means they are reading different targets — which
    /// makes every signature statement in the bundle a statement about a
    /// contract this link is not resolving against.
    SighashProfileDisagreement {
        /// What the bundle recorded.
        bundle: Box<OwnerProfileDisposition>,
        /// What this link derives.
        link: Box<OwnerProfileDisposition>,
    },
    /// A §11.2 role no definition in the census fills.
    ///
    /// A role nobody fills is a role nobody has resolved, and a link that
    /// returned a bundle anyway would be publishing programs whose
    /// symbols were settled somewhere nobody can name.
    LiveSymbolRoleUnfilled {
        /// Every unfilled role, in census order.
        roles: BTreeSet<LiveLinkRole>,
    },

    // --- Live-transfer relocation (§11.1, §13.4) ----------------------
    /// A live relocation's symbol has no definition to substitute.
    UnresolvedLiveRelocation(Box<LiveLinkSymbol>),
    /// A live leaf's relinked program did not rebuild.
    InvalidLinkedLiveProgram {
        /// The leaf.
        leaf: LiveTransferLeafRole,
        /// Why the rebuild failed.
        cause: Box<LiveProgramRefusal>,
    },
    /// A live leaf's relinked program does not carry the resolved value
    /// at a site the bundle records a relocation for.
    LiveRelocationNotApplied {
        /// The leaf.
        leaf: LiveTransferLeafRole,
        /// The symbol whose site does not carry it.
        symbol: LiveBundleSymbol,
    },
    /// A live leaf's relinked program changed at an instruction no
    /// relocation covers, so the link mutated something untracked.
    UntrackedLiveProgramMutation {
        /// The leaf.
        leaf: LiveTransferLeafRole,
        /// The instruction index.
        index: usize,
    },
    /// A relinked live program did not survive the §13.2 round trip.
    LiveRoundTripMismatch(LiveTransferLeafRole),
    /// A relinked live program no longer schedules from §10.2's
    /// precondition.
    LinkedLiveProgramDoesNotSchedule {
        /// The leaf.
        leaf: LiveTransferLeafRole,
    },
    /// A relinked live program no longer satisfies §10.9.
    ///
    /// The emitter held every leaf to §10.9 before publishing it, and a
    /// substitution has no business changing a program's stack behaviour
    /// — but a leaf that stopped satisfying it after linking would be
    /// exactly the artifact §10.9 exists to refuse, published by a layer
    /// that never looked.
    LinkedLiveProgramFailsTheFinalStackRule {
        /// The leaf.
        leaf: LiveTransferLeafRole,
        /// Every way it fails, in canonical order.
        defects: Vec<FinalStackDefect>,
    },

    // --- Live-transfer carrier closure (§11.5) ------------------------
    /// A representation plan was offered a committed tree and the
    /// validated plan has no projection for it (§11.5).
    MissingLivePlanProjection {
        /// The plan.
        plan: LiveTransferRepresentationPlan,
    },
    /// A relation-case one plan requires has no site the emitted bundle
    /// serves (§11.5).
    ///
    /// The compiler-required and backend-emitted censuses failing to
    /// meet, per plan: a relation that vanished at the boundary is what
    /// §1.3 forbids, and it is a fact about one representation rather
    /// than about their union.
    LiveCarrierCensusMismatch {
        /// The plan whose census does not meet.
        plan: LiveTransferRepresentationPlan,
        /// The relation-case with no served site.
        relation_case: Box<RelationCaseKey>,
    },
    /// A placed carrier names a site no committed live leaf occupies
    /// (§11.5).
    UnreachableLiveRelationCarrier {
        /// The plan.
        plan: LiveTransferRepresentationPlan,
        /// The relation-case left uncarried.
        relation_case: Box<RelationCaseKey>,
    },
    /// A relation-case has exactly one carrying program under one plan
    /// and that program is not in the plan's committed tree (§11.5).
    UniqueLiveCarrierRemoved {
        /// The plan.
        plan: LiveTransferRepresentationPlan,
        /// The relation-case left uncarried.
        relation_case: Box<RelationCaseKey>,
        /// The program that alone carried it.
        leaf: LiveTransferLeafRole,
    },
    /// One plan's requirement is reachable only in another plan's tree
    /// (§11.5).
    ///
    /// §11.5's own sentence, as a refusal: a carrier reachable only in
    /// the explicit plan does not satisfy the private plan. The defect
    /// names the plan that starves rather than reporting that some tree
    /// somewhere carries the case, because the plan that starves is the
    /// one whose spenders would find nothing there.
    PlanStarvedOfCarrier {
        /// The plan whose requirement nothing in its own tree carries.
        starved: LiveTransferRepresentationPlan,
        /// The plan whose tree does carry it.
        reachable_in: LiveTransferRepresentationPlan,
        /// The relation-case.
        relation_case: Box<RelationCaseKey>,
    },
    /// A relation-case named as external evidence is also given a local
    /// carrying program (§11.5, §6.3).
    ///
    /// The reassignment §11.5 forbids. An external confidential-value
    /// requirement is the target's own consensus rule; a local program
    /// that appeared to discharge it would be claiming a rule it cannot
    /// evaluate, and §10.6 says plainly that no backend pattern is minted
    /// for that behaviour.
    ExternalRequirementCarriedLocally {
        /// The plan.
        plan: LiveTransferRepresentationPlan,
        /// The relation-case carried both ways.
        relation_case: Box<RelationCaseKey>,
        /// Every external role the plan leaves open.
        roles: BTreeSet<ExternalEvidenceRole>,
    },

    // --- Live-transfer taptree (§11.4) --------------------------------
    /// One live-transfer leaf identity was declared more than once, so
    /// the leaf set does not know its own size and declaration order
    /// would decide which declaration survived (§11.4).
    DuplicateLiveTreeLeaf(LiveTransferLeafRole),
    /// The deterministic live-transfer tree exceeds the declared maximum
    /// depth (§11.4).
    LiveTreeDepthExceeded {
        /// The deepest leaf.
        leaf: LiveTransferLeafRole,
        /// The depth it reached.
        depth: u32,
        /// The declared maximum.
        maximum: u32,
    },
    /// One tree was offered leaves of more than one representation plan
    /// (§11.3).
    ///
    /// The dispatch §11.3 forbids, arriving as a tree rather than as an
    /// opcode: a taproot output committing both representations' leaves
    /// lets a spender choose which semantics to run, and no care inside
    /// the programs takes that choice back.
    MixedRepresentationTree {
        /// Every representation the declarations named.
        representations: BTreeSet<LiveTransferRepresentationPlan>,
    },

    // --- Carrier closure ----------------------------------------------
    /// A relation-case the compiler requires has no placement (§14.6).
    MissingRelationCarrier(RelationCaseKey),
    /// A placed carrier names a site no linked leaf occupies (§14.6).
    UnreachableRelationCarrier(RelationCaseKey),
    /// The compiler-required and backend-emitted carrier censuses are
    /// not the same set (§14.6).
    CarrierCensusMismatch {
        /// Required by the compiler, absent from the bundle.
        required_only: BTreeSet<RelationCaseKey>,
        /// Emitted by the bundle, absent from the plan.
        emitted_only: BTreeSet<RelationCaseKey>,
    },
    /// Two carriers of one relation-case disagree about the sites they
    /// discharge it on, so the duplication is not deliberate (§14.6).
    DuplicateIncompatibleCarrier(RelationCaseKey),
    /// A relation-case has exactly one carrying program and that
    /// program is not in the committed tree (§14.6).
    UniqueCarrierRemoved {
        /// The relation-case left uncarried.
        relation_case: RelationCaseKey,
        /// The program that alone carried it.
        leaf: LeafRole,
    },

    // --- Resources ----------------------------------------------------
    /// A linked leaf charges no figure for a dimension the pre-link
    /// bundle charged.
    MissingResourceFormula {
        /// The leaf.
        leaf: LeafRole,
        /// The dimension.
        dimension: ResourceDimension,
    },
    /// A linked figure exceeds what the reviewed target admits.
    StaticTargetLimitExceeded {
        /// The dimension.
        dimension: ResourceDimension,
        /// The linked figure.
        linked: u64,
        /// The reviewed limit.
        limit: u64,
    },

    // --- Live-transfer bundle (§11.6) ---------------------------------
    /// No relocatable live-transfer bundle was offered to link.
    ///
    /// A link over nothing has no leaf set, so §7.5's key-path escape
    /// arrives by omission before any other check could run.
    NoLiveBundleOffered,
    /// A live bundle handed in already claims more than a candidate, so
    /// it is not this linker's subject (§1.12).
    LiveBundleIsNotACandidate,
    /// Two offered bundles were emitted for different validated plans.
    ///
    /// A link over two plans would be a link over two operations, and
    /// §11.5's per-plan comparison would be comparing projections nobody
    /// derived together.
    LiveBundlePlansDisagree,
    /// Two offered bundles claim one (owner, representation).
    ///
    /// §7.6 gives each owner one constructor per representation, so a
    /// second claim on one key is two constructors for one destination
    /// and no way to say which a linked output is under.
    DuplicateLinkedConstructor {
        /// The representation claimed twice.
        representation: LiveTransferRepresentationPlan,
    },

    // --- Status -------------------------------------------------------
    /// The bundle handed in already claims more than a candidate, so it
    /// is not this linker's subject (§1.9).
    BundleIsNotACandidate,
}
