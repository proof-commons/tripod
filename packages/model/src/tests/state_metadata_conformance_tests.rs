//! STATE metadata projection and announce-maturity conformance.
//!
//! PoolState remains the semantic owner: these tests execute the model
//! independently and compare its projected observations with realization.
//! Exhaustive conversion in both directions pins the six-field census and
//! proves that typed pipeline metadata preserves every model field.
//!
//! Sealed and BadSignature precede the maturity checks. They are authorization
//! checks, not maturity semantics, so the agreement table excludes them.

use proptest::prelude::*;

use super::test_fixtures;
use crate::*;

fn project(state: PoolState) -> realization::StateMetadata {
    let PoolState {
        omega,
        y_l,
        y_t,
        q,
        cycle,
        maturity,
    } = state;

    realization::StateMetadata {
        omega: realization::ProtocolAmount::new(omega.get()).unwrap(),
        y_l: realization::ProtocolAmount::new(y_l.get()).unwrap(),
        y_t: realization::ProtocolAmount::new(y_t.get()).unwrap(),
        q: realization::ProtocolAmount::new(q.get()).unwrap(),
        cycle: realization::Cycle::new(cycle),
        maturity: match maturity {
            Maturity::Unannounced => realization::Maturity::Unannounced,
            Maturity::Announced { cycle } => realization::Maturity::Announced {
                cycle: realization::Cycle::new(cycle),
            },
            Maturity::Complete => realization::Maturity::Complete,
        },
    }
}

fn embed(meta: realization::StateMetadata) -> PoolState {
    let realization::StateMetadata {
        omega,
        y_l,
        y_t,
        q,
        cycle,
        maturity,
    } = meta;

    PoolState {
        omega: Sat::new(omega.get()).unwrap(),
        y_l: Sat::new(y_l.get()).unwrap(),
        y_t: Sat::new(y_t.get()).unwrap(),
        q: Sat::new(q.get()).unwrap(),
        cycle: cycle.get(),
        maturity: match maturity {
            realization::Maturity::Unannounced => Maturity::Unannounced,
            realization::Maturity::Announced { cycle } => {
                Maturity::Announced { cycle: cycle.get() }
            }
            realization::Maturity::Complete => Maturity::Complete,
        },
    }
}

fn amount() -> impl Strategy<Value = u64> {
    prop_oneof![Just(0), Just(1), Just(TWO_51 - 1), 0..TWO_51]
}

fn cycle() -> impl Strategy<Value = u64> {
    prop_oneof![Just(0), Just(1), Just(u64::MAX), any::<u64>()]
}

fn model_maturity() -> impl Strategy<Value = Maturity> {
    prop_oneof![
        Just(Maturity::Unannounced),
        cycle().prop_map(|cycle| Maturity::Announced { cycle }),
        Just(Maturity::Complete),
    ]
}

fn realization_maturity() -> impl Strategy<Value = realization::Maturity> {
    prop_oneof![
        Just(realization::Maturity::Unannounced),
        cycle().prop_map(|cycle| realization::Maturity::Announced {
            cycle: realization::Cycle::new(cycle),
        }),
        Just(realization::Maturity::Complete),
    ]
}

proptest! {
    #[test]
    fn project_then_embed_is_identity(
        omega in amount(), y_l in amount(), y_t in amount(), q in amount(),
        cycle in cycle(), maturity in model_maturity(),
    ) {
        let state = PoolState {
            omega: Sat::new(omega).unwrap(),
            y_l: Sat::new(y_l).unwrap(),
            y_t: Sat::new(y_t).unwrap(),
            q: Sat::new(q).unwrap(),
            cycle,
            maturity,
        };

        prop_assert_eq!(embed(project(state)), state);
    }

    #[test]
    fn embed_then_project_is_identity(
        omega in amount(), y_l in amount(), y_t in amount(), q in amount(),
        cycle in cycle(), maturity in realization_maturity(),
    ) {
        let meta = realization::StateMetadata {
            omega: realization::ProtocolAmount::new(omega).unwrap(),
            y_l: realization::ProtocolAmount::new(y_l).unwrap(),
            y_t: realization::ProtocolAmount::new(y_t).unwrap(),
            q: realization::ProtocolAmount::new(q).unwrap(),
            cycle: realization::Cycle::new(cycle),
            maturity,
        };

        prop_assert_eq!(project(embed(meta)), meta);
    }

    #[test]
    fn constants_validity_agrees_with_lead_bounds(
        (min, max) in prop_oneof![
            (cycle(), cycle()),
            (Just(0), cycle()),
            cycle().prop_map(|value| (value, value)),
        ],
    ) {
        let mut constants = test_fixtures::constants();
        prop_assert_eq!(constants.validate(), Ok(()));
        constants.min_maturity_lead = min;
        constants.max_maturity_lead = max;
        let bounds = realization::AnnouncementLeadBounds::new(
            realization::Cycle::new(min),
            realization::Cycle::new(max),
        );

        prop_assert_eq!(constants.validate().is_ok(), bounds.is_ok());
        if bounds.is_err() {
            prop_assert_eq!(constants.validate(), Err(Guard::BadConstant));
        }
    }
}

#[test]
fn both_censuses_have_six_fields() {
    let state = PoolState {
        omega: Sat::new(2).unwrap(),
        y_l: Sat::new(3).unwrap(),
        y_t: Sat::new(5).unwrap(),
        q: Sat::new(7).unwrap(),
        cycle: 11,
        maturity: Maturity::Announced { cycle: 13 },
    };
    let PoolState {
        omega,
        y_l,
        y_t,
        q,
        cycle,
        maturity,
    } = state;
    let realization::StateMetadata {
        omega: projected_omega,
        y_l: projected_y_l,
        y_t: projected_y_t,
        q: projected_q,
        cycle: projected_cycle,
        maturity: projected_maturity,
    } = project(state);

    assert_eq!(projected_omega.get(), omega.get());
    assert_eq!(projected_y_l.get(), y_l.get());
    assert_eq!(projected_y_t.get(), y_t.get());
    assert_eq!(projected_q.get(), q.get());
    assert_eq!(projected_cycle.get(), cycle);
    assert_eq!(maturity, Maturity::Announced { cycle: 13 });
    assert_eq!(
        projected_maturity,
        realization::Maturity::Announced {
            cycle: realization::Cycle::new(13),
        },
    );
}
