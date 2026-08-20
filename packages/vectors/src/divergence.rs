//! Where the protocol's amount domain and the target's bound disagree.
//!
//! Guide-12 §17.2 keeps target facts out of a semantic fixture, and
//! §18.1 requires a positive class whose values sum to \(2^{51}-1\).
//! Both hold, and together they produce a row the reviewed Elements
//! target cannot state at all: the protocol's amount domain reaches to
//! \(2^{51}-1\) and the target's stated-amount bound stops at
//! `21000000 * COIN`, which is smaller. A world can therefore be
//! model-valid and unencodable on this target at once.
//!
//! # Why this is derived rather than flagged
//!
//! The alternative was a per-row marker saying "this one is expected to
//! fail", which is a hand-written expectation and would have to be
//! rewritten for every target. What this module does instead is compare
//! the amounts a row requires the target to state against the bound the
//! target's own reviewed facts publish. A different target publishes a
//! different bound and the same rows reclassify themselves; a row
//! nobody thought about is classified anyway.
//!
//! # What this module is not
//!
//! It states no expectation about a layer, a diagnostic, or a message.
//! A bound is a number and this compares numbers with it. Which of the
//! target's gates answers first — and whether an adapter's own reserve
//! arithmetic answers ahead of all of them — is an observation, made
//! elsewhere, and is deliberately not predicted here.

use target_elements::{StatedAmountBound, reviewed_stated_amount_bound};

use crate::fixture::CompactAshSemanticCase;

/// Which amount of a row the target would have to state.
///
/// The two places are reached at different times, which is why they are
/// distinguished: an input amount is stated by the funding ceremony,
/// before any vector exists, and the successor amount is stated by the
/// vector itself. A row can be beyond the bound at either.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum StatedAmountPlace {
    /// The value of the coin one ASH input spends, by member position.
    AshInput(usize),
    /// The successor amount, which one output carries whole.
    Successor,
}

/// One amount a row requires and the target's bound forbids.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AmountBeyondTargetBound {
    place: StatedAmountPlace,
    stated: u64,
    bound: u64,
}

impl AmountBeyondTargetBound {
    /// Where in the row the amount is stated.
    #[must_use]
    pub const fn place(self) -> StatedAmountPlace {
        self.place
    }

    /// The amount the row requires.
    #[must_use]
    pub const fn stated(self) -> u64 {
        self.stated
    }

    /// The bound that forbids it.
    #[must_use]
    pub const fn bound(self) -> u64 {
        self.bound
    }

    /// By how much the amount overshoots the bound.
    ///
    /// Reported rather than left to the reader, because "beyond the
    /// bound" covers both one unit over and three orders of magnitude
    /// over, and a record that could not tell them apart would make a
    /// near miss and a category error look alike.
    #[must_use]
    pub const fn excess(self) -> u64 {
        self.stated.saturating_sub(self.bound)
    }
}

/// Whether the target's bound admits every amount a row requires.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum TargetAmountStanding {
    /// Every amount this row states is one the target admits.
    Admitted,
    /// The bound forbids one of them, the first in ceremony order.
    BeyondBound(AmountBeyondTargetBound),
}

impl TargetAmountStanding {
    /// The offending amount, where there is one.
    #[must_use]
    pub const fn beyond_bound(self) -> Option<AmountBeyondTargetBound> {
        match self {
            Self::Admitted => None,
            Self::BeyondBound(beyond) => Some(beyond),
        }
    }

    /// The ASH input no ceremony on this target can create a coin for.
    ///
    /// Present only for a row whose *input* amount is beyond the bound.
    /// A row that is beyond it only at the successor is fundable,
    /// builds, and is refused when it is submitted — which is a
    /// different fact established by a different step, so the two are
    /// not merged.
    #[must_use]
    pub const fn unfundable_input(self) -> Option<(usize, AmountBeyondTargetBound)> {
        match self {
            Self::Admitted => None,
            Self::BeyondBound(beyond) => match beyond.place {
                StatedAmountPlace::AshInput(member) => Some((member, beyond)),
                StatedAmountPlace::Successor => None,
            },
        }
    }

    /// Whether no ceremony on this target can create the row's coins.
    #[must_use]
    pub const fn is_unfundable(self) -> bool {
        self.unfundable_input().is_some()
    }
}

/// Classify one semantic case against the reviewed target bound.
#[must_use]
pub fn target_amount_standing(case: &CompactAshSemanticCase) -> TargetAmountStanding {
    standing_against(case, reviewed_stated_amount_bound())
}

/// Classify one semantic case against a stated bound.
///
/// The bound is a parameter so that a test can vary it: a rule checked
/// only at the one bound the target happens to publish today would be
/// indistinguishable from a rule that hard-codes that bound's answers.
///
/// The order of the comparisons is the ceremony's own order — every ASH
/// input in member order, then the successor — because the first
/// offending amount is the one the run actually reaches first, and a
/// record naming a later one would not describe what happened.
#[must_use]
pub fn standing_against(
    case: &CompactAshSemanticCase,
    bound: StatedAmountBound,
) -> TargetAmountStanding {
    for (member, amount) in case.inputs().iter().enumerate() {
        if !bound.admits(amount.get()) {
            return TargetAmountStanding::BeyondBound(AmountBeyondTargetBound {
                place: StatedAmountPlace::AshInput(member),
                stated: amount.get(),
                bound: bound.maximum(),
            });
        }
    }

    // The successor is the realization layer's own derived amount and
    // is read off the fixture's expectation rather than re-summed here.
    // Summing again would be a second authored arithmetic for one
    // number, and the two could drift apart.
    let successor = case.expected().successor().1.get();
    if bound.admits(successor) {
        TargetAmountStanding::Admitted
    } else {
        TargetAmountStanding::BeyondBound(AmountBeyondTargetBound {
            place: StatedAmountPlace::Successor,
            stated: successor,
            bound: bound.maximum(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        StatedAmountPlace, TargetAmountStanding, standing_against, target_amount_standing,
    };
    use crate::fixture::{SemanticFixtureId, SponsorCase, positive_semantic_census, semantic_case};
    use crate::matrix::POSITIVE_SEMANTIC;
    use target_elements::{StatedAmountBound, reviewed_stated_amount_bound};

    #[test]
    fn exactly_one_positive_row_is_beyond_the_targets_bound() {
        // The census is a fixed finite set, so the figure is pinned
        // rather than asserted to be positive: a second divergent row
        // appearing is a fact somebody has to look at, not a number to
        // absorb.
        let census = positive_semantic_census().expect("the positive census builds");
        let divergent: Vec<_> = census
            .iter()
            .filter(|case| target_amount_standing(case) != TargetAmountStanding::Admitted)
            .collect();
        assert_eq!(divergent.len(), 1);
        assert_eq!(
            divergent[0].class().name(),
            "values-summing-to-two-pow-51-minus-one"
        );
    }

    #[test]
    fn the_divergent_row_is_named_at_the_input_that_overshoots() {
        let census = positive_semantic_census().expect("the positive census builds");
        let case = census
            .iter()
            .find(|case| case.class().name() == "values-summing-to-two-pow-51-minus-one")
            .expect("the boundary row is present");
        let beyond = target_amount_standing(case)
            .beyond_bound()
            .expect("the row is beyond the bound");

        // The first ASH input, not the successor: the funding ceremony
        // reaches it before anything is built, and that is the step the
        // run actually stops at.
        assert_eq!(beyond.place(), StatedAmountPlace::AshInput(0));
        assert_eq!(beyond.stated(), (1 << 51) - 2);
        assert_eq!(beyond.bound(), reviewed_stated_amount_bound().maximum());
        assert_eq!(beyond.excess(), ((1 << 51) - 2) - beyond.bound());
        assert!(target_amount_standing(case).is_unfundable());
    }

    #[test]
    fn a_row_beyond_the_bound_only_at_its_successor_is_still_fundable() {
        // Two amounts the target admits individually whose sum it does
        // not, by exactly one unit. Nothing in the census reaches this
        // today, and the arm exists because the two comparisons are two
        // facts: a rule that only checked the inputs would call this
        // row fine and then be surprised at submission.
        let bound = reviewed_stated_amount_bound().maximum();
        let id = SemanticFixtureId::new("successor-only", 0);
        let case = semantic_case(id, POSITIVE_SEMANTIC[0], &[bound, 1], SponsorCase::Absent)
            .expect("the world is inside the protocol domain");

        let beyond = target_amount_standing(&case)
            .beyond_bound()
            .expect("the successor is beyond the bound");
        assert_eq!(beyond.place(), StatedAmountPlace::Successor);
        assert_eq!(beyond.stated(), bound + 1);
        assert_eq!(beyond.excess(), 1);
        assert!(
            !target_amount_standing(&case).is_unfundable(),
            "every coin this row spends is one the target can create"
        );
    }

    #[test]
    fn every_other_positive_row_is_admitted_whole() {
        let census = positive_semantic_census().expect("the positive census builds");
        let bound = reviewed_stated_amount_bound();
        for case in &census {
            if case.class().name() == "values-summing-to-two-pow-51-minus-one" {
                continue;
            }
            assert_eq!(
                target_amount_standing(case),
                TargetAmountStanding::Admitted,
                "{:?} was classified divergent",
                case.id()
            );
            // Recomputed the long way round, so the classification is
            // compared against something rather than restated.
            for amount in case.inputs() {
                assert!(bound.admits(amount.get()));
            }
            assert!(bound.admits(case.expected().successor().1.get()));
        }
    }

    #[test]
    fn the_classification_follows_the_bound_it_is_given() {
        // The property that makes this a derivation: shrink the bound
        // and rows reclassify themselves. A rule that hard-coded the
        // one divergent row would pass every test above and fail this.
        let census = positive_semantic_census().expect("the positive census builds");
        let ordinary = census
            .iter()
            .find(|case| case.class().name() == "minimum-two-ash-inputs")
            .expect("the first row is present");
        assert_eq!(
            target_amount_standing(ordinary),
            TargetAmountStanding::Admitted
        );

        // A target admitting less than this row's first input. The row
        // did not change; the bound did.
        let first = ordinary.inputs()[0].get();
        let narrow = StatedAmountBound::hypothetical(first - 1);
        let beyond = standing_against(ordinary, narrow)
            .beyond_bound()
            .expect("the narrowed bound forbids the first input");
        assert_eq!(beyond.place(), StatedAmountPlace::AshInput(0));
        assert_eq!(beyond.stated(), first);
        assert_eq!(beyond.bound(), first - 1);
        assert_eq!(beyond.excess(), 1);
    }
}
