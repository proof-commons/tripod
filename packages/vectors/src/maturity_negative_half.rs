//! The maturity announcement's negative half: every §16 row a target run
//! has not answered, each carrying the NAMED GAP that keeps it waiting.
//!
//! # Why a register and not a number
//!
//! [`crate::maturity_evidence`] classifies every row of the §16 matrix
//! and counts how many stand at
//! [`MaturityRowStanding::NativeRunRequired`]. The count is enough to
//! know the half is unfinished and not enough to work on: it says HOW
//! MANY rows are waiting and never WHICH, so a wave reading it has to
//! re-derive the classifier's routing to recover the list, and two waves
//! can agree about the number while disagreeing about the rows. The
//! count stays on the census and is deliberately not repeated here — a
//! second place for one figure is a second place for it to drift.
//!
//! This register is that list, held against the classifier by
//! `every_outstanding_row_is_registered`. The test is set equality in
//! BOTH directions, which is what makes the register load-bearing rather
//! than documentation: a row that reaches an answered standing and is
//! left here fails it, and a row that stands waiting without being
//! listed here fails it too. Neither can happen quietly.
//!
//! # Why every entry carries a gap
//!
//! [`MaturityRowStanding::NativeRunRequired`] says a run would answer the
//! row. §14.2's rule is that an answerable capability is not an answered
//! standing, and the sharper reading this register is built on is that
//! "answerable" is itself a claim: for most of the rows below a run alone
//! would answer nothing, because nothing stages the row's change, or the
//! shape it needs is not the shape this ceremony builds, or the verdict
//! it would draw belongs to a rule that never reads the row. A standing
//! that says a run is owed, for a row no run can reach, sends every wave
//! that reads it to charter the run.
//!
//! So each entry names its own [`MaturityNegativeHalfGap`]. The
//! vocabulary has no member meaning "not yet looked at": an entry without
//! a gap does not compile, so the register's coverage is total rather
//! than aspirational. What the gaps buy is triage a reader can act on —
//! four of the six members are work somebody can do, and two are not
//! moved by any run at all.
//!
//! # What keeps a STATE row waiting, and what does not
//!
//! Two facts are true of all forty-three rows and therefore name none of
//! them. No module of this crate plans a target operation for the
//! announcement or submits bytes to one, and the two mutation registries
//! the evidence plan carries stand explicitly outstanding with a census
//! of zero, so neither "no run" nor "no stager" separates one row from
//! another. Each row's gap is instead the obstacle that OUTLIVES those
//! two: what would still keep the row waiting on the day a planner and a
//! run exist. That is what makes the vocabulary a plan rather than a
//! restatement of the wave's position, and it is why the largest member
//! is the one a single stager discharges while the smallest are walls.
//!
//! # A row's identity is its table and its name
//!
//! Three names repeat across §16's tables and two of the three repeat
//! INSIDE the waiting set: `wrong-leaf-version` and `wrong-internal-key`
//! each stand in the predecessor-constructor table and again in the
//! successor-constructor table, with different mutation layers, different
//! locators and different grounds. A register keyed by name alone would
//! hold one of each pair and drop the other, and set equality against the
//! classifier would still pass. So an entry's identity is the section and
//! the name together, and the comparison is made on that pair.
//!
//! # One published requirement, many rows
//!
//! Thirty-three of the forty-three rows share their declared relation and
//! published class with at least one other waiting row, and the
//! resolution selects on relation, case, boundary and class — so a
//! cluster of rows resolves to ONE requirement identity, and eight rows
//! of the operator table resolve together. This is a fact about the
//! matrix and the plan rather than about any row, which is why it is
//! recorded here and is not a member of the vocabulary: turning it into a
//! gap would give thirty-three rows one word and say nothing about what
//! each of them needs. It does bound what a first run can conclude,
//! because an observation filed against a shared requirement cannot say
//! which of its rows it answered.
//!
//! # What this register is not
//!
//! It is not evidence, and it moves no row. A row leaves this list by
//! being ANSWERED at its own site — an observed refusal recorded against
//! an accepted control at exactly the boundary the row declared — and
//! never by being deleted from here. The register follows the classifier;
//! the classifier never follows the register.
//!
//! Nor is a gap a verdict on the row. A gap names what stands between the
//! row and an answer at this tip, and several of the entries below record
//! that the answer would need a ruling or a published relation rather
//! than a run. Naming that is not settling it.

use std::collections::BTreeMap;

use crate::maturity_evidence::{
    MaturityEvidenceRefusal, MaturityExecutorProvenanceExpectation, MaturityRowStanding,
    derive_maturity_evidence_plan_with,
};
use crate::maturity_safety::MaturitySafetySection;

/// One waiting row's identity: the table it is drawn from and its name.
///
/// Spelled once because a name alone is not an identity in this matrix,
/// and a register that keyed on one would lose a row silently.
type RowIdentity = (MaturitySafetySection, &'static str);

/// Why one §16 row is still waiting on a target run.
///
/// The first four members are work: each names something a wave can
/// build, and building it makes the row answerable. The last two are not
/// moved by any run — one because the refusal a run would draw belongs to
/// a rule that never reads the row, the other because the row names no
/// published requirement an observation could be filed against. The two
/// walls are not ranked against one another, because they obstruct
/// different things: one the verdict, the other the filing.
///
/// There is deliberately no member meaning `unassessed`. A row whose gap
/// nobody has established gets no entry, and a row without an entry fails
/// `every_outstanding_row_is_registered`, so the absence of that member
/// is what keeps the register's coverage total.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum MaturityNegativeHalfGap {
    /// The row's candidate is the honest announcement, and only the run
    /// is missing.
    ///
    /// The lightest gap in the vocabulary and the only one that needs no
    /// mutation at all: the linked bundle already carries the bytes this
    /// row asks a target about, and what is absent is a planner that
    /// states the operation and a run that submits it. Work, and the
    /// smallest of it.
    TargetRunNotYetPlanned,
    /// The change is a substitution at a position the linked candidate
    /// already carries.
    ///
    /// The row's locator names a field, a witness item or an output the
    /// candidate has, and staging the row means writing another value
    /// there rather than rebuilding anything. Work: one stager over the
    /// linked candidate discharges every row here, which is why this is
    /// the largest member and the wave's cheapest remaining step.
    MutationStagingOwedOnTheLinkedCandidate,
    /// No shape this ceremony builds has the position the row mutates.
    ///
    /// Not a missing stager but a missing SUBJECT. The announcement is
    /// built at one fixed shape — sponsorless, script-path, one state
    /// successor — and these rows name a sponsor input, a second or
    /// absent state output, or a key-path witness, none of which that
    /// shape has anywhere to put. Work, and larger than a stager: the row
    /// needs a transaction built differently rather than a value written
    /// differently.
    NoAdmittedShapeCarriesTheFault,
    /// The fault is in a value the linker derives, and no entry point
    /// emits a wrong one.
    ///
    /// The row mutates an internal key, a parity, a leaf version, a
    /// control recipe, a static subtree or a whole program — values the
    /// emission computes from typed inputs rather than accepts from a
    /// caller. Staging the row therefore needs a second linker that emits
    /// the wrong value on purpose, which is work, and work no surgery on
    /// the emitted bytes substitutes for: a program written over is a
    /// program that no longer matches its own commitments, which is a
    /// different row.
    ProgramDerivationHasNoKnobForTheFault,
    /// The refusal is program-generic and can never name the row's class.
    ///
    /// A wall. A leaf executes only from a taptree the spent program
    /// commits to, so a candidate revealing a control block that does not
    /// commit is refused by the commitment rule before a single opcode
    /// runs, and every foreign block draws that identical verdict. An
    /// observation of it establishes the commitment rule and says nothing
    /// about the row, so a run does not move these rows however well it
    /// is planned.
    RefusalIsProgramGeneric,
    /// The row names no published relation, so a refusal has nothing to
    /// be filed against.
    ///
    /// The other wall, and a wall of the specification rather than of the
    /// target. The classifier carries the requirement a row resolves to
    /// precisely so that a run's observation lands against a relation;
    /// these rows resolve to none, so a run could refuse the candidate
    /// and the record would still have no obligation to mark answered.
    /// What moves them is a published relation or a ruling that they need
    /// none, which is not work a run performs.
    NoPublishedRelationFilesTheObservation,
}

/// Whether a run's observation of this row would have a published
/// requirement to be filed against.
///
/// Two members and no more, because the classifier's own standing carries
/// an optional requirement and this marker mirrors exactly that option.
/// The requirement's IDENTITY is deliberately not carried: the resolution
/// computes it from the relation, the case, the boundary and the class,
/// and a table of identities kept beside the matrix would be a third
/// authority that could agree with neither the guide nor the plan while
/// still agreeing with itself after the plan moved. The marker keeps what
/// a test can check and leaves the identity where it is computed.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityNegativeHalfLink {
    /// The row resolves to exactly one published requirement.
    FiledAgainstAPublishedRequirement,
    /// The row resolves to no requirement, so an observation of it has
    /// nowhere to land.
    NotFiledAgainstAnyRequirement,
}

/// One waiting row, its gap, and the ground the gap rests on.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MaturityNegativeHalfEntry {
    section: MaturitySafetySection,
    row: &'static str,
    gap: MaturityNegativeHalfGap,
    link: MaturityNegativeHalfLink,
    ground: &'static str,
}

impl MaturityNegativeHalfEntry {
    /// The §16 table this row is drawn from.
    #[must_use]
    pub const fn section(&self) -> MaturitySafetySection {
        self.section
    }

    /// The row's name within its table.
    #[must_use]
    pub const fn row(&self) -> &'static str {
        self.row
    }

    /// Why it is still waiting.
    #[must_use]
    pub const fn gap(&self) -> MaturityNegativeHalfGap {
        self.gap
    }

    /// Whether a refusal of this row could be filed against a published
    /// requirement.
    #[must_use]
    pub const fn link(&self) -> MaturityNegativeHalfLink {
        self.link
    }

    /// The specific fact the gap rests on for THIS row.
    ///
    /// One sentence, and about the row rather than about the gap: the
    /// gap's own meaning sits on its member, and repeating it here would
    /// make the register say one thing twice. The test below holds every
    /// ground distinct for that reason.
    #[must_use]
    pub const fn ground(&self) -> &'static str {
        self.ground
    }
}

/// One entry, spelled once.
const fn entry(
    section: MaturitySafetySection,
    row: &'static str,
    gap: MaturityNegativeHalfGap,
    link: MaturityNegativeHalfLink,
    ground: &'static str,
) -> MaturityNegativeHalfEntry {
    MaturityNegativeHalfEntry {
        section,
        row,
        gap,
        link,
        ground,
    }
}

use MaturityNegativeHalfGap as G;
use MaturityNegativeHalfLink as K;
use MaturitySafetySection as S;

/// Every §16 row standing at
/// [`MaturityRowStanding::NativeRunRequired`], in the matrix's own order.
///
/// Derived from the classifier's routing rather than kept by hand: a row
/// reaches this standing exactly when its declared boundary is one of the
/// five a target produces, so what is written here is the result of that
/// routing and the test below is what keeps the writing and the routing
/// one list.
///
/// The waiting set contains forty-two faults after the committed run answers the sponsorless positive row at its declared relay boundary. Acceptance remains outstanding; the other positive rows state typed non-answers and do not enter this target-run register.
pub const STILL_REQUIRED: &[MaturityNegativeHalfEntry] = &[
    // §16.2 — the two window faults, both a write at the successor's own
    // metadata field.
    entry(
        S::WindowFault,
        "successor-remains-unannounced",
        G::MutationStagingOwedOnTheLinkedCandidate,
        K::FiledAgainstAPublishedRequirement,
        "leaving the successor's maturity at its unannounced value is a write at a metadata field the linked candidate already carries, and no stage of this crate writes one",
    ),
    entry(
        S::WindowFault,
        "successor-becomes-complete",
        G::MutationStagingOwedOnTheLinkedCandidate,
        K::FiledAgainstAPublishedRequirement,
        "the completed value the row asks for fits the same metadata field without disturbing another byte, so the row waits on a stager rather than on a second ceremony",
    ),
    // §16.3 — the metadata fault whose subject travels as a witness item.
    entry(
        S::MetadataFault,
        "metadata-from-another-state-object",
        G::MutationStagingOwedOnTheLinkedCandidate,
        K::FiledAgainstAPublishedRequirement,
        "the predecessor's record travels as a witness item of the candidate, so substituting another state object's record is a write at that item and leaves the spent program as it was funded",
    ),
    // §16.4 — the operator faults. Eight are writes at the signing
    // response or at an output the candidate has; one names a sponsor
    // input this sponsorless shape has nowhere to put.
    entry(
        S::OperatorFault,
        "empty-signature",
        G::MutationStagingOwedOnTheLinkedCandidate,
        K::FiledAgainstAPublishedRequirement,
        "the operator's signing response is a witness item the candidate carries complete, so emptying it is a substitution at that item alone",
    ),
    entry(
        S::OperatorFault,
        "malformed-signature",
        G::MutationStagingOwedOnTheLinkedCandidate,
        K::FiledAgainstAPublishedRequirement,
        "a malformed value fits the same signing-response item without rebuilding the witness around it, so only the stage that writes it is missing",
    ),
    entry(
        S::OperatorFault,
        "wrong-operator",
        G::MutationStagingOwedOnTheLinkedCandidate,
        K::FiledAgainstAPublishedRequirement,
        "a response from an unapproved operator replaces the signing-response item while the approved key stays where the linked bundle put it",
    ),
    entry(
        S::OperatorFault,
        "stale-operator",
        G::MutationStagingOwedOnTheLinkedCandidate,
        K::FiledAgainstAPublishedRequirement,
        "the approved operator key is itself a position of the candidate's witness, so a superseded key is written there without re-deriving any program",
    ),
    entry(
        S::OperatorFault,
        "valid-signature-under-another-key",
        G::MutationStagingOwedOnTheLinkedCandidate,
        K::FiledAgainstAPublishedRequirement,
        "a signature valid under another key is a well-formed value for the signing-response item, so the row needs a signer over a second key rather than a second shape",
    ),
    entry(
        S::OperatorFault,
        "valid-signature-over-another-candidate",
        G::MutationStagingOwedOnTheLinkedCandidate,
        K::FiledAgainstAPublishedRequirement,
        "a signature over other bytes is likewise a value that item accepts, so producing it is a second signing over a candidate this crate can already build",
    ),
    entry(
        S::OperatorFault,
        "successor-metadata-changed-after-signing",
        G::MutationStagingOwedOnTheLinkedCandidate,
        K::FiledAgainstAPublishedRequirement,
        "the successor's metadata sits in an output the candidate already has, so changing it once the operator has signed is a write at that output with the signature left untouched",
    ),
    entry(
        S::OperatorFault,
        "successor-program-changed-after-signing",
        G::MutationStagingOwedOnTheLinkedCandidate,
        K::FiledAgainstAPublishedRequirement,
        "substituting another program into that same output after signing needs no re-derivation, because the row asks about the signature's reach and not about how the program was built",
    ),
    entry(
        S::OperatorFault,
        "sponsor-input-added-after-signing",
        G::NoAdmittedShapeCarriesTheFault,
        K::FiledAgainstAPublishedRequirement,
        "the shape this ceremony builds is sponsorless, so there is no sponsor envelope to add an input to and the row waits on the sponsored form nothing here constructs",
    ),
    entry(
        S::OperatorFault,
        "fee-output-changed-after-signing",
        G::MutationStagingOwedOnTheLinkedCandidate,
        K::FiledAgainstAPublishedRequirement,
        "the shape carries a fee output of its own, so the row's change is a substitution at a position the candidate already has rather than an added one",
    ),
    // §16.5 — the predecessor-constructor faults. Four are writes, three
    // ask the linker for a value it derives, one presents an uncommitted
    // control block and one asks for a witness with no leaf at all.
    entry(
        S::PredecessorConstructorFault,
        "wrong-state-asset",
        G::MutationStagingOwedOnTheLinkedCandidate,
        K::FiledAgainstAPublishedRequirement,
        "the spent state input names its asset in a transaction field of the candidate, and writing another asset there leaves the emitted program untouched",
    ),
    entry(
        S::PredecessorConstructorFault,
        "wrong-singleton-amount",
        G::MutationStagingOwedOnTheLinkedCandidate,
        K::FiledAgainstAPublishedRequirement,
        "the singleton's amount sits in that same input, so a value off the singleton's own is a field write rather than a re-derivation",
    ),
    entry(
        S::PredecessorConstructorFault,
        "wrong-predecessor-program",
        G::ProgramDerivationHasNoKnobForTheFault,
        K::FiledAgainstAPublishedRequirement,
        "the predecessor's program is what the emission derives from typed inputs, and no public entry point of the linker emits a bundle at a program a caller chose",
    ),
    entry(
        S::PredecessorConstructorFault,
        "predecessor-metadata-reconstructs-another-program",
        G::MutationStagingOwedOnTheLinkedCandidate,
        K::FiledAgainstAPublishedRequirement,
        "the record that reconstructs a program is a witness item, so it is written at that item while the spent output stays funded at the program it was funded at",
    ),
    entry(
        S::PredecessorConstructorFault,
        "wrong-leaf-version",
        G::ProgramDerivationHasNoKnobForTheFault,
        K::FiledAgainstAPublishedRequirement,
        "the leaf version the predecessor's program commits to is fixed by the emission, which exposes no way to build a bundle at another version",
    ),
    entry(
        S::PredecessorConstructorFault,
        "wrong-internal-key",
        G::ProgramDerivationHasNoKnobForTheFault,
        K::FiledAgainstAPublishedRequirement,
        "the internal key the predecessor's output key is tweaked from is derived in that emission too, and no caller-chosen key reaches it",
    ),
    entry(
        S::PredecessorConstructorFault,
        "wrong-control-block",
        G::RefusalIsProgramGeneric,
        K::FiledAgainstAPublishedRequirement,
        "a control block that does not commit to the spent output key is refused by the commitment rule before an opcode runs, and every such block draws that one verdict whatever the row was about",
    ),
    entry(
        S::PredecessorConstructorFault,
        "metadata-leaf-selected-for-execution",
        G::MutationStagingOwedOnTheLinkedCandidate,
        K::FiledAgainstAPublishedRequirement,
        "the taptree commits the metadata leaf beside the static root, so revealing it is a control block the bundle's own hashes determine and the leaf then runs on its own terms",
    ),
    entry(
        S::PredecessorConstructorFault,
        "key-path-spend-attempt",
        G::NoAdmittedShapeCarriesTheFault,
        K::FiledAgainstAPublishedRequirement,
        "the ceremony builds a script-path witness, and a key-path spend carries no leaf and no control block at all, so the row asks for a witness of another shape rather than a changed item of this one",
    ),
    // §16.6 — the successor-constructor faults. Three are metadata
    // writes, two name no published relation, six ask the emission for a
    // value it derives, and three need an output cardinality this shape
    // does not have.
    entry(
        S::SuccessorConstructorFault,
        "successor-from-wrong-semantic-metadata",
        G::MutationStagingOwedOnTheLinkedCandidate,
        K::FiledAgainstAPublishedRequirement,
        "the semantic record the successor is built from is a field of the candidate's own output, so building it from another state object's record is a write at that field",
    ),
    entry(
        S::SuccessorConstructorFault,
        "successor-from-predecessor-metadata-unchanged",
        G::MutationStagingOwedOnTheLinkedCandidate,
        K::FiledAgainstAPublishedRequirement,
        "copying the predecessor's record through leaves every other byte of the candidate where it was, which is a substitution the shape already admits",
    ),
    entry(
        S::SuccessorConstructorFault,
        "wrong-announcement-cycle",
        G::MutationStagingOwedOnTheLinkedCandidate,
        K::FiledAgainstAPublishedRequirement,
        "the announced cycle is one value inside that record, so a cycle other than the transition's is written at that position and nowhere else",
    ),
    entry(
        S::SuccessorConstructorFault,
        "wrong-representation-nonce",
        G::NoPublishedRelationFilesTheObservation,
        K::NotFiledAgainstAnyRequirement,
        "the row names no published relation, so its standing carries no requirement and a refusal of a wrong nonce would have nothing to be recorded against",
    ),
    entry(
        S::SuccessorConstructorFault,
        "later-admissible-nonce-instead-of-first",
        G::NoPublishedRelationFilesTheObservation,
        K::NotFiledAgainstAnyRequirement,
        "the first-admissible rule this row tests is a property of the search rather than an obligation the plan publishes, so the row resolves to no requirement either",
    ),
    entry(
        S::SuccessorConstructorFault,
        "successor-under-another-static-subtree",
        G::ProgramDerivationHasNoKnobForTheFault,
        K::FiledAgainstAPublishedRequirement,
        "the static subtree the successor is built under belongs to the emission, and nothing here emits a bundle under a subtree supplied from outside",
    ),
    entry(
        S::SuccessorConstructorFault,
        "wrong-internal-key",
        G::ProgramDerivationHasNoKnobForTheFault,
        K::FiledAgainstAPublishedRequirement,
        "the successor's internal key is computed in that same emission, so the row needs a linker that accepts a wrong key rather than a write on the candidate",
    ),
    entry(
        S::SuccessorConstructorFault,
        "wrong-parity",
        G::ProgramDerivationHasNoKnobForTheFault,
        K::FiledAgainstAPublishedRequirement,
        "the output key's parity falls out of the tweak the emission computes, so there is no position on the candidate at which a wrong parity could be written",
    ),
    entry(
        S::SuccessorConstructorFault,
        "wrong-leaf-version",
        G::ProgramDerivationHasNoKnobForTheFault,
        K::FiledAgainstAPublishedRequirement,
        "the successor's leaf version is fixed by the same derivation, and the emitted bundle carries no knob at which another version could enter",
    ),
    entry(
        S::SuccessorConstructorFault,
        "wrong-control-recipe",
        G::ProgramDerivationHasNoKnobForTheFault,
        K::FiledAgainstAPublishedRequirement,
        "the control recipe is computed from the taptree the emission built, so a wrong recipe is a derivation this workspace does not expose rather than a value to substitute",
    ),
    entry(
        S::SuccessorConstructorFault,
        "arbitrary-caller-supplied-output-program",
        G::ProgramDerivationHasNoKnobForTheFault,
        K::FiledAgainstAPublishedRequirement,
        "a caller-supplied output program is exactly the emission the linker declines to make, so the candidate offers no position at which to place one",
    ),
    entry(
        S::SuccessorConstructorFault,
        "no-state-successor",
        G::NoAdmittedShapeCarriesTheFault,
        K::FiledAgainstAPublishedRequirement,
        "the shape this ceremony builds always carries one state successor, so removing it is a differently built transaction rather than a changed field of this one",
    ),
    entry(
        S::SuccessorConstructorFault,
        "two-state-successors",
        G::NoAdmittedShapeCarriesTheFault,
        K::FiledAgainstAPublishedRequirement,
        "that same fixed shape has no second state output to fill, so the row waits on a build that emits two",
    ),
    entry(
        S::SuccessorConstructorFault,
        "extra-state-like-output",
        G::NoAdmittedShapeCarriesTheFault,
        K::FiledAgainstAPublishedRequirement,
        "an additional state-like output is a position the shape does not have, so the row waits on a build that emits one beside the successor",
    ),
    // §16.11 — the ABI and linker faults. Five name no published relation
    // at all; the sixth presents a control block from another program.
    entry(
        S::AbiLinkerFault,
        "wrong-transaction-version",
        G::NoPublishedRelationFilesTheObservation,
        K::NotFiledAgainstAnyRequirement,
        "the row names no published relation, so a consensus refusal of a wrong version would establish the node's own rule with no requirement to record it against",
    ),
    entry(
        S::AbiLinkerFault,
        "wrong-sequence",
        G::NoPublishedRelationFilesTheObservation,
        K::NotFiledAgainstAnyRequirement,
        "the sequence field carries no published relation either, and the plan names nothing a refusal at it could discharge",
    ),
    entry(
        S::AbiLinkerFault,
        "witness-item-reorder",
        G::NoPublishedRelationFilesTheObservation,
        K::NotFiledAgainstAnyRequirement,
        "reordering the witness names no published relation, so an observation a run produced would have no obligation to answer",
    ),
    entry(
        S::AbiLinkerFault,
        "predecessor-and-successor-metadata-witnesses-exchanged",
        G::NoPublishedRelationFilesTheObservation,
        K::NotFiledAgainstAnyRequirement,
        "exchanging the two metadata witnesses names no published relation, so the row could not be filed as answered even once a target had refused it",
    ),
    entry(
        S::AbiLinkerFault,
        "control-block-from-another-program",
        G::RefusalIsProgramGeneric,
        K::FiledAgainstAPublishedRequirement,
        "a block taken from another program does not commit to this spend's output key, so the commitment rule answers before the covenant's clause and answers alike for every such block",
    ),
    entry(
        S::AbiLinkerFault,
        "target-bytes-changed-after-abi-validation",
        G::NoPublishedRelationFilesTheObservation,
        K::NotFiledAgainstAnyRequirement,
        "a byte changed after validation asks about this workspace's own handoff, and the plan publishes no relation the row could resolve to",
    ),
];

/// The rows the classifier itself leaves waiting on a run, recomputed.
///
/// The register's counterpart and the reason it can be trusted: this
/// reads every row's standing from the derived plan rather than from any
/// list, so the two can be compared instead of one being asserted. The
/// plan is derived with the provenance the operator has not stated, as
/// the plan's own tests derive it, so the answer does not change with the
/// environment the caller happens to run in.
///
/// Each row is returned as its table and its name, because a name alone
/// does not identify a row of this matrix.
///
/// # Errors
///
/// Whatever [`derive_maturity_evidence_plan_with`] refuses with,
/// propagated unchanged. This function mints no refusal of its own: a
/// register that disagrees with the classifier is a defect in the
/// register, which a test reports, and not a condition the crate should
/// hand a caller at run time.
pub fn outstanding_rows()
-> Result<Vec<(MaturitySafetySection, &'static str)>, MaturityEvidenceRefusal> {
    let plan = derive_maturity_evidence_plan_with(
        MaturityExecutorProvenanceExpectation::NotStatedByTheOperator,
    )?;
    let waiting: Vec<RowIdentity> = plan
        .rows()
        .iter()
        .filter(|row| matches!(row.standing(), MaturityRowStanding::NativeRunRequired(_)))
        .map(|row| (row.row().section(), row.row().name()))
        .collect();
    Ok(waiting)
}

/// How many rows carry each gap, counted from the register.
///
/// Counted rather than written down, and counted from `STILL_REQUIRED`
/// alone, so the distribution has exactly one place to be wrong in.
#[must_use]
pub fn gap_census() -> BTreeMap<MaturityNegativeHalfGap, usize> {
    let mut census = BTreeMap::new();
    for entry in STILL_REQUIRED {
        *census.entry(entry.gap()).or_insert(0) += 1;
    }
    census
}

#[cfg(test)]
mod tests {
    use super::{
        MaturityEvidenceRefusal, MaturityExecutorProvenanceExpectation, MaturityNegativeHalfEntry,
        MaturityNegativeHalfGap, MaturityNegativeHalfLink, MaturityRowStanding, RowIdentity,
        STILL_REQUIRED, derive_maturity_evidence_plan_with, gap_census, outstanding_rows,
    };
    use crate::maturity_evidence::MaturityAnnouncementEvidencePlan;
    use std::collections::BTreeSet;

    /// One entry's identity, spelled the way the classifier spells a
    /// row's.
    fn identity(entry: &MaturityNegativeHalfEntry) -> RowIdentity {
        (entry.section(), entry.row())
    }

    /// The plan every test here reads, derived the way the register
    /// derives it.
    fn plan() -> Result<MaturityAnnouncementEvidencePlan, MaturityEvidenceRefusal> {
        derive_maturity_evidence_plan_with(
            MaturityExecutorProvenanceExpectation::NotStatedByTheOperator,
        )
    }

    /// The register and the classifier name the same rows.
    ///
    /// Set equality in both directions, reported as the two differences
    /// rather than as one boolean, so a failure says which rows drifted
    /// and in which direction instead of only that something did.
    #[test]
    fn every_outstanding_row_is_registered() {
        let computed: BTreeSet<RowIdentity> = outstanding_rows()
            .expect("the evidence plan derives")
            .into_iter()
            .collect();
        let registered: BTreeSet<RowIdentity> = STILL_REQUIRED.iter().map(identity).collect();

        let unregistered: Vec<&RowIdentity> = computed.difference(&registered).collect();
        assert_eq!(
            unregistered.len(),
            0,
            "rows still waiting on a run and absent from the register: {unregistered:?}",
        );

        let stale: Vec<&RowIdentity> = registered.difference(&computed).collect();
        assert_eq!(
            stale.len(),
            0,
            "rows registered as waiting that the classifier has answered: {stale:?}",
        );
    }

    /// The register lists each row once.
    ///
    /// A duplicated entry would satisfy set equality above while making
    /// the register's own length disagree with what it holds, and the
    /// identity is the table and the name because two names repeat inside
    /// the waiting set.
    #[test]
    fn the_register_holds_no_duplicate() {
        let unique: BTreeSet<RowIdentity> = STILL_REQUIRED.iter().map(identity).collect();
        assert_eq!(
            unique.len(),
            STILL_REQUIRED.len(),
            "the register lists a row more than once",
        );
    }

    /// Every entry states a ground, and states its own.
    ///
    /// A gap without a ground would be the `unassessed` member the
    /// vocabulary deliberately lacks, reintroduced as an empty string; a
    /// ground shared between two entries would be the gap's own meaning
    /// copied down, which says nothing about either row.
    #[test]
    fn every_entry_grounds_its_gap() {
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for entry in STILL_REQUIRED {
            assert_ne!(
                entry.ground().len(),
                0,
                "{:?} {} names a gap without grounding it",
                entry.section(),
                entry.row(),
            );
            assert!(
                seen.insert(entry.ground()),
                "{:?} {} restates a ground another entry already gave",
                entry.section(),
                entry.row(),
            );
        }
    }

    /// The gap census covers the register and nothing else.
    ///
    /// Recomputed rather than asserted per member: what matters is that
    /// no row escapes a gap, which is the sum agreeing with the length,
    /// and that no member is carried without a row to carry it.
    #[test]
    fn the_gap_census_accounts_for_every_row() {
        let census = gap_census();
        assert_eq!(
            census.get(&MaturityNegativeHalfGap::TargetRunNotYetPlanned),
            None
        );
        assert_eq!(census.values().sum::<usize>(), 42);
        assert_eq!(
            census.values().sum::<usize>(),
            STILL_REQUIRED.len(),
            "the gap census and the register disagree about how many rows there are",
        );
        for (gap, count) in &census {
            assert_ne!(*count, 0, "{gap:?} is carried by no row");
        }
    }

    /// The register's length is the figure the plan recomputes.
    ///
    /// Held against the census bucket rather than against a number
    /// written here, so a row entering or leaving the waiting standing
    /// fails a test instead of leaving a remembered total behind.
    #[test]
    fn the_register_length_is_the_figure_the_plan_recomputes() {
        let plan = plan().expect("the evidence plan derives");
        assert_eq!(STILL_REQUIRED.len(), 42);
        assert!(
            !STILL_REQUIRED
                .iter()
                .any(|entry| entry.row() == "sponsorless")
        );
        assert_eq!(
            STILL_REQUIRED.len(),
            plan.census().native_run_required(),
            "the register and the census disagree about the waiting denominator",
        );
    }

    /// The rows a wave could simply do are separable from the walls.
    ///
    /// The register's practical purpose, asserted so it keeps working: a
    /// reader planning the next bite filters for the work and gets it.
    /// Either side is allowed to shrink — a wall answered by a ruling
    /// leaves, and the work is finished when a stager lands — but neither
    /// may swallow the register, because a vocabulary that sorts every
    /// row into one bucket has stopped triaging.
    #[test]
    fn the_workable_rows_are_separable_from_the_walls() {
        let walls = STILL_REQUIRED
            .iter()
            .filter(|entry| {
                matches!(
                    entry.gap(),
                    MaturityNegativeHalfGap::RefusalIsProgramGeneric
                        | MaturityNegativeHalfGap::NoPublishedRelationFilesTheObservation
                )
            })
            .count();
        assert_ne!(walls, 0, "no row is a wall, so the split says nothing");
        assert_ne!(
            walls,
            STILL_REQUIRED.len(),
            "every row cannot be a wall, or no work remains to plan",
        );
    }

    /// Each entry's link marker is the one the classifier computed.
    ///
    /// The marker mirrors the optional requirement the standing carries,
    /// so this holds the register's claim against the resolution rather
    /// than against a table of identities kept beside it: filed exactly
    /// where the standing resolved to one requirement, and not filed
    /// exactly where it resolved to none.
    #[test]
    fn every_entry_agrees_with_the_published_link() {
        let plan = plan().expect("the evidence plan derives");
        let filed: BTreeSet<RowIdentity> = plan
            .rows()
            .iter()
            .filter(|row| {
                matches!(
                    row.standing(),
                    MaturityRowStanding::NativeRunRequired(Some(_))
                )
            })
            .map(|row| (row.row().section(), row.row().name()))
            .collect();

        for entry in STILL_REQUIRED {
            let expected = if filed.contains(&identity(entry)) {
                MaturityNegativeHalfLink::FiledAgainstAPublishedRequirement
            } else {
                MaturityNegativeHalfLink::NotFiledAgainstAnyRequirement
            };
            assert_eq!(
                entry.link(),
                expected,
                "{:?} {} disagrees with the requirement its standing resolved to",
                entry.section(),
                entry.row(),
            );
        }
    }
}
