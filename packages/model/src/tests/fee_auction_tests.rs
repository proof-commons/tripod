//! Any-valid-winner fee-auction safety smoke test.
//!
//! Implements `´test:verification:native-fee-auction´`.
//!
//! The candidate set is representative, not exhaustive; the stronger
//! two-valid-candidate contention test lives in
//! `stronger_fee_auction_tests`.

use super::scenario_fixtures::*;
use super::test_fixtures;
use crate::*;

#[test]
fn representative_valid_state_candidates_preserve_invariant() {
    let world = test_fixtures::world();

    let candidates = vec![
        StateCandidate::Announce(AnnounceMaturity {
            maturity_cycle: world.constants.min_maturity_lead,

            signers: signers(&[OPERATOR_KEY]),

            fee_envelope: FeeEnvelope::default(),
        }),
        StateCandidate::Cycle(RunCycle {
            caller: CycleCaller::Anyone,

            operator_signers: SignerSet::new(),

            fee_envelope: FeeEnvelope::default(),
        }),
    ];

    for candidate in candidates {
        let order = next_order(&world);

        match candidate.apply(&world, order) {
            Ok(next) => {
                check_invariant(&next).unwrap();

                let certificate = next.history.transitions.last().unwrap();

                assert!(certificate.consumed.contains(&world.roots.state));
            }

            Err(Guard::CadenceTooEarly) => {
                // Candidate is not valid in this state.
            }

            Err(error) => {
                panic!("unexpected candidate error: {error:?}");
            }
        }
    }
}
