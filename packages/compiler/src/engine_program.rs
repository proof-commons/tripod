//! Production whole-program record consumer and statically bound re-derivation.
use crate::{
    engine_cases, engine_constructibility, engine_foundation, engine_lifecycle, engine_operation,
    engine_proof,
    engine_records::{self, LoweredRelations, OperationKey, RelationKey, RelationOwner},
    engine_requirement, engine_source,
    input::{BoundCompilerInput, ProofSearchLimits},
    placement::PlacementSearchLimits,
};
use petgraph::visit::EdgeRef as _;
use std::collections::{BTreeMap, BTreeSet};

/// Opaque source identity. The engine compares the complete binding and never
/// branches on its variant to choose policy or an owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    Attestation(Box<crate::analyzed::AnalyzedSource>),
    Keyed {
        manifest: Box<architecture::keyed::KeyedManifest>,
        realization: realization::keyed::KeyedRealizationDraft,
        scope: Vec<architecture::semantic::SemanticOperationId>,
    },
}
pub struct Records {
    pub source: Source,
    pub relations: engine_records::RelationRecords,
    pub predicates: engine_foundation::Records,
    pub sources: engine_source::SourceRecords,
    pub constructibility: engine_constructibility::Records,
    pub lifecycle: engine_lifecycle::LifecycleRecords,
    pub proofs: engine_proof::Records,
    pub requirements: Vec<engine_requirement::Table>,
    pub cases: engine_cases::Tables,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    pub requirements: BTreeMap<RelationKey, engine_requirement::Bundle>,
    pub operations: BTreeMap<OperationKey, engine_operation::Operation>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub source: Source,
    pub relations: engine_records::RelationRecords,
    pub predicates: engine_foundation::Records,
    pub sources: Vec<Vec<engine_source::SourceRow>>,
    pub regions: Vec<engine_source::RegionRecord>,
    pub constructibility: engine_constructibility::Records,
    pub construction: Vec<engine_constructibility::OperationAnalysis>,
    pub lifecycle: engine_lifecycle::LifecycleProjection,
    pub plans: BTreeMap<engine_proof::Candidate, Plan>,
    pub evidence: BTreeSet<engine_source::Evidence>,
    pub proof_search: crate::proof::ProofSearchReport,
    pub proof_limits: ProofSearchLimits,
    pub placement_limits: PlacementSearchLimits,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Lowering(Box<crate::CompileError>),
    Relations(engine_records::RelationRecordError),
    Predicates(engine_foundation::Error),
    Sources(Box<engine_source::SourceError>),
    Constructibility(Box<engine_constructibility::RecordError>),
    Lifecycle(engine_lifecycle::LifecycleRecordError),
    Proof(engine_proof::Error),
    Requirements(engine_requirement::SelectionError),
    Aggregate(Box<engine_requirement::Error>),
    Cases(engine_cases::Error),
    Operations(Box<engine_operation::Error>),
    StageCensus {
        stage: Stage,
        missing: Vec<RelationKey>,
        unexpected: Vec<RelationKey>,
    },
    SourceMismatch,
    ProgramMismatch,
    ProofSourceMismatch(engine_proof::AlternativeKey),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Source,
    Proof,
    Requirement,
    Case,
}

#[allow(dead_code)]
pub fn lower<O: RelationOwner>(input: &BoundCompilerInput<O>) -> Result<Records, Error> {
    let span = tracing::debug_span!(target: "compiler::engine_program", "lower_bound_owner");
    let _entered = span.enter();
    let relation = input.owner().lower_relations();
    let refusal = |error| Error::Lowering(Box::new(error));
    Ok(Records {
        source: input.owner().source_binding(),
        relations: relation.records().clone(),
        lifecycle: relation.lifecycle().clone(),
        predicates: O::lower_predicates(input).map_err(refusal)?,
        sources: input.owner().lower_sources().map_err(refusal)?,
        constructibility: input.owner().lower_constructibility().map_err(refusal)?,
        proofs: O::lower_proofs(input).map_err(refusal)?,
        requirements: O::lower_requirements(input).map_err(refusal)?,
        cases: O::lower_cases(input).map_err(refusal)?,
    })
}

/// One orchestration for both record producers, with no owner parameter after
/// lowering. Search, factorization and all seven stages use concrete records.
#[allow(dead_code)]
pub fn analyze(
    records: &Records,
    proof_limits: ProofSearchLimits,
    placement_limits: PlacementSearchLimits,
) -> Result<Program, Error> {
    let span = tracing::debug_span!(target: "compiler::engine_program", "analyze", scope_count = crate::instrumentation::count(records.relations.scope.len()));
    let _entered = span.enter();
    let relation_graph =
        engine_records::analyze_relations(&records.relations).map_err(Error::Relations)?;
    engine_foundation::validate_predicates(&records.predicates).map_err(Error::Predicates)?;
    let relations = project_relations(&records.relations.scope, &relation_graph);
    stage_census(records, &relations)?;
    let sources = engine_source::analyze_sources(&records.sources)
        .map_err(|error| Error::Sources(Box::new(error)))?;
    proof_source_join(records)?;
    let construction = engine_constructibility::analyze_constructibility(&records.constructibility)
        .map_err(|error| Error::Constructibility(Box::new(error)))?;
    let lifecycle =
        engine_lifecycle::analyze_lifecycle(&records.lifecycle).map_err(Error::Lifecycle)?;
    let mut proof_search = crate::proof::ProofSearchReport::default();
    let candidates = engine_proof::enumerate(
        &records.proofs,
        &crate::capability::CapabilityView::Unconstrained,
        proof_limits,
        &mut proof_search,
    )
    .map_err(Error::Proof)?;
    let prerequisites = relations
        .edges
        .iter()
        .map(|edge| (edge.source.clone(), edge.target.clone()))
        .collect::<Vec<_>>();
    let mut plans = BTreeMap::new();
    let mut evidence = BTreeSet::new();
    for candidate in candidates {
        let requirements = engine_requirement::derive(&records.requirements, &candidate.proofs)
            .map_err(Error::Requirements)?;
        let aggregate = engine_requirement::Bundle {
            capabilities: candidate.capabilities.clone(),
            sources: candidate.sources.iter().cloned().collect(),
            evidence: candidate.evidence.clone(),
        };
        engine_requirement::validate_aggregate(&aggregate, &requirements)
            .map_err(|error| Error::Aggregate(Box::new(error)))?;
        let selected = engine_cases::select(&records.cases, &candidate).map_err(Error::Cases)?;
        let operations = engine_operation::analyze(&selected, &prerequisites, placement_limits)
            .map_err(|error| Error::Operations(Box::new(error)))?;
        evidence.extend(
            requirements
                .values()
                .flat_map(|row| row.evidence.iter().cloned()),
        );
        plans.insert(
            candidate,
            Plan {
                requirements,
                operations,
            },
        );
    }
    Ok(Program {
        source: records.source.clone(),
        relations,
        predicates: records.predicates.clone(),
        sources,
        regions: records.sources.regions.clone(),
        constructibility: records.constructibility.clone(),
        construction: construction.operations,
        lifecycle: lifecycle.project(),
        plans,
        evidence,
        proof_search,
        proof_limits,
        placement_limits,
    })
}

fn project_relations(
    scope: &[OperationKey],
    graph: &engine_records::RelationGraph,
) -> engine_records::RelationRecords {
    let mut nodes = graph.graph.node_weights().cloned().collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.id.cmp(&right.id));
    let mut edges = graph
        .graph
        .edge_references()
        .map(|edge| engine_records::DependencyRecord {
            source: graph.graph[edge.source()].id.clone(),
            target: graph.graph[edge.target()].id.clone(),
            kind: *edge.weight(),
        })
        .collect::<Vec<_>>();
    edges.sort();
    engine_records::RelationRecords {
        scope: scope.to_vec(),
        nodes,
        edges,
    }
}

fn stage_census(
    records: &Records,
    relations: &engine_records::RelationRecords,
) -> Result<(), Error> {
    let expected = relations
        .nodes
        .iter()
        .map(|node| node.id.clone())
        .collect::<BTreeSet<_>>();
    for (stage, offered, complete) in [
        (
            Stage::Source,
            records
                .sources
                .tables
                .iter()
                .map(|row| row.relation.clone())
                .collect::<BTreeSet<_>>(),
            false,
        ),
        (
            Stage::Proof,
            records.proofs.obligations.keys().cloned().collect(),
            true,
        ),
        (
            Stage::Requirement,
            records
                .requirements
                .iter()
                .map(|row| row.relation.clone())
                .collect(),
            true,
        ),
        (Stage::Case, records.cases.relations.clone(), true),
    ] {
        if !offered.is_subset(&expected) || (complete && offered != expected) {
            return Err(Error::StageCensus {
                stage,
                missing: if complete {
                    expected.difference(&offered).cloned().collect()
                } else {
                    Vec::new()
                },
                unexpected: offered.difference(&expected).cloned().collect(),
            });
        }
    }
    Ok(())
}

fn proof_source_join(records: &Records) -> Result<(), Error> {
    for (relation, obligation) in &records.proofs.obligations {
        let alternatives: &[engine_proof::Alternative] = match obligation {
            engine_proof::Obligation::Static => &[],
            engine_proof::Obligation::Evidence(alternative) => std::slice::from_ref(alternative),
            engine_proof::Obligation::Required(alternatives) => alternatives,
        };
        for alternative in alternatives {
            if &alternative.id.relation != relation
                || !records.sources.tables.iter().any(|table| {
                    table.relation == *relation
                        && table.proof_namespace == alternative.id.namespace
                        && table.proof_kind == alternative.id.kind
                })
            {
                return Err(Error::ProofSourceMismatch(alternative.id.clone()));
            }
        }
    }
    Ok(())
}

/// The type fixed at binding lowers again. No source variant is inspected to
/// recover an owner; the complete offered value is compared with that result.
#[allow(dead_code)]
pub fn validate<O: RelationOwner>(
    input: &BoundCompilerInput<O>,
    limits: PlacementSearchLimits,
    offered: &Program,
) -> Result<(), Error> {
    let span = tracing::debug_span!(target: "compiler::engine_program", "rederive_bound_owner");
    let _entered = span.enter();
    if input.owner().source_binding() != offered.source {
        return Err(Error::SourceMismatch);
    }
    let expected = crate::analyzed::analyze_bound_records(input, limits)?;
    if &expected == offered {
        Ok(())
    } else {
        Err(Error::ProgramMismatch)
    }
}
