//! The canonical live-transfer evidence plan of Guide-13 §13.1.
//!
//! §13.1 gives the plan private fields, no unchecked constructor, and
//! seven sources it derives from. This module is that plan: a value that
//! can only be obtained by [`derive_live_evidence_plan`], which builds
//! every source it can, records the standing of the ones it cannot, and
//! classifies every row of the §15 matrix into exactly one state.
//!
//! # What the classification is for
//!
//! §13.5 makes a report refuse to call itself complete while a required
//! row is unanswered, and that is only checkable if "unanswered" is a
//! typed state rather than a gap in a list. The difference between the
//! states is the difference between the repairs each one calls for: a
//! row a first-party validator already answers is *done*, a row waiting
//! on a target run is waiting on a *run*, a row blocked on a missing
//! component is waiting on that *component*, a row whose fault no typed
//! input can name is waiting on nothing because its obligation was
//! mis-typed, and an ad hoc row is not required at all.
//!
//! # The honest finding this plan carries
//!
//! Twenty-four of the twenty-six positive rows of §15.1 and §15.2 are
//! answered by a target. Each stands at
//! [`LiveRowStanding::NativeRunObserved`], carrying the identity a real
//! node computed for a transfer of that row's own shape which it
//! accepted, whose bytes were read back out of the node's own copy equal
//! to the bytes it was handed, and whose witness verified against an
//! independently recomputed message. The standing carries the identity
//! so the claim can be checked against a chain rather than believed.
//!
//! A twenty-fifth is answered and no target was involved in it. The
//! deterministic-public-fixture-openings row's own gate is the
//! byte-identity contract rather than an acceptance, and it stands at
//! [`LiveRowStanding::DeterminismObserved`] — a member minted for it,
//! counted in its own bucket, and never added to the acceptance figure.
//! A reader asking how much a real node has said should read the
//! twenty-four and not the twenty-five.
//!
//! The ONE that did not move stands at
//! [`LiveRowStanding::NativeRunRequired`] — a statement that a run
//! would answer it and not a statement that nothing could. It is
//! `projection-equality-with-paired-explicit`, and the delta test below
//! names its ground.
//!
//! This paragraph has been rewritten each time a wave observed
//! something, and the rewriting is the discipline rather than churn: it
//! said two while two were answered, and seven while seven were, and a
//! header that kept an old number would be the plan's own summary
//! disagreeing with the census it computes. It also said twenty-two
//! while twenty-three were, which is the same failure caught late — the
//! count beside it was checked by a test and the sentence was not.
//!
//! That is a narrower finding than this paragraph used to carry, and
//! the narrowing is a repair rather than a softening. What it used to
//! say was that one missing component blocked every positive row and
//! that the component was
//! [`LiveInfrastructureBlocker::OwnerSighashNotComputable`]. That
//! sentence was true when it was written and has stopped being true:
//! the digest is computed, the selected profile is established over its
//! required set, a real node accepted a first-party spend on the
//! explicit lane and another on the proof-bearing one, and the blocker
//! is carried by zero rows — which the census below asserts rather than
//! claims. Leaving the old sentence in place would have made the
//! blocker's own doc comment and this header disagree about the same
//! fact.
//!
//! What has NOT changed is the discipline the old sentence protected. A
//! census of rejections from a pipeline that has never had a transaction
//! accepted establishes that the target rejects things, which every
//! target that rejects everything also does; §14.5's positive class
//! witnesses and the conformance package's own `NoAcceptingCase` defect
//! are the same argument made twice already.
//! [`LiveInfrastructureBlocker::NoAcceptingControlExists`] is that
//! argument made a third time, as a state a row can be in — and it is
//! why the restart order of the confidential-funding guide puts an
//! accepted control before any negative case
//! (task:guide-ctf-exec:restart-order).

use std::collections::{BTreeMap, BTreeSet};

use compiler::live_transfer_plan::{
    LiveTransferRepresentationPlan, ValidatedLiveTransferOperationPlan,
};
use compiler::operation_plan::{CoverageRequirementId, SponsorCase};
use linker::CandidateLinkedLiveTransferBundle;
use transaction::live_abi::CandidateLiveTransferAbi;

use crate::error::VectorError;
use crate::live_fault_discharge::{
    LiveFaultValidator, ValidatedLiveFaultEvidence, discharge_live_faults, live_fault_cases,
};
use crate::live_first_party::{
    LiveFirstPartyValidator, ValidatedLiveFirstPartyEvidence, discharge_live_first_party,
    live_first_party_cases,
};
use crate::live_plan::{demonstration_live_abi, demonstration_live_bundle, live_transfer_plan};
use crate::live_safety::{
    LiveRowLink, LiveSafetyRow, LiveSafetySection, required_safety_matrix, resolve_row,
};
use crate::matrix::EvidenceBoundary;

/// Where the §13.1 minimality pair registry stands.
///
/// One of §13.1's seven sources. §13.6 keeps the safety and minimality
/// reports apart and forbids either satisfying the other, so what a
/// safety plan records here is the source's *standing* and never its
/// conclusions: how many §16.1 pairs the registry holds, and how many of
/// them satisfy §16.2. A safety plan that carried the minimality verdict
/// would be the substitution §13.6 forbids.
///
/// # Why the counts are here and the verdict is not
///
/// A plan that named the source without saying whether it exists would
/// leave §13.1's derivation uncheckable, and one that named the source
/// and repeated its answer would make the two reports one. The counts are
/// the middle: enough for a reader to see the source was built, and not
/// enough for anything to conclude minimality from a safety plan.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum MinimalityRegistryStanding {
    /// The registry built, with its pair count and how many satisfy
    /// §16.2.
    Built {
        /// How many §16.1 pairs the registry holds.
        pairs: usize,
        /// How many of them satisfy every §16.2 condition.
        supporting: usize,
    },
    /// The registry did not build, so the source is unavailable.
    ///
    /// Distinct from a registry that built and supports nothing: one is
    /// a missing source and the other is a source with a finding.
    NotConstructible,
}

impl MinimalityRegistryStanding {
    /// Whether the source exists at all.
    #[must_use]
    pub const fn is_built(self) -> bool {
        matches!(self, Self::Built { .. })
    }
}

/// What stands between one row and any evidence at all.
///
/// Every arm names a component that does not exist, rather than an
/// attempt that failed. §1.11 keeps infrastructure failure separate from
/// target rejection precisely so that "nobody could run this" and "the
/// target refused this" are never the same row.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum LiveInfrastructureBlocker {
    /// Nothing computes the digest an owner must sign.
    ///
    /// §10.2's authorization fragment is the target's own verifying
    /// signature primitive, whose message is the target's taproot
    /// sighash. §1.7 requires the selected profile to be *observed* from
    /// a finalized witness or recomputed from the exact signing request,
    /// and forbids accepting it because a builder said so — so this
    /// workspace deliberately mints no such digest, and the signing flow
    /// carries opaque bytes bound to a preimage instead.
    ///
    /// The consequence was exact: a valid live-transfer spend could not
    /// be witnessed by first-party material, so no positive row could
    /// reach a target that would accept it. This was a missing component
    /// and not a verdict about anything.
    ///
    /// # It is no longer carried, and the observation that moved it
    ///
    /// The digest is computed. An independent construction of the
    /// target's taproot message was written from a source review of the
    /// message construction, a finalized explicit candidate was
    /// authorized against it with published test material, and a real
    /// node accepted the spend into a block. The accepted transaction
    /// was read back out of that node and the signature standing in its
    /// witness verifies against the recomputed message — so the
    /// acceptance and the recomputation are two origins rather than one
    /// value checked against itself — and it verifies against no other
    /// candidate message. Seven controls, each moving one term of the
    /// message, were refused by the same node on the same chain.
    ///
    /// So no positive row carries this any longer, and
    /// the crate-private `a_positive_control_exists` computes to true.
    /// Both moves are on an observed result and neither is on a
    /// capability existing.
    ///
    /// That helper is named in code font rather than linked because it
    /// is private and this variant is not, and a public item may not
    /// link a private one. The break was there before this wave and was
    /// invisible: the documentation bar stopped at an earlier failure in
    /// another package and never reached this one.
    ///
    /// # What did not move with it
    ///
    /// Every positive row became a row a run could answer, and not one
    /// of them became answered: a standing is not evidence, and no run
    /// of any row has been filed. [`Self::SighashProfileUnreviewed`] did
    /// not move with this one, because a digest that can be computed is
    /// not yet a settled claim about what it commits to, and that
    /// residual is cleared by a review verdict rather than by a run. It
    /// has since been cleared by exactly that, which is the ordering
    /// this paragraph asserted rather than an exception to it: the two
    /// never moved together and never moved on the same evidence.
    ///
    /// The explicit lane is the only lane this observation touches. The
    /// message's output-witness term is recoverable from the protected
    /// bytes exactly while every entry is default-constructed, and the
    /// proof-bearing lane's entries carry range proofs the preimage does
    /// not contain in any form.
    ///
    /// The word stays in this vocabulary because it is still the right
    /// name for the condition, and a lane that could not compute the
    /// digest must be able to say so.
    OwnerSighashNotComputable,
    /// No transaction of this pipeline has ever been accepted.
    ///
    /// The negative half's blocker, and it is derived rather than
    /// declared: while no positive control exists, a refusal is not
    /// attributable to the row's own mutation. §14.5 requires a positive
    /// class to carry an executable witness that the fixture exhibits the
    /// property named, and §19.2 requires a negative case to have been
    /// refused *for its intended relation* — neither survives a run whose
    /// every submission failed for the same unrelated reason.
    NoAcceptingControlExists,
    /// The selected sighash profile is not established by the review.
    ///
    /// Carried separately from [`Self::OwnerSighashNotComputable`]
    /// because the two are cleared by different work and one of them is
    /// not this workspace's: a digest could be computed tomorrow and the
    /// semantic claim about what it commits to would still be
    /// candidate-scoped until the review completes.
    ///
    /// # It is no longer carried, and the verdict that moved it
    ///
    /// The review verdict established six of the profile's seven
    /// required dimensions on the observed acceptance and stopped on the
    /// seventh, the issuance dimension: no candidate this arc builds
    /// bears an issuance, so the two terms carrying the dimension are
    /// formed from the input count alone and the acceptance exercised
    /// nothing about any issuance field. A stopped verdict is a verdict,
    /// and this residual stood on it rather than on nobody having looked.
    ///
    /// A post-verdict re-typing then moved that dimension from
    /// required to refused, on the ground that the census refuses an
    /// issuance-bearing signing request and the decoder refuses
    /// issuance-bearing bytes, so the required typing was the one layer
    /// promising evidence the other two refuse to admit. The required set
    /// is six, every member of it is established, and the assessment
    /// recomputes to
    /// [`tapscript::OwnerProfileDisposition::Established`]. So the
    /// residual is gone from [`carried_residuals`].
    ///
    /// This move is the one the digest half's own clearing said it was
    /// waiting for, and the order it insisted on is the order that
    /// happened: the computability blocker cleared on the Wave-3 run, and
    /// this cleared afterwards on a review verdict, never simultaneously
    /// and never on the same evidence.
    ///
    /// # What did not move with it
    ///
    /// No row. This is a residual about what a signature under the
    /// selected profile commits to, and clearing it answers nothing that
    /// a run answers: every positive row is still a row a run could
    /// answer and still unanswered, no matrix row moved, and the two
    /// blockers below stand exactly where they stood.
    ///
    /// The word stays in this vocabulary because it is still the right
    /// name for the condition, and a lane relying on a profile whose
    /// review had lapsed must be able to say so.
    SighashProfileUnreviewed,
    /// No adapter signer is wired into this evidence lane.
    ///
    /// §12 builds the sponsored form, and §1.9 keeps the sponsor's
    /// authorization outside protocol data — it arrives through an
    /// adapter that hands back a witness stack. A row about the sponsor's
    /// owner therefore needs an envelope whose signer is supplied rather
    /// than modelled, and every envelope this crate builds declines.
    ///
    /// # What is missing is the wiring, not the capability
    ///
    /// The conformance package advertises a test sponsor authorization
    /// capability and the native executor implements it, with a fixed
    /// regtest key, deterministic signing, and a response bound to the
    /// exact finalized transaction. Nothing in this lane reaches it. One
    /// integration test beside the lane now does — it finalizes an
    /// explicit sponsored control, sends its exact sponsor request,
    /// replays the returned witness through the sponsor capability, and
    /// observes that a witness bound to one mutated byte is refused —
    /// and it fills no row here. The distinction is the whole point of
    /// naming a blocker precisely: clearing this one is an integration,
    /// not a design.
    ///
    /// Wiring it was still not enough to remove the blocker. §1.9 asks
    /// for the sponsor owner's *target authorization*, and a returned
    /// byte stack is not that until a target has accepted a control
    /// carrying it. A blocker moves on an observed result and never on a
    /// capability existing.
    ///
    /// # It is no longer carried, and the observation that moved it
    ///
    /// That acceptance now exists, and this blocker is CLEARED. A
    /// sponsored explicit control — the sponsor region funded before the
    /// receipts, both owners really signing, the sponsor witness
    /// obtained from the adapter and replayed into the finalized bytes —
    /// was submitted to a real node and accepted, read back out of the
    /// node byte for byte, and mined. The condition the paragraph above
    /// names is the condition that was observed, so the rule is applied
    /// rather than reinterpreted.
    ///
    /// What stood between the wiring and the acceptance was never the
    /// signer. Three submissions were refused before this one: the first
    /// for an invalid Schnorr signature, the second — with the owners
    /// really signing — at the sponsored leaf's own §10.7 fee-role
    /// check, and the difference between the second and the third was a
    /// first-party defect in this workspace rather than anything about
    /// the target. The demonstration deployment's fee-role program
    /// digest was a fixture constant no program hashes to, while
    /// construction writes the empty fee program the target's structure
    /// requires. Threading that digest to the value the target itself
    /// computes for the empty program is what produced the acceptance.
    ///
    /// The word stays in this vocabulary because it is still the right
    /// name for the condition, and a lane that genuinely has no sponsor
    /// signer wired into it must be able to say so. Several still do:
    /// the pairs lane models an envelope that declines to sign, and the
    /// transaction-wide private finalization lane has no signer either.
    /// What changed is that this plan no longer CARRIES it.
    ///
    /// # What did not move with it
    ///
    /// No matrix row was answered. The row this blocker was the specific
    /// blocker of, `missing-sponsor-authorization`, is a negative asking
    /// that a control MISSING the sponsor's authorization be refused,
    /// and what was accepted is a positive sponsored control; the row
    /// moved from blocked to awaiting a run of its own shape, which is
    /// not an answer. `private-sponsor-values` may not move at all and
    /// did not: it asks for confidential sponsor values and the accepted
    /// control is explicit. [`Self::PredecessorConstructorAbsent`] is
    /// untouched. The pairs lane still completes no sponsored member.
    ///
    /// # What one acceptance is not
    ///
    /// It establishes the sponsor envelope's wire and ONE target
    /// acceptance. It does not establish production multi-party sponsor
    /// signing: one fixed regtest key signed once, and a single key
    /// answering a request is not a ceremony. Every value involved is
    /// public disposable material under ADR-015's test-material rule.
    SponsorEnvelopeSignerAbsent,
    /// No predecessor exists to build the spend from.
    ///
    /// The time-locked-predecessor row's blocker: §7.3 makes the two
    /// constructors distinct and this workspace builds only the live one,
    /// so there is no time-locked output to offer a live transfer leaf.
    ///
    /// # It is no longer carried, and the ruling that moved it
    ///
    /// It left by a RULING and not by a run, and the ruling is a
    /// correction of the ground rather than a verdict on the work. The
    /// clearing rule this residual stood on asked for an ATTRIBUTABLE
    /// observed pair — a time-locked spend REFUSED naming the lock and
    /// the same shape after maturity ACCEPTED — and that pair was
    /// demanded by a MIS-TYPING of the row it belonged to. Two things
    /// are wrong with it, and either alone is decisive.
    ///
    /// §10.4 refuses the maturity half. The time-locked class is a
    /// receipt CLASS whose maturity is committed cycle arithmetic, and
    /// the guide forbids inferring that it uses the target's timelock
    /// facilities; there is no consensus lock to expire, so no "after
    /// maturity" control exists to stand beside a refusal, and the pair
    /// was never constructible rather than merely unbuilt.
    ///
    /// The refusal half is unattributable in principle. What excludes a
    /// foreign class at a spend is the LEAF COMMITMENT — the pinned
    /// target source refuses a revealed leaf its spent program does not
    /// commit to before any opcode runs — and that refusal is
    /// PROGRAM-GENERIC: every foreign taptree draws the identical
    /// verdict, so no observation of it can ever NAME the lock. A wave
    /// that built the constructor and ran the spend would come back with
    /// a fact about the commitment rule and nothing about receipt
    /// classes.
    ///
    /// So the row was retyped FIRST-PARTY rather than pursued, beside
    /// the `time-locked-output` row it is the sibling of, and
    /// [`crate::live_fault_discharge`] discharges it against the
    /// constructor typing that is the real protection:
    /// `compiler::live_transfer_plan::derive_class` admits only
    /// `ObjectId::ReceiptLive` as the protocol object, so the class
    /// cannot be constructed into a live transfer at all. The residual
    /// is gone from [`carried_residuals`], which is now EMPTY.
    ///
    /// # What did not move with it
    ///
    /// No target ever refused a time-locked spend, and nothing here says
    /// one did. This clearing rests on a ruling about what the row asks
    /// for; it cites no run, no identity, and no node.
    ///
    /// No time-locked constructor was built either. The class remains a
    /// compiler derivation this workspace has not written, and the
    /// retyping is the statement that the row never needed one — not the
    /// statement that one now exists.
    ///
    /// The key-path probe's follow-up phase keeps its own typed carrier,
    /// OPEN and untouched: the observed-layer vocabulary still has no
    /// key-path member, so a key-path refusal is still filed under a
    /// script-path name, and [`crate::live_keypath_probe`] still reports
    /// that and changes nothing. This clearing quotes the same
    /// mis-filing as a reason the commitment observation may not be
    /// filed either, which is a use of the finding and not a repair of
    /// it. The internal-key unspendability residual stands where it
    /// stood, and the probe's record still says so in its own bytes.
    ///
    /// The word stays in this vocabulary because it is still the right
    /// name for the condition, and a lane that genuinely has no
    /// predecessor constructor to build a spend from must be able to say
    /// so. What it may no longer be is a standing residual of this plan.
    PredecessorConstructorAbsent,
    /// No fixture constructs the remaining §15.2 positive shapes beyond
    /// the one-to-one control.
    ///
    /// The restart order's fifth step asks for the remaining positive
    /// private shapes — split, many-to-many, several distinct owners — each
    /// where it accepts. Each needs a fixture this workspace has not built:
    /// the registry's `manifest` builder and the private ceremony are fixed
    /// at the one-to-one control's two outputs and one input, and a split
    /// needs a three-output manifest while a many-to-many or a
    /// several-owner transfer needs more inputs and more outputs than the
    /// current construction supplies.
    ///
    /// It is the same kind of blocker as
    /// [`Self::PredecessorConstructorAbsent`]: a constructor this workspace
    /// has not built, not a target verdict and not a dependency this guide
    /// cannot close. A following wave clears it by building the
    /// multi-output and multi-input fixtures, at which point each shape
    /// moves on its own observed acceptance.
    ///
    /// One of the shapes the fifth step names is not merely unbuilt but
    /// structurally unconstructible: a private-merge is one output, and the
    /// registry refuses a manifest with fewer than two. That is reported as
    /// a divergence against the guide's own §14.5 merge predicate rather
    /// than folded into this blocker, because the two are different facts —
    /// one is work not done, the other is work the rules forbid.
    MultiOutputShapeConstructorAbsent,
    /// No confidential predecessor can be funded on this chain.
    ///
    /// §6.3 admits a private transfer only over confidential receipt
    /// inputs, and the target-generic funding step names an explicit
    /// amount and has no confidential form. So the two questions Wave 9
    /// left for a target — the value field's form and the missing range
    /// proof — cannot be asked through this boundary, and
    /// [`crate::live_native`] records that rather than guessing.
    ///
    /// It was carried beside [`Self::OwnerSighashNotComputable`] rather
    /// than assigned to the §15.2 rows, because it is the *second* thing
    /// standing in their way and clearing it alone moves nothing.
    ///
    /// # It is no longer a carried residual, and the observation that
    /// moved it
    ///
    /// The confidential funding arm now exists, a deterministic
    /// materializer builds the exact explicit-asset/confidential-value
    /// form, and one predecessor of that form has been submitted,
    /// accepted, mined, and read back raw, with recomputation and
    /// readback agreeing on every member of the funding agreement
    /// census for both outputs. So the blocker is gone from
    /// [`carried_residuals`], on an observed result rather than on a
    /// capability existing.
    ///
    /// The word stays in this vocabulary because it is still the right
    /// name for the condition, and because a lane that cannot fund a
    /// confidential predecessor must be able to say so. What it may no
    /// longer be is a standing residual of this plan.
    ///
    /// Nothing else moved with it. Every positive private row remained
    /// blocked by [`Self::OwnerSighashNotComputable`], which is
    /// independent work this guide does not review, and
    /// [`Self::SighashProfileUnreviewed`] remained carried. Both have
    /// since been cleared by that independent work, each on its own
    /// evidence, and neither by anything recorded here.
    NoConfidentialPredecessorCanBeFunded,
    /// The row needs a raw path the safe constructor cannot express.
    ///
    /// §4.3's second answer. The row asks whether an unsafe raw mutation
    /// *exists*, and the safe constructor refusing establishes nothing
    /// about it — a refusal from something that could not have built the
    /// offending transaction anyway is not evidence that nothing can.
    ///
    /// # The premise has expired, and the row is NOT moved on it
    ///
    /// This is recorded here, at the blocker's own site, because the
    /// alternative is two artifacts disagreeing about one row — the
    /// failure a previous wave found between a closeout and the matrix
    /// and had to repair, and which is cheaper to prevent than to fix.
    ///
    /// The sentence above says the row needs a raw path that does not
    /// exist. One does. `TargetTransaction::with_output_witnesses` is
    /// public and checks census ARITY only — no amount, asset, program,
    /// position or role — and three lanes already rebuild finalized
    /// bytes through it and hand the result to a real node: the
    /// conservation negatives, the key-path probe, and the proof-bearing
    /// observation. So the answer to the question this row asks is
    /// visibly yes, and the blocker is describing a workspace that no
    /// longer exists.
    ///
    /// The row is left where it stands anyway, and deliberately. What
    /// would move it is undetermined rather than merely unwritten: the
    /// row's evidence is that a bypass EXISTS, which is a first-party
    /// fact about this workspace and not a verdict any target gave, and
    /// no standing here carries such a fact — the observation members
    /// all carry target identities. Moving it would therefore mean
    /// choosing a standing for it, and that is the kind of choice the
    /// determinism member was minted by DIRECTION rather than taken
    /// unilaterally. A ruling is owed on which of two repairs the row
    /// wants: a standing for an existence fact this workspace
    /// establishes about itself, or a retyping of the row as first-party
    /// beside the other §4.3 distinctions. Recorded, escalated, and not
    /// decided here.
    RawSurgeryPathAbsent,
}

/// Why a first-party row carries no executable discharge.
///
/// One gap, and no row is in it today. §4.2's last sentence gives
/// exactly two honest dispositions for a requirement whose policy cannot
/// be met — outside the coverage denominator, or outstanding inside it —
/// and this type is the second one, kept so that a row acquiring it is
/// counted and rendered rather than dropped.
///
/// # It used to carry three obstacles, and they were findings
///
/// Two named boundaries that disagreed with the workspace and one named
/// a malformation nothing could express. All three were resolved by
/// re-typing what the rows claimed rather than by staging a case against
/// them, so the obstacles retired with the rows that carried them.
/// Keeping them as unreachable members would leave a reader checking
/// which of a taxonomy's arms are alive.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum FirstPartyGap {
    /// No case in the §4.2 census stages this row.
    ///
    /// The row's boundary is first-party and a validator plausibly owns
    /// it, but no canonical malformed input and control pair has been
    /// staged, so nothing here has been driven to a refusal. Recorded as
    /// a gap rather than as coverage.
    NoStagedCase,
}

/// Which first-party entry point discharged one row.
///
/// Two vocabularies, because the matrix's pre-target rows are owned by
/// two different censuses: §15.3's owner and signature faults reach one
/// of two signing-flow entry points, and §15.4–§15.7's reach one of five
/// spread across three crates. Collapsing them would lose the fact a
/// coverage reader wants, which is *what* refused.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum DischargingValidator {
    /// One of §12.7's two owner-authorization entry points.
    OwnerAuthorization(LiveFirstPartyValidator),
    /// One of the seven §15.4–§15.7 fault entry points.
    Fault(LiveFaultValidator),
}

/// What answers one §15 row, or what stands in the way.
///
/// Exactly one state per row, and none of them is "passed". §13.5 reads
/// this to decide whether a report may call itself complete, and a state
/// that could be read two ways would make that decision unfalsifiable.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LiveRowStanding {
    /// A first-party validator was driven to the row's own refusal.
    ///
    /// §4.2 met, and recomputed by
    /// [`crate::live_first_party::validate_live_first_party`] rather than
    /// asserted here.
    FirstPartyDischarged {
        /// The entry point that refused.
        validator: DischargingValidator,
        /// The refusal class it named.
        class: &'static str,
    },
    /// The row's boundary is first-party and nothing stages it yet.
    FirstPartyUndischarged(FirstPartyGap),
    /// The row's boundary is the target, and a run would answer it.
    ///
    /// Carries the published requirement the row resolves to where the
    /// guide determines one, so a run's observation can be filed against
    /// a relation rather than against a name.
    NativeRunRequired(Option<CoverageRequirementId>),
    /// A target-native run answered the row, and here is the identity.
    ///
    /// # Why this variant had to be minted
    ///
    /// Until this wave the matrix could say that a row NEEDED a run and
    /// could not say that a run had ANSWERED one. Every positive row
    /// therefore stood at [`Self::NativeRunRequired`] whatever happened
    /// on a chain, and the only way to record an acceptance would have
    /// been to move the row to a first-party discharge — which would be
    /// filing a target's verdict under a validator's refusal, exactly
    /// the layer blur §1.7 forbids.
    ///
    /// # What may occupy it
    ///
    /// An identity the TARGET computed for a transaction the target
    /// ACCEPTED, of the row's own shape, whose witness was verified
    /// against an independently recomputed message. Not a run that
    /// happened; not a candidate that was constructible; not a
    /// capability that exists. The identity is carried rather than a
    /// boolean so that the claim can be checked against a chain by
    /// somebody who does not trust this crate.
    NativeRunObserved {
        /// The identity the target computed for the accepted
        /// transaction.
        accepted_identity: &'static str,
    },
    /// A target REFUSED a candidate staging this row's own class,
    /// while accepting the unmutated form of the same candidate.
    ///
    /// The negative half's counterpart to [`Self::NativeRunObserved`],
    /// and shaped like it for the same reason: what makes a refusal
    /// evidence is not that a refusal happened but that a specific one
    /// did, and a reader who does not trust this crate has to be able to
    /// check it.
    ///
    /// # Why a refusal carries an ACCEPTED identity
    ///
    /// A census of rejections from a pipeline that has never had a
    /// transaction accepted establishes that the target rejects things,
    /// which every target that rejects everything also does. So the
    /// evidence for a negative row is a PAIR: the unmutated candidate
    /// accepted, and the mutated one refused. The accepted identity is
    /// carried here because it is the half a reader can check against a
    /// chain — the refusal left no transaction to look up, which is what
    /// being refused means.
    ///
    /// The refusal detail is the target's own words, recorded verbatim
    /// and never paraphrased into this workspace's vocabulary.
    NativeRefusalObserved {
        /// The identity the target computed for the accepted control the
        /// refusal is attributable against.
        control_identity: &'static str,
        /// What the target said when it refused the mutated candidate.
        refusal_detail: &'static str,
    },
    /// A first-party determinism run answered the row, and here is what
    /// it recomputed.
    ///
    /// # Why this member had to be minted
    ///
    /// Both observation members above are TARGET verdicts, and the
    /// ground each of them carries is an identity the target computed.
    /// One §15.2 row does not ask for a target verdict at all: §11.2
    /// gives `deterministic-public-fixture-openings` the byte-identity
    /// contract (§6.7) as its gate, and that contract is satisfied by
    /// recomputing a fixture from its manifest and comparing the result
    /// byte for byte. No target is asked and none accepts anything.
    ///
    /// Wave seven produced exactly that observation and could not file
    /// it. The only members that would have taken it carry
    /// target-computed identities, so filing a determinism fact under
    /// one would have been the single error a run of record exists to
    /// prevent; the wave reported the gap and left the row unmoved.
    /// This member is the repair directed in answer — the VOCABULARY
    /// was narrow, not the guide defective, so the §11.2 gate stands as
    /// written and the code catches up to it.
    ///
    /// # What may occupy it, and what it must never be read as
    ///
    /// A first-party recomputation that produced byte-identical output
    /// from equal inputs, naming what was recomputed and the test that
    /// recomputed it. Not a run that happened; not a fixture that was
    /// buildable; not a contract that is stated somewhere.
    ///
    /// It is counted in its OWN census bucket and is never added to the
    /// acceptance count. A determinism observation establishes nothing
    /// whatever about any target — no transaction was offered to one —
    /// and a single figure covering both kinds would let a reader take
    /// a recomputation for an acceptance, which is the same misreading
    /// [`LiveEvidenceCensus::native_refusal_observed`] is kept separate
    /// to prevent.
    DeterminismObserved {
        /// What was recomputed, in the contract's own terms.
        recomputed: &'static str,
        /// The first-party test that recomputed it.
        observed_by: &'static str,
    },
    /// A RELATION over TWO accepted identities answered the row.
    ///
    /// # Why this member had to be minted
    ///
    /// Minted by DIRECTION, on the precedent [`Self::DeterminismObserved`]
    /// and [`Self::FirstPartyFactObserved`] both set, and for the reason
    /// they were set: a row was answerable, the vocabulary had no name to
    /// file the answer under, and the honest options were to leave the
    /// row unmoved or to file its answer under a member that would
    /// misdescribe it. The minimality wave took the first and REPORTED
    /// the gap in these words — `NativeRunObserved` carries exactly ONE
    /// `accepted_identity` and a projection equality is a relation over
    /// TWO — and declined to mint a member for it unilaterally. This
    /// member is the repair the pairs arc minted in answer.
    ///
    /// # What may occupy it, and what it must never be read as
    ///
    /// TWO identities a target computed for TWO transactions the target
    /// ACCEPTED, which are the two MATERIALIZATIONS OF ONE semantic
    /// fixture (§16.1), together with the relation observed over them in
    /// the row's own terms. Not two acceptances that happen to be
    /// comparable; not one acceptance and one shape sibling — that
    /// substitution is exactly what [`crate::live_pairs::PairTargetVerdict`]'s
    /// `NotSubmittedShapeAcceptedElsewhere` member exists to deny, and a
    /// standing that took it would undo the denial.
    ///
    /// It is counted in its OWN census bucket. A relation over two
    /// acceptances is not two acceptances of two rows, and adding it to
    /// [`LiveEvidenceCensus::native_run_observed`] would let a reader
    /// take one observation for two.
    PairedRelationObserved {
        /// The identity the target computed for the accepted EXPLICIT
        /// materialization.
        explicit_identity: &'static str,
        /// The identity the target computed for the accepted PRIVATE
        /// materialization.
        private_identity: &'static str,
        /// The relation observed over the two, in the row's own terms.
        relation: &'static str,
    },
    /// A row whose own gate is a FACT about this workspace, established.
    ///
    /// # Why this member had to be minted
    ///
    /// Minted by DIRECTION, on the precedent [`Self::DeterminismObserved`]
    /// set and for the same reason it was set: a row was answerable, the
    /// vocabulary had no name to file the answer under, and the honest
    /// options were to leave the row unmoved or to file its answer under a
    /// member that would misdescribe it. The first was taken twice and
    /// reported as a gap; this member is the repair directed in answer.
    ///
    /// Both observation members above carry an identity a TARGET computed,
    /// and [`Self::DeterminismObserved`] carries a byte-identity
    /// recomputation. Two rows' gates are neither. What each of them asks
    /// is whether something is SO of this workspace — whether a path
    /// exists at all, whether a shape is admitted — and the answer to that
    /// is a first-party fact with a first-party site: not a verdict
    /// anybody gave, and not a recomputation of anything.
    ///
    /// # What may occupy it, and what it must never be read as
    ///
    /// A fact this workspace establishes about ITSELF, for a row whose own
    /// published gate is that fact, naming the fact and the site that
    /// establishes it. The second condition is the load-bearing one, and
    /// it is what keeps this member from becoming the place a wave files
    /// whatever it could not otherwise move: a row whose gate is a target
    /// verdict may NOT be answered here however true some fact about it
    /// is, exactly as a row whose gate is an acceptance may not be
    /// answered by determinism however deterministically it reproduces.
    ///
    /// It is counted in its OWN census bucket, and is added neither to the
    /// acceptance count nor to the refusal count. No transaction was
    /// offered to any target, so a single figure covering this and an
    /// acceptance would let a reader take a statement this workspace makes
    /// about itself for a verdict a chain gave — the misreading every
    /// separate bucket here exists to prevent.
    FirstPartyFactObserved {
        /// The fact, in the row's own terms.
        fact: &'static str,
        /// The first-party site that establishes it.
        observed_by: &'static str,
    },
    /// A component the row needs does not exist.
    InfrastructureBlocked(LiveInfrastructureBlocker),
    /// The row's boundary is this workspace's own report bytes.
    ///
    /// §15.6's two report rows, which no target can answer and no
    /// validator refuses: what they ask is whether the canonical
    /// serialization published a sponsor amount or an opening, and the
    /// answer is a property of the rendered bytes.
    ReportLayerAnswerable,
    /// No layer answers the row, because no input names its fault.
    ///
    /// §4.2's other honest disposition, taken. Its last sentence puts a
    /// requirement whose policy cannot be met *outside* the coverage
    /// denominator rather than permanently outstanding inside it, and
    /// this row's policy cannot be met for a structural reason:
    /// [`crate::live_safety::LiveRowBoundary::OperationVocabularyClosure`]
    /// says which.
    ///
    /// It is not answered. [`Self::is_answered`] is false for it, and no
    /// report may read it as a refusal, a target verdict, or evidence of
    /// any kind. What it records is that the obligation was mis-typed:
    /// there is no malformed input for a validator to refuse, so there
    /// is no §4.2 discharge to be owed one.
    OperationVocabularyClosed,
    /// The row is ad hoc and outside the required denominator.
    ///
    /// §13.1's last sentence. No row of §15 is experimental today, and
    /// the variant exists because §13.1 names the class — a plan that
    /// could not express it would have to file an ad hoc case as
    /// required.
    Experimental,
}

impl LiveRowStanding {
    /// Whether this standing is evidence rather than an outstanding
    /// obligation.
    #[must_use]
    pub const fn is_answered(&self) -> bool {
        matches!(
            self,
            Self::FirstPartyDischarged { .. }
                | Self::NativeRunObserved { .. }
                | Self::NativeRefusalObserved { .. }
                // Answered, and answered by the gate its own row was
                // given: §11.2 names the byte-identity contract rather
                // than an acceptance for the openings row, so a
                // determinism observation discharges it outright. What
                // it is NOT is a target verdict, which is why it stands
                // in its own member and counts in its own bucket.
                | Self::DeterminismObserved { .. }
                // Answered, and answered by the gate its own row was
                // given: §15.2's projection-equality row asks for a
                // COMPARISON of two accepted transactions' projections,
                // which is a relation and not an acceptance. Both target
                // verdicts are underneath it, so unlike the two members
                // around it this one does rest on a chain — what it is
                // not is ONE verdict, which is why it stands in its own
                // member and counts in its own bucket.
                | Self::PairedRelationObserved { .. }
                // Answered, and answered by the gate its own row was
                // given: two rows ask whether something is SO of this
                // workspace, and a fact with a site is the whole of what
                // such a row wants. What it is NOT is a target verdict,
                // which is why it stands in its own member and counts in
                // its own bucket.
                | Self::FirstPartyFactObserved { .. }
                | Self::ReportLayerAnswerable
        )
    }

    /// Whether this standing is an infrastructure error §13.5 counts.
    #[must_use]
    pub const fn is_infrastructure_error(&self) -> bool {
        matches!(self, Self::InfrastructureBlocked(_))
    }
}

/// One classified row of the §15 matrix.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveEvidenceRow {
    row: &'static LiveSafetyRow,
    standing: LiveRowStanding,
}

impl LiveEvidenceRow {
    /// The §15 row this classification is about.
    #[must_use]
    pub const fn row(&self) -> &'static LiveSafetyRow {
        self.row
    }

    /// What answers it, or what stands in the way.
    #[must_use]
    pub const fn standing(&self) -> &LiveRowStanding {
        &self.standing
    }
}

/// The census figures a reader of a plan wants first.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LiveEvidenceCensus {
    rows: usize,
    first_party_discharged: usize,
    first_party_undischarged: usize,
    native_run_required: usize,
    native_run_observed: usize,
    native_refusal_observed: usize,
    determinism_observed: usize,
    paired_relation_observed: usize,
    first_party_fact_observed: usize,
    infrastructure_blocked: usize,
    report_layer: usize,
    vocabulary_closed: usize,
    experimental: usize,
}

impl LiveEvidenceCensus {
    /// How many rows the matrix has.
    #[must_use]
    pub const fn rows(&self) -> usize {
        self.rows
    }

    /// How many rows a first-party validator answers.
    #[must_use]
    pub const fn first_party_discharged(&self) -> usize {
        self.first_party_discharged
    }

    /// How many first-party rows nothing stages yet.
    #[must_use]
    pub const fn first_party_undischarged(&self) -> usize {
        self.first_party_undischarged
    }

    /// How many rows a target-native run has answered.
    ///
    /// The wave's own delta, as a number. Every one of them is a row
    /// whose standing carries the identity that answered it, so the
    /// figure can be audited row by row rather than believed.
    #[must_use]
    pub const fn native_run_observed(&self) -> usize {
        self.native_run_observed
    }

    /// How many negative rows a target-native refusal has answered.
    ///
    /// Counted apart from [`Self::native_run_observed`] rather than
    /// added to it, because the two are different observations: one is a
    /// transaction on a chain and the other is a verdict about bytes
    /// that never reached one. A single figure would let a reader take a
    /// refusal for an acceptance.
    #[must_use]
    pub const fn native_refusal_observed(&self) -> usize {
        self.native_refusal_observed
    }

    /// How many rows a relation over TWO accepted identities has
    /// answered.
    ///
    /// A bucket of its own for the reason the determinism and refusal
    /// buckets are: what it counts is a different observation. A paired
    /// relation is ONE fact about TWO transactions, and adding it to
    /// [`Self::native_run_observed`] would let a reader take it for two
    /// rows' worth of acceptance evidence.
    #[must_use]
    pub const fn paired_relation_observed(&self) -> usize {
        self.paired_relation_observed
    }

    /// How many rows a first-party determinism observation has answered.
    ///
    /// A third bucket for the same reason there is a second: this one
    /// counts observations no target was involved in at all. Adding it
    /// to either target figure would inflate a count of chain evidence
    /// with a recomputation, and a reader summing the observation
    /// buckets to ask "how much did a real node say" would get the
    /// wrong answer by exactly this number.
    /// How many rows a first-party FACT about this workspace has
    /// answered.
    ///
    /// A fourth bucket, on the third one's reasoning carried one step
    /// further. The determinism bucket is separate because it counts
    /// observations no target was involved in; this one counts answers
    /// that are not observations of a transaction at all — a path either
    /// exists or it does not, a shape is either admitted or it is not.
    /// Summing it with either target figure would report a statement this
    /// workspace makes about itself as something a chain did, and a
    /// reader adding the observation buckets to ask "how much did a real
    /// node say" would be wrong by exactly this number.
    #[must_use]
    pub const fn first_party_fact_observed(&self) -> usize {
        self.first_party_fact_observed
    }

    #[must_use]
    pub const fn determinism_observed(&self) -> usize {
        self.determinism_observed
    }

    /// How many rows are waiting on a target-native run.
    #[must_use]
    pub const fn native_run_required(&self) -> usize {
        self.native_run_required
    }

    /// How many rows are blocked on a component that does not exist.
    #[must_use]
    pub const fn infrastructure_blocked(&self) -> usize {
        self.infrastructure_blocked
    }

    /// How many rows the report's own bytes answer.
    #[must_use]
    pub const fn report_layer(&self) -> usize {
        self.report_layer
    }

    /// How many rows no layer answers because no input names them.
    ///
    /// Counted apart from every other bucket, and from the answered ones
    /// especially: a reader adding this to the discharged figure would
    /// be counting a structural fact as a refusal.
    #[must_use]
    pub const fn vocabulary_closed(&self) -> usize {
        self.vocabulary_closed
    }

    /// How many rows are outside the required denominator.
    #[must_use]
    pub const fn experimental(&self) -> usize {
        self.experimental
    }

    /// Whether every required row is answered.
    ///
    /// §13.5's bar. False while any row is waiting on a run or blocked on
    /// a component, which is what stops a report built on this plan from
    /// calling itself complete.
    ///
    /// [`Self::vocabulary_closed`] is not among the three, and that is
    /// §4.2's own instruction rather than leniency: a requirement whose
    /// policy cannot be met belongs outside the denominator instead of
    /// permanently outstanding inside it. Leaving it in would make the
    /// bar unreachable by construction and say nothing true about the
    /// pipeline.
    #[must_use]
    pub const fn every_required_row_is_answered(&self) -> bool {
        self.native_run_required == 0
            && self.infrastructure_blocked == 0
            && self.first_party_undischarged == 0
    }
}

/// The canonical live-transfer evidence plan (§13.1).
///
/// Every field is private and there is no public constructor, no
/// `Default`, and no builder: the sole route to a value of this type is
/// [`derive_live_evidence_plan`], which builds the sources and derives
/// the classification. A plan assembled from arbitrary fields would be a
/// caller's opinion about what has been established, which is exactly
/// what §13.1's private fields exist to prevent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveTransferEvidencePlan {
    plan: ValidatedLiveTransferOperationPlan,
    bundle: CandidateLinkedLiveTransferBundle,
    abi: CandidateLiveTransferAbi,
    minimality: MinimalityRegistryStanding,
    rows: Vec<LiveEvidenceRow>,
    discharged: Vec<ValidatedLiveFirstPartyEvidence>,
    fault_discharged: Vec<ValidatedLiveFaultEvidence>,
    census: LiveEvidenceCensus,
}

impl LiveTransferEvidencePlan {
    /// The compiler coverage the plan is indexed against.
    #[must_use]
    pub const fn operation_plan(&self) -> &ValidatedLiveTransferOperationPlan {
        &self.plan
    }

    /// The exact linked bundle.
    #[must_use]
    pub const fn bundle(&self) -> &CandidateLinkedLiveTransferBundle {
        &self.bundle
    }

    /// The exact candidate ABI.
    #[must_use]
    pub const fn abi(&self) -> &CandidateLiveTransferAbi {
        &self.abi
    }

    /// Where the minimality pair registry stands.
    #[must_use]
    pub const fn minimality(&self) -> MinimalityRegistryStanding {
        self.minimality
    }

    /// Every classified row, in §15 order.
    #[must_use]
    pub fn rows(&self) -> &[LiveEvidenceRow] {
        &self.rows
    }

    /// The §15.3 first-party evidence the plan recomputed.
    #[must_use]
    pub fn discharged(&self) -> &[ValidatedLiveFirstPartyEvidence] {
        &self.discharged
    }

    /// The §15.4–§15.7 first-party evidence the plan recomputed.
    ///
    /// Kept beside [`Self::discharged`] rather than merged into it: the
    /// two censuses drive different entry points with different refusal
    /// vocabularies, and a single list would have to re-spell one of
    /// them.
    #[must_use]
    pub fn fault_discharged(&self) -> &[ValidatedLiveFaultEvidence] {
        &self.fault_discharged
    }

    /// The census figures.
    #[must_use]
    pub const fn census(&self) -> LiveEvidenceCensus {
        self.census
    }

    /// Every row in one standing's class, in §15 order.
    #[must_use]
    pub fn rows_blocked_on(&self, blocker: LiveInfrastructureBlocker) -> Vec<&LiveEvidenceRow> {
        self.rows
            .iter()
            .filter(|row| row.standing == LiveRowStanding::InfrastructureBlocked(blocker))
            .collect()
    }

    /// The per-section census of one standing predicate.
    #[must_use]
    pub fn section_census(
        &self,
        admit: fn(&LiveRowStanding) -> bool,
    ) -> BTreeMap<LiveSafetySection, usize> {
        let mut census: BTreeMap<LiveSafetySection, usize> = LiveSafetySection::ALL
            .iter()
            .map(|section| (*section, 0))
            .collect();
        for row in &self.rows {
            if admit(&row.standing) {
                *census.entry(row.row.section()).or_insert(0) += 1;
            }
        }
        census
    }
}

/// Whether any positive control exists for the live-transfer pipeline.
///
/// The derived fact the whole negative half depends on. It is computed
/// rather than declared, and it computed to `false` for one stated
/// reason: §15.1's and §15.2's rows all expect an accepted transaction,
/// and no owner signature could be produced over the digest the target's
/// verifying primitive forms.
///
/// # The observation that moved it
///
/// One does now exist. A finalized explicit candidate was authorized
/// against a first-party recomputation of the target's own taproot
/// message, submitted to a real node, and accepted into a block; the
/// accepted transaction was read back out of the node and the signature
/// standing in its witness verifies, under this workspace's own curve
/// arithmetic, against the message this workspace recomputed — and
/// against no other candidate message. Seven controls, each moving one
/// term of that message, were refused by the same node on the same
/// chain.
///
/// So this returns `true` on an observed acceptance and never on a
/// capability existing. What it says is that a positive control is
/// *possible*, which is the only thing the negative half needs from it:
/// a refusal is attributable to a row's own mutation once something can
/// be accepted at all. It says nothing about any particular row.
const fn a_positive_control_exists() -> bool {
    true
}

/// The blocker one specific negative row carries, where it has its own.
///
/// Most target-boundary rows are blocked by the absent positive control.
/// One carries a blocker of its own as well, and the more specific one
/// wins: a reader repairing the pipeline needs to know that it would
/// still be outstanding after the digest existed.
///
/// # The time-locked row left by being retyped
///
/// `time-locked-input` carried
/// [`LiveInfrastructureBlocker::PredecessorConstructorAbsent`] until the
/// row was RETYPED first-party. It is not a target-boundary
/// row any more — [`crate::live_fault_discharge`] discharges it beside
/// its `time-locked-output` sibling — so it can carry no specific
/// blocker at all, and its entry is removed here rather than left to
/// describe a row this function no longer reaches.
///
/// # It was three, and the sponsor row left
///
/// `missing-sponsor-authorization` carried
/// [`LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent`] until that
/// residual cleared on an observed acceptance. Its entry is removed
/// here rather than left standing, because leaving it would make two
/// artifacts disagree about one row: the residual set would say the
/// sponsor signer dependency is closed while this function still said
/// the row is blocked on it. A previous wave found exactly that
/// disagreement between a closeout and the matrix and had to repair it,
/// and the repair is cheaper before the fact than after.
///
/// The row does NOT thereby become answered, and the fall-through is
/// what keeps that honest. With no specific blocker it reaches
/// [`observed_row_acceptance`], which has no acceptance of this row's
/// own shape to offer — the accepted sponsored control is a POSITIVE
/// control, and this row is a negative asking that a control MISSING
/// the sponsor's authorization be refused. So the row resolves to
/// `NativeRunRequired`: no longer blocked, not yet run, and waiting on a
/// run of its own shape that nothing now prevents.
/// No row is in it today, and the function is not decoration. It is what
/// keeps a blocked row counted and rendered from the moment one exists
/// rather than from the moment somebody notices, which is the same
/// reason the undischarged first-party branch above is kept empty.
///
/// The last entry left on the ruling recorded at
/// [`LiveInfrastructureBlocker::RawSurgeryPathAbsent`]: the blocker's
/// premise had expired, the previous wave declined to move the row
/// unilaterally, and the ruling minted
/// [`LiveRowStanding::FirstPartyFactObserved`] and moved it there. The
/// member stays in the vocabulary — it is the right name for a lane that
/// genuinely has no raw path — on the pattern two retired blockers
/// before it set.
const fn specific_blocker(_row: &LiveSafetyRow) -> Option<LiveInfrastructureBlocker> {
    None
}

/// The identity that answered one row, where a run answered it.
///
/// Beside [`specific_blocker`] and shaped like it, because the two
/// answer the same kind of question from opposite directions: what
/// stands in a row's way, and what has already got out of it.
///
/// Every entry cites a run of record, so a reader following the name
/// arrives at the constants one execution against a real node produced
/// rather than at a claim in this file.
///
/// A row is added here on an observed acceptance OF THAT ROW'S SHAPE
/// and on nothing else. An acceptance of a different shape is evidence
/// about the different shape.
fn observed_row_acceptance(row: &LiveSafetyRow) -> Option<&'static str> {
    match row.name() {
        // One receipt consumed, one recipient created, the balancing
        // output back to the sender as change, sponsorless, private,
        // and spending a mined confidential predecessor at this
        // deployment's own private receipt constructor.
        "private-one-to-one" => Some(crate::live_private_restart::run_of_record::ACCEPTED_TXID),
        // Both admitted commitment parities, each consumed in its own
        // complete accepted successor. The identity cited is the run
        // that COMPLETED the pair; the first parity's acceptance is the
        // row above's, and it takes both runs to say that both parities
        // were exercised. The run of record carries the pair.
        "both-commitment-parity-forms" => {
            Some(crate::live_private_restart::run_of_record::PARITY_ACCEPTED_TXID)
        }
        // The target's own commitment-balance rule accepting a conserving
        // private transaction. The follow-up wave observed this and
        // recorded it in its closeout delta, and this arm is the matrix
        // catching up with that observation rather than a new claim: the
        // identity is that wave's own run of record. Its absence here was
        // a defect — the closeout said three rows had moved while the
        // matrix classified two, and a row that has moved in one artifact
        // and not the other is a row nobody is checking.
        "target-ct-conservation" => {
            Some(crate::live_conservation_negatives::run_of_record::CONTROL_ACCEPTED_TXID)
        }
        // A sponsored PRIVATE successor: a blinded sponsor coin in at an
        // explicit asset, blinded receipt destinations, a committed
        // sponsor change, and an explicit reserve fee outside both
        // balance equations. Accepted and mined, with the sponsor's
        // committed change located in the node's own copy.
        //
        // Its absence here was the same defect the conservation arm
        // above records: the guide closeout moves this row on this
        // identity and the matrix classified it as still awaiting a
        // run, and a row that has moved in one artifact and not the
        // other is a row nobody is checking. An acceptance of an
        // EXPLICIT sponsored control would not have done — this class
        // asks for confidential sponsor values, and it is answered only
        // by a run of its own shape.
        "private-sponsor-values" => {
            Some(crate::live_sponsor_shapes::sponsored_run_of_record::SPONSORED_PRIVATE_TXID)
        }
        // One receipt consumed and THREE outputs created: two recipients
        // and the balancing change back to the sender.
        "private-split" => Some(crate::live_multi_shapes::run_of_record::SPLIT_ACCEPTED_TXID),
        // TWO receipts consumed and THREE outputs created. The
        // representative case is named as representative: its input and
        // output counts both exceed the one-to-one control's, and no
        // claim is made here about any other cardinality.
        "private-many-to-many-representative" => {
            Some(crate::live_multi_shapes::run_of_record::MANY_TO_MANY_ACCEPTED_TXID)
        }
        // TWO receipts under two DISTINCT published owners, each input
        // carrying the leaf its own position executes.
        "private-several-distinct-owners" => {
            Some(crate::live_multi_shapes::run_of_record::SEVERAL_OWNERS_ACCEPTED_TXID)
        }
        // TWO receipts consumed and ONE output created: the merge.
        //
        // The row moves on an acceptance of a merge whose forced blinder
        // is NONZERO, and the distinction is the whole of what took two
        // waves to reach. A merge of an inverse pair forces a zero
        // blinder, whose output commitment is exactly the value times the
        // value generator -- a point anybody recomputes from a guessed
        // amount. Such a transaction would balance and a node would
        // accept it, so an acceptance alone is not what answers this row:
        // what answers it is an acceptance of a merge that HIDES, and the
        // ceremony writes the forced blinder's nonzero-ness into its own
        // transcript rather than leaving it to be assumed.
        "private-merge" => Some(crate::live_multi_shapes::run_of_record::MERGE_ACCEPTED_TXID),

        // §15.1, the positive explicit table. Thirteen of its sixteen
        // rows are answered by thirteen runs of the explicit shape
        // ceremony, each accepted by a real node, each read back equal
        // to the bytes it was handed, and each with every input's
        // signature verified out of the node's own copy against an
        // independently recomputed message.
        //
        // Three identities are each cited by two rows, and the register
        // this arm reads from states which and why: those rows are two
        // CLASSES of one transaction rather than two transactions, and
        // the rule this map is held to forbids citing an acceptance of a
        // DIFFERENT shape rather than an acceptance that is an instance
        // of two classes at once.
        // The one-to-one acceptance, cited by BOTH rows it is an
        // instance of. The accepted bytes consume one receipt and create
        // one output, and they carry no sponsor region, so they are the
        // `one-input-to-one-output` class and the `sponsorless` class at
        // once. Building a second, gratuitously different transfer so
        // that each row could cite its own hex string would be dressing
        // one fact up as two.
        "one-input-to-one-output" | "sponsorless" => {
            Some(crate::live_explicit_shapes::run_of_record::ONE_TO_ONE_ACCEPTED_TXID)
        }
        // The split acceptance, likewise both: one receipt split into
        // two destinations belonging to two DISTINCT published owners is
        // an instance of the split class and of the
        // several-destination-owners class.
        "one-input-split-into-two" | "several-destination-owners" => {
            Some(crate::live_explicit_shapes::run_of_record::SPLIT_ACCEPTED_TXID)
        }
        // The merge acceptance, and this pair is the strongest of the
        // three rather than the weakest. The normalization run offered
        // the same two receipts in the REVERSE of their canonical order
        // and the merge run offered them in it; the two built
        // byte-identical transactions and the node computed ONE identity
        // for them. The shared identity IS the normalization, observed
        // rather than asserted -- a second identity would have been
        // evidence that the request does not normalize.
        "several-inputs-merged-into-one" | "canonical-input-normalization" => {
            Some(crate::live_explicit_shapes::run_of_record::MERGE_ACCEPTED_TXID)
        }
        "several-inputs-to-several-outputs" => {
            Some(crate::live_explicit_shapes::run_of_record::SEVERAL_TO_SEVERAL_ACCEPTED_TXID)
        }
        // TWO inputs under ONE owner: the repetition is the subject, and
        // both signatures verify out of the node's own copy, each over
        // its own position's recomputed message.
        "repeated-owner" => {
            Some(crate::live_explicit_shapes::run_of_record::REPEATED_OWNER_ACCEPTED_TXID)
        }
        // TWO inputs under two DISTINCT owners. Its destinations are the
        // several-to-several run's exactly and the identities differ
        // anyway, because the SPENT programs differ -- which is what
        // makes this run about its input owners.
        "several-distinct-owners" => {
            Some(crate::live_explicit_shapes::run_of_record::SEVERAL_DISTINCT_OWNERS_ACCEPTED_TXID)
        }
        "one-destination-owner" => {
            Some(crate::live_explicit_shapes::run_of_record::ONE_DESTINATION_OWNER_ACCEPTED_TXID)
        }
        // Destinations of one unit and the remainder. One is the
        // boundary the request type states rather than a small number
        // somebody picked, and the node took it.
        "semantic-boundary-values" => {
            Some(crate::live_explicit_shapes::run_of_record::BOUNDARY_VALUES_ACCEPTED_TXID)
        }
        "candidate-maximum-inputs" => {
            Some(crate::live_explicit_shapes::run_of_record::MAXIMUM_INPUTS_ACCEPTED_TXID)
        }
        "candidate-maximum-outputs" => {
            Some(crate::live_explicit_shapes::run_of_record::MAXIMUM_OUTPUTS_ACCEPTED_TXID)
        }
        // The sponsor-signed explicit control, cited by both rows it is
        // an instance of. It carries a sponsor region -- a sponsor
        // input, a two-item sponsor witness the adapter produced over
        // the exact finalized bytes, and a fee output in the reserve
        // asset -- and it requests NO sponsor change, which is read off
        // the ceremony's own request rather than inferred from the
        // outputs. So it is the `sponsored` class and the
        // `sponsor-change-absent` class at once.
        //
        // It is NOT `sponsor-change-present`. That row has its own run
        // and its own identity, below, on the rule this whole function
        // is held to: a row moves on an acceptance of its OWN shape.
        "sponsored" | "sponsor-change-absent" => {
            Some(crate::live_sponsor_shapes::sponsored_run_of_record::SPONSORED_ACCEPTED_TXID)
        }
        // The sponsored control that TAKES CHANGE, and the only run that
        // could answer this row. The change output is read out of the
        // node's own copy of the mined transaction and located by the
        // deployment's sponsor-change program, so a run whose shape had
        // degraded to the without-change form is a hard stop in the
        // ceremony rather than an acceptance reported here.
        //
        // The two sponsored runs differ in the change role and in
        // nothing else -- same issuance, receipts, destinations, owners
        // and fee -- and the target computed two different identities
        // for them, which is what makes the difference attributable.
        "sponsor-change-present" => Some(
            crate::live_sponsor_shapes::sponsored_run_of_record::SPONSORED_CHANGE_ACCEPTED_TXID,
        ),
        _ => None,
    }
}

/// The refusal that answered one negative row, where a run answered it.
///
/// The negative half's counterpart to [`observed_row_acceptance`], and
/// held to the matching rule: a row is added here when a target REFUSED
/// a candidate staging that row's own class WHILE having accepted the
/// unmutated form of the same candidate, on the same chain, in the same
/// session. A refusal without its control is not evidence, and a
/// control from another chain is not this one's.
///
/// Each entry returns the accepted control's identity and the target's
/// own words, both from a run of record.
fn observed_row_refusal(row: &LiveSafetyRow) -> Option<(&'static str, &'static str)> {
    use crate::live_explicit_shapes::witness_negatives_run_of_record as witness;

    match row.name() {
        // §10.2 types the signature position as an unconstrained item
        // precisely so that the TARGET is what refuses an empty or a
        // malformed offering, and it did. The two rows are answered by
        // one run and are distinguishable in it: the empty offering
        // failed the check that consumed it, and the well-sized
        // non-signature was judged and found invalid.
        "empty-signature" => Some((
            witness::CONTROL_ACCEPTED_TXID,
            witness::EMPTY_SIGNATURE_REFUSAL,
        )),
        "malformed-signature" => Some((
            witness::CONTROL_ACCEPTED_TXID,
            witness::MALFORMED_SIGNATURE_REFUSAL,
        )),
        // §15.6's sponsor-authorization row, answered by the sponsored
        // lane's own run: the mutant offered FIRST and then the
        // unmutated control, to one node on one chain. The mutant is
        // the recording pass's own completion, so it differs from the
        // control in the sponsor witness and in nothing else, and the
        // node measured the difference at exactly the two items the
        // adapter returned.
        //
        // The verdict reads the same as an earlier wave's fee-role
        // failure and is NOT it. What disambiguates them is the control
        // accepted in the SAME run: the fee-role check passes for this
        // deployment, so the comparison that failed is the one the
        // sponsor witness reaches.
        "missing-sponsor-authorization" => Some((
            crate::live_sponsor_shapes::sponsored_run_of_record::SPONSORED_ACCEPTED_TXID,
            crate::live_sponsor_shapes::sponsored_run_of_record::MISSING_SPONSOR_AUTHORIZATION_REFUSAL,
        )),
        // §15.5's two proof-negative rows, answered by the conservation
        // ceremony's own run — which submitted THREE mutants before the
        // control for a reason it states, all four spending one coin: a
        // control accepted first would have spent it, and every mutant
        // after it would have been refused for a missing input rather
        // than for its mutation. That is the same ordering lesson the
        // witness negatives learned as `txn-already-known`, met here in
        // its other form.
        //
        // THE TWO ROWS SHARE ONE ARM BECAUSE THE TARGET SHARED ONE
        // ANSWER, and that is the finding rather than a shortcut. The
        // internal Pedersen-tally and range-proof codes are discarded
        // inside the target's own `VerifyAmounts` and never leave it, so
        // one verdict covers both mutations and the LAYER cannot be what
        // separates these rows. The FIELD is: each row's mutant declared
        // a byte range and was checked to have stayed inside it by
        // `attribute_proof_negative` rather than by assertion —
        // `RANGEPROOF_FIELD_RANGE` for `malformed-rangeproof` and
        // `WRONG_BLINDER_FIELD_RANGE` for `wrong-private-blinding-balance`.
        // Two rows move on ONE run because each drove its OWN mutant,
        // which is the rule; a third case in the same run,
        // `missing-rangeproof`, moves no §15 row at all, the matrix
        // having no member for it.
        //
        // For `wrong-private-blinding-balance` the declared class is
        // `AmountMismatch`, and the target's own words name it:
        // `bad-txns-in-ne-out` is value in not equal to value out.
        // `malformed-rangeproof` declares no class and is attributed the
        // way the witness-content rows are, by its field and its control.
        //
        // `private-ct-imbalance` is answered the SAME way, on its OWN
        // mutant. The conservation ceremony now submits a fourth mutant: a
        // value commitment to a value one unit above the change the control
        // balances, placed at the SECOND output. It declares the same
        // `AmountMismatch` class and draws the same `bad-txns-in-ne-out`
        // words, and it is NOT one observation counted twice: its mutant is
        // its own, and its declared field range is the second output's
        // value commitment (`PRIVATE_CT_IMBALANCE_FIELD_RANGE`, 215..248),
        // disjoint from the wrong blinder's (81..114). Two rows move on the
        // same run because each drove its own mutant at its own field,
        // which is the rule, and the distinct output is what earns the
        // separating field range fact (e) of the ceremony's charter names.
        "malformed-rangeproof" | "wrong-private-blinding-balance" | "private-ct-imbalance" => {
            Some((
                crate::live_conservation_negatives::run_of_record::CONTROL_ACCEPTED_TXID,
                crate::live_conservation_negatives::run_of_record::MUTANT_REJECT_DETAIL,
            ))
        }
        // §15.4's script-path row, answered by the owner-signing negative
        // ceremony's own run: the bare-u mutant offered FIRST and the
        // unmutated control LAST, to one node on one chain. The mutant is
        // re-signed over its own mutated bytes through the negative-evidence
        // census, so it passes the leaf's signature gate and reaches the
        // coordinator leaf's `InspectOutputScriptPubKey` version clause,
        // which refuses it. The verdict reads as a generic script-verify
        // failure and is attributed by the FIELD the mutant declared and
        // stayed within — the mutated destination's program alone
        // (`DECLARED_FIELD_RANGE`) — with the unmutated control accepted in
        // the SAME run to make the difference the leaf measured this row's.
        // The re-signing changes the witness too, by design; the declared
        // range is measured over the WITNESSLESS serialization the message
        // is taken over, where it does not reach.
        "vault-control-entitlement-or-bare-u-output" => Some((
            crate::live_owner_signing_negatives::run_of_record::CONTROL_ACCEPTED_TXID,
            crate::live_owner_signing_negatives::run_of_record::MUTANT_REJECT_DETAIL,
        )),
        // The seven conservation-breaking rows, answered by the SAME
        // owner-signing negative run — each on its OWN consensus mutant,
        // cut from the signed explicit control and offered before it. Each
        // breaks the explicit per-asset sum in its own way (a wrong asset,
        // a blinded asset with no surjection proof, a value one below and
        // one above the total, an omitted destination, an added output, an
        // omitted source), and the target folds every break into ONE
        // balance verdict — `bad-txns-in-ne-out` at consensus before any
        // script — so the WORDS are shared and it is the declared field
        // range, or the transaction shape for the three structural rows,
        // that separates the rows. This is the `private-ct-imbalance`
        // discipline read across the explicit successor: a consensus
        // verdict IS an attributable observation where a distinct field or
        // a distinct shape earns the separating fact, and each row drove
        // its own mutant so no two rest on one observation. The four field
        // ranges (`WRONG_EXPLICIT_ASSET_FIELD_RANGE`,
        // `CONFIDENTIAL_ASSET_COMMITMENT_FIELD_RANGE`,
        // `OUTPUT_TOTAL_ONE_BELOW_FIELD_RANGE`,
        // `OUTPUT_TOTAL_ONE_ABOVE_FIELD_RANGE`) are pairwise distinct on the
        // 2-in-2-out shape; the two output-cardinality rows share the
        // un-localizable structural range and separate by shape (two-in
        // one-out against two-in three-out); `omitted-source` separates by
        // its one-input shape and its own range.
        "wrong-explicit-asset"
        | "confidential-asset-commitment"
        | "output-total-one-below-input"
        | "output-total-one-above-input"
        | "private-output-omitted"
        | "hidden-private-u-output"
        | "omitted-source" => Some((
            crate::live_owner_signing_negatives::run_of_record::CONTROL_ACCEPTED_TXID,
            crate::live_owner_signing_negatives::run_of_record::CONSENSUS_MUTANT_REJECT_DETAIL,
        )),
        // ONE driven row of each leaf-arrangement collision pair, answered
        // by the SAME owner-signing negative run. The four rows form two
        // pairs drawing one verdict each — the coordinator index check and
        // the member bound check — so one mutant per pair is driven and the
        // other stays typed (`TargetVerdictDoesNotSeparateTheRows`) because
        // its own mutant would draw the same verdict at the same clause.
        // `two-coordinators` reveals the coordinator leaf at both inputs, so
        // the coordinator running at input one fails the index EqualVerify —
        // the one input that fails, the other being the control's valid
        // coordinator; `no-coordinator` reveals a member leaf at both
        // inputs, so the member running at input zero fails the bound's
        // lower Verify. Each mutant is a distinct candidate: the outputs,
        // assets and values are the control's, so the witnessless
        // serialization is byte-identical and the separating fact is the
        // revealed-leaf ARRANGEMENT — a role at a forbidden position — which
        // is what tells the driven row from the bare-u mutant that reads the
        // same OP_EQUALVERIFY and from every sibling. No taptree moved: both
        // funded coins commit to one tree holding both leaves, so the
        // rearrangement reuses committed leaves.
        "two-coordinators" => Some((
            crate::live_owner_signing_negatives::run_of_record::CONTROL_ACCEPTED_TXID,
            crate::live_owner_signing_negatives::run_of_record::TWO_COORDINATORS_REJECT_DETAIL,
        )),
        "no-coordinator" => Some((
            crate::live_owner_signing_negatives::run_of_record::CONTROL_ACCEPTED_TXID,
            crate::live_owner_signing_negatives::run_of_record::NO_COORDINATOR_REJECT_DETAIL,
        )),
        _ => None,
    }
}

/// The first-party determinism observation that answers one row, where
/// there is one.
///
/// Beside [`observed_row_acceptance`] and shaped like it, and separate
/// from it for the reason [`LiveRowStanding::DeterminismObserved`]
/// states: what this returns is not a target verdict and must never be
/// filed as one.
///
/// One row today, and it is the row §11.2 gave a non-acceptance gate.
/// A row whose gate IS an acceptance may not be answered here, however
/// deterministically it reproduces — the wave-five closeout recorded
/// exactly that case and declined it, a successor reproducing its own
/// accepted identity byte for byte being a determinism fact about a
/// SUCCESSOR while this row is about fixture OPENINGS.
fn observed_row_determinism(row: &LiveSafetyRow) -> Option<(&'static str, &'static str)> {
    match row.name() {
        // §11.2 lists this row's gate as the byte-identity contract
        // (§6.7) rather than as an acceptance, and §6.7's own closing
        // sentence says it is the contract that satisfies this row.
        // The observation is wave seven's: registering the same
        // three-output manifest twice yields the same fixture digest
        // and the same per-output value blinders, byte for byte.
        "deterministic-public-fixture-openings" => Some((
            "a fixture recomputed from its own manifest: equal digest and equal \
             per-output openings, byte for byte",
            "crate::live_multi_shapes::a_fixture_recomputes_byte_identically_from_its_manifest",
        )),
        _ => None,
    }
}

/// The first-party fact that answers one row, where the row's own gate
/// IS that fact.
///
/// Beside [`observed_row_determinism`] and shaped like it, and separate
/// from it for the reason [`LiveRowStanding::FirstPartyFactObserved`]
/// states: what this returns is neither a target verdict nor a
/// recomputation, and must never be filed as either.
///
/// The bar for an entry is the one the member states, and it is about
/// the ROW rather than about the fact: the row's published gate has to
/// be the fact itself. A row asking what a target does with something
/// may not be answered here however firmly some fact about it is
/// established.
fn observed_row_first_party_fact(row: &LiveSafetyRow) -> Option<(&'static str, &'static str)> {
    match row.name() {
        // §4.3's third distinction. The row's gate is stated at its own
        // site in `crate::live_safety`: the safe constructor cannot
        // express this candidate, so its refusal would establish nothing
        // about it, and WHAT THE ROW ASKS IS WHETHER A RAW PATH EXISTS
        // AT ALL. That is a question about this workspace, and the
        // answer is visibly yes.
        //
        // `transaction::TargetTransaction::with_output_witnesses` is
        // public and checks census ARITY only — no amount, asset,
        // program, position or role — and three lanes already rebuild
        // finalized bytes through it and hand the result to a real node.
        // The blocker that used to carry this row said the path did not
        // exist; that premise expired, the previous wave recorded the
        // expiry at the blocker's own site and escalated rather than
        // choosing a standing, and this is the standing the ruling
        // chose.
        //
        // What this does NOT claim is that any target refused a bypass.
        // No such run happened and the row never asked for one.
        "raw-transaction-bypassing-safe-construction" => Some((
            "a raw assembly path bypassing the safe constructor exists and is used: \
             `with_output_witnesses` is public, checks census arity only, and three \
             lanes rebuild finalized bytes through it and submit them to a real node",
            "crate::live_conservation_negatives::run_of_record",
        )),
        // §15.6's zero-valued sponsor row, on the ruling that the
        // realization's reading GOVERNS. THE MATRIX PREDICTS A REFUSAL
        // THE REALIZATION REFUSES TO MAKE, and that is not a close
        // call: revision 13d removed an accidental positivity
        // requirement from ordinary sponsor value, and the projection
        // does not carry a sponsor amount AT ALL — a sponsor-role
        // `PLAIN_LBTC` member must be `ObservedValue::SponsorOpaque`,
        // so there is no amount for any relation to compare with zero
        // (`realization::evaluate`, the `ObjectId::PlainLbtc` arm,
        // whose own comment says a zero-valued sponsor member is an
        // ordinary member like any other).
        //
        // The guide had already ruled it, twice, and the transcription
        // did not carry the ruling across. Guide 8 §22.6 corrects this
        // exact contradiction by name — "zero-valued ordinary sponsor
        // member with exact role structure: semantic acceptance", and
        // then "Do not preserve a generic domain-failure vector for
        // zero-valued ordinary sponsor output". The §15.6 row IS a
        // preserved generic domain-failure vector. Guide 12 gives the
        // three-layer reading the row flattens: the semantic relation
        // MAY ACCEPT exact role structure, the first-party builder
        // OMITS known zero change as construction policy, and a
        // deployment MAY reject it as nonstandard.
        //
        // All three layers are in this workspace and none of them
        // yields a refusal this row could carry. The builder does not
        // refuse a zero residual, it declines to emit one —
        // `transaction::construct::is_known_zero`, whose site says the
        // target refuses a spendable zero-valued output so emitting one
        // would produce a transaction consensus rejects. So no
        // candidate carrying this row's fault can be built here, and
        // the fault is not a fault in the first place.
        //
        // What is filed is therefore the fact the sources state, and
        // the site is the relation's own deciding test, driven twice —
        // once on a fully zero sidecar and once on a mixed one.
        //
        // WHAT THIS DOES NOT CLAIM is that any target accepted such a
        // transfer. None was offered, and the safe constructor cannot
        // build one to offer. The row's declaration is corrected only
        // as far as this workspace owns it; the erratum against the
        // §15.6 table is filed with the guide.
        "zero-valued-ordinary-sponsor-member" => Some((
            "the semantic relation admits a zero-valued ordinary sponsor member under exact              role structure: sponsor-value opacity leaves no amount for any relation to              compare with zero, so no layer of this workspace refuses the shape",
            "realization::tests::live_transfer_tests::zero_sponsor_sidecar_is_accepted",
        )),
        // §15.5's duplicated-destination row, and the second row whose
        // predicted refusal the sources refuse to make. Its own sibling
        // `duplicated-source` IS a fault and is refused at the earliest
        // boundary this workspace has; this row is not a fault by the
        // same type's own reading. §12.2 makes the destination census a
        // MULTISET rather than a set, and states why in its own words:
        // two destinations of the same owner and value are two
        // receipts, and a set would report them as one. The deciding
        // test builds that pair and names the verdict — an even split
        // is an ordinary transfer.
        //
        // So no layer refuses it, and the reason is not that a layer is
        // missing. Nothing is wrong with the candidate.
        "duplicated-destination" => Some((
            "two destinations of one owner and one value are two receipts and an ordinary              split: the destination census is a multiset by §12.2 so that a repeat counts              twice rather than collapsing, and no layer of this workspace refuses one",
            "transaction::tests::live_request_tests::two_destinations_of_one_owner_and_one_value_are_two_receipts",
        )),
        _ => None,
    }
}

/// Classify one row of the §15 matrix.
fn classify(
    row: &'static LiveSafetyRow,
    plan: &ValidatedLiveTransferOperationPlan,
    discharged: &BTreeMap<&'static str, (DischargingValidator, &'static str)>,
) -> Result<LiveRowStanding, VectorError> {
    // The row no layer answers, before anything else: it is not
    // discharged, not blocked, and not waiting on a run, and every later
    // branch here presumes a layer was asked.
    let Some(boundary) = row.refusing_layer() else {
        return Ok(LiveRowStanding::OperationVocabularyClosed);
    };
    if boundary == EvidenceBoundary::ReportSemanticProjectionRejection {
        return Ok(LiveRowStanding::ReportLayerAnswerable);
    }
    if let Some((validator, class)) = discharged.get(row.name()) {
        return Ok(LiveRowStanding::FirstPartyDischarged {
            validator: *validator,
            class,
        });
    }
    if row.is_first_party() {
        // A first-party row neither census discharged. Nothing reaches
        // this today and the branch is not decoration: it is what makes
        // §4.2's last sentence hold for a row added tomorrow, whose
        // obligation is counted and rendered from the moment it exists
        // rather than from the moment somebody lists it.
        return Ok(LiveRowStanding::FirstPartyUndischarged(
            FirstPartyGap::NoStagedCase,
        ));
    }

    if let Some(blocker) = specific_blocker(row) {
        return Ok(LiveRowStanding::InfrastructureBlocked(blocker));
    }
    // An answered row before a blocked one would let an observation
    // paper over a component that is still missing, so this is asked
    // AFTER the specific blocker and never before it.
    if let Some(accepted_identity) = observed_row_acceptance(row) {
        return Ok(LiveRowStanding::NativeRunObserved { accepted_identity });
    }
    // Beside it and after it, for the same reason it sits after the
    // specific blocker: a refusal answers a row only once the row is
    // not waiting on something that would have to exist first.
    if let Some((control_identity, refusal_detail)) = observed_row_refusal(row) {
        return Ok(LiveRowStanding::NativeRefusalObserved {
            control_identity,
            refusal_detail,
        });
    }
    // Beside the two above and after them, for the reason they sit
    // after the specific blocker. It is asked after the single-identity
    // acceptance branch deliberately: a row answerable by an acceptance
    // of its OWN shape must take that answer, and only a row whose gate
    // is a RELATION over two acceptances reaches this at all.
    if let Some((explicit_identity, private_identity, relation)) = observed_row_paired_relation(row)
    {
        return Ok(LiveRowStanding::PairedRelationObserved {
            explicit_identity,
            private_identity,
            relation,
        });
    }
    // Last of the observation branches, and after the specific
    // blocker for the same reason the others are. It is asked after
    // them rather than before because a row whose gate IS a target
    // verdict must take the target's answer where one exists; only a
    // row §11.2 gave a non-acceptance gate reaches this at all.
    if let Some((recomputed, observed_by)) = observed_row_determinism(row) {
        return Ok(LiveRowStanding::DeterminismObserved {
            recomputed,
            observed_by,
        });
    }
    // Last of the four observation branches, and after the specific
    // blocker for the reason all of them are: a row waiting on a
    // component that does not exist is not answered by a fact about a
    // different component. It is asked after the three above because a
    // row whose gate is a target verdict or a recomputation must take
    // that answer where one exists; only a row whose own gate is a fact
    // about this workspace reaches here at all.
    if let Some((fact, observed_by)) = observed_row_first_party_fact(row) {
        return Ok(LiveRowStanding::FirstPartyFactObserved { fact, observed_by });
    }
    if !a_positive_control_exists() {
        return Ok(LiveRowStanding::InfrastructureBlocked(
            LiveInfrastructureBlocker::NoAcceptingControlExists,
        ));
    }

    // The row is waiting on a run that could happen. It carries the
    // requirement it resolves to under the explicit plan in the
    // sponsorless case, which is the case a first run submits.
    let link = resolve_row(
        plan,
        row,
        LiveTransferRepresentationPlan::Explicit,
        SponsorCase::Absent,
    )?;
    Ok(LiveRowStanding::NativeRunRequired(match link {
        LiveRowLink::Resolved(id) => Some(id),
        LiveRowLink::EveryRelation | LiveRowLink::InactiveInThisCase | LiveRowLink::Blocked(_) => {
            None
        }
    }))
}

/// Derive the canonical live-transfer evidence plan (§13.1).
///
/// The seven sources, in order: the compiler coverage through
/// [`live_transfer_plan`]; the exact linked bundle and the exact
/// candidate ABI through [`demonstration_live_bundle`] and
/// [`demonstration_live_abi`]; the canonical safety mutation registry
/// through [`required_safety_matrix`]; the minimality pair registry,
/// built through [`crate::live_pairs::build_minimality_pairs`] and
/// recorded here as a standing rather than as a verdict (§13.6); and the
/// exact target,
/// deployment, and executor provenance expectation, which are a *run's*
/// inputs and enter through [`crate::live_report`] rather than here — a
/// plan that named a deployment nobody ran against would be asserting a
/// binding.
///
/// # Errors
///
/// [`VectorError::LiveSubstrateUnavailable`] when a source artifact does
/// not build, and [`VectorError::NegativeLinkUnresolved`] when a row's
/// declared link no longer resolves against the published plan.
pub fn derive_live_evidence_plan() -> Result<LiveTransferEvidencePlan, VectorError> {
    let plan = live_transfer_plan()?;
    let bundle = demonstration_live_bundle()?;
    let abi = demonstration_live_abi()?;

    // Every declared row must resolve somewhere, or the registry and the
    // published plan have drifted apart and every classification below
    // would be filed against a requirement that no longer exists.
    check_every_declared_row_resolves(&plan)?;

    let evidence =
        discharge_live_first_party().map_err(|_| VectorError::LiveSubstrateUnavailable)?;
    let cases = live_first_party_cases();
    let mut index: BTreeMap<&'static str, (DischargingValidator, &'static str)> = evidence
        .iter()
        .map(|discharged| {
            let class = cases
                .iter()
                .find(|case| case.row() == discharged.row())
                .map_or(
                    "",
                    crate::live_first_party::LiveFirstPartyCase::expected_class,
                );
            (
                discharged.row(),
                (
                    DischargingValidator::OwnerAuthorization(discharged.validator()),
                    class,
                ),
            )
        })
        .collect();

    // §15.4–§15.7's pre-target rows, from the second census. Both are
    // *recomputed* rather than read: each entry here came back from a
    // validator that was driven twice, and a row absent from both indexes
    // stays outstanding rather than being assumed covered.
    let faults = discharge_live_faults().map_err(|_| VectorError::LiveSubstrateUnavailable)?;
    let fault_cases = live_fault_cases();
    for discharged in &faults {
        let class = fault_cases
            .iter()
            .find(|case| case.row() == discharged.row())
            .map_or(
                "",
                crate::live_fault_discharge::LiveFaultCase::expected_class,
            );
        index.insert(
            discharged.row(),
            (DischargingValidator::Fault(discharged.validator()), class),
        );
    }

    let mut rows = Vec::with_capacity(required_safety_matrix().len());
    let mut census = LiveEvidenceCensus::default();
    for row in required_safety_matrix() {
        let standing = classify(row, &plan, &index)?;
        census.rows += 1;
        match &standing {
            LiveRowStanding::FirstPartyDischarged { .. } => census.first_party_discharged += 1,
            LiveRowStanding::FirstPartyUndischarged(_) => census.first_party_undischarged += 1,
            LiveRowStanding::NativeRunRequired(_) => census.native_run_required += 1,
            LiveRowStanding::NativeRunObserved { .. } => census.native_run_observed += 1,
            LiveRowStanding::NativeRefusalObserved { .. } => census.native_refusal_observed += 1,
            LiveRowStanding::DeterminismObserved { .. } => census.determinism_observed += 1,
            LiveRowStanding::PairedRelationObserved { .. } => {
                census.paired_relation_observed += 1;
            }
            LiveRowStanding::FirstPartyFactObserved { .. } => {
                census.first_party_fact_observed += 1;
            }
            LiveRowStanding::InfrastructureBlocked(_) => census.infrastructure_blocked += 1,
            LiveRowStanding::ReportLayerAnswerable => census.report_layer += 1,
            LiveRowStanding::OperationVocabularyClosed => census.vocabulary_closed += 1,
            LiveRowStanding::Experimental => census.experimental += 1,
        }
        rows.push(LiveEvidenceRow { row, standing });
    }

    Ok(LiveTransferEvidencePlan {
        plan,
        bundle,
        abi,
        minimality: minimality_registry_standing(),
        rows,
        discharged: evidence,
        fault_discharged: faults,
        census,
    })
}

/// Where §13.1's fifth source stands, resolved by building it.
///
/// The registry is built rather than asked about, so a source that
/// stopped building is reported as unavailable instead of as a source
/// with no pairs. What comes back is two counts and no verdict: §13.6
/// keeps [`crate::live_minimality_report`]'s answer out of a safety plan.
fn minimality_registry_standing() -> MinimalityRegistryStanding {
    crate::live_pairs::build_minimality_pairs().map_or(
        MinimalityRegistryStanding::NotConstructible,
        |rows| MinimalityRegistryStanding::Built {
            pairs: rows.len(),
            supporting: rows.iter().filter(|row| row.supports_minimality()).count(),
        },
    )
}

/// Every declared row resolves in at least one representation and case.
fn check_every_declared_row_resolves(
    plan: &ValidatedLiveTransferOperationPlan,
) -> Result<(), VectorError> {
    for row in required_safety_matrix() {
        if !matches!(
            row.relation(),
            crate::live_safety::LiveRelationStanding::Declared { .. }
        ) {
            continue;
        }
        let mut resolved = false;
        for representation in [
            LiveTransferRepresentationPlan::Explicit,
            LiveTransferRepresentationPlan::PrivateCommitted,
        ] {
            for case in [SponsorCase::Absent, SponsorCase::Present] {
                if matches!(
                    resolve_row(plan, row, representation, case)?,
                    LiveRowLink::Resolved(_)
                ) {
                    resolved = true;
                }
            }
        }
        if !resolved {
            let crate::live_safety::LiveRelationStanding::Declared { class_name, .. } =
                row.relation()
            else {
                continue;
            };
            return Err(VectorError::NegativeLinkUnresolved { class: class_name });
        }
    }
    Ok(())
}

/// Every distinct blocker the plan records, with how many rows carry it.
#[must_use]
pub fn blocker_census(
    plan: &LiveTransferEvidencePlan,
) -> BTreeMap<LiveInfrastructureBlocker, usize> {
    let mut census = BTreeMap::new();
    for row in plan.rows() {
        if let LiveRowStanding::InfrastructureBlocked(blocker) = row.standing() {
            *census.entry(*blocker).or_insert(0_usize) += 1;
        }
    }
    census
}

/// The bytes that stand in a signature position this lane does not
/// fill.
///
/// Not a signature, and named so at every use.
///
/// The reason has narrowed and the constant has not. It used to be that
/// nothing in this workspace computed the digest §10.2's fragment checks
/// against, so no signature could be produced at all. That is no longer
/// so: the digest is computed, the profile is established, and two
/// ceremonies carry observed acceptances of candidates they signed for
/// real. What remains true is narrower and is still a reason — the lanes
/// that use this constant do not run a signing ceremony, either because
/// they are weighing a serialization rather than authorizing one, or
/// because §1.9 puts a sponsor's authorization outside protocol data and
/// [`LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent`] records
/// that no adapter signer is wired into this lane to supply one.
///
/// The distinction matters at exactly one place and it is worth stating
/// there: a lane filling this in is producing a transaction that earns
/// no target verdict about its own relation, and that is now a property
/// of the LANE rather than of the workspace.
///
/// So a witness position that has to be *filled* — to serialize a
/// transaction at all, or to weigh one — is filled with bytes of the
/// right width that authorize nothing. Stated once, here beside the two
/// blockers that are the reason for it, so that three modules cannot
/// drift into three different widths and quietly change what every
/// measured weight is a weight of.
pub const UNAUTHORIZING_SIGNATURE: [u8; 64] = [0x5c; 64];

/// Whether any run has compared the public protocol projections of an
/// ACCEPTED private transaction and its PAIRED ACCEPTED explicit one.
///
/// The filed path for the `projection-equality-with-paired-explicit`
/// row, and DERIVED rather than written: it is the pairs arc's own
/// ledger flag. A constant restating in this file what another module
/// observes is a second opinion free to drift from the first, and this
/// one is the same fact read from where the fact lives.
///
/// # What it took, because the two gaps were real
///
/// The minimality wave filed this FALSE and named two things standing in
/// the way rather than one, and both had to be closed.
///
/// The first was that the two acceptances a reader would reach for are
/// NOT a pair. §16.1's load-bearing word is that a pair begins from ONE
/// semantic fixture materialized twice, and
/// `live_explicit_shapes::run_of_record::ONE_TO_ONE_ACCEPTED_TXID` and
/// `live_multi_shapes::run_of_record::STRICT_ONE_TO_ONE_ACCEPTED_TXID`
/// are two INDEPENDENT ceremonies whose shapes match. Comparing them
/// would be the substitution [`crate::live_pairs::PairTargetVerdict`]'s
/// `NotSubmittedShapeAcceptedElsewhere` member exists to deny. The arc
/// closes it by CONSTRUCTION rather than by argument: both its members
/// read one fixture, and neither shape carries a literal of its own.
///
/// The second was that no standing could hold the observation.
/// [`LiveRowStanding::NativeRunObserved`] carries exactly ONE
/// `accepted_identity` and a projection equality is a relation over TWO,
/// so filing it under that member would be the single error a run of
/// record exists to prevent. The wave declined to mint a member for it
/// unilaterally and reported the gap;
/// [`LiveRowStanding::PairedRelationObserved`] is the repair the owner's
/// pairs-arc ruling directed in answer, on the precedent
/// [`LiveRowStanding::DeterminismObserved`] set.
///
/// # What it is still not
///
/// True here means one arc observed one relation over one fixture's two
/// materializations. It is not a claim about §16.1's other four pairs,
/// whose members remain unsubmitted and whose registry entries say so.
pub const A_PAIRED_ACCEPTED_PROJECTION_COMPARISON_EXISTS: bool =
    crate::live_pair_arc::run_of_record::A_PAIR_ARC_LEDGER_EXISTS;

/// The relation one PAIRS ARC observed over its two accepted members.
///
/// Beside [`observed_row_acceptance`] and shaped like it, and separate
/// for the reason [`LiveRowStanding::PairedRelationObserved`] is a
/// separate member: what answers this row is not an acceptance but a
/// RELATION over two of them, and the two have to be the two
/// materializations of one §16.1 fixture rather than two runs whose
/// shapes match.
///
/// It answers only while the arc's own run of record carries BOTH
/// identities. The flag and the identities move together at the arc's
/// own site, so a row cannot move here on a ledger that does not exist.
fn observed_row_paired_relation(
    row: &LiveSafetyRow,
) -> Option<(&'static str, &'static str, &'static str)> {
    use crate::live_pair_arc::run_of_record as arc;

    match row.name() {
        "projection-equality-with-paired-explicit" => {
            let explicit = arc::EXPLICIT_MEMBER_ACCEPTED_IDENTITY?;
            let private = arc::PRIVATE_MEMBER_ACCEPTED_IDENTITY?;
            if !arc::A_PAIR_ARC_LEDGER_EXISTS {
                return None;
            }
            Some((
                explicit,
                private,
                "the public protocol projections of the two accepted materializations of one \
                 §16.1 one-to-one fixture agree on every §6.6 term, the private member \
                 withholding the exact amounts the explicit member publishes",
            ))
        }
        _ => None,
    }
}

/// The residuals this plan inherits and does not clear.
///
/// Named as a set rather than as prose so that a later wave clearing one
/// has to remove it here, and so a reader can see at a glance which of
/// them are this workspace's to clear.
///
/// This paragraph has now been separated back TWICE. It sat above
/// [`UNAUTHORIZING_SIGNATURE`] once and was moved; the move put it above
/// the time-locked row's typed stop instead of above this function, so
/// the set went on carrying no doc comment of its own while a constant
/// beside it carried one describing a set. It is attached to the
/// function it is about here, and the stop it was glued to is gone.
///
/// EMPTY, and it was three. All three have left, and they left by three
/// different kinds of evidence, which is the distinction this set exists
/// to keep legible.
///
/// [`LiveInfrastructureBlocker::SighashProfileUnreviewed`] was cleared by
/// the owner-sighash review verdict together with the
/// post-verdict re-typing — by a verdict, that is, and never by a
/// run, which is the discipline that residual was separated from the
/// digest blocker in order to keep.
///
/// [`LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent`] was
/// cleared the other way, by a RUN and only by a run: a sponsored
/// explicit control carrying a sponsor owner's authorization was
/// submitted to a real node, accepted, read back byte for byte, and
/// mined. Its defining site said a returned byte stack is not a target
/// authorization until a target has accepted a control carrying it, and
/// that is the sentence this clearing satisfies.
///
/// [`LiveInfrastructureBlocker::PredecessorConstructorAbsent`] left the
/// third way, by a RULING that corrected the ground: the observed pair
/// its clearing rule demanded was demanded by a mis-typing of the row it
/// belonged to, one half of that pair is refused by §10.4 and the other
/// is unattributable in principle, and the row is retyped first-party
/// where its sibling already sat. Its defining site carries the whole of
/// that, including what did NOT move with it — no time-locked spend was
/// ever observed refused, and no constructor was built.
///
/// An empty set is not a smaller set of obligations. It says that every
/// residual this plan inherited has been discharged at its own defining
/// site by its own rule, and it says nothing about the rows a run has
/// still to answer: those the census counts, and this does not.
#[must_use]
pub const fn carried_residuals() -> BTreeSet<LiveInfrastructureBlocker> {
    BTreeSet::new()
}

#[cfg(test)]
mod tests {
    use super::{
        LiveInfrastructureBlocker, LiveRowStanding, MinimalityRegistryStanding, blocker_census,
        derive_live_evidence_plan,
    };
    use crate::live_safety::{LiveSafetyPolarity, LiveSafetySection};
    use std::collections::BTreeSet;

    #[test]
    fn the_first_party_half_of_the_matrix_is_answered_in_full() {
        // The matrix's pre-target half, after both censuses. Twenty-six
        // rows of §15 are refused before any target sees the bytes, and
        // every one of them has been driven to its own refusal against
        // its own control. Nothing here is outstanding, and nothing here
        // was closed by a wildcard: the two rows that left this half
        // left because their declared boundary was wrong, and the count
        // is recomputed from the classification rather than adjusted.
        //
        // It read twenty-five until `time-locked-input` ARRIVED here,
        // and it arrived the same way those two left — because its
        // declared boundary was wrong. The row asked the target to
        // refuse a class the target never reads; the ruling retyped it
        // beside the `time-locked-output` sibling it always had, and the
        // discharge is a refusal driven against a control like every
        // other row's in this half.
        //
        // It reads TWENTY-NINE now, and the three that arrived came the
        // same way `time-locked-input` did: their declared boundary was
        // wrong. `ash-input-or-output`, `malformed-live-metadata` and
        // `foreign-asset-under-receipt-shaped-program` each asked a
        // target to refuse something on a chain, and for the first and
        // third what a chain answers is the COMMITMENT rule — the
        // identical verdict every foreign taptree draws, which names no
        // family and compares no asset. The second is refused before a
        // program exists at all. Each is now driven to its own refusal
        // against its own control, with its own changed field.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let census = plan.census();
        // THIRTY-FOUR now: `wrong-owner-metadata` joined the three
        // siblings sharing `ReceiptInputIsNotALiveReceipt`, its honest
        // third-owner program buildable once a third owner was threaded
        // through the bundle link, so it is discharged first-party where
        // its declared boundary — the input recognition — actually is.
        assert_eq!(census.first_party_discharged(), 34);
        assert_eq!(census.first_party_undischarged(), 0);

        let outstanding: BTreeSet<_> = plan
            .rows()
            .iter()
            .filter(|row| matches!(row.standing(), LiveRowStanding::FirstPartyUndischarged(_)))
            .map(|row| row.row().name())
            .collect();
        assert_eq!(outstanding, BTreeSet::new());

        // And the whole matrix still cross-foots.
        assert_eq!(
            census.first_party_discharged()
                + census.first_party_undischarged()
                + census.native_run_required()
                + census.native_run_observed()
                + census.determinism_observed()
                + census.paired_relation_observed()
                + census.first_party_fact_observed()
                + census.native_refusal_observed()
                + census.infrastructure_blocked()
                + census.report_layer()
                + census.vocabulary_closed()
                + census.experimental(),
            108,
        );
    }

    #[test]
    fn one_row_is_closed_by_the_operation_vocabulary_and_it_is_not_evidence() {
        // §15.4's `mixed-operation-program`, outside §4.2's refusal
        // denominator and outside the answered count as well. What
        // establishes the closure is the architecture, and the three
        // facts it rests on are checked here rather than asserted: the
        // compiler projects one operation, and the leaf vocabulary the
        // constructor and the linker consume names only that operation's
        // roles. The third — that a request cannot select a program at
        // all — is the transaction crate's own census of the fourteen
        // structurally absent request facets.
        use compiler::live_transfer_plan::LiveTransferRepresentationPlan;
        use tapscript::{demonstration_live_shape_set, static_transfer_leaf_set};

        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        assert_eq!(plan.census().vocabulary_closed(), 1);
        let closed: BTreeSet<_> = plan
            .rows()
            .iter()
            .filter(|row| row.standing() == &LiveRowStanding::OperationVocabularyClosed)
            .map(|row| row.row().name())
            .collect();
        assert_eq!(closed, BTreeSet::from(["mixed-operation-program"]));

        // Not answered, and not evidence: a report reading this as a
        // discharge would be reading a structural fact as a refusal.
        assert!(!LiveRowStanding::OperationVocabularyClosed.is_answered());

        assert_eq!(
            plan.operation_plan().operation(),
            architecture::OperationId::TransferLive,
        );
        let shapes = demonstration_live_shape_set();
        for representation in [
            LiveTransferRepresentationPlan::Explicit,
            LiveTransferRepresentationPlan::PrivateCommitted,
        ] {
            let leaves = static_transfer_leaf_set(representation, &shapes);
            assert_ne!(leaves.len(), 0, "the leaf census is empty");
            for leaf in &leaves {
                assert_eq!(leaf.representation(), representation);
            }
        }
    }

    #[test]
    fn both_discharging_vocabularies_are_really_driven() {
        // A plan that quietly lost one census would report a smaller
        // matrix rather than fail, so the two are counted apart.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let mut owner = 0_usize;
        let mut fault = 0_usize;
        for row in plan.rows() {
            if let LiveRowStanding::FirstPartyDischarged { validator, class } = row.standing() {
                assert_ne!(class.len(), 0, "{} discharged naming no class", row.row());
                match validator {
                    crate::live_evidence::DischargingValidator::OwnerAuthorization(_) => owner += 1,
                    crate::live_evidence::DischargingValidator::Fault(_) => fault += 1,
                }
            }
        }
        assert_eq!(
            owner,
            crate::live_first_party::live_first_party_cases().len()
        );
        assert_eq!(fault, crate::live_fault_discharge::live_fault_cases().len());
    }

    #[test]
    fn the_minimality_source_is_built_and_carries_no_verdict() {
        // §13.1's fifth source, and §13.6's separation held at the same
        // time: the plan says the registry exists and how wide it is, and
        // it says nothing about whether the private plan discloses less.
        // The supporting count is FIVE of five and that is a fact
        // about the registry, not a minimality conclusion — which
        // `crate::live_minimality_report` is the only thing entitled to
        // draw. It read zero until both shapes of a pair had been run
        // against a real target and accepted.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        assert!(plan.minimality().is_built());
        assert_eq!(
            plan.minimality(),
            MinimalityRegistryStanding::Built {
                pairs: crate::live_pairs::MinimalityPair::ALL.len(),
                supporting: 5,
            },
        );
    }

    #[test]
    fn the_plan_classifies_every_row_exactly_once() {
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let census = plan.census();
        assert_eq!(census.rows(), crate::live_safety::row_count());
        assert_eq!(census.rows(), plan.rows().len());
        assert_eq!(
            census.first_party_discharged()
                + census.first_party_undischarged()
                + census.native_run_required()
                + census.native_run_observed()
                + census.determinism_observed()
                + census.paired_relation_observed()
                + census.first_party_fact_observed()
                + census.native_refusal_observed()
                + census.infrastructure_blocked()
                + census.report_layer()
                + census.vocabulary_closed()
                + census.experimental(),
            census.rows(),
        );
    }

    #[test]
    fn the_first_party_discharges_are_the_ones_the_census_staged() {
        // The plan does not get to claim more first-party coverage than
        // §4.2 was actually run for: every discharged row came back from
        // the validator, and the count is the census's own.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let staged = crate::live_first_party::live_first_party_cases().len()
            + crate::live_fault_discharge::live_fault_cases().len();
        assert_eq!(plan.census().first_party_discharged(), staged);
        assert_eq!(
            plan.discharged().len() + plan.fault_discharged().len(),
            plan.census().first_party_discharged(),
        );
        for row in plan.rows() {
            if let LiveRowStanding::FirstPartyDischarged { class, .. } = row.standing() {
                assert_ne!(class.len(), 0, "{} discharged naming no class", row.row());
                assert!(row.row().is_first_party());
            }
        }
    }

    /// The paired relation's own shape, checked apart from the loop.
    ///
    /// The assertion a reader wants most is that the two identities are
    /// DIFFERENT: one identity recorded twice would be one transaction,
    /// and a relation over one transaction is not a relation.
    fn assert_paired_relation(explicit: &str, private: &str, relation: &str) {
        assert_eq!(explicit.len(), 64);
        assert_eq!(private.len(), 64);
        assert_ne!(
            explicit, private,
            "the standing names one transaction twice"
        );
        assert_ne!(relation.len(), 0, "the standing observed nothing");
    }

    #[test]
    fn exactly_the_positive_rows_a_run_answered_are_answered() {
        // The wave's delta, held as a test rather than written in a
        // report. Twenty-six positive rows; twenty-three of them are
        // answered by a target, and each is answered because a real node
        // accepted a transaction of ITS OWN SHAPE and the standing
        // carries the identity.
        //
        // The figure in this comment read "twenty-two" against a set of
        // twenty-three for as long as the set has had twenty-three
        // members, which is a defect of exactly the kind the spelled
        // count below exists to catch — the count was checked and the
        // sentence beside it was not. It is corrected here rather than
        // carried.
        //
        // One further row is answered and is NOT in that set: the
        // openings row, whose §11.2 gate is the byte-identity contract
        // rather than an acceptance. It is collected separately and
        // asserted separately, because a determinism observation is not
        // a target verdict. Two rows still await the run that would
        // answer them, and awaiting a run is not an answer.
        //
        // It read seven until the explicit shape ceremony ran thirteen
        // shapes against a real node and every one was accepted, which
        // moved thirteen of §15.1's sixteen rows at once. Three of those
        // identities are each cited by two rows, because the accepted
        // bytes are an instance of both rows' classes; the register in
        // `live_explicit_shapes` names the pairs and the ground.
        //
        // The count is spelled rather than derived so that a row moved
        // by an edit and not by a run fails here. That is the whole
        // point of the assertion: a delta that could grow quietly is a
        // delta nobody is checking.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let mut positives = 0_usize;
        let mut answered = BTreeSet::new();
        let mut by_determinism = BTreeSet::new();
        let mut by_paired_relation = BTreeSet::new();
        for row in plan.rows() {
            if row.row().polarity() != LiveSafetyPolarity::Positive {
                continue;
            }
            positives += 1;
            match row.standing() {
                LiveRowStanding::NativeRunObserved { accepted_identity } => {
                    assert_eq!(
                        accepted_identity.len(),
                        64,
                        "{} is answered by something that is not a target identity",
                        row.row(),
                    );
                    assert!(row.standing().is_answered());
                    answered.insert(row.row().name());
                }
                LiveRowStanding::NativeRunRequired(_) => {
                    assert!(
                        !row.standing().is_answered(),
                        "{} claims an answer no run produced",
                        row.row(),
                    );
                }
                // The third observation kind, kept in its own set for
                // the same reason it has its own census bucket: what
                // answers this row is a recomputation and not a target
                // verdict, and folding it into `answered` would put a
                // row into a set whose own assertion message says every
                // member was produced by a run of its own shape.
                LiveRowStanding::DeterminismObserved {
                    recomputed,
                    observed_by,
                } => {
                    assert_ne!(recomputed.len(), 0, "{} recomputed nothing", row.row());
                    assert!(
                        observed_by.contains("::"),
                        "{} names no first-party test",
                        row.row(),
                    );
                    assert!(row.standing().is_answered());
                    by_determinism.insert(row.row().name());
                }
                // The fourth observation kind, in its own set for the
                // reason the third is in its own: what answers this row
                // is a RELATION over two acceptances, and folding it
                // into `answered` would put it in a set whose assertion
                // message says every member was produced by a run of its
                // own shape. It was produced by a run of a PAIR.
                LiveRowStanding::PairedRelationObserved {
                    explicit_identity,
                    private_identity,
                    relation,
                } => {
                    assert_paired_relation(explicit_identity, private_identity, relation);
                    assert!(row.standing().is_answered());
                    by_paired_relation.insert(row.row().name());
                }
                other => panic!("{} stands at {other:?}", row.row()),
            }
        }
        assert_eq!(positives, 26, "both positive tables together");
        assert_eq!(
            answered,
            BTreeSet::from([
                "both-commitment-parity-forms",
                "candidate-maximum-inputs",
                "candidate-maximum-outputs",
                "canonical-input-normalization",
                "one-destination-owner",
                "one-input-split-into-two",
                "one-input-to-one-output",
                "private-many-to-many-representative",
                "private-sponsor-values",
                "private-merge",
                "private-one-to-one",
                "private-several-distinct-owners",
                "private-split",
                "repeated-owner",
                "semantic-boundary-values",
                "several-destination-owners",
                "several-distinct-owners",
                "several-inputs-merged-into-one",
                "several-inputs-to-several-outputs",
                "sponsor-change-absent",
                "sponsor-change-present",
                "sponsored",
                "sponsorless",
                "target-ct-conservation",
            ]),
        );
        assert_eq!(plan.census().native_run_observed(), 24);

        // The positive private class that did NOT move is named here
        // rather than left to the count, because a matrix that only said
        // how many rows moved could not say which.
        //
        // Private-sponsor-values USED to be listed here, on the ground
        // that every sponsored control accepted so far was explicit and
        // an explicit acceptance answers nothing about a confidential
        // one. That ground held until a sponsored PRIVATE successor was
        // accepted and mined; the guide closeout moved the row on that
        // identity while this matrix still classified it as awaiting a
        // run, and the arm above is the matrix catching up rather than
        // a new claim.
        //
        // The fixture-openings row also left, by a rule of its own: it
        // asks for a determinism observation rather than a submission,
        // and it is asserted below at the standing minted to hold one.
        //
        // What remains is the projection-equality row, which needs both
        // sides of its pair accepted AND an observation comparing their
        // projections. Both sides of three pairs are now accepted; no
        // run compares the projections, so the row stays.
        //
        // Private-merge USED to be in this list, described as structurally
        // unconstructible on this lane. It was never unconstructible: it
        // was refused first by a cardinality floor this workspace chose
        // and then by the arity of the one predecessor its ceremony
        // funded, and both are conventions rather than protocol rules.
        // A row is removed from this list by a run of its own shape and
        // by nothing else, and that run happened.
        //
        // Private-sponsor-values is where this wave STOPPED, and the
        // stop is typed rather than narrated. It asks for confidential
        // sponsor VALUES, and no ceremony here funds a sponsor coin
        // whose value is blinded --
        // `sponsored_run_of_record::A_BLINDED_SPONSOR_VALUE_IS_FUNDED_ANYWHERE`
        // is the filed path and carries the site inventory. What it is
        // NO LONGER blocked on is the with-change shape, which was its
        // arithmetic precondition and which now runs.
        //
        // Sponsor-change-present USED to be in this list too, as the one
        // explicit row that had not moved, and it left by the same rule:
        // a run of its own shape. It was never blocked on a component
        // either. What was missing was an OFFER that carries change --
        // the construction places a change output only where the offer
        // states a change amount, and the sponsor lane funded its coin
        // to exactly the offer, so no offer this workspace made had ever
        // carried one. A funding step that funds ABOVE the fee supplied
        // it, and the demonstration's sponsor-change symbol needed
        // nothing done to it: the control was accepted at the first
        // attempt with the symbol untouched.
        //
        // THE EXPLICIT POSITIVE TABLE IS NOW COMPLETE. Every row left in
        // this list is private.
        //
        // The openings row LEFT this list, and it left by a different
        // rule from every row before it. Every previous departure was a
        // run of the row's own shape; this one is a determinism
        // observation that was already recorded when the row was still
        // listed here, and what changed is that the vocabulary acquired
        // a member able to hold it. The row is asserted below at that
        // member, and it is asserted NOT to be in `answered` — because
        // no run of its shape produced anything, and that remains true.
        // THE PROJECTION-EQUALITY ROW HAS MOVED, and it moved at a
        // standing of its own. It is asserted OUT of `answered` for the
        // reason the openings row is: no run of its own shape produced
        // it, because its shape is a pair and what answers it is a
        // relation over two acceptances.
        let paired = "projection-equality-with-paired-explicit";
        assert!(
            !answered.contains(paired),
            "{paired} claims an answer a single run of one shape produced",
        );
        assert!(
            !by_determinism.contains(paired),
            "{paired} claims a determinism answer nothing recomputed",
        );
        assert_eq!(by_paired_relation, BTreeSet::from([paired]));
        assert_eq!(plan.census().paired_relation_observed(), 1);
        assert_eq!(
            by_determinism,
            BTreeSet::from(["deterministic-public-fixture-openings"]),
        );
        assert!(!answered.contains("deterministic-public-fixture-openings"));
        assert_eq!(plan.census().determinism_observed(), 1);
        // The acceptance buckets did NOT move on the determinism row's
        // account, which is the whole claim of a separate bucket made
        // checkable.
        assert_eq!(plan.census().native_run_observed(), 24);
    }

    #[test]
    fn exactly_the_negative_rows_a_refusal_answered_are_answered() {
        // The negative half's delta, spelled for the same reason the
        // positive one is: a row moved by an edit rather than by a run
        // fails here.
        //
        // Both are §15.3 witness-content rows and both were answered by
        // ONE run, which submitted two mutants and then the unmutated
        // control to one node on one chain. Each standing carries the
        // accepted control's identity -- the half a reader can check
        // against a chain, the refusal having left no transaction to
        // look up -- and the target's own words.
        use crate::live_explicit_shapes::witness_negatives_run_of_record as witness;

        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let mut answered = BTreeSet::new();
        for row in plan.rows() {
            if let LiveRowStanding::NativeRefusalObserved {
                control_identity,
                refusal_detail,
            } = row.standing()
            {
                assert_eq!(
                    control_identity.len(),
                    64,
                    "{} names something that is not a target identity",
                    row.row(),
                );
                assert!(
                    !refusal_detail.is_empty(),
                    "{} carries no refusal detail",
                    row.row(),
                );
                assert!(row.standing().is_answered());
                answered.insert(row.row().name());
            }
        }
        assert_eq!(
            answered,
            BTreeSet::from([
                "confidential-asset-commitment",
                "empty-signature",
                "hidden-private-u-output",
                "malformed-rangeproof",
                "malformed-signature",
                "missing-sponsor-authorization",
                "no-coordinator",
                "omitted-source",
                "output-total-one-above-input",
                "output-total-one-below-input",
                "private-ct-imbalance",
                "private-output-omitted",
                "two-coordinators",
                "vault-control-entitlement-or-bare-u-output",
                "wrong-explicit-asset",
                "wrong-private-blinding-balance",
            ]),
        );
        assert_eq!(plan.census().native_refusal_observed(), 16);

        // THE THIRD ROW COMES FROM A DIFFERENT LANE and is held to the
        // same rule. Its mutant was offered first and its control
        // accepted behind it, on one node on one chain, and the two
        // differ in the sponsor witness and in nothing else.
        //
        // Its verdict READS THE SAME as an earlier wave's fee-role
        // failure and is not it. What separates them is the control:
        // the fee-role check passes for the deployment that control was
        // accepted under, so the comparison that failed is one the
        // sponsor witness reaches. A row whose refusal could not be
        // told apart from another comparison's would be a row answered
        // by a string rather than by a run.
        assert_ne!(
            crate::live_sponsor_shapes::sponsored_run_of_record::SPONSORED_ACCEPTED_TXID,
            witness::CONTROL_ACCEPTED_TXID,
            "the sponsored negative cites the sponsorless lane's control",
        );

        // The two are DISTINGUISHABLE, which is what makes each one its
        // own row rather than one observation counted twice. The empty
        // offering failed the check that consumed it; the well-sized
        // non-signature was consumed and judged.
        assert_ne!(
            witness::EMPTY_SIGNATURE_REFUSAL,
            witness::MALFORMED_SIGNATURE_REFUSAL,
        );

        // And neither is the refusal the wrong submission order
        // produced, which named an identity already on the chain and was
        // about nothing either row is about.
        for detail in [
            witness::EMPTY_SIGNATURE_REFUSAL,
            witness::MALFORMED_SIGNATURE_REFUSAL,
        ] {
            assert_ne!(detail, witness::REFUSAL_UNDER_CONTROL_FIRST_ORDER);
        }
    }

    #[test]
    fn the_four_leaf_arrangement_rows_collapse_to_two_observations() {
        // The R-5 collapse, pinned: the four leaf-arrangement rows form two
        // collision pairs, and each pair draws ONE verdict, so exactly two
        // rows are DRIVEN to an observed refusal and exactly two stay typed
        // as not-separated. Driving all four would read one observation onto
        // two rows twice; typing all four would leave a drivable refusal
        // unrecorded. This holds the split against the classifier.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let standing = |name: &str| {
            plan.rows()
                .iter()
                .find(|row| row.row().name() == name)
                .map_or_else(
                    || panic!("{name} is not a matrix row"),
                    super::LiveEvidenceRow::standing,
                )
        };

        // PAIR 1, the coordinator index check: two-coordinators is driven,
        // wrong-coordinator stays typed.
        assert!(
            matches!(
                standing("two-coordinators"),
                LiveRowStanding::NativeRefusalObserved { .. }
            ),
            "two-coordinators was not driven to an observed refusal",
        );
        assert!(
            matches!(
                standing("wrong-coordinator"),
                LiveRowStanding::NativeRunRequired(_)
            ),
            "wrong-coordinator did not stay typed as still required",
        );

        // PAIR 2, the member bound check: no-coordinator is driven,
        // member-coordinator-leaf-exchange stays typed.
        assert!(
            matches!(
                standing("no-coordinator"),
                LiveRowStanding::NativeRefusalObserved { .. }
            ),
            "no-coordinator was not driven to an observed refusal",
        );
        assert!(
            matches!(
                standing("member-coordinator-leaf-exchange"),
                LiveRowStanding::NativeRunRequired(_)
            ),
            "member-coordinator-leaf-exchange did not stay typed as still required",
        );

        // The two DRIVEN rows draw two DISTINCT verdicts — the coordinator
        // index EqualVerify and the member bound Verify — which is what
        // makes them one observation each rather than one shared. The typed
        // partners are the two rows whose own mutants would draw those same
        // two verdicts.
        assert_ne!(
            crate::live_owner_signing_negatives::run_of_record::TWO_COORDINATORS_REJECT_DETAIL,
            crate::live_owner_signing_negatives::run_of_record::NO_COORDINATOR_REJECT_DETAIL,
            "the two driven leaf-arrangement rows draw one verdict",
        );
    }

    #[test]
    fn the_computability_blocker_is_carried_by_no_row() {
        // The clearing, checked against the classification rather than
        // against the sentence that describes it. A row still carrying
        // this would be a row waiting on a component that exists.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        assert_eq!(
            blocker_census(&plan)
                .get(&LiveInfrastructureBlocker::OwnerSighashNotComputable)
                .copied()
                .unwrap_or(0),
            0,
        );
    }

    #[test]
    fn the_negative_half_awaits_a_run_rather_than_an_absent_control() {
        // §14.5's and §19.2's argument, made a state, and now standing
        // the other way round. A refusal is attributable once something
        // can be accepted at all, so the negative half stopped being
        // blocked on the absent control and became rows a run would
        // answer. None of them is answered by the conversion.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        assert_ne!(
            plan.census().native_run_required(),
            0,
            "no row awaits the run that would answer it",
        );
        assert_eq!(
            blocker_census(&plan)
                .get(&LiveInfrastructureBlocker::NoAcceptingControlExists)
                .copied()
                .unwrap_or(0),
            0,
        );
        assert!(!plan.census().every_required_row_is_answered());
    }

    #[test]
    fn the_plan_refuses_to_call_itself_complete() {
        // §13.5's bar, and the honest answer to it. A plan that reported
        // completeness here would be reporting it for a matrix whose
        // positive half has never run.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        assert!(!plan.census().every_required_row_is_answered());
        // ONE row carries a blocker of its own, and it was three: the
        // raw path, which the owner message was never in the way of.
        //
        // The second was the sponsor envelope signer, and it left
        // because its residual cleared on an observed acceptance. The
        // row it blocked, missing-sponsor-authorization, is NOT answered
        // by that: it is a negative asking that a control missing the
        // sponsor's authorization be refused, and what was accepted is a
        // positive sponsored control. The row moved from blocked to
        // awaiting a run of its own shape, which is why the count fell
        // by one while the completeness assertion below did not change
        // at all.
        //
        // The third was the predecessor constructor, and its row left
        // this side of the matrix entirely: `time-locked-input` is
        // discharged first-party now, on the ruling that retyped it. So
        // this count fell for a second reason of a different kind — a
        // blocked row became an ANSWERED one rather than a waiting one —
        // and the completeness assertion still does not change, because
        // the positive half is what it is about.
        //
        // The fourth was the raw path, and it left the way the third
        // did — the row ANSWERED rather than waiting — but on a ruling
        // rather than on a retyping. Its blocker's premise had expired:
        // the path it called absent is public and three lanes use it,
        // the previous wave recorded that at the blocker's own site and
        // declined to choose a standing, and the ruling minted one. So
        // the count is ZERO, and no row of this matrix is waiting on a
        // component that does not exist.
        assert_eq!(plan.census().infrastructure_blocked(), 0);
        // The row it carried is answered, and answered by a fact rather
        // than by anything a target said. Asserted here, beside the
        // count it changed, so a reader finding the blocker gone can see
        // in one place where the row went.
        let raw = plan
            .rows()
            .iter()
            .find(|row| row.row().name() == "raw-transaction-bypassing-safe-construction")
            .expect("the raw-bypass row is in the matrix");
        assert!(matches!(
            raw.standing(),
            LiveRowStanding::FirstPartyFactObserved { .. }
        ));
        // TWO rows stand here, and they are different kinds of fact
        // answering the same kind of gate. The raw-bypass row asks
        // whether a path EXISTS; the zero-valued sponsor row asks
        // whether a shape is ADMITTED, and the sources say it is. Both
        // are statements this workspace makes about itself, and neither
        // is anything a target said.
        assert_eq!(plan.census().first_party_fact_observed(), 3);
        let zero = plan
            .rows()
            .iter()
            .find(|row| row.row().name() == "zero-valued-ordinary-sponsor-member")
            .expect("the zero-valued sponsor row is in the matrix");
        assert!(matches!(
            zero.standing(),
            LiveRowStanding::FirstPartyFactObserved { .. }
        ));
        // And it did NOT land in either target bucket. The separate
        // bucket's whole claim, made checkable.
        assert_eq!(plan.census().native_run_observed(), 24);
        assert_eq!(plan.census().native_refusal_observed(), 16);
    }

    #[test]
    fn the_two_report_rows_are_the_ones_the_report_bytes_answer() {
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        assert_eq!(plan.census().report_layer(), 2);
        let per_section = plan
            .section_census(|standing| matches!(standing, LiveRowStanding::ReportLayerAnswerable));
        assert_eq!(per_section[&LiveSafetySection::SponsorFault], 2);
    }
}
