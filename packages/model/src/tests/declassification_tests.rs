//! The declassification map — `generated/declassification.json`.
//!
//! The derivation lives in [`crate::artifacts`] (see its module docs
//! for the derivation law); these tests pin the private membrane and
//! check the committed artifact without writing. Regeneration is
//! `generate-all`'s job, never a test side effect.

use crate::artifacts::{declassification_json, declassification_rows};

/// The committed artifact, embedded at compile time so cargo tracks
/// the fixture and the test never resolves a repository path at
/// runtime (ADR-014 hermetic-test rule).
const COMMITTED_INDEX: &str = include_str!("../../generated/declassification.json");

/// The private membrane, pinned: lateral-only operations declassify
/// nothing.
#[test]
fn lateral_operations_declassify_nothing() {
    let map = declassification_rows();

    for lateral in [
        "transfer-live-receipts",
        "transfer-time-locked-receipts",
        "receipt-relabel",
        "compact-ash",
    ] {
        let row = map
            .iter()
            .find(|row| row.operation == lateral)
            .expect("declared operation is mapped");

        assert!(
            row.declassifies.is_empty(),
            "lateral operation {lateral} must declassify nothing, got {:?}",
            row.declassifies,
        );
    }
}

/// Boundary operations declassify their public synchronization facts.
#[test]
fn boundary_operations_declassify_their_deltas() {
    let map = declassification_rows();

    let expect_nonempty = ["admit-deposits", "cycle", "redeem", "burn", "clear"];

    for boundary in expect_nonempty {
        let row = map
            .iter()
            .find(|row| row.operation == boundary)
            .expect("declared operation is mapped");

        assert!(
            !row.declassifies.is_empty(),
            "boundary operation {boundary} must declassify its public facts",
        );
    }
}

/// The committed artifact equals the in-memory derivation. Pure check:
/// the test never writes; a stale artifact is regenerated with
/// `meson compile -C build generate-artifacts`.
#[test]
fn declassification_index_is_current() {
    let rendered = declassification_json();

    assert_eq!(
        COMMITTED_INDEX, rendered,
        "generated/declassification.json is stale; run \
         `meson compile -C build generate-artifacts` \
         and commit the diff",
    );
}
