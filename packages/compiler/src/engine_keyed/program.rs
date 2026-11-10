//! Keyed preparation, shared orchestration and owner-only publication.
use super::{KeyedIdentities, KeyedOwner};
use crate::{
    engine_program as core,
    engine_records::RelationOwner,
    input::BoundCompilerInput,
    keyed_plan::{
        EvidenceObligation, EvidencePremises, KeyedPlanError, KeyedPlanProjection,
        PlacementSearchLimits,
    },
};
use std::collections::BTreeMap;

#[cfg(test)]
mod negative_tests;
mod refusal;
#[cfg(test)]
mod refusal_tests;
mod restore;
mod stage_refusal;
#[cfg(test)]
mod tests;

/// Local to one derivation. Independent validation never receives this value.
pub(super) struct Prepared<'a> {
    input: &'a BoundCompilerInput<KeyedOwner>,
    pub(super) records: core::Records,
    pub(super) identities: KeyedIdentities,
    relations: BTreeMap<
        crate::engine_records::RelationKey,
        realization::relation::KeyedRelationDeclaration,
    >,
    operations:
        BTreeMap<crate::engine_records::OperationKey, architecture::semantic::SemanticOperationId>,
    evidence: BTreeMap<crate::engine_source::Evidence, EvidenceObligation>,
}

impl<'a> Prepared<'a> {
    pub(super) fn new(input: &'a BoundCompilerInput<KeyedOwner>) -> Result<Self, KeyedPlanError> {
        let span = tracing::debug_span!(target: "compiler::engine_keyed", "prepare", scope_count = crate::instrumentation::count(input.owner().scope().len()));
        let _entered = span.enter();
        let owner = input.owner();
        let shells = owner
            .shells()
            .map_err(|error| refusal::lowering(owner, error))?;
        let relations = owner
            .source()
            .project()
            .into_iter()
            .map(|row| (super::relation_key(row.id), row))
            .collect::<BTreeMap<_, _>>();
        let sources = super::lower_sources(owner);
        let identities = KeyedIdentities::from_source(owner, &sources);
        let evidence = sources
            .tables
            .iter()
            .flat_map(|table| &table.rows)
            .filter_map(|row| {
                if let crate::engine_source::Payload::Evidence(value) = &row.operand.payload {
                    Some((value.clone(), &row.operand.relation))
                } else {
                    None
                }
            })
            .map(|(key, relation)| {
                let row = relations.get(relation).ok_or_else(missing)?;
                Ok((
                    key.clone(),
                    EvidenceObligation {
                        relation: row.id,
                        requirement: row.relation,
                        premises: premises(&key)?,
                    },
                ))
            })
            .collect::<Result<_, KeyedPlanError>>()?;
        let proofs = super::proof_records(sources.clone());
        let requirements = crate::engine_requirement::proof_tables(&proofs).map_err(|error| {
            refusal::lowering(owner, super::KeyedLoweringError::ProofBundle(error))
        })?;
        let cases = super::case_records(owner, &sources)
            .map_err(|error| refusal::lowering(owner, error))?;
        let records = core::Records {
            source: owner.source_binding(),
            relations: crate::engine_records::RelationRecords {
                scope: shells.scope.clone(),
                nodes: relations
                    .keys()
                    .map(|id| crate::engine_records::RelationRecord {
                        id: id.clone(),
                        proof_owners: Vec::new(),
                    })
                    .collect(),
                edges: Vec::new(),
            },
            predicates: shells.predicates.clone(),
            lifecycle: shells.lifecycle(),
            constructibility: shells.construction(),
            sources,
            proofs,
            requirements,
            cases,
        };
        Ok(Self {
            input,
            records,
            identities,
            relations,
            evidence,
            operations: owner
                .scope()
                .iter()
                .map(|op| (super::operation_key(*op), *op))
                .collect(),
        })
    }

    /// Analyze this preparation only; stage results are rebuilt from its own
    /// records here, including on the production-route corruption tests.
    pub(super) fn analyze(
        &self,
        limits: PlacementSearchLimits,
    ) -> Result<core::Program, KeyedPlanError> {
        let relations = crate::engine_records::analyze_relations(&self.records.relations)
            .map_err(|e| self.refusal(core::Error::Relations(e)))?;
        let construction = crate::engine_constructibility::analyze_constructibility(
            &self.records.constructibility,
        )
        .map_err(|e| self.refusal(core::Error::Constructibility(Box::new(e))))?;
        crate::analyzed::analyze_bound_prepared(
            self.input,
            limits,
            (
                self.records.clone(),
                core::PreparedStages {
                    relations,
                    construction,
                },
            ),
        )
        .map_err(|e| self.refusal(e))
    }
}

fn missing() -> KeyedPlanError {
    KeyedPlanError::Compiler(Box::new(crate::CompileError::MissingOwnerIdentity))
}

fn premises(value: &crate::engine_source::Evidence) -> Result<EvidencePremises, KeyedPlanError> {
    use crate::engine_source::{Evidence as E, Recurrence};
    use realization::relation::RecordRecurrence as R;
    Ok(match value {
        E::HandleClassification {
            side,
            minimum,
            maximum,
            ..
        } => EvidencePremises::HandleClassification {
            side: decode_side(*side)?,
            minimum: *minimum,
            maximum: *maximum,
        },
        E::SourceClassification { .. } => EvidencePremises::SourceClassification,
        E::MembershipReading {
            recurrence,
            inclusion,
            retrieval,
            finality_for_final_reading,
            ..
        } => EvidencePremises::MembershipReading {
            recurrence: match recurrence {
                Recurrence::Singleton => R::SourceSingleton,
                Recurrence::ConsumedUnion => R::ConsumedUnion,
            },
            inclusion: *inclusion,
            retrieval: *retrieval,
            finality_for_final_reading: *finality_for_final_reading,
        },
        E::Operator { .. } | E::ConfidentialValue { .. } | E::SubstrateValue { .. } => {
            return Err(fact_refusal(crate::keyed_plan::Defect::ForeignEvidence));
        }
    })
}

pub fn derive(
    input: &BoundCompilerInput<KeyedOwner>,
    limits: PlacementSearchLimits,
) -> Result<KeyedPlanProjection, KeyedPlanError> {
    let prepared = Prepared::new(input)?;
    let program = prepared.analyze(limits)?;
    prepared.publish(&program)
}

const fn fact_refusal(defect: crate::keyed_plan::Defect) -> KeyedPlanError {
    KeyedPlanError::Refused {
        stage: crate::keyed_plan::Stage::Lowering,
        defect,
        relations: Vec::new(),
        operations: Vec::new(),
        unrestored_key_count: 0,
    }
}
fn decode_side(code: u16) -> Result<realization::TransactionSide, KeyedPlanError> {
    use realization::TransactionSide;
    [TransactionSide::Input, TransactionSide::Output]
        .into_iter()
        .find(|side| crate::engine_identity::side_code(*side) == code)
        .ok_or_else(|| fact_refusal(crate::keyed_plan::Defect::UnsupportedRuntimeScope))
}
