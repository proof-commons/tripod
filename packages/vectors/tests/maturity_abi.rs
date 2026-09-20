//! A maturity announcement carried to submit-ready, with a signature
//! that verifies.
//!
//! The whole announcement chain is run here over one deployment: the
//! public current-STATE view, the derived ABI, the typed request, the
//! construction, the finalization, the frozen operator request, the
//! authorization under a construction right, and the bound submit-ready
//! candidate. What makes the run worth making is the deployment it is
//! run over. A committed operator key is what the leaf's first
//! instruction pair verifies a signature against, so a deployment that
//! commits meaningless fill is one no signature can be produced for at
//! all, and a chain over it stops at the authorization by construction
//! rather than by any property of the chain. The deployment used here
//! commits the x-only key of a published test signer, whose scalar the
//! conformance layer publishes, so the signature offered is a real
//! Schnorr signature over the frozen message and an independent
//! verifier's acceptance of it is a fact about the signature rather
//! than about a stub.
//!
//! The wrong-key control is the other published scalar, and it is
//! offered in two shapes. One keeps the committed operator identity and
//! changes only the scalar that signed, which is refused by the
//! verifier on public data; the other keeps the signature and changes
//! the operator the response names, which is refused by the binding
//! before any verification runs. Together they say that the acceptance
//! above is decided by the signature and the key jointly, and not by
//! either alone.
//!
//! Determinism is stated over the complete input tuple — the
//! deployment's three values, the seven view statements, the request,
//! the signature type byte and the signing auxiliary — and the second
//! build shares nothing with the first but those inputs: it links its
//! own bundle through the same real curve, builds its own view, issues
//! from its own registry, and freezes its own message. The one-term
//! control beside it announces a different cycle the same window
//! admits and gets different bytes, which is what keeps the equality a
//! property of the inputs rather than of the fixture.
//!
//! What this file does not claim: nothing here is submitted, no node
//! runs, and no interpreter is consulted. The verification is
//! in-process and the candidate's standing records exactly that. The
//! predecessor outpoint and the branch binding are caller fixtures that
//! nothing here checks, which is the validated view's own stated
//! residual: that the outpoint is the current root is established by no
//! layer in this chain.

use std::sync::LazyLock;

use linker::{CandidateDeploymentIdentity, CandidateLinkedMaturityBundle};
use realization::Cycle;
use tapscript::{OperatorKey, operator_key_encoding_closure};
use target_elements::{ReviewedElementsTapscriptDefinition, TargetContractVersion};
use target_elements_conformance::test_material::PublicTestSignerHandle;
use transaction::bytes::{AssetField, AssetId, Outpoint, TargetTransaction, Txid, ValueField};
use transaction::error::TransactionRefusal;
use transaction::live_request::{RequestedForm, SponsorChangeRequest};
use transaction::operator_right::{
    BranchContext, ConstructionRight, NonEquivocationEvent, OperatorRightRegistry,
};
use transaction::operator_signing::{
    OPERATOR_SIGHASH_TYPE_BYTE, OperatorEvidenceStanding, OperatorSigningRefusal,
    OperatorSigningResponse, ScriptPathSignatureVerifier,
};
use transaction::state_abi::{MaturityAbiStatus, derive_maturity_announcement_abi};
use transaction::state_construct::construct_maturity_announcement;
use transaction::state_finalize::{FinalizedMaturityAnnouncement, finalize_maturity_announcement};
use transaction::state_request::MaturityAnnouncementRequest;
use transaction::state_signing::{
    MaturityAuthorizationFailure, MaturitySubmissionStatus, OperatorAuthorizedMaturityAnnouncement,
    OperatorSigningStarted, SubmitReadyMaturityAnnouncement,
};
use transaction::state_view::{
    MaturityViewResidual, MaturityViewStatement, PublicMaturityStateView,
    ValidatedMaturityStateView,
};
use vectors::live_capability::OracleLiveCurve;
use vectors::maturity_closure::{
    MaturityDeployment, OracleStateCurve, closure_target, linked_maturity_bundle,
};
use vectors::maturity_operator::{OPERATOR_HANDLE, OperatorVerifier};

// --- The deployment and the values a caller supplies ---------------------

/// The deployment every chain here is taken over.
///
/// The one whose committed operator key a published signer holds. The
/// other two commit public fill, and a signature under fill is not
/// something a test can arrange.
const DEPLOYMENT: MaturityDeployment = MaturityDeployment::PublishedSignerHeld;

/// The cycle every announcement here is built for.
///
/// The predecessor this deployment's sources fix sits at cycle five,
/// and this deployment's lead window is four to six, so nine, ten and
/// eleven are the three admissible figures. Ten is the middle one: a
/// window that drifted by one in either direction would fail here
/// rather than pass by sitting on a boundary it still touches.
const ANNOUNCED: Cycle = Cycle::new(10);

/// The second admitted figure, which the one-term control announces.
const CONTROL_CYCLE: Cycle = Cycle::new(9);

/// The auxiliary the published signer masks its scalar with.
///
/// A stated value rather than randomness, which is what makes the
/// signature reproducible from the inputs this file publishes and the
/// determinism below a property with content.
const AUXILIARY: [u8; 32] = [0; 32];

/// The transaction the caller states the current STATE output sits in.
const PREDECESSOR_TXID: [u8; 32] = [0x71; 32];

/// The branch identifier the caller binds the current root at.
const BRANCH: [u8; 32] = [0x9b; 32];

/// The checkpoint ordinal beside it.
const CHECKPOINT: u64 = 12;

/// The seven items the ABI schedules before the executing leaf.
const WITNESS_ROLES: usize = 7;

// --- The fixtures --------------------------------------------------------

/// The reviewed contract every call here is made against.
fn target() -> ReviewedElementsTapscriptDefinition {
    closure_target().expect("the reviewed contract validates")
}

/// One fresh link of the deployment's bundle, through the real curve.
///
/// Every call links again rather than cloning, which is what lets a
/// second build of the chain be independent of the first.
fn link() -> CandidateLinkedMaturityBundle {
    linked_maturity_bundle(DEPLOYMENT).expect("the deployment links through the real curve")
}

/// The bundle the tests that need only one share, linked once.
fn shared_bundle() -> &'static CandidateLinkedMaturityBundle {
    static BUNDLE: LazyLock<CandidateLinkedMaturityBundle> = LazyLock::new(link);
    &BUNDLE
}

/// The seven statements of a view over one linked bundle.
///
/// Three of them are the bundle's own: the predecessor's semantic
/// metadata, its representation nonce and its output program are read
/// off the constructor application the link retained, so the view
/// states what was actually linked instead of a second copy of it that
/// could come to disagree. The remaining four are the caller's, and two
/// of those four — the outpoint and the branch binding — are checked by
/// nothing in this chain.
fn statements(bundle: &CandidateLinkedMaturityBundle) -> Vec<MaturityViewStatement> {
    let retained = &bundle.instances()[0];
    let parameters = DEPLOYMENT
        .parameters()
        .expect("the deployment resolves its committed key");
    vec![
        MaturityViewStatement::CurrentStateOutpoint(
            Outpoint::new(Txid::from_internal(PREDECESSOR_TXID), 0)
                .expect("output zero is an admitted index"),
        ),
        MaturityViewStatement::AssetAndAmount(
            AssetField::Explicit(AssetId::from_internal(*parameters.singleton())),
            ValueField::Explicit(1),
        ),
        MaturityViewStatement::PredecessorMetadata(retained.metadata().semantic),
        MaturityViewStatement::PredecessorRepresentationNonce(retained.metadata().representation),
        MaturityViewStatement::CurrentRootBinding(
            BranchContext::new(BRANCH, CHECKPOINT).expect("the branch identifier is nonzero"),
        ),
        MaturityViewStatement::PredecessorProgram(retained.constructor().output_program()),
        MaturityViewStatement::AcceptedLinkedBundle(Box::new(bundle.clone())),
    ]
}

/// The view over one bundle, with its one checkable relation computed.
fn validated(bundle: &CandidateLinkedMaturityBundle) -> ValidatedMaturityStateView {
    PublicMaturityStateView::new(statements(bundle))
        .expect("the seven statements name distinct entries")
        .validate(&target(), &OracleStateCurve)
        .expect("the stated pair reproduces the stated program")
}

/// The sponsorless request for one announced cycle.
fn request(cycle: Cycle) -> MaturityAnnouncementRequest {
    MaturityAnnouncementRequest::new(
        cycle,
        RequestedForm::Sponsorless,
        SponsorChangeRequest::NotRequested,
    )
    .expect("the sponsorless form without change is an admitted pair")
}

/// The finalized announcement over one bundle and one announced cycle.
///
/// Owned and returned by value, because the three states after it each
/// borrow a finalized value: a helper that finalized internally and
/// handed back a later state would be handing back a borrow of its own
/// temporary.
fn finalized_over(
    bundle: &CandidateLinkedMaturityBundle,
    cycle: Cycle,
) -> FinalizedMaturityAnnouncement {
    let target = target();
    let validated = validated(bundle);
    let abi = derive_maturity_announcement_abi(&target, &validated)
        .expect("the validated view derives its ABI");
    let construction = construct_maturity_announcement(
        &target,
        &abi,
        &validated,
        &request(cycle),
        &OracleStateCurve,
    )
    .expect("the deployment constructs its announcement");
    finalize_maturity_announcement(construction)
}

/// The frozen operator request over one finalized announcement.
fn started(finalized: &FinalizedMaturityAnnouncement) -> OperatorSigningStarted<'_> {
    let target = target();
    OperatorSigningStarted::open(finalized, &target, &OracleLiveCurve::new(target.clone()))
        .expect("the candidate freezes its operator request")
}

/// One published handle's signature over one frozen message.
fn signature(state: &OperatorSigningStarted<'_>, handle: PublicTestSignerHandle) -> Vec<u8> {
    handle
        .material()
        .expect("the published handle resolves its material")
        .sign(state.request().message().with_vector_grown(), &AUXILIARY)
        .expect("the published scalar signs a thirty-two byte message")
        .to_vec()
}

/// One response's fields, before they are sealed into a response.
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
///
/// Every field is read off the state itself, so a control below changes
/// exactly the one term it means to change.
fn parts(state: &OperatorSigningStarted<'_>) -> ResponseParts {
    let bound = state.request().binding();
    ResponseParts {
        index: state.request().input_index(),
        signature: signature(state, OPERATOR_HANDLE),
        type_byte: OPERATOR_SIGHASH_TYPE_BYTE,
        echo: state.protected_bytes().to_vec(),
        operator: bound.key().clone(),
        deployment: bound.deployment().clone(),
        revision: bound.capability_revision(),
    }
}

/// The other published signer's key, in the operator encoding.
fn other_operator_key() -> OperatorKey {
    let target = target();
    let closure = operator_key_encoding_closure(target.definition().authorization());
    OperatorKey::new(
        &closure,
        closure.approved(),
        PublicTestSignerHandle::First
            .x_only_public_key()
            .expect("the first published handle resolves its key")
            .to_vec(),
    )
    .expect("a published x-only key is an admitted operator encoding")
}

/// A right over one started state's own scope and bytes.
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
        .authorize(&mut registry, right, [answer.response()], &OperatorVerifier)
        .expect("one well-formed response authorizes the candidate")
}

/// The submit-ready state over one finalized announcement.
fn submit_ready_over(
    finalized: &FinalizedMaturityAnnouncement,
) -> SubmitReadyMaturityAnnouncement<'_> {
    authorized_over(finalized)
        .bind_for_submission(&target())
        .expect("the authorized candidate binds its one witness")
}

/// The boundary's own finding inside one refused authorization.
fn operator_refusal(failure: &MaturityAuthorizationFailure) -> &OperatorSigningRefusal {
    match &failure.refusal {
        TransactionRefusal::MaturityOperatorAuthorizationRefused { refusal } => refusal,
        other => panic!("the operator boundary's own finding was expected, not {other:?}"),
    }
}

/// What one registry observed, in order.
fn events(registry: &OperatorRightRegistry) -> Vec<NonEquivocationEvent> {
    registry
        .record()
        .entries()
        .iter()
        .map(|entry| entry.event.clone())
        .collect()
}

// --- The view and the ABI ------------------------------------------------

#[test]
fn the_deployments_view_validates_through_the_real_curve_and_derives_its_abi() {
    let target = target();
    let validated = validated(shared_bundle());

    assert_eq!(
        validated.residual(),
        MaturityViewResidual::CurrentStateRootFreshness
    );
    assert_eq!(validated.view().current_cycle(), Cycle::new(5));

    let abi = derive_maturity_announcement_abi(&target, &validated)
        .expect("the validated view derives its ABI");
    assert_eq!(abi.contract(), target.definition().version());
    assert_eq!(abi.status(), MaturityAbiStatus::Candidate);
    assert_eq!(abi.witness_roles().len(), WITNESS_ROLES);
}

// --- The positive --------------------------------------------------------

#[test]
fn the_chain_reaches_a_submit_ready_candidate_the_published_signer_authorized() {
    let finalized = finalized_over(shared_bundle(), ANNOUNCED);
    let ready = submit_ready_over(&finalized);

    assert_eq!(
        ready.standing(),
        &OperatorEvidenceStanding::InProcessVerified {
            verifier: OperatorVerifier.description().to_owned(),
        }
    );
    assert_eq!(ready.status(), MaturitySubmissionStatus::SubmitReady);

    let roles = finalized.construction().abi().witness_roles().len();
    assert_eq!(roles, WITNESS_ROLES);

    let leaf = finalized.executing_leaf(&target());
    let stack = ready.witness().stack();
    assert_eq!(stack.len(), roles + 2);

    let widths: Vec<usize> = stack.iter().take(roles).map(Vec::len).collect();
    assert_eq!(widths, vec![1, 4, 8, 32, 86, 1, 64]);
    assert_eq!(stack[roles], leaf.leaf_script());
    assert_eq!(stack[roles + 1], leaf.control_block());
}

#[test]
fn the_submit_ready_candidate_carries_the_finalized_bytes_unchanged() {
    let finalized = finalized_over(shared_bundle(), ANNOUNCED);
    let ready = submit_ready_over(&finalized);

    assert_eq!(ready.protected_bytes(), finalized.protected_bytes());
    assert_eq!(
        ready.candidate().encode_without_witness(),
        finalized.protected_bytes()
    );
    assert_ne!(ready.bytes(), finalized.protected_bytes());
}

#[test]
fn the_authorized_signature_verifies_against_the_committed_key_outside_the_chain() {
    let finalized = finalized_over(shared_bundle(), ANNOUNCED);

    // Recomputed through a second freeze rather than read out of the
    // state under test, so the two sides of the comparison come from
    // two derivations. The freeze is deterministic over one finalized
    // value, so this is the message the witness was signed against.
    let fresh = started(&finalized);
    let committed = fresh.request().binding().key().bytes().to_vec();
    let message = *fresh.request().message().with_vector_grown();

    let state = started(&finalized);
    let answer = parts(&state);
    let mut registry = OperatorRightRegistry::default();
    let right = issue(&mut registry, &state);
    let ready = state
        .authorize(&mut registry, right, [answer.response()], &OperatorVerifier)
        .expect("one well-formed response authorizes the candidate")
        .bind_for_submission(&target())
        .expect("the authorized candidate binds its one witness");

    OperatorVerifier
        .verify(&committed, &message, ready.authorization())
        .expect("the committed key accepts this authorization over the frozen message");
    assert_eq!(
        events(&registry),
        vec![NonEquivocationEvent::Issued, NonEquivocationEvent::Signed]
    );
}

// --- The wrong-key control, in both its faces ----------------------------

#[test]
fn a_signature_by_the_other_published_scalar_is_refused_for_the_frozen_message() {
    let finalized = finalized_over(shared_bundle(), ANNOUNCED);
    let state = started(&finalized);
    let mut answer = parts(&state);
    answer.signature = signature(&state, PublicTestSignerHandle::First);

    let mut registry = OperatorRightRegistry::default();
    let right = issue(&mut registry, &state);
    let predecessor = state.construction_right_scope().predecessor();
    let failure = *state
        .authorize(&mut registry, right, [answer.response()], &OperatorVerifier)
        .expect_err("a signature by another scalar cannot authorize this candidate");

    assert!(matches!(
        operator_refusal(&failure),
        OperatorSigningRefusal::SignatureDoesNotVerifyForFrozenMessage { .. }
    ));
    assert_eq!(failure.right.scope().predecessor(), predecessor);
    assert_eq!(
        events(&registry),
        vec![
            NonEquivocationEvent::Issued,
            NonEquivocationEvent::SigningRefused
        ]
    );
}

#[test]
fn a_response_naming_the_other_published_key_as_operator_is_refused() {
    let finalized = finalized_over(shared_bundle(), ANNOUNCED);
    let state = started(&finalized);
    let mut answer = parts(&state);
    answer.operator = other_operator_key();

    let mut registry = OperatorRightRegistry::default();
    let right = issue(&mut registry, &state);
    let failure = *state
        .authorize(&mut registry, right, [answer.response()], &OperatorVerifier)
        .expect_err("a response naming another operator cannot authorize this candidate");

    assert!(matches!(
        operator_refusal(&failure),
        OperatorSigningRefusal::WrongOperator { .. }
    ));
}

// --- The strict decoder --------------------------------------------------

#[test]
fn the_finalized_and_candidate_bytes_decode_back_to_the_values_they_encode() {
    let finalized = finalized_over(shared_bundle(), ANNOUNCED);
    let protected = TargetTransaction::decode(finalized.protected_bytes())
        .expect("the finalized bytes are in the decoder's admitted language");
    assert_eq!(&protected, finalized.protected());

    let ready = submit_ready_over(&finalized);
    let candidate = TargetTransaction::decode(ready.bytes())
        .expect("the candidate bytes are in the decoder's admitted language");
    assert_eq!(&candidate, ready.candidate());
}

// --- Determinism over the complete input tuple ---------------------------

#[test]
fn equal_complete_input_tuples_give_equal_candidate_bytes() {
    let first_bundle = link();
    let second_bundle = link();
    let first = finalized_over(&first_bundle, ANNOUNCED);
    let second = finalized_over(&second_bundle, ANNOUNCED);
    let one = submit_ready_over(&first);
    let two = submit_ready_over(&second);

    assert_eq!(one.bytes(), two.bytes());
    assert_eq!(one.protected_bytes(), two.protected_bytes());
    assert_eq!(one.witness().stack(), two.witness().stack());
}

#[test]
fn a_different_admitted_cycle_gives_different_protected_bytes() {
    let announced = finalized_over(shared_bundle(), ANNOUNCED);
    let control = finalized_over(shared_bundle(), CONTROL_CYCLE);

    assert_ne!(announced.protected_bytes(), control.protected_bytes());
}
