//! The canonical operation evidence plan of Guide-12 §16.4.
//!
//! The plan is the only route to canonical standing. Its fields are
//! private, its constructor is checked, and everything it admits comes
//! out wrapped in a [`CanonicalSubject`]; anything assembled outside it
//! is experimental by construction, which is what "ad hoc vectors
//! produce experimental reports only" means once it is a type.
//!
//! # The censuses are recomputed, never restated
//!
//! §1.3 says no relation disappears at a package boundary, and the only
//! way to know that is to count both sides. Every number in
//! [`PlanCensus`] is derived from the validated operation plan the
//! fixture bundle carries — the relations it publishes, the cases it
//! publishes, the coverage requirements it publishes — and then checked
//! against an independently computed value. A census this package
//! asserted from prose would be this package agreeing with itself.

use std::collections::{BTreeMap, BTreeSet};

use compiler::operation_plan::{
    CoverageRequirementId, EvidenceRole, RelationActivity, RelationCaseKey, SponsorCase,
    TargetCoverageObligation, TargetCoverageRequirement,
};
use realization::RelationId;
use target_elements_conformance::protocol::ObservedOutcomeLayer;
use transaction::FundingCeremonyStep;

use crate::bundle::FixtureBundle;
use crate::divergence::target_amount_standing;
use crate::error::VectorError;
use crate::fixture::{CompactAshSemanticCase, positive_semantic_census};
use crate::materialize::{
    AshFunding, MaterializedTargetVector, TargetVectorId, has_candidate_program, is_materializable,
    materialize, needs_authorization, vector_id,
};
use crate::matrix::{EvidenceBoundary, class_count};
use crate::mutation::NegativeMutation;
use crate::report::ValidatedCompactAshOperationReport;
use crate::subject::CanonicalSubject;
use crate::violation::matching_requirement;

/// Why a coverage row carries no observation yet.
///
/// §19.3 rules that lifecycle incompleteness is a status and not a
/// passing target case, so a row's unobserved state is named rather than
/// left blank. A blank row and a satisfied row must not look alike.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OutstandingReason {
    /// No target has executed anything in this wave.
    NoTargetHasExecuted,
    /// The requirement is answered by an external report that does not
    /// exist yet.
    ExternalReportAbsent,
    /// The requirement's boundary is compiler-static or
    /// backend-structural, and its structural evidence is not assembled
    /// in this wave.
    StructuralEvidenceNotAssembled,
}

/// Where one negative requirement's refusal can be observed.
///
/// §19.2's negative half is not one kind of evidence. A mutation the
/// target must refuse, a mutation first-party code refuses before any
/// transaction exists, and a mutation nothing in this candidate can
/// stage are three different claims, and a census that called them all
/// "negative coverage" would let the third hide behind the first.
///
/// # Derived, never marked
///
/// The arm is a function of the requirement's own [`EvidenceRole`],
/// which the compiler fixes from its coverage boundary. Nothing here
/// reads a list somebody maintained: a requirement that moved to
/// another boundary reclassifies itself, and a boundary that stopped
/// producing a role would fail to classify rather than silently keep
/// its old column.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum NegativeObservability {
    /// A complete mutated transaction exists and the target must refuse
    /// it. The only arm whose discharge needs a live target.
    TargetExecutable,
    /// The refusal happens in first-party code, at the compiler's own
    /// analysis or in the emitted structure, before any transaction is
    /// built. No target is involved and none may be credited.
    FirstPartyRefusal,
    /// Nothing this candidate can stage produces the observation,
    /// because the evidence is a typed external report that does not
    /// exist. Never dischargeable by a run, and counted so that no
    /// reading of the census mistakes it for work merely not done yet.
    UnreachableInCandidate,
}

/// Which arm a negative requirement's evidence role puts it in.
///
/// `None` for a role no negative requirement carries, so a future
/// boundary that started emitting negatives would surface here rather
/// than being folded into whichever column looked closest.
const fn negative_observability(role: &EvidenceRole) -> Option<NegativeObservability> {
    match role {
        EvidenceRole::TargetExecution => Some(NegativeObservability::TargetExecutable),
        EvidenceRole::CompilerAnalysisResult | EvidenceRole::EmittedStructure => {
            Some(NegativeObservability::FirstPartyRefusal)
        }
        EvidenceRole::ExternalReport { .. } => Some(NegativeObservability::UnreachableInCandidate),
        // A vacuous relation-case states an acceptance obligation and
        // never a mutation, so this role reaching a negative row would
        // be a compiler change rather than a classification gap.
        EvidenceRole::InactiveCaseAcceptance => None,
    }
}

/// What is known about one coverage requirement.
///
/// Two variants: nothing observed, or one target run that answered it.
/// It is an enum rather than an `Option` so that the observed arm was
/// *added* rather than filling a hole, and so that no row can be read
/// as satisfied by being absent.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum CoverageObservation {
    /// Nothing has been observed for this requirement.
    Outstanding(OutstandingReason),
    /// One target run answered this requirement.
    ///
    /// # Every field here came off a transcript
    ///
    /// §19.1's positive coverage needs an accepted target transaction
    /// *and* a matched semantic projection, and §1.4 keeps the two
    /// verdicts apart. So a discharged row records which vector was
    /// submitted, what the target made of it, and whether the accepted
    /// projection compared equal — three separate facts, none of which
    /// can be inferred from the others.
    ///
    /// A row is never built from a plan's intention. The only
    /// constructor is [`CompactAshEvidencePlan::discharge`], which reads
    /// submissions the executor recorded.
    Observed(ObservedCoverage),
    /// One mutation the target refused where its class said it would.
    ///
    /// The negative half's own arm. §19.2 asks a different question
    /// from §19.1 — a mutated transaction the target rejected, at the
    /// boundary the class named in advance — so it is a separate
    /// variant rather than an [`ObservedCoverage`] with the verdict
    /// read backwards.
    ObservedRefusal(ObservedRefusal),
}

/// What one refused mutation established about one requirement.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ObservedRefusal {
    vector: TargetVectorId,
    mutation: NegativeMutation,
    layer: ObservedOutcomeLayer,
}

impl ObservedRefusal {
    /// The accepted vector the mutation was made from.
    #[must_use]
    pub const fn vector(&self) -> TargetVectorId {
        self.vector
    }

    /// The arm that made the change.
    #[must_use]
    pub const fn mutation(&self) -> NegativeMutation {
        self.mutation
    }

    /// Where the target put the mutated transaction.
    #[must_use]
    pub const fn layer(&self) -> ObservedOutcomeLayer {
        self.layer
    }
}

/// What one target run established about one requirement.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ObservedCoverage {
    vector: TargetVectorId,
    layer: ObservedOutcomeLayer,
    projection: ProjectionComparison,
}

impl ObservedCoverage {
    /// The vector whose submission answered this requirement.
    #[must_use]
    pub const fn vector(&self) -> TargetVectorId {
        self.vector
    }

    /// Where the target put that submission.
    #[must_use]
    pub const fn layer(&self) -> ObservedOutcomeLayer {
        self.layer
    }

    /// What the accepted-projection comparison found.
    #[must_use]
    pub const fn projection(&self) -> ProjectionComparison {
        self.projection
    }
}

/// The result of §17.4's accepted-projection comparison.
///
/// §1.4 makes acceptance and semantic agreement two verdicts, and this
/// is the second one. It is not a boolean: a comparison that was never
/// performed — because the target never accepted anything — is a third
/// state, and collapsing it into `false` would make an unaccepted
/// transaction look like a semantic disagreement.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ProjectionComparison {
    /// The target accepted, and every §17.4 term compared equal.
    Matched,
    /// The target accepted, and some §17.4 term differed.
    Differed,
    /// No comparison was performed, because nothing was accepted.
    NotPerformed,
}

impl CoverageObservation {
    /// Whether this observation discharges its requirement.
    ///
    /// §19.1 is the bar and both halves of it are checked here: the
    /// target accepted the transaction, and the accepted projection
    /// matched. Acceptance alone is not a pass
    /// `(´[PLAN-rule:guide12-exec:two-verdicts]´)`.
    ///
    /// Deliberately a method rather than a pattern match at each call
    /// site, so that the one place that decides cannot be forgotten.
    #[must_use]
    pub const fn is_discharged(self) -> bool {
        match self {
            Self::Outstanding(_) => false,
            Self::Observed(observed) => matches!(
                (observed.layer, observed.projection),
                (
                    ObservedOutcomeLayer::Accepted,
                    ProjectionComparison::Matched
                )
            ),
            // The constructor already checked every §19.2 condition
            // this package can check, and refuses to build the value
            // otherwise, so reaching here means the refusal counted.
            Self::ObservedRefusal(_) => true,
        }
    }
}

/// What one submission established, as a run reports it.
///
/// The vector that was submitted, where the target put it, and what the
/// §17.4 comparison found — three separate facts, none inferable from
/// the others.
///
/// Crate-private, and the reason is the whole of `G13-R01`. Every
/// component is a public enum or a public identity a consumer can name
/// for itself, so a tuple is exactly as easy to author as to observe;
/// while this alias was public, the counters a gate reads could be moved
/// without an executor, a target binding, or a comparison anyone
/// performed. The public route is
/// [`CompactAshEvidencePlan::discharge_observed_run`], whose argument
/// cannot be authored.
pub(crate) type Outcome = (TargetVectorId, ObservedOutcomeLayer, ProjectionComparison);

/// What one submitted mutation established, as a run reports it.
///
/// The arm that made the change, the accepted vector it was made from,
/// and where the target put the result. Crate-private for the reason
/// [`Outcome`] gives, and with an edge of its own: the negative half's
/// discharge predicate accepts every refusal it is handed, so a stated
/// one was indistinguishable from an observed one.
pub(crate) type MutantObservation = (NegativeMutation, TargetVectorId, ObservedOutcomeLayer);

/// One row of §19's relation-indexed coverage matrix.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationCoverageRow {
    key: RelationCaseKey,
    activity: RelationActivity,
    role: EvidenceRole,
    positive: bool,
    observability: Option<NegativeObservability>,
    carrier_alternatives: usize,
    projection_required: bool,
    observation: CoverageObservation,
}

impl RelationCoverageRow {
    /// The relation-case this row indexes.
    #[must_use]
    pub const fn key(&self) -> &RelationCaseKey {
        &self.key
    }

    /// Whether the relation is active in this case.
    #[must_use]
    pub const fn activity(&self) -> RelationActivity {
        self.activity
    }

    /// Which artifact answers this requirement.
    #[must_use]
    pub const fn role(&self) -> &EvidenceRole {
        &self.role
    }

    /// Whether the requirement is the positive half.
    #[must_use]
    pub const fn is_positive(&self) -> bool {
        self.positive
    }

    /// Where this row's refusal can be observed, for a negative row.
    ///
    /// `None` for every positive row, which has no refusal to place.
    #[must_use]
    pub const fn observability(&self) -> Option<NegativeObservability> {
        self.observability
    }

    /// How many carrier assignments the relation-case admits.
    #[must_use]
    pub const fn carrier_alternatives(&self) -> usize {
        self.carrier_alternatives
    }

    /// Whether an accepted-projection comparison applies here.
    #[must_use]
    pub const fn projection_required(&self) -> bool {
        self.projection_required
    }

    /// What has been observed for this requirement.
    #[must_use]
    pub const fn observation(&self) -> CoverageObservation {
        self.observation
    }
}

/// Every count this package can recompute, and did.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanCensus {
    relations: usize,
    cases: usize,
    relation_cases: usize,
    coverage_requirements: usize,
    positive_requirements: usize,
    negative_requirements: usize,
    negative_target_executable: usize,
    negative_first_party: usize,
    negative_unreachable: usize,
    vacuous_relation_cases: usize,
    matrix_classes: usize,
    semantic_cases: usize,
    materialized_vectors: usize,
    submittable_vectors: usize,
    divergent_vectors: usize,
    sponsored_vectors: usize,
    unbuilt_shape_vectors: usize,
}

impl PlanCensus {
    /// Relations the plan publishes for the operation.
    #[must_use]
    pub const fn relations(&self) -> usize {
        self.relations
    }

    /// Execution cases the plan publishes.
    #[must_use]
    pub const fn cases(&self) -> usize {
        self.cases
    }

    /// Relation-cases, counted across every relation's case map.
    #[must_use]
    pub const fn relation_cases(&self) -> usize {
        self.relation_cases
    }

    /// Coverage requirements the plan publishes.
    #[must_use]
    pub const fn coverage_requirements(&self) -> usize {
        self.coverage_requirements
    }

    /// How many of those are the positive half.
    #[must_use]
    pub const fn positive_requirements(&self) -> usize {
        self.positive_requirements
    }

    /// How many of those are the negative half.
    #[must_use]
    pub const fn negative_requirements(&self) -> usize {
        self.negative_requirements
    }

    /// Negative requirements a live target must refuse.
    ///
    /// The only negative column a run can move. Kept apart from the two
    /// below because they are discharged by different evidence
    /// entirely, and a single "negative" figure would let a wave that
    /// executed nothing look like one that executed everything
    /// first-party.
    #[must_use]
    pub const fn negative_target_executable(&self) -> usize {
        self.negative_target_executable
    }

    /// Negative requirements first-party code refuses before any
    /// transaction exists.
    #[must_use]
    pub const fn negative_first_party(&self) -> usize {
        self.negative_first_party
    }

    /// Negative requirements nothing in this candidate can stage.
    ///
    /// Their evidence is a typed external report that does not exist,
    /// so no run discharges them and the honest accounting is that they
    /// stay unanswered. Counted in their own column for exactly the
    /// reason [`Self::unbuilt_shape_vectors`] is.
    #[must_use]
    pub const fn negative_unreachable(&self) -> usize {
        self.negative_unreachable
    }

    /// Relation-cases present with no obligation.
    #[must_use]
    pub const fn vacuous_relation_cases(&self) -> usize {
        self.vacuous_relation_cases
    }

    /// Named §18 vector classes.
    #[must_use]
    pub const fn matrix_classes(&self) -> usize {
        self.matrix_classes
    }

    /// Positive semantic cases admitted.
    #[must_use]
    pub const fn semantic_cases(&self) -> usize {
        self.semantic_cases
    }

    /// Target vectors materialized to exact bytes.
    ///
    /// Every sponsorless positive case, including one this target
    /// cannot be asked to accept. The bytes exist either way — they are
    /// what the reference cross-checks decode — and whether a target
    /// can be handed them is [`Self::submittable_vectors`].
    #[must_use]
    pub const fn materialized_vectors(&self) -> usize {
        self.materialized_vectors
    }

    /// Materialized vectors this target could be asked to accept.
    ///
    /// The count a live run's submissions is measured against. It is
    /// smaller than [`Self::materialized_vectors`] exactly when the
    /// target's own stated-amount bound forbids a row the protocol's
    /// amount domain admits.
    #[must_use]
    pub const fn submittable_vectors(&self) -> usize {
        self.submittable_vectors
    }

    /// Materialized vectors the target's money bound forbids.
    ///
    /// Counted separately and never folded into either of the two
    /// above, so that no reading of this census can take a documented
    /// divergence for a live acceptance or for a vector that was simply
    /// not run yet.
    #[must_use]
    pub const fn divergent_vectors(&self) -> usize {
        self.divergent_vectors
    }

    /// Sponsored rows a run builds and submits.
    ///
    /// Not counted among the materialized vectors, because this plan
    /// holds no bytes for them: a sponsored row's witness is settled
    /// during the run by an authorization from the executor. They are
    /// work the target does and they are not fixtures this package can
    /// state, and the census keeps the two apart so no reading of it
    /// can take a row built during a run for a byte-stable one.
    #[must_use]
    pub const fn sponsored_vectors(&self) -> usize {
        self.sponsored_vectors
    }

    /// Rows this candidate emitted no program for.
    ///
    /// A limit of the candidate's own shape bounds and never a target
    /// answer: nothing was submitted, so nothing diverged. Counted in
    /// its own column for exactly that reason.
    #[must_use]
    pub const fn unbuilt_shape_vectors(&self) -> usize {
        self.unbuilt_shape_vectors
    }
}

/// One piece of target work this plan needs done.
///
/// # Why the plan names its own workload
///
/// Every coverage row below is outstanding for want of a target, and a
/// plan that could not say *what* it was waiting for would leave the
/// wait unfalsifiable: a wave could execute something, discharge
/// nothing, and no row would be able to tell that it had executed the
/// wrong thing.
///
/// # Why nothing here is an executor record
///
/// The names are this package's own and the vocabulary they are stated
/// in is already authored elsewhere: the ceremony steps are
/// [`FundingCeremonyStep`]'s census, minted in `transaction` where the
/// synthetic origin lives, and a submission is identified by the
/// [`TargetVectorId`] this plan already materialized. Nothing is
/// restated, so there is no second authored spelling of either
/// `(´[PLAN-rule:guide12-exec:typed-source]´)`.
///
/// The executor's own vocabulary — the wire records, the step kinds, the
/// capabilities — lives in the package that owns process supervision and
/// protocol framing, and none of it appears here. Mapping one of these
/// onto one of those is what a Wave-11 plan implementation does, once,
/// in this package `(´[PLAN-rule:guide12-exec:executor-ownership]´)`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum RequiredTargetWork {
    /// One step of the §15.9 test-only funding ceremony.
    ///
    /// The whole census is required, in its stated order. The ceremony
    /// creates the outputs every materialized vector spends, so a wave
    /// that skipped a step would be submitting transactions against
    /// coins that do not exist.
    FundingCeremony(FundingCeremonyStep),
    /// One materialized target vector, handed to the target.
    VectorSubmission(TargetVectorId),
    /// One row this target cannot be asked to accept, and why.
    ///
    /// # Why an impossible submission is still work
    ///
    /// The row's amounts are inside the protocol's domain and outside
    /// this target's, so no ceremony on this target can create the
    /// coins it spends and no submission of it can exist. Dropping it
    /// from the workload would make the plan's census say the wave got
    /// everything it asked for, which is not what happened. Leaving it
    /// as a submission would make the workload permanently unfinishable
    /// with nothing saying why.
    ///
    /// So the item stays, renamed to what is actually required: ask the
    /// target for the one coin the bound forbids and record the answer.
    /// Its discharge criterion is that refusal, and it is *not* the
    /// criterion any other item has — an acceptance here would discharge
    /// nothing and would falsify the reviewed bound instead.
    TargetAmountDivergence(TargetVectorId),
    /// One sponsored row, built during a run and handed to the target.
    ///
    /// # Why it is not a [`Self::VectorSubmission`]
    ///
    /// Because the plan cannot state its bytes. Every other submission
    /// names a vector this plan already materialized, and the bytes
    /// exist here before any run starts. A sponsored row's witness
    /// carries an authorization produced by the executor, so its bytes
    /// are settled during the run and are not a function of the fixture
    /// — two runs against two executors would produce two different
    /// transactions from one row, both correct.
    ///
    /// The work is still required and is named, so the wait stays
    /// falsifiable. What differs is what the plan can promise about it
    /// beforehand, and that difference is in the type rather than in a
    /// note beside it.
    SponsoredVectorSubmission(TargetVectorId),
    /// One row this candidate emitted no program for, and why.
    ///
    /// # A limit of the candidate, not of the target
    ///
    /// §18's matrix names a sponsor region with two members and this
    /// candidate's demonstration bounds admit one, so there is no
    /// program to spend such a family with. The target was never asked
    /// and has diverged from nothing — which is exactly why this is a
    /// separate item from [`Self::TargetAmountDivergence`], whose whole
    /// content is an answer the target gave.
    ///
    /// It has no discharge criterion at all. A run cannot perform it,
    /// and the honest accounting is that the row stays unanswered until
    /// a candidate is linked under bounds that reach it.
    CandidateShapeUnbuilt(TargetVectorId),
}

/// The canonical compact-ASH evidence plan.
///
/// Private fields, no public constructor, no `Default`, no builder: the
/// sole route is [`derive_evidence_plan`], which recomputes every
/// census before admitting anything.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactAshEvidencePlan {
    semantic_cases: Vec<CanonicalSubject<CompactAshSemanticCase>>,
    target_cases: Vec<CanonicalSubject<MaterializedTargetVector>>,
    relation_coverage: BTreeMap<CoverageRequirementId, RelationCoverageRow>,
    required_target_work: Vec<RequiredTargetWork>,
    census: PlanCensus,
    negative_link: BTreeMap<(NegativeMutation, SponsorCase), CoverageRequirementId>,
}

impl CompactAshEvidencePlan {
    /// The admitted semantic cases.
    #[must_use]
    pub fn semantic_cases(&self) -> &[CanonicalSubject<CompactAshSemanticCase>] {
        &self.semantic_cases
    }

    /// The admitted target cases.
    #[must_use]
    pub fn target_cases(&self) -> &[CanonicalSubject<MaterializedTargetVector>] {
        &self.target_cases
    }

    /// The relation-indexed coverage matrix.
    #[must_use]
    pub const fn relation_coverage(&self) -> &BTreeMap<CoverageRequirementId, RelationCoverageRow> {
        &self.relation_coverage
    }

    /// Every count this plan recomputed.
    #[must_use]
    pub const fn census(&self) -> PlanCensus {
        self.census
    }

    /// What a target must do before any coverage row can be discharged.
    ///
    /// The ceremony first, in its own census order, then one submission
    /// per materialized vector in the order the vectors were
    /// materialized. The order is the dependency: the coins have to exist
    /// before anything can spend them.
    #[must_use]
    pub fn required_target_work(&self) -> &[RequiredTargetWork] {
        &self.required_target_work
    }

    /// Whether every coverage requirement is discharged.
    ///
    /// It says so through the rows rather than through a flag someone
    /// could set.
    #[must_use]
    pub fn coverage_complete(&self) -> bool {
        self.relation_coverage
            .values()
            .all(|row| row.observation().is_discharged())
    }

    /// How many rows carry an observation of any kind.
    #[must_use]
    pub fn observed_rows(&self) -> usize {
        self.relation_coverage
            .values()
            .filter(|row| {
                matches!(
                    row.observation(),
                    CoverageObservation::Observed(_) | CoverageObservation::ObservedRefusal(_)
                )
            })
            .count()
    }

    /// How many rows are discharged.
    #[must_use]
    pub fn discharged_rows(&self) -> usize {
        self.relation_coverage
            .values()
            .filter(|row| row.observation().is_discharged())
            .count()
    }

    /// How many positive rows are discharged.
    ///
    /// Reported apart from the negative count because the two halves are
    /// answered by different evidence and reach very different numbers;
    /// one total would let the larger half carry the smaller.
    #[must_use]
    pub fn discharged_positive_rows(&self) -> usize {
        self.relation_coverage
            .values()
            .filter(|row| row.is_positive() && row.observation().is_discharged())
            .count()
    }

    /// How many negative rows are discharged.
    #[must_use]
    pub fn discharged_negative_rows(&self) -> usize {
        self.relation_coverage
            .values()
            .filter(|row| !row.is_positive() && row.observation().is_discharged())
            .count()
    }

    /// Move rows from outstanding to observed, from a run's outcomes.
    ///
    /// # What a submission can and cannot answer
    ///
    /// One accepted sponsorless transaction is evidence about the
    /// relation-cases that transaction actually exercised: the ones
    /// whose execution case is the sponsorless one, whose relation is
    /// active there, and whose evidence role is target execution. It is
    /// evidence about nothing else, and this function refuses to spread
    /// it further:
    ///
    /// - a compiler-static or backend-structural requirement is left
    ///   alone, because §19.1 answers those with structural evidence and
    ///   inventing a target run for them would be the opposite of what
    ///   that rule says;
    /// - an external-report requirement is left alone, because §19.4
    ///   keeps whole-transaction conservation external;
    /// - a negative requirement is left alone, because nothing in this
    ///   wave submits a mutation;
    /// - a requirement whose execution case no outcome in this run
    ///   belongs to is left alone.
    ///
    /// # Why the case is matched rather than assumed
    ///
    /// The outcomes now arrive from both execution cases at once: a run
    /// submits sponsorless rows and sponsored ones, and each is evidence
    /// about its own case only. An accepted sponsorless transaction says
    /// nothing about how the target treats a sponsor region, and an
    /// accepted sponsored one says nothing about the sponsorless shape.
    /// So the outcomes are grouped by the case their vector belongs to,
    /// and each group discharges only rows indexed at that case — which
    /// is the same rule the old code applied to one case, now applied to
    /// both instead of hard-coding which one it was.
    ///
    /// # Nothing is discharged by intent
    ///
    /// Each outcome states a vector, the layer the target put it at, and
    /// what the projection comparison found. All three come off a
    /// transcript, and [`CoverageObservation::is_discharged`] is what
    /// decides whether the triple amounts to coverage — so a row can
    /// carry an observation and still not be discharged, which is
    /// exactly what a rejected or unmatched submission should produce.
    ///
    /// # Why this is crate-private
    ///
    /// "All three come off a transcript" was a description of the one
    /// caller, not a property of the function: nothing in the signature
    /// said so, and every component was a public value a consumer could
    /// state. The property is now carried by the type the public entry
    /// takes — see [`Self::discharge_observed_run`].
    pub(crate) fn discharge(&mut self, outcomes: &[Outcome]) {
        // Which execution case an outcome belongs to is read off the
        // vector's own sponsor count, so a row cannot be filed under a
        // case by anything but what it actually carried.
        let mut by_case: BTreeMap<SponsorCase, Vec<Outcome>> = BTreeMap::new();
        for &outcome in outcomes {
            let case = if outcome.0.sponsors() == 0 {
                SponsorCase::Absent
            } else {
                SponsorCase::Present
            };
            by_case.entry(case).or_default().push(outcome);
        }

        for (case, group) in &by_case {
            let Some(&(vector, layer, projection)) = group.first() else {
                continue;
            };
            // The strongest outcome the group produced, preferring one
            // that actually discharges. A group with one acceptance and
            // eight refusals has established the acceptance; one with
            // none has established a refusal, and the row records that
            // instead.
            let chosen = group
                .iter()
                .copied()
                .find(|&(vector, layer, projection)| {
                    CoverageObservation::Observed(ObservedCoverage {
                        vector,
                        layer,
                        projection,
                    })
                    .is_discharged()
                })
                .unwrap_or((vector, layer, projection));

            for row in self.relation_coverage.values_mut() {
                if !row.positive
                    || row.role != EvidenceRole::TargetExecution
                    || row.activity != RelationActivity::Active
                    || row.key.case.sponsor != *case
                {
                    continue;
                }
                row.observation = CoverageObservation::Observed(ObservedCoverage {
                    vector: chosen.0,
                    layer: chosen.1,
                    projection: chosen.2,
                });
            }
        }
    }

    /// Move negative rows to observed, from a run's mutation outcomes.
    ///
    /// # What a refused mutation can and cannot answer
    ///
    /// §19.2 asks for six things at once, and this checks the three a
    /// run can establish while the plan supplies the rest:
    ///
    /// - the arm must name an intended violated relation, and the link
    ///   the plan resolved is the only route to one — an arm the guide
    ///   does not determine discharges nothing, whatever the target
    ///   said about it;
    /// - the intended carrier must have executed, and the covenant
    ///   script is that carrier, so only a script-path refusal counts. A
    ///   transaction the target threw out before running any script was
    ///   invalid for some reason, and nothing says which relation;
    /// - the observed layer must be the one the class named in advance,
    ///   so a refusal at a boundary nobody predicted is a finding rather
    ///   than a pass.
    ///
    /// The case is read off the vector's own sponsor count, exactly as
    /// the positive half reads it, so a mutation of a sponsorless vector
    /// answers only the sponsorless row.
    ///
    /// # Collateral is recorded, never claimed away
    ///
    /// Every runtime negative requirement carries the policy that the
    /// intended relation and its typed dependency closure must both be
    /// reported blocked, and the closure travels with the requirement
    /// the plan published. Discharging one row therefore says the target
    /// refused a transaction that violates that relation together with
    /// its closure — which is what §19.2 means by a focused mutation
    /// with dependency collateral, and is why no row here is described
    /// as isolated.
    ///
    /// Crate-private for the reason [`Self::discharge`] gives, and the
    /// case here was worse: [`CoverageObservation::is_discharged`]
    /// accepts every [`ObservedRefusal`] unconditionally, on the
    /// strength of the checks above — which are checks about the
    /// *mutation*, not about whether anybody ran it.
    pub(crate) fn discharge_mutants(&mut self, outcomes: &[MutantObservation]) {
        for &(mutation, vector, layer) in outcomes {
            // The class's own boundary, by lookup, so an arm whose
            // matrix row moves takes this with it.
            let Ok(EvidenceBoundary::ScriptPathRejection) = mutation.expected_boundary() else {
                continue;
            };
            if layer != ObservedOutcomeLayer::ScriptPathRejection {
                continue;
            }
            let case = if vector.sponsors() == 0 {
                SponsorCase::Absent
            } else {
                SponsorCase::Present
            };
            let Some(id) = self.negative_link.get(&(mutation, case)) else {
                continue;
            };
            let Some(row) = self.relation_coverage.get_mut(id) else {
                continue;
            };
            // The link only ever resolves negative runtime rows; the
            // guard is here so that a change which broke that would stop
            // rather than file a refusal against a positive row.
            if row.positive {
                continue;
            }
            row.observation = CoverageObservation::ObservedRefusal(ObservedRefusal {
                vector,
                mutation,
                layer,
            });
        }
    }

    /// Discharge this plan's coverage from one validated run.
    ///
    /// # The only public route, and why there is exactly one
    ///
    /// §4.4 asks that coverage be attributable to a supervised target
    /// run. Attributability is not something a call site can promise on
    /// a function's behalf, because the promise is invisible to everyone
    /// reading the function afterwards; it has to be carried by the
    /// argument. So the argument is a
    /// [`ValidatedCompactAshOperationReport`], which exists only where
    /// an executor transcript and a planner transcript were laid beside
    /// each other and agreed — and an executor transcript has no public
    /// constructor at all.
    ///
    /// Both halves are discharged from the one report, positive and
    /// negative together, because they came out of one run and offering
    /// them separately would let half a run be presented as a whole one.
    ///
    /// What the report says the target *decided* is still what decides.
    /// A validated run in which every submission was refused moves no
    /// row to discharged; it moves rows to observed and the counters say
    /// what happened.
    pub fn discharge_observed_run(&mut self, report: &ValidatedCompactAshOperationReport<'_>) {
        self.discharge(report.outcomes());
        self.discharge_mutants(report.mutant_observations());
    }
}

const fn outstanding_reason(requirement: &TargetCoverageRequirement) -> OutstandingReason {
    match &requirement.role {
        EvidenceRole::TargetExecution | EvidenceRole::InactiveCaseAcceptance => {
            OutstandingReason::NoTargetHasExecuted
        }
        EvidenceRole::ExternalReport { .. } => OutstandingReason::ExternalReportAbsent,
        EvidenceRole::CompilerAnalysisResult | EvidenceRole::EmittedStructure => {
            OutstandingReason::StructuralEvidenceNotAssembled
        }
    }
}

/// The coverage matrix and the tallies taken while building it.
struct CoverageBuild {
    rows: BTreeMap<CoverageRequirementId, RelationCoverageRow>,
    covered: BTreeSet<RelationId>,
    positive: usize,
    negative: usize,
    target_executable: usize,
    first_party: usize,
    unreachable: usize,
}

/// Build one row per published coverage requirement.
fn build_coverage(
    plan: &compiler::operation_plan::ValidatedTargetOperationPlan,
) -> Result<CoverageBuild, VectorError> {
    let mut rows = BTreeMap::new();
    let mut covered = BTreeSet::new();
    let mut positive = 0_usize;
    let mut negative = 0_usize;
    let mut published = 0_usize;
    let mut target_executable = 0_usize;
    let mut first_party = 0_usize;
    let mut unreachable = 0_usize;

    for requirement in plan.coverage() {
        published += 1;
        let key = requirement.id.key();
        covered.insert(key.relation.clone());

        let mut observability = None;
        let is_positive = match &requirement.obligation {
            TargetCoverageObligation::Positive(_) => {
                positive += 1;
                true
            }
            TargetCoverageObligation::Negative(_) => {
                negative += 1;
                // A negative requirement whose role names no arm is a
                // requirement this package cannot honestly place, and
                // refusing is the only answer that does not invent a
                // column for it.
                let kind = negative_observability(&requirement.role)
                    .ok_or(VectorError::UnclassifiableNegativeRequirement)?;
                // Matched rather than indexed, so an arm added later
                // has to be given a column here before this compiles.
                match kind {
                    NegativeObservability::TargetExecutable => target_executable += 1,
                    NegativeObservability::FirstPartyRefusal => first_party += 1,
                    NegativeObservability::UnreachableInCandidate => unreachable += 1,
                }
                observability = Some(kind);
                false
            }
        };

        let row = RelationCoverageRow {
            key,
            activity: requirement.activity,
            role: requirement.role.clone(),
            positive: is_positive,
            observability,
            carrier_alternatives: requirement.carrier.len(),
            projection_required: requirement.projection.is_some(),
            observation: CoverageObservation::Outstanding(outstanding_reason(requirement)),
        };
        if rows.insert(requirement.id.clone(), row).is_some() {
            return Err(VectorError::DuplicateCoverageRequirement);
        }
    }

    // The map's size and the number published are two routes to one
    // count; a silent overwrite would separate them.
    if rows.len() != published {
        return Err(VectorError::DuplicateCoverageRequirement);
    }

    Ok(CoverageBuild {
        rows,
        covered,
        positive,
        negative,
        target_executable,
        first_party,
        unreachable,
    })
}

/// Everything a target must do, in dependency order.
///
/// The ceremony's whole census first, then one item per positive row.
/// Derived rather than stated: a list written out by hand could name a
/// vector the plan does not hold, or miss one it does.
///
/// The four kinds are what the plan can honestly promise about each
/// row. A materialized row is a submission or, where the reviewed
/// target bound forbids its coins, a divergence probe. A row this plan
/// holds no bytes for is a sponsored submission where the candidate has
/// a program for its shape, and unbuilt where it does not — one is work
/// a run performs, the other work nothing can perform, and folding them
/// together would make a plan that cannot finish look like one that
/// merely has not.
fn derive_required_work(
    semantic: &[CompactAshSemanticCase],
    target_cases: &[CanonicalSubject<MaterializedTargetVector>],
    divergent: &BTreeSet<TargetVectorId>,
) -> Vec<RequiredTargetWork> {
    let mut work: Vec<RequiredTargetWork> = FundingCeremonyStep::ALL
        .iter()
        .copied()
        .map(RequiredTargetWork::FundingCeremony)
        .collect();
    work.extend(target_cases.iter().map(|subject| {
        let id = subject.subject().id();
        if divergent.contains(&id) {
            RequiredTargetWork::TargetAmountDivergence(id)
        } else {
            RequiredTargetWork::VectorSubmission(id)
        }
    }));
    for case in semantic.iter().filter(|case| needs_authorization(case)) {
        let id = vector_id(case);
        if has_candidate_program(case) {
            work.push(RequiredTargetWork::SponsoredVectorSubmission(id));
        } else {
            work.push(RequiredTargetWork::CandidateShapeUnbuilt(id));
        }
    }
    work
}

/// §1.3, checked in both directions: no relation disappears at this
/// package boundary, and none appears that the plan never published.
fn check_relation_closure(
    planned: &BTreeSet<RelationId>,
    covered: &BTreeSet<RelationId>,
) -> Result<(), VectorError> {
    for relation in planned {
        if !covered.contains(relation) {
            return Err(VectorError::MissingRelation(relation.clone()));
        }
    }
    for relation in covered {
        if !planned.contains(relation) {
            return Err(VectorError::UnexpectedRelation(relation.clone()));
        }
    }
    Ok(())
}

/// Derive the canonical evidence plan from the fixture bundle.
///
/// # Errors
///
/// [`VectorError::RelationCensusMismatch`],
/// [`VectorError::CaseCensusMismatch`], or
/// [`VectorError::RelationCaseCensusMismatch`] when a census recomputed
/// two ways disagrees; [`VectorError::MissingRelation`] when a planned
/// relation has no coverage requirement and
/// [`VectorError::UnexpectedRelation`] when a covered relation is not
/// planned; [`VectorError::DuplicateCoverageRequirement`] when two
/// requirements claim one identity; and any refusal from fixture
/// construction or materialization.
pub fn derive_evidence_plan(
    fixture: &FixtureBundle,
) -> Result<CompactAshEvidencePlan, VectorError> {
    let plan = fixture.plan();

    let relations: Vec<_> = plan.relations().collect();
    let cases: Vec<_> = plan.cases().collect();
    let relation_ids: BTreeSet<RelationId> = relations
        .iter()
        .map(|requirement| requirement.relation.clone())
        .collect();

    // Two independent routes to the relation count: the published
    // iterator, and the deduplicated identity set. A relation published
    // twice would make the first larger than the second.
    if relations.len() != relation_ids.len() {
        return Err(VectorError::RelationCensusMismatch {
            derived: relations.len(),
            recomputed: relation_ids.len(),
        });
    }

    // Two independent routes to the relation-case count: the sum over
    // each relation's own case map, and the product with the case
    // census. The product only equals the sum because every relation is
    // present in every case, active or vacuous — so a relation that
    // dropped a case would separate them.
    let summed: usize = relations
        .iter()
        .map(|requirement| requirement.cases.len())
        .sum();
    let product = relations.len() * cases.len();
    if summed != product {
        return Err(VectorError::RelationCaseCensusMismatch {
            derived: summed,
            recomputed: product,
        });
    }

    let case_ids: BTreeSet<_> = cases.iter().map(|case| case.id.clone()).collect();
    if case_ids.len() != cases.len() {
        return Err(VectorError::CaseCensusMismatch {
            derived: cases.len(),
            recomputed: case_ids.len(),
        });
    }

    let vacuous = relations
        .iter()
        .flat_map(|requirement| requirement.cases.values())
        .filter(|row| row.activity == RelationActivity::Vacuous)
        .count();

    let coverage = build_coverage(plan)?;
    check_relation_closure(&relation_ids, &coverage.covered)?;

    let relation_coverage = coverage.rows;
    let positive = coverage.positive;
    let negative = coverage.negative;

    let semantic = positive_semantic_census()?;
    let mut seen = BTreeSet::new();
    for case in &semantic {
        if !seen.insert(case.id()) {
            return Err(VectorError::DuplicateSemanticFixture(case.id()));
        }
    }

    // The canonical plan materializes against placeholder funding, and
    // says so in the name. Its vectors carry exact bytes for the
    // reference cross-checks and name coins no chain created; the
    // executed plan is built from a ceremony's own answers instead.
    let mut target_cases = Vec::new();
    // Which of them this target could be asked to accept. Derived from
    // the reviewed target bound rather than marked on a row, so the
    // classification changes when the target does and not when somebody
    // remembers to edit a list.
    let mut divergent = BTreeSet::new();
    for case in semantic.iter().filter(|case| is_materializable(case)) {
        let id = vector_id(case);
        if target_amount_standing(case).is_unfundable() {
            divergent.insert(id);
        }
        let funding = AshFunding::unexecutable_placeholder(id);
        target_cases.push(CanonicalSubject::admit(materialize(
            fixture, case, &funding,
        )?));
    }

    let required_target_work = derive_required_work(&semantic, &target_cases, &divergent);

    let census = PlanCensus {
        relations: relations.len(),
        cases: cases.len(),
        relation_cases: summed,
        coverage_requirements: relation_coverage.len(),
        positive_requirements: positive,
        negative_requirements: negative,
        negative_target_executable: coverage.target_executable,
        negative_first_party: coverage.first_party,
        negative_unreachable: coverage.unreachable,
        vacuous_relation_cases: vacuous,
        matrix_classes: class_count(),
        semantic_cases: semantic.len(),
        materialized_vectors: target_cases.len(),
        // Counted off the work items rather than off the set above, so
        // the census and the workload cannot disagree about which rows
        // a target will actually be asked for.
        submittable_vectors: required_target_work
            .iter()
            .filter(|work| matches!(work, RequiredTargetWork::VectorSubmission(_)))
            .count(),
        divergent_vectors: required_target_work
            .iter()
            .filter(|work| matches!(work, RequiredTargetWork::TargetAmountDivergence(_)))
            .count(),
        sponsored_vectors: required_target_work
            .iter()
            .filter(|work| matches!(work, RequiredTargetWork::SponsoredVectorSubmission(_)))
            .count(),
        unbuilt_shape_vectors: required_target_work
            .iter()
            .filter(|work| matches!(work, RequiredTargetWork::CandidateShapeUnbuilt(_)))
            .count(),
    };

    Ok(CompactAshEvidencePlan {
        semantic_cases: semantic.into_iter().map(CanonicalSubject::admit).collect(),
        target_cases,
        relation_coverage,
        required_target_work,
        census,
        negative_link: resolve_negative_link(plan)?,
    })
}

/// Which requirement each mutation arm can answer, per execution case.
///
/// Resolved once, from the published plan, so that every later discharge
/// reads a link the plan itself produced. An arm the guide does not
/// determine contributes no entry rather than a placeholder one, which
/// is why an unlinked arm cannot discharge anything by accident.
///
/// # Errors
///
/// Whatever [`matching_requirement`] refuses: a declared violation that
/// matches no published requirement, or more than one.
fn resolve_negative_link(
    plan: &compiler::operation_plan::ValidatedTargetOperationPlan,
) -> Result<BTreeMap<(NegativeMutation, SponsorCase), CoverageRequirementId>, VectorError> {
    let mut link = BTreeMap::new();
    for &arm in NegativeMutation::ALL {
        let violation = arm.intended_violation();
        for case in [SponsorCase::Absent, SponsorCase::Present] {
            if let Some(id) = matching_requirement(plan, &violation, case)? {
                link.insert((arm, case), id);
            }
        }
    }
    Ok(link)
}

#[cfg(test)]
mod tests {
    use super::{CoverageObservation, OutstandingReason, derive_evidence_plan};
    use crate::bundle::fixture_bundle;
    use crate::subject::SubjectStanding;
    use std::collections::BTreeSet;

    fn plan() -> super::CompactAshEvidencePlan {
        let fixture = fixture_bundle().expect("the fixture bundle builds");
        derive_evidence_plan(&fixture).expect("the evidence plan derives")
    }

    #[test]
    fn the_relation_and_case_censuses_are_the_reviewed_ones() {
        // The guide's compact-ASH figures. Asserted here against numbers
        // this package recomputed from the plan, so a drift in either
        // the compiler or the guide surfaces as a failure rather than as
        // a quietly smaller matrix.
        let census = plan().census();
        assert_eq!(census.relations(), 23);
        assert_eq!(census.cases(), 2);
        assert_eq!(census.relation_cases(), 46);
        assert_eq!(
            census.relation_cases(),
            census.relations() * census.cases(),
            "the product identity holds only while every relation is present in every case"
        );
    }

    #[test]
    fn four_relation_cases_are_vacuous_and_stay_in_the_census() {
        // §19.3: an inactive relation retains explicit inactive-valid
        // coverage. Dropping it would be indistinguishable from never
        // having known about it.
        assert_eq!(plan().census().vacuous_relation_cases(), 4);
    }

    #[test]
    fn every_coverage_requirement_has_exactly_one_row() {
        let plan = plan();
        assert_eq!(
            plan.relation_coverage().len(),
            plan.census().coverage_requirements()
        );
        assert_eq!(
            plan.census().positive_requirements() + plan.census().negative_requirements(),
            plan.census().coverage_requirements(),
            "every requirement is one half or the other"
        );
        // The exact figure, pinned. §1.11 asks for exact finite sets;
        // a `> 0` assertion here would pass just as happily on a census
        // that had quietly lost two hundred rows.
        assert_eq!(plan.census().coverage_requirements(), 211);
        assert_eq!(plan.census().positive_requirements(), 139);
        assert_eq!(plan.census().negative_requirements(), 72);
    }

    #[test]
    fn the_negative_half_splits_into_three_kinds_that_sum_to_it() {
        // The Wave-13 classification, pinned. Forty-eight requirements
        // a live target must refuse, eighteen first-party code refuses
        // before any transaction exists, and six whose evidence is an
        // external report nobody has written.
        //
        // The sum is asserted against the negative total rather than
        // against 72 alone, so a requirement that gained a new kind of
        // role fails here instead of quietly leaving one column short.
        let census = plan().census();
        assert_eq!(census.negative_target_executable(), 48);
        assert_eq!(census.negative_first_party(), 18);
        assert_eq!(census.negative_unreachable(), 6);
        assert_eq!(
            census.negative_target_executable()
                + census.negative_first_party()
                + census.negative_unreachable(),
            census.negative_requirements(),
            "a negative requirement is in exactly one column",
        );
    }

    #[test]
    fn only_negative_rows_are_placed_and_every_one_of_them_is() {
        // The classification is a property of the rows, not only of the
        // tallies taken while building them: a census computed from a
        // counter could agree with itself while the rows disagreed.
        let plan = plan();
        let mut placed = 0_usize;
        for row in plan.relation_coverage().values() {
            if row.is_positive() {
                assert!(
                    row.observability().is_none(),
                    "a positive row was given a refusal to place",
                );
            } else {
                assert!(
                    row.observability().is_some(),
                    "a negative row carries no place its refusal could be observed",
                );
                placed += 1;
            }
        }
        assert_eq!(placed, plan.census().negative_requirements());
    }

    #[test]
    fn a_negative_rows_kind_agrees_with_the_role_that_answers_it() {
        // Recomputed from each row's own evidence role by a second
        // route, so this is a comparison rather than a restatement of
        // the function under test.
        use compiler::operation_plan::EvidenceRole;
        for row in plan().relation_coverage().values() {
            let Some(kind) = row.observability() else {
                continue;
            };
            let expected = match row.role() {
                EvidenceRole::TargetExecution => super::NegativeObservability::TargetExecutable,
                EvidenceRole::CompilerAnalysisResult | EvidenceRole::EmittedStructure => {
                    super::NegativeObservability::FirstPartyRefusal
                }
                EvidenceRole::ExternalReport { .. } => {
                    super::NegativeObservability::UnreachableInCandidate
                }
                EvidenceRole::InactiveCaseAcceptance => {
                    panic!("a vacuous role answered a negative row")
                }
            };
            assert_eq!(
                kind,
                expected,
                "{:?} was placed in the wrong column",
                row.key()
            );
        }
    }

    #[test]
    fn no_negative_row_is_discharged_before_anything_refuses() {
        // The negative half starts where the positive half started: at
        // nothing. A run is what moves it.
        for row in plan().relation_coverage().values() {
            if !row.is_positive() {
                assert!(
                    !row.observation().is_discharged(),
                    "a negative row was discharged by deriving the plan",
                );
            }
        }
    }

    #[test]
    fn a_freshly_derived_plan_discharges_nothing() {
        // Deriving a plan is not executing one. Every row starts
        // outstanding, and only a transcript moves one.
        let plan = plan();
        assert!(!plan.coverage_complete());
        for row in plan.relation_coverage().values() {
            assert!(!row.observation().is_discharged());
        }
    }

    #[test]
    fn acceptance_alone_does_not_discharge_a_row() {
        // §1.4's two verdicts, as a property of the type rather than of
        // a convention: an accepted transaction whose projection was not
        // compared, or compared and differed, discharges nothing.
        use super::{ObservedCoverage, ProjectionComparison};
        use crate::materialize::vector_id;
        use target_elements_conformance::protocol::ObservedOutcomeLayer;

        let case = crate::fixture::positive_semantic_census()
            .expect("the positive census builds")
            .into_iter()
            .find(crate::materialize::is_materializable)
            .expect("a sponsorless case exists");
        let vector = vector_id(&case);

        for (layer, projection, discharged) in [
            (
                ObservedOutcomeLayer::Accepted,
                ProjectionComparison::Matched,
                true,
            ),
            (
                ObservedOutcomeLayer::Accepted,
                ProjectionComparison::Differed,
                false,
            ),
            (
                ObservedOutcomeLayer::Accepted,
                ProjectionComparison::NotPerformed,
                false,
            ),
            (
                ObservedOutcomeLayer::ScriptPathRejection,
                ProjectionComparison::NotPerformed,
                false,
            ),
            (
                ObservedOutcomeLayer::ExecutorInfrastructureFailure,
                ProjectionComparison::NotPerformed,
                false,
            ),
        ] {
            let observation = CoverageObservation::Observed(ObservedCoverage {
                vector,
                layer,
                projection,
            });
            assert_eq!(
                observation.is_discharged(),
                discharged,
                "{layer:?} with {projection:?} discharged the wrong way"
            );
        }
    }

    #[test]
    fn every_outstanding_reason_is_actually_reached() {
        // A reason no row ever carries would be decoration. This records
        // which of the three the current plan actually produces.
        let plan = plan();
        let reasons: BTreeSet<OutstandingReason> = plan
            .relation_coverage()
            .values()
            .filter_map(|row| match row.observation() {
                CoverageObservation::Outstanding(reason) => Some(reason),
                CoverageObservation::Observed(_) | CoverageObservation::ObservedRefusal(_) => None,
            })
            .collect();
        assert!(
            reasons.contains(&OutstandingReason::NoTargetHasExecuted),
            "no requirement is waiting on a target, which cannot be right"
        );
        assert!(!reasons.is_empty());
    }

    #[test]
    fn the_plan_admits_the_positive_census_and_the_bytes_it_materialized() {
        let plan = plan();
        assert_eq!(plan.census().semantic_cases(), 14);
        // Nine of the fourteen positive classes are sponsorless and
        // therefore materializable without a signing capability; the
        // five sponsored ones are admitted as semantic cases and left
        // unmaterialized rather than approximated.
        assert_eq!(plan.census().materialized_vectors(), 9);
        assert_eq!(plan.semantic_cases().len(), 14);
        assert_eq!(plan.target_cases().len(), 9);
        assert_eq!(plan.census().matrix_classes(), 153);
    }

    #[test]
    fn one_materialized_vector_is_beyond_this_targets_money_bound() {
        // The exact accounting, pinned. Eight of the nine sponsorless
        // vectors can be handed to this target; the ninth states an
        // amount the target cannot encode, and is counted in its own
        // column rather than folded into either of the others. A reader
        // adding up this census cannot reach nine live submissions.
        let plan = plan();
        let census = plan.census();
        assert_eq!(census.submittable_vectors(), 8);
        assert_eq!(census.divergent_vectors(), 1);
        assert_eq!(
            census.submittable_vectors() + census.divergent_vectors(),
            census.materialized_vectors(),
            "a materialized vector is one or the other and never both or neither"
        );

        // And it is the row §18.1 asks for at the protocol's own
        // ceiling, not some arbitrary one.
        let divergent: Vec<_> = plan
            .required_target_work()
            .iter()
            .filter_map(|work| match work {
                super::RequiredTargetWork::TargetAmountDivergence(id) => Some(*id),
                _ => None,
            })
            .collect();
        assert_eq!(divergent.len(), 1);
        assert_eq!(
            divergent[0].fixture().name(),
            "values-summing-to-two-pow-51-minus-one"
        );
    }

    #[test]
    fn everything_the_plan_admits_carries_canonical_standing() {
        let plan = plan();
        for subject in plan.semantic_cases() {
            assert_eq!(subject.standing(), SubjectStanding::Canonical);
        }
        for subject in plan.target_cases() {
            assert_eq!(subject.standing(), SubjectStanding::Canonical);
            assert_ne!(subject.subject().bytes(), [] as [u8; 0]);
        }
    }

    #[test]
    fn the_plan_is_the_same_plan_every_time() {
        assert_eq!(plan(), plan());
    }

    #[test]
    fn the_plan_names_the_target_work_it_is_waiting_for() {
        // §16.2, from this side: the wait is falsifiable. The whole
        // ceremony census plus one submission per materialized vector,
        // and the exact figure pinned so a wave that quietly lost a
        // vector fails here rather than discharging fewer rows.
        let plan = plan();
        let work = plan.required_target_work();
        // The ceremony, then one item per positive row: nine this plan
        // materialized to bytes, four sponsored rows a run builds, and
        // one the candidate emitted no program for.
        assert_eq!(work.len(), 18);
        assert_eq!(
            work.len(),
            super::FundingCeremonyStep::ALL.len()
                + plan.census().materialized_vectors()
                + plan.census().sponsored_vectors()
                + plan.census().unbuilt_shape_vectors(),
        );
        assert_eq!(plan.census().sponsored_vectors(), 4);
        assert_eq!(plan.census().unbuilt_shape_vectors(), 1);
    }

    #[test]
    fn the_row_with_two_sponsor_members_has_no_program_in_this_candidate() {
        // §18's matrix names a sponsor region of two and this
        // candidate's demonstration bounds admit one, so the row is
        // unbuilt. It is counted in its own column and never as a
        // divergence: nothing was submitted, so the target has not
        // disagreed with anything.
        let plan = plan();
        let unbuilt: Vec<_> = plan
            .required_target_work()
            .iter()
            .filter_map(|work| match work {
                super::RequiredTargetWork::CandidateShapeUnbuilt(id) => Some(*id),
                _ => None,
            })
            .collect();
        assert_eq!(unbuilt.len(), 1);
        assert_eq!(unbuilt[0].sponsors(), 2);

        // Recomputed from the candidate's own bounds rather than
        // restated: the row is refused because the shape is above the
        // bound, and every other sponsored row is admitted.
        let census = crate::fixture::positive_semantic_census().expect("the census builds");
        for case in &census {
            let admitted = crate::materialize::has_candidate_program(case);
            assert_eq!(
                admitted,
                case.sponsor().members() <= 1,
                "{:?} was classified against the wrong bound",
                case.id()
            );
        }
    }

    #[test]
    fn every_ceremony_step_and_every_vector_is_named_exactly_once() {
        // A step missing from the plan would be a step nobody performs,
        // and a step named twice would be one performed twice against a
        // chain that has already moved on.
        let plan = plan();
        let ceremony: Vec<_> = plan
            .required_target_work()
            .iter()
            .filter_map(|work| match work {
                super::RequiredTargetWork::FundingCeremony(step) => Some(*step),
                super::RequiredTargetWork::VectorSubmission(_)
                | super::RequiredTargetWork::TargetAmountDivergence(_)
                | super::RequiredTargetWork::SponsoredVectorSubmission(_)
                | super::RequiredTargetWork::CandidateShapeUnbuilt(_) => None,
            })
            .collect();
        assert_eq!(ceremony, super::FundingCeremonyStep::ALL.to_vec());

        // The items this plan holds bytes for, which is exactly what it
        // materialized.
        let per_vector: Vec<_> = plan
            .required_target_work()
            .iter()
            .filter_map(|work| match work {
                super::RequiredTargetWork::VectorSubmission(id)
                | super::RequiredTargetWork::TargetAmountDivergence(id) => Some(*id),
                super::RequiredTargetWork::FundingCeremony(_)
                | super::RequiredTargetWork::SponsoredVectorSubmission(_)
                | super::RequiredTargetWork::CandidateShapeUnbuilt(_) => None,
            })
            .collect();
        let materialized: Vec<_> = plan
            .target_cases()
            .iter()
            .map(|subject| subject.subject().id())
            .collect();
        assert_eq!(per_vector, materialized);

        // And every item naming a vector names a distinct one, across
        // all four kinds. Restricting this to the materialized ones
        // would let a sponsored row share an identity with a
        // sponsorless one and nothing would notice.
        let every: Vec<_> = plan
            .required_target_work()
            .iter()
            .filter_map(|work| match work {
                super::RequiredTargetWork::VectorSubmission(id)
                | super::RequiredTargetWork::TargetAmountDivergence(id)
                | super::RequiredTargetWork::SponsoredVectorSubmission(id)
                | super::RequiredTargetWork::CandidateShapeUnbuilt(id) => Some(*id),
                super::RequiredTargetWork::FundingCeremony(_) => None,
            })
            .collect();
        assert_eq!(
            every.iter().copied().collect::<BTreeSet<_>>().len(),
            every.len(),
            "a vector named twice would be submitted twice",
        );
        assert_eq!(
            every.len(),
            plan.census().semantic_cases(),
            "every positive row is named by exactly one work item",
        );
    }

    #[test]
    fn each_execution_case_is_discharged_only_by_its_own_outcomes() {
        // The rule the sponsored half turns on. An accepted sponsorless
        // transaction is evidence about the sponsorless case and about
        // nothing else, so discharging one case must leave the other
        // exactly where it was.
        use super::{ProjectionComparison, RequiredTargetWork};
        use crate::materialize::vector_id;
        use compiler::operation_plan::SponsorCase;
        use target_elements_conformance::protocol::ObservedOutcomeLayer;

        let sponsorless = crate::fixture::positive_semantic_census()
            .expect("the census builds")
            .into_iter()
            .find(crate::materialize::is_materializable)
            .expect("a sponsorless row exists");
        let sponsored = crate::fixture::positive_semantic_census()
            .expect("the census builds")
            .into_iter()
            .find(|case| {
                crate::materialize::needs_authorization(case)
                    && crate::materialize::has_candidate_program(case)
            })
            .expect("a sponsored row with a program exists");

        let accepted = |case| {
            (
                vector_id(case),
                ObservedOutcomeLayer::Accepted,
                ProjectionComparison::Matched,
            )
        };

        // Sponsorless only.
        let mut only_sponsorless = plan();
        only_sponsorless.discharge(&[accepted(&sponsorless)]);
        let after_sponsorless = only_sponsorless.discharged_rows();
        assert!(after_sponsorless > 0, "the sponsorless case discharged");
        for row in only_sponsorless.relation_coverage().values() {
            if row.key().case.sponsor == SponsorCase::Present {
                assert!(
                    !row.observation().is_discharged(),
                    "a sponsorless acceptance discharged a sponsored row",
                );
            }
        }

        // Sponsored only, from a fresh plan.
        let mut only_sponsored = plan();
        only_sponsored.discharge(&[accepted(&sponsored)]);
        let after_sponsored = only_sponsored.discharged_rows();
        assert!(after_sponsored > 0, "the sponsored case discharged");
        for row in only_sponsored.relation_coverage().values() {
            if row.key().case.sponsor == SponsorCase::Absent {
                assert!(
                    !row.observation().is_discharged(),
                    "a sponsored acceptance discharged a sponsorless row",
                );
            }
        }

        // Both together discharge the sum of the two, and no more: the
        // cases partition the rows, so overlap would show up here.
        let mut both = plan();
        both.discharge(&[accepted(&sponsorless), accepted(&sponsored)]);
        assert_eq!(
            both.discharged_rows(),
            after_sponsorless + after_sponsored,
            "the two cases are disjoint and together discharge both",
        );

        // And the plan still names the sponsored row as work, so the
        // discharge cannot be read as coming from nowhere.
        let wanted = vector_id(&sponsored);
        assert!(both.required_target_work().iter().any(|work| matches!(
            work,
            RequiredTargetWork::SponsoredVectorSubmission(id) if *id == wanted
        )));
    }

    #[test]
    fn a_sponsored_row_that_was_refused_discharges_nothing() {
        // The two-verdict rule, on the new half. A sponsored submission
        // the target refused is an observation and not a pass, and it
        // must not discharge the case it belongs to.
        use super::{CoverageObservation, ProjectionComparison};
        use crate::materialize::vector_id;
        use target_elements_conformance::protocol::ObservedOutcomeLayer;

        let sponsored = crate::fixture::positive_semantic_census()
            .expect("the census builds")
            .into_iter()
            .find(|case| {
                crate::materialize::needs_authorization(case)
                    && crate::materialize::has_candidate_program(case)
            })
            .expect("a sponsored row with a program exists");

        let mut refused = plan();
        refused.discharge(&[(
            vector_id(&sponsored),
            ObservedOutcomeLayer::ScriptPathRejection,
            ProjectionComparison::NotPerformed,
        )]);
        assert_eq!(refused.discharged_rows(), 0, "a refusal is not coverage");

        // It is still recorded, which is the difference between a row
        // nobody ran and a row the target refused.
        let observed = refused
            .relation_coverage()
            .values()
            .filter(|row| matches!(row.observation(), CoverageObservation::Observed(_)))
            .count();
        assert!(observed > 0, "the refusal was recorded rather than dropped");
    }

    #[test]
    fn the_ceremony_comes_before_anything_that_spends_it() {
        // The order is the dependency, not a presentation choice: the
        // coins have to exist before a transaction can spend them.
        let plan = plan();
        let first_submission = plan
            .required_target_work()
            .iter()
            .position(|work| {
                matches!(
                    work,
                    super::RequiredTargetWork::VectorSubmission(_)
                        | super::RequiredTargetWork::TargetAmountDivergence(_)
                )
            })
            .expect("the plan submits something");
        assert!(
            plan.required_target_work()[..first_submission]
                .iter()
                .all(|work| matches!(work, super::RequiredTargetWork::FundingCeremony(_))),
            "a submission is planned before the ceremony that funds it",
        );
    }
}

#[cfg(test)]
mod negative_discharge_tests {
    use super::derive_evidence_plan;
    use crate::bundle::fixture_bundle;
    use crate::materialize::TargetVectorId;
    use crate::mutation::NegativeMutation;
    use crate::violation::{IntendedViolation, UnlinkedReason};
    use target_elements_conformance::protocol::ObservedOutcomeLayer;

    fn plan() -> super::CompactAshEvidencePlan {
        let fixture = fixture_bundle().expect("the fixture bundle builds");
        derive_evidence_plan(&fixture).expect("the evidence plan derives")
    }

    /// A sponsorless vector this plan holds bytes for.
    fn sponsorless(plan: &super::CompactAshEvidencePlan) -> TargetVectorId {
        plan.target_cases()
            .iter()
            .map(|subject| subject.subject().id())
            .find(|id| id.sponsors() == 0)
            .expect("a sponsorless vector exists")
    }

    fn discharged_negatives(plan: &super::CompactAshEvidencePlan) -> usize {
        plan.relation_coverage()
            .values()
            .filter(|row| !row.is_positive() && row.observation().is_discharged())
            .count()
    }

    #[test]
    fn the_five_script_path_refusals_discharge_exactly_the_one_linked_row() {
        // The run of record refuses five arms at the script path.
        // `successor-one-below-the-sum` is not among them: its own
        // mutation unbalances the closed asset, which Elements checks
        // before any script runs, so its class's boundary is a
        // consensus one and `discharge_mutants` never gets to ask
        // whether it links. Only one of the five names a relation the
        // guide determines, so only one row moves — which is the whole
        // point of resolving the link instead of counting refusals.
        let mut plan = plan();
        let vector = sponsorless(&plan);
        assert_eq!(discharged_negatives(&plan), 0, "nothing starts discharged");

        let refused = [
            NegativeMutation::SplitSuccessorInTwo,
            NegativeMutation::ReverseAshInputOrder,
            NegativeMutation::RedirectSuccessorProgram,
            NegativeMutation::RouteUnitIntoUndeclaredOutput,
            NegativeMutation::ReorderWitnessItems,
        ];
        let outcomes: Vec<_> = refused
            .iter()
            .map(|&arm| (arm, vector, ObservedOutcomeLayer::ScriptPathRejection))
            .collect();
        plan.discharge_mutants(&outcomes);

        assert_eq!(
            discharged_negatives(&plan),
            1,
            "only the arm whose intended violation resolves may discharge",
        );
    }

    #[test]
    fn a_row_that_is_discharged_is_also_counted_as_observed() {
        // The two tallies are reported side by side, and a discharged
        // row that the observed count missed would read as coverage
        // arriving from nowhere.
        let mut plan = plan();
        let vector = sponsorless(&plan);
        let before = plan.observed_rows();
        plan.discharge_mutants(&[(
            NegativeMutation::SplitSuccessorInTwo,
            vector,
            ObservedOutcomeLayer::ScriptPathRejection,
        )]);
        assert_eq!(
            plan.observed_rows(),
            before + 1,
            "a refused mutation is an observation as much as an acceptance",
        );
        assert!(
            plan.observed_rows() >= plan.discharged_rows(),
            "no row may be discharged without being observed",
        );
    }

    #[test]
    fn a_refusal_before_the_script_discharges_nothing() {
        // §19.2 wants the intended carrier to have executed. A target
        // that threw the transaction out before running any script
        // established that it was invalid, not which relation refused
        // it.
        let mut plan = plan();
        let vector = sponsorless(&plan);
        plan.discharge_mutants(&[(
            NegativeMutation::SplitSuccessorInTwo,
            vector,
            ObservedOutcomeLayer::ConsensusRejectionBeforeScript,
        )]);
        assert_eq!(
            discharged_negatives(&plan),
            0,
            "a refusal the carrier never produced is not carrier coverage",
        );
    }

    #[test]
    fn an_accepted_mutation_discharges_nothing() {
        // The arm that matters most: a mutation the target took is a
        // finding about the class, never coverage of the requirement.
        let mut plan = plan();
        let vector = sponsorless(&plan);
        plan.discharge_mutants(&[(
            NegativeMutation::SuccessorOneBelowTheSum,
            vector,
            ObservedOutcomeLayer::Accepted,
        )]);
        assert_eq!(
            discharged_negatives(&plan),
            0,
            "an accepted mutation is a finding, not a discharge",
        );
    }

    #[test]
    fn the_unlinked_arms_stay_unlinked_whatever_the_target_said() {
        // Four arms name no requirement. Feeding each a perfect
        // script-path refusal must still move nothing, because there is
        // no row their refusal is about.
        let mut plan = plan();
        let vector = sponsorless(&plan);
        for &arm in NegativeMutation::ALL {
            if matches!(arm.intended_violation(), IntendedViolation::Declared { .. }) {
                continue;
            }
            plan.discharge_mutants(&[(arm, vector, ObservedOutcomeLayer::ScriptPathRejection)]);
        }
        assert_eq!(
            discharged_negatives(&plan),
            0,
            "an arm with no intended violation cannot discharge a row",
        );
    }

    #[test]
    fn each_unlinked_arm_states_which_kind_of_gap_it_is() {
        // The three reasons are not interchangeable, and the census
        // reports them apart, so this pins which arm carries which.
        let expected = [
            (
                NegativeMutation::ReverseAshInputOrder,
                UnlinkedReason::NoSemanticMutationClass,
            ),
            (
                NegativeMutation::ReorderWitnessItems,
                UnlinkedReason::NoSemanticMutationClass,
            ),
            (
                NegativeMutation::RedirectSuccessorProgram,
                UnlinkedReason::SemanticClassUnderdetermined,
            ),
            (
                NegativeMutation::RouteUnitIntoUndeclaredOutput,
                UnlinkedReason::SemanticClassUnderdetermined,
            ),
            (
                NegativeMutation::ChangeInputSequence,
                UnlinkedReason::BoundaryPrecedesTarget,
            ),
            (
                NegativeMutation::ChangeTransactionVersion,
                UnlinkedReason::BoundaryPrecedesTarget,
            ),
        ];
        for (arm, reason) in expected {
            assert_eq!(
                arm.intended_violation(),
                IntendedViolation::Unlinked(reason),
                "{arm:?} carries another reason",
            );
        }
    }
}
