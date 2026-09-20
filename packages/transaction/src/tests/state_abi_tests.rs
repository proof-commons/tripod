//! The candidate maturity-announcement ABI: its status, its layouts, its
//! seven witness records, the target's verdict, and the partition.
//!
//! # The ABI under test is derived over a real validated view
//!
//! Every test below derives from the shared fixture's validated view,
//! which carries the shared linked bundle. Nothing here hand-assembles an
//! ABI: a hand-assembled one would let a record agree with a witness
//! schedule no record declared and a partition agree with an obligation
//! set no link owes, which is the whole of what these tests check.
//!
//! # Three refusals this file cannot reach, and why
//!
//! The derivation refuses a bundle claiming more than the prototype
//! status, a bundle and target disagreeing about the reviewed revision,
//! and a partition that does not cover the link's own set. None of the
//! three is reachable from this crate's fixtures, and each is stated here
//! rather than quietly absent: a linked bundle's status is returned by a
//! constant function, so no link produces a bundle claiming more; the
//! crate builds one reviewed target and the linked policy is bound to
//! that same revision, so the two cannot disagree — no test in this crate
//! reaches either revision refusal, for the live generation or this one;
//! and the link's five obligations are a constant set, so the two named
//! halves always cover them. Each refusal is a contract this function
//! keeps on its own rather than a branch a fixture can currently walk.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;

use linker::{StateLinkObligation, state_witness_component};
use tapscript::{StateAnnouncementId, StateLeafRole, StateProgramComponent, StateProgramWitness};
use target_elements::{
    EncodingClass, FormAdmission, RelayCondition, StackValueType, TransactionForm,
    reviewed_transaction_forms,
};

use super::state_support::{linked_bundle, record, validated_view};
use crate::abi::{SequenceConstraint, TargetTransactionVersion};
use crate::error::TransactionRefusal;
use crate::state_abi::{
    CandidateMaturityAnnouncementAbi, MaturityAbiObligation, MaturityAbiStatus,
    MaturityOutputPlacementRule, MaturityOutputRole, MaturityRelayVerdict, MaturityWitnessRole,
    WitnessClassification, derive_maturity_announcement_abi,
};

/// The ABI over the demonstration validated view.
fn abi() -> CandidateMaturityAnnouncementAbi {
    derive_maturity_announcement_abi(&super::reviewed_target(), &validated_view())
        .expect("the demonstration validated view derives an ABI")
}

#[test]
fn the_demonstration_view_derives_a_candidate_bound_to_the_bundles_own_versions() {
    let abi = abi();
    let bundle = linked_bundle();
    assert_eq!(abi.status(), MaturityAbiStatus::Candidate);
    assert_eq!(abi.contract(), bundle.policy().target_policy());
    assert_eq!(abi.leaf_version(), bundle.policy().leaf_version());
    assert_eq!(abi.sequence(), SequenceConstraint::FinalOnEveryInput);
    assert_eq!(abi.version(), TargetTransactionVersion::Standard);
}

#[test]
fn the_partition_covers_the_links_five_exactly_with_disjoint_halves() {
    let abi = abi();
    let inherited = abi.inherited_link_obligations();
    let owed: BTreeSet<_> = linked_bundle()
        .obligations()
        .obligations()
        .copied()
        .collect();
    let accounted: BTreeSet<_> = inherited
        .discharged()
        .union(inherited.carried())
        .copied()
        .collect();

    assert_eq!(accounted, owed);
    assert_eq!(owed.len(), 5);
    assert!(inherited.discharged().is_disjoint(inherited.carried()));
    assert_eq!(
        inherited.discharged(),
        &BTreeSet::from([
            StateLinkObligation::CurrentStateValidationUndischarged,
            StateLinkObligation::AbiUnsupplied,
        ])
    );
    assert_eq!(
        inherited.carried(),
        &BTreeSet::from([
            StateLinkObligation::SuccessorNonceSearchUndischarged,
            StateLinkObligation::FinalizationAndWitnessPopulationUndischarged,
            StateLinkObligation::ModelScopeRelationsUnenforced,
        ])
    );
}

#[test]
fn the_outstanding_set_is_the_two_residuals_and_one_member_per_carried_obligation() {
    let abi = abi();
    let outstanding = abi.outstanding_obligations();
    let members: Vec<_> = outstanding.obligations().copied().collect();

    assert_eq!(
        members,
        vec![
            MaturityAbiObligation::CurrentStateRootFreshnessUnestablished,
            MaturityAbiObligation::RelayAdmissibleWitnessSplitUnopened,
            MaturityAbiObligation::SuccessorNonceSearchUndischarged,
            MaturityAbiObligation::FinalizationAndWitnessPopulationUndischarged,
            MaturityAbiObligation::ModelScopeRelationsUnenforced,
        ]
    );
    for member in &members {
        assert!(outstanding.holds(*member));
    }
    assert_eq!(
        outstanding.count(),
        NonZeroUsize::new(members.len()).expect("the set is structurally non-empty")
    );
    assert_eq!(
        members.len(),
        2 + abi.inherited_link_obligations().carried().len()
    );
}

#[test]
fn the_relay_verdict_is_the_reviewed_targets_own_refusal_with_direct_submission_as_the_route() {
    let abi = abi();
    let verdict = abi.relay_verdict();
    let reviewed = reviewed_transaction_forms();
    let review = reviewed
        .get(&TransactionForm::MaturityAnnouncement)
        .expect("the reviewed census carries the announcement form");

    assert_eq!(verdict.consensus(), FormAdmission::Admitted);
    assert_eq!(verdict.relay(), FormAdmission::Refused);
    assert_eq!(
        verdict.relay_conditions(),
        &BTreeSet::from([RelayCondition::DirectSubmissionToProducer])
    );
    assert!(!verdict.fee_output_present());

    assert_eq!(verdict.consensus(), review.consensus());
    assert_eq!(verdict.relay(), review.relay());
    assert_eq!(verdict.relay_conditions(), review.relay_conditions());
    assert_eq!(verdict.fee_output_present(), review.fee_output_present());
}

#[test]
fn a_census_carrying_no_announcement_verdict_refuses_by_name() {
    let absent = TransactionRefusal::MissingReviewedTransactionForm {
        form: TransactionForm::MaturityAnnouncement,
    };
    assert_eq!(
        MaturityRelayVerdict::read(&BTreeMap::new()).unwrap_err(),
        absent
    );

    let reviewed = reviewed_transaction_forms();
    let elsewhere = reviewed
        .get(&TransactionForm::Sponsorless)
        .expect("the reviewed census carries the sponsorless form")
        .clone();
    let crossed = BTreeMap::from([(TransactionForm::MaturityAnnouncement, elsewhere)]);
    assert_eq!(MaturityRelayVerdict::read(&crossed).unwrap_err(), absent);
}

#[test]
fn the_seven_records_are_the_schedules_own_roles_types_widths_and_positions() {
    let abi = abi();
    let record = record();
    let published = abi.witness_roles();

    let roles: Vec<_> = published.iter().map(MaturityWitnessRole::role).collect();
    assert_eq!(
        roles,
        vec![
            StateProgramWitness::SuccessorOutputKeyPrefix,
            StateProgramWitness::SuccessorNonce,
            StateProgramWitness::RequestedCycle,
            StateProgramWitness::StaticSubtreeRoot,
            StateProgramWitness::PredecessorMetadata,
            StateProgramWitness::PredecessorOutputKeyPrefix,
            StateProgramWitness::OperatorSignature,
        ]
    );
    assert_eq!(
        roles,
        record
            .witness()
            .iter()
            .map(|(role, _)| *role)
            .collect::<Vec<_>>()
    );

    let widths: Vec<_> = published
        .iter()
        .map(|entry| (entry.minimum_width(), entry.maximum_width()))
        .collect();
    assert_eq!(
        widths,
        vec![
            (Some(1), Some(1)),
            (Some(4), Some(4)),
            (Some(8), Some(8)),
            (Some(32), Some(32)),
            (Some(86), Some(86)),
            (Some(1), Some(1)),
            (Some(64), Some(64)),
        ]
    );

    let positions: Vec<_> = published
        .iter()
        .map(MaturityWitnessRole::position)
        .collect();
    assert_eq!(positions, (0..7).collect::<Vec<_>>());

    for (declared, entry) in record.witness().iter().zip(published) {
        assert_eq!(entry.declared(), &declared.1);
        assert_eq!(entry.program(), StateLeafRole::Announcement);
    }
    assert_eq!(
        published
            .iter()
            .find(|entry| entry.role() == StateProgramWitness::OperatorSignature)
            .map(MaturityWitnessRole::declared),
        Some(&StackValueType::Encoded(EncodingClass::SchnorrSignature))
    );
}

#[test]
fn every_records_consumer_is_the_carrier_maps_component_and_one_role_is_signer_held() {
    let abi = abi();
    let published = abi.witness_roles();

    for entry in published {
        assert_eq!(entry.consumer(), state_witness_component(entry.role()));
    }

    let held: Vec<_> = published
        .iter()
        .filter(|entry| entry.classification() == WitnessClassification::SignerHeld)
        .map(MaturityWitnessRole::role)
        .collect();
    assert_eq!(held, vec![StateProgramWitness::OperatorSignature]);
    assert_eq!(
        published
            .iter()
            .filter(|entry| entry.classification() == WitnessClassification::Public)
            .count(),
        6
    );

    // The successor nonce is where the two statements visibly differ: the
    // schedule declares its type through the copy-through component while
    // the component that reads it is the successor reconstruction, and the
    // record carries both rather than choosing one.
    let nonce = published
        .iter()
        .find(|entry| entry.role() == StateProgramWitness::SuccessorNonce)
        .expect("the schedule declares the successor nonce");
    assert_eq!(
        nonce.consumer(),
        StateProgramComponent::Semantic(StateAnnouncementId::SuccessorReconstruction)
    );
    assert_eq!(
        nonce.declared(),
        &record()
            .witness()
            .iter()
            .find(|(role, _)| *role == StateProgramWitness::SuccessorNonce)
            .expect("the schedule declares the successor nonce")
            .1
    );
}

#[test]
fn the_layouts_pin_input_zero_and_output_zero_and_no_role_stands_for_another() {
    let abi = abi();
    assert_eq!(abi.inputs().coordinator().index(), 0);
    assert_eq!(abi.inputs().sponsor_suffix().first(), 1);
    assert_eq!(abi.outputs().successor_position(), 0);

    assert_eq!(MaturityOutputRole::ALL.len(), 3);
    for &role in MaturityOutputRole::ALL {
        for &other in MaturityOutputRole::ALL {
            assert_eq!(role.can_satisfy(other), role == other);
        }
    }

    let placements: Vec<_> = MaturityOutputRole::ALL
        .iter()
        .map(|role| role.placement())
        .collect();
    assert_eq!(
        placements,
        vec![
            MaturityOutputPlacementRule::OutputZero,
            MaturityOutputPlacementRule::NextOptionalOutput,
            MaturityOutputPlacementRule::FinalOutputWhereRequired,
        ]
    );
}
