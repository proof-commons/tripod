//! Signature, sighash, and relative-timelock dimensions.
//!
//! This module states what the target *offers*. It does not choose
//! what the project will use. No sighash profile is selected here, no
//! cadence band is encoded here, and nothing here asserts that a
//! target signature check establishes any protocol authorization: a
//! verified signature is one later evidence layer, not a proof that
//! the right party approved the right thing.

use std::collections::{BTreeMap, BTreeSet};

use crate::capability::census_enum;
use crate::encoding::EncodingClass;
use crate::evidence::TargetEvidenceRequirementId;
use crate::opcode::FailureOutcome;

census_enum! {
    /// One dimension a sighash may commit to.
    #[non_exhaustive]
    pub enum SighashDimension {
        /// Every output of the transaction.
        AllOutputs,
        /// The single output at the signing input's index.
        SingleOutput,
        /// Every input of the transaction.
        AllInputs,
        /// Only the signing input, permitting other inputs to be added.
        InputExtensionPermitted,
        /// The issuance fields of the inputs.
        Issuance,
        /// The transaction version field.
        Version,
        /// The transaction locktime field.
        LockTime,
        /// The hash of the executing leaf.
        TapleafHash,
        /// The taproot internal key.
        InternalKey,
        /// The full set of outputs being spent.
        SpentOutputs,
    }
}

/// Where the source review read one dimension's term in the target.
///
/// Carried as data rather than left in a doc comment because the
/// accepted evidence ruling makes the citation half of what a reviewed
/// dimension *is*. A reviewed set whose citations lived only in prose
/// would be a set a later edit could grow without anyone having read
/// anything, which is the source-review-alone option the owner closed.
///
/// The three fields are the three questions a reader of a reviewed
/// dimension asks: which terms of the message carry it, where the
/// target writes them, and where the reading that says so is recorded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SighashSourceCitation {
    terms: &'static str,
    written_at: &'static str,
    review_anchor: &'static str,
}

impl SighashSourceCitation {
    /// States one citation.
    #[must_use]
    pub const fn new(
        terms: &'static str,
        written_at: &'static str,
        review_anchor: &'static str,
    ) -> Self {
        Self {
            terms,
            written_at,
            review_anchor,
        }
    }

    /// Which terms of the message carry the dimension.
    #[must_use]
    pub const fn terms(&self) -> &'static str {
        self.terms
    }

    /// Where the target writes them.
    #[must_use]
    pub const fn written_at(&self) -> &'static str {
        self.written_at
    }

    /// Where the reading that says so is recorded.
    #[must_use]
    pub const fn review_anchor(&self) -> &'static str {
        self.review_anchor
    }
}

/// The run whose acceptance a reviewed dimension rests on.
///
/// Three fields and not one string, because "an observation" that named
/// only a transaction identity would be unfindable a month later: the
/// ceremony case says which of the run's cases it was, the identity says
/// which transaction the target accepted, and the record says where the
/// run is written down. The last matters more than it looks — this
/// workspace commits no transcript of the ceremony, so the observation's
/// only durable home is the backlog row and the merge that carries it,
/// and a citation that did not say so would be pointing at nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObservationIdentity {
    ceremony_case: &'static str,
    accepted_transaction: &'static str,
    recorded_at: &'static str,
}

impl ObservationIdentity {
    /// States one observation's identity.
    #[must_use]
    pub const fn new(
        ceremony_case: &'static str,
        accepted_transaction: &'static str,
        recorded_at: &'static str,
    ) -> Self {
        Self {
            ceremony_case,
            accepted_transaction,
            recorded_at,
        }
    }

    /// The ceremony's own name for the case.
    #[must_use]
    pub const fn ceremony_case(&self) -> &'static str {
        self.ceremony_case
    }

    /// The transaction identity the target accepted.
    #[must_use]
    pub const fn accepted_transaction(&self) -> &'static str {
        self.accepted_transaction
    }

    /// Where the run is recorded.
    #[must_use]
    pub const fn recorded_at(&self) -> &'static str {
        self.recorded_at
    }
}

/// What exercised one dimension, beyond the reading of the source.
///
/// # What counts as exercising a dimension
///
/// The accepted spend and the recomputation are one another's halves: a
/// node formed its own message and ran its own verifying primitive, and
/// an independently written construction formed the same message from
/// the source review's term table. Their agreement is over the whole
/// stream at once, so a model wrong about *any* term — its position, its
/// width, its byte order, or its content — would have produced a
/// different digest and a signature that verified against nothing.
///
/// That is what makes the agreement per-dimension evidence, and it is
/// also what bounds it. The agreement exercises a dimension only where
/// the recomputation actually read that dimension's own subject out of
/// the candidate. A term the recomputation synthesizes without consulting
/// the dimension's subject cannot be wrong about it, and a model that
/// cannot be wrong about a dimension is not checked about it by any
/// number of agreeing digests.
///
/// # Why the controls are a slice
///
/// Zero or many, because a negative control is a strengthening and not a
/// requirement. A control moves one term alone and observes the target
/// refuse, which turns "the target agreed on this value" into "the target
/// distinguishes this value from another". Recording the count honestly
/// is the point: most of the message's terms have no control, and a
/// vocabulary that made one mandatory would have invited an unmoved term
/// to be filed as a moved one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExercisingObservation {
    observation: ObservationIdentity,
    controls: &'static [&'static str],
}

impl ExercisingObservation {
    /// States what exercised one dimension.
    #[must_use]
    pub const fn new(observation: ObservationIdentity, controls: &'static [&'static str]) -> Self {
        Self {
            observation,
            controls,
        }
    }

    /// The accepted spend this dimension rests on.
    #[must_use]
    pub const fn observation(&self) -> ObservationIdentity {
        self.observation
    }

    /// The negative controls that moved a term of this dimension alone.
    #[must_use]
    pub const fn controls(&self) -> &'static [&'static str] {
        self.controls
    }
}

/// What established one dimension the review moved to reviewed.
///
/// Both halves, never one. The accepted evidence ruling is source review
/// *and* recomputation *and* one observed acceptance, and a ground that
/// could hold a citation without an observation would let the first of
/// the three pass as all of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReviewedGround {
    citation: SighashSourceCitation,
    exercised_by: ExercisingObservation,
}

impl ReviewedGround {
    /// States what established one dimension.
    #[must_use]
    pub const fn new(citation: SighashSourceCitation, exercised_by: ExercisingObservation) -> Self {
        Self {
            citation,
            exercised_by,
        }
    }

    /// Where the source review read the dimension's terms.
    #[must_use]
    pub const fn citation(&self) -> SighashSourceCitation {
        self.citation
    }

    /// What exercised it.
    #[must_use]
    pub const fn exercised_by(&self) -> ExercisingObservation {
        self.exercised_by
    }
}

/// Why one dimension is not in the reviewed set.
///
/// Three members and no fourth, because the three are the three
/// *different* things "not reviewed" has meant in this arc, and merging
/// any two of them would lose the one fact a later wave needs: whether
/// the dimension is reachable at all, and by whom.
///
/// None of them is "nobody looked". Every member carries the reading
/// that reached the dimension, so an unreviewed dimension here is one
/// the review arrived at and could not exercise — never one it skipped.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum UnreviewedGround {
    /// The message construction contains no term for the dimension at
    /// all, so no reading of the message and no recomputation of it can
    /// ever move it.
    ///
    /// Permanent by construction rather than pending: this is not a
    /// dimension awaiting a run. The protection the profile wanted from
    /// it is recorded as carried by another dimension, and the selector
    /// that says which one carries the argument.
    NoMessageTermCarriesIt {
        /// Where the reading that settles it is recorded.
        review_anchor: &'static str,
    },
    /// The message carries the dimension and the selected profile
    /// refuses it, so no candidate this arc produces takes the branch
    /// that would exercise it.
    ///
    /// A refusal is not a gap. The dimension stays unreviewed because
    /// establishing it would mean exercising a commitment the profile
    /// declines to make, and a reviewed set that grew by exercising
    /// refused dimensions would be establishing a different profile.
    TheSelectedProfileRefusesIt(SighashSourceCitation),
    /// The dimension's terms are in every message this arc forms, and no
    /// candidate this arc can build puts the dimension's own subject
    /// into them.
    ///
    /// The sharpest of the three and the only one that blocks a required
    /// dimension. The terms are present, the target writes them, and the
    /// recomputation reproduces them — but it reproduces them from
    /// something other than the dimension's subject, so their agreement
    /// says nothing about the subject. What repairs it is a candidate
    /// carrying the subject, which is a construction this arc refuses by
    /// type rather than a run nobody has scheduled.
    NoCandidateThisArcBuildsCarriesTheSubject(SighashSourceCitation),
}

/// Which sighash dimensions the review actually established.
///
/// # Why this is split rather than a set of booleans
///
/// The source review covered the signature primitives themselves —
/// their operands, their two distinct failure modes, and their budget
/// accounting — but it did not cover the sighash type enumeration or
/// the message construction. Recording the dimensions as a set of
/// booleans would have forced a yes-or-no answer to questions the
/// review never asked, and the resulting contract would have read as
/// authoritative.
///
/// So each dimension is classified as reviewed or not reviewed.
/// Absence from the reviewed set means "this package has not established
/// it", never "the target does not offer it". A downstream selector must
/// treat an unreviewed dimension as unavailable and say why.
///
/// # Why each side carries a ground
///
/// A bare set member is a claim with nothing behind it, and the two sets
/// were bare when the review had established nothing — which cost
/// nothing while both were empty of content. They stop being free the
/// moment a dimension moves: a reviewed set of bare members could be
/// grown by an edit that read no source and ran nothing, and an
/// unreviewed set of bare members cannot tell a dimension nobody reached
/// from one that was reached and could not be exercised.
///
/// So each side is a map to its ground. [`ReviewedGround`] carries both
/// halves the accepted evidence ruling requires, and [`UnreviewedGround`]
/// carries the recorded reason that ruling demands of a dimension the
/// review could not exercise.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SighashCapability {
    reviewed: BTreeMap<SighashDimension, ReviewedGround>,
    unreviewed: BTreeMap<SighashDimension, UnreviewedGround>,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
}

impl SighashCapability {
    /// States the sighash review.
    #[must_use]
    pub fn new(
        reviewed: impl IntoIterator<Item = (SighashDimension, ReviewedGround)>,
        unreviewed: impl IntoIterator<Item = (SighashDimension, UnreviewedGround)>,
        evidence: impl IntoIterator<Item = TargetEvidenceRequirementId>,
    ) -> Self {
        Self {
            reviewed: reviewed.into_iter().collect(),
            unreviewed: unreviewed.into_iter().collect(),
            evidence: evidence.into_iter().collect(),
        }
    }

    /// The dimensions the review established, each with its ground.
    #[must_use]
    pub const fn reviewed(&self) -> &BTreeMap<SighashDimension, ReviewedGround> {
        &self.reviewed
    }

    /// The dimensions the review did not establish, each with its
    /// recorded reason.
    #[must_use]
    pub const fn unreviewed(&self) -> &BTreeMap<SighashDimension, UnreviewedGround> {
        &self.unreviewed
    }

    /// Whether one dimension is established.
    ///
    /// Published so that a selector recomputing a disposition asks the
    /// capability rather than reaching into a collection, and so that
    /// the question has one spelling across the workspace.
    #[must_use]
    pub fn is_reviewed(&self, dimension: SighashDimension) -> bool {
        self.reviewed.contains_key(&dimension)
    }

    /// The evidence a deployment must produce for the sighash.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }

    /// Any dimension classified both ways.
    #[must_use]
    pub fn contradictory(&self) -> Option<SighashDimension> {
        self.reviewed
            .keys()
            .find(|dimension| self.unreviewed.contains_key(dimension))
            .copied()
    }

    /// Any dimension classified neither way.
    #[must_use]
    pub fn unclassified(&self) -> Option<SighashDimension> {
        SighashDimension::ALL
            .iter()
            .find(|dimension| {
                !self.reviewed.contains_key(dimension) && !self.unreviewed.contains_key(dimension)
            })
            .copied()
    }
}

/// What the target does with a public key whose encoding it does not
/// recognize.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum UnknownPublicKeyTypeRule {
    /// The check reports success *without performing any
    /// verification*, provided the offered signature is non-empty.
    ///
    /// This is the reviewed behavior and it is a sharp edge worth
    /// stating loudly: an unknown key type turns signature
    /// verification into a no-op that returns true. Any backend that
    /// relies on a signature check for authorization must constrain
    /// the key encoding itself rather than trusting the check's
    /// result. The edge is kept as an upstream friction
    /// `(´[PLAN-obs:upstream:eg-008]´)`, and this variant is why a
    /// backend has to ask which rule the deployment runs under.
    SucceedsWithoutVerification,
    /// The check refuses the key.
    Rejected,
}

/// The reviewed contract of the target's signature primitives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignaturePrimitiveContract {
    public_key_encoding: EncodingClass,
    signature_encoding: EncodingClass,
    empty_signature: FailureOutcome,
    invalid_signature: FailureOutcome,
    unknown_public_key_type: UnknownPublicKeyTypeRule,
    budget_per_check: u64,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
}

impl SignaturePrimitiveContract {
    /// States the signature primitive contract.
    #[must_use]
    pub fn new(
        public_key_encoding: EncodingClass,
        signature_encoding: EncodingClass,
        empty_signature: FailureOutcome,
        invalid_signature: FailureOutcome,
        unknown_public_key_type: UnknownPublicKeyTypeRule,
        budget_per_check: u64,
        evidence: impl IntoIterator<Item = TargetEvidenceRequirementId>,
    ) -> Self {
        Self {
            public_key_encoding,
            signature_encoding,
            empty_signature,
            invalid_signature,
            unknown_public_key_type,
            budget_per_check,
            evidence: evidence.into_iter().collect(),
        }
    }

    /// The encoding a public key operand carries.
    #[must_use]
    pub const fn public_key_encoding(&self) -> EncodingClass {
        self.public_key_encoding
    }

    /// The encoding a signature operand carries.
    #[must_use]
    pub const fn signature_encoding(&self) -> EncodingClass {
        self.signature_encoding
    }

    /// What an empty signature does.
    #[must_use]
    pub const fn empty_signature(&self) -> FailureOutcome {
        self.empty_signature
    }

    /// What a non-empty signature that does not verify does.
    #[must_use]
    pub const fn invalid_signature(&self) -> FailureOutcome {
        self.invalid_signature
    }

    /// What an unrecognized public key encoding does.
    #[must_use]
    pub const fn unknown_public_key_type(&self) -> UnknownPublicKeyTypeRule {
        self.unknown_public_key_type
    }

    /// The validation budget one check can consume.
    #[must_use]
    pub const fn budget_per_check(&self) -> u64 {
        self.budget_per_check
    }

    /// The evidence a deployment must produce for the primitive.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }
}

/// Whether a relative timelock counts blocks or elapsed time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TimelockMode {
    /// The lock counts confirmations.
    BlockHeight,
    /// The lock counts fixed time intervals.
    TimeInterval,
}

/// How a sequence field's bits are divided.
///
/// A value of its own because the three masks are meaningless apart:
/// each covers part of the same word, and a validator has to be able
/// to check that they do not overlap.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SequenceFieldLayout {
    /// The bit that disables the lock entirely.
    pub disable_flag: u32,
    /// The bit that selects block or time mode.
    pub mode_flag: u32,
    /// The bits carrying the lock's magnitude.
    pub value_mask: u32,
    /// The left shift converting a time-mode magnitude to seconds.
    pub time_interval_shift: u8,
}

impl SequenceFieldLayout {
    /// Whether any two of the fields claim the same bit, which would
    /// make them indistinguishable to a decoder.
    #[must_use]
    pub const fn has_overlapping_fields(self) -> bool {
        (self.disable_flag & self.mode_flag) != 0
            || (self.disable_flag & self.value_mask) != 0
            || (self.mode_flag & self.value_mask) != 0
    }
}

/// The reviewed relative-timelock dimensions.
///
/// These are target facts. The protocol's cadence band — the window in
/// which an operator acts and after which anyone may — is not encoded
/// here and must not be: a downstream planner builds that band out of
/// these primitives, and the target has no opinion about it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelativeTimelockContract {
    modes: BTreeSet<TimelockMode>,
    layout: SequenceFieldLayout,
    minimum_transaction_version: u32,
    unsatisfied: FailureOutcome,
    evidence: BTreeSet<TargetEvidenceRequirementId>,
}

impl RelativeTimelockContract {
    /// States the relative-timelock contract.
    #[must_use]
    pub fn new(
        modes: impl IntoIterator<Item = TimelockMode>,
        layout: SequenceFieldLayout,
        minimum_transaction_version: u32,
        unsatisfied: FailureOutcome,
        evidence: impl IntoIterator<Item = TargetEvidenceRequirementId>,
    ) -> Self {
        Self {
            modes: modes.into_iter().collect(),
            layout,
            minimum_transaction_version,
            unsatisfied,
            evidence: evidence.into_iter().collect(),
        }
    }

    /// How the sequence field's bits are divided.
    #[must_use]
    pub const fn layout(&self) -> SequenceFieldLayout {
        self.layout
    }

    /// The modes the target supports.
    #[must_use]
    pub const fn modes(&self) -> &BTreeSet<TimelockMode> {
        &self.modes
    }

    /// The sequence bit that disables the lock entirely.
    #[must_use]
    pub const fn sequence_disable_flag(&self) -> u32 {
        self.layout.disable_flag
    }

    /// The sequence bit that selects block or time mode.
    #[must_use]
    pub const fn sequence_mode_flag(&self) -> u32 {
        self.layout.mode_flag
    }

    /// The sequence bits carrying the lock's magnitude.
    #[must_use]
    pub const fn sequence_value_mask(&self) -> u32 {
        self.layout.value_mask
    }

    /// The left shift converting a time-mode magnitude to seconds.
    #[must_use]
    pub const fn time_interval_shift(&self) -> u8 {
        self.layout.time_interval_shift
    }

    /// The lowest transaction version at which the lock is enforced.
    ///
    /// Below this version the check does not merely pass — the lock is
    /// not enforced at all, so a program relying on it must constrain
    /// the version too.
    #[must_use]
    pub const fn minimum_transaction_version(&self) -> u32 {
        self.minimum_transaction_version
    }

    /// What an unsatisfied lock does.
    #[must_use]
    pub const fn unsatisfied(&self) -> FailureOutcome {
        self.unsatisfied
    }

    /// The evidence a deployment must produce for the lock.
    #[must_use]
    pub const fn evidence(&self) -> &BTreeSet<TargetEvidenceRequirementId> {
        &self.evidence
    }

    /// Whether the flag and mask bits overlap, which would make the
    /// fields indistinguishable.
    #[must_use]
    pub const fn has_overlapping_fields(&self) -> bool {
        self.layout.has_overlapping_fields()
    }
}

/// The target's authorization dimensions, taken together.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorizationContract {
    signature: SignaturePrimitiveContract,
    sighash: SighashCapability,
    relative_timelock: RelativeTimelockContract,
}

impl AuthorizationContract {
    /// States the authorization contract.
    #[must_use]
    pub const fn new(
        signature: SignaturePrimitiveContract,
        sighash: SighashCapability,
        relative_timelock: RelativeTimelockContract,
    ) -> Self {
        Self {
            signature,
            sighash,
            relative_timelock,
        }
    }

    /// The signature primitive contract.
    #[must_use]
    pub const fn signature(&self) -> &SignaturePrimitiveContract {
        &self.signature
    }

    /// The sighash dimensions.
    #[must_use]
    pub const fn sighash(&self) -> &SighashCapability {
        &self.sighash
    }

    /// The relative-timelock dimensions.
    #[must_use]
    pub const fn relative_timelock(&self) -> &RelativeTimelockContract {
        &self.relative_timelock
    }
}

/// The run every reviewed dimension below rests on.
///
/// One observation and not seven, because the accepted evidence ruling
/// asks for *one* target-accepted spend of a first-party candidate whose
/// signature verifies against the recomputed message. What differs
/// between dimensions is not which run exercised them but whether the
/// run's message carried each dimension's own subject.
const OWNER_OBSERVATION: ObservationIdentity = ObservationIdentity::new(
    "selected-profile-authorization",
    "40cb6c4ee284ed38555a4840198c8130d1e2c3246b57b9d8b93842c3c6730029",
    "plans/backlog.md T5-026",
);

/// The dimension-by-dimension verdict of the owner sighash review.
///
/// # What moved, and what moved it
///
/// Six of the profile's seven required dimensions are established, each
/// by the same accepted spend and the same independently written
/// recomputation of its message. The candidate consumed two receipts and
/// created two destinations, so the four whole-transaction dimensions
/// were exercised at a cardinality that distinguishes "every input" from
/// "the signing input" and "every output" from "the output at this
/// index" — a single-input, single-output candidate would have left that
/// distinction unobserved and the honest verdict would have been
/// narrower.
///
/// Two of the six carry a negative control besides. The output side is
/// the one the arc had a recorded hazard for, so it got both: one
/// control emptied the output-witness vector and one exchanged the two
/// destinations, and the target refused each.
///
/// # Why `Issuance` did not move
///
/// The seventh required dimension is the one this review reached and
/// could not exercise, and the reason is a property of this workspace
/// rather than of the target. The target's own message does carry the
/// dimension: term 9 writes each input's issuance or one zero byte where
/// it is null, and term 10 hashes the issuance rangeproofs of every
/// input-witness entry. But no candidate this arc builds bears an
/// issuance — the census refuses a declared one by type rather than
/// carrying a field for it — so the construction that recomputes the
/// message reads no issuance field at all. It writes one zero byte per
/// input for term 9 and two per input for term 10, both from the input
/// count alone.
///
/// A term computed without consulting the dimension's subject cannot
/// disagree with the target about the subject, so no number of agreeing
/// digests checks it. The accepted spend exercised the input count in
/// those two positions and nothing about any issuance field, and the six
/// negative controls moved neither term.
///
/// The consequence is the one the source review put on this arc's desk
/// by name (`rule:sighash-review:census-consequence`): the census's
/// silence about the input side is a claim with a precondition, and the
/// precondition is exactly the shape that would exercise this dimension.
/// The repair is a candidate that bears an issuance together with the
/// census field its input-witness proofs need — a construction, not a
/// rerun — or an owner ruling that re-types the dimension. Neither is
/// this review's to perform, and stating that is the whole of what a
/// review verdict owes here.
#[must_use]
pub(crate) fn reviewed_sighash_capability() -> SighashCapability {
    SighashCapability::new(
        established_sighash_dimensions(),
        unestablished_sighash_dimensions(),
        [TargetEvidenceRequirementId::SighashSemantics],
    )
}

/// The review's anchor for the dimension table.
const DIMENSIONS: &str = "tab:sighash-review:dimensions";

/// The review's anchor for the refusal table.
const REFUSALS: &str = "tab:sighash-review:refusals";

/// The six dimensions the observed acceptance exercised.
const fn established_sighash_dimensions() -> [(SighashDimension, ReviewedGround); 6] {
    use SighashDimension as Dimension;

    [
        (
            Dimension::AllOutputs,
            ReviewedGround::new(
                SighashSourceCitation::new(
                    "terms 11 and 12",
                    "src/script/interpreter.cpp:2741-2743",
                    DIMENSIONS,
                ),
                // Both halves of the output side were moved and refused:
                // the witness half by the recorded hazard reproduced
                // deliberately, and the list half by a candidate whose
                // two destinations are exchanged.
                ExercisingObservation::new(
                    OWNER_OBSERVATION,
                    &[
                        "control-empty-output-witness-vector",
                        "control-another-candidate",
                    ],
                ),
            ),
        ),
        (
            Dimension::AllInputs,
            ReviewedGround::new(
                SighashSourceCitation::new(
                    "terms 4, 5 and 8",
                    "src/script/interpreter.cpp:2727-2729 and :2737",
                    DIMENSIONS,
                ),
                // The three whole-transaction input hashes were formed
                // over both consumed receipts, from the outpoints and
                // sequences the candidate actually carried.
                ExercisingObservation::new(OWNER_OBSERVATION, &[]),
            ),
        ),
        (
            Dimension::Version,
            ReviewedGround::new(
                SighashSourceCitation::new("term 2", "src/script/interpreter.cpp:2725", DIMENSIONS),
                ExercisingObservation::new(OWNER_OBSERVATION, &[]),
            ),
        ),
        (
            Dimension::LockTime,
            ReviewedGround::new(
                SighashSourceCitation::new("term 3", "src/script/interpreter.cpp:2726", DIMENSIONS),
                ExercisingObservation::new(OWNER_OBSERVATION, &[]),
            ),
        ),
        (
            Dimension::TapleafHash,
            ReviewedGround::new(
                SighashSourceCitation::new(
                    "term 16",
                    "src/script/interpreter.cpp:2793-2795",
                    DIMENSIONS,
                ),
                // The leaf hash written is the one the control-block
                // check already computed, and the accepted candidate's
                // two inputs execute two different leaves.
                ExercisingObservation::new(OWNER_OBSERVATION, &[]),
            ),
        ),
        (
            Dimension::SpentOutputs,
            ReviewedGround::new(
                SighashSourceCitation::new(
                    "terms 6 and 7",
                    "src/script/interpreter.cpp:2730 and :2736",
                    DIMENSIONS,
                ),
                // The asset, value and program hashed here are the
                // node's own report of the funded coins rather than the
                // ceremony's expectation of them, so the agreement is
                // over what the target holds and not over what a builder
                // assumed.
                ExercisingObservation::new(OWNER_OBSERVATION, &[]),
            ),
        ),
    ]
}

/// The four dimensions the review reached and could not establish.
const fn unestablished_sighash_dimensions() -> [(SighashDimension, UnreviewedGround); 4] {
    use SighashDimension as Dimension;

    [
        (
            Dimension::Issuance,
            UnreviewedGround::NoCandidateThisArcBuildsCarriesTheSubject(
                SighashSourceCitation::new(
                    "terms 9 and 10",
                    "src/script/interpreter.cpp:2738-2739",
                    DIMENSIONS,
                ),
            ),
        ),
        (
            Dimension::SingleOutput,
            UnreviewedGround::TheSelectedProfileRefusesIt(SighashSourceCitation::new(
                "terms 11 and 12, replaced by the single-output branch",
                "src/script/interpreter.cpp:2741 and :2774-2790",
                REFUSALS,
            )),
        ),
        (
            Dimension::InputExtensionPermitted,
            UnreviewedGround::TheSelectedProfileRefusesIt(SighashSourceCitation::new(
                "terms 4 to 10, replaced by the signing input's own fields",
                "src/script/interpreter.cpp:2727 and :2750-2766",
                REFUSALS,
            )),
        ),
        (
            Dimension::InternalKey,
            UnreviewedGround::NoMessageTermCarriesIt {
                review_anchor: "rule:sighash-review:internal-key",
            },
        ),
    ]
}

/// Builds the reviewed authorization contract.
pub(crate) fn reviewed_authorization() -> AuthorizationContract {
    use TargetEvidenceRequirementId as R;

    AuthorizationContract::new(
        SignaturePrimitiveContract::new(
            EncodingClass::XOnlyPublicKey,
            EncodingClass::SchnorrSignature,
            // An empty signature consumes the operands and pushes a
            // false, which a program can branch on.
            FailureOutcome::ConsumeOperandsPushFalse,
            // A non-empty signature that does not verify aborts. There
            // is no branchable "verification failed".
            FailureOutcome::AbortEvaluation,
            UnknownPublicKeyTypeRule::SucceedsWithoutVerification,
            crate::opcode::VALIDATION_BUDGET_PER_CHECK,
            [R::SignatureSemantics],
        ),
        reviewed_sighash_capability(),
        RelativeTimelockContract::new(
            [TimelockMode::BlockHeight, TimelockMode::TimeInterval],
            SequenceFieldLayout {
                disable_flag: 0x8000_0000,
                mode_flag: 0x0040_0000,
                value_mask: 0x0000_ffff,
                time_interval_shift: 9,
            },
            2,
            FailureOutcome::AbortEvaluation,
            [R::RelativeTimelockSemantics],
        ),
    )
}
