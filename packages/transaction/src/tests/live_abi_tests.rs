//! The derived candidate live-transfer ABI (§12.1, §12.2, §12.9).

use std::collections::BTreeSet;

use linker::LiveLinkObligation;
use linker::live_backend::{
    LiveFamily, LiveFamilyRange, LiveInputFamily, LiveOutputFamily, LiveTransferRepresentationPlan,
    OwnerProfileDisposition,
};

use super::live_support::{
    FIRST_OWNER, RefusingCurve, SECOND_OWNER, linked_live_bundle, live_abi, owner,
};
use super::reviewed_target;
use crate::error::TransactionRefusal;
use crate::live_abi::{
    LiveAbiObligation, LiveAbiStatus, LiveCanonicalOrdering, LiveTransactionForm, LiveWitnessItem,
    derive_live_transfer_abi,
};
use crate::live_request::ProtocolValue;

// --- A candidate, and nothing more ------------------------------------

#[test]
fn the_derived_abi_is_a_candidate_and_owes_at_least_a_target_run() {
    let abi = live_abi();
    assert_eq!(abi.status(), LiveAbiStatus::Candidate);

    let obligations = abi.outstanding_obligations();
    assert_eq!(obligations.count().get(), 4);
    assert!(obligations.holds(LiveAbiObligation::TargetExecutionEvidenceAbsent));
    assert!(obligations.holds(LiveAbiObligation::SelectedSighashProfileUnreviewed));
    assert!(obligations.holds(LiveAbiObligation::InternalKeyUnspendabilityUnverified));
    assert!(obligations.holds(LiveAbiObligation::ConfidentialFieldFormSettledOnlyOnTheTarget));
}

#[test]
fn the_two_obligations_the_link_owed_are_both_discharged_and_none_is_carried() {
    // §11's handoff owes exactly two, and §12 answers both: the
    // destination constructor table is the selection the linker had no
    // request for, and the output key is the tweak the capability
    // performed over the committed root.
    let abi = live_abi();
    let inherited = abi.inherited_link_obligations();

    assert_eq!(
        inherited.discharged(),
        &BTreeSet::from([
            LiveLinkObligation::TaprootOutputKeyUndischarged,
            LiveLinkObligation::DestinationConstructorTableUndischarged,
        ]),
    );
    assert_eq!(inherited.carried(), &BTreeSet::new());

    // And the partition is the handoff's own set rather than a list this
    // crate wrote down.
    let bundle = linked_live_bundle();
    let accounted: BTreeSet<_> = inherited
        .discharged()
        .union(inherited.carried())
        .copied()
        .collect();
    assert_eq!(&accounted, bundle.abi_handoff().owed());
}

#[test]
fn the_selected_sighash_profile_travels_with_its_review_disposition() {
    // The residual §1.7 leaves open: the profile is the link's, and its
    // disposition comes with it rather than being restated here.
    let abi = live_abi();
    assert!(!abi.sighash_profile().is_established());
    assert!(matches!(
        abi.sighash_profile().disposition(),
        OwnerProfileDisposition::ReviewIncomplete { .. },
    ));
    assert!(
        abi.outstanding_obligations()
            .holds(LiveAbiObligation::SelectedSighashProfileUnreviewed)
    );
}

#[test]
fn the_protected_data_census_is_the_fifteen_the_profile_covers() {
    // §1.7's list, reached through the profile's own coverage map rather
    // than transcribed. Fifteen protected data, every one of them
    // carried by some required dimension.
    let abi = live_abi();
    assert_eq!(abi.protected_data().len(), 15);
}

// --- The input layout (§12.1) -----------------------------------------

#[test]
fn every_shape_puts_the_coordinator_at_input_zero() {
    let abi = live_abi();
    assert_eq!(abi.coordinator().index(), 0);
    assert_ne!(abi.shapes().len(), 0);
    for layout in abi.shapes().values() {
        assert_eq!(layout.coordinator().index(), 0);
        assert_eq!(layout.receipt_input_range().0, 0);
    }
}

#[test]
fn the_receipt_family_precedes_the_sponsor_suffix_in_every_sponsored_shape() {
    let abi = live_abi();
    let mut sponsored = 0_usize;
    for layout in abi.shapes().values() {
        if layout.form() != LiveTransactionForm::Sponsored {
            continue;
        }
        sponsored += 1;
        let (_, receipts_end) = layout.receipt_input_range();
        let (sponsor_first, sponsor_end) = layout.sponsor_input_range();
        assert_eq!(sponsor_first, receipts_end);
        assert!(sponsor_end > sponsor_first);
    }
    assert_ne!(sponsored, 0);
}

#[test]
fn the_ordering_is_ascending_outpoint_and_the_witness_order_is_the_three_items() {
    let abi = live_abi();
    assert_eq!(abi.ordering(), LiveCanonicalOrdering::AscendingOutpoint);
    assert_eq!(
        abi.witness_order(),
        &[
            LiveWitnessItem::OwnerSignature,
            LiveWitnessItem::LeafScript,
            LiveWitnessItem::ControlBlock,
        ],
    );
}

#[test]
fn the_coordinator_executes_the_coordinator_leaf_and_the_rest_the_member_leaf() {
    let abi = live_abi();
    let layout = abi
        .shapes()
        .values()
        .find(|layout| layout.shape().receipt_inputs() == 3)
        .expect("the candidate admits a three-receipt shape");

    let first = layout
        .receipt_leaf(LiveTransferRepresentationPlan::Explicit, 0)
        .expect("input zero is a receipt");
    let second = layout
        .receipt_leaf(LiveTransferRepresentationPlan::Explicit, 1)
        .expect("input one is a receipt");
    assert_ne!(first, second);
    assert_eq!(
        layout.receipt_leaf(LiveTransferRepresentationPlan::Explicit, 1),
        layout.receipt_leaf(LiveTransferRepresentationPlan::Explicit, 2),
    );
    assert_eq!(
        layout.receipt_leaf(LiveTransferRepresentationPlan::Explicit, 9),
        Err(TransactionRefusal::ReceiptPositionOutsideFamily { position: 9 }),
    );
}

// --- The output layout (§12.2) ----------------------------------------

#[test]
fn destinations_come_first_then_sponsor_change_then_the_fee_role() {
    let abi = live_abi();
    let mut with_change = 0_usize;
    for layout in abi.shapes().values() {
        let (first, end) = layout.destination_range();
        assert_eq!(first, 0);
        assert!(end > first);

        if let Some(change) = layout.sponsor_change_position() {
            with_change += 1;
            assert!(change >= end);
            let fee = layout
                .fee_position()
                .expect("a sponsored shape carries the fee role");
            assert!(fee > change);
        }
    }
    assert_ne!(with_change, 0);
}

#[test]
fn a_sponsorless_shape_carries_neither_sponsor_change_nor_a_fee_role() {
    let abi = live_abi();
    let mut sponsorless = 0_usize;
    for layout in abi.shapes().values() {
        if layout.form() != LiveTransactionForm::Sponsorless {
            continue;
        }
        sponsorless += 1;
        assert_eq!(layout.sponsor_change_position(), None);
        assert_eq!(layout.fee_position(), None);
        assert_eq!(
            layout.sponsor_input_range().0,
            layout.sponsor_input_range().1
        );
    }
    assert_ne!(sponsorless, 0);
}

#[test]
fn every_shape_the_handoff_admits_has_a_layout_and_the_ranges_are_the_links_own() {
    let bundle = linked_live_bundle();
    let abi = live_abi();

    let admitted: BTreeSet<_> = bundle.abi_handoff().shapes().shapes().collect();
    let laid_out: BTreeSet<_> = abi.shapes().keys().copied().collect();
    assert_eq!(admitted, laid_out);
    assert_eq!(laid_out.len(), 27);

    for (shape, layout) in abi.shapes() {
        let linked = bundle
            .abi_handoff()
            .family_ranges()
            .get(shape)
            .expect("the handoff places every admitted shape");
        assert_eq!(layout.ranges(), linked);
        // And the layout's own positions agree with the placement it
        // came from, rather than being a second derivation.
        assert_eq!(
            layout.destination_range(),
            linked
                .range(LiveFamily::Output(LiveOutputFamily::Destination))
                .map(|range| (range.first(), range.end()))
                .expect("every shape places its destinations"),
        );
        assert_eq!(
            layout.receipt_input_range().0,
            linked
                .range(LiveFamily::Input(LiveInputFamily::Coordinator))
                .map(LiveFamilyRange::first)
                .expect("every shape places its coordinator"),
        );
    }
}

// --- The destination constructor table (§12.3's induction) ------------

#[test]
fn the_table_holds_one_constructor_per_owner_and_representation() {
    let abi = live_abi();
    assert_eq!(abi.destinations().entries().len(), 4);
    for representation in [
        LiveTransferRepresentationPlan::Explicit,
        LiveTransferRepresentationPlan::PrivateCommitted,
    ] {
        assert_eq!(
            abi.destinations().owners(representation),
            BTreeSet::from([owner(&FIRST_OWNER), owner(&SECOND_OWNER)]),
        );
    }
}

#[test]
fn two_owners_under_one_representation_pay_to_two_different_programs() {
    // The whole content of owner parameterization at the ABI: if the
    // destination program did not depend on the owner, the table would
    // be decoration.
    let abi = live_abi();
    let first = abi
        .destinations()
        .get(
            &owner(&FIRST_OWNER),
            LiveTransferRepresentationPlan::Explicit,
        )
        .expect("the first owner has a constructor");
    let second = abi
        .destinations()
        .get(
            &owner(&SECOND_OWNER),
            LiveTransferRepresentationPlan::Explicit,
        )
        .expect("the second owner has a constructor");

    assert_ne!(first.instance().program(), second.instance().program());
    assert_ne!(
        first.instance().tree().merkle_root(),
        second.instance().tree().merkle_root(),
    );
    assert_eq!(first.placement().owner(), &owner(&FIRST_OWNER));
    assert_eq!(second.placement().owner(), &owner(&SECOND_OWNER));
}

#[test]
fn one_owner_under_two_representations_pays_to_two_different_programs() {
    let abi = live_abi();
    let explicit = abi
        .destinations()
        .get(
            &owner(&FIRST_OWNER),
            LiveTransferRepresentationPlan::Explicit,
        )
        .expect("the explicit constructor");
    let private = abi
        .destinations()
        .get(
            &owner(&FIRST_OWNER),
            LiveTransferRepresentationPlan::PrivateCommitted,
        )
        .expect("the private constructor");
    assert_ne!(explicit.instance().program(), private.instance().program());
}

#[test]
fn a_program_is_recognized_as_exactly_one_owners_receipt() {
    let abi = live_abi();
    let program = abi
        .destinations()
        .get(
            &owner(&SECOND_OWNER),
            LiveTransferRepresentationPlan::Explicit,
        )
        .expect("the second owner has a constructor")
        .instance()
        .program()
        .to_vec();

    assert_eq!(
        abi.destinations()
            .owner_of_program(LiveTransferRepresentationPlan::Explicit, &program),
        Some(&owner(&SECOND_OWNER)),
    );
    // And not under the other representation, whose leaves are disjoint.
    assert_eq!(
        abi.destinations()
            .owner_of_program(LiveTransferRepresentationPlan::PrivateCommitted, &program),
        None,
    );
    assert_eq!(
        abi.destinations()
            .owner_of_program(LiveTransferRepresentationPlan::Explicit, &[0xde; 34]),
        None,
    );
}

#[test]
fn the_destination_program_is_a_witness_version_one_program_over_the_tweaked_key() {
    let abi = live_abi();
    let constructor = abi
        .destinations()
        .get(
            &owner(&FIRST_OWNER),
            LiveTransferRepresentationPlan::Explicit,
        )
        .expect("the first owner has a constructor");
    let instance = constructor.instance();

    let mut expected = vec![0x51, 0x20];
    expected.extend_from_slice(instance.output_key().key());
    assert_eq!(instance.program(), expected.as_slice());
    assert_eq!(instance.internal_key().len(), 32);
    assert_ne!(instance.tree().leaf_programs().len(), 0);
}

#[test]
fn every_committed_leaf_has_a_control_block_carrying_the_internal_key() {
    let abi = live_abi();
    let constructor = abi
        .destinations()
        .get(
            &owner(&FIRST_OWNER),
            LiveTransferRepresentationPlan::Explicit,
        )
        .expect("the first owner has a constructor");
    let instance = constructor.instance();

    assert_eq!(instance.tree().leaf_programs().len(), 29);
    for leaf in instance.tree().leaf_programs().keys() {
        let block = instance
            .control_block(*leaf)
            .expect("every committed leaf has a path");
        assert_eq!(block.len() % 32, 1);
        assert_eq!(&block[1..33], instance.internal_key());
    }
}

// --- The curve capability is load-bearing -----------------------------

#[test]
fn an_owner_key_that_names_no_point_has_no_destination() {
    // §1.8's second conjunct, at the one place every owner passes
    // through: the encoding gate already ran at the link, and this is
    // the question it said it could not answer.
    let result =
        derive_live_transfer_abi(&reviewed_target(), &linked_live_bundle(), &RefusingCurve);
    assert!(matches!(
        result,
        Err(TransactionRefusal::OwnerKeyIsNotACurvePoint { .. }),
    ));
}

#[test]
fn the_deployment_symbols_are_the_links_resolved_values() {
    let abi = live_abi();
    assert_eq!(
        abi.symbols().protocol_asset().internal(),
        &super::live_support::LIVE_PROTOCOL_ASSET,
    );
    assert_eq!(
        abi.symbols().reserve_asset().internal(),
        &super::live_support::LIVE_RESERVE_ASSET,
    );
    assert_eq!(abi.symbols().destination_program_version(), 1);
    assert_eq!(
        abi.symbols().sponsor_change_program(),
        &super::live_support::LIVE_SPONSOR_CHANGE_PROGRAM,
    );
    assert_ne!(abi.symbols().fee_program_digest().len(), 0);
}

#[test]
fn the_locktime_is_zero_and_a_positive_value_is_representable_elsewhere() {
    // The constant is a constant of the ABI rather than a request field
    // (§12.3), and a test naming it is what keeps a later wave from
    // making it a parameter by accident.
    assert_eq!(live_abi().lock_time(), 0);
    assert_eq!(crate::live_abi::LIVE_TRANSFER_LOCK_TIME, 0);
    assert_eq!(ProtocolValue::new(1).map(ProtocolValue::amount), Ok(1));
}
