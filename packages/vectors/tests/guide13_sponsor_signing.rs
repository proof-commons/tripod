//! Three facts about the sponsored deployment's own symbols.
//!
//! # The ceremony that used to live here has moved
//!
//! This file once carried the sponsor-envelope ceremony itself: finalize
//! an explicit sponsored control, send its exact sponsor request, replay
//! the returned witness through the sponsor capability, verify the byte
//! binding, and submit. That ceremony is now
//! [`vectors::live_sponsor_shapes`], and the native lane drives it for
//! both of its shapes.
//!
//! It moved because a ceremony inside a test can answer exactly one row.
//! A test cannot be called from another test, so the sponsored control
//! that TAKES CHANGE had nowhere to be built from — and the change role
//! is the sponsored side of the fee matrix. Lifting it cost nothing that
//! can be argued about: the without-change member reproduced its own
//! recorded identity after the move.
//!
//! # What stayed, and why it stayed here rather than moving too
//!
//! Three claims about what this repository CARRIES, each held without a
//! node on purpose. A claim only a node can check is one nobody checks,
//! and none of these three needs one: two are about what the linker
//! resolves for a deployment's fee-role digest, and the third is about
//! which residuals the live evidence set holds.
//!
//! The acceptances that earned the third needed a node. The fact that
//! the set no longer holds the residual does not.
//!
//! # Every value here is published test material
//!
//! ADR-015 public disposable test material at each use, authorizing
//! nothing on any network anybody uses.

use vectors::bundle::fee_program_digest;
use vectors::live_plan::demonstration_live_abi;

/// The demonstration deployment still cannot satisfy its own fee-role
/// check, and that is now a CHOICE rather than a defect.
///
/// Held WITHOUT a node, so the finding stays a property of the
/// deployment rather than a verdict somebody has to re-run a chain to
/// see. What it asserts changed when the digest was threaded, and the
/// distinction is the whole point of the threading.
///
/// §10.7's sponsor isolation ends by inspecting the fee output's
/// scriptPubKey and requiring its digest to equal the deployment's
/// `fee_program_digest` symbol. Construction writes the fee role with
/// the EMPTY program, because the fee role's identity is
/// target-structural and that is the structure. So the check demands
/// SHA-256 of nothing, and the demonstration's symbol is a fixture
/// constant no program hashes to.
///
/// It STAYS that constant deliberately. The demonstration is welded to
/// its symbols: moving this one would move the committed taptree and
/// with it every live run-of-record identity the plans cite. A
/// deployment that intends to spend a sponsored control supplies the
/// derived digest instead, which is what this lane's own `relink` now
/// does — so the demonstration's inability is no longer a blocker on
/// anything, only a fact about a deployment nobody submits.
///
/// The literal that used to sit beside this test is gone on purpose: a
/// written-out digest next to a helper that computes the same value is
/// the exact drift the helper exists to prevent.
#[test]
fn the_demonstration_fee_role_digest_is_a_constant_no_fee_program_hashes_to() {
    let abi = demonstration_live_abi().expect("the demonstration ABI derives");
    let pinned = abi.symbols().fee_program_digest().to_vec();

    // The empty program is what construction writes for the fee role.
    assert_ne!(
        pinned,
        fee_program_digest().to_vec(),
        "the demonstration's fee-role digest now matches the empty program, so the \
         demonstration deployment has MOVED and every live run-of-record identity \
         the plans cite must be re-derived before this passes again"
    );
}

/// A deployment linked for a real submission carries the derived digest.
///
/// The positive counterpart of the test above, and the reason the
/// threading is worth anything: the same three functions that keep the
/// demonstration's constant hand a submitting lane the digest of the fee
/// program construction actually writes. Held without a node, because
/// the claim is about what the linker resolves and not about what a
/// chain thinks of it.
///
/// The assets are the demonstration's own — this test is not about
/// which assets a deployment names, and using the constants keeps it
/// from depending on a chain having answered anything.
#[test]
fn a_deployment_linked_for_submission_carries_the_derived_fee_digest() {
    let derived = fee_program_digest();
    let abi = vectors::live_plan::live_abi_for_asset(
        vectors::live_plan::PROTOCOL_ASSET,
        vectors::live_plan::RESERVE_ASSET,
        derived,
    )
    .expect("the ABI derives for a stated fee digest");

    assert_eq!(
        abi.symbols().fee_program_digest().to_vec(),
        derived.to_vec(),
        "the threaded fee digest did not reach the resolved symbols, so a submitting \
         lane would still be linking the demonstration's fixture constant"
    );
}

/// The residual is CLEARED, and this says so where a change would have
/// to notice.
///
/// It runs in the ordinary lane rather than behind the node gate, on
/// purpose: the claim is about what this repository carries, and a claim
/// only a node can check is one nobody checks. The acceptance that
/// cleared it needed a node; the fact that the set no longer holds it
/// does not.
///
/// The assertion is inverted from the one that stood here through three
/// waves. It asserted the residual was still carried, because wiring a
/// signer is not the same as a target accepting what the signer
/// produced. A target has now accepted one, which is the condition the
/// blocker's own defining site named, so the set is one member shorter
/// and this test is what makes a regression say so.
#[test]
fn the_sponsor_residual_is_no_longer_carried() {
    assert!(
        !vectors::live_evidence::carried_residuals().contains(
            &vectors::live_evidence::LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent
        ),
        "the sponsor residual is carried again, and an observed acceptance of a \
         control carrying a sponsor witness cannot be un-observed"
    );
}
