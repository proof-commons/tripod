//! The complete required safety matrix of Guide-14 §16.
//!
//! §16 states thirteen tables and, in one preamble above all of them,
//! nine facts every row carries: polarity, mutation layer, declared
//! evidence boundary, intended relation, intended carrier, collateral
//! closure, canonical control, mutation locator, and expected
//! projection. This module is that transcription: every row of every
//! table, once, in the guide's own order, each carrying all nine.
//!
//! # One transcription, one denominator
//!
//! The tables are transcribed here and nowhere else, and the row count
//! is computed from the rows rather than written beside them. Two
//! modules transcribing one guide section into two tables would
//! reproduce, at the specification level, the defect the boundary rule's
//! own record names: two consumers of one semantic rule, one of which
//! had no access to it, is how wrong-boundary rows came to stand as
//! answered. A second table would also give the matrix a second
//! denominator, and a completeness claim with two denominators is a
//! claim about whichever one the reader happened to open.
//!
//! # Why all nine facts and not the five a live row carries
//!
//! The live generation's row carries a section and a name plus five of
//! the nine — polarity, mutation layer, declared boundary, intended
//! relation and collateral. The intended carrier, the canonical control,
//! the mutation locator and the expected projection have no field on it,
//! and a fact with nowhere to sit is a fact nobody can check. All four
//! are decidable before anything executes: which carrier a row means to
//! exercise, which accepted control its mutant departs from, where its
//! change sits, and what its projection is expected to say are
//! properties of the row's design rather than of a run. The facts a run
//! computes — the accepted control's identity, the exact submitted bytes
//! — belong on an observed standing instead, where the run that computed
//! them can be checked against them.
//!
//! # A row another wave answers carries a reason, never a silence
//!
//! Several rows here are not this wave's to answer. Each of them carries
//! a typed member of [`MaturityRowBoundary`] naming why, because an
//! absent row reads as a discharged one and a row left standing with no
//! reason reads as an oversight. A named reason states what would have to
//! change for the row to be answerable, and it fails loudly when that
//! thing changes: the reason is a value a test reads, not a sentence in a
//! comment.
//!
//! # A link is computed, never tabulated
//!
//! A row states which relation its change is meant to violate and which
//! published class of that relation it stages, and nothing here writes
//! down the requirement those two select. [`resolve_row`] computes it
//! against the compiler's validated announcement plan, so a plan that
//! moves underneath the matrix fails the matrix: a relation that stops
//! being published, a class that stops being required, or a class that
//! starts being required twice all turn a row that used to resolve into a
//! refusal or a stated non-answer. A table of requirement identities
//! would instead keep agreeing with itself, which is the one thing a
//! completeness claim must not be able to do.
//!
//! # Nothing here is evidence
//!
//! This module holds no fixture, no bundle, no transaction, no witness,
//! no verdict and no report. It is a specification transcribed into
//! types, and it observes nothing. That is what keeps the later waves
//! falsifiable rather than self-confirming: a row whose expectations were
//! written after the run would be passed by whatever the run did.
//!
//! # The row vocabulary has one home
//!
//! §16.1's fourteen positive rows are named by the table's own wording in
//! the kebab form [`MaturitySafetyRow`] renders, and they stand in the
//! table's order, so a row's ordinal is its position. The canonical
//! semantic fixture registry transcribes the same fourteen and keys them
//! by those ordinals; once it keys to this matrix, the wording will have
//! one home here and the registry will carry fixtures rather than a
//! second copy of the names.
//!
//! # Metadata carrier moves
//!
//! Under the variable witness schedule, the witness carries neither the
//! constant header nor reserved zeros. The leaf re-materializes them before
//! the existing checks. The ABI still refuses non-canonical encodings at
//! construction, so the declared boundary stays with each row even though
//! its byte carrier moves.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{AssetId, ObjectId, OperationId};
use compiler::maturity_announcement_plan::{
    MaturityAnnouncementRepresentationPlan, ValidatedMaturityAnnouncementOperationPlan,
};
use compiler::operation_plan::{
    CollateralPolicy, CoverageBoundary, CoverageRequirementId, RelationMutation, SponsorCase,
    TargetCoverageObligation,
};
use realization::{RelationId, RelationKind, RelationSubject, TransactionSide};

use crate::error::VectorError;
use crate::matrix::{EvidenceBoundary, MutationLayer};

/// The §16 table one row is drawn from.
///
/// The thirteen sections partition the matrix exactly: every row belongs
/// to one table, and the per-section counts recomputed in this module's
/// tests must sum to the whole.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturitySafetySection {
    /// §16.1 — positive cases.
    Positive,
    /// §16.2 — maturity-window faults.
    WindowFault,
    /// §16.3 — metadata faults.
    MetadataFault,
    /// §16.4 — operator faults.
    OperatorFault,
    /// §16.5 — predecessor-constructor faults.
    PredecessorConstructorFault,
    /// §16.6 — successor-constructor faults.
    SuccessorConstructorFault,
    /// §16.7 — branch-order and totality faults.
    TotalityFault,
    /// §16.8 — root-history faults.
    RootHistoryFault,
    /// §16.9 — absence and economic faults.
    AbsenceFault,
    /// §16.10 — sponsor faults.
    SponsorFault,
    /// §16.11 — ABI and linker faults.
    AbiLinkerFault,
    /// §16.12 — protocol and report faults.
    ProtocolReportFault,
    /// §16.13 — public-recovery faults.
    RecoveryFault,
}

impl MaturitySafetySection {
    /// Every section, in §16 order.
    pub const ALL: &'static [Self] = &[
        Self::Positive,
        Self::WindowFault,
        Self::MetadataFault,
        Self::OperatorFault,
        Self::PredecessorConstructorFault,
        Self::SuccessorConstructorFault,
        Self::TotalityFault,
        Self::RootHistoryFault,
        Self::AbsenceFault,
        Self::SponsorFault,
        Self::AbiLinkerFault,
        Self::ProtocolReportFault,
        Self::RecoveryFault,
    ];

    /// The guide section this table is stated in.
    #[must_use]
    pub const fn section(self) -> &'static str {
        match self {
            Self::Positive => "16.1",
            Self::WindowFault => "16.2",
            Self::MetadataFault => "16.3",
            Self::OperatorFault => "16.4",
            Self::PredecessorConstructorFault => "16.5",
            Self::SuccessorConstructorFault => "16.6",
            Self::TotalityFault => "16.7",
            Self::RootHistoryFault => "16.8",
            Self::AbsenceFault => "16.9",
            Self::SponsorFault => "16.10",
            Self::AbiLinkerFault => "16.11",
            Self::ProtocolReportFault => "16.12",
            Self::RecoveryFault => "16.13",
        }
    }

    /// Whether every row of this table expects the announcement to stand.
    #[must_use]
    pub const fn is_positive(self) -> bool {
        matches!(self, Self::Positive)
    }
}

/// What a row expects of the announcement it describes.
///
/// Two members and no third. §16 is a table of valid announcements that
/// must stand and invalid ones that must be refused; a row whose subject
/// were the executor would belong to the infrastructure layer, which this
/// matrix does not name and this module therefore does not invent.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturitySafetyPolarity {
    /// The complete announcement is expected to stand, and its
    /// projections to equal the expected ones.
    Positive,
    /// The announcement is expected to be refused at the row's stated
    /// boundary.
    Negative,
}

impl MaturitySafetyPolarity {
    /// Both polarities.
    pub const ALL: &'static [Self] = &[Self::Positive, Self::Negative];
}

/// Where one §16 row's verdict comes from, or why none is available.
///
/// Most rows expect a layer to accept or refuse, and [`Self::Layer`]
/// carries which. A row this wave cannot answer carries the exact reason
/// instead, so that the row stands with a stated cause rather than as an
/// unexplained gap in a table that claims to be complete.
///
/// # Why a typed reason rather than an omission or a note
///
/// An omitted row makes the denominator smaller and the completeness
/// claim stronger, which is the wrong direction for a matrix whose whole
/// purpose is to bound what has been established. A note in a comment
/// cannot be read by a census. A member can: a test asserts which rows
/// carry which reason, so a reason that stopped applying — because the
/// wave that owned it landed its evidence — fails here rather than ageing
/// quietly into a claim nobody rechecks.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum MaturityRowBoundary {
    /// One named layer produces the verdict.
    Layer(EvidenceBoundary),
    /// The row's answer is an acceptance, and no acceptance is reachable.
    ///
    /// The announcement spend meets a standardness width measured before
    /// execution, so a node accepts the block and refuses the spend at
    /// relay policy: the whole chain from linked bytes through the static
    /// subtree, the output key and the control block is exercised, and
    /// the spend is stopped by a width and by nothing else. A row whose
    /// answer is an acceptance has two routes to one and neither has
    /// landed — the relay restructure the candidate ABI already carries
    /// as an unopened obligation, and a protocol revision adding a
    /// block-layer submission subject, both sides moving together with
    /// the schema bumped and the historical contract still parsed. A
    /// relaxed node policy is not a third route: no deployed argument
    /// list relaxes standardness, and an acceptance under a policy nobody
    /// runs would have to carry that caveat wherever it went.
    AcceptanceAwaitsRelayAdmissibility,
    /// The sponsor relations have no region, so the row is refused by
    /// name rather than at the boundary it would declare.
    ///
    /// Until the region-scoping refit gives the sponsor relations a
    /// region, a sponsored row's declared boundary would name a layer the
    /// row cannot arrive at, and a row whose declared boundary is
    /// unreachable can be passed only by a verdict from somewhere else.
    /// Declaring one anyway would be worse than stating the standing: the
    /// layer named would never have been asked the row's question.
    SponsorArmAwaitsRegionScoping,
    /// The row is answered by landed nonce and tweak evidence.
    ///
    /// The admissible-nonce search and the tweak arithmetic were
    /// established with their own oracle and their own negatives, and
    /// that evidence stands. Transcribing the row as outstanding would
    /// record one obligation twice, and the moment either copy moved the
    /// other would be wrong with nothing to catch it.
    TotalityAnsweredByLandedNonceEvidence,
    /// The row asserts a property over repeated construction.
    ///
    /// Equal typed inputs producing equal candidate bytes is a statement
    /// about the build rather than about any one fixture: no mutation
    /// locates it, no control precedes it, and no layer refuses it. It is
    /// answered by constructing twice and comparing, which is a
    /// determinism check and not a row of this matrix.
    PropertyOfTheBuildRatherThanAFixture,
}

impl MaturityRowBoundary {
    /// The layer that produces the verdict, where one does.
    #[must_use]
    pub const fn layer(self) -> Option<EvidenceBoundary> {
        match self {
            Self::Layer(boundary) => Some(boundary),
            Self::AcceptanceAwaitsRelayAdmissibility
            | Self::SponsorArmAwaitsRegionScoping
            | Self::TotalityAnsweredByLandedNonceEvidence
            | Self::PropertyOfTheBuildRatherThanAFixture => None,
        }
    }

    /// Whether the row states a reason no layer answers it.
    #[must_use]
    pub const fn is_typed_non_answer(self) -> bool {
        self.layer().is_none()
    }
}

/// The carrier one row intends to exercise.
///
/// The first five are the carrier plan's own classes, transcribed. The
/// sixth names what that plan does not: the carrier plan assigns semantic
/// relations to carriers, and a row whose subject is a protocol record
/// carries no relation to assign, so its carrier has no class there. A
/// protocol row filed under the report would claim the report's bytes as
/// its subject, which is a different question with a different answer.
///
/// A row's intended carrier is not where its verdict comes from. A row
/// may intend the announcement leaf and be refused by the ABI before the
/// leaf runs at all; whether the carrier executed is a fact of the run,
/// recorded on an observed standing rather than claimed here.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityIntendedCarrier {
    /// Predecessor metadata authentication, the maturity predecessor, the
    /// lead window, successor metadata derivation, successor constructor
    /// reconstruction, and operator authorization.
    AnnouncementLeaf,
    /// Exact transaction counts, the STATE input and output closure, root
    /// and event absence, and the sponsor boundary.
    CoordinatorStructure,
    /// The metadata-leaf commitment, static-subtree continuity, and
    /// internal-key and leaf-version policy.
    LinkedConstructor,
    /// Selected signature semantics, whole-transaction conservation, and
    /// taproot commitment and control-path validity.
    Target,
    /// Current-root freshness, the root-history edge sequence, and public
    /// reconstruction.
    Report,
    /// The typed request and response records of the conformance
    /// protocol.
    TypedProtocolExchange,
}

/// The source of fixed metadata bytes restored by a variable-schedule leaf.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaturityReMaterializedCarrier {
    /// The linker-resolved constant metadata header.
    LeafHeaderSymbol,
    /// The leaf's literal reserved zeros.
    LeafReservedLiteral,
}

/// One metadata fault whose fixed bytes move from witness to leaf.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaturityCarrierMove {
    row: &'static str,
    whole_carrier: MaturityIntendedCarrier,
    variable_carrier: MaturityReMaterializedCarrier,
    boundary: MaturityRowBoundary,
}

impl MaturityCarrierMove {
    const fn from_row(
        row: &MaturitySafetyRow,
        variable_carrier: MaturityReMaterializedCarrier,
    ) -> Self {
        Self {
            row: row.name(),
            whole_carrier: row.carrier(),
            variable_carrier,
            boundary: row.boundary(),
        }
    }

    /// The matrix row whose changed bytes moved.
    #[must_use]
    pub const fn row(&self) -> &'static str {
        self.row
    }

    /// The carrier named by the whole-schedule matrix row.
    #[must_use]
    pub const fn whole_carrier(&self) -> MaturityIntendedCarrier {
        self.whole_carrier
    }

    /// The source of those bytes in the variable-schedule leaf.
    #[must_use]
    pub const fn variable_carrier(&self) -> MaturityReMaterializedCarrier {
        self.variable_carrier
    }

    /// The unchanged declared refusal boundary copied from the row.
    #[must_use]
    pub const fn boundary(&self) -> MaturityRowBoundary {
        self.boundary
    }
}

/// The accepted control a row's mutant departs from.
///
/// A negative row is answered by a refusal only when its control was
/// accepted: without one, a refusal establishes that the construction
/// fails, not that the row's fault is what fails it. Naming the control
/// on the row is what makes that condition checkable before a run rather
/// than argued after one.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityCanonicalControl {
    /// The row is itself a control and departs from nothing.
    TheRowIsTheControl,
    /// The canonical sponsorless announcement.
    SponsorlessAnnouncement,
    /// The canonical sponsored announcement.
    SponsoredAnnouncement,
    /// The canonical well-formed request and response exchange.
    CanonicalProtocolExchange,
    /// The canonical safety report rendered from a completed run.
    CanonicalSafetyReport,
    /// The published announcement transaction a recovery reads back.
    AcceptedAnnouncementPublication,
}

/// Where one row's change sits.
///
/// The locator is the structural place a reviewer can point at, stated
/// before anything runs. A refusal that named no locator would leave the
/// reader to infer which change was being refused, and a refusal for an
/// unrelated reason reads exactly like a refusal for the intended one.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum MaturityMutationLocator {
    /// A field of the typed semantic request.
    SemanticRequestField,
    /// A semantic field of the predecessor STATE metadata.
    PredecessorMetadataField,
    /// A semantic field of the successor STATE metadata.
    SuccessorMetadataField,
    /// The metadata record's encoding: its schema tag, domain separator,
    /// enum tags, reserved fields, field order, or trailing bytes.
    MetadataEncoding,
    /// One operator signing response or its signature.
    OperatorSigningResponse,
    /// The approved operator key material or its profile.
    ApprovedOperatorKey,
    /// One leaf of the static subtree.
    StaticSubtreeLeaf,
    /// The linked constructor's emitted program.
    LinkedProgram,
    /// The control block, control recipe, internal key, parity, or leaf
    /// version of a spend path.
    ControlBlock,
    /// One witness item's position or content.
    WitnessStack,
    /// One transaction input.
    TransactionInput,
    /// One transaction output.
    TransactionOutput,
    /// A whole-transaction field: version, sequence, or a count.
    TransactionField,
    /// The set of object families one transaction side admits.
    ///
    /// The place a row changes when it adds a member of a family the
    /// operation's closure does not admit. The side is deliberately not
    /// part of the locator for those rows, because the table names the
    /// family and not the side.
    AdmittedObjectFamily,
    /// The sponsor envelope, its members, or its range.
    SponsorEnvelope,
    /// One edge or cursor of the root history.
    RootHistoryEdge,
    /// The taptree's branch order or the source order it was built from.
    BranchOrder,
    /// The admissible representation-nonce search.
    NonceSearch,
    /// The tweak arithmetic over the internal key.
    TweakArithmetic,
    /// A linker symbol or relocation.
    LinkerRelocation,
    /// A field of the typed protocol request.
    ProtocolRequestField,
    /// A field of the typed protocol response.
    ProtocolResponseField,
    /// A field, row, or summary of the canonical safety report.
    ReportField,
    /// The published material a public recovery reads.
    PublicationRecord,
}

/// What one row's change is about, as against where it sits.
///
/// A locator names the structural place a reviewer points at; a subject
/// names the object whose value or shape is different. The two are
/// different questions, and the second is the one a refusal has to be
/// checked against: a run reports where it saw a change, and a reader
/// asking whether that refusal answered the row it was built for is
/// asking whether the same object was changed, not whether the same
/// words were used for the place. Stating the subject as its own closed
/// vocabulary, reached by a total map, is what makes "the row's change
/// and no other" a comparison rather than a paraphrase.
///
/// # Why no two locators share a subject
///
/// The map is a bijection over the locators the matrix uses, and that is
/// the finding rather than a redundancy: it states that no locator is a
/// synonym of another, which is what keeps "exactly one subject" a claim
/// instead of a collapse. Two pairs read as candidates for merging and
/// are argued apart where they stand, at
/// [`Self::StaticSubtree`] against [`Self::BranchOrder`] and at
/// [`Self::LinkedProgram`] against [`Self::RelocationCensus`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityMutationSubject {
    /// The typed semantic request, as submitted.
    SemanticRequest,
    /// The authenticated state the transition reads.
    PredecessorMetadata,
    /// The state the transition derives.
    ///
    /// A different subject from the predecessor's: a row changing what
    /// was read claims the transition accepted a state it should have
    /// refused, and a row changing what was derived claims it produced
    /// one it should not have.
    SuccessorMetadata,
    /// The metadata record's bytes at unchanged semantics.
    ///
    /// The one subject whose change can sit between fields rather than
    /// in one, because the places it names include the domain
    /// separator, the field order, the reserved fields and the trailing
    /// bytes.
    MetadataEncoding,
    /// The operator response set submitted for authorization.
    OperatorResponse,
    /// The approved operator key material the responses are checked
    /// against.
    ///
    /// The authorization input rather than the answer to it, so a row
    /// changing the approved set is not a row changing a response.
    ApprovedOperatorKey,
    /// The static subtree's leaf content.
    ///
    /// Kept apart from [`Self::BranchOrder`] because changing a leaf's
    /// bytes reorders nothing and reordering the leaves changes no
    /// leaf's bytes: one row claims a commitment covers the wrong
    /// program, the other that the same programs commit in the wrong
    /// arrangement.
    StaticSubtree,
    /// The program the linked constructor emits.
    ///
    /// Kept apart from [`Self::RelocationCensus`] because the census is
    /// what the linker consumes and accounts for, and the program is
    /// what it produces: a row changing the census asks the linker to
    /// resolve something different, and a row changing the program asks
    /// a later layer to accept something the linker never emitted.
    LinkedProgram,
    /// A spend path's control material: internal key, parity, leaf
    /// version, control recipe.
    ControlBlock,
    /// The offered proof's items and their order.
    Witness,
    /// One transaction input.
    TransactionInput,
    /// One transaction output.
    ///
    /// Distinct from an input because the operation's closure is stated
    /// per side, so a row changing one side is answered by a different
    /// requirement from a row changing the other.
    TransactionOutput,
    /// A whole-transaction field: version, sequence, a count.
    TransactionField,
    /// The set of object families a transaction side admits.
    ///
    /// A set rather than a member, which is why it is not one of the
    /// two side subjects: the change adds a family the closure does not
    /// admit rather than altering an object it does.
    AdmittedFamily,
    /// The sponsor envelope, its members, or its range.
    SponsorEnvelope,
    /// The root history's edges and cursor.
    RootHistory,
    /// The order the taptree's branches stand in, and the source order
    /// it was built from.
    BranchOrder,
    /// The admissible representation nonce the search returns.
    Nonce,
    /// The tweak arithmetic over the internal key.
    ///
    /// The operation rather than the value the search produced, so a
    /// row changing the arithmetic is not a row changing the nonce.
    Tweak,
    /// The linker's symbol and relocation record.
    RelocationCensus,
    /// The typed protocol request record.
    WireRequest,
    /// The typed protocol response record.
    ///
    /// Distinct from the request because a malformed question and a
    /// malformed answer are refused by different halves of the
    /// exchange.
    WireResponse,
    /// A field, row, or summary of the rendered safety report.
    Report,
    /// The published material a recovery reads back.
    Publication,
}

impl MaturityMutationSubject {
    /// Every subject a locator of this matrix names.
    ///
    /// Written out rather than folded over the locators, so that it is
    /// an independent statement of the vocabulary's width: a subject
    /// added with no locator reaching it, and a locator whose rows all
    /// disappear, both fail the comparison against what the rows
    /// actually name.
    pub const ALL: &'static [Self] = &[
        Self::SemanticRequest,
        Self::PredecessorMetadata,
        Self::SuccessorMetadata,
        Self::MetadataEncoding,
        Self::OperatorResponse,
        Self::ApprovedOperatorKey,
        Self::StaticSubtree,
        Self::LinkedProgram,
        Self::ControlBlock,
        Self::Witness,
        Self::TransactionInput,
        Self::TransactionOutput,
        Self::TransactionField,
        Self::AdmittedFamily,
        Self::SponsorEnvelope,
        Self::RootHistory,
        Self::BranchOrder,
        Self::Nonce,
        Self::Tweak,
        Self::RelocationCensus,
        Self::WireRequest,
        Self::WireResponse,
        Self::Report,
        Self::Publication,
    ];
}

impl MaturityMutationLocator {
    /// The subject the change at this locator is about.
    ///
    /// Total by exhaustion and with no fallback arm: a locator minted
    /// later cannot compile until somebody states what it changes, and
    /// a wildcard here would answer that question by accident for every
    /// locator added after it — which is the failure the fail-closed
    /// direction exists to prevent.
    #[must_use]
    pub const fn subject(self) -> MaturityMutationSubject {
        match self {
            Self::SemanticRequestField => MaturityMutationSubject::SemanticRequest,
            Self::PredecessorMetadataField => MaturityMutationSubject::PredecessorMetadata,
            Self::SuccessorMetadataField => MaturityMutationSubject::SuccessorMetadata,
            Self::MetadataEncoding => MaturityMutationSubject::MetadataEncoding,
            Self::OperatorSigningResponse => MaturityMutationSubject::OperatorResponse,
            Self::ApprovedOperatorKey => MaturityMutationSubject::ApprovedOperatorKey,
            Self::StaticSubtreeLeaf => MaturityMutationSubject::StaticSubtree,
            Self::LinkedProgram => MaturityMutationSubject::LinkedProgram,
            Self::ControlBlock => MaturityMutationSubject::ControlBlock,
            Self::WitnessStack => MaturityMutationSubject::Witness,
            Self::TransactionInput => MaturityMutationSubject::TransactionInput,
            Self::TransactionOutput => MaturityMutationSubject::TransactionOutput,
            Self::TransactionField => MaturityMutationSubject::TransactionField,
            Self::AdmittedObjectFamily => MaturityMutationSubject::AdmittedFamily,
            Self::SponsorEnvelope => MaturityMutationSubject::SponsorEnvelope,
            Self::RootHistoryEdge => MaturityMutationSubject::RootHistory,
            Self::BranchOrder => MaturityMutationSubject::BranchOrder,
            Self::NonceSearch => MaturityMutationSubject::Nonce,
            Self::TweakArithmetic => MaturityMutationSubject::Tweak,
            Self::LinkerRelocation => MaturityMutationSubject::RelocationCensus,
            Self::ProtocolRequestField => MaturityMutationSubject::WireRequest,
            Self::ProtocolResponseField => MaturityMutationSubject::WireResponse,
            Self::ReportField => MaturityMutationSubject::Report,
            Self::PublicationRecord => MaturityMutationSubject::Publication,
        }
    }
}

/// One term of the projection comparison an accepted announcement
/// receives.
///
/// The four terms §16.1's last row names together, kept apart because a
/// row that expects one of them to differ is making a different claim
/// from a row that expects another.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityProjectionTerm {
    /// The semantic projection of the announcement.
    Semantic,
    /// The constructor projection.
    Constructor,
    /// The root-history projection.
    RootHistory,
    /// The public-recovery projection.
    PublicRecovery,
}

/// What a row expects the projection comparison to say.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityExpectedProjection {
    /// Every term matches.
    EveryTermMatches,
    /// Nothing is projected, because the row is refused before any
    /// acceptance and an unaccepted candidate projects nothing.
    NoProjection,
    /// The announcement stands and the named term is expected to differ.
    TermDiffers(MaturityProjectionTerm),
    /// The canonical serialization refuses to publish the value at all,
    /// so no term is compared.
    ForbiddenPublicationRefused,
}

/// One published negative class of the announcement's coverage.
///
/// A class is a boundary and a mutation together, because a boundary and
/// a mutation together is what the plan publishes: a relation's negatives
/// are its runtime mutation vocabulary at the carrier boundary plus the
/// boundary-and-mutation pairs it requires away from the runtime. Naming
/// the mutation alone would leave a row unable to say which obligation it
/// stages wherever one relation publishes one mutation at two boundaries,
/// and the announcement's lifecycle exit is such a relation: the plan
/// must require the exit and the emitted program must carry it, two
/// obligations whose mutation is the same word.
///
/// The members are the classes this matrix's rows stage and no others. A
/// member no row names would be a vocabulary entry standing on nothing,
/// and a class the plan does not publish would be a row's private
/// invention; the census tests refuse both.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityMutationClass {
    /// Fewer members of the relation's family than its minimum.
    CardinalityBelowMinimum,
    /// More members of the relation's family than its maximum.
    ///
    /// The ceiling the published mutation carries is deliberately not
    /// restated here: it is an architecture-owned bound the plan cites
    /// rather than copies, a copy on the row would be a draft that can
    /// drift from the bound it came from, and one relation-case publishes
    /// one above-maximum class, so the ceiling narrows nothing.
    CardinalityAboveMaximum,
    /// A member of the family carrying another asset.
    WrongRecognizedAsset,
    /// A member of the family that is another object.
    WrongRecognizedObject,
    /// A root effect other than the one the operation declares.
    WrongRootEffect,
    /// A projection the operation requires, absent.
    MissingRequiredProjection,
    /// A projection the operation forbids, present.
    ForbiddenProjectionPresent,
    /// No external report arrived where the relation needs one.
    ExternalReportMissing,
    /// An external report arrived and does not establish its premise.
    ExternalReportFailed,
    /// An external report establishes its premise for another subject.
    ExternalReportSubjectMismatch,
    /// The required lifecycle exit is absent from the validated plan.
    ///
    /// The plan-side half of the exit relation's pair. The rows that name
    /// it change a semantic fact of the announcement window, and what a
    /// semantic fact disturbs is the exit condition the plan's own static
    /// discharge decides; whether the emitted program retains the exit is
    /// a structural property of bytes, which no row of that table moves.
    RequiredExitMissingFromThePlan,
}

impl MaturityMutationClass {
    /// Every class the matrix's rows stage.
    pub const ALL: &'static [Self] = &[
        Self::CardinalityBelowMinimum,
        Self::CardinalityAboveMaximum,
        Self::WrongRecognizedAsset,
        Self::WrongRecognizedObject,
        Self::WrongRootEffect,
        Self::MissingRequiredProjection,
        Self::ForbiddenProjectionPresent,
        Self::ExternalReportMissing,
        Self::ExternalReportFailed,
        Self::ExternalReportSubjectMismatch,
        Self::RequiredExitMissingFromThePlan,
    ];

    /// The class's own name, for a report and for the census.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::CardinalityBelowMinimum => "CardinalityBelowMinimum",
            Self::CardinalityAboveMaximum => "CardinalityAboveMaximum",
            Self::WrongRecognizedAsset => "WrongRecognizedAsset",
            Self::WrongRecognizedObject => "WrongRecognizedObject",
            Self::WrongRootEffect => "WrongRootEffect",
            Self::MissingRequiredProjection => "MissingRequiredProjection",
            Self::ForbiddenProjectionPresent => "ForbiddenProjectionPresent",
            Self::ExternalReportMissing => "ExternalReportMissing",
            Self::ExternalReportFailed => "ExternalReportFailed",
            Self::ExternalReportSubjectMismatch => "ExternalReportSubjectMismatch",
            Self::RequiredExitMissingFromThePlan => "RequiredExitMissingFromThePlan",
        }
    }

    /// Where the plan indexes this class's requirement.
    #[must_use]
    pub const fn boundary(self) -> CoverageBoundary {
        match self {
            Self::CardinalityBelowMinimum
            | Self::CardinalityAboveMaximum
            | Self::WrongRecognizedAsset
            | Self::WrongRecognizedObject
            | Self::WrongRootEffect
            | Self::MissingRequiredProjection
            | Self::ForbiddenProjectionPresent => CoverageBoundary::RuntimeCarrier,
            Self::ExternalReportMissing
            | Self::ExternalReportFailed
            | Self::ExternalReportSubjectMismatch => CoverageBoundary::ExternalEvidence,
            Self::RequiredExitMissingFromThePlan => CoverageBoundary::CompilerStatic,
        }
    }

    /// Whether one published mutation is this class.
    #[must_use]
    pub const fn matches(self, mutation: &RelationMutation) -> bool {
        match self {
            Self::CardinalityBelowMinimum => {
                matches!(mutation, RelationMutation::CardinalityBelowMinimum)
            }
            Self::CardinalityAboveMaximum => {
                matches!(mutation, RelationMutation::CardinalityAboveMaximum { .. })
            }
            Self::WrongRecognizedAsset => {
                matches!(mutation, RelationMutation::WrongRecognizedAsset)
            }
            Self::WrongRecognizedObject => {
                matches!(mutation, RelationMutation::WrongRecognizedObject)
            }
            Self::WrongRootEffect => matches!(mutation, RelationMutation::WrongRootEffect),
            Self::MissingRequiredProjection => {
                matches!(mutation, RelationMutation::MissingRequiredProjection)
            }
            Self::ForbiddenProjectionPresent => {
                matches!(mutation, RelationMutation::ForbiddenProjectionPresent)
            }
            Self::ExternalReportMissing => {
                matches!(mutation, RelationMutation::ExternalEvidenceMissing)
            }
            Self::ExternalReportFailed => {
                matches!(mutation, RelationMutation::ExternalEvidenceFailed)
            }
            Self::ExternalReportSubjectMismatch => {
                matches!(mutation, RelationMutation::ExternalEvidenceIdentityMismatch)
            }
            Self::RequiredExitMissingFromThePlan => {
                matches!(mutation, RelationMutation::RequiredLifecycleExitMissing)
            }
        }
    }
}

/// Why one §16 row names no published relation.
///
/// Four distinct situations, kept apart because they call for four
/// different repairs and only some of them are this workspace's to make.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum MaturityUnlinkedReason {
    /// The published relation inventory has nothing to index the row by.
    ///
    /// §16 asks for the class and the operation's relation census names
    /// no member of it. That is a gap between two authorities rather than
    /// a defect in either, and it is reported rather than closed by
    /// filing the row under whichever relation looked nearest.
    NoPublishedRelationNamesIt,
    /// Several published relations fit the row's wording equally.
    ///
    /// §16 names the row and nothing narrows it to one relation, so
    /// picking one would file the row under part of its own subject.
    SeveralPublishedRelationsFit,
    /// The row's subject is a report, not a transaction.
    ///
    /// No target can answer what the report published, and no relation is
    /// indexed by it: the boundary is this workspace's own canonical
    /// serialization, and the evidence is a property of the bytes a
    /// report renders rather than of anything an announcement did.
    TheReportIsTheSubject,
    /// The relation the row's fault belongs to is published, and is
    /// evaluated over the whole observed transaction.
    ///
    /// Five of the announcement's relations quantify over everything the
    /// observed transaction carries rather than over the positions the
    /// operation claims: the object-family closures of both sides, the
    /// sponsor isolation, the canonical delta partition, the open-flow
    /// policy and the sponsor envelope count. No leaf enforces any of
    /// them, and the realization that does enforce them reads a whole
    /// transaction, so linking such a row to a published requirement
    /// would claim the plan constrains the region this operation occupies
    /// when what it constrains is every operation the transaction
    /// happens to carry. A link here would be a guess in the one
    /// direction the closure principle forbids as firmly as the other:
    /// the model would refuse what the covenant accepts.
    ///
    /// What would have to land for the row to link is the region-scoping
    /// refit: region membership becomes part of the observation, a
    /// transaction becomes the union of its operations' regions, and each
    /// of those relations becomes a statement about one region. After
    /// that the relation publishes a class about this operation and the
    /// row returns to [`MaturityRelationStanding::Declared`] and
    /// resolves, with the sponsor envelope count retired or re-scoped
    /// with the rest because it is resource policy rather than semantics.
    RelationScopedToTheWholeTransaction,
}

/// What one §16 row intends to violate, or to preserve.
///
/// The relation and the class travel as stated identities and no
/// requirement identity is tabulated beside them. The row says which
/// relation it means and which published class of it the change stages;
/// [`resolve_row`] is what turns that into one requirement, and what
/// refutes a row whose relation or class the plan does not publish. A
/// requirement identity written down here would be a second source of
/// truth that could agree with neither side, and it would keep agreeing
/// with itself after the plan moved.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaturityRelationStanding {
    /// The row intends every published relation of the operation to hold.
    ///
    /// The honest standing of a positive row. §16.1 names announcement
    /// *cases* — a minimum lead, a sponsorless shape — and a case is not
    /// a relation: what a valid announcement must preserve is the whole
    /// relation census, not one member of it. A positive row naming a
    /// single relation would be claiming the others were not its
    /// business.
    EveryPublishedRelation,
    /// The row names one published relation and one of its classes.
    Declared {
        /// The relation the change is intended to violate.
        relation: RelationId,
        /// The published negative class the change stages.
        class: MaturityMutationClass,
    },
    /// The row names none, for a stated reason.
    Unlinked(MaturityUnlinkedReason),
}

/// One row of the §16 required safety matrix.
///
/// The name is the guide's own wording normalized to an identifier, so a
/// reader can check the transcription against §16 line by line without
/// trusting a paraphrase.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturitySafetyRow {
    section: MaturitySafetySection,
    name: &'static str,
    polarity: MaturitySafetyPolarity,
    mutation: Option<MutationLayer>,
    boundary: MaturityRowBoundary,
    relation: MaturityRelationStanding,
    carrier: MaturityIntendedCarrier,
    collateral: Option<CollateralPolicy>,
    control: MaturityCanonicalControl,
    locator: Option<MaturityMutationLocator>,
    projection: MaturityExpectedProjection,
}

impl MaturitySafetyRow {
    /// The §16 table this row comes from.
    #[must_use]
    pub const fn section(&self) -> MaturitySafetySection {
        self.section
    }

    /// The row's stable name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// What the row expects of the announcement.
    #[must_use]
    pub const fn polarity(&self) -> MaturitySafetyPolarity {
        self.polarity
    }

    /// The layer the row's change is applied at, if it changes anything.
    ///
    /// `None` for a positive row, which changes nothing at all.
    #[must_use]
    pub const fn mutation(&self) -> Option<MutationLayer> {
        self.mutation
    }

    /// Where the row expects its verdict to come from.
    #[must_use]
    pub const fn boundary(&self) -> MaturityRowBoundary {
        self.boundary
    }

    /// The named layer the row expects to produce its verdict.
    ///
    /// `None` for a row carrying a typed reason no layer answers it,
    /// which is the whole reason [`MaturityRowBoundary`] has more than
    /// one member.
    #[must_use]
    pub const fn refusing_layer(&self) -> Option<EvidenceBoundary> {
        self.boundary.layer()
    }

    /// The relation the row intends to violate, or the reason there is
    /// none.
    #[must_use]
    pub const fn relation(&self) -> &MaturityRelationStanding {
        &self.relation
    }

    /// The carrier the row intends to exercise.
    #[must_use]
    pub const fn carrier(&self) -> MaturityIntendedCarrier {
        self.carrier
    }

    /// The dependency collateral a linked row expects its requirement to
    /// demand.
    ///
    /// `None` for a row that names no relation, which has nothing to
    /// demand it of.
    #[must_use]
    pub const fn collateral(&self) -> Option<CollateralPolicy> {
        self.collateral
    }

    /// The accepted control this row's mutant departs from.
    #[must_use]
    pub const fn control(&self) -> MaturityCanonicalControl {
        self.control
    }

    /// Where the row's change sits.
    ///
    /// `None` exactly when the row changes nothing: a row with nowhere to
    /// point and nothing to point at would be describing an input that
    /// does not exist.
    #[must_use]
    pub const fn locator(&self) -> Option<MaturityMutationLocator> {
        self.locator
    }

    /// What the row expects the projection comparison to say.
    #[must_use]
    pub const fn projection(&self) -> MaturityExpectedProjection {
        self.projection
    }
}

impl core::fmt::Display for MaturitySafetyRow {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "§{}:{}", self.section.section(), self.name)
    }
}

/// Whether a layer's change can reach the stage that produces a verdict.
///
/// A validator refuses only what it is offered. A change made at a stage
/// the validator runs before never reaches it, so a row pairing such a
/// change with that boundary declares a verdict nothing can produce — and
/// a row whose boundary is unreachable is passed only by a verdict from
/// somewhere else, which is the confusion the declared boundary exists to
/// prevent. The arms below say, for each boundary, which changes the
/// stage behind it consumes.
#[must_use]
pub const fn boundary_admits(boundary: EvidenceBoundary, layer: MutationLayer) -> bool {
    match boundary {
        // The request validator and planner see the typed request and
        // predecessor world. The validated reports compare semantic facts
        // projected from accepted bytes; every row declaring either report
        // layer mutates that fact.
        // EvidenceBoundary::RootHistoryReportRejection | EvidenceBoundary::PublicRecoveryReportRejection
        EvidenceBoundary::SemanticRequestRejection
        | EvidenceBoundary::CompilerPlanRejection
        | EvidenceBoundary::RootHistoryReportRejection
        | EvidenceBoundary::PublicRecoveryReportRejection => {
            matches!(layer, MutationLayer::SemanticFact)
        }
        // Derivation is handed an admissible leaf schema and produces a
        // constructor from it; no linked program exists yet to disturb.
        EvidenceBoundary::ConstructorDerivationRejection => {
            matches!(layer, MutationLayer::StaticConstructorSchema)
        }
        // Emission reads the schema and the classification it is emitting
        // against.
        EvidenceBoundary::BackendEmissionRejection => matches!(
            layer,
            MutationLayer::StaticConstructorSchema | MutationLayer::AbiLayout
        ),
        // Linking consumes an already-derived constructor and produces a
        // program, so both are in front of it.
        EvidenceBoundary::LinkerRejection => matches!(
            layer,
            MutationLayer::StaticConstructorSchema | MutationLayer::LinkedConstructorProgram
        ),
        // The ABI and the constructor assemble the concrete transaction
        // and its witness from the linked program, so every artifact from
        // the layout down is theirs to refuse.
        EvidenceBoundary::AbiConstructionRejection => matches!(
            layer,
            MutationLayer::AbiLayout
                | MutationLayer::LinkedConstructorProgram
                | MutationLayer::TargetTransaction
                | MutationLayer::WitnessProof
        ),
        // The two boundaries no change reaches, for opposite reasons. A
        // failed environment is never evidence that a change was refused,
        // so no row declares it at all; an acceptance is what a row
        // expects when it changes nothing, so a row that both mutated and
        // expected acceptance would be a negative row in the positive
        // table.
        EvidenceBoundary::ExecutorInfrastructureFailure | EvidenceBoundary::AcceptedTransaction => {
            false
        }
        // Consensus checks the transaction's own fields before any script
        // runs; a witness it has not executed cannot have refused it.
        EvidenceBoundary::ConsensusRejectionBeforeScript => matches!(
            layer,
            MutationLayer::AbiLayout | MutationLayer::TargetTransaction
        ),
        // A key-path spend offers a signature against a committed key and
        // runs no script, so what it can refuse is the witness it was
        // given and the program that key commits to.
        EvidenceBoundary::KeyPathRejection => matches!(
            layer,
            MutationLayer::WitnessProof | MutationLayer::LinkedConstructorProgram
        ),
        // The covenant script reads the transaction it is spending, the
        // witness it was handed, the program it is, and the semantic
        // facts that program pins.
        EvidenceBoundary::ScriptPathRejection => matches!(
            layer,
            MutationLayer::SemanticFact
                | MutationLayer::LinkedConstructorProgram
                | MutationLayer::TargetTransaction
                | MutationLayer::WitnessProof
        ),
        // Relay policy measures the serialized transaction and its
        // witness before executing anything.
        EvidenceBoundary::RelayPolicyRejection => matches!(
            layer,
            MutationLayer::TargetTransaction | MutationLayer::WitnessProof
        ),
        // The projection comparison runs over an accepted transaction and
        // the semantic facts it should project, and over nothing else.
        EvidenceBoundary::ReportSemanticProjectionRejection => matches!(
            layer,
            MutationLayer::SemanticFact | MutationLayer::TargetTransaction
        ),
    }
}

use EvidenceBoundary as B;
use MaturityCanonicalControl as Control;
use MaturityExpectedProjection as Projection;
use MaturityIntendedCarrier as C;
use MaturityMutationClass as Class;
use MaturityMutationLocator as Loc;
use MaturityProjectionTerm as Term;
use MaturityRelationStanding as Standing;
use MaturityRowBoundary as Bound;
use MaturitySafetySection as S;
use MaturityUnlinkedReason as Why;
use MutationLayer as L;

/// One announce-maturity relation of the given kind and subject.
const fn relation(kind: RelationKind, subject: RelationSubject) -> RelationId {
    RelationId::new(OperationId::AnnounceMaturity, kind, subject)
}

/// One relation of the STATE object family on one side.
const fn state_family(kind: RelationKind, side: TransactionSide) -> RelationId {
    relation(
        kind,
        RelationSubject::ObjectFamily {
            side,
            object: ObjectId::State,
        },
    )
}

/// One relation of the optional sponsor object family on one side.
const fn sponsor_family(kind: RelationKind, side: TransactionSide) -> RelationId {
    relation(
        kind,
        RelationSubject::ObjectFamily {
            side,
            object: ObjectId::PlainLbtc,
        },
    )
}

/// The operation-wide operator authorization relation.
const fn authorization() -> RelationId {
    relation(RelationKind::Authorization, RelationSubject::Operation)
}

/// The input-side STATE recognition relation.
const fn input_recognition() -> RelationId {
    state_family(RelationKind::Recognition, TransactionSide::Input)
}

/// The output-side STATE recognition relation.
const fn output_recognition() -> RelationId {
    state_family(RelationKind::Recognition, TransactionSide::Output)
}

/// The input-side STATE cardinality relation.
const fn input_cardinality() -> RelationId {
    state_family(RelationKind::Cardinality, TransactionSide::Input)
}

/// The output-side STATE cardinality relation.
const fn output_cardinality() -> RelationId {
    state_family(RelationKind::Cardinality, TransactionSide::Output)
}

/// The input-side sponsor-family recognition relation.
const fn sponsor_recognition() -> RelationId {
    sponsor_family(RelationKind::Recognition, TransactionSide::Input)
}

/// The closed substrate asset's conservation relation.
const fn substrate_conservation() -> RelationId {
    relation(
        RelationKind::SubstrateConservation,
        RelationSubject::Asset {
            asset: AssetId::Lbtc,
        },
    )
}

/// The operation-wide root policy relation.
const fn root_policy() -> RelationId {
    relation(RelationKind::RootPolicy, RelationSubject::Operation)
}

/// The operation-wide projection policy relation.
const fn projection_policy() -> RelationId {
    relation(RelationKind::ProjectionPolicy, RelationSubject::Operation)
}

/// The operation-wide constructibility relation.
const fn constructibility() -> RelationId {
    relation(RelationKind::Constructibility, RelationSubject::Operation)
}

/// The STATE lifecycle relation for the announce-maturity exit.
const fn announcement_exit() -> RelationId {
    relation(
        RelationKind::Lifecycle,
        RelationSubject::LifecycleExit {
            object: ObjectId::State,
            exit: OperationId::AnnounceMaturity,
        },
    )
}

/// The row names this published relation and this class of it.
const fn names(id: RelationId, class: MaturityMutationClass) -> MaturityRelationStanding {
    Standing::Declared {
        relation: id,
        class,
    }
}

/// The row names no published relation, for a stated reason.
const fn unlinked(reason: MaturityUnlinkedReason) -> MaturityRelationStanding {
    Standing::Unlinked(reason)
}

/// The collateral a row's relation standing demands.
///
/// A row that names a relation expects that relation and its typed
/// dependency closure both reported blocked; a row that names none has
/// nothing to demand it of. The closure itself is not restated on the
/// row: it travels with the published requirement, and copying it here
/// would be a third source of truth that could agree with neither side.
const fn collateral_for(relation: &MaturityRelationStanding) -> Option<CollateralPolicy> {
    match relation {
        Standing::Declared { .. } => Some(CollateralPolicy::RequireIntendedAndDependencyClosure),
        Standing::EveryPublishedRelation | Standing::Unlinked(_) => None,
    }
}

/// The control a row of this table departs from.
///
/// Every fault table mutates the canonical sponsorless announcement
/// except the three whose subject is something else: a sponsor row needs
/// a sponsored control to have a sponsor at all, a protocol row departs
/// from a well-formed exchange, and a recovery row reads back a published
/// transaction rather than building one.
const fn control_of(section: MaturitySafetySection) -> MaturityCanonicalControl {
    match section {
        S::Positive => Control::TheRowIsTheControl,
        S::SponsorFault => Control::SponsoredAnnouncement,
        S::ProtocolReportFault => Control::CanonicalProtocolExchange,
        S::RecoveryFault => Control::AcceptedAnnouncementPublication,
        S::WindowFault
        | S::MetadataFault
        | S::OperatorFault
        | S::PredecessorConstructorFault
        | S::SuccessorConstructorFault
        | S::TotalityFault
        | S::RootHistoryFault
        | S::AbsenceFault
        | S::AbiLinkerFault => Control::SponsorlessAnnouncement,
    }
}

/// One §16.1 positive row.
///
/// A positive row changes nothing, so it carries no mutation layer and no
/// locator, it is its own control, and it expects every projection term
/// to match. What differs between positive rows is only the case they
/// exercise and whether anything can answer them today.
const fn positive(
    name: &'static str,
    boundary: MaturityRowBoundary,
    carrier: MaturityIntendedCarrier,
) -> MaturitySafetyRow {
    MaturitySafetyRow {
        section: S::Positive,
        name,
        polarity: MaturitySafetyPolarity::Positive,
        mutation: None,
        boundary,
        relation: Standing::EveryPublishedRelation,
        carrier,
        collateral: None,
        control: Control::TheRowIsTheControl,
        locator: None,
        projection: Projection::EveryTermMatches,
    }
}

/// One negative row refused before anything is projected.
const fn fault(
    section: MaturitySafetySection,
    name: &'static str,
    layer: MutationLayer,
    locator: MaturityMutationLocator,
    boundary: MaturityRowBoundary,
    carrier: MaturityIntendedCarrier,
    relation: MaturityRelationStanding,
) -> MaturitySafetyRow {
    let collateral = collateral_for(&relation);
    MaturitySafetyRow {
        section,
        name,
        polarity: MaturitySafetyPolarity::Negative,
        mutation: Some(layer),
        boundary,
        relation,
        carrier,
        collateral,
        control: control_of(section),
        locator: Some(locator),
        projection: Projection::NoProjection,
    }
}

/// One negative row whose boundary is this workspace's own canonical
/// serialization.
///
/// The announcement stands and the report layer answers: either a
/// projection term differs, or the serialization refuses to publish a
/// value at all. Both are properties of bytes this workspace renders, so
/// the carrier is the report and the control is a rendered one.
const fn report_fault(
    section: MaturitySafetySection,
    name: &'static str,
    layer: MutationLayer,
    locator: MaturityMutationLocator,
    relation: MaturityRelationStanding,
    projection: MaturityExpectedProjection,
) -> MaturitySafetyRow {
    let collateral = collateral_for(&relation);
    MaturitySafetyRow {
        section,
        name,
        polarity: MaturitySafetyPolarity::Negative,
        mutation: Some(layer),
        boundary: Bound::Layer(B::ReportSemanticProjectionRejection),
        relation,
        carrier: C::Report,
        collateral,
        control: Control::CanonicalSafetyReport,
        locator: Some(locator),
        projection,
    }
}

/// The complete §16 matrix, in the guide's order.
///
/// The rows stand table by table and, inside a table, in the order §16
/// lists them, so a row's position in its table is its ordinal and a
/// reader can walk the guide and this array side by side.
pub const MATURITY_SAFETY_ROWS: &[MaturitySafetyRow] = &[
    // §16.1 — the fourteen positive cases.
    //
    // Eleven of them ask for an acceptance. The admitted accepted run
    // answers the sponsorless case under the variable schedule. Its
    // declaration here remains the relay-policy layer of the whole
    // schedule; the schedule-indexed boundary rule maps the case for
    // both schedules. Naming the sponsorless row's layer while the
    // other ten state their reason is not an inconsistency — it
    // distinguishes a row answered by an admitted run from rows whose
    // acceptance remains outstanding.
    positive(
        "minimum-valid-lead",
        Bound::AcceptanceAwaitsRelayAdmissibility,
        C::AnnouncementLeaf,
    ),
    positive(
        "maximum-valid-lead",
        Bound::AcceptanceAwaitsRelayAdmissibility,
        C::AnnouncementLeaf,
    ),
    positive(
        "representative-interior-lead",
        Bound::AcceptanceAwaitsRelayAdmissibility,
        C::AnnouncementLeaf,
    ),
    positive(
        "smallest-current-cycle",
        Bound::AcceptanceAwaitsRelayAdmissibility,
        C::AnnouncementLeaf,
    ),
    positive(
        "current-cycle-near-checked-upper-domain",
        Bound::AcceptanceAwaitsRelayAdmissibility,
        C::AnnouncementLeaf,
    ),
    positive(
        "nontrivial-unaffected-fields",
        Bound::AcceptanceAwaitsRelayAdmissibility,
        C::AnnouncementLeaf,
    ),
    positive(
        "representation-nonce-zero",
        Bound::AcceptanceAwaitsRelayAdmissibility,
        C::LinkedConstructor,
    ),
    positive(
        "representation-nonce-nonzero",
        Bound::AcceptanceAwaitsRelayAdmissibility,
        C::LinkedConstructor,
    ),
    // The one §16.1 row a run answers at this tip. The chain from linked
    // bytes through the static subtree, the output key and the control
    // block is exercised whole, and the spend is stopped by a
    // standardness width measured before execution and by nothing else.
    positive(
        "sponsorless",
        Bound::Layer(B::RelayPolicyRejection),
        C::Target,
    ),
    positive(
        "sponsored-without-change",
        Bound::AcceptanceAwaitsRelayAdmissibility,
        C::CoordinatorStructure,
    ),
    positive(
        "sponsored-with-change-where-supported",
        Bound::AcceptanceAwaitsRelayAdmissibility,
        C::CoordinatorStructure,
    ),
    positive(
        "public-successor-recovery",
        Bound::Layer(B::PublicRecoveryReportRejection),
        C::Report,
    ),
    positive(
        "repeated-equal-construction-producing-equal-candidate-bytes",
        Bound::PropertyOfTheBuildRatherThanAFixture,
        C::LinkedConstructor,
    ),
    positive(
        "accepted-target-transaction-with-semantic-constructor-root-and-recovery-projections-all-matching",
        Bound::AcceptanceAwaitsRelayAdmissibility,
        C::Target,
    ),
    // §16.2 — the eleven maturity-window faults.
    //
    // The lead window is a symbolic law of the requirement layer rather
    // than a member of the operation's relation census, so most of these
    // rows name no published relation and say so. The two that turn on
    // the predecessor's status, and the two that turn on the successor's,
    // do name one: whether this exit is admissible from this state is
    // exactly what the lifecycle relation decides.
    fault(
        S::WindowFault,
        "predecessor-already-announced",
        L::SemanticFact,
        Loc::PredecessorMetadataField,
        Bound::Layer(B::SemanticRequestRejection),
        C::AnnouncementLeaf,
        names(announcement_exit(), Class::RequiredExitMissingFromThePlan),
    ),
    fault(
        S::WindowFault,
        "predecessor-complete",
        L::SemanticFact,
        Loc::PredecessorMetadataField,
        Bound::Layer(B::SemanticRequestRejection),
        C::AnnouncementLeaf,
        names(announcement_exit(), Class::RequiredExitMissingFromThePlan),
    ),
    fault(
        S::WindowFault,
        "one-below-minimum",
        L::SemanticFact,
        Loc::SemanticRequestField,
        Bound::Layer(B::SemanticRequestRejection),
        C::AnnouncementLeaf,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::WindowFault,
        "one-above-maximum",
        L::SemanticFact,
        Loc::SemanticRequestField,
        Bound::Layer(B::SemanticRequestRejection),
        C::AnnouncementLeaf,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::WindowFault,
        "equal-to-current-cycle-where-outside-the-lead",
        L::SemanticFact,
        Loc::SemanticRequestField,
        Bound::Layer(B::SemanticRequestRejection),
        C::AnnouncementLeaf,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::WindowFault,
        "lower-bound-addition-overflow",
        L::SemanticFact,
        Loc::SemanticRequestField,
        Bound::Layer(B::SemanticRequestRejection),
        C::AnnouncementLeaf,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::WindowFault,
        "upper-bound-addition-overflow",
        L::SemanticFact,
        Loc::SemanticRequestField,
        Bound::Layer(B::SemanticRequestRejection),
        C::AnnouncementLeaf,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::WindowFault,
        "malformed-cycle-encoding",
        L::SemanticFact,
        Loc::SemanticRequestField,
        Bound::Layer(B::SemanticRequestRejection),
        C::AnnouncementLeaf,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::WindowFault,
        "request-cycle-differs-from-successor",
        L::SemanticFact,
        Loc::SemanticRequestField,
        Bound::Layer(B::SemanticRequestRejection),
        C::AnnouncementLeaf,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::WindowFault,
        "successor-remains-unannounced",
        L::SemanticFact,
        Loc::SuccessorMetadataField,
        Bound::Layer(B::ScriptPathRejection),
        C::AnnouncementLeaf,
        names(announcement_exit(), Class::RequiredExitMissingFromThePlan),
    ),
    fault(
        S::WindowFault,
        "successor-becomes-complete",
        L::SemanticFact,
        Loc::SuccessorMetadataField,
        Bound::Layer(B::ScriptPathRejection),
        C::AnnouncementLeaf,
        names(announcement_exit(), Class::RequiredExitMissingFromThePlan),
    ),
    // §16.3 — eleven listed faults and one row for each semantic field
    // the announcement leaves unaffected.
    //
    // The metadata record carries six semantic fields and the transition
    // moves the maturity status alone, copying the other five through, so
    // the table's per-field family is five rows and not one.
    //
    // Those five resolve §16.3's disjunction — refusal or report-layer
    // semantic mismatch — to the report layer definitely, and they say
    // why: a change to a field the transition copies leaves the maturity
    // transition correct and the constructor shape valid, so nothing
    // upstream has a reason to refuse and the candidate reaches the
    // target intact. What catches it is the projection comparison. A row
    // that declared both halves of the disjunction could be passed by
    // either of two layers, and a row two layers can pass is a row
    // neither was asked.
    //
    // The fault they stage belongs to the canonical-delta policy
    // relation: the announcement's expected canonical delta is empty
    // because the transition moves the maturity status and copies every
    // other semantic quantity through, so a change to a copied quantity
    // is a fault against that policy rather than against a field. That
    // relation compares the whole observed transaction's partition, so
    // these rows state the reason and name no requirement.
    report_fault(
        S::MetadataFault,
        "unaffected-field-omega-changed-alone",
        L::SemanticFact,
        Loc::SuccessorMetadataField,
        unlinked(Why::RelationScopedToTheWholeTransaction),
        Projection::TermDiffers(Term::Semantic),
    ),
    report_fault(
        S::MetadataFault,
        "unaffected-field-y-l-changed-alone",
        L::SemanticFact,
        Loc::SuccessorMetadataField,
        unlinked(Why::RelationScopedToTheWholeTransaction),
        Projection::TermDiffers(Term::Semantic),
    ),
    report_fault(
        S::MetadataFault,
        "unaffected-field-y-t-changed-alone",
        L::SemanticFact,
        Loc::SuccessorMetadataField,
        unlinked(Why::RelationScopedToTheWholeTransaction),
        Projection::TermDiffers(Term::Semantic),
    ),
    report_fault(
        S::MetadataFault,
        "unaffected-field-q-changed-alone",
        L::SemanticFact,
        Loc::SuccessorMetadataField,
        unlinked(Why::RelationScopedToTheWholeTransaction),
        Projection::TermDiffers(Term::Semantic),
    ),
    report_fault(
        S::MetadataFault,
        "unaffected-field-cycle-changed-alone",
        L::SemanticFact,
        Loc::SuccessorMetadataField,
        unlinked(Why::RelationScopedToTheWholeTransaction),
        Projection::TermDiffers(Term::Semantic),
    ),
    fault(
        S::MetadataFault,
        "omit-one-field",
        L::WitnessProof,
        Loc::MetadataEncoding,
        Bound::Layer(B::AbiConstructionRejection),
        C::AnnouncementLeaf,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::MetadataFault,
        "duplicate-one-field",
        L::WitnessProof,
        Loc::MetadataEncoding,
        Bound::Layer(B::AbiConstructionRejection),
        C::AnnouncementLeaf,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::MetadataFault,
        "reorder-fields",
        L::WitnessProof,
        Loc::MetadataEncoding,
        Bound::Layer(B::AbiConstructionRejection),
        C::AnnouncementLeaf,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::MetadataFault,
        "unknown-schema",
        L::WitnessProof,
        Loc::MetadataEncoding,
        Bound::Layer(B::AbiConstructionRejection),
        C::AnnouncementLeaf,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::MetadataFault,
        "wrong-domain-separator",
        L::WitnessProof,
        Loc::MetadataEncoding,
        Bound::Layer(B::AbiConstructionRejection),
        C::AnnouncementLeaf,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::MetadataFault,
        "noncanonical-enum-tag",
        L::WitnessProof,
        Loc::MetadataEncoding,
        Bound::Layer(B::AbiConstructionRejection),
        C::AnnouncementLeaf,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::MetadataFault,
        "nonzero-reserved-field",
        L::WitnessProof,
        Loc::MetadataEncoding,
        Bound::Layer(B::AbiConstructionRejection),
        C::AnnouncementLeaf,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::MetadataFault,
        "trailing-bytes",
        L::WitnessProof,
        Loc::MetadataEncoding,
        Bound::Layer(B::AbiConstructionRejection),
        C::AnnouncementLeaf,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::MetadataFault,
        "metadata-from-another-state-object",
        L::WitnessProof,
        Loc::PredecessorMetadataField,
        Bound::Layer(B::ScriptPathRejection),
        C::AnnouncementLeaf,
        names(input_recognition(), Class::WrongRecognizedObject),
    ),
    fault(
        S::MetadataFault,
        "correct-semantic-metadata-with-noncanonical-representation-nonce",
        L::WitnessProof,
        Loc::MetadataEncoding,
        Bound::Layer(B::AbiConstructionRejection),
        C::LinkedConstructor,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    // A compensated pair of changes leaves every upstream shape valid and
    // every field individually admissible, so it reaches the target for
    // the same reason the per-field rows do and is answered where they
    // are.
    report_fault(
        S::MetadataFault,
        "semantic-field-changes-compensated-by-another-field",
        L::SemanticFact,
        Loc::SuccessorMetadataField,
        unlinked(Why::RelationScopedToTheWholeTransaction),
        Projection::TermDiffers(Term::Semantic),
    ),
    // §16.4 — the eighteen operator faults.
    //
    // Every one of them is a row against the operation's authorization
    // relation: what each changes is which key authorized what, or
    // whether the authorization still binds the bytes it was given. The
    // rows split by where the change can first be seen — a response the
    // signing flow can refuse to assemble, against an offering the leaf's
    // own verification is required to reject.
    fault(
        S::OperatorFault,
        "missing-operator-signature",
        L::WitnessProof,
        Loc::OperatorSigningResponse,
        Bound::Layer(B::AbiConstructionRejection),
        C::AnnouncementLeaf,
        names(authorization(), Class::ExternalReportMissing),
    ),
    fault(
        S::OperatorFault,
        "empty-signature",
        L::WitnessProof,
        Loc::OperatorSigningResponse,
        Bound::Layer(B::ScriptPathRejection),
        C::AnnouncementLeaf,
        names(authorization(), Class::ExternalReportFailed),
    ),
    fault(
        S::OperatorFault,
        "malformed-signature",
        L::WitnessProof,
        Loc::OperatorSigningResponse,
        Bound::Layer(B::ScriptPathRejection),
        C::AnnouncementLeaf,
        names(authorization(), Class::ExternalReportFailed),
    ),
    fault(
        S::OperatorFault,
        "wrong-operator",
        L::WitnessProof,
        Loc::OperatorSigningResponse,
        Bound::Layer(B::ScriptPathRejection),
        C::AnnouncementLeaf,
        names(authorization(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::OperatorFault,
        "stale-operator",
        L::WitnessProof,
        Loc::ApprovedOperatorKey,
        Bound::Layer(B::ScriptPathRejection),
        C::AnnouncementLeaf,
        names(authorization(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::OperatorFault,
        "unknown-key-encoding",
        L::WitnessProof,
        Loc::ApprovedOperatorKey,
        Bound::Layer(B::AbiConstructionRejection),
        C::AnnouncementLeaf,
        names(authorization(), Class::ExternalReportFailed),
    ),
    fault(
        S::OperatorFault,
        "malformed-approved-key",
        L::WitnessProof,
        Loc::ApprovedOperatorKey,
        Bound::Layer(B::AbiConstructionRejection),
        C::AnnouncementLeaf,
        names(authorization(), Class::ExternalReportFailed),
    ),
    fault(
        S::OperatorFault,
        "valid-signature-under-another-key",
        L::WitnessProof,
        Loc::OperatorSigningResponse,
        Bound::Layer(B::ScriptPathRejection),
        C::AnnouncementLeaf,
        names(authorization(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::OperatorFault,
        "valid-signature-over-another-candidate",
        L::WitnessProof,
        Loc::OperatorSigningResponse,
        Bound::Layer(B::ScriptPathRejection),
        C::AnnouncementLeaf,
        names(authorization(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::OperatorFault,
        "wrong-profile",
        L::WitnessProof,
        Loc::OperatorSigningResponse,
        Bound::Layer(B::AbiConstructionRejection),
        C::AnnouncementLeaf,
        names(authorization(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::OperatorFault,
        "wrong-input",
        L::WitnessProof,
        Loc::OperatorSigningResponse,
        Bound::Layer(B::AbiConstructionRejection),
        C::AnnouncementLeaf,
        names(authorization(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::OperatorFault,
        "wrong-leaf",
        L::WitnessProof,
        Loc::OperatorSigningResponse,
        Bound::Layer(B::AbiConstructionRejection),
        C::AnnouncementLeaf,
        names(authorization(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::OperatorFault,
        "duplicate-response",
        L::WitnessProof,
        Loc::OperatorSigningResponse,
        Bound::Layer(B::AbiConstructionRejection),
        C::AnnouncementLeaf,
        names(authorization(), Class::ExternalReportFailed),
    ),
    fault(
        S::OperatorFault,
        "unexpected-response",
        L::WitnessProof,
        Loc::OperatorSigningResponse,
        Bound::Layer(B::AbiConstructionRejection),
        C::AnnouncementLeaf,
        names(authorization(), Class::ExternalReportFailed),
    ),
    fault(
        S::OperatorFault,
        "successor-metadata-changed-after-signing",
        L::TargetTransaction,
        Loc::TransactionOutput,
        Bound::Layer(B::ScriptPathRejection),
        C::AnnouncementLeaf,
        names(authorization(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::OperatorFault,
        "successor-program-changed-after-signing",
        L::TargetTransaction,
        Loc::TransactionOutput,
        Bound::Layer(B::ScriptPathRejection),
        C::AnnouncementLeaf,
        names(authorization(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::OperatorFault,
        "sponsor-input-added-after-signing",
        L::TargetTransaction,
        Loc::TransactionInput,
        Bound::Layer(B::ScriptPathRejection),
        C::AnnouncementLeaf,
        names(authorization(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::OperatorFault,
        "fee-output-changed-after-signing",
        L::TargetTransaction,
        Loc::TransactionOutput,
        Bound::Layer(B::ScriptPathRejection),
        C::AnnouncementLeaf,
        names(authorization(), Class::ExternalReportSubjectMismatch),
    ),
    // §16.5 — the fifteen predecessor-constructor faults.
    //
    // The first three change what the spent object is, which is the
    // input side's recognition question. The rest change how it was
    // built, which is the operation's constructibility question, and
    // they split by the stage that first sees the change: a leaf schema
    // is refused where a constructor is derived, a program from another
    // bundle where linking binds one, and a control path only where a
    // target walks it.
    fault(
        S::PredecessorConstructorFault,
        "wrong-state-asset",
        L::TargetTransaction,
        Loc::TransactionInput,
        Bound::Layer(B::ScriptPathRejection),
        C::AnnouncementLeaf,
        names(input_recognition(), Class::WrongRecognizedAsset),
    ),
    fault(
        S::PredecessorConstructorFault,
        "wrong-singleton-amount",
        L::TargetTransaction,
        Loc::TransactionInput,
        Bound::Layer(B::ScriptPathRejection),
        C::AnnouncementLeaf,
        names(input_recognition(), Class::WrongRecognizedObject),
    ),
    fault(
        S::PredecessorConstructorFault,
        "wrong-predecessor-program",
        L::LinkedConstructorProgram,
        Loc::LinkedProgram,
        Bound::Layer(B::ScriptPathRejection),
        C::LinkedConstructor,
        names(input_recognition(), Class::WrongRecognizedObject),
    ),
    fault(
        S::PredecessorConstructorFault,
        "predecessor-metadata-reconstructs-another-program",
        L::WitnessProof,
        Loc::PredecessorMetadataField,
        Bound::Layer(B::ScriptPathRejection),
        C::AnnouncementLeaf,
        names(constructibility(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::PredecessorConstructorFault,
        "wrong-static-subtree",
        L::StaticConstructorSchema,
        Loc::StaticSubtreeLeaf,
        Bound::Layer(B::ConstructorDerivationRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::PredecessorConstructorFault,
        "wrong-leaf-version",
        L::LinkedConstructorProgram,
        Loc::ControlBlock,
        Bound::Layer(B::ScriptPathRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportFailed),
    ),
    fault(
        S::PredecessorConstructorFault,
        "wrong-internal-key",
        L::LinkedConstructorProgram,
        Loc::ControlBlock,
        Bound::Layer(B::ScriptPathRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::PredecessorConstructorFault,
        "wrong-control-block",
        L::WitnessProof,
        Loc::ControlBlock,
        Bound::Layer(B::ScriptPathRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportFailed),
    ),
    fault(
        S::PredecessorConstructorFault,
        "stale-constructor-from-another-bundle",
        L::LinkedConstructorProgram,
        Loc::LinkedProgram,
        Bound::Layer(B::LinkerRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::PredecessorConstructorFault,
        "metadata-leaf-missing",
        L::StaticConstructorSchema,
        Loc::StaticSubtreeLeaf,
        Bound::Layer(B::ConstructorDerivationRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportMissing),
    ),
    fault(
        S::PredecessorConstructorFault,
        "metadata-leaf-duplicated",
        L::StaticConstructorSchema,
        Loc::StaticSubtreeLeaf,
        Bound::Layer(B::ConstructorDerivationRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::PredecessorConstructorFault,
        "operation-leaf-missing",
        L::StaticConstructorSchema,
        Loc::StaticSubtreeLeaf,
        Bound::Layer(B::ConstructorDerivationRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportMissing),
    ),
    fault(
        S::PredecessorConstructorFault,
        "extra-escape-leaf",
        L::StaticConstructorSchema,
        Loc::StaticSubtreeLeaf,
        Bound::Layer(B::ConstructorDerivationRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::PredecessorConstructorFault,
        "metadata-leaf-selected-for-execution",
        L::WitnessProof,
        Loc::ControlBlock,
        Bound::Layer(B::ScriptPathRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportFailed),
    ),
    // A key-path spend runs no script at all: it offers a signature with
    // no leaf and no control block, so no covenant clause is reached and
    // the refusal belongs to the key path rather than to the script path
    // it never entered.
    fault(
        S::PredecessorConstructorFault,
        "key-path-spend-attempt",
        L::WitnessProof,
        Loc::WitnessStack,
        Bound::Layer(B::KeyPathRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportFailed),
    ),
    // §16.6 — the fifteen successor-constructor faults.
    //
    // Three of them change what the successor means, three change how
    // many STATE outputs there are, and the rest change how the
    // successor was built. The two nonce rows name no published
    // relation: the operation's representation relation fixes which
    // representation modes are admitted and says nothing about which
    // admissible nonce a search must settle on.
    fault(
        S::SuccessorConstructorFault,
        "successor-from-wrong-semantic-metadata",
        L::SemanticFact,
        Loc::SuccessorMetadataField,
        Bound::Layer(B::ScriptPathRejection),
        C::AnnouncementLeaf,
        names(output_recognition(), Class::WrongRecognizedObject),
    ),
    fault(
        S::SuccessorConstructorFault,
        "successor-from-predecessor-metadata-unchanged",
        L::SemanticFact,
        Loc::SuccessorMetadataField,
        Bound::Layer(B::ScriptPathRejection),
        C::AnnouncementLeaf,
        names(output_recognition(), Class::WrongRecognizedObject),
    ),
    fault(
        S::SuccessorConstructorFault,
        "wrong-announcement-cycle",
        L::SemanticFact,
        Loc::SuccessorMetadataField,
        Bound::Layer(B::ScriptPathRejection),
        C::AnnouncementLeaf,
        names(output_recognition(), Class::WrongRecognizedObject),
    ),
    fault(
        S::SuccessorConstructorFault,
        "wrong-representation-nonce",
        L::LinkedConstructorProgram,
        Loc::NonceSearch,
        Bound::Layer(B::ScriptPathRejection),
        C::LinkedConstructor,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::SuccessorConstructorFault,
        "later-admissible-nonce-instead-of-first",
        L::LinkedConstructorProgram,
        Loc::NonceSearch,
        Bound::Layer(B::ScriptPathRejection),
        C::LinkedConstructor,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::SuccessorConstructorFault,
        "successor-under-another-static-subtree",
        L::LinkedConstructorProgram,
        Loc::LinkedProgram,
        Bound::Layer(B::ScriptPathRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::SuccessorConstructorFault,
        "wrong-internal-key",
        L::LinkedConstructorProgram,
        Loc::ControlBlock,
        Bound::Layer(B::ScriptPathRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::SuccessorConstructorFault,
        "wrong-parity",
        L::LinkedConstructorProgram,
        Loc::ControlBlock,
        Bound::Layer(B::ScriptPathRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportFailed),
    ),
    fault(
        S::SuccessorConstructorFault,
        "wrong-leaf-version",
        L::LinkedConstructorProgram,
        Loc::ControlBlock,
        Bound::Layer(B::ScriptPathRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportFailed),
    ),
    fault(
        S::SuccessorConstructorFault,
        "wrong-control-recipe",
        L::LinkedConstructorProgram,
        Loc::ControlBlock,
        Bound::Layer(B::ScriptPathRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportFailed),
    ),
    fault(
        S::SuccessorConstructorFault,
        "arbitrary-caller-supplied-output-program",
        L::LinkedConstructorProgram,
        Loc::LinkedProgram,
        Bound::Layer(B::ScriptPathRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::SuccessorConstructorFault,
        "no-state-successor",
        L::TargetTransaction,
        Loc::TransactionOutput,
        Bound::Layer(B::ScriptPathRejection),
        C::CoordinatorStructure,
        names(output_cardinality(), Class::CardinalityBelowMinimum),
    ),
    fault(
        S::SuccessorConstructorFault,
        "two-state-successors",
        L::TargetTransaction,
        Loc::TransactionOutput,
        Bound::Layer(B::ScriptPathRejection),
        C::CoordinatorStructure,
        names(output_cardinality(), Class::CardinalityAboveMaximum),
    ),
    fault(
        S::SuccessorConstructorFault,
        "extra-state-like-output",
        L::TargetTransaction,
        Loc::TransactionOutput,
        Bound::Layer(B::ScriptPathRejection),
        C::CoordinatorStructure,
        names(output_cardinality(), Class::CardinalityAboveMaximum),
    ),
    fault(
        S::SuccessorConstructorFault,
        "spendable-metadata-leaf",
        L::StaticConstructorSchema,
        Loc::StaticSubtreeLeaf,
        Bound::Layer(B::ConstructorDerivationRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportSubjectMismatch),
    ),
    // §16.7 — the fourteen branch-order and totality faults.
    //
    // The first three are branch-order rows, and a derivation answers
    // them. The eleven that follow are the nonce and tweak rows, whose
    // evidence landed with its own oracle and its own negatives: the
    // admissible-nonce search, the retry discipline around it, and the
    // tweak arithmetic over the internal key were established there.
    // Restating them as outstanding would record one obligation twice.
    fault(
        S::TotalityFault,
        "metadata-child-on-wrong-side",
        L::StaticConstructorSchema,
        Loc::BranchOrder,
        Bound::Layer(B::ConstructorDerivationRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::TotalityFault,
        "caller-supplied-branch-order",
        L::StaticConstructorSchema,
        Loc::BranchOrder,
        Bound::Layer(B::ConstructorDerivationRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::TotalityFault,
        "source-order-dependent-tree",
        L::StaticConstructorSchema,
        Loc::BranchOrder,
        Bound::Layer(B::ConstructorDerivationRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::TotalityFault,
        "nonce-starts-at-one",
        L::LinkedConstructorProgram,
        Loc::NonceSearch,
        Bound::TotalityAnsweredByLandedNonceEvidence,
        C::LinkedConstructor,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::TotalityFault,
        "nonce-skips-an-admissible-value",
        L::LinkedConstructorProgram,
        Loc::NonceSearch,
        Bound::TotalityAnsweredByLandedNonceEvidence,
        C::LinkedConstructor,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::TotalityFault,
        "nonce-search-exceeds-bound",
        L::LinkedConstructorProgram,
        Loc::NonceSearch,
        Bound::TotalityAnsweredByLandedNonceEvidence,
        C::LinkedConstructor,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::TotalityFault,
        "invalid-internal-key-retried",
        L::LinkedConstructorProgram,
        Loc::NonceSearch,
        Bound::TotalityAnsweredByLandedNonceEvidence,
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportMissing),
    ),
    fault(
        S::TotalityFault,
        "malformed-metadata-retried",
        L::LinkedConstructorProgram,
        Loc::NonceSearch,
        Bound::TotalityAnsweredByLandedNonceEvidence,
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportMissing),
    ),
    fault(
        S::TotalityFault,
        "missing-leaf-retried",
        L::LinkedConstructorProgram,
        Loc::NonceSearch,
        Bound::TotalityAnsweredByLandedNonceEvidence,
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportMissing),
    ),
    fault(
        S::TotalityFault,
        "zero-tweak-rejected-merely-for-zero",
        L::LinkedConstructorProgram,
        Loc::TweakArithmetic,
        Bound::TotalityAnsweredByLandedNonceEvidence,
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportMissing),
    ),
    fault(
        S::TotalityFault,
        "tweak-at-or-above-group-order-accepted",
        L::LinkedConstructorProgram,
        Loc::TweakArithmetic,
        Bound::TotalityAnsweredByLandedNonceEvidence,
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportMissing),
    ),
    fault(
        S::TotalityFault,
        "identity-result-accepted",
        L::LinkedConstructorProgram,
        Loc::TweakArithmetic,
        Bound::TotalityAnsweredByLandedNonceEvidence,
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportMissing),
    ),
    fault(
        S::TotalityFault,
        "target-and-oracle-parity-disagree",
        L::LinkedConstructorProgram,
        Loc::TweakArithmetic,
        Bound::TotalityAnsweredByLandedNonceEvidence,
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::TotalityFault,
        "repeated-hashing-used-as-fixed-point-search",
        L::LinkedConstructorProgram,
        Loc::NonceSearch,
        Bound::TotalityAnsweredByLandedNonceEvidence,
        C::LinkedConstructor,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    // §16.8 — the sixteen root-history faults.
    //
    // Every row against the operation's root policy is answered by the
    // validated root-history continuity report over the accepted
    // announcement. For stale-predecessor and wrong-current-root-view,
    // validation compares the predecessor with the stated cursor before
    // the edge. Chain currency remains the report's freshness residual.
    fault(
        S::RootHistoryFault,
        "stale-predecessor",
        L::SemanticFact,
        Loc::RootHistoryEdge,
        Bound::Layer(B::RootHistoryReportRejection),
        C::Report,
        names(root_policy(), Class::WrongRootEffect),
    ),
    fault(
        S::RootHistoryFault,
        "wrong-current-root-view",
        L::SemanticFact,
        Loc::RootHistoryEdge,
        Bound::Layer(B::RootHistoryReportRejection),
        C::Report,
        names(root_policy(), Class::WrongRootEffect),
    ),
    fault(
        S::RootHistoryFault,
        "two-predecessors",
        L::SemanticFact,
        Loc::RootHistoryEdge,
        Bound::Layer(B::RootHistoryReportRejection),
        C::Report,
        names(root_policy(), Class::WrongRootEffect),
    ),
    fault(
        S::RootHistoryFault,
        "two-successors",
        L::SemanticFact,
        Loc::RootHistoryEdge,
        Bound::Layer(B::RootHistoryReportRejection),
        C::Report,
        names(root_policy(), Class::WrongRootEffect),
    ),
    fault(
        S::RootHistoryFault,
        "missing-edge",
        L::SemanticFact,
        Loc::RootHistoryEdge,
        Bound::Layer(B::RootHistoryReportRejection),
        C::Report,
        names(root_policy(), Class::WrongRootEffect),
    ),
    fault(
        S::RootHistoryFault,
        "duplicate-edge",
        L::SemanticFact,
        Loc::RootHistoryEdge,
        Bound::Layer(B::RootHistoryReportRejection),
        C::Report,
        names(root_policy(), Class::WrongRootEffect),
    ),
    fault(
        S::RootHistoryFault,
        "successor-cursor-restored-after-invalid-intermediate-edge",
        L::SemanticFact,
        Loc::RootHistoryEdge,
        Bound::Layer(B::RootHistoryReportRejection),
        C::Report,
        names(root_policy(), Class::WrongRootEffect),
    ),
    fault(
        S::RootHistoryFault,
        "state-termination",
        L::SemanticFact,
        Loc::RootHistoryEdge,
        Bound::Layer(B::RootHistoryReportRejection),
        C::Report,
        names(root_policy(), Class::WrongRootEffect),
    ),
    fault(
        S::RootHistoryFault,
        "old-static-subtree-to-new-subtree-without-migration",
        L::SemanticFact,
        Loc::RootHistoryEdge,
        Bound::Layer(B::RootHistoryReportRejection),
        C::Report,
        names(root_policy(), Class::WrongRootEffect),
    ),
    fault(
        S::RootHistoryFault,
        "unrelated-state-shaped-input",
        L::SemanticFact,
        Loc::RootHistoryEdge,
        Bound::Layer(B::RootHistoryReportRejection),
        C::Report,
        names(root_policy(), Class::WrongRootEffect),
    ),
    fault(
        S::RootHistoryFault,
        "root-cursor-points-to-sponsor-change",
        L::SemanticFact,
        Loc::RootHistoryEdge,
        Bound::Layer(B::RootHistoryReportRejection),
        C::Report,
        names(root_policy(), Class::WrongRootEffect),
    ),
    fault(
        S::RootHistoryFault,
        "resv-edge",
        L::SemanticFact,
        Loc::RootHistoryEdge,
        Bound::Layer(B::RootHistoryReportRejection),
        C::Report,
        names(root_policy(), Class::WrongRootEffect),
    ),
    fault(
        S::RootHistoryFault,
        "pace-edge",
        L::SemanticFact,
        Loc::RootHistoryEdge,
        Bound::Layer(B::RootHistoryReportRejection),
        C::Report,
        names(root_policy(), Class::WrongRootEffect),
    ),
    fault(
        S::RootHistoryFault,
        "authority-edge",
        L::SemanticFact,
        Loc::RootHistoryEdge,
        Bound::Layer(B::RootHistoryReportRejection),
        C::Report,
        names(root_policy(), Class::WrongRootEffect),
    ),
    fault(
        S::RootHistoryFault,
        "transition-certificate-names-another-predecessor",
        L::SemanticFact,
        Loc::RootHistoryEdge,
        Bound::Layer(B::RootHistoryReportRejection),
        C::Report,
        names(root_policy(), Class::WrongRootEffect),
    ),
    fault(
        S::RootHistoryFault,
        "transition-certificate-names-another-successor",
        L::SemanticFact,
        Loc::RootHistoryEdge,
        Bound::Layer(B::RootHistoryReportRejection),
        C::Report,
        names(root_policy(), Class::WrongRootEffect),
    ),
    // §16.9 — the fifteen absence and economic faults.
    //
    // Nine of them add a member of a family the operation admits on
    // neither side, and they name no single relation: the closure is
    // published per transaction side and the table names the family
    // without naming a side, so filing such a row under one side would
    // file it under half its own subject. The leaf does not answer these
    // rows either — a spend carrying further inputs and outputs of
    // another asset at positions the leaf never names is accepted by the
    // leaf — so what refuses them is the classification the ABI builds
    // before a target sees anything.
    fault(
        S::AbsenceFault,
        "add-live-receipt-input-or-output",
        L::AbiLayout,
        Loc::AdmittedObjectFamily,
        Bound::Layer(B::AbiConstructionRejection),
        C::CoordinatorStructure,
        unlinked(Why::SeveralPublishedRelationsFit),
    ),
    fault(
        S::AbsenceFault,
        "add-time-locked-receipt",
        L::AbiLayout,
        Loc::AdmittedObjectFamily,
        Bound::Layer(B::AbiConstructionRejection),
        C::CoordinatorStructure,
        unlinked(Why::SeveralPublishedRelationsFit),
    ),
    fault(
        S::AbsenceFault,
        "add-ash",
        L::AbiLayout,
        Loc::AdmittedObjectFamily,
        Bound::Layer(B::AbiConstructionRejection),
        C::CoordinatorStructure,
        unlinked(Why::SeveralPublishedRelationsFit),
    ),
    fault(
        S::AbsenceFault,
        "add-entitlement",
        L::AbiLayout,
        Loc::AdmittedObjectFamily,
        Bound::Layer(B::AbiConstructionRejection),
        C::CoordinatorStructure,
        unlinked(Why::SeveralPublishedRelationsFit),
    ),
    fault(
        S::AbsenceFault,
        "add-request",
        L::AbiLayout,
        Loc::AdmittedObjectFamily,
        Bound::Layer(B::AbiConstructionRejection),
        C::CoordinatorStructure,
        unlinked(Why::SeveralPublishedRelationsFit),
    ),
    fault(
        S::AbsenceFault,
        "add-distribution-control-or-vault",
        L::AbiLayout,
        Loc::AdmittedObjectFamily,
        Bound::Layer(B::AbiConstructionRejection),
        C::CoordinatorStructure,
        unlinked(Why::SeveralPublishedRelationsFit),
    ),
    fault(
        S::AbsenceFault,
        "add-resv-input-or-output",
        L::AbiLayout,
        Loc::AdmittedObjectFamily,
        Bound::Layer(B::AbiConstructionRejection),
        C::CoordinatorStructure,
        unlinked(Why::SeveralPublishedRelationsFit),
    ),
    fault(
        S::AbsenceFault,
        "add-pace",
        L::AbiLayout,
        Loc::AdmittedObjectFamily,
        Bound::Layer(B::AbiConstructionRejection),
        C::CoordinatorStructure,
        unlinked(Why::SeveralPublishedRelationsFit),
    ),
    fault(
        S::AbsenceFault,
        "add-authority-object",
        L::AbiLayout,
        Loc::AdmittedObjectFamily,
        Bound::Layer(B::AbiConstructionRejection),
        C::CoordinatorStructure,
        unlinked(Why::SeveralPublishedRelationsFit),
    ),
    fault(
        S::AbsenceFault,
        "add-issuance",
        L::AbiLayout,
        Loc::TransactionField,
        Bound::Layer(B::AbiConstructionRejection),
        C::CoordinatorStructure,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::AbsenceFault,
        "add-destruction",
        L::AbiLayout,
        Loc::TransactionField,
        Bound::Layer(B::AbiConstructionRejection),
        C::CoordinatorStructure,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::AbsenceFault,
        "add-burn-record",
        L::AbiLayout,
        Loc::TransactionField,
        Bound::Layer(B::AbiConstructionRejection),
        C::CoordinatorStructure,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::AbsenceFault,
        "add-clear-event",
        L::AbiLayout,
        Loc::TransactionField,
        Bound::Layer(B::AbiConstructionRejection),
        C::CoordinatorStructure,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::AbsenceFault,
        "add-residue-projection",
        L::AbiLayout,
        Loc::TransactionOutput,
        Bound::Layer(B::AbiConstructionRejection),
        C::CoordinatorStructure,
        names(projection_policy(), Class::ForbiddenProjectionPresent),
    ),
    // The fault belongs to the open-flow policy relation, which
    // quantifies over every open flow the observed transaction carries
    // rather than over the ones this operation opens.
    fault(
        S::AbsenceFault,
        "introduce-canonical-u-ent-or-dist-ctl-flow",
        L::AbiLayout,
        Loc::TransactionOutput,
        Bound::Layer(B::AbiConstructionRejection),
        C::CoordinatorStructure,
        unlinked(Why::RelationScopedToTheWholeTransaction),
    ),
    // §16.10 — the fourteen sponsor faults.
    //
    // Twelve of them stand where the sponsored arm stands: the sponsor
    // relations have no region, so a sponsored candidate is refused by
    // name before it reaches the boundary the row would declare. The
    // other two ask what the canonical report published, which is a
    // property of bytes this workspace renders and needs no sponsor
    // region to check: no target can answer them and no relation is
    // indexed by them.
    //
    // The isolation and envelope-count relations that most of these rows
    // would name read the observed transaction's flows entire, so nine
    // rows of this table state that reason rather than a link. The two
    // that name a relation about this operation's own positions — the
    // sponsor family's recognition by its asset, and the substrate
    // asset's conservation — keep naming it, because a region gives
    // neither of them anything it lacks.
    fault(
        S::SponsorFault,
        "state-sponsor-overlap",
        L::AbiLayout,
        Loc::SponsorEnvelope,
        Bound::SponsorArmAwaitsRegionScoping,
        C::CoordinatorStructure,
        unlinked(Why::RelationScopedToTheWholeTransaction),
    ),
    fault(
        S::SponsorFault,
        "two-sponsor-envelopes",
        L::AbiLayout,
        Loc::SponsorEnvelope,
        Bound::SponsorArmAwaitsRegionScoping,
        C::CoordinatorStructure,
        unlinked(Why::RelationScopedToTheWholeTransaction),
    ),
    fault(
        S::SponsorFault,
        "foreign-sponsor-asset",
        L::AbiLayout,
        Loc::SponsorEnvelope,
        Bound::SponsorArmAwaitsRegionScoping,
        C::CoordinatorStructure,
        names(sponsor_recognition(), Class::WrongRecognizedAsset),
    ),
    // The fault is the sponsor's own authorization missing, and the class
    // published for it belongs to the sponsor isolation relation rather
    // than to the operator's: that relation is one of the five evaluated
    // over the whole observed transaction, so the row cannot link yet.
    // Naming the operator's authorization relation instead would file a
    // sponsor fault under the nearest published name, which is the
    // guessed link every other row here refuses.
    fault(
        S::SponsorFault,
        "missing-sponsor-authorization",
        L::WitnessProof,
        Loc::WitnessStack,
        Bound::SponsorArmAwaitsRegionScoping,
        C::CoordinatorStructure,
        unlinked(Why::RelationScopedToTheWholeTransaction),
    ),
    fault(
        S::SponsorFault,
        "sponsor-change-at-state-output-0",
        L::TargetTransaction,
        Loc::TransactionOutput,
        Bound::SponsorArmAwaitsRegionScoping,
        C::CoordinatorStructure,
        unlinked(Why::RelationScopedToTheWholeTransaction),
    ),
    fault(
        S::SponsorFault,
        "state-successor-in-sponsor-range",
        L::AbiLayout,
        Loc::SponsorEnvelope,
        Bound::SponsorArmAwaitsRegionScoping,
        C::CoordinatorStructure,
        unlinked(Why::RelationScopedToTheWholeTransaction),
    ),
    fault(
        S::SponsorFault,
        "fee-change-substitution",
        L::TargetTransaction,
        Loc::TransactionOutput,
        Bound::SponsorArmAwaitsRegionScoping,
        C::CoordinatorStructure,
        unlinked(Why::RelationScopedToTheWholeTransaction),
    ),
    fault(
        S::SponsorFault,
        "sponsor-member-unclassified",
        L::AbiLayout,
        Loc::SponsorEnvelope,
        Bound::SponsorArmAwaitsRegionScoping,
        C::CoordinatorStructure,
        unlinked(Why::RelationScopedToTheWholeTransaction),
    ),
    fault(
        S::SponsorFault,
        "empty-sponsor-offer-for-sponsored-request",
        L::AbiLayout,
        Loc::SponsorEnvelope,
        Bound::SponsorArmAwaitsRegionScoping,
        C::CoordinatorStructure,
        unlinked(Why::RelationScopedToTheWholeTransaction),
    ),
    report_fault(
        S::SponsorFault,
        "report-publishes-sponsor-amount",
        L::SemanticFact,
        Loc::ReportField,
        unlinked(Why::TheReportIsTheSubject),
        Projection::ForbiddenPublicationRefused,
    ),
    report_fault(
        S::SponsorFault,
        "report-publishes-sponsor-opening",
        L::SemanticFact,
        Loc::ReportField,
        unlinked(Why::TheReportIsTheSubject),
        Projection::ForbiddenPublicationRefused,
    ),
    fault(
        S::SponsorFault,
        "balanced-state-corruption-compensated-by-sponsor-change",
        L::TargetTransaction,
        Loc::TransactionOutput,
        Bound::SponsorArmAwaitsRegionScoping,
        C::CoordinatorStructure,
        names(
            substrate_conservation(),
            Class::ExternalReportSubjectMismatch,
        ),
    ),
    fault(
        S::SponsorFault,
        "zero-valued-sponsor-member-under-exact-role-structure",
        L::AbiLayout,
        Loc::SponsorEnvelope,
        Bound::SponsorArmAwaitsRegionScoping,
        C::CoordinatorStructure,
        unlinked(Why::RelationScopedToTheWholeTransaction),
    ),
    fault(
        S::SponsorFault,
        "confidential-sponsor-value-where-the-selected-target-policy-claims-support",
        L::TargetTransaction,
        Loc::SponsorEnvelope,
        Bound::SponsorArmAwaitsRegionScoping,
        C::CoordinatorStructure,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    // §16.11 — the twenty-three ABI and linker faults.
    //
    // The coordinator, count, version and sequence rows disturb what the
    // ABI laid out; the symbol, relocation, leaf-weight and leaf-role
    // rows disturb what linking resolved. Most of them name no published
    // relation, and that is a statement about the relation census rather
    // than about the rows: a linker obligation is not a semantic relation
    // and the census has no member to index one by.
    fault(
        S::AbiLinkerFault,
        "wrong-coordinator",
        L::AbiLayout,
        Loc::TransactionField,
        Bound::Layer(B::AbiConstructionRejection),
        C::CoordinatorStructure,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::AbiLinkerFault,
        "duplicate-coordinator",
        L::AbiLayout,
        Loc::TransactionField,
        Bound::Layer(B::AbiConstructionRejection),
        C::CoordinatorStructure,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::AbiLinkerFault,
        "no-coordinator",
        L::AbiLayout,
        Loc::TransactionField,
        Bound::Layer(B::AbiConstructionRejection),
        C::CoordinatorStructure,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    // Exchanging the two regions' positions is a fault against the
    // sponsor isolation relation, which reads the observed transaction's
    // flows entire and so indexes nothing about this operation's
    // positions yet.
    fault(
        S::AbiLinkerFault,
        "state-and-sponsor-positions-exchanged",
        L::AbiLayout,
        Loc::SponsorEnvelope,
        Bound::Layer(B::AbiConstructionRejection),
        C::CoordinatorStructure,
        unlinked(Why::RelationScopedToTheWholeTransaction),
    ),
    fault(
        S::AbiLinkerFault,
        "wrong-input-count",
        L::AbiLayout,
        Loc::TransactionField,
        Bound::Layer(B::AbiConstructionRejection),
        C::CoordinatorStructure,
        names(input_cardinality(), Class::CardinalityAboveMaximum),
    ),
    fault(
        S::AbiLinkerFault,
        "wrong-output-count",
        L::AbiLayout,
        Loc::TransactionField,
        Bound::Layer(B::AbiConstructionRejection),
        C::CoordinatorStructure,
        names(output_cardinality(), Class::CardinalityAboveMaximum),
    ),
    fault(
        S::AbiLinkerFault,
        "wrong-transaction-version",
        L::TargetTransaction,
        Loc::TransactionField,
        Bound::Layer(B::ConsensusRejectionBeforeScript),
        C::Target,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::AbiLinkerFault,
        "wrong-sequence",
        L::TargetTransaction,
        Loc::TransactionField,
        Bound::Layer(B::ConsensusRejectionBeforeScript),
        C::Target,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::AbiLinkerFault,
        "witness-item-reorder",
        L::WitnessProof,
        Loc::WitnessStack,
        Bound::Layer(B::ScriptPathRejection),
        C::Target,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::AbiLinkerFault,
        "predecessor-and-successor-metadata-witnesses-exchanged",
        L::WitnessProof,
        Loc::WitnessStack,
        Bound::Layer(B::ScriptPathRejection),
        C::AnnouncementLeaf,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::AbiLinkerFault,
        "leaf-from-another-bundle",
        L::LinkedConstructorProgram,
        Loc::StaticSubtreeLeaf,
        Bound::Layer(B::LinkerRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::AbiLinkerFault,
        "control-block-from-another-program",
        L::WitnessProof,
        Loc::ControlBlock,
        Bound::Layer(B::ScriptPathRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::AbiLinkerFault,
        "unresolved-metadata-schema-symbol",
        L::LinkedConstructorProgram,
        Loc::LinkerRelocation,
        Bound::Layer(B::LinkerRejection),
        C::LinkedConstructor,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::AbiLinkerFault,
        "unresolved-lead-bound-symbol",
        L::LinkedConstructorProgram,
        Loc::LinkerRelocation,
        Bound::Layer(B::LinkerRejection),
        C::LinkedConstructor,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::AbiLinkerFault,
        "unresolved-operator-symbol",
        L::LinkedConstructorProgram,
        Loc::LinkerRelocation,
        Bound::Layer(B::LinkerRejection),
        C::LinkedConstructor,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::AbiLinkerFault,
        "relocation-omitted",
        L::LinkedConstructorProgram,
        Loc::LinkerRelocation,
        Bound::Layer(B::LinkerRejection),
        C::LinkedConstructor,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::AbiLinkerFault,
        "relocation-applied-twice",
        L::LinkedConstructorProgram,
        Loc::LinkerRelocation,
        Bound::Layer(B::LinkerRejection),
        C::LinkedConstructor,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::AbiLinkerFault,
        "duplicate-tree-leaf",
        L::StaticConstructorSchema,
        Loc::StaticSubtreeLeaf,
        Bound::Layer(B::ConstructorDerivationRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportMissing),
    ),
    fault(
        S::AbiLinkerFault,
        "conflicting-leaf-weight",
        L::StaticConstructorSchema,
        Loc::StaticSubtreeLeaf,
        Bound::Layer(B::ConstructorDerivationRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportMissing),
    ),
    fault(
        S::AbiLinkerFault,
        "conflicting-leaf-role",
        L::StaticConstructorSchema,
        Loc::StaticSubtreeLeaf,
        Bound::Layer(B::ConstructorDerivationRejection),
        C::LinkedConstructor,
        names(constructibility(), Class::ExternalReportMissing),
    ),
    fault(
        S::AbiLinkerFault,
        "candidate-outside-bounds",
        L::LinkedConstructorProgram,
        Loc::LinkedProgram,
        Bound::Layer(B::LinkerRejection),
        C::LinkedConstructor,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::AbiLinkerFault,
        "raw-transaction-bypassing-safe-construction",
        L::TargetTransaction,
        Loc::TransactionField,
        Bound::Layer(B::AbiConstructionRejection),
        C::Target,
        names(constructibility(), Class::ExternalReportMissing),
    ),
    fault(
        S::AbiLinkerFault,
        "target-bytes-changed-after-abi-validation",
        L::TargetTransaction,
        Loc::TransactionField,
        Bound::Layer(B::ScriptPathRejection),
        C::Target,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    // §16.12 — the twenty-two protocol and report faults.
    //
    // Fifteen of them disturb a typed record of the conformance
    // exchange, and one boundary owns all fifteen: the vocabulary names
    // the stage that owns the typed record, and the request and the
    // response records are owned by the same typed-exchange validator.
    // The vocabulary has no separate transport member and this
    // transcription does not mint one — minting a boundary changes a
    // shared vocabulary, which is not something a transcription of one
    // table may do on its own.
    //
    // The last seven ask what the canonical report published. Their
    // subject is this workspace's own serialization, so no target answers
    // them and no relation is indexed by them.
    fault(
        S::ProtocolReportFault,
        "blank-request",
        L::SemanticFact,
        Loc::ProtocolRequestField,
        Bound::Layer(B::SemanticRequestRejection),
        C::TypedProtocolExchange,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::ProtocolReportFault,
        "oversized-request",
        L::SemanticFact,
        Loc::ProtocolRequestField,
        Bound::Layer(B::SemanticRequestRejection),
        C::TypedProtocolExchange,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::ProtocolReportFault,
        "unterminated-request",
        L::SemanticFact,
        Loc::ProtocolRequestField,
        Bound::Layer(B::SemanticRequestRejection),
        C::TypedProtocolExchange,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::ProtocolReportFault,
        "malformed-json",
        L::SemanticFact,
        Loc::ProtocolRequestField,
        Bound::Layer(B::SemanticRequestRejection),
        C::TypedProtocolExchange,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::ProtocolReportFault,
        "unknown-request-field",
        L::SemanticFact,
        Loc::ProtocolRequestField,
        Bound::Layer(B::SemanticRequestRejection),
        C::TypedProtocolExchange,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::ProtocolReportFault,
        "wrong-schema",
        L::SemanticFact,
        Loc::ProtocolRequestField,
        Bound::Layer(B::SemanticRequestRejection),
        C::TypedProtocolExchange,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::ProtocolReportFault,
        "wrong-environment",
        L::SemanticFact,
        Loc::ProtocolRequestField,
        Bound::Layer(B::SemanticRequestRejection),
        C::TypedProtocolExchange,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::ProtocolReportFault,
        "wrong-provenance",
        L::SemanticFact,
        Loc::ProtocolRequestField,
        Bound::Layer(B::SemanticRequestRejection),
        C::TypedProtocolExchange,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::ProtocolReportFault,
        "signing-response-bound-to-other-bytes",
        L::SemanticFact,
        Loc::ProtocolResponseField,
        Bound::Layer(B::SemanticRequestRejection),
        C::TypedProtocolExchange,
        names(authorization(), Class::ExternalReportSubjectMismatch),
    ),
    fault(
        S::ProtocolReportFault,
        "accepted-submission-without-identity",
        L::SemanticFact,
        Loc::ProtocolResponseField,
        Bound::Layer(B::SemanticRequestRejection),
        C::TypedProtocolExchange,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::ProtocolReportFault,
        "accepted-submission-without-mined-readback-where-required",
        L::SemanticFact,
        Loc::ProtocolResponseField,
        Bound::Layer(B::SemanticRequestRejection),
        C::TypedProtocolExchange,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::ProtocolReportFault,
        "rejected-submission-carrying-accepted-identity",
        L::SemanticFact,
        Loc::ProtocolResponseField,
        Bound::Layer(B::SemanticRequestRejection),
        C::TypedProtocolExchange,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::ProtocolReportFault,
        "rejected-signing-response-carrying-witness",
        L::SemanticFact,
        Loc::ProtocolResponseField,
        Bound::Layer(B::SemanticRequestRejection),
        C::TypedProtocolExchange,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::ProtocolReportFault,
        "infrastructure-response-carrying-target-observation",
        L::SemanticFact,
        Loc::ProtocolResponseField,
        Bound::Layer(B::SemanticRequestRejection),
        C::TypedProtocolExchange,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    fault(
        S::ProtocolReportFault,
        "conservation-rejection-carrying-accepted-only-openings",
        L::SemanticFact,
        Loc::ProtocolResponseField,
        Bound::Layer(B::SemanticRequestRejection),
        C::TypedProtocolExchange,
        unlinked(Why::NoPublishedRelationNamesIt),
    ),
    report_fault(
        S::ProtocolReportFault,
        "caller-authored-evidence-standing",
        L::SemanticFact,
        Loc::ReportField,
        unlinked(Why::TheReportIsTheSubject),
        Projection::ForbiddenPublicationRefused,
    ),
    report_fault(
        S::ProtocolReportFault,
        "report-summary-edited",
        L::SemanticFact,
        Loc::ReportField,
        unlinked(Why::TheReportIsTheSubject),
        Projection::ForbiddenPublicationRefused,
    ),
    report_fault(
        S::ProtocolReportFault,
        "failed-row-removed",
        L::SemanticFact,
        Loc::ReportField,
        unlinked(Why::TheReportIsTheSubject),
        Projection::ForbiddenPublicationRefused,
    ),
    report_fault(
        S::ProtocolReportFault,
        "duplicate-passing-row",
        L::SemanticFact,
        Loc::ReportField,
        unlinked(Why::TheReportIsTheSubject),
        Projection::ForbiddenPublicationRefused,
    ),
    report_fault(
        S::ProtocolReportFault,
        "observed-target-layer-differs-from-declared-row-boundary",
        L::SemanticFact,
        Loc::ReportField,
        unlinked(Why::TheReportIsTheSubject),
        Projection::ForbiddenPublicationRefused,
    ),
    report_fault(
        S::ProtocolReportFault,
        "canonical-report-includes-wall-time",
        L::SemanticFact,
        Loc::ReportField,
        unlinked(Why::TheReportIsTheSubject),
        Projection::ForbiddenPublicationRefused,
    ),
    report_fault(
        S::ProtocolReportFault,
        "typed-diagnostic-contains-a-caller-path-or-injected-line",
        L::SemanticFact,
        Loc::ReportField,
        unlinked(Why::TheReportIsTheSubject),
        Projection::ForbiddenPublicationRefused,
    ),
    // §16.13 — the thirteen public-recovery faults.
    //
    // The validated public successor recovery report reads the accepted
    // announcement's public handoff and reconstructs its successor. Its
    // constrained process answers nine faults by refusal and four by
    // construction proof; the report retains its chain and origin limits.
    fault(
        S::RecoveryFault,
        "missing-transaction",
        L::SemanticFact,
        Loc::PublicationRecord,
        Bound::Layer(B::PublicRecoveryReportRejection),
        C::Report,
        names(projection_policy(), Class::MissingRequiredProjection),
    ),
    fault(
        S::RecoveryFault,
        "copied-transaction-bytes",
        L::SemanticFact,
        Loc::PublicationRecord,
        Bound::Layer(B::PublicRecoveryReportRejection),
        C::Report,
        names(projection_policy(), Class::MissingRequiredProjection),
    ),
    fault(
        S::RecoveryFault,
        "wrong-output-index",
        L::SemanticFact,
        Loc::PublicationRecord,
        Bound::Layer(B::PublicRecoveryReportRejection),
        C::Report,
        names(projection_policy(), Class::MissingRequiredProjection),
    ),
    fault(
        S::RecoveryFault,
        "stale-successor",
        L::SemanticFact,
        Loc::PublicationRecord,
        Bound::Layer(B::PublicRecoveryReportRejection),
        C::Report,
        names(projection_policy(), Class::MissingRequiredProjection),
    ),
    fault(
        S::RecoveryFault,
        "malformed-public-witness",
        L::SemanticFact,
        Loc::PublicationRecord,
        Bound::Layer(B::PublicRecoveryReportRejection),
        C::Report,
        names(projection_policy(), Class::MissingRequiredProjection),
    ),
    fault(
        S::RecoveryFault,
        "metadata-from-another-successor",
        L::SemanticFact,
        Loc::PublicationRecord,
        Bound::Layer(B::PublicRecoveryReportRejection),
        C::Report,
        names(projection_policy(), Class::MissingRequiredProjection),
    ),
    fault(
        S::RecoveryFault,
        "missing-representation-nonce",
        L::SemanticFact,
        Loc::PublicationRecord,
        Bound::Layer(B::PublicRecoveryReportRejection),
        C::Report,
        names(projection_policy(), Class::MissingRequiredProjection),
    ),
    fault(
        S::RecoveryFault,
        "wrong-static-subtree",
        L::SemanticFact,
        Loc::PublicationRecord,
        Bound::Layer(B::PublicRecoveryReportRejection),
        C::Report,
        names(projection_policy(), Class::MissingRequiredProjection),
    ),
    fault(
        S::RecoveryFault,
        "creator-process-memory-required",
        L::SemanticFact,
        Loc::PublicationRecord,
        Bound::Layer(B::PublicRecoveryReportRejection),
        C::Report,
        names(projection_policy(), Class::MissingRequiredProjection),
    ),
    fault(
        S::RecoveryFault,
        "temporary-file-required",
        L::SemanticFact,
        Loc::PublicationRecord,
        Bound::Layer(B::PublicRecoveryReportRejection),
        C::Report,
        names(projection_policy(), Class::MissingRequiredProjection),
    ),
    fault(
        S::RecoveryFault,
        "wallet-descriptor-required",
        L::SemanticFact,
        Loc::PublicationRecord,
        Bound::Layer(B::PublicRecoveryReportRejection),
        C::Report,
        names(projection_policy(), Class::MissingRequiredProjection),
    ),
    fault(
        S::RecoveryFault,
        "unknown-publication-field",
        L::SemanticFact,
        Loc::PublicationRecord,
        Bound::Layer(B::PublicRecoveryReportRejection),
        C::Report,
        names(projection_policy(), Class::ForbiddenProjectionPresent),
    ),
    fault(
        S::RecoveryFault,
        "reconstructed-program-differs-from-chain-output",
        L::SemanticFact,
        Loc::PublicationRecord,
        Bound::Layer(B::PublicRecoveryReportRejection),
        C::Report,
        names(projection_policy(), Class::MissingRequiredProjection),
    ),
];

/// Every row of the matrix, in the guide's order.
#[must_use]
pub const fn rows() -> &'static [MaturitySafetyRow] {
    MATURITY_SAFETY_ROWS
}

/// Read the two fixed-metadata carrier moves from their matrix rows.
///
/// The row table owns each whole carrier and declared boundary. The
/// variable leaf supplies the constant header by symbol and reserved zeros
/// by literal before the same checks run.
#[must_use]
pub fn carrier_moves() -> [MaturityCarrierMove; 2] {
    let mut moves = [
        MaturityCarrierMove::from_row(&rows()[0], MaturityReMaterializedCarrier::LeafHeaderSymbol),
        MaturityCarrierMove::from_row(
            &rows()[0],
            MaturityReMaterializedCarrier::LeafReservedLiteral,
        ),
    ];
    for row in rows() {
        match row.name() {
            "wrong-domain-separator" => {
                moves[0] = MaturityCarrierMove::from_row(
                    row,
                    MaturityReMaterializedCarrier::LeafHeaderSymbol,
                );
            }
            "nonzero-reserved-field" => {
                moves[1] = MaturityCarrierMove::from_row(
                    row,
                    MaturityReMaterializedCarrier::LeafReservedLiteral,
                );
            }
            _ => {}
        }
    }
    debug_assert_eq!(moves[0].row(), "wrong-domain-separator");
    debug_assert_eq!(moves[1].row(), "nonzero-reserved-field");
    moves
}

/// Every row of one §16 table.
pub fn rows_of(section: MaturitySafetySection) -> impl Iterator<Item = &'static MaturitySafetyRow> {
    rows().iter().filter(move |row| row.section() == section)
}

/// How many rows each §16 table contributes.
///
/// Counted from the rows themselves rather than written beside them: a
/// figure kept next to a table is a second statement of the table's
/// width, and the two disagree the first time a row moves.
#[must_use]
pub fn census() -> BTreeMap<MaturitySafetySection, usize> {
    rows().iter().fold(BTreeMap::new(), |mut census, row| {
        *census.entry(row.section()).or_insert(0) += 1;
        census
    })
}

/// How many rows the complete matrix has.
#[must_use]
pub const fn row_count() -> usize {
    rows().len()
}

/// Which requirement one row names, once resolved against the plan.
///
/// Four answers and no fifth. A row either resolves to exactly one
/// published requirement, intends the whole relation census rather than
/// one member of it, names a relation the plan does not require anything
/// of in this case, or names none at all for a reason it states.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaturityRowLink {
    /// Exactly one published requirement matched.
    Resolved(CoverageRequirementId),
    /// The row intends every published relation of the operation to hold.
    EveryRelation,
    /// The relation is not active in this representation and case.
    ///
    /// An inactive relation-case carries one obligation — that the case is
    /// accepted and the relation never activates — and no mutation at all,
    /// because a target rejecting a case this relation does not exercise
    /// would prove nothing about it. A sponsor family's recognition is
    /// exactly such a relation in the sponsorless case, and saying so is
    /// different from saying the row names no requirement anywhere.
    ///
    /// This is not a licence for a row to resolve nowhere. A relation
    /// whose activation does not depend on the sponsor envelope cannot
    /// reach this answer honestly, and the tests below hold every
    /// declared row to that: an inactive answer from a relation about this
    /// operation's own positions is a disagreement between the row and the
    /// plan, reported rather than absorbed.
    InactiveInThisCase,
    /// The row names no requirement, for a stated reason.
    Blocked(MaturityUnlinkedReason),
}

/// Resolve one §16 row against the published announcement plan.
///
/// The relation, the execution case and the published class select the
/// requirement, and the resolution must find exactly one. A table mapping
/// rows to requirement identities would be a third authority that could
/// agree with neither the guide nor the plan, and it would still agree
/// with itself after the plan changed; a resolution that must hit exactly
/// one fails at the moment the plan moves.
///
/// # What the class contributes, and why the boundary is part of it
///
/// A published negative class is a boundary and a mutation together, so
/// the class carries both. Selecting on the mutation alone would be exact
/// for most of this operation's relations and wrong for its lifecycle
/// exit, which requires the same mutation of the validated plan and of
/// the emitted program: two obligations, one word. Taking either of them
/// as the row's answer would be the discharge-by-intent the census exists
/// to prevent, and taking both would leave the row unable to say which it
/// meant.
///
/// # A row's own boundary is deliberately not consulted
///
/// Where a row expects its verdict to come from is a different question
/// from what its change violates. A construction the builder refuses and
/// a construction the script refuses violate the same relation, and the
/// row names that relation either way; what the declared boundary decides
/// is whether a link is a discharge, which belongs to an evidence plan
/// and not to a resolution.
///
/// # Errors
///
/// [`VectorError::NegativeLinkUnresolved`] when the plan publishes this
/// representation not at all, or publishes more than one requirement of
/// the row's relation, case and class — either of which means the row and
/// the plan disagree about what exists.
pub fn resolve_row(
    plan: &ValidatedMaturityAnnouncementOperationPlan,
    row: &MaturitySafetyRow,
    representation: MaturityAnnouncementRepresentationPlan,
    case: SponsorCase,
) -> Result<MaturityRowLink, VectorError> {
    let (relation, class) = match row.relation() {
        Standing::EveryPublishedRelation => return Ok(MaturityRowLink::EveryRelation),
        Standing::Unlinked(reason) => return Ok(MaturityRowLink::Blocked(*reason)),
        Standing::Declared { relation, class } => (relation, *class),
    };
    let class_name = class.name();

    let projection = plan
        .projection(representation)
        .ok_or(VectorError::NegativeLinkUnresolved { class: class_name })?;

    let mut found: BTreeSet<CoverageRequirementId> = BTreeSet::new();
    for requirement in projection.coverage() {
        let TargetCoverageObligation::Negative(negative) = &requirement.obligation else {
            continue;
        };
        if requirement.id.relation != *relation
            || requirement.id.case.sponsor != case
            || requirement.id.boundary != class.boundary()
            || !class.matches(&negative.mutation)
        {
            continue;
        }
        found.insert(requirement.id.clone());
    }

    let mut resolved = found.into_iter();
    match (resolved.next(), resolved.next()) {
        (Some(only), None) => Ok(MaturityRowLink::Resolved(only)),
        // Nothing published here is the conditional answer rather than a
        // failure: an inactive relation-case publishes no negative, which
        // in the sponsorless case is what a sponsor family's relations
        // are. Two or more is always a failure — the row cannot say which
        // it meant, and taking the first would answer the row with
        // whichever requirement happened to sort earliest.
        (None, _) => Ok(MaturityRowLink::InactiveInThisCase),
        (Some(_), Some(_)) => Err(VectorError::NegativeLinkUnresolved { class: class_name }),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        B, Bound, C, Control, CoverageRequirementId, L, Loc, MaturityMutationClass,
        MaturityMutationSubject as Subj, MaturityRelationStanding, MaturityRowBoundary,
        MaturityRowLink, MaturitySafetyPolarity, MaturitySafetyRow, MaturitySafetySection,
        ObjectId, Projection, RelationId, RelationSubject, S, SponsorCase, Standing,
        TargetCoverageObligation, ValidatedMaturityAnnouncementOperationPlan, Why, boundary_admits,
        carrier_moves, census, resolve_row, row_count, rows, rows_of,
    };
    use crate::observed_boundary::{matches_boundary, observed_boundary};
    use std::collections::{BTreeMap, BTreeSet};
    use target_elements_conformance::protocol::ObservedOutcomeLayer;

    /// Every observed layer this workspace knows.
    const OBSERVED_LAYERS: &[ObservedOutcomeLayer] = &[
        ObservedOutcomeLayer::FixtureConstructionFailure,
        ObservedOutcomeLayer::ExecutorInfrastructureFailure,
        ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
        ObservedOutcomeLayer::ScriptPathRejection,
        ObservedOutcomeLayer::KeyPathRejection,
        ObservedOutcomeLayer::RelayPolicyRejection,
        ObservedOutcomeLayer::Accepted,
    ];

    /// The control each row departs from, against the subject it changes,
    /// with the rows standing on each pair.
    ///
    /// Stated here and recomputed from the rows in the test, so that the
    /// two have to agree: a row that changes its control, its locator or
    /// its table moves a figure, and a pair appearing or disappearing is
    /// a line of this table rather than a silence. The positive rows
    /// stand on the one pair with no subject, because they change
    /// nothing.
    const CONTROL_AGAINST_SUBJECT: &[(Control, Option<Subj>, usize)] = &[
        (Control::TheRowIsTheControl, None, 14),
        (
            Control::AcceptedAnnouncementPublication,
            Some(Subj::Publication),
            13,
        ),
        (
            Control::CanonicalProtocolExchange,
            Some(Subj::WireRequest),
            8,
        ),
        (
            Control::CanonicalProtocolExchange,
            Some(Subj::WireResponse),
            7,
        ),
        (Control::CanonicalSafetyReport, Some(Subj::Report), 9),
        (
            Control::CanonicalSafetyReport,
            Some(Subj::SuccessorMetadata),
            6,
        ),
        (
            Control::SponsoredAnnouncement,
            Some(Subj::SponsorEnvelope),
            8,
        ),
        (
            Control::SponsoredAnnouncement,
            Some(Subj::TransactionOutput),
            3,
        ),
        (Control::SponsoredAnnouncement, Some(Subj::Witness), 1),
        (
            Control::SponsorlessAnnouncement,
            Some(Subj::AdmittedFamily),
            9,
        ),
        (
            Control::SponsorlessAnnouncement,
            Some(Subj::ApprovedOperatorKey),
            3,
        ),
        (Control::SponsorlessAnnouncement, Some(Subj::BranchOrder), 3),
        (
            Control::SponsorlessAnnouncement,
            Some(Subj::ControlBlock),
            9,
        ),
        (
            Control::SponsorlessAnnouncement,
            Some(Subj::LinkedProgram),
            5,
        ),
        (
            Control::SponsorlessAnnouncement,
            Some(Subj::MetadataEncoding),
            9,
        ),
        (Control::SponsorlessAnnouncement, Some(Subj::Nonce), 9),
        (
            Control::SponsorlessAnnouncement,
            Some(Subj::OperatorResponse),
            11,
        ),
        (
            Control::SponsorlessAnnouncement,
            Some(Subj::PredecessorMetadata),
            4,
        ),
        (
            Control::SponsorlessAnnouncement,
            Some(Subj::RelocationCensus),
            5,
        ),
        (
            Control::SponsorlessAnnouncement,
            Some(Subj::RootHistory),
            16,
        ),
        (
            Control::SponsorlessAnnouncement,
            Some(Subj::SemanticRequest),
            7,
        ),
        // One row of the ABI and linker table changes the sponsor
        // envelope while departing from the sponsorless announcement,
        // which has none to change. The incidence records it as it
        // stands: the matrix is transcribed here and a row is not
        // retyped by the test that measures it.
        (
            Control::SponsorlessAnnouncement,
            Some(Subj::SponsorEnvelope),
            1,
        ),
        (
            Control::SponsorlessAnnouncement,
            Some(Subj::StaticSubtree),
            10,
        ),
        (
            Control::SponsorlessAnnouncement,
            Some(Subj::SuccessorMetadata),
            5,
        ),
        (
            Control::SponsorlessAnnouncement,
            Some(Subj::TransactionField),
            13,
        ),
        (
            Control::SponsorlessAnnouncement,
            Some(Subj::TransactionInput),
            3,
        ),
        (
            Control::SponsorlessAnnouncement,
            Some(Subj::TransactionOutput),
            8,
        ),
        (Control::SponsorlessAnnouncement, Some(Subj::Tweak), 4),
        (Control::SponsorlessAnnouncement, Some(Subj::Witness), 3),
    ];

    #[test]
    fn the_matrix_transcribes_every_table_at_its_recounted_width() {
        // §16's own counts, recomputed from the rows rather than asserted
        // from prose. A row added or dropped has to change this line on
        // purpose, which is the whole reason the matrix is an object and
        // not a reading.
        //
        // §16.3 is sixteen and not eleven: the table lists eleven faults
        // and then asks for one row per semantic field the announcement
        // leaves unaffected, and the metadata record carries six fields
        // of which the transition moves exactly one.
        let census = census();
        assert_eq!(census[&S::Positive], 14);
        assert_eq!(census[&S::WindowFault], 11);
        assert_eq!(census[&S::MetadataFault], 16);
        assert_eq!(census[&S::OperatorFault], 18);
        assert_eq!(census[&S::PredecessorConstructorFault], 15);
        assert_eq!(census[&S::SuccessorConstructorFault], 15);
        assert_eq!(census[&S::TotalityFault], 14);
        assert_eq!(census[&S::RootHistoryFault], 16);
        assert_eq!(census[&S::AbsenceFault], 15);
        assert_eq!(census[&S::SponsorFault], 14);
        assert_eq!(census[&S::AbiLinkerFault], 23);
        assert_eq!(census[&S::ProtocolReportFault], 22);
        assert_eq!(census[&S::RecoveryFault], 13);
        assert_eq!(row_count(), 206);
        assert_eq!(census.values().sum::<usize>(), row_count());
        assert_eq!(census.len(), MaturitySafetySection::ALL.len());
    }

    #[test]
    fn carrier_moves_copy_the_named_metadata_rows() {
        let moves = carrier_moves();
        let names = ["wrong-domain-separator", "nonzero-reserved-field"];
        let variable_carriers = [
            super::MaturityReMaterializedCarrier::LeafHeaderSymbol,
            super::MaturityReMaterializedCarrier::LeafReservedLiteral,
        ];
        for ((movement, name), variable_carrier) in moves.iter().zip(names).zip(variable_carriers) {
            let matching: Vec<_> = rows().iter().filter(|row| row.name() == name).collect();
            assert_eq!(matching.len(), 1);
            let row = matching[0];
            assert_eq!(movement.row(), row.name());
            assert_eq!(movement.whole_carrier(), row.carrier());
            assert_eq!(movement.variable_carrier(), variable_carrier);
            assert_eq!(movement.boundary(), row.boundary());
            assert_eq!(movement.whole_carrier(), C::AnnouncementLeaf);
            assert_eq!(
                movement.boundary(),
                Bound::Layer(B::AbiConstructionRejection)
            );
        }
    }

    #[test]
    fn carrier_moves_leave_the_matrix_census_and_rendering_unchanged() {
        let before_rows = rows().to_vec();
        let before_rendering: Vec<_> = rows().iter().map(ToString::to_string).collect();
        let before_census = census();
        let _moves = carrier_moves();
        assert_eq!(rows(), before_rows);
        assert_eq!(
            rows().iter().map(ToString::to_string).collect::<Vec<_>>(),
            before_rendering
        );
        assert_eq!(census(), before_census);
        assert_eq!(row_count(), 206);
        assert_eq!(census().len(), 13);
    }

    #[test]
    fn every_row_is_named_once() {
        // Within a table, because a duplicate name makes two rows one row
        // for every consumer that keys by it while the census still
        // counts two.
        for section in MaturitySafetySection::ALL {
            let rows: Vec<_> = rows_of(*section).collect();
            let names: BTreeSet<_> = rows.iter().copied().map(MaturitySafetyRow::name).collect();
            assert_eq!(names.len(), rows.len(), "{section:?} names a row twice");
        }
        // Across the whole matrix by rendered identity rather than by
        // bare name. Several tables use the same wording for a fault
        // against a different object — a wrong leaf version of the
        // predecessor and of the successor are different rows with the
        // same words — and renaming one of them would paraphrase the
        // guide, which a transcription may not do. The section is what
        // tells them apart, so the section is part of the identity.
        let rendered: BTreeSet<String> = rows().iter().map(ToString::to_string).collect();
        assert_eq!(rendered.len(), row_count());
    }

    #[test]
    fn every_row_sits_in_its_own_table_in_guide_order() {
        for section in MaturitySafetySection::ALL {
            for row in rows_of(*section) {
                assert_eq!(row.section(), *section, "{row} sits in another table");
            }
        }
        // The tables are contiguous and in §16's order, so a row's place
        // in its table is its ordinal and a reader can walk the guide and
        // this array side by side.
        let mut order: Vec<MaturitySafetySection> = Vec::new();
        for row in rows() {
            if order.last() != Some(&row.section()) {
                assert!(
                    !order.contains(&row.section()),
                    "{row} breaks its table's contiguity",
                );
                order.push(row.section());
            }
        }
        assert_eq!(order.as_slice(), MaturitySafetySection::ALL);
    }

    #[test]
    fn each_typed_non_answer_stands_on_exactly_the_rows_its_reason_names() {
        let standing = |wanted: MaturityRowBoundary| {
            rows().iter().filter(|row| row.boundary() == wanted).count()
        };
        // Eleven positive rows ask for an acceptance that has no route,
        // and the twelfth such row is §16.1's last, which asks for the
        // acceptance itself.
        assert_eq!(standing(Bound::AcceptanceAwaitsRelayAdmissibility), 11);
        assert_eq!(standing(Bound::SponsorArmAwaitsRegionScoping), 12);
        assert_eq!(standing(Bound::TotalityAnsweredByLandedNonceEvidence), 11);
        assert_eq!(standing(Bound::PropertyOfTheBuildRatherThanAFixture), 1);
        let non_answers = rows()
            .iter()
            .filter(|row| row.boundary().is_typed_non_answer())
            .count();
        assert_eq!(non_answers, 35);
        assert_eq!(row_count() - non_answers, 171);
        assert_eq!(
            rows()
                .iter()
                .filter(|row| row.refusing_layer() == Some(B::RootHistoryReportRejection))
                .count(),
            16,
        );
        assert_eq!(
            rows()
                .iter()
                .filter(|row| row.refusing_layer() == Some(B::PublicRecoveryReportRejection))
                .count(),
            14,
        );
        // And each reason stands only where it applies. A reason carried
        // by a row of another table would be a reason borrowed to cover a
        // row nobody had an argument for.
        for row in rows() {
            match row.boundary() {
                Bound::AcceptanceAwaitsRelayAdmissibility
                | Bound::PropertyOfTheBuildRatherThanAFixture => {
                    assert_eq!(row.section(), S::Positive, "{row} is not a positive row");
                }
                Bound::SponsorArmAwaitsRegionScoping => {
                    assert_eq!(row.section(), S::SponsorFault, "{row} is not a sponsor row");
                }
                Bound::TotalityAnsweredByLandedNonceEvidence => {
                    assert_eq!(
                        row.section(),
                        S::TotalityFault,
                        "{row} is not a totality row"
                    );
                }
                Bound::Layer(layer) => match layer {
                    B::RootHistoryReportRejection => assert_eq!(
                        row.section(),
                        S::RootHistoryFault,
                        "{row} is not a root-history row",
                    ),
                    B::PublicRecoveryReportRejection => assert!(
                        matches!(row.section(), S::Positive | S::RecoveryFault),
                        "{row} is not a public-recovery row",
                    ),
                    _ => {}
                },
            }
        }
    }

    #[test]
    fn polarity_follows_the_table_and_a_change_has_somewhere_to_sit() {
        for row in rows() {
            let positive = row.polarity() == MaturitySafetyPolarity::Positive;
            assert_eq!(
                positive,
                row.section().is_positive(),
                "{row} sits in a table of the other polarity",
            );
            // A positive row changes nothing, so it has no layer to
            // change at and nowhere to point; a negative row has both,
            // because a change with no place is an input that does not
            // exist.
            assert_eq!(
                row.mutation().is_some(),
                !positive,
                "{row} disagrees with its table about changing something",
            );
            assert_eq!(
                row.locator().is_some(),
                row.mutation().is_some(),
                "{row} states a change with nowhere to point",
            );
            if positive {
                assert_eq!(row.control(), Control::TheRowIsTheControl);
                assert_eq!(row.projection(), Projection::EveryTermMatches);
                assert_eq!(row.relation(), &Standing::EveryPublishedRelation);
                assert_eq!(row.collateral(), None);
            }
        }
    }

    #[test]
    fn a_declared_boundary_admits_its_change_and_agrees_with_the_observation_mapping() {
        for row in rows() {
            let Some(boundary) = row.refusing_layer() else {
                continue;
            };
            if let Some(layer) = row.mutation() {
                assert!(
                    boundary_admits(boundary, layer),
                    "{row} changes a layer its declared boundary never sees",
                );
            }
            // The whole product of observed layers against this row's
            // declared boundary, read through the one mapping both
            // consumers of the rule share. A target-verdict boundary is
            // reached by exactly one observed layer and a pre-target one
            // by none, so a row cannot be answered by a run at a layer it
            // did not declare.
            //
            // The report-semantic, root-history continuity, and public
            // successor recovery layers are reached by none either:
            // their comparisons over accepted bytes are report evidence,
            // rather than observed target verdicts.
            let reachable = OBSERVED_LAYERS
                .iter()
                .filter(|observed| matches_boundary(boundary, **observed))
                .count();
            let expected = usize::from(matches!(
                boundary,
                B::ConsensusRejectionBeforeScript
                    | B::KeyPathRejection
                    | B::ScriptPathRejection
                    | B::RelayPolicyRejection
                    | B::AcceptedTransaction
            ));
            assert_eq!(
                reachable, expected,
                "{row} declares a boundary the observation mapping does not reach once",
            );
        }
    }

    #[test]
    fn the_rendered_table_holds_no_bytes_and_no_object_names() {
        for row in rows() {
            let rendered = row.to_string();
            assert!(
                rendered.starts_with('\u{a7}'),
                "{row} does not render as a guide row",
            );
            assert!(
                rendered.contains(row.section().section()),
                "{row} renders under another table",
            );
            let name = row.name();
            assert!(
                name.chars().all(|letter| letter.is_ascii_lowercase()
                    || letter.is_ascii_digit()
                    || letter == '-'),
                "{row} is not named in the table's own words",
            );
            // A fixture byte string and an object name both read as a run
            // of hexadecimal digits. The table's own wording never
            // produces one this long, so a run of seven is bytes that
            // entered a file whose whole claim is that it holds none.
            let longest = name
                .split(|letter: char| !letter.is_ascii_hexdigit())
                .map(str::len)
                .max()
                .unwrap_or_default();
            assert!(longest < 7, "{row} carries what reads as an object name");
        }
    }

    /// The facts a row carries agree with one another.
    ///
    /// Nine facts on one row are nine chances to say two incompatible
    /// things, and the pairs below are the ones where a disagreement
    /// would be invisible to every other test here.
    #[test]
    fn the_facts_a_row_carries_agree_with_one_another() {
        for row in rows() {
            let carrier = row.carrier();
            // A row that names a relation expects that relation and its
            // dependency closure both reported blocked; a row that names
            // none has nothing to demand it of.
            assert_eq!(
                row.collateral().is_some(),
                matches!(row.relation(), MaturityRelationStanding::Declared { .. }),
                "{row} demands collateral of a requirement it does not name",
            );
            // A report row's subject is the report, and a row whose
            // subject is the report neither names a relation nor pretends
            // a transaction carrier answers it.
            if matches!(
                row.relation(),
                Standing::Unlinked(Why::TheReportIsTheSubject)
            ) {
                assert_eq!(
                    carrier,
                    C::Report,
                    "{row} files a report question elsewhere"
                );
            }
        }
    }

    /// Every layer a row declares is one of the six the shared vocabulary
    /// publishes, read back through the admission rule.
    #[test]
    fn the_admission_rule_is_total_over_the_shared_vocabularies() {
        for boundary in B::ALL {
            for layer in L::ALL {
                let admitted = boundary_admits(*boundary, *layer);
                // An infrastructure failure and an acceptance admit no
                // change at all: the first is never evidence that a
                // change was refused, and the second is what a row
                // expects when it changes nothing.
                if matches!(
                    boundary,
                    B::ExecutorInfrastructureFailure | B::AcceptedTransaction
                ) {
                    assert!(!admitted, "{boundary:?} admits {layer:?}");
                }
            }
        }
        // And the rule is actually exercised: every layer the rows use is
        // admitted somewhere, so the arms are a rule rather than a
        // decoration.
        for row in rows() {
            if let (Some(boundary), Some(layer)) = (row.refusing_layer(), row.mutation()) {
                assert!(boundary_admits(boundary, layer), "{row}");
            }
        }
    }

    /// Every locator the matrix uses names exactly one subject, and every
    /// subject the vocabulary publishes stands on a row.
    ///
    /// Totality is the compiler's: the map has no fallback arm, so a
    /// locator minted later fails to build rather than resolving by
    /// accident. What a test can add is the other two halves —
    /// injectivity, which is what makes "exactly one" a claim rather
    /// than a collapse of several places onto one name, and
    /// inhabitation, which is what keeps the vocabulary the rows' own
    /// rather than a wider list nobody points at.
    #[test]
    fn every_locator_names_one_subject_and_every_subject_stands_on_a_row() {
        let named: BTreeSet<Loc> = rows()
            .iter()
            .filter_map(MaturitySafetyRow::locator)
            .collect();
        let subjects: BTreeSet<Subj> = named.iter().copied().map(Loc::subject).collect();
        assert_eq!(
            subjects.len(),
            named.len(),
            "two locators of the matrix resolve to one subject",
        );
        let published: BTreeSet<Subj> = Subj::ALL.iter().copied().collect();
        assert_eq!(
            published.len(),
            Subj::ALL.len(),
            "a subject is listed twice"
        );
        assert_eq!(
            subjects, published,
            "the subjects the rows name and the vocabulary's own list differ",
        );
        assert_eq!(named.len(), 24);
    }

    /// The rows that point nowhere are exactly the fourteen positive ones.
    ///
    /// A positive row changes nothing, so it has no subject to name: that
    /// is the only reason a row of this matrix may carry no locator, and
    /// the count is stated so that a negative row losing its locator
    /// fails here rather than being read as a positive one. The
    /// consequence for the observed side is stated where it is used: no
    /// negative row's locator is absent at this tip, so the structural
    /// separator a run may report is never the form a row declared.
    #[test]
    fn the_rows_that_point_nowhere_are_exactly_the_positive_fourteen() {
        let mut pointing_nowhere = 0usize;
        for row in rows() {
            if row.locator().is_some() {
                continue;
            }
            pointing_nowhere += 1;
            assert_eq!(
                row.polarity(),
                MaturitySafetyPolarity::Positive,
                "{row} changes something and names no subject",
            );
            assert_eq!(row.section(), S::Positive, "{row} sits outside its table");
            assert_eq!(
                row.control(),
                Control::TheRowIsTheControl,
                "{row} departs from a control while changing nothing",
            );
        }
        assert_eq!(pointing_nowhere, 14);
        let negatives_pointing_nowhere = rows()
            .iter()
            .filter(|row| {
                row.polarity() == MaturitySafetyPolarity::Negative && row.locator().is_none()
            })
            .count();
        assert_eq!(negatives_pointing_nowhere, 0);
    }

    /// Each control shape stands only on rows whose subject it can build.
    ///
    /// The control a row departs from is derived from the row's table, so
    /// a test restating that derivation would agree with itself. The
    /// incidence of controls against subjects does not: it states which
    /// objects each canonical control actually has to offer a mutant,
    /// and it moves the moment a row changes either fact. Two of the six
    /// controls are additionally pinned to a fact outside the table — a
    /// row is its own control exactly when it is positive, and the
    /// rendered report is the control exactly where the row's verdict
    /// comes from the report layer.
    #[test]
    fn each_control_shape_stands_only_on_rows_whose_subject_it_can_build() {
        let mut measured: BTreeMap<(Control, Option<Subj>), usize> = BTreeMap::new();
        for row in rows() {
            *measured
                .entry((row.control(), row.locator().map(Loc::subject)))
                .or_default() += 1;
        }
        let stated: BTreeMap<(Control, Option<Subj>), usize> = CONTROL_AGAINST_SUBJECT
            .iter()
            .map(|(control, subject, count)| ((*control, *subject), *count))
            .collect();
        assert_eq!(
            stated.len(),
            CONTROL_AGAINST_SUBJECT.len(),
            "the stated incidence names one pair twice",
        );
        assert_eq!(
            measured, stated,
            "the control a row departs from, against the subject it changes, is not the stated incidence",
        );
        assert_eq!(stated.values().sum::<usize>(), row_count());
        for row in rows() {
            assert_eq!(
                row.control() == Control::TheRowIsTheControl,
                row.polarity() == MaturitySafetyPolarity::Positive,
                "{row} is its own control without being positive, or the other way about",
            );
            assert_eq!(
                row.control() == Control::CanonicalSafetyReport,
                row.refusing_layer() == Some(B::ReportSemanticProjectionRejection),
                "{row} departs from a rendered report without the report answering it",
            );
            if row.control() == Control::CanonicalSafetyReport {
                assert_eq!(
                    row.carrier(),
                    C::Report,
                    "{row} files a report question elsewhere"
                );
            }
        }
    }

    /// The observed product, walked per row.
    ///
    /// The shared mapping's own test walks the whole product of layers
    /// against boundaries and proves the mapping is diagonal. That says
    /// nothing about the rows: whether any row's declared boundary is one
    /// an observation can reach at all, and how many rows wait on a layer
    /// no run produces, are facts of this matrix. Walking the product
    /// once per row states them as figures — and ties them to the
    /// classification's own denominators, since the rows no observation
    /// reaches are exactly the pre-target and three report-layer rows.
    #[test]
    fn the_observed_product_reaches_each_row_at_its_own_boundary_only() {
        let mut reachable = 0usize;
        let mut unreachable = 0usize;
        for row in rows() {
            let Some(boundary) = row.refusing_layer() else {
                continue;
            };
            let mut hits = 0usize;
            for observed in OBSERVED_LAYERS {
                let matched = matches_boundary(boundary, *observed);
                assert_eq!(
                    matched,
                    observed_boundary(*observed) == Some(boundary),
                    "{row} against {observed:?} did not follow the mapping",
                );
                if matched {
                    hits += 1;
                }
            }
            if hits == 0 {
                unreachable += 1;
            } else {
                assert_eq!(hits, 1, "{row} is reached at more than one observed layer");
                reachable += 1;
            }
        }
        assert_eq!(reachable, 43);
        assert_eq!(unreachable, 128);
        assert_eq!(reachable + unreachable, 171);
        let pre_target = rows()
            .iter()
            .filter(|row| row.refusing_layer().is_some_and(B::is_pre_target))
            .count();
        let report = rows()
            .iter()
            .filter(|row| {
                matches!(
                    row.refusing_layer(),
                    Some(
                        B::ReportSemanticProjectionRejection
                            | B::RootHistoryReportRejection
                            | B::PublicRecoveryReportRejection
                    )
                )
            })
            .count();
        assert_eq!(pre_target, 83);
        assert_eq!(report, 45);
        assert_eq!(
            pre_target + report,
            unreachable,
            "a row no observation reaches is neither pre-target nor answered by a report layer",
        );
    }

    /// The plan every resolution here is read against.
    ///
    /// The one derivation the crate already keeps, rather than a second
    /// one built for the tests: a plan assembled here could pass a
    /// resolution the shipped plan would fail.
    fn announcement_plan() -> ValidatedMaturityAnnouncementOperationPlan {
        crate::maturity_closure::announcement_plan().expect("the announcement plan derives")
    }

    /// Whether a relation exists only where a sponsor envelope does.
    ///
    /// The sponsor family's own relations, and not the relations subjected
    /// to the sponsor envelope: the isolation and envelope-count relations
    /// are activated in every case and quantify over whatever the observed
    /// transaction carries, so an inactive answer from one of them would
    /// say nothing about a sponsor.
    fn sponsor_conditional(relation: &RelationId) -> bool {
        matches!(
            relation.subject(),
            RelationSubject::ObjectFamily {
                object: ObjectId::PlainLbtc,
                ..
            }
        )
    }

    /// Every declaration the rows carry, deduplicated.
    fn declarations() -> BTreeSet<(RelationId, MaturityMutationClass)> {
        rows()
            .iter()
            .filter_map(|row| match row.relation() {
                Standing::Declared { relation, class } => Some((relation.clone(), *class)),
                Standing::EveryPublishedRelation | Standing::Unlinked(_) => None,
            })
            .collect()
    }

    #[test]
    fn every_declared_row_resolves_in_the_sponsorless_case_of_every_representation() {
        let plan = announcement_plan();
        let mut representations = 0_usize;
        for projection in plan.representations() {
            representations += 1;
            let mut resolved = 0_usize;
            let mut inactive: BTreeSet<&str> = BTreeSet::new();
            for row in rows() {
                let Standing::Declared { relation, class } = row.relation() else {
                    continue;
                };
                let link = resolve_row(&plan, row, projection.plan(), SponsorCase::Absent)
                    .unwrap_or_else(|error| {
                        panic!(
                            "{row} names {} and the plan disagrees: {error:?}",
                            class.name(),
                        )
                    });
                match link {
                    MaturityRowLink::Resolved(id) => {
                        assert_eq!(&id.relation, relation, "{row} resolved to another relation");
                        resolved += 1;
                    }
                    // A relation about this operation's own positions has
                    // no honest inactive answer: the plan requires its
                    // negatives in every case it publishes, so nothing
                    // published is the row and the plan disagreeing rather
                    // than a disposition the case decides.
                    MaturityRowLink::InactiveInThisCase => {
                        assert!(
                            sponsor_conditional(relation),
                            "{row} resolves nowhere and its relation is not a sponsor family's",
                        );
                        inactive.insert(row.name());
                    }
                    MaturityRowLink::EveryRelation | MaturityRowLink::Blocked(_) => {
                        panic!("{row} is declared and answered as though it were not")
                    }
                }
            }
            // The hundred and two declared rows, less the one naming the
            // sponsor family's recognition, which a sponsorless case does
            // not exercise at all.
            assert_eq!(
                resolved,
                101,
                "{:?} resolves a different census",
                projection.plan()
            );
            assert_eq!(inactive.len(), 1);
        }
        assert_eq!(representations, 2, "both admitted modes are read");
    }

    #[test]
    fn every_resolved_requirement_is_the_negative_the_row_named() {
        let plan = announcement_plan();
        for projection in plan.representations() {
            for row in rows() {
                let Standing::Declared { relation, class } = row.relation() else {
                    continue;
                };
                let link = resolve_row(&plan, row, projection.plan(), SponsorCase::Absent)
                    .expect("every declared row resolves");
                let MaturityRowLink::Resolved(id) = link else {
                    continue;
                };
                // Read back out of the plan by the identity the resolution
                // returned, so a resolver answering with an identity the
                // plan does not publish is caught here rather than
                // believed by every later consumer.
                let requirement = projection
                    .coverage_requirement(&id)
                    .expect("the resolved identity is published");
                let TargetCoverageObligation::Negative(negative) = &requirement.obligation else {
                    panic!("{row} resolved to an obligation that requires no rejection")
                };
                assert_eq!(requirement.id, id, "{row} read back another requirement");
                assert_eq!(&id.relation, relation, "{row} answered another relation");
                assert_eq!(id.case.sponsor, SponsorCase::Absent);
                assert_eq!(
                    id.boundary,
                    class.boundary(),
                    "{row} answered at another boundary"
                );
                assert!(
                    class.matches(&negative.mutation),
                    "{row} answered another mutation class",
                );
            }
        }
    }

    #[test]
    fn two_rows_share_a_requirement_only_by_sharing_their_declaration() {
        // Eighteen distinct declarations stand on the hundred and two
        // declared rows, because §16 states many wordings of one published
        // class: sixteen root-history rows are sixteen ways to disturb the
        // one root effect the operation declares. Rows sharing a
        // requirement is therefore the ordinary case, and the claim worth
        // making is the other one — that the resolution distinguishes rows
        // exactly as far as their declarations do, so two rows meeting at
        // one requirement have met by declaring the same thing.
        assert_eq!(declarations().len(), 18);
        let plan = announcement_plan();
        for projection in plan.representations() {
            let mut by_requirement: BTreeMap<
                CoverageRequirementId,
                BTreeSet<(RelationId, MaturityMutationClass)>,
            > = BTreeMap::new();
            for row in rows() {
                let Standing::Declared { relation, class } = row.relation() else {
                    continue;
                };
                let link = resolve_row(&plan, row, projection.plan(), SponsorCase::Absent)
                    .expect("every declared row resolves");
                if let MaturityRowLink::Resolved(id) = link {
                    by_requirement
                        .entry(id)
                        .or_default()
                        .insert((relation.clone(), *class));
                }
            }
            for (id, declared) in &by_requirement {
                assert_eq!(
                    declared.len(),
                    1,
                    "one requirement of {:?} answers two declarations",
                    id.relation,
                );
            }
            // Seventeen of the eighteen resolve here; the eighteenth is
            // the sponsor family's recognition, inactive in this case.
            assert_eq!(by_requirement.len(), 17);
        }
    }

    #[test]
    fn the_whole_transaction_scoped_rows_are_exactly_the_recounted_set() {
        let scoped: Vec<&MaturitySafetyRow> = rows()
            .iter()
            .filter(|row| {
                matches!(
                    row.relation(),
                    Standing::Unlinked(Why::RelationScopedToTheWholeTransaction)
                )
            })
            .collect();
        // Six report-layer metadata rows against the canonical delta, one
        // absence row against the open-flow policy, one ABI row and nine
        // sponsor rows against the isolation, the envelope count and the
        // sponsor's own authorization. Each names a relation the
        // realization evaluates over the whole observed transaction, and
        // none of them is a row this workspace could link by choosing a
        // nearer published name.
        let mut per_section: BTreeMap<MaturitySafetySection, usize> = BTreeMap::new();
        for row in &scoped {
            *per_section.entry(row.section()).or_insert(0) += 1;
        }
        assert_eq!(scoped.len(), 17);
        assert_eq!(per_section[&S::MetadataFault], 6);
        assert_eq!(per_section[&S::AbsenceFault], 1);
        assert_eq!(per_section[&S::SponsorFault], 9);
        assert_eq!(per_section[&S::AbiLinkerFault], 1);
        assert_eq!(per_section.len(), 4);
        // The three reasons that stood before stand on the same rows, so a
        // row was re-standinged and none was reclassified: a relation
        // standing is not a boundary and the census does not move.
        let reason = |wanted: Why| {
            rows()
                .iter()
                .filter(|row| row.relation() == &Standing::Unlinked(wanted))
                .count()
        };
        assert_eq!(reason(Why::NoPublishedRelationNamesIt), 55);
        assert_eq!(reason(Why::SeveralPublishedRelationsFit), 9);
        assert_eq!(reason(Why::TheReportIsTheSubject), 9);
        // And the three standings partition the matrix.
        let positive = rows()
            .iter()
            .filter(|row| row.relation() == &Standing::EveryPublishedRelation)
            .count();
        let declared = rows()
            .iter()
            .filter(|row| matches!(row.relation(), Standing::Declared { .. }))
            .count();
        let unlinked = rows()
            .iter()
            .filter(|row| matches!(row.relation(), Standing::Unlinked(_)))
            .count();
        assert_eq!(positive, 14);
        assert_eq!(declared, 102);
        assert_eq!(unlinked, 90);
        assert_eq!(positive + declared + unlinked, row_count());
    }

    #[test]
    fn every_class_stands_on_a_row_and_on_a_published_requirement() {
        let plan = announcement_plan();
        let names: BTreeSet<&str> = MaturityMutationClass::ALL
            .iter()
            .map(|class| class.name())
            .collect();
        assert_eq!(names.len(), MaturityMutationClass::ALL.len());
        for class in MaturityMutationClass::ALL {
            // A class no row names is a vocabulary entry standing on
            // nothing, and a class the plan publishes nowhere is a row's
            // private invention. Both are caught here rather than by a
            // reader comparing two files.
            assert!(
                declarations().iter().any(|(_, stated)| stated == class),
                "{} stands on no row",
                class.name(),
            );
            let published = plan.representations().any(|projection| {
                projection.coverage().any(|requirement| {
                    let TargetCoverageObligation::Negative(negative) = &requirement.obligation
                    else {
                        return false;
                    };
                    requirement.id.boundary == class.boundary() && class.matches(&negative.mutation)
                })
            });
            assert!(published, "the plan publishes no {}", class.name());
        }
    }
}
