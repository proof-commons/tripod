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
//! No positive row of §15.1 or §15.2 is answerable today, and the reason
//! is one missing component rather than a shortage of effort:
//! [`LiveInfrastructureBlocker::OwnerSighashNotComputable`]. §10.2 checks
//! an owner signature with the target's own verifying primitive over the
//! target's own taproot sighash, and no first-party component in this
//! workspace computes that digest — §1.7 leaves it to the target and
//! forbids a builder from asserting one. So no valid transfer can be
//! witnessed, and no target can accept one.
//!
//! That blocks the negative half too, and the plan says so rather than
//! collecting refusals. A census of rejections from a pipeline that has
//! never had a transaction accepted establishes that the target rejects
//! things, which every target that rejects everything also does; §14.5's
//! positive class witnesses and the conformance package's own
//! `NoAcceptingCase` defect are the same argument made twice already.
//! [`LiveInfrastructureBlocker::NoAcceptingControlExists`] is that
//! argument made a third time, as a state a row can be in.

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
    /// [`a_positive_control_exists`] computes to true. Both moves are on
    /// an observed result and neither is on a capability existing.
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
    /// exact finalized transaction. Nothing here reaches it. The
    /// distinction is the whole point of naming a blocker precisely:
    /// clearing this one is an integration, not a design.
    ///
    /// Wiring it is still not enough to remove the blocker. §1.9 asks
    /// for the sponsor owner's *target authorization*, and a returned
    /// byte stack is not that until a target has accepted a control
    /// carrying it — which needs the owner sighash first. A blocker
    /// moves on an observed result and never on a capability existing.
    SponsorEnvelopeSignerAbsent,
    /// No predecessor exists to build the spend from.
    ///
    /// The time-locked-predecessor row's blocker: §7.3 makes the two
    /// constructors distinct and this workspace builds only the live one,
    /// so there is no time-locked output to offer a live transfer leaf.
    PredecessorConstructorAbsent,
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
    /// One of the five §15.4–§15.7 fault entry points.
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
            Self::FirstPartyDischarged { .. } | Self::ReportLayerAnswerable
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
/// Three carry a blocker of their own as well, and the more specific one
/// wins: a reader repairing the pipeline needs to know that these three
/// would still be outstanding after the digest existed.
fn specific_blocker(row: &LiveSafetyRow) -> Option<LiveInfrastructureBlocker> {
    match row.name() {
        "time-locked-input" => Some(LiveInfrastructureBlocker::PredecessorConstructorAbsent),
        "missing-sponsor-authorization" => {
            Some(LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent)
        }
        "raw-transaction-bypassing-safe-construction" => {
            Some(LiveInfrastructureBlocker::RawSurgeryPathAbsent)
        }
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

/// The bytes that stand in a signature position no signer can fill.
///
/// Not a signature, and named so at every use. §10.2's fragment checks
/// the target's own verifying primitive over the target's own taproot
/// sighash, and [`LiveInfrastructureBlocker::OwnerSighashNotComputable`]
/// records that nothing here computes that digest; §1.9 puts a sponsor's
/// authorization outside protocol data and
/// [`LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent`] records
/// that no adapter signer is wired into this lane to supply one.
///
/// So a witness position that has to be *filled* — to serialize a
/// transaction at all, or to weigh one — is filled with bytes of the
/// right width that authorize nothing. Stated once, here beside the two
/// blockers that are the reason for it, so that three modules cannot
/// drift into three different widths and quietly change what every
/// measured weight is a weight of.
pub const UNAUTHORIZING_SIGNATURE: [u8; 64] = [0x5c; 64];

/// The residuals this plan inherits and does not clear.
///
/// Named as a set rather than as prose so that a later wave clearing one
/// has to remove it here, and so a reader can see at a glance which of
/// them are this workspace's to clear.
///
/// The doc comment stating that was sitting above
/// [`UNAUTHORIZING_SIGNATURE`] rather than above this function, which is
/// where the two are separated back. It is repaired here rather than
/// left because this function is what the paragraph is about and a
/// reader following it would otherwise be told the constant is a set.
///
/// Two members, and it was three. The one that left is
/// [`LiveInfrastructureBlocker::SighashProfileUnreviewed`], cleared by
/// the owner-sighash review verdict together with the
/// post-verdict re-typing — by a verdict, that is, and never by a
/// run, which is the discipline that residual was separated from the
/// digest blocker in order to keep.
#[must_use]
pub fn carried_residuals() -> BTreeSet<LiveInfrastructureBlocker> {
    BTreeSet::from([
        LiveInfrastructureBlocker::SponsorEnvelopeSignerAbsent,
        LiveInfrastructureBlocker::PredecessorConstructorAbsent,
    ])
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
        // The matrix's pre-target half, after both censuses. Twenty-five
        // rows of §15 are refused before any target sees the bytes, and
        // every one of them has been driven to its own refusal against
        // its own control. Nothing here is outstanding, and nothing here
        // was closed by a wildcard: the two rows that left this half
        // left because their declared boundary was wrong, and the count
        // is recomputed from the classification rather than adjusted.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let census = plan.census();
        assert_eq!(census.first_party_discharged(), 25);
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
        // The supporting count is zero and that is a fact about the
        // registry, not a minimality conclusion — which
        // `crate::live_minimality_report` is the only thing entitled to
        // draw.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        assert!(plan.minimality().is_built());
        assert_eq!(
            plan.minimality(),
            MinimalityRegistryStanding::Built {
                pairs: crate::live_pairs::MinimalityPair::ALL.len(),
                supporting: 0,
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

    #[test]
    fn no_positive_row_is_answered_and_every_one_of_them_awaits_a_run() {
        // What the observation converted, and what it did not. Every
        // positive row stopped being infrastructure-blocked, because the
        // component they were all waiting on exists; not one of them
        // became answered, because a standing is not evidence and no run
        // of any row has been filed.
        let plan = derive_live_evidence_plan().expect("the evidence plan derives");
        let mut positives = 0_usize;
        for row in plan.rows() {
            if row.row().polarity() != LiveSafetyPolarity::Positive {
                continue;
            }
            positives += 1;
            assert!(
                matches!(row.standing(), LiveRowStanding::NativeRunRequired(_)),
                "{} does not await the run that would answer it",
                row.row(),
            );
            assert!(
                !row.standing().is_answered(),
                "{} claims an answer no run produced",
                row.row(),
            );
        }
        assert_eq!(positives, 26, "both positive tables together");
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
        // Three rows carry a blocker of their own, and those three do
        // not move: a predecessor constructor, a sponsor envelope
        // signer, and a raw path, none of which the owner message was
        // ever in the way of.
        assert_eq!(plan.census().infrastructure_blocked(), 3);
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
