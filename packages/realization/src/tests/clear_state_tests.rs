//! Clear's authenticated-family law; no world recognition or declaration claim.

use std::collections::BTreeSet;

use proptest::prelude::*;

use crate::{
    ClearTransitionRefusal as Refusal, Cycle, Maturity, PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE,
    ProtocolAmount, StateField, StateMetadata, clear_ash,
};

fn amount(value: u64) -> ProtocolAmount {
    ProtocolAmount::new(value).unwrap()
}

fn state(y_l: u64, y_t: u64) -> StateMetadata {
    StateMetadata {
        omega: amount(137),
        y_l: amount(y_l),
        y_t: amount(y_t),
        q: amount(23),
        cycle: Cycle::new(11),
        maturity: Maturity::Announced {
            cycle: Cycle::new(17),
        },
    }
}

fn refuses(y_l: u64, y_t: u64, ash: &[(u64, u64)], bound: usize, expected: Refusal) {
    let ash: Vec<_> = ash.iter().map(|&(id, value)| (id, amount(value))).collect();
    assert_eq!(clear_ash(&state(y_l, y_t), &ash, bound), Err(expected));
}

#[test]
fn empty_batch() {
    refuses(10, 3, &[], 2, Refusal::EmptyBatch);
}

#[test]
fn batch_too_large() {
    refuses(10, 3, &[(1, 2), (2, 3)], 1, Refusal::BatchTooLarge);
}

#[test]
fn duplicate_input() {
    refuses(10, 3, &[(1, 2), (1, 2)], 2, Refusal::DuplicateInput);
}

#[test]
fn supply_out_of_domain() {
    refuses(
        PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 1,
        1,
        &[(1, 2)],
        2,
        Refusal::SupplyOutOfDomain,
    );
}

#[test]
fn sealed() {
    refuses(0, 0, &[(1, 2)], 2, Refusal::Sealed);
}

#[test]
fn zero_ash_amount() {
    refuses(10, 3, &[(1, 0)], 2, Refusal::ZeroAshAmount);
}

#[test]
fn ash_total_out_of_domain() {
    refuses(
        10,
        3,
        &[(1, PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 1), (2, 1)],
        2,
        Refusal::AshTotalOutOfDomain,
    );
}

#[test]
fn zero_progress() {
    refuses(0, 3, &[(1, 2)], 2, Refusal::ZeroProgress);
    refuses(1, 0, &[(1, 2)], 2, Refusal::ZeroProgress);
}

fn accepts(y_l: u64, y_t: u64, ash: &[(u64, u64)], bound: usize) {
    let ash: Vec<_> = ash.iter().map(|&(id, value)| (id, amount(value))).collect();
    assert!(clear_ash(&state(y_l, y_t), &ash, bound).is_ok());
}

#[test]
fn each_refusal_has_an_accepted_control_one_coordinate_away() {
    let maximum = PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 1;
    refuses(10, 3, &[], 2, Refusal::EmptyBatch);
    accepts(10, 3, &[(1, 2)], 2);
    refuses(10, 3, &[(1, 2), (2, 3)], 1, Refusal::BatchTooLarge);
    accepts(10, 3, &[(1, 2), (2, 3)], 2);
    refuses(10, 3, &[(1, 2), (1, 2)], 2, Refusal::DuplicateInput);
    accepts(10, 3, &[(1, 2), (2, 2)], 2);
    refuses(maximum, 1, &[(1, 2)], 2, Refusal::SupplyOutOfDomain);
    accepts(maximum, 0, &[(1, 2)], 2);
    refuses(0, 0, &[(1, 2)], 2, Refusal::Sealed);
    accepts(2, 0, &[(1, 2)], 2);
    refuses(10, 3, &[(1, 0)], 2, Refusal::ZeroAshAmount);
    accepts(10, 3, &[(1, 1)], 2);
    refuses(
        10,
        3,
        &[(1, maximum), (2, 1)],
        2,
        Refusal::AshTotalOutOfDomain,
    );
    accepts(10, 3, &[(1, maximum - 1), (2, 1)], 2);
    refuses(0, 3, &[(1, 2)], 2, Refusal::ZeroProgress);
    accepts(1, 3, &[(1, 2)], 2);
    refuses(1, 0, &[(1, 2)], 2, Refusal::ZeroProgress);
    accepts(2, 0, &[(1, 2)], 2);
}

#[test]
fn recognition_aggregation_and_progress_follow_model_order() {
    let maximum = PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 1;
    refuses(10, 3, &[(1, 2), (1, 2)], 1, Refusal::BatchTooLarge);
    refuses(maximum, 1, &[(1, 2), (1, 2)], 2, Refusal::DuplicateInput);
    refuses(maximum, 1, &[(1, 0)], 2, Refusal::SupplyOutOfDomain);
    refuses(0, 0, &[(1, 0)], 2, Refusal::Sealed);
    refuses(
        10,
        3,
        &[(1, maximum), (2, 1), (3, 0)],
        3,
        Refusal::AshTotalOutOfDomain,
    );
    refuses(
        10,
        3,
        &[(1, 0), (2, maximum), (3, 1)],
        3,
        Refusal::ZeroAshAmount,
    );
    refuses(0, 3, &[(1, 0)], 2, Refusal::ZeroAshAmount);
    refuses(
        0,
        3,
        &[(1, maximum), (2, 1)],
        2,
        Refusal::AshTotalOutOfDomain,
    );
}

#[test]
fn every_refusal_is_reached_and_named() {
    let maximum = PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 1;
    let cases = [
        (10, 3, vec![], 2),
        (10, 3, vec![(1, 2), (2, 3)], 1),
        (10, 3, vec![(1, 2), (1, 2)], 2),
        (maximum, 1, vec![(1, 2)], 2),
        (0, 0, vec![(1, 2)], 2),
        (10, 3, vec![(1, 0)], 2),
        (10, 3, vec![(1, maximum), (2, 1)], 2),
        (1, 0, vec![(1, 2)], 2),
    ];
    let reached: Vec<_> = cases
        .into_iter()
        .map(|(y_l, y_t, ash, bound)| {
            let ash: Vec<_> = ash
                .into_iter()
                .map(|(id, value)| (id, amount(value)))
                .collect();
            clear_ash(&state(y_l, y_t), &ash, bound).unwrap_err()
        })
        .collect();
    assert_eq!(reached, Refusal::ALL);
    let names: BTreeSet<_> = reached.iter().map(|refusal| refusal.name()).collect();
    assert_eq!(names.len(), reached.len());
    assert!(
        names
            .iter()
            .all(|name| name.chars().all(|c| c.is_ascii_lowercase() || c == '-'))
    );
}

#[test]
fn sealed_precedes_ash_overflow_and_structural_guards_precede_sealed() {
    let maximum = PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 1;
    refuses(0, 0, &[(1, maximum), (2, 1)], 2, Refusal::Sealed);
    refuses(0, 0, &[], 2, Refusal::EmptyBatch);
    refuses(0, 0, &[(1, 2), (2, 3)], 1, Refusal::BatchTooLarge);
    refuses(0, 0, &[(1, 2), (1, 2)], 2, Refusal::DuplicateInput);
}

fn assert_assignment(predecessor: StateMetadata, b: u64, x: u64, residual: u64) {
    let result = clear_ash(&predecessor, &[(1, amount(b))], 1).unwrap();
    assert_eq!(result.clear_amount, amount(x));
    assert_eq!(result.residual, amount(residual));
    assert_eq!(StateField::ALL.len(), 6);
    assert_eq!(StateField::ALL.iter().collect::<BTreeSet<_>>().len(), 6);
    for field in StateField::ALL {
        match field {
            StateField::Omega => assert_eq!(result.successor.omega, predecessor.omega),
            StateField::YL => {
                assert_eq!(result.successor.y_l.get(), predecessor.y_l.get() - x);
            }
            StateField::YT => assert_eq!(result.successor.y_t, predecessor.y_t),
            StateField::Q => assert_eq!(result.successor.q, predecessor.q),
            StateField::Cycle => assert_eq!(result.successor.cycle, predecessor.cycle),
            StateField::Maturity => assert_eq!(result.successor.maturity, predecessor.maturity),
        }
    }
    let y_next = result.successor.y_l.get() + result.successor.y_t.get();
    assert!(y_next >= 1);
    assert_eq!(y_next, predecessor.y_l.get() + predecessor.y_t.get() - x);
    assert_eq!(b, x + residual);
}

#[test]
fn each_limiter_and_tie_full_and_partial() {
    // (YL, YT, B, X, residual): three strict limiters and all four ties.
    for (y_l, y_t, b, x, residual) in [
        (10, 3, 4, 4, 0),
        (4, 3, 10, 4, 6),
        (5, 0, 10, 4, 6),
        (4, 3, 4, 4, 0),
        (5, 0, 4, 4, 0),
        (4, 1, 10, 4, 6),
        (4, 1, 4, 4, 0),
    ] {
        for maturity in [
            Maturity::Unannounced,
            Maturity::Announced {
                cycle: Cycle::new(17),
            },
            Maturity::Complete,
        ] {
            assert_assignment(state(y_l, y_t).with_maturity(maturity), b, x, residual);
        }
    }
}

#[test]
fn batch_boundary_aggregate_and_maximum_domain_accept() {
    let predecessor = state(PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 1, 0);
    let result = clear_ash(&predecessor, &[(1, amount(2)), (2, amount(3))], 2).unwrap();
    assert_eq!(result.clear_amount, amount(5));
    assert_eq!(result.residual, ProtocolAmount::ZERO);
    assert_assignment(
        predecessor,
        PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 1,
        PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 2,
        1,
    );
}

#[test]
fn clear_refusal_tracing_preserves_the_return_value() {
    use super::tracing_tests::{capture, in_capture_child};
    if !in_capture_child(
        "tests::clear_state_tests::clear_refusal_tracing_preserves_the_return_value",
    ) {
        return;
    }
    let predecessor = state(0, 0);
    let ash = [(1, amount(2))];
    let plain = clear_ash(&predecessor, &ash, 1);
    let (captured, records) = capture(|| clear_ash(&predecessor, &ash, 1));
    assert_eq!(plain, captured);
    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.target, "realization::state");
    assert_eq!(record.level, tracing::Level::DEBUG);
    assert_eq!(record.fields["message"], "clear transition refused");
    assert_eq!(record.fields["outcome"], "refused");
    assert_eq!(record.fields["refusal_kind"], "sealed");
    assert_eq!(record.fields.len(), 3);
    assert_eq!(record.types.get("refusal_kind"), Some(&"str"));
    assert_eq!(record.types.get("outcome"), Some(&"str"));
    super::tracing_tests::assert_no_warning_or_error(&records);
    let (_, records) = capture(|| clear_ash(&state(10, 3), &ash, 1).unwrap());
    assert!(records.is_empty());
}

fn clear_operand() -> impl Strategy<Value = u64> {
    prop_oneof![
        Just(0),
        Just(1),
        Just(2),
        0_u64..8,
        Just(PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 1),
        0..PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE,
    ]
}

fn positive_clear_operand() -> impl Strategy<Value = u64> {
    prop_oneof![
        Just(1),
        Just(2),
        1_u64..8,
        Just(PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE - 1),
        1..PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE,
    ]
}

proptest! {
    #[test]
    fn clear_matches_wide_arithmetic(
        y_l in clear_operand(),
        y_t in clear_operand(),
        values in prop::collection::vec(positive_clear_operand(), 1..5),
    ) {
        let ash: Vec<_> = values.iter().enumerate().map(|(id, &value)| (id, amount(value))).collect();
        let y = u128::from(y_l) + u128::from(y_t);
        let b: u128 = values.iter().map(|&value| u128::from(value)).sum();
        let limit = u128::from(PROTOCOL_AMOUNT_LIMIT_EXCLUSIVE);
        let result = clear_ash(&state(y_l, y_t), &ash, 4);
        if y >= limit {
            prop_assert_eq!(result, Err(Refusal::SupplyOutOfDomain));
        } else if y == 0 {
            prop_assert_eq!(result, Err(Refusal::Sealed));
        } else if b >= limit {
            prop_assert_eq!(result, Err(Refusal::AshTotalOutOfDomain));
        } else {
            let x = b.min(u128::from(y_l)).min(y - 1);
            if x == 0 {
                prop_assert_eq!(result, Err(Refusal::ZeroProgress));
            } else {
                let result = result.unwrap();
                prop_assert_eq!(u128::from(result.clear_amount.get()), x);
                prop_assert_eq!(u128::from(result.residual.get()), b - x);
                prop_assert_eq!(u128::from(result.successor.y_l.get()), u128::from(y_l) - x);
            }
        }
    }
}
