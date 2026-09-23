//! The validated maturity-announcement safety report (§14.4, §14.9).
//!
//! §14.4 asks whether valid maturity announcements preserved the exact semantic operation and whether every invalid semantic, operator, constructor, root, sponsor, ABI and protocol mutation failed at its declared boundary. This module answers the second half of that question over the refusals this workspace has actually driven, names in a type which clause of the refinement question that half is, and states as a figure how many rows the clause is still owed.
//!
//! # What it answers, and what it does not
//!
//! Host continuity comparisons and projector refusals travel as exact-row records
//! alongside the unchanged standing census. Their material presence is explicit;
//! their source and branch context confer no target acceptance or freshness.
//! Root history and public recovery retain their separate obligations. A host
//! comparison alone supplies no accepted concrete step or forward simulation.
//!
//! # One clause, stated in its own type
//!
//! [`MaturityRefinementClause`] has one member, and the omission is the point. The clause this report witnesses is soundness — every refused concrete step corresponds to no allowed semantic step at its declared boundary — and the forward clause is a different statement over an accepted step nobody has observed. A member named for a clause nobody has written would be weaker than witnessing one, because a reader could not tell a clause that is owned and outstanding from a clause that is merely named.
//!
//! # A partial witness is stated as partial, never omitted
//!
//! Forty entry points were each driven twice, accepting an honest control and refusing exactly one changed input, and each refusal arrived at the boundary its row declared, which is what makes it a witness of the clause rather than a refusal about something else. Omitting the witness would say nothing was established when forty recomputed discharges were; stating it without its outstanding count would say the clause holds over the matrix, which is false. [`MaturitySoundnessWitness`] therefore carries both the refusals it stands on and the number of rows it does not cover, so the overstatement a reader should fear is the one the value itself refuses to make.
//!
//! # The exclusions are a type, and the timing travels apart
//!
//! [`MaturityVolatileField`] names the eleven excluded categories. The renderer
//! emits a closed set of keys, source classes, byte identities and refusal names;
//! archived addresses and diagnostic text remain outside that projection.
//! [`MaturityReportTiming`] renders separately and is neither a report field nor
//! a canonical renderer argument. Equal plans therefore render equal bytes.
//!
//! # The wrapper is where the figures are established
//!
//! [`MaturityAnnouncementSafetyReport`] is a value with private fields that establishes nothing. [`validate_maturity_safety_report`] recomputes every figure it carries from the evidence plan, the register and the matrix, records each comparison only after it ran, and refuses on the first disagreement naming the item that disagreed. A report whose stated totals did not follow from its own rows is refused rather than published.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::time::Duration;

use crate::matrix::EvidenceBoundary;
use crate::maturity_continuity::{MaturityByteSource, MaturityContinuityRefusal};
use crate::maturity_evidence::{
    MaturityAnnouncementEvidencePlan, MaturityConstructorMaterialPresence,
    MaturityContinuityObservation, MaturityContinuityRecord, MaturityEvidenceCensus,
    MaturityExecutorProvenanceExpectation, MaturityGuaranteeQuantifier, MaturityRowStanding,
    MaturityTargetBinding,
};
use crate::maturity_first_party::MaturityFirstPartyValidator;
use crate::maturity_negative_half::{
    MaturityNegativeHalfGap, MaturityNegativeHalfLink, STILL_REQUIRED, gap_census,
};
use crate::maturity_safety::{MaturityExpectedProjection, MaturitySafetySection};

/// The schema of the canonical rendered safety report.
///
/// Stated in the bytes so a reader never infers which revision a document is. Revision 1 is the first: no earlier field set was ever rendered, so there is no historical reader to strand and no earlier revision this one must decline.
pub const MATURITY_SAFETY_REPORT_SCHEMA: u32 = 1;

/// What kind of report this is, said in the bytes.
///
/// One member, and it is named rather than assumed. §1.14 keeps the evidence classes apart, and a reader holding only the rendered bytes must be able to tell which class of report they have before reading a figure out of it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturitySafetyReportRole {
    /// The maturity-announcement safety report of §14.4.
    MaturityAnnouncementSafety,
}

impl MaturitySafetyReportRole {
    /// The role's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::MaturityAnnouncementSafety => "maturity-announcement-safety",
        }
    }
}

/// Whether a report may call itself complete.
///
/// Three states and no inference. Completeness is never implied by the absence of failures, so the token is explicit, and the middle state exists to keep "nothing failed" and "everything ran" apart.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturitySafetyCompleteness {
    /// Every required row of the matrix is answered at its own boundary.
    CompleteForTheRequiredMatrix,
    /// Some required row is unanswered, and the report names which.
    PartialRequiredRowsOutstanding,
    /// A row was refused away from its declared boundary, or a row's component does not exist.
    Failed,
}

impl MaturitySafetyCompleteness {
    /// The token's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::CompleteForTheRequiredMatrix => "complete-for-the-required-matrix",
            Self::PartialRequiredRowsOutstanding => "partial-required-rows-outstanding",
            Self::Failed => "failed",
        }
    }
}

/// One volatile field §14.9 keeps out of the canonical bytes.
///
/// Eleven members, in that section's own order, enumerated so the exclusion is a census a test can walk rather than a sentence in a comment. The generation whose report is already landed carries ten members under a different partition of the same space — it splits fixture material from an environment value and has no member at all for a path the caller chose — so its type is read here as a practice to mirror and not as a vocabulary to share: reusing it would either drop an exclusion this section names or rename two it does not.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityVolatileField {
    /// The wall-clock time at which something happened.
    WallClockTime,
    /// How long something took.
    ElapsedTime,
    /// The host it happened on.
    Hostname,
    /// The account it ran as.
    Username,
    /// The process identifier.
    ProcessId,
    /// A temporary directory path.
    TemporaryPath,
    /// The executor program's own path.
    ExecutorPath,
    /// A filesystem path the caller chose.
    CallerSelectedFilesystemPath,
    /// A value read out of the ambient environment.
    AmbientEnvironment,
    /// Raw text a child process wrote to its error stream.
    RawChildStandardError,
    /// A private scalar, whether a production one or a test one.
    ProductionOrTestPrivateScalar,
}

impl MaturityVolatileField {
    /// All eleven, in §14.9's order.
    pub const ALL: &'static [Self] = &[
        Self::WallClockTime,
        Self::ElapsedTime,
        Self::Hostname,
        Self::Username,
        Self::ProcessId,
        Self::TemporaryPath,
        Self::ExecutorPath,
        Self::CallerSelectedFilesystemPath,
        Self::AmbientEnvironment,
        Self::RawChildStandardError,
        Self::ProductionOrTestPrivateScalar,
    ];

    /// The field's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::WallClockTime => "wall-clock-time",
            Self::ElapsedTime => "elapsed-time",
            Self::Hostname => "hostname",
            Self::Username => "username",
            Self::ProcessId => "process-id",
            Self::TemporaryPath => "temporary-path",
            Self::ExecutorPath => "executor-path",
            Self::CallerSelectedFilesystemPath => "caller-selected-filesystem-path",
            Self::AmbientEnvironment => "ambient-environment",
            Self::RawChildStandardError => "raw-child-standard-error",
            Self::ProductionOrTestPrivateScalar => "production-or-test-private-scalar",
        }
    }
}

/// The clause of the refinement question this report witnesses.
///
/// One member. Soundness is the half an unaccepted matrix can answer: a refusal is a concrete step somebody drove, and whether it corresponds to no allowed semantic step at its declared boundary is decidable from the refusal and the row together. The forward half — that an accepted concrete step corresponds to exactly one allowed semantic step — wants an acceptance and the constructor projections on both sides of it, and a member named for it here would be a name standing in place of a statement.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityRefinementClause {
    /// Every refused concrete step corresponds to no allowed semantic step at its declared boundary.
    SoundnessOfRefusedSteps,
}

impl MaturityRefinementClause {
    /// The clause's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::SoundnessOfRefusedSteps => "soundness-of-refused-steps",
        }
    }
}

/// How far a witness carries its clause.
///
/// Three standings, and the third is not a stronger kind of waiting. A refusal observed away from the boundary its row declared contradicts the clause for that row, and a report that rendered it as a row still waiting would understate what it found, which is the same ordering the completeness token keeps.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityClauseStanding {
    /// Every row of the matrix has produced a refusal this witness stands on.
    WitnessedOverEveryRow,
    /// The clause holds over the refusals observed, and the rows it does not cover are counted.
    PartialOverTheRefusalsObserved,
    /// A refusal arrived at a boundary other than the row's declared one.
    ContradictedAtAnObservedBoundary,
}

impl MaturityClauseStanding {
    /// The standing's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::WitnessedOverEveryRow => "witnessed-over-every-row",
            Self::PartialOverTheRefusalsObserved => "partial-over-the-refusals-observed",
            Self::ContradictedAtAnObservedBoundary => "contradicted-at-an-observed-boundary",
        }
    }
}

/// One refusal the soundness witness stands on.
///
/// The entry point that refused, the boundary the row declared, and the class the refusal named. The refusal value itself is deliberately absent: the owning layers answer in several different vocabularies whose payloads carry bytes, and rendering one of those into a canonical document would put material in the bytes that §14.9 keeps out of them. What a reader needs in order to check the claim is the call to make and the class to expect, and both are here.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MaturityWitnessedRefusal {
    section: MaturitySafetySection,
    row: &'static str,
    validator: MaturityFirstPartyValidator,
    declared_boundary: EvidenceBoundary,
    class: &'static str,
}

impl MaturityWitnessedRefusal {
    /// The table the refused row is drawn from.
    #[must_use]
    pub const fn section(&self) -> MaturitySafetySection {
        self.section
    }

    /// The row's name within its table.
    #[must_use]
    pub const fn row(&self) -> &'static str {
        self.row
    }

    /// The entry point that produced the refusal.
    #[must_use]
    pub const fn validator(&self) -> MaturityFirstPartyValidator {
        self.validator
    }

    /// The boundary the row declared, which is where the refusal arrived.
    #[must_use]
    pub const fn declared_boundary(&self) -> EvidenceBoundary {
        self.declared_boundary
    }

    /// The refusal class the owning layer named.
    #[must_use]
    pub const fn class(&self) -> &'static str {
        self.class
    }
}

/// The clause this report witnesses, and what it is still owed.
///
/// The counts are figures rather than prose so a reader can hold the witness against the matrix it is stated over. The owed figure counts every row this report has not answered, which is deliberately the larger of the two readings available: a row answered by an acceptance would belong to the forward clause rather than to this one, and a witness that shrank its own denominator to the rows it might eventually cover would look nearer to total than it is.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MaturitySoundnessWitness {
    clause: MaturityRefinementClause,
    refusals_witnessed: usize,
    native_refusals_witnessed: usize,
    rows_still_owed: usize,
    standing: MaturityClauseStanding,
}

impl MaturitySoundnessWitness {
    /// The clause witnessed.
    #[must_use]
    pub const fn clause(&self) -> MaturityRefinementClause {
        self.clause
    }

    /// How many first-party refusals the witness stands on.
    #[must_use]
    pub const fn refusals_witnessed(&self) -> usize {
        self.refusals_witnessed
    }

    /// How many target refusals it stands on.
    #[must_use]
    pub const fn native_refusals_witnessed(&self) -> usize {
        self.native_refusals_witnessed
    }

    /// How many rows the clause is still owed.
    #[must_use]
    pub const fn rows_still_owed(&self) -> usize {
        self.rows_still_owed
    }

    /// How far the witness carries the clause.
    #[must_use]
    pub const fn standing(&self) -> MaturityClauseStanding {
        self.standing
    }
}

/// One row waiting on a target run, with the gap that keeps it waiting.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MaturityWaitingRow {
    section: MaturitySafetySection,
    row: &'static str,
    gap: MaturityNegativeHalfGap,
    link: MaturityNegativeHalfLink,
}

impl MaturityWaitingRow {
    /// The table the waiting row is drawn from.
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

    /// Whether a refusal of it could be filed against a published requirement.
    #[must_use]
    pub const fn link(&self) -> MaturityNegativeHalfLink {
        self.link
    }
}

/// One obligation this report states about itself.
///
/// The rows whose declared boundary is this workspace's own canonical serialization are the rows a validation reading these bytes would answer. This report is their subject rather than their answer, so it carries them as requirements it is owed and leaves the reading to the validation that owns it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MaturityReportLayerRequirement {
    section: MaturitySafetySection,
    row: &'static str,
    projection: MaturityExpectedProjection,
}

impl MaturityReportLayerRequirement {
    /// The table the row is drawn from.
    #[must_use]
    pub const fn section(&self) -> MaturitySafetySection {
        self.section
    }

    /// The row's name within its table.
    #[must_use]
    pub const fn row(&self) -> &'static str {
        self.row
    }

    /// What the row expects the projection comparison to say.
    #[must_use]
    pub const fn projection(&self) -> MaturityExpectedProjection {
        self.projection
    }
}

/// The derivation's wall, which the canonical bytes do not carry.
///
/// A duration is a property of the machine a derivation ran on rather than of what it established, and a canonical stream carrying one would differ from every other by construction, so two derivations of a single plan could never be compared byte for byte. It renders itself, under its own marker, through a function the canonical renderer does not call and could not reach.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MaturityReportTiming {
    derivation: Duration,
}

impl MaturityReportTiming {
    /// Record one derivation's wall.
    #[must_use]
    pub const fn measured(derivation: Duration) -> Self {
        Self { derivation }
    }

    /// The recorded wall.
    #[must_use]
    pub const fn derivation(self) -> Duration {
        self.derivation
    }

    /// The timing's own rendering, marked noncanonical in its own bytes.
    ///
    /// # Panics
    ///
    /// Never: the sink is a `String`, whose writes cannot fail.
    #[must_use]
    pub fn render(self) -> String {
        let mut text = String::new();
        let _ = writeln!(text, "role maturity-announcement-safety-timing");
        let _ = writeln!(text, "canonical false");
        let _ = writeln!(text, "derivation_nanos {}", self.derivation.as_nanos());
        text
    }
}

/// The maturity-announcement safety report of §14.4.
///
/// A value a caller can hold and one that establishes nothing on its own. Every field is private and there is no public constructor, because a constructor taking a total would let a caller state a conclusion the rows do not support and leave the validation checking that caller against themselves.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityAnnouncementSafetyReport {
    schema: u32,
    role: MaturitySafetyReportRole,
    binding: MaturityTargetBinding,
    provenance: MaturityExecutorProvenanceExpectation,
    quantifier: MaturityGuaranteeQuantifier,
    witnessed: Vec<MaturityWitnessedRefusal>,
    waiting: Vec<MaturityWaitingRow>,
    material: MaturityConstructorMaterialPresence,
    records: Vec<MaturityContinuityRecord>,
    report_layer: Vec<MaturityReportLayerRequirement>,
    census: MaturityEvidenceCensus,
    answered: usize,
    outstanding: usize,
    completeness: MaturitySafetyCompleteness,
    clause: MaturitySoundnessWitness,
}

impl MaturityAnnouncementSafetyReport {
    /// The report's schema.
    #[must_use]
    pub const fn schema(&self) -> u32 {
        self.schema
    }

    /// What kind of report this is.
    #[must_use]
    pub const fn role(&self) -> MaturitySafetyReportRole {
        self.role
    }

    /// The exact target and deployment the evidence is stated against.
    #[must_use]
    pub const fn binding(&self) -> &MaturityTargetBinding {
        &self.binding
    }

    /// What the operator declared the executor was built from, or that they declared nothing.
    #[must_use]
    pub const fn provenance(&self) -> &MaturityExecutorProvenanceExpectation {
        &self.provenance
    }

    /// The quantifier every figure here is stated under.
    #[must_use]
    pub const fn quantifier(&self) -> MaturityGuaranteeQuantifier {
        self.quantifier
    }

    /// The refusals the soundness witness stands on, in the matrix's order.
    #[must_use]
    pub fn witnessed(&self) -> &[MaturityWitnessedRefusal] {
        &self.witnessed
    }

    /// The rows waiting on a target run, in the matrix's order.
    #[must_use]
    pub fn waiting(&self) -> &[MaturityWaitingRow] {
        &self.waiting
    }

    /// Whether the evidence plan received constructor-continuity material.
    #[must_use]
    pub const fn constructor_material(&self) -> MaturityConstructorMaterialPresence {
        self.material
    }

    /// Exact-row continuity records in the evidence plan's input order.
    #[must_use]
    pub fn continuity_records(&self) -> &[MaturityContinuityRecord] {
        &self.records
    }

    /// The obligations this report states about itself, in the matrix's order.
    #[must_use]
    pub fn report_layer(&self) -> &[MaturityReportLayerRequirement] {
        &self.report_layer
    }

    /// The row census.
    #[must_use]
    pub const fn census(&self) -> MaturityEvidenceCensus {
        self.census
    }

    /// How many rows are answered.
    #[must_use]
    pub const fn answered(&self) -> usize {
        self.answered
    }

    /// How many rows stand outstanding.
    #[must_use]
    pub const fn outstanding(&self) -> usize {
        self.outstanding
    }

    /// The completeness token.
    #[must_use]
    pub const fn completeness(&self) -> MaturitySafetyCompleteness {
        self.completeness
    }

    /// The clause the report witnesses, and what it is still owed.
    #[must_use]
    pub const fn clause(&self) -> MaturitySoundnessWitness {
        self.clause
    }
}

/// One figure this validation recomputed rather than read.
///
/// A census rather than prose, so a validation that stopped performing one has to remove it here and fail the count.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityRecomputedItem {
    /// The report's schema.
    Schema,
    /// The report's role.
    Role,
    /// The exact target and deployment binding.
    TargetBinding,
    /// The expected executor provenance.
    ExecutorProvenanceExpectation,
    /// The quantifier the figures are stated under.
    GuaranteeQuantifier,
    /// Every bucket of the row census.
    CensusBuckets,
    /// The answered and outstanding halves, and that they partition the matrix.
    AnsweredAndOutstanding,
    /// The completeness token.
    Completeness,
    /// The refusals the witness stands on.
    WitnessedRefusals,
    /// The register's set equality with the plan's waiting rows.
    RegisterSetEquality,
    /// Material presence and exact-row continuity records.
    ConstructorContinuityMaterial,
    /// The obligations the report states about itself.
    ReportLayerRequirements,
    /// The clause witness and its counts.
    ClauseWitness,
}

impl MaturityRecomputedItem {
    /// All thirteen, in the order the validation performs them.
    pub const ALL: &'static [Self] = &[
        Self::Schema,
        Self::Role,
        Self::TargetBinding,
        Self::ExecutorProvenanceExpectation,
        Self::GuaranteeQuantifier,
        Self::CensusBuckets,
        Self::AnsweredAndOutstanding,
        Self::Completeness,
        Self::WitnessedRefusals,
        Self::RegisterSetEquality,
        Self::ConstructorContinuityMaterial,
        Self::ReportLayerRequirements,
        Self::ClauseWitness,
    ];

    /// The item's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Schema => "schema",
            Self::Role => "role",
            Self::TargetBinding => "target-binding",
            Self::ExecutorProvenanceExpectation => "executor-provenance-expectation",
            Self::GuaranteeQuantifier => "guarantee-quantifier",
            Self::CensusBuckets => "census-buckets",
            Self::AnsweredAndOutstanding => "answered-and-outstanding",
            Self::Completeness => "completeness",
            Self::WitnessedRefusals => "witnessed-refusals",
            Self::RegisterSetEquality => "register-set-equality",
            Self::ConstructorContinuityMaterial => "constructor-continuity-material",
            Self::ReportLayerRequirements => "report-layer-requirements",
            Self::ClauseWitness => "clause-witness",
        }
    }
}

/// Why one report could not be assembled or validated.
///
/// No member carries another layer's refusal, and that is a fact about the inputs rather than a simplification. The evidence plan has private fields and no public constructor, so a value of it can only have come from a derivation that already succeeded; the register and the matrix are constants. Nothing this module calls can refuse, so every member here is a disagreement between a report and what this module recomputed, and [`Self::failed_item`] names which recomputation found it.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MaturitySafetyReportRefusal {
    /// A row waiting on a target run has no entry in the register.
    ///
    /// The one refusal assembly raises. It is unreachable while the register and the classifier agree, which they do at this tip and which the register's own test is what keeps true; the member exists because the two can drift apart later, and a report assembled from a drifted pair must refuse rather than omit the row or take the register's silence for a gap.
    WaitingRowCarriesNoRegisteredGap {
        /// The table the row is drawn from.
        section: MaturitySafetySection,
        /// The row's name within its table.
        name: &'static str,
    },
    /// The report states a schema this validation does not read.
    UnsupportedSchema(u32),
    /// The report states a role that is not the maturity-announcement safety one.
    WrongRole(MaturitySafetyReportRole),
    /// The report's target and deployment binding is not the plan's.
    TargetBindingDiffers,
    /// The report's executor provenance expectation is not the plan's.
    ExecutorProvenanceExpectationDiffers,
    /// The report's quantifier is not the one the census states.
    GuaranteeQuantifierDiffers,
    /// One census bucket is not the figure the plan recomputes.
    CensusBucketDiffers {
        /// The bucket's own name.
        bucket: &'static str,
        /// What the report said.
        stated: usize,
        /// What the plan recomputes to.
        recomputed: usize,
    },
    /// The report's answered or outstanding figure is not the census's.
    AnsweredOrOutstandingDiffers {
        /// Which of the two figures disagreed.
        figure: &'static str,
        /// What the report said.
        stated: usize,
        /// What the census recomputes to.
        recomputed: usize,
    },
    /// The answered and outstanding halves do not partition the matrix.
    AnsweredAndOutstandingDoNotPartitionTheMatrix {
        /// The denominator.
        rows: usize,
        /// The answered half.
        answered: usize,
        /// The outstanding half.
        outstanding: usize,
    },
    /// The report's completeness token is not the one the census supports.
    CompletenessDiffers {
        /// What the report said.
        stated: MaturitySafetyCompleteness,
        /// What the census supports.
        recomputed: MaturitySafetyCompleteness,
    },
    /// One witnessed refusal is not the one the plan recomputes at that position.
    WitnessedRefusalDiffers {
        /// The table the disagreeing row is drawn from.
        section: MaturitySafetySection,
        /// The row's name within its table.
        name: &'static str,
    },
    /// The witnessed refusals are not as many as the plan discharged.
    WitnessedRefusalCountDiffers {
        /// What the report carried.
        stated: usize,
        /// What the plan recomputes to.
        recomputed: usize,
    },
    /// A row the report carries as waiting has no entry in the register.
    WaitingRowIsNotRegistered {
        /// The table the row is drawn from.
        section: MaturitySafetySection,
        /// The row's name within its table.
        name: &'static str,
    },
    /// A register entry names a row the plan does not have waiting.
    RegisteredRowIsNotWaiting {
        /// The table the entry names.
        section: MaturitySafetySection,
        /// The row's name within its table.
        name: &'static str,
    },
    /// The waiting rows are not as many as the register or the census counts.
    WaitingRowCountDiffers {
        /// Which count disagreed.
        against: &'static str,
        /// What the report carried.
        stated: usize,
        /// What the named source counts.
        recomputed: usize,
    },
    /// The report's material presence differs from the supplied plan's.
    ConstructorMaterialPresenceDiffers {
        /// What the report states.
        stated: MaturityConstructorMaterialPresence,
        /// What the plan supplies.
        recomputed: MaturityConstructorMaterialPresence,
    },
    /// The ordered continuity records differ from the supplied plan's.
    ContinuityRecordsDiffer,
    /// One report-layer requirement is not the one the matrix states.
    ReportLayerRequirementDiffers {
        /// The table the disagreeing row is drawn from.
        section: MaturitySafetySection,
        /// The row's name within its table.
        name: &'static str,
    },
    /// The report-layer requirements are not as many as the census counts.
    ReportLayerRequirementCountDiffers {
        /// What the report carried.
        stated: usize,
        /// What the census counts.
        recomputed: usize,
    },
    /// The report witnesses a clause other than the one this wave can answer.
    ClauseDiffers {
        /// What the report said.
        stated: MaturityRefinementClause,
        /// What this module witnesses.
        recomputed: MaturityRefinementClause,
    },
    /// The witness's standing is not the one its counts support.
    ClauseStandingDiffers {
        /// What the report said.
        stated: MaturityClauseStanding,
        /// What the counts support.
        recomputed: MaturityClauseStanding,
    },
    /// One of the witness's counts is not the figure recomputed for it.
    ClauseCountDiffers {
        /// Which count disagreed.
        figure: &'static str,
        /// What the report said.
        stated: usize,
        /// What the plan recomputes to.
        recomputed: usize,
    },
}

impl MaturitySafetyReportRefusal {
    /// The recomputation that found the disagreement, where one did.
    ///
    /// `None` for the assembly refusal, which is raised before any recomputation runs and therefore names none.
    #[must_use]
    pub const fn failed_item(&self) -> Option<MaturityRecomputedItem> {
        match self {
            Self::WaitingRowCarriesNoRegisteredGap { .. } => None,
            Self::UnsupportedSchema(_) => Some(MaturityRecomputedItem::Schema),
            Self::WrongRole(_) => Some(MaturityRecomputedItem::Role),
            Self::TargetBindingDiffers => Some(MaturityRecomputedItem::TargetBinding),
            Self::ExecutorProvenanceExpectationDiffers => {
                Some(MaturityRecomputedItem::ExecutorProvenanceExpectation)
            }
            Self::GuaranteeQuantifierDiffers => Some(MaturityRecomputedItem::GuaranteeQuantifier),
            Self::CensusBucketDiffers { .. } => Some(MaturityRecomputedItem::CensusBuckets),
            Self::AnsweredOrOutstandingDiffers { .. }
            | Self::AnsweredAndOutstandingDoNotPartitionTheMatrix { .. } => {
                Some(MaturityRecomputedItem::AnsweredAndOutstanding)
            }
            Self::CompletenessDiffers { .. } => Some(MaturityRecomputedItem::Completeness),
            Self::WitnessedRefusalDiffers { .. } | Self::WitnessedRefusalCountDiffers { .. } => {
                Some(MaturityRecomputedItem::WitnessedRefusals)
            }
            Self::WaitingRowIsNotRegistered { .. }
            | Self::RegisteredRowIsNotWaiting { .. }
            | Self::WaitingRowCountDiffers { .. } => {
                Some(MaturityRecomputedItem::RegisterSetEquality)
            }
            Self::ConstructorMaterialPresenceDiffers { .. } | Self::ContinuityRecordsDiffer => {
                Some(MaturityRecomputedItem::ConstructorContinuityMaterial)
            }
            Self::ReportLayerRequirementDiffers { .. }
            | Self::ReportLayerRequirementCountDiffers { .. } => {
                Some(MaturityRecomputedItem::ReportLayerRequirements)
            }
            Self::ClauseDiffers { .. }
            | Self::ClauseStandingDiffers { .. }
            | Self::ClauseCountDiffers { .. } => Some(MaturityRecomputedItem::ClauseWitness),
        }
    }
}

/// The report every figure of which has been recomputed.
///
/// Private fields and one constructor, [`validate_maturity_safety_report`], which records an item only after its comparison ran. There is no intermediate state in which a wrapper exists with an item unmarked, so the recomputed set is a property of the value rather than a claim beside it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedMaturityAnnouncementSafetyReport {
    report: MaturityAnnouncementSafetyReport,
    recomputed_items: BTreeSet<MaturityRecomputedItem>,
}

impl ValidatedMaturityAnnouncementSafetyReport {
    /// The report whose every figure was recomputed.
    #[must_use]
    pub const fn report(&self) -> &MaturityAnnouncementSafetyReport {
        &self.report
    }

    /// The items this validation recomputed.
    #[must_use]
    pub const fn recomputed_items(&self) -> &BTreeSet<MaturityRecomputedItem> {
        &self.recomputed_items
    }
}

/// The three facts the completeness token is decided from.
///
/// Projected out of the census rather than read from it one call at a time, so the rule below is a total function of three values a test can state directly. That matters here in a way it would not elsewhere: the census's own fields are private to the module that counts them and it publishes no constructor for a hand-built one, so a rule written against the census type could be exercised on the tree as it stands and on nothing else, and a failing condition no test can reach is a rule stated rather than kept.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CompletenessFacts {
    contradicted: usize,
    blocked: usize,
    every_required_row_is_answered: bool,
}

impl CompletenessFacts {
    /// The three facts one census determines.
    const fn of(census: MaturityEvidenceCensus) -> Self {
        Self {
            contradicted: census.native_refusal_at_unexpected_boundary(),
            blocked: census.infrastructure_blocked(),
            every_required_row_is_answered: census.every_required_row_is_answered(),
        }
    }

    /// The completeness these facts support.
    ///
    /// The two failing conditions are asked first, and the order is the rule. A row refused away from its declared boundary was answered somewhere the row never asked about, and a row whose component does not exist was never asked at all; rendering either as a row still waiting would name a wait that no run ends. Only once neither holds does the absence of outstanding rows mean the matrix is complete.
    const fn completeness(self) -> MaturitySafetyCompleteness {
        if self.contradicted > 0 || self.blocked > 0 {
            MaturitySafetyCompleteness::Failed
        } else if self.every_required_row_is_answered {
            MaturitySafetyCompleteness::CompleteForTheRequiredMatrix
        } else {
            MaturitySafetyCompleteness::PartialRequiredRowsOutstanding
        }
    }
}

/// The completeness one census supports.
const fn completeness_of(census: MaturityEvidenceCensus) -> MaturitySafetyCompleteness {
    CompletenessFacts::of(census).completeness()
}

/// Every census bucket, paired with the name it renders under.
const fn census_buckets(census: MaturityEvidenceCensus) -> [(&'static str, usize); 16] {
    [
        ("rows", census.rows()),
        ("first_party_discharged", census.first_party_discharged()),
        ("first_party_required", census.first_party_required()),
        (
            "native_acceptance_observed",
            census.native_acceptance_observed(),
        ),
        ("native_refusal_observed", census.native_refusal_observed()),
        (
            "native_declared_boundary_observed",
            census.native_declared_boundary_observed(),
        ),
        (
            "native_refusal_at_unexpected_boundary",
            census.native_refusal_at_unexpected_boundary(),
        ),
        ("native_run_required", census.native_run_required()),
        (
            "constructor_continuity_observed",
            census.constructor_continuity_observed(),
        ),
        ("root_history_observed", census.root_history_observed()),
        (
            "public_recovery_observed",
            census.public_recovery_observed(),
        ),
        ("report_layer_observed", census.report_layer_observed()),
        ("report_layer_required", census.report_layer_required()),
        (
            "outstanding_under_typed_non_answer",
            census.outstanding_under_typed_non_answer(),
        ),
        ("infrastructure_blocked", census.infrastructure_blocked()),
        ("experimental", census.experimental()),
    ]
}

/// The standing word one executor provenance expectation renders under.
const fn provenance_standing(expectation: &MaturityExecutorProvenanceExpectation) -> &'static str {
    match expectation {
        MaturityExecutorProvenanceExpectation::Stated(_) => "stated",
        MaturityExecutorProvenanceExpectation::NotStatedByTheOperator => {
            "not-stated-by-the-operator"
        }
    }
}

/// The refusals the soundness witness stands on, in the matrix's order.
fn witnessed_refusals(plan: &MaturityAnnouncementEvidencePlan) -> Vec<MaturityWitnessedRefusal> {
    plan.rows()
        .iter()
        .filter_map(|classified| match classified.standing() {
            MaturityRowStanding::FirstPartyDischarged { validator, class } => classified
                .row()
                .refusing_layer()
                .map(|declared_boundary| MaturityWitnessedRefusal {
                    section: classified.row().section(),
                    row: classified.row().name(),
                    validator: *validator,
                    declared_boundary,
                    class,
                }),
            _ => None,
        })
        .collect()
}

/// The rows waiting on a target run, each joined to its registered gap.
fn waiting_rows(
    plan: &MaturityAnnouncementEvidencePlan,
) -> Result<Vec<MaturityWaitingRow>, MaturitySafetyReportRefusal> {
    let mut waiting = Vec::new();
    for classified in plan.rows() {
        if !matches!(
            classified.standing(),
            MaturityRowStanding::NativeRunRequired(_)
        ) {
            continue;
        }
        let section = classified.row().section();
        let name = classified.row().name();
        let entry = STILL_REQUIRED
            .iter()
            .find(|entry| entry.section() == section && entry.row() == name)
            .ok_or(
                MaturitySafetyReportRefusal::WaitingRowCarriesNoRegisteredGap { section, name },
            )?;
        waiting.push(MaturityWaitingRow {
            section,
            row: name,
            gap: entry.gap(),
            link: entry.link(),
        });
    }
    Ok(waiting)
}

/// The obligations the report states about itself, in the matrix's order.
fn report_layer_requirements(
    plan: &MaturityAnnouncementEvidencePlan,
) -> Vec<MaturityReportLayerRequirement> {
    plan.rows()
        .iter()
        .filter_map(|classified| match classified.standing() {
            MaturityRowStanding::ReportLayerRequired(projection) => {
                Some(MaturityReportLayerRequirement {
                    section: classified.row().section(),
                    row: classified.row().name(),
                    projection: *projection,
                })
            }
            _ => None,
        })
        .collect()
}

/// The witness one census and one refusal count support.
const fn soundness_witness(
    census: MaturityEvidenceCensus,
    refusals_witnessed: usize,
) -> MaturitySoundnessWitness {
    let standing = if census.native_refusal_at_unexpected_boundary() > 0 {
        MaturityClauseStanding::ContradictedAtAnObservedBoundary
    } else if census.answered() == census.rows() {
        MaturityClauseStanding::WitnessedOverEveryRow
    } else {
        MaturityClauseStanding::PartialOverTheRefusalsObserved
    };
    MaturitySoundnessWitness {
        clause: MaturityRefinementClause::SoundnessOfRefusedSteps,
        refusals_witnessed,
        native_refusals_witnessed: census.native_refusal_observed(),
        rows_still_owed: census.rows().saturating_sub(census.answered()),
        standing,
    }
}

/// Assemble the safety report one evidence plan determines (§14.4).
///
/// Every carrier is derived from the plan, the register and the matrix, which is why this returns a report rather than taking one: a constructor accepting a total would let a caller state a conclusion the rows do not support, and the validation would then be checking that caller against themselves.
///
/// # Errors
///
/// [`MaturitySafetyReportRefusal::WaitingRowCarriesNoRegisteredGap`] if the plan routes a row to a target run that the register carries no entry for.
pub fn assemble_maturity_safety_report(
    plan: &MaturityAnnouncementEvidencePlan,
) -> Result<MaturityAnnouncementSafetyReport, MaturitySafetyReportRefusal> {
    let census = plan.census();
    let witnessed = witnessed_refusals(plan);
    let clause = soundness_witness(census, witnessed.len());
    Ok(MaturityAnnouncementSafetyReport {
        schema: MATURITY_SAFETY_REPORT_SCHEMA,
        role: MaturitySafetyReportRole::MaturityAnnouncementSafety,
        binding: plan.binding().clone(),
        provenance: plan.executor_provenance().clone(),
        quantifier: census.quantifier(),
        witnessed,
        waiting: waiting_rows(plan)?,
        material: plan.constructor_material().presence(),
        records: plan.continuity_records().to_vec(),
        report_layer: report_layer_requirements(plan),
        census,
        answered: census.answered(),
        outstanding: census.outstanding(),
        completeness: completeness_of(census),
        clause,
    })
}

/// Recompute material presence and every ordered continuity-record operand.
fn validate_constructor_material(
    report: &MaturityAnnouncementSafetyReport,
    plan: &MaturityAnnouncementEvidencePlan,
    done: &mut BTreeSet<MaturityRecomputedItem>,
) -> Result<(), MaturitySafetyReportRefusal> {
    let recomputed = plan.constructor_material().presence();
    if report.material != recomputed {
        return Err(
            MaturitySafetyReportRefusal::ConstructorMaterialPresenceDiffers {
                stated: report.material,
                recomputed,
            },
        );
    }
    if report.records != plan.continuity_records() {
        return Err(MaturitySafetyReportRefusal::ContinuityRecordsDiffer);
    }
    done.insert(MaturityRecomputedItem::ConstructorContinuityMaterial);
    Ok(())
}

/// Recompute the envelope: schema, role, binding, provenance, quantifier.
fn validate_envelope(
    report: &MaturityAnnouncementSafetyReport,
    plan: &MaturityAnnouncementEvidencePlan,
    done: &mut BTreeSet<MaturityRecomputedItem>,
) -> Result<(), MaturitySafetyReportRefusal> {
    if report.schema != MATURITY_SAFETY_REPORT_SCHEMA {
        return Err(MaturitySafetyReportRefusal::UnsupportedSchema(
            report.schema,
        ));
    }
    done.insert(MaturityRecomputedItem::Schema);

    if report.role != MaturitySafetyReportRole::MaturityAnnouncementSafety {
        return Err(MaturitySafetyReportRefusal::WrongRole(report.role));
    }
    done.insert(MaturityRecomputedItem::Role);

    if &report.binding != plan.binding() {
        return Err(MaturitySafetyReportRefusal::TargetBindingDiffers);
    }
    done.insert(MaturityRecomputedItem::TargetBinding);

    if &report.provenance != plan.executor_provenance() {
        return Err(MaturitySafetyReportRefusal::ExecutorProvenanceExpectationDiffers);
    }
    done.insert(MaturityRecomputedItem::ExecutorProvenanceExpectation);

    if report.quantifier != plan.census().quantifier() {
        return Err(MaturitySafetyReportRefusal::GuaranteeQuantifierDiffers);
    }
    done.insert(MaturityRecomputedItem::GuaranteeQuantifier);

    Ok(())
}

/// Recompute the census buckets, the two halves, and the completeness token.
fn validate_census(
    report: &MaturityAnnouncementSafetyReport,
    plan: &MaturityAnnouncementEvidencePlan,
    done: &mut BTreeSet<MaturityRecomputedItem>,
) -> Result<(), MaturitySafetyReportRefusal> {
    let recomputed = plan.census();
    for ((bucket, stated), (_, expected)) in census_buckets(report.census)
        .into_iter()
        .zip(census_buckets(recomputed))
    {
        if stated != expected {
            return Err(MaturitySafetyReportRefusal::CensusBucketDiffers {
                bucket,
                stated,
                recomputed: expected,
            });
        }
    }
    done.insert(MaturityRecomputedItem::CensusBuckets);

    for (figure, stated, expected) in [
        ("answered", report.answered, recomputed.answered()),
        ("outstanding", report.outstanding, recomputed.outstanding()),
    ] {
        if stated != expected {
            return Err(MaturitySafetyReportRefusal::AnsweredOrOutstandingDiffers {
                figure,
                stated,
                recomputed: expected,
            });
        }
    }
    if report.answered + report.outstanding != recomputed.rows() {
        return Err(
            MaturitySafetyReportRefusal::AnsweredAndOutstandingDoNotPartitionTheMatrix {
                rows: recomputed.rows(),
                answered: report.answered,
                outstanding: report.outstanding,
            },
        );
    }
    done.insert(MaturityRecomputedItem::AnsweredAndOutstanding);

    let supported = completeness_of(recomputed);
    if report.completeness != supported {
        return Err(MaturitySafetyReportRefusal::CompletenessDiffers {
            stated: report.completeness,
            recomputed: supported,
        });
    }
    done.insert(MaturityRecomputedItem::Completeness);

    Ok(())
}

/// Recompute the witnessed refusals against the plan's own discharges.
fn validate_witnessed(
    report: &MaturityAnnouncementSafetyReport,
    plan: &MaturityAnnouncementEvidencePlan,
    done: &mut BTreeSet<MaturityRecomputedItem>,
) -> Result<(), MaturitySafetyReportRefusal> {
    let recomputed = witnessed_refusals(plan);
    if report.witnessed.len() != recomputed.len() {
        return Err(MaturitySafetyReportRefusal::WitnessedRefusalCountDiffers {
            stated: report.witnessed.len(),
            recomputed: recomputed.len(),
        });
    }
    if recomputed.len() != plan.census().first_party_discharged() {
        return Err(MaturitySafetyReportRefusal::WitnessedRefusalCountDiffers {
            stated: recomputed.len(),
            recomputed: plan.census().first_party_discharged(),
        });
    }
    if recomputed.len() != plan.discharged().len() {
        return Err(MaturitySafetyReportRefusal::WitnessedRefusalCountDiffers {
            stated: recomputed.len(),
            recomputed: plan.discharged().len(),
        });
    }
    for (stated, expected) in report.witnessed.iter().zip(&recomputed) {
        if stated != expected {
            return Err(MaturitySafetyReportRefusal::WitnessedRefusalDiffers {
                section: expected.section,
                name: expected.row,
            });
        }
        if !plan.discharged().iter().any(|evidence| {
            evidence.section() == expected.section && evidence.name() == expected.row
        }) {
            return Err(MaturitySafetyReportRefusal::WitnessedRefusalDiffers {
                section: expected.section,
                name: expected.row,
            });
        }
    }
    done.insert(MaturityRecomputedItem::WitnessedRefusals);
    Ok(())
}

/// Recompute the register's set equality with the plan's waiting rows.
fn validate_register(
    report: &MaturityAnnouncementSafetyReport,
    plan: &MaturityAnnouncementEvidencePlan,
    done: &mut BTreeSet<MaturityRecomputedItem>,
) -> Result<(), MaturitySafetyReportRefusal> {
    let recomputed = waiting_rows(plan)?;
    for (against, expected) in [
        ("register", STILL_REQUIRED.len()),
        ("census", plan.census().native_run_required()),
        ("plan", recomputed.len()),
    ] {
        if report.waiting.len() != expected {
            return Err(MaturitySafetyReportRefusal::WaitingRowCountDiffers {
                against,
                stated: report.waiting.len(),
                recomputed: expected,
            });
        }
    }
    let gaps: usize = gap_census().values().sum();
    if gaps != STILL_REQUIRED.len() {
        return Err(MaturitySafetyReportRefusal::WaitingRowCountDiffers {
            against: "gap-census",
            stated: gaps,
            recomputed: STILL_REQUIRED.len(),
        });
    }
    for stated in &report.waiting {
        if !STILL_REQUIRED.iter().any(|entry| {
            entry.section() == stated.section
                && entry.row() == stated.row
                && entry.gap() == stated.gap
                && entry.link() == stated.link
        }) {
            return Err(MaturitySafetyReportRefusal::WaitingRowIsNotRegistered {
                section: stated.section,
                name: stated.row,
            });
        }
    }
    for entry in STILL_REQUIRED {
        if !recomputed
            .iter()
            .any(|row| row.section == entry.section() && row.row == entry.row())
        {
            return Err(MaturitySafetyReportRefusal::RegisteredRowIsNotWaiting {
                section: entry.section(),
                name: entry.row(),
            });
        }
    }
    done.insert(MaturityRecomputedItem::RegisterSetEquality);
    Ok(())
}

/// Recompute the obligations the report states about itself.
fn validate_report_layer(
    report: &MaturityAnnouncementSafetyReport,
    plan: &MaturityAnnouncementEvidencePlan,
    done: &mut BTreeSet<MaturityRecomputedItem>,
) -> Result<(), MaturitySafetyReportRefusal> {
    let recomputed = report_layer_requirements(plan);
    for expected in [recomputed.len(), plan.census().report_layer_required()] {
        if report.report_layer.len() != expected {
            return Err(
                MaturitySafetyReportRefusal::ReportLayerRequirementCountDiffers {
                    stated: report.report_layer.len(),
                    recomputed: expected,
                },
            );
        }
    }
    for (stated, expected) in report.report_layer.iter().zip(&recomputed) {
        if stated != expected {
            return Err(MaturitySafetyReportRefusal::ReportLayerRequirementDiffers {
                section: expected.section,
                name: expected.row,
            });
        }
    }
    done.insert(MaturityRecomputedItem::ReportLayerRequirements);
    Ok(())
}

/// Recompute the clause witness and each of its counts.
fn validate_clause(
    report: &MaturityAnnouncementSafetyReport,
    plan: &MaturityAnnouncementEvidencePlan,
    done: &mut BTreeSet<MaturityRecomputedItem>,
) -> Result<(), MaturitySafetyReportRefusal> {
    let census = plan.census();
    let recomputed = soundness_witness(census, witnessed_refusals(plan).len());
    let stated = report.clause;
    if stated.clause != recomputed.clause {
        return Err(MaturitySafetyReportRefusal::ClauseDiffers {
            stated: stated.clause,
            recomputed: recomputed.clause,
        });
    }
    for (figure, said, expected) in [
        (
            "refusals-witnessed",
            stated.refusals_witnessed,
            recomputed.refusals_witnessed,
        ),
        (
            "native-refusals-witnessed",
            stated.native_refusals_witnessed,
            recomputed.native_refusals_witnessed,
        ),
        (
            "rows-still-owed",
            stated.rows_still_owed,
            recomputed.rows_still_owed,
        ),
    ] {
        if said != expected {
            return Err(MaturitySafetyReportRefusal::ClauseCountDiffers {
                figure,
                stated: said,
                recomputed: expected,
            });
        }
    }
    if stated.standing != recomputed.standing {
        return Err(MaturitySafetyReportRefusal::ClauseStandingDiffers {
            stated: stated.standing,
            recomputed: recomputed.standing,
        });
    }
    done.insert(MaturityRecomputedItem::ClauseWitness);
    Ok(())
}

/// Validate one safety report against the plan it claims to be about.
///
/// Twelve recomputations, each recorded only after its comparison ran, in the order [`MaturityRecomputedItem::ALL`] lists them. Nothing is taken from the report: every figure it carries is recomputed here from the evidence plan, the register and the matrix, and the first disagreement refuses with the item that found it.
///
/// # Errors
///
/// [`MaturitySafetyReportRefusal`], naming the first disagreement found in the order the checks are written. [`MaturitySafetyReportRefusal::failed_item`] names the recomputation that found it.
pub fn validate_maturity_safety_report(
    report: MaturityAnnouncementSafetyReport,
    plan: &MaturityAnnouncementEvidencePlan,
) -> Result<ValidatedMaturityAnnouncementSafetyReport, MaturitySafetyReportRefusal> {
    let mut done = BTreeSet::new();
    validate_envelope(&report, plan, &mut done)?;
    validate_census(&report, plan, &mut done)?;
    validate_witnessed(&report, plan, &mut done)?;
    validate_register(&report, plan, &mut done)?;
    validate_constructor_material(&report, plan, &mut done)?;
    validate_report_layer(&report, plan, &mut done)?;
    validate_clause(&report, plan, &mut done)?;
    Ok(ValidatedMaturityAnnouncementSafetyReport {
        report,
        recomputed_items: done,
    })
}

/// The header block: what the report is and what it is bound to.
fn render_header(text: &mut String, report: &MaturityAnnouncementSafetyReport) {
    let projection = report.binding.target().projection();
    let _ = writeln!(text, "schema {}", report.schema);
    let _ = writeln!(text, "role {}", report.role.name());
    let _ = writeln!(text, "operation announce-maturity");
    let _ = writeln!(text, "target_contract {:?}", projection.version());
    let _ = writeln!(text, "execution_domain {:?}", projection.execution_domain());
    let _ = writeln!(text, "deployment {}", report.binding.deployment().name());
    let _ = writeln!(
        text,
        "executor_provenance_expectation {}",
        provenance_standing(&report.provenance)
    );
    let _ = writeln!(text, "quantifier {}", report.quantifier);
}

/// The census block, the two halves, and the completeness token.
fn render_census(text: &mut String, report: &MaturityAnnouncementSafetyReport) {
    for (bucket, value) in census_buckets(report.census) {
        let _ = writeln!(text, "{bucket} {value}");
    }
    let _ = writeln!(text, "answered {}", report.answered);
    let _ = writeln!(text, "outstanding {}", report.outstanding);
    let _ = writeln!(text, "completeness {}", report.completeness.name());
}

/// The clause block: what is witnessed, how far, and what is still owed.
fn render_clause(text: &mut String, report: &MaturityAnnouncementSafetyReport) {
    let clause = report.clause;
    let _ = writeln!(text, "clause {}", clause.clause.name());
    let _ = writeln!(text, "clause_standing {}", clause.standing.name());
    let _ = writeln!(
        text,
        "clause_refusals_witnessed {}",
        clause.refusals_witnessed
    );
    let _ = writeln!(
        text,
        "clause_native_refusals_witnessed {}",
        clause.native_refusals_witnessed
    );
    let _ = writeln!(text, "clause_rows_still_owed {}", clause.rows_still_owed);
}

/// Render only a refusal's stable variant name, excluding diagnostic payloads.
const fn continuity_refusal_name(refusal: &MaturityContinuityRefusal) -> &'static str {
    use MaturityContinuityRefusal as Refusal;
    match refusal {
        Refusal::PrefixEvidence { .. } => "PrefixEvidence",
        Refusal::TweakEvidence { .. } => "TweakEvidence",
        Refusal::ControlEvidence { .. } => "ControlEvidence",
        Refusal::Decode(_) => "Decode",
        Refusal::InputCount { .. } => "InputCount",
        Refusal::OutputCount { .. } => "OutputCount",
        Refusal::WitnessItemCount { .. } => "WitnessItemCount",
        Refusal::WitnessWidth { .. } => "WitnessWidth",
        Refusal::WitnessLowering(_) => "WitnessLowering",
        Refusal::SpentOutpoint { .. } => "SpentOutpoint",
        Refusal::MetadataDecode(_) => "MetadataDecode",
        Refusal::RetainedContext { .. } => "RetainedContext",
        Refusal::PredecessorReconstruction(_) => "PredecessorReconstruction",
        Refusal::PredecessorProgram { .. } => "PredecessorProgram",
        Refusal::PredecessorSearch(_) => "PredecessorSearch",
        Refusal::StaticRoot(_) => "StaticRoot",
        Refusal::Target(_) => "Target",
        Refusal::LeafReconstruction(_) => "LeafReconstruction",
        Refusal::LeafScript(_) => "LeafScript",
        Refusal::ControlRecipe(_) => "ControlRecipe",
        Refusal::ControlBlock(_) => "ControlBlock",
        Refusal::PredecessorPrefix(_) => "PredecessorPrefix",
        Refusal::Transition(_) => "Transition",
        Refusal::SuccessorReconstruction(_) => "SuccessorReconstruction",
        Refusal::OutputProgram(_) => "OutputProgram",
        Refusal::SuccessorPrefix(_) => "SuccessorPrefix",
        Refusal::SuccessorSearch(_) => "SuccessorSearch",
        Refusal::AcceptanceClaim(_) => "AcceptanceClaim",
    }
}

/// Append binary identity bytes in canonical lowercase hexadecimal.
fn render_identity(text: &mut String, bytes: &[u8]) {
    for byte in bytes {
        let _ = write!(text, "{byte:02x}");
    }
}

/// Material and host records are rendered separately from the standing census.
fn render_constructor_material(text: &mut String, report: &MaturityAnnouncementSafetyReport) {
    let _ = writeln!(text, "constructor_material {}", report.material.name());
    for record in &report.records {
        let source = match record.source() {
            MaturityByteSource::NodeFreeSubmitReady => "node-free-submit-ready",
            MaturityByteSource::ArchivedSubmission { .. } => "archived-submission",
        };
        let _ = write!(
            text,
            "continuity_record {} {} {source} ",
            record.section().section(),
            record.row(),
        );
        render_identity(text, record.branch().identifier());
        let _ = write!(text, " {} ", record.branch().checkpoint());
        render_identity(text, record.byte_identity());
        match record.observation() {
            MaturityContinuityObservation::ValidatedComparison {
                comparisons,
                agreements,
            } => {
                let _ = writeln!(text, " compared {comparisons} {agreements}");
            }
            MaturityContinuityObservation::RefusedMutant { refusal } => {
                let _ = writeln!(text, " refused {}", continuity_refusal_name(refusal));
            }
        }
    }
}

/// The items this validation recomputed, in the item vocabulary's order.
fn render_recomputed(text: &mut String, items: &BTreeSet<MaturityRecomputedItem>) {
    for item in items {
        let _ = writeln!(text, "recomputed {}", item.name());
    }
}

/// The three row blocks, each in the matrix's own order.
fn render_rows(text: &mut String, report: &MaturityAnnouncementSafetyReport) {
    for witness in &report.witnessed {
        let _ = writeln!(
            text,
            "witness {} {} {} {:?} {}",
            witness.section.section(),
            witness.row,
            witness.validator.entry_point(),
            witness.declared_boundary,
            witness.class
        );
    }
    for waiting in &report.waiting {
        let _ = writeln!(
            text,
            "waiting {} {} {:?} {:?}",
            waiting.section.section(),
            waiting.row,
            waiting.gap,
            waiting.link
        );
    }
    for requirement in &report.report_layer {
        let _ = writeln!(
            text,
            "report_layer {} {} {:?}",
            requirement.section.section(),
            requirement.row,
            requirement.projection
        );
    }
}

/// Render the canonical bytes of one validated safety report.
///
/// A pure function of its argument. The canonical projection renders source
/// classes and byte identities, excluding archived addresses and refusal
/// diagnostic text. Timing travels separately in [`MaturityReportTiming`].
///
/// The bytes are computed here rather than sealed into the wrapper because nothing validates them: the rows whose boundary is this serialization are carried as obligations the report states about itself, and the validation that reads these bytes is the subject of a later step. A sealed field would be a cache with no reader.
///
/// # Panics
///
/// Never: the sink is a `String`, whose writes cannot fail.
#[must_use]
pub fn render_maturity_safety_report(
    validated: &ValidatedMaturityAnnouncementSafetyReport,
) -> String {
    let mut text = String::new();
    render_header(&mut text, &validated.report);
    render_census(&mut text, &validated.report);
    render_constructor_material(&mut text, &validated.report);
    render_clause(&mut text, &validated.report);
    render_recomputed(&mut text, &validated.recomputed_items);
    render_rows(&mut text, &validated.report);
    text
}

#[cfg(test)]
mod tests {
    use super::{
        CompletenessFacts, MATURITY_SAFETY_REPORT_SCHEMA, MaturityAnnouncementSafetyReport,
        MaturityClauseStanding, MaturityRecomputedItem, MaturityRefinementClause,
        MaturityReportTiming, MaturitySafetyCompleteness, MaturitySafetyReportRefusal,
        MaturitySafetyReportRole, MaturityVolatileField, ValidatedMaturityAnnouncementSafetyReport,
        assemble_maturity_safety_report, render_maturity_safety_report,
        validate_maturity_safety_report,
    };
    use crate::matrix::EvidenceBoundary;
    use crate::maturity_evidence::tests::{absent_material, present_material};
    use crate::maturity_evidence::{
        MaturityAnnouncementEvidencePlan, MaturityConstructorMaterial,
        MaturityConstructorMaterialPresence, MaturityContinuityObservation,
        MaturityEvidenceRefusal, MaturityExecutorProvenanceExpectation, MaturityRowStanding,
        derive_maturity_evidence_plan_with,
    };
    use crate::maturity_first_party::{MaturityFirstPartyCase, maturity_first_party_cases};
    use crate::maturity_negative_half::STILL_REQUIRED;
    use crate::maturity_safety::{MaturitySafetySection, rows};
    use std::collections::BTreeSet;
    use std::sync::LazyLock;
    use std::time::Duration;

    /// Every key the canonical renderer writes, and no other.
    ///
    /// Written here rather than read out of the renderer, so a render that grew a key fails this list instead of extending it.
    const RENDERED_KEYS: &[&str] = &[
        "schema",
        "role",
        "operation",
        "target_contract",
        "execution_domain",
        "deployment",
        "executor_provenance_expectation",
        "quantifier",
        "rows",
        "first_party_discharged",
        "first_party_required",
        "native_acceptance_observed",
        "native_refusal_observed",
        "native_declared_boundary_observed",
        "native_refusal_at_unexpected_boundary",
        "native_run_required",
        "constructor_continuity_observed",
        "root_history_observed",
        "public_recovery_observed",
        "report_layer_observed",
        "report_layer_required",
        "outstanding_under_typed_non_answer",
        "infrastructure_blocked",
        "experimental",
        "answered",
        "outstanding",
        "completeness",
        "constructor_material",
        "continuity_record",
        "clause",
        "clause_standing",
        "clause_refusals_witnessed",
        "clause_native_refusals_witnessed",
        "clause_rows_still_owed",
        "recomputed",
        "witness",
        "waiting",
        "report_layer",
    ];

    /// Derive the plan the way the register and the evidence tests derive it.
    ///
    /// The expectation is stated as unstated, so no test here depends on the environment it happens to run in.
    fn derive() -> Result<MaturityAnnouncementEvidencePlan, MaturityEvidenceRefusal> {
        derive_maturity_evidence_plan_with(
            MaturityExecutorProvenanceExpectation::NotStatedByTheOperator,
            absent_material(),
        )
    }

    /// The plan every test below reads, derived once.
    static PLAN: LazyLock<MaturityAnnouncementEvidencePlan> =
        LazyLock::new(|| derive().expect("the evidence plan derives from its eight inputs"));

    static PRESENT_PLAN: LazyLock<MaturityAnnouncementEvidencePlan> = LazyLock::new(|| {
        derive_maturity_evidence_plan_with(
            MaturityExecutorProvenanceExpectation::NotStatedByTheOperator,
            present_material(),
        )
        .expect("plan with independently validated material")
    });

    fn validated_from(
        plan: &MaturityAnnouncementEvidencePlan,
    ) -> ValidatedMaturityAnnouncementSafetyReport {
        validate_maturity_safety_report(
            assemble_maturity_safety_report(plan).expect("assembly"),
            plan,
        )
        .expect("validation")
    }

    /// The assembled report for the plan above.
    fn assembled() -> MaturityAnnouncementSafetyReport {
        assemble_maturity_safety_report(&PLAN).expect("the report assembles from the plan")
    }

    /// The validated report for the plan above.
    fn validated() -> ValidatedMaturityAnnouncementSafetyReport {
        validate_maturity_safety_report(assembled(), &PLAN)
            .expect("the assembled report validates against its own plan")
    }

    /// Assemble, validate and render one plan end to end.
    fn rendered_from(plan: &MaturityAnnouncementEvidencePlan) -> String {
        let report = assemble_maturity_safety_report(plan).expect("the report assembles");
        let validated =
            validate_maturity_safety_report(report, plan).expect("the report validates");
        render_maturity_safety_report(&validated)
    }

    #[test]
    fn the_report_assembles_and_validates_on_the_derived_plan() {
        let validated = validated();
        assert_eq!(validated.report().schema(), MATURITY_SAFETY_REPORT_SCHEMA);
        assert_eq!(
            validated.report().role(),
            MaturitySafetyReportRole::MaturityAnnouncementSafety,
        );

        // Every one of the thirteen recomputations ran. The set is what the
        // wrapper is, so a validation that skipped one could not produce
        // this value at all.
        let every: BTreeSet<MaturityRecomputedItem> =
            MaturityRecomputedItem::ALL.iter().copied().collect();
        assert_eq!(validated.recomputed_items(), &every);
        assert_eq!(MaturityRecomputedItem::ALL.len(), 13);
        assert_eq!(validated_from(&PRESENT_PLAN).recomputed_items(), &every);
    }

    #[test]
    fn two_derivations_of_one_plan_render_identical_canonical_bytes() {
        for material in [absent_material(), present_material()] {
            let derive = |material: MaturityConstructorMaterial| {
                derive_maturity_evidence_plan_with(
                    MaturityExecutorProvenanceExpectation::NotStatedByTheOperator,
                    material,
                )
                .expect("independent derivation")
            };
            let first = derive(material.clone());
            let second = derive(material);
            assert_eq!(rendered_from(&first), rendered_from(&second));
        }
    }

    #[test]
    fn the_canonical_bytes_do_not_move_with_the_timing() {
        let quick = MaturityReportTiming::measured(Duration::from_nanos(1_234_567));
        let slow = MaturityReportTiming::measured(Duration::from_nanos(7_654_321));

        // The two timings are genuinely different and each says so in its
        // own bytes, which is where a duration belongs.
        assert_ne!(quick.render(), slow.render());
        assert!(quick.render().contains("canonical false"));
        assert!(
            slow.render()
                .contains(&slow.derivation().as_nanos().to_string()),
            "a timing carries its own duration in its own bytes",
        );

        // And neither reaches the canonical bytes. The renderer takes no
        // timing argument, so this is a statement about what the digits
        // could have been rather than about a filter that removed them.
        for plan in [&*PLAN, &*PRESENT_PLAN] {
            let bytes = rendered_from(plan);
            assert!(!bytes.contains(&quick.derivation().as_nanos().to_string()));
            assert!(!bytes.contains(&slow.derivation().as_nanos().to_string()));
        }
    }

    #[test]
    fn the_completeness_is_partial_at_this_tip() {
        let validated = validated();
        let census = PLAN.census();

        assert!(!census.every_required_row_is_answered());
        assert_eq!(census.native_refusal_at_unexpected_boundary(), 0);
        assert_eq!(census.infrastructure_blocked(), 0);
        assert_eq!(
            validated.report().completeness(),
            MaturitySafetyCompleteness::PartialRequiredRowsOutstanding,
        );

        // The two halves partition the matrix, summed from both sides
        // rather than one subtracted from the other.
        assert_eq!(
            validated.report().answered() + validated.report().outstanding(),
            census.rows(),
        );
        assert_eq!(
            validated.report().answered(),
            census.first_party_discharged() + census.native_acceptance_observed()
        );
        assert_eq!(validated.report().answered(), census.answered());
        assert_eq!(validated.report().outstanding(), census.outstanding());
        let rendered = render_maturity_safety_report(&validated);
        for (key, figure) in [
            (
                "native_declared_boundary_observed",
                census.native_declared_boundary_observed(),
            ),
            ("native_run_required", census.native_run_required()),
            ("answered", census.answered()),
            ("outstanding", census.outstanding()),
        ] {
            assert!(rendered.contains(&format!("{key} {figure}\n")));
        }
        let present = validated_from(&PRESENT_PLAN);
        assert_eq!(present.report().census(), census);
        assert_eq!(
            present.report().completeness(),
            validated.report().completeness()
        );
    }

    #[test]
    fn a_contradiction_and_a_block_each_fail_the_report_before_a_waiting_row_does() {
        // A contradiction takes precedence over ordinary outstanding rows:
        // a row refused away from its declared boundary must not render as
        // a row still waiting.
        let contradicted = CompletenessFacts {
            contradicted: 1,
            blocked: 0,
            every_required_row_is_answered: false,
        };
        assert_eq!(
            contradicted.completeness(),
            MaturitySafetyCompleteness::Failed,
        );

        // A missing component fails for the other reason: the row was
        // never asked its question, so naming a wait would name one that
        // no run ends.
        let blocked = CompletenessFacts {
            contradicted: 0,
            blocked: 1,
            every_required_row_is_answered: true,
        };
        assert_eq!(blocked.completeness(), MaturitySafetyCompleteness::Failed);

        let waiting = CompletenessFacts {
            contradicted: 0,
            blocked: 0,
            every_required_row_is_answered: false,
        };
        assert_eq!(
            waiting.completeness(),
            MaturitySafetyCompleteness::PartialRequiredRowsOutstanding,
        );

        let clear = CompletenessFacts {
            contradicted: 0,
            blocked: 0,
            every_required_row_is_answered: true,
        };
        assert_eq!(
            clear.completeness(),
            MaturitySafetyCompleteness::CompleteForTheRequiredMatrix,
        );

        assert_eq!(MaturitySafetyCompleteness::Failed.name(), "failed");
    }

    #[test]
    fn the_validation_refuses_an_altered_answered_figure_naming_its_item() {
        let mut report = assembled();
        report.answered += 1;

        let refusal = validate_maturity_safety_report(report, &PLAN)
            .expect_err("a figure the census does not support is refused");
        assert_eq!(
            refusal.failed_item(),
            Some(MaturityRecomputedItem::AnsweredAndOutstanding),
        );
        assert!(matches!(
            refusal,
            MaturitySafetyReportRefusal::AnsweredOrOutstandingDiffers { figure, .. }
                if figure == "answered"
        ));
    }

    #[test]
    fn the_validation_refuses_an_altered_completeness_token() {
        let mut report = assembled();
        report.completeness = MaturitySafetyCompleteness::CompleteForTheRequiredMatrix;

        let refusal = validate_maturity_safety_report(report, &PLAN)
            .expect_err("a completeness the census does not support is refused");
        assert_eq!(
            refusal.failed_item(),
            Some(MaturityRecomputedItem::Completeness),
        );
    }

    #[test]
    fn the_validation_refuses_a_raised_witness_count() {
        let mut report = assembled();
        report.clause.refusals_witnessed += 1;

        let refusal = validate_maturity_safety_report(report, &PLAN)
            .expect_err("a witness standing on more refusals than were driven is refused");
        assert_eq!(
            refusal.failed_item(),
            Some(MaturityRecomputedItem::ClauseWitness),
        );
        assert!(matches!(
            refusal,
            MaturitySafetyReportRefusal::ClauseCountDiffers { figure, .. }
                if figure == "refusals-witnessed"
        ));
    }

    #[test]
    fn the_validation_refuses_a_truncated_row_block() {
        let mut short_witness = assembled();
        short_witness.witnessed.pop();
        assert_eq!(
            validate_maturity_safety_report(short_witness, &PLAN)
                .expect_err("a dropped witness is refused")
                .failed_item(),
            Some(MaturityRecomputedItem::WitnessedRefusals),
        );

        let mut short_waiting = assembled();
        short_waiting.waiting.pop();
        assert_eq!(
            validate_maturity_safety_report(short_waiting, &PLAN)
                .expect_err("a dropped waiting row is refused")
                .failed_item(),
            Some(MaturityRecomputedItem::RegisterSetEquality),
        );

        let mut short_report_layer = assembled();
        short_report_layer.report_layer.pop();
        assert_eq!(
            validate_maturity_safety_report(short_report_layer, &PLAN)
                .expect_err("a dropped report-layer requirement is refused")
                .failed_item(),
            Some(MaturityRecomputedItem::ReportLayerRequirements),
        );
    }

    #[test]
    fn the_validation_refuses_an_unread_schema() {
        let mut report = assembled();
        report.schema = MATURITY_SAFETY_REPORT_SCHEMA + 1;

        let refusal = validate_maturity_safety_report(report, &PLAN)
            .expect_err("a schema this validation does not read is refused");
        assert_eq!(refusal.failed_item(), Some(MaturityRecomputedItem::Schema));
    }

    #[test]
    fn the_exclusions_are_the_guides_eleven_and_none_is_a_rendered_key() {
        assert_eq!(MaturityVolatileField::ALL.len(), 11);

        let names: BTreeSet<&str> = MaturityVolatileField::ALL
            .iter()
            .map(|field| field.name())
            .collect();
        assert_eq!(names.len(), 11, "every exclusion has its own spelling");

        // The closed key set and the eleven exclusions are disjoint, which
        // is what makes the key check below a statement about §14.9 rather
        // than about tidiness.
        for key in RENDERED_KEYS {
            assert!(!names.contains(key), "{key} is one of the excluded fields");
        }
    }

    #[test]
    fn the_canonical_bytes_carry_no_volatile_field() {
        for plan in [&*PLAN, &*PRESENT_PLAN] {
            assert_canonical_fields(&validated_from(plan));
        }
    }

    fn assert_canonical_fields(validated: &ValidatedMaturityAnnouncementSafetyReport) {
        let bytes = render_maturity_safety_report(validated);

        // Every block is as long as the value it was rendered from, so a
        // block that silently stopped rendering fails here rather than
        // passing a shorter document.
        let key_count = |wanted: &str| {
            bytes
                .lines()
                .filter(|line| line.split(' ').next() == Some(wanted))
                .count()
        };
        assert_eq!(
            key_count("recomputed"),
            validated.recomputed_items().len(),
            "every recomputed item renders",
        );
        assert_eq!(key_count("witness"), validated.report().witnessed().len());
        assert_eq!(key_count("waiting"), validated.report().waiting().len());
        assert_eq!(key_count("constructor_material"), 1);
        assert_eq!(
            key_count("continuity_record"),
            validated.report().continuity_records().len()
        );
        assert_eq!(
            key_count("report_layer"),
            validated.report().report_layer().len(),
        );

        for line in bytes.lines() {
            let key = line
                .split(' ')
                .next()
                .expect("a rendered line has a first token");
            assert!(
                RENDERED_KEYS.contains(&key),
                "the renderer wrote an unlisted key: {key}",
            );
            assert!(
                line.len() > key.len() + 1,
                "every rendered line carries a value: {line}",
            );
        }

        // Rendering is a function of the validated report and nothing
        // else, so the same value renders the same bytes twice.
        assert_eq!(bytes, render_maturity_safety_report(validated));
    }

    #[test]
    fn the_witness_stands_on_the_discharged_rows_and_names_what_it_is_owed() {
        let validated = validated();
        let clause = validated.report().clause();
        let census = PLAN.census();

        assert_eq!(
            clause.clause(),
            MaturityRefinementClause::SoundnessOfRefusedSteps
        );
        assert_eq!(
            clause.standing(),
            MaturityClauseStanding::PartialOverTheRefusalsObserved,
        );

        // Recomputed from the plan on both sides: the witness stands on
        // every row a first-party validator refused, and on no other.
        let discharged = PLAN
            .rows()
            .iter()
            .filter(|row| {
                matches!(
                    row.standing(),
                    MaturityRowStanding::FirstPartyDischarged { .. }
                )
            })
            .count();
        assert_eq!(clause.refusals_witnessed(), discharged);
        assert_eq!(clause.refusals_witnessed(), census.first_party_discharged());
        assert_eq!(clause.refusals_witnessed(), PLAN.discharged().len());
        assert_eq!(validated.report().witnessed().len(), discharged);

        // No target refusal exists yet, and the owed figure is the rows
        // this report has not answered.
        assert_eq!(clause.native_refusals_witnessed(), 0);
        assert_eq!(clause.rows_still_owed(), census.rows() - census.answered());

        // Every witnessed refusal names the boundary its row declared and
        // the class its own discharge expects, which is what makes it a
        // witness of the clause rather than a refusal about something
        // else that happened to be raised nearby.
        let cases = maturity_first_party_cases();
        for witness in validated.report().witnessed() {
            let identity = (witness.section(), witness.row());
            let row = rows()
                .iter()
                .find(|row| (row.section(), row.name()) == identity)
                .expect("a witnessed row is a row of the matrix");
            assert_eq!(Some(witness.declared_boundary()), row.refusing_layer());

            let discharge = cases
                .iter()
                .find(|case| (case.section(), case.name()) == identity)
                .and_then(MaturityFirstPartyCase::discharge)
                .expect("a witnessed row carries a discharge");
            assert_eq!(witness.class(), discharge.expected_class());
            assert_eq!(witness.validator(), discharge.validator());
        }
    }

    #[test]
    fn the_report_layer_requirements_are_the_matrixs_report_rows() {
        let validated = validated();
        let stated = validated.report().report_layer();

        let expected: Vec<_> = PLAN
            .rows()
            .iter()
            .filter_map(|row| match row.standing() {
                MaturityRowStanding::ReportLayerRequired(projection) => {
                    Some((row.row().section(), row.row().name(), *projection))
                }
                _ => None,
            })
            .collect();

        assert_eq!(stated.len(), expected.len());
        assert_eq!(stated.len(), PLAN.census().report_layer_required());
        for (requirement, (section, name, projection)) in stated.iter().zip(&expected) {
            assert_eq!(requirement.section(), *section);
            assert_eq!(requirement.row(), *name);
            assert_eq!(requirement.projection(), *projection);
        }

        // The subject of these rows is this report's own serialization,
        // whichever table each is drawn from. The matrix spreads them
        // across more than one, so what makes a row one of these is its
        // declared boundary and never the table it sits in.
        for requirement in stated {
            let identity = (requirement.section(), requirement.row());
            let row = rows()
                .iter()
                .find(|row| (row.section(), row.name()) == identity)
                .expect("a report-layer requirement is a row of the matrix");
            assert_eq!(
                row.refusing_layer(),
                Some(EvidenceBoundary::ReportSemanticProjectionRejection),
            );
            assert_eq!(row.projection(), requirement.projection());
        }
    }

    #[test]
    fn the_register_is_exactly_the_plans_waiting_rows() {
        let validated = validated();
        let waiting = validated.report().waiting();

        let plan_waiting: BTreeSet<(MaturitySafetySection, &'static str)> = PLAN
            .rows()
            .iter()
            .filter(|row| matches!(row.standing(), MaturityRowStanding::NativeRunRequired(_)))
            .map(|row| (row.row().section(), row.row().name()))
            .collect();
        let registered: BTreeSet<(MaturitySafetySection, &'static str)> = STILL_REQUIRED
            .iter()
            .map(|entry| (entry.section(), entry.row()))
            .collect();

        assert_eq!(plan_waiting, registered, "the two sets are one set");
        assert_eq!(waiting.len(), registered.len());
        assert_eq!(waiting.len(), PLAN.census().native_run_required());
        let present = validated_from(&PRESENT_PLAN);
        assert_eq!(present.report().waiting(), waiting);
        assert_eq!(PRESENT_PLAN.rows(), PLAN.rows());

        // Each carried gap is the register's own rather than a second
        // opinion kept beside it.
        for row in waiting {
            let entry = STILL_REQUIRED
                .iter()
                .find(|entry| entry.section() == row.section() && entry.row() == row.row())
                .expect("a waiting row is a registered row");
            assert_eq!(row.gap(), entry.gap());
            assert_eq!(row.link(), entry.link());
        }
    }

    fn hex(bytes: &[u8]) -> String {
        use std::fmt::Write as _;
        let mut text = String::new();
        for byte in bytes {
            write!(text, "{byte:02x}").expect("String write");
        }
        text
    }

    #[test]
    fn present_constructor_material_is_validated_and_rendered() {
        let validated = validated_from(&PRESENT_PLAN);
        let report = validated.report();
        assert_eq!(
            report.constructor_material(),
            MaturityConstructorMaterialPresence::Present
        );
        assert_eq!(
            report.continuity_records(),
            PRESENT_PLAN.continuity_records()
        );
        assert!(MaturityRecomputedItem::ALL.windows(3).any(|items| items
            == [
                MaturityRecomputedItem::RegisterSetEquality,
                MaturityRecomputedItem::ConstructorContinuityMaterial,
                MaturityRecomputedItem::ReportLayerRequirements,
            ]));
        assert!(
            validated
                .recomputed_items()
                .contains(&MaturityRecomputedItem::ConstructorContinuityMaterial)
        );
        let bytes = render_maturity_safety_report(&validated);
        assert!(bytes.contains("constructor_material present\n"));
        let lines: Vec<_> = bytes
            .lines()
            .filter(|line| line.starts_with("continuity_record "))
            .collect();
        assert_eq!(lines.len(), PRESENT_PLAN.continuity_records().len());
        for (line, record) in lines.iter().zip(PRESENT_PLAN.continuity_records()) {
            let source = match record.source() {
                crate::maturity_continuity::MaturityByteSource::NodeFreeSubmitReady => {
                    "node-free-submit-ready"
                }
                crate::maturity_continuity::MaturityByteSource::ArchivedSubmission {
                    run_address,
                } => {
                    assert!(!bytes.contains(run_address));
                    "archived-submission"
                }
            };
            let observation = match record.observation() {
                MaturityContinuityObservation::ValidatedComparison {
                    comparisons,
                    agreements,
                } => format!("compared {comparisons} {agreements}"),
                MaturityContinuityObservation::RefusedMutant { refusal } => {
                    assert!(matches!(
                        refusal,
                        crate::maturity_continuity::MaturityContinuityRefusal::RetainedContext { .. }
                    ));
                    "refused RetainedContext".to_owned()
                }
            };
            assert_eq!(
                *line,
                format!(
                    "continuity_record {} {} {source} {} {} {} {observation}",
                    record.section().section(),
                    record.row(),
                    hex(record.branch().identifier()),
                    record.branch().checkpoint(),
                    hex(record.byte_identity()),
                )
            );
        }
    }

    #[test]
    fn validation_refuses_an_altered_constructor_presence() {
        for (plan, altered) in [
            (&*PLAN, MaturityConstructorMaterialPresence::Present),
            (&*PRESENT_PLAN, absent_material().presence()),
        ] {
            let mut report = assemble_maturity_safety_report(plan).expect("assembly");
            report.material = altered;
            let refusal =
                validate_maturity_safety_report(report, plan).expect_err("altered presence");
            assert_eq!(
                refusal,
                MaturitySafetyReportRefusal::ConstructorMaterialPresenceDiffers {
                    stated: altered,
                    recomputed: plan.constructor_material().presence(),
                }
            );
            assert_eq!(
                refusal.failed_item(),
                Some(MaturityRecomputedItem::ConstructorContinuityMaterial)
            );
        }
    }

    #[test]
    fn validation_refuses_altered_continuity_records() {
        let original = assemble_maturity_safety_report(&PRESENT_PLAN).expect("assembly");
        let mut replaced = original.clone();
        replaced.records[0] = replaced.records[2].clone();
        let mut reordered = original.clone();
        reordered.records.swap(0, 1);
        let mut removed = original.clone();
        removed.records.pop();
        let mut duplicated = original.clone();
        duplicated.records.push(original.records[0].clone());
        for report in [replaced, reordered, removed, duplicated] {
            let refusal = validate_maturity_safety_report(report, &PRESENT_PLAN)
                .expect_err("altered records");
            assert_eq!(
                refusal,
                MaturitySafetyReportRefusal::ContinuityRecordsDiffer
            );
            assert_eq!(
                refusal.failed_item(),
                Some(MaturityRecomputedItem::ConstructorContinuityMaterial)
            );
        }
    }

    #[test]
    fn material_keys_are_exact_and_in_canonical_block_order() {
        assert_eq!(RENDERED_KEYS.len(), 38);
        for plan in [&*PLAN, &*PRESENT_PLAN] {
            let bytes = rendered_from(plan);
            let keys: Vec<_> = bytes
                .lines()
                .map(|line| line.split_once(' ').expect("key and value").0)
                .collect();
            let actual: BTreeSet<_> = keys.iter().copied().collect();
            let expected: BTreeSet<_> = RENDERED_KEYS
                .iter()
                .copied()
                .filter(|key| *key != "continuity_record" || !plan.continuity_records().is_empty())
                .collect();
            assert_eq!(actual, expected);
            let positions: Vec<_> = keys
                .iter()
                .map(|key| {
                    RENDERED_KEYS
                        .iter()
                        .position(|expected| expected == key)
                        .expect("listed key")
                })
                .collect();
            assert!(positions.windows(2).all(|pair| pair[0] <= pair[1]));
            assert!(bytes.contains(&format!(
                "constructor_material {}\n",
                plan.constructor_material().presence().name()
            )));
        }
    }

    #[test]
    fn the_wire_spellings_stay_distinct() {
        let completeness: BTreeSet<&str> = [
            MaturitySafetyCompleteness::CompleteForTheRequiredMatrix,
            MaturitySafetyCompleteness::PartialRequiredRowsOutstanding,
            MaturitySafetyCompleteness::Failed,
        ]
        .iter()
        .map(|token| token.name())
        .collect();
        assert_eq!(completeness.len(), 3);

        let standings: BTreeSet<&str> = [
            MaturityClauseStanding::WitnessedOverEveryRow,
            MaturityClauseStanding::PartialOverTheRefusalsObserved,
            MaturityClauseStanding::ContradictedAtAnObservedBoundary,
        ]
        .iter()
        .map(|standing| standing.name())
        .collect();
        assert_eq!(standings.len(), 3);

        let items: BTreeSet<&str> = MaturityRecomputedItem::ALL
            .iter()
            .map(|item| item.name())
            .collect();
        assert_eq!(items.len(), MaturityRecomputedItem::ALL.len());
        assert_eq!(
            MaturityRecomputedItem::ConstructorContinuityMaterial.name(),
            "constructor-continuity-material"
        );
        assert_eq!(
            MaturityConstructorMaterialPresence::Present.name(),
            "present"
        );
        assert_eq!(absent_material().presence().name(), "absent");

        assert_eq!(
            MaturityRefinementClause::SoundnessOfRefusedSteps.name(),
            "soundness-of-refused-steps",
        );
    }
}
