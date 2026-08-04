//! Runtime bound conformance against architecture cardinality minima.
//!
//! A nonzero finite bound is not sufficient: `ash_batch_max = 1` is
//! nonzero yet makes `compact-ash` (declared minimum of two ASH
//! inputs) unsatisfiable. The runtime check must derive each bound's
//! floor from the typed architecture — the same authority deployment
//! calibration validates against — so the model and the deployment
//! profile cannot disagree about which constants are runnable.

use architecture::{ARCHITECTURE, BoundId, manifest_minimum_for_bound};

use super::test_fixtures;
use crate::*;

fn set_bound(constants: &mut Constants, bound: BoundId, value: usize) {
    match bound {
        BoundId::AdmissionBatchMax => constants.admission_batch_max = value,
        BoundId::SettlementBatchMax => constants.settlement_batch_max = value,
        BoundId::RelabelBatchMax => constants.relabel_batch_max = value,
        BoundId::AshBatchMax => constants.ash_batch_max = value,
        BoundId::BurnInputMax => constants.burn_input_max = value,
        BoundId::BurnChangeMax => constants.burn_change_max = value,
        BoundId::BurnRecordMax => constants.burn_record_max = value,
        BoundId::TransferInputMax => constants.transfer_input_max = value,
        BoundId::TransferOutputMax => constants.transfer_output_max = value,
        BoundId::FeeSponsorInputMax => constants.fee_sponsor_input_max = value,
    }
}

#[test]
fn fixture_constants_conform() {
    validate_bound_conformance(&test_fixtures::constants()).unwrap();
}

/// One mutation per declared bound: a runtime value strictly below
/// the architecture's derived minimum must be rejected, and the exact
/// minimum must be accepted, for every bound with a nonzero semantic
/// minimum.
#[test]
fn every_bound_must_dominate_its_manifest_minimum() {
    for bound in ARCHITECTURE.bounds {
        let minimum = manifest_minimum_for_bound(&ARCHITECTURE, bound.id);

        let minimum = usize::try_from(minimum).unwrap();

        if minimum == 0 {
            continue;
        }

        let mut below = test_fixtures::constants();
        set_bound(&mut below, bound.id, minimum - 1);

        assert_eq!(
            validate_bound_conformance(&below),
            Err(Guard::BadConstant),
            "bound {:?} below its manifest minimum must be rejected",
            bound.id,
        );

        let mut exact = test_fixtures::constants();
        set_bound(&mut exact, bound.id, minimum);

        assert_eq!(
            validate_bound_conformance(&exact),
            Ok(()),
            "bound {:?} at its manifest minimum must be accepted",
            bound.id,
        );
    }
}

/// The compact-ash floor is a derived fact of the typed architecture,
/// not a hardcoded constant: at least one declared cardinality
/// requires two ASH inputs.
#[test]
fn ash_batch_minimum_is_two_by_derivation() {
    assert_eq!(
        manifest_minimum_for_bound(&ARCHITECTURE, BoundId::AshBatchMax),
        2
    );
}

/// Raw model-world construction shares the deployment profile's bound
/// authority: a genesis whose constants cannot satisfy compact-ash
/// must not exist.
#[test]
fn genesis_rejects_ash_batch_max_below_the_compact_ash_minimum() {
    let mut constants = test_fixtures::constants();
    constants.ash_batch_max = 1;

    let result = genesis(
        constants,
        Sat::new(1_000_000).unwrap(),
        CanonicalOrder {
            height: 0,
            tx_index: 0,
        },
        TxId([0_u8; 32]),
    );

    assert_eq!(result, Err(Guard::BadConstant));
}

// --- T4: the global invariant welds the same bound authority. ---

/// The named regression from the static review: `ash_batch_max = 1` is
/// nonzero (so `Constants::validate` accepts it) yet makes compact-ash
/// unconstructible. A publicly mutated world carrying it must fail the
/// global invariant, not only genesis.
#[test]
fn ash_batch_max_of_one_fails_the_global_invariant() {
    let mut world = test_fixtures::world();
    world.constants.ash_batch_max = 1;

    assert!(world.constants.validate().is_ok());
    assert!(validate_bound_conformance(&world.constants).is_err());
    assert_eq!(check_invariant(&world), Err(InvariantError::Domains));
}

/// For every declared bound with a nonzero derived minimum:
/// minimum-minus-one fails the global invariant; the exact minimum
/// passes when the rest of the fixture is valid.
#[test]
fn global_invariant_enforces_every_architecture_bound_minimum() {
    for bound in ARCHITECTURE.bounds {
        let minimum = usize::try_from(manifest_minimum_for_bound(&ARCHITECTURE, bound.id)).unwrap();

        if minimum == 0 {
            continue;
        }

        let mut below = test_fixtures::world();
        set_bound(&mut below.constants, bound.id, minimum - 1);
        assert_eq!(
            check_invariant(&below),
            Err(InvariantError::Domains),
            "bound {:?} below its manifest minimum must fail the invariant",
            bound.id,
        );

        let mut exact = test_fixtures::world();
        set_bound(&mut exact.constants, bound.id, minimum);
        assert_eq!(
            check_invariant(&exact),
            Ok(()),
            "bound {:?} at its manifest minimum must pass the invariant",
            bound.id,
        );
    }
}
