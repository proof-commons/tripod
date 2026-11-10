//! Fallible restoration through the statically bound keyed owner's registries.
use super::{Prepared, missing};
use crate::{
    engine_coverage as coverage, engine_coverage_graph as graph, engine_operation,
    engine_placement as placement, engine_program,
    keyed_plan::{
        CoverageDependency, CoverageNode, CoveragePurpose, CoverageRequirement, CoverageSymbol,
        EvidenceDefect, EvidenceObligation, KeyedPlanError, KeyedPlanProjection, OperationPlan,
        ProofPlan, RelationPlan, RuntimeCarrier, StageCensus,
    },
};
use realization::relation::KeyedRelationId;
use std::collections::BTreeSet;

impl Prepared<'_> {
    pub(super) fn relation(
        &self,
        key: &crate::engine_records::RelationKey,
    ) -> Result<KeyedRelationId, KeyedPlanError> {
        self.relations
            .get(key)
            .map(|row| row.id)
            .ok_or_else(missing)
    }
    fn case(&self, key: &placement::Key) -> Result<KeyedRelationId, KeyedPlanError> {
        let relation = self.relation(&key.relation)?;
        let operation = self
            .identities
            .case(&key.case)
            .map_err(|e| KeyedPlanError::Compiler(Box::new(e)))?;
        if relation.operation != operation {
            return Err(missing());
        }
        Ok(relation)
    }
    fn evidence(
        &self,
        key: &crate::engine_source::Evidence,
    ) -> Result<EvidenceObligation, KeyedPlanError> {
        self.evidence.get(key).cloned().ok_or_else(missing)
    }
    fn carrier(&self, value: &placement::Placed) -> Result<RuntimeCarrier, KeyedPlanError> {
        let (operation, object) = self
            .identities
            .carrier(&value.carrier)
            .map_err(|e| KeyedPlanError::Compiler(Box::new(e)))?;
        Ok(RuntimeCarrier {
            operation,
            object,
            quantification: value.quantification,
        })
    }
    fn requirement(
        &self,
        value: &coverage::Requirement,
    ) -> Result<CoverageRequirement, KeyedPlanError> {
        use super::super::{EvidenceDefect as D, KeyedPurpose as P};
        let purpose = match &value.purpose {
            coverage::Purpose::AcceptedProjection => CoveragePurpose::AcceptedProjection,
            coverage::Purpose::CarrierExecution(value) => {
                CoveragePurpose::CarrierExecution(self.carrier(value)?)
            }
            coverage::Purpose::Owner(key) => match self
                .identities
                .purpose(key)
                .map_err(|e| KeyedPlanError::Compiler(Box::new(e)))?
            {
                P::ClauseAccepted => CoveragePurpose::ClauseAccepted,
                P::ClauseRejected => CoveragePurpose::ClauseRejected,
                P::Verified(value) => CoveragePurpose::Verified(self.evidence(&value)?),
                P::Rejected { evidence, defect } => CoveragePurpose::Rejected {
                    evidence: self.evidence(&evidence)?,
                    defect: match defect {
                        D::Missing => EvidenceDefect::Missing,
                        D::Failed => EvidenceDefect::Failed,
                        D::IdentityMismatch => EvidenceDefect::IdentityMismatch,
                    },
                },
            },
        };
        Ok(CoverageRequirement {
            relation: self.case(&value.key)?,
            boundary: value.boundary,
            purpose,
            inactive: value.inactive,
        })
    }
    pub(super) fn symbol(&self, key: &graph::Key) -> Result<CoverageSymbol, KeyedPlanError> {
        Ok(match key {
            graph::Key::Relation(key) => CoverageSymbol::Relation(self.case(key)?),
            graph::Key::Requirement(value) => {
                CoverageSymbol::Requirement(Box::new(self.requirement(value)?))
            }
            graph::Key::Evidence(value) => CoverageSymbol::Evidence(self.evidence(value)?),
            graph::Key::Carrier { key, placed } => CoverageSymbol::Carrier {
                relation: self.case(key)?,
                carrier: self.carrier(placed)?,
            },
            graph::Key::Layout(_) => return Err(missing()),
            #[cfg(test)]
            graph::Key::Owner(_) => return Err(missing()),
        })
    }
    fn relation_plans(
        &self,
        value: &engine_operation::Operation,
    ) -> Result<std::collections::BTreeMap<KeyedRelationId, RelationPlan>, KeyedPlanError> {
        let result = value
            .bundles
            .iter()
            .map(|(key, bundle)| {
                let relation = self.case(key)?;
                let plan = &bundle.coverage.plan;
                let alternatives = bundle
                    .carriers
                    .iter()
                    .map(|alternative| {
                        if !alternative.layout.is_empty() {
                            return Err(missing());
                        }
                        alternative
                            .carriers
                            .iter()
                            .map(|placed| self.carrier(placed))
                            .collect::<Result<BTreeSet<_>, _>>()
                    })
                    .collect::<Result<_, _>>()?;
                Ok((
                    relation,
                    RelationPlan {
                        declaration: self
                            .relations
                            .get(&key.relation)
                            .ok_or_else(missing)?
                            .relation,
                        active: plan.active,
                        boundaries: plan.boundaries.clone(),
                        positive: plan
                            .positive
                            .iter()
                            .map(|row| self.requirement(row))
                            .collect::<Result<_, _>>()?,
                        negative: plan
                            .negative
                            .iter()
                            .map(|row| self.requirement(row))
                            .collect::<Result<_, _>>()?,
                        evidence: plan
                            .evidence
                            .iter()
                            .map(|row| self.evidence(row))
                            .collect::<Result<_, _>>()?,
                        alternatives,
                        projections: bundle.coverage.projections.clone(),
                    },
                ))
            })
            .collect::<Result<_, KeyedPlanError>>()?;
        Ok(result)
    }
    fn operation(
        &self,
        value: &engine_operation::Operation,
    ) -> Result<OperationPlan, KeyedPlanError> {
        if !value.layout.is_empty() {
            return Err(missing());
        }
        let relations = self.relation_plans(value)?;
        let placements = value
            .placements
            .iter()
            .map(|candidate| {
                if !candidate.layout.is_empty() {
                    return Err(missing());
                }
                candidate
                    .assignments
                    .iter()
                    .map(|row| {
                        Ok((
                            self.case(&row.key)?,
                            row.carriers
                                .iter()
                                .map(|value| self.carrier(value))
                                .collect::<Result<_, _>>()?,
                        ))
                    })
                    .collect::<Result<_, _>>()
            })
            .collect::<Result<_, _>>()?;
        let coverage_nodes = value
            .coverage
            .nodes
            .iter()
            .map(|node| {
                Ok(CoverageNode {
                    id: self.symbol(&node.id)?,
                    operation: *self.operations.get(&node.operation).ok_or_else(missing)?,
                    class: node.class,
                    relation: node
                        .relation_case
                        .as_ref()
                        .map(|key| self.case(key))
                        .transpose()?,
                    active: node.active,
                })
            })
            .collect::<Result<_, KeyedPlanError>>()?;
        let coverage_edges = value
            .coverage
            .edges
            .iter()
            .map(|edge| {
                Ok(CoverageDependency {
                    source: self.symbol(&edge.source)?,
                    target: self.symbol(&edge.target)?,
                    edge: edge.edge,
                })
            })
            .collect::<Result<_, KeyedPlanError>>()?;
        Ok(OperationPlan {
            relations,
            placements,
            coverage_nodes,
            coverage_edges,
            search: value.search,
        })
    }
    pub(super) fn publish(
        &self,
        value: &engine_program::Program,
    ) -> Result<KeyedPlanProjection, KeyedPlanError> {
        let span = tracing::debug_span!(target: "compiler::engine_keyed", "publish", plan_count = crate::instrumentation::count(value.plans.len()));
        let _entered = span.enter();
        if value.source != self.records.source {
            return Err(KeyedPlanError::PublicationMismatch);
        }
        let plans = value
            .plans
            .iter()
            .map(|(candidate, plan)| {
                let selected_relations = candidate
                    .proofs
                    .keys()
                    .map(|key| self.relation(key))
                    .collect::<Result<_, _>>()?;
                let requirements = plan
                    .requirements
                    .iter()
                    .map(|(relation, bundle)| {
                        Ok((
                            self.relation(relation)?,
                            bundle
                                .evidence
                                .iter()
                                .map(|row| self.evidence(row))
                                .collect::<Result<_, _>>()?,
                        ))
                    })
                    .collect::<Result<_, KeyedPlanError>>()?;
                let operations = plan
                    .operations
                    .iter()
                    .map(|(operation, value)| {
                        Ok((
                            *self.operations.get(operation).ok_or_else(missing)?,
                            self.operation(value)?,
                        ))
                    })
                    .collect::<Result<_, KeyedPlanError>>()?;
                Ok(ProofPlan {
                    selected_relations,
                    requirements,
                    operations,
                })
            })
            .collect::<Result<Vec<_>, KeyedPlanError>>()?;
        let stages = StageCensus {
            relations: value.relations.nodes.len(),
            source_tables: value.sources.len(),
            construction_nodes: value.constructibility.nodes.len(),
            lifecycle_nodes: value.lifecycle.nodes.len(),
            proof_plans: plans.len(),
            placement_assignments: plans
                .iter()
                .flat_map(|plan| plan.operations.values())
                .flat_map(|operation| &operation.placements)
                .map(std::collections::BTreeMap::len)
                .sum(),
            coverage_nodes: plans
                .iter()
                .flat_map(|plan| plan.operations.values())
                .map(|operation| operation.coverage_nodes.len())
                .sum(),
        };
        Ok(KeyedPlanProjection {
            binding: self.input.owner().source().binding().clone(),
            scope: self.input.owner().scope().to_vec(),
            plans,
            evidence: value
                .evidence
                .iter()
                .map(|row| self.evidence(row))
                .collect::<Result<_, _>>()?,
            stages,
            proof_limits: value.proof_limits,
            placement_limits: value.placement_limits,
            proof_search: value.proof_search.clone(),
        })
    }
}
