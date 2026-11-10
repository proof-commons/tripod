//! Restore public owner identities at the keyed error boundary.
use super::{Prepared, missing};
use crate::{
    engine_program as core,
    keyed_plan::{Defect as D, KeyedPlanError as E, Stage as S},
};

pub(super) fn lowering(
    owner: &super::super::KeyedOwner,
    error: super::super::KeyedLoweringError,
) -> E {
    use super::super::KeyedLoweringError as L;
    let (defect, key, proof) = match error {
        L::UnsupportedShell {
            operation,
            field,
            count,
        } => {
            return E::UnsupportedShell {
                operation,
                field,
                count,
            };
        }
        L::MissingSourceTable(key) => (D::MissingSourceTable, key, None),
        L::RuntimeSourcePayload(key) => (D::RuntimeSourcePayload, key, None),
        L::ProofBundle(crate::engine_requirement::ProofBundleError::Constructibility(key)) => {
            (D::Constructibility, key.relation, Some(None))
        }
        L::ProofBundle(crate::engine_requirement::ProofBundleError::Source {
            alternative,
            reason,
        }) => (D::SourceRows, alternative.relation, Some(Some(reason))),
    };
    let Some(row) = owner
        .source()
        .project()
        .into_iter()
        .find(|row| super::super::relation_key(row.id) == key)
    else {
        return missing();
    };
    if let Some(reason) = proof {
        return E::ProofBundle {
            relation: row.id,
            reason,
        };
    }
    E::Refused {
        stage: S::Lowering,
        defect,
        relations: vec![row.id],
        operations: vec![row.id.operation],
        unrestored_key_count: 0,
    }
}
impl Prepared<'_> {
    pub(super) fn refused(
        &self,
        stage: S,
        defect: D,
        keys: Vec<crate::engine_records::RelationKey>,
    ) -> E {
        let mut unrestored_key_count = 0;
        let relations = keys
            .into_iter()
            .filter_map(|key| {
                self.relation(&key).map_or_else(
                    |_| {
                        unrestored_key_count += 1;
                        None
                    },
                    Some,
                )
            })
            .collect::<std::collections::BTreeSet<_>>();
        E::Refused {
            stage,
            defect,
            operations: relations
                .iter()
                .map(|r| r.operation)
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect(),
            relations: relations.into_iter().collect(),
            unrestored_key_count,
        }
    }
    pub(super) fn refusal(&self, error: core::Error) -> E {
        use crate::engine_proof::Error as P;
        #[cfg(test)]
        let empty = |stage| E::Refused {
            stage,
            defect: D::InvalidRecords,
            relations: Vec::new(),
            operations: Vec::new(),
            unrestored_key_count: 0,
        };
        match error {
            core::Error::Relations(error) => self.relations_refusal(error),
            core::Error::Sources(error) => self.source_refusal(*error),
            core::Error::Proof(error) => match error {
                P::NoFeasiblePlan(keys) => match self.refused(S::Proof, D::NoFeasiblePlan, keys) {
                    E::Refused {
                        relations,
                        unrestored_key_count: 0,
                        ..
                    } => E::NoFeasiblePlan(relations),
                    partial => partial,
                },
                P::FixedSource(key) => self.relation(&key).map_or_else(
                    |_| self.refused(S::Proof, D::FixedSource, vec![key]),
                    E::FixedSource,
                ),
                P::StateLimit(limit) => E::SearchLimit {
                    stage: S::Proof,
                    candidates: false,
                    limit,
                },
                P::CandidateLimit(limit) => E::SearchLimit {
                    stage: S::Proof,
                    candidates: true,
                    limit,
                },
            },
            core::Error::StageJoin(error) => {
                let (defect, keys) = match error {
                    core::StageJoinError::RepresentationChoices => {
                        (D::RepresentationChoices, Vec::new())
                    }
                    core::StageJoinError::LifecycleRequirements => {
                        (D::LifecycleRequirements, Vec::new())
                    }
                    core::StageJoinError::Constructibility(key) => {
                        (D::Constructibility, vec![key.relation])
                    }
                    core::StageJoinError::SourceRows(key) => (D::SourceRows, vec![key.relation]),
                    core::StageJoinError::DuplicateSourceTable(key) => {
                        (D::DuplicateSourceTable, vec![key.relation])
                    }
                    core::StageJoinError::UnnamedSourceTable(key) => {
                        (D::UnnamedSourceTable, vec![key.relation])
                    }
                };
                self.refused(S::Join, defect, keys)
            }
            core::Error::ProofSourceMismatch(key) => {
                self.refused(S::Join, D::ProofSource, vec![key.relation])
            }
            core::Error::StageCensus {
                stage,
                missing,
                unexpected,
            } => self.refused(
                match stage {
                    core::Stage::Source => S::Sources,
                    core::Stage::Proof => S::Proof,
                    core::Stage::Requirement => S::Requirements,
                    core::Stage::Case => S::Cases,
                },
                D::CensusMismatch,
                missing.into_iter().chain(unexpected).collect(),
            ),
            core::Error::Operations(error) => self.operation_refusal(*error),
            core::Error::Predicates(error) => self.predicate_refusal(error),
            core::Error::Constructibility(error) => self.construction_refusal(*error),
            core::Error::Lifecycle(error) => self.lifecycle_refusal(&error),
            core::Error::Requirements(error) => self.requirement_refusal(error),
            core::Error::Aggregate(error) => self.aggregate_refusal(*error),
            core::Error::Cases(error) => self.case_refusal(error),
            #[cfg(test)]
            core::Error::Lowering(_) => empty(S::Lowering),
            #[cfg(test)]
            core::Error::SourceMismatch | core::Error::ProgramMismatch => E::PublicationMismatch,
        }
    }
    fn operation_refusal(&self, error: crate::engine_operation::Error) -> E {
        use crate::{engine_operation::Error as O, engine_placement::Error as P};
        match error {
            O::Placement(error) => match *error {
                P::StateLimit(limit) => E::SearchLimit {
                    stage: S::Placement,
                    candidates: false,
                    limit,
                },
                P::CandidateLimit(limit) => E::SearchLimit {
                    stage: S::Placement,
                    candidates: true,
                    limit,
                },
                P::PlacementCensusMismatch {
                    missing,
                    unexpected,
                } => self.refused(
                    S::Placement,
                    D::CensusMismatch,
                    missing
                        .into_iter()
                        .chain(unexpected)
                        .map(|key| key.relation)
                        .collect(),
                ),
                P::NoEligibleCarrier(key) => {
                    self.refused(S::Placement, D::NoEligibleCarrier, vec![key.relation])
                }
                P::GlobalRelationHasOnlyLocalCarrier(key) => self.refused(
                    S::Placement,
                    D::GlobalRelationHasOnlyLocalCarrier,
                    vec![key.relation],
                ),
                P::UnconditionalRelationOnOptionalCarrier(key) => self.refused(
                    S::Placement,
                    D::UnconditionalRelationOnOptionalCarrier,
                    vec![key.relation],
                ),
                P::NonCanonicalCarrierAssignment(key) => self.refused(
                    S::Placement,
                    D::NonCanonicalCarrierAssignment,
                    vec![key.relation],
                ),
                P::MissingLayoutRequirement(key) => self.refused(
                    S::Placement,
                    D::MissingLayoutRequirement,
                    vec![key.relation],
                ),
                P::DuplicatePlacedCarrier { key, .. } => {
                    self.refused(S::Placement, D::DuplicatePlacedCarrier, vec![key.relation])
                }
                P::UnpermittedCarrierPlacement { key, .. } => self.refused(
                    S::Placement,
                    D::UnpermittedCarrierPlacement,
                    vec![key.relation],
                ),
                P::UnexpectedLayoutRequirement(keys) => E::Refused {
                    stage: S::Placement,
                    defect: D::UnexpectedLayoutRequirement,
                    relations: Vec::new(),
                    operations: Vec::new(),
                    unrestored_key_count: keys.len(),
                },
            },
            O::Coverage(error) => self.coverage_refusal(*error),
            O::Join(error) => self.join_refusal(error),
            O::Graph(error) => self.graph_refusal(error),
            O::Prerequisite(pair) => self.refused(
                S::Coverage,
                D::Prerequisite,
                vec![pair.0.relation, pair.1.relation],
            ),
            O::CaseFacts { key, .. } => {
                self.refused(S::Cases, D::CaseFactsMismatch, vec![key.relation])
            }
            O::CaseCensus {
                missing,
                unexpected,
            } => self.refused(
                S::Cases,
                D::CensusMismatch,
                missing
                    .into_iter()
                    .chain(unexpected)
                    .map(|key| key.relation)
                    .collect(),
            ),
        }
    }
    fn relations_refusal(&self, error: crate::engine_records::RelationRecordError) -> E {
        use crate::engine_records::RelationRecordError as R;
        let (defect, keys) = match error {
            R::ForeignProofOwner {
                relation,
                alternative_relation,
            } => (D::ForeignProofOwner, vec![relation, alternative_relation]),
            R::DuplicateRelation { relation } => (D::DuplicateRelation, vec![relation]),
            R::UnknownEndpoint { relation } => (D::UnknownEndpoint, vec![relation]),
            R::EscapesScope {
                prerequisite,
                dependent,
            } => (D::EscapesScope, vec![prerequisite, dependent]),
            R::DuplicateDependency {
                prerequisite,
                dependent,
                ..
            } => (D::DuplicateDependency, vec![prerequisite, dependent]),
            R::Cycle { components } => (
                D::Cycle,
                components.into_iter().flat_map(|row| row.members).collect(),
            ),
            R::CensusMismatch {
                missing,
                unexpected,
            } => (
                D::CensusMismatch,
                missing.into_iter().chain(unexpected).collect(),
            ),
        };
        self.refused(S::Relations, defect, keys)
    }

    fn source_refusal(&self, error: crate::engine_source::SourceError) -> E {
        use crate::engine_source::SourceError as F;
        let (defect, key) = match error {
            F::DuplicateOperand(key) => (D::DuplicateOperand, key.relation),
            F::MissingOperand(key) => (D::MissingOperand, key.relation),
            F::ForeignOperand(key) => (D::ForeignOperand, key.relation),
            F::DuplicateRow(key) => (D::DuplicateRow, key.relation),
            F::UndecidableRegion(operation) => {
                return self.operations.get(&operation).map_or_else(
                    || self.operation_defect(S::Sources, D::UndecidableRegion, vec![operation]),
                    |operation| E::UndecidableRegion(*operation),
                );
            }
        };
        self.refused(S::Sources, defect, vec![key])
    }
}
