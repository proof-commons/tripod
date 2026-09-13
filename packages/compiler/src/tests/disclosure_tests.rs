//! Disclosure analysis tests (Guide-3 §17.5).

use std::collections::BTreeMap;

use architecture::{ObjectId, OperationId};
use realization::{FactId, RepresentationMode, TransactionSide};

use crate::{capability::CapabilityView, proof::enumerate_feasible_plans};
use architecture::OperationId::AnnounceMaturity;

use super::{announcement_input, bound_input};
use crate::{
    CompileError,
    disclosure::{derive_disclosure, is_sponsor_amount, validate_disclosure},
    lifecycle::RepresentationChoiceId,
};

fn live_amount(side: TransactionSide) -> FactId {
    FactId::FamilyAmount {
        operation: OperationId::TransferLive,
        side,
        object: ObjectId::ReceiptLive,
    }
}

fn live_choice(mode: RepresentationMode) -> BTreeMap<RepresentationChoiceId, RepresentationMode> {
    BTreeMap::from([(
        RepresentationChoiceId {
            operation: OperationId::TransferLive,
            object: ObjectId::ReceiptLive,
        },
        mode,
    )])
}

#[test]
fn inherited_disclosure_is_preserved_exactly() {
    let input = bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let inherited = input.realization().declassification().clone();
    let analysis = derive_disclosure(&inherited, &BTreeMap::new()).expect("derives");

    assert_eq!(
        analysis.inherited_required_public,
        inherited.required_public
    );
    assert_eq!(analysis.retained_private, inherited.retained_private);
    assert!(analysis.added_required_public.is_empty());
    validate_disclosure(&analysis).expect("consistent");
}

#[test]
fn explicit_live_transfer_records_amount_disclosure_with_reason() {
    let input = bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let inherited = input.realization().declassification().clone();

    let explicit =
        derive_disclosure(&inherited, &live_choice(RepresentationMode::Explicit)).expect("derives");

    for side in [TransactionSide::Input, TransactionSide::Output] {
        let fact = live_amount(side);
        assert!(
            explicit.added_required_public.contains_key(&fact),
            "explicit representation must disclose {fact:?}",
        );
        assert!(!explicit.retained_private.contains(&fact));
        assert!(
            !explicit.added_required_public[&fact].is_empty(),
            "every added disclosure carries a typed reason",
        );
    }

    validate_disclosure(&explicit).expect("consistent");
}

#[test]
fn private_live_transfer_retains_private_amounts() {
    let input = bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let inherited = input.realization().declassification().clone();

    let private = derive_disclosure(
        &inherited,
        &live_choice(RepresentationMode::PrivateCommitted),
    )
    .expect("derives");

    assert!(private.added_required_public.is_empty());
    for side in [TransactionSide::Input, TransactionSide::Output] {
        assert!(private.retained_private.contains(&live_amount(side)));
    }
    validate_disclosure(&private).expect("consistent");
}

#[test]
fn compact_ash_adds_no_disclosure() {
    let input = bound_input(&[OperationId::CompactAsh]);
    let inherited = input.realization().declassification().clone();

    // ASH values are already public; either representation selection
    // adds nothing.
    for mode in [
        RepresentationMode::Explicit,
        RepresentationMode::PublicCommitted,
    ] {
        let analysis = derive_disclosure(
            &inherited,
            &BTreeMap::from([(
                RepresentationChoiceId {
                    operation: OperationId::CompactAsh,
                    object: ObjectId::Ash,
                },
                mode,
            )]),
        )
        .expect("derives");

        assert!(analysis.added_required_public.is_empty());
    }
}

#[test]
fn a_sponsor_amount_fact_is_rejected_even_if_marked_private() {
    let input = bound_input(&[OperationId::CompactAsh]);
    let mut inherited = input.realization().declassification().clone();
    let sponsor = FactId::FamilyAmount {
        operation: OperationId::CompactAsh,
        side: TransactionSide::Input,
        object: ObjectId::PlainLbtc,
    };
    assert!(is_sponsor_amount(&sponsor));
    inherited.retained_private.insert(sponsor);

    assert_eq!(
        derive_disclosure(&inherited, &BTreeMap::new()).unwrap_err(),
        CompileError::SponsorValueRead,
    );
}

#[test]
fn announcement_inherits_fifteen_public_facts_without_amount_additions() {
    use realization::{AnnouncementLeadBound, StateField};
    use std::collections::BTreeSet;
    let operation = AnnounceMaturity;
    let mut expected: BTreeSet<_> = [TransactionSide::Input, TransactionSide::Output]
        .into_iter()
        .flat_map(|side| {
            StateField::ALL.iter().map(move |field| FactId::StateField {
                operation,
                side,
                field: *field,
            })
        })
        .collect();
    expected.extend([
        FactId::RequestedAnnouncementCycle { operation },
        FactId::AnnouncementLead {
            operation,
            bound: AnnouncementLeadBound::Minimum,
        },
        FactId::AnnouncementLead {
            operation,
            bound: AnnouncementLeadBound::Maximum,
        },
    ]);
    let input = announcement_input();
    for candidate in enumerate_feasible_plans(&input, &CapabilityView::Unconstrained)
        .unwrap()
        .candidates
    {
        assert_eq!(
            candidate
                .disclosure
                .inherited_required_public
                .keys()
                .cloned()
                .collect::<BTreeSet<_>>(),
            expected
        );
        assert_eq!(candidate.disclosure.inherited_required_public.len(), 15);
        assert!(candidate.disclosure.added_required_public.is_empty());
        assert!(candidate.disclosure.retained_private.is_empty());
    }
}
