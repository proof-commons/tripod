//! Total counter discipline for the exact searches (Guide-8 D.8).
//!
//! Every exact search in this crate promises the same contract: work
//! exhaustion returns a typed error and no partial result. An unchecked
//! `+= 1` cannot keep that promise at the top of the counter's range —
//! a debug build panics and a release build wraps, so a search that
//! reached its limit could report a restarted count instead of the
//! typed failure. A configured maximum is a [`NonZeroU64`], so
//! `u64::MAX` is a legal limit and the boundary is reachable in the
//! type rather than only in principle.
//!
//! Two counter kinds exist, and they are deliberately different:
//!
//! - a *budgeted* counter decides whether the search may continue, so
//!   it is checked against its limit before it moves and reports
//!   exhaustion as a typed error;
//! - a *diagnostic* counter only describes how the search walked, so it
//!   saturates rather than failing an otherwise complete analysis over
//!   a statistic that carries no semantic identity.

use std::num::NonZeroU64;

/// Admit one more visited state against an explicit state budget.
///
/// The budget is inclusive of every visited state, the root state
/// included: a search configured for `n` states may visit exactly `n`,
/// and the attempt to visit the `n + 1`-th fails. Checking before the
/// increment is what makes the rule total — the counter can never pass
/// its maximum, so it can never reach the point where the increment
/// itself would panic or wrap.
///
/// # Errors
///
/// The configured maximum, when the budget is already spent. The caller
/// maps it to its own typed exhaustion error, because the proof search
/// and the placement search exhaust for different reasons and say so
/// separately.
pub const fn admit_search_state(visited: &mut u64, maximum: NonZeroU64) -> Result<(), u64> {
    if *visited >= maximum.get() {
        return Err(maximum.get());
    }

    *visited += 1;
    Ok(())
}

/// Record one more diagnostic-only search event.
///
/// Saturating rather than checked: this counter never decides whether a
/// result is accepted, and it is excluded from every stable projection,
/// so a saturated statistic is a less harmful outcome than failing a
/// complete and correct analysis. It still may not wrap — a wrapped
/// count reads as a small honest number and would misdescribe the
/// search rather than visibly pinning at the top of its range.
pub const fn record_search_event(counter: &mut u64) {
    *counter = counter.saturating_add(1);
}

#[cfg(test)]
mod tests {
    use super::{admit_search_state, record_search_event};

    fn limit(value: u64) -> std::num::NonZeroU64 {
        std::num::NonZeroU64::new(value).expect("test limit is nonzero")
    }

    #[test]
    fn a_budget_of_one_admits_exactly_one_state() {
        let mut visited = 0;

        assert_eq!(admit_search_state(&mut visited, limit(1)), Ok(()));
        assert_eq!(visited, 1);
        assert_eq!(admit_search_state(&mut visited, limit(1)), Err(1));
        assert_eq!(visited, 1, "a refused admission leaves the counter alone");
    }

    #[test]
    fn a_budget_admits_exactly_its_maximum_states() {
        let mut visited = 0;

        for expected in 1..=4 {
            assert_eq!(admit_search_state(&mut visited, limit(4)), Ok(()));
            assert_eq!(visited, expected);
        }

        assert_eq!(admit_search_state(&mut visited, limit(4)), Err(4));
    }

    #[test]
    fn a_budgeted_counter_never_overflows_at_the_top_of_its_range() {
        let mut visited = u64::MAX - 1;

        assert_eq!(admit_search_state(&mut visited, limit(u64::MAX)), Ok(()));
        assert_eq!(visited, u64::MAX);
        assert_eq!(
            admit_search_state(&mut visited, limit(u64::MAX)),
            Err(u64::MAX),
            "the maximal budget still refuses rather than wrapping",
        );
        assert_eq!(visited, u64::MAX);
    }

    #[test]
    fn a_diagnostic_counter_saturates_instead_of_wrapping() {
        let mut counter = u64::MAX - 1;

        record_search_event(&mut counter);
        assert_eq!(counter, u64::MAX);

        record_search_event(&mut counter);
        assert_eq!(counter, u64::MAX, "a saturated statistic never restarts");
    }
}
