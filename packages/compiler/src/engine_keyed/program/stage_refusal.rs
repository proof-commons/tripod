//! Exhaustive stage vocabulary restoration; no profile dispatch.
use super::Prepared;
use crate::keyed_plan::{Defect as D, KeyedPlanError as E, Stage as S};

impl Prepared<'_> {
    pub(super) fn operation_defect(
        &self,
        stage: S,
        defect: D,
        keys: Vec<crate::engine_records::OperationKey>,
    ) -> E {
        let mut unrestored_key_count = 0;
        let operations = keys
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .filter_map(|key| {
                self.operations.get(&key).map_or_else(
                    || {
                        unrestored_key_count += 1;
                        None
                    },
                    |value| Some(*value),
                )
            })
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        E::Refused {
            stage,
            defect,
            relations: Vec::new(),
            operations,
            unrestored_key_count,
        }
    }
    pub(super) fn predicate_refusal(&self, error: crate::engine_foundation::Error) -> E {
        use crate::engine_foundation::Error as F;
        let (defect, relation) = match error {
            F::UnknownExpression { relation, .. } => (D::UnknownExpression, relation),
            F::NonBoolean { relation, .. } => (D::NonBoolean, relation),
            F::OutsideScope { relation, .. } => (D::OutsideScope, relation),
        };
        self.refused(S::Predicates, defect, vec![relation])
    }
    pub(super) fn construction_refusal(
        &self,
        error: crate::engine_constructibility::RecordError,
    ) -> E {
        use crate::engine_constructibility::RecordError as C;
        let (defect, operations) = match error {
            C::DuplicateConstructibilityNode { node } => {
                (D::DuplicateConstructibilityNode, vec![node.operation])
            }
            C::DuplicateConstructibilityDependency { node } => {
                (D::DuplicateConstructibilityDependency, vec![node.operation])
            }
            C::UnknownConstructibilityNode { node } => {
                (D::UnknownConstructibilityNode, vec![node.operation])
            }
            C::ConstructibilityCycle { components } => (
                D::ConstructibilityCycle,
                components
                    .into_iter()
                    .flatten()
                    .map(|node| node.operation)
                    .collect(),
            ),
            C::ConstructibilityCrossOperationDependency { operation, node } => (
                D::ConstructibilityCrossOperationDependency,
                vec![operation, node.operation],
            ),
            C::PermissionlessPrivateDependency { operation, .. } => {
                (D::PermissionlessPrivateDependency, vec![operation])
            }
            C::ConstructibilityWitnessUnavailable { operation, .. } => {
                (D::ConstructibilityWitnessUnavailable, vec![operation])
            }
            C::SponsorDependencyEscaped { operation, .. } => {
                (D::SponsorDependencyEscaped, vec![operation])
            }
        };
        self.operation_defect(S::Constructibility, defect, operations)
    }
    pub(super) fn lifecycle_refusal(
        &self,
        error: &crate::engine_lifecycle::LifecycleRecordError,
    ) -> E {
        use crate::engine_lifecycle::LifecycleRecordError as L;
        let (defect, operation) = match error {
            L::MissingRepresentationChoice { operation, .. } => {
                (D::MissingRepresentationChoice, operation)
            }
            L::MissingLifecyclePath { exit, .. } => (D::MissingLifecyclePath, exit),
        };
        self.operation_defect(S::Lifecycle, defect, vec![*operation])
    }
    pub(super) fn requirement_refusal(
        &self,
        error: crate::engine_requirement::SelectionError,
    ) -> E {
        use crate::engine_requirement::SelectionError as R;
        let (defect, relation) = match error {
            R::Missing(relation) => (D::MissingSelection, relation),
            R::Unexpected(relation) => (D::UnexpectedSelection, relation),
            R::Unapproved(relation) => (D::UnapprovedSelection, relation),
            R::Duplicate(relation) => (D::DuplicateSelection, relation),
            R::OwnerRefusal(error) => return E::Compiler(error),
        };
        self.refused(S::Requirements, defect, vec![relation])
    }
    pub(super) fn aggregate_refusal(&self, error: crate::engine_requirement::Error) -> E {
        use crate::engine_requirement::Error as R;
        let (defect, relation) = match error {
            R::Capability { relation, .. } => (D::CapabilityMismatch, relation),
            R::Source { relation, .. } => (D::SourceMismatch, relation),
            R::Evidence { relation, .. } => (D::EvidenceMismatch, relation),
        };
        self.refused(S::Requirements, defect, relation.into_iter().collect())
    }
    pub(super) fn case_refusal(&self, error: crate::engine_cases::Error) -> E {
        use crate::engine_cases::Error as C;
        match error {
            C::MissingRepresentationChoice(choice) => self.operation_defect(
                S::Cases,
                D::MissingRepresentationChoice,
                vec![choice.operation],
            ),
            C::UnsupportedRepresentation { choice, .. } => self.operation_defect(
                S::Cases,
                D::UnsupportedRepresentation,
                vec![choice.operation],
            ),
            C::CaseCensusMismatch {
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
            C::DuplicateCase(key) => self.refused(S::Cases, D::DuplicateCase, vec![key.relation]),
            C::CaseFactsMismatch(key) => {
                self.refused(S::Cases, D::CaseFactsMismatch, vec![key.relation])
            }
            C::OwnerRefusal(error) => E::Compiler(error),
        }
    }
    pub(super) fn coverage_refusal(&self, error: crate::engine_coverage::Error) -> E {
        use crate::engine_coverage::Error as C;
        let (defect, keys) = match error {
            C::DuplicateRequirement(row) => (D::DuplicateRequirement, vec![row.key]),
            C::CensusMismatch {
                missing,
                unexpected,
            } => (
                D::CensusMismatch,
                missing.into_iter().chain(unexpected).collect(),
            ),
            C::MissingInactive(key) => (D::MissingInactive, vec![key]),
            C::UnexpectedRejection(key) => (D::UnexpectedRejection, vec![key]),
            C::MissingPositive { key, .. } => (D::MissingPositive, vec![key]),
            C::MissingNegative { key, .. } => (D::MissingNegative, vec![key]),
        };
        self.refused(
            S::Coverage,
            defect,
            keys.into_iter().map(|key| key.relation).collect(),
        )
    }
    pub(super) fn join_refusal(&self, error: crate::engine_coverage_join::Error) -> E {
        use crate::engine_coverage_join::Error as C;
        let (defect, key) = match error {
            C::UnexpectedCarrier(key) => (D::UnexpectedCarrier, key),
            C::MissingCarrier(key) => (D::MissingCarrier, key),
            C::MissingProjection(key) => (D::MissingProjection, key),
            C::UnexpectedLayout(key) => (D::UnexpectedLayout, key),
            C::MissingLayout(key) => (D::MissingLayout, key),
        };
        self.refused(S::Coverage, defect, vec![key.relation])
    }
    pub(super) fn graph_refusal(&self, error: crate::engine_coverage_graph::Error) -> E {
        use crate::engine_coverage_graph::Error as G;
        let (defect, symbols) = match error {
            G::UnknownSymbol(symbol) => (D::UnknownSymbol, vec![*symbol]),
            G::DuplicateSymbol(symbol) => (D::DuplicateSymbol, vec![*symbol]),
            G::EndpointClass(edge) => (D::EndpointClass, vec![edge.source, edge.target]),
            G::CrossOperation(edge) => (D::CrossOperation, vec![edge.source, edge.target]),
            G::DuplicateDependency(edge) => {
                (D::DuplicateDependency, vec![edge.source, edge.target])
            }
            G::Cycle(cycles) => (
                D::Cycle,
                cycles.into_iter().flat_map(|row| row.members).collect(),
            ),
        };
        let mut unrestored_key_count = 0;
        let symbols = symbols
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .filter_map(|key| {
                self.symbol(key).map_or_else(
                    |_| {
                        unrestored_key_count += 1;
                        None
                    },
                    Some,
                )
            })
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        E::CoverageGraph {
            defect,
            symbols,
            unrestored_key_count,
        }
    }
}
