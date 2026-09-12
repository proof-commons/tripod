use std::collections::BTreeSet;

use proptest::prelude::*;

use crate::{
    AnnouncementLeadBounds, Cycle, Maturity, MaturityTransitionRefusal, ProtocolAmount,
    RealizationError, StateMetadata, announce_maturity,
};

fn amount(value: u64) -> ProtocolAmount {
    ProtocolAmount::new(value).unwrap()
}

fn announced(cycle: u64) -> Maturity {
    Maturity::Announced {
        cycle: Cycle::new(cycle),
    }
}

fn bounds(minimum: u64, maximum: u64) -> AnnouncementLeadBounds {
    let minimum = Cycle::new(minimum);
    let maximum = Cycle::new(maximum);

    AnnouncementLeadBounds::new(minimum, maximum).unwrap()
}

// The four quantities are pairwise distinct, so a copy-through that
// swapped two of them would fail a per-field assertion rather than
// pass by coincidence.
fn unannounced(cycle: u64) -> StateMetadata {
    StateMetadata {
        omega: amount(1_000),
        y_l: amount(700),
        y_t: amount(200),
        q: amount(50),
        cycle: Cycle::new(cycle),
        maturity: Maturity::Unannounced,
    }
}

#[test]
fn cycle_add_is_checked() {
    let sum = Cycle::new(2).checked_add(Cycle::new(3)).unwrap();
    let unchanged = Cycle::MAX.checked_add(Cycle::ZERO).unwrap();
    let overflowed = Cycle::MAX.checked_add(Cycle::new(1));

    assert_eq!(sum, Cycle::new(5));
    assert_eq!(unchanged, Cycle::MAX);
    assert_eq!(overflowed, Err(RealizationError::CycleOverflow));
}

#[test]
fn lead_bounds_reject_zero_minimum() {
    let refused = AnnouncementLeadBounds::new(Cycle::ZERO, Cycle::new(10));
    let expected = RealizationError::InvalidAnnouncementLeadBounds {
        minimum: 0,
        maximum: 10,
    };

    assert_eq!(refused, Err(expected));
}

#[test]
fn lead_bounds_reject_inverted_pair() {
    let refused = AnnouncementLeadBounds::new(Cycle::new(10), Cycle::new(9));
    let expected = RealizationError::InvalidAnnouncementLeadBounds {
        minimum: 10,
        maximum: 9,
    };

    assert_eq!(refused, Err(expected));

    let degenerate = bounds(10, 10);

    assert_eq!(degenerate.minimum(), Cycle::new(10));
    assert_eq!(degenerate.maximum(), Cycle::new(10));
}

#[test]
fn lead_bounds_window_is_inclusive() {
    let lead = bounds(3, 7);
    let (earliest, latest) = lead.window(Cycle::new(100)).unwrap();

    assert_eq!(lead.minimum(), Cycle::new(3));
    assert_eq!(lead.maximum(), Cycle::new(7));
    assert_eq!(earliest, Cycle::new(103));
    assert_eq!(latest, Cycle::new(107));
}

#[test]
fn lead_bounds_window_overflow_is_cycle_overflow() {
    let expected = MaturityTransitionRefusal::CycleArithmeticOverflow;
    let penultimate = Cycle::new(u64::MAX - 1);

    assert_eq!(bounds(1, 2).window(Cycle::MAX), Err(expected));

    // The lower endpoint still fits here: only the upper one leaves
    // the domain, and the window refuses rather than wrapping.
    assert_eq!(bounds(1, 2).window(penultimate), Err(expected));
}

#[test]
fn minimum_lead_accepts() {
    let predecessor = unannounced(100);
    let successor = announce_maturity(&predecessor, Cycle::new(103), bounds(3, 7)).unwrap();

    assert_eq!(successor.maturity, announced(103));
}

#[test]
fn maximum_lead_accepts() {
    let predecessor = unannounced(100);
    let successor = announce_maturity(&predecessor, Cycle::new(107), bounds(3, 7)).unwrap();

    assert_eq!(successor.maturity, announced(107));
}

#[test]
fn one_below_minimum_rejects() {
    let predecessor = unannounced(100);
    let refused = announce_maturity(&predecessor, Cycle::new(102), bounds(3, 7));
    let expected = MaturityTransitionRefusal::AnnouncementBelowMinimum;

    assert_eq!(refused, Err(expected));
}

#[test]
fn one_above_maximum_rejects() {
    let predecessor = unannounced(100);
    let refused = announce_maturity(&predecessor, Cycle::new(108), bounds(3, 7));
    let expected = MaturityTransitionRefusal::AnnouncementAboveMaximum;

    assert_eq!(refused, Err(expected));
}

#[test]
fn already_announced_rejects() {
    let predecessor = unannounced(100).with_maturity(announced(105));
    let refused = announce_maturity(&predecessor, Cycle::new(104), bounds(3, 7));
    let expected = MaturityTransitionRefusal::PredecessorAlreadyAnnounced;

    assert_eq!(refused, Err(expected));
}

#[test]
fn complete_rejects() {
    let predecessor = unannounced(100).with_maturity(Maturity::Complete);
    let refused = announce_maturity(&predecessor, Cycle::new(104), bounds(3, 7));
    let expected = MaturityTransitionRefusal::PredecessorMaturityComplete;

    assert_eq!(refused, Err(expected));
}

#[test]
fn successor_changes_only_maturity() {
    let predecessor = unannounced(100);
    let successor = announce_maturity(&predecessor, Cycle::new(104), bounds(3, 7)).unwrap();

    assert_eq!(successor.omega, predecessor.omega);
    assert_eq!(successor.y_l, predecessor.y_l);
    assert_eq!(successor.y_t, predecessor.y_t);
    assert_eq!(successor.q, predecessor.q);
    assert_eq!(successor.cycle, predecessor.cycle);
    assert_eq!(successor.maturity, announced(104));
    assert_ne!(successor.maturity, predecessor.maturity);
}

#[test]
fn with_maturity_changes_only_maturity() {
    let value = unannounced(100);
    let updated = value.with_maturity(Maturity::Complete);

    assert_eq!(updated.omega, value.omega);
    assert_eq!(updated.y_l, value.y_l);
    assert_eq!(updated.y_t, value.y_t);
    assert_eq!(updated.q, value.q);
    assert_eq!(updated.cycle, value.cycle);
    assert_eq!(updated.maturity, Maturity::Complete);
    assert_eq!(value.maturity, Maturity::Unannounced);
}

// The destructuring is the census: a seventh field would fail to
// compile here rather than travel unasserted through the transition.
#[test]
fn metadata_has_exactly_six_fields() {
    let value = unannounced(100);

    let StateMetadata {
        omega,
        y_l,
        y_t,
        q,
        cycle,
        maturity,
    } = value;

    assert_eq!(omega, amount(1_000));
    assert_eq!(y_l, amount(700));
    assert_eq!(y_t, amount(200));
    assert_eq!(q, amount(50));
    assert_eq!(cycle, Cycle::new(100));
    assert_eq!(maturity, Maturity::Unannounced);
}

#[test]
fn every_refusal_variant_is_reached() {
    let lead = bounds(3, 7);
    let predecessor = unannounced(100);
    let already = predecessor.with_maturity(announced(105));
    let complete = predecessor.with_maturity(Maturity::Complete);
    let saturated = unannounced(u64::MAX);

    let observed = [
        announce_maturity(&already, Cycle::new(104), lead),
        announce_maturity(&complete, Cycle::new(104), lead),
        announce_maturity(&predecessor, Cycle::new(102), lead),
        announce_maturity(&predecessor, Cycle::new(108), lead),
        announce_maturity(&saturated, Cycle::ZERO, lead),
    ]
    .map(|outcome| outcome.unwrap_err());

    assert_eq!(MaturityTransitionRefusal::ALL.len(), 5);
    assert_eq!(observed.as_slice(), MaturityTransitionRefusal::ALL);

    for refusal in MaturityTransitionRefusal::ALL {
        assert!(observed.contains(refusal), "unreached refusal: {refusal:?}");
    }
}

#[test]
fn refusal_names_are_distinct() {
    let mut names = Vec::new();
    let mut unique = BTreeSet::new();

    for refusal in MaturityTransitionRefusal::ALL {
        names.push(refusal.name());
        unique.insert(refusal.name());
    }

    assert_eq!(names.len(), 5);
    assert_eq!(unique.len(), names.len());

    let overflow = MaturityTransitionRefusal::CycleArithmeticOverflow;

    assert_eq!(overflow.name(), "cycle-arithmetic-overflow");

    for name in &names {
        let kebab = name.chars().all(|c| c.is_ascii_lowercase() || c == '-');

        assert!(kebab, "name is not kebab-case: {name}");
    }
}

// Bounds and an announced cycle drawn together, so the cycle always
// lies inside the window the bounds describe.
fn valid_announcement() -> impl Strategy<Value = (u64, u64, u64)> {
    let leads = (1_u64..=1 << 32, 1_u64..=1 << 32);

    leads.prop_flat_map(|(first, second)| {
        let minimum = first.min(second);
        let maximum = first.max(second);

        (Just(minimum), Just(maximum), minimum..=maximum)
    })
}

proptest! {
    #[test]
    fn valid_announcements_change_only_maturity(
        omega in 0_u64..1 << 40,
        y_l in 0_u64..1 << 40,
        y_t in 0_u64..1 << 40,
        q in 0_u64..1 << 40,
        cycle in 0_u64..1 << 40,
        (minimum, maximum, lead) in valid_announcement(),
    ) {
        let predecessor = StateMetadata {
            omega: amount(omega),
            y_l: amount(y_l),
            y_t: amount(y_t),
            q: amount(q),
            cycle: Cycle::new(cycle),
            maturity: Maturity::Unannounced,
        };

        let lead_bounds = bounds(minimum, maximum);
        let announced_cycle = Cycle::new(cycle + lead);
        let successor = announce_maturity(&predecessor, announced_cycle, lead_bounds).unwrap();

        prop_assert_eq!(successor.omega, predecessor.omega);
        prop_assert_eq!(successor.y_l, predecessor.y_l);
        prop_assert_eq!(successor.y_t, predecessor.y_t);
        prop_assert_eq!(successor.q, predecessor.q);
        prop_assert_eq!(successor.cycle, predecessor.cycle);
        prop_assert_eq!(successor.maturity, announced(cycle + lead));
        prop_assert_ne!(successor, predecessor);
    }
}
