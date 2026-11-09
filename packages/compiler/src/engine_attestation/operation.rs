//! Finite compatibility payloads; restoring a factor never calls an owner back.
use crate::{
    CompileError,
    analyzed_operation::{AnalyzedOperation, RelationCaseRequirements},
    coverage::{
        self, CoverageRequirementId, NegativeCoverageRequirement, PositiveCoverageRequirement,
    },
    coverage_graph::{CoverageDependency, CoverageGraphProjection, CoverageNode, CoverageNodeId},
    engine_coverage::{Purpose, Requirement},
    engine_coverage_graph::Key as Symbol,
    engine_placement::{Key, Placed},
    placement::{PlacedCarrier, RelationCaseKey, RelationCasePlan},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
pub struct Payload {
    plan: RelationCasePlan,
    eligibility: Vec<crate::carrier::CarrierEligibility>,
    positive: BTreeMap<Requirement, PositiveCoverageRequirement>,
    negative: BTreeMap<Requirement, NegativeCoverageRequirement>,
    projections: BTreeMap<coverage::CoverageBoundary, coverage::SemanticProjectionRequirement>,
    sources: BTreeMap<crate::engine_source::SourceRow, crate::source::SourceRequirement>,
    carriers: BTreeMap<Placed, PlacedCarrier>,
    evidence: BTreeMap<crate::engine_source::Evidence, realization::ExternalEvidenceRequirement>,
    compiler: BTreeMap<
        crate::engine_case_facts::CompilerRequirement,
        crate::placement::CompilerStaticRequirement,
    >,
    structural: BTreeMap<
        crate::engine_case_facts::StructuralRequirement,
        crate::placement::BackendStructuralRequirement,
    >,
}

impl Payload {
    pub fn new(
        plan: &RelationCasePlan,
        coverage: &coverage::RelationCoveragePlan,
        sources: &[crate::source::SourceRequirement],
        eligibility: &[crate::carrier::CarrierEligibility],
    ) -> Self {
        let span =
            tracing::debug_span!(target: "compiler::engine_attestation", "lower_operation_payload");
        let _entered = span.enter();
        let projections = coverage
            .boundaries
            .iter()
            .filter_map(|boundary| {
                coverage::projection_requirement(&coverage.key(), coverage, plan, *boundary)
                    .map(|row| (*boundary, row))
            })
            .collect::<BTreeMap<_, _>>();
        let carriers = eligibility
            .iter()
            .flat_map(|row| &row.eligible)
            .map(|row| {
                let value = PlacedCarrier {
                    carrier: row.carrier.clone(),
                    quantification: row.quantification,
                };
                (
                    Placed {
                        carrier: crate::engine_placement::CarrierKey::Attestation(
                            value.carrier.clone(),
                        ),
                        quantification: value.quantification,
                    },
                    value,
                )
            })
            .collect::<BTreeMap<_, _>>();
        let positive = positives(plan, coverage, &projections, &carriers);
        Self {
            plan: plan.clone(),
            eligibility: eligibility.to_vec(),
            positive,
            negative: coverage
                .negative
                .iter()
                .map(|row| (super::coverage::requirement(&row.id), row.clone()))
                .collect(),
            projections,
            carriers,
            sources: sources
                .iter()
                .map(|row| (super::source::row_key(row), row.clone()))
                .collect(),
            evidence: coverage
                .external_evidence
                .iter()
                .map(|row| (super::proof::evidence_key(row), row.clone()))
                .collect(),
            compiler: plan
                .compiler_requirements
                .iter()
                .map(|row| (super::case_facts::compiler(row), row.clone()))
                .collect(),
            structural: plan
                .structural_requirements
                .iter()
                .map(|row| (super::case_facts::structural(row), row.clone()))
                .collect(),
        }
    }
    fn key(&self) -> RelationCaseKey {
        RelationCaseKey {
            relation: self.plan.relation.clone(),
            case: self.plan.case.clone(),
        }
    }
}

pub type Rows = BTreeMap<(Key, Option<crate::engine_proof::AlternativeKey>), Payload>;

pub fn obligations(
    rows: &Rows,
    candidate: &crate::engine_proof::Candidate,
    factor: &crate::engine_operation::Operation,
) -> Result<crate::analyzed_validate::OperationObligations, CompileError> {
    let payloads = factor
        .bundles
        .keys()
        .map(|key| {
            rows.get(&(key.clone(), candidate.proofs.get(&key.relation).cloned()))
                .ok_or(CompileError::InvalidRealization)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(crate::analyzed_validate::OperationObligations {
        relation_case_plans: payloads.iter().map(|row| row.plan.clone()).collect(),
        eligibility: payloads
            .iter()
            .flat_map(|row| row.eligibility.iter().cloned())
            .collect(),
    })
}

pub fn refusal(
    rows: &Rows,
    layout: &BTreeSet<crate::engine_placement::LayoutKey>,
    error: crate::engine_operation::Error,
) -> CompileError {
    use crate::engine_operation::Error;
    let payloads = rows
        .iter()
        .map(|((key, _), payload)| (key.clone(), payload))
        .collect::<BTreeMap<_, _>>();
    let keys = payloads
        .iter()
        .map(|(key, payload)| (key.clone(), payload.key()))
        .collect::<BTreeMap<_, _>>();
    let carriers = payloads
        .values()
        .flat_map(|payload| {
            payload
                .carriers
                .iter()
                .map(|(key, value)| (key.carrier.clone(), value.carrier.clone()))
        })
        .collect();
    let ids = payloads
        .values()
        .flat_map(|payload| {
            payload
                .positive
                .iter()
                .map(|(key, row)| (key.clone(), row.id.clone()))
                .chain(
                    payload
                        .negative
                        .iter()
                        .map(|(key, row)| (key.clone(), row.id.clone())),
                )
        })
        .collect();
    match error {
        Error::Placement(error) => super::placement::refusal(*error, &keys, &carriers),
        Error::Coverage(error) => super::coverage::refusal(*error, &keys, &ids),
        Error::Join(error) => super::coverage_join::refusal(error, &keys),
        Error::Graph(error) => {
            crate::coverage_graph::refusal_from_identities(error, &symbols(&payloads, layout))
        }
        Error::Prerequisite(pair) => CompileError::CrossOperationCoverageDependency {
            prerequisite: Box::new(CoverageNodeId::RelationCase(keys[&pair.0].clone())),
            dependent: Box::new(CoverageNodeId::RelationCase(keys[&pair.1].clone())),
        },
        Error::CaseFacts { key, defect } => CompileError::InvalidRelationCaseRequirements {
            relation: keys[&key].relation.clone(),
            case: keys[&key].case.clone(),
            defect,
        },
        Error::CaseCensus {
            missing,
            unexpected,
        } => CompileError::RelationCaseCensusMismatch {
            missing: missing.iter().map(|key| keys[key].clone()).collect(),
            unexpected: unexpected.iter().map(|key| keys[key].clone()).collect(),
        },
    }
}

/// Select stored payloads by complete identifiers, then translate the concrete
/// result. No obligation, carrier or proof is selected by this compatibility view.
pub fn restore(
    rows: &Rows,
    candidate: &crate::engine_proof::Candidate,
    factor: &crate::engine_operation::Operation,
) -> Result<AnalyzedOperation, CompileError> {
    restore_selected(rows, &candidate.proofs, factor)
}
pub fn restore_selected(
    rows: &Rows,
    proofs: &BTreeMap<crate::engine_records::RelationKey, crate::engine_proof::AlternativeKey>,
    factor: &crate::engine_operation::Operation,
) -> Result<AnalyzedOperation, CompileError> {
    let span = tracing::debug_span!(target: "compiler::engine_attestation", "restore_operation", case_count = crate::instrumentation::count(factor.bundles.len()));
    let _entered = span.enter();
    let payloads = factor
        .bundles
        .keys()
        .map(|key| {
            let proof = proofs.get(&key.relation).cloned();
            rows.get(&(key.clone(), proof))
                .map(|payload| (key.clone(), payload))
                .ok_or(CompileError::InvalidRealization)
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let first = payloads
        .values()
        .next()
        .ok_or(CompileError::InvalidRealization)?;
    let operation = first.plan.case.operation;
    let coverage_dependencies = restore_graph(&payloads, factor)?;
    let mut relation_cases = BTreeMap::new();
    let mut requirements = BTreeMap::new();
    for (key, bundle) in &factor.bundles {
        let payload = payloads[key];
        let coverage = restore_coverage(payload, bundle, factor, &payloads)?;
        let value = restore_bundle(payload, bundle, &coverage)?;
        requirements.insert(payload.key(), coverage);
        relation_cases.insert(payload.key(), value);
    }
    let cases = payloads
        .values()
        .map(|row| row.plan.case.clone())
        .collect::<BTreeSet<_>>();
    let placements = factor
        .placements
        .iter()
        .map(|placement| {
            let assignments = placement
                .assignments
                .iter()
                .map(|assignment| {
                    let payload = payloads
                        .get(&assignment.key)
                        .ok_or(CompileError::InvalidRealization)?;
                    Ok(crate::placement::PlacementAssignment {
                        relation: payload.plan.relation.clone(),
                        case: payload.plan.case.clone(),
                        carriers: assignment
                            .carriers
                            .iter()
                            .map(|row| lookup(&payload.carriers, row))
                            .collect::<Result<_, _>>()?,
                    })
                })
                .collect::<Result<Vec<_>, CompileError>>()?;
            Ok(crate::placement::PlacementCandidate {
                assignments,
                layout_requirements: placement.layout.iter().map(|row| row.0.clone()).collect(),
            }
            .project())
        })
        .collect::<Result<_, CompileError>>()?;
    Ok(AnalyzedOperation {
        operation,
        execution_cases: cases.clone(),
        relation_cases,
        feasible_placements: placements,
        layout_requirements: factor.layout.iter().map(|row| row.0.clone()).collect(),
        coverage: coverage::OperationCoverageProjection {
            operation,
            cases,
            requirements,
        },
        coverage_dependencies,
    })
}

fn lookup<K: Ord, V: Clone>(values: &BTreeMap<K, V>, key: &K) -> Result<V, CompileError> {
    values
        .get(key)
        .cloned()
        .ok_or(CompileError::InvalidRealization)
}

fn symbols(
    payloads: &BTreeMap<Key, &Payload>,
    layout: &BTreeSet<crate::engine_placement::LayoutKey>,
) -> BTreeMap<Symbol, CoverageNodeId> {
    let mut ids = BTreeMap::new();
    for (key, payload) in payloads {
        ids.insert(
            Symbol::Relation(key.clone()),
            CoverageNodeId::RelationCase(payload.key()),
        );
        for (id, row) in &payload.positive {
            ids.insert(
                Symbol::Requirement(id.clone()),
                CoverageNodeId::Requirement(row.id.clone()),
            );
        }
        for (id, row) in &payload.negative {
            ids.insert(
                Symbol::Requirement(id.clone()),
                CoverageNodeId::Requirement(row.id.clone()),
            );
        }
        for (id, row) in &payload.evidence {
            ids.insert(
                Symbol::Evidence(id.clone()),
                CoverageNodeId::ExternalEvidence(row.clone()),
            );
        }
        for (id, row) in &payload.carriers {
            ids.insert(
                Symbol::Carrier {
                    key: key.clone(),
                    placed: id.clone(),
                },
                CoverageNodeId::Carrier {
                    relation_case: payload.key(),
                    carrier: row.clone(),
                },
            );
        }
    }
    for layout in layout {
        ids.insert(
            Symbol::Layout(layout.clone()),
            CoverageNodeId::Layout(layout.0.clone()),
        );
    }
    ids
}

fn restore_coverage(
    payload: &Payload,
    bundle: &crate::engine_case_facts::Bundle,
    factor: &crate::engine_operation::Operation,
    payloads: &BTreeMap<Key, &Payload>,
) -> Result<coverage::RelationCoverageProjection, CompileError> {
    let joined = &bundle.coverage;
    let mut negative = BTreeSet::new();
    for id in &joined.plan.negative {
        let mut row = lookup(&payload.negative, id)?;
        let targets = factor
            .coverage
            .edges
            .iter()
            .filter(|edge| {
                edge.edge == crate::coverage_graph::CoverageEdge::DependencyCollateral
                    && edge.source == Symbol::Requirement(id.clone())
            })
            .map(|edge| &edge.target)
            .collect::<BTreeSet<_>>();
        row.collateral.dependency_closure = payloads
            .iter()
            .filter(|(key, _)| targets.contains(&Symbol::Relation((*key).clone())))
            .map(|(_, payload)| payload.key())
            .collect();
        negative.insert(row);
    }
    Ok(coverage::RelationCoverageProjection {
        activity: if joined.plan.active {
            crate::placement::RelationActivity::Active
        } else {
            crate::placement::RelationActivity::Vacuous
        },
        boundaries: joined.plan.boundaries.clone(),
        positive: joined
            .plan
            .positive
            .iter()
            .map(|row| lookup(&payload.positive, row))
            .collect::<Result<_, _>>()?,
        negative,
        carrier: joined
            .carrier
            .iter()
            .flatten()
            .map(|row| {
                Ok(coverage::CarrierAssignmentAlternative {
                    carriers: row
                        .carriers
                        .iter()
                        .map(|row| lookup(&payload.carriers, row))
                        .collect::<Result<_, _>>()?,
                    layout: row.layout.iter().map(|row| row.0.clone()).collect(),
                })
            })
            .collect::<Result<_, CompileError>>()?,
        projections: joined
            .projections
            .iter()
            .map(|boundary| Ok((*boundary, lookup(&payload.projections, boundary)?)))
            .collect::<Result<_, CompileError>>()?,
        external_evidence: joined
            .plan
            .evidence
            .iter()
            .map(|row| lookup(&payload.evidence, row))
            .collect::<Result<_, _>>()?,
    })
}
fn positives(
    plan: &RelationCasePlan,
    coverage: &coverage::RelationCoveragePlan,
    projections: &BTreeMap<coverage::CoverageBoundary, coverage::SemanticProjectionRequirement>,
    carriers: &BTreeMap<Placed, PlacedCarrier>,
) -> BTreeMap<Requirement, PositiveCoverageRequirement> {
    let mut positive = coverage
        .positive
        .iter()
        .map(|row| (super::coverage::requirement(&row.id), row.clone()))
        .collect::<BTreeMap<_, _>>();
    let key = super::placement::key(&coverage.key());
    for (boundary, projection) in projections {
        positive.insert(
            Requirement {
                key: key.clone(),
                boundary: *boundary,
                purpose: Purpose::AcceptedProjection,
                inactive: false,
            },
            PositiveCoverageRequirement {
                id: CoverageRequirementId {
                    relation: plan.relation.clone(),
                    case: plan.case.clone(),
                    boundary: *boundary,
                    purpose: coverage::CoveragePurpose::AcceptedProjection,
                },
                role: coverage::boundary_role(coverage, *boundary),
                representation: None,
                operands: projection.operands.clone(),
            },
        );
    }
    for (placed, value) in carriers {
        let boundary = coverage::CoverageBoundary::RuntimeCarrier;
        positive.insert(
            Requirement {
                key: key.clone(),
                boundary,
                purpose: Purpose::CarrierExecution(placed.clone()),
                inactive: false,
            },
            PositiveCoverageRequirement {
                id: CoverageRequirementId {
                    relation: plan.relation.clone(),
                    case: plan.case.clone(),
                    boundary,
                    purpose: coverage::CoveragePurpose::CarrierExecution(value.clone()),
                },
                role: coverage::EvidenceRole::TargetExecution,
                representation: None,
                operands: Vec::new(),
            },
        );
    }
    positive
}
fn restore_graph(
    payloads: &BTreeMap<Key, &Payload>,
    factor: &crate::engine_operation::Operation,
) -> Result<CoverageGraphProjection, CompileError> {
    let ids = symbols(payloads, &factor.layout);
    let mut nodes = factor
        .coverage
        .nodes
        .iter()
        .map(|node| {
            Ok(CoverageNode {
                id: lookup(&ids, &node.id)?,
                activity: node.active.map(|active| {
                    if active {
                        crate::placement::RelationActivity::Active
                    } else {
                        crate::placement::RelationActivity::Vacuous
                    }
                }),
            })
        })
        .collect::<Result<Vec<_>, CompileError>>()?;
    nodes.sort();
    let mut edges = factor
        .coverage
        .edges
        .iter()
        .map(|edge| {
            Ok(CoverageDependency {
                source: lookup(&ids, &edge.source)?,
                target: lookup(&ids, &edge.target)?,
                edge: edge.edge,
            })
        })
        .collect::<Result<Vec<_>, CompileError>>()?;
    edges.sort();
    Ok(CoverageGraphProjection { nodes, edges })
}
fn restore_bundle(
    payload: &Payload,
    bundle: &crate::engine_case_facts::Bundle,
    coverage: &coverage::RelationCoverageProjection,
) -> Result<RelationCaseRequirements, CompileError> {
    Ok(RelationCaseRequirements {
        key: payload.key(),
        activity: coverage.activity,
        activation: payload.plan.activation,
        boundaries: bundle.facts.boundaries.clone(),
        active_sources: bundle
            .facts
            .sources
            .iter()
            .map(|row| lookup(&payload.sources, row))
            .collect::<Result<_, _>>()?,
        compiler_requirements: bundle
            .facts
            .compiler
            .iter()
            .map(|row| lookup(&payload.compiler, row))
            .collect::<Result<_, _>>()?,
        structural_requirements: bundle
            .facts
            .structural
            .iter()
            .map(|row| lookup(&payload.structural, row))
            .collect::<Result<_, _>>()?,
        carrier_assignments: coverage.carrier.clone(),
        layout_requirements: bundle
            .facts
            .layout
            .iter()
            .map(|row| row.0.clone())
            .collect(),
        external_evidence: bundle
            .facts
            .evidence
            .iter()
            .map(|row| lookup(&payload.evidence, row))
            .collect::<Result<_, _>>()?,
        coverage: coverage.clone(),
    })
}
