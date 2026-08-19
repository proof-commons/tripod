//! What the operator meant to run, and whether the run was that.
//!
//! # Why an expectation is first-party
//!
//! A report's provenance fields are what the executor *said*. Read alone
//! they establish nothing: an executor that reports no revision, or
//! reports one nobody compared against anything, has described a program
//! without identifying it. ADR-018 assigns the comparison to the
//! evidence gate, and a comparison needs a second operand — the tip the
//! operator intended to execute, the upstream base it derives from, and
//! the census of local topics folded into it. That operand is
//! configuration this repository supplies, never a value the executor
//! offers.
//!
//! # What this does not claim
//!
//! A dishonest executor can report whatever it likes, and no comparison
//! here detects that. This closes omission and honest mismatch — a
//! missing field, a blank string, a binary built from a different tip
//! than the one the operator meant — which is exactly the scope ADR-018
//! gives the gate. Authenticity is not on offer and is not claimed.
//!
//! # No checkout fallback
//!
//! Nothing here derives a revision from a working tree. A checkout's
//! `HEAD` identifies intended source; the field being compared is what
//! the *binary* embeds about itself, and substituting one for the other
//! would attribute one program's identity to another.

use std::collections::BTreeSet;
use std::fmt;

use crate::error::NativeConformanceError;
use crate::report::ExecutorProvenance;

/// The narrowest revision prefix a binary may report.
///
/// Seven lowercase hex digits, which is the shortest abbreviation the
/// upstream tooling emits and the width the reviewed adapter's own
/// version-line pattern admits.
pub const MINIMUM_REVISION_PREFIX_WIDTH: usize = 7;

/// The width of a full object identifier.
pub const FULL_REVISION_WIDTH: usize = 40;

/// The longest topic branch name admitted.
pub const MAXIMUM_TOPIC_NAME_BYTES: usize = 100;

/// Why one offered string is not an admitted revision or topic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ProvenanceSyntaxDefect {
    /// Shorter than [`MINIMUM_REVISION_PREFIX_WIDTH`].
    RevisionTooShort,
    /// Longer than [`FULL_REVISION_WIDTH`].
    RevisionTooLong,
    /// Not made of lowercase hexadecimal digits.
    RevisionNotLowercaseHex,
    /// An expectation was stated as a prefix rather than a full
    /// identifier.
    RevisionNotFullWidth,
    /// A topic name that is empty or only spacing.
    TopicBlank,
    /// A topic name longer than [`MAXIMUM_TOPIC_NAME_BYTES`].
    TopicTooLong,
    /// A topic name holding a byte outside the admitted set.
    TopicCharacterNotAdmitted,
}

impl fmt::Display for ProvenanceSyntaxDefect {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::RevisionTooShort => "the revision is shorter than the admitted minimum width",
            Self::RevisionTooLong => "the revision is longer than a full object identifier",
            Self::RevisionNotLowercaseHex => {
                "the revision is not written in lowercase hexadecimal digits"
            }
            Self::RevisionNotFullWidth => {
                "an expected revision must be a full object identifier, not a prefix"
            }
            Self::TopicBlank => "the topic name is blank",
            Self::TopicTooLong => "the topic name is longer than the admitted maximum",
            Self::TopicCharacterNotAdmitted => {
                "the topic name holds a byte outside the admitted set"
            }
        };
        formatter.write_str(text)
    }
}

/// A revision identifier in the admitted syntax.
///
/// Lowercase hexadecimal, at least [`MINIMUM_REVISION_PREFIX_WIDTH`]
/// digits and at most [`FULL_REVISION_WIDTH`]. The type admits a prefix
/// because a binary reports one; whether a *prefix* may stand where a
/// full identifier is expected is the matching rule's question, not the
/// syntax's.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RevisionId(String);

impl RevisionId {
    /// One revision identifier, or why the text is not one.
    ///
    /// # Errors
    ///
    /// [`ProvenanceSyntaxDefect::RevisionTooShort`],
    /// [`ProvenanceSyntaxDefect::RevisionTooLong`], or
    /// [`ProvenanceSyntaxDefect::RevisionNotLowercaseHex`].
    pub fn new(text: &str) -> Result<Self, ProvenanceSyntaxDefect> {
        if text.len() < MINIMUM_REVISION_PREFIX_WIDTH {
            return Err(ProvenanceSyntaxDefect::RevisionTooShort);
        }
        if text.len() > FULL_REVISION_WIDTH {
            return Err(ProvenanceSyntaxDefect::RevisionTooLong);
        }
        if !text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(ProvenanceSyntaxDefect::RevisionNotLowercaseHex);
        }
        Ok(Self(text.to_owned()))
    }

    /// The identifier's digits.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether this identifier names the same object as `full` under the
    /// repository's one matching rule.
    ///
    /// # The rule, stated once
    ///
    /// A reported revision matches an expected full identifier when it
    /// is at least [`MINIMUM_REVISION_PREFIX_WIDTH`] digits long and is
    /// an exact prefix of it. Equality is the case where the reported
    /// width is [`FULL_REVISION_WIDTH`].
    ///
    /// # Why a prefix at all
    ///
    /// Because that is what the reviewed executable reports. The node
    /// binary embeds an abbreviated revision in its version line, and
    /// the reviewed adapter reads exactly that — a seven-to-forty digit
    /// abbreviation — rather than a full object identifier, which the
    /// binary does not carry. A rule demanding full equality would
    /// refuse every honest run of the real target; a rule accepting
    /// arbitrary text equality would accept a run that named a different
    /// object. The prefix rule is the one that fits what the binary
    /// actually says.
    ///
    /// # What a prefix costs
    ///
    /// A short prefix names an object less precisely than a full
    /// identifier does, and a repository large enough will eventually
    /// hold two objects sharing seven digits. The minimum width is
    /// therefore a floor rather than a target: an executor reporting
    /// more digits is compared against more of them.
    #[must_use]
    pub fn matches_full(&self, full: &FullRevisionId) -> bool {
        self.0.len() >= MINIMUM_REVISION_PREFIX_WIDTH && full.as_str().starts_with(&self.0)
    }
}

impl fmt::Display for RevisionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A revision identifier that is a full object identifier.
///
/// # Why the width is a type rather than a check
///
/// [`RevisionId`] admits a prefix because a binary reports one. An
/// *expectation* is the other operand of the comparison, and an
/// expectation stated as a prefix compares a prefix against a prefix —
/// a weaker statement than the one the comparison exists to make, and
/// one no caller can see they have made. Carrying the width in the type
/// means a value that reaches a matching rule has already been refused
/// if it is an abbreviation, so no caller and no later edit can restate
/// the check incorrectly or skip it.
///
/// [`FullRevisionId::new`] is the only way to build one, so the
/// invariant holds for every value of this type that exists.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FullRevisionId(RevisionId);

impl FullRevisionId {
    /// One full object identifier, refusing a prefix.
    ///
    /// # Errors
    ///
    /// Everything [`RevisionId::new`] states, and
    /// [`ProvenanceSyntaxDefect::RevisionNotFullWidth`] for an
    /// admitted prefix offered where a full identifier is required.
    pub fn new(text: &str) -> Result<Self, ProvenanceSyntaxDefect> {
        let revision = RevisionId::new(text)?;
        if revision.as_str().len() != FULL_REVISION_WIDTH {
            return Err(ProvenanceSyntaxDefect::RevisionNotFullWidth);
        }
        Ok(Self(revision))
    }

    /// The identifier's digits.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// This identifier read as an admitted revision.
    #[must_use]
    pub const fn as_revision(&self) -> &RevisionId {
        &self.0
    }

    /// Whether the width invariant still holds.
    ///
    /// The constructor establishes it and no public API can break it.
    /// Code inside this module can reach the private member, though, so
    /// the gate re-asserts this rather than trusting that no future edit
    /// here ever builds one another way.
    fn holds_full_width(&self) -> bool {
        self.0.as_str().len() == FULL_REVISION_WIDTH
    }
}

impl fmt::Display for FullRevisionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// One local topic branch name.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TopicName(String);

impl TopicName {
    /// One topic name, or why the text is not one.
    ///
    /// # Errors
    ///
    /// [`ProvenanceSyntaxDefect::TopicBlank`],
    /// [`ProvenanceSyntaxDefect::TopicTooLong`], or
    /// [`ProvenanceSyntaxDefect::TopicCharacterNotAdmitted`].
    pub fn new(text: &str) -> Result<Self, ProvenanceSyntaxDefect> {
        if text.trim().is_empty() {
            return Err(ProvenanceSyntaxDefect::TopicBlank);
        }
        if text.len() > MAXIMUM_TOPIC_NAME_BYTES {
            return Err(ProvenanceSyntaxDefect::TopicTooLong);
        }
        if !text.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'/' | b'+')
        }) {
            return Err(ProvenanceSyntaxDefect::TopicCharacterNotAdmitted);
        }
        Ok(Self(text.to_owned()))
    }

    /// The name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TopicName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// What the operator declares the run was meant to execute (ADR-018).
///
/// # Why the members are private
///
/// The two revisions are full object identifiers, and that is the whole
/// content of the type: an expectation is the operand a reported
/// revision is compared *against*, so an abbreviated one silently
/// weakens every comparison the gate makes. Public members would let a
/// caller assemble the same type by literal without passing through
/// [`ExpectedExecutorProvenance::new`], which is how the invariant was
/// bypassed before. With private members and [`FullRevisionId`] fields,
/// the constructor is the only way in and the width cannot be weakened
/// from outside this module at all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpectedExecutorProvenance {
    intended_tip: FullRevisionId,
    upstream_base: FullRevisionId,
    included_local_topics: BTreeSet<TopicName>,
}

impl ExpectedExecutorProvenance {
    /// One expectation, from full identifiers and topic names.
    ///
    /// # Errors
    ///
    /// Every [`ProvenanceSyntaxDefect`] the members state. The two
    /// revisions must be full object identifiers: an expectation stated
    /// as an abbreviation would compare a prefix against a prefix, and
    /// two abbreviations agreeing is a weaker statement than the one
    /// this type exists to make.
    pub fn new<'a>(
        intended_tip: &str,
        upstream_base: &str,
        included_local_topics: impl IntoIterator<Item = &'a str>,
    ) -> Result<Self, ProvenanceSyntaxDefect> {
        Ok(Self {
            intended_tip: FullRevisionId::new(intended_tip)?,
            upstream_base: FullRevisionId::new(upstream_base)?,
            included_local_topics: included_local_topics
                .into_iter()
                .map(TopicName::new)
                .collect::<Result<BTreeSet<TopicName>, ProvenanceSyntaxDefect>>()?,
        })
    }

    /// The integration tip the run was meant to execute, in full.
    #[must_use]
    pub const fn intended_tip(&self) -> &FullRevisionId {
        &self.intended_tip
    }

    /// The upstream base that tip derives from, in full.
    #[must_use]
    pub const fn upstream_base(&self) -> &FullRevisionId {
        &self.upstream_base
    }

    /// The local topic branches folded into that tip.
    #[must_use]
    pub const fn included_local_topics(&self) -> &BTreeSet<TopicName> {
        &self.included_local_topics
    }
}

/// Why one run's reported provenance is not the expected one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ProvenanceDefect {
    /// The adapter did not name itself.
    BlankAdapterName,
    /// The adapter stated no version.
    BlankAdapterVersion,
    /// The node did not name itself.
    BlankNodeName,
    /// The node stated no version.
    BlankNodeVersion,
    /// The binary reported no revision of its own.
    MissingBinaryRevision,
    /// The run named no intended tip.
    MissingIntendedTip,
    /// The run named no upstream base.
    MissingUpstreamBase,
    /// The binary's reported revision is not in the admitted syntax.
    MalformedBinaryRevision(ProvenanceSyntaxDefect),
    /// The reported intended tip is not in the admitted syntax.
    MalformedIntendedTip(ProvenanceSyntaxDefect),
    /// The reported upstream base is not in the admitted syntax.
    MalformedUpstreamBase(ProvenanceSyntaxDefect),
    /// A reported topic name is not in the admitted syntax.
    MalformedTopicName(ProvenanceSyntaxDefect),
    /// The binary's revision does not match the intended tip.
    BinaryRevisionIsNotTheIntendedTip,
    /// The run's intended tip is not the one that was expected.
    IntendedTipIsNotTheExpectedTip,
    /// The run's upstream base is not the one that was expected.
    UpstreamBaseIsNotTheExpectedBase,
    /// The local topic census is not the expected one.
    TopicCensusIsNotTheExpectedCensus,
    /// An expected revision reached the gate as an abbreviation.
    ExpectedRevisionIsNotFullWidth,
}

impl fmt::Display for ProvenanceDefect {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankAdapterName => formatter.write_str("the adapter did not name itself"),
            Self::BlankAdapterVersion => formatter.write_str("the adapter stated no version"),
            Self::BlankNodeName => formatter.write_str("the node did not name itself"),
            Self::BlankNodeVersion => formatter.write_str("the node stated no version"),
            Self::MissingBinaryRevision => {
                formatter.write_str("the binary reported no revision of its own")
            }
            Self::MissingIntendedTip => formatter.write_str("the run named no intended tip"),
            Self::MissingUpstreamBase => formatter.write_str("the run named no upstream base"),
            Self::MalformedBinaryRevision(defect) => {
                write!(formatter, "the binary's reported revision: {defect}")
            }
            Self::MalformedIntendedTip(defect) => {
                write!(formatter, "the reported intended tip: {defect}")
            }
            Self::MalformedUpstreamBase(defect) => {
                write!(formatter, "the reported upstream base: {defect}")
            }
            Self::MalformedTopicName(defect) => {
                write!(formatter, "a reported local topic name: {defect}")
            }
            Self::BinaryRevisionIsNotTheIntendedTip => formatter.write_str(
                "the revision the binary reports is not an admitted prefix of the intended tip",
            ),
            Self::IntendedTipIsNotTheExpectedTip => {
                formatter.write_str("the run's intended tip is not the expected one")
            }
            Self::UpstreamBaseIsNotTheExpectedBase => {
                formatter.write_str("the run's upstream base is not the expected one")
            }
            Self::TopicCensusIsNotTheExpectedCensus => {
                formatter.write_str("the run's local topic census is not the expected one")
            }
            Self::ExpectedRevisionIsNotFullWidth => formatter.write_str(
                "an expected revision reached the gate as an abbreviation rather than a full \
                 object identifier",
            ),
        }
    }
}

/// Provenance that has been compared against an explicit expectation.
///
/// The wrapper has no constructor other than
/// [`validate_executor_provenance`]. A gate takes one as evidence that
/// the comparison happened, in the same way it takes a validated report
/// as evidence that the recomputation happened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedExecutorProvenance {
    intended_tip: FullRevisionId,
    upstream_base: FullRevisionId,
    binary_reported_revision: RevisionId,
    included_local_topics: BTreeSet<TopicName>,
}

impl ValidatedExecutorProvenance {
    /// The tip the run executed, as the operator declared it.
    ///
    /// Full width, because the comparison that built this value proved
    /// the reported tip equal to a full expected identifier.
    #[must_use]
    pub const fn intended_tip(&self) -> &FullRevisionId {
        &self.intended_tip
    }

    /// The upstream base that tip derives from, in full.
    #[must_use]
    pub const fn upstream_base(&self) -> &FullRevisionId {
        &self.upstream_base
    }

    /// The revision the binary reported about itself.
    ///
    /// This one may be an abbreviation: it is what the binary embeds,
    /// and the matching rule is what relates it to the intended tip.
    #[must_use]
    pub const fn binary_reported_revision(&self) -> &RevisionId {
        &self.binary_reported_revision
    }

    /// The local topics folded into the executed tip.
    #[must_use]
    pub const fn included_local_topics(&self) -> &BTreeSet<TopicName> {
        &self.included_local_topics
    }
}

/// Compares one run's reported provenance against the expectation.
///
/// Every check ADR-018 assigns to the gate, in one place: the five
/// provenance roles are nonblank where they must be, the revision fields
/// are in the admitted syntax, the binary's own revision matches the
/// intended tip under the one rule [`RevisionId::matches_full`] states,
/// the intended tip and upstream base are the expected ones, and the
/// local topic census is equal — not a superset, not a subset, since a
/// tip carrying an unexpected topic is not the tip that was reviewed.
///
/// # Errors
///
/// [`NativeConformanceError::ExecutorProvenanceUnestablished`] carrying
/// the first defect found, in the order the checks are written.
pub fn validate_executor_provenance(
    reported: &ExecutorProvenance,
    expected: &ExpectedExecutorProvenance,
) -> Result<ValidatedExecutorProvenance, NativeConformanceError> {
    let refuse = NativeConformanceError::ExecutorProvenanceUnestablished;

    // The expectation before the report. [`FullRevisionId`] establishes
    // this at construction and no caller outside this module can build
    // one another way, so reaching this branch means an edit *here*
    // broke the invariant. It is checked anyway because the cost of the
    // check is nothing and the cost of missing it is that every
    // comparison below silently compares abbreviations.
    if !expected.intended_tip.holds_full_width() || !expected.upstream_base.holds_full_width() {
        return Err(refuse(ProvenanceDefect::ExpectedRevisionIsNotFullWidth));
    }

    for (text, defect) in [
        (&reported.adapter_name, ProvenanceDefect::BlankAdapterName),
        (
            &reported.adapter_version,
            ProvenanceDefect::BlankAdapterVersion,
        ),
        (&reported.node_name, ProvenanceDefect::BlankNodeName),
        (&reported.node_version, ProvenanceDefect::BlankNodeVersion),
    ] {
        if text.trim().is_empty() {
            return Err(refuse(defect));
        }
    }

    let binary = revision_field(
        reported.binary_reported_revision.as_deref(),
        ProvenanceDefect::MissingBinaryRevision,
        ProvenanceDefect::MalformedBinaryRevision,
    )?;
    let intended_tip = revision_field(
        reported.intended_executed_tip.as_deref(),
        ProvenanceDefect::MissingIntendedTip,
        ProvenanceDefect::MalformedIntendedTip,
    )?;
    let upstream_base = revision_field(
        reported.upstream_base.as_deref(),
        ProvenanceDefect::MissingUpstreamBase,
        ProvenanceDefect::MalformedUpstreamBase,
    )?;

    let mut topics = BTreeSet::new();
    for topic in &reported.included_local_topics {
        topics.insert(
            TopicName::new(topic)
                .map_err(|defect| refuse(ProvenanceDefect::MalformedTopicName(defect)))?,
        );
    }

    // The operator's own declaration first: a run that says it meant to
    // execute some other tip is not this run, whatever its binary says.
    // Exact equality against the full expected identifier, not a prefix
    // rule: what the operator declared is compared digit for digit.
    if intended_tip != *expected.intended_tip.as_revision() {
        return Err(refuse(ProvenanceDefect::IntendedTipIsNotTheExpectedTip));
    }
    if upstream_base != *expected.upstream_base.as_revision() {
        return Err(refuse(ProvenanceDefect::UpstreamBaseIsNotTheExpectedBase));
    }
    if topics != expected.included_local_topics {
        return Err(refuse(ProvenanceDefect::TopicCensusIsNotTheExpectedCensus));
    }
    // Then the binary's own statement about itself, against the tip the
    // expectation names rather than against the reported one: the two
    // are equal by the check above, and comparing against the
    // expectation keeps the executor from supplying both operands.
    if !binary.matches_full(&expected.intended_tip) {
        return Err(refuse(ProvenanceDefect::BinaryRevisionIsNotTheIntendedTip));
    }

    // The two declared revisions are equal to the expected full
    // identifiers by the checks above, so the validated value carries
    // the expectation's full-width form rather than the reported text.
    debug_assert_eq!(intended_tip, *expected.intended_tip.as_revision());
    debug_assert_eq!(upstream_base, *expected.upstream_base.as_revision());
    Ok(ValidatedExecutorProvenance {
        intended_tip: expected.intended_tip.clone(),
        upstream_base: expected.upstream_base.clone(),
        binary_reported_revision: binary,
        included_local_topics: topics,
    })
}

/// Why one command's provenance arguments do not state an expectation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ProvenanceArgumentDefect {
    /// A reviewed nonmock run named no intended tip.
    MissingIntendedTip,
    /// A reviewed nonmock run named no upstream base.
    MissingUpstreamBase,
    /// An argument is not in the admitted syntax.
    Malformed(ProvenanceSyntaxDefect),
    /// A mock run stated a provenance expectation.
    ProvenanceStatedForMockRun,
}

impl fmt::Display for ProvenanceArgumentDefect {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingIntendedTip => formatter.write_str(
                "a reviewed nonmock run must state the intended executed tip it was built from",
            ),
            Self::MissingUpstreamBase => formatter.write_str(
                "a reviewed nonmock run must state the upstream base that tip derives from",
            ),
            Self::Malformed(defect) => write!(formatter, "{defect}"),
            Self::ProvenanceStatedForMockRun => formatter.write_str(
                "a mock run states no provenance expectation, because no expectation makes it \
                 evidence",
            ),
        }
    }
}

/// The expectation one command's arguments state, if any.
///
/// # Why the class decides
///
/// A reviewed nonmock run is a run that intends to be evidence, so
/// ADR-018's binding values are not optional for it: a caller who
/// selects a reviewed executable without saying what it was built from
/// has selected a program nobody can identify, and the gate would refuse
/// the report anyway. Failing here instead makes that a configuration
/// error the caller sees before a node boots.
///
/// A mock run is the other way round. It can never be evidence whatever
/// its provenance says, so stating an expectation for one is a caller
/// believing they configured something they did not.
///
/// # Errors
///
/// [`ProvenanceArgumentDefect`], which the caller reports as a
/// configuration failure.
pub fn expected_provenance_from_arguments(
    reviewed_non_mock: bool,
    intended_tip: Option<&str>,
    upstream_base: Option<&str>,
    local_topics: &[String],
) -> Result<Option<ExpectedExecutorProvenance>, ProvenanceArgumentDefect> {
    if !reviewed_non_mock {
        if intended_tip.is_some() || upstream_base.is_some() || !local_topics.is_empty() {
            return Err(ProvenanceArgumentDefect::ProvenanceStatedForMockRun);
        }
        return Ok(None);
    }

    let tip = intended_tip.ok_or(ProvenanceArgumentDefect::MissingIntendedTip)?;
    let base = upstream_base.ok_or(ProvenanceArgumentDefect::MissingUpstreamBase)?;
    // An empty topic census is a statement, not an omission: it says the
    // tip folded in no local branch. ADR-018 asks for the census, and a
    // census of nothing is one.
    ExpectedExecutorProvenance::new(tip, base, local_topics.iter().map(String::as_str))
        .map(Some)
        .map_err(ProvenanceArgumentDefect::Malformed)
}

/// One optional revision field, present and in the admitted syntax.
fn revision_field(
    offered: Option<&str>,
    missing: ProvenanceDefect,
    malformed: fn(ProvenanceSyntaxDefect) -> ProvenanceDefect,
) -> Result<RevisionId, NativeConformanceError> {
    let text = offered.ok_or(NativeConformanceError::ExecutorProvenanceUnestablished(
        missing,
    ))?;
    // A present-but-blank field is an absent field that answered. The
    // syntax refuses it either way; naming it as malformed rather than
    // missing keeps the distinction the executor actually made.
    RevisionId::new(text).map_err(|defect| {
        NativeConformanceError::ExecutorProvenanceUnestablished(malformed(defect))
    })
}
