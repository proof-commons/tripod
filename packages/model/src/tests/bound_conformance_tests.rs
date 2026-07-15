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
