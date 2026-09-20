//! Finalization of the maturity announcement: the settled censuses, the
//! exact-byte comparison, and the nine terms the signing request derives
//! from one candidate.
//!
//! # Everything here runs over the demonstration link
//!
//! The bundle, the view, the ABI and the construction are the shared
//! fixtures' — derived, planned, composed, bound and linked through the
//! public builders — so the static subtree a control block is folded
//! against and the output key it must recompute to are a real link's.
//!
//! # Why one test links a second bundle
//!
//! The demonstration deployment's genesis is byte-uniform, so it is its
//! own reversal and a conversion between the printed and internal byte
//! orders is unobservable over it: the request would carry the right
//! value whether the conversion ran, ran backwards, or did not run. The
//! seed test therefore also links a bundle over an identity whose
//! genesis differs from its reversal, which is the only arrangement in
//! which the direction is a fact rather than a coincidence.
//!
//! # What the recomputations are computed from
//!
//! The tapleaf hash, the folded merkle root and the witness program are
//! recomputed here from the control block's own bytes rather than read
//! back from the value that produced them. A comparison whose two sides
//! came from one derivation would agree however wrong that derivation
//! was, which is the whole reason the control block is taken apart
//! byte by byte below.

use linker::{CandidateDeploymentIdentity, CandidateLinkedMaturityBundle};
use realization::Cycle;
use tapscript::{CandidateStateConstructor, StateLeafRole, StateNonceBudget};

use super::reviewed_target;
use super::state_support::{
    FixtureLiveCurve, FixtureLiveCurveRefusingTheOperatorKey, FixtureStateCurve,
    asymmetric_genesis_identity, demonstration_identity, linked_bundle_over, linked_pair,
    state_metadata, statements, validated_view,
};
use crate::bytes::{TargetInput, TargetOutput, TargetTransaction, ValueField};
use crate::error::TransactionRefusal;
use crate::live_request::{RequestedForm, SponsorChangeRequest};
use crate::live_taproot::LiveCurveCapability;
use crate::operator_signing::{OPERATOR_CODESEPARATOR_POSITION, OperatorSigningRefusal};
use crate::script_path_signing::SpentOutputCensusEntry;
use crate::state_abi::derive_maturity_announcement_abi;
use crate::state_construct::construct_maturity_announcement;
use crate::state_finalize::{
    FinalizedMaturityAnnouncement, MaturityFinalizedFact, finalize_maturity_announcement,
};
use crate::state_request::MaturityAnnouncementRequest;
use crate::state_view::{PublicMaturityStateView, ValidatedMaturityStateView};
use crate::taproot::{
    CONTROL_BASE_BYTES, DIGEST_BYTES, TAPROOT_WITNESS_VERSION, branch_hash, leaf_hash,
    witness_program_script,
};

/// The cycle the tests announce.
///
/// The fixture predecessor sits at cycle five under a lead window of two
/// and four, so this is the middle of the three admissible figures.
const ANNOUNCED: Cycle = Cycle::new(8);

/// The sponsorless request every finalization here is built from.
fn sponsorless() -> MaturityAnnouncementRequest {
    MaturityAnnouncementRequest::new(
        ANNOUNCED,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
    )
    .expect("the sponsorless form without change is an admitted pair")
}

/// The validated view one bundle's own application states.
fn validated_over(bundle: &CandidateLinkedMaturityBundle) -> ValidatedMaturityStateView {
    let (nonce, program) = linked_pair(bundle);
    PublicMaturityStateView::new(statements(bundle, state_metadata(), nonce, program))
        .expect("the seven statements name distinct entries")
        .validate(&reviewed_target(), &FixtureStateCurve)
        .expect("the stated pair reproduces the stated program")
}

/// The finalized announcement over one validated view.
fn finalized_over(validated: &ValidatedMaturityStateView) -> FinalizedMaturityAnnouncement {
    let target = reviewed_target();
    let abi = derive_maturity_announcement_abi(&target, validated)
        .expect("the demonstration view derives its ABI");
    let construction = construct_maturity_announcement(
        &target,
        &abi,
        validated,
        &sponsorless(),
        &FixtureStateCurve,
    )
    .expect("the demonstration deployment constructs its announcement");
    finalize_maturity_announcement(construction)
}

/// The finalized announcement every positive reads.
fn demonstration() -> FinalizedMaturityAnnouncement {
    finalized_over(&validated_view())
}

/// The predecessor constructor the link itself retained.
fn predecessor(finalized: &FinalizedMaturityAnnouncement) -> &CandidateStateConstructor {
    finalized
        .construction()
        .validated_view()
        .view()
        .accepted_linked_bundle()
        .instances()[0]
        .constructor()
}

/// One offered transaction differing from the finalized one as stated.
fn offered(
    finalized: &FinalizedMaturityAnnouncement,
    inputs: Vec<TargetInput>,
    outputs: Vec<TargetOutput>,
    version: u32,
    lock_time: u32,
) -> TargetTransaction {
    let witnesses = finalized.protected().witnesses().to_vec();
    let witnesses = vec![witnesses[0].clone(); inputs.len()];
    TargetTransaction::new(version, inputs, outputs, lock_time, witnesses)
        .expect("the offered roles assemble")
}

/// The finalized transaction's own roles, ready to be varied.
fn roles(finalized: &FinalizedMaturityAnnouncement) -> (Vec<TargetInput>, Vec<TargetOutput>) {
    let protected = finalized.protected();
    (protected.inputs().to_vec(), protected.outputs().to_vec())
}

#[test]
fn the_finalized_protected_transaction_is_the_constructions_own() {
    let finalized = demonstration();
    assert_eq!(
        finalized.protected(),
        finalized.construction().transaction()
    );
    assert_eq!(
        finalized.protected_bytes(),
        finalized.construction().bytes()
    );
}

#[test]
fn the_finalized_bytes_survive_the_strict_decoders_round_trip() {
    let finalized = demonstration();
    let decoded = TargetTransaction::decode(finalized.protected_bytes())
        .expect("the finalized bytes are a transaction the strict decoder admits");
    assert_eq!(&decoded, finalized.protected());
    assert_eq!(decoded.encode(), finalized.protected_bytes());
}

#[test]
fn the_output_census_is_the_successor_alone_with_no_sponsor_change_and_no_fee() {
    let finalized = demonstration();
    let census = finalized.outputs();
    assert_eq!(census.outputs().len(), 1);
    assert_eq!(census.outputs(), finalized.protected().outputs());
    assert_eq!(census.successor_position(), 0);
    assert_eq!(census.sponsor_change_position(), None);
    assert_eq!(census.fee_position(), None);
}

#[test]
fn the_spent_output_record_is_the_views_four_statements() {
    let finalized = demonstration();
    let view = finalized.construction().validated_view().view();
    let spent = finalized.spent_output();
    assert_eq!(spent.position(), 0);
    assert_eq!(spent.outpoint(), view.current_state_outpoint());
    assert_eq!(spent.asset(), view.asset());
    assert_eq!(spent.value(), view.value());
    assert_eq!(spent.program(), view.predecessor_program());
    assert_eq!(
        spent.census_entry(),
        SpentOutputCensusEntry::new(
            view.asset(),
            view.value(),
            view.predecessor_program().to_vec()
        )
    );
}

#[test]
fn the_settled_census_is_every_one_of_its_eleven_members() {
    let finalized = demonstration();
    assert_eq!(MaturityFinalizedFact::ALL.len(), 11);
    assert_eq!(
        finalized.settled(),
        MaturityFinalizedFact::ALL.iter().copied().collect()
    );
}

#[test]
fn the_offered_comparison_accepts_the_transaction_that_was_finalized() {
    let finalized = demonstration();
    assert_eq!(finalized.check_offered(finalized.protected()), Ok(()));
}

#[test]
fn an_added_input_is_refused_as_an_extension() {
    let finalized = demonstration();
    let (mut inputs, outputs) = roles(&finalized);
    let repeated = inputs[0].clone();
    inputs.push(repeated);
    let protected = finalized.protected();
    assert_eq!(
        finalized.check_offered(&offered(
            &finalized,
            inputs,
            outputs,
            protected.version(),
            protected.lock_time()
        )),
        Err(TransactionRefusal::InputExtendedAfterSigning {
            finalized: 1,
            offered: 2,
        })
    );
}

#[test]
fn an_added_output_is_refused_at_the_position_beyond_the_census() {
    let finalized = demonstration();
    let (inputs, mut outputs) = roles(&finalized);
    let repeated = outputs[0].clone();
    outputs.push(repeated);
    let protected = finalized.protected();
    assert_eq!(
        finalized.check_offered(&offered(
            &finalized,
            inputs,
            outputs,
            protected.version(),
            protected.lock_time()
        )),
        Err(TransactionRefusal::OutputMutatedAfterSigning { position: 1 })
    );
}

#[test]
fn a_mutated_output_is_refused_at_its_own_position() {
    let finalized = demonstration();
    let (inputs, outputs) = roles(&finalized);
    let moved = TargetOutput::new(
        outputs[0].asset(),
        ValueField::Explicit(2),
        outputs[0].nonce(),
        outputs[0].program().to_vec(),
    );
    let protected = finalized.protected();
    assert_eq!(
        finalized.check_offered(&offered(
            &finalized,
            inputs,
            vec![moved],
            protected.version(),
            protected.lock_time()
        )),
        Err(TransactionRefusal::OutputMutatedAfterSigning { position: 0 })
    );
}

#[test]
fn a_changed_version_is_refused_by_its_own_region() {
    let finalized = demonstration();
    let (inputs, outputs) = roles(&finalized);
    let protected = finalized.protected();
    let moved = protected.version() + 1;
    assert_eq!(
        finalized.check_offered(&offered(
            &finalized,
            inputs,
            outputs,
            moved,
            protected.lock_time()
        )),
        Err(
            TransactionRefusal::MaturityVersionChangedAfterFinalization {
                finalized: protected.version(),
                offered: moved,
            }
        )
    );
}

#[test]
fn a_changed_lock_time_is_refused_by_its_own_region() {
    let finalized = demonstration();
    let (inputs, outputs) = roles(&finalized);
    let protected = finalized.protected();
    let moved = protected.lock_time() + 1;
    assert_eq!(
        finalized.check_offered(&offered(
            &finalized,
            inputs,
            outputs,
            protected.version(),
            moved
        )),
        Err(
            TransactionRefusal::MaturityLockTimeChangedAfterFinalization {
                finalized: protected.lock_time(),
                offered: moved,
            }
        )
    );
}

#[test]
fn a_changed_input_sequence_is_refused_by_the_exact_byte_close() {
    let finalized = demonstration();
    let (inputs, outputs) = roles(&finalized);
    let protected = finalized.protected();
    let moved = TargetInput::new(inputs[0].outpoint(), inputs[0].sequence() - 1);
    let candidate = offered(
        &finalized,
        vec![moved],
        outputs,
        protected.version(),
        protected.lock_time(),
    );

    // The four region comparisons all pass over this candidate: one
    // input at the same outpoint, the same output, the same version and
    // the same lock time. Only the encoding differs.
    let encoding = candidate.encode_without_witness();
    assert_eq!(encoding.len(), finalized.protected_bytes().len());
    let at = encoding
        .iter()
        .zip(finalized.protected_bytes())
        .position(|(offered, fixed)| offered != fixed)
        .expect("the sequence field moved, so the encodings differ somewhere");
    assert_eq!(
        finalized.check_offered(&candidate),
        Err(TransactionRefusal::MaturityBytesDifferAfterFinalization { at })
    );
}

#[test]
fn the_executing_leaf_is_the_committed_announcement_leaf_of_the_predecessors_tree() {
    let target = reviewed_target();
    let finalized = demonstration();
    let leaf = finalized.executing_leaf(&target);
    let predecessor = predecessor(&finalized);

    assert_eq!(
        leaf.tapleaf_hash(),
        &leaf_hash(leaf.leaf_version(), leaf.leaf_script())
    );
    let recipe = predecessor
        .control_recipe(StateLeafRole::Announcement)
        .expect("the predecessor carries the announcement recipe");
    assert_eq!(leaf.tapleaf_hash(), &recipe.executing_leaf_hash);
    assert_eq!(leaf.leaf_version(), recipe.leaf_version);
    assert_eq!(
        leaf.control_block(),
        recipe
            .control_bytes()
            .expect("the recipe derives its bytes")
            .as_slice()
    );

    // The linked taptree exposes no per-leaf hash or program of its own;
    // what it exposes is the committed static subtree, and that is where
    // the hash the control block authenticates was taken. Read back
    // through the bundle rather than through the constructor, so the two
    // accounts of one tree are compared rather than assumed equal.
    let bundle = finalized
        .construction()
        .validated_view()
        .view()
        .accepted_linked_bundle();
    let committed = bundle
        .taptree()
        .subtree()
        .leaves()
        .iter()
        .find(|entry| entry.leaf.role == StateLeafRole::Announcement)
        .expect("the linked subtree carries the announcement leaf");
    assert_eq!(leaf.tapleaf_hash(), &committed.hash);
    assert_eq!(leaf.leaf_script(), committed.leaf.program.encode(&target));
    assert_eq!(bundle.taptree().merkle_root(), predecessor.merkle_root());
}

#[test]
fn the_control_block_recomputes_to_the_predecessors_output_key_and_program() {
    let target = reviewed_target();
    let finalized = demonstration();
    let leaf = finalized.executing_leaf(&target);
    let predecessor = predecessor(&finalized);
    let block = leaf.control_block();

    let internal_key = &block[1..CONTROL_BASE_BYTES];
    let mut root = *leaf.tapleaf_hash();
    for sibling in block[CONTROL_BASE_BYTES..].as_chunks::<DIGEST_BYTES>().0 {
        root = branch_hash(root, *sibling);
    }

    // The metadata leaf is the outermost sibling, because the committed
    // root is the branch of that leaf with the static subtree's root.
    let bundle = finalized
        .construction()
        .validated_view()
        .view()
        .accepted_linked_bundle();
    assert_eq!(
        &block[block.len() - DIGEST_BYTES..],
        bundle.taptree().metadata_hash().as_slice()
    );
    assert_eq!(&root, predecessor.merkle_root());

    let output_key = FixtureLiveCurve
        .output_key(internal_key, &root)
        .expect("the fixture curve determines the output key");
    assert_eq!(output_key.key(), predecessor.output_key());
    assert_eq!(output_key.parity().bit(), block[0] & 1);

    // The predecessor program the view states is the witness program
    // over exactly that key, which is the equality the freeze's own leaf
    // commitment check runs.
    assert_eq!(
        witness_program_script(&target, TAPROOT_WITNESS_VERSION, output_key.key())
            .expect("the reviewed grammar builds a version-one program"),
        finalized.spent_output().program()
    );
}

#[test]
fn the_signing_request_derives_the_nine_census_terms_from_the_one_candidate() {
    let target = reviewed_target();
    let finalized = demonstration();
    let leaf = finalized.executing_leaf(&target);
    let request = finalized
        .signing_request(&target, &FixtureLiveCurve)
        .expect("the demonstration candidate freezes");
    let census = request.census();
    let signing = &census.signing_inputs()[0];

    assert_eq!(census.candidate(), finalized.protected());
    assert_eq!(census.protected_bytes(), finalized.protected_bytes());
    assert_eq!(request.frozen_bytes(), finalized.protected_bytes());
    assert_eq!(census.spent_outputs().len(), 1);
    assert_eq!(
        census.spent_outputs()[0],
        finalized.spent_output().census_entry()
    );
    assert_eq!(request.input_index(), 0);
    assert_eq!(signing.input_index(), 0);
    assert_eq!(
        signing.tapleaf_hash(),
        &leaf_hash(leaf.leaf_version(), leaf.leaf_script())
    );
    assert_eq!(signing.leaf_version(), leaf.leaf_version());
    assert_eq!(
        signing.codeseparator_position(),
        OPERATOR_CODESEPARATOR_POSITION
    );
    assert_eq!(
        request.predecessor_outpoint(),
        Some(finalized.spent_output().outpoint())
    );
    assert_eq!(census.output_witnesses().len(), 1);
}

#[test]
fn the_request_seed_is_the_bindings_genesis_in_the_messages_byte_order() {
    let target = reviewed_target();
    let asymmetric = asymmetric_genesis_identity();
    let mut reversed = *asymmetric.genesis_id();
    reversed.reverse();
    assert_ne!(&reversed, asymmetric.genesis_id());

    for identity in [demonstration_identity(), asymmetric] {
        let bundle = linked_bundle_over(
            identity.clone(),
            StateNonceBudget::default(),
            state_metadata(),
        );
        let finalized = finalized_over(&validated_over(&bundle));
        let request = finalized
            .signing_request(&target, &FixtureLiveCurve)
            .expect("the candidate freezes over its own deployment");
        let mut expected = *identity.genesis_id();
        expected.reverse();
        assert_eq!(request.census().genesis_block_hash(), &expected);
        assert_eq!(
            request.binding().deployment(),
            &CandidateDeploymentIdentity::new(*identity.network_id(), *identity.genesis_id())
                .expect("the fixture identity is nonzero")
        );
    }
}

#[test]
fn the_requests_message_pair_names_two_distinct_candidates() {
    let target = reviewed_target();
    let finalized = demonstration();
    let request = finalized
        .signing_request(&target, &FixtureLiveCurve)
        .expect("the demonstration candidate freezes");
    assert!(request.message().candidates_are_distinct());
}

#[test]
fn the_request_publishes_no_opening_blinder_key_or_verdict() {
    let target = reviewed_target();
    let finalized = demonstration();
    let request = finalized
        .signing_request(&target, &FixtureLiveCurve)
        .expect("the demonstration candidate freezes");

    // The whole public surface, named: a candidate, its bytes, the one
    // input's index and outpoint, the validated census, a recomputed
    // diagnostic message, the borrowed deployment commitment and the
    // record that the supplied capability was consulted. No accessor
    // returns an opening, a blinder, a private key or a claim about
    // what a target would decide, because there is none to return.
    assert_eq!(request.candidate(), finalized.protected());
    assert_eq!(request.frozen_bytes(), finalized.protected_bytes());
    assert_eq!(request.input_index(), 0);
    assert_eq!(
        request.predecessor_outpoint(),
        Some(finalized.spent_output().outpoint())
    );
    assert_eq!(request.census().signing_inputs().len(), 1);
    let message = request.message();
    assert_ne!(message.with_vector_grown(), message.with_vector_empty());
    assert_eq!(
        request.binding().key().bytes(),
        finalized
            .construction()
            .validated_view()
            .view()
            .operator_public_identity()
            .bytes()
    );
    assert!(request.curve_validity_discharged());
}

#[test]
fn a_curve_refusing_the_committed_operator_key_refuses_the_signing_request() {
    let target = reviewed_target();
    let finalized = demonstration();
    let key = finalized
        .construction()
        .validated_view()
        .view()
        .operator_public_identity()
        .bytes()
        .to_vec();
    assert_eq!(
        finalized.signing_request(&target, &FixtureLiveCurveRefusingTheOperatorKey),
        Err(
            TransactionRefusal::MaturityAnnouncementSigningRequestRefused {
                refusal: Box::new(OperatorSigningRefusal::OperatorKeyIsNotACurvePoint { key }),
            }
        )
    );
}
