//! The complete required safety matrix of Guide-13 §15.
//!
//! §15 names every class the live-transfer safety report must carry, and
//! requires the implementation to transcribe the complete matrix into
//! typed classes with polarity, mutation layer, expected boundary,
//! intended relation, and collateral. This module is that transcription:
//! one row per named class, in the guide's own order, carrying exactly
//! those five facts and nothing observed.
//!
//! # Why the expected boundary is written down before anything runs
//!
//! §1.11 keeps construction failure, infrastructure failure, and target
//! rejection distinct, and forbids inferring a failure layer from the
//! expected result. A row that did not say in advance where its verdict
//! belongs could be "passed" by a refusal at any layer at all — a
//! signature the builder rejected, an executor that never started, a
//! target that refused for an unrelated reason. Naming the boundary here,
//! in a file that holds no bytes and observes nothing, is what makes the
//! later waves falsifiable instead of self-confirming.
//!
//! # Most rows name no relation, and say why
//!
//! §15's tables are lists of names. No row there names a semantic
//! relation, a mutation class, or a carrier, so for most rows the guide
//! does not determine a coverage requirement at all. §4.1 is explicit
//! about what happens then: an underdetermined vector stays blocked and
//! does not receive a guessed relation. The rows this module does link
//! are the ones whose §15 wording reads directly in the published
//! relation vocabulary, and every link is *resolved* against the plan by
//! [`resolve_row`] rather than tabulated here, so a link that stopped
//! being true fails loudly instead of ageing quietly.
//!
//! # Nothing here is evidence
//!
//! This module carries no fixture, no bundle, no transaction, no verdict,
//! and no report. It is a specification transcribed into types. What a
//! row's boundary actually did is [`crate::live_first_party`]'s question
//! for the first-party half and a target-native run's for the rest.

use std::collections::BTreeSet;

use architecture::{AssetId, ObjectId, OperationId};
use compiler::live_transfer_plan::{
    LiveTransferRepresentationPlan, ValidatedLiveTransferOperationPlan,
};
use compiler::operation_plan::{
    CollateralPolicy, CoverageRequirementId, EvidenceRole, RelationMutation, SponsorCase,
    TargetCoverageObligation,
};
use realization::{RelationId, RelationKind, RelationSubject, TransactionSide};

use crate::error::VectorError;
use crate::matrix::{EvidenceBoundary, MutationLayer};

/// The §15 table one row is drawn from.
///
/// The seven sections partition the matrix exactly: every row belongs to
/// one table, and the per-section counts recomputed in this module's
/// tests must sum to the whole.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LiveSafetySection {
    /// §15.1 — positive explicit classes.
    PositiveExplicit,
    /// §15.2 — positive private classes.
    PositivePrivate,
    /// §15.3 — owner and signature faults.
    OwnerSignatureFault,
    /// §15.4 — class, asset, and constructor faults.
    ObjectFault,
    /// §15.5 — value and partition faults.
    ValueFault,
    /// §15.6 — sponsor faults.
    SponsorFault,
    /// §15.7 — root, event, ABI, and linker faults.
    StructuralFault,
}

impl LiveSafetySection {
    /// Every section, in §15 order.
    pub const ALL: &'static [Self] = &[
        Self::PositiveExplicit,
        Self::PositivePrivate,
        Self::OwnerSignatureFault,
        Self::ObjectFault,
        Self::ValueFault,
        Self::SponsorFault,
        Self::StructuralFault,
    ];

    /// The guide section this table is stated in.
    #[must_use]
    pub const fn section(self) -> &'static str {
        match self {
            Self::PositiveExplicit => "15.1",
            Self::PositivePrivate => "15.2",
            Self::OwnerSignatureFault => "15.3",
            Self::ObjectFault => "15.4",
            Self::ValueFault => "15.5",
            Self::SponsorFault => "15.6",
            Self::StructuralFault => "15.7",
        }
    }

    /// Whether every row of this table expects the target to accept.
    #[must_use]
    pub const fn is_positive(self) -> bool {
        matches!(self, Self::PositiveExplicit | Self::PositivePrivate)
    }
}

/// What a row expects its boundary to establish.
///
/// Two members and no third. §15 is a matrix of valid transfers that must
/// pass and invalid ones that must fail; a row whose subject were the
/// executor would belong to §1.11's infrastructure layer, which this
/// matrix does not name and this module therefore does not invent.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LiveSafetyPolarity {
    /// The complete transfer is expected to be accepted, and its
    /// projection to equal the expected live-transfer projection.
    Positive,
    /// The transfer is expected to be refused at the row's stated
    /// boundary.
    Negative,
}

impl LiveSafetyPolarity {
    /// Both polarities.
    pub const ALL: &'static [Self] = &[Self::Positive, Self::Negative];
}

/// Why one §15 row names no relation-indexed requirement.
///
/// Four distinct situations, kept apart because they call for four
/// different repairs and only some of them are this workspace's to make.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum LiveUnlinkedReason {
    /// The row expects its refusal before any target sees the bytes.
    ///
    /// Every negative requirement the live plan publishes at the runtime
    /// carrier is answerable by a submitted transaction. A row refused by
    /// the request validator, the constructor, the linker, or the signing
    /// flow has no such row to answer, because its boundary is not among
    /// the four the plan indexes.
    BoundaryPrecedesTarget,
    /// The compiler's negative vocabulary describes no such change.
    ///
    /// §15 asks for the class and the published relation inventory has
    /// nothing to index it by. That is a gap between two authorities
    /// rather than a defect in either, and it is reported rather than
    /// closed by filing the row under whichever mutation looked nearest.
    NoSemanticMutationClass,
    /// Several published mutation classes fit the guide's wording.
    ///
    /// §15 names the row and nothing narrows it to one requirement, so
    /// picking one would be the discharge-by-intent the census exists to
    /// prevent.
    SemanticClassUnderdetermined,
    /// The row's subject is a report, not a transaction.
    ///
    /// §15.6 asks whether the *report* publishes a sponsor amount or a
    /// sponsor opening. No target can answer that and no relation is
    /// indexed by it: the boundary is this workspace's own canonical
    /// serialization, and the evidence is a property of the bytes a
    /// report renders rather than of anything a transfer did.
    ReportIsTheBoundary,
}

/// What one §15 row intends to violate, or to preserve.
///
/// The mutation class travels as a predicate rather than a value, for the
/// reason [`crate::violation::IntendedViolation`] states: an
/// above-maximum mutation carries the ceiling it must exceed, and that
/// ceiling is the architecture's to state.
#[derive(Clone, Debug)]
pub enum LiveRelationStanding {
    /// The row intends every published relation of its case to hold.
    ///
    /// The honest standing of a positive class. §15.1 and §15.2 name
    /// transaction *shapes* — a split, a merge, a repeated owner — and a
    /// shape is not a relation: what a valid split must preserve is the
    /// whole relation census, not one member of it. A positive row
    /// naming a single relation would be claiming the others were not its
    /// business.
    EveryPublishedRelation,
    /// The row names one relation and one semantic mutation class.
    Declared {
        /// The relation the change is intended to violate.
        relation: RelationId,
        /// Whether one published mutation is the class this row stages.
        class: fn(&RelationMutation) -> bool,
        /// That class's name, for reports and for the census.
        class_name: &'static str,
    },
    /// The row names no requirement, for a stated reason.
    Unlinked(LiveUnlinkedReason),
}

impl PartialEq for LiveRelationStanding {
    /// Compared by what a reader can check, never by function address.
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::EveryPublishedRelation, Self::EveryPublishedRelation) => true,
            (
                Self::Declared {
                    relation: left,
                    class_name: left_name,
                    ..
                },
                Self::Declared {
                    relation: right,
                    class_name: right_name,
                    ..
                },
            ) => left == right && left_name == right_name,
            (Self::Unlinked(left), Self::Unlinked(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for LiveRelationStanding {}

/// Where one §15 row's verdict comes from.
///
/// Two members, because §15 names one class no layer answers. Most rows
/// expect a layer to accept or refuse, and [`Self::Layer`] carries which.
///
/// # Why a row may name no layer at all
///
/// §4.2 discharges a negative row with a canonical malformed typed input
/// offered to its owning validator. That presupposes the input exists.
/// §15.4's `mixed-operation-program` asks for a program mixing this
/// operation with another, and the architecture admits no value naming
/// one: compiler projection fixes the operation to `TransferLive` before
/// a program is planned, the leaf-role vocabulary has none but transfer
/// roles, the constructor and the linker consume only that role type,
/// and the request cannot select a program at all. There is nothing to
/// offer any layer, so no layer produces a verdict.
///
/// Declaring a refusal boundary anyway would be worse than saying
/// nothing: a row that named the linker could be "passed" by a linker
/// refusal for an unrelated reason, and the layer named would never have
/// been asked the row's question.
///
/// # The second member is deliberately specific
///
/// It names *this* closure rather than typed inexpressibility in
/// general. A general standing would need a proof contract identifying
/// the exact closed type, every construction route, and every package
/// crossing — and without one, "inexpressible" could launder a missing
/// validator into evidence. A row closed by some other vocabulary gets
/// its own member and its own argument.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum LiveRowBoundary {
    /// One §1.11 layer produces the verdict.
    Layer(EvidenceBoundary),
    /// No layer does: the operation vocabulary admits no such input.
    OperationVocabularyClosure,
}

impl LiveRowBoundary {
    /// The layer that produces the verdict, where one does.
    #[must_use]
    pub const fn layer(self) -> Option<EvidenceBoundary> {
        match self {
            Self::Layer(boundary) => Some(boundary),
            Self::OperationVocabularyClosure => None,
        }
    }
}

/// One row of the §15 required safety matrix.
///
/// The name is the guide's own wording normalized to an identifier, so a
/// reader can check the transcription against §15 line by line without
/// trusting a paraphrase.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveSafetyRow {
    section: LiveSafetySection,
    name: &'static str,
    polarity: LiveSafetyPolarity,
    mutation: Option<MutationLayer>,
    boundary: LiveRowBoundary,
    relation: LiveRelationStanding,
    collateral: Option<CollateralPolicy>,
}

impl LiveSafetyRow {
    /// The §15 table this row comes from.
    #[must_use]
    pub const fn section(&self) -> LiveSafetySection {
        self.section
    }

    /// The row's stable name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// What the row expects its boundary to establish.
    #[must_use]
    pub const fn polarity(&self) -> LiveSafetyPolarity {
        self.polarity
    }

    /// The layer the row's mutation is applied at, if it mutates.
    ///
    /// `None` for a positive row, which mutates nothing, and for a row
    /// whose fault no typed input expresses, which has no change to
    /// apply anywhere.
    #[must_use]
    pub const fn mutation(&self) -> Option<MutationLayer> {
        self.mutation
    }

    /// Where the row expects its verdict to come from.
    #[must_use]
    pub const fn boundary(&self) -> LiveRowBoundary {
        self.boundary
    }

    /// The §1.11 layer the row expects to produce its verdict.
    ///
    /// `None` for the row no layer answers, which is the whole reason
    /// [`LiveRowBoundary`] exists.
    #[must_use]
    pub const fn refusing_layer(&self) -> Option<EvidenceBoundary> {
        self.boundary.layer()
    }

    /// The relation the row intends to violate, or the reason there is
    /// none.
    #[must_use]
    pub const fn relation(&self) -> &LiveRelationStanding {
        &self.relation
    }

    /// The dependency collateral a linked row expects its requirement to
    /// demand.
    ///
    /// `None` for a row that links to no requirement, which has nothing
    /// to demand it of.
    #[must_use]
    pub const fn collateral(&self) -> Option<CollateralPolicy> {
        self.collateral
    }

    /// Whether this row's boundary is one first-party code owns.
    ///
    /// The partition the evidence plan is built on: a row refused before
    /// the target sees anything is answered by driving the exact owning
    /// validator, and a row whose boundary is the target is answered by a
    /// run and by nothing else (§4.2, §4.3).
    #[must_use]
    pub const fn is_first_party(&self) -> bool {
        match self.boundary {
            LiveRowBoundary::Layer(boundary) => boundary.is_pre_target(),
            // No layer is asked, so no first-party validator owns a
            // refusal §4.2 could ask for.
            LiveRowBoundary::OperationVocabularyClosure => false,
        }
    }
}

impl core::fmt::Display for LiveSafetyRow {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "§{}:{}", self.section.section(), self.name)
    }
}

use EvidenceBoundary as B;
use LiveSafetyPolarity as P;
use LiveSafetySection as S;
use MutationLayer as L;

/// One live-transfer relation of the given kind and subject.
const fn relation(kind: RelationKind, subject: RelationSubject) -> RelationId {
    RelationId::new(OperationId::TransferLive, kind, subject)
}

/// The input-side live-receipt owner-authorization relation.
const fn owner_authorization() -> RelationId {
    relation(
        RelationKind::Authorization,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
    )
}

/// One side's object-family recognition relation.
const fn recognition(side: TransactionSide, object: ObjectId) -> RelationId {
    relation(
        RelationKind::Recognition,
        RelationSubject::ObjectFamily { side, object },
    )
}

/// One side's allowed-object-family closure.
const fn allowed_families(side: TransactionSide) -> RelationId {
    relation(
        RelationKind::AllowedObjectFamilies,
        RelationSubject::TransactionSide { side },
    )
}

/// The closed asset's conservation relation.
const fn conservation() -> RelationId {
    relation(
        RelationKind::Conservation,
        RelationSubject::Asset { asset: AssetId::U },
    )
}

/// The operation's canonical-partition policy.
const fn delta_policy() -> RelationId {
    relation(
        RelationKind::CanonicalDeltaPolicy,
        RelationSubject::Operation,
    )
}

/// The sponsor-isolation relation.
const fn sponsor_isolation() -> RelationId {
    relation(RelationKind::SponsorIsolation, RelationSubject::Sponsor)
}

/// One positive row: accepted, mutating nothing, preserving everything.
const fn positive(section: LiveSafetySection, name: &'static str) -> LiveSafetyRow {
    LiveSafetyRow {
        section,
        name,
        polarity: P::Positive,
        mutation: None,
        boundary: LiveRowBoundary::Layer(B::AcceptedTransaction),
        relation: LiveRelationStanding::EveryPublishedRelation,
        collateral: None,
    }
}

/// One negative row whose §15 wording determines a published
/// requirement.
const fn linked(
    section: LiveSafetySection,
    name: &'static str,
    mutation: MutationLayer,
    boundary: EvidenceBoundary,
    relation: RelationId,
    class: fn(&RelationMutation) -> bool,
    class_name: &'static str,
) -> LiveSafetyRow {
    LiveSafetyRow {
        section,
        name,
        polarity: P::Negative,
        mutation: Some(mutation),
        boundary: LiveRowBoundary::Layer(boundary),
        relation: LiveRelationStanding::Declared {
            relation,
            class,
            class_name,
        },
        // Every negative requirement the live plan publishes demands the
        // intended relation and its typed dependency closure be reported
        // blocked. The closure itself is not restated here: it travels
        // with the requirement, and copying it would be a third source of
        // truth.
        collateral: Some(CollateralPolicy::RequireIntendedAndDependencyClosure),
    }
}

/// One negative row the guide does not determine a requirement for.
const fn unlinked(
    section: LiveSafetySection,
    name: &'static str,
    mutation: MutationLayer,
    boundary: EvidenceBoundary,
    reason: LiveUnlinkedReason,
) -> LiveSafetyRow {
    LiveSafetyRow {
        section,
        name,
        polarity: P::Negative,
        mutation: Some(mutation),
        boundary: LiveRowBoundary::Layer(boundary),
        relation: LiveRelationStanding::Unlinked(reason),
        collateral: None,
    }
}

/// One negative row no layer answers, because no input expresses it.
///
/// It carries no mutation layer for the same reason it carries no
/// boundary: there is no change to apply. What establishes the closure
/// is the architecture rather than a staged refusal, and
/// [`crate::live_evidence`] records it as a standing of its own rather
/// than as evidence.
const fn vocabulary_closure(
    section: LiveSafetySection,
    name: &'static str,
    reason: LiveUnlinkedReason,
) -> LiveSafetyRow {
    LiveSafetyRow {
        section,
        name,
        polarity: P::Negative,
        mutation: None,
        boundary: LiveRowBoundary::OperationVocabularyClosure,
        relation: LiveRelationStanding::Unlinked(reason),
        collateral: None,
    }
}

/// A row refused before the target sees the bytes.
const fn pre_target(
    section: LiveSafetySection,
    name: &'static str,
    mutation: MutationLayer,
    boundary: EvidenceBoundary,
) -> LiveSafetyRow {
    unlinked(
        section,
        name,
        mutation,
        boundary,
        LiveUnlinkedReason::BoundaryPrecedesTarget,
    )
}

/// A row whose subject no published mutation class describes.
const fn no_class(
    section: LiveSafetySection,
    name: &'static str,
    mutation: MutationLayer,
    boundary: EvidenceBoundary,
) -> LiveSafetyRow {
    unlinked(
        section,
        name,
        mutation,
        boundary,
        LiveUnlinkedReason::NoSemanticMutationClass,
    )
}

/// A row several published mutation classes fit equally well.
const fn ambiguous(
    section: LiveSafetySection,
    name: &'static str,
    mutation: MutationLayer,
    boundary: EvidenceBoundary,
) -> LiveSafetyRow {
    unlinked(
        section,
        name,
        mutation,
        boundary,
        LiveUnlinkedReason::SemanticClassUnderdetermined,
    )
}

/// The `MissingRequiredOwner` predicate, spelled once.
const fn missing_owner(mutation: &RelationMutation) -> bool {
    matches!(mutation, RelationMutation::MissingRequiredOwner)
}

/// The `WrongRecognizedObject` predicate, spelled once.
const fn wrong_object(mutation: &RelationMutation) -> bool {
    matches!(mutation, RelationMutation::WrongRecognizedObject)
}

/// The `WrongRecognizedAsset` predicate, spelled once.
const fn wrong_asset(mutation: &RelationMutation) -> bool {
    matches!(mutation, RelationMutation::WrongRecognizedAsset)
}

/// The `UndeclaredObjectFamily` predicate, spelled once.
const fn undeclared_family(mutation: &RelationMutation) -> bool {
    matches!(mutation, RelationMutation::UndeclaredObjectFamily)
}

/// The `AmountMismatch` predicate, spelled once.
const fn amount_mismatch(mutation: &RelationMutation) -> bool {
    matches!(mutation, RelationMutation::AmountMismatch)
}

/// The `DuplicateCanonicalSourceOrDestination` predicate, spelled once.
const fn duplicate_endpoint(mutation: &RelationMutation) -> bool {
    matches!(
        mutation,
        RelationMutation::DuplicateCanonicalSourceOrDestination
    )
}

/// The `UnexpectedCanonicalDeltaFamily` predicate, spelled once.
const fn unexpected_family(mutation: &RelationMutation) -> bool {
    matches!(mutation, RelationMutation::UnexpectedCanonicalDeltaFamily)
}

/// The `MissingCanonicalDeltaFamily` predicate, spelled once.
const fn missing_family(mutation: &RelationMutation) -> bool {
    matches!(mutation, RelationMutation::MissingCanonicalDeltaFamily)
}

/// The `UndeclaredOpenFlow` predicate, spelled once.
const fn undeclared_open_flow(mutation: &RelationMutation) -> bool {
    matches!(mutation, RelationMutation::UndeclaredOpenFlow)
}

/// The `SponsorProtocolOverlap` predicate, spelled once.
const fn sponsor_overlap(mutation: &RelationMutation) -> bool {
    matches!(mutation, RelationMutation::SponsorProtocolOverlap)
}

/// The `MissingSponsorAuthorization` predicate, spelled once.
const fn missing_sponsor_authorization(mutation: &RelationMutation) -> bool {
    matches!(mutation, RelationMutation::MissingSponsorAuthorization)
}

/// The `SponsorEnvelopeMultiplicityExceeded` predicate, spelled once.
const fn envelope_multiplicity(mutation: &RelationMutation) -> bool {
    matches!(
        mutation,
        RelationMutation::SponsorEnvelopeMultiplicityExceeded
    )
}

/// The `ExternalEvidenceFailed` predicate, spelled once.
const fn external_evidence_failed(mutation: &RelationMutation) -> bool {
    matches!(mutation, RelationMutation::ExternalEvidenceFailed)
}

/// The `WrongRootEffect` predicate, spelled once.
const fn wrong_root_effect(mutation: &RelationMutation) -> bool {
    matches!(mutation, RelationMutation::WrongRootEffect)
}

/// The `ForbiddenProjectionPresent` predicate, spelled once.
const fn forbidden_projection(mutation: &RelationMutation) -> bool {
    matches!(mutation, RelationMutation::ForbiddenProjectionPresent)
}

/// The `MissingRequiredProjection` predicate, spelled once.
const fn missing_projection(mutation: &RelationMutation) -> bool {
    matches!(mutation, RelationMutation::MissingRequiredProjection)
}

/// §15.1 — the sixteen positive explicit classes.
///
/// Every one of them expects [`EvidenceBoundary::AcceptedTransaction`],
/// which is the invariant this table is checked for: a positive class
/// whose boundary were a refusal would be a contradiction in terms, and
/// §1.4's second verdict — the independent semantic-projection comparison
/// every acceptance receives — is a condition on top of the acceptance
/// rather than a different boundary.
pub const POSITIVE_EXPLICIT: &[LiveSafetyRow] = &[
    positive(S::PositiveExplicit, "one-input-to-one-output"),
    positive(S::PositiveExplicit, "one-input-split-into-two"),
    positive(S::PositiveExplicit, "several-inputs-merged-into-one"),
    positive(S::PositiveExplicit, "several-inputs-to-several-outputs"),
    positive(S::PositiveExplicit, "repeated-owner"),
    positive(S::PositiveExplicit, "several-distinct-owners"),
    positive(S::PositiveExplicit, "one-destination-owner"),
    positive(S::PositiveExplicit, "several-destination-owners"),
    positive(S::PositiveExplicit, "semantic-boundary-values"),
    positive(S::PositiveExplicit, "canonical-input-normalization"),
    positive(S::PositiveExplicit, "sponsorless"),
    positive(S::PositiveExplicit, "sponsored"),
    positive(S::PositiveExplicit, "sponsor-change-present"),
    positive(S::PositiveExplicit, "sponsor-change-absent"),
    positive(S::PositiveExplicit, "candidate-maximum-inputs"),
    positive(S::PositiveExplicit, "candidate-maximum-outputs"),
];

/// §15.2 — the ten positive private classes.
///
/// §13.4 makes each of these mandatory *for a claimed shape* and forbids
/// inferring any of them from malformed private transactions rejecting.
/// A shape this workspace does not claim is un-claimed with a typed
/// reason in the evidence plan; it is never quietly dropped from the
/// table here, because a matrix that shrank to fit what ran would report
/// completeness it does not have.
pub const POSITIVE_PRIVATE: &[LiveSafetyRow] = &[
    positive(S::PositivePrivate, "private-one-to-one"),
    positive(S::PositivePrivate, "private-split"),
    positive(S::PositivePrivate, "private-merge"),
    positive(S::PositivePrivate, "private-many-to-many-representative"),
    positive(S::PositivePrivate, "private-several-distinct-owners"),
    positive(S::PositivePrivate, "private-sponsor-values"),
    positive(S::PositivePrivate, "both-commitment-parity-forms"),
    positive(S::PositivePrivate, "deterministic-public-fixture-openings"),
    positive(S::PositivePrivate, "target-ct-conservation"),
    positive(
        S::PositivePrivate,
        "projection-equality-with-paired-explicit",
    ),
];

/// §15.3 — the fourteen owner and signature faults.
///
/// Ten of them are §12.7's rejection table, which the signing flow
/// already implements as typed refusals: a missing owner, a duplicated
/// response, an unexpected signer, the wrong owner, the wrong input, the
/// wrong sighash profile, a response bound to other bytes, and mutation
/// of an output, an input, or an omitted output after signing. Their
/// boundary is [`EvidenceBoundary::AbiConstructionRejection`] and their
/// evidence is a first-party refusal, not a target run.
///
/// The three witness-content rows change what the witness *offers*
/// rather than what the builder assembled. §10.2 types the signature
/// position as an unconstrained item precisely so that the target
/// refuses an empty or malformed offering; a schedule that ruled them
/// out would have answered the question the program is required to
/// answer.
pub const OWNER_SIGNATURE_FAULTS: &[LiveSafetyRow] = &[
    // The three rows whose §15 wording reads directly as the published
    // class: a required owner is not represented in the authorized set.
    // §12.7 refuses all three before any target sees them, so the link is
    // stated and the boundary is the builder's.
    linked(
        S::OwnerSignatureFault,
        "missing-owner-signature",
        L::WitnessProof,
        B::AbiConstructionRejection,
        owner_authorization(),
        missing_owner,
        "MissingRequiredOwner",
    ),
    pre_target(
        S::OwnerSignatureFault,
        "wrong-owner",
        L::WitnessProof,
        B::AbiConstructionRejection,
    ),
    linked(
        S::OwnerSignatureFault,
        "one-omitted-owner",
        L::WitnessProof,
        B::AbiConstructionRejection,
        owner_authorization(),
        missing_owner,
        "MissingRequiredOwner",
    ),
    pre_target(
        S::OwnerSignatureFault,
        "unrelated-signer",
        L::WitnessProof,
        B::AbiConstructionRejection,
    ),
    pre_target(
        S::OwnerSignatureFault,
        "duplicated-signature-substituted-for-another-owner",
        L::WitnessProof,
        B::AbiConstructionRejection,
    ),
    pre_target(
        S::OwnerSignatureFault,
        "wrong-sighash-profile",
        L::WitnessProof,
        B::AbiConstructionRejection,
    ),
    pre_target(
        S::OwnerSignatureFault,
        "signature-over-another-transaction",
        L::WitnessProof,
        B::AbiConstructionRejection,
    ),
    pre_target(
        S::OwnerSignatureFault,
        "output-changed-after-signing",
        L::TargetTransaction,
        B::AbiConstructionRejection,
    ),
    pre_target(
        S::OwnerSignatureFault,
        "input-added-after-signing",
        L::TargetTransaction,
        B::AbiConstructionRejection,
    ),
    pre_target(
        S::OwnerSignatureFault,
        "output-removed-after-signing",
        L::TargetTransaction,
        B::AbiConstructionRejection,
    ),
    // The owner-key encoding is authenticated by the constructor (§1.8,
    // §7.2), so an unknown key type never reaches a program: `OwnerKey`
    // refuses it against the approved encoding closure before a
    // constructor exists to place it in.
    pre_target(
        S::OwnerSignatureFault,
        "unknown-key-type",
        L::LinkedConstructorProgram,
        B::AbiConstructionRejection,
    ),
    no_class(
        S::OwnerSignatureFault,
        "empty-signature",
        L::WitnessProof,
        B::ScriptPathRejection,
    ),
    no_class(
        S::OwnerSignatureFault,
        "malformed-signature",
        L::WitnessProof,
        B::ScriptPathRejection,
    ),
    linked(
        S::OwnerSignatureFault,
        "repeated-owner-with-one-input-signature-omitted",
        L::WitnessProof,
        B::AbiConstructionRejection,
        owner_authorization(),
        missing_owner,
        "MissingRequiredOwner",
    ),
];

/// §15.4 — the sixteen class, asset, and constructor faults.
///
/// The table splits on §7.3's rule that live class is structural. A
/// mutation the constructor can express is refused by the constructor; a
/// mutation about what a *foreign output* carries is only expressible in
/// bytes, and the target's own introspection is what refuses it.
///
/// The split is not "a predecessor implies the target". Whether the
/// target refuses a predecessor depends on whether the covenant reads
/// the property in question, and for the receipt CLASS it does not: the
/// class is constructor typing, which emits nothing for a leaf to run.
/// The `time-locked-input` row was typed target-side on that inference
/// and is typed first-party here instead, beside the
/// `time-locked-output` row it is the sibling of.
pub const OBJECT_FAULTS: &[LiveSafetyRow] = &[
    // A time-locked predecessor is refused where its *output* sibling is,
    // and the row is typed as that sibling rather than as a target
    // introspection. §7.3 makes the two constructors distinct
    // and the distinction is carried by CONSTRUCTOR TYPING, which emits no
    // instructions: no live fragment introspects an input's program, and
    // the two receipt classes carry the same asset, so a live leaf that
    // ran would compare nothing that separates them. What separates them
    // before any leaf runs is the linked table the input recognition
    // searches — `compiler::live_transfer_plan::derive_class` admits only
    // `ReceiptLive` as the protocol object, so no time-locked constructor
    // can be in it — and, on a chain, the leaf commitment. The commitment
    // refusal is PROGRAM-GENERIC and may not be filed here: it attributes
    // to a leaf the spent program does not commit to, which is true of
    // every foreign taptree, so no observation of it can ever name the
    // lock. [`crate::live_fault_discharge`] discharges the row.
    pre_target(
        S::ObjectFault,
        "time-locked-input",
        L::LinkedConstructorProgram,
        B::AbiConstructionRejection,
    ),
    // A time-locked *output* is refused by the constructor: the
    // destination table selects among linked live constructors and has no
    // time-locked member to select.
    pre_target(
        S::ObjectFault,
        "time-locked-output",
        L::LinkedConstructorProgram,
        B::AbiConstructionRejection,
    ),
    // RETYPED FIRST-PARTY, on the ruling and on the `time-locked-input`
    // precedent exactly. The row was typed target-side, asking that an
    // ASH input offered to a live transfer be refused by something that
    // reads its family. Nothing on a chain does: what separates one
    // receipt family from another before any leaf runs is the LEAF
    // COMMITMENT, and a spend of a foreign program draws the identical
    // verdict every foreign taptree draws. THE COMMITMENT REFUSAL IS
    // PROGRAM-GENERIC AND CAN NEVER NAME THE FAMILY — the pinned target
    // source refuses a revealed leaf before a single opcode executes at
    // `src/script/interpreter.cpp:3286-3290`, where a failed
    // `VerifyTaprootCommitment` is `SCRIPT_ERR_WITNESS_PROGRAM_MISMATCH`
    // — so an observation of it establishes the commitment rule and says
    // nothing about families. No later wave is owed that observation and
    // producing it would not answer this row; it is written here so the
    // demand cannot be raised again.
    //
    // The structural protection is first-party and is driven: the input
    // recognition searches the linked destination table, an honest ASH
    // program is not in it, and the finalization answers naming the
    // class it required.
    pre_target(
        S::ObjectFault,
        "ash-input-or-output",
        L::LinkedConstructorProgram,
        B::AbiConstructionRejection,
    ),
    linked(
        S::ObjectFault,
        "vault-control-entitlement-or-bare-u-output",
        L::SemanticFact,
        B::ScriptPathRejection,
        allowed_families(TransactionSide::Output),
        undeclared_family,
        "UndeclaredObjectFamily",
    ),
    // Owner metadata is committed by the constructor, so replacing it
    // changes the program the output pays to. What reaches the target is a
    // spend of an output whose constructor is not the one the linked
    // bundle emits, which the input recognition refuses.
    linked(
        S::ObjectFault,
        "wrong-owner-metadata",
        L::SemanticFact,
        B::ScriptPathRejection,
        recognition(TransactionSide::Input, ObjectId::ReceiptLive),
        wrong_object,
        "WrongRecognizedObject",
    ),
    // RETYPED FIRST-PARTY, and this row is the one of the seven whose
    // refusal is NOT program-generic at all. Owner metadata is
    // authenticated by the constructor before a program exists to place
    // it in (§1.8, §7.2), exactly as the owner-key encoding is, so
    // malformed metadata is refused at the same entry point
    // `unknown-key-type` is refused at and names its own class: a width
    // the approved closure does not fix. Nothing reaches a chain to be
    // refused generically, which is why the row is here rather than
    // waiting on a run that would answer a different question.
    pre_target(
        S::ObjectFault,
        "malformed-live-metadata",
        L::LinkedConstructorProgram,
        B::AbiConstructionRejection,
    ),
    // §15.4's erratum, filed in the feature-request register. The row
    // arrived declaring `LinkerRejection`, and the linker neither derives
    // a leaf schema nor can be handed a malformed one: constructor and
    // bundle are sealed types whose only construction path has already
    // validated the schema. What owns it is tapscript's constructor
    // derivation, and the three malformations that reach it are a leaf of
    // the other representation, an absent required coordinator or member,
    // and a leaf serving no admitted shape. An empty leaf set is a
    // key-path escape and stays with that row.
    pre_target(
        S::ObjectFault,
        "wrong-constructor-schema",
        L::StaticConstructorSchema,
        B::ConstructorDerivationRejection,
    ),
    // The row no layer answers. A program mixing this operation with
    // another is not a malformed value this architecture can hold: the
    // compiler's projection fixes the operation before any program is
    // planned, and the leaf-role vocabulary the constructor and the
    // linker consume names transfer roles only. Foreign-operation
    // *effects* on a target are a different question, and §15.5's
    // issuance and destruction rows and §15.7's root and
    // specialized-event rows own it.
    vocabulary_closure(
        S::ObjectFault,
        "mixed-operation-program",
        LiveUnlinkedReason::NoSemanticMutationClass,
    ),
    // RETYPED FIRST-PARTY. A stale constructor is a different program
    // for the same owner, so what a chain answers is the COMMITMENT
    // rule — refused before a single opcode executes, in the identical
    // words every foreign taptree draws, naming nothing about staleness.
    // The pinned target source is `src/script/interpreter.cpp:3286-3290`,
    // where a failed `VerifyTaprootCommitment` is
    // `SCRIPT_ERR_WITNESS_PROGRAM_MISMATCH`. No later wave is owed that
    // observation.
    //
    // First-party the row is answerable and its mutant is honest: the
    // other shape vocabulary's constructor for the SAME owner and the
    // SAME representation, which the input recognition does not find in
    // this deployment's linked table.
    pre_target(
        S::ObjectFault,
        "stale-constructor",
        L::LinkedConstructorProgram,
        B::AbiConstructionRejection,
    ),
    linked(
        S::ObjectFault,
        "wrong-explicit-asset",
        L::SemanticFact,
        B::ScriptPathRejection,
        recognition(TransactionSide::Output, ObjectId::ReceiptLive),
        wrong_asset,
        "WrongRecognizedAsset",
    ),
    // §1.3: no confidential asset commitment may carry `U`. The asset
    // field is explicit under both representation plans, so a commitment
    // in it is the recognized asset being wrong.
    linked(
        S::ObjectFault,
        "confidential-asset-commitment",
        L::SemanticFact,
        B::ScriptPathRejection,
        recognition(TransactionSide::Output, ObjectId::ReceiptLive),
        wrong_asset,
        "WrongRecognizedAsset",
    ),
    linked(
        S::ObjectFault,
        "unclassified-u",
        L::SemanticFact,
        B::ScriptPathRejection,
        allowed_families(TransactionSide::Output),
        undeclared_family,
        "UndeclaredObjectFamily",
    ),
    linked(
        S::ObjectFault,
        "sponsor-or-fee-role-carrying-u",
        L::SemanticFact,
        B::ScriptPathRejection,
        allowed_families(TransactionSide::Output),
        undeclared_family,
        "UndeclaredObjectFamily",
    ),
    // RETYPED FIRST-PARTY, and the cleanest of the seven: the asset is
    // checked BEFORE the program lookup, so the refusal is attributable
    // to the asset alone rather than to the shape of the program. The
    // row was typed target-side on the reading that a receipt-shaped
    // program carrying a foreign asset reaches a leaf that compares
    // assets. It does not — the receipt-shaped program is not the
    // committed one, so the asset is never compared and the commitment
    // rule answers instead, program-generically. First-party, the honest
    // linked program is kept and ONLY the asset field is changed, which
    // is what makes the class the row's own.
    pre_target(
        S::ObjectFault,
        "foreign-asset-under-receipt-shaped-program",
        L::SemanticFact,
        B::AbiConstructionRejection,
    ),
    // §7.5 admits no key-path escape, and the internal key is the
    // published unspendable one. A key-path spend is expressible only in
    // raw bytes and the target is what refuses it.
    no_class(
        S::ObjectFault,
        "key-path-escape",
        L::WitnessProof,
        B::ScriptPathRejection,
    ),
    no_class(
        S::ObjectFault,
        "malformed-control-path",
        L::WitnessProof,
        B::ConsensusRejectionBeforeScript,
    ),
];

/// §15.5 — the twenty-two value and partition faults.
///
/// The largest table, and the one whose boundaries are most varied. The
/// explicit conservation rows reach the coordinator's own checked
/// arithmetic; the private rows reach target CT conservation, which §9.3
/// keeps as external target evidence; and the representation rows are
/// refused by the ABI, which admits one homogeneous plan at a time.
pub const VALUE_FAULTS: &[LiveSafetyRow] = &[
    linked(
        S::ValueFault,
        "output-total-one-below-input",
        L::SemanticFact,
        B::ScriptPathRejection,
        conservation(),
        amount_mismatch,
        "AmountMismatch",
    ),
    linked(
        S::ValueFault,
        "output-total-one-above-input",
        L::SemanticFact,
        B::ScriptPathRejection,
        conservation(),
        amount_mismatch,
        "AmountMismatch",
    ),
    // §5.1 fixes every semantic amount strictly positive, and the typed
    // request refuses a zero destination before a transaction exists.
    pre_target(
        S::ValueFault,
        "zero-receipt-output",
        L::SemanticFact,
        B::SemanticRequestRejection,
    ),
    // Owner ruling: the ceiling is blockchain-enforced, the same class as
    // conservation. The row arrived declaring a semantic-request
    // rejection, which read as a missing request-path validator; it is
    // not one. The protocol domain is `realization::ProtocolAmount`'s
    // exclusive limit of two to the fifty-first, and the reviewed target
    // refuses a stated amount above twenty-one million coins in
    // `CheckTransaction` — before any script runs. Every amount outside
    // the protocol domain is therefore above the target's own bound, and
    // the target is what refuses it. `ProtocolValue` deliberately gains
    // no new check: the ruling changes who enforces the ceiling, not
    // whether anything has observed the enforcement.
    no_class(
        S::ValueFault,
        "amount-outside-semantic-domain",
        L::SemanticFact,
        B::ConsensusRejectionBeforeScript,
    ),
    // §6.2 consumes every arithmetic success flag immediately, so an
    // overflowing total is refused where the sum is computed rather than
    // where it is spent.
    pre_target(
        S::ValueFault,
        "explicit-arithmetic-overflow",
        L::SemanticFact,
        B::AbiConstructionRejection,
    ),
    // §9.3: CT conservation is external target evidence, and the target
    // refuses an unbalanced confidential transaction before any script
    // runs.
    linked(
        S::ValueFault,
        "private-ct-imbalance",
        L::SemanticFact,
        B::ConsensusRejectionBeforeScript,
        conservation(),
        amount_mismatch,
        "AmountMismatch",
    ),
    linked(
        S::ValueFault,
        "wrong-private-blinding-balance",
        L::SemanticFact,
        B::ConsensusRejectionBeforeScript,
        conservation(),
        amount_mismatch,
        "AmountMismatch",
    ),
    no_class(
        S::ValueFault,
        "malformed-rangeproof",
        L::WitnessProof,
        B::ConsensusRejectionBeforeScript,
    ),
    no_class(
        S::ValueFault,
        "malformed-surjection-proof",
        L::WitnessProof,
        B::ConsensusRejectionBeforeScript,
    ),
    ambiguous(
        S::ValueFault,
        "copied-commitment",
        L::TargetTransaction,
        B::ConsensusRejectionBeforeScript,
    ),
    linked(
        S::ValueFault,
        "private-output-omitted",
        L::SemanticFact,
        B::ScriptPathRejection,
        delta_policy(),
        missing_family,
        "MissingCanonicalDeltaFamily",
    ),
    linked(
        S::ValueFault,
        "hidden-private-u-output",
        L::SemanticFact,
        B::ScriptPathRejection,
        delta_policy(),
        unexpected_family,
        "UnexpectedCanonicalDeltaFamily",
    ),
    // §6.5 admits one homogeneous plan at a time. Both representation
    // rows are refused by the ABI, which selects a destination
    // constructor per plan and has no mixed member.
    pre_target(
        S::ValueFault,
        "representation-mismatch",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    pre_target(
        S::ValueFault,
        "mixed-representation-under-homogeneous-only-abi",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    linked(
        S::ValueFault,
        "omitted-source",
        L::SemanticFact,
        B::ScriptPathRejection,
        delta_policy(),
        missing_family,
        "MissingCanonicalDeltaFamily",
    ),
    // RETYPED to the boundary it has. The row declared a script path,
    // and §12.1's request type refuses a repeated receipt outpoint
    // before sorting rather than collapsing it — earlier than an ABI,
    // earlier than a program lookup, earlier than a candidate. No run
    // was ever going to answer this row, because nothing carrying the
    // fault could be built to offer.
    pre_target(
        S::ValueFault,
        "duplicated-source",
        L::SemanticFact,
        B::AbiConstructionRejection,
    ),
    // THE SECOND ROW OF THIS MATRIX WHOSE PREDICTION THE SOURCES REFUSE,
    // and it sits beside its own opposite. `duplicated-source` above is
    // a real fault refused at the earliest boundary there is; this row
    // is NOT A FAULT AT ALL by the same type's own reading. §12.2 keeps
    // the destination census a MULTISET precisely so that two
    // destinations of one owner and one value count as two receipts
    // rather than one, and the deciding test builds exactly that pair
    // and calls it what it is — an even split is an ordinary transfer.
    //
    // The declaration is left standing and the STANDING is corrected,
    // for the reason the zero-valued sponsor row states: which rows
    // §15.5 lists is the guide's to say, and the erratum is filed
    // there rather than executed here.
    linked(
        S::ValueFault,
        "duplicated-destination",
        L::SemanticFact,
        B::ScriptPathRejection,
        delta_policy(),
        duplicate_endpoint,
        "DuplicateCanonicalSourceOrDestination",
    ),
    linked(
        S::ValueFault,
        "output-claimed-through-two-flows",
        L::SemanticFact,
        B::ScriptPathRejection,
        delta_policy(),
        duplicate_endpoint,
        "DuplicateCanonicalSourceOrDestination",
    ),
    linked(
        S::ValueFault,
        "issuance",
        L::SemanticFact,
        B::ScriptPathRejection,
        delta_policy(),
        unexpected_family,
        "UnexpectedCanonicalDeltaFamily",
    ),
    linked(
        S::ValueFault,
        "destruction",
        L::SemanticFact,
        B::ScriptPathRejection,
        delta_policy(),
        missing_family,
        "MissingCanonicalDeltaFamily",
    ),
    linked(
        S::ValueFault,
        "value-routed-into-ash-or-time-locked-receipt",
        L::SemanticFact,
        B::ScriptPathRejection,
        delta_policy(),
        unexpected_family,
        "UnexpectedCanonicalDeltaFamily",
    ),
    linked(
        S::ValueFault,
        "second-offsetting-u-flow",
        L::SemanticFact,
        B::ScriptPathRejection,
        relation(RelationKind::OpenFlowPolicy, RelationSubject::Operation),
        undeclared_open_flow,
        "UndeclaredOpenFlow",
    ),
];

/// §15.6 — the thirteen sponsor faults.
///
/// §1.9 is what shapes this table: protocol-owner signatures do not make
/// sponsor values protocol data, so the sponsor relation is membership,
/// disjointness, authorization, multiplicity, and whole-transaction
/// conservation — and never an amount. Two rows here are consequently not
/// about a transaction at all: they ask whether the *report* published a
/// sponsor amount or an opening, and their boundary is this workspace's
/// own canonical serialization.
///
/// The balanced-theft row is mandatory and is the table's sharpest: the
/// transaction must fail because the `U` transfer relation is wrong, not
/// because a sponsor amount failed a positivity check.
pub const SPONSOR_FAULTS: &[LiveSafetyRow] = &[
    linked(
        S::SponsorFault,
        "sponsor-protocol-overlap",
        L::AbiLayout,
        B::ScriptPathRejection,
        sponsor_isolation(),
        sponsor_overlap,
        "SponsorProtocolOverlap",
    ),
    linked(
        S::SponsorFault,
        "two-sponsor-envelopes",
        L::AbiLayout,
        B::ScriptPathRejection,
        relation(
            RelationKind::SponsorEnvelopeMultiplicity,
            RelationSubject::Sponsor,
        ),
        envelope_multiplicity,
        "SponsorEnvelopeMultiplicityExceeded",
    ),
    // A foreign sponsor asset leaves the reserve asset unbalanced, which
    // the target settles for the whole transaction and no script sees.
    linked(
        S::SponsorFault,
        "foreign-sponsor-asset",
        L::SemanticFact,
        B::ConsensusRejectionBeforeScript,
        relation(
            RelationKind::SubstrateConservation,
            RelationSubject::Asset {
                asset: AssetId::Lbtc,
            },
        ),
        external_evidence_failed,
        "ExternalEvidenceFailed",
    ),
    linked(
        S::SponsorFault,
        "missing-sponsor-authorization",
        L::WitnessProof,
        B::ScriptPathRejection,
        sponsor_isolation(),
        missing_sponsor_authorization,
        "MissingSponsorAuthorization",
    ),
    // §12.5's form exactness, refused by construction: an empty sponsor
    // capability never downgrades a sponsored request into the
    // sponsorless form.
    pre_target(
        S::SponsorFault,
        "empty-sponsor-offer-for-a-sponsored-request",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    linked(
        S::SponsorFault,
        "sponsor-change-in-protocol-range",
        L::AbiLayout,
        B::ScriptPathRejection,
        sponsor_isolation(),
        sponsor_overlap,
        "SponsorProtocolOverlap",
    ),
    pre_target(
        S::SponsorFault,
        "fee-change-substitution",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    ambiguous(
        S::SponsorFault,
        "sponsor-member-unclassified",
        L::AbiLayout,
        B::ScriptPathRejection,
    ),
    unlinked(
        S::SponsorFault,
        "report-publishes-sponsor-amount",
        L::AbiLayout,
        B::ReportSemanticProjectionRejection,
        LiveUnlinkedReason::ReportIsTheBoundary,
    ),
    unlinked(
        S::SponsorFault,
        "report-publishes-sponsor-opening",
        L::AbiLayout,
        B::ReportSemanticProjectionRejection,
        LiveUnlinkedReason::ReportIsTheBoundary,
    ),
    // RE-ATTRIBUTED, on the ruling. The row arrived declaring that
    // closed-asset conservation must refuse a balanced rearrangement,
    // and conservation is precisely the relation that CANNOT: the
    // fragment folds every destination into ONE sum and compares it
    // with the folded receipts, so a swap that preserves the total
    // passes it by construction. Attributing the row there was
    // attributing it to the one guard the fault is designed to slip
    // past.
    //
    // What actually stops it is the OWNER'S AUTHORIZATION OVER ALL
    // OUTPUTS. The authorization is bound to every output by position,
    // so moving value between two destinations is refused whatever it
    // does to the total — and that is a pre-target boundary, which is
    // why the row is here rather than waiting on a chain.
    //
    // On a chain the same theft would be refused by CHECKSIG, and that
    // observation is not owed for this row: a signature failure names a
    // signature and not a rearrangement, so it would not attribute.
    pre_target(
        S::SponsorFault,
        "balanced-theft",
        L::TargetTransaction,
        B::AbiConstructionRejection,
    ),
    // THE ONE ROW OF THIS MATRIX WHOSE PREDICTION THE SOURCES REFUSE.
    //
    // Its declaration below says a script path rejects a zero-valued
    // ordinary sponsor member. Nothing does, and the guide series had
    // already said so twice before this row was written: Guide 8 §22.6
    // rules the shape a SEMANTIC ACCEPTANCE under exact role structure
    // and instructs in the next line that no generic domain-failure
    // vector for it be preserved, and Guide 12 gives the layered
    // reading — semantic relation may accept, first-party builder omits
    // known zero change, deployment may reject as nonstandard. The
    // transcription faithfully carried a §15.6 fault-table ENTRY across
    // and did not carry the ruling that governs it.
    //
    // The declaration is left standing and the row's STANDING is
    // corrected instead, which is a deliberate division of ownership
    // rather than a half-measure. Two things prevent the honest repair
    // here: this matrix's tables are polarity-HOMOGENEOUS by
    // construction — `a_positive_row_mutates_nothing_and_expects_acceptance`
    // requires a row's polarity to equal its section's — so a
    // fault-table row cannot be flipped positive without moving it to
    // another table, and WHICH TABLE §15.6 LISTS IS THE GUIDE'S to say.
    // Deleting or moving the row here would be this workspace editing a
    // published matrix to agree with itself.
    //
    // So the erratum is filed with the guide, and the row is answered
    // at `LiveRowStanding::FirstPartyFactObserved` by the fact the
    // sources state, cited at that site to its deciding test. A reader
    // finding this declaration and that standing in disagreement is
    // seeing the erratum, not a row nobody checked.
    ambiguous(
        S::SponsorFault,
        "zero-valued-ordinary-sponsor-member",
        L::AbiLayout,
        B::ScriptPathRejection,
    ),
    ambiguous(
        S::SponsorFault,
        "confidential-sponsor-values",
        L::SemanticFact,
        B::ScriptPathRejection,
    ),
];

/// §15.7 — the seventeen root, event, ABI, and linker faults.
///
/// Mostly a first-party table. §11's linker refuses a duplicate leaf, an
/// unresolved relocation, and a wrong symbol; §12's ABI refuses a plan
/// paired with the other representation's program; and §4.3's
/// ABI-validation result is what distinguishes a safe constructor
/// refusing from an unsafe raw mutation existing, so the last row here is
/// about the existence of a bypass rather than about a verdict.
pub const STRUCTURAL_FAULTS: &[LiveSafetyRow] = &[
    linked(
        S::StructuralFault,
        "any-root-input-or-output",
        L::SemanticFact,
        B::ScriptPathRejection,
        relation(RelationKind::RootPolicy, RelationSubject::Operation),
        wrong_root_effect,
        "WrongRootEffect",
    ),
    linked(
        S::StructuralFault,
        "burn-record-or-specialized-event",
        L::SemanticFact,
        B::ScriptPathRejection,
        relation(RelationKind::ProjectionPolicy, RelationSubject::Operation),
        forbidden_projection,
        "ForbiddenProjectionPresent",
    ),
    linked(
        S::StructuralFault,
        "omitted-transition-certificate",
        L::SemanticFact,
        B::ScriptPathRejection,
        relation(RelationKind::ProjectionPolicy, RelationSubject::Operation),
        missing_projection,
        "MissingRequiredProjection",
    ),
    // §10.3 fixes input 0 as the coordinator and admits member leaves
    // only at nonzero receipt positions. All four leaf-arrangement rows
    // are expressible only in bytes: the ABI places the coordinator and
    // offers no way to ask for another arrangement.
    no_class(
        S::StructuralFault,
        "wrong-coordinator",
        L::WitnessProof,
        B::ScriptPathRejection,
    ),
    no_class(
        S::StructuralFault,
        "two-coordinators",
        L::WitnessProof,
        B::ScriptPathRejection,
    ),
    no_class(
        S::StructuralFault,
        "no-coordinator",
        L::WitnessProof,
        B::ScriptPathRejection,
    ),
    no_class(
        S::StructuralFault,
        "member-coordinator-leaf-exchange",
        L::WitnessProof,
        B::ScriptPathRejection,
    ),
    linked(
        S::StructuralFault,
        "receipt-sponsor-range-exchange",
        L::AbiLayout,
        B::ScriptPathRejection,
        sponsor_isolation(),
        sponsor_overlap,
        "SponsorProtocolOverlap",
    ),
    pre_target(
        S::StructuralFault,
        "destination-order-changed-after-signing",
        L::TargetTransaction,
        B::AbiConstructionRejection,
    ),
    no_class(
        S::StructuralFault,
        "witness-reorder",
        L::WitnessProof,
        B::ScriptPathRejection,
    ),
    // RETYPED FIRST-PARTY. On a chain a foreign control block draws the
    // verdict every foreign taptree draws — a failed
    // `VerifyTaprootCommitment` at the pinned target's
    // `src/script/interpreter.cpp:3286-3290`, which is
    // `SCRIPT_ERR_WITNESS_PROGRAM_MISMATCH` and names nothing about
    // control blocks. That observation would establish the commitment
    // rule and say nothing about this row, so no wave is owed it.
    //
    // First-party the row is answered PRECISELY, and by the only site in
    // this workspace that recomputes what the target recomputes: the
    // owner signing census folds each declared leaf hash up its offered
    // control block's path, tweaks the offered internal key and compares
    // the result with the program spent. Swapping the two receipts'
    // control blocks and changing nothing else leaves two real paths of
    // two real trees, neither committing to the input it is offered
    // for, and the refusal names the input.
    pre_target(
        S::StructuralFault,
        "control-block-from-another-program",
        L::WitnessProof,
        B::AbiConstructionRejection,
    ),
    pre_target(
        S::StructuralFault,
        "unresolved-or-duplicate-relocation",
        L::LinkedConstructorProgram,
        B::LinkerRejection,
    ),
    pre_target(
        S::StructuralFault,
        "wrong-u-or-owner-schema-relocation",
        L::LinkedConstructorProgram,
        B::LinkerRejection,
    ),
    pre_target(
        S::StructuralFault,
        "explicit-plan-paired-with-private-abi",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    pre_target(
        S::StructuralFault,
        "private-plan-paired-with-explicit-program",
        L::AbiLayout,
        B::AbiConstructionRejection,
    ),
    no_class(
        S::StructuralFault,
        "target-bytes-changed-after-abi-validation",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
    // §4.3's third distinction, and the reason it is a row rather than a
    // footnote: the safe constructor cannot express this, so its refusal
    // establishes nothing about it. What the row asks is whether a raw
    // path exists at all.
    no_class(
        S::StructuralFault,
        "raw-transaction-bypassing-safe-construction",
        L::TargetTransaction,
        B::ScriptPathRejection,
    ),
];

/// The seven §15 tables, in guide order.
pub const SECTIONS: &[(LiveSafetySection, &[LiveSafetyRow])] = &[
    (S::PositiveExplicit, POSITIVE_EXPLICIT),
    (S::PositivePrivate, POSITIVE_PRIVATE),
    (S::OwnerSignatureFault, OWNER_SIGNATURE_FAULTS),
    (S::ObjectFault, OBJECT_FAULTS),
    (S::ValueFault, VALUE_FAULTS),
    (S::SponsorFault, SPONSOR_FAULTS),
    (S::StructuralFault, STRUCTURAL_FAULTS),
];

/// The complete §15 matrix, in guide order.
#[must_use]
pub fn required_safety_matrix() -> Vec<&'static LiveSafetyRow> {
    SECTIONS.iter().flat_map(|(_, rows)| rows.iter()).collect()
}

/// Every row of one §15 section.
#[must_use]
pub fn rows_of(section: LiveSafetySection) -> &'static [LiveSafetyRow] {
    SECTIONS
        .iter()
        .find_map(|(named, rows)| (*named == section).then_some(*rows))
        .unwrap_or(&[])
}

/// How many rows each §15 section contributes.
#[must_use]
pub fn section_census() -> std::collections::BTreeMap<LiveSafetySection, usize> {
    SECTIONS
        .iter()
        .map(|(section, rows)| (*section, rows.len()))
        .collect()
}

/// How many rows the complete matrix has.
#[must_use]
pub fn row_count() -> usize {
    SECTIONS.iter().map(|(_, rows)| rows.len()).sum()
}

/// Which requirement one row names, once resolved against the plan.
///
/// Three answers and no fourth. A row either resolves to exactly one
/// published requirement, intends the whole census rather than one
/// member, or names none for a reason it states.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LiveRowLink {
    /// Exactly one published requirement matched.
    Resolved(CoverageRequirementId),
    /// The row intends every published relation of its case to hold.
    EveryRelation,
    /// The relation is not active in this representation and case.
    ///
    /// §19.4's conditional coverage, seen from the resolution's side.
    /// Explicit arithmetic is active under the explicit plan and inactive
    /// under the private one, where target CT conservation takes its
    /// place; a sponsor relation is active only where a sponsor envelope
    /// exists. A row whose relation is inactive has no requirement to
    /// answer *here*, and saying so is different from saying the row
    /// names no requirement at all.
    ///
    /// This is not a licence for a row to resolve nowhere.
    /// [`crate::live_evidence::derive_live_evidence_plan`] requires every
    /// declared row to resolve in at least one representation and case,
    /// so a link the plan stopped publishing altogether is still caught.
    InactiveInThisCase,
    /// The row names no requirement, for a stated reason.
    Blocked(LiveUnlinkedReason),
}

/// Resolve one §15 row against the published live-transfer plan.
///
/// The relation and the mutation class select the requirement, and the
/// resolution must find exactly one. A table mapping rows to requirement
/// identities would be a third source of truth that could agree with
/// neither side; a resolution that must hit exactly one row fails the
/// moment the plan moves underneath it.
///
/// # A pre-target row names the requirement it anticipates
///
/// The row's own boundary is deliberately not consulted. §4.3 keeps
/// three answers apart — the safe constructor refused, an unsafe raw
/// mutation exists, and the target was not asked — and the first of them
/// is a statement *about* a runtime requirement rather than a discharge
/// of it. A missing owner is a violation of the input-authorization
/// relation whether the builder refuses first or the script does, so the
/// row names that relation; what changes with the boundary is whether
/// the link is a discharge, and that is the evidence plan's to record.
///
/// # Errors
///
/// [`VectorError::NegativeLinkUnresolved`] when the plan publishes no
/// matching requirement or more than one, either of which means the row
/// and the plan disagree about what exists.
pub fn resolve_row(
    plan: &ValidatedLiveTransferOperationPlan,
    row: &LiveSafetyRow,
    representation: LiveTransferRepresentationPlan,
    case: SponsorCase,
) -> Result<LiveRowLink, VectorError> {
    let (relation, class, class_name) = match row.relation() {
        LiveRelationStanding::EveryPublishedRelation => return Ok(LiveRowLink::EveryRelation),
        LiveRelationStanding::Unlinked(reason) => return Ok(LiveRowLink::Blocked(*reason)),
        LiveRelationStanding::Declared {
            relation,
            class,
            class_name,
        } => (relation, class, class_name),
    };

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
            || !class(&negative.mutation)
        {
            continue;
        }
        found.insert(requirement.id.clone());
    }

    let mut resolved = found.into_iter();
    match (resolved.next(), resolved.next()) {
        (Some(only), None) => Ok(LiveRowLink::Resolved(only)),
        // Nothing published here is the conditional answer, not a
        // failure: the relation is inactive in this representation or
        // this sponsor case. Two or more is always a failure — the row
        // cannot say which it meant, and taking the first would be the
        // discharge-by-intent the census exists to prevent.
        (None, _) => Ok(LiveRowLink::InactiveInThisCase),
        (Some(_), Some(_)) => Err(VectorError::NegativeLinkUnresolved { class: class_name }),
    }
}

/// Whether one published requirement is answered by a target run.
#[must_use]
pub const fn answered_by_a_target(role: &EvidenceRole) -> bool {
    matches!(role, EvidenceRole::TargetExecution)
}

#[cfg(test)]
mod tests {
    use super::{
        LiveRelationStanding, LiveRowLink, LiveSafetyPolarity, LiveSafetyRow, LiveSafetySection,
        required_safety_matrix, resolve_row, row_count, section_census,
    };
    use crate::matrix::EvidenceBoundary;
    use std::collections::BTreeSet;

    #[test]
    fn the_matrix_transcribes_every_section_at_its_stated_width() {
        // §15's own counts, recomputed rather than asserted from prose. A
        // row added or dropped has to change this line on purpose, which
        // is the whole reason the matrix is an object rather than a
        // reading.
        let census = section_census();
        assert_eq!(census[&LiveSafetySection::PositiveExplicit], 16);
        assert_eq!(census[&LiveSafetySection::PositivePrivate], 10);
        assert_eq!(census[&LiveSafetySection::OwnerSignatureFault], 14);
        assert_eq!(census[&LiveSafetySection::ObjectFault], 16);
        assert_eq!(census[&LiveSafetySection::ValueFault], 22);
        assert_eq!(census[&LiveSafetySection::SponsorFault], 13);
        assert_eq!(census[&LiveSafetySection::StructuralFault], 17);
        assert_eq!(row_count(), 108);
        assert_eq!(census.values().sum::<usize>(), row_count());
    }

    #[test]
    fn every_row_name_is_distinct_within_its_section() {
        // A duplicate name would make two rows one row for every consumer
        // that keys by it, and the census would still count two.
        for section in LiveSafetySection::ALL {
            let rows = super::rows_of(*section);
            let names: BTreeSet<_> = rows.iter().map(LiveSafetyRow::name).collect();
            assert_eq!(names.len(), rows.len(), "{section:?} names a row twice");
            // And every row of a table agrees it belongs to that table,
            // so a row filed under the wrong section cannot be counted
            // twice by a consumer that trusts either answer.
            for row in rows {
                assert_eq!(row.section(), *section, "{row} sits in another table");
            }
        }
    }

    #[test]
    fn a_positive_row_mutates_nothing_and_expects_acceptance() {
        // The invariant the two positive tables are built on. A positive
        // class whose boundary were a refusal would be a contradiction,
        // and one carrying a mutation layer would be a negative row
        // filed in the wrong table.
        for row in required_safety_matrix() {
            let positive = row.polarity() == LiveSafetyPolarity::Positive;
            assert_eq!(
                positive,
                row.section().is_positive(),
                "{row} sits in a table of the other polarity",
            );
            if positive {
                assert_eq!(row.mutation(), None, "{row} is positive and mutates");
                assert_eq!(
                    row.refusing_layer(),
                    Some(EvidenceBoundary::AcceptedTransaction)
                );
                assert_eq!(
                    row.relation(),
                    &LiveRelationStanding::EveryPublishedRelation
                );
                assert_eq!(row.collateral(), None);
            } else {
                // A negative row names a mutation layer exactly when a
                // layer is asked to refuse the change. The row no layer
                // answers has no change to apply, and saying it mutated
                // somewhere would be describing an input that does not
                // exist.
                assert_eq!(
                    row.mutation().is_some(),
                    row.refusing_layer().is_some(),
                    "{row} disagrees with itself about having an input",
                );
                assert_ne!(
                    row.refusing_layer(),
                    Some(EvidenceBoundary::AcceptedTransaction)
                );
            }
        }
    }

    #[test]
    fn a_declared_row_carries_a_collateral_policy_and_the_rest_do_not() {
        // Collateral is what a *requirement* demands, so a row naming no
        // requirement has nothing to demand it of. A row carrying one
        // anyway would be stating an expectation nothing could contradict.
        for row in required_safety_matrix() {
            let declared = matches!(row.relation(), LiveRelationStanding::Declared { .. });
            assert_eq!(
                declared,
                row.collateral().is_some(),
                "{row} disagrees with itself about having a requirement",
            );
        }
    }

    #[test]
    fn no_row_expects_a_verdict_from_a_layer_that_produces_none() {
        // §1.11's partition, held as a property of the table: a row whose
        // boundary is the executor's infrastructure would be claiming a
        // safety verdict from a layer that produces no target observation
        // at all, and §15 names no such class.
        for row in required_safety_matrix() {
            assert_ne!(
                row.refusing_layer(),
                Some(EvidenceBoundary::ExecutorInfrastructureFailure),
                "{row} expects a safety verdict from the infrastructure",
            );
        }
    }

    #[test]
    fn a_first_party_row_is_exactly_one_refused_before_the_target() {
        for row in required_safety_matrix() {
            assert_eq!(
                row.is_first_party(),
                row.refusing_layer()
                    .is_some_and(EvidenceBoundary::is_pre_target),
                "{row} disagrees with its own boundary about who owns it",
            );
        }
    }

    #[test]
    fn exactly_one_row_names_no_layer_at_all() {
        // §15's one class no layer answers, held as a count so that a
        // second row acquiring the standing has to be argued for rather
        // than added. A row here is outside §4.2's refusal denominator
        // and is not evidence either: `crate::live_evidence` gives it a
        // standing of its own and leaves it out of the discharged count.
        let closed: BTreeSet<_> = required_safety_matrix()
            .into_iter()
            .filter(|row| row.refusing_layer().is_none())
            .map(LiveSafetyRow::name)
            .collect();
        assert_eq!(closed, BTreeSet::from(["mixed-operation-program"]));
    }

    #[test]
    fn the_two_members_minted_for_this_matrix_are_the_ones_it_uses() {
        // `crate::matrix`'s vocabulary serves Guide-12 §18 as well, and
        // §18 reaches neither of these. If §15 stopped reaching them
        // too, they would be unreachable rather than shared.
        use crate::matrix::MutationLayer;

        let rows = required_safety_matrix();
        assert!(
            rows.iter().any(|row| row.refusing_layer()
                == Some(EvidenceBoundary::ConstructorDerivationRejection)),
            "no §15 row expects the constructor derivation to refuse",
        );
        assert!(
            rows.iter()
                .any(|row| row.mutation() == Some(MutationLayer::StaticConstructorSchema)),
            "no §15 row mutates the static constructor schema",
        );
    }

    #[test]
    fn every_declared_row_resolves_to_exactly_one_published_requirement() {
        // The link made falsifiable. Each declared row must hit exactly
        // one requirement in each case of each admitted representation,
        // recomputed against the plan the compiler publishes rather than
        // against a table written here.
        use compiler::live_transfer_plan::LiveTransferRepresentationPlan;
        use compiler::operation_plan::SponsorCase;

        let plan = crate::live_plan::live_transfer_plan().expect("the live plan validates");
        let mut resolved = 0_usize;
        let mut every = 0_usize;
        let mut blocked = 0_usize;
        let mut inactive = 0_usize;
        for row in required_safety_matrix() {
            let mut resolved_somewhere = false;
            for representation in [
                LiveTransferRepresentationPlan::Explicit,
                LiveTransferRepresentationPlan::PrivateCommitted,
            ] {
                for case in [SponsorCase::Absent, SponsorCase::Present] {
                    match resolve_row(&plan, row, representation, case)
                        .unwrap_or_else(|error| panic!("{row} did not resolve: {error:?}"))
                    {
                        LiveRowLink::Resolved(id) => {
                            resolved += 1;
                            resolved_somewhere = true;
                            assert_eq!(id.case.sponsor, case, "{row} resolved into another case");
                        }
                        LiveRowLink::EveryRelation => every += 1,
                        LiveRowLink::InactiveInThisCase => inactive += 1,
                        LiveRowLink::Blocked(reason) => {
                            blocked += 1;
                            assert_eq!(
                                row.relation(),
                                &LiveRelationStanding::Unlinked(reason),
                                "{row} was blocked for a reason it did not declare",
                            );
                        }
                    }
                }
            }
            // A declared row that resolved nowhere at all is a link the
            // plan no longer publishes, which the conditional answer
            // must not be allowed to hide.
            if matches!(row.relation(), LiveRelationStanding::Declared { .. }) {
                assert!(
                    resolved_somewhere,
                    "{row} declares a requirement the plan publishes in no case",
                );
            }
        }
        // Twenty-six positive rows, in four case-representation pairs
        // each. The honest number, recomputed.
        assert_eq!(every, 26 * 4);
        assert_ne!(resolved, 0, "no row resolved to a published requirement");
        assert_eq!(
            resolved + every + blocked + inactive,
            row_count() * 4,
            "every row answered exactly once per representation and case",
        );
    }
}
