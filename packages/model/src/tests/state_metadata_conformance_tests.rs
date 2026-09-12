//! STATE metadata projection and announce-maturity conformance.
//!
//! PoolState remains the semantic owner: these tests execute the model
//! independently and compare its projected observations with realization.
//! Exhaustive conversion in both directions pins the six-field census and
//! proves that typed pipeline metadata preserves every model field.
//!
//! Sealed and BadSignature precede the maturity checks. They are authorization
//! checks, not maturity semantics, so the agreement table excludes them.
//! Function equivalence compares the six fields under valid lead bounds on
//! unsealed predecessors with operator authorization. Whole-transition acceptance
//! equivalence additionally requires a valid model world and fee envelope; the
//! bound-execution tests assert both in that domain. Sealed pools and missing
//! operator signatures are model authorization outcomes outside the maturity law,
//! and their precedence tests never consult the realization function.
//!
//! | Realization refusal | Model guard |
//! | --- | --- |
//! | PredecessorAlreadyAnnounced | MaturityAlreadyAnnounced |
//! | PredecessorMaturityComplete | MaturityAlreadyAnnounced |
//! | AnnouncementBelowMinimum | MaturityLeadTooShort |
//! | AnnouncementAboveMaximum | MaturityLeadTooLong |
//! | CycleArithmeticOverflow | CycleOverflow |

use proptest::prelude::*;

use super::distribution_fixtures::mature_world;
use super::scenario_fixtures::*;
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

fn lead_bounds(constants: &Constants) -> realization::AnnouncementLeadBounds {
    realization::AnnouncementLeadBounds::new(
        realization::Cycle::new(constants.min_maturity_lead),
        realization::Cycle::new(constants.max_maturity_lead),
    )
    .unwrap()
}

fn map_refusal(refusal: realization::MaturityTransitionRefusal) -> Guard {
    use realization::MaturityTransitionRefusal;

    match refusal {
        // The model collapses announced and complete into the same guard.
        MaturityTransitionRefusal::PredecessorAlreadyAnnounced
        | MaturityTransitionRefusal::PredecessorMaturityComplete => Guard::MaturityAlreadyAnnounced,
        MaturityTransitionRefusal::AnnouncementBelowMinimum => Guard::MaturityLeadTooShort,
        MaturityTransitionRefusal::AnnouncementAboveMaximum => Guard::MaturityLeadTooLong,
        MaturityTransitionRefusal::CycleArithmeticOverflow => Guard::CycleOverflow,
    }
}

fn announcement_world() -> World {
    let world = test_fixtures::world();
    let aged = advance_blocks(&world, world.constants.max_cadence_blocks);

    // A nonzero current cycle makes confusing an ordinal with a lead observable.
    apply_checked(
        &aged,
        &RunCycle {
            caller: CycleCaller::Anyone,
            operator_signers: SignerSet::new(),
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&aged),
    )
}

fn assert_agrees(
    world: &World,
    maturity_cycle: Cycle,
    expected: Result<(), realization::MaturityTransitionRefusal>,
) -> Result<World, Guard> {
    let state = world.state().unwrap().1;
    assert!(!state.is_sealed().unwrap());
    let transition = AnnounceMaturity {
        maturity_cycle,
        signers: signers(&[OPERATOR_KEY]),
        fee_envelope: FeeEnvelope::default(),
    };
    let model_result =
        execute_bound(world, transition, next_order(world)).map(ExecutedTransition::into_world);
    let realization_result = realization::announce_maturity(
        &project(state),
        realization::Cycle::new(maturity_cycle),
        lead_bounds(&world.constants),
    );

    // Pin the expected outcome as well as agreement: two unexpected refusals
    // must not masquerade as a successful boundary test.
    assert_eq!(realization_result.map(|_| ()), expected);
    assert_eq!(
        model_result.as_ref().map(|_| ()).map_err(|guard| *guard),
        expected.map_err(map_refusal),
    );
    assert_eq!(
        model_result
            .as_ref()
            .map(|next| project(next.state().unwrap().1))
            .map_err(|guard| *guard),
        realization_result.map_err(map_refusal),
    );
    if let Ok(next) = &model_result {
        check_invariant(next).unwrap();
    }

    model_result
}

#[test]
fn minimum_lead_agrees() {
    let world = announcement_world();
    let maturity_cycle = world.state().unwrap().1.cycle + world.constants.min_maturity_lead;

    assert_agrees(&world, maturity_cycle, Ok(())).unwrap();
}

#[test]
fn maximum_lead_agrees() {
    let world = announcement_world();
    let maturity_cycle = world.state().unwrap().1.cycle + world.constants.max_maturity_lead;

    assert_agrees(&world, maturity_cycle, Ok(())).unwrap();
}

#[test]
fn one_below_minimum_agrees() {
    let world = announcement_world();
    let maturity_cycle = world.state().unwrap().1.cycle + world.constants.min_maturity_lead - 1;

    assert_agrees(
        &world,
        maturity_cycle,
        Err(realization::MaturityTransitionRefusal::AnnouncementBelowMinimum),
    )
    .unwrap_err();
}

#[test]
fn one_above_maximum_agrees() {
    let world = announcement_world();
    let maturity_cycle = world.state().unwrap().1.cycle + world.constants.max_maturity_lead + 1;

    assert_agrees(
        &world,
        maturity_cycle,
        Err(realization::MaturityTransitionRefusal::AnnouncementAboveMaximum),
    )
    .unwrap_err();
}

#[test]
fn already_announced_agrees() {
    let world = announcement_world();
    let maturity_cycle = world.state().unwrap().1.cycle + world.constants.min_maturity_lead;
    let announced = assert_agrees(&world, maturity_cycle, Ok(())).unwrap();

    // Include invalid requests to pin predecessor-status precedence over bounds.
    for requested in [maturity_cycle, 0, u64::MAX] {
        assert_agrees(
            &announced,
            requested,
            Err(realization::MaturityTransitionRefusal::PredecessorAlreadyAnnounced),
        )
        .unwrap_err();
    }
}

#[test]
fn complete_agrees() {
    // This fixture announces and runs cycles through completion; no STATE
    // metadata mutation is needed to reach Complete.
    let world = mature_world(&test_fixtures::world());
    check_invariant(&world).unwrap();
    assert_eq!(world.state().unwrap().1.maturity, Maturity::Complete);
    let maturity_cycle = world.state().unwrap().1.cycle + world.constants.min_maturity_lead;

    for requested in [maturity_cycle, 0, u64::MAX] {
        assert_agrees(
            &world,
            requested,
            Err(realization::MaturityTransitionRefusal::PredecessorMaturityComplete),
        )
        .unwrap_err();
    }
}

#[test]
fn successor_projects_to_realization_successor() {
    let world = create_request_for(&announcement_world(), ALICE, BOB, sat(1234), Sat::ONE);
    let world = admit_all_requests(&world);
    let (predecessor_outpoint, state) = world.state().unwrap();
    assert!(!state.q.is_zero());
    assert_ne!(state.cycle, 0);
    let maturity_cycle = state.cycle + world.constants.min_maturity_lead;
    let successor = assert_agrees(&world, maturity_cycle, Ok(())).unwrap();
    let (successor_outpoint, successor_state) = successor.state().unwrap();

    assert_ne!(successor_outpoint, predecessor_outpoint);
    assert!(!successor.utxos.contains_key(&predecessor_outpoint));
    let output = successor.utxo(successor_outpoint).unwrap();
    assert_eq!(output.asset, Asset::Pid);
    assert_eq!(output.value, Sat::ONE);
    assert_eq!(output.meta, Meta::State(successor_state));
    assert_eq!(
        project(successor_state),
        realization::announce_maturity(
            &project(state),
            realization::Cycle::new(maturity_cycle),
            lead_bounds(&world.constants),
        )
        .unwrap(),
    );
}

#[test]
fn cycle_overflow_agrees() {
    let fixture = test_fixtures::world();
    let min = fixture.constants.min_maturity_lead;
    let max = fixture.constants.max_maturity_lead;
    assert!(min < max);

    // Mutate only STATE metadata: these are branch-local refusal probes, not
    // claims that a history with almost u64::MAX cycles was executed.
    // The first case overflows the earliest endpoint; the second reaches and
    // overflows the latest endpoint while the earliest is still representable.
    for current in [u64::MAX - min + 1, u64::MAX - max + 1] {
        let mut world = fixture.clone();
        let (outpoint, mut state) = world.state().unwrap();
        state.cycle = current;
        world.utxos.get_mut(&outpoint).unwrap().meta = Meta::State(state);
        assert!(current.checked_add(max).is_none());
        if current == u64::MAX - max + 1 {
            assert!(current.checked_add(min).is_some());
        } else {
            assert!(current.checked_add(min).is_none());
        }

        for requested in [0, u64::MAX] {
            assert_agrees(
                &world,
                requested,
                Err(realization::MaturityTransitionRefusal::CycleArithmeticOverflow),
            )
            .unwrap_err();
        }
    }
}

#[test]
fn every_realization_refusal_is_mapped() {
    for &refusal in realization::MaturityTransitionRefusal::ALL {
        assert!(
            matches!(
                map_refusal(refusal),
                Guard::MaturityAlreadyAnnounced
                    | Guard::MaturityLeadTooShort
                    | Guard::MaturityLeadTooLong
                    | Guard::CycleOverflow
            ),
            "unmapped refusal: {}",
            refusal.name(),
        );
    }
}

fn announcement_with_distinct_fields() -> World {
    let world = super::advanced_fixtures::give_live_receipt(&announcement_world(), ALICE, sat(37));
    let receipt = find_receipts(&world, ALICE, ReceiptClass::Live)[0];
    let redeemed = apply_checked(
        &world,
        &RedeemReceipt {
            receipt,
            signers: signers(&[ALICE]),
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&world),
    );
    let requested = create_request_for(&redeemed, ALICE, BOB, sat(1234), Sat::ONE);
    admit_all_requests(&requested)
}

#[test]
fn executed_successor_projects_to_realization_successor() {
    let world = announcement_with_distinct_fields();
    let (input, predecessor) = world.state().unwrap();
    assert_eq!(
        std::collections::BTreeSet::from([
            predecessor.omega.get(),
            predecessor.y_l.get(),
            predecessor.y_t.get(),
            predecessor.q.get(),
            predecessor.cycle,
        ])
        .len(),
        5
    );
    assert!(!predecessor.q.is_zero());
    assert_ne!(predecessor.cycle, 0);

    for lead in [
        world.constants.min_maturity_lead,
        world.constants.max_maturity_lead,
    ] {
        let request = AnnounceMaturity {
            maturity_cycle: predecessor.cycle + lead,
            signers: signers(&[OPERATOR_KEY]),
            fee_envelope: FeeEnvelope::default(),
        };
        let executed = execute_bound(&world, request, next_order(&world)).unwrap();
        let observation = observe_announce_maturity(&executed).unwrap();
        let (output, successor) = executed.after().state().unwrap();
        let realized = realization::announce_maturity(
            &project(executed.before().state().unwrap().1),
            realization::Cycle::new(executed.request().maturity_cycle),
            lead_bounds(&executed.before().constants),
        )
        .unwrap();

        assert_eq!(project(successor), realized);
        assert_eq!(successor, embed(realized));
        assert_eq!(successor.omega, predecessor.omega);
        assert_eq!(successor.y_l, predecessor.y_l);
        assert_eq!(successor.y_t, predecessor.y_t);
        assert_eq!(successor.q, predecessor.q);
        assert_eq!(successor.cycle, predecessor.cycle);
        assert_eq!(
            successor.maturity,
            Maturity::Announced {
                cycle: executed.request().maturity_cycle,
            }
        );
        assert_ne!(input, output);
        assert!(!executed.after().utxos.contains_key(&input));
        assert_eq!(
            executed.certificate().state_edge,
            Some(RootEdge::Succ { input, output })
        );
        assert_eq!(observation.observation().objects.len(), 2);
        for object in &observation.observation().objects {
            assert_eq!(
                object.kind,
                realization::ObservedObjectKind::Declared(architecture::ObjectId::State)
            );
            assert_eq!(
                object.asset,
                realization::ObservedAsset::Declared(architecture::AssetId::Pid)
            );
            assert_eq!(
                object.value,
                realization::ObservedValue::Protocol(realization::ProtocolAmount::ONE)
            );
            assert_eq!(
                object.representation,
                realization::RepresentationMode::Explicit
            );
        }
    }
}

fn overflow_execution_world(minimum: Cycle) -> World {
    let mut constants = test_fixtures::constants();
    constants.min_maturity_lead = minimum;
    constants.max_maturity_lead = u64::MAX;
    let world = genesis(
        constants,
        sat(1_000_000),
        CanonicalOrder {
            height: 0,
            tx_index: 0,
        },
        test_fixtures::txid(0),
    )
    .unwrap();
    let aged = advance_blocks(&world, world.constants.max_cadence_blocks);
    execute_bound(
        &aged,
        RunCycle {
            caller: CycleCaller::Anyone,
            operator_signers: SignerSet::new(),
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&aged),
    )
    .unwrap()
    .into_world()
}

#[test]
fn earliest_endpoint_overflow_agrees_through_execution() {
    // Valid custom genesis bounds let one executed cycle overflow the earliest
    // endpoint. With the fixed fixture bounds, reaching this ordinal would need
    // almost u64::MAX cycles; cycle_overflow_agrees uses metadata mutation there.
    let world = overflow_execution_world(u64::MAX);
    let state = world.state().unwrap().1;
    assert_eq!(state.cycle, 1);
    check_invariant(&world).unwrap();
    assert!(
        state
            .cycle
            .checked_add(world.constants.min_maturity_lead)
            .is_none()
    );
    for requested in [0, u64::MAX] {
        assert_agrees(
            &world,
            requested,
            Err(realization::MaturityTransitionRefusal::CycleArithmeticOverflow),
        )
        .unwrap_err();
    }
}

#[test]
fn latest_endpoint_overflow_agrees_through_execution() {
    // A distinct valid genesis configuration keeps the earliest endpoint in
    // range and overflows only the latest after one executed cycle. The existing
    // fixed-constant near-maximum-cycle probe remains a metadata mutation.
    let world = overflow_execution_world(test_fixtures::constants().min_maturity_lead);
    let state = world.state().unwrap().1;
    assert_eq!(state.cycle, 1);
    check_invariant(&world).unwrap();
    assert!(
        state
            .cycle
            .checked_add(world.constants.min_maturity_lead)
            .is_some()
    );
    assert!(
        state
            .cycle
            .checked_add(world.constants.max_maturity_lead)
            .is_none()
    );
    for requested in [0, u64::MAX] {
        assert_agrees(
            &world,
            requested,
            Err(realization::MaturityTransitionRefusal::CycleArithmeticOverflow),
        )
        .unwrap_err();
    }
}

#[test]
fn sealed_pool_precedes_signature_and_maturity_checks() {
    let (world, receipt) = super::advanced_fixtures::sealing_world();
    let sealed = execute_bound(
        &world,
        RedeemReceipt {
            receipt,
            signers: signers(&[GENESIS_OWNER]),
            fee_envelope: FeeEnvelope::default(),
        },
        next_order(&world),
    )
    .unwrap()
    .into_world();
    assert!(sealed.state().unwrap().1.is_sealed().unwrap());
    assert_eq!(sealed.state().unwrap().1.maturity, Maturity::Complete);

    // Even an absent signature and invalid request cannot outrank Sealed.
    // These authorization verdicts do not consult the realization function.
    for signers in [SignerSet::new(), signers(&[OPERATOR_KEY])] {
        for maturity_cycle in [0, u64::MAX] {
            let request = AnnounceMaturity {
                maturity_cycle,
                signers: signers.clone(),
                fee_envelope: FeeEnvelope::default(),
            };
            assert_eq!(
                execute_bound(&sealed, request, next_order(&sealed)),
                Err(Guard::Sealed)
            );
        }
    }
}

#[test]
fn missing_operator_signature_precedes_maturity_checks() {
    let world = announcement_world();
    let (announced, _) = super::distribution_fixtures::announce_at_minimum_lead(&world);
    let complete = mature_world(&world);

    // Unannounced, announced, and complete worlds all reject without consulting
    // realization, even when the requested cycle would violate the maturity law.
    let overflow = overflow_execution_world(test_fixtures::constants().min_maturity_lead);
    for world in [world, announced, complete, overflow] {
        check_invariant(&world).unwrap();
        assert!(!world.state().unwrap().1.is_sealed().unwrap());
        for maturity_cycle in [0, u64::MAX] {
            let request = AnnounceMaturity {
                maturity_cycle,
                signers: SignerSet::new(),
                fee_envelope: FeeEnvelope::default(),
            };
            assert_eq!(
                execute_bound(&world, request, next_order(&world)),
                Err(Guard::BadSignature)
            );
        }
    }
}
