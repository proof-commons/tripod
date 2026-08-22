//! Guide-13 preflight reproductions owned by this crate.
//!
//! Each test here belongs to a row of the Guide-13 preflight register
//! and reaches the branch that row reports. Wave 0 wrote each assertion
//! as the property that *should* hold and marked it `#[ignore]`, so the
//! test failed while the defect stood; the wave that repairs a row lifts
//! the attribute rather than writing a new test, which is what makes the
//! test that reproduced the defect the same test that now witnesses its
//! absence. No attribute remains: both rows below are repaired.
//!
//! - `G13-R07` — REPAIRED: `is_valid_scalar` refused a zero tweak, so
//!   the oracle rejected a construction the target accepts. The guard
//!   is now overflow alone, and the identity result is refused after
//!   the addition that establishes it.
//! - `G13-R14` — REPAIRED: neither response validator checked a record
//!   against the role it answers or against its own verdict. Both are
//!   now exhaustive over role and outcome.
//!
//! # One repair here had to flip a standing test
//!
//! `constructor_tests::a_tweak_that_is_not_a_scalar_has_no_output_key`
//! asserted the old behaviour for the zero tweak directly, so the row
//! could not close without changing that assertion as well as lifting
//! the attributes here. The two together were the whole of what the row
//! cost, and the standing test now holds the corrected rule beside the
//! boundary — the group order — that zero was mistaken for.

use crate::constructor::curve::{FIELD_ELEMENT_BYTES, generator};
use crate::constructor::internal_key::UNSPENDABLE_INTERNAL_KEY;
use crate::constructor::tagged::Digest32;
use crate::constructor::tree::{TweakDefect, tweaked_key};
use crate::lifecycle::LifecycleOutcome;
use crate::protocol::{
    LifecycleCaseId, LifecycleStepRole, NATIVE_PROTOCOL_SCHEMA, NativeLifecycleResponse,
    NativeOperationResponse, NativeResourceObservation, ObservedOutcomeLayer, OperationCaseId,
    OperationStepKind, ResponseShapeDefect,
};

/// One 32-byte value from its big-endian hexadecimal spelling.
///
/// Public test material in the sense
/// `(´[ADR015-rule:security:test-material]´)` fixes: curve constants
/// and boundary values, standing for no secret.
fn digest(hexadecimal: &str) -> Digest32 {
    let mut value = [0_u8; FIELD_ELEMENT_BYTES];
    for (slot, pair) in value.iter_mut().zip(hexadecimal.as_bytes().chunks(2)) {
        let text = std::str::from_utf8(pair).expect("the fixture spelling is ASCII");
        *slot = u8::from_str_radix(text, 16).expect("the fixture spelling is hexadecimal");
    }
    value
}

/// The order of the group, which is the first invalid multiplier.
fn group_order() -> Digest32 {
    digest("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141")
}

/// `G13-R07`: a zero tweak has an output key, and it is the input key.
///
/// The target's rule is overflow alone. `secp256k1_ec_pubkey_tweak_add`
/// reads the tweak as a scalar and returns `!overflow && …`, and the
/// addition beneath it fails only on the identity, so `t = 0` is
/// accepted and gives `Q = P + 0G = P`. The oracle refuses it before
/// the addition, which invents a target rule and classifies a valid
/// construction as `TweakNotAScalar`.
///
/// The curve arithmetic here is already total for zero — a zero scalar
/// multiplies to the identity and the identity is the additive unit —
/// so the whole of the row is the guard in `is_valid_scalar`.
#[test]
fn a_zero_tweak_has_an_output_key_and_it_is_the_internal_key() {
    let (output, parity) = tweaked_key(&UNSPENDABLE_INTERNAL_KEY, &[0_u8; FIELD_ELEMENT_BYTES])
        .expect("Q = P + 0G = P is a valid output key");

    assert_eq!(
        output, UNSPENDABLE_INTERNAL_KEY,
        "adding the identity leaves the internal key alone",
    );
    assert_eq!(
        parity, 0,
        "the lifted internal key has even y, and so does the point equal to it",
    );
}

/// `G13-R07`, the boundary that is genuinely invalid.
///
/// The order itself is the least value that is not a multiplier, and
/// the oracle is right to refuse it. Kept as a running control so a
/// repair that accepted zero by dropping the range check entirely would
/// fail here rather than pass quietly.
#[test]
fn a_tweak_at_the_group_order_still_has_no_output_key() {
    assert_eq!(
        tweaked_key(&UNSPENDABLE_INTERNAL_KEY, &group_order()),
        Err(TweakDefect::TweakNotAScalar),
    );
}

/// `G13-R07`, the largest tweak that is a multiplier.
///
/// One below the order, which the oracle already accepts. It is here to
/// fix the boundary from the other side: the refusal above is about the
/// order exactly, not about large values.
#[test]
fn a_tweak_one_below_the_group_order_has_an_output_key() {
    tweaked_key(&UNSPENDABLE_INTERNAL_KEY, &one_below_the_group_order())
        .expect("one below the order is a valid multiplier");
}

/// One below the order, which is the last value that is a multiplier.
fn one_below_the_group_order() -> Digest32 {
    digest("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364140")
}

/// `G13-R07`, the failure that is genuinely about the sum.
///
/// The identity result, constructed rather than searched for. `n-1` is
/// the negation of one, so for the generator itself the sum is
/// `G + (n-1)G = nG`, which is the identity — the one point with no
/// x-only encoding, and the whole of what
/// [`TweakDefect::TweakedKeyIsIdentity`] is for.
///
/// This is what the repaired guard leaves standing, and it is the
/// reason the repair is a *move* rather than a removal. Zero is
/// accepted before the addition because zero says nothing about the
/// sum; this is refused after it, because the sum is where the fact
/// is. A repair that had dropped the range check entirely, or that had
/// kept refusing zero to cover this case, would fail one of the two
/// tests above and this one respectively.
///
/// It is also the only construction in this file whose internal key is
/// not the unspendable one. That is deliberate: the identity result
/// requires knowing the internal key's discrete logarithm, and the
/// generator is the one point whose logarithm anybody knows. Nobody
/// reaches this case with a key nothing has the logarithm of, which is
/// exactly why the constructor's policy calls it negligible rather
/// than impossible.
#[test]
fn a_tweak_that_sums_to_the_identity_has_no_output_key() {
    assert_eq!(
        tweaked_key(&generator().x_only_bytes(), &one_below_the_group_order()),
        Err(TweakDefect::TweakedKeyIsIdentity),
    );
}

/// One lifecycle response, in whatever shape a test needs.
fn lifecycle(role: LifecycleStepRole, outcome: LifecycleOutcome) -> NativeLifecycleResponse {
    NativeLifecycleResponse {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: LifecycleCaseId { lifecycle: role },
        outcome,
        handoff: None,
        authorization_profile: None,
        observed_witness_sizes: Vec::new(),
        observed_outputs: Vec::new(),
        checks: Vec::new(),
        spend: None,
        superseded_by: None,
        supersede_failure: None,
        detail: None,
    }
}

/// `G13-R14`, part A: a lifecycle verdict belongs to its own role.
///
/// `LifecycleOutcome::Verified` is Process B's verdict — it says the
/// object was located, parsed, checked, and spent. A `Construct`
/// response claiming it is a record of Process A reporting what Process
/// B concluded, which no run produced.
///
/// `validate_shape` never read `case.lifecycle` at all. Its one rule
/// was about a step that did *not* run, and `Verified` runs, so the
/// record passed untouched — while also carrying no handoff, which is
/// the one thing a construct step exists to publish.
///
/// The module documentation claimed the two roles' field separation is
/// enforced here, and that claim is what this test holds it to.
#[test]
fn a_construct_response_cannot_report_the_verifier_s_verdict() {
    let contradictory = lifecycle(LifecycleStepRole::Construct, LifecycleOutcome::Verified);

    // Wave 0 asserted the refusal as a refusal, because no variant
    // named a role contradiction and the row was not to pre-empt which
    // one the repair would mint. It minted this one, so the assertion
    // names it: a refusal under some other variant would mean the
    // record was refused for some other reason.
    assert_eq!(
        contradictory.validate_shape(),
        Err(ResponseShapeDefect::LifecycleOutcomeMismatchesRole),
        "a construct step reported the verdict only a verify step reaches",
    );
}

/// `G13-R14`, part A, the other direction.
///
/// A `Verify` response claiming `Constructed` is the same defect
/// mirrored: Process B does not publish a record, and the outcome that
/// says one was published is not its to report.
#[test]
fn a_verify_response_cannot_report_the_constructor_s_verdict() {
    let contradictory = lifecycle(LifecycleStepRole::Verify, LifecycleOutcome::Constructed);

    assert_eq!(
        contradictory.validate_shape(),
        Err(ResponseShapeDefect::LifecycleOutcomeMismatchesRole),
        "a verify step reported the verdict only a construct step reaches",
    );
}

/// One operation response, in whatever shape a test needs.
fn operation(kind: OperationStepKind, layer: ObservedOutcomeLayer) -> NativeOperationResponse {
    NativeOperationResponse {
        schema: NATIVE_PROTOCOL_SCHEMA,
        case: OperationCaseId {
            operation: kind,
            step: "step".to_owned(),
        },
        observed_layer: layer,
        observed_detail: None,
        issued_asset: None,
        funded_outputs: Vec::new(),
        accepted_txid: None,
        sponsor_witness: Vec::new(),
        signature_bound_to: None,
        resources: NativeResourceObservation::default(),
    }
}

/// `G13-R14`, part B: a rejected submission has no accepted identity.
///
/// A transaction identity is one the target computed over bytes it
/// accepted. A record that reports a consensus rejection and an
/// accepted txid in the same breath carries two answers to one
/// question, and a consumer reading either field alone gets a different
/// verdict from the same row.
///
/// `validate_shape` requires the identity to be *present* on
/// acceptance, and never requires it to be absent otherwise: the
/// converse the row names.
#[test]
fn a_rejected_submission_carries_no_accepted_transaction_identity() {
    let mut contradictory = operation(
        OperationStepKind::Submit,
        ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
    );
    contradictory.accepted_txid = Some("00".repeat(32));

    assert_eq!(
        contradictory.validate_shape(),
        Err(ResponseShapeDefect::RefusedOperationCarriesObservation),
        "a rejection reported the identity of an acceptance",
    );
}

/// `G13-R14`, part B: a rejected funding step created no coins.
///
/// A funded output is a coin the target created. A funding step the
/// target refused created none, so a nonempty output list beside a
/// script-path rejection is an observation with no provenance.
#[test]
fn a_rejected_funding_step_reports_no_created_outputs() {
    let mut contradictory = operation(
        OperationStepKind::Fund,
        ObservedOutcomeLayer::ScriptPathRejection,
    );
    contradictory.issued_asset = Some("00".repeat(32));

    assert_eq!(
        contradictory.validate_shape(),
        Err(ResponseShapeDefect::RefusedOperationCarriesObservation),
        "a refused funding step reported an asset the target never issued",
    );
}

/// `G13-R14`, part B: a rejected signing step produced no authorization.
///
/// The strongest of the three, because the authorization is the artifact
/// the whole step exists to produce. A witness stack and the bytes it
/// was bound to, reported beside a target rejection, is a signature
/// attributed to a run that the target refused.
#[test]
fn a_rejected_signing_step_reports_no_authorization() {
    let mut contradictory = operation(
        OperationStepKind::SignSponsor,
        ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
    );
    contradictory.sponsor_witness = vec![vec![0x30], vec![0x02]];
    contradictory.signature_bound_to = Some(vec![0x02]);

    assert_eq!(
        contradictory.validate_shape(),
        Err(ResponseShapeDefect::RefusedOperationCarriesObservation),
        "a refused signing step reported an authorization",
    );
}

/// `G13-R14`, the directions that already hold.
///
/// The row is about the *converse* rules being absent, so the rules
/// that are present are pinned here: an acceptance must show what its
/// kind produces, and a non-verdict layer must show nothing at all. A
/// repair that added the converse by loosening these would fail here.
#[test]
fn an_accepted_step_must_still_report_what_its_kind_produces() {
    assert_eq!(
        operation(OperationStepKind::Submit, ObservedOutcomeLayer::Accepted).validate_shape(),
        Err(ResponseShapeDefect::AcceptedOperationOmitsObservation),
    );

    let mut infrastructure = operation(
        OperationStepKind::Submit,
        ObservedOutcomeLayer::ExecutorInfrastructureFailure,
    );
    infrastructure.accepted_txid = Some("00".repeat(32));
    assert_eq!(
        infrastructure.validate_shape(),
        Err(ResponseShapeDefect::InfrastructureResponseCarriesObservation),
    );
}
