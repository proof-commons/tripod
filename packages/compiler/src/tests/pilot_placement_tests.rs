//! End-to-end pilot placement analysis (Guide-5 §16, Tranche G).
//!
//! These tests run the real Phase-1 pilots through the whole pipeline —
//! feasible proof plans, execution cases, relation discharge
//! classification, carrier eligibility, exact placement, and layout
//! requirements — over the *complete* feasible plan set rather than one
//! sampled candidate, and assert the censuses exactly.
//!
//! A multi-operation scope is analyzed per operation for placement. The
//! feasible placement set of one plan is a product across that plan's
//! relation-cases, and the relation-cases of two operations are
//! independent, so a two-operation scope's placement set is the product
//! of the two operations' sets: complete, but exponential in the number
//! of operations and carrying no information the factors do not already
//! carry. The combined scope is still analyzed end to end below, and the
//! product structure is asserted rather than assumed.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{ObjectId, OperationId};
use realization::{Relation, RelationId, RelationKind, RepresentationMode, TransactionSide};

use super::bound_input;
use crate::{
    CompileError,
    capability::{CapabilityView, RequiredCapability},
    carrier::{
        CarrierEligibility, CarrierQuantification, CarrierRole, coordinator_anchors,
        is_sponsor_region_carrier, relation_case_eligibility,
    },
    case::{ExecutionCaseId, SponsorCase, is_sponsor_object},
    layout::{LayoutRequirement, layout_requirements, names_sponsor_amount},
    placement::{
        ActivationCondition, DischargeBoundary, PlacedCarrier, PlacedProofPlanCandidate,
        PlacedProofPlans, PlacementCandidate, PlacementSearchLimits, RelationActivity,
        RelationCasePlan, classify_relation_cases, place_feasible_proof_plans, validate_placement,
    },
    proof::{ProofPlanCandidate, enumerate_feasible_plans},
    relation::{CompilerRelationAnalysis, build_relation_analysis, build_relation_graph},
};

/// Generous limits: the pilot searches must run to completion, so a
/// limit that truncated one would hide a defect rather than bound it.
fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(10_000_000).expect("nonzero"),
        std::num::NonZeroU64::new(1_000_000).expect("nonzero"),
    )
}

/// One pilot scope analyzed end to end.
struct Analysis {
    operations: Vec<OperationId>,
    relations: CompilerRelationAnalysis,
    candidates: Vec<ProofPlanCandidate>,
    placed: PlacedProofPlans,
}

/// The relations and feasible proof plans of one scope, without
/// placement — the stages a combined scope can afford to run whole.
fn planned(operations: &[OperationId]) -> (CompilerRelationAnalysis, Vec<ProofPlanCandidate>) {
    let input = bound_input(operations);
    let relations = build_relation_analysis(&input).expect("relations");
    let candidates = enumerate_feasible_plans(&input, &CapabilityView::Unconstrained)
        .expect("feasible plans")
        .candidates;

    assert!(!candidates.is_empty());
    (relations, candidates)
}

fn analyze(operations: &[OperationId]) -> Analysis {
    let (relations, candidates) = planned(operations);
    let placed =
        place_feasible_proof_plans(&relations, &candidates, limits()).expect("placement analysis");

    Analysis {
        operations: operations.to_vec(),
        relations,
        candidates,
        placed,
    }
}

fn pilots() -> [Analysis; 2] {
    [
        analyze(&[OperationId::CompactAsh]),
        analyze(&[OperationId::TransferLive]),
    ]
}

/// The object family each pilot's protocol relations range over.
const fn protocol_object(operation: OperationId) -> ObjectId {
    match operation {
        OperationId::TransferLive => ObjectId::ReceiptLive,
        _ => ObjectId::Ash,
    }
}

impl Analysis {
    fn operation(&self) -> OperationId {
        *self.operations.first().expect("one operation")
    }

    fn object(&self) -> ObjectId {
        protocol_object(self.operation())
    }

    /// Every relation the compiler holds in scope.
    fn relation_ids(&self) -> BTreeSet<RelationId> {
        self.relations
            .graph
            .node_weights()
            .map(|node| node.source.id.clone())
            .collect()
    }

    /// The declared relation behind one ID.
    fn declaration(&self, relation: &RelationId) -> Relation {
        self.relations
            .graph
            .node_weights()
            .find(|node| &node.source.id == relation)
            .expect("relation is in scope")
            .source
            .relation
            .clone()
    }

    /// The one relation of one kind in one operation.
    fn relation_of_kind(&self, operation: OperationId, kind: RelationKind) -> RelationId {
        let mut found = self
            .relation_ids()
            .into_iter()
            .filter(|relation| relation.operation() == operation && relation.kind() == kind)
            .collect::<Vec<_>>();

        assert_eq!(found.len(), 1, "{operation:?} declares one {kind:?}");
        found.pop().expect("one relation")
    }

    /// Every relation-case plan across the whole placed plan set.
    fn plans(&self) -> impl Iterator<Item = &RelationCasePlan> {
        self.placed
            .placed
            .iter()
            .flat_map(|entry| entry.relation_case_plans.iter())
    }

    /// Every feasible placement across the whole placed plan set, with
    /// the placed candidate it belongs to.
    fn placements(&self) -> impl Iterator<Item = (&PlacedProofPlanCandidate, &PlacementCandidate)> {
        self.placed.placed.iter().flat_map(|entry| {
            entry
                .feasible_placements
                .iter()
                .map(move |placement| (entry, placement))
        })
    }

    /// Every layout requirement the analysis states, per-candidate
    /// census and per-placement dependency alike.
    fn all_layout_requirements(&self) -> BTreeSet<&LayoutRequirement> {
        self.placed
            .placed
            .iter()
            .flat_map(|entry| {
                entry.layout_requirements.iter().chain(
                    entry
                        .feasible_placements
                        .iter()
                        .flat_map(|placement| placement.layout_requirements.iter()),
                )
            })
            .collect()
    }
}

/// The relation-case plan behind one placed assignment.
fn plan_of<'a>(
    entry: &'a PlacedProofPlanCandidate,
    relation: &RelationId,
    case: &ExecutionCaseId,
) -> &'a RelationCasePlan {
    entry
        .relation_case_plans
        .iter()
        .find(|plan| &plan.relation == relation && &plan.case == case)
        .expect("every placed relation-case is planned")
}

/// The eligible carrier sets behind one placed candidate.
fn eligibility_of(
    analysis: &Analysis,
    entry: &PlacedProofPlanCandidate,
) -> Vec<CarrierEligibility> {
    relation_case_eligibility(&analysis.relations, &entry.relation_case_plans).expect("eligibility")
}

// --- §16.1 case census ---

#[test]
fn each_pilot_has_exactly_four_semantic_execution_cases() {
    for analysis in pilots() {
        let operation = analysis.operation();
        let object = analysis.object();
        let census = analysis.placed.execution_case_census();

        assert_eq!(census.len(), 4, "{operation:?}");
        assert!(census.iter().all(|case| case.operation == operation));

        let sponsors = census
            .iter()
            .map(|case| case.sponsor)
            .collect::<BTreeSet<_>>();
        assert_eq!(
            sponsors,
            BTreeSet::from([SponsorCase::Absent, SponsorCase::Present]),
        );

        // Two representation families, each crossed with both sponsor
        // cases — never one family with four sponsor shapes.
        let representations = census
            .iter()
            .map(|case| case.representations.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(representations.len(), 2, "{operation:?}");

        let modes = census
            .iter()
            .map(|case| case.representations[&object])
            .collect::<BTreeSet<_>>();
        let expected = match operation {
            OperationId::TransferLive => BTreeSet::from([
                RepresentationMode::Explicit,
                RepresentationMode::PrivateCommitted,
            ]),
            _ => BTreeSet::from([
                RepresentationMode::Explicit,
                RepresentationMode::PublicCommitted,
            ]),
        };

        assert_eq!(modes, expected, "{operation:?}");
    }
}

#[test]
fn every_pilot_candidate_carries_exactly_two_sponsor_cases() {
    for analysis in pilots() {
        for entry in &analysis.placed.placed {
            assert_eq!(entry.execution_cases.len(), 2);

            let sponsors = entry
                .execution_cases
                .iter()
                .map(|case| case.id.sponsor)
                .collect::<BTreeSet<_>>();
            assert_eq!(
                sponsors,
                BTreeSet::from([SponsorCase::Absent, SponsorCase::Present]),
            );

            // The plan fixed the representation, so the two cases of one
            // candidate differ in the sponsor dimension alone.
            let representations = entry
                .execution_cases
                .iter()
                .map(|case| case.id.representations.clone())
                .collect::<BTreeSet<_>>();
            assert_eq!(representations.len(), 1);
        }
    }
}

// --- §16.2 relation-case census ---

#[test]
fn every_candidate_plans_every_relation_in_every_applicable_case() {
    for analysis in pilots() {
        let relations = analysis.relation_ids();

        for entry in &analysis.placed.placed {
            let expected = relations
                .iter()
                .flat_map(|relation| {
                    entry
                        .execution_cases
                        .iter()
                        .map(move |case| (relation.clone(), case.id.clone()))
                })
                .collect::<BTreeSet<_>>();
            let planned = entry
                .relation_case_plans
                .iter()
                .map(|plan| (plan.relation.clone(), plan.case.clone()))
                .collect::<BTreeSet<_>>();

            assert_eq!(planned, expected, "{:?}", analysis.operation());
            assert_eq!(entry.relation_case_plans.len(), expected.len());
        }
    }
}

#[test]
fn every_discharge_disposition_is_represented_in_each_pilot() {
    for analysis in pilots() {
        let operation = analysis.operation();
        let mut vacuous = 0_usize;
        let mut boundaries = BTreeSet::new();

        for plan in analysis.plans() {
            if plan.activity == RelationActivity::Vacuous {
                vacuous += 1;
                assert!(plan.runtime_requirements.is_empty());
            }

            boundaries.extend(plan.boundaries.iter().copied());
        }

        // Vacuous is a stated disposition, not an omission.
        assert!(vacuous > 0, "{operation:?}");
        assert_eq!(
            boundaries,
            BTreeSet::from([
                DischargeBoundary::CompilerStatic,
                DischargeBoundary::BackendStructural,
                DischargeBoundary::RuntimeCarrier,
                DischargeBoundary::ExternalEvidence,
            ]),
            "{operation:?}",
        );
    }
}

// --- §16.3 mandatory coordinator ---

#[test]
fn each_pilot_anchors_its_coordinator_in_its_own_input_family() {
    for analysis in pilots() {
        let operation = analysis.operation();
        let object = analysis.object();

        assert_eq!(
            coordinator_anchors(&analysis.relations, operation),
            BTreeSet::from([object]),
            "{operation:?}",
        );

        let mut coordinators = 0_usize;

        for (_, placement) in analysis.placements() {
            for assignment in &placement.assignments {
                for placed in &assignment.carriers {
                    let CarrierRole::OperationGlobal { anchor, .. } = placed.carrier else {
                        continue;
                    };

                    coordinators += 1;
                    assert_eq!(anchor, object, "{operation:?}");
                    assert!(!is_sponsor_object(anchor));
                }
            }
        }

        assert!(coordinators > 0, "{operation:?}");
    }
}

#[test]
fn no_unconditional_relation_is_placed_on_the_optional_sponsor_region() {
    let mut sponsor_carriers = 0_usize;

    for analysis in pilots() {
        let operation = analysis.operation();

        for (entry, placement) in analysis.placements() {
            for assignment in &placement.assignments {
                let plan = plan_of(entry, &assignment.relation, &assignment.case);

                for placed in &assignment.carriers {
                    if !is_sponsor_region_carrier(&placed.carrier) {
                        continue;
                    }

                    sponsor_carriers += 1;
                    assert_eq!(
                        plan.activation,
                        ActivationCondition::WhenSponsorPresent,
                        "{:?} in {operation:?}",
                        assignment.relation,
                    );
                    assert_eq!(plan.case.sponsor, SponsorCase::Present);
                }
            }
        }
    }

    // The restriction above is not vacuous: both pilots really do place
    // their own sponsor relations inside the sponsor region. Compact ASH
    // joined live transfer here once its sponsor cardinality relations
    // were declared (Guide-6 §4), which is what makes the sponsor family
    // one of its declared input families and offers a member-anchored
    // carrier at all.
    assert!(sponsor_carriers > 0);
}

#[test]
fn sponsor_family_cardinality_is_vacuous_unsponsored_and_active_when_sponsored() {
    // Both pilots now own the architecture's sponsor cardinalities, and
    // an optional family is never dropped from a case: it is stated
    // vacuous in the sponsorless case and active in the sponsored one.
    for analysis in pilots() {
        let operation = analysis.operation();

        for side in [TransactionSide::Input, TransactionSide::Output] {
            let relation = RelationId::new(
                operation,
                RelationKind::Cardinality,
                realization::RelationSubject::ObjectFamily {
                    side,
                    object: ObjectId::PlainLbtc,
                },
            );

            assert!(
                analysis.relation_ids().contains(&relation),
                "{operation:?} declares {side:?} sponsor cardinality",
            );

            let mut observed = BTreeMap::new();

            for plan in analysis.plans().filter(|plan| plan.relation == relation) {
                assert_eq!(plan.activation, ActivationCondition::WhenSponsorPresent);

                let expected = match plan.case.sponsor {
                    SponsorCase::Absent => RelationActivity::Vacuous,
                    SponsorCase::Present => RelationActivity::Active,
                };

                assert_eq!(plan.activity, expected, "{operation:?} {side:?}");

                if plan.case.sponsor == SponsorCase::Absent {
                    assert!(plan.runtime_requirements.is_empty());
                } else {
                    assert_eq!(plan.runtime_requirements.len(), 1);
                }

                observed.insert(plan.case.sponsor, plan.activity);
            }

            assert_eq!(
                observed.keys().copied().collect::<BTreeSet<_>>(),
                BTreeSet::from([SponsorCase::Absent, SponsorCase::Present]),
                "{operation:?} {side:?}",
            );
        }
    }
}

#[test]
fn sponsor_cardinality_operands_name_counts_and_bounds_but_no_amount() {
    // The repaired relations must expose the sponsor family's *census*,
    // never its value: a sponsor amount operand here would breach the
    // erasure boundary the whole pipeline maintains.
    for analysis in pilots() {
        let operation = analysis.operation();

        for side in [TransactionSide::Input, TransactionSide::Output] {
            let relation = RelationId::new(
                operation,
                RelationKind::Cardinality,
                realization::RelationSubject::ObjectFamily {
                    side,
                    object: ObjectId::PlainLbtc,
                },
            );
            let declaration = analysis
                .relations
                .graph
                .node_weights()
                .find(|node| node.source.id == relation)
                .expect("sponsor cardinality is in scope")
                .source
                .clone();
            let operands = crate::source::relation_operands(&declaration).expect("operands");

            assert!(!operands.is_empty());
            for operand in &operands {
                assert!(
                    !crate::source::is_sponsor_amount_operand(operand.role()),
                    "{operand:?}",
                );
            }
        }
    }
}

// --- §16.4 local owner authorization ---

#[test]
fn live_owner_authorization_is_placed_per_member_in_every_candidate_and_case() {
    let analysis = analyze(&[OperationId::TransferLive]);
    let relation =
        analysis.relation_of_kind(OperationId::TransferLive, RelationKind::Authorization);
    let expected = vec![PlacedCarrier {
        carrier: CarrierRole::EveryInputFamilyMember {
            object: ObjectId::ReceiptLive,
        },
        quantification: CarrierQuantification::PerMember,
    }];

    for (entry, placement) in analysis.placements() {
        let placed = placement
            .assignments
            .iter()
            .filter(|assignment| assignment.relation == relation)
            .collect::<Vec<_>>();

        // Once per execution case, and never replaced by a coordinator
        // standing in for every owner.
        assert_eq!(placed.len(), entry.execution_cases.len());

        for assignment in placed {
            assert_eq!(assignment.carriers, expected);
        }
    }
}

// --- §16.5 global conservation ---

/// The amount-conservation relation of one pilot, with the families it
/// ranges over.
fn conservation(analysis: &Analysis) -> (RelationId, BTreeSet<(TransactionSide, ObjectId)>) {
    let relation = analysis.relation_of_kind(analysis.operation(), RelationKind::Conservation);
    let Relation::AmountConservation {
        input_objects,
        output_objects,
        ..
    } = analysis.declaration(&relation)
    else {
        panic!("the conservation relation conserves amounts");
    };

    let families = input_objects
        .iter()
        .map(|object| (TransactionSide::Input, *object))
        .chain(
            output_objects
                .iter()
                .map(|object| (TransactionSide::Output, *object)),
        )
        .collect();

    (relation, families)
}

#[test]
fn amount_conservation_is_placed_on_one_complete_global_carrier() {
    for analysis in pilots() {
        let operation = analysis.operation();
        let (relation, families) = conservation(&analysis);

        for (_, placement) in analysis.placements() {
            let assignment = placement
                .assignments
                .iter()
                .find(|assignment| assignment.relation == relation)
                .expect("conservation is placed");

            assert_eq!(assignment.carriers.len(), 1, "{operation:?}");

            let placed = &assignment.carriers[0];
            assert!(
                matches!(placed.carrier, CarrierRole::OperationGlobal { .. }),
                "{operation:?} placed conservation on {:?}",
                placed.carrier,
            );
            assert_eq!(placed.quantification, CarrierQuantification::Single);
        }

        // The global carrier depends on authenticated totals for every
        // family the relation conserves, on both sides.
        let stated = analysis.all_layout_requirements();

        for (side, object) in families {
            assert!(
                stated.contains(&LayoutRequirement::AuthenticateFamilyCensus {
                    relation: relation.clone(),
                    side,
                    object,
                }),
                "{operation:?} {side:?} {object:?}",
            );
        }

        for side in [TransactionSide::Input, TransactionSide::Output] {
            assert!(
                stated.contains(&LayoutRequirement::CompleteAndDisjointFamilies {
                    relation: relation.clone(),
                    side,
                })
            );
        }
    }
}

#[test]
fn a_per_member_conservation_placement_is_rejected() {
    for analysis in pilots() {
        let (relation, _) = conservation(&analysis);
        let entry = analysis.placed.placed.first().expect("a placed candidate");
        let eligibility = eligibility_of(&analysis, entry);
        let mut placement = entry
            .feasible_placements
            .first()
            .expect("a feasible placement")
            .clone();

        let assignment = placement
            .assignments
            .iter_mut()
            .find(|assignment| assignment.relation == relation)
            .expect("conservation is placed");
        let carrier = CarrierRole::EveryInputFamilyMember {
            object: analysis.object(),
        };
        assignment.carriers = vec![PlacedCarrier {
            carrier: carrier.clone(),
            quantification: CarrierQuantification::PerMember,
        }];

        let case = assignment.case.clone();

        assert_eq!(
            validate_placement(&entry.relation_case_plans, &eligibility, &placement),
            Err(CompileError::UnpermittedCarrierPlacement {
                relation,
                case,
                carrier,
            }),
            "{:?}",
            analysis.operation(),
        );
    }
}

// --- §16.6 external conservation ---

#[test]
fn substrate_conservation_stays_external_evidence_with_no_runtime_carrier() {
    for analysis in pilots() {
        let operation = analysis.operation();
        let relation = analysis.relation_of_kind(operation, RelationKind::SubstrateConservation);
        let mut planned = 0_usize;

        for plan in analysis.plans() {
            if plan.relation != relation {
                continue;
            }

            planned += 1;
            assert_eq!(
                plan.boundaries,
                BTreeSet::from([DischargeBoundary::ExternalEvidence]),
            );
            assert!(plan.runtime_requirements.is_empty());
            assert!(plan.compiler_requirements.is_empty());
            assert!(plan.structural_requirements.is_empty());
            assert!(!plan.external_evidence.is_empty());
        }

        assert!(planned > 0, "{operation:?}");

        // No placement carries it, and no layout requirement stands in
        // for the evidence it still needs.
        for (_, placement) in analysis.placements() {
            assert!(
                placement
                    .assignments
                    .iter()
                    .all(|assignment| assignment.relation != relation),
                "{operation:?}",
            );
        }

        for requirement in analysis.all_layout_requirements() {
            assert_ne!(requirement.relation(), Some(&relation), "{operation:?}");
        }

        // The capability the evidence depends on survives every plan.
        for candidate in &analysis.candidates {
            assert!(
                candidate
                    .required_capabilities
                    .contains(&RequiredCapability::WholeTransactionValueConservation),
                "{operation:?}",
            );
            assert!(!candidate.external_evidence.is_empty());
        }
    }
}

// --- §16.7 sponsor erasure ---

#[test]
fn no_pilot_placement_or_layout_requirement_names_a_sponsor_amount() {
    for analysis in pilots() {
        let operation = analysis.operation();
        let mut sponsor_requirements = 0_usize;

        for requirement in analysis.all_layout_requirements() {
            assert!(!names_sponsor_amount(requirement), "{requirement:?}");

            if matches!(requirement, LayoutRequirement::IsolateSponsorRegion { .. }) {
                sponsor_requirements += 1;
            }
        }

        // Region isolation is expressible; the sponsor's own amount is
        // not, and the pilots really do state the former.
        assert!(sponsor_requirements > 0, "{operation:?}");
    }
}

// --- complete-set properties ---

#[test]
fn every_feasible_proof_plan_yields_a_validated_feasible_placement_set() {
    for analysis in pilots() {
        let operation = analysis.operation();

        assert_eq!(analysis.placed.placed.len(), analysis.candidates.len());

        let offered = analysis.candidates.iter().cloned().collect::<BTreeSet<_>>();
        let placed = analysis
            .placed
            .placed
            .iter()
            .map(|entry| entry.proof_plan.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(placed, offered, "{operation:?}");

        for entry in &analysis.placed.placed {
            assert!(!entry.feasible_placements.is_empty(), "{operation:?}");

            // Canonical order, no duplicate, and no weighting.
            let mut canonical = entry.feasible_placements.clone();
            canonical.sort();
            canonical.dedup();
            assert_eq!(canonical, entry.feasible_placements);

            let eligibility = eligibility_of(&analysis, entry);

            for placement in &entry.feasible_placements {
                validate_placement(&entry.relation_case_plans, &eligibility, placement)
                    .expect("every retained placement satisfies every hard constraint");
            }

            // The layout census of this plan validates against the same
            // plans and carriers the placements were built from.
            let census = layout_requirements(
                &analysis.relations,
                &entry.relation_case_plans,
                &eligibility,
            )
            .expect("layout census");
            assert_eq!(census, entry.layout_requirements);
        }
    }
}

#[test]
fn a_duplicate_proof_plan_candidate_is_rejected() {
    let (relations, candidates) = planned(&[OperationId::CompactAsh]);
    let candidate = candidates.first().expect("a feasible candidate").clone();

    assert_eq!(
        place_feasible_proof_plans(&relations, &[candidate.clone(), candidate], limits()),
        Err(CompileError::DuplicatePlacedProofPlan),
    );
}

// --- §16.8 determinism ---

#[test]
fn repeated_pilot_analysis_is_equal_and_projects_equally() {
    for analysis in pilots() {
        let again = place_feasible_proof_plans(&analysis.relations, &analysis.candidates, limits())
            .expect("second analysis");

        assert_eq!(again, analysis.placed, "{:?}", analysis.operation());
        assert_eq!(again.project(), analysis.placed.project());
        assert_eq!(again.project().plans.len(), analysis.candidates.len());
    }
}

#[test]
fn a_permuted_proof_plan_candidate_order_is_analyzed_equally() {
    for analysis in pilots() {
        let mut permuted = analysis.candidates.clone();
        permuted.reverse();

        let again = place_feasible_proof_plans(&analysis.relations, &permuted, limits())
            .expect("permuted analysis");

        assert_eq!(again, analysis.placed, "{:?}", analysis.operation());
        assert_eq!(again.project(), analysis.placed.project());
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
fn permuted_relation_declarations_are_analyzed_equally() {
    for analysis in pilots() {
        let permuted = permuted_relations(&analysis.operations);
        let again = place_feasible_proof_plans(&permuted, &analysis.candidates, limits())
            .expect("permuted analysis");

        assert_eq!(again, analysis.placed, "{:?}", analysis.operation());
        assert_eq!(again.project(), analysis.placed.project());
    }
}

// --- the combined two-pilot scope ---

/// The combined scope's cases, relation-case plans, carrier eligibility,
/// and layout census, for every feasible plan.
fn combined_requirements(
    relations: &CompilerRelationAnalysis,
    candidates: &[ProofPlanCandidate],
) -> BTreeMap<ProofPlanCandidate, (Vec<RelationCasePlan>, Vec<LayoutRequirement>)> {
    candidates
        .iter()
        .map(|candidate| {
            let cases = crate::case::execution_cases(relations, candidate).expect("cases");
            let plans = classify_relation_cases(relations, &cases).expect("classification");
            let eligibility = relation_case_eligibility(relations, &plans).expect("eligibility");
            let requirements =
                layout_requirements(relations, &plans, &eligibility).expect("layout census");

            (candidate.clone(), (plans, requirements))
        })
        .collect()
}

#[test]
fn the_combined_pilot_scope_is_analyzed_deterministically() {
    let operations = [OperationId::CompactAsh, OperationId::TransferLive];
    let (relations, candidates) = planned(&operations);
    let first = combined_requirements(&relations, &candidates);

    // Both operations' cases, and both representation families of each.
    let cases = crate::case::case_census(&relations, &candidates).expect("case census");
    assert_eq!(cases.len(), 8);
    assert_eq!(
        cases
            .iter()
            .map(|case| case.operation)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(operations),
    );

    // Repeated analysis, a permuted candidate order, and permuted
    // relation declarations all agree.
    let mut permuted = candidates.clone();
    permuted.reverse();

    assert_eq!(combined_requirements(&relations, &candidates), first);
    assert_eq!(combined_requirements(&relations, &permuted), first);
    assert_eq!(
        combined_requirements(&permuted_relations(&operations), &candidates),
        first,
    );
}

#[test]
fn the_combined_scope_placement_set_is_the_product_of_its_operations() {
    let combined = analyze(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let factors = pilots();

    // Each operation's plans agree on their placement count here, so the
    // product is well defined without pairing plans across scopes.
    let sizes = |analysis: &Analysis| {
        analysis
            .placed
            .placed
            .iter()
            .map(|entry| entry.feasible_placements.len())
            .collect::<BTreeSet<_>>()
    };
    let factor_sizes = factors
        .iter()
        .map(|analysis| {
            let sizes = sizes(analysis);
            assert_eq!(sizes.len(), 1, "{:?}", analysis.operation());
            *sizes.iter().next().expect("one size")
        })
        .collect::<Vec<_>>();
    let expected = factor_sizes.iter().product::<usize>();

    assert_eq!(sizes(&combined), BTreeSet::from([expected]));

    // Every combined plan still has a complete, non-empty, canonical
    // placement set over both operations' cases.
    for entry in &combined.placed.placed {
        assert_eq!(entry.execution_cases.len(), 4);
        assert!(!entry.feasible_placements.is_empty());
    }
}
