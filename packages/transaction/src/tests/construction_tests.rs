//! The construction pipeline, end to end, against a real linked
//! bundle.
//!
//! # The sponsor here is a fixture adapter, not a signer
//!
//! [`FixtureSponsor`] holds no key and computes no signature. It
//! returns a fixed, public, meaningless stack — test material in the
//! sense `(´[ADR015-rule:security:test-material]´)` fixes — and echoes
//! the transaction it was handed. That is exactly enough to exercise
//! the boundary this crate owns: what a signing request binds, when it
//! is issued, and what happens when the echo does not match.

use std::cell::RefCell;

use linker::backend::OutputRole;
use target_elements::TransactionForm;

use crate::bytes::{AssetField, TargetTransaction, ValueField};
use crate::construct::{check_weight, construct};
use crate::error::TransactionRefusal;
use crate::request::CompactAshRequest;
use crate::sponsor::{
    SighashProfile, SignerRole, SponsorCapability, SponsorOffer, SponsorSignature,
    SponsorSigningRequest,
};
use crate::synthetic::SyntheticDisclaimer;
use crate::tests::{
    CLOSED_ASSET, RESERVE_ASSET, SPONSOR_CHANGE_PROGRAM, ash_view, candidate_abi, outpoint,
    reviewed_target, shape, sponsor_view, view,
};

/// A sponsor adapter that contributes one input and echoes what it is
/// asked to sign.
struct FixtureSponsor {
    fee: u64,
    change: Option<ValueField>,
    stack: Vec<Vec<u8>>,
    echo_wrong: bool,
    seen: RefCell<Vec<SponsorSigningRequest>>,
}

impl FixtureSponsor {
    fn new(fee: u64, change: Option<ValueField>) -> Self {
        Self {
            fee,
            change,
            stack: vec![vec![0x30, 0x44, 0x01], vec![0x02, 0x0a]],
            echo_wrong: false,
            seen: RefCell::new(Vec::new()),
        }
    }
}

impl SponsorCapability for FixtureSponsor {
    fn offer(&self) -> SponsorOffer {
        SponsorOffer::new([outpoint(0xdd, 2)], self.fee, self.change)
            .expect("the fixture offer names one outpoint")
    }

    fn change_destination(&self) -> Option<(u8, Vec<u8>)> {
        Some((0, SPONSOR_CHANGE_PROGRAM.to_vec()))
    }

    fn sign(&self, request: &SponsorSigningRequest) -> Option<SponsorSignature> {
        self.seen.borrow_mut().push(request.clone());
        let bound = if self.echo_wrong {
            let mut other = request.transaction().to_vec();
            other.push(0x00);
            other
        } else {
            request.transaction().to_vec()
        };
        Some(SponsorSignature::new(bound, self.stack.clone()))
    }
}

#[test]
fn the_sponsorless_form_constructs_to_exact_bytes() {
    let target = reviewed_target();
    let abi = candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let view = view([
        ash_view(&target, first, 120),
        ash_view(&target, second, 180),
    ]);
    let request = CompactAshRequest::new([first, second], false).expect("a two-input request");

    let built =
        construct(&target, &abi, &request, &view, None).expect("the sponsorless form constructs");

    // The shape, the form, and the version the ABI states for it.
    assert_eq!(built.report().shape(), shape(2, 0, false));
    assert_eq!(built.report().form(), TransactionForm::Sponsorless);
    assert_eq!(built.transaction().version(), 3);

    // The consolidated amount, and the single successor output.
    assert_eq!(built.report().successor_amount(), 300);
    assert_eq!(built.transaction().outputs().len(), 1);
    let successor = &built.transaction().outputs()[0];
    assert_eq!(
        successor.asset(),
        AssetField::Explicit(crate::bytes::AssetId::from_internal(CLOSED_ASSET))
    );
    assert_eq!(successor.value(), ValueField::Explicit(300));
    assert_eq!(
        successor.program(),
        abi.pin()
            .output_script(&target)
            .expect("the pinned script")
            .as_slice()
    );

    // No fee output at all: the reviewed zero-fee representation is an
    // absent output, and the target refuses the zero-valued spelling.
    assert!(
        built
            .transaction()
            .outputs()
            .iter()
            .all(|out| !out.is_fee())
    );

    // Canonical input order: the family is sorted ascending by
    // outpoint, so 0xaa precedes 0xbb whatever order the request named
    // them in.
    let inputs = built.transaction().inputs();
    assert_eq!(inputs.len(), 2);
    assert_eq!(inputs[0].outpoint(), first);
    assert_eq!(inputs[1].outpoint(), second);
    assert!(inputs.iter().all(|input| input.sequence() == 0xffff_ffff));
    assert!(inputs.iter().all(|input| input.script_sig().is_empty()));

    // The bytes decode back to the same transaction, and the encoding
    // is stable.
    let bytes = built.bytes();
    assert_eq!(
        TargetTransaction::decode(&bytes).expect("the constructed bytes decode"),
        *built.transaction()
    );
    assert_eq!(
        construct(&target, &abi, &request, &view, None)
            .expect("the same construction repeats")
            .bytes(),
        bytes
    );
}

#[test]
fn the_ash_witness_is_the_leaf_program_then_the_control_block() {
    let target = reviewed_target();
    let abi = candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let view = view([
        ash_view(&target, first, 120),
        ash_view(&target, second, 180),
    ]);
    let request = CompactAshRequest::new([first, second], false).expect("a two-input request");
    let built = construct(&target, &abi, &request, &view, None).expect("the construction");

    let shape_abi = abi.shape(shape(2, 0, false)).expect("the shape's ABI");
    for (position, witness) in built.transaction().witnesses().iter().enumerate() {
        let role = if position == 0 {
            linker::backend::InputRole::Coordinator
        } else {
            linker::backend::InputRole::Member
        };
        let leaf = *shape_abi.tapleaf().get(&role).expect("the role's leaf");
        assert_eq!(witness.stack().len(), 2);
        assert_eq!(
            witness.stack()[0],
            *abi.tree()
                .leaf_programs()
                .get(&leaf)
                .expect("the leaf's program")
        );
        assert_eq!(
            witness.stack()[1],
            abi.tree()
                .control_block(leaf, abi.pin())
                .expect("the leaf's control block")
        );
    }

    // The anchor spends the coordinator leaf and the rest spend the
    // member leaf, so the two witnesses differ.
    assert_ne!(
        built.transaction().witnesses()[0],
        built.transaction().witnesses()[1]
    );
}

/// The sequence field is fixed here or nowhere.
///
/// No emitted program inspects a sequence, no signature covers an ASH
/// input, and a sequence one below final engages neither a relative
/// timelock nor replaceability — so the target accepts whatever it is
/// given and cannot be the thing that pins this. The constructor is,
/// and a request carries no field to argue with it. That makes this
/// test the whole enforcement of `SequenceConstraint`, not a
/// restatement of something checked downstream.
#[test]
fn every_constructed_input_carries_the_abi_sequence() {
    let target = reviewed_target();
    let abi = candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let sponsor_input = outpoint(0xdd, 2);

    // The value the ABI names, checked against the target's own final
    // sequence rather than against a literal repeated from the ABI.
    assert_eq!(
        abi.sequence().sequence(),
        0xffff_ffff,
        "the ABI's sequence is the final one"
    );

    let sponsorless = view([
        ash_view(&target, first, 120),
        ash_view(&target, second, 180),
    ]);
    let request = CompactAshRequest::new([first, second], false).expect("a two-input request");
    let built = construct(&target, &abi, &request, &sponsorless, None).expect("the construction");
    for input in built.transaction().inputs() {
        assert_eq!(
            input.sequence(),
            abi.sequence().sequence(),
            "an ASH input left the constructor with another sequence"
        );
    }

    // The sponsor input is written by the same rule, so a sponsored
    // form cannot be the one that leaks a different sequence.
    let sponsored_view = view([
        ash_view(&target, first, 120),
        ash_view(&target, second, 180),
        sponsor_view(sponsor_input, 1_000),
    ]);
    let sponsored = CompactAshRequest::new([first, second], true).expect("a sponsored request");
    let sponsor = FixtureSponsor::new(500, Some(ValueField::Explicit(490)));
    let built = construct(&target, &abi, &sponsored, &sponsored_view, Some(&sponsor))
        .expect("the sponsored form constructs");
    assert_eq!(built.transaction().inputs().len(), 3);
    for input in built.transaction().inputs() {
        assert_eq!(
            input.sequence(),
            abi.sequence().sequence(),
            "an input of the sponsored form left the constructor with another sequence"
        );
    }
}

#[test]
fn the_sponsored_form_carries_a_suffix_a_change_role_and_a_fee_role() {
    let target = reviewed_target();
    let abi = candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let sponsor_input = outpoint(0xdd, 2);
    let view = view([
        ash_view(&target, first, 120),
        ash_view(&target, second, 180),
        sponsor_view(sponsor_input, 1_000),
    ]);
    let request = CompactAshRequest::new([first, second], true).expect("a sponsored request");
    let sponsor = FixtureSponsor::new(500, Some(ValueField::Explicit(490)));

    let built = construct(&target, &abi, &request, &view, Some(&sponsor))
        .expect("the sponsored form constructs");

    assert_eq!(built.report().shape(), shape(2, 1, true));
    assert_eq!(built.report().form(), TransactionForm::Sponsored);
    assert_eq!(built.transaction().version(), 2);

    // The sponsor sits after the whole ASH family, and nowhere else.
    let inputs = built.transaction().inputs();
    assert_eq!(inputs.len(), 3);
    assert_eq!(inputs[2].outpoint(), sponsor_input);
    assert_eq!(built.report().roles().ash_inputs(), &[0, 1]);
    assert_eq!(built.report().roles().sponsor_inputs(), &[2]);

    // Three outputs: successor, change, and the fee role last.
    let outputs = built.transaction().outputs();
    assert_eq!(outputs.len(), 3);
    assert_eq!(outputs[0].value(), ValueField::Explicit(300));
    assert_eq!(
        outputs[1].asset(),
        AssetField::Explicit(crate::bytes::AssetId::from_internal(RESERVE_ASSET))
    );
    assert_eq!(outputs[1].value(), ValueField::Explicit(490));
    assert_eq!(outputs[2].value(), ValueField::Explicit(500));
    assert!(outputs[2].is_fee());
    assert_eq!(outputs[2].program(), [] as [u8; 0]);
    assert_eq!(
        built.report().roles().outputs().get(&2),
        Some(&OutputRole::TargetFee)
    );

    // The sponsor's witness is the stack it returned, unchanged.
    assert_eq!(built.transaction().witnesses()[2].stack().len(), 2);

    let bytes = built.bytes();
    assert_eq!(
        TargetTransaction::decode(&bytes).expect("the constructed bytes decode"),
        *built.transaction()
    );
}

#[test]
fn a_signing_request_binds_the_finalized_transaction_and_its_outputs() {
    let target = reviewed_target();
    let abi = candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let sponsor_input = outpoint(0xdd, 2);
    let view = view([
        ash_view(&target, first, 120),
        ash_view(&target, second, 180),
        sponsor_view(sponsor_input, 1_000),
    ]);
    let request = CompactAshRequest::new([first, second], true).expect("a sponsored request");
    let sponsor = FixtureSponsor::new(500, Some(ValueField::Explicit(490)));

    let built = construct(&target, &abi, &request, &view, Some(&sponsor)).expect("a construction");

    let seen = sponsor.seen.borrow();
    assert_eq!(seen.len(), 1);
    let signing = &seen[0];
    assert_eq!(signing.input(), 2);
    assert_eq!(signing.role(), SignerRole::SponsorSuffixMember);
    assert_eq!(signing.profile(), SighashProfile::AllInputsAllOutputs);

    // Every output was already final when the request was issued: the
    // protected set the request carries is the output set the finished
    // transaction has.
    assert_eq!(signing.protected(), built.transaction().outputs());

    // And the bytes the request bound differ from the finished ones
    // only in the sponsor's own witness, which is what a signature over
    // them is allowed not to cover.
    let bound = TargetTransaction::decode(signing.transaction()).expect("the bound bytes decode");
    assert_eq!(bound.outputs(), built.transaction().outputs());
    assert_eq!(bound.inputs(), built.transaction().inputs());
}

#[test]
fn a_signature_bound_to_other_bytes_is_refused() {
    let target = reviewed_target();
    let abi = candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let sponsor_input = outpoint(0xdd, 2);
    let view = view([
        ash_view(&target, first, 120),
        ash_view(&target, second, 180),
        sponsor_view(sponsor_input, 1_000),
    ]);
    let request = CompactAshRequest::new([first, second], true).expect("a sponsored request");
    let mut sponsor = FixtureSponsor::new(500, Some(ValueField::Explicit(490)));
    sponsor.echo_wrong = true;

    assert_eq!(
        construct(&target, &abi, &request, &view, Some(&sponsor)),
        Err(TransactionRefusal::SponsorSignatureBindingMismatch(
            sponsor_input
        ))
    );
}

#[test]
fn a_witness_stack_the_admitted_class_does_not_take_is_refused() {
    let target = reviewed_target();
    let abi = candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let sponsor_input = outpoint(0xdd, 2);
    let view = view([
        ash_view(&target, first, 120),
        ash_view(&target, second, 180),
        sponsor_view(sponsor_input, 1_000),
    ]);
    let request = CompactAshRequest::new([first, second], true).expect("a sponsored request");
    let mut sponsor = FixtureSponsor::new(500, Some(ValueField::Explicit(490)));
    sponsor.stack = vec![vec![0x30]];

    assert_eq!(
        construct(&target, &abi, &request, &view, Some(&sponsor)),
        Err(TransactionRefusal::SponsorWitnessShapeRefused {
            outpoint: sponsor_input,
            offered: 1,
            expected: 2,
        })
    );
}

#[test]
fn a_known_zero_change_is_omitted_rather_than_emitted() {
    // The target refuses a spendable zero-valued output, so emitting
    // one would produce a transaction consensus rejects. The builder
    // therefore selects the shape with no change role at all.
    let target = reviewed_target();
    let abi = candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let sponsor_input = outpoint(0xdd, 2);
    let view = view([
        ash_view(&target, first, 120),
        ash_view(&target, second, 180),
        sponsor_view(sponsor_input, 500),
    ]);
    let request = CompactAshRequest::new([first, second], true).expect("a sponsored request");
    let sponsor = FixtureSponsor::new(500, Some(ValueField::Explicit(0)));

    let built = construct(&target, &abi, &request, &view, Some(&sponsor))
        .expect("a sponsored construction with no change");

    assert_eq!(built.report().shape(), shape(2, 1, false));
    assert_eq!(built.transaction().outputs().len(), 2);
    assert!(
        built
            .transaction()
            .outputs()
            .iter()
            .all(|output| output.value() != ValueField::Explicit(0) || output.is_fee())
    );
}

#[test]
fn a_confidential_residual_is_carried_rather_than_compared_with_zero() {
    // A sponsor whose change value is a commitment gets a change
    // output, because comparing that value with zero is exactly what
    // the erasure law forbids. The shape leakage is real and recorded:
    // the presence of the output reveals that the builder was not told
    // the residual was zero.
    let target = reviewed_target();
    let abi = candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let sponsor_input = outpoint(0xdd, 2);
    let view = view([
        ash_view(&target, first, 120),
        ash_view(&target, second, 180),
        sponsor_view(sponsor_input, 1_000),
    ]);
    let request = CompactAshRequest::new([first, second], true).expect("a sponsored request");
    let sponsor = FixtureSponsor::new(500, Some(ValueField::Commitment([0x08; 33])));

    let built = construct(&target, &abi, &request, &view, Some(&sponsor))
        .expect("a sponsored construction with a committed residual");

    assert_eq!(built.report().shape(), shape(2, 1, true));
    assert_eq!(
        built.transaction().outputs()[1].value(),
        ValueField::Commitment([0x08; 33])
    );
    // The asset stays explicit even so: the reviewed profile forces it,
    // and sponsor asset opacity does not exist under this relation.
    assert!(matches!(
        built.transaction().outputs()[1].asset(),
        AssetField::Explicit(_)
    ));
}

#[test]
fn the_report_carries_the_three_settled_dimensions_and_no_sponsor_value() {
    let target = reviewed_target();
    let abi = candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let view = view([
        ash_view(&target, first, 120),
        ash_view(&target, second, 180),
    ]);
    let request = CompactAshRequest::new([first, second], false).expect("a two-input request");
    let built = construct(&target, &abi, &request, &view, None).expect("a construction");

    let resources = built.report().resources();
    assert_eq!(
        resources.witness_bytes(),
        built.transaction().witness_bytes()
    );
    assert_eq!(resources.weight(), built.transaction().weight());
    assert_eq!(resources.virtual_size(), built.transaction().virtual_size());
    assert_eq!(resources.package_transactions(), 2);
    // The sponsorless form is the parent of the topology-restricted
    // package, so it takes the parent ceiling.
    assert_eq!(resources.package_virtual_size(), 10_000);
    assert_eq!(resources.dimensions().len(), 3);

    // The weight identity, recomputed the target's own way rather than
    // read back from the same accessor.
    let stripped = built.transaction().encode_without_witness().len() as u64;
    let total = built.bytes().len() as u64;
    assert_eq!(resources.weight(), stripped * 3 + total);
    assert_eq!(resources.witness_bytes(), total - stripped);
}

#[test]
fn a_synthetic_instance_makes_every_report_carry_all_five_disclaimers() {
    let target = reviewed_target();
    let abi = candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let view = view([
        ash_view(&target, first, 120),
        ash_view(&target, second, 180),
    ]);
    let request = CompactAshRequest::new([first, second], false).expect("a two-input request");
    let built = construct(&target, &abi, &request, &view, None).expect("a construction");

    let disclaimers = built.report().disclaimers();
    assert_eq!(disclaimers.len(), SyntheticDisclaimer::ALL.len());
    for disclaimer in SyntheticDisclaimer::ALL {
        assert!(disclaimers.contains(disclaimer));
    }
}

#[test]
fn a_duplicate_or_overlapping_outpoint_is_refused_before_sorting() {
    let first = outpoint(0xaa, 0);
    assert_eq!(
        CompactAshRequest::new([first, first], false),
        Err(TransactionRefusal::DuplicateOutpoint(first))
    );

    // The overlap between the two regions, which no single set can rule
    // out.
    let target = reviewed_target();
    let abi = candidate_abi();
    let sponsor_input = outpoint(0xdd, 2);
    let view = view([
        ash_view(&target, first, 120),
        ash_view(&target, sponsor_input, 180),
    ]);
    let request =
        CompactAshRequest::new([first, sponsor_input], true).expect("a two-input request");
    let sponsor = FixtureSponsor::new(500, None);
    assert_eq!(
        construct(&target, &abi, &request, &view, Some(&sponsor)),
        Err(TransactionRefusal::OverlappingOutpoint(sponsor_input))
    );
}

#[test]
fn an_ash_input_carrying_another_program_or_asset_is_refused() {
    let target = reviewed_target();
    let abi = candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let request = CompactAshRequest::new([first, second], false).expect("a two-input request");

    // A foreign program: the pin is what says which instance this ABI
    // builds against, so an input carrying anything else is not a
    // member of this family.
    let foreign_program = view([
        ash_view(&target, first, 120),
        crate::view::PublicOutputView::new(
            second,
            AssetField::Explicit(crate::bytes::AssetId::from_internal(CLOSED_ASSET)),
            ValueField::Explicit(180),
            vec![0x51, 0x20, 0x00],
        ),
    ]);
    assert_eq!(
        construct(&target, &abi, &request, &foreign_program, None),
        Err(TransactionRefusal::AshInputCarriesForeignProgram(second))
    );

    // A foreign asset.
    let foreign_asset = view([
        ash_view(&target, first, 120),
        crate::view::PublicOutputView::new(
            second,
            AssetField::Explicit(crate::bytes::AssetId::from_internal(RESERVE_ASSET)),
            ValueField::Explicit(180),
            abi.pin().output_script(&target).expect("the pinned script"),
        ),
    ]);
    assert_eq!(
        construct(&target, &abi, &request, &foreign_asset, None),
        Err(TransactionRefusal::AshInputCarriesForeignAsset(second))
    );
}

#[test]
fn a_confidential_ash_amount_is_refused_as_a_construction_failure() {
    let target = reviewed_target();
    let abi = candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let request = CompactAshRequest::new([first, second], false).expect("a two-input request");
    let committed = view([
        ash_view(&target, first, 120),
        crate::view::PublicOutputView::new(
            second,
            AssetField::Explicit(crate::bytes::AssetId::from_internal(CLOSED_ASSET)),
            ValueField::Commitment([0x08; 33]),
            abi.pin().output_script(&target).expect("the pinned script"),
        ),
    ]);
    assert_eq!(
        construct(&target, &abi, &request, &committed, None),
        Err(TransactionRefusal::AshInputAmountNotExplicit(second))
    );
}

#[test]
fn a_family_size_no_shape_admits_is_refused() {
    let target = reviewed_target();
    let abi = candidate_abi();
    let only = outpoint(0xaa, 0);
    let view = view([ash_view(&target, only, 120)]);
    let request = CompactAshRequest::new([only], false).expect("a one-input request");

    assert_eq!(
        construct(&target, &abi, &request, &view, None),
        Err(TransactionRefusal::UnsupportedShape {
            ash_inputs: 1,
            sponsor_inputs: 0,
            sponsor_change: false,
        })
    );
}

#[test]
fn a_request_and_a_capability_must_agree_about_sponsorship() {
    // Neither mismatch is downgraded. A caller asking for a sponsored
    // transaction gets one or gets a refusal, because the two forms
    // have different relay verdicts and different versions and building
    // the other one would answer a question nobody asked.
    let target = reviewed_target();
    let abi = candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let view = view([
        ash_view(&target, first, 120),
        ash_view(&target, second, 180),
    ]);

    let asked = CompactAshRequest::new([first, second], true).expect("a sponsored request");
    assert_eq!(
        construct(&target, &abi, &asked, &view, None),
        Err(TransactionRefusal::SponsorRequestedWithoutCapability)
    );

    let unasked = CompactAshRequest::new([first, second], false).expect("a sponsorless request");
    let sponsor = FixtureSponsor::new(500, None);
    assert_eq!(
        construct(&target, &abi, &unasked, &view, Some(&sponsor)),
        Err(TransactionRefusal::SponsorCapabilityWithoutRequest)
    );
}

#[test]
fn a_transaction_above_the_reviewed_weight_bound_is_refused() {
    // The consensus maximum is four million weight units, so a
    // transaction reaching it needs about a megabyte of non-witness
    // bytes. Built directly rather than through the pipeline, because
    // no admitted compact-ASH shape can reach it and a bound whose
    // failing branch could only be reasoned about is a bound nobody has
    // checked.
    let target = reviewed_target();
    let oversized = TargetTransaction::new(
        3,
        vec![crate::bytes::TargetInput::new(
            outpoint(0xaa, 0),
            0xffff_ffff,
        )],
        vec![crate::bytes::TargetOutput::new(
            AssetField::Explicit(crate::bytes::AssetId::from_internal(CLOSED_ASSET)),
            ValueField::Explicit(1),
            crate::bytes::NonceField::Null,
            vec![0x00; 1_000_000],
        )],
        0,
        vec![crate::bytes::InputWitness::default()],
    )
    .expect("an oversized transaction is still well formed");

    assert!(oversized.weight() > 4_000_000);
    assert_eq!(
        crate::construct::check_weight(&target, &oversized),
        Err(TransactionRefusal::ResourceBoundExceeded {
            dimension: target_elements::ResourceDimension::TransactionWeight,
            reached: oversized.weight(),
            bound: 4_000_000,
        })
    );

    // And an ordinary construction is nowhere near it.
    let abi = candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let view = view([
        ash_view(&target, first, 120),
        ash_view(&target, second, 180),
    ]);
    let request = CompactAshRequest::new([first, second], false).expect("a two-input request");
    let built = construct(&target, &abi, &request, &view, None).expect("a construction");
    assert!(check_weight(&target, built.transaction()).is_ok());
}

#[test]
fn a_sponsor_spending_more_than_it_contributes_is_refused() {
    // A construction check over the sponsor's own explicit values,
    // which §1.6 permits the constructor to use and forbids the
    // protocol relation to depend on. The sponsor here declares a fee
    // and a change that together exceed the input it brought.
    let target = reviewed_target();
    let abi = candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let sponsor_input = outpoint(0xdd, 2);
    let view = view([
        ash_view(&target, first, 120),
        ash_view(&target, second, 180),
        sponsor_view(sponsor_input, 600),
    ]);
    let request = CompactAshRequest::new([first, second], true).expect("a sponsored request");
    let sponsor = FixtureSponsor::new(500, Some(ValueField::Explicit(400)));

    assert_eq!(
        construct(&target, &abi, &request, &view, Some(&sponsor)),
        Err(TransactionRefusal::SponsorValueDoesNotCoverFee)
    );
}

#[test]
fn a_committed_sponsor_value_is_never_compared_at_all() {
    // The same arithmetic that refused above cannot even be attempted
    // here, because the sponsor's input value is a commitment. The
    // construction succeeds, and that is the erasure law holding rather
    // than a check being skipped.
    let target = reviewed_target();
    let abi = candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let sponsor_input = outpoint(0xdd, 2);
    let mut program = vec![0x00, 0x14];
    program.extend_from_slice(&[0xd0; 20]);
    let view = view([
        ash_view(&target, first, 120),
        ash_view(&target, second, 180),
        crate::view::PublicOutputView::new(
            sponsor_input,
            AssetField::Explicit(crate::bytes::AssetId::from_internal(RESERVE_ASSET)),
            ValueField::Commitment([0x08; 33]),
            program,
        ),
    ]);
    let request = CompactAshRequest::new([first, second], true).expect("a sponsored request");
    let sponsor = FixtureSponsor::new(500, Some(ValueField::Explicit(400)));

    assert!(construct(&target, &abi, &request, &view, Some(&sponsor)).is_ok());
}

#[test]
fn a_sponsor_input_of_an_unadmitted_program_class_is_refused() {
    let target = reviewed_target();
    let abi = candidate_abi();
    let first = outpoint(0xaa, 0);
    let second = outpoint(0xbb, 1);
    let sponsor_input = outpoint(0xdd, 2);
    let view = view([
        ash_view(&target, first, 120),
        ash_view(&target, second, 180),
        crate::view::PublicOutputView::new(
            sponsor_input,
            AssetField::Explicit(crate::bytes::AssetId::from_internal(RESERVE_ASSET)),
            ValueField::Explicit(1_000),
            // A bare taproot output rather than the admitted
            // version-zero key hash.
            vec![0x51, 0x20],
        ),
    ]);
    let request = CompactAshRequest::new([first, second], true).expect("a sponsored request");
    let sponsor = FixtureSponsor::new(500, None);

    assert_eq!(
        construct(&target, &abi, &request, &view, Some(&sponsor)),
        Err(TransactionRefusal::SponsorProgramClassNotAdmitted(
            sponsor_input
        ))
    );
}
