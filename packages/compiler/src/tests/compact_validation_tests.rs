//! Exact compact comparisons over retained controls and single-coordinate changes.

use std::collections::{BTreeMap, BTreeSet};

use architecture::{
    AssetId, BoundId, DeltaKind, ObjectId, OpenFlowKind, OperationId, ProjectionId,
};
use petgraph::algo::has_path_connecting;
use petgraph::graph::DiGraph;
use petgraph::visit::EdgeRef as _;
use realization::{
    Count, LifecycleDependencyProjection, LifecycleEdge, LifecycleGraphProjection, LifecycleNode,
    LifecycleNodeId, ObservedAsset, ObservedCanonicalFlow, ObservedCanonicalPartition,
    ObservedObject, ObservedObjectKind, ObservedObjectRef, ObservedOpenFlow,
    ObservedOperationRegion, ObservedSide, ObservedValue, OperationObservation, OwnerId,
    ProtocolAmount, RealizationError, Relation, RelationFailure, RelationId, RelationKind,
    RelationStatus, RelationSubject, RelationVerdict, RepresentationMode,
};

use super::{announcement_input, bound_input, phase1_realization};
use crate::{
    BoundCompilerInput, CompileError, SponsorEnvelopeMultiplicityError,
    analyzed::analyze_scoped_program,
    lifecycle::{
        CompilerLifecycleAnalysis, LifecycleExitStatus, LifecycleRequirement, RepresentationChoice,
        RepresentationChoiceId, build_lifecycle_analysis, build_lifecycle_analysis_from_projection,
    },
    operation_plan::{
        ActivationCondition, CoverageBoundary, DischargeBoundary, EvidenceRole,
        PlacementSearchLimits, RelationActivity, RelationMutation, SponsorCase,
        TargetCoverageObligation, plan_compact_ash_target_operation,
        validate_compact_representation, validate_target_operation_plan,
    },
    relation::{CompilerRelationAnalysis, build_relation_analysis},
    validate_compact_lifecycle_projection, validate_sponsor_envelope_multiplicity,
};

fn limits() -> PlacementSearchLimits {
    PlacementSearchLimits::new(
        std::num::NonZeroU64::new(10_000_000).unwrap(),
        std::num::NonZeroU64::new(1_000_000).unwrap(),
    )
}

#[test]
fn explicit_control_passes_and_only_private_ash_selection_is_refused() {
    let input = bound_input(&[OperationId::CompactAsh]);
    let plan = plan_compact_ash_target_operation(&input, limits()).unwrap();
    let analyzed = analyze_scoped_program(&input, limits()).unwrap();
    assert_eq!(
        plan.representation().selection(ObjectId::Ash),
        Some(RepresentationMode::Explicit),
    );
    assert_eq!(validate_target_operation_plan(&analyzed, &plan), Ok(()));
    assert_eq!(
        validate_compact_representation(&input, limits(), &plan, RepresentationMode::Explicit),
        Ok(()),
    );
    let mut subject = plan.with_compact_representation(RepresentationMode::PrivateCommitted);
    let refusal = Err(CompileError::TargetPlanRepresentationMismatch {
        object: ObjectId::Ash,
    });
    assert_eq!(validate_target_operation_plan(&analyzed, &subject), refusal);
    assert_eq!(
        validate_compact_representation(
            &input,
            limits(),
            &plan,
            RepresentationMode::PrivateCommitted
        ),
        refusal,
    );
    assert_ne!(subject, plan);
    subject
        .representation_mut()
        .selected_mut()
        .insert(ObjectId::Ash, RepresentationMode::Explicit);
    assert_eq!(subject, plan);
    assert_eq!(validate_target_operation_plan(&analyzed, &subject), Ok(()));
}

#[test]
fn foreign_source_is_not_a_representation_refusal() {
    let input = bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let plan = plan_compact_ash_target_operation(&input, limits()).unwrap();
    let other_input = bound_input(&[OperationId::CompactAsh]);
    let other = plan_compact_ash_target_operation(&other_input, limits()).unwrap();
    let mut subject = plan.clone();
    *subject.source_mut() = other.source().clone();
    assert_eq!(
        validate_compact_representation(&input, limits(), &subject, RepresentationMode::Explicit),
        Err(CompileError::TargetPlanSourceMismatch),
    );
    *subject.source_mut() = plan.source().clone();
    assert_eq!(subject, plan);
    assert_eq!(
        validate_compact_representation(&input, limits(), &subject, RepresentationMode::Explicit),
        Ok(()),
    );
}

fn assert_missing_exit(exit: OperationId) {
    let input = bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let control = input.realization().project().lifecycle;
    assert_eq!(
        validate_compact_lifecycle_projection(&input, &control),
        Ok(())
    );
    let removed = LifecycleDependencyProjection {
        source: LifecycleNodeId::Representation {
            object: ObjectId::Ash,
            mode: RepresentationMode::Explicit,
        },
        target: LifecycleNodeId::RequiredExit {
            object: ObjectId::Ash,
            operation: exit,
        },
        edge: LifecycleEdge::RequiresExit,
    };
    let mut subject = control.clone();
    subject.edges.retain(|edge| edge != &removed);
    assert_eq!(subject.edges.len() + 1, control.edges.len());
    assert_eq!(subject.nodes, control.nodes);
    assert!(subject.nodes.iter().any(|node| node.id == removed.target));
    assert_eq!(
        subject.edges,
        control
            .edges
            .iter()
            .filter(|edge| *edge != &removed)
            .cloned()
            .collect::<Vec<_>>(),
    );
    assert_eq!(
        validate_compact_lifecycle_projection(&input, &subject),
        Err(CompileError::MissingLifecyclePath {
            object: ObjectId::Ash,
            representation: RepresentationMode::Explicit,
            exit,
        }),
    );
    subject.edges.push(removed);
    subject.edges.sort();
    assert_eq!(subject, control);
    assert_eq!(
        validate_compact_lifecycle_projection(&input, &subject),
        Ok(())
    );
}

#[test]
fn removing_only_the_explicit_compact_path_is_refused() {
    assert_missing_exit(OperationId::CompactAsh);
}

#[test]
fn removing_only_the_explicit_clear_path_is_refused() {
    assert_missing_exit(OperationId::Clear);
}

#[test]
fn retained_lifecycle_control_preserves_compact_and_clear_statuses() {
    let input = bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]);
    let projection = input.realization().project().lifecycle;
    assert_eq!(
        validate_compact_lifecycle_projection(&input, &projection),
        Ok(())
    );
    let relations = build_relation_analysis(&input).unwrap();
    let analysis =
        build_lifecycle_analysis_from_projection(&input, &relations, &projection).unwrap();
    for representation in [
        RepresentationMode::Explicit,
        RepresentationMode::PublicCommitted,
    ] {
        for (exit, status) in [
            (
                OperationId::CompactAsh,
                LifecycleExitStatus::AvailableInCompilerScope,
            ),
            (
                OperationId::Clear,
                LifecycleExitStatus::DeclaredOutsideCompilerScope,
            ),
        ] {
            assert!(analysis.requirements.contains(&LifecycleRequirement {
                object: ObjectId::Ash,
                representation,
                exit,
                status,
            }));
        }
    }
    assert!(!analysis.is_deployment_complete());
}

#[test]
fn plan_lifecycle_status_is_not_a_missing_path_refusal() {
    let input = bound_input(&[OperationId::CompactAsh]);
    let plan = plan_compact_ash_target_operation(&input, limits()).unwrap();
    let analyzed = analyze_scoped_program(&input, limits()).unwrap();
    assert_eq!(validate_target_operation_plan(&analyzed, &plan), Ok(()));
    let mut subject = plan.clone();
    subject.lifecycle_mut().clear_outstanding();
    assert_eq!(
        validate_target_operation_plan(&analyzed, &subject),
        Err(CompileError::TargetPlanLifecycleMismatch),
    );
    *subject.lifecycle_mut() = plan.lifecycle().clone();
    assert_eq!(subject, plan);
    assert_eq!(validate_target_operation_plan(&analyzed, &subject), Ok(()));
}

fn graph_projection(analysis: &CompilerLifecycleAnalysis) -> LifecycleGraphProjection {
    let mut nodes = analysis.graph.node_weights().cloned().collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.id.cmp(&right.id));
    let mut edges = analysis
        .graph
        .edge_references()
        .map(|edge| LifecycleDependencyProjection {
            source: analysis.graph[edge.source()].id.clone(),
            target: analysis.graph[edge.target()].id.clone(),
            edge: *edge.weight(),
        })
        .collect::<Vec<_>>();
    edges.sort();
    LifecycleGraphProjection { nodes, edges }
}

fn assert_equivalent(left: &CompilerLifecycleAnalysis, right: &CompilerLifecycleAnalysis) {
    assert_eq!(left.choices, right.choices);
    assert_eq!(left.requirements, right.requirements);
    assert_eq!(left.node_by_id, right.node_by_id);
    assert_eq!(graph_projection(left), graph_projection(right));
    assert_eq!(
        left.is_deployment_complete(),
        right.is_deployment_complete()
    );
}

#[test]
fn retained_lifecycle_routes_equal_the_pre_extraction_traversal() {
    for input in [
        bound_input(&[OperationId::CompactAsh]),
        bound_input(&[OperationId::TransferLive]),
        bound_input(&[OperationId::CompactAsh, OperationId::TransferLive]),
        announcement_input(),
    ] {
        let relations = build_relation_analysis(&input).unwrap();
        let projection = input.realization().project().lifecycle;
        let old_route = build_lifecycle_analysis(&input, &relations).unwrap();
        let offered_route =
            build_lifecycle_analysis_from_projection(&input, &relations, &projection).unwrap();
        let reference = pre_extraction_lifecycle(&input, &relations).unwrap();
        assert_equivalent(&old_route, &offered_route);
        assert_equivalent(&old_route, &reference);
        assert_equivalent(&offered_route, &reference);
        assert_eq!(
            validate_compact_lifecycle_projection(&input, &projection),
            Ok(())
        );
    }
}

const SPONSOR: OwnerId = OwnerId([3_u8; 32]);

fn ash(side: ObservedSide, ordinal: u32, value: u64) -> ObservedObject {
    ObservedObject {
        reference: ObservedObjectRef { side, ordinal },
        kind: ObservedObjectKind::Declared(ObjectId::Ash),
        asset: ObservedAsset::Declared(AssetId::U),
        value: ObservedValue::Protocol(ProtocolAmount::new(value).unwrap()),
        owner: None,
        representation: RepresentationMode::Explicit,
    }
}

fn sponsor_member(side: ObservedSide, ordinal: u32) -> ObservedObject {
    ObservedObject {
        reference: ObservedObjectRef { side, ordinal },
        kind: ObservedObjectKind::Declared(ObjectId::PlainLbtc),
        asset: ObservedAsset::Declared(AssetId::Lbtc),
        value: ObservedValue::SponsorOpaque,
        owner: Some(SPONSOR),
        representation: RepresentationMode::Explicit,
    }
}

fn valid_observation() -> OperationObservation {
    let input0 = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 0,
    };
    let input1 = ObservedObjectRef {
        side: ObservedSide::Input,
        ordinal: 1,
    };
    let output0 = ObservedObjectRef {
        side: ObservedSide::Output,
        ordinal: 0,
    };
    let objects = vec![
        ash(ObservedSide::Input, 0, 40),
        ash(ObservedSide::Input, 1, 60),
        ash(ObservedSide::Output, 0, 100),
    ];
    OperationObservation {
        operation: OperationId::CompactAsh,
        protocol_signers: BTreeSet::new(),
        sponsor_signers: BTreeSet::new(),
        region: ObservedOperationRegion {
            members: objects.iter().map(|object| object.reference).collect(),
            canonical_partition: ObservedCanonicalPartition {
                issuances: Vec::new(),
                flows: vec![ObservedCanonicalFlow {
                    asset: AssetId::U,
                    sources: vec![input0, input1],
                    destinations: vec![output0],
                    movement_kind: Some(DeltaKind::OwnerlessLateral),
                    destructions: Vec::new(),
                }],
            },
            open_flows: Vec::new(),
        },
        other_regions: Vec::new(),
        objects,
        root_effects: Vec::new(),
        projections: BTreeSet::from([ProjectionId::TransitionCertificate]),
        bounds: BTreeMap::from([
            (BoundId::AshBatchMax, Count::new(64)),
            (BoundId::FeeSponsorInputMax, Count::new(16)),
        ]),
    }
}

fn valid_sponsored_observation() -> OperationObservation {
    let mut observation = valid_observation();
    let input = sponsor_member(ObservedSide::Input, 2);
    let output = sponsor_member(ObservedSide::Output, 1);
    let envelope = ObservedOpenFlow {
        kind: OpenFlowKind::FeeSponsor,
        sources: vec![input.reference],
        destinations: vec![output.reference],
        fee: ProtocolAmount::new(3).unwrap(),
    };
    observation.claim_object(input);
    observation.claim_object(output);
    observation.region.open_flows.push(envelope);
    observation.sponsor_signers.insert(SPONSOR);
    observation
}

fn sponsor_relation(kind: RelationKind) -> RelationId {
    RelationId::new(OperationId::CompactAsh, kind, RelationSubject::Sponsor)
}

#[test]
fn another_operations_observation_has_no_multiplicity_declaration() {
    let realization = phase1_realization();
    let mut offered = valid_observation();
    offered.operation = OperationId::AnnounceMaturity;
    assert_eq!(
        validate_sponsor_envelope_multiplicity(&realization, &offered),
        Err(SponsorEnvelopeMultiplicityError::MissingDeclaration {
            operation: OperationId::AnnounceMaturity,
        }),
    );
}

#[test]
fn zero_and_one_owned_envelope_pass_but_two_are_refused() {
    let realization = phase1_realization();
    let zero = valid_observation();
    let control = valid_sponsored_observation();
    for offer in [&zero, &control] {
        assert!(
            realization
                .evaluate_operation(offer)
                .unwrap()
                .is_conformant()
        );
        assert_eq!(
            validate_sponsor_envelope_multiplicity(&realization, offer),
            Ok(())
        );
    }
    let relation = sponsor_relation(RelationKind::SponsorEnvelopeMultiplicity);
    assert_eq!(
        realization.relation(&relation).unwrap().relation,
        Relation::SponsorEnvelopeMultiplicity {
            maximum: Count::ONE
        },
    );
    let mut subject = control.clone();
    let extra = ObservedOpenFlow {
        kind: OpenFlowKind::FeeSponsor,
        sources: Vec::new(),
        destinations: Vec::new(),
        fee: ProtocolAmount::ZERO,
    };
    subject.region.open_flows.push(extra.clone());
    assert_eq!(subject.region.open_flows.len(), 2);
    assert_eq!(subject.objects, control.objects);
    assert_eq!(
        validate_sponsor_envelope_multiplicity(&realization, &subject),
        Err(SponsorEnvelopeMultiplicityError::Refused(RelationVerdict {
            relation,
            status: RelationStatus::Failed {
                reason: RelationFailure::SponsorEnvelopeMultiplicity
            },
        })),
    );
    assert_eq!(subject.region.open_flows.pop(), Some(extra));
    assert_eq!(subject, control);
    assert_eq!(
        validate_sponsor_envelope_multiplicity(&realization, &subject),
        Ok(())
    );
}

#[test]
fn overlapping_envelope_is_an_evaluation_error_not_multiplicity_evidence() {
    let realization = phase1_realization();
    let control = valid_sponsored_observation();
    assert_eq!(
        validate_sponsor_envelope_multiplicity(&realization, &control),
        Ok(())
    );
    let mut subject = control.clone();
    let envelope = control.region.open_flows[0].clone();
    subject.region.open_flows.push(envelope.clone());
    assert_eq!(
        validate_sponsor_envelope_multiplicity(&realization, &subject),
        Err(SponsorEnvelopeMultiplicityError::Evaluation(
            RealizationError::ObservedOpenFlowOverlap(envelope.sources[0]),
        )),
    );
    assert_eq!(subject.region.open_flows.pop(), Some(envelope));
    assert_eq!(subject, control);
    assert_eq!(
        validate_sponsor_envelope_multiplicity(&realization, &subject),
        Ok(())
    );
}

#[test]
fn isolation_failure_is_not_relabelled_as_multiplicity() {
    let realization = phase1_realization();
    let control = valid_sponsored_observation();
    assert_eq!(
        validate_sponsor_envelope_multiplicity(&realization, &control),
        Ok(())
    );
    let mut subject = control.clone();
    subject.sponsor_signers.clear();
    let report = realization.evaluate_operation(&subject).unwrap();
    assert_eq!(
        report
            .verdict(&sponsor_relation(RelationKind::SponsorIsolation))
            .unwrap()
            .status,
        RelationStatus::Failed {
            reason: RelationFailure::SponsorIsolation
        },
    );
    assert_eq!(
        validate_sponsor_envelope_multiplicity(&realization, &subject),
        Ok(())
    );
    subject.sponsor_signers = control.sponsor_signers.clone();
    assert_eq!(subject, control);
    assert!(
        realization
            .evaluate_operation(&subject)
            .unwrap()
            .is_conformant()
    );
    assert_eq!(
        validate_sponsor_envelope_multiplicity(&realization, &subject),
        Ok(())
    );
}

#[test]
fn multiplicity_remains_unconditional_model_scope_for_both_sponsor_identities() {
    let input = bound_input(&[OperationId::CompactAsh]);
    let plan = plan_compact_ash_target_operation(&input, limits()).unwrap();
    let relation = sponsor_relation(RelationKind::SponsorEnvelopeMultiplicity);
    let cases = &plan.relation(&relation).unwrap().cases;
    let mut sponsors = BTreeSet::new();
    for row in plan.coverage() {
        let TargetCoverageObligation::Negative(negative) = &row.obligation else {
            continue;
        };
        if negative.mutation != RelationMutation::SponsorEnvelopeMultiplicityExceeded {
            continue;
        }
        assert_eq!(row.id.relation, relation);
        assert_eq!(row.id.boundary, CoverageBoundary::ModelScope);
        assert_eq!(row.role, EvidenceRole::ModelScope);
        assert_eq!(row.activation, ActivationCondition::Always);
        assert_eq!(row.activity, RelationActivity::Active);
        assert!(row.carrier.is_empty());
        let case = &cases[&row.id.case];
        assert_eq!(
            case.boundaries,
            BTreeSet::from([DischargeBoundary::ModelScope])
        );
        assert_eq!(case.activation, ActivationCondition::Always);
        assert!(sponsors.insert(row.id.case.sponsor));
    }
    assert_eq!(
        sponsors,
        BTreeSet::from([SponsorCase::Absent, SponsorCase::Present])
    );
}

// Retained pre-extraction traversal: an independent reference for the shared
// production body, with its original projection acquisition and iteration order.
fn pre_extraction_lifecycle(
    input: &BoundCompilerInput,
    relations: &CompilerRelationAnalysis,
) -> Result<CompilerLifecycleAnalysis, CompileError> {
    let projection = input.realization().project().lifecycle;

    // Representation variables from the exact representation relations.
    let mut choices = Vec::new();
    let mut choice_objects = BTreeSet::new();

    for node in relations.graph.node_weights() {
        if let Relation::Representation { object, allowed } = &node.source.relation {
            let mut candidates = allowed.iter().copied().collect::<Vec<_>>();
            candidates.sort();

            choice_objects.insert(*object);
            choices.push(RepresentationChoice {
                id: RepresentationChoiceId {
                    operation: node.source.id.operation(),
                    object: *object,
                },
                candidates,
            });
        }
    }

    choices.sort_by(|left, right| left.id.cmp(&right.id));

    // Every in-scope lifecycle-exit relation needs a representation
    // variable for its object.
    for node in relations.graph.node_weights() {
        if let Relation::LifecycleExit { object, .. } = &node.source.relation
            && !choice_objects.contains(object)
        {
            return Err(CompileError::MissingRepresentationChoice {
                operation: node.source.id.operation(),
                object: *object,
            });
        }
    }

    // Rebuild the lifecycle graph, restricted to the chosen objects.
    let nodes = projection
        .nodes
        .iter()
        .filter(|node| choice_objects.contains(&lifecycle_object(&node.id)))
        .cloned()
        .collect::<Vec<_>>();
    let mut graph = DiGraph::<LifecycleNode, LifecycleEdge, u32>::with_capacity(
        nodes.len(),
        projection.edges.len(),
    );
    let mut node_by_id = BTreeMap::new();

    for node_weight in nodes {
        let id = node_weight.id.clone();
        let node = graph.add_node(node_weight);
        node_by_id.insert(id, node);
    }

    for edge in &projection.edges {
        let (Some(source), Some(target)) = (
            node_by_id.get(&edge.source).copied(),
            node_by_id.get(&edge.target).copied(),
        ) else {
            continue;
        };

        graph.add_edge(source, target, edge.edge);
    }

    // Every supported representation must reach every required exit of
    // its object; the exit's scope membership decides the status.
    let mut requirements = Vec::new();

    for (id, node) in &node_by_id {
        let LifecycleNodeId::Representation { object, mode } = id else {
            continue;
        };

        for (exit_id, exit_node) in &node_by_id {
            let LifecycleNodeId::RequiredExit {
                object: exit_object,
                operation,
            } = exit_id
            else {
                continue;
            };

            if exit_object != object {
                continue;
            }

            if !has_path_connecting(&graph, *node, *exit_node, None) {
                return Err(CompileError::MissingLifecyclePath {
                    object: *object,
                    representation: *mode,
                    exit: *operation,
                });
            }

            let status = if input.scope().operations().contains(operation) {
                LifecycleExitStatus::AvailableInCompilerScope
            } else {
                LifecycleExitStatus::DeclaredOutsideCompilerScope
            };

            requirements.push(LifecycleRequirement {
                object: *object,
                representation: *mode,
                exit: *operation,
                status,
            });
        }
    }

    requirements.sort();

    Ok(CompilerLifecycleAnalysis {
        graph,
        node_by_id,
        choices,
        requirements,
    })
}

const fn lifecycle_object(node: &LifecycleNodeId) -> ObjectId {
    match node {
        LifecycleNodeId::Representation { object, .. }
        | LifecycleNodeId::RequiredExit { object, .. } => *object,
    }
}
