//! Signing the maturity announcement: the three states, the twelve
//! protected regions, the witness the ABI's records schedule, and
//! §13.4's seventeen faults.
//!
//! # Everything runs over the demonstration link
//!
//! The bundle, the view, the ABI, the construction and the finalization
//! are the shared fixtures' — derived, planned, composed, bound and
//! linked through the public builders — so the static subtree a control
//! block folds against, the output key a program carries and the parity
//! a witness states are a real link's rather than a literal's.
//!
//! # What the fixture signer is, and is not
//!
//! The signer and the verifier are one deterministic function and a
//! double that accepts exactly its output. Neither is Schnorr, neither
//! is cryptography, and the operator key they run over is the fixture's
//! committed public fill. What they establish is the transaction
//! layer's side of §13.3: that a well-formed response is bound to the
//! frozen bytes, the frozen input, the committed operator and the
//! selected profile before any witness exists, and that a response
//! failing any of those bindings leaves no accepted artifact behind.
//!
//! # Why the expected witness items are recomputed
//!
//! Each of the seven items is recomputed from the source the ABI names
//! for it — a constructor's parity, a constructor's nonce, the request's
//! cycle, the bundle's subtree root, the codec over the view's metadata,
//! the retained predecessor's parity, the signer's output — rather than
//! read back out of the stack that was built from them. A comparison
//! whose two sides came from one derivation would agree however wrong
//! that derivation was.

use linker::live_backend::{OperatorKey, OperatorProfileDisposition};
use linker::{CandidateDeploymentIdentity, OperatorDeploymentBinding};
use realization::{
    Cycle, STATE_METADATA_BYTES, STATE_METADATA_VARIABLE_RANGE, StateRepresentationNonce,
    decode_state_metadata, encode_state_metadata, rebuild_state_metadata,
    state_metadata_variable_region,
};
use tapscript::{StateProgramWitness, operator_key_encoding_closure};
use target_elements::{EncodingClass, TargetContractVersion};

use super::reviewed_target;
use super::state_support::{
    FIXTURE_VERIFIER, FixtureLiveCurve, FixtureScriptPathVerifier, FixtureStateCurve,
    asymmetric_genesis_identity, fixture_sign, validated_view, variable_validated_view,
};
use crate::bytes::{
    InputWitness, NonceField, OutputWitness, TargetInput, TargetOutput, TargetTransaction,
};
use crate::error::TransactionRefusal;
use crate::live_request::{RequestedForm, SponsorChangeRequest};
use crate::operator_right::{
    ConstructionRight, NonEquivocationEvent, OperatorRightOutcome, OperatorRightRegistry,
    RightRefusal,
};
use crate::operator_signing::{
    OPERATOR_SIGHASH_TYPE_BYTE, OperatorEvidenceStanding, OperatorSigningInput,
    OperatorSigningRefusal, OperatorSigningRequest, OperatorSigningResponse,
};
use crate::script_path_signing::LiveDeployment;
use crate::state_abi::{MaturityWitnessRole, derive_maturity_announcement_abi};
use crate::state_construct::construct_maturity_announcement;
use crate::state_finalize::{FinalizedMaturityAnnouncement, finalize_maturity_announcement};
use crate::state_request::MaturityAnnouncementRequest;
use crate::state_signing::{
    EVEN_OUTPUT_KEY_PREFIX, MaturityAuthorizationFailure, MaturityProtectedRegion,
    MaturitySubmissionStatus, ODD_OUTPUT_KEY_PREFIX, OperatorAuthorizedMaturityAnnouncement,
    OperatorSigningStarted, SubmitReadyMaturityAnnouncement, checked_witness_item_width,
};
use crate::taproot::leaf_hash;

/// The cycle every announcement here is built for.
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

/// The finalized announcement every test below starts from.
fn demonstration() -> FinalizedMaturityAnnouncement {
    let target = reviewed_target();
    let validated = validated_view();
    let abi = derive_maturity_announcement_abi(&target, &validated)
        .expect("the demonstration view derives its ABI");
    let construction = construct_maturity_announcement(
        &target,
        &abi,
        &validated,
        &sponsorless(),
        &FixtureStateCurve,
    )
    .expect("the demonstration deployment constructs its announcement");
    finalize_maturity_announcement(construction)
}

fn variable_demonstration() -> FinalizedMaturityAnnouncement {
    let target = reviewed_target();
    let validated = variable_validated_view();
    let abi = derive_maturity_announcement_abi(&target, &validated)
        .expect("the variable view derives an ABI");
    let construction = construct_maturity_announcement(
        &target,
        &abi,
        &validated,
        &sponsorless(),
        &FixtureStateCurve,
    )
    .expect("the variable deployment constructs an announcement");
    finalize_maturity_announcement(construction)
}

#[test]
fn variable_loader_carries_the_region_and_strict_rebuild_decodes_the_input() {
    let finalized = variable_demonstration();
    let ready = submit_ready_over(&finalized);
    let stack = ready.witness().stack();
    assert_eq!(stack.len(), 9);
    assert_eq!(
        stack.iter().take(7).map(Vec::len).collect::<Vec<_>>(),
        vec![1, 4, 8, 32, 53, 1, 64]
    );
    let view = finalized.construction().validated_view().view();
    let encoded = encode_state_metadata(
        &view.predecessor_metadata(),
        view.predecessor_representation_nonce(),
    );
    let canonical: [u8; STATE_METADATA_BYTES] =
        encoded.try_into().expect("encoder fixes the width");
    let variable = state_metadata_variable_region(&canonical);
    assert_eq!(stack[4], variable);
    let rebuilt = rebuild_state_metadata(&variable);
    assert_eq!(rebuilt, canonical);
    assert_eq!(
        decode_state_metadata(&rebuilt),
        decode_state_metadata(&canonical)
    );
    let mut wrong = variable;
    wrong[65 - STATE_METADATA_VARIABLE_RANGE.start] = 0xff;
    assert!(decode_state_metadata(&rebuild_state_metadata(&wrong)).is_err());
}

#[test]
fn loader_refuses_an_item_shorter_or_longer_than_the_declared_role() {
    let finalized = variable_demonstration();
    let record = &finalized.construction().abi().witness_roles()[4];
    for populated in [52, 54] {
        assert_eq!(
            checked_witness_item_width(record, vec![0; populated]),
            Err(TransactionRefusal::WitnessItemWidthMismatch {
                role: StateProgramWitness::PredecessorMetadata,
                declared: 53,
                populated,
            })
        );
    }
}

/// The started state over one finalized announcement.
fn started(finalized: &FinalizedMaturityAnnouncement) -> OperatorSigningStarted<'_> {
    OperatorSigningStarted::open(finalized, &reviewed_target(), &FixtureLiveCurve)
        .expect("the demonstration candidate freezes its operator request")
}

/// The deployment's operator binding, as the finalized value carries it.
fn binding(finalized: &FinalizedMaturityAnnouncement) -> &OperatorDeploymentBinding {
    finalized
        .construction()
        .validated_view()
        .view()
        .accepted_linked_bundle()
        .deployment()
        .operator()
}

/// One response's fields, before they are sealed into a response.
#[derive(Clone)]
struct ResponseParts {
    index: u32,
    signature: Vec<u8>,
    type_byte: u8,
    echo: Vec<u8>,
    operator: OperatorKey,
    deployment: CandidateDeploymentIdentity,
    revision: TargetContractVersion,
}

impl ResponseParts {
    fn response(self) -> OperatorSigningResponse {
        OperatorSigningResponse::new(
            self.index,
            self.signature,
            self.type_byte,
            self.echo,
            self.operator,
            self.deployment,
            self.revision,
        )
    }
}

/// The well-formed answer one started state admits.
fn parts(state: &OperatorSigningStarted<'_>) -> ResponseParts {
    let bound = state.request().binding();
    ResponseParts {
        index: state.request().input_index(),
        signature: fixture_sign(
            bound.key().bytes(),
            state.request().message().with_vector_grown(),
        )
        .to_vec(),
        type_byte: OPERATOR_SIGHASH_TYPE_BYTE,
        echo: state.protected_bytes().to_vec(),
        operator: bound.key().clone(),
        deployment: bound.deployment().clone(),
        revision: bound.capability_revision(),
    }
}

/// The signature the fixture signer produces over one candidate.
///
/// Recomputed through a freshly opened state rather than read out of the
/// witness under test, so the comparison has two independent sides. The
/// freeze is deterministic over one finalized value, so the message is
/// the one the state under test was signed against.
fn operator_signature(finalized: &FinalizedMaturityAnnouncement) -> Vec<u8> {
    parts(&started(finalized)).signature
}

/// A right over the started state's own scope and bytes.
fn issue(
    registry: &mut OperatorRightRegistry,
    state: &OperatorSigningStarted<'_>,
) -> ConstructionRight {
    registry
        .issue(state.construction_right_scope(), state.protected_bytes())
        .expect("a fresh registry issues this candidate's scope")
}

/// The authorized state over one finalized announcement.
fn authorized_over(
    finalized: &FinalizedMaturityAnnouncement,
) -> OperatorAuthorizedMaturityAnnouncement<'_> {
    let state = started(finalized);
    let answer = parts(&state);
    let mut registry = OperatorRightRegistry::default();
    let right = issue(&mut registry, &state);
    state
        .authorize(
            &mut registry,
            right,
            [answer.response()],
            &FixtureScriptPathVerifier,
        )
        .expect("one well-formed response authorizes the demonstration candidate")
}

/// The submit-ready state over one finalized announcement.
fn submit_ready_over(
    finalized: &FinalizedMaturityAnnouncement,
) -> SubmitReadyMaturityAnnouncement<'_> {
    authorized_over(finalized)
        .bind_for_submission(&reviewed_target())
        .expect("the authorized candidate binds its one witness")
}

/// The refusal a crafted set of responses draws.
fn refused_over(change: impl FnOnce(&mut Vec<ResponseParts>)) -> TransactionRefusal {
    let finalized = demonstration();
    let state = started(&finalized);
    let mut offered = vec![parts(&state)];
    change(&mut offered);
    let mut registry = OperatorRightRegistry::default();
    let right = issue(&mut registry, &state);
    let failure = state
        .authorize(
            &mut registry,
            right,
            offered.into_iter().map(ResponseParts::response),
            &FixtureScriptPathVerifier,
        )
        .expect_err("a refused authorization produces no accepted artifact");
    failure.refusal
}

/// The boundary's own finding, unwrapped from this crate's vocabulary.
fn operator_fault(refusal: TransactionRefusal) -> OperatorSigningRefusal {
    match refusal {
        TransactionRefusal::MaturityOperatorAuthorizationRefused { refusal } => *refusal,
        other => panic!("a response fault arrives in the operator wrapper: {other:?}"),
    }
}

/// The refusal one crafted response draws, in the boundary's own words.
fn refused_with(change: impl FnOnce(&mut ResponseParts)) -> OperatorSigningRefusal {
    operator_fault(refused_over(|offered| change(&mut offered[0])))
}

/// The region a refused protected operation named.
fn region_of(
    result: Result<OperatorSigningStarted<'_>, TransactionRefusal>,
) -> MaturityProtectedRegion {
    match result.expect_err("a protected mutation returns no state") {
        TransactionRefusal::MaturityMutationAfterSigningStarted { region } => region,
        other => panic!("a protected mutation refuses by its region: {other:?}"),
    }
}

/// The prefix byte one parity is stated with.
fn prefix(odd: bool) -> u8 {
    if odd {
        ODD_OUTPUT_KEY_PREFIX
    } else {
        EVEN_OUTPUT_KEY_PREFIX
    }
}

/// One offered transaction over the finalized input, as stated.
fn offered(
    finalized: &FinalizedMaturityAnnouncement,
    outputs: Vec<TargetOutput>,
    inputs: usize,
) -> TargetTransaction {
    let protected = finalized.protected();
    let one = protected
        .inputs()
        .first()
        .expect("the announcement spends one input")
        .clone();
    let census = vec![one; inputs];
    let witnesses = vec![InputWitness::default(); inputs];
    TargetTransaction::new(
        protected.version(),
        census,
        outputs,
        protected.lock_time(),
        witnesses,
    )
    .expect("the offered census is one witness per input")
}

/// The successor output the finalized form fixed.
fn successor(finalized: &FinalizedMaturityAnnouncement) -> TargetOutput {
    finalized
        .outputs()
        .outputs()
        .first()
        .expect("the announcement creates one output")
        .clone()
}

#[test]
fn the_chain_reaches_a_submit_ready_candidate_over_the_demonstration_link() {
    let finalized = demonstration();
    let ready = submit_ready_over(&finalized);

    // Submit-ready is a state of this candidate and not a promotion of
    // it: the status is the one variant, the protected bytes are the
    // finalized ones unchanged, and the finalized value is still
    // readable through it.
    assert_eq!(ready.status(), MaturitySubmissionStatus::SubmitReady);
    assert_eq!(ready.protected_bytes(), finalized.protected_bytes());
    assert_eq!(
        ready.finalized().protected_bytes(),
        finalized.protected_bytes()
    );
    assert_eq!(
        ready.standing(),
        &OperatorEvidenceStanding::InProcessVerified {
            verifier: FIXTURE_VERIFIER.to_owned(),
        }
    );

    // The one witness is the only thing that moved. Every other region
    // of the candidate is the finalized one, field for field.
    let protected = finalized.protected();
    assert_eq!(ready.candidate().version(), protected.version());
    assert_eq!(ready.candidate().inputs(), protected.inputs());
    assert_eq!(ready.candidate().outputs(), protected.outputs());
    assert_eq!(ready.candidate().lock_time(), protected.lock_time());
    assert_eq!(ready.candidate().witnesses(), vec![ready.witness().clone()]);
    assert_eq!(ready.bytes(), ready.candidate().encode());
    assert_eq!(
        ready.candidate().encode_without_witness(),
        finalized.protected_bytes()
    );
    assert_eq!(ready.authorization(), operator_signature(&finalized));
}

#[test]
fn the_witness_is_the_records_seven_items_then_the_leaf_and_the_control_block() {
    let finalized = demonstration();
    let target = reviewed_target();
    let ready = submit_ready_over(&finalized);
    let leaf = finalized.executing_leaf(&target);
    let stack = ready.witness().stack();

    let records = finalized.construction().abi().witness_roles();
    let roles: Vec<_> = records.iter().map(MaturityWitnessRole::role).collect();
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

    // Each item, recomputed from the source the ABI names for its role.
    let construction = finalized.construction();
    let view = construction.validated_view().view();
    let bundle = view.accepted_linked_bundle();
    let predecessor = bundle.instances()[0].constructor();
    let items: Vec<Vec<u8>> = vec![
        vec![prefix(construction.successor_constructor().parity())],
        construction
            .successor_constructor()
            .nonce()
            .get()
            .to_be_bytes()
            .to_vec(),
        ANNOUNCED.get().to_be_bytes().to_vec(),
        bundle.static_subtree().root().to_vec(),
        encode_state_metadata(
            &view.predecessor_metadata(),
            view.predecessor_representation_nonce(),
        ),
        vec![prefix(predecessor.parity())],
        operator_signature(&finalized),
        leaf.leaf_script().to_vec(),
        leaf.control_block().to_vec(),
    ];
    assert_eq!(stack, items);

    // Every declared width answers, and every item is the width its own
    // record declares.
    let widths: Vec<usize> = stack.iter().take(records.len()).map(Vec::len).collect();
    assert_eq!(widths, vec![1, 4, 8, 32, 86, 1, 64]);
    for (record, item) in records.iter().zip(stack) {
        assert_eq!(record.minimum_width(), Some(item.len()));
        assert_eq!(record.maximum_width(), Some(item.len()));
    }
    assert_eq!(stack.len(), records.len() + 2);
}

#[test]
fn the_two_parity_bytes_are_the_two_constructors_and_not_the_view() {
    let finalized = demonstration();
    let ready = submit_ready_over(&finalized);
    let stack = ready.witness().stack();
    let construction = finalized.construction();
    let view = construction.validated_view().view();
    let predecessor = view.accepted_linked_bundle().instances()[0].constructor();

    // Two commitments over different metadata, each stating its own
    // point's parity. Taking either from the other would state one
    // point's bit about the other.
    assert_eq!(
        stack[0],
        vec![prefix(construction.successor_constructor().parity())]
    );
    assert_eq!(stack[5], vec![prefix(predecessor.parity())]);

    // The view carries no parity to take one from: it states the
    // predecessor program as the target does, the two-byte
    // witness-version-one prefix and a thirty-two-byte x-only key, with
    // no compressed-key prefix anywhere in it.
    let program = view.predecessor_program();
    assert_eq!(program.len(), 34);
    assert_eq!(program.first(), Some(&0x51));
    assert_eq!(program.get(1), Some(&0x20));
}

#[test]
fn the_reviewed_compressed_key_class_publishes_the_two_prefix_bytes() {
    // The two constants are held against the reviewed registry's own
    // prefix set rather than asserted here, so a revision that moved a
    // prefix moves this test with it.
    let target = reviewed_target();
    let spec = target
        .definition()
        .encodings()
        .get(&EncodingClass::CompressedPublicKey)
        .expect("the reviewed target publishes the compressed-key encoding");
    let prefixes: Vec<u8> = spec.prefixes().iter().copied().collect();
    assert_eq!(
        prefixes,
        vec![EVEN_OUTPUT_KEY_PREFIX, ODD_OUTPUT_KEY_PREFIX]
    );
}

#[test]
fn the_submit_ready_bytes_decode_to_the_candidate_through_the_strict_decoder() {
    let finalized = demonstration();
    let ready = submit_ready_over(&finalized);
    assert_eq!(
        TargetTransaction::decode(ready.bytes()).as_ref(),
        Ok(ready.candidate())
    );
}

#[test]
fn the_protected_region_census_is_the_guides_twelve_in_its_own_order() {
    assert_eq!(MaturityProtectedRegion::ALL.len(), 12);
    assert_eq!(
        MaturityProtectedRegion::ALL.to_vec(),
        vec![
            MaturityProtectedRegion::Inputs,
            MaturityProtectedRegion::SpentOutputCensus,
            MaturityProtectedRegion::Outputs,
            MaturityProtectedRegion::SuccessorMetadata,
            MaturityProtectedRegion::SuccessorRepresentationNonce,
            MaturityProtectedRegion::SuccessorProgram,
            MaturityProtectedRegion::OutputWitnesses,
            MaturityProtectedRegion::Version,
            MaturityProtectedRegion::LockTime,
            MaturityProtectedRegion::SponsorRegion,
            MaturityProtectedRegion::FeeRegion,
            MaturityProtectedRegion::ExecutingLeafData,
        ]
    );
}

#[test]
fn every_protected_region_refuses_its_own_mutation_by_name() {
    let finalized = demonstration();
    let target = reviewed_target();
    let state = started(&finalized);
    let leaf = finalized.executing_leaf(&target);
    let input = finalized
        .protected()
        .inputs()
        .first()
        .expect("the announcement spends one input")
        .clone();
    let output = successor(&finalized);

    // One reach per member, in the census's own order, so the list this
    // produces can be compared with the census rather than with a
    // second copy of it.
    let named: Vec<MaturityProtectedRegion> = vec![
        region_of(state.add_input(&input)),
        region_of(state.restate_spent_output(&finalized.spent_output().census_entry())),
        region_of(state.add_output(&output)),
        region_of(state.set_successor_metadata(finalized.construction().successor_metadata())),
        region_of(state.set_successor_nonce(StateRepresentationNonce::new(1))),
        region_of(state.set_successor_program(output.program())),
        region_of(state.set_output_witness(&OutputWitness::empty())),
        region_of(state.set_version(3)),
        region_of(state.set_lock_time(1)),
        region_of(state.add_sponsor_input(&input)),
        region_of(state.set_fee_role(&output)),
        region_of(state.substitute_executing_leaf(&leaf)),
    ];
    assert_eq!(named, MaturityProtectedRegion::ALL);
}

#[test]
fn the_scope_is_built_from_the_finalized_value_alone_and_is_stable() {
    let finalized = demonstration();
    let state = started(&finalized);
    let scope = state.construction_right_scope();
    let view = finalized.construction().validated_view().view();
    let bound = binding(&finalized);

    assert_eq!(scope.deployment(), bound.deployment());
    assert_eq!(scope.branch(), &view.current_root_binding());
    assert_eq!(scope.predecessor(), view.current_state_outpoint());
    assert_eq!(scope.operator(), bound.key().bytes());
    assert_eq!(scope.capability_revision(), bound.capability_revision());

    // Re-opening the same finalized value freezes the same bytes under
    // the same scope, which is what lets a returned token still fit.
    let reopened = started(&finalized);
    assert_eq!(reopened.construction_right_scope(), scope);
    assert_eq!(reopened.protected_bytes(), finalized.protected_bytes());
}

#[test]
fn the_registry_records_the_issue_and_the_signature_and_refuses_a_second_right() {
    let finalized = demonstration();
    let mut registry = OperatorRightRegistry::default();
    let state = started(&finalized);
    let scope = state.construction_right_scope();
    let bytes = state.protected_bytes().to_vec();
    let answer = parts(&state);
    let right = issue(&mut registry, &state);
    let authorized = state
        .authorize(
            &mut registry,
            right,
            [answer.response()],
            &FixtureScriptPathVerifier,
        )
        .expect("one well-formed response authorizes the candidate");

    let events: Vec<NonEquivocationEvent> = registry
        .record()
        .entries()
        .iter()
        .map(|entry| entry.event.clone())
        .collect();
    assert_eq!(
        events,
        vec![NonEquivocationEvent::Issued, NonEquivocationEvent::Signed]
    );

    // The scope is spent: a second right over it is refused as consumed
    // rather than issued again.
    let identity = registry
        .record()
        .entries()
        .first()
        .expect("the record opened with the issue")
        .right
        .clone();
    assert_eq!(
        registry
            .issue(scope.clone(), &bytes)
            .expect_err("a signed scope issues no second right"),
        RightRefusal::Consumed(identity)
    );

    // An identical retry reads the cached artifact without a signer,
    // and the artifact is the witness the boundary returned.
    let outcome = registry
        .retry(&scope, &bytes)
        .expect("an identical retry reads the cache");
    match outcome {
        OperatorRightOutcome::Cached(cached) => {
            assert_eq!(&cached.witness, authorized.witness());
            assert_eq!(&cached.standing, authorized.standing());
        }
        OperatorRightOutcome::Fresh(_) => panic!("a retry invokes no signer"),
    }
    assert_eq!(
        registry
            .record()
            .entries()
            .last()
            .map(|entry| entry.event.clone()),
        Some(NonEquivocationEvent::Cached)
    );
}

#[test]
fn a_refused_authorization_returns_its_token_for_the_reopened_state() {
    let finalized = demonstration();
    let mut registry = OperatorRightRegistry::default();
    let state = started(&finalized);
    let mut broken = parts(&state);
    broken.signature.truncate(63);
    let right = issue(&mut registry, &state);
    let failure = state
        .authorize(
            &mut registry,
            right,
            [broken.response()],
            &FixtureScriptPathVerifier,
        )
        .expect_err("a misshapen signature produces no accepted artifact");
    let MaturityAuthorizationFailure { refusal, right } = *failure;
    assert_eq!(
        refusal,
        TransactionRefusal::MaturityOperatorAuthorizationRefused {
            refusal: Box::new(OperatorSigningRefusal::MalformedSignature { offered: 63 }),
        }
    );

    // The scope stayed outstanding under a recorded refusal, so the
    // returned token consumes the re-opened request. A signature that
    // dropped the token would have locked this scope for good.
    let reopened = started(&finalized);
    let answer = parts(&reopened);
    let authorized = reopened
        .authorize(
            &mut registry,
            right,
            [answer.response()],
            &FixtureScriptPathVerifier,
        )
        .expect("the returned token still fits the re-opened request");
    assert_eq!(authorized.protected_bytes(), finalized.protected_bytes());

    let events: Vec<NonEquivocationEvent> = registry
        .record()
        .entries()
        .iter()
        .map(|entry| entry.event.clone())
        .collect();
    assert_eq!(
        events,
        vec![
            NonEquivocationEvent::Issued,
            NonEquivocationEvent::SigningRefused,
            NonEquivocationEvent::Signed,
        ]
    );
}

#[test]
fn a_token_from_another_registry_is_refused_as_a_construction_right_failure() {
    let finalized = demonstration();
    let state = started(&finalized);
    let scope = state.construction_right_scope();
    let answer = parts(&state);
    let mut foreign = OperatorRightRegistry::default();
    let right = issue(&mut foreign, &state);

    let mut registry = OperatorRightRegistry::default();
    let failure = state
        .authorize(
            &mut registry,
            right,
            [answer.response()],
            &FixtureScriptPathVerifier,
        )
        .expect_err("a registry that issued nothing authorizes nothing");
    assert_eq!(
        failure.refusal,
        TransactionRefusal::MaturityConstructionRightRefused {
            refusal: Box::new(RightRefusal::NoRight(Box::new(scope))),
        }
    );
}

// --- §13.4's thirteen operator faults ---------------------------------

#[test]
fn a_missing_response_is_named() {
    assert_eq!(
        operator_fault(refused_over(Vec::clear)),
        OperatorSigningRefusal::MissingResponse { input_index: 0 }
    );
}

#[test]
fn a_duplicate_response_is_named() {
    let refusal = operator_fault(refused_over(|offered| {
        let repeat = offered[0].clone();
        offered.push(repeat);
    }));
    assert_eq!(
        refusal,
        OperatorSigningRefusal::DuplicateResponse { input_index: 0 }
    );
}

#[test]
fn a_surplus_response_for_another_input_is_named() {
    let refusal = operator_fault(refused_over(|offered| {
        let mut surplus = offered[0].clone();
        surplus.index = 1;
        offered.push(surplus);
    }));
    assert_eq!(
        refusal,
        OperatorSigningRefusal::UnexpectedResponse { input_index: 1 }
    );
}

#[test]
fn a_response_for_another_input_is_named() {
    assert_eq!(
        refused_with(|parts| parts.index = 1),
        OperatorSigningRefusal::WrongInput {
            expected: 0,
            offered: 1,
        }
    );
}

#[test]
fn a_response_from_another_operator_is_named() {
    let target = reviewed_target();
    let closure = operator_key_encoding_closure(target.definition().authorization());
    let other = OperatorKey::new(&closure, closure.approved(), vec![0x44; 32])
        .expect("public fixture bytes have the approved encoding");
    let bound = OperatorKey::new(&closure, closure.approved(), vec![0x33; 32])
        .expect("public fixture bytes have the approved encoding");
    assert_eq!(
        refused_with(|parts| parts.operator = other.clone()),
        OperatorSigningRefusal::WrongOperator {
            bound,
            offered: other,
        }
    );
}

#[test]
fn a_response_for_another_deployment_is_named() {
    let elsewhere = asymmetric_genesis_identity();
    let refusal = refused_with(|parts| parts.deployment = elsewhere.clone());
    match refusal {
        OperatorSigningRefusal::WrongDeployment { offered, .. } => {
            assert_eq!(*offered, elsewhere);
        }
        other => panic!("another deployment is named as one: {other:?}"),
    }
}

#[test]
fn a_response_under_another_profile_revision_is_named() {
    assert!(matches!(
        refused_with(|parts| parts.revision = TargetContractVersion::V1),
        OperatorSigningRefusal::WrongProfile(OperatorProfileDisposition::StaleRevision { .. })
    ));
}

#[test]
fn a_response_with_another_type_byte_is_named() {
    assert_eq!(
        refused_with(|parts| parts.type_byte = 0x01),
        OperatorSigningRefusal::WrongTypeByte { offered: 0x01 }
    );
}

#[test]
fn an_empty_signature_is_named() {
    assert_eq!(
        refused_with(|parts| parts.signature.clear()),
        OperatorSigningRefusal::EmptySignature { input_index: 0 }
    );
}

#[test]
fn a_misshapen_signature_is_named() {
    assert_eq!(
        refused_with(|parts| parts.signature.truncate(63)),
        OperatorSigningRefusal::MalformedSignature { offered: 63 }
    );
}

#[test]
fn an_echo_of_other_bytes_is_named() {
    assert_eq!(
        refused_with(|parts| parts.echo = vec![0x02; 60]),
        OperatorSigningRefusal::BoundToOtherBytes { input_index: 0 }
    );
}

#[test]
fn a_signature_over_another_message_is_named() {
    // The fixture function answers differently for a different message,
    // so a signature taken over another one fails the verifier rather
    // than passing by coincidence.
    let refusal = refused_with(|parts| {
        parts.signature = fixture_sign(&[0x33; 32], &[0x5a; 32]).to_vec();
    });
    assert!(matches!(
        refusal,
        OperatorSigningRefusal::SignatureDoesNotVerifyForFrozenMessage { input_index: 0, .. }
    ));
}

#[test]
fn a_mismatched_leaf_selection_is_refused_by_the_freeze() {
    // What this shows, and what it does not. The STATE chain cannot
    // reach this fault with a crafted response: the executing leaf is
    // derived from the predecessor's committed tree inside the
    // finalized form's own request builder, and there is no parameter,
    // argument or setter through which another could arrive. The fault
    // belongs to the freeze, and it is reached here by freezing a
    // request over this same candidate with a script that does not hash
    // to the selection — which shows that the boundary names it, not
    // that a caller of this chain could provoke it. §12.7's answer for
    // a caller reaching for another leaf is the executing-leaf region's
    // own refusal, which the census test reaches by name.
    let finalized = demonstration();
    let target = reviewed_target();
    let leaf = finalized.executing_leaf(&target);
    let bound = binding(&finalized);
    let mut genesis = *bound.deployment().genesis_id();
    genesis.reverse();

    let mut script = leaf.leaf_script().to_vec();
    script.push(0x51);
    let computed = leaf_hash(leaf.leaf_version(), &script);
    let refusal = OperatorSigningRequest::freeze(
        &target,
        bound,
        finalized.protected().clone(),
        vec![finalized.spent_output().census_entry()],
        LiveDeployment::new(genesis),
        OperatorSigningInput::new(
            0,
            *leaf.tapleaf_hash(),
            leaf.leaf_version(),
            script,
            leaf.control_block().to_vec(),
        ),
        &FixtureLiveCurve,
    )
    .expect_err("the script must hash to the selected leaf");
    assert_eq!(
        refusal,
        OperatorSigningRefusal::WrongLeaf {
            input_index: 0,
            expected: *leaf.tapleaf_hash(),
            computed,
        }
    );
}

// --- §13.4's four post-signing mutations ------------------------------

#[test]
fn a_changed_successor_metadata_is_named() {
    let finalized = demonstration();
    let authorized = authorized_over(&finalized);
    let mut moved = finalized.construction().successor_metadata();
    moved.cycle = Cycle::new(99);

    assert_eq!(
        authorized
            .check_offered(finalized.protected(), moved)
            .expect_err("a moved metadata is not the signed one"),
        TransactionRefusal::MaturitySuccessorMetadataChangedAfterSigning
    );

    // The unmoved pair passes, so the check is about the metadata
    // rather than about there being a second argument.
    assert_eq!(
        authorized.check_offered(
            finalized.protected(),
            finalized.construction().successor_metadata()
        ),
        Ok(())
    );
}

#[test]
fn a_changed_successor_program_is_named() {
    let finalized = demonstration();
    let authorized = authorized_over(&finalized);
    let fixed = successor(&finalized);
    let mut program = fixed.program().to_vec();
    if let Some(last) = program.last_mut() {
        *last ^= 0x01;
    }
    let moved = TargetOutput::new(fixed.asset(), fixed.value(), NonceField::Null, program);

    assert_eq!(
        authorized
            .check_offered(
                &offered(&finalized, vec![moved], 1),
                finalized.construction().successor_metadata()
            )
            .expect_err("a moved successor program is not the signed one"),
        TransactionRefusal::MaturitySuccessorProgramChangedAfterSigning {
            position: finalized.outputs().successor_position(),
        }
    );
}

#[test]
fn a_sponsor_input_added_after_signing_is_named() {
    let finalized = demonstration();
    let authorized = authorized_over(&finalized);
    let sponsored = offered(&finalized, vec![successor(&finalized)], 2);

    assert_eq!(
        authorized
            .check_offered(&sponsored, finalized.construction().successor_metadata())
            .expect_err("a second input opens the region the form fixed as absent"),
        TransactionRefusal::MaturitySponsorInputAddedAfterSigning {
            finalized: 1,
            offered: 2,
        }
    );
}

#[test]
fn a_fee_role_changed_after_signing_is_named() {
    let finalized = demonstration();
    let authorized = authorized_over(&finalized);
    let fixed = successor(&finalized);
    // An empty program is the target's fee marker, and the finalized
    // form fixed the fee role as absent.
    let fee = TargetOutput::new(fixed.asset(), fixed.value(), NonceField::Null, Vec::new());
    let with_fee = offered(&finalized, vec![fixed, fee], 1);

    assert_eq!(
        authorized
            .check_offered(&with_fee, finalized.construction().successor_metadata())
            .expect_err("an output beyond the successor changes the fee role"),
        TransactionRefusal::MaturityFeeRoleChangedAfterSigning { position: 1 }
    );
}

#[test]
fn what_the_four_do_not_reach_falls_through_to_the_finalized_comparison() {
    // An input's sequence moves without the input census changing size,
    // without any output moving and without the version or lock time
    // differing, so none of the four names it and the finalized form's
    // own exact-byte close is what catches it.
    let finalized = demonstration();
    let authorized = authorized_over(&finalized);
    let protected = finalized.protected();
    let one = protected
        .inputs()
        .first()
        .expect("the announcement spends one input")
        .clone();
    let resequenced = TargetInput::new(one.outpoint(), one.sequence().wrapping_sub(1));
    let moved = TargetTransaction::new(
        protected.version(),
        vec![resequenced],
        protected.outputs().to_vec(),
        protected.lock_time(),
        vec![InputWitness::default()],
    )
    .expect("one input carries one witness");

    assert!(matches!(
        authorized
            .check_offered(&moved, finalized.construction().successor_metadata())
            .expect_err("a moved sequence is not the signed candidate"),
        TransactionRefusal::MaturityBytesDifferAfterFinalization { .. }
    ));
}

#[test]
fn the_started_state_reads_its_request_and_its_finalized_candidate() {
    let finalized = demonstration();
    let state = started(&finalized);
    assert_eq!(state.protected_bytes(), finalized.protected_bytes());
    assert_eq!(state.request().frozen_bytes(), finalized.protected_bytes());
    assert_eq!(state.request().input_index(), 0);
    assert_eq!(
        state.finalized().protected_bytes(),
        finalized.protected_bytes()
    );
    assert_eq!(
        state.request().predecessor_outpoint(),
        Some(
            finalized
                .construction()
                .validated_view()
                .view()
                .current_state_outpoint()
        )
    );
}
