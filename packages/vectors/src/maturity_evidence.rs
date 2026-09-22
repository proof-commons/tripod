//! The canonical maturity-announcement evidence plan of §14.1, its
//! standings, and the closed observation vocabulary they stand inside.
//!
//! §14.1 gives the plan private fields, no public constructor accepting a
//! caller-authored verdict, and eight inputs it is built from. This module
//! is that plan: a value obtainable only from
//! [`derive_maturity_evidence_plan_with`], which retains explicit premises,
//! builds the other inputs and classifies every safety-matrix row.
//!
//! # Why the derivation takes the eight inputs and nothing else
//!
//! A plan assembled field by field would be a caller's opinion about what
//! has been established, and a plan that read an input from somewhere the
//! signature does not mention would be making a claim its own type cannot
//! be checked against. Executor provenance and constructor material are
//! explicit arguments: a comparison that manufactured its own premises
//! would supply both operands. The other six inputs are derived here.
//! [`derive_maturity_evidence_plan`] is the convenience that obtains that
//! argument through [`stated_executor_provenance`] and then calls the
//! derivation, so the ambient read has one site and it is not inside the
//! derivation.
//!
//! # Material and history have distinct obligations
//!
//! Validated host comparisons and observed projector refusals are records
//! associated with exact rows. They carry source and caller-stated branch
//! context without authenticating freshness or target acceptance. Their
//! acceptance vocabulary contains only an outstanding obligation, so they
//! cannot mint an accepted-continuity standing. Missing constructor material
//! is an explicit input with a typed reason. The root-history registry stays
//! outstanding because branch-indexed history cannot be projected.
//!
//! # What an answered standing is here
//!
//! This plan's answered standings are exactly the ones it recomputed or
//! observed. A row a first-party validator refused is answered because
//! [`crate::maturity_first_party::discharge_maturity_first_party`] drove
//! that validator here, twice, and came back with the refusal; a row whose
//! evidence stands in another census is not answered here, because taking
//! another census's word for a row is the discharge-by-assertion the whole
//! classification exists to prevent. That is why
//! [`MaturityRowStanding::OutstandingUnderTypedNonAnswer`] is outstanding
//! even where the matrix's own reason says the row's evidence landed
//! elsewhere: the reason names where to look, and this plan did not look.
//!
//! [`MaturityRowStanding::NativeRefusalAtUnexpectedBoundary`] is not
//! answered either, and for a different reason: the run happened, the
//! control was accepted and the target's words are carried exactly, but
//! the row asked whether a particular layer refuses a particular fault and
//! the target refused earlier, so the layer the row names was never
//! reached. Reading it as an answer would claim a covenant clause ran when
//! the transaction never got that far.
//! [`MaturityRowStanding::InfrastructureBlocked`] is not answered because a
//! failed environment is never evidence that a change was refused. Both
//! stand in their own census buckets, and both keep
//! [`MaturityEvidenceCensus::every_required_row_is_answered`] false while
//! they are non-zero.
//!
//! # Why the poison marker lives here, beside the standings
//!
//! The observation vocabulary is closed over three classes: intended
//! transitions; modeled-but-unintended, rely-conforming environment
//! transitions; and model-falsifying observations with no preimage under
//! the abstraction relation, which are representable only as a marker
//! carrying verbatim evidence bytes. A marker voids derived guarantees on
//! its branch context and can never be absorbed into an ordinary standing,
//! and that sentence is worth more as a compile-time fact than as a check
//! some report runs. [`MaturityBranchPoisonMarker`] is therefore declared
//! beside [`MaturityRowStanding`], and the exclusion is a property of the
//! standing's own fields: every field of every member is one of
//! [`crate::maturity_first_party::MaturityFirstPartyValidator`],
//! [`crate::maturity_first_party::MaturityCarriedReason`],
//! [`crate::maturity_safety::MaturityRowBoundary`],
//! [`crate::maturity_safety::MaturityExpectedProjection`],
//! [`crate::maturity_safety::MaturityIntendedCarrier`],
//! [`crate::maturity_safety::MaturityMutationLocator`],
//! [`crate::matrix::EvidenceBoundary`], an observed outcome layer, a
//! published requirement identity, a target-computed identity, a string, a
//! byte vector, [`MaturityAcceptedControl`] — whose two fields are a
//! canonical control shape and such an identity — or
//! [`MaturityCarrierOutcome`], and none of them is the marker or
//! transitively contains one. The marker offers no conversion,
//! no `From`, and no rendering into any of those types, so there is no
//! expression that places one in a standing: a program that tried would
//! not compile. Its bytes travel verbatim and are never paraphrased into
//! this workspace's vocabulary, which is what keeps a falsifying
//! observation checkable by a reader who does not trust this crate.
//!
//! # Every guarantee here carries the class quantifier
//!
//! Nothing this plan states is claimed unconditionally. Its guarantees
//! hold over the first two observation classes and are conditional on no
//! observation of the third, and that quantifier is rendered as a value by
//! [`MaturityEvidenceCensus::quantifier`] rather than left as a sentence
//! in a comment, so a report carrying a figure from this census carries
//! the condition the figure was computed under.
//!
//! # Nothing here is a run
//!
//! No target is asked anything by this module. The admitted corpus supplies
//! its recorded boundary observation. Constructor records preserve host
//! comparisons separately from row standings, and the census counts only
//! those standings.

use std::collections::BTreeMap;
use std::fmt;

use compiler::maturity_announcement_plan::{
    MaturityAnnouncementRepresentationPlan, ValidatedMaturityAnnouncementOperationPlan,
};
use compiler::operation_plan::{CoverageRequirementId, SponsorCase};
use linker::{CandidateLinkedMaturityBundle, StateLinkDeploymentParameters};
use target_elements::ReviewedElementsTapscriptDefinition;
use target_elements_conformance::protocol::ObservedOutcomeLayer;
use target_elements_conformance::provenance::{ExpectedExecutorProvenance, ProvenanceSyntaxDefect};
use transaction::bytes::{AssetField, AssetId, Outpoint, Txid, ValueField};
use transaction::error::TransactionRefusal;
use transaction::operator_right::{BranchContext, RightRefusal};
use transaction::state_abi::{CandidateMaturityAnnouncementAbi, derive_maturity_announcement_abi};
use transaction::state_view::{
    MaturityViewStatement, PublicMaturityStateView, ValidatedMaturityStateView,
};

use crate::error::VectorError;
use crate::matrix::EvidenceBoundary;
use crate::maturity_closure::{
    MaturityClosureRefusal, MaturityDeployment, OracleStateCurve, announcement_plan,
    closure_target, linked_maturity_bundle,
};
use crate::maturity_continuity::{
    MaturityByteComparison, MaturityByteSource, MaturityContinuityMutant,
    MaturityContinuityRefusal, MaturityFieldComparison,
};
use crate::maturity_continuity_report::{
    MaturityContinuityReportEntry, ValidatedMaturityContinuityReport,
};
use crate::maturity_corpus::{
    MATURITY_RUN_ADDRESS, MaturityCorpusImportRefusal, maturity_run_of_record,
};
use crate::maturity_first_party::{
    MaturityCarriedReason, MaturityFirstPartyRefusal, MaturityFirstPartyValidator,
    ValidatedMaturityFirstPartyEvidence, discharge_maturity_first_party,
    maturity_first_party_cases,
};
use crate::maturity_fixture::{MaturitySemanticCase, positive_semantic_census};
use crate::maturity_native::{MaturityAcceptanceObligation, MaturityNativeStanding};
use crate::maturity_safety::{
    MaturityCanonicalControl, MaturityExpectedProjection, MaturityIntendedCarrier,
    MaturityMutationLocator, MaturityMutationSubject, MaturityRowBoundary, MaturityRowLink,
    MaturitySafetyRow, MaturitySafetySection, resolve_row, rows,
};
use crate::observed_boundary::matches_boundary;
use crate::subject::{CanonicalSubject, ExperimentalSubject};

/// The deployment this plan's bundle, view and candidate ABI are taken
/// over.
///
/// The one whose committed operator key a published test signer holds.
/// The other two commit meaningless public fill, for which no signature
/// can be produced at all, and it is the deployment the first-party
/// evidence this plan reads was itself recomputed over: a plan whose
/// bundle and whose first-party census stood over two different
/// deployments would be holding evidence against a bundle nobody produced
/// it from.
const DEPLOYMENT: MaturityDeployment = MaturityDeployment::PublishedSignerHeld;

/// The transaction this module states the current STATE output sits in.
///
/// A caller statement rather than a chain fact, and it is one the view
/// validates rather than trusts. The candidate ABI is derived from the
/// validated view, and nothing in this module reads a chain, so what this
/// position has to be is well-formed and stated in one place.
const PREDECESSOR_TXID: [u8; 32] = [0x2e; 32];

/// The branch identifier this module binds the current root at.
///
/// Also a caller statement. A branch identity is the index a root-history
/// claim would be relative to, and this module makes no such claim: it
/// states the binding the view requires and leaves the history to the
/// registry that stands outstanding for it.
const BRANCH: [u8; 32] = [0x4d; 32];

/// The checkpoint ordinal stated beside the branch.
const CHECKPOINT: u64 = 7;

/// The environment variable naming the tip an executor was built from.
const EXECUTOR_TIP: &str = "TRIPOD_OPERATION_EXECUTOR_TIP";

/// The environment variable naming the upstream base that tip derives
/// from.
const UPSTREAM_BASE: &str = "TRIPOD_OPERATION_UPSTREAM_BASE";

/// The environment variable naming the local topics folded into the tip.
const LOCAL_TOPICS: &str = "TRIPOD_OPERATION_LOCAL_TOPICS";

/// The closed observation vocabulary.
///
/// Three classes and no fourth, and the enum is deliberately not open: a
/// vocabulary whose whole content is that it is closed cannot be stated by
/// a type that admits additions. The members carry no data, so no value of
/// this type can hold evidence of any kind — the third class's evidence
/// lives on [`MaturityBranchPoisonMarker`] and nowhere else.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityObservationClass {
    /// A transition this workspace intended: the announcement it built,
    /// behaving as its semantics say.
    IntendedTransition,
    /// A transition the model admits but this workspace did not intend,
    /// made by a rely-conforming environment — a competing spend, another
    /// instance under the same keys, a realization surfaced by a
    /// reorganization.
    ModeledUnintendedEnvironmentTransition,
    /// An observation with no preimage under the abstraction relation:
    /// the target moved in a way the model does not admit at all.
    ///
    /// Representable only as [`MaturityBranchPoisonMarker`]. It voids
    /// derived guarantees on its branch context, and no standing of this
    /// module can hold it.
    ModelFalsifyingWithNoPreimage,
}

impl MaturityObservationClass {
    /// Every class, in the vocabulary's own order.
    pub const ALL: &'static [Self] = &[
        Self::IntendedTransition,
        Self::ModeledUnintendedEnvironmentTransition,
        Self::ModelFalsifyingWithNoPreimage,
    ];

    /// Whether a guarantee of this module is quantified over this class.
    ///
    /// True for the first two and false for the third, which is the whole
    /// of the quantifier: a guarantee holds over intended and
    /// modeled-but-unintended observations, and an observation of the
    /// third class voids it rather than being an instance of it.
    #[must_use]
    pub const fn is_quantified_over(self) -> bool {
        matches!(
            self,
            Self::IntendedTransition | Self::ModeledUnintendedEnvironmentTransition
        )
    }
}

/// A model-falsifying observation, carried as bytes and nothing else.
///
/// # What may occupy it
///
/// The verbatim bytes of an observation for which no semantic step of the
/// model is a preimage, together with the branch context the observation
/// was made under, because what a marker voids is the derived guarantees
/// on that branch and not the model everywhere.
///
/// # What it must never become
///
/// A standing. The bytes are not paraphrased into a refusal detail, not
/// classified into a layer, and not compared against any expectation:
/// every one of those operations would file an observation the model
/// cannot explain under a vocabulary that presumes it can. No member of
/// [`MaturityRowStanding`] has a field of this type, this type offers no
/// conversion into any type such a field has, and it renders itself into
/// no string, so the exclusion is a fact about what compiles rather than a
/// rule somebody has to remember.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityBranchPoisonMarker {
    branch: BranchContext,
    evidence: Vec<u8>,
}

impl MaturityBranchPoisonMarker {
    /// Record one model-falsifying observation.
    ///
    /// Named for what it does: a marker is an observation somebody made,
    /// never a verdict this crate reached, and the bytes are taken exactly
    /// as they arrived.
    #[must_use]
    pub const fn observe(branch: BranchContext, evidence: Vec<u8>) -> Self {
        Self { branch, evidence }
    }

    /// The branch context whose derived guarantees this marker voids.
    #[must_use]
    pub const fn branch(&self) -> &BranchContext {
        &self.branch
    }

    /// The observation's bytes, exactly as they were observed.
    #[must_use]
    pub fn evidence(&self) -> &[u8] {
        &self.evidence
    }

    /// The class a marker always is.
    #[must_use]
    pub const fn class(&self) -> MaturityObservationClass {
        MaturityObservationClass::ModelFalsifyingWithNoPreimage
    }
}

/// The quantifier every guarantee of this plan is stated under.
///
/// Carried as a value rather than as a sentence so that a report
/// rendering a figure from this census renders the condition the figure
/// was computed under beside it. Two admitted classes and one excluded
/// class, which is the whole statement.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MaturityGuaranteeQuantifier {
    admitted: [MaturityObservationClass; 2],
    excluded: MaturityObservationClass,
}

impl MaturityGuaranteeQuantifier {
    /// The quantifier this module's guarantees carry.
    const fn of_this_plan() -> Self {
        Self {
            admitted: [
                MaturityObservationClass::IntendedTransition,
                MaturityObservationClass::ModeledUnintendedEnvironmentTransition,
            ],
            excluded: MaturityObservationClass::ModelFalsifyingWithNoPreimage,
        }
    }

    /// The classes a guarantee is quantified over.
    #[must_use]
    pub const fn admitted(self) -> [MaturityObservationClass; 2] {
        self.admitted
    }

    /// The class whose observation voids a guarantee instead of
    /// instantiating it.
    #[must_use]
    pub const fn excluded(self) -> MaturityObservationClass {
        self.excluded
    }
}

impl fmt::Display for MaturityGuaranteeQuantifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "over intended transitions and modeled-but-unintended rely-conforming environment transitions, conditional on no model-falsifying observation with no preimage under the abstraction relation",
        )
    }
}

/// Why constructor-continuity material is absent from a derivation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityConstructorMaterialAbsence {
    /// The derivation received no constructor-continuity material.
    NotSuppliedToDerivation,
}

/// Whether a derivation received constructor-continuity material.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaturityConstructorMaterialPresence {
    /// Validated comparison material was supplied.
    Present,
    /// No material was supplied, for the stated reason.
    Absent(MaturityConstructorMaterialAbsence),
}

impl MaturityConstructorMaterialPresence {
    /// The presence token's wire spelling.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Present => "present",
            Self::Absent(_) => "absent",
        }
    }
}

/// Constructor input supplied independently of the evidence derivation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaturityConstructorMaterial {
    /// A validated continuity report and observed projector refusals.
    Present(MaturityPresentConstructorMaterial),
    /// Explicit absence preserves the input and its reason.
    Absent(MaturityConstructorMaterialAbsence),
}

impl MaturityConstructorMaterial {
    /// Whether material was supplied, preserving the reason for absence.
    #[must_use]
    pub const fn presence(&self) -> MaturityConstructorMaterialPresence {
        match self {
            Self::Present(_) => MaturityConstructorMaterialPresence::Present,
            Self::Absent(reason) => MaturityConstructorMaterialPresence::Absent(*reason),
        }
    }
}

/// Validated comparison material without any target-acceptance assertion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityPresentConstructorMaterial {
    report: Box<ValidatedMaturityContinuityReport>,
    mutants: Vec<ExperimentalSubject<MaturityContinuityMutant>>,
}

impl MaturityPresentConstructorMaterial {
    /// Retain a validated report and subjects obtainable through observation.
    #[must_use]
    pub fn new(
        report: ValidatedMaturityContinuityReport,
        mutants: Vec<ExperimentalSubject<MaturityContinuityMutant>>,
    ) -> Self {
        Self {
            report: Box::new(report),
            mutants,
        }
    }

    /// The independently validated continuity report.
    #[must_use]
    pub fn report(&self) -> &ValidatedMaturityContinuityReport {
        &self.report
    }

    /// Refused experimental subjects in supplied order.
    #[must_use]
    pub fn mutants(&self) -> &[ExperimentalSubject<MaturityContinuityMutant>] {
        &self.mutants
    }

    /// The report's outstanding acceptance obligation.
    #[must_use]
    pub fn acceptance(&self) -> MaturityAcceptanceObligation {
        self.report.report().acceptance()
    }
}

/// A host observation, separate from any row standing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaturityContinuityObservation {
    /// Comparison counts recomputed from a validated report entry.
    ValidatedComparison {
        /// Number of individual comparison results.
        comparisons: usize,
        /// Number of successful comparison results.
        agreements: usize,
    },
    /// A projector refusal over an observed mutant's exact bytes.
    RefusedMutant {
        /// The projector's refusal, preserving its diagnostics.
        refusal: MaturityContinuityRefusal,
    },
}

/// A first-party continuity record associated with one exact matrix row.
///
/// Branch context is caller-stated and makes no freshness claim. Neither
/// observation carries target-computed acceptance or a class-three marker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityContinuityRecord {
    section: MaturitySafetySection,
    row: &'static str,
    source: MaturityByteSource,
    branch: BranchContext,
    byte_identity: [u8; 32],
    observation: MaturityContinuityObservation,
}

impl MaturityContinuityRecord {
    /// The exact row's matrix section.
    #[must_use]
    pub const fn section(&self) -> MaturitySafetySection {
        self.section
    }

    /// The exact row's name within its section.
    #[must_use]
    pub const fn row(&self) -> &'static str {
        self.row
    }

    /// The source class of the compared or refused bytes.
    #[must_use]
    pub const fn source(&self) -> &MaturityByteSource {
        &self.source
    }

    /// Caller-stated branch identity and checkpoint.
    #[must_use]
    pub const fn branch(&self) -> BranchContext {
        self.branch
    }

    /// SHA-256 identity of the exact compared or refused bytes.
    #[must_use]
    pub const fn byte_identity(&self) -> &[u8; 32] {
        &self.byte_identity
    }

    /// The host observation, carrying no row standing.
    #[must_use]
    pub const fn observation(&self) -> &MaturityContinuityObservation {
        &self.observation
    }
}

/// The mutation registry whose branch-indexed material is outstanding.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityMutationRegistryKind {
    /// The canonical root-history mutation registry.
    RootHistory,
}

/// Why the history registry carries no mutations.
///
/// A typed reason rather than an absence, because an absent registry makes
/// the plan's input list shorter and its completeness claim stronger,
/// which is the wrong direction for a plan whose purpose is to bound what
/// has been established.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum MaturityRegistryOutstandingReason {
    /// No typed branch context can be projected, rewound and reprojected,
    /// so a root-history mutation has nothing to be relative to.
    ///
    /// A root-history mutation is a change to an edge sequence read
    /// against a branch. This module states a branch binding a view
    /// validates and can observe no branch at all, and an entry written
    /// against a context nobody can project would be an entry against an
    /// assumption.
    BranchIndexedHistoryIsNotProjectable,
}

impl MaturityRegistryOutstandingReason {
    /// The registry this reason belongs to.
    #[must_use]
    pub const fn kind(self) -> MaturityMutationRegistryKind {
        match self {
            Self::BranchIndexedHistoryIsNotProjectable => MaturityMutationRegistryKind::RootHistory,
        }
    }
}

/// The standing a registry census counts its mutations under.
///
/// One member, read and never written, on the precedent of the candidate
/// ABI's own single-variant status: a count with no standing beside it
/// would read as a registry that has been surveyed and found empty, which
/// is the opposite of what this registry records.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityRegistryStanding {
    /// The registry is outstanding until the material it indexes exists.
    OutstandingUntilItsMaterialLands,
}

/// One mutation a landed registry would carry.
///
/// The row it stages and where its change sits, both in the matrix's own
/// vocabulary rather than in one invented here: a registry entry naming a
/// locator this workspace does not otherwise use would be a second
/// vocabulary for the same facts, and the two would disagree the first
/// time a row moved. There is no public constructor and this module builds
/// none, which is what makes the emptiness of the history registry a property
/// of the type rather than a claim about a list.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MaturityRegisteredMutation {
    section: MaturitySafetySection,
    row: &'static str,
    locator: MaturityMutationLocator,
}

impl MaturityRegisteredMutation {
    /// The §16 table the staged row comes from.
    #[must_use]
    pub const fn section(&self) -> MaturitySafetySection {
        self.section
    }

    /// The staged row's stable name.
    #[must_use]
    pub const fn row(&self) -> &'static str {
        self.row
    }

    /// Where the mutation sits.
    #[must_use]
    pub const fn locator(&self) -> MaturityMutationLocator {
        self.locator
    }
}

/// What one registry of §14.1 contains, as a figure and a standing.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MaturityMutationRegistryCensus {
    mutations: usize,
    standing: MaturityRegistryStanding,
}

impl MaturityMutationRegistryCensus {
    /// How many mutations the registry carries.
    #[must_use]
    pub const fn mutations(&self) -> usize {
        self.mutations
    }

    /// The standing that count is read under.
    #[must_use]
    pub const fn standing(&self) -> MaturityRegistryStanding {
        self.standing
    }
}

/// The root-history registry whose branch-indexed material is outstanding.
///
/// Structurally present and explicitly outstanding: it names which
/// registry it is, why it carries nothing, and counts its mutations under
/// a standing. The mutation list retains the matrix's own identity vocabulary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityOutstandingMutationRegistry {
    kind: MaturityMutationRegistryKind,
    reason: MaturityRegistryOutstandingReason,
    mutations: Vec<MaturityRegisteredMutation>,
}

impl MaturityOutstandingMutationRegistry {
    /// The outstanding registry one reason describes.
    const fn outstanding(reason: MaturityRegistryOutstandingReason) -> Self {
        Self {
            kind: reason.kind(),
            reason,
            mutations: Vec::new(),
        }
    }

    /// Which registry this is.
    #[must_use]
    pub const fn kind(&self) -> MaturityMutationRegistryKind {
        self.kind
    }

    /// Why it carries no mutations.
    #[must_use]
    pub const fn reason(&self) -> MaturityRegistryOutstandingReason {
        self.reason
    }

    /// Every mutation it carries.
    #[must_use]
    pub fn mutations(&self) -> &[MaturityRegisteredMutation] {
        &self.mutations
    }

    /// The count and the standing it is read under.
    #[must_use]
    pub const fn census(&self) -> MaturityMutationRegistryCensus {
        MaturityMutationRegistryCensus {
            mutations: self.mutations.len(),
            standing: MaturityRegistryStanding::OutstandingUntilItsMaterialLands,
        }
    }
}

/// What the operator declares the executor was built from, or that they
/// declared nothing.
///
/// Half an expectation is none: comparing an intended tip while the
/// upstream base goes unstated establishes less than the pair does and
/// would read as though it had established the pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaturityExecutorProvenanceExpectation {
    /// The operator stated both revisions, and this is the expectation a
    /// reported provenance is compared against.
    Stated(ExpectedExecutorProvenance),
    /// The operator stated no expectation, so no comparison is owed and
    /// none is claimed.
    NotStatedByTheOperator,
}

impl MaturityExecutorProvenanceExpectation {
    /// The stated expectation, where there is one.
    #[must_use]
    pub const fn stated(&self) -> Option<&ExpectedExecutorProvenance> {
        match self {
            Self::Stated(expectation) => Some(expectation),
            Self::NotStatedByTheOperator => None,
        }
    }
}

/// The exact target and deployment one plan is bound to.
///
/// The reviewed contract, the deployment the bundle was linked for, and
/// that deployment's own link parameters, kept together because they are
/// one input: a target without the deployment it was linked under names no
/// particular candidate, and a deployment without its target names no
/// particular contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityTargetBinding {
    target: ReviewedElementsTapscriptDefinition,
    deployment: MaturityDeployment,
    parameters: StateLinkDeploymentParameters,
}

impl MaturityTargetBinding {
    /// The reviewed contract.
    #[must_use]
    pub const fn target(&self) -> &ReviewedElementsTapscriptDefinition {
        &self.target
    }

    /// The deployment the bundle was linked for.
    #[must_use]
    pub const fn deployment(&self) -> MaturityDeployment {
        self.deployment
    }

    /// The link parameters that deployment fixes.
    #[must_use]
    pub const fn parameters(&self) -> &StateLinkDeploymentParameters {
        &self.parameters
    }
}

/// What was submitted, in the form the protocol could carry it.
///
/// §14.2 asks a native refusal to store the submitted subject's identity
/// *or* its exact bytes, and the two are kept apart because they are
/// different evidence: an identity is checkable against a chain and exists
/// only where something was accepted, and bytes are checkable against a
/// construction and exist whether or not anything was.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaturitySubmittedSubject {
    /// The identity the target computed for the submitted transaction.
    Identity(Txid),
    /// The exact submitted bytes.
    ExactBytes(Vec<u8>),
}

/// Where the refused change sat, as the run could name it.
///
/// A row states its locator before anything runs; this is what the
/// submitted subject actually changed, which is the same fact observed
/// rather than declared. A structural separator is the honest second form:
/// where a change sits between fields rather than in one, a locator would
/// have to name a field that was not the subject.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaturityMutationSite {
    /// The change sat at a locator the matrix names.
    Locator(MaturityMutationLocator),
    /// The change sat at a structural separator, named as the run named
    /// it.
    StructuralSeparator(String),
}

/// Whether the carrier the row intended actually ran.
///
/// A row may intend the announcement leaf and be refused by the ABI before
/// the leaf runs at all, and a refusal that never reached the carrier
/// establishes nothing about it. A boolean would leave that distinction to
/// whoever read the field next.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityCarrierExecution {
    /// The intended carrier executed.
    Executed,
    /// The intended carrier was never reached.
    NotReached,
}

/// Which carrier a submission aimed at, and whether it ran.
///
/// One value rather than two, because the two facts are only meaningful
/// together: a record saying that a carrier ran without saying which one
/// states nothing a reader can check, and a carrier named without the
/// fact of its execution repeats what the row already declared.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MaturityCarrierOutcome {
    intended: MaturityIntendedCarrier,
    execution: MaturityCarrierExecution,
}

impl MaturityCarrierOutcome {
    /// The carrier a submission aimed at, and what became of it.
    #[must_use]
    pub const fn new(
        intended: MaturityIntendedCarrier,
        execution: MaturityCarrierExecution,
    ) -> Self {
        Self {
            intended,
            execution,
        }
    }

    /// The carrier the submission intended to exercise.
    #[must_use]
    pub const fn intended(self) -> MaturityIntendedCarrier {
        self.intended
    }

    /// Whether that carrier ran.
    #[must_use]
    pub const fn execution(self) -> MaturityCarrierExecution {
        self.execution
    }
}

/// Which control a target accepted, and the identity it computed for it.
///
/// One value rather than two fields, on the same argument the carrier
/// outcome is one value: an identity on its own is thirty-two bytes
/// saying that something was accepted, and a shape on its own repeats
/// what the row already declared. The pair is what makes the condition
/// checkable — that the control a refusal departs from was accepted, and
/// that it was the row's control and not another shape that happened to
/// be accepted in the same run.
///
/// The shape is the expectation and the identity is the observation, and
/// both travel on the refusal for the reason the declared boundary
/// travels beside the observed layer: a comparison whose operands are
/// not both carried cannot be rechecked by a reader who has only the
/// record.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MaturityAcceptedControl {
    shape: MaturityCanonicalControl,
    identity: Txid,
}

impl MaturityAcceptedControl {
    /// The control shape a target accepted, with the identity it
    /// computed for it.
    #[must_use]
    pub const fn new(shape: MaturityCanonicalControl, identity: Txid) -> Self {
        Self { shape, identity }
    }

    /// Which canonical control was accepted.
    #[must_use]
    pub const fn shape(self) -> MaturityCanonicalControl {
        self.shape
    }

    /// The identity the target computed for it.
    #[must_use]
    pub const fn identity(self) -> Txid {
        self.identity
    }
}

/// The eight facts §14.2 requires every native refusal to store, and the
/// control's shape beside the control's identity.
///
/// One struct rather than eight fields repeated on two members, because
/// "every native refusal stores" is a statement about all native refusals:
/// a refusal at the declared boundary and a refusal at another one are the
/// same observation differently classified, and storing different facts
/// about them would make the classification unfalsifiable.
///
/// Four of the eight are the ones a run computes and the live generation
/// already carries where it computed them — the declared boundary, the
/// observed layer, the accepted control's identity and the target's own
/// words. The other four are §14.2's remainder: what was submitted, where
/// the change sat, which carrier the submission intended, and whether that
/// carrier ran. The intended carrier and the mutation site also stand on
/// the row, as the design facts they are, and carrying both is the point:
/// a row declares them before anything executes and a run reports what it
/// actually reached, exactly as the declared boundary and the observed
/// layer are both carried rather than compared into a boolean.
///
/// The control's shape stands beside its identity on that same argument,
/// in [`MaturityAcceptedControl`], and not as a ninth field: the identity
/// records that something was accepted and the shape records what it was,
/// and only the pair can answer whether the control a refusal departs
/// from was the row's. Pairing the two facts rather than adding a
/// parameter is also how the record keeps one argument per fact a caller
/// can get wrong.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityNativeRefusal {
    declared_boundary: EvidenceBoundary,
    observed_layer: ObservedOutcomeLayer,
    control: MaturityAcceptedControl,
    refusal_detail: String,
    submitted: MaturitySubmittedSubject,
    site: MaturityMutationSite,
    carrier: MaturityCarrierOutcome,
}

impl MaturityNativeRefusal {
    /// Record one native refusal with all eight facts.
    ///
    /// Recording is not classifying: which standing this refusal produces
    /// is decided by [`MaturityRowStanding::from_native_refusal`], against
    /// the one mapping from observed layers to declared boundaries, and
    /// never here.
    #[must_use]
    pub const fn record(
        declared_boundary: EvidenceBoundary,
        observed_layer: ObservedOutcomeLayer,
        control: MaturityAcceptedControl,
        refusal_detail: String,
        submitted: MaturitySubmittedSubject,
        site: MaturityMutationSite,
        carrier: MaturityCarrierOutcome,
    ) -> Self {
        Self {
            declared_boundary,
            observed_layer,
            control,
            refusal_detail,
            submitted,
            site,
            carrier,
        }
    }

    /// The boundary the row declared in advance.
    #[must_use]
    pub const fn declared_boundary(&self) -> EvidenceBoundary {
        self.declared_boundary
    }

    /// The layer the target actually refused at.
    #[must_use]
    pub const fn observed_layer(&self) -> ObservedOutcomeLayer {
        self.observed_layer
    }

    /// The control this refusal's mutant departs from, with its identity.
    #[must_use]
    pub const fn accepted_control(&self) -> MaturityAcceptedControl {
        self.control
    }

    /// Which canonical control was accepted.
    #[must_use]
    pub const fn control(&self) -> MaturityCanonicalControl {
        self.control.shape()
    }

    /// The identity the target computed for the accepted control.
    #[must_use]
    pub const fn control_identity(&self) -> Txid {
        self.control.identity()
    }

    /// What the target said, in the target's own words.
    #[must_use]
    pub fn refusal_detail(&self) -> &str {
        &self.refusal_detail
    }

    /// What was submitted.
    #[must_use]
    pub const fn submitted(&self) -> &MaturitySubmittedSubject {
        &self.submitted
    }

    /// Where the change sat.
    #[must_use]
    pub const fn site(&self) -> &MaturityMutationSite {
        &self.site
    }

    /// The carrier the submission intended to exercise, and whether it
    /// ran.
    #[must_use]
    pub const fn carrier(&self) -> MaturityCarrierOutcome {
        self.carrier
    }

    /// The carrier the submission intended to exercise.
    #[must_use]
    pub const fn intended_carrier(&self) -> MaturityIntendedCarrier {
        self.carrier.intended()
    }

    /// Whether that carrier ran.
    #[must_use]
    pub const fn carrier_execution(&self) -> MaturityCarrierExecution {
        self.carrier.execution()
    }

    /// Whether the observed layer is exactly the declared boundary.
    #[must_use]
    pub fn is_at_the_declared_boundary(&self) -> bool {
        matches_boundary(self.declared_boundary, self.observed_layer)
    }
}

/// Whether a native refusal is bound to one row, and which condition
/// failed where it is not.
///
/// A boolean would say that a refusal does not answer a row without
/// saying what about it did not, and the difference matters to whoever
/// reads the outcome: a refusal at another layer is a row still waiting,
/// a refusal of another subject is a mutant built wrong, and a refusal
/// whose carrier never ran is a run that stopped earlier than the row is
/// about. Those are three different pieces of work.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaturityRowBinding {
    /// Every condition this crate can decide holds of the pair.
    Bound,
    /// The row is answered by no refusal at all.
    ///
    /// Either no layer answers it, so there is no boundary for an
    /// observation to equal, or the row is its own control, and a
    /// control is what a mutant departs from rather than something a
    /// refusal is offered against.
    RowAdmitsNoRefusal,
    /// The refusal was observed somewhere other than the row's declared
    /// boundary, or it was recorded against another boundary entirely.
    ObservedElsewhere,
    /// The change the run named is not the row's subject.
    AnotherSubject,
    /// The carrier differed from the row's, or was never reached.
    CarrierDeparted,
    /// The control accepted was not of the row's own shape.
    AnotherControl,
}

/// Whether one native refusal answers one row, on the conditions a
/// boundary equality is decidable by.
///
/// §1.15 answers a negative row by a target refusal on five conditions:
/// the control was accepted, the mutation was attributable, the intended
/// carrier was reached, the observed layer equals the row's declared
/// boundary, and the result is bound to the exact submitted bytes. Three
/// of them are decided here, from facts the row declares before anything
/// runs against facts the refusal stores: the boundary equality through
/// the crate's own `matches_boundary`, read and never restated;
/// attributability, as the subject the run named against the subject the
/// row's locator names; and the carrier, which must be the row's and must
/// have executed, because a refusal that never reached the carrier
/// establishes nothing about it.
///
/// The other two are the run's and no in-crate reader can settle them.
/// What is decidable about the control is that the shape accepted is the
/// row's; that the identity carried is of a transaction a chain accepted
/// is checkable only against that chain, which is why the identity is
/// carried rather than reduced to a boolean. The byte binding is not a
/// condition a verdict member can fail either: the submitted subject is a
/// field of the record and exists in one of the two forms §14.2 admits,
/// so its shape holds by construction, and whether those bytes are the
/// mutant's is a comparison against the construction that produced them.
/// A refusal this function calls bound is therefore bound on the three
/// conditions the row's own facts settle, and no further.
///
/// A run that names a structural separator rather than a locator answers
/// exactly the rows whose subject is the metadata encoding. That subject
/// is the one whose documented places — the domain separator, the field
/// order, the reserved fields, the trailing bytes — sit between fields
/// rather than in one, so a run reporting a separator there has reported
/// the row's own subject honestly and a rule that refused it would call a
/// correct observation unbound. A separator against any other subject is
/// a departure. The mirrored clause, a row with no locator answered by a
/// separator, is stated for the rule's sake: no negative row of the
/// matrix carries an absent locator at this tip, so that clause admits
/// nothing today and is the rule a future such row would be handled by
/// rather than a fallthrough.
#[must_use]
pub fn native_refusal_binds_to_row(
    row: &MaturitySafetyRow,
    refusal: &MaturityNativeRefusal,
) -> MaturityRowBinding {
    let Some(boundary) = row.refusing_layer() else {
        return MaturityRowBinding::RowAdmitsNoRefusal;
    };
    if row.control() == MaturityCanonicalControl::TheRowIsTheControl {
        return MaturityRowBinding::RowAdmitsNoRefusal;
    }
    if refusal.control() != row.control() {
        return MaturityRowBinding::AnotherControl;
    }
    if refusal.declared_boundary() != boundary || !refusal.is_at_the_declared_boundary() {
        return MaturityRowBinding::ObservedElsewhere;
    }
    if !site_names_the_rows_subject(row, refusal.site()) {
        return MaturityRowBinding::AnotherSubject;
    }
    if refusal.intended_carrier() != row.carrier()
        || refusal.carrier_execution() != MaturityCarrierExecution::Executed
    {
        return MaturityRowBinding::CarrierDeparted;
    }
    MaturityRowBinding::Bound
}

/// Whether the site a run named is the subject the row changed.
///
/// Compared as subjects rather than as locators, because that is the
/// claim: two names for one object answer the same row, and one name for
/// two objects would answer neither. The map from locators to subjects is
/// injective over the matrix, so this is exactly locator equality today
/// and stays the right comparison if a later locator names a place inside
/// a subject that already has one.
fn site_names_the_rows_subject(row: &MaturitySafetyRow, site: &MaturityMutationSite) -> bool {
    match (row.locator(), site) {
        (Some(locator), MaturityMutationSite::Locator(named)) => {
            locator.subject() == named.subject()
        }
        (Some(locator), MaturityMutationSite::StructuralSeparator(_)) => {
            locator.subject() == MaturityMutationSubject::MetadataEncoding
        }
        (None, MaturityMutationSite::StructuralSeparator(_)) => true,
        (None, MaturityMutationSite::Locator(_)) => false,
    }
}

/// What answers one §16 row, or what stands in the way.
///
/// Exactly one standing per row, and none of them is "passed". §14.2's
/// eleven classes are here, and three more stand beside them for the rows
/// that are waiting: §14.2 names what a run's answer is called and not
/// what a row waiting for one is called, and its own sentence that an
/// answerable capability is not an answered standing needs somewhere to
/// put such a row. Filing a waiting row under an observed member would say
/// a verdict was reached, and leaving it unclassified would make the
/// completeness question unanswerable rather than unanswered.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MaturityRowStanding {
    /// A first-party validator was driven to the row's own refusal.
    ///
    /// Recomputed by
    /// [`crate::maturity_first_party::validate_maturity_first_party`]
    /// rather than asserted here: the entry point ran twice, accepted the
    /// honest input and refused the changed one naming the class below.
    FirstPartyDischarged {
        /// The entry point that refused.
        validator: MaturityFirstPartyValidator,
        /// The refusal class it named.
        class: &'static str,
    },
    /// The row's boundary precedes the target and nothing stages it yet.
    ///
    /// Carries the first-party census's own typed reason, so that a row
    /// whose owning entry point was located and whose discharge waits on
    /// substrate is legibly different from a row whose stated change no
    /// public entry point accepts.
    FirstPartyRequired(MaturityCarriedReason),
    /// A target accepted a candidate of the row's own shape.
    ///
    /// The identity is carried rather than a boolean, so the claim can be
    /// checked against a chain by somebody who does not trust this crate.
    /// No row stands here at this tip.
    NativeAcceptanceObserved {
        /// The identity the target computed for the accepted candidate.
        identity: Txid,
    },
    /// A target refused this row's mutant at exactly the boundary the row
    /// declared.
    ///
    /// Minted only through [`Self::from_native_refusal`], which compares
    /// the observed layer against the declared boundary through the one
    /// mapping both consumers of that rule share.
    NativeRefusalObserved(MaturityNativeRefusal),
    /// A committed run reached a positive row's declared boundary.
    ///
    /// A positive row departs from no control, so a mutant's refusal cannot answer it. Exact replay of the admitted run establishes this observation; reaching the relay boundary leaves the accepted positive control outstanding.
    NativeDeclaredBoundaryObserved {
        /// The pinned SHA-256 content address of the run report.
        run_address: &'static str,
        /// The recorded refusal detail cross-checked against the response.
        recorded_detail: &'static str,
    },
    /// A target refused this row's mutant at a boundary that is not the
    /// one the row declared.
    ///
    /// A recorded observation and not an answer: the row asked whether a
    /// particular layer refuses a particular fault, and the target refused
    /// earlier, so that layer was never reached.
    /// [`Self::is_answered`] is false for it and it counts in its own
    /// bucket.
    NativeRefusalAtUnexpectedBoundary(MaturityNativeRefusal),
    /// The row's boundary is the target, and a run would answer it.
    ///
    /// Carries the published requirement the row resolves to where the
    /// plan publishes one, so a run's observation can be filed against a
    /// relation rather than against a name.
    NativeRunRequired(Option<CoverageRequirementId>),
    /// The constructor-continuity report observed the row.
    ///
    /// The class exists because §14.2 names it; the evidence belongs to
    /// the report that establishes it, and a field invented here would fix
    /// the shape of evidence nobody has produced. No row stands here at
    /// this tip.
    ConstructorContinuityObserved,
    /// The root-history report observed the row.
    ///
    /// The class exists because §14.2 names it, on the same argument as
    /// the member above. No row stands here at this tip.
    RootHistoryObserved,
    /// The public-recovery report observed the row.
    ///
    /// The class exists because §14.2 names it, on the same argument. No
    /// row stands here at this tip.
    PublicRecoveryObserved,
    /// Report validation observed the row's projection in the rendered
    /// bytes.
    ///
    /// The answered counterpart of [`Self::ReportLayerRequired`]. No row
    /// stands here at this tip.
    ReportLayerObserved,
    /// The row's boundary is this workspace's own rendered report, and no
    /// validation has read it yet.
    ///
    /// Carries what the row expects the projection comparison to say, so
    /// the outstanding obligation is the row's own requirement rather than
    /// a note that something is missing.
    ReportLayerRequired(MaturityExpectedProjection),
    /// The row carries a typed reason no layer answers it.
    ///
    /// The reason travels as the matrix's own value rather than as a copy
    /// of its vocabulary here, because the matrix is where a reason that
    /// stopped applying has to fail. Minted only where the row's boundary
    /// states such a reason. [`Self::is_answered`] is false for it,
    /// including where the reason names evidence that landed in another
    /// census: what that reason states is where to look, and this plan did
    /// not look.
    OutstandingUnderTypedNonAnswer(MaturityRowBoundary),
    /// A component the row needs does not exist.
    ///
    /// Not answered: a failed environment is never evidence that a change
    /// was refused. The class exists because §14.2 names it, and no
    /// component this plan needs is missing at this tip.
    InfrastructureBlocked,
    /// The row is ad hoc and outside the required denominator.
    ///
    /// §14.2's last class. No row of §16 is experimental, and the member
    /// exists because a plan that could not express the class would have
    /// to file an ad hoc case as required.
    Experimental,
}

impl MaturityRowStanding {
    /// Classify one native refusal into the standing it earns.
    ///
    /// The only route to either refusal member. A refusal answers a row
    /// only if it happened where the row said it would, so the comparison
    /// is made here, once, through
    /// the crate's own `matches_boundary` — the same mapping
    /// the live generation's classifier uses, read rather than restated,
    /// because one rule with two implementations is how wrong-boundary
    /// rows came to stand as answered before.
    #[must_use]
    pub fn from_native_refusal(refusal: MaturityNativeRefusal) -> Self {
        if refusal.is_at_the_declared_boundary() {
            Self::NativeRefusalObserved(refusal)
        } else {
            Self::NativeRefusalAtUnexpectedBoundary(refusal)
        }
    }

    /// Whether this standing is evidence rather than an outstanding
    /// obligation.
    ///
    /// True for the standings this plan recomputed or observed, and for no
    /// others. The three required members and the outstanding one are
    /// rows waiting; the unexpected-boundary member is a row contradicted,
    /// which is a different thing from a row waiting and must not render
    /// alike; the blocked member is a row whose environment failed; and an
    /// experimental row is outside the denominator altogether.
    #[must_use]
    pub const fn is_answered(&self) -> bool {
        matches!(
            self,
            Self::FirstPartyDischarged { .. }
                | Self::NativeAcceptanceObserved { .. }
                | Self::NativeRefusalObserved(_)
                | Self::NativeDeclaredBoundaryObserved { .. }
                | Self::ConstructorContinuityObserved
                | Self::RootHistoryObserved
                | Self::PublicRecoveryObserved
                | Self::ReportLayerObserved
        )
    }
}

/// One classified row of the §16 matrix.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityEvidenceRow {
    row: &'static MaturitySafetyRow,
    standing: MaturityRowStanding,
}

impl MaturityEvidenceRow {
    /// The §16 row this classification is about.
    #[must_use]
    pub const fn row(&self) -> &'static MaturitySafetyRow {
        self.row
    }

    /// What answers it, or what stands in the way.
    #[must_use]
    pub const fn standing(&self) -> &MaturityRowStanding {
        &self.standing
    }

    /// Count records associated with this row's section and exact name.
    #[must_use]
    pub fn continuity_record_count(&self, records: &[MaturityContinuityRecord]) -> usize {
        records
            .iter()
            .filter(|record| {
                (record.section(), record.row()) == (self.row.section(), self.row.name())
            })
            .count()
    }
}

/// One bucket per standing, and the denominator they partition.
///
/// Every figure here is counted from the classified rows rather than
/// written beside them, and the buckets are exhaustive over the standing
/// vocabulary: a member added without a bucket fails to compile rather
/// than quietly leaving rows uncounted.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MaturityEvidenceCensus {
    rows: usize,
    first_party_discharged: usize,
    first_party_required: usize,
    native_acceptance_observed: usize,
    native_refusal_observed: usize,
    native_declared_boundary_observed: usize,
    native_refusal_at_unexpected_boundary: usize,
    native_run_required: usize,
    constructor_continuity_observed: usize,
    root_history_observed: usize,
    public_recovery_observed: usize,
    report_layer_observed: usize,
    report_layer_required: usize,
    outstanding_under_typed_non_answer: usize,
    infrastructure_blocked: usize,
    experimental: usize,
}

impl MaturityEvidenceCensus {
    /// How many rows the matrix has.
    #[must_use]
    pub const fn rows(&self) -> usize {
        self.rows
    }

    /// How many rows a first-party validator answered.
    #[must_use]
    pub const fn first_party_discharged(&self) -> usize {
        self.first_party_discharged
    }

    /// How many pre-target rows nothing stages yet.
    #[must_use]
    pub const fn first_party_required(&self) -> usize {
        self.first_party_required
    }

    /// How many rows a target acceptance answered.
    #[must_use]
    pub const fn native_acceptance_observed(&self) -> usize {
        self.native_acceptance_observed
    }

    /// How many rows a target refusal answered at their declared
    /// boundary.
    #[must_use]
    pub const fn native_refusal_observed(&self) -> usize {
        self.native_refusal_observed
    }

    /// How many positive rows an admitted run answered at their declared boundary.
    #[must_use]
    pub const fn native_declared_boundary_observed(&self) -> usize {
        self.native_declared_boundary_observed
    }

    /// How many rows were refused somewhere other than where they said
    /// they would be.
    #[must_use]
    pub const fn native_refusal_at_unexpected_boundary(&self) -> usize {
        self.native_refusal_at_unexpected_boundary
    }

    /// How many rows are waiting on a target run.
    #[must_use]
    pub const fn native_run_required(&self) -> usize {
        self.native_run_required
    }

    /// How many rows the constructor-continuity report answered.
    #[must_use]
    pub const fn constructor_continuity_observed(&self) -> usize {
        self.constructor_continuity_observed
    }

    /// How many rows the root-history report answered.
    #[must_use]
    pub const fn root_history_observed(&self) -> usize {
        self.root_history_observed
    }

    /// How many rows the public-recovery report answered.
    #[must_use]
    pub const fn public_recovery_observed(&self) -> usize {
        self.public_recovery_observed
    }

    /// How many rows report validation answered.
    #[must_use]
    pub const fn report_layer_observed(&self) -> usize {
        self.report_layer_observed
    }

    /// How many rows are waiting on report validation.
    #[must_use]
    pub const fn report_layer_required(&self) -> usize {
        self.report_layer_required
    }

    /// How many rows carry a typed reason no layer answers them.
    #[must_use]
    pub const fn outstanding_under_typed_non_answer(&self) -> usize {
        self.outstanding_under_typed_non_answer
    }

    /// How many rows are blocked on a missing component.
    #[must_use]
    pub const fn infrastructure_blocked(&self) -> usize {
        self.infrastructure_blocked
    }

    /// How many rows are outside the required denominator.
    #[must_use]
    pub const fn experimental(&self) -> usize {
        self.experimental
    }

    /// How many rows this plan recomputed or observed an answer for.
    #[must_use]
    pub const fn answered(&self) -> usize {
        self.first_party_discharged
            + self.native_acceptance_observed
            + self.native_refusal_observed
            + self.native_declared_boundary_observed
            + self.constructor_continuity_observed
            + self.root_history_observed
            + self.public_recovery_observed
            + self.report_layer_observed
    }

    /// How many rows stand outstanding.
    ///
    /// Summed from the outstanding buckets rather than subtracted from the
    /// denominator, so that the two halves of the partition are two
    /// computations a test can hold against each other.
    #[must_use]
    pub const fn outstanding(&self) -> usize {
        self.first_party_required
            + self.native_run_required
            + self.report_layer_required
            + self.outstanding_under_typed_non_answer
            + self.native_refusal_at_unexpected_boundary
            + self.infrastructure_blocked
            + self.experimental
    }

    /// Whether every required row is answered.
    ///
    /// False while any row is waiting on a validator, a run, report
    /// validation or another layer's evidence, while any row was refused
    /// somewhere other than where it declared, and while any row is
    /// blocked on a missing component. The typed non-answers are among
    /// those conditions on this plan's own discipline: its answered
    /// standings are the ones it recomputed or observed, and a bar that
    /// took another census's word for a row could be met without this plan
    /// having established anything.
    #[must_use]
    pub const fn every_required_row_is_answered(&self) -> bool {
        self.first_party_required == 0
            && self.native_run_required == 0
            && self.report_layer_required == 0
            && self.outstanding_under_typed_non_answer == 0
            && self.native_refusal_at_unexpected_boundary == 0
            && self.infrastructure_blocked == 0
    }

    /// The quantifier every figure above is stated under.
    #[must_use]
    pub const fn quantifier(&self) -> MaturityGuaranteeQuantifier {
        MaturityGuaranteeQuantifier::of_this_plan()
    }
}

/// The canonical maturity-announcement evidence plan of §14.1.
///
/// Every field is private and there is no public constructor, no
/// `Default` and no builder: the sole route to a value is
/// [`derive_maturity_evidence_plan_with`], which builds the inputs and
/// derives the classification. A plan assembled from arbitrary fields
/// would be a caller's opinion about what has been established, which is
/// exactly what §14.1's private fields exist to prevent.
///
/// Its guarantees are quantified over the first two observation classes
/// and are conditional on no observation of the third;
/// [`MaturityEvidenceCensus::quantifier`] renders that condition as a
/// value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaturityAnnouncementEvidencePlan {
    plan: ValidatedMaturityAnnouncementOperationPlan,
    bundle: CandidateLinkedMaturityBundle,
    abi: CandidateMaturityAnnouncementAbi,
    fixtures: Vec<CanonicalSubject<MaturitySemanticCase>>,
    material: MaturityConstructorMaterial,
    records: Vec<MaturityContinuityRecord>,
    root_history_mutations: MaturityOutstandingMutationRegistry,
    binding: MaturityTargetBinding,
    provenance: MaturityExecutorProvenanceExpectation,
    rows: Vec<MaturityEvidenceRow>,
    discharged: Vec<ValidatedMaturityFirstPartyEvidence>,
    census: MaturityEvidenceCensus,
}

impl MaturityAnnouncementEvidencePlan {
    /// The validated compiler operation plan every row is resolved
    /// against.
    #[must_use]
    pub const fn operation_plan(&self) -> &ValidatedMaturityAnnouncementOperationPlan {
        &self.plan
    }

    /// The exact linked bundle.
    #[must_use]
    pub const fn bundle(&self) -> &CandidateLinkedMaturityBundle {
        &self.bundle
    }

    /// The exact candidate ABI, derived over the validated view this plan
    /// built.
    #[must_use]
    pub const fn abi(&self) -> &CandidateMaturityAnnouncementAbi {
        &self.abi
    }

    /// The canonical semantic fixture registry, admitted after the census
    /// that validates it.
    #[must_use]
    pub fn semantic_fixtures(&self) -> &[CanonicalSubject<MaturitySemanticCase>] {
        &self.fixtures
    }

    /// The supplied constructor material or its explicit absence.
    #[must_use]
    pub const fn constructor_material(&self) -> &MaturityConstructorMaterial {
        &self.material
    }

    /// Validated entry records followed by mutant records, in input order.
    #[must_use]
    pub fn continuity_records(&self) -> &[MaturityContinuityRecord] {
        &self.records
    }

    /// Count this plan's records associated with an exact matrix row.
    #[must_use]
    pub fn continuity_record_count(&self, row: &MaturityEvidenceRow) -> usize {
        row.continuity_record_count(self.continuity_records())
    }

    /// The canonical root-history mutation registry.
    #[must_use]
    pub const fn root_history_mutations(&self) -> &MaturityOutstandingMutationRegistry {
        &self.root_history_mutations
    }

    /// The exact target and deployment binding.
    #[must_use]
    pub const fn binding(&self) -> &MaturityTargetBinding {
        &self.binding
    }

    /// The expected executor provenance, as the operator stated it or did
    /// not.
    #[must_use]
    pub const fn executor_provenance(&self) -> &MaturityExecutorProvenanceExpectation {
        &self.provenance
    }

    /// Every classified row, in the matrix's order.
    #[must_use]
    pub fn rows(&self) -> &[MaturityEvidenceRow] {
        &self.rows
    }

    /// The first-party evidence this plan recomputed.
    #[must_use]
    pub fn discharged(&self) -> &[ValidatedMaturityFirstPartyEvidence] {
        &self.discharged
    }

    /// The census figures.
    #[must_use]
    pub const fn census(&self) -> MaturityEvidenceCensus {
        self.census
    }
}

/// Why the evidence plan could not be derived.
///
/// One member per input that can refuse, plus the two disagreements a
/// classification can meet. The cause each input refused with is carried
/// rather than flattened: the layers that own these inputs answer in four
/// different vocabularies, and a single name for all of them would tell a
/// reader that something failed without telling them what.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MaturityEvidenceRefusal {
    /// The committed maturity run failed its byte, grammar, binding or replay admission.
    NativeCorpusImportRefused(MaturityCorpusImportRefusal),
    /// The validated announcement plan could not be derived.
    OperationPlanUnavailable(MaturityClosureRefusal),
    /// The exact linked bundle could not be linked.
    LinkedBundleUnavailable(MaturityClosureRefusal),
    /// The reviewed target or the deployment's parameters were
    /// unavailable.
    TargetBindingUnavailable(MaturityClosureRefusal),
    /// The linked bundle retained no constructor instance, so there is no
    /// predecessor world to state a view over.
    LinkedBundleCarriesNoRetainedInstance,
    /// The caller-stated root binding was refused.
    CallerStatedRootBindingRefused(RightRefusal),
    /// The public view could not be stated or did not validate.
    ValidatedViewRefused(TransactionRefusal),
    /// The candidate ABI could not be derived over the validated view.
    CandidateAbiRefused(TransactionRefusal),
    /// The canonical semantic fixture registry refused.
    SemanticFixtureRegistryRefused(VectorError),
    /// The first-party evidence could not be recomputed.
    FirstPartyEvidenceRefused(MaturityFirstPartyRefusal),
    /// A pre-target row has no entry in the first-party census, which is
    /// total over the pre-target rows, so the two have drifted apart.
    PreTargetRowCarriesNoFirstPartyCase {
        /// The §16 table the row comes from.
        section: MaturitySafetySection,
        /// The row's stable name.
        name: &'static str,
    },
    /// A row declares the executor-infrastructure boundary, which is never
    /// evidence that a change was refused and which no row of a safety
    /// matrix may name.
    RowDeclaresTheInfrastructureBoundary {
        /// The §16 table the row comes from.
        section: MaturitySafetySection,
        /// The row's stable name.
        name: &'static str,
    },
    /// A row's declared relation no longer resolves against the published
    /// plan.
    RowLinkUnresolved(VectorError),
    /// The operator stated an expectation the comparison cannot use.
    ExecutorProvenanceExpectationMalformed(ProvenanceSyntaxDefect),
}

/// The operator's stated expectation, read from their own environment.
///
/// Read here and never inside the derivation, so that the plan's inputs
/// are the arguments its signature names. The pair is required together:
/// an intended tip compared while the upstream base goes unstated
/// establishes less than the pair does and would read as though it had
/// established the pair, so a half statement is no statement.
///
/// A statement in a syntax the comparison cannot use is refused rather
/// than read as silence: the operator believed a comparison would be made,
/// and quietly making none is the outcome that must not be available.
///
/// # Errors
///
/// [`MaturityEvidenceRefusal::ExecutorProvenanceExpectationMalformed`]
/// when both revisions are stated and either is not in the admitted
/// syntax.
pub fn stated_executor_provenance()
-> Result<MaturityExecutorProvenanceExpectation, MaturityEvidenceRefusal> {
    let (Some(tip), Some(base)) = (environment(EXECUTOR_TIP), environment(UPSTREAM_BASE)) else {
        return Ok(MaturityExecutorProvenanceExpectation::NotStatedByTheOperator);
    };
    let topics = environment(LOCAL_TOPICS).unwrap_or_default();
    ExpectedExecutorProvenance::new(
        &tip,
        &base,
        topics.split(',').filter(|topic| !topic.trim().is_empty()),
    )
    .map(MaturityExecutorProvenanceExpectation::Stated)
    .map_err(MaturityEvidenceRefusal::ExecutorProvenanceExpectationMalformed)
}

/// One environment value, where it is set and carries something.
fn environment(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

/// Derive the canonical evidence plan of §14.1.
///
/// Read provenance through [`stated_executor_provenance`] and supply explicit
/// absent constructor material to [`derive_maturity_evidence_plan_with`].
/// Call that derivation directly to supply independent comparison material.
///
/// # Errors
///
/// Everything [`stated_executor_provenance`] and
/// [`derive_maturity_evidence_plan_with`] refuse.
#[must_use = "the derived plan carries the evidence partition"]
pub fn derive_maturity_evidence_plan()
-> Result<MaturityAnnouncementEvidencePlan, MaturityEvidenceRefusal> {
    derive_maturity_evidence_plan_with(
        stated_executor_provenance()?,
        MaturityConstructorMaterial::Absent(
            MaturityConstructorMaterialAbsence::NotSuppliedToDerivation,
        ),
    )
}

/// Derive the canonical evidence plan from the eight inputs of §14.1.
///
/// Six inputs are built here: the validated operation plan, the exact
/// linked bundle, the candidate ABI over the validated view this module
/// states, the canonical semantic fixture registry admitted after the
/// census that validates it, the outstanding root-history registry, and
/// the exact target and deployment binding. Provenance and constructor
/// material are independently supplied premises: manufacturing either
/// here would give a comparison both its operands. Every matrix row is
/// then classified into exactly one standing, and the census is counted
/// from the classified rows.
///
/// # Errors
///
/// [`MaturityEvidenceRefusal`], naming the input that refused and carrying
/// the refusal that layer raised, or the disagreement a classification met
/// between the matrix and the first-party census or the published plan.
#[must_use = "the derived plan carries the evidence partition and supplied material"]
pub fn derive_maturity_evidence_plan_with(
    provenance: MaturityExecutorProvenanceExpectation,
    material: MaturityConstructorMaterial,
) -> Result<MaturityAnnouncementEvidencePlan, MaturityEvidenceRefusal> {
    let plan = announcement_plan().map_err(MaturityEvidenceRefusal::OperationPlanUnavailable)?;
    let bundle = linked_maturity_bundle(
        DEPLOYMENT,
        crate::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
    )
    .map_err(MaturityEvidenceRefusal::LinkedBundleUnavailable)?;
    let target = closure_target().map_err(MaturityEvidenceRefusal::TargetBindingUnavailable)?;
    let view = validated_view(&target, &bundle)?;
    let abi = derive_maturity_announcement_abi(&target, &view)
        .map_err(MaturityEvidenceRefusal::CandidateAbiRefused)?;
    let fixtures = admitted_fixtures()?;
    let binding = MaturityTargetBinding {
        target,
        deployment: DEPLOYMENT,
        parameters: bundle.deployment().clone(),
    };

    let discharged = discharge_maturity_first_party()
        .map_err(MaturityEvidenceRefusal::FirstPartyEvidenceRefused)?;
    let corpus =
        maturity_run_of_record().map_err(MaturityEvidenceRefusal::NativeCorpusImportRefused)?;
    let index = FirstPartyIndex::of(&discharged);
    let mut classified = Vec::with_capacity(rows().len());
    for row in rows() {
        let standing = classify(row, &plan, &index)?;
        let standing = if row.section() == MaturitySafetySection::Positive
            && row.name() == "sponsorless"
            && row.refusing_layer() == Some(EvidenceBoundary::RelayPolicyRejection)
            && matches!(standing, MaturityRowStanding::NativeRunRequired(_))
            && corpus.evidence().standing() == MaturityNativeStanding::AnsweredAtDeclaredBoundary
        {
            MaturityRowStanding::NativeDeclaredBoundaryObserved {
                run_address: MATURITY_RUN_ADDRESS,
                recorded_detail: corpus.recorded_refusal_detail(),
            }
        } else {
            standing
        };
        classified.push(MaturityEvidenceRow { row, standing });
    }
    let census = census_from_rows(&classified);

    Ok(MaturityAnnouncementEvidencePlan {
        plan,
        bundle,
        abi,
        fixtures,
        records: constructor_records(&material),
        material,
        root_history_mutations: MaturityOutstandingMutationRegistry::outstanding(
            MaturityRegistryOutstandingReason::BranchIndexedHistoryIsNotProjectable,
        ),
        binding,
        provenance,
        rows: classified,
        discharged,
        census,
    })
}

/// Recompute the same individual comparisons carried by a continuity entry.
fn entry_observation(entry: &MaturityContinuityReportEntry) -> MaturityContinuityObservation {
    let static_facts = entry.static_facts();
    let semantic = entry.semantic_facts();
    let controls = entry.controls();
    let mut results = vec![
        static_facts.root_binding().agrees(),
        static_facts.control_binding().agrees(),
        static_facts.descriptor_result().is_ok(),
        static_facts.constructor_result().is_ok(),
        semantic.transition_agrees(),
    ];
    results.extend(semantic.reconstruction_agrees());
    results.extend(
        semantic
            .predecessor_fields()
            .iter()
            .map(MaturityFieldComparison::agrees),
    );
    if let Some(fields) = semantic.successor_fields() {
        results.extend(fields.iter().map(MaturityFieldComparison::agrees));
    }
    for tweak in [entry.predecessor_tweak(), entry.successor_tweak()] {
        results.extend([
            tweak.internal().agrees(),
            tweak.metadata().agrees(),
            tweak.branch().agrees(),
            tweak.digest().agrees(),
            tweak.key().agrees(),
            tweak.oddness().agrees(),
            tweak.program().agrees(),
        ]);
    }
    results.extend([controls.observed().agrees(), controls.inner().agrees()]);
    results.extend(controls.outer().iter().map(MaturityByteComparison::agrees));
    let (actual, predicted) = controls.first_byte_relation();
    results.push(actual == predicted);
    MaturityContinuityObservation::ValidatedComparison {
        comparisons: results.len(),
        agreements: results.iter().filter(|result| **result).count(),
    }
}

/// Associate host observations without entering the standing classifier.
fn constructor_records(material: &MaturityConstructorMaterial) -> Vec<MaturityContinuityRecord> {
    let MaturityConstructorMaterial::Present(present) = material else {
        return Vec::new();
    };
    let MaturityAcceptanceObligation::Outstanding { .. } = present.acceptance();
    let entries =
        present
            .report()
            .report()
            .entries()
            .iter()
            .map(|entry| MaturityContinuityRecord {
                section: MaturitySafetySection::Positive,
                row: "sponsorless",
                source: entry.source().clone(),
                branch: entry.branch(),
                byte_identity: *entry.byte_identity(),
                observation: entry_observation(entry),
            });
    let mutants = present.mutants().iter().map(|observed| {
        let mutant = observed.subject();
        let row = mutant.context().row();
        MaturityContinuityRecord {
            section: row.section(),
            row: row.name(),
            source: mutant.source().clone(),
            branch: mutant.branch(),
            byte_identity: *mutant.digest(),
            observation: MaturityContinuityObservation::RefusedMutant {
                refusal: mutant.failure().clone(),
            },
        }
    });
    entries.chain(mutants).collect()
}

/// The validated view this module's candidate ABI is derived over.
///
/// Built from the bundle's own retained world — the predecessor metadata,
/// its representation nonce and the program that link emitted — together
/// with the positions this module states as a caller: the outpoint the
/// current STATE output sits at and the branch the current root is bound
/// at. Nothing here reads a chain, and the view validates what is stated
/// rather than trusting it.
fn validated_view(
    target: &ReviewedElementsTapscriptDefinition,
    bundle: &CandidateLinkedMaturityBundle,
) -> Result<ValidatedMaturityStateView, MaturityEvidenceRefusal> {
    let retained = bundle
        .instances()
        .first()
        .ok_or(MaturityEvidenceRefusal::LinkedBundleCarriesNoRetainedInstance)?;
    let parameters = DEPLOYMENT
        .parameters()
        .map_err(MaturityEvidenceRefusal::TargetBindingUnavailable)?;
    let outpoint = Outpoint::new(Txid::from_internal(PREDECESSOR_TXID), 0)
        .map_err(MaturityEvidenceRefusal::ValidatedViewRefused)?;
    let branch = BranchContext::new(BRANCH, CHECKPOINT)
        .map_err(MaturityEvidenceRefusal::CallerStatedRootBindingRefused)?;
    let statements = vec![
        MaturityViewStatement::CurrentStateOutpoint(outpoint),
        MaturityViewStatement::AssetAndAmount(
            AssetField::Explicit(AssetId::from_internal(*parameters.singleton())),
            ValueField::Explicit(1),
        ),
        MaturityViewStatement::PredecessorMetadata(retained.metadata().semantic),
        MaturityViewStatement::PredecessorRepresentationNonce(retained.metadata().representation),
        MaturityViewStatement::CurrentRootBinding(branch),
        MaturityViewStatement::PredecessorProgram(retained.constructor().output_program()),
        MaturityViewStatement::AcceptedLinkedBundle(Box::new(bundle.clone())),
    ];
    PublicMaturityStateView::new(statements)
        .map_err(MaturityEvidenceRefusal::ValidatedViewRefused)?
        .validate(target, &OracleStateCurve)
        .map_err(MaturityEvidenceRefusal::ValidatedViewRefused)
}

/// The canonical semantic fixtures, admitted after the census validates
/// them.
///
/// The census is the registry, so the fixtures are read out of it rather
/// than asked for separately, and each is admitted only after the whole
/// census came back: admission is what the validation is for, and
/// admitting first would make the wrapper mean nothing.
fn admitted_fixtures()
-> Result<Vec<CanonicalSubject<MaturitySemanticCase>>, MaturityEvidenceRefusal> {
    let census = positive_semantic_census()
        .map_err(MaturityEvidenceRefusal::SemanticFixtureRegistryRefused)?;
    Ok(census
        .iter()
        .filter_map(|entry| entry.fixture().copied())
        .map(CanonicalSubject::admit)
        .collect())
}

/// One row's identity.
///
/// The table and the name together: three names repeat across the
/// matrix's tables, so a name alone is not a row identity.
type RowKey = (MaturitySafetySection, &'static str);

/// What the first-party census says about each pre-target row.
///
/// Both halves are built once, from the evidence this plan recomputed and
/// from the census that is total over the pre-target rows, so classifying
/// a row is two lookups rather than a rebuild of the census per row.
struct FirstPartyIndex {
    discharged: BTreeMap<RowKey, (MaturityFirstPartyValidator, &'static str)>,
    carried: BTreeMap<RowKey, MaturityCarriedReason>,
}

impl FirstPartyIndex {
    /// Index the recomputed evidence against the census beside it.
    fn of(discharged: &[ValidatedMaturityFirstPartyEvidence]) -> Self {
        let cases = maturity_first_party_cases();
        let classes: BTreeMap<RowKey, &'static str> = cases
            .iter()
            .filter_map(|case| {
                case.discharge()
                    .map(|discharge| ((case.section(), case.name()), discharge.expected_class()))
            })
            .collect();
        Self {
            discharged: discharged
                .iter()
                .map(|evidence| {
                    let key = (evidence.section(), evidence.name());
                    let class = classes.get(&key).copied().unwrap_or_default();
                    (key, (evidence.validator(), class))
                })
                .collect(),
            carried: cases
                .iter()
                .filter_map(|case| {
                    case.carried_reason()
                        .map(|reason| ((case.section(), case.name()), reason))
                })
                .collect(),
        }
    }
}

/// Classify one row into exactly one standing.
///
/// Decided from the row's own facts and from the evidence this plan
/// recomputed, never from a list kept beside the matrix: a row that starts
/// declaring another boundary moves here without anything being
/// remembered.
fn classify(
    row: &MaturitySafetyRow,
    plan: &ValidatedMaturityAnnouncementOperationPlan,
    index: &FirstPartyIndex,
) -> Result<MaturityRowStanding, MaturityEvidenceRefusal> {
    let Some(layer) = row.refusing_layer() else {
        return Ok(MaturityRowStanding::OutstandingUnderTypedNonAnswer(
            row.boundary(),
        ));
    };
    match layer {
        EvidenceBoundary::SemanticRequestRejection
        | EvidenceBoundary::CompilerPlanRejection
        | EvidenceBoundary::ConstructorDerivationRejection
        | EvidenceBoundary::BackendEmissionRejection
        | EvidenceBoundary::LinkerRejection
        | EvidenceBoundary::AbiConstructionRejection => first_party_standing(row, index),
        EvidenceBoundary::ReportSemanticProjectionRejection => {
            Ok(MaturityRowStanding::ReportLayerRequired(row.projection()))
        }
        EvidenceBoundary::ExecutorInfrastructureFailure => Err(
            MaturityEvidenceRefusal::RowDeclaresTheInfrastructureBoundary {
                section: row.section(),
                name: row.name(),
            },
        ),
        EvidenceBoundary::ConsensusRejectionBeforeScript
        | EvidenceBoundary::KeyPathRejection
        | EvidenceBoundary::ScriptPathRejection
        | EvidenceBoundary::RelayPolicyRejection
        | EvidenceBoundary::AcceptedTransaction => native_standing(row, plan),
    }
}

/// The standing of one pre-target row.
fn first_party_standing(
    row: &MaturitySafetyRow,
    index: &FirstPartyIndex,
) -> Result<MaturityRowStanding, MaturityEvidenceRefusal> {
    let key = (row.section(), row.name());
    if let Some((validator, class)) = index.discharged.get(&key) {
        return Ok(MaturityRowStanding::FirstPartyDischarged {
            validator: *validator,
            class,
        });
    }
    index
        .carried
        .get(&key)
        .copied()
        .map(MaturityRowStanding::FirstPartyRequired)
        .ok_or_else(
            || MaturityEvidenceRefusal::PreTargetRowCarriesNoFirstPartyCase {
                section: row.section(),
                name: row.name(),
            },
        )
}

/// The standing of one row whose verdict a target produces.
///
/// Before an admitted observation is applied, the row carries the requirement it resolves to under the explicit representation in the sponsorless case, so that a run's observation can be filed against a relation. An unresolved relation disagrees with the published plan and refuses rather than standing as waiting.
fn native_standing(
    row: &MaturitySafetyRow,
    plan: &ValidatedMaturityAnnouncementOperationPlan,
) -> Result<MaturityRowStanding, MaturityEvidenceRefusal> {
    let link = resolve_row(
        plan,
        row,
        MaturityAnnouncementRepresentationPlan::Explicit,
        SponsorCase::Absent,
    )
    .map_err(MaturityEvidenceRefusal::RowLinkUnresolved)?;
    Ok(MaturityRowStanding::NativeRunRequired(match link {
        MaturityRowLink::Resolved(id) => Some(id),
        MaturityRowLink::EveryRelation
        | MaturityRowLink::InactiveInThisCase
        | MaturityRowLink::Blocked(_) => None,
    }))
}

/// Count the classified rows into one bucket per standing.
fn census_from_rows(classified: &[MaturityEvidenceRow]) -> MaturityEvidenceCensus {
    let mut census = MaturityEvidenceCensus {
        rows: classified.len(),
        ..MaturityEvidenceCensus::default()
    };
    for row in classified {
        match row.standing {
            MaturityRowStanding::FirstPartyDischarged { .. } => census.first_party_discharged += 1,
            MaturityRowStanding::FirstPartyRequired(_) => census.first_party_required += 1,
            MaturityRowStanding::NativeAcceptanceObserved { .. } => {
                census.native_acceptance_observed += 1;
            }
            MaturityRowStanding::NativeRefusalObserved(_) => census.native_refusal_observed += 1,
            MaturityRowStanding::NativeDeclaredBoundaryObserved { .. } => {
                census.native_declared_boundary_observed += 1;
            }
            MaturityRowStanding::NativeRefusalAtUnexpectedBoundary(_) => {
                census.native_refusal_at_unexpected_boundary += 1;
            }
            MaturityRowStanding::NativeRunRequired(_) => census.native_run_required += 1,
            MaturityRowStanding::ConstructorContinuityObserved => {
                census.constructor_continuity_observed += 1;
            }
            MaturityRowStanding::RootHistoryObserved => census.root_history_observed += 1,
            MaturityRowStanding::PublicRecoveryObserved => census.public_recovery_observed += 1,
            MaturityRowStanding::ReportLayerObserved => census.report_layer_observed += 1,
            MaturityRowStanding::ReportLayerRequired(_) => census.report_layer_required += 1,
            MaturityRowStanding::OutstandingUnderTypedNonAnswer(_) => {
                census.outstanding_under_typed_non_answer += 1;
            }
            MaturityRowStanding::InfrastructureBlocked => census.infrastructure_blocked += 1,
            MaturityRowStanding::Experimental => census.experimental += 1,
        }
    }
    census
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{
        DEPLOYMENT, MaturityAcceptedControl, MaturityAnnouncementEvidencePlan,
        MaturityBranchPoisonMarker, MaturityCarrierExecution, MaturityCarrierOutcome,
        MaturityConstructorMaterial, MaturityConstructorMaterialAbsence,
        MaturityConstructorMaterialPresence, MaturityContinuityObservation,
        MaturityEvidenceRefusal, MaturityExecutorProvenanceExpectation,
        MaturityMutationRegistryKind, MaturityMutationSite, MaturityNativeRefusal,
        MaturityObservationClass, MaturityPresentConstructorMaterial,
        MaturityRegistryOutstandingReason, MaturityRegistryStanding, MaturityRowBinding,
        MaturityRowStanding, MaturitySubmittedSubject, derive_maturity_evidence_plan_with,
        native_refusal_binds_to_row, site_names_the_rows_subject, stated_executor_provenance,
    };
    use crate::live_owner_observation::{asset_of, decode_hex, outpoint_of};
    use crate::matrix::{EvidenceBoundary, MutationLayer};
    use crate::maturity_closure::announcement_plan;
    use crate::maturity_continuity::{
        MaturityByteSource, MaturityContinuityMutant, MaturityContinuityRefusal,
        MaturityFundedPredecessor, MaturityMutationCarrier, MaturityMutationContext,
        MaturityProjectionInput, MaturitySignatureDisposition, ValidatedMaturityContinuity,
        project_maturity_continuity,
    };
    use crate::maturity_continuity_report::{
        assemble_maturity_continuity_report, validate_maturity_continuity_report,
    };
    use crate::maturity_corpus::maturity_run_of_record;
    use crate::maturity_first_party::maturity_first_party_cases;
    use crate::maturity_fixture::positive_semantic_census;
    use crate::maturity_native::MaturityAnnouncementPlanner;
    use crate::maturity_safety::{
        MaturityCanonicalControl, MaturityIntendedCarrier, MaturityMutationLocator,
        MaturityMutationSubject, MaturitySafetyRow, MaturitySafetySection, row_count, rows,
    };
    use crate::subject::ExperimentalSubject;
    use linker::CandidateDeploymentIdentity;
    use std::fmt::Write as _;
    use std::io::Write as _;
    use std::sync::LazyLock;
    use target_elements_conformance::executor::{OperationStep, TargetOperationPlanner};
    use target_elements_conformance::protocol::{
        FundedOutput, NATIVE_PROTOCOL_SCHEMA, NativeOperationResponse, NativeResourceObservation,
        ObservedOutcomeLayer, OperationSubject, WireOutpoint,
    };
    use target_elements_conformance::provenance::ExpectedExecutorProvenance;
    use transaction::bytes::Txid;
    use transaction::operator_right::BranchContext;

    /// The plan, derived once and read by every test below.
    ///
    /// Derived through the entry that takes the eighth input as an
    /// argument, with the expectation stated as unstated: a test that
    /// depended on the environment it happens to run in would be checking
    /// the harness rather than the plan.
    static PLAN: LazyLock<MaturityAnnouncementEvidencePlan> = LazyLock::new(|| {
        derive_maturity_evidence_plan_with(
            MaturityExecutorProvenanceExpectation::NotStatedByTheOperator,
            absent_material(),
        )
        .expect("the evidence plan derives from its eight inputs")
    });

    /// Explicit absence used by deterministic test derivations.
    #[must_use]
    pub const fn absent_material() -> MaturityConstructorMaterial {
        MaturityConstructorMaterial::Absent(
            MaturityConstructorMaterialAbsence::NotSuppliedToDerivation,
        )
    }

    fn hex(bytes: &[u8]) -> String {
        let mut text = String::new();
        for byte in bytes {
            write!(text, "{byte:02x}").expect("String write");
        }
        text
    }

    fn refused_constructor(
        source: &ValidatedMaturityContinuity,
    ) -> ExperimentalSubject<MaturityContinuityMutant> {
        let mut funded = source.funded().clone();
        funded.program = source.successor().output_program().to_vec();
        assert_ne!(funded.program, source.funded().program);
        let context = MaturityMutationContext::new(
            MaturitySafetySection::PredecessorConstructorFault,
            "wrong-predecessor-program",
            MutationLayer::LinkedConstructorProgram,
            MaturityMutationCarrier::new(
                "funding program replaced by the constructed successor program".to_owned(),
                Vec::new(),
                None,
                "original retained predecessor and deployment".to_owned(),
                MaturitySignatureDisposition::SubmittedUnverified,
            ),
        )
        .expect("exact constructor row");
        let branch = BranchContext::new([0x73; 32], 19).expect("distinct stated context");
        let mutant = MaturityContinuityMutant::observe(
            context,
            MaturityProjectionInput {
                source: source.source().clone(),
                submitted_bytes: source.submitted_bytes(),
                funded: &funded,
                branch,
                bundle: source.bundle(),
                identity: source.identity(),
            },
        )
        .expect("constructed program substitution refuses");
        assert!(matches!(
            mutant.subject().failure(),
            MaturityContinuityRefusal::RetainedContext { .. }
        ));
        mutant
    }

    /// Public-path material shared by evidence and safety-report tests.
    ///
    /// # Panics
    ///
    /// Panics if a fixed test input refuses projection or report validation.
    #[must_use]
    pub fn present_material() -> MaturityConstructorMaterial {
        static MATERIAL: LazyLock<MaturityPresentConstructorMaterial> = LazyLock::new(|| {
            let sources = [archived(), node_free()];
            let references = [&sources[0], &sources[1]];
            let binding = PLAN.binding();
            let provenance = PLAN.executor_provenance();
            let report = assemble_maturity_continuity_report(
                &references,
                binding.clone(),
                provenance.clone(),
            );
            let validated =
                validate_maturity_continuity_report(&report, &references, binding, provenance)
                    .expect("independent report validation");
            MaturityPresentConstructorMaterial::new(
                validated,
                vec![refused_constructor(&sources[1])],
            )
        });
        MaturityConstructorMaterial::Present(MATERIAL.clone())
    }

    fn derive_with_material(
        material: MaturityConstructorMaterial,
    ) -> MaturityAnnouncementEvidencePlan {
        derive_maturity_evidence_plan_with(
            MaturityExecutorProvenanceExpectation::NotStatedByTheOperator,
            material,
        )
        .expect("evidence plan")
    }

    fn present_plan() -> &'static MaturityAnnouncementEvidencePlan {
        static PRESENT: LazyLock<MaturityAnnouncementEvidencePlan> =
            LazyLock::new(|| derive_with_material(present_material()));
        &PRESENT
    }

    fn assert_standings_equal(
        first: &MaturityAnnouncementEvidencePlan,
        second: &MaturityAnnouncementEvidencePlan,
    ) {
        assert_eq!(first.census(), second.census());
        assert_eq!(first.census().answered(), second.census().answered());
        assert_eq!(first.census().outstanding(), second.census().outstanding());
        assert_eq!(first.rows(), second.rows());
        assert_eq!(
            first.root_history_mutations(),
            second.root_history_mutations()
        );
        assert_eq!(first.census().constructor_continuity_observed(), 0);
        assert_eq!(second.census().constructor_continuity_observed(), 0);
        assert!(!first.census().every_required_row_is_answered());
        assert!(!second.census().every_required_row_is_answered());
    }

    #[test]
    fn absent_material_preserves_the_entire_partition_and_has_no_records() {
        let absent = derive_with_material(absent_material());
        assert_standings_equal(&PLAN, &absent);
        assert_eq!(absent.constructor_material(), &absent_material());
        assert_eq!(absent.continuity_records(), []);
        for row in absent.rows() {
            assert_eq!(row.continuity_record_count(absent.continuity_records()), 0);
            assert_eq!(absent.continuity_record_count(row), 0);
        }
        writeln!(
            std::io::stdout().lock(),
            "\nRUN-REPORT evidence absent census={:?} answered={} outstanding={} records={}",
            absent.census(),
            absent.census().answered(),
            absent.census().outstanding(),
            absent.continuity_records().len()
        )
        .expect("census observation");
    }

    #[test]
    fn present_material_associates_exact_records_without_moving_standings() {
        let plan = present_plan();
        let material = plan.constructor_material();
        let MaturityConstructorMaterial::Present(present) = material else {
            panic!("present fixture");
        };
        let report = present.report().report();
        assert_standings_equal(&PLAN, plan);
        assert_eq!(plan.constructor_material(), &present_material());
        assert_eq!(
            plan.constructor_material().presence(),
            MaturityConstructorMaterialPresence::Present
        );
        assert_eq!(present.acceptance(), report.acceptance());
        let records = plan.continuity_records();
        assert_eq!(
            records.len(),
            report.entries().len() + present.mutants().len()
        );
        let (comparisons, agreements) = assert_comparison_records(plan, present);
        assert_mutant_records(plan, present);
        for row in plan.rows() {
            let identity = (row.row().section(), row.row().name());
            let expected = if identity == (MaturitySafetySection::Positive, "sponsorless") {
                report.entries().len()
            } else {
                present
                    .mutants()
                    .iter()
                    .filter(|mutant| {
                        let context = mutant.subject().context().row();
                        identity == (context.section(), context.name())
                    })
                    .count()
            };
            assert_eq!(row.continuity_record_count(records), expected);
            assert_eq!(plan.continuity_record_count(row), expected);
        }
        let sponsorless = plan
            .rows()
            .iter()
            .find(|row| {
                (row.row().section(), row.row().name())
                    == (MaturitySafetySection::Positive, "sponsorless")
            })
            .expect("positive row");
        let mut other_section = records[0].clone();
        other_section.section = MaturitySafetySection::PredecessorConstructorFault;
        assert_eq!(sponsorless.continuity_record_count(&[other_section]), 0);
        writeln!(std::io::stdout().lock(), "\nRUN-REPORT evidence present census={:?} answered={} outstanding={} records={} entries={} mutants={} comparisons={comparisons} agreements={agreements}", plan.census(), plan.census().answered(), plan.census().outstanding(), records.len(), report.entries().len(), present.mutants().len()).expect("census observation");
    }

    fn assert_comparison_records(
        plan: &MaturityAnnouncementEvidencePlan,
        present: &MaturityPresentConstructorMaterial,
    ) -> (usize, usize) {
        let report = present.report().report();
        let records = plan.continuity_records();
        let mut comparisons = 0;
        let mut agreements = 0;
        for (record, entry) in records.iter().zip(report.entries()) {
            assert_eq!(
                (record.section(), record.row()),
                (MaturitySafetySection::Positive, "sponsorless")
            );
            assert_eq!(record.source(), entry.source());
            assert_eq!(record.branch(), entry.branch());
            assert_eq!(record.byte_identity(), entry.byte_identity());
            let MaturityContinuityObservation::ValidatedComparison {
                comparisons: count,
                agreements: agreed,
            } = record.observation()
            else {
                panic!("comparison record");
            };
            comparisons += count;
            agreements += agreed;
        }
        assert_eq!(comparisons, report.census().comparisons());
        assert_eq!(agreements, report.census().agreements());
        (comparisons, agreements)
    }

    fn assert_mutant_records(
        plan: &MaturityAnnouncementEvidencePlan,
        present: &MaturityPresentConstructorMaterial,
    ) {
        let report = present.report().report();
        let records = plan.continuity_records();
        for (record, observed) in records[report.entries().len()..]
            .iter()
            .zip(present.mutants())
        {
            let mutant = observed.subject();
            let row = mutant.context().row();
            assert_eq!(
                (record.section(), record.row()),
                (row.section(), row.name())
            );
            assert_eq!(record.source(), mutant.source());
            assert_eq!(record.branch(), mutant.branch());
            assert!(
                report
                    .entries()
                    .iter()
                    .all(|entry| entry.branch() != record.branch())
            );
            assert_eq!(record.byte_identity(), mutant.digest());
            assert_eq!(
                record.observation(),
                &MaturityContinuityObservation::RefusedMutant {
                    refusal: mutant.failure().clone()
                }
            );
            assert_eq!(records.iter().filter(|record| (record.section(), record.row()) == (row.section(), row.name())).count(), 1);
        }
    }

    fn funding(output: &FundedOutput) -> MaturityFundedPredecessor {
        MaturityFundedPredecessor {
            outpoint: outpoint_of(&output.outpoint).expect("funding outpoint"),
            asset: asset_of(&output.asset).expect("funding asset"),
            amount: output.amount_satoshis,
            program: decode_hex(&output.script).expect("funding program"),
        }
    }

    fn archived() -> ValidatedMaturityContinuity {
        let corpus = maturity_run_of_record().expect("validated archive");
        let identity = corpus.evidence().identity().clone();
        let branch = corpus.evidence().branch();
        let mut planner = MaturityAnnouncementPlanner::new(
            identity.clone(),
            branch,
            crate::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
        )
        .expect("planner");
        let mut next = planner.next_step(None).expect("issuance");
        for (step, response) in &corpus.exchanges()[..2] {
            assert_eq!(next.as_ref(), Some(step));
            next = planner
                .next_step(Some((step.case(), response)))
                .expect("replayed funding");
        }
        assert_eq!(next.as_ref(), Some(&corpus.exchanges()[2].0));
        let submission = match corpus.exchanges()[2].0.subject() {
            OperationSubject::Submission(value) => Some(value),
            _ => None,
        }
        .expect("submission subject");
        assert_eq!(
            planner.submission_bytes(),
            Some(submission.transaction_bytes.as_slice())
        );
        let outputs = &corpus.exchanges()[1].1.funded_outputs;
        assert_eq!(outputs.len(), 1);
        project_maturity_continuity(MaturityProjectionInput {
            source: MaturityByteSource::ArchivedSubmission {
                run_address: corpus.report().run_address().to_owned(),
            },
            submitted_bytes: &submission.transaction_bytes,
            funded: &funding(&outputs[0]),
            branch,
            bundle: planner.bundle(),
            identity: &identity,
        })
        .expect("archived projection")
    }

    fn scripted_response(step: &OperationStep) -> NativeOperationResponse {
        let subject = match step.subject() {
            OperationSubject::Funding(value) => Some(value),
            _ => None,
        }
        .expect("funding subject");
        let asset = format!("01{}fe", "55".repeat(30));
        NativeOperationResponse {
            schema: NATIVE_PROTOCOL_SCHEMA,
            case: step.case().clone(),
            observed_layer: ObservedOutcomeLayer::Accepted,
            observed_detail: None,
            issued_asset: subject.issue_asset.then(|| asset.clone()),
            funded_outputs: vec![FundedOutput {
                outpoint: WireOutpoint {
                    txid: format!("02{}fd", "66".repeat(30)),
                    vout: if subject.issue_asset { 3 } else { 7 },
                },
                asset,
                amount_satoshis: subject.amount_per_output,
                script: hex(&subject.output_program),
            }],
            confidential_funded_outputs: Vec::new(),
            mined_readback: None,
            accepted_txid: None,
            sponsor_witness: Vec::new(),
            script_path_witness: Vec::new(),
            signer_public_key: None,
            signed_profile: None,
            signing_genesis: None,
            signature_bound_to: None,
            resources: NativeResourceObservation::default(),
        }
    }

    fn node_free() -> ValidatedMaturityContinuity {
        let mut genesis = [0x22; 32];
        genesis[0] = 0x01;
        genesis[31] = 0xfe;
        let identity = CandidateDeploymentIdentity::new([0x17; 32], genesis).expect("identity");
        let branch = BranchContext::new([0x41; 32], 7).expect("branch");
        let mut planner = MaturityAnnouncementPlanner::new(
            identity.clone(),
            branch,
            crate::maturity_closure::MaturityWitnessSelection::retained_whole_metadata(),
        )
        .expect("planner");
        let issue = planner.next_step(None).expect("issuance").expect("step");
        let issued = scripted_response(&issue);
        issued.validate_shape().expect("issuance response shape");
        let fund = planner
            .next_step(Some((issue.case(), &issued)))
            .expect("funding")
            .expect("step");
        let response = scripted_response(&fund);
        response.validate_shape().expect("funding response shape");
        let step = planner
            .next_step(Some((fund.case(), &response)))
            .expect("submission")
            .expect("step");
        let submission = match step.subject() {
            OperationSubject::Submission(value) => Some(value),
            _ => None,
        }
        .expect("submission subject");
        assert_eq!(
            planner.submission_bytes(),
            Some(submission.transaction_bytes.as_slice())
        );
        project_maturity_continuity(MaturityProjectionInput {
            source: MaturityByteSource::NodeFreeSubmitReady,
            submitted_bytes: &submission.transaction_bytes,
            funded: &funding(&response.funded_outputs[0]),
            branch,
            bundle: planner.bundle(),
            identity: &identity,
        })
        .expect("node-free projection")
    }

    /// A published pattern in the admitted revision syntax.
    ///
    /// Fixed public bytes and nothing more: no revision here names an
    /// object in any repository and none authorizes anything.
    const STATED_TIP: &str = "0123456789abcdef0123456789abcdef01234567";

    /// The upstream base beside it, in the same syntax and with the same
    /// standing.
    const STATED_BASE: &str = "fedcba9876543210fedcba9876543210fedcba98";

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

    /// The identity a hand-built refusal carries for its control.
    ///
    /// Fixed public fill. Nothing accepted it, which is the point: a
    /// hand-built refusal can exercise this crate's own arithmetic and
    /// can establish nothing whatever about a target.
    const CONTROL_IDENTITY: [u8; 32] = [0x11; 32];

    /// One native refusal, with all eight facts stated.
    fn refusal(
        declared: EvidenceBoundary,
        observed: ObservedOutcomeLayer,
    ) -> MaturityNativeRefusal {
        MaturityNativeRefusal::record(
            declared,
            observed,
            MaturityAcceptedControl::new(
                MaturityCanonicalControl::SponsorlessAnnouncement,
                Txid::from_internal(CONTROL_IDENTITY),
            ),
            "the target refused the mutated candidate".to_owned(),
            MaturitySubmittedSubject::ExactBytes(vec![0x01, 0x02]),
            MaturityMutationSite::Locator(MaturityMutationLocator::WitnessStack),
            MaturityCarrierOutcome::new(
                MaturityIntendedCarrier::AnnouncementLeaf,
                MaturityCarrierExecution::Executed,
            ),
        )
    }

    /// One native refusal built to match a row on every fact the row
    /// declares.
    ///
    /// The control shape, the mutation site and the intended carrier are
    /// taken from the row and the carrier is recorded as having run, so a
    /// departure a test introduces is the only thing about the pair that
    /// differs — which is the shape the live generation's exact-subject
    /// check uses over bytes, transposed to the facts a row declares.
    fn refusal_for(
        row: &MaturitySafetyRow,
        declared: EvidenceBoundary,
        observed: ObservedOutcomeLayer,
    ) -> MaturityNativeRefusal {
        let site = row.locator().map_or_else(
            || MaturityMutationSite::StructuralSeparator("trailing bytes".to_owned()),
            MaturityMutationSite::Locator,
        );
        MaturityNativeRefusal::record(
            declared,
            observed,
            MaturityAcceptedControl::new(row.control(), Txid::from_internal(CONTROL_IDENTITY)),
            "the target refused the mutated candidate".to_owned(),
            MaturitySubmittedSubject::ExactBytes(vec![0x01, 0x02]),
            site,
            MaturityCarrierOutcome::new(row.carrier(), MaturityCarrierExecution::Executed),
        )
    }

    /// The same refusal with its mutation site replaced and nothing else.
    fn with_site(
        refusal: &MaturityNativeRefusal,
        site: MaturityMutationSite,
    ) -> MaturityNativeRefusal {
        MaturityNativeRefusal::record(
            refusal.declared_boundary(),
            refusal.observed_layer(),
            refusal.accepted_control(),
            refusal.refusal_detail().to_owned(),
            refusal.submitted().clone(),
            site,
            refusal.carrier(),
        )
    }

    /// The same refusal with its carrier outcome replaced and nothing
    /// else.
    fn with_carrier(
        refusal: &MaturityNativeRefusal,
        carrier: MaturityCarrierOutcome,
    ) -> MaturityNativeRefusal {
        MaturityNativeRefusal::record(
            refusal.declared_boundary(),
            refusal.observed_layer(),
            refusal.accepted_control(),
            refusal.refusal_detail().to_owned(),
            refusal.submitted().clone(),
            refusal.site().clone(),
            carrier,
        )
    }

    /// The same refusal with its accepted control replaced and nothing
    /// else.
    fn with_control(
        refusal: &MaturityNativeRefusal,
        control: MaturityAcceptedControl,
    ) -> MaturityNativeRefusal {
        MaturityNativeRefusal::record(
            refusal.declared_boundary(),
            refusal.observed_layer(),
            control,
            refusal.refusal_detail().to_owned(),
            refusal.submitted().clone(),
            refusal.site().clone(),
            refusal.carrier(),
        )
    }

    #[test]
    fn the_plan_classifies_every_row_of_the_matrix_exactly_once() {
        let plan = &*PLAN;
        assert_eq!(plan.rows().len(), row_count());
        assert_eq!(plan.census().rows(), row_count());
        for (classified, row) in plan.rows().iter().zip(rows()) {
            assert_eq!(
                classified.row().name(),
                row.name(),
                "the classification walks the matrix in the matrix's own order",
            );
            assert_eq!(classified.row().section(), row.section());
        }
    }

    #[test]
    fn the_census_buckets_are_the_figures_the_inputs_recompute() {
        let census = PLAN.census();
        assert_eq!(present_plan().census(), census);
        let cases = maturity_first_party_cases();
        let discharged = cases
            .iter()
            .filter(|case| case.discharge().is_some())
            .count();
        let pre_target = rows()
            .iter()
            .filter(|row| {
                row.refusing_layer()
                    .is_some_and(EvidenceBoundary::is_pre_target)
            })
            .count();
        let report_required = rows()
            .iter()
            .filter(|row| {
                row.refusing_layer() == Some(EvidenceBoundary::ReportSemanticProjectionRejection)
            })
            .count();
        let run_required = rows()
            .iter()
            .filter(|row| {
                row.refusing_layer().is_some_and(|layer| {
                    layer.requires_target_execution()
                        && layer != EvidenceBoundary::ReportSemanticProjectionRejection
                })
            })
            .count();
        let outstanding = rows()
            .iter()
            .filter(|row| row.boundary().is_typed_non_answer())
            .count();

        assert_eq!(census.first_party_discharged(), discharged);
        assert_eq!(census.first_party_discharged(), PLAN.discharged().len());
        assert_eq!(census.first_party_required(), pre_target - discharged);
        assert_eq!(census.native_run_required(), run_required - 1);
        assert_eq!(census.native_declared_boundary_observed(), 1);
        assert_eq!(census.report_layer_required(), report_required);
        assert_eq!(census.outstanding_under_typed_non_answer(), outstanding);

        // Host records do not change the standing partition.
        assert_eq!(census.native_acceptance_observed(), 0);
        assert_eq!(census.native_refusal_observed(), 0);
        assert_eq!(census.native_refusal_at_unexpected_boundary(), 0);
        assert_eq!(census.constructor_continuity_observed(), 0);
        assert_eq!(census.root_history_observed(), 0);
        assert_eq!(census.public_recovery_observed(), 0);
        assert_eq!(census.report_layer_observed(), 0);
        assert_eq!(census.infrastructure_blocked(), 0);
        assert_eq!(census.experimental(), 0);

        assert_eq!(
            census.first_party_discharged()
                + census.first_party_required()
                + census.native_run_required()
                + census.native_declared_boundary_observed()
                + census.report_layer_required()
                + census.outstanding_under_typed_non_answer(),
            row_count(),
            "the buckets partition the matrix's own denominator",
        );
    }

    #[test]
    fn only_the_discharged_and_observed_standings_are_answered() {
        let at_the_boundary = MaturityRowStanding::from_native_refusal(refusal(
            EvidenceBoundary::ScriptPathRejection,
            ObservedOutcomeLayer::ScriptPathRejection,
        ));
        let elsewhere = MaturityRowStanding::from_native_refusal(refusal(
            EvidenceBoundary::ScriptPathRejection,
            ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
        ));
        assert!(
            matches!(
                at_the_boundary,
                MaturityRowStanding::NativeRefusalObserved(_)
            ),
            "a refusal at the declared boundary answers the row",
        );
        assert!(at_the_boundary.is_answered());
        assert!(
            matches!(
                elsewhere,
                MaturityRowStanding::NativeRefusalAtUnexpectedBoundary(_)
            ),
            "a refusal anywhere else is not the row's answer",
        );
        assert!(!elsewhere.is_answered());
        assert!(!MaturityRowStanding::InfrastructureBlocked.is_answered());
        assert!(!MaturityRowStanding::Experimental.is_answered());

        for classified in PLAN.rows() {
            let answered = classified.standing().is_answered();
            assert_eq!(
                answered,
                matches!(
                    classified.standing(),
                    MaturityRowStanding::FirstPartyDischarged { .. }
                        | MaturityRowStanding::NativeDeclaredBoundaryObserved { .. }
                ),
                "answers are recomputed first-party refusals or the admitted positive boundary",
            );
        }
    }

    #[test]
    fn required_rows_remain_outstanding_after_the_admitted_run() {
        let census = PLAN.census();
        assert_eq!(present_plan().census(), census);
        assert!(!census.every_required_row_is_answered());
        assert_eq!(
            census.answered(),
            census.first_party_discharged() + census.native_declared_boundary_observed()
        );
        assert_eq!(
            census.answered() + census.outstanding(),
            census.rows(),
            "the two halves of the partition are counted apart and meet at the denominator",
        );
        assert_eq!(
            census.outstanding(),
            census.rows()
                - census.first_party_discharged()
                - census.native_declared_boundary_observed(),
        );
    }

    #[test]
    fn absent_constructor_material_and_history_carry_their_reasons() {
        let history = PLAN.root_history_mutations();
        assert_eq!(PLAN.constructor_material(), &absent_material(),);
        assert_eq!(history.kind(), MaturityMutationRegistryKind::RootHistory);
        assert_eq!(
            history.reason(),
            MaturityRegistryOutstandingReason::BranchIndexedHistoryIsNotProjectable,
        );
        assert_eq!(history.mutations().len(), 0);
        assert_eq!(history.census().mutations(), 0);
        assert_eq!(
            history.census().standing(),
            MaturityRegistryStanding::OutstandingUntilItsMaterialLands,
        );
        assert_eq!(history.reason().kind(), history.kind());
    }

    #[test]
    fn a_poison_marker_carries_its_bytes_and_no_standing_can_hold_it() {
        // A model-falsifying observation: an announcement output read back
        // carrying a metadata record whose maturity field decodes to a
        // value the transition has no step to produce. No semantic step is
        // a preimage of it, so there is nothing for an ordinary standing to
        // say about it.
        let observed = vec![0x53, 0x54, 0x41, 0x54, 0x45, 0xff, 0xff, 0x00];
        let branch = BranchContext::new([0x4d; 32], 7).expect("the branch identifier is non-zero");
        let marker = MaturityBranchPoisonMarker::observe(branch, observed.clone());

        assert_eq!(marker.evidence(), observed.as_slice());
        assert_eq!(
            marker.class(),
            MaturityObservationClass::ModelFalsifyingWithNoPreimage,
        );
        assert_eq!(marker.branch(), &branch);
        assert_eq!(MaturityObservationClass::ALL.len(), 3);
        assert!(!marker.class().is_quantified_over());

        // The exclusion is a property of the standing's fields rather than
        // of anything this test does: no member of the vocabulary has a
        // field of the marker's type, the marker converts into none of the
        // types those fields have, and a standing holding one would not
        // compile. What a test can check is that the vocabulary is the one
        // the exclusion was argued over, which the exhaustive match the
        // census performs pins for every member.
        let vocabulary = [
            MaturityRowStanding::ConstructorContinuityObserved,
            MaturityRowStanding::RootHistoryObserved,
            MaturityRowStanding::PublicRecoveryObserved,
            MaturityRowStanding::ReportLayerObserved,
            MaturityRowStanding::InfrastructureBlocked,
            MaturityRowStanding::Experimental,
        ];
        for standing in &vocabulary {
            assert_eq!(
                standing.is_answered(),
                !matches!(
                    standing,
                    MaturityRowStanding::InfrastructureBlocked | MaturityRowStanding::Experimental
                ),
            );
        }
    }

    #[test]
    fn the_census_renders_the_quantifier_the_guarantees_are_stated_under() {
        let quantifier = PLAN.census().quantifier();
        assert_eq!(
            quantifier.admitted(),
            [
                MaturityObservationClass::IntendedTransition,
                MaturityObservationClass::ModeledUnintendedEnvironmentTransition,
            ],
        );
        assert_eq!(
            quantifier.excluded(),
            MaturityObservationClass::ModelFalsifyingWithNoPreimage,
        );
        for class in MaturityObservationClass::ALL {
            assert_eq!(
                class.is_quantified_over(),
                quantifier.admitted().contains(class),
            );
        }
        let rendered = quantifier.to_string();
        assert!(
            rendered.contains("conditional on no model-falsifying observation"),
            "the rendering states the condition the figures were computed under: {rendered}",
        );
    }

    #[test]
    fn the_plan_exposes_exactly_the_eight_inputs() {
        let plan = &*PLAN;
        let published = announcement_plan().expect("the announcement plan derives");
        assert_eq!(plan.operation_plan(), &published);
        assert_eq!(plan.binding().deployment(), DEPLOYMENT);
        assert_eq!(plan.binding().parameters(), plan.bundle().deployment());
        assert_eq!(
            plan.abi().contract(),
            plan.binding().target().definition().version(),
        );
        let census = positive_semantic_census().expect("the fixture census builds");
        assert_eq!(
            plan.semantic_fixtures().len(),
            census
                .iter()
                .filter(|entry| entry.fixture().is_some())
                .count(),
        );
        assert_eq!(
            plan.executor_provenance(),
            &MaturityExecutorProvenanceExpectation::NotStatedByTheOperator,
        );
        assert_eq!(plan.constructor_material(), &absent_material());
        assert_eq!(plan.root_history_mutations().census().mutations(), 0);

        // The same derivation under a stated expectation: the eighth input
        // is the argument, so a caller states it and the plan carries what
        // was stated rather than what the environment happened to hold.
        let expectation = ExpectedExecutorProvenance::new(STATED_TIP, STATED_BASE, [])
            .expect("the stated pattern is in the admitted syntax");
        let stated = derive_maturity_evidence_plan_with(
            MaturityExecutorProvenanceExpectation::Stated(expectation.clone()),
            absent_material(),
        )
        .expect("the plan derives under a stated expectation");
        assert_eq!(stated.executor_provenance().stated(), Some(&expectation));
        assert_eq!(stated.census(), plan.census());
    }

    #[test]
    fn the_environment_read_states_an_expectation_only_where_both_revisions_are_stated() {
        let tip = std::env::var("TRIPOD_OPERATION_EXECUTOR_TIP")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let base = std::env::var("TRIPOD_OPERATION_UPSTREAM_BASE")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let read = stated_executor_provenance();
        match (tip, base) {
            (Some(_), Some(_)) => assert!(
                matches!(
                    read,
                    Ok(MaturityExecutorProvenanceExpectation::Stated(_))
                        | Err(MaturityEvidenceRefusal::ExecutorProvenanceExpectationMalformed(_))
                ),
                "a stated pair is either the expectation or a refusal naming its syntax",
            ),
            _ => assert_eq!(
                read,
                Ok(MaturityExecutorProvenanceExpectation::NotStatedByTheOperator),
                "half an expectation is none",
            ),
        }
    }

    /// A refusal matching a row on every fact binds, and a departure in
    /// the boundary or the subject does not.
    ///
    /// One fact at a time, with everything else the matching refusal's
    /// own. A check that moved two facts could not say which one the
    /// verdict came from, and a verdict that names its condition is the
    /// whole reason the reader is typed rather than a boolean.
    #[test]
    fn a_refusal_matching_every_fact_binds_and_a_departed_boundary_or_subject_does_not() {
        let row = rows()
            .iter()
            .find(|row| row.refusing_layer() == Some(EvidenceBoundary::ScriptPathRejection))
            .expect("the matrix declares a script-path row");
        let locator = row.locator().expect("a negative row points somewhere");
        assert_ne!(locator.subject(), MaturityMutationSubject::MetadataEncoding);
        let matching = refusal_for(
            row,
            EvidenceBoundary::ScriptPathRejection,
            ObservedOutcomeLayer::ScriptPathRejection,
        );
        assert_eq!(
            native_refusal_binds_to_row(row, &matching),
            MaturityRowBinding::Bound,
        );

        // The observed layer, and then the boundary the refusal was
        // recorded against: a row is answered at its own boundary or not
        // at all, and a refusal about another boundary is about another
        // row.
        let observed_earlier = refusal_for(
            row,
            EvidenceBoundary::ScriptPathRejection,
            ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
        );
        assert_eq!(
            native_refusal_binds_to_row(row, &observed_earlier),
            MaturityRowBinding::ObservedElsewhere,
        );
        let recorded_elsewhere = refusal_for(
            row,
            EvidenceBoundary::KeyPathRejection,
            ObservedOutcomeLayer::KeyPathRejection,
        );
        assert_eq!(
            native_refusal_binds_to_row(row, &recorded_elsewhere),
            MaturityRowBinding::ObservedElsewhere,
        );

        // The subject, as another locator and as a separator, which
        // answers only the rows whose subject sits between fields.
        let other_locator = rows()
            .iter()
            .filter_map(MaturitySafetyRow::locator)
            .find(|candidate| candidate.subject() != locator.subject())
            .expect("the matrix names more than one subject");
        let another_subject = with_site(&matching, MaturityMutationSite::Locator(other_locator));
        assert_eq!(
            native_refusal_binds_to_row(row, &another_subject),
            MaturityRowBinding::AnotherSubject,
        );
        let separator = with_site(
            &matching,
            MaturityMutationSite::StructuralSeparator("field order".to_owned()),
        );
        assert_eq!(
            native_refusal_binds_to_row(row, &separator),
            MaturityRowBinding::AnotherSubject,
        );
    }

    /// A departure in the carrier or in the control's shape does not
    /// bind either.
    ///
    /// The carrier fails in both of its ways — another carrier, and the
    /// row's own carrier never reached — because a refusal that stopped
    /// before the carrier establishes nothing about it. The control
    /// fails at its shape while carrying the identity the match carried,
    /// which is what shows the identity alone cannot answer the
    /// condition.
    #[test]
    fn a_departed_carrier_or_control_does_not_bind() {
        let row = rows()
            .iter()
            .find(|row| row.refusing_layer() == Some(EvidenceBoundary::ScriptPathRejection))
            .expect("the matrix declares a script-path row");
        let matching = refusal_for(
            row,
            EvidenceBoundary::ScriptPathRejection,
            ObservedOutcomeLayer::ScriptPathRejection,
        );
        let other_carrier = if row.carrier() == MaturityIntendedCarrier::Report {
            MaturityIntendedCarrier::AnnouncementLeaf
        } else {
            MaturityIntendedCarrier::Report
        };
        let departed = with_carrier(
            &matching,
            MaturityCarrierOutcome::new(other_carrier, MaturityCarrierExecution::Executed),
        );
        assert_eq!(
            native_refusal_binds_to_row(row, &departed),
            MaturityRowBinding::CarrierDeparted,
        );
        let not_reached = with_carrier(
            &matching,
            MaturityCarrierOutcome::new(row.carrier(), MaturityCarrierExecution::NotReached),
        );
        assert_eq!(
            native_refusal_binds_to_row(row, &not_reached),
            MaturityRowBinding::CarrierDeparted,
        );
        let other_shape = if row.control() == MaturityCanonicalControl::SponsoredAnnouncement {
            MaturityCanonicalControl::SponsorlessAnnouncement
        } else {
            MaturityCanonicalControl::SponsoredAnnouncement
        };
        let another_control = with_control(
            &matching,
            MaturityAcceptedControl::new(other_shape, matching.control_identity()),
        );
        assert_eq!(
            native_refusal_binds_to_row(row, &another_control),
            MaturityRowBinding::AnotherControl,
        );
        assert_eq!(
            another_control.control_identity(),
            matching.control_identity()
        );
    }

    /// A separator answers exactly the rows whose subject sits between
    /// fields, and a locator answers exactly its own row's subject.
    ///
    /// Walked over every row that points somewhere, so the rule is a
    /// rule rather than an example: the metadata encoding is the one
    /// subject whose documented places include a separator, a field
    /// order and a trailing region, and every other subject is a field
    /// a run can name.
    #[test]
    fn a_structural_separator_answers_only_the_subject_that_sits_between_fields() {
        let separator = MaturityMutationSite::StructuralSeparator("domain separator".to_owned());
        let mut between_fields = 0usize;
        let mut in_a_field = 0usize;
        for row in rows() {
            let Some(locator) = row.locator() else {
                continue;
            };
            assert!(
                site_names_the_rows_subject(row, &MaturityMutationSite::Locator(locator)),
                "{row} is not named by its own locator",
            );
            if locator.subject() == MaturityMutationSubject::MetadataEncoding {
                between_fields += 1;
                assert!(
                    site_names_the_rows_subject(row, &separator),
                    "{row} changes the encoding and refuses the form a run can report it in",
                );
            } else {
                in_a_field += 1;
                assert!(
                    !site_names_the_rows_subject(row, &separator),
                    "{row} accepts a separator for a subject that sits in a field",
                );
            }
        }
        assert_eq!(between_fields, 9);
        assert_eq!(in_a_field, 183);
        assert_eq!(between_fields + in_a_field, 192);
    }

    /// No refusal binds to a row that admits none.
    ///
    /// Two populations, and the second is the one a boundary comparison
    /// alone would miss: a row with no declared layer has no boundary an
    /// observation could equal, and a row that is its own control is
    /// what a mutant departs from, so a refusal offered against it is a
    /// category error rather than a near miss.
    #[test]
    fn no_refusal_binds_to_a_row_that_admits_none() {
        let mut admitting_none = 0usize;
        for row in rows() {
            let own_control = row.control() == MaturityCanonicalControl::TheRowIsTheControl;
            if row.refusing_layer().is_some() && !own_control {
                continue;
            }
            admitting_none += 1;
            let declared = row
                .refusing_layer()
                .unwrap_or(EvidenceBoundary::ScriptPathRejection);
            let refusal = refusal_for(row, declared, ObservedOutcomeLayer::ScriptPathRejection);
            assert_eq!(
                native_refusal_binds_to_row(row, &refusal),
                MaturityRowBinding::RowAdmitsNoRefusal,
                "{row} admits a refusal it has no boundary or no mutant for",
            );
        }
        assert_eq!(admitting_none, 66);
    }

    /// The classifier, walked row by row against every observed layer.
    ///
    /// The mapping's own test proves the mapping diagonal over the two
    /// vocabularies, and two hand-built refusals showed the classifier
    /// reads it. Neither says what the classifier does with a row: that
    /// a refusal recorded at each layer in turn answers a row exactly
    /// where the layer is the row's own, and that the rows a run can
    /// answer at all are the forty-three the matrix declares a
    /// target verdict for, are facts about the matrix and the classifier
    /// together.
    #[test]
    fn from_native_refusal_classifies_every_declared_row_at_every_observed_layer() {
        let mut pairs = 0usize;
        let mut at_the_boundary = 0usize;
        let mut bound = 0usize;
        for row in rows() {
            let Some(boundary) = row.refusing_layer() else {
                continue;
            };
            for observed in OBSERVED_LAYERS {
                pairs += 1;
                let refusal = refusal_for(row, boundary, *observed);
                let binding = native_refusal_binds_to_row(row, &refusal);
                let standing = MaturityRowStanding::from_native_refusal(refusal);
                let diagonal = matches!(standing, MaturityRowStanding::NativeRefusalObserved(_));
                assert_eq!(
                    diagonal,
                    standing.is_answered(),
                    "{row} at {observed:?} is answered off the diagonal",
                );
                if diagonal {
                    at_the_boundary += 1;
                } else {
                    assert!(
                        matches!(
                            standing,
                            MaturityRowStanding::NativeRefusalAtUnexpectedBoundary(_)
                        ),
                        "{row} at {observed:?} classified as neither refusal member",
                    );
                }
                if row.control() == MaturityCanonicalControl::TheRowIsTheControl {
                    assert_eq!(
                        binding,
                        MaturityRowBinding::RowAdmitsNoRefusal,
                        "{row} is its own control and still admits a refusal",
                    );
                    continue;
                }
                assert_eq!(
                    binding == MaturityRowBinding::Bound,
                    diagonal,
                    "{row} at {observed:?} binds and classifies differently",
                );
                if binding == MaturityRowBinding::Bound {
                    bound += 1;
                }
            }
        }
        assert_eq!(pairs, 987);
        assert_eq!(at_the_boundary, 43);
        assert_eq!(bound, 42);
    }

    /// The census figures the readers this module adds leave unmoved.
    ///
    /// A reader that decides whether a refusal answers a row must not
    /// answer one: the buckets are stated as the figures they are, so a
    /// standing minted by a reader, or a row retyped underneath the
    /// classification, moves a number here rather than passing quietly.
    #[test]
    fn the_census_figures_are_unmoved_by_the_binding_readers() {
        let census = PLAN.census();
        assert_eq!(census.rows(), 206);
        assert_eq!(census.first_party_discharged(), 40);
        assert_eq!(census.first_party_required(), 43);
        assert_eq!(census.native_run_required(), 42);
        assert_eq!(census.report_layer_required(), 15);
        assert_eq!(census.outstanding_under_typed_non_answer(), 65);
        assert_eq!(census.answered(), 41);
        assert_eq!(census.native_declared_boundary_observed(), 1);
        assert_eq!(census.native_acceptance_observed(), 0);
        assert_eq!(census.native_refusal_observed(), 0);
        assert_eq!(census.native_refusal_at_unexpected_boundary(), 0);
        assert_eq!(
            census.first_party_discharged()
                + census.first_party_required()
                + census.native_run_required()
                + census.native_declared_boundary_observed()
                + census.report_layer_required()
                + census.outstanding_under_typed_non_answer(),
            row_count(),
        );
        assert!(!census.every_required_row_is_answered());
    }

    #[test]
    fn the_positive_boundary_names_the_admitted_run_without_claiming_acceptance() {
        let corpus = crate::maturity_corpus::maturity_run_of_record().expect("admitted run");
        let observed: Vec<_> = PLAN
            .rows()
            .iter()
            .filter(|row| {
                matches!(
                    row.standing(),
                    MaturityRowStanding::NativeDeclaredBoundaryObserved { .. }
                )
            })
            .collect();
        assert_eq!(observed.len(), 1);
        assert_eq!(observed[0].row().section(), MaturitySafetySection::Positive);
        assert_eq!(observed[0].row().name(), "sponsorless");
        assert_eq!(
            observed[0].standing(),
            &MaturityRowStanding::NativeDeclaredBoundaryObserved {
                run_address: crate::maturity_corpus::MATURITY_RUN_ADDRESS,
                recorded_detail: corpus.recorded_refusal_detail(),
            }
        );
        assert_eq!(PLAN.census().native_acceptance_observed(), 0);
        assert_eq!(PLAN.census().native_refusal_observed(), 0);
        assert_eq!(
            corpus.evidence().acceptance_obligation(),
            crate::maturity_native::MaturityAcceptanceObligation::Outstanding {
                routes: [
                    crate::maturity_native::MaturityAcceptanceRoute::RelayWitnessRestructure,
                    crate::maturity_native::MaturityAcceptanceRoute::BlockLayerSubmissionSubject,
                ],
            }
        );
    }
}
