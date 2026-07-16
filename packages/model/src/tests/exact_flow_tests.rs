//! Exact-flow destruction test (kernel structural).
//!
//! Implements `´test:verification:exact-flow-destruction´` and
//! `(´test:verification:kernel-structural-validity´)`.
//!
//! This test proves both:
//!
//! - exact flow partition supports output plus destruction;
//! - branch-specific tag policy still constrains which destruction is
//!   legal.

use super::test_fixtures::*;
use crate::kernel::{DestructionLeg, TxBuilder, mixed_flow};
use crate::*;

#[test]
fn source_partition_can_mix_output_and_destruction() {
    let world = world();

    let receipt = find_live_receipt(&world, GENESIS_OWNER);

    let value = world.utxo(receipt).unwrap().value;

    let keep = Sat::new(value.get() / 2).unwrap();

    let destroy = value.checked_sub(keep).unwrap();

    let mut tx = TxBuilder::new(&world, BranchKind::Burn, order(1));

    tx.consume(receipt).unwrap();

    let ash = tx.emit(Asset::U, keep, Meta::Ash);

    tx.declare_flow(mixed_flow(
        Asset::U,
        DeltaKind::Lateral,
        vec![receipt],
        vec![ash],
        vec![DestructionLeg {
            tag: Tag::Recon,
            amount: destroy,
        }],
    ))
    .unwrap();

    // The branch family is intentionally wrong:
    // Burn is not allowed to destroy U under tag_recon.
    assert_eq!(tx.finish(), Err(Guard::BadAuthorization));
}
