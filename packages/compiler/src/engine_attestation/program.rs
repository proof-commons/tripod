//! Attestation publication from the common record-program orchestration.
use crate::{
    BoundCompilerInput, CompileError,
    analyzed::{
        AnalysisExecutionReport, AnalyzedProofPlan, AnalyzedSource, accumulate,
        architecture_scope_status, lifecycle_completeness, revalidate_input,
    },
    analyzed_validate::AnalyzedExpectations,
    engine_program as core,
    placement::PlacementSearchLimits,
};
use std::collections::BTreeMap;

struct Prepared {
    foundation: crate::foundation::CompilerAnalysisFoundation,
    lifecycle: crate::lifecycle::CompilerLifecycleAnalysis,
    proofs: super::proof::Lowered,
    requirements: super::requirement::Prepared,
    cases: super::cases::Registered,
}
impl Prepared {
    fn new(input: &BoundCompilerInput) -> Result<Self, CompileError> {
        let foundation = crate::foundation::analyze_foundation(input)?;
        let lifecycle = crate::lifecycle::build_lifecycle_analysis(input, &foundation.relations)?;
        Ok(Self {
            foundation,
            lifecycle,
            proofs: super::proof::lower(input)?,
            requirements: super::requirement::prepare(input)?,
            cases: super::cases::lower_registered(input)?,
        })
    }
    fn refusal(&self, error: core::Error) -> CompileError {
        match error {
            core::Error::Lowering(error)
            | core::Error::Requirements(crate::engine_requirement::SelectionError::OwnerRefusal(
                error,
            ))
            | core::Error::Cases(crate::engine_cases::Error::OwnerRefusal(error)) => *error,
            core::Error::Proof(error) => self.proofs.refusal(error),
            core::Error::Operations(error) => {
                let layout = self
                    .cases
                    .tables
                    .rows
                    .iter()
                    .filter_map(|row| row.facts.as_ref())
                    .flat_map(|facts| facts.layout.iter().cloned())
                    .collect();
                super::operation::refusal(&self.cases.payloads, &layout, *error)
            }
            defect => CompileError::InvalidOwnerProgramRecords {
                defect: Box::new(defect),
            },
        }
    }
}

/// Both production assembly and independent validation re-enter through the
/// retained bound owner. Publication performs finite identity restoration only.
pub fn derive(
    input: &BoundCompilerInput,
    limits: PlacementSearchLimits,
) -> Result<AnalyzedExpectations, CompileError> {
    let span =
        tracing::debug_span!(target: "compiler::engine_attestation", "publish_bound_program");
    let _entered = span.enter();
    revalidate_input(input)?;
    let prepared = Prepared::new(input)?;
    let program = crate::analyzed::analyze_bound_records(input, limits)
        .map_err(|error| prepared.refusal(error))?;
    let foundation = restore_foundation(&prepared.foundation.project(), &program.relations)?;
    let mut plans = BTreeMap::new();
    let mut obligations = BTreeMap::new();
    let mut placement_search = BTreeMap::new();
    for (candidate, plan) in &program.plans {
        let legacy = prepared.proofs.candidate(candidate.clone());
        let requirements = prepared
            .requirements
            .restore(&plan.requirements, candidate, &legacy)?;
        crate::requirement::validate_aggregate_closure(&legacy, &requirements)?;
        crate::requirement::validate_disclosure_closure(input, &legacy, &requirements)?;
        crate::requirement::validate_lifecycle_closure(
            &prepared.lifecycle,
            &legacy,
            &requirements,
        )?;
        let mut operations = BTreeMap::new();
        for factor in plan.operations.values() {
            let operation = super::operation::restore(&prepared.cases.payloads, candidate, factor)?;
            crate::analyzed_operation::validate_analyzed_operation(
                &prepared.foundation.relations,
                &legacy,
                &operation,
            )?;
            let entry = placement_search.entry(operation.operation).or_default();
            *entry = accumulate(*entry, factor.search);
            obligations.insert(
                (legacy.clone(), operation.operation),
                super::operation::obligations(&prepared.cases.payloads, candidate, factor)?,
            );
            operations.insert(operation.operation, operation);
        }
        crate::analyzed_operation::validate_operation_factorization(
            &prepared.foundation.relations,
            &legacy,
            &operations,
        )?;
        let analyzed = AnalyzedProofPlan {
            proof_plan: legacy.clone(),
            lifecycle: lifecycle_completeness(&legacy),
            relation_requirements: requirements,
            operations,
        };
        if plans.insert(legacy, analyzed).is_some() {
            return Err(CompileError::DuplicateAnalyzedProofPlan);
        }
    }
    Ok(AnalyzedExpectations {
        source: AnalyzedSource {
            architecture: input.architecture_binding().clone(),
            realization: input.realization().project(),
            compilation_scope: input.scope().clone(),
        },
        relation_census: foundation
            .relations
            .nodes
            .iter()
            .map(|node| node.source.id.clone())
            .collect(),
        foundation,
        relations: prepared.foundation.relations,
        lifecycle: prepared.lifecycle,
        architecture_scope: architecture_scope_status(input),
        scope: input.scope().operations().iter().copied().collect(),
        plans,
        obligations,
        execution_report: AnalysisExecutionReport {
            proof_search_limits: program.proof_limits,
            placement_search_limits: program.placement_limits,
            proof_search: program.proof_search,
            operation_placement_search: placement_search,
        },
    })
}

fn restore_foundation(
    source: &crate::foundation::CompilerAnalysisFoundationProjection,
    records: &crate::engine_records::RelationRecords,
) -> Result<crate::foundation::CompilerAnalysisFoundationProjection, CompileError> {
    use crate::relation::{AnalysisNodeId, CompilerRelationEdge};
    let nodes = source
        .relations
        .nodes
        .iter()
        .map(|node| (super::relation_key(&node.source.id), node))
        .collect::<BTreeMap<_, _>>();
    let edges = source
        .relations
        .edges
        .iter()
        .map(|edge| {
            let AnalysisNodeId::SourceRelation(source) = &edge.source;
            let AnalysisNodeId::SourceRelation(target) = &edge.target;
            let CompilerRelationEdge::SourceDependency(kind) = edge.edge;
            (
                crate::engine_records::DependencyRecord {
                    source: super::relation_key(source),
                    target: super::relation_key(target),
                    kind: super::dependency_kind(kind),
                },
                edge,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut projection = source.clone();
    projection.relations.nodes = records
        .nodes
        .iter()
        .map(|node| {
            nodes
                .get(&node.id)
                .map(|value| (*value).clone())
                .ok_or(CompileError::InvalidRealization)
        })
        .collect::<Result<_, _>>()?;
    projection
        .relations
        .nodes
        .sort_by(|left, right| left.id.cmp(&right.id));
    projection.relations.edges = records
        .edges
        .iter()
        .map(|edge| {
            edges
                .get(edge)
                .map(|value| (*value).clone())
                .ok_or(CompileError::InvalidRealization)
        })
        .collect::<Result<_, _>>()?;
    projection.relations.edges.sort();
    Ok(projection)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_owner_requirement_refusal_keeps_its_original_boundary() {
        let input = crate::tests::bound_input(&[architecture::OperationId::CompactAsh]);
        let limits = PlacementSearchLimits::new(
            std::num::NonZeroU64::new(10_000_000).expect("states"),
            std::num::NonZeroU64::new(1_000_000).expect("candidates"),
        );
        let mut records = core::lower(&input).expect("owner records");
        let accepted = core::analyze(&records, input.policy().proof_search_limits, limits)
            .expect("accepted control");
        let candidate = accepted.plans.keys().next().expect("plan");
        let table = records.requirements.first_mut().expect("requirement table");
        let choice = candidate.proofs.get(&table.relation).cloned();
        *table.choices.get_mut(&choice).expect("selected row") =
            Err(CompileError::SponsorValueRead);
        let refusal = core::analyze(&records, input.policy().proof_search_limits, limits)
            .expect_err("one retained owner refusal");
        let prepared = Prepared::new(&input).expect("same owner");
        assert_eq!(prepared.refusal(refusal), CompileError::SponsorValueRead);
    }
}
