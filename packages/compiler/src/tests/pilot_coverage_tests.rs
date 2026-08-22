//! End-to-end pilot coverage analysis (Guide-6 §11, §15, §16; Wave 5).
//!
//! Both Phase-1 pilots run the whole pipeline here — feasible proof
//! plans, execution cases, discharge classification, exact placement,
//! layout requirements, relation-indexed coverage, and the typed
//! coverage dependency graph — over the *complete* feasible plan set,
//! and the Guide-6 §15 acceptance matrices are restated as data and
//! compared with what the aggregate produced. A matrix derived from the
//! analysis would only prove the analysis agrees with itself.
//!
//! Everything asserted below is an obligation: coverage says evidence
//! is required, never that evidence exists, and no assertion here
//! claims a target executed anything.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::LazyLock,
};

use architecture::{AssetId, ObjectId, OperationId};
use realization::{
    Relation, RelationDeclaration, RelationId, RelationKind, RelationSubject, RepresentationMode,
    TransactionSide,
};

use super::bound_input;
use crate::{
    capability::CapabilityView,
    carrier::{CarrierQuantification, CarrierRole},
    case::{ExecutionCaseId, SponsorCase},
    coverage::{
        CollateralPolicy, CoverageBoundary, CoveragePurpose, PlanCoverageAnalysis,
        RelationCoveragePlan, RelationMutation, ScopeCoverageAnalysis, analyze_placed_coverage,
        analyze_scope_coverage, coverage_names_sponsor_value, validate_conditional_coverage,
        validate_representation_coverage,
    },
    coverage_graph::{
        CoverageEdge, CoverageGraphProjection, CoverageNodeId, resolve_coverage_dependencies,
    },
    layout::names_sponsor_amount,
    placement::{
        PlacedCarrier, PlacedProofPlans, PlacementSearchLimits, RelationActivity, RelationCaseKey,
        place_feasible_proof_plans,
    },
    proof::{ProofPlanCandidate, enumerate_feasible_plans},
    relation::{CompilerRelationAnalysis, build_relation_graph},
    source::{OperandRole, is_sponsor_amount_operand},
};

/// Generous limits: a truncated pilot search would hide a defect rather
/// than bound one.
fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(10_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
    )
}

/// One scope analyzed from validated input through coverage.
struct Pilot {
    operations: Vec<OperationId>,
    relations: CompilerRelationAnalysis,
    candidates: Vec<ProofPlanCandidate>,
    placed: PlacedProofPlans,
    coverage: ScopeCoverageAnalysis,
}

fn analyze(operations: &[OperationId]) -> Pilot {
    let input = bound_input(operations);
    let relations = crate::relation::build_relation_analysis(&input).expect("relations");
    let candidates = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained)
        .expect("feasible plans")
        .candidates;

    assert_ne!(candidates, [] as [ProofPlanCandidate; 0]);

    let placed =
        place_feasible_proof_plans(&relations, &candidates, limits()).expect("placement analysis");
    let coverage = analyze_scope_coverage(&relations, &placed).expect("scope coverage");

    Pilot {
        operations: operations.to_vec(),
        relations,
        candidates,
        placed,
        coverage,
    }
}

// The pilots are analyzed once and shared: every test below reads the
// same complete analysis, so each scope's placement product is
// enumerated once rather than once per assertion.
static COMPACT_ASH: LazyLock<Pilot> = LazyLock::new(|| analyze(&[OperationId::CompactAsh]));
static TRANSFER_LIVE: LazyLock<Pilot> = LazyLock::new(|| analyze(&[OperationId::TransferLive]));

fn pilots() -> [&'static Pilot; 2] {
    [&COMPACT_ASH, &TRANSFER_LIVE]
}

impl Pilot {
    fn operation(&self) -> OperationId {
        *self.operations.first().expect("one operation")
    }

    /// Every relation the compiler holds in scope.
    fn relation_ids(&self) -> BTreeSet<RelationId> {
        self.relations
            .graph
            .node_weights()
            .map(|node| node.source.id.clone())
            .collect()
    }

    fn declaration(&self, relation: &RelationId) -> RelationDeclaration {
        self.relations
            .graph
            .node_weights()
            .find(|node| &node.source.id == relation)
            .expect("relation is in scope")
            .source
            .clone()
    }

    /// The placed candidate one covered plan was derived from.
    fn placed_plan(
        &self,
        plan: &ProofPlanCandidate,
    ) -> &crate::placement::PlacedProofPlanCandidate {
        self.placed
            .placed
            .iter()
            .find(|candidate| &candidate.proof_plan == plan)
            .expect("every covered plan is placed")
    }

    /// Every relation-case coverage plan across the whole plan set.
    fn plans(&self) -> impl Iterator<Item = &RelationCoveragePlan> {
        self.coverage
            .analyses()
            .flat_map(PlanCoverageAnalysis::plans)
    }

    fn plans_of<'a>(
        &'a self,
        relation: &'a RelationId,
    ) -> impl Iterator<Item = &'a RelationCoveragePlan> {
        self.plans().filter(move |plan| &plan.relation == relation)
    }
}

// --- Guide-6 §15 acceptance matrices, restated as data ---

/// In which cases the matrix expects one relation to be active.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MatrixCases {
    /// Active in every applicable case of its operation.
    All,
    /// Active when the case sponsors, inactive but valid otherwise.
    WhenSponsored,
}

/// One row of the Guide-6 §15 acceptance matrix.
struct MatrixRow {
    relation: RelationId,
    boundaries: BTreeSet<CoverageBoundary>,
    /// The boundaries this row takes where the case holds its
    /// conserved amounts as commitments, when that differs.
    ///
    /// One row uses it. Conservation is discharged by the target rather
    /// than by a carrier when nothing publishes the amounts (§10.6,
    /// §19.4), so the matrix states both answers and the comparison
    /// stays an exact set equality in either case. A row that widened
    /// to a containment check to cover both would stop noticing a plan
    /// that kept the carrier it should have given up.
    committed: Option<BTreeSet<CoverageBoundary>>,
    cases: MatrixCases,
}

impl MatrixRow {
    /// State this row's boundaries under a committed representation.
    fn when_committed(mut self, boundaries: &[CoverageBoundary]) -> Self {
        self.committed = Some(boundaries.iter().copied().collect());
        self
    }

    /// The boundaries this row takes in one case.
    fn boundaries(&self, case: &ExecutionCaseId) -> &BTreeSet<CoverageBoundary> {
        match &self.committed {
            Some(committed)
                if case
                    .representations
                    .values()
                    .any(|mode| *mode == RepresentationMode::PrivateCommitted) =>
            {
                committed
            }
            _ => &self.boundaries,
        }
    }
}

fn row(
    operation: OperationId,
    kind: RelationKind,
    subject: RelationSubject,
    boundaries: &[CoverageBoundary],
    cases: MatrixCases,
) -> MatrixRow {
    MatrixRow {
        relation: RelationId::new(operation, kind, subject),
        boundaries: boundaries.iter().copied().collect(),
        committed: None,
        cases,
    }
}

fn family(side: TransactionSide, object: ObjectId) -> RelationSubject {
    RelationSubject::ObjectFamily { side, object }
}

/// A whole transaction side, which is what a closure relation is about.
const fn whole_side(side: TransactionSide) -> RelationSubject {
    RelationSubject::TransactionSide { side }
}

/// The Guide-6 §15.1 compact-ASH matrix.
///
/// One row per relation, written out rather than folded into a loop: a
/// table that computed its own rows would restate the derivation
/// instead of the guide.
#[allow(clippy::too_many_lines)]
fn compact_ash_matrix() -> Vec<MatrixRow> {
    use CoverageBoundary as At;
    use MatrixCases::{All, WhenSponsored};
    use RelationKind as Kind;

    let operation = OperationId::CompactAsh;
    let at =
        |kind, subject, boundaries: &[At], cases| row(operation, kind, subject, boundaries, cases);
    let runtime = [At::RuntimeCarrier];
    let hybrid = [At::CompilerStatic, At::BackendStructural];

    vec![
        // ASH and sponsor cardinality. The sponsor family is optional,
        // so its cardinality is inactive-valid in the unsponsored case
        // rather than absent from the census.
        at(
            Kind::Cardinality,
            family(TransactionSide::Input, ObjectId::Ash),
            &runtime,
            All,
        ),
        at(
            Kind::Cardinality,
            family(TransactionSide::Output, ObjectId::Ash),
            &runtime,
            All,
        ),
        at(
            Kind::Cardinality,
            family(TransactionSide::Input, ObjectId::PlainLbtc),
            &runtime,
            WhenSponsored,
        ),
        at(
            Kind::Cardinality,
            family(TransactionSide::Output, ObjectId::PlainLbtc),
            &runtime,
            WhenSponsored,
        ),
        // Recognition, protocol and sponsor alike.
        at(
            Kind::Recognition,
            family(TransactionSide::Input, ObjectId::Ash),
            &runtime,
            All,
        ),
        at(
            Kind::Recognition,
            family(TransactionSide::Output, ObjectId::Ash),
            &runtime,
            All,
        ),
        at(
            Kind::Recognition,
            family(TransactionSide::Input, ObjectId::PlainLbtc),
            &runtime,
            WhenSponsored,
        ),
        at(
            Kind::Recognition,
            family(TransactionSide::Output, ObjectId::PlainLbtc),
            &runtime,
            WhenSponsored,
        ),
        // Input and output closure.
        at(
            Kind::AllowedObjectFamilies,
            whole_side(TransactionSide::Input),
            &runtime,
            All,
        ),
        at(
            Kind::AllowedObjectFamilies,
            whole_side(TransactionSide::Output),
            &runtime,
            All,
        ),
        // Ownerless U conservation, canonical delta policy, open-flow
        // policy, the no-root policy, and the transition-certificate
        // only projection policy.
        at(
            Kind::Conservation,
            RelationSubject::Asset { asset: AssetId::U },
            &runtime,
            All,
        )
        .when_committed(&[CoverageBoundary::ExternalEvidence]),
        at(
            Kind::CanonicalDeltaPolicy,
            RelationSubject::Operation,
            &runtime,
            All,
        ),
        at(
            Kind::OpenFlowPolicy,
            RelationSubject::Operation,
            &runtime,
            All,
        ),
        at(Kind::RootPolicy, RelationSubject::Operation, &runtime, All),
        at(
            Kind::ProjectionPolicy,
            RelationSubject::Operation,
            &runtime,
            All,
        ),
        // Sponsor isolation and envelope multiplicity hold in every
        // case: the unsponsored case proves no sponsor region exists.
        at(
            Kind::SponsorIsolation,
            RelationSubject::Sponsor,
            &runtime,
            All,
        ),
        at(
            Kind::SponsorEnvelopeMultiplicity,
            RelationSubject::Sponsor,
            &runtime,
            All,
        ),
        // Public permissionless constructibility is compiler-static;
        // the secret-free path it implies is backend-structural.
        at(
            Kind::Constructibility,
            RelationSubject::Operation,
            &[At::CompilerStatic],
            All,
        ),
        at(
            Kind::Authorization,
            RelationSubject::Operation,
            &[At::BackendStructural],
            All,
        ),
        // Representation selection and encoding; the compact and clear
        // lifecycle exits.
        at(
            Kind::Representation,
            RelationSubject::Representation {
                object: ObjectId::Ash,
            },
            &hybrid,
            All,
        ),
        at(
            Kind::Lifecycle,
            RelationSubject::LifecycleExit {
                object: ObjectId::Ash,
                exit: OperationId::CompactAsh,
            },
            &hybrid,
            All,
        ),
        at(
            Kind::Lifecycle,
            RelationSubject::LifecycleExit {
                object: ObjectId::Ash,
                exit: OperationId::Clear,
            },
            &hybrid,
            All,
        ),
        // L-BTC substrate conservation stays external evidence.
        at(
            Kind::SubstrateConservation,
            RelationSubject::Asset {
                asset: AssetId::Lbtc,
            },
            &[At::ExternalEvidence],
            All,
        ),
    ]
}

/// The Guide-6 §15.2 live-transfer matrix.
#[allow(clippy::too_many_lines)]
fn transfer_live_matrix() -> Vec<MatrixRow> {
    use CoverageBoundary as At;
    use MatrixCases::{All, WhenSponsored};
    use RelationKind as Kind;

    let operation = OperationId::TransferLive;
    let at =
        |kind, subject, boundaries: &[At], cases| row(operation, kind, subject, boundaries, cases);
    let runtime = [At::RuntimeCarrier];
    let hybrid = [At::CompilerStatic, At::BackendStructural];
    let object = ObjectId::ReceiptLive;

    vec![
        at(
            Kind::Cardinality,
            family(TransactionSide::Input, object),
            &runtime,
            All,
        ),
        at(
            Kind::Cardinality,
            family(TransactionSide::Output, object),
            &runtime,
            All,
        ),
        at(
            Kind::Cardinality,
            family(TransactionSide::Input, ObjectId::PlainLbtc),
            &runtime,
            WhenSponsored,
        ),
        at(
            Kind::Cardinality,
            family(TransactionSide::Output, ObjectId::PlainLbtc),
            &runtime,
            WhenSponsored,
        ),
        at(
            Kind::Recognition,
            family(TransactionSide::Input, object),
            &runtime,
            All,
        ),
        at(
            Kind::Recognition,
            family(TransactionSide::Output, object),
            &runtime,
            All,
        ),
        at(
            Kind::Recognition,
            family(TransactionSide::Input, ObjectId::PlainLbtc),
            &runtime,
            WhenSponsored,
        ),
        at(
            Kind::Recognition,
            family(TransactionSide::Output, ObjectId::PlainLbtc),
            &runtime,
            WhenSponsored,
        ),
        // Every-owner authorization, quantified over the consumed
        // live-receipt family.
        at(
            Kind::Authorization,
            family(TransactionSide::Input, object),
            &runtime,
            All,
        ),
        at(
            Kind::AllowedObjectFamilies,
            whole_side(TransactionSide::Input),
            &runtime,
            All,
        ),
        at(
            Kind::AllowedObjectFamilies,
            whole_side(TransactionSide::Output),
            &runtime,
            All,
        ),
        at(
            Kind::Conservation,
            RelationSubject::Asset { asset: AssetId::U },
            &runtime,
            All,
        )
        .when_committed(&[CoverageBoundary::ExternalEvidence]),
        at(
            Kind::CanonicalDeltaPolicy,
            RelationSubject::Operation,
            &runtime,
            All,
        ),
        at(
            Kind::OpenFlowPolicy,
            RelationSubject::Operation,
            &runtime,
            All,
        ),
        at(Kind::RootPolicy, RelationSubject::Operation, &runtime, All),
        at(
            Kind::ProjectionPolicy,
            RelationSubject::Operation,
            &runtime,
            All,
        ),
        at(
            Kind::SponsorIsolation,
            RelationSubject::Sponsor,
            &runtime,
            All,
        ),
        at(
            Kind::SponsorEnvelopeMultiplicity,
            RelationSubject::Sponsor,
            &runtime,
            All,
        ),
        at(
            Kind::Constructibility,
            RelationSubject::Operation,
            &[At::CompilerStatic],
            All,
        ),
        at(
            Kind::Representation,
            RelationSubject::Representation { object },
            &hybrid,
            All,
        ),
        at(
            Kind::Lifecycle,
            RelationSubject::LifecycleExit {
                object,
                exit: OperationId::TransferLive,
            },
            &hybrid,
            All,
        ),
        at(
            Kind::Lifecycle,
            RelationSubject::LifecycleExit {
                object,
                exit: OperationId::Burn,
            },
            &hybrid,
            All,
        ),
        at(
            Kind::Lifecycle,
            RelationSubject::LifecycleExit {
                object,
                exit: OperationId::Redeem,
            },
            &hybrid,
            All,
        ),
        at(
            Kind::SubstrateConservation,
            RelationSubject::Asset {
                asset: AssetId::Lbtc,
            },
            &[At::ExternalEvidence],
            All,
        ),
    ]
}

fn matrix(operation: OperationId) -> Vec<MatrixRow> {
    match operation {
        OperationId::CompactAsh => compact_ash_matrix(),
        OperationId::TransferLive => transfer_live_matrix(),
        other => panic!("{other:?} is not a pilot"),
    }
}

/// The activity the matrix expects of one row in one case.
const fn expected_activity(cases: MatrixCases, case: &ExecutionCaseId) -> RelationActivity {
    match (cases, case.sponsor) {
        (MatrixCases::All, _) | (MatrixCases::WhenSponsored, SponsorCase::Present) => {
            RelationActivity::Active
        }
        (MatrixCases::WhenSponsored, SponsorCase::Absent) => RelationActivity::Vacuous,
    }
}

// --- §11.1 exact scope equality ---

#[test]
fn every_pilot_scope_agrees_from_realization_through_coverage() {
    for pilot in pilots() {
        let operation = pilot.operation();

        // The realization relation census, restricted to the pilot's
        // operation, and the compiler's own scope.
        let realized = bound_input(&[operation])
            .realization()
            .project()
            .relations
            .nodes
            .into_iter()
            .map(|declaration| declaration.id)
            .filter(|relation| relation.operation() == operation)
            .collect::<BTreeSet<_>>();
        let compiled = pilot.relation_ids();

        assert_eq!(realized, compiled, "{operation:?}");
        assert!(!compiled.is_empty());

        for (plan, entry) in &pilot.coverage.plans {
            let placed = pilot.placed_plan(plan);
            let cases = placed
                .execution_cases
                .iter()
                .map(|case| case.id.clone())
                .collect::<BTreeSet<_>>();
            let expected = compiled
                .iter()
                .flat_map(|relation| {
                    cases.iter().map(move |case| RelationCaseKey {
                        relation: relation.clone(),
                        case: case.clone(),
                    })
                })
                .collect::<BTreeSet<_>>();
            let planned = placed
                .relation_case_plans
                .iter()
                .map(|plan| RelationCaseKey {
                    relation: plan.relation.clone(),
                    case: plan.case.clone(),
                })
                .collect::<BTreeSet<_>>();

            // Relation-case plan scope and coverage scope alike.
            assert_eq!(planned, expected, "{operation:?}");
            assert_eq!(entry.coverage.keys(), expected, "{operation:?}");
        }

        // The scope's case census survives aggregation, and every
        // feasible plan is covered.
        assert_eq!(
            pilot.coverage.cases(),
            pilot.placed.execution_case_census(),
            "{operation:?}",
        );
        assert_eq!(pilot.coverage.plans.len(), pilot.candidates.len());
    }
}

// --- §15.1 and §15.2 acceptance matrices over the aggregate ---

#[test]
fn every_pilot_relation_covers_exactly_its_acceptance_matrix_row() {
    for pilot in pilots() {
        let operation = pilot.operation();
        let rows = matrix(operation);

        // The matrix is the complete census, not a sample of it.
        assert_eq!(
            rows.iter()
                .map(|row| row.relation.clone())
                .collect::<BTreeSet<_>>(),
            pilot.relation_ids(),
            "{operation:?}",
        );

        for row in rows {
            let mut cases = BTreeSet::new();

            for plan in pilot.plans_of(&row.relation) {
                cases.insert(plan.case.sponsor);

                assert_eq!(
                    &plan.boundaries,
                    row.boundaries(&plan.case),
                    "{:?} in {operation:?} case {:?}",
                    row.relation,
                    plan.case,
                );
                assert_eq!(
                    plan.activity,
                    expected_activity(row.cases, &plan.case),
                    "{:?} in {operation:?} case {:?}",
                    row.relation,
                    plan.case.sponsor,
                );

                let runtime = plan.boundaries.contains(&CoverageBoundary::RuntimeCarrier);

                match plan.activity {
                    RelationActivity::Active => {
                        // Active acceptance, a focused rejection, and an
                        // accepted projection at every boundary the row
                        // names; a carrier requirement exactly where the
                        // row is runtime-carried.
                        for boundary in &plan.boundaries {
                            assert!(
                                plan.positive
                                    .iter()
                                    .any(|requirement| requirement.id.boundary == *boundary),
                                "{:?} at {boundary:?}",
                                row.relation,
                            );
                            assert!(
                                plan.negative
                                    .iter()
                                    .any(|requirement| requirement.id.boundary == *boundary),
                                "{:?} at {boundary:?}",
                                row.relation,
                            );
                            assert!(
                                plan.projections.contains_key(boundary),
                                "{:?} at {boundary:?}",
                                row.relation,
                            );
                        }

                        assert_eq!(plan.carrier.is_some(), runtime, "{:?}", row.relation);
                    }
                    RelationActivity::Vacuous => {
                        assert!(plan.carrier.is_none(), "{:?}", row.relation);
                        assert!(plan.projections.is_empty(), "{:?}", row.relation);
                        assert!(plan.negative.is_empty(), "{:?}", row.relation);
                        assert!(
                            plan.positive.iter().all(|requirement| {
                                requirement.id.purpose == CoveragePurpose::InactiveAccept
                            }),
                            "{:?}",
                            row.relation,
                        );
                    }
                }
            }

            // Every row is exercised in both sponsor cases.
            assert_eq!(
                cases,
                BTreeSet::from([SponsorCase::Absent, SponsorCase::Present]),
                "{:?} in {operation:?}",
                row.relation,
            );
        }
    }
}

#[test]
fn live_owner_authorization_requires_every_member_and_offers_no_all_owner_shortcut() {
    let pilot: &Pilot = &TRANSFER_LIVE;
    let relation = RelationId::new(
        OperationId::TransferLive,
        RelationKind::Authorization,
        family(TransactionSide::Input, ObjectId::ReceiptLive),
    );
    let expected = BTreeSet::from([PlacedCarrier {
        carrier: CarrierRole::EveryInputFamilyMember {
            object: ObjectId::ReceiptLive,
        },
        quantification: CarrierQuantification::PerMember,
    }]);
    let mut covered = 0_usize;

    for plan in pilot.plans_of(&relation) {
        let carrier = plan
            .carrier
            .as_ref()
            .expect("owner authorization is runtime-carried");
        covered += 1;

        // One alternative, quantified over every consumed owner: no
        // coordinator and no complete-family proof stands in for it.
        assert_eq!(carrier.allowed_assignments.len(), 1);

        for alternative in &carrier.allowed_assignments {
            assert_eq!(alternative.carriers, expected);
            assert!(alternative.carriers.iter().all(|placed| {
                placed.quantification == CarrierQuantification::PerMember
                    && !matches!(placed.carrier, CarrierRole::OperationGlobal { .. })
            }));
        }

        assert!(
            plan.negative
                .iter()
                .any(|requirement| requirement.mutation == RelationMutation::MissingRequiredOwner),
        );
    }

    assert!(covered > 0);
}

// --- §11.2 placement scope equality ---

#[test]
fn carrier_coverage_keys_are_exactly_the_active_runtime_relation_cases() {
    for pilot in pilots() {
        let operation = pilot.operation();

        for (plan, entry) in &pilot.coverage.plans {
            let placed = pilot.placed_plan(plan);
            let carried = placed
                .relation_case_plans
                .iter()
                .filter(|plan| !plan.runtime_requirements.is_empty())
                .map(|plan| RelationCaseKey {
                    relation: plan.relation.clone(),
                    case: plan.case.clone(),
                })
                .collect::<BTreeSet<_>>();
            let covered = entry
                .coverage
                .plans()
                .filter(|plan| plan.carrier.is_some())
                .map(RelationCoveragePlan::key)
                .collect::<BTreeSet<_>>();

            assert_eq!(covered, carried, "{operation:?}");
            assert!(!covered.is_empty());

            // Compiler-static, backend-structural, external, and
            // inactive relation-cases hold no carrier key at all.
            for coverage in entry.coverage.plans() {
                if coverage.carrier.is_some() {
                    assert_eq!(coverage.activity, RelationActivity::Active);
                    assert!(
                        coverage
                            .boundaries
                            .contains(&CoverageBoundary::RuntimeCarrier)
                    );
                }
            }
        }
    }
}

// --- §11.3 layout scope ---

#[test]
fn every_selected_assignment_layout_requirement_is_stated_by_its_plan() {
    for pilot in pilots() {
        let operation = pilot.operation();

        for (plan, entry) in &pilot.coverage.plans {
            let placed = pilot.placed_plan(plan);
            let stated = placed.layout_requirements.iter().collect::<BTreeSet<_>>();
            let mut referenced = BTreeSet::new();

            for coverage in entry.coverage.plans() {
                for alternative in coverage
                    .carrier
                    .iter()
                    .flat_map(|carrier| carrier.allowed_assignments.iter())
                {
                    for requirement in &alternative.layout {
                        assert!(stated.contains(requirement), "{operation:?}");
                        referenced.insert(requirement.clone());
                    }
                }
            }

            assert!(!referenced.is_empty(), "{operation:?}");

            // Every selectable assignment of every feasible placement is
            // covered by an alternative of its own relation-case.
            for placement in &placed.feasible_placements {
                for assignment in &placement.assignments {
                    let key = assignment.key();
                    let carrier = entry
                        .coverage
                        .plan(&key)
                        .and_then(|coverage| coverage.carrier.as_ref())
                        .expect("every placed assignment is covered");
                    let selected = assignment.carriers.iter().cloned().collect::<BTreeSet<_>>();

                    assert!(
                        carrier
                            .allowed_assignments
                            .iter()
                            .any(|alternative| alternative.carriers == selected),
                        "{operation:?} {:?}",
                        assignment.relation,
                    );
                }
            }
        }
    }
}

// --- §11.4 positive, negative, projection, and carrier completeness ---

#[test]
fn active_and_inactive_relation_cases_state_their_required_coverage() {
    for pilot in pilots() {
        let operation = pilot.operation();
        let mut active = 0_usize;
        let mut inactive = 0_usize;

        for plan in pilot.plans() {
            match plan.activity {
                RelationActivity::Active => {
                    active += 1;

                    for boundary in &plan.boundaries {
                        assert!(
                            plan.positive
                                .iter()
                                .any(|requirement| requirement.id.boundary == *boundary),
                            "{operation:?} {:?}",
                            plan.relation,
                        );
                        assert!(
                            plan.negative
                                .iter()
                                .any(|requirement| requirement.id.boundary == *boundary),
                            "{operation:?} {:?}",
                            plan.relation,
                        );
                    }

                    if plan.boundaries.contains(&CoverageBoundary::RuntimeCarrier) {
                        assert!(plan.carrier.is_some());
                        assert!(
                            plan.projections
                                .contains_key(&CoverageBoundary::RuntimeCarrier)
                        );
                    }
                }
                RelationActivity::Vacuous => {
                    inactive += 1;

                    // One inactive-valid requirement per boundary, no
                    // runtime carrier, and no rejection demanded of a
                    // branch this case never exercises.
                    assert_eq!(plan.positive.len(), plan.boundaries.len());
                    assert_eq!(
                        plan.negative,
                        [] as [crate::coverage::NegativeCoverageRequirement; 0]
                    );
                    assert!(plan.carrier.is_none());
                }
            }
        }

        assert!(active > 0, "{operation:?}");
        assert!(inactive > 0, "{operation:?}");
    }
}

// --- §11.5 and §11.6 plan-set censuses ---

#[test]
fn the_conditional_triplet_and_representation_census_hold_across_the_plan_set() {
    for pilot in pilots() {
        let operation = pilot.operation();
        let analyses = pilot
            .coverage
            .analyses()
            .cloned()
            .collect::<Vec<PlanCoverageAnalysis>>();

        validate_conditional_coverage(&analyses).expect("conditional triplet");
        validate_representation_coverage(&pilot.relations, &analyses)
            .expect("representation census");

        // The triplet restated concretely for the sponsor families:
        // inactive valid, active valid, and active invalid.
        for side in [TransactionSide::Input, TransactionSide::Output] {
            for kind in [RelationKind::Cardinality, RelationKind::Recognition] {
                let relation = RelationId::new(operation, kind, family(side, ObjectId::PlainLbtc));
                let mut inactive = false;
                let mut accepted = false;
                let mut rejected = false;

                for plan in pilot.plans_of(&relation) {
                    match plan.activity {
                        RelationActivity::Vacuous => {
                            inactive |= plan.positive.iter().any(|requirement| {
                                requirement.id.purpose == CoveragePurpose::InactiveAccept
                            });
                        }
                        RelationActivity::Active => {
                            accepted |= plan.positive.iter().any(|requirement| {
                                requirement.id.purpose == CoveragePurpose::ActiveAccept
                            });
                            rejected |= !plan.negative.is_empty();
                        }
                    }
                }

                assert!(inactive, "{operation:?} {kind:?} {side:?}");
                assert!(accepted, "{operation:?} {kind:?} {side:?}");
                assert!(rejected, "{operation:?} {kind:?} {side:?}");
            }
        }

        // Each plan fixes one mode, so the plan set — never one plan —
        // covers both allowed modes of the representation relation.
        let representation = pilot
            .relation_ids()
            .into_iter()
            .find(|relation| relation.kind() == RelationKind::Representation)
            .expect("one representation relation");
        let Relation::Representation { allowed, .. } = pilot.declaration(&representation).relation
        else {
            panic!("the representation relation selects a representation");
        };
        let covered = pilot
            .plans_of(&representation)
            .flat_map(|plan| plan.positive.iter())
            .filter_map(|requirement| requirement.representation)
            .collect::<BTreeSet<RepresentationMode>>();

        assert_eq!(covered, allowed, "{operation:?}");
        assert_eq!(covered.len(), 2, "{operation:?}");
    }
}

// --- §15.3 sponsor opacity over the complete aggregate projection ---

#[test]
fn no_pilot_coverage_projection_names_an_erased_sponsor_value() {
    for pilot in pilots() {
        let operation = pilot.operation();
        let mut sponsor_relations = 0_usize;

        for analysis in pilot.coverage.analyses() {
            assert!(!coverage_names_sponsor_value(analysis), "{operation:?}");
        }

        for entry in pilot.coverage.plans.values() {
            for coverage in entry.coverage.plans() {
                let sponsor = matches!(
                    coverage.relation.subject(),
                    RelationSubject::ObjectFamily {
                        object: ObjectId::PlainLbtc,
                        ..
                    } | RelationSubject::Sponsor
                );

                if sponsor {
                    sponsor_relations += 1;

                    // No sponsor amount exists to compare, so no sponsor
                    // row may require an amount mutation: the sponsor
                    // relations require census and membership classes.
                    for requirement in &coverage.negative {
                        assert_ne!(
                            requirement.mutation,
                            RelationMutation::AmountMismatch,
                            "{operation:?} {:?}",
                            coverage.relation,
                        );
                    }
                }

                // No family-amount operand over the sponsor family, in
                // any positive requirement or accepted projection — the
                // public sponsor total included.
                let operands = coverage
                    .positive
                    .iter()
                    .flat_map(|requirement| requirement.operands.iter())
                    .chain(
                        coverage
                            .projections
                            .values()
                            .flat_map(|requirement| requirement.operands.iter()),
                    );

                for operand in operands {
                    assert!(
                        !is_sponsor_amount_operand(operand.role()),
                        "{operation:?} {operand:?}",
                    );
                    assert!(
                        !matches!(
                            operand.role(),
                            OperandRole::ObjectFamilyAmount {
                                object: ObjectId::PlainLbtc,
                                ..
                            }
                        ),
                        "{operation:?} {operand:?}",
                    );
                }

                // No authenticated consensus value stands behind a
                // sponsor amount in any projected source row.
                for source in coverage
                    .projections
                    .values()
                    .flat_map(|requirement| requirement.sources.iter())
                {
                    assert!(
                        !is_sponsor_amount_operand(source.operand.role()),
                        "{operation:?} {source:?}",
                    );
                }

                // Nor through a layout requirement a selectable carrier
                // assignment depends on.
                for alternative in coverage
                    .carrier
                    .iter()
                    .flat_map(|carrier| carrier.allowed_assignments.iter())
                {
                    for requirement in &alternative.layout {
                        assert!(!names_sponsor_amount(requirement), "{operation:?}");
                    }
                }
            }
        }

        // The traversal is not vacuous: both pilots really do cover
        // sponsor relations.
        assert!(sponsor_relations > 0, "{operation:?}");
    }
}

// --- Tranche E: the coverage dependency graph of every pilot plan ---

/// The active relation-case descendants of one symbol, by repeated scan.
///
/// Independent of the production Petgraph closure: the projection's own
/// prerequisite edges are scanned to a fixed point, so a bound closure
/// is compared with one derived from the edges rather than with itself.
fn expected_collateral(
    projection: &CoverageGraphProjection,
    start: &RelationCaseKey,
) -> BTreeSet<RelationCaseKey> {
    let active = projection
        .nodes
        .iter()
        .filter(|node| node.activity == Some(RelationActivity::Active))
        .filter_map(|node| match &node.id {
            CoverageNodeId::RelationCase(key) => Some(key.clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let mut reached = BTreeSet::from([start.clone()]);

    loop {
        let mut grown = false;

        for edge in &projection.edges {
            if edge.edge != CoverageEdge::RelationPrerequisite {
                continue;
            }

            let (CoverageNodeId::RelationCase(source), CoverageNodeId::RelationCase(target)) =
                (&edge.source, &edge.target)
            else {
                continue;
            };

            if reached.contains(source) && !reached.contains(target) {
                reached.insert(target.clone());
                grown = true;
            }
        }

        if !grown {
            break;
        }
    }

    reached.remove(start);
    reached.retain(|key| active.contains(key));
    reached
}

#[test]
fn every_pilot_coverage_graph_is_acyclic_and_binds_its_dependency_collateral() {
    for pilot in pilots() {
        let operation = pilot.operation();
        let mut claimed = 0_usize;

        for entry in pilot.coverage.plans.values() {
            let projection = &entry.dependencies;

            assert!(!projection.nodes.is_empty(), "{operation:?}");
            assert!(!projection.edges.is_empty(), "{operation:?}");

            // The graph's relation-case symbols are exactly the coverage
            // census, and no symbol reaches itself.
            let symbols = projection
                .nodes
                .iter()
                .filter_map(|node| match &node.id {
                    CoverageNodeId::RelationCase(key) => Some(key.clone()),
                    _ => None,
                })
                .collect::<BTreeSet<_>>();
            assert_eq!(symbols, entry.coverage.keys(), "{operation:?}");

            for key in &symbols {
                assert!(
                    !expected_collateral(projection, key).contains(key),
                    "{operation:?} {key:?}",
                );
            }

            for coverage in entry.coverage.plans() {
                let expected = expected_collateral(projection, &coverage.key());

                for requirement in &coverage.negative {
                    match requirement.collateral.policy {
                        CollateralPolicy::RequireIntendedAndDependencyClosure => {
                            assert_eq!(
                                requirement.collateral.dependency_closure, expected,
                                "{operation:?} {:?}",
                                coverage.relation,
                            );

                            if !expected.is_empty() {
                                claimed += 1;
                            }
                        }
                        // Nothing here claims isolation: an unclaimed
                        // closure is reported later, never assumed
                        // absent.
                        CollateralPolicy::ReportAdditional => {
                            assert!(requirement.collateral.dependency_closure.is_empty());
                        }
                    }
                }
            }
        }

        // Some runtime negative really does claim a non-empty closure.
        assert!(claimed > 0, "{operation:?}");
    }
}

#[test]
fn the_coverage_graph_projection_joins_the_aggregate_projection() {
    for pilot in pilots() {
        let projection = pilot.coverage.project();

        assert_eq!(projection.plans.len(), pilot.candidates.len());

        for (plan, entry) in &projection.plans {
            let coverage = &pilot.coverage.plans[plan];

            assert_eq!(entry.coverage, coverage.coverage.project());
            assert_eq!(entry.dependencies, coverage.dependencies);

            // Every dependency endpoint resolves to a symbol the
            // projection also carries, inside one operation.
            let symbols = entry
                .dependencies
                .nodes
                .iter()
                .map(|node| node.id.clone())
                .collect::<BTreeSet<_>>();

            for edge in &entry.dependencies.edges {
                assert!(symbols.contains(&edge.source));
                assert!(symbols.contains(&edge.target));
                assert_eq!(edge.source.operation(), edge.target.operation());
            }
        }
    }
}

// --- §16.3 determinism ---

#[test]
fn repeated_pilot_coverage_analysis_projects_equally() {
    for pilot in pilots() {
        let again =
            analyze_scope_coverage(&pilot.relations, &pilot.placed).expect("second analysis");

        assert_eq!(again, pilot.coverage, "{:?}", pilot.operation());
        assert_eq!(again.project(), pilot.coverage.project());
    }
}

#[test]
fn a_permuted_proof_plan_candidate_order_is_covered_equally() {
    for pilot in pilots() {
        let mut permuted = pilot.candidates.clone();
        permuted.reverse();

        let placed = place_feasible_proof_plans(&pilot.relations, &permuted, limits())
            .expect("permuted placement");
        let again = analyze_scope_coverage(&pilot.relations, &placed).expect("permuted coverage");

        assert_eq!(
            again.project(),
            pilot.coverage.project(),
            "{:?}",
            pilot.operation(),
        );
    }
}

/// The same scope's relations, built from reversed declarations.
fn permuted_relations(operations: &[OperationId]) -> CompilerRelationAnalysis {
    let input = bound_input(operations);
    let source = input.realization().project();
    let mut nodes = source.relations.nodes;
    let mut edges = source.relations.edges;

    nodes.reverse();
    edges.reverse();

    build_relation_graph(input.scope().operations(), &nodes, &edges)
        .expect("permuted relation graph")
}

#[test]
fn permuted_relation_declarations_are_covered_equally() {
    for pilot in pilots() {
        let permuted = permuted_relations(&pilot.operations);
        let placed = place_feasible_proof_plans(&permuted, &pilot.candidates, limits())
            .expect("permuted placement");
        let again = analyze_scope_coverage(&permuted, &placed).expect("permuted coverage");

        assert_eq!(
            again.project(),
            pilot.coverage.project(),
            "{:?}",
            pilot.operation(),
        );
    }
}

#[test]
fn a_permuted_placement_order_is_covered_equally() {
    for pilot in pilots() {
        let mut permuted = pilot.placed.clone();

        for entry in &mut permuted.placed {
            entry.feasible_placements.reverse();
        }

        let again = analyze_scope_coverage(&pilot.relations, &permuted).expect("permuted coverage");

        assert_eq!(
            again.project(),
            pilot.coverage.project(),
            "{:?}",
            pilot.operation(),
        );
    }
}

#[test]
fn a_permuted_relation_case_plan_order_is_covered_equally() {
    for pilot in pilots() {
        for entry in &pilot.placed.placed {
            let mut permuted = entry.clone();
            permuted.relation_case_plans.reverse();

            let mut coverage =
                analyze_placed_coverage(&pilot.relations, &permuted).expect("permuted coverage");
            resolve_coverage_dependencies(&mut coverage, &pilot.relations).expect("dependencies");

            assert_eq!(
                coverage.project(),
                pilot.coverage.plans[&entry.proof_plan].coverage.project(),
                "{:?}",
                pilot.operation(),
            );
        }
    }
}

// --- the combined two-pilot scope ---

/// The combined two-pilot coverage analysis over the full placement
/// product.
///
/// Demoted to the manual lane under Guide-7 §20.3. The replacement
/// signal is in place: `analyzed_program_oracle_tests` restates the
/// relation-case and coverage censuses independently per operation,
/// compares each placement factor with the independent whole-plan route,
/// and proves structurally that no coverage key, carrier, layout
/// requirement, or dependency edge crosses an operation — all without
/// covering the 46,656 combined placements per plan this test analyzes.
///
/// ```sh
/// cargo test -p tripod-compiler \
///   the_combined_pilot_scope_coverage_factorizes_per_operation \
///   -- --ignored --nocapture
/// ```
#[test]
#[ignore = "phase-exit regression; run with -- --ignored"]
fn the_combined_pilot_scope_coverage_factorizes_per_operation() {
    let operations = [OperationId::CompactAsh, OperationId::TransferLive];
    let combined = analyze(&operations);

    // Repeated analysis of the same placed plan set agrees.
    let again =
        analyze_scope_coverage(&combined.relations, &combined.placed).expect("second analysis");
    assert_eq!(again.project(), combined.coverage.project());

    // The combined coverage is stored per operation and never as a
    // cross-operation product: each plan carries one analysis per
    // operation, over that operation's own cases and relations.
    for entry in combined.coverage.plans.values() {
        assert_eq!(
            entry
                .coverage
                .operations
                .keys()
                .copied()
                .collect::<BTreeSet<_>>(),
            BTreeSet::from(operations),
        );

        for (operation, analysis) in &entry.coverage.operations {
            assert!(
                analysis
                    .cases
                    .iter()
                    .all(|case| case.operation == *operation)
            );
            assert!(analysis.requirements.keys().all(|key| {
                key.relation.operation() == *operation && key.case.operation == *operation
            }));
        }
    }

    // Each operation's factors are exactly the factors that operation's
    // own scope produces — compared as sets of per-operation
    // projections, so nothing pairs plans across scopes and no coverage
    // product is materialized on either side.
    for pilot in pilots() {
        let operation = pilot.operation();

        assert_eq!(
            combined.coverage.operation_projections(operation),
            pilot.coverage.operation_projections(operation),
            "{operation:?}",
        );
    }

    // The combined case census is the union of the two operations'.
    let mut expected = BTreeSet::new();

    for pilot in pilots() {
        expected.extend(pilot.coverage.cases());
    }

    assert_eq!(combined.coverage.cases(), expected);
    assert_eq!(expected.len(), 8);

    // The relation-case census is the sum of the factors', never their
    // product.
    let factors = pilots()
        .iter()
        .map(|pilot| {
            pilot
                .coverage
                .analyses()
                .map(|analysis| analysis.keys().len())
                .max()
                .expect("a covered plan")
        })
        .sum::<usize>();

    for entry in combined.coverage.plans.values() {
        assert_eq!(entry.coverage.keys().len(), factors);
    }
}

// --- measured aggregate shape ---

#[test]
fn each_pilot_states_a_complete_relation_case_census() {
    // (relations in scope, relation-cases per plan): two sponsor cases
    // of one fixed representation per plan.
    let expected: BTreeMap<OperationId, (usize, usize)> = BTreeMap::from([
        (OperationId::CompactAsh, (23, 46)),
        (OperationId::TransferLive, (24, 48)),
    ]);

    for pilot in pilots() {
        let operation = pilot.operation();
        let (relations, keys) = expected[&operation];

        assert_eq!(pilot.relation_ids().len(), relations, "{operation:?}");
        assert_eq!(pilot.coverage.plans.len(), 2, "{operation:?}");

        for entry in pilot.coverage.plans.values() {
            assert_eq!(entry.coverage.keys().len(), keys, "{operation:?}");

            // Four inactive-valid rows per plan: the sponsor input and
            // output cardinality and recognition relations, in the
            // unsponsored case.
            let inactive = entry
                .coverage
                .plans()
                .filter(|plan| plan.activity == RelationActivity::Vacuous)
                .count();
            assert_eq!(inactive, 4, "{operation:?}");
        }
    }
}
