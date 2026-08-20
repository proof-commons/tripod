//! The typed backend policy that selects one concrete candidate
//! (Guide-12 §7.3).
//!
//! # Where the tie-break lives, and why here
//!
//! The compiler retains every feasible target-independent plan and the
//! Phase-4 filter narrows them by operation, representation,
//! constructibility, and lifecycle — but §7.3 leaves the *target*
//! choice open on purpose: "no target-specific choice yet". Wave 4
//! therefore published a plan with no selection in it, and the
//! selection rule had nowhere to live until a package existed that
//! could name a target projection, a shape set, a sponsor profile, and
//! a proof pattern. That package is this one, so the tie-break is
//! stated here, once, as a value.
//!
//! # The selection is a refusal by default
//!
//! [`CompactAshBackendPolicy::select`] refuses unless every condition
//! §7.3 attaches to a lexicographic least-key selection holds: the
//! policy states the tie-break, the candidates have been reduced to
//! those the accepted objective cannot separate, and their semantic
//! equivalence is *established* rather than assumed. A policy that
//! silently took the first candidate would be choosing on an
//! unaccountable ground, which is exactly the failure the rule names.
//!
//! # No amount of anything appears here
//!
//! The sponsor profile this policy carries is the reviewed one, whose
//! inspected-field census is region membership, region extent, and
//! asset. There is no sponsor value field, no sponsor subtotal, and no
//! accessor that could report one (§1.6).

use std::collections::{BTreeMap, BTreeSet};

use target_elements::{
    ExplicitZeroValueRule, FeeOutputContract, SponsorInputProfile, TransactionForm,
    TransactionFormReview, reviewed_explicit_zero_value_rule, reviewed_fee_output_contract,
    reviewed_sponsor_input_profile, reviewed_transaction_forms,
};

use crate::capability::BackendPatternId;
use crate::shape::{CandidateShapeSet, demonstration_shape_set};

/// The exact reviewed target facts a compact-ASH backend projects onto.
///
/// Every field is a Wave-5 reviewed value copied from the target
/// package rather than restated: a second spelling of the fee contract
/// or the sponsor profile would be a second source of truth that could
/// drift from the reviewed one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactTargetProjection {
    fee: FeeOutputContract,
    forms: BTreeMap<TransactionForm, TransactionFormReview>,
    zero_value: BTreeSet<ExplicitZeroValueRule>,
    sponsor: SponsorInputProfile,
}

impl ExactTargetProjection {
    /// The reviewed fee-role contract.
    #[must_use]
    pub const fn fee(&self) -> &FeeOutputContract {
        &self.fee
    }

    /// The reviewed transaction-form census, sponsorless and sponsored.
    #[must_use]
    pub const fn forms(&self) -> &BTreeMap<TransactionForm, TransactionFormReview> {
        &self.forms
    }

    /// The reviewed explicit-zero-value rules.
    #[must_use]
    pub const fn zero_value(&self) -> &BTreeSet<ExplicitZeroValueRule> {
        &self.zero_value
    }

    /// The reviewed sponsor input profile.
    #[must_use]
    pub const fn sponsor(&self) -> &SponsorInputProfile {
        &self.sponsor
    }
}

/// The reviewed target projection.
#[must_use]
pub fn reviewed_target_projection() -> ExactTargetProjection {
    ExactTargetProjection {
        fee: reviewed_fee_output_contract(),
        forms: reviewed_transaction_forms(),
        zero_value: reviewed_explicit_zero_value_rule(),
        sponsor: reviewed_sponsor_input_profile(),
    }
}

/// Which representation of the ASH object the backend emits for.
///
/// One variant, because Guide 11 fixes the Phase-4 representation and
/// the Wave-4 filter already refuses every plan that selects another.
/// It is an enum rather than a unit so that a second admitted
/// representation is a new variant every match must answer, instead of
/// a field whose meaning quietly widens.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AshRepresentationSelection {
    /// The explicit form: the amount is on the chain in the clear.
    Explicit,
}

/// The candidate layout family the backend emits against.
///
/// One variant: §10 fixes one candidate layout — the ASH range first
/// with input 0 as the canonical coordinator, the sponsor suffix after
/// it, the successor at output 0, then the optional change role, then
/// the target fee role where the form carries one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LayoutFamily {
    /// Coordinator-prefixed ASH range, then the sponsor suffix.
    CoordinatorPrefixedSponsorSuffix,
}

/// The measured quantity the accepted objective compares candidates by.
///
/// Encoded program bytes, exactly as the resource projection charges
/// them. Not a heuristic and not a proxy: a candidate is preferred
/// because it is smaller by the measure the target actually bills, and
/// any other objective is a new variant with its own measurement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SelectionObjective {
    /// Total encoded program bytes across the candidate's programs.
    EncodedProgramBytes,
}

/// How the policy separates candidates the objective cannot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TieBreak {
    /// No tie-break is stated; a tie is a refusal.
    ///
    /// The honest default. A policy with no stated rule has no ground
    /// to prefer one tied candidate over another, and §7.3 permits a
    /// least-key selection *only* where the policy states it.
    NoneStated,
    /// The lexicographically least candidate key wins a tie.
    LexicographicLeastKey {
        /// The objective whose ties this rule breaks.
        objective: SelectionObjective,
    },
}

/// Whether tied candidates have been established semantically
/// equivalent.
///
/// §7.3 permits a least-key selection only when every remaining
/// candidate is semantically equivalent under the accepted objective.
/// That is not a property this module can compute from a measurement —
/// two programs of equal size can enforce different relations — so it
/// is an input, and the input names its ground.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticEquivalence {
    /// Nothing establishes it, so a tie cannot be broken.
    NotEstablished,
    /// Every candidate enforces the same relation set over the same
    /// shape set, checked rather than assumed.
    EstablishedByRelationAndShapeIdentity,
}

/// One concrete backend candidate, and what the objective measured.
///
/// The key is the pattern selection and the shape set, which is what
/// makes one concrete candidate different from another. Ordering is
/// derived, so "lexicographically least" is the ordering of the typed
/// key itself rather than of some rendering of it (§1.10: exact typed
/// comparison, no digest).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConcreteCandidate {
    patterns: BTreeSet<BackendPatternId>,
    shapes: CandidateShapeSet,
    measure: u64,
}

impl ConcreteCandidate {
    /// A candidate using `patterns` over `shapes`, measuring `measure`.
    #[must_use]
    pub const fn new(
        patterns: BTreeSet<BackendPatternId>,
        shapes: CandidateShapeSet,
        measure: u64,
    ) -> Self {
        Self {
            patterns,
            shapes,
            measure,
        }
    }

    /// The proof patterns this candidate uses.
    #[must_use]
    pub const fn patterns(&self) -> &BTreeSet<BackendPatternId> {
        &self.patterns
    }

    /// The shapes this candidate emits programs for.
    #[must_use]
    pub const fn shapes(&self) -> &CandidateShapeSet {
        &self.shapes
    }

    /// What the accepted objective measured for this candidate.
    #[must_use]
    pub const fn measure(&self) -> u64 {
        self.measure
    }
}

/// Why a policy declined to select a concrete candidate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectionRefusal {
    /// Nothing was offered.
    NoCandidates,
    /// Several candidates tie and the policy states no tie-break.
    TiedWithNoTieBreak {
        /// How many candidates the objective could not separate.
        tied: usize,
    },
    /// Several candidates tie and their equivalence is not established.
    TiedWithoutEstablishedEquivalence {
        /// How many candidates the objective could not separate.
        tied: usize,
    },
}

/// The typed policy under which a backend selects one concrete
/// candidate (§7.3).
///
/// Every field §7.3 lists is present and none is optional: a policy
/// missing one of them would be a policy that decided something without
/// saying on what ground.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompactAshBackendPolicy {
    projection: ExactTargetProjection,
    representation: AshRepresentationSelection,
    cardinality: CandidateShapeSet,
    layout: LayoutFamily,
    patterns: Vec<BackendPatternId>,
    objective: SelectionObjective,
    tie_break: TieBreak,
}

impl CompactAshBackendPolicy {
    /// A policy with every §7.3 ground stated.
    #[must_use]
    pub const fn new(
        projection: ExactTargetProjection,
        representation: AshRepresentationSelection,
        cardinality: CandidateShapeSet,
        layout: LayoutFamily,
        patterns: Vec<BackendPatternId>,
        objective: SelectionObjective,
        tie_break: TieBreak,
    ) -> Self {
        Self {
            projection,
            representation,
            cardinality,
            layout,
            patterns,
            objective,
            tie_break,
        }
    }

    /// The exact target projection.
    #[must_use]
    pub const fn projection(&self) -> &ExactTargetProjection {
        &self.projection
    }

    /// The selected ASH representation.
    #[must_use]
    pub const fn representation(&self) -> AshRepresentationSelection {
        self.representation
    }

    /// The candidate cardinality assignment.
    #[must_use]
    pub const fn cardinality(&self) -> &CandidateShapeSet {
        &self.cardinality
    }

    /// The layout family.
    #[must_use]
    pub const fn layout(&self) -> LayoutFamily {
        self.layout
    }

    /// The sponsor profile, read from the target projection.
    ///
    /// Delegated rather than stored twice: §7.3 lists the sponsor
    /// profile as its own ground, and a second copy could disagree with
    /// the reviewed one.
    #[must_use]
    pub const fn sponsor_profile(&self) -> &SponsorInputProfile {
        self.projection.sponsor()
    }

    /// The proof-pattern preference, in preference order.
    ///
    /// A vector rather than a set, because a preference that lost its
    /// order would not be a preference.
    #[must_use]
    pub fn pattern_preference(&self) -> &[BackendPatternId] {
        &self.patterns
    }

    /// The accepted objective.
    #[must_use]
    pub const fn objective(&self) -> SelectionObjective {
        self.objective
    }

    /// The deterministic tie-break.
    #[must_use]
    pub const fn tie_break(&self) -> TieBreak {
        self.tie_break
    }

    /// Whether this policy's preference names only admitted patterns.
    ///
    /// A preference for an identity nobody minted would be a decision
    /// made on something that does not exist, so the check is offered
    /// rather than left to a reader: a policy is only as good as the
    /// patterns it can actually reach.
    #[must_use]
    pub fn preference_is_backed(&self, admitted: &BTreeSet<BackendPatternId>) -> bool {
        self.patterns
            .iter()
            .all(|pattern| admitted.contains(pattern))
    }

    /// Select one concrete candidate, or refuse and say why.
    ///
    /// The candidates are measured by the caller under
    /// [`Self::objective`] — the policy states one accepted objective
    /// and there is nowhere else a measurement could have come from.
    /// The objective narrows first; only then does the tie-break apply,
    /// and only under the conditions §7.3 attaches to it.
    ///
    /// # Errors
    ///
    /// [`SelectionRefusal::NoCandidates`] when nothing was offered;
    /// [`SelectionRefusal::TiedWithNoTieBreak`] when the objective
    /// leaves several candidates and the policy states no rule; and
    /// [`SelectionRefusal::TiedWithoutEstablishedEquivalence`] when it
    /// states one but the survivors' equivalence is not established.
    pub fn select<'a>(
        &self,
        candidates: &'a BTreeSet<ConcreteCandidate>,
        equivalence: SemanticEquivalence,
    ) -> Result<&'a ConcreteCandidate, SelectionRefusal> {
        let Some(best) = candidates.iter().map(ConcreteCandidate::measure).min() else {
            return Err(SelectionRefusal::NoCandidates);
        };

        let surviving = candidates
            .iter()
            .filter(|candidate| candidate.measure() == best)
            .collect::<Vec<_>>();

        // One survivor needs no tie-break, and demanding one would
        // refuse a selection the objective already made on its own.
        let [single] = surviving[..] else {
            return self.break_tie(&surviving, equivalence);
        };
        Ok(single)
    }

    /// Apply the stated tie-break to two or more tied candidates.
    fn break_tie<'a>(
        &self,
        surviving: &[&'a ConcreteCandidate],
        equivalence: SemanticEquivalence,
    ) -> Result<&'a ConcreteCandidate, SelectionRefusal> {
        let tied = surviving.len();
        let TieBreak::LexicographicLeastKey { objective: _ } = self.tie_break else {
            return Err(SelectionRefusal::TiedWithNoTieBreak { tied });
        };
        if equivalence != SemanticEquivalence::EstablishedByRelationAndShapeIdentity {
            return Err(SelectionRefusal::TiedWithoutEstablishedEquivalence { tied });
        }

        // The candidates come from a `BTreeSet`, so they arrive in the
        // typed key's own order and the least is the first — taken by
        // an explicit minimum rather than by that ordering, so the rule
        // survives a caller passing them in some other container.
        surviving
            .iter()
            .copied()
            .min()
            .ok_or(SelectionRefusal::NoCandidates)
    }
}

/// The Phase-4 demonstration policy.
///
/// The reviewed target projection, the explicit representation Guide 11
/// fixed, the demonstration shape set, the one candidate layout family,
/// every admitted proof pattern in the order a coordinator schedules
/// them, encoded program bytes as the objective, and the stated
/// least-key tie-break.
///
/// The preference order is the schedule order rather than a ranking of
/// quality: the shape is authenticated before anything is introspected,
/// the successor before the sources it will be compared with, and the
/// permissionless audit covers the whole program, so it comes last.
#[must_use]
pub fn demonstration_policy() -> CompactAshBackendPolicy {
    CompactAshBackendPolicy::new(
        reviewed_target_projection(),
        AshRepresentationSelection::Explicit,
        demonstration_shape_set(),
        LayoutFamily::CoordinatorPrefixedSponsorSuffix,
        vec![
            BackendPatternId::CompactAshCoordinatorRoleV1,
            BackendPatternId::CompactAshShapeV1,
            BackendPatternId::CompactAshObjectRecognitionV1,
            BackendPatternId::CompactAshExplicitSumV1,
            BackendPatternId::CompactAshCanonicalPartitionV1,
            BackendPatternId::CompactAshSponsorIsolationV1,
            BackendPatternId::CompactAshMemberRoleV1,
            BackendPatternId::CompactAshPermissionlessPathV1,
        ],
        SelectionObjective::EncodedProgramBytes,
        TieBreak::LexicographicLeastKey {
            objective: SelectionObjective::EncodedProgramBytes,
        },
    )
}
