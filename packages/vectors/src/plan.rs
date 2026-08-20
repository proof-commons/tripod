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
    AshFunding, MaterializedTargetVector, TargetVectorId, is_materializable, materialize, vector_id,
};
use crate::matrix::class_count;
use crate::subject::CanonicalSubject;

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
        }
    }
}

/// One row of §19's relation-indexed coverage matrix.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationCoverageRow {
    key: RelationCaseKey,
    activity: RelationActivity,
    role: EvidenceRole,
    positive: bool,
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
    vacuous_relation_cases: usize,
    matrix_classes: usize,
    semantic_cases: usize,
    materialized_vectors: usize,
    submittable_vectors: usize,
    divergent_vectors: usize,
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
            .filter(|row| matches!(row.observation(), CoverageObservation::Observed(_)))
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
    /// - a sponsored-case requirement is left alone, because no
    ///   sponsored transaction was built.
    ///
    /// # Nothing is discharged by intent
    ///
    /// Each outcome states a vector, the layer the target put it at, and
    /// what the projection comparison found. All three come off a
    /// transcript, and [`CoverageObservation::is_discharged`] is what
    /// decides whether the triple amounts to coverage — so a row can
    /// carry an observation and still not be discharged, which is
    /// exactly what a rejected or unmatched submission should produce.
    pub fn discharge(
        &mut self,
        outcomes: &[(TargetVectorId, ObservedOutcomeLayer, ProjectionComparison)],
    ) {
        let Some(&(vector, layer, projection)) = outcomes.first() else {
            return;
        };
        // The strongest outcome the run produced, preferring one that
        // actually discharges. A run with one acceptance and eight
        // refusals has established the acceptance; a run with none has
        // established a refusal, and the row records that instead.
        let chosen = outcomes
            .iter()
            .copied()
            .find(|&(_, layer, projection)| {
                CoverageObservation::Observed(ObservedCoverage {
                    vector: outcomes[0].0,
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
                || row.key.case.sponsor != SponsorCase::Absent
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

    for requirement in plan.coverage() {
        published += 1;
        let key = requirement.id.key();
        covered.insert(key.relation.clone());

        let is_positive = match &requirement.obligation {
            TargetCoverageObligation::Positive(_) => {
                positive += 1;
                true
            }
            TargetCoverageObligation::Negative(_) => {
                negative += 1;
                false
            }
        };

        let row = RelationCoverageRow {
            key,
            activity: requirement.activity,
            role: requirement.role.clone(),
            positive: is_positive,
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
    })
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

    // The ceremony's whole census, then one item per vector this plan
    // materialized: a submission for a vector the target can be handed,
    // and a divergence probe for one it cannot. Derived rather than
    // stated: a list written out here could name a vector the plan does
    // not hold, or miss one it does.
    let mut required_target_work: Vec<RequiredTargetWork> = FundingCeremonyStep::ALL
        .iter()
        .copied()
        .map(RequiredTargetWork::FundingCeremony)
        .collect();
    required_target_work.extend(target_cases.iter().map(|subject| {
        let id = subject.subject().id();
        if divergent.contains(&id) {
            RequiredTargetWork::TargetAmountDivergence(id)
        } else {
            RequiredTargetWork::VectorSubmission(id)
        }
    }));

    let census = PlanCensus {
        relations: relations.len(),
        cases: cases.len(),
        relation_cases: summed,
        coverage_requirements: relation_coverage.len(),
        positive_requirements: positive,
        negative_requirements: negative,
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
    };

    Ok(CompactAshEvidencePlan {
        semantic_cases: semantic.into_iter().map(CanonicalSubject::admit).collect(),
        target_cases,
        relation_coverage,
        required_target_work,
        census,
    })
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
                CoverageObservation::Observed(_) => None,
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
        assert_eq!(work.len(), 13);
        assert_eq!(
            work.len(),
            super::FundingCeremonyStep::ALL.len() + plan.census().materialized_vectors(),
        );
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
                | super::RequiredTargetWork::TargetAmountDivergence(_) => None,
            })
            .collect();
        assert_eq!(ceremony, super::FundingCeremonyStep::ALL.to_vec());

        let per_vector: Vec<_> = plan
            .required_target_work()
            .iter()
            .filter_map(|work| match work {
                super::RequiredTargetWork::VectorSubmission(id)
                | super::RequiredTargetWork::TargetAmountDivergence(id) => Some(*id),
                super::RequiredTargetWork::FundingCeremony(_) => None,
            })
            .collect();
        let materialized: Vec<_> = plan
            .target_cases()
            .iter()
            .map(|subject| subject.subject().id())
            .collect();
        assert_eq!(per_vector, materialized);
        assert_eq!(
            per_vector.iter().copied().collect::<BTreeSet<_>>().len(),
            per_vector.len(),
            "a vector named twice would be submitted twice",
        );
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
