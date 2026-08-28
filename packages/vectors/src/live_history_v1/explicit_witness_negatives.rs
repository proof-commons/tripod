//! Historical explicit witness-negative observations.

use target_elements_conformance::protocol::ObservedOutcomeLayer;

/// The identity the target computed for the accepted control.
///
/// The same one-input one-output candidate the positive table cites,
/// accepted again here after both mutants had been refused — which
/// is what makes each refusal attributable rather than merely
/// recorded.
pub const CONTROL_ACCEPTED_TXID: &str =
    "872a2294da5ea650a7a74ffd8a5932210930ab70d6a08a991eb3ea471ee29abb";

crate::recorded_acceptance::mint_recorded_acceptance!(control_accepted, CONTROL_ACCEPTED_TXID);

/// What the target said to a signature position offering nothing.
///
/// Its own words, verbatim. The offering was empty, so the check
/// that consumed it failed rather than the signature being judged
/// invalid — which is why this row and the malformed one are
/// distinguishable at all.
pub const EMPTY_SIGNATURE_REFUSAL: &str =
    "mandatory-script-verify-flag-failed (Script failed an OP_CHECKSIGVERIFY operation)";

/// How many bytes the empty-signature candidate handed the node.
///
/// Sixty-four fewer than the control, which is the signature that is
/// no longer there.
pub const EMPTY_SIGNATURE_SUBMITTED_BYTES: usize = 529;

/// What the target said to a well-sized offering that is not a
/// signature.
///
/// A DIFFERENT verdict from the empty case, and the difference is
/// what makes each row its own: the width was kept, so the check
/// consumed an item and judged it, and the target named the judgement
/// rather than the arity.
pub const MALFORMED_SIGNATURE_REFUSAL: &str =
    "mandatory-script-verify-flag-failed (Invalid Schnorr signature)";

/// How many bytes the malformed-signature candidate handed the node.
///
/// Exactly the control's count, because only the CONTENT of a
/// well-sized item moved.
pub const MALFORMED_SIGNATURE_SUBMITTED_BYTES: usize = 593;

/// What the earlier, control-first ordering produced.
///
/// Kept rather than deleted, because a register that recorded only
/// the ordering that worked would lose the reason the ordering
/// matters, and a reader reversing it would rediscover this the
/// expensive way.
pub const REFUSAL_UNDER_CONTROL_FIRST_ORDER: &str = "txn-already-known";

/// The layer BOTH witness mutants were refused at, TYPED.
///
/// One constant for the two rows because the layer is the same fact
/// for both: each offering reached the leaf and was judged there, and
/// what separates the rows is the target's WORDS — the arity check
/// against the signature judgement — not where it spoke. Typed so the
/// classifier can check the layer instead of assuming it.
pub const WITNESS_REFUSAL_OBSERVED_LAYER: ObservedOutcomeLayer =
    ObservedOutcomeLayer::ScriptPathRejection;
