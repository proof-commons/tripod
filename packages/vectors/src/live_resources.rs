//! The candidate resource study of Guide-13 §18.
//!
//! §18.1 asks for a finite enumeration of candidate bound assignments,
//! §18.2 for complete transactions measured under one of them, §18.3 for
//! seventeen dimensions recorded separately, and §18.4 for a first-party
//! prediction laid beside a target observation over the same exact bytes.
//! This module is the enumeration and the arithmetic; the measurements
//! and the comparison are in [`crate::live_measurements`], and the
//! validated document is [`crate::live_resource_report`].
//!
//! # A bound assignment is enumerated, and never selected
//!
//! §18.5 admits one sentence — that candidate bounds fit a tested
//! candidate bundle and ABI — and forbids recording a production value.
//! No function here returns a chosen bound. What they return is what each
//! assignment *costs*: how many shapes it unrolls to, how many leaves
//! those shapes commit, how many transaction positions the widest member
//! reaches, and whether a committed tree over that leaf set can be built
//! at all under a stated deployment. Choosing among them is a decision
//! this study supplies figures for and does not make.
//!
//! # Why the enumeration is arithmetic rather than emission
//!
//! Compact ASH's study ([`crate::resource_study`]) emits every leaf of
//! every assignment, because its widest research assignment commits a few
//! hundred programs. The live enumeration's widest commits a hundred and
//! thirty-five thousand: §5.3 frees the created count from the consumed
//! one, so the shape set grows as the *product* of two axes rather than
//! along one, and emitting the enumeration would be emitting more
//! programs than any deployment could commit.
//!
//! So the counts here are closed forms, and the closed forms are checked
//! against the emission rather than trusted: [`admitted_shape_count`] and
//! [`committed_leaf_count`] are compared with
//! [`tapscript::dense_live_shape_set`] and
//! [`tapscript::static_transfer_leaf_set`] at every assignment small
//! enough to build one, and a disagreement is a defect in this module
//! rather than a tolerance to widen.

use std::collections::BTreeSet;
use std::num::{NonZeroU8, NonZeroU32};

use linker::TREE_LEAF_BUDGET;
use tapscript::upstream::LiveTransferRepresentationPlan;
use tapscript::{
    LiveTransferLeafRole, LiveTransferShapeBounds, LiveTransferShapeSet, dense_live_shape_set,
    static_transfer_leaf_set,
};

/// §18.1's research candidates for `TRANSFER_INPUT_MAX`.
///
/// Research inputs, not accepted values, and the name says so at every
/// use site. §18.1 lists these seven and calls them initial research
/// candidates; nothing in this workspace has accepted one.
pub const RESEARCH_RECEIPT_INPUT_BOUNDS: [u8; 7] = [1, 2, 4, 8, 16, 32, 64];

/// §18.1's research candidates for `TRANSFER_OUTPUT_MAX`.
///
/// The same seven values as [`RESEARCH_RECEIPT_INPUT_BOUNDS`], and a
/// separate constant rather than an alias of it: §5.3 makes the created
/// count a free axis, so the day one list moves the other must be able to
/// stay where it is.
pub const RESEARCH_RECEIPT_OUTPUT_BOUNDS: [u8; 7] = [1, 2, 4, 8, 16, 32, 64];

/// §18.1's research candidates for `FEE_SPONSOR_INPUT_MAX`.
///
/// Six values, and the first is zero: §5.8 admits the sponsorless form,
/// so a candidate that commits no sponsor region at all is a candidate
/// rather than a degenerate one.
pub const RESEARCH_SPONSOR_INPUT_BOUNDS: [u8; 6] = [0, 1, 2, 4, 8, 16];

/// Every §18.1 candidate bound assignment, in canonical order.
///
/// The product of the three research lists, receipt-input major and
/// sponsor-input minor. §18.1 requires the enumeration to be finite, and
/// it is: the lists are constants, so the assignment count is their
/// product and two calls produce the same assignments in the same order.
#[must_use]
pub fn research_bound_assignments() -> Vec<LiveTransferShapeBounds> {
    let capacity = RESEARCH_RECEIPT_INPUT_BOUNDS.len()
        * RESEARCH_RECEIPT_OUTPUT_BOUNDS.len()
        * RESEARCH_SPONSOR_INPUT_BOUNDS.len();
    let mut assignments = Vec::with_capacity(capacity);
    for inputs in RESEARCH_RECEIPT_INPUT_BOUNDS {
        for outputs in RESEARCH_RECEIPT_OUTPUT_BOUNDS {
            for sponsors in RESEARCH_SPONSOR_INPUT_BOUNDS {
                assignments.push(LiveTransferShapeBounds::new(
                    nonzero(inputs),
                    nonzero(outputs),
                    sponsors,
                ));
            }
        }
    }
    assignments
}

/// How many shapes one assignment's dense unrolling admits.
///
/// The closed form of [`tapscript::dense_live_shape_set`]'s own loop:
/// each receipt-input count from the minimum through the input bound,
/// each destination count from the minimum through the output bound, and
/// one sponsor arm per admitted sponsor region — the sponsorless arm,
/// which carries no change role, plus a present and an absent arm for
/// each nonzero sponsor count.
///
/// Saturating rather than wrapping. The widest research assignment
/// reaches a hundred and thirty-five thousand shapes, which is nowhere
/// near a `u64`, and saturation is what keeps a future list edit from
/// turning a count into a small number that reads like a measurement.
#[must_use]
pub const fn admitted_shape_count(bounds: LiveTransferShapeBounds) -> u64 {
    let inputs = bounds.receipt_inputs() as u64;
    let outputs = bounds.receipt_outputs() as u64;
    let sponsor_arms = 1 + 2 * bounds.sponsor_inputs() as u64;
    inputs.saturating_mul(outputs).saturating_mul(sponsor_arms)
}

/// How many taptree leaves one assignment's shapes commit, per
/// representation.
///
/// One coordinator leaf per admitted shape, plus one member leaf per
/// receipt-input count above one — §10.3 makes a member leaf a function
/// of the count alone, and the one-to-one shape contributes none because
/// it has no nonzero receipt position for one to serve.
///
/// Per *representation*, because §11.4 builds one tree per constructor
/// and a constructor belongs to one owner and one representation plan. A
/// deployment linking both plans for both published owners builds four
/// trees of this size rather than one tree of four times it.
#[must_use]
pub const fn committed_leaf_count(bounds: LiveTransferShapeBounds) -> u64 {
    let members = (bounds.receipt_inputs() as u64).saturating_sub(1);
    admitted_shape_count(bounds).saturating_add(members)
}

/// The widest transaction one assignment can ask a target to accept.
///
/// §18.1 requires the total-position domain to be one that cannot
/// overflow an accepted shape, and this type is where that is stated
/// once. The counts are `u8` because a bound is a candidate decision and
/// §18.1 keeps those small; the positions are `u32` because a position is
/// an index into a target's own transaction and the sum of two counts is
/// not a count.
///
/// # Why `u32` and not `u16`
///
/// The target's own position type is `u16`, and every accepted shape fits
/// it with room — the widest research assignment reaches eighty inputs
/// and sixty-six outputs. Accumulating in `u32` is what makes that a
/// *finding* rather than an assumption: a sum computed in the type it has
/// to fit could not report exceeding it, and
/// [`Self::fits_target_position_domain`] would have nothing to answer.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CandidatePositionDomain {
    inputs: u32,
    outputs: u32,
}

impl CandidatePositionDomain {
    /// The widest input and output counts one assignment reaches.
    ///
    /// §12.1 lays the receipt family out first and the sponsor suffix
    /// after it, so the input count is the exact sum of the two bounds.
    /// §12.2 lays the destinations out first, then the sponsor-change
    /// role where the shape declares one, then the target fee role where
    /// the form carries one — so the widest output count is the
    /// destination bound plus both optional roles, which a sponsored
    /// shape with change reaches together.
    #[must_use]
    pub const fn of(bounds: LiveTransferShapeBounds) -> Self {
        let optional_outputs = if bounds.sponsor_inputs() > 0 { 2 } else { 0 };
        Self {
            inputs: bounds.receipt_inputs() as u32 + bounds.sponsor_inputs() as u32,
            outputs: bounds.receipt_outputs() as u32 + optional_outputs,
        }
    }

    /// The widest input count.
    #[must_use]
    pub const fn inputs(self) -> u32 {
        self.inputs
    }

    /// The widest output count.
    #[must_use]
    pub const fn outputs(self) -> u32 {
        self.outputs
    }

    /// The widest position either family reaches.
    #[must_use]
    pub const fn widest(self) -> u32 {
        if self.inputs > self.outputs {
            self.inputs
        } else {
            self.outputs
        }
    }

    /// Whether every position this assignment reaches fits the target's
    /// own position type.
    ///
    /// The §18.1 obligation, answered rather than asserted. True for
    /// every research assignment, and it is checked at each of them
    /// rather than argued from the widest, because an argument from the
    /// widest assumes the monotonicity §18.1 declines to assume.
    #[must_use]
    pub const fn fits_target_position_domain(self) -> bool {
        self.widest() <= u16::MAX as u32
    }
}

/// Whether a committed tree over one assignment's leaves can be built.
///
/// Three answers, and the two refusals name different components. The
/// leaf budget is the linker's own and is a property of no deployment;
/// the depth capacity is a property of the deployment parameters in hand,
/// so it is computed from a stated maximum depth rather than from a
/// constant here.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum CandidateTreeAdmission {
    /// A committed tree over this leaf set is buildable.
    TreeIsBuildable {
        /// How many leaves one representation's tree carries.
        leaves: u64,
        /// The least depth an equal-weight tree over them can have.
        least_depth: u32,
    },
    /// More leaves than [`TREE_LEAF_BUDGET`] admits.
    AboveTreeLeafBudget {
        /// How many leaves the assignment commits.
        leaves: u64,
        /// The linker's own budget.
        budget: u64,
    },
    /// More leaves than the deployment's declared maximum depth carries.
    AboveDeclaredTreeDepth {
        /// How many leaves the assignment commits.
        leaves: u64,
        /// The most an equal-weight tree of the declared depth holds.
        capacity: u64,
        /// The declared maximum depth.
        declared: u32,
    },
}

impl CandidateTreeAdmission {
    /// Whether a tree is buildable.
    #[must_use]
    pub const fn is_buildable(self) -> bool {
        matches!(self, Self::TreeIsBuildable { .. })
    }

    /// The admission's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::TreeIsBuildable { .. } => "tree-is-buildable",
            Self::AboveTreeLeafBudget { .. } => "above-tree-leaf-budget",
            Self::AboveDeclaredTreeDepth { .. } => "above-declared-tree-depth",
        }
    }
}

/// Whether one assignment's committed tree fits a stated deployment.
///
/// The linker's budget first, because it binds whatever the deployment
/// says, then the deployment's own declared depth. Both refusals carry
/// the figures that produced them, so a reader repairing a candidate can
/// see which one to move and by how much.
///
/// # The depth capacity is the linker's own bound, restated
///
/// [`linker::taptree`] states it: an equal-weight tree over `n` leaves
/// has depth `ceil(log2(n))`, and that bound is attained. So a declared
/// maximum depth of `d` carries at most `2^d` leaves — computed here by
/// doubling rather than by exponentiation, so a large declared depth
/// saturates instead of overflowing.
#[must_use]
pub const fn tree_admission(
    bounds: LiveTransferShapeBounds,
    declared_depth: NonZeroU32,
) -> CandidateTreeAdmission {
    let leaves = committed_leaf_count(bounds);
    if leaves > TREE_LEAF_BUDGET as u64 {
        return CandidateTreeAdmission::AboveTreeLeafBudget {
            leaves,
            budget: TREE_LEAF_BUDGET as u64,
        };
    }

    let capacity = depth_capacity(declared_depth.get());
    if leaves > capacity {
        return CandidateTreeAdmission::AboveDeclaredTreeDepth {
            leaves,
            capacity,
            declared: declared_depth.get(),
        };
    }

    CandidateTreeAdmission::TreeIsBuildable {
        leaves,
        least_depth: least_equal_weight_depth(leaves),
    }
}

/// How many equal-weight leaves a tree of this depth holds.
///
/// `2^depth`, by doubling under a saturating ceiling, so a declared depth
/// above sixty-three reports a capacity no leaf count can exceed rather
/// than wrapping to a small one.
const fn depth_capacity(depth: u32) -> u64 {
    if depth >= 63 {
        return u64::MAX;
    }
    1_u64 << depth
}

/// The least depth an equal-weight tree over this many leaves can have.
///
/// `ceil(log2(leaves))`, counted by doubling for the same reason the
/// capacity is: the count is small, the loop is exact, and neither can
/// overflow.
const fn least_equal_weight_depth(leaves: u64) -> u32 {
    let mut depth = 0_u32;
    let mut held = 1_u64;
    while held < leaves {
        held = held.saturating_mul(2);
        depth += 1;
    }
    depth
}

/// What one enumerated assignment costs, before anything is emitted.
///
/// Every field is a closed form of the assignment's own counts, and every
/// one of them is checked against the machinery that would produce it —
/// see this module's header. The type carries no measurement of bytes:
/// bytes belong to a *complete transaction* under §18.2, and an
/// assignment nothing can commit a tree for has none.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CandidateBoundCost {
    bounds: LiveTransferShapeBounds,
    shapes: u64,
    leaves: u64,
    positions: CandidatePositionDomain,
    admission: CandidateTreeAdmission,
}

impl CandidateBoundCost {
    /// The assignment costed.
    #[must_use]
    pub const fn bounds(self) -> LiveTransferShapeBounds {
        self.bounds
    }

    /// How many shapes the dense unrolling admits.
    #[must_use]
    pub const fn shapes(self) -> u64 {
        self.shapes
    }

    /// How many leaves one representation's tree commits.
    #[must_use]
    pub const fn leaves(self) -> u64 {
        self.leaves
    }

    /// The widest transaction the assignment reaches.
    #[must_use]
    pub const fn positions(self) -> CandidatePositionDomain {
        self.positions
    }

    /// Whether a committed tree over the leaf set is buildable.
    #[must_use]
    pub const fn admission(self) -> CandidateTreeAdmission {
        self.admission
    }
}

/// Cost one enumerated assignment against a stated deployment depth.
#[must_use]
pub const fn cost_bound_assignment(
    bounds: LiveTransferShapeBounds,
    declared_depth: NonZeroU32,
) -> CandidateBoundCost {
    CandidateBoundCost {
        bounds,
        shapes: admitted_shape_count(bounds),
        leaves: committed_leaf_count(bounds),
        positions: CandidatePositionDomain::of(bounds),
        admission: tree_admission(bounds, declared_depth),
    }
}

/// Cost every §18.1 assignment, in the enumeration's own order.
///
/// One entry per assignment whatever it answered, so the result is a
/// table rather than a list of the assignments that happened to fit.
#[must_use]
pub fn bound_assignment_costs(declared_depth: NonZeroU32) -> Vec<CandidateBoundCost> {
    research_bound_assignments()
        .into_iter()
        .map(|bounds| cost_bound_assignment(bounds, declared_depth))
        .collect()
}

/// Whether one assignment's shapes all lie inside a tested candidate's
/// published set.
///
/// §18.5's fitting relation, and the direction matters: an assignment
/// fits a tested candidate when every shape the assignment's dense
/// unrolling admits has a program in the tested set. A tested set wider
/// than the assignment still fits it; a tested set missing one member
/// does not, whatever its bounds say.
#[must_use]
pub fn assignment_fits_tested_set(
    bounds: LiveTransferShapeBounds,
    tested: &LiveTransferShapeSet,
) -> bool {
    dense_live_shape_set(bounds)
        .shapes()
        .all(|shape| tested.admits(shape))
}

/// Every enumerated assignment one tested candidate set realizes.
///
/// The honest form of §18.5's result: not "these bounds were chosen" but
/// "these enumerated assignments have a program for every shape they
/// admit, in the bundle and ABI this study measured". Computed over the
/// tested set's own members rather than over its declared bounds, because
/// a set is the shapes it emits programs for (§18.1).
#[must_use]
pub fn assignments_realized_by(tested: &LiveTransferShapeSet) -> Vec<LiveTransferShapeBounds> {
    research_bound_assignments()
        .into_iter()
        .filter(|bounds| assignment_fits_tested_set(*bounds, tested))
        .collect()
}

/// The leaf set one representation commits under one assignment.
///
/// The emission's own answer, for the closed form to be checked against.
/// Built here rather than in a test so the two callers — the check and
/// the study's own tree arithmetic — cannot drift into asking different
/// questions.
#[must_use]
pub fn committed_leaves(
    representation: LiveTransferRepresentationPlan,
    bounds: LiveTransferShapeBounds,
) -> BTreeSet<LiveTransferLeafRole> {
    static_transfer_leaf_set(representation, &dense_live_shape_set(bounds))
}

/// A nonzero count, for the enumeration's own loops.
///
/// Every research receipt bound is at or above one, so the zero case is
/// unreachable; it yields the smallest nonzero count rather than
/// panicking, which is the one-to-one bound and the narrowest assignment
/// this module can mean.
const fn nonzero(value: u8) -> NonZeroU8 {
    match NonZeroU8::new(value) {
        Some(count) => count,
        None => NonZeroU8::MIN,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CandidatePositionDomain, CandidateTreeAdmission, RESEARCH_RECEIPT_INPUT_BOUNDS,
        RESEARCH_RECEIPT_OUTPUT_BOUNDS, RESEARCH_SPONSOR_INPUT_BOUNDS, admitted_shape_count,
        assignments_realized_by, bound_assignment_costs, committed_leaf_count, committed_leaves,
        research_bound_assignments, tree_admission,
    };
    use std::collections::BTreeSet;
    use std::num::NonZeroU32;
    use tapscript::upstream::LiveTransferRepresentationPlan;
    use tapscript::{
        LiveTransferShapeBounds, MINIMUM_TRANSFER_RECEIPT_INPUTS, MINIMUM_TRANSFER_RECEIPT_OUTPUTS,
        demonstration_live_shape_set, dense_live_shape_set,
    };

    /// The depth the demonstration deployment declares.
    ///
    /// Eight, from `crate::live_plan::live_deployment_for_asset`. Stated
    /// here as the fixture's own figure rather than imported, because a
    /// study that read the deployment's parameter would report whatever
    /// the deployment happened to say instead of checking it.
    fn declared_depth() -> NonZeroU32 {
        NonZeroU32::new(8).expect("eight is nonzero")
    }

    /// One assignment, from three counts.
    fn bounds(inputs: u8, outputs: u8, sponsors: u8) -> LiveTransferShapeBounds {
        LiveTransferShapeBounds::new(
            std::num::NonZeroU8::new(inputs).expect("the fixture input bound is nonzero"),
            std::num::NonZeroU8::new(outputs).expect("the fixture output bound is nonzero"),
            sponsors,
        )
    }

    #[test]
    fn the_enumeration_is_the_finite_deterministic_product_section_eighteen_states() {
        // §18.1 states three research lists. The count is recomputed from
        // the lists rather than written as 294, so editing a list moves
        // the expectation with it instead of leaving a stale constant.
        let assignments = research_bound_assignments();

        assert_eq!(
            assignments.len(),
            RESEARCH_RECEIPT_INPUT_BOUNDS.len()
                * RESEARCH_RECEIPT_OUTPUT_BOUNDS.len()
                * RESEARCH_SPONSOR_INPUT_BOUNDS.len(),
        );
        assert_eq!(assignments, research_bound_assignments());

        let distinct: BTreeSet<_> = assignments.iter().copied().collect();
        assert_eq!(distinct.len(), assignments.len());

        for assignment in &assignments {
            assert!(RESEARCH_RECEIPT_INPUT_BOUNDS.contains(&assignment.receipt_inputs()));
            assert!(RESEARCH_RECEIPT_OUTPUT_BOUNDS.contains(&assignment.receipt_outputs()));
            assert!(RESEARCH_SPONSOR_INPUT_BOUNDS.contains(&assignment.sponsor_inputs()));
        }
    }

    #[test]
    fn every_candidate_shape_set_validates_every_member_within_its_bounds() {
        // §18.1's own sentence, checked at every enumerated assignment
        // rather than at a chosen one. The dense unrolling is the set the
        // enumeration is about, and `LiveTransferShapeSet::new` is what
        // refuses a member outside the window — so a set that exists is
        // one whose members were checked, and this asserts the property
        // again over the members themselves.
        for assignment in research_bound_assignments() {
            let set = dense_live_shape_set(assignment);
            assert_eq!(set.bounds(), assignment);

            for shape in set.shapes() {
                assert!(shape.receipt_inputs() >= MINIMUM_TRANSFER_RECEIPT_INPUTS);
                assert!(shape.receipt_inputs() <= assignment.receipt_inputs());
                assert!(shape.receipt_outputs() >= MINIMUM_TRANSFER_RECEIPT_OUTPUTS);
                assert!(shape.receipt_outputs() <= assignment.receipt_outputs());
                assert!(shape.sponsor_inputs() <= assignment.sponsor_inputs());
            }
        }
    }

    #[test]
    fn the_closed_forms_agree_with_the_emission_they_summarize() {
        // The check this module's header promises. The counts are closed
        // forms because the widest assignment cannot be built; they are
        // checked against the machinery that would build one at every
        // assignment small enough to afford it, and a disagreement is a
        // defect here rather than a tolerance.
        for assignment in research_bound_assignments() {
            if committed_leaf_count(assignment) > 512 {
                continue;
            }
            let set = dense_live_shape_set(assignment);
            assert_eq!(
                admitted_shape_count(assignment),
                set.len() as u64,
                "the shape count disagrees with the unrolling at {assignment:?}",
            );

            for representation in [
                LiveTransferRepresentationPlan::Explicit,
                LiveTransferRepresentationPlan::PrivateCommitted,
            ] {
                assert_eq!(
                    committed_leaf_count(assignment),
                    committed_leaves(representation, assignment).len() as u64,
                    "the leaf count disagrees with the leaf set at {assignment:?}",
                );
            }
        }
    }

    #[test]
    fn no_enumerated_assignment_overflows_the_target_position_domain() {
        // §18.1's overflow obligation, answered at every assignment. The
        // widest reaches eighty inputs and sixty-six outputs, and both
        // are asserted rather than the maximum alone, because §18.1
        // declines to assume monotonicity and an argument from the widest
        // assumes it.
        let mut widest_inputs = 0;
        let mut widest_outputs = 0;

        for assignment in research_bound_assignments() {
            let domain = CandidatePositionDomain::of(assignment);
            assert!(
                domain.fits_target_position_domain(),
                "{assignment:?} reaches a position the target's own type cannot hold",
            );
            widest_inputs = widest_inputs.max(domain.inputs());
            widest_outputs = widest_outputs.max(domain.outputs());
        }

        assert_eq!(widest_inputs, 80);
        assert_eq!(widest_outputs, 66);
    }

    #[test]
    fn the_declared_tree_depth_is_what_bounds_a_live_candidate_today() {
        // The live study's headline constraint, and it is not the one
        // compact ASH found. The live tree's policy admits the
        // equal-weight closed form past the subset oracle's budget, so
        // sixteen leaves stop nothing; what stops a wider candidate is
        // the deployment's own declared depth of eight, which carries two
        // hundred and fifty-six leaves, and past a thousand and
        // twenty-four the linker's leaf budget takes over.
        let demonstration = bounds(3, 3, 1);
        assert_eq!(admitted_shape_count(demonstration), 27);
        assert_eq!(committed_leaf_count(demonstration), 29);
        assert!(matches!(
            tree_admission(demonstration, declared_depth()),
            CandidateTreeAdmission::TreeIsBuildable {
                leaves: 29,
                least_depth: 5,
            },
        ));

        // The declared depth carries two hundred and fifty-six leaves,
        // and the boundary is asserted from both sides rather than from
        // the side that says yes: two hundred and fifty-five leaves build
        // and two hundred and fifty-eight do not.
        assert_eq!(committed_leaf_count(bounds(1, 85, 1)), 255);
        assert!(matches!(
            tree_admission(bounds(1, 85, 1), declared_depth()),
            CandidateTreeAdmission::TreeIsBuildable {
                leaves: 255,
                least_depth: 8,
            },
        ));
        assert_eq!(committed_leaf_count(bounds(1, 86, 1)), 258);
        assert!(matches!(
            tree_admission(bounds(1, 86, 1), declared_depth()),
            CandidateTreeAdmission::AboveDeclaredTreeDepth {
                capacity: 256,
                declared: 8,
                ..
            },
        ));
        assert!(matches!(
            tree_admission(bounds(64, 64, 16), declared_depth()),
            CandidateTreeAdmission::AboveTreeLeafBudget { budget: 1024, .. },
        ));
    }

    #[test]
    fn most_of_the_enumeration_commits_a_tree_no_deployment_can_build() {
        // The non-monotone fact the enumeration exposes. Widening either
        // receipt axis multiplies the leaf set and widening the sponsor
        // axis multiplies it again, so the assignments a deployment of
        // declared depth eight can commit are a small corner of the
        // matrix rather than a prefix of it.
        let costs = bound_assignment_costs(declared_depth());
        let buildable: Vec<_> = costs
            .iter()
            .filter(|cost| cost.admission().is_buildable())
            .map(|cost| {
                (
                    cost.bounds().receipt_inputs(),
                    cost.bounds().receipt_outputs(),
                    cost.bounds().sponsor_inputs(),
                )
            })
            .collect();

        assert!(buildable.contains(&(3, 3, 1)) || buildable.contains(&(2, 2, 1)));
        assert!(!buildable.contains(&(64, 64, 16)));
        assert!(buildable.len() < costs.len());
        assert_ne!(buildable.len(), 0);
    }

    #[test]
    fn the_tested_candidate_realizes_exactly_the_narrow_corner_of_the_enumeration() {
        // §18.5's fitting relation, computed rather than asserted. The
        // tested candidate publishes three receipts in, three out, and
        // one sponsor — a window §18.1 does not enumerate — so the honest
        // statement is which *enumerated* assignments it has a program
        // for, and the answer is the eight whose three counts are all at
        // or below the tested set's.
        let realized = assignments_realized_by(&demonstration_live_shape_set());
        let triples: Vec<_> = realized
            .iter()
            .map(|bounds| {
                (
                    bounds.receipt_inputs(),
                    bounds.receipt_outputs(),
                    bounds.sponsor_inputs(),
                )
            })
            .collect();

        assert_eq!(
            triples,
            vec![
                (1, 1, 0),
                (1, 1, 1),
                (1, 2, 0),
                (1, 2, 1),
                (2, 1, 0),
                (2, 1, 1),
                (2, 2, 0),
                (2, 2, 1),
            ],
        );
    }
}
