//! Why a link did not produce a bundle.
//!
//! Construction failure, never target rejection (§1.5). Every variant
//! below names something wrong with what the linker was asked to
//! resolve, and none of them is a statement about what a target node
//! would do with the result. A refusal returns no partial bundle
//! (§1.11): there is no variant carrying a half-linked artifact and no
//! entry point that returns one alongside a diagnostic.

use std::collections::BTreeSet;

use tapscript::{BundleSymbol, LeafRole, TapscriptError};
use target_elements::ResourceDimension;

use crate::graph::{ReferenceEdgeId, ReferenceNode, SccId};
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
    TreeOracleBudgetExceeded {
        /// How many leaves were offered.
        leaves: usize,
        /// The budget.
        budget: usize,
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

    // --- Status -------------------------------------------------------
    /// The bundle handed in already claims more than a candidate, so it
    /// is not this linker's subject (§1.9).
    BundleIsNotACandidate,
}
