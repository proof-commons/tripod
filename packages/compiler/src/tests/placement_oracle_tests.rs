//! Independent exhaustive placement oracle (Guide-5 §13).
//!
//! The production placement search is an exact combinatorial algorithm,
//! so agreeing with itself proves nothing. This oracle enumerates every
//! allowed carrier assignment of every obligation as a plain subset
//! power set, evaluates the hard constraints from restated predicates,
//! retains the inclusion-minimal feasible assignments, and compares the
//! *entire* feasible set with production.
//!
//! Independence is the point of the file. The oracle calls no production
//! carrier eligibility, relation classification, case activation, or
//! layout-completeness helper: every predicate it applies — scope
//! admissibility, quantification against multiplicity, optional sponsor
//! carriers, the runtime census, and the layout dependency of a selected
//! carrier — is written out again here. It shares the stable typed
//! values, because comparing two different vocabularies would prove
//! nothing either.
//!
//! The instances are synthetic by design. A pilot instance exercises
//! what the pilots happen to declare; these exercise the adversarial
//! shapes a validated realization never produces — a global relation
//! offered only a local carrier, an unconditional relation offered only
//! an optional sponsor carrier, an externally evidenced relation handed
//! a runtime carrier — which is where an exact search actually fails.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{AssetId, ObjectId, OperationId};
use realization::{
    AvailabilityClass, ExternalEvidenceRequirement, RelationId, RelationKind, RelationSubject,
    TransactionSide,
};

use crate::{
    CompileError,
    carrier::{
        CarrierAvailability, CarrierEligibility, CarrierIneligibility, CarrierQuantification,
        CarrierRole, EligibleCarrier, IneligibleCarrier, SourceRouting,
    },
    case::{ExecutionCaseId, SponsorCase},
    layout::LayoutRequirement,
    placement::{
        ActivationCondition, CarrierMultiplicity, DischargeBoundary, PlacedCarrier,
        PlacementAssignment, PlacementCandidate, PlacementSearchLimits, RelationActivity,
        RelationCaseKey, RelationCasePlan, RuntimePlacementRequirement, SemanticScope,
        enumerate_feasible_placements, validate_placement,
    },
    source::{
        OperandId, OperandRole, RequiredSourceKind, RequirementActivation, SourceRequirement,
    },
};

// --- the oracle ---

/// What the oracle concludes about one instance.
#[derive(Clone, Debug, PartialEq, Eq)]
enum OracleOutcome {
    /// The complete feasible set, canonically ordered. Empty when some
    /// obligation admits no carrier at all.
    Placements(Vec<PlacementCandidate>),
    /// The carried relation-cases are not the runtime relation-cases.
    CensusDefect,
}

/// Enumerate the complete feasible placement set independently.
///
/// One deliberately linear test-only function: factoring it would start
/// sharing structure with the search it must stay independent of.
#[allow(clippy::too_many_lines)]
fn oracle_enumerate(
    plans: &[RelationCasePlan],
    eligibility: &[CarrierEligibility],
) -> OracleOutcome {
    // Restated census: the carried relation-cases are exactly those
    // whose plan states a runtime requirement, each carried once.
    let mut carried = BTreeSet::new();

    for analysis in eligibility {
        if !carried.insert((analysis.relation.clone(), analysis.case.clone())) {
            return OracleOutcome::CensusDefect;
        }
    }

    let mut runtime = BTreeSet::new();

    for plan in plans {
        if !plan.runtime_requirements.is_empty() {
            runtime.insert((plan.relation.clone(), plan.case.clone()));
        }
    }

    if carried != runtime {
        return OracleOutcome::CensusDefect;
    }

    // One retained assignment list per obligation, in census-key order.
    let mut ordered = eligibility.iter().collect::<Vec<_>>();
    ordered
        .sort_by(|left, right| (&left.relation, &left.case).cmp(&(&right.relation, &right.case)));

    let mut retained = Vec::new();

    for analysis in &ordered {
        let plan = plans
            .iter()
            .find(|plan| plan.relation == analysis.relation && plan.case == analysis.case)
            .expect("the census above proved a plan exists");

        // Every allowed assignment: the complete power set of the
        // eligible carriers, minus the empty one.
        let entries = analysis.eligible.iter().collect::<Vec<_>>();
        let mut feasible: Vec<Vec<&EligibleCarrier>> = Vec::new();

        for mask in 1..(1_usize << entries.len()) {
            let subset = entries
                .iter()
                .enumerate()
                .filter(|(index, _)| mask & (1_usize << index) != 0)
                .map(|(_, entry)| *entry)
                .collect::<Vec<_>>();

            if oracle_feasible(plan, analysis, &subset) {
                feasible.push(subset);
            }
        }

        // Inclusion-minimal only: an assignment with a feasible proper
        // subset adds no semantic content.
        let carriers_of = |subset: &[&EligibleCarrier]| {
            subset
                .iter()
                .map(|entry| entry.carrier.clone())
                .collect::<BTreeSet<_>>()
        };
        let minimal = feasible
            .iter()
            .filter(|subset| {
                let outer = carriers_of(subset);

                !feasible.iter().any(|other| {
                    let inner = carriers_of(other);

                    inner != outer && inner.is_subset(&outer)
                })
            })
            .cloned()
            .collect::<Vec<_>>();

        retained.push(minimal);
    }

    if retained.iter().any(Vec::is_empty) {
        return OracleOutcome::Placements(Vec::new());
    }

    // The product of the retained assignments, by odometer.
    let mut odometer = vec![0_usize; retained.len()];
    let mut placements = Vec::new();

    'outer: loop {
        let mut assignments = Vec::with_capacity(ordered.len());
        let mut layout = BTreeSet::new();

        for (position, analysis) in ordered.iter().enumerate() {
            let subset = &retained[position][odometer[position]];
            let mut carriers = subset
                .iter()
                .map(|entry| PlacedCarrier {
                    carrier: entry.carrier.clone(),
                    quantification: entry.quantification,
                })
                .collect::<Vec<_>>();

            carriers.sort();
            carriers.dedup();

            for entry in subset {
                layout.extend(oracle_layout(analysis, entry));
            }

            assignments.push(PlacementAssignment {
                relation: analysis.relation.clone(),
                case: analysis.case.clone(),
                carriers,
            });
        }

        assignments.sort();
        placements.push(PlacementCandidate {
            assignments,
            layout_requirements: layout.into_iter().collect(),
        });

        for position in (0..retained.len()).rev() {
            odometer[position] += 1;

            if odometer[position] < retained[position].len() {
                continue 'outer;
            }

            odometer[position] = 0;
        }

        break;
    }

    placements.sort();
    placements.dedup();
    OracleOutcome::Placements(placements)
}

/// The hard constraints of one assignment, restated.
fn oracle_feasible(
    plan: &RelationCasePlan,
    analysis: &CarrierEligibility,
    subset: &[&EligibleCarrier],
) -> bool {
    if subset.is_empty() {
        return false;
    }

    for entry in subset {
        if !oracle_admits(plan, analysis, entry) {
            return false;
        }
    }

    match analysis.multiplicity {
        // One carrier and no second one.
        CarrierMultiplicity::ExactlyOne => subset.len() == 1,

        // Any non-empty set of carriers that each discharge the
        // obligation's quantification.
        CarrierMultiplicity::EveryMember | CarrierMultiplicity::AtLeastOne => true,

        // Duplication is deliberate: the assignment is the complete set
        // of carriers the obligation permits, never a smaller one.
        CarrierMultiplicity::DeliberateDuplication => {
            let selected = subset
                .iter()
                .map(|entry| entry.carrier.clone())
                .collect::<BTreeSet<_>>();
            let permitted = analysis
                .eligible
                .iter()
                .filter(|entry| oracle_admits(plan, analysis, entry))
                .map(|entry| entry.carrier.clone())
                .collect::<BTreeSet<_>>();

            selected == permitted
        }
    }
}

/// Whether one carrier may discharge one obligation, restated.
fn oracle_admits(
    plan: &RelationCasePlan,
    analysis: &CarrierEligibility,
    entry: &EligibleCarrier,
) -> bool {
    // A per-member role sees one member, and a family coordinator sees
    // one family, so neither can establish a property of a wider scope.
    // The non-runtime roles carry nothing at runtime at all.
    let scoped = match &entry.carrier {
        CarrierRole::OperationGlobal { .. } => true,

        CarrierRole::InputFamilyCoordinator { object: family } => match analysis.scope {
            SemanticScope::MemberLocal { side, object }
            | SemanticScope::FamilyGlobal { side, object } => {
                side == TransactionSide::Input && object == *family
            }
            _ => false,
        },

        CarrierRole::EveryInputFamilyMember { object: family } => match analysis.scope {
            SemanticScope::MemberLocal { side, object } => {
                side == TransactionSide::Input && object == *family
            }
            _ => false,
        },

        CarrierRole::BackendStructural { .. } | CarrierRole::ExternalEvidence { .. } => false,
    };

    if !scoped {
        return false;
    }

    // An every-member obligation is discharged per member or by a typed
    // complete-family proof, and by nothing else; every other
    // multiplicity is discharged by an unquantified single carrier.
    let quantified = match analysis.multiplicity {
        CarrierMultiplicity::EveryMember => matches!(
            entry.quantification,
            CarrierQuantification::PerMember | CarrierQuantification::CompleteFamilyProof
        ),
        _ => entry.quantification == CarrierQuantification::Single,
    };

    if !quantified {
        return false;
    }

    // An optional sponsor carrier may carry its own conditional
    // relations, never one that holds with or without the region.
    let optional = match &entry.carrier {
        CarrierRole::EveryInputFamilyMember { object }
        | CarrierRole::InputFamilyCoordinator { object } => *object == ObjectId::PlainLbtc,
        _ => false,
    };

    !(optional && plan.activation == ActivationCondition::Always)
}

/// What one selected carrier depends on, restated.
fn oracle_layout(
    analysis: &CarrierEligibility,
    entry: &EligibleCarrier,
) -> BTreeSet<LayoutRequirement> {
    let mut requirements = BTreeSet::new();
    let relation = analysis.relation.clone();

    if matches!(
        entry.carrier,
        CarrierRole::OperationGlobal { .. } | CarrierRole::InputFamilyCoordinator { .. }
    ) {
        requirements.insert(LayoutRequirement::CanonicalCoordinator {
            operation: relation.operation(),
            anchor: entry.carrier.clone(),
        });
    }

    if entry.quantification == CarrierQuantification::CompleteFamilyProof
        && let SemanticScope::MemberLocal { side, object } = analysis.scope
    {
        requirements.insert(LayoutRequirement::AuthenticateFamilyCensus {
            relation: relation.clone(),
            side,
            object,
        });
        requirements.insert(LayoutRequirement::CompleteAndDisjointFamilies {
            relation: relation.clone(),
            side,
        });
    }

    for routing in &entry.routings {
        match routing.availability {
            CarrierAvailability::Intrinsic => {}

            CarrierAvailability::LayoutProvided => {
                requirements.insert(LayoutRequirement::MakeSourceAvailable {
                    relation: relation.clone(),
                    case: analysis.case.clone(),
                    carrier: entry.carrier.clone(),
                    source: routing.source.clone(),
                });
            }

            CarrierAvailability::SponsorRegionConfined => {
                requirements.insert(LayoutRequirement::IsolateSponsorRegion {
                    relation: relation.clone(),
                    case: analysis.case.clone(),
                });
            }
        }
    }

    requirements
}

// --- synthetic instances ---

const OPERATION: OperationId = OperationId::TransferLive;

fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(10_000).expect("nonzero"),
    )
}

fn case(sponsor: SponsorCase) -> ExecutionCaseId {
    ExecutionCaseId {
        operation: OPERATION,
        sponsor,
        representations: BTreeMap::new(),
    }
}

fn relation(kind: RelationKind, subject: RelationSubject) -> RelationId {
    RelationId::new(OPERATION, kind, subject)
}

fn family_relation(kind: RelationKind, object: ObjectId) -> RelationId {
    relation(
        kind,
        RelationSubject::ObjectFamily {
            side: TransactionSide::Input,
            object,
        },
    )
}

/// One source row of one relation, with an explicit availability.
fn row(
    relation: &RelationId,
    role: OperandRole,
    source: RequiredSourceKind,
    availability: AvailabilityClass,
) -> SourceRequirement {
    SourceRequirement {
        operand: OperandId::new(relation.clone(), role),
        source,
        availability,
        activation: RequirementActivation::Always,
    }
}

fn routing(source: SourceRequirement, availability: CarrierAvailability) -> SourceRouting {
    SourceRouting {
        source,
        availability,
    }
}

fn eligible(
    carrier: CarrierRole,
    quantification: CarrierQuantification,
    routings: Vec<SourceRouting>,
) -> EligibleCarrier {
    let mut routings = routings;
    routings.sort();

    EligibleCarrier {
        carrier,
        quantification,
        routings,
    }
}

const fn member(object: ObjectId) -> CarrierRole {
    CarrierRole::EveryInputFamilyMember { object }
}

const fn coordinator(object: ObjectId) -> CarrierRole {
    CarrierRole::InputFamilyCoordinator { object }
}

const fn global(anchor: ObjectId) -> CarrierRole {
    CarrierRole::OperationGlobal {
        operation: OPERATION,
        anchor,
    }
}

/// One synthetic runtime obligation.
struct Obligation {
    relation: RelationId,
    case: ExecutionCaseId,
    activation: ActivationCondition,
    scope: SemanticScope,
    multiplicity: CarrierMultiplicity,
    eligible: Vec<EligibleCarrier>,
    ineligible: Vec<IneligibleCarrier>,
}

impl Obligation {
    fn new(
        relation: RelationId,
        scope: SemanticScope,
        multiplicity: CarrierMultiplicity,
        eligible: Vec<EligibleCarrier>,
    ) -> Self {
        Self {
            relation,
            case: case(SponsorCase::Absent),
            activation: ActivationCondition::Always,
            scope,
            multiplicity,
            eligible,
            ineligible: Vec::new(),
        }
    }

    fn in_case(mut self, sponsor: SponsorCase) -> Self {
        self.case = case(sponsor);
        self
    }

    fn when_sponsored(mut self) -> Self {
        self.activation = ActivationCondition::WhenSponsorPresent;
        self.case = case(SponsorCase::Present);
        self
    }

    fn rejecting(mut self, carrier: CarrierRole, reason: CarrierIneligibility) -> Self {
        self.ineligible.push(IneligibleCarrier { carrier, reason });
        self
    }

    fn plan(&self) -> RelationCasePlan {
        RelationCasePlan {
            relation: self.relation.clone(),
            case: self.case.clone(),
            activation: self.activation,
            activity: RelationActivity::Active,
            boundaries: BTreeSet::from([DischargeBoundary::RuntimeCarrier]),
            compiler_requirements: Vec::new(),
            structural_requirements: Vec::new(),
            runtime_requirements: vec![RuntimePlacementRequirement {
                relation: self.relation.clone(),
                scope: self.scope,
                multiplicity: self.multiplicity,
                sources: self
                    .eligible
                    .first()
                    .map(|entry| {
                        entry
                            .routings
                            .iter()
                            .map(|routing| routing.source.clone())
                            .collect()
                    })
                    .unwrap_or_default(),
            }],
            external_evidence: BTreeSet::new(),
        }
    }

    fn analysis(&self) -> CarrierEligibility {
        let mut eligible = self.eligible.clone();
        let mut ineligible = self.ineligible.clone();

        eligible.sort();
        ineligible.sort();

        CarrierEligibility {
            relation: self.relation.clone(),
            case: self.case.clone(),
            scope: self.scope,
            multiplicity: self.multiplicity,
            eligible,
            ineligible,
        }
    }
}

/// One relation-case discharged away from the runtime boundary.
fn non_runtime_plan(
    relation: &RelationId,
    boundary: DischargeBoundary,
    external: BTreeSet<ExternalEvidenceRequirement>,
) -> RelationCasePlan {
    RelationCasePlan {
        relation: relation.clone(),
        case: case(SponsorCase::Absent),
        activation: ActivationCondition::Always,
        activity: RelationActivity::Active,
        boundaries: BTreeSet::from([boundary]),
        compiler_requirements: Vec::new(),
        structural_requirements: Vec::new(),
        runtime_requirements: Vec::new(),
        external_evidence: external,
    }
}

fn instance(obligations: &[Obligation]) -> (Vec<RelationCasePlan>, Vec<CarrierEligibility>) {
    (
        obligations.iter().map(Obligation::plan).collect(),
        obligations.iter().map(Obligation::analysis).collect(),
    )
}

/// Production and the oracle agree on this instance; the feasible set is
/// returned when one exists.
fn agree(
    plans: &[RelationCasePlan],
    eligibility: &[CarrierEligibility],
) -> Vec<PlacementCandidate> {
    let oracle = oracle_enumerate(plans, eligibility);
    let production = enumerate_feasible_placements(plans, eligibility, limits());

    match (oracle, production) {
        (OracleOutcome::Placements(oracle), Ok(production)) => {
            assert!(!oracle.is_empty(), "production feasible, oracle empty");
            assert_eq!(production.candidates, oracle);
            oracle
        }

        (OracleOutcome::Placements(oracle), Err(error)) => {
            assert!(oracle.is_empty(), "production rejected a feasible instance");
            assert!(
                matches!(
                    error,
                    CompileError::NoEligibleCarrier { .. }
                        | CompileError::GlobalRelationHasOnlyLocalCarrier { .. }
                        | CompileError::UnconditionalRelationOnOptionalCarrier { .. }
                ),
                "unexpected rejection {error:?}",
            );
            Vec::new()
        }

        (OracleOutcome::CensusDefect, production) => {
            assert!(
                matches!(
                    production,
                    Err(CompileError::PlacementCensusMismatch { .. })
                ),
                "oracle found a census defect, production did not",
            );
            Vec::new()
        }
    }
}

// --- the required oracle cases (§13) ---

/// One member-local every-member obligation over the mandatory family,
/// and one transaction-global obligation on its coordinator.
fn mandatory_family_and_coordinator() -> Vec<Obligation> {
    let recognition = family_relation(RelationKind::Recognition, ObjectId::ReceiptLive);
    let conservation = relation(
        RelationKind::Conservation,
        RelationSubject::Asset { asset: AssetId::U },
    );
    let members = row(
        &recognition,
        OperandRole::ObjectFamilyMembers {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
        RequiredSourceKind::AuthenticatedInputObject,
        AvailabilityClass::Public,
    );
    let census = row(
        &conservation,
        OperandRole::ObjectFamilyAmount {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
        RequiredSourceKind::AuthenticatedFamilyCensus,
        AvailabilityClass::Public,
    );

    vec![
        Obligation::new(
            recognition,
            SemanticScope::MemberLocal {
                side: TransactionSide::Input,
                object: ObjectId::ReceiptLive,
            },
            CarrierMultiplicity::EveryMember,
            vec![
                eligible(
                    member(ObjectId::ReceiptLive),
                    CarrierQuantification::PerMember,
                    vec![routing(members.clone(), CarrierAvailability::Intrinsic)],
                ),
                eligible(
                    global(ObjectId::ReceiptLive),
                    CarrierQuantification::CompleteFamilyProof,
                    vec![routing(members, CarrierAvailability::LayoutProvided)],
                ),
            ],
        ),
        Obligation::new(
            conservation,
            SemanticScope::TransactionGlobal,
            CarrierMultiplicity::ExactlyOne,
            vec![eligible(
                global(ObjectId::ReceiptLive),
                CarrierQuantification::Single,
                vec![routing(census, CarrierAvailability::LayoutProvided)],
            )],
        ),
    ]
}

#[test]
fn one_mandatory_family_and_one_coordinator_agree_with_the_oracle() {
    let obligations = mandatory_family_and_coordinator();
    let (plans, eligibility) = instance(&obligations);
    let placements = agree(&plans, &eligibility);

    // The member-local obligation admits the quantified role or the
    // typed complete-family proof, and nothing else; the global one
    // admits its coordinator alone.
    assert_eq!(placements.len(), 2);

    for placement in &placements {
        for assignment in &placement.assignments {
            assert_eq!(assignment.carriers.len(), 1);
        }
    }
}

#[test]
fn two_eligible_coordinators_agree_with_the_oracle() {
    let conservation = relation(
        RelationKind::Conservation,
        RelationSubject::Asset { asset: AssetId::U },
    );
    let obligations = vec![Obligation::new(
        conservation,
        SemanticScope::TransactionGlobal,
        CarrierMultiplicity::ExactlyOne,
        vec![
            eligible(
                global(ObjectId::ReceiptLive),
                CarrierQuantification::Single,
                Vec::new(),
            ),
            eligible(
                global(ObjectId::Ash),
                CarrierQuantification::Single,
                Vec::new(),
            ),
        ],
    )];
    let (plans, eligibility) = instance(&obligations);
    let placements = agree(&plans, &eligibility);

    // Both anchors survive: no cost model prefers one over the other.
    assert_eq!(placements.len(), 2);
}

#[test]
fn an_obligation_with_no_eligible_carrier_is_rejected() {
    let obligations = vec![Obligation::new(
        family_relation(RelationKind::Recognition, ObjectId::ReceiptLive),
        SemanticScope::MemberLocal {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
        CarrierMultiplicity::EveryMember,
        Vec::new(),
    )];
    let (plans, eligibility) = instance(&obligations);

    assert_eq!(agree(&plans, &eligibility), [] as [PlacementCandidate; 0]);
    assert!(matches!(
        enumerate_feasible_placements(&plans, &eligibility, limits()),
        Err(CompileError::NoEligibleCarrier { .. }),
    ));
}

#[test]
fn a_conditional_relation_may_be_carried_by_the_optional_sponsor_family_alone() {
    let recognition = family_relation(RelationKind::Recognition, ObjectId::PlainLbtc);
    let obligations = vec![
        Obligation::new(
            recognition,
            SemanticScope::MemberLocal {
                side: TransactionSide::Input,
                object: ObjectId::PlainLbtc,
            },
            CarrierMultiplicity::EveryMember,
            vec![eligible(
                member(ObjectId::PlainLbtc),
                CarrierQuantification::PerMember,
                Vec::new(),
            )],
        )
        .when_sponsored(),
    ];
    let (plans, eligibility) = instance(&obligations);
    let placements = agree(&plans, &eligibility);

    assert_eq!(placements.len(), 1);
    assert_eq!(
        placements[0].assignments[0].carriers[0].carrier,
        member(ObjectId::PlainLbtc),
    );
}

#[test]
fn an_unconditional_relation_with_only_an_optional_carrier_is_rejected() {
    let recognition = family_relation(RelationKind::Recognition, ObjectId::PlainLbtc);
    let obligations = vec![Obligation::new(
        recognition,
        SemanticScope::MemberLocal {
            side: TransactionSide::Input,
            object: ObjectId::PlainLbtc,
        },
        CarrierMultiplicity::EveryMember,
        vec![eligible(
            member(ObjectId::PlainLbtc),
            CarrierQuantification::PerMember,
            Vec::new(),
        )],
    )];
    let (plans, eligibility) = instance(&obligations);

    assert_eq!(agree(&plans, &eligibility), [] as [PlacementCandidate; 0]);
    assert!(matches!(
        enumerate_feasible_placements(&plans, &eligibility, limits()),
        Err(CompileError::UnconditionalRelationOnOptionalCarrier { .. }),
    ));
}

#[test]
fn a_member_local_relation_missing_its_member_role_keeps_only_the_complete_family_proof() {
    let recognition = family_relation(RelationKind::Recognition, ObjectId::ReceiptLive);
    let members = row(
        &recognition,
        OperandRole::ObjectFamilyMembers {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
        RequiredSourceKind::AuthenticatedInputObject,
        AvailabilityClass::Public,
    );
    let scope = SemanticScope::MemberLocal {
        side: TransactionSide::Input,
        object: ObjectId::ReceiptLive,
    };

    // The quantified per-member role is absent, so only the typed
    // complete-family proof remains — and it states the census it
    // relies on rather than assuming completeness.
    let obligations = vec![Obligation::new(
        recognition.clone(),
        scope,
        CarrierMultiplicity::EveryMember,
        vec![eligible(
            global(ObjectId::ReceiptLive),
            CarrierQuantification::CompleteFamilyProof,
            vec![routing(members, CarrierAvailability::LayoutProvided)],
        )],
    )];
    let (plans, eligibility) = instance(&obligations);
    let placements = agree(&plans, &eligibility);

    assert_eq!(placements.len(), 1);
    assert!(placements[0].layout_requirements.contains(
        &LayoutRequirement::AuthenticateFamilyCensus {
            relation: recognition.clone(),
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
    ));

    // An unquantified single carrier is a different obligation, and it
    // may not stand in for every member.
    let obligations = vec![Obligation::new(
        recognition,
        scope,
        CarrierMultiplicity::EveryMember,
        vec![eligible(
            global(ObjectId::ReceiptLive),
            CarrierQuantification::Single,
            Vec::new(),
        )],
    )];
    let (plans, eligibility) = instance(&obligations);

    assert_eq!(agree(&plans, &eligibility), [] as [PlacementCandidate; 0]);
}

#[test]
fn a_transaction_global_relation_offered_only_a_local_carrier_is_rejected() {
    let conservation = relation(
        RelationKind::Conservation,
        RelationSubject::Asset { asset: AssetId::U },
    );
    let obligations = vec![Obligation::new(
        conservation,
        SemanticScope::TransactionGlobal,
        CarrierMultiplicity::ExactlyOne,
        vec![
            eligible(
                member(ObjectId::ReceiptLive),
                CarrierQuantification::Single,
                Vec::new(),
            ),
            eligible(
                coordinator(ObjectId::ReceiptLive),
                CarrierQuantification::Single,
                Vec::new(),
            ),
        ],
    )];
    let (plans, eligibility) = instance(&obligations);

    assert_eq!(agree(&plans, &eligibility), [] as [PlacementCandidate; 0]);
    assert!(matches!(
        enumerate_feasible_placements(&plans, &eligibility, limits()),
        Err(CompileError::GlobalRelationHasOnlyLocalCarrier { .. }),
    ));
}

#[test]
fn a_carrier_whose_source_is_unavailable_is_never_selected() {
    let authorization = family_relation(RelationKind::Authorization, ObjectId::ReceiptLive);
    let witness = row(
        &authorization,
        OperandRole::ProtocolSignerSet,
        RequiredSourceKind::InputOwnerWitness,
        AvailabilityClass::InputOwners {
            object: ObjectId::ReceiptLive,
        },
    );
    let obligations = vec![
        Obligation::new(
            authorization,
            SemanticScope::MemberLocal {
                side: TransactionSide::Input,
                object: ObjectId::ReceiptLive,
            },
            CarrierMultiplicity::EveryMember,
            vec![eligible(
                member(ObjectId::ReceiptLive),
                CarrierQuantification::PerMember,
                vec![routing(witness.clone(), CarrierAvailability::Intrinsic)],
            )],
        )
        .rejecting(
            global(ObjectId::ReceiptLive),
            CarrierIneligibility::SourceUnavailable {
                operand: witness.operand,
            },
        ),
    ];
    let (plans, eligibility) = instance(&obligations);
    let placements = agree(&plans, &eligibility);

    // The rejected coordinator carries nothing: no owner witness is
    // routed out of its authorized input family.
    assert_eq!(placements.len(), 1);

    for placement in &placements {
        for assignment in &placement.assignments {
            for carrier in &assignment.carriers {
                assert_ne!(carrier.carrier, global(ObjectId::ReceiptLive));
            }
        }
    }
}

#[test]
fn a_layout_provided_source_becomes_a_placement_layout_requirement() {
    let cardinality = family_relation(RelationKind::Cardinality, ObjectId::ReceiptLive);
    let census = row(
        &cardinality,
        OperandRole::ObjectFamilyCount {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
        RequiredSourceKind::AuthenticatedFamilyCensus,
        AvailabilityClass::Public,
    );
    let obligations = vec![Obligation::new(
        cardinality.clone(),
        SemanticScope::FamilyGlobal {
            side: TransactionSide::Input,
            object: ObjectId::ReceiptLive,
        },
        CarrierMultiplicity::ExactlyOne,
        vec![eligible(
            global(ObjectId::ReceiptLive),
            CarrierQuantification::Single,
            vec![routing(census.clone(), CarrierAvailability::LayoutProvided)],
        )],
    )];
    let (plans, eligibility) = instance(&obligations);
    let placements = agree(&plans, &eligibility);

    assert_eq!(placements.len(), 1);
    assert!(
        placements[0]
            .layout_requirements
            .contains(&LayoutRequirement::MakeSourceAvailable {
                relation: cardinality,
                case: case(SponsorCase::Absent),
                carrier: global(ObjectId::ReceiptLive),
                source: census,
            },)
    );
}

#[test]
fn an_external_evidence_relation_given_a_runtime_carrier_is_a_census_defect() {
    let substrate = relation(
        RelationKind::SubstrateConservation,
        RelationSubject::Asset {
            asset: AssetId::Lbtc,
        },
    );
    let obligations = vec![Obligation::new(
        substrate.clone(),
        SemanticScope::TransactionGlobal,
        CarrierMultiplicity::ExactlyOne,
        vec![eligible(
            global(ObjectId::ReceiptLive),
            CarrierQuantification::Single,
            Vec::new(),
        )],
    )];
    let (_, eligibility) = instance(&obligations);

    // The relation is externally evidenced, so its plan states no
    // runtime requirement — and a runtime carrier for it would report an
    // unresolved evidence obligation as target execution.
    let plans = vec![non_runtime_plan(
        &substrate,
        DischargeBoundary::ExternalEvidence,
        BTreeSet::from([ExternalEvidenceRequirement::SubstrateConservation {
            operation: OPERATION,
            asset: AssetId::Lbtc,
        }]),
    )];

    assert_eq!(agree(&plans, &eligibility), [] as [PlacementCandidate; 0]);
    assert!(matches!(
        enumerate_feasible_placements(&plans, &eligibility, limits()),
        Err(CompileError::PlacementCensusMismatch { .. }),
    ));
}

#[test]
fn a_compiler_static_relation_given_a_runtime_carrier_is_a_census_defect() {
    let representation = relation(
        RelationKind::Representation,
        RelationSubject::Representation {
            object: ObjectId::ReceiptLive,
        },
    );
    let obligations = vec![Obligation::new(
        representation.clone(),
        SemanticScope::ObjectFamilyGlobal {
            object: ObjectId::ReceiptLive,
        },
        CarrierMultiplicity::ExactlyOne,
        vec![eligible(
            global(ObjectId::ReceiptLive),
            CarrierQuantification::Single,
            Vec::new(),
        )],
    )];
    let (_, eligibility) = instance(&obligations);
    let plans = vec![non_runtime_plan(
        &representation,
        DischargeBoundary::CompilerStatic,
        BTreeSet::new(),
    )];

    assert_eq!(agree(&plans, &eligibility), [] as [PlacementCandidate; 0]);
    assert!(matches!(
        enumerate_feasible_placements(&plans, &eligibility, limits()),
        Err(CompileError::PlacementCensusMismatch { .. }),
    ));
}

#[test]
fn deliberate_duplication_retains_the_complete_carrier_set() {
    let policy = relation(RelationKind::RootPolicy, RelationSubject::Operation);
    let obligations = vec![Obligation::new(
        policy,
        SemanticScope::TransactionGlobal,
        CarrierMultiplicity::DeliberateDuplication,
        vec![
            eligible(
                global(ObjectId::ReceiptLive),
                CarrierQuantification::Single,
                Vec::new(),
            ),
            eligible(
                global(ObjectId::Ash),
                CarrierQuantification::Single,
                Vec::new(),
            ),
        ],
    )];
    let (plans, eligibility) = instance(&obligations);
    let placements = agree(&plans, &eligibility);

    // Duplication is required, not optional: one carrier is not a
    // smaller feasible answer, and the pair is not an arbitrary
    // superset of one.
    assert_eq!(placements.len(), 1);
    assert_eq!(placements[0].assignments[0].carriers.len(), 2);
}

#[test]
fn the_oracle_agrees_with_production_on_every_pilot_scope() {
    // The upstream stages are the search's *input*, so both sides are
    // handed the same relation-case plans and eligible carrier sets; the
    // oracle still derives every placement from its own restated
    // predicates.
    for operation in [OperationId::CompactAsh, OperationId::TransferLive] {
        let input = super::bound_input(&[operation]);
        let relations = crate::relation::build_relation_analysis(&input).expect("relations");
        let candidates = crate::proof::enumerate_feasible_plans(
            &input,
            &crate::capability::CapabilityView::Unconstrained,
        )
        .expect("plans")
        .candidates;

        for candidate in &candidates {
            let cases = crate::case::execution_cases(&relations, candidate).expect("cases");
            let plans = crate::placement::classify_relation_cases(&relations, &cases)
                .expect("classification");
            let eligibility =
                crate::carrier::relation_case_eligibility(&relations, &plans).expect("eligibility");
            let placements = agree(&plans, &eligibility);

            assert!(!placements.is_empty(), "{operation:?}");
        }
    }
}

// --- determinism (§16.8) ---

/// The same instance with every input vector reversed.
fn reversed(
    plans: &[RelationCasePlan],
    eligibility: &[CarrierEligibility],
) -> (Vec<RelationCasePlan>, Vec<CarrierEligibility>) {
    let mut plans = plans.to_vec();
    let mut eligibility = eligibility.to_vec();

    plans.reverse();
    eligibility.reverse();

    for analysis in &mut eligibility {
        analysis.eligible.reverse();
        analysis.ineligible.reverse();
    }

    (plans, eligibility)
}

#[test]
fn permuted_relation_carrier_and_case_order_produces_equal_placements() {
    let mut obligations = mandatory_family_and_coordinator();

    // A second execution case of the same relations, so the permutation
    // crosses cases as well as relations and carriers.
    let sponsored = mandatory_family_and_coordinator()
        .into_iter()
        .map(|obligation| obligation.in_case(SponsorCase::Present))
        .collect::<Vec<_>>();
    obligations.extend(sponsored);

    let (plans, eligibility) = instance(&obligations);
    let first = enumerate_feasible_placements(&plans, &eligibility, limits()).expect("placements");

    let (plans, eligibility) = reversed(&plans, &eligibility);
    let second = enumerate_feasible_placements(&plans, &eligibility, limits()).expect("placements");

    assert_eq!(first, second);
    assert_eq!(first.project(), second.project());
    assert_eq!(
        oracle_enumerate(&plans, &eligibility),
        OracleOutcome::Placements(first.candidates.clone()),
    );

    // Four independent binary obligations: the complete product, with
    // nothing pruned by preference.
    assert_eq!(first.candidates.len(), 4);
}

#[test]
fn repeated_placement_search_is_equal() {
    let obligations = mandatory_family_and_coordinator();
    let (plans, eligibility) = instance(&obligations);

    let first = enumerate_feasible_placements(&plans, &eligibility, limits()).expect("first");
    let second = enumerate_feasible_placements(&plans, &eligibility, limits()).expect("second");

    assert_eq!(first, second);
    assert_eq!(first.project(), second.project());
}

#[test]
fn placement_census_diagnostics_are_stably_ordered() {
    let missing = mandatory_family_and_coordinator();
    let (plans, _) = instance(&missing);
    let (_, eligibility) = instance(&[Obligation::new(
        relation(RelationKind::ProjectionPolicy, RelationSubject::Operation),
        SemanticScope::TransactionGlobal,
        CarrierMultiplicity::ExactlyOne,
        vec![eligible(
            global(ObjectId::ReceiptLive),
            CarrierQuantification::Single,
            Vec::new(),
        )],
    )]);

    let Err(CompileError::PlacementCensusMismatch {
        missing,
        unexpected,
    }) = enumerate_feasible_placements(&plans, &eligibility, limits())
    else {
        panic!("expected a census defect");
    };

    assert_eq!(missing.len(), 2);
    assert_eq!(unexpected.len(), 1);
    assert!(missing.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(unexpected.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(
        missing
            .iter()
            .map(|key| key.relation.kind())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([RelationKind::Conservation, RelationKind::Recognition]),
    );
}

// --- complexity limits (§10.5) ---

#[test]
fn state_limit_exhaustion_returns_no_partial_placement() {
    let obligations = mandatory_family_and_coordinator();
    let (plans, eligibility) = instance(&obligations);
    let one = std::num::NonZeroU64::new(1).expect("nonzero");
    let generous = std::num::NonZeroU64::new(1_000_000).expect("nonzero");

    assert_eq!(
        enumerate_feasible_placements(
            &plans,
            &eligibility,
            PlacementSearchLimits::new(one, generous),
        ),
        Err(CompileError::PlacementSearchStateLimitExceeded { maximum: 1 }),
    );
}

#[test]
fn candidate_limit_exhaustion_returns_no_partial_placement() {
    let obligations = mandatory_family_and_coordinator();
    let (plans, eligibility) = instance(&obligations);
    let one = std::num::NonZeroU64::new(1).expect("nonzero");
    let generous = std::num::NonZeroU64::new(1_000_000).expect("nonzero");

    // The oracle finds two feasible placements, so a one-candidate limit
    // truncates a complete result — and truncation is a typed failure
    // rather than a retained first placement.
    assert_eq!(oracle_enumerate(&plans, &eligibility), {
        let complete = enumerate_feasible_placements(&plans, &eligibility, limits())
            .expect("complete")
            .candidates;
        assert_eq!(complete.len(), 2);
        OracleOutcome::Placements(complete)
    });
    assert_eq!(
        enumerate_feasible_placements(
            &plans,
            &eligibility,
            PlacementSearchLimits::new(generous, one),
        ),
        Err(CompileError::PlacementCandidateLimitExceeded { maximum: 1 }),
    );
}

// --- placements built elsewhere are validated, not trusted ---

#[test]
fn a_placement_naming_an_impermissible_carrier_is_rejected() {
    let obligations = mandatory_family_and_coordinator();
    let (plans, eligibility) = instance(&obligations);
    let mut placement = enumerate_feasible_placements(&plans, &eligibility, limits())
        .expect("placements")
        .candidates
        .swap_remove(0);

    // A non-runtime role is not a carrier: the backend-structural and
    // external-evidence boundaries discharge their own obligations, and
    // neither executes anything.
    placement.assignments[0].carriers = vec![PlacedCarrier {
        carrier: CarrierRole::BackendStructural {
            operation: OPERATION,
        },
        quantification: CarrierQuantification::Single,
    }];

    assert!(matches!(
        validate_placement(&plans, &eligibility, &placement),
        Err(CompileError::UnpermittedCarrierPlacement { .. }),
    ));
}

#[test]
fn a_placement_dropping_a_relation_case_is_rejected() {
    let obligations = mandatory_family_and_coordinator();
    let (plans, eligibility) = instance(&obligations);
    let mut placement = enumerate_feasible_placements(&plans, &eligibility, limits())
        .expect("placements")
        .candidates
        .swap_remove(0);
    let dropped = placement.assignments.remove(0);

    let Err(CompileError::PlacementCensusMismatch { missing, .. }) =
        validate_placement(&plans, &eligibility, &placement)
    else {
        panic!("expected a census defect");
    };

    assert_eq!(
        missing,
        vec![RelationCaseKey {
            relation: dropped.relation,
            case: dropped.case,
        }],
    );
}

#[test]
fn a_placement_omitting_a_layout_dependency_is_rejected() {
    let obligations = mandatory_family_and_coordinator();
    let (plans, eligibility) = instance(&obligations);
    let placement = enumerate_feasible_placements(&plans, &eligibility, limits())
        .expect("placements")
        .candidates
        .into_iter()
        .find(|placement| !placement.layout_requirements.is_empty())
        .expect("a placement with a layout dependency");
    let stripped = PlacementCandidate {
        assignments: placement.assignments,
        layout_requirements: Vec::new(),
    };

    assert!(matches!(
        validate_placement(&plans, &eligibility, &stripped),
        Err(CompileError::MissingLayoutRequirement { .. }),
    ));
}

// --- the exact candidate language (§3.4) ---
//
// Enumeration retains one inclusion-minimal option per obligation, so a
// validator that accepted supersets, duplicates, or surplus layout would
// describe a wider language than the search can produce — and a later
// stage could not delegate exactness to it.

/// One feasible placement of the two-obligation instance.
fn one_placement(
    plans: &[RelationCasePlan],
    eligibility: &[CarrierEligibility],
) -> PlacementCandidate {
    enumerate_feasible_placements(plans, eligibility, limits())
        .expect("placements")
        .candidates
        .swap_remove(0)
}

/// The mutable assignment of one relation inside one placement.
fn assignment_of<'a>(
    placement: &'a mut PlacementCandidate,
    relation: &RelationId,
) -> &'a mut PlacementAssignment {
    placement
        .assignments
        .iter_mut()
        .find(|assignment| &assignment.relation == relation)
        .expect("the relation is placed")
}

#[test]
fn a_placement_repeating_a_carrier_is_rejected() {
    let obligations = mandatory_family_and_coordinator();
    let (plans, eligibility) = instance(&obligations);
    let mut placement = one_placement(&plans, &eligibility);
    let repeated = placement.assignments[0].carriers[0].clone();

    placement.assignments[0].carriers.push(repeated.clone());

    assert_eq!(
        validate_placement(&plans, &eligibility, &placement),
        Err(CompileError::DuplicatePlacedCarrier {
            relation: placement.assignments[0].relation.clone(),
            case: placement.assignments[0].case.clone(),
            carrier: repeated.carrier,
        }),
    );
}

#[test]
fn an_every_member_obligation_may_not_select_both_quantified_alternatives() {
    let obligations = mandatory_family_and_coordinator();
    let (plans, eligibility) = instance(&obligations);
    let recognition = family_relation(RelationKind::Recognition, ObjectId::ReceiptLive);
    let mut placement = one_placement(&plans, &eligibility);

    // The per-member role and the complete-family proof each discharge
    // the obligation alone, so selecting both is a superset of two
    // retained options rather than a third one.
    let assignment = assignment_of(&mut placement, &recognition);
    assignment.carriers = vec![
        PlacedCarrier {
            carrier: member(ObjectId::ReceiptLive),
            quantification: CarrierQuantification::PerMember,
        },
        PlacedCarrier {
            carrier: global(ObjectId::ReceiptLive),
            quantification: CarrierQuantification::CompleteFamilyProof,
        },
    ];
    assignment.carriers.sort();

    let case = assignment.case.clone();

    assert_eq!(
        validate_placement(&plans, &eligibility, &placement),
        Err(CompileError::NonCanonicalCarrierAssignment {
            relation: recognition,
            case,
        }),
    );
}

#[test]
fn an_exactly_one_obligation_may_not_select_two_admissible_carriers() {
    let conservation = relation(
        RelationKind::Conservation,
        RelationSubject::Asset { asset: AssetId::U },
    );
    let obligations = vec![Obligation::new(
        conservation.clone(),
        SemanticScope::TransactionGlobal,
        CarrierMultiplicity::ExactlyOne,
        vec![
            eligible(
                global(ObjectId::ReceiptLive),
                CarrierQuantification::Single,
                Vec::new(),
            ),
            eligible(
                global(ObjectId::Ash),
                CarrierQuantification::Single,
                Vec::new(),
            ),
        ],
    )];
    let (plans, eligibility) = instance(&obligations);
    let mut placement = one_placement(&plans, &eligibility);

    // Both anchors are admissible one at a time; neither is a second
    // carrier of the other.
    let assignment = assignment_of(&mut placement, &conservation);
    assignment.carriers = vec![
        PlacedCarrier {
            carrier: global(ObjectId::Ash),
            quantification: CarrierQuantification::Single,
        },
        PlacedCarrier {
            carrier: global(ObjectId::ReceiptLive),
            quantification: CarrierQuantification::Single,
        },
    ];
    assignment.carriers.sort();

    let case = assignment.case.clone();

    assert_eq!(
        validate_placement(&plans, &eligibility, &placement),
        Err(CompileError::NonCanonicalCarrierAssignment {
            relation: conservation,
            case,
        }),
    );
}

#[test]
fn an_at_least_one_obligation_may_not_carry_a_redundant_second_carrier() {
    let policy = relation(RelationKind::ProjectionPolicy, RelationSubject::Operation);
    let obligations = vec![Obligation::new(
        policy.clone(),
        SemanticScope::TransactionGlobal,
        CarrierMultiplicity::AtLeastOne,
        vec![
            eligible(
                global(ObjectId::ReceiptLive),
                CarrierQuantification::Single,
                Vec::new(),
            ),
            eligible(
                global(ObjectId::Ash),
                CarrierQuantification::Single,
                Vec::new(),
            ),
        ],
    )];
    let (plans, eligibility) = instance(&obligations);
    let placements = agree(&plans, &eligibility);

    // At-least-one retains the inclusion-minimal singletons, so the two
    // carriers are two placements rather than one doubled assignment.
    assert_eq!(placements.len(), 2);

    let mut placement = one_placement(&plans, &eligibility);
    let assignment = assignment_of(&mut placement, &policy);
    assignment.carriers = vec![
        PlacedCarrier {
            carrier: global(ObjectId::Ash),
            quantification: CarrierQuantification::Single,
        },
        PlacedCarrier {
            carrier: global(ObjectId::ReceiptLive),
            quantification: CarrierQuantification::Single,
        },
    ];
    assignment.carriers.sort();

    let case = assignment.case.clone();

    assert_eq!(
        validate_placement(&plans, &eligibility, &placement),
        Err(CompileError::NonCanonicalCarrierAssignment {
            relation: policy,
            case,
        }),
    );
}

#[test]
fn a_placement_stating_an_unrelated_layout_requirement_is_rejected() {
    let obligations = mandatory_family_and_coordinator();
    let (plans, eligibility) = instance(&obligations);
    let mut placement = one_placement(&plans, &eligibility);

    // No carrier this placement selected depends on the sponsor region
    // being isolated, so stating it would record an obligation against a
    // future backend that nothing here justified.
    let unrelated = LayoutRequirement::IsolateSponsorRegion {
        relation: family_relation(RelationKind::Recognition, ObjectId::PlainLbtc),
        case: case(SponsorCase::Present),
    };

    placement.layout_requirements.push(unrelated.clone());
    placement.layout_requirements.sort();

    assert_eq!(
        validate_placement(&plans, &eligibility, &placement),
        Err(CompileError::UnexpectedLayoutRequirement {
            unexpected: vec![unrelated],
        }),
    );
}

#[test]
fn a_placement_repeating_a_layout_requirement_is_rejected() {
    let obligations = mandatory_family_and_coordinator();
    let (plans, eligibility) = instance(&obligations);
    let mut placement = enumerate_feasible_placements(&plans, &eligibility, limits())
        .expect("placements")
        .candidates
        .into_iter()
        .find(|placement| !placement.layout_requirements.is_empty())
        .expect("a placement with a layout dependency");
    let repeated = placement.layout_requirements[0].clone();

    placement.layout_requirements.push(repeated.clone());
    placement.layout_requirements.sort();

    assert_eq!(
        validate_placement(&plans, &eligibility, &placement),
        Err(CompileError::UnexpectedLayoutRequirement {
            unexpected: vec![repeated],
        }),
    );
}

#[test]
fn a_placement_stating_another_relation_cases_layout_requirement_is_rejected() {
    let obligations = mandatory_family_and_coordinator();
    let (plans, eligibility) = instance(&obligations);
    let mut placement = one_placement(&plans, &eligibility);

    // The requirement is a real dependency of the *other* obligation's
    // eligible carrier, and it belongs to the operation-wide census
    // rather than to a placement that did not select that carrier.
    let foreign = eligibility
        .iter()
        .flat_map(|analysis| {
            analysis
                .eligible
                .iter()
                .flat_map(move |entry| oracle_layout(analysis, entry))
        })
        .find(|requirement| !placement.layout_requirements.contains(requirement))
        .expect("some eligible carrier depends on a requirement this placement omits");

    placement.layout_requirements.push(foreign.clone());
    placement.layout_requirements.sort();

    assert_eq!(
        validate_placement(&plans, &eligibility, &placement),
        Err(CompileError::UnexpectedLayoutRequirement {
            unexpected: vec![foreign],
        }),
    );
}
