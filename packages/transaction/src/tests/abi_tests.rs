//! What the derived ABI states, and what it refuses to state.

use linker::backend::{InputRole, LeafRole, OutputRole};
use target_elements::{FormAdmission, TransactionForm};

use crate::abi::{
    AbiObligation, AbiStatus, CanonicalOrdering, SequenceConstraint, TargetTransactionVersion,
    WitnessItem, derive_candidate_abi,
};
use crate::error::TransactionRefusal;
use crate::taproot::{AshInstanceOrigin, OutputKeyParity, PinnedAshInstance};
use crate::tests::{
    CLOSED_ASSET, INTERNAL_KEY, PINNED_PROGRAM, RESERVE_ASSET, SPONSOR_CHANGE_PROGRAM,
    candidate_abi, linked_bundle, reviewed_target, shape,
};

#[test]
fn the_abi_states_every_item_the_guide_lists() {
    let abi = candidate_abi();

    // Input family order and the coordinator rule.
    assert_eq!(abi.ordering(), CanonicalOrdering::AscendingOutpoint);
    assert_eq!(abi.coordinator().index(), 0);

    // Witness item order, sequence constraints, and the lock time.
    assert_eq!(
        abi.witness_order(),
        &[WitnessItem::LeafScript, WitnessItem::ControlBlock]
    );
    assert_eq!(abi.sequence(), SequenceConstraint::FinalOnEveryInput);
    assert_eq!(abi.sequence().sequence(), 0xffff_ffff);
    assert_eq!(abi.lock_time(), 0);

    // The public deployment constants, read from the link rather than
    // taken as parameters.
    assert_eq!(abi.symbols().closed_asset().internal(), &CLOSED_ASSET);
    assert_eq!(abi.symbols().reserve_asset().internal(), &RESERVE_ASSET);
    assert_eq!(
        abi.symbols().sponsor_change_program(),
        &SPONSOR_CHANGE_PROGRAM
    );
    assert_eq!(abi.symbols().sponsor_change_version(), 0);

    // The target policy status, carried whole rather than summarized.
    let forms = abi.target_policy_status();
    assert_eq!(
        forms
            .get(&TransactionForm::Sponsorless)
            .map(target_elements::TransactionFormReview::consensus),
        Some(FormAdmission::Admitted)
    );
    assert_eq!(
        forms
            .get(&TransactionForm::Sponsorless)
            .map(target_elements::TransactionFormReview::relay),
        Some(FormAdmission::AdmittedUnderCondition)
    );

    // Concrete relation placements, and a non-empty candidate bound
    // census.
    assert!(!abi.placements().is_empty());
    assert!(!abi.candidate_bounds().is_empty());
    assert!(!abi.witness_roles().is_empty());
}

#[test]
fn every_shape_states_its_ranges_roles_and_leaves() {
    let abi = candidate_abi();

    let sponsorless = abi.shape(shape(2, 0, false)).expect("a sponsorless shape");
    assert_eq!(sponsorless.ash_range(), (0, 2));
    assert_eq!(sponsorless.sponsor_range(), (2, 2));
    assert_eq!(sponsorless.successor_position(), 0);
    assert_eq!(sponsorless.sponsor_change_position(), None);
    assert_eq!(sponsorless.fee_position(), None);
    assert_eq!(sponsorless.form(), TransactionForm::Sponsorless);
    assert_eq!(
        sponsorless.version(),
        TargetTransactionVersion::TopologyRestricted
    );
    assert_eq!(sponsorless.version().version(), 3);
    assert_eq!(
        sponsorless.tapleaf().get(&InputRole::Coordinator),
        Some(&LeafRole::Coordinator {
            shape: shape(2, 0, false)
        })
    );
    assert_eq!(
        sponsorless.tapleaf().get(&InputRole::Member),
        Some(&LeafRole::Member { ash_inputs: 2 })
    );

    let sponsored = abi.shape(shape(2, 1, true)).expect("a sponsored shape");
    assert_eq!(sponsored.ash_range(), (0, 2));
    assert_eq!(sponsored.sponsor_range(), (2, 3));
    assert_eq!(sponsored.form(), TransactionForm::Sponsored);
    assert_eq!(sponsored.version(), TargetTransactionVersion::Standard);
    assert_eq!(sponsored.version().version(), 2);
    assert!(sponsored.sponsor_change_position().is_some());
    assert!(sponsored.fee_position().is_some());

    // The fee role sits last, which is the target's own interface
    // convention rather than a consensus rule — and the ABI follows it
    // because a construction path through that interface has to.
    let last = u16::try_from(sponsored.layout().outputs().len() - 1).expect("a short layout");
    assert_eq!(sponsored.fee_position(), Some(last));
}

#[test]
fn the_abi_is_a_candidate_and_owes_at_least_the_pinned_output_key() {
    let abi = candidate_abi();
    assert_eq!(abi.status(), AbiStatus::Candidate);

    let obligations = abi.outstanding_obligations();
    assert!(obligations.holds(AbiObligation::PinnedOutputKeyUnverifiedAgainstTree));
    assert!(obligations.holds(AbiObligation::InternalKeyUnspendabilityUnverified));
    assert!(obligations.holds(AbiObligation::ClearLifecycleAbsent));
    assert!(obligations.holds(AbiObligation::TargetExecutionEvidenceAbsent));
    assert_eq!(obligations.count().get(), 4);
    assert_eq!(obligations.obligations().count(), 4);
}

#[test]
fn a_pin_naming_another_internal_key_is_refused() {
    let mut key = INTERNAL_KEY;
    key[0] ^= 0xff;
    let pin = PinnedAshInstance::new(
        &PINNED_PROGRAM,
        OutputKeyParity::Even,
        target_elements::LeafVersion::TAPSCRIPT,
        key.to_vec(),
        AshInstanceOrigin::SyntheticTestFunding,
    )
    .expect("the fixture program is the reviewed width");

    assert_eq!(
        derive_candidate_abi(&reviewed_target(), &linked_bundle(), pin),
        Err(TransactionRefusal::PinnedInternalKeyMismatch)
    );
}

#[test]
fn a_pin_of_the_wrong_width_is_refused() {
    let refusal = PinnedAshInstance::new(
        &[0xcc; 20],
        OutputKeyParity::Even,
        target_elements::LeafVersion::TAPSCRIPT,
        INTERNAL_KEY.to_vec(),
        AshInstanceOrigin::SyntheticTestFunding,
    )
    .expect_err("a twenty-byte program is not a taproot program");
    assert_eq!(
        refusal,
        TransactionRefusal::MalformedPinnedProgram { offered: 20 }
    );
}

#[test]
fn the_pinned_program_builds_a_witness_version_one_script() {
    let target = reviewed_target();
    let script = crate::tests::pin()
        .output_script(&target)
        .expect("the pinned program builds a script");
    // Version one is the small-number push of one, and the payload's
    // length prefix is the direct push whose opcode is the width.
    assert_eq!(script.len(), 34);
    assert_eq!(script[0], 0x51);
    assert_eq!(script[1], 0x20);
    assert_eq!(&script[2..], &PINNED_PROGRAM);
}

#[test]
fn witness_version_zero_is_the_empty_push_and_not_the_small_number_one() {
    // The distinction that would otherwise produce a well-formed and
    // unspendable script: zero comes from a different push form than
    // one does, and its opcode is not one less.
    let target = reviewed_target();
    let script = crate::taproot::witness_program_script(&target, 0, &[0xd0; 20])
        .expect("a version-zero program builds");
    assert_eq!(script[0], 0x00);
    assert_eq!(script[1], 0x14);
    assert_ne!(script[0], 0x50);
}

#[test]
fn the_representation_and_value_policy_come_from_the_linked_constructor() {
    let abi = candidate_abi();
    let bundle = linked_bundle();
    assert_eq!(abi.representation(), bundle.constructor().representation());
    assert_eq!(abi.value_policy(), bundle.constructor().value_policy());
    assert_eq!(
        abi.internal_key_policy(),
        bundle.constructor().internal_key_policy()
    );
    assert_eq!(abi.key_path(), bundle.constructor().key_path());
    assert_eq!(abi.assumptions(), bundle.constructor().assumptions());
    assert_eq!(abi.leaf_version(), bundle.constructor().leaf_version());
}

#[test]
fn the_package_limits_are_the_reviewed_topology_restricted_ones() {
    let limits = candidate_abi().package_limits();
    assert_eq!(limits.transactions(), 2);
    assert_eq!(limits.parent_virtual_size(), 10_000);
    assert_eq!(limits.child_virtual_size(), 1_000);
}

#[test]
fn every_shape_places_its_successor_first() {
    // §10.2's output order, checked across the whole admitted set
    // rather than on one shape: a layout that put the successor
    // anywhere else would still be internally consistent.
    let abi = candidate_abi();
    assert!(!abi.shapes().is_empty());
    for shape in abi.shapes().values() {
        assert_eq!(shape.successor_position(), 0);
        assert_eq!(
            shape.layout().output_position(OutputRole::Successor),
            Some(0)
        );
        assert_eq!(shape.ash_range().0, 0);
    }
}
