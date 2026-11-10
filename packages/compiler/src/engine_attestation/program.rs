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

// Publication still takes expression nodes/edges and the legacy lifecycle
// analysis from this pass: the predicate records do not encode expression
// payloads, and closure validators require the owner-typed lifecycle graph.
// Requirement/case/proof maps are identity-restoration registries; relation
// projections, selected plans, operation factors and search reports come from
// the common orchestration. Its lifecycle/source/construction outputs are now
// joined to proof inputs before search, not independently trusted copies.
// The owner-typed relation analysis remains for legacy validators; its published
// relation projection is reconstructed from the common output below.
struct Prepared {
    foundation: crate::foundation::CompilerAnalysisFoundation,
    lifecycle: crate::lifecycle::CompilerLifecycleAnalysis,
    proofs: super::proof::Lowered,
    requirements: super::requirement::Prepared,
    cases: super::cases::Registered,
}

/// Production preparation exposed only to cross-owner corruption controls.
#[cfg(test)]
pub fn test_preparation(
    input: &BoundCompilerInput,
) -> Result<(core::Records, core::PreparedStages), CompileError> {
    let (_, records, stages) = Prepared::new(input)?;
    Ok((records, stages))
}
impl Prepared {
    fn new(
        input: &BoundCompilerInput,
    ) -> Result<(Self, core::Records, core::PreparedStages), CompileError> {
        use crate::engine_records::{LoweredRelations, RelationOwner};
        let lowered = input.owner().lower_relations()?;
        let graph = crate::engine_records::analyze_relations(lowered.records())
            .map_err(|error| lowered.refusal(error))?;
        let relations = lowered.compatibility_view(Ok(graph.clone()))?;
        crate::sponsor_region::validate_decidable_sponsor_regions(
            &relations,
            input.scope().operations(),
        )?;
        let mut expressions = crate::expression::build_expression_analysis(input)?;
        let predicates = super::foundation::lower(&relations, &expressions);
        predicates.validate()?;
        crate::fold::fold_expressions(&mut expressions)?;
        let foundation = crate::foundation::CompilerAnalysisFoundation {
            relations,
            expressions,
        };
        let construction = super::constructibility::lower(input.owner())?;
        Self::from_stages(
            input,
            foundation,
            predicates.records,
            &lowered,
            graph,
            construction,
        )
    }

    fn from_stages(
        input: &BoundCompilerInput,
        foundation: crate::foundation::CompilerAnalysisFoundation,
        predicates: crate::engine_foundation::Records,
        lowered: &super::AttestationRelations,
        graph: crate::engine_records::RelationGraph,
        construction: super::constructibility::Lowered,
    ) -> Result<(Self, core::Records, core::PreparedStages), CompileError> {
        use crate::engine_records::{LoweredRelations, RelationOwner};
        // Proof prerequisites retain the base order: construction before
        // lifecycle. Reuse the same analyses for proof and case lowering.
        let construction_result =
            crate::engine_constructibility::analyze_constructibility(&construction.records)
                .map_err(|error| construction.refusal(error))?;
        let legacy_construction =
            construction.compatibility_view(Ok(construction_result.clone()))?;
        let lifecycle = lowered.lifecycle.analyze()?;
        let proofs = super::proof::lower_prepared(
            input,
            &foundation.relations,
            &legacy_construction,
            &lifecycle,
        )?;
        let mut requirements =
            super::requirement::prepare_from_analyses(&foundation.relations, &legacy_construction)?;
        let mut cases = super::cases::lower_prepared(input, &foundation.relations, &lifecycle)?;
        let records = core::Records {
            source: input.owner().source_binding(),
            relations: lowered.records().clone(),
            predicates,
            sources: input.owner().lower_sources()?,
            constructibility: construction.records,
            lifecycle: lowered.lifecycle().clone(),
            proofs: proofs.records.clone(),
            requirements: std::mem::take(&mut requirements.tables),
            cases: std::mem::take(&mut cases.tables),
        };
        Ok((
            Self {
                foundation,
                lifecycle,
                proofs,
                requirements,
                cases,
            },
            records,
            core::PreparedStages {
                relations: graph,
                construction: construction_result,
            },
        ))
    }
    fn refusal(&self, error: core::Error) -> CompileError {
        match error {
            #[cfg(test)]
            core::Error::Lowering(error) => match *error {
                crate::engine_records::LoweringError::Attestation(error) => error,
                other @ crate::engine_records::LoweringError::Keyed(_) => {
                    self.record_refusal(core::Error::Lowering(Box::new(other)))
                }
            },
            core::Error::Requirements(crate::engine_requirement::SelectionError::OwnerRefusal(
                error,
            ))
            | core::Error::Cases(crate::engine_cases::Error::OwnerRefusal(error)) => *error,
            core::Error::Proof(error) => self.proofs.refusal(error),
            core::Error::Operations(error) => {
                super::operation::refusal(&self.cases.payloads, &self.cases.layouts, *error)
            }
            defect => self.record_refusal(defect),
        }
    }

    fn record_refusal(&self, error: core::Error) -> CompileError {
        let (stage, keys) = refusal_subjects(error);
        let identities = self
            .foundation
            .relations
            .project()
            .nodes
            .into_iter()
            .map(|node| (super::relation_key(&node.source.id), node.source.id))
            .collect::<BTreeMap<_, _>>();
        let keys = keys.into_iter().collect::<std::collections::BTreeSet<_>>();
        let relations = keys
            .iter()
            .filter_map(|key| identities.get(key).cloned())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let unknown_relation_count = keys
            .iter()
            .filter(|key| !identities.contains_key(*key))
            .count();
        tracing::debug!(target: "compiler::engine_attestation", ?stage,
            relation_count = crate::instrumentation::count(relations.len()),
            unknown_relation_count = crate::instrumentation::count(unknown_relation_count),
            outcome = "refused", "owner record refusal restored");
        CompileError::InvalidOwnerProgramRecords {
            defect: Box::new(crate::OwnerProgramDefect {
                stage,
                relations,
                unknown_relation_count,
            }),
        }
    }
}

// This adapter erases private record vocabulary only after restoring its owner
// identities. Non-relation failures carry their stage and an empty relation set.
fn refusal_subjects(
    error: core::Error,
) -> (
    crate::OwnerProgramStage,
    Vec<crate::engine_records::RelationKey>,
) {
    use crate::OwnerProgramStage as S;
    use crate::engine_requirement::{Error as A, SelectionError as R};
    match error {
        core::Error::Relations(error) => (S::Relations, relation_subjects(error)),
        core::Error::Predicates(error) => {
            use crate::engine_foundation::Error as E;
            let (E::UnknownExpression { relation, .. }
            | E::NonBoolean { relation, .. }
            | E::OutsideScope { relation, .. }) = error;
            (S::Predicates, vec![relation])
        }
        core::Error::Sources(error) => {
            use crate::engine_source::SourceError as E;
            let keys = match *error {
                E::DuplicateOperand(key)
                | E::MissingOperand(key)
                | E::ForeignOperand(key)
                | E::DuplicateRow(key) => vec![key.relation],
                E::UndecidableRegion(_) => Vec::new(),
            };
            (S::Sources, keys)
        }
        core::Error::Constructibility(_) => (S::Constructibility, Vec::new()),
        core::Error::Lifecycle(_) => (S::Lifecycle, Vec::new()),
        core::Error::Requirements(
            R::Missing(key) | R::Unexpected(key) | R::Unapproved(key) | R::Duplicate(key),
        ) => (S::Requirements, vec![key]),
        core::Error::Aggregate(error) => {
            let (A::Capability { relation, .. }
            | A::Source { relation, .. }
            | A::Evidence { relation, .. }) = *error;
            (S::Aggregate, relation.into_iter().collect())
        }
        core::Error::Cases(error) => {
            use crate::engine_cases::Error as E;
            let keys = match error {
                E::CaseCensusMismatch {
                    missing,
                    unexpected,
                } => missing
                    .into_iter()
                    .chain(unexpected)
                    .map(|key| key.relation)
                    .collect(),
                E::DuplicateCase(key) | E::CaseFactsMismatch(key) => vec![key.relation],
                E::MissingRepresentationChoice(_) | E::UnsupportedRepresentation { .. } => {
                    Vec::new()
                }
                // Normally restored by Prepared::refusal. Retain a typed
                // stage diagnostic even if this private adapter is called directly.
                E::OwnerRefusal(_) => Vec::new(),
            };
            (S::Cases, keys)
        }
        core::Error::StageCensus {
            stage,
            missing,
            unexpected,
        } => {
            let stage = match stage {
                core::Stage::Source => S::SourceCensus,
                core::Stage::Proof => S::ProofCensus,
                core::Stage::Requirement => S::RequirementCensus,
                core::Stage::Case => S::CaseCensus,
            };
            (stage, missing.into_iter().chain(unexpected).collect())
        }
        #[cfg(test)]
        core::Error::SourceMismatch => (S::Program, Vec::new()),
        #[cfg(test)]
        core::Error::ProgramMismatch => (S::Program, Vec::new()),
        core::Error::ProofSourceMismatch(key) => (S::ProofSource, vec![key.relation]),
        core::Error::StageJoin(error) => {
            use core::StageJoinError as J;
            match error {
                J::RepresentationChoices => (S::ProofRepresentationJoin, Vec::new()),
                J::LifecycleRequirements => (S::ProofLifecycleJoin, Vec::new()),
                J::Constructibility(key) => (S::ProofConstructibilityJoin, vec![key.relation]),
                J::SourceRows(key) => (S::ProofSourceRowsJoin, vec![key.relation]),
                J::DuplicateSourceTable(key) => (S::DuplicateSourceTableJoin, vec![key.relation]),
                J::UnnamedSourceTable(key) => (S::UnnamedSourceTableJoin, vec![key.relation]),
            }
        }
        #[cfg(test)]
        core::Error::Lowering(_) => (S::Lowering, Vec::new()),
        core::Error::Requirements(R::OwnerRefusal(_)) => (S::Requirements, Vec::new()),
        core::Error::Proof(_) => (S::ProofSource, Vec::new()),
        core::Error::Operations(_) => (S::Program, Vec::new()),
    }
}

fn relation_subjects(
    error: crate::engine_records::RelationRecordError,
) -> Vec<crate::engine_records::RelationKey> {
    use crate::engine_records::RelationRecordError as E;
    match error {
        E::ForeignProofOwner {
            relation,
            alternative_relation,
        } => vec![relation, alternative_relation],
        E::DuplicateRelation { relation } | E::UnknownEndpoint { relation } => vec![relation],
        E::EscapesScope {
            prerequisite,
            dependent,
        }
        | E::DuplicateDependency {
            prerequisite,
            dependent,
            ..
        } => vec![prerequisite, dependent],
        E::Cycle { components } => components
            .into_iter()
            .flat_map(|component| component.members)
            .collect(),
        E::CensusMismatch {
            missing,
            unexpected,
        } => missing.into_iter().chain(unexpected).collect(),
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
    let (prepared, records, stages) = Prepared::new(input)?;
    let program = crate::analyzed::analyze_bound_prepared(input, limits, (records, stages))
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
            let operation = super::operation::restore(
                &prepared.cases.payloads,
                &prepared.cases.layouts,
                candidate,
                factor,
            )?;
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
    fn moved_case_tables_preserve_typed_layout_refusals() {
        let input = crate::tests::bound_input(&[architecture::OperationId::CompactAsh]);
        let (prepared, _, _) = Prepared::new(&input).expect("prepared control");
        let layout = prepared
            .cases
            .layouts
            .keys()
            .next()
            .expect("real layout key")
            .clone();
        let graph_error = crate::engine_coverage_graph::Error::UnknownSymbol(Box::new(
            crate::engine_coverage_graph::Key::Layout(layout.clone()),
        ));
        // The previous adapter rebuilt this set from the drained tables. Pin
        // its missing-registry refusal as well as the typed boundary restored below.
        let old_layout = BTreeMap::new();
        assert_eq!(
            super::super::operation::refusal(
                &prepared.cases.payloads,
                &old_layout,
                crate::engine_operation::Error::Graph(graph_error.clone()),
            ),
            CompileError::MissingOwnerIdentity
        );
        assert_eq!(
            prepared.refusal(core::Error::Operations(Box::new(
                crate::engine_operation::Error::Graph(graph_error),
            ))),
            CompileError::UnknownCoverageSymbol {
                symbol: crate::coverage_graph::CoverageNodeId::Layout(
                    prepared
                        .cases
                        .layouts
                        .get(&layout)
                        .expect("registered layout")
                        .clone()
                ),
            }
        );
    }

    #[test]
    fn record_diagnostics_have_typed_fallbacks_for_owner_refusals() {
        for (error, stage) in [
            (
                core::Error::Cases(crate::engine_cases::Error::OwnerRefusal(Box::new(
                    CompileError::InvalidRealization,
                ))),
                crate::OwnerProgramStage::Cases,
            ),
            (
                core::Error::Requirements(crate::engine_requirement::SelectionError::OwnerRefusal(
                    Box::new(CompileError::InvalidRealization),
                )),
                crate::OwnerProgramStage::Requirements,
            ),
            (
                core::Error::Proof(crate::engine_proof::Error::NoFeasiblePlan(Vec::new())),
                crate::OwnerProgramStage::ProofSource,
            ),
        ] {
            assert_eq!(refusal_subjects(error), (stage, Vec::new()));
        }
    }

    #[test]
    fn construction_refusal_precedes_lifecycle_when_both_inputs_are_defective() {
        use crate::engine_records::{LoweredRelations, RelationOwner};
        let input = crate::tests::bound_input(&[architecture::OperationId::CompactAsh]);
        for (bad_construction, bad_lifecycle) in
            [(false, false), (false, true), (true, false), (true, true)]
        {
            let foundation = crate::foundation::analyze_foundation(&input).expect("foundation");
            let predicates =
                super::super::foundation::lower(&foundation.relations, &foundation.expressions)
                    .records;
            let mut lowered = input.owner().lower_relations().expect("relation records");
            let graph = crate::engine_records::analyze_relations(lowered.records()).expect("graph");
            let mut construction =
                super::super::constructibility::lower(input.owner()).expect("construction records");
            if bad_construction {
                let node = construction
                    .records
                    .nodes
                    .iter()
                    .find(|node| construction.records.scope.contains(&node.id.operation))
                    .expect("scoped node")
                    .clone();
                construction.records.nodes.push(node);
            }
            if bad_lifecycle {
                lowered.lifecycle.records.choices.clear();
            }
            let result = Prepared::from_stages(
                &input,
                foundation,
                predicates,
                &lowered,
                graph,
                construction,
            );
            match (bad_construction, bad_lifecycle) {
                (false, false) => assert!(result.is_ok()),
                (false, true) => assert!(matches!(
                    result.err(),
                    Some(CompileError::MissingRepresentationChoice { .. })
                )),
                (true, _) => assert!(matches!(
                    result.err(),
                    Some(CompileError::DuplicateConstructibilityNode { .. })
                )),
            }
        }
    }

    #[test]
    fn record_refusal_restores_public_subjects_and_counts_unknown_keys() {
        let input = crate::tests::bound_input(&[architecture::OperationId::CompactAsh]);
        let (prepared, _, _) = Prepared::new(&input).expect("accepted owner");
        let relation = prepared.foundation.relations.project().nodes[0]
            .source
            .id
            .clone();
        let key = super::super::relation_key(&relation);
        let mut foreign = key.clone();
        foreign.operation.namespace = u16::MAX;
        let refusal = prepared.refusal(core::Error::StageCensus {
            stage: core::Stage::Proof,
            missing: vec![key],
            unexpected: vec![foreign],
        });
        assert_eq!(
            refusal,
            CompileError::InvalidOwnerProgramRecords {
                defect: Box::new(crate::OwnerProgramDefect {
                    stage: crate::OwnerProgramStage::ProofCensus,
                    relations: vec![relation],
                    unknown_relation_count: 1,
                }),
            }
        );
    }

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
        let (prepared, _, _) = Prepared::new(&input).expect("same owner");
        assert_eq!(prepared.refusal(refusal), CompileError::SponsorValueRead);
    }
}
